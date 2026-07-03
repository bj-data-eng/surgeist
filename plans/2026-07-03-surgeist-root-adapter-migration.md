# Surgeist Root Adapter Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move Surgeist-to-Surgeist lowering and integration adapters into the root `surgeist` crate so leaf crates own domain models and algorithms without tracking sibling crate implementation drift.

**Architecture:** Root `surgeist` becomes the integration adapter layer for CSS/style, retained/style, style/text, style/layout, fixture metadata, and future strict semantic tree lowering. Leaf crates keep backend-local adapters such as `text -> parley`, `render -> wgpu/kurbo`, and `window -> winit`, because those adapters are implementation details of the owning crate. Migration proceeds in compatibility-safe slices: introduce root adapters first, migrate callers/tests, remove leaf adapter modules/dependencies, then update root API artifacts and pointers only after green checks.

**Tech Stack:** Rust 2024, Cargo workspaces, current Surgeist sibling crates, root API generator, root `plans/` convention, `guidance/surgeist-rust-modeling-guide.md`.

---

## Boundary Rule

Use this rule to classify every adapter:

- Root-owned: conversions that compose two or more Surgeist crates, such as CSS syntax into style declarations, retained snapshots into style trees, resolved style into layout input, resolved style into text metric input, and retained/style/text/layout tree lowering.
- Leaf-owned: conversions from a crate's public model into that crate's private backend or algorithm dependency, such as `surgeist-text -> parley`, `surgeist-window -> winit`, and `surgeist-render -> wgpu/kurbo`.
- Test/root-owned: fixture schemas and reusable harnesses belong in `surgeist-test`; fixture generation that requires root adapters belongs in root. Leaf algorithm crates consume generated, layout-ready artifacts.

Do not move backend adapters into root. The goal is to remove sibling-crate lowering knowledge from leaf crates, not to make root a backend god object.

## Intended Post-Migration Dependency Shape

The migration must preserve an acyclic graph:

```text
root surgeist crate
  -> surgeist-css
  -> surgeist-dialog
  -> surgeist-layout
  -> surgeist-render
  -> surgeist-retained
  -> surgeist-shape
  -> surgeist-style
  -> surgeist-task
  -> surgeist-text
  -> surgeist-window

surgeist-css: no Surgeist crate dependencies after CSS-owned syntax migration
surgeist-style: no surgeist-css, surgeist-layout, surgeist-retained, or surgeist-text dependency after adapter migration
surgeist-layout: no surgeist-style, surgeist-retained, or root dependency in production or fixture support
surgeist-test: may depend on production leaf crates for harnesses and schemas, but must not depend on root surgeist
root: must not production-depend on surgeist-test; root tools may depend on surgeist-test
```

Root-owned fixture generation tools may use root adapters to write generated fixture metadata and may depend on public `surgeist-test` schema APIs. `surgeist-test` may define shared fixture schemas or harness helpers, but it must not call root adapters directly.

## Intermediate Root Visibility Gates

Some root adapter tasks depend on public APIs added by sibling crates earlier
in this plan. Before starting any root task that depends on a crate-local API
change, the root coordinator must:

- confirm the crate-local change is committed and pushed by the owning crate
  coordinator
- fetch or pull that crate's updated commit into the root submodule checkout
- do not commit root code that depends on the new crate API until Task 15
- keep reviewed root diffs in the working tree, together with the matching
  uncommitted submodule pointer updates, until the final pointer/API commit
- temporary uncommitted submodule pointer updates are allowed only while
  iterating; do not commit root code that requires a different submodule commit
  than the one recorded by that root commit
- run the focused root task checks against that working-tree pointer

Root ports that use only currently pinned public APIs may commit before leaf
adapter sources are deleted, as long as API artifacts are refreshed in the same
root commit. Root code that depends on new crate public APIs waits until Task
15. That final commit includes the remaining root adapter/tool changes,
matching submodule pointers, regenerated API artifacts, and full pointer-update
checks, so every committed root revision remains reproducible from a clean
checkout.

## Current Adapter Inventory

Known migration targets at plan time:

- `surgeist-style/src/adapters/layout.rs`: lowers `style::Resolved` into `layout::NodeInput` and `layout::LayoutCalcStore`.
- `surgeist-style/src/adapters/retained.rs`: implements `style::Tree` for `retained::Snapshot`, maps retained change flags to style changes, and clears style resolver caches for retained changes.
- `surgeist-css/src/lib.rs`: parses CSS directly into `surgeist-style` types.
- `surgeist-style/src/value.rs`: stores text-facing style values using `surgeist-text` types.
- `surgeist-layout` browser parity fixture support currently has dev-time dependencies on `surgeist-style` and `surgeist-retained`.

Known non-targets:

- `surgeist-window/src/winit_adapter.rs`
- `surgeist-text` conversions into `parley`
- `surgeist-render` conversions into rendering backend types

## Orphan Rule Constraint

Root cannot implement a foreign trait for a foreign type. For example, root cannot directly move:

```rust
impl style::Tree for retained::Snapshot<'_>
```

because root owns neither `style::Tree` nor `retained::Snapshot`. The migration must use root-owned wrapper types:

```rust
pub struct RetainedStyleTree<'a> {
    snapshot: retained::Snapshot<'a>,
}

impl style::Tree for RetainedStyleTree<'_> {
    // delegate to retained snapshot
}
```

Similarly, root cannot add inherent methods to `style::Change`. Existing leaf inherent adapter helpers must become root free functions or root adapter methods.

---

## Task 1: Update Governance And Public API Feasibility Gates

**Repo:** `/Users/codex/Development/surgeist`

**Files:**

- Modify: `/Users/codex/Development/surgeist/AGENTS.md`
- Modify: `/Users/codex/Development/surgeist/guidance/surgeist-rust-modeling-guide.md`
- Test/read: current public API reports under `/Users/codex/Development/surgeist/api/`

Steps:

