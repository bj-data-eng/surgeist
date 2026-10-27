//! Intrinsic declaration expansion, before cascade or contextual resolution.
//!
//! The property schema owns intrinsic expansion for every recognized property:
//! longhand types, initial values, shorthand members and reset-only members.
//! Custom declarations retain their symbolic specified values.

use crate::color::{CssColor, CssSystemColor};
use crate::numeric::{
    CssInteger, CssLength, CssLengthPercentage, CssNonNegative, CssNumber, CssNumeric,
    CssNumericLiteral, CssPositive,
};
use crate::query::container::{CssContainer, CssContainerNames, CssContainerType};
use crate::serialization::specified_rule_serialization::SpecifiedProvider as _;
use crate::values::animation::{
    CssAnimationDirection, CssAnimationDirectionList, CssAnimationFillMode,
    CssAnimationFillModeList, CssAnimationIterationCount, CssAnimationIterationCountList,
    CssAnimationList, CssAnimationName, CssAnimationNameList, CssAnimationPlayState,
    CssAnimationPlayStateList, CssDelayList, CssDurationList, CssEasing, CssEasingKeyword,
    CssEasingList, CssTransitionList, CssTransitionProperty, CssTransitionPropertyList,
};
use crate::values::background::{
    CssBackground, CssBackgroundAttachment, CssBackgroundAttachmentList, CssBackgroundBox,
    CssBackgroundBoxList, CssBackgroundLayer, CssBackgroundPosition, CssBackgroundPositionList,
    CssBackgroundRepeat, CssBackgroundRepeatList, CssBackgroundRepeatStyle, CssBackgroundSize,
    CssBackgroundSizeList, CssBlendMode, CssBlendModeList,
};
use crate::values::border::border_radius::CornerRadiusLength;
use crate::values::border::{
    CssBorder, CssBorderImage, CssBorderImageOutset, CssBorderImageOutsetComponent,
    CssBorderImageRepeat, CssBorderImageRepeatKeyword, CssBorderImageSlice,
    CssBorderImageSliceComponent, CssBorderImageWidth, CssBorderImageWidthComponent,
    CssBorderRadiusShorthand, CssLineStyle, CssLineWidth, CssOutline, CssOutlineColor,
    CssOutlineStyle,
};
use crate::values::box_model::{
    CssAspectRatioValue, CssBoxDecorationBreak, CssBoxSizing, CssBreakBetween, CssBreakInside,
    CssClear, CssContain, CssContainIntrinsicSizeFallback, CssContainIntrinsicSizeValue,
    CssContentVisibility, CssFloat, CssFlowTolerance, CssFrameSizing, CssInsetValue,
    CssLayoutPosition, CssMarginValue, CssMaxSizeValue, CssMinIntrinsicSizing, CssOverflow,
    CssOverflowClipMargin, CssOverflowValue, CssPaddingValue, CssScrollBehavior,
    CssScrollbarGutter, CssSizeValue, CssVisibility, CssZIndexValue,
};
use crate::values::color_adjust::{CssColorScheme, CssForcedColorAdjust, CssPrintColorAdjust};
use crate::values::common::{CssAutoOr, CssBoxEdgeKeyword, CssFourSides, CssSidePair};
use crate::values::content::{CssListStylePosition, CssListStyleValue, CssQuotes};
use crate::values::effects::{
    CssBoxShadow, CssClip, CssClipRule, CssColorInterpolationFilters, CssFilter, CssIsolation,
    CssMaskBorder, CssMaskBoxList, CssMaskClip, CssMaskClipList, CssMaskComposite,
    CssMaskCompositeList, CssMaskList, CssMaskMode, CssMaskModeList, CssMaskType, CssOpacityValue,
};
use crate::values::font::{
    CssAbsoluteFontWeight, CssAuthoredFontFeatureSettings, CssFontFamilyList, CssFontKerning,
    CssFontLanguageOverride, CssFontOpticalSizing, CssFontPalette, CssFontSize, CssFontSizeAdjust,
    CssFontStyle, CssFontStyleKeyword, CssFontSynthesis, CssFontSynthesisPosition,
    CssFontSynthesisSmallCaps, CssFontSynthesisStyle, CssFontSynthesisWeight, CssFontValue,
    CssFontVariant, CssFontVariantAlternates, CssFontVariantCaps, CssFontVariantEastAsian,
    CssFontVariantEmoji, CssFontVariantLigatures, CssFontVariantNumeric, CssFontVariantPosition,
    CssFontVariantValue, CssFontVariationSettings, CssFontWeight, CssFontWidth,
    CssFontWidthKeyword,
};
use crate::values::image::{
    CssImageOrientation, CssImageRendering, CssImageValue, CssImageValueList, CssObjectFit,
};
use crate::values::layout::scroll_snap::{ScrollMarginSide, ScrollPaddingSide};
use crate::values::layout::{
    CssAlignContentValue, CssAlignItemsValue, CssAlignSelfValue, CssAlignmentValue, CssColumnCount,
    CssColumnFill, CssColumnRule, CssColumnSpan, CssColumns, CssFlexBasisValue, CssFlexDirection,
    CssFlexFlow, CssFlexValue, CssFlexWrap, CssGapValue, CssGrid, CssGridArea, CssGridAutoFlow,
    CssGridLine, CssGridLineRange, CssGridTemplate, CssGridTrackBreadth, CssGridTrackList,
    CssGridTrackSize, CssGridTrackSizeList, CssItemFlow, CssJustifyContentValue,
    CssJustifyItemsValue, CssJustifySelfValue, CssPlaceContentValue, CssPlaceItemsValue,
    CssPlaceSelfValue, CssScrollSnapAlignment, CssScrollSnapStop, CssScrollSnapType,
};
use crate::values::position::{
    CssHorizontalPosition, CssPosition, CssPositionList, CssVerticalPosition,
};
use crate::values::speech::{
    CssCue, CssSpeak, CssSpeakAs, CssSpeechBreak, CssVoiceBalance, CssVoiceBalanceKeyword,
    CssVoiceDuration, CssVoiceFamily, CssVoiceLevel, CssVoicePitchRange, CssVoiceRate,
    CssVoiceRateKeyword, CssVoiceStress, CssVoiceVolume, CssVoiceVolumeLevel,
};
use crate::values::table::{CssBorderCollapse, CssCaptionSide, CssEmptyCells, CssTableLayout};
use crate::values::text::{
    CssDirection, CssHangingPunctuation, CssHyphenateCharacter, CssHyphenateLimitChars,
    CssHyphenateLimitLast, CssHyphenateLimitLines, CssHyphens, CssLineBreak, CssLineHeight,
    CssOverflowWrap, CssSpacingTrim, CssTabSize, CssTextAlignAllValue, CssTextAlignLastValue,
    CssTextAlignValue, CssTextAutospace, CssTextBox, CssTextBoxEdge, CssTextBoxTrim,
    CssTextCombineUpright, CssTextDecoration, CssTextDecorationLine, CssTextDecorationSkip,
    CssTextDecorationSkipBox, CssTextDecorationSkipInk, CssTextDecorationSkipInset,
    CssTextDecorationSkipSelf, CssTextDecorationSkipSpaces, CssTextDecorationStyle,
    CssTextDecorationThickness, CssTextEmphasis, CssTextEmphasisPosition, CssTextEmphasisSkip,
    CssTextEmphasisStyle, CssTextEmphasisVertical, CssTextGroupAlign, CssTextIndent,
    CssTextJustify, CssTextOrientation, CssTextOverflow, CssTextShadow, CssTextSide,
    CssTextSpacing, CssTextSpacingTrim, CssTextTransform, CssTextUnderlineOffset,
    CssTextUnderlinePosition, CssTextWrap, CssTextWrapMode, CssTextWrapStyle, CssUnicodeBidi,
    CssVerticalAlign, CssWhiteSpace, CssWhiteSpaceCollapse, CssWhiteSpaceTrim, CssWordBreak,
    CssWordSpaceTransform, CssWrapBoundary, CssWrapInside, CssWritingMode,
};
use crate::values::transform::{
    CssBackfaceVisibility, CssCoordBox, CssOffset, CssOffsetAnchor, CssOffsetPath,
    CssOffsetPosition, CssOffsetRotate, CssPerspective, CssRotate, CssScale, CssTransform,
    CssTransformBox, CssTransformOrigin, CssTransformStyle, CssTranslate,
};
use crate::values::ui::{
    CssAccentColor, CssAppearance, CssCaret, CssCaretAnimation, CssCaretColor, CssCaretShape,
    CssCursor, CssCursorKeyword, CssInteractivity, CssInterestDelayValue, CssNavigation,
    CssPointerEvents, CssResize, CssScrollbarColor, CssScrollbarWidth, CssUserSelect,
};
use std::fmt;
use std::sync::Arc;

