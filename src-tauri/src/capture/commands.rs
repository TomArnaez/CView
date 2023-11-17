use crate::appdata::AppData;
use crate::capture::advanced_capture::DarkMapCapture;
use crate::capture::capture_manager::AdvCaptureMessage;
use crate::capture::types::CaptureProgressEvent;
use crate::events::StreamCaptureEvent;
use crate::image::{ImageHandler, ImageStack};
use crate::ImageService;
use crate::StreamBuffer;
use chrono::Utc;
use log::error;
use log::info;
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
pub fn run_capture(
    app: AppHandle,
    image_service_mutex: State<Mutex<ImageService>>,
    stream_buffer_mutex: State<Mutex<StreamBuffer>>,
    capture_manager_mutex: State<Mutex<CaptureManager>>,
    capture: AdvancedCapture,
) -> Result<(), CaptureError> {
    info!("Called Run Capture with {:?}", capture);

    let mut capture_manager = capture_manager_mutex.lock().unwrap();
    let rx = capture_manager.run_capture(app.clone(), capture.clone())?;
    drop(capture_manager);

    loop {
        match rx.recv() {
            Ok(message) => match message {
                AdvCaptureMessage::CapturedImage(image) => {
                    let mut image_handler = ImageHandler::new(image.data, image.metadata);
                    //image_handler.apply_histogram_equilization();
                    let stream_buffer = stream_buffer_mutex.lock().unwrap();
                    stream_buffer.q.push(image_handler);
                    StreamCaptureEvent().emit_all(&app).unwrap();
                }
                AdvCaptureMessage::CaptureCompleted(vec) => {
                    let mut image_handlers = Vec::new();
                    for image in vec {
                        let mut image_handler = ImageHandler::new(image.data, image.metadata);
                        image_handler.apply_histogram_equilization();
                        image_handlers.push(image_handler);
                    }

                    let mut image_service = image_service_mutex.lock().unwrap();
                    image_service.add_image_stack(ImageStack {
                        timestamp: Some(Utc::now()),
                        image_handlers,
                        capture: Some(capture),
                    });

                    info!("finished");
                    break;
                }
                AdvCaptureMessage::Error => return Err(CaptureError::DetectorDisconnected),
                AdvCaptureMessage::Progress(progress) => {
                    CaptureProgressEvent(progress).emit_all(&app).unwrap();
                }
                AdvCaptureMessage::Stopped => return Ok(()),
            },
            Err(e) => {
                error!(
                    "Receiving advanced capture message crashed with error {}",
                    e
                );
                return Err(CaptureError::DetectorDisconnected);
            }
        }
    }

    Ok(())
}

#[tauri::command(async)]
#[specta::specta]
pub fn stop_capture(capture_manager_mutex: State<Mutex<CaptureManager>>) {
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
pub fn generate_dark_maps(
    app: AppHandle,
    app_data_mutex: State<Mutex<AppData>>,
    capture_manager_mutex: State<Mutex<CaptureManager>>,
) -> Result<(), CaptureError> {
    info!("Generating Dark Maps");
    let capture = AdvancedCapture::DarkMapCapture(DarkMapCapture {
        exp_times: vec![100, 200],
        frames_per_capture: 10,
    });

    let mut capture_manager = capture_manager_mutex.lock().unwrap();
    let rx = capture_manager.run_capture(app.clone(), capture.clone())?;

    loop {
        match rx.recv() {
            Ok(message) => match message {
                AdvCaptureMessage::CapturedImage(_) => {}
                AdvCaptureMessage::CaptureCompleted(dark_maps) => {
                    info!("Finished generating dark maps");

                    info!("{}", dark_maps.len());

                    let mut app_data = app_data_mutex.lock().unwrap();
                    match app_data.set_dark_maps(app.clone(), dark_maps) {
                        Ok(()) => info!("Succesfully saved dark maps"),
                        Err(()) => {
                            error!("Couldn't save dark maps");
                            break;
                        }
                    }
                }
                AdvCaptureMessage::Error => return Err(CaptureError::Unknown),
                AdvCaptureMessage::Progress(p) => {}
                AdvCaptureMessage::Stopped => return Ok(()),
            },
            Err(e) => {
                error!("Receiving image crashed with error {}", e);
                return Err(CaptureError::Unknown);
            }
        }
    }

    Ok(())
}
