//! Native construction and the controlled autocomplete materialization charging a scope that a
//! host supplies.

use crate::html::support::{construction_limits, html_element};
use surgeist_html::{
    accounting::{HtmlAccountingFault, HtmlHostScope, HtmlLimits, HtmlResource, HtmlWork},
    attributes::HtmlInputType,
    bounded::attributes::{
        HtmlAutocompleteMaterializationError, HtmlAutocompleteMaterializationLimits,
    },
    construction::{
        HtmlConstructionCause, HtmlConstructionLimits, HtmlConstructionResource, HtmlDocument,
        HtmlDocumentMode, HtmlFragment,
    },
    values::{Attribute, HtmlValueInterpreter, OrderedAttributes},
    vocabulary::{KnownHtmlAttribute, KnownHtmlElement},
};
use surgeist_ledger::{Limits, Operation, Scope, SharedOperation, Usage};

type Ledger = SharedOperation<HtmlWork, HtmlResource>;

/// A host operation whose root carries `limits`, with a scope bound of `scopes`.
fn host(limits: HtmlLimits, scopes: usize) -> (Ledger, HtmlHostScope) {
    let operation = Operation::new("host", limits, scopes).unwrap();
    let root = operation.root();
    let ledger = operation.share();
    let scope = HtmlHostScope::new(ledger.clone(), root);
    (ledger, scope)
}

/// Caps on one resource of the host's root and none elsewhere.
fn cap(resource: HtmlResource, limit: u64) -> HtmlLimits {
    Limits::from_fn(
        |_| u64::MAX,
        |dimension| {
            if dimension == resource {
                limit
            } else {
                u64::MAX
            }
        },
    )
}

fn usage(ledger: &Ledger, scope: Scope) -> Usage<HtmlWork, HtmlResource> {
    ledger.usage(scope).unwrap()
}

#[test]
fn a_hosted_builder_charges_the_host_scope_and_the_host_keeps_it_after_the_result_drops() {
    let (ledger, host) = host(HtmlLimits::unlimited(), 2);
    let mut builder = HtmlFragment::builder_with_accounting(construction_limits(), &host).unwrap();
    let root = builder.root();
    builder
        .append_element(&root, html_element("div"), OrderedAttributes::default())
        .unwrap();
    builder
        .append_element(&root, html_element("span"), OrderedAttributes::default())
        .unwrap();
    let admitted = builder.usage();
    // The root and two elements.
    assert_eq!(admitted.nodes(), 3);
    let fragment = builder.finish().unwrap();
    assert_eq!(fragment.usage(), admitted);
    let live = |ledger: &Ledger| usage(ledger, host.scope()).live(HtmlResource::ManagedNodes);
    assert_eq!(live(&ledger), 3);
    drop(fragment);
    assert_eq!(live(&ledger), 3, "the host operation keeps the charge");
    let held = usage(&ledger, host.scope());
    assert_eq!(held.live(HtmlResource::ManagedEdges), 2);
}

#[test]
fn a_hosted_document_builder_opens_one_construction_scope_below_the_host_scope() {
    let (_, host) = host(HtmlLimits::unlimited(), 2);
    HtmlDocument::builder_with_accounting(construction_limits(), HtmlDocumentMode::NoQuirks, &host)
        .unwrap();
    // The bound of two holds the root and the builder's scope, so a second builder finds no room.
    let error = HtmlDocument::builder_with_accounting(
        construction_limits(),
        HtmlDocumentMode::Quirks,
        &host,
    )
    .unwrap_err();
    assert_eq!(
        error.accounting_fault(),
        Some(HtmlAccountingFault::ScopeLimit { limit: 2 })
    );
}

#[test]
fn a_cap_of_the_host_scope_refuses_an_append_with_the_hosts_limit_and_releases_the_charge() {
    let (ledger, host) = host(cap(HtmlResource::ManagedNodes, 2), 2);
    let mut builder = HtmlFragment::builder_with_accounting(construction_limits(), &host).unwrap();
    let root = builder.root();
    builder
        .append_element(&root, html_element("div"), OrderedAttributes::default())
        .unwrap();
    let before = usage(&ledger, host.scope());
    let error = builder
        .append_element(&root, html_element("span"), OrderedAttributes::default())
        .unwrap_err();
    assert_eq!(
        error.cause(),
        HtmlConstructionCause::LimitExceeded {
            resource: HtmlConstructionResource::Nodes,
            limit: 2,
            attempted: 3
        }
    );
    assert_eq!(usage(&ledger, host.scope()), before);
    assert_eq!(builder.usage().nodes(), 2);
}

