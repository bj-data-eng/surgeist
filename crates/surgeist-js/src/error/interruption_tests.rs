use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::{
    Diagnostic, DiagnosticEngine, DiagnosticKind, DiagnosticNativeKind, EngineError,
    ErasedEngineError, JsError, JsNativeError, PanicError, StoredJsError,
};
use crate::{
    Context, JsObject, JsValue, Module, NativeFunction, Runtime, Source,
    builtins::promise::PromiseState, execution::ExecutionOptions, js_string,
};
use surgeist_js_gc::heap::{AllocationCollection, ObjectId, Rooted};

thread_local! {
    static CALLBACK_ERROR: RefCell<Option<JsError>> = const { RefCell::new(None) };
    static MODULE_MARKER: RefCell<Option<JsValue>> = const { RefCell::new(None) };
}

#[derive(crate::JsData, surgeist_js_gc::heap::Trace)]
struct DisposalProbe(#[heap_trace(skip)] Rc<Cell<usize>>);

impl Drop for DisposalProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    context
        .register_global_builtin_callable(
            js_string!("stopForCustody"),
            0,
            NativeFunction::from_fn_ptr(|_, _, _| {
                Err(CALLBACK_ERROR.with(|slot| slot.borrow_mut().take().unwrap()))
            }),
        )
        .unwrap();
    context
        .register_global_builtin_callable(
            js_string!("takeModuleMarker"),
            0,
            NativeFunction::from_fn_ptr(|_, _, _| {
                Ok(MODULE_MARKER.with(|slot| slot.borrow_mut().take().unwrap()))
            }),
        )
        .unwrap();
    context
}

fn probe(runtime: &Runtime, drops: &Rc<Cell<usize>>) -> JsObject {
    JsObject::new_unique(runtime, None, DisposalProbe(drops.clone()))
        .unwrap()
        .upcast()
}

fn interrupt(
    original: JsError,
    context: &mut Context,
    module_drops: &Rc<Cell<usize>>,
) -> (Module, JsError) {
    let marker = probe(context.runtime(), module_drops);
    MODULE_MARKER.with(|slot| assert!(slot.replace(Some(marker.into())).is_none()));
    context
        .global_object()
        .unwrap()
        .set(js_string!("custodyEffects"), 0, true, context.access())
        .unwrap();
    let module = Module::parse(Source::from_bytes(
        "export const moduleOnlyMarker = takeModuleMarker(); globalThis.custodyEffects += 1; stopForCustody(); globalThis.custodyEffects += 100;"
    ), None, context.access()).unwrap();
    assert_eq!(
        module.load(context.access()).unwrap().state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    module.link(context.access()).unwrap();
    CALLBACK_ERROR.with(|slot| assert!(slot.replace(Some(original.clone())).is_none()));
    let first = context
        .with_execution(ExecutionOptions::new(100_000), |access| {
            module.evaluate(access)
        })
        .unwrap();
    assert_eq!(first.result.unwrap_err(), original);
    let terminal = context
        .with_execution(ExecutionOptions::new(100_000), |access| {
            module.evaluate(access)
        })
        .unwrap();
    assert_eq!(terminal.report.instructions().unwrap(), 0);
    let terminal = terminal.result.unwrap_err();
    assert_eq!(
        context
            .global_object()
            .unwrap()
            .get(js_string!("custodyEffects"), context.access())
            .unwrap(),
        1.into()
    );
    CALLBACK_ERROR.with(|slot| assert!(slot.borrow().is_none()));
    MODULE_MARKER.with(|slot| assert!(slot.borrow().is_none()));
    (module, terminal)
}

fn identity(error: &JsError) -> ObjectId {
    let Some(EngineError::ModuleInterrupted(value)) = error.as_engine() else {
        panic!("actual terminal module result")
    };
    value.identity()
}

fn cause(error: &JsError) -> Diagnostic<'_> {
    let Some(EngineError::ModuleInterrupted(value)) = error.as_engine() else {
        panic!("actual terminal module result")
    };
    value.cause()
}

fn panic_source(error: &JsError) -> Diagnostic<'_> {
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        message,
        diagnostic: Some(source),
    }) = cause(error).kind()
    else {
        panic!("authored Panic cause")
    };
    assert_eq!(message, "custody source");
    source
}

