use log::{error, info};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use specta::Type;
use std::path::PathBuf;
use std::{collections::HashMap, fs};
use tauri::AppHandle;
use tauri::{path::BaseDirectory, Manager};
use tauri_specta::Event;

use crate::capture::capture::CaptureError;
use crate::capture::capture_manager::CapturedImage;
use crate::events::AppDataEvent;
use crate::wrapper::SLImageRs;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Type, Debug)]
pub struct AppData {
    dark_maps_files: HashMap<u32, PathBuf>,
    defect_map: Option<PathBuf>,
}

impl AppData {
    pub fn new(app: AppHandle) -> Self {
        let app_data_dir = app.path().app_data_dir().unwrap();

        info!(
            "Looking for defect map resources at {}",
            app_data_dir.display()
        );

        let mut dark_maps_files: HashMap<u32, PathBuf> = HashMap::new();

        let pattern = r"DarkMap_(\d+)ms\.tif";
        let regex = Regex::new(pattern).expect("Invalid regex pattern");

        let dark_map_path = app_data_dir.join("DarkMaps/");

        if let Ok(paths) = fs::read_dir(dark_map_path) {
            for path in paths {
                if let Ok(file) = path {
                    let file_name = file.file_name().to_str().unwrap().to_owned();
                    if let Some(captures) = regex.captures(&file_name) {
                        if let Some(dark_map_exp_str) = captures.get(1) {
                            let dark_map_exp = dark_map_exp_str.as_str().parse::<u32>();
                            match dark_map_exp {
                                Ok(exp_time) => {
                                    info!(
                                        "Found dark map with exp time {exp_time} {}",
                                        file.path().display()
                                    );
                                    dark_maps_files.insert(exp_time, file.path());
                                }
                                Err(err) => {
                                    error!("Failed to parse decimal value: {:?}", err);
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut defect_map = None;

        let path = app_data_dir.join(PathBuf::from("DefectMap/DefectMap.tif"));
        if (path.exists()) {
            defect_map = Some(path)
        }

        let app_data = AppData {
            dark_maps_files,
            defect_map,
        };

        AppDataEvent(app_data.clone()).emit_all(&app).unwrap();
        app.emit("app-data-event", app_data.clone());

        app_data
    }

    pub fn set_dark_maps(
        &mut self,
        app: AppHandle,
        dark_maps: Vec<CapturedImage>,
    ) -> Result<(), ()> {
        self.dark_maps_files.clear();
        match app.path().app_data_dir() {
            Ok(app_data_path) => {
                for captured_image in dark_maps {
                    let img = captured_image.data;
                    let exp_time = captured_image.metadata.capture_settings.unwrap().exp_time;
                    let dark_map_path =
                        app_data_path.join(format!("DarkMaps\\DarkMap_{exp_time}ms.tif"));

                    match img.save_with_format(&dark_map_path, image::ImageFormat::Tiff) {
                        Ok(()) => {
                            info!("Successfully saved dark map to {}", dark_map_path.display());
                            self.dark_maps_files.insert(exp_time, dark_map_path);
                        }
                        Err(e) => {
                            error!(
                                "Failed to save dark map to {} with error {e}",
                                dark_map_path.display()
                            );
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(()),
        }
    }

    pub fn dark_map(&self, exp_time: u32) -> Result<&PathBuf, CaptureError> {
        match self.dark_maps_files.get(&exp_time) {
            Some(path) => return Ok(path),
            None => Err(CaptureError::Unknown),
        }
    }

    pub fn dark_maps(&self) -> &HashMap<u32, PathBuf> {
        &self.dark_maps_files
    }

    pub fn defect_map(&self) -> Option<&PathBuf> {
        self.defect_map.as_ref().clone()
    }

    pub fn set_defect_map(&mut self, app: AppHandle, mut defect_map: SLImageRs) -> Result<(), ()> {
        let path = app
            .path()
            .resolve(format!("DefectMap/DefectMap.tif"), BaseDirectory::AppData)
            .unwrap();
        match defect_map.write_tiff_image(&path) {
            Ok(()) => {
                info!("Successfully saved defect map to {}", path.display());
                AppDataEvent(self.clone()).emit_all(&app).unwrap();
                Ok(())
            }
            Err(e) => {
                error!("Failed to save defect map to {}", path.display());
                Err(())
            }
        }
    }
}

unsafe impl Send for AppData {}
unsafe impl Sync for AppData {}
