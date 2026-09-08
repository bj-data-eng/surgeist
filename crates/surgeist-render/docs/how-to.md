# How-to

These procedures assume the [headless first-success test](getting-started.md)
has passed and the required dependencies are already available locally. They
verify this leaf crate; root facade, adapters, and browser-host integration have
their own owning repository.

## Present both rendering routes in a native window

Use the tracked [render_window_smoke example](../examples/render_window_smoke.rs)
when a compatible native GPU and live graphical session are available. The
`render-window` feature enables the local `surgeist-window` dependency from
[Cargo.toml](../Cargo.toml).

Run from the crate directory:

```sh
CARGO_NET_OFFLINE=true cargo run -p surgeist-render --example render_window_smoke --features render-window
```

The example opens a window, renders and presents one `DirectVello` frame followed
by one `GpuGraph` frame, asserts both reported routes, and exits after the two
successful presentations. It owns only its example lifecycle. An unavailable
graphical host prevents this check from passing; the unit tests' display-free
presentation harness does not substitute for this live-host evidence.

To exercise the same native example with both additive host features compiled,
run:

```sh
CARGO_NET_OFFLINE=true cargo run -p surgeist-render --example render_window_smoke --features render-window,render-web
```

The expected result is the same two presentations and successful exit. Enabling
`render-web` on this native run does not turn it into browser execution.

## Check the browser compilation boundary

Use an already-installed `wasm32-unknown-unknown` target and locally available
dependencies. From the crate directory, run:

```sh
CARGO_NET_OFFLINE=true cargo check -p surgeist-render --target wasm32-unknown-unknown --features render-web --lib --tests
```

A successful exit verifies compilation of the library and tests for this target
and feature selection. It does not run those tests or exercise a browser canvas,
event loop, or presentation. Those runtime checks require a browser host and
remain root integration evidence.

See the [reference](reference.md) for the feature matrix and the
[explanation](explanation.md) for the surface and repository boundaries.
