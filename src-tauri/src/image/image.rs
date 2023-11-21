use super::types::{Annotation, DataExtractor, Line, Rect};
use super::{ExtraData, ImageMetadata};
use crate::capture::capture::CaptureSetting;
use crate::capture::types::AdvancedCapture;
use crate::charts::charts::ChartSubscriber;
use crate::operations::HistogramEquilisation;
use crate::utils::serialize_dt;
use chrono::prelude::{DateTime, Utc};
use image::ImageEncoder;
use image::{ImageBuffer, Luma};
use image_lib::{imageops, EncodableLayout};
use log::info;
use rayon::prelude::ParallelIterator;
use rayon::slice::ParallelSlice;
use serde::Serialize;
use serde_with::serde_as;
use specta::Type;
use std::fmt::Debug;
use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use tiff::encoder::{colortype, TiffEncoder};

const RANGE_SIZE: usize = 16384;

#[derive(Serialize, Type, Clone, Debug)]
pub struct LineProfileData {
    idx: u32,
    value: u32,
}

pub type LineProfile = Vec<LineProfileData>;

#[derive(Serialize, Type)]
pub struct ImageService {
    #[serde(skip)]
    app: AppHandle,
    pub image_stacks: Vec<ImageStack>,
}

impl Debug for ImageService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

unsafe impl Send for ImageService {}
unsafe impl Sync for ImageService {}
unsafe impl Send for ImageHandler {}
unsafe impl Sync for ImageHandler {}

impl ImageService {
    pub fn new(app: AppHandle) -> Self {
        ImageService {
            app,
            image_stacks: Vec::new(),
        }
    }

    pub fn add_image_stack(&mut self, stack: ImageStack) {
        self.image_stacks.push(stack);

        self.app
            .emit("image-state-event", &self.image_stacks)
            .unwrap();
    }

    pub fn remove_image_stack(&mut self, stack_idx: usize) {
        self.image_stacks.remove(stack_idx);
        self.app
            .emit("image-state-event", &self.image_stacks)
            .unwrap();
    }

    pub fn get_handler(&self, stack_idx: usize, ip_idx: usize) -> Option<&ImageHandler> {
        if let Some(stack) = self.image_stacks.get(stack_idx) {
            if let Some(image_handler) = stack.image_handlers.get(ip_idx) {
                return Some(image_handler);
            }
        }

        return None;
    }

    pub fn get_mut_handler(
        &mut self,
        stack_idx: usize,
        ip_idx: usize,
    ) -> Option<&mut ImageHandler> {
        if let Some(stack) = self.image_stacks.get_mut(stack_idx) {
            if let Some(image_handler) = stack.image_handlers.get_mut(ip_idx) {
                return Some(image_handler);
            }
        }

        return None;
    }

    pub fn save_image(&self, stack_idx: usize, image_index: usize, path: &Path) {
        if let Some(stack) = self.image_stacks.get(stack_idx) {
            if let Some(image_handler) = stack.image_handlers.get(image_index) {
                image_handler.image.save(path);
            }
        }
    }
}

#[serde_as]
#[derive(Type, Serialize)]
pub struct ImageStack {
    #[serde(serialize_with = "serialize_dt")]
    pub timestamp: Option<DateTime<Utc>>,
    pub image_handlers: Vec<ImageHandler>,
    pub capture: Option<AdvancedCapture>,
}

impl ImageStack {
    pub fn new(
        images: Vec<ImageBuffer<Luma<u16>, Vec<u16>>>,
        timestamp: Option<DateTime<Utc>>,
        capture: Option<AdvancedCapture>,
    ) -> Self {
        let mut image_handlers: Vec<ImageHandler> = Vec::new();

        for image in images {
            image_handlers.push(ImageHandler::new(
                image,
                ImageMetadata {
                    capture_settings: None,
                    date_created: None,
                    extra_info: None,
                },
            ));
        }

        return Self {
            timestamp,
            image_handlers,
            capture,
        };
    }

