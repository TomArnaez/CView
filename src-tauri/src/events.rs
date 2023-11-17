use std::{collections::HashMap, sync::Arc};

use crate::{
    appdata::AppData,
    capture::{types::CaptureManagerInfo},
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

#[derive(Debug, Clone, Serialize, Type, Event)]
pub struct AppDataEvent(pub AppData);

#[derive(Debug, Clone, Serialize, Type, Event)]
pub struct CaptureManagerEvent(pub CaptureManagerInfo);

#[serde_as]
#[derive(Serialize, Deserialize, Type, Clone)]
pub struct Histogram {
    #[serde_as(as = "Vec<(_,_)>")]
    pub data: HashMap<u16, u32>,
}

#[derive(Clone, Serialize, Type, Event)]
pub struct HistogramEvent(pub Histogram);

/*
#[derive(Clone, Serialize, Type, Event)]
pub struct HistogramEvent(pub Histogram);
*/

#[derive(Serialize, Type, Event)]
pub struct ImageStateEvent(Arc<ImageService>);
