//! Tests of the report of every refusal of HTML's construction storage.
use super::*;
use crate::parser::control::{HtmlParseInvariant, HtmlParseStop};
use surgeist_html::{
    accounting::HtmlAccountingFault,
    construction::{
        HtmlConstructionCause, HtmlConstructionError, HtmlConstructionLimits,
        HtmlConstructionResource, HtmlDocument, HtmlDocumentMode, HtmlFragment,
    },
    values::{Namespace, OrderedAttributes, QualifiedName},
};

fn invariant(invariant: HtmlParseInvariant) -> SessionError {
    SessionError::Stop(HtmlParseStop::InternalInvariant(invariant))
}

fn html_name(local: &str) -> QualifiedName {
    QualifiedName::new(Namespace::html(), None, local)
}

/// The report of `error` and the stop it latched.
fn reported(error: HtmlConstructionError) -> (SessionError, Option<HtmlParseStop>) {
    let mut control = crate::construction::tests::control();
    let session = construction_error(&mut control, error);
    (session, control.stop())
}

/// A handle of another builder and a child under a leaf are document-shaped, as the builder
/// reports them.
#[test]
fn builder_handle_and_parent_refusals_report_document_shaped_session_errors() {
    let limits = HtmlConstructionLimits::new(100, 100, 100, 100_000, 100);
    let other = HtmlFragment::builder(limits).unwrap();
    let mut builder = HtmlFragment::builder(limits).unwrap();
    let foreign = builder.append_text(&other.root(), "x").unwrap_err();
    assert_eq!(reported(foreign), (SessionError::ForeignNode, None));

    let mut builder = HtmlDocument::builder(limits, HtmlDocumentMode::NoQuirks).unwrap();
    let root = builder.root();
    let leaf = builder.append_comment(&root, "").unwrap();
    let under_leaf = builder.append_text(&leaf, "x").unwrap_err();
    assert_eq!(reported(under_leaf), (SessionError::NotContainer, None));
    let template = builder
        .append_element(&root, html_name("template"), OrderedAttributes::default())
        .unwrap();
    let under_template = builder.append_text(&template, "x").unwrap_err();
    assert_eq!(
        reported(under_template),
        (SessionError::TemplateContentsRequired, None)
    );
}

/// A limit the owner reports disagrees with what the controller charged, since the controller
/// admits every resource before the owner is asked.
#[test]
fn an_owner_limit_refusal_latches_the_capacity_mismatch_invariant() {
    let mut builder = HtmlFragment::builder(HtmlConstructionLimits::new(3, 2, 0, 2, 2)).unwrap();
    let root = builder.root();
    let error = builder
        .append_text(&root, DocumentString::from("too long"))
        .unwrap_err();
    let mismatch = HtmlParseInvariant::CapacityMismatch;
    assert_eq!(
        reported(error),
        (
            invariant(mismatch),
            Some(HtmlParseStop::InternalInvariant(mismatch))
        )
    );
}

/// The cause values of HTML's construction owner's error, one per variant. A cause added to
/// the owner's enum does not compile in [`disposition`] until it has a row there, and a row
/// here is what this test file then pins.
fn every_cause() -> [HtmlConstructionCause; 30] {
    use HtmlConstructionCause as C;
    [
        C::ForeignBuilder,
        C::ParentCannotHaveChildren,
        C::TemplateContentsRequired,
        C::ShadowHostMustBeElement,
        C::ShadowAlreadyAttached,
        C::UnknownNode,
        C::WrongNodeKind,
        C::RootMove,
        C::AssociationMove,
        C::WrongSibling,
        C::Cycle,
        C::AssociationAlreadyOwned,
        C::InvalidTemplateAssociation,
        C::DuplicateAttribute {
            first: 0,
            duplicate: 1,
        },
        C::AttributeNameMismatch,
        C::AttributeOrderMismatch,
        C::InvalidShadowAssociation,
        C::InvalidContentsReference,
        C::ParentChildConflict,
        C::InvalidOutputRoot,
        C::OperationNotReady,
        C::ForeignAllocation,
        C::CapacityMismatch,
        C::IndexUnavailable,
        C::SharedAllocationAliased,
        C::ReleaseUnderflow,
        C::LimitExceeded {
            resource: HtmlConstructionResource::Nodes,
            limit: 1,
            attempted: 2,
        },
        C::RepresentationExhausted,
        C::AllocationFailed,
        C::Accounting(HtmlAccountingFault::ForeignScope),
    ]
}

