//! Intrinsic authored keyword values from CSS Speech 1 §§7.1–7.2.
//!
//! These values retain author choices. Voice selection, pronunciation and the
//! contextual computed/used behavior of `speak: auto` belong downstream.
//!
//! Both properties apply to all elements, inherit, have no percentage values,
//! and use grammar order for specified serialization. Their initial values are
//! `auto` and `normal`. Speech 1's property tables label computed values as the
//! specified value; the prose additionally gives contextual rules for `auto`.

use crate::specified_serialization::serialize_keyword_sequence;
use crate::{CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits};

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
