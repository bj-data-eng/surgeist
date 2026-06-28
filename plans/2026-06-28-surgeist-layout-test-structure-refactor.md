# Surgeist Layout Test Structure Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move layout-owned unit tests and oracle helpers into idiomatic Rust/Cargo unit-test structure, remove duplicated root layout-oracle coverage, and keep integration tests reserved for public API, fixture, parity, and cross-crate behavior.

**Architecture:** `surgeist-layout` owns the oracle because it is a layout unit-testing helper. Unit tests move under `surgeist-layout/src/*_tests.rs` behind `#[cfg(test)]`, with shared layout test helpers under `surgeist-layout/src/test_support/`. Root `surgeist` keeps only facade/integration smoke coverage and stops path-importing layout's test support.

**Tech Stack:** Rust, Cargo unit tests, Cargo integration tests, `#[cfg(test)]` modules, existing `surgeist-layout` oracle and browser parity fixtures.

---

## Current Problem

Cargo treats every file under `tests/` as a separate integration test crate. The layout repo currently has files named `tests/layout/unit/*.rs`, but those are structurally integration tests even when their intent is unit-level layout algorithm coverage.

The root repo currently carries layout-oracle tests that import layout's test support by path:

- `/Users/codex/Development/surgeist/tests/oracle.rs`
- `/Users/codex/Development/surgeist/tests/layout_oracle.rs`

Those files are also structurally integration tests, but most of their coverage belongs to layout. This creates three problems:

- layout unit coverage is split between layout and root
- oracle algorithms are treated like shared integration support instead of layout-owned test support
- root facade tests duplicate or shadow layout algorithm tests

## Target Structure

After this refactor, layout tests should be organized like this:

```text
/Users/codex/Development/surgeist-layout/
  src/
    lib.rs
    block.rs
    block_tests.rs
    cache.rs
    cache_tests.rs
    contract_tests.rs
    compute.rs
    compute_tests.rs
    flex.rs
    flex_tests.rs
    grid/
      mod.rs
      alignment.rs
      axis.rs
      child.rs
      lanes.rs
      named.rs
      placement.rs
      subgrid.rs
      tracks.rs
    grid_tests.rs       # mounted from `grid/mod.rs` for grid-private access
    inline.rs
    inline_tests.rs
    leaf_tests.rs
    lib_tests.rs
    root_tests.rs
    test_support/
      mod.rs
      layout_tree.rs
      grid_layout_comparison.rs
      oracle/
        mod.rs
        inline.rs
        grid/
          mod.rs
          alignment.rs
          axis.rs
          baseline.rs
          contributions.rs
          lanes.rs
          named.rs
          placement.rs
          scenario.rs
          subgrid.rs
          tracks.rs
  tests/
    layout.rs
    layout/
      browser_parity.rs
      browser_parity/
        README.md
        corpus.toml
        support.rs
        xml/
```

Root should end with no path imports from `crates/surgeist-layout/tests/support`:

```text
/Users/codex/Development/surgeist/
  tests/
    app.rs
    css.rs
    style.rs
    layout.rs          # facade smoke file only when root lacks layout facade coverage
```

The optional root `tests/layout.rs` must not duplicate layout unit behavior. It may only verify that root reexports the intended layout public API through the facade.

## Ownership Rules

- Oracle algorithms live in `surgeist-layout/src/test_support/oracle/**`.
- `OracleTree` lives in `surgeist-layout/src/test_support/layout_tree.rs`.
- Oracle code stays behind `#[cfg(test)]`; it must not become public production API.
- Oracle algorithms must first be moved by direct copy or `git mv` with their
  contents unchanged. Only after the files are in the new location may line
  edits be made, and only when required to fit the moved code into the new
  module path or visibility context. Do not rewrite, summarize, simplify,
  reimplement, partially port, or "clean up" oracle algorithm bodies during
  this refactor.
- Layout unit tests live in suffixed source files such as `block_tests.rs`, `flex_tests.rs`, `grid_tests.rs`, `inline_tests.rs`, and `root_tests.rs`.
- Existing inline `mod tests` blocks in production source files must also move
  into suffixed source test files. After this refactor, production modules
  should not contain large inline `mod tests` blocks.
- Browser parity fixture parsing and XML corpus tests remain under `surgeist-layout/tests/` because they are integration/parity tests.
- Root `surgeist` may test facade reexports, but it must not own layout algorithm correctness coverage.
- Do not add conversion layers, compatibility shims, or root-owned copies of oracle helpers.

## Task 1: Baseline And Classification Inventory

**Files:**
- Inspect: `/Users/codex/Development/surgeist/tests/oracle.rs`
- Inspect: `/Users/codex/Development/surgeist/tests/layout_oracle.rs`
- Inspect: `/Users/codex/Development/surgeist-layout/tests/layout/unit/*.rs`
- Inspect: `/Users/codex/Development/surgeist-layout/tests/support/**`
- Create: `/Users/codex/Development/surgeist-layout/plans/layout-test-classification.md`

- [ ] **Step 1: Record current test counts**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
cargo test -- --list > /tmp/surgeist-layout-tests-before.txt
wc -l /tmp/surgeist-layout-tests-before.txt

