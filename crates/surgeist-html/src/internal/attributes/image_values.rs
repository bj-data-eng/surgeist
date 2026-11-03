//! Local Input Image strings and embedded dimension content values.
//! URL validity, label adequacy, resource restrictions, natural-dimension author
//! relations and the determine-dimensions algorithm are separate operations.

use super::*;
use crate::{
    HtmlNonNegativeInteger, HtmlValueAuthorState, HtmlValueFailure, HtmlValueRecoveryState,
    HtmlValueReport,
};
use std::{error::Error, fmt, ops::Range};

pub(super) const ALT_SOURCE: &str = source!("attr-input-alt");
pub(super) const SRC_SOURCE: &str = source!("attr-input-src");
pub(super) const DIMENSIONS_SOURCE: &str = source!("dimension-attributes");
pub(super) const SOURCE_ELEMENT_SOURCE: &str = source!("the-source-element");

/// Local value evidence only, independently of resource and context requirements.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageAttributeAuthorState {
    /// An optional dimension attribute is absent.
    Missing,
    /// Actual input state or source-parent evidence has not selected the rule.
    Unchecked,
    /// The value meets the local author rule for its selected role.
    Conforming,
    /// The value violates the local author rule. The payload names the rule.
    Nonconforming(HtmlImageAttributeConformanceError),
    /// The rule does not apply to these names or this context.
    Unapplied,
}

/// The local author rule that an image string or dimension value violates.
///
/// The causes are the nonempty `alt` and `src` requirements of the Image Button state (HTML877d
/// #image-button-state-(type=image)) and the dimension attribute grammar (HTML877d
/// #dimension-attributes).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlImageAttributeConformanceError {
    /// A required attribute is absent: `alt` or `src` of an Image Button.
    RequiredAttributeMissing,
    /// `alt` is present but empty. Whitespace-only text is not empty here.
    EmptyAlt,
    /// `src` has no code units left after stripping ASCII whitespace at both
    /// edges.
    EmptySource,
    /// A dimension value is not a valid non-negative integer. The payload is always the syntax
    /// failure of that value: a value too large to represent is still grammatical, so it is
    /// reported as conforming and not as this error.
    CommonValue(HtmlValueFailure),
}

impl std::fmt::Display for HtmlImageAttributeConformanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequiredAttributeMissing => f.write_str("required image attribute is missing"),
            Self::EmptyAlt => f.write_str("image alt attribute is empty"),
            Self::EmptySource => f.write_str("image src attribute is empty"),
            Self::CommonValue(_) => f.write_str("image dimension attribute value is not valid"),
        }
    }
}
impl std::error::Error for HtmlImageAttributeConformanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CommonValue(error) => Some(error),
            _ => None,
        }
    }
}

/// A delegated or resource-dependent requirement is not certified locally.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageExternalConformance {
    /// No attribute value is present for this selected requirement.
    NotRequired,
    /// The requirement depends on external facts that this crate does not
    /// evaluate, so conformance is neither confirmed nor refuted.
    Unchecked,
    /// Expanded names or actual context did not select the requirement.
    Unapplied,
}

/// Exact local Input Image authoring, including an emitted actual Image type.
///
/// This checks required nonempty alt and src's ASCII-edge-stripped nonemptiness.
/// It does not prove label adequacy, URL validity or image-resource restrictions.
/// Original UTF-16 is stored without trimming, decoding or URL resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlInputImageAttributes {
    alt: DocumentString,
    src: DocumentString,
}

/// The refusal to build `HtmlInputImageAttributes` from an invalid string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlInputImageAttributeError {
    /// The `alt` string is empty. An Image Button requires nonempty text.
    EmptyAlt,
    /// The `src` string is empty after stripping ASCII whitespace at both edges.
    EmptySource,
}
impl fmt::Display for HtmlInputImageAttributeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyAlt => "image-button alt attribute is empty",
            Self::EmptySource => "image-button src attribute is empty",
        })
    }
}
impl Error for HtmlInputImageAttributeError {}

