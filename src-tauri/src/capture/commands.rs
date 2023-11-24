use crate::capture::types::CaptureProgressEvent;
use crate::capture::types::CaptureStreamItem;
use crate::events::StreamCaptureEvent;
use crate::image::ImageStack;
use crate::ImageService;
use crate::StreamBuffer;
use chrono::Utc;
use futures_util::{pin_mut, StreamExt};
use log::debug;
use log::error;
use log::info;
use tauri::Manager;
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
    save_capture: bool,
) -> Result<(), CaptureError> {
    let mut capture_result = None;

    let stream = capture_manager_mutex
        .lock()
        .unwrap()
        .start_capture(app.clone(), capture.clone())?;

    pin_mut!(stream);

    while let Some(stream_item) = stream.next().await {
        match stream_item {
        CaptureStreamItem::Image(image_handler) => {
            let stream_buffer = stream_buffer_mutex.lock().unwrap();
            match stream_buffer.q.push(image_handler) {
                Err(e) => error!("Failed to push to stream buffer with e {e}"),
                _ => {}
            }
            match StreamCaptureEvent().emit_all(&app) {
                Err(e) => error!("Failed to stream capture event event with error {e}"),
                _ => {}
            }

        },
        CaptureStreamItem::CaptureResult(vec) => {
            capture_result = Some(vec);
        },
        CaptureStreamItem::Progress(progress) => {
            info!("got progress event");
            match CaptureProgressEvent(progress).emit_all(&app) {
                Err(e) => error!("Failed to emit capture progress event with error {e}"),
                _ => {}
                }
            },
        }
    }

    if let Some(capture_result) = capture_result {
        info!("go capture result");
        let image_stack = ImageStack {
            timestamp: Some(Utc::now()),
            image_handlers: capture_result,
            capture: Some(capture),
        };

        image_service_mutex
        .lock()
        .unwrap()
        .add_image_stack(image_stack);
    }

    if (save_capture) {
        if let Ok(local_data_dir) = app.path().local_data_dir() {

        }
    }

    Ok(())
}

#[tauri::command(async)]
#[specta::specta]
pub fn stop_capture(capture_manager_mutex: State<Mutex<CaptureManager>>, stream_buffer_mutex: State<Mutex<StreamBuffer>>) {
    info!("Stopping capture");
    let capture_manager = capture_manager_mutex.lock().unwrap();
    capture_manager.stop_capture();
    info!("Clearing stream buffer");
    stream_buffer_mutex.lock().unwrap().clear();
}

#[tauri::command(async)]
pub fn read_stream_buffer(stream_buffer_mutex: State<Mutex<StreamBuffer>>, saturated_pixel_threshold: Option<u32>, saturated_pixel_RGB_colour: String) -> Response {
    debug!("Reading stream buffer");
    let stream_buffer = stream_buffer_mutex.lock().unwrap();
    
    if let Ok(image_handler) = stream_buffer.q.pop() {
        let mut return_data = Vec::new();
        return_data.extend_from_slice(&image_handler.image.width().to_le_bytes());
        return_data.extend_from_slice(&image_handler.image.height().to_le_bytes());
        return_data.append(&mut image_handler.get_rgba_image(saturated_pixel_threshold, None));
        return Response::new(return_data);
    }
    Response::new(vec![])
}

#[tauri::command(async)]
#[specta::specta]
pub async fn generate_defect_map(
    app: AppHandle,
    capture_manager_mutex: State<'_, Mutex<CaptureManager>>,
) -> Result<(), ()> {
    info!("Generating Defect Maps");
    capture_manager_mutex
        .lock()
        .unwrap()
        .generate_defect_map(app, vec![100, 200, 300], 10);
    Ok(())
}

#[tauri::command(async)]
#[specta::specta]
pub async fn generate_dark_maps(
    app: AppHandle,
    capture_manager_mutex: State<'_, Mutex<CaptureManager>>,
) -> Result<(), CaptureError> {
    info!("Generating Dark Maps");
    capture_manager_mutex
        .lock()
        .unwrap()
        .generate_dark_maps(app, vec![100, 200, 300], 10);
    Ok(())
}