    pub fn save(&self, path: PathBuf) {
        let mut img_file = Cursor::new(Vec::new());

        {
            // first create a multipage image with 2 images
            let mut img_encoder = TiffEncoder::new(&mut img_file).unwrap();

            for image_handler in &self.image_handlers {
                let image = &image_handler.image;
                img_encoder
                    .write_image::<colortype::Gray16>(image.width(), image.height(), image.as_raw())
                    .unwrap();
            }
        }

        let mut output_file = File::create(path).unwrap();
        img_file.set_position(0);
        std::io::copy(&mut img_file, &mut output_file).unwrap();
    }
}

#[derive(Serialize, Type)]
pub struct ImageHandler {
    #[serde(skip)]
    pub lut: Option<Vec<u16>>,
    #[serde(skip)]
    pub image: ImageBuffer<Luma<u16>, Vec<u16>>,
    #[serde(skip)]
    subscribers: Vec<Box<dyn ChartSubscriber + Send>>,
    pub image_metadata: ImageMetadata,
    pub roi: Option<Annotation>,
    pub inverted_colours: bool,
}

impl Clone for ImageHandler {
    fn clone(&self) -> Self {
        // Create a new ImageHandler with the same values as self
        ImageHandler {
            lut: self.lut.clone(),
            image: self.image.clone(),
            subscribers: Vec::new(), // Provide an empty vector for subscribers
            image_metadata: self.image_metadata.clone(),
            roi: self.roi.clone(),
            inverted_colours: self.inverted_colours,
        }
    }
}

