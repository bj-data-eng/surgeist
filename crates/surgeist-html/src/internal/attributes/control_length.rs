//! Input size, control length bounds, reflected getters and initial-text author checks.
//!
//! Content processing does not implement rendering, current/dirty control values,
//! user editing, constraint validation, WebIDL setters or DOM mutation. WHATWG
//! source attribution is in `licenses/whatwg/NOTICE.md`.

use super::*;
use crate::{
    internal::microsyntax::numeric::HtmlNonNegativeInteger,
    internal::values::interpretation::HtmlDefinedAuthorState,
    internal::values::interpretation::HtmlDefinedValueReport,
    internal::values::interpretation::HtmlValueFailure,
    internal::values::interpretation::HtmlValueRecoveryState,
    internal::values::interpretation::HtmlValueReport,
};
use std::{cmp::Ordering, error::Error, fmt};

/// Admission is per supplied value, before any name/context or syntax processing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlControlLengthError {
    /// A supplied raw value (minimum, maximum or initial text) was refused by
    /// raw-value admission, before any name, context or syntax processing.
    Admission(HtmlValueAdmissionError),
    /// The sum of supplied UTF-16 lengths cannot be represented by usize.
    InputCodeUnitsOverflow,
}

impl From<HtmlValueAdmissionError> for HtmlControlLengthError {
    fn from(error: HtmlValueAdmissionError) -> Self {
        Self::Admission(error)
    }
}

impl fmt::Display for HtmlControlLengthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Admission(_) => f.write_str("control value was refused by raw-value admission"),
            Self::InputCodeUnitsOverflow => {
                f.write_str("control input UTF-16 length cannot be represented")
            }
        }
    }
}
impl Error for HtmlControlLengthError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Admission(error) => Some(error),
            Self::InputCodeUnitsOverflow => None,
        }
    }
}

/// Whole-value authorship is independent of numeric prefix recovery and context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlControlIntegerAuthorState {
    /// The attribute is absent, so nothing is authored.
    Missing,
    /// The whole value is a valid non-negative integer (a positive one for
    /// input `size`).
    Conforming,
    /// The whole value is not a valid non-negative integer. The failure is the
    /// syntax or representation cause.
    Nonconforming(HtmlValueFailure),
    /// Only input size requires a strictly positive authored number.
    ZeroMustBePositive,
    /// The rule does not apply to these names or this context, so no author
    /// check was made.
    Unapplied,
}

/// Application of the content bound/advisory, not reflected getter recognition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlControlAttributeApplication {
    /// The attribute takes effect: the element is a textarea, or an input whose
    /// supplied state is Text, Search, Telephone, Url, Email or Password.
    Applicable,
    /// The element is an input and no actual input state was supplied, so
    /// whether the attribute takes effect cannot be decided.
    InputStateRequired,
    /// The supplied input state is not one that honours the attribute. The
    /// attribute has no effect, and its presence is nonconforming.
    IgnoredInputState,
    /// The rule does not apply to these names or this context. The payload is
    /// the reason, as reported for every attribute rule.
    Unapplied(HtmlAttributeApplicability),
}

/// A reflected getter's fallback cause. No content default follows from this.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlControlGetterOrigin {
    /// The recovered number is within the getter's range and is returned as is.
    Recovered,
    /// The attribute is absent, so the getter returns its IDL default.
    MissingDefault,
    /// Parsing the value failed, so the getter returns its IDL default.
    ParseFailureDefault,
    /// The recovered number exceeds 2147483647, the largest value the getter
    /// can return, so the getter returns its IDL default.
    OutOfRangeDefault,
    /// Input size's positive-only getter rejects recovered zero.
    ZeroDefault,
}

/// The content effect of an applicable input `size` attribute.
///
/// The `size` attribute is defined in HTML877d #the-size-attribute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlInputSizeValue {
    /// A positive number of characters was requested. The number is the exact
    /// recovered value and may exceed what the getter can return.
    PositiveCharacters(HtmlNonNegativeInteger),
    /// No positive size was requested: the attribute is missing, its value does
    /// not parse, or it recovers to zero. This is not the getter's default of
    /// 20, which only `reflected_getter` applies.
    NoPositiveRequest,
}

