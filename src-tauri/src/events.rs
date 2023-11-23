use std::{collections::HashMap, sync::Arc};

use crate::{
    capture::types::CaptureManagerInfo,
    image::image::{ImageService, LineProfile},
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Clone, Serialize, Type, Event)]
pub struct StreamCaptureEvent();

#[derive(Debug, Clone, Serialize, Type, Event)]
pub struct CancelCaptureEvent();

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct Histogram(Vec<u32>);

#[derive(Clone, Serialize, Type, Event)]
pub struct HistogramEvent(pub Histogram);

/*
#[derive(Clone, Serialize, Type, Event)]
pub struct HistogramEvent(pub Histogram);
*/

#[derive(Serialize, Type, Event)]
pub struct ImageStateEvent(Arc<ImageService>);
