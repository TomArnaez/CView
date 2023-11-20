use crate::image::Annotation;
use crate::image::ImageService;
use image::EncodableLayout;
use log::info;
use std::sync::Mutex;
use tauri::ipc::Response;
use tauri::{AppHandle, Manager, State};

#[tauri::command(async)]
#[specta::specta]
pub fn get_image_binary(
    image_service_mutex: State<'_, Mutex<ImageService>>,
    stack_idx: u32,
    image_idx: u32,
    resize: bool,
) -> Response {
    let image_service = image_service_mutex.lock().unwrap();

    if let Some(image_handler) = image_service.get_handler(stack_idx as usize, image_idx as usize) {
        let mut image = image_handler.get_image();
        if resize {
            image =
                image_lib::imageops::resize(&image, 100, 100, image::imageops::FilterType::Nearest);
        }
        Response::new(image.as_bytes().to_owned())
    } else {
        Response::new(vec![])
    }
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

#[tauri::command(async)]
#[specta::specta]
pub fn remove_image_stack(image_service_mutex: State<Mutex<ImageService>>, stack_idx: u32) {
    let mut image_service = image_service_mutex.lock().unwrap();
    image_service.remove_image_stack(stack_idx as usize);
}
