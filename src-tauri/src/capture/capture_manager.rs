use std::{
    collections::HashMap,
    fmt, fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

use async_stream::stream;
use futures::stream::{self, iter, Stream, StreamExt};
use futures_util::pin_mut;
use log::{error, info};
use regex::Regex;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

use crate::{
    image::{ImageHandler, ImageMetadata},
    wrapper::{self, *},
};

use super::{
    capture::{Capture, CaptureError, CaptureSettingBuilder, SequenceCapture},
    detector::{DetectorController, DetectorStatus},
    types::{
        AdvCapture, AdvancedCapture, CaptureManagerEvent, CaptureManagerEventPayload,
        CaptureManagerInfo, CaptureManagerStatus, CaptureProgress,
    },
};

pub struct CaptureManager {
    capture_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    detector_controller: Arc<Mutex<DetectorController>>,
    stop_signal: Arc<AtomicBool>,
    info: Arc<Mutex<CaptureManagerInfo>>,
    dark_maps: Arc<Mutex<HashMap<u32, SLImageRs>>>,
    defect_map: Arc<Mutex<Option<SLImageRs>>>,
    dark_map_path: PathBuf,
    defect_map_path: PathBuf,
}

impl CaptureManager {
    pub fn new<T: Runtime>(app: AppHandle<T>) -> Self {
        let local_data = app.path().app_local_data_dir().unwrap();

        let dark_map_path = local_data.join("DarkMaps");
        let mut defect_map_path = local_data.join("DefectMap");
        fs::create_dir(&dark_map_path);
        fs::create_dir(&defect_map_path);
        let dark_maps = Arc::new(Mutex::new(read_dark_maps(&dark_map_path)));
        let defect_map = Arc::new(Mutex::new(read_defect_map(
            &defect_map_path.join("GlobalDefectMap.tif"),
        )));

        let info = Arc::new(Mutex::new(CaptureManagerInfo {
            status: CaptureManagerStatus::DetectorDisconnected,
            detector_info: { None },
        }));

        let detector_controller = DetectorController::new(Self::create_detector_callback(
            app.clone(),
            dark_maps.clone(),
            info.clone(),
        ));

        Self {
            capture_handle: Arc::new(Mutex::new(None)),
            detector_controller: Arc::new(Mutex::new(detector_controller)),
            stop_signal: Arc::new(AtomicBool::new(false)),
            info,
            dark_maps,
            defect_map,
            dark_map_path,
            defect_map_path,
        }
    }

    pub async fn generate_dark_maps(&self, exp_times: Vec<u32>, num_frames: u32) {
        stream::iter(exp_times)
            .then(|exp_time| {
                let detector_controller = self.detector_controller.clone();
                let dark_maps = self.dark_maps.clone();
                let defect_map = self.defect_map.clone();

                async move {
                    let capture_settings = CaptureSettingBuilder::new(
                        exp_time,
                        Box::new(SequenceCapture { num_frames }),
                    )
                    .corrected(false)
                    .build();

                    let mut average_image = SLImageRs::new_depth(1031, 1536, num_frames);

                    let mut enumerated_stream = detector_controller
                        .lock()
                        .unwrap()
                        .run_capture_stream(
                            capture_settings.clone(),
                            dark_maps.clone(),
                            defect_map.clone(),
                        )
                        .expect("Failed to run capture stream")
                        .enumerate();

                    while let Some((index, mut image)) = enumerated_stream.next().await {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                average_image.get_data_pointer(index as u32),
                                image.get_data_pointer(0),
                                (image.get_width() * image.get_width() * 2) as usize,
                            );
                        }
                    }

                    average_image = average_image.get_average_image();

                    average_image
                        .write_tiff_image(
                            &self.dark_map_path.join(format!("DarkMap_{exp_time}.tif")),
                        )
                        .unwrap();

                    dark_maps.lock().unwrap().insert(exp_time, average_image);
                }
            })
            .collect::<Vec<_>>()
            .await;
    }

    pub async fn generate_defect_map<T: Runtime>(
        &self,
        app: AppHandle<T>,
        dark_exp_times: Vec<u32>,
        num_frames: u32,
    ) {
        stream::iter(dark_exp_times)
            .flat_map(|exp_time| {
                let full_well_modes = [
                    FullWellModesRS {
                        remote_ty: crate::wrapper::FullWellModes::High,
                    },
                    FullWellModesRS {
                        remote_ty: crate::wrapper::FullWellModes::Low,
                    },
                ];

                stream::iter(
                    full_well_modes
                        .into_iter()
                        .map(move |full_well_mode| (exp_time, full_well_mode)),
                )
            })
            .then(|(exp_time, full_well_mode)| {
                let detector_controller = self.detector_controller.clone();
                let dark_maps = self.dark_maps.clone();
                let defect_map = self.defect_map.clone();

                async move {
                    let capture_settings = CaptureSettingBuilder::new(
                        exp_time,
                        Box::new(SequenceCapture { num_frames }),
                    )
                    .full_well(full_well_mode.clone())
                    .build();

                    let mut average_image = SLImageRs::new_depth(1536, 1031, num_frames);

                    let mut enumerated_stream = detector_controller
                        .lock()
                        .unwrap()
                        .run_capture_stream(capture_settings.clone(), dark_maps, defect_map)
                        .expect("Failed to run capture stream")
                        .enumerate();

                    while let Some((index, mut image)) = enumerated_stream.next().await {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                average_image.get_data_pointer(index as u32),
                                image.get_data_pointer(0),
                                (image.get_width() * image.get_width() * 2) as usize,
                            );
                        }
                    }

                    let dir = self
                        .defect_map_path
                        .join(format!("Test_Image_{exp_time}_{full_well_mode}.tif"));

                    info!("Saving defect map gen image {}", dir.display());

                    average_image.write_tiff_image(&dir);
                }
            })
            .collect::<Vec<_>>()
            .await;
    }

    fn create_detector_callback<T: Runtime>(
        app: AppHandle<T>,
        dark_maps: Arc<Mutex<HashMap<u32, SLImageRs>>>,
        info: Arc<Mutex<CaptureManagerInfo>>,
    ) -> impl FnMut(DetectorStatus) {
        move |status| {
            let mut info = info.lock().unwrap();
            match status {
                DetectorStatus::Available => {
                    if info.status == CaptureManagerStatus::DetectorDisconnected {
                        info.status = CaptureManagerStatus::Available;
                    }
                }
                DetectorStatus::Disconnected => {
                    info.status = CaptureManagerStatus::DetectorDisconnected;
                }
                _ => {}
            }

            CaptureManagerEvent(CaptureManagerEventPayload {
                dark_maps: dark_maps.lock().unwrap().keys().cloned().collect(),
                status: info.status.clone(),
            })
            .emit_all(&app)
            .unwrap();
        }
    }

    // Tell the capture manager the stream has concluded, and make available again
    fn wrap_stream<S, T>(
        input_stream: S,
        info: Arc<Mutex<CaptureManagerInfo>>,
    ) -> impl Stream<Item = T>
    where
        S: Stream<Item = T> + Unpin,
        T: Unpin,
    {
        let c = stream! {
            pin_mut!(input_stream);
            while let Some(item) = input_stream.next().await {
                yield item;
            }
            info.lock().unwrap().status = CaptureManagerStatus::Available;
        };
        return c;
    }

    pub fn start_capture<T: Runtime>(
        &mut self,
        app: AppHandle<T>,
        capture: AdvancedCapture,
    ) -> Result<impl Stream<Item = ImageHandler>, CaptureError> {
        if self.info.lock().unwrap().status != CaptureManagerStatus::Available {
            return Err(CaptureError::DetectorDisconnected);
        }

        self.emit_event(app.clone());
        self.info.lock().unwrap().status = CaptureManagerStatus::Capturing;

        self.stop_signal.store(false, Ordering::SeqCst);
        let stop_signal = self.stop_signal.clone();

        let stream = capture.start_stream(
            app,
            self.detector_controller.clone(),
            self.dark_maps.clone(),
            self.defect_map.clone(),
            stop_signal,
        );

        Ok(Self::wrap_stream(stream, self.info.clone()))
    }

    pub fn stop_capture(&self) {
        self.stop_signal.store(true, Ordering::SeqCst);
        let mut capture_guard = self.capture_handle.lock().unwrap();
        *capture_guard = None;
    }

    fn emit_event<T: Runtime>(&self, app: AppHandle<T>) {
        CaptureManagerEvent(CaptureManagerEventPayload {
            dark_maps: self.dark_maps.lock().unwrap().keys().cloned().collect(),
            status: self.info.lock().unwrap().status.clone(),
        })
        .emit_all(&app)
        .unwrap();
    }
}

