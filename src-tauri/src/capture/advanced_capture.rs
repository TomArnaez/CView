use super::{
    capture::{
        CaptureMessage, CaptureSettingBuilder, SequenceCapture,
        StreamCapture,
    }, capture_manager::AdvCaptureMessage, detector::DetectorController, types::AdvCapture,
};

use crate::{
    capture::{corrections::apply_corrections, capture_manager::CapturedImage, types::CaptureProgress},
    image::{ImageMetadata, ImageMetadataBuilder, SmartCaptureData, ExtraData, SignalAccumulationData},
    operations,
    statistics::snr_threaded,
    wrapper::FullWellModesRS,
};

use log::{error, info};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{
    sync::{
        atomic::AtomicBool,
        mpsc::Sender,
        Arc, Mutex,
    }, path::PathBuf,
};
use tauri::{AppHandle, Runtime, Manager, path::BaseDirectory};


#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(tag = "type")]
pub struct SmartCapture {
    exp_times: Vec<u32>,
    frames_per_capture: u32,
    window_size: u32,
    median_filtered: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(tag = "type")]
pub struct SignalAccumulationCapture {
    exp_times: Vec<u32>,
    frames_per_capture: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(tag = "type")]
pub struct DarkMapCapture {
    pub exp_times: Vec<u32>,
    pub frames_per_capture: u32,
}

#[derive(Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type")]
pub struct DefectMapCapture {
    dark_exp_times: Vec<u32>,
    full_well_modes: Vec<FullWellModesRS>,
    frames_per_capture: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(tag = "type")]
pub struct MultiCapture {
    exp_times: Vec<u32>,
    frames_per_capture: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(tag = "type")]
pub struct LiveCapture {
    pub exp_time: u32,
}

impl AdvCapture for DefectMapCapture {
    fn start<T: Runtime>(
        &self,
        app: AppHandle<T>,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        tx: Sender<AdvCaptureMessage>,
        stop_signal: Arc<AtomicBool>,
    ) {
        self.dark_exp_times.iter().flat_map(|time| {
            self.full_well_modes.iter().map(move |mode| (time, mode))
        }).for_each(|(time, mode)| {
            let capture_settings =
            CaptureSettingBuilder::new(*time, Box::new(SequenceCapture { num_frames: 1 })).full_well(mode.clone()).build();

            let mut detector_controller = detector_controller_mutex.lock().unwrap();

            detector_controller.run_capture_with_handler(capture_settings.clone(), |message| {
                match message {
                    CaptureMessage::CapturedImage(mut image) => {
                        let metadata = ImageMetadataBuilder::new().capture_settings(capture_settings.clone()).build();

                        let defect_images_path = app.path().resolve(format!("DefectMapGeneration/Images/1510HS_1510_33_{time}ms_Dark{mode}_Mean.tif"), BaseDirectory::AppLocalData).unwrap();
                        match image.write_tiff_image(&PathBuf::from(format!("C:/Users/ThomasArnaez/1510HS_1510_33_{time}ms_Dark{mode}_Mean.tif"))) {
                            Ok(_) => info!("Saved defect map image succesfully"),
                            Err(_) => error!("Failed to save defect map image"),
                        }
                        true
                    },                    
                    CaptureMessage::CaptureCompleted => {
                        info!("defect capture completed");
                        true
                    },
                    CaptureMessage::CaptureCancelled => {
                        info!("defect capture cancelled");
                        false
                    },
                }
            }).unwrap();
        });
    }
}

impl AdvCapture for LiveCapture {
    fn start<T: Runtime>(
        &self,
        app: AppHandle<T>,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        tx: Sender<AdvCaptureMessage>,
        stop_signal: Arc<AtomicBool>,
    ) {
        info!("Starting Live Capture");

        let capture_settings =
            CaptureSettingBuilder::new(self.exp_time, Box::new(StreamCapture { duration: None })).build();

        let mut detector_controller = detector_controller_mutex.lock().unwrap();

        match detector_controller.run_capture(capture_settings.clone()) {
            Ok(rx) => {
                for recv in rx {
                    if self.check_stop_signal(&stop_signal, &tx, &mut *detector_controller) {
                        return;
                    }

                    match recv {
                        CaptureMessage::CapturedImage(mut image) => {
                            match apply_corrections(app.clone(), self.exp_time, &mut image) {
                                Ok(mut corrected_image) => {
                                    tx.send(AdvCaptureMessage::CapturedImage(CapturedImage {
                                        data: corrected_image.to_image_buffer(),
                                        metadata: ImageMetadata {
                                            capture_settings: Some(capture_settings.clone()),
                                            date_created: None,
                                            extra_info: None,
                                        },
                                    }));
                                }
                                Err(e) => {
                                    error!("Got error during Live Capture: {e}");
                                    tx.send(AdvCaptureMessage::Error);
                                    return;
                                }
                            }
                        }
                        CaptureMessage::CaptureCancelled => return,
                        _ => {}
                    }
                }
            }
            Err(_) => todo!(),
        }
    }
}

