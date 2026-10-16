# surgeist-runtime

Rust orchestration contracts for Surgeist application and adapter authors. The
crate coordinates application state, surface lifecycle, invalidation, and queued
inputs, and reports abstract effects for root-owned adapters to execute.

The package is version `0.1.0`, and its runtime API is being designed. Concrete
task execution, native event loops and rendering belong outside this crate.
Runtime owns its consumer adapters to lower-level contracts. Its canonical DOM
host adapter associates generation-qualified surface elements with canonical DOM
elements and publishes exact Runtime focus/hover selections. This does not claim
complete HTML focus processing, native activity or pointer-device state. Host
publication revisions are independent of render invalidations and acknowledgements.

`DomStyleSession` keeps the original canonical DOM, its CSSOM resources and pending
Style work together. Obtain an execution permission with `execution()` and edit
through the controlled `dom()` view. Permissions and event observations do not
own the session. `try_shutdown` returns the whole session when mandatory work is
pending; dropping the session abandons it without delivering callbacks or claiming
completion. Runtime's native event dispatcher also drives the bounded Details
toggle operations. See [DOM orchestration](docs/explanation.md#dom-orchestration)
for custody, continuation and capability limits.

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
