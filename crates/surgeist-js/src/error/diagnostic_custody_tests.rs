use std::{cell::Cell, rc::Rc};

use super::{EngineError, JsError, JsNativeError, PanicError};
use crate::{Context, JsObject};

const PANIC_MESSAGE: &str = "diagnostic custody failure";
const FOREIGN_REALM_MESSAGE: &str = "error realm belongs to a different runtime";

#[derive(crate::JsData, surgeist_js_gc::heap::Trace)]
struct DisposalProbe(#[heap_trace(skip)] Rc<Cell<usize>>);

impl Drop for DisposalProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn panic_with_foreign_parent_and_local_cause(
    foreign: &Context,
    receiving: &Context,
    drops: &Rc<Cell<usize>>,
) -> JsError {
    let cause = JsObject::new_unique(receiving.runtime(), None, DisposalProbe(drops.clone()))
        .unwrap()
        .upcast();
    let diagnostic = JsNativeError::range()
        .with_message("foreign diagnostic parent")
        .with_realm(foreign.realm().clone())
        .with_cause(JsError::from_opaque(cause.into()).unwrap());
    PanicError::new(PANIC_MESSAGE)
        .with_source(diagnostic)
        .into()
}

fn assert_foreign_parent_was_replaced(error: &JsError) {
    let expected = JsError::from(
        PanicError::new(PANIC_MESSAGE)
            .with_source(JsNativeError::typ().with_message(FOREIGN_REALM_MESSAGE)),
    );
    assert_eq!(error, &expected);
    let Some(EngineError::Panic(panic)) = error.as_engine() else {
        panic!("diagnostic admission must preserve the outer panic")
    };
    assert_eq!(panic.message(), PANIC_MESSAGE);
}

#[test]
fn admission_replaces_a_foreign_diagnostic_parent_and_releases_its_local_cause() {
    let foreign = Context::builder().build().unwrap();
    let receiving = Context::builder().build().unwrap();
    let drops = Rc::new(Cell::new(0));
    let source = panic_with_foreign_parent_and_local_cause(&foreign, &receiving, &drops);

    receiving.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "the unadmitted diagnostic still owns its local opaque cause"
    );

    let admitted = source.admit_runtime(receiving.runtime());
    assert_foreign_parent_was_replaced(&admitted);
    receiving.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        1,
        "replacing the whole foreign parent releases its hidden local cause"
    );
    // Keep the admitted panic alive across collection and inspect its unchanged
    // semantic value afterward; it must not retain the removed child as storage.
    assert_foreign_parent_was_replaced(&admitted);
    drop(admitted);
    receiving.runtime().collect().unwrap();
    assert_eq!(drops.get(), 1);
}

#[test]
fn an_unadmitted_alias_keeps_a_removed_diagnostic_cause_until_the_alias_is_dropped() {
    let foreign = Context::builder().build().unwrap();
    let receiving = Context::builder().build().unwrap();
    let drops = Rc::new(Cell::new(0));
    let source = panic_with_foreign_parent_and_local_cause(&foreign, &receiving, &drops);
    let unadmitted_alias = source.clone();
    let admitted = source.admit_runtime(receiving.runtime());

    assert_foreign_parent_was_replaced(&admitted);
    receiving.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "the independent unadmitted alias legitimately keeps the child alive"
    );

    drop(unadmitted_alias);
    receiving.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        1,
        "the admitted panic must not keep the child after its last real owner is dropped"
    );
    assert_foreign_parent_was_replaced(&admitted);
    drop(admitted);
    receiving.runtime().collect().unwrap();
    assert_eq!(drops.get(), 1);
}

use super::{DiagnosticKind, DiagnosticNativeKind, StoredJsError};
use crate::Runtime;
use surgeist_js_gc::heap::{AccessError, AllocationCollection, Rooted};

#[derive(surgeist_js_gc::heap::Trace)]
struct StoredErrorOwner {
    error: StoredJsError,
}

fn publish_stored(runtime: &Runtime, original: &JsError) -> Rooted<StoredErrorOwner> {
    // The external original remains alive until publication has installed every
    // stored edge. The pending-edge owner also protects allocation-time GC.
    runtime
        .heap
        .with_pending_edges(
            StoredErrorOwner {
                error: original.clone().into_stored(),
            },
            |payload| runtime.heap.allocate(payload).unwrap(),
        )
        .unwrap()
}

fn panic_diagnostic(error: &JsError) -> super::Diagnostic<'_> {
    let Some(EngineError::Panic(panic)) = error.as_engine() else {
        panic!("expected an engine panic")
    };
    panic.diagnostic().unwrap()
}

