
pub mod test_utils {
    use std::time::Duration;

    use tauri::{Manager, AppHandle, test::MockRuntime};
    use tauri_plugin_log::{TargetKind, fern::colors::ColoredLevelConfig, Target};

    use crate::capture::detector::DetectorController;


    pub fn create_app<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::App<R> {
        builder
            .setup(|_| {
                Ok(())
            })
            .plugin(
                tauri_plugin_log::Builder::default()
                    .targets([
                        Target::new(TargetKind::Stdout),
                    ])
                    .with_colors(ColoredLevelConfig::default())
                    .build(),
            )
            .build(tauri::generate_context!())
            .expect("failed to build app")
    }

    pub fn setup_controller_handle(app_handle: AppHandle<MockRuntime>) -> DetectorController {
        let controller = DetectorController::new(|status| {});
        std::thread::sleep(Duration::from_secs(2));
        controller
    }

    pub fn setup_controller() -> DetectorController {
        let app = create_app(tauri::test::mock_builder());
        let controller = DetectorController::new(|_| {});
        std::thread::sleep(Duration::from_secs(2));
        controller
    }
}