fn source_range(value: &DocumentString) -> Range<usize> {
    let units = value.code_units();
    let start = units
        .iter()
        .position(|&unit| !crate::internal::microsyntax::tokens::is_ascii_whitespace(unit))
        .unwrap_or(units.len());
    let end = units[start..]
        .iter()
        .rposition(|&unit| !crate::internal::microsyntax::tokens::is_ascii_whitespace(unit))
        .map_or(start, |last| start + last + 1);
    start..end
}

impl HtmlInputImageAttributes {
    /// Uses ordinary native allocation; no resource or URL parser is invoked.
    pub fn new(
        alt: impl Into<DocumentString>,
        src: impl Into<DocumentString>,
    ) -> Result<Self, HtmlInputImageAttributeError> {
        let alt = alt.into();
        let src = src.into();
        if alt.is_empty() {
            return Err(HtmlInputImageAttributeError::EmptyAlt);
        }
        if source_range(&src).is_empty() {
            return Err(HtmlInputImageAttributeError::EmptySource);
        }
        Ok(Self { alt, src })
    }
    /// The exact nonempty `alt` text, untrimmed.
    pub fn alt(&self) -> &DocumentString {
        &self.alt
    }
    /// The exact `src` string, untrimmed and unresolved. Its ASCII-stripped
    /// form is nonempty.
    pub fn src(&self) -> &DocumentString {
        &self.src
    }
    /// Returns type=image, alt and src in that order, preserving both strings.
    pub fn into_attributes(self) -> [Attribute; 3] {
        [
            HtmlInputType::Image.into_attribute(),
            Attribute::new(KnownHtmlAttribute::Alt.qualified_name(), self.alt),
            Attribute::new(KnownHtmlAttribute::Src.qualified_name(), self.src),
        ]
    }
}

/// One required Image string with exact raw metadata and local evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlInputImageStringReport {
    attribute: HtmlAttributeReport<()>,
    author: HtmlImageAttributeAuthorState,
    source_range: Option<Range<usize>>,
    external: HtmlImageExternalConformance,
}
impl HtmlInputImageStringReport {
    /// The shared name/raw/source/context envelope. Generic keyword author and
    /// recognition fields are Unapplied; use this report's local author state.
    pub fn attribute(&self) -> &HtmlAttributeReport<()> {
        &self.attribute
    }
    /// The local nonempty check of this string, for the role the report selected.
    pub fn author(&self) -> HtmlImageAttributeAuthorState {
        self.author
    }
    /// Src's ASCII-edge-stripped units, borrowed from unchanged raw storage.
    /// This is not a URL parse result. Alt and unselected src return None.
    pub fn source_reference(&self) -> Option<&[u16]> {
        let range = self.source_range.as_ref()?;
        Some(&self.attribute.raw()?.code_units()[range.clone()])
    }
    /// Equivalent-button label adequacy, selected only for actual Image alt.
    pub fn label_adequacy(&self) -> HtmlImageExternalConformance {
        if self.attribute.attribute() == KnownHtmlAttribute::Alt {
            self.external
        } else {
            HtmlImageExternalConformance::Unapplied
        }
    }
    /// Delegated URL grammar, without resolution or a guessed base.
    pub fn url_conformance(&self) -> HtmlImageExternalConformance {
        if self.attribute.attribute() == KnownHtmlAttribute::Src {
            self.external
        } else {
            HtmlImageExternalConformance::Unapplied
        }
    }
    /// Noninteractive, optionally animated, neither paged nor scripted image.
    /// Actual resource classification/loading is not performed here.
    pub fn resource_conformance(&self) -> HtmlImageExternalConformance {
        self.url_conformance()
    }
}