- [ ] Verify root `AGENTS.md` crate roles say:
  - root owns Surgeist-to-Surgeist integration adapters
  - `surgeist-css` owns CSS syntax parsing, not style lowering
  - `surgeist-style` owns style model, cascade, resolution, property validation, and invalidation, not layout/text/retained lowering
  - `surgeist-test` owns schemas/harnesses/quality coordination, not root adapter execution
  - `surgeist-layout` owns layout-ready fixture consumption, not multi-crate fixture preparation
- [ ] Patch root `AGENTS.md` only if the current text has drifted from those roles.
- [ ] Verify root `AGENTS.md` dependency direction matches the post-migration shape in this plan, and patch only if drift is found.
- [ ] Verify the modeling guide crate prompts no longer say CSS lowers into style or style owns adapter contracts into layout/text/retained.
- [ ] Verify the modeling guide includes this adapter boundary, and patch only if drift is found:

```text
Surgeist-to-Surgeist lowering belongs in the root facade or in root-owned
tools. Leaf crates should expose front-door domain APIs and keep backend-local
adapters only when the backend is part of that crate's implementation.
```

- [ ] Add a public API feasibility checklist to this plan's task output:

```sh
rg -n "pub (struct|enum|trait|fn|type)|pub use" /Users/codex/Development/surgeist/crates/surgeist-{css,style,layout,retained,text}/src
```

- [ ] For each adapter migration target, list whether root can implement it using public front-door APIs:
  - style-to-layout
  - retained-to-style
  - CSS-to-style
  - style-to-text
  - fixture metadata generation
- [ ] If a target cannot be implemented through public APIs, create a crate-local follow-up task in this plan before the root adapter task. Do not expose private internals casually from root.

### Task 1 Feasibility Checklist

Public API scan command run from root:

```sh
rg -n "pub (struct|enum|trait|fn|type)|pub use" /Users/codex/Development/surgeist/crates/surgeist-{css,style,layout,retained,text}/src
```

- style-to-layout: feasible through current public front-door APIs. `surgeist-style`
  publicly exports `Resolved`, `Property`, `Value`, `Length`, calc/grid/style value
  types, and `surgeist-layout` publicly exports `NodeInput`, `LayoutCalcStore`, calc
  expressions, layout dimensions, grid placement, track sizing, and related layout
  enums used by the existing lowering code.
- retained-to-style: feasible through current public front-door APIs when implemented
  with a root-owned wrapper. `surgeist-retained` publicly exports `Snapshot`,
  `NodeRef`, `Kind`, `Tag`, `ProjectionSlot`, `ChangeFlags`, and `ChangeSet`;
  `surgeist-style` publicly exports `Tree`, `Node`, `Traversal`, `Change`, and
  `Resolver` cache-clearing methods needed by root free functions.
- CSS-to-style: not feasible as the intended root-owned adapter yet. Current
  `surgeist-css::parse_sheet` returns `surgeist_style::Sheet` directly and the crate
  has no public CSS-owned syntax model to adapt from. Task 7 is the required
  crate-local prerequisite before Task 8.
- style-to-text: not feasible as the intended dependency-removing adapter yet.
  Current `surgeist-style::TextValue` still stores `surgeist_text` public types and
  reexports text enums from style. Task 9 is the required crate-local prerequisite
  before Task 10.
- fixture metadata generation: feasible only after the planned schema/root-adapter
  prerequisites land. Current root can compose public leaf APIs for layout-ready
  values, but reusable fixture schemas are not yet exposed by `surgeist-test`; Task 11
  is the required crate-local prerequisite before Task 12, and Task 13 then moves
  layout parity support to consume layout-ready metadata.

Checks:

```sh
cargo fmt --check
```

After worker and reviewer are clean, the root coordinator commits this logical point. Workers do not commit.

---

## Task 2: Establish Root Adapter Module And Error Boundary

**Repo:** `/Users/codex/Development/surgeist`

**Files:**

- Create: `/Users/codex/Development/surgeist/src/adapters/mod.rs`
- Create: `/Users/codex/Development/surgeist/src/adapters/error.rs`
- Modify: `/Users/codex/Development/surgeist/src/lib.rs`
- Test: `/Users/codex/Development/surgeist/src/adapters/tests.rs`

Steps:

- [ ] Add `pub mod adapters;` to root `src/lib.rs`.
- [ ] Create `src/adapters/mod.rs` with module declarations:

```rust
//! Cross-crate Surgeist integration adapters.
//!
//! Backend-local adapters remain in their owning crates. This module only
//! composes public Surgeist crate contracts.

mod error;

#[cfg(test)]
mod tests;

pub use error::{AdapterBoundary, AdapterError, AdapterErrorKind, AdapterResult};
```

- [ ] Create `src/adapters/error.rs` with a root adapter error:

```rust
use std::{error, fmt};

pub type AdapterResult<T> = Result<T, AdapterError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterError {
    kind: AdapterErrorKind,
}

impl AdapterError {
    #[must_use]
    pub const fn new(kind: AdapterErrorKind) -> Self {
        Self { kind }
    }

    #[must_use]
    pub const fn kind(&self) -> &AdapterErrorKind {
        &self.kind
    }

    #[must_use]
    pub const fn boundary(&self) -> AdapterBoundary {
        self.kind.boundary()
    }
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {:?}", self.boundary(), self.kind)
    }
}

impl error::Error for AdapterError {}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterBoundary {
    CssToStyle,
    RetainedToStyle,
    StyleToText,
    StyleToLayout,
    StrictTreeToLayout,
    UnsupportedAdapterInput,
}

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterErrorKind {
    CssValueUnsupported { property: String, reason: String },
    CssStyleValidation { property: String, reason: String },
    RetainedTraversal { node: String, reason: String },
    RetainedChangeMapping { reason: String },
    StyleTextValue { property: String, reason: String },
    StyleLayoutValue { property: String, reason: String },
    StrictTreeInput { node: String, reason: String },
    UnsupportedAdapterInput { boundary: AdapterBoundary, reason: String },
}

impl AdapterErrorKind {
    #[must_use]
    pub const fn boundary(&self) -> AdapterBoundary {
        match self {
            Self::CssValueUnsupported { .. } | Self::CssStyleValidation { .. } => {
                AdapterBoundary::CssToStyle
            }
            Self::RetainedTraversal { .. } | Self::RetainedChangeMapping { .. } => {
                AdapterBoundary::RetainedToStyle
            }
            Self::StyleTextValue { .. } => AdapterBoundary::StyleToText,
            Self::StyleLayoutValue { .. } => AdapterBoundary::StyleToLayout,
            Self::StrictTreeInput { .. } => AdapterBoundary::StrictTreeToLayout,
            Self::UnsupportedAdapterInput { boundary, .. } => *boundary,
        }
    }
}
```

