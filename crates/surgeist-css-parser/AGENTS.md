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
(Its tests use `surgeist-fuzz` for the property runner and `cssparser` as a
black-box comparison; both are dev-dependencies and ship nowhere.) Behavior
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

`Site`, `Mark` and `Stream` implement `Clone`, `Debug` and, for the first two,
`Copy`, `PartialEq` and `Eq` by hand so that the payload type `P` needs no
bounds; do not replace them with derives. The depth policy is applied by
`Document::parse` alone (a `TreeBuilder` always fails a build that nests too
deep, because only text lets it find the end of a denied block).
`Stream::comma_segments` hands the grammar every segment, empty ones included; it is
not the algorithm of section 5.4.2. The first meter refusal is latched on the
`Document`: later metered steps fail without asking the meter again, and a method
that cannot report a failure must never be the only check of an end condition (use
the fallible form, as `comma_segments` does).

The sizes of the public handles (`Site`, `Mark`, `Stream`, `Component`, the errors, `Token`) are pinned in
[tests/layout_pins.rs](tests/layout_pins.rs), and those of the private per-node storage
(`Node`, `List`; 168 and 64 bytes at step 6) in the unit test of
[src/document/store.rs](src/document/store.rs); a change is a decision, not an accident.

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
