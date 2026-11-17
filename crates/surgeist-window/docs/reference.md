# Reference

## Package and features

[Cargo.toml](../Cargo.toml) owns package identity, dependency versions, features,
and targets.

| Item | Current value |
| --- | --- |
| Package | `surgeist-window` version `0.1.0` |
| Rust import | `surgeist_window` |
| Edition | Rust 2024 |
| Library entry point | [src/lib.rs](../src/lib.rs) |
| Default features | Empty |
| `accessibility` feature | Enables optional `accesskit` and `accesskit_winit` dependencies and the typed accessibility surface |
| Native backend dependency | `winit` |
| Public dependency reexports | `Instant` at the root, `cursor::CursorIcon`, `event::keyboard_types`, `registry::raw_window_handle`; `accessibility::accesskit` with `accessibility` enabled |

## Public interfaces

The crate root holds `App`, `app`, `Open`, `open`, `Handler`, `Error`, `ErrorCode`,
`Result` and `Instant`. Every other public item has exactly one path, in the module
named in the first column; a root path to a module item does not resolve. Backend
adapters and planning implementation types remain private except for the public
`command::HostCommandPlan` diagnostic surface.

| Surface | Purpose | Source |
| --- | --- | --- |
| Root `App`, `app`, `Open`, `open`; `descriptor::WindowRequest` | Authored startup and window creation intent | [DSL](../src/dsl.rs), [descriptors](../src/descriptor.rs) |
| `host::Loop`; root `Handler` | Native event-loop ownership and consumer callbacks | [loop](../src/loop_.rs), [handler](../src/handler.rs) |
| `callback::{Context, Target, ..}` scopes; `command::Command` | Queued window commands, draw/close/exit actions, and host services | [context](../src/context.rs), [DSL](../src/dsl.rs), [commands](../src/command.rs) |
| `descriptor::{WindowSnapshot, Metrics}`; `event::{EventKind, InputEvent}` | Observed state, metrics, and event payloads | [descriptors](../src/descriptor.rs), [events](../src/event.rs) |
| Root `Instant` | Shared platform monotonic clock for deadlines and input receipt times | [clock contract](explanation.md#platform-monotonic-clock) |
| `event::{MagnificationEvent, MagnificationGesture, MagnificationDelta}` | Raw native pinch or smart intent, with explicit unavailable pinch delta | [magnification input](explanation.md#native-magnification-input), [events](../src/event.rs) |
| `environment::HostEnvironment`, viewport, screen, input and preference values | Captured host facts with explicit unavailable observations | [environment](../src/environment.rs) |
| `native_geometry::{NativeGeometryRequest, NativeOpenFeatures}` | Host-authorized CSS-pixel geometry resolved into ordinary window commands | [native geometry](../src/native_geometry.rs) |
| `event::NativePointerCoordinates` | Fractional native client, desktop and screen-relative pointer observations | [pointer coordinates](../src/pointer_coordinates.rs) |
| `system_fonts::SystemFontEnvironment` | Native UI role preferences, with AppKit face metrics and Windows logical font requests | [system fonts](../src/system_fonts.rs) |
| `capability::HostCapabilities`; `command::HostCommandPlan` | Immutable host support report and planned diagnostics | [capabilities](../src/capability.rs), [planning](../src/planning.rs) |
| `registry::{Registry, Id, Ref, Handle, Access, Proxy}` | Live lookup, runtime identity, native access, and queued ingress | [registry](../src/registry.rs) |
| `clipboard::{Clipboard, MemoryClipboard}`, image types | Loop-owned text and image service | [clipboard](../src/clipboard.rs) |
| `testing::Runner`, `testing::Host` | Deterministic lifecycle execution and callback-free recording | [testing](../src/testing.rs) |
| Root `Error`, `ErrorCode`, `Result` | Structured failures and retained terminal outcomes | [errors](../src/error.rs) |

## Callback routes

[Handler](../src/handler.rs) owns callback contracts.

| Observation or opportunity | Callback |
| --- | --- |
| Initial resume | Global `resume` after startup validation and staging, before startup application |
| Later resume and suspend | Per-window generic events, then global `resume` or `suspend` |
| Successful creation | `event` with `Created`, then `ready` while still live |
| Resize or scale-factor change | `resize`, after committed metrics |
| Normalized input | `input` |
| Focus, move, occlusion, theme, environment, file drag, optional accessibility | `event` |
| Draw delivery | `draw` |
| Close request | `close`; default accepts |
| Completed closing | `closed`, once, with historical state |
| Opted-in idle opportunity | `idle` when `wants_idle` is true after first resume |

Successful `ready` and `resize` callbacks schedule a next-frame draw. Input and
metric callbacks are not also delivered through generic `event`. Close requests
and destruction are absent from the generic `EventKind` enum.

## Capabilities

`HostCapabilities` reports roles, fullscreen modes, cursor presentation and grab,
mutable window operations, IME semantics, and accessibility. Its immutable values
guide planning and do not describe observed native state.

| `CapabilitySupport` | Planning meaning |
| --- | --- |
| `Supported` | Permits the requested capability |
| `Unsupported` | Rejects the requested capability |
| `RuntimeDependent` | Permits the request for runtime host resolution |

The [capability model](../src/capability.rs) owns these meanings; the private
[native adapter](../src/winit_adapter.rs) resolves the current native report.

## Geometry and errors

`Point`, `Size`, `Rect`, and `Insets` use logical `f64` window units.
`PhysicalPoint` uses integer `i32` native pixel coordinates; `PhysicalSize` uses
`u32` pixel dimensions. `Metrics` retains a positive display scale and conversion
methods. Rounding and saturation in logical-to-physical point conversion make
that conversion non-reversible. See [geometry](../src/geometry.rs) and
[Metrics](../src/descriptor.rs).

`Error` retains a semantic `code`, diagnostic `message`, optional window `id`,
optional `command_kind`, a `completed_prefix` count, and an error source when
available. `CommandBatchFailed` records the number of commands committed before
a backend failure. `WindowClosing` and `StaleWindow` distinguish closing from
unknown or completed identities for synchronous work. See
[the error definitions](../src/error.rs) and [batch semantics](explanation.md#callback-transactions-and-terminal-results).

## Verification sources and commands

The [repository guide](../AGENTS.md) owns command discovery. Focused verification
lives in [src/tests.rs](../src/tests.rs) and inline test modules in
[pump](../src/pump.rs), [lease](../src/lease.rs), [testing](../src/testing.rs), and
[the native adapter](../src/winit_adapter.rs). Rustdoc examples live in
[src/lib.rs](../src/lib.rs) and [src/event.rs](../src/event.rs).

These invocations use existing local dependencies without fetching:

| Command | Evidence |
| --- | --- |
| `cargo check --offline -p surgeist-window` | Default-feature compilation |
| `cargo test --offline -p surgeist-window` | Default-feature unit tests and rustdoc examples |
| `cargo test --offline -p surgeist-window --features accessibility` | Unit tests and rustdoc examples with accessibility enabled |
| `cargo clippy --offline -p surgeist-window --all-targets --no-deps -- -F unsafe-code -D warnings` | Default-feature Clippy check with unsafe code forbidden and warnings denied |
| `cargo fmt -p surgeist-window --check` | Rust formatting |

The [getting-started guide](getting-started.md) supplies the focused first-success
command. A passing deterministic check demonstrates the exercised source
contract; it does not establish native behavior on every platform.