impl AdvCapture for DarkMapCapture {
    fn start<T: Runtime>(
        &self,
        app: AppHandle<T>,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        tx: Sender<AdvCaptureMessage>,
        stop_signal: Arc<AtomicBool>,
    ) {
        info!("Running Dark Map Capture");
        let mut dark_maps = Vec::new();

        for exp_time in self.exp_times.iter() {
            info!("Exposure Time: {}", exp_time);

            let mut detector_controller =
                detector_controller_mutex.lock().unwrap();

            if self.check_stop_signal(&stop_signal, &tx, &mut *detector_controller) {
                return;
            }

            let capture_settings = CaptureSettingBuilder::new(*exp_time, Box::new(SequenceCapture {
                num_frames: self.frames_per_capture,
            })).build();

            detector_controller.run_capture_with_handler(capture_settings.clone(), |message| {
                match message {
                    CaptureMessage::CapturedImage(mut image) => {
                        let metadata = ImageMetadataBuilder::new().capture_settings(capture_settings.clone()).build();

                        dark_maps.push(
                            (CapturedImage {
                                data: image.to_image_buffer(),
                                metadata,
                            }),
                        );

                        tx.send(AdvCaptureMessage::CapturedImage(dark_maps.last().unwrap().clone()));

                        true
                    },
                    CaptureMessage::CaptureCompleted => {
                        tx.send(AdvCaptureMessage::CaptureCompleted(dark_maps.to_vec()));
                        return true;
                    }
                    CaptureMessage::CaptureCancelled => false,
                }
            }).unwrap();
        }

        tx.send(AdvCaptureMessage::CaptureCompleted(Vec::new()));
    }
}

impl AdvCapture for SmartCapture {
    fn start<T: Runtime>(
        &self,
        app: AppHandle<T>,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        tx: Sender<AdvCaptureMessage>,
        stop_signal: Arc<AtomicBool>,
    ) {
        info!("Starting Smart Capture");
        let mut detector_controller = detector_controller_mutex.lock().unwrap();

        let mut images = Vec::new();
        let mut curr_snr: Option<f64> = None;

        for (i, exp_time) in self.exp_times.iter().enumerate() {
            if self.check_stop_signal(&stop_signal, &tx, &mut *detector_controller) {
                return;
            }
            info!("Capturing frame {i} with exposure time {exp_time}");

            let capture_settings = CaptureSettingBuilder::new(*exp_time, Box::new(SequenceCapture {
                num_frames: self.frames_per_capture,
            })).build();

            detector_controller.run_capture_with_handler(capture_settings.clone(), |message| {
                match message {
                    CaptureMessage::CapturedImage(mut image) => {
                        match apply_corrections(app.clone(), *exp_time, &mut image) {
                            Ok(mut corrected_image) => {
                                let mut image_buffer = corrected_image.to_image_buffer();

                                let metadata = ImageMetadataBuilder::new().capture_settings(capture_settings.clone()).build();

                                if self.median_filtered {
                                    image_buffer = operations::median_filter_threaded(
                                        &image_buffer,
                                        3,
                                        3,
                                    );
                                }

                                let mut captured_image = CapturedImage {
                                    data: image_buffer.clone(),
                                    metadata,
                                };

                                tx.send(AdvCaptureMessage::CapturedImage(
                                    captured_image.clone(),
                                ));

                                if let Ok(snr_results) =
                                snr_threaded(&mut captured_image.data, self.window_size)
                            {
                                info!("SNR of new frame: {}", snr_results.0);
                                
                                captured_image.metadata.extra_info = Some(ExtraData::SmartCaptureData(SmartCaptureData {
                                    signal_noise_ratio: snr_results.0,
                                    background_rect: snr_results.1,
                                    foreground_rect: snr_results.2
                                }));

                                if let Some(last_snr) = curr_snr {
                                    if snr_results.0 > last_snr {
                                        images.clear();
                                        curr_snr = Some(snr_results.0);
                                        images.push(captured_image);
                                    }
                                } else {
                                    curr_snr = Some(snr_results.0);
                                    images.push(captured_image);
                                }
                            } else {
                            }
                                true
                            },
                            Err(e) => {
                                error!("Got error {e} whilst applying corrections");
                                false
                            },
                        }
                    },
                    CaptureMessage::CaptureCompleted => true,
                    CaptureMessage::CaptureCancelled => true,
                }
            }).unwrap();
        }

        info!("Finished smart capture");
        tx.send(AdvCaptureMessage::CaptureCompleted(images));
    }
}

