# Surgeist Test And Tool Inventory

> **For agentic workers:** This is an inventory and planning input, not an
> implementation plan. Use it to scope later crate-isolation migration plans.
> Do not move files from this inventory without a crate-local migration plan and
> reviewer cycle.

**Goal:** Inventory test, fixture, generator, oracle, and tooling ownership
across the Surgeist crate repos so later isolation work can move shared
verification out of production crates deliberately.

**Architecture:** Production crates should own focused unit and crate-local
integration tests for their behavior. `surgeist-test` should become the home for
shared fixtures, corpus metadata, cross-crate parity harnesses, and large
generated artifacts once migration plans exist. The root `surgeist` repo should
own facade and whole-system coordination checks, not large algorithm oracle
logic or corpus churn.

**Tech Stack:** Rust 2024, Cargo workspaces, root submodules, crate-local
`tests/`, `examples/`, API generator artifacts, layout browser parity XML/HTML,
trybuild compile tests, proptest, and optional generator features.

---

## Inventory Method

Commands used from `/Users/codex/Development/surgeist` on 2026-06-28:

```sh
for d in /Users/codex/Development/surgeist /Users/codex/Development/surgeist-{css,dialog,layout,render,retained,shape,style,task,test,text,window}; do
  (cd "$d" && git status --short --branch)
done

rg --files -g '!crates/**' -g '!tmp/**'
rg -n '#\[test\]|#\[tokio::test\]|proptest!' src tests examples dev

find /Users/codex/Development/surgeist-layout/tests -path '*/target' -prune -o -type f
find /Users/codex/Development/surgeist-layout/tests/layout/browser_parity/xml -type f -name '*.xml'
find /Users/codex/Development/surgeist-layout/tests/layout/browser_parity/html -type f -name '*.html'
```

All repos were clean on `main` at inventory time. Root file counts below exclude
submodule contents; crate counts use the sibling crate repos directly.

## Summary Table

| Repo | Rust files | Test attrs | Examples | API tooling | Plans | XML fixtures | HTML fixtures | Notes |
| --- | ---: | ---: | ---: | --- | ---: | ---: | ---: | --- |
| `surgeist` | 36 | 455 | 3 | yes | 18 | 0 | 0 | facade tests plus large root oracle/style/css integration files |
| `surgeist-css` | 2 | 13 | 0 | yes | 3 | 0 | 0 | parser unit tests in `src/lib.rs` |
| `surgeist-dialog` | 9 | 5 | 0 | yes | 1 | 0 | 0 | crate-local `src/tests.rs` |
| `surgeist-layout` | 53 | 708 | 0 | yes | 27 | 4,993 | 1,335 | largest corpus/tooling owner |
| `surgeist-render` | 17 | 34 | 0 | yes | 1 | 0 | 0 | crate-local `src/tests.rs` |
| `surgeist-retained` | 15 | 29 | 0 | yes | 2 | 0 | 0 | crate-local `src/tests.rs` |
| `surgeist-shape` | 11 | 17 | 0 | yes | 1 | 0 | 0 | crate-local `src/tests.rs` |
| `surgeist-style` | 34 | 31 | 0 | yes | 8 | 0 | 0 | crate-local tests plus trybuild compile tests |
| `surgeist-task` | 1 | 1 | 0 | partial | 1 | 0 | 0 | new skeleton crate |
| `surgeist-test` | 2 | 0 | 0 | yes | 2 | 0 | 0 | intended shared harness crate; mostly empty |
| `surgeist-text` | 14 | 63 | 0 | yes | 1 | 0 | 0 | crate-local tests; render-facing tests exist |
| `surgeist-window` | 22 | 78 | 0 | yes | 4 | 0 | 0 | crate-local `src/tests.rs` |

`Test attrs` counts `#[test]`, `#[tokio::test]`, and `proptest!` matches. It
is a rough density signal, not a canonical test count.

