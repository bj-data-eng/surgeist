//! Message text and `Error` contract of the input, provenance and DOM host
//! adapter error types.
//!
//! Each expected message is written from the documented meaning of the variant
//! and from every place the crate raises it. The lane names printed by the
//! input and queue errors are pinned by earlier tests; the cases here keep
//! them and add the other types. A wrapping variant states only its own
//! context and returns the wrapped error from `source()`.
#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    Dom, DomBatch, DomError,
    capture::{DomHostDomains, DomHostError},
    identity::{DocumentId, NodeId},
    limits::DomLimits,
};
use surgeist_runtime::{
    ElementId, ElementPhase, ElementRegistration, InputProvenance, ProvenanceErrorCode, RootId,
    Runtime, RuntimeDomHostError, RuntimeQueuePolicy, ServiceId, ServiceInput,
    SurfaceDomBindingLimits, SurfaceElementRef, SurfaceErrorCode, SurfaceGeneration, SurfaceId,
    SurfaceRef, SurfaceRoot, TaskInput, TaskIntentAttemptId, TaskIntentId, UiInput, UiSurface,
    WindowId,
};

/// Checks one type's table: row order against `index` (an exhaustive match where the type allows one), the exact
/// message, no trailing period, no `Debug` form, and `source()` present exactly
/// for the wrapping variants. `skipped` lists row numbers whose payload cannot be
/// built through the public API.
fn check_table<E>(
    rows: Vec<(E, &'static str)>,
    index: fn(&E) -> usize,
    wraps: fn(&E) -> bool,
    skipped: &[usize],
) where
    E: Error + 'static + Debug,
{
    let mut position = 0;
    for (value, expected) in &rows {
        while skipped.contains(&position) {
            position += 1;
        }
        assert_eq!(index(value), position, "row order for {value:?}");
        position += 1;
        let text = value.to_string();
        assert_eq!(&text, expected, "message for {value:?}");
        assert!(!text.ends_with('.'), "no trailing period: {text}");
        assert!(
            text.chars().next().unwrap().is_lowercase(),
            "a lowercase clause: {text}"
        );
        assert_ne!(text, format!("{value:?}"), "message is the Debug form");
        assert_eq!(
            value.source().is_some(),
            wraps(value),
            "source presence for {value:?}"
        );
    }
}

/// `source()` returns exactly the wrapped error and the message does not repeat
/// its text.
fn assert_chains<E: Error>(outer: &E, inner: &dyn Error) {
    let source = outer.source().expect("a wrapping variant has a source");
    assert_eq!(source.to_string(), inner.to_string());
    assert!(
        !outer.to_string().contains(&inner.to_string()),
        "the wrapped error is reported by source(), not repeated: {outer}"
    );
}

fn assert_send_sync<E: Error + Send + Sync + 'static>() {}

fn surface_ref() -> SurfaceRef {
    SurfaceRef::new(SurfaceId::from_u64(3), SurfaceGeneration::from_u64(1))
}

// Provenance.

#[test]
fn a_rejected_surface_attachment_says_why_the_origin_cannot_carry_it() {
    assert_send_sync::<surgeist_runtime::ProvenanceError>();
    let first = surface_ref();
    let second = SurfaceRef::new(SurfaceId::from_u64(4), SurfaceGeneration::from_u64(1));
    let task = task_provenance;

    let error = task()
        .try_with_surface(first)
        .unwrap()
        .try_with_surface(second)
        .unwrap_err();
    assert_eq!(error.code(), ProvenanceErrorCode::SurfaceAlreadyAttached);
    assert_eq!(
        error.to_string(),
        "surface attachment rejected: the task origin already has a different surface attached"
    );

    for provenance in [
        InputProvenance::ui(first),
        InputProvenance::adapter(first),
        InputProvenance::window(first),
    ] {
        let error = provenance.try_with_surface(second).unwrap_err();
        assert_eq!(
            error.code(),
            ProvenanceErrorCode::SurfaceOverwriteUnsupported
        );
        assert_eq!(
            error.to_string(),
            "surface attachment rejected: a UI, adapter or window origin cannot be attached to a different surface"
        );
    }

    for provenance in [InputProvenance::system(), service_provenance()] {
        let error = provenance.try_with_surface(first).unwrap_err();
        assert_eq!(error.code(), ProvenanceErrorCode::SurfaceUnsupportedOrigin);
        assert_eq!(
            error.to_string(),
            "surface attachment rejected: a system or service origin cannot carry a surface attachment"
        );
        assert!(error.source().is_none());
    }
}

// Lane errors keep their pinned text.

fn task_provenance() -> InputProvenance {
    InputProvenance::task(TaskIntentId::from_u64(1), TaskIntentAttemptId::from_u64(1))
}

fn service_provenance() -> InputProvenance {
    InputProvenance::service(ServiceId::new("indexer"))
}

#[test]
fn input_errors_name_the_lane_that_rejected_the_provenance() {
    assert_eq!(
        UiInput::<()>::new((), task_provenance())
            .unwrap_err()
            .to_string(),
        "runtime input has invalid provenance for Ui lane"
    );
    assert_eq!(
        TaskInput::<()>::new((), InputProvenance::system())
            .unwrap_err()
            .to_string(),
        "runtime input has invalid provenance for Task lane"
    );
    assert_eq!(
        ServiceInput::<()>::new((), InputProvenance::system())
            .unwrap_err()
            .to_string(),
        "runtime input has invalid provenance for Service lane"
    );
}

#[test]
fn queue_overflow_names_the_lane_and_the_capacity() {
    let mut runtime =
        Runtime::<(), (), ()>::new_with_queue_policy((), (), RuntimeQueuePolicy::new(0, 0, 0));
    assert_eq!(
        runtime
            .enqueue_ui(UiInput::new((), InputProvenance::system()).unwrap())
            .unwrap_err()
            .to_string(),
        "runtime Ui queue overflow at capacity 0"
    );
    assert_eq!(
        runtime
            .enqueue_task(TaskInput::new((), task_provenance()).unwrap())
            .unwrap_err()
            .to_string(),
        "runtime Task queue overflow at capacity 0"
    );
    assert_eq!(
        runtime
            .enqueue_service(ServiceInput::new((), service_provenance()).unwrap())
            .unwrap_err()
            .to_string(),
        "runtime Service queue overflow at capacity 0"
    );
}

// The DOM host adapter.

fn document() -> (Dom, DocumentId, [NodeId; 2]) {
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            4096,
            128,
            0,
            0,
            16,
            65536,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let a = batch.create_element(&document, "A");
    let b = batch.create_element(&document, "B");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        [
            commit.created(&a).unwrap().unwrap(),
            commit.created(&b).unwrap().unwrap(),
        ],
    )
}