/// Both original slots are admitted before element/state/value processing.
/// Ordinary report allocation and per-original-value UTF-16 caps do not provide
/// aggregate memory/work/RSS limits or cancellation. No overall conformance or
/// display, label, URL, resource or submission outcome is claimed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlInputImageAttributeReport {
    alt: HtmlInputImageStringReport,
    src: HtmlInputImageStringReport,
}
impl HtmlInputImageAttributeReport {
    /// The report for the `alt` slot, which is present even when the attribute
    /// is absent.
    pub fn alt(&self) -> &HtmlInputImageStringReport {
        &self.alt
    }
    /// The report for the `src` slot, which is present even when the attribute
    /// is absent.
    pub fn src(&self) -> &HtmlInputImageStringReport {
        &self.src
    }
}

/// Actual ordinary-parent evidence, not an inferred picture-parent boolean.
/// Captured callers must obtain this from their coherently qualified capture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlImageSourceParent {
    /// The `source` element has a parent with this expanded name. Only an HTML
    /// `picture` parent makes the dimension attributes apply (HTML877d #dimension-attributes).
    Parent(QualifiedName),
    /// The `source` element has no parent, which is established fact.
    NoParent,
    /// The parent could not be established, so applicability is undecided.
    Unavailable,
}

/// Concrete role-specific facts supplied to the dimension companion.
/// A fact for the wrong element family is DifferentContext, not a negative fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlEmbeddedDimensionContext {
    /// The actual state of an `input` element. The dimensions apply only in the
    /// Image Button state.
    InputState(HtmlInputType),
    /// The parent evidence for a `source` element.
    SourceParent(HtmlImageSourceParent),
}

/// A dimension's strict author grammar and independent exact prefix recovery.
/// No source processing default, machine-integer clamp or canvas default applies.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlEmbeddedDimensionReport {
    attribute: HtmlAttributeReport<()>,
    common: Option<HtmlValueReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>>,
    author: HtmlImageAttributeAuthorState,
}
impl HtmlEmbeddedDimensionReport {
    /// Exact names/raw/provenance and independent author context. Generic keyword
    /// recognition/author are Unapplied; grammar evidence belongs to common().
    pub fn attribute(&self) -> &HtmlAttributeReport<()> {
        &self.attribute
    }
    /// None means grammar was not selected, including unknown context. Applicable
    /// missing input has a real common report with Missing author/recovery states.
    pub fn common(
        &self,
    ) -> Option<&HtmlValueReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>> {
        self.common.as_ref()
    }
    /// Whether the value is a valid non-negative integer, per HTML877d #dimension-attributes.
    /// `Unchecked` means the input state or source parent has not selected the
    /// rule; `Missing` means the optional attribute is absent.
    pub fn author(&self) -> HtmlImageAttributeAuthorState {
        self.author
    }
    /// Borrowed exact recovery; absence or parse failure supplies no dimension.
    /// This is not determine-dimensions, natural size or a rendered content box.
    pub fn recovered(&self) -> Option<&HtmlNonNegativeInteger> {
        match self.common.as_ref()?.recovery() {
            HtmlValueRecoveryState::Value(value) => Some(value),
            HtmlValueRecoveryState::Missing | HtmlValueRecoveryState::Failed(_) => None,
        }
    }
}

/// Local dimension values, not resource-qualified geometry or full author proof.
/// Natural-dimension omission and the coupled ±0.5/ratio relation remain unchecked.
/// Input state/source parent are retained independently of raw value failure.
/// Storage uses ordinary allocation and the existing per-raw UTF-16 admission;
/// there is no controlled captured-query, work/byte ledger or cancellation claim.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlEmbeddedDimensionAttributeReport {
    width: HtmlEmbeddedDimensionReport,
    height: HtmlEmbeddedDimensionReport,
    context: Option<HtmlEmbeddedDimensionContext>,
}
impl HtmlEmbeddedDimensionAttributeReport {
    /// The independent observation of the `width` attribute.
    pub fn width(&self) -> &HtmlEmbeddedDimensionReport {
        &self.width
    }
    /// The independent observation of the `height` attribute.
    pub fn height(&self) -> &HtmlEmbeddedDimensionReport {
        &self.height
    }
    /// The supplied role evidence, exactly as given. `None` when the element
    /// needs none.
    pub fn context(&self) -> Option<&HtmlEmbeddedDimensionContext> {
        self.context.as_ref()
    }
}

