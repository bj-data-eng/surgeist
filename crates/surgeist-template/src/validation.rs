//! The validated template tree and validation diagnostics.
//!
//! [`validate_template`](crate::validate_template) checks an authored
//! [`TemplateDocument`](crate::ast::TemplateDocument) against the supplied
//! registries and returns a [`ValidatedTemplate`]. The validated tree keeps the
//! authored structure, expressions and spans, and distinguishes
//! [`ValidatedNode::NativeElement`] from [`ValidatedNode::ComponentElement`].
//! Conditionals and iteration stay symbolic: validation never selects a branch
//! or runs a loop.
//!
//! A failed validation returns a [`ValidationError`] whose
//! [`ValidationErrorKind`] names the unknown element, the duplicate or
//! undeclared attribute, or the unaccepted value form, with the source span of
//! the offending element or attribute.
//! [`render_to_rust`](crate::render_to_rust) and
//! [`append_static_html`](crate::dom::append_static_html) consume a
//! [`ValidatedTemplate`].

#[doc(inline)]
pub use crate::error::{ValidationError, ValidationErrorKind};
#[doc(inline)]
pub use crate::validate::{
    ValidatedAttrPart, ValidatedAttrValue, ValidatedAttribute, ValidatedElement, ValidatedForEach,
    ValidatedIf, ValidatedIfBranch, ValidatedInterpolation, ValidatedNode, ValidatedTemplate,
    ValidatedText,
};