cd /Users/codex/Development/surgeist
cargo test -- --list > /tmp/surgeist-root-tests-before.txt
wc -l /tmp/surgeist-root-tests-before.txt
```

Expected: both commands complete successfully and produce before-count files in `/tmp`.

- [ ] **Step 2: Classify layout-owned root tests**

Run:

```bash
cd /Users/codex/Development/surgeist
rg -n "^fn |^#\\[test\\]" tests/oracle.rs tests/layout_oracle.rs > /tmp/surgeist-root-layout-oracle-tests.txt
awk '
  /#\[test\]/ { want = 1; next }
  want && /^[[:space:]]*fn [A-Za-z0-9_]+\(/ {
    line = $0
    sub(/^[[:space:]]*fn /, "", line)
    sub(/\(.*/, "", line)
    print "| `" line "` | `" FILENAME ":" FNR "` | pending |  | |"
    want = 0
  }
' tests/oracle.rs tests/layout_oracle.rs \
  > /tmp/surgeist-root-layout-oracle-ledger.md
```

Create `/Users/codex/Development/surgeist-layout/plans/layout-test-classification.md` with this content. The migration ledger must contain one row for every root test discovered by `/tmp/surgeist-root-layout-oracle-ledger.md`; no row may remain `pending` before root files are deleted.

```markdown
# Layout Test Classification

## Root Tests To Migrate Into Layout Unit Tests

- `/Users/codex/Development/surgeist/tests/oracle.rs`
  - oracle inline algorithm tests -> `/Users/codex/Development/surgeist-layout/src/inline_tests.rs`
  - oracle grid alignment, track sizing, baseline, placement, named-grid, subgrid, and lanes algorithm tests -> `/Users/codex/Development/surgeist-layout/src/grid_tests.rs`
- `/Users/codex/Development/surgeist/tests/layout_oracle.rs`
  - inline/block layout versus oracle tests -> `/Users/codex/Development/surgeist-layout/src/block_tests.rs` and `/Users/codex/Development/surgeist-layout/src/inline_tests.rs`
  - grid, subgrid, lanes, and named-grid production-versus-oracle tests -> `/Users/codex/Development/surgeist-layout/src/grid_tests.rs`

## Per-Test Migration Ledger

| Root test | Source | Disposition | Destination or covering test | Notes |
| --- | --- | --- | --- | --- |
| `example_test_name` | `tests/oracle.rs:1` | migrated | `src/grid_tests.rs::example_test_name` | example row; replace with generated rows |

Allowed dispositions:

- `migrated`: the test body was copied into the named layout source test file.
- `covered`: an equivalent existing layout test already covers the same behavior.
- `obsolete`: the test asserted an old harness behavior that no longer exists after this refactor.

Every `covered` row must name the exact covering test and why it is equivalent.
Every `obsolete` row must name the removed harness assumption and must be
approved by the reviewer before root deletion.

Helper functions are not test ledger rows. Move helper functions together with
the migrated tests that call them, or delete them only after `rg` proves there
are no remaining call sites.

## Tests That Stay In Layout Integration Tests

- `/Users/codex/Development/surgeist-layout/tests/layout/browser_parity.rs`
- `/Users/codex/Development/surgeist-layout/tests/layout/browser_parity/support.rs`
- `/Users/codex/Development/surgeist-layout/tests/layout/browser_parity/xml/**`

## Tests That Stay In Root

- Root facade smoke tests that verify `surgeist::layout` reexports compile.
- No root tests may path-import `/Users/codex/Development/surgeist/crates/surgeist-layout/tests/support`.
```

- [ ] **Step 3: Commit the classification**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
git add plans/layout-test-classification.md
git commit -m "Classify layout test ownership"
```

Expected: one layout commit containing only the classification file.

## Task 2: Move Oracle Support Into Layout `src/test_support`

**Files:**
- Create: `/Users/codex/Development/surgeist-layout/src/test_support/mod.rs`
- Move: `/Users/codex/Development/surgeist-layout/tests/support/oracle/**` -> `/Users/codex/Development/surgeist-layout/src/test_support/oracle/**`
- Move: `/Users/codex/Development/surgeist-layout/tests/support/oracle_tree.rs` -> `/Users/codex/Development/surgeist-layout/src/test_support/layout_tree.rs`
- Move: `/Users/codex/Development/surgeist-layout/tests/support/grid_layout_comparison.rs` -> `/Users/codex/Development/surgeist-layout/src/test_support/grid_layout_comparison.rs`
- Modify: `/Users/codex/Development/surgeist-layout/src/lib.rs`

- [ ] **Step 1: Create test support module wiring**

Add this to `/Users/codex/Development/surgeist-layout/src/lib.rs` near the existing `#[cfg(test)] mod tests;` line:

```rust
#[cfg(test)]
mod test_support;
```

Create `/Users/codex/Development/surgeist-layout/src/test_support/mod.rs`:

```rust
#[allow(dead_code)]
pub(crate) mod grid_layout_comparison;
#[allow(dead_code)]
pub(crate) mod layout_tree;
#[allow(dead_code)]
pub(crate) mod oracle;
```

- [ ] **Step 2: Move support files with git**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
mkdir -p src/test_support/oracle/grid
git mv tests/support/oracle/mod.rs src/test_support/oracle/mod.rs
git mv tests/support/oracle/inline.rs src/test_support/oracle/inline.rs
git mv tests/support/oracle/grid/*.rs src/test_support/oracle/grid/
git mv tests/support/oracle_tree.rs src/test_support/layout_tree.rs
git mv tests/support/grid_layout_comparison.rs src/test_support/grid_layout_comparison.rs
```

- [ ] **Step 2a: Confirm oracle algorithm bodies were not rewritten**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
git diff --find-renames --find-copies --stat -- src/test_support/oracle tests/support/oracle
git diff --find-renames --find-copies -- src/test_support/oracle tests/support/oracle
```

Expected before Step 3: the diff shows file moves only. No moved oracle file
should already contain import rewrites, body edits, shortened code, collapsed
logic, or reimplemented logic. If any content edit appears before Step 3,
restore the copied text exactly before continuing.

- [ ] **Step 3: Update support imports**

Replace only the module prefixes in the moved files:

```bash
cd /Users/codex/Development/surgeist-layout
rg -n "crate::support::oracle|crate::support::oracle_tree" src/test_support
```

Expected current matches in `src/test_support/grid_layout_comparison.rs`.

Change:

```rust
use crate::support::oracle::grid::{
```

to:

```rust
use crate::test_support::oracle::grid::{
```

and change:

```rust
use crate::support::oracle_tree::OracleTree;
```

to:

```rust
use crate::test_support::layout_tree::OracleTree;
```

Also change self-crate imports in moved support files:

```rust
use surgeist_layout::{
```

to:

```rust
use crate::{
```

and change any qualified paths:

```rust
surgeist_layout::
```

to:

```rust
crate::
```

These line edits are allowed only because the files have already been moved.
Keep them to required module-path or visibility fit-up. Do not alter algorithm
logic, control flow, data modeling, numeric formulas, or test expectations.

- [ ] **Step 4: Preserve integration-test access temporarily**

Modify `/Users/codex/Development/surgeist-layout/tests/support/mod.rs` to reexport the new source-owned helpers while migration is in progress:

```rust
#[path = "../../src/test_support/grid_layout_comparison.rs"]
#[allow(dead_code)]
pub mod grid_layout_comparison;

#[path = "../../src/test_support/layout_tree.rs"]
#[allow(dead_code)]
pub mod oracle_tree;

#[path = "../../src/test_support/oracle/mod.rs"]
#[allow(dead_code)]
pub mod oracle;
```

Also modify `/Users/codex/Development/surgeist-layout/tests/layout.rs` to add a root-level alias module for the path-included bridge:

```rust
#[path = "support/mod.rs"]
mod support;

pub use surgeist_layout::*;

mod test_support {
    pub use crate::support::grid_layout_comparison;
    pub use crate::support::oracle;
    pub use crate::support::oracle_tree as layout_tree;
}

#[path = "layout/mod.rs"]
mod layout;
```

This temporary bridge keeps existing integration tests compiling while later tasks move unit tests into `src`.

- [ ] **Step 5: Verify support move**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
cargo test --test layout -- --list
cargo test --lib -- --list
```

Expected: both commands compile and list tests. No behavior changes are expected yet.

- [ ] **Step 6: Commit support move**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
git add src/test_support tests/support src/lib.rs
git commit -m "Move layout oracle support under src test support"
```

## Task 3: Move Existing Layout Unit Tests Into `src/*_tests.rs`

**Files:**
- Move: `/Users/codex/Development/surgeist-layout/tests/layout/unit/block.rs` -> `/Users/codex/Development/surgeist-layout/src/block_tests.rs`
- Move: `/Users/codex/Development/surgeist-layout/tests/layout/unit/cache.rs` -> `/Users/codex/Development/surgeist-layout/src/cache_tests.rs`
- Move: `/Users/codex/Development/surgeist-layout/tests/layout/unit/contract.rs` -> `/Users/codex/Development/surgeist-layout/src/contract_tests.rs`
- Move: `/Users/codex/Development/surgeist-layout/tests/layout/unit/flex.rs` -> `/Users/codex/Development/surgeist-layout/src/flex_tests.rs`
- Move: `/Users/codex/Development/surgeist-layout/tests/layout/unit/grid.rs` -> `/Users/codex/Development/surgeist-layout/src/grid_tests.rs`
- Move: `/Users/codex/Development/surgeist-layout/tests/layout/unit/leaf.rs` -> `/Users/codex/Development/surgeist-layout/src/leaf_tests.rs`
- Move: `/Users/codex/Development/surgeist-layout/tests/layout/unit/root.rs` -> `/Users/codex/Development/surgeist-layout/src/root_tests.rs`
- Move: `/Users/codex/Development/surgeist-layout/src/tests.rs` -> `/Users/codex/Development/surgeist-layout/src/lib_tests.rs`
- Extract inline `mod tests` blocks from `/Users/codex/Development/surgeist-layout/src/block.rs`, `/Users/codex/Development/surgeist-layout/src/compute.rs`, `/Users/codex/Development/surgeist-layout/src/flex.rs`, `/Users/codex/Development/surgeist-layout/src/inline.rs`, `/Users/codex/Development/surgeist-layout/src/grid/lanes.rs`, and `/Users/codex/Development/surgeist-layout/src/grid/tracks.rs`
- Move: `/Users/codex/Development/surgeist-layout/src/grid/tests.rs` -> merge into `/Users/codex/Development/surgeist-layout/src/grid_tests.rs`, mounted as a child of `grid`
- Modify: `/Users/codex/Development/surgeist-layout/src/lib.rs`
- Modify: `/Users/codex/Development/surgeist-layout/src/grid/mod.rs`
- Modify: `/Users/codex/Development/surgeist-layout/tests/layout/mod.rs`

- [ ] **Step 1: Add source test module declarations**

Add these declarations to `/Users/codex/Development/surgeist-layout/src/lib.rs`:

```rust
#[cfg(test)]
mod block_tests;
#[cfg(test)]
mod cache_tests;
#[cfg(test)]
mod contract_tests;
#[cfg(test)]
mod flex_tests;
#[cfg(test)]
mod lib_tests;
#[cfg(test)]
mod leaf_tests;
#[cfg(test)]
mod root_tests;
```

- [ ] **Step 2: Move the files**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
git mv tests/layout/unit/block.rs src/block_tests.rs
git mv tests/layout/unit/cache.rs src/cache_tests.rs
git mv tests/layout/unit/contract.rs src/contract_tests.rs
git mv tests/layout/unit/flex.rs src/flex_tests.rs
git mv tests/layout/unit/grid.rs src/grid_tests.rs
git mv tests/layout/unit/leaf.rs src/leaf_tests.rs
git mv tests/layout/unit/root.rs src/root_tests.rs
git mv src/tests.rs src/lib_tests.rs
```

Do not add `mod grid_tests;` to `/Users/codex/Development/surgeist-layout/src/lib.rs`. Add this to `/Users/codex/Development/surgeist-layout/src/grid/mod.rs`, replacing the existing `#[cfg(test)] mod tests;` declaration:

```rust
#[cfg(test)]
#[path = "../grid_tests.rs"]
mod tests;
```

This keeps the suffixed `src/grid_tests.rs` file mounted inside the `grid` module, so tests moved from `src/grid/tests.rs` can still access grid-private internals without making production modules more public.

- [ ] **Step 3: Replace integration-test imports with crate-local imports**

In each moved file, replace integration harness imports:

```rust
use super::*;
```

with explicit crate imports needed by that file. Start with this broad import to preserve behavior during the move:

```rust
use crate::*;
```

Then replace support imports:

```rust
use super::support::oracle_tree::{OracleMeasurement, OracleTree};
```

with:

```rust
use crate::test_support::layout_tree::{OracleMeasurement, OracleTree};
```

Replace any remaining `support::` paths with `crate::test_support::`.

Replace self-crate imports and qualified paths in moved source test files:

```rust
use surgeist_layout::{
```

becomes:

```rust
use crate::{
```

and:

```rust
surgeist_layout::
```

becomes:

```rust
crate::
```

Run:

```bash
cd /Users/codex/Development/surgeist-layout
rg -n "super::support|crate::support|support::oracle|support::oracle_tree|surgeist_layout::|use surgeist_layout" src/*_tests.rs src/test_support
rg -n "use super::\\*" src/block_tests.rs src/cache_tests.rs src/contract_tests.rs src/compute_tests.rs src/flex_tests.rs src/leaf_tests.rs src/lib_tests.rs src/root_tests.rs src/inline_tests.rs 2>/dev/null || true
```

Expected:

- no `support::` or `surgeist_layout` self-crate imports remain in moved source tests or test support
- no crate-root suffixed test file uses `use super::*`
- `src/grid_tests.rs` may keep `use super::*` or replace it with explicit `super::{...}` imports because it is mounted from `src/grid/mod.rs` and intentionally tests grid-private internals

- [ ] **Step 4: Extract existing inline source tests into suffixed files**

Move existing source-level test modules into suffixed test files without changing test bodies except for required imports:

| Current location | Target |
| --- | --- |
| `/Users/codex/Development/surgeist-layout/src/block.rs` inline `mod tests` | append to `/Users/codex/Development/surgeist-layout/src/block_tests.rs` |
| `/Users/codex/Development/surgeist-layout/src/compute.rs` inline `mod tests` | `/Users/codex/Development/surgeist-layout/src/compute_tests.rs` |
| `/Users/codex/Development/surgeist-layout/src/flex.rs` inline `mod tests` | append to `/Users/codex/Development/surgeist-layout/src/flex_tests.rs` |
| `/Users/codex/Development/surgeist-layout/src/inline.rs` inline `mod tests` | `/Users/codex/Development/surgeist-layout/src/inline_tests.rs` |
| `/Users/codex/Development/surgeist-layout/src/grid/lanes.rs` inline `mod tests` | append to `/Users/codex/Development/surgeist-layout/src/grid_tests.rs` |
| `/Users/codex/Development/surgeist-layout/src/grid/tracks.rs` inline `mod tests` | append to `/Users/codex/Development/surgeist-layout/src/grid_tests.rs` |
| `/Users/codex/Development/surgeist-layout/src/grid/tests.rs` | append to `/Users/codex/Development/surgeist-layout/src/grid_tests.rs`, then delete the source file; keep the file mounted from `src/grid/mod.rs` |

Add this declaration to `/Users/codex/Development/surgeist-layout/src/lib.rs` when creating `compute_tests.rs`:

```rust
#[cfg(test)]
mod compute_tests;
```

Remove only the `#[cfg(test)] mod tests { ... }` wrappers from production files after their contents are copied into the target suffixed files. Preserve test function bodies exactly unless an import path must change because the test moved to crate root.

Run:

```bash
cd /Users/codex/Development/surgeist-layout
rg -n "mod tests \\{" src/block.rs src/compute.rs src/flex.rs src/inline.rs src/grid/lanes.rs src/grid/tracks.rs
```

Expected: no large inline `mod tests { ... }` blocks remain in production source files. Small `#[cfg(test)]` test-only helper functions or constructors may remain only when they are used by the new suffixed test files.

- [ ] **Step 5: Remove moved unit modules from integration harness**

Remove these lines from `/Users/codex/Development/surgeist-layout/tests/layout/mod.rs`:

```rust
#[path = "unit/block.rs"]
mod block;
#[path = "unit/cache.rs"]
mod cache;
#[path = "unit/contract.rs"]
mod contract;
#[path = "unit/flex.rs"]
mod flex;
#[path = "unit/grid.rs"]
mod grid;
#[path = "unit/leaf.rs"]
mod leaf;
#[path = "unit/root.rs"]
mod root;
```

Then trim `/Users/codex/Development/surgeist-layout/tests/layout/mod.rs` so it only wires the parity module:

```rust
mod browser_parity;
```

Remove the old `use crate::support;`, `use std::collections::HashMap;`, `use surgeist_layout::{...};`, and `use support::oracle::grid::{...};` imports from that file after the integration unit modules are gone.

- [ ] **Step 6: Verify moved tests**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
cargo test --lib -- --list > /tmp/surgeist-layout-lib-tests-after-unit-move.txt
cargo test --test layout -- --list > /tmp/surgeist-layout-integration-tests-after-unit-move.txt
rg -n "tests/layout/unit" .
rg -n "mod tests \\{" src/block.rs src/compute.rs src/flex.rs src/inline.rs src/grid/lanes.rs src/grid/tracks.rs
cargo test --lib
cargo test --test layout
```

Expected:

- `cargo test --lib` includes the moved block/cache/contract/flex/grid/leaf/root tests.
- `cargo test --test layout` still includes browser parity tests.
- `rg -n "tests/layout/unit" .` returns no matches outside historic plan text.
- production source files no longer contain inline `mod tests` blocks.

- [ ] **Step 7: Commit unit test move**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
git add src tests
git commit -m "Move layout unit tests into source modules"
```

## Task 4: Migrate Root Oracle Algorithm Tests Into Layout Source Tests

**Files:**
- Read: `/Users/codex/Development/surgeist/tests/oracle.rs`
- Modify: `/Users/codex/Development/surgeist-layout/src/inline_tests.rs`
- Modify: `/Users/codex/Development/surgeist-layout/src/grid_tests.rs`
- Modify: `/Users/codex/Development/surgeist-layout/src/lib.rs`

- [ ] **Step 1: Create `inline_tests.rs` if absent**

If `/Users/codex/Development/surgeist-layout/src/inline_tests.rs` does not exist, create it and add this declaration to `/Users/codex/Development/surgeist-layout/src/lib.rs`:

```rust
#[cfg(test)]
mod inline_tests;
```

Start `/Users/codex/Development/surgeist-layout/src/inline_tests.rs` with:

```rust
use crate::*;
use crate::test_support::oracle::inline;
```

- [ ] **Step 2: Pre-check root oracle duplicates**

Before copying tests from root `tests/oracle.rs`, generate its test-name list and look for existing destination coverage:

```bash
cd /Users/codex/Development/surgeist
awk '
  /#\[test\]/ { want = 1; next }
  want && /^[[:space:]]*fn [A-Za-z0-9_]+\(/ {
    line = $0
    sub(/^[[:space:]]*fn /, "", line)
    sub(/\(.*/, "", line)
    print line
    want = 0
  }