fn runtime(ids: &[u64]) -> (Runtime, SurfaceRef, Vec<SurfaceElementRef>) {
    let mut root = SurfaceRoot::new(RootId::new("root"));
    for &id in ids {
        root.register_element(
            ElementRegistration::try_new(ElementId::from_u64(id), [ElementPhase::Target]).unwrap(),
        )
        .unwrap();
    }
    let mut runtime = Runtime::new((), ());
    let reference = runtime
        .register_surface(
            UiSurface::try_new(SurfaceId::from_u64(37), WindowId::from_u64(91), root).unwrap(),
        )
        .unwrap();
    let elements = ids
        .iter()
        .map(|&id| SurfaceElementRef::new(reference, ElementId::from_u64(id)))
        .collect();
    (runtime, reference, elements)
}

fn limits() -> SurfaceDomBindingLimits {
    SurfaceDomBindingLimits::new(32, 1024)
}

#[test]
fn dom_host_errors_state_the_rejected_adapter_operation() {
    assert_send_sync::<RuntimeDomHostError>();
    let (dom, id, [a, _]) = document();
    let snapshot = dom.snapshot();
    let stamp = snapshot.document(id).unwrap().stamp();
    let (runtime, surface, elements) = runtime(&[5]);
    let unknown = runtime
        .bind_dom_host(
            surface,
            snapshot.document(id).unwrap(),
            vec![(SurfaceElementRef::new(surface, ElementId::from_u64(600)), a)],
            DomHostDomains::Focus,
            limits(),
        )
        .unwrap_err();
    let RuntimeDomHostError::Surface(surface_error) = &unknown else {
        panic!("an unknown surface element is a surface error: {unknown:?}");
    };
    assert_eq!(surface_error.code(), SurfaceErrorCode::UnknownElement);
    check_table(
        vec![
            (
                RuntimeDomHostError::UnsupportedDomains(DomHostDomains::DocumentTarget),
                "the Runtime has no producer for the requested external domain configuration",
            ),
            (
                RuntimeDomHostError::ForeignRuntime,
                "the DOM host adapter belongs to another Runtime instance",
            ),
            (
                unknown,
                "the source surface failed validation for the DOM host operation",
            ),
            (
                RuntimeDomHostError::Host(DomHostError::ForeignOwner),
                "the canonical DOM host validation refused the operation",
            ),
            (
                RuntimeDomHostError::BindingDomBasisMismatch {
                    expected: stamp,
                    actual: stamp,
                },
                "the supplied DOM basis differs from the basis admitted by bind or rebind",
            ),
            (
                RuntimeDomHostError::DomSubject {
                    node: a,
                    error: DomError::WrongNodeKind,
                },
                "an association names a DOM node that is not a valid element of the bound document",
            ),
            (
                RuntimeDomHostError::MissingAssociation(elements[0]),
                "a registered source element has no canonical DOM association",
            ),
            (
                RuntimeDomHostError::DuplicateSurfaceElement(elements[0]),
                "a source element reference occurs more than once in the associations",
            ),
            (
                RuntimeDomHostError::DuplicateDomElement(a),
                "a canonical DOM element occurs more than once in the associations",
            ),
            (
                RuntimeDomHostError::AssociationCapacity {
                    limit: 4,
                    required: 9,
                },
                "the association allocation needs capacity 9 but the limit is 4",
            ),
            (
                RuntimeDomHostError::ValidationWork {
                    limit: 4,
                    required: 5,
                },
                "validation would need 5 work credits but the limit is 4",
            ),
            (
                RuntimeDomHostError::RepresentationExhausted,
                "validation work accounting overflowed",
            ),
        ],
        |error| match error {
            RuntimeDomHostError::UnsupportedDomains(_) => 0,
            RuntimeDomHostError::ForeignRuntime => 1,
            RuntimeDomHostError::Surface(_) => 2,
            RuntimeDomHostError::Host(_) => 3,
            RuntimeDomHostError::BindingDomBasisMismatch { .. } => 4,
            RuntimeDomHostError::DomSubject { .. } => 5,
            RuntimeDomHostError::MissingAssociation(_) => 6,
            RuntimeDomHostError::DuplicateSurfaceElement(_) => 7,
            RuntimeDomHostError::DuplicateDomElement(_) => 8,
            RuntimeDomHostError::AssociationCapacity { .. } => 9,
            RuntimeDomHostError::ValidationWork { .. } => 10,
            RuntimeDomHostError::RepresentationExhausted => 11,
        },
        |error| {
            matches!(
                error,
                RuntimeDomHostError::Surface(_)
                    | RuntimeDomHostError::Host(_)
                    | RuntimeDomHostError::DomSubject { .. }
            )
        },
        &[],
    );
    assert_chains(
        &RuntimeDomHostError::Host(DomHostError::ForeignOwner),
        &DomHostError::ForeignOwner,
    );
    assert_chains(
        &RuntimeDomHostError::DomSubject {
            node: a,
            error: DomError::WrongNodeKind,
        },
        &DomError::WrongNodeKind,
    );
}

