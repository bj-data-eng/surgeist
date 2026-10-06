//! Intrinsic authored values from CSS Speech 1 §§7–12.
//!
//! These values retain author choices. Voice selection, pronunciation and the
//! contextual computed/used behavior of `speak: auto` belong downstream.
//!
//! The `speak` and `speak-as` properties apply to all elements, inherit, have no percentage values,
//! and use grammar order for specified serialization. Their initial values are
//! `auto` and `normal`. Speech 1's property tables label computed values as the
//! specified value; the prose additionally gives contextual rules for `auto`.

use crate::specified_serialization::SpecifiedSerializationContext;
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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::Never => "never",
            Self::Always => "always",
        })
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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
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
        writer.keyword(text)
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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.append_specified(context, output)?;
        Ok(())
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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        self.append_to_rule_writer_with_comparison(
            writer,
            &mut |after, before, _| Ok(after.specified_value_eq(before)),
            &mut |_| {},
        )
    }

    pub(crate) fn append_to_rule_writer_with_comparison(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
        equal: &mut impl FnMut(
            &CssSpeechBreak,
            &CssSpeechBreak,
            &mut SpecifiedSerializationContext,
        ) -> Result<bool, crate::CssSpecifiedValueSerializationError>,
        before_value: &mut impl FnMut(usize),
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        before_value(0);
        let context = &mut writer.context;
        context.charge_input(1)?;
        context.charge_projection(1)?;
        let output = &mut writer.css;
        self.before.append_specified(context, output)?;
        if let Some(after) = &self.after {
            before_value(1);
            let omit = equal(after, &self.before, context)?;
            let suppress = context.output_suppressed() || omit;
            let previous = context.replace_output_suppression(suppress);
            let result = (|| {
                if !omit {
                    context.append(output, " ")?;
                }
                after.append_specified(context, output)
            })();
            context.replace_output_suppression(previous);
            result?;
        }
        Ok(())
    }
}

/// An exact signed ordinary Speech 1 decibel dimension, retaining its origin.
///
/// The decoded unit must be `dB` (ASCII case insensitive). Coefficients remain
/// finite authored decimals even when outside binary floating-point range.
/// This terminal does not admit or evaluate decibel calculations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssDecibelLiteral {
    component: Box<crate::CssComponentValue>,
}

impl CssDecibelLiteral {
    /// Constructs an exact coefficient with the Speech 1 `dB` unit.
    pub fn try_new(number: &str) -> Result<Self, crate::CssComponentValueError> {
        Self::try_from_component(crate::CssComponentValue::try_dimension(number, "dB")?)
    }

    /// Requires one ordinary dimension token with the decoded `dB` unit.
    pub fn try_from_component(
        component: crate::CssComponentValue,
    ) -> Result<Self, crate::CssComponentValueError> {
        if !matches!(component.view(),
            crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Dimension { unit, .. })
                if unit.eq_ignore_ascii_case("dB"))
        {
            return Err(crate::CssComponentValueError::new(
                crate::CssComponentValueErrorKind::InvalidToken,
                component.origin().clone(),
            ));
        }
        Ok(Self {
            component: Box::new(component),
        })
    }

    /// Borrows the original coefficient spelling without conversion.
    #[must_use]
    pub fn numeric(&self) -> crate::CssNumericTokenRef<'_> {
        let crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Dimension {
            number, ..
        }) = self.component.view()
        else {
            unreachable!("checked decibel dimension")
        };
        number
    }

    #[must_use]
    pub const fn component(&self) -> &crate::CssComponentValue {
        &self.component
    }

    #[must_use]
    pub const fn origin(&self) -> &crate::CssValueOrigin {
        self.component.origin()
    }

    fn is_zero(&self) -> bool {
        crate::exact_decimal::LexicalDecimal::new(self.numeric().representation()).len == 0
    }

    fn equivalent(&self, other: &Self) -> bool {
        crate::exact_decimal::LexicalDecimal::new(self.numeric().representation()).value_eq(
            &crate::exact_decimal::LexicalDecimal::new(other.numeric().representation()),
        )
    }

    /// Formats the retained coefficient with shared specified precision and
    /// the canonical lowercase `db` unit; authored spelling remains unchanged.
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges one terminal under the shared input, projection and byte limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }

    fn append_specified(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        if writer.context.output_suppressed() {
            return Ok(());
        }
        let text = crate::specified_serialization::format_coefficient(
            self.numeric().representation(),
            0,
            "db",
            writer.context.remaining_bytes(),
        )?;
        writer.append(&text)
    }
}

