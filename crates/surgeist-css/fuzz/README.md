# CSS ingestion, recovery and authored-value fuzz cases

This verification package consumes the public `surgeist-css` front doors
(byte decoding, sheet, style-attribute and property-value parsing, specified
serialization, CSSOM declaration selection and known-property metadata) and the
public `surgeist-css-parser` encoding types. Shared
[surgeist-fuzz](../../surgeist-fuzz/README.md) owns engines, workers,
supervision, artifacts, replay and reduction. CSS owns the codecs, generators,
expectations and seeds here. No CSS production API or private
algorithm is copied or exposed, and no production crate depends on this
package.

| Target | Encoding | Section | Oracle |
| --- | --- | --- | --- |
| `surgeist-css::byte-decoding` | structured | [Byte decoding](#byte-decoding) | Syntax 3 section 3.2 selection order and standard-library UTF-8 and UTF-16 decoders |
| `surgeist-css::utf8-source` | raw bytes | [UTF-8 source](#utf-8-source) | The documented coordinate, clean-report, retained-node and UTF-16 parity contracts |
| `surgeist-css::property-value` | structured | [Property values](#property-values) | Property metadata, handwritten known values and normalization rules from the crate reference |
| `surgeist-css::structured-rules` | structured | [Structured rules](#structured-rules) | The recipe that rendered the source, and the CSSOM selection rule |

## Shared contracts

Every target uses one evaluator for ordinary workers, property testing,
coverage and canonical replay. Features `property` and `coverage` are optional;
default workers have neither engine, and the owned Rust forbids unsafe code,
including the coverage macro. A case has a strict versioned canonical encoding
(`CSBD`, `CSPV` and `CSRL` magic, revision 1) that rejects unknown tags and
revisions, truncation, trailing bytes and table indices or counts out of bounds
before any allocation. The raw-bytes target's canonical case is its exact
source bytes. A random seed is never the reproducer: the framework captures
canonical bytes before evaluation.

The checks shared by the targets, in [src/check.rs](src/check.rs) and
[src/nodes.rs](src/nodes.rs), are:

- Diagnostic and retained-node positions follow the decoded-source coordinate
  contract of the [crate reference](../docs/reference.md#source-coordinates-and-diagnostics):
  a UTF-8 byte offset on a scalar boundary, a zero-based line, and a column in
  UTF-16 units, where CR LF, CR, LF and FF each end one line. The expectation
  is an independent scan of the source, not the parser's position routine. The
  responsible position lies inside its recovery span, and a report orders its
  diagnostics by that offset.
- `is_clean` is exactly an empty diagnostic slice, and validation accepts
  exactly that report while preserving every diagnostic in order.
- Retained declarations are consistent with their property identity, appear in
  strictly increasing source order, carry provenance inside the parsed source,
  and keep their position at the start of their name. Retained rules appear in
  source order.

Generation reads entropy through a cursor that returns zero past the end, so one
slice always yields one case, and charges the framework budget before
construction. Bounds are in [fuzz.toml](fuzz.toml): 8 KiB of canonical bytes for
a byte-decoding case; 16 KiB for a source or a rules case, which also bounds
recipes to 64 nodes, depth 5 and 12 items per block; 4 KiB, 24 value fragments
and numbers of at most 40 digits (at most 2,304 text units, which covers the
largest admitted recipe) for a value case. A generation never discards; every
generated case is evaluated.

Outcomes are distinct. A property violation is a failure whose key is the
violated property and a stable discriminator. A decode refused at a declared
byte limit is an expected rejection with checked properties. A serialization
that the crate declines is reported by omitting the round-trip property from the
checked list of that case, so it never counts as round-trip coverage.

## Byte decoding

The case keeps the exact original bytes (up to 4096), an optional protocol
label (up to 64 bytes of UTF-8), an optional environment encoding from a table
of nine Encoding Standard labels, and the original and decoded byte limits.
Bytes reach `CssDecodedStylesheetInput::decode` unchanged; no lossy UTF-8
conversion stands in for the decoder.

The expected encoding and selection cause follow the order of
[Syntax 3 section 3.2](../../../references/SOURCE-EDITIONS.md#css-syntax-3-edition)
as the [encoding guidance](../docs/reference.md#stylesheet-encoding-guidance)
states it: a byte order mark overrides everything; otherwise a valid protocol
label, then an exact `@charset "label";` wholly within the first 1024 bytes (a
UTF-16 label selects UTF-8), then the environment encoding, then UTF-8. The
label-to-encoding expectation is a handwritten table of the Encoding Standard
labels the generator's vocabularies use (leading and trailing ASCII whitespace
and case do not matter, as the reference states); a test checks that it covers
every vocabulary label and agrees with the registry the decoder consults,
`surgeist_css_parser::input::Encoding::for_label`, which only builds the hints.

The expected text uses the standard library: `String::from_utf8_lossy` for
UTF-8 and `char::decode_utf16` for UTF-16, with the Encoding Standard's single
error for a lone high surrogate followed by an odd final byte. Windows-1252 is
checked where no byte lies in 0x80 to 0x9F. Other encodings (Shift_JIS, EUC-KR,
ISO-2022-JP, ISO-8859-2, the replacement encoding) have no independent decoder
here, so their text is not claimed as checked. Limits must refuse the whole
decode at their exact edge, and an accepted limited decode must equal the
unlimited one. The reference says the two limits bound original and decoded
bytes independently and that an input over either refuses the whole decode
([reference](../docs/reference.md#entry-points-and-features) and the
`CssDecodedStylesheetInput::decode` rustdoc); neither orders the two errors, so
when both are exceeded either limit error is accepted. The decoded snapshot is then parsed in Standards and Quirks
modes: diagnostics against the decoded-source coordinates, the clean report,
equality with parsing the same text, and, for a recognized charset prefix, the
documented leading `UnknownAtRule` drop at offset 0.

## UTF-8 source

The raw case is valid UTF-8 or it is rejected by the codec. The evaluator runs
`parse_sheet` and `parse_block_contents` in both parser modes and checks the
shared contracts, retained rule and declaration order, and determinism: two
parses are equal and `source::same_provenance`. Where CR, FF and NUL are absent
UTF-16 preprocessing is the identity, so `parse_sheet_utf16` and
`parse_block_contents_utf16` must retain equal syntax and report the same
diagnostic kinds, actions and coordinates.

Generation assembles rules and declaration lists from escaped, uppercase,
custom and Unicode names, strings, comments, numeric extremes, `var()` and
`env()`, truncated constructs, `<!--` and `-->`, stray delimiters, every line
ending, U+FEFF and U+2028; builds punctuation soup and wide Unicode; and nests
`{`, `(` and `[` around depths 1, 62, 63, 64, 127, 128, 129, 255, 256, 257 and
300, which straddle the 63, 128 and 256 level thresholds the crate documents.

## Property values

Five lanes share one case. The metadata lane takes a property from
`CssKnownProperty::all()` and checks that `from_name` of its canonical name and
every alias returns it, that metadata exists and agrees with `grammar()`, and
that each CSS-wide keyword, `var()` and `env()` form is retained as that
property with the right global or substitution-dependent classification, clean,
with and without `!important`, in the property-value front and the
block-contents front. The text written for these forms is fixed by the metadata
name and the form alone. Aliases must write the same declaration as the
canonical name.

The known-value lane carries 22 handwritten rows from the crate reference:
exact length units, six fractional places, `0.25` for `25%`, the shortest CSSOM
shorthand form, two-value `background-size`, and two rows whose number does not
survive six-place text. Each row gives the expected text of the first
serialization and of the serialization of that text reparsed.

Specified serialization is canonicalizing, so the round trip uses an explicit
normalization. The first serialization must reparse cleanly to the same property
and importance. Omission of a default is selected on exact values before the
lossy six-place number text (CSSOM section 6.7.2; see the header and
`tiny_nonzero_math_does_not_compare_equal_to_actual_zero` in
[projected_number_formatting_lifecycle.rs](../tests/css/numeric/projected_number_formatting_lifecycle.rs)),
so a number that does not survive six-place text serializes rounded first and,
reparsed as that exact value, can then equal and omit a default:
`transition: e 0.0000001s` writes `transition: e 0s;` and then
`transition: e;`; `border-image: none 100.0000001%` writes
`border-image: 100%;` and then `border-image: none;`. The fixed point is
therefore required at the first serialization only when every number in the case
survives six-place fixed text: for a row, when its handwritten second text equals
its first; for a generated recipe, when its text has no math function, no
exponent and no number with more than three fraction digits (three digits stay
within six places after the largest unit shift in the vocabulary, milliseconds to
seconds). That decision reads only the text this package wrote, never product
output. Otherwise the serialization of the first text must reparse cleanly to the
same property and importance, and reserialize to itself, so the second and third
texts are equal. Only rows whose serialization changes spelling alone also
require structural equality (`==`) with the original parse; `opacity: 25%` and a
four-value `margin` are normalized and are checked for their exact texts and as
fixed points. A recovered original may repeat a diagnostic it already reported
when its text is reparsed, and no other.

The sibling lane injects one of twelve balanced invalid units before, between or
after two independent valid rows and requires both siblings back in order with
their documented text and importance, a diagnostic, and every diagnostic inside
the injected unit. The closure lane takes values cut before their closing
delimiter: the property-value front must retain them with an implicit-closure
recovery, and `CssComponentValues::parse` plus
`CssDeclaration::try_from_components` must reject the cut form and accept the
complete one. The recipe lane builds values for 36 curated names (34 known
properties and two custom properties) from vocabulary words, numbers made from digit counts (signs,
exponents to 999, 40-digit integers and fractions), and separators including
comments, and checks the fronts agree, diagnostics, retained nodes and the round
trip.

Literals 30 and 31 are the closed and the cut form of a `{}` block beside
`var()` for `width`. Syntax 3 section 5.5.6, step 8 refuses such a value in
every front, as the declaration front does
([#2884](https://github.com/bj-data-eng/surgeist/issues/2884)); the recipe lane
checks that the property-value front and the block-contents front agree on it.

## Structured rules

A recipe tree of style rules, conditional groups (`@media`, `@supports`,
`@layer`, `@container`), unknown and invalid units, ignorable text and, inside
blocks, declarations, invalid declarations, nested rules and comments is
rendered by [src/rules.rs](src/rules.rs), which records the byte range of every
injected invalid unit and what must survive. Sheet and style-attribute fronts are
both used; the latter drops and reports valid nested rules.

The evaluator checks: retained rule kinds in source order and recursively;
valid declaration identities and importance in source order through escaped,
uppercase, Unicode and custom names (custom names stay case-sensitive and
decode); every diagnostic inside an injected unit and one in each; a clean
report exactly when nothing invalid was injected; and CSS-owned declaration
selection. For each rule's declaration list,
`CssSpecifiedDeclarationBlock::try_from_declarations` must keep exactly the last
important declaration of each name, or else the last one, at the winner's own
authored position. This restates the CSSOM selection rule over longhands and
custom properties only; shorthand expansion belongs to the crate's own tests.
Where the parser splits declarations between a rule and its nested declaration
runs is not asserted.

## Seeds and imported corpus

`seeds/bytes`, `seeds/source`, `seeds/values` and `seeds/rules` are the
source-qualified literal cases of each target. They are generated from the
literal factories through the owning codec and checked for freshness by
`tests/replay.rs`:

```sh
cargo run --offline --locked -j 1 -p surgeist-css-fuzz --example materialize_seeds -- crates/surgeist-css/fuzz/seeds
```

`seeds/csstree` holds 349 inputs from the CSS-owned
[CSSTree corpus](../tests/corpus/csstree/README.md), imported through its
committed neutral expectations rather than from an upstream checkout. The
importer enumerates the expectation files through the owner's generation report
(`artifacts[].output_path`), which lists every file including those in nested
directories, and requires each file's case count and the total to equal the
report's. The corpus holds 935 cases (`expected_cases` in its `corpus.toml`,
`counts.active` in the report). The seeds are raw `utf8-source` inputs from the
contexts `stylesheet`, `declarationList`, `block`, `rule`, `declaration` and
`atrule` (364 cases). [corpus-import.toml](corpus-import.toml) records the
corpus revision and manifest digest, the MIT licence file, each imported case
and its corpus id, 15 of those inputs skipped as duplicates of an earlier import,
and the excluded contexts with their case counts and reasons: `value` (193
cases, a value needs its property, which the corpus does not carry), `selector`
(317), `selectorList` (10), `mediaQuery` (49) and `atrulePrelude` (2), which
need a fragment front this package does not call. `tests/corpus.rs` takes the
owner's total from `corpus.toml` and the report and walks the expectation tree
itself; it checks that every seed is the corpus input of its id, that the record
matches the corpus revision and digest, that imported, duplicate and excluded
cases add up to the owner's total, and that every seed passes the shared
evaluator and replays exactly.

```sh
cargo run --offline --locked -j 1 -p surgeist-css-fuzz --example import_csstree -- crates/surgeist-css/fuzz
```

## Explicit exclusions

These are exclusions, not coverage:

- Other source fronts: selector, media query, `@import`, `@supports`,
  `@page`, `@font-face`, keyframe and other descriptor fronts, raw single rule,
  style block and declaration fronts, `parse_page_block_contents` and
  `parse_font_face_block_contents`.
- Color resolution, calculations beyond serialization, substitution, cascade,
  selector matching, resource loading and CSSOM mutation, which are not CSS
  ingestion contracts or belong to other crates.
- Serializers other than a declaration's `serialize_specified`: sheet, rule and
  CSSOM serialization are not round-tripped.
- Shorthand declaration selection and `expand()` results. `expand()` is called
  for totality only.
- UTF-16 sources with unpaired surrogates, and UTF-16 inputs where CR, FF or NUL
  preprocessing changes the text. The byte-decoding target includes unpaired
  UTF-16 code units, which its decoder replaces.
- Windows-1252 bytes 0x80 to 0x9F and every non-UTF encoding other than
  Windows-1252 have no independent decoded-text expectation.
- Component limits, parser-thread failure and resource refusals are not
  exercised beyond the default limits and the byte-decoding limits.
- The first-error text of diagnostics and display strings.

## Verification and campaign boundary

```sh
cargo test --offline --locked -j 1 -p surgeist-css-fuzz
cargo test --offline --locked -j 1 -p surgeist-css-fuzz --all-features
cargo clippy --offline --locked -j 1 -p surgeist-css-fuzz --all-targets --all-features -- -F unsafe-code -D warnings
cargo fmt --all -- --check
```

Use bounded `pisct run` and an isolated target for concurrent package changes.
Tests exercise real worker binaries over the framed protocol, registration
through Cargo metadata, every literal, strict codecs, deterministic generation,
finite property and coverage drivers, seed freshness, the corpus import, and one
planted-failure control per target. Each control runs a deliberately broken
expectation, detects the failure, replays from canonical bytes under both the
raw and structured encodings and reduces to the same failure key, while the
sound expectations accept the same input. Coverage-driver parity is not an
instrumented libFuzzer campaign.

A clean committed source and a separate finite campaign directory serve the
framework's `list`, `validate`, `check`, `run`, `replay` and `minimize`; see its
[workflow](../../surgeist-fuzz/README.md#run-a-finite-campaign). `--input-bytes`
must not exceed the target's declared byte limit and must admit the largest
canonical case the campaign evaluates; the framework default is too small.

## Bounded runs

A finite successful run establishes only the inputs it evaluated, the properties
they checked and the build it used. It is not a conformance percentage, a
performance baseline or a memory bound: the framework's process backend reports
RSS sampling unavailable and cannot enforce a hard memory limit, so these
commands use memory policy `unbounded`. "Evaluated" counts attempts that checked
at least one property, and the summary does not separate verified outcomes from
expected rejections (a decode refused at a byte limit is one).

Build the framework command, prepare a clean committed checkout of the source
under test outside the worktree you are changing, and use a work directory that
does not exist yet (an existing empty one failed with an I/O failure whose cause
was not established):

```sh
cargo build --offline --locked -j 1 -p surgeist-fuzz --features execution --bin surgeist-fuzz
surgeist-fuzz list --source "$src" --crate surgeist-css
surgeist-fuzz validate --source "$src" --crate surgeist-css
```

Generated `check` (stable toolchain), one target; `--input-bytes` is the
target's declared byte limit (8192 `byte-decoding`, 16384 `utf8-source` and
`structured-rules`, 4096 `property-value`):

```sh
surgeist-fuzz check --source "$src" --work "$work" --target surgeist-css::structured-rules \
  --toolchain 1.99.1 --memory-policy unbounded --cases 2000 --wall-ms 900000 \
  --case-ms 5000 --build-ms 1200000 --input-bytes 16384 --seed 1147 \
  --storage-entries 100000 --storage-bytes 8589934592
```

Regression run over a registered seed directory (every regular file once;
`--cases` must admit them all, so use at least the directory's file count; the
`seeds/csstree` directory has 349):

```sh
surgeist-fuzz check --source "$src" --work "$work" --target surgeist-css::utf8-source \
  --toolchain 1.99.1 --memory-policy unbounded \
  --regressions "$src/crates/surgeist-css/fuzz/seeds/csstree" \
  --cases 400 --wall-ms 600000 --case-ms 5000 --build-ms 1200000 --input-bytes 16384 \
  --storage-entries 100000 --storage-bytes 8589934592
```

Coverage-guided `run` (libFuzzer with AddressSanitizer; a dated nightly that is
already installed; the per-case deadline is a whole number of seconds):

```sh
surgeist-fuzz run --source "$src" --work "$work" --target surgeist-css::utf8-source \
  --toolchain nightly-2026-09-01 --memory-policy unbounded --cases 3000 \
  --wall-ms 900000 --case-ms 5000 --build-ms 3000000 --input-bytes 16384 --seed 1147 \
  --storage-entries 100000 --storage-bytes 8589934592
```

A finding stops the run and is stored as an immutable artifact under
`failures/`. `replay --artifact failures/ID` reproduces its failure key from the
canonical bytes, and `minimize --artifact failures/ID --attempts 200
--confirmations 2 --wall-ms 600000 --case-ms 5000` reduces it while keeping the
key. Both need the same storage quotas as above: the default quota stopped a
replay with "artifact entry count exceeds quota".