/// Every cause of HTML's construction owner's error, with the error the parser reports for it
/// and whether that error latches a stop.
///
/// - Handle, placement and cycle refusals are document-shaped errors the
///   callers classify themselves and latch nothing.
/// - A shared payload that was aliased and a count that cannot be represented
///   keep their own stops.
/// - A ticket, layout or release that disagrees with what was charged is the
///   charged-accounting fault, and an allocation that failed is a reached
///   environment stop, not an invariant.
/// - A failure of the ledger scope stops with its accounting fault.
/// - Every other refusal is the owner declining a request that the parser's
///   sequencing guarantees valid.
#[test]
fn every_owner_cause_reports_what_it_violates() {
    use HtmlConstructionCause as C;
    let invariant = |invariant| SessionError::Stop(HtmlParseStop::InternalInvariant(invariant));
    let mismatch = invariant(HtmlParseInvariant::CapacityMismatch);
    let refusal = invariant(HtmlParseInvariant::ConstructionOwnerRefusal);
    let allocation_failed = SessionError::Stop(HtmlParseStop::AllocationFailed {
        resource: HtmlParseResource::ManagedCapacityBytes,
        layout_bytes: 0,
    });
    let exhausted = SessionError::Stop(HtmlParseStop::RepresentationExhausted {
        resource: HtmlParseResource::ManagedCapacityBytes,
    });
    let limit = C::LimitExceeded {
        resource: HtmlConstructionResource::Nodes,
        limit: 1,
        attempted: 2,
    };
    let duplicate = C::DuplicateAttribute {
        first: 0,
        duplicate: 1,
    };
    let scope_fault = HtmlAccountingFault::ForeignScope;
    let rows = [
        (C::ForeignBuilder, SessionError::ForeignNode, false),
        (C::UnknownNode, SessionError::ForeignNode, false),
        (C::RootMove, SessionError::RootMove, false),
        (C::AssociationMove, SessionError::AssociationMove, false),
        (
            C::ParentCannotHaveChildren,
            SessionError::NotContainer,
            false,
        ),
        (
            C::TemplateContentsRequired,
            SessionError::TemplateContentsRequired,
            false,
        ),
        (C::WrongSibling, SessionError::WrongSibling, false),
        (C::ParentChildConflict, SessionError::WrongSibling, false),
        (C::Cycle, SessionError::Cycle, false),
        (
            C::SharedAllocationAliased,
            invariant(HtmlParseInvariant::SharedAllocationAliased),
            true,
        ),
        (C::RepresentationExhausted, exhausted, true),
        (C::CapacityMismatch, mismatch, true),
        (C::IndexUnavailable, refusal, true),
        (C::ForeignAllocation, mismatch, true),
        (limit, mismatch, true),
        (
            C::ReleaseUnderflow,
            invariant(HtmlParseInvariant::ReleaseUnderflow),
            true,
        ),
        (C::AllocationFailed, allocation_failed, true),
        (
            C::Accounting(scope_fault),
            SessionError::Stop(HtmlParseStop::Accounting(scope_fault)),
            true,
        ),
        (C::ShadowHostMustBeElement, refusal, true),
        (C::ShadowAlreadyAttached, refusal, true),
        (C::WrongNodeKind, refusal, true),
        (C::AssociationAlreadyOwned, refusal, true),
        (C::InvalidTemplateAssociation, refusal, true),
        (duplicate, refusal, true),
        (C::AttributeNameMismatch, refusal, true),
        (C::AttributeOrderMismatch, refusal, true),
        (C::InvalidShadowAssociation, refusal, true),
        (C::InvalidContentsReference, refusal, true),
        (C::InvalidOutputRoot, refusal, true),
        (C::OperationNotReady, refusal, true),
    ];
    assert_eq!(rows.len(), every_cause().len());
    for cause in every_cause() {
        assert_eq!(
            rows.iter().filter(|(row, _, _)| *row == cause).count(),
            1,
            "{cause:?} has one row"
        );
    }
    for (cause, expected, latched) in rows {
        let mut c = crate::construction::tests::control();
        assert_eq!(report(&mut c, disposition(cause)), expected, "{cause:?}");
        let stop = match expected {
            SessionError::Stop(stop) => Some(stop),
            _ => None,
        };
        assert_eq!(c.stop(), stop.filter(|_| latched), "{cause:?}");
    }
}

/// A failure of the ledger scope that the owner reports stops the parse with that accounting
/// fault, as every other ledger fault of the parser does. The parser's bounded storage cannot
/// produce this cause, so the test reports a constructed one.
#[test]
fn an_owner_accounting_cause_stops_with_its_accounting_fault() {
    for fault in [
        HtmlAccountingFault::ForeignScope,
        HtmlAccountingFault::Allocation,
        HtmlAccountingFault::ReleaseUnderflow,
        HtmlAccountingFault::ScopeLimit { limit: 2 },
    ] {
        let mut c = crate::construction::tests::control();
        let reported = report(
            &mut c,
            disposition(HtmlConstructionCause::Accounting(fault)),
        );
        assert_eq!(
            reported,
            SessionError::Stop(HtmlParseStop::Accounting(fault))
        );
        assert_eq!(c.stop(), Some(HtmlParseStop::Accounting(fault)));
    }
}

/// The construction path has no classification of its own: a document-shaped
/// cause is a session error that `session_class` classifies as a fault of the
/// parser, an accounting fault is the class a stage error would carry, and the
/// error returned for a latched stop is the outcome `session_class` assigns to
/// that stop.
#[test]
fn owner_dispositions_agree_with_the_session_classification() {
    for cause in every_cause() {
        let mut c = crate::construction::tests::control();
        let reported = report(&mut c, disposition(cause));
        match disposition(cause) {
            Disposition::Session(error) => {
                assert_eq!(reported, error, "{cause:?}");
                assert!(
                    matches!(session_class(&error), SessionClass::Fault(_)),
                    "{cause:?}"
                );
                assert_eq!(c.stop(), None, "{cause:?}");
            }
            Disposition::Class(class) => {
                assert!(matches!(class, SessionClass::Fault(_)), "{cause:?}");
                let SessionError::Stop(stop) = reported else {
                    panic!("{cause:?} did not latch a stop");
                };
                assert_eq!(session_class(&reported), SessionClass::Outcome(stop));
                assert_eq!(c.stop(), Some(stop), "{cause:?}");
            }
            Disposition::Representation
            | Disposition::AllocationFailed
            | Disposition::Accounting(_) => {
                let SessionError::Stop(stop) = reported else {
                    panic!("{cause:?} did not latch a stop");
                };
                assert_eq!(session_class(&reported), SessionClass::Outcome(stop));
                assert_eq!(c.stop(), Some(stop), "{cause:?}");
            }
        }
    }
}
