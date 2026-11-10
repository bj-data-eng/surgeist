//! Fixtures shared by every module of the HTML integration binary.
//!
//! Each helper is written once here. Modules import a helper under the
//! local name their tests already use, so a module still states which
//! fixture it depends on without copying its constructor.

use std::convert::Infallible;
use surgeist_html::accounting::{HtmlChange, HtmlResource, HtmlWork};
use surgeist_ledger::{Change, Charge};

use surgeist_html::bounded::native::native_element_initialization_with;
use surgeist_html::{
    attributes::{
        HtmlAttributeApplicability, HtmlAttributeContextAuthorState,
        HtmlAttributeContextConformanceError,
        scalar::{
            HtmlNumericAttributeAuthorState, HtmlNumericAttributeReport,
            HtmlNumericAttributeValueOrigin,
        },
    },
    construction::{
        HtmlConstructionCause, HtmlConstructionError, HtmlConstructionLimits,
        HtmlConstructionResource,
    },
    elements::{
        input::{HtmlInputSanitization, HtmlInputValueSanitization},
        native::HtmlNativeInitialization,
    },
    microsyntax::{numeric::HtmlNonNegativeInteger, temporal::HtmlYear},
    values::{
        Attribute, DocumentString, HtmlValueInterpreter, Namespace, OrderedAttributes,
        QualifiedName,
    },
    vocabulary::{KnownHtmlAttribute, KnownHtmlElement},
};

/// Construction limits that ordinary builder fixtures do not reach: one
/// hundred nodes, edges and attributes, 100,000 UTF-16 units and depth one
/// hundred.
pub fn construction_limits() -> HtmlConstructionLimits {
    HtmlConstructionLimits::new(100, 100, 100, 100_000, 100)
}

/// HTML-namespace element name with the exact local name and no prefix.
pub fn html_element(local: &str) -> QualifiedName {
    QualifiedName::new(Namespace::html(), None, local)
}

/// Qualified name of a known HTML element.
pub fn known_element(name: KnownHtmlElement) -> QualifiedName {
    name.qualified_name()
}

/// Qualified name of the HTML `input` element.
pub fn input_element() -> QualifiedName {
    KnownHtmlElement::Input.qualified_name()
}

/// Known HTML attribute carrying the exact raw value.
pub fn known_attribute(name: KnownHtmlAttribute, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(name.qualified_name(), value)
}

/// Known HTML attribute whose raw value is borrowed text, for callers that
/// pass borrowed `String` values.
pub fn known_text_attribute(name: KnownHtmlAttribute, value: &str) -> Attribute {
    known_attribute(name, value)
}

/// The `name` attribute carrying the exact raw value.
pub fn name_attribute(value: impl Into<DocumentString>) -> Attribute {
    known_attribute(KnownHtmlAttribute::Name, value)
}

/// Null-namespace, unprefixed attribute with the exact local name and value.
pub fn unqualified_attribute(local: &str, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, local), value)
}

/// Null-namespace, unprefixed attribute whose raw value is borrowed text,
/// for callers that pass borrowed `String` values.
pub fn unqualified_text_attribute(local: &str, value: &str) -> Attribute {
    unqualified_attribute(local, value)
}

/// Validated ordered attributes in the supplied order.
pub fn ordered_attributes(values: impl IntoIterator<Item = Attribute>) -> OrderedAttributes {
    OrderedAttributes::new(values).unwrap()
}

/// Value interpreter admitting 4096 raw input units.
pub fn value_interpreter() -> HtmlValueInterpreter {
    HtmlValueInterpreter::new(4096)
}

/// Native non-negative integer from exact decimal digits.
pub fn non_negative_integer(digits: &str) -> HtmlNonNegativeInteger {
    HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(digits)).unwrap()
}

/// Valid HTML year.
pub fn year(value: u64) -> HtmlYear {
    HtmlYear::new(value).unwrap()
}

/// Native initialization of an element whose name reads cannot fail.
pub fn initialize_native_element(name: &QualifiedName) -> HtmlNativeInitialization {
    native_element_initialization_with(name, |_| Ok::<_, Infallible>(())).unwrap()
}

/// Author state of an attribute that the input element's current state
/// forbids.
pub fn input_state_forbids_attribute() -> HtmlAttributeContextAuthorState {
    HtmlAttributeContextAuthorState::Nonconforming(
        HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
    )
}

/// String sanitization of an input value policy; the literal cases that use
/// it never select the range or color operation.
pub fn string_sanitization(policy: HtmlInputValueSanitization) -> HtmlInputSanitization {
    match policy {
        HtmlInputValueSanitization::String(policy) => policy,
        HtmlInputValueSanitization::Range | HtmlInputValueSanitization::Color => {
            panic!("these literal cases require the string operation")
        }
    }
}

/// Report of a table cell or column span attribute.
pub type SpanReport = HtmlNumericAttributeReport<HtmlNonNegativeInteger, u16>;

