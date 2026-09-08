# surgeist-runtime

Rust orchestration contracts for Surgeist application and adapter authors. The
crate coordinates application state, surface lifecycle, invalidation, and queued
inputs, and reports abstract effects for root-owned adapters to execute.

The package is version `0.1.0`, and its runtime API is being designed. Concrete
task execution, native event loops, rendering, and cross-crate adapters belong
outside this leaf.

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
