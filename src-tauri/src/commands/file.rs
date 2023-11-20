
use crate::image::{ImageService, ImageStack};
use image::io::Reader as ImageReader;
use log::info;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_specta::Event;

#[tauri::command(async)]
#[specta::specta]
pub fn open_images(app: AppHandle, image_service_mutex: State<Mutex<ImageService>>) {
    let mut image_service = image_service_mutex.lock().unwrap();

    if let Some(file_path) = app.dialog().file().blocking_pick_files() {
        let mut image_vec = Vec::new();

        for file in file_path.iter() {
            let img = ImageReader::open(file.path.clone())
                .unwrap()
                .decode()
                .unwrap()
                .to_luma16();
            image_vec.push(img);
        }

        image_service.add_image_stack(ImageStack::new(image_vec, None, None));
    }
}

#[tauri::command(async)]
#[specta::specta]
pub fn save_stack(
    app: AppHandle,
    image_service_mutex: State<'_, Mutex<ImageService>>,
    stack_idx: u32,
) {
    let image_service = image_service_mutex.lock().unwrap();

    if let Some(stack) = image_service.image_stacks.get(stack_idx as usize) {
        if let Some(file_path) = app
            .dialog()
            .file()
            .add_filter("TIFF", &["tiff"])
            .blocking_save_file()
        {
            stack.save(file_path);
        }
    }
}

#[tauri::command(async)]
#[specta::specta]
pub fn save_image(
    app: AppHandle,
    image_service_mutex: State<'_, Mutex<ImageService>>,
    stack_index: u32,
    image_index: u32,
) -> Result<(), ()> {
    let image_service = image_service_mutex.lock().unwrap();

    if let Some(file_path) = app
        .dialog()
        .file()
        .add_filter("TIFF", &["tif"])
        .set_file_name("test")
        .blocking_save_file()
    {
        image_service.save_image(
            stack_index as usize,
            image_index as usize,
            &file_path.as_path(),
        );
    }

    Ok(())
}
