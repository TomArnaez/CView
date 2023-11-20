use log::{error, info};
use serde::Serialize;
use specta::Type;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, Runtime};
use thiserror::Error;

use crate::wrapper::InternalSLError;
use crate::{appdata::AppData, wrapper::SLImageRs};
use std::process::Command;

#[derive(Error, Debug, Type, Serialize)]
pub enum CorrectionError {
    #[error("Internal SDK Error")]
    SLError(InternalSLError),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

pub fn run_defect_map_gen<T: Runtime>(app: AppHandle<T>) -> Result<(), CorrectionError> {
    let defect_images_path = app
        .path()
        .resolve("DefectMapGeneration/Images/", BaseDirectory::Resource)
        .unwrap();
    let defect_gen_exe_path = app
        .path()
        .resolve(
            "DefectMapGeneration/DefectMapGen.exe",
            BaseDirectory::Resource,
        )
        .unwrap();
    info!("{:?}", defect_gen_exe_path);

    let args = [
        defect_images_path.to_str().unwrap(),
        "1",
        "0",
        "-f",
        "-a",
        "-p",
    ];
    Command::new(defect_gen_exe_path)
        .args(args)
        .output()
        .expect("Failed to execute defect map generation");

    let defect_map_path = app
        .path()
        .resolve(
            "DefectMapGeneration/Images/GlobalDefectMap.tif",
            BaseDirectory::Resource,
        )
        .unwrap();

    Ok(())
}

pub fn apply_corrections<T: Runtime>(
    app: AppHandle<T>,
    exp_time: u32,
    image: &mut SLImageRs,
) -> Result<SLImageRs, CorrectionError> {
    let app_data_mutex = app.state::<Mutex<AppData>>();
    let app_data = app_data_mutex.lock().unwrap();

    let dark_map_path = app_data.dark_map(exp_time).unwrap();
    let defect_map_path = &app_data.defect_map().unwrap();
    let mut dark_map = SLImageRs::new(image.get_height(), image.get_width());

    if dark_map.read_tiff_image(&dark_map_path).is_err() {
        error!(
            "Failed to read dark map at path {}",
            dark_map_path.display().to_string()
        );
        return Err(CorrectionError::FileNotFound(
            dark_map_path.display().to_string(),
        ));
    }
    image
        .offset_correction(&mut dark_map, 300)
        .map_err(CorrectionError::SLError)?;

    let mut defect_map = SLImageRs::new(image.get_height(), image.get_width());

    if defect_map.read_tiff_image(&defect_map_path).is_err() {
        error!("Failed to read defect map");
        return Err(CorrectionError::FileNotFound(
            defect_map_path.display().to_string(),
        ));
    }

    let mut out_image = SLImageRs::new(image.get_height(), image.get_width());
    image
        .defect_correction(&mut out_image, &mut defect_map)
        .map_err(CorrectionError::SLError)?;

    Ok(out_image)
}

#[cfg(test)]
mod tests {
    use crate::capture::test_utils::test_utils::create_app;

    use super::run_defect_map_gen;

    #[test]
    pub fn defect_map_gen() {
        let app = create_app(tauri::test::mock_builder());
        run_defect_map_gen(app.handle().clone());
    }
}
