//! Intrinsic authored text alignment before direction, font, or layout resolution.

use std::fmt;

use unicode_segmentation::UnicodeSegmentation;

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssComponentValue, CssComponentValueErrorKind, CssComponentValueRef,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits, CssTextAlign,
    CssValueOrigin, CssValueTokenRef,
};

type SerializationResult<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

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

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssCharacterAlignmentErrorKind {
    InvalidCharacter,
    Component(CssComponentValueErrorKind),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCharacterAlignmentError {
    kind: CssCharacterAlignmentErrorKind,
    origin: CssValueOrigin,
}

impl CssCharacterAlignmentError {
    #[must_use]
    pub const fn kind(&self) -> CssCharacterAlignmentErrorKind {
        self.kind
    }

    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
}

impl fmt::Display for CssCharacterAlignmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            CssCharacterAlignmentErrorKind::InvalidCharacter => {
                formatter.write_str("text alignment requires exactly one extended grapheme cluster")
            }
            CssCharacterAlignmentErrorKind::Component(_) => {
                formatter.write_str("invalid text alignment string component")
            }
        }
    }
}

impl std::error::Error for CssCharacterAlignmentError {}

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
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        serialize(limits, |context, output| {
            self.append_specified(context, output)
        })
    }

    pub fn try_new(
        decoded: impl Into<String>,
        authored_fallback: Option<CssTextAlignPosition>,
    ) -> Result<Self, CssCharacterAlignmentError> {
        let component =
            CssComponentValue::try_string(decoded).map_err(|error| CssCharacterAlignmentError {
                kind: CssCharacterAlignmentErrorKind::Component(error.kind()),
                origin: error.origin().clone(),
            })?;
        Self::try_from_component(component, authored_fallback)
    }

    pub fn try_from_component(
        component: CssComponentValue,
        authored_fallback: Option<CssTextAlignPosition>,
    ) -> Result<Self, CssCharacterAlignmentError> {
        let CssComponentValueRef::Token(CssValueTokenRef::String(decoded)) = component.view()
        else {
            return Err(CssCharacterAlignmentError {
                kind: CssCharacterAlignmentErrorKind::Component(
                    CssComponentValueErrorKind::InvalidString,
                ),
                origin: component.origin().clone(),
            });
        };
        if UnicodeSegmentation::graphemes(decoded, true).count() != 1 {
            return Err(CssCharacterAlignmentError {
                kind: CssCharacterAlignmentErrorKind::InvalidCharacter,
                origin: component.origin().clone(),
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
        context.charge_input(1)?;
        context.charge_projection(1)?;
        let mut writer = BoundedWriter {
            context,
            output,
            failure: None,
        };
        if cssparser::serialize_string(self.decoded(), &mut writer).is_err() {
            return Err(writer.failure.unwrap_or_else(|| {
                CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            }));
        }
        if let Some(fallback) = self.authored_fallback {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            context.append(output, " ")?;
            context.append(output, fallback.keyword())?;
        }
        Ok(())
    }
}

struct BoundedWriter<'a> {
    context: &'a mut SpecifiedSerializationContext,
    output: &'a mut String,
    failure: Option<CssSpecifiedValueSerializationError>,
}

impl fmt::Write for BoundedWriter<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.context.append(self.output, text).map_err(|error| {
            self.failure = Some(error);
            fmt::Error
        })
    }
}

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

fn serialize(
    limits: CssSpecifiedValueSerializationLimits,
    append: impl FnOnce(&mut SpecifiedSerializationContext, &mut String) -> SerializationResult<()>,
) -> SerializationResult<String> {
    let mut context = SpecifiedSerializationContext::new(limits);
    let mut output = String::new();
    append(&mut context, &mut output)?;
    Ok(output)
}

impl CssTextAlignAllValue {
    #[must_use]
    pub const fn initial() -> Self {
        Self::Keyword(CssTextAlign::Start)
    }

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        serialize(limits, |context, output| {
            self.append_specified(context, output)
        })
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

impl CssTextAlignValue {
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        serialize(limits, |context, output| match self {
            Self::Alignment(value) => value.append_specified(context, output),
            Self::JustifyAll => append_raw_keyword("justify-all", context, output),
        })
    }
}

impl CssTextAlignLastValue {
    #[must_use]
    pub const fn initial() -> Self {
        Self::Auto
    }

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        serialize(limits, |context, output| match self {
            Self::Auto => append_raw_keyword("auto", context, output),
            Self::Keyword(value) => append_keyword(*value, context, output),
        })
    }
}

fn append_keyword(
    value: CssTextAlign,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> SerializationResult<()> {
    append_raw_keyword(
        match value {
            CssTextAlign::Start => "start",
            CssTextAlign::End => "end",
            CssTextAlign::Left => "left",
            CssTextAlign::Right => "right",
            CssTextAlign::Center => "center",
            CssTextAlign::Justify => "justify",
            CssTextAlign::MatchParent => "match-parent",
        },
        context,
        output,
    )
}

fn append_raw_keyword(
    keyword: &str,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> SerializationResult<()> {
    context.charge_input(1)?;
    context.charge_projection(1)?;
    context.append(output, keyword)
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
