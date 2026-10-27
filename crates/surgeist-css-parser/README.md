# surgeist-css-parser

`surgeist-css-parser` implements CSS Syntax Module Level 3 for Surgeist:
decoding and preprocessing the input stream, tokenization, the parsing results
and token streams, the parser entry points and algorithms, the `An+B`
microsyntax and token serialization. It gives syntax no CSS meaning. Grammars,
property values, rules, selectors, queries and recovery diagnostics belong to
[surgeist-css](../surgeist-css/README.md), which is to build CSS language
interpretation on this crate.

## Status

The crate is being built module by module, each with the specification-derived
tests for the clauses it implements. It currently offers:

- `lexical`: the code-point classes of Syntax 3 section 4.2 and the checks of
  sections 4.3.8 to 4.3.11 (valid escape, would start an ident sequence, a
  number or a unicode-range) as `const fn`s. The non-ASCII ident code point
  follows the 2021 rule for now; adopting the pinned edition's narrower set is
  a deliberate later change.
- `Offset`: a byte offset into the text a tokenizer reads.
- `limits`: `Limits`, `DepthExcess`, the `Meter` contract through which a host
  admits work and storage, and the charged helpers `grow`, `copy_str` and
  `copy_boxed`.
- `match_ignore_ascii_case!`: ASCII case-insensitive matching of a string
  against lowercase literal arms.

There is no tokenizer, document or parser yet. The
[coverage map](docs/conformance-coverage.md) lists the clauses of the pinned
edition with their tests and claims only what those tests establish.

## Edition

The behavioral authority is the 1 October 2026 Candidate Recommendation Draft
of CSS Syntax Module Level 3, recorded under the
[CSS Syntax 3 edition](../../references/SOURCE-EDITIONS.md#css-syntax-3-edition)
with its [local copy](../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md).
A new edition is adopted deliberately and recorded there.

## Dependencies

The crate depends on no other Surgeist crate, and `surgeist-css` is to depend
on it; it does not yet. The direction differs from [surgeist-html-parser](../surgeist-html-parser/README.md),
which sits above the HTML values it constructs: CSS Syntax 3 is a
language-independent layer below every CSS grammar, so it cannot depend on CSS
values. The [explanation](../../docs/explanation.md#language-values-authored-state-and-live-identity)
places both among the repository's crate boundaries.

## Verification

Run from the repository root:

```sh
cargo check --offline --locked -j 1 -p surgeist-css-parser
cargo test --offline --locked -j 1 -p surgeist-css-parser
cargo clippy --offline --locked -j 1 -p surgeist-css-parser --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-css-parser -- --check
```
