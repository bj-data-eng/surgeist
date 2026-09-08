# Getting started

This guide exercises a native-window lifecycle contract without opening a native
display. You will observe creation, readiness, drawing, close acceptance, and a
successful terminal result through `testing::Runner`.

## Prerequisites

Use a local checkout with Rust and Cargo that support the Rust 2024 edition
declared in [Cargo.toml](../Cargo.toml). The dependency sources and native build
prerequisites for the host must already be available. The commands here use
`--offline` and do not acquire missing dependencies; a missing-cache error means
that prerequisite has not been met. No display session is needed for the
deterministic tests.

## First success

1. Open a terminal at the crate directory, which contains [Cargo.toml](../Cargo.toml).
2. Run the existing creation-order test:

   ```sh
   cargo test --offline -p surgeist-window --lib tests::test_runner_open_delivers_created_then_ready_like_native_pump -- --exact
   ```

3. Confirm that Cargo reports one passing test and zero failures. The
   [test implementation](../src/tests.rs) starts with an empty registry, resumes
   the runner, and asserts the callback sequence `["created", "ready"]`, a live
   window, and no terminal result.

The public import name is `surgeist_window`. Its
[public front door](../src/lib.rs) reexports the application types used below.

## Follow the complete lifecycle

`testing::Runner` uses the same callback lifecycle pump as the native host. It
lets a test choose when resume, draw, and close requests arrive. The following
example is also an executable rustdoc example in [src/lib.rs](../src/lib.rs).

```rust
use surgeist_window::{
    Close, Closed, Event, EventKind, Frame, Handler, Id, Ready, Result, open,
    testing::Runner,
};

#[derive(Default)]
struct Lifecycle {
    callbacks: Vec<&'static str>,
    id: Option<Id>,
    title: Option<String>,
}

impl Handler for Lifecycle {
    fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
        if matches!(event.event(), EventKind::Created(_)) {
            self.callbacks.push("created");
            self.id = Some(event.id());
            self.title = event.state().map(|state| state.title().to_owned());
        }
        Ok(())
    }

    fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
        self.callbacks.push("ready");
        assert_eq!(ready.state().title(), "Lifecycle");
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame<'_>) -> Result<()> {
        self.callbacks.push("draw");
        assert_eq!(frame.id(), self.id.expect("created window identity"));
        assert_eq!(frame.state().title(), "Lifecycle");
        Ok(())
    }

    fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
        self.callbacks.push("close");
        close.close();
        Ok(())
    }

    fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
        self.callbacks.push("closed");
        assert_eq!(closed.id(), self.id.expect("created window identity"));
        assert_eq!(closed.state().title(), "Lifecycle");
        closed.exit();
        Ok(())
    }
}

let mut runner = Runner::new(Lifecycle::default());
runner
    .startup(vec![open("main").title("Lifecycle").into()])
    .expect("startup is configured before the first resume");
runner.resume();

let id = runner.handler().id.expect("created window identity");
assert_eq!(runner.handler().id, Some(id));
assert_eq!(runner.handler().title.as_deref(), Some("Lifecycle"));
assert_eq!(runner.handler().callbacks, ["created", "ready"]);

runner.draw(id);
assert_eq!(runner.handler().callbacks, ["created", "ready", "draw"]);

runner.request_close(id);
assert_eq!(
    runner.handler().callbacks,
    ["created", "ready", "draw", "close", "closed"]
);
assert!(matches!(runner.result(), Some(Ok(()))));
```

Each assertion observes committed state or callback order. After `resume`, the
window exists and readiness has completed. Explicit `draw` adds the frame
callback. The accepted close delivers `closed` once, and `closed.exit()` leaves
`Some(Ok(()))` as the terminal result.

To execute the source-owned rustdoc examples, run:

```sh
cargo test --offline -p surgeist-window --doc
```

Expected result: the rustdoc examples pass, including the lifecycle assertions
and compile-fail examples that keep close requests and destruction out of the
generic event enum.

Continue with [how-to procedures](how-to.md) to configure startup, choose a test
harness, and use host services. The [explanation](explanation.md) describes why
authored commands and observed state are separate phases.