## Root `surgeist`

Root-owned test and tool surface:

- `tests/app.rs`: public facade tests for `surgeist::app` and the headless
  thumbnail example contract.
- `tests/css.rs`: root-level CSS facade/integration tests over
  `surgeist::css`/style behavior.
- `tests/style.rs`: root-level style facade/integration and proptest coverage.
- `tests/oracle.rs`: large copied/imported layout oracle test suite through
  the root facade.
- `tests/layout_oracle.rs`: root layout oracle integration checks through
  `surgeist::layout`.
- `examples/hello-window.rs`: window facade example.
- `examples/render-window.rs`: render/window facade example.
- `examples/app-thumbnail-import.rs`: headless app runtime example.
- `api/generator/`: source-derived public API artifact generator.
- `api/public-api.txt`: root API artifact.

Ownership classification:

- Keep in root: `tests/app.rs` facade coverage and small public examples that
  prove root reexports.
- Candidate for `surgeist-test`: `tests/oracle.rs` and `tests/layout_oracle.rs`
  because they are layout oracle/integration verification rather than root
  facade behavior.
- Candidate for crate-local/root split: `tests/css.rs` and `tests/style.rs`.
  Keep one or two facade smoke checks in root, but move detailed parsing/style
  domain coverage to `surgeist-css`, `surgeist-style`, or `surgeist-test`.

Risk:

- Root currently pulls a lot of algorithm context into the integration facade.
  That is useful while splitting the monorepo, but it is likely to keep root
  agents overloaded if layout/style/css continue to evolve independently.

## `surgeist-layout`

Layout-owned test and tool surface:

- `src/grid/tests.rs`, `src/flex.rs`, `src/grid/lanes.rs`, `src/tests.rs`:
  production algorithm unit tests.
- `tests/layout/unit/*.rs`: integration-style unit suites for block, grid,
  flex, leaf, root, cache, and public contracts.
- `tests/support/*.rs`: test support for layout comparison and oracle trees.
- `tests/support/oracle/**/*.rs`: independent oracle models for grid, lanes,
  subgrid, inline, placement, tracks, alignment, baseline, and named lines.
- `tests/layout/browser_parity.rs`: browser parity smoke/full runner.
- `tests/layout/browser_parity/support.rs`: parity XML/support parser and
  comparison harness.
- `tests/layout/browser_parity/corpus.toml`: source provenance, imports,
  expected failures, and quarantine accounting.
- `tests/layout/browser_parity/html/`: constrained human-readable HTML sources.
- `tests/layout/browser_parity/xml/`: generated browser expectation artifacts.
- `tests/layout/browser_parity/scripts/gentest/`: exactly
  `test_helper.js` and `test_base_style.css`.
- `tests/bin/surgeist-layout-generate.rs` and
  `tests/bin/surgeist-layout-generate/generator.rs`: Rust generator binary.
- Feature `layout-golden-generate`: enables `chromiumoxide`, `futures`,
  `serde`, `serde_json`, `sha2`, `tokio`, `toml`, and `url`.

Fixture counts:

- XML fixtures: 4,993.
- HTML fixtures: 1,335.
- XML by top-level bucket:
  - `block`: 824
  - `blockflex`: 28
  - `blockgrid`: 56
  - `flex`: 2,280
  - `float`: 4
  - `grid`: 1,148
  - `grid-lanes`: 56
  - `gridflex`: 24
  - `leaf`: 56
  - `subgrid`: 508

Documented commands:

```sh
cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
SURGEIST_PARITY_FILTER=subgrid cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- import-taffy
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-taffy-corpus
```

Ownership classification:

- Keep in `surgeist-layout`: focused production algorithm tests in `src/` and
  small crate-local contract tests in `tests/layout/unit/`.
- Candidate for `surgeist-test`: browser parity fixture storage, XML/HTML
  corpus root, corpus manifest, parity XML parser, fixture discovery, and
  generator reports.
