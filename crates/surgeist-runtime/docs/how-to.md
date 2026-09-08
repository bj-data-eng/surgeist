# How-to

These procedures assume the [first proof](getting-started.md) passes. They use
the public `surgeist_runtime` reexports; the linked source owns the exact API and
error contracts.

## Emit work for root adapters

For a reducer that requests task, resource, or service work, return a successful
`ReducerResult` carrying a `ReducerCommit` with the desired effects. Enqueue a
typed input, drain one bounded turn, and inspect the reported dispositions and
intents. This example requests task, resource, and service work in one commit:

```rust
use surgeist_runtime::{
    AppEffect, AppInput, AppScope, EffectDisposition, InputProvenance, Reducer, ReducerCommit,
    ReducerResult, ResourceId, Runtime, RuntimeBudget, RuntimeIntent, ServiceId, TaskIntentKey,
    TaskIntentName, UiInput,
};

struct IntentReducer;

impl Reducer<(), ()> for IntentReducer {
    fn reduce(&mut self, _: &(), _: &AppInput<()>) -> ReducerResult<()> {
        ReducerResult::unchanged(
            ReducerCommit::new()
                .with_effect(AppEffect::start_task(
                    TaskIntentName::new("thumbnail"),
                    TaskIntentKey::new("photo:42"),
                    AppScope::app(),
                ))
                .with_effect(AppEffect::invalidate_resource(
                    ResourceId::new("photo:42"),
                    "source changed",
                ))
                .with_effect(AppEffect::start_service(ServiceId::new("indexer"))),
        )
    }
}

let mut runtime = Runtime::new((), IntentReducer);
runtime.enqueue_ui(UiInput::new((), InputProvenance::system())?)?;
let outcomes = runtime.drain_once(RuntimeBudget::default())?;

assert_eq!(outcomes.forwarded_effects(), 3);
assert!(outcomes
    .effect_outcomes()
    .iter()
    .all(|outcome| outcome.disposition() == EffectDisposition::Forwarded));
assert!(matches!(outcomes.intents()[0], RuntimeIntent::StartTask(_)));
assert!(matches!(outcomes.intents()[1], RuntimeIntent::InvalidateResource(_)));
assert!(matches!(outcomes.intents()[2], RuntimeIntent::StartService(_)));
# Ok::<(), Box<dyn std::error::Error>>(())
```

The assertions verify three forwarded intents in their emitted order. Root
`surgeist` lowers them through concrete adapters into sibling crates, then returns
their resulting inputs through the runtime task and service queues. Forwarding
does not mean the requested work has completed. See [effect.rs](../src/effect.rs)
and [runtime.rs](../src/runtime.rs) for the supported dispositions.

## Handle queue capacity and continued work

When connecting input producers, choose a `RuntimeQueuePolicy` at runtime
construction and a `RuntimeBudget` for each drain turn.

1. Wrap inputs with the appropriate `UiInput`, `TaskInput`, or `ServiceInput`
   constructor. Handle invalid provenance before enqueueing.
2. Enqueue into the matching runtime lane. On capacity rejection, recover the
   exact input with `RuntimeQueueError::into_rejected`; retain it for a later
   admission attempt when space is available.
3. Call `drain_once` and handle its effect outcomes and intents. Use
   `has_pending_inputs()` and the three remaining-input counts to decide whether
   another runtime turn is needed.
4. If a checked version advance fails, inspect
   `RuntimeDrainError::partial_report()` for earlier committed work. The triggering
   input remains at the front of its lane; another immediate retry cannot resolve
   an exhausted version counter.

The queue rejection example in [runtime.rs](../src/runtime.rs) verifies that a
zero-capacity UI queue returns the original payload. Its focused tests cover
exact retry after space becomes available and cyclic lane draining.

For a thread-safe producer using `AppProxy`, the root host adapter supplies a
`WakeBridge` that schedules a future host turn. In that turn, drain with a
`NonZeroUsize` limit and inspect `continuation_wake_error()` before consuming the
report with `into_drained()`. A continuation failure means remaining proxy inputs
need a host-arranged turn. Transfer each `ProxyInput` to its runtime lane while
retaining any input rejected by runtime capacity. See
[the proxy contract](../src/proxy.rs) and
[wake delivery](explanation.md#queues-and-host-wakeups).

## Render a registered surface

With a `SurfaceRoot` and runtime-owned window/surface IDs available:

1. Construct the surface with `UiSurface::try_new` and pass it to
   `Runtime::register_surface`. Retain the returned `SurfaceRef`; registration
   may assign a successor generation to a reused ID.
2. Apply lifecycle changes through `Runtime::update_surface`, and use the
   runtime's resize, scroll, focus, and hover methods for interaction state.
   A surface must be `Ready` or `Resized` to render.
3. Call `Runtime::begin_render` to borrow the application state and frame
   metadata. Let the root rendering adapter consume that state, then use
   `SurfaceRenderState::into_frame` to release the borrow and retain the frame.
4. Once that frame's rendering work is complete, pass the frame to
   `Runtime::mark_rendered`. Inspect the acknowledgement's consumed and remaining
   invalidation counts and `redraw_required()`.

The public example on [Runtime](../src/runtime.rs) verifies that viewport and
scroll changes produce two consumed invalidations and no remaining work after
acknowledgement. Stale frames return a typed error; later invalidations may still
require rendering after an otherwise successful acknowledgement.

## Create a snapshot from declared bindings

When an application needs a snapshot, begin with its authored `AppManifest`.

1. Add each root's `SnapshotBinding` declarations to its `RootDescriptor`.
2. Construct `App` with `App::try_new`. Handle the collected
   `ManifestValidationError::issues()` if validation fails.
3. Call `App::new_snapshot` with a declared root ID and the represented
   `StateVersion`.
4. Add each `SnapshotEntry` with a declared binding ID, its matching source type,
   and a validated `SnapshotValue`.

Verify the snapshot's root, version, declarations, and accepted entries. Unknown
roots, undeclared or duplicate entries, and source-type mismatches produce typed
errors; a rejected entry leaves the accepted entries unchanged. The
[App snapshot example](../src/descriptor.rs) shows construction, and
[snapshot.rs](../src/snapshot.rs) contains rejection examples. Values are opaque
serialized text; runtime does not supply a codec or validate an application
schema.
