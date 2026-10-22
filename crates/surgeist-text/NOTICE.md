# surgeist-text

## Dependencies

This notice covers the direct dependencies declared for `surgeist-text`
0.1.0 in [Cargo.toml](Cargo.toml), including the default configuration, the
optional `text-accessibility` feature.
It records source-level dependency attribution, not a resolved binary, release,
or transitive dependency inventory. A dependency declaration does not establish
that its code is redistributed in a particular artifact. Upstream example and
test assets are not bundled except for the Ahem font identified below.

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
