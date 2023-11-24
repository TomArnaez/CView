use image::{ImageBuffer, Luma};
use image_lib::GenericImageView;
use log::info;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use crate::image::types::{Point, Rect};

pub fn snr(
    image: &ImageBuffer<Luma<u16>, Vec<u16>>,
    window_size: u32,
) -> Result<(f64, Rect, Rect), ()> {
    let mut min_mean: f64 = u32::MAX as f64;
    let mut max_mean: f64 = 0.0;

    let dark_offset: f64 = 300.0;

    let mut bg_rect = Rect {
        width: window_size,
        height: window_size,
        pos: Point { x: 0, y: 0 },
    };

    let mut fg_rect = Rect {
        width: window_size,
        height: window_size,
        pos: Point { x: 0, y: 0 },
    };

    let width: u32 = image.width();
    let height: u32 = image.height();

    if window_size > width || window_size > height {
        return Err(());
    }

    for x in 0..width - window_size {
        for y in 0..height - window_size {
            let window = image.view(x, y, window_size, window_size).to_image();

            let (window_mean, window_std_dev) = calculate_mean_and_std(&window);

            if (window_mean < min_mean) {
                min_mean = window_mean;
                bg_rect.pos.x = x;
                bg_rect.pos.y = y;
            }

            if (window_mean > max_mean) {
                max_mean = window_mean;
                fg_rect.pos.x = x;
                fg_rect.pos.y = y;
            }
        }
    }

    println!("{} {}", max_mean, min_mean);

    Ok((
        (max_mean - min_mean) / (min_mean - dark_offset).abs(),
        bg_rect,
        fg_rect,
    ))
}

pub fn snr_threaded(
    image: &ImageBuffer<Luma<u16>, Vec<u16>>,
    window_size: u32,
) -> Result<(f64, Rect, Rect), ()> {
    let width = image.width();
    let height = image.height();

    if window_size > width || window_size > height {
        return Err(());
    }

    struct SharedState {
        min_mean: f64,
        max_mean: f64,
        bg_rect: Rect,
        fg_rect: Rect,
    }

    let shared_state = Arc::new(Mutex::new(SharedState {
        min_mean: u32::MAX as f64,
        max_mean: 0.0,
        bg_rect: Rect {
            width: window_size,
            height: window_size,
            pos: Point { x: 0, y: 0 },
        },
        fg_rect: Rect {
            width: window_size,
            height: window_size,
            pos: Point { x: 0, y: 0 },
        },
    }));

    let integral_image = compute_integral_image(image);

    let positions = (0..width - (window_size - 1))
    .flat_map(move |x| (0..height - (window_size - 1)).map(move |y| (x, y)))
    .collect::<Vec<_>>();

    // Process each position concurrently
    (0..width - (window_size - 1)).into_par_iter().for_each(|x| {
        let mut local_min_mean = u32::MAX as f64;
        let mut local_max_mean = 0.0;
        let mut local_bg_rect = Rect {pos: Point { x: 0, y: 0}, width: 0, height :0}; // Replace with your default Rect
        let mut local_fg_rect = Rect {pos: Point { x: 0, y: 0}, width: 0, height :0}; // Replace with your default Rect

        for y in 0..height - (window_size - 1) {
            let sum = area_sum(&integral_image, x, y, window_size);
            let window_mean = sum as f64 / (window_size * window_size) as f64;

            if window_mean < local_min_mean {
                local_min_mean = window_mean;
                local_bg_rect = Rect { pos: Point { x, y}, width: window_size, height: window_size };
            }
            if window_mean > local_max_mean {
                local_max_mean = window_mean;
                local_fg_rect = Rect { pos: Point { x, y}, width: window_size, height: window_size };
            }
        }

        let mut state = shared_state.lock().unwrap();
        if local_min_mean < state.min_mean {
            state.min_mean = local_min_mean;
            state.bg_rect = local_bg_rect;
        }
        if local_max_mean > state.max_mean {
            state.max_mean = local_max_mean;
            state.fg_rect = local_fg_rect;
        }
    });

    let state = shared_state.lock().unwrap();
    let snr = (state.max_mean - state.min_mean) / (state.min_mean - 300.0).abs();

    Ok((snr, state.bg_rect.clone(), state.fg_rect.clone()))
}

pub fn calculate_mean_and_std_iter<'a, I>(vals: I) -> (f64, f64)
where
    I: IntoIterator<Item = &'a u16> + Clone,
{
    let mut sum = 0.0;
    let mut count = 0;

    // First pass to calculate mean
    for &val in vals.clone().into_iter() {
        sum += val as f64;
        count += 1;
    }

    if count == 0 {
        return (0.0, 0.0); // Avoid division by zero
    }

    let mean = sum / count as f64;

    // Second pass to calculate standard deviation
    let mut variance_sum = 0.0;
    for &val in vals.into_iter() {
        variance_sum += (val as f64 - mean).powi(2);
    }

    let variance = variance_sum / count as f64;
    let std_dev = variance.sqrt();

    (mean, std_dev)
}

