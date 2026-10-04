//! Intrinsic authored values from CSS Speech 1 §§7–9.
//!
//! These values retain author choices. Voice selection, pronunciation and the
//! contextual computed/used behavior of `speak: auto` belong downstream.
//!
//! Both properties apply to all elements, inherit, have no percentage values,
//! and use grammar order for specified serialization. Their initial values are
//! `auto` and `normal`. Speech 1's property tables label computed values as the
//! specified value; the prose additionally gives contextual rules for `auto`.

use crate::specified_serialization::{SpecifiedSerializationContext, serialize_keyword_sequence};
use crate::{
    CssDuration, CssNumericConstructionError, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssTimeValue,
};

type SerializationResult = Result<String, CssSpecifiedValueSerializationError>;

/// Whether an element is authored to participate in speech rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSpeak {
    /// Contextual participation, resolved downstream using display/visibility.
    Auto,
    /// Suppress this element's aural contribution.
    Never,
    /// Request this element's aural contribution.
    Always,
}

impl CssSpeak {
    /// Serializes the specified keyword without resolving speech participation.
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes one semantic leaf under the shared keyword resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        serialize_keyword_sequence(
            match self {
                Self::Auto => "auto",
                Self::Never => "never",
                Self::Always => "always",
            },
            limits,
        )
    }
}

/// An explicitly authored punctuation mode for `speak-as`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSpeakAsPunctuation {
    /// Name punctuation aloud (`literal-punctuation`).
    Literal,
    /// Omit punctuation from speech and pauses (`no-punctuation`).
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpeakAsState {
    Normal,
    Modifiers {
        spell_out: bool,
        digits: bool,
        punctuation: Option<CssSpeakAsPunctuation>,
    },
}

/// Checked `normal` or a nonempty unordered speech modifier composition.
///
/// `normal` cannot carry modifiers. A modifier composition contains each of
/// `spell-out` and `digits` at most once and at most one punctuation mode.
/// Original order, case, escapes and token origins remain available through
/// the property wrapper and declaration components, independently of this model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssSpeakAs {
    state: SpeakAsState,
}

impl CssSpeakAs {
    /// Constructs the distinct language-dependent `normal` alternative.
    #[must_use]
    pub const fn normal() -> Self {
        Self {
            state: SpeakAsState::Normal,
        }
    }

    /// Constructs a nonempty composition; the empty composition is rejected.
    ///
    /// `None` leaves punctuation to language-dependent pronunciation rules;
    /// `Some(CssSpeakAsPunctuation::None)` explicitly suppresses punctuation.
    #[must_use]
    pub const fn try_new(
        spell_out: bool,
        digits: bool,
        punctuation: Option<CssSpeakAsPunctuation>,
    ) -> Option<Self> {
        if !spell_out && !digits && punctuation.is_none() {
            return None;
        }
        Some(Self {
            state: SpeakAsState::Modifiers {
                spell_out,
                digits,
                punctuation,
            },
        })
    }

    /// Reports the distinct `normal` alternative.
    #[must_use]
    pub const fn is_normal(self) -> bool {
        matches!(self.state, SpeakAsState::Normal)
    }

    /// Reports an authored `spell-out` modifier.
    #[must_use]
    pub const fn spell_out(self) -> bool {
        matches!(
            self.state,
            SpeakAsState::Modifiers {
                spell_out: true,
                ..
            }
        )
    }

    /// Reports an authored `digits` modifier.
    #[must_use]
    pub const fn digits(self) -> bool {
        matches!(self.state, SpeakAsState::Modifiers { digits: true, .. })
    }

    /// Returns the explicit punctuation mode, if authored.
    #[must_use]
    pub const fn punctuation(self) -> Option<CssSpeakAsPunctuation> {
        match self.state {
            SpeakAsState::Normal => None,
            SpeakAsState::Modifiers { punctuation, .. } => punctuation,
        }
    }

    /// Serializes in grammar order: spell-out, digits, then punctuation.
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes one semantic keyword-combination leaf under shared limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        use CssSpeakAsPunctuation::{Literal, None as Omit};
        let text = match self.state {
            SpeakAsState::Normal => "normal",
            SpeakAsState::Modifiers {
                spell_out,
                digits,
                punctuation,
            } => match (spell_out, digits, punctuation) {
                (true, false, None) => "spell-out",
                (false, true, None) => "digits",
                (true, true, None) => "spell-out digits",
                (false, false, Some(Literal)) => "literal-punctuation",
                (true, false, Some(Literal)) => "spell-out literal-punctuation",
                (false, true, Some(Literal)) => "digits literal-punctuation",
                (true, true, Some(Literal)) => "spell-out digits literal-punctuation",
                (false, false, Some(Omit)) => "no-punctuation",
                (true, false, Some(Omit)) => "spell-out no-punctuation",
                (false, true, Some(Omit)) => "digits no-punctuation",
                (true, true, Some(Omit)) => "spell-out digits no-punctuation",
                (false, false, None) => unreachable!("checked nonempty modifier composition"),
            },
        };
        serialize_keyword_sequence(text, limits)
    }
}

