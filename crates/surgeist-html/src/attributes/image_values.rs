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
    Conforming,
    Nonconforming(HtmlImageAttributeConformanceError),
    Unapplied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageAttributeConformanceError {
    RequiredAttributeMissing,
    EmptyAlt,
    EmptySource,
    CommonValue(HtmlValueFailure),
}

/// A delegated or resource-dependent requirement is not certified locally.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageExternalConformance {
    /// No attribute value is present for this selected requirement.
    NotRequired,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputImageAttributeError {
    EmptyAlt,
    EmptySource,
}
impl fmt::Display for HtmlInputImageAttributeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid local Input Image string: {self:?}")
    }
}
impl Error for HtmlInputImageAttributeError {}

fn source_range(value: &DocumentString) -> Range<usize> {
    let units = value.code_units();
    let start = units
        .iter()
        .position(|&unit| !crate::tokens::ascii_whitespace(unit))
        .unwrap_or(units.len());
    let end = units[start..]
        .iter()
        .rposition(|&unit| !crate::tokens::ascii_whitespace(unit))
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
    pub fn alt(&self) -> &DocumentString {
        &self.alt
    }
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
    pub fn alt(&self) -> &HtmlInputImageStringReport {
        &self.alt
    }
    pub fn src(&self) -> &HtmlInputImageStringReport {
        &self.src
    }
}

/// Actual ordinary-parent evidence, not an inferred picture-parent boolean.
/// Captured callers must obtain this from their coherently qualified capture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlImageSourceParent {
    Parent(QualifiedName),
    NoParent,
    Unavailable,
}