#[test]
fn binding_a_dom_host_reports_the_rejected_association_or_budget() {
    let (mut dom, id, [a, b]) = document();
    let mut batch = DomBatch::new();
    let text = batch.create_text(id, "not an element");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let (runtime, surface, elements) = runtime(&[5, 8]);
    let bind = |associations: Vec<(SurfaceElementRef, NodeId)>,
                domains: DomHostDomains,
                limits: SurfaceDomBindingLimits| {
        runtime
            .bind_dom_host(surface, document, associations, domains, limits)
            .unwrap_err()
    };

    let error = bind(vec![(elements[0], a)], DomHostDomains::Focus, limits());
    assert_eq!(
        error.to_string(),
        "a registered source element has no canonical DOM association"
    );
    let error = bind(
        vec![(elements[0], a), (elements[0], b)],
        DomHostDomains::Focus,
        limits(),
    );
    assert_eq!(
        error.to_string(),
        "a source element reference occurs more than once in the associations"
    );
    let error = bind(
        vec![(elements[0], a), (elements[1], a)],
        DomHostDomains::Focus,
        limits(),
    );
    assert_eq!(
        error.to_string(),
        "a canonical DOM element occurs more than once in the associations"
    );
    let error = bind(
        vec![(elements[0], text), (elements[1], b)],
        DomHostDomains::Focus,
        limits(),
    );
    assert_eq!(
        error.to_string(),
        "an association names a DOM node that is not a valid element of the bound document"
    );
    let error = bind(
        vec![],
        DomHostDomains::DocumentTarget,
        SurfaceDomBindingLimits::new(0, 0),
    );
    assert_eq!(
        error.to_string(),
        "the Runtime has no producer for the requested external domain configuration"
    );
    let error = bind(
        vec![(elements[0], a), (elements[1], b)],
        DomHostDomains::Focus,
        SurfaceDomBindingLimits::new(0, 1024),
    );
    assert!(
        matches!(error, RuntimeDomHostError::AssociationCapacity { .. }),
        "{error:?}"
    );
    assert_eq!(
        error.to_string(),
        "the association allocation needs capacity 2 but the limit is 0"
    );
    let error = bind(
        vec![(elements[0], a), (elements[1], b)],
        DomHostDomains::Focus,
        SurfaceDomBindingLimits::new(32, 1),
    );
    assert!(
        matches!(error, RuntimeDomHostError::ValidationWork { .. }),
        "{error:?}"
    );
    assert!(
        error.to_string().starts_with("validation would need "),
        "{error}"
    );
}