const IMAGE_ELEMENTS: &[KnownHtmlElement] = &[
    KnownHtmlElement::Img,
    KnownHtmlElement::Iframe,
    KnownHtmlElement::Embed,
    KnownHtmlElement::Object,
    KnownHtmlElement::Video,
    KnownHtmlElement::Source,
    KnownHtmlElement::Input,
];

/// Already established expanded-name evidence; None is known non-HTML/unknown,
/// never a missing capture relation. This classifier owns no tree or controller.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlEmbeddedDimensionParent {
    /// The `source` element has an HTML parent of the known kind, or `None`
    /// for a parent that is not a known HTML element.
    Parent(Option<KnownHtmlElement>),
    /// The `source` element has no parent.
    NoParent,
    /// The parent could not be established.
    Unavailable,
}
/// Role evidence for the embedded dimension attributes, as a copyable fact for
/// `embedded_dimension_applicability`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlEmbeddedDimensionApplicabilityContext {
    /// The actual state of an `input` element.
    InputState(HtmlInputType),
    /// The parent evidence of a `source` element.
    SourceParent(HtmlEmbeddedDimensionParent),
}
/// The kind of evidence that was missing when applicability is undecided.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlEmbeddedDimensionEvidenceNeeded {
    /// The element is an `input` and its actual state was not supplied.
    InputState,
    /// The element is a `source` and its parent was not established.
    SourceParent,
}
/// Whether `width` and `height` apply to an element, given the evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlEmbeddedDimensionApplicability {
    /// The element carries the dimension attributes in this context.
    Applicable,
    /// The element is not among those that carry them, or the supplied
    /// evidence rules them out (an `input` outside the Image Button state, or a
    /// `source` outside a `picture`).
    Inapplicable,
    /// The evidence needed to decide is missing. The payload names it.
    Unavailable(HtmlEmbeddedDimensionEvidenceNeeded),
    /// The supplied evidence belongs to another element family.
    DifferentContext,
}
/// Decides whether `width` and `height` apply to `element`, from the HTML
/// element list of HTML877d #dimension-attributes and the role evidence in `context`.
///
/// `None` for `element` is a non-HTML or unknown element and is inapplicable.
pub fn embedded_dimension_applicability(
    element: Option<KnownHtmlElement>,
    context: Option<HtmlEmbeddedDimensionApplicabilityContext>,
) -> HtmlEmbeddedDimensionApplicability {
    use HtmlEmbeddedDimensionApplicability as A;
    let Some(element) = element.filter(|e| IMAGE_ELEMENTS.contains(e)) else {
        return A::Inapplicable;
    };
    match (element, context) {
        (KnownHtmlElement::Input, None) => {
            A::Unavailable(HtmlEmbeddedDimensionEvidenceNeeded::InputState)
        }
        (
            KnownHtmlElement::Input,
            Some(HtmlEmbeddedDimensionApplicabilityContext::InputState(HtmlInputType::Image)),
        ) => A::Applicable,
        (
            KnownHtmlElement::Input,
            Some(HtmlEmbeddedDimensionApplicabilityContext::InputState(_)),
        ) => A::Inapplicable,
        (
            KnownHtmlElement::Source,
            None
            | Some(HtmlEmbeddedDimensionApplicabilityContext::SourceParent(
                HtmlEmbeddedDimensionParent::Unavailable,
            )),
        ) => A::Unavailable(HtmlEmbeddedDimensionEvidenceNeeded::SourceParent),
        (
            KnownHtmlElement::Source,
            Some(HtmlEmbeddedDimensionApplicabilityContext::SourceParent(
                HtmlEmbeddedDimensionParent::Parent(Some(KnownHtmlElement::Picture)),
            )),
        ) => A::Applicable,
        (
            KnownHtmlElement::Source,
            Some(HtmlEmbeddedDimensionApplicabilityContext::SourceParent(_)),
        ) => A::Inapplicable,
        (_, Some(_)) => A::DifferentContext,
        (_, None) => A::Applicable,
    }
}

