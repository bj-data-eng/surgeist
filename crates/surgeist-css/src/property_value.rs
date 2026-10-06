//! Checked property-value construction and diagnostics mapped to input components.

use std::fmt;

use crate::{
    CssComponentValueError, CssComponentValueErrorKind, CssComponentValues, CssDeclaration,
    CssImportance, CssPropertyNameRef, CssSerializedOrigin, CssSerializedValue, Error, ErrorKind,
};

/// A component invariant or property grammar rejected by checked value construction.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssPropertyValueErrorKind {
    /// Component serialization could not preserve the supplied syntax or resource bounds.
    Component(CssComponentValueErrorKind),
    /// The declaration-value boundary or selected property grammar rejected the input.
    Grammar(ErrorKind),
}

/// A property-value construction failure located in the original component input.
///
/// Unlike a stylesheet [`Error`], this failure can originate in programmatic or mixed-source
/// tokens. It never presents generated serialization coordinates as authored source positions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssPropertyValueParseError {
    detail: Box<PropertyValueParseErrorDetail>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PropertyValueParseErrorDetail {
    kind: CssPropertyValueErrorKind,
    origin: CssSerializedOrigin,
}

impl CssPropertyValueParseError {
    /// Returns the structured component or grammar failure.
    #[must_use]
    pub fn kind(&self) -> &CssPropertyValueErrorKind {
        &self.detail.kind
    }

    /// Returns the responsible original token, generated boundary, or EOF origin.
    #[must_use]
    pub fn origin(&self) -> &CssSerializedOrigin {
        &self.detail.origin
    }

    fn from_component(error: CssComponentValueError) -> Self {
        Self {
            detail: Box::new(PropertyValueParseErrorDetail {
                kind: CssPropertyValueErrorKind::Component(error.kind()),
                origin: CssSerializedOrigin::Token(error.origin().clone()),
            }),
        }
    }

    pub(crate) fn from_grammar(error: Error, serialized: &CssSerializedValue) -> Self {
        if let ErrorKind::InvalidComponentValue(detail) = error.kind()
            && crate::error::is_component_resource_error(detail)
        {
            return Self::from_component((**detail).clone());
        }
        let origin = serialized
            .origin_at(error.position().byte_offset().value())
            .expect("the property parser reports a cursor within its serialized input or at EOF")
            .clone();
        Self {
            detail: Box::new(PropertyValueParseErrorDetail {
                kind: CssPropertyValueErrorKind::Grammar(error.kind().clone()),
                origin,
            }),
        }
    }
}

impl fmt::Display for CssPropertyValueParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS property value {:?}", self.kind())
    }
}

impl std::error::Error for CssPropertyValueParseError {}

fn is_time_property(property: crate::CssKnownProperty) -> bool {
    matches!(
        property,
        crate::CssKnownProperty::TransitionDuration
            | crate::CssKnownProperty::TransitionDelay
            | crate::CssKnownProperty::AnimationDuration
            | crate::CssKnownProperty::AnimationDelay
            | crate::CssKnownProperty::Transition
            | crate::CssKnownProperty::Animation
    )
}

