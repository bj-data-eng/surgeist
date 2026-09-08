# Getting started

This guide runs a focused library test that sends an application input through a
reducer and observes a forwarded task intent. It exercises runtime orchestration
without a native window or task executor.

## Prerequisites

Use a local Surgeist checkout with Rust `1.97` or newer and Cargo
already available. The [manifest](../Cargo.toml) declares edition `2024`, a
minimum Rust version of `1.97`, no dependencies, and no enabled default features.
The proof below uses offline mode and does not acquire tooling or dependencies.

## Run the first proof

1. Open a terminal in the repository directory containing `Cargo.toml` and
   `src/lib.rs`.
2. Run the existing focused test:

   ```sh
   cargo test --offline -p surgeist-runtime --lib tests::runtime_forwards_task_work_as_intents_without_executing_it -- --exact
   ```

3. Confirm that Cargo reports one passed test and zero failures. The test name
   should be `tests::runtime_forwards_task_work_as_intents_without_executing_it`;
   zero selected tests is not a successful proof.

In [the test](../src/tests.rs), `CounterReducer` receives `CounterInput::StartTask`
through a `UiInput`. After one `Runtime::drain_once`, the assertions check one
forwarded effect, one intent, the effect kind `runtime.start_task`, and an empty
diagnostic log. The reducer fixture is defined in the same test module and is
compiled only for this crate's tests.

If the compiler is older than the manifest's minimum or required tooling is
unavailable, resolve that local prerequisite before retrying. If compilation or
the assertion fails, keep the command output as the failure evidence.

Continue with the [intent ownership example](how-to.md#emit-work-for-root-adapters)
to see task, resource, and service effects together. The [reference](reference.md)
lists the broader local checks.