fn calculate_mean_and_std(buffer: &ImageBuffer<Luma<u16>, Vec<u16>>) -> (f64, f64) {
    let width = buffer.width();
    let height = buffer.height();
    let mut sum = 0.0;
    let mut squared_diff_sum = 0.0;
    let total_pixels = (width * height) as f64;

    for y in 0..height {
        for x in 0..width {
            let pixel = buffer.get_pixel(x, y);
            sum += pixel[0] as f64;
        }
    }

    let mean = sum / total_pixels;

    for y in 0..height {
        for x in 0..width {
            let pixel = buffer.get_pixel(x, y);
            let diff = pixel[0] as f64 - mean;
            squared_diff_sum += diff * diff;
        }
    }

    let standard_deviation = (squared_diff_sum / total_pixels).sqrt();

    (mean, standard_deviation)
}

pub fn get_points_along_line(x1: isize, y1: isize, x2: isize, y2: isize) -> Vec<(isize, isize)> {
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

fn compute_integral_image(image: &ImageBuffer<Luma<u16>, Vec<u16>>) -> Vec<Vec<u64>> {
    let width = image.width() as usize;
    let height = image.height() as usize;

    // Create a 2D vector filled with zeros
    let mut integral_image = vec![vec![0u64; width]; height];

    for y in 0..height {
        for x in 0..width {
            let left = if x > 0 { integral_image[y][x - 1] } else { 0 };
            let above = if y > 0 { integral_image[y - 1][x] } else { 0 };
            let above_left = if x > 0 && y > 0 { integral_image[y - 1][x - 1] } else { 0 };

            // Pixel value at (x, y)
            let pixel_value = image.get_pixel(x as u32, y as u32).0[0] as u64;

            integral_image[y][x] = pixel_value + left + above - above_left;
        }
    }

    integral_image
}

fn area_sum(integral_image: &[Vec<u64>], x: u32, y: u32, window_size: u32) -> u64 {
    let x = x as usize;
    let y = y as usize;
    let window_size = window_size as usize;

    // Bottom right corner of the window
    let br_x = x + window_size - 1;
    let br_y = y + window_size - 1;

    // Sum within the window is calculated using the values at the corners of the window
    let sum_br = integral_image[br_y][br_x]; // Bottom right

    // The other three corners (top left, top right, bottom left)
    let sum_tl = if x > 0 && y > 0 {
        integral_image[y - 1][x - 1]
    } else {
        0
    };
    let sum_tr = if y > 0 {
        integral_image[y - 1][br_x]
    } else {
        0
    };
    let sum_bl = if x > 0 {
        integral_image[br_y][x - 1]
    } else {
        0
    };

    // Final sum calculation
    sum_br + sum_tl - sum_tr - sum_bl
}


#[cfg(test)]
mod tests {
    use std::ops::DerefMut;

    use super::*;
    use image::{GenericImage, ImageBuffer, Luma};
    use imageproc::integral_image;
    use log::info;

    fn create_test_image(width: u32, height: u32, value: u16) -> ImageBuffer<Luma<u16>, Vec<u16>> {
        ImageBuffer::from_pixel(width, height, Luma([value]))
    }

    
    fn set_region_to_value(
        img: &mut ImageBuffer<Luma<u16>, Vec<u16>>,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        value: u16,
    ) {
        for i in x..x + width {
            for j in y..y + height {
                if i < img.width() && j < img.height() {
                    img.put_pixel(i, j, Luma([value]));
                }
            }
        }
    }

    #[test]
    fn test_integral() {
        let test_image: ImageBuffer<Luma<u16>, Vec<u16>> = create_test_image(1031, 1536, 500);
        let integral_image = compute_integral_image(&test_image);
        let sum = area_sum(&integral_image, 50, 50, 50);
        println!("{}", sum);
    }

    #[test]
    fn test_snr() {
        let mut test_image: ImageBuffer<Luma<u16>, Vec<u16>> = create_test_image(1031, 1536, 500);
        let window_size = 50;

        set_region_to_value(&mut test_image, 0, 0, 50, 50, 100);
        let result = snr_threaded(&test_image, window_size);

        assert!(result.is_ok());
        let (snr, bg_rect, fg_rect) = result.unwrap();
        println!("{} {:?} {:?}", snr, bg_rect, fg_rect);
    }

    #[test]
    fn test_snr_normal_case() {
        let mut image: ImageBuffer<Luma<u16>, Vec<u16>> = create_test_image(1500, 1500, 100);
        set_region_to_value(&mut image, 0, 0, 5, 5, 500);

        let window_size = 5;

        // Replace the following line with the actual expected SNR value for the test image
        let expected_snr = 2.0;

        let x = snr_threaded(&mut image, window_size).unwrap();
        println!("{:?}", x);
    }
}