use crate::declaration::{
    CssCustomDeclaration, CssDeclaration, CssDeclarationBody, CssGlobalKeyword,
};
use crate::numeric::CssTimeUnit;
use crate::parser::variables::contains_substitution;
use crate::property::CssPropertyNameRef;
use crate::property::{CssKnownDeclaration, CssKnownDeclaredValueRef, CssKnownPropertyValueRef};
use crate::values::animation::{CssAnimation, CssTransition};
use crate::values::background::CssBackgroundLayerBoxes;
use crate::values::effects::{CssMaskLayer, CssMaskLayerBoxes};
use crate::values::font::CssSystemFont;
use crate::values::layout::{
    CssGridAutoFlowAxis, CssGridGeneralTrackComponent, CssGridGeneralTrackList, CssGridLineNames,
    CssGridTrackRepeatComponent,
};
use crate::values::position::CssPhysicalPosition;
use crate::values::text::{CssAutospace, CssWhiteSpaceKeyword};

// The 50% 50% initial of object-position and perspective-origin.
pub(crate) fn initial_position() -> CssPosition {
    CssPosition::from(initial_transform_position())
}

// Transforms 1/2 select the same intrinsic 50% 50% planar initial.
pub(crate) fn initial_transform_position() -> CssPhysicalPosition {
    let center = CssNumeric::<CssLengthPercentage>::try_from_component(
        crate::component::CssComponentValue::try_token("50%").expect("percentage token"),
    )
    .expect("percentage position offset");
    CssPhysicalPosition::try_new(
        CssHorizontalPosition::Offset(center.clone()),
        CssVerticalPosition::Offset(center),
    )
    .expect("two physical free offsets")
}

// Text4's special spellings project source-defined constituents. Components
// retain omissions so the existing terminal owner supplies intrinsic initials.
pub(crate) fn white_space_collapse(value: &CssWhiteSpace) -> Option<CssWhiteSpaceCollapse> {
    match value.keyword() {
        Some(CssWhiteSpaceKeyword::Normal) => Some(CssWhiteSpaceCollapse::Collapse),
        Some(CssWhiteSpaceKeyword::Pre | CssWhiteSpaceKeyword::PreWrap) => {
            Some(CssWhiteSpaceCollapse::Preserve)
        }
        Some(CssWhiteSpaceKeyword::PreLine) => Some(CssWhiteSpaceCollapse::PreserveBreaks),
        None => value.collapse(),
    }
}
pub(crate) fn white_space_mode(value: &CssWhiteSpace) -> Option<CssTextWrapMode> {
    match value.keyword() {
        Some(CssWhiteSpaceKeyword::Pre) => Some(CssTextWrapMode::NoWrap),
        Some(_) => Some(CssTextWrapMode::Wrap),
        None => value.mode(),
    }
}
pub(crate) fn white_space_trim(value: &CssWhiteSpace) -> Option<CssWhiteSpaceTrim> {
    if value.keyword().is_some() {
        Some(CssWhiteSpaceTrim::none())
    } else {
        value.trim()
    }
}

// Inline3 §6.1's noninitial omitted trim is explicitly projected, rather than
// requesting the longhand's `none` initial. PR12765 corrects its printed name.
pub(crate) fn text_box_trim(value: &CssTextBox) -> Option<CssTextBoxTrim> {
    Some(match value {
        CssTextBox::Normal => CssTextBoxTrim::None,
        CssTextBox::Components(value) => value.trim().unwrap_or(CssTextBoxTrim::TrimBoth),
    })
}
pub(crate) fn text_box_edge(value: &CssTextBox) -> Option<CssTextBoxEdge> {
    match value {
        CssTextBox::Normal => Some(CssTextBoxEdge::Auto),
        CssTextBox::Components(value) => value.edge(),
    }
}

// Text4 spacing has exactly two settable terminals; whole-member omissions
// request the terminal's shared intrinsic normal initial.
pub(crate) fn text_spacing_trim(value: &CssTextSpacing) -> Option<CssTextSpacingTrim> {
    match value {
        CssTextSpacing::None => Some(CssTextSpacingTrim::Trim(CssSpacingTrim::SpaceAll)),
        CssTextSpacing::Auto => Some(CssTextSpacingTrim::Auto),
        CssTextSpacing::Components(value) => value.trim().map(CssTextSpacingTrim::Trim),
    }
}
pub(crate) fn text_spacing_autospace(value: &CssTextSpacing) -> Option<CssTextAutospace> {
    match value {
        CssTextSpacing::None => Some(CssTextAutospace::Autospace(CssAutospace::NoAutospace)),
        CssTextSpacing::Auto => Some(CssTextAutospace::Auto),
        CssTextSpacing::Components(value) => value.autospace().map(CssTextAutospace::Autospace),
    }
}

// Grid alternatives project typed children into the existing schema. Omitted
// values request that terminal's central initial; no CSS is serialized/reparsed.
pub(crate) fn grid_template_rows(value: &CssGridTemplate) -> Option<CssGridTrackList> {
    if let Some(rows) = value.rows() {
        return Some(rows.clone());
    }
    let rows = value.area_rows()?;
    let mut components = Vec::new();
    let mut previous_after: Option<&CssGridLineNames> = None;
    for row in rows {
        if previous_after.is_some() || row.before().is_some() {
            let names = previous_after
                .into_iter()
                .chain(row.before())
                .flat_map(|group| group.names().iter().cloned())
                .collect();
            components.push(CssGridGeneralTrackComponent::LineNames(
                CssGridLineNames::new(names),
            ));
        }
        components.push(CssGridGeneralTrackComponent::TrackSize(
            row.size()
                .cloned()
                .unwrap_or_else(|| CssGridTrackSize::from_breadth(CssGridTrackBreadth::auto())),
        ));
        previous_after = row.after();
    }
    if let Some(names) = previous_after {
        components.push(CssGridGeneralTrackComponent::LineNames(names.clone()));
    }
    Some(CssGridTrackList::general(
        CssGridGeneralTrackList::try_new(components)
            .expect("validated area rows supply nonadjacent boundaries and track sizes"),
    ))
}

pub(crate) fn grid_template_columns(value: &CssGridTemplate) -> Option<CssGridTrackList> {
    if let Some(columns) = value.columns() {
        return Some(columns.clone());
    }
    value.area_columns().map(|columns| {
        let components = columns
            .components()
            .iter()
            .map(|component| match component {
                CssGridTrackRepeatComponent::LineNames(names) => {
                    CssGridGeneralTrackComponent::LineNames(names.clone())
                }
                CssGridTrackRepeatComponent::TrackSize(size) => {
                    CssGridGeneralTrackComponent::TrackSize(size.clone())
                }
            })
            .collect();
        CssGridTrackList::general(
            CssGridGeneralTrackList::try_new(components)
                .expect("checked repeat-free area columns are a general track list"),
        )
    })
}

pub(crate) fn grid_template_areas(
    value: &CssGridTemplate,
) -> Option<crate::values::layout::CssGridTemplateAreas> {
    value.area_rows().map(|rows| {
        crate::values::layout::CssGridTemplateAreas::try_rows(
            rows.iter().map(|row| row.area().clone()).collect(),
        )
        .expect("immutable template area matrix was checked at construction")
    })
}

pub(crate) fn grid_rows(value: &CssGrid) -> Option<CssGridTrackList> {
    if let Some(template) = value.template_value() {
        return grid_template_rows(template);
    }
    (value.auto_flow()?.axis() == CssGridAutoFlowAxis::Column).then(|| {
        value
            .explicit_tracks()
            .expect("auto-flow branch has an explicit axis")
            .clone()
    })
}

pub(crate) fn grid_columns(value: &CssGrid) -> Option<CssGridTrackList> {
    if let Some(template) = value.template_value() {
        return grid_template_columns(template);
    }
    (value.auto_flow()?.axis() == CssGridAutoFlowAxis::Row).then(|| {
        value
            .explicit_tracks()
            .expect("auto-flow branch has an explicit axis")
            .clone()
    })
}

pub(crate) fn grid_areas(value: &CssGrid) -> Option<crate::values::layout::CssGridTemplateAreas> {
    grid_template_areas(value.template_value()?)
}

pub(crate) fn grid_auto_rows(value: &CssGrid) -> Option<CssGridTrackSizeList> {
    if value.auto_flow()?.axis() == CssGridAutoFlowAxis::Row {
        value.auto_tracks().cloned()
    } else {
        None
    }
}

pub(crate) fn grid_auto_columns(value: &CssGrid) -> Option<CssGridTrackSizeList> {
    if value.auto_flow()?.axis() == CssGridAutoFlowAxis::Column {
        value.auto_tracks().cloned()
    } else {
        None
    }
}

pub(crate) fn grid_auto_flow(value: &CssGrid) -> Option<CssGridAutoFlow> {
    value.auto_flow().map(CssGridAutoFlow::ExplicitAxis)
}
use crate::component::CssComponentValues;
use crate::property::{CssKnownProperty, CssPropertyValueError};

/// Why intrinsic expansion could not produce completed contributions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CssExpansionErrorKind {
    /// Strict reentry still contains a decoded `var()` function.
    ResidualSubstitution,
    /// Replacement components failed serialization or the original property grammar.
    /// Boxed so that every error carrying an expansion failure stays small.
    InvalidReplacement(Box<CssPropertyValueError>),
}

/// A typed expansion capability or strict-reentry failure.
///
/// A rejected replacement is exposed by [`Self::replacement_error`] and is the error's
/// source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssExpansionError {
    kind: CssExpansionErrorKind,
}

impl CssExpansionError {
    /// Returns whether strict reentry still contains a decoded `var()` function.
    #[must_use]
    pub const fn is_residual_substitution(&self) -> bool {
        matches!(self.kind, CssExpansionErrorKind::ResidualSubstitution)
    }