#[test]
fn stored_edges_keep_a_diagnostic_alive_and_restore_it_after_the_old_lease_is_gone() {
    for policy in [
        AllocationCollection::Manual,
        AllocationCollection::BeforeAllocation,
    ] {
        let runtime = Runtime::default();
        runtime.heap.set_allocation_collection(policy).unwrap();
        let drops = Rc::new(Cell::new(0));
        let object = JsObject::new_unique(&runtime, None, DisposalProbe(drops.clone()))
            .unwrap()
            .upcast();
        let identity = object.identity();
        let source: JsError = PanicError::new("stored panic")
            .with_source(
                JsNativeError::range()
                    .with_message("stored source")
                    .with_cause(JsError::from_opaque(object.into()).unwrap()),
            )
            .into();
        let owner = publish_stored(&runtime, &source);
        drop(source);
        runtime.collect().unwrap();
        assert_eq!(
            drops.get(),
            0,
            "the rooted stored owner traces its visible leaf"
        );

        let restored = owner.error.to_error().unwrap();
        let DiagnosticKind::Native(native) = panic_diagnostic(&restored).kind() else {
            unreachable!()
        };
        assert_eq!(native.kind(), DiagnosticNativeKind::Range);
        assert_eq!(native.message(), "stored source");
        let DiagnosticKind::Opaque(leaf) = native.cause().unwrap().kind() else {
            unreachable!()
        };
        assert_eq!(leaf.object_identity(), Some(identity));
        let expected: JsError = PanicError::new("stored panic")
            .with_source(
                JsNativeError::range()
                    .with_message("stored source")
                    .with_cause(JsError::from_opaque(leaf.value().unwrap()).unwrap()),
            )
            .into();
        assert_eq!(restored, expected);
        drop(expected);
        drop(owner);
        runtime.collect().unwrap();
        assert_eq!(
            drops.get(),
            0,
            "explicit restoration installs fresh external custody"
        );
        drop(restored);
        runtime.collect().unwrap();
        assert_eq!(drops.get(), 1);
    }
}

#[derive(crate::JsData, surgeist_js_gc::heap::Trace)]
struct ErrorCycle {
    error: std::cell::RefCell<Option<StoredJsError>>,
    #[heap_trace(skip)]
    drops: Rc<Cell<usize>>,
}

impl Drop for ErrorCycle {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn object_to_stored_panic_cycle_collects_after_its_last_external_error_root() {
    let runtime = Runtime::default();
    runtime
        .heap
        .set_allocation_collection(AllocationCollection::BeforeAllocation)
        .unwrap();
    let drops = Rc::new(Cell::new(0));
    let object = JsObject::new_unique(
        &runtime,
        None,
        ErrorCycle {
            error: std::cell::RefCell::new(None),
            drops: drops.clone(),
        },
    )
    .unwrap();
    let source: JsError = PanicError::new("cycle")
        .with_source(JsError::from_opaque(object.clone().upcast().into()).unwrap())
        .into();
    runtime
        .heap
        .with_pending_edges(source.clone().into_stored(), |stored| {
            *object.borrow().unwrap().data().error.borrow_mut() = Some(stored);
        })
        .unwrap();
    drop(object);
    runtime.collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "the external panic legitimately roots the cycle"
    );
    drop(source);
    runtime.collect().unwrap();
    assert_eq!(
        drops.get(),
        1,
        "persistent diagnostic data must contain only traced edges"
    );
}

#[test]
fn projection_retains_an_identity_that_is_hidden_once_but_visible_in_another_member() {
    let foreign = Context::builder().build().unwrap();
    let receiving = Context::builder().build().unwrap();
    let drops = Rc::new(Cell::new(0));
    let object = JsObject::new_unique(receiving.runtime(), None, DisposalProbe(drops.clone()))
        .unwrap()
        .upcast();
    let identity = object.identity();
    let source: JsError = PanicError::new("duplicate identity")
        .with_source(JsNativeError::aggregate(vec![
            JsNativeError::range()
                .with_realm(foreign.realm().clone())
                .with_cause(JsError::from_opaque(object.clone().into()).unwrap())
                .into(),
            JsError::from_opaque(object.into()).unwrap(),
        ]))
        .into();
    let admitted = source.admit_runtime(receiving.runtime());
    let DiagnosticKind::Native(aggregate) = panic_diagnostic(&admitted).kind() else {
        unreachable!()
    };
    let mut members = aggregate.members();
    let DiagnosticKind::Native(hidden_parent) = members.next().unwrap().kind() else {
        unreachable!()
    };
    assert_eq!(hidden_parent.kind(), DiagnosticNativeKind::Type);
    assert_eq!(hidden_parent.message(), FOREIGN_REALM_MESSAGE);
    assert!(hidden_parent.cause().is_none());
    let DiagnosticKind::Opaque(visible) = members.next().unwrap().kind() else {
        unreachable!()
    };
    assert_eq!(visible.object_identity(), Some(identity));
    receiving.runtime().collect().unwrap();
    assert_eq!(drops.get(), 0);
    drop(admitted);
    receiving.runtime().collect().unwrap();
    assert_eq!(drops.get(), 1);
}

