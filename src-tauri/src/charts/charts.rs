use image::{ImageBuffer, Luma};
use log::{error, info};
use tauri::{AppHandle, Window};
use tauri_specta::Event;

use crate::{
    image::{types::DataExtractor, Annotation, LineProfile}, charts::types::LineProfileEvent,
};

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
        info!("updating!");
        if let Some(roi) = roi {
            let line_profile_data: LineProfile = roi.get_profile(&image);

            if let Err(e) = LineProfileEvent(line_profile_data).emit(&self.window) {
                error!("Failed to update line profile with error {e}");
            } else {
            }
        }
    }
}

impl ChartSubscriber for HistogramSubscriber {
    fn update(&self, image: &ImageBuffer<Luma<u16>, Vec<u16>>, roi: Option<Annotation>) {
        todo!();
    }
}

/*
fn get_points_along_line(x1: isize, y1: isize, x2: isize, y2: isize) -> Vec<(isize, isize)> {
    let mut points = Vec::new();

    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();

    let step_x = if x1 < x2 { 1 } else { -1 };
    let step_y = if y1 < y2 { 1 } else { -1 };

    let mut x = x1;
    let mut y = y1;
    let mut err = dx - dy;

    while x != x2 || y != y2 {
        points.push((x, y));

        let err2 = 2 * err;

        if err2 > -dy {
            err -= dy;
            x += step_x;
        }

        if err2 < dx {
            err += dx;
            y += step_y;
        }
    }

    points.push((x, y));

    points
}
*/
