//! Text, decoration, spacing, wrapping, white-space and writing-mode values.
//!
//! # Authored wrapping and whitespace
//!
//! Selected Text 4 wrapping and whitespace shorthands preserve authored fields
//! separately from their intrinsic two/three-member projections. Missing fields
//! reset to longhand initials during expansion. White-space's four special forms
//! keep their spelling, while trim flags form one contiguous constituent group.
//! Word breaking includes manual, auto-phrase and the deprecated CSS break-word.
//! These values do not execute text processing or resolve contextual wrapping.
//!
//! ```rust
//! use surgeist_css::property::CssKnownPropertyValueRef;
//! use surgeist_css::values::text::{CssTextWrap, CssTextWrapMode, CssTextWrapStyle, CssWhiteSpaceKeyword};
//! use surgeist_css::CssParser;
//!
//! let wrapping = CssTextWrap::try_new(
//!     Some(CssTextWrapMode::NoWrap), Some(CssTextWrapStyle::Balance),
//! ).unwrap();
//! assert_eq!(wrapping.serialize_specified(Default::default()).unwrap(), "nowrap balance");
//! assert!(CssTextWrap::try_new(None, None).is_none());
//! let report = CssParser::default().parse_block_contents("white-space: pre-wrap");
//! assert!(report.is_clean());
//! let CssKnownPropertyValueRef::WhiteSpace(value) = report.syntax()[0]
//!     .known().unwrap().property_value().unwrap()
//! else { panic!("white-space") };
//! assert_eq!(value.value().keyword(), Some(CssWhiteSpaceKeyword::PreWrap));
//! assert_eq!(value.value().serialize_specified(Default::default()).unwrap(), "pre-wrap");
//! ```
//!
//! # Authored text transformations, break preferences and tabs
//!
//! [`CssTextTransform`] distinguishes `none`, `math-auto` and a checked nonempty
//! [`CssTextTransformSet`]. Case, width and kana roles retain independent presence;
//! specified output emits them in that order. [`CssWrapInside`],
//! [`CssWrapBoundary`] and [`CssLineBreak`] retain finite break preferences, while
//! [`CssWordSpaceTransform`] retains `none` or a separator base and optional phrase flag.
//! [`CssTabSize`] keeps checked Number and Length domains distinct: bare zero is
//! a number, explicit `0px` a length, and typed calculation ranges remain deferred.
//!
//! These seven Text 4 longhands, [`CssTextIndent`] from Text 4 and
//! [`CssVerticalAlign`] from CSS2 expose intrinsic initials, inheritance metadata
//! and one typed contribution each. CSS-wide values stay symbolic; pending
//! substitution uses strict, retryable reentry preserving original importance and
//! replacement origins. Language processing, font metrics, indentation,
//! alignment and tab-stop execution belong to downstream owners.
//!
//! ```rust
//! use surgeist_css::property::CssKnownPropertyValueRef;
//! use surgeist_css::values::text::{CssTabSize, CssTextTransform, CssTextTransformCase, CssTextTransformSet};
//! use surgeist_css::CssParser;
//!
//! let set = CssTextTransformSet::try_new(
//!     Some(CssTextTransformCase::Uppercase), true, false,
//! ).unwrap();
//! assert_eq!(set.case(), Some(CssTextTransformCase::Uppercase));
//! assert!(set.full_width());
//! assert!(!set.full_size_kana());
//! assert!(CssTextTransformSet::try_new(None, false, false).is_none());
//! let transform = CssTextTransform::Transforms(set);
//! assert_eq!(transform.serialize_specified(Default::default()).unwrap(), "uppercase full-width");
//!
//! let report = CssParser::default().parse_block_contents("text-transform:full-width uppercase;tab-size:0px");
//! assert!(report.is_clean());
//! let CssKnownPropertyValueRef::TextTransform(value) = report.syntax()[0]
//!     .known().unwrap().property_value().unwrap()
//! else { panic!("text-transform") };
//! assert_eq!(value.value(), &transform);
//! let CssKnownPropertyValueRef::TabSize(tab) = report.syntax()[1]
//!     .known().unwrap().property_value().unwrap()
//! else { panic!("tab-size") };
//! assert!(matches!(tab.value(), CssTabSize::Length(_)));
//! assert_eq!(tab.value().serialize_specified(Default::default()).unwrap(), "0px");
//! ```
//!
//! # Authored hyphenation, justification and spacing
//!
//! Hyphenation preserves exact nonnegative integer tokens, one to three authored
//! character-count slots, and symbolic Integer-root calculations. Effective
//! omitted slots remain separate from authored serialization and computed rounding.
//! Hyphenate-character retains its original string component and decoded content;
//! zone and line-padding reuse signed numeric owners. Text-justify preserves
//! authored distribute and no-compress presence, with computed equivalence downstream.
//!
//! Text-spacing expands exactly spacing-trim then autospace. Its complete
//! constituents remain contiguous, and omitted members receive intrinsic normal
//! initials without extra authored output tokens. Autospace preserves omitted
//! insertion mode; hanging-punctuation keeps an exclusive force/allow end role.
//! These properties share strict checked admission, pending replacement, intrinsic
//! metadata and cumulative specified output. Language, glyph and layout execution
//! belongs to downstream owners. See the
//! [reference](https://github.com/bj-data-eng/surgeist/blob/main/crates/surgeist-css/docs/reference.md#authored-hyphenation-justification-and-spacing).
//!
//! ```rust
//! use surgeist_css::property::{CssContributions, CssExpansion, CssKnownProperty};
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_block_contents("hyphenate-limit-chars:auto 2;text-justify:no-compress distribute;\
//!      text-spacing:replace punctuation trim-both");
//! assert!(report.is_clean());
//! assert_eq!(report.syntax()[0].serialize_specified(Default::default()).unwrap(),
//!     "hyphenate-limit-chars: auto 2;");
//! assert_eq!(report.syntax()[1].serialize_specified(Default::default()).unwrap(),
//!     "text-justify: distribute no-compress;");
//! assert_eq!(report.syntax()[2].serialize_specified(Default::default()).unwrap(),
//!     "text-spacing: trim-both punctuation replace;");
//! let CssExpansion::Contributions(CssContributions::Longhands(values)) =
//!     report.syntax()[2].expand().unwrap()
//! else { panic!("two spacing terminals") };
//! assert_eq!(values.items().iter().map(|v| v.property()).collect::<Vec<_>>(),
//!     [CssKnownProperty::TextSpacingTrim, CssKnownProperty::TextAutospace]);
//! ```
//!
//! Checked constituent construction preserves authored omissions:
//!
//! ```rust
//! use surgeist_css::values::text::{CssAutospace, CssAutospaceMode, CssAutospaceValues, CssSpacingTrim, CssTextJustify, CssTextJustifyBase, CssTextSpacing, CssTextSpacingValues};
//!
//! assert!(CssTextJustify::try_new(None, false).is_none());
//! let justify = CssTextJustify::try_new(Some(CssTextJustifyBase::Distribute), true)
//!     .unwrap();
//! assert_eq!(justify.serialize_specified(Default::default()).unwrap(), "distribute no-compress");
//! let flags = CssAutospaceValues::try_new(true, false, true, None).unwrap();
//! assert_eq!(flags.mode(), None);
//! assert_eq!(flags.effective_mode(), CssAutospaceMode::Insert);
//! let fields = CssTextSpacingValues::try_new(
//!     Some(CssSpacingTrim::TrimBoth), Some(CssAutospace::Spacing(flags)),
//! ).unwrap();
//! assert_eq!(CssTextSpacing::Components(fields).serialize_specified(Default::default()).unwrap(),
//!     "trim-both ideograph-alpha punctuation");
//! ```
//!
//! # Authored Text Decoration
//!
//! The selected effective Decoration 4 grammar covers decoration, underline,
//! skipping, emphasis and text-shadow. All roles participate in intrinsic
//! metadata, shorthand expansion, strict construction and pending reentry.
//! The decoration shorthand sets Line, Thickness, Style and Color only; skip
//! sets Self, Box, Inset, Spaces and Ink; emphasis sets Style and Color only.
//! CSS-wide values and pending values stay symbolic, with original components.
//! [`CssTextEmphasisMark`] preserves fill/shape omissions and origins;
//! [`CssTextEmphasisPosition`] preserves an omitted Right side. Canonical output
//! omits assumed Filled before a shape and explicit Right after a vertical side.
//! Strings remain untruncated and font-dependent shape selection stays downstream.
//! [`CssTextShadowLayer`] checks nonnegative text spread and original closure,
//! reusing the box-shadow provider while preserving its signed-spread policy.
//!
//! ```
//! use surgeist_css::values::text::{CssTextEmphasisFill, CssTextEmphasisMark, CssTextEmphasisShape, CssTextEmphasisPosition, CssTextEmphasisVertical, CssTextSide};
//! let mark = CssTextEmphasisMark::try_new(Some(CssTextEmphasisFill::Filled),
//!     Some(CssTextEmphasisShape::Dot)).unwrap();
//! assert_eq!(mark.serialize_specified(Default::default()).unwrap(), "dot");
//! assert_eq!(mark.fill(), Some(CssTextEmphasisFill::Filled));
//! let position = CssTextEmphasisPosition::new(CssTextEmphasisVertical::Over, None);
//! assert_eq!(position.side(), None);
//! assert_eq!(position.effective_side(), CssTextSide::Right);
//! assert_eq!(position.serialize_specified(Default::default()).unwrap(), "over");
//! ```

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