fn requires_closed_components(property: crate::CssKnownProperty) -> bool {
    matches!(
        property,
        crate::CssKnownProperty::Transform
            | crate::CssKnownProperty::TransformBox
            | crate::CssKnownProperty::TransformOrigin
            | crate::CssKnownProperty::Translate
            | crate::CssKnownProperty::Rotate
            | crate::CssKnownProperty::Scale
            | crate::CssKnownProperty::Perspective
            | crate::CssKnownProperty::PerspectiveOrigin
            | crate::CssKnownProperty::TransformStyle
            | crate::CssKnownProperty::BackfaceVisibility
            | crate::CssKnownProperty::CaretColor
            | crate::CssKnownProperty::Outline
            | crate::CssKnownProperty::OutlineColor
            | crate::CssKnownProperty::OutlineStyle
            | crate::CssKnownProperty::OutlineWidth
            | crate::CssKnownProperty::OutlineOffset
            | crate::CssKnownProperty::NavUp
            | crate::CssKnownProperty::NavRight
            | crate::CssKnownProperty::NavDown
            | crate::CssKnownProperty::NavLeft
            | crate::CssKnownProperty::CaretAnimation
            | crate::CssKnownProperty::CaretShape
            | crate::CssKnownProperty::Caret
            | crate::CssKnownProperty::Interactivity
            | crate::CssKnownProperty::InterestDelayStart
            | crate::CssKnownProperty::InterestDelayEnd
            | crate::CssKnownProperty::InterestDelay
            | crate::CssKnownProperty::AccentColor
            | crate::CssKnownProperty::Appearance
            | crate::CssKnownProperty::TextDecoration
            | crate::CssKnownProperty::TextDecorationLine
            | crate::CssKnownProperty::TextDecorationColor
            | crate::CssKnownProperty::TextDecorationStyle
            | crate::CssKnownProperty::TextDecorationThickness
            | crate::CssKnownProperty::TextUnderlinePosition
            | crate::CssKnownProperty::TextUnderlineOffset
            | crate::CssKnownProperty::TextDecorationSkip
            | crate::CssKnownProperty::TextDecorationSkipSelf
            | crate::CssKnownProperty::TextDecorationSkipBox
            | crate::CssKnownProperty::TextDecorationSkipInset
            | crate::CssKnownProperty::TextDecorationSkipSpaces
            | crate::CssKnownProperty::TextDecorationSkipInk
            | crate::CssKnownProperty::TextEmphasis
            | crate::CssKnownProperty::TextEmphasisStyle
            | crate::CssKnownProperty::TextEmphasisColor
            | crate::CssKnownProperty::TextEmphasisPosition
            | crate::CssKnownProperty::TextEmphasisSkip
            | crate::CssKnownProperty::TextShadow
            | crate::CssKnownProperty::Mask
            | crate::CssKnownProperty::MaskImage
            | crate::CssKnownProperty::MaskSize
            | crate::CssKnownProperty::MaskPosition
            | crate::CssKnownProperty::MaskRepeat
            | crate::CssKnownProperty::MaskMode
            | crate::CssKnownProperty::MaskOrigin
            | crate::CssKnownProperty::MaskClip
            | crate::CssKnownProperty::MaskComposite
            | crate::CssKnownProperty::MaskBorder
            | crate::CssKnownProperty::MaskBorderSource
            | crate::CssKnownProperty::MaskBorderSlice
            | crate::CssKnownProperty::MaskBorderWidth
            | crate::CssKnownProperty::MaskBorderOutset
            | crate::CssKnownProperty::MaskBorderRepeat
            | crate::CssKnownProperty::MaskBorderMode
            | crate::CssKnownProperty::ClipRule
            | crate::CssKnownProperty::MaskType
            | crate::CssKnownProperty::Contain
            | crate::CssKnownProperty::ContentVisibility
            | crate::CssKnownProperty::Cursor
            | crate::CssKnownProperty::PointerEvents
            | crate::CssKnownProperty::UserSelect
            | crate::CssKnownProperty::Resize
            | crate::CssKnownProperty::Hyphens
            | crate::CssKnownProperty::HyphenateCharacter
            | crate::CssKnownProperty::HyphenateLimitZone
            | crate::CssKnownProperty::HyphenateLimitChars
            | crate::CssKnownProperty::HyphenateLimitLines
            | crate::CssKnownProperty::HyphenateLimitLast
            | crate::CssKnownProperty::TextJustify
            | crate::CssKnownProperty::TextGroupAlign
            | crate::CssKnownProperty::LinePadding
            | crate::CssKnownProperty::TextAutospace
            | crate::CssKnownProperty::TextSpacingTrim
            | crate::CssKnownProperty::TextSpacing
            | crate::CssKnownProperty::HangingPunctuation
            | crate::CssKnownProperty::TextAlign
            | crate::CssKnownProperty::TextAlignAll
            | crate::CssKnownProperty::TextAlignLast
            | crate::CssKnownProperty::OverflowWrap
            | crate::CssKnownProperty::WordSpacing
            | crate::CssKnownProperty::LetterSpacing
            | crate::CssKnownProperty::TextTransform
            | crate::CssKnownProperty::WrapInside
            | crate::CssKnownProperty::WrapBefore
            | crate::CssKnownProperty::WrapAfter
            | crate::CssKnownProperty::LineBreak
            | crate::CssKnownProperty::WordSpaceTransform
            | crate::CssKnownProperty::TabSize
            | crate::CssKnownProperty::TextIndent
            | crate::CssKnownProperty::VerticalAlign
            | crate::CssKnownProperty::ClipPath
            | crate::CssKnownProperty::TextWrap
            | crate::CssKnownProperty::TextWrapMode
            | crate::CssKnownProperty::TextWrapStyle
            | crate::CssKnownProperty::WhiteSpace
            | crate::CssKnownProperty::WhiteSpaceCollapse
            | crate::CssKnownProperty::WhiteSpaceTrim
            | crate::CssKnownProperty::WordBreak
            | crate::CssKnownProperty::ItemDirection
            | crate::CssKnownProperty::ItemWrap
            | crate::CssKnownProperty::ItemPack
            | crate::CssKnownProperty::ItemFlow
            | crate::CssKnownProperty::FlowTolerance
            | crate::CssKnownProperty::GridTemplateRows
            | crate::CssKnownProperty::GridTemplateColumns
            | crate::CssKnownProperty::GridTemplate
            | crate::CssKnownProperty::Grid
            | crate::CssKnownProperty::ObjectPosition
            | crate::CssKnownProperty::ImageOrientation
            | crate::CssKnownProperty::ImageRendering
            | crate::CssKnownProperty::ObjectFit
            | crate::CssKnownProperty::BackgroundBlendMode
            | crate::CssKnownProperty::Isolation
            | crate::CssKnownProperty::MixBlendMode
            | crate::CssKnownProperty::CueBefore
            | crate::CssKnownProperty::CueAfter
            | crate::CssKnownProperty::Cue
            | crate::CssKnownProperty::TransitionProperty
            | crate::CssKnownProperty::TransitionTimingFunction
            | crate::CssKnownProperty::AnimationTimingFunction
            | crate::CssKnownProperty::AnimationName
            | crate::CssKnownProperty::AnimationIterationCount
            | crate::CssKnownProperty::AnimationDirection
            | crate::CssKnownProperty::AnimationFillMode
            | crate::CssKnownProperty::AnimationPlayState
            | crate::CssKnownProperty::TransitionDuration
            | crate::CssKnownProperty::TransitionDelay
            | crate::CssKnownProperty::AnimationDuration
            | crate::CssKnownProperty::AnimationDelay
            | crate::CssKnownProperty::Transition
            | crate::CssKnownProperty::Animation
            | crate::CssKnownProperty::FontVariant
            | crate::CssKnownProperty::FontVariantLigatures
            | crate::CssKnownProperty::FontVariantCaps
            | crate::CssKnownProperty::FontVariantAlternates
            | crate::CssKnownProperty::FontVariantNumeric
            | crate::CssKnownProperty::FontVariantEastAsian
            | crate::CssKnownProperty::FontVariantPosition
            | crate::CssKnownProperty::FontVariantEmoji
            | crate::CssKnownProperty::FontFeatureSettings
            | crate::CssKnownProperty::FontSizeAdjust
            | crate::CssKnownProperty::FontVariationSettings
            | crate::CssKnownProperty::FontPalette
    )
}

