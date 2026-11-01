# surgeist-css Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns authored CSS syntax, intrinsic grammar validation, browser recovery,
diagnostic provenance, support metadata, and shared pure CSS language operations.
Ordinary authored parsing returns retained syntax and ordered diagnostics;
`CssParseReport::into_validation_result` accepts exactly clean reports without selecting
another grammar. Keep authored symbolic values unresolved. CSS owns source parsing,
exact authored expressions/decimals, contextual resolution, source/fact
qualification and specified serialization. [surgeist-color](../surgeist-color/README.md)
owns numerical values, conversion, interpolation, transformations, resolved
serialization and finite formatting under the [selected source editions](../../references/SOURCE-EDITIONS.md#html-color-delegation-and-intrinsic-contrast-policy).
Source and checked-authored used-color operations share one CSS resolver; the
borrowed authored feeder preserves actual paths and origins. Cascade, inheritance,
substitution, selector/query evaluation, resource retrieval, layout and painting
remain downstream. Consumers own their direct adapters, including HTML parser's
CSS/Color integration.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door | [src/lib.rs](src/lib.rs): the five crate-root items and the module list; every other public item has one path in its domain module |
| Parsing and recovery behavior | [src/parser/](src/parser/), [src/report.rs](src/report.rs), [src/parser_context.rs](src/parser_context.rs), and [src/diagnostic/](src/diagnostic/) |
| Token tree and token stream | [src/parser/syntax_types.rs](src/parser/syntax_types.rs) fixes the parser crate's `Document`, `Stream`, `Site` and errors to CSS's payload (checked components, and the host snapshot and control of a document); [src/syntax_consumption.rs](src/syntax_consumption.rs) builds the documents and holds the rule-unit consumers; [src/parser/property_input.rs](src/parser/property_input.rs) holds the provenance and metering extensions and `adapt_native`, the one bridge by which a native callback hands a scope, a value or a component to a ported grammar |
| Ported rule and declaration grammars | `KeyframeDeclarationParser` ([src/parser/keyframes.rs](src/parser/keyframes.rs)) and `DetachedMarginParser` with its block grammar ([src/parser/page.rs](src/parser/page.rs)) implement the parser crate's `RuleGrammar` and `BlockGrammar`, and so do the bodies of `@color-profile`, `@font-palette-values`, `@counter-style` and `@font-feature-values` (their files in [src/parser/](src/parser/)), whose descriptor values read from token streams through [src/parser/descriptor_stream.rs](src/parser/descriptor_stream.rs) (the value windows, the importance boundary, the native-error locations and the descriptor recovery driver); the value grammars of `@font-face` ([src/parser/font_face.rs](src/parser/font_face.rs), with its `unicode-range` re-tokenized as Syntax 3 section 5.5.11 says in [src/parser/font_face/unicode_range.rs](src/parser/font_face/unicode_range.rs)), the keyframe selectors, the `@page` selector list and descriptors, the `@keyframes` name, the `@layer` prelude, the media query grammar ([src/parser/queries.rs](src/parser/queries.rs) and its [features](src/parser/queries/features.rs)), the `@import` prelude ([src/parser/import.rs](src/parser/import.rs)), the `@supports` condition ([src/parser/supports.rs](src/parser/supports.rs)), the `@custom-media` prelude and the selectors ([src/parser/selectors.rs](src/parser/selectors.rs)), whose entry points for a native parser read its scope as a stream, read the same way; [src/parser/block_units.rs](src/parser/block_units.rs) turns the driver's failed units into the native errors and spans the recovery diagnostics take, and [src/parser/declaration_unit.rs](src/parser/declaration_unit.rs) admits a driver's declaration as `parse_declaration_core` admits a native one |
| Authored syntax, source coordinates, and property schema | The domain modules: [src/rule/](src/rule/), [src/declaration/](src/declaration/), [src/selector/](src/selector/), [src/query/](src/query/), [src/values/](src/values/), [src/property/](src/property/), [src/numeric/](src/numeric/), [src/color/](src/color/), [src/component/](src/component/), and [src/source/](src/source/) |
| Serialization, normalization, and specification provenance | [src/serialization/](src/serialization/), [src/normalize/](src/normalize/), and [src/conformance/](src/conformance/) |
| Verification and public-contract evidence | Tracked test modules in [src/](src/), rustdoc examples, and [tests/](tests/) |
| Shared handwritten property inventories and their consumers | [tests/css/support/property_expectations/README.md](tests/css/support/property_expectations/README.md) and `records.rs` in that directory |
| Corpus provenance, generated set, and maintenance owner | [tests/corpus/csstree/README.md](tests/corpus/csstree/README.md) and [tests/corpus/csstree/corpus.toml](tests/corpus/csstree/corpus.toml) |
| CSS-owned expectation and oracle contracts | [tests/csstree/](tests/csstree/), [tests/css/support/csstree.rs](tests/css/support/csstree.rs), and [tests/css/support/csstree_adapters.rs](tests/css/support/csstree_adapters.rs) |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

Integration tests share [tests/css.rs](tests/css.rs), grouped under
[tests/css/](tests/css/). Select focused coverage with `--test css` and a
module/test-name filter, rather than the former per-file target names.
Allocator tests use separate binaries; [Cargo.toml](Cargo.toml) and [tests/](tests/) establish the
current target inventory.

Validation and parsing share the default public surface. Full package tests include corpus
parser probes. The [corpus guide](tests/corpus/csstree/README.md) owns maintenance
with the local `surgeist-generator` CLI; do not hand-edit its generated set.
`tests/csstree/expected-classes.json` is CSS-owned expectation data; `oracle.json`
is separately captured by the ignored environment-gated test in
`tests/css/corpora/csstree/csstree_baseline.rs`. Capture is a writing
operation, not a routine check.

```sh
cargo check -j 1 --offline -p surgeist-css
cargo test -j 1 --offline -p surgeist-css
cargo test -j 1 --offline -p surgeist-css --doc
cargo clippy -j 1 --offline -p surgeist-css --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-css -- --check
```