pub fn read_dark_maps(path: &PathBuf) -> HashMap<u32, SLImageRs> {
    info!("Looking for dark map resources at {}", path.display());

    let mut dark_maps = HashMap::new();
    let regex = Regex::new(r"DarkMap_(\d+)ms\.tif").expect("Failed to compile regex");

    let paths = match fs::read_dir(path) {
        Ok(paths) => paths,
        Err(err) => {
            error!("Failed to read directory: {:?}", err);
            return dark_maps;
        }
    };

    for path in paths.filter_map(Result::ok) {
        let file_name = path.file_name();
        let file_name = match file_name.to_str() {
            Some(name) => name,
            None => continue,
        };

        if let Some(captures) = regex.captures(file_name) {
            if let Some(dark_map_exp_str) = captures.get(1) {
                match dark_map_exp_str.as_str().parse::<u32>() {
                    Ok(exp_time) => {
                        info!(
                            "Found dark map with exp time {} at {}",
                            exp_time,
                            path.path().display()
                        );
                        let mut image = SLImageRs::new(1536, 1031);
                        image.read_tiff_image(&path.path());
                        dark_maps.insert(exp_time, image);
                    }
                    Err(err) => error!("Failed to parse exposure time: {:?}", err),
                }
            }
        }
    }

    dark_maps
}

