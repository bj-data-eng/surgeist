# Getting started

Run a task that emits one progress event and one output event through the public
`surgeist_task` contracts.

## Prerequisites

Use a checkout of the Surgeist repository with Cargo and an already installed Rust
toolchain capable of compiling its Rust 2024 source. The
[manifest](../Cargo.toml) does not declare a minimum supported Rust version. The
offline command below needs the manifest-selected dependencies already in
Cargo's cache. The shared product resolution is committed in
[../../../Cargo.lock](../../../Cargo.lock).

## Run the example

1. Open a terminal in the crate directory, beside [Cargo.toml](../Cargo.toml).
2. Run:

   ```sh
   cargo run --offline -p surgeist-task --example basic_task
   ```

3. Confirm the process exits successfully and prints:

   ```text
   emitted 2 task events
   ```

The example also asserts that it received exactly two events. Cargo may print
build messages before the program's output. If Cargo reports a missing cached
dependency, the checkout is not prepared for this offline proof.

## What happened

The tracked [example](../examples/basic_task.rs) builds a `TaskSpawnRequest`
with a task id, attempt id, key, workspace scope, cancellation token, event sink,
input, and blocking function. It creates a `TaskContext` from that request and
invokes the job directly. The job reports 10 of 100 rows and outputs
`loaded orders.csv`; it does not read a file.

This demonstrates request construction, task context, and event emission. It
does not start `TokioTaskExecutor` or exercise terminal lifecycle emission.
Continue with [How-to](how-to.md) to route work through the executor and handle
its completion reports.