/// An authored auditory resource and its optional relative decibel offset.
///
/// Omission remains distinct from an explicit zero offset. Resource loading,
/// alternative cues and the computed relationship to voice-volume are downstream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAudioCue {
    url: crate::CssUrl,
    decibel: Option<CssDecibelLiteral>,
}

impl CssAudioCue {
    /// Couples checked providers, rejecting recovered URL modifier arguments.
    /// Original root closure belongs to declaration components, as for all URLs.
    pub fn try_new(
        url: crate::CssUrl,
        decibel: Option<CssDecibelLiteral>,
    ) -> Result<Self, crate::CssComponentValueError> {
        let value = Self { url, decibel };
        value.ensure_closed()?;
        Ok(value)
    }

    pub(crate) const fn from_parser(
        url: crate::CssUrl,
        decibel: Option<CssDecibelLiteral>,
    ) -> Self {
        Self { url, decibel }
    }

    #[must_use]
    pub const fn url(&self) -> &crate::CssUrl {
        &self.url
    }

    /// Borrows the explicitly authored offset; omission does not insert 0dB.
    #[must_use]
    pub const fn decibel(&self) -> Option<&CssDecibelLiteral> {
        self.decibel.as_ref()
    }

    fn ensure_closed(&self) -> Result<(), crate::CssComponentValueError> {
        for modifier in self.url.modifiers() {
            if let crate::CssUrlModifier::Function(function) = modifier
                && let Some(origin) = function.argument_components().first_implicit_origin()
            {
                return Err(crate::CssComponentValueError::new(
                    crate::CssComponentValueErrorKind::InvalidFunction,
                    origin.clone(),
                ));
            }
        }
        Ok(())
    }

    fn equivalent(&self, other: &Self) -> bool {
        self.url == other.url
            && match (&self.decibel, &other.decibel) {
                (None, None) => true,
                (Some(left), Some(right)) => left.equivalent(right),
                (Some(value), None) | (None, Some(value)) => value.is_zero(),
            }
    }

    /// Serializes the shared URL followed by a nonzero optional offset.
    /// CSSOM omission of the implied 0dB leaves the authored field unchanged.
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Shares the cumulative budget with the URL and the optional offset.
    /// Explicit zero still incurs its terminal visit even when omitted.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }

    fn append_specified(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        self.url.append_specified(writer)?;
        if let Some(decibel) = &self.decibel {
            if decibel.is_zero() {
                writer.without_output(|writer| decibel.append_specified(writer))?;
            } else {
                writer.append(" ")?;
                decibel.append_specified(writer)?;
            }
        }
        Ok(())
    }
}

/// One authored cue: no auditory icon, or a shared URL plus optional dB offset.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCue {
    None,
    Audio(CssAudioCue),
}

impl CssCue {
    fn ensure_closed(&self) -> Result<(), crate::CssComponentValueError> {
        match self {
            Self::None => Ok(()),
            Self::Audio(value) => value.ensure_closed(),
        }
    }

    fn equivalent(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Audio(left), Self::Audio(right)) => left.equivalent(right),
            _ => false,
        }
    }

    pub(crate) fn specified_inverse_eq(&self, other: &Self) -> bool {
        self.equivalent(other)
    }

    /// Serializes the specified cue without loading or resolving its resource.
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Uses the shared cumulative input, projection and emitted-byte limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }

    fn append_specified(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        match self {
            Self::None => {
                writer.context.charge_input(1)?;
                writer.context.charge_projection(1)?;
                writer.append("none")
            }
            Self::Audio(value) => value.append_specified(writer),
        }
    }
}

/// Ordered cue shorthand components, retaining whether after was authored.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCuePair {
    before: CssCue,
    after: Option<CssCue>,
}

impl CssCuePair {
    /// Composes cues, rejecting retained recovered URL modifier arguments.
    pub fn try_new(
        before: CssCue,
        after: Option<CssCue>,
    ) -> Result<Self, crate::CssComponentValueError> {
        before.ensure_closed()?;
        if let Some(after) = &after {
            after.ensure_closed()?;
        }
        Ok(Self { before, after })
    }

    pub(crate) const fn from_parser(before: CssCue, after: Option<CssCue>) -> Self {
        Self { before, after }
    }

    #[must_use]
    pub const fn before(&self) -> &CssCue {
        &self.before
    }

    #[must_use]
    pub const fn authored_after(&self) -> Option<&CssCue> {
        self.after.as_ref()
    }

    /// Shares before when the authored second component was omitted.
    #[must_use]
    pub const fn after(&self) -> &CssCue {
        match &self.after {
            Some(value) => value,
            None => &self.before,
        }
    }

