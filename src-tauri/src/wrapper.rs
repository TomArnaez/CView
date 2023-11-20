use autocxx::prelude::*;
use autocxx::WithinUniquePtr;
use image::ImageBuffer;
use image::Luma;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use specta::Type;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

autocxx::include_cpp! {
    #include "SLDevice.h"
    #include "SLImage.h"
    #include "Defs.h"
    safety!(unsafe_ffi)
    generate_pod!("SpectrumLogic::SLError")
    generate!("SpectrumLogic::SLImage")
    generate!("SpectrumLogic::SLDevice")
    generate!("SpectrumLogic::SLErrorToString")
    generate!("SpectrumLogic::LogTraceLevel")
}

pub use ffi::SpectrumLogic::*;

unsafe impl Send for SLDevice {}
unsafe impl Sync for SLDevice {}
unsafe impl Send for SLImage {}
unsafe impl Sync for SLImage {}

#[derive(Clone)]
pub struct SLDeviceRS {
    device: Arc<Mutex<UniquePtr<SLDevice>>>,
}

impl SLDeviceRS {
    pub fn new() -> Self {
        Self {
            device: Arc::new(Mutex::new(
                SLDevice::new(DeviceInterface::USB, autocxx::c_int(1), "", "", "")
                    .within_unique_ptr(),
            )),
        }
    }

    pub fn set_log_trace_level(
        &mut self,
        sdk_log_trace_level: LogTraceLevel,
        internal_log_trace_level: LogTraceLevel,
    ) {
        let mut lock = self.device.lock().unwrap();
        lock.pin_mut()
            .SetLogTraceLevel(sdk_log_trace_level, internal_log_trace_level);
    }

    pub fn image_width(&mut self) -> Result<u32, ()> {
        let mut lock = self.device.lock().unwrap();
        let width: i32 = lock.pin_mut().GetImageXDim().into();
        match width == -1 {
            false => return Ok(width as u32),
            true => return Err(()),
        }
    }

    pub fn image_height(&mut self) -> Result<u32, ()> {
        let mut lock = self.device.lock().unwrap();
        let height: i32 = lock.pin_mut().GetImageYDim().into();
        match height == -1 {
            false => return Ok(height as u32),
            true => return Err(()),
        }
    }

    pub fn set_exposure_mode(&mut self, ex_mode: ExposureModes) -> Result<(), InternalSLError> {
        let mut lock = self.device.lock().unwrap();
        match lock.pin_mut().SetExposureMode(ex_mode) {
            SLError::SL_ERROR_SUCCESS => Ok(()),
            err => Err(err.into()),
        }
    }

    pub fn set_exposure_time(&mut self, exp_time: u32) -> Result<(), InternalSLError> {
        let mut lock = self.device.lock().unwrap();
        match lock
            .pin_mut()
            .SetExposureTime(autocxx::c_int(exp_time as i32))
        {
            SLError::SL_ERROR_SUCCESS => Ok(()),
            err => Err(err.into()),
        }
    }

    pub fn set_number_frames(&mut self, frame_count: u32) -> Result<(), InternalSLError> {
        let mut lock = self.device.lock().unwrap();
        match lock
            .pin_mut()
            .SetNumberOfFrames(autocxx::c_int(frame_count as i32))
        {
            SLError::SL_ERROR_SUCCESS => Ok(()),
            err => Err(err.into()),
        }
    }

    pub fn set_full_well(&mut self, full_well: FullWellModesRS) -> Result<(), InternalSLError> {
        let mut lock = self.device.lock().unwrap();
        match lock.pin_mut().SetFullWell(full_well.remote_ty) {
            SLError::SL_ERROR_SUCCESS => Ok(()),
            err => Err(err.into()),
        }
    }

    pub fn open_camera(&mut self, buffer_depth: u32) -> Result<(), InternalSLError> {
        let mut lock = self.device.lock().unwrap();

        match lock
            .pin_mut()
            .OpenCamera(autocxx::c_int(buffer_depth as i32))
        {
            SLError::SL_ERROR_SUCCESS => Ok(()),
            err => Err(err.into()),
        }
    }

