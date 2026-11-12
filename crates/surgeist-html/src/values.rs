//! Immutable construction values shared by HTML and its tree consumers.
//!
//! Document strings, names, attributes and borrowed semantic operands preserve exact
//! UTF-16. This module also holds newline normalization, checked custom element names
//! and the source-qualified interpretation of raw values.
//!
//! The native constructors allocate without accounting: [`DocumentString::from_utf16`], its
//! `From<&str>` and `From<String>` conversions, `Default` for [`DocumentString`] (an empty private
//! block), and [`OrderedAttributes::new`], which allocates the attribute vector and a duplicate
//! table sized to its input before any limit applies. A caller that must bound allocation or
//! work admits the input before it builds the value, or builds attributes through the
//! accounted forms in [`crate::bounded::values`], whose actions describe their work and
//! allocation before they run.

pub use crate::internal::values::custom_names::{
    HtmlCustomElementName, HtmlCustomElementNameError,
};
pub use crate::internal::values::interpretation::{
    HtmlValueAdmissionError, HtmlValueAuthorState, HtmlValueFailure, HtmlValueInterpretationError,
    HtmlValueInterpreter, HtmlValueRecoveryState, HtmlValueReport, HtmlValueRepresentationError,
    HtmlValueSyntax, HtmlValueSyntaxError,
};
pub use crate::internal::values::newlines::HtmlNewlineNormalizer;
pub use crate::internal::values::semantic_input::{
    HtmlAttributeInput, HtmlAttributesInput, HtmlNameInput,
};
pub use crate::internal::values::{
    Attribute, DocumentString, DuplicateAttributeError, Namespace, OrderedAttributes,
    QualifiedName, TextBlock, TextBlockTable, TextBlockTableError, TextRangeError,
    Utf8ConversionError, WeakTextBlock,
};
