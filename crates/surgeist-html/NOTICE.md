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
prologue and doctype tables adapt two additional files. Their original notices are
retained verbatim, with separate adaptation notes and origin hashes. The same
MIT OR Apache-2.0 attribution applies; see the
[tree-source notice](../../licenses/html5ever/NOTICE.md#incorporated-tree-insertion-source).

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
