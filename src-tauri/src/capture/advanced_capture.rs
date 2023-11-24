use super::{
    capture::{CaptureSettingBuilder, SequenceCapture, StreamCapture},
    detector::DetectorController,
    types::{AdvCapture, CaptureProgress, CaptureStreamItem},
};
use async_stream::stream;
use crate::{
    image::{
        snr_threaded, CaptureResultData, ImageHandler, ImageMetadata, ImageMetadataBuilder,
        SignalAccumulationData, SmartCaptureData,
    },
    wrapper::SLImageRs,
};

use futures::stream::{self, StreamExt};

use futures_core::Stream;
use image::{ImageBuffer, Luma};
use log::info;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{
    collections::HashMap,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::{AppHandle, Runtime};

#[derive(Clone, Serialize, Deserialize, Debug, Type, PartialEq)]
#[serde(tag = "type", rename = "SmartCapture")]
pub struct SmartCapture {
    pub exp_times: Vec<u32>,
    pub frames_per_capture: u32,
    pub window_size: u32,
    pub median_filtered: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type, PartialEq)]
#[serde(tag = "type", rename = "SignalAccumulation")]
pub struct SignalAccumulationCapture {
    pub exp_times: Vec<u32>,
    pub frames_per_capture: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type, PartialEq)]
#[serde(tag = "type", rename = "MultiCapture")]
pub struct MultiCapture {
    pub exp_times: Vec<u32>,
    pub frames_per_capture: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type, PartialEq)]
pub struct DarkMapCapture {
    pub exp_times: Vec<u32>,
    pub frames_per_capture: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type, PartialEq)]
pub struct DefectMapCapture {
    pub exp_times: Vec<u32>,
    pub frames_per_capture: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type, PartialEq)]
#[serde(tag = "type")]
pub struct LiveCapture {
    pub exp_time: u32,
}

impl AdvCapture for DarkMapCapture {
    fn start_stream(
        &self,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        dark_maps_mutex: Arc<Mutex<HashMap<u32, SLImageRs>>>,
        defect_map_mutex: Arc<Mutex<Option<SLImageRs>>>,
        stop_signal: Arc<AtomicBool>,
    ) -> Pin<Box<dyn Stream<Item = CaptureStreamItem> + Send>> {
        let stream = stream! { yield CaptureStreamItem::Progress(CaptureProgress::new(0, "test".to_string())) };
        Box::pin(stream)
    }
}

impl AdvCapture for DefectMapCapture {
    fn start_stream(
        &self,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        dark_maps_mutex: Arc<Mutex<HashMap<u32, SLImageRs>>>,
        defect_map_mutex: Arc<Mutex<Option<SLImageRs>>>,
        stop_signal: Arc<AtomicBool>,
    ) -> Pin<Box<dyn Stream<Item = CaptureStreamItem> + Send>> {
        let stream = stream! { yield CaptureStreamItem::Progress(CaptureProgress::new(0, "test".to_string())) };
        Box::pin(stream)
    }
}

impl AdvCapture for LiveCapture {
    fn start_stream(
        &self,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        dark_maps_mutex: Arc<Mutex<HashMap<u32, SLImageRs>>>,
        defect_map_mutex: Arc<Mutex<Option<SLImageRs>>>,
        stop_signal: Arc<AtomicBool>,
    ) -> Pin<Box<dyn Stream<Item = CaptureStreamItem> + Send>> {
        info!("Starting Live Capture");

        let capture_settings =
            CaptureSettingBuilder::new(self.exp_time, Box::new(StreamCapture { duration: None }))
                .build();
        let stream = detector_controller_mutex
            .lock()
            .unwrap()
            .run_capture_stream(
                capture_settings.clone(),
                dark_maps_mutex,
                defect_map_mutex,
                stop_signal.clone(),
            )
            .unwrap();

        let s = stream
            .map(move |mut image| {
                let mut image_handler = ImageHandler::new(
                    image.to_image_buffer(),
                    ImageMetadataBuilder::new()
                        .capture_settings(capture_settings.clone())
                        .build(),
                );
                image_handler.apply_histogram_equilization();
                CaptureStreamItem::Image(image_handler)
            })
            .boxed();

        s
    }
}