- [ ] Add `src/adapters/tests.rs`:

```rust
use super::{AdapterBoundary, AdapterError, AdapterErrorKind};

#[test]
fn adapter_error_exposes_kind_and_boundary() {
    let error = AdapterError::new(AdapterErrorKind::StyleLayoutValue {
        property: "width".to_owned(),
        reason: "bad width".to_owned(),
    });
    assert_eq!(error.boundary(), AdapterBoundary::StyleToLayout);
    assert!(matches!(
        error.kind(),
        AdapterErrorKind::StyleLayoutValue { property, .. } if property == "width"
    ));
}
```

Checks:

```sh
cargo fmt --check
cargo test -p surgeist adapter_error_exposes_kind_and_boundary
cargo clippy -p surgeist --all-targets -- -D warnings
cargo run --manifest-path api/generator/Cargo.toml
cargo run --manifest-path api/generator/Cargo.toml -- --check
```

After worker and reviewer are clean, the root coordinator commits this logical point with the matching root API artifact update. Workers do not commit.

---

## Task 3: Migrate Style-To-Layout Lowering Into Root

**Repo:** `/Users/codex/Development/surgeist`

**Files:**

- Create: `/Users/codex/Development/surgeist/src/adapters/style_layout.rs`
- Modify: `/Users/codex/Development/surgeist/src/adapters/mod.rs`
- Test: `/Users/codex/Development/surgeist/src/adapters/style_layout_tests.rs`
- Reference source: `/Users/codex/Development/surgeist/crates/surgeist-style/src/adapters/layout.rs`

Steps:

- [ ] Copy the behavior of `surgeist-style/src/adapters/layout.rs` into root `src/adapters/style_layout.rs`.
- [ ] Record the current `surgeist-style` submodule commit SHA in the task output as the source revision used for the port.
- [ ] Before copying code, confirm every imported style/layout item is public through each crate's `lib.rs`. If not, stop and add a crate-local front-door API task before continuing.
- [ ] Rename public types to root-owned names:
  - `LayoutLoweringOutput` stays `LayoutLoweringOutput`
  - `LayoutLoweringSession` stays `LayoutLoweringSession`
  - `lower` becomes `lower_style_to_layout`
  - `lower_with_store` becomes `lower_style_to_layout_with_store`
- [ ] Convert return types from `style::Result<T>` to `AdapterResult<T>`.
- [ ] Map style validation/lowering failures to `AdapterErrorKind::StyleLayoutValue { property, reason }`.
- [ ] Keep calc store handling in root. Root is the composition layer that can see both style calc values and layout calc stores.
- [ ] Export the adapter from `src/adapters/mod.rs`:

```rust
mod style_layout;

pub use style_layout::{
    LayoutLoweringOutput, LayoutLoweringSession, lower_style_to_layout,
    lower_style_to_layout_with_store,
};
```

- [ ] Port the existing style layout adapter tests into root. Preserve test intent and expected values exactly unless root-owned names require mechanical updates.
- [ ] Add a regression test that a bad layout-specific style value maps to `AdapterBoundary::StyleToLayout` with `AdapterErrorKind::StyleLayoutValue`.

Checks:

```sh
cargo fmt --check
cargo test -p surgeist style_layout
cargo clippy -p surgeist --all-targets -- -D warnings
cargo run --manifest-path api/generator/Cargo.toml
cargo run --manifest-path api/generator/Cargo.toml -- --check
```

After worker and reviewer are clean, the root coordinator commits this root adapter port before Task 4 removes the leaf source, with matching API artifact updates. Workers do not commit.

---

## Task 4: Remove Style-To-Layout Adapter From Style

**Repo:** `/Users/codex/Development/surgeist-style`

**Files:**

- Modify: `/Users/codex/Development/surgeist-style/src/adapters/mod.rs`
- Delete: `/Users/codex/Development/surgeist-style/src/adapters/layout.rs`
- Modify: `/Users/codex/Development/surgeist-style/src/lib.rs`
- Modify: `/Users/codex/Development/surgeist-style/Cargo.toml`
- Modify tests that import `style::adapters::layout`

Steps:

- [ ] Remove `pub mod layout;` from `src/adapters/mod.rs`.
- [ ] Delete `src/adapters/layout.rs`.
- [ ] If `src/adapters/mod.rs` becomes retained-only after this task, keep it temporarily. Task 6 removes the retained adapter.
- [ ] Remove `surgeist-layout` from `[dependencies]` in `Cargo.toml`.
- [ ] Search for remaining layout dependency:

```sh
rg -n "surgeist_layout|surgeist-layout|adapters::layout|LayoutLowering|lower_with_store|lower\\(" src tests Cargo.toml
```

- [ ] Update or delete tests that only tested the old style-owned layout adapter. Do not weaken style model tests.
- [ ] Confirm style still owns `Resolved`, `Length`, calc values, selectors, declarations, sheets, and validation.

Checks:

```sh
cargo fmt --check
cargo test -p surgeist-style
cargo clippy -p surgeist-style --all-targets -- -D warnings
```

After worker and reviewer are clean, the style coordinator commits this logical point. Workers do not commit.

---

## Task 5: Migrate Retained-To-Style Tree Adapter Into Root