/// A source-null bound differs from an ignored, unresolved or unapplied rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlControlLengthValue {
    /// The exact recovered bound in UTF-16 code units. Zero is a genuine bound.
    Bound(HtmlNonNegativeInteger),
    /// The attribute is missing or its value does not parse, so the source
    /// defines no bound.
    NoBound,
}

#[derive(Clone, Debug, PartialEq)]
struct IntegerObservation {
    attribute: HtmlAttributeReport<()>,
    common: Option<HtmlValueReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>>,
    author: HtmlControlIntegerAuthorState,
    application: HtmlControlAttributeApplication,
}

impl IntegerObservation {
    fn recovered(&self) -> Option<&HtmlNonNegativeInteger> {
        match self.common.as_ref()?.recovery() {
            HtmlValueRecoveryState::Value(value) => Some(value),
            _ => None,
        }
    }
    fn getter(&self, positive: bool) -> Option<(u32, HtmlControlGetterOrigin)> {
        let common = self.common.as_ref()?;
        let origin = match common.recovery() {
            HtmlValueRecoveryState::Missing => HtmlControlGetterOrigin::MissingDefault,
            HtmlValueRecoveryState::Failed(_) => HtmlControlGetterOrigin::ParseFailureDefault,
            HtmlValueRecoveryState::Value(value) if positive && value.is_zero() => {
                HtmlControlGetterOrigin::ZeroDefault
            }
            HtmlValueRecoveryState::Value(value) => {
                if let Ok(value) = value.to_u64()
                    && value <= 2_147_483_647
                {
                    return Some((value as u32, HtmlControlGetterOrigin::Recovered));
                }
                HtmlControlGetterOrigin::OutOfRangeDefault
            }
        };
        Some((0, origin))
    }
}

/// Exact size content plus its separate positive unsigned-long getter projection.
/// Without actual input state, lexical recovery/getter remain observable while
/// the content advisory is unresolved. Raw admission is not allocation accounting.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlInputSizeReport {
    integer: IntegerObservation,
    content: Option<HtmlInputSizeValue>,
}

/// One independently selected minlength or maxlength content attribute.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlControlLengthReport {
    integer: IntegerObservation,
    content: Option<HtmlControlLengthValue>,
}