fn assert_leaf(error: &JsError, expected: ObjectId) {
    let DiagnosticKind::Opaque(leaf) = panic_source(error).kind() else {
        panic!("original opaque leaf")
    };
    assert_eq!(leaf.object_identity(), Some(expected));
    assert_eq!(
        leaf.value().unwrap().as_object().unwrap().identity(),
        expected
    );
}

fn original(leaf: JsObject) -> JsError {
    PanicError::new("custody source")
        .with_source(JsError::from_opaque(leaf.into()).unwrap())
        .into()
}

#[derive(surgeist_js_gc::heap::Trace)]
struct StoredOwner {
    error: StoredJsError,
}

fn store(runtime: &Runtime, original: &JsError) -> Rooted<StoredOwner> {
    // The external source remains alive through publication. Pending edges also
    // cover collection performed by this allocation itself.
    runtime
        .heap
        .with_pending_edges(
            StoredOwner {
                error: original.clone().into_stored(),
            },
            |payload| runtime.heap.allocate(payload).unwrap(),
        )
        .unwrap()
}

#[test]
fn error_only_observation_retains_its_cause_and_module_until_the_final_native_drop() {
    let mut context = context();
    let drops = Rc::new(Cell::new(0));
    let module_drops = Rc::new(Cell::new(0));
    let leaf = probe(context.runtime(), &drops);
    let leaf_identity = leaf.identity();
    let (module, terminal) = interrupt(original(leaf), &mut context, &module_drops);
    let alias = module.clone();
    drop(module);
    drop(alias);
    context.runtime().collect().unwrap();
    assert_eq!(drops.get(), 0);
    assert_eq!(
        module_drops.get(),
        0,
        "the error observation legitimately roots the source module"
    );
    assert_leaf(&terminal, leaf_identity);
    drop(terminal);
    context.runtime().collect().unwrap();
    assert_eq!(drops.get(), 1);
    assert_eq!(module_drops.get(), 1);
}

#[test]
fn stored_owner_restores_the_same_cause_after_the_source_module_environment_is_collected() {
    let mut context = context();
    context
        .runtime()
        .heap
        .set_allocation_collection(AllocationCollection::BeforeAllocation)
        .unwrap();
    let drops = Rc::new(Cell::new(0));
    let module_drops = Rc::new(Cell::new(0));
    let leaf = probe(context.runtime(), &drops);
    let leaf_identity = leaf.identity();
    let (module, terminal) = interrupt(original(leaf), &mut context, &module_drops);
    let interruption_identity = identity(&terminal);
    let owner = store(context.runtime(), &terminal);
    drop(module);
    drop(terminal);
    context.runtime().collect().unwrap();
    assert_eq!(
        module_drops.get(),
        1,
        "the module-only exported binding has no remaining owner"
    );
    assert_eq!(
        drops.get(),
        0,
        "the traced cause anchor independently preserves its leaf"
    );
    let restored = owner.error.to_error().unwrap();
    assert_eq!(identity(&restored), interruption_identity);
    assert_leaf(&restored, leaf_identity);
    drop(owner);
    context.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "explicit restoration roots the cause anchor without resurrecting the module"
    );
    assert_eq!(module_drops.get(), 1);
    assert_leaf(&restored, leaf_identity);
    drop(restored);
    context.runtime().collect().unwrap();
    assert_eq!(drops.get(), 1);
}

#[derive(crate::JsData, surgeist_js_gc::heap::Trace)]
struct CycleLeaf {
    error: RefCell<Option<StoredJsError>>,
    #[heap_trace(skip)]
    drops: Rc<Cell<usize>>,
}

impl Drop for CycleLeaf {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn a_real_object_to_stored_interruption_cause_cycle_collects_without_external_observations() {
    let mut context = context();
    context
        .runtime()
        .heap
        .set_allocation_collection(AllocationCollection::BeforeAllocation)
        .unwrap();
    let drops = Rc::new(Cell::new(0));
    let module_drops = Rc::new(Cell::new(0));
    let object = JsObject::new_unique(
        context.runtime(),
        None,
        CycleLeaf {
            error: RefCell::new(None),
            drops: drops.clone(),
        },
    )
    .unwrap();
    let (module, terminal) = interrupt(
        original(object.clone().upcast()),
        &mut context,
        &module_drops,
    );
    context
        .runtime()
        .heap
        .with_pending_edges(terminal.clone().into_stored(), |stored| {
            *object.borrow().unwrap().data().error.borrow_mut() = Some(stored);
        })
        .unwrap();
    drop(object);
    drop(module);
    context.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "the native terminal result is the positive liveness control"
    );
    drop(terminal);
    context.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        1,
        "the object/cause cycle contains only traced edges"
    );
    assert_eq!(module_drops.get(), 1);
}