**Repo:** `/Users/codex/Development/surgeist`

**Files:**

- Create: `/Users/codex/Development/surgeist/src/adapters/retained_style.rs`
- Modify: `/Users/codex/Development/surgeist/src/adapters/mod.rs`
- Test: `/Users/codex/Development/surgeist/src/adapters/retained_style_tests.rs`
- Reference source: `/Users/codex/Development/surgeist/crates/surgeist-style/src/adapters/retained.rs`

Steps:

- [ ] Create a root-owned wrapper:

```rust
pub struct RetainedStyleTree<'a> {
    snapshot: retained::Snapshot<'a>,
}

impl<'a> RetainedStyleTree<'a> {
    #[must_use]
    pub const fn new(snapshot: retained::Snapshot<'a>) -> Self {
        Self { snapshot }
    }
}
```

- [ ] Implement `style::Tree` for `RetainedStyleTree<'_>` by porting the current retained snapshot behavior from style.
- [ ] Record the current `surgeist-style` submodule commit SHA in the task output as the source revision used for the port.
- [ ] Before porting code, confirm every retained/style item required by the wrapper is public through each crate's `lib.rs`. If not, stop and add a crate-local front-door API task before continuing.
- [ ] Port `tag_for_kind` and retained error mapping as private root adapter helpers.
- [ ] Replace `impl Change { from_retained_flags(...) }` with a root free function:

```rust
#[must_use]
pub fn style_change_from_retained_flags(flags: retained::ChangeFlags) -> style::Change;
```

- [ ] Replace `Resolver::clear_cache_for_changes` with a root free function:

```rust
pub fn clear_style_cache_for_retained_changes(
    resolver: &mut style::Resolver,
    changes: &retained::ChangeSet,
);
```

- [ ] Export these items from `src/adapters/mod.rs`.
- [ ] Port retained adapter tests into root and add one explicit orphan-rule regression test that uses `RetainedStyleTree::new(snapshot)` with `style::Resolver`.

Checks:

```sh
cargo fmt --check
cargo test -p surgeist retained_style
cargo clippy -p surgeist --all-targets -- -D warnings
cargo run --manifest-path api/generator/Cargo.toml
cargo run --manifest-path api/generator/Cargo.toml -- --check
```

After worker and reviewer are clean, the root coordinator commits this root adapter port before Task 6 removes the leaf source, with matching API artifact updates. Workers do not commit.

---

## Task 5A: Split Style Tree And Selector Vocabulary From Retained

**Repo:** `/Users/codex/Development/surgeist-style`

**Why this task exists:** Task 6's dependency search found `surgeist-retained`
usage outside `src/adapters/retained.rs`. Style's public tree and selector
contracts currently use retained `Tag`, `Class`, `Key`, `Attribute`, `State`,
and `Role` types. That is a style-domain modeling dependency, not adapter
residue, so deleting the retained adapter alone would leave the crate coupled
to retained.

**Files:**

- Create or modify: `/Users/codex/Development/surgeist-style/src/identity.rs`
- Modify: `/Users/codex/Development/surgeist-style/src/tree.rs`
- Modify: `/Users/codex/Development/surgeist-style/src/selector.rs`
- Modify: `/Users/codex/Development/surgeist-style/src/sheet.rs`
- Modify: `/Users/codex/Development/surgeist-style/src/resolver.rs`
- Modify: `/Users/codex/Development/surgeist-style/src/lib.rs`
- Modify style tests that construct retained-backed trees directly

Steps:

- [ ] Record the current dependency search output before edits:

```sh
rg -n "surgeist_retained|surgeist-retained|retained::" src tests Cargo.toml
```

- [ ] Add style-owned identity/fact types for style selector and tree matching:
  - `StyleTag`
  - `StyleClass`
  - `StyleKey`
  - `StyleAttributeName`
  - `StyleAttributeValue`
  - `StyleAttribute`
  - `StyleRole`
  - `StyleState`
- [ ] Preserve current validation semantics from retained types. If a retained
  constructor rejected an empty or malformed value, the equivalent style-owned
  constructor must reject it too.
- [ ] Update `Selector`, `Compound`, `AttributeSelector`, `PrimaryKey`,
  `Sheet` indexes, `Tree::Node`, and resolver cache hashing to use style-owned
  types.
- [ ] Keep these types style-domain-facing. Do not make them wrappers around
  retained types internally.
- [ ] Keep selector string constructors such as `Selector::tag`,
  `Selector::class`, `Selector::key`, and attribute selector constructors
  working with the same validation behavior.
- [ ] Update tests so style model tests use style-owned types or simple test
  trees instead of retained models except where a retained adapter test still
  intentionally covers the temporary style-owned retained adapter.
- [ ] Do not remove `src/adapters/retained.rs` in this task; Task 6 removes it
  after root has a matching adapter update.
- [ ] Run dependency search again. Remaining `surgeist-retained` usage must be
  limited to `src/adapters/retained.rs`, adapter tests, temporary tests that
  explicitly exercise that adapter, and the temporary `Cargo.toml` dependency
  required by that adapter until Task 6 removes it.

Checks:

```sh
cargo fmt --check
cargo test -p surgeist-style
cargo clippy -p surgeist-style --all-targets -- -D warnings
```

After worker and reviewer are clean, the style coordinator commits this logical point and pushes it. Workers do not commit.

---

## Task 5B: Update Root Retained Adapter For Style-Owned Tree Facts

**Repo:** `/Users/codex/Development/surgeist`

**Files:**

- Modify: `/Users/codex/Development/surgeist/src/adapters/retained_style.rs`
- Modify: `/Users/codex/Development/surgeist/src/adapters/retained_style_tests.rs`
- Modify generated API artifacts under `/Users/codex/Development/surgeist/api/`

Steps:

- [ ] Confirm Task 5A is committed and pushed by the style coordinator.
- [ ] Fetch or pull the updated `surgeist-style` commit into the root
  submodule checkout. Do not commit the pointer yet.
