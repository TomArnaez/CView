use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use image::{ImageBuffer, Luma};
use imageproc::stats::histogram;
use log::info;
use rayon::prelude::*;

use crate::image::types::DataExtractor;

type Histogram = HashMap<u16, u32>;

pub fn median_filter_threaded(
    image: &ImageBuffer<Luma<u16>, Vec<u16>>,
    window_height: u32,
    window_width: u32,
) -> ImageBuffer<Luma<u16>, Vec<u16>> {
    let width = image.width();
    let height = image.height();

    let radius_x = (window_width / 2) as i32;
    let radius_y = (window_height / 2) as i32;

    let output_image = Arc::new(Mutex::new(ImageBuffer::new(width, height)));

    let mut pixel_coords: Vec<(u32, u32)> = Vec::new();

    // Create a vector of pixel coordinates to be processed in parallel
    for y in 0..height {
        for x in 0..width {
            pixel_coords.push((x, y));
        }
    }

    pixel_coords.par_iter().for_each(|&(x, y)| {
        let mut window_values = Vec::with_capacity((window_width * window_height) as usize);

        for j in -radius_y..=radius_y {
            for i in -radius_x..=radius_x {
                let new_x = x as i32 + i;
                let new_y = y as i32 + j;

                if new_x >= 0 && new_x < width as i32 && new_y >= 0 && new_y < height as i32 {
                    let pixel_value = image.get_pixel(new_x as u32, new_y as u32)[0] as u16;
                    window_values.push(pixel_value);
                }
            }
        }

        window_values.sort();
        let median_index = window_values.len() / 2;
        let median_value = window_values[median_index];

        let mut output_image = output_image.lock().unwrap();
        output_image.put_pixel(x, y, Luma([median_value]));
    });

    Arc::try_unwrap(output_image)
        .ok()
        .expect("Failed to unwrap Arc")
        .into_inner()
        .expect("Failed to get Mutex inner value")
}

pub fn calculate_histogram<'a>(vals: impl Iterator<Item = &'a u16>) -> Histogram {
    let mut histogram = HashMap::new();

    for value in vals {
        *histogram.entry(*value).or_insert(0) += 1;
    }

    histogram
}

pub fn create_lut(histogram: &Histogram, range: usize) -> Vec<u16> {
    let mut cdf_min = None;
    let mut cdf = 0;
    let mut lut = vec![0u16; range];

    let total: u32 = histogram.values().sum();

    for i in 0..16384u16 {
        if let Some(&count) = histogram.get(&i) {
            cdf += count;
            cdf_min = cdf_min.or_else(|| Some(cdf));
        }
        if let Some(cdf_min) = cdf_min {
            // Ensuring the scaling is appropriate for the range of values in your LUT
            lut[i as usize] =
                ((((cdf - cdf_min) as f64) / ((total - cdf_min) as f64)) * 16383.0).round() as u16;
        }
    }

    lut
}

pub trait HistogramEquilisation {
    fn cumulative_histogram(&self, range: usize) -> Vec<u16>;
    fn cumulative_histogram_roi(&self, roi: &dyn DataExtractor, range: usize) -> Vec<u16>;
}

impl HistogramEquilisation for ImageBuffer<image::Luma<u16>, Vec<u16>> {
    fn cumulative_histogram(&self, range: usize) -> Vec<u16> {
        info!("calculate cumulative histogram");
        let histogram = calculate_histogram(self.into_iter());
        create_lut(&histogram, range)
    }

    fn cumulative_histogram_roi(&self, roi: &dyn DataExtractor, range: usize) -> Vec<u16> {
        info!("calculate cumulative histogram with roi");
        let histogram = calculate_histogram(roi.iter_values(&self));
        create_lut(&histogram, range)
    }
}

pub fn adjust_brightness(
    image: &ImageBuffer<Luma<u16>, Vec<u16>>,
    delta: u16,
) -> ImageBuffer<Luma<u16>, Vec<u16>> {
    let mut adjusted_image = ImageBuffer::<Luma<u16>, Vec<u16>>::new(image.width(), image.height());
    for (x, y, pixel) in adjusted_image.enumerate_pixels_mut() {
        let old_pixel_intensity = image.get_pixel(x, y)[0] as u16;
        let new_intensity = old_pixel_intensity.saturating_add(delta); // Ensure the new value doesn't exceed u16 range
        *pixel = Luma([new_intensity]);
    }

    adjusted_image
}

pub fn adjust_contrast(
    image: &ImageBuffer<Luma<u16>, Vec<u16>>,
    contrast: f32,
) -> ImageBuffer<Luma<u16>, Vec<u16>> {
    let mut adjusted_image = ImageBuffer::<Luma<u16>, Vec<u16>>::new(image.width(), image.height());

    for (x, y, pixel) in adjusted_image.enumerate_pixels_mut() {
        let original_intensity = image.get_pixel(x, y)[0] as f32;
        let new_intensity = 32768.0 + (original_intensity - 32768.0) * contrast;
        let clamped_intensity = new_intensity.max(0.0).min(65535.0);
        *pixel = Luma([clamped_intensity as u16]);
    }

    adjusted_image
}

pub fn invert_colors_grayscale(
    image: &ImageBuffer<Luma<u16>, Vec<u16>>,
) -> ImageBuffer<Luma<u16>, Vec<u16>> {
    let (width, height) = image.dimensions();
    let mut inverted_image = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let inverted_pixel_value = u16::max_value() - pixel[0];
            inverted_image.put_pixel(x, y, Luma([inverted_pixel_value]));
        }
    }
    inverted_image
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caculate_histogram() {
        let numbers: Vec<u16> = vec![1, 2, 1, 3, 2, 2, 4, 1, 5];
        let histogram = calculate_histogram(numbers.iter());

        let mut expected_histogram = HashMap::new();
        expected_histogram.insert(1, 3);
        expected_histogram.insert(2, 3);
        expected_histogram.insert(3, 1);
        expected_histogram.insert(4, 1);
        expected_histogram.insert(5, 1);

        assert_eq!(histogram, expected_histogram)
    }

    #[test]
    fn test_create_lut_uniform_histogram() {
        let total_pixels = 16384;
        let mut histogram = Histogram::new();

        let range = 16384;

        // Create a uniform histogram
        for i in 0..range {
            histogram.insert(i, 1);
        }

        let lut = create_lut(&histogram, 16384);

        assert_eq!(lut.len(), 16384);

        // In a uniform histogram, the LUT should increment linearly
        for (i, &lut_value) in lut.iter().enumerate() {
            let expected_value = i as u16;
            assert_eq!(
                lut_value, expected_value,
                "LUT value at index {} is incorrect",
                i
            );
        }
    }

    #[test]
    fn test_create_lut_non_uniform_histogram() {
        let total_pixels = 50;
        let mut histogram = Histogram::new();

        let range = 16384;

        // Create a non-uniform histogram
        histogram.insert(0, 10);
        histogram.insert(1, 20);
        histogram.insert(2, 10);
        histogram.insert(3, 10);

        let lut = create_lut(&histogram, range);

        assert_eq!(lut.len(), range);

        // Test specific values in the LUT
        // These values depend on the exact histogram and total pixels
        let expected_values = vec![0, 8192, 12287, 16383];
        for (i, &expected) in expected_values.iter().enumerate() {
            assert_eq!(lut[i], expected, "LUT value at index {} is incorrect", i);
        }
    }
}
