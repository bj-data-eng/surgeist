use super::*;
use crate::{
    JsObject,
    context::time::{JsDuration, JsInstant},
};
use std::{cell::Cell, rc::Rc, time::Duration};
use surgeist_js_gc::heap::{AllocationCollection, Rooted};

fn mismatch(receiving: &Context, selected: &Context) -> RealmOwnershipMismatch {
    match receiving.check_publication_realm(selected.realm()) {
        Ok(()) => panic!("distinct runtimes must refuse a selected realm"),
        Err(mismatch) => mismatch,
    }
}

fn payload(error: &JsError) -> &ForeignSelectedRealm {
    match error.as_engine() {
        Some(EngineError::ForeignSelectedRealm(value)) => value,
        other => panic!("expected a publication realm refusal, got {other:?}"),
    }
}

#[test]
fn receiving_ownership_produces_direct_type_error_and_typed_publication_refusal() {
    let mut receiving = Context::default();
    let selected = Context::default();
    let direct = receiving.check_realm(selected.realm()).unwrap_err();
    assert!(direct.is_catchable());
    assert!(matches!(
        direct.as_native().unwrap().kind,
        JsNativeErrorKind::Type
    ));
    let direct_diagnostic = PanicDiagnostic::new(direct);
    let publication = mismatch(&receiving, &selected).into_publication();
    assert!(!publication.is_catchable());
    let value = payload(&publication);
    assert_eq!(value.selected_realm(), selected.realm().identity());
    assert_eq!(value.receiving_realm(), receiving.realm().identity());
    assert_eq!(value.diagnostic(), direct_diagnostic.view());
    assert!(receiving.check_realm(receiving.realm()).is_ok());
    assert!(receiving.check_publication_realm(receiving.realm()).is_ok());
    let erased = publication.clone().into_erased(receiving.access());
    let Some(ErasedEngineError::ForeignSelectedRealm(erased)) = erased.as_engine() else {
        panic!("erased export must retain the publication kind")
    };
    assert_eq!(erased.selected_realm(), selected.realm().identity());
    assert_eq!(erased.receiving_realm(), receiving.realm().identity());
    assert!(erased.diagnostic().as_native().is_some());
    assert_eq!(
        publication.clone().into_stored().to_error().unwrap(),
        publication
    );
}