    /// Serializes in grammar order, omitting an equivalent optional second cue.
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Explicitly redundant children still charge cumulative traversal work.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        writer.source_member(0, |writer| self.before.append_specified(writer))?;
        if let Some(after) = &self.after {
            writer.source_member(1, |writer| {
                if self.before.equivalent(after) {
                    writer.without_output(|writer| after.append_specified(writer))
                } else {
                    writer.append(" ")?;
                    after.append_specified(writer)
                }
            })?;
        }
        Ok(())
    }
}

/// Symbolic vocal stress; acoustic realization belongs to a speech renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceStress {
    Normal,
    Strong,
    Moderate,
    None,
    Reduced,
}
impl CssVoiceStress {
    /// Serializes the specified keyword.
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Serializes under the shared cumulative resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        writer.keyword(match self {
            Self::Normal => "normal",
            Self::Strong => "strong",
            Self::Moderate => "moderate",
            Self::None => "none",
            Self::Reduced => "reduced",
        })
    }
}

/// An authored generic voice age.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceAge {
    Child,
    Young,
    Old,
}
impl CssVoiceAge {
    pub(crate) fn keyword(self) -> &'static str {
        match self {
            Self::Child => "child",
            Self::Young => "young",
            Self::Old => "old",
        }
    }
}
/// An authored generic voice gender.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceGender {
    Male,
    Female,
    Neutral,
}
impl CssVoiceGender {
    pub(crate) fn keyword(self) -> &'static str {
        match self {
            Self::Male => "male",
            Self::Female => "female",
            Self::Neutral => "neutral",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum VoiceName {
    Quoted(String),
    Identifiers(Vec<crate::CssIdent>),
}
/// A checked voice name retaining its quoted or identifier-sequence form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssVoiceFamilyName {
    name: VoiceName,
}
/// Borrowed authored voice-name representation.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssVoiceFamilyNameRef<'a> {
    Quoted(&'a str),
    Identifiers(&'a [crate::CssIdent]),
}
impl CssVoiceFamilyName {
    /// Constructs a decoded quoted string, including the empty string.
    pub fn try_quoted(value: impl Into<String>) -> Result<Self, crate::CssComponentValueError> {
        let value = value.into();
        crate::CssComponentValue::try_string(value.clone())?;
        Ok(Self {
            name: VoiceName::Quoted(value),
        })
    }
    /// Constructs a nonempty sequence of decoded identifiers. Speech keywords,
    /// CSS-wide keywords and `default` require a quoted name.
    #[must_use]
    pub fn try_identifiers(identifiers: Vec<crate::CssIdent>) -> Option<Self> {
        if identifiers.is_empty()
            || identifiers
                .iter()
                .any(|ident| reserved_voice_identifier(ident.as_str()))
        {
            return None;
        }
        Some(Self {
            name: VoiceName::Identifiers(identifiers),
        })
    }
    /// Borrows the retained authored form without joining identifiers.
    #[must_use]
    pub fn view(&self) -> CssVoiceFamilyNameRef<'_> {
        match &self.name {
            VoiceName::Quoted(value) => CssVoiceFamilyNameRef::Quoted(value),
            VoiceName::Identifiers(values) => CssVoiceFamilyNameRef::Identifiers(values),
        }
    }
}
pub(crate) fn reserved_voice_identifier(value: &str) -> bool {
    [
        "male",
        "female",
        "neutral",
        "preserve",
        "default",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
    ]
    .iter()
    .any(|keyword| value.eq_ignore_ascii_case(keyword))
}

/// Checked `[<age>? <gender> <integer>?]`; a variant is exact and positive
/// when ordinary, or retained integer math with deferred range processing.
#[derive(Clone, Debug, PartialEq)]
pub struct CssGenericVoice {
    age: Option<CssVoiceAge>,
    gender: CssVoiceGender,
    variant: Option<crate::CssPositiveIntegerValue>,
}
impl CssGenericVoice {
    /// Constructs a generic voice without selecting an installed voice.
    /// Bare calculation literals also pass exact positivity validation.
    pub fn try_new(
        age: Option<CssVoiceAge>,
        gender: CssVoiceGender,
        variant: Option<crate::CssPositiveIntegerValue>,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(crate::CssPositiveIntegerValue::Calculation(value)) = &variant
            && let Some(origin) = value.components().first_implicit_origin()
        {
            return Err(CssNumericConstructionError::at_origin(
                crate::CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        Self::from_parser(age, gender, variant)
    }
    pub(crate) fn from_parser(
        age: Option<CssVoiceAge>,
        gender: CssVoiceGender,
        mut variant: Option<crate::CssPositiveIntegerValue>,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(crate::CssPositiveIntegerValue::Calculation(value)) = &variant {
            let root = crate::specified_numeric::significant_root(value.components())?;
            if matches!(root.view(), crate::CssComponentValueRef::Token(_)) {
                let integer =
                    crate::CssIntegerLiteral::try_from_component(root.clone()).map_err(|_| {
                        CssNumericConstructionError::at(
                            crate::CssNumericConstructionErrorKind::RootDomainMismatch,
                            Some(root),
                        )
                    })?;
                let positive =
                    crate::CssPositiveIntegerLiteral::try_new(integer).ok_or_else(|| {
                        CssNumericConstructionError::at(
                            crate::CssNumericConstructionErrorKind::OutOfRange,
                            Some(root),
                        )
                    })?;
                variant = Some(crate::CssPositiveIntegerValue::Literal(positive));
            }
        }
        Ok(Self {
            age,
            gender,
            variant,
        })
    }
    #[must_use]
    pub const fn age(&self) -> Option<CssVoiceAge> {
        self.age
    }
    #[must_use]
    pub const fn gender(&self) -> CssVoiceGender {
        self.gender
    }
    /// `None` retains omission of the variant, independently of variant `1`.
    #[must_use]
    pub const fn variant(&self) -> Option<&crate::CssPositiveIntegerValue> {
        self.variant.as_ref()
    }
}
/// One authored entry in a prioritized voice list.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceFamilyEntry {
    Name(CssVoiceFamilyName),
    Generic(CssGenericVoice),
}

/// A nonempty prioritized list of authored voices.
#[derive(Clone, Debug, PartialEq)]
pub struct CssVoiceFamilyList {
    entries: Vec<CssVoiceFamilyEntry>,
}
impl CssVoiceFamilyList {
    /// Rejects an empty list or an originally recovered variant calculation.
    #[must_use]
    pub fn try_new(entries: Vec<CssVoiceFamilyEntry>) -> Option<Self> {
        if entries.iter().any(|entry| matches!(entry,
            CssVoiceFamilyEntry::Generic(value)
                if matches!(value.variant(), Some(crate::CssPositiveIntegerValue::Calculation(value))
                    if value.components().first_implicit_origin().is_some()))) {
            return None;
        }
        Self::from_parser(entries)
    }
    pub(crate) fn from_parser(entries: Vec<CssVoiceFamilyEntry>) -> Option<Self> {
        (!entries.is_empty()).then_some(Self { entries })
    }
    #[must_use]
    pub fn entries(&self) -> &[CssVoiceFamilyEntry] {
        &self.entries
    }
}
/// A prioritized voice list or the distinct language-preservation alternative.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceFamily {
    Voices(CssVoiceFamilyList),
    Preserve,
}
impl CssVoiceFamily {
    /// Serializes specified author choices without contextual voice selection.
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Uses shared identifier/string escaping and cumulative integer projection.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        use crate::specified_rule_serialization::SpecifiedRuleWriter;
        fn leaf(
            writer: &mut SpecifiedRuleWriter,
        ) -> Result<(), CssSpecifiedValueSerializationError> {
            writer.context.charge_input(1)?;
            writer.context.charge_projection(1)
        }