fn assert_masked(error: &JsError) {
    let DiagnosticKind::Native(aggregate) = panic_source(error).kind() else {
        panic!("visible aggregate topology")
    };
    assert_eq!(aggregate.kind(), DiagnosticNativeKind::Aggregate);
    let DiagnosticKind::Opaque(primitive) = aggregate.cause().unwrap().kind() else {
        unreachable!()
    };
    assert_eq!(
        primitive.value().unwrap(),
        js_string!("visible primitive").into()
    );
    let mut members = aggregate.members();
    assert_eq!(members.len(), 2);
    let DiagnosticKind::Native(replacement) = members.next().unwrap().kind() else {
        unreachable!()
    };
    assert_eq!(replacement.kind(), DiagnosticNativeKind::Type);
    assert_eq!(
        replacement.message(),
        "object belongs to a different runtime"
    );
    assert_eq!(replacement.realm_identity(), None);
    assert!(replacement.cause().is_none());
    let DiagnosticKind::Native(visible) = members.next().unwrap().kind() else {
        unreachable!()
    };
    assert_eq!(visible.kind(), DiagnosticNativeKind::Range);
    assert_eq!(visible.message(), "visible native member");
    assert!(members.next().is_none());
}

#[test]
fn masked_stored_interruption_restores_and_exports_after_its_foreign_runtime_is_disposed() {
    let mut foreign = context();
    let mut receiving = context();
    receiving
        .runtime()
        .heap
        .set_allocation_collection(AllocationCollection::BeforeAllocation)
        .unwrap();
    let drops = Rc::new(Cell::new(0));
    let module_drops = Rc::new(Cell::new(0));
    let input: JsError = PanicError::new("custody source")
        .with_source(
            JsNativeError::aggregate(vec![
                JsError::from_opaque(probe(foreign.runtime(), &drops).into()).unwrap(),
                JsNativeError::range()
                    .with_message("visible native member")
                    .into(),
            ])
            .with_cause(JsError::from_opaque(js_string!("visible primitive").into()).unwrap()),
        )
        .into();
    let (module, terminal) = interrupt(input, &mut foreign, &module_drops);
    let interruption_identity = identity(&terminal);
    drop(module);
    drop(foreign);
    assert_eq!(
        drops.get(),
        0,
        "the original observation still owns the foreign runtime"
    );
    assert_eq!(module_drops.get(), 0);
    let masked = terminal.admit_runtime(receiving.runtime());
    assert_eq!(
        drops.get(),
        1,
        "the final foreign root is actually disposed"
    );
    assert_eq!(module_drops.get(), 1);
    assert_masked(&masked);
    let owner = store(receiving.runtime(), &masked);
    drop(masked);
    receiving.runtime().collect().unwrap();
    let restored = owner.error.to_error().unwrap();
    assert_eq!(identity(&restored), interruption_identity);
    assert_masked(&restored);
    let exported = restored.clone().into_erased(receiving.access());
    let Some(ErasedEngineError::ModuleInterrupted {
        identity: erased_identity,
        cause,
    }) = exported.as_engine()
    else {
        unreachable!()
    };
    assert_eq!(*erased_identity, interruption_identity);
    let expected: JsError = PanicError::new("custody source")
        .with_source(
            JsNativeError::aggregate(vec![
                JsNativeError::typ()
                    .with_message("object belongs to a different runtime")
                    .into(),
                JsNativeError::range()
                    .with_message("visible native member")
                    .into(),
            ])
            .with_cause(JsError::from_opaque(js_string!("visible primitive").into()).unwrap()),
        )
        .into();
    assert_eq!(cause.as_ref(), &expected.into_erased(receiving.access()));
    let third = context();
    let readmitted = restored.admit_runtime(third.runtime());
    assert_eq!(identity(&readmitted), interruption_identity);
    assert_masked(&readmitted);
    drop(owner);
    receiving.runtime().collect().unwrap();
    assert_masked(&readmitted);
    assert_eq!(drops.get(), 1);
    assert_eq!(module_drops.get(), 1);
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
    succeeded: Rc<Cell<bool>>,
}