impl AdvCapture for SignalAccumulationCapture {
    fn start<T: Runtime>(
        &self,
        app: AppHandle<T>,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        tx: Sender<AdvCaptureMessage>,
        stop_signal: Arc<AtomicBool>,
    ) {
        info!("Starting Signal Accumulation Capture");

        let mut detector_controller = detector_controller_mutex.lock().unwrap();
        let mut images: Vec<CapturedImage> = Vec::new();

        let mut progress = CaptureProgress::new(self.exp_times.len() as u32, String::from("Starting Signal Accumulation Capture"));

        tx.send(AdvCaptureMessage::Progress(progress.clone()));

        for exp_time in &self.exp_times {
            if self.check_stop_signal(&stop_signal, &tx, &mut *detector_controller) {
                return;
            }

            tx.send(AdvCaptureMessage::Progress(progress.update(format!("Capturing images for {exp_time}"))));

            let capture_settings = CaptureSettingBuilder::new(*exp_time, Box::new(SequenceCapture {
                num_frames: self.frames_per_capture as u32,
            })).build();

            detector_controller.run_capture_with_handler(capture_settings.clone(), | message | {
                match message {
                    CaptureMessage::CapturedImage(mut image) => {

                        match apply_corrections(app.clone(), *exp_time, &mut image) {
                            Err(e) => {
                                error!("Error whilst applying corrections {}", e);
                                tx.send(AdvCaptureMessage::Error);
                                false
                            },
                            Ok(mut corrected_image) => {
                                let mut image_buffer = corrected_image.to_image_buffer();

                                let metadata = ImageMetadataBuilder::new()
                                .capture_settings(capture_settings.clone())
                                .extra_info(ExtraData::SignalAccumulationData(SignalAccumulationData { accumulated_exp_time: *exp_time}))
                                .build();

                                let captured_image = CapturedImage {
                                    data: corrected_image.to_image_buffer(),
                                    metadata
                                };

                                // Add the pixel values of last iamge to new image
                                if let Some(prev) = images.last() {
                                    for (current_pixel, prev_pixel) in
                                        image_buffer.pixels_mut().zip(prev.data.pixels())
                                    {
                                        let sum = current_pixel[0].saturating_add(prev_pixel[0]);
                                        // The 14-bit max value
                                        if sum < 16384 {
                                            current_pixel[0] = sum;
                                        } else {
                                            current_pixel[0] = 16383;
                                        }
                                    }
                                }

                                images.push(captured_image.clone());
                                tx.send(AdvCaptureMessage::CapturedImage(captured_image));
                                true
                            }
                        }
                    },
                    _ => {
                        false
                    }
                }
            }).unwrap();
        }

        tx.send(AdvCaptureMessage::CaptureCompleted(images));

        info!("Finished Signal Accumulation Capture");
    }
}