        leaf(writer)?;
        match self {
            Self::Preserve => writer.append("preserve")?,
            Self::Voices(list) => {
                for (index, entry) in list.entries.iter().enumerate() {
                    if index > 0 {
                        writer.append(", ")?;
                    }
                    leaf(writer)?;
                    match entry {
                        CssVoiceFamilyEntry::Name(name) => match name.view() {
                            CssVoiceFamilyNameRef::Quoted(value) => {
                                leaf(writer)?;
                                writer.append_string(value)?;
                            }
                            CssVoiceFamilyNameRef::Identifiers(values) => {
                                for (index, value) in values.iter().enumerate() {
                                    if index > 0 {
                                        writer.append(" ")?;
                                    }
                                    leaf(writer)?;
                                    writer.append_identifier(value.as_str())?;
                                }
                            }
                        },
                        CssVoiceFamilyEntry::Generic(value) => {
                            if let Some(age) = value.age {
                                leaf(writer)?;
                                writer.append(age.keyword())?;
                                writer.append(" ")?;
                            }
                            leaf(writer)?;
                            writer.append(value.gender.keyword())?;
                            if let Some(variant) = &value.variant {
                                writer.append(" ")?;
                                match variant {
                                    crate::CssPositiveIntegerValue::Literal(value) => value
                                        .integer()
                                        .append_specified(&mut writer.context, &mut writer.css)?,
                                    crate::CssPositiveIntegerValue::Calculation(value) => value
                                        .serialize_specified_into(
                                            &mut writer.context,
                                            &mut writer.css,
                                        )?,
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// The authored speech-subtree duration; subtree precedence is resolved downstream.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceDuration {
    Auto,
    Time(CssDuration),
}
impl CssVoiceDuration {
    /// Checks ordinary nonnegativity and original calculation closure.
    pub fn try_time(time: CssTimeValue) -> Result<Self, CssNumericConstructionError> {
        CssDuration::try_new(time).map(Self::Time)
    }
    /// Serializes the specified value without applying subtree precedence.
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Uses the existing duration provider under one cumulative resource policy.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        match self {
            Self::Auto => writer.keyword("auto"),
            Self::Time(value) => {
                value.append_specified(&mut writer.context, &mut writer.css)?;
                Ok(())
            }
        }
    }
}

/// A voice-dependent symbolic pitch or pitch-range level.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceLevel {
    XLow,
    Low,
    Medium,
    High,
    XHigh,
}
impl CssVoiceLevel {
    pub(crate) fn keyword(self) -> &'static str {
        match self {
            Self::XLow => "x-low",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::XHigh => "x-high",
        }
    }
}

/// The Speech-local ordinary signed `st` dimension, retaining exact spelling and origin.
/// No semitone calculation type or voice-dependent conversion is introduced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSemitoneLiteral {
    component: Box<crate::CssComponentValue>,
}
impl CssSemitoneLiteral {
    pub fn try_new(number: &str) -> Result<Self, crate::CssComponentValueError> {
        Self::try_from_component(crate::CssComponentValue::try_dimension(number, "st")?)
    }
    pub fn try_from_component(
        component: crate::CssComponentValue,
    ) -> Result<Self, crate::CssComponentValueError> {
        if !matches!(component.view(), crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Dimension { unit, .. }) if unit.eq_ignore_ascii_case("st"))
        {
            return Err(crate::CssComponentValueError::new(
                crate::CssComponentValueErrorKind::InvalidToken,
                component.origin().clone(),
            ));
        }
        Ok(Self {
            component: Box::new(component),
        })
    }
    /// Borrows the exact authored coefficient without machine-float narrowing.
    #[must_use]
    pub fn numeric(&self) -> crate::CssNumericTokenRef<'_> {
        let crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Dimension {
            number, ..
        }) = self.component.view()
        else {
            unreachable!("checked semitone dimension")
        };
        number
    }
    #[must_use]
    pub const fn component(&self) -> &crate::CssComponentValue {
        &self.component
    }
    #[must_use]
    pub const fn origin(&self) -> &crate::CssValueOrigin {
        self.component.origin()
    }
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
    fn append_specified(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        if writer.context.output_suppressed() {
            return Ok(());
        }
        let text = crate::specified_serialization::format_coefficient(
            self.numeric().representation(),
            0,
            "st",
            writer.context.remaining_bytes(),
        )?;
        writer.append(&text)
    }
}

/// One signed relative offset, without applying it to an inherited voice frequency.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceOffset {
    Frequency(crate::CssFrequencyValue),
    Semitones(CssSemitoneLiteral),
    Percentage(crate::CssSpecifiedPercentage),
    /// Typed math permits frequency and percentage against an unresolved frequency basis.
    Calculation(crate::CssFrequencyPercentageCalculation),
}
impl CssVoiceOffset {
    /// Requires original closure and normalizes an ordinary calculation root back
    /// into the corresponding frequency/percentage literal owner.
    pub fn try_from_calculation(
        value: crate::CssFrequencyPercentageCalculation,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(origin) = value.components().first_implicit_origin() {
            return Err(CssNumericConstructionError::at_origin(
                crate::CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        let root = crate::specified_numeric::significant_root(value.components())?;
        if matches!(root.view(), crate::CssComponentValueRef::Token(_)) {
            return Self::from_literal_component(root.clone());
        }
        Ok(Self::Calculation(value))
    }
    fn from_literal_component(
        component: crate::CssComponentValue,
    ) -> Result<Self, CssNumericConstructionError> {
        match component.view() {
            crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Percentage(_)) => {
                crate::CssSpecifiedPercentage::try_from_component(component).map(Self::Percentage)
            }
            crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Dimension {
                unit, ..
            }) if unit.eq_ignore_ascii_case("st") => {
                CssSemitoneLiteral::try_from_component(component)
                    .map(Self::Semitones)
                    .map_err(CssNumericConstructionError::component)
            }
            _ => crate::CssFrequencyLiteral::try_from_component(component)
                .map(crate::CssFrequencyValue::from_literal)
                .map(Self::Frequency)
                .map_err(CssNumericConstructionError::component),
        }
    }
    pub(crate) fn from_parser_component(
        component: crate::CssComponentValue,
        context: &crate::numeric::NumericInputContext<'_>,
    ) -> Result<Self, CssNumericConstructionError> {
        if matches!(component.view(), crate::CssComponentValueRef::Token(_)) {
            return Self::from_literal_component(component);
        }
        let values = crate::CssComponentValues::try_new(vec![component])
            .map_err(CssNumericConstructionError::component)?;
        let expression =
            context.admit(values, crate::numeric::CalculationRoot::FrequencyPercentage)?;
        Ok(Self::Calculation(
            crate::CssFrequencyPercentageCalculation::from_expression(expression),
        ))
    }
    fn ensure_closed(&self) -> Result<(), CssNumericConstructionError> {
        let components = match self {
            Self::Frequency(value) => return value.ensure_closed(),
            Self::Semitones(_) => None,
            Self::Percentage(value) => value
                .calculation()
                .map(crate::CssPercentageCalculation::components),
            Self::Calculation(value) => Some(value.components()),
        };
        if let Some(origin) = components.and_then(crate::CssComponentValues::first_implicit_origin)
        {
            return Err(CssNumericConstructionError::at_origin(
                crate::CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        Ok(())
    }
    fn append_specified(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        match self {
            Self::Frequency(value) => value.append_specified(&mut writer.context, &mut writer.css),
            Self::Semitones(value) => value.append_specified(writer),
            Self::Percentage(value) => value.append_specified(&mut writer.context, &mut writer.css),
            Self::Calculation(value) => crate::numeric::project_specified_into(
                &value.expression,
                &mut writer.context,
                &mut writer.css,
            )
            .map(|_| ()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum VoicePitchRangeState {
    Absolute(crate::CssFrequencyValue),
    Relative {
        level: Option<CssVoiceLevel>,
        offset: Option<CssVoiceOffset>,
    },
}
/// Checked shared authored grammar for `voice-pitch` and `voice-range`.
/// Absolute frequency cannot carry a level; relative composition is nonempty.
#[derive(Clone, Debug, PartialEq)]
pub struct CssVoicePitchRange {
    state: VoicePitchRangeState,
}
impl CssVoicePitchRange {
    /// Requires strictly positive ordinary absolute frequency. Typed frequency
    /// calculations retain their authored range for downstream processing.
    pub fn try_absolute(
        value: crate::CssFrequencyValue,
    ) -> Result<Self, CssNumericConstructionError> {
        value.ensure_closed()?;
        Self::from_parser_absolute(value)
    }
    pub(crate) fn from_parser_absolute(
        value: crate::CssFrequencyValue,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(literal) = value.literal() {
            let coefficient =
                crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation());
            if coefficient.negative || coefficient.len == 0 {
                return Err(CssNumericConstructionError::at_origin(
                    crate::CssNumericConstructionErrorKind::OutOfRange,
                    literal.origin().clone(),
                ));
            }
        }
        Ok(Self {
            state: VoicePitchRangeState::Absolute(value),
        })
    }
    /// Checks a nonempty relative composition and original calculation closure.
    pub fn try_relative(
        level: Option<CssVoiceLevel>,
        offset: Option<CssVoiceOffset>,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(offset) = &offset {
            offset.ensure_closed()?;
        }
        Self::from_parser_relative(level, offset)
    }
    pub(crate) fn from_parser_relative(
        level: Option<CssVoiceLevel>,
        offset: Option<CssVoiceOffset>,
    ) -> Result<Self, CssNumericConstructionError> {
        if level.is_none() && offset.is_none() {
            return Err(CssNumericConstructionError::at(
                crate::CssNumericConstructionErrorKind::EmptyValue,
                None,
            ));
        }
        Ok(Self {
            state: VoicePitchRangeState::Relative { level, offset },
        })
    }
    /// Constructs a relative symbolic level without resolving its frequency.
    #[must_use]
    pub const fn level_only(level: CssVoiceLevel) -> Self {
        Self {
            state: VoicePitchRangeState::Relative {
                level: Some(level),
                offset: None,
            },
        }
    }
    #[must_use]
    pub const fn absolute_frequency(&self) -> Option<&crate::CssFrequencyValue> {
        match &self.state {
            VoicePitchRangeState::Absolute(value) => Some(value),
            VoicePitchRangeState::Relative { .. } => None,
        }
    }
    #[must_use]
    pub const fn level(&self) -> Option<CssVoiceLevel> {
        match &self.state {
            VoicePitchRangeState::Relative { level, .. } => *level,
            VoicePitchRangeState::Absolute(_) => None,
        }
    }
    #[must_use]
    pub const fn offset(&self) -> Option<&CssVoiceOffset> {
        match &self.state {
            VoicePitchRangeState::Relative { offset, .. } => offset.as_ref(),
            VoicePitchRangeState::Absolute(_) => None,
        }
    }
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Emits frequency before `absolute`, or level before offset, under one
    /// cumulative visit and output budget. It does not compute a voice frequency.
    /// Ordinary frequency output shares its six-place precision owner. If a
    /// positive absolute literal rounds to zero, returns `UnrepresentableValue`
    /// atomically rather than emitting invalid grammar. Exact input remains
    /// retained; this does not perform computed voice clamping.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        match &self.state {
            VoicePitchRangeState::Absolute(value) => {
                let start = writer.css.len();
                value.append_specified(&mut writer.context, &mut writer.css)?;
                if let Some(literal) = value.literal() {
                    let coefficient = writer.css[start..]
                        .strip_suffix(crate::frequency::suffix(literal.unit()))
                        .expect("frequency provider emits its selected canonical unit");
                    if crate::exact_decimal::LexicalDecimal::new(coefficient).len == 0 {
                        return Err(CssSpecifiedValueSerializationError::new(
                            crate::CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
                        ));
                    }
                }
                writer.context.charge_input(1)?;
                writer.context.charge_projection(1)?;
                writer.append(" absolute")?;
            }
            VoicePitchRangeState::Relative { level, offset } => {
                if let Some(level) = level {
                    writer.context.charge_input(1)?;
                    writer.context.charge_projection(1)?;
                    writer.append(level.keyword())?;
                }
                if let Some(offset) = offset {
                    if level.is_some() {
                        writer.append(" ")?;
                    }
                    offset.append_specified(writer)?;
                }
            }
        }
        Ok(())
    }
}

/// A symbolic voice-rate keyword, without a words-per-minute mapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceRateKeyword {
    Normal,
    XSlow,
    Slow,
    Medium,
    Fast,
    XFast,
}
impl CssVoiceRateKeyword {
    pub(crate) fn keyword(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::XSlow => "x-slow",
            Self::Slow => "slow",
            Self::Medium => "medium",
            Self::Fast => "fast",
            Self::XFast => "x-fast",
        }
    }
}
/// Checked nonempty rate keyword/percentage composition; a retained 100% stays
/// distinct from omission even when canonical output omits a neutral modifier.
#[derive(Clone, Debug, PartialEq)]
pub struct CssVoiceRate {
    keyword: Option<CssVoiceRateKeyword>,
    percentage: Option<crate::CssSpecifiedNonNegativePercentage>,
}
impl CssVoiceRate {
    pub fn try_new(
        keyword: Option<CssVoiceRateKeyword>,
        percentage: Option<crate::CssSpecifiedNonNegativePercentage>,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(value) = percentage
            .as_ref()
            .and_then(crate::CssSpecifiedNonNegativePercentage::calculation)
            && let Some(origin) = value.components().first_implicit_origin()
        {
            return Err(CssNumericConstructionError::at_origin(
                crate::CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        Self::from_parser(keyword, percentage)
    }
    pub(crate) fn from_parser(
        keyword: Option<CssVoiceRateKeyword>,
        percentage: Option<crate::CssSpecifiedNonNegativePercentage>,
    ) -> Result<Self, CssNumericConstructionError> {
        if keyword.is_none() && percentage.is_none() {
            return Err(CssNumericConstructionError::at(
                crate::CssNumericConstructionErrorKind::EmptyValue,
                None,
            ));
        }
        Ok(Self {
            keyword,
            percentage,
        })
    }
    #[must_use]
    pub const fn keyword_only(keyword: CssVoiceRateKeyword) -> Self {
        Self {
            keyword: Some(keyword),
            percentage: None,
        }
    }
    #[must_use]
    pub const fn keyword(&self) -> Option<CssVoiceRateKeyword> {
        self.keyword
    }
    #[must_use]
    pub const fn percentage(&self) -> Option<&crate::CssSpecifiedNonNegativePercentage> {
        self.percentage.as_ref()
    }
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Emits grammar order and omits only an exact ordinary neutral 100%
    /// modifier beside an explicit keyword. The suppressed child is still visited.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        if let Some(keyword) = self.keyword {
            writer.context.charge_input(1)?;
            writer.context.charge_projection(1)?;
            writer.append(keyword.keyword())?;
        }
        if let Some(percentage) = &self.percentage {
            let neutral = percentage.literal_component().is_some_and(|component| {
                matches!(component.view(), crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Percentage(number))
                    if crate::exact_decimal::LexicalDecimal::new(number.representation()).value_eq(&crate::exact_decimal::LexicalDecimal::new("100")))
            });
            if self.keyword.is_some() && neutral {
                writer.without_output(|writer| {
                    percentage.append_specified(&mut writer.context, &mut writer.css)
                })?;
            } else {
                if self.keyword.is_some() {
                    writer.append(" ")?;
                }
                percentage.append_specified(&mut writer.context, &mut writer.css)?;
            }
        }
        Ok(())
    }
}

/// An authored spatial-balance keyword; numerical resolution belongs downstream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceBalanceKeyword {
    Left,
    Center,
    Right,
    Leftwards,
    Rightwards,
}
impl CssVoiceBalanceKeyword {
    const fn keyword(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
            Self::Leftwards => "leftwards",
            Self::Rightwards => "rightwards",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum VoiceBalanceState {
    Keyword(CssVoiceBalanceKeyword),
    Number(crate::CssSpecifiedNumber),
}
/// Exact authored balance keyword or unrestricted pure Number value.
/// Specified admission does not clamp numbers or resolve inherited adjustments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssVoiceBalance {
    state: VoiceBalanceState,
}
impl CssVoiceBalance {
    #[must_use]
    pub const fn from_keyword(keyword: CssVoiceBalanceKeyword) -> Self {
        Self {
            state: VoiceBalanceState::Keyword(keyword),
        }
    }
    /// Retains pure Number syntax, rejecting original recovered math closure.
    /// Percentage values cannot cross this Number-only construction boundary.
    /// ```compile_fail
    /// use surgeist_css::{CssComponentValue, CssSpecifiedPercentage, CssVoiceBalance};
    /// let percentage = CssSpecifiedPercentage::try_from_component(
    ///     CssComponentValue::try_percentage("50").unwrap()).unwrap();
    /// CssVoiceBalance::try_number(percentage);
    /// ```
    pub fn try_number(
        number: crate::CssSpecifiedNumber,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(value) = number.calculation()
            && let Some(origin) = value.components().first_implicit_origin()
        {
            return Err(CssNumericConstructionError::at_origin(
                crate::CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        Ok(Self::from_parser_number(number))
    }
    pub(crate) const fn from_parser_number(number: crate::CssSpecifiedNumber) -> Self {
        Self {
            state: VoiceBalanceState::Number(number),
        }
    }
    #[must_use]
    pub const fn keyword(&self) -> Option<CssVoiceBalanceKeyword> {
        match &self.state {
            VoiceBalanceState::Keyword(value) => Some(*value),
            VoiceBalanceState::Number(_) => None,
        }
    }
    #[must_use]
    pub const fn number(&self) -> Option<&crate::CssSpecifiedNumber> {
        match &self.state {
            VoiceBalanceState::Number(value) => Some(value),
            VoiceBalanceState::Keyword(_) => None,
        }
    }
    /// Formats through shared Number precision without Speech clamping.
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Uses the keyword or pure Number provider's cumulative resource policy.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        match &self.state {
            VoiceBalanceState::Keyword(value) => writer.keyword(value.keyword()),
            VoiceBalanceState::Number(value) => value.append_to_rule_writer(writer),
        }
    }
}

/// A symbolic user-calibrated volume level, excluding the distinct silent state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceVolumeLevel {
    XSoft,
    Soft,
    Medium,
    Loud,
    XLoud,
}
impl CssVoiceVolumeLevel {
    const fn keyword(self) -> &'static str {
        match self {
            Self::XSoft => "x-soft",
            Self::Soft => "soft",
            Self::Medium => "medium",
            Self::Loud => "loud",
            Self::XLoud => "x-loud",
        }
    }
}
/// A nonempty authored volume grammar, retaining absent level and offset.
/// These alternatives exclude empty compositions and every companion to silent.
/// Calibration, inheritance and synthesis remain downstream.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceVolume {
    Silent,
    Level {
        level: CssVoiceVolumeLevel,
        decibel: Option<CssDecibelLiteral>,
    },
    Offset(CssDecibelLiteral),
}
impl CssVoiceVolume {
    /// Serializes level before offset, preserving standalone offset meaning.
    pub fn serialize_specified(&self) -> SerializationResult {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Visits an explicitly authored offset even when an exact zero companion
    /// is omitted. Standalone zero and rounded tiny nonzero offsets are emitted.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        match self {
            Self::Silent => writer.keyword("silent"),
            Self::Offset(value) => value.append_to_rule_writer(writer),
            Self::Level { level, decibel } => {
                writer.context.charge_input(1)?;
                writer.context.charge_projection(1)?;
                writer.context.charge_input(1)?;
                writer.context.charge_projection(1)?;
                writer.append(level.keyword())?;
                if let Some(value) = decibel {
                    if value.is_zero() {
                        writer.without_output(|writer| value.append_specified(writer))?;
                    } else {
                        writer.append(" ")?;
                        value.append_specified(writer)?;
                    }
                }
                Ok(())
            }
        }
    }
}