impl ImageHandler {
    pub fn new(image: ImageBuffer<Luma<u16>, Vec<u16>>, image_metadata: ImageMetadata) -> Self {
        Self {
            lut: None,
            image,
            roi: None,
            inverted_colours: true,
            image_metadata,
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, subscriber: Box<dyn ChartSubscriber + Send>) {
        subscriber.update(&self.image, self.roi.clone());
        self.subscribers.push(subscriber);
    }

    pub fn unsubscribe(&mut self, subscriber_index: usize) {
        self.subscribers.remove(subscriber_index);
    }

    pub fn notify_subscribers(&mut self) {
        for subsriber in self.subscribers.iter() {
            subsriber.update(&self.image, self.roi.clone())
        }
    }

    pub fn update_roi(&mut self, annotation: Annotation) {
        self.roi = Some(annotation);
        self.notify_subscribers();
    }

    pub fn apply_histogram_equilization(&mut self) {
        match &self.roi {
            Some(roi) => {
                self.lut = Some(self.image.cumulative_histogram_roi(roi, RANGE_SIZE));
                info!("{:?}", *roi);
            }
            None => {
                self.lut = Some(self.image.cumulative_histogram(RANGE_SIZE));
            }
        }
    }

    pub fn rotate_left(&mut self) {
        self.image = imageops::rotate270(&mut self.image);
    }

    pub fn rotate_right(&mut self) {
        self.image = imageops::rotate90(&mut self.image);
    }

    pub fn flip(&mut self, vertically: bool) {
        if vertically {
            imageops::flip_horizontal_in_place(&mut self.image);
        } else {
            imageops::flip_vertical_in_place(&mut self.image);
        }
    }

    pub fn invert_colours(&mut self) {
        self.inverted_colours = !self.inverted_colours;
    }

    pub fn reset_lut(&mut self) {
        self.lut = None;
    }

    pub fn set_threshold(&mut self, min_threshold: u16, max_threshold: u16) {
        if let Some(lut_vec) = self.lut.as_mut() {
            for i in 0..65535 {
                if i < min_threshold {
                    lut_vec[i as usize] = min_threshold;
                } else if i > max_threshold {
                    lut_vec[i as usize] = max_threshold;
                } else {
                    lut_vec[i as usize] = i;
                }
            }
        }
    }

    fn apply_lut(brightness: i32, contrast: f32) -> [u16; RANGE_SIZE] {
        let mut lut = [0u16; RANGE_SIZE];

        let mid_point = RANGE_SIZE as f32 / 2.0;

        for i in 0..RANGE_SIZE {
            let mut value = (i as f32 - mid_point) * contrast + mid_point + brightness as f32;
            value = value.max(0.0).min(RANGE_SIZE as f32 - 1.0); // Clamping to 0-RANGE_SIZE-1
            lut[i] = value as u16;
        }

        lut
    }

    pub fn get_image(&self) -> ImageBuffer<Luma<u16>, Vec<u16>> {
        let mut thresholded_image = self.image.clone();

        if let Some(lut_array) = &self.lut {
            #[cfg(feature = "rayon")]
            let iter = thresholded_image.par_iter_mut();

            #[cfg(not(feature = "rayon"))]
            let iter = thresholded_image.iter_mut();

            info!("getting image");

            iter.for_each(|p| {
                let lut_val = lut_array[*p as usize];
                let max_value = RANGE_SIZE as u16 - 1;

                *p = lut_val as u16;
            });
        };

        if self.inverted_colours {
            let iter = thresholded_image.iter_mut();

            iter.for_each(|p| {
                let max_value = (1u16 << 14) - 1; // Maximum value for a 14-bit image

                // Invert the pixel value directly
                *p = max_value - *p;
            });
        }

        thresholded_image
    }

    pub fn get_image_as_bytes(&self) -> Vec<u8> {
        self.get_image().as_bytes().to_owned()
    }

    pub fn create_rgba_image(&self) -> Vec<u8> {
        self.image
            .clone()
            .into_raw()
            .par_chunks_exact(1)
            .map(|chunk| {
                let luma_u8 = (chunk[0] / 256) as u8;
                [luma_u8, luma_u8, luma_u8, 255]
            })
            .flatten()
            .collect()
    }

    pub fn create_image<T: ImageEncoder>(&mut self, encoder: T) {
        let (width, height) = self.image.dimensions();

        let mut thresholded_image: ImageBuffer<Luma<u16>, Vec<u16>> =
            ImageBuffer::new(width, height);

        if let Some(lut_array) = &self.lut {
            for (x, y, pixel) in thresholded_image.enumerate_pixels_mut() {
                let intensity = self.image.get_pixel(x, y)[0] as usize;
                let lutval = lut_array[intensity];

                // TODO: put this elsewhere
                if (self.inverted_colours) {
                    *pixel = Luma([u16::MAX - lutval as u16])
                } else {
                    *pixel = Luma([lutval as u16]);
                }
            }
        } else {
            for (x, y, pixel) in self.image.enumerate_pixels() {
                thresholded_image.put_pixel(x, y, Luma([pixel[0]]));
            }
        }

        encoder
            .write_image(
                thresholded_image.as_bytes(),
                width,
                height,
                image_lib::ColorType::L16,
            )
            .unwrap();
    }
}

pub struct ImageIterator<'a> {
    image: &'a ImageBuffer<Luma<u16>, Vec<u16>>,
    roi: Annotation,
    current: u32,
    coord_iterators: Option<CoordIterators>,
}

impl<'a> ImageIterator<'a> {
    pub fn new(image: &'a ImageBuffer<Luma<u16>, Vec<u16>>, roi: Annotation) -> Self {
        Self {
            image,
            roi,
            current: 0,
            coord_iterators: None,
        }
    }
}

impl<'a> Iterator for ImageIterator<'a> {
    type Item = &'a u16;

    fn next(&mut self) -> Option<Self::Item> {
        // Determine the appropriate coordinate iterator based on ROI type
        if self.coord_iterators.is_none() {
            match &self.roi {
                Annotation::Rect(rect) => {
                    let rect_iterator = RectIterator {
                        rect: rect.clone(),
                        current_x: rect.pos.x,
                        current_y: rect.pos.y,
                    };
                    self.coord_iterators = Some(CoordIterators::Rect(rect_iterator));
                }
                Annotation::Line(line) => {
                    let line_iterator = LineIterator {
                        line: line.clone(),
                        current: 0,
                    };
                    self.coord_iterators = Some(CoordIterators::Line(line_iterator));
                }
            }
        }

        if let Some(coord_iterator) = &mut self.coord_iterators {
            while let Some((x, y)) = coord_iterator.next() {
                if x < self.image.width() && y < self.image.height() {
                    return Some(&self.image.get_pixel(x, y).0[0]);
                }
            }
        }

        None
    }
}