#[test]
fn checked_leaf_access_reports_a_heap_borrow_without_changing_identity_or_type() {
    let runtime = Runtime::default();
    let object = JsObject::with_null_proto(&runtime).unwrap();
    let identity = object.identity();
    let source: JsError = PanicError::new("borrowed heap")
        .with_source(JsError::from_opaque(object.into()).unwrap())
        .into();
    let DiagnosticKind::Opaque(leaf) = panic_diagnostic(&source).kind() else {
        unreachable!()
    };
    runtime
        .heap
        .read(|_| {
            assert_eq!(leaf.object_identity(), Some(identity));
            assert_eq!(leaf.value_type(), crate::value::Type::Object);
            assert!(matches!(leaf.value(), Err(AccessError::HeapBorrowed)));
        })
        .unwrap();
    assert_eq!(
        leaf.value().unwrap().as_object().unwrap().identity(),
        identity
    );
}

#[derive(crate::JsData, surgeist_js_gc::heap::Trace)]
struct CollectReceiverOnDrop {
    #[heap_trace(skip)]
    receiver: Runtime,
    #[heap_trace(skip)]
    local_drops: Rc<Cell<usize>>,
    #[heap_trace(skip)]
    observed: Rc<Cell<Option<usize>>>,
    #[heap_trace(skip)]
    collection_succeeded: Rc<Cell<bool>>,
}

impl Drop for CollectReceiverOnDrop {
    fn drop(&mut self) {
        self.collection_succeeded
            .set(self.receiver.collect().is_ok());
        self.observed.set(Some(self.local_drops.get()));
    }
}

#[test]
fn final_foreign_runtime_disposal_reenters_after_surviving_local_roots_are_protected() {
    let mut foreign = Context::builder().build().unwrap();
    let receiving = Context::builder().build().unwrap();
    let local_drops = Rc::new(Cell::new(0));
    let observed = Rc::new(Cell::new(None));
    let collection_succeeded = Rc::new(Cell::new(false));
    let sentinel = JsObject::new_unique(
        foreign.runtime(),
        None,
        CollectReceiverOnDrop {
            receiver: receiving.runtime().clone(),
            local_drops: local_drops.clone(),
            observed: observed.clone(),
            collection_succeeded: collection_succeeded.clone(),
        },
    )
    .unwrap()
    .upcast();
    foreign
        .global_object()
        .unwrap()
        .set(
            crate::js_string!("diagnosticDisposalSentinel"),
            sentinel,
            true,
            foreign.access(),
        )
        .unwrap();
    let local = JsObject::new_unique(
        receiving.runtime(),
        None,
        DisposalProbe(local_drops.clone()),
    )
    .unwrap()
    .upcast();
    let source: JsError = PanicError::new("foreign release")
        .with_source(JsNativeError::aggregate(vec![
            JsNativeError::range()
                .with_realm(foreign.realm().clone())
                .into(),
            JsError::from_opaque(local.into()).unwrap(),
        ]))
        .into();
    drop(foreign);
    assert_eq!(
        observed.get(),
        None,
        "the original foreign diagnostic still roots its runtime"
    );

    let admitted = source.admit_runtime(receiving.runtime());
    assert!(
        collection_succeeded.get(),
        "final foreign disposal must not retain an engine heap borrow"
    );
    assert_eq!(
        observed.get(),
        Some(0),
        "the new visible local root lease is installed before foreign disposal"
    );
    receiving.runtime().collect().unwrap();
    assert_eq!(local_drops.get(), 0);
    let DiagnosticKind::Native(aggregate) = panic_diagnostic(&admitted).kind() else {
        unreachable!()
    };
    assert!(matches!(
        aggregate.members().next_back().unwrap().kind(),
        DiagnosticKind::Opaque(_)
    ));
    drop(admitted);
    receiving.runtime().collect().unwrap();
    assert_eq!(local_drops.get(), 1);
}