fn read_defect_map(path: &PathBuf) -> Option<SLImageRs> {
    info!("Looking for defect map resources at {}", path.display());
    if path.exists() {
        let mut image = SLImageRs::new(1, 1);
        image.read_tiff_image(&path);
        info!("Found defect map at {}", path.display());
        Some(image)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use futures_util::{pin_mut, StreamExt};
    use tauri::Manager;

    use crate::capture::{
        advanced_capture::{MultiCapture, SmartCapture},
        capture_manager::CaptureManager,
        test_utils::test_utils::create_app,
        types::AdvancedCapture,
    };

    use super::read_dark_maps;

    #[test]
    fn test_read_defects() {
        let app = create_app(tauri::test::mock_builder());
        read_dark_maps(
            &app.handle()
                .clone()
                .path()
                .app_local_data_dir()
                .unwrap()
                .join("DarkMaps"),
        );
    }

    #[tokio::test]
    async fn test_dark_map_generate() {
        let app = create_app(tauri::test::mock_builder());
        let capture_manager = Arc::new(Mutex::new(CaptureManager::new(app.handle().clone())));

        std::thread::sleep(Duration::from_secs(2));

        capture_manager
            .lock()
            .unwrap()
            .generate_dark_maps(vec![100, 200, 300], 10)
            .await;
    }

    #[tokio::test]
    async fn test_defect_map_gen() {
        let app = create_app(tauri::test::mock_builder());
        let capture_manager = Arc::new(Mutex::new(CaptureManager::new(app.handle().clone())));

        std::thread::sleep(Duration::from_secs(2));

        capture_manager
            .lock()
            .unwrap()
            .generate_defect_map(app.handle().clone(), vec![100, 200, 300], 10)
            .await;
    }

    #[tokio::test]
    async fn test_capture_manager() {
        let app = create_app(tauri::test::mock_builder());
        let capture_manager = Arc::new(Mutex::new(CaptureManager::new(app.handle().clone())));

        std::thread::sleep(Duration::from_secs(2));
        let smart_capture = AdvancedCapture::SmartCapture(SmartCapture {
            exp_times: vec![100, 200, 300],
            frames_per_capture: 10,
            median_filtered: false,
            window_size: 5,
        });

        let stream = capture_manager
            .lock()
            .unwrap()
            .start_capture(app.handle().clone(), smart_capture)
            .unwrap();

        pin_mut!(stream);
        while let Some(image) = stream.next().await {
            println!("{}", "got image");
        }
    }

    #[tokio::test]
    async fn test_capture_status() {
        let app = create_app(tauri::test::mock_builder());
        let capture_manager = Arc::new(Mutex::new(CaptureManager::new(app.handle().clone())));

        let multi_capture = AdvancedCapture::MultiCapture {
            0: MultiCapture {
                exp_times: vec![100, 200],
                frames_per_capture: 10,
            },
        };

        std::thread::sleep(Duration::from_secs(1));

        let stream = capture_manager
            .lock()
            .unwrap()
            .start_capture(app.handle().clone(), multi_capture)
            .unwrap();

        let capture_manager_clone = capture_manager.clone();
        let app_handle_clone = app.handle().clone();

        tauri::async_runtime::spawn(async move {
            std::thread::sleep(Duration::from_secs(1));
            let multi_capture = AdvancedCapture::MultiCapture {
                0: MultiCapture {
                    exp_times: vec![100, 200],
                    frames_per_capture: 10,
                },
            };
            assert!(capture_manager_clone
                .lock()
                .unwrap()
                .start_capture(app_handle_clone, multi_capture)
                .is_err());
        });

        pin_mut!(stream);
    }
}
