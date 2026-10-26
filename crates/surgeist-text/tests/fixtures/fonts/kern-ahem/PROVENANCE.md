# Surgeist Kern Ahem test fixture

This is an original derivative of the repository's retained
[Ahem](../ahem/PROVENANCE.md), developed by Todd Fahrner and Myles C. Maxfield.
The upstream homepage is <https://www.w3.org/Style/CSS/Test/Fonts/Ahem/>.
The unchanged [COPYING](COPYING) and [UPSTREAM-README](UPSTREAM-README)
retain its public-domain declaration with CC0 fallback. The original font and
the separate Phrase Ahem fixture remain unchanged. No font, tool or dependency
was downloaded for this derivative.

The original, standard-library-only [generator](generate.py) uses the owned
Phrase Ahem generator's SFNT assembly mechanics. Its full-cmap preservation,
one-pair `kern` construction and all-platform identity renaming are local code.
No upstream shaping implementation is copied into the generator.

Input `../ahem/Ahem.ttf` SHA-256:
`f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc`.

Generated `KernAhem.ttf` SHA-256:
`51fb4d3bc99025f0a98b2b080101bd9deec0b2aa8a084d3fac91f03e522af4f0`
(23,068 bytes).

The family is **Surgeist Kern Ahem**; the PostScript name is
**SurgeistKernAhem-Regular**. All original platform identity records are renamed.
Every other name record, including upstream legal/description strings, retains
its original bytes and encoding. The complete original cmap, outlines, hmtx and
other metric tables are unchanged. The only changes are identity names, the new
`kern` table, the SFNT directory/checksums and `head.checkSumAdjustment`.
Original timestamps remain unchanged. Generation is explicit maintenance and
never runs at build time.

## Exact kerning contract

The retained Unicode cmap maps U+0041 A to glyph35 and U+002E period to glyph16.
Both have unscaled advance1000 at UPEM1000. The sole added `kern` table is
OpenType version0 with one horizontal format0 subtable and one sorted pair:
**(35,16,-200)**. Minimum, cross-stream and override flags are absent. Neither
GPOS nor `kerx` is present, so there is no alternate positioning table competing
with this pair in the fixture.

The pinned Harfrust horizontal legacy-kern implementation splits this adjustment
between both glyph advances and adds the second part to the second glyph's
x offset. At size10, the prospective source-derived result is9+9 with a-1
period offset. The [native Text qualification](../../../../src/tests/kern_font.rs)
executes current prepared, selected, finalized and placed public interfaces and
independently confirms:

| Feature setting | A advance | Period advance | Period pen | Period x offset | Period draw x | Raw/fit advance |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Default kerning | 9 | 9 | 9 | -1 | 8 | 18 |
| Explicit `"kern" off` | 10 | 10 | 10 | 0 | 10 | 20 |

Both original and derivative are registered in a registered-only catalogue.
Tests request this derivative's distinct family and verify retained font bytes,
face0, the actual10px instance, glyph IDs, original UTF16 source ranges and raw
provider offsets separately from finalized logical pens. Each actual TCU keeps
its9-unit contribution (10 with kerning disabled), original-origin association
and issued-unit custody. The parent's source caret between A and period remains
at9 (10 with kerning disabled), independently of the period's draw origin8.
Source identity, original/processed text, selection and plaintext remain intact.

This is a deterministic font/provider fixture, not a natural punctuation
typeface, browser-equivalence assertion, hanging-punctuation implementation or
claim about another feature's line-fit policy. No hanging behavior is simulated.

## Reproduce

From the repository root, using installed Python3 and the command supervisor:

```sh
pisct run --timeout-seconds 60 -- python3 -B crates/surgeist-text/tests/fixtures/fonts/kern-ahem/generate.py
pisct run --timeout-seconds 60 -- python3 -B crates/surgeist-text/tests/fixtures/fonts/kern-ahem/generate.py --check
pisct run --timeout-seconds 60 -- python3 -B -m unittest discover -s crates/surgeist-text/tests/fixtures/fonts/kern-ahem -p test_generate.py
pisct run --timeout-seconds 600 -- cargo test --offline --locked -j 1 -p surgeist-text --lib tests::kern_font::
```

The input hash is checked before output writes. `--check` never modifies
fresh/stale output or creates missing output. The ten
[generator tests](test_generate.py) independently parse the wire directory,
checksums, exact pair, full cmap and name records; check preserved tables/legal
files, deterministic bytes and changed-input refusal; and exercise generation
and fresh/stale/missing nonmutating checks. Scratch files are scoped temporary
directories removed by their contexts. `-B` avoids Python bytecode residue.
