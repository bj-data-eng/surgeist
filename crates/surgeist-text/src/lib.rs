//! Owned text preparation and standalone layout for Surgeist.
//!
//! This module owns text layout facts: shaping, line breaking, glyph runs,
//! inline boxes, cursor geometry, selection geometry, and low-level movement.
//! It does not own rendering, widgets, document identity, style resolution, or
//! application commands.
//!
//! # Where things live
//!
//! The crate root holds only [`System`], [`Error`], [`ErrorCode`], [`ErrorDetail`] and [`Result`]. Every
//! other public item has exactly one path, through the module that holds it:
//!
//! | Module | Holds |
//! | --- | --- |
//! | `accessibility` | Optional AccessKit projection of an analyzed source or a placed layout (feature `text-accessibility`) |
//! | [`allocation`] | Layout-allocated atomic, control, boundary, strut and fit geometry |
//! | [`analyzed`] | Font-independent semantic analysis with exact original-source custody |
//! | [`cache`] | Layout cache identity and statistics |
//! | [`capture`] | Contextual source capture and its processed paragraph views |
//! | [`composer`] | Incremental construction of a source |
//! | [`conditional`] | Conditional searches and phrase pieces |
//! | [`finalized`] | Finalized lines, fragments, feedback, measurements, prefixes and readback views |
//! | [`font`] | Font resources, instances, metrics, baselines and outlines |
//! | [`geometry`] | Points, sizes, rectangles, glyph frames and numeric-category diagnostics |
//! | [`hyphenation`] | Native hyphenation policy and indication inputs with bounded en-US resources |
//! | [`inline_box`] | Inline-box edges, fragment families, pieces and supplied components |
//! | [`layout`] | The committed layout and every way to read it |
//! | [`leader`] | Isolated leader-pattern preparation and readback |
//! | [`observation`] | Measurement flow, semantic window and scope-incidence observations |
//! | [`options`] | Paragraph-level layout options |
//! | [`paragraph`] | Native paragraph input: origins, connections, controls, anchors and boundary policies |
//! | [`placement`] | Physical placement of finalized lines and atomics |
//! | [`prepared`] | Prepared text, its issued capabilities and borrowed views |
//! | [`processing`] | Source-local whitespace processing and its plaintext view |
//! | [`range_fragments`] | Nonempty original-source observations over placed, selected grapheme units |
//! | [`scope_alignment`] | Captured-scope alignment: limits, metrics, allocations and census |
//! | [`selection`] | Line selection requests and the selected line |
//! | [`source`] | Authored text: exact UTF-16 text, ranges, sources with spans and boxes, and edits |
//! | [`spacing`] | Selected spacing slots, allocations and spacing diagnostics |
//! | [`style`] | Resolved text style values and the text-style support matrix |
//! | [`supplied_line`] | Supplied resolved-level ordering and text-edge decisions for mixed inline lines |
//! | [`tabs`] | Captured CSS tab values and resource demand |
//! | [`upright`] | Upright and vertical typography diagnostics vocabulary |
//!
//! Items are not re-exported at the root:
//!
//! ```compile_fail
//! use surgeist_text::Style;
//! ```
//!
//! The same name resolves through its module:
//!
//! ```
//! use surgeist_text::style::Style;
//! let _ = Style::default();
//! ```

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