' tests/oracle.rs | sort > /tmp/surgeist-root-oracle-test-names.txt

cd /Users/codex/Development/surgeist-layout
while IFS= read -r TEST_NAME; do
  printf '\n== %s ==\n' "$TEST_NAME"
  rg -n "fn ${TEST_NAME}\\b|${TEST_NAME}" src/*_tests.rs || true
done < /tmp/surgeist-root-oracle-test-names.txt
```

If an equivalent test already exists, do not add a duplicate. Update that test's row in the `Per-Test Migration Ledger` table to `covered`, naming the exact covering test and why the behavior is equivalent.

- [ ] **Step 3: Move oracle inline tests**

Move tests from `/Users/codex/Development/surgeist/tests/oracle.rs` whose names begin with `oracle_atomic_inline_` into `/Users/codex/Development/surgeist-layout/src/inline_tests.rs`.

Use these import rewrites:

```rust
support::oracle::inline
```

becomes:

```rust
inline
```

and:

```rust
surgeist::layout::
```

becomes:

```rust
crate::
```

- [ ] **Step 4: Move oracle grid algorithm tests**

Move the remaining oracle grid algorithm tests from `/Users/codex/Development/surgeist/tests/oracle.rs` into `/Users/codex/Development/surgeist-layout/src/grid_tests.rs`, grouped near existing tests for the same domain:

- baseline helpers near baseline/grid baseline tests
- named grid helpers near named grid tests
- lane placement and intrinsic sizing near grid lane tests
- subgrid helpers near subgrid tests
- track sizing and alignment near track/alignment tests

Use these import rewrites:

```rust
support::oracle::grid
```

becomes:

```rust
crate::test_support::oracle::grid
```

and:

```rust
support::oracle_tree
```

becomes:

```rust
crate::test_support::layout_tree
```

- [ ] **Step 5: Check root oracle file is fully accounted for**

Run:

```bash
cd /Users/codex/Development/surgeist
awk '
  /#\[test\]/ { want = 1; next }
  want && /^[[:space:]]*fn [A-Za-z0-9_]+\(/ {
    line = $0
    sub(/^[[:space:]]*fn /, "", line)
    sub(/\(.*/, "", line)
    print line
    want = 0
  }