#[test]
fn another_runtime_may_not_use_a_dom_host_adapter() {
    let (dom, id, [a, _]) = document();
    let snapshot = dom.snapshot();
    let document = snapshot.document(id).unwrap();
    let (first, surface, elements) = runtime(&[12]);
    let (second, _, _) = runtime(&[12]);
    let mut host = first
        .bind_dom_host(
            surface,
            document,
            vec![(elements[0], a)],
            DomHostDomains::Focus,
            limits(),
        )
        .unwrap();
    let error = second.capture_dom_host(&mut host, document).unwrap_err();
    assert_eq!(error, RuntimeDomHostError::ForeignRuntime);
    assert_eq!(
        error.to_string(),
        "the DOM host adapter belongs to another Runtime instance"
    );
}

#[test]
fn capturing_after_a_canonical_change_reports_the_basis_mismatch() {
    let (mut dom, id, [a, _]) = document();
    let old = dom.snapshot();
    let old_document = old.document(id).unwrap();
    let (runtime, surface, elements) = runtime(&[11]);
    let mut host = runtime
        .bind_dom_host(
            surface,
            old_document,
            vec![(elements[0], a)],
            DomHostDomains::Focus,
            limits(),
        )
        .unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "key", "value");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let new = dom.snapshot();
    let document = new.document(id).unwrap();
    let error = runtime.capture_dom_host(&mut host, document).unwrap_err();
    assert!(matches!(
        error,
        RuntimeDomHostError::BindingDomBasisMismatch { .. }
    ));
    assert_eq!(
        error.to_string(),
        "the supplied DOM basis differs from the basis admitted by bind or rebind"
    );
    runtime
        .rebind_dom_host(&mut host, document, vec![(elements[0], a)], limits())
        .unwrap();
    let error = runtime
        .rebind_dom_host(&mut host, old_document, vec![(elements[0], a)], limits())
        .unwrap_err();
    assert!(matches!(error, RuntimeDomHostError::Host(_)), "{error:?}");
    assert_eq!(
        error.to_string(),
        "the canonical DOM host validation refused the operation"
    );
}
