use log::{error, info};
use serde::Serialize;
use specta::Type;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, Runtime};
use thiserror::Error;

use crate::wrapper::InternalSLError;
use crate::{wrapper::SLImageRs};
use std::process::Command;

#[derive(Error, Debug, Type, Serialize)]
pub enum CorrectionError {
    #[error("Internal SDK Error")]
    SLError(InternalSLError),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

pub fn run_defect_map_gen(images_dir: &PathBuf, exe_dir: &PathBuf) -> Result<(), CorrectionError> {
    info!("Running defect map gen exe");
    let args = [
        images_dir.to_str().unwrap(),
        "1",
        "0",
        "-f",
        "-a",
        "-p",
    ];
    let mut cmd = Command::new(exe_dir);
    let cmd = cmd
        .args(args);

    let child =  cmd.spawn().unwrap();
    info!("Defect Gen Completed");
    Ok(())
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tauri::Manager;

    use crate::capture::test_utils::test_utils::create_app;

    use super::run_defect_map_gen;

    #[test]
    pub fn defect_map_gen() {
        let app = create_app(tauri::test::mock_builder());
        let images_dir = app.path().app_local_data_dir().unwrap().join("DefectMap");
        let exe_dir = PathBuf::from("C:\\dev\\repos\\CView\\src-tauri\\target\\debug\\resources\\DefectMapGeneration");
        let log_dir = app.path().app_log_dir().unwrap();
        println!("{}", log_dir.display());
        if (exe_dir.exists()) {
            println!("yay");
        }
        println!("{}", exe_dir.display());
        run_defect_map_gen(&images_dir, &exe_dir);
    }
}