macro_rules! integer_observation_accessors {
    ($report:ident) => {
        impl $report {
            /// The exact element name, including any stored prefix.
            pub fn element(&self) -> &QualifiedName {
                self.integer.attribute.element()
            }
            /// The exact supplied attribute name, prefix and value, if present.
            pub fn raw_attribute(&self) -> Option<&Attribute> {
                self.integer.attribute.raw_attribute()
            }
            /// The exact supplied raw value. `None` is an absent attribute; an
            /// empty value is `Some`.
            pub fn raw(&self) -> Option<&DocumentString> {
                self.integer.attribute.raw()
            }
            /// Whether the attribute is absent, empty or nonempty, independent of
            /// validity.
            pub fn observation(&self) -> HtmlAttributeObservation {
                self.integer.attribute.observation()
            }
            /// Expanded-name/fact-family applicability, independent of input-state effects.
            pub fn applicability(&self) -> HtmlAttributeApplicability {
                self.integer.attribute.applicability()
            }
            /// The selected standard attribute (`size`, `minlength` or
            /// `maxlength`), not raw recognition of the supplied name.
            pub fn attribute(&self) -> KnownHtmlAttribute {
                self.integer.attribute.attribute()
            }
            /// The original index row identifiers that this defining rule covers.
            pub fn inventory_rows(&self) -> &'static [u16] {
                self.integer.attribute.inventory_rows()
            }
            /// The normative clause in the pinned edition that defines the
            /// attribute for this element, not an index summary.
            pub fn source_clause(&self) -> &'static str {
                self.integer.attribute.source_clause()
            }
            /// The clause that defines reflecting a content attribute in an IDL
            /// attribute (HTML877d #reflecting-content-attributes-in-idl-attributes).
            pub fn getter_source_clause(&self) -> &'static str {
                source!("reflecting-content-attributes-in-idl-attributes")
            }
            /// The supplied actual input state, if any. It is evidence from the
            /// caller and is never inferred from a `type` attribute.
            pub fn input_state(&self) -> Option<HtmlInputType> {
                self.integer.attribute.input_state()
            }
            /// The clause that defines the supplied input state, if one was
            /// supplied.
            pub fn input_state_source_clause(&self) -> Option<&'static str> {
                self.input_state().map(input_state_source)
            }
            /// The UTF-16 code units in the raw value that was admitted, or zero
            /// when the attribute is absent.
            pub fn input_code_units(&self) -> usize {
                self.integer.attribute.input_code_units()
            }
            /// The shared non-negative integer report for the raw value: parse
            /// result, author state and recovery. `None` when the rule is
            /// unapplied, because no value was interpreted.
            pub fn common(
                &self,
            ) -> Option<&HtmlValueReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>> {
                self.integer.common.as_ref()
            }
            /// Whether the whole value is a valid non-negative integer. This is
            /// independent of numeric prefix recovery and of context.
            pub fn author(&self) -> HtmlControlIntegerAuthorState {
                self.integer.author
            }
            /// The extra author-context check selected for this attribute. For an
            /// input it reports whether the supplied state permits the attribute.
            pub fn context_author(&self) -> HtmlAttributeContextAuthorState {
                self.integer.attribute.context_author()
            }
            /// Whether the attribute takes effect for the supplied element and
            /// input state.
            pub fn application(&self) -> HtmlControlAttributeApplication {
                self.integer.application
            }
            /// Exact recovered number, even when content effects are ignored/unresolved.
            pub fn recovered(&self) -> Option<&HtmlNonNegativeInteger> {
                self.integer.recovered()
            }
        }
    };
}
integer_observation_accessors!(HtmlInputSizeReport);
integer_observation_accessors!(HtmlControlLengthReport);

impl HtmlInputSizeReport {
    /// The clause that defines the `size` IDL attribute, or `None` when the
    /// element is not an input.
    pub fn getter_definition_source_clause(&self) -> Option<&'static str> {
        (KnownHtmlElement::from_name(self.element()) == Some(KnownHtmlElement::Input))
            .then_some(source!("dom-input-size"))
    }
    /// None means ignored/unresolved/unapplied; NoPositiveRequest has no default20.
    pub fn content(&self) -> Option<&HtmlInputSizeValue> {
        self.content.as_ref()
    }
    /// Source-defined size getter only; no input-state guard or WebIDL setter.
    pub fn reflected_getter(&self) -> Option<u32> {
        self.integer.getter(true).map(|(value, origin)| {
            if origin == HtmlControlGetterOrigin::Recovered {
                value
            } else {
                20
            }
        })
    }
    /// Why the `size` getter returns what `reflected_getter` reports. `None`
    /// when the rule is unapplied.
    pub fn getter_origin(&self) -> Option<HtmlControlGetterOrigin> {
        self.integer.getter(true).map(|(_, origin)| origin)
    }
}