impl DataExtractor for Rect {
    fn iter_values<'a>(&self, image: &'a ImageBuffer<Luma<u16>, Vec<u16>>) -> ImageIterator<'a> {
        ImageIterator::new(image, Annotation::Rect(self.clone()))

        /*
        let x1 = self.pos.x;
        let y1 = self.pos.y;
        let x2 = x1 + self.width;
        let y2 = y1 + self.height;

        let x1 = x1.clamp(0, image.width() - 1);
        let y1 = y1.clamp(0, image.height() - 1);
        let x2 = x2.clamp(0, image.width() - 1);
        let y2 = y2.clamp(0, image.height() - 1);

        let mut values = Vec::new();

        for y in y1..=y2 {
            for x in x1..=x2 {
                let pixel = image.get_pixel(x, y);
                values.push(&pixel.0[0]);
            }
        }

        values.into_iter()
        */
    }

    fn get_std(&self, img: &ImageBuffer<Luma<u16>, Vec<u16>>) -> f64 {
        let mut sum = 0;

        for y in self.pos.y..self.pos.y + self.height {
            for x in self.pos.x..self.pos.x + self.width {
                let pixel = img.get_pixel(x, y);
                let intensity = pixel[0]; // assuming grayscale image

                sum += intensity as u32;
            }
        }

        let count = self.width * self.height;
        let mean = sum as f64 / count as f64;

        let mut sum_squared_diff = 0.0;
        for y in self.pos.y..self.pos.y + self.height {
            for x in self.pos.x..self.pos.x + self.width {
                let pixel = img.get_pixel(x, y);
                let intensity = pixel[0] as f64; // assuming grayscale image
                sum_squared_diff += (intensity - mean).powi(2);
            }
        }
        let variance = sum_squared_diff / count as f64;
        let std = variance.sqrt();

        return std;
    }

    fn get_profile(&self, img: &ImageBuffer<Luma<u16>, Vec<u16>>) -> LineProfile {
        let mut line_profile = LineProfile::new();

        if self.height == 0 || self.width == 0 {
            return Vec::new();
        }

        for x in self.pos.x..self.pos.x + self.width {
            let mut column_sum = 0;
            for y in self.pos.y..self.pos.y + self.height {
                column_sum += img.get_pixel(x, y)[0] as u32;
            }

            let column_average = column_sum / self.height;
            line_profile.push(LineProfileData {
                idx: x,
                value: column_average,
            });
        }

        line_profile
    }
}

