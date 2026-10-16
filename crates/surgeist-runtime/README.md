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

`DomSession` keeps the original canonical DOM, CSSOM resources, Details tasks and
native hosted lifecycle together. Supply explicit `DomSessionLimits`, edit
through `dom()`, and obtain weak execution permissions with `source_execution()`,
`style_execution()`, `details_execution()` and `hosted_execution()`. The Source
driver executes one edit or named feature operation; the Style handle retains
CSSOM resources, notifications and readback. `try_shutdown` returns the whole
session while mandatory work remains. Dropping it abandons execution without
delivering callbacks or claiming completion. The native dispatcher handles real
Details toggle events, including Global capture for hosted Documents. See
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