    pub fn is_connected(&mut self) -> bool {
        let mut lock = self.device.lock().unwrap();
        lock.pin_mut().IsConnected()
    }

    pub fn start_stream(&mut self, exp_time: u32) -> Result<(), InternalSLError> {
        let mut lock = self.device.lock().unwrap();
        match lock.pin_mut().StartStream(autocxx::c_int(exp_time as i32)) {
            SLError::SL_ERROR_SUCCESS => Ok(()),
            err => Err(err.into()),
        }
    }

    pub fn go_live(&mut self) -> Result<(), InternalSLError> {
        let mut lock = self.device.lock().unwrap();

        match lock.pin_mut().GoLive() {
            SLError::SL_ERROR_SUCCESS => Ok(()),
            err => Err(err.into()),
        }
    }

    pub fn go_unlive(&mut self, wipe_stack: bool) -> Result<(), InternalSLError> {
        let mut lock = self.device.lock().unwrap();

        match lock.pin_mut().GoUnLive(wipe_stack) {
            SLError::SL_ERROR_SUCCESS => Ok(()),
            err => Err(err.into()),
        }
    }

    pub fn software_trigger(&mut self) -> Result<(), InternalSLError> {
        let mut lock = self.device.lock().unwrap();

        match lock.pin_mut().SoftwareTrigger() {
            SLError::SL_ERROR_SUCCESS => Ok(()),
            err => Err(err.into()),
        }
    }

    pub fn read_buffer(
        &mut self,
        buffer: &mut SLImageRs,
        buf_num: u32,
        timeout: u32,
    ) -> Result<(), InternalSLError> {
        unsafe {
            let mut lock = self.device.lock().unwrap();

            match lock.pin_mut().ReadBuffer(
                buffer.get_data_pointer(0),
                autocxx::c_int(buf_num as i32),
                autocxx::c_int(timeout as i32),
            ) {
                SLError::SL_ERROR_SUCCESS => Ok(()),
                err => Err(err.into()),
            }
        }
    }

    /*
    pub fn get_model_info(&mut self) -> UniquePtr<ModelInfo> {
        self.device.pin_mut().GetModelInfo().within_unique_ptr()
    }
    */

    pub fn read_frame(&mut self, buffer: *mut u8, read_oldest_first: bool) -> bool {
        let mut lock = self.device.lock().unwrap();

        unsafe {
            lock.pin_mut()
                .ReadFrame(buffer as *mut c_ushort, read_oldest_first)
        }
    }
}

pub struct SLImageRs {
    image: UniquePtr<SLImage>,
}
unsafe impl Send for SLImageRs {}
unsafe impl Sync for SLImageRs {}

impl SLImageRs {
    fn get_underlying(&mut self) -> &mut UniquePtr<SLImage> {
        &mut self.image
    }

    pub fn new(height: u32, width: u32) -> Self {
        Self {
            image: SLImage::new2(autocxx::c_int(width as i32), autocxx::c_int(height as i32))
                .within_unique_ptr(),
        }
    }

    pub fn new_depth(height: u32, width: u32, depth: u32) -> Self {
        Self {
            image: SLImage::new3(
                autocxx::c_int(width as i32),
                autocxx::c_int(height as i32),
                autocxx::c_int(depth as i32),
            )
            .within_unique_ptr(),
        }
    }

