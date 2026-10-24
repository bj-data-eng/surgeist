# MathML Core math-auto mapping data

[generate.py](generate.py) is a Text-owned Python-standard-library generator.
It reads the exact retained [C.1 table](../../data/MathMLCore-20250624-italic-mappings.html)
and writes [math_data.rs](../../src/transform/math_data.rs). It performs no
network access and runs no upstream code. Paths resolve from the script, so the
working directory does not affect the result.

```sh
python3 crates/surgeist-text/tools/math-auto/generate.py
python3 crates/surgeist-text/tools/math-auto/generate.py --check
```

`--check` compares exact output bytes and never rewrites files. Missing, stale,
malformed or differently pinned inputs/outputs fail with a nonzero exit.
Generation validates the explicit HTML table/header/row/cell structure, literal
characters and declared scalar values, code-point deltas, 112 distinct BMP
originals and distinct scalar targets, exact raw input checksum, and directed
pair checksum. The generated relation is sorted by original code point.

## Source and license

This material is copied from or derived from
[Mathematical Markup Language (MathML) Core, W3C Candidate Recommendation Snapshot,
24 June 2025](https://www.w3.org/TR/2025/CR-mathml-core-20250624/),
[Appendix C.1](https://www.w3.org/TR/2025/CR-mathml-core-20250624/#italic-mappings).
Copyright © 2025 World Wide Web Consortium. The
[W3C Software and Document Notice and License, 2023](../../licenses/w3c/software-license-2023.txt)
applies; [official license](https://www.w3.org/copyright/software-license-2023/).
Retain this source-specific notice and the full license with derived material.

The HTML input is an **unmodified excerpt**, starting at its original `<table>`
and ending at `</table>`, with no inserted header or newline. The generated Rust
is an adaptation of the original/italic columns into sorted numeric pairs.
The owning generator is newly authored; no upstream executable was copied.

| Material | Bytes | SHA256 |
| --- | ---: | --- |
| Complete official CR HTML, hashed before extraction | 1,127,295 | `f61ebea74ba54c489646e6969983ef977f448c26602edff75ec4e3ba77216373` |
| Exact retained C.1 table HTML | 12,037 | `cdf7adc6b280864a1080ae523209bb285d9d13baf4ae3c1014eacb09eb7d9ab7` |
| Directed pairs, sorted `INPUT;OUTPUT\n`, uppercase hex with at least four digits | — | `29d715b30b4186423707df44b0552eeef6100672bee18f76a64597433fbe6604` |

The existing full local license text has SHA256
`bd0e4ada87e6f70c3daceada3401c2f018e3333600b5a50fd57d3da6cea82da4`.
Its License and Disclaimers text was compared with the official 2023 license;
HTML and plaintext are different byte representations.

The relation has 112 unique targets: the specified mathematical italic scalars
and U+210E for italic h. It is not a general inverse of Unicode font
decompositions, normalization, or every mathematical alphabet. Normative
[MathML Core §4.2](https://www.w3.org/TR/2025/CR-mathml-core-20250624/#math-auto-transform)
references these columns even though Appendix C is labelled non-normative.
This artifact provides mapping data only. Complete-original-node eligibility,
post-Phase-I execution and source correspondence remain with Text's owning
transform implementation and contract; the generator does not implement them.
