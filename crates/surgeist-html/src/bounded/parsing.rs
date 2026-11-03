//! Fragment-parsing mode decisions and integration-point rules evaluated over caller-supplied comparisons.

pub use crate::internal::parsing::fragment_initialization::{
    HtmlFragmentNameTest, fragment_initial_modes, stack_reset_insertion_mode,
};
pub use crate::internal::parsing::integration_points::{
    HtmlIntegrationPointComparison, HtmlIntegrationPointRule, HtmlMathmlTextPointTest,
    html_integration_point_encoding_attribute, html_integration_point_rule,
    mathml_text_integration_point,
};
