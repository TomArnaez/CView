use enum_dispatch::enum_dispatch;
use image::{ImageBuffer, Luma};
use serde::{Deserialize, Serialize};
use specta::Type;

use super::image::{ImageIterator, LineProfile};

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct Circle {
    pub pos: Point,
    pub radius: u32,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct Rect {
    pub width: u32,
    pub height: u32,
    pub pos: Point,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct Line {
    pub start: Point,
    pub finish: Point,
}

#[enum_dispatch(DataExtractor)]
#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub enum Annotation {
    Rect(Rect),
    Line(Line),
}

#[enum_dispatch]
pub trait DataExtractor {
    fn iter_values<'a>(&self, image: &'a ImageBuffer<Luma<u16>, Vec<u16>>) -> ImageIterator<'a>;
    fn get_std(&self, img: &ImageBuffer<Luma<u16>, Vec<u16>>) -> f64;
    fn get_profile(&self, img: &ImageBuffer<Luma<u16>, Vec<u16>>) -> LineProfile;
}