- [ ] Update `RetainedStyleTree` so retained `Tag`, `Class`, `Key`,
  attributes, role, and state are converted into style-owned tree facts at the
  root adapter boundary.
- [ ] Keep conversions explicit and local to the root adapter. Do not add a
  retained dependency back into style.
- [ ] Preserve cache invalidation behavior from Task 5.
- [ ] Preserve the orphan-rule wrapper shape: root still implements
  `style::Tree` for `RetainedStyleTree<'_>`, not for retained snapshot types.
- [ ] Add or update tests proving retained nodes resolve through style-owned
  selector/tree facts, including at least tag, class, key, attribute, state,
  and text-node matching.

Checks:

```sh
cargo fmt --check
cargo test -p surgeist retained_style
cargo clippy -p surgeist --all-targets -- -D warnings
cargo run --manifest-path api/generator/Cargo.toml
cargo run --manifest-path api/generator/Cargo.toml -- --check
```

After worker and reviewer are clean, the root coordinator keeps this reviewed root adapter/API-artifact diff uncommitted until Task 15 because it depends on crate API changes. Workers do not commit.

---

## Task 6: Remove Retained Adapter From Style

**Repo:** `/Users/codex/Development/surgeist-style`

**Files:**

- Delete: `/Users/codex/Development/surgeist-style/src/adapters/retained.rs`
- Modify: `/Users/codex/Development/surgeist-style/src/adapters/mod.rs`
- Modify: `/Users/codex/Development/surgeist-style/src/lib.rs`
- Modify: `/Users/codex/Development/surgeist-style/Cargo.toml`
- Modify tests that import retained adapter behavior

Steps:

- [ ] Confirm Task 5A is committed and pushed by the style coordinator.
- [ ] Delete `src/adapters/retained.rs`.
- [ ] Delete `src/adapters/mod.rs` if no adapter modules remain.
- [ ] Remove `pub mod adapters;` from `src/lib.rs` if the adapters module is empty.
- [ ] Remove `surgeist-retained` from `[dependencies]` if no non-adapter style modules still need it.
- [ ] Run dependency search:

```sh
rg -n "surgeist_retained|surgeist-retained|retained::|adapters::retained|from_retained_flags|clear_cache_for_changes" src tests Cargo.toml
```

- [ ] If non-adapter style modules still depend on retained types for selectors/tree contracts, stop and report the remaining dependency as a separate modeling issue. Do not hide it behind new adapters.

Checks:

```sh
cargo fmt --check
cargo test -p surgeist-style
cargo clippy -p surgeist-style --all-targets -- -D warnings
```

After worker and reviewer are clean, the style coordinator commits this logical point. Workers do not commit.

---

## Task 7: Make CSS Parse Into CSS-Owned Syntax Types

**Repo:** `/Users/codex/Development/surgeist-css`

**Files:**

- Create: `/Users/codex/Development/surgeist-css/src/syntax.rs`
- Modify: `/Users/codex/Development/surgeist-css/src/lib.rs`
- Modify: `/Users/codex/Development/surgeist-css/Cargo.toml`
- Modify parser tests in `/Users/codex/Development/surgeist-css/src/lib.rs` or moved test modules

Steps:

- [ ] Add CSS-owned syntax types in `src/syntax.rs`: `CssSheet`, `CssRule`, `CssDeclaration`, `CssProperty`, `CssValue`, `CssLength`, `CssEdges`, `CssColor`, selector syntax types, and calc syntax types for the currently supported property set.
- [ ] Before deleting direct style construction, record the current `surgeist-css` commit SHA in the task output as the behavior source for the future root CSS-to-style port.
- [ ] Capture the current parser-to-style expectation surface before migration:

```sh
rg -n "surgeist_style|style::|Sheet|Rule|Declaration|Value|Length|parse_sheet|assert" src tests
```

- [ ] List every parser test whose expected output currently uses style types, so Task 8 can port the same expectations into root adapter tests after CSS returns CSS-owned syntax.
- [ ] Document in `syntax.rs` that `CssValue` is authored syntax and must not become a broad cross-property validation bag.
- [ ] Keep the CSS syntax types authored/parser-facing. Do not import `surgeist-style` in `syntax.rs`.
- [ ] Change `parse_sheet` to return `surgeist_css::CssSheet`.
- [ ] Replace direct construction of `surgeist_style::{Sheet, Rule, Declaration, Value, Length, ...}` with CSS-owned syntax construction.
- [ ] Keep strict property-specific parsers in CSS. Do not use one broad `parse_length` for every property. Add dedicated parsers for at least:
  - box size values
  - margin components
  - padding components
  - border width components
  - gap values
  - font-size
  - line-height
- [ ] For line-height, accept only the strict supported set: `normal`, `px`, `%`, `0`, and `calc(...)` if calc is retained as authored syntax. Reject `auto`, `min-content`, `max-content`, and `fit-content`.
- [ ] For padding and border width, reject `auto` and intrinsic sizing keywords.
- [ ] For margin, accept `auto` but reject intrinsic sizing keywords.
- [ ] Remove `surgeist-style` from `surgeist-css/Cargo.toml`.
- [ ] Add parser tests proving invalid property-specific keyword leakage is rejected:
  - `line-height: auto`
  - `line-height: min-content`
  - `font-size: auto`
  - `padding: auto`
  - `border-width: 10%`
  - `gap: auto`
  - `margin: auto` remains accepted

Checks:

```sh
cargo fmt --check
cargo test -p surgeist-css
cargo clippy -p surgeist-css --all-targets -- -D warnings
```

After worker and reviewer are clean, the CSS coordinator commits this logical point. Workers do not commit.

---

## Task 8: Add Root CSS-To-Style Adapter

**Repo:** `/Users/codex/Development/surgeist`

**Files:**

- Create: `/Users/codex/Development/surgeist/src/adapters/css_style.rs`
- Modify: `/Users/codex/Development/surgeist/src/adapters/mod.rs`
- Test: `/Users/codex/Development/surgeist/src/adapters/css_style_tests.rs`

