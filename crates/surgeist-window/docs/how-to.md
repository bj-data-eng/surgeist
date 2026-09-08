# How-to

These procedures assume you have completed [getting started](getting-started.md).
The linked source owns the exact API contract.

## Configure startup before running

Use this procedure when a handler needs windows at startup.

1. Build authored `Open` requests and add the complete batch with `App::open`.
2. Call `App::run` for native execution, or retain a display-free `Loop` with
   `App::into_loop` and later call `Loop::run`. Conversion retains authored
   commands without validating them; either run entry point intrinsically
   validates the batch before creating the native event loop.
3. In a deterministic test, create `Runner::new(handler)`, call
   `Runner::startup(Vec<Command>)`, check its result, and call `Runner::resume`.
   `startup` intrinsically validates and stores the batch before resume.
4. Observe `Created` then `Ready` for a successfully opened, still-live window.
   Inspect `Runner::result` after deterministic dispatch: `None` means it remains
   active, `Some(Ok(()))` means successful exit, and `Some(Err(error))` exposes the
   retained failure. Native execution returns its terminal outcome from `run`.

Do not dispatch runtime commands before the deterministic runner's first resume:
that records a terminal `InvalidRequest`. Startup replacement is available only
before resume. See [App](../src/dsl.rs), [Loop](../src/loop_.rs),
[Runner](../src/testing.rs), and the [phase explanation](explanation.md#authored-work-and-observed-state).

## Choose a deterministic test harness

For tests of handlers, callback ordering, or terminal behavior, use
`testing::Runner`. Configure startup, deliver the relevant resume, draw, metrics,
input, or close operation, and assert handler observations plus `Runner::result`.
The [complete example](getting-started.md#follow-the-complete-lifecycle)
shows creation through close completion.

For a focused command, state, or effect assertion, use `testing::Host`. Apply the
command, check the returned `Result`, and inspect its registry and recordings.
`Host` normalizes, plans, applies, and records facts without invoking a handler.
Use `Host::with_capabilities` or `Runner::with_capabilities` when the test needs an
explicit immutable capability report. A report permits planning; it does not
prove behavior on a native platform. Both harnesses are defined in
[src/testing.rs](../src/testing.rs).

## Submit work through a proxy

Use a `Proxy` when work originates outside the owning callback. Obtain it from
`Context::proxy` when available, or from `Runner::proxy` in a deterministic test.

1. Submit a typed command with `Proxy::send` and check the immediate result.
   An error here can reject intrinsically malformed work before queue acceptance.
2. Let the owning native loop drain accepted work. In a deterministic test, call
   `Runner::flush_proxy` after first resume.
3. Verify the resulting state or callback and inspect the terminal result.
   Queue acceptance alone does not prove application. Stale or closing target
   work is ignored when drained, and closed ingress rejects later sends.

See [Proxy](../src/registry.rs), its [typed helpers](../src/dsl.rs), and
[identity and ingress semantics](explanation.md#identity-and-proxy-ingress).

## Use a loop-owned clipboard

Use `App::with_clipboard`, `Loop::with_clipboard`, or `Runner::with_clipboard` to
provide a `Box<dyn Clipboard>` before execution. `MemoryClipboard` provides
deterministic text and image storage and is the loop and runner default.

During a callback, borrow the service with `Context::clipboard`, perform the read
or write, and handle its `Result`. Verify data through the injected implementation
or `MemoryClipboard`; [clipboard tests](../src/tests.rs) cover shared callback
access, propagated failures, and retained successful writes after a later handler
error. Clipboard I/O happens immediately and is outside queued-command rollback.
See [the clipboard interface](../src/clipboard.rs) and
[callback transactions](explanation.md#callback-transactions-and-terminal-results).

## Submit an initial accessibility tree

This procedure requires the `accessibility` feature, a live window, and a host
capability report that permits accessibility. Import semantic tree types through
`surgeist_window::accesskit`; the native adapter remains private.

1. Wait for `Handler::event` to receive
   `EventKind::Accessibility(AccessibilityEvent::InitialTreeRequested(id))`.
2. From that callback, submit the first `TreeUpdate` through
   `Context::update_accessibility`, using that exact live `id`.
3. Include `tree: Some(Tree::new(root))`, the root node in `nodes`,
   `TreeId::ROOT` for the main tree, and a focus node belonging to the tree.
4. Return `Ok(())` to commit the queued submission. Once the initial update has
   been applied, later active-tree updates may be incremental. After deactivation,
   wait for another initial-tree request and submit a complete update again.

Stale targets, unsupported capability, invalid tree phase, or planning or host
application failures become the retained terminal error and suppress later
callbacks. See [Context::update_accessibility](../src/context.rs) and the complete
[typed rustdoc example](../src/lib.rs). That example injects the parity event
after deterministic resume and verifies the applied `Command::UpdateAccessibility`
through the runner's `Host`.

Run the example with its feature enabled:

```sh
cargo test --offline -p surgeist-window --features accessibility --doc
```

Expected result: all rustdoc examples pass, with the accessibility example's
feature-gated body exercising initial-tree submission and a nonterminal runner.