    pub fn read_tiff_image(&mut self, path: &PathBuf) -> Result<(), ()> {
        if SLImage::ReadTiffImage(path.to_str().unwrap(), self.image.pin_mut()) {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn write_tiff_image(&mut self, path: &PathBuf) -> Result<(), ()> {
        if SLImage::WriteTiffImage(
            path.to_str().unwrap(),
            self.image.pin_mut(),
            autocxx::c_int(16),
        ) {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn get_average_image(&mut self) -> SLImageRs {
        SLImageRs {
            image: self.image.pin_mut().GetAverageImage().within_unique_ptr(),
        }
    }

    pub fn offset_correction(
        &mut self,
        dark_map: &mut SLImageRs,
        offset: u32,
    ) -> Result<(), InternalSLError> {
        unsafe {
            {
                match SLImage::OffsetCorrection(
                    self.image.pin_mut(),
                    std::pin::Pin::<&mut SLImage>::into_inner_unchecked(
                        dark_map.get_underlying().pin_mut(),
                    ),
                    autocxx::c_int(offset as i32),
                ) {
                    SLError::SL_ERROR_SUCCESS => Ok(()),
                    e => Err(e.into()),
                }
            }
        }
    }

    pub fn defect_correction(
        &mut self,
        out_image: &mut SLImageRs,
        defect_map: &mut SLImageRs,
    ) -> Result<(), InternalSLError> {
        unsafe {
            match SLImage::DefectCorrection(
                self.image.pin_mut(),
                out_image.get_underlying().pin_mut(),
                std::pin::Pin::<&mut SLImage>::into_inner_unchecked(
                    defect_map.get_underlying().pin_mut(),
                ),
            ) {
                SLError::SL_ERROR_SUCCESS => Ok(()),
                e => Err(e.into()),
            }
        }
    }

    pub fn get_height(&mut self) -> u32 {
        i32::from(self.image.pin_mut().GetHeight()) as u32
    }

    pub fn get_width(&mut self) -> u32 {
        i32::from(self.image.pin_mut().GetWidth()) as u32
    }

    pub fn get_data_pointer(&mut self, frame: u32) -> *mut u8 {
        self.image
            .pin_mut()
            .GetDataPointer(autocxx::c_int(frame as i32)) as *mut u8
    }

    pub fn to_image_buffer(&mut self) -> ImageBuffer<Luma<u16>, Vec<u16>> {
        let height = self.get_height();
        let width = self.get_width();

        let u16_vec: Vec<u16>;
        unsafe {
            u16_vec = std::slice::from_raw_parts(
                self.get_data_pointer(0) as *const u16,
                (width * height) as usize,
            )
            .to_vec();
        }

        ImageBuffer::<Luma<u16>, Vec<u16>>::from_vec(width as u32, height as u32, u16_vec).unwrap()
    }
}

#[derive(Serialize, Debug, Type)]
pub struct InternalSLError(String);

#[derive(Type)]
pub enum RemoteBinningModes {
    BinningUnknown,
    x11,
    x22,
    x44,
}

#[derive(Serialize, Type, Deserialize, Clone)]
pub struct BinningModesRS(#[specta(type = RemoteBinningModes)] pub BinningModes);

impl Serialize for BinningModes {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            BinningModes::BinningUnknown => serializer.serialize_str("Unknown"),
            BinningModes::x11 => serializer.serialize_str("x11"),
            BinningModes::x22 => serializer.serialize_str("x22"),
            BinningModes::x44 => serializer.serialize_str("x44"),
        }
    }
}

impl<'de> Deserialize<'de> for BinningModes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_ref() {
            "Unknown" => Ok(BinningModes::BinningUnknown),
            "x11" => Ok(BinningModes::x11),
            "x22" => Ok(BinningModes::x22),
            "x44" => Ok(BinningModes::x44),
            _ => Err(serde::de::Error::custom("Invalid value for Binning Mode")),
        }
    }
}

impl fmt::Debug for BinningModesRS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                BinningModes::BinningUnknown => "Unknown",
                BinningModes::x11 => "x11",
                BinningModes::x22 => "x22",
                BinningModes::x44 => "x44",
            }
        )
    }
}

#[derive(Type)]
pub enum RemoteFullWellModes {
    High,
    Low,
    Enum,
}

#[derive(Serialize, Type, Deserialize, Clone)]
pub struct FullWellModesRS {
    #[specta(type = RemoteFullWellModes)]
    pub remote_ty: FullWellModes,
}

