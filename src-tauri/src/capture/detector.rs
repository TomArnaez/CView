use futures_core::stream::Stream;
use futures_util::StreamExt;
use log::{error, info, debug};
use serde::Serialize;
use specta::Type;
use std::{
    collections::HashMap,
    future::{self},
    pin::Pin,
    sync::{Arc, Mutex, atomic::AtomicBool},
    thread,
    time::Duration
};
use tokio_util::sync::CancellationToken;

use crate::wrapper::{SLDeviceRS, SLImageRs};

use super::capture::{CaptureError, CaptureSetting};

// Required for opening camera on a detector
const BUFFER_DEPTH: u32 = 100;
const HEARTBEAT_REFRESH_TIME_MILLIS: u64 = 100;

#[derive(PartialEq, Clone, Serialize, Debug, Type)]
pub enum DetectorStatus {
    Available,
    Disconnected,
    Capturing,
}

pub struct DetectorController {
    detector: SLDeviceRS,
    detector_status: Arc<Mutex<DetectorStatus>>,
    cancellation_token: Option<CancellationToken>,
}

impl DetectorController {
    pub fn new<F>(heartbeat_callback: F) -> Self
    where
        F: FnMut(DetectorStatus) + Send + 'static,
    {
        let controller = DetectorController {
            detector: SLDeviceRS::new(),
            detector_status: Arc::new(Mutex::new(DetectorStatus::Disconnected)),
            cancellation_token: None,
        };

        Self::launch_heartbeat_thread::<F>(
            controller.detector.clone(),
            controller.detector_status.clone(),
            heartbeat_callback,
        );

        controller
    }

    pub fn run_capture_stream(
        &mut self,
        capture_settings: CaptureSetting,
        dark_maps: Arc<Mutex<HashMap<u32, SLImageRs>>>,
        defect_map: Arc<Mutex<Option<SLImageRs>>>,
        stop_signal: Arc<AtomicBool>,
    ) -> Result<Pin<Box<dyn Stream<Item = SLImageRs> + Send>>, CaptureError> {
        let stream = capture_settings
            .capture_mode
            .stream_results(capture_settings.exp_time, self.detector.clone(), stop_signal.clone())?;

        let dark_maps_clone = dark_maps.clone();
        let stop_signal_clone = stop_signal.clone();

        Ok(stream.
            take_while(move |_| {
                future::ready(!stop_signal_clone.load(std::sync::atomic::Ordering::Relaxed))
            })
            .map(move |mut image| {
                if capture_settings.corrected {
                    if let Some(mut dark_map) = dark_maps_clone
                        .lock()
                        .unwrap()
                        .get_mut(&capture_settings.exp_time)
                    {
                        info!("Dark Correction Successful");
                        image.offset_correction(&mut dark_map, 300);
                    } else {
                        error!("Couldn't access dark map");
                    }

                    if let Some(ref mut defect_map) = *defect_map.lock().unwrap() {
                        info!("Defect correcting");
                        let mut out_image = SLImageRs::new(image.get_height(), image.get_width());

                        image.defect_correction(&mut out_image, defect_map).unwrap();
                        out_image
                    } else {
                        error!("Couldn't access defect map");
                        image
                    }
                } else {
                    image
                }
            })
            .boxed())
    }

    pub fn stop_capture(&mut self) {
        if let Some(token) = &self.cancellation_token {
            token.cancel();
            self.cancellation_token = None;
        }
    }

    fn launch_heartbeat_thread<F>(
        mut detector: SLDeviceRS,
        detector_status_mutex: Arc<Mutex<DetectorStatus>>,
        mut heartbeat_callback: F,
    ) where
        F: FnMut(DetectorStatus) + Send + 'static,
    {
        info!("Launching heartbeat thread");
        tauri::async_runtime::spawn(async move {
            loop {
                thread::sleep(Duration::from_millis(HEARTBEAT_REFRESH_TIME_MILLIS));

                let mut detector_status = detector_status_mutex.lock().unwrap();

                match *detector_status {
                    DetectorStatus::Disconnected => {
                        match detector.open_camera(BUFFER_DEPTH) {
                            Ok(_) => {
                                info!("Connected to device");
                                *detector_status = DetectorStatus::Available;
                            },
                            Err(e) => debug!("Error opening camera")
                        }
                    }
                    DetectorStatus::Available | DetectorStatus::Capturing => {
                        if !detector.is_connected() {
                            info!("Disconnected from device");
                            *detector_status = DetectorStatus::Disconnected;
                        }
                    }
                }

                heartbeat_callback(detector_status.clone());
            }
        });
    }
}

