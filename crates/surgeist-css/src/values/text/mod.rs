//! Text, decoration, spacing, wrapping, white-space and writing-mode values.

pub(crate) mod keyword_property_serialization;
pub(crate) mod line_height;
pub(crate) mod selector_syntax;
pub(crate) mod syntax;
pub(crate) mod text_alignment;
pub(crate) mod text_box;
pub(crate) mod text_controls;
pub(crate) mod text_decoration;
pub(crate) mod text_spacing;
pub(crate) mod text_value_serialization;
pub(crate) mod writing_modes;

#[doc(inline)]
pub use line_height::CssLineHeight;
#[doc(inline)]
pub use selector_syntax::CssDirectionality;
#[doc(inline)]
pub use syntax::{
    CssDirection, CssLineBreak, CssOverflowWrap, CssTabSize, CssTextAlign,
    CssTextCombineDigitCount, CssTextCombineUpright, CssTextDecoration, CssTextDecorationErrorLine,
    CssTextDecorationLine, CssTextDecorationLineComponent, CssTextDecorationStyle,
    CssTextDecorationThickness, CssTextIndent, CssTextOrientation, CssTextOverflow,
    CssTextTransform, CssTextTransformCase, CssTextTransformSet, CssTextWrap, CssTextWrapMode,
    CssTextWrapStyle, CssUnicodeBidi, CssVerticalAlign, CssWhiteSpace, CssWhiteSpaceCollapse,
    CssWhiteSpaceKeyword, CssWhiteSpaceTrim, CssWordBreak, CssWordSpaceTransform, CssWrapBoundary,
    CssWrapInside, CssWritingMode,
};
#[doc(inline)]
pub use text_alignment::{
    CssCharacterAlignment, CssTextAlignAllValue, CssTextAlignLastValue, CssTextAlignPosition,
    CssTextAlignValue, CssTextConstructionError,
};
#[doc(inline)]
pub use text_box::{
    CssTextBox, CssTextBoxEdge, CssTextBoxTrim, CssTextBoxValues, CssTextEdge, CssTextEdgeMetric,
    CssTextOverEdge, CssTextUnderEdge,
};
#[doc(inline)]
pub use text_controls::{
    CssAutospace, CssAutospaceMode, CssAutospaceValues, CssHangingPunctuation,
    CssHangingPunctuationEnd, CssHangingPunctuationValues, CssHyphenateCharacter,
    CssHyphenateLimitChars, CssHyphenateLimitCharsComponent, CssHyphenateLimitLast,
    CssHyphenateLimitLines, CssHyphenateString, CssHyphens, CssSpacingTrim, CssTextAutospace,
    CssTextGroupAlign, CssTextJustify, CssTextJustifyBase, CssTextSpacing, CssTextSpacingTrim,
    CssTextSpacingValues,
};
#[doc(inline)]
pub use text_decoration::{
    CssTextDecorationSkip, CssTextDecorationSkipBox, CssTextDecorationSkipInk,
    CssTextDecorationSkipInset, CssTextDecorationSkipSelf, CssTextDecorationSkipSpaces,
    CssTextEmphasis, CssTextEmphasisFill, CssTextEmphasisMark, CssTextEmphasisPosition,
    CssTextEmphasisShape, CssTextEmphasisSkip, CssTextEmphasisString, CssTextEmphasisStyle,
    CssTextEmphasisVertical, CssTextShadow, CssTextShadowLayer, CssTextShadowList, CssTextSide,
    CssTextUnderlineOffset, CssTextUnderlineOffsetLength, CssTextUnderlinePosition,
    CssUnderlinePosition, CssUnderlinePositionVertical,
};
#[doc(inline)]
pub use text_spacing::CssTextSpacingAdjustment;