impl HtmlControlLengthReport {
    /// The clause that defines the matching `minLength` or `maxLength` IDL
    /// attribute, or `None` when the element is neither an input nor a textarea.
    pub fn getter_definition_source_clause(&self) -> Option<&'static str> {
        match (
            KnownHtmlElement::from_name(self.element()),
            self.attribute(),
        ) {
            (Some(KnownHtmlElement::Input), KnownHtmlAttribute::Minlength) => {
                Some(source!("dom-input-minlength"))
            }
            (Some(KnownHtmlElement::Input), KnownHtmlAttribute::Maxlength) => {
                Some(source!("dom-input-maxlength"))
            }
            (Some(KnownHtmlElement::Textarea), KnownHtmlAttribute::Minlength) => {
                Some(source!("dom-textarea-minlength"))
            }
            (Some(KnownHtmlElement::Textarea), KnownHtmlAttribute::Maxlength) => {
                Some(source!("dom-textarea-maxlength"))
            }
            _ => None,
        }
    }
    /// The clause that defines the `ReflectNonNegative` limitation applied by
    /// the getter (HTML877d #reflecting-content-attributes-in-idl-attributes).
    pub fn getter_limitation_source_clause(&self) -> &'static str {
        source!("xattr-reflectnonnegative")
    }
    /// The content bound, or `None` when the attribute is ignored, unresolved
    /// or unapplied. `Some(NoBound)` is the source-defined absence of a bound.
    pub fn content(&self) -> Option<&HtmlControlLengthValue> {
        self.content.as_ref()
    }
    /// Nonnegative signed-long reflection: failure/missing/out-of-range yields −1.
    /// This has no input-state guard and does not select a content bound.
    pub fn reflected_getter(&self) -> Option<i32> {
        self.integer.getter(false).map(|(value, origin)| {
            if origin == HtmlControlGetterOrigin::Recovered {
                value as i32
            } else {
                -1
            }
        })
    }
    /// Why the getter returns what `reflected_getter` reports. `None` when the
    /// rule is unapplied.
    pub fn getter_origin(&self) -> Option<HtmlControlGetterOrigin> {
        self.integer.getter(false).map(|(_, origin)| origin)
    }
    /// The clause that defines how the form control `minlength` or `maxlength`
    /// attribute is processed (HTML877d #attr-fe-maxlength and #attr-fe-minlength).
    pub fn processing_source_clause(&self) -> &'static str {
        if self.attribute() == KnownHtmlAttribute::Minlength {
            source!("attr-fe-minlength")
        } else {
            source!("attr-fe-maxlength")
        }
    }
}

/// Actual authored initial text, supplied by its attribute/tree owner.
/// Input absence is established absence, not an unavailable current value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlControlInitialText {
    /// An input's authored `value` attribute. `None` is an established absent
    /// attribute. A present attribute must be named `value`, or the check
    /// reports `DifferentValueAttribute`.
    InputValue(Option<Attribute>),
    /// A textarea's descendant text, exactly as authored and before line break
    /// normalization.
    TextareaDescendantText(DocumentString),
}

impl HtmlControlInitialText {
    fn raw(&self) -> Option<&DocumentString> {
        match self {
            Self::InputValue(value) => value.as_ref().map(HtmlAttributeInput::value),
            Self::TextareaDescendantText(value) => Some(value),
        }
    }
}

/// Only the common minimum≤maximum author requirement is checked here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlControlLengthPairState {
    /// One or both attributes have no recovered number, or an ignored input
    /// state makes the pair moot, so no ordering is required.
    NotRequired,
    /// Both attributes have numbers, but the input state that decides whether
    /// they apply was not supplied.
    Unchecked,
    /// Both bounds apply and the minimum does not exceed the maximum.
    Conforming,
    /// Both bounds apply and the minimum is greater than the maximum, which the
    /// source forbids.
    MinimumExceedsMaximum,
    /// The rule does not apply to these names or this context.
    Unapplied,
}

/// Initial authored text's maximum-length check, not runtime tooLong/tooShort.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlControlInitialTextState {
    /// There is no maximum number, or the input has no authored value, so the
    /// source requires nothing of the initial text.
    NotRequired,
    /// A maximum applies but no initial text was supplied to compare with it.
    Unchecked,
    /// A maximum number exists, but the input state that decides whether it
    /// applies was not supplied.
    InputStateRequired,
    /// The supplied input state ignores `maxlength`, so the text is unchecked.
    IgnoredInputState,
    /// The supplied text kind does not belong to the element: input text for a
    /// textarea or textarea text for an input.
    DifferentContext,
    /// The input's supplied attribute is not named `value`.
    DifferentValueAttribute,
    /// The text length is within the maximum.
    Conforming {
        /// UTF-16 code units of the text, counted after textarea line break
        /// normalization where it applies.
        code_units: usize,
    },
    /// The text length is greater than the maximum.
    ExceedsMaximum {
        /// UTF-16 code units of the text, counted after textarea line break
        /// normalization where it applies.
        code_units: usize,
    },
    /// The rule does not apply to these names or this context.
    Unapplied,
}