impl AdvCapture for MultiCapture {
    fn start<T: Runtime>(
        &self,
        app: AppHandle<T>,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        tx: Sender<AdvCaptureMessage>,
        stop_signal: Arc<AtomicBool>,
    ) {
        info!("Starting Multi Capture");

        let mut detector_controller = detector_controller_mutex.lock().unwrap();
        let mut images = Vec::new();

        for (i, exp_time) in self.exp_times.iter().enumerate() {
            if self.check_stop_signal(&stop_signal, &tx, &mut *detector_controller) {
                return;
            }

            let capture_mode = Box::new(SequenceCapture {
                num_frames: self.frames_per_capture as u32,
            });

            let capture_settings = CaptureSettingBuilder::new(*exp_time, capture_mode).build();

            detector_controller.run_capture_with_handler(capture_settings.clone(), |message| {
                match message {
                    CaptureMessage::CapturedImage(mut image) => {
                        info!("Got multi capture image");
                        match apply_corrections(app.clone(), *exp_time, &mut image) {
                            Err(_) => {
                                error!("Error whilst applying corrections with Multi Capture");
                                false
                            }
                            Ok(mut corrected_image) => {
                                let image_buffer = corrected_image.to_image_buffer();

                                let metadata = ImageMetadata {
                                    capture_settings: Some(capture_settings.clone()),
                                    date_created: None,
                                    extra_info: None,
                                };

                                let captured_image = CapturedImage {
                                    data: image_buffer,
                                    metadata,
                                };

                                images.push(captured_image.clone());
                                tx.send(AdvCaptureMessage::CapturedImage(captured_image));
                                true
                            }
                        }
                    },
                    CaptureMessage::CaptureCompleted => {
                        true
                    },
                    CaptureMessage::CaptureCancelled => {
                        false
                    },
                }
            }).unwrap();
        }

        tx.send(AdvCaptureMessage::CaptureCompleted(images));
    }

}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex, mpsc::channel, atomic::AtomicBool};

    use crate::{capture::{test_utils::test_utils::{create_app, setup_controller_handle}, types::AdvCapture, advanced_capture::SignalAccumulationCapture}, wrapper::{FullWellModesRS, FullWellModes}};

    use super::{DarkMapCapture, AdvCaptureMessage, DefectMapCapture, SmartCapture};

    #[test]
    fn dark_capture() {
        let app = create_app(tauri::test::mock_builder());
        let controller = Arc::new(Mutex::new(setup_controller_handle(app.handle().clone())));

        let dark_capture = DarkMapCapture {
            exp_times: vec![100, 200, 500],
            frames_per_capture: 10,
        };

        let (tx, rx) = channel();
        let stop_signal = Arc::new(AtomicBool::new(false));

        dark_capture.start(app.handle().clone(), controller, tx, stop_signal.clone());
        
        for recv in rx.iter() {
            match recv {
                AdvCaptureMessage::CapturedImage(image) => {
                    println!("got image!");
                },
                AdvCaptureMessage::CaptureCompleted(_) => {
                    println!("completed!");
                    break;
                },
                AdvCaptureMessage::Progress(_) => {},
                AdvCaptureMessage::Stopped => {},
                AdvCaptureMessage::Error => {},
            }
        }
    }

    #[test]
    fn defect_map_capture() {
        let app = create_app(tauri::test::mock_builder());
        let controller = Arc::new(Mutex::new(setup_controller_handle(app.handle().clone())));

        let defect_map_capture = DefectMapCapture {
            dark_exp_times: vec![100, 200, 300],
            full_well_modes: vec![FullWellModesRS { remote_ty: FullWellModes::High}],
            frames_per_capture: 1
        };

        let (tx, rx) = channel();
        let stop_signal = Arc::new(AtomicBool::new(false));

        defect_map_capture.start(app.handle().clone(), controller, tx, stop_signal.clone());
        
        for recv in rx.iter() {
            match recv {
                AdvCaptureMessage::CapturedImage(image) => {
                    println!("got image!");
                },
                AdvCaptureMessage::CaptureCompleted(_) => {
                    println!("completed!");
                    break;
                },
                AdvCaptureMessage::Progress(_) => {},
                AdvCaptureMessage::Stopped => {},
                AdvCaptureMessage::Error => {},
            }
        }
    }

    #[test]
    fn smart_capture() {
        let app = create_app(tauri::test::mock_builder());
        let controller = Arc::new(Mutex::new(setup_controller_handle(app.handle().clone())));

        let smart_capture = SmartCapture {
            exp_times: vec![100, 200, 300],
            frames_per_capture: 10,
            median_filtered: false,
            window_size: 5
        };

        let (tx, rx) = channel();
        let stop_signal = Arc::new(AtomicBool::new(false));
        smart_capture.start(app.handle().clone(), controller, tx, stop_signal.clone());
        for recv in rx.iter() {
            match recv {
                AdvCaptureMessage::CapturedImage(image) => {
                    println!("got image!");
                },
                AdvCaptureMessage::CaptureCompleted(_) => {
                    println!("completed!");
                    break;
                },
                AdvCaptureMessage::Progress(_) => {},
                AdvCaptureMessage::Stopped => {},
                AdvCaptureMessage::Error => {},
            }
        }
    }

    #[test]
    fn signal_accumulation_captue() {
        let app = create_app(tauri::test::mock_builder());
        let controller = Arc::new(Mutex::new(setup_controller_handle(app.handle().clone())));

        let signal_accumulation_capture = SignalAccumulationCapture {
            exp_times: vec![100, 200, 300],
            frames_per_capture: 10
        };

        let (tx, rx) = channel();
        let stop_signal = Arc::new(AtomicBool::new(false));

        signal_accumulation_capture.start(app.handle().clone(), controller, tx, stop_signal.clone());

        for recv in rx.iter() {
            match recv {
                AdvCaptureMessage::CapturedImage(image) => {
                    println!("got image!");
                },
                AdvCaptureMessage::CaptureCompleted(_) => {
                    println!("completed!");
                    break;
                },
                AdvCaptureMessage::Progress(_) => {},
                AdvCaptureMessage::Stopped => {},
                AdvCaptureMessage::Error => {},
            }
        }
    }
}