/// The span report applies `value` from the stated origin.
pub fn assert_span_quantity(
    report: &SpanReport,
    value: u16,
    origin: HtmlNumericAttributeValueOrigin,
) {
    assert_eq!(report.intrinsic(), Some(&value));
    assert_eq!(report.origin(), Some(origin));
}

/// The span attribute is not applicable and reports no value.
pub fn assert_span_unapplied(report: &SpanReport) {
    assert_ne!(
        report.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Unapplied);
    assert!(report.common().is_none());
    assert!(report.intrinsic().is_none());
    assert!(report.origin().is_none());
}

/// A construction refusal named by the cause that recognizes it.
///
/// A test states the expected cause by its `HtmlConstructionCause` variant and
/// compares it with `assert_eq!(error, cause::CYCLE)`.
#[derive(Clone, Copy)]
pub struct Cause {
    name: &'static str,
    recognizes: fn(HtmlConstructionCause) -> bool,
}

impl Cause {
    pub const fn new(name: &'static str, recognizes: fn(HtmlConstructionCause) -> bool) -> Self {
        Self { name, recognizes }
    }
}

impl std::fmt::Debug for Cause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

impl PartialEq<Cause> for HtmlConstructionError {
    fn eq(&self, cause: &Cause) -> bool {
        (cause.recognizes)(self.cause())
    }
}

/// Whether an optional refusal is present and has the expected cause.
pub fn has_cause(error: Option<&HtmlConstructionError>, cause: Cause) -> bool {
    error.is_some_and(|error| error == &cause)
}

/// Whether an optional refusal is a limit failure of exactly this resource,
/// attempted total and configured limit.
pub fn is_limit(
    error: Option<&HtmlConstructionError>,
    resource: HtmlConstructionResource,
    attempted: usize,
    limit: usize,
) -> bool {
    error.is_some_and(|error| {
        error.cause()
            == HtmlConstructionCause::LimitExceeded {
                resource,
                limit,
                attempted,
            }
    })
}

/// The refusal causes tests expect.
pub mod cause {
    use super::Cause;
    use surgeist_html::construction::HtmlConstructionCause;

    pub const FOREIGN_BUILDER: Cause = Cause::new("ForeignBuilder", |cause| {
        matches!(cause, HtmlConstructionCause::ForeignBuilder)
    });
    pub const PARENT_CANNOT_HAVE_CHILDREN: Cause =
        Cause::new("ParentCannotHaveChildren", |cause| {
            matches!(cause, HtmlConstructionCause::ParentCannotHaveChildren)
        });
    pub const TEMPLATE_CONTENTS_REQUIRED: Cause = Cause::new("TemplateContentsRequired", |cause| {
        matches!(cause, HtmlConstructionCause::TemplateContentsRequired)
    });
    pub const SHADOW_HOST_MUST_BE_ELEMENT: Cause = Cause::new("ShadowHostMustBeElement", |cause| {
        matches!(cause, HtmlConstructionCause::ShadowHostMustBeElement)
    });
    pub const SHADOW_ALREADY_ATTACHED: Cause = Cause::new("ShadowAlreadyAttached", |cause| {
        matches!(cause, HtmlConstructionCause::ShadowAlreadyAttached)
    });
    pub const WRONG_NODE_KIND: Cause = Cause::new("WrongNodeKind", |cause| {
        matches!(cause, HtmlConstructionCause::WrongNodeKind)
    });
    pub const WRONG_SIBLING: Cause = Cause::new("WrongSibling", |cause| {
        matches!(cause, HtmlConstructionCause::WrongSibling)
    });
    pub const CYCLE: Cause = Cause::new("Cycle", |cause| {
        matches!(cause, HtmlConstructionCause::Cycle)
    });
    pub const INVALID_TEMPLATE_ASSOCIATION: Cause =
        Cause::new("InvalidTemplateAssociation", |cause| {
            matches!(cause, HtmlConstructionCause::InvalidTemplateAssociation)
        });
    pub const OPERATION_NOT_READY: Cause = Cause::new("OperationNotReady", |cause| {
        matches!(cause, HtmlConstructionCause::OperationNotReady)
    });
    pub const FOREIGN_ALLOCATION: Cause = Cause::new("ForeignAllocation", |cause| {
        matches!(cause, HtmlConstructionCause::ForeignAllocation)
    });
    pub const CAPACITY_MISMATCH: Cause = Cause::new("CapacityMismatch", |cause| {
        matches!(cause, HtmlConstructionCause::CapacityMismatch)
    });
    pub const SHARED_ALLOCATION_ALIASED: Cause = Cause::new("SharedAllocationAliased", |cause| {
        matches!(cause, HtmlConstructionCause::SharedAllocationAliased)
    });
    pub const REPRESENTATION_EXHAUSTED: Cause = Cause::new("RepresentationExhausted", |cause| {
        matches!(cause, HtmlConstructionCause::RepresentationExhausted)
    });
    pub const ALLOCATION_FAILED: Cause = Cause::new("AllocationFailed", |cause| {
        matches!(cause, HtmlConstructionCause::AllocationFailed)
    });
}

