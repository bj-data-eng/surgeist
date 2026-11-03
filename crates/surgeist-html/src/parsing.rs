//! Fragment initialization and integration-point decisions that the parser consumes.

pub use crate::internal::parsing::fragment_initialization::{
    HtmlFragmentInsertionMode, HtmlFragmentTokenizerState, HtmlParserScriptingMode,
    fragment_insertion_mode, fragment_tokenizer_state,
};
pub use crate::internal::parsing::integration_points::{
    is_html_integration_point, is_mathml_text_integration_point,
};
