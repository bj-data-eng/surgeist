//! Immutable construction values shared by HTML and its tree consumers.
//!
//! Document strings, names, attributes and borrowed semantic operands preserve exact
//! UTF-16. This module also holds newline normalization, checked custom element names
//! and the source-qualified interpretation of raw values.

pub use crate::internal::values::custom_names::{
    HtmlCustomElementName, HtmlCustomElementNameError,
};
pub use crate::internal::values::interpretation::{
    HtmlValueAdmissionError, HtmlValueAuthorState, HtmlValueFailure, HtmlValueInterpreter,
    HtmlValueRecoveryState, HtmlValueReport, HtmlValueRepresentationError, HtmlValueSyntax,
    HtmlValueSyntaxError,
};
pub use crate::internal::values::newlines::HtmlNewlineNormalizer;
pub use crate::internal::values::semantic_input::{
    HtmlAttributeInput, HtmlAttributesInput, HtmlNameInput,
};
pub use crate::internal::values::{
    Attribute, DocumentString, DuplicateAttributeError, Namespace, OrderedAttributes,
    QualifiedName, Utf8ConversionError,
};