impl AdvCapture for SmartCapture {
    fn start_stream(
        &self,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        dark_maps_mutex: Arc<Mutex<HashMap<u32, SLImageRs>>>,
        defect_map_mutex: Arc<Mutex<Option<SLImageRs>>>,
        stop_signal: Arc<AtomicBool>,
    ) -> Pin<Box<dyn Stream<Item = CaptureStreamItem> + Send>> {
        info!("Starting Smart Capture");

        let best_capture: Arc<Mutex<(Option<ImageHandler>, f64)>> = Arc::new(Mutex::new((None, 0.0)));

        let streams = self
            .exp_times
            .iter()
            .map(|&exp_time| {
                let mut detector_controller = detector_controller_mutex
                    .lock()
                    .expect("Failed to acquire detector controller lock");

                let capture_settings = CaptureSettingBuilder::new(
                    exp_time,
                    Box::new(SequenceCapture {
                        num_frames: self.frames_per_capture,
                    }),
                )
                .build();

                let window_size = self.window_size;
                let best_capture = best_capture.clone();

                let progress_stream = stream::once(async move {
                    CaptureStreamItem::Progress(CaptureProgress::new(
                        0,
                        format!("Capturing images for exposure time {exp_time}ms").to_string(),
                    ))
                });

                let capture_stream = detector_controller
                    .run_capture_stream(
                        capture_settings.clone(),
                        dark_maps_mutex.clone(),
                        defect_map_mutex.clone(),
                        stop_signal.clone(),
                    )
                    .expect("Failed to run capture stream")
                    .map(move |mut image| {
                        info!("got image");
                        let image_buffer = image.to_image_buffer();
                        let snr_results = snr_threaded(&image_buffer, window_size).unwrap();
                        info!("done snr");
                        let image_metadata = ImageMetadataBuilder::new()
                            .capture_settings(capture_settings.clone())
                            .extra_info(CaptureResultData::SmartCaptureData(SmartCaptureData {
                                signal_noise_ratio: snr_results.0.clone(),
                                background_rect: snr_results.1.clone(),
                                foreground_rect: snr_results.2.clone(),
                            }))
                            .build();


                        let mut image_handler = ImageHandler::new(image_buffer, image_metadata);

                        image_handler.apply_histogram_equilization();

                        let mut best = best_capture.lock().unwrap();
                        if snr_results.0 > best.1 {
                            *best = (Some(image_handler.clone()), snr_results.0);
                        }

                        CaptureStreamItem::Image(image_handler)
                    });

                    progress_stream.chain(capture_stream)
            })
            .collect::<Vec<_>>();

            let mut stream = stream::iter(streams).flatten().boxed();

            let best_capture = best_capture.clone();
            let new_stream = stream! {
                while let Some(stream_item) = stream.next().await {
                    yield stream_item;
                }

                let final_best_image = {
                    let best_capture_guard = best_capture.lock().unwrap();
                    best_capture_guard.0.clone()
                };


                if let Some(image_handler) = final_best_image {
                    info!("{:?}", image_handler.image_metadata);
                    yield CaptureStreamItem::CaptureResult(vec![image_handler]);
                }
            };

            Box::pin(new_stream)
        }
}

