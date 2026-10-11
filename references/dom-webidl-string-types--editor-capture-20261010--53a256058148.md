# Web IDL Standard: bounded DOM foundation source capture

Attribution and reformatting notice added for Surgeist on 2026-10-10. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Original English standard and normative/informative labels remain authoritative; this partial format conversion is implementation support, not a replacement specification or an implementation-conformance claim. No endorsement is implied.

Source: [Web IDL Standard](https://webidl.spec.whatwg.org/). Retrieved 2026-10-10. Displayed status: Web IDL Living Standard — Last Updated 10 October 2026.

The exact upstream repository LICENSE files from DOM `3071e5f269448350fdea79bb284eedaf17bc431e`, HTML `877d6146487f15b15bc8dd0e3c625063a37b456a`, Web IDL `0620cf171d1b97138847088748e363cae4005156` and Infra `c67ae1fb1d161b23d0f8c9bf9917ea1993f7b19a` all match the already retained [WHATWG LICENSE](../licenses/whatwg/LICENSE) byte-for-byte, SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. Specification prose is Creative Commons Attribution 4.0 International; the BSD 3-Clause scope applies when portions are incorporated into code, not as an alternative license for this Markdown.

Full retrieved HTML: 2,550,436 bytes, SHA-256 `802c98f87b3bb8d353a5e071041fb5091188d11669aaed6201627574ff18202c`. Complete-section extraction input: 8,696 bytes, SHA-256 `53a256058148d31c3b02c927031ba2febeceae8a3df7ba240a3ec5171408912a`. Source revision metadata: `0620cf171d1b97138847088748e363cae4005156`.

Retained complete heading sections, including descendant clauses and bounded at the next equal-or-higher-level heading: `idl-DOMString`, `idl-USVString`, `ipr`. Original source header is also retained. Every other clause and all cross-chapter references remain upstream. These are explicitly partial captures. The living page explicitly identifies its source revision above; exact capture hashes preserve that edition as implementation evidence. String types and algorithms are selected independently of JavaScript bindings.

Conversion uses existing repository HTML preparation/table-span models, approved Pandoc 3.1.11.1 and the semantic Lua filter. Source scripts/styles are omitted without execution. Source headings, IDs, links, prose, inline code, variables, literal blocks, images and every selected table cell/span are compared against parsed GFM. Selected tables remain literal semantic HTML, preserving every original table cell, code/link/variable, row/column span and anchor; table flattening is not used. Source examples and Web IDL remain evidence even where the corresponding JavaScript API is outside Surgeist's native Rust contract.

---

<!-- captured-body-start -->

[![WHATWG](https://resources.whatwg.org/logo-webidl.svg)](https://whatwg.org/)

# <a id="title"></a>Web IDL

<a id="subtitle"></a>Living Standard — Last Updated 10 October 2026

<strong>Participate:</strong>

[GitHub whatwg/webidl](https://github.com/whatwg/webidl) ([new issue](https://github.com/whatwg/webidl/issues/new/choose), [open issues](https://github.com/whatwg/webidl/issues))

[Chat on Matrix](https://whatwg.org/chat)

<strong>Commits:</strong>

[GitHub whatwg/webidl/commits](https://github.com/whatwg/webidl/commits)

<a id="commit-snapshot-link"></a>[Snapshot as of this commit](https://webidl.spec.whatwg.org/commit-snapshots/0620cf171d1b97138847088748e363cae4005156/)

<strong>Tests:</strong>

[web-platform-tests webidl/](https://github.com/web-platform-tests/wpt/tree/master/webidl) ([ongoing work](https://github.com/web-platform-tests/wpt/labels/webidl))

<strong>Translations (non-normative):</strong>

[日本語](https://triple-underscore.github.io/WebIDL-ja.html)

[简体中文](https://htmlspecs.com/webidl/)

[한국어](https://ko.htmlspecs.com/webidl/)

#### <a id="idl-DOMString"></a>2.13.17. <a id="dom-DOMString"></a>DOMString<a id="ref-for-idl-DOMString④⑥"></a>[](#idl-DOMString)

The <code><a id="ref-for-idl-DOMString①⑧"></a>[DOMString](#idl-DOMString)</code> type corresponds to <a id="ref-for-string"></a>[strings](https://infra.spec.whatwg.org/#string).

Note: null is not a value of type <code><a id="ref-for-idl-DOMString①⑨"></a>[DOMString](#idl-DOMString)</code>. To allow null, a <a id="ref-for-dfn-nullable-type①①"></a>[nullable](https://webidl.spec.whatwg.org/#dfn-nullable-type) <code><a id="ref-for-idl-DOMString②⓪"></a>[DOMString](#idl-DOMString)</code>, written as <code>DOMString?</code> in IDL, needs to be used.

Note: A <code><a id="ref-for-idl-DOMString②①"></a>[DOMString](#idl-DOMString)</code> value might include unmatched <a id="ref-for-surrogate"></a>[surrogate](https://infra.spec.whatwg.org/#surrogate) <a id="ref-for-code-point①"></a>[code points](https://infra.spec.whatwg.org/#code-point). Use <code><a id="ref-for-idl-USVString④"></a>[USVString](#idl-USVString)</code> if this is not desirable.

There is no way to represent a constant <code><a id="ref-for-idl-DOMString②②"></a>[DOMString](#idl-DOMString)</code> value in IDL, although <code><a id="ref-for-idl-DOMString②③"></a>[DOMString](#idl-DOMString)</code> <a id="ref-for-dfn-dictionary-member①⑨"></a>[dictionary member](https://webidl.spec.whatwg.org/#dfn-dictionary-member) <a id="ref-for-dfn-dictionary-member-default-value⑦"></a>[default values](https://webidl.spec.whatwg.org/#dfn-dictionary-member-default-value) and <a id="ref-for-dfn-optional-argument②⓪"></a>[operation optional argument](https://webidl.spec.whatwg.org/#dfn-optional-argument) <a id="ref-for-dfn-optional-argument-default-value⑧"></a>[default values](https://webidl.spec.whatwg.org/#dfn-optional-argument-default-value) can be set to <a id="ref-for-value-of-string-literal-tokens"></a>[the value](https://webidl.spec.whatwg.org/#value-of-string-literal-tokens) of a [string](https://webidl.spec.whatwg.org/#prod-string) literal.

#### <a id="idl-USVString"></a>2.13.19. <a id="dom-USVString"></a>USVString<a id="ref-for-idl-USVString①⑥"></a>[](#idl-USVString)

The <code><a id="ref-for-idl-USVString⑤"></a>[USVString](#idl-USVString)</code> type corresponds to <a id="ref-for-scalar-value-string"></a>[scalar value strings](https://infra.spec.whatwg.org/#scalar-value-string). Depending on the context, these can be treated as sequences of <a id="ref-for-code-unit"></a>[code units](https://infra.spec.whatwg.org/#code-unit) or <a id="ref-for-scalar-value⑦"></a>[scalar values](https://infra.spec.whatwg.org/#scalar-value).

There is no way to represent a constant <code><a id="ref-for-idl-USVString⑥"></a>[USVString](#idl-USVString)</code> value in IDL, although <code><a id="ref-for-idl-USVString⑦"></a>[USVString](#idl-USVString)</code> <a id="ref-for-dfn-dictionary-member②①"></a>[dictionary member](https://webidl.spec.whatwg.org/#dfn-dictionary-member) <a id="ref-for-dfn-dictionary-member-default-value⑨"></a>[default values](https://webidl.spec.whatwg.org/#dfn-dictionary-member-default-value) and <a id="ref-for-dfn-optional-argument②②"></a>[operation optional argument](https://webidl.spec.whatwg.org/#dfn-optional-argument) <a id="ref-for-dfn-optional-argument-default-value①⓪"></a>[default values](https://webidl.spec.whatwg.org/#dfn-optional-argument-default-value) can be set to <a id="ref-for-value-of-string-literal-tokens②"></a>[the value](https://webidl.spec.whatwg.org/#value-of-string-literal-tokens) of a [string](https://webidl.spec.whatwg.org/#prod-string) literal.

<strong>Warning:</strong>

Specifications should only use <code><a id="ref-for-idl-USVString⑧"></a>[USVString](#idl-USVString)</code> for APIs that perform text processing and need a string of <a id="ref-for-scalar-value⑧"></a>[scalar values](https://infra.spec.whatwg.org/#scalar-value) to operate on. Most APIs that use strings should instead be using <code><a id="ref-for-idl-DOMString②⑤"></a>[DOMString](#idl-DOMString)</code>, which does not make any interpretations of the <a id="ref-for-code-unit①"></a>[code units](https://infra.spec.whatwg.org/#code-unit) in the string. When in doubt, use <code><a id="ref-for-idl-DOMString②⑥"></a>[DOMString](#idl-DOMString)</code>.

## <a id="ipr"></a>Intellectual property rights[](#ipr)

This Living Standard includes material copied from W3C’s [WebIDL \[sic\] Level 1](https://www.w3.org/TR/2016/REC-WebIDL-1-20161215/), which is available under the [W3C Software and Document License](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document).

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). This work is licensed under a [Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/). To the extent portions of it are incorporated into source code, such portions in the source code are licensed under the [BSD 3-Clause License](https://opensource.org/licenses/BSD-3-Clause) instead.

This is the Living Standard. Those interested in the patent-review version should view the [Living Standard Review Draft](https://webidl.spec.whatwg.org/review-drafts/2026-09/).
