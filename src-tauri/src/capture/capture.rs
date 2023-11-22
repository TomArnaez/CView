use std::{
    fmt,
    pin::Pin,
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread,
    time::{Instant, Duration},
};

use async_stream::stream;
use futures_core::stream::Stream;
use futures_util::StreamExt;
use log::error;
use serde::Serialize;
use specta::Type;
use thiserror::Error;

use crate::wrapper::{
    BinningModes, BinningModesRS, ExposureModes, FullWellModes, FullWellModesRS, InternalSLError,
    SLDeviceRS, SLImageRs,
};

use super::corrections::CorrectionError;

impl From<InternalSLError> for CaptureError {
    fn from(err: InternalSLError) -> Self {
        CaptureError::SLError(err)
    }
}

#[derive(Debug, Error, Type, Serialize)]
pub enum CaptureError {
    #[error("Detector is disconnected")]
    DetectorDisconnected,

    #[error("Detector currently in use")]
    DetectorInUse,

    #[error("Correction error: {0}")]
    File2Error(#[from] CorrectionError),

    #[error("Got internal SDK Error")]
    SLError(InternalSLError),

    #[error("Error")]
    Unknown,
}
unsafe impl Send for CaptureSetting {}
unsafe impl Sync for CaptureSetting {}

#[derive(Serialize, Type)]
pub struct CaptureSetting {
    pub exp_time: u32,
    #[specta(skip)]
    #[serde(skip)]
    pub capture_mode: Box<dyn Capture + Send + 'static>,
    pub dds: bool,
    pub full_well: FullWellModesRS,
    pub binning_mode: BinningModesRS,
    pub roi: Option<Vec<u32>>,
    pub corrected: bool,
}

// Manual implementation of Debug
impl fmt::Debug for CaptureSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CaptureSetting")
            .field("exp_time", &self.exp_time)
            .field("capture_mode", &format_args!("<capture mode>"))
            .field("dds", &self.dds)
            .field("full_well", &self.full_well)
            .field("binning_mode", &self.binning_mode)
            .field("roi", &self.roi)
            .finish()
    }
}

impl Clone for CaptureSetting {
    fn clone(&self) -> Self {
        CaptureSetting {
            exp_time: self.exp_time,
            capture_mode: self.capture_mode.clone_box(),
            dds: self.dds,
            full_well: self.full_well.clone(),
            binning_mode: self.binning_mode.clone(),
            roi: self.roi.clone(),
            corrected: self.corrected,
        }
    }
}

pub struct CaptureSettingBuilder {
    corrected: bool,
    exp_time: u32,
    capture_mode: Box<dyn Capture + Send + Sync + 'static>,
    dds: bool,
    full_well: FullWellModesRS,
    binning_mode: BinningModesRS,
    pub roi: Option<Vec<u32>>,
}

impl CaptureSettingBuilder {
    pub fn new(exp_time: u32, capture_mode: Box<dyn Capture + Send + Sync + 'static>) -> Self {
        CaptureSettingBuilder {
            corrected: true,
            exp_time,
            capture_mode,
            dds: false,
            full_well: FullWellModesRS {
                remote_ty: FullWellModes::High,
            },
            binning_mode: BinningModesRS(BinningModes::x11),
            roi: None,
        }
    }

    pub fn corrected(mut self, corrected: bool) -> Self {
        self.corrected = corrected;
        self
    }

    pub fn dds(mut self, dds: bool) -> Self {
        self.dds = dds;
        self
    }

    pub fn full_well(mut self, full_well: FullWellModesRS) -> Self {
        self.full_well = full_well;
        self
    }

    pub fn binning_mode(mut self, binning_mode: BinningModesRS) -> Self {
        self.binning_mode = binning_mode;
        self
    }

    pub fn build(self) -> CaptureSetting {
        CaptureSetting {
            exp_time: self.exp_time,
            capture_mode: self.capture_mode,
            binning_mode: self.binning_mode,
            full_well: self.full_well,
            dds: self.dds,
            roi: self.roi,
            corrected: self.corrected,
        }
    }
}

pub trait Capture {
    fn stream_results(
        &self,
        exp_time: u32,
        detector: SLDeviceRS,
        stop_signal: Arc<AtomicBool>
    ) -> Result<Pin<Box<dyn Stream<Item = SLImageRs> + Send>>, CaptureError>;

    fn clone_box(&self) -> Box<dyn Capture + Send + 'static>;
}

#[derive(Clone)]
pub struct SequenceCapture {
    pub num_frames: u32,
}

#[derive(Clone)]
pub struct StreamCapture {
    pub duration: Option<Duration>,
}

impl Capture for SequenceCapture {
    fn stream_results(
        &self,
        exp_time: u32,
        mut detector: SLDeviceRS,
        stop_signal: Arc<AtomicBool>
    ) -> Result<Pin<Box<dyn Stream<Item = SLImageRs> + Send>>, CaptureError> {
        println!("Setting up a sequence stream");
        let capture = self.clone();

        let stream = stream! {
            println!("Running sequence stream");
            detector.set_exposure_time(exp_time);
            detector.set_exposure_mode(ExposureModes::seq_mode);
            detector.set_number_frames(capture.num_frames as u32);
            detector.go_live();
            detector.software_trigger();

            for frame_num in 0.. capture.num_frames {
                if stop_signal.load(Ordering::Relaxed) {
                    break
                }
                let mut image = SLImageRs::new(
                    detector.image_height().unwrap(),
                    detector.image_width().unwrap(),
                );

                while detector.read_buffer(&mut image, frame_num as u32, 100).is_err() {}
                yield image;
            }

            detector.go_unlive(true);
        };
        Ok(stream.boxed())
    }

    fn clone_box(&self) -> Box<dyn Capture + Send + 'static> {
        Box::new(self.clone())
    }
}

impl Capture for StreamCapture {
    fn stream_results(
        &self,
        exp_time: u32,
        mut detector: SLDeviceRS,
        stop_signal: Arc<AtomicBool>
    ) -> Result<Pin<Box<dyn Stream<Item = SLImageRs> + Send>>, CaptureError> {
        let capture = self.clone();

        let stream = stream! {
            let start_time = Instant::now();
            detector.start_stream(exp_time);
            while !stop_signal.load(Ordering::Relaxed) && (capture.duration.is_none() || start_time.elapsed() < capture.duration.unwrap()) {
                let mut image: SLImageRs = SLImageRs::new(
                    detector.image_height().unwrap(),
                    detector.image_width().unwrap(),
                );

                thread::sleep(Duration::from_millis(1));
                if detector.read_frame(image.get_data_pointer(0), true) {
                    yield image;
                }
            }
            detector.go_unlive(true);
        };

        Ok(stream.boxed())
    }

    fn clone_box(&self) -> Box<dyn Capture + Send + 'static> {
        Box::new(self.clone())
    }
}
