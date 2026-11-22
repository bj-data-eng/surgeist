#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod accounting;
pub mod border_image;
pub mod gradient_stops;
pub mod linear_gradient;
pub mod paint_order;
pub mod radial_gradient;
pub mod stacking;
pub mod zero_circle_gradient;
pub mod zero_height_gradient;

mod algorithm;
mod model;

pub use model::PaintError;