impl Drop for CollectReceiverOnDrop {
    fn drop(&mut self) {
        self.succeeded.set(self.receiver.collect().is_ok());
        self.observed.set(Some(self.local_drops.get()));
    }
}

#[test]
fn final_foreign_interruption_disposal_reenters_only_after_visible_local_roots_are_published() {
    let mut foreign = context();
    let receiving = context();
    let foreign_drops = Rc::new(Cell::new(0));
    let module_drops = Rc::new(Cell::new(0));
    let local_drops = Rc::new(Cell::new(0));
    let observed = Rc::new(Cell::new(None));
    let succeeded = Rc::new(Cell::new(false));
    let sentinel = JsObject::new_unique(
        foreign.runtime(),
        None,
        CollectReceiverOnDrop {
            receiver: receiving.runtime().clone(),
            local_drops: local_drops.clone(),
            observed: observed.clone(),
            succeeded: succeeded.clone(),
        },
    )
    .unwrap()
    .upcast();
    foreign
        .global_object()
        .unwrap()
        .set(
            js_string!("interruptionDisposalSentinel"),
            sentinel,
            true,
            foreign.access(),
        )
        .unwrap();
    let leaf = probe(foreign.runtime(), &foreign_drops);
    let (module, terminal) = interrupt(original(leaf), &mut foreign, &module_drops);
    let local = probe(receiving.runtime(), &local_drops);
    let local_identity = local.identity();
    let outer: JsError = PanicError::new("outer container")
        .with_source(JsNativeError::aggregate(vec![
            terminal,
            JsError::from_opaque(local.into()).unwrap(),
        ]))
        .into();
    drop(module);
    drop(foreign);
    assert_eq!(observed.get(), None);
    let admitted = outer.admit_runtime(receiving.runtime());
    assert!(
        succeeded.get(),
        "foreign disposal reenters without an engine heap borrow"
    );
    assert_eq!(
        observed.get(),
        Some(0),
        "the visible local root survives reentrant collection"
    );
    assert_eq!(foreign_drops.get(), 1);
    assert_eq!(module_drops.get(), 1);
    receiving.runtime().collect().unwrap();
    let Some(EngineError::Panic(panic)) = admitted.as_engine() else {
        unreachable!()
    };
    let DiagnosticKind::Native(aggregate) = panic.diagnostic().unwrap().kind() else {
        unreachable!()
    };
    let mut members = aggregate.members();
    let DiagnosticKind::Engine(DiagnosticEngine::ModuleInterrupted(nested)) =
        members.next().unwrap().kind()
    else {
        unreachable!()
    };
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        diagnostic: Some(replacement),
        ..
    }) = nested.cause().kind()
    else {
        unreachable!()
    };
    let DiagnosticKind::Native(replacement) = replacement.kind() else {
        unreachable!()
    };
    assert_eq!(
        replacement.message(),
        "object belongs to a different runtime"
    );
    let DiagnosticKind::Opaque(visible) = members.next().unwrap().kind() else {
        unreachable!()
    };
    assert_eq!(
        visible.value().unwrap().as_object().unwrap().identity(),
        local_identity
    );
    assert_eq!(local_drops.get(), 0);
    drop(admitted);
    receiving.runtime().collect().unwrap();
    assert_eq!(local_drops.get(), 1);
}

fn clone_during_heap_read(runtime: &Runtime, error: &JsError, leaf_identity: ObjectId) -> JsError {
    let expected_identity = identity(error);
    runtime
        .heap
        .read(|_| {
            let cloned = error.clone();
            assert_eq!(identity(&cloned), expected_identity);
            let DiagnosticKind::Opaque(leaf) = panic_source(&cloned).kind() else {
                panic!("the borrowed cause stays opaque while the heap is read")
            };
            assert_eq!(leaf.object_identity(), Some(leaf_identity));
            assert_eq!(leaf.value_type(), crate::value::Type::Object);
            assert!(matches!(
                leaf.value(),
                Err(surgeist_js_gc::heap::AccessError::HeapBorrowed)
            ));
            cloned
        })
        .unwrap()
}