impl AdvCapture for SignalAccumulationCapture {
    fn start_stream(
        &self,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        dark_maps_mutex: Arc<Mutex<HashMap<u32, SLImageRs>>>,
        defect_map_mutex: Arc<Mutex<Option<SLImageRs>>>,
        stop_signal: Arc<AtomicBool>,
    ) -> Pin<Box<dyn Stream<Item = CaptureStreamItem> + Send>> {
        const MAX_PIXEL_VALUE: u16 = 16383;
        info!("Starting Smart Capture");

        let last_image_mutex: Arc<Mutex<Option<ImageBuffer<Luma<u16>, Vec<u16>>>>> =
            Arc::new(Mutex::new(None));
        
        let capture_result: Arc<Mutex<Option<Vec<ImageHandler>>>> = Arc::new(Mutex::new(Some(Vec::new())));

        let streams = self
            .exp_times
            .iter()
            .map(|&exp_time| {
                let mut detector_controller = detector_controller_mutex.lock().unwrap();

                let capture_settings = CaptureSettingBuilder::new(
                    exp_time,
                    Box::new(SequenceCapture {
                        num_frames: self.frames_per_capture,
                    }),
                )
                .build();

                let last_image = Arc::clone(&last_image_mutex);
                let capture_result = capture_result.clone();
                let accumulated_exp_time = Arc::new(Mutex::new(exp_time));

                let progress_stream = stream::once(async move {
                    CaptureStreamItem::Progress(CaptureProgress::new(
                        0,
                        format!("Capturing images for exposure time {exp_time}ms").to_string(),
                    ))
                });

                let capture_stream = detector_controller
                    .run_capture_stream(
                        capture_settings.clone(),
                        dark_maps_mutex.clone(),
                        defect_map_mutex.clone(),
                        stop_signal.clone(),
                    )
                    .expect("Failed to run capture stream")
                    .map(move |mut image| {
                        let mut image_buffer = image.to_image_buffer();
                        let mut lock = last_image.lock().unwrap();
                        if let Some(ref mut prev) = *lock {
                            image_buffer.pixels_mut().zip(prev.pixels()).for_each(
                                |(current_pixel, prev_pixel)| {
                                    current_pixel[0] = current_pixel[0]
                                        .saturating_add(prev_pixel[0])
                                        .min(MAX_PIXEL_VALUE);
                                },
                            );
                        }

                        let mut image_handler = ImageHandler::new(
                            image_buffer,
                            ImageMetadataBuilder::new()
                                .capture_settings(capture_settings.clone())
                                .extra_info(CaptureResultData::SignalAccumulationData(
                                    SignalAccumulationData {
                                        accumulated_exp_time: *accumulated_exp_time.lock().unwrap(),
                                    },
                                ))
                                .build(),
                        );

                        image_handler.apply_histogram_equilization();

                        let mut lock = capture_result.lock().unwrap();
                        let mut_vec = lock.as_mut();
                        mut_vec.unwrap().push(image_handler.clone());

                        *accumulated_exp_time.lock().unwrap() += exp_time;

                        CaptureStreamItem::Image(image_handler)
                    });

                    progress_stream.chain(capture_stream)
            })
            .collect::<Vec<_>>();

            let mut stream = stream::iter(streams).flatten().boxed();

            let capture_result = capture_result.clone();
            let new_stream = stream! {
                while let Some(stream_item) = stream.next().await {
                    yield stream_item;
                }

                let capture_result_vec = {
                    capture_result.lock().unwrap().take().unwrap()
                };

                yield CaptureStreamItem::CaptureResult(capture_result_vec);
            };

            Box::pin(new_stream)
    }
}