/// Concrete role-specific facts supplied to the dimension companion.
/// A fact for the wrong element family is DifferentContext, not a negative fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlEmbeddedDimensionContext {
    InputState(HtmlInputType),
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
    pub fn width(&self) -> &HtmlEmbeddedDimensionReport {
        &self.width
    }
    pub fn height(&self) -> &HtmlEmbeddedDimensionReport {
        &self.height
    }
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
#[derive(Clone, Copy)]
pub(crate) enum EmbeddedDimensionParent {
    Parent(Option<KnownHtmlElement>),
    NoParent,
    Unavailable,
}
#[derive(Clone, Copy)]
pub(crate) enum EmbeddedDimensionContext {
    InputState(HtmlInputType),
    SourceParent(EmbeddedDimensionParent),
}
#[derive(Clone, Copy)]
pub(crate) enum EmbeddedDimensionEvidenceNeeded {
    InputState,
    SourceParent,
}
#[derive(Clone, Copy)]
pub(crate) enum EmbeddedDimensionApplicability {
    Applicable,
    Inapplicable,
    Unavailable(EmbeddedDimensionEvidenceNeeded),
    DifferentContext,
}
pub(crate) fn embedded_dimension_applicability(
    element: Option<KnownHtmlElement>,
    context: Option<EmbeddedDimensionContext>,
) -> EmbeddedDimensionApplicability {
    use EmbeddedDimensionApplicability as A;
    let Some(element) = element.filter(|e| IMAGE_ELEMENTS.contains(e)) else {
        return A::Inapplicable;
    };
    match (element, context) {
        (KnownHtmlElement::Input, None) => {
            A::Unavailable(EmbeddedDimensionEvidenceNeeded::InputState)
        }
        (
            KnownHtmlElement::Input,
            Some(EmbeddedDimensionContext::InputState(HtmlInputType::Image)),
        ) => A::Applicable,
        (KnownHtmlElement::Input, Some(EmbeddedDimensionContext::InputState(_))) => A::Inapplicable,
        (
            KnownHtmlElement::Source,
            None
            | Some(EmbeddedDimensionContext::SourceParent(EmbeddedDimensionParent::Unavailable)),
        ) => A::Unavailable(EmbeddedDimensionEvidenceNeeded::SourceParent),
        (
            KnownHtmlElement::Source,
            Some(EmbeddedDimensionContext::SourceParent(EmbeddedDimensionParent::Parent(Some(
                KnownHtmlElement::Picture,
            )))),
        ) => A::Applicable,
        (KnownHtmlElement::Source, Some(EmbeddedDimensionContext::SourceParent(_))) => {
            A::Inapplicable
        }
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
            EmbeddedDimensionContext::InputState(*state)
        }
        HtmlEmbeddedDimensionContext::SourceParent(parent) => {
            EmbeddedDimensionContext::SourceParent(match parent {
                HtmlImageSourceParent::Parent(name) => {
                    EmbeddedDimensionParent::Parent(KnownHtmlElement::from_name(name))
                }
                HtmlImageSourceParent::NoParent => EmbeddedDimensionParent::NoParent,
                HtmlImageSourceParent::Unavailable => EmbeddedDimensionParent::Unavailable,
            })
        }
    });
    match embedded_dimension_applicability(
        KnownHtmlElement::from_name(attribute.element()),
        classified,
    ) {
        EmbeddedDimensionApplicability::Applicable => {
            if let Some(context) = context {
                if let HtmlEmbeddedDimensionContext::InputState(state) = context {
                    attribute.supplied_context = SuppliedContext::Input(*state);
                }
                attribute.context_author = HtmlAttributeContextAuthorState::Conforming;
            }
            ImageRuleSelection::Selected
        }
        EmbeddedDimensionApplicability::Unavailable(_) => {
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
        EmbeddedDimensionApplicability::Inapplicable => {
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
        EmbeddedDimensionApplicability::DifferentContext => {
            attribute.applicability = HtmlAttributeApplicability::DifferentContext;
            attribute.context_author = HtmlAttributeContextAuthorState::Unapplied;
            ImageRuleSelection::Unselected
        }
    }
}

impl HtmlValueInterpreter {
    fn input_image_string(
        &self,
        element: &QualifiedName,
        selected: KnownHtmlAttribute,
        raw: Option<&Attribute>,
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
        element: &QualifiedName,
        state: Option<HtmlInputType>,
        alt: Option<&Attribute>,
        src: Option<&Attribute>,
    ) -> Result<HtmlInputImageAttributeReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(alt.map(Attribute::value))?;
        self.admit_attribute_raw(src.map(Attribute::value))?;
        Ok(HtmlInputImageAttributeReport {
            alt: self.input_image_string(element, KnownHtmlAttribute::Alt, alt, state),
            src: self.input_image_string(element, KnownHtmlAttribute::Src, src, state),
        })
    }

    /// Retains local input strings without assuming its actual type state.
    /// Unknown state leaves required-string checks Unchecked, even on absence.
    pub fn input_image_attributes(
        &self,
        element: &QualifiedName,
        alt: Option<&Attribute>,
        src: Option<&Attribute>,
    ) -> Result<HtmlInputImageAttributeReport, HtmlValueAdmissionError> {
        self.input_image_report(element, None, alt, src)
    }

    /// Observes the actual existing InputType classification. Only Image selects
    /// these local checks; other states retain forbidden-presence context evidence.
    pub fn input_image_attributes_for_state(
        &self,
        element: &QualifiedName,
        state: HtmlInputType,
        alt: Option<&Attribute>,
        src: Option<&Attribute>,
    ) -> Result<HtmlInputImageAttributeReport, HtmlValueAdmissionError> {
        self.input_image_report(element, Some(state), alt, src)
    }

    fn embedded_dimension(
        &self,
        element: &QualifiedName,
        selected: KnownHtmlAttribute,
        raw: Option<&Attribute>,
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
        let common = self.nonnegative_integer(raw.map(Attribute::value))?;
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
        element: &QualifiedName,
        width: Option<&Attribute>,
        height: Option<&Attribute>,
        context: Option<&HtmlEmbeddedDimensionContext>,
    ) -> Result<HtmlEmbeddedDimensionAttributeReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(width.map(Attribute::value))?;
        self.admit_attribute_raw(height.map(Attribute::value))?;
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
        element: &QualifiedName,
        width: Option<&Attribute>,
        height: Option<&Attribute>,
    ) -> Result<HtmlEmbeddedDimensionAttributeReport, HtmlValueAdmissionError> {
        self.embedded_dimension_report(element, width, height, None)
    }

    /// Applies actual input-state or source-parent facts without inferring them
    /// from raw values. This is not a captured proof or natural-resource check.
    pub fn embedded_dimension_attributes_with_context(
        &self,
        element: &QualifiedName,
        width: Option<&Attribute>,
        height: Option<&Attribute>,
        context: &HtmlEmbeddedDimensionContext,
    ) -> Result<HtmlEmbeddedDimensionAttributeReport, HtmlValueAdmissionError> {
        self.embedded_dimension_report(element, width, height, Some(context))
    }
}
