# Root-context qualification

This example uses the maintained Layout adapter, [source inventory](corpus.toml) and Generator-owned cached Chrome lifecycle. See the [owning browser-corpus guide](../../tests/layout/browser_parity/README.md) for the shared resource/provenance boundary. Ordinary qualification writes an invocation-owned corpus and keeps the declared fixture, original source digests and expected case counts. The fixed durable edition has the separate publication/check boundary below. Browser acquisition is ExistingOnly.

Run one ordinary declared source with:

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --fixture html/subgrid/subgrid_baseline_inline_column_inner_col1_first.html
~~~

Omitting the fixture selects the ordinary geometry cohort. The
[prepared Range recipe](../../tests/layout/browser_parity/README.md#prepared-range-qualification)
selects its own capture profile. Numeric diagnostics use an explicit mode and
fail on every numeric difference.

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
strict Native geometry and Generator check to agree. It does not prepare/copy
inputs, run filesystem tamper probes, regenerate, repair or remove files.
Maintenance currently requires Apple-Silicon macOS, including browser-free Generator
checks. Portable geometry consumers require neither Generator nor a browser.

Successful publication/checking is a gate **before** a separate reviewed library
registration or commit; directory existence and the generated report alone do not
register fixtures. Historical XML remains unchanged. The later library migration
covers 15 source families/60 artifacts and 43 distinct comparisons in ten XML
callers under grid::tests::browser_controls; that prefix also selects additional
Native/support tests. No automatic lookup alias is installed by these commands.