#[test]
fn stored_diagnostics_preserve_all_engine_families_and_exact_context_identity() {
    use crate::error::{EngineResourceError, RuntimeLimitError};
    use crate::job::JobRetirement;
    use surgeist_js_ast::scope::ScopeStateError;
    let context = Context::builder().build().unwrap();
    let mut cases = vec![
        EngineError::Resource(EngineResourceError::Allocation),
        EngineError::Resource(EngineResourceError::SizeOverflow),
        EngineError::InvalidEntryPermit,
        EngineError::ContextAccessBusy,
        EngineError::JobContextRequired {
            required: context.id(),
        },
        EngineError::InvocationIdExhausted,
        EngineError::RuntimeLimit(RuntimeLimitError::Instruction { limit: 9 }),
        EngineError::RuntimeLimit(RuntimeLimitError::JobPreparation { limit: 10 }),
        EngineError::RuntimeLimit(RuntimeLimitError::JobInvocation { limit: 11 }),
        EngineError::RuntimeLimit(RuntimeLimitError::JobPoll { limit: 12 }),
    ];
    cases.extend(
        [
            JobRetirement::Cancelled,
            JobRetirement::OwnerDropped,
            JobRetirement::Interrupted,
            JobRetirement::Dropped,
        ]
        .map(EngineError::JobRetired),
    );
    cases.extend(
        [
            ScopeStateError::Conflict,
            ScopeStateError::IdentityExhausted,
            ScopeStateError::Allocation,
        ]
        .map(EngineError::ScopeAnalysis),
    );
    #[cfg(feature = "fuzz")]
    cases.push(EngineError::NoInstructionsRemain);
    for engine in cases {
        let original: JsError = PanicError::new("stored family")
            .with_source(JsError::from(engine))
            .into();
        let stored = original.clone().into_stored();
        assert_eq!(stored.to_error().unwrap(), original);
    }
}

#[test]
fn deep_causes_and_many_members_survive_clone_storage_and_ordered_restoration() {
    const DEPTH: usize = 128;
    const MEMBERS: usize = 1024;
    let runtime = Runtime::default();
    runtime
        .heap
        .set_allocation_collection(AllocationCollection::BeforeAllocation)
        .unwrap();
    let drops = Rc::new(Cell::new(0));
    let mut identities = Vec::with_capacity(MEMBERS);
    let mut members = Vec::with_capacity(MEMBERS);
    for _ in 0..MEMBERS {
        let object = JsObject::new_unique(&runtime, None, DisposalProbe(drops.clone()))
            .unwrap()
            .upcast();
        identities.push(object.identity());
        members.push(JsError::from_opaque(object.into()).unwrap());
    }
    let mut tree = JsError::from(JsNativeError::aggregate(members).with_message("wide source"));
    for depth in 0..DEPTH {
        tree = JsNativeError::range()
            .with_message(format!("depth {depth}"))
            .with_cause(tree)
            .into();
    }
    let source: JsError = PanicError::new("large diagnostic").with_source(tree).into();
    let aliases: Vec<_> = (0..16).map(|_| source.clone()).collect();
    drop(source);
    runtime.collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "cloned Panic leases retain every real member"
    );
    let owner = publish_stored(&runtime, &aliases[0]);
    drop(aliases);
    runtime.collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "the stored owner traces the complete deep tree"
    );

    let restored = owner.error.to_error().unwrap();
    let mut view = panic_diagnostic(&restored);
    for depth in (0..DEPTH).rev() {
        let DiagnosticKind::Native(native) = view.kind() else {
            panic!("every authored cause remains native")
        };
        assert_eq!(native.kind(), DiagnosticNativeKind::Range);
        assert_eq!(native.message(), format!("depth {depth}"));
        assert_eq!(native.members().len(), 0);
        view = native.cause().unwrap();
    }
    let DiagnosticKind::Native(aggregate) = view.kind() else {
        panic!("the deepest cause retains the authored aggregate")
    };
    assert_eq!(aggregate.kind(), DiagnosticNativeKind::Aggregate);
    assert_eq!(aggregate.message(), "wide source");
    assert!(aggregate.cause().is_none());
    let mut members = aggregate.members();
    assert_eq!(members.len(), MEMBERS);
    for (index, identity) in identities.iter().enumerate() {
        assert_eq!(members.len(), MEMBERS - index);
        let DiagnosticKind::Opaque(leaf) = members.next().unwrap().kind() else {
            panic!("every authored member remains opaque")
        };
        assert_eq!(leaf.object_identity(), Some(*identity));
        assert_eq!(
            leaf.value().unwrap().as_object().unwrap().identity(),
            *identity
        );
    }
    assert!(members.next().is_none());
    assert!(members.next_back().is_none());
    drop(owner);
    runtime.collect().unwrap();
    assert_eq!(drops.get(), 0, "the restored lease retains every member");
    drop(restored);
    runtime.collect().unwrap();
    assert_eq!(
        drops.get(),
        MEMBERS,
        "all members are released exactly once"
    );
}

