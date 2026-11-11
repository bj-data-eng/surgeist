# Surgeist Layout Browser Parity

This harness runs Surgeist layout against browser-derived parity fixtures. The
corpus is rooted at `tests/layout/browser_parity`:

- `html/`: current constrained HTML corpus. This includes the pinned Taffy
  green baseline plus Surgeist-authored fixtures.
- `xml/`: generated browser expectations consumed by the Rust parity runner.
- `corpus.toml`: source provenance, import contracts, and expected-failure or
  quarantine accounting.
- `scripts/gentest/`: shared browser measurement helper and base CSS.

XML files are generated artifacts. Do not edit browser geometry in XML by hand;
update the source fixture, importer, manifest, or generator instead.

The sizing bridge is a finite fixture adapter, not app-facing CSS parsing. It
accepts finite px and percentage values plus the existing unitless fixture
values; affine `calc()`; nested `min()`, `max()`, and `clamp()`; one-argument
`fit-content()`; canonical `calc-size()`; and only property-valid keywords.
Expression depth is limited to 64. A finite, non-negative `fr` is accepted only
for a maximum track breadth, never a minimum track breadth or a box/flex
property. Integration layers outside this layout-ready harness own authored CSS
parsing and lowering.

Prepared Text parity retains the original-source profile introduced in
[measurement protocol version 2](measurement-protocol.md)
and the exact [pinned font](fonts/PROVENANCE.md). Source capture and browser Range
observations are separate channels. Historical measured-width text tuples are
schema-only; the native production reader rejects them. The complete
[124-variant accounting](prepared-text-accounting.json) records their actual
source/resource/profile and execution state. No historical tuple or passing
schema test is counted as native Text parity.

## FRI-05 Normalized Scroll Adapter

The fixture adapter reads browser computed style and serializes the normalized
attributes `overflow-x`,
`overflow-y`, `scrollbar-width`, `overflow-clip-margin`, `scrollbar-gutter`, the
four physical `scroll-padding-*` edges, the four physical `scroll-margin-*`
edges, `scroll-snap-type`, `scroll-snap-align`, and `scroll-snap-stop`. It emits
the atomic overflow pair together and omits exact initial values where the
fixture schema permits. The Rust adapter accepts only the finite keyword,
pixel, percentage, and `calc()` subsets represented by the production layout
types; it rejects authored shorthands (`overflow`, `scroll-padding`, and
`scroll-margin`), CSS-wide keywords, variables, unresolved units, and ambiguous
snap forms. Root CSS/style integration owns full grammar, cascade, computed
value normalization, logical-to-physical lowering, and explicit scrollbar
environment selection.

Fixture `scroll_size` means the span of the canonical physical range:
`x.maximum() - x.minimum()` and `y.maximum() - y.minimum()`. An expected zero is
still compared; missing geometry, wrong x span, and wrong y span fail rather
than being skipped. The comparator does not treat a maximum endpoint as a size.
Target fields are layout-produced metadata only: this harness does not model
retained association, transformed coordinates, current offsets, snap selection,
CSSOM, host scrollbar UI, or runtime events.

The [computed overflow edition](captured/fri05-computed-overflow/qualification.json)
covers 44 variants across the original Block, Flex, Grid, and Grid Lanes
sources. The default Layout test verifies the original topology and compares
every registered box tree and recorded scroll-span pair.

Current source pins, import rules, artifact inventory, and expected outcome counts
are declared in `corpus.toml`. Ordinary Cargo test runs compare the original XML
and registered current editions without launching a browser or regenerating
fixtures. Generation infrastructure changes must preserve XML bytes and complete
per-fixture outcomes unless browser expectation changes are separately intended.

Run checked-in fixtures:

```sh
cargo test --offline --locked -j 1 -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml
```

This is the raw-exact result: every original XML runs on its own captured
evidence, and registered current outputs run separately even when their names
match an original. Any mismatch or incomplete observation fails. Historical
missing-root-context errors remain raw failures; a current capture never turns
them into historical passes.

