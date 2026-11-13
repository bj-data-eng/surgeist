//! The accounting fault of every public error that carries one directly or inside an error it
//! wraps. Each route returns the literal fault it was built with, and an error without a fault
//! returns none.

use crate::*;
use std::convert::Infallible;
use surgeist_html::accounting::HtmlAccountingFault;

const FAULT: HtmlAccountingFault = HtmlAccountingFault::ScopeLimit { limit: 3 };
const OTHER: HtmlAccountingFault = HtmlAccountingFault::ReleaseUnderflow;

fn capture() -> HtmlCaptureError {
    HtmlCaptureError::Accounting(FAULT)
}

fn no_fault_capture() -> HtmlCaptureError {
    HtmlCaptureError::ForeignBuilder
}

#[test]
fn a_capture_error_has_a_fault_only_from_its_accounting_variant() {
    assert_eq!(capture().accounting_fault(), Some(FAULT));
    assert_eq!(
        HtmlCaptureError::Accounting(OTHER).accounting_fault(),
        Some(OTHER)
    );
    assert_eq!(no_fault_capture().accounting_fault(), None);
    assert_eq!(HtmlCaptureError::ChargeProtocol.accounting_fault(), None);
    assert_eq!(
        HtmlCaptureError::RepresentationExhausted.accounting_fault(),
        None
    );
}

/// A fact error with an `Accounting` variant and a `Capture` variant: each route returns the
/// fault, and a capture error without one returns none.
macro_rules! fact_routes {
    ($($test:ident: $error:ident),+ $(,)?) => {$(
        #[test]
        fn $test() {
            assert_eq!($error::Accounting(FAULT).accounting_fault(), Some(FAULT));
            assert_eq!(
                $error::Capture(capture()).accounting_fault(),
                Some(FAULT)
            );
            assert_eq!($error::Capture(no_fault_capture()).accounting_fault(), None);
        }
    )+};
}

fact_routes! {
    base_facts_route: HtmlBaseFactsError,
    construction_capture_route: HtmlConstructionCaptureError,
    natural_dimension_route: HtmlNaturalDimensionFactsError,
    element_reference_route: HtmlElementReferenceFactsError,
    control_facts_route: HtmlControlFactsError,
    rel_facts_route: HtmlRelFactsError,
    global_inheritance_route: HtmlGlobalInheritanceError,
    image_dimension_route: HtmlImageDimensionFactsError,
    output_default_route: HtmlOutputDefaultError,
    language_direction_route: HtmlLanguageDirectionError,
    media_facts_route: HtmlMediaFactsError,
    inert_route: HtmlInertFactError,
}

#[test]
fn a_microdata_facts_error_has_a_fault_only_through_its_capture_error() {
    assert_eq!(
        HtmlMicrodataFactsError::Capture(capture()).accounting_fault(),
        Some(FAULT)
    );
    assert_eq!(
        HtmlMicrodataFactsError::Capture(no_fault_capture()).accounting_fault(),
        None
    );
    assert_eq!(HtmlMicrodataFactsError::WrongRole.accounting_fault(), None);
}

#[test]
fn a_context_error_finds_the_fault_through_every_nesting() {
    use HtmlSemanticContextError as Context;
    for (error, expected) in [
        (Context::Accounting(FAULT), Some(FAULT)),
        (Context::Capture(capture()), Some(FAULT)),
        (
            Context::ImageDimensionsFacts(HtmlImageDimensionFactsError::Accounting(FAULT)),
            Some(FAULT),
        ),
        (
            Context::ImageDimensionsFacts(HtmlImageDimensionFactsError::Capture(capture())),
            Some(FAULT),
        ),
        (
            Context::ControlsFacts(HtmlControlFactsError::Accounting(FAULT)),
            Some(FAULT),
        ),
        (
            Context::ControlsFacts(HtmlControlFactsError::Capture(capture())),
            Some(FAULT),
        ),
        (
            Context::ControlsFacts(HtmlControlFactsError::Capture(no_fault_capture())),
            None,
        ),
        (
            Context::ImageDimensionsFacts(HtmlImageDimensionFactsError::RecordLimit),
            None,
        ),
        (Context::HostRevisionMismatch, None),
    ] {
        assert_eq!(error.accounting_fault(), expected, "{error:?}");
    }
}

#[test]
fn an_option_value_control_error_has_a_fault_only_from_its_accounting_variant() {
    assert_eq!(
        HtmlOptionValueControlError::Accounting(FAULT).accounting_fault(),
        Some(FAULT)
    );
    assert_eq!(
        HtmlOptionValueControlError::Cancelled.accounting_fault(),
        None
    );
    assert_eq!(
        HtmlOptionValueControlError::RepresentationExhausted.accounting_fault(),
        None
    );
}