fn recovered_component_error(
    property: crate::CssKnownProperty,
    value: &CssComponentValues,
) -> Option<CssPropertyValueParseError> {
    if !requires_closed_components(property) {
        return None;
    }
    let origin = value.first_implicit_origin()?;
    Some(CssPropertyValueParseError {
        detail: Box::new(PropertyValueParseErrorDetail {
            kind: CssPropertyValueErrorKind::Grammar(crate::error::implicit_eof("").kind().clone()),
            origin: CssSerializedOrigin::End(Some(origin.clone())),
        }),
    })
}

/// Checks one property's owned components and constructs an authored declaration occurrence.
///
/// The property identity selects the same grammar as stylesheet declarations. Whole-value
/// CSS-wide keywords and valid substitution-dependent values remain symbolic. Custom names
/// preserve case and allow empty values. Top-level annotations and declaration delimiters are
/// rejected: importance comes only from the separate argument.
///
/// Supplied token origins remain intact. The returned declaration has no parsed property-name
/// position or single parsed declaration-value span; inspect its components for their origins.
/// Serialization preserves token boundaries, checks byte arithmetic, and obeys the immutable
/// component tree's previously validated structural and serialization constraints. This function
/// does not substitute variables, expand shorthands, resolve values, or apply cascade.
pub fn parse_property_value(
    property: CssPropertyNameRef<'_>,
    value: CssComponentValues,
    importance: CssImportance,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let body = checked_property_value_body(property, &value)?;
    Ok(CssDeclaration::new_constructed(body, importance, value))
}

