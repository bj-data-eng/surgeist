# Root-context qualification

This example uses the maintained Layout adapter, [source inventory](corpus.toml) and Generator-owned cached Chrome lifecycle. See the [owning browser-corpus guide](../../tests/layout/browser_parity/README.md) for the shared resource/provenance boundary. Ordinary qualification writes an invocation-owned corpus and keeps the declared fixture, original source digests and expected case counts. The fixed durable edition has the separate publication/check boundary below. Browser acquisition is ExistingOnly.

Run one ordinary declared source with:

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --fixture html/subgrid/subgrid_baseline_inline_column_inner_col1_first.html
~~~

Omitting the fixture selects the ordinary geometry cohort. The original Grid
Lanes axis sources retain the adopted-versus-browser orientation difference, so
strict original comparison reports it; see the [Chromium orientation witness](../../../../references/chromium-grid-lanes-orientation--149.0.7827.115.md).
Generated declarations describe capture admission, not matching geometry.
The
[prepared Range recipe](../../tests/layout/browser_parity/README.md#prepared-range-qualification)
selects its own capture profile. Numeric diagnostics use an explicit mode and
fail on every numeric difference.

Paired ordinary geometry uses a separate strict mode. It selects the declared
outgoing peer closure, binds that mode and the selected records in the focused
input, compares every local box through Native, and checks the declared world
rounding phase and raw DOM-size invariance. It does not admit the separate
ElementScrollSizes Native channel for nonhorizontal boxes.

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --paired-geometry-capture --fixture html/qualification/fixed-orthogonal-translation-0-75.html
~~~

## Element scroll sizes

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --element-scroll-size-diagnostics --fixture html/grid/grid_fit_content_percent_indefinite_max_content_hidden.html
~~~

This explicit request adds the actual DOM `clientWidth`, `clientHeight`,
`scrollWidth` and `scrollHeight` observations in a separate V4 sideband. The
initial profile admits horizontal ordinary DIV boxes with zero edges/gutters
and Visible, Clip or Hidden overflow. Unsupported roles or missing geometry
refuse. Historical expectation trees and their scroll-range fields retain
their existing meaning.

The request is authenticated as an invocation input. The complete numeric walk
compares its integer sizes against unrounded, origin-aware Native geometry;
world-rounded diagnostics are separate. All differences remain fatal and
unclassified. A selected source includes the closure of its declared observation
peers, so selecting `block-translation-0-75.html` also runs its existing phase
control without duplicating the authored sources. Paired element-size tuples
must agree while the declared world-corner phase change remains observable.

## Stylesheet custody

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --stylesheet-custody --fixture html/qualification/fixed-grid.html
~~~

This mode requires one Generated declaration without named-font certificates. It checks successful repeated setup, actual CSSOM rule changes without text changes, unknown and adopted sheets, and replacement by an identical STYLE clone. It restores the original STYLE node and real rules after each mutation, then compares a fresh complete capture with the pristine capture.

Detaching a stylesheet can create a new browser FontFace generation even when the same STYLE node is reattached. Therefore this destructive test mode rejects a declaration with named-font custody before browser acquisition; it cannot promise recovery of that declaration's original loaded font identity. The stylesheet/source-identity refusal checks remain enabled on the applicable source.

## Loaded-font custody

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --font-custody --fixture html/subgrid/subgrid_baseline_inline_column_inner_col1_first.html
~~~

This mode requires a Generated declaration with a positive named-font demand: atomic paragraph certificates, own-Grid reference records, or both. The one source inventory declares their separate counts/owner paths, and every variant must carry the actual complete profile. Grid-only sources such as `html/qualification/grid-reference-mixed.html` use the same command and retained loaded face; Grid name selection does not borrow paragraph zero-strut metrics. It does not detach or replace the stylesheet. It adds one real competing FontFace from the already owned Ahem rule, requires the exact competing-resource refusal, repeats the actual configure and font-preparation owners, deletes only the introduced face, and requires recovery with the same original loaded face/state. A temporary observer delegates the retained face's actual load method and rejects any reload during repetition/recovery; its original method property is restored afterward.

Both custody modes use the same helper infrastructure appended to the invocation's real helper bytes before hash registration. They do not replace the lowerer or synthesize outcomes. The final value is a new real capture after cleanup, not the retained pre-mutation snapshot. Historical observations, runtime certificate validation, strict Native comparison, source hashes and filesystem-tamper checks remain required; unrelated comparison failures remain fatal.


## Durable root-context edition

The fixed captured edition is separate from the historical raw HTML/XML corpus.
Its handwritten parent is [captured/](../../tests/layout/browser_parity/captured/README.md).
From the repository root, first verify the actual offline source import using the
explicit corpus path (the generator default is relative to its working directory):

~~~sh
SURGEIST_LAYOUT_BROWSER_PARITY_ROOT=crates/surgeist-layout/tests/layout/browser_parity cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-taffy-corpus
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --publish-root-context-v4
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-root-context-v4
~~~

Publish exclusively creates the fixed final directory

    tests/layout/browser_parity/captured/root-context-v4/

and generates there through existing Generator transactions. Any existing target,
including a raced-in file, directory or link, refuses. There is no overwrite,
relocation, retry, arbitrary-output or automatic cleanup mode. Failed preparation,
capture or validation leaves an **unregistered** diagnostic candidate; authorize
identity-aware recovery separately rather than deleting or regenerating over it.

The two durable modes reconstruct the complete fixed edition from the first
36,126 bytes of the current canonical [inventory](corpus.toml), sealed by SHA-256
`3d8fd3be19d7e4b5411133a09816954d174725f25d2f76725fc46594ec156abb`.
They parse that sealed declaration for all cases/counts; they never select jobs
from observed results or read captured output as source. Its retained corpus.toml
input is exactly those sealed source bytes, so the immutable edition is not
rewritten when qualification gains records.

Append new records after that byte boundary in the one canonical inventory.
Shortening, editing or reordering the prior prefix refuses, as do duplicate or
shadowed source keys anywhere in the complete current declaration. Current
capture commands select records from the canonical declaration according to
their request profile. The fixed-edition check covers only its sealed records.

Focused captures validate that complete catalogue first, then materialize
`focused-declaration.json` from the requested source, normalized capture mode
and complete selected typed records. Diagnostic selection includes the allowed
outgoing transitive peer closure. The input binds all selected metadata and
historical/policy identities; an unrelated catalogue append leaves it unchanged.
Runtime selection and the derived bytes share the same private state. Retained
`corpus.toml` receipts use their authenticated original bytes.

Both modes still compare the whole current canonical file with the current
executable before and after validation. Shared helper/base/font/browser-owner
freshness and all original source/import/history/certificate/Native/report gates
remain required. Changes to shared inputs or the sealed source prefix continue
to invalidate the fixed edition.

Snapshot HTML, helper/base/font files,
owner declarations and historical XML are capture witnesses of the handwritten
owners, not another editable source tree. The full canonical source-import receipt
and checked provenance are retained as hashed origin inputs. Selected originals
must match an imported entry/hash or an actual repository-authored member of the
same strict source owner; qualifier-authored records keep their separate origin.
This selected snapshot does not pretend to be another complete Taffy import.

The checker is read-only, browser-free and acquisition-free. It reconstructs the
same contract, checks current-owner freshness and exact input/output/report census,
then requires the historical trees, V4/font/control certificates, declared pairs,
Native geometry and Generator check to agree. Grid static-position XML must
match its exact qualified capture byte for byte. The remaining XML comparisons
require strict geometry. The checker does not prepare/copy inputs, run filesystem
tamper probes, regenerate, repair or remove files.
Maintenance currently requires Apple-Silicon macOS, including browser-free Generator
checks. Portable geometry consumers require neither Generator nor a browser.

Successful publication/checking is a gate **before** a separate reviewed library
registration or commit; directory existence and the generated report alone do not
register fixtures. Historical XML remains unchanged. The later library migration
covers 15 source families/60 artifacts and 43 distinct comparisons in ten XML
callers under grid::tests::browser_controls; that prefix also selects additional
Native/support tests. No automatic lookup alias is installed by these commands.

## Derive current terminal authority

Use the named derivation command to revalidate a stored Exact, Specialized or
Matching edition without capturing new browser observations:

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --derive-published-qualification grid-static-position-differences > /tmp/grid-static-position-qualification.json
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --derive-published-qualification prepared-range-qualification > /tmp/prepared-range-qualification.json
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --derive-published-qualification box-controls > /tmp/box-controls-qualification.json
~~~

The command retains the producing helper and all source, history, XML, report,
ledger and case identities. It runs the same complete source, observation,
Native and freshness checks as the edition's browser-free checker, then emits a
candidate terminal to standard output. The existing terminal remains unchanged;
only derivation permits that terminal to have stale validation authority.
A changed or missing retained input still refuses. Keep candidate files outside
capture roots because the complete artifact census rejects extra files.
Derivation never installs a terminal or changes its registration pin. Requalify
Grid static-position before checking the older `root-context-v4` predecessor
relationship. The historical `canonical-box-differences` predecessor remains
byte-preserved and refuses current derivation: `wrapping-world-corner-differences`
owns its live source record. Wire-schema fixture checks grant no live authority
to that predecessor. `root-context-v4` itself has no terminal.

## Matching geometry editions

The catalogue's `matching_publications` tables name cohorts of Generated box
sources. The [box-controls edition](../../tests/layout/browser_parity/captured/box-controls/qualification.json)
contains four originals across both box modes and directions, for sixteen cases.
The [scriptless-order edition](../../tests/layout/browser_parity/captured/scriptless-order/qualification.json)
contains the original scriptless Flex order fixture across all four variants.
Each case requires the declared root context, finite coordinates and sizes for
every box, and a complete Native comparison with zero raw differences. Original
sources must preserve their pinned historical observations. Authored sources
are read from this example's fixture directory and carry no historical
predecessor. Raw fields and qualification remain separate outputs.

Publication requires an absent `tests/layout/browser_parity/captured/box-controls`
directory, the verified original source import, and the cached pinned browser.
Check an existing edition with the second command.

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --publish-matching-geometry box-controls
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-matching-geometry box-controls
~~~

Publication uses the same Generator lifecycle and retains failed candidates without a
qualification terminal. Checking uses retained inputs and requires no browser.
Both commands authenticate the source, resources, report, ledger and current
owner, including the retained history for original sources. Registration
separately pins the complete terminal digest. An existing capture directory
alone establishes no authority. Each named cohort
retains its published membership and bytes.

The [FRI06 Subgrid edition](../../tests/layout/browser_parity/captured/fri06-subgrid-representatives/qualification.json)
contains three original BlockFlow sources in both box modes and directions.
The comparison checks ordinary box geometry, the six loaded-font paragraph
and BR observations, and the declared Grid references for each case.

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-matching-geometry fri06-subgrid-representatives
~~~

## Grid Lanes original diagnostics

The [Grid Lanes diagnostic receipt](../../tests/layout/browser_parity/captured/grid-lanes-axis-original-diagnostics/diagnostics.json)
binds all nine original axis fixtures in both box modes and directions. It
retains each original source and historical capture, the new browser
observation, and the full numeric difference list for that case. It grants no
matching authority.

Check the retained edition without a browser:

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-grid-lanes-original-diagnostics
~~~

The check authenticates the retained inputs and outputs and replays every
comparison. Numeric differences are recorded in the receipt; incomplete
comparisons or changes to the pinned evidence fail the check.

## Grid Lanes axis controls

The [Grid Lanes matching edition](../../tests/layout/browser_parity/captured/grid-lanes-axis-controls/qualification.json)
covers nine authored controls in both box modes and directions. The
[catalogue](corpus.toml) maps each control to its original and pins the original
diagnostic receipt. Each control removes only the template on the stacking
axis in each of its two Grid Lanes containers.

Check the matching edition and its original diagnostics without a browser:

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-matching-geometry grid-lanes-axis-controls
~~~

The check requires zero browser comparison differences for every control. It
replays the original diagnostics, checks the source changes and root contexts,
and compares the complete unrounded and final Native box packets, including
scroll geometry, for each pair. Independent checks cover the selected axes,
item placements and physical coordinates.

## Grid static-position edition

The [Grid static-position policy](../../semantic/grid-static-position-policy.md)
specifies the exact Layout differences for both original sources in both box
modes and directions. The catalogue attaches per-variant specifications after
the sealed root-context declaration. Qualification checks all requested
observations and both Native packet channels. Browser XML matches the sealed
root-context capture byte for byte.

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --publish-grid-static-position-differences
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-grid-static-position-differences
~~~

## Relative inset edition

The [relative inset policy](../../semantic/relative-inset-policy.md) covers
the original auto-height fixture in both box modes and directions. The edition
checks every requested coordinate and size, both Native packet channels, and
the three exact block-axis differences per variant.

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --publish-relative-inset-differences
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-relative-inset-differences
~~~

## Wrapping world-corner edition

The [wrapping policy](../../semantic/wrapping-world-corner-policy.md) binds the
original root world-corner height to its current browser measurement. The
edition checks that pair separately from the 24 root-local geometry fields
and retains both Native packet channels for every box. The catalogue pins
the superseded reference map and selects this edition for all four variants.

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --publish-wrapping-world-corner-differences
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-wrapping-world-corner-differences
~~~

## Original fixed-width Flex parent qualification

This edition captures the actual absolute Flex parent, its sole Grid child
and the two Ahem text leaves in three original Grid fixtures. Each fixture
has four direction and box-sizing variants. The capture verifies the original
stylesheet and its embedded Ahem WOFF2. Historical correspondence compares the
Grid-local subtree, and Native replay compares the whole parent tree with the
signed Grid offset.

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --fixture html/grid/grid_available_space_smaller_than_min_content.html
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-original-flex-parent-qualification
~~~

The first command captures one original with the cached browser. The second
replays the [fixed edition](../../tests/layout/browser_parity/captured/original-flex-parent-qualification/).
The publisher `--publish-original-flex-parent-qualification` requires an
absent edition directory and the verified original source import.

## Original percentage Flex parent qualification

The [percentage edition](../../tests/layout/browser_parity/captured/original-percentage-flex-parent-qualification/)
captures the absolute Flex parent and the three Grid subtree boxes in
`chrome_issue_325928327.html`. The four direction and box-sizing variants
retain the historical Grid-local geometry. Each browser rectangle agrees
before and after rounding, and Native replay compares both phases.

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --fixture html/grid/chrome_issue_325928327.html
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-original-percentage-flex-parent-qualification
~~~

The first command captures the four variants with the pinned browser. The
second replays the retained edition without a browser. The publisher
`--publish-original-percentage-flex-parent-qualification` requires an absent
edition directory and a verified original source import.

## Original subgrid body qualification

This edition authenticates the original subgrid gap fixture and its inline
stylesheet. The actual zero-edge body supplies the Native block-flow root.
The bordered Grid is its in-flow child. The comparison checks the Grid's
body-relative placement and all 16 Grid subtree boxes. Separate capture rules
select direction and box sizing for each variant.

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --fixture html/subgrid/subgrid_gap_percentage_border_bottom.html
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-original-subgrid-body-qualification
~~~

The first command runs a focused live capture. The second replays the
[fixed edition](../../tests/layout/browser_parity/captured/original-subgrid-body-qualification/)
with its retained inputs. The publisher
`--publish-original-subgrid-body-qualification` requires an absent edition
directory and the verified source import and cached browser.

## Prepared atomic percentage qualification

The original paragraph supplies its authored 80px containing block and the
actual inline block's 20px width and 50% height. Layout measures the atomic
before Text preparation. The pinned Ahem capture records both original text
occurrences and compares all boxes, line identities and Range rectangles
across both directions and box modes. The historical generic-font captures
supply a separate check of the container and atomic sizes.

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --fixture html/block/fri06_atomic_inline_percentage_block_size.html
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --check-published-prepared-atomic-percentage-qualification
~~~

The first command captures from the cached browser. The second replays the
[fixed edition](../../tests/layout/browser_parity/captured/prepared-atomic-percentage-qualification/).
The publisher `--publish-prepared-atomic-percentage-qualification` requires an
absent edition directory and the verified original source import. The
[Ahem resource provenance](../../tests/layout/browser_parity/fonts/PROVENANCE.md)
links the current license and upstream record.