#[test]
fn module_and_restored_anchor_observations_clone_and_borrow_under_a_held_heap_read() {
    let mut context = context();
    context
        .runtime()
        .heap
        .set_allocation_collection(AllocationCollection::BeforeAllocation)
        .unwrap();
    for restore in [false, true] {
        let drops = Rc::new(Cell::new(0));
        let module_drops = Rc::new(Cell::new(0));
        let leaf = probe(context.runtime(), &drops);
        let leaf_identity = leaf.identity();
        let (module, terminal) = interrupt(original(leaf), &mut context, &module_drops);
        let observation = if restore {
            let owner = store(context.runtime(), &terminal);
            drop(module);
            drop(terminal);
            context.runtime().collect().unwrap();
            assert_eq!(module_drops.get(), 1);
            let restored = owner.error.to_error().unwrap();
            drop(owner);
            restored
        } else {
            drop(module);
            terminal
        };
        let cloned = clone_during_heap_read(context.runtime(), &observation, leaf_identity);
        drop(observation);
        context.runtime().collect().unwrap();
        assert_eq!(
            drops.get(),
            0,
            "the returned clone is a real external observation owner"
        );
        assert_eq!(module_drops.get(), usize::from(restore));
        assert_leaf(&cloned, leaf_identity);
        drop(cloned);
        context.runtime().collect().unwrap();
        assert_eq!(drops.get(), 1);
        assert_eq!(module_drops.get(), 1);
    }
}

#[test]
fn preexisting_native_future_polls_terminal_cause_under_a_held_heap_read_without_late_rooting() {
    use std::{
        future::Future,
        pin::Pin,
        task::{Context as TaskContext, Poll, Waker},
    };

    let mut context = context();
    context
        .runtime()
        .heap
        .set_allocation_collection(AllocationCollection::BeforeAllocation)
        .unwrap();
    let runtime = context.runtime().clone();
    let drops = Rc::new(Cell::new(0));
    let module_drops = Rc::new(Cell::new(0));
    let leaf = probe(&runtime, &drops);
    let leaf_identity = leaf.identity();
    let original = original(leaf);
    let marker = probe(&runtime, &module_drops);
    MODULE_MARKER.with(|slot| assert!(slot.replace(Some(marker.into())).is_none()));
    let (gate, resolvers) =
        crate::object::builtins::JsPromise::new_pending(context.access()).unwrap();
    context
        .global_object()
        .unwrap()
        .set(js_string!("custodyGate"), gate, true, context.access())
        .unwrap();
    context
        .global_object()
        .unwrap()
        .set(js_string!("custodyEffects"), 0, true, context.access())
        .unwrap();
    let module = Module::parse(
        Source::from_bytes(
            "export const moduleOnlyMarker = takeModuleMarker(); await custodyGate;
             globalThis.custodyEffects += 1; stopForCustody(); globalThis.custodyEffects += 100;",
        ),
        None,
        context.access(),
    )
    .unwrap();
    assert_eq!(
        module.load(context.access()).unwrap().state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    module.link(context.access()).unwrap();
    let promise = context
        .with_execution(ExecutionOptions::new(100_000), |access| {
            module.evaluate(access)
        })
        .unwrap()
        .result
        .unwrap();
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    let mut future = promise.clone().into_js_future(context.access()).unwrap();
    let mut task = TaskContext::from_waker(Waker::noop());
    assert!(Pin::new(&mut future).poll(&mut task).is_pending());
    MODULE_MARKER.with(|slot| assert!(slot.borrow().is_none()));
    CALLBACK_ERROR.with(|slot| assert!(slot.replace(Some(original.clone())).is_none()));
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[1.into()], context.access())
        .unwrap();
    let stopped = context
        .with_execution(ExecutionOptions::new(100_000), |mut access| {
            access.run_jobs()
        })
        .unwrap();
    assert_eq!(stopped.result.unwrap_err(), original);
    drop(stopped.report);
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    assert_eq!(
        context
            .global_object()
            .unwrap()
            .get(js_string!("custodyEffects"), context.access())
            .unwrap(),
        1.into()
    );
    let terminal = context
        .with_execution(ExecutionOptions::new(100_000), |access| {
            module.evaluate(access)
        })
        .unwrap()
        .result
        .unwrap_err();
    let interruption_identity = identity(&terminal);
    drop(terminal);
    drop(original);
    drop(module);
    drop(promise);
    drop(resolvers);
    CALLBACK_ERROR.with(|slot| assert!(slot.borrow().is_none()));
    runtime.collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "the original subscribed future retains its cause after other native aliases are gone"
    );

    let returned = runtime.heap.read(|_| {
        let Poll::Ready(Err(error)) = Pin::new(&mut future).poll(&mut task) else {
            panic!("the preexisting future returns its terminal module error under the held heap read")
        };
        assert_eq!(identity(&error), interruption_identity);
        let DiagnosticKind::Opaque(leaf) = panic_source(&error).kind() else { unreachable!() };
        assert_eq!(leaf.object_identity(), Some(leaf_identity));
        assert!(matches!(leaf.value(), Err(surgeist_js_gc::heap::AccessError::HeapBorrowed)));
        error
    }).unwrap();
    drop(future);
    runtime.collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "the returned native error independently owns its association/cause"
    );
    assert_leaf(&returned, leaf_identity);
    drop(returned);
    runtime.collect().unwrap();
    assert_eq!(drops.get(), 1);
    assert_eq!(module_drops.get(), 1);
}

