use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

use crate::{image::LineProfile, events::Histogram};

#[derive(Serialize, Deserialize, Type)]
pub enum Chart {
    Histogram,
    LineProfile,
}

#[derive(Clone, Serialize, Type, Event)]
pub struct LineProfileEvent(pub LineProfile);

#[derive(Clone, Serialize, Type, Event)]
pub struct HistogramEvent(pub Histogram);