#[test]
fn a_controlled_option_value_error_finds_the_fault_through_every_nesting() {
    use HtmlControlledOptionValueError as Error;
    for (error, expected) in [
        (
            Error::Control(HtmlOptionValueControlError::Accounting(FAULT)),
            Some(FAULT),
        ),
        (Error::Capture(capture()), Some(FAULT)),
        (
            Error::ElementValues(HtmlCaptureElementError::Capture(capture())),
            Some(FAULT),
        ),
        (
            Error::Context(HtmlSemanticContextError::ControlsFacts(
                HtmlControlFactsError::Capture(capture()),
            )),
            Some(FAULT),
        ),
        (Error::Capture(no_fault_capture()), None),
        (Error::Control(HtmlOptionValueControlError::Cancelled), None),
        (Error::AllocationFailed, None),
        (Error::InvalidPhase, None),
    ] {
        assert_eq!(error.accounting_fault(), expected, "{error:?}");
    }
}

#[test]
fn a_semantic_failure_kind_finds_the_fault_through_every_nesting() {
    use HtmlSemanticFailureKind as Kind;
    for (kind, expected) in [
        (Kind::Accounting(FAULT), Some(FAULT)),
        (Kind::Capture(capture()), Some(FAULT)),
        (
            Kind::ControlledOptionValue(HtmlOptionValueControlError::Accounting(FAULT)),
            Some(FAULT),
        ),
        (Kind::Capture(no_fault_capture()), None),
        (Kind::AllocationFailed, None),
        (Kind::ProtocolViolated, None),
        (Kind::RepresentationExhausted, None),
    ] {
        assert_eq!(kind.accounting_fault(), expected, "{kind:?}");
    }
}

#[test]
fn a_registry_formation_error_finds_the_fault_through_its_accounting_and_capture_variants() {
    type Error = HtmlRegistryFormationError<Infallible>;
    assert_eq!(Error::Accounting(FAULT).accounting_fault(), Some(FAULT));
    assert_eq!(Error::Capture(capture()).accounting_fault(), Some(FAULT));
    assert_eq!(Error::Capture(no_fault_capture()).accounting_fault(), None);
    assert_eq!(Error::AllocationFailed.accounting_fault(), None);
    assert_eq!(
        Error::FormationBytes {
            limit: 1,
            required: 2
        }
        .accounting_fault(),
        None
    );
}

#[test]
fn the_capture_wrappers_find_the_fault_through_their_capture_error() {
    assert_eq!(
        HtmlCaptureElementError::<Infallible>::Capture(capture()).accounting_fault(),
        Some(FAULT)
    );
    assert_eq!(
        HtmlCaptureElementError::<()>::Interrupted(()).accounting_fault(),
        None
    );
    assert_eq!(
        HtmlCaptureEdgeError::<Infallible>::Capture(capture()).accounting_fault(),
        Some(FAULT)
    );
    assert_eq!(
        HtmlCaptureEdgeError::<()>::Interrupted(()).accounting_fault(),
        None
    );
    assert_eq!(
        HtmlHyperlinkCaptureError::<Infallible>::Capture(capture()).accounting_fault(),
        Some(FAULT)
    );
    assert_eq!(
        HtmlHyperlinkCaptureError::<()>::Interrupted(()).accounting_fault(),
        None
    );
}

#[test]
fn the_encoding_url_errors_find_the_fault_through_their_base_facts_error() {
    let formation = |error| HtmlEncodingUrlFactsError::Formation(error);
    assert_eq!(
        formation(HtmlBaseFactsError::Accounting(FAULT)).accounting_fault(),
        Some(FAULT)
    );
    assert_eq!(
        formation(HtmlBaseFactsError::Capture(capture())).accounting_fault(),
        Some(FAULT)
    );
    assert_eq!(
        formation(HtmlBaseFactsError::Capture(no_fault_capture())).accounting_fault(),
        None
    );
    assert_eq!(
        formation(HtmlBaseFactsError::AllocationFailed).accounting_fault(),
        None
    );
    assert_eq!(
        HtmlEncodingUrlFactsError::ForeignProducer.accounting_fault(),
        None
    );
    assert_eq!(
        HtmlEncodingUrlLookupError::<Infallible>::Facts(formation(HtmlBaseFactsError::Accounting(
            OTHER
        )))
        .accounting_fault(),
        Some(OTHER)
    );
    assert_eq!(
        HtmlEncodingUrlLookupError::<()>::Interrupted(()).accounting_fault(),
        None
    );
}