    /// Returns why the replacement components failed serialization or the original
    /// property grammar.
    #[must_use]
    pub fn replacement_error(&self) -> Option<&CssPropertyValueError> {
        match &self.kind {
            CssExpansionErrorKind::InvalidReplacement(error) => Some(error),
            CssExpansionErrorKind::ResidualSubstitution => None,
        }
    }

    pub(crate) const fn new(kind: CssExpansionErrorKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for CssExpansionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            CssExpansionErrorKind::ResidualSubstitution => {
                formatter.write_str("replacement still contains var() or env() substitution")
            }
            CssExpansionErrorKind::InvalidReplacement(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for CssExpansionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            CssExpansionErrorKind::InvalidReplacement(error) => Some(&**error),
            CssExpansionErrorKind::ResidualSubstitution => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MappingLogic {
    Logical,
    Neutral,
    Physical,
}

macro_rules! logical_annotation {
    () => {
        (None, MappingLogic::Neutral)
    };
    ($group:ident, $mapping:ident) => {
        (Some(stringify!($group)), MappingLogic::$mapping)
    };
}

macro_rules! intrinsic_initial {
    ($variant:ident, value, $initial:expr) => {
        CssLonghandInitialValue {
            value: InitialValue::Value(CssLonghandValue {
                value: Box::new(OwnedLonghandValue::$variant($initial)),
            }),
        }
    };
    ($variant:ident, user_agent, $initial:expr) => {
        CssLonghandInitialValue {
            value: InitialValue::UserAgent($initial),
        }
    };
}

// First filter the full property inventory to annotated rows. The bounded
// collector then emits complete enums and matches; macros never expand to
// partial enum variants or match arms, and no second property registry exists.
// New inverse-only canonical spelling stays with each typed value owner;
// existing authored occurrence serialization continues through its original method.
macro_rules! append_inverse_value {
    (PlaceContent, $value:ident, $writer:ident) => {
        $value.append_cssom_inverse_to_rule_writer($writer)
    };
    (PlaceItems, $value:ident, $writer:ident) => {
        $value.append_cssom_inverse_to_rule_writer($writer)
    };
    (PlaceSelf, $value:ident, $writer:ident) => {
        $value.append_cssom_inverse_to_rule_writer($writer)
    };
    (Columns, $value:ident, $writer:ident) => {
        $value.append_cssom_inverse_to_rule_writer($writer)
    };
    (ItemFlow, $value:ident, $writer:ident) => {
        $value.append_cssom_inverse_to_rule_writer($writer)
    };
    (FlexFlow, $value:ident, $writer:ident) => {
        $value.append_cssom_inverse_to_rule_writer($writer)
    };
    (Flex, $value:ident, $writer:ident) => {
        $value.append_cssom_inverse_to_rule_writer($writer)
    };
    ($variant:ident, $value:ident, $writer:ident) => {
        $value.append_to_rule_writer($writer)
    };
}

macro_rules! define_expansion_schema {
    ($input:ident, $numeric:ident;
        All, $all_canonical:literal, [$($all_alias:literal),*], $all_stable_id:literal,
        $all_value:ty, $all_parser:ident, $all_dispatch:block, expansion = universal { $($all_metadata:tt)* };
        $(
        $variant:ident, $canonical:literal, [$($alias:literal),*], $stable_id:literal,
        $value:ty, $parser:ident, $dispatch:block
        $(, expansion = $kind:ident { $($metadata:tt)* })?;
    )*) => {
        define_expansion_schema!(@collect [] [] [] []; All, universal { $($all_metadata)* };
            $($( $variant, $value, $kind { $($metadata)* }; )?)*
        );
    };
    (@collect [$($longhands:tt)*] [$($shorthands:tt)*] [$($universal:tt)*] [$($four:tt)*];
        $variant:ident, $row_value:ty, longhand {
            value: $value:ty,
            inherited: $inherited:literal, initial_kind: $initial_kind:ident, initial: $initial:expr $(, logical_group: $group:ident, mapping: $mapping:ident)?
        }; $($rest:tt)*
    ) => {
        define_expansion_schema!(@collect
            [$($longhands)* ($variant, $value, $inherited, $initial_kind, $initial $(, $group, $mapping)?)]
            [$($shorthands)*] [$($universal)*] [$($four)*]; $($rest)*
        );
    };
    (@collect [$($longhands:tt)*] [$($shorthands:tt)*] [$($universal:tt)*] [$($four:tt)*];
        $variant:ident, $value:ty, shorthand {
            members: [$($member:ident => $projection:expr),+],
            reset_only: [$($reset:ident),*]
        }; $($rest:tt)*
    ) => {
        define_expansion_schema!(@collect
            [$($longhands)*]
            [$($shorthands)* ($variant, $value, [$($member => $projection),+], [$($reset),*])]
            [$($universal)*] [$($four)*]; $($rest)*
        );
    };
    (@collect [$($longhands:tt)*] [$($shorthands:tt)*] [$($universal:tt)*] [$($four:tt)*];
        $variant:ident, universal { exclude_custom: $exclude_custom:literal,
            excluded: [$($excluded:ident),+]
        }; $($rest:tt)*
    ) => {
        define_expansion_schema!(@collect
            [$($longhands)*] [$($shorthands)*]
            [$($universal)* ($variant, $exclude_custom, [$($excluded),+])]
            [$($four)*]; $($rest)*
        );
    };
    (@collect [$($longhands:tt)*] [$($shorthands:tt)*] [$($universal:tt)*] [$($four:tt)*];
        $variant:ident, $value:ty, four_side {
            mode: $mode:expr,
            physical: [$($physical:ident => $physical_projection:expr),+],
            logical: [$($logical:ident => $logical_projection:expr),+]
        }; $($rest:tt)*
    ) => {
        define_expansion_schema!(@collect
            [$($longhands)*] [$($shorthands)*] [$($universal)*]
            [$($four)* ($variant, $value, $mode, [$($physical => $physical_projection),+], [$($logical => $logical_projection),+])]; $($rest)*
        );
    };
    (@collect
        [$(($longhand:ident, $value:ty, $inherited:literal, $initial_kind:ident, $initial:expr $(, $group:ident, $mapping:ident)?))*]
        [$(($shorthand:ident, $shorthand_value:ty,
            [$($member:ident => $projection:expr),+], [$($reset:ident),*]))*]
        [($universal:ident, $exclude_custom:literal, [$($excluded:ident),+])]
        [$(($four_property:ident, $four_value:ty, $four_mode:expr,
            [$($physical:ident => $physical_projection:expr),+],
            [$($logical:ident => $logical_projection:expr),+]))*];
    ) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        enum Longhand {
            $($longhand,)*
        }

        #[derive(Clone, Debug, PartialEq)]
        enum OwnedLonghandValue {
            $($longhand($value),)*
        }

        /// A borrowed exact ordinary longhand value coupled to its property.
        ///
        /// The variants are generated only for schema-selected longhands.
        /// Symbolic values stay unresolved.
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum CssLonghandValueRef<'a> {
            $(#[doc = concat!("The exact ordinary value for `", stringify!($longhand), "`.")]
            $longhand(&'a $value),)*
        }

        /// A privately constructed exact shorthand value used by CSSOM inversion.
        #[derive(Clone, Debug)]
        pub(crate) enum SpecifiedInverseValue {
            $($shorthand($shorthand_value),)*
            $($four_property($four_value),)*
        }

        impl SpecifiedInverseValue {
            pub(crate) fn append(&self, writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter)
                -> Result<(), crate::serialization::CssSerializationError> {
                match self {
                    $(Self::$shorthand(value) => append_inverse_value!($shorthand, value, writer),)*
                    $(Self::$four_property(value) => value.append_to_rule_writer(writer),)*
                }
            }

            pub(crate) fn projected(
                &self,
                count: usize,
            ) -> Result<Vec<OwnedContributionValue>, crate::serialization::CssSerializationError> {
                let mut result = Vec::new();
                result.try_reserve(count).map_err(|_| {
                    crate::serialization::CssSerializationError::new(
                        crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                    )
                })?;
                if let Self::Font(CssFontValue::System(system)) = self {
                    let CssPropertyKindRef::Shorthand(meta) = CssKnownProperty::Font.metadata().expect("font schema").kind() else {
                        unreachable!("font shorthand metadata")
                    };
                    result.extend(meta.members().iter().enumerate().map(|(index, property)| {
                        if index < meta.settable_members().len() {
                            OwnedContributionValue::SystemFont(*property, *system)
                        } else { property.0.initial() }
                    }));
                    return Ok(result);
                }
                match self {
                    $(Self::$shorthand(value) => result.extend([
                        $(match ($projection)(value) {
                            Some(projected) => OwnedContributionValue::Ordinary(CssLonghandValue {
                                value: Box::new(OwnedLonghandValue::$member(projected)),
                            }),
                            None => Longhand::$member.initial(),
                        },)+
                        $(Longhand::$reset.initial(),)*
                    ]),)*
                    $(Self::$four_property(value) => match ($four_mode)(value) {
                        crate::values::common::CssBoxSideKind::Physical => result.extend([
                            $(OwnedContributionValue::Ordinary(CssLonghandValue {
                                value: Box::new(OwnedLonghandValue::$physical(($physical_projection)(value))),
                            }),)+
                        ]),
                        crate::values::common::CssBoxSideKind::Logical => result.extend([
                            $(OwnedContributionValue::Ordinary(CssLonghandValue {
                                value: Box::new(OwnedLonghandValue::$logical(($logical_projection)(value))),
                            }),)+
                        ]),
                    },)*
                }
                Ok(result)
            }
        }

        impl Longhand {
            fn initial(self) -> OwnedContributionValue {
                OwnedContributionValue::from_initial(self.initial_value())
            }

            fn initial_value(self) -> CssLonghandInitialValue {
                match self {
                    $(Self::$longhand => intrinsic_initial!($longhand, $initial_kind, $initial),)*
                }
            }

            const fn property(self) -> CssKnownProperty {
                match self { $(Self::$longhand => CssKnownProperty::$longhand,)* }
            }

            const fn inherited(self) -> bool {
                match self { $(Self::$longhand => $inherited,)* }
            }

            fn mapping(self) -> (Option<&'static str>, MappingLogic) {
                match self {
                    $(Self::$longhand => logical_annotation!($($group, $mapping)?),)*
                }
            }

            fn global(self, keyword: CssGlobalKeyword) -> OwnedContributionValue {
                OwnedContributionValue::Global(CssLonghandProperty(self), keyword)
            }
        }

        impl OwnedLonghandValue {
            const fn property(&self) -> CssLonghandProperty {
                match self {
                    $(Self::$longhand(_) => CssLonghandProperty(Longhand::$longhand),)*
                }
            }

            const fn view(&self) -> CssLonghandValueRef<'_> {
                match self { $(Self::$longhand(value) => CssLonghandValueRef::$longhand(value),)* }
            }
        }

        pub(crate) const SPECIFIED_TERMINALS: &[CssLonghandProperty] = &[
            $(CssLonghandProperty(Longhand::$longhand),)*
        ];

        pub(crate) fn grammar_metadata(
            grammar: crate::property::CssPropertyGrammar,
        ) -> Result<&'static CssPropertyMetadata, CssPropertyMetadataError> {
            match grammar.resolved() {
                crate::property::properties::CssResolvedPropertyName::LegacyShorthand(
                    crate::property::properties::CssLegacyPropertyAlias::GlyphOrientationVertical,
                ) => {
                    const METADATA: CssPropertyMetadata = CssPropertyMetadata {
                        grammar: CssKnownProperty::TextOrientation.legacy_shorthands()[0],
                        kind: CssPropertyKindRef::Shorthand(&CssShorthandMetadata {
                            members: &[CssLonghandProperty(Longhand::TextOrientation)],
                            settable: &[CssLonghandProperty(Longhand::TextOrientation)],
                            reset: &[],
                            legacy: true,
                        }),
                    };
                    return Ok(&METADATA);
                }
                crate::property::properties::CssResolvedPropertyName::LegacyShorthand(
                    crate::property::properties::CssLegacyPropertyAlias::PageBreakBefore,
                ) => {
                    const METADATA: CssPropertyMetadata = CssPropertyMetadata {
                        grammar: CssKnownProperty::BreakBefore.legacy_shorthands()[0],
                        kind: CssPropertyKindRef::Shorthand(&CssShorthandMetadata {
                            members: &[CssLonghandProperty(Longhand::BreakBefore)],
                            settable: &[CssLonghandProperty(Longhand::BreakBefore)],
                            reset: &[], legacy: true,
                        }),
                    };
                    return Ok(&METADATA);
                }
                crate::property::properties::CssResolvedPropertyName::LegacyShorthand(
                    crate::property::properties::CssLegacyPropertyAlias::PageBreakAfter,
                ) => {
                    const METADATA: CssPropertyMetadata = CssPropertyMetadata {
                        grammar: CssKnownProperty::BreakAfter.legacy_shorthands()[0],
                        kind: CssPropertyKindRef::Shorthand(&CssShorthandMetadata {
                            members: &[CssLonghandProperty(Longhand::BreakAfter)],
                            settable: &[CssLonghandProperty(Longhand::BreakAfter)],
                            reset: &[], legacy: true,
                        }),
                    };
                    return Ok(&METADATA);
                }
                crate::property::properties::CssResolvedPropertyName::LegacyShorthand(
                    crate::property::properties::CssLegacyPropertyAlias::PageBreakInside,
                ) => {
                    const METADATA: CssPropertyMetadata = CssPropertyMetadata {
                        grammar: CssKnownProperty::BreakInside.legacy_shorthands()[0],
                        kind: CssPropertyKindRef::Shorthand(&CssShorthandMetadata {
                            members: &[CssLonghandProperty(Longhand::BreakInside)],
                            settable: &[CssLonghandProperty(Longhand::BreakInside)],
                            reset: &[], legacy: true,
                        }),
                    };
                    return Ok(&METADATA);
                }
                crate::property::properties::CssResolvedPropertyName::Canonical(_) => {}
            }
            match grammar.target_property() {
                $(CssKnownProperty::$longhand => {
                    const METADATA: CssPropertyMetadata = CssPropertyMetadata {
                        grammar: CssKnownProperty::$longhand.grammar(),
                        kind: CssPropertyKindRef::Longhand(&CssLonghandMetadata {
                            property: CssLonghandProperty(Longhand::$longhand),
                        }),
                    };
                    Ok(&METADATA)
                },)*
                $(CssKnownProperty::$shorthand => {
                    const METADATA: CssPropertyMetadata = CssPropertyMetadata {
                        grammar: CssKnownProperty::$shorthand.grammar(),
                        kind: CssPropertyKindRef::Shorthand(&CssShorthandMetadata {
                            members: &[
                                $(CssLonghandProperty(Longhand::$member),)+
                                $(CssLonghandProperty(Longhand::$reset),)*
                            ],
                            settable: &[$(CssLonghandProperty(Longhand::$member),)+],
                            reset: &[$(CssLonghandProperty(Longhand::$reset),)*],
                            legacy: false,
                        }),
                    };
                    Ok(&METADATA)
                },)*
                $(CssKnownProperty::$four_property => {
                    const METADATA: CssPropertyMetadata = CssPropertyMetadata {
                        grammar: CssKnownProperty::$four_property.grammar(),
                        kind: CssPropertyKindRef::FourSideShorthand(&CssFourSideShorthandMetadata {
                            physical: &[$(CssLonghandProperty(Longhand::$physical),)+],
                            logical: &[$(CssLonghandProperty(Longhand::$logical),)+],
                        }),
                    };
                    Ok(&METADATA)
                },)*
                CssKnownProperty::$universal => {
                    const METADATA: CssPropertyMetadata = CssPropertyMetadata {
                        grammar: CssKnownProperty::$universal.grammar(),
                        kind: CssPropertyKindRef::UniversalReset(&CssUniversalResetMetadata { _private: () }),
                    };
                    Ok(&METADATA)
                },
            }
        }

        fn expansion_shape(known: &CssKnownDeclaration) -> Result<ExpansionShape, CssExpansionError> {
            let property = known.property();
            match property {
                CssKnownProperty::$universal => Ok(ExpansionShape::UniversalReset),
                $(CssKnownProperty::$longhand => {
                    Ok(ExpansionShape::Longhands(&[Longhand::$longhand]))
                })*
                $(CssKnownProperty::$shorthand => {
                    Ok(ExpansionShape::Longhands(&[$(Longhand::$member,)+ $(Longhand::$reset,)*]))
                })*
                $(CssKnownProperty::$four_property => {
                    let mode = match known.property_value() {
                        Some(CssKnownPropertyValueRef::$four_property(value)) => ($four_mode)(value.value()),
                        _ => crate::values::common::CssBoxSideKind::Physical,
                    };
                    Ok(ExpansionShape::Longhands(match mode {
                        crate::values::common::CssBoxSideKind::Physical => &[$(Longhand::$physical,)+],
                        crate::values::common::CssBoxSideKind::Logical => &[$(Longhand::$logical,)+],
                    }))
                })*
            }
        }

        fn universal_excludes(property: CssPropertyNameRef<'_>) -> bool {
            match property {
                CssPropertyNameRef::Custom(_) => $exclude_custom,
                CssPropertyNameRef::SvgGlyphOrientationVertical => false,
                CssPropertyNameRef::Known(property) => matches!(property, $(CssKnownProperty::$excluded)|+),
            }
        }

        fn ordinary_values(
            value: CssKnownPropertyValueRef<'_>,
        ) -> Result<Vec<OwnedContributionValue>, CssExpansionError> {
            match value {
                $(CssKnownPropertyValueRef::$longhand(value) => Ok(vec![
                    OwnedContributionValue::Ordinary(CssLonghandValue { value: Box::new(OwnedLonghandValue::$longhand(value.value().to_owned())) })
                ]),)*
                $(CssKnownPropertyValueRef::$shorthand(value) => {
                    let value = value.value();
                    Ok(vec![
                        $(match ($projection)(value) {
                            Some(projected) => OwnedContributionValue::Ordinary(CssLonghandValue { value: Box::new(OwnedLonghandValue::$member(projected)) }),
                            None => Longhand::$member.initial(),
                        },)+
                        $(Longhand::$reset.initial(),)*
                    ])
                })*
                $(CssKnownPropertyValueRef::$four_property(value) => {
                    let value = value.value();
                    Ok(match ($four_mode)(value) {
                        crate::values::common::CssBoxSideKind::Physical => vec![
                            $(OwnedContributionValue::Ordinary(CssLonghandValue { value: Box::new(OwnedLonghandValue::$physical(($physical_projection)(value))) }),)+
                        ],
                        crate::values::common::CssBoxSideKind::Logical => vec![
                            $(OwnedContributionValue::Ordinary(CssLonghandValue { value: Box::new(OwnedLonghandValue::$logical(($logical_projection)(value))) }),)+
                        ],
                    })
                })*
            }
        }
    };
}

crate::property::properties::property_schema!(
    define_expansion_schema,
    expansion_input,
    numeric_input
);

pub(crate) fn initial_zero_time<R: crate::numeric::CssRange<crate::numeric::CssTime>>()
-> crate::numeric::CssNumeric<crate::numeric::CssTime, R> {
    crate::numeric::CssNumeric::try_from_literal(
        crate::numeric::CssNumericLiteral::<crate::numeric::CssTime>::try_new(
            "0",
            CssTimeUnit::Seconds,
        )
        .expect("ordinary zero seconds"),
    )
    .expect("every time range admits zero")
}

// Each authored item contributes one entry to each list, with a scalar schema
// initial for each omitted slot. Parser-admitted time children may retain implicit
// closure; projecting them must clone their graph rather than recheck construction.
macro_rules! timing_list_projection {
    ($function:ident, $source:ty, $property:ident, $list:ty, $items:ident, $project:expr, $construct:expr) => {
        pub(crate) fn $function(authored: &$source) -> Option<$list> {
            let initial = Longhand::$property.initial_value();
            let InitialValue::Value(initial) = initial.value else {
                unreachable!("timing list has an ordinary schema initial")
            };
            let OwnedLonghandValue::$property(initial) = *initial.value else {
                unreachable!("schema initial belongs to its longhand")
            };
            let initial = &initial.$items()[0];
            let values = authored
                .values()
                .iter()
                .map(|item| ($project)(item).unwrap_or_else(|| initial.to_owned()))
                .collect();
            Some(($construct)(values))
        }
    };
}

timing_list_projection!(
    transition_properties,
    CssTransitionList,
    TransitionProperty,
    CssTransitionPropertyList,
    properties,
    |transition: &CssTransition| transition.property().cloned(),
    |values| CssTransitionPropertyList::try_new(values)
        .expect("admitted transitions are nonempty and none is singleton")
);
timing_list_projection!(
    transition_durations,
    CssTransitionList,
    TransitionDuration,
    CssDurationList,
    values,
    |transition: &CssTransition| transition.duration().cloned(),
    CssDurationList::from_parser
);
timing_list_projection!(
    transition_timing_functions,
    CssTransitionList,
    TransitionTimingFunction,
    CssEasingList,
    values,
    |transition: &CssTransition| transition.timing_function().cloned(),
    |values| CssEasingList::try_new(values).expect("admitted transitions are nonempty")
);
timing_list_projection!(
    transition_delays,
    CssTransitionList,
    TransitionDelay,
    CssDelayList,
    values,
    |transition: &CssTransition| transition.delay().cloned(),
    CssDelayList::from_parser
);

timing_list_projection!(
    animation_durations,
    CssAnimationList,
    AnimationDuration,
    CssDurationList,
    values,
    |animation: &CssAnimation| animation.duration().cloned(),
    CssDurationList::from_parser
);
timing_list_projection!(
    animation_timing_functions,
    CssAnimationList,
    AnimationTimingFunction,
    CssEasingList,
    values,
    |animation: &CssAnimation| animation.timing_function().cloned(),
    |values| CssEasingList::try_new(values).expect("admitted animations are nonempty")
);
timing_list_projection!(
    animation_delays,
    CssAnimationList,
    AnimationDelay,
    CssDelayList,
    values,
    |animation: &CssAnimation| animation.delay().cloned(),
    CssDelayList::from_parser
);
timing_list_projection!(
    animation_iteration_counts,
    CssAnimationList,
    AnimationIterationCount,
    CssAnimationIterationCountList,
    values,
    |animation: &CssAnimation| animation.iteration_count().cloned(),
    |values| CssAnimationIterationCountList::try_new(values)
        .expect("admitted animations are nonempty")
);
timing_list_projection!(
    animation_directions,
    CssAnimationList,
    AnimationDirection,
    CssAnimationDirectionList,
    directions,
    |animation: &CssAnimation| animation.direction(),
    |values| CssAnimationDirectionList::try_new(values).expect("admitted animations are nonempty")
);
timing_list_projection!(
    animation_fill_modes,
    CssAnimationList,
    AnimationFillMode,
    CssAnimationFillModeList,
    modes,
    |animation: &CssAnimation| animation.fill_mode(),
    |values| CssAnimationFillModeList::try_new(values).expect("admitted animations are nonempty")
);
timing_list_projection!(
    animation_play_states,
    CssAnimationList,
    AnimationPlayState,
    CssAnimationPlayStateList,
    states,
    |animation: &CssAnimation| animation.play_state(),
    |values| CssAnimationPlayStateList::try_new(values).expect("admitted animations are nonempty")
);
timing_list_projection!(
    animation_names,
    CssAnimationList,
    AnimationName,
    CssAnimationNameList,
    names,
    |animation: &CssAnimation| animation.name().cloned(),
    |values| CssAnimationNameList::try_new(values).expect("admitted animations are nonempty")
);

pub(crate) fn initial_mask_positions() -> CssPositionList {
    let zero =
        crate::numeric::CssNumeric::<crate::numeric::CssLengthPercentage>::try_from_component(
            crate::component::CssComponentValue::try_token("0%").expect("percentage token"),
        )
        .expect("percentage length");
    CssPositionList::try_new(vec![
        CssPhysicalPosition::try_new(
            CssHorizontalPosition::Offset(zero.clone()),
            CssVerticalPosition::Offset(zero),
        )
        .expect("physical position")
        .into(),
    ])
    .expect("one initial position")
}
pub(crate) fn initial_mask_border_slice() -> CssBorderImageSlice {
    CssBorderImageSlice::try_new(
        vec![
            CssBorderImageSliceComponent::Number(
                crate::numeric::CssNumeric::<
                    crate::numeric::CssNumber,
                    crate::numeric::CssNonNegative,
                >::try_from_component(
                    crate::component::CssComponentValue::try_number("0").expect("number token"),
                )
                .expect("nonnegative zero"),
            ),
        ],
        false,
    )
    .expect("one initial slice")
}

// Sparse authored layers need one scalar default per slot, not a whole-list fallback.
// Each projection obtains its typed initial once from the central property schema.
macro_rules! background_list_projection {
    ($function:ident, $source:ty, $property:ident, $list:ty, $items:ident, $project:expr) => {
        pub(crate) fn $function(background: &$source) -> Option<$list> {
            let initial = Longhand::$property.initial_value();
            let InitialValue::Value(initial) = initial.value else {
                unreachable!("background list has an ordinary schema initial")
            };
            let OwnedLonghandValue::$property(initial) = *initial.value else {
                unreachable!("schema initial belongs to its longhand")
            };
            let initial = &initial.$items()[0];
            let values = background
                .layers()
                .iter()
                .map(|layer| ($project)(layer).unwrap_or_else(|| initial.clone()))
                .collect();
            Some(<$list>::try_new(values).expect("background has nonempty checked layers"))
        }
    };
}

background_list_projection!(
    background_images,
    CssBackground,
    BackgroundImage,
    CssImageValueList,
    images,
    |layer: &CssBackgroundLayer| layer.image().cloned()
);
background_list_projection!(
    background_positions,
    CssBackground,
    BackgroundPosition,
    CssBackgroundPositionList,
    positions,
    |layer: &CssBackgroundLayer| layer.position().cloned()
);
background_list_projection!(
    background_sizes,
    CssBackground,
    BackgroundSize,
    CssBackgroundSizeList,
    sizes,
    |layer: &CssBackgroundLayer| layer.size().cloned()
);
background_list_projection!(
    background_repeats,
    CssBackground,
    BackgroundRepeat,
    CssBackgroundRepeatList,
    repeats,
    |layer: &CssBackgroundLayer| layer.repeat()
);
background_list_projection!(
    background_attachments,
    CssBackground,
    BackgroundAttachment,
    CssBackgroundAttachmentList,
    attachments,
    |layer: &CssBackgroundLayer| layer.attachment()
);
background_list_projection!(
    background_origins,
    CssBackground,
    BackgroundOrigin,
    CssBackgroundBoxList,
    boxes,
    |layer: &CssBackgroundLayer| layer.boxes().map(CssBackgroundLayerBoxes::origin)
);
background_list_projection!(
    background_clips,
    CssBackground,
    BackgroundClip,
    CssBackgroundBoxList,
    boxes,
    |layer: &CssBackgroundLayer| layer.boxes().map(CssBackgroundLayerBoxes::clip)
);

background_list_projection!(
    mask_images,
    CssMaskList,
    MaskImage,
    CssImageValueList,
    images,
    |layer: &CssMaskLayer| layer.image().cloned()
);
background_list_projection!(
    mask_positions,
    CssMaskList,
    MaskPosition,
    CssPositionList,
    positions,
    |layer: &CssMaskLayer| layer.position().cloned()
);
background_list_projection!(
    mask_sizes,
    CssMaskList,
    MaskSize,
    CssBackgroundSizeList,
    sizes,
    |layer: &CssMaskLayer| layer.size().cloned()
);
background_list_projection!(
    mask_repeats,
    CssMaskList,
    MaskRepeat,
    CssBackgroundRepeatList,
    repeats,
    |layer: &CssMaskLayer| layer.repeat()
);
background_list_projection!(
    mask_origins,
    CssMaskList,
    MaskOrigin,
    CssMaskBoxList,
    boxes,
    |layer: &CssMaskLayer| layer.boxes().and_then(CssMaskLayerBoxes::origin)
);
background_list_projection!(
    mask_clips,
    CssMaskList,
    MaskClip,
    CssMaskClipList,
    clips,
    |layer: &CssMaskLayer| layer.boxes().map(CssMaskLayerBoxes::clip)
);
background_list_projection!(
    mask_composites,
    CssMaskList,
    MaskComposite,
    CssMaskCompositeList,
    operators,
    |layer: &CssMaskLayer| layer.composite()
);
background_list_projection!(
    mask_modes,
    CssMaskList,
    MaskMode,
    CssMaskModeList,
    modes,
    |layer: &CssMaskLayer| layer.mode()
);

#[derive(Clone, Copy, Debug)]
enum ExpansionShape {
    Longhands(&'static [Longhand]),
    UniversalReset,
}

/// A borrowed longhand contribution's ordinary value or whole-value CSS-wide keyword.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CssContributionValueRef<'a> {
    /// A completed ordinary longhand value.
    Ordinary(CssLonghandValueRef<'a>),
    /// A whole-value CSS-wide keyword.
    Global(CssGlobalKeyword),
    /// An intrinsic initial requiring a user-agent environment.
    UserAgentInitial(CssUserAgentInitial),
    /// A settable `font` member requiring the selected system-font environment.
    SystemFont(CssSystemFont),
}

#[derive(Debug)]
struct ContributionContext {
    source: CssDeclaration,
    replacement: Option<CssComponentValues>,
}
impl crate::source::CssRetainedSources for ContributionContext {
    fn visit_retained_sources(
        &self,
        sources: &mut crate::source::CssSourceRetention,
    ) -> Result<(), crate::source::CssSourceRetentionError> {
        sources.visit(&self.source)?;
        sources.visit(&self.replacement)
    }
}

macro_rules! retained_contribution_context {
    ($($ty:ty),+ $(,)?) => {$(
        impl crate::source::CssRetainedSources for $ty {
            fn visit_retained_sources(
                &self,
                sources: &mut crate::source::CssSourceRetention,
            ) -> Result<(), crate::source::CssSourceRetentionError> {
                // Checked values derive from this original/replacement input;
                // the immutable context retains both, including unused tokens.
                sources.visit(self.context.as_ref())
            }
        }
    )+};
}
retained_contribution_context!(
    CssLonghandContribution,
    CssUniversalReset,
    CssSvgGlyphOrientationVerticalContribution,
);

/// One property-coupled contribution retaining its authored source occurrence.
///
/// Fields and construction are private: the ordinary payload always belongs to
/// [`Self::property`]. Contributions share one source/replacement context, so
/// expansion does not duplicate a complete component tree for each longhand.
#[derive(Clone, Debug)]
pub struct CssLonghandContribution {
    value: Arc<OwnedContributionValue>,
    context: Arc<ContributionContext>,
}

impl CssLonghandContribution {
    /// Returns the property identity derived from the active value variant.
    #[must_use]
    pub fn property(&self) -> CssKnownProperty {
        self.value.property()
    }