impl AdvCapture for MultiCapture {
    fn start_stream(
        &self,
        detector_controller_mutex: Arc<Mutex<DetectorController>>,
        dark_maps_mutex: Arc<Mutex<HashMap<u32, SLImageRs>>>,
        defect_map_mutex: Arc<Mutex<Option<SLImageRs>>>,
        stop_signal: Arc<AtomicBool>,
    ) -> Pin<Box<dyn Stream<Item = CaptureStreamItem> + Send>> {
        info!("Starting Multi Capture");

        let capture_result = Arc::new(Mutex::new(Some(Vec::new())));

        let streams = self
            .exp_times
            .iter()
            .enumerate()
            .map(|(index, &exp_time)| {
                let mut detector_controller = detector_controller_mutex
                    .lock()
                    .expect("Failed to acquire detector controller lock");

                let capture_settings = CaptureSettingBuilder::new(
                    exp_time,
                    Box::new(SequenceCapture {
                        num_frames: self.frames_per_capture,
                    }),
                )
                .build();

                let stop_signal_clone = stop_signal.clone();
                let capture_result = capture_result.clone();

                let progress_stream = stream::once(async move {
                    CaptureStreamItem::Progress(CaptureProgress::new(
                        0,
                        format!("Capturing images for exposure time {exp_time}ms").to_string(),
                    ))
                });

                let capture_stream = detector_controller
                    .run_capture_stream(
                        capture_settings.clone(),
                        dark_maps_mutex.clone(),
                        defect_map_mutex.clone(),
                        stop_signal.clone(),
                    )
                    .expect("Failed to run capture stream")
                    .take_while(move |_| {
                        let stop_signal_clone_inner = stop_signal_clone.clone();
                        async move { !stop_signal_clone_inner.load(Ordering::Relaxed) }
                    })
                    .map(move |mut image| {
                        let mut image_handler = ImageHandler::new(
                            image.to_image_buffer(),
                            ImageMetadata {
                                capture_settings: Some(capture_settings.clone()),
                                date_created: None,
                                extra_info: None,
                            },
                        );

                        image_handler.apply_histogram_equilization();
                        
                        let mut lock = capture_result.lock().unwrap();
                        let mut_vec = lock.as_mut();
                        mut_vec.unwrap().push(image_handler.clone());

                        CaptureStreamItem::Image(image_handler)
                    })
                    .chain(stream::once(async move {
                        CaptureStreamItem::Progress(CaptureProgress::new(
                            0,
                            format!("Capturing images for exposure time {exp_time}ms").to_string(),
                        ))
                    }));

                    progress_stream.chain(capture_stream)
            })
            .collect::<Vec<_>>();

            let mut stream = stream::iter(streams).flatten().boxed();

            let capture_result = capture_result.clone();
            let new_stream = stream! {
                while let Some(stream_item) = stream.next().await {
                    yield stream_item;
                }

                let capture_result_vec = {
                    capture_result.lock().unwrap().take().unwrap()
                };

                yield CaptureStreamItem::CaptureResult(capture_result_vec);
            };

            Box::pin(new_stream)    }
}

#[cfg(test)]
mod tests {
    use std::sync::{atomic::AtomicBool, mpsc::channel, Arc, Mutex};

    use futures_util::{pin_mut, StreamExt};

    use crate::{
        capture::{
            advanced_capture::{SignalAccumulationCapture, SmartCapture},
            capture_manager::CaptureManager,
            test_utils::test_utils::{create_app, setup_controller_handle},
            types::AdvCapture,
        },
        wrapper::{FullWellModes, FullWellModesRS},
    };
    /*
    #[tokio::test]
    async fn smart_capture() {
        let app = create_app(tauri::test::mock_builder());
        let controller = Arc::new(Mutex::new(setup_controller_handle(app.handle().clone())));

        let smart_capture = SmartCapture {
            exp_times: vec![100, 200, 300],
            frames_per_capture: 10,
            median_filtered: false,
            window_size: 5,
        };

        let count = smart_capture.exp_times.len() as u32 * smart_capture.frames_per_capture;

        let stop_signal = Arc::new(AtomicBool::new(false));
        let stream =
            smart_capture.start_stream(app.handle().clone(), controller, stop_signal.clone());
        pin_mut!(stream);

        let mut counter = 0;
        while let Some(_) = stream.next().await {
            counter += 1;
        }

        assert_eq!(counter, count);
    }


    #[tokio::test]
    async fn signal_accumulation_captue() {
        let app = create_app(tauri::test::mock_builder());
        let controller = Arc::new(Mutex::new(CaptureManager::new()));

        let signal_accumulation_capture = SignalAccumulationCapture {
            exp_times: vec![100, 200, 300],
            frames_per_capture: 10,
        };

        let count = signal_accumulation_capture.exp_times.len() as u32
            * signal_accumulation_capture.frames_per_capture;

        let stop_signal = Arc::new(AtomicBool::new(false));

        let stream =
            signal_accumulation_capture.start_stream(app.handle().clone(), controller, stop_signal);
        pin_mut!(stream);

        let mut counter = 0;
        while let Some(_) = stream.next().await {
            counter += 1;
        }

        assert_eq!(counter, count);
    }
    */
}