- Candidate for split ownership: oracle support code. The pure oracle models
  can move to `surgeist-test` if they are used for cross-crate verification;
  layout should keep only small local oracle/unit helpers needed for algorithm
  development.
- Candidate for generator crate/module in `surgeist-test`: the
  `surgeist-layout-generate` binary, or a renamed equivalent, because its
  dependencies and generated-output churn are test infrastructure rather than
  production layout behavior.

Risk:

- `surgeist-layout` owns too much test infrastructure and generated artifact
  churn. The fixture corpus and generator are especially likely to swamp agent
  context and git diffs.
- Moving everything at once is risky. Split migration into: fixture reader
  library, checked-in minimal fixture seed, generator relocation, full corpus
  relocation, then root/layout runner rewiring.

## `surgeist-test`

Current test/tool surface:

- `src/lib.rs`: skeleton shared test crate.
- `api/generator/` and `api/public-api.txt`: standard API artifact tooling.
- `plans/2026-06-24-layout-browser-parity-migration.md`: existing plan draft
  for moving layout browser parity infrastructure.

Intended ownership:

- Shared fixture metadata.
- Cross-crate fixture readers and parsers.
- Integration, e2e, system, and quality/coverage verification support.
- Layout/browser parity corpus once migrated.
- Reusable oracle harnesses that should not live in production crates.

Current gap:

- `surgeist-test` is not yet carrying the large test apparatus it was created
  to own. It is the natural target but needs staged APIs before large corpus
  movement.

## `surgeist-style`

Style-owned test/tool surface:

- Unit tests in `src/calc.rs`, `src/resolver.rs`, `src/value.rs`,
  `src/declaration.rs`, and `src/adapters/layout.rs`.
- `tests/type_safety.rs`: trybuild harness.
- `tests/compile_fail/*.rs` and `*.stderr`: negative compile tests for public
  construction and invariant privacy.
- `tests/compile_pass/typed_public_construction.rs`: positive public
  construction compile test.
- API generator and artifact.

Ownership classification:

- Keep in `surgeist-style`: typed value, declaration, resolver, adapter, and
  trybuild tests. These protect public modeling boundaries and should stay
  near the crate.
- Candidate for `surgeist-test`: only broad cross-crate fixture scenarios that
  combine CSS parsing, style resolution, and layout outcomes.

Risk:

- Layout adapter tests are legitimate style responsibility while the adapter
  lives in style. If adapters move upward into root later, their tests should
  move with the adapter boundary.

## `surgeist-css`

CSS-owned test/tool surface:

- Parser tests live in `src/lib.rs`.
- Depends on `surgeist-style`.
- API generator and artifact.

Ownership classification:

- Keep in `surgeist-css`: CSS syntax, strict error classification, declaration
  lowering into style values.
- Candidate for `surgeist-test`: end-to-end CSS-to-layout fixture tests once a
  full app/testing surface exists.

Risk:

- Detailed CSS/style integration should not expand in root. Root should keep
  facade smoke tests only.

## `surgeist-window`

Window-owned test/tool surface:

- Large `src/tests.rs` suite covering fake host, descriptors, state patches,
  event delivery, host command planning, proxy/front-door commands, and optional
  accessibility behavior.
- Feature-specific accessibility checks are documented in plans.
- API generator and artifact.

Ownership classification:

- Keep in `surgeist-window`: host/window contract tests, fake-host behavior,
  event lifecycle, command planning, and accessibility feature checks.
- Candidate for `surgeist-test`: future cross-crate app/window e2e tests once
  Surgeist has a real app surface and test harness.

Risk:

- The fake host is useful crate-local infrastructure. Do not move it just
  because it is a test helper unless another crate needs it as shared API.

## `surgeist-text`

Text-owned test/tool surface:

- `src/tests.rs` covers text primitives, font facts, shaping/layout behavior,
  and render-facing text scene integration.
