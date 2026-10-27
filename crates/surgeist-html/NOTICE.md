# surgeist-html

Surgeist-authored code uses the repository [MIT License](../../LICENSE).

The selected HTML algorithms and source adaptations retain the [WHATWG notice](../../licenses/whatwg/NOTICE.md).

## Initial opener-policy record

The five opener-policy values and four initial record fields adapt the selected
HTML edition's [complete definitions](../../references/html-opener-policy-defaults--877d6146487f--d124556cde21.md).
Reference prose retains CC BY 4.0 attribution; the Rust adaptation uses the
WHATWG BSD 3-Clause code-incorporation provision with the complete
[copyright, conditions and disclaimer](../../licenses/whatwg/LICENSE).
This incorporation covers language values and immutable initial state, without
header parsing, policy matching, reporting or browsing-context group changes.

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

This crate has no external Cargo dependencies, so no third-party package
attribution applies. Its one dependency, [surgeist-color](../surgeist-color/NOTICE.md),
is repository-owned; it supplies the opaque named-color table that the legacy
color recovery uses, written from the retained CSS Color 4 publication under
that notice. No bundled upstream implementation is involved.