/// Independent min/max raw observations and narrowly scoped author checks.
///
/// Each supplied raw value is admitted separately before scans, even for wrong
/// names or ignored context. input_code_units is their checked sum, not another
/// aggregate limit. This standalone operation does not guarantee allocation
/// failure recovery, work credits, cancellation or complete author conformance.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlControlLengthBoundsReport {
    minimum: HtmlControlLengthReport,
    maximum: HtmlControlLengthReport,
    initial_text: Option<HtmlControlInitialText>,
    pair: HtmlControlLengthPairState,
    text: HtmlControlInitialTextState,
    input_code_units: usize,
}

impl HtmlControlLengthBoundsReport {
    /// The independent observation of the `minlength` attribute.
    pub fn minimum(&self) -> &HtmlControlLengthReport {
        &self.minimum
    }
    /// The independent observation of the `maxlength` attribute.
    pub fn maximum(&self) -> &HtmlControlLengthReport {
        &self.maximum
    }
    /// The supplied initial text, retained exactly as it was given.
    pub fn initial_text(&self) -> Option<&HtmlControlInitialText> {
        self.initial_text.as_ref()
    }
    /// Whether the minimum does not exceed the maximum.
    pub fn pair_author(&self) -> HtmlControlLengthPairState {
        self.pair
    }
    /// Whether the supplied initial text fits within the maximum.
    pub fn initial_text_author(&self) -> HtmlControlInitialTextState {
        self.text
    }
    /// The checked sum of the admitted raw value lengths in UTF-16 code units:
    /// minimum, maximum and initial text. It is not another aggregate limit.
    pub fn input_code_units(&self) -> usize {
        self.input_code_units
    }
    /// The clause that defines the minimum-not-above-maximum requirement.
    pub fn pair_source_clause(&self) -> &'static str {
        source!("attr-fe-minlength")
    }
    /// The clause that defines the initial text length requirement for the
    /// maximum's element, or `None` for any other element.
    pub fn initial_text_source_clause(&self) -> Option<&'static str> {
        match KnownHtmlElement::from_name(self.maximum.element()) {
            Some(KnownHtmlElement::Input) => Some(source!("attr-input-maxlength")),
            Some(KnownHtmlElement::Textarea) => Some(source!("attr-textarea-maxlength")),
            _ => None,
        }
    }
    /// The clause that defines textarea line break normalization, which sets the
    /// counted length of textarea text. `None` for any other element.
    pub fn newline_source_clause(&self) -> Option<&'static str> {
        (KnownHtmlElement::from_name(self.maximum.element()) == Some(KnownHtmlElement::Textarea))
            .then_some(source!("textarea-line-break-normalisation-transformation"))
    }
}

/// The three attributes this owner reads as nonnegative integers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ControlIntegerRole {
    Size,
    Minlength,
    Maxlength,
}
impl ControlIntegerRole {
    fn attribute(self) -> KnownHtmlAttribute {
        match self {
            Self::Size => KnownHtmlAttribute::Size,
            Self::Minlength => KnownHtmlAttribute::Minlength,
            Self::Maxlength => KnownHtmlAttribute::Maxlength,
        }
    }
}

fn metadata(
    role: ControlIntegerRole,
    element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
) -> RuleMetadata {
    let textarea = KnownHtmlElement::from_name(element) == Some(KnownHtmlElement::Textarea);
    let (rows, source, context): (&'static [u16], _, _) = match role {
        ControlIntegerRole::Size => (
            &[143],
            source!("attr-input-size"),
            Context::Elements(&[KnownHtmlElement::Input]),
        ),
        ControlIntegerRole::Minlength => (
            &[99],
            if textarea {
                source!("attr-textarea-minlength")
            } else {
                source!("attr-input-minlength")
            },
            Context::Elements(&[KnownHtmlElement::Input, KnownHtmlElement::Textarea]),
        ),
        ControlIntegerRole::Maxlength => (
            &[94],
            if textarea {
                source!("attr-textarea-maxlength")
            } else {
                source!("attr-input-maxlength")
            },
            Context::Elements(&[KnownHtmlElement::Input, KnownHtmlElement::Textarea]),
        ),
    };
    RuleMetadata {
        attribute: role.attribute(),
        rows,
        source,
        context,
        vocabulary_source: None,
    }
}

