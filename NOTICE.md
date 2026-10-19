# Surgeist

[Surgeist](https://github.com/bj-data-eng/surgeist) is licensed under its own
[MIT License](LICENSE). This notice records attribution for the repository source
checkout: facade production path dependencies in [Cargo.toml](Cargo.toml),
the `surgeist-test` support crate, the shared `surgeist-generator` corpus tooling,
the `surgeist-fuzz` verification framework,
and the two direct dependencies of the
[API generator](https://github.com/bj-data-eng/surgeist/blob/0b0c6338c20f77f9dfe743c47914afaf6b98792d/api/generator/Cargo.toml).
Crates under `crates/` are source owned in this repository. The
original consolidation imported the 14 revisions selected by root
[`e0303b14ccd6a81cf3d092105201daf7797ef6aa`](https://github.com/bj-data-eng/surgeist/tree/e0303b14ccd6a81cf3d092105201daf7797ef6aa/crates)
and `surgeist-generator`
[`17f6159a4adb18f0d03cab58f81ad635660e2054`](https://github.com/bj-data-eng/surgeist-generator/tree/17f6159a4adb18f0d03cab58f81ad635660e2054).
The original project homepages and immutable citations below preserve those
source origins; current package facts come from this checkout's manifests.

The checkout also retains attribution for adapted source and bundled test assets
carried by the CSS, layout, and render crates. The copies linked below preserve
their upstream legal text and modification notices. The 14 originally imported
`LICENSE` files contained identical text, preserved in one shared copy here; the root
project's own license remains separate.

The former surgeist-retained package is retired. Its
[last preserved source and notice](https://github.com/bj-data-eng/surgeist/tree/6b6c60b8eda9aac70dce8aca8021d77d238c377e/crates/surgeist-retained)
and [original project homepage](https://github.com/bj-data-eng/surgeist-retained)
retain its consolidation provenance. Canonical live document authoring is owned
by surgeist-dom in this repository.

This source checkout does not contain a linked application. The summary below
does not enumerate every crate's transitive, optional, platform, or compiled
release dependencies. Crate-local notices retain their stated inventory scopes;
the [crate notice index](#crate-notices) locates that material. A compiled release
requires an inventory matching the material, targets, and features actually
distributed. API-generator
dependencies are development tooling, not root facade runtime dependencies;
their entries do not imply that their source or compiled libraries are bundled
with this checkout. Distribute this notice and its accompanying `licenses/`
files together, retaining crate notices, their linked legal material, and source
provenance with redistributed crate source. The local crate links cover the
complete source checkout; a separately packaged subset needs its own valid
notice and license layout.

The checkout also includes the bounded [reference collection](references/README.md):
the retained documents, source witnesses and metadata-only exceptions listed in
its [source catalog](references/SOURCE-CATALOG.md), plus 81 passive SVG figures, an
informative Unicode rule-data companion, and navigation/index material. The
reference entries below cover this added collection only; they do not expand
or re-audit the dependency inventories above. Source notices remain in the
converted documents, and the [conversion report](references/CONVERSION-REPORT.md)
records changes and limitations. The originals remain authoritative.

## Dependencies

### Ahem

The layout and render crates include Ahem test fonts. Layout embeds WOFF2 bytes
retained from Taffy's browser-test support and credits Todd Fahrner and Paul
Nelson through Kozea/Ahem. Render bundles a W3C-distributed TTF and credits Todd
Fahrner and Myles C. Maxfield in the included upstream README.

* License: [Public domain with the upstream Creative Commons Zero fallback](licenses/ahem/COPYING)
* Homepages: [Kozea/Ahem](https://github.com/Kozea/Ahem) and [W3C Ahem](https://www.w3.org/Style/CSS/Test/Fonts/Ahem/)
* Authorship and usage: [W3C upstream README](licenses/ahem/UPSTREAM-README)

Both crates carry byte-identical `COPYING` declarations, shared here. Layout's
[source notice](https://github.com/bj-data-eng/surgeist-layout/blob/b65a18c042655f47d0870f712c7b5e647b5e28a6/NOTICE.md#ahem)
identifies the declaration's Kozea revision; render's
[fixture provenance](https://github.com/bj-data-eng/surgeist-render/blob/e44860e7df530795fef2d423024723d5fa5dc447/tests/fixtures/fonts/ahem/PROVENANCE.md)
records its font and notice hashes. These fonts are test assets.

### Chromium Blink implementation references

The reference collection retains selected Blink source files as readable
implementation evidence for CSSOM rules, text, painting, layout, content parsing, style substitution, container eligibility, animation, accessibility and media/preferences, and bounded
implementation gaps. Each file preserves its original copyright and license notice; the accompanying legal
material identifies the exact retained source scope.

* Licenses and exact covered files: [Chromium reference attribution](licenses/chromium/NOTICE.md)
* Homepage: [Chromium](https://www.chromium.org/)

### CSSTree

The CSS crate includes CSSTree test fixtures and derived neutral expectations
from commit `88e3d965c0b1628642a30a841745b410d6835052`, distributed by Roman Dvornov.

* License: [MIT](licenses/csstree/LICENSE)
* Homepage: [CSSTree](https://github.com/csstree/csstree)

The [pinned corpus README](https://github.com/bj-data-eng/surgeist-css/blob/14ff8f37b2417c5c836eb04986188aadbc9310f1/tests/corpus/csstree/README.md)
records the source tree, import, and transformation provenance. The source
license is included verbatim, retaining the 2016–2026 copyright attribution.

### Fontations source references

The [text contract witness](references/parley-0.9-text-contract-witness.md#supporting-provider-skrifa-0421-metricsrs)
also retains a bounded metric declaration and documentation excerpt from Skrifa
0.42.1 in Fontations. Its package identity, original file hash and covered lines
are recorded separately from the Parley excerpts.

* Licenses: [Apache-2.0](crates/surgeist-render/licenses/fontations/LICENSE-APACHE) OR [MIT](crates/surgeist-render/licenses/fontations/LICENSE-MIT)
* Homepage: [Fontations](https://github.com/googlefonts/fontations)

### html5lib-tests fixtures

The HTML crate bundles selected tree-construction and tokenizer test data and
their upstream formats, license and credits from html5lib-tests revision
`f994590f528ac8b6073665791ddb1ed85c66dfb2`. This is test-only material, not an
upstream parser implementation or runtime dependency.

* License: [MIT](crates/surgeist-html-parser/tests/fixtures/html5lib-f994590f/LICENSE)
* Credits: [complete upstream AUTHORS.rst](crates/surgeist-html-parser/tests/fixtures/html5lib-f994590f/AUTHORS.rst)
* Homepage: [html5lib-tests](https://github.com/html5lib/html5lib-tests)
* Selected source: [pinned revision](https://github.com/html5lib/html5lib-tests/tree/f994590f528ac8b6073665791ddb1ed85c66dfb2)
* Bundled scope: [HTML crate notice](crates/surgeist-html-parser/NOTICE.md#html5lib-tests-fixtures)

The exact included license retains Copyright (c) 2006-2013 James Graham,
Geoffrey Sneddon, and other contributors, with its full permission notice and
disclaimer. Keep that license and credits with redistributed fixture files.

### Parley 0.9.0 source references

The [text contract witness](references/parley-0.9-text-contract-witness.md)
retains bounded excerpts from Parley 0.9.0, distributed by the Parley Authors.
Each excerpt records the original file identity, covered lines and copyright
header. These are reference excerpts, not an ingested production implementation.

* Licenses: [Apache-2.0](crates/surgeist-text/licenses/parley/LICENSE-APACHE) OR [MIT](crates/surgeist-text/licenses/parley/LICENSE-MIT)
* Homepage: [Linebender Parley](https://github.com/linebender/parley)

### public-api 0.50.0

The API generator depends on public-api, distributed by the contributors to
cargo-public-api.

* License: [MIT](licenses/cargo-public-api/LICENSE)
* Homepage: [public-api](https://github.com/cargo-public-api/cargo-public-api/tree/main/public-api)

The exact package's source metadata identifies commit
`05a842ac79b84f44e8fcabaeebfd351c137391b3`. Its
[upstream license](https://github.com/cargo-public-api/cargo-public-api/blob/05a842ac79b84f44e8fcabaeebfd351c137391b3/LICENSE)
is included verbatim and is identical to the selected rustdoc-json license.

### rustdoc-json 0.9.7

The API generator depends on rustdoc-json, distributed by the contributors to
cargo-public-api.

* License: [MIT](licenses/cargo-public-api/LICENSE)
* Homepage: [rustdoc-json](https://github.com/cargo-public-api/cargo-public-api/tree/main/rustdoc-json)

The exact package's source metadata identifies commit
`5e7a38ed6bcfa2a5a4c19c5911d80e00b28d72a6`. Its
[upstream license](https://github.com/cargo-public-api/cargo-public-api/blob/5e7a38ed6bcfa2a5a4c19c5911d80e00b28d72a6/LICENSE)
is included verbatim in the shared cargo-public-api license file.

### surgeist-animation 0.1.0

The root facade depends on surgeist-animation, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-animation](https://github.com/bj-data-eng/surgeist-animation)

### surgeist-color 0.1.0

The root facade depends on surgeist-color, authored in this repository. Its
numerical and finite-formatting implementation was relocated from surgeist-css;
CSS retains authored syntax, exact decimal formatting and source resolution.

* License: [MIT](LICENSE)
* Homepage: [Surgeist](https://github.com/bj-data-eng/surgeist)
* Source attribution: [Color crate notice](crates/surgeist-color/NOTICE.md)

### surgeist-css 0.1.0

The root facade depends on surgeist-css, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-css](https://github.com/bj-data-eng/surgeist-css)

### surgeist-cssom 0.1.0

The root facade depends on surgeist-cssom, authored in this repository.
It owns its integration adapter with the source-owned surgeist-css crate.

* License: [MIT](LICENSE)
* Homepage: [Surgeist](https://github.com/bj-data-eng/surgeist)

### surgeist-dialog 0.1.0

The root facade depends on surgeist-dialog, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-dialog](https://github.com/bj-data-eng/surgeist-dialog)

### surgeist-dom 0.1.0

The root facade depends on surgeist-dom, authored in this repository.
It owns canonical tree identity, atomic mutation and immutable captures.

* License: [MIT](LICENSE)
* Homepage: [Surgeist](https://github.com/bj-data-eng/surgeist)

### surgeist-generator 0.2.0

The source checkout includes surgeist-generator, distributed by bj-data-eng.
It supplies shared CSS corpus tooling and browser-corpus infrastructure, used
by the layout corpus adapter; it is not a dependency of the root facade.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-generator](https://github.com/bj-data-eng/surgeist-generator)
* Direct dependency attribution: [crate notice](crates/surgeist-generator/NOTICE.md) and [license material](crates/surgeist-generator/licenses)

The imported notice covers nine direct dependencies, including optional browser
features and the Apple-Silicon macOS dependency. Preserve its license
alternatives, Rustix exception text, and copyright material with the crate.

### surgeist-layout 0.1.0

The root facade depends on surgeist-layout, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-layout](https://github.com/bj-data-eng/surgeist-layout)

The crate records adaptation from Taffy 0.10.1 and continued use of Taffy layout
fixtures and browser-test support. The [retained adaptation notice](licenses/taffy/NOTICE.md)
records that source lineage; the
[snapshot source notice](https://github.com/bj-data-eng/surgeist-layout/blob/b65a18c042655f47d0870f712c7b5e647b5e28a6/NOTICE.md#taffy)
identifies the incorporated material and fixture pins. The current
[Taffy MIT license](licenses/taffy/LICENSE.md) is included verbatim and retains
the original authors' copyright statement and the 2018 Visly Inc. attribution
for Stretch. Upstream:
[Taffy](https://github.com/DioxusLabs/taffy).

### surgeist-render 0.1.0

The root facade depends on surgeist-render, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-render](https://github.com/bj-data-eng/surgeist-render)

The crate includes private raster-engine source adapted from Vello 0.9.0,
copyright 2020 the Vello Authors. The adapted material retains the
[Apache-2.0](licenses/vello/LICENSES/Vello-0.9.0-APACHE-2.0.txt) OR
[MIT](licenses/vello/LICENSES/Vello-0.9.0-MIT.txt) license alternatives.
The [original Vello provenance and modification notice](licenses/vello/NOTICE-VELLO.md)
is included verbatim with its relative license links intact. Upstream:
[Vello](https://github.com/linebender/vello).

### surgeist-runtime 0.1.0

The root facade depends on surgeist-runtime, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-runtime](https://github.com/bj-data-eng/surgeist-runtime)

### surgeist-shape 0.1.0

The root facade depends on surgeist-shape, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-shape](https://github.com/bj-data-eng/surgeist-shape)

### surgeist-task 0.1.0

The root facade depends on surgeist-task, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-task](https://github.com/bj-data-eng/surgeist-task)

### surgeist-template 0.1.0

The root facade depends on surgeist-template, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-template](https://github.com/bj-data-eng/surgeist-template)

### surgeist-test 0.1.0

The source checkout includes the surgeist-test workspace member,
distributed by bj-data-eng. It supplies shared test infrastructure and fixtures;
it is also an API-audit input and is not a dependency of the root facade.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-test](https://github.com/bj-data-eng/surgeist-test)

### surgeist-text 0.1.0

The root facade depends on surgeist-text, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-text](https://github.com/bj-data-eng/surgeist-text)

### surgeist-window 0.1.0

The root facade depends on surgeist-window, distributed by bj-data-eng.

* License: [MIT](licenses/surgeist/LICENSE)
* Homepage: [surgeist-window](https://github.com/bj-data-eng/surgeist-window)

### Unicode 17 grapheme implementation rule-data companion

The checkout includes Unicode's licensed grapheme rule data from the Unicode 17
final-tag `unicodetools` revision `ecd26be2e82be37609090942c14e518e57e905d8`,
distributed by Unicode, Inc., with separate informative implementation notes.

* License: [Unicode License v3](licenses/unicode/LICENSE)
* Homepage: [Unicode tools](https://github.com/unicode-org/unicodetools)
* Exact source, notices and coverage: [companion attribution](licenses/unicode/NOTICE.md)

The complete upstream copyright and permission text is preserved byte-for-byte.
This entry covers the rule-data companion. The full normative UAX 29 revision 47
report is excluded from public distribution; its catalog entry is metadata only
and normative citations remain upstream.

### Unicode 18 bidirectional character data

`surgeist-html-semantics` includes the complete Unicode 18.0.0 DerivedBidiClass data and
an offline-generated private first-strong classification table, distributed by
Unicode, Inc. The checked-in reference retains the complete data text.

* License: [Unicode License v3](licenses/unicode/18.0.0/LICENSE)
* Homepage: [Unicode Character Database](https://www.unicode.org/Public/18.0.0/ucd/)
* Exact source and coverage: [data attribution](licenses/unicode/18.0.0/NOTICE.md)

The 1991–2026 copyright and permission text is retained separately from the
older Unicode companion's legal material. The normative UAX #9 revision 52
report remains an external citation with metadata only in the public references.

### W3C reference documents and CSSWG source witnesses

The checkout includes reformatted W3C documents and readable generated renderings
of six pinned CSSWG sources as software implementation support, distributed by the World Wide Web
Consortium and the contributors identified in the exact originals. These
source-based references and 81 passive SVG figures are bounded by the
[source catalog](references/SOURCE-CATALOG.md). The collection also preserves a
factual GitHub source-history metadata capture as a supporting provenance
witness, not as a separate specification or copied source-code component. The figures
comprise 53 source-extracted SVGs and 28 railroad diagrams generated from the pinned
CSS Syntax 3 source; the corresponding source grammars and attribution are retained.

* Licenses: [W3C Software and Document License, 2023](licenses/w3c/software-license-2023.txt) and [2015](licenses/w3c/software-license-2015.txt), and [W3C Document License, 2023](licenses/w3c/document-license-2023.txt) and [2015](licenses/w3c/document-license-2015.txt), as identified per document
* Historical legal material: [W3C Document License, 2002](licenses/w3c/document-license-2002.txt)
* Homepages: [W3C technical reports](https://www.w3.org/TR/) and [CSSWG drafts](https://github.com/w3c/csswg-drafts)
* Exact-document copyright statements, applicable licenses and modification notices: [reference attribution](licenses/w3c/NOTICE.md)
* Pinned CSSWG repository declaration: [LICENSE.md](licenses/w3c/csswg-drafts/LICENSE.md)
* Bounded Referrer Policy vocabulary and exact repository declaration: [source and license attribution](licenses/w3c/NOTICE.md#referrer-policy-vocabulary)

The older document-use publications are also available under the 2015 document
license, as expressly stated in the [official W3C licensing announcement](https://www.w3.org/news/2015/w3c-updates-general-document-license/).
Their reformatted copies accompany software under its implementation-support
exception and retain the required derived-material notices. They are not
published for use as new technical specifications. Original copyright statements,
authorship and status remain intact; no endorsement is implied.

### WebKit implementation references

The reference collection retains selected WebKit source files as readable
implementation evidence for CSSOM, text, effects, layout, content parsing, style, transitions, contextual lengths, animation, accessibility, intrinsic color analysis, native Document focus initialization and focus-algorithm source interpretation. The
[contrast-color witness](references/webkit-contrast-color--73aa6c89e2cb--6592a4862701.md)
preserves its [file-specific BSD notices](licenses/webkit/NOTICE.md#contrast-color-source-witnesses). These are documentation copies,
not compiled or linked dependencies. Each retained file preserves its original
copyright, redistribution conditions and disclaimer.

* Licenses and exact covered files: [WebKit reference attribution](licenses/webkit/NOTICE.md)
* Homepage: [WebKit](https://webkit.org/)

### Web Platform Tests

The layout crate includes subgrid and grid-lanes browser fixtures adapted from
Web Platform Tests (WPT), distributed by the web-platform-tests contributors,
and generated expectations derived from those fixtures.

The CSS crate also includes an explicit CSS source subset at revision
`ddcca5943fd41232d42149aaa19d9a04c5651b18` and reviewed data adaptations. Its
[corpus guide](crates/surgeist-css/tests/corpus/wpt/README.md) and
[acquisition record](crates/surgeist-css/tests/corpus/wpt/acquisition.json)
identify the selected files and retained authorship. This subset uses the
preserved [BSD-3-Clause license](crates/surgeist-css/tests/corpus/wpt/source/LICENSE.md).

* License: [BSD-3-Clause](licenses/web-platform-tests/LICENSE.md), with [CC0-1.0](licenses/web-platform-tests/legalcode.txt) for individually dedicated source material
* Homepage: [Web Platform Tests](https://web-platform-tests.org)
* Source authors and individual dedications: [upstream notices](licenses/web-platform-tests/NOTICE.md)

The [snapshot source notice](https://github.com/bj-data-eng/surgeist-layout/blob/b65a18c042655f47d0870f712c7b5e647b5e28a6/NOTICE.md#web-platform-tests)
records revision `f01d00b6963a8374784e6aadc67e608b100069d0` for pinned source
URLs. Some adapted fixtures record only an upstream path, so their exact
adaptation revision remains unrecorded. The included license and source notices
were checked by the leaf against the named files at the pinned revision.

### WHATWG standards

The checkout includes the Fullscreen specification snapshot at
`7c38d773117aa1e6bfa13754afe77483f40c908f`, distributed by WHATWG
(Apple, Google, Mozilla, Microsoft), as reformatted implementation reference text.
The collection also retains bounded DOM, HTML, Web IDL, Infra and MIME Sniffing
captures with their original notices and independently hashed source scope.

* License: [Creative Commons Attribution 4.0 International, with the upstream BSD 3-Clause provision for portions incorporated into source code](licenses/whatwg/LICENSE)
* Homepages: [WHATWG Fullscreen](https://fullscreen.spec.whatwg.org/), [DOM](https://dom.spec.whatwg.org/), [HTML](https://html.spec.whatwg.org/), [Web IDL](https://webidl.spec.whatwg.org/), [Infra](https://infra.spec.whatwg.org/) and [MIME Sniffing](https://mimesniff.spec.whatwg.org/)
* Exact revision, copyright statement and changes: [reference attribution](licenses/whatwg/NOTICE.md)

The complete selected upstream LICENSE is preserved byte-for-byte. Its BSD
provision applies to the stated source-code scope; the Markdown specification
text is covered by Creative Commons Attribution 4.0 International.

The [bounded JavaScript MIME essence definitions](references/mimesniff-javascript-essence--39aa53511b13.md)
retain the 16-string vocabulary and matching hook from the
[MIME Sniffing project](https://github.com/whatwg/mimesniff), revision
`39aa53511b13953d84fef8d4131d6f61d0ccbde6`; the private script adaptation
incorporates those strings under the BSD source-code provision. The
[complete microtask checkpoint](references/html-microtask-checkpoint--877d6146487f.md)
adds a bounded representation of the existing HTML877d edition, including its
external cleanup/timing hooks. The
[source notice](licenses/whatwg/NOTICE.md#javascript-mime-essence-and-html-checkpoint--2026-10-11)
records these conversions and their exact original hashes. They introduce no
MIME parser/sniffer or browser event-loop implementation; the selected
standalone absent-runtime-domain reduction is an explicit architecture
capability specialization.

The HTML crate also incorporates the selected edition's common microsyntax
algorithms and element and attribute indexes under that BSD provision; see the
[common microsyntax notice](licenses/whatwg/NOTICE.md#html-common-microsyntax-incorporation--2026-10-11)
and [vocabulary adaptation notice](licenses/whatwg/NOTICE.md#html-vocabulary-incorporation--2026-10-11).
The intrinsic iframe sandbox operation incorporates the selected edition's
keyword and local directive rules under the same code provision; the
[sandbox source notice](licenses/whatwg/NOTICE.md#html-iframe-sandbox-directive--2026-10-11)
records its separate prose conversion and excludes active-policy enforcement.

## Crate Notices

These local records accompany the imported source and describe each crate's
own attribution scope. Keep their relative license paths and bundled-asset
provenance intact. Their direct-dependency inventories do not imply a complete
transitive binary-release inventory.

| Crate | Local attribution |
| --- | --- |
| surgeist-animation | [NOTICE.md](crates/surgeist-animation/NOTICE.md) |
| surgeist-color | [NOTICE.md](crates/surgeist-color/NOTICE.md) |
| surgeist-css | [NOTICE.md](crates/surgeist-css/NOTICE.md) |
| surgeist-cssom | [MIT License](LICENSE); authored in this repository |
| surgeist-dialog | [LICENSE](crates/surgeist-dialog/LICENSE); no separate notice |
| surgeist-dom | [MIT License](LICENSE); authored in this repository |
| surgeist-fuzz | [NOTICE.md](crates/surgeist-fuzz/NOTICE.md) |
| surgeist-generator | [NOTICE.md](crates/surgeist-generator/NOTICE.md) |
| surgeist-html | [NOTICE.md](crates/surgeist-html/NOTICE.md) |
| surgeist-html-parser | [NOTICE.md](crates/surgeist-html-parser/NOTICE.md) |
| surgeist-html-semantics | [NOTICE.md](crates/surgeist-html-semantics/NOTICE.md) |
| surgeist-layout | [NOTICE.md](crates/surgeist-layout/NOTICE.md) |
| surgeist-render | [NOTICE.md](crates/surgeist-render/NOTICE.md) |
| surgeist-runtime | [NOTICE.md](crates/surgeist-runtime/NOTICE.md) |
| surgeist-shape | [NOTICE.md](crates/surgeist-shape/NOTICE.md) |
| surgeist-task | [NOTICE.md](crates/surgeist-task/NOTICE.md) |
| surgeist-template | [NOTICE.md](crates/surgeist-template/NOTICE.md) |
| surgeist-test | [NOTICE.md](crates/surgeist-test/NOTICE.md) |
| surgeist-text | [NOTICE.md](crates/surgeist-text/NOTICE.md) |
| surgeist-window | [NOTICE.md](crates/surgeist-window/NOTICE.md) |

## HTML parser research references

Pinned source excerpts and selected complete files are retained solely as research references. [html5ever](https://github.com/servo/html5ever) is dual-licensed MIT OR Apache-2.0; see its [local attribution and licenses](licenses/html5ever/NOTICE.md). [html5gum](https://github.com/untitaker/html5gum) is MIT-licensed; see its [local attribution and license](licenses/html5gum/NOTICE.md). This capture adds no production dependency.

Separately, `surgeist-html-parser` incorporates adapted html5ever tokenizer
source in its private production parser machinery. The
[origin record](crates/surgeist-html-parser/PARSER-ORIGINS.md) identifies the selected
revision, exact original files and modifications. The retained
[MIT OR Apache-2.0 notice and legal material](licenses/html5ever/NOTICE.md#incorporated-tokenizer-source)
cover this adaptation; it adds no external parser Cargo dependency.

The same private machinery also incorporates bounded tree-insertion and scope
operations, an ordinary-document Initial/BeforeHtml prologue and doctype tables
under the [tree-source notice](licenses/html5ever/NOTICE.md#incorporated-tree-insertion-source).
Its origin record preserves a separate initial adaptation checkpoint and the
complete original notices. The crate README and conformance matrix qualify the
supported public parsing profiles; this attribution certifies no full browser
conformance.

## Selector matching research references

Pinned [Servo selectors](https://github.com/servo/stylo/tree/64416a22f9d06d07e108176d724dd20509a8c52d/selectors) source is retained as research material under MPL-2.0. See [local attribution and license](licenses/servo-selectors/NOTICE.md). This capture adds no production dependency.

The [pinned HTML language sources](references/SOURCE-EDITIONS.md#pinned-html-language-baseline) retain complete selected chapters from an immutable WHATWG HTML snapshot, with [WHATWG attribution and exact license](licenses/whatwg/NOTICE.md#pinned-html-language-snapshot--2026-10-11). The [supplemental html5ever reference](references/html5ever-parser-supplemental-contracts--d7232d746d31.md) retains bounded upstream code excerpts under the existing [MIT OR Apache-2.0 attribution](licenses/html5ever/NOTICE.md). These additions select source evidence and do not change Surgeist’s license or add a production dependency.

## HTML-delegated editability material

The bounded [execCommand definitions](references/execcommand-editability--85de9ff4a981.md)
retain the original document notice and immutable w3c/editing source identity,
with the exact [repository license declaration](licenses/w3c/editing/LICENSE.md)
and [W3C Software and Document License](licenses/w3c/software-license-2023.txt).
The [attribution and modification notice](licenses/w3c/NOTICE.md#html-delegated-editability-definitions)
identifies the selected paragraphs and Markdown conversion. HTML retains its
separate WHATWG behavioral authority; no editing-command runtime is incorporated.

The [nullable active-range reference](references/designmode-active-range--877d6146487f.md)
adds the delegated active-range paragraph under those same W3C notices, and DOM
boundary-point/live-range definitions under the existing
[WHATWG license and attribution](licenses/whatwg/NOTICE.md#designmode-range-definitions--2026-10-16).
Its WebKit section contains behavioral metadata only.

## Initial native Permissions Policy source

The [bounded Permissions Policy creation reference](references/permissions-policy-native-creation--29398046f1e0.md)
contains material derived from W3C Permissions Policy commit
`29398046f1e0cb45c091adda853df74925dc740a`, with the original source identity,
copyright and [W3C attribution and modification notice](licenses/w3c/NOTICE.md#initial-permissions-policy-creation).
The existing complete [W3C Software and Document License, 2015](licenses/w3c/software-license-2015.txt)
accompanies that material. HTML opaque-origin and focus-feature definitions retain their separate
[WHATWG license and attribution](licenses/whatwg/NOTICE.md#permissions-policy-opaque-origin-definitions--2026-10-16).
This is implementation-support source material, not a complete policy engine.

## Native hosted lifecycle and clock sources

The [navigable lifecycle](references/html-hosted-navigable-lifecycle--877d6146487f--3a9658155866.md),
[initial Document/global/agent/settings clauses](references/html-hosted-global-agent-settings--877d6146487f--03a314e0aa55.md)
and [initial teardown clauses](references/html-hosted-initial-teardown--877d6146487f--c2dd632ac56d.md)
contain bounded extracts of the existing immutable WHATWG HTML edition, under
its [license and modification notice](licenses/whatwg/NOTICE.md#native-hosted-lifecycle-and-global-clauses--2026-10-16).
Literal HTML source structure and JavaScript-only source steps remain visible;
the separate native execution projection does not modify those source steps.

The [High Resolution Time clauses](references/hr-time-native-clock--1f0b9faf793d--5f2276cce0af.md)
contain material derived from W3C commit
`1f0b9faf793db3b06c8e85462448e8a812d00fb9`. The complete existing
[W3C Software and Document License 2023](licenses/w3c/software-license-2023.txt)
and [attribution and modification notice](licenses/w3c/NOTICE.md#native-hosted-high-resolution-time-clauses)
accompany the literal bounded source. These references support implementation;
they do not certify a complete browser environment or change this project's license.

## Child navigable history and traversal source

The [child history/traversal clauses](references/html-child-navigable-history--877d6146487f--a9f635b298df.md)
contain bounded extracts from the existing immutable WHATWG HTML edition.
The [complete license](licenses/whatwg/LICENSE) and
[attribution and modification notice](licenses/whatwg/NOTICE.md#child-navigable-history-and-traversal-clauses--2026-10-16)
accompany the literal source. Editorial native qualifications remain distinct
from the unchanged algorithms; this reference does not establish a general
navigation/history service, transitive source closure or browser conformance.

## Legacy Frame sandbox metadata

The [Frame sandbox witness](references/webkit-frame-sandbox--73aa6c89e2cb-metadata.md) retains immutable
WebKit file/blob/line provenance and bounded behavioral summaries, without
transcribing browser source or ingesting an implementation. Original copyright
and license information remains in the exact linked upstream files and
[WebKit license inventory](https://webkit.org/licensing-webkit/).
The [source qualification](references/SOURCE-EDITIONS.md#legacy-frame-creation-sandbox-contribution)
is a bounded #1124 decision; it does not adopt a browser loader or certify
native child lifecycle completion.

## Child history and destruction metadata

The [child lifetime source record](references/html-child-history-lifetime--pinned-browser-metadata.md)
retains pinned WebKit/Chromium URLs, Git-blob identities, line locators and
behavioral summaries. It contains no copied browser implementation or discussion
transcript. Original copyright and license provenance remains in those exact
upstream files and linked project license inventories. Dated current-standard
and issue observations do not change the license of the retained HTML source
or certify a native lifetime policy or implementation.

## Initial opener-policy source

The [opener-policy values and default record](references/html-opener-policy-defaults--877d6146487f--d124556cde21.md)
contain bounded material from the existing immutable WHATWG HTML edition.
The [complete WHATWG license](licenses/whatwg/LICENSE) and
[attribution and modification notice](licenses/whatwg/NOTICE.md#initial-opener-policy-values-and-record--2026-10-16)
accompany the literal source. This supplies language definitions and default
record evidence; it does not establish child lifecycle completion, header
processing, COOP enforcement or browser conformance.

## Native hosted random UUID source

The [random UUID generation clauses](references/webcrypto-random-uuid--811c24c69eb2--1c697c6b1d48.md)
contain bounded material derived from W3C Web Cryptography commit
`811c24c69eb22d477af5f1678cb70dbc611c7c40`, supporting the existing
[HTML session-history-entry caller](references/html-hosted-navigable-lifecycle--877d6146487f--3a9658155866.md#complete-session-history-entry-items-and-document-getter).
The complete [W3C Document License 2023](licenses/w3c/document-license-2023.txt)
applies to the prose; the marked Web IDL is separately covered by its Code
Components provision under the [W3C Software and Document License](licenses/w3c/software-license-2023.txt).
The [attribution and modification notice](licenses/w3c/NOTICE.md#native-hosted-random-uuid-clauses)
identifies the exact source and literal extraction. This source reference
does not claim full Web Crypto, JavaScript execution or a newer UUID RFC,
and does not change Surgeist's license.

## HTML-delegated SVG script material

The [bounded SVG Tiny 1.2 reference](references/svg-tiny12-script-processing--REC-20081222--2f36eb76440c.md) supplies HTML-delegated script
processing clauses with the original Recommendation notice and status.
The [W3C attribution](licenses/w3c/NOTICE.md#html-delegated-svg-script-processing)
and existing complete local document licenses accompany the software-support
Markdown conversion and the disclosed bounded private Rust algorithm adaptation.
The restricted operation preserves Tiny’s processed/connectivity guard and
connected processed mark while separate no-loader and Disabled capabilities
suppress acquisition and execution. It is not literal Tiny Disabled behavior or
an SVG runtime. The
[crate notice](crates/surgeist-html-parser/NOTICE.md#restricted-svg-script-processing)
and [origin record](crates/surgeist-html-parser/PARSER-ORIGINS.md#restricted-svg-script-extension)
distinguish the HTML wrapper, Tiny incorporation and metadata-only WebKit clone
selection. No browser source excerpt or new dependency is incorporated.

## HTML-delegated Fetch vocabulary

The bounded [Fetch script-like destination definition](references/fetch-script-like-destinations--e9460d1b22ea.md)
supplies the finite vocabulary delegated by HTML link destinations. The
[WHATWG attribution](licenses/whatwg/NOTICE.md#html-delegated-fetch-destinations--2026-10-12)
and complete local license cover the Markdown extraction. This source selection
incorporates no Fetch runtime or new dependency.

## DOM-delegated XML pseudo-attribute grammar

The bounded [XML-stylesheet parsing reference](references/xml-stylesheet-pseudo-attributes--REC-xml-stylesheet-20101028--a0e1a5198a21.md) and
[XML grammar closure](references/xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md) retain the actual 2010/2008
Recommendation titles, editors, original copyright and complete STATUS.
The [source-specific attribution](licenses/w3c/NOTICE.md#dom-delegated-xml-pseudo-attribute-grammar)
and complete local [2002](licenses/w3c/document-license-2002.txt) and
[2015](licenses/w3c/document-license-2015.txt) document terms accompany these
implementation-support conversions, including separate required 2015 derivative
notices. The official 2015 update applies to existing publications. These are
not replacement technical specifications, a complete XML parser, or evidence
of stylesheet loading; no software dependency or project license changes.

## HTML-delegated ECMAScript Number serialization reference

The bounded [ECMAScript 2026 Number serialization reference](references/ecmascript-number-serialization--2026--cd65f2eaa201.md)
includes official Ecma International material under the
[complete original copyright/license/disclaimer and software terms](licenses/ecma262/LICENSE-2026.txt).
Its [source and modification notice](licenses/ecma262/NOTICE.md) identifies
the dated edition, copied portions and Markdown changes. No JavaScript runtime,
dtoa code, dependency or product implementation is added by this reference.

## Native DOM and HTML event references

The bounded [DOM event reference](references/dom-native-events--3071e5f26944--332564d9519c.md)
and [HTML ToggleEvent reference](references/html-toggle-event--877d6146487f--eeb465762458.md)
reuse the existing WHATWG source editions and
[complete license and attribution](licenses/whatwg/NOTICE.md#native-event-dispatch-sources--2026-10-15).
They retain original headers and intellectual-property notices with disclosed
Markdown conversion changes. Specification prose uses CC BY 4.0; BSD 3-Clause
applies only to portions incorporated into code. No project license, dependency
or JavaScript-runtime change follows from these reference additions.

## HTML Document policy-container initial state

The bounded [initial-state reference](references/html-policy-container-initial-state--877d6146487f.md)
reuses the selected HTML snapshot under the existing
[WHATWG notice and complete license](licenses/whatwg/NOTICE.md#document-policy-container-initial-state--2026-10-16).
It preserves Document and policy-container defaults needed by native Meta work.
The disclosed Markdown conversion adds no implementation or dependency.

## Hosted same-site definitions

The [bounded HTML same-site reference](references/html-hosted-same-site--877d6146487f--d13a3b5702d5.md)
reuses the already selected HTML877d edition and unchanged complete WHATWG
license. Its [attribution and modification notice](licenses/whatwg/NOTICE.md#hosted-same-site-definitions--2026-10-16)
records the complete selected site/origin comparison units and offline
extraction. Specification prose uses CC BY 4.0; the BSD 3-Clause provision
applies only to portions incorporated into code. Project licensing is unchanged.

## URL equality and serialization source

The [bounded URL reference](references/url-equality-serialization--fde3f74f0633.md)
retains selected complete algorithms and supporting definitions from WHATWG URL
Standard revision `fde3f74f063341a28d437216f75735b8b40128f5`. The
[attribution and modification notice](licenses/whatwg/NOTICE.md#url-equality-and-serialization--2026-10-16)
and unchanged complete local license cover this reference. Specification prose
uses CC BY 4.0; BSD 3-Clause applies only to portions incorporated into code.
This selection adds no URL parser, dependency, JavaScript runtime or change to
Surgeist's own license.

## Close-watcher cancellation source

The [bounded HTML cancellation reference](references/html-close-watcher-cancel-result--877d6146487f.md)
retains selected clauses from the existing immutable HTML877d edition under the
[WHATWG attribution and modification notice](licenses/whatwg/NOTICE.md#close-watcher-cancel-result--2026-10-17)
and unchanged complete local license. Specification prose uses CC BY 4.0;
source-code incorporation uses the separate BSD 3-Clause provision. The literal
source and its separately linked implementation determination remain distinct.

The [Window manager reference](references/html-close-watcher-manager--877d6146487f.md)
retains the same edition's complete selected defaults, watcher definition,
establishment and destruction units as inert literal source. The existing full
license and [modification notice](licenses/whatwg/NOTICE.md#close-watcher-manager--2026-10-17)
cover its bounded selection and Markdown framing.

## Dialog modal assertion comparison

The [dated HTML comparison](references/html-dialog-modal-assertion-comparison--2026-10-17.md)
retains selected current-source clauses separately from the pinned implementation
edition. The [WHATWG attribution and modification notice](licenses/whatwg/NOTICE.md#dialog-modal-assertion-comparison--2026-10-17)
and unchanged complete license cover its Markdown transcription under CC BY 4.0.
It supplies no upstream correction, replacement standard or implementation acceptance.

## Native Range association source

The [bounded Selection API reference](references/selection-api-range-association--20260611.md)
retains complete selected clauses from the W3C Working Draft of 11 June 2026.
Its [attribution and modification notice](licenses/w3c/NOTICE.md#native-range-association-source)
and the existing complete [W3C Software and Document License 2023](licenses/w3c/software-license-2023.txt)
accompany the conversion. The source selection supports #1243's bounded native
association contract without claiming a complete Selection implementation or
changing Surgeist's license.
