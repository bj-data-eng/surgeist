# surgeist-window

`surgeist-window` is Surgeist's Rust library for application authors who need
native windows, lifecycle callbacks, events, commands, capabilities, and handles.
It provides an app-facing boundary around `winit` and a deterministic lifecycle
runner that works without a native display. The current package is version
`0.1.0`; its [manifest](Cargo.toml) defines the available features and dependencies.
Renderers, surfaces, UI semantics, layout, hit testing, and application behavior
belong outside this crate.

## Start

From this checkout, exercise window creation and callback ordering:

```sh
cargo test --offline -p surgeist-window --lib tests::test_runner_open_delivers_created_then_ready_like_native_pump -- --exact
```

Expected result: one test passes, confirming that the deterministic runner opens
a window and delivers `Created` before `Ready` without opening a native display.
See [getting started](docs/getting-started.md) for prerequisites and the complete
lifecycle example.

## Documentation

| Guide | Purpose |
| --- | --- |
| [Getting started](docs/getting-started.md) | Run the first check and follow a complete window lifecycle. |
| [How-to](docs/how-to.md) | Configure startup, test callbacks, use clipboard services, and submit accessibility updates. |
| [Reference](docs/reference.md) | Find public interfaces, feature settings, units, and verification commands. |
| [Explanation](docs/explanation.md) | Understand planning phases, callback transactions, identity, and native ownership. |

## Stewardship

[MIT license](LICENSE) · [Third-party attribution](NOTICE.md)
