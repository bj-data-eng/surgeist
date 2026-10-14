# surgeist-html

Surgeist-authored code uses the repository [MIT License](../../LICENSE).

The selected HTML algorithms and source adaptations retain the [WHATWG notice](../../licenses/whatwg/NOTICE.md).

## Intrinsic iframe sandbox directive

The immutable sandbox value/report adapts the selected HTML edition's thirteen
keywords, author-only redundancy restrictions and sixteen directive flags.
The [source notice](../../licenses/whatwg/NOTICE.md#html-iframe-sandbox-directive--2026-10-11)
identifies the complete retained section and conversion. Reference prose remains
CC BY 4.0; incorporated Rust portions use the WHATWG BSD 3-Clause provision
with its exact [copyright, conditions and disclaimer](../../licenses/whatwg/LICENSE).
This adaptation does not incorporate Document active-policy composition,
navigation or sandbox enforcement.

## JavaScript MIME essence

The local script-type classifier incorporates the exact 16
JavaScript MIME essence strings and ASCII case-insensitive matching definition
from [MIME Sniffing §4.6](../../references/mimesniff-javascript-essence--39aa53511b13.md),
revision `39aa53511b13953d84fef8d4131d6f61d0ccbde6`, into Rust source under
the WHATWG BSD 3-Clause code-incorporation provision. The bounded reference
prose remains CC BY 4.0; the existing complete
[WHATWG license](../../licenses/whatwg/LICENSE) and
[source notice](../../licenses/whatwg/NOTICE.md#javascript-mime-essence-and-html-checkpoint--2026-10-11)
retain the copyright, conditions, disclaimer and adaptation scope. HTML owns
this classification; the parser owns surrounding preparation rules. No MIME
parser or sniffer is incorporated.

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

## Ordinary-template and PI grammar incorporation

The parser-owned [fresh ordinary-template algorithms](../surgeist-html-parser/PARSER-ORIGINS.md) use the existing pinned
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

## Dependencies

This notice covers the 16 registry packages in this crate’s default-feature
normal and build dependency graph. They are external Cargo dependencies, not
vendored source. The direct cssparser dependency supplies safe named-color
lookup. The license texts below reuse the exact retained copies in the CSS crate.
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
