// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod appdata;
mod capture {
    pub mod advanced_capture;
    pub mod capture;
    pub mod capture_manager;
    pub mod commands;
    pub mod corrections;
    pub mod detector;
    pub mod test_utils;
    pub mod types;
}
mod charts {
    pub mod charts;
    pub mod commands;
    pub mod types;
}
mod commands {
    pub mod file;
    pub mod image;
}
mod events;
mod image;
mod operations;
mod statistics;
mod utils;
mod wrapper;

extern crate image as image_lib;

use appdata::AppData;
use capture::{
    capture_manager::CaptureManager,
    types::{CaptureManagerEvent, CaptureProgressEvent},
};
use charts::types::LineProfileEvent;
use concurrent_queue::ConcurrentQueue;
use events::{
    AppDataEvent, CancelCaptureEvent, HistogramEvent, ImageStateEvent, StreamCaptureEvent,
};
use http::{header::*, response::Builder as ResponseBuilder};
use image::{ImageHandler, ImageService};
use log::{error, info};
use regex::Regex;
use std::{collections::HashMap, fs, path::PathBuf, sync::Mutex};
use tauri::{http, AppHandle, Manager, Runtime};

use tauri_plugin_log::{fern::colors::ColoredLevelConfig, Target, TargetKind};

use crate::wrapper::SLImageRs;

pub struct StreamBuffer {
    pub q: ConcurrentQueue<ImageHandler>,
}

impl StreamBuffer {
    fn new(size: usize) -> StreamBuffer {
        let buffer = StreamBuffer {
            q: ConcurrentQueue::<ImageHandler>::bounded(size),
        };
        buffer
    }
}

fn main() {
    let specta_builder = {
        let specta_builder = tauri_specta::ts::builder()
            .commands(tauri_specta::collect_commands![
                capture::commands::generate_dark_maps,
                capture::commands::run_capture,
                capture::commands::stop_capture,
                commands::file::startup,
                commands::file::open_images,
                commands::file::save_image,
                commands::file::save_stack,
                commands::image::histogram_equilization,
                commands::image::get_pixel_value,
                commands::image::update_roi,
                commands::image::invert_colours,
                commands::image::rotate,
                charts::commands::subscribe_chart,
            ])
            .events(tauri_specta::collect_events!(
                StreamCaptureEvent,
                CaptureProgressEvent,
                CancelCaptureEvent,
                AppDataEvent,
                CaptureManagerEvent,
                ImageStateEvent,
                LineProfileEvent,
                HistogramEvent
            ));

        #[cfg(debug_assertions)]
        let specta_builder = specta_builder.path("../src/bindings.ts");

        specta_builder.into_plugin()
    };

    tauri::Builder::default()
        .plugin(specta_builder)
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                ])
                .with_colors(ColoredLevelConfig::default())
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window::init())
        .setup(|app| {
            let handle = app.app_handle();

            app.manage(Mutex::new(CaptureManager::new(handle.clone())));
            app.manage(Mutex::new(AppData::new(handle.clone())));
            app.manage(Mutex::new(ImageService::new(handle.clone())));
            app.manage(Mutex::new(StreamBuffer::new(10)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::image::get_image_binary,
            capture::commands::run_capture,
            capture::commands::read_stream_buffer
        ])
        .register_asynchronous_uri_scheme_protocol("stream", move |app, request, responder| {
            let handle = app.clone();

            let stream_buffer_mutex: tauri::State<'_, Mutex<StreamBuffer>> =
                handle.state::<Mutex<StreamBuffer>>();
            let stream_buffer = stream_buffer_mutex.lock().unwrap();

            if let Ok(image_handler) = stream_buffer.q.pop() {
                std::thread::spawn(move || {
                    responder.respond(
                        ResponseBuilder::new()
                            .header(CONTENT_TYPE, "text/plain")
                            .header("Access-Control-Allow-Origin", "*") // Set CORS heade
                            .body(image_handler.get_image_as_bytes())
                            .unwrap(),
                    );
                });
            } else {
                responder.respond(
                    ResponseBuilder::new()
                        .header("Access-Control-Allow-Origin", "*")
                        .status(404)
                        .body(Vec::new())
                        .unwrap(),
                );
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
