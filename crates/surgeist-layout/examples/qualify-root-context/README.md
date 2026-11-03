# Root-context qualification

This example uses the maintained Layout adapter, [source inventory](corpus.toml) and Generator-owned cached Chrome lifecycle. See the [owning browser-corpus guide](../../tests/layout/browser_parity/README.md) for the shared resource/provenance boundary. It writes only an invocation-owned corpus and keeps the declared fixture, original source digests and expected case counts. Browser acquisition is ExistingOnly.

Run one ordinary declared source with:

~~~sh
cargo run --offline --locked -j 1 -p surgeist-layout --features layout-golden-generate --example qualify-root-context -- --fixture html/subgrid/subgrid_baseline_inline_column_inner_col1_first.html
~~~

Omitting the fixture selects the complete declared inventory. Numeric diagnostics remain a separate explicit mode; every numeric difference is still fatal and does not become an approved intentional difference.

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