/// Adapters that keep a construction protocol owner in an `Option` slot, as a driver that stores
/// it in a field does, and put the owner back when its consuming step refuses it.
pub mod slots {
    use surgeist_html::bounded::construction::{
        HtmlConstructionInitialization, HtmlConstructionLease, HtmlConstructionOperation,
        HtmlConstructionReadyInitialization, HtmlConstructionReadyOperation,
        HtmlConstructionRecovery, HtmlConstructionRefusal, HtmlConstructionResult,
    };
    use surgeist_html::construction::HtmlConstructionError;

    fn step<T, R>(
        slot: &mut Option<T>,
        consume: impl FnOnce(T) -> Result<R, HtmlConstructionRefusal<T>>,
    ) -> Result<R, HtmlConstructionError> {
        let owner = slot.take().expect("slot holds its owner");
        match consume(owner) {
            Ok(value) => Ok(value),
            Err(refusal) => {
                let (owner, reason) = refusal.into_parts();
                *slot = Some(owner);
                Err(reason)
            }
        }
    }
    pub fn initialization(
        slot: &mut Option<HtmlConstructionInitialization>,
    ) -> Result<HtmlConstructionReadyInitialization, HtmlConstructionError> {
        step(slot, HtmlConstructionInitialization::try_ready)
    }
    pub fn operation(
        slot: &mut Option<HtmlConstructionOperation>,
    ) -> Result<HtmlConstructionReadyOperation, HtmlConstructionError> {
        step(slot, HtmlConstructionOperation::try_ready)
    }
    pub fn recover(
        result: &mut Option<HtmlConstructionResult>,
        lease: &mut Option<HtmlConstructionLease>,
    ) -> Result<HtmlConstructionRecovery, HtmlConstructionError> {
        let owners = (
            result.take().expect("slot holds its result"),
            lease.take().expect("slot holds its lease"),
        );
        match HtmlConstructionLease::try_recover(owners.0, owners.1) {
            Ok(value) => Ok(value),
            Err(refusal) => {
                let ((returned_result, returned_lease), reason) = refusal.into_parts();
                *result = Some(returned_result);
                *lease = Some(returned_lease);
                Err(reason)
            }
        }
    }
}

/// A charger that admits and records every batch, in order, so a test reads exactly what an
/// operation charged and in which batches.
#[derive(Debug, Default)]
pub struct Recorder {
    batches: Vec<Vec<HtmlChange>>,
    gauges: Vec<(HtmlResource, u64)>,
}

impl Recorder {
    /// Every batch charged so far, oldest first.
    pub fn batches(&self) -> &[Vec<HtmlChange>] {
        &self.batches
    }

    /// Every gauge recorded so far, oldest first.
    pub fn gauges(&self) -> &[(HtmlResource, u64)] {
        &self.gauges
    }

    /// The amount charged to one work dimension across all batches.
    pub fn total(&self, dimension: HtmlWork) -> u64 {
        self.batches
            .iter()
            .flatten()
            .map(|change| match *change {
                Change::Work(charged, amount) if charged == dimension => amount,
                _ => 0,
            })
            .sum()
    }
}

impl Charge<HtmlWork, HtmlResource> for Recorder {
    type Stop = Infallible;

    fn charge(&mut self, batch: &[HtmlChange]) -> Result<(), Infallible> {
        self.batches.push(batch.to_vec());
        Ok(())
    }

    fn record_largest(&mut self, resource: HtmlResource, value: u64) -> Result<(), Infallible> {
        self.gauges.push((resource, value));
        Ok(())
    }
}

/// The stop of a [`Refuser`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Refused;

/// A charger that refuses every batch and records it.
#[derive(Debug, Default)]
pub struct Refuser {
    pub attempts: Vec<Vec<HtmlChange>>,
}

impl Charge<HtmlWork, HtmlResource> for Refuser {
    type Stop = Refused;

    fn charge(&mut self, batch: &[HtmlChange]) -> Result<(), Refused> {
        self.attempts.push(batch.to_vec());
        Err(Refused)
    }

    fn record_largest(&mut self, _: HtmlResource, _: u64) -> Result<(), Refused> {
        Err(Refused)
    }
}

/// The work a construction action description charges through `charge`, read from a recorder.
pub fn charged_by(
    charge: impl FnOnce(
        &mut Recorder,
    ) -> Result<
        (),
        surgeist_html::accounting::HtmlChargedError<
            Infallible,
            surgeist_html::bounded::construction::HtmlConstructionChargeFault,
        >,
    >,
) -> Recorder {
    let mut recorder = Recorder::default();
    charge(&mut recorder).expect("the description charges the part it has");
    recorder
}