    /// Borrows the exact ordinary value or symbolic global keyword.
    #[must_use]
    pub fn value(&self) -> CssContributionValueRef<'_> {
        self.value.view()
    }

    /// Borrows the coupled ordinary value, excluding CSS-wide and UA initial states.
    #[must_use]
    pub fn ordinary_value(&self) -> Option<&CssLonghandValue> {
        match self.value.as_ref() {
            OwnedContributionValue::Ordinary(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the original declaration, including importance and occurrence identity.
    #[must_use]
    pub fn source(&self) -> &CssDeclaration {
        &self.context.source
    }

    /// Returns the caller's replacement input after strict reentry, if any.
    #[must_use]
    pub fn replacement_components(&self) -> Option<&CssComponentValues> {
        self.context.replacement.as_ref()
    }
}

/// Completed selected longhand contributions from one declaration occurrence.
#[derive(Clone, Debug)]
pub struct CssLonghandContributions {
    items: Vec<CssLonghandContribution>,
}

impl CssLonghandContributions {
    /// Returns the unique contributed longhands in property-schema member order.
    #[must_use]
    pub fn items(&self) -> &[CssLonghandContribution] {
        &self.items
    }
}

/// The symbolic effect of `all`, before selecting applicable cascade targets.
#[derive(Clone, Debug)]
pub struct CssUniversalReset {
    keyword: CssGlobalKeyword,
    context: Arc<ContributionContext>,
}

impl CssUniversalReset {
    /// Returns the authored or replacement CSS-wide keyword without resolving it.
    #[must_use]
    pub const fn keyword(&self) -> CssGlobalKeyword {
        self.keyword
    }

    /// Returns the original `all` declaration occurrence and importance.
    #[must_use]
    pub fn source(&self) -> &CssDeclaration {
        &self.context.source
    }

    /// Returns the caller's replacement components after strict reentry, if any.
    #[must_use]
    pub fn replacement_components(&self) -> Option<&CssComponentValues> {
        self.context.replacement.as_ref()
    }

    /// Reports only the explicit `all` exclusions: custom properties, `direction`
    /// and `unicode-bidi`. A false result is not full target applicability;
    /// selection, inheritance and cascade remain downstream responsibilities.
    #[must_use]
    pub fn excludes(&self, property: CssPropertyNameRef<'_>) -> bool {
        universal_excludes(property)
    }
}

/// One symbolic custom-property contribution before cascade and substitution.
///
/// Private construction guarantees a custom declaration. The original occurrence
/// supplies its case-sensitive name, token value or CSS-wide keyword, importance,
/// and provenance without copying a second representation of those semantics.
#[derive(Clone, Debug)]
pub struct CssCustomPropertyContribution {
    source: CssDeclaration,
}

impl CssCustomPropertyContribution {
    /// Returns the unchanged authored declaration and its occurrence identity.
    #[must_use]
    pub const fn source(&self) -> &CssDeclaration {
        &self.source
    }

    /// Borrows the validated custom name and symbolic specified value.
    #[must_use]
    pub fn declaration(&self) -> &CssCustomDeclaration {
        self.source
            .custom()
            .expect("custom contributions are constructed only from custom declarations")
    }
}

/// One completed independent SVG terminal with its original occurrence and replacement origins.
#[derive(Clone, Debug)]
pub struct CssSvgGlyphOrientationVerticalContribution {
    declaration: crate::property::CssSvgGlyphOrientationVerticalDeclaration,
    context: Arc<ContributionContext>,
}
impl CssSvgGlyphOrientationVerticalContribution {
    /// Returns the property, always `glyph-orientation-vertical`.
    #[must_use]
    pub const fn property(&self) -> CssPropertyNameRef<'static> {
        CssPropertyNameRef::SvgGlyphOrientationVertical
    }
    /// Returns the completed ordinary value, or `None` when the declaration was a CSS-wide keyword.
    #[must_use]
    pub const fn ordinary_value(
        &self,
    ) -> Option<&crate::property::CssSvgGlyphOrientationVerticalValue> {
        self.declaration.value()
    }
    /// Returns the CSS-wide keyword, or `None` when the declaration held an ordinary value.
    #[must_use]
    pub const fn global(&self) -> Option<CssGlobalKeyword> {
        self.declaration.global()
    }
    /// Returns the original authored declaration occurrence.
    #[must_use]
    pub fn source(&self) -> &CssDeclaration {
        &self.context.source
    }
    /// Returns the substituted replacement components this contribution was completed from, or
    /// `None` when it was not completed from a substitution reentry.
    #[must_use]
    pub fn replacement_components(&self) -> Option<&CssComponentValues> {
        self.context.replacement.as_ref()
    }
}
fn complete_svg_contribution(
    declaration: &crate::property::CssSvgGlyphOrientationVerticalDeclaration,
    context: Arc<ContributionContext>,
) -> CssContributions {
    debug_assert!(declaration.substitution_dependent().is_none());
    CssContributions::SvgGlyphOrientationVertical(CssSvgGlyphOrientationVerticalContribution {
        declaration: declaration.clone(),
        context,
    })
}

/// Completed intrinsic contributions; custom values and universal resets stay symbolic.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum CssContributions {
    /// One independent SVG terminal write, without a modern TextOrientation projection.
    SvgGlyphOrientationVertical(CssSvgGlyphOrientationVerticalContribution),
    /// The longhand values of a declaration or shorthand expansion.
    Longhands(CssLonghandContributions),
    /// The symbolic effect of an `all` declaration.
    UniversalReset(CssUniversalReset),
    /// A symbolic custom-property declaration.
    Custom(CssCustomPropertyContribution),
}

/// An intrinsic expansion result, possibly awaiting external substitution.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum CssExpansion {
    /// Completed contributions.
    Contributions(CssContributions),
    /// A declaration whose grammar awaits substitution.
    Pending(CssPendingSubstitution),
}

/// A supported authored declaration whose grammar awaits substitution.
///
/// This object retains the original occurrence. Reentry is immutable and
/// retryable; failures never publish a partially expanded contribution set.
#[derive(Clone, Debug)]
pub struct CssPendingSubstitution {
    source: CssDeclaration,
}

impl CssPendingSubstitution {
    /// Returns the unchanged authored declaration awaiting replacement.
    #[must_use]
    pub const fn source(&self) -> &CssDeclaration {
        &self.source
    }