#[test]
fn a_local_module_adopting_foreign_identity_does_not_retain_an_unrelated_foreign_leaf() {
    let mut origin = context();
    let mut receiving = context();
    let origin_module_drops = Rc::new(Cell::new(0));
    let receiving_module_drops = Rc::new(Cell::new(0));
    let original = EngineError::Resource(super::EngineResourceError::Allocation);
    let (origin_module, origin_error) =
        interrupt(original.clone().into(), &mut origin, &origin_module_drops);
    let interruption_identity = identity(&origin_error);
    let masked = origin_error.admit_runtime(receiving.runtime());
    let (receiving_module, adopted) = interrupt(masked, &mut receiving, &receiving_module_drops);
    assert_eq!(identity(&adopted), interruption_identity);
    let DiagnosticKind::Engine(DiagnosticEngine::Resource(resource)) = cause(&adopted).kind()
    else {
        unreachable!()
    };
    assert_eq!(resource, super::EngineResourceError::Allocation);
    drop(origin_module);
    drop(receiving_module);
    let drops = Rc::new(Cell::new(0));
    let foreign = probe(origin.runtime(), &drops);
    // No receiving object/realm leaf competes with the adopted interruption's
    // local observation root. The foreign leaf has no independent alias.
    let outer: JsError = PanicError::new("composed custody")
        .with_source(JsNativeError::aggregate(vec![
            adopted,
            JsError::from_opaque(foreign.into()).unwrap(),
        ]))
        .into();
    origin.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "the unrestricted outer error owns its authored leaf"
    );
    let admitted = outer.admit_runtime(receiving.runtime());
    origin.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        1,
        "receiving admission must release the unrelated foreign leaf while the admitted error remains live"
    );
    let Some(EngineError::Panic(panic)) = admitted.as_engine() else {
        unreachable!()
    };
    let DiagnosticKind::Native(aggregate) = panic.diagnostic().unwrap().kind() else {
        unreachable!()
    };
    let mut members = aggregate.members();
    let DiagnosticKind::Engine(DiagnosticEngine::ModuleInterrupted(retained)) =
        members.next().unwrap().kind()
    else {
        unreachable!()
    };
    assert_eq!(retained.identity(), interruption_identity);
    assert!(matches!(
        retained.cause().kind(),
        DiagnosticKind::Engine(DiagnosticEngine::Resource(
            super::EngineResourceError::Allocation
        ))
    ));
    let DiagnosticKind::Native(replacement) = members.next().unwrap().kind() else {
        panic!("the foreign object is replaced")
    };
    assert_eq!(replacement.kind(), DiagnosticNativeKind::Type);
    assert_eq!(
        replacement.message(),
        "object belongs to a different runtime"
    );
    drop(admitted);
    origin.runtime().collect().unwrap();
    assert_eq!(drops.get(), 1);
}