fn state_allows_lengths(state: HtmlInputType) -> bool {
    matches!(
        state,
        HtmlInputType::Text
            | HtmlInputType::Search
            | HtmlInputType::Telephone
            | HtmlInputType::Url
            | HtmlInputType::Email
            | HtmlInputType::Password
    )
}

fn compare(left: &HtmlNonNegativeInteger, right: &HtmlNonNegativeInteger) -> Ordering {
    let left = left.decimal_digits();
    let right = right.decimal_digits();
    left.len()
        .cmp(&right.len())
        .then_with(|| left.as_bytes().cmp(right.as_bytes()))
}

fn author(
    common: &HtmlDefinedValueReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>,
    positive: bool,
) -> HtmlControlIntegerAuthorState {
    match common.author() {
        HtmlDefinedAuthorState::Missing => HtmlControlIntegerAuthorState::Missing,
        HtmlDefinedAuthorState::Valid(value) if positive && value.is_zero() => {
            HtmlControlIntegerAuthorState::ZeroMustBePositive
        }
        HtmlDefinedAuthorState::Valid(_) => HtmlControlIntegerAuthorState::Conforming,
        HtmlDefinedAuthorState::Invalid(error) => {
            HtmlControlIntegerAuthorState::Nonconforming(HtmlValueFailure::Syntax(*error))
        }
        HtmlDefinedAuthorState::Unrepresentable(error) => {
            HtmlControlIntegerAuthorState::Nonconforming(HtmlValueFailure::Representation(*error))
        }
    }
}

impl HtmlValueInterpreter {
    // All request raw values have been admitted before this shared owner runs.
    fn control_integer(
        &self,
        selected: ControlIntegerRole,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        state: Option<HtmlInputType>,
    ) -> Result<IntegerObservation, HtmlValueAdmissionError> {
        let mut attribute = report(metadata(selected, element), element, raw);
        if let Some(state) = state {
            attribute.supplied_context = SuppliedContext::Input(state);
            if attribute.applicability == HtmlAttributeApplicability::Applicable
                && KnownHtmlElement::from_name(element) != Some(KnownHtmlElement::Input)
            {
                attribute.applicability = HtmlAttributeApplicability::DifferentContext;
            }
        }
        if attribute.applicability != HtmlAttributeApplicability::Applicable {
            attribute.context_author = HtmlAttributeContextAuthorState::Unapplied;
            return Ok(IntegerObservation {
                application: HtmlControlAttributeApplication::Unapplied(attribute.applicability),
                attribute,
                common: None,
                author: HtmlControlIntegerAuthorState::Unapplied,
            });
        }
        let application = if KnownHtmlElement::from_name(element) == Some(KnownHtmlElement::Input) {
            match state {
                None => HtmlControlAttributeApplication::InputStateRequired,
                Some(state) if state_allows_lengths(state) => {
                    HtmlControlAttributeApplication::Applicable
                }
                Some(_) => HtmlControlAttributeApplication::IgnoredInputState,
            }
        } else {
            HtmlControlAttributeApplication::Applicable
        };
        attribute.context_author = if raw.is_none()
            || KnownHtmlElement::from_name(element) == Some(KnownHtmlElement::Textarea)
        {
            HtmlAttributeContextAuthorState::NotRequired
        } else {
            match application {
                HtmlControlAttributeApplication::InputStateRequired => {
                    HtmlAttributeContextAuthorState::Unchecked
                }
                HtmlControlAttributeApplication::IgnoredInputState => {
                    HtmlAttributeContextAuthorState::Nonconforming(
                        HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
                    )
                }
                _ => HtmlAttributeContextAuthorState::Conforming,
            }
        };
        let common = self.nonnegative_integer_defined(raw.map(HtmlAttributeInput::value))?;
        let author = author(&common, selected == ControlIntegerRole::Size);
        Ok(IntegerObservation {
            attribute,
            common: Some(common.into_report()),
            author,
            application,
        })
    }

