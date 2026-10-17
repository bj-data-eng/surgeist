# surgeist-runtime

Rust orchestration contracts for Surgeist application and adapter authors. The
crate coordinates application state, surface lifecycle, invalidation, and queued
inputs, and reports abstract effects for root-owned adapters to execute.

The package is version `0.1.0`, and its runtime API is being designed. Concrete
background task execution, OS window event loops and rendering belong outside
this crate. Runtime owns the bounded native DOM task loop described below.
Runtime owns its consumer adapters to lower-level contracts. Its canonical DOM
host adapter associates generation-qualified surface elements with canonical DOM
elements and publishes exact Runtime focus/hover selections. This does not claim
complete HTML focus processing, native activity or pointer-device state. Host
publication revisions are independent of render invalidations and acknowledgements.

`DomSession` keeps the original canonical DOM, CSSOM resources, element tasks,
native hosted lifecycle and Source event continuations together. Supply explicit
`DomSessionLimits`, edit through `dom()`, and obtain weak execution permissions
with `source_execution()`, `style_execution()`, `element_task_execution()` and
`hosted_execution()`. The Source driver executes one edit or named feature
operation; the Style handle retains CSSOM resources, notifications and readback.
`try_shutdown` returns the whole session while mandatory work remains. Dropping
it abandons execution without delivering callbacks or claiming completion.

The native dispatcher handles actual Details and Dialog toggles, Dialog method
beforetoggle events and queued Close notifications, including Global capture for
hosted Documents. One element-task owner retains both DOM-manipulation and
user-interaction work, with explicit source selection and shared finite limits.
Toggle tasks coalesce through their family trackers; Close notifications are
independent user-interaction tasks without a toggle tracker or coalescing.
Completed Dialog toggles report `TrackerCleared` or
`TargetRetiredDuringDispatch` through `dialog_toggle_disposition` beside the
actual dispatch receipt. Clearing a callback's replacement tracker leaves its
queued task intact. Cleanup retries retain the original result without replaying
listeners. Other task families have no Dialog toggle disposition.

Nonmodal Show queues its actual toggle, publishes `open=""` through ordinary
hooks, and consumes qualified focus and popover observations. The sealed headless
Host can supply actual remembered-focus absence. Qualified raw members' actual
hidden popover state advances the empty popover steps; proven negative allow-focus
can complete Show or ShowModal without focus
movement. Unknown or unsupported actual inputs retain their precise action wait.
Permitted calls run the HTML focusing child. Actual autofocus or completed empty
delegate traversals can select the subject. An own-inert subject can
finish called focusing without a change, then return on an actual origin
difference or complete the same-origin top Document's autofocus processing.
The same affine focus bridge serves real autofocus insertion occurrences through
DOM's canonical candidate owner. Unsupported delegate queries retain their exact
root, queried node, step and operation occurrence through `FocusDelegate` requests;
other unsupported work retains its specific action. No focus movement or rendering-time autofocus flush is
inferred.
The standard nonmodal Close method dispatches genuine noncancelable beforetoggle,
rebases its open read after callbacks, runs full open-removal/watcher hooks, applies
a supplied return value and queues its independent Close notification when previous
focus is absent. Modal Close requests deferred top-layer removal and clears its
modal flag. Positive focus restoration remains unavailable.

Source edits establish and destroy watchers through the original Window manager.
Execution retirement preserves that Window while a watcher or its unsettled
canonical publication needs it; later open removal can finish cleanup on the
surviving inactive Dialog. An unfinished synchronous method tail prevents queued
delivery. Positive focus, broader modal services, platform close requests and
nonempty popover processing remain separate contributions. See
[DOM orchestration](docs/explanation.md#dom-orchestration) for creation, teardown,
custody and capability limits.

## Start

From this checkout, prove that a reducer's task request becomes a runtime intent:

```sh
cargo test --offline -p surgeist-runtime --lib tests::runtime_forwards_task_work_as_intents_without_executing_it -- --exact
```

Expected result: one test passes with no failures. It checks one forwarded task
intent and no diagnostics. See [getting started](docs/getting-started.md) for
prerequisites and the example's source.

## Documentation

- [Getting started](docs/getting-started.md): run and understand the first proof.
- [How-to](docs/how-to.md): emit intents, handle queues, render surfaces, and build snapshots.
- [Reference](docs/reference.md): package facts, public entry points, defaults, and checks.
- [Explanation](docs/explanation.md): ownership, state transitions, generations, and wake delivery.

## Stewardship

[MIT license](LICENSE) · [Attribution](NOTICE.md)
