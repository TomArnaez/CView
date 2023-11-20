pub mod test_utils {
    use std::time::Duration;

    use tauri::{test::MockRuntime, AppHandle};
    use tauri_plugin_log::{fern::colors::ColoredLevelConfig, Target, TargetKind};

    use crate::capture::{detector::DetectorController, types::CaptureManagerEvent};

    pub fn create_app<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::App<R> {
        let specta_builder =
            tauri_specta::ts::builder().events(tauri_specta::collect_events!(CaptureManagerEvent));

        builder
            .setup(|_| Ok(()))
            .plugin(specta_builder.into_plugin())
            .plugin(
                tauri_plugin_log::Builder::default()
                    .targets([Target::new(TargetKind::Stdout)])
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