    /// Interprets input size without guessing its actual input state. Reflected
    /// getter and numeric recovery remain available; content advisory is unresolved.
    pub fn input_size(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlInputSizeReport, HtmlValueAdmissionError> {
        self.input_size_with_state(element, raw, None)
    }

    /// Adds a supplied actual input state; ignored states retain raw/recovery/getter.
    pub fn input_size_for_input(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        state: HtmlInputType,
    ) -> Result<HtmlInputSizeReport, HtmlValueAdmissionError> {
        self.input_size_with_state(element, raw, Some(state))
    }

    fn input_size_with_state(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        state: Option<HtmlInputType>,
    ) -> Result<HtmlInputSizeReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let integer = self.control_integer(ControlIntegerRole::Size, element, raw, state)?;
        let content =
            (integer.application == HtmlControlAttributeApplication::Applicable).then(|| {
                match integer.recovered() {
                    Some(value) if !value.is_zero() => {
                        HtmlInputSizeValue::PositiveCharacters(value.clone())
                    }
                    _ => HtmlInputSizeValue::NoPositiveRequest,
                }
            });
        Ok(HtmlInputSizeReport { integer, content })
    }

    /// Interprets independent length bounds, getter projections, the min≤max
    /// relation and optionally supplied initial authored text. Input state is
    /// actual evidence; textarea must not receive an input-state fact.
    /// Every supplied value is admitted before any grammar/name/text scan.
    pub fn control_length_bounds(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        minimum: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        maximum: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        input_state: Option<HtmlInputType>,
        initial_text: Option<&HtmlControlInitialText>,
    ) -> Result<HtmlControlLengthBoundsReport, HtmlControlLengthError> {
        let raws = [
            minimum.map(HtmlAttributeInput::value),
            maximum.map(HtmlAttributeInput::value),
            initial_text.and_then(HtmlControlInitialText::raw),
        ];
        for raw in raws {
            self.admit_attribute_raw(raw)?;
        }
        let input_code_units = raws
            .into_iter()
            .flatten()
            .try_fold(0usize, |total, raw| total.checked_add(raw.len()))
            .ok_or(HtmlControlLengthError::InputCodeUnitsOverflow)?;
        let minimum =
            self.length_attribute(ControlIntegerRole::Minlength, element, minimum, input_state)?;
        let maximum =
            self.length_attribute(ControlIntegerRole::Maxlength, element, maximum, input_state)?;
        let pair = pair_author(&minimum, &maximum);
        let text = initial_text_author(&maximum, initial_text);
        Ok(HtmlControlLengthBoundsReport {
            minimum,
            maximum,
            initial_text: initial_text.cloned(),
            pair,
            text,
            input_code_units,
        })
    }

    fn length_attribute(
        &self,
        selected: ControlIntegerRole,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        state: Option<HtmlInputType>,
    ) -> Result<HtmlControlLengthReport, HtmlValueAdmissionError> {
        let integer = self.control_integer(selected, element, raw, state)?;
        let content =
            (integer.application == HtmlControlAttributeApplication::Applicable).then(|| {
                integer
                    .recovered()
                    .map_or(HtmlControlLengthValue::NoBound, |value| {
                        HtmlControlLengthValue::Bound(value.clone())
                    })
            });
        Ok(HtmlControlLengthReport { integer, content })
    }
}

fn pair_author(
    minimum: &HtmlControlLengthReport,
    maximum: &HtmlControlLengthReport,
) -> HtmlControlLengthPairState {
    use HtmlControlAttributeApplication::{
        Applicable, IgnoredInputState, InputStateRequired, Unapplied,
    };
    if matches!(minimum.application(), Unapplied(_))
        || matches!(maximum.application(), Unapplied(_))
    {
        return HtmlControlLengthPairState::Unapplied;
    }
    let (Some(min), Some(max)) = (minimum.recovered(), maximum.recovered()) else {
        return HtmlControlLengthPairState::NotRequired;
    };
    match (minimum.application(), maximum.application()) {
        (InputStateRequired, _) | (_, InputStateRequired) => HtmlControlLengthPairState::Unchecked,
        (IgnoredInputState, _) | (_, IgnoredInputState) => HtmlControlLengthPairState::NotRequired,
        (Applicable, Applicable) if compare(min, max) == Ordering::Greater => {
            HtmlControlLengthPairState::MinimumExceedsMaximum
        }
        (Applicable, Applicable) => HtmlControlLengthPairState::Conforming,
        _ => HtmlControlLengthPairState::Unapplied,
    }
}

