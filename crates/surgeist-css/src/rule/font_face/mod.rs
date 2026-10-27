//! `@font-face` rules and descriptors.

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
