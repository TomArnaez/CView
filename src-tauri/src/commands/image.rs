use crate::image::Annotation;
use crate::image::ImageService;
use image::imageops;
use image::DynamicImage;
use image::EncodableLayout;
use log::info;
use std::sync::Mutex;
use tauri::ipc::Response;
use tauri::{AppHandle, Manager, State};

#[tauri::command(async)]
#[specta::specta]
pub fn get_image_binary_rgba(
    image_service_mutex: State<'_, Mutex<ImageService>>,
    stack_idx: u32,
    image_idx: u32,
    saturated_pixel_threshold: Option<u32>,
    saturated_pixel_rgb: Option<String>,
    resize_size: Option<(u32, u32)>,
) -> Response {
    let image_service = image_service_mutex.lock().unwrap();

    if let Some(image_handler) = image_service.get_handler(stack_idx as usize, image_idx as usize) {
        let mut return_data = Vec::new();
        return_data.extend_from_slice(&image_handler.image.width().to_le_bytes());
        return_data.extend_from_slice(&image_handler.image.height().to_le_bytes());
        return_data.append(&mut image_handler.get_rgba_image(
            saturated_pixel_threshold,
            resize_size,
            None,
        ));
        return Response::new(return_data);
    }
    Response::new(vec![])
}

#[tauri::command(async)]
#[specta::specta]
pub fn get_pixel_value(
    image_service_mutex: State<Mutex<ImageService>>,
    x: u32,
    y: u32,
    stack_idx: u32,
    image_idx: u32,
) -> Option<u16> {
    let image_service = image_service_mutex.lock().unwrap();

    if let Some(image_handler) = image_service.get_handler(stack_idx as usize, image_idx as usize) {
        match image_handler.image.get_pixel_checked(x, y) {
            Some(val) => return Some(val[0]),
            None => return None,
        };
    }

    None
}

#[tauri::command(async)]
#[specta::specta]
pub fn update_roi(
    image_service_mutex: State<Mutex<ImageService>>,
    annotation: Annotation,
    image_idx: u32,
    stack_idx: u32,
) {
    let mut image_service = image_service_mutex.lock().unwrap();

    if let Some(image_handler) =
        image_service.get_mut_handler(stack_idx as usize, image_idx as usize)
    {
        image_handler.update_roi(annotation);
    }
}

#[tauri::command(async)]
#[specta::specta]
pub fn histogram_equilization(
    image_service_mutex: State<Mutex<ImageService>>,
    image_idx: u32,
    stack_idx: u32,
    app: AppHandle,
) {
    let mut image_service = image_service_mutex.lock().unwrap();

    if let Some(image_handler) =
        image_service.get_mut_handler(stack_idx as usize, image_idx as usize)
    {
        image_handler.apply_histogram_equilization();
        app.emit("image-modified", "").unwrap();
    }
}

#[tauri::command(async)]
#[specta::specta]
pub fn invert_colours(
    image_service_mutex: State<Mutex<ImageService>>,
    image_idx: u32,
    stack_idx: u32,
) {
    info!("Image command called: Invert colours");
    let mut image_service = image_service_mutex.lock().unwrap();

    if let Some(image_handler) =
        image_service.get_mut_handler(stack_idx as usize, image_idx as usize)
    {
        image_handler.invert_colours();
    }
}

#[tauri::command(async)]
#[specta::specta]
pub fn rotate(
    image_service_mutex: State<Mutex<ImageService>>,
    image_idx: u32,
    stack_idx: u32,
    rotate_left: bool,
) {
    info!("Image command called: Rotate");
    let mut image_service = image_service_mutex.lock().unwrap();

    if let Some(image_handler) =
        image_service.get_mut_handler(stack_idx as usize, image_idx as usize)
    {
        if rotate_left {
            image_handler.rotate_left();
        } else {
            image_handler.rotate_right();
        }
    }
}

#[tauri::command(async)]
#[specta::specta]
pub fn flip(
    image_service_mutex: State<Mutex<ImageService>>,
    image_idx: u32,
    stack_idx: u32,
    vertically: bool,
) {
    let mut image_service = image_service_mutex.lock().unwrap();
    if let Some(image_handler) =
        image_service.get_mut_handler(stack_idx as usize, image_idx as usize)
    {
        image_handler.flip(vertically);
    }
}
