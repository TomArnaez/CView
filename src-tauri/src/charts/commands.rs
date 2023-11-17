use crate::{
    charts::charts::{HistogramSubscriber, LineProfileSubscriber},
    image::ImageService,
};
use log::info;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

use super::types::Chart;

#[tauri::command(async)]
#[specta::specta]
pub fn subscribe_chart(
    image_service_mutex: State<Mutex<ImageService>>,
    app: AppHandle,
    label: String,
    image_idx: u32,
    stack_idx: u32,
    chart_type: Chart,
) {
    info!("Subscribe command called");
    let mut image_service = image_service_mutex.lock().unwrap();

    if let Some(window) = app.get_window(&label) {
        if let Some(image_handler) =
            image_service.get_mut_handler(stack_idx as usize, image_idx as usize)
        {
            match chart_type {
                Chart::Histogram => {
                    image_handler.subscribe(Box::new(HistogramSubscriber { app, window }));
                }
                Chart::LineProfile => {
                    image_handler.subscribe(Box::new(LineProfileSubscriber { app, window }));
                }
            }
        }
    }
}