' tests/oracle.rs | sort > /tmp/surgeist-root-oracle-test-names.txt

cd /Users/codex/Development/surgeist-layout
cargo test --lib -- --list \
  | sed -E 's#^([^:]+): test$#\1#' \
  | sed -E 's#.*::##' \
  | sort > /tmp/surgeist-layout-lib-test-names.txt

cd /Users/codex/Development/surgeist
comm -23 /tmp/surgeist-root-oracle-test-names.txt /tmp/surgeist-layout-lib-test-names.txt \
  > /tmp/surgeist-root-oracle-not-migrated-by-name.txt
```

Expected:

- `/tmp/surgeist-root-oracle-not-migrated-by-name.txt` is empty, or every listed test has a `covered` or reviewer-approved `obsolete` row in `/Users/codex/Development/surgeist-layout/plans/layout-test-classification.md`.
- No test from root `tests/oracle.rs` may be deleted from root until its ledger row is `migrated`, `covered`, or reviewer-approved `obsolete`.

- [ ] **Step 6: Verify and commit**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
cargo test --lib
git add src plans/layout-test-classification.md
git commit -m "Move root oracle algorithm tests into layout"
```

## Task 5: Migrate Root Layout-Versus-Oracle Tests Into Layout Source Tests

**Files:**
- Read: `/Users/codex/Development/surgeist/tests/layout_oracle.rs`
- Modify: `/Users/codex/Development/surgeist-layout/src/block_tests.rs`
- Modify: `/Users/codex/Development/surgeist-layout/src/grid_tests.rs`
- Modify: `/Users/codex/Development/surgeist-layout/src/inline_tests.rs`

