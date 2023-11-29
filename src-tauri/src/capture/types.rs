use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
        Arc,
    },
};

use crate::image::ImageHandler;

use super::{
    advanced_capture::{
        DarkMapCapture, DefectMapCapture, LiveCapture, MultiCapture, SignalAccumulationCapture,
        SmartCapture,
    },
    capture_manager::CorrectionMaps,
    detector::DetectorController,
};

use enum_dispatch::enum_dispatch;
use futures_core::Stream;
use serde::{Deserialize, Serialize};
use specta::Type;
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
    Capturing(AdvancedCapture),
    DarkMapsRequired,
    DefectMapsRequired,
    DetectorDisconnected,
}

#[enum_dispatch(AdvCapture)]
#[derive(Clone, Serialize, Deserialize, Type, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum AdvancedCapture {
    SmartCapture,
    SignalAccumulationCapture,
    MultiCapture,
    LiveCapture,
    DarkMapCapture,
    DefectMapCapture,
}

pub enum CaptureStreamItem {
    Image(ImageHandler),
    Progress(CaptureProgress),
    CaptureResult(Vec<ImageHandler>),
}

#[derive(Clone, Serialize, Type, Debug)]
pub struct CaptureProgress {
    message: String,
    current_step: u32,
    total_steps: u32,
}

#[enum_dispatch]
pub trait AdvCapture {
    fn start_stream(
        &self,
        detector_controller_mutex: DetectorController,
        correction_maps: &CorrectionMaps,
        progress_tx: Sender<CaptureProgress>,
    ) -> Pin<Box<dyn Stream<Item = CaptureStreamItem> + Send>>;

    fn check_stop_signal(
        &self,
        stop_signal: &Arc<AtomicBool>,
        detector_controller: &mut DetectorController,
    ) -> bool {
        if stop_signal.load(Ordering::SeqCst) {
            true // Indicating that it should stop
        } else {
            false // Indicating that the capture should continue
        }
    }
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