- Depends optionally on render-facing contracts in production code.
- API generator and artifact.

Ownership classification:

- Keep in `surgeist-text`: shaping, text layout, font abstractions, and local
  render projection tests for text-owned output.
- Candidate for `surgeist-test`: end-to-end text rendering snapshots or
  browser/platform comparison fixtures if they are added later.

Risk:

- Render-facing tests are acceptable while `surgeist-text -> surgeist-render`
  optional remains intended. If render integration becomes root-owned later,
  move broad render snapshots upward.

## `surgeist-render`

Render-owned test/tool surface:

- `src/tests.rs` covers scene, paint, draw data, backend-facing contracts, and
  rendering primitives.
- Optional `render-web` and `render-window` features.
- API generator and artifact.

Ownership classification:

- Keep in `surgeist-render`: render data model and backend contract tests.
- Candidate for `surgeist-test`: cross-backend visual/reftest harnesses once a
  shared rendering test apparatus exists.

## `surgeist-retained`

Retained-owned test/tool surface:

- `src/tests.rs` covers retained identity, tree mutation, state patches,
  snapshots, revisions, and stable handles.
- API generator and artifact.

Ownership classification:

- Keep in `surgeist-retained`: retained identity/state tests.
- Candidate for `surgeist-test`: cross-crate UI lifecycle tests combining
  retained, style, layout, render, and window.

## `surgeist-shape`

Shape-owned test/tool surface:

- `src/tests.rs` covers shape primitives and geometry/path data.
- API generator and artifact.

Ownership classification:

- Keep in `surgeist-shape`: all current tests. No shared apparatus pressure
  found.

## `surgeist-dialog`

Dialog-owned test/tool surface:

- `src/tests.rs` covers dialog contracts and coordination primitives.
- Optional `system` feature.
- API generator and artifact.

Ownership classification:

- Keep in `surgeist-dialog`: current dialog contract tests.
- Candidate for `surgeist-test`: future app/dialog/window e2e flows if dialogs
  become part of a full app surface test.

## `surgeist-task`

Task-owned test/tool surface:

- New skeleton crate with `src/lib.rs` smoke test and `api/public-api.txt`.

Ownership classification:

- Keep in `surgeist-task`: task scheduler contracts, task DSL, resource-class
  admission tests, Tokio-backed scheduler adapter tests, blocking/process lane
  tests, and connector-task test doubles.
- Candidate for `surgeist-test`: full app/task/connector e2e harnesses once
  they involve multiple production crates.

## API Artifact Tooling

Most crates use the same command-only API artifact pattern:

```sh
cargo run --manifest-path api/generator/Cargo.toml
```

Observed issue:

- Several repos have `api/generator/target/` present locally. `.gitignore`
  generally excludes these, but agents should avoid including generator build
  residue in inventory or diffs.

Ownership classification:

- Keep the API generator in each owning crate unless a dedicated shared API
  tooling crate is planned. The generated artifact is source-derived and
  crate-local.

## Recommended Ownership Rules

1. Production crate unit tests stay in the production crate.
2. Tests that need private helper knowledge stay near the owning crate until
   the helper is intentionally public or test-crate-facing.
3. Generated fixture corpora and browser/WPT parity artifacts should live in
   `surgeist-test`.
4. Generators that need browser downloads, upstream repository imports, or large
   fixture manifests should live in `surgeist-test` or a test-tool binary owned
   by `surgeist-test`.
5. Root tests should prove root facade wiring and small whole-system smoke
   behavior only.
6. Cross-crate e2e, reftest, system, and corpus tests belong in `surgeist-test`.
7. Compile-fail tests that protect a crate's public construction invariants
   should stay in that crate.
8. API generators and API artifacts remain crate-local unless explicitly
   replaced by a shared source-derived API tooling plan.

## Candidate Migration Lanes

### Lane 1: Root Oracle Extraction

