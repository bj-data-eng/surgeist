# Surgeist Phrase Ahem test fixture

This is a narrowly mapped derivative of the repository's
[CC0/public-domain Ahem](../ahem/PROVENANCE.md), originally developed by
Todd Fahrner and Myles C. Maxfield. The original source is
<https://www.w3.org/Style/CSS/Test/Fonts/Ahem/>. The unchanged
[COPYING](COPYING) and [UPSTREAM-README](UPSTREAM-README) apply to the source
font; the latter describes the original Ahem, rather than this derivative's
deliberately restricted cmap. No new font or software was downloaded.

The original, stdlib-only [generator](generate.py) requires input
`../ahem/Ahem.ttf` with SHA256
`f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc`.
It replaces the cmap, changes the family to `Surgeist Phrase Ahem` and the
PostScript name to `SurgeistPhraseAhem-Regular`, adjusts OS/2 coverage metadata,
and rebuilds table checksums and `head.checkSumAdjustment`. Upstream legal name
strings, all outlines, glyph metrics and other tables are preserved. Timestamps
are inherited unchanged from the pinned input. Generation never runs at build
time and has no non-stdlib dependencies.

Generated `PhraseAhem.ttf` SHA256:
`89e2e33cb2874df25a62f41d483071c65c42ac8c2fa6ff7e5f75602a8a8091d8`.

The exact cmap is:

| Characters | Codepoints | Actual Ahem glyph | Advance in font units |
| --- | --- | ---: | ---: |
| space | U+0020 | 3 | 1000 |
| A | U+0041 | 35 | 1000 |
| う こ そ へ よ 日 本 | U+3046 U+3053 U+305D U+3078 U+3088 U+65E5 U+672C | 35 | 1000 |

Glyph 35 is Ahem's real square outline `(0,-200)..(1000,800)` at UPEM 1000.
Each selected Japanese scalar has an actual nonzero cmap mapping, rather than
missing-glyph substitution. This is a fixed-advance test font, not a Japanese
typeface or a claim about natural Japanese ink, typography or linguistic quality.
The registered-only Text tests verify actual source ranges and shaped advances
of 10 at size 10, with no host font fallback.

## Independent phrase expectation

BudouX 0.9.3 at commit `25946bbbb271dc8a53bb5b04082c6859abc5c847`
publishes the marked quality row `gh1389`:
[`日本へ▁ようこそ`](../../../../data/budoux/0.9.3/ja.tsv).
The unmarked full input has seven BMP scalars and seven UTF16 units; its
published phrase cut is 3. The full row is an upstream-authored expectation.

The isolated three-scalar prefix `日本へ` requires separate qualification;
the published full row alone does not establish its isolated inference.
The cached original upstream Python `Parser.__init__` and `Parser.parse`
methods, extracted without imports or HTML handling, independently return
`["日本へ"]` for that complete input and `["日本へ", "ようこそ"]` for the full row.
This qualifies the pinned provider's isolated-input behavior, not an additional
published linguistic judgment or a fresh Surgeist inference used as an oracle.
The exact paired reference and model hashes are retained in the
[automatic-phrase source documentation](../../../../tools/automatic-phrase/README.md).
The parser SHA256 is
`f5d1d285989cd20071bff486f4e35798a579c229e4ba9f45ee9922b9cda2c9e9`;
the model SHA256 is
`580db00a5e3e4a5b99b72cdea0db7ea70dac79599ce9d356d16bfdf0d92e7798`.
BudouX's retained model/marked rows and Apache license remain at their existing
paths. No upstream parser code is included in this fixture generator.

## Reproduce

From the repository root:

```sh
pisct run --timeout-seconds 30 -- python3 -B crates/surgeist-text/tests/fixtures/fonts/phrase-ahem/generate.py
pisct run --timeout-seconds 30 -- python3 -B crates/surgeist-text/tests/fixtures/fonts/phrase-ahem/generate.py --check
pisct run --timeout-seconds 30 -- python3 -B -m unittest discover -s crates/surgeist-text/tests/fixtures/fonts/phrase-ahem -p test_generate.py
```

`--check` compares exact bytes and never rewrites the fixture. Generator tests
also check stale nonmutation, changed-input refusal, checksums, cmap/name data
and unchanged outline/metric tables. The owning Rust qualification module is
[phrase_font.rs](../../../../src/tests/phrase_font.rs).
