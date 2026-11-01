# surgeist-css-parser Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns CSS Syntax Module Level 3: input decoding and preprocessing,
tokenization, parsing results and token streams, parser entry points and
algorithms, the `An+B` microsyntax and token serialization. It gives syntax no
CSS meaning. [surgeist-css](../surgeist-css/AGENTS.md) owns grammars, property
and descriptor values, rules, selectors, queries and recovery diagnostics, and
depends on this crate. This crate depends on no Surgeist crate; do not add one.
Its one runtime dependency is `encoding_rs` (pinned exactly, with its license
files in [licenses/encoding-rs](licenses/encoding-rs)), which implements the
Encoding Standard's decoders for `input::decode`. (Its tests use
`surgeist-fuzz` for the property runner and `cssparser` as a black-box
comparison; both are dev-dependencies and ship nowhere.) Behavior
follows the
[pinned CSS Syntax 3 edition](../../references/SOURCE-EDITIONS.md#css-syntax-3-edition),
not the conventions of another implementation.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies and features | [Cargo.toml](Cargo.toml) |
| Public front door and module list | [src/lib.rs](src/lib.rs) |
| Clauses covered, their tests and what is not claimed | [docs/conformance-coverage.md](docs/conformance-coverage.md) |
| Crate purpose, status and edition pointer | [README.md](README.md) |

## Stepwise Protocol Facts

`input::InputCursor` and `lexer::protocol::TokenCursor` are host-only sans-IO
protocols: a host admits each request, owns every bank and may refuse an
append, and the cursors stay unchanged and resumable. Their sizes, and those
of `Scalar`, `Record` and the types inside them, are pinned in
[tests/layout_pins.rs](tests/layout_pins.rs) because hosts charge them (the
`scratch` amount of surgeist-css's UTF-16 characterization rows includes
`size_of::<InputCursor>()`). Do not change a pinned layout; keep new state
outside these types.

The cursors read preprocessed scalars (Syntax 3 section 3.3) and use the
[lexical](src/lexical.rs) predicates, which describe preprocessed input. Text
that has not been preprocessed is preprocessed per code point before asking a
predicate.

## Input Facts

[src/input/](src/input/) holds section 3.2 (`decode.rs`) and section 3.3
(`cursor.rs`, `filter.rs`, `preprocess.rs`). `decode` is the fallback-encoding
algorithm in the clause's order followed by the Encoding Standard's decode;
the decoded text is unfiltered. The `@charset` prefix is the exact byte
sequence of the clause, and its tests are the whole evidence for section 3.2
(the WPT `charset/*` tests are not in the corpus). `preprocess_utf16` loops the
`InputCursor` and charges a `Meter` with the growth model of the UTF-16
preparation it was moved from: capacities double from one entry (not the
four-entry floor of `limits::grow`), growth takes no opening step, and the text
moves one step per scalar. `limits::grow` charges differently, so it cannot
stand in without changing the work and bytes that hosts compare; the literal
values in [tests/receipts/utf16-preprocessing.tsv](tests/receipts/utf16-preprocessing.tsv)
were recorded from the old function and are checked by
[tests/input_receipts.rs](tests/input_receipts.rs). Do not regenerate them from
this crate. A failed preprocessing releases everything it charged.

## Lexer Facts

[src/lexer/machine.rs](src/lexer/machine.rs) is the one implementation of the
section 4.3 decisions: `step`, `start_kind`, `name_end_kind` and `suffix_end`
read the scalars ahead and say what to do; they change nothing. Two drivers
apply them, [`TokenCursor`](src/lexer/protocol/tokenizer.rs) one request at a
time and [`Lexer`](src/lexer/driver.rs) at once. The lexer's runs consume
scalars that an unbroken series of the machine's transitions would consume one
at a time; they take no decision of their own that the machine takes
differently. Add or change a decision in the machine, never in a driver, and
keep `every_short_text_is_lexed_as_the_stepwise_tokenizer_reads_it` and the
properties in [tests/properties.rs](tests/properties.rs) passing.

[tests/cssparser_oracle.rs](tests/cssparser_oracle.rs) compares the lexer with
cssparser 0.37.0 through its public API. Its differences are committed in
[tests/oracle/differences.tsv](tests/oracle/differences.tsv), and each row cites
the section of the pinned edition that makes the lexer's output correct. A new
difference is resolved from the specification text, never from the other
tokenizer's behavior; agreement with it is not evidence of conformance.

[tests/lexer_performance.rs](tests/lexer_performance.rs) is the performance gate
against cssparser's tokenizer. It is a measurement, ignored by default; see the
README for the command.

## Document And Stream Facts

[src/document/](src/document/) builds the token tree and
[src/stream/](src/stream/) reads it. `Document::parse` runs the `Lexer` once and
builds iteratively; a `TreeBuilder` adds tokens and groups one at a time and is
the only place that charges a `Meter` while building (one step per token, growth
through `limits::grow`, the storage returned when the document drops). Keep the
charge order of `TreeBuilder` and the step count per observation of `Stream`
(the metering section of the `Stream` docs and `tests/metering.rs`) unchanged: hosts
compare receipts across versions.

`Document::parse_owned` builds the same tree and keeps the text, so a host whose
document outlives the caller's text (surgeist-css's recovery) needs no lifetime;
the `String` is moved, not copied, and only a text an escape or NUL decoded is
appended after it. `Document::attach` gives a
document one payload of its own (`Document::attached`), for what a reader of a
`Site` or `Stream` needs and no node says, such as where the text came from.