Steps:

- [ ] Confirm Task 7 is committed and pushed by the CSS coordinator.
- [ ] Fetch or pull the updated `surgeist-css` commit into the root submodule checkout. Do not commit the pointer yet.
- [ ] Use the pre-removal `surgeist-css` source SHA and parser-to-style expectation list recorded by Task 7 as the behavior source for this port. If Task 7 did not record them, stop and ask the CSS coordinator to provide a source commit/diff before continuing.
- [ ] Confirm every CSS/style item required by this adapter is public through each crate's `lib.rs`. If not, stop and add a crate-local front-door API task before continuing.
- [ ] Add `lower_css_sheet_to_style(sheet: &css::CssSheet) -> AdapterResult<style::Sheet>`.
- [ ] Add private helpers for each syntax-to-style conversion:
  - selector lowering
  - property lowering
  - value lowering
  - length lowering
  - edges lowering
  - calc lowering
  - color lowering
- [ ] Map CSS syntax values into style model values using only public `surgeist-css` and `surgeist-style` APIs.
- [ ] Map unsupported or invalid conversions to `AdapterErrorKind::CssValueUnsupported` or `AdapterErrorKind::CssStyleValidation`.
- [ ] Export `lower_css_sheet_to_style` from `src/adapters/mod.rs`.
- [ ] Port previous CSS parser assertions that expected style values into root adapter tests.
- [ ] Add a full-path test:

```rust
#[test]
fn css_sheet_lowers_to_style_sheet() {
    let css = css::parse_sheet(".panel { width: 12px; margin: auto; }").unwrap();
    let sheet = super::lower_css_sheet_to_style(&css).unwrap();
    assert_eq!(sheet.rules().len(), 1);
}
```

Checks:

```sh
cargo fmt --check
cargo test -p surgeist css_style
cargo clippy -p surgeist --all-targets -- -D warnings
```

After worker and reviewer are clean, the root coordinator keeps this reviewed root adapter diff uncommitted until Task 15 because it depends on crate API changes. Workers do not commit.

---

## Task 9: Split Style Text Model From Text Crate Contracts

**Repo:** `/Users/codex/Development/surgeist-style`

**Files:**

- Modify: `/Users/codex/Development/surgeist-style/src/value.rs`
- Modify: `/Users/codex/Development/surgeist-style/src/lib.rs`
- Modify: `/Users/codex/Development/surgeist-style/src/declaration.rs`
- Modify: `/Users/codex/Development/surgeist-style/Cargo.toml`
- Modify style tests that reference `surgeist_text::*`

Steps:

- [ ] Replace direct `surgeist_text` enum fields in `TextValue` with style-owned enums:
  - `TextWeight`
  - `TextSlant`
  - `StyleTextAlign`
  - `TextWrap`
  - `WhiteSpace`
  - `WordBreak`
  - `OverflowWrap`
  - `Decoration`
- [ ] Preserve existing variant names where possible.
- [ ] Before deleting direct `surgeist-text` fields or reexports, record the current `surgeist-style` commit SHA in the task output as the behavior source for the future root style-to-text port.
- [ ] Capture the current style-to-text expectation surface before migration:

```sh
rg -n "surgeist_text|TextValue|TextWeight|TextSlant|TextAlign|TextWrap|WhiteSpace|WordBreak|OverflowWrap|Decoration|assert" src tests
```

- [ ] List every style test whose expected output currently uses `surgeist-text` types, so Task 10 can port the same expectations into root adapter tests after style owns its text-facing enums.
- [ ] Keep style-owned values authored/resolved-style-facing, not shaping-facing.
- [ ] Remove public reexports from `surgeist_text` in `src/lib.rs`.
- [ ] Remove `surgeist-text` from `[dependencies]` if no non-adapter style code still needs it.
- [ ] Add tests proving style text defaults and validation did not change.
- [ ] Run dependency search:

```sh
rg -n "surgeist_text|surgeist-text" src tests Cargo.toml
```

- [ ] If remaining uses are not adapter-related, stop and report the modeling issue. Do not add a convenience dependency back.

Checks:

```sh
cargo fmt --check
cargo test -p surgeist-style
cargo clippy -p surgeist-style --all-targets -- -D warnings
```

After worker and reviewer are clean, the style coordinator commits this logical point. Workers do not commit.

---

## Task 10: Add Root Style-To-Text Adapter

**Repo:** `/Users/codex/Development/surgeist`

**Files:**

- Create: `/Users/codex/Development/surgeist/src/adapters/style_text.rs`
- Modify: `/Users/codex/Development/surgeist/src/adapters/mod.rs`
- Test: `/Users/codex/Development/surgeist/src/adapters/style_text_tests.rs`

Steps:

- [ ] Confirm Task 9 is committed and pushed by the style coordinator.
- [ ] Fetch or pull the updated `surgeist-style` commit into the root submodule checkout. Do not commit the pointer yet.
- [ ] Use the pre-removal `surgeist-style` source SHA and style-to-text expectation list recorded by Task 9 as the behavior source for this port. If Task 9 did not record them, stop and ask the style coordinator to provide a source commit/diff before continuing.
- [ ] Confirm every style/text item required by this adapter is public through each crate's `lib.rs`. If not, stop and add a crate-local front-door API task before continuing.
- [ ] Add `lower_style_text_to_text_style(value: &style::TextValue) -> AdapterResult<text::Style>`.
- [ ] Add conversion helpers for every style-owned text enum introduced in Task 9.
- [ ] Keep text metric derivation in `surgeist-text`. Root only adapts style's resolved text properties into text's public input type.
- [ ] Add tests for every enum conversion.
- [ ] Add a test that default `style::TextValue` lowers to default-compatible `text::Style`.
- [ ] Export `lower_style_text_to_text_style` from `src/adapters/mod.rs`.

Checks:

```sh
cargo fmt --check
cargo test -p surgeist style_text
cargo clippy -p surgeist --all-targets -- -D warnings
```

