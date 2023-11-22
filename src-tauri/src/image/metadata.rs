use chrono::{DateTime, Utc};
use serde::Serialize;
use specta::Type;

use crate::capture::capture::CaptureSetting;

use super::types::Rect;

#[derive(Clone, Serialize, Type, Debug)]
pub struct ImageMetadata {
    pub capture_settings: Option<CaptureSetting>,
    pub date_created: Option<DateTime<Utc>>,
    pub extra_info: Option<CaptureResultData>,
}

#[derive(Clone, Serialize, Type, Debug)]
#[serde(tag = "type")]
pub enum CaptureResultData {
    SmartCaptureData(SmartCaptureData),
    SignalAccumulationData(SignalAccumulationData),
}

#[derive(Clone, Serialize, Type, Debug)]
pub struct SmartCaptureData {
    pub signal_noise_ratio: f64,
    pub background_rect: Rect,
    pub foreground_rect: Rect,
}

#[derive(Clone, Serialize, Type, Debug)]
pub struct SignalAccumulationData {
    pub accumulated_exp_time: u32,
}

pub struct ImageMetadataBuilder {
    capture_settings: Option<CaptureSetting>,
    date_created: Option<DateTime<Utc>>,
    extra_info: Option<CaptureResultData>,
}

impl ImageMetadataBuilder {
    pub fn new() -> Self {
        ImageMetadataBuilder {
            capture_settings: None,
            date_created: None,
            extra_info: None,
        }
    }

    pub fn capture_settings(&mut self, settings: CaptureSetting) -> &mut Self {
        self.capture_settings = Some(settings);
        self
    }

    pub fn date_created(&mut self, date: DateTime<Utc>) -> &mut Self {
        self.date_created = Some(date);
        self
    }

    pub fn extra_info(&mut self, extra: CaptureResultData) -> &mut Self {
        self.extra_info = Some(extra);
        self
    }

    pub fn build(&self) -> ImageMetadata {
        ImageMetadata {
            capture_settings: self.capture_settings.clone(),
            date_created: self.date_created.clone(),
            extra_info: self.extra_info.clone(),
        }
    }
}