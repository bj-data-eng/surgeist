# surgeist-html

Surgeist-authored code uses the repository [MIT License](../../LICENSE).

The selected HTML algorithms and source adaptations are covered by the
[WHATWG notice](../../licenses/whatwg/NOTICE.md).

The private parser character-reference data is generated from the complete
named-reference and C1 tables in the pinned HTML syntax capture. The generator
retains all 2,231 names, 93 two-scalar values and 27 C1 replacements; it also
compares them with the acquired html5ever tables as independent witnesses.
Those engine files are not imported source or required generation inputs.

## Unicode bidirectional character data

The private first-strong lookup and its offline generator use complete Unicode
18.0.0 DerivedBidiClass data from Unicode, Inc. The source data and generated
table are distributed under the exact
[Unicode License v3](../../licenses/unicode/18.0.0/LICENSE), with
[source attribution](../../licenses/unicode/18.0.0/NOTICE.md). The selected
UAX #9 revision 52 remains an external normative citation; its report text is
not bundled. This lookup supplies HTML directionality rather than a full
bidirectional text-layout implementation.

## Delegated editable definition

The immutable editable predicate adapts the HTML-delegated definition from
[w3c/editing](https://github.com/w3c/editing), revision
`85de9ff4a9816139d06fc2dc39d5dacdc456fddc`. The source's original CC0 and
contributor-agreement notice and its separately declared W3C Software and
Document License are retained in the
[source notice](../../licenses/w3c/NOTICE.md#html-delegated-editability-definitions),
[repository license declaration](../../licenses/w3c/editing/LICENSE.md) and
[complete license text](../../licenses/w3c/software-license-2023.txt).
Surgeist adapts the definition to bounded immutable captures and explicit
document-state facts. It does not incorporate editing-command execution.
HTML itself remains the owner of editing-host classification, under the
separate WHATWG attribution above.

## Intrinsic iframe sandbox directive

The immutable sandbox value/report adapts the selected HTML edition's thirteen
keywords, author-only redundancy restrictions and sixteen directive flags.
The [source notice](../../licenses/whatwg/NOTICE.md#html-iframe-sandbox-directive--2026-10-11)
identifies the complete retained section and conversion. Reference prose remains
CC BY 4.0; incorporated Rust portions use the WHATWG BSD 3-Clause provision
with its exact [copyright, conditions and disclaimer](../../licenses/whatwg/LICENSE).
This adaptation does not incorporate Document active-policy composition,
navigation or sandbox enforcement.

## JavaScript MIME essence and checkpoint sources

The private Disabled-script preparation adaptation incorporates the exact 16
JavaScript MIME essence strings and ASCII case-insensitive matching definition
from [MIME Sniffing §4.6](../../references/mimesniff-javascript-essence--39aa53511b13.md),
revision `39aa53511b13953d84fef8d4131d6f61d0ccbde6`, into Rust source under
the WHATWG BSD 3-Clause code-incorporation provision. The bounded reference
prose remains CC BY 4.0; the existing complete
[WHATWG license](../../licenses/whatwg/LICENSE) and
[source notice](../../licenses/whatwg/NOTICE.md#javascript-mime-essence-and-html-checkpoint--2026-10-11)
retain the copyright, conditions, disclaimer and adaptation scope. HTML owns
the surrounding preparation rules; no MIME parser or sniffer is incorporated.

The [microtask queue and complete checkpoint](../../references/html-microtask-checkpoint--877d6146487f.md)
come from the existing HTML877d edition, including all external cleanup/timing
hooks. The
[selected standalone reduction](../../references/SOURCE-EDITIONS.md#disabled-script-preparation-and-standalone-checkpoint-sources)
is an architecture capability specialization over absent runtime domains, not
a browser event-loop implementation or an external timing algorithm. No timing
source pin or normative null-script-return rule is selected. These private
adaptations do not expose a complete parser or add a production dependency.

## Adapted parser source

The private, test-compiled tokenizer adapts four source files from
[html5ever](https://github.com/servo/html5ever), revision
`d7232d746d3112f08926d169591576bd12fe2fdf`. It preserves the upstream file
notices and the choice of [MIT](../../licenses/html5ever/LICENSE-MIT) or
[Apache-2.0](../../licenses/html5ever/LICENSE-APACHE), with exact
[copyright](../../licenses/html5ever/COPYRIGHT) and
[authorship](../../licenses/html5ever/AUTHORS) material. The
[origin record](PARSER-ORIGINS.md) identifies the incorporated files and their
modifications. This source adaptation adds no Cargo dependency on html5ever,
markup5ever, Tendril or web_atoms and does not yet expose a complete parser.

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

## Srcset density overflow adaptation

The intrinsic srcset producer follows the pinned HTML author grammar and
processing algorithm under the WHATWG attribution above. Its sole selected
WebKit correction treats failed density conversion as a candidate-local parse
error, omits that candidate and continues. The
[source determination](../../references/SOURCE-EDITIONS.md#srcset-density-conversion-overflow)
identifies the complete files at WebKit revision
`73aa6c89e2cb77c46184a81aec944e4ab99d114d` and the adapted branches.
HTMLParserIdioms.cpp retains the exact
[two-clause header](../../licenses/webkit/html-parser-idioms-bsd-2-clause.txt),
Copyright (C) 2010–2023 Apple Inc., 2014 Google Inc. and 2025 Samuel Weinig.
HTMLSrcsetParser.cpp retains the exact
[three-clause header](../../licenses/webkit/html-srcset-parser-bsd-3-clause.txt),
Copyright (C) 2013 Apple Inc. and 2013 Google Inc. Both complete conditions and
disclaimers are retained in the
[WebKit notice](../../licenses/webkit/NOTICE.md#srcset-density-conversion-witnesses).
The Rust adaptation incorporates only this recovery disposition, with native
exact values and standalone admission. It imports no WebKit dependency or
other grammar, integer bound, normalization or resource-selection pipeline.

## Dependencies

This notice covers the 16 registry packages in this crate’s default-feature
normal and build dependency graph. They are external Cargo dependencies, not
vendored source. The direct cssparser dependency supplies only its safe named-color
lookup; HTML does not invoke CSS tokenization or a Surgeist CSS parser. The
license texts below reuse the exact retained copies in the CSS crate. These
links identify legal material, not a production dependency on surgeist-css.
The graph is not an inventory of every downstream binary; linker behavior and
resolution determine a consumer’s contents. License alternatives remain intact.

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

## Ordinary-template and PI grammar incorporation

The private fresh ordinary-template algorithms use the existing pinned
HTML and DOM clauses under the
[WHATWG source-incorporation notice](../../licenses/whatwg/NOTICE.md#ordinary-template-and-pi-map-incorporation--2026-10-12).
The separate PI pseudo-attribute grammar comes from the dated W3C
[XML-stylesheet §3](../../references/xml-stylesheet-pseudo-attributes--REC-xml-stylesheet-20101028--a0e1a5198a21.md)
and [XML grammar closure](../../references/xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md),
with original ©2010/©2008 notices and the separately required ©2015
implementation-support derivative notices. The
[W3C attribution](../../licenses/w3c/NOTICE.md#dom-delegated-xml-pseudo-attribute-grammar)
and complete local [2002](../../licenses/w3c/document-license-2002.txt) and
[2015](../../licenses/w3c/document-license-2015.txt) document terms apply to those
distinct origins; no WHATWG or generic software-license substitution is made.
This is a bounded software/support implementation, not a replacement technical
specification, complete XML parser or stylesheet loader. No upstream W3C
marked code component or executable example is imported.