After worker and reviewer are clean, the root coordinator keeps this reviewed root adapter diff uncommitted until Task 15 because it depends on crate API changes. Workers do not commit.

---

## Task 11: Keep Shared Fixture Schemas In Surgeist-Test

**Repo:** `/Users/codex/Development/surgeist-test`

**Files:**

- Modify: `/Users/codex/Development/surgeist-test/Cargo.toml`
- Create or modify: `/Users/codex/Development/surgeist-test/src/fixtures/**`
- Modify: `/Users/codex/Development/surgeist-test/README.md`

Steps:

- [ ] Add shared fixture schema types or helpers for layout-ready metadata.
- [ ] Keep the schema module layout-independent. Any schema consumed by `surgeist-layout` must not depend on `surgeist-layout`.
- [ ] Do not depend on root `surgeist`.
- [ ] Do not call root adapters.
- [ ] If a fixture helper needs multi-crate production types, depend only on leaf crates and keep the helper schema-focused.
- [ ] Do not add any `surgeist-layout` dependency to `surgeist-test`, including optional dependencies or feature-gated helper modules. This avoids a Cargo package cycle when layout consumes `surgeist-test` schemas.
- [ ] Document that root generates adapter-composed fixture metadata, `surgeist-test` owns reusable schemas/harnesses, and layout consumes layout-ready fixture metadata.

Checks:

```sh
cargo fmt --check
cargo test -p surgeist-test
cargo clippy -p surgeist-test --all-targets -- -D warnings
```

After worker and reviewer are clean, the test coordinator commits this logical point. Workers do not commit.

---

## Task 12: Add Root Fixture Metadata Generation Tool

**Repo:** `/Users/codex/Development/surgeist`

**Files:**

- Create: `/Users/codex/Development/surgeist/tools/surgeist-fixture-metadata/Cargo.toml`
- Create: `/Users/codex/Development/surgeist/tools/surgeist-fixture-metadata/src/main.rs`
- Create: `/Users/codex/Development/surgeist/tools/surgeist-fixture-metadata/src/lib.rs`
- Test: `/Users/codex/Development/surgeist/tools/surgeist-fixture-metadata/tests/fixture_metadata.rs`
- Modify: `/Users/codex/Development/surgeist/Cargo.toml`
- Modify docs if fixture commands are documented in `/Users/codex/Development/surgeist/README.md`

Steps:

- [ ] Confirm Task 11 is committed and pushed by the test coordinator.
- [ ] Confirm root adapter tasks that this tool uses are reviewed and present in the root working tree.
- [ ] Fetch or pull any updated crate commits the tool depends on into root submodule checkouts. Do not commit pointers yet.
- [ ] Add a root-owned tool crate under `tools/surgeist-fixture-metadata`.
- [ ] Add the tool crate to root workspace members.
- [ ] The tool crate may depend on root `surgeist` and public `surgeist-test` schema APIs. Root `surgeist` production code must not depend on `surgeist-test`.
- [ ] Root `surgeist` must not depend on this tool crate or on `surgeist-test`.
- [ ] Add fixture metadata generation that uses root adapters to compose CSS, style, text, retained, and layout-ready metadata.
- [ ] Generate layout-ready fixture attributes such as `surgeist-inline-line-height` and `surgeist-inline-baseline` using root adapters and text-owned metric derivation.
- [ ] Do not make `surgeist-test` depend on root.
- [ ] Do not make `surgeist-layout` depend on root.
- [ ] Repeat the public front-door API gate for every crate the tool composes:

```sh
rg -n "pub (struct|enum|trait|fn|type)|pub use" /Users/codex/Development/surgeist/crates/surgeist-{css,style,layout,retained,text}/src
```

- [ ] Add tests proving fixture metadata generation rejects invalid CSS property values before layout fixture output is produced.
- [ ] Keep fixture metadata helpers inside the tool crate unless a documented root public API needs them.
- [ ] Add a test proving generated metadata conforms to the public `surgeist-test` schema API.

Checks:

```sh
cargo fmt --check
cargo test -p surgeist-fixture-metadata
cargo clippy -p surgeist-fixture-metadata --all-targets -- -D warnings
```

After worker and reviewer are clean, the root coordinator keeps this reviewed root tool diff uncommitted until Task 15 because it depends on crate API changes. Workers do not commit.

---

## Task 13: Remove Multi-Crate Dev Lowering From Layout

**Repo:** `/Users/codex/Development/surgeist-layout`

**Files:**

- Modify: `/Users/codex/Development/surgeist-layout/Cargo.toml`
- Modify: `/Users/codex/Development/surgeist-layout/tests/layout/browser_parity/support.rs`
- Modify: `/Users/codex/Development/surgeist-layout/tests/layout/browser_parity/README.md`

Steps:

- [ ] Update browser parity support to consume layout-ready fixture metadata that conforms to the `surgeist-test` schema. This task must not require the root fixture metadata tool to be committed first.
- [ ] Use committed schema/sample metadata from `surgeist-test` for layout-side tests until root-generated fixture artifacts are available.
- [ ] Remove dev-time style/retained lowering from layout fixture support.
- [ ] Remove `surgeist-style` and `surgeist-retained` dev-dependencies from `surgeist-layout/Cargo.toml` if no layout-owned tests still need them.
- [ ] Keep the layout oracle and layout algorithm tests in layout.
- [ ] Run dependency search:

```sh
rg -n "surgeist_style|surgeist-style|surgeist_retained|surgeist-retained|style::|retained::" tests src Cargo.toml
```

- [ ] If any remaining dependency is required for layout-owned contracts, document it in the task output. Otherwise remove it.

Checks:

```sh
cargo fmt --check
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
```

After worker and reviewer are clean, the layout coordinator commits this logical point. Workers do not commit.

---

## Task 14: Update Root Governance Docs

**Repo:** `/Users/codex/Development/surgeist`

**Files:**

- Modify: `/Users/codex/Development/surgeist/README.md`
- Modify: `/Users/codex/Development/surgeist/AGENTS.md`