#[derive(crate::JsData, surgeist_js_gc::heap::Trace)]
struct Probe(#[heap_trace(skip)] Rc<Cell<usize>>);
impl Drop for Probe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[derive(surgeist_js_gc::heap::Trace)]
struct Owner {
    error: StoredJsError,
}

fn store(runtime: &Runtime, error: &JsError) -> Rooted<Owner> {
    runtime
        .heap
        .with_pending_edges(
            Owner {
                error: error.clone().into_stored(),
            },
            |owner| runtime.heap.allocate(owner).unwrap(),
        )
        .unwrap()
}

// These fixtures exercise the diagnostic adapter's complete data contract;
// production mismatch construction remains exclusively the real owner check.
fn with_diagnostic(receiving: &Context, selected: &Context, original: JsError) -> JsError {
    let mut checked = mismatch(receiving, selected);
    checked.original = original;
    checked.into_publication()
}

#[test]
fn publication_diagnostic_preserves_topology_identity_trace_and_stored_edge_custody() {
    for policy in [
        AllocationCollection::Manual,
        AllocationCollection::BeforeAllocation,
    ] {
        let receiving = Context::default();
        let selected = Context::default();
        let runtime = receiving.runtime();
        runtime.heap.set_allocation_collection(policy).unwrap();
        let drops = Rc::new(Cell::new(0));
        let object = JsObject::new_unique(runtime, None, Probe(drops.clone()))
            .unwrap()
            .upcast();
        let identity = object.identity();
        let mut original: JsError = JsNativeError::aggregate(vec![
            JsNativeError::range().with_message("member").into(),
            JsError::from_opaque(object.into()),
        ])
        .with_message("authored aggregate")
        .with_cause(JsNativeError::syntax().with_message("cause"))
        .into();
        original.backtrace = Some(Backtrace::default());
        let expected = PanicDiagnostic::new(original.clone());
        let error = with_diagnostic(&receiving, &selected, original);
        assert!(
            error.backtrace.is_some(),
            "outer trace capture must survive adaptation"
        );
        assert!(payload(&error).diagnostic().backtrace().is_some());
        assert_eq!(payload(&error).diagnostic(), expected.view());
        let nested = PanicDiagnostic::new(error.clone());
        let DiagnosticKind::Engine(DiagnosticEngine::ForeignSelectedRealm(borrowed)) =
            nested.view().kind()
        else {
            panic!("borrowed publication diagnostic")
        };
        assert_eq!(borrowed.selected_realm(), selected.realm().identity());
        assert_eq!(borrowed.receiving_realm(), receiving.realm().identity());
        assert_eq!(borrowed.diagnostic(), expected.view());
        runtime
            .heap
            .read(|_| {
                let DiagnosticKind::Native(native) = borrowed.diagnostic().kind() else {
                    panic!("borrowed aggregate")
                };
                let DiagnosticKind::Opaque(member) = native.members().nth(1).unwrap().kind() else {
                    panic!("borrowed member")
                };
                assert_eq!(member.object_identity(), Some(identity));
                assert!(matches!(
                    member.value(),
                    Err(surgeist_js_gc::heap::AccessError::HeapBorrowed)
                ));
            })
            .unwrap();
        drop(nested);
        drop(expected);
        let owner = store(runtime, &error);
        drop(error);
        runtime.collect().unwrap();
        assert_eq!(
            drops.get(),
            0,
            "stored edges retain the visible opaque member"
        );
        let restored = owner.error.to_error().unwrap();
        let DiagnosticKind::Native(native) = payload(&restored).diagnostic().kind() else {
            panic!("aggregate")
        };
        assert_eq!(native.kind(), DiagnosticNativeKind::Aggregate);
        assert_eq!(native.message(), "authored aggregate");
        let DiagnosticKind::Native(cause) = native.cause().unwrap().kind() else {
            panic!("native cause")
        };
        assert_eq!(cause.kind(), DiagnosticNativeKind::Syntax);
        assert_eq!(cause.message(), "cause");
        let DiagnosticKind::Opaque(member) = native.members().nth(1).unwrap().kind() else {
            panic!("opaque member")
        };
        assert_eq!(member.object_identity(), Some(identity));
        drop(restored);
        drop(owner);
        runtime.collect().unwrap();
        assert_eq!(
            drops.get(),
            1,
            "stored diagnostics contain no external root bundle"
        );
    }
}

#[test]
fn publication_admission_masks_a_foreign_parent_without_retaining_local_descendants() {
    let receiving = Context::default();
    let selected = Context::default();
    let drops = Rc::new(Cell::new(0));
    let object = JsObject::new_unique(receiving.runtime(), None, Probe(drops.clone()))
        .unwrap()
        .upcast();
    let original = JsNativeError::range()
        .with_message("foreign parent")
        .with_realm(selected.realm().clone())
        .with_cause(JsError::from_opaque(object.into()))
        .into();
    let source = with_diagnostic(&receiving, &selected, original);
    let alias = source.clone();
    let admitted = source.admit_runtime(receiving.runtime());
    let DiagnosticKind::Native(native) = payload(&admitted).diagnostic().kind() else {
        panic!("masked native")
    };
    assert_eq!(native.kind(), DiagnosticNativeKind::Type);
    assert!(native.cause().is_none());
    receiving.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "independent original observation retains its own roots"
    );
    drop(alias);
    receiving.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        1,
        "masked observation must release hidden descendants"
    );
    assert_eq!(admitted.clone().into_stored().to_error().unwrap(), admitted);
}

