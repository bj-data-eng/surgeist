# Japanese phrase model and inference

Text adapts the plain character-indexed inference of
[Google BudouX 0.9.3](https://github.com/google/budoux/tree/25946bbbb271dc8a53bb5b04082c6859abc5c847),
commit `25946bbbb271dc8a53bb5b04082c6859abc5c847`. The
[private adapter](../../src/phrase.rs) returns raw internal cut
positions. Its shared selector owns admission from the actual validated content
language; each consuming Text phase owns CSS Appendix H admissibility,
contextual ownership and source correspondence.
The model is not an HTML parser, dictionary service or browser-equivalence claim.

## Retained inputs and attribution

The [Japanese model](../../data/budoux/0.9.3/ja.json),
[marked quality rows](../../data/budoux/0.9.3/ja.tsv), and
[full Apache License 2.0](../../licenses/budoux/LICENSE) are unchanged upstream
bytes. Adapted inference and generated tables retain Copyright 2021 Google LLC
and identify their modifications. The upstream
[project homepage](https://github.com/google/budoux) and pinned originals below
identify the source; no package installation, network service or training
corpus is required. Surgeist's own project license remains separate.

| Original at the pinned commit | Bytes | SHA256 | Git blob |
| --- | ---: | --- | --- |
| [budoux/models/ja.json](https://github.com/google/budoux/blob/25946bbbb271dc8a53bb5b04082c6859abc5c847/budoux/models/ja.json) | 20190 | `580db00a5e3e4a5b99b72cdea0db7ea70dac79599ce9d356d16bfdf0d92e7798` | `10914865ae24b402cdfb3330f2f70ffc86631ca4` |
| [tests/quality/ja.tsv](https://github.com/google/budoux/blob/25946bbbb271dc8a53bb5b04082c6859abc5c847/tests/quality/ja.tsv) | 2772 | `eb7ea47fb591447ac7bc0d01ec0e01be58da9903865d032b76b4aa72352e4a01` | `7e5974b79766722cf16421f65fa3f080355bbd29` |
| [LICENSE](https://github.com/google/budoux/blob/25946bbbb271dc8a53bb5b04082c6859abc5c847/LICENSE) | 11357 | `58d1e17ffe5109a7ae296caafcadfdbe6a7d176f0bc4ab01e12a689b0499d8bd` | `7a4a3ea2424c09fbe48d455aed1eaa94d9124835` |

Paired reference sources were qualified separately and are not bundled here:

| Pinned reference | SHA256 | Use |
| --- | --- | --- |
| [budoux/parser.py](https://github.com/google/budoux/blob/25946bbbb271dc8a53bb5b04082c6859abc5c847/budoux/parser.py#L34-L104) | `f5d1d285989cd20071bff486f4e35798a579c229e4ba9f45ee9922b9cda2c9e9` | Constructor and plain inference windows/sign rule |
| [tests/test_parser.py](https://github.com/google/budoux/blob/25946bbbb271dc8a53bb5b04082c6859abc5c847/tests/test_parser.py) | `a726b116ad6b98382f71d470ea38c2832bfc83d57190389f6c1d2bf521a1fcb5` | Independent default-Japanese mission-sentence expectation |
| [README.md](https://github.com/google/budoux/blob/25946bbbb271dc8a53bb5b04082c6859abc5c847/README.md) | `3b4e96b5211a8044e2294ba42314bd3734c1211e6c182d5d9bfdf4616e09d1a7` | Published `今日は▁天気です。` expectation |
| [scripts/README.md](https://github.com/google/budoux/blob/25946bbbb271dc8a53bb5b04082c6859abc5c847/scripts/README.md) | `121856eafc65103cee0805e27d29567b859708304420ce302c4c9edc20c3130e` | Japanese bunsetsu and KNBC/training provenance |

The selected release's distributed model is `ja.json`, not the separate older
`ja_knbc.json`. Upstream describes Japanese bunsetsu training from KNBC plus
issue-specific fine-tuning. The underlying corpus, training inputs and training
software are not redistributed or reproduced here. The model's Apache license
does not establish permission to redistribute that underlying corpus.

## Native inference semantics

The adapter follows the paired Python character positions. Each input value is
one decoded semantic code point: a supplementary scalar is one position, and an
isolated UTF-16 surrogate remains one opaque position with its exact numeric
value. It is not replaced by literal U+FFFD, deleted, truncated to sixteen bits
or expanded into two inference positions. All pinned feature keys contain only
BMP nonsurrogate values, so any complete n-gram containing a supplementary or
isolated value has zero feature weight. Neighboring windows retain its position.
Returned cuts index input positions; the caller maps them through real Text
correspondence to UTF-16. Raw proposals never permit surrogate-pair interiors.

This is an explicit native UA linguistic policy permitted by the pinned
[CSS Text 4 Appendix H](../../../../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#word-phrase-detection).
It does not claim the UTF-16-indexed JavaScript implementation makes identical
predictions near supplementary characters. Four recorded qualification cases
distinguish these indexing policies; unit tests retain the paired-character
results as inference-conformance diagnostics, not Japanese-quality oracles.

Relative to internal cut `i`, UW1–6 inspect `i-3` through `i+2`; BW1–3 inspect
`[i-2,i)`, `[i-1,i+1)`, `[i,i+2)`; TW1–4 inspect `[i-3,i)`, `[i-2,i+1)`,
`[i-1,i+2)`, `[i,i+3)`. Only complete windows contribute. The exact doubled
score is `-3216 + 2 * sum(selected weights)` and a split requires strictly
positive score. Weights fit i16; checked i32 accumulation retains exact signs.
The qualified bounds `[-84136,82152]` cover scores and partial accumulation;
the sum of absolute model weights is 1383462. Overflow is an invalid-data
failure, and allocation failure is a capacity failure, never a no-cut fallback.

Raw inference has no locale argument. The shared native model selector
requires actual validated primary language `ja` with
absent script or `Jpan`; other explicit scripts are initially unsupported.
It never guesses Japanese from glyphs,
region, an undefined language, or neighboring content. That native admission
policy belongs to the shared provider; CSS filtering remains with each
consuming Text phase.

## Generate and verify

From the repository root, using installed Python 3 and no downloaded packages:

```sh
pisct run --timeout-seconds 30 -- python3 -B crates/surgeist-text/tools/automatic-phrase/generate.py
pisct run --timeout-seconds 30 -- python3 -B crates/surgeist-text/tools/automatic-phrase/generate.py --check
pisct run --timeout-seconds 30 -- python3 -B -m unittest discover -s crates/surgeist-text/tools/automatic-phrase -p test_generate.py
```

The [stdlib generator](generate.py) owns
[data.rs](../../src/phrase/data.rs). It pins the exact input
bytes, rejects malformed or ambiguous schema, validates 13 group sizes and
1619 weights, and emits numeric keys in stable lexical order. One-, two- and
three-position keys have distinct fixed array lengths. `--check` compares exact
bytes without rewriting inputs or output. No timestamps, runtime JSON parser,
build-time fetching or automatic build-time generation are involved.

[Generator tests](test_generate.py) verify pinned asset identities, schema
refusals, deterministic output independent of JSON member order, and both fresh
and stale nonmutating checks. Private Rust tests exercise all 41 original marked
quality rows, the published/paired literals, all 13 windows and short guards,
strict sign/overflow behavior and supplementary/isolated-unit policy.
The 41 rows are upstream-authored regression expectations, not the separately
reported 44/44 upstream result, a KNBC holdout measurement, training reproduction
or general Japanese linguistic-quality guarantee. Fresh model outputs must not
be promoted to independently reviewed linguistic expectations.