#[test]
fn equal_cause_identities_with_different_observation_roots_keep_every_visible_occurrence_alive() {
    let mut origin = context();
    let mut receiving = context();
    let drops = Rc::new(Cell::new(0));
    let origin_module_drops = Rc::new(Cell::new(0));
    let receiving_module_drops = Rc::new(Cell::new(0));
    let leaf = probe(origin.runtime(), &drops);
    let leaf_identity = leaf.identity();
    let (origin_module, original_error) =
        interrupt(original(leaf), &mut origin, &origin_module_drops);
    let interruption_identity = identity(&original_error);
    let narrowed = original_error.clone().admit_runtime(receiving.runtime());
    let (receiving_module, adopted) = interrupt(narrowed, &mut receiving, &receiving_module_drops);
    assert_eq!(identity(&adopted), interruption_identity);
    drop(origin_module);
    drop(receiving_module);
    origin.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "the independent original observation is the initial liveness control"
    );
    assert_leaf(&original_error, leaf_identity);

    // Authored order is deterministic. The first occurrence has a local B
    // observation root and narrowed data; the second has the same cause identity
    // but still visibly owns the A leaf. No independent original alias survives.
    let outer: JsError = PanicError::new("same identity, distinct custody")
        .with_source(JsNativeError::aggregate(vec![adopted, original_error]))
        .into();
    origin.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "a narrowed occurrence cannot replace the original occurrence's legitimate external custody"
    );
    assert_eq!(origin_module_drops.get(), 0);
    let Some(EngineError::Panic(panic)) = outer.as_engine() else {
        unreachable!()
    };
    let DiagnosticKind::Native(aggregate) = panic.diagnostic().unwrap().kind() else {
        unreachable!()
    };
    let mut members = aggregate.members();
    assert_eq!(members.len(), 2);
    let DiagnosticKind::Engine(DiagnosticEngine::ModuleInterrupted(masked)) =
        members.next().unwrap().kind()
    else {
        unreachable!()
    };
    assert_eq!(masked.identity(), interruption_identity);
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        diagnostic: Some(masked_leaf),
        ..
    }) = masked.cause().kind()
    else {
        unreachable!()
    };
    let DiagnosticKind::Native(replacement) = masked_leaf.kind() else {
        unreachable!()
    };
    assert_eq!(replacement.kind(), DiagnosticNativeKind::Type);
    assert_eq!(
        replacement.message(),
        "object belongs to a different runtime"
    );
    let DiagnosticKind::Engine(DiagnosticEngine::ModuleInterrupted(visible)) =
        members.next().unwrap().kind()
    else {
        unreachable!()
    };
    assert_eq!(visible.identity(), interruption_identity);
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        diagnostic: Some(visible_leaf),
        ..
    }) = visible.cause().kind()
    else {
        unreachable!()
    };
    let DiagnosticKind::Opaque(leaf) = visible_leaf.kind() else {
        unreachable!()
    };
    assert_eq!(leaf.object_identity(), Some(leaf_identity));
    assert_eq!(
        leaf.value().unwrap().as_object().unwrap().identity(),
        leaf_identity
    );
    assert!(members.next().is_none());
    drop(outer);
    origin.runtime().collect().unwrap();
    receiving.runtime().collect().unwrap();
    assert_eq!(drops.get(), 1);
    assert_eq!(origin_module_drops.get(), 1);
    assert_eq!(receiving_module_drops.get(), 1);
}

#[test]
fn checked_interruption_anchor_restoration_preserves_identity_and_cause_on_retry() {
    use super::{RetainedInterruption, RootingError};
    use surgeist_js_gc::heap::AccessError;

    let context = Context::builder().build().unwrap();
    let (retained, anchor) = RetainedInterruption::prepare(&context).unwrap();
    let source: JsError = EngineError::InvalidEntryPermit.into();
    let retained = retained.seal(&source);
    let before = retained.external().unwrap();
    let identity = before.identity();
    context
        .runtime()
        .heap
        .read(|_| {
            assert_eq!(
                retained.external(),
                Err(RootingError::Access(AccessError::HeapBorrowed))
            );
        })
        .unwrap();
    let restored = retained.external().unwrap();
    assert_eq!(restored.identity(), identity);
    assert_eq!(
        restored.cause().kind(),
        DiagnosticKind::Engine(DiagnosticEngine::InvalidEntryPermit)
    );
    drop(before);
    drop(anchor);
    context.runtime().collect().unwrap();
    assert_eq!(restored.identity(), identity);
    drop(restored);
    drop(context);
    assert_eq!(retained.external(), Err(RootingError::Expired));
}