impl fmt::Display for FullWellModesRS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.remote_ty {
                FullWellModes::Low => "LFW",
                FullWellModes::High => "HFW",
                FullWellModes::Unknown => "Uknown",
            }
        )
    }
}

impl fmt::Debug for FullWellModesRS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Serialize for FullWellModes {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let b = FullWellModesRS {
            remote_ty: FullWellModes::High,
        };
        match self {
            FullWellModes::High => serializer.serialize_str("High"),
            FullWellModes::Low => serializer.serialize_str("Low"),
            FullWellModes::Unknown => serializer.serialize_str("Unknown"),
        }
    }
}

impl<'de> Deserialize<'de> for FullWellModes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_ref() {
            "High" => Ok(FullWellModes::High),
            "Low" => Ok(FullWellModes::Low),
            "Unknown" => Ok(FullWellModes::Unknown),
            _ => Err(serde::de::Error::custom("Invalid value for FullWellModes")),
        }
    }
}

impl std::fmt::Debug for SLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SL_ERROR_SUCCESS => write!(f, "SL_ERROR_SUCCESS"),
            Self::SL_ERROR_INVALID_PARAM => write!(f, "SL_ERROR_INVALID_PARAM"),
            Self::SL_ERROR_NO_DEVICE => write!(f, "SL_ERROR_NO_DEVICE"),
            Self::SL_ERROR_NOT_FOUND => write!(f, "SL_ERROR_NOT_FOUND"),
            Self::SL_ERROR_BUSY => write!(f, "SL_ERROR_BUSY"),
            Self::SL_ERROR_TIMEOUT => write!(f, "SL_ERROR_TIMEOUT"),
            Self::SL_ERROR_CORRECTION => write!(f, "SL_ERROR_CORRECTION"),
            Self::SL_ERROR_NOT_SUPPORTED => write!(f, "SL_ERROR_NOT_SUPPORTED"),
            Self::SL_ERROR_ALREADY_EXISTS => write!(f, "SL_ERROR_ALREADY_EXISTS"),
            Self::SL_ERROR_INTERNAL => write!(f, "SL_ERROR_INTERNAL"),
            Self::SL_ERROR_OTHER => write!(f, "SL_ERROR_OTHER"),
            Self::SL_ERROR_DEVICE_CLOSED => write!(f, "SL_ERROR_DEVICE_CLOSED"),
            Self::SL_ERROR_DEVICE_STREAMING => write!(f, "SL_ERROR_DEVICE_STREAMING"),
            Self::SL_ERROR_CONFIG_FAILED => write!(f, "SL_ERROR_CONFIG_FAILED"),
            Self::SL_ERROR_CONFIG_FILE_NOT_FOUND => write!(f, "SL_ERROR_CONFIG_FILE_NOT_FOUND"),
            Self::SL_ERROR_NOT_ENOUGH_MEMORY => write!(f, "SL_ERROR_NOT_ENOUGH_MEMORY"),
            Self::SL_ERROR_OVERFLOW => write!(f, "SL_ERROR_OVERFLOW"),
            Self::SL_ERROR_PIPE => write!(f, "SL_ERROR_PIPE"),
            Self::SL_ERROR_INTERRUPTED => write!(f, "SL_ERROR_INTERRUPTED"),
            Self::SL_ERROR_IO => write!(f, "SL_ERROR_IO"),
            Self::SL_ERROR_ACCESS => write!(f, "SL_ERROR_ACCESS"),
            Self::SL_ERROR_REQUIRES_ADMIN => write!(f, "SL_ERROR_REQUIRES_ADMIN"),
            Self::SL_ERROR_CRITICAL => write!(f, "SL_ERROR_CRITICAL"),
        }
    }
}

impl From<SLError> for InternalSLError {
    fn from(error: SLError) -> Self {
        let error_str = format!("{:?}", error); // Using Debug implementation to get the string representation
        InternalSLError(error_str)
    }
}
