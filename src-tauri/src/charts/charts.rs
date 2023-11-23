use image::{ImageBuffer, Luma};
use log::{error, info};
use tauri::{AppHandle, Window};
use tauri_specta::Event;

use crate::{image::{
 types::DataExtractor, Annotation, ImageIterator, LineProfile, calculate_histogram_min_max,
}, charts::types::HistogramBin};

use super::types::{ChartData, ChartDataEvent};

pub trait ChartSubscriber {
    fn update(&self, image: &ImageBuffer<Luma<u16>, Vec<u16>>, roi: Option<Annotation>);
}

pub struct LineProfileSubscriber {
    pub app: AppHandle,
    pub window: Window,
}

pub struct HistogramSubscriber {
    pub app: AppHandle,
    pub window: Window,
}

impl ChartSubscriber for LineProfileSubscriber {
    fn update(&self, image: &ImageBuffer<Luma<u16>, Vec<u16>>, roi: Option<Annotation>) {
        if let Some(roi) = roi {
            let line_profile_data: LineProfile = roi.get_profile(&image);
            if let Err(e) =
                ChartDataEvent(ChartData::LineProfileData(line_profile_data)).emit(&self.window)
            {
                error!("Error when emitting chart data event for line profile {e}");
            } else {
            }
        }
    }
}

impl ChartSubscriber for HistogramSubscriber {
    fn update(&self, image: &ImageBuffer<Luma<u16>, Vec<u16>>, roi: Option<Annotation>) {
        let histogram = roi.map(|roi| {
            let iter = ImageIterator::new(image, roi);
            calculate_histogram_min_max(iter.collect(), 256)
        })
        .unwrap_or_else(|| calculate_histogram_min_max(image.iter().collect(), 256));

        if let Err(e) = ChartDataEvent(ChartData::HistogramData(histogram)).emit(&self.window) {
            error!("Error when emitting chart data event for histogram: {e}");
        }
    }
}