- [ ] **Step 1: Pre-check layout-oracle duplicates**

Before copying tests from root `tests/layout_oracle.rs`, generate its test-name list and check existing destination coverage:

```bash
cd /Users/codex/Development/surgeist
awk '
  /#\[test\]/ { want = 1; next }
  want && /^[[:space:]]*fn [A-Za-z0-9_]+\(/ {
    line = $0
    sub(/^[[:space:]]*fn /, "", line)
    sub(/\(.*/, "", line)
    print line
    want = 0
  }
' tests/layout_oracle.rs | sort > /tmp/surgeist-root-layout-oracle-test-names.txt

cd /Users/codex/Development/surgeist-layout
while IFS= read -r TEST_NAME; do
  printf '\n== %s ==\n' "$TEST_NAME"
  rg -n "fn ${TEST_NAME}\\b|${TEST_NAME}" src/*_tests.rs || true
done < /tmp/surgeist-root-layout-oracle-test-names.txt
```

If an equivalent test already exists, do not add a duplicate. Update that test's row in the `Per-Test Migration Ledger` table to `covered`, naming the exact covering test and why the behavior is equivalent. Name matches alone are not enough for coverage removal.

- [ ] **Step 2: Move inline and block layout-oracle tests**

Move these categories from root `tests/layout_oracle.rs` into layout source tests:

- `oracle_layout_inline_*` -> `/Users/codex/Development/surgeist-layout/src/inline_tests.rs`
- block or atomic inline layout helpers -> `/Users/codex/Development/surgeist-layout/src/block_tests.rs`

Rewrite imports:

```rust
surgeist::layout::
```

to:

```rust
crate::
```

and rewrite support paths to `crate::test_support::*`.

- [ ] **Step 3: Move grid, subgrid, named-grid, and lanes layout-oracle tests**

Move these categories from root `tests/layout_oracle.rs` into `/Users/codex/Development/surgeist-layout/src/grid_tests.rs`:

- `named_grid_layout_oracle_*`
- `subgrid_*`
- `oracle_layout_*tracks*`
- `oracle_layout_*placement*`
- `lanes_*`
- helpers such as `assert_production_lane_placement_matches_oracle`, `assert_production_lane_intrinsic_matches_oracle`, `oracle_lane_facts`, and `production_lane_facts`

Keep helpers private to `grid_tests.rs` unless they are needed by another source test file.

- [ ] **Step 4: Verify migrated behavior**

Run:

```bash
cd /Users/codex/Development/surgeist
awk '
  /#\[test\]/ { want = 1; next }
  want && /^[[:space:]]*fn [A-Za-z0-9_]+\(/ {
    line = $0
    sub(/^[[:space:]]*fn /, "", line)
    sub(/\(.*/, "", line)
    print line
    want = 0
  }
' tests/layout_oracle.rs | sort > /tmp/surgeist-root-layout-oracle-test-names.txt

cd /Users/codex/Development/surgeist-layout
cargo test --lib
cargo test --lib -- --list > /tmp/surgeist-layout-lib-tests-after-root-migration.txt
cargo test --lib -- --list \
  | sed -E 's#^([^:]+): test$#\1#' \
  | sed -E 's#.*::##' \
  | sort > /tmp/surgeist-layout-lib-test-names.txt
comm -23 /tmp/surgeist-root-layout-oracle-test-names.txt /tmp/surgeist-layout-lib-test-names.txt \
  > /tmp/surgeist-root-layout-oracle-not-migrated-by-name.txt
```