The [original Block axis edition](../../../examples/qualify-root-context/README.md#original-block-axes-qualification)
covers all 20 direction and box-sizing variants across five writing modes.
The focused Block axis test checks the original topology and compares every
registered rectangle tree through Native layout.

The [original Flex axis edition](../../../examples/qualify-root-context/README.md#original-flex-axes-qualification)
covers 80 variants across five writing modes and four Flex directions.
The focused Flex axis test checks the original topology and compares every
registered rectangle tree through Native layout.

The [original Grid axis edition](../../../examples/qualify-root-context/README.md#original-grid-axes-qualification)
covers 36 variants across nine parent and child writing-mode combinations.
The focused Grid axis test checks the original tracks and diagonal placement,
then compares each registered rectangle tree through Native layout.

The [original Subgrid axis edition](../../../examples/qualify-root-context/README.md#original-subgrid-axes-qualification)
covers 36 variants across nine parent and child writing-mode combinations.
The focused Subgrid axis test checks both inherited axes and the original
topology, then compares each registered rectangle tree through Native layout.

The [original inline Subgrid alignment edition](captured/original-inline-subgrid-alignment/qualification.json)
retains the two static inline Grid sources and all eight historical rectangle
trees. The two focused callers replay each source's registered cases, including
the vertical-lr nested Subgrid and the aligned 20px items.

The [positioned RTL Subgrid edition](captured/original-relative-rtl-subgrid-positioned/qualification.json)
compares the four historical Grid trees beneath the actual Body. The focused
caller checks all registered variants, including the absolute child's local
rectangle in the relative RTL Subgrid.

The [FRI08 plain-box manifest edition](captured/fri08-manifest-plain-box-predecessors/qualification.json)
qualifies twelve active Grid and Subgrid variants. The focused manifest test
checks their exact historical associations and complete geometry.

The [Grid Lanes original diagnostic receipt](captured/grid-lanes-axis-original-diagnostics/diagnostics.json)
records the complete raw numeric differences from all 36 source-qualified
original axis captures. It is separate from matching editions. Use the
[diagnostic check](../../../examples/qualify-root-context/README.md#grid-lanes-original-diagnostics)
to replay those comparisons.

The [Grid Lanes axis controls](../../../examples/qualify-root-context/README.md#grid-lanes-axis-controls)
cover the same nine families and four variants. The qualifier catalogue
accounts for each original with its source-equivalent control. Matching
authority requires zero raw differences for the control and exact Native
geometry for the authenticated pair. The original differences remain in the
diagnostic receipt. The Grid Lanes axis test checks the original both-template
topology and executes the mapped controls.

The ordinary default-feature test target runs all registered editions.

```sh
cargo test --offline --locked -j 1 -p surgeist-layout --test layout runs_registered_current_browser_parity_qualification
```

The [canonical box edition](captured/canonical-box-differences/qualification.json)
covers four collapsed Flex wrapping variants. The
[measured Grid edition](captured/measured-grid-differences/qualification.json)
covers 28 variants across min-content and fit-content, max-content and
fit-content(10px), and thirteen-track intrinsic sizing. The measured Ahem
leaf witness binds four or sixteen characters on each side of U+200B.
The comparison checks Layout geometry and DOM integer sizes in both box
modes and directions. Hidden captures include both Layout scroll-range spans.

The [Flex text edition](captured/flex-text-differences/qualification.json)
covers four collapsed single-line variants. It binds the original WOFF2
face and direct text occurrences before preparing the labels. The raw
comparison covers the five original elements. Qualification also checks
both packet channels for the anonymous items and text leaves, and
verifies the shared placed baseline.

The [prepared Range edition](captured/prepared-range-qualification/qualification.json)
covers four continuity variants. The two LTR controls match exactly. The two RTL
variants qualify only the PDI start under the Native source-anchor policy and
retain the original source and actual Range piece identity.

The [sideways Flex edition](captured/flex-composition-qualification/qualification.json)
covers four composition variants. Each has nine collapse differences. Each RTL
variant also has one scrollbar difference under a separate policy. The edition
binds the measured image, both Native packet phases, and the browser environment
used by the collapse-disabled gutter control.

The reader authenticates each terminal seal, retained inputs and outputs,
browser profile, policy, and per-variant identity. It compares the retained
selected records with one live catalogue and checks exact bytes for the other
live inputs. Unfiltered execution requires capture authority for every
registered variant across the editions. Each qualification checks the full
independent Native oracle, both packet channels, and the exact raw difference
set. A missing field, new or changed difference, unexpected pass, or late
freshness failure fails qualification. Raw differences remain available
independently.

`SURGEIST_PARITY_FILTER` can select a fixture substring for diagnosis. Filtered
execution makes no complete-edition coverage claim. Run without it for the
full declared cohort. These checked-in comparisons need no browser or fixture
regeneration.

The publication owner creates and validates each immutable edition.

```sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --publish-canonical-box-differences
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-canonical-box-differences
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --publish-measured-grid-differences
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-measured-grid-differences
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --publish-flex-text-differences
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-flex-text-differences
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --publish-prepared-range-qualification
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-prepared-range-qualification
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --publish-flex-composition-qualification
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-flex-composition-qualification
```

Publication uses the already-acquired pinned browser and refuses an existing
edition directory. The check is read-only and browser-free.
`--derive-published-qualification <edition>` runs the same complete source,
observation, Native and freshness checks for stored Exact, Specialized or Matching
editions and writes candidate terminal bytes to standard output. It retains the
producing helper and leaves the edition directory untouched. Keep redirected
candidates outside capture roots; derivation neither installs a terminal nor
updates its registration pin.

A successful publication emits terminal authority only after all source, capture, raw,
Native, qualification and freshness checks pass; registration is a separately
reviewed seal change. The earlier `root-context-v4` publication remains a
distinct immutable edition.

Browser generation and source import require the shared generator's supported
mutation host, Apple-Silicon macOS. Default layout builds remain independent of
this tooling requirement.

Regenerate XML fixtures from constrained HTML fixtures:

```sh
cargo run --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate
CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH=tmp/surgeist-browser/.../Google\ Chrome\ for\ Testing cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
```

`generate` resolves the manifest-owned Chrome-for-Testing pin and may use the
managed cache/fetcher. `generate-existing` is the no-fetch artifact path: its
browser path must be nonempty, repository-relative, under the manifest cache,
executable, and report the exact pinned `--version`. Both modes use the one
manifest-owned launch profile, with disabled Chromium defaults/cache and
`use-mock-keychain`; batch, timeout, polling, retry, profile, and page lifetimes
are all manifest-owned.

`SURGEIST_BROWSER_CACHE` and `SURGEIST_BROWSER_VERSION` are not overrides and
are rejected for generation. `SURGEIST_LAYOUT_GENERATE_FILTER` is empty for an
unfiltered full run. With `generate-existing`, it may instead name one
normalized fixture path or prefix that matches at least one source. Such a
filtered run is an optional iteration diagnostic: it writes matching XML and
the private ownership ledger, leaves reports unchanged, and is not current
corpus verification evidence.

Import or verify the pinned Taffy green baseline:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- import-taffy
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-taffy-corpus
```

The Taffy baseline is acquired at the exact repository/revision declared in
`corpus.toml` under `tmp/surgeist-sources/taffy/<commit>`. Verified imports retain
Surgeist-authored HTML and write `html/.surgeist-source.json`, binding the source
pin and imported bytes. Adopting a corpus without this attestation first checks
its existing imports against the verified checkout. `check-taffy-corpus` verifies
the existing checkout and import without fetching.

Browser installations live under `tmp/surgeist-browser/`, separated by version.
Both caches are repository-local, already ignored by `tmp/`, and survive normal
`cargo clean`. Completed acquisitions are reused only after verification;
profile and publication recovery do not remove them.

`SURGEIST_LAYOUT_BROWSER_PARITY_ROOT` may select a self-contained corpus root.
The exact corpus and browser-owner roots are checked separately. A full run
validates declared accounting before atomically publishing XML and reports.
Filtered generation leaves full reports and unrelated artifacts unchanged.
A full run with per-fixture failures publishes diagnostic accounting and retains
outputs that were not regenerated. The private `xml/generation-ownership.json`
ledger permits a subsequent full run to repair filtered or diagnostic state; it
does not replace full-report provenance. Machine-local coordination and recovery
state lives in the ignored `.surgeist-generator/` directory beneath the corpus.

`xml/generation-reports/all.json` uses the shared browser engine's schema 4 and
is the generated-artifact provenance authority. It binds engine/host identity,
browser executable and launch settings, source/helper inputs, import attestation,
linked resources, XML hashes, and generated/unsupported/expected-failure/
quarantined/generation-failure accounting. XML remains comment-free.

`check-corpus` is browser-free and acquisition-free. It validates persisted
attestations, source/resource/artifact hashes, paths, inventory, and accounting
without requiring either cache. Browser-selection variables and generation
filters do not affect this command. XML parsing and semantic comparison remain
layout-owned tests; the generic engine treats artifact bytes as opaque.

HTML fixtures are the human-readable source of truth. The constrained `html/`
fixtures are runnable today. XML fixtures are generated browser expectations for
fast Rust-side regression tests.

Unsupported browser features are not silently skipped. A fixture outside the
closed native source/font/context profile remains visible with a classified
unsupported result; historical measured-width text cannot enter production Text.

The supported entry point is layout's thin `surgeist-layout-generate` binary,
enabled by `layout-golden-generate`. It depends on the sibling workspace
`surgeist-generator` at version `=0.2.0` with `browser-corpus`; default layout
builds do not include that tooling graph. The shared crate owns acquisition,
browser processes, provenance, accounting, and publication. Layout owns
document/helper preparation, measurement decoding, unsupported classification,
and XML serialization.
`scripts/gentest` remains helper-only: `test_helper.js` and `test_base_style.css`.

The helper and adapter use the strict [version-4 measurement
protocol](measurement-protocol.md). Closed wire decoding and semantic validation
precede XML serialization. Preloaded helpers must implement that protocol;
unversioned or malformed measurements return contextual adapter errors.

Inline display values are parsed so constrained fixtures remain readable:
`inline-block`, `inline-grid`, and `inline-grid-lanes` participate as atomic
inline boxes with inner block, grid, or grid-lanes formatting contexts. Full
parity failures from fixtures containing those displays now report their actual
geometry mismatch kind rather than an unsupported-inline bucket.

Ordinary box-only atomic displays (`inline-block`, `inline-grid`, and
`inline-grid-lanes`) reach normal geometry comparison. Native prepared text uses
original UTF16, the exact pinned Ahem face and captured supported style context.
Its finite profile includes transparent inline boundaries, bidi-format controls,
forced breaks and authored fixed-pixel empty atomics. Uncovered text, unsupported
inline box/strut context and unresolved atomic sizing are explicitly refused;
version-1 numeric records remain historical schema evidence.

Native Range observations query the completed batch's actual placed Text through
[`source_range_fragments`](../../../../surgeist-text/src/range_fragments.rs),
using each exact prepared origin's nonempty original-local interval. Source
coverage and finalized-line ownership are checked before comparing observations.
Text supplies the font-metric geometry and the distinction between removed source
and retained zero-width units; the adapter does not reconstruct these from line
struts or glyph presence. Browser comparison still checks source, line and inline
position/advance only. Native font-band qualification adds no browser block-axis
or glyph-ink parity claim.

Inline engine impact measured on 2026-06-17:

- `SURGEIST_PARITY_FILTER=subgrid`: 632 failures, with no unsupported inline,
  named placement lowering, or grid-line validation bucket. Remaining failures
  are geometry buckets: height mismatch 248, width mismatch 188, x mismatch
  172, y mismatch 24.
- `SURGEIST_PARITY_FILTER=grid-lanes`: 48 failures, with no parse, lowering, or
  inline-dispatch failures. Remaining failures are geometry buckets: height
  mismatch 24, width mismatch 24.

Named grid placement syntax is parsed directly into layout inputs for parity
fixtures and exercised through production layout. The formerly blocked subgrid
line-name fixtures `subgrid_line_names_004_b_to_b_minus_1` and
`subgrid_line_names_repeat_outer_span_a_to_a_8` now run as ordinary parity
fixtures, so any future failures should report their concrete mismatch or
validation kind.

## Prepared Range qualification

The Text-migration accounting records four executed historical successors and
twelve additional native representatives: 14 match, and the two RTL continuity
variants retain an executed zero-width PDI Range observation difference.
The [V4 prepared Range receipt](captured/prepared-range-differences/raw-comparisons.jsonl)
qualifies both RTL starts under the Native source-anchor policy and retains the
two matching LTR controls. The browser-free replay checks all four captures and
compares the full report byte for byte. The
[measurement protocol](measurement-protocol.md#executed-range-observation-difference)
defines the policy and source bindings.

```sh
cargo test --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context prepared_range_qualification_accepts_only_the_declared_rtl_pdi_differences
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --prepared-range-capture --fixture html/block/prepared_text_continuity_bidi.html
```

The second command selects the prepared Range profile, uses the pinned browser,
and writes the comparison report to standard output. The ordinary root-context
capture selects the geometry cohort separately.
The [horizontal BR edition](captured/horizontal-br-font-band-qualification/)
compares four original pinned Ahem variants. The
[vertical BR edition](captured/vertical-br-font-band-qualification/) compares
twelve original variants in their authored writing modes and directions.
Both compare the selected font band, both inline blocks and the exact control
relations. The [empty-line BR edition](captured/empty-line-br-font-band-qualification/)
compares two bands per variant across eight original variants. Its font comes from the original base stylesheet.
See the [prepared-input and observation contract](measurement-protocol.md#authentic-prepared-text-input).

```sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-horizontal-br-font-band-qualification
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-vertical-br-font-band-qualification
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-empty-line-br-font-band-qualification
```

Root-context capture is versioned separately from the unchanged prepared font/source
profile. The [measurement contract](measurement-protocol.md#root-requests-and-observation-frames)
defines source-qualified root contexts and separate ICB/nearest-scrollport/local-limit
facts. Historical viewport metadata cannot issue a Native operation. Preserve original
XML and raw diagnostics while qualifying a self-contained candidate corpus. No browser
root rectangle, used Auto body height or viewport-filling heuristic supplies new inputs.

## Sideways Flex composition qualification

The fixed sideways Flex source uses separate
[collapse](../../../semantic/flex-composition-collapse-policy.md) and
[classic scrollbar edge](../../../semantic/flex-composition-scrollbar-policy.md)
policies. The [comparison receipt](captured/flex-composition-differences/raw-comparisons.jsonl)
retains all requested browser and Native fields, both Native packet phases,
and the exact differences assigned to each policy.

Run the browser-free replay from the workspace root.

```sh
cargo test --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context retained_sideways_flex_replay_qualifies_all_four_variants
```

To regenerate the comparison, run the focused capture with the pinned browser
available in the Layout cache. It writes the comparison records to standard
output.

The [current focused report](flex_composition_computed_margin_report.json) and
[focused declaration](flex_composition_fixed_orthogonal_declaration.json) bind
the captured helper, inherited Ahem typography, browser, and original source
to the collapse-disabled gutter control. Admission checks the complete
environment and its exact per-variant input projections. Published replay
checks its separate publication contract and producing inputs.

```sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --fixture html/flex/fri07_flex_composition.html
```

On macOS arm64, the separate
[gutter diagnostic](scripts/diagnose-flex-composition-gutter.mjs) uses the
browser executable from the sealed receipt and a Node runtime with a
WebSocket client. Its JSON output matches the retained
[collapse-disabled control](captured/flex-composition-differences/gutter-control.json).

```sh
pisct run --timeout-seconds 55 -- node crates/surgeist-layout/tests/layout/browser_parity/scripts/diagnose-flex-composition-gutter.mjs
```
