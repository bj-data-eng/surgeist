# surgeist-html-parser

Surgeist-authored code uses the repository [MIT License](../../LICENSE).

The selected HTML algorithms and source adaptations retain the [WHATWG notice](../../licenses/whatwg/NOTICE.md).

## Adapted parser source

The private production tokenizer adapts four source files from
[html5ever](https://github.com/servo/html5ever), revision
`d7232d746d3112f08926d169591576bd12fe2fdf`. It preserves the upstream file
notices and the choice of [MIT](../../licenses/html5ever/LICENSE-MIT) or
[Apache-2.0](../../licenses/html5ever/LICENSE-APACHE), with exact
[copyright](../../licenses/html5ever/COPYRIGHT) and
[authorship](../../licenses/html5ever/AUTHORS) material. The
[origin record](PARSER-ORIGINS.md) identifies the incorporated files and their
modifications. This source adaptation adds no Cargo dependency on html5ever,
markup5ever, Tendril or web_atoms. The Surgeist-owned public request/report
boundary exposes decoded-Unicode ordinary document and qualified contextual
fragment parsing; the tokenizer remains private implementation material.

The private tree-insertion kernel additionally adapts bounded operations and
scope lists from two files at that same revision. Its Initial/BeforeHtml
prologue and doctype tables adapt two additional files; the owning head-stage
driver further adapts selected head, noscript and Text rules, including script
insertion and EOF/end-tag dispatch for the fresh Disabled operation.
The same source files also supply selected ordinary body and trailing-mode rows,
generic end handling and implied-end/special-element lists, followed by active
formatting, Noah comparison, reconstruction, adoption and applet/marquee/object
markers. Selected button, fieldset and object rows additionally use the same
source files and notices; their exact source ranges and association adaptations
are recorded in the origin record.
Ordinary frameset entry, InFrameset and its trailing modes additionally adapt
the same rules file; the new private frameset module retains its complete
original notice and records its source ranges in the origin record.
Ordinary textarea entry also adapts the same rules file, lines 874–878, with
its original notice retained; controlled raw/reset providers follow the selected
HTML and DOM clauses identified in the origin record.
Ordinary template start/end and InTemplate rules additionally adapt
tree_builder/rules.rs lines 257–307 and 1403–1462, with insertion routing/reset
and shadow-permission factoring from tree_builder/mod.rs lines 417–450,
1265–1313 and 1463–1505. The new src/parser/tree_builder/template.rs retains
the complete original eight-line notice under the same MIT OR Apache-2.0 terms.
Session template/PI state, adoption/cloning and controlled patching are
Surgeist-owned implementations, not additional copied html5ever modules.
Surgeist-owned
controlled providers retain actual text and recovered-tag ownership.
Their original notices are
retained verbatim, with separate adaptation notes and origin hashes. The same
MIT OR Apache-2.0 attribution applies; see the
[tree-source notice](../../licenses/html5ever/NOTICE.md#incorporated-tree-insertion-source).

## Restricted SVG script processing

The private fresh standalone SVG operation adapts the pinned HTML foreign-end
wrapper under the existing [WHATWG terms](../../licenses/whatwg/NOTICE.md).
Its processed/connectivity guard, parser-end versus nonparser connection and
processed mark derive separately from
[SVG Tiny 1.2 §15.2.1](../../references/svg-tiny12-script-processing--REC-20081222--2f36eb76440c.md#ScriptContentProcessing).
The [W3C notice](../../licenses/w3c/NOTICE.md#html-delegated-svg-script-processing)
retains the original Recommendation notice/status and the required ©2015
implementation-support derivative notice, with complete local
[2002](../../licenses/w3c/document-license-2002.txt) and
[2015](../../licenses/w3c/document-license-2015.txt) document terms.
This Rust algorithm adaptation is a bounded software implementation, not a
replacement technical specification or imported marked W3C code component.

Surgeist's separate closed no-loader and Disabled capabilities suppress all
acquisition and execution; the actual no-recipient domain eliminates SVG
notification work. The restriction preserves Tiny step 1 and connected step 3,
not a purported literal Tiny Disabled return. The session SVG child, state
registry, actual insertion cause, clone and connection integration are
Surgeist-authored. Existing html5ever-derived drivers retain their notices;
the [origin record](PARSER-ORIGINS.md#restricted-svg-script-extension) distinguishes
the pinned implementation's unimplemented SVG branches from the new HTML-owned
wrapper. The [WebKit clone metadata](../../references/webkit-svg-script-clone--73aa6c89e2cb-metadata.md)
supplies only the selected processed-bit preservation witness: no Library-GPL
source excerpt or executable WebKit code is incorporated. No SVG runtime,
loader, rendering-mode implementation or new dependency is added.

## Disabled-script checkpoint

The [microtask queue and complete checkpoint](../../references/html-microtask-checkpoint--877d6146487f.md)
come from the existing HTML877d edition, including all external cleanup/timing
hooks. The
[selected standalone reduction](../../references/SOURCE-EDITIONS.md#disabled-script-preparation-and-standalone-checkpoint-sources)
is an architecture capability specialization over absent runtime domains, not
a browser event-loop implementation or an external timing algorithm. No timing
source pin or normative null-script-return rule is selected. These adaptations
run within the crate's bounded immutable parser; they add no runtime or MIME
parser dependency and do not supply browser event-loop execution.

## html5lib-tests fixtures

The selected tree-construction and tokenizer fixture subset comes from
[html5lib-tests](https://github.com/html5lib/html5lib-tests), revision
[`f994590f528ac8b6073665791ddb1ed85c66dfb2`](https://github.com/html5lib/html5lib-tests/tree/f994590f528ac8b6073665791ddb1ed85c66dfb2).
The bundled files retain their exact upstream bytes; no upstream harness,
parser implementation or new runtime dependency is included.

* License: [complete upstream MIT license](tests/fixtures/html5lib-f994590f/LICENSE)
* Credits: [complete upstream AUTHORS.rst](tests/fixtures/html5lib-f994590f/AUTHORS.rst)
* Format: [upstream tree-construction README](tests/fixtures/html5lib-f994590f/tree-construction/README.md)
  and [upstream tokenizer README](tests/fixtures/html5lib-f994590f/tokenizer/README.md)

The license's original notice is Copyright (c) 2006-2013 James Graham,
Geoffrey Sneddon, and other contributors. Its copyright, permission notice and
disclaimer remain unchanged in the bundled LICENSE. Preserve LICENSE and
AUTHORS.rst with redistributed copies or substantial portions.

The 40 complete tree-construction data files under
tests/fixtures/html5lib-f994590f/tree-construction/ are adoption01.dat,
adoption02.dat, comments01.dat, entities01.dat, entities02.dat,
foreign-fragment.dat, namespace-sensitivity.dat, noscript01.dat, quirks01.dat,
ruby.dat, tables01.dat, template.dat, tests1.dat–tests12.dat,
tests14.dat–tests26.dat, tests_innerHTML_1.dat, webkit01.dat and webkit02.dat;
with their README they contain 386,921 bytes and 1,495 cases. The 13 complete
tokenizer files under tests/fixtures/html5lib-f994590f/tokenizer/ are
contentModelFlags.test, domjs.test, entities.test, escapeFlag.test,
namedEntities.test, numericEntities.test, pendingSpecChanges.test,
test1.test–test4.test, unicodeChars.test and unicodeCharsProblematic.test;
with their README they contain 1,649,095 bytes and 6,806 cases. Upstream
xmlViolation.test is not bundled. Together with LICENSE and AUTHORS.rst, the
57 files contain 2,037,579 upstream bytes, each matching its upstream git blob
SHA-1 at the pinned revision. Retaining those cases does not mean all cases
were executed or passed.
The [crate overview](../../docs/reference/html-operations.md#private-conformance-observation) identifies the
actual structural replay boundary and pending error-metadata comparison.
Surgeist's private checked native observer, structural adapter and tokenizer
replay are separately authored; they do not change this project's own license.

## Dependencies

This notice covers the 19 registry packages in this crate’s default-feature
normal and build dependency graph. They are external Cargo dependencies, not
vendored source. The HTML value dependency uses cssparser for safe legacy named-color
lookup. The repository-owned surgeist-css dependency supplies controlled color grammar
and source-qualified resolution; surgeist-color supplies numerical conversion
and serialization for parser input-color construction.
The license texts below reuse the exact retained copies in the CSS crate.
The graph is not an inventory of every downstream binary; linker behavior and
resolution determine a consumer’s contents. License alternatives remain intact.

This product depends on `cfg-if` 1.0.4 through surgeist-css stylesheet decoding,
distributed by Alex Crichton:

* License: [MIT](../surgeist-css/licenses/cfg-if/LICENSE-MIT) OR [Apache-2.0](../surgeist-css/licenses/cfg-if/LICENSE-APACHE)
* Homepage: [cfg-if](https://github.com/rust-lang/cfg-if)

This product depends on `cssparser` 0.37.0 for named-color lookup, with `cssparser-macros` 0.7.0 for procedural macros, distributed by Simon Sapin:

* License: [MPL-2.0](../surgeist-css/licenses/rust-cssparser/LICENSE)
* Homepage: [rust-cssparser](https://github.com/servo/rust-cssparser)

This product depends on `dtoa` 1.0.11 as a transitive runtime dependency,
distributed by David Tolnay:

* License: [MIT](../surgeist-css/licenses/dtoa/LICENSE-MIT) OR [Apache-2.0](../surgeist-css/licenses/dtoa/LICENSE-APACHE)
* Homepage: [dtoa](https://github.com/dtolnay/dtoa)

This product depends on `dtoa-short` 0.3.5 as a transitive runtime dependency,
distributed by Xidorn Quan:

* License: [MPL-2.0](../surgeist-css/licenses/dtoa-short/LICENSE)
* Homepage: [dtoa-short](https://github.com/upsuper/dtoa-short)

This product depends on `encoding_rs` 0.8.35 through surgeist-css stylesheet byte
decoding, copyright Mozilla Foundation, authored by Henri Sivonen. Its included
WHATWG Encoding Standard data carries the additional BSD license:

* License: ([Apache-2.0](../surgeist-css/licenses/encoding-rs/LICENSE-APACHE) OR [MIT](../surgeist-css/licenses/encoding-rs/LICENSE-MIT)) AND [BSD-3-Clause](../surgeist-css/licenses/encoding-rs/LICENSE-WHATWG); [upstream copyright and declaration](../surgeist-css/licenses/encoding-rs/COPYRIGHT)
* Homepage: [encoding_rs](https://github.com/hsivonen/encoding_rs)

This product depends on `fastrand` 2.4.1 for PHF macro generation at build time,
distributed by Stjepan Glavina:

* License: [Apache-2.0](../surgeist-css/licenses/fastrand/LICENSE-APACHE) OR [MIT](../surgeist-css/licenses/fastrand/LICENSE-MIT)
* Homepage: [fastrand](https://github.com/smol-rs/fastrand)

This product depends on `itoa` 1.0.18 as a transitive dependency,
distributed by David Tolnay:

* License: [MIT](../surgeist-css/licenses/itoa/LICENSE-MIT) OR [Apache-2.0](../surgeist-css/licenses/itoa/LICENSE-APACHE)
* Homepage: [itoa](https://github.com/dtolnay/itoa)

This product depends on `phf`, `phf_shared`, `phf_generator`, and `phf_macros`
0.13.1, distributed by Steven Fackler and Yuki Okushi. `phf` and `phf_shared`
support runtime lookup; `phf_generator`, `phf_macros`, and `phf_shared` also
support macro generation at build time:

* License: [MIT](../surgeist-css/licenses/rust-phf/LICENSE)
* Homepage: [rust-phf](https://github.com/rust-phf/rust-phf)

This product depends on `proc-macro2` 1.0.106 for procedural macro compilation,
distributed by David Tolnay and Alex Crichton:

* License: [MIT](../surgeist-css/licenses/proc-macro2/LICENSE-MIT) OR [Apache-2.0](../surgeist-css/licenses/proc-macro2/LICENSE-APACHE)
* Homepage: [proc-macro2](https://github.com/dtolnay/proc-macro2)

This product depends on `quote` 1.0.45 for procedural macro compilation,
distributed by David Tolnay:

* License: [MIT](../surgeist-css/licenses/quote/LICENSE-MIT) OR [Apache-2.0](../surgeist-css/licenses/quote/LICENSE-APACHE)
* Homepage: [quote](https://github.com/dtolnay/quote)

This product depends on `siphasher` 1.0.3 through PHF's runtime and macro
generation support, distributed by Frank Denis and the Rust Project Developers:

* License: [MIT](../surgeist-css/licenses/siphasher/MIT-LICENSE-TEXT) OR [Apache-2.0](../surgeist-css/licenses/siphasher/APACHE-2.0-LICENSE-TEXT); [upstream copyright and declaration](../surgeist-css/licenses/siphasher/COPYING)
* Homepage: [rust-siphash](https://github.com/jedisct1/rust-siphash)
* License-text provenance: [siphasher note](../surgeist-css/licenses/siphasher/NOTICE.md)

This product depends on `smallvec` 1.15.2 as a transitive runtime dependency,
distributed by the Servo Project Developers:

* License: [MIT](../surgeist-css/licenses/smallvec/LICENSE-MIT) OR [Apache-2.0](../surgeist-css/licenses/smallvec/LICENSE-APACHE)
* Homepage: [rust-smallvec](https://github.com/servo/rust-smallvec)

This product depends on `syn` 2.0.118 for procedural macro compilation,
distributed by David Tolnay:

* License: [MIT](../surgeist-css/licenses/syn/LICENSE-MIT) OR [Apache-2.0](../surgeist-css/licenses/syn/LICENSE-APACHE)
* Homepage: [syn](https://github.com/dtolnay/syn)

This product depends on `unicode-ident` 1.0.24 for procedural macro compilation,
distributed by David Tolnay, with Unicode data from Unicode, Inc.:

* License: ([MIT](../surgeist-css/licenses/unicode-ident/LICENSE-MIT) OR [Apache-2.0](../surgeist-css/licenses/unicode-ident/LICENSE-APACHE)) AND [Unicode-3.0](../surgeist-css/licenses/unicode-ident/LICENSE-UNICODE)
* Homepage: [unicode-ident](https://github.com/dtolnay/unicode-ident)

This product depends on `unicode-segmentation` 1.13.3 through surgeist-css
authored text-alignment string validation. Its package lists kwantam and Manish
Goregaokar as authors; the included MIT text credits the Rust Project Developers:

* License: [MIT](../surgeist-css/licenses/unicode-segmentation/LICENSE-MIT) OR [Apache-2.0](../surgeist-css/licenses/unicode-segmentation/LICENSE-APACHE); [upstream copyright and declaration](../surgeist-css/licenses/unicode-segmentation/COPYRIGHT)
* Homepage: [unicode-segmentation](https://github.com/unicode-rs/unicode-segmentation)
