//! Intrinsic authored values from CSS Speech 1 §§7–12.
//!
//! These values retain author choices. Voice selection, pronunciation and the
//! contextual computed/used behavior of `speak: auto` belong downstream.
//!
//! The `speak` and `speak-as` properties apply to all elements, inherit, have no percentage values,
//! and use grammar order for specified serialization. Their initial values are
//! `auto` and `normal`. Speech 1's property tables label computed values as the
//! specified value; the prose additionally gives contextual rules for `auto`.

use crate::numeric::{CssNonNegative, CssNumeric, CssNumericConstructionError, CssTime};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_serialization::SpecifiedSerializationContext;
use crate::values::common::CssSidePair;
use crate::values::common::common_values::side_element;

/// Whether an element is authored to participate in speech rendering.
///
/// # Specified serialization
///
/// Serializes the specified keyword without resolving speech participation.
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

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssSpeak {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::Never => "never",
            Self::Always => "always",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssSpeak);

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
///
/// # Specified serialization
///
/// Serializes in grammar order: spell-out, digits, then punctuation.
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
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssSpeakAs {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
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
crate::serialization::specified_rule_serialization::specified_serialization!(CssSpeakAs);

/// An authored pause/rest strength whose absolute duration remains contextual.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSpeechBreakStrength {
    /// `x-weak`: the weakest prosodic break strength.
    XWeak,
    /// `weak`: a prosodic break no weaker than `x-weak`.
    Weak,
    /// `medium`: a prosodic break no weaker than `weak`.
    Medium,
    /// `strong`: a prosodic break no weaker than `medium`.
    Strong,
    /// `x-strong`: the strongest prosodic break strength.
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
/// Ordinary times are nonnegative under [`CssNumeric<CssTime, CssNonNegative>`]'s exact lexical rule;
/// calculations remain authored until computed range handling downstream.
/// `none` is preserved separately from an explicit zero time. This model does
/// not assign strength durations or execute pause collapse or additive rests.
///
/// # Specified serialization
///
/// Serializes a specified keyword or canonical time without speech execution.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSpeechBreak {
    /// `none`: no prosodic break is produced; the specification equates it with `0ms`, but it is kept distinct from an explicit zero time.
    None,
    /// A break strength keyword; the exact time is left to the speech processor.
    Strength(CssSpeechBreakStrength),
    /// A non-negative `<time>` giving the break as an absolute duration.
    Time(CssNumeric<CssTime, CssNonNegative>),
}

impl CssSpeechBreak {
    /// Checks the nonnegative range of an ordinary time and the closure of original math.
    pub fn try_time(time: CssNumeric<CssTime>) -> Result<Self, CssNumericConstructionError> {
        time.try_narrow_checked().map(Self::Time)
    }