fn image_metadata(attribute: KnownHtmlAttribute) -> RuleMetadata {
    let (rows, clause, elements): (&'static [u16], _, _) = match attribute {
        KnownHtmlAttribute::Alt => (&[9][..], ALT_SOURCE, &[KnownHtmlElement::Input][..]),
        KnownHtmlAttribute::Src => (&[149][..], SRC_SOURCE, &[KnownHtmlElement::Input][..]),
        KnownHtmlAttribute::Width => (&[178][..], DIMENSIONS_SOURCE, IMAGE_ELEMENTS),
        KnownHtmlAttribute::Height => (&[64][..], DIMENSIONS_SOURCE, IMAGE_ELEMENTS),
        _ => unreachable!("private image roles are fixed"),
    };
    RuleMetadata {
        attribute,
        rows,
        source: clause,
        context: Context::Elements(elements),
        vocabulary_source: None,
    }
}

#[derive(Clone, Copy)]
enum ImageRuleSelection {
    Selected,
    Unknown,
    Unselected,
}

fn input_image_selection<T>(
    attribute: &mut HtmlAttributeReport<T>,
    state: Option<HtmlInputType>,
) -> ImageRuleSelection {
    if let Some(state) = state {
        attribute.supplied_context = SuppliedContext::Input(state);
    }
    if attribute.applicability != HtmlAttributeApplicability::Applicable {
        return ImageRuleSelection::Unselected;
    }
    match state {
        None => {
            attribute.context_author = HtmlAttributeContextAuthorState::Unchecked;
            ImageRuleSelection::Unknown
        }
        Some(HtmlInputType::Image) => {
            attribute.context_author = HtmlAttributeContextAuthorState::Conforming;
            ImageRuleSelection::Selected
        }
        Some(_) => {
            attribute.context_author = if attribute.raw_attribute.is_some() {
                HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
                )
            } else {
                HtmlAttributeContextAuthorState::NotRequired
            };
            ImageRuleSelection::Unselected
        }
    }
}

fn dimension_selection(
    attribute: &mut HtmlAttributeReport<()>,
    context: Option<&HtmlEmbeddedDimensionContext>,
) -> ImageRuleSelection {
    if attribute.applicability != HtmlAttributeApplicability::Applicable {
        return ImageRuleSelection::Unselected;
    }
    let classified = context.map(|context| match context {
        HtmlEmbeddedDimensionContext::InputState(state) => {
            HtmlEmbeddedDimensionApplicabilityContext::InputState(*state)
        }
        HtmlEmbeddedDimensionContext::SourceParent(parent) => {
            HtmlEmbeddedDimensionApplicabilityContext::SourceParent(match parent {
                HtmlImageSourceParent::Parent(name) => {
                    HtmlEmbeddedDimensionParent::Parent(KnownHtmlElement::from_name(name))
                }
                HtmlImageSourceParent::NoParent => HtmlEmbeddedDimensionParent::NoParent,
                HtmlImageSourceParent::Unavailable => HtmlEmbeddedDimensionParent::Unavailable,
            })
        }
    });
    match embedded_dimension_applicability(
        KnownHtmlElement::from_name(attribute.element()),
        classified,
    ) {
        HtmlEmbeddedDimensionApplicability::Applicable => {
            if let Some(context) = context {
                if let HtmlEmbeddedDimensionContext::InputState(state) = context {
                    attribute.supplied_context = SuppliedContext::Input(*state);
                }
                attribute.context_author = HtmlAttributeContextAuthorState::Conforming;
            }
            ImageRuleSelection::Selected
        }
        HtmlEmbeddedDimensionApplicability::Unavailable(_) => {
            attribute.context_author = if matches!(
                context,
                Some(HtmlEmbeddedDimensionContext::SourceParent(
                    HtmlImageSourceParent::Unavailable
                ))
            ) {
                HtmlAttributeContextAuthorState::Unavailable
            } else {
                HtmlAttributeContextAuthorState::Unchecked
            };
            ImageRuleSelection::Unknown
        }
        HtmlEmbeddedDimensionApplicability::Inapplicable => {
            let error = match context {
                Some(HtmlEmbeddedDimensionContext::InputState(state)) => {
                    attribute.supplied_context = SuppliedContext::Input(*state);
                    HtmlAttributeContextConformanceError::InputStateForbidsAttribute
                }
                Some(HtmlEmbeddedDimensionContext::SourceParent(_)) => {
                    HtmlAttributeContextConformanceError::SourceDimensionsRequirePictureParent
                }
                None => return ImageRuleSelection::Unselected,
            };
            attribute.context_author = if attribute.raw_attribute.is_some() {
                HtmlAttributeContextAuthorState::Nonconforming(error)
            } else {
                HtmlAttributeContextAuthorState::NotRequired
            };
            ImageRuleSelection::Unselected
        }
        HtmlEmbeddedDimensionApplicability::DifferentContext => {
            attribute.applicability = HtmlAttributeApplicability::DifferentContext;
            attribute.context_author = HtmlAttributeContextAuthorState::Unapplied;
            ImageRuleSelection::Unselected
        }
    }
}