/// An authored pause/rest strength whose absolute duration remains contextual.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSpeechBreakStrength {
    XWeak,
    Weak,
    Medium,
    Strong,
    XStrong,
}

impl CssSpeechBreakStrength {
    const fn as_css(self) -> &'static str {
        match self {
            Self::XWeak => "x-weak",
            Self::Weak => "weak",
            Self::Medium => "medium",
            Self::Strong => "strong",
            Self::XStrong => "x-strong",
        }
    }
}

/// One authored pause or rest, retaining a strength, `none`, or checked duration.
///
/// Ordinary times are nonnegative under [`CssDuration`]'s exact lexical rule;
/// calculations remain authored until computed range handling downstream.
/// `none` is preserved separately from an explicit zero time. This model does
/// not assign strength durations or execute pause collapse or additive rests.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSpeechBreak {
    None,
    Strength(CssSpeechBreakStrength),
    Time(CssDuration),
}

impl CssSpeechBreak {
    /// Checks ordinary time range and original closure through the duration owner.
    pub fn try_time(time: CssTimeValue) -> Result<Self, CssNumericConstructionError> {
        CssDuration::try_new(time).map(Self::Time)
    }

    fn ensure_closed(&self) -> Result<(), CssNumericConstructionError> {
        match self {
            Self::Time(duration) => duration.time().ensure_closed(),
            _ => Ok(()),
        }
    }
    fn specified_value_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Time(left), Self::Time(right)) => left.specified_value_eq(right),
            (Self::None, Self::Time(duration)) | (Self::Time(duration), Self::None) => {
                duration.is_exact_ordinary_zero()
            }
            _ => self == other,
        }
    }
    /// Serializes a specified keyword or canonical time without speech execution.
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Uses the shared cumulative input, projection and emitted-byte policy.
    /// Time output follows [`CssDuration::serialize_specified`]; authored input
    /// and provenance remain unchanged, including on atomic failure.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
    }

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        let keyword = match self {
            Self::None => "none",
            Self::Strength(strength) => strength.as_css(),
            Self::Time(duration) => return duration.append_specified(context, output),
        };
        context.charge_input(1)?;
        context.charge_projection(1)?;
        context.append(output, keyword)
    }
}

/// Authored before/after components of a `pause` or `rest` shorthand.
///
/// The optional second component remains distinct from an explicitly equal
/// value. Omission applies the first component to both longhands; the property
/// schema supplies their pause/rest identities and source occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpeechBreakPair {
    before: CssSpeechBreak,
    after: Option<CssSpeechBreak>,
}

impl CssSpeechBreakPair {
    /// Composes checked components, rejecting original recovered time closure
    /// while retaining whether the second component was authored.
    pub fn try_new(
        before: CssSpeechBreak,
        after: Option<CssSpeechBreak>,
    ) -> Result<Self, CssNumericConstructionError> {
        before.ensure_closed()?;
        if let Some(after) = &after {
            after.ensure_closed()?;
        }
        Ok(Self { before, after })
    }

    pub(crate) const fn from_parser(before: CssSpeechBreak, after: Option<CssSpeechBreak>) -> Self {
        Self { before, after }
    }

    #[must_use]
    pub const fn before(&self) -> &CssSpeechBreak {
        &self.before
    }

    /// Borrows the optional explicitly authored second component.
    #[must_use]
    pub const fn authored_after(&self) -> Option<&CssSpeechBreak> {
        self.after.as_ref()
    }

    /// Borrows the after value, sharing the first when the second was omitted.
    #[must_use]
    pub const fn after(&self) -> &CssSpeechBreak {
        match &self.after {
            Some(value) => value,
            None => &self.before,
        }
    }

    /// Serializes in grammar order, omitting an equal second component as CSSOM
    /// requires, without discarding its authored presence from this model.
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Shares one cumulative budget across the pair and both authored children.
    /// An omitted effective-after value incurs no duplicate input traversal.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut context = SpecifiedSerializationContext::new(limits);
        context.charge_input(1)?;
        context.charge_projection(1)?;
        let mut output = String::new();
        self.before.append_specified(&mut context, &mut output)?;
        if let Some(after) = &self.after {
            let omit = after.specified_value_eq(&self.before);
            let previous = context.replace_output_suppression(omit);
            if !omit {
                context.append(&mut output, " ")?;
            }
            after.append_specified(&mut context, &mut output)?;
            context.replace_output_suppression(previous);
        }
        Ok(output)
    }
}
