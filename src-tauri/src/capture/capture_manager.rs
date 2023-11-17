use std::{sync::{atomic::{AtomicBool, Ordering}, Mutex, Arc, mpsc::{Receiver, channel}}, thread::JoinHandle};

use image::{Luma, ImageBuffer};
use tauri::AppHandle;
use tauri_specta::Event;

use crate::{events::CaptureManagerEvent, image::ImageMetadata};

use super::{types::{CaptureManagerInfo, CaptureManagerStatus, AdvancedCapture, AdvCapture, CaptureProgress}, detector::{DetectorController, DetectorStatus}, capture::CaptureError};

#[derive(Clone)]
pub struct CapturedImage {
    pub data: ImageBuffer<Luma<u16>, Vec<u16>>,
    pub metadata: ImageMetadata,
}

pub enum AdvCaptureMessage {
    CapturedImage(CapturedImage),
    CaptureCompleted(Vec<CapturedImage>),
    Progress(CaptureProgress),
    Stopped,
    Error,
}

pub struct CaptureManager {
    capture_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    detector_controller: Arc<Mutex<DetectorController>>,
    stop_signal: Arc<AtomicBool>,
    info: Arc<Mutex<CaptureManagerInfo>>,
}

impl CaptureManager {
    pub fn new(app: AppHandle) -> Self {
        let info =  Arc::new(Mutex::new(CaptureManagerInfo {
            status: CaptureManagerStatus::DetectorDisconnected,
            detector_info: { None },
        }));

        let detector_controller = DetectorController::new(Self::create_detector_callback(app, info.clone()));

        Self {
            capture_handle: Arc::new(Mutex::new(None)),
            detector_controller: Arc::new(Mutex::new(detector_controller)),
            stop_signal: Arc::new(AtomicBool::new(false)),
            info
        }
    }

    /// Handles updates from the detector controller on status of the detector
    fn create_detector_callback(app: AppHandle, info: Arc<Mutex<CaptureManagerInfo>>) -> impl FnMut(DetectorStatus) {
        move |status| {
            let mut info = info.lock().unwrap();
            match status {
                DetectorStatus::Available => {
                    if info.status == CaptureManagerStatus::DetectorDisconnected {
                        info.status = CaptureManagerStatus::Available;
                    }
                },
                DetectorStatus::Disconnected => {
                    info.status = CaptureManagerStatus::DetectorDisconnected;
                },
                _ => {},
            }

            CaptureManagerEvent(CaptureManagerInfo { 
                detector_info: None, 
                status: info.status.clone() 
            }).emit_all(&app).unwrap();
        }
    }

    pub fn run_capture(
        &mut self,
        app: AppHandle,
        capture: AdvancedCapture,
    ) -> Result<Receiver<AdvCaptureMessage>, CaptureError> {
        let mut capture_guard = self.capture_handle.lock().unwrap();

        if capture_guard.is_some() {
            Err(CaptureError::DetectorInUse)
        } else {
            self.info.lock().unwrap().status = CaptureManagerStatus::Capturing;
            self.emit_event(app.clone());

            let (tx, rx) = channel();
            let (done_tx, done_rx) = channel();

            self.stop_signal.store(false, Ordering::SeqCst);
            let stop_signal = self.stop_signal.clone();

            let detector_controller_clone = self.detector_controller.clone();
            let handle = std::thread::spawn(move || {
                capture.start(app, detector_controller_clone, tx, stop_signal.clone());
                done_tx.send(()).expect("Could not send completion signal");
            });

            *capture_guard = Some(handle);

            let capture_handle = self.capture_handle.clone();
            let info_clone = self.info.clone();

            std::thread::spawn(move || {
                if done_rx.recv().is_ok() {
                    info_clone.lock().unwrap().status = CaptureManagerStatus::Available;
                    *capture_handle.lock().unwrap() = None;
                }
            });

            Ok(rx)
        }
    }

    pub fn stop_capture(&self) {
        self.stop_signal.store(true, Ordering::SeqCst);
        let mut capture_guard = self.capture_handle.lock().unwrap();
        *capture_guard = None;
    }

    fn emit_event(&self, app: AppHandle) {
        CaptureManagerEvent(CaptureManagerInfo { 
            detector_info: None, 
            status: self.info.lock().unwrap().status.clone() 
        }).emit_all(&app).unwrap();
    }
}