#[test]
fn deadline_failure_preserves_exact_large_operands_through_all_error_forms() {
    let mut context = Context::default();
    let now = JsInstant::new(u64::MAX, 999_999_999);
    let delay = JsDuration::from(Duration::new(u64::MAX, 123));
    let error: JsError = EngineError::JobDeadlineOverflow { now, delay }.into();
    assert!(!error.is_catchable());
    assert_eq!(error.clone().into_stored().to_error().unwrap(), error);
    let diagnostic = PanicDiagnostic::new(error.clone());
    assert_eq!(
        diagnostic.view().kind(),
        DiagnosticKind::Engine(DiagnosticEngine::JobDeadlineOverflow { now, delay })
    );
    assert_eq!(
        error.into_erased(context.access()).as_engine(),
        Some(&ErasedEngineError::JobDeadlineOverflow { now, delay })
    );
}

#[derive(crate::JsData, surgeist_js_gc::heap::Trace)]
struct Cycle {
    error: std::cell::RefCell<Option<StoredJsError>>,
    #[heap_trace(skip)]
    drops: Rc<Cell<usize>>,
}
impl Drop for Cycle {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn stored_publication_diagnostic_cycle_collects_after_external_observation_drops() {
    let receiving = Context::default();
    let selected = Context::default();
    let runtime = receiving.runtime();
    runtime
        .heap
        .set_allocation_collection(AllocationCollection::BeforeAllocation)
        .unwrap();
    let drops = Rc::new(Cell::new(0));
    let object = JsObject::new_unique(
        runtime,
        None,
        Cycle {
            error: std::cell::RefCell::new(None),
            drops: drops.clone(),
        },
    )
    .unwrap();
    let error = with_diagnostic(
        &receiving,
        &selected,
        JsError::from_opaque(object.clone().upcast().into()),
    );
    runtime
        .heap
        .with_pending_edges(error.clone().into_stored(), |stored| {
            *object.borrow().data().error.borrow_mut() = Some(stored);
        })
        .unwrap();
    drop(object);
    runtime.collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "external publication observation owns the cycle"
    );
    drop(error);
    runtime.collect().unwrap();
    assert_eq!(
        drops.get(),
        1,
        "stored publication diagnostics contain edges, not root bundles"
    );
}

#[test]
fn enclosing_diagnostic_traces_and_masks_the_nested_publication_source() {
    let receiving = Context::default();
    let selected = Context::default();
    let drops = Rc::new(Cell::new(0));
    let object = JsObject::new_unique(receiving.runtime(), None, Probe(drops.clone()))
        .unwrap()
        .upcast();
    let identity = object.identity();
    let publication = with_diagnostic(&receiving, &selected, JsError::from_opaque(object.into()));
    let enclosing: JsError = PanicError::new("enclosing").with_source(publication).into();
    receiving.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "enclosing diagnostics take real source custody"
    );
    let Some(EngineError::Panic(panic)) = enclosing.as_engine() else {
        panic!("enclosing panic")
    };
    let DiagnosticKind::Engine(DiagnosticEngine::ForeignSelectedRealm(value)) =
        panic.diagnostic().unwrap().kind()
    else {
        panic!("nested publication diagnostic")
    };
    let DiagnosticKind::Opaque(leaf) = value.diagnostic().kind() else {
        panic!("nested opaque diagnostic")
    };
    assert_eq!(leaf.object_identity(), Some(identity));

    let masked = enclosing.admit_runtime(selected.runtime());
    receiving.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        1,
        "outer admission releases hidden nested roots"
    );
    let Some(EngineError::Panic(panic)) = masked.as_engine() else {
        panic!("outer kind is preserved")
    };
    let DiagnosticKind::Engine(DiagnosticEngine::ForeignSelectedRealm(value)) =
        panic.diagnostic().unwrap().kind()
    else {
        panic!("publication kind is preserved")
    };
    let DiagnosticKind::Native(replacement) = value.diagnostic().kind() else {
        panic!("foreign opaque leaf is replaced")
    };
    assert_eq!(replacement.kind(), DiagnosticNativeKind::Type);
    assert_eq!(masked.clone().into_stored().to_error().unwrap(), masked);
}
