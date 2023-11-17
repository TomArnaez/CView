use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    thread,
    time::Duration,
};

use log::info;
use serde::Serialize;
use specta::Type;
use tokio_util::sync::CancellationToken;

use crate::wrapper::SLDeviceRS;

use super::capture::{CaptureError, CaptureMessage, CaptureSetting};

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
    where F: FnMut(DetectorStatus) + Send + 'static, {
        let controller = DetectorController {
            detector: SLDeviceRS::new(),
            detector_status: Arc::new(Mutex::new(DetectorStatus::Disconnected)),
            cancellation_token: None,
        };

        Self::launch_heartbeat_thread::<F>(
            controller.detector.clone(),
            controller.detector_status.clone(),
            heartbeat_callback
        );

        controller
    }

    pub fn run_capture_with_handler<F>(&mut self, capture_settings: CaptureSetting, mut handler: F) -> Result<(), CaptureError>
    where
        F: FnMut(crate::capture::capture::CaptureMessage) -> bool {
            match *self.detector_status.lock().unwrap() {
                DetectorStatus::Disconnected => return Err(CaptureError::DetectorDisconnected),
                DetectorStatus::Capturing => return Err(CaptureError::DetectorInUse),
                _ => {}
            }

            self.detector.set_exposure_time(capture_settings.exp_time)?;
            self.detector.set_full_well(capture_settings.full_well)?;

            let token = CancellationToken::new();
            self.cancellation_token = Some(token.clone());
            let rx = capture_settings.capture_mode.start(self.detector.clone(), token)?;
            for recv in rx.iter() {
                if !handler(recv) {
                    break;
                }
            }

            Ok(())
    }

    pub fn run_capture(
        &mut self,
        capture_settings: CaptureSetting,
    ) -> Result<Receiver<CaptureMessage>, CaptureError> {
        if *self.detector_status.lock().unwrap() != DetectorStatus::Available {
            return Err(CaptureError::DetectorInUse);
        }
        self.detector.set_exposure_time(capture_settings.exp_time)?;

        let token = CancellationToken::new();
        self.cancellation_token = Some(token.clone());
        capture_settings
            .capture_mode
            .start(self.detector.clone(), token)

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
    )  where F: FnMut(DetectorStatus) + Send + 'static, {
        info!("Launching heartbeat thread");
        tauri::async_runtime::spawn(async move {
            loop {
                thread::sleep(Duration::from_millis(500));

                let mut detector_status = detector_status_mutex.lock().unwrap();

                match *detector_status {
                    DetectorStatus::Disconnected => {
                        if detector.open_camera(100).is_ok() {
                            info!("Connected to device");
                            *detector_status = DetectorStatus::Available;
                        } else {
                            info!("can't open camera");
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

#[cfg(test)]
mod tests {
    use std::{time::Duration, sync::{Arc, Mutex}};

    use crate::capture::{capture::{CaptureSettingBuilder, SequenceCapture}, test_utils::test_utils::setup_controller};

    #[test]
    fn run_sequence_capture() {
        let mut controller = setup_controller();

        let capture_settings =
        CaptureSettingBuilder::new(100, Box::new(SequenceCapture { num_frames: 10 })).build();

        let mut counter = 0;
        controller.run_capture_with_handler(capture_settings, |message| {
            match message {
                crate::capture::capture::CaptureMessage::CapturedImage(_) => counter += 1,
                crate::capture::capture::CaptureMessage::CaptureCancelled => panic!("Capture was unexpectedly cancelled"),
                crate::capture::capture::CaptureMessage::CaptureCompleted => return false,
                _ => (),
            }
            true
        }).unwrap();

        assert_eq!(counter, 10);
    }

    #[test]
    fn cancel_capture() {
        let controller = Arc::new(Mutex::new(setup_controller()));

        let capture_settings =
        CaptureSettingBuilder::new(100, Box::new(SequenceCapture { num_frames: 10 })).build();

        controller.lock().unwrap().run_capture_with_handler(capture_settings, |message| {
            match message {
                crate::capture::capture::CaptureMessage::CapturedImage(_) => controller.lock().unwrap().stop_capture(),
                crate::capture::capture::CaptureMessage::CaptureCompleted => panic!("Capture did not capture"),
                _ => {return true}
            }
            true
        }).unwrap();
    }

    #[test]
    fn cancel_capture_restart() {
        let controller = Arc::new(Mutex::new(setup_controller()));

        std::thread::sleep(Duration::from_secs(2));

        let capture_settings =
        CaptureSettingBuilder::new(100, Box::new(SequenceCapture { num_frames: 10 })).build();

        controller.lock().unwrap().run_capture_with_handler(capture_settings, |message| {
            match message {
                crate::capture::capture::CaptureMessage::CapturedImage(_) => controller.lock().unwrap().stop_capture(),
                crate::capture::capture::CaptureMessage::CaptureCompleted => panic!("Capture did not capture"),
                _ => {return true}
            }
            true
        }).unwrap();

        let capture_settings2 =
        CaptureSettingBuilder::new(100, Box::new(SequenceCapture { num_frames: 10 })).build();

        let mut counter = 0;
        controller.lock().unwrap().run_capture_with_handler(capture_settings2, |message| {
            match message {
                crate::capture::capture::CaptureMessage::CapturedImage(_) => counter += 1,
                crate::capture::capture::CaptureMessage::CaptureCancelled => panic!("Capture was unexpectedly cancelled"),
                crate::capture::capture::CaptureMessage::CaptureCompleted => return false,
                _ => (),
            }
            true
        }).unwrap();

        assert_eq!(counter, 10);
    }
}

unsafe impl Send for DetectorController {}
unsafe impl Sync for DetectorController {}