    fn ensure_closed(&self) -> Result<(), CssNumericConstructionError> {
        match self {
            Self::Time(duration) => duration.ensure_closed(),
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

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSerializationError> {
        let keyword = match self {
            Self::None => "none",
            Self::Strength(strength) => strength.as_css(),
            Self::Time(duration) => return duration.append_specified(context, output),
        };
        context.charge_nodes(1)?;
        context.append(output, keyword)
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssSpeechBreak {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.append_specified(context, output)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssSpeechBreak);

// `pause` and `rest` take `<'pause-before'> <'pause-after'>?` (respectively `rest`). A second
// component that equals the first is omitted from the specified serialization, while the model
// keeps whether it was authored.
side_element!(
    CssSpeechBreak,
    same = |a, b, _context| Ok(a.specified_value_eq(b))
);

impl CssSidePair<CssSpeechBreak> {
    /// Composes checked components, rejecting original recovered time closure
    /// while retaining whether the second component was authored.
    ///
    /// # Errors
    ///
    /// Returns [`CssNumericConstructionError`] when a time component was closed implicitly.
    pub fn try_new(
        first: CssSpeechBreak,
        authored_second: Option<CssSpeechBreak>,
    ) -> Result<Self, CssNumericConstructionError> {
        first.ensure_closed()?;
        if let Some(second) = &authored_second {
            second.ensure_closed()?;
        }
        Ok(Self::from_parts(first, authored_second))
    }
}
crate::values::common::common_values::side_pair_serialization!(CssSpeechBreak);

/// An exact signed ordinary Speech 1 decibel dimension, retaining its origin.
///
/// The decoded unit must be `dB` (ASCII case insensitive). Coefficients remain
/// finite authored decimals even when outside binary floating-point range.
/// This terminal does not admit or evaluate decibel calculations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssDecibelLiteral {
    component: Box<crate::component::CssComponentValue>,
}

impl CssDecibelLiteral {
    /// Constructs an exact coefficient with the Speech 1 `dB` unit.
    pub fn try_new(number: &str) -> Result<Self, crate::component::CssComponentValueError> {
        Self::try_from_component(crate::component::CssComponentValue::try_dimension(
            number, "dB",
        )?)
    }

    /// Requires one ordinary dimension token with the decoded `dB` unit.
    pub fn try_from_component(
        component: crate::component::CssComponentValue,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        if !matches!(component.view(),
            crate::component::CssComponentValueRef::Token(crate::component::CssValueTokenRef::Dimension { unit, .. })
                if unit.eq_ignore_ascii_case("dB"))
        {
            return Err(crate::component::CssComponentValueError::new(
                crate::component::values::CssComponentValueErrorKind::InvalidToken,
                component.origin().clone(),
            ));
        }
        Ok(Self {
            component: Box::new(component),
        })
    }

    /// Borrows the original coefficient spelling without conversion.
    #[must_use]
    pub fn numeric(&self) -> crate::component::CssNumericTokenRef<'_> {
        let crate::component::CssComponentValueRef::Token(
            crate::component::CssValueTokenRef::Dimension { number, .. },
        ) = self.component.view()
        else {
            unreachable!("checked decibel dimension")
        };
        number
    }

    /// Returns the checked `dB` dimension component, including its original source.
    #[must_use]
    pub const fn component(&self) -> &crate::component::CssComponentValue {
        &self.component
    }

    /// Returns the provenance of the dimension component.
    #[must_use]
    pub const fn origin(&self) -> &crate::component::CssValueOrigin {
        self.component.origin()
    }

    fn is_zero(&self) -> bool {
        crate::serialization::exact_decimal::LexicalDecimal::new(self.numeric().representation())
            .is_zero()
    }

    fn equivalent(&self, other: &Self) -> bool {
        crate::serialization::exact_decimal::LexicalDecimal::new(self.numeric().representation())
            .value_eq(&crate::serialization::exact_decimal::LexicalDecimal::new(
                other.numeric().representation(),
            ))
    }

    fn append_specified(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        writer.node()?;
        if writer.context.output_suppressed() {
            return Ok(());
        }
        let text = crate::serialization::specified_serialization::format_coefficient(
            self.numeric().representation(),
            0,
            "db",
            writer.context.remaining_bytes(),
        )?;
        writer.append(&text)
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssDecibelLiteral {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssDecibelLiteral);

/// An authored auditory resource and its optional relative decibel offset.
///
/// Omission remains distinct from an explicit zero offset. Resource loading,
/// alternative cues and the computed relationship to voice-volume are downstream.
///
/// # Specified serialization
///
/// Serializes the shared URL followed by a nonzero optional offset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAudioCue {
    url: crate::values::common::CssUrl,
    decibel: Option<CssDecibelLiteral>,
}

impl CssAudioCue {
    /// Couples checked providers, rejecting recovered URL modifier arguments.
    /// Original root closure belongs to declaration components, as for all URLs.
    pub fn try_new(
        url: crate::values::common::CssUrl,
        decibel: Option<CssDecibelLiteral>,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        let value = Self { url, decibel };
        value.ensure_closed()?;
        Ok(value)
    }

    pub(crate) const fn from_parser(
        url: crate::values::common::CssUrl,
        decibel: Option<CssDecibelLiteral>,
    ) -> Self {
        Self { url, decibel }
    }

    /// Returns the shared authored URL of the cue.
    #[must_use]
    pub const fn url(&self) -> &crate::values::common::CssUrl {
        &self.url
    }

    /// Borrows the explicitly authored offset; omission does not insert 0dB.
    #[must_use]
    pub const fn decibel(&self) -> Option<&CssDecibelLiteral> {
        self.decibel.as_ref()
    }

    fn ensure_closed(&self) -> Result<(), crate::component::CssComponentValueError> {
        for modifier in self.url.modifiers() {
            if let crate::values::common::CssUrlModifier::Function(function) = modifier
                && let Some(origin) = function.argument_components().first_implicit_origin()
            {
                return Err(crate::component::CssComponentValueError::new(
                    crate::component::values::CssComponentValueErrorKind::InvalidFunction,
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

    fn append_specified(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        writer.node()?;
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
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssAudioCue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssAudioCue);

/// One authored cue: no auditory icon, or a shared URL plus optional dB offset.
///
/// # Specified serialization
///
/// Serializes the specified cue without loading or resolving its resource.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCue {
    /// `none`: no auditory icon.
    None,
    /// An auditory icon: a URL with an optional decibel offset.
    Audio(CssAudioCue),
}

impl CssCue {
    fn ensure_closed(&self) -> Result<(), crate::component::CssComponentValueError> {
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

    fn append_specified(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        match self {
            Self::None => {
                writer.node()?;
                writer.append("none")
            }
            Self::Audio(value) => value.append_specified(writer),
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssCue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssCue);

// `cue` takes `<'cue-before'> <'cue-after'>?`; an equivalent second cue is omitted from the
// specified serialization, while the model keeps whether it was authored.
side_element!(CssCue, same = |a, b, _context| Ok(a.equivalent(b)));

impl CssSidePair<CssCue> {
    /// Composes cues, rejecting retained recovered URL modifier arguments.
    ///
    /// # Errors
    ///
    /// Returns [`crate::component::CssComponentValueError`] when a cue retains a recovered argument.
    pub fn try_new(
        first: CssCue,
        authored_second: Option<CssCue>,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        first.ensure_closed()?;
        if let Some(second) = &authored_second {
            second.ensure_closed()?;
        }
        Ok(Self::from_parts(first, authored_second))
    }
}
crate::values::common::common_values::side_pair_serialization!(CssCue);

/// Symbolic vocal stress; acoustic realization belongs to a speech renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceStress {
    /// `normal`: the default emphasis produced by the speech synthesizer.
    Normal,
    /// `strong`: more emphasis than `normal`, and no weaker than `moderate`.
    Strong,
    /// `moderate`: more emphasis than `normal`.
    Moderate,
    /// `none`: prevents the synthesizer from emphasizing text it would normally emphasize.
    None,
    /// `reduced`: effectively the opposite of emphasizing a word.
    Reduced,
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssVoiceStress {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Normal => "normal",
            Self::Strong => "strong",
            Self::Moderate => "moderate",
            Self::None => "none",
            Self::Reduced => "reduced",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssVoiceStress);

/// An authored generic voice age.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceAge {
    /// `child`: the preferred age category is a child.
    Child,
    /// `young`: the preferred age category is young.
    Young,
    /// `old`: the preferred age category is old.
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
    /// `male`: the preferred gender is male.
    Male,
    /// `female`: the preferred gender is female.
    Female,
    /// `neutral`: the preferred gender is neutral.
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
    Identifiers(Vec<crate::values::common::CssIdent>),
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
    /// A quoted voice name, decoded; it may be empty.
    Quoted(&'a str),
    /// A name written as a sequence of identifiers, one decoded identifier per element.
    Identifiers(&'a [crate::values::common::CssIdent]),
}
impl CssVoiceFamilyName {
    /// Constructs a decoded quoted string, including the empty string.
    pub fn try_quoted(
        value: impl Into<String>,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        let value = value.into();
        crate::component::CssComponentValue::try_string(value.clone())?;
        Ok(Self {
            name: VoiceName::Quoted(value),
        })
    }
    /// Constructs a nonempty sequence of decoded identifiers. Speech keywords,
    /// CSS-wide keywords and `default` require a quoted name.
    #[must_use]
    pub fn try_identifiers(identifiers: Vec<crate::values::common::CssIdent>) -> Option<Self> {
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
    variant:
        Option<crate::numeric::CssNumeric<crate::numeric::CssInteger, crate::numeric::CssPositive>>,
}

impl CssGenericVoice {
    /// Constructs a generic voice without selecting an installed voice.
    /// The variant's own type checks positivity; recovered variant math is rejected.
    pub fn try_new(
        age: Option<CssVoiceAge>,
        gender: CssVoiceGender,
        variant: Option<
            crate::numeric::CssNumeric<crate::numeric::CssInteger, crate::numeric::CssPositive>,
        >,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(value) = &variant {
            value.ensure_closed()?;
        }
        Self::from_parser(age, gender, variant)
    }
    pub(crate) fn from_parser(
        age: Option<CssVoiceAge>,
        gender: CssVoiceGender,
        variant: Option<
            crate::numeric::CssNumeric<crate::numeric::CssInteger, crate::numeric::CssPositive>,
        >,
    ) -> Result<Self, CssNumericConstructionError> {
        Ok(Self {
            age,
            gender,
            variant,
        })
    }
    /// Returns the age keyword, or `None` when none was authored.
    #[must_use]
    pub const fn age(&self) -> Option<CssVoiceAge> {
        self.age
    }
    /// Returns the gender keyword.
    #[must_use]
    pub const fn gender(&self) -> CssVoiceGender {
        self.gender
    }
    /// `None` retains omission of the variant, independently of variant `1`.
    #[must_use]
    pub const fn variant(
        &self,
    ) -> Option<&crate::numeric::CssNumeric<crate::numeric::CssInteger, crate::numeric::CssPositive>>
    {
        self.variant.as_ref()
    }
}
/// One authored entry in a prioritized voice list.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceFamilyEntry {
    /// A specific voice named by a quoted string or an identifier sequence.
    Name(CssVoiceFamilyName),
    /// A generic voice described by an optional age, a gender and an optional variant.
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
        if entries.iter().any(|entry| {
            matches!(entry,
            CssVoiceFamilyEntry::Generic(value)
                if value.variant().is_some_and(|value| value.first_implicit_origin().is_some()))
        }) {
            return None;
        }
        Self::from_parser(entries)
    }
    pub(crate) fn from_parser(entries: Vec<CssVoiceFamilyEntry>) -> Option<Self> {
        (!entries.is_empty()).then_some(Self { entries })
    }
    /// Returns the entries in priority order; the slice is never empty.
    #[must_use]
    pub fn entries(&self) -> &[CssVoiceFamilyEntry] {
        &self.entries
    }
}
/// A prioritized voice list or the distinct language-preservation alternative.
///
/// # Specified serialization
///
/// Serializes specified author choices without contextual voice selection.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceFamily {
    /// A prioritized list of voices.
    Voices(CssVoiceFamilyList),
    /// `preserve`: the inherited `voice-family` value is kept and used even when the content language changes.
    Preserve,
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssVoiceFamily {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.node()?;
        match self {
            Self::Preserve => writer.append("preserve")?,
            Self::Voices(list) => {
                for (index, entry) in list.entries.iter().enumerate() {
                    if index > 0 {
                        writer.append(", ")?;
                    }
                    writer.node()?;
                    match entry {
                        CssVoiceFamilyEntry::Name(name) => match name.view() {
                            CssVoiceFamilyNameRef::Quoted(value) => {
                                writer.node()?;
                                writer.append_string(value)?;
                            }
                            CssVoiceFamilyNameRef::Identifiers(values) => {
                                for (index, value) in values.iter().enumerate() {
                                    if index > 0 {
                                        writer.append(" ")?;
                                    }
                                    writer.node()?;
                                    writer.append_identifier(value.as_str())?;
                                }
                            }
                        },
                        CssVoiceFamilyEntry::Generic(value) => {
                            if let Some(age) = value.age {
                                writer.node()?;
                                writer.append(age.keyword())?;
                                writer.append(" ")?;
                            }
                            writer.node()?;
                            writer.append(value.gender.keyword())?;
                            if let Some(variant) = &value.variant {
                                writer.append(" ")?;
                                variant.append_specified(&mut writer.context, &mut writer.css)?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssVoiceFamily);

/// The authored speech-subtree duration; subtree precedence is resolved downstream.
///
/// # Specified serialization
///
/// Serializes the specified value without applying subtree precedence.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceDuration {
    /// `auto`: the duration is that of the speech synthesis at the inherited `voice-rate`.
    Auto,
    /// A non-negative `<time>` giving the duration in absolute time units.
    Time(CssNumeric<CssTime, CssNonNegative>),
}

impl CssVoiceDuration {
    /// Checks ordinary nonnegativity and original calculation closure.
    pub fn try_time(time: CssNumeric<CssTime>) -> Result<Self, CssNumericConstructionError> {
        time.try_narrow_checked().map(Self::Time)
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssVoiceDuration {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        match self {
            Self::Auto => writer.keyword("auto"),
            Self::Time(value) => {
                value.append_specified(&mut writer.context, &mut writer.css)?;
                Ok(())
            }
        }
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssVoiceDuration);

/// A voice-dependent symbolic pitch or pitch-range level.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceLevel {
    /// `x-low`: the lowest level.
    XLow,
    /// `low`: a level no lower than `x-low`.
    Low,
    /// `medium`: a level no lower than `low`.
    Medium,
    /// `high`: a level no lower than `medium`.
    High,
    /// `x-high`: the highest level.
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
    component: Box<crate::component::CssComponentValue>,
}
impl CssSemitoneLiteral {
    /// Creates a semitone dimension from a number spelling, attaching the `st` unit.
    ///
    /// # Errors
    ///
    /// Returns an error when `number` is not a valid number token.
    pub fn try_new(number: &str) -> Result<Self, crate::component::CssComponentValueError> {
        Self::try_from_component(crate::component::CssComponentValue::try_dimension(
            number, "st",
        )?)
    }
    /// Retains one dimension component whose decoded unit is `st`, compared ASCII case-insensitively.
    ///
    /// # Errors
    ///
    /// Returns an error when `component` is not a dimension token with the `st` unit.
    pub fn try_from_component(
        component: crate::component::CssComponentValue,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        if !matches!(component.view(), crate::component::CssComponentValueRef::Token(crate::component::CssValueTokenRef::Dimension { unit, .. }) if unit.eq_ignore_ascii_case("st"))
        {
            return Err(crate::component::CssComponentValueError::new(
                crate::component::values::CssComponentValueErrorKind::InvalidToken,
                component.origin().clone(),
            ));
        }
        Ok(Self {
            component: Box::new(component),
        })
    }
    /// Borrows the exact authored coefficient without machine-float narrowing.
    #[must_use]
    pub fn numeric(&self) -> crate::component::CssNumericTokenRef<'_> {
        let crate::component::CssComponentValueRef::Token(
            crate::component::CssValueTokenRef::Dimension { number, .. },
        ) = self.component.view()
        else {
            unreachable!("checked semitone dimension")
        };
        number
    }
    /// Returns the checked `st` dimension component, including its original source.
    #[must_use]
    pub const fn component(&self) -> &crate::component::CssComponentValue {
        &self.component
    }
    /// Returns the provenance of the dimension component.
    #[must_use]
    pub const fn origin(&self) -> &crate::component::CssValueOrigin {
        self.component.origin()
    }

    fn append_specified(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        writer.node()?;
        if writer.context.output_suppressed() {
            return Ok(());
        }
        let text = crate::serialization::specified_serialization::format_coefficient(
            self.numeric().representation(),
            0,
            "st",
            writer.context.remaining_bytes(),
        )?;
        writer.append(&text)
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssSemitoneLiteral {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssSemitoneLiteral);

/// One signed relative offset, without applying it to an inherited voice frequency.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceOffset {
    /// A relative `<frequency>` offset, such as `+30Hz`.
    Frequency(crate::numeric::CssNumeric<crate::numeric::CssFrequency>),
    /// A relative offset in semitones (`<semitones>`), such as `+2st`.
    Semitones(CssSemitoneLiteral),
    /// A relative `<percentage>` offset.
    Percentage(crate::numeric::CssNumeric<crate::numeric::CssPercentage>),
    /// Typed math permits frequency and percentage against an unresolved frequency basis.
    Calculation(crate::numeric::CssCalculation<crate::numeric::CssFrequencyPercentage>),
}
impl CssVoiceOffset {
    /// Requires original closure and normalizes an ordinary calculation root back
    /// into the corresponding frequency/percentage literal owner.
    pub fn try_from_calculation(
        value: crate::numeric::CssCalculation<crate::numeric::CssFrequencyPercentage>,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(origin) = value.components().first_implicit_origin() {
            return Err(CssNumericConstructionError::at_origin(
                crate::numeric::expression::CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        let root = crate::numeric::specified_numeric::significant_root(value.components())?;
        if matches!(
            root.view(),
            crate::component::CssComponentValueRef::Token(_)
        ) {
            return Self::from_literal_component(root.clone());
        }
        Ok(Self::Calculation(value))
    }
    fn from_literal_component(
        component: crate::component::CssComponentValue,
    ) -> Result<Self, CssNumericConstructionError> {
        match component.view() {
            crate::component::CssComponentValueRef::Token(crate::component::CssValueTokenRef::Percentage(_)) => {
                crate::numeric::CssNumeric::<crate::numeric::CssPercentage>::try_from_component(component)
                    .map(Self::Percentage)
            }
            crate::component::CssComponentValueRef::Token(crate::component::CssValueTokenRef::Dimension {
                unit, ..
            }) if unit.eq_ignore_ascii_case("st") => {
                CssSemitoneLiteral::try_from_component(component)
                    .map(Self::Semitones)
                    .map_err(CssNumericConstructionError::component)
            }
            _ => crate::numeric::CssNumericLiteral::<crate::numeric::CssFrequency>::from_component_token(component)
                .map(crate::numeric::CssNumeric::<crate::numeric::CssFrequency>::from_literal)
                .map(Self::Frequency)
                .map_err(CssNumericConstructionError::component),
        }
    }
    pub(crate) fn from_parser_component(
        component: crate::component::CssComponentValue,
        context: &crate::numeric::expression::NumericInputContext<'_>,
    ) -> Result<Self, CssNumericConstructionError> {
        if matches!(
            component.view(),
            crate::component::CssComponentValueRef::Token(_)
        ) {
            return Self::from_literal_component(component);
        }
        let values =
            crate::component::CssComponentValues::try_new(vec![component], Default::default())
                .map_err(CssNumericConstructionError::component)?;
        let expression = context.admit(
            values,
            crate::numeric::expression::CalculationRoot::FrequencyPercentage,
        )?;
        Ok(Self::Calculation(crate::numeric::CssCalculation::<
            crate::numeric::CssFrequencyPercentage,
        >::from_expression(expression)))
    }
    fn ensure_closed(&self) -> Result<(), CssNumericConstructionError> {
        let components = match self {
            Self::Frequency(value) => return value.ensure_closed(),
            Self::Semitones(_) => None,
            Self::Percentage(value) => value
                .calculation()
                .map(crate::numeric::CssCalculation::<crate::numeric::CssPercentage>::components),
            Self::Calculation(value) => Some(value.components()),
        };
        if let Some(origin) =
            components.and_then(crate::component::CssComponentValues::first_implicit_origin)
        {
            return Err(CssNumericConstructionError::at_origin(
                crate::numeric::expression::CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        Ok(())
    }
    fn append_specified(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        match self {
            Self::Frequency(value) => value.append_specified(&mut writer.context, &mut writer.css),
            Self::Semitones(value) => value.append_specified(writer),
            Self::Percentage(value) => value.append_specified(&mut writer.context, &mut writer.css),
            Self::Calculation(value) => crate::numeric::projection::project_specified_into(
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
    Absolute(crate::numeric::CssNumeric<crate::numeric::CssFrequency>),
    Relative {
        level: Option<CssVoiceLevel>,
        offset: Option<CssVoiceOffset>,
    },
}
/// Checked shared authored grammar for `voice-pitch` and `voice-range`.
/// Absolute frequency cannot carry a level; relative composition is nonempty.
///
/// # Specified serialization
///
/// Emits frequency before `absolute`, or level before offset. It does not compute a voice
/// frequency. Ordinary frequency output shares its six-place precision owner. If a positive
/// absolute literal rounds to zero, serialization returns `UnrepresentableValue` rather than
/// emitting invalid grammar. Exact input remains retained; this does not perform computed voice
/// clamping.
#[derive(Clone, Debug, PartialEq)]
pub struct CssVoicePitchRange {
    state: VoicePitchRangeState,
}
impl CssVoicePitchRange {
    /// Requires strictly positive ordinary absolute frequency. Typed frequency
    /// calculations retain their authored range for downstream processing.
    pub fn try_absolute(
        value: crate::numeric::CssNumeric<crate::numeric::CssFrequency>,
    ) -> Result<Self, CssNumericConstructionError> {
        value.ensure_closed()?;
        Self::from_parser_absolute(value)
    }
    pub(crate) fn from_parser_absolute(
        value: crate::numeric::CssNumeric<crate::numeric::CssFrequency>,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(literal) = value.literal() {
            let coefficient = crate::serialization::exact_decimal::LexicalDecimal::new(
                literal.numeric().representation(),
            );
            if coefficient.negative || coefficient.len == 0 {
                return Err(CssNumericConstructionError::at_origin(
                    crate::numeric::expression::CssNumericConstructionErrorKind::OutOfRange,
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
                crate::numeric::expression::CssNumericConstructionErrorKind::EmptyValue,
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
    /// Returns the frequency of the `absolute` form, or `None` for a relative composition.
    #[must_use]
    pub const fn absolute_frequency(
        &self,
    ) -> Option<&crate::numeric::CssNumeric<crate::numeric::CssFrequency>> {
        match &self.state {
            VoicePitchRangeState::Absolute(value) => Some(value),
            VoicePitchRangeState::Relative { .. } => None,
        }
    }
    /// Returns the symbolic level of a relative composition, or `None` when none was authored or the value is absolute.
    #[must_use]
    pub const fn level(&self) -> Option<CssVoiceLevel> {
        match &self.state {
            VoicePitchRangeState::Relative { level, .. } => *level,
            VoicePitchRangeState::Absolute(_) => None,
        }
    }
    /// Returns the offset of a relative composition, or `None` when none was authored or the value is absolute.
    #[must_use]
    pub const fn offset(&self) -> Option<&CssVoiceOffset> {
        match &self.state {
            VoicePitchRangeState::Relative { offset, .. } => offset.as_ref(),
            VoicePitchRangeState::Absolute(_) => None,
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssVoicePitchRange {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.node()?;
        match &self.state {
            VoicePitchRangeState::Absolute(value) => {
                let start = writer.css.len();
                value.append_specified(&mut writer.context, &mut writer.css)?;
                if let Some(literal) = value.literal() {
                    let coefficient = writer.css[start..]
                        .strip_suffix(crate::numeric::units::suffix(
                            crate::numeric::CssNumericUnit::Frequency(literal.unit()),
                        ))
                        .expect("frequency provider emits its selected canonical unit");
                    if crate::serialization::exact_decimal::LexicalDecimal::new(coefficient)
                        .is_zero()
                    {
                        return Err(CssSerializationError::new(
                            crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
                        ));
                    }
                }
                writer.node()?;
                writer.append(" absolute")?;
            }
            VoicePitchRangeState::Relative { level, offset } => {
                if let Some(level) = level {
                    writer.node()?;
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
crate::serialization::specified_rule_serialization::specified_serialization!(CssVoicePitchRange);

/// A symbolic voice-rate keyword, without a words-per-minute mapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceRateKeyword {
    /// `normal`: the default rate produced by the speech synthesizer for the active voice.
    Normal,
    /// `x-slow`: the slowest speaking rate keyword.
    XSlow,
    /// `slow`: a speaking rate no slower than `x-slow`.
    Slow,
    /// `medium`: a speaking rate no slower than `slow`.
    Medium,
    /// `fast`: a speaking rate no slower than `medium`.
    Fast,
    /// `x-fast`: the fastest speaking rate keyword.
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
///
/// # Specified serialization
///
/// Emits grammar order and omits only an exact ordinary neutral 100% modifier beside an explicit
/// keyword. The suppressed child is still visited.
#[derive(Clone, Debug, PartialEq)]
pub struct CssVoiceRate {
    keyword: Option<CssVoiceRateKeyword>,
    percentage: Option<
        crate::numeric::CssNumeric<crate::numeric::CssPercentage, crate::numeric::CssNonNegative>,
    >,
}

impl CssVoiceRate {
    /// Creates a rate from an optional keyword and an optional non-negative percentage.
    ///
    /// # Errors
    ///
    /// Returns an error when both are absent or when the percentage calculation was closed implicitly.
    pub fn try_new(
        keyword: Option<CssVoiceRateKeyword>,
        percentage: Option<
            crate::numeric::CssNumeric<
                crate::numeric::CssPercentage,
                crate::numeric::CssNonNegative,
            >,
        >,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(value) = percentage.as_ref().and_then(
            crate::numeric::CssNumeric::<
                crate::numeric::CssPercentage,
                crate::numeric::CssNonNegative,
            >::calculation,
        ) && let Some(origin) = value.components().first_implicit_origin()
        {
            return Err(CssNumericConstructionError::at_origin(
                crate::numeric::expression::CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        Self::from_parser(keyword, percentage)
    }
    pub(crate) fn from_parser(
        keyword: Option<CssVoiceRateKeyword>,
        percentage: Option<
            crate::numeric::CssNumeric<
                crate::numeric::CssPercentage,
                crate::numeric::CssNonNegative,
            >,
        >,
    ) -> Result<Self, CssNumericConstructionError> {
        if keyword.is_none() && percentage.is_none() {
            return Err(CssNumericConstructionError::at(
                crate::numeric::expression::CssNumericConstructionErrorKind::EmptyValue,
                None,
            ));
        }
        Ok(Self {
            keyword,
            percentage,
        })
    }
    /// Creates a rate holding only a keyword, with no percentage.
    #[must_use]
    pub const fn keyword_only(keyword: CssVoiceRateKeyword) -> Self {
        Self {
            keyword: Some(keyword),
            percentage: None,
        }
    }
    /// Returns the keyword, or `None` when only a percentage was authored.
    #[must_use]
    pub const fn keyword(&self) -> Option<CssVoiceRateKeyword> {
        self.keyword
    }
    /// Returns the authored percentage, or `None` when none was authored; an authored `100%` is kept.
    #[must_use]
    pub const fn percentage(
        &self,
    ) -> Option<
        &crate::numeric::CssNumeric<crate::numeric::CssPercentage, crate::numeric::CssNonNegative>,
    > {
        self.percentage.as_ref()
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssVoiceRate {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.node()?;
        if let Some(keyword) = self.keyword {
            writer.node()?;
            writer.append(keyword.keyword())?;
        }
        if let Some(percentage) = &self.percentage {
            let neutral = percentage.literal_component().is_some_and(|component| {
                matches!(component.view(), crate::component::CssComponentValueRef::Token(crate::component::CssValueTokenRef::Percentage(number))
                    if crate::serialization::exact_decimal::LexicalDecimal::new(number.representation()).value_eq(&crate::serialization::exact_decimal::LexicalDecimal::new("100")))
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
crate::serialization::specified_rule_serialization::specified_serialization!(CssVoiceRate);

/// An authored spatial-balance keyword; numerical resolution belongs downstream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceBalanceKeyword {
    /// `left`: the left side; equivalent to the number `-100`.
    Left,
    /// `center`: the center point; equivalent to the number `0`.
    Center,
    /// `right`: the right side; equivalent to the number `100`.
    Right,
    /// `leftwards`: moves the sound left of the inherited balance, by subtracting 20 and clamping to `-100`.
    Leftwards,
    /// `rightwards`: moves the sound right of the inherited balance, by adding 20 and clamping to `100`.
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
    Number(crate::numeric::CssNumeric<crate::numeric::CssNumber>),
}
/// Exact authored balance keyword or unrestricted pure Number value.
/// Specified admission does not clamp numbers or resolve inherited adjustments.
///
/// # Specified serialization
///
/// Formats through shared Number precision without Speech clamping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssVoiceBalance {
    state: VoiceBalanceState,
}
impl CssVoiceBalance {
    /// Creates a balance holding a keyword.
    #[must_use]
    pub const fn from_keyword(keyword: CssVoiceBalanceKeyword) -> Self {
        Self {
            state: VoiceBalanceState::Keyword(keyword),
        }
    }
    /// Retains pure Number syntax, rejecting original recovered math closure.
    /// Percentage values cannot cross this Number-only construction boundary.
    /// ```compile_fail,E0308
    /// use surgeist_css::component::CssComponentValue;
    /// use surgeist_css::numeric::{CssNumeric, CssPercentage};
    /// use surgeist_css::values::speech::CssVoiceBalance;
    /// let percentage = CssNumeric::<CssPercentage>::try_from_component(
    ///     CssComponentValue::try_percentage("50").unwrap()).unwrap();
    /// CssVoiceBalance::try_number(percentage);
    /// ```
    pub fn try_number(
        number: crate::numeric::CssNumeric<crate::numeric::CssNumber>,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(value) = number.calculation()
            && let Some(origin) = value.components().first_implicit_origin()
        {
            return Err(CssNumericConstructionError::at_origin(
                crate::numeric::expression::CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        Ok(Self::from_parser_number(number))
    }
    pub(crate) const fn from_parser_number(
        number: crate::numeric::CssNumeric<crate::numeric::CssNumber>,
    ) -> Self {
        Self {
            state: VoiceBalanceState::Number(number),
        }
    }
    /// Returns the keyword, or `None` when the balance is a number.
    #[must_use]
    pub const fn keyword(&self) -> Option<CssVoiceBalanceKeyword> {
        match &self.state {
            VoiceBalanceState::Keyword(value) => Some(*value),
            VoiceBalanceState::Number(_) => None,
        }
    }
    /// Returns the authored number, which is not clamped, or `None` when the balance is a keyword.
    #[must_use]
    pub const fn number(&self) -> Option<&crate::numeric::CssNumeric<crate::numeric::CssNumber>> {
        match &self.state {
            VoiceBalanceState::Number(value) => Some(value),
            VoiceBalanceState::Keyword(_) => None,
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssVoiceBalance {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        match &self.state {
            VoiceBalanceState::Keyword(value) => writer.keyword(value.keyword()),
            VoiceBalanceState::Number(value) => value.append_to_rule_writer(writer),
        }
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssVoiceBalance);

/// A symbolic user-calibrated volume level, excluding the distinct silent state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceVolumeLevel {
    /// `x-soft`: the quietest calibrated volume level.
    XSoft,
    /// `soft`: a calibrated level no quieter than `x-soft`.
    Soft,
    /// `medium`: a calibrated level no quieter than `soft`.
    Medium,
    /// `loud`: a calibrated level no quieter than `medium`.
    Loud,
    /// `x-loud`: the loudest calibrated volume level.
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
///
/// # Specified serialization
///
/// Serializes level before offset, preserving standalone offset meaning. Standalone zero and
/// rounded tiny nonzero offsets are emitted.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVoiceVolume {
    /// `silent`: no sound is generated, while the element still takes the time it would have been spoken in.
    Silent,
    /// A calibrated level keyword with an optional decibel offset.
    Level {
        /// The level keyword.
        level: CssVoiceVolumeLevel,
        /// The optional relative offset in decibels; omission is distinct from an explicit zero offset.
        decibel: Option<CssDecibelLiteral>,
    },
    /// A `<decibel>` offset with no level keyword.
    Offset(CssDecibelLiteral),
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssVoiceVolume {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        match self {
            Self::Silent => writer.keyword("silent"),
            Self::Offset(value) => value.append_to_rule_writer(writer),
            Self::Level { level, decibel } => {
                writer.node()?;
                writer.node()?;
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
crate::serialization::specified_rule_serialization::specified_serialization!(CssVoiceVolume);