Expected:

- migrated root layout-oracle tests are listed under lib/unit tests and pass
- `/tmp/surgeist-root-layout-oracle-not-migrated-by-name.txt` is empty, or every listed test has a `covered` or reviewer-approved `obsolete` row in `/Users/codex/Development/surgeist-layout/plans/layout-test-classification.md`

- [ ] **Step 5: Commit root layout-oracle migration**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
git add src plans/layout-test-classification.md
git commit -m "Move root layout oracle tests into layout"
```

## Task 6: Remove Root Layout-Unit Duplication

**Files:**
- Delete: `/Users/codex/Development/surgeist/tests/oracle.rs`
- Delete: `/Users/codex/Development/surgeist/tests/layout_oracle.rs`
- Create when needed: `/Users/codex/Development/surgeist/tests/layout.rs`

- [ ] **Step 1: Confirm migration ledger is fully resolved**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
rg -n "\\| pending \\|" plans/layout-test-classification.md || true
rg -n "\\| obsolete \\|" plans/layout-test-classification.md || true
```

Expected:

- the `pending` search returns no matches
- every `obsolete` row, if any, has a reviewer note in the same row explaining the removed harness assumption
- every `covered` row names the exact covering test and why it is equivalent

Do not delete root files while any ledger row is still `pending`.

- [ ] **Step 2: Confirm root no longer needs path-imported layout support**

Run:

```bash
cd /Users/codex/Development/surgeist
rg -n "crates/surgeist-layout/tests/support|support::oracle|oracle_tree|layout_oracle|mod support" tests
```

Expected before deletion: matches only in `tests/oracle.rs` and `tests/layout_oracle.rs`.

- [ ] **Step 3: Delete root layout-unit files**

Run:

```bash
cd /Users/codex/Development/surgeist
git rm tests/oracle.rs tests/layout_oracle.rs
```

- [ ] **Step 4: Add a minimal root facade smoke test only if needed**

If deleting both files leaves root without layout facade coverage, create `/Users/codex/Development/surgeist/tests/layout.rs` with only public facade type checks:

```rust
use surgeist::layout::{
    Available, Dimension, Display, Edges, GridAutoFlow, GridPlacement, Length, NodeInput,
    Position, Size, TrackComponent,
};

#[test]
fn root_facade_reexports_layout_front_door_types() {
    let input = NodeInput {
        display: Display::Block,
        size: Size::new(Dimension::px(10.0), Dimension::AUTO),
        margin: Edges::all(Length::px(2.0)),
        position: Position::Relative,
        grid_auto_flow: GridAutoFlow::Row,
        grid_column: GridPlacement::AUTO,
        grid_row: GridPlacement::AUTO,
        ..NodeInput::DEFAULT
    };

    let available = Size::splat(Available::definite(100.0));
    let track = TrackComponent::px(10.0);

    assert_eq!(input.display, Display::Block);
    assert_eq!(available.width.into_option(), Some(100.0));
    assert_eq!(track, TrackComponent::px(10.0));
}
```

This test must stay a facade smoke test. Do not add oracle helpers, layout tree fixtures, or production-versus-oracle assertions to root.

- [ ] **Step 5: Verify root no longer path-imports layout internals**

Run:

```bash
cd /Users/codex/Development/surgeist
rg -n "crates/surgeist-layout/tests/support|support::oracle|oracle_tree|layout_oracle|mod support" tests
cargo test --tests
```

Expected:

- no matches for the `rg` command
- root integration tests pass

- [ ] **Step 6: Commit root cleanup**

Run:

```bash
cd /Users/codex/Development/surgeist
git add tests
git commit -m "Remove duplicated layout oracle tests from root"
```

## Task 7: Remove Temporary Layout Integration Support Bridge

**Files:**
- Modify: `/Users/codex/Development/surgeist-layout/tests/support/mod.rs`
- Delete: `/Users/codex/Development/surgeist-layout/tests/support/` if no longer used
- Modify: `/Users/codex/Development/surgeist-layout/tests/layout/browser_parity/support.rs`

