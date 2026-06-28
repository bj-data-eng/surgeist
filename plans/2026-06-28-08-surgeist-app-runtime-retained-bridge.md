# Surgeist App Runtime Foundation 08: Retained Bridge For Typed App Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the bridge from retained command reports into typed app inputs and diagnostics.

**Architecture:** This is split 08 of the numbered app-runtime foundation sequence. It should be implemented only after the earlier numbered splits are complete and reviewed, and it should preserve the typed app/work-plane boundary established by the sequence.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 8: Retained Bridge For Typed App Commands

**Files:**
- Create: `src/app/bridge.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/input.rs`
- Modify: `src/app/tests.rs`

- [ ] **Step 1: Write retained bridge success test**

Add:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
enum BridgeCommand {
    Open,
    OpenWithPayload(String),
}

#[test]
fn retained_bridge_decodes_registered_command() {
    let command_name = surgeist::retained::CommandName::new("open").unwrap();
    let mut bridge = RetainedBridge::<BridgeCommand>::new()
        .command(command_name.clone(), |_| Ok(BridgeCommand::Open));

    let retained = retained_command_for_test(command_name);
    let context = BridgeContext::new(
        SurfaceId::from_u64(1),
        retained.route().clone(),
        CorrelationId::from_u64(7),
    );
    let inputs = bridge
        .commands_to_inputs(context, std::slice::from_ref(&retained))
        .unwrap();

    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].payload(), &BridgeCommand::Open);
    assert_eq!(inputs[0].provenance().source(), &InputSourceId::RETAINED);
    assert_eq!(inputs[0].provenance().surface_id(), Some(SurfaceId::from_u64(1)));
    assert_eq!(inputs[0].provenance().correlation_id(), CorrelationId::from_u64(7));
}
```

- [ ] **Step 2: Write retained bridge diagnostic tests**

Add:

```rust
#[test]
fn retained_bridge_reports_unknown_command() {
    let command_name = surgeist::retained::CommandName::new("unknown").unwrap();
    let mut bridge = RetainedBridge::<BridgeCommand>::new();
    let retained = retained_command_for_test(command_name);
    let context = BridgeContext::new(
        SurfaceId::from_u64(1),
        retained.route().clone(),
        CorrelationId::from_u64(8),
    );

    let error = bridge
        .commands_to_inputs(context, std::slice::from_ref(&retained))
        .unwrap_err();

    assert_eq!(error.diagnostic().code(), &DiagnosticCode::UNKNOWN_RETAINED_COMMAND);
    assert_eq!(error.diagnostic().provenance().surface_id(), Some(SurfaceId::from_u64(1)));
}

#[test]
fn retained_bridge_reports_invalid_payload() {
    let command_name = surgeist::retained::CommandName::new("open").unwrap();
    let mut bridge = RetainedBridge::<BridgeCommand>::new()
        .command(command_name.clone(), |_| {
            Err(BridgeDecodeError::invalid_payload("expected folder id"))
        });
    let retained = retained_command_for_test(command_name);
    let context = BridgeContext::new(
        SurfaceId::from_u64(1),
        retained.route().clone(),
        CorrelationId::from_u64(9),
    );

    let error = bridge
        .commands_to_inputs(context, std::slice::from_ref(&retained))
        .unwrap_err();

    assert_eq!(error.diagnostic().code(), &DiagnosticCode::INVALID_RETAINED_PAYLOAD);
    assert!(error.diagnostic().message().contains("expected folder id"));
}
```

Add a crate-local helper at the bottom of the test module:

```rust
fn retained_command_for_test(name: surgeist::retained::CommandName) -> surgeist::retained::Command {
    let button = surgeist::retained::Element::tagged(
        surgeist::retained::Tag::new("button").unwrap(),
    )
    .with_hook(surgeist::retained::Hook::new(
        surgeist::retained::Trigger::Event(surgeist::retained::EventKind::Click),
        name,
    ));
    let mut model = surgeist::retained::Model::new(
        surgeist::retained::Element::root().with_child(button),
    )
    .unwrap();
    let target = model
        .snapshot()
        .children(model.root())
        .unwrap()
        .next()
        .unwrap();
    let report = model
        .dispatch(surgeist::retained::Event::new(
            target,
            surgeist::retained::EventKind::Click,
        ))
        .unwrap();
    report.commands()[0].clone()
}
```

- [ ] **Step 3: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing `RetainedBridge`.

- [ ] **Step 4: Implement bridge**

Add `bridge.rs` with:

- `RetainedBridge<T>` storing command decoders by `retained::CommandName`;
- `command(name, decoder)` builder;
- `BridgeContext { surface_id, route, correlation }`;
- `commands_to_inputs(context, commands)` returning `Vec<AppInput<T>>`;
- `BridgeError` carrying a `Diagnostic`;
- `BridgeDecodeError::invalid_payload(message)` mapped to `DiagnosticCode::INVALID_RETAINED_PAYLOAD`;
- diagnostics for unknown command names and decoder failures;
- provenance with surface id, retained source, route phase sequence, and the correlation id supplied through `BridgeContext`.

Update `input.rs` only if `AppInput::new` or `InputProvenance` needs a helper to preserve the `BridgeContext` correlation id. Do not invent a fallback correlation id in the bridge. Keep payload decoding closure-based in this slice. A generated command registry can target the same registration map.

- [ ] **Step 5: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app/bridge.rs src/app/input.rs src/app/tests.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 6: Commit**

```sh
git add src/app/mod.rs src/app/bridge.rs src/app/input.rs src/app/tests.rs
git commit -m "Add retained command bridge"
```

---