fn initial_text_author(
    maximum: &HtmlControlLengthReport,
    initial: Option<&HtmlControlInitialText>,
) -> HtmlControlInitialTextState {
    use HtmlControlInitialTextState as State;
    let element = KnownHtmlElement::from_name(maximum.element());
    // Supplied wrong fact families/names are explicit even if no bound exists.
    match (element, initial) {
        (
            Some(KnownHtmlElement::Input),
            Some(HtmlControlInitialText::TextareaDescendantText(_)),
        )
        | (Some(KnownHtmlElement::Textarea), Some(HtmlControlInitialText::InputValue(_))) => {
            return State::DifferentContext;
        }
        (Some(KnownHtmlElement::Input), Some(HtmlControlInitialText::InputValue(Some(value))))
            if KnownHtmlAttribute::from_name(value.name()) != Some(KnownHtmlAttribute::Value) =>
        {
            return State::DifferentValueAttribute;
        }
        _ => {}
    }
    match maximum.application() {
        HtmlControlAttributeApplication::Unapplied(_) => return State::Unapplied,
        _ if maximum.recovered().is_none()
            || matches!(initial, Some(HtmlControlInitialText::InputValue(None))) =>
        {
            return State::NotRequired;
        }
        HtmlControlAttributeApplication::InputStateRequired => return State::InputStateRequired,
        HtmlControlAttributeApplication::IgnoredInputState => return State::IgnoredInputState,
        HtmlControlAttributeApplication::Applicable => {}
    }
    let Some(HtmlControlLengthValue::Bound(bound)) = maximum.content() else {
        return State::NotRequired;
    };
    let Some(initial) = initial else {
        return State::Unchecked;
    };
    let Some(raw) = initial.raw() else {
        return State::NotRequired;
    };
    let code_units = if element == Some(KnownHtmlElement::Textarea) {
        normalized_text_length(raw)
    } else {
        raw.len()
    };
    if compare(&HtmlNonNegativeInteger::from(code_units as u128), bound) == Ordering::Greater {
        State::ExceedsMaximum { code_units }
    } else {
        State::Conforming { code_units }
    }
}

fn normalized_text_length(raw: &DocumentString) -> usize {
    // Normalization can only shorten this already represented UTF-16 length.
    let units = raw.code_units();
    let mut length = units.len();
    for pair in units.windows(2) {
        if pair == [0x0d, 0x0a] {
            length -= 1;
        }
    }
    length
}

/// The refusal to emit input `size` for zero, which the source forbids because
/// the attribute must be a positive integer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputSizeZeroError;
impl fmt::Display for HtmlInputSizeZeroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("input size must be positive")
    }
}
impl Error for HtmlInputSizeZeroError {}

impl HtmlNonNegativeInteger {
    /// Canonical syntax for a positive input size, without an IDL range claim.
    pub fn into_input_size_attribute(self) -> Result<Attribute, HtmlInputSizeZeroError> {
        if self.is_zero() {
            return Err(HtmlInputSizeZeroError);
        }
        Ok(Attribute::new(
            KnownHtmlAttribute::Size.qualified_name(),
            self.as_document_string(),
        ))
    }
    /// Canonical nonnegative syntax; applicability/pair/text checks remain separate.
    pub fn into_minlength_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Minlength.qualified_name(),
            self.as_document_string(),
        )
    }
    /// Canonical nonnegative syntax, including a genuine zero maximum.
    pub fn into_maxlength_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Maxlength.qualified_name(),
            self.as_document_string(),
        )
    }
}
