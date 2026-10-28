# Pinned WPT css-syntax evidence

This corpus preserves the [Web Platform Tests](https://github.com/web-platform-tests/wpt)
`css/css-syntax` files that the pinned CSS Syntax 3 edition cites, at commit
[`ddcca5943fd41232d42149aaa19d9a04c5651b18`](https://github.com/web-platform-tests/wpt/commit/ddcca5943fd41232d42149aaa19d9a04c5651b18),
committed `2026-09-10T01:09:39Z`. It is the same pin as the
surgeist-css WPT corpus (`crates/surgeist-css/tests/corpus/wpt`) and the same
reviewed cutoff, so `serialize-consecutive-tokens.html` and the
license have the same digests in both. The pin predates the edition
(`2026-10-01`); 22 of the 23 cited tests were found at the pin and one was not.

It contains 22 authored HTML tests plus the upstream
[BSD-3-Clause license](source/LICENSE.md), totaling 42,822 source/license bytes.
Copyright belongs to web-platform-tests contributors. Original author
declarations remain in the files and in [acquisition.json](acquisition.json).

[corpus.toml](corpus.toml) binds each selected source and license path to its
reviewed SHA-256. The acquisition record gives the exact immutable HTTPS URL,
size, digest and applicable license of every file. The generator-owned
`source/` tree and its `.surgeist-source.json` receipt are imported together by
`surgeist-css-generate import-wpt`, never hand-edited. The receipt verifies
declared manifest file digests; it does not claim a local Git tree attestation.
Acquisition and import do not execute the HTML, JavaScript or browser harness.

## What is and is not here

- **The 23 cited tests.** The edition's "Tests" lists name 44 files; 23 of them
  are outside `css/css-syntax/charset/` and are the allowlist.
  `parse-invalid-input.html` is cited but returned HTTP 404 at the pin, so it is
  recorded under `cited_but_absent_at_revision` in the acquisition record and
  nothing is adapted from it.
- **The 21 `charset/*` tests are not acquired.** Their observables are HTTP
  headers, `<meta>` and byte-order-mark bearing HTML documents and linked style
  sheets, which a crate that takes text cannot present. Section 3.2 has no WPT
  backing; its claim rests on the crate's specification-derived tests.
- **No support files.** The tests load only the shared `/resources/testharness.js`
  and `/resources/testharnessreport.js`, which are not acquired and never run.

## Crate-owned adaptations

The JSON files are reviewed literal adaptations, owned by `surgeist-css-parser`
and separate from the imported source and the generator's receipt. Each case
binds to its upstream path, the digest of the raw file, a one-based line and the
literal text of that line. The harness [wpt_syntax.rs](../../wpt_syntax.rs)
verifies every binding with a digest helper before it exercises the crate, and
never derives an expectation from the crate's output: each was written from the
upstream assertion and the pinned edition.

A case claims only the Syntax 3 observable of its upstream test. What the
upstream assertion reads through the CSS object model, selector matching,
property validity or `var()` substitution is not a Syntax 3 result; it is stated
in the case's `remainder`, or the whole case is deferred, and is claimed nowhere.

| Data | Observable and count |
| --- | --- |
| [anb.json](anb.json) | 87 cases (`anb-parsing`, `anb-serialization`): the text must be one `<an+b>` with the upstream-serialized value, or not be one |
| [tokens.json](tokens.json) | 107 cases: the tokens of a text with the exact source text of each, and for unclosed urls the lexical errors |
| [not-whitespace.json](not-whitespace.json) | 26 cases: a code point that is not CSS whitespace makes no whitespace token |
| [units.json](units.json) | 46 cases: the rules and declarations the algorithms of section 5.5 return for a style sheet or a declaration list, with the way each is written back |
| [consecutive-tokens.json](consecutive-tokens.json) | 65 cases: two tokens written one after the other keep a comment between them and read back as the same two tokens |
| [escaped-identifier-round-trips.json](escaped-identifier-round-trips.json) | 11 cases: escaped names decode to the expected tokens and read back as the same tokens once written |
| [unclosed-constructs.json](unclosed-constructs.json) | 2 cases: an unclosed group holds what the closed group holds |
| [deferred.json](deferred.json) | 40 cases, none exercised (below) |

The unit vectors describe what the algorithms return. The crate also reports a
construct that an algorithm drops ("return nothing") as a failed unit so that a
host can say what was dropped; the vectors do not claim those reports, and
describe only accepted units and invalid rule errors, which the algorithms
return. The grammar that decides whether a rule or declaration is valid is an
input of a case (`grammar`), because validity is not a Syntax 3 result.

### Deferred dispositions

| Disposition | Cases | Why |
| --- | --- | --- |
| `adapted-at-step-8a` | 11 | The 10 cases of `input-preprocessing.html` and the lone-surrogate case of `non-ascii-codepoints.html` need section 3.3 preprocessing as a function and UTF-16 input, which the crate's input module adds at step 8a |
| `pending-step-15` | 22 | The cases of `non-ascii-codepoints.html` whose code point is not a non-ASCII ident code point in section 4.2 of the pinned edition. The lexer still applies the 2021 rule until step 15 replaces it with the pinned ranges; the 35 cases whose code point is listed are claimed now |
| `computed-value-substitution` | 7 | The comment and empty-fallback cases of `serialize-consecutive-tokens.html` observe a computed custom property after `var()` substitution |

`non-ascii-codepoints.html` omits the comma after its `[0x203f, 0x2040]` line, so
JavaScript reads the next line as an index into that array and the element is
`undefined`. The first loop of the file then throws at `undefined[0]`, so the
later ranges and every `testInvalid` call, including `0xd800`, never run
upstream at this pin. The lone-surrogate expectation is therefore one upstream
has never exercised. The vectors adapt each range the file lists, as written in
the array.

No case disagrees with the pinned edition for a reason that needs a profile
file. The one possible disagreement, the lone surrogate of `non-ascii-codepoints.html`,
depends on preprocessing and is decided at step 8a.

## Verification and maintenance

Ordinary verification is offline and consumes committed data, from the
repository root:

```sh
cargo test --offline --locked -j 1 -p surgeist-css-parser --test wpt_syntax
```

Verify the raw source/license inventory and receipt with the generator, from the
repository root, with `CORPUS_OWNER` the absolute path of the
`crates/surgeist-css-parser` directory and `CORPUS_ROOT` the absolute path of
this directory:

```sh
cargo run --offline --locked -j 1 -p surgeist-generator --no-default-features --features css-corpus --bin surgeist-css-generate -- --owner-root "$CORPUS_OWNER" --corpus-root "$CORPUS_ROOT" check-corpus
```

For explicitly authorized maintenance, acquire only the reviewed allowlist at
immutable URLs, update the acquisition record and manifest, then use the same
generator command with `import-wpt --source-root "$WPT_SOURCE_BUNDLE"`. Review
raw source/license changes and every affected vector binding together.
Import mutation requires Apple-Silicon macOS; acquisition, import and vector
edits are separate maintenance operations.
