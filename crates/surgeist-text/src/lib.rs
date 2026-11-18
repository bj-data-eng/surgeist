//! Owned text preparation and standalone layout for Surgeist.
//!
//! This module owns text layout facts: shaping, line breaking, glyph runs,
//! inline boxes, cursor geometry, selection geometry, and low-level movement.
//! It does not own rendering, widgets, document identity, style resolution, or
//! application commands.

#[cfg(feature = "text-accessibility")]
pub mod accessibility;
pub mod allocation;
pub mod analyzed;
pub mod cache;
pub mod capture;
pub mod composer;
pub mod conditional;
pub mod finalized;
pub mod font;
pub mod geometry;
pub mod hyphenation;
pub mod inline_box;
pub mod layout;
pub mod leader;
pub mod observation;
pub mod options;
pub mod paragraph;
pub mod placement;
pub mod prepared;
pub mod processing;
pub mod range_fragments;
pub mod scope_alignment;
pub mod selection;
pub mod source;
pub mod spacing;
pub mod style;
pub mod supplied_line;
pub mod tabs;
pub mod upright;

mod control;
mod edit;
mod engine;
mod error;
mod id;
mod inline_box_components;
mod phrase;
mod projection;
mod protocol;
mod range;
mod style_support;
mod system;
mod text;
mod transform;
mod views;
mod wrapping;

pub use error::{Error, ErrorCode, ErrorDetail, Result};
pub use system::System;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_fonts;
#[cfg(test)]
mod test_wrapping;

#[cfg(test)]
#[path = "test_fonts/shaping.rs"]
mod shaping_test_fonts;

#[cfg(test)]
#[path = "test_fonts/baseline.rs"]
mod baseline_test_fonts;

#[cfg(test)]
#[path = "test_fonts/vertical.rs"]
mod vertical_test_fonts;

#[cfg(test)]
#[path = "test_fonts/mixed.rs"]
mod mixed_test_fonts;
