use futures_core::stream::Stream;
use futures_util::StreamExt;
use log::{debug, error, info};
use serde::Serialize;
use specta::Type;
use std::{
    collections::HashMap,
    future::{self},
    pin::Pin,
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread,
    time::Duration,
};
use tokio_util::sync::CancellationToken;

use crate::wrapper::{SLDeviceRS, SLImageRs};

use super::{
    capture::{CaptureError, CaptureSetting},
    capture_manager::CorrectionMaps,
};

// Required for opening camera on a detector
const BUFFER_DEPTH: u32 = 100;
const HEARTBEAT_REFRESH_TIME_MILLIS: u64 = 100;

#[derive(PartialEq, Clone, Serialize, Debug, Type)]
pub enum DetectorStatus {
    Available,
    Disconnected,
    Capturing,
}

#[derive(Clone)]
pub struct DetectorController {
    detector: SLDeviceRS,
    detector_status: Arc<Mutex<DetectorStatus>>,
}

impl DetectorController {
    pub fn new<F>(heartbeat_callback: F) -> Self
    where
        F: FnMut(DetectorStatus) + Send + 'static,
    {
        let controller = DetectorController {
            detector: SLDeviceRS::new(),
            detector_status: Arc::new(Mutex::new(DetectorStatus::Disconnected)),
        };

        Self::launch_heartbeat_thread::<F>(
            controller.detector.clone(),
            controller.detector_status.clone(),
            heartbeat_callback,
        );

        controller
    }

    pub fn run_capture_stream(
        &mut self,
        capture_settings: CaptureSetting,
        correction_maps: CorrectionMaps,
    ) -> Pin<Box<dyn Stream<Item = SLImageRs> + Send>> {
        let stream = capture_settings
            .capture_mode
            .stream_results(capture_settings.exp_time, self.detector.clone());

        stream
            .unwrap()
            .map(move |mut image| {
                if capture_settings.corrected {
                    if correction_maps
                        .dark_correct_image(&mut image, capture_settings.exp_time)
                        .is_ok()
                    {
                        info!("Dark Correction Successful");
                    } else {
                        error!("Couldn't access dark map");
                    }

                    if let Ok(corrected_image) =
                        correction_maps.defect_correct_image(&mut image, capture_settings.exp_time)
                    {
                        return corrected_image;
                    } else {
                        return image;
                    }
                } else {
                    image
                }
            })
            .boxed()
    }

    pub fn stop_capture(&mut self) {
        self.detector.go_unlive(true);
    }

    fn launch_heartbeat_thread<F>(
        mut detector: SLDeviceRS,
        detector_status_mutex: Arc<Mutex<DetectorStatus>>,
        mut heartbeat_callback: F,
    ) where
        F: FnMut(DetectorStatus) + Send + 'static,
    {
        info!("Launching heartbeat thread");
        tauri::async_runtime::spawn(async move {
            loop {
                thread::sleep(Duration::from_millis(HEARTBEAT_REFRESH_TIME_MILLIS));

                let mut detector_status = detector_status_mutex.lock().unwrap();

                match *detector_status {
                    DetectorStatus::Disconnected => match detector.open_camera(BUFFER_DEPTH) {
                        Ok(_) => {
                            info!("Connected to device");
                            *detector_status = DetectorStatus::Available;
                        }
                        Err(e) => debug!("Error opening camera"),
                    },
                    DetectorStatus::Available | DetectorStatus::Capturing => {
                        if !detector.is_connected() {
                            info!("Disconnected from device");
                            *detector_status = DetectorStatus::Disconnected;
                        }
                    }
                }

                heartbeat_callback(detector_status.clone());
            }
        });
    }
}

unsafe impl Send for DetectorController {}
unsafe impl Sync for DetectorController {}

#[cfg(test)]
mod tests {
    use crate::capture::types::AdvCapture;
    use crate::wrapper::{ExposureModes, SLDeviceRS, SLImageRs};
    use std::pin::{self, Pin};
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex};

    use super::DetectorController;
    use async_stream::stream;
    use futures::stream::{self, StreamExt}; // import StreamExt for chain method
    use futures::Stream;
    use futures_util::pin_mut;
    use tauri::Manager;

    fn create_app<R: tauri::Runtime>(mut builder: tauri::Builder<R>) -> tauri::App<R> {
        builder
            .setup(|app| {
                // do something
                Ok(())
            })
            .build(tauri::generate_context!())
            .expect("failed to build app")
    }

    fn test(count: u32, mut detector: SLDeviceRS) -> Pin<Box<dyn Stream<Item = SLImageRs>>> {
        let stream = stream! {
            println!("called");
            detector.set_exposure_time(100);
            detector.set_exposure_mode(ExposureModes::seq_mode);
            detector.set_number_frames(count);
            detector.go_live();
            detector.software_trigger();

            for frame_num in 0..count {
                let mut image = SLImageRs::new(
                    detector.image_height().unwrap(),
                    detector.image_width().unwrap(),
                );

                while detector
                    .read_buffer(&mut image, frame_num as u32, 100)
                    .is_err() {}
                yield image;
            }

            detector.go_unlive(true);
            println!("finished");
        };
        stream.boxed()
    }

    /*
    #[tokio::test]
    async fn run_capture_stream() {
        let app = create_app(tauri::test::mock_builder());
        let mut controller = DetectorController::new(|f| {});

        std::thread::sleep(Duration::from_secs(1));

        let stream_capture = Box::new(SequenceCapture { num_frames: 100 });
        let capture_settings = CaptureSettingBuilder::new(100, stream_capture).build();

        let mut stream = controller.run_capture_stream(capture_settings).unwrap();

        while let Some(mut item) = stream.next().await {
            println!("Item: {}", item.get_width());
        }
    }

    #[tokio::test]
    async fn test_signal_accum() {
        let app = create_app(tauri::test::mock_builder());
        let mut controller = Arc::new(Mutex::new(DetectorController::new(|f| {})));

        let signal_accum_capture = SignalAccumulationCapture {
            exp_times: vec![],
            frames_per_capture: 100,
        };
        std::thread::sleep(Duration::from_secs(1));

        let mut stream = signal_accum_capture.start_stream(
            app.handle().clone(),
            controller,
            Arc::new(AtomicBool::new(false)),
        );

        while let Some(image) = stream.next().await {
            println!("got image");
        }
    }

    #[tokio::test]
    async fn test_multi_capture() {
        let app = create_app(tauri::test::mock_builder());
        let controller = Arc::new(Mutex::new(DetectorController::new(|f| {})));

        let multi_capture = MultiCapture {
            exp_times: vec![100, 200],
            frames_per_capture: 10,
        };
        std::thread::sleep(Duration::from_secs(1));

        let mut stream = multi_capture.start_stream(
            app.handle().clone(),
            controller,
            Arc::new(AtomicBool::new(false)),
        );

        while let Some(image) = stream.next().await {
            println!("got image");
        }
    }

    #[tokio::test]
    async fn test_cancel_capture() {
        let app = create_app(tauri::test::mock_builder());
        let controller = Arc::new(Mutex::new(DetectorController::new(|f| {})));

        let multi_capture = MultiCapture {
            exp_times: vec![100, 200],
            frames_per_capture: 10,
        };
        std::thread::sleep(Duration::from_secs(1));

        let stop_signal = Arc::new(AtomicBool::new(false));

        let mut stream =
            multi_capture.start_stream(app.handle().clone(), controller, stop_signal.clone());

        while let Some(image) = stream.next().await {
            println!("got image");
            stop_signal.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
    */
}
