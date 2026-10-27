//! Authored value types, grouped by specification family.
//!
//! Each submodule holds the types of one specification family. The property values of
//! [`property`](crate::property) borrow them through `value()` accessors.
//!
//! # Dedicated authored function grammars
//!
//! Property accessors expose dedicated typed function families. Timing-function
//! wrappers expose the sole [`CssEasingList`](crate::values::animation::CssEasingList) through `value()`.
//! Transform wrappers expose the sole [`CssTransform`](crate::values::transform::CssTransform) through `value()`; filter and
//! backdrop-filter wrappers return [`CssFilter`](crate::values::effects::CssFilter), box-shadow returns [`CssBoxShadow`](crate::values::effects::CssBoxShadow), and
//! clip-path exposes the sole [`CssClipPath`](crate::values::effects::CssClipPath) through `value()`.

pub mod animation;
pub mod background;
pub mod border;
pub mod box_model;
pub mod color_adjust;
pub mod common;
pub mod content;
pub mod effects;
pub mod font;
pub mod image;
pub mod layout;
pub mod position;
pub mod shape;
pub mod speech;
pub mod table;
pub mod text;
pub mod transform;
pub mod ui;
pub mod view_transition;