#[test]
fn the_tighter_of_the_builders_limit_and_the_host_cap_refuses() {
    let (_, host) = host(cap(HtmlResource::ManagedNodes, 100), 2);
    let limits = HtmlConstructionLimits::new(2, 100, 100, 100_000, 100);
    let mut builder = HtmlFragment::builder_with_accounting(limits, &host).unwrap();
    let root = builder.root();
    builder
        .append_element(&root, html_element("div"), OrderedAttributes::default())
        .unwrap();
    let error = builder
        .append_element(&root, html_element("span"), OrderedAttributes::default())
        .unwrap_err();
    assert!(matches!(
        error.cause(),
        HtmlConstructionCause::LimitExceeded {
            limit: 2,
            attempted: 3,
            ..
        }
    ));
}

#[test]
fn a_host_operation_without_room_for_the_scope_stops_the_builder_with_the_scope_fault() {
    let (_, host) = host(HtmlLimits::unlimited(), 1);
    let error = HtmlFragment::builder_with_accounting(construction_limits(), &host).unwrap_err();
    assert_eq!(
        error.accounting_fault(),
        Some(HtmlAccountingFault::ScopeLimit { limit: 1 })
    );
}

#[test]
fn a_scope_of_another_operation_stops_the_builder_with_the_foreign_scope_fault() {
    let (_, host) = host(HtmlLimits::unlimited(), 2);
    let other = Operation::<HtmlWork, HtmlResource>::new("other", Limits::unlimited(), 1)
        .unwrap()
        .share();
    let foreign = HtmlHostScope::new(other, host.scope());
    let error = HtmlFragment::builder_with_accounting(construction_limits(), &foreign).unwrap_err();
    assert_eq!(
        error.accounting_fault(),
        Some(HtmlAccountingFault::ForeignScope)
    );
}

fn autocomplete() -> (HtmlValueInterpreter, Attribute) {
    (
        HtmlValueInterpreter::new(100),
        Attribute::new(
            KnownHtmlAttribute::Autocomplete.qualified_name(),
            "section-a shipping home tel webauthn",
        ),
    )
}

fn roomy() -> HtmlAutocompleteMaterializationLimits {
    HtmlAutocompleteMaterializationLimits::new(100_000, 100_000, 100_000)
}

#[test]
fn a_hosted_materialization_charges_the_host_what_a_private_one_reports() {
    let (interpreter, raw) = autocomplete();
    let name = KnownHtmlElement::Input.qualified_name();
    let private = interpreter
        .control_autocomplete_controlled(&name, Some(&raw), Some(HtmlInputType::Text), roomy())
        .unwrap();
    let (ledger, host) = host(HtmlLimits::unlimited(), 3);
    let hosted = interpreter
        .control_autocomplete_controlled_with_accounting(
            &name,
            Some(&raw),
            Some(HtmlInputType::Text),
            roomy(),
            &host,
        )
        .unwrap();
    assert_eq!(hosted.usage(), private.usage());
    assert!(private.usage().allocated_bytes() > 0);
    let held = usage(&ledger, host.scope());
    assert_eq!(
        held.live(HtmlResource::AllocatedBytes),
        u64::try_from(private.usage().allocated_bytes()).unwrap()
    );
    assert_eq!(
        held.live(HtmlResource::NestedOutputRecords),
        u64::try_from(private.usage().nested_output_records()).unwrap()
    );
    assert_eq!(
        held.live(HtmlResource::ScratchRecords),
        0,
        "scratch returns to zero at the end of each step"
    );
}

#[test]
fn a_host_byte_cap_below_the_materialization_need_refuses_with_the_hosts_limit() {
    let (interpreter, raw) = autocomplete();
    let name = KnownHtmlElement::Input.qualified_name();
    let needed = interpreter
        .control_autocomplete_controlled(&name, Some(&raw), Some(HtmlInputType::Text), roomy())
        .unwrap()
        .usage()
        .allocated_bytes();
    let cap_bytes = needed - 1;
    let (_, host) = host(
        cap(
            HtmlResource::AllocatedBytes,
            u64::try_from(cap_bytes).unwrap(),
        ),
        3,
    );
    let failure = interpreter
        .control_autocomplete_controlled_with_accounting(
            &name,
            Some(&raw),
            Some(HtmlInputType::Text),
            roomy(),
            &host,
        )
        .unwrap_err();
    let HtmlAutocompleteMaterializationError::BytesLimit { limit, required } = failure.kind()
    else {
        panic!("expected a byte refusal, found {:?}", failure.kind());
    };
    assert_eq!(limit, cap_bytes);
    assert!(required > cap_bytes);
}

#[test]
fn a_host_operation_without_room_for_two_scopes_stops_the_materialization() {
    let (interpreter, raw) = autocomplete();
    let name = KnownHtmlElement::Input.qualified_name();
    let (_, host) = host(HtmlLimits::unlimited(), 2);
    let failure = interpreter
        .control_autocomplete_controlled_with_accounting(
            &name,
            Some(&raw),
            Some(HtmlInputType::Text),
            roomy(),
            &host,
        )
        .unwrap_err();
    assert_eq!(
        failure.kind(),
        HtmlAutocompleteMaterializationError::Accounting(HtmlAccountingFault::ScopeLimit {
            limit: 2
        })
    );
}