`Site`, `Mark` and `Stream` implement `Clone`, `Debug` and, for the first two,
`Copy`, `PartialEq` and `Eq` by hand so that the payload type `P` needs no
bounds; do not replace them with derives. The depth policy is applied by
`Document::parse` alone (a `TreeBuilder` always fails a build that nests too
deep, because only text lets it find the end of a denied block). `parse` and
`parse_owned` take the `base_depth` that `Document::builder` takes, with the
same meaning: it counts against `Limits::max_depth`, a denied block is judged at
the same total depth, and it appears in `Metrics::maximum_depth`.
`Stream::comma_segments` hands the grammar every segment, empty ones included; it is
not the algorithm of section 5.4.2. The first meter refusal is latched on the
`Document`: later metered steps fail without asking the meter again, and a method
that cannot report a failure must never be the only check of an end condition (use
the fallible form, as `comma_segments` does).

A node holds only what every node has (a `Stored` token, its interval, the list
and place that hold it, the list it owns, the payload), 72 bytes for `P = ()`;
what only a function or simple block has (its kind, how it ended, where its
opening token ends) lives in the `List` the group owns, so a leaf pays nothing
for it, and `list_end` is derived from how the group ended, never stored twice.
`NodeId` and `ListId` are `u32`: a document holds at most 2^32 nodes (`NodeId`
positions `0` through `u32::MAX`, and the same bound on a list's nodes) and at
most `u32::MAX` lists (`ListId` is a `NonZeroU32` of position plus one), and a
pooled text is two `u32` positions, so the decode pool of `Document::parse`
holds at most `u32::MAX` bytes. Going past any of these fails the build with a
capacity overflow. Every charge is the
`size_of` of what is stored (`tests` in [src/document/tree.rs](src/document/tree.rs)
check it), so a layout change moves the byte totals hosts compare, never the
step counts.

The sizes of the public handles (`Site`, `Mark`, `Stream`, `Component`, the errors, `Token`) are pinned in
[tests/layout_pins.rs](tests/layout_pins.rs), and those of the private per-node storage
(`Text`, `Stored`, `Node`, `List`; 16, 40, 72 and 88 bytes, `Node` 88 with a one-word payload)
in the unit test of [src/document/store.rs](src/document/store.rs); a change is a
decision, not an accident.

Tests: [tests/document.rs](tests/document.rs) (tree and navigation),
[tests/stream.rs](tests/stream.rs), [tests/metering.rs](tests/metering.rs) and
the tree properties of [tests/document_properties.rs](tests/document_properties.rs),
whose oracle is a recursive reading of sections 5.5.7 to 5.5.10 written in the
file, not the builder under test.

## Rule Driver, An+B And Serialization Facts

[src/grammar/](src/grammar/) holds the section 5.4 entry points
(`entry.rs`, methods of `Stream`), the section 5.5 algorithms (`algorithm.rs`:
`consume_at_rule`, `consume_qualified_rule`, `consume_declaration`, each over
the nodes of one list) and the two iterators of units (`iterator.rs`). The
drivers reach nodes only through `Stream::peek_at`, which takes a meter step and
returns the refusal as an error: never read a driver's end condition from
`next_token`, `is_empty` or `discard_whitespace`, which turn a refusal into the
end of the stream. A refusal ends an iterator with one `Failed` for which
`is_refused()` holds, and `tests/grammar.rs` refuses every call of a run in turn
to keep that. Keep a driver's consumption independent of what the grammar says
about a unit (`verdicts-do-not-change-extents` in
[tests/syntax_properties.rs](tests/syntax_properties.rs)). `RuleGrammar` and
`BlockGrammar` are provisional until `surgeist-css` has ported implementors; do
not widen them for a test.

[src/serialize.rs](src/serialize.rs) writes tokens so that they read back as
themselves. Its escaping asks `lexical::is_ident_code_point`, so it follows the
non-ASCII rule of the lexer and changes with it. `Writer` adds two separators
that the table of section 9 lacks (a `<!` before two hyphens, and the ident `--`
before `>`), found by the round-trip properties of
[tests/properties.rs](tests/properties.rs), which must keep passing for any
change to the lexer or the serializer.

[tests/cssparser_oracle.rs](tests/cssparser_oracle.rs) also compares `parse_nth`,
`serialize_identifier`, `serialize_string` and the separator table of cssparser
with `anb::parse`, `serialize::identifier`, `serialize::string` and
`serialize::needs_separator`; the differences are the rows of
[tests/oracle/syntax_differences.tsv](tests/oracle/syntax_differences.tsv), each
citing the clause that makes this crate's answer correct.

`anb::parse` and `anb::parse_saturating` are one recognizer (the `Policy` of
[src/anb.rs](src/anb.rs)): they differ only in what an integer outside `i32` is,
an overflow or the nearest `i32` as each integer token reads before the
production's sign applies. Keep the recognition in one place.

The sizes of `AnPlusB`, `AnPlusBError`, `RuleEnd`, `UnitKind`, `Failed`,
`AtRule`, `QualifiedRule`, `Declaration` and `Adjacency` are pinned in
[tests/layout_pins.rs](tests/layout_pins.rs).

## WPT Vector Facts

[tests/corpus/wpt](tests/corpus/wpt/README.md) holds the `css/css-syntax` WPT
files that the pinned edition cites, imported by `surgeist-css-generate
import-wpt` (never hand-edit `source/` or its receipt) with the vectors adapted
from them. [tests/wpt_syntax.rs](tests/wpt_syntax.rs) checks every vector's
binding to its upstream path, digest and line before running it. Write an
expectation from the upstream assertion and the pinned edition, never from this
crate's output; a case whose observable is a CSS object-model, selector,
property-validity or `var()` result is deferred and claimed nowhere. A
disagreement between upstream and the edition goes in a profile file, and the
edition governs. [NOTICE.md](NOTICE.md) records the packaged fixtures' license.

## Command Inventory

Select affected checks under the root process/resource policy, from repository
root:

```sh
cargo check --offline --locked -j 1 -p surgeist-css-parser
cargo test --offline --locked -j 1 -p surgeist-css-parser
cargo clippy --offline --locked -j 1 -p surgeist-css-parser --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-css-parser -- --check
```
