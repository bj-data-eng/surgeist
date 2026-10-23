//! Intrinsic authored text alignment before direction, font, or layout resolution.

use std::fmt;

use unicode_segmentation::UnicodeSegmentation;

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef,
    CssSerializationError, CssTextAlign, CssValueOrigin, CssValueTokenRef,
};

type SerializationResult<T> = std::result::Result<T, CssSerializationError>;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssTextAlignPosition {
    Start,
    End,
    Left,
    Right,
    Center,
}

impl CssTextAlignPosition {
    const fn keyword(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
            Self::Left => "left",
            Self::Right => "right",
            Self::Center => "center",
        }
    }
}

/// A rejected checked text value: a character-based alignment string that is not exactly
/// one extended grapheme cluster, or whose component is not a representable string.
///
/// A rejected component is exposed by [`Self::component_error`] and is the error's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTextConstructionError {
    kind: TextConstructionKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TextConstructionKind {
    InvalidCharacter(CssValueOrigin),
    Component(CssComponentValueError),
}

impl CssTextConstructionError {
    /// Returns whether an alignment string is not exactly one extended grapheme cluster.
    #[must_use]
    pub const fn is_invalid_character(&self) -> bool {
        matches!(self.kind, TextConstructionKind::InvalidCharacter(_))
    }

    /// Returns the rejected string component.
    #[must_use]
    pub const fn component_error(&self) -> Option<&CssComponentValueError> {
        match &self.kind {
            TextConstructionKind::Component(error) => Some(error),
            TextConstructionKind::InvalidCharacter(_) => None,
        }
    }

    /// Returns the origin of the rejected component.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        match &self.kind {
            TextConstructionKind::InvalidCharacter(origin) => origin,
            TextConstructionKind::Component(error) => error.origin(),
        }
    }
}

impl From<CssComponentValueError> for CssTextConstructionError {
    fn from(error: CssComponentValueError) -> Self {
        Self {
            kind: TextConstructionKind::Component(error),
        }
    }
}

impl fmt::Display for CssTextConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            TextConstructionKind::InvalidCharacter(_) => {
                formatter.write_str("text alignment requires exactly one extended grapheme cluster")
            }
            TextConstructionKind::Component(_) => {
                formatter.write_str("invalid text alignment string component")
            }
        }
    }
}

impl std::error::Error for CssTextConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.component_error()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}

/// One decoded default extended grapheme cluster and its authored fallback.
///
/// The retained component carries original token spelling and provenance. A
/// missing fallback remains distinct from an explicitly authored `right`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCharacterAlignment {
    component: Box<CssComponentValue>,
    authored_fallback: Option<CssTextAlignPosition>,
}

impl CssCharacterAlignment {
    pub fn try_new(
        decoded: impl Into<String>,
        authored_fallback: Option<CssTextAlignPosition>,
    ) -> Result<Self, CssTextConstructionError> {
        let component = CssComponentValue::try_string(decoded)?;
        Self::try_from_component(component, authored_fallback)
    }

    pub fn try_from_component(
        component: CssComponentValue,
        authored_fallback: Option<CssTextAlignPosition>,
    ) -> Result<Self, CssTextConstructionError> {
        let CssComponentValueRef::Token(CssValueTokenRef::String(decoded)) = component.view()
        else {
            return Err(CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidString,
                component.origin().clone(),
            )
            .into());
        };
        if UnicodeSegmentation::graphemes(decoded, true).count() != 1 {
            return Err(CssTextConstructionError {
                kind: TextConstructionKind::InvalidCharacter(component.origin().clone()),
            });
        }
        Ok(Self {
            component: Box::new(component),
            authored_fallback,
        })
    }

    #[must_use]
    pub fn decoded(&self) -> &str {
        let CssComponentValueRef::Token(CssValueTokenRef::String(decoded)) = self.component.view()
        else {
            unreachable!("checked string component")
        };
        decoded
    }

    #[must_use]
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }

    #[must_use]
    pub const fn authored_fallback(&self) -> Option<CssTextAlignPosition> {
        self.authored_fallback
    }

    #[must_use]
    pub fn effective_fallback(&self) -> CssTextAlignPosition {
        self.authored_fallback
            .unwrap_or(CssTextAlignPosition::Right)
    }

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        context.charge_nodes(1)?;
        crate::serialization_escaping::append_string(self.decoded(), context, output)?;
        if let Some(fallback) = self.authored_fallback {
            context.charge_nodes(1)?;
            context.append(output, " ")?;
            context.append(output, fallback.keyword())?;
        }
        Ok(())
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssCharacterAlignment {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(&mut writer.context, &mut writer.css)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssCharacterAlignment);

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssTextAlignAllValue {
    Keyword(CssTextAlign),
    Character(CssCharacterAlignment),
}

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssTextAlignValue {
    Alignment(CssTextAlignAllValue),
    JustifyAll,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssTextAlignLastValue {
    Auto,
    Keyword(CssTextAlign),
}

impl CssTextAlignAllValue {
    #[must_use]
    pub const fn initial() -> Self {
        Self::Keyword(CssTextAlign::Start)
    }

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Keyword(value) => append_keyword(*value, context, output),
            Self::Character(value) => value.append_specified(context, output),
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssTextAlignAllValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(&mut writer.context, &mut writer.css)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssTextAlignAllValue);

impl crate::specified_rule_serialization::SpecifiedProvider for CssTextAlignValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        match self {
            Self::Alignment(value) => value.append_specified(context, output),
            Self::JustifyAll => context.keyword(output, "justify-all"),
        }
    }
}
crate::specified_rule_serialization::specified_serialization!(CssTextAlignValue);

impl CssTextAlignLastValue {
    #[must_use]
    pub const fn initial() -> Self {
        Self::Auto
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssTextAlignLastValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        match self {
            Self::Auto => context.keyword(output, "auto"),
            Self::Keyword(value) => append_keyword(*value, context, output),
        }
    }
}
crate::specified_rule_serialization::specified_serialization!(CssTextAlignLastValue);

fn append_keyword(
    value: CssTextAlign,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> SerializationResult<()> {
    context.keyword(
        output,
        match value {
            CssTextAlign::Start => "start",
            CssTextAlign::End => "end",
            CssTextAlign::Left => "left",
            CssTextAlign::Right => "right",
            CssTextAlign::Center => "center",
            CssTextAlign::Justify => "justify",
            CssTextAlign::MatchParent => "match-parent",
        },
    )
}

pub(crate) fn shorthand_all(value: &CssTextAlignValue) -> CssTextAlignAllValue {
    match value {
        CssTextAlignValue::Alignment(value) => value.clone(),
        CssTextAlignValue::JustifyAll => CssTextAlignAllValue::Keyword(CssTextAlign::Justify),
    }
}

pub(crate) fn shorthand_last(value: &CssTextAlignValue) -> CssTextAlignLastValue {
    match value {
        CssTextAlignValue::Alignment(CssTextAlignAllValue::Keyword(CssTextAlign::MatchParent)) => {
            CssTextAlignLastValue::Keyword(CssTextAlign::MatchParent)
        }
        CssTextAlignValue::JustifyAll => CssTextAlignLastValue::Keyword(CssTextAlign::Justify),
        _ => CssTextAlignLastValue::Auto,
    }
}