- [ ] **Step 1: Check remaining integration support usage**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
rg -n "crate::support::oracle|crate::support::oracle_tree|crate::support::grid_layout_comparison|mod support|tests/support" tests src
```

Expected: only browser parity support may still reference `crate::support::grid_layout_comparison::ComparisonTolerance`.

- [ ] **Step 2: Decouple browser parity from oracle support**

If `/Users/codex/Development/surgeist-layout/tests/layout/browser_parity/support.rs` still imports:

```rust
use crate::support::grid_layout_comparison::ComparisonTolerance;
```

replace it with a local browser parity tolerance type in that same file:

```rust
#[derive(Clone, Copy, Debug)]
struct ComparisonTolerance {
    value: Scalar,
}

impl ComparisonTolerance {
    const fn browser_parity() -> Self {
        Self { value: 0.1 }
    }

    fn contains(self, delta: Scalar) -> bool {
        delta.abs() <= self.value
    }
}
```

Use the existing `type Scalar = layout::Scalar;` alias already present in `browser_parity/support.rs`.

This preserves the existing call sites such as:

```rust
ComparisonTolerance::browser_parity().contains(actual - expected)
```

Do not introduce an `allows(actual, expected)` API unless all call sites and tests are deliberately updated in the same task.

- [ ] **Step 3: Remove integration support bridge**

If no integration tests use `/Users/codex/Development/surgeist-layout/tests/support/mod.rs`, delete the bridge:

```bash
cd /Users/codex/Development/surgeist-layout
git rm tests/support/mod.rs
rmdir tests/support 2>/dev/null || true
```

Also remove the temporary modules from `/Users/codex/Development/surgeist-layout/tests/layout.rs`, leaving only:

```rust
#[path = "layout/mod.rs"]
mod layout;
```

- [ ] **Step 4: Verify integration tests remain parity-only**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
find tests -maxdepth 3 -type f | sort
cargo test --test layout
```

Expected: `tests/layout.rs` now wires browser parity only, and `cargo test --test layout` passes.

- [ ] **Step 5: Commit integration support cleanup**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
git add tests src
git commit -m "Keep layout integration tests parity-focused"
```

## Task 8: Final Verification And Pointer-Readiness

**Files:**
- Inspect: `/Users/codex/Development/surgeist-layout`
- Inspect: `/Users/codex/Development/surgeist`

- [ ] **Step 1: Run layout checks**

Run:

```bash
cd /Users/codex/Development/surgeist-layout
cargo fmt --check
cargo test --lib
cargo test --test layout
cargo clippy --all-targets -- -D warnings
git status --short --branch
```

Expected: all commands pass; layout status is clean after committing.

- [ ] **Step 2: Run root checks without pointer update**

Run:

```bash
cd /Users/codex/Development/surgeist
cargo fmt --check
cargo test --tests
cargo check --workspace
git status --short --branch
```

Expected: root tests pass after duplicate layout-oracle tests are removed. Root may show only the submodule pointer changed after layout is committed and fetched.

- [ ] **Step 3: Compare before/after test lists**

Run:

```bash
comm -23 \
  <(sort /tmp/surgeist-root-tests-before.txt) \
  <(cd /Users/codex/Development/surgeist && cargo test -- --list | sort) \
  > /tmp/surgeist-root-tests-removed.txt

comm -13 \
  <(sort /tmp/surgeist-layout-tests-before.txt) \
  <(cd /Users/codex/Development/surgeist-layout && cargo test -- --list | sort) \
  > /tmp/surgeist-layout-tests-added.txt
```

Expected: removed root layout-oracle tests are accounted for by added layout lib tests or documented deduplications in `layout-test-classification.md`.

- [ ] **Step 4: Final reviewer checklist**

Ask a clean reviewer to inspect:

- No layout-owned unit tests remain under `surgeist-layout/tests/layout/unit`.
- No root test path-imports `crates/surgeist-layout/tests/support`.
- Oracle algorithms are single-owned under `surgeist-layout/src/test_support/oracle`.
- Browser parity tests remain under `surgeist-layout/tests`.
- Root contains only facade/integration tests, not layout algorithm correctness tests.
- Test count deltas are explained by migration or deduplication.

- [ ] **Step 5: Push layout and then update root pointer only after green**

Run only after the reviewer is clean:

```bash
cd /Users/codex/Development/surgeist-layout
git push

cd /Users/codex/Development/surgeist
git diff --submodule=log
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

Expected: root pointer update is eligible only if the root pointer gates pass or the coordinator explicitly records a red-pointer exception.

## Completion Criteria

- `surgeist-layout` owns all oracle code and oracle algorithm tests.
- Layout unit tests live under `src/*_tests.rs`, not under `tests/layout/unit`.
- `surgeist-layout/tests` contains integration/parity tests, not unit tests named as integration tests.
- Root `surgeist` no longer duplicates layout algorithm tests.
- Root has no path imports into layout test support.
- Layout and root checks pass.
- A clean reviewer confirms the structure matches idiomatic Rust/Cargo test boundaries.