unsafe impl Send for DetectorController {}
unsafe impl Sync for DetectorController {}

#[cfg(test)]
mod tests {
    use std::pin::{self, Pin};
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex};
    use crate::capture::types::AdvCapture;
    use crate::wrapper::{ExposureModes, SLDeviceRS, SLImageRs};

    use super::DetectorController;
    use async_stream::stream;
    use futures::stream::{self, StreamExt}; // import StreamExt for chain method
    use futures::Stream;
    use futures_util::pin_mut;
    use tauri::Manager;

    fn create_app<R: tauri::Runtime>(mut builder: tauri::Builder<R>) -> tauri::App<R> {
        builder
            .setup(|app| {
                // do something
                Ok(())
            })
            .build(tauri::generate_context!())
            .expect("failed to build app")
    }

    fn test(count: u32, mut detector: SLDeviceRS) -> Pin<Box<dyn Stream<Item = SLImageRs>>> {
        let stream = stream! {
            println!("called");
            detector.set_exposure_time(100);
            detector.set_exposure_mode(ExposureModes::seq_mode);
            detector.set_number_frames(count);
            detector.go_live();
            detector.software_trigger();

            for frame_num in 0..count {
                let mut image = SLImageRs::new(
                    detector.image_height().unwrap(),
                    detector.image_width().unwrap(),
                );

                while detector
                    .read_buffer(&mut image, frame_num as u32, 100)
                    .is_err() {}
                yield image;
            }

            detector.go_unlive(true);
            println!("finished");
        };
        stream.boxed()
    }


    /*
    #[tokio::test]
    async fn run_capture_stream() {
        let app = create_app(tauri::test::mock_builder());
        let mut controller = DetectorController::new(|f| {});

        std::thread::sleep(Duration::from_secs(1));

        let stream_capture = Box::new(SequenceCapture { num_frames: 100 });
        let capture_settings = CaptureSettingBuilder::new(100, stream_capture).build();

        let mut stream = controller.run_capture_stream(capture_settings).unwrap();

        while let Some(mut item) = stream.next().await {
            println!("Item: {}", item.get_width());
        }
    }

    #[tokio::test]
    async fn test_signal_accum() {
        let app = create_app(tauri::test::mock_builder());
        let mut controller = Arc::new(Mutex::new(DetectorController::new(|f| {})));

        let signal_accum_capture = SignalAccumulationCapture {
            exp_times: vec![],
            frames_per_capture: 100,
        };
        std::thread::sleep(Duration::from_secs(1));

        let mut stream = signal_accum_capture.start_stream(
            app.handle().clone(),
            controller,
            Arc::new(AtomicBool::new(false)),
        );

        while let Some(image) = stream.next().await {
            println!("got image");
        }
    }

    #[tokio::test]
    async fn test_multi_capture() {
        let app = create_app(tauri::test::mock_builder());
        let controller = Arc::new(Mutex::new(DetectorController::new(|f| {})));

        let multi_capture = MultiCapture {
            exp_times: vec![100, 200],
            frames_per_capture: 10,
        };
        std::thread::sleep(Duration::from_secs(1));

        let mut stream = multi_capture.start_stream(
            app.handle().clone(),
            controller,
            Arc::new(AtomicBool::new(false)),
        );

        while let Some(image) = stream.next().await {
            println!("got image");
        }
    }

    #[tokio::test]
    async fn test_cancel_capture() {
        let app = create_app(tauri::test::mock_builder());
        let controller = Arc::new(Mutex::new(DetectorController::new(|f| {})));

        let multi_capture = MultiCapture {
            exp_times: vec![100, 200],
            frames_per_capture: 10,
        };
        std::thread::sleep(Duration::from_secs(1));

        let stop_signal = Arc::new(AtomicBool::new(false));

        let mut stream =
            multi_capture.start_stream(app.handle().clone(), controller, stop_signal.clone());

        while let Some(image) = stream.next().await {
            println!("got image");
            stop_signal.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
    */
}
