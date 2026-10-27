//! `@font-face` rules and descriptors.
//!
//! # Font faces
//!
//! [`CssFontFaceDescriptors::occurrences`](crate::rule::font_face::CssFontFaceDescriptors::occurrences) exposes valid descriptor occurrences
//! in authored order, while typed effective accessors return the last valid
//! occurrence. Invalid and unknown occurrences recover with
//! [`CssRecoveryAction::DropDescriptor`](crate::diagnostic::CssRecoveryAction::DropDescriptor) without erasing valid neighbors. Empty
//! [`CssFontFaceRule`] values remain in the authored tree; whether a face can be
//! used belongs to later contextual processing.
//!
//! Family, font, source-list, and font-face records cite `I-FONTS4-20260907`.
//! Family grammar, the font shorthand, and modern source hints are Complete;
//! source list and selected authored font-face rule are Complete, including
//! checked composition and canonical effective-value rule serialization.
//! Named-instance, language and metric descriptors retain typed ordinary values
//! or whole-descriptor environment values. Their payload serializers are bounded
//! and context independent. Values 4 `url()` and `src()` preserve authored
//! function identity and modifiers without loading resources.
//! Older immutable source identities retain their original editions.
//! Family names and lists provide bounded canonical specified serialization.
//! Font loading, matching, fallback, shaping, cascade, substitution, computed
//! values, and live CSSOM behavior belong to their downstream owners.

pub(crate) mod font_face_declaration_block;
pub(crate) mod font_face_values;
pub(crate) mod font_family_serialization;
pub(crate) mod font_rule_serialization;
pub(crate) mod font_source_serialization;
pub(crate) mod syntax;

#[doc(inline)]
pub use font_face_declaration_block::{
    CssFontFaceDeclarationBlockError, CssSpecifiedFontFaceDeclarationBlock,
};
#[doc(inline)]
pub use font_face_values::{
    CssFontMetricOverride, CssFontNamedInstance, CssFontNamedInstanceString,
};
#[doc(inline)]
pub use font_family_serialization::serialize_font_face_family_list;
#[doc(inline)]
pub use syntax::pending::CssFontFaceValueError;
#[doc(inline)]
pub use syntax::{
    CssAuthoredFontFaceDescriptorValue, CssFontDisplay, CssFontFaceDescriptor,
    CssFontFaceDescriptorKind, CssFontFaceDescriptorValue, CssFontFaceDescriptors,
    CssFontFaceFamily, CssFontFaceRule, CssFontFaceSource, CssFontFaceSourceList,
    CssFontFaceUrlSource, CssFontFormat, CssFontFormatHint, CssFontFormatString, CssFontLocalName,
    CssFontTechHint, CssPendingFontFaceDescriptorValue, CssUnicodeRange, CssUnicodeRangeList,
};
