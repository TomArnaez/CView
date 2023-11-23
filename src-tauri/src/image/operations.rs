use std::sync::{Arc, Mutex};

use image::{ImageBuffer, Luma};
use log::info;
use rayon::prelude::*;

use crate::charts::types::HistogramBin;

use super::DataExtractor;

type Histogram = Vec<u32>;

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

pub fn calculate_histogram<'a, I>(vals: I, max_value: u32, num_bins: u32) -> Histogram
where
    I: IntoIterator<Item = &'a u16>,
{
    let bin_size = (max_value + 1) / num_bins;

    vals.into_iter()
        .fold(vec![0; num_bins as usize], |mut histogram, &value| {
            let bin_index = (value as u32 / bin_size) as usize;
            histogram[bin_index] += 1;
            histogram
        })
}

pub fn calculate_histogram_min_max(vals: Vec<&u16>, num_bins: u32) -> Vec<HistogramBin> {
    let min_value = match vals.iter().min() {
        Some(&&val) => val as u32,
        None => return Vec::new(),
    };

    let max_value = match vals.iter().max() {
        Some(&&val) => val as u32,
        None => return Vec::new(),
    };

    let range = max_value - min_value;
    let bin_size = if range < num_bins { 1 } else { (range + 1) / num_bins };

    let mut bins = vec![HistogramBin { range: 0, count: 0 }; num_bins as usize];

    for &value in vals.iter() {
        let value = *value as u32;
        let bin_index = if bin_size > 0 {
            ((value - min_value) / bin_size) as usize
        } else {
            0
        };

        let bin_index = bin_index.min(num_bins as usize - 1);
        bins[bin_index].count += 1;
    }

    // Update range for each bin
    for (i, bin) in bins.iter_mut().enumerate() {
        bin.range = min_value + (i as u32 * bin_size);
    }

    bins
}

fn create_lut(histogram: &Histogram) -> Vec<u32> {
    let num_pixels: u32 = histogram.iter().sum();
    let max_intensity: u32 = 16383; // Maximum intensity for a 14-bit image
    let scale_factor = max_intensity as f32 / num_pixels as f32;

    let mut cdf_min = None;
    let mut cdf = 0;
    let mut lut = Vec::with_capacity(histogram.len());

    for &freq in histogram.iter() {
        if freq > 0 && cdf_min.is_none() {
            cdf_min = Some(cdf);
        }
        cdf += freq;
        let lut_value = if let Some(cdf_min) = cdf_min {
            ((cdf - cdf_min) as f32 * scale_factor).round() as u32
        } else {
            0
        };
        lut.push(lut_value.min(max_intensity));
    }

    lut
}
pub trait HistogramEquilisation {
    fn cumulative_histogram(&self, range: usize) -> Vec<u32>;
    fn cumulative_histogram_roi(&self, roi: &dyn DataExtractor, range: usize) -> Vec<u32>;
}

impl HistogramEquilisation for ImageBuffer<image::Luma<u16>, Vec<u16>> {
    fn cumulative_histogram(&self, range: usize) -> Vec<u32> {
        let histogram = calculate_histogram(self.iter(), 16383, 16384);
        create_lut(&histogram)
    }

    fn cumulative_histogram_roi(&self, roi: &dyn DataExtractor, range: usize) -> Vec<u32> {
        let histogram = calculate_histogram(roi.iter_values(&self), 16383, 16384);
        create_lut(&histogram)
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
    use std::time::{Duration, Instant};

    use super::*;

    fn measure_time<F>(f: F) -> Duration
    where
        F: FnOnce(),
    {
        let start = Instant::now();
        f();
        start.elapsed()
    }

    #[test]
    fn test_histogram_time() {
        let data: Vec<u16> = vec![16384; 1031 * 1536];

        let duration = measure_time(|| {
            let histogram = calculate_histogram(&data, 16383, 16384);
            let lut = create_lut(&histogram);
            println!("{:?}", lut);
        });
        println!("Time for find_min_single: {:?}", duration);
    }
}