impl HtmlValueInterpreter {
    fn input_image_string(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        selected: KnownHtmlAttribute,
        raw: Option<&impl crate::HtmlAttributeInput>,
        state: Option<HtmlInputType>,
    ) -> HtmlInputImageStringReport {
        let mut attribute = report(image_metadata(selected), element, raw);
        let selection = input_image_selection(&mut attribute, state);
        let mut result = HtmlInputImageStringReport {
            attribute,
            author: HtmlImageAttributeAuthorState::Unapplied,
            source_range: None,
            external: HtmlImageExternalConformance::Unapplied,
        };
        match selection {
            ImageRuleSelection::Unknown => {
                result.author = HtmlImageAttributeAuthorState::Unchecked;
                return result;
            }
            ImageRuleSelection::Unselected => return result,
            ImageRuleSelection::Selected => {}
        }
        result.external = if raw.is_some() {
            HtmlImageExternalConformance::Unchecked
        } else {
            HtmlImageExternalConformance::NotRequired
        };
        let error = match raw {
            None => Some(HtmlImageAttributeConformanceError::RequiredAttributeMissing),
            Some(raw) if selected == KnownHtmlAttribute::Alt => raw
                .value()
                .is_empty()
                .then_some(HtmlImageAttributeConformanceError::EmptyAlt),
            Some(raw) => {
                let range = source_range(raw.value());
                let error = range
                    .is_empty()
                    .then_some(HtmlImageAttributeConformanceError::EmptySource);
                result.source_range = Some(range);
                error
            }
        };
        result.author = error.map_or(HtmlImageAttributeAuthorState::Conforming, |error| {
            HtmlImageAttributeAuthorState::Nonconforming(error)
        });
        result
    }

    fn input_image_report(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        state: Option<HtmlInputType>,
        alt: Option<&impl crate::HtmlAttributeInput>,
        src: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlInputImageAttributeReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(alt.map(HtmlAttributeInput::value))?;
        self.admit_attribute_raw(src.map(HtmlAttributeInput::value))?;
        Ok(HtmlInputImageAttributeReport {
            alt: self.input_image_string(element, KnownHtmlAttribute::Alt, alt, state),
            src: self.input_image_string(element, KnownHtmlAttribute::Src, src, state),
        })
    }

