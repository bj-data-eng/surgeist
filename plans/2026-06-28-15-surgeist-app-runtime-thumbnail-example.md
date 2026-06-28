# Surgeist App Runtime Foundation 15: Small Fake Thumbnail Import Example Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a small fake thumbnail import example that exercises the app runtime surface.

**Architecture:** This is split 15 of the numbered app-runtime foundation sequence. It should be implemented only after the earlier numbered splits are complete and reviewed, and it should preserve the typed app/work-plane boundary established by the sequence.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 15: Small Fake Thumbnail Import Example

**Files:**
- Create: `examples/app-thumbnail-import.rs`
- Modify: `Cargo.toml`
- Modify: `tests/app.rs`

- [ ] **Step 1: Write example compile test**

Add to `tests/app.rs`:

```rust
#[test]
fn thumbnail_import_example_contract_runs_headless() {
    let mut example = surgeist::app::testing::ThumbnailImportExample::new();

    example.choose_folder("/tmp/photos");
    example.drain_once();

    assert_eq!(example.initial_tile_count(), 3);
    assert_eq!(example.thumbnail_status(0), surgeist::app::ResourceStatus::Starting);

    example.finish_thumbnail(0);
    example.navigate_away();
    example.drain_all();

    assert_eq!(example.import_task_status(), surgeist::app::TaskStatus::Running);
    assert!(example.redrawn_surfaces().contains(&example.gallery_surface()));
}
```

- [ ] **Step 2: Add example manifest entry**

Add to `Cargo.toml`:

```toml
[[example]]
name = "app-thumbnail-import"
path = "examples/app-thumbnail-import.rs"
```

- [ ] **Step 3: Create example**

Create `examples/app-thumbnail-import.rs` demonstrating:

- an app-scoped fake import task registration;
- stable initial photo tile ids created synchronously;
- thumbnail resources moving through `Starting`, `Ready`, and `Refreshing`;
- task progress events becoming app inputs;
- navigation away removing observation while policy keeps useful work running;
- targeted redraw for the gallery surface.

The example should use the headless testing fixture for deterministic output. It should print a compact text summary:

```text
initial_tiles=3
thumbnail_0=ready
import=running
```

- [ ] **Step 4: Run failing test and example build**

Run:

```sh
cargo test --package surgeist --test app thumbnail_import_example_contract_runs_headless
cargo run -p surgeist --example app-thumbnail-import
```

Expected before implementation: the test or example compile fails because the example fixture is absent.

- [ ] **Step 5: Implement example fixture support**

Add `ThumbnailImportExample` to `testing.rs` if the example needs shared deterministic helpers. Keep the public example small and readable; the example should demonstrate the API, not contain test-only machinery.

- [ ] **Step 6: Verify**

Run:

```sh
cargo test --package surgeist --test app thumbnail_import_example_contract_runs_headless
cargo run -p surgeist --example app-thumbnail-import
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' examples/app-thumbnail-import.rs src/app/testing.rs tests/app.rs
```

Expected: test passes and example output contains:

```text
initial_tiles=3
thumbnail_0=ready
import=running
```

- [ ] **Step 7: Commit**

```sh
git add Cargo.toml examples/app-thumbnail-import.rs src/app/testing.rs tests/app.rs
git commit -m "Add app thumbnail import example"
```

---

