use std::{
    fmt,
    sync::mpsc::{channel, Receiver},
    thread,
    time::{Duration, Instant},
};

use log::{debug, error, info};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

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

pub enum CaptureMessage {
    CapturedImage(SLImageRs),
    CaptureCompleted,
    CaptureCancelled,
}

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
        }
    }
}

pub struct CaptureSettingBuilder {
    exp_time: u32,
    capture_mode: Box<dyn Capture + Send + 'static>,
    dds: bool,
    full_well: FullWellModesRS,
    binning_mode: BinningModesRS,
    pub roi: Option<Vec<u32>>,
}

impl CaptureSettingBuilder {
    pub fn new(exp_time: u32, capture_mode: Box<dyn Capture + Send + 'static>) -> Self {
        CaptureSettingBuilder {
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
        }
    }
}

pub trait Capture {
    fn start(
        &self,
        detector: SLDeviceRS,
        cancellation_token: CancellationToken,
    ) -> Result<Receiver<CaptureMessage>, CaptureError>;

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
    fn start(
        &self,
        mut detector: SLDeviceRS,
        cancellation_token: CancellationToken,
    ) -> Result<Receiver<CaptureMessage>, CaptureError> {
        info!("Capturing sequence");
        detector.set_exposure_mode(ExposureModes::seq_mode)?;
        detector.set_number_frames(self.num_frames as u32)?;
        detector.go_live()?;
        detector.software_trigger()?;

        let (tx, rx) = channel();
        let capture = self.clone();
        std::thread::spawn(move || {
            info!("Spawned Seqence Capture thread");
            for frame_num in 0..capture.num_frames {
                if cancellation_token.is_cancelled() {
                    println!("cancelled");
                    detector.go_unlive(true);
                    tx.send(CaptureMessage::CaptureCancelled);
                    break;
                }

                let mut image = SLImageRs::new(
                    detector.image_height().unwrap(),
                    detector.image_width().unwrap(),
                );
                let data_ptr = image.get_data_pointer(0);

                if detector
                    .read_buffer(data_ptr as *mut u8, frame_num as u32, 1000)
                    .is_err()
                {
                    error!("Failed to read buffer {frame_num}");
                } else {
                    match tx.send(CaptureMessage::CapturedImage(image)) {
                        Ok(_) => info!("Successfuly sent frame {frame_num}"),
                        Err(e) => error!("Error whilst sending frame {frame_num} with error {e}"),
                    };
                }
            }

            tx.send(CaptureMessage::CaptureCompleted);
            detector.go_unlive(true);
            info!("Finished sequence capture");
        });
        Ok(rx)
    }

    fn clone_box(&self) -> Box<dyn Capture + Send + 'static> {
        Box::new(self.clone())
    }
}

impl Capture for StreamCapture {
    fn start(
        &self,
        mut detector: SLDeviceRS,
        cancellation_token: CancellationToken,
    ) -> Result<Receiver<CaptureMessage>, CaptureError> {
        let start_time = Instant::now();

        let (tx, rx) = channel();
        let capture = self.clone();

        detector.start_stream(100);

        tauri::async_runtime::spawn(async move {
            while !cancellation_token.is_cancelled()
                && (capture.duration.is_none() || start_time.elapsed() < capture.duration.unwrap())
            {
                let mut image: SLImageRs = SLImageRs::new(
                    detector.image_height().unwrap(),
                    detector.image_width().unwrap(),
                );

                thread::sleep(Duration::from_millis(1));

                if detector.read_frame(image.get_data_pointer(0), true) {
                    match tx.send(CaptureMessage::CapturedImage(image)) {
                        Ok(_) => {
                            debug!("Image sent from sequence capture")
                        }
                        Err(e) => {
                            error!("Failed to data image frame through channel whilst streaming with error {}", e)
                        }
                    }
                }
            }

            detector.go_unlive(true);
            tx.send(CaptureMessage::CaptureCompleted);
            info!("Finished stream capture");
        });

        Ok(rx)
    }

    fn clone_box(&self) -> Box<dyn Capture + Send + 'static> {
        Box::new(self.clone())
    }
}
