# surgeist-css-parser

`surgeist-css-parser` implements CSS Syntax Module Level 3 for Surgeist:
decoding and preprocessing the input stream, tokenization, the parsing results
and token streams, the parser entry points and algorithms, the `An+B`
microsyntax and token serialization. It gives syntax no CSS meaning. Grammars,
property values, rules, selectors, queries and recovery diagnostics belong to
[surgeist-css](../surgeist-css/README.md), which builds CSS language
interpretation on this crate.

## Status

The crate is being built module by module, each with the specification-derived
tests for the clauses it implements. It currently offers:

- `lexer`: the free-running `Lexer`, which reads text as authored (section 3.3
  is applied as it reads, offsets are original bytes) and returns one `Lexeme`
  at a time, a `Token` with its source interval, and the lexical errors the
  specification calls parse errors. `Options` allows `unicode-range` tokens
  (section 4.3.14) and rebases offsets for a sub-range of a larger text.
  `lexer::protocol` is the same state machine in its stepwise form, for hosts
  that admit each scalar, decision and stored record: a narrow host-only
  contract, not a general lexer API. The sizes of its cursors and records are
  pinned because hosts charge them (`tests/layout_pins.rs`).
- `Token`, with `token::TokenKind`, `token::HashKind`, `token::Number` (exact
  spelling, sign and type flag, converted only on request) and
  `unicode_range::CodePointRange`.
- `input`: the stepwise `InputCursor` for section 3.3. It decodes UTF-8 bytes
  or UTF-16 units, applies the preprocessing, and reports each scalar with the
  interval of original units it came from.
- `lexical`: the code-point classes of Syntax 3 section 4.2 and the checks of
  sections 4.3.8 to 4.3.11 (valid escape, would start an ident sequence, a
  number or a unicode-range) as `const fn`s. The non-ASCII ident code point
  follows the 2021 rule for now; adopting the pinned edition's narrower set is
  a deliberate later change.
- `Offset`: a byte offset into the text a tokenizer reads.
- `limits`: `Limits`, `DepthExcess`, the `Meter` contract through which a host
  admits work and storage, and the charged helpers `grow`, `copy_str` and
  `copy_boxed`.
- `Document`: the token tree of section 5.2, built once and iteratively by
  `Document::parse` from text, or by a `document::TreeBuilder` from tokens and
  groups a caller already holds (checked components, for example). A function
  or simple block is one node that owns a list, so skipping a group costs
  nothing. Each node may carry a payload of a type `P` the consumer chooses
  (the default `()` means none). The document answers where a node is
  (`location`, `parent_group`, `children`, `list_end`, `node_at_offset`) and
  carries the `limits::Meter` of its build, if any. A `DepthExcess::DenyBlock`
  build keeps a `{}`-block that nests too deep as one opaque node.
- `Stream`: the token stream of section 5.3 over a document, with the
  operations the section names (next token, empty, consume a token, discard
  whitespace, `mark` and `restore`), consumers for one kind of token, entering
  a function or block (`within_block`), splitting a list at a stop token
  (`within_list_until`, `stream::StopToken`), and `Site` boundaries that give
  offsets and the text between them. Every observation, including the scan
  of `within_list_until` for its stop token, takes a step of the document's
  meter per node examined; the first refusal is latched on the document
  (`Document::is_refused`, `Stream::is_refused`). `Stream::comma_segments`
  (the segment helper for the `#` multiplier, which is not the algorithm of
  section 5.4.2) and `Stream::parse_complete` (section 5.4.1) are primitives
  for grammars. The errors are `stream::StreamError` and `SyntaxError`, structs
  with `is_*` predicates.
- `grammar`: the parser entry points of section 5.4 (`parse_rule`,
  `parse_declaration`, `parse_component_value`, `parse_list_of_component_values`,
  `parse_comma_separated_component_values` and the literal
  `parse_comma_separated_list`) and the drivers of section 5.5 (a stylesheet's
  contents, a block's contents, with the algorithms for an at-rule, a qualified
  rule, a declaration and the rules about custom properties and `!important`).
  A driver consumes a whole unit, prelude and block, and then asks the grammar
  through the `RuleGrammar` and `BlockGrammar` traits whether it is valid, so a
  rejection leaves nothing to recover; each unit reports how it ended
  (`RuleEnd`), and a unit that was not accepted is a `Failed` record. The trait
  shapes are provisional until the first grammars of `surgeist-css` prove them.
- `anb`: the `An+B` microsyntax of section 6, recognized by `anb::parse` into
  two `i32` and serialized by `AnPlusB`'s `Display` (section 9.1). A value
  outside `i32` is a typed overflow, never a saturated number.
- `unicode_range::consume_value`: section 5.5.11, which tokenizes the text of a
  `unicode-range` descriptor with unicode ranges allowed.
- `serialize`: section 9. `Token`'s `Display`, `serialize::identifier`,
  `serialize::string` and `serialize::url` write tokens so they read back as the
  same tokens (the round trip governs where it differs from the escaping in
  CSSOM), `serialize::needs_separator` is the table of tokens that need a comment
  between them, `serialize::Writer` writes a run of tokens with the comments
  and `serialize::list` the component values of a document.
- `match_ignore_ascii_case!`: ASCII case-insensitive matching of a string
  against lowercase literal arms.

Sections 3.2 (decoding bytes) and 5.4.3 (the stylesheet object) are not here
yet. The [coverage map](docs/conformance-coverage.md) lists the clauses of the
pinned edition with their tests and claims only what those tests establish.

## Edition

The behavioral authority is the 1 October 2026 Candidate Recommendation Draft
of CSS Syntax Module Level 3, recorded under the
[CSS Syntax 3 edition](../../references/SOURCE-EDITIONS.md#css-syntax-3-edition)
with its [local copy](../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md).
A new edition is adopted deliberately and recorded there.

## Dependencies

The crate depends on no other Surgeist crate, and `surgeist-css` depends on it.
The direction differs from
[surgeist-html-parser](../surgeist-html-parser/README.md), which sits above the
HTML values it constructs: CSS Syntax 3 is a language-independent layer below
every CSS grammar, so it cannot depend on CSS values. The [explanation](../../docs/explanation.md#language-values-authored-state-and-live-identity)
places both among the repository's crate boundaries.

## Verification

Run from the repository root:

```sh
cargo check --offline --locked -j 1 -p surgeist-css-parser
cargo test --offline --locked -j 1 -p surgeist-css-parser
cargo clippy --offline --locked -j 1 -p surgeist-css-parser --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-css-parser -- --check
```

The WPT `css/css-syntax` tests that the edition cites are kept as test fixtures
with reviewed vectors adapted from them ([corpus](tests/corpus/wpt/README.md));
their license is in the [notice](NOTICE.md).

## Performance gate

The lexer must match or beat cssparser's tokenizer on the
CSSTree corpus sources and on a 1 MiB synthetic style sheet, in a release build
and unmetered. `tests/lexer_performance.rs` measures it; it is ignored and runs
only when `SURGEIST_CSS_PARSER_PERF=1` is set:

```sh
SURGEIST_CSS_PARSER_PERF=1 cargo test --release --offline --locked -j 1 -p surgeist-css-parser --test lexer_performance -- --ignored --nocapture
```
