# How-to

## Verify native preferences and monitor facts

In a macOS WindowServer session, explicitly run the native preference probe:

```sh
SURGEIST_RUN_NATIVE_PREFERENCES_PROBE=1 cargo test --offline --locked -j 1 -p surgeist-window --test native_preferences_native
```

The probe opens an invisible native window, captures its public environment in
`Ready`, exits the event loop, and compares the capture with actual AppKit
observations. It reads settings and screen geometry without changing them.
The work-area comparison requires a single checked SDK screen because the public
snapshot exposes no native monitor ID; otherwise the probe reports ambiguity
and skips that comparison instead of guessing. The diagnostic includes the SDK
color-space name, sample bits and component count. Exact-ID selection, conversion
boundaries, grayscale and callback refresh behavior have owning-module tests.
Normal headless
checks print an explicit skip for this probe; the ordinary library tests cover
preference mapping and callback publication without a display.

In a Windows desktop session, run the same opt-in probe from PowerShell:

```powershell
$env:SURGEIST_RUN_NATIVE_PREFERENCES_PROBE = '1'
cargo test --offline --locked -j 1 -p surgeist-window --test native_preferences_native
Remove-Item Env:SURGEIST_RUN_NATIVE_PREFERENCES_PROBE
```

The Windows probe opens an invisible window through the production loop and
compares its `Ready` preference snapshot with Windows Runtime animation,
transparency and contrast-theme observations. It independently locates that
process-owned native window, obtains its current monitor directly, and compares
the captured screen, work area, client origin, DPI scale and fixed viewport
variants with observations from the safe typed Windows bridge. A complete
Windows Runtime pointer-device sample independently checks the host's primary
and union input capabilities;
unavailable native input remains explicit. It works with multiple monitors without
selecting a device by screen geometry or enumeration order. Keep settings and
monitor configuration and input devices stable during the probe. It reads facts without changing
them. An independent fresh read-only display-state query also compares RGB wire
color depth when that witness is available; unavailable or unsupported witnesses
print an explicit skip for that comparison. On an 8-bit RGB output, the resulting
24 bits equal the public unknown-depth fallback, so that native comparison alone
does not distinguish observation from fallback. Private fixture checks cover
other channel depths and invalid states. The probe makes no claim about panel
precision, HDR behavior, notification wakeups or rendering.

See [host observations](explanation.md#host-observations-and-consumer-inputs) for
the supported values, sampling opportunities and unavailable observations.

## Verify native UI font preferences

In a Windows desktop session, explicitly enable the native font probe:

```powershell
$env:SURGEIST_RUN_NATIVE_FONT_PROBE = '1'
cargo test --offline --locked -j 1 -p surgeist-window --test system_fonts_native
Remove-Item Env:SURGEIST_RUN_NATIVE_FONT_PROBE
```

The probe compares every public Windows logical-font field for all six roles
with a fresh safe native query at 96 DPI. Keep UI font settings stable while it
runs. It reads preferences without selecting a physical font or changing host
settings. Ordinary Windows test runs print an explicit skip for this probe.

On macOS, run the same test target on the process main thread without an opt-in
variable. It checks AppKit names, native weight and metric validity, and compares
the selected role sizes with independent AppKit defaults. Other platforms check
the explicit unsupported-platform result. See [native UI fonts](explanation.md#native-ui-fonts)
for the platform-specific observation contracts.

## Verify the platform clock

Use the Window clock when creating a deadline:

```rust
use std::time::Duration;
use surgeist_window::Instant;

let deadline = Instant::now() + Duration::from_millis(20);
```

Pass the resulting instant to a callback scope's `at` method or to `Proxy::at`.
Native callers may also pass `std::time::Instant` directly because it is the
same type there. Browser callers use `surgeist_window::Instant` from the owning
host context; no conversion from a native clock is provided. The
[clock contract](explanation.md#platform-monotonic-clock) explains the separate
host and DOM domains.

With the `wasm32-unknown-unknown` target already installed, check the complete
Window library and test targets:

```sh
cargo check --offline --locked -j 1 -p surgeist-window --target wasm32-unknown-unknown --lib --tests
cargo check --offline --locked -j 1 -p surgeist-window --target wasm32-unknown-unknown --lib --tests --features accessibility
```

These commands compile the browser clock contract without executing it. Real
browser qualification additionally needs compatible Wasm bindings and a browser
host to observe clock sampling and deadline wakeups. Use binding tools matching
the versions resolved in [Cargo.lock](../../../Cargo.lock). These compilation
checks provide no browser runtime harness or runtime evidence.

## Reproduce original-unit word navigation

Run the real AccessKit consumer-tree regression without a display:

```sh
cargo test --offline --locked -j 1 -p surgeist-window --features accessibility --test accesskit_word_navigation
```

Check the maintained consumer's own unit tests with:

```sh
cargo test --offline --locked -j 1 -p accesskit_consumer --lib
```

The fixtures preserve original text, selectable units and exact logical positions
through true end. They cover ordinary runs and 262-unit ASCII/UTF8 runs, including
word starts at 5 and the maximum encoded start 255. Every position checks word
classification and forward/backward navigation against independent boundaries.
Physical Cell caret policy does not participate.

The workspace selects the maintained consumer 0.36.0 through its root Cargo
patch. Three word searches widen stored starts to `usize`, preserving original
indices above 255. See [consumer provenance](../../../vendor/accesskit-consumer/UPSTREAM.md)
and [issue #1613](https://github.com/bj-data-eng/surgeist/issues/1613) for the
source correction and native integration qualification. Core 0.24.1 remains
shared with Text.

When consuming Window from another application workspace, select the maintained
consumer in that application's root manifest:

```toml
[patch.crates-io]
accesskit_consumer = { path = "/path/to/surgeist/vendor/accesskit-consumer" }
```

[Cargo reads patch settings at the workspace root](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#the-patch-section).
Verify that the resolved native adapters use this copy; a dependency manifest
does not propagate the override into a downstream workspace. The consumer-tree
regression is display-free. Native UIA and AT-SPI client interaction remains a
separate platform qualification.

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

To deliver captured magnification input, pass a `MagnificationEvent` through
`NativeEventTransition::magnification` to `Runner::transition`. Observe it in
`Handler::input`, where `Input::position` exposes optional pointer context and
`Input::modifiers` retains the captured modifiers. Assert the gesture separately
from any consumer-selected response: delivery itself applies no zoom or window
command. See the [raw intent contract](explanation.md#native-magnification-input)
and [conversion and routing tests](../src/magnification_tests.rs).

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