Steps:

- [ ] Document the adapter boundary in root README:
  - root owns Surgeist-to-Surgeist adapters
  - leaf crates own domain models and backend-local adapters
  - adapter-composed fixture generation belongs in root-owned tools
  - reusable fixture schemas and harnesses belong in `surgeist-test`
- [ ] Confirm `AGENTS.md` and the modeling guide still match the final dependency direction after implementation.
- [ ] Do not edit root facade code in this task. Root facade/API code that depends on new crate APIs is committed in Task 15 with matching pointers and API artifacts.
- [ ] Do not regenerate API artifacts in this task. API artifacts are regenerated in Task 15 after pointer updates make crate source changes visible to root.

Checks:

```sh
cargo fmt --check
```

After worker and reviewer are clean, the root coordinator commits this logical point. Workers do not commit.

---

## Task 15: Cross-Crate Verification, API Artifacts, And Pointer Update

**Repo:** `/Users/codex/Development/surgeist`

**Coordinator-only:** This task updates root submodule pointers, generated API artifacts, and root commits. Do not assign it to an implementation worker.

Prerequisites:

- [ ] `surgeist-css` changes are committed and pushed.
- [ ] `surgeist-style` changes are committed and pushed.
- [ ] `surgeist-layout` changes are committed and pushed.
- [ ] `surgeist-test` changes are committed and pushed.
- [ ] Each owning crate coordinator reported focused crate checks and reviewer results clean.
- [ ] Independent root setup/docs changes are committed locally.
- [ ] Root adapter/tool/facade diffs that depend on new crate APIs are reviewed, clean, and present in the working tree.

Steps:

- [ ] Fetch every changed submodule remote and confirm each target commit is fetchable from its configured remote.
- [ ] Update submodule pointers to the committed crate revisions.
- [ ] Review:

```sh
git diff --submodule=log
```

- [ ] Confirm crate-local focused checks reported by crate coordinators cover the changed crates. Re-run any missing focused check before continuing.
- [ ] Run adapter/dependency discovery and confirm no migration target was missed:

```sh
rg -n "pub mod adapters|mod adapters|adapter|lower\\(|lower_|Lowering|surgeist_layout|surgeist_style|surgeist_text|surgeist_retained" src crates/*/src crates/*/tests Cargo.toml crates/*/Cargo.toml
cargo tree -p surgeist-layout --edges normal,dev,build
cargo tree -p surgeist-test --edges normal,dev,build
cargo metadata --format-version 1 --no-deps
```

- [ ] Inspect the discovery output for forbidden remaining production or fixture-support edges:
  - `surgeist-css` must not depend on `surgeist-style`.
  - `surgeist-style` must not depend on `surgeist-css`, `surgeist-layout`, `surgeist-retained`, or `surgeist-text`.
  - `surgeist-layout` production code and browser fixture support must not depend on `surgeist-style`, `surgeist-retained`, or root `surgeist`.
  - root `surgeist` production code must not depend on `surgeist-test`.
  - `surgeist-test` must not depend on `surgeist-layout` in any active normal, dev, build, optional, or feature-gated edge used by the workspace checks.
- [ ] Inspect `cargo metadata` for package cycles and feature-activated optional edges involving `surgeist-layout` and `surgeist-test`; fail the pointer update if any active layout/test cycle exists.

- [ ] Run root checks:

```sh
cargo run --manifest-path api/generator/Cargo.toml
git diff -- api/
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run --manifest-path api/generator/Cargo.toml -- --check
cargo metadata --format-version 1 --no-deps
```

- [ ] Inspect `cargo metadata` output and fail the pointer update if root dependencies use sibling checkout paths outside the root workspace instead of `crates/*` submodule paths.
- [ ] Commit green root adapter/tool/facade diffs, matching submodule pointers, and API artifact updates together in root.
- [ ] Push root if the user requests publication or other coordinators need the updated pointers.

Exception: do not commit red pointers unless the user explicitly requests a red pointer update and the task output names the failing command, owning issue, and reason the red pointer is intentional.

---

## Review Requirements

Each implementation task must use the normal coordinator workflow:

- Root-owned tasks are implemented in the root `surgeist` project.
- Crate-owned tasks are assigned to that crate's coordinator/project. The root
  coordinator sends the scoped task prompt, relevant files, commands, and
  constraints, then waits for the crate coordinator to report clean worker and
  reviewer results, commits, and pushes.
- Do not edit sibling submodule contents from the root project to complete a
  crate-owned task.
- Assign one worker only the current scoped task or tightly coupled task group.
- Do not hand this entire plan to one worker unless the user explicitly approves.
- Workers do not commit.
- Before assigning a task, the coordinator checks `git status --short --branch`
  in the owning repo and in root when root pointers or root diffs are involved.
- After worker/reviewer reconciliation and before each logical coordinator
  commit, the coordinator reviews `git status --short --branch`,
  `git diff --stat`, and the relevant detailed diff in the repo being
  committed.
- Before Task 15's root commit, the coordinator reviews both root detailed
  diffs and `git diff --submodule=log`.
- After every worker result, assign a separate reviewer for that scoped diff.
- Coordinators commit at traceable logical points after the scoped worker and reviewer both come back clean.
- After all tasks are complete, run a final holistic clean-context review against:
  - this plan
  - `guidance/surgeist-rust-modeling-guide.md`
  - root `AGENTS.md`
  - dependency direction
  - public API reports
  - generated fixture/API artifacts

The final reviewer must explicitly answer:

- Are all Surgeist-to-Surgeist lowering adapters moved to root or root-owned tools?
- Did leaf crates keep backend-local adapters only?
- Did any leaf crate retain a sibling lowering dependency by convenience?
- Are Rust orphan-rule constraints handled with root-owned wrapper types?
- Are CSS property grammars strict and property-specific?
- Are root adapter errors semantic enough for callers and tests?
- Are API artifacts source-derived and updated only in root?

Completion is only allowed when that holistic review comes back clean.