// Declaration construction and strict expansion reentry share both the checked
// grammar and its original-component error mapping. Reentry retains the original
// occurrence and therefore must not manufacture a replacement declaration.
pub(crate) fn checked_property_value_body(
    property: CssPropertyNameRef<'_>,
    value: &CssComponentValues,
) -> Result<crate::CssDeclarationBody, CssPropertyValueParseError> {
    if let CssPropertyNameRef::Known(known) = property
        && is_time_property(known)
        && let Some(error) = recovered_component_error(known, value)
    {
        return Err(error);
    }
    let serialized = value
        .serialize()
        .map_err(CssPropertyValueParseError::from_component)?;
    if let CssPropertyNameRef::Known(known) = property
        && !is_time_property(known)
        && let Some(error) = recovered_component_error(known, value)
    {
        return Err(error);
    }
    crate::parser::parse_property_value_body(
        property,
        serialized.as_css(),
        &crate::numeric::NumericInputContext::components(value, &serialized),
    )
    .map_err(|error| CssPropertyValueParseError::from_grammar(error, &serialized))
}

/// Checks components using an explicit canonical or legacy authored grammar.
///
/// This shares token-boundary preservation, limits, and original-origin mapping
/// with [`parse_property_value`]. The constructed occurrence has programmatic
/// declaration provenance; supplied component origins remain intact.
pub fn parse_property_value_for_grammar(
    grammar: crate::CssPropertyGrammar,
    value: CssComponentValues,
    importance: CssImportance,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let body = checked_grammar_value_body(grammar, &value)?;
    Ok(CssDeclaration::new_constructed(body, importance, value))
}
pub(crate) fn checked_grammar_value_body(
    grammar: crate::CssPropertyGrammar,
    value: &CssComponentValues,
) -> Result<crate::CssDeclarationBody, CssPropertyValueParseError> {
    if is_time_property(grammar.target_property())
        && let Some(error) = recovered_component_error(grammar.target_property(), value)
    {
        return Err(error);
    }
    let serialized = value
        .serialize()
        .map_err(CssPropertyValueParseError::from_component)?;
    if !is_time_property(grammar.target_property())
        && let Some(error) = recovered_component_error(grammar.target_property(), value)
    {
        return Err(error);
    }
    crate::parser::parse_property_value_body_for_grammar(
        grammar,
        serialized.as_css(),
        &crate::numeric::NumericInputContext::components(value, &serialized),
    )
    .map_err(|error| CssPropertyValueParseError::from_grammar(error, &serialized))
}
