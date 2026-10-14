//! Controlled HTML-owned operations, driven by a caller's admission policy.
//!
//! Actions describe their work before reading their operands. This module owns
//! value invariants; callers retain cancellation, work and allocation accounting.

mod attribute_rules;
mod local;
mod media;
mod microdata;
mod numeric;
mod rel;
mod time_element;
pub use attribute_rules::*;
pub use local::*;
pub use media::*;
pub use microdata::*;
pub use numeric::*;
pub use rel::*;
pub use time_element::*;

pub use crate::fragment_initialization::{
    HtmlFragmentNameTest, fragment_initial_modes, stack_reset_insertion_mode,
};
pub use crate::integration_points::{
    HtmlIntegrationPointComparison, HtmlIntegrationPointRule, HtmlMathmlTextPointTest,
    html_integration_point_encoding_attribute, html_integration_point_rule,
    mathml_text_integration_point,
};
pub use crate::values::bounded::{
    HtmlAttributeStorage, HtmlAttributeValidation, HtmlAttributeValidationAction,
    HtmlAttributeValidationError, HtmlAttributeValidationLayout, HtmlValidatedAttributes,
};

pub use crate::construction::bounded::*;