    /// Retains local input strings without assuming its actual type state.
    /// Unknown state leaves required-string checks Unchecked, even on absence.
    pub fn input_image_attributes(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        alt: Option<&impl crate::HtmlAttributeInput>,
        src: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlInputImageAttributeReport, HtmlValueAdmissionError> {
        self.input_image_report(element, None, alt, src)
    }

    /// Observes the actual existing InputType classification. Only Image selects
    /// these local checks; other states retain forbidden-presence context evidence.
    pub fn input_image_attributes_for_state(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        state: HtmlInputType,
        alt: Option<&impl crate::HtmlAttributeInput>,
        src: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlInputImageAttributeReport, HtmlValueAdmissionError> {
        self.input_image_report(element, Some(state), alt, src)
    }

    fn embedded_dimension(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        selected: KnownHtmlAttribute,
        raw: Option<&impl crate::HtmlAttributeInput>,
        context: Option<&HtmlEmbeddedDimensionContext>,
    ) -> Result<HtmlEmbeddedDimensionReport, HtmlValueAdmissionError> {
        let mut attribute = report(image_metadata(selected), element, raw);
        let selection = dimension_selection(&mut attribute, context);
        let mut result = HtmlEmbeddedDimensionReport {
            attribute,
            common: None,
            author: HtmlImageAttributeAuthorState::Unapplied,
        };
        match selection {
            ImageRuleSelection::Unknown => {
                result.author = HtmlImageAttributeAuthorState::Unchecked;
                return Ok(result);
            }
            ImageRuleSelection::Unselected => return Ok(result),
            ImageRuleSelection::Selected => {}
        }
        let common = self.nonnegative_integer(raw.map(HtmlAttributeInput::value))?;
        result.author = match common.author() {
            HtmlValueAuthorState::Missing => HtmlImageAttributeAuthorState::Missing,
            HtmlValueAuthorState::Valid(_) => HtmlImageAttributeAuthorState::Conforming,
            HtmlValueAuthorState::Invalid(error) => HtmlImageAttributeAuthorState::Nonconforming(
                HtmlImageAttributeConformanceError::CommonValue(HtmlValueFailure::Syntax(*error)),
            ),
            // The exact integer owner does not report representation overflow;
            // grammar conformance would nevertheless survive that capability
            // failure, while the common report retained it independently.
            HtmlValueAuthorState::Unrepresentable(_) => HtmlImageAttributeAuthorState::Conforming,
            HtmlValueAuthorState::NotDefined => unreachable!("exact integer author grammar"),
        };
        result.common = Some(common);
        Ok(result)
    }

    fn embedded_dimension_report(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        width: Option<&impl crate::HtmlAttributeInput>,
        height: Option<&impl crate::HtmlAttributeInput>,
        context: Option<&HtmlEmbeddedDimensionContext>,
    ) -> Result<HtmlEmbeddedDimensionAttributeReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(width.map(HtmlAttributeInput::value))?;
        self.admit_attribute_raw(height.map(HtmlAttributeInput::value))?;
        Ok(HtmlEmbeddedDimensionAttributeReport {
            width: self.embedded_dimension(element, KnownHtmlAttribute::Width, width, context)?,
            height: self.embedded_dimension(
                element,
                KnownHtmlAttribute::Height,
                height,
                context,
            )?,
            context: context.cloned(),
        })
    }

    /// Selects unconditional embedded roles; actual input state and source parent
    /// remain unknown. Canvas has its separate, unchanged 300/150 default rules.
    pub fn embedded_dimension_attributes(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        width: Option<&impl crate::HtmlAttributeInput>,
        height: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlEmbeddedDimensionAttributeReport, HtmlValueAdmissionError> {
        self.embedded_dimension_report(element, width, height, None)
    }

    /// Applies actual input-state or source-parent facts without inferring them
    /// from raw values. This is not a captured proof or natural-resource check.
    pub fn embedded_dimension_attributes_with_context(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        width: Option<&impl crate::HtmlAttributeInput>,
        height: Option<&impl crate::HtmlAttributeInput>,
        context: &HtmlEmbeddedDimensionContext,
    ) -> Result<HtmlEmbeddedDimensionAttributeReport, HtmlValueAdmissionError> {
        self.embedded_dimension_report(element, width, height, Some(context))
    }
}