    /// Checks replacement components against the original property and expands
    /// them atomically. This strict transition returns completed contributions
    /// or an error, never another pending state. It preserves the caller's token
    /// origins and the original declaration's identity and importance.
    ///
    /// A decoded `var()`, `env()`, or `attr()` at any component depth returns
    /// `ResidualSubstitution`
    /// before grammar checking. Other invalid values retain the same mapped
    /// error as the source's owning checked grammar, including source resource limits.
    /// Page occurrences use [`CssDeclaration::try_from_components`] with [`crate::property::CssPropertyTarget::PageProperty`] and retain their
    /// explicit `em`/`ex`, physical-side and unitless-length restrictions.
    pub fn reenter(
        &self,
        replacement: CssComponentValues,
    ) -> Result<CssContributions, CssExpansionError> {
        if contains_substitution(&replacement) {
            return Err(CssExpansionError::new(
                CssExpansionErrorKind::ResidualSubstitution,
            ));
        }
        let body = match self.source.body() {
            CssDeclarationBody::Known(known) => match self.source.declaration_context() {
                crate::declaration::syntax::DeclarationContext::Ordinary => {
                    crate::property::property_value::checked_grammar_value_body(
                        known.grammar(),
                        &replacement,
                        self.source.parser_context(),
                    )
                }
                crate::declaration::syntax::DeclarationContext::Page => {
                    crate::property::property_value::checked_page_property_value_body(
                        crate::property::CssPropertyNameRef::Known(known.property()),
                        &replacement,
                        self.source.parser_context(),
                    )
                }
            },
            CssDeclarationBody::SvgGlyphOrientationVertical(value) => {
                crate::property::property_value::checked_svg_glyph_value_body(
                    value.admission(),
                    &replacement,
                    self.source.parser_context(),
                )
            }
            CssDeclarationBody::Custom(_) => {
                unreachable!("custom declarations do not create pending expansion")
            }
        }
        .map_err(|error| {
            CssExpansionError::new(CssExpansionErrorKind::InvalidReplacement(Box::new(error)))
        })?;
        let context = Arc::new(ContributionContext {
            source: self.source.clone(),
            replacement: Some(replacement),
        });
        let known = match body {
            CssDeclarationBody::Known(known) => known,
            CssDeclarationBody::SvgGlyphOrientationVertical(value) => {
                return Ok(complete_svg_contribution(&value, context));
            }
            CssDeclarationBody::Custom(_) => unreachable!("pending grammar cannot become custom"),
        };
        let shape = expansion_shape(&known)?;
        complete_contributions(&known, shape, context)
    }
}

pub(crate) fn expand_declaration(
    source: &CssDeclaration,
) -> Result<CssExpansion, CssExpansionError> {
    let known = match source.body() {
        CssDeclarationBody::SvgGlyphOrientationVertical(value) => {
            if value.substitution_dependent().is_some() {
                return Ok(CssExpansion::Pending(CssPendingSubstitution {
                    source: source.clone(),
                }));
            }
            return Ok(CssExpansion::Contributions(complete_svg_contribution(
                value,
                Arc::new(ContributionContext {
                    source: source.clone(),
                    replacement: None,
                }),
            )));
        }
        CssDeclarationBody::Known(known) => known,
        CssDeclarationBody::Custom(_) => {
            return Ok(CssExpansion::Contributions(CssContributions::Custom(
                CssCustomPropertyContribution {
                    source: source.clone(),
                },
            )));
        }
    };
    let shape = expansion_shape(known);
    if known.substitution_dependent().is_some() && shape.is_ok() {
        return Ok(CssExpansion::Pending(CssPendingSubstitution {
            source: source.clone(),
        }));
    }
    let shape = shape?;
    complete_contributions(
        known,
        shape,
        Arc::new(ContributionContext {
            source: source.clone(),
            replacement: None,
        }),
    )
    .map(CssExpansion::Contributions)
}

/// Counts the schema-selected output before normalization allocates expansion members.
/// Pending, custom, and universal-reset groups each occupy one symbolic output unit.
pub(crate) fn expansion_member_count(source: &CssDeclaration) -> Result<usize, CssExpansionError> {
    let Some(known) = source.known() else {
        return Ok(1);
    };
    let shape = expansion_shape(known);
    if known.substitution_dependent().is_some() && shape.is_ok() {
        return Ok(1);
    }
    let shape = shape?;
    Ok(match shape {
        ExpansionShape::Longhands(members) => members.len(),
        ExpansionShape::UniversalReset => 1,
    })
}

fn complete_contributions(
    known: &CssKnownDeclaration,
    shape: ExpansionShape,
    context: Arc<ContributionContext>,
) -> Result<CssContributions, CssExpansionError> {
    let values = match known.declared_value() {
        CssKnownDeclaredValueRef::Global(keyword) => match shape {
            ExpansionShape::UniversalReset => {
                return Ok(CssContributions::UniversalReset(CssUniversalReset {
                    keyword,
                    context,
                }));
            }
            ExpansionShape::Longhands(members) => members
                .iter()
                .map(|member| member.global(keyword))
                .collect(),
        },
        CssKnownDeclaredValueRef::Property(value) => match value {
            CssKnownPropertyValueRef::Font(font) => match font.value() {
                CssFontValue::System(system) => {
                    let CssPropertyKindRef::Shorthand(metadata) = CssKnownProperty::Font
                        .metadata()
                        .expect("font expansion metadata")
                        .kind()
                    else {
                        unreachable!("font is a shorthand")
                    };
                    let ExpansionShape::Longhands(members) = shape else {
                        unreachable!("font expands into longhands")
                    };
                    members
                        .iter()
                        .enumerate()
                        .map(|(index, member)| {
                            if index < metadata.settable_members().len() {
                                OwnedContributionValue::SystemFont(
                                    CssLonghandProperty(*member),
                                    *system,
                                )
                            } else {
                                member.initial()
                            }
                        })
                        .collect()
                }
                CssFontValue::Explicit(_) => ordinary_values(value)?,
            },
            _ => ordinary_values(value)?,
        },
        CssKnownDeclaredValueRef::SubstitutionDependent(_) => {
            return Err(CssExpansionError::new(
                CssExpansionErrorKind::ResidualSubstitution,
            ));
        }
    };
    Ok(CssContributions::Longhands(CssLonghandContributions {
        items: values
            .into_iter()
            .map(|value| CssLonghandContribution {
                value: Arc::new(value),
                context: Arc::clone(&context),
            })
            .collect(),
    }))
}

#[cfg(test)]
pub(crate) mod metadata_initial_tests;

/// A checked terminal property identity. Construction stays with the schema owner.
/// ```compile_fail
/// use surgeist_css::property::{CssKnownProperty, CssLonghandProperty};
/// let _ = CssLonghandProperty(CssKnownProperty::Width);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssLonghandProperty(Longhand);
impl CssLonghandProperty {
    /// Whether CSSOM ordering must distinguish these terminals because they
    /// share a logical-property group but use different mapping logic. Symbolic
    /// axes stay unresolved; the property schema is the sole relation authority.
    #[must_use]
    pub fn has_mapping_order_conflict(self, other: Self) -> bool {
        let (group, mapping) = self.mapping();
        let (other_group, other_mapping) = other.mapping();
        group.is_some() && group == other_group && mapping != other_mapping
    }