impl DataExtractor for Line {
    fn iter_values<'a>(&self, image: &'a ImageBuffer<Luma<u16>, Vec<u16>>) -> ImageIterator<'a> {
        ImageIterator::new(image, Annotation::Line(self.clone()))

        /*
        let x1 = self.start.x;
        let y1 = self.start.y;
        let x2 = self.finish.x;
        let y2 = self.finish.y;

        let dx = x2 as f64 - x1 as f64;
        let dy = y2 as f64 - y1 as f64;

        let length = (dx.powi(2) + dy.powi(2)).sqrt();

        let unit_x = dx / length;
        let unit_y = dy / length;

        let mut values = Vec::new();

        for i in 0..=length as u32 {
            let x = (x1 as f64 + unit_x * i as f64).round() as u32;
            let y = (y1 as f64 + unit_y * i as f64).round() as u32;

            if x < image.width() && y < image.height() {
                let pixel = image.get_pixel(x, y);
                values.push(&pixel.0[0]);
            }
        }

        values.into_iter()
        */
    }

    fn get_std(&self, img: &ImageBuffer<Luma<u16>, Vec<u16>>) -> f64 {
        let points = crate::statistics::get_points_along_line(
            self.start.x as isize,
            self.start.y as isize,
            self.finish.x as isize,
            self.finish.y as isize,
        );

        // Calculate the mean pixel value along the line
        let mut sum = 0;
        for point in &points {
            if let Some(pixel) = img.get_pixel(point.0 as u32, point.1 as u32).0.get(0) {
                sum += *pixel as u64;
            }
        }
        let mean = sum as f64 / points.len() as f64;

        // Calculate the sum of squared differences
        let mut sum_squared_diff = 0.0;
        for point in &points {
            if let Some(pixel) = img.get_pixel(point.0 as u32, point.1 as u32).0.get(0) {
                let diff = (*pixel as f64 - mean).powi(2);
                sum_squared_diff += diff;
            }
        }

        // Calculate the variance and standard deviation
        let variance = sum_squared_diff / points.len() as f64;
        let std = variance.sqrt();

        std
    }

    fn get_profile(&self, img: &ImageBuffer<Luma<u16>, Vec<u16>>) -> LineProfile {
        // TODO: Handle minus cases properly

        let points = crate::statistics::get_points_along_line(
            self.start.x as isize,
            self.start.y as isize,
            self.finish.x as isize,
            self.finish.y as isize,
        );

        let mut line_profile = LineProfile::new();

        // Build the profile data by averaging each column
        let mut prev_point = points.get(0).unwrap();
        let mut column_sum = img.get_pixel(prev_point.0 as u32, prev_point.1 as u32)[0] as u64;
        let mut column_count: u16 = 1;
        for point in points.iter().skip(1) {
            let intensity = img.get_pixel(point.0 as u32, point.1 as u32)[0];
            if prev_point.0 == point.0 {
                column_sum += intensity as u64;
                column_count += 1;
            } else {
                line_profile.push(LineProfileData {
                    idx: prev_point.0 as u32,
                    value: (column_sum as u32 / column_count as u32) as u32,
                });
                prev_point = point;
                column_sum = intensity as u64;
                column_count = 1;
            }
        }

        line_profile.push(LineProfileData {
            idx: prev_point.0 as u32,
            value: (column_sum as u32 / column_count as u32) as u32,
        });

        line_profile
    }
}

pub struct ImageMetadataBuilder {
    capture_settings: Option<CaptureSetting>,
    date_created: Option<DateTime<Utc>>,
    extra_info: Option<ExtraData>,
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

    pub fn extra_info(&mut self, extra: ExtraData) -> &mut Self {
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

trait CoordIterator: Iterator<Item = (u32, u32)> {}

struct RectIterator {
    rect: Rect,
    current_x: u32,
    current_y: u32,
}

impl Iterator for RectIterator {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_y < self.rect.pos.y + self.rect.height {
            let current_point = (self.current_x, self.current_y);
            self.current_x += 1;
            if self.current_x == self.rect.pos.x + self.rect.width {
                self.current_x = self.rect.pos.x;
                self.current_y += 1;
            }
            Some(current_point)
        } else {
            None
        }
    }
}

struct LineIterator {
    line: Line,
    current: u32,
}

impl Iterator for LineIterator {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let dx = (self.line.finish.x as f64) - (self.line.start.x as f64);
        let dy = (self.line.finish.y as f64) - (self.line.start.y as f64);
        let length = (dx.powi(2) + dy.powi(2)).sqrt();

        if self.current <= length as u32 {
            let t = self.current as f64 / length as f64;
            let x = (self.line.start.x as f64 + dx * t).round() as u32;
            let y = (self.line.start.y as f64 + dy * t).round() as u32;
            self.current += 1;
            Some((x, y))
        } else {
            None
        }
    }
}

enum CoordIterators {
    Rect(RectIterator),
    Line(LineIterator),
}

impl Iterator for CoordIterators {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CoordIterators::Rect(rect_iterator) => rect_iterator.next(),
            CoordIterators::Line(line_iterator) => line_iterator.next(),
        }
    }
}
