use std::sync::Mutex;

use tauri::State;

use super::{ImageService, Annotation};

/*
#[tauri::command(async)]
#[specta::specta]
pub fn get_image_mean(image_idx: u32, stack_idx: u32, image_service_mutex: State<Mutex<ImageService>>, roi: Option<Annotation>) -> f64 {
    let image_service = image_service_mutex.lock().unwrap();
    
    if let Some(handler) = image_service.get_handler(stack_idx as usize, stack_idx as usize) {
        handler.
    }
}
*/