    pub(crate) fn mapping(self) -> (Option<&'static str>, MappingLogic) {
        self.0.mapping()
    }

    /// Returns the corresponding canonical property identity.
    #[must_use]
    pub const fn known_property(self) -> CssKnownProperty {
        self.0.property()
    }
}
/// An owned ordinary value whose active payload determines its terminal property.
/// ```compile_fail
/// use surgeist_css::property::CssLonghandValue;
/// let _ = CssLonghandValue { value: todo!() };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssLonghandValue {
    value: Box<OwnedLonghandValue>,
}
impl CssLonghandValue {
    /// Returns the terminal property coupled to this value.
    #[must_use]
    pub fn property(&self) -> CssLonghandProperty {
        self.value.property()
    }
    /// Borrows its exact symbolic ordinary payload.
    #[must_use]
    pub fn view(&self) -> CssLonghandValueRef<'_> {
        self.value.view()
    }
}
/// Intrinsic initial requirements resolved only by the downstream user agent.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssUserAgentInitial {
    /// The initial `font-family`, which depends on the user agent's default font family.
    FontFamily,
    /// The initial `voice-family`, which depends on the user agent's default voice.
    VoiceFamily,
}
impl CssUserAgentInitial {
    /// Returns the terminal property requiring context.
    #[must_use]
    pub const fn property(self) -> CssLonghandProperty {
        match self {
            Self::FontFamily => CssLonghandProperty(Longhand::FontFamily),
            Self::VoiceFamily => CssLonghandProperty(Longhand::VoiceFamily),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
enum InitialValue {
    Value(CssLonghandValue),
    UserAgent(CssUserAgentInitial),
}
/// An intrinsic initial, retaining either a symbolic ordinary value or a UA requirement.
/// ```compile_fail
/// use surgeist_css::property::CssLonghandInitialValue;
/// let _ = CssLonghandInitialValue { value: todo!() };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssLonghandInitialValue {
    value: InitialValue,
}
/// A borrowed intrinsic initial without invented resolved values or provenance.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CssInitialValueRef<'a> {
    /// A symbolic ordinary initial value.
    Value(&'a CssLonghandValue),
    /// A requirement the user agent resolves.
    UserAgent(CssUserAgentInitial),
}
impl CssLonghandInitialValue {
    /// Returns the property determined by the active initial state.
    #[must_use]
    pub fn property(&self) -> CssLonghandProperty {
        match &self.value {
            InitialValue::Value(v) => v.property(),
            InitialValue::UserAgent(v) => v.property(),
        }
    }
    /// Borrows the initial without resolving external context.
    #[must_use]
    pub fn view(&self) -> CssInitialValueRef<'_> {
        match &self.value {
            InitialValue::Value(v) => CssInitialValueRef::Value(v),
            InitialValue::UserAgent(v) => CssInitialValueRef::UserAgent(*v),
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) enum OwnedContributionValue {
    Ordinary(CssLonghandValue),
    Global(CssLonghandProperty, CssGlobalKeyword),
    UserAgent(CssUserAgentInitial),
    SystemFont(CssLonghandProperty, CssSystemFont),
}
impl OwnedContributionValue {
    fn from_initial(value: CssLonghandInitialValue) -> Self {
        match value.value {
            InitialValue::Value(v) => Self::Ordinary(v),
            InitialValue::UserAgent(v) => Self::UserAgent(v),
        }
    }
    pub(crate) fn property(&self) -> CssKnownProperty {
        match self {
            Self::Ordinary(v) => v.property(),
            Self::Global(p, _) => *p,
            Self::UserAgent(v) => v.property(),
            Self::SystemFont(property, _) => *property,
        }
        .known_property()
    }
    pub(crate) fn view(&self) -> CssContributionValueRef<'_> {
        match self {
            Self::Ordinary(v) => CssContributionValueRef::Ordinary(v.view()),
            Self::Global(_, v) => CssContributionValueRef::Global(*v),
            Self::UserAgent(v) => CssContributionValueRef::UserAgentInitial(*v),
            Self::SystemFont(_, font) => CssContributionValueRef::SystemFont(*font),
        }
    }
}
/// Intrinsic metadata availability is separate from recognition and parser support.
///
/// The grammar's intrinsic metadata is unavailable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssPropertyMetadataError {
    grammar: crate::property::CssPropertyGrammar,
}
impl CssPropertyMetadataError {
    /// Returns the grammar whose intrinsic metadata is unavailable.
    #[must_use]
    pub const fn grammar(&self) -> crate::property::CssPropertyGrammar {
        self.grammar
    }
}
impl fmt::Display for CssPropertyMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "intrinsic metadata unavailable for {}",
            self.grammar.name()
        )
    }
}
impl std::error::Error for CssPropertyMetadataError {}
/// Schema-owned intrinsic grammar, initial and expansion metadata.
#[derive(Debug)]
pub struct CssPropertyMetadata {
    grammar: crate::property::CssPropertyGrammar,
    kind: CssPropertyKindRef<'static>,
}
impl CssPropertyMetadata {
    /// Returns the authored grammar described by this metadata.
    #[must_use]
    pub const fn grammar(&self) -> crate::property::CssPropertyGrammar {
        self.grammar
    }
    /// Returns the exact intrinsic property kind.
    #[must_use]
    pub const fn kind(&self) -> CssPropertyKindRef<'_> {
        self.kind
    }
}
/// Intrinsic metadata branches, never an unavailable placeholder.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum CssPropertyKindRef<'a> {
    /// A terminal property.
    Longhand(&'a CssLonghandMetadata),
    /// A shorthand that expands to longhands.
    Shorthand(&'a CssShorthandMetadata),
    /// An authored coordinate-mode shorthand with exactly four selected sides.
    FourSideShorthand(&'a CssFourSideShorthandMetadata),
    /// The `all` reset.
    UniversalReset(&'a CssUniversalResetMetadata),
}
/// A terminal property's inheritance and intrinsic initial.
#[derive(Debug)]
pub struct CssLonghandMetadata {
    property: CssLonghandProperty,
}
impl CssLonghandMetadata {
    /// Returns the terminal property identity.
    #[must_use]
    pub const fn property(&self) -> CssLonghandProperty {
        self.property
    }
    /// Reports specified inheritance behavior, without performing cascade.
    #[must_use]
    pub const fn inherited_by_default(&self) -> bool {
        self.property.0.inherited()
    }
    /// Constructs its intrinsic initial without a fabricated authored occurrence.
    #[must_use]
    pub fn initial_value(&self) -> CssLonghandInitialValue {
        self.property.0.initial_value()
    }
}
/// Ordered terminal members of a canonical or legacy shorthand.
#[derive(Debug)]
pub struct CssShorthandMetadata {
    members: &'static [CssLonghandProperty],
    settable: &'static [CssLonghandProperty],
    reset: &'static [CssLonghandProperty],
    legacy: bool,
}
impl CssShorthandMetadata {
    /// Returns settable members followed by reset-only members.
    #[must_use]
    pub const fn members(&self) -> &'static [CssLonghandProperty] {
        self.members
    }
    /// Returns members directly settable by the shorthand grammar.
    #[must_use]
    pub const fn settable_members(&self) -> &'static [CssLonghandProperty] {
        self.settable
    }
    /// Returns reset-only terminal members.
    #[must_use]
    pub const fn reset_only_members(&self) -> &'static [CssLonghandProperty] {
        self.reset
    }
    /// Reports a distinct legacy grammar rather than a name-equivalent alias.
    #[must_use]
    pub const fn is_legacy(&self) -> bool {
        self.legacy
    }
}
/// Mode-selected terminal membership for an authored four-side shorthand.
///
/// Physical mode is corroborated by the frozen WebKit boundary. Logical mode
/// preserves Logical 1's authored assignments under Surgeist's explicit no-
/// complementary-reset policy; the selected draft does not settle that reset
/// question. CSS-wide values use physical mode; reentry uses the replacement mode.
#[derive(Debug)]
pub struct CssFourSideShorthandMetadata {
    physical: &'static [CssLonghandProperty],
    logical: &'static [CssLonghandProperty],
}
impl CssFourSideShorthandMetadata {
    /// Returns exactly four ordered sides in the requested coordinate mode.
    #[must_use]
    pub const fn members(
        &self,
        mode: crate::values::common::CssBoxSideKind,
    ) -> &'static [CssLonghandProperty] {
        match mode {
            crate::values::common::CssBoxSideKind::Physical => self.physical,
            crate::values::common::CssBoxSideKind::Logical => self.logical,
        }
    }
    /// Returns the four directly settable sides, without complementary writes.
    #[must_use]
    pub const fn settable_members(
        &self,
        mode: crate::values::common::CssBoxSideKind,
    ) -> &'static [CssLonghandProperty] {
        self.members(mode)
    }
    /// Returns the empty reset-only set in either mode.
    #[must_use]
    pub const fn reset_only_members(&self) -> &'static [CssLonghandProperty] {
        &[]
    }
}
/// Intrinsic exclusions for `all`, before contextual target selection.
/// ```compile_fail
/// use surgeist_css::property::CssUniversalResetMetadata;
/// let _ = CssUniversalResetMetadata { _private: () };
/// ```
#[derive(Debug)]
pub struct CssUniversalResetMetadata {
    _private: (),
}
impl CssUniversalResetMetadata {
    /// Reports explicit schema exclusions without selecting cascade winners.
    #[must_use]
    pub fn excludes(&self, property: CssPropertyNameRef<'_>) -> bool {
        universal_excludes(property)
    }
}
impl CssDeclaration {
    /// Expands custom declarations and every recognized property intrinsically.
    ///
    /// Ordinary shorthands contribute every member, applying intrinsic initial
    /// values to omissions. `border` also resets the five border-image longhands;
    /// CSS-wide keywords propagate to ordinary and reset-only members alike. `all`
    /// remains a symbolic reset. Substitution-dependent known declarations return a
    /// pending handle for strict grammar reentry after substitution.
    /// Custom declarations return one completed symbolic contribution even when their
    /// tokens contain `var()`; their values are not substituted or computed here.
    /// This operation does not choose cascade winners, substitute variables,
    /// resolve writing modes, evaluate lengths or colors, or load images.
    ///
    /// # Errors
    ///
    /// Returns the declaration the intrinsic expansion cannot admit.
    pub fn expand(&self) -> Result<CssExpansion, CssExpansionError> {
        expand_declaration(self)
    }
}
