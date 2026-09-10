# Pinned WPT CSS parsing evidence

This corpus preserves an explicit subset of [Web Platform Tests](https://github.com/web-platform-tests/wpt)
at commit [`ddcca5943fd41232d42149aaa19d9a04c5651b18`](https://github.com/web-platform-tests/wpt/commit/ddcca5943fd41232d42149aaa19d9a04c5651b18),
committed `2026-09-10T01:09:39Z`, before the reviewed
`2026-09-10T02:16:24Z` cutoff. It contains ten authored HTML tests plus the
upstream [BSD-3-Clause license](source/LICENSE.md), totaling 34,542 source/license
bytes. Copyright belongs to web-platform-tests contributors. Original source
author declarations remain in the files and in [acquisition.json](acquisition.json).

[corpus.toml](corpus.toml) binds each selected source and license path to its
reviewed SHA-256. The acquisition record gives the exact immutable HTTPS URL,
size, digest, and applicable license for every file. The generator-owned
`source/` tree and its `.surgeist-source.json` receipt must be imported together,
never hand-edited. The receipt verifies declared manifest file digests; it does
not claim a local Git tree attestation. Acquisition and import do not execute
the HTML, JavaScript, browser test harness, or font resources.

## CSS-owned adaptations

The JSON vectors are reviewed literal adaptations, owned by `surgeist-css`.
They are separate from imported source and from the generator's receipt. Each
case binds to its upstream path, raw source digest, one-based source line, and
literal case text. The [parser harness](../../wpt_parsing.rs) verifies those
bindings with the shared test digest helper before exercising `parse_sheet`.
It never derives expected acceptance from Surgeist output.

| Data | Active expectation |
| --- | --- |
| [font-face-src-presence.json](font-face-src-presence.json) | 109 literal `src` records: 63 surviving descriptors and 46 absent descriptors after recovery |
| [nesting-selector-presence.json](nesting-selector-presence.json) | 31 retained inner style rules and one rejected inner rule, with the authored parent retained |
| [nesting-declaration-order.json](nesting-declaration-order.json) | Five authored rule/declaration sequences, including nested media and trailing declaration groups |
| [deferred-cssom-and-execution.json](deferred-cssom-and-execution.json) | 116 source-only assertions with explicit deferred dispositions; no parser-conformance pass is claimed for them |

Font-source acceptance means that `src` remains present in a recovered
`@font-face` rule. It does not mean the entire descriptor input is a clean strict
parse. A bad comma-separated member may be discarded while a valid fallback
survives; an all-invalid list leaves no `src` descriptor. These upstream tables
do not specify exact whole-descriptor serialization or font loading outcomes.

Nesting vectors test authored structure and order. They preserve symbolic
selectors without treating CSSOM's contextual selector rewriting as a
standalone selector-normalization rule. Leading declarations and later
declaration runs are compared through the owning public syntax APIs. Expected
property sequences are a reviewed adaptation of the source bodies and upstream
group-order assertions, not an assertion of DOM object identity.

The deferred data retains exact upstream CSSOM shorthand getter and declaration
serialization results, contextual selector serialization, nested CSSOM object
projections, technology-set serialization, and computed custom-property tests.
A pending `border-color` getter returning an empty string does not reject its
authored `border` shorthand. The 65 token-pair cases assert computed results
start/end with their supplied token strings and differ from naive concatenation;
they do not supply an exact separator string. The six comment cases and empty
fallback case also require computed substitution. Do not reuse these results
as authored-value canonical output. CSSOM mutation, substitution execution,
matching, cascade, font loading, layout, and rendering remain downstream work.

## Verification and maintenance

Ordinary parser verification is offline and consumes committed data:

```sh
cargo test --offline --locked -j 1 -p surgeist-css --test wpt_parsing -- --test-threads=1
```

Run the generator from the Surgeist root with existing absolute `CORPUS_OWNER`
and `CORPUS_ROOT` paths to verify raw source/license inventory and receipt:

```sh
cargo run --offline --locked -j 1 -p surgeist-generator --no-default-features --features css-corpus --bin surgeist-css-generate -- --owner-root "$CORPUS_OWNER" --corpus-root "$CORPUS_ROOT" check-corpus
```

For explicitly authorized maintenance, acquire only the reviewed allowlist at
immutable URLs, update its acquisition record and manifest, then use the same
generator command with `import-wpt --source-root "$WPT_SOURCE_BUNDLE"`.
Review raw source/license changes and every affected vector binding together.
`generate` does not support WPT transformation; it cannot run or adapt the
upstream JavaScript. Import mutation requires Apple-Silicon macOS; source
acquisition, import, and vector edits are separate maintenance operations.
