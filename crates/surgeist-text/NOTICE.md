# surgeist-text

## Dependencies

This notice covers the direct dependencies declared for `surgeist-text`
0.1.0 in [Cargo.toml](Cargo.toml), including the default configuration, the
optional `text-accessibility` feature.
It records source-level dependency attribution, not a resolved binary, release,
or transitive dependency inventory. A dependency declaration does not establish
that its code is redistributed in a particular artifact. Upstream example and
test assets are not bundled except for the Ahem font and BudouX marked quality
rows identified below.

The project's own MIT license remains in [LICENSE](LICENSE). Keep this notice
and its accompanying `licenses/` files together.

### accesskit 0.24.1

This product depends on accesskit, distributed by The AccessKit contributors:

* License: [MIT](licenses/accesskit/LICENSE-MIT) or
  [Apache License 2.0](licenses/accesskit/LICENSE-APACHE)
* Homepage: [AccessKit](https://accesskit.dev/)
* Additional material: [source notices](licenses/accesskit/NOTICE.md),
  [Chromium BSD license](licenses/accesskit/LICENSE.chromium), and
  [copyright authors](licenses/accesskit/AUTHORS)

This optional dependency is enabled by `text-accessibility`. AccessKit contains
code derived from Chromium and kurbo; the accompanying source notices retain
those attributions and their applicable license terms.

### BudouX 0.9.3 adapted source and Japanese model

This product includes Japanese phrase inference adapted from Google BudouX,
distributed by Google LLC, and its unchanged Japanese model and marked quality
rows at commit `25946bbbb271dc8a53bb5b04082c6859abc5c847`.
Copyright 2021 Google LLC.

* License: [Apache License 2.0](licenses/budoux/LICENSE)
* Homepage: [BudouX](https://github.com/google/budoux)
* Exact source, data and modifications: [phrase-model guide](tools/automatic-phrase/README.md)

The safe Rust adaptation uses decoded semantic positions, checked integer scoring
and fallible cut storage. Text owns language admission, CSS eligibility, scope
custody and source correspondence. The checked-in table is generated offline from
the retained model. The quality rows are upstream-authored regression fixtures;
they do not establish general linguistic quality. Training data and training
software are not bundled, and the model license does not establish permission
to redistribute that underlying corpus. The project license remains unchanged.

### fontique 0.9.0

This product depends on Fontique, distributed by the Parley Authors:

* License: [Apache License 2.0](licenses/fontique/LICENSE-APACHE) or
  [MIT](licenses/fontique/LICENSE-MIT)
* Homepage: [Fontique](https://github.com/linebender/parley)

Fontique supplies safe catalogue, registration and font selection interfaces.
Its native implementation remains an external dependency.

### harfrust 0.6.2

This product depends on the maintained Harfrust source in this repository:

* License: [MIT](licenses/harfrust/LICENSE)
* Homepage: [Harfrust](https://github.com/harfbuzz/harfrust)
* Additional material: [Unicode license](licenses/harfrust/UNICODE-LICENSE)
* Exact source and modifications: [provider provenance](../../vendor/harfrust/PROVENANCE.md)

Text consumes its safe shaping interface and explicit completion/refusal status.

### hypher 0.1.8

This product depends on Hypher, distributed by Laurenz:

* License: [MIT](licenses/hypher/LICENSE-MIT) or
  [Apache License 2.0](licenses/hypher/LICENSE-APACHE)
* Homepage: [Hypher](https://github.com/typst/hypher)
* American-English data: [copyright and permission notice](licenses/hypher/DATA-NOTICE.txt)

Only the allocation and English features are enabled. The American-English
2005-05-30 patterns and source exceptions carry Gerard D.C. Kuiken's separate
copyright and notice-preserving permission, retained with this distribution.
Text owns checked source correspondence and exception integration; this resource
does not establish other English dialects or languages.

### icu_casemap 2.2.0

This product depends on ICU4X full casing data, distributed by Unicode, Inc.:

* License: [Unicode License 3.0](licenses/icu4x/LICENSE)
* Homepage: [ICU4X](https://icu4x.unicode.org/)

Text reads the safe public baked trie and checked full-mapping exceptions. Its
source-attributed contextual traversal is described below.

### icu_locale 2.2.0

This product depends on ICU4X locale identifiers and compiled likely subtags:

* License: [Unicode License 3.0](licenses/icu4x/LICENSE)
* Homepage: [ICU4X](https://icu4x.unicode.org/)

The extended paired data supplies the native implied-script policy without
overriding an authored explicit script or inventing an unknown language.

### icu_normalizer 2.2.0

This product depends on ICU4X normalization, distributed by the ICU4X Project Developers:

* License: [Unicode License 3.0](licenses/icu4x/LICENSE)
* Homepage: [ICU4X](https://icu4x.unicode.org/)

The owned analyzer uses compiled canonical composition and decomposition data.

### icu_properties 2.2.0

This product depends on ICU4X properties, distributed by the ICU4X Project Developers:

* License: [Unicode License 3.0](licenses/icu4x/LICENSE)
* Homepage: [ICU4X](https://icu4x.unicode.org/)

Source-local whitespace processing uses compiled Unicode bidi-control,
East Asian Width and Script properties.

### icu_provider 2.2.0

This product depends directly on the safe ICU4X data-provider interface:

* License: [Unicode License 3.0](licenses/icu4x/LICENSE)
* Homepage: [ICU4X](https://icu4x.unicode.org/)

It loads the qualified baked casing payload; Text does not decode unsafe wire
representations or accept a mutable provider during finalization.

### icu_segmenter 2.2.0

This product depends on ICU4X segmentation, distributed by the ICU4X Project Developers:

* License: [Unicode License 3.0](licenses/icu4x/LICENSE)
* Homepage: [ICU4X](https://icu4x.unicode.org/)

The Text interaction map uses compiled Unicode grapheme boundary data.

### linebender_resource_handle 0.1.1

This product depends on Linebender resource handles:

* License: [Apache License 2.0](licenses/linebender-resource-handle/LICENSE-APACHE) or
  [MIT](licenses/linebender-resource-handle/LICENSE-MIT)
* Homepage: [Linebender resource handles](https://github.com/linebender/raw_resource_handle)

The catalogue retains registered immutable font bytes through its safe Blob API.

### parlance 0.1.0 and parley_data 0.9.0

This product depends on Parlance and Parley Data, distributed by the Parley Authors:

* License: [Apache License 2.0](licenses/parley/LICENSE-APACHE) or
  [MIT](licenses/parley/LICENSE-MIT)
* Homepage: [Parley](https://github.com/linebender/parley)

Parlance supplies font attributes and settings; Parley Data supplies compiled
Unicode analysis facts. Their exact distributed license texts match the retained
Parley license files. Neither dependency retains the Parley layout intermediary.

### Parley 0.9.0 adapted source

The private owned engine and optional accessibility projection adapt mechanisms
from Parley 0.9.0, distributed by the Parley Authors under the same retained
[Apache-2.0](licenses/parley/LICENSE-APACHE) OR [MIT](licenses/parley/LICENSE-MIT)
licenses. Derived source retains original copyright and SPDX notices. Text owns
the adapted source analysis, font resolution, shaping, immutable result phases,
line finalization, editing and accessibility behavior. The
[owned-engine provenance](docs/owned-engine-provenance.md) records exact upstream
file identities, derived paths and modifications. The
[accessibility provenance](docs/accessibility-provenance.md) identifies that
projection's exact input and modifications.

### skrifa 0.42.1

This product depends on Skrifa from Fontations:

* License: [MIT](licenses/fontations/LICENSE-MIT) or
  [Apache License 2.0](licenses/fontations/LICENSE-APACHE)
* Homepage: [Fontations](https://github.com/googlefonts/fontations)

The catalogue uses its safe font-file interface for whole-file face admission.
Glyph-free metrics use the same-instance metric and variation-data interfaces.
The owned selected-field provenance predicates adapt Skrifa's static selection;
[font metrics provenance](docs/font-metrics-provenance.md) identifies the exact
source and the retained and changed mechanisms.

## Bundled test font

Ahem was developed by Todd Fahrner and Myles C. Maxfield. This repository includes
it for deterministic Text tests, copied from the existing Render fixture:

* License: [public-domain dedication with CC0 fallback](tests/fixtures/fonts/ahem/COPYING)
* Homepage: [Ahem](https://www.w3.org/Style/CSS/Test/Fonts/Ahem/)
* Original description: [upstream README](tests/fixtures/fonts/ahem/UPSTREAM-README)
* Exact file identity and facts: [provenance](tests/fixtures/fonts/ahem/PROVENANCE.md)

The font and license files are unchanged. The fixture is not a production font
or evidence of general script, variation or shaping conformance.

## CSS Text small-kana mapping

The full-size-kana mapping facts in `src/transform.rs` and the independent
literal test vectors derive from Appendix G of CSS Text Module Level 4,
Working Draft 14 August 2026, distributed by the World Wide Web Consortium.
Copyright © 2026 World Wide Web Consortium.

* License: [W3C Software and Document License 2023](licenses/w3c/software-license-2023.txt)
* Original: [CSS Text Module Level 4, Appendix G](https://www.w3.org/TR/2026/WD-css-text-4-20260814/#small-kana)

The table is transcribed into Rust scalar matching and literal expectations;
Surgeist owns execution, original-source correspondence and composed consumers.
The crate's own license remains unchanged. This table does not supply casing or
width-mapping data.

## ICU4X contextual casing adaptation

The Text-owned contextual casing traversal and Greek uppercase helper adapt
`components/casemap/src/internals.rs` and `src/greek_to_me/{mod,data}.rs` from
ICU4X 2.2.0, revision `c9fac4e625ccb2c6a7aa35079fff9709db4385ac`, distributed
by Unicode, Inc. The exact Greek table remains in
`src/transform/greek_data.rs`; the adapted helpers are in
`src/transform/{casing,greek}.rs`.

* License: [Unicode License 3.0](licenses/icu4x/LICENSE)
* Homepage: [ICU4X](https://icu4x.unicode.org/)
* Source: [ICU4X casing internals](https://github.com/unicode-org/icu4x/blob/c9fac4e625ccb2c6a7aa35079fff9709db4385ac/components/casemap/src/internals.rs)

The adaptation reads the paired ICU4X public baked trie and checked exception
decoder. Text traverses immutable original components and retains each actual
emission or removal's correspondence. It does not copy the provider's unsafe
wire representation or reconstruct provenance from a mapped string. ICU4X
locale data supplies native implied-script resolution; the authored language
tag remains unchanged. The crate's own license remains unchanged.

## MathML Core italic mapping

This crate includes an unmodified Appendix C.1 table excerpt from
Mathematical Markup Language (MathML) Core, W3C Candidate Recommendation
Snapshot 24 June 2025, distributed by the World Wide Web Consortium.
[Copyright](https://www.w3.org/policies/#copyright) © 2025
[World Wide Web Consortium](https://www.w3.org/). W3C®
[liability](https://www.w3.org/policies/#Legal_Disclaimer),
[trademark](https://www.w3.org/policies/#W3C_Trademarks) and
[permissive document license](https://www.w3.org/copyright/software-license-2023/)
rules apply.

* License: [W3C Software and Document License, 2023](licenses/w3c/software-license-2023.txt)
* Original: [MathML Core, Appendix C.1](https://www.w3.org/TR/2025/CR-mathml-core-20250624/#italic-mappings)
* Source and adaptation: [mapping guide](tools/math-auto/README.md)

The original HTML table is unchanged. The private generated Rust table adapts
its original/italic columns into sorted numeric pairs; the generator is owned
by Text. Retain this source-specific notice and the full accompanying license
with copied or derived material. The crate's own license remains unchanged.
