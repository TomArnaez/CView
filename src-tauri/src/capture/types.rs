use std::{
    collections::HashMap,
    path::PathBuf,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use crate::{image::ImageHandler, wrapper::SLImageRs};

use super::{
    advanced_capture::{LiveCapture, MultiCapture, SignalAccumulationCapture, SmartCapture},
    detector::DetectorController,
};
use enum_dispatch::enum_dispatch;
use futures_core::Stream;
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
    NeedsDefectMaps,
    DetectorDisconnected,
}

#[enum_dispatch(AdvCapture)]
#[derive(Clone, Serialize, Deserialize, Type, Debug)]
#[serde(tag = "type")]
pub enum AdvancedCapture {
    SmartCapture,
    SignalAccumulationCapture,
    MultiCapture,
    LiveCapture,
}

#[enum_dispatch]
pub trait AdvCapture {
    fn start_stream<T: Runtime>(
        &self,
        app: AppHandle<T>,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        dark_maps: Arc<Mutex<HashMap<u32, SLImageRs>>>,
        defect_map: Arc<Mutex<Option<SLImageRs>>>,
        stop_signal: Arc<AtomicBool>,
    ) -> Pin<Box<dyn Stream<Item = ImageHandler> + Send>>;

    fn check_stop_signal(
        &self,
        stop_signal: &Arc<AtomicBool>,
        detector_controller: &mut DetectorController,
    ) -> bool {
        if stop_signal.load(Ordering::SeqCst) {
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
pub struct CaptureManagerEvent(pub CaptureManagerEventPayload);

#[derive(Debug, Clone, Serialize, Type, Event)]
pub struct CaptureManagerEventPayload {
    pub dark_maps: Vec<u32>,
    pub status: CaptureManagerStatus,
}

#[derive(Debug, Clone, Serialize, Type, Event)]
pub struct CaptureProgressEvent(pub CaptureProgress);