Move or replace root `tests/oracle.rs` and `tests/layout_oracle.rs` with
`surgeist-test`-owned oracle/integration tests, keeping only a tiny root facade
smoke test that imports `surgeist::layout`.

Dependencies:

- `surgeist-test` needs layout oracle support APIs or copied test-only modules.
- Root should depend on `surgeist-test` only as a dev-dependency if needed.

Review focus:

- Root remains a facade.
- Layout still has focused local algorithm tests.
- No loss of coverage for grid/subgrid/inline oracle behavior.

### Lane 2: Layout Browser Parity Harness Extraction

Move `tests/layout/browser_parity/{README.md,corpus.toml,html,xml,scripts}` and
the parity XML parser/discovery harness from `surgeist-layout` to
`surgeist-test`.

Dependencies:

- `surgeist-test` needs fixture root APIs such as `fixture_files`,
  `Golden::parse_file`, and corpus metadata loading.
- `surgeist-layout` can keep a small smoke test or consume `surgeist-test` as a
  dev-dependency if that dependency direction is approved.

Review focus:

- Generated XML remains generated-only.
- HTML remains source of truth.
- Fixture counts and manifest accounting match before/after.
- Ignored/full corpus commands are preserved or intentionally renamed.

### Lane 3: Layout Generator Extraction

Move `surgeist-layout-generate` and the `layout-golden-generate` feature
dependencies from `surgeist-layout` into `surgeist-test` as a test-tool binary.

Dependencies:

- `surgeist-test` needs generator dependencies: browser driver, async runtime,
  serialization, SHA, TOML, and URL parsing.
- Generator output paths must target the new fixture root by default.

Review focus:

- `surgeist-layout` no longer carries browser/download/generator dependencies.
- Import/check-taffy behavior remains reproducible.
- Browser cache and environment variables are documented in the new owner.

### Lane 4: Root CSS/Style Integration Slimming

Move detailed root `tests/css.rs` and `tests/style.rs` cases to their owning
crates or to `surgeist-test` if they are truly cross-crate integration tests.
Leave root with public facade smoke coverage.

Dependencies:

- `surgeist-css` and `surgeist-style` should already own most detailed parsing
  and modeling tests.
- `surgeist-test` can own full CSS -> style -> layout fixture flows later.

Review focus:

- Root still proves reexports compile.
- Detailed failures point to the owning crate instead of the root facade.

### Lane 5: Shared Test Crate Growth

Build `surgeist-test` APIs in small increments before moving large corpora:

- fixture root discovery
- corpus manifest loading
- generated artifact provenance parsing
- XML expectation parsing
- oracle comparison helpers
- command wrappers for ignored/full corpus runs

Review focus:

- `surgeist-test` does not become a production dependency.
- APIs are test-facing and explicit.
- Large generated diffs are isolated from implementation crates.

## Immediate Next Questions

1. Should `surgeist-layout` be allowed a dev-dependency on `surgeist-test`, or
   should root/CI invoke `surgeist-test` separately for parity corpus checks?
2. Should the browser parity generator move before or after the checked-in
   fixture corpus?
3. Should root oracle files be copied into `surgeist-test` first, then removed
   from root after coverage parity is proven?
4. Should `surgeist-test` own all WPT/HTML fixture source files, including the
   future Smarty app-surface reftest corpus?
5. Should API generator tooling remain duplicated per crate for now?

## Suggested First Migration Plan

Start with a low-risk seed in `surgeist-test`:

1. Create `surgeist-test::layout::browser_parity` with fixture discovery and a
   minimal XML parser.
2. Copy one small checked-in XML fixture and a tiny `corpus.toml` seed.
3. Add `cargo test -p surgeist-test --test layout_browser_parity`.
4. Review the API shape.
5. Only then plan full corpus movement.

This avoids moving thousands of fixtures before the shared test crate has a
stable owner-facing API.
