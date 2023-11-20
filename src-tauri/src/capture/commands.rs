use crate::capture::corrections::run_defect_map_gen;
use crate::events::StreamCaptureEvent;
use crate::image::ImageHandler;
use crate::image::ImageStack;
use crate::ImageService;
use crate::StreamBuffer;
use chrono::Utc;
use futures_util::{pin_mut, StreamExt};
use log::info;
use tauri::Manager;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::ipc::Response;
use tauri::AppHandle;
use tauri::State;
use tauri_specta::Event;

use super::capture::CaptureError;
use super::capture_manager::CaptureManager;
use super::types::AdvancedCapture;

#[tauri::command(async)]
#[specta::specta]
pub async fn run_capture(
    app: AppHandle,
    image_service_mutex: State<'_, Mutex<ImageService>>,
    stream_buffer_mutex: State<'_, Mutex<StreamBuffer>>,
    capture_manager_mutex: State<'_, Mutex<CaptureManager>>,
    capture: AdvancedCapture,
) -> Result<(), CaptureError> {
    info!("Called Run Capture with {:?}", capture);

    let stream = capture_manager_mutex
        .lock()
        .unwrap()
        .start_capture(app.clone(), capture.clone())?;

    pin_mut!(stream);

    let mut image_handlers: Vec<ImageHandler> = Vec::new();

    while let Some(image_handler) = stream.next().await {
        let image_handler_clone = ImageHandler::new(
            image_handler.image.clone(),
            image_handler.image_metadata.clone(),
        );
        image_handlers.push(image_handler_clone);

        let stream_buffer = stream_buffer_mutex.lock().unwrap();
        stream_buffer.q.push(image_handler);
        StreamCaptureEvent().emit_all(&app).unwrap();
    }

    image_service_mutex
        .lock()
        .unwrap()
        .add_image_stack(ImageStack {
            timestamp: Some(Utc::now()),
            image_handlers,
            capture: Some(capture),
        });

    Ok(())
}

#[tauri::command(async)]
#[specta::specta]
pub fn stop_capture(capture_manager_mutex: State<Mutex<CaptureManager>>) {
    info!("Stopping capture");
    let capture_manager = capture_manager_mutex.lock().unwrap();
    capture_manager.stop_capture();
}

#[tauri::command(async)]
pub fn read_stream_buffer(stream_buffer_mutex: State<Mutex<StreamBuffer>>) -> Response {
    info!("Reading stream buffer");
    let stream_buffer = stream_buffer_mutex.lock().unwrap();

    if let Ok(image_handler) = stream_buffer.q.pop() {
        return Response::new(image_handler.get_image_as_bytes());
    }

    Response::new(vec![])
}

#[tauri::command(async)]
#[specta::specta]
pub async fn generate_defect_map(app: AppHandle, capture_manager_mutex: State<'_, Mutex<CaptureManager>>) -> Result<(), ()> {
    info!("Generating Defect Maps");
    capture_manager_mutex.lock().unwrap().generate_defect_map(app.clone(), vec![100, 200, 300], 10);
    Ok(())
}

#[tauri::command(async)]
#[specta::specta]
pub async fn generate_dark_maps(
    app: AppHandle,
    capture_manager_mutex: State<'_, Mutex<CaptureManager>>,
) -> Result<(), CaptureError> {
    info!("Generating Dark Maps");
    capture_manager_mutex.lock().unwrap().generate_dark_maps(vec![100, 200, 300], 10);
    Ok(())
}
