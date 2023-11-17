use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
    Arc, Mutex,
};

use super::{
    advanced_capture::{
        DarkMapCapture, LiveCapture, MultiCapture, SignalAccumulationCapture,
        SmartCapture,
    },
    detector::DetectorController, capture_manager::AdvCaptureMessage,
};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Runtime};
use tauri_specta::Event;

#[derive(Type, Serialize, Debug, Clone)]
pub struct CaptureManagerInfo {
    pub detector_info: Option<DetectorInfo>,
    pub status: CaptureManagerStatus,
}

#[derive(Type, Serialize, Debug, Clone)]
pub struct DetectorInfo {
    pub interface: String,
}

#[derive(Type, Serialize, Debug, Clone, PartialEq)]
pub enum CaptureManagerStatus {
    Available,
    Capturing,
    DetectorDisconnected,
}

#[enum_dispatch(AdvCapture)]
#[derive(Clone, Serialize, Deserialize, Type, Debug)]
#[serde(tag = "type")]
pub enum AdvancedCapture {
    SmartCapture,
    SignalAccumulationCapture,
    MultiCapture,
    DarkMapCapture,
    LiveCapture,
}

#[enum_dispatch]
pub trait AdvCapture {
    fn start<T: Runtime>(
        &self,
        app: AppHandle<T>,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        tx: Sender<AdvCaptureMessage>,
        stop_signal: Arc<AtomicBool>,
    );

    fn check_stop_signal(
        &self,
        stop_signal: &Arc<AtomicBool>,
        tx: &Sender<AdvCaptureMessage>,
        detector_controller: &mut DetectorController,
    ) -> bool {
        if stop_signal.load(Ordering::SeqCst) {
            let _ = tx.send(AdvCaptureMessage::Stopped);
            detector_controller.stop_capture();
            true // Indicating that it should stop
        } else {
            false // Indicating that the capture should continue
        }
    }
}

#[derive(Clone, Serialize, Type, Debug)]
pub struct CaptureProgress {
    message: String,
    current_step: u32,
    total_steps: u32,
}

impl CaptureProgress {
    pub fn new(total_steps: u32, message: String) -> Self {
        CaptureProgress {
            message,
            current_step: 0,
            total_steps,
        }
    }

    pub fn update(&mut self, new_message: String) -> Self {
        self.message = new_message;
        self.current_step += 1;
        self.clone()
    }
}

#[derive(Debug, Clone, Serialize, Type, Event)]
pub struct CaptureProgressEvent(pub CaptureProgress);