#[test]
fn masked_stored_diagnostics_trace_and_restore_after_the_hidden_heaps_and_leaves_are_gone() {
    let mut foreign = Context::builder().build().unwrap();
    let mut receiving = Context::builder().build().unwrap();
    receiving
        .runtime()
        .heap
        .set_allocation_collection(AllocationCollection::BeforeAllocation)
        .unwrap();
    let foreign_drops = Rc::new(Cell::new(0));
    let hidden_drops = Rc::new(Cell::new(0));
    let visible_drops = Rc::new(Cell::new(0));
    let sentinel = JsObject::new_unique(
        foreign.runtime(),
        None,
        DisposalProbe(foreign_drops.clone()),
    )
    .unwrap()
    .upcast();
    foreign
        .global_object()
        .unwrap()
        .set(
            crate::js_string!("maskedDiagnosticSentinel"),
            sentinel,
            true,
            foreign.access(),
        )
        .unwrap();
    let hidden = JsObject::new_unique(
        receiving.runtime(),
        None,
        DisposalProbe(hidden_drops.clone()),
    )
    .unwrap()
    .upcast();
    let visible = JsObject::new_unique(
        receiving.runtime(),
        None,
        DisposalProbe(visible_drops.clone()),
    )
    .unwrap()
    .upcast();
    let visible_identity = visible.identity();
    let original: JsError = PanicError::new("masked stored source")
        .with_source(JsNativeError::aggregate(vec![
            JsNativeError::range()
                .with_message("discarded foreign payload")
                .with_realm(foreign.realm().clone())
                .with_cause(JsError::from_opaque(hidden.into()).unwrap())
                .into(),
            JsError::from_opaque(visible.into()).unwrap(),
        ]))
        .into();
    drop(foreign);
    assert_eq!(
        foreign_drops.get(),
        0,
        "the original still owns the foreign realm"
    );
    let admitted = original.admit_runtime(receiving.runtime());
    assert_eq!(
        foreign_drops.get(),
        1,
        "masking releases the final foreign runtime owner"
    );
    receiving.runtime().collect().unwrap();
    assert_eq!(
        hidden_drops.get(),
        1,
        "the hidden local cause is actually gone"
    );
    assert_eq!(visible_drops.get(), 0);
    let owner = publish_stored(receiving.runtime(), &admitted);
    drop(admitted);
    receiving.runtime().collect().unwrap();
    assert_eq!(hidden_drops.get(), 1);
    assert_eq!(
        visible_drops.get(),
        0,
        "the real stored owner traces only visible custody"
    );

    let restored = owner.error.to_error().unwrap();
    let DiagnosticKind::Native(aggregate) = panic_diagnostic(&restored).kind() else {
        unreachable!()
    };
    let mut members = aggregate.members();
    assert_eq!(members.len(), 2);
    let DiagnosticKind::Native(replacement) = members.next().unwrap().kind() else {
        unreachable!()
    };
    assert_eq!(replacement.kind(), DiagnosticNativeKind::Type);
    assert_eq!(replacement.message(), FOREIGN_REALM_MESSAGE);
    assert_eq!(replacement.realm_identity(), None);
    assert!(replacement.cause().is_none());
    assert_eq!(replacement.members().len(), 0);
    let DiagnosticKind::Opaque(visible) = members.next().unwrap().kind() else {
        unreachable!()
    };
    assert_eq!(visible.object_identity(), Some(visible_identity));
    assert!(members.next().is_none());
    let expected: JsError = PanicError::new("masked stored source")
        .with_source(JsNativeError::aggregate(vec![
            JsNativeError::typ()
                .with_message(FOREIGN_REALM_MESSAGE)
                .into(),
            JsError::from_opaque(visible.value().unwrap()).unwrap(),
        ]))
        .into();
    assert_eq!(restored, expected);
    assert_eq!(
        restored.clone().into_erased(receiving.access()),
        expected.into_erased(receiving.access())
    );
    assert!(!format!("{restored:?}").contains("discarded foreign payload"));
    drop(owner);
    receiving.runtime().collect().unwrap();
    assert_eq!(
        visible_drops.get(),
        0,
        "restoration acquires independent visible roots"
    );
    drop(restored);
    receiving.runtime().collect().unwrap();
    assert_eq!(foreign_drops.get(), 1);
    assert_eq!(hidden_drops.get(), 1);
    assert_eq!(visible_drops.get(), 1);
}
