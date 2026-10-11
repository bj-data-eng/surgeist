# DOM Standard: bounded DOM foundation source capture

Attribution and reformatting notice added for Surgeist on 2026-10-10. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Original English standard and normative/informative labels remain authoritative; this partial format conversion is implementation support, not a replacement specification or an implementation-conformance claim. No endorsement is implied.

Source: [DOM Standard](https://dom.spec.whatwg.org/). Retrieved 2026-10-10. Displayed status: DOM Living Standard — Last Updated 9 October 2026.

The exact upstream repository LICENSE files from DOM `3071e5f269448350fdea79bb284eedaf17bc431e`, HTML `877d6146487f15b15bc8dd0e3c625063a37b456a`, Web IDL `0620cf171d1b97138847088748e363cae4005156` and Infra `c67ae1fb1d161b23d0f8c9bf9917ea1993f7b19a` all match the already retained [WHATWG LICENSE](../licenses/whatwg/LICENSE) byte-for-byte, SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. Specification prose is Creative Commons Attribution 4.0 International; the BSD 3-Clause scope applies when portions are incorporated into code, not as an alternative license for this Markdown.

Full retrieved HTML: 3,179,611 bytes, SHA-256 `034b308db8482bddb37070849e9d4ab40bcb2d10e6494cffad14678a252a58b1`. Complete-section extraction input: 776,620 bytes, SHA-256 `08dff0a4827fcd15c96ed49cdd2bc2b1bbc0c67adc54f2a979c1c1f887ffd73b`. Source revision metadata: `3071e5f269448350fdea79bb284eedaf17bc431e`.

Retained complete heading sections, including descendant clauses and bounded at the next equal-or-higher-level heading: `trees`, `namespaces`, `node-trees`, `interface-node`, `interface-document`, `interface-documenttype`, `interface-documentfragment`, `interface-shadowroot`, `interface-element`, `interface-characterdata`, `interface-text`, `interface-cdatasection`, `interface-processinginstruction`, `interface-comment`, `ipr`. Original source header is also retained. Every other clause and all cross-chapter references remain upstream. These are explicitly partial captures. The living page explicitly identifies DOM revision `3071e5f269448350fdea79bb284eedaf17bc431e`; the accessible [immutable commit witness](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/) is historical provenance. Normative selection refers to the living standard captured here, without promoting the snapshot warning into an implementation mandate.

Conversion uses existing repository HTML preparation/table-span models, approved Pandoc 3.1.11.1 and the semantic Lua filter. Source scripts/styles are omitted without execution. Source headings, IDs, links, prose, inline code, variables, literal blocks, images and every selected table cell/span are compared against parsed GFM. Selected tables remain literal semantic HTML, preserving every original table cell, code/link/variable, row/column span and anchor; table flattening is not used. Source examples and Web IDL remain evidence even where the corresponding JavaScript API is outside Surgeist's native Rust contract.

---

<!-- captured-body-start -->

[![WHATWG](https://resources.whatwg.org/logo-dom.svg)](https://whatwg.org/)

# <a id="title"></a>DOM

<a id="subtitle"></a>Living Standard — Last Updated 9 October 2026

<strong>Participate:</strong>

[GitHub whatwg/dom](https://github.com/whatwg/dom) ([new issue](https://github.com/whatwg/dom/issues/new/choose), [open issues](https://github.com/whatwg/dom/issues))

[Chat on Matrix](https://whatwg.org/chat)

<strong>Commits:</strong>

[GitHub whatwg/dom/commits](https://github.com/whatwg/dom/commits)

<a id="commit-snapshot-link"></a>[Snapshot as of this commit](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/)

<strong>Tests:</strong>

[web-platform-tests dom/](https://github.com/web-platform-tests/wpt/tree/master/dom) ([ongoing work](https://github.com/web-platform-tests/wpt/labels/dom))

<strong>Translations (non-normative):</strong>

[日本語](https://triple-underscore.github.io/DOM4-ja.html)

[简体中文](https://htmlspecs.com/dom/)

[한국어](https://ko.htmlspecs.com/dom/)

### <a id="trees"></a>1.1. Trees[](#trees)

A <a id="concept-tree"></a><strong>tree</strong> is a finite hierarchical tree structure. In <a id="concept-tree-order"></a><strong>tree order</strong> is preorder, depth-first traversal of a <a id="ref-for-concept-tree"></a>[tree](#concept-tree).

An object that <a id="concept-tree-participate"></a><strong>participates</strong> in a <a id="ref-for-concept-tree①"></a>[tree](#concept-tree) has a <a id="concept-tree-parent"></a><strong>parent</strong>, which is either null or an object, and has <a id="concept-tree-child"></a><strong>children</strong>, which is an <a id="ref-for-ordered-set"></a>[ordered set](https://infra.spec.whatwg.org/#ordered-set) of objects. An object <var>A</var> whose <a id="ref-for-concept-tree-parent"></a>[parent](#concept-tree-parent) is object <var>B</var> is a <a id="ref-for-concept-tree-child"></a>[child](#concept-tree-child) of <var>B</var>.

The <a id="concept-tree-root"></a><strong>root</strong> of an object is itself, if its <a id="ref-for-concept-tree-parent①"></a>[parent](#concept-tree-parent) is null, or else it is the <a id="ref-for-concept-tree-root"></a>[root](#concept-tree-root) of its <a id="ref-for-concept-tree-parent②"></a>[parent](#concept-tree-parent). The <a id="ref-for-concept-tree-root①"></a>[root](#concept-tree-root) of a <a id="ref-for-concept-tree②"></a>[tree](#concept-tree) is any object <a id="ref-for-concept-tree-participate"></a>[participating](#concept-tree-participate) in that <a id="ref-for-concept-tree③"></a>[tree](#concept-tree) whose <a id="ref-for-concept-tree-parent③"></a>[parent](#concept-tree-parent) is null.

An object <var>A</var> is called a <a id="concept-tree-descendant"></a><strong>descendant</strong> of an object <var>B</var>, if either <var>A</var> is a <a id="ref-for-concept-tree-child①"></a>[child](#concept-tree-child) of <var>B</var> or <var>A</var> is a <a id="ref-for-concept-tree-child②"></a>[child](#concept-tree-child) of an object <var>C</var> that is a <a id="ref-for-concept-tree-descendant"></a>[descendant](#concept-tree-descendant) of <var>B</var>.

An <a id="concept-tree-inclusive-descendant"></a><strong>inclusive descendant</strong> is an object or one of its <a id="ref-for-concept-tree-descendant①"></a>[descendants](#concept-tree-descendant).

An object <var>A</var> is called an <a id="concept-tree-ancestor"></a><strong>ancestor</strong> of an object <var>B</var> if and only if <var>B</var> is a <a id="ref-for-concept-tree-descendant②"></a>[descendant](#concept-tree-descendant) of <var>A</var>.

An <a id="concept-tree-inclusive-ancestor"></a><strong>inclusive ancestor</strong> is an object or one of its <a id="ref-for-concept-tree-ancestor"></a>[ancestors](#concept-tree-ancestor).

An object <var>A</var> is called a <a id="concept-tree-sibling"></a><strong>sibling</strong> of an object <var>B</var>, if and only if <var>B</var> and <var>A</var> share the same non-null <a id="ref-for-concept-tree-parent④"></a>[parent](#concept-tree-parent).

An <a id="concept-tree-inclusive-sibling"></a><strong>inclusive sibling</strong> is an object or one of its <a id="ref-for-concept-tree-sibling"></a>[siblings](#concept-tree-sibling).

An object <var>A</var> is <a id="concept-tree-preceding"></a><strong>preceding</strong> an object <var>B</var> if <var>A</var> and <var>B</var> are in the same <a id="ref-for-concept-tree④"></a>[tree](#concept-tree) and <var>A</var> comes before <var>B</var> in <a id="ref-for-concept-tree-order"></a>[tree order](#concept-tree-order).

An object <var>A</var> is <a id="concept-tree-following"></a><strong>following</strong> an object <var>B</var> if <var>A</var> and <var>B</var> are in the same <a id="ref-for-concept-tree⑤"></a>[tree](#concept-tree) and <var>A</var> comes after <var>B</var> in <a id="ref-for-concept-tree-order①"></a>[tree order](#concept-tree-order).

The <a id="concept-tree-first-child"></a><strong>first child</strong> of an object is its first <a id="ref-for-concept-tree-child③"></a>[child](#concept-tree-child) or null if it has no <a id="ref-for-concept-tree-child④"></a>[children](#concept-tree-child).

The <a id="concept-tree-last-child"></a><strong>last child</strong> of an object is its last <a id="ref-for-concept-tree-child⑤"></a>[child](#concept-tree-child) or null if it has no <a id="ref-for-concept-tree-child⑥"></a>[children](#concept-tree-child).

The <a id="concept-tree-previous-sibling"></a><strong>previous sibling</strong> of an object is its first <a id="ref-for-concept-tree-preceding"></a>[preceding](#concept-tree-preceding) <a id="ref-for-concept-tree-sibling①"></a>[sibling](#concept-tree-sibling) or null if it has no <a id="ref-for-concept-tree-preceding①"></a>[preceding](#concept-tree-preceding) <a id="ref-for-concept-tree-sibling②"></a>[sibling](#concept-tree-sibling).

The <a id="concept-tree-next-sibling"></a><strong>next sibling</strong> of an object is its first <a id="ref-for-concept-tree-following"></a>[following](#concept-tree-following) <a id="ref-for-concept-tree-sibling③"></a>[sibling](#concept-tree-sibling) or null if it has no <a id="ref-for-concept-tree-following①"></a>[following](#concept-tree-following) <a id="ref-for-concept-tree-sibling④"></a>[sibling](#concept-tree-sibling).

The <a id="concept-tree-index"></a><strong>index</strong> of an object is its number of <a id="ref-for-concept-tree-preceding②"></a>[preceding](#concept-tree-preceding) <a id="ref-for-concept-tree-sibling⑤"></a>[siblings](#concept-tree-sibling), or 0 if it has none.

### <a id="namespaces"></a>1.4. Name validation[](#namespaces)

A <a id="ref-for-string"></a>[string](https://infra.spec.whatwg.org/#string) is a <a id="valid-namespace-prefix"></a><strong>valid namespace prefix</strong> if its <a id="ref-for-string-length"></a>[length](https://infra.spec.whatwg.org/#string-length) is at least 1 and it does not contain <a id="ref-for-ascii-whitespace"></a>[ASCII whitespace](https://infra.spec.whatwg.org/#ascii-whitespace), U+0000 NULL, U+002F (/), or U+003E (\>).

A <a id="ref-for-string①"></a>[string](https://infra.spec.whatwg.org/#string) is a <a id="valid-attribute-local-name"></a><strong>valid attribute local name</strong> if its <a id="ref-for-string-length①"></a>[length](https://infra.spec.whatwg.org/#string-length) is at least 1 and it does not contain <a id="ref-for-ascii-whitespace①"></a>[ASCII whitespace](https://infra.spec.whatwg.org/#ascii-whitespace), U+0000 NULL, U+002F (/), U+003D (=), or U+003E (\>).

A <a id="ref-for-string②"></a>[string](https://infra.spec.whatwg.org/#string) <var>name</var> is a <a id="valid-element-local-name"></a><strong>valid element local name</strong> if the following steps return true:

1.  If <var>name</var>’s <a id="ref-for-string-length②"></a>[length](https://infra.spec.whatwg.org/#string-length) is 0, then return false.

2.  If <var>name</var>’s 0th <a id="ref-for-code-point"></a>[code point](https://infra.spec.whatwg.org/#code-point) is an <a id="ref-for-ascii-alpha"></a>[ASCII alpha](https://infra.spec.whatwg.org/#ascii-alpha):

    1.  If <var>name</var> contains <a id="ref-for-ascii-whitespace②"></a>[ASCII whitespace](https://infra.spec.whatwg.org/#ascii-whitespace), U+0000 NULL, U+002F (/), or U+003E (\>), then return false.

    2.  Return true.

3.  If <var>name</var>’s 0th <a id="ref-for-code-point①"></a>[code point](https://infra.spec.whatwg.org/#code-point) is not U+003A (:), U+005F (\_), or in the range U+0080 to U+10FFFF, inclusive, then return false.

4.  If <var>name</var>’s subsequent <a id="ref-for-code-point②"></a>[code points](https://infra.spec.whatwg.org/#code-point), if any, are not <a id="ref-for-ascii-alpha①"></a>[ASCII alphas](https://infra.spec.whatwg.org/#ascii-alpha), <a id="ref-for-ascii-digit"></a>[ASCII digits](https://infra.spec.whatwg.org/#ascii-digit), U+002D (-), U+002E (.), U+003A (:), U+005F (\_), or in the range U+0080 to U+10FFFF, inclusive, then return false.

5.  Return true.

<strong>Note:</strong>

This concept is used to validate <a id="ref-for-concept-element"></a>[element](#concept-element) <a id="ref-for-concept-element-local-name"></a>[local names](#concept-element-local-name), when constructed by DOM APIs. The intention is to allow any name that is possible to construct using the HTML parser (the branch where the first <a id="ref-for-code-point③"></a>[code point](https://infra.spec.whatwg.org/#code-point) is an <a id="ref-for-ascii-alpha②"></a>[ASCII alpha](https://infra.spec.whatwg.org/#ascii-alpha)), plus some additional possibilities. For those additional possibilities, the ASCII range is restricted for historical reasons, but beyond ASCII anything is allowed.

<strong>Note:</strong>

The following JavaScript-compatible regular expression is an implementation of <a id="ref-for-valid-element-local-name"></a>[valid element local name](#valid-element-local-name):

``` text
/^(?:[A-Za-z][^\0\t\n\f\r\u0020/>]*|[:_\u0080-\u{10FFFF}][A-Za-z0-9-.:_\u0080-\u{10FFFF}]*)$/u
```

A <a id="ref-for-string③"></a>[string](https://infra.spec.whatwg.org/#string) is a <a id="valid-doctype-name"></a><strong>valid doctype name</strong> if it does not contain <a id="ref-for-ascii-whitespace③"></a>[ASCII whitespace](https://infra.spec.whatwg.org/#ascii-whitespace), U+0000 NULL, or U+003E (\>).

<strong>Note:</strong>

The empty string is a <a id="ref-for-valid-doctype-name"></a>[valid doctype name](#valid-doctype-name).

To <a id="validate-and-extract"></a><strong>validate and extract</strong> a <var>namespace</var> and <var>qualifiedName</var>, given a <var>context</var>:

1.  If <var>namespace</var> is the empty string, then set it to null.

2.  Let <var>prefix</var> be null.

3.  Let <var>localName</var> be <var>qualifiedName</var>.

4.  If <var>qualifiedName</var> contains a U+003A (:):

    1.  Set <var>prefix</var> to the part of <var>qualifiedName</var> before the first U+003A (:).

    2.  Set <var>localName</var> to the part of <var>qualifiedName</var> after the first U+003A (:).

    3.  If <var>prefix</var> is not a <a id="ref-for-valid-namespace-prefix"></a>[valid namespace prefix](#valid-namespace-prefix), then <a id="ref-for-dfn-throw①"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-invalidcharactererror"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException①"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

5.  <a id="ref-for-assert"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>prefix</var> is either null or a <a id="ref-for-valid-namespace-prefix①"></a>[valid namespace prefix](#valid-namespace-prefix).

6.  If <var>context</var> is "<code>attribute</code>" and <var>localName</var> is not a <a id="ref-for-valid-attribute-local-name"></a>[valid attribute local name](#valid-attribute-local-name), then <a id="ref-for-dfn-throw②"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-invalidcharactererror①"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException②"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

7.  If <var>context</var> is "<code>element</code>" and <var>localName</var> is not a <a id="ref-for-valid-element-local-name①"></a>[valid element local name](#valid-element-local-name), then <a id="ref-for-dfn-throw③"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-invalidcharactererror②"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException③"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

8.  If <var>prefix</var> is non-null and <var>namespace</var> is null, then <a id="ref-for-dfn-throw④"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-namespaceerror"></a>[NamespaceError](https://webidl.spec.whatwg.org/#namespaceerror)</code>" <code><a id="ref-for-idl-DOMException④"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

9.  If <var>prefix</var> is "<code>xml</code>" and <var>namespace</var> is not the <a id="ref-for-xml-namespace"></a>[XML namespace](https://infra.spec.whatwg.org/#xml-namespace), then <a id="ref-for-dfn-throw⑤"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-namespaceerror①"></a>[NamespaceError](https://webidl.spec.whatwg.org/#namespaceerror)</code>" <code><a id="ref-for-idl-DOMException⑤"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

10. If either <var>qualifiedName</var> or <var>prefix</var> is "<code>xmlns</code>" and <var>namespace</var> is not the <a id="ref-for-xmlns-namespace"></a>[XMLNS namespace](https://infra.spec.whatwg.org/#xmlns-namespace), then <a id="ref-for-dfn-throw⑥"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-namespaceerror②"></a>[NamespaceError](https://webidl.spec.whatwg.org/#namespaceerror)</code>" <code><a id="ref-for-idl-DOMException⑥"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

11. If <var>namespace</var> is the <a id="ref-for-xmlns-namespace①"></a>[XMLNS namespace](https://infra.spec.whatwg.org/#xmlns-namespace) and neither <var>qualifiedName</var> nor <var>prefix</var> is "<code>xmlns</code>", then <a id="ref-for-dfn-throw⑦"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-namespaceerror③"></a>[NamespaceError](https://webidl.spec.whatwg.org/#namespaceerror)</code>" <code><a id="ref-for-idl-DOMException⑦"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

12. Return (<var>namespace</var>, <var>prefix</var>, <var>localName</var>).

<strong>Note:</strong>

Various APIs in this specification used to validate namespace prefixes, attribute local names, element local names, and doctype names more strictly. This was done in a way that aligned with various XML-related specifications. (Although not all rules from the those specifications were enforced.)

This was found to be annoying for web developers, especially since it meant there were some names that could be created by the HTML parser, but not by DOM APIs. So, the validations have been loosened to just those described above.

### <a id="node-trees"></a>4.2. Node tree[](#node-trees)

<a id="concept-node"></a><strong>Nodes</strong> are objects that <a id="ref-for-implements"></a>[implement](https://webidl.spec.whatwg.org/#implements) <code><a id="ref-for-node"></a>[Node](#node)</code>. <a id="ref-for-concept-node①④"></a>[Nodes](#concept-node) <a id="ref-for-concept-tree-participate④"></a>[participate](#concept-tree-participate) in a <a id="ref-for-concept-tree①④"></a>[tree](#concept-tree), which is known as the <a id="concept-node-tree"></a><strong>node tree</strong>.

<strong>Note:</strong>

In practice you deal with more specific objects.

Objects that <a id="ref-for-implements①"></a>[implement](https://webidl.spec.whatwg.org/#implements) <code><a id="ref-for-node①"></a>[Node](#node)</code> also implement an inherited interface: <code><a id="ref-for-document"></a>[Document](#document)</code>, <code><a id="ref-for-documenttype"></a>[DocumentType](#documenttype)</code>, <code><a id="ref-for-documentfragment"></a>[DocumentFragment](#documentfragment)</code>, <code><a id="ref-for-element④"></a>[Element](#element)</code>, <code><a id="ref-for-characterdata"></a>[CharacterData](#characterdata)</code>, or <code><a id="ref-for-attr"></a>[Attr](#attr)</code>.

Objects that implement <code><a id="ref-for-documentfragment①"></a>[DocumentFragment](#documentfragment)</code> sometimes implement <code><a id="ref-for-shadowroot①"></a>[ShadowRoot](#shadowroot)</code>.

Objects that implement <code><a id="ref-for-element⑤"></a>[Element](#element)</code> also typically implement an inherited interface, such as <code><a id="ref-for-htmlanchorelement"></a>[HTMLAnchorElement](https://html.spec.whatwg.org/multipage/text-level-semantics.html#htmlanchorelement)</code>.

Objects that implement <code><a id="ref-for-characterdata①"></a>[CharacterData](#characterdata)</code> also implement an inherited interface: <code><a id="ref-for-text④"></a>[Text](#text)</code>, <code><a id="ref-for-processinginstruction"></a>[ProcessingInstruction](#processinginstruction)</code>, or <code><a id="ref-for-comment"></a>[Comment](#comment)</code>.

Objects that implement <code><a id="ref-for-text⑤"></a>[Text](#text)</code> sometimes implement <code><a id="ref-for-cdatasection"></a>[CDATASection](#cdatasection)</code>.

Thus, every <a id="ref-for-concept-node①⑤"></a>[node](#concept-node)’s <a id="ref-for-dfn-primary-interface"></a>[primary interface](https://webidl.spec.whatwg.org/#dfn-primary-interface) is one of: <code><a id="ref-for-document①"></a>[Document](#document)</code>, <code><a id="ref-for-documenttype①"></a>[DocumentType](#documenttype)</code>, <code><a id="ref-for-documentfragment②"></a>[DocumentFragment](#documentfragment)</code>, <code><a id="ref-for-shadowroot②"></a>[ShadowRoot](#shadowroot)</code>, <code><a id="ref-for-element⑥"></a>[Element](#element)</code> or an inherited interface of <code><a id="ref-for-element⑦"></a>[Element](#element)</code>, <code><a id="ref-for-attr①"></a>[Attr](#attr)</code>, <code><a id="ref-for-text⑥"></a>[Text](#text)</code>, <code><a id="ref-for-cdatasection①"></a>[CDATASection](#cdatasection)</code>, <code><a id="ref-for-processinginstruction①"></a>[ProcessingInstruction](#processinginstruction)</code>, or <code><a id="ref-for-comment①"></a>[Comment](#comment)</code>.

For brevity, this specification refers to an object that <a id="ref-for-implements②"></a>[implements](https://webidl.spec.whatwg.org/#implements) <code><a id="ref-for-node②"></a>[Node](#node)</code> and an inherited interface <code><var>NodeInterface</var></code>, as a <code><var>NodeInterface</var></code> <a id="ref-for-concept-node①⑥"></a>[node](#concept-node).

A <a id="ref-for-concept-node-tree①"></a>[node tree](#concept-node-tree) is constrained as follows, expressed as a relationship between a <a id="ref-for-concept-node①⑦"></a>[node](#concept-node) and its potential <a id="ref-for-concept-tree-child⑧"></a>[children](#concept-tree-child):

<strong><code><a id="ref-for-document②"></a>[Document](#document)</code></strong>

In <a id="ref-for-concept-tree-order⑤"></a>[tree order](#concept-tree-order):

1.  Zero or more <code><a id="ref-for-processinginstruction②"></a>[ProcessingInstruction](#processinginstruction)</code> or <code><a id="ref-for-comment②"></a>[Comment](#comment)</code> <a id="ref-for-concept-node①⑧"></a>[nodes](#concept-node).

2.  Optionally one <code><a id="ref-for-documenttype②"></a>[DocumentType](#documenttype)</code> <a id="ref-for-concept-node①⑨"></a>[node](#concept-node).

3.  Zero or more <code><a id="ref-for-processinginstruction③"></a>[ProcessingInstruction](#processinginstruction)</code> or <code><a id="ref-for-comment③"></a>[Comment](#comment)</code> <a id="ref-for-concept-node②⓪"></a>[nodes](#concept-node).

4.  Optionally one <code><a id="ref-for-element⑧"></a>[Element](#element)</code> <a id="ref-for-concept-node②①"></a>[node](#concept-node).

5.  Zero or more <code><a id="ref-for-processinginstruction④"></a>[ProcessingInstruction](#processinginstruction)</code> or <code><a id="ref-for-comment④"></a>[Comment](#comment)</code> <a id="ref-for-concept-node②②"></a>[nodes](#concept-node).

<strong><code><a id="ref-for-documentfragment③"></a>[DocumentFragment](#documentfragment)</code></strong>

<strong><code><a id="ref-for-element⑨"></a>[Element](#element)</code></strong>

Zero or more <code><a id="ref-for-element①⓪"></a>[Element](#element)</code> or <code><a id="ref-for-characterdata②"></a>[CharacterData](#characterdata)</code> <a id="ref-for-concept-node②③"></a>[nodes](#concept-node).

<strong><code><a id="ref-for-documenttype③"></a>[DocumentType](#documenttype)</code></strong>

<strong><code><a id="ref-for-characterdata③"></a>[CharacterData](#characterdata)</code></strong>

<strong><code><a id="ref-for-attr②"></a>[Attr](#attr)</code></strong>

No <a id="ref-for-concept-tree-child⑨"></a>[children](#concept-tree-child).

<strong>Note:</strong>

<code><a id="ref-for-attr③"></a>[Attr](#attr)</code> <a id="ref-for-concept-node②④"></a>[nodes](#concept-node) <a id="ref-for-concept-tree-participate⑤"></a>[participate](#concept-tree-participate) in a <a id="ref-for-concept-tree①⑤"></a>[tree](#concept-tree) for historical reasons; they never have a (non-null) <a id="ref-for-concept-tree-parent⑤"></a>[parent](#concept-tree-parent) or any <a id="ref-for-concept-tree-child①⓪"></a>[children](#concept-tree-child) and are therefore alone in a <a id="ref-for-concept-tree①⑥"></a>[tree](#concept-tree).

To determine the <a id="concept-node-length"></a><strong>length</strong> of a <a id="ref-for-concept-node②⑤"></a>[node](#concept-node) <var>node</var>, run these steps:

1.  If <var>node</var> is a <code><a id="ref-for-documenttype④"></a>[DocumentType](#documenttype)</code> or <code><a id="ref-for-attr④"></a>[Attr](#attr)</code> <a id="ref-for-concept-node②⑥"></a>[node](#concept-node), then return 0.

2.  If <var>node</var> is a <code><a id="ref-for-characterdata④"></a>[CharacterData](#characterdata)</code> <a id="ref-for-concept-node②⑦"></a>[node](#concept-node), then return <var>node</var>’s <a id="ref-for-concept-cd-data"></a>[data](#concept-cd-data)’s <a id="ref-for-string-length③"></a>[length](https://infra.spec.whatwg.org/#string-length).

3.  Return the number of <var>node</var>’s <a id="ref-for-concept-tree-child①①"></a>[children](#concept-tree-child).

A <a id="ref-for-concept-node②⑧"></a>[node](#concept-node) is considered <a id="concept-node-empty"></a><strong>empty</strong> if its <a id="ref-for-concept-node-length"></a>[length](#concept-node-length) is 0.

#### <a id="document-trees"></a>4.2.1. Document tree[](#document-trees)

A <a id="concept-document-tree"></a><strong>document tree</strong> is a <a id="ref-for-concept-node-tree②"></a>[node tree](#concept-node-tree) whose <a id="ref-for-concept-tree-root⑧"></a>[root](#concept-tree-root) is a <a id="ref-for-concept-document③"></a>[document](#concept-document).

The <a id="document-element"></a><strong>document element</strong> of a <a id="ref-for-concept-document④"></a>[document](#concept-document) is the <a id="ref-for-concept-element⑤"></a>[element](#concept-element) whose <a id="ref-for-concept-tree-parent⑥"></a>[parent](#concept-tree-parent) is that <a id="ref-for-concept-document⑤"></a>[document](#concept-document), if it exists; otherwise null.

<strong>Note:</strong>

Per the <a id="ref-for-concept-node-tree③"></a>[node tree](#concept-node-tree) constraints, there can be only one such <a id="ref-for-concept-element⑥"></a>[element](#concept-element).

A <a id="ref-for-concept-node②⑨"></a>[node](#concept-node) is <a id="in-a-document-tree"></a><strong>in a document tree</strong> if its <a id="ref-for-concept-tree-root⑨"></a>[root](#concept-tree-root) is a <a id="ref-for-concept-document⑥"></a>[document](#concept-document).

A <a id="ref-for-concept-node③⓪"></a>[node](#concept-node) is <a id="in-a-document"></a><strong>in a document</strong> if it is <a id="ref-for-in-a-document-tree"></a>[in a document tree](#in-a-document-tree).

<strong>Note:</strong>

The term <a id="ref-for-in-a-document"></a>[in a document](#in-a-document) is no longer supposed to be used. It indicates that the standard using it has not been updated to account for <a id="ref-for-concept-shadow-tree②"></a>[shadow trees](#concept-shadow-tree).

#### <a id="shadow-trees"></a>4.2.2. Shadow tree[](#shadow-trees)

A <a id="concept-shadow-tree"></a><strong>shadow tree</strong> is a <a id="ref-for-concept-node-tree④"></a>[node tree](#concept-node-tree) whose <a id="ref-for-concept-tree-root①⓪"></a>[root](#concept-tree-root) is a <a id="ref-for-concept-shadow-root⑥"></a>[shadow root](#concept-shadow-root).

A <a id="ref-for-concept-shadow-root⑦"></a>[shadow root](#concept-shadow-root) is always attached to another <a id="ref-for-concept-node-tree⑤"></a>[node tree](#concept-node-tree) through its <a id="ref-for-concept-documentfragment-host"></a>[host](#concept-documentfragment-host). A <a id="ref-for-concept-shadow-tree③"></a>[shadow tree](#concept-shadow-tree) is therefore never alone. The <a id="ref-for-concept-node-tree⑥"></a>[node tree](#concept-node-tree) of a <a id="ref-for-concept-shadow-root⑧"></a>[shadow root](#concept-shadow-root)’s <a id="ref-for-concept-documentfragment-host①"></a>[host](#concept-documentfragment-host) is sometimes referred to as the <a id="concept-light-tree"></a><strong>light tree</strong>.

<strong>Note:</strong>

A <a id="ref-for-concept-shadow-tree④"></a>[shadow tree](#concept-shadow-tree)’s corresponding <a id="ref-for-concept-light-tree"></a>[light tree](#concept-light-tree) can be a <a id="ref-for-concept-shadow-tree⑤"></a>[shadow tree](#concept-shadow-tree) itself.

<a id="in-a-shadow-including-document"></a>A <a id="ref-for-concept-node③①"></a>[node](#concept-node) is <a id="connected"></a><strong>connected</strong> if its <a id="ref-for-concept-shadow-including-root"></a>[shadow-including root](#concept-shadow-including-root) is a <a id="ref-for-concept-document⑦"></a>[document](#concept-document).

##### <a id="shadow-tree-slots"></a>4.2.2.1. Slots[](#shadow-tree-slots)

A <a id="ref-for-concept-shadow-tree⑥"></a>[shadow tree](#concept-shadow-tree) contains zero or more <a id="ref-for-concept-element⑦"></a>[elements](#concept-element) that are <a id="concept-slot"></a><strong>slots</strong>.

<strong>Note:</strong>

A <a id="ref-for-concept-slot①"></a>[slot](#concept-slot) can only be created through HTML’s <code><a id="ref-for-the-slot-element"></a>[slot](https://html.spec.whatwg.org/multipage/scripting.html#the-slot-element)</code> element.

A <a id="ref-for-concept-slot②"></a>[slot](#concept-slot) has an associated <a id="slot-name"></a><strong>name</strong> (a string). Unless stated otherwise it is the empty string.

Use these <a id="ref-for-concept-element-attributes-change-ext"></a>[attribute change steps](#concept-element-attributes-change-ext) to update a <a id="ref-for-concept-slot③"></a>[slot](#concept-slot)’s <a id="ref-for-slot-name"></a>[name](#slot-name):

1.  If <var>element</var> is a <a id="ref-for-concept-slot④"></a>[slot](#concept-slot), <var>localName</var> is <code>name</code>, and <var>namespace</var> is null:

    1.  If <var>value</var> is <var>oldValue</var>, then return.

    2.  If <var>value</var> is null and <var>oldValue</var> is the empty string, then return.

    3.  If <var>value</var> is the empty string and <var>oldValue</var> is null, then return.

    4.  If <var>value</var> is null or the empty string, then set <var>element</var>’s <a id="ref-for-slot-name①"></a>[name](#slot-name) to the empty string.

    5.  Otherwise, set <var>element</var>’s <a id="ref-for-slot-name②"></a>[name](#slot-name) to <var>value</var>.

    6.  Run <a id="ref-for-assign-slotables-for-a-tree"></a>[assign slottables for a tree](#assign-slotables-for-a-tree) with <var>element</var>’s <a id="ref-for-concept-tree-root①①"></a>[root](#concept-tree-root).

<strong>Note:</strong>

The first <a id="ref-for-concept-slot⑤"></a>[slot](#concept-slot) in a <a id="ref-for-concept-shadow-tree⑦"></a>[shadow tree](#concept-shadow-tree), in <a id="ref-for-concept-tree-order⑥"></a>[tree order](#concept-tree-order), whose <a id="ref-for-slot-name③"></a>[name](#slot-name) is the empty string, is sometimes known as the "default slot".

A <a id="ref-for-concept-slot⑥"></a>[slot](#concept-slot) has an associated <a id="slot-assigned-nodes"></a><strong>assigned nodes</strong> (a list of <a id="ref-for-concept-slotable②"></a>[slottables](#concept-slotable)). Unless stated otherwise it is empty.

##### <a id="light-tree-slotables"></a>4.2.2.2. Slottables[](#light-tree-slotables)

<code><a id="ref-for-element①①"></a>[Element](#element)</code> and <code><a id="ref-for-text⑦"></a>[Text](#text)</code> <a id="ref-for-concept-node③②"></a>[nodes](#concept-node) are <a id="concept-slotable"></a><strong>slottables</strong>.

<strong>Note:</strong>

A <a id="ref-for-concept-slot⑦"></a>[slot](#concept-slot) can be a <a id="ref-for-concept-slotable③"></a>[slottable](#concept-slotable).

A <a id="ref-for-concept-slotable④"></a>[slottable](#concept-slotable) has an associated <a id="slotable-name"></a><strong>name</strong> (a string). Unless stated otherwise it is the empty string.

Use these <a id="ref-for-concept-element-attributes-change-ext①"></a>[attribute change steps](#concept-element-attributes-change-ext) to update a <a id="ref-for-concept-slotable⑤"></a>[slottable](#concept-slotable)’s <a id="ref-for-slotable-name"></a>[name](#slotable-name):

1.  If <var>localName</var> is <code>slot</code> and <var>namespace</var> is null:

    1.  If <var>value</var> is <var>oldValue</var>, then return.

    2.  If <var>value</var> is null and <var>oldValue</var> is the empty string, then return.

    3.  If <var>value</var> is the empty string and <var>oldValue</var> is null, then return.

    4.  If <var>value</var> is null or the empty string, then set <var>element</var>’s <a id="ref-for-slotable-name①"></a>[name](#slotable-name) to the empty string.

    5.  Otherwise, set <var>element</var>’s <a id="ref-for-slotable-name②"></a>[name](#slotable-name) to <var>value</var>.

    6.  If <var>element</var> is <a id="ref-for-slotable-assigned②"></a>[assigned](#slotable-assigned), then run <a id="ref-for-assign-slotables"></a>[assign slottables](#assign-slotables) for <var>element</var>’s <a id="ref-for-slotable-assigned-slot"></a>[assigned slot](#slotable-assigned-slot).

    7.  Run <a id="ref-for-assign-a-slot"></a>[assign a slot](#assign-a-slot) for <var>element</var>.

A <a id="ref-for-concept-slotable⑥"></a>[slottable](#concept-slotable) has an associated <a id="slotable-assigned-slot"></a><strong>assigned slot</strong> (null or a <a id="ref-for-concept-slot⑧"></a>[slot](#concept-slot)). Unless stated otherwise it is null. A <a id="ref-for-concept-slotable⑦"></a>[slottable](#concept-slotable) is <a id="slotable-assigned"></a><strong>assigned</strong> if its <a id="ref-for-slotable-assigned-slot①"></a>[assigned slot](#slotable-assigned-slot) is non-null.

A <a id="ref-for-concept-slotable⑧"></a>[slottable](#concept-slotable) has an associated <a id="slottable-manual-slot-assignment"></a><strong>manual slot assignment</strong> (null or a <a id="ref-for-concept-slot⑨"></a>[slot](#concept-slot)). Unless stated otherwise, it is null.

<strong>Note:</strong>

A <a id="ref-for-concept-slotable⑨"></a>[slottable](#concept-slotable)’s <a id="ref-for-slottable-manual-slot-assignment"></a>[manual slot assignment](#slottable-manual-slot-assignment) can be implemented using a weak reference to the <a id="ref-for-concept-slot①⓪"></a>[slot](#concept-slot), because this variable is not directly accessible from script.

##### <a id="finding-slots-and-slotables"></a>4.2.2.3. Finding slots and slottables[](#finding-slots-and-slotables)

To <a id="find-a-slot"></a><strong>find a slot</strong> for a given <a id="ref-for-concept-slotable①⓪"></a>[slottable](#concept-slotable) <var>slottable</var> and an optional boolean <var>open</var> (default false):

1.  If <var>slottable</var>’s <a id="ref-for-concept-tree-parent⑦"></a>[parent](#concept-tree-parent) is null, then return null.

2.  Let <var>shadow</var> be <var>slottable</var>’s <a id="ref-for-concept-tree-parent⑧"></a>[parent](#concept-tree-parent)’s <a id="ref-for-concept-element-shadow-root"></a>[shadow root](#concept-element-shadow-root).

3.  If <var>shadow</var> is null, then return null.

4.  If <var>open</var> is true and <var>shadow</var>’s <a id="ref-for-shadowroot-mode③"></a>[mode](#shadowroot-mode) is not "<code>open</code>", then return null.

5.  If <var>shadow</var>’s <a id="ref-for-shadowroot-slot-assignment"></a>[slot assignment](#shadowroot-slot-assignment) is "<code>manual</code>", then return the <a id="ref-for-concept-slot①①"></a>[slot](#concept-slot) in <var>shadow</var>’s <a id="ref-for-concept-tree-descendant③"></a>[descendants](#concept-tree-descendant) whose <a id="ref-for-manually-assigned-nodes"></a>[manually assigned nodes](https://html.spec.whatwg.org/multipage/scripting.html#manually-assigned-nodes) <a id="ref-for-list-contain③"></a>[contains](https://infra.spec.whatwg.org/#list-contain) <var>slottable</var>, if any; otherwise null.

6.  Return the first <a id="ref-for-concept-slot①②"></a>[slot](#concept-slot) in <a id="ref-for-concept-tree-order⑦"></a>[tree order](#concept-tree-order) in <var>shadow</var>’s <a id="ref-for-concept-tree-descendant④"></a>[descendants](#concept-tree-descendant) whose <a id="ref-for-slot-name④"></a>[name](#slot-name) is <var>slottable</var>’s <a id="ref-for-slotable-name③"></a>[name](#slotable-name), if any; otherwise null.

To <a id="find-slotables"></a><strong>find slottables</strong> for a given <a id="ref-for-concept-slot①③"></a>[slot](#concept-slot) <var>slot</var>:

1.  Let <var>result</var> be « ».

2.  Let <var>root</var> be <var>slot</var>’s <a id="ref-for-concept-tree-root①②"></a>[root](#concept-tree-root).

3.  If <var>root</var> is not a <a id="ref-for-concept-shadow-root⑨"></a>[shadow root](#concept-shadow-root), then return <var>result</var>.

4.  Let <var>host</var> be <var>root</var>’s <a id="ref-for-concept-documentfragment-host②"></a>[host](#concept-documentfragment-host).

5.  If <var>root</var>’s <a id="ref-for-shadowroot-slot-assignment①"></a>[slot assignment](#shadowroot-slot-assignment) is "<code>manual</code>":

    1.  <a id="ref-for-list-iterate①④"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <a id="ref-for-concept-slotable①①"></a>[slottable](#concept-slotable) <var>slottable</var> of <var>slot</var>’s <a id="ref-for-manually-assigned-nodes①"></a>[manually assigned nodes](https://html.spec.whatwg.org/multipage/scripting.html#manually-assigned-nodes), if <var>slottable</var>’s <a id="ref-for-concept-tree-parent⑨"></a>[parent](#concept-tree-parent) is <var>host</var>, <a id="ref-for-list-append⑧"></a>[append](https://infra.spec.whatwg.org/#list-append) <var>slottable</var> to <var>result</var>.

6.  Otherwise, for each <a id="ref-for-concept-slotable①②"></a>[slottable](#concept-slotable) <a id="ref-for-concept-tree-child①②"></a>[child](#concept-tree-child) <var>slottable</var> of <var>host</var>, in <a id="ref-for-concept-tree-order⑧"></a>[tree order](#concept-tree-order):

    1.  Let <var>foundSlot</var> be the result of <a id="ref-for-find-a-slot"></a>[finding a slot](#find-a-slot) given <var>slottable</var>.

    2.  If <var>foundSlot</var> is <var>slot</var>, then <a id="ref-for-list-append⑨"></a>[append](https://infra.spec.whatwg.org/#list-append) <var>slottable</var> to <var>result</var>.

7.  Return <var>result</var>.

To <a id="find-flattened-slotables"></a><strong>find flattened slottables</strong> for a given <a id="ref-for-concept-slot①④"></a>[slot](#concept-slot) <var>slot</var>:

1.  Let <var>result</var> be « ».

2.  If <var>slot</var>’s <a id="ref-for-concept-tree-root①③"></a>[root](#concept-tree-root) is not a <a id="ref-for-concept-shadow-root①⓪"></a>[shadow root](#concept-shadow-root), then return <var>result</var>.

3.  Let <var>slottables</var> be the result of <a id="ref-for-find-slotables"></a>[finding slottables](#find-slotables) given <var>slot</var>.

4.  If <var>slottables</var> is the empty list, then append each <a id="ref-for-concept-slotable①③"></a>[slottable](#concept-slotable) <a id="ref-for-concept-tree-child①③"></a>[child](#concept-tree-child) of <var>slot</var>, in <a id="ref-for-concept-tree-order⑨"></a>[tree order](#concept-tree-order), to <var>slottables</var>.

5.  For each <var>node</var> of <var>slottables</var>:

    1.  If <var>node</var> is a <a id="ref-for-concept-slot①⑤"></a>[slot](#concept-slot) whose <a id="ref-for-concept-tree-root①④"></a>[root](#concept-tree-root) is a <a id="ref-for-concept-shadow-root①①"></a>[shadow root](#concept-shadow-root):

        1.  Let <var>temporaryResult</var> be the result of <a id="ref-for-find-flattened-slotables"></a>[finding flattened slottables](#find-flattened-slotables) given <var>node</var>.

        2.  Append each <a id="ref-for-concept-slotable①④"></a>[slottable](#concept-slotable) in <var>temporaryResult</var>, in order, to <var>result</var>.

    2.  Otherwise, append <var>node</var> to <var>result</var>.

6.  Return <var>result</var>.

##### <a id="assigning-slotables-and-slots"></a>4.2.2.4. Assigning slottables and slots[](#assigning-slotables-and-slots)

To <a id="assign-slotables"></a><strong>assign slottables</strong> for a <a id="ref-for-concept-slot①⑥"></a>[slot](#concept-slot) <var>slot</var>:

1.  Let <var>slottables</var> be the result of <a id="ref-for-find-slotables①"></a>[finding slottables](#find-slotables) for <var>slot</var>.

2.  If <var>slottables</var> and <var>slot</var>’s <a id="ref-for-slot-assigned-nodes"></a>[assigned nodes](#slot-assigned-nodes) are not identical, then run <a id="ref-for-signal-a-slot-change"></a>[signal a slot change](#signal-a-slot-change) for <var>slot</var>.

3.  Set <var>slot</var>’s <a id="ref-for-slot-assigned-nodes①"></a>[assigned nodes](#slot-assigned-nodes) to <var>slottables</var>.

4.  For each <var>slottable</var> of <var>slottables</var>: set <var>slottable</var>’s <a id="ref-for-slotable-assigned-slot②"></a>[assigned slot](#slotable-assigned-slot) to <var>slot</var>.

To <a id="assign-slotables-for-a-tree"></a><strong>assign slottables for a tree</strong>, given a <a id="ref-for-concept-node③③"></a>[node](#concept-node) <var>root</var>, run <a id="ref-for-assign-slotables①"></a>[assign slottables](#assign-slotables) for each <a id="ref-for-concept-slot①⑦"></a>[slot](#concept-slot) of <var>root</var>’s <a id="ref-for-concept-tree-inclusive-descendant"></a>[inclusive descendants](#concept-tree-inclusive-descendant), in <a id="ref-for-concept-tree-order①⓪"></a>[tree order](#concept-tree-order).

To <a id="assign-a-slot"></a><strong>assign a slot</strong>, given a <a id="ref-for-concept-slotable①⑤"></a>[slottable](#concept-slotable) <var>slottable</var>:

1.  Let <var>slot</var> be the result of <a id="ref-for-find-a-slot①"></a>[finding a slot](#find-a-slot) with <var>slottable</var>.

2.  If <var>slot</var> is non-null, then run <a id="ref-for-assign-slotables②"></a>[assign slottables](#assign-slotables) for <var>slot</var>.

##### <a id="signaling-slot-change"></a>4.2.2.5. Signaling slot change[](#signaling-slot-change)

Each <a id="ref-for-similar-origin-window-agent"></a>[similar-origin window agent](https://html.spec.whatwg.org/multipage/webappapis.html#similar-origin-window-agent) has <a id="signal-slot-list"></a><strong>signal slots</strong> (a <a id="ref-for-ordered-set⑤"></a>[set](https://infra.spec.whatwg.org/#ordered-set) of <a id="ref-for-concept-slot①⑧"></a>[slots](#concept-slot)), which is initially empty. [\[HTML\]](https://dom.spec.whatwg.org/#biblio-html)

To <a id="signal-a-slot-change"></a><strong>signal a slot change</strong>, for a <a id="ref-for-concept-slot①⑨"></a>[slot](#concept-slot) <var>slot</var>:

1.  <a id="ref-for-set-append⑥"></a>[Append](https://infra.spec.whatwg.org/#set-append) <var>slot</var> to <var>slot</var>’s <a id="ref-for-relevant-agent"></a>[relevant agent](https://html.spec.whatwg.org/multipage/webappapis.html#relevant-agent)’s <a id="ref-for-signal-slot-list"></a>[signal slots](#signal-slot-list).

2.  <a id="ref-for-queue-a-mutation-observer-compound-microtask"></a>[Queue a mutation observer microtask](https://dom.spec.whatwg.org/#queue-a-mutation-observer-compound-microtask).

#### <a id="mutation-algorithms"></a>4.2.3. Mutation algorithms[](#mutation-algorithms)

To <a id="concept-node-ensure-pre-insertion-validity"></a><strong>ensure pre-insert validity</strong> given a <a id="ref-for-concept-node③④"></a>[node](#concept-node) <var>node</var>, a <a id="ref-for-concept-node③⑤"></a>[node](#concept-node) <var>parent</var>, null or a <a id="ref-for-concept-node③⑥"></a>[node](#concept-node) <var>child</var>, and a <a id="ref-for-list①①"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-concept-node③⑦"></a>[nodes](#concept-node) <var>childrenToExclude</var>:

1.  If <var>parent</var> is not a <code><a id="ref-for-document③"></a>[Document](#document)</code>, <code><a id="ref-for-documentfragment④"></a>[DocumentFragment](#documentfragment)</code>, or <code><a id="ref-for-element①②"></a>[Element](#element)</code> <a id="ref-for-concept-node③⑧"></a>[node](#concept-node), then <a id="ref-for-dfn-throw⑨"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException①⑤"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  If <var>node</var> is a <a id="ref-for-concept-tree-host-including-inclusive-ancestor"></a>[host-including inclusive ancestor](#concept-tree-host-including-inclusive-ancestor) of <var>parent</var>, then <a id="ref-for-dfn-throw①⓪"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror①"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException①⑥"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  If <var>child</var> is non-null and its <a id="ref-for-concept-tree-parent①⓪"></a>[parent](#concept-tree-parent) is not <var>parent</var>, then <a id="ref-for-dfn-throw①①"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notfounderror"></a>[NotFoundError](https://webidl.spec.whatwg.org/#notfounderror)</code>" <code><a id="ref-for-idl-DOMException①⑦"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

4.  If <var>node</var> is not a <code><a id="ref-for-documentfragment⑤"></a>[DocumentFragment](#documentfragment)</code>, <code><a id="ref-for-documenttype⑤"></a>[DocumentType](#documenttype)</code>, <code><a id="ref-for-element①③"></a>[Element](#element)</code>, or <code><a id="ref-for-characterdata⑤"></a>[CharacterData](#characterdata)</code> <a id="ref-for-concept-node③⑨"></a>[node](#concept-node), then <a id="ref-for-dfn-throw①②"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror②"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException①⑧"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

5.  If <var>parent</var> is not a <a id="ref-for-concept-document⑧"></a>[document](#concept-document):

    1.  If <var>node</var> is a <a id="ref-for-concept-doctype①"></a>[doctype](#concept-doctype), then <a id="ref-for-dfn-throw①③"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror③"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException①⑨"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

    2.  Return.

6.  If <var>node</var> is a <code><a id="ref-for-text⑧"></a>[Text](#text)</code> <a id="ref-for-concept-node④⓪"></a>[node](#concept-node), then <a id="ref-for-dfn-throw①④"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror④"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException②⓪"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

7.  If <var>node</var> is a <code><a id="ref-for-characterdata⑥"></a>[CharacterData](#characterdata)</code> <a id="ref-for-concept-node④①"></a>[node](#concept-node), then return.

8.  If <var>node</var> is a <code><a id="ref-for-documentfragment⑥"></a>[DocumentFragment](#documentfragment)</code> <a id="ref-for-concept-node④②"></a>[node](#concept-node):

    1.  If <var>node</var> has more than one <a id="ref-for-concept-element⑧"></a>[element](#concept-element) <a id="ref-for-concept-tree-child①④"></a>[child](#concept-tree-child) or has a <code><a id="ref-for-text⑨"></a>[Text](#text)</code> <a id="ref-for-concept-node④③"></a>[node](#concept-node) <a id="ref-for-concept-tree-child①⑤"></a>[child](#concept-tree-child), then <a id="ref-for-dfn-throw①⑤"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror⑤"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException②①"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

    2.  If <var>node</var> has no <a id="ref-for-concept-element⑨"></a>[element](#concept-element) <a id="ref-for-concept-tree-child①⑥"></a>[child](#concept-tree-child), then return.

9.  If <var>node</var> is a <code><a id="ref-for-documentfragment⑦"></a>[DocumentFragment](#documentfragment)</code> or <code><a id="ref-for-element①④"></a>[Element](#element)</code> <a id="ref-for-concept-node④④"></a>[node](#concept-node):

    1.  If any of the following are true:

        - <var>parent</var> has an <a id="ref-for-concept-element①⓪"></a>[element](#concept-element) <a id="ref-for-concept-tree-child①⑦"></a>[child](#concept-tree-child) that <var>childrenToExclude</var> does not <a id="ref-for-list-contain④"></a>[contain](https://infra.spec.whatwg.org/#list-contain);

        - <var>child</var> is non-null and a <a id="ref-for-concept-doctype②"></a>[doctype](#concept-doctype) is <a id="ref-for-concept-tree-following②"></a>[following](#concept-tree-following) <var>child</var>; or

        - <var>child</var> is a <a id="ref-for-concept-doctype③"></a>[doctype](#concept-doctype) that <var>childrenToExclude</var> does not <a id="ref-for-list-contain⑤"></a>[contain](https://infra.spec.whatwg.org/#list-contain),

        then <a id="ref-for-dfn-throw①⑥"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror⑥"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException②②"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

    2.  Return.

10. <a id="ref-for-assert②"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>node</var> is a <a id="ref-for-concept-doctype④"></a>[doctype](#concept-doctype).

11. If any of the following are true:

    - <var>parent</var> has a <a id="ref-for-concept-doctype⑤"></a>[doctype](#concept-doctype) <a id="ref-for-concept-tree-child①⑧"></a>[child](#concept-tree-child) that <var>childrenToExclude</var> does not <a id="ref-for-list-contain⑥"></a>[contain](https://infra.spec.whatwg.org/#list-contain);

    - <var>child</var> is non-null and an <a id="ref-for-concept-element①①"></a>[element](#concept-element) is <a id="ref-for-concept-tree-preceding③"></a>[preceding](#concept-tree-preceding) <var>child</var>; or

    - <var>child</var> is null and <var>parent</var> has an <a id="ref-for-concept-element①②"></a>[element](#concept-element) <a id="ref-for-concept-tree-child①⑨"></a>[child](#concept-tree-child) that <var>childrenToExclude</var> does not <a id="ref-for-list-contain⑦"></a>[contain](https://infra.spec.whatwg.org/#list-contain),

    then <a id="ref-for-dfn-throw①⑦"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror⑦"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException②③"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

To <a id="concept-node-pre-insert"></a><strong>pre-insert</strong> a <a id="ref-for-concept-node④⑤"></a>[node](#concept-node) <var>node</var> into a <a id="ref-for-concept-node④⑥"></a>[node](#concept-node) <var>parent</var> before null or <a id="ref-for-concept-node④⑦"></a>[node](#concept-node) <var>child</var>:

1.  <a id="ref-for-concept-node-ensure-pre-insertion-validity"></a>[Ensure pre-insert validity](#concept-node-ensure-pre-insertion-validity) given <var>node</var>, <var>parent</var>, <var>child</var>, and « ».

2.  Let <var>referenceChild</var> be <var>child</var>.

3.  If <var>referenceChild</var> is <var>node</var>, then set <var>referenceChild</var> to <var>node</var>’s <a id="ref-for-concept-tree-next-sibling"></a>[next sibling](#concept-tree-next-sibling).

4.  <a id="ref-for-concept-node-insert"></a>[Insert](#concept-node-insert) <var>node</var> into <var>parent</var> before <var>referenceChild</var>.

5.  Return <var>node</var>.

<a id="ref-for-other-applicable-specifications①"></a>[Specifications](https://dom.spec.whatwg.org/#other-applicable-specifications) may define <a id="concept-node-insert-ext"></a><strong>insertion steps</strong> for all or some <a id="ref-for-concept-node④⑧"></a>[nodes](#concept-node). The algorithm is passed <var>insertedNode</var>, as indicated in the <a id="ref-for-concept-node-insert①"></a>[insert](#concept-node-insert) algorithm below. These steps must not modify the <a id="ref-for-concept-node-tree⑦"></a>[node tree](#concept-node-tree) that <var>insertedNode</var> <a id="ref-for-concept-tree-participate⑥"></a>[participates](#concept-tree-participate) in, create <a id="ref-for-browsing-context"></a>[browsing contexts](https://html.spec.whatwg.org/multipage/document-sequences.html#browsing-context), <a id="ref-for-concept-event-fire⑥"></a>[fire events](https://dom.spec.whatwg.org/#concept-event-fire), or otherwise execute JavaScript. These steps may <a id="ref-for-queue-a-global-task②"></a>[queue tasks](https://html.spec.whatwg.org/multipage/webappapis.html#queue-a-global-task) to do these things asynchronously, however.

<a id="example-foo-what-do-i-put-here"></a>

<strong>Example:</strong>

[](#example-foo-what-do-i-put-here)

While the <a id="ref-for-concept-node-insert-ext"></a>[insertion steps](#concept-node-insert-ext) cannot execute JavaScript (among other things), they will indeed have script-observable consequences. Consider the below example:

``` text
const h1 = document.querySelector('h1');

const fragment = new DocumentFragment();
const script = fragment.appendChild(document.createElement('script'));
const style = fragment.appendChild(document.createElement('style'));

script.innerText= 'console.log(getComputedStyle(h1).color)'; // Logs 'rgb(255, 0, 0)'
style.innerText = 'h1 {color: rgb(255, 0, 0);}';

document.body.append(fragment);
```

The script in the above example logs <code>'rgb(255, 0, 0)'</code> because the following happen in order:

1.  The <a id="ref-for-concept-node-insert②"></a>[insert](#concept-node-insert) algorithm runs, which will insert the <code><a id="ref-for-script"></a>[script](https://html.spec.whatwg.org/multipage/scripting.html#script)</code> and <code> <a id="ref-for-the-style-element"></a>[style](https://html.spec.whatwg.org/multipage/semantics.html#the-style-element)</code> elements in order.

    1.  The HTML Standard’s <a id="ref-for-concept-node-insert-ext①"></a>[insertion steps](#concept-node-insert-ext) run for the <code><a id="ref-for-script①"></a>[script](https://html.spec.whatwg.org/multipage/scripting.html#script)</code> element; they do nothing. [\[HTML\]](https://dom.spec.whatwg.org/#biblio-html)

    2.  The HTML Standard’s <a id="ref-for-concept-node-insert-ext②"></a>[insertion steps](#concept-node-insert-ext) run for the <code> <a id="ref-for-the-style-element①"></a>[style](https://html.spec.whatwg.org/multipage/semantics.html#the-style-element)</code> element; they immediately apply its style rules to the document. [\[HTML\]](https://dom.spec.whatwg.org/#biblio-html)

    3.  The HTML Standard’s <a id="ref-for-concept-node-post-connection-ext"></a>[post-connection steps](#concept-node-post-connection-ext) run for the <code><a id="ref-for-script②"></a>[script](https://html.spec.whatwg.org/multipage/scripting.html#script)</code> element; they run the script, which immediately observes the style rules that were applied in the above step. [\[HTML\]](https://dom.spec.whatwg.org/#biblio-html)

<a id="ref-for-other-applicable-specifications②"></a>[Specifications](https://dom.spec.whatwg.org/#other-applicable-specifications) may also define <a id="concept-node-post-connection-ext"></a><strong>post-connection steps</strong> for all or some <a id="ref-for-concept-node④⑨"></a>[nodes](#concept-node). The algorithm is passed <var>connectedNode</var>, as indicated in the <a id="ref-for-concept-node-insert③"></a>[insert](#concept-node-insert) algorithm below.

<strong>Note:</strong>

The purpose of the <a id="ref-for-concept-node-post-connection-ext①"></a>[post-connection steps](#concept-node-post-connection-ext) is to provide an opportunity for <a id="ref-for-concept-node⑤⓪"></a>[nodes](#concept-node) to perform any connection-related operations that modify the <a id="ref-for-concept-node-tree⑧"></a>[node tree](#concept-node-tree) that <var>connectedNode</var> <a id="ref-for-concept-tree-participate⑦"></a>[participates](#concept-tree-participate) in, create <a id="ref-for-browsing-context①"></a>[browsing contexts](https://html.spec.whatwg.org/multipage/document-sequences.html#browsing-context), or otherwise execute JavaScript. These steps allow a batch of <a id="ref-for-concept-node⑤①"></a>[nodes](#concept-node) to be <a id="ref-for-concept-node-insert④"></a>[inserted](#concept-node-insert) <em>atomically</em> with respect to script, with all major side effects occurring <em>after</em> the batch insertions into the <a id="ref-for-concept-node-tree⑨"></a>[node tree](#concept-node-tree) is complete. This ensures that all pending <a id="ref-for-concept-node-tree①⓪"></a>[node tree](#concept-node-tree) insertions completely finish before more insertions can occur.

<a id="ref-for-other-applicable-specifications③"></a>[Specifications](https://dom.spec.whatwg.org/#other-applicable-specifications) may define <a id="concept-node-children-changed-ext"></a><strong>children changed steps</strong> for all or some <a id="ref-for-concept-node⑤②"></a>[nodes](#concept-node). The algorithm is passed no argument and is called from <a id="ref-for-concept-node-insert⑤"></a>[insert](#concept-node-insert), <a id="ref-for-concept-node-remove"></a>[remove](#concept-node-remove), and <a id="ref-for-concept-cd-replace"></a>[replace data](#concept-cd-replace).

To <a id="concept-node-insert"></a><strong>insert</strong> a <a id="ref-for-concept-node⑤③"></a>[node](#concept-node) <var>node</var> into a <a id="ref-for-concept-node⑤④"></a>[node](#concept-node) <var>parent</var> before null or a <a id="ref-for-concept-node⑤⑤"></a>[node](#concept-node) <var>child</var>, with an optional boolean <a id="insert-suppressobservers"></a><strong><var>suppressObservers</var></strong> (default false):

1.  Let <var>nodes</var> be <var>node</var>’s <a id="ref-for-concept-tree-child②⓪"></a>[children](#concept-tree-child), if <var>node</var> is a <code><a id="ref-for-documentfragment⑧"></a>[DocumentFragment](#documentfragment)</code> <a id="ref-for-concept-node⑤⑥"></a>[node](#concept-node); otherwise « <var>node</var> ».

2.  Let <var>count</var> be <var>nodes</var>’s <a id="ref-for-list-size②"></a>[size](https://infra.spec.whatwg.org/#list-size).

3.  If <var>count</var> is 0, then return.

4.  If <var>node</var> is a <code><a id="ref-for-documentfragment⑨"></a>[DocumentFragment](#documentfragment)</code> <a id="ref-for-concept-node⑤⑦"></a>[node](#concept-node):

    1.  <a id="ref-for-concept-node-remove①"></a>[Remove](#concept-node-remove) its <a id="ref-for-concept-tree-child②①"></a>[children](#concept-tree-child) with <a id="ref-for-remove-suppressobservers"></a>[<em>suppressObservers</em>](#remove-suppressobservers) set to true.

    2.  <a id="ref-for-queue-a-tree-mutation-record"></a>[Queue a tree mutation record](https://dom.spec.whatwg.org/#queue-a-tree-mutation-record) for <var>node</var> with « », <var>nodes</var>, null, and null.

        <strong>Note:</strong>

        This step intentionally does not pay attention to <var>suppressObservers</var>.

5.  If <var>child</var> is non-null:

    1.  For each <a id="ref-for-concept-live-range"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-start-node"></a>[start node](https://dom.spec.whatwg.org/#concept-range-start-node) is <var>parent</var> and <a id="ref-for-concept-range-start-offset"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) is greater than <var>child</var>’s <a id="ref-for-concept-tree-index"></a>[index](#concept-tree-index): increase its <a id="ref-for-concept-range-start-offset①"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) by <var>count</var>.

    2.  For each <a id="ref-for-concept-live-range①"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-end-node"></a>[end node](https://dom.spec.whatwg.org/#concept-range-end-node) is <var>parent</var> and <a id="ref-for-concept-range-end-offset"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) is greater than <var>child</var>’s <a id="ref-for-concept-tree-index①"></a>[index](#concept-tree-index): increase its <a id="ref-for-concept-range-end-offset①"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) by <var>count</var>.

6.  Let <var>previousSibling</var> be <var>child</var>’s <a id="ref-for-concept-tree-previous-sibling"></a>[previous sibling](#concept-tree-previous-sibling) or <var>parent</var>’s <a id="ref-for-concept-tree-last-child"></a>[last child](#concept-tree-last-child) if <var>child</var> is null.

7.  For each <var>node</var> in <var>nodes</var>, in <a id="ref-for-concept-tree-order①①"></a>[tree order](#concept-tree-order):

    1.  <a id="ref-for-concept-node-adopt"></a>[Adopt](#concept-node-adopt) <var>node</var> into <var>parent</var>’s <a id="ref-for-concept-node-document④"></a>[node document](#concept-node-document).

    2.  If <var>child</var> is null, then <a id="ref-for-set-append⑦"></a>[append](https://infra.spec.whatwg.org/#set-append) <var>node</var> to <var>parent</var>’s <a id="ref-for-concept-tree-child②②"></a>[children](#concept-tree-child).

    3.  Otherwise, <a id="ref-for-list-insert"></a>[insert](https://infra.spec.whatwg.org/#list-insert) <var>node</var> into <var>parent</var>’s <a id="ref-for-concept-tree-child②③"></a>[children](#concept-tree-child) before <var>child</var>’s <a id="ref-for-concept-tree-index②"></a>[index](#concept-tree-index).

    4.  If <var>parent</var> is a <a id="ref-for-element-shadow-host"></a>[shadow host](#element-shadow-host) whose <a id="ref-for-concept-shadow-root①②"></a>[shadow root](#concept-shadow-root)’s <a id="ref-for-shadowroot-slot-assignment②"></a>[slot assignment](#shadowroot-slot-assignment) is "<code>named</code>" and <var>node</var> is a <a id="ref-for-concept-slotable①⑥"></a>[slottable](#concept-slotable), then <a id="ref-for-assign-a-slot①"></a>[assign a slot](#assign-a-slot) for <var>node</var>.

    5.  If <var>parent</var>’s <a id="ref-for-concept-tree-root①⑤"></a>[root](#concept-tree-root) is a <a id="ref-for-concept-shadow-root①③"></a>[shadow root](#concept-shadow-root), and <var>parent</var> is a <a id="ref-for-concept-slot②⓪"></a>[slot](#concept-slot) whose <a id="ref-for-slot-assigned-nodes②"></a>[assigned nodes](#slot-assigned-nodes) is the empty list, then run <a id="ref-for-signal-a-slot-change①"></a>[signal a slot change](#signal-a-slot-change) for <var>parent</var>.

    6.  Run <a id="ref-for-assign-slotables-for-a-tree①"></a>[assign slottables for a tree](#assign-slotables-for-a-tree) with <var>node</var>’s <a id="ref-for-concept-tree-root①⑥"></a>[root](#concept-tree-root).

    7.  For each <a id="ref-for-concept-shadow-including-inclusive-descendant"></a>[shadow-including inclusive descendant](#concept-shadow-including-inclusive-descendant) <var>inclusiveDescendant</var> of <var>node</var>, in <a id="ref-for-concept-shadow-including-tree-order"></a>[shadow-including tree order](#concept-shadow-including-tree-order):

        1.  Run the <a id="ref-for-concept-node-insert-ext③"></a>[insertion steps](#concept-node-insert-ext) with <var>inclusiveDescendant</var>.

        2.  If <var>inclusiveDescendant</var> is not <a id="ref-for-connected"></a>[connected](#connected), then <a id="ref-for-iteration-continue⑤"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

        3.  If <var>inclusiveDescendant</var> is an <a id="ref-for-concept-element①③"></a>[element](#concept-element) and <var>inclusiveDescendant</var>’s <a id="ref-for-element-custom-element-registry"></a>[custom element registry](#element-custom-element-registry) is non-null:

            1.  If <var>inclusiveDescendant</var>’s <a id="ref-for-element-custom-element-registry①"></a>[custom element registry](#element-custom-element-registry)’s <a id="ref-for-is-scoped"></a>[is scoped](https://html.spec.whatwg.org/multipage/custom-elements.html#is-scoped) is true, then <a id="ref-for-set-append⑧"></a>[append](https://infra.spec.whatwg.org/#set-append) <var>inclusiveDescendant</var>’s <a id="ref-for-concept-node-document⑤"></a>[node document](#concept-node-document) to <var>inclusiveDescendant</var>’s <a id="ref-for-element-custom-element-registry②"></a>[custom element registry](#element-custom-element-registry)’s <a id="ref-for-scoped-document-set"></a>[scoped document set](https://html.spec.whatwg.org/multipage/custom-elements.html#scoped-document-set).

            2.  If <var>inclusiveDescendant</var> is <a id="ref-for-concept-element-custom"></a>[custom](#concept-element-custom), then <a id="ref-for-enqueue-a-custom-element-callback-reaction"></a>[enqueue a custom element callback reaction](https://html.spec.whatwg.org/multipage/custom-elements.html#enqueue-a-custom-element-callback-reaction) with <var>inclusiveDescendant</var>, callback name "<code>connectedCallback</code>", and « ».

            3.  Otherwise, <a id="ref-for-concept-try-upgrade"></a>[try to upgrade](https://html.spec.whatwg.org/multipage/custom-elements.html#concept-try-upgrade) <var>inclusiveDescendant</var>.

                <strong>Note:</strong>

                If this successfully upgrades <var>inclusiveDescendant</var>, its <code>connectedCallback</code> will be enqueued automatically during the <a id="ref-for-concept-upgrade-an-element"></a>[upgrade an element](https://html.spec.whatwg.org/multipage/custom-elements.html#concept-upgrade-an-element) algorithm.

        4.  Otherwise, if <var>inclusiveDescendant</var> is a <a id="ref-for-concept-shadow-root①④"></a>[shadow root](#concept-shadow-root), <var>inclusiveDescendant</var>’s <a id="ref-for-shadowroot-custom-element-registry"></a>[custom element registry](#shadowroot-custom-element-registry) is non-null, and <var>inclusiveDescendant</var>’s <a id="ref-for-shadowroot-custom-element-registry①"></a>[custom element registry](#shadowroot-custom-element-registry)’s <a id="ref-for-is-scoped①"></a>[is scoped](https://html.spec.whatwg.org/multipage/custom-elements.html#is-scoped) is true, then <a id="ref-for-set-append⑨"></a>[append](https://infra.spec.whatwg.org/#set-append) <var>inclusiveDescendant</var>’s <a id="ref-for-concept-node-document⑥"></a>[node document](#concept-node-document) to <var>inclusiveDescendant</var>’s <a id="ref-for-shadowroot-custom-element-registry②"></a>[custom element registry](#shadowroot-custom-element-registry)’s <a id="ref-for-scoped-document-set①"></a>[scoped document set](https://html.spec.whatwg.org/multipage/custom-elements.html#scoped-document-set).

8.  If <var>suppressObservers</var> is false, then <a id="ref-for-queue-a-tree-mutation-record①"></a>[queue a tree mutation record](https://dom.spec.whatwg.org/#queue-a-tree-mutation-record) for <var>parent</var> with <var>nodes</var>, « », <var>previousSibling</var>, and <var>child</var>.

9.  Run the <a id="ref-for-concept-node-children-changed-ext"></a>[children changed steps](#concept-node-children-changed-ext) for <var>parent</var>.

10. Let <var>staticNodeList</var> be a <a id="ref-for-list①②"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-concept-node⑤⑧"></a>[nodes](#concept-node), initially « ».

    <strong>Note:</strong>

    We collect all <a id="ref-for-concept-node⑤⑨"></a>[nodes](#concept-node) <em>before</em> calling the <a id="ref-for-concept-node-post-connection-ext②"></a>[post-connection steps](#concept-node-post-connection-ext) on any one of them, instead of calling the <a id="ref-for-concept-node-post-connection-ext③"></a>[post-connection steps](#concept-node-post-connection-ext) <em>while</em> we’re traversing the <a id="ref-for-concept-node-tree①①"></a>[node tree](#concept-node-tree). This is because the <a id="ref-for-concept-node-post-connection-ext④"></a>[post-connection steps](#concept-node-post-connection-ext) can modify the tree’s structure, making live traversal unsafe, possibly leading to the <a id="ref-for-concept-node-post-connection-ext⑤"></a>[post-connection steps](#concept-node-post-connection-ext) being called multiple times on the same <a id="ref-for-concept-node⑥⓪"></a>[node](#concept-node).

11. For each <var>node</var> of <var>nodes</var>, in <a id="ref-for-concept-tree-order①②"></a>[tree order](#concept-tree-order):

    1.  For each <a id="ref-for-concept-shadow-including-inclusive-descendant①"></a>[shadow-including inclusive descendant](#concept-shadow-including-inclusive-descendant) <var>inclusiveDescendant</var> of <var>node</var>, in <a id="ref-for-concept-shadow-including-tree-order①"></a>[shadow-including tree order](#concept-shadow-including-tree-order): <a id="ref-for-list-append①⓪"></a>[append](https://infra.spec.whatwg.org/#list-append) <var>inclusiveDescendant</var> to <var>staticNodeList</var>.

12. <a id="ref-for-list-iterate①⑤"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>node</var> of <var>staticNodeList</var>: if <var>node</var> is <a id="ref-for-connected①"></a>[connected](#connected), then run the <a id="ref-for-concept-node-post-connection-ext⑥"></a>[post-connection steps](#concept-node-post-connection-ext) with <var>node</var>.

To <a id="concept-node-append"></a><strong>append</strong> a <a id="ref-for-concept-node⑥①"></a>[node](#concept-node) <var>node</var> to a <a id="ref-for-concept-node⑥②"></a>[node](#concept-node) <var>parent</var>: <a id="ref-for-concept-node-pre-insert"></a>[pre-insert](#concept-node-pre-insert) <var>node</var> into <var>parent</var> before null.

------------------------------------------------------------------------

<a id="ref-for-other-applicable-specifications④"></a>[Specifications](https://dom.spec.whatwg.org/#other-applicable-specifications) may define <a id="concept-node-move-ext"></a><strong>moving steps</strong> for all or some <a id="ref-for-concept-node⑥③"></a>[nodes](#concept-node). The algorithm is passed a <a id="ref-for-concept-node⑥④"></a>[node](#concept-node) <var>movedNode</var>, a boolean <var>isSubtreeRoot</var>, and a <a id="ref-for-concept-node⑥⑤"></a>[node](#concept-node) <var>oldAncestor</var> as indicated in the <a id="ref-for-move"></a>[move](#move) algorithm below. Like the <a id="ref-for-concept-node-insert-ext④"></a>[insertion steps](#concept-node-insert-ext), these steps must not modify the <a id="ref-for-concept-node-tree①②"></a>[node tree](#concept-node-tree) that <var>movedNode</var> <a id="ref-for-concept-tree-participate⑧"></a>[participates](#concept-tree-participate) in, create <a id="ref-for-browsing-context②"></a>[browsing contexts](https://html.spec.whatwg.org/multipage/document-sequences.html#browsing-context), <a id="ref-for-concept-event-fire⑦"></a>[fire events](https://dom.spec.whatwg.org/#concept-event-fire), or otherwise execute JavaScript. These steps may queue tasks to do these things asynchronously, however.

To <a id="move"></a><strong>move</strong> a <a id="ref-for-concept-node⑥⑥"></a>[node](#concept-node) <var>node</var> into a <a id="ref-for-concept-node⑥⑦"></a>[node](#concept-node) <var>newParent</var> before null or a <a id="ref-for-concept-node⑥⑧"></a>[node](#concept-node) <var>child</var>:

1.  If <var>newParent</var>’s <a id="ref-for-concept-shadow-including-root①"></a>[shadow-including root](#concept-shadow-including-root) is not the same as <var>node</var>’s <a id="ref-for-concept-shadow-including-root②"></a>[shadow-including root](#concept-shadow-including-root), then <a id="ref-for-dfn-throw①⑧"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror⑧"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException②④"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

    <strong>Note:</strong>

    This has the side effect of ensuring that a move is only performed if <var>newParent</var>’s <a id="ref-for-connected②"></a>[connected](#connected) is <var>node</var>’s <a id="ref-for-connected③"></a>[connected](#connected).

2.  If <var>node</var> is a <a id="ref-for-concept-tree-host-including-inclusive-ancestor①"></a>[host-including inclusive ancestor](#concept-tree-host-including-inclusive-ancestor) of <var>newParent</var>, then <a id="ref-for-dfn-throw①⑨"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror⑨"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException②⑤"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  If <var>child</var> is non-null and its <a id="ref-for-concept-tree-parent①①"></a>[parent](#concept-tree-parent) is not <var>newParent</var>, then <a id="ref-for-dfn-throw②⓪"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notfounderror①"></a>[NotFoundError](https://webidl.spec.whatwg.org/#notfounderror)</code>" <code><a id="ref-for-idl-DOMException②⑥"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

4.  If <var>node</var> is not an <code><a id="ref-for-element①⑤"></a>[Element](#element)</code> or a <code><a id="ref-for-characterdata⑦"></a>[CharacterData](#characterdata)</code> <a id="ref-for-concept-node⑥⑨"></a>[node](#concept-node), then <a id="ref-for-dfn-throw②①"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror①⓪"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException②⑦"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

5.  If <var>node</var> is a <code><a id="ref-for-text①⓪"></a>[Text](#text)</code> <a id="ref-for-concept-node⑦⓪"></a>[node](#concept-node) and <var>newParent</var> is a <a id="ref-for-concept-document⑨"></a>[document](#concept-document), then <a id="ref-for-dfn-throw②②"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror①①"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException②⑧"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

6.  If <var>newParent</var> is a <a id="ref-for-concept-document①⓪"></a>[document](#concept-document), <var>node</var> is an <code><a id="ref-for-element①⑥"></a>[Element](#element)</code> <a id="ref-for-concept-node⑦①"></a>[node](#concept-node), and either <var>newParent</var> has an <a id="ref-for-concept-element①④"></a>[element](#concept-element) <a id="ref-for-concept-tree-child②④"></a>[child](#concept-tree-child), <var>child</var> is a <a id="ref-for-concept-doctype⑥"></a>[doctype](#concept-doctype), or <var>child</var> is non-null and a <a id="ref-for-concept-doctype⑦"></a>[doctype](#concept-doctype) is <a id="ref-for-concept-tree-following③"></a>[following](#concept-tree-following) <var>child</var> then <a id="ref-for-dfn-throw②③"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror①②"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException②⑨"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

7.  Let <var>oldParent</var> be <var>node</var>’s <a id="ref-for-concept-tree-parent①②"></a>[parent](#concept-tree-parent).

8.  <a id="ref-for-assert③"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>oldParent</var> is non-null.

9.  Run the <a id="ref-for-live-range-pre-remove-steps"></a>[live range pre-remove steps](https://dom.spec.whatwg.org/#live-range-pre-remove-steps), given <var>node</var>.

10. For each <code><a id="ref-for-nodeiterator"></a>[NodeIterator](https://dom.spec.whatwg.org/#nodeiterator)</code> object <var>iterator</var> whose <a id="ref-for-concept-traversal-root"></a>[root](https://dom.spec.whatwg.org/#concept-traversal-root)’s <a id="ref-for-concept-node-document⑦"></a>[node document](#concept-node-document) is <var>node</var>’s <a id="ref-for-concept-node-document⑧"></a>[node document](#concept-node-document): run the <a id="ref-for-nodeiterator-pre-removing-steps"></a>[<code>NodeIterator</code> pre-remove steps](https://dom.spec.whatwg.org/#nodeiterator-pre-removing-steps) given <var>node</var> and <var>iterator</var>.

11. Let <var>oldPreviousSibling</var> be <var>node</var>’s <a id="ref-for-concept-tree-previous-sibling①"></a>[previous sibling](#concept-tree-previous-sibling).

12. Let <var>oldNextSibling</var> be <var>node</var>’s <a id="ref-for-concept-tree-next-sibling①"></a>[next sibling](#concept-tree-next-sibling).

13. <a id="ref-for-list-remove②"></a>[Remove](https://infra.spec.whatwg.org/#list-remove) <var>node</var> from <var>oldParent</var>’s <a id="ref-for-concept-tree-child②⑤"></a>[children](#concept-tree-child).

14. If <var>node</var> is <a id="ref-for-slotable-assigned③"></a>[assigned](#slotable-assigned), then run <a id="ref-for-assign-slotables③"></a>[assign slottables](#assign-slotables) for <var>node</var>’s <a id="ref-for-slotable-assigned-slot③"></a>[assigned slot](#slotable-assigned-slot).

15. If <var>oldParent</var>’s <a id="ref-for-concept-tree-root①⑦"></a>[root](#concept-tree-root) is a <a id="ref-for-concept-shadow-root①⑤"></a>[shadow root](#concept-shadow-root), and <var>oldParent</var> is a <a id="ref-for-concept-slot②①"></a>[slot](#concept-slot) whose <a id="ref-for-slot-assigned-nodes③"></a>[assigned nodes](#slot-assigned-nodes) <a id="ref-for-list-is-empty①"></a>[is empty](https://infra.spec.whatwg.org/#list-is-empty), then run <a id="ref-for-signal-a-slot-change②"></a>[signal a slot change](#signal-a-slot-change) for <var>oldParent</var>.

16. If <var>node</var> has an <a id="ref-for-concept-tree-inclusive-descendant①"></a>[inclusive descendant](#concept-tree-inclusive-descendant) that is a <a id="ref-for-concept-slot②②"></a>[slot](#concept-slot):

    1.  Run <a id="ref-for-assign-slotables-for-a-tree②"></a>[assign slottables for a tree](#assign-slotables-for-a-tree) with <var>oldParent</var>’s <a id="ref-for-concept-tree-root①⑧"></a>[root](#concept-tree-root).

    2.  Run <a id="ref-for-assign-slotables-for-a-tree③"></a>[assign slottables for a tree](#assign-slotables-for-a-tree) with <var>node</var>.

17. <a id="ref-for-add-transient-registered-observers"></a>[Add transient registered observers](https://dom.spec.whatwg.org/#add-transient-registered-observers) given <var>node</var> and <var>oldParent</var>.

18. If <var>child</var> is non-null:

    1.  For each <a id="ref-for-concept-live-range②"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-start-node①"></a>[start node](https://dom.spec.whatwg.org/#concept-range-start-node) is <var>newParent</var> and <a id="ref-for-concept-range-start-offset②"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) is greater than <var>child</var>’s <a id="ref-for-concept-tree-index③"></a>[index](#concept-tree-index): increase its <a id="ref-for-concept-range-start-offset③"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) by 1.

    2.  For each <a id="ref-for-concept-live-range③"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-end-node①"></a>[end node](https://dom.spec.whatwg.org/#concept-range-end-node) is <var>newParent</var> and <a id="ref-for-concept-range-end-offset②"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) is greater than <var>child</var>’s <a id="ref-for-concept-tree-index④"></a>[index](#concept-tree-index): increase its <a id="ref-for-concept-range-end-offset③"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) by 1.

19. Let <var>newPreviousSibling</var> be <var>child</var>’s <a id="ref-for-concept-tree-previous-sibling②"></a>[previous sibling](#concept-tree-previous-sibling) if <var>child</var> is non-null, and <var>newParent</var>’s <a id="ref-for-concept-tree-last-child①"></a>[last child](#concept-tree-last-child) otherwise.

20. If <var>child</var> is null, then <a id="ref-for-set-append①⓪"></a>[append](https://infra.spec.whatwg.org/#set-append) <var>node</var> to <var>newParent</var>’s <a id="ref-for-concept-tree-child②⑥"></a>[children](#concept-tree-child).

21. Otherwise, <a id="ref-for-list-insert①"></a>[insert](https://infra.spec.whatwg.org/#list-insert) <var>node</var> into <var>newParent</var>’s <a id="ref-for-concept-tree-child②⑦"></a>[children](#concept-tree-child) before <var>child</var>’s <a id="ref-for-concept-tree-index⑤"></a>[index](#concept-tree-index).

22. If <var>newParent</var> is a <a id="ref-for-element-shadow-host①"></a>[shadow host](#element-shadow-host) whose <a id="ref-for-concept-shadow-root①⑥"></a>[shadow root](#concept-shadow-root)’s <a id="ref-for-shadowroot-slot-assignment③"></a>[slot assignment](#shadowroot-slot-assignment) is "<code>named</code>" and <var>node</var> is a <a id="ref-for-concept-slotable①⑦"></a>[slottable](#concept-slotable), then <a id="ref-for-assign-a-slot②"></a>[assign a slot](#assign-a-slot) for <var>node</var>.

23. If <var>newParent</var>’s <a id="ref-for-concept-tree-root①⑨"></a>[root](#concept-tree-root) is a <a id="ref-for-concept-shadow-root①⑦"></a>[shadow root](#concept-shadow-root), and <var>newParent</var> is a <a id="ref-for-concept-slot②③"></a>[slot](#concept-slot) whose <a id="ref-for-slot-assigned-nodes④"></a>[assigned nodes](#slot-assigned-nodes) <a id="ref-for-list-is-empty②"></a>[is empty](https://infra.spec.whatwg.org/#list-is-empty), then run <a id="ref-for-signal-a-slot-change③"></a>[signal a slot change](#signal-a-slot-change) for <var>newParent</var>.

24. Run <a id="ref-for-assign-slotables-for-a-tree④"></a>[assign slottables for a tree](#assign-slotables-for-a-tree) with <var>node</var>’s <a id="ref-for-concept-tree-root②⓪"></a>[root](#concept-tree-root).

25. For each <a id="ref-for-concept-shadow-including-inclusive-descendant②"></a>[shadow-including inclusive descendant](#concept-shadow-including-inclusive-descendant) <var>inclusiveDescendant</var> of <var>node</var>, in <a id="ref-for-concept-shadow-including-tree-order②"></a>[shadow-including tree order](#concept-shadow-including-tree-order):

    1.  Let <var>isSubtreeRoot</var> be true if <var>inclusiveDescendant</var> is <var>node</var>; otherwise false.

    2.  Run the <a id="ref-for-concept-node-move-ext"></a>[moving steps](#concept-node-move-ext) with <var>inclusiveDescendant</var>, <var>isSubtreeRoot</var>, and <var>oldParent</var>.

        <strong>Note:</strong>

        Because the <a id="ref-for-move①"></a>[move](#move) algorithm is a separate primitive from <a id="ref-for-concept-node-insert⑥"></a>[insert](#concept-node-insert) and <a id="ref-for-concept-node-remove②"></a>[remove](#concept-node-remove), it does not invoke the <a id="ref-for-concept-node-insert-ext⑤"></a>[insertion steps](#concept-node-insert-ext) or <a id="ref-for-concept-node-remove-ext"></a>[removing steps](#concept-node-remove-ext) for <var>inclusiveDescendant</var>.

    3.  If <var>inclusiveDescendant</var> is <a id="ref-for-concept-element-custom①"></a>[custom](#concept-element-custom) and <var>newParent</var> is <a id="ref-for-connected④"></a>[connected](#connected), then <a id="ref-for-enqueue-a-custom-element-callback-reaction①"></a>[enqueue a custom element callback reaction](https://html.spec.whatwg.org/multipage/custom-elements.html#enqueue-a-custom-element-callback-reaction) with <var>inclusiveDescendant</var>, callback name "<code>connectedMoveCallback</code>", and « ».

26. <a id="ref-for-queue-a-tree-mutation-record②"></a>[Queue a tree mutation record](https://dom.spec.whatwg.org/#queue-a-tree-mutation-record) for <var>oldParent</var> with « », « <var>node</var> », <var>oldPreviousSibling</var>, and <var>oldNextSibling</var>.

27. <a id="ref-for-queue-a-tree-mutation-record③"></a>[Queue a tree mutation record](https://dom.spec.whatwg.org/#queue-a-tree-mutation-record) for <var>newParent</var> with « <var>node</var> », « », <var>newPreviousSibling</var>, and <var>child</var>.

------------------------------------------------------------------------

To <a id="concept-node-replace"></a><strong>replace</strong> a <a id="ref-for-concept-node⑦②"></a>[node](#concept-node) <var>child</var> with a <a id="ref-for-concept-node⑦③"></a>[node](#concept-node) <var>node</var> within a <a id="ref-for-concept-node⑦④"></a>[node](#concept-node) <var>parent</var>:

1.  <a id="ref-for-concept-node-ensure-pre-insertion-validity①"></a>[Ensure pre-insert validity](#concept-node-ensure-pre-insertion-validity) given <var>node</var>, <var>parent</var>, <var>child</var>, and « <var>child</var> ».

2.  Let <var>referenceChild</var> be <var>child</var>’s <a id="ref-for-concept-tree-next-sibling②"></a>[next sibling](#concept-tree-next-sibling).

3.  If <var>referenceChild</var> is <var>node</var>, then set <var>referenceChild</var> to <var>node</var>’s <a id="ref-for-concept-tree-next-sibling③"></a>[next sibling](#concept-tree-next-sibling).

4.  Let <var>previousSibling</var> be <var>child</var>’s <a id="ref-for-concept-tree-previous-sibling③"></a>[previous sibling](#concept-tree-previous-sibling).

5.  Let <var>removedNodes</var> be the empty set.

6.  <a id="ref-for-concept-node-adopt①"></a>[Adopt](#concept-node-adopt) <var>node</var> into <var>parent</var>’s <a id="ref-for-concept-node-document⑨"></a>[node document](#concept-node-document).

7.  If <var>child</var>’s <a id="ref-for-concept-tree-parent①③"></a>[parent](#concept-tree-parent) is non-null:

    1.  Set <var>removedNodes</var> to « <var>child</var> ».

    2.  <a id="ref-for-concept-node-remove③"></a>[Remove](#concept-node-remove) <var>child</var> with <a id="ref-for-remove-suppressobservers①"></a>[<em>suppressObservers</em>](#remove-suppressobservers) set to true.

    <strong>Note:</strong>

    The above can only be false if <var>child</var> is <var>node</var>.

8.  Let <var>nodes</var> be <var>node</var>’s <a id="ref-for-concept-tree-child②⑧"></a>[children](#concept-tree-child) if <var>node</var> is a <code><a id="ref-for-documentfragment①⓪"></a>[DocumentFragment](#documentfragment)</code> <a id="ref-for-concept-node⑦⑤"></a>[node](#concept-node); otherwise « <var>node</var> ».

9.  <a id="ref-for-concept-node-insert⑦"></a>[Insert](#concept-node-insert) <var>node</var> into <var>parent</var> before <var>referenceChild</var> with <a id="ref-for-insert-suppressobservers"></a>[<em>suppressObservers</em>](#insert-suppressobservers) set to true.

10. <a id="ref-for-queue-a-tree-mutation-record④"></a>[Queue a tree mutation record](https://dom.spec.whatwg.org/#queue-a-tree-mutation-record) for <var>parent</var> with <var>nodes</var>, <var>removedNodes</var>, <var>previousSibling</var>, and <var>referenceChild</var>.

11. Return <var>child</var>.

To <a id="concept-node-replace-all"></a><strong>replace all</strong> with a <a id="ref-for-concept-node⑦⑥"></a>[node](#concept-node) or null <var>node</var> within a <a id="ref-for-concept-node⑦⑦"></a>[node](#concept-node) <var>parent</var>:

1.  Let <var>removedNodes</var> be <var>parent</var>’s <a id="ref-for-concept-tree-child②⑨"></a>[children](#concept-tree-child).

2.  Let <var>addedNodes</var> be the empty set.

3.  If <var>node</var> is a <code><a id="ref-for-documentfragment①①"></a>[DocumentFragment](#documentfragment)</code> <a id="ref-for-concept-node⑦⑧"></a>[node](#concept-node), then set <var>addedNodes</var> to <var>node</var>’s <a id="ref-for-concept-tree-child③⓪"></a>[children](#concept-tree-child).

4.  Otherwise, if <var>node</var> is non-null, set <var>addedNodes</var> to « <var>node</var> ».

5.  <a id="ref-for-concept-node-remove④"></a>[Remove](#concept-node-remove) all <var>parent</var>’s <a id="ref-for-concept-tree-child③①"></a>[children](#concept-tree-child), in <a id="ref-for-concept-tree-order①③"></a>[tree order](#concept-tree-order), with <a id="ref-for-remove-suppressobservers②"></a>[<em>suppressObservers</em>](#remove-suppressobservers) set to true.

6.  If <var>node</var> is non-null, then <a id="ref-for-concept-node-insert⑧"></a>[insert](#concept-node-insert) <var>node</var> into <var>parent</var> before null with <a id="ref-for-insert-suppressobservers①"></a>[<em>suppressObservers</em>](#insert-suppressobservers) set to true.

7.  If either <var>addedNodes</var> or <var>removedNodes</var> <a id="ref-for-list-is-empty③"></a>[is not empty](https://infra.spec.whatwg.org/#list-is-empty), then <a id="ref-for-queue-a-tree-mutation-record⑤"></a>[queue a tree mutation record](https://dom.spec.whatwg.org/#queue-a-tree-mutation-record) for <var>parent</var> with <var>addedNodes</var>, <var>removedNodes</var>, null, and null.

<strong>Note:</strong>

This algorithm does not make any checks with regards to the <a id="ref-for-concept-node-tree①③"></a>[node tree](#concept-node-tree) constraints. Specification authors need to use it wisely.

------------------------------------------------------------------------

To <a id="concept-node-pre-remove"></a><strong>pre-remove</strong> a <a id="ref-for-concept-node⑦⑨"></a>[node](#concept-node) <var>child</var> from a <a id="ref-for-concept-node⑧⓪"></a>[node](#concept-node) <var>parent</var>:

1.  If <var>child</var>’s <a id="ref-for-concept-tree-parent①④"></a>[parent](#concept-tree-parent) is not <var>parent</var>, then <a id="ref-for-dfn-throw②④"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notfounderror②"></a>[NotFoundError](https://webidl.spec.whatwg.org/#notfounderror)</code>" <code><a id="ref-for-idl-DOMException③⓪"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  <a id="ref-for-concept-node-remove⑤"></a>[Remove](#concept-node-remove) <var>child</var>.

3.  Return <var>child</var>.

<a id="ref-for-other-applicable-specifications⑤"></a>[Specifications](https://dom.spec.whatwg.org/#other-applicable-specifications) may define <a id="concept-node-remove-ext"></a><strong>removing steps</strong> for all or some <a id="ref-for-concept-node⑧①"></a>[nodes](#concept-node). The algorithm is passed a <a id="ref-for-concept-node⑧②"></a>[node](#concept-node) <var>removedNode</var>, a boolean <var>isSubtreeRoot</var>, and a <a id="ref-for-concept-node⑧③"></a>[node](#concept-node) <var>oldAncestor</var> as indicated in the <a id="ref-for-concept-node-remove⑥"></a>[remove](#concept-node-remove) algorithm below.

To <a id="concept-node-remove"></a><strong>remove</strong> a <a id="ref-for-concept-node⑧④"></a>[node](#concept-node) <var>node</var>, with an optional boolean <a id="remove-suppressobservers"></a><strong><var>suppressObservers</var></strong> (default false):

1.  Let <var>parent</var> be <var>node</var>’s <a id="ref-for-concept-tree-parent①⑤"></a>[parent](#concept-tree-parent).

2.  Assert: <var>parent</var> is non-null.

3.  Run the <a id="ref-for-live-range-pre-remove-steps①"></a>[live range pre-remove steps](https://dom.spec.whatwg.org/#live-range-pre-remove-steps), given <var>node</var>.

4.  For each <code><a id="ref-for-nodeiterator①"></a>[NodeIterator](https://dom.spec.whatwg.org/#nodeiterator)</code> object <var>iterator</var> whose <a id="ref-for-concept-traversal-root①"></a>[root](https://dom.spec.whatwg.org/#concept-traversal-root)’s <a id="ref-for-concept-node-document①⓪"></a>[node document](#concept-node-document) is <var>node</var>’s <a id="ref-for-concept-node-document①①"></a>[node document](#concept-node-document): run the <a id="ref-for-nodeiterator-pre-removing-steps①"></a>[<code>NodeIterator</code> pre-remove steps](https://dom.spec.whatwg.org/#nodeiterator-pre-removing-steps) given <var>node</var> and <var>iterator</var>.

5.  Let <var>oldPreviousSibling</var> be <var>node</var>’s <a id="ref-for-concept-tree-previous-sibling④"></a>[previous sibling](#concept-tree-previous-sibling).

6.  Let <var>oldNextSibling</var> be <var>node</var>’s <a id="ref-for-concept-tree-next-sibling④"></a>[next sibling](#concept-tree-next-sibling).

7.  <a id="ref-for-list-remove③"></a>[Remove](https://infra.spec.whatwg.org/#list-remove) <var>node</var> from its <var>parent</var>’s <a id="ref-for-concept-tree-child③②"></a>[children](#concept-tree-child).

8.  If <var>node</var> is <a id="ref-for-slotable-assigned④"></a>[assigned](#slotable-assigned), then run <a id="ref-for-assign-slotables④"></a>[assign slottables](#assign-slotables) for <var>node</var>’s <a id="ref-for-slotable-assigned-slot④"></a>[assigned slot](#slotable-assigned-slot).

9.  If <var>parent</var>’s <a id="ref-for-concept-tree-root②①"></a>[root](#concept-tree-root) is a <a id="ref-for-concept-shadow-root①⑧"></a>[shadow root](#concept-shadow-root), and <var>parent</var> is a <a id="ref-for-concept-slot②④"></a>[slot](#concept-slot) whose <a id="ref-for-slot-assigned-nodes⑤"></a>[assigned nodes](#slot-assigned-nodes) is the empty list, then run <a id="ref-for-signal-a-slot-change④"></a>[signal a slot change](#signal-a-slot-change) for <var>parent</var>.

10. If <var>node</var> has an <a id="ref-for-concept-tree-inclusive-descendant②"></a>[inclusive descendant](#concept-tree-inclusive-descendant) that is a <a id="ref-for-concept-slot②⑤"></a>[slot](#concept-slot):

    1.  Run <a id="ref-for-assign-slotables-for-a-tree⑤"></a>[assign slottables for a tree](#assign-slotables-for-a-tree) with <var>parent</var>’s <a id="ref-for-concept-tree-root②②"></a>[root](#concept-tree-root).

    2.  Run <a id="ref-for-assign-slotables-for-a-tree⑥"></a>[assign slottables for a tree](#assign-slotables-for-a-tree) with <var>node</var>.

11. Run the <a id="ref-for-concept-node-remove-ext①"></a>[removing steps](#concept-node-remove-ext) with <var>node</var>, true, and <var>parent</var>.

12. Let <var>isParentConnected</var> be <var>parent</var>’s <a id="ref-for-connected⑤"></a>[connected](#connected).

13. If <var>node</var> is <a id="ref-for-concept-element-custom②"></a>[custom](#concept-element-custom) and <var>isParentConnected</var> is true, then <a id="ref-for-enqueue-a-custom-element-callback-reaction②"></a>[enqueue a custom element callback reaction](https://html.spec.whatwg.org/multipage/custom-elements.html#enqueue-a-custom-element-callback-reaction) with <var>node</var>, callback name "<code>disconnectedCallback</code>", and « ».

    <strong>Note:</strong>

    It is intentional for now that <a id="ref-for-concept-element-custom③"></a>[custom](#concept-element-custom) <a id="ref-for-concept-element①⑤"></a>[elements](#concept-element) do not get <var>parent</var> passed. This might change in the future if there is a need.

14. For each <a id="ref-for-concept-shadow-including-descendant"></a>[shadow-including descendant](#concept-shadow-including-descendant) <var>descendant</var> of <var>node</var>, in <a id="ref-for-concept-shadow-including-tree-order③"></a>[shadow-including tree order](#concept-shadow-including-tree-order):

    1.  Run the <a id="ref-for-concept-node-remove-ext②"></a>[removing steps](#concept-node-remove-ext) with <var>descendant</var>, false, and <var>parent</var>.

    2.  If <var>descendant</var> is <a id="ref-for-concept-element-custom④"></a>[custom](#concept-element-custom) and <var>isParentConnected</var> is true, then <a id="ref-for-enqueue-a-custom-element-callback-reaction③"></a>[enqueue a custom element callback reaction](https://html.spec.whatwg.org/multipage/custom-elements.html#enqueue-a-custom-element-callback-reaction) with <var>descendant</var>, callback name "<code>disconnectedCallback</code>", and « ».

15. <a id="ref-for-add-transient-registered-observers①"></a>[Add transient registered observers](https://dom.spec.whatwg.org/#add-transient-registered-observers) given <var>node</var> and <var>parent</var>.

16. If <var>suppressObservers</var> is false, then <a id="ref-for-queue-a-tree-mutation-record⑥"></a>[queue a tree mutation record](https://dom.spec.whatwg.org/#queue-a-tree-mutation-record) for <var>parent</var> with « », « <var>node</var> », <var>oldPreviousSibling</var>, and <var>oldNextSibling</var>.

17. Run the <a id="ref-for-concept-node-children-changed-ext①"></a>[children changed steps](#concept-node-children-changed-ext) for <var>parent</var>.

#### <a id="interface-nonelementparentnode"></a>4.2.4. Mixin <code><a id="ref-for-nonelementparentnode"></a>[NonElementParentNode](#nonelementparentnode)</code>[](#interface-nonelementparentnode)

<strong>Note:</strong>

Web compatibility prevents the <code><a id="ref-for-dom-nonelementparentnode-getelementbyid"></a>[getElementById()](#dom-nonelementparentnode-getelementbyid)</code> method from being exposed on <a id="ref-for-concept-element①⑥"></a>[elements](#concept-element) (and therefore on <code><a id="ref-for-parentnode"></a>[ParentNode](#parentnode)</code>).

<a id="nonelementparentnode"></a><a id="ref-for-element①⑦"></a><a id="ref-for-dom-nonelementparentnode-getelementbyid①"></a><a id="ref-for-idl-DOMString⑦"></a><a id="dom-nonelementparentnode-getelementbyid-elementid-elementid"></a><a id="ref-for-document④"></a><a id="ref-for-nonelementparentnode①"></a><a id="ref-for-documentfragment①②"></a><a id="ref-for-nonelementparentnode②"></a>

``` text
interface mixin NonElementParentNode {
  Element? getElementById(DOMString elementId);
};
Document includes NonElementParentNode;
DocumentFragment includes NonElementParentNode;
```

<strong><code><var>node</var> . <a id="ref-for-dom-nonelementparentnode-getelementbyid②"></a>[getElementById](#dom-nonelementparentnode-getelementbyid)(<var>elementId</var>)</code></strong>

Returns the first <a id="ref-for-concept-element①⑦"></a>[element](#concept-element) within <var>node</var>’s <a id="ref-for-concept-tree-descendant⑤"></a>[descendants](#concept-tree-descendant) whose <a id="ref-for-concept-id"></a>[ID](#concept-id) is <var>elementId</var>.

To <a id="get-an-element-by-id"></a><strong>get an element by ID</strong>, given a <a id="ref-for-concept-node⑧⑤"></a>[node](#concept-node) <var>node</var> and a string <var>elementId</var>, return the first <a id="ref-for-concept-element①⑧"></a>[element](#concept-element), in <a id="ref-for-concept-tree-order①④"></a>[tree order](#concept-tree-order), within <var>node</var>’s <a id="ref-for-concept-tree-descendant⑥"></a>[descendants](#concept-tree-descendant), whose <a id="ref-for-concept-id①"></a>[ID](#concept-id) is <var>elementId</var>; otherwise, if there is no such <a id="ref-for-concept-element①⑨"></a>[element](#concept-element), null.

The <a id="dom-nonelementparentnode-getelementbyid"></a><strong><code>getElementById(<var>elementId</var>)</code></strong> method steps are to return the result of <a id="ref-for-get-an-element-by-id"></a>[getting an element by ID](#get-an-element-by-id) given <a id="ref-for-this③②"></a>[this](https://webidl.spec.whatwg.org/#this) and <var>elementId</var>.

#### <a id="mixin-documentorshadowroot"></a>4.2.5. Mixin <code><a id="ref-for-documentorshadowroot"></a>[DocumentOrShadowRoot](#documentorshadowroot)</code>[](#mixin-documentorshadowroot)

<a id="documentorshadowroot"></a><a id="ref-for-customelementregistry"></a><a id="ref-for-dom-documentorshadowroot-customelementregistry"></a><a id="ref-for-document⑤"></a><a id="ref-for-documentorshadowroot①"></a><a id="ref-for-shadowroot③"></a><a id="ref-for-documentorshadowroot②"></a>

``` text
interface mixin DocumentOrShadowRoot {
  readonly attribute CustomElementRegistry? customElementRegistry;
};
Document includes DocumentOrShadowRoot;
ShadowRoot includes DocumentOrShadowRoot;
```

<strong><code><var>registry</var> = <var>documentOrShadowRoot</var> . <code><a id="ref-for-dom-documentorshadowroot-customelementregistry①"></a>[customElementRegistry](#dom-documentorshadowroot-customelementregistry)</code></code></strong>

Returns <var>documentOrShadowRoot</var>’s <code><a id="ref-for-customelementregistry①"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code> object, if any; otherwise null.

The <a id="dom-documentorshadowroot-customelementregistry"></a><strong><code>customElementRegistry</code></strong> getter steps are:

1.  If <a id="ref-for-this③③"></a>[this](https://webidl.spec.whatwg.org/#this) is a <a id="ref-for-concept-document①①"></a>[document](#concept-document), then return <a id="ref-for-this③④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-document-custom-element-registry"></a>[custom element registry](#document-custom-element-registry).

2.  <a id="ref-for-assert④"></a>[Assert](https://infra.spec.whatwg.org/#assert): <a id="ref-for-this③⑤"></a>[this](https://webidl.spec.whatwg.org/#this) is a <code><a id="ref-for-shadowroot④"></a>[ShadowRoot](#shadowroot)</code> <a id="ref-for-concept-node⑧⑥"></a>[node](#concept-node).

3.  Return <a id="ref-for-this③⑥"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-shadowroot-custom-element-registry③"></a>[custom element registry](#shadowroot-custom-element-registry).

<strong>Note:</strong>

The <code><a id="ref-for-documentorshadowroot③"></a>[DocumentOrShadowRoot](#documentorshadowroot)</code> mixin is also expected to be used by other standards that want to define APIs shared between <a id="ref-for-concept-document①②"></a>[documents](#concept-document) and <a id="ref-for-concept-shadow-root①⑨"></a>[shadow roots](#concept-shadow-root).

#### <a id="interface-parentnode"></a>4.2.6. Mixin <code><a id="ref-for-parentnode①"></a>[ParentNode](#parentnode)</code>[](#interface-parentnode)

To <a id="convert-nodes-into-a-node"></a><strong>convert nodes into a node</strong>, given a <a id="ref-for-list①③"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-concept-node⑧⑦"></a>[nodes](#concept-node) and strings <var>nodes</var>, and <a id="ref-for-concept-document①③"></a>[document](#concept-document) <var>document</var>:

1.  Replace each string in <var>nodes</var> with the result of <a id="ref-for-create-a-text-node"></a>[creating a text node](#create-a-text-node) given <var>document</var> and the string.

2.  If <var>nodes</var>’s <a id="ref-for-list-size③"></a>[size](https://infra.spec.whatwg.org/#list-size) is 1, then return <var>nodes</var>\[0\].

3.  Let <var>fragment</var> be the result of <a id="ref-for-create-a-document-fragment"></a>[creating a document fragment](#create-a-document-fragment) given <var>document</var>.

4.  For each <var>node</var> of <var>nodes</var>: <a id="ref-for-concept-node-append"></a>[append](#concept-node-append) <var>node</var> to <var>fragment</var>.

5.  Return <var>fragment</var>.

<a id="parentnode"></a><a id="ref-for-SameObject①"></a><a id="ref-for-htmlcollection"></a><a id="ref-for-dom-parentnode-children"></a><a id="ref-for-element①⑧"></a><a id="ref-for-dom-parentnode-firstelementchild"></a><a id="ref-for-element①⑨"></a><a id="ref-for-dom-parentnode-lastelementchild"></a><a id="ref-for-idl-unsigned-long"></a><a id="ref-for-dom-parentnode-childelementcount"></a><a id="ref-for-cereactions"></a><a id="ref-for-Unscopable"></a><a id="ref-for-idl-undefined①①"></a><a id="ref-for-dom-parentnode-prepend"></a><a id="ref-for-node③"></a><a id="ref-for-idl-DOMString⑧"></a><a id="dom-parentnode-prepend-nodes-nodes"></a><a id="ref-for-cereactions①"></a><a id="ref-for-Unscopable①"></a><a id="ref-for-idl-undefined①②"></a><a id="ref-for-dom-parentnode-append"></a><a id="ref-for-node④"></a><a id="ref-for-idl-DOMString⑨"></a><a id="dom-parentnode-append-nodes-nodes"></a><a id="ref-for-cereactions②"></a><a id="ref-for-Unscopable②"></a><a id="ref-for-idl-undefined①③"></a><a id="ref-for-dom-parentnode-replacechildren"></a><a id="ref-for-node⑤"></a><a id="ref-for-idl-DOMString①⓪"></a><a id="dom-parentnode-replacechildren-nodes-nodes"></a><a id="ref-for-cereactions③"></a><a id="ref-for-idl-undefined①④"></a><a id="ref-for-dom-parentnode-movebefore"></a><a id="ref-for-node⑥"></a><a id="dom-parentnode-movebefore-node-child-node"></a><a id="ref-for-node⑦"></a><a id="dom-parentnode-movebefore-node-child-child"></a><a id="ref-for-element②⓪"></a><a id="ref-for-dom-parentnode-queryselector"></a><a id="ref-for-idl-DOMString①①"></a><a id="dom-parentnode-queryselector-selectors-selectors"></a><a id="ref-for-NewObject③"></a><a id="ref-for-nodelist"></a><a id="ref-for-dom-parentnode-queryselectorall"></a><a id="ref-for-idl-DOMString①②"></a><a id="dom-parentnode-queryselectorall-selectors-selectors"></a><a id="ref-for-document⑥"></a><a id="ref-for-parentnode②"></a><a id="ref-for-documentfragment①③"></a><a id="ref-for-parentnode③"></a><a id="ref-for-element②①"></a><a id="ref-for-parentnode④"></a>

``` text
interface mixin ParentNode {
  [SameObject] readonly attribute HTMLCollection children;
  readonly attribute Element? firstElementChild;
  readonly attribute Element? lastElementChild;
  readonly attribute unsigned long childElementCount;

  [CEReactions, Unscopable] undefined prepend((Node or DOMString)... nodes);
  [CEReactions, Unscopable] undefined append((Node or DOMString)... nodes);
  [CEReactions, Unscopable] undefined replaceChildren((Node or DOMString)... nodes);

  [CEReactions] undefined moveBefore(Node node, Node? child);

  Element? querySelector(DOMString selectors);
  [NewObject] NodeList querySelectorAll(DOMString selectors);
};
Document includes ParentNode;
DocumentFragment includes ParentNode;
Element includes ParentNode;
```

<strong><code><var>collection</var> = <var>node</var> . <code><a id="ref-for-dom-parentnode-children①"></a>[children](#dom-parentnode-children)</code></code></strong>

Returns the <a id="ref-for-concept-tree-child③③"></a>[child](#concept-tree-child) <a id="ref-for-concept-element②⓪"></a>[elements](#concept-element).

<strong><code><var>element</var> = <var>node</var> . <code><a id="ref-for-dom-parentnode-firstelementchild①"></a>[firstElementChild](#dom-parentnode-firstelementchild)</code></code></strong>

Returns the first <a id="ref-for-concept-tree-child③④"></a>[child](#concept-tree-child) that is an <a id="ref-for-concept-element②①"></a>[element](#concept-element); otherwise null.

<strong><code><var>element</var> = <var>node</var> . <code><a id="ref-for-dom-parentnode-lastelementchild①"></a>[lastElementChild](#dom-parentnode-lastelementchild)</code></code></strong>

Returns the last <a id="ref-for-concept-tree-child③⑤"></a>[child](#concept-tree-child) that is an <a id="ref-for-concept-element②②"></a>[element](#concept-element); otherwise null.

<strong><code><var>node</var> . <a id="ref-for-dom-parentnode-prepend①"></a>[prepend](#dom-parentnode-prepend)(<var>nodes</var>)</code></strong>

Inserts <var>nodes</var> before the <a id="ref-for-concept-tree-first-child"></a>[first child](#concept-tree-first-child) of <var>node</var>, while replacing strings in <var>nodes</var> with equivalent <code><a id="ref-for-text①①"></a>[Text](#text)</code> <a id="ref-for-concept-node⑧⑧"></a>[nodes](#concept-node).

<a id="ref-for-dfn-throw②⑤"></a>[Throws](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror①③"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException③①"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> if the constraints of the <a id="ref-for-concept-node-tree①④"></a>[node tree](#concept-node-tree) are violated.

<strong><code><var>node</var> . <a id="ref-for-dom-parentnode-append①"></a>[append](#dom-parentnode-append)(<var>nodes</var>)</code></strong>

Inserts <var>nodes</var> after the <a id="ref-for-concept-tree-last-child②"></a>[last child](#concept-tree-last-child) of <var>node</var>, while replacing strings in <var>nodes</var> with equivalent <code><a id="ref-for-text①②"></a>[Text](#text)</code> <a id="ref-for-concept-node⑧⑨"></a>[nodes](#concept-node).

<a id="ref-for-dfn-throw②⑥"></a>[Throws](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror①④"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException③②"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> if the constraints of the <a id="ref-for-concept-node-tree①⑤"></a>[node tree](#concept-node-tree) are violated.

<strong><code><var>node</var> . <a id="ref-for-dom-parentnode-replacechildren①"></a>[replaceChildren](#dom-parentnode-replacechildren)(<var>nodes</var>)</code></strong>

Replace all <a id="ref-for-concept-tree-child③⑥"></a>[children](#concept-tree-child) of <var>node</var> with <var>nodes</var>, while replacing strings in <var>nodes</var> with equivalent <code><a id="ref-for-text①③"></a>[Text](#text)</code> <a id="ref-for-concept-node⑨⓪"></a>[nodes](#concept-node).

<a id="ref-for-dfn-throw②⑦"></a>[Throws](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror①⑤"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException③③"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> if the constraints of the <a id="ref-for-concept-node-tree①⑥"></a>[node tree](#concept-node-tree) are violated.

<strong><code><var>node</var> . <a id="ref-for-dom-parentnode-movebefore①"></a>[moveBefore](#dom-parentnode-movebefore)(<var>movedNode</var>, <var>child</var>)</code></strong>

Moves, without first removing, <var>movedNode</var> into <var>node</var> after <var>child</var> if <var>child</var> is non-null; otherwise after the <a id="ref-for-concept-tree-last-child③"></a>[last child](#concept-tree-last-child) of <var>node</var>. This method preserves state associated with <var>movedNode</var>.

<a id="ref-for-dfn-throw②⑧"></a>[Throws](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror①⑥"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException③④"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> if the constraints of the <a id="ref-for-concept-node-tree①⑦"></a>[node tree](#concept-node-tree) are violated, or the state associated with the moved node cannot be preserved.

<strong><code><var>node</var> . <a id="ref-for-dom-parentnode-queryselector①"></a>[querySelector](#dom-parentnode-queryselector)(<var>selectors</var>)</code></strong>

Returns the first <a id="ref-for-concept-element②③"></a>[element](#concept-element) that is a <a id="ref-for-concept-tree-descendant⑦"></a>[descendant](#concept-tree-descendant) of <var>node</var> that matches <var>selectors</var>.

<strong><code><var>node</var> . <a id="ref-for-dom-parentnode-queryselectorall①"></a>[querySelectorAll](#dom-parentnode-queryselectorall)(<var>selectors</var>)</code></strong>

Returns all <a id="ref-for-concept-element②④"></a>[element](#concept-element) <a id="ref-for-concept-tree-descendant⑧"></a>[descendants](#concept-tree-descendant) of <var>node</var> that match <var>selectors</var>.

The <a id="dom-parentnode-children"></a><strong><code>children</code></strong> getter steps are to return an <code><a id="ref-for-htmlcollection①"></a>[HTMLCollection](#htmlcollection)</code> <a id="ref-for-concept-collection"></a>[collection](#concept-collection) rooted at <a id="ref-for-this③⑦"></a>[this](https://webidl.spec.whatwg.org/#this) matching only <a id="ref-for-concept-element②⑤"></a>[element](#concept-element) <a id="ref-for-concept-tree-child③⑦"></a>[children](#concept-tree-child).

The <a id="dom-parentnode-firstelementchild"></a><strong><code>firstElementChild</code></strong> getter steps are to return the first <a id="ref-for-concept-tree-child③⑧"></a>[child](#concept-tree-child) that is an <a id="ref-for-concept-element②⑥"></a>[element](#concept-element); otherwise null.

The <a id="dom-parentnode-lastelementchild"></a><strong><code>lastElementChild</code></strong> getter steps are to return the last <a id="ref-for-concept-tree-child③⑨"></a>[child](#concept-tree-child) that is an <a id="ref-for-concept-element②⑦"></a>[element](#concept-element); otherwise null.

The <a id="dom-parentnode-childelementcount"></a><strong><code>childElementCount</code></strong> getter steps are to return the number of <a id="ref-for-concept-tree-child④⓪"></a>[children](#concept-tree-child) of <a id="ref-for-this③⑧"></a>[this](https://webidl.spec.whatwg.org/#this) that are <a id="ref-for-concept-element②⑧"></a>[elements](#concept-element).

The <a id="dom-parentnode-prepend"></a><strong><code>prepend(<var>nodes</var>)</code></strong> method steps are:

1.  Let <var>node</var> be the result of <a id="ref-for-convert-nodes-into-a-node"></a>[converting nodes into a node](#convert-nodes-into-a-node) given <var>nodes</var> and <a id="ref-for-this③⑨"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document①②"></a>[node document](#concept-node-document).

2.  <a id="ref-for-concept-node-pre-insert①"></a>[Pre-insert](#concept-node-pre-insert) <var>node</var> into <a id="ref-for-this④⓪"></a>[this](https://webidl.spec.whatwg.org/#this) before <a id="ref-for-this④①"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-first-child①"></a>[first child](#concept-tree-first-child).

The <a id="dom-parentnode-append"></a><strong><code>append(<var>nodes</var>)</code></strong> method steps are:

1.  Let <var>node</var> be the result of <a id="ref-for-convert-nodes-into-a-node①"></a>[converting nodes into a node](#convert-nodes-into-a-node) given <var>nodes</var> and <a id="ref-for-this④②"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document①③"></a>[node document](#concept-node-document).

2.  <a id="ref-for-concept-node-append①"></a>[Append](#concept-node-append) <var>node</var> to <a id="ref-for-this④③"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-parentnode-replacechildren"></a><strong><code>replaceChildren(<var>nodes</var>)</code></strong> method steps are:

1.  Let <var>node</var> be the result of <a id="ref-for-convert-nodes-into-a-node②"></a>[converting nodes into a node](#convert-nodes-into-a-node) given <var>nodes</var> and <a id="ref-for-this④④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document①④"></a>[node document](#concept-node-document).

2.  <a id="ref-for-concept-node-ensure-pre-insertion-validity②"></a>[Ensure pre-insert validity](#concept-node-ensure-pre-insertion-validity) given <var>node</var>, <a id="ref-for-this④⑤"></a>[this](https://webidl.spec.whatwg.org/#this), null, and <a id="ref-for-this④⑥"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-child④①"></a>[children](#concept-tree-child).

3.  <a id="ref-for-concept-node-replace-all"></a>[Replace all](#concept-node-replace-all) with <var>node</var> within <a id="ref-for-this④⑦"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-parentnode-movebefore"></a><strong><code>moveBefore(<var>node</var>, <var>child</var>)</code></strong> method steps are:

1.  Let <var>referenceChild</var> be <var>child</var>.

2.  If <var>referenceChild</var> is <var>node</var>, then set <var>referenceChild</var> to <var>node</var>’s <a id="ref-for-concept-tree-next-sibling⑤"></a>[next sibling](#concept-tree-next-sibling).

3.  <a id="ref-for-move②"></a>[Move](#move) <var>node</var> into <a id="ref-for-this④⑧"></a>[this](https://webidl.spec.whatwg.org/#this) before <var>referenceChild</var>.

The <a id="dom-parentnode-queryselector"></a><strong><code>querySelector(<var>selectors</var>)</code></strong> method steps are to return the first result of running <a id="ref-for-scope-match-a-selectors-string"></a>[scope-match a selectors string](https://dom.spec.whatwg.org/#scope-match-a-selectors-string) <var>selectors</var> against <a id="ref-for-this④⑨"></a>[this](https://webidl.spec.whatwg.org/#this), if the result is not an empty list; otherwise null.

The <a id="dom-parentnode-queryselectorall"></a><strong><code>querySelectorAll(<var>selectors</var>)</code></strong> method steps are to return the <a id="ref-for-concept-collection-static"></a>[static](#concept-collection-static) result of running <a id="ref-for-scope-match-a-selectors-string①"></a>[scope-match a selectors string](https://dom.spec.whatwg.org/#scope-match-a-selectors-string) <var>selectors</var> against <a id="ref-for-this⑤⓪"></a>[this](https://webidl.spec.whatwg.org/#this).

#### <a id="interface-nondocumenttypechildnode"></a>4.2.7. Mixin <code><a id="ref-for-nondocumenttypechildnode"></a>[NonDocumentTypeChildNode](#nondocumenttypechildnode)</code>[](#interface-nondocumenttypechildnode)

<strong>Note:</strong>

Web compatibility prevents the <code><a id="ref-for-dom-nondocumenttypechildnode-previouselementsibling"></a>[previousElementSibling](#dom-nondocumenttypechildnode-previouselementsibling)</code> and <code><a id="ref-for-dom-nondocumenttypechildnode-nextelementsibling"></a>[nextElementSibling](#dom-nondocumenttypechildnode-nextelementsibling)</code> attributes from being exposed on <a id="ref-for-concept-doctype⑧"></a>[doctypes](#concept-doctype) (and therefore on <code><a id="ref-for-childnode"></a>[ChildNode](#childnode)</code>).

<a id="nondocumenttypechildnode"></a><a id="ref-for-element②②"></a><a id="ref-for-dom-nondocumenttypechildnode-previouselementsibling①"></a><a id="ref-for-element②③"></a><a id="ref-for-dom-nondocumenttypechildnode-nextelementsibling①"></a><a id="ref-for-element②④"></a><a id="ref-for-nondocumenttypechildnode①"></a><a id="ref-for-characterdata⑧"></a><a id="ref-for-nondocumenttypechildnode②"></a>

``` text
interface mixin NonDocumentTypeChildNode {
  readonly attribute Element? previousElementSibling;
  readonly attribute Element? nextElementSibling;
};
Element includes NonDocumentTypeChildNode;
CharacterData includes NonDocumentTypeChildNode;
```

<strong><code><var>element</var> = <var>node</var> . <code><a id="ref-for-dom-nondocumenttypechildnode-previouselementsibling②"></a>[previousElementSibling](#dom-nondocumenttypechildnode-previouselementsibling)</code></code></strong>

Returns the first <a id="ref-for-concept-tree-preceding④"></a>[preceding](#concept-tree-preceding) <a id="ref-for-concept-tree-sibling⑥"></a>[sibling](#concept-tree-sibling) that is an <a id="ref-for-concept-element②⑨"></a>[element](#concept-element); otherwise null.

<strong><code><var>element</var> = <var>node</var> . <code><a id="ref-for-dom-nondocumenttypechildnode-nextelementsibling②"></a>[nextElementSibling](#dom-nondocumenttypechildnode-nextelementsibling)</code></code></strong>

Returns the first <a id="ref-for-concept-tree-following④"></a>[following](#concept-tree-following) <a id="ref-for-concept-tree-sibling⑦"></a>[sibling](#concept-tree-sibling) that is an <a id="ref-for-concept-element③⓪"></a>[element](#concept-element); otherwise null.

The <a id="dom-nondocumenttypechildnode-previouselementsibling"></a><strong><code>previousElementSibling</code></strong> getter steps are to return the first <a id="ref-for-concept-tree-preceding⑤"></a>[preceding](#concept-tree-preceding) <a id="ref-for-concept-tree-sibling⑧"></a>[sibling](#concept-tree-sibling) that is an <a id="ref-for-concept-element③①"></a>[element](#concept-element); otherwise null.

The <a id="dom-nondocumenttypechildnode-nextelementsibling"></a><strong><code>nextElementSibling</code></strong> getter steps are to return the first <a id="ref-for-concept-tree-following⑤"></a>[following](#concept-tree-following) <a id="ref-for-concept-tree-sibling⑨"></a>[sibling](#concept-tree-sibling) that is an <a id="ref-for-concept-element③②"></a>[element](#concept-element); otherwise null.

#### <a id="interface-childnode"></a>4.2.8. Mixin <code><a id="ref-for-childnode①"></a>[ChildNode](#childnode)</code>[](#interface-childnode)

<a id="childnode"></a><a id="ref-for-cereactions④"></a><a id="ref-for-Unscopable③"></a><a id="ref-for-idl-undefined①⑤"></a><a id="ref-for-dom-childnode-before"></a><a id="ref-for-node⑧"></a><a id="ref-for-idl-DOMString①③"></a><a id="dom-childnode-before-nodes-nodes"></a><a id="ref-for-cereactions⑤"></a><a id="ref-for-Unscopable④"></a><a id="ref-for-idl-undefined①⑥"></a><a id="ref-for-dom-childnode-after"></a><a id="ref-for-node⑨"></a><a id="ref-for-idl-DOMString①④"></a><a id="dom-childnode-after-nodes-nodes"></a><a id="ref-for-cereactions⑥"></a><a id="ref-for-Unscopable⑤"></a><a id="ref-for-idl-undefined①⑦"></a><a id="ref-for-dom-childnode-replacewith"></a><a id="ref-for-node①⓪"></a><a id="ref-for-idl-DOMString①⑤"></a><a id="dom-childnode-replacewith-nodes-nodes"></a><a id="ref-for-cereactions⑦"></a><a id="ref-for-Unscopable⑥"></a><a id="ref-for-idl-undefined①⑧"></a><a id="ref-for-dom-childnode-remove"></a><a id="ref-for-documenttype⑥"></a><a id="ref-for-childnode②"></a><a id="ref-for-element②⑤"></a><a id="ref-for-childnode③"></a><a id="ref-for-characterdata⑨"></a><a id="ref-for-childnode④"></a>

``` text
interface mixin ChildNode {
  [CEReactions, Unscopable] undefined before((Node or DOMString)... nodes);
  [CEReactions, Unscopable] undefined after((Node or DOMString)... nodes);
  [CEReactions, Unscopable] undefined replaceWith((Node or DOMString)... nodes);
  [CEReactions, Unscopable] undefined remove();
};
DocumentType includes ChildNode;
Element includes ChildNode;
CharacterData includes ChildNode;
```

<strong><code><var>node</var> . <code><a id="ref-for-dom-childnode-before①"></a>[before(...nodes)](#dom-childnode-before)</code></code></strong>

Inserts <var>nodes</var> just before <var>node</var>, while replacing strings in <var>nodes</var> with equivalent <code><a id="ref-for-text①④"></a>[Text](#text)</code> <a id="ref-for-concept-node⑨①"></a>[nodes](#concept-node).

<a id="ref-for-dfn-throw②⑨"></a>[Throws](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror①⑦"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException③⑤"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> if the constraints of the <a id="ref-for-concept-node-tree①⑧"></a>[node tree](#concept-node-tree) are violated.

<strong><code><var>node</var> . <code><a id="ref-for-dom-childnode-after①"></a>[after(...nodes)](#dom-childnode-after)</code></code></strong>

Inserts <var>nodes</var> just after <var>node</var>, while replacing strings in <var>nodes</var> with equivalent <code><a id="ref-for-text①⑤"></a>[Text](#text)</code> <a id="ref-for-concept-node⑨②"></a>[nodes](#concept-node).

<a id="ref-for-dfn-throw③⓪"></a>[Throws](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror①⑧"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException③⑥"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> if the constraints of the <a id="ref-for-concept-node-tree①⑨"></a>[node tree](#concept-node-tree) are violated.

<strong><code><var>node</var> . <code><a id="ref-for-dom-childnode-replacewith①"></a>[replaceWith(...nodes)](#dom-childnode-replacewith)</code></code></strong>

Replaces <var>node</var> with <var>nodes</var>, while replacing strings in <var>nodes</var> with equivalent <code><a id="ref-for-text①⑥"></a>[Text](#text)</code> <a id="ref-for-concept-node⑨③"></a>[nodes](#concept-node).

<a id="ref-for-dfn-throw③①"></a>[Throws](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror①⑨"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException③⑦"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> if the constraints of the <a id="ref-for-concept-node-tree②⓪"></a>[node tree](#concept-node-tree) are violated.

<strong><code><var>node</var> . <code><a id="ref-for-dom-childnode-remove①"></a>[remove()](#dom-childnode-remove)</code></code></strong>

Removes <var>node</var>.

The <a id="dom-childnode-before"></a><strong><code>before(<var>nodes</var>)</code></strong> method steps are:

1.  Let <var>parent</var> be <a id="ref-for-this⑤①"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-parent①⑥"></a>[parent](#concept-tree-parent).

2.  If <var>parent</var> is null, then return.

3.  Let <var>viablePreviousSibling</var> be <a id="ref-for-this⑤②"></a>[this](https://webidl.spec.whatwg.org/#this)’s first <a id="ref-for-concept-tree-preceding⑥"></a>[preceding](#concept-tree-preceding) <a id="ref-for-concept-tree-sibling①⓪"></a>[sibling](#concept-tree-sibling) not in <var>nodes</var>; otherwise null.

4.  Let <var>node</var> be the result of <a id="ref-for-convert-nodes-into-a-node③"></a>[converting nodes into a node](#convert-nodes-into-a-node), given <var>nodes</var> and <a id="ref-for-this⑤③"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document①⑤"></a>[node document](#concept-node-document).

5.  If <var>viablePreviousSibling</var> is null, then set it to <var>parent</var>’s <a id="ref-for-concept-tree-first-child②"></a>[first child](#concept-tree-first-child); otherwise to <var>viablePreviousSibling</var>’s <a id="ref-for-concept-tree-next-sibling⑥"></a>[next sibling](#concept-tree-next-sibling).

6.  <a id="ref-for-concept-node-pre-insert②"></a>[Pre-insert](#concept-node-pre-insert) <var>node</var> into <var>parent</var> before <var>viablePreviousSibling</var>.

The <a id="dom-childnode-after"></a><strong><code>after(<var>nodes</var>)</code></strong> method steps are:

1.  Let <var>parent</var> be <a id="ref-for-this⑤④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-parent①⑦"></a>[parent](#concept-tree-parent).

2.  If <var>parent</var> is null, then return.

3.  Let <var>viableNextSibling</var> be <a id="ref-for-this⑤⑤"></a>[this](https://webidl.spec.whatwg.org/#this)’s first <a id="ref-for-concept-tree-following⑥"></a>[following](#concept-tree-following) <a id="ref-for-concept-tree-sibling①①"></a>[sibling](#concept-tree-sibling) not in <var>nodes</var>; otherwise null.

4.  Let <var>node</var> be the result of <a id="ref-for-convert-nodes-into-a-node④"></a>[converting nodes into a node](#convert-nodes-into-a-node), given <var>nodes</var> and <a id="ref-for-this⑤⑥"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document①⑥"></a>[node document](#concept-node-document).

5.  <a id="ref-for-concept-node-pre-insert③"></a>[Pre-insert](#concept-node-pre-insert) <var>node</var> into <var>parent</var> before <var>viableNextSibling</var>.

The <a id="dom-childnode-replacewith"></a><strong><code>replaceWith(<var>nodes</var>)</code></strong> method steps are:

1.  Let <var>parent</var> be <a id="ref-for-this⑤⑦"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-parent①⑧"></a>[parent](#concept-tree-parent).

2.  If <var>parent</var> is null, then return.

3.  Let <var>viableNextSibling</var> be <a id="ref-for-this⑤⑧"></a>[this](https://webidl.spec.whatwg.org/#this)’s first <a id="ref-for-concept-tree-following⑦"></a>[following](#concept-tree-following) <a id="ref-for-concept-tree-sibling①②"></a>[sibling](#concept-tree-sibling) not in <var>nodes</var>; otherwise null.

4.  Let <var>node</var> be the result of <a id="ref-for-convert-nodes-into-a-node⑤"></a>[converting nodes into a node](#convert-nodes-into-a-node), given <var>nodes</var> and <a id="ref-for-this⑤⑨"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document①⑦"></a>[node document](#concept-node-document).

5.  If <a id="ref-for-this⑥⓪"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-parent①⑨"></a>[parent](#concept-tree-parent) is <var>parent</var>, <a id="ref-for-concept-node-replace"></a>[replace](#concept-node-replace) <a id="ref-for-this⑥①"></a>[this](https://webidl.spec.whatwg.org/#this) with <var>node</var> within <var>parent</var>.

    <strong>Note:</strong>

    <a id="ref-for-this⑥②"></a>[This](https://webidl.spec.whatwg.org/#this) could have been inserted into <var>node</var>.

6.  Otherwise, <a id="ref-for-concept-node-pre-insert④"></a>[pre-insert](#concept-node-pre-insert) <var>node</var> into <var>parent</var> before <var>viableNextSibling</var>.

The <a id="dom-childnode-remove"></a><strong><code>remove()</code></strong> method steps are:

1.  If <a id="ref-for-this⑥③"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-parent②⓪"></a>[parent](#concept-tree-parent) is null, then return.

2.  <a id="ref-for-concept-node-remove⑦"></a>[Remove](#concept-node-remove) <a id="ref-for-this⑥④"></a>[this](https://webidl.spec.whatwg.org/#this).

#### <a id="mixin-slotable"></a>4.2.9. Mixin <code><a id="ref-for-slotable"></a>[Slottable](#slotable)</code>[](#mixin-slotable)

<a id="slotable"></a><a id="ref-for-htmlslotelement"></a><a id="ref-for-dom-slotable-assignedslot"></a><a id="ref-for-element②⑥"></a><a id="ref-for-slotable①"></a><a id="ref-for-text①⑦"></a><a id="ref-for-slotable②"></a>

``` text
interface mixin Slottable {
  readonly attribute HTMLSlotElement? assignedSlot;
};
Element includes Slottable;
Text includes Slottable;
```

The <a id="dom-slotable-assignedslot"></a><strong><code>assignedSlot</code></strong> getter steps are to return the result of <a id="ref-for-find-a-slot②"></a>[find a slot](#find-a-slot) given <a id="ref-for-this⑥⑤"></a>[this](https://webidl.spec.whatwg.org/#this) and true.

#### <a id="old-style-collections"></a>4.2.10. Old-style collections: <code><a id="ref-for-nodelist①"></a>[NodeList](#nodelist)</code> and <code><a id="ref-for-htmlcollection②"></a>[HTMLCollection](#htmlcollection)</code>[](#old-style-collections)

A <a id="concept-collection"></a><strong>collection</strong> is an object that represents a list of <a id="ref-for-concept-node⑨④"></a>[nodes](#concept-node). A <a id="ref-for-concept-collection①"></a>[collection](#concept-collection) can be either <a id="concept-collection-live"></a><strong>live</strong> or <a id="concept-collection-static"></a><strong>static</strong>. Unless otherwise stated, a <a id="ref-for-concept-collection②"></a>[collection](#concept-collection) must be <a id="ref-for-concept-collection-live"></a>[live](#concept-collection-live).

If a <a id="ref-for-concept-collection③"></a>[collection](#concept-collection) is <a id="ref-for-concept-collection-live①"></a>[live](#concept-collection-live), then the attributes and methods on that object must operate on the actual underlying data, not a snapshot of the data.

When a <a id="ref-for-concept-collection④"></a>[collection](#concept-collection) is created, a filter and a root are associated with it.

The <a id="ref-for-concept-collection⑤"></a>[collection](#concept-collection) then <a id="represented-by-the-collection"></a><strong>represents</strong> a view of the subtree rooted at the <a id="ref-for-concept-collection⑥"></a>[collection’s](#concept-collection) root, containing only nodes that match the given filter. The view is linear. In the absence of specific requirements to the contrary, the nodes within the <a id="ref-for-concept-collection⑦"></a>[collection](#concept-collection) must be sorted in <a id="ref-for-concept-tree-order①⑤"></a>[tree order](#concept-tree-order).

##### <a id="interface-nodelist"></a>4.2.10.1. Interface <code><a id="ref-for-nodelist②"></a>[NodeList](#nodelist)</code>[](#interface-nodelist)

A <code><a id="ref-for-nodelist③"></a>[NodeList](#nodelist)</code> object is a <a id="ref-for-concept-collection⑧"></a>[collection](#concept-collection) of <a id="ref-for-concept-node⑨⑤"></a>[nodes](#concept-node).

<a id="ref-for-Exposed①"></a><a id="nodelist"></a><a id="ref-for-node①①"></a><a id="ref-for-dom-nodelist-item"></a><a id="ref-for-idl-unsigned-long①"></a><a id="dom-nodelist-item-index-index"></a><a id="ref-for-idl-unsigned-long②"></a><a id="ref-for-dom-nodelist-length"></a><a id="ref-for-dfn-iterable"></a><a id="ref-for-node①②"></a>

``` text
[Exposed=Window]
interface NodeList {
  getter Node? item(unsigned long index);
  readonly attribute unsigned long length;
  iterable<Node>;
};
```

<strong><var>collection</var> . <code><a id="ref-for-dom-nodelist-length①"></a>[length](#dom-nodelist-length)</code></strong>

Returns the number of <a id="ref-for-concept-node⑨⑥"></a>[nodes](#concept-node) in the <a id="ref-for-concept-collection⑨"></a>[collection](#concept-collection).

<strong><var>element</var> = <var>collection</var> . <code><a id="ref-for-dom-nodelist-item①"></a>[item(index)](#dom-nodelist-item)</code></strong>

<strong><var>element</var> = <var>collection</var>\[<var>index</var>\]</strong>

Returns the <a id="ref-for-concept-node⑨⑦"></a>[node](#concept-node) with index <var>index</var> from the <a id="ref-for-concept-collection①⓪"></a>[collection](#concept-collection). The <a id="ref-for-concept-node⑨⑧"></a>[nodes](#concept-node) are sorted in <a id="ref-for-concept-tree-order①⑥"></a>[tree order](#concept-tree-order).

The object’s <a id="ref-for-dfn-supported-property-indices"></a>[supported property indices](https://webidl.spec.whatwg.org/#dfn-supported-property-indices) are the numbers in the range zero to one less than the number of nodes <a id="ref-for-represented-by-the-collection"></a>[represented by the collection](#represented-by-the-collection). If there are no such elements, then there are no <a id="ref-for-dfn-supported-property-indices①"></a>[supported property indices](https://webidl.spec.whatwg.org/#dfn-supported-property-indices).

The <a id="dom-nodelist-length"></a><strong><code>length</code></strong> attribute must return the number of nodes <a id="ref-for-represented-by-the-collection①"></a>[represented by the collection](#represented-by-the-collection).

The <a id="dom-nodelist-item"></a><strong><code>item(<var>index</var>)</code></strong> method must return the <var>index</var><sup>th</sup> <a id="ref-for-concept-node⑨⑨"></a>[node](#concept-node) in the <a id="ref-for-concept-collection①①"></a>[collection](#concept-collection). If there is no <var>index</var><sup>th</sup> <a id="ref-for-concept-node①⓪⓪"></a>[node](#concept-node) in the <a id="ref-for-concept-collection①②"></a>[collection](#concept-collection), then the method must return null.

##### <a id="interface-htmlcollection"></a>4.2.10.2. Interface <code><a id="ref-for-htmlcollection③"></a>[HTMLCollection](#htmlcollection)</code>[](#interface-htmlcollection)

<a id="ref-for-Exposed②"></a><a id="ref-for-LegacyUnenumerableNamedProperties"></a><a id="htmlcollection"></a><a id="ref-for-idl-unsigned-long③"></a><a id="ref-for-dom-htmlcollection-length"></a><a id="ref-for-element②⑦"></a><a id="ref-for-dom-htmlcollection-item"></a><a id="ref-for-idl-unsigned-long④"></a><a id="dom-htmlcollection-item-index-index"></a><a id="ref-for-element②⑧"></a><a id="dom-htmlcollection-nameditem"></a><a id="ref-for-idl-DOMString①⑥"></a><a id="dom-htmlcollection-nameditem-name-name"></a>

``` text
[Exposed=Window, LegacyUnenumerableNamedProperties]
interface HTMLCollection {
  readonly attribute unsigned long length;
  getter Element? item(unsigned long index);
  getter Element? namedItem(DOMString name);
};
```

An <code><a id="ref-for-htmlcollection④"></a>[HTMLCollection](#htmlcollection)</code> object is a <a id="ref-for-concept-collection①③"></a>[collection](#concept-collection) of <a id="ref-for-concept-element③③"></a>[elements](#concept-element).

<strong>Note:</strong>

<code><a id="ref-for-htmlcollection⑤"></a>[HTMLCollection](#htmlcollection)</code> is a historical artifact we cannot rid the web of. While developers are of course welcome to keep using it, new API standard designers ought not to use it (use <code>sequence&#60;T&#62;</code> in IDL instead).

<strong><var>collection</var> . <code><a id="ref-for-dom-htmlcollection-length①"></a>[length](#dom-htmlcollection-length)</code></strong>

Returns the number of <a id="ref-for-concept-element③④"></a>[elements](#concept-element) in the <a id="ref-for-concept-collection①④"></a>[collection](#concept-collection).

<strong><var>element</var> = <var>collection</var> . <code><a id="ref-for-dom-htmlcollection-item①"></a>[item(index)](#dom-htmlcollection-item)</code></strong>

<strong><var>element</var> = <var>collection</var>\[<var>index</var>\]</strong>

Returns the <a id="ref-for-concept-element③⑤"></a>[element](#concept-element) with index <var>index</var> from the <a id="ref-for-concept-collection①⑤"></a>[collection](#concept-collection). The <a id="ref-for-concept-element③⑥"></a>[elements](#concept-element) are sorted in <a id="ref-for-concept-tree-order①⑦"></a>[tree order](#concept-tree-order).

<strong><var>element</var> = <var>collection</var> . <code><a id="ref-for-dom-htmlcollection-nameditem"></a>[namedItem(name)](#dom-htmlcollection-nameditem)</code></strong>

<strong><var>element</var> = <var>collection</var>\[<var>name</var>\]</strong>

Returns the first <a id="ref-for-concept-element③⑦"></a>[element](#concept-element) with <a id="ref-for-concept-id②"></a>[ID](#concept-id) or name <var>name</var> from the collection.

The object’s <a id="ref-for-dfn-supported-property-indices②"></a>[supported property indices](https://webidl.spec.whatwg.org/#dfn-supported-property-indices) are the numbers in the range zero to one less than the number of elements <a id="ref-for-represented-by-the-collection②"></a>[represented by the collection](#represented-by-the-collection). If there are no such elements, then there are no <a id="ref-for-dfn-supported-property-indices③"></a>[supported property indices](https://webidl.spec.whatwg.org/#dfn-supported-property-indices).

The <a id="dom-htmlcollection-length"></a><strong><code>length</code></strong> getter steps are to return the number of nodes <a id="ref-for-represented-by-the-collection③"></a>[represented by the collection](#represented-by-the-collection).

The <a id="dom-htmlcollection-item"></a><strong><code>item(<var>index</var>)</code></strong> method steps are to return the <var>index</var><sup>th</sup> <a id="ref-for-concept-element③⑧"></a>[element](#concept-element) in the <a id="ref-for-concept-collection①⑥"></a>[collection](#concept-collection). If there is no <var>index</var><sup>th</sup> <a id="ref-for-concept-element③⑨"></a>[element](#concept-element) in the <a id="ref-for-concept-collection①⑦"></a>[collection](#concept-collection), then the method must return null.

The <a id="ref-for-dfn-supported-property-names"></a>[supported property names](https://webidl.spec.whatwg.org/#dfn-supported-property-names) are the values from the list returned by these steps:

1.  Let <var>result</var> be an empty list.

2.  For each <var>element</var> <a id="ref-for-represented-by-the-collection④"></a>[represented by the collection](#represented-by-the-collection), in <a id="ref-for-concept-tree-order①⑧"></a>[tree order](#concept-tree-order):

    1.  If <var>element</var> has an <a id="ref-for-concept-id③"></a>[ID](#concept-id) which is not in <var>result</var>, append <var>element</var>’s <a id="ref-for-concept-id④"></a>[ID](#concept-id) to <var>result</var>.

    2.  If <var>element</var> is in the <a id="ref-for-html-namespace"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) and <a id="ref-for-concept-element-attribute-has"></a>[has](#concept-element-attribute-has) a <a id="ref-for-concept-named-attribute"></a>[<code>name</code> attribute](#concept-named-attribute) whose <a id="ref-for-concept-attribute-value"></a>[value](#concept-attribute-value) is neither the empty string nor is in <var>result</var>, append <var>element</var>’s <a id="ref-for-concept-named-attribute①"></a>[<code>name</code> attribute](#concept-named-attribute) <a id="ref-for-concept-attribute-value①"></a>[value](#concept-attribute-value) to <var>result</var>.

3.  Return <var>result</var>.

The <a id="dom-htmlcollection-nameditem-key"></a><strong><code>namedItem(<var>key</var>)</code></strong> method steps are:

1.  If <var>key</var> is the empty string, return null.

2.  Return the first <a id="ref-for-concept-element④⓪"></a>[element](#concept-element) in the <a id="ref-for-concept-collection①⑧"></a>[collection](#concept-collection) for which at least one of the following is true:

    - it has an <a id="ref-for-concept-id⑤"></a>[ID](#concept-id) which is <var>key</var>;
    - it is in the <a id="ref-for-html-namespace①"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) and <a id="ref-for-concept-element-attribute-has①"></a>[has](#concept-element-attribute-has) a <a id="ref-for-concept-named-attribute②"></a>[<code>name</code> attribute](#concept-named-attribute) whose <a id="ref-for-concept-attribute-value②"></a>[value](#concept-attribute-value) is <var>key</var>;

    or null if there is no such <a id="ref-for-concept-element④①"></a>[element](#concept-element).

### <a id="interface-node"></a>4.4. Interface <code><a id="ref-for-node①⑦"></a>[Node](#node)</code>[](#interface-node)

<a id="ref-for-Exposed⑤"></a><a id="node"></a><a id="ref-for-eventtarget②⑦"></a><a id="ref-for-idl-unsigned-short⑤"></a><a id="ref-for-dom-node-element&#95;node"></a><a id="ref-for-idl-unsigned-short⑥"></a><a id="ref-for-dom-node-attribute&#95;node"></a><a id="ref-for-idl-unsigned-short⑦"></a><a id="ref-for-dom-node-text&#95;node"></a><a id="ref-for-idl-unsigned-short⑧"></a><a id="ref-for-dom-node-cdata&#95;section&#95;node"></a><a id="ref-for-idl-unsigned-short⑨"></a><a id="dom-node-entity&#95;reference&#95;node"></a><a id="ref-for-idl-unsigned-short①⓪"></a><a id="dom-node-entity&#95;node"></a><a id="ref-for-idl-unsigned-short①①"></a><a id="ref-for-dom-node-processing&#95;instruction&#95;node"></a><a id="ref-for-idl-unsigned-short①②"></a><a id="ref-for-dom-node-comment&#95;node"></a><a id="ref-for-idl-unsigned-short①③"></a><a id="ref-for-dom-node-document&#95;node"></a><a id="ref-for-idl-unsigned-short①④"></a><a id="ref-for-dom-node-document&#95;type&#95;node"></a><a id="ref-for-idl-unsigned-short①⑤"></a><a id="ref-for-dom-node-document&#95;fragment&#95;node"></a><a id="ref-for-idl-unsigned-short①⑥"></a><a id="dom-node-notation&#95;node"></a><a id="ref-for-idl-unsigned-short①⑦"></a><a id="ref-for-dom-node-nodetype"></a><a id="ref-for-idl-DOMString②②"></a><a id="ref-for-dom-node-nodename"></a><a id="ref-for-idl-USVString"></a><a id="ref-for-dom-node-baseuri"></a><a id="ref-for-idl-boolean②⑦"></a><a id="ref-for-dom-node-isconnected"></a><a id="ref-for-document⑦"></a><a id="ref-for-dom-node-ownerdocument"></a><a id="ref-for-node①⑧"></a><a id="ref-for-dom-node-getrootnode"></a><a id="ref-for-dictdef-getrootnodeoptions"></a><a id="dom-node-getrootnode-options-options"></a><a id="ref-for-node①⑨"></a><a id="ref-for-dom-node-parentnode"></a><a id="ref-for-element②⑨"></a><a id="ref-for-dom-node-parentelement"></a><a id="ref-for-idl-boolean②⑧"></a><a id="ref-for-dom-node-haschildnodes"></a><a id="ref-for-SameObject⑤"></a><a id="ref-for-nodelist⑥"></a><a id="ref-for-dom-node-childnodes"></a><a id="ref-for-node②⓪"></a><a id="ref-for-dom-node-firstchild"></a><a id="ref-for-node②①"></a><a id="ref-for-dom-node-lastchild"></a><a id="ref-for-node②②"></a><a id="ref-for-dom-node-previoussibling"></a><a id="ref-for-node②③"></a><a id="ref-for-dom-node-nextsibling"></a><a id="ref-for-cereactions⑧"></a><a id="ref-for-idl-DOMString②③"></a><a id="ref-for-dom-node-nodevalue"></a><a id="ref-for-cereactions⑨"></a><a id="ref-for-idl-DOMString②④"></a><a id="ref-for-dom-node-textcontent"></a><a id="ref-for-cereactions①⓪"></a><a id="ref-for-idl-undefined②②"></a><a id="ref-for-dom-node-normalize"></a><a id="ref-for-cereactions①①"></a><a id="ref-for-NewObject④"></a><a id="ref-for-node②④"></a><a id="ref-for-dom-node-clonenode"></a><a id="ref-for-idl-boolean②⑨"></a><a id="dom-node-clonenode-subtree-subtree"></a><a id="ref-for-idl-boolean③⓪"></a><a id="ref-for-dom-node-isequalnode"></a><a id="ref-for-node②⑤"></a><a id="dom-node-isequalnode-othernode-othernode"></a><a id="ref-for-idl-boolean③①"></a><a id="ref-for-dom-node-issamenode"></a><a id="ref-for-node②⑥"></a><a id="dom-node-issamenode-othernode-othernode"></a><a id="ref-for-idl-unsigned-short①⑧"></a><a id="ref-for-dom-node-document&#95;position&#95;disconnected"></a><a id="ref-for-idl-unsigned-short①⑨"></a><a id="ref-for-dom-node-document&#95;position&#95;preceding"></a><a id="ref-for-idl-unsigned-short②⓪"></a><a id="ref-for-dom-node-document&#95;position&#95;following"></a><a id="ref-for-idl-unsigned-short②①"></a><a id="ref-for-dom-node-document&#95;position&#95;contains"></a><a id="ref-for-idl-unsigned-short②②"></a><a id="ref-for-dom-node-document&#95;position&#95;contained&#95;by"></a><a id="ref-for-idl-unsigned-short②③"></a><a id="ref-for-dom-node-document&#95;position&#95;implementation&#95;specific"></a><a id="ref-for-idl-unsigned-short②④"></a><a id="ref-for-dom-node-comparedocumentposition"></a><a id="ref-for-node②⑦"></a><a id="dom-node-comparedocumentposition-other-other"></a><a id="ref-for-idl-boolean③②"></a><a id="ref-for-dom-node-contains"></a><a id="ref-for-node②⑧"></a><a id="dom-node-contains-other-other"></a><a id="ref-for-idl-DOMString②⑤"></a><a id="ref-for-dom-node-lookupprefix"></a><a id="ref-for-idl-DOMString②⑥"></a><a id="dom-node-lookupprefix-namespace-namespace"></a><a id="ref-for-idl-DOMString②⑦"></a><a id="ref-for-dom-node-lookupnamespaceuri"></a><a id="ref-for-idl-DOMString②⑧"></a><a id="dom-node-lookupnamespaceuri-prefix-prefix"></a><a id="ref-for-idl-boolean③③"></a><a id="ref-for-dom-node-isdefaultnamespace"></a><a id="ref-for-idl-DOMString②⑨"></a><a id="dom-node-isdefaultnamespace-namespace-namespace"></a><a id="ref-for-cereactions①②"></a><a id="ref-for-node②⑨"></a><a id="ref-for-dom-node-insertbefore"></a><a id="ref-for-node③⓪"></a><a id="dom-node-insertbefore-node-child-node"></a><a id="ref-for-node③①"></a><a id="dom-node-insertbefore-node-child-child"></a><a id="ref-for-cereactions①③"></a><a id="ref-for-node③②"></a><a id="ref-for-dom-node-appendchild"></a><a id="ref-for-node③③"></a><a id="dom-node-appendchild-node-node"></a><a id="ref-for-cereactions①④"></a><a id="ref-for-node③④"></a><a id="ref-for-dom-node-replacechild"></a><a id="ref-for-node③⑤"></a><a id="dom-node-replacechild-node-child-node"></a><a id="ref-for-node③⑥"></a><a id="dom-node-replacechild-node-child-child"></a><a id="ref-for-cereactions①⑤"></a><a id="ref-for-node③⑦"></a><a id="ref-for-dom-node-removechild"></a><a id="ref-for-node③⑧"></a><a id="dom-node-removechild-child-child"></a><a id="dictdef-getrootnodeoptions"></a><a id="ref-for-idl-boolean③④"></a><a id="dom-getrootnodeoptions-composed"></a>

``` text
[Exposed=Window]
interface Node : EventTarget {
  const unsigned short ELEMENT_NODE = 1;
  const unsigned short ATTRIBUTE_NODE = 2;
  const unsigned short TEXT_NODE = 3;
  const unsigned short CDATA_SECTION_NODE = 4;
  const unsigned short ENTITY_REFERENCE_NODE = 5; // legacy
  const unsigned short ENTITY_NODE = 6; // legacy
  const unsigned short PROCESSING_INSTRUCTION_NODE = 7;
  const unsigned short COMMENT_NODE = 8;
  const unsigned short DOCUMENT_NODE = 9;
  const unsigned short DOCUMENT_TYPE_NODE = 10;
  const unsigned short DOCUMENT_FRAGMENT_NODE = 11;
  const unsigned short NOTATION_NODE = 12; // legacy
  readonly attribute unsigned short nodeType;
  readonly attribute DOMString nodeName;

  readonly attribute USVString baseURI;

  readonly attribute boolean isConnected;
  readonly attribute Document? ownerDocument;
  Node getRootNode(optional GetRootNodeOptions options = {});
  readonly attribute Node? parentNode;
  readonly attribute Element? parentElement;
  boolean hasChildNodes();
  [SameObject] readonly attribute NodeList childNodes;
  readonly attribute Node? firstChild;
  readonly attribute Node? lastChild;
  readonly attribute Node? previousSibling;
  readonly attribute Node? nextSibling;

  [CEReactions] attribute DOMString? nodeValue;
  [CEReactions] attribute DOMString? textContent;
  [CEReactions] undefined normalize();

  [CEReactions, NewObject] Node cloneNode(optional boolean subtree = false);
  boolean isEqualNode(Node? otherNode);
  boolean isSameNode(Node? otherNode); // legacy alias of ===

  const unsigned short DOCUMENT_POSITION_DISCONNECTED = 0x01;
  const unsigned short DOCUMENT_POSITION_PRECEDING = 0x02;
  const unsigned short DOCUMENT_POSITION_FOLLOWING = 0x04;
  const unsigned short DOCUMENT_POSITION_CONTAINS = 0x08;
  const unsigned short DOCUMENT_POSITION_CONTAINED_BY = 0x10;
  const unsigned short DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC = 0x20;
  unsigned short compareDocumentPosition(Node other);
  boolean contains(Node? other);

  DOMString? lookupPrefix(DOMString? namespace);
  DOMString? lookupNamespaceURI(DOMString? prefix);
  boolean isDefaultNamespace(DOMString? namespace);

  [CEReactions] Node insertBefore(Node node, Node? child);
  [CEReactions] Node appendChild(Node node);
  [CEReactions] Node replaceChild(Node node, Node child);
  [CEReactions] Node removeChild(Node child);
};

dictionary GetRootNodeOptions {
  boolean composed = false;
};
```

<strong>Note:</strong>

<code><a id="ref-for-node③⑨"></a>[Node](#node)</code> is an abstract interface that is used by all <a id="ref-for-concept-node①②⓪"></a>[nodes](#concept-node). You cannot get a direct instance of it.

Each <a id="ref-for-concept-node①②①"></a>[node](#concept-node) has an associated <a id="concept-node-document"></a><strong>node document</strong>, set upon creation, that is a <a id="ref-for-concept-document①④"></a>[document](#concept-document).

<strong>Note:</strong>

A <a id="ref-for-concept-node①②②"></a>[node](#concept-node)’s <a id="ref-for-concept-node-document①⑧"></a>[node document](#concept-node-document) can be changed by the <a id="ref-for-concept-node-adopt②"></a>[adopt](#concept-node-adopt) algorithm.

A <a id="ref-for-concept-node①②③"></a>[node](#concept-node)’s <a id="ref-for-get-the-parent⑤"></a>[get the parent](https://dom.spec.whatwg.org/#get-the-parent) algorithm, given an <var>event</var>, returns the <a id="ref-for-concept-node①②④"></a>[node](#concept-node)’s <a id="ref-for-slotable-assigned-slot⑤"></a>[assigned slot](#slotable-assigned-slot), if <a id="ref-for-concept-node①②⑤"></a>[node](#concept-node) is <a id="ref-for-slotable-assigned⑤"></a>[assigned](#slotable-assigned); otherwise <a id="ref-for-concept-node①②⑥"></a>[node](#concept-node)’s <a id="ref-for-concept-tree-parent②②"></a>[parent](#concept-tree-parent).

<strong>Note:</strong>

Each <a id="ref-for-concept-node①②⑦"></a>[node](#concept-node) also has a <a id="ref-for-registered-observer-list①⓪"></a>[registered observer list](https://dom.spec.whatwg.org/#registered-observer-list).

To <a id="create-a-node"></a><strong>create a node</strong> that implements an interface <var>interface</var>, given a <a id="ref-for-concept-document①⑤"></a>[document](#concept-document) <var>document</var>:

1.  <a id="ref-for-assert⑥"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>interface</var> is neither <code><a id="ref-for-document⑧"></a>[Document](#document)</code> nor an interface that inherits from <code><a id="ref-for-document⑨"></a>[Document](#document)</code>.

    <strong>Note:</strong>

    To create a <a id="ref-for-concept-document①⑥"></a>[document](#concept-document), use <a id="ref-for-create-a-document"></a>[create a document](#create-a-document) instead.

2.  Return a new <a id="ref-for-concept-node①②⑧"></a>[node](#concept-node) that <a id="ref-for-implements③"></a>[implements](https://webidl.spec.whatwg.org/#implements) <var>interface</var>, in <var>document</var>’s <a id="ref-for-concept-relevant-realm①"></a>[relevant realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm), with its <a id="ref-for-concept-node-document①⑨"></a>[node document](#concept-node-document) set to <var>document</var>.

------------------------------------------------------------------------

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-nodetype①"></a>[nodeType](#dom-node-nodetype)</code></code></strong>

Returns a number appropriate for the type of <var>node</var>, as follows:

<strong><code><a id="ref-for-element③⓪"></a>[Element](#element)</code></strong>

<code><code><a id="ref-for-node④⓪"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-element&#95;node①"></a>[ELEMENT_NODE](#dom-node-element_node)</code></code> (1).

<strong><code><a id="ref-for-attr⑤"></a>[Attr](#attr)</code></strong>

<code><code><a id="ref-for-node④①"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-attribute&#95;node①"></a>[ATTRIBUTE_NODE](#dom-node-attribute_node)</code></code> (2).

<strong>An <a id="ref-for-exclusive-text-node"></a>[exclusive <code>Text</code> node](#exclusive-text-node)</strong>

<code><code><a id="ref-for-node④②"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-text&#95;node①"></a>[TEXT_NODE](#dom-node-text_node)</code></code> (3).

<strong><code><a id="ref-for-cdatasection②"></a>[CDATASection](#cdatasection)</code></strong>

<code><code><a id="ref-for-node④③"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-cdata&#95;section&#95;node①"></a>[CDATA_SECTION_NODE](#dom-node-cdata_section_node)</code></code> (4).

<strong><code><a id="ref-for-processinginstruction⑤"></a>[ProcessingInstruction](#processinginstruction)</code></strong>

<code><code><a id="ref-for-node④④"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-processing&#95;instruction&#95;node①"></a>[PROCESSING_INSTRUCTION_NODE](#dom-node-processing_instruction_node)</code></code> (7).

<strong><code><a id="ref-for-comment⑤"></a>[Comment](#comment)</code></strong>

<code><code><a id="ref-for-node④⑤"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-comment&#95;node①"></a>[COMMENT_NODE](#dom-node-comment_node)</code></code> (8).

<strong><code><a id="ref-for-document①⓪"></a>[Document](#document)</code></strong>

<code><code><a id="ref-for-node④⑥"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-document&#95;node①"></a>[DOCUMENT_NODE](#dom-node-document_node)</code></code> (9).

<strong><code><a id="ref-for-documenttype⑦"></a>[DocumentType](#documenttype)</code></strong>

<code><code><a id="ref-for-node④⑦"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-document&#95;type&#95;node①"></a>[DOCUMENT_TYPE_NODE](#dom-node-document_type_node)</code></code> (10).

<strong><code><a id="ref-for-documentfragment①④"></a>[DocumentFragment](#documentfragment)</code></strong>

<code><code><a id="ref-for-node④⑧"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-document&#95;fragment&#95;node①"></a>[DOCUMENT_FRAGMENT_NODE](#dom-node-document_fragment_node)</code></code> (11).

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-nodename①"></a>[nodeName](#dom-node-nodename)</code></code></strong>

Returns a string appropriate for the type of <var>node</var>, as follows:

<strong><code><a id="ref-for-element③①"></a>[Element](#element)</code></strong>

Its <a id="ref-for-element-html-uppercased-qualified-name"></a>[HTML-uppercased qualified name](#element-html-uppercased-qualified-name).

<strong><code><a id="ref-for-attr⑥"></a>[Attr](#attr)</code></strong>

Its <a id="ref-for-concept-attribute-qualified-name"></a>[qualified name](#concept-attribute-qualified-name).

<strong>An <a id="ref-for-exclusive-text-node①"></a>[exclusive <code>Text</code> node](#exclusive-text-node)</strong>

"<code>&#35;text</code>".

<strong><code><a id="ref-for-cdatasection③"></a>[CDATASection](#cdatasection)</code></strong>

"<code>&#35;cdata-section</code>".

<strong><code><a id="ref-for-processinginstruction⑥"></a>[ProcessingInstruction](#processinginstruction)</code></strong>

Its <a id="ref-for-concept-pi-target"></a>[target](#concept-pi-target).

<strong><code><a id="ref-for-comment⑥"></a>[Comment](#comment)</code></strong>

"<code>&#35;comment</code>".

<strong><code><a id="ref-for-document①①"></a>[Document](#document)</code></strong>

"<code>&#35;document</code>".

<strong><code><a id="ref-for-documenttype⑧"></a>[DocumentType](#documenttype)</code></strong>

Its <a id="ref-for-concept-doctype-name"></a>[name](#concept-doctype-name).

<strong><code><a id="ref-for-documentfragment①⑤"></a>[DocumentFragment](#documentfragment)</code></strong>

"<code>&#35;document-fragment</code>".

The <a id="dom-node-nodetype"></a><strong><code>nodeType</code></strong> getter steps are to return the first matching statement, switching on the interface <a id="ref-for-this⑧③"></a>[this](https://webidl.spec.whatwg.org/#this) <a id="ref-for-implements④"></a>[implements](https://webidl.spec.whatwg.org/#implements):

<strong><code><a id="ref-for-element③②"></a>[Element](#element)</code></strong>

<a id="dom-node-element&#95;node"></a><strong><code>ELEMENT&#95;NODE</code></strong> (1)

<strong><code><a id="ref-for-attr⑦"></a>[Attr](#attr)</code></strong>

<a id="dom-node-attribute&#95;node"></a><strong><code>ATTRIBUTE&#95;NODE</code></strong> (2);

<strong>An <a id="ref-for-exclusive-text-node②"></a>[exclusive <code>Text</code> node](#exclusive-text-node)</strong>

<a id="dom-node-text&#95;node"></a><strong><code>TEXT&#95;NODE</code></strong> (3);

<strong><code><a id="ref-for-cdatasection④"></a>[CDATASection](#cdatasection)</code></strong>

<a id="dom-node-cdata&#95;section&#95;node"></a><strong><code>CDATA&#95;SECTION&#95;NODE</code></strong> (4);

<strong><code><a id="ref-for-processinginstruction⑦"></a>[ProcessingInstruction](#processinginstruction)</code></strong>

<a id="dom-node-processing&#95;instruction&#95;node"></a><strong><code>PROCESSING&#95;INSTRUCTION&#95;NODE</code></strong> (7);

<strong><code><a id="ref-for-comment⑦"></a>[Comment](#comment)</code></strong>

<a id="dom-node-comment&#95;node"></a><strong><code>COMMENT&#95;NODE</code></strong> (8);

<strong><code><a id="ref-for-document①②"></a>[Document](#document)</code></strong>

<a id="dom-node-document&#95;node"></a><strong><code>DOCUMENT&#95;NODE</code></strong> (9);

<strong><code><a id="ref-for-documenttype⑨"></a>[DocumentType](#documenttype)</code></strong>

<a id="dom-node-document&#95;type&#95;node"></a><strong><code>DOCUMENT&#95;TYPE&#95;NODE</code></strong> (10);

<strong><code><a id="ref-for-documentfragment①⑥"></a>[DocumentFragment](#documentfragment)</code></strong>

<a id="dom-node-document&#95;fragment&#95;node"></a><strong><code>DOCUMENT&#95;FRAGMENT&#95;NODE</code></strong> (11).

The <a id="dom-node-nodename"></a><strong><code>nodeName</code></strong> getter steps are to return the first matching statement, switching on the interface <a id="ref-for-this⑧④"></a>[this](https://webidl.spec.whatwg.org/#this) <a id="ref-for-implements⑤"></a>[implements](https://webidl.spec.whatwg.org/#implements):

<strong><code><a id="ref-for-element③③"></a>[Element](#element)</code></strong>

Its <a id="ref-for-element-html-uppercased-qualified-name①"></a>[HTML-uppercased qualified name](#element-html-uppercased-qualified-name).

<strong><code><a id="ref-for-attr⑧"></a>[Attr](#attr)</code></strong>

Its <a id="ref-for-concept-attribute-qualified-name①"></a>[qualified name](#concept-attribute-qualified-name).

<strong>An <a id="ref-for-exclusive-text-node③"></a>[exclusive <code>Text</code> node](#exclusive-text-node)</strong>

"<code>&#35;text</code>".

<strong><code><a id="ref-for-cdatasection⑤"></a>[CDATASection](#cdatasection)</code></strong>

"<code>&#35;cdata-section</code>".

<strong><code><a id="ref-for-processinginstruction⑧"></a>[ProcessingInstruction](#processinginstruction)</code></strong>

Its <a id="ref-for-concept-pi-target①"></a>[target](#concept-pi-target).

<strong><code><a id="ref-for-comment⑧"></a>[Comment](#comment)</code></strong>

"<code>&#35;comment</code>".

<strong><code><a id="ref-for-document①③"></a>[Document](#document)</code></strong>

"<code>&#35;document</code>".

<strong><code><a id="ref-for-documenttype①⓪"></a>[DocumentType](#documenttype)</code></strong>

Its <a id="ref-for-concept-doctype-name①"></a>[name](#concept-doctype-name).

<strong><code><a id="ref-for-documentfragment①⑦"></a>[DocumentFragment](#documentfragment)</code></strong>

"<code>&#35;document-fragment</code>".

------------------------------------------------------------------------

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-baseuri①"></a>[baseURI](#dom-node-baseuri)</code></code></strong>

Returns <var>node</var>’s <a id="ref-for-concept-node-document②⓪"></a>[node document](#concept-node-document)’s <a id="ref-for-document-base-url"></a>[document base URL](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#document-base-url).

The <a id="dom-node-baseuri"></a><strong><code>baseURI</code></strong> getter steps are to return <a id="ref-for-this⑧⑤"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document②①"></a>[node document](#concept-node-document)’s <a id="ref-for-document-base-url①"></a>[document base URL](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#document-base-url), <a id="ref-for-concept-url-serializer"></a>[serialized](https://url.spec.whatwg.org/#concept-url-serializer).

------------------------------------------------------------------------

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-isconnected①"></a>[isConnected](#dom-node-isconnected)</code></code></strong>

Returns true if <var>node</var> is <a id="ref-for-connected⑥"></a>[connected](#connected); otherwise false.

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-ownerdocument①"></a>[ownerDocument](#dom-node-ownerdocument)</code></code></strong>

Returns the <a id="ref-for-concept-node-document②②"></a>[node document](#concept-node-document). Returns null for <a id="ref-for-concept-document①⑦"></a>[documents](#concept-document).

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-getrootnode①"></a>[getRootNode()](#dom-node-getrootnode)</code></code></strong>

Returns <var>node</var>’s <a id="ref-for-concept-tree-root②③"></a>[root](#concept-tree-root).

<strong><code><var>node</var> . <a id="ref-for-dom-node-getrootnode②"></a>[getRootNode](#dom-node-getrootnode)({ composed:true })</code></strong>

Returns <var>node</var>’s <a id="ref-for-concept-shadow-including-root③"></a>[shadow-including root](#concept-shadow-including-root).

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-parentnode①"></a>[parentNode](#dom-node-parentnode)</code></code></strong>

Returns the <a id="ref-for-concept-tree-parent②③"></a>[parent](#concept-tree-parent).

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-parentelement①"></a>[parentElement](#dom-node-parentelement)</code></code></strong>

Returns the <a id="ref-for-parent-element"></a>[parent element](#parent-element).

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-haschildnodes①"></a>[hasChildNodes()](#dom-node-haschildnodes)</code></code></strong>

Returns whether <var>node</var> has <a id="ref-for-concept-tree-child④④"></a>[children](#concept-tree-child).

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-childnodes①"></a>[childNodes](#dom-node-childnodes)</code></code></strong>

Returns the <a id="ref-for-concept-tree-child④⑤"></a>[children](#concept-tree-child).

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-firstchild①"></a>[firstChild](#dom-node-firstchild)</code></code></strong>

Returns the <a id="ref-for-concept-tree-first-child③"></a>[first child](#concept-tree-first-child).

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-lastchild①"></a>[lastChild](#dom-node-lastchild)</code></code></strong>

Returns the <a id="ref-for-concept-tree-last-child④"></a>[last child](#concept-tree-last-child).

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-previoussibling①"></a>[previousSibling](#dom-node-previoussibling)</code></code></strong>

Returns the <a id="ref-for-concept-tree-previous-sibling⑥"></a>[previous sibling](#concept-tree-previous-sibling).

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-nextsibling①"></a>[nextSibling](#dom-node-nextsibling)</code></code></strong>

Returns the <a id="ref-for-concept-tree-next-sibling⑧"></a>[next sibling](#concept-tree-next-sibling).

The <a id="dom-node-isconnected"></a><strong><code>isConnected</code></strong> getter steps are to return true, if <a id="ref-for-this⑧⑥"></a>[this](https://webidl.spec.whatwg.org/#this) is <a id="ref-for-connected⑦"></a>[connected](#connected); otherwise false.

The <a id="dom-node-ownerdocument"></a><strong><code>ownerDocument</code></strong> getter steps are to return null, if <a id="ref-for-this⑧⑦"></a>[this](https://webidl.spec.whatwg.org/#this) is a <a id="ref-for-concept-document①⑧"></a>[document](#concept-document); otherwise <a id="ref-for-this⑧⑧"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document②③"></a>[node document](#concept-node-document).

<strong>Note:</strong>

The <a id="ref-for-concept-node-document②④"></a>[node document](#concept-node-document) of a <a id="ref-for-concept-document①⑨"></a>[document](#concept-document) is that <a id="ref-for-concept-document②⓪"></a>[document](#concept-document) itself. All <a id="ref-for-concept-node①②⑨"></a>[nodes](#concept-node) have a <a id="ref-for-concept-node-document②⑤"></a>[node document](#concept-node-document) at all times.

The <a id="dom-node-getrootnode"></a><strong><code>getRootNode(<var>options</var>)</code></strong> method steps are to return <a id="ref-for-this⑧⑨"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-shadow-including-root④"></a>[shadow-including root](#concept-shadow-including-root) if <var>options</var>\["<code><a id="ref-for-dom-getrootnodeoptions-composed"></a>[composed](#dom-getrootnodeoptions-composed)</code>"\] is true; otherwise <a id="ref-for-this⑨⓪"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-root②④"></a>[root](#concept-tree-root).

The <a id="dom-node-parentnode"></a><strong><code>parentNode</code></strong> getter steps are to return <a id="ref-for-this⑨①"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-parent②④"></a>[parent](#concept-tree-parent).

The <a id="dom-node-parentelement"></a><strong><code>parentElement</code></strong> getter steps are to return <a id="ref-for-this⑨②"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-parent-element①"></a>[parent element](#parent-element).

The <a id="dom-node-haschildnodes"></a><strong><code>hasChildNodes()</code></strong> method steps are to return true if <a id="ref-for-this⑨③"></a>[this](https://webidl.spec.whatwg.org/#this) has <a id="ref-for-concept-tree-child④⑥"></a>[children](#concept-tree-child); otherwise false.

The <a id="dom-node-childnodes"></a><strong><code>childNodes</code></strong> getter steps are to return a <code><a id="ref-for-nodelist⑦"></a>[NodeList](#nodelist)</code> rooted at <a id="ref-for-this⑨④"></a>[this](https://webidl.spec.whatwg.org/#this) matching only <a id="ref-for-concept-tree-child④⑦"></a>[children](#concept-tree-child).

The <a id="dom-node-firstchild"></a><strong><code>firstChild</code></strong> getter steps are to return <a id="ref-for-this⑨⑤"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-first-child④"></a>[first child](#concept-tree-first-child).

The <a id="dom-node-lastchild"></a><strong><code>lastChild</code></strong> getter steps are to return <a id="ref-for-this⑨⑥"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-last-child⑤"></a>[last child](#concept-tree-last-child).

The <a id="dom-node-previoussibling"></a><strong><code>previousSibling</code></strong> getter steps are to return <a id="ref-for-this⑨⑦"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-previous-sibling⑦"></a>[previous sibling](#concept-tree-previous-sibling).

The <a id="dom-node-nextsibling"></a><strong><code>nextSibling</code></strong> getter steps are to return <a id="ref-for-this⑨⑧"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-next-sibling⑨"></a>[next sibling](#concept-tree-next-sibling).

------------------------------------------------------------------------

The <a id="dom-node-nodevalue"></a><strong><code>nodeValue</code></strong> getter steps are to return the following, switching on the interface <a id="ref-for-this⑨⑨"></a>[this](https://webidl.spec.whatwg.org/#this) <a id="ref-for-implements⑥"></a>[implements](https://webidl.spec.whatwg.org/#implements):

<strong><code><a id="ref-for-attr⑨"></a>[Attr](#attr)</code></strong>

<a id="ref-for-this①⓪⓪"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-attribute-value⑤"></a>[value](#concept-attribute-value).

<strong><code><a id="ref-for-characterdata①②"></a>[CharacterData](#characterdata)</code></strong>

<a id="ref-for-this①⓪①"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-cd-data④"></a>[data](#concept-cd-data).

<strong>Otherwise</strong>

Null.

The <code><a id="ref-for-dom-node-nodevalue①"></a>[nodeValue](#dom-node-nodevalue)</code> setter steps are to, if the given value is null, act as if it was the empty string instead, and then do as described below, switching on the interface <a id="ref-for-this①⓪②"></a>[this](https://webidl.spec.whatwg.org/#this) <a id="ref-for-implements⑦"></a>[implements](https://webidl.spec.whatwg.org/#implements):

<strong><code><a id="ref-for-attr①⓪"></a>[Attr](#attr)</code></strong>

<a id="ref-for-set-an-existing-attribute-value"></a>[Set an existing attribute value](#set-an-existing-attribute-value) with <a id="ref-for-this①⓪③"></a>[this](https://webidl.spec.whatwg.org/#this) and the given value.

<strong><code><a id="ref-for-characterdata①③"></a>[CharacterData](#characterdata)</code></strong>

<a id="ref-for-concept-cd-replace①"></a>[Replace data](#concept-cd-replace) of <a id="ref-for-this①⓪④"></a>[this](https://webidl.spec.whatwg.org/#this) with 0, <a id="ref-for-this①⓪⑤"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-length①"></a>[length](#concept-node-length), and the given value.

<strong>Otherwise</strong>

Do nothing.

To <a id="get-text-content"></a><strong>get text content</strong> with a <a id="ref-for-concept-node①③⓪"></a>[node](#concept-node) <var>node</var>, return the following, switching on the interface <var>node</var> <a id="ref-for-implements⑧"></a>[implements](https://webidl.spec.whatwg.org/#implements):

<strong><code><a id="ref-for-documentfragment①⑧"></a>[DocumentFragment](#documentfragment)</code></strong>

<strong><code><a id="ref-for-element③④"></a>[Element](#element)</code></strong>

The <a id="ref-for-concept-descendant-text-content"></a>[descendant text content](#concept-descendant-text-content) of <var>node</var>.

<strong><code><a id="ref-for-attr①①"></a>[Attr](#attr)</code></strong>

<var>node</var>’s <a id="ref-for-concept-attribute-value⑥"></a>[value](#concept-attribute-value).

<strong><code><a id="ref-for-characterdata①④"></a>[CharacterData](#characterdata)</code></strong>

<var>node</var>’s <a id="ref-for-concept-cd-data⑤"></a>[data](#concept-cd-data).

<strong>Otherwise</strong>

Null.

The <a id="dom-node-textcontent"></a><strong><code>textContent</code></strong> getter steps are to return the result of running <a id="ref-for-get-text-content"></a>[get text content](#get-text-content) with <a id="ref-for-this①⓪⑥"></a>[this](https://webidl.spec.whatwg.org/#this).

To <a id="string-replace-all"></a><strong>string replace all</strong> with a string <var>string</var> within a <a id="ref-for-concept-node①③①"></a>[node](#concept-node) <var>parent</var>, run these steps:

1.  Let <var>node</var> be null.

2.  If <var>string</var> is not the empty string, then set <var>node</var> to the result of <a id="ref-for-create-a-text-node①"></a>[creating a text node](#create-a-text-node) given <var>parent</var>’s <a id="ref-for-concept-node-document②⑥"></a>[node document](#concept-node-document) and <var>string</var>.

3.  <a id="ref-for-concept-node-replace-all①"></a>[Replace all](#concept-node-replace-all) with <var>node</var> within <var>parent</var>.

To <a id="set-text-content"></a><strong>set text content</strong> with a <a id="ref-for-concept-node①③②"></a>[node](#concept-node) <var>node</var> and a string <var>value</var>, do as defined below, switching on the interface <var>node</var> <a id="ref-for-implements⑨"></a>[implements](https://webidl.spec.whatwg.org/#implements):

<strong><code><a id="ref-for-documentfragment①⑨"></a>[DocumentFragment](#documentfragment)</code></strong>

<strong><code><a id="ref-for-element③⑤"></a>[Element](#element)</code></strong>

<a id="ref-for-string-replace-all"></a>[String replace all](#string-replace-all) with <var>value</var> within <var>node</var>.

<strong><code><a id="ref-for-attr①②"></a>[Attr](#attr)</code></strong>

<a id="ref-for-set-an-existing-attribute-value①"></a>[Set an existing attribute value](#set-an-existing-attribute-value) with <var>node</var> and <var>value</var>.

<strong><code><a id="ref-for-characterdata①⑤"></a>[CharacterData](#characterdata)</code></strong>

<a id="ref-for-concept-cd-replace②"></a>[Replace data](#concept-cd-replace) of <var>node</var> with 0, <var>node</var>’s <a id="ref-for-concept-node-length②"></a>[length](#concept-node-length), and <var>value</var>.

<strong>Otherwise</strong>

Do nothing.

The <code><a id="ref-for-dom-node-textcontent①"></a>[textContent](#dom-node-textcontent)</code> setter steps are to, if the given value is null, act as if it was the empty string instead, and then run <a id="ref-for-set-text-content"></a>[set text content](#set-text-content) with <a id="ref-for-this①⓪⑦"></a>[this](https://webidl.spec.whatwg.org/#this) and the given value.

------------------------------------------------------------------------

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-normalize①"></a>[normalize()](#dom-node-normalize)</code></code></strong>

Removes <a id="ref-for-concept-node-empty"></a>[empty](#concept-node-empty) <a id="ref-for-exclusive-text-node④"></a>[exclusive <code>Text</code> nodes](#exclusive-text-node) and concatenates the <a id="ref-for-concept-cd-data⑥"></a>[data](#concept-cd-data) of remaining <a id="ref-for-contiguous-exclusive-text-nodes"></a>[contiguous exclusive <code>Text</code> nodes](#contiguous-exclusive-text-nodes) into the first of their <a id="ref-for-concept-node①③③"></a>[nodes](#concept-node).

The <a id="dom-node-normalize"></a><strong><code>normalize()</code></strong> method steps are to run these steps for each <a id="ref-for-concept-tree-descendant①①"></a>[descendant](#concept-tree-descendant) <a id="ref-for-exclusive-text-node⑤"></a>[exclusive <code>Text</code> node](#exclusive-text-node) <var>node</var> of <a id="ref-for-this①⓪⑧"></a>[this](https://webidl.spec.whatwg.org/#this):

1.  Let <var>length</var> be <var>node</var>’s <a id="ref-for-concept-node-length③"></a>[length](#concept-node-length).

2.  If <var>length</var> is zero, then <a id="ref-for-concept-node-remove⑧"></a>[remove](#concept-node-remove) <var>node</var> and continue with the next <a id="ref-for-exclusive-text-node⑥"></a>[exclusive <code>Text</code> node](#exclusive-text-node), if any.

3.  Let <var>data</var> be the <a id="ref-for-string-concatenate①"></a>[concatenation](https://infra.spec.whatwg.org/#string-concatenate) of the <a id="ref-for-concept-cd-data⑦"></a>[data](#concept-cd-data) of <var>node</var>’s <a id="ref-for-contiguous-exclusive-text-nodes①"></a>[contiguous exclusive <code>Text</code> nodes](#contiguous-exclusive-text-nodes) (excluding itself), in <a id="ref-for-concept-tree-order①⑨"></a>[tree order](#concept-tree-order).

4.  <a id="ref-for-concept-cd-replace③"></a>[Replace data](#concept-cd-replace) of <var>node</var> with <var>length</var>, 0, and <var>data</var>.

5.  Let <var>currentNode</var> be <var>node</var>’s <a id="ref-for-concept-tree-next-sibling①⓪"></a>[next sibling](#concept-tree-next-sibling).

6.  While <var>currentNode</var> is an <a id="ref-for-exclusive-text-node⑦"></a>[exclusive <code>Text</code> node](#exclusive-text-node):

    1.  For each <a id="ref-for-concept-live-range④"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-start-node②"></a>[start node](https://dom.spec.whatwg.org/#concept-range-start-node) is <var>currentNode</var>: add <var>length</var> to its <a id="ref-for-concept-range-start-offset④"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) and set its <a id="ref-for-concept-range-start-node③"></a>[start node](https://dom.spec.whatwg.org/#concept-range-start-node) to <var>node</var>.

    2.  For each <a id="ref-for-concept-live-range⑤"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-end-node②"></a>[end node](https://dom.spec.whatwg.org/#concept-range-end-node) is <var>currentNode</var>: add <var>length</var> to its <a id="ref-for-concept-range-end-offset④"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) and set its <a id="ref-for-concept-range-end-node③"></a>[end node](https://dom.spec.whatwg.org/#concept-range-end-node) to <var>node</var>.

    3.  For each <a id="ref-for-concept-live-range⑥"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-start-node④"></a>[start node](https://dom.spec.whatwg.org/#concept-range-start-node) is <var>currentNode</var>’s <a id="ref-for-concept-tree-parent②⑤"></a>[parent](#concept-tree-parent) and <a id="ref-for-concept-range-start-offset⑤"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) is <var>currentNode</var>’s <a id="ref-for-concept-tree-index⑥"></a>[index](#concept-tree-index): set its <a id="ref-for-concept-range-start-node⑤"></a>[start node](https://dom.spec.whatwg.org/#concept-range-start-node) to <var>node</var> and its <a id="ref-for-concept-range-start-offset⑥"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) to <var>length</var>.

    4.  For each <a id="ref-for-concept-live-range⑦"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-end-node④"></a>[end node](https://dom.spec.whatwg.org/#concept-range-end-node) is <var>currentNode</var>’s <a id="ref-for-concept-tree-parent②⑥"></a>[parent](#concept-tree-parent) and <a id="ref-for-concept-range-end-offset⑤"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) is <var>currentNode</var>’s <a id="ref-for-concept-tree-index⑦"></a>[index](#concept-tree-index): set its <a id="ref-for-concept-range-end-node⑤"></a>[end node](https://dom.spec.whatwg.org/#concept-range-end-node) to <var>node</var> and its <a id="ref-for-concept-range-end-offset⑥"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) to <var>length</var>.

    5.  Add <var>currentNode</var>’s <a id="ref-for-concept-node-length④"></a>[length](#concept-node-length) to <var>length</var>.

    6.  Set <var>currentNode</var> to its <a id="ref-for-concept-tree-next-sibling①①"></a>[next sibling](#concept-tree-next-sibling).

7.  <a id="ref-for-concept-node-remove⑨"></a>[Remove](#concept-node-remove) <var>node</var>’s <a id="ref-for-contiguous-exclusive-text-nodes②"></a>[contiguous exclusive <code>Text</code> nodes](#contiguous-exclusive-text-nodes) (excluding itself), in <a id="ref-for-concept-tree-order②⓪"></a>[tree order](#concept-tree-order).

------------------------------------------------------------------------

<strong><code><var>node</var> . <a id="ref-for-dom-node-clonenode①"></a>[cloneNode](#dom-node-clonenode)(\[<var>subtree</var> = false\])</code></strong>

Returns a copy of <var>node</var>. If <var>subtree</var> is true, the copy also includes the <var>node</var>’s <a id="ref-for-concept-tree-descendant①②"></a>[descendants](#concept-tree-descendant).

<strong><code><var>node</var> . <a id="ref-for-dom-node-isequalnode①"></a>[isEqualNode](#dom-node-isequalnode)(<var>otherNode</var>)</code></strong>

Returns whether <var>node</var> and <var>otherNode</var> have the same properties.

<a id="ref-for-other-applicable-specifications⑥"></a>[Specifications](https://dom.spec.whatwg.org/#other-applicable-specifications) may define <a id="concept-node-clone-ext"></a><strong>cloning steps</strong> for all or some <a id="ref-for-concept-node①③④"></a>[nodes](#concept-node). The algorithm is passed <var>node</var>, <var>copy</var>, and <var>subtree</var> as indicated in the <a id="ref-for-concept-node-clone"></a>[clone a node](#concept-node-clone) algorithm.

<strong>Note:</strong>

HTML defines <a id="ref-for-concept-node-clone-ext"></a>[cloning steps](#concept-node-clone-ext) for several elements, such as <code><a id="ref-for-the-input-element①"></a>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code>, <code><a id="ref-for-script③"></a>[script](https://html.spec.whatwg.org/multipage/scripting.html#script)</code>, and <code><a id="ref-for-the-template-element"></a>[template](https://html.spec.whatwg.org/multipage/scripting.html#the-template-element)</code>. SVG ought to do the same for its <code><a id="ref-for-script④"></a>[script](https://html.spec.whatwg.org/multipage/scripting.html#script)</code> elements, but does not.

To <a id="concept-node-clone"></a><strong>clone a node</strong> given a <a id="ref-for-concept-node①③⑤"></a>[node](#concept-node) <var>node</var> and an optional <a id="ref-for-concept-document②①"></a>[document](#concept-document) <a id="clone-a-node-document"></a><strong><var>document</var></strong> (default <var>node</var>’s <a id="ref-for-concept-node-document②⑦"></a>[node document](#concept-node-document)), boolean <a id="clone-a-node-subtree"></a><strong><var>subtree</var></strong> (default false), <a id="ref-for-concept-node①③⑥"></a>[node](#concept-node)-or-null <a id="clone-a-node-parent"></a><strong><var>parent</var></strong> (default null), and null or a <code><a id="ref-for-customelementregistry②"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code> object <a id="clone-a-node-fallbackregistry"></a><strong><var>fallbackRegistry</var></strong> (default null):

1.  <a id="ref-for-assert⑦"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>node</var> is not a <a id="ref-for-concept-document②②"></a>[document](#concept-document) or <var>node</var> is <var>document</var>.

2.  Let <var>copy</var> be the result of <a id="ref-for-clone-a-single-node"></a>[cloning a single node](#clone-a-single-node) given <var>node</var>, <var>document</var>, and <var>fallbackRegistry</var>.

3.  Run any <a id="ref-for-concept-node-clone-ext①"></a>[cloning steps](#concept-node-clone-ext) defined for <var>node</var> in <a id="ref-for-other-applicable-specifications⑦"></a>[other applicable specifications](https://dom.spec.whatwg.org/#other-applicable-specifications) and pass <var>node</var>, <var>copy</var>, and <var>subtree</var> as parameters.

4.  If <var>parent</var> is non-null, then <a id="ref-for-concept-node-append②"></a>[append](#concept-node-append) <var>copy</var> to <var>parent</var>.

5.  If <var>subtree</var> is true, then for each <var>child</var> of <var>node</var>’s <a id="ref-for-concept-tree-child④⑧"></a>[children](#concept-tree-child), in <a id="ref-for-concept-tree-order②①"></a>[tree order](#concept-tree-order): <a id="ref-for-concept-node-clone①"></a>[clone a node](#concept-node-clone) given <var>child</var> with <a id="ref-for-clone-a-node-document"></a>[<em>document</em>](#clone-a-node-document) set to <var>document</var>, <a id="ref-for-clone-a-node-subtree"></a>[<em>subtree</em>](#clone-a-node-subtree) set to <var>subtree</var>, <a id="ref-for-clone-a-node-parent"></a>[<em>parent</em>](#clone-a-node-parent) set to <var>copy</var>, and <a id="ref-for-clone-a-node-fallbackregistry"></a>[<em>fallbackRegistry</em>](#clone-a-node-fallbackregistry) set to <var>fallbackRegistry</var>.

6.  If <var>node</var> is an <a id="ref-for-concept-element④③"></a>[element](#concept-element), <var>node</var> is a <a id="ref-for-element-shadow-host②"></a>[shadow host](#element-shadow-host), and <var>node</var>’s <a id="ref-for-concept-element-shadow-root①"></a>[shadow root](#concept-element-shadow-root)’s <a id="ref-for-shadowroot-clonable"></a>[clonable](#shadowroot-clonable) is true:

    1.  <a id="ref-for-assert⑧"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>copy</var> is not a <a id="ref-for-element-shadow-host③"></a>[shadow host](#element-shadow-host).

    2.  Let <var>shadowRootRegistry</var> be <var>node</var>’s <a id="ref-for-concept-element-shadow-root②"></a>[shadow root](#concept-element-shadow-root)’s <a id="ref-for-shadowroot-custom-element-registry④"></a>[custom element registry](#shadowroot-custom-element-registry).

    3.  If <var>shadowRootRegistry</var> <a id="ref-for-is-a-global-custom-element-registry"></a>[is a global custom element registry](#is-a-global-custom-element-registry), then set <var>shadowRootRegistry</var> to <var>document</var>’s <a id="ref-for-document-custom-element-registry①"></a>[custom element registry](#document-custom-element-registry)’s <a id="ref-for-effective-global-custom-element-registry"></a>[effective global custom element registry](#effective-global-custom-element-registry).

    4.  <a id="ref-for-concept-attach-a-shadow-root"></a>[Attach a shadow root](#concept-attach-a-shadow-root) with <var>copy</var>, <var>node</var>’s <a id="ref-for-concept-element-shadow-root③"></a>[shadow root](#concept-element-shadow-root)’s <a id="ref-for-shadowroot-mode④"></a>[mode](#shadowroot-mode), <var>node</var>’s <a id="ref-for-concept-element-shadow-root④"></a>[shadow root](#concept-element-shadow-root)’s <a id="ref-for-shadowroot-delegates-focus"></a>[delegates focus](#shadowroot-delegates-focus), <var>node</var>’s <a id="ref-for-concept-element-shadow-root⑤"></a>[shadow root](#concept-element-shadow-root)’s <a id="ref-for-shadowroot-serializable"></a>[serializable](#shadowroot-serializable), <var>node</var>’s <a id="ref-for-concept-element-shadow-root⑥"></a>[shadow root](#concept-element-shadow-root)’s <a id="ref-for-shadowroot-slot-assignment④"></a>[slot assignment](#shadowroot-slot-assignment), true, and <var>shadowRootRegistry</var>.

    5.  Set <var>copy</var>’s <a id="ref-for-concept-element-shadow-root⑦"></a>[shadow root](#concept-element-shadow-root)’s <a id="ref-for-shadowroot-declarative"></a>[declarative](#shadowroot-declarative) to <var>node</var>’s <a id="ref-for-concept-element-shadow-root⑧"></a>[shadow root](#concept-element-shadow-root)’s <a id="ref-for-shadowroot-declarative①"></a>[declarative](#shadowroot-declarative).

    6.  Set <var>copy</var>’s <a id="ref-for-concept-element-shadow-root⑨"></a>[shadow root](#concept-element-shadow-root)’s <a id="ref-for-shadowroot-keep-custom-element-registry-null"></a>[keep custom element registry null](#shadowroot-keep-custom-element-registry-null) to <var>node</var>’s <a id="ref-for-concept-element-shadow-root①⓪"></a>[shadow root](#concept-element-shadow-root)’s <a id="ref-for-shadowroot-keep-custom-element-registry-null①"></a>[keep custom element registry null](#shadowroot-keep-custom-element-registry-null).

    7.  For each <var>child</var> of <var>node</var>’s <a id="ref-for-concept-element-shadow-root①①"></a>[shadow root](#concept-element-shadow-root)’s <a id="ref-for-concept-tree-child④⑨"></a>[children](#concept-tree-child), in <a id="ref-for-concept-tree-order②②"></a>[tree order](#concept-tree-order): <a id="ref-for-concept-node-clone②"></a>[clone a node](#concept-node-clone) given <var>child</var> with <a id="ref-for-clone-a-node-document①"></a>[<em>document</em>](#clone-a-node-document) set to <var>document</var>, <a id="ref-for-clone-a-node-subtree①"></a>[<em>subtree</em>](#clone-a-node-subtree) set to true, and <a id="ref-for-clone-a-node-parent①"></a>[<em>parent</em>](#clone-a-node-parent) set to <var>copy</var>’s <a id="ref-for-concept-element-shadow-root①②"></a>[shadow root](#concept-element-shadow-root).

        <strong>Note:</strong>

        This intentionally does not pass the <a id="ref-for-clone-a-node-fallbackregistry①"></a>[<em>fallbackRegistry</em>](#clone-a-node-fallbackregistry) argument.

7.  Return <var>copy</var>.

To <a id="clone-a-single-node"></a><strong>clone a single node</strong> given a <a id="ref-for-concept-node①③⑦"></a>[node](#concept-node) <var>node</var>, <a id="ref-for-concept-document②③"></a>[document](#concept-document) <var>document</var>, and null or a <code><a id="ref-for-customelementregistry③"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code> object <var>fallbackRegistry</var>:

1.  Let <var>copy</var> be null.

2.  If <var>node</var> is an <a id="ref-for-concept-element④④"></a>[element](#concept-element):

    1.  Let <var>registry</var> be <var>node</var>’s <a id="ref-for-element-custom-element-registry③"></a>[custom element registry](#element-custom-element-registry).

    2.  If <var>registry</var> is null, then set <var>registry</var> to <var>fallbackRegistry</var>.

    3.  If <var>registry</var> <a id="ref-for-is-a-global-custom-element-registry①"></a>[is a global custom element registry](#is-a-global-custom-element-registry), then set <var>registry</var> to <var>document</var>’s <a id="ref-for-document-custom-element-registry②"></a>[custom element registry](#document-custom-element-registry)’s <a id="ref-for-effective-global-custom-element-registry①"></a>[effective global custom element registry](#effective-global-custom-element-registry).

    4.  Set <var>copy</var> to the result of <a id="ref-for-concept-create-element"></a>[creating an element](#concept-create-element), given <var>document</var>, <var>node</var>’s <a id="ref-for-concept-element-local-name①"></a>[local name](#concept-element-local-name), <var>node</var>’s <a id="ref-for-concept-element-namespace"></a>[namespace](#concept-element-namespace), <var>node</var>’s <a id="ref-for-concept-element-namespace-prefix"></a>[namespace prefix](#concept-element-namespace-prefix), <var>node</var>’s <a id="ref-for-concept-element-is-value"></a>[<code>is</code> value](#concept-element-is-value), false, and <var>registry</var>.

    5.  <a id="ref-for-list-iterate②③"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>attribute</var> of <var>node</var>’s <a id="ref-for-concept-element-attribute"></a>[attribute list](#concept-element-attribute):

        1.  Let <var>copyAttribute</var> be the result of <a id="ref-for-clone-a-single-node①"></a>[cloning a single node](#clone-a-single-node) given <var>attribute</var>, <var>document</var>, and null.

        2.  <a id="ref-for-concept-element-attributes-append"></a>[Append](#concept-element-attributes-append) <var>copyAttribute</var> to <var>copy</var>.

3.  Otherwise, if <var>node</var> is a <a id="ref-for-concept-document②④"></a>[document](#concept-document), set <var>copy</var> to the result of <a id="ref-for-create-a-document①"></a>[creating a document](#create-a-document) that implements the same interfaces as <var>node</var>, given <var>document</var>’s <a id="ref-for-concept-relevant-realm②"></a>[relevant realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm).

4.  Otherwise, set <var>copy</var> to the result of <a id="ref-for-create-a-node"></a>[creating a node](#create-a-node) that <a id="ref-for-implements①⓪"></a>[implements](https://webidl.spec.whatwg.org/#implements) the same interfaces as <var>node</var>, given <var>document</var>.

5.  Fulfill these additional requirements, switching on the interface <var>node</var> <a id="ref-for-implements①①"></a>[implements](https://webidl.spec.whatwg.org/#implements):

    <strong><code><a id="ref-for-document①④"></a>[Document](#document)</code></strong>

    1.  Set <var>copy</var>’s <a id="ref-for-concept-document-encoding"></a>[encoding](#concept-document-encoding), <a id="ref-for-concept-document-content-type"></a>[content type](#concept-document-content-type), <a id="ref-for-concept-document-url"></a>[URL](#concept-document-url), <a id="ref-for-concept-document-origin"></a>[origin](#concept-document-origin), <a id="ref-for-concept-document-type"></a>[type](#concept-document-type), <a id="ref-for-concept-document-mode"></a>[mode](#concept-document-mode), and <a id="ref-for-document-allow-declarative-shadow-roots"></a>[allow declarative shadow roots](#document-allow-declarative-shadow-roots), to those of <var>node</var>.

    2.  If <var>node</var>’s <a id="ref-for-document-custom-element-registry③"></a>[custom element registry](#document-custom-element-registry)’s <a id="ref-for-is-scoped②"></a>[is scoped](https://html.spec.whatwg.org/multipage/custom-elements.html#is-scoped) is true, then set <var>copy</var>’s <a id="ref-for-document-custom-element-registry④"></a>[custom element registry](#document-custom-element-registry) to <var>node</var>’s <a id="ref-for-document-custom-element-registry⑤"></a>[custom element registry](#document-custom-element-registry).

    <strong><code><a id="ref-for-documenttype①①"></a>[DocumentType](#documenttype)</code></strong>

    Set <var>copy</var>’s <a id="ref-for-concept-doctype-name②"></a>[name](#concept-doctype-name), <a id="ref-for-concept-doctype-publicid"></a>[public ID](#concept-doctype-publicid), and <a id="ref-for-concept-doctype-systemid"></a>[system ID](#concept-doctype-systemid) to those of <var>node</var>.

    <strong><code><a id="ref-for-attr①③"></a>[Attr](#attr)</code></strong>

    Set <var>copy</var>’s <a id="ref-for-concept-attribute-namespace②"></a>[namespace](#concept-attribute-namespace), <a id="ref-for-concept-attribute-namespace-prefix"></a>[namespace prefix](#concept-attribute-namespace-prefix), <a id="ref-for-concept-attribute-local-name②"></a>[local name](#concept-attribute-local-name), and <a id="ref-for-concept-attribute-value⑦"></a>[value](#concept-attribute-value) to those of <var>node</var>.

    <strong><code><a id="ref-for-text①⑧"></a>[Text](#text)</code></strong>

    <strong><code><a id="ref-for-comment⑨"></a>[Comment](#comment)</code></strong>

    Set <var>copy</var>’s <a id="ref-for-concept-cd-data⑧"></a>[data](#concept-cd-data) to that of <var>node</var>.

    <strong><code><a id="ref-for-processinginstruction⑨"></a>[ProcessingInstruction](#processinginstruction)</code></strong>

    Set <var>copy</var>’s <a id="ref-for-concept-pi-target②"></a>[target](#concept-pi-target) and <a id="ref-for-concept-cd-data⑨"></a>[data](#concept-cd-data) to those of <var>node</var>.

    <strong>Otherwise</strong>

    Do nothing.

6.  <a id="ref-for-assert⑨"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>copy</var> is a <a id="ref-for-concept-node①③⑧"></a>[node](#concept-node).

7.  If <var>node</var> is a <a id="ref-for-concept-document②⑤"></a>[document](#concept-document), then set <var>document</var> to <var>copy</var>.

8.  Set <var>copy</var>’s <a id="ref-for-concept-node-document②⑧"></a>[node document](#concept-node-document) to <var>document</var>.

9.  Return <var>copy</var>.

The <a id="dom-node-clonenode"></a><strong><code>cloneNode(<var>subtree</var>)</code></strong> method steps are:

1.  If <a id="ref-for-this①⓪⑨"></a>[this](https://webidl.spec.whatwg.org/#this) is a <a id="ref-for-concept-shadow-root②⓪"></a>[shadow root](#concept-shadow-root), then <a id="ref-for-dfn-throw③⑥"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException③⑧"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  Return the result of <a id="ref-for-concept-node-clone③"></a>[cloning a node](#concept-node-clone) given <a id="ref-for-this①①⓪"></a>[this](https://webidl.spec.whatwg.org/#this) with <a id="ref-for-clone-a-node-subtree②"></a>[<em>subtree</em>](#clone-a-node-subtree) set to <var>subtree</var>.

A <a id="ref-for-concept-node①③⑨"></a>[node](#concept-node) <var>A</var> <a id="concept-node-equals"></a><strong>equals</strong> a <a id="ref-for-concept-node①④⓪"></a>[node](#concept-node) <var>B</var> if all of the following conditions are true:

- <var>A</var> and <var>B</var> <a id="ref-for-implements①②"></a>[implement](https://webidl.spec.whatwg.org/#implements) the same interfaces.

- The following are equal, switching on the interface <var>A</var> <a id="ref-for-implements①③"></a>[implements](https://webidl.spec.whatwg.org/#implements):

  <strong><code><a id="ref-for-documenttype①②"></a>[DocumentType](#documenttype)</code></strong>

  Its <a id="ref-for-concept-doctype-name③"></a>[name](#concept-doctype-name), <a id="ref-for-concept-doctype-publicid①"></a>[public ID](#concept-doctype-publicid), and <a id="ref-for-concept-doctype-systemid①"></a>[system ID](#concept-doctype-systemid).
  <strong><code><a id="ref-for-element③⑥"></a>[Element](#element)</code></strong>

  Its <a id="ref-for-concept-element-namespace①"></a>[namespace](#concept-element-namespace), <a id="ref-for-concept-element-namespace-prefix①"></a>[namespace prefix](#concept-element-namespace-prefix), <a id="ref-for-concept-element-local-name②"></a>[local name](#concept-element-local-name), and its <a id="ref-for-concept-element-attribute①"></a>[attribute list](#concept-element-attribute)’s <a id="ref-for-list-size④"></a>[size](https://infra.spec.whatwg.org/#list-size).
  <strong><code><a id="ref-for-attr①④"></a>[Attr](#attr)</code></strong>

  Its <a id="ref-for-concept-attribute-namespace③"></a>[namespace](#concept-attribute-namespace), <a id="ref-for-concept-attribute-local-name③"></a>[local name](#concept-attribute-local-name), and <a id="ref-for-concept-attribute-value⑧"></a>[value](#concept-attribute-value).
  <strong><code><a id="ref-for-processinginstruction①⓪"></a>[ProcessingInstruction](#processinginstruction)</code></strong>

  Its <a id="ref-for-concept-pi-target③"></a>[target](#concept-pi-target) and <a id="ref-for-concept-cd-data①⓪"></a>[data](#concept-cd-data).
  <strong><code><a id="ref-for-text①⑨"></a>[Text](#text)</code></strong>

  <strong><code><a id="ref-for-comment①⓪"></a>[Comment](#comment)</code></strong>

  Its <a id="ref-for-concept-cd-data①①"></a>[data](#concept-cd-data).
  <strong>Otherwise</strong>

  —

- If <var>A</var> is an <a id="ref-for-concept-element④⑤"></a>[element](#concept-element), each <a id="ref-for-concept-attribute⑨"></a>[attribute](#concept-attribute) in its <a id="ref-for-concept-element-attribute②"></a>[attribute list](#concept-element-attribute) <a id="ref-for-concept-node-equals"></a>[equals](#concept-node-equals) an <a id="ref-for-concept-attribute①⓪"></a>[attribute](#concept-attribute) in <var>B</var>’s <a id="ref-for-concept-element-attribute③"></a>[attribute list](#concept-element-attribute).

- <var>A</var> and <var>B</var> have the same number of <a id="ref-for-concept-tree-child⑤⓪"></a>[children](#concept-tree-child).

- Each <a id="ref-for-concept-tree-child⑤①"></a>[child](#concept-tree-child) of <var>A</var> <a id="ref-for-concept-node-equals①"></a>[equals](#concept-node-equals) the <a id="ref-for-concept-tree-child⑤②"></a>[child](#concept-tree-child) of <var>B</var> at the identical <a id="ref-for-concept-tree-index⑧"></a>[index](#concept-tree-index).

The <a id="dom-node-isequalnode"></a><strong><code>isEqualNode(<var>otherNode</var>)</code></strong> method steps are to return true if <var>otherNode</var> is non-null and <a id="ref-for-this①①①"></a>[this](https://webidl.spec.whatwg.org/#this) <a id="ref-for-concept-node-equals②"></a>[equals](#concept-node-equals) <var>otherNode</var>; otherwise false.

The <a id="dom-node-issamenode"></a><strong><code>isSameNode(<var>otherNode</var>)</code></strong> method steps are to return true if <var>otherNode</var> is <a id="ref-for-this①①②"></a>[this](https://webidl.spec.whatwg.org/#this); otherwise false.

------------------------------------------------------------------------

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-comparedocumentposition①"></a>[compareDocumentPosition(other)](#dom-node-comparedocumentposition)</code></code></strong>

Returns a bitmask indicating the position of <var>other</var> relative to <var>node</var>. These are the bits that can be set:

<strong><code><code><a id="ref-for-node④⑨"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-document&#95;position&#95;disconnected①"></a>[DOCUMENT_POSITION_DISCONNECTED](#dom-node-document_position_disconnected)</code></code> (1)</strong>

Set when <var>node</var> and <var>other</var> are not in the same <a id="ref-for-concept-tree①⑨"></a>[tree](#concept-tree).

<strong><code><code><a id="ref-for-node⑤⓪"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-document&#95;position&#95;preceding①"></a>[DOCUMENT_POSITION_PRECEDING](#dom-node-document_position_preceding)</code></code> (2)</strong>

Set when <var>other</var> is <a id="ref-for-concept-tree-preceding⑦"></a>[preceding](#concept-tree-preceding) <var>node</var>.

<strong><code><code><a id="ref-for-node⑤①"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-document&#95;position&#95;following①"></a>[DOCUMENT_POSITION_FOLLOWING](#dom-node-document_position_following)</code></code> (4)</strong>

Set when <var>other</var> is <a id="ref-for-concept-tree-following⑧"></a>[following](#concept-tree-following) <var>node</var>.

<strong><code><code><a id="ref-for-node⑤②"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-document&#95;position&#95;contains①"></a>[DOCUMENT_POSITION_CONTAINS](#dom-node-document_position_contains)</code></code> (8)</strong>

Set when <var>other</var> is an <a id="ref-for-concept-tree-ancestor③"></a>[ancestor](#concept-tree-ancestor) of <var>node</var>.

<strong><code><code><a id="ref-for-node⑤③"></a>[Node](#node)</code> . <code><a id="ref-for-dom-node-document&#95;position&#95;contained&#95;by①"></a>[DOCUMENT_POSITION_CONTAINED_BY](#dom-node-document_position_contained_by)</code></code> (16, 10 in hexadecimal)</strong>

Set when <var>other</var> is a <a id="ref-for-concept-tree-descendant①③"></a>[descendant](#concept-tree-descendant) of <var>node</var>.

<strong><code><var>node</var> . <code><a id="ref-for-dom-node-contains①"></a>[contains(other)](#dom-node-contains)</code></code></strong>

Returns true if <var>other</var> is an <a id="ref-for-concept-tree-inclusive-descendant③"></a>[inclusive descendant](#concept-tree-inclusive-descendant) of <var>node</var>; otherwise false.

These are the constants <code><a id="ref-for-dom-node-comparedocumentposition②"></a>[compareDocumentPosition()](#dom-node-comparedocumentposition)</code> returns as mask:

- <a id="dom-node-document&#95;position&#95;disconnected"></a><strong><code>DOCUMENT&#95;POSITION&#95;DISCONNECTED</code></strong> (1);
- <a id="dom-node-document&#95;position&#95;preceding"></a><strong><code>DOCUMENT&#95;POSITION&#95;PRECEDING</code></strong> (2);
- <a id="dom-node-document&#95;position&#95;following"></a><strong><code>DOCUMENT&#95;POSITION&#95;FOLLOWING</code></strong> (4);
- <a id="dom-node-document&#95;position&#95;contains"></a><strong><code>DOCUMENT&#95;POSITION&#95;CONTAINS</code></strong> (8);
- <a id="dom-node-document&#95;position&#95;contained&#95;by"></a><strong><code>DOCUMENT&#95;POSITION&#95;CONTAINED&#95;BY</code></strong> (16, 10 in hexadecimal);
- <a id="dom-node-document&#95;position&#95;implementation&#95;specific"></a><strong><code>DOCUMENT&#95;POSITION&#95;IMPLEMENTATION&#95;SPECIFIC</code></strong> (32, 20 in hexadecimal).

The <a id="dom-node-comparedocumentposition"></a><strong><code>compareDocumentPosition(<var>other</var>)</code></strong> method steps are:

1.  If <a id="ref-for-this①①③"></a>[this](https://webidl.spec.whatwg.org/#this) is <var>other</var>, then return zero.

2.  Let <var>node1</var> be <var>other</var> and <var>node2</var> be <a id="ref-for-this①①④"></a>[this](https://webidl.spec.whatwg.org/#this).

3.  Let <var>attr1</var> and <var>attr2</var> be null.

4.  If <var>node1</var> is an <a id="ref-for-concept-attribute①①"></a>[attribute](#concept-attribute), then set <var>attr1</var> to <var>node1</var> and <var>node1</var> to <var>attr1</var>’s <a id="ref-for-concept-attribute-element"></a>[element](#concept-attribute-element).

5.  If <var>node2</var> is an <a id="ref-for-concept-attribute①②"></a>[attribute](#concept-attribute):

    1.  Set <var>attr2</var> to <var>node2</var> and <var>node2</var> to <var>attr2</var>’s <a id="ref-for-concept-attribute-element①"></a>[element](#concept-attribute-element).

    2.  If <var>attr1</var> and <var>node1</var> are non-null, and <var>node2</var> is <var>node1</var>:

        1.  <a id="ref-for-list-iterate②④"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>attr</var> of <var>node2</var>’s <a id="ref-for-concept-element-attribute④"></a>[attribute list](#concept-element-attribute):

            1.  If <var>attr</var> <a id="ref-for-concept-node-equals③"></a>[equals](#concept-node-equals) <var>attr1</var>, then return the result of adding <code><a id="ref-for-dom-node-document&#95;position&#95;implementation&#95;specific①"></a>[DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC](#dom-node-document_position_implementation_specific)</code> and <code><a id="ref-for-dom-node-document&#95;position&#95;preceding②"></a>[DOCUMENT_POSITION_PRECEDING](#dom-node-document_position_preceding)</code>.

            2.  If <var>attr</var> <a id="ref-for-concept-node-equals④"></a>[equals](#concept-node-equals) <var>attr2</var>, then return the result of adding <code><a id="ref-for-dom-node-document&#95;position&#95;implementation&#95;specific②"></a>[DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC](#dom-node-document_position_implementation_specific)</code> and <code><a id="ref-for-dom-node-document&#95;position&#95;following②"></a>[DOCUMENT_POSITION_FOLLOWING](#dom-node-document_position_following)</code>.

6.  If <var>node1</var> or <var>node2</var> is null, or <var>node1</var>’s <a id="ref-for-concept-tree-root②⑤"></a>[root](#concept-tree-root) is not <var>node2</var>’s <a id="ref-for-concept-tree-root②⑥"></a>[root](#concept-tree-root), then return the result of adding <code><a id="ref-for-dom-node-document&#95;position&#95;disconnected②"></a>[DOCUMENT_POSITION_DISCONNECTED](#dom-node-document_position_disconnected)</code>, <code><a id="ref-for-dom-node-document&#95;position&#95;implementation&#95;specific③"></a>[DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC](#dom-node-document_position_implementation_specific)</code>, and either <code><a id="ref-for-dom-node-document&#95;position&#95;preceding③"></a>[DOCUMENT_POSITION_PRECEDING](#dom-node-document_position_preceding)</code> or <code><a id="ref-for-dom-node-document&#95;position&#95;following③"></a>[DOCUMENT_POSITION_FOLLOWING](#dom-node-document_position_following)</code>, with the constraint that this is to be consistent, together.

    <strong>Note:</strong>

    Whether to return <code><a id="ref-for-dom-node-document&#95;position&#95;preceding④"></a>[DOCUMENT_POSITION_PRECEDING](#dom-node-document_position_preceding)</code> or <code><a id="ref-for-dom-node-document&#95;position&#95;following④"></a>[DOCUMENT_POSITION_FOLLOWING](#dom-node-document_position_following)</code> is typically implemented via pointer comparison. In JavaScript implementations a cached <code>Math.random()</code> value can be used.

7.  If <var>node1</var> is an <a id="ref-for-concept-tree-ancestor④"></a>[ancestor](#concept-tree-ancestor) of <var>node2</var> and <var>attr1</var> is null, or <var>node1</var> is <var>node2</var> and <var>attr2</var> is non-null, then return the result of adding <code><a id="ref-for-dom-node-document&#95;position&#95;contains②"></a>[DOCUMENT_POSITION_CONTAINS](#dom-node-document_position_contains)</code> to <code><a id="ref-for-dom-node-document&#95;position&#95;preceding⑤"></a>[DOCUMENT_POSITION_PRECEDING](#dom-node-document_position_preceding)</code>.

8.  If <var>node1</var> is a <a id="ref-for-concept-tree-descendant①④"></a>[descendant](#concept-tree-descendant) of <var>node2</var> and <var>attr2</var> is null, or <var>node1</var> is <var>node2</var> and <var>attr1</var> is non-null, then return the result of adding <code><a id="ref-for-dom-node-document&#95;position&#95;contained&#95;by②"></a>[DOCUMENT_POSITION_CONTAINED_BY](#dom-node-document_position_contained_by)</code> to <code><a id="ref-for-dom-node-document&#95;position&#95;following⑤"></a>[DOCUMENT_POSITION_FOLLOWING](#dom-node-document_position_following)</code>.

9.  If <var>node1</var> is <a id="ref-for-concept-tree-preceding⑧"></a>[preceding](#concept-tree-preceding) <var>node2</var>, then return <code><a id="ref-for-dom-node-document&#95;position&#95;preceding⑥"></a>[DOCUMENT_POSITION_PRECEDING](#dom-node-document_position_preceding)</code>.

    <strong>Note:</strong>

    Due to the way <a id="ref-for-concept-attribute①③"></a>[attributes](#concept-attribute) are handled in this algorithm this results in a <a id="ref-for-concept-node①④①"></a>[node](#concept-node)’s <a id="ref-for-concept-attribute①④"></a>[attributes](#concept-attribute) counting as <a id="ref-for-concept-tree-preceding⑨"></a>[preceding](#concept-tree-preceding) that <a id="ref-for-concept-node①④②"></a>[node](#concept-node)’s <a id="ref-for-concept-tree-child⑤③"></a>[children](#concept-tree-child), despite <a id="ref-for-concept-attribute①⑤"></a>[attributes](#concept-attribute) not <a id="ref-for-concept-tree-participate⑨"></a>[participating](#concept-tree-participate) in the same <a id="ref-for-concept-tree②⓪"></a>[tree](#concept-tree).

10. Return <code><a id="ref-for-dom-node-document&#95;position&#95;following⑥"></a>[DOCUMENT_POSITION_FOLLOWING](#dom-node-document_position_following)</code>.

The <a id="dom-node-contains"></a><strong><code>contains(<var>other</var>)</code></strong> method steps are to return true if <var>other</var> is an <a id="ref-for-concept-tree-inclusive-descendant④"></a>[inclusive descendant](#concept-tree-inclusive-descendant) of <a id="ref-for-this①①⑤"></a>[this](https://webidl.spec.whatwg.org/#this); otherwise false (including when <var>other</var> is null).

------------------------------------------------------------------------

To <a id="locate-a-namespace-prefix"></a><strong>locate a namespace prefix</strong> for an <var>element</var> using <var>namespace</var>, run these steps:

1.  If <var>element</var>’s <a id="ref-for-concept-element-namespace②"></a>[namespace](#concept-element-namespace) is <var>namespace</var> and its <a id="ref-for-concept-element-namespace-prefix②"></a>[namespace prefix](#concept-element-namespace-prefix) is non-null, then return its <a id="ref-for-concept-element-namespace-prefix③"></a>[namespace prefix](#concept-element-namespace-prefix).

2.  If <var>element</var> <a id="ref-for-concept-element-attribute-has②"></a>[has](#concept-element-attribute-has) an <a id="ref-for-concept-attribute①⑥"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-namespace-prefix①"></a>[namespace prefix](#concept-attribute-namespace-prefix) is "<code>xmlns</code>" and <a id="ref-for-concept-attribute-value⑨"></a>[value](#concept-attribute-value) is <var>namespace</var>, then return <var>element</var>’s first such <a id="ref-for-concept-attribute①⑦"></a>[attribute](#concept-attribute)’s <a id="ref-for-concept-attribute-local-name④"></a>[local name](#concept-attribute-local-name).

3.  If <var>element</var>’s <a id="ref-for-parent-element②"></a>[parent element](#parent-element) is non-null, then return the result of running <a id="ref-for-locate-a-namespace-prefix"></a>[locate a namespace prefix](#locate-a-namespace-prefix) on that <a id="ref-for-concept-element④⑥"></a>[element](#concept-element) using <var>namespace</var>.

4.  Return null.

To <a id="locate-a-namespace"></a><strong>locate a namespace</strong> for a <var>node</var> using <var>prefix</var>, switch on the interface <var>node</var> <a id="ref-for-implements①④"></a>[implements](https://webidl.spec.whatwg.org/#implements):

<strong><code><a id="ref-for-element③⑦"></a>[Element](#element)</code></strong>

1.  If <var>prefix</var> is "<code>xml</code>", then return the <a id="ref-for-xml-namespace①"></a>[XML namespace](https://infra.spec.whatwg.org/#xml-namespace).

2.  If <var>prefix</var> is "<code>xmlns</code>", then return the <a id="ref-for-xmlns-namespace②"></a>[XMLNS namespace](https://infra.spec.whatwg.org/#xmlns-namespace).

3.  If its <a id="ref-for-concept-element-namespace③"></a>[namespace](#concept-element-namespace) is non-null and its <a id="ref-for-concept-element-namespace-prefix④"></a>[namespace prefix](#concept-element-namespace-prefix) is <var>prefix</var>, then return <a id="ref-for-concept-element-namespace④"></a>[namespace](#concept-element-namespace).

4.  If it <a id="ref-for-concept-element-attribute-has③"></a>[has](#concept-element-attribute-has) an <a id="ref-for-concept-attribute①⑧"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-namespace④"></a>[namespace](#concept-attribute-namespace) is the <a id="ref-for-xmlns-namespace③"></a>[XMLNS namespace](https://infra.spec.whatwg.org/#xmlns-namespace), <a id="ref-for-concept-attribute-namespace-prefix②"></a>[namespace prefix](#concept-attribute-namespace-prefix) is "<code>xmlns</code>", and <a id="ref-for-concept-attribute-local-name⑤"></a>[local name](#concept-attribute-local-name) is <var>prefix</var>, or if <var>prefix</var> is null and it <a id="ref-for-concept-element-attribute-has④"></a>[has](#concept-element-attribute-has) an <a id="ref-for-concept-attribute①⑨"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-namespace⑤"></a>[namespace](#concept-attribute-namespace) is the <a id="ref-for-xmlns-namespace④"></a>[XMLNS namespace](https://infra.spec.whatwg.org/#xmlns-namespace), <a id="ref-for-concept-attribute-namespace-prefix③"></a>[namespace prefix](#concept-attribute-namespace-prefix) is null, and <a id="ref-for-concept-attribute-local-name⑥"></a>[local name](#concept-attribute-local-name) is "<code>xmlns</code>", then return its <a id="ref-for-concept-attribute-value①⓪"></a>[value](#concept-attribute-value) if it is not the empty string, and null otherwise.

5.  If its <a id="ref-for-parent-element③"></a>[parent element](#parent-element) is null, then return null.

6.  Return the result of running <a id="ref-for-locate-a-namespace"></a>[locate a namespace](#locate-a-namespace) on its <a id="ref-for-parent-element④"></a>[parent element](#parent-element) using <var>prefix</var>.

<strong><code><a id="ref-for-document①⑤"></a>[Document](#document)</code></strong>

1.  If its <a id="ref-for-document-element①"></a>[document element](#document-element) is null, then return null.

2.  Return the result of running <a id="ref-for-locate-a-namespace①"></a>[locate a namespace](#locate-a-namespace) on its <a id="ref-for-document-element②"></a>[document element](#document-element) using <var>prefix</var>.

<strong><code><a id="ref-for-documenttype①③"></a>[DocumentType](#documenttype)</code></strong>

<strong><code><a id="ref-for-documentfragment②⓪"></a>[DocumentFragment](#documentfragment)</code></strong>

Return null.

<strong><code><a id="ref-for-attr①⑤"></a>[Attr](#attr)</code></strong>

1.  If its <a id="ref-for-concept-attribute-element②"></a>[element](#concept-attribute-element) is null, then return null.

2.  Return the result of running <a id="ref-for-locate-a-namespace②"></a>[locate a namespace](#locate-a-namespace) on its <a id="ref-for-concept-attribute-element③"></a>[element](#concept-attribute-element) using <var>prefix</var>.

<strong>Otherwise</strong>

1.  If its <a id="ref-for-parent-element⑤"></a>[parent element](#parent-element) is null, then return null.

2.  Return the result of running <a id="ref-for-locate-a-namespace③"></a>[locate a namespace](#locate-a-namespace) on its <a id="ref-for-parent-element⑥"></a>[parent element](#parent-element) using <var>prefix</var>.

The <a id="dom-node-lookupprefix"></a><strong><code>lookupPrefix(<var>namespace</var>)</code></strong> method steps are:

1.  If <var>namespace</var> is null or the empty string, then return null.

2.  Switch on the interface <a id="ref-for-this①①⑥"></a>[this](https://webidl.spec.whatwg.org/#this) <a id="ref-for-implements①⑤"></a>[implements](https://webidl.spec.whatwg.org/#implements):

    <strong><code><a id="ref-for-element③⑧"></a>[Element](#element)</code></strong>

    Return the result of <a id="ref-for-locate-a-namespace-prefix①"></a>[locating a namespace prefix](#locate-a-namespace-prefix) for <a id="ref-for-this①①⑦"></a>[this](https://webidl.spec.whatwg.org/#this) using <var>namespace</var>.

    <strong><code><a id="ref-for-document①⑥"></a>[Document](#document)</code></strong>

    1.  If <a id="ref-for-this①①⑧"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-document-element③"></a>[document element](#document-element) is null, then return null.

    2.  Return the result of <a id="ref-for-locate-a-namespace-prefix②"></a>[locating a namespace prefix](#locate-a-namespace-prefix) for <a id="ref-for-this①①⑨"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-document-element④"></a>[document element](#document-element) using <var>namespace</var>.

    <strong><code><a id="ref-for-documenttype①④"></a>[DocumentType](#documenttype)</code></strong>

    <strong><code><a id="ref-for-documentfragment②①"></a>[DocumentFragment](#documentfragment)</code></strong>

    Return null.

    <strong><code><a id="ref-for-attr①⑥"></a>[Attr](#attr)</code></strong>

    1.  If <a id="ref-for-this①②⓪"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-attribute-element④"></a>[element](#concept-attribute-element) is null, then return null.

    2.  Return the result of <a id="ref-for-locate-a-namespace-prefix③"></a>[locating a namespace prefix](#locate-a-namespace-prefix) for <a id="ref-for-this①②①"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-attribute-element⑤"></a>[element](#concept-attribute-element) using <var>namespace</var>.

    <strong>Otherwise</strong>

    1.  If <a id="ref-for-this①②②"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-parent-element⑦"></a>[parent element](#parent-element) is null, then return null.

    2.  Return the result of <a id="ref-for-locate-a-namespace-prefix④"></a>[locating a namespace prefix](#locate-a-namespace-prefix) for <a id="ref-for-this①②③"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-parent-element⑧"></a>[parent element](#parent-element) using <var>namespace</var>.

The <a id="dom-node-lookupnamespaceuri"></a><strong><code>lookupNamespaceURI(<var>prefix</var>)</code></strong> method steps are:

1.  If <var>prefix</var> is the empty string, then set it to null.

2.  Return the result of running <a id="ref-for-locate-a-namespace④"></a>[locate a namespace](#locate-a-namespace) for <a id="ref-for-this①②④"></a>[this](https://webidl.spec.whatwg.org/#this) using <var>prefix</var>.

The <a id="dom-node-isdefaultnamespace"></a><strong><code>isDefaultNamespace(<var>namespace</var>)</code></strong> method steps are:

1.  If <var>namespace</var> is the empty string, then set it to null.

2.  Let <var>defaultNamespace</var> be the result of running <a id="ref-for-locate-a-namespace⑤"></a>[locate a namespace](#locate-a-namespace) for <a id="ref-for-this①②⑤"></a>[this](https://webidl.spec.whatwg.org/#this) using null.

3.  Return true if <var>defaultNamespace</var> is the same as <var>namespace</var>; otherwise false.

------------------------------------------------------------------------

The <a id="dom-node-insertbefore"></a><strong><code>insertBefore(<var>node</var>, <var>child</var>)</code></strong> method steps are to return the result of <a id="ref-for-concept-node-pre-insert⑤"></a>[pre-inserting](#concept-node-pre-insert) <var>node</var> into <a id="ref-for-this①②⑥"></a>[this](https://webidl.spec.whatwg.org/#this) before <var>child</var>.

The <a id="dom-node-appendchild"></a><strong><code>appendChild(<var>node</var>)</code></strong> method steps are to return the result of <a id="ref-for-concept-node-append③"></a>[appending](#concept-node-append) <var>node</var> to <a id="ref-for-this①②⑦"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-node-replacechild"></a><strong><code>replaceChild(<var>node</var>, <var>child</var>)</code></strong> method steps are to return the result of <a id="ref-for-concept-node-replace①"></a>[replacing](#concept-node-replace) <var>child</var> with <var>node</var> within <a id="ref-for-this①②⑧"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-node-removechild"></a><strong><code>removeChild(<var>child</var>)</code></strong> method steps are to return the result of <a id="ref-for-concept-node-pre-remove"></a>[pre-removing](#concept-node-pre-remove) <var>child</var> from <a id="ref-for-this①②⑨"></a>[this](https://webidl.spec.whatwg.org/#this).

------------------------------------------------------------------------

The <a id="concept-getelementsbytagname"></a><strong>list of elements with qualified name <var>qualifiedName</var></strong> for a <a id="ref-for-concept-node①④③"></a>[node](#concept-node) <var>root</var> is the <code><a id="ref-for-htmlcollection⑥"></a>[HTMLCollection](#htmlcollection)</code> returned by the following algorithm:

1.  If <var>qualifiedName</var> is U+002A (\*), then return an <code><a id="ref-for-htmlcollection⑦"></a>[HTMLCollection](#htmlcollection)</code> rooted at <var>root</var>, whose filter matches only <a id="ref-for-concept-tree-descendant①⑤"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element④⑦"></a>[elements](#concept-element).

2.  Otherwise, if <var>root</var>’s <a id="ref-for-concept-node-document②⑨"></a>[node document](#concept-node-document) is an <a id="ref-for-html-document"></a>[HTML document](#html-document), return an <code><a id="ref-for-htmlcollection⑧"></a>[HTMLCollection](#htmlcollection)</code> rooted at <var>root</var>, whose filter matches the following <a id="ref-for-concept-tree-descendant①⑥"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element④⑧"></a>[elements](#concept-element):

    - Whose <a id="ref-for-concept-element-namespace⑤"></a>[namespace](#concept-element-namespace) is the <a id="ref-for-html-namespace②"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) and whose <a id="ref-for-concept-element-qualified-name"></a>[qualified name](#concept-element-qualified-name) is <var>qualifiedName</var>, in <a id="ref-for-ascii-lowercase"></a>[ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).

    - Whose <a id="ref-for-concept-element-namespace⑥"></a>[namespace](#concept-element-namespace) is <em>not</em> the <a id="ref-for-html-namespace③"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) and whose <a id="ref-for-concept-element-qualified-name①"></a>[qualified name](#concept-element-qualified-name) is <var>qualifiedName</var>.

3.  Otherwise, return an <code><a id="ref-for-htmlcollection⑨"></a>[HTMLCollection](#htmlcollection)</code> rooted at <var>root</var>, whose filter matches <a id="ref-for-concept-tree-descendant①⑦"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element④⑨"></a>[elements](#concept-element) whose <a id="ref-for-concept-element-qualified-name②"></a>[qualified name](#concept-element-qualified-name) is <var>qualifiedName</var>.

When invoked with the same argument, and as long as <var>root</var>’s <a id="ref-for-concept-node-document③⓪"></a>[node document](#concept-node-document)’s <a id="ref-for-concept-document-type①"></a>[type](#concept-document-type) has not changed, the same <code><a id="ref-for-htmlcollection①⓪"></a>[HTMLCollection](#htmlcollection)</code> object may be returned as returned by an earlier call.

The <a id="concept-getelementsbytagnamens"></a><strong>list of elements with namespace <var>namespace</var> and local name <var>localName</var></strong> for a <a id="ref-for-concept-node①④④"></a>[node](#concept-node) <var>root</var> is the <code><a id="ref-for-htmlcollection①①"></a>[HTMLCollection](#htmlcollection)</code> returned by the following algorithm:

1.  If <var>namespace</var> is the empty string, then set it to null.

2.  If both <var>namespace</var> and <var>localName</var> are U+002A (\*), then return an <code><a id="ref-for-htmlcollection①②"></a>[HTMLCollection](#htmlcollection)</code> rooted at <var>root</var>, whose filter matches <a id="ref-for-concept-tree-descendant①⑧"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element⑤⓪"></a>[elements](#concept-element).

3.  If <var>namespace</var> is U+002A (\*), then return an <code><a id="ref-for-htmlcollection①③"></a>[HTMLCollection](#htmlcollection)</code> rooted at <var>root</var>, whose filter matches <a id="ref-for-concept-tree-descendant①⑨"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element⑤①"></a>[elements](#concept-element) whose <a id="ref-for-concept-element-local-name③"></a>[local name](#concept-element-local-name) is <var>localName</var>.

4.  If <var>localName</var> is U+002A (\*), then return an <code><a id="ref-for-htmlcollection①④"></a>[HTMLCollection](#htmlcollection)</code> rooted at <var>root</var>, whose filter matches <a id="ref-for-concept-tree-descendant②⓪"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element⑤②"></a>[elements](#concept-element) whose <a id="ref-for-concept-element-namespace⑦"></a>[namespace](#concept-element-namespace) is <var>namespace</var>.

5.  Return an <code><a id="ref-for-htmlcollection①⑤"></a>[HTMLCollection](#htmlcollection)</code> rooted at <var>root</var>, whose filter matches <a id="ref-for-concept-tree-descendant②①"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element⑤③"></a>[elements](#concept-element) whose <a id="ref-for-concept-element-namespace⑧"></a>[namespace](#concept-element-namespace) is <var>namespace</var> and <a id="ref-for-concept-element-local-name④"></a>[local name](#concept-element-local-name) is <var>localName</var>.

When invoked with the same arguments, the same <code><a id="ref-for-htmlcollection①⑥"></a>[HTMLCollection](#htmlcollection)</code> object may be returned as returned by an earlier call.

The <a id="concept-getelementsbyclassname"></a><strong>list of elements with class names <var>classNames</var></strong> for a <a id="ref-for-concept-node①④⑤"></a>[node](#concept-node) <var>root</var> is the <code><a id="ref-for-htmlcollection①⑦"></a>[HTMLCollection](#htmlcollection)</code> returned by the following algorithm:

1.  Let <var>classes</var> be the result of running the <a id="ref-for-concept-ordered-set-parser"></a>[ordered set parser](https://dom.spec.whatwg.org/#concept-ordered-set-parser) on <var>classNames</var>.

2.  If <var>classes</var> is the empty set, return an empty <code><a id="ref-for-htmlcollection①⑧"></a>[HTMLCollection](#htmlcollection)</code>.

3.  Return an <code><a id="ref-for-htmlcollection①⑨"></a>[HTMLCollection](#htmlcollection)</code> rooted at <var>root</var>, whose filter matches <a id="ref-for-concept-tree-descendant②②"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element⑤④"></a>[elements](#concept-element) that have all their <a id="ref-for-concept-class"></a>[classes](#concept-class) in <var>classes</var>.

    The comparisons for the <a id="ref-for-concept-class①"></a>[classes](#concept-class) must be done in an <a id="ref-for-ascii-case-insensitive"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) manner if <var>root</var>’s <a id="ref-for-concept-node-document③①"></a>[node document](#concept-node-document)’s <a id="ref-for-concept-document-mode①"></a>[mode](#concept-document-mode) is "<code>quirks</code>"; otherwise in an <a id="ref-for-string-is"></a>[identical to](https://infra.spec.whatwg.org/#string-is) manner.

When invoked with the same argument, the same <code><a id="ref-for-htmlcollection②⓪"></a>[HTMLCollection](#htmlcollection)</code> object may be returned as returned by an earlier call.

### <a id="interface-document"></a>4.5. Interface <code><a id="ref-for-document①⑦"></a>[Document](#document)</code>[](#interface-document)

<a id="ref-for-Exposed⑥"></a><a id="document"></a><a id="ref-for-node⑤④"></a><a id="ref-for-dom-document-document"></a><a id="ref-for-SameObject⑥"></a><a id="ref-for-domimplementation"></a><a id="ref-for-dom-document-implementation"></a><a id="ref-for-idl-USVString①"></a><a id="ref-for-dom-document-url"></a><a id="ref-for-idl-USVString②"></a><a id="ref-for-dom-document-documenturi"></a><a id="ref-for-idl-DOMString③⓪"></a><a id="ref-for-dom-document-compatmode"></a><a id="ref-for-idl-DOMString③①"></a><a id="ref-for-dom-document-characterset"></a><a id="ref-for-idl-DOMString③②"></a><a id="ref-for-dom-document-charset"></a><a id="ref-for-idl-DOMString③③"></a><a id="ref-for-dom-document-inputencoding"></a><a id="ref-for-idl-DOMString③④"></a><a id="ref-for-dom-document-contenttype"></a><a id="ref-for-documenttype①⑤"></a><a id="ref-for-dom-document-doctype"></a><a id="ref-for-element③⑨"></a><a id="ref-for-dom-document-documentelement"></a><a id="ref-for-htmlcollection②①"></a><a id="ref-for-dom-document-getelementsbytagname"></a><a id="ref-for-idl-DOMString③⑤"></a><a id="dom-document-getelementsbytagname-qualifiedname-qualifiedname"></a><a id="ref-for-htmlcollection②②"></a><a id="ref-for-dom-document-getelementsbytagnamens"></a><a id="ref-for-idl-DOMString③⑥"></a><a id="dom-document-getelementsbytagnamens-namespace-localname-namespace"></a><a id="ref-for-idl-DOMString③⑦"></a><a id="dom-document-getelementsbytagnamens-namespace-localname-localname"></a><a id="ref-for-htmlcollection②③"></a><a id="ref-for-dom-document-getelementsbyclassname"></a><a id="ref-for-idl-DOMString③⑧"></a><a id="dom-document-getelementsbyclassname-classnames-classnames"></a><a id="ref-for-cereactions①⑥"></a><a id="ref-for-NewObject⑤"></a><a id="ref-for-element④⓪"></a><a id="ref-for-dom-document-createelement"></a><a id="ref-for-idl-DOMString③⑨"></a><a id="dom-document-createelement-localname-options-localname"></a><a id="ref-for-idl-DOMString④⓪"></a><a id="ref-for-dictdef-elementcreationoptions"></a><a id="dom-document-createelement-localname-options-options"></a><a id="ref-for-cereactions①⑦"></a><a id="ref-for-NewObject⑥"></a><a id="ref-for-element④①"></a><a id="ref-for-dom-document-createelementns"></a><a id="ref-for-idl-DOMString④①"></a><a id="dom-document-createelementns-namespace-qualifiedname-options-namespace"></a><a id="ref-for-idl-DOMString④②"></a><a id="dom-document-createelementns-namespace-qualifiedname-options-qualifiedname"></a><a id="ref-for-idl-DOMString④③"></a><a id="ref-for-dictdef-elementcreationoptions①"></a><a id="dom-document-createelementns-namespace-qualifiedname-options-options"></a><a id="ref-for-NewObject⑦"></a><a id="ref-for-documentfragment②②"></a><a id="ref-for-dom-document-createdocumentfragment"></a><a id="ref-for-NewObject⑧"></a><a id="ref-for-text②⓪"></a><a id="ref-for-dom-document-createtextnode"></a><a id="ref-for-idl-DOMString④④"></a><a id="dom-document-createtextnode-data-data"></a><a id="ref-for-NewObject⑨"></a><a id="ref-for-cdatasection⑥"></a><a id="ref-for-dom-document-createcdatasection"></a><a id="ref-for-idl-DOMString④⑤"></a><a id="dom-document-createcdatasection-data-data"></a><a id="ref-for-NewObject①⓪"></a><a id="ref-for-comment①①"></a><a id="ref-for-dom-document-createcomment"></a><a id="ref-for-idl-DOMString④⑥"></a><a id="dom-document-createcomment-data-data"></a><a id="ref-for-NewObject①①"></a><a id="ref-for-processinginstruction①①"></a><a id="ref-for-dom-document-createprocessinginstruction"></a><a id="ref-for-idl-DOMString④⑦"></a><a id="dom-document-createprocessinginstruction-target-data-target"></a><a id="ref-for-idl-DOMString④⑧"></a><a id="dom-document-createprocessinginstruction-target-data-data"></a><a id="ref-for-cereactions①⑧"></a><a id="ref-for-NewObject①②"></a><a id="ref-for-node⑤⑤"></a><a id="ref-for-dom-document-importnode"></a><a id="ref-for-node⑤⑥"></a><a id="dom-document-importnode-node-options-node"></a><a id="ref-for-idl-boolean③⑤"></a><a id="ref-for-dictdef-importnodeoptions"></a><a id="dom-document-importnode-node-options-options"></a><a id="ref-for-cereactions①⑨"></a><a id="ref-for-node⑤⑦"></a><a id="ref-for-dom-document-adoptnode"></a><a id="ref-for-node⑤⑧"></a><a id="dom-document-adoptnode-node-node"></a><a id="ref-for-NewObject①③"></a><a id="ref-for-attr①⑦"></a><a id="ref-for-dom-document-createattribute"></a><a id="ref-for-idl-DOMString④⑨"></a><a id="dom-document-createattribute-localname-localname"></a><a id="ref-for-NewObject①④"></a><a id="ref-for-attr①⑧"></a><a id="ref-for-dom-document-createattributens"></a><a id="ref-for-idl-DOMString⑤⓪"></a><a id="dom-document-createattributens-namespace-qualifiedname-namespace"></a><a id="ref-for-idl-DOMString⑤①"></a><a id="dom-document-createattributens-namespace-qualifiedname-qualifiedname"></a><a id="ref-for-NewObject①⑤"></a><a id="ref-for-event①⑦"></a><a id="ref-for-dom-document-createevent"></a><a id="ref-for-idl-DOMString⑤②"></a><a id="dom-document-createevent-interface-interface"></a><a id="ref-for-NewObject①⑥"></a><a id="ref-for-range"></a><a id="ref-for-dom-document-createrange"></a><a id="ref-for-NewObject①⑦"></a><a id="ref-for-nodeiterator②"></a><a id="ref-for-dom-document-createnodeiterator"></a><a id="ref-for-node⑤⑨"></a><a id="dom-document-createnodeiterator-root-whattoshow-filter-root"></a><a id="ref-for-idl-unsigned-long⑤"></a><a id="dom-document-createnodeiterator-root-whattoshow-filter-whattoshow"></a><a id="ref-for-callbackdef-nodefilter"></a><a id="dom-document-createnodeiterator-root-whattoshow-filter-filter"></a><a id="ref-for-NewObject①⑧"></a><a id="ref-for-treewalker"></a><a id="ref-for-dom-document-createtreewalker"></a><a id="ref-for-node⑥⓪"></a><a id="dom-document-createtreewalker-root-whattoshow-filter-root"></a><a id="ref-for-idl-unsigned-long⑥"></a><a id="dom-document-createtreewalker-root-whattoshow-filter-whattoshow"></a><a id="ref-for-callbackdef-nodefilter①"></a><a id="dom-document-createtreewalker-root-whattoshow-filter-filter"></a><a id="ref-for-Exposed⑦"></a><a id="xmldocument"></a><a id="ref-for-document①⑧"></a><a id="dictdef-elementcreationoptions"></a><a id="ref-for-customelementregistry④"></a><a id="dom-elementcreationoptions-customelementregistry"></a><a id="ref-for-idl-DOMString⑤③"></a><a id="dom-elementcreationoptions-is"></a><a id="dictdef-importnodeoptions"></a><a id="ref-for-customelementregistry⑤"></a><a id="dom-importnodeoptions-customelementregistry"></a><a id="ref-for-idl-boolean③⑥"></a><a id="dom-importnodeoptions-selfonly"></a>

``` text
[Exposed=Window]
interface Document : Node {
  constructor();

  [SameObject] readonly attribute DOMImplementation implementation;
  readonly attribute USVString URL;
  readonly attribute USVString documentURI;
  readonly attribute DOMString compatMode;
  readonly attribute DOMString characterSet;
  readonly attribute DOMString charset; // legacy alias of .characterSet
  readonly attribute DOMString inputEncoding; // legacy alias of .characterSet
  readonly attribute DOMString contentType;

  readonly attribute DocumentType? doctype;
  readonly attribute Element? documentElement;
  HTMLCollection getElementsByTagName(DOMString qualifiedName);
  HTMLCollection getElementsByTagNameNS(DOMString? namespace, DOMString localName);
  HTMLCollection getElementsByClassName(DOMString classNames);

  [CEReactions, NewObject] Element createElement(DOMString localName, optional (DOMString or ElementCreationOptions) options = {});
  [CEReactions, NewObject] Element createElementNS(DOMString? namespace, DOMString qualifiedName, optional (DOMString or ElementCreationOptions) options = {});
  [NewObject] DocumentFragment createDocumentFragment();
  [NewObject] Text createTextNode(DOMString data);
  [NewObject] CDATASection createCDATASection(DOMString data);
  [NewObject] Comment createComment(DOMString data);
  [NewObject] ProcessingInstruction createProcessingInstruction(DOMString target, DOMString data);

  [CEReactions, NewObject] Node importNode(Node node, optional (boolean or ImportNodeOptions) options = false);
  [CEReactions] Node adoptNode(Node node);

  [NewObject] Attr createAttribute(DOMString localName);
  [NewObject] Attr createAttributeNS(DOMString? namespace, DOMString qualifiedName);

  [NewObject] Event createEvent(DOMString interface); // legacy

  [NewObject] Range createRange();

  // NodeFilter.SHOW_ALL = 0xFFFFFFFF
  [NewObject] NodeIterator createNodeIterator(Node root, optional unsigned long whatToShow = 0xFFFFFFFF, optional NodeFilter? filter = null);
  [NewObject] TreeWalker createTreeWalker(Node root, optional unsigned long whatToShow = 0xFFFFFFFF, optional NodeFilter? filter = null);
};

[Exposed=Window]
interface XMLDocument : Document {};

dictionary ElementCreationOptions {
  CustomElementRegistry? customElementRegistry;
  DOMString is;
};

dictionary ImportNodeOptions {
  CustomElementRegistry customElementRegistry;
  boolean selfOnly = false;
};
```

<code><a id="ref-for-document①⑨"></a>[Document](#document)</code> <a id="ref-for-concept-node①④⑥"></a>[nodes](#concept-node) are simply known as <a id="concept-document"></a><strong>documents</strong>.

A <a id="ref-for-concept-document②⑥"></a>[document](#concept-document)’s <a id="ref-for-concept-node-document③②"></a>[node document](#concept-node-document) is itself.

Each <a id="ref-for-concept-document②⑦"></a>[document](#concept-document) has an associated <a id="concept-document-encoding"></a><strong>encoding</strong> (an <a id="ref-for-encoding"></a>[encoding](https://encoding.spec.whatwg.org/#encoding)), <a id="concept-document-content-type"></a><strong>content type</strong> (a string), <a id="concept-document-url"></a><strong>URL</strong> (a <a id="ref-for-concept-url"></a>[URL](https://url.spec.whatwg.org/#concept-url)), <a id="concept-document-origin"></a><strong>origin</strong> (an <a id="ref-for-concept-origin"></a>[origin](https://html.spec.whatwg.org/multipage/browsers.html#concept-origin)), <a id="concept-document-type"></a><strong>type</strong> ("<code>xml</code>" or "<code>html</code>"), <a id="concept-document-mode"></a><strong>mode</strong> ("<code>no-quirks</code>", "<code>quirks</code>", or "<code>limited-quirks</code>"), <a id="document-allow-declarative-shadow-roots"></a><strong>allow declarative shadow roots</strong> (a boolean), and <a id="document-custom-element-registry"></a><strong>custom element registry</strong> (null or a <code><a id="ref-for-customelementregistry⑥"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code> object). [\[ENCODING\]](https://dom.spec.whatwg.org/#biblio-encoding) [\[URL\]](https://dom.spec.whatwg.org/#biblio-url) [\[HTML\]](https://dom.spec.whatwg.org/#biblio-html)

Unless stated otherwise, a <a id="ref-for-concept-document②⑧"></a>[document](#concept-document)’s <a id="ref-for-concept-document-encoding①"></a>[encoding](#concept-document-encoding) is the <a id="ref-for-utf-8"></a>[utf-8](https://encoding.spec.whatwg.org/#utf-8) <a id="ref-for-encoding①"></a>[encoding](https://encoding.spec.whatwg.org/#encoding), <a id="ref-for-concept-document-content-type①"></a>[content type](#concept-document-content-type) is "<code>application/xml</code>", <a id="ref-for-concept-document-url①"></a>[URL](#concept-document-url) is "<code>about&#58;blank</code>", <a id="ref-for-concept-document-origin①"></a>[origin](#concept-document-origin) is an <a id="ref-for-concept-origin-opaque"></a>[opaque origin](https://html.spec.whatwg.org/multipage/browsers.html#concept-origin-opaque), <a id="ref-for-concept-document-type②"></a>[type](#concept-document-type) is "<code>xml</code>", <a id="ref-for-concept-document-mode②"></a>[mode](#concept-document-mode) is "<code>no-quirks</code>", <a id="ref-for-document-allow-declarative-shadow-roots①"></a>[allow declarative shadow roots](#document-allow-declarative-shadow-roots) is false, and <a id="ref-for-document-custom-element-registry⑥"></a>[custom element registry](#document-custom-element-registry) is null.

A <a id="ref-for-concept-document②⑨"></a>[document](#concept-document) is said to be an <a id="xml-document"></a><strong>XML document</strong> if its <a id="ref-for-concept-document-type③"></a>[type](#concept-document-type) is "<code>xml</code>"; otherwise an <a id="html-document"></a><strong>HTML document</strong>. Whether a <a id="ref-for-concept-document③⓪"></a>[document](#concept-document) is an <a id="ref-for-html-document①"></a>[HTML document](#html-document) or an <a id="ref-for-xml-document"></a>[XML document](#xml-document) affects the behavior of certain APIs.

A <a id="ref-for-concept-document③①"></a>[document](#concept-document) is said to be in <a id="concept-document-no-quirks"></a><strong>no-quirks mode</strong> if its <a id="ref-for-concept-document-mode③"></a>[mode](#concept-document-mode) is "<code>no-quirks</code>", <a id="concept-document-quirks"></a><strong>quirks mode</strong> if its <a id="ref-for-concept-document-mode④"></a>[mode](#concept-document-mode) is "<code>quirks</code>", and <a id="concept-document-limited-quirks"></a><strong>limited-quirks mode</strong> if its <a id="ref-for-concept-document-mode⑤"></a>[mode](#concept-document-mode) is "<code>limited-quirks</code>".

<strong>Note:</strong>

The <a id="ref-for-concept-document-mode⑥"></a>[mode](#concept-document-mode) is only ever changed from the default for <a id="ref-for-concept-document③②"></a>[documents](#concept-document) created by the <a id="ref-for-html-parser①"></a>[HTML parser](https://html.spec.whatwg.org/multipage/parsing.html#html-parser) based on the presence, absence, or value of the DOCTYPE string, and by a new <a id="ref-for-browsing-context③"></a>[browsing context](https://html.spec.whatwg.org/multipage/document-sequences.html#browsing-context) (initial "<code>about&#58;blank</code>"). [\[HTML\]](https://dom.spec.whatwg.org/#biblio-html)

<a id="ref-for-concept-document-no-quirks"></a>[No-quirks mode](#concept-document-no-quirks) was originally known as "standards mode" and <a id="ref-for-concept-document-limited-quirks"></a>[limited-quirks mode](#concept-document-limited-quirks) was once known as "almost standards mode". They have been renamed because their details are now defined by standards. (And because Ian Hickson vetoed their original names on the basis that they are nonsensical.)

A <a id="ref-for-concept-document③③"></a>[document](#concept-document)’s <a id="ref-for-get-the-parent⑥"></a>[get the parent](https://dom.spec.whatwg.org/#get-the-parent) algorithm, given an <var>event</var>, returns null if <var>event</var>’s <code><a id="ref-for-dom-event-type①⑦"></a>[type](https://dom.spec.whatwg.org/#dom-event-type)</code> attribute value is "<code>load</code>" or <a id="ref-for-concept-document③④"></a>[document](#concept-document) does not have a <a id="ref-for-concept-document-bc"></a>[browsing context](https://html.spec.whatwg.org/multipage/document-sequences.html#concept-document-bc); otherwise the <a id="ref-for-concept-document③⑤"></a>[document](#concept-document)’s <a id="ref-for-concept-relevant-global⑤"></a>[relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global).

To <a id="create-a-document"></a><strong>create a document</strong> that implements an interface <var>interface</var>, given a <a id="ref-for-realm①"></a>[realm](https://tc39.es/ecma262/#realm) <var>realm</var>:

1.  <a id="ref-for-assert①⓪"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>interface</var> is <code><a id="ref-for-document②⓪"></a>[Document](#document)</code> or an interface that inherits from <code><a id="ref-for-document②①"></a>[Document](#document)</code>.

2.  Return a new <a id="ref-for-concept-node①④⑦"></a>[node](#concept-node) that <a id="ref-for-implements①⑥"></a>[implements](https://webidl.spec.whatwg.org/#implements) <var>interface</var>, in <var>realm</var>.

------------------------------------------------------------------------

<strong><code><var>document</var> = new <code><a id="ref-for-dom-document-document①"></a>[Document()](#dom-document-document)</code></code></strong>

Returns a new <a id="ref-for-concept-document③⑥"></a>[document](#concept-document).

<strong><code><var>document</var> . <code><a id="ref-for-dom-document-implementation①"></a>[implementation](#dom-document-implementation)</code></code></strong>

Returns <var>document</var>’s <code><a id="ref-for-domimplementation①"></a>[DOMImplementation](#domimplementation)</code> object.

<strong><code><var>document</var> . <code><a id="ref-for-dom-document-url①"></a>[URL](#dom-document-url)</code></code></strong>

<strong><code><var>document</var> . <code><a id="ref-for-dom-document-documenturi①"></a>[documentURI](#dom-document-documenturi)</code></code></strong>

Returns <var>document</var>’s <a id="ref-for-concept-document-url②"></a>[URL](#concept-document-url).

<strong><code><var>document</var> . <code><a id="ref-for-dom-document-compatmode①"></a>[compatMode](#dom-document-compatmode)</code></code></strong>

Returns the string "<code>BackCompat</code>" if <var>document</var>’s <a id="ref-for-concept-document-mode⑦"></a>[mode](#concept-document-mode) is "<code>quirks</code>"; otherwise "<code>CSS1Compat</code>".

<strong><code><var>document</var> . <code><a id="ref-for-dom-document-characterset①"></a>[characterSet](#dom-document-characterset)</code></code></strong>

Returns <var>document</var>’s <a id="ref-for-concept-document-encoding②"></a>[encoding](#concept-document-encoding).

<strong><code><var>document</var> . <code><a id="ref-for-dom-document-contenttype①"></a>[contentType](#dom-document-contenttype)</code></code></strong>

Returns <var>document</var>’s <a id="ref-for-concept-document-content-type②"></a>[content type](#concept-document-content-type).

The <a id="dom-document-document"></a><strong><code>new Document()</code></strong> constructor steps are to set <a id="ref-for-this①③⓪"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-document-origin②"></a>[origin](#concept-document-origin) to the <a id="ref-for-concept-document-origin③"></a>[origin](#concept-document-origin) of <a id="ref-for-current-global-object"></a>[current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object)’s <a id="ref-for-concept-document-window②"></a>[associated <code>Document</code>](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window). [\[HTML\]](https://dom.spec.whatwg.org/#biblio-html)

<strong>Note:</strong>

Unlike <code><a id="ref-for-dom-domimplementation-createdocument"></a>[createDocument()](#dom-domimplementation-createdocument)</code>, this constructor does not return an <code><a id="ref-for-xmldocument"></a>[XMLDocument](#xmldocument)</code> object, but a <a id="ref-for-concept-document③⑦"></a>[document](#concept-document) (<code><a id="ref-for-document②②"></a>[Document](#document)</code> object).

The <a id="dom-document-implementation"></a><strong><code>implementation</code></strong> getter steps are to return the <code><a id="ref-for-domimplementation②"></a>[DOMImplementation](#domimplementation)</code> object that is associated with <a id="ref-for-this①③①"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-document-url"></a><strong><code>URL</code></strong> and <a id="dom-document-documenturi"></a><strong><code>documentURI</code></strong> getter steps are to return <a id="ref-for-this①③②"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-document-url③"></a>[URL](#concept-document-url), <a id="ref-for-concept-url-serializer①"></a>[serialized](https://url.spec.whatwg.org/#concept-url-serializer).

The <a id="dom-document-compatmode"></a><strong><code>compatMode</code></strong> getter steps are to return "<code>BackCompat</code>" if <a id="ref-for-this①③③"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-document-mode⑧"></a>[mode](#concept-document-mode) is "<code>quirks</code>"; otherwise "<code>CSS1Compat</code>".

The <a id="dom-document-characterset"></a><strong><code>characterSet</code></strong>, <a id="dom-document-charset"></a><strong><code>charset</code></strong>, and <a id="dom-document-inputencoding"></a><strong><code>inputEncoding</code></strong> getter steps are to return <a id="ref-for-this①③④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-document-encoding③"></a>[encoding](#concept-document-encoding)’s <a id="ref-for-name"></a>[name](https://encoding.spec.whatwg.org/#name).

The <a id="dom-document-contenttype"></a><strong><code>contentType</code></strong> getter steps are to return <a id="ref-for-this①③⑤"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-document-content-type③"></a>[content type](#concept-document-content-type).

------------------------------------------------------------------------

<strong><var>document</var> . <code><a id="ref-for-dom-document-doctype①"></a>[doctype](#dom-document-doctype)</code></strong>

Returns the <a id="ref-for-concept-doctype⑨"></a>[doctype](#concept-doctype) or null if there is none.

<strong><var>document</var> . <code><a id="ref-for-dom-document-documentelement①"></a>[documentElement](#dom-document-documentelement)</code></strong>

Returns the <a id="ref-for-document-element⑤"></a>[document element](#document-element).

<strong><code><var>collection</var> = <var>document</var> . <a id="ref-for-dom-document-getelementsbytagname①"></a>[getElementsByTagName](#dom-document-getelementsbytagname)(<var>qualifiedName</var>)</code></strong>

If <var>qualifiedName</var> is "<code>&#42;</code>" returns an <code><a id="ref-for-htmlcollection②④"></a>[HTMLCollection](#htmlcollection)</code> of all <a id="ref-for-concept-tree-descendant②③"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element⑤⑤"></a>[elements](#concept-element).

Otherwise, returns an <code><a id="ref-for-htmlcollection②⑤"></a>[HTMLCollection](#htmlcollection)</code> of all <a id="ref-for-concept-tree-descendant②④"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element⑤⑥"></a>[elements](#concept-element) whose <a id="ref-for-concept-element-qualified-name③"></a>[qualified name](#concept-element-qualified-name) is <var>qualifiedName</var>. (Matches case-insensitively against <a id="ref-for-concept-element⑤⑦"></a>[elements](#concept-element) in the <a id="ref-for-html-namespace④"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) within an <a id="ref-for-html-document②"></a>[HTML document](#html-document).)

<strong><code><var>collection</var> = <var>document</var> . <a id="ref-for-dom-document-getelementsbytagnamens①"></a>[getElementsByTagNameNS](#dom-document-getelementsbytagnamens)(<var>namespace</var>, <var>localName</var>)</code></strong>

If <var>namespace</var> and <var>localName</var> are "<code>&#42;</code>", returns an <code><a id="ref-for-htmlcollection②⑥"></a>[HTMLCollection](#htmlcollection)</code> of all <a id="ref-for-concept-tree-descendant②⑤"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element⑤⑧"></a>[elements](#concept-element).

If only <var>namespace</var> is "<code>&#42;</code>", returns an <code><a id="ref-for-htmlcollection②⑦"></a>[HTMLCollection](#htmlcollection)</code> of all <a id="ref-for-concept-tree-descendant②⑥"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element⑤⑨"></a>[elements](#concept-element) whose <a id="ref-for-concept-element-local-name⑤"></a>[local name](#concept-element-local-name) is <var>localName</var>.

If only <var>localName</var> is "<code>&#42;</code>", returns an <code><a id="ref-for-htmlcollection②⑧"></a>[HTMLCollection](#htmlcollection)</code> of all <a id="ref-for-concept-tree-descendant②⑦"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element⑥⓪"></a>[elements](#concept-element) whose <a id="ref-for-concept-element-namespace⑨"></a>[namespace](#concept-element-namespace) is <var>namespace</var>.

Otherwise, returns an <code><a id="ref-for-htmlcollection②⑨"></a>[HTMLCollection](#htmlcollection)</code> of all <a id="ref-for-concept-tree-descendant②⑧"></a>[descendant](#concept-tree-descendant) <a id="ref-for-concept-element⑥①"></a>[elements](#concept-element) whose <a id="ref-for-concept-element-namespace①⓪"></a>[namespace](#concept-element-namespace) is <var>namespace</var> and <a id="ref-for-concept-element-local-name⑥"></a>[local name](#concept-element-local-name) is <var>localName</var>.

<strong><code><var>collection</var> = <var>document</var> . <a id="ref-for-dom-document-getelementsbyclassname①"></a>[getElementsByClassName](#dom-document-getelementsbyclassname)(<var>classNames</var>)</code></strong>

<strong><code><var>collection</var> = <var>element</var> . <a id="ref-for-dom-element-getelementsbyclassname"></a>[getElementsByClassName](#dom-element-getelementsbyclassname)(<var>classNames</var>)</code></strong>

Returns an <code><a id="ref-for-htmlcollection③⓪"></a>[HTMLCollection](#htmlcollection)</code> of the <a id="ref-for-concept-element⑥②"></a>[elements](#concept-element) in the object on which the method was invoked (a <a id="ref-for-concept-document③⑧"></a>[document](#concept-document) or an <a id="ref-for-concept-element⑥③"></a>[element](#concept-element)) that have all the classes given by <var>classNames</var>. The <var>classNames</var> argument is interpreted as a space-separated list of classes.

The <a id="dom-document-doctype"></a><strong><code>doctype</code></strong> getter steps are to return the <a id="ref-for-concept-tree-child⑤④"></a>[child](#concept-tree-child) of <a id="ref-for-this①③⑥"></a>[this](https://webidl.spec.whatwg.org/#this) that is a <a id="ref-for-concept-doctype①⓪"></a>[doctype](#concept-doctype); otherwise null.

The <a id="dom-document-documentelement"></a><strong><code>documentElement</code></strong> getter steps are to return <a id="ref-for-this①③⑦"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-document-element⑥"></a>[document element](#document-element).

The <a id="dom-document-getelementsbytagname"></a><strong><code>getElementsByTagName(<var>qualifiedName</var>)</code></strong> method steps are to return the <a id="ref-for-concept-getelementsbytagname"></a>[list of elements with qualified name <var>qualifiedName</var>](#concept-getelementsbytagname) for <a id="ref-for-this①③⑧"></a>[this](https://webidl.spec.whatwg.org/#this).

<strong>Note:</strong>

Thus, in an <a id="ref-for-html-document③"></a>[HTML document](#html-document), <code>document.getElementsByTagName(&#34;FOO&#34;)</code> will match <code>&#60;FOO&#62;</code> elements that are not in the <a id="ref-for-html-namespace⑤"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace), and <code>&#60;foo&#62;</code> elements that are in the <a id="ref-for-html-namespace⑥"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace), but not <code>&#60;FOO&#62;</code> elements that are in the <a id="ref-for-html-namespace⑦"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace).

The <a id="dom-document-getelementsbytagnamens"></a><strong><code>getElementsByTagNameNS(<var>namespace</var>, <var>localName</var>)</code></strong> method steps are to return the <a id="ref-for-concept-getelementsbytagnamens"></a>[list of elements with namespace <var>namespace</var> and local name <var>localName</var>](#concept-getelementsbytagnamens) for <a id="ref-for-this①③⑨"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-document-getelementsbyclassname"></a><strong><code>getElementsByClassName(<var>classNames</var>)</code></strong> method steps are to return the <a id="ref-for-concept-getelementsbyclassname"></a>[list of elements with class names <var>classNames</var>](#concept-getelementsbyclassname) for <a id="ref-for-this①④⓪"></a>[this](https://webidl.spec.whatwg.org/#this).

<a id="example-5ffcda00"></a>

<strong>Example:</strong>

[](#example-5ffcda00) Given the following XHTML fragment:

``` text
<div id="example">
  <p id="p1" class="aaa bbb"/>
  <p id="p2" class="aaa ccc"/>
  <p id="p3" class="bbb ccc"/>
</div>
```

A call to <code>document.getElementById(&#34;example&#34;).getElementsByClassName(&#34;aaa&#34;)</code> would return an <code><a id="ref-for-htmlcollection③①"></a>[HTMLCollection](#htmlcollection)</code> with the two paragraphs <code>p1</code> and <code>p2</code> in it.

A call to <code>getElementsByClassName(&#34;ccc bbb&#34;)</code> would only return one node, however, namely <code>p3</code>. A call to <code>document.getElementById(&#34;example&#34;).getElementsByClassName(&#34;bbb  ccc &#34;)</code> would return the same thing.

A call to <code>getElementsByClassName(&#34;aaa,bbb&#34;)</code> would return no nodes; none of the elements above are in the <code>aaa,bbb</code> class.

------------------------------------------------------------------------

<strong><code><var>element</var> = <var>document</var> . <a id="ref-for-dom-document-createelement①"></a>[createElement(localName \[, options\])](#dom-document-createelement)</code></strong>

Returns an <a id="ref-for-concept-element⑥④"></a>[element](#concept-element) with <var>localName</var> as <a id="ref-for-concept-element-local-name⑦"></a>[local name](#concept-element-local-name) (if <var>document</var> is an <a id="ref-for-html-document④"></a>[HTML document](#html-document), <var>localName</var> gets lowercased). The <a id="ref-for-concept-element⑥⑤"></a>[element](#concept-element)’s <a id="ref-for-concept-element-namespace①①"></a>[namespace](#concept-element-namespace) is the <a id="ref-for-html-namespace⑧"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) when <var>document</var> is an <a id="ref-for-html-document⑤"></a>[HTML document](#html-document) or <var>document</var>’s <a id="ref-for-concept-document-content-type④"></a>[content type](#concept-document-content-type) is "<code>application/xhtml+xml</code>"; otherwise null.

When supplied, <var>options</var>’s <code><a id="ref-for-dom-elementcreationoptions-customelementregistry"></a>[customElementRegistry](#dom-elementcreationoptions-customelementregistry)</code> can be used to set the <code><a id="ref-for-customelementregistry⑦"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code>.

When supplied, <var>options</var>’s <code><a id="ref-for-dom-elementcreationoptions-is"></a>[is](#dom-elementcreationoptions-is)</code> can be used to create a <a id="ref-for-customized-built-in-element"></a>[customized built-in element](https://html.spec.whatwg.org/multipage/custom-elements.html#customized-built-in-element).

If <var>localName</var> is not a <a id="ref-for-valid-element-local-name②"></a>[valid element local name](#valid-element-local-name) an "<code><a id="ref-for-invalidcharactererror③"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException③⑨"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> will be thrown.

When both <var>options</var>’s <code><a id="ref-for-dom-elementcreationoptions-customelementregistry①"></a>[customElementRegistry](#dom-elementcreationoptions-customelementregistry)</code> and <var>options</var>’s <code><a id="ref-for-dom-elementcreationoptions-is①"></a>[is](#dom-elementcreationoptions-is)</code> are supplied, a "<code><a id="ref-for-notsupportederror①"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException④⓪"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> will be thrown.

<strong><code><var>element</var> = <var>document</var> . <a id="ref-for-dom-document-createelementns①"></a>[createElementNS(namespace, qualifiedName \[, options\])](#dom-document-createelementns)</code></strong>

Returns an <a id="ref-for-concept-element⑥⑥"></a>[element](#concept-element) with <a id="ref-for-concept-element-namespace①②"></a>[namespace](#concept-element-namespace) <var>namespace</var>. Its <a id="ref-for-concept-element-namespace-prefix⑤"></a>[namespace prefix](#concept-element-namespace-prefix) will be everything before U+003A (:) in <var>qualifiedName</var> or null. Its <a id="ref-for-concept-element-local-name⑧"></a>[local name](#concept-element-local-name) will be everything after U+003A (:) in <var>qualifiedName</var> or <var>qualifiedName</var>.

When supplied, <var>options</var>’s <code><a id="ref-for-dom-elementcreationoptions-customelementregistry②"></a>[customElementRegistry](#dom-elementcreationoptions-customelementregistry)</code> can be used to set the <code><a id="ref-for-customelementregistry⑧"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code>.

When supplied, <var>options</var>’s <code><a id="ref-for-dom-elementcreationoptions-is②"></a>[is](#dom-elementcreationoptions-is)</code> can be used to create a <a id="ref-for-customized-built-in-element①"></a>[customized built-in element](https://html.spec.whatwg.org/multipage/custom-elements.html#customized-built-in-element).

If <var>qualifiedName</var> is not a (possibly-prefixed) <a id="ref-for-valid-element-local-name③"></a>[valid element local name](#valid-element-local-name) an "<code><a id="ref-for-invalidcharactererror④"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException④①"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> will be thrown.

If one of the following conditions is true a "<code><a id="ref-for-namespaceerror④"></a>[NamespaceError](https://webidl.spec.whatwg.org/#namespaceerror)</code>" <code><a id="ref-for-idl-DOMException④②"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> will be thrown:

- <a id="ref-for-concept-element-namespace-prefix⑥"></a>[Namespace prefix](#concept-element-namespace-prefix) is non-null and <var>namespace</var> is the empty string.
- <a id="ref-for-concept-element-namespace-prefix⑦"></a>[Namespace prefix](#concept-element-namespace-prefix) is "<code>xml</code>" and <var>namespace</var> is not the <a id="ref-for-xml-namespace②"></a>[XML namespace](https://infra.spec.whatwg.org/#xml-namespace).
- <var>qualifiedName</var> or <a id="ref-for-concept-element-namespace-prefix⑧"></a>[namespace prefix](#concept-element-namespace-prefix) is "<code>xmlns</code>" and <var>namespace</var> is not the <a id="ref-for-xmlns-namespace⑤"></a>[XMLNS namespace](https://infra.spec.whatwg.org/#xmlns-namespace).
- <var>namespace</var> is the <a id="ref-for-xmlns-namespace⑥"></a>[XMLNS namespace](https://infra.spec.whatwg.org/#xmlns-namespace) and neither <var>qualifiedName</var> nor <a id="ref-for-concept-element-namespace-prefix⑨"></a>[namespace prefix](#concept-element-namespace-prefix) is "<code>xmlns</code>".

When both <var>options</var>’s <code><a id="ref-for-dom-elementcreationoptions-customelementregistry③"></a>[customElementRegistry](#dom-elementcreationoptions-customelementregistry)</code> and <var>options</var>’s <code><a id="ref-for-dom-elementcreationoptions-is③"></a>[is](#dom-elementcreationoptions-is)</code> are supplied, a "<code><a id="ref-for-notsupportederror②"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException④③"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> will be thrown.

<strong><code><var>documentFragment</var> = <var>document</var> . <code><a id="ref-for-dom-document-createdocumentfragment①"></a>[createDocumentFragment()](#dom-document-createdocumentfragment)</code></code></strong>

Returns a <code><a id="ref-for-documentfragment②③"></a>[DocumentFragment](#documentfragment)</code> <a id="ref-for-concept-node①④⑧"></a>[node](#concept-node).

<strong><code><var>text</var> = <var>document</var> . <code><a id="ref-for-dom-document-createtextnode①"></a>[createTextNode(data)](#dom-document-createtextnode)</code></code></strong>

Returns a <code><a id="ref-for-text②①"></a>[Text](#text)</code> <a id="ref-for-concept-node①④⑨"></a>[node](#concept-node) whose <a id="ref-for-concept-cd-data①②"></a>[data](#concept-cd-data) is <var>data</var>.

<strong><code><var>text</var> = <var>document</var> . <code><a id="ref-for-dom-document-createcdatasection①"></a>[createCDATASection(data)](#dom-document-createcdatasection)</code></code></strong>

Returns a <code><a id="ref-for-cdatasection⑦"></a>[CDATASection](#cdatasection)</code> <a id="ref-for-concept-node①⑤⓪"></a>[node](#concept-node) whose <a id="ref-for-concept-cd-data①③"></a>[data](#concept-cd-data) is <var>data</var>.

<strong><code><var>comment</var> = <var>document</var> . <code><a id="ref-for-dom-document-createcomment①"></a>[createComment(data)](#dom-document-createcomment)</code></code></strong>

Returns a <code><a id="ref-for-comment①②"></a>[Comment](#comment)</code> <a id="ref-for-concept-node①⑤①"></a>[node](#concept-node) whose <a id="ref-for-concept-cd-data①④"></a>[data](#concept-cd-data) is <var>data</var>.

<strong><code><var>processingInstruction</var> = <var>document</var> . <code><a id="ref-for-dom-document-createprocessinginstruction①"></a>[createProcessingInstruction(target, data)](#dom-document-createprocessinginstruction)</code></code></strong>

Returns a <code><a id="ref-for-processinginstruction①②"></a>[ProcessingInstruction](#processinginstruction)</code> <a id="ref-for-concept-node①⑤②"></a>[node](#concept-node) whose <a id="ref-for-concept-pi-target④"></a>[target](#concept-pi-target) is <var>target</var> and <a id="ref-for-concept-cd-data①⑤"></a>[data](#concept-cd-data) is <var>data</var>. If <var>target</var> does not match the <code><a id="ref-for-NT-Name"></a>[Name](https://www.w3.org/TR/xml/#NT-Name)</code> production an "<code><a id="ref-for-invalidcharactererror⑤"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException④④"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> will be thrown. If <var>data</var> contains "<code>?&#62;</code>" an "<code><a id="ref-for-invalidcharactererror⑥"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException④⑤"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> will be thrown.

The <a id="concept-element-interface"></a><strong>element interface</strong> for any <var>name</var> and <var>namespace</var> is <code><a id="ref-for-element④②"></a>[Element](#element)</code>, unless stated otherwise.

<strong>Note:</strong>

The HTML Standard will, e.g., define that for <code>html</code> and the <a id="ref-for-html-namespace⑨"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace), the <code><a id="ref-for-htmlhtmlelement"></a>[HTMLHtmlElement](https://html.spec.whatwg.org/multipage/semantics.html#htmlhtmlelement)</code> interface is used. [\[HTML\]](https://dom.spec.whatwg.org/#biblio-html)

The <a id="dom-document-createelement"></a><strong><code>createElement(<var>localName</var>, <var>options</var>)</code></strong> method steps are:

1.  If <var>localName</var> is not a <a id="ref-for-valid-element-local-name④"></a>[valid element local name](#valid-element-local-name), then <a id="ref-for-dfn-throw③⑦"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-invalidcharactererror⑦"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException④⑥"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  If <a id="ref-for-this①④①"></a>[this](https://webidl.spec.whatwg.org/#this) is an <a id="ref-for-html-document⑥"></a>[HTML document](#html-document), then set <var>localName</var> to <var>localName</var> in <a id="ref-for-ascii-lowercase①"></a>[ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).

3.  Let <var>registry</var> and <var>is</var> be the result of <a id="ref-for-flatten-element-creation-options"></a>[flattening element creation options](#flatten-element-creation-options) given <var>options</var> and <a id="ref-for-this①④②"></a>[this](https://webidl.spec.whatwg.org/#this).

4.  Let <var>namespace</var> be the <a id="ref-for-html-namespace①⓪"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace), if <a id="ref-for-this①④③"></a>[this](https://webidl.spec.whatwg.org/#this) is an <a id="ref-for-html-document⑦"></a>[HTML document](#html-document) or <a id="ref-for-this①④④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-document-content-type⑤"></a>[content type](#concept-document-content-type) is "<code>application/xhtml+xml</code>"; otherwise null.

5.  Return the result of <a id="ref-for-concept-create-element①"></a>[creating an element](#concept-create-element) given <a id="ref-for-this①④⑤"></a>[this](https://webidl.spec.whatwg.org/#this), <var>localName</var>, <var>namespace</var>, null, <var>is</var>, true, and <var>registry</var>.

The <a id="internal-createelementns-steps"></a><strong>internal <code>createElementNS</code> steps</strong>, given <var>document</var>, <var>namespace</var>, <var>qualifiedName</var>, and <var>options</var>, are as follows:

1.  Let (<var>namespace</var>, <var>prefix</var>, <var>localName</var>) be the result of <a id="ref-for-validate-and-extract"></a>[validating and extracting](#validate-and-extract) <var>namespace</var> and <var>qualifiedName</var> given "<code>element</code>".

2.  Let <var>registry</var> and <var>is</var> be the result of <a id="ref-for-flatten-element-creation-options①"></a>[flattening element creation options](#flatten-element-creation-options) given <var>options</var> and <a id="ref-for-this①④⑥"></a>[this](https://webidl.spec.whatwg.org/#this).

3.  Return the result of <a id="ref-for-concept-create-element②"></a>[creating an element](#concept-create-element) given <var>document</var>, <var>localName</var>, <var>namespace</var>, <var>prefix</var>, <var>is</var>, true, and <var>registry</var>.

The <a id="dom-document-createelementns"></a><strong><code>createElementNS(<var>namespace</var>, <var>qualifiedName</var>, <var>options</var>)</code></strong> method steps are to return the result of running the <a id="ref-for-internal-createelementns-steps"></a>[internal <code>createElementNS</code> steps](#internal-createelementns-steps), given <a id="ref-for-this①④⑦"></a>[this](https://webidl.spec.whatwg.org/#this), <var>namespace</var>, <var>qualifiedName</var>, and <var>options</var>.

To <a id="flatten-element-creation-options"></a><strong>flatten element creation options</strong>, given a string or <code><a id="ref-for-dictdef-elementcreationoptions②"></a>[ElementCreationOptions](#dictdef-elementcreationoptions)</code> dictionary <var>options</var> and a <a id="ref-for-concept-document③⑨"></a>[document](#concept-document) <var>document</var>:

1.  Let <var>registry</var> be the result of <a id="ref-for-look-up-a-custom-element-registry"></a>[looking up a custom element registry](https://html.spec.whatwg.org/multipage/custom-elements.html#look-up-a-custom-element-registry) given <var>document</var>.

2.  Let <var>is</var> be null.

3.  If <var>options</var> is a dictionary:

    1.  If <var>options</var>\["<code><a id="ref-for-dom-elementcreationoptions-is④"></a>[is](#dom-elementcreationoptions-is)</code>"\] <a id="ref-for-map-exists①②"></a>[exists](https://infra.spec.whatwg.org/#map-exists), then set <var>is</var> to it.

    2.  If <var>options</var>\["<code><a id="ref-for-dom-elementcreationoptions-customelementregistry④"></a>[customElementRegistry](#dom-elementcreationoptions-customelementregistry)</code>"\] <a id="ref-for-map-exists①③"></a>[exists](https://infra.spec.whatwg.org/#map-exists):

        1.  If <var>is</var> is non-null, then <a id="ref-for-dfn-throw③⑧"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror③"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException④⑦"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

        2.  Set <var>registry</var> to <var>options</var>\["<code><a id="ref-for-dom-elementcreationoptions-customelementregistry⑤"></a>[customElementRegistry](#dom-elementcreationoptions-customelementregistry)</code>"\].

    3.  If <var>registry</var> is non-null, <var>registry</var>’s <a id="ref-for-is-scoped③"></a>[is scoped](https://html.spec.whatwg.org/multipage/custom-elements.html#is-scoped) is false, and <var>registry</var> is not <var>document</var>’s <a id="ref-for-document-custom-element-registry⑦"></a>[custom element registry](#document-custom-element-registry), then <a id="ref-for-dfn-throw③⑨"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror④"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException④⑧"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

4.  Return <var>registry</var> and <var>is</var>.

<strong>Note:</strong>

<code><a id="ref-for-dom-document-createelement②"></a>[createElement()](#dom-document-createelement)</code> and <code><a id="ref-for-dom-document-createelementns②"></a>[createElementNS()](#dom-document-createelementns)</code>’s <var>options</var> parameter is allowed to be a string for web compatibility.

The <a id="dom-document-createdocumentfragment"></a><strong><code>createDocumentFragment()</code></strong> method steps are to return the result of <a id="ref-for-create-a-document-fragment①"></a>[creating a document fragment](#create-a-document-fragment) given <a id="ref-for-this①④⑧"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-document-createtextnode"></a><strong><code>createTextNode(<var>data</var>)</code></strong> method steps are to return the result of <a id="ref-for-create-a-text-node②"></a>[creating a text node](#create-a-text-node) given <a id="ref-for-this①④⑨"></a>[this](https://webidl.spec.whatwg.org/#this) and <var>data</var>.

The <a id="dom-document-createcdatasection"></a><strong><code>createCDATASection(<var>data</var>)</code></strong> method steps are:

1.  If <a id="ref-for-this①⑤⓪"></a>[this](https://webidl.spec.whatwg.org/#this) is an <a id="ref-for-html-document⑧"></a>[HTML document](#html-document), then <a id="ref-for-dfn-throw④⓪"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror⑤"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException④⑨"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  If <var>data</var> contains the string "<code>&#93;&#93;&#62;</code>", then <a id="ref-for-dfn-throw④①"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-invalidcharactererror⑧"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException⑤⓪"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  Return the result of <a id="ref-for-create-a-cdata-section-node"></a>[creating a CDATA section node](#create-a-cdata-section-node) given <a id="ref-for-this①⑤①"></a>[this](https://webidl.spec.whatwg.org/#this) and <var>data</var>.

The <a id="dom-document-createcomment"></a><strong><code>createComment(<var>data</var>)</code></strong> method steps are to return the result of <a id="ref-for-create-a-comment-node"></a>[creating a comment node](#create-a-comment-node) given <a id="ref-for-this①⑤②"></a>[this](https://webidl.spec.whatwg.org/#this) and <var>data</var>.

The <a id="dom-document-createprocessinginstruction"></a><strong><code>createProcessingInstruction(<var>target</var>, <var>data</var>)</code></strong> method steps are to return the result of <a id="ref-for-create-a-processing-instruction-node"></a>[creating a processing instruction node](#create-a-processing-instruction-node) given <a id="ref-for-this①⑤③"></a>[this](https://webidl.spec.whatwg.org/#this), <var>target</var>, and <var>data</var>.

------------------------------------------------------------------------

<strong><code><var>clone</var> = <var>document</var> . <a id="ref-for-dom-document-importnode①"></a>[importNode](#dom-document-importnode)(<var>node</var> \[, <var>options</var> = false\])</code></strong>

Returns a copy of <var>node</var>. If <var>options</var> is true or <var>options</var> is a dictionary whose <code><a id="ref-for-dom-importnodeoptions-selfonly"></a>[selfOnly](#dom-importnodeoptions-selfonly)</code> is false, the copy also includes the <var>node</var>’s <a id="ref-for-concept-tree-descendant②⑨"></a>[descendants](#concept-tree-descendant).

<var>options</var>’s <code><a id="ref-for-dom-importnodeoptions-customelementregistry"></a>[customElementRegistry](#dom-importnodeoptions-customelementregistry)</code> can be used to set the <code><a id="ref-for-customelementregistry⑨"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code> of elements that have none.

If <var>node</var> is a <a id="ref-for-concept-document④⓪"></a>[document](#concept-document) or a <a id="ref-for-concept-shadow-root②①"></a>[shadow root](#concept-shadow-root), throws a "<code><a id="ref-for-notsupportederror⑥"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑤①"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

<strong><code><var>node</var> = <var>document</var> . <a id="ref-for-dom-document-adoptnode①"></a>[adoptNode](#dom-document-adoptnode)(<var>node</var>)</code></strong>

Moves <var>node</var> from another <a id="ref-for-concept-document④①"></a>[document](#concept-document) and returns it.

If <var>node</var> is a <a id="ref-for-concept-document④②"></a>[document](#concept-document), throws a "<code><a id="ref-for-notsupportederror⑦"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑤②"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> or, if <var>node</var> is a <a id="ref-for-concept-shadow-root②②"></a>[shadow root](#concept-shadow-root), throws a "<code><a id="ref-for-hierarchyrequesterror②⓪"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException⑤③"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

The <a id="dom-document-importnode"></a><strong><code>importNode(<var>node</var>, <var>options</var>)</code></strong> method steps are:

1.  If <var>node</var> is a <a id="ref-for-concept-document④③"></a>[document](#concept-document) or <a id="ref-for-concept-shadow-root②③"></a>[shadow root](#concept-shadow-root), then <a id="ref-for-dfn-throw④②"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror⑧"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑤④"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  Let <var>subtree</var> be false.

3.  Let <var>registry</var> be null.

4.  If <var>options</var> is a boolean, then set <var>subtree</var> to <var>options</var>.

5.  Otherwise:

    1.  Set <var>subtree</var> to the negation of <var>options</var>\["<code><a id="ref-for-dom-importnodeoptions-selfonly①"></a>[selfOnly](#dom-importnodeoptions-selfonly)</code>"\].

    2.  If <var>options</var>\["<code><a id="ref-for-dom-importnodeoptions-customelementregistry①"></a>[customElementRegistry](#dom-importnodeoptions-customelementregistry)</code>"\] <a id="ref-for-map-exists①④"></a>[exists](https://infra.spec.whatwg.org/#map-exists), then set <var>registry</var> to it.

    3.  If <var>registry</var>’s <a id="ref-for-is-scoped④"></a>[is scoped](https://html.spec.whatwg.org/multipage/custom-elements.html#is-scoped) is false and <var>registry</var> is not <a id="ref-for-this①⑤④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-document-custom-element-registry⑧"></a>[custom element registry](#document-custom-element-registry), then <a id="ref-for-dfn-throw④③"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror⑨"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑤⑤"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

6.  If <var>registry</var> is null, then set <var>registry</var> to the result of <a id="ref-for-look-up-a-custom-element-registry①"></a>[looking up a custom element registry](https://html.spec.whatwg.org/multipage/custom-elements.html#look-up-a-custom-element-registry) given <a id="ref-for-this①⑤⑤"></a>[this](https://webidl.spec.whatwg.org/#this).

7.  Return the result of <a id="ref-for-concept-node-clone④"></a>[cloning a node](#concept-node-clone) given <var>node</var> with <a id="ref-for-clone-a-node-document②"></a>[<em>document</em>](#clone-a-node-document) set to <a id="ref-for-this①⑤⑥"></a>[this](https://webidl.spec.whatwg.org/#this), <a id="ref-for-clone-a-node-subtree③"></a>[<em>subtree</em>](#clone-a-node-subtree) set to <var>subtree</var>, and <a id="ref-for-clone-a-node-fallbackregistry②"></a>[<em>fallbackRegistry</em>](#clone-a-node-fallbackregistry) set to <var>registry</var>.

<a id="ref-for-other-applicable-specifications⑧"></a>[Specifications](https://dom.spec.whatwg.org/#other-applicable-specifications) may define <a id="concept-node-adopt-ext"></a><strong>adopting steps</strong> for all or some <a id="ref-for-concept-node①⑤③"></a>[nodes](#concept-node). The algorithm is passed <var>node</var> and <var>oldDocument</var>, as indicated in the <a id="ref-for-concept-node-adopt③"></a>[adopt](#concept-node-adopt) algorithm.

To <a id="concept-node-adopt"></a><strong>adopt</strong> a <a id="ref-for-concept-node①⑤④"></a>[node](#concept-node) <var>node</var> into a <a id="ref-for-concept-document④④"></a>[document](#concept-document) <var>document</var>:

1.  Let <var>oldDocument</var> be <var>node</var>’s <a id="ref-for-concept-node-document③③"></a>[node document](#concept-node-document).

2.  If <var>node</var>’s <a id="ref-for-concept-tree-parent②⑦"></a>[parent](#concept-tree-parent) is non-null, then <a id="ref-for-concept-node-remove①⓪"></a>[remove](#concept-node-remove) <var>node</var>.

3.  If <var>document</var> is not <var>oldDocument</var>, then for each <var>inclusiveDescendant</var> of <var>node</var>’s <a id="ref-for-concept-shadow-including-inclusive-descendant③"></a>[shadow-including inclusive descendants](#concept-shadow-including-inclusive-descendant), in <a id="ref-for-concept-shadow-including-tree-order④"></a>[shadow-including tree order](#concept-shadow-including-tree-order):

    1.  Set <var>inclusiveDescendant</var>’s <a id="ref-for-concept-node-document③④"></a>[node document](#concept-node-document) to <var>document</var>.

    2.  If <var>inclusiveDescendant</var> is a <a id="ref-for-concept-shadow-root②④"></a>[shadow root](#concept-shadow-root) and if any of the following are true:

        - <var>inclusiveDescendant</var>’s <a id="ref-for-shadowroot-custom-element-registry⑤"></a>[custom element registry](#shadowroot-custom-element-registry) is null and <var>inclusiveDescendant</var>’s <a id="ref-for-shadowroot-keep-custom-element-registry-null②"></a>[keep custom element registry null](#shadowroot-keep-custom-element-registry-null) is false; or

        - <var>inclusiveDescendant</var>’s <a id="ref-for-shadowroot-custom-element-registry⑥"></a>[custom element registry](#shadowroot-custom-element-registry) <a id="ref-for-is-a-global-custom-element-registry②"></a>[is a global custom element registry](#is-a-global-custom-element-registry),

        then set <var>inclusiveDescendant</var>’s <a id="ref-for-shadowroot-custom-element-registry⑦"></a>[custom element registry](#shadowroot-custom-element-registry) to <var>document</var>’s <a id="ref-for-document-custom-element-registry⑨"></a>[custom element registry](#document-custom-element-registry)’s <a id="ref-for-effective-global-custom-element-registry②"></a>[effective global custom element registry](#effective-global-custom-element-registry).

    3.  Otherwise, if <var>inclusiveDescendant</var> is an <a id="ref-for-concept-element⑥⑦"></a>[element](#concept-element):

        1.  Set the <a id="ref-for-concept-node-document③⑤"></a>[node document](#concept-node-document) of each <a id="ref-for-concept-attribute②⓪"></a>[attribute](#concept-attribute) in <var>inclusiveDescendant</var>’s <a id="ref-for-concept-element-attribute⑤"></a>[attribute list](#concept-element-attribute) to <var>document</var>.

        2.  If <var>inclusiveDescendant</var>’s <a id="ref-for-element-custom-element-registry④"></a>[custom element registry](#element-custom-element-registry) is null or <var>inclusiveDescendant</var>’s <a id="ref-for-element-custom-element-registry⑤"></a>[custom element registry](#element-custom-element-registry)’s <a id="ref-for-is-scoped⑤"></a>[is scoped](https://html.spec.whatwg.org/multipage/custom-elements.html#is-scoped) is false:

            1.  Let <var>registry</var> be null.

            2.  If <var>inclusiveDescendant</var>’s <a id="ref-for-element-custom-element-registry⑥"></a>[custom element registry](#element-custom-element-registry) is non-null, <var>inclusiveDescendant</var>’s <a id="ref-for-concept-tree-parent②⑧"></a>[parent](#concept-tree-parent) is null, or <var>inclusiveDescendant</var>’s <a id="ref-for-concept-tree-parent②⑨"></a>[parent](#concept-tree-parent) is an <a id="ref-for-exclusive-documentfragment-node"></a>[exclusive <code>DocumentFragment</code> node](#exclusive-documentfragment-node), then set <var>registry</var> to <var>document</var>’s <a id="ref-for-document-custom-element-registry①⓪"></a>[custom element registry](#document-custom-element-registry).

            3.  Otherwise, set <var>registry</var> to the result of <a id="ref-for-look-up-a-custom-element-registry②"></a>[looking up a custom element registry](https://html.spec.whatwg.org/multipage/custom-elements.html#look-up-a-custom-element-registry) given <var>inclusiveDescendant</var>’s <a id="ref-for-concept-tree-parent③⓪"></a>[parent](#concept-tree-parent).

            4.  Set <var>inclusiveDescendant</var>’s <a id="ref-for-element-custom-element-registry⑦"></a>[custom element registry](#element-custom-element-registry) to <var>registry</var>’s <a id="ref-for-effective-global-custom-element-registry③"></a>[effective global custom element registry](#effective-global-custom-element-registry).

        3.  If <var>inclusiveDescendant</var> is <a id="ref-for-concept-element-custom⑤"></a>[custom](#concept-element-custom), then <a id="ref-for-enqueue-a-custom-element-callback-reaction④"></a>[enqueue a custom element callback reaction](https://html.spec.whatwg.org/multipage/custom-elements.html#enqueue-a-custom-element-callback-reaction) with <var>inclusiveDescendant</var>, callback name "<code>adoptedCallback</code>", and « <var>oldDocument</var>, <var>document</var> ».

    4.  Run the <a id="ref-for-concept-node-adopt-ext"></a>[adopting steps](#concept-node-adopt-ext) with <var>inclusiveDescendant</var> and <var>oldDocument</var>.

The <a id="dom-document-adoptnode"></a><strong><code>adoptNode(<var>node</var>)</code></strong> method steps are:

1.  If <var>node</var> is a <a id="ref-for-concept-document④⑤"></a>[document](#concept-document), then <a id="ref-for-dfn-throw④④"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror①⓪"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑤⑥"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  If <var>node</var> is a <a id="ref-for-concept-shadow-root②⑤"></a>[shadow root](#concept-shadow-root), then <a id="ref-for-dfn-throw④⑤"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-hierarchyrequesterror②①"></a>[HierarchyRequestError](https://webidl.spec.whatwg.org/#hierarchyrequesterror)</code>" <code><a id="ref-for-idl-DOMException⑤⑦"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  <a id="ref-for-concept-node-adopt④"></a>[Adopt](#concept-node-adopt) <var>node</var> into <a id="ref-for-this①⑤⑦"></a>[this](https://webidl.spec.whatwg.org/#this).

4.  Return <var>node</var>.

------------------------------------------------------------------------

Null or a <code><a id="ref-for-customelementregistry①⓪"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code> object <var>registry</var> <a id="is-a-global-custom-element-registry"></a><strong>is a global custom element registry</strong> if <var>registry</var> is non-null and <var>registry</var>’s <a id="ref-for-is-scoped⑥"></a>[is scoped](https://html.spec.whatwg.org/multipage/custom-elements.html#is-scoped) is false.

Null or a <code><a id="ref-for-customelementregistry①①"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code> object <var>registry</var>’s <a id="effective-global-custom-element-registry"></a><strong>effective global custom element registry</strong> is:

1.  If <var>registry</var> <a id="ref-for-is-a-global-custom-element-registry③"></a>[is a global custom element registry](#is-a-global-custom-element-registry), then return <var>registry</var>.

2.  Return null.

------------------------------------------------------------------------

The <a id="dom-document-createattribute"></a><strong><code>createAttribute(<var>localName</var>)</code></strong> method steps are:

1.  If <var>localName</var> is not a <a id="ref-for-valid-attribute-local-name①"></a>[valid attribute local name](#valid-attribute-local-name), then <a id="ref-for-dfn-throw④⑥"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-invalidcharactererror⑨"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException⑤⑧"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  If <a id="ref-for-this①⑤⑧"></a>[this](https://webidl.spec.whatwg.org/#this) is an <a id="ref-for-html-document⑨"></a>[HTML document](#html-document), then set <var>localName</var> to <var>localName</var> in <a id="ref-for-ascii-lowercase②"></a>[ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).

3.  Return the result of <a id="ref-for-create-an-attribute"></a>[creating an attribute](#create-an-attribute) given <a id="ref-for-this①⑤⑨"></a>[this](https://webidl.spec.whatwg.org/#this) and <var>localName</var>.

The <a id="dom-document-createattributens"></a><strong><code>createAttributeNS(<var>namespace</var>, <var>qualifiedName</var>)</code></strong> method steps are:

1.  Let (<var>namespace</var>, <var>prefix</var>, <var>localName</var>) be the result of <a id="ref-for-validate-and-extract①"></a>[validating and extracting](#validate-and-extract) <var>namespace</var> and <var>qualifiedName</var> given "<code>attribute</code>".

2.  Return the result of <a id="ref-for-create-an-attribute①"></a>[creating an attribute](#create-an-attribute) given <a id="ref-for-this①⑥⓪"></a>[this](https://webidl.spec.whatwg.org/#this), <var>localName</var>, <var>namespace</var>, and <var>prefix</var>.

------------------------------------------------------------------------

The <a id="dom-document-createevent"></a><strong><code>createEvent(<var>interface</var>)</code></strong> method steps are:

1.  Let <var>constructor</var> be null.

2.  If <var>interface</var> is an <a id="ref-for-ascii-case-insensitive①"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for any of the strings in the first column in the following table, then set <var>constructor</var> to the interface in the second column on the same row as the matching string:

    <table>       <thead>        <tr>         <th>String         </th><th>Interface         </th><td>Notes           </td></tr></thead><tbody>        <tr>         <td>"<code>beforeunloadevent</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://html.spec.whatwg.org/multipage/nav-history-apis.html#beforeunloadevent" id="ref-for-beforeunloadevent">BeforeUnloadEvent</a></code>         </td><td><a data-link-type="biblio" href="https://dom.spec.whatwg.org/#biblio-html" title="HTML Standard">[HTML]</a>             </td></tr><tr>         <td>"<code>compositionevent</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://w3c.github.io/uievents/#compositionevent" id="ref-for-compositionevent">CompositionEvent</a></code>         </td><td><a data-link-type="biblio" href="https://dom.spec.whatwg.org/#biblio-uievents" title="UI Events">[UIEVENTS]</a>             </td></tr><tr>         <td>"<code>customevent</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://dom.spec.whatwg.org/#customevent" id="ref-for-customevent③">CustomEvent</a></code>         </td><td>        </td></tr><tr>         <td>"<code>devicemotionevent</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://w3c.github.io/deviceorientation/spec-source-orientation.html#devicemotion" id="ref-for-devicemotion">DeviceMotionEvent</a></code>         </td><td rowspan="2"><a data-link-type="biblio" href="https://dom.spec.whatwg.org/#biblio-device-orientation" title="Device Orientation and Motion">[DEVICE-ORIENTATION]</a>             </td></tr><tr>         <td>"<code>deviceorientationevent</code>"         </td><td><code class="idl"><a data-link-type="idl" data-refhint-key="116e25eb" href="https://w3c.github.io/deviceorientation/spec-source-orientation.html#devicemotion" id="ref-for-devicemotion①">DeviceOrientationEvent</a></code>             </td></tr><tr>         <td>"<code>dragevent</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://html.spec.whatwg.org/multipage/dnd.html#dragevent" id="ref-for-dragevent">DragEvent</a></code>         </td><td><a data-link-type="biblio" href="https://dom.spec.whatwg.org/#biblio-html" title="HTML Standard">[HTML]</a>             </td></tr><tr>         <td>"<code>event</code>"         </td><td rowspan="2"><code class="idl"><a data-link-type="idl" href="https://dom.spec.whatwg.org/#event" id="ref-for-event①⑧">Event</a></code>         </td><td rowspan="2">        </td></tr><tr>         <td>"<code>events</code>"             </td></tr><tr>         <td>"<code>focusevent</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://w3c.github.io/uievents/#focusevent" id="ref-for-focusevent">FocusEvent</a></code>         </td><td><a data-link-type="biblio" href="https://dom.spec.whatwg.org/#biblio-uievents" title="UI Events">[UIEVENTS]</a>             </td></tr><tr>         <td>"<code>hashchangeevent</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://html.spec.whatwg.org/multipage/nav-history-apis.html#hashchangeevent" id="ref-for-hashchangeevent">HashChangeEvent</a></code>         </td><td><a data-link-type="biblio" href="https://dom.spec.whatwg.org/#biblio-html" title="HTML Standard">[HTML]</a>             </td></tr><tr>         <td>"<code>htmlevents</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://dom.spec.whatwg.org/#event" id="ref-for-event①⑨">Event</a></code>         </td><td>        </td></tr><tr>         <td>"<code>keyboardevent</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://w3c.github.io/uievents/#keyboardevent" id="ref-for-keyboardevent">KeyboardEvent</a></code>         </td><td><a data-link-type="biblio" href="https://dom.spec.whatwg.org/#biblio-uievents" title="UI Events">[UIEVENTS]</a>             </td></tr><tr>         <td>"<code>messageevent</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://html.spec.whatwg.org/multipage/comms.html#messageevent" id="ref-for-messageevent">MessageEvent</a></code>         </td><td><a data-link-type="biblio" href="https://dom.spec.whatwg.org/#biblio-html" title="HTML Standard">[HTML]</a>             </td></tr><tr>         <td>"<code>mouseevent</code>"         </td><td rowspan="2"><code class="idl"><a data-link-type="idl" href="https://w3c.github.io/pointerevents/#dom-mouseevent" id="ref-for-dom-mouseevent③">MouseEvent</a></code>         </td><td rowspan="2"><a data-link-type="biblio" href="https://dom.spec.whatwg.org/#biblio-uievents" title="UI Events">[UIEVENTS]</a>             </td></tr><tr>         <td>"<code>mouseevents</code>"             </td></tr><tr>         <td>"<code>storageevent</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://html.spec.whatwg.org/multipage/webstorage.html#storageevent" id="ref-for-storageevent">StorageEvent</a></code>         </td><td><a data-link-type="biblio" href="https://dom.spec.whatwg.org/#biblio-html" title="HTML Standard">[HTML]</a>             </td></tr><tr>         <td>"<code>svgevents</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://dom.spec.whatwg.org/#event" id="ref-for-event②⓪">Event</a></code>         </td><td>        </td></tr><tr>         <td>"<code>textevent</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://w3c.github.io/uievents/#textevent" id="ref-for-textevent">TextEvent</a></code>         </td><td><a data-link-type="biblio" href="https://dom.spec.whatwg.org/#biblio-uievents" title="UI Events">[UIEVENTS]</a>             </td></tr><tr>         <td>"<code>touchevent</code>"         </td><td><code class="idl"><a data-link-type="idl" href="https://w3c.github.io/touch-events/#idl-def-touchevent" id="ref-for-idl-def-touchevent③">TouchEvent</a></code>         </td><td><a data-link-type="biblio" href="https://dom.spec.whatwg.org/#biblio-touch-events" title="Touch Events">[TOUCH-EVENTS]</a>             </td></tr><tr>         <td>"<code>uievent</code>"         </td><td rowspan="2"><code class="idl"><a data-link-type="idl" href="https://w3c.github.io/uievents/#uievent" id="ref-for-uievent">UIEvent</a></code>         </td><td rowspan="2"><a data-link-type="biblio" href="https://dom.spec.whatwg.org/#biblio-uievents" title="UI Events">[UIEVENTS]</a>             </td></tr><tr>         <td>"<code>uievents</code>"         </td></tr></tbody></table>

3.  If <var>constructor</var> is null, then <a id="ref-for-dfn-throw④⑦"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror①①"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑤⑨"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

4.  If the interface indicated by <var>constructor</var> is not exposed on the <a id="ref-for-concept-relevant-global⑥"></a>[relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) of <a id="ref-for-this①⑥①"></a>[this](https://webidl.spec.whatwg.org/#this), then <a id="ref-for-dfn-throw④⑧"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror①②"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑥⓪"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

    <strong>Note:</strong>

    Typically user agents disable support for touch events in some configurations, in which case this clause would be triggered for the interface <code><a id="ref-for-idl-def-touchevent④"></a>[TouchEvent](https://w3c.github.io/touch-events/#idl-def-touchevent)</code>.

5.  Let <var>event</var> be the result of <a id="ref-for-concept-event-create④"></a>[creating an event](https://dom.spec.whatwg.org/#concept-event-create) given <var>constructor</var>.

6.  Initialize <var>event</var>’s <code><a id="ref-for-dom-event-type①⑧"></a>[type](https://dom.spec.whatwg.org/#dom-event-type)</code> attribute to the empty string.

7.  Initialize <var>event</var>’s <code><a id="ref-for-dom-event-timestamp③"></a>[timeStamp](https://dom.spec.whatwg.org/#dom-event-timestamp)</code> attribute to the result of calling <a id="ref-for-dfn-current-high-resolution-time"></a>[current high resolution time](https://w3c.github.io/hr-time/#dfn-current-high-resolution-time) with <a id="ref-for-this①⑥②"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-relevant-global⑦"></a>[relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global).

8.  Initialize <var>event</var>’s <code><a id="ref-for-dom-event-istrusted⑨"></a>[isTrusted](https://dom.spec.whatwg.org/#dom-event-istrusted)</code> attribute to false.

9.  Unset <var>event</var>’s <a id="ref-for-initialized-flag③"></a>[initialized flag](https://dom.spec.whatwg.org/#initialized-flag).

10. Return <var>event</var>.

<strong>Note:</strong>

<a id="ref-for-concept-event⑤⑧"></a>[Event](https://dom.spec.whatwg.org/#concept-event) constructors ought to be used instead.

------------------------------------------------------------------------

The <a id="dom-document-createrange"></a><strong><code>createRange()</code></strong> method steps are to return a new <a id="ref-for-concept-live-range⑧"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) with (<a id="ref-for-this①⑥③"></a>[this](https://webidl.spec.whatwg.org/#this), 0) as its <a id="ref-for-concept-range-start"></a>[start](https://dom.spec.whatwg.org/#concept-range-start) an <a id="ref-for-concept-range-end"></a>[end](https://dom.spec.whatwg.org/#concept-range-end).

<strong>Note:</strong>

The <code><a id="ref-for-dom-range-range"></a>[Range()](https://dom.spec.whatwg.org/#dom-range-range)</code> constructor can be used instead.

------------------------------------------------------------------------

The <a id="dom-document-createnodeiterator"></a><strong><code>createNodeIterator(<var>root</var>, <var>whatToShow</var>, <var>filter</var>)</code></strong> method steps are:

1.  Let <var>iterator</var> be a new <code><a id="ref-for-nodeiterator③"></a>[NodeIterator](https://dom.spec.whatwg.org/#nodeiterator)</code> object.

2.  Set <var>iterator</var>’s <a id="ref-for-concept-traversal-root②"></a>[root](https://dom.spec.whatwg.org/#concept-traversal-root) to <var>root</var>.

3.  Set <var>iterator</var>’s <a id="ref-for-nodeiterator-reference"></a>[reference](https://dom.spec.whatwg.org/#nodeiterator-reference) to (<var>root</var>, true).

4.  Set <var>iterator</var>’s <a id="ref-for-concept-traversal-whattoshow"></a>[whatToShow](https://dom.spec.whatwg.org/#concept-traversal-whattoshow) to <var>whatToShow</var>.

5.  Set <var>iterator</var>’s <a id="ref-for-concept-traversal-filter"></a>[filter](https://dom.spec.whatwg.org/#concept-traversal-filter) to <var>filter</var>.

6.  Return <var>iterator</var>.

The <a id="dom-document-createtreewalker"></a><strong><code>createTreeWalker(<var>root</var>, <var>whatToShow</var>, <var>filter</var>)</code></strong> method steps are:

1.  Let <var>walker</var> be a new <code><a id="ref-for-treewalker①"></a>[TreeWalker](https://dom.spec.whatwg.org/#treewalker)</code> object.

2.  Set <var>walker</var>’s <a id="ref-for-concept-traversal-root③"></a>[root](https://dom.spec.whatwg.org/#concept-traversal-root) and <var>walker</var>’s <a id="ref-for-treewalker-current"></a>[current](https://dom.spec.whatwg.org/#treewalker-current) to <var>root</var>.

3.  Set <var>walker</var>’s <a id="ref-for-concept-traversal-whattoshow①"></a>[whatToShow](https://dom.spec.whatwg.org/#concept-traversal-whattoshow) to <var>whatToShow</var>.

4.  Set <var>walker</var>’s <a id="ref-for-concept-traversal-filter①"></a>[filter](https://dom.spec.whatwg.org/#concept-traversal-filter) to <var>filter</var>.

5.  Return <var>walker</var>.

#### <a id="interface-domimplementation"></a>4.5.1. Interface <code><a id="ref-for-domimplementation③"></a>[DOMImplementation](#domimplementation)</code>[](#interface-domimplementation)

User agents must create a <code><a id="ref-for-domimplementation④"></a>[DOMImplementation](#domimplementation)</code> object whenever a <a id="ref-for-concept-document④⑥"></a>[document](#concept-document) is created and associate it with that <a id="ref-for-concept-document④⑦"></a>[document](#concept-document).

<a id="ref-for-Exposed⑧"></a><a id="domimplementation"></a><a id="ref-for-NewObject①⑨"></a><a id="ref-for-documenttype①⑥"></a><a id="ref-for-dom-domimplementation-createdocumenttype"></a><a id="ref-for-idl-DOMString⑤④"></a><a id="dom-domimplementation-createdocumenttype-name-publicid-systemid-name"></a><a id="ref-for-idl-DOMString⑤⑤"></a><a id="dom-domimplementation-createdocumenttype-name-publicid-systemid-publicid"></a><a id="ref-for-idl-DOMString⑤⑥"></a><a id="dom-domimplementation-createdocumenttype-name-publicid-systemid-systemid"></a><a id="ref-for-NewObject②⓪"></a><a id="ref-for-xmldocument①"></a><a id="ref-for-dom-domimplementation-createdocument①"></a><a id="ref-for-idl-DOMString⑤⑦"></a><a id="dom-domimplementation-createdocument-namespace-qualifiedname-doctype-namespace"></a><a id="ref-for-LegacyNullToEmptyString"></a><a id="ref-for-idl-DOMString⑤⑧"></a><a id="dom-domimplementation-createdocument-namespace-qualifiedname-doctype-qualifiedname"></a><a id="ref-for-documenttype①⑦"></a><a id="dom-domimplementation-createdocument-namespace-qualifiedname-doctype-doctype"></a><a id="ref-for-NewObject②①"></a><a id="ref-for-document②③"></a><a id="ref-for-dom-domimplementation-createhtmldocument"></a><a id="ref-for-idl-DOMString⑤⑨"></a><a id="dom-domimplementation-createhtmldocument-title-title"></a><a id="ref-for-idl-boolean③⑦"></a><a id="ref-for-dom-domimplementation-hasfeature"></a>

``` text
[Exposed=Window]
interface DOMImplementation {
  [NewObject] DocumentType createDocumentType(DOMString name, DOMString publicId, DOMString systemId);
  [NewObject] XMLDocument createDocument(DOMString? namespace, [LegacyNullToEmptyString] DOMString qualifiedName, optional DocumentType? doctype = null);
  [NewObject] Document createHTMLDocument(optional DOMString title);

  boolean hasFeature(); // historical; always returns true
};
```

<strong><code><var>doctype</var> = <var>document</var> . <code><a id="ref-for-dom-document-implementation②"></a>[implementation](#dom-document-implementation)</code> . <code><a id="ref-for-dom-domimplementation-createdocumenttype①"></a>[createDocumentType(name, publicId, systemId)](#dom-domimplementation-createdocumenttype)</code></code></strong>

Returns a <a id="ref-for-concept-doctype①①"></a>[doctype](#concept-doctype), with the given <var>name</var>, <var>publicId</var>, and <var>systemId</var>.

If <var>name</var> is not a <a id="ref-for-valid-doctype-name①"></a>[valid doctype name](#valid-doctype-name), an "<code><a id="ref-for-invalidcharactererror①⓪"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException⑥①"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code> is thrown.

<strong><code><var>doc</var> = <var>document</var> . <code><a id="ref-for-dom-document-implementation③"></a>[implementation](#dom-document-implementation)</code> . <a id="ref-for-dom-domimplementation-createdocument②"></a>[createDocument(<var>namespace</var>, <var>qualifiedName</var> \[, <var>doctype</var> = null\])](#dom-domimplementation-createdocument)</code></strong>

Returns an <code><a id="ref-for-xmldocument②"></a>[XMLDocument](#xmldocument)</code>, with a <a id="ref-for-document-element⑦"></a>[document element](#document-element) whose <a id="ref-for-concept-element-local-name⑨"></a>[local name](#concept-element-local-name) is <var>qualifiedName</var> and whose <a id="ref-for-concept-element-namespace①③"></a>[namespace](#concept-element-namespace) is <var>namespace</var> (unless <var>qualifiedName</var> is the empty string), and with <var>doctype</var>, if it is given, as its <a id="ref-for-concept-doctype①②"></a>[doctype](#concept-doctype).

This method throws the same exceptions as the <code><a id="ref-for-dom-document-createelementns③"></a>[createElementNS()](#dom-document-createelementns)</code> method, when invoked with <var>namespace</var> and <var>qualifiedName</var>.

<strong><code><var>doc</var> = <var>document</var> . <code><a id="ref-for-dom-document-implementation④"></a>[implementation](#dom-document-implementation)</code> . <a id="ref-for-dom-domimplementation-createhtmldocument①"></a>[createHTMLDocument(\[<var>title</var>\])](#dom-domimplementation-createhtmldocument)</code></strong>

Returns a <a id="ref-for-concept-document④⑧"></a>[document](#concept-document), with a basic <a id="ref-for-concept-tree②①"></a>[tree](#concept-tree) already constructed including a <code><a id="ref-for-the-title-element"></a>[title](https://html.spec.whatwg.org/multipage/semantics.html#the-title-element)</code> element, unless the <var>title</var> argument is omitted.

The <a id="dom-domimplementation-createdocumenttype"></a><strong><code>createDocumentType(<var>name</var>, <var>publicId</var>, <var>systemId</var>)</code></strong> method steps are:

1.  If <var>name</var> is not a <a id="ref-for-valid-doctype-name②"></a>[valid doctype name](#valid-doctype-name), then throw an "<code><a id="ref-for-invalidcharactererror①①"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException⑥②"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  Return the result of <a id="ref-for-create-a-doctype"></a>[creating a doctype](#create-a-doctype) given the associated <a id="ref-for-concept-document④⑨"></a>[document](#concept-document) of <a id="ref-for-this①⑥④"></a>[this](https://webidl.spec.whatwg.org/#this), <var>name</var>, <var>publicId</var>, and <var>systemId</var>.

The <a id="dom-domimplementation-createdocument"></a><strong><code>createDocument(<var>namespace</var>, <var>qualifiedName</var>, <var>doctype</var>)</code></strong> method steps are:

1.  Let <var>document</var> be the result of <a id="ref-for-create-a-document②"></a>[creating a document](#create-a-document) that implements <code><a id="ref-for-xmldocument③"></a>[XMLDocument](#xmldocument)</code>, given <a id="ref-for-this①⑥⑤"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-relevant-realm③"></a>[relevant realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm).

2.  Let <var>element</var> be null.

3.  If <var>qualifiedName</var> is not the empty string, then set <var>element</var> to the result of running the <a id="ref-for-internal-createelementns-steps①"></a>[internal <code>createElementNS</code> steps](#internal-createelementns-steps), given <var>document</var>, <var>namespace</var>, <var>qualifiedName</var>, and an empty dictionary.

4.  If <var>doctype</var> is non-null, <a id="ref-for-concept-node-append④"></a>[append](#concept-node-append) <var>doctype</var> to <var>document</var>.

5.  If <var>element</var> is non-null, <a id="ref-for-concept-node-append⑤"></a>[append](#concept-node-append) <var>element</var> to <var>document</var>.

6.  <var>document</var>’s <a id="ref-for-concept-document-origin④"></a>[origin](#concept-document-origin) is <a id="ref-for-this①⑥⑥"></a>[this](https://webidl.spec.whatwg.org/#this)’s associated <a id="ref-for-concept-document⑤⓪"></a>[document](#concept-document)’s <a id="ref-for-concept-document-origin⑤"></a>[origin](#concept-document-origin).

7.  <var>document</var>’s <a id="ref-for-concept-document-content-type⑥"></a>[content type](#concept-document-content-type) is determined by <var>namespace</var>:

    <strong><a id="ref-for-html-namespace①①"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace)</strong>

    <code>application/xhtml+xml</code>
    <strong><a id="ref-for-svg-namespace"></a>[SVG namespace](https://infra.spec.whatwg.org/#svg-namespace)</strong>

    <code>image/svg+xml</code>
    <strong>Any other namespace</strong>

    <code>application/xml</code>

8.  Return <var>document</var>.

The <a id="dom-domimplementation-createhtmldocument"></a><strong><code>createHTMLDocument(<var>title</var>)</code></strong> method steps are:

1.  Let <var>doc</var> be the result of <a id="ref-for-create-a-document③"></a>[creating a document](#create-a-document) that implements <code><a id="ref-for-document②④"></a>[Document](#document)</code>, given <a id="ref-for-this①⑥⑦"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-relevant-realm④"></a>[relevant realm](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-realm).

2.  Set <var>doc</var>’s <a id="ref-for-concept-document-type④"></a>[type](#concept-document-type) to "<code>html</code>" and <a id="ref-for-concept-document-content-type⑦"></a>[content type](#concept-document-content-type) to "<code>text/html</code>".

3.  Let <var>doctype</var> be the result of <a id="ref-for-create-a-doctype①"></a>[creating a doctype](#create-a-doctype) given <var>doc</var> and "<code>html</code>".

4.  <a id="ref-for-concept-node-append⑥"></a>[Append](#concept-node-append) <var>doctype</var> to <var>doc</var>.

5.  <a id="ref-for-concept-node-append⑦"></a>[Append](#concept-node-append) the result of <a id="ref-for-concept-create-element③"></a>[creating an element](#concept-create-element) given <var>doc</var>, "<code>html</code>", and the <a id="ref-for-html-namespace①②"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace), to <var>doc</var>.

6.  <a id="ref-for-concept-node-append⑧"></a>[Append](#concept-node-append) the result of <a id="ref-for-concept-create-element④"></a>[creating an element](#concept-create-element) given <var>doc</var>, "<code>head</code>", and the <a id="ref-for-html-namespace①③"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace), to the <code><a id="ref-for-the-html-element"></a>[html](https://html.spec.whatwg.org/multipage/semantics.html#the-html-element)</code> element created earlier.

7.  If <var>title</var> is given:

    1.  <a id="ref-for-concept-node-append⑨"></a>[Append](#concept-node-append) the result of <a id="ref-for-concept-create-element⑤"></a>[creating an element](#concept-create-element) given <var>doc</var>, "<code>title</code>", and the <a id="ref-for-html-namespace①④"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace), to the <code><a id="ref-for-the-head-element"></a>[head](https://html.spec.whatwg.org/multipage/semantics.html#the-head-element)</code> element created earlier.

    2.  Let <var>text</var> be the result of <a id="ref-for-create-a-text-node③"></a>[creating a text node](#create-a-text-node) given <var>doc</var> and <var>title</var> (which could be the empty string).

    3.  <a id="ref-for-concept-node-append①⓪"></a>[Append](#concept-node-append) <var>text</var> to the <code><a id="ref-for-the-title-element①"></a>[title](https://html.spec.whatwg.org/multipage/semantics.html#the-title-element)</code> element created earlier.

8.  <a id="ref-for-concept-node-append①①"></a>[Append](#concept-node-append) the result of <a id="ref-for-concept-create-element⑥"></a>[creating an element](#concept-create-element) given <var>doc</var>, "<code>body</code>", and the <a id="ref-for-html-namespace①⑤"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace), to the <code><a id="ref-for-the-html-element①"></a>[html](https://html.spec.whatwg.org/multipage/semantics.html#the-html-element)</code> element created earlier.

9.  <var>doc</var>’s <a id="ref-for-concept-document-origin⑥"></a>[origin](#concept-document-origin) is <a id="ref-for-this①⑥⑧"></a>[this](https://webidl.spec.whatwg.org/#this)’s associated <a id="ref-for-concept-document⑤①"></a>[document](#concept-document)’s <a id="ref-for-concept-document-origin⑦"></a>[origin](#concept-document-origin).

10. Return <var>doc</var>.

The <a id="dom-domimplementation-hasfeature"></a><strong><code>hasFeature()</code></strong> method steps are to return true.

<strong>Note:</strong>

<code><a id="ref-for-dom-domimplementation-hasfeature①"></a>[hasFeature()](#dom-domimplementation-hasfeature)</code> originally would report whether the user agent claimed to support a given DOM feature, but experience proved it was not nearly as reliable or granular as simply checking whether the desired objects, attributes, or methods existed. As such, it is no longer to be used, but continues to exist (and simply returns true) so that old pages don’t stop working.

### <a id="interface-documenttype"></a>4.6. Interface <code><a id="ref-for-documenttype①⑧"></a>[DocumentType](#documenttype)</code>[](#interface-documenttype)

<a id="ref-for-Exposed⑨"></a><a id="documenttype"></a><a id="ref-for-node⑥①"></a><a id="ref-for-idl-DOMString⑥⓪"></a><a id="ref-for-dom-documenttype-name"></a><a id="ref-for-idl-DOMString⑥①"></a><a id="ref-for-dom-documenttype-publicid"></a><a id="ref-for-idl-DOMString⑥②"></a><a id="ref-for-dom-documenttype-systemid"></a>

``` text
[Exposed=Window]
interface DocumentType : Node {
  readonly attribute DOMString name;
  readonly attribute DOMString publicId;
  readonly attribute DOMString systemId;
};
```

<code><a id="ref-for-documenttype①⑨"></a>[DocumentType](#documenttype)</code> <a id="ref-for-concept-node①⑤⑤"></a>[nodes](#concept-node) are simply known as <a id="concept-doctype"></a><strong>doctypes</strong>.

<a id="ref-for-concept-doctype①③"></a>[Doctypes](#concept-doctype) have an associated <a id="concept-doctype-name"></a><strong>name</strong> (a string), <a id="concept-doctype-publicid"></a><strong>public ID</strong> (a string), and <a id="concept-doctype-systemid"></a><strong>system ID</strong> (a string).

To <a id="create-a-doctype"></a><strong>create a doctype</strong> given a <a id="ref-for-concept-document⑤②"></a>[document](#concept-document) <var>document</var>, a string <var>name</var>, and optionally a string <var>publicId</var> (default the empty string) and a string <var>systemId</var> (default the empty string):

1.  Let <var>doctype</var> be the result of <a id="ref-for-create-a-node①"></a>[creating a node](#create-a-node) that implements <code><a id="ref-for-documenttype②⓪"></a>[DocumentType](#documenttype)</code>, given <var>document</var>.

2.  Set <var>doctype</var>’s <a id="ref-for-concept-doctype-name④"></a>[name](#concept-doctype-name) to <var>name</var>, <a id="ref-for-concept-doctype-publicid②"></a>[public ID](#concept-doctype-publicid) to <var>publicId</var>, and <a id="ref-for-concept-doctype-systemid②"></a>[system ID](#concept-doctype-systemid) to <var>systemId</var>.

3.  Return <var>doctype</var>.

The <a id="dom-documenttype-name"></a><strong><code>name</code></strong> getter steps are to return <a id="ref-for-this①⑥⑨"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-doctype-name⑤"></a>[name](#concept-doctype-name).

The <a id="dom-documenttype-publicid"></a><strong><code>publicId</code></strong> getter steps are to return <a id="ref-for-this①⑦⓪"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-doctype-publicid③"></a>[public ID](#concept-doctype-publicid).

The <a id="dom-documenttype-systemid"></a><strong><code>systemId</code></strong> getter steps are to return <a id="ref-for-this①⑦①"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-doctype-systemid③"></a>[system ID](#concept-doctype-systemid).

### <a id="interface-documentfragment"></a>4.7. Interface <code><a id="ref-for-documentfragment②④"></a>[DocumentFragment](#documentfragment)</code>[](#interface-documentfragment)

<a id="ref-for-Exposed①⓪"></a><a id="documentfragment"></a><a id="ref-for-node⑥②"></a><a id="ref-for-dom-documentfragment-documentfragment"></a>

``` text
[Exposed=Window]
interface DocumentFragment : Node {
  constructor();
};
```

<strong><code><var>tree</var> = new <code><a id="ref-for-dom-documentfragment-documentfragment①"></a>[DocumentFragment()](#dom-documentfragment-documentfragment)</code></code></strong>

Returns a new <code><a id="ref-for-documentfragment②⑤"></a>[DocumentFragment](#documentfragment)</code> <a id="ref-for-concept-node①⑤⑥"></a>[node](#concept-node).

------------------------------------------------------------------------

An <a id="exclusive-documentfragment-node"></a><strong>exclusive <code><a id="ref-for-documentfragment②⑥"></a>[DocumentFragment](#documentfragment)</code> node</strong> is a <code><a id="ref-for-documentfragment②⑦"></a>[DocumentFragment](#documentfragment)</code> <a id="ref-for-concept-node①⑤⑦"></a>[node](#concept-node) that is not a <code><a id="ref-for-shadowroot⑤"></a>[ShadowRoot](#shadowroot)</code> <a id="ref-for-concept-node①⑤⑧"></a>[node](#concept-node).

A <code><a id="ref-for-documentfragment②⑧"></a>[DocumentFragment](#documentfragment)</code> <a id="ref-for-concept-node①⑤⑨"></a>[node](#concept-node) has an associated <a id="concept-documentfragment-host"></a><strong>host</strong> (null or an <a id="ref-for-concept-element⑥⑧"></a>[element](#concept-element) in a different <a id="ref-for-concept-node-tree②①"></a>[node tree](#concept-node-tree)). It is null unless otherwise stated.

An object <var>A</var> is a <a id="concept-tree-host-including-inclusive-ancestor"></a><strong>host-including inclusive ancestor</strong> of an object <var>B</var>, if either <var>A</var> is an <a id="ref-for-concept-tree-inclusive-ancestor④"></a>[inclusive ancestor](#concept-tree-inclusive-ancestor) of <var>B</var>, or if <var>B</var>’s <a id="ref-for-concept-tree-root②⑦"></a>[root](#concept-tree-root) has a non-null <a id="ref-for-concept-documentfragment-host③"></a>[host](#concept-documentfragment-host) and <var>A</var> is a <a id="ref-for-concept-tree-host-including-inclusive-ancestor②"></a>[host-including inclusive ancestor](#concept-tree-host-including-inclusive-ancestor) of <var>B</var>’s <a id="ref-for-concept-tree-root②⑧"></a>[root](#concept-tree-root)’s <a id="ref-for-concept-documentfragment-host④"></a>[host](#concept-documentfragment-host).

<strong>Note:</strong>

The <code><a id="ref-for-documentfragment②⑨"></a>[DocumentFragment](#documentfragment)</code> <a id="ref-for-concept-node①⑥⓪"></a>[node](#concept-node)’s <a id="ref-for-concept-documentfragment-host⑤"></a>[host](#concept-documentfragment-host) concept is useful for HTML’s <code><a id="ref-for-the-template-element①"></a>[template](https://html.spec.whatwg.org/multipage/scripting.html#the-template-element)</code> element and for <a id="ref-for-concept-shadow-root②⑥"></a>[shadow roots](#concept-shadow-root), and impacts the <a id="ref-for-concept-node-pre-insert⑥"></a>[pre-insert](#concept-node-pre-insert) and <a id="ref-for-concept-node-replace②"></a>[replace](#concept-node-replace) algorithms.

To <a id="create-a-document-fragment"></a><strong>create a document fragment</strong> given a <a id="ref-for-concept-document⑤③"></a>[document](#concept-document) <var>document</var>: return the result of <a id="ref-for-create-a-node②"></a>[creating a node](#create-a-node) that implements <code><a id="ref-for-documentfragment③⓪"></a>[DocumentFragment](#documentfragment)</code>, given <var>document</var>.

The <a id="dom-documentfragment-documentfragment"></a><strong><code>new DocumentFragment()</code></strong> constructor steps are to set <a id="ref-for-this①⑦②"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document③⑥"></a>[node document](#concept-node-document) to <a id="ref-for-current-global-object①"></a>[current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object)’s <a id="ref-for-concept-document-window③"></a>[associated <code>Document</code>](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

### <a id="interface-shadowroot"></a>4.8. Interface <code><a id="ref-for-shadowroot⑥"></a>[ShadowRoot](#shadowroot)</code>[](#interface-shadowroot)

<a id="ref-for-Exposed①①"></a><a id="shadowroot"></a><a id="ref-for-documentfragment③①"></a><a id="ref-for-enumdef-shadowrootmode"></a><a id="ref-for-dom-shadowroot-mode"></a><a id="ref-for-idl-boolean③⑧"></a><a id="ref-for-dom-shadowroot-delegatesfocus"></a><a id="ref-for-idl-boolean③⑨"></a><a id="ref-for-dom-shadowroot-serializable"></a><a id="ref-for-enumdef-slotassignmentmode"></a><a id="ref-for-dom-shadowroot-slotassignment"></a><a id="ref-for-idl-boolean④⓪"></a><a id="ref-for-dom-shadowroot-clonable"></a><a id="ref-for-element④③"></a><a id="ref-for-dom-shadowroot-host"></a><a id="ref-for-eventhandler①"></a><a id="ref-for-dom-shadowroot-onslotchange"></a><a id="enumdef-shadowrootmode"></a><a id="dom-shadowrootmode-open"></a><a id="dom-shadowrootmode-closed"></a><a id="enumdef-slotassignmentmode"></a><a id="dom-slotassignmentmode-manual"></a><a id="dom-slotassignmentmode-named"></a>

``` text
[Exposed=Window]
interface ShadowRoot : DocumentFragment {
  readonly attribute ShadowRootMode mode;
  readonly attribute boolean delegatesFocus;
  readonly attribute boolean serializable;
  readonly attribute SlotAssignmentMode slotAssignment;
  readonly attribute boolean clonable;
  readonly attribute Element host;

  attribute EventHandler onslotchange;
};

enum ShadowRootMode { "open", "closed" };
enum SlotAssignmentMode { "manual", "named" };
```

<code><a id="ref-for-shadowroot⑦"></a>[ShadowRoot](#shadowroot)</code> <a id="ref-for-concept-node①⑥①"></a>[nodes](#concept-node) are simply known as <a id="concept-shadow-root"></a><strong>shadow roots</strong>.

<a id="ref-for-concept-shadow-root②⑦"></a>[Shadow roots](#concept-shadow-root)’s associated <a id="ref-for-concept-documentfragment-host⑥"></a>[host](#concept-documentfragment-host) is never null.

<a id="ref-for-concept-shadow-root②⑧"></a>[Shadow roots](#concept-shadow-root) have an associated <a id="shadowroot-mode"></a><strong>mode</strong> ("<code>open</code>" or "<code>closed</code>").

<a id="ref-for-concept-shadow-root②⑨"></a>[Shadow roots](#concept-shadow-root) have an associated <a id="shadowroot-delegates-focus"></a><strong>delegates focus</strong> (a boolean). It is initially set to false.

<a id="ref-for-concept-shadow-root③⓪"></a>[Shadow roots](#concept-shadow-root) have an associated <a id="shadowroot-available-to-element-internals"></a><strong>available to element internals</strong> (a boolean). It is initially set to false.

<a id="ref-for-concept-shadow-root③①"></a>[Shadow roots](#concept-shadow-root) have an associated <a id="shadowroot-declarative"></a><strong>declarative</strong> (a boolean). It is initially set to false.

<a id="ref-for-concept-shadow-root③②"></a>[Shadow roots](#concept-shadow-root) have an associated <a id="shadowroot-serializable"></a><strong>serializable</strong> (a boolean). It is initially set to false.

<a id="ref-for-concept-shadow-root③③"></a>[Shadow roots](#concept-shadow-root) have an associated <a id="shadowroot-slot-assignment"></a><strong>slot assignment</strong> ("<code>manual</code>" or "<code>named</code>").

<a id="ref-for-concept-shadow-root③④"></a>[Shadow roots](#concept-shadow-root) have an associated <a id="shadowroot-clonable"></a><strong>clonable</strong> (a boolean). It is initially set to false.

<a id="ref-for-concept-shadow-root③⑤"></a>[Shadow roots](#concept-shadow-root) have an associated <a id="shadowroot-custom-element-registry"></a><strong>custom element registry</strong> (null or a <code><a id="ref-for-customelementregistry①②"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code> object). It is initially null.

<a id="ref-for-concept-shadow-root③⑥"></a>[Shadow roots](#concept-shadow-root) have an associated <a id="shadowroot-keep-custom-element-registry-null"></a><strong>keep custom element registry null</strong> (a boolean). It is initially false.

<strong>Note:</strong>

This can only ever be true in combination with declarative shadow roots. And it only matters for as long as the <a id="ref-for-concept-shadow-root③⑦"></a>[shadow root](#concept-shadow-root)’s <a id="ref-for-shadowroot-custom-element-registry⑧"></a>[custom element registry](#shadowroot-custom-element-registry) is null.

------------------------------------------------------------------------

A <a id="ref-for-concept-shadow-root③⑧"></a>[shadow root](#concept-shadow-root)’s <a id="ref-for-get-the-parent⑦"></a>[get the parent](https://dom.spec.whatwg.org/#get-the-parent) algorithm, given an <var>event</var>, returns null if <var>event</var>’s <a id="ref-for-composed-flag①"></a>[composed flag](https://dom.spec.whatwg.org/#composed-flag) is unset and <a id="ref-for-concept-shadow-root③⑨"></a>[shadow root](#concept-shadow-root) is the <a id="ref-for-concept-tree-root②⑨"></a>[root](#concept-tree-root) of <var>event</var>’s <a id="ref-for-event-path①⓪"></a>[path](https://dom.spec.whatwg.org/#event-path)’s first <a id="ref-for-event-path-item⑤"></a>[event path item](https://dom.spec.whatwg.org/#event-path-item)’s <a id="ref-for-event-path-invocation-target⑥"></a>[invocation target](https://dom.spec.whatwg.org/#event-path-invocation-target); otherwise <a id="ref-for-concept-shadow-root④⓪"></a>[shadow root](#concept-shadow-root)’s <a id="ref-for-concept-documentfragment-host⑦"></a>[host](#concept-documentfragment-host).

------------------------------------------------------------------------

The <a id="dom-shadowroot-mode"></a><strong><code>mode</code></strong> getter steps are to return <a id="ref-for-this①⑦③"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-shadowroot-mode⑤"></a>[mode](#shadowroot-mode).

The <a id="dom-shadowroot-delegatesfocus"></a><strong><code>delegatesFocus</code></strong> getter steps are to return <a id="ref-for-this①⑦④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-shadowroot-delegates-focus①"></a>[delegates focus](#shadowroot-delegates-focus).

The <a id="dom-shadowroot-serializable"></a><strong><code>serializable</code></strong> getter steps are to return <a id="ref-for-this①⑦⑤"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-shadowroot-serializable①"></a>[serializable](#shadowroot-serializable).

The <a id="dom-shadowroot-slotassignment"></a><strong><code>slotAssignment</code></strong> getter steps are to return <a id="ref-for-this①⑦⑥"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-shadowroot-slot-assignment⑤"></a>[slot assignment](#shadowroot-slot-assignment).

The <a id="dom-shadowroot-clonable"></a><strong><code>clonable</code></strong> getter steps are to return <a id="ref-for-this①⑦⑦"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-shadowroot-clonable①"></a>[clonable](#shadowroot-clonable).

The <a id="dom-shadowroot-host"></a><strong><code>host</code></strong> getter steps are to return <a id="ref-for-this①⑦⑧"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-documentfragment-host⑧"></a>[host](#concept-documentfragment-host).

------------------------------------------------------------------------

The <a id="dom-shadowroot-onslotchange"></a><strong><code>onslotchange</code></strong> attribute is an <a id="ref-for-event-handler-idl-attributes①"></a>[event handler IDL attribute](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-idl-attributes) for the <a id="shadowroot-onslotchange"></a><strong><code>onslotchange</code></strong> <a id="ref-for-event-handlers②"></a>[event handler](https://html.spec.whatwg.org/multipage/webappapis.html#event-handlers), whose <a id="ref-for-event-handler-event-type①"></a>[event handler event type](https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-event-type) is <code><a id="ref-for-eventdef-htmlslotelement-slotchange"></a>[slotchange](https://dom.spec.whatwg.org/#eventdef-htmlslotelement-slotchange)</code>.

------------------------------------------------------------------------

In <a id="concept-shadow-including-tree-order"></a><strong>shadow-including tree order</strong> is <a id="ref-for-shadow-including-preorder-depth-first-traversal"></a>[shadow-including preorder, depth-first traversal](#shadow-including-preorder-depth-first-traversal) of a <a id="ref-for-concept-node-tree②②"></a>[node tree](#concept-node-tree). <a id="shadow-including-preorder-depth-first-traversal"></a><strong>Shadow-including preorder, depth-first traversal</strong> of a <a id="ref-for-concept-node-tree②③"></a>[node tree](#concept-node-tree) <var>tree</var> is preorder, depth-first traversal of <var>tree</var>, with for each <a id="ref-for-element-shadow-host④"></a>[shadow host](#element-shadow-host) encountered in <var>tree</var>, <a id="ref-for-shadow-including-preorder-depth-first-traversal①"></a>[shadow-including preorder, depth-first traversal](#shadow-including-preorder-depth-first-traversal) of that <a id="ref-for-concept-element⑥⑨"></a>[element](#concept-element)’s <a id="ref-for-concept-element-shadow-root①③"></a>[shadow root](#concept-element-shadow-root)’s <a id="ref-for-concept-node-tree②④"></a>[node tree](#concept-node-tree) just after it is encountered.

The <a id="concept-shadow-including-root"></a><strong>shadow-including root</strong> of an object is its <a id="ref-for-concept-tree-root③⓪"></a>[root](#concept-tree-root)’s <a id="ref-for-concept-documentfragment-host⑨"></a>[host](#concept-documentfragment-host)’s <a id="ref-for-concept-shadow-including-root⑤"></a>[shadow-including root](#concept-shadow-including-root), if the object’s <a id="ref-for-concept-tree-root③①"></a>[root](#concept-tree-root) is a <a id="ref-for-concept-shadow-root④①"></a>[shadow root](#concept-shadow-root); otherwise its <a id="ref-for-concept-tree-root③②"></a>[root](#concept-tree-root).

An object <var>A</var> is a <a id="concept-shadow-including-descendant"></a><strong>shadow-including descendant</strong> of an object <var>B</var>, if <var>A</var> is a <a id="ref-for-concept-tree-descendant③⓪"></a>[descendant](#concept-tree-descendant) of <var>B</var>, or <var>A</var>’s <a id="ref-for-concept-tree-root③③"></a>[root](#concept-tree-root) is a <a id="ref-for-concept-shadow-root④②"></a>[shadow root](#concept-shadow-root) and <var>A</var>’s <a id="ref-for-concept-tree-root③④"></a>[root](#concept-tree-root)’s <a id="ref-for-concept-documentfragment-host①⓪"></a>[host](#concept-documentfragment-host) is a <a id="ref-for-concept-shadow-including-inclusive-descendant④"></a>[shadow-including inclusive descendant](#concept-shadow-including-inclusive-descendant) of <var>B</var>.

A <a id="concept-shadow-including-inclusive-descendant"></a><strong>shadow-including inclusive descendant</strong> is an object or one of its <a id="ref-for-concept-shadow-including-descendant①"></a>[shadow-including descendants](#concept-shadow-including-descendant).

An object <var>A</var> is a <a id="concept-shadow-including-ancestor"></a><strong>shadow-including ancestor</strong> of an object <var>B</var>, if and only if <var>B</var> is a <a id="ref-for-concept-shadow-including-descendant②"></a>[shadow-including descendant](#concept-shadow-including-descendant) of <var>A</var>.

A <a id="concept-shadow-including-inclusive-ancestor"></a><strong>shadow-including inclusive ancestor</strong> is an object or one of its <a id="ref-for-concept-shadow-including-ancestor"></a>[shadow-including ancestors](#concept-shadow-including-ancestor).

A <a id="ref-for-concept-node①⑥②"></a>[node](#concept-node) <var>A</var> is <a id="concept-closed-shadow-hidden"></a><strong>closed-shadow-hidden</strong> from a <a id="ref-for-concept-node①⑥③"></a>[node](#concept-node) <var>B</var> if all of the following conditions are true:

- <var>A</var>’s <a id="ref-for-concept-tree-root③⑤"></a>[root](#concept-tree-root) is a <a id="ref-for-concept-shadow-root④③"></a>[shadow root](#concept-shadow-root).

- <var>A</var>’s <a id="ref-for-concept-tree-root③⑥"></a>[root](#concept-tree-root) is not a <a id="ref-for-concept-shadow-including-inclusive-ancestor①"></a>[shadow-including inclusive ancestor](#concept-shadow-including-inclusive-ancestor) of <var>B</var>.

- <var>A</var>’s <a id="ref-for-concept-tree-root③⑦"></a>[root](#concept-tree-root) is a <a id="ref-for-concept-shadow-root④④"></a>[shadow root](#concept-shadow-root) whose <a id="ref-for-shadowroot-mode⑥"></a>[mode](#shadowroot-mode) is "<code>closed</code>" or <var>A</var>’s <a id="ref-for-concept-tree-root③⑧"></a>[root](#concept-tree-root)’s <a id="ref-for-concept-documentfragment-host①①"></a>[host](#concept-documentfragment-host) is <a id="ref-for-concept-closed-shadow-hidden"></a>[closed-shadow-hidden](#concept-closed-shadow-hidden) from <var>B</var>.

To <a id="retarget"></a><strong>retarget</strong> an object <var>A</var> against an object <var>B</var>, repeat these steps until they return an object:

1.  If one of the following is true

    - <var>A</var> is not a <a id="ref-for-concept-node①⑥④"></a>[node](#concept-node)
    - <var>A</var>’s <a id="ref-for-concept-tree-root③⑨"></a>[root](#concept-tree-root) is not a <a id="ref-for-concept-shadow-root④⑤"></a>[shadow root](#concept-shadow-root)
    - <var>B</var> is a <a id="ref-for-concept-node①⑥⑤"></a>[node](#concept-node) and <var>A</var>’s <a id="ref-for-concept-tree-root④⓪"></a>[root](#concept-tree-root) is a <a id="ref-for-concept-shadow-including-inclusive-ancestor②"></a>[shadow-including inclusive ancestor](#concept-shadow-including-inclusive-ancestor) of <var>B</var>

    then return <var>A</var>.

2.  Set <var>A</var> to <var>A</var>’s <a id="ref-for-concept-tree-root④①"></a>[root](#concept-tree-root)’s <a id="ref-for-concept-documentfragment-host①②"></a>[host](#concept-documentfragment-host).

<strong>Note:</strong>

The <a id="ref-for-retarget④"></a>[retargeting](#retarget) algorithm is used by <a id="ref-for-concept-event-dispatch②⑦"></a>[event dispatch](https://dom.spec.whatwg.org/#concept-event-dispatch) as well as other specifications, such as Fullscreen. [\[FULLSCREEN\]](https://dom.spec.whatwg.org/#biblio-fullscreen)

### <a id="interface-element"></a>4.9. Interface <code><a id="ref-for-element④④"></a>[Element](#element)</code>[](#interface-element)

<a id="ref-for-Exposed①②"></a><a id="element"></a><a id="ref-for-node⑥③"></a><a id="ref-for-idl-DOMString⑥③"></a><a id="ref-for-dom-element-namespaceuri"></a><a id="ref-for-idl-DOMString⑥④"></a><a id="ref-for-dom-element-prefix"></a><a id="ref-for-idl-DOMString⑥⑤"></a><a id="ref-for-dom-element-localname"></a><a id="ref-for-idl-DOMString⑥⑥"></a><a id="ref-for-dom-element-tagname"></a><a id="ref-for-cereactions②⓪"></a><a id="ref-for-idl-DOMString⑥⑦"></a><a id="ref-for-dom-element-id"></a><a id="ref-for-cereactions②①"></a><a id="ref-for-idl-DOMString⑥⑧"></a><a id="ref-for-dom-element-classname"></a><a id="ref-for-SameObject⑦"></a><a id="ref-for-PutForwards"></a><a id="ref-for-dom-domtokenlist-value"></a><a id="ref-for-domtokenlist"></a><a id="ref-for-dom-element-classlist"></a><a id="ref-for-cereactions②②"></a><a id="ref-for-Unscopable⑦"></a><a id="ref-for-idl-DOMString⑥⑨"></a><a id="ref-for-dom-element-slot"></a><a id="ref-for-idl-boolean④①"></a><a id="ref-for-dom-element-hasattributes"></a><a id="ref-for-SameObject⑧"></a><a id="ref-for-namednodemap"></a><a id="ref-for-dom-element-attributes"></a><a id="ref-for-idl-sequence⑤"></a><a id="ref-for-idl-DOMString⑦⓪"></a><a id="ref-for-dom-element-getattributenames"></a><a id="ref-for-idl-DOMString⑦①"></a><a id="ref-for-dom-element-getattribute"></a><a id="ref-for-idl-DOMString⑦②"></a><a id="dom-element-getattribute-qualifiedname-qualifiedname"></a><a id="ref-for-idl-DOMString⑦③"></a><a id="ref-for-dom-element-getattributens"></a><a id="ref-for-idl-DOMString⑦④"></a><a id="dom-element-getattributens-namespace-localname-namespace"></a><a id="ref-for-idl-DOMString⑦⑤"></a><a id="dom-element-getattributens-namespace-localname-localname"></a><a id="ref-for-cereactions②③"></a><a id="ref-for-idl-undefined②③"></a><a id="ref-for-dom-element-setattribute"></a><a id="ref-for-idl-DOMString⑦⑥"></a><a id="dom-element-setattribute-qualifiedname-value-qualifiedname"></a><a id="ref-for-typedefdef-trustedtype"></a><a id="ref-for-idl-DOMString⑦⑦"></a><a id="dom-element-setattribute-qualifiedname-value-value"></a><a id="ref-for-cereactions②④"></a><a id="ref-for-idl-undefined②④"></a><a id="ref-for-dom-element-setattributens"></a><a id="ref-for-idl-DOMString⑦⑧"></a><a id="dom-element-setattributens-namespace-qualifiedname-value-namespace"></a><a id="ref-for-idl-DOMString⑦⑨"></a><a id="dom-element-setattributens-namespace-qualifiedname-value-qualifiedname"></a><a id="ref-for-typedefdef-trustedtype①"></a><a id="ref-for-idl-DOMString⑧⓪"></a><a id="dom-element-setattributens-namespace-qualifiedname-value-value"></a><a id="ref-for-cereactions②⑤"></a><a id="ref-for-idl-undefined②⑤"></a><a id="ref-for-dom-element-removeattribute"></a><a id="ref-for-idl-DOMString⑧①"></a><a id="dom-element-removeattribute-qualifiedname-qualifiedname"></a><a id="ref-for-cereactions②⑥"></a><a id="ref-for-idl-undefined②⑥"></a><a id="ref-for-dom-element-removeattributens"></a><a id="ref-for-idl-DOMString⑧②"></a><a id="dom-element-removeattributens-namespace-localname-namespace"></a><a id="ref-for-idl-DOMString⑧③"></a><a id="dom-element-removeattributens-namespace-localname-localname"></a><a id="ref-for-cereactions②⑦"></a><a id="ref-for-idl-boolean④②"></a><a id="ref-for-dom-element-toggleattribute"></a><a id="ref-for-idl-DOMString⑧④"></a><a id="dom-element-toggleattribute-qualifiedname-force-qualifiedname"></a><a id="ref-for-idl-boolean④③"></a><a id="dom-element-toggleattribute-qualifiedname-force-force"></a><a id="ref-for-idl-boolean④④"></a><a id="ref-for-dom-element-hasattribute"></a><a id="ref-for-idl-DOMString⑧⑤"></a><a id="dom-element-hasattribute-qualifiedname-qualifiedname"></a><a id="ref-for-idl-boolean④⑤"></a><a id="ref-for-dom-element-hasattributens"></a><a id="ref-for-idl-DOMString⑧⑥"></a><a id="dom-element-hasattributens-namespace-localname-namespace"></a><a id="ref-for-idl-DOMString⑧⑦"></a><a id="dom-element-hasattributens-namespace-localname-localname"></a><a id="ref-for-attr①⑨"></a><a id="ref-for-dom-element-getattributenode"></a><a id="ref-for-idl-DOMString⑧⑧"></a><a id="dom-element-getattributenode-qualifiedname-qualifiedname"></a><a id="ref-for-attr②⓪"></a><a id="ref-for-dom-element-getattributenodens"></a><a id="ref-for-idl-DOMString⑧⑨"></a><a id="dom-element-getattributenodens-namespace-localname-namespace"></a><a id="ref-for-idl-DOMString⑨⓪"></a><a id="dom-element-getattributenodens-namespace-localname-localname"></a><a id="ref-for-cereactions②⑧"></a><a id="ref-for-attr②①"></a><a id="ref-for-dom-element-setattributenode"></a><a id="ref-for-attr②②"></a><a id="dom-element-setattributenode-attr-attr"></a><a id="ref-for-cereactions②⑨"></a><a id="ref-for-attr②③"></a><a id="ref-for-dom-element-setattributenodens"></a><a id="ref-for-attr②④"></a><a id="dom-element-setattributenodens-attr-attr"></a><a id="ref-for-cereactions③⓪"></a><a id="ref-for-attr②⑤"></a><a id="ref-for-dom-element-removeattributenode"></a><a id="ref-for-attr②⑥"></a><a id="dom-element-removeattributenode-attr-attr"></a><a id="ref-for-shadowroot⑧"></a><a id="ref-for-dom-element-attachshadow"></a><a id="ref-for-dictdef-shadowrootinit"></a><a id="dom-element-attachshadow-init-init"></a><a id="ref-for-shadowroot⑨"></a><a id="ref-for-dom-element-shadowroot"></a><a id="ref-for-customelementregistry①③"></a><a id="ref-for-dom-element-customelementregistry"></a><a id="ref-for-element④⑤"></a><a id="ref-for-dom-element-closest"></a><a id="ref-for-idl-DOMString⑨①"></a><a id="dom-element-closest-selectors-selectors"></a><a id="ref-for-idl-boolean④⑥"></a><a id="ref-for-dom-element-matches"></a><a id="ref-for-idl-DOMString⑨②"></a><a id="dom-element-matches-selectors-selectors"></a><a id="ref-for-idl-boolean④⑦"></a><a id="ref-for-dom-element-webkitmatchesselector"></a><a id="ref-for-idl-DOMString⑨③"></a><a id="dom-element-webkitmatchesselector-selectors-selectors"></a><a id="ref-for-htmlcollection③②"></a><a id="ref-for-dom-element-getelementsbytagname"></a><a id="ref-for-idl-DOMString⑨④"></a><a id="dom-element-getelementsbytagname-qualifiedname-qualifiedname"></a><a id="ref-for-htmlcollection③③"></a><a id="ref-for-dom-element-getelementsbytagnamens"></a><a id="ref-for-idl-DOMString⑨⑤"></a><a id="dom-element-getelementsbytagnamens-namespace-localname-namespace"></a><a id="ref-for-idl-DOMString⑨⑥"></a><a id="dom-element-getelementsbytagnamens-namespace-localname-localname"></a><a id="ref-for-htmlcollection③④"></a><a id="ref-for-dom-element-getelementsbyclassname①"></a><a id="ref-for-idl-DOMString⑨⑦"></a><a id="dom-element-getelementsbyclassname-classnames-classnames"></a><a id="ref-for-cereactions③①"></a><a id="ref-for-element④⑥"></a><a id="ref-for-dom-element-insertadjacentelement"></a><a id="ref-for-idl-DOMString⑨⑧"></a><a id="dom-element-insertadjacentelement-where-element-where"></a><a id="ref-for-element④⑦"></a><a id="dom-element-insertadjacentelement-where-element-element"></a><a id="ref-for-idl-undefined②⑦"></a><a id="ref-for-dom-element-insertadjacenttext"></a><a id="ref-for-idl-DOMString⑨⑨"></a><a id="dom-element-insertadjacenttext-where-data-where"></a><a id="ref-for-idl-DOMString①⓪⓪"></a><a id="dom-element-insertadjacenttext-where-data-data"></a><a id="dictdef-shadowrootinit"></a><a id="ref-for-enumdef-shadowrootmode①"></a><a id="dom-shadowrootinit-mode"></a><a id="ref-for-idl-boolean④⑧"></a><a id="dom-shadowrootinit-delegatesfocus"></a><a id="ref-for-idl-boolean④⑨"></a><a id="dom-shadowrootinit-serializable"></a><a id="ref-for-enumdef-slotassignmentmode①"></a><a id="dom-shadowrootinit-slotassignment"></a><a id="ref-for-idl-boolean⑤⓪"></a><a id="dom-shadowrootinit-clonable"></a><a id="ref-for-customelementregistry①④"></a><a id="dom-shadowrootinit-customelementregistry"></a>

``` text
[Exposed=Window]
interface Element : Node {
  readonly attribute DOMString? namespaceURI;
  readonly attribute DOMString? prefix;
  readonly attribute DOMString localName;
  readonly attribute DOMString tagName;

  [CEReactions] attribute DOMString id;
  [CEReactions] attribute DOMString className;
  [SameObject, PutForwards=value] readonly attribute DOMTokenList classList;
  [CEReactions, Unscopable] attribute DOMString slot;

  boolean hasAttributes();
  [SameObject] readonly attribute NamedNodeMap attributes;
  sequence<DOMString> getAttributeNames();
  DOMString? getAttribute(DOMString qualifiedName);
  DOMString? getAttributeNS(DOMString? namespace, DOMString localName);
  [CEReactions] undefined setAttribute(DOMString qualifiedName, (TrustedType or DOMString) value);
  [CEReactions] undefined setAttributeNS(DOMString? namespace, DOMString qualifiedName, (TrustedType or DOMString) value);
  [CEReactions] undefined removeAttribute(DOMString qualifiedName);
  [CEReactions] undefined removeAttributeNS(DOMString? namespace, DOMString localName);
  [CEReactions] boolean toggleAttribute(DOMString qualifiedName, optional boolean force);
  boolean hasAttribute(DOMString qualifiedName);
  boolean hasAttributeNS(DOMString? namespace, DOMString localName);

  Attr? getAttributeNode(DOMString qualifiedName);
  Attr? getAttributeNodeNS(DOMString? namespace, DOMString localName);
  [CEReactions] Attr? setAttributeNode(Attr attr);
  [CEReactions] Attr? setAttributeNodeNS(Attr attr);
  [CEReactions] Attr removeAttributeNode(Attr attr);

  ShadowRoot attachShadow(ShadowRootInit init);
  readonly attribute ShadowRoot? shadowRoot;

  readonly attribute CustomElementRegistry? customElementRegistry;

  Element? closest(DOMString selectors);
  boolean matches(DOMString selectors);
  boolean webkitMatchesSelector(DOMString selectors); // legacy alias of .matches

  HTMLCollection getElementsByTagName(DOMString qualifiedName);
  HTMLCollection getElementsByTagNameNS(DOMString? namespace, DOMString localName);
  HTMLCollection getElementsByClassName(DOMString classNames);

  [CEReactions] Element? insertAdjacentElement(DOMString where, Element element); // legacy
  undefined insertAdjacentText(DOMString where, DOMString data); // legacy
};

dictionary ShadowRootInit {
  required ShadowRootMode mode;
  boolean delegatesFocus = false;
  boolean serializable = false;
  SlotAssignmentMode slotAssignment = "named";
  boolean clonable = false;
  CustomElementRegistry? customElementRegistry;
};
```

<strong>Note:</strong>

<code><a id="ref-for-dictdef-shadowrootinit①"></a>[ShadowRootInit](#dictdef-shadowrootinit)</code> somewhat unusually allows both <code>undefined</code> and <code>null</code> to be passed to its <code><a id="ref-for-dom-shadowrootinit-customelementregistry"></a>[customElementRegistry](#dom-shadowrootinit-customelementregistry)</code> member to allow web developers to pass a <code><a id="ref-for-shadowroot①⓪"></a>[ShadowRoot](#shadowroot)</code> node instead of a dictionary to <code><a id="ref-for-dom-element-attachshadow①"></a>[attachShadow()](#dom-element-attachshadow)</code>.

<code><a id="ref-for-element④⑧"></a>[Element](#element)</code> <a id="ref-for-concept-node①⑥⑥"></a>[nodes](#concept-node) are simply known as <a id="concept-element"></a><strong>elements</strong>.

<a id="ref-for-concept-element⑦⓪"></a>[Elements](#concept-element) have an associated:

<strong><a id="concept-element-namespace"></a><strong>namespace</strong></strong>

Null or a non-empty string.

<strong><a id="concept-element-namespace-prefix"></a><strong>namespace prefix</strong></strong>

Null or a non-empty string.

<strong><a id="concept-element-local-name"></a><strong>local name</strong></strong>

A non-empty string.

<strong><a id="element-custom-element-registry"></a><strong>custom element registry</strong></strong>

Null or a <code><a id="ref-for-customelementregistry①⑤"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code> object.

<strong><a id="concept-element-custom-element-state"></a><strong>custom element state</strong></strong>

"<code>undefined</code>", "<code>failed</code>", "<code>uncustomized</code>", "<code>precustomized</code>", or "<code>custom</code>".

<strong><a id="concept-element-custom-element-definition"></a><strong>custom element definition</strong></strong>

Null or a <a id="ref-for-custom-element-definition"></a>[custom element definition](https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element-definition).

<strong><a id="concept-element-is-value"></a><strong><code>is</code> value</strong></strong>

Null or a <a id="ref-for-valid-custom-element-name"></a>[valid custom element name](https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name).

When an <a id="ref-for-concept-element⑦①"></a>[element](#concept-element) is <a id="ref-for-concept-create-element⑦"></a>[created](#concept-create-element), all of these values are initialized.

An <a id="ref-for-concept-element⑦②"></a>[element](#concept-element) whose <a id="ref-for-concept-element-custom-element-state"></a>[custom element state](#concept-element-custom-element-state) is "<code>uncustomized</code>" or "<code>custom</code>" is said to be <a id="concept-element-defined"></a><strong>defined</strong>. An <a id="ref-for-concept-element⑦③"></a>[element](#concept-element) whose <a id="ref-for-concept-element-custom-element-state①"></a>[custom element state](#concept-element-custom-element-state) is "<code>custom</code>" is said to be <a id="concept-element-custom"></a><strong>custom</strong>.

<strong>Note:</strong>

Whether or not an element is <a id="ref-for-concept-element-defined"></a>[defined](#concept-element-defined) is used to determine the behavior of the <a id="ref-for-defined-pseudo"></a>[:defined](https://drafts.csswg.org/selectors-4/#defined-pseudo) pseudo-class. Whether or not an element is <a id="ref-for-concept-element-custom⑥"></a>[custom](#concept-element-custom) is used to determine the behavior of the [mutation algorithms](#mutation-algorithms). The "<code>failed</code>" and "<code>precustomized</code>" states are used to ensure that if a <a id="ref-for-custom-element-constructor"></a>[custom element constructor](https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element-constructor) fails to execute correctly the first time, it is not executed again by an <a id="ref-for-concept-upgrade-an-element①"></a>[upgrade](https://html.spec.whatwg.org/multipage/custom-elements.html#concept-upgrade-an-element).

<a id="example-c5b21302"></a>

<strong>Example:</strong>

[](#example-c5b21302)

The following code illustrates elements in each of these four states:

``` text
<!DOCTYPE html>
<script>
  window.customElements.define("sw-rey", class extends HTMLElement {})
  window.customElements.define("sw-finn", class extends HTMLElement {}, { extends: "p" })
  window.customElements.define("sw-kylo", class extends HTMLElement {
    constructor() {
      // super() intentionally omitted for this example
    }
  })
</script>

<!-- "undefined" (not defined, not custom) -->
<sw-han></sw-han>
<p is="sw-luke"></p>
<p is="asdf"></p>

<!-- "failed" (not defined, not custom) -->
<sw-kylo></sw-kylo>

<!-- "uncustomized" (defined, not custom) -->
<p></p>
<asdf></asdf>

<!-- "custom" (defined, custom) -->
<sw-rey></sw-rey>
<p is="sw-finn"></p>
```

<a id="ref-for-concept-element⑦④"></a>[Elements](#concept-element) also have an associated <a id="concept-element-shadow-root"></a><strong>shadow root</strong> (null or a <a id="ref-for-concept-shadow-root④⑥"></a>[shadow root](#concept-shadow-root)). It is null unless otherwise stated. An <a id="ref-for-concept-element⑦⑤"></a>[element](#concept-element) is a <a id="element-shadow-host"></a><strong>shadow host</strong> if its <a id="ref-for-concept-element-shadow-root①④"></a>[shadow root](#concept-element-shadow-root) is non-null.

An <a id="ref-for-concept-element⑦⑥"></a>[element](#concept-element)’s <a id="concept-element-qualified-name"></a><strong>qualified name</strong> is its <a id="ref-for-concept-element-local-name①⓪"></a>[local name](#concept-element-local-name) if its <a id="ref-for-concept-element-namespace-prefix①⓪"></a>[namespace prefix](#concept-element-namespace-prefix) is null; otherwise its <a id="ref-for-concept-element-namespace-prefix①①"></a>[namespace prefix](#concept-element-namespace-prefix), followed by "<code>&#58;</code>", followed by its <a id="ref-for-concept-element-local-name①①"></a>[local name](#concept-element-local-name).

An <a id="ref-for-concept-element⑦⑦"></a>[element](#concept-element)’s <a id="element-html-uppercased-qualified-name"></a><strong>HTML-uppercased qualified name</strong> is the return value of these steps:

1.  Let <var>qualifiedName</var> be <a id="ref-for-this①⑦⑨"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-element-qualified-name④"></a>[qualified name](#concept-element-qualified-name).

2.  If <a id="ref-for-this①⑧⓪"></a>[this](https://webidl.spec.whatwg.org/#this) is in the <a id="ref-for-html-namespace①⑥"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) and its <a id="ref-for-concept-node-document③⑦"></a>[node document](#concept-node-document) is an <a id="ref-for-html-document①⓪"></a>[HTML document](#html-document), then return <var>qualifiedName</var> in <a id="ref-for-ascii-uppercase"></a>[ASCII uppercase](https://infra.spec.whatwg.org/#ascii-uppercase).

3.  Return <var>qualifiedName</var>.

<strong>Note:</strong>

User agents could optimize <a id="ref-for-concept-element-qualified-name⑤"></a>[qualified name](#concept-element-qualified-name) and <a id="ref-for-element-html-uppercased-qualified-name②"></a>[HTML-uppercased qualified name](#element-html-uppercased-qualified-name) by storing them in internal slots.

To <a id="concept-create-element"></a><strong>create an element</strong>, given a <a id="ref-for-concept-document⑤④"></a>[document](#concept-document) <var>document</var>, string <var>localName</var>, string-or-null <var>namespace</var>, and optionally a string-or-null <var>prefix</var> (default null), string-or-null <var>is</var> (default null), boolean <var>synchronousCustomElements</var> (default false), and "<code>default</code>", null, or a <code><a id="ref-for-customelementregistry①⑥"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code> object <var>registry</var> (default "<code>default</code>"):

1.  Let <var>result</var> be null.

2.  If <var>registry</var> is "<code>default</code>", then set <var>registry</var> to the result of <a id="ref-for-look-up-a-custom-element-registry③"></a>[looking up a custom element registry](https://html.spec.whatwg.org/multipage/custom-elements.html#look-up-a-custom-element-registry) given <var>document</var>.

3.  Let <var>definition</var> be the result of <a id="ref-for-look-up-a-custom-element-definition"></a>[looking up a custom element definition](https://html.spec.whatwg.org/multipage/custom-elements.html#look-up-a-custom-element-definition) given <var>registry</var>, <var>namespace</var>, <var>localName</var>, and <var>is</var>.

4.  If <var>definition</var> is non-null, and <var>definition</var>’s <a id="ref-for-concept-custom-element-definition-name"></a>[name](https://html.spec.whatwg.org/multipage/custom-elements.html#concept-custom-element-definition-name) is not equal to its <a id="ref-for-concept-custom-element-definition-local-name"></a>[local name](https://html.spec.whatwg.org/multipage/custom-elements.html#concept-custom-element-definition-local-name) (i.e., <var>definition</var> represents a <a id="ref-for-customized-built-in-element②"></a>[customized built-in element](https://html.spec.whatwg.org/multipage/custom-elements.html#customized-built-in-element)):

    1.  Let <var>interface</var> be the <a id="ref-for-concept-element-interface"></a>[element interface](#concept-element-interface) for <var>localName</var> and the <a id="ref-for-html-namespace①⑦"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace).

    2.  Set <var>result</var> to the result of <a id="ref-for-create-an-element-internal"></a>[creating an element internal](#create-an-element-internal) given <var>document</var>, <var>interface</var>, <var>localName</var>, the <a id="ref-for-html-namespace①⑧"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace), <var>prefix</var>, "<code>undefined</code>", <var>is</var>, and <var>registry</var>.

    3.  If <var>synchronousCustomElements</var> is true, then run this step while catching any exceptions:

        1.  <a id="ref-for-concept-upgrade-an-element②"></a>[Upgrade](https://html.spec.whatwg.org/multipage/custom-elements.html#concept-upgrade-an-element) <var>result</var> using <var>definition</var>.

        If this step threw an exception <var>exception</var>:

        1.  <a id="ref-for-report-an-exception①"></a>[Report](https://html.spec.whatwg.org/multipage/webappapis.html#report-an-exception) <var>exception</var> for <var>definition</var>’s <a id="ref-for-concept-custom-element-definition-constructor"></a>[constructor](https://html.spec.whatwg.org/multipage/custom-elements.html#concept-custom-element-definition-constructor)’s corresponding JavaScript object’s <a id="ref-for-dfn-associated-realm②"></a>[associated realm](https://webidl.spec.whatwg.org/#dfn-associated-realm)’s <a id="ref-for-concept-realm-global②"></a>[global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-realm-global).

        2.  Set <var>result</var>’s <a id="ref-for-concept-element-custom-element-state②"></a>[custom element state](#concept-element-custom-element-state) to "<code>failed</code>".

    4.  Otherwise, <a id="ref-for-enqueue-a-custom-element-upgrade-reaction"></a>[enqueue a custom element upgrade reaction](https://html.spec.whatwg.org/multipage/custom-elements.html#enqueue-a-custom-element-upgrade-reaction) given <var>result</var> and <var>definition</var>.

5.  Otherwise, if <var>definition</var> is non-null:

    1.  If <var>synchronousCustomElements</var> is true:

        1.  Let <var>C</var> be <var>definition</var>’s <a id="ref-for-concept-custom-element-definition-constructor①"></a>[constructor](https://html.spec.whatwg.org/multipage/custom-elements.html#concept-custom-element-definition-constructor).

        2.  Let <var>previousRegistry</var> be the <a id="ref-for-surrounding-agent⑨"></a>[surrounding agent](https://tc39.es/ecma262/#surrounding-agent)’s <a id="ref-for-active-custom-element-constructor-map"></a>[active custom element constructor map](https://html.spec.whatwg.org/multipage/custom-elements.html#active-custom-element-constructor-map)\[<var>C</var>\] <a id="ref-for-map-with-default"></a>[with default](https://infra.spec.whatwg.org/#map-with-default) null.

        3.  <a id="ref-for-map-set②"></a>[Set](https://infra.spec.whatwg.org/#map-set) the <a id="ref-for-surrounding-agent①⓪"></a>[surrounding agent](https://tc39.es/ecma262/#surrounding-agent)’s <a id="ref-for-active-custom-element-constructor-map①"></a>[active custom element constructor map](https://html.spec.whatwg.org/multipage/custom-elements.html#active-custom-element-constructor-map)\[<var>C</var>\] to <var>registry</var>.

        4.  Run these steps while catching any exceptions:

            1.  Set <var>result</var> to the result of <a id="ref-for-construct-a-callback-function"></a>[constructing](https://webidl.spec.whatwg.org/#construct-a-callback-function) <var>C</var>, with no arguments.

            2.  <a id="ref-for-assert①①"></a>[Assert](https://infra.spec.whatwg.org/#assert): one of the following is true:

                - <var>result</var>’s <a id="ref-for-concept-element-custom-element-state③"></a>[custom element state](#concept-element-custom-element-state) is "<code>custom</code>" and its <a id="ref-for-concept-element-custom-element-definition"></a>[custom element definition](#concept-element-custom-element-definition) is non-null;
                - <var>result</var>’s <a id="ref-for-concept-element-custom-element-state④"></a>[custom element state](#concept-element-custom-element-state) is "<code>precustomized</code>"; or
                - <var>result</var>’s <a id="ref-for-concept-element-custom-element-state⑤"></a>[custom element state](#concept-element-custom-element-state) is neither "<code>custom</code>" nor "<code>precustomized</code>" and its <a id="ref-for-concept-element-custom-element-definition①"></a>[custom element definition](#concept-element-custom-element-definition) is null.

                <a id="example-return-override"></a>
                <strong>Example:</strong>

                [](#example-return-override)
                ``` text
                customElements.define("x-example", class extends HTMLElement {
                  constructor() {
                    // Non-conformant: returns an existing element instead of calling super().
                    return document.createElement("p");
                  }
                });

                document.createElement("x-example");
                ```

                Here <var>result</var> is the <code>p</code> <a id="ref-for-concept-element⑦⑧"></a>[element](#concept-element), whose <a id="ref-for-concept-element-custom-element-state⑥"></a>[custom element state](#concept-element-custom-element-state) is "<code>uncustomized</code>" and whose <a id="ref-for-concept-element-custom-element-definition②"></a>[custom element definition](#concept-element-custom-element-definition) is null.

            3.  Assert: <var>result</var>’s <a id="ref-for-concept-element-namespace①④"></a>[namespace](#concept-element-namespace) is the <a id="ref-for-html-namespace①⑨"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace).

                <strong>Note:</strong>

                IDL enforces that <var>result</var> is an <code><a id="ref-for-htmlelement"></a>[HTMLElement](https://html.spec.whatwg.org/multipage/dom.html#htmlelement)</code> object, which all use the <a id="ref-for-html-namespace②⓪"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace).

            4.  If <var>result</var>’s <a id="ref-for-concept-element-attribute⑥"></a>[attribute list](#concept-element-attribute) <a id="ref-for-list-is-empty⑥"></a>[is not empty](https://infra.spec.whatwg.org/#list-is-empty), then <a id="ref-for-dfn-throw④⑨"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror①③"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑥③"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

            5.  If <var>result</var> has <a id="ref-for-concept-tree-child⑤⑤"></a>[children](#concept-tree-child), then <a id="ref-for-dfn-throw⑤⓪"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror①④"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑥④"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

            6.  If <var>result</var>’s <a id="ref-for-concept-tree-parent③①"></a>[parent](#concept-tree-parent) is non-null, then <a id="ref-for-dfn-throw⑤①"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror①⑤"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑥⑤"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

            7.  If <var>result</var>’s <a id="ref-for-concept-node-document③⑧"></a>[node document](#concept-node-document) is not <var>document</var>, then <a id="ref-for-dfn-throw⑤②"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror①⑥"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑥⑥"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

            8.  If <var>result</var>’s <a id="ref-for-concept-element-local-name①②"></a>[local name](#concept-element-local-name) is not equal to <var>localName</var>, then <a id="ref-for-dfn-throw⑤③"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror①⑦"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑥⑦"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

            9.  Set <var>result</var>’s <a id="ref-for-concept-element-namespace-prefix①②"></a>[namespace prefix](#concept-element-namespace-prefix) to <var>prefix</var>.

            10. Set <var>result</var>’s <a id="ref-for-concept-element-is-value①"></a>[<code>is</code> value](#concept-element-is-value) to null.

            11. Set <var>result</var>’s <a id="ref-for-element-custom-element-registry⑧"></a>[custom element registry](#element-custom-element-registry) to <var>registry</var>.

            If any of these steps threw an exception <var>exception</var>:

            1.  <a id="ref-for-report-an-exception②"></a>[Report](https://html.spec.whatwg.org/multipage/webappapis.html#report-an-exception) <var>exception</var> for <var>definition</var>’s <a id="ref-for-concept-custom-element-definition-constructor②"></a>[constructor](https://html.spec.whatwg.org/multipage/custom-elements.html#concept-custom-element-definition-constructor)’s corresponding JavaScript object’s <a id="ref-for-dfn-associated-realm③"></a>[associated realm](https://webidl.spec.whatwg.org/#dfn-associated-realm)’s <a id="ref-for-concept-realm-global③"></a>[global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-realm-global).

            2.  Set <var>result</var> to the result of <a id="ref-for-create-an-element-internal①"></a>[creating an element internal](#create-an-element-internal) given <var>document</var>, <code><a id="ref-for-htmlunknownelement"></a>[HTMLUnknownElement](https://html.spec.whatwg.org/multipage/dom.html#htmlunknownelement)</code>, <var>localName</var>, the <a id="ref-for-html-namespace②①"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace), <var>prefix</var>, "<code>failed</code>", null, and <var>registry</var>.

        5.  If <var>previousRegistry</var> is null, then <a id="ref-for-map-remove"></a>[remove](https://infra.spec.whatwg.org/#map-remove) the <a id="ref-for-surrounding-agent①①"></a>[surrounding agent](https://tc39.es/ecma262/#surrounding-agent)’s <a id="ref-for-active-custom-element-constructor-map②"></a>[active custom element constructor map](https://html.spec.whatwg.org/multipage/custom-elements.html#active-custom-element-constructor-map)\[<var>C</var>\].

        6.  Otherwise, <a id="ref-for-map-set③"></a>[set](https://infra.spec.whatwg.org/#map-set) the <a id="ref-for-surrounding-agent①②"></a>[surrounding agent](https://tc39.es/ecma262/#surrounding-agent)’s <a id="ref-for-active-custom-element-constructor-map③"></a>[active custom element constructor map](https://html.spec.whatwg.org/multipage/custom-elements.html#active-custom-element-constructor-map)\[<var>C</var>\] to <var>previousRegistry</var>.

    2.  Otherwise:

        1.  Set <var>result</var> to the result of <a id="ref-for-create-an-element-internal②"></a>[creating an element internal](#create-an-element-internal) given <var>document</var>, <code><a id="ref-for-htmlelement①"></a>[HTMLElement](https://html.spec.whatwg.org/multipage/dom.html#htmlelement)</code>, <var>localName</var>, the <a id="ref-for-html-namespace②②"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace), <var>prefix</var>, "<code>undefined</code>", null, and <var>registry</var>.

        2.  <a id="ref-for-enqueue-a-custom-element-upgrade-reaction①"></a>[Enqueue a custom element upgrade reaction](https://html.spec.whatwg.org/multipage/custom-elements.html#enqueue-a-custom-element-upgrade-reaction) given <var>result</var> and <var>definition</var>.

6.  Otherwise:

    1.  Let <var>interface</var> be the <a id="ref-for-concept-element-interface①"></a>[element interface](#concept-element-interface) for <var>localName</var> and <var>namespace</var>.

    2.  Set <var>result</var> to the result of <a id="ref-for-create-an-element-internal③"></a>[creating an element internal](#create-an-element-internal) given <var>document</var>, <var>interface</var>, <var>localName</var>, <var>namespace</var>, <var>prefix</var>, "<code>uncustomized</code>", <var>is</var>, and <var>registry</var>.

    3.  If <var>namespace</var> is the <a id="ref-for-html-namespace②③"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace), and either <var>localName</var> is a <a id="ref-for-valid-custom-element-name①"></a>[valid custom element name](https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name) or <var>is</var> is non-null, then set <var>result</var>’s <a id="ref-for-concept-element-custom-element-state⑦"></a>[custom element state](#concept-element-custom-element-state) to "<code>undefined</code>".

7.  Return <var>result</var>.

To <a id="create-an-element-internal"></a><strong>create an element internal</strong> given a <a id="ref-for-concept-document⑤⑤"></a>[document](#concept-document) <var>document</var>, an interface <var>interface</var> a string <var>localName</var>, a string-or-null <var>namespace</var>, a string-or-null <var>prefix</var>, a string <var>state</var>, a string-or-null <var>is</var>, and null or a <code><a id="ref-for-customelementregistry①⑦"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code> object <var>registry</var>:

1.  Let <var>element</var> be the result of <a id="ref-for-create-a-node③"></a>[creating a node](#create-a-node) that implements <var>interface</var>, given <var>document</var>.

2.  Set <var>element</var>’s <a id="ref-for-concept-element-namespace①⑤"></a>[namespace](#concept-element-namespace) to <var>namespace</var>, <a id="ref-for-concept-element-namespace-prefix①③"></a>[namespace prefix](#concept-element-namespace-prefix) to <var>prefix</var>, <a id="ref-for-concept-element-local-name①③"></a>[local name](#concept-element-local-name) to <var>localName</var>, <a id="ref-for-element-custom-element-registry⑨"></a>[custom element registry](#element-custom-element-registry) to <var>registry</var>, <a id="ref-for-concept-element-custom-element-state⑧"></a>[custom element state](#concept-element-custom-element-state) to <var>state</var>, <a id="ref-for-concept-element-custom-element-definition③"></a>[custom element definition](#concept-element-custom-element-definition) to null, and <a id="ref-for-concept-element-is-value②"></a>[<code>is</code> value](#concept-element-is-value) to <var>is</var>.

3.  <a id="ref-for-assert①②"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>element</var>’s <a id="ref-for-concept-element-attribute⑦"></a>[attribute list](#concept-element-attribute) <a id="ref-for-list-is-empty⑦"></a>[is empty](https://infra.spec.whatwg.org/#list-is-empty).

4.  Return <var>element</var>.

<a id="ref-for-concept-element⑦⑨"></a>[Elements](#concept-element) also have an <a id="concept-element-attribute"></a><strong>attribute list</strong>, which is a <a id="ref-for-list①⑥"></a>[list](https://infra.spec.whatwg.org/#list) exposed through a <code><a id="ref-for-namednodemap①"></a>[NamedNodeMap](#namednodemap)</code>. Unless explicitly given when an <a id="ref-for-concept-element⑧⓪"></a>[element](#concept-element) is created, its <a id="ref-for-concept-element-attribute⑧"></a>[attribute list](#concept-element-attribute) <a id="ref-for-list-is-empty⑧"></a>[is empty](https://infra.spec.whatwg.org/#list-is-empty).

An <a id="ref-for-concept-element⑧①"></a>[element](#concept-element) <a id="concept-element-attribute-has"></a><strong>has</strong> an <a id="ref-for-concept-attribute②①"></a>[attribute](#concept-attribute) <var>A</var> if its <a id="ref-for-concept-element-attribute⑨"></a>[attribute list](#concept-element-attribute) <a id="ref-for-list-contain①⓪"></a>[contains](https://infra.spec.whatwg.org/#list-contain) <var>A</var>.

This and <a id="ref-for-other-applicable-specifications⑨"></a>[other specifications](https://dom.spec.whatwg.org/#other-applicable-specifications) may define <a id="concept-element-attributes-change-ext"></a><strong>attribute change steps</strong> for <a id="ref-for-concept-element⑧②"></a>[elements](#concept-element). The algorithm is passed <var>element</var>, <var>localName</var>, <var>oldValue</var>, <var>value</var>, and <var>namespace</var>.

To <a id="handle-attribute-changes"></a><strong>handle attribute changes</strong> for an <a id="ref-for-concept-attribute②②"></a>[attribute](#concept-attribute) <var>attribute</var> with <var>element</var>, <var>oldValue</var>, and <var>newValue</var>, run these steps:

1.  <a id="ref-for-queue-a-mutation-record①"></a>[Queue a mutation record](https://dom.spec.whatwg.org/#queue-a-mutation-record) of "<code>attributes</code>" for <var>element</var> with <var>attribute</var>’s <a id="ref-for-concept-attribute-local-name⑦"></a>[local name](#concept-attribute-local-name), <var>attribute</var>’s <a id="ref-for-concept-attribute-namespace⑥"></a>[namespace](#concept-attribute-namespace), <var>oldValue</var>, « », « », null, and null.

2.  If <var>element</var> is <a id="ref-for-concept-element-custom⑦"></a>[custom](#concept-element-custom), then <a id="ref-for-enqueue-a-custom-element-callback-reaction⑤"></a>[enqueue a custom element callback reaction](https://html.spec.whatwg.org/multipage/custom-elements.html#enqueue-a-custom-element-callback-reaction) with <var>element</var>, callback name "<code>attributeChangedCallback</code>", and « <var>attribute</var>’s <a id="ref-for-concept-attribute-local-name⑧"></a>[local name](#concept-attribute-local-name), <var>oldValue</var>, <var>newValue</var>, <var>attribute</var>’s <a id="ref-for-concept-attribute-namespace⑦"></a>[namespace](#concept-attribute-namespace) ».

3.  Run the <a id="ref-for-concept-element-attributes-change-ext②"></a>[attribute change steps](#concept-element-attributes-change-ext) with <var>element</var>, <var>attribute</var>’s <a id="ref-for-concept-attribute-local-name⑨"></a>[local name](#concept-attribute-local-name), <var>oldValue</var>, <var>newValue</var>, and <var>attribute</var>’s <a id="ref-for-concept-attribute-namespace⑧"></a>[namespace](#concept-attribute-namespace).

To <a id="concept-element-attributes-change"></a><strong>change</strong> an <a id="ref-for-concept-attribute②③"></a>[attribute](#concept-attribute) <var>attribute</var> to <var>value</var>:

1.  Let <var>oldValue</var> be <var>attribute</var>’s <a id="ref-for-concept-attribute-value①①"></a>[value](#concept-attribute-value).

2.  Set <var>attribute</var>’s <a id="ref-for-concept-attribute-value①②"></a>[value](#concept-attribute-value) to <var>value</var>.

3.  <a id="ref-for-handle-attribute-changes"></a>[Handle attribute changes](#handle-attribute-changes) for <var>attribute</var> with <var>attribute</var>’s <a id="ref-for-concept-attribute-element⑥"></a>[element](#concept-attribute-element), <var>oldValue</var>, and <var>value</var>.

To <a id="concept-element-attributes-append"></a><strong>append</strong> an <a id="ref-for-concept-attribute②④"></a>[attribute](#concept-attribute) <var>attribute</var> to an <a id="ref-for-concept-element⑧③"></a>[element](#concept-element) <var>element</var>:

1.  <a id="ref-for-list-append①④"></a>[Append](https://infra.spec.whatwg.org/#list-append) <var>attribute</var> to <var>element</var>’s <a id="ref-for-concept-element-attribute①⓪"></a>[attribute list](#concept-element-attribute).

2.  Set <var>attribute</var>’s <a id="ref-for-concept-attribute-element⑦"></a>[element](#concept-attribute-element) to <var>element</var>.

3.  Set <var>attribute</var>’s <a id="ref-for-concept-node-document③⑨"></a>[node document](#concept-node-document) to <var>element</var>’s <a id="ref-for-concept-node-document④⓪"></a>[node document](#concept-node-document).

4.  <a id="ref-for-handle-attribute-changes①"></a>[Handle attribute changes](#handle-attribute-changes) for <var>attribute</var> with <var>element</var>, null, and <var>attribute</var>’s <a id="ref-for-concept-attribute-value①③"></a>[value](#concept-attribute-value).

To <a id="concept-element-attributes-remove"></a><strong>remove</strong> an <a id="ref-for-concept-attribute②⑤"></a>[attribute](#concept-attribute) <var>attribute</var>:

1.  Let <var>element</var> be <var>attribute</var>’s <a id="ref-for-concept-attribute-element⑧"></a>[element](#concept-attribute-element).

2.  <a id="ref-for-list-remove⑦"></a>[Remove](https://infra.spec.whatwg.org/#list-remove) <var>attribute</var> from <var>element</var>’s <a id="ref-for-concept-element-attribute①①"></a>[attribute list](#concept-element-attribute).

3.  Set <var>attribute</var>’s <a id="ref-for-concept-attribute-element⑨"></a>[element](#concept-attribute-element) to null.

4.  <a id="ref-for-handle-attribute-changes②"></a>[Handle attribute changes](#handle-attribute-changes) for <var>attribute</var> with <var>element</var>, <var>attribute</var>’s <a id="ref-for-concept-attribute-value①④"></a>[value](#concept-attribute-value), and null.

To <a id="concept-element-attributes-replace"></a><strong>replace</strong> an <a id="ref-for-concept-attribute②⑥"></a>[attribute](#concept-attribute) <var>oldAttribute</var> with an <a id="ref-for-concept-attribute②⑦"></a>[attribute](#concept-attribute) <var>newAttribute</var>:

1.  Let <var>element</var> be <var>oldAttribute</var>’s <a id="ref-for-concept-attribute-element①⓪"></a>[element](#concept-attribute-element).

2.  <a id="ref-for-list-replace"></a>[Replace](https://infra.spec.whatwg.org/#list-replace) <var>oldAttribute</var> by <var>newAttribute</var> in <var>element</var>’s <a id="ref-for-concept-element-attribute①②"></a>[attribute list](#concept-element-attribute).

3.  Set <var>newAttribute</var>’s <a id="ref-for-concept-attribute-element①①"></a>[element](#concept-attribute-element) to <var>element</var>.

4.  Set <var>newAttribute</var>’s <a id="ref-for-concept-node-document④①"></a>[node document](#concept-node-document) to <var>element</var>’s <a id="ref-for-concept-node-document④②"></a>[node document](#concept-node-document).

5.  Set <var>oldAttribute</var>’s <a id="ref-for-concept-attribute-element①②"></a>[element](#concept-attribute-element) to null.

6.  <a id="ref-for-handle-attribute-changes③"></a>[Handle attribute changes](#handle-attribute-changes) for <var>oldAttribute</var> with <var>element</var>, <var>oldAttribute</var>’s <a id="ref-for-concept-attribute-value①⑤"></a>[value](#concept-attribute-value), and <var>newAttribute</var>’s <a id="ref-for-concept-attribute-value①⑥"></a>[value](#concept-attribute-value).

------------------------------------------------------------------------

To <a id="concept-element-attributes-get-by-name"></a><strong>get an attribute by name</strong> given a string <var>qualifiedName</var> and an <a id="ref-for-concept-element⑧④"></a>[element](#concept-element) <var>element</var>:

1.  If <var>element</var> is in the <a id="ref-for-html-namespace②④"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) and its <a id="ref-for-concept-node-document④③"></a>[node document](#concept-node-document) is an <a id="ref-for-html-document①①"></a>[HTML document](#html-document), then set <var>qualifiedName</var> to <var>qualifiedName</var> in <a id="ref-for-ascii-lowercase③"></a>[ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).

2.  Return the first <a id="ref-for-concept-attribute②⑧"></a>[attribute](#concept-attribute) in <var>element</var>’s <a id="ref-for-concept-element-attribute①③"></a>[attribute list](#concept-element-attribute) whose <a id="ref-for-concept-attribute-qualified-name②"></a>[qualified name](#concept-attribute-qualified-name) is <var>qualifiedName</var>; otherwise null.

To <a id="concept-element-attributes-get-by-namespace"></a><strong>get an attribute by namespace and local name</strong> given null or a string <var>namespace</var>, a string <var>localName</var>, and an <a id="ref-for-concept-element⑧⑤"></a>[element](#concept-element) <var>element</var>:

1.  If <var>namespace</var> is the empty string, then set it to null.

2.  Return the <a id="ref-for-concept-attribute②⑨"></a>[attribute](#concept-attribute) in <var>element</var>’s <a id="ref-for-concept-element-attribute①④"></a>[attribute list](#concept-element-attribute) whose <a id="ref-for-concept-attribute-namespace⑨"></a>[namespace](#concept-attribute-namespace) is <var>namespace</var> and <a id="ref-for-concept-attribute-local-name①⓪"></a>[local name](#concept-attribute-local-name) is <var>localName</var>, if any; otherwise null.

To <a id="concept-element-attributes-get-value"></a><strong>get an attribute value</strong> given an <a id="ref-for-concept-element⑧⑥"></a>[element](#concept-element) <var>element</var>, a string <var>localName</var>, and an optional null or string <var>namespace</var> (default null):

1.  Let <var>attr</var> be the result of <a id="ref-for-concept-element-attributes-get-by-namespace"></a>[getting an attribute](#concept-element-attributes-get-by-namespace) given <var>namespace</var>, <var>localName</var>, and <var>element</var>.

2.  If <var>attr</var> is null, then return the empty string.

3.  Return <var>attr</var>’s <a id="ref-for-concept-attribute-value①⑦"></a>[value](#concept-attribute-value).

To <a id="concept-element-attributes-set"></a><strong>set an attribute</strong> given an <a id="ref-for-concept-attribute③⓪"></a>[attribute](#concept-attribute) <var>attr</var> and an <a id="ref-for-concept-element⑧⑦"></a>[element](#concept-element) <var>element</var>:

1.  Let <var>verifiedValue</var> be the result of calling <a id="ref-for-get-trusted-type-compliant-attribute-value"></a>[get trusted type compliant attribute value](https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-attribute-value) with <var>attr</var>’s <a id="ref-for-concept-attribute-local-name①①"></a>[local name](#concept-attribute-local-name), <var>attr</var>’s <a id="ref-for-concept-attribute-namespace①⓪"></a>[namespace](#concept-attribute-namespace), <var>element</var>, and <var>attr</var>’s <a id="ref-for-concept-attribute-value①⑧"></a>[value](#concept-attribute-value). [\[TRUSTED-TYPES\]](https://dom.spec.whatwg.org/#biblio-trusted-types)

2.  If <var>attr</var>’s <a id="ref-for-concept-attribute-element①③"></a>[element](#concept-attribute-element) is neither null nor <var>element</var>, <a id="ref-for-dfn-throw⑤④"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-inuseattributeerror"></a>[InUseAttributeError](https://webidl.spec.whatwg.org/#inuseattributeerror)</code>" <code><a id="ref-for-idl-DOMException⑥⑧"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  Let <var>oldAttr</var> be the result of <a id="ref-for-concept-element-attributes-get-by-namespace①"></a>[getting an attribute](#concept-element-attributes-get-by-namespace) given <var>attr</var>’s <a id="ref-for-concept-attribute-namespace①①"></a>[namespace](#concept-attribute-namespace), <var>attr</var>’s <a id="ref-for-concept-attribute-local-name①②"></a>[local name](#concept-attribute-local-name), and <var>element</var>.

4.  If <var>oldAttr</var> is <var>attr</var>, return <var>attr</var>.

5.  Set <var>attr</var>’s <a id="ref-for-concept-attribute-value①⑨"></a>[value](#concept-attribute-value) to <var>verifiedValue</var>.

6.  If <var>oldAttr</var> is non-null, then <a id="ref-for-concept-element-attributes-replace"></a>[replace](#concept-element-attributes-replace) <var>oldAttr</var> with <var>attr</var>.

7.  Otherwise, <a id="ref-for-concept-element-attributes-append①"></a>[append](#concept-element-attributes-append) <var>attr</var> to <var>element</var>.

8.  Return <var>oldAttr</var>.

To <a id="concept-element-attributes-set-value"></a><strong>set an attribute value</strong> given an <a id="ref-for-concept-element⑧⑧"></a>[element](#concept-element) <var>element</var>, a string <var>localName</var>, a string <var>value</var>, an optional null or string <var>prefix</var> (default null), and an optional null or string <var>namespace</var> (default null):

1.  Let <var>attribute</var> be the result of <a id="ref-for-concept-element-attributes-get-by-namespace②"></a>[getting an attribute](#concept-element-attributes-get-by-namespace) given <var>namespace</var>, <var>localName</var>, and <var>element</var>.

2.  If <var>attribute</var> is null, then <a id="ref-for-concept-element-attributes-append②"></a>[append](#concept-element-attributes-append) the result of <a id="ref-for-create-an-attribute②"></a>[creating an attribute](#create-an-attribute) given <var>element</var>’s <a id="ref-for-concept-node-document④④"></a>[node document](#concept-node-document), <var>localName</var>, <var>namespace</var>, <var>prefix</var>, and <var>value</var> to <var>element</var>, and then return.

3.  <a id="ref-for-concept-element-attributes-change"></a>[Change](#concept-element-attributes-change) <var>attribute</var> to <var>value</var>.

To <a id="concept-element-attributes-remove-by-name"></a><strong>remove an attribute by name</strong> given a string <var>qualifiedName</var> and an <a id="ref-for-concept-element⑧⑨"></a>[element](#concept-element) <var>element</var>:

1.  Let <var>attr</var> be the result of <a id="ref-for-concept-element-attributes-get-by-name"></a>[getting an attribute](#concept-element-attributes-get-by-name) given <var>qualifiedName</var> and <var>element</var>.

2.  If <var>attr</var> is non-null, then <a id="ref-for-concept-element-attributes-remove"></a>[remove](#concept-element-attributes-remove) <var>attr</var>.

3.  Return <var>attr</var>.

To <a id="concept-element-attributes-remove-by-namespace"></a><strong>remove an attribute by namespace and local name</strong> given null or a string <var>namespace</var>, a string <var>localName</var>, and an <a id="ref-for-concept-element⑨⓪"></a>[element](#concept-element) <var>element</var>:

1.  Let <var>attr</var> be the result of <a id="ref-for-concept-element-attributes-get-by-namespace③"></a>[getting an attribute](#concept-element-attributes-get-by-namespace) given <var>namespace</var>, <var>localName</var>, and <var>element</var>.

2.  If <var>attr</var> is non-null, then <a id="ref-for-concept-element-attributes-remove①"></a>[remove](#concept-element-attributes-remove) <var>attr</var>.

3.  Return <var>attr</var>.

------------------------------------------------------------------------

An <a id="ref-for-concept-element⑨①"></a>[element](#concept-element) can have an associated <a id="concept-id"></a><strong>unique identifier (ID)</strong>

<strong>Note:</strong>

Historically <a id="ref-for-concept-element⑨②"></a>[elements](#concept-element) could have multiple identifiers e.g., by using the HTML <code>id</code> <a id="ref-for-concept-attribute③①"></a>[attribute](#concept-attribute) and a DTD. This specification makes <a id="ref-for-concept-id⑥"></a>[ID](#concept-id) a concept of the DOM and allows for only one per <a id="ref-for-concept-element⑨③"></a>[element](#concept-element), given by an <a id="ref-for-concept-named-attribute③"></a>[<code>id</code> attribute](#concept-named-attribute).

Use these <a id="ref-for-concept-element-attributes-change-ext③"></a>[attribute change steps](#concept-element-attributes-change-ext) to update an <a id="ref-for-concept-element⑨④"></a>[element](#concept-element)’s <a id="ref-for-concept-id⑦"></a>[ID](#concept-id):

1.  If <var>localName</var> is <code>id</code>, <var>namespace</var> is null, and <var>value</var> is null or the empty string, then unset <var>element</var>’s <a id="ref-for-concept-id⑧"></a>[ID](#concept-id).

2.  Otherwise, if <var>localName</var> is <code>id</code>, <var>namespace</var> is null, then set <var>element</var>’s <a id="ref-for-concept-id⑨"></a>[ID](#concept-id) to <var>value</var>.

<strong>Note:</strong>

While this specification defines requirements for <code>class</code>, <code>id</code>, and <code>slot</code> <a id="ref-for-concept-attribute③②"></a>[attributes](#concept-attribute) on any <a id="ref-for-concept-element⑨⑤"></a>[element](#concept-element), it makes no claims as to whether using them is conforming or not.

------------------------------------------------------------------------

A <a id="ref-for-concept-node①⑥⑦"></a>[node](#concept-node)’s <a id="ref-for-concept-tree-parent③②"></a>[parent](#concept-tree-parent) of type <code><a id="ref-for-element④⑨"></a>[Element](#element)</code> is known as its <a id="parent-element"></a><strong>parent element</strong>. If the <a id="ref-for-concept-node①⑥⑧"></a>[node](#concept-node) has a <a id="ref-for-concept-tree-parent③③"></a>[parent](#concept-tree-parent) of a different type, its <a id="ref-for-parent-element⑨"></a>[parent element](#parent-element) is null.

------------------------------------------------------------------------

<strong><var>namespace</var> = <var>element</var> . <code><a id="ref-for-dom-element-namespaceuri①"></a>[namespaceURI](#dom-element-namespaceuri)</code></strong>

Returns the <a id="ref-for-concept-element-namespace①⑥"></a>[namespace](#concept-element-namespace).

<strong><var>prefix</var> = <var>element</var> . <code><a id="ref-for-dom-element-prefix①"></a>[prefix](#dom-element-prefix)</code></strong>

Returns the <a id="ref-for-concept-element-namespace-prefix①④"></a>[namespace prefix](#concept-element-namespace-prefix).

<strong><var>localName</var> = <var>element</var> . <code><a id="ref-for-dom-element-localname①"></a>[localName](#dom-element-localname)</code></strong>

Returns the <a id="ref-for-concept-element-local-name①④"></a>[local name](#concept-element-local-name).

<strong><var>qualifiedName</var> = <var>element</var> . <code><a id="ref-for-dom-element-tagname①"></a>[tagName](#dom-element-tagname)</code></strong>

Returns the <a id="ref-for-element-html-uppercased-qualified-name③"></a>[HTML-uppercased qualified name](#element-html-uppercased-qualified-name).

The <a id="dom-element-namespaceuri"></a><strong><code>namespaceURI</code></strong> getter steps are to return <a id="ref-for-this①⑧①"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-element-namespace①⑦"></a>[namespace](#concept-element-namespace).

The <a id="dom-element-prefix"></a><strong><code>prefix</code></strong> getter steps are to return <a id="ref-for-this①⑧②"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-element-namespace-prefix①⑤"></a>[namespace prefix](#concept-element-namespace-prefix).

The <a id="dom-element-localname"></a><strong><code>localName</code></strong> getter steps are to return <a id="ref-for-this①⑧③"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-element-local-name①⑤"></a>[local name](#concept-element-local-name).

The <a id="dom-element-tagname"></a><strong><code>tagName</code></strong> getter steps are to return <a id="ref-for-this①⑧④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-element-html-uppercased-qualified-name④"></a>[HTML-uppercased qualified name](#element-html-uppercased-qualified-name).

------------------------------------------------------------------------

<strong><code><var>element</var> . <a id="ref-for-dom-element-id①"></a>[id](#dom-element-id) \[ = <var>value</var> \]</code></strong>

Returns the value of <var>element</var>’s <code>id</code> content attribute. Can be set to change it.

<strong><code><var>element</var> . <a id="ref-for-dom-element-classname①"></a>[className](#dom-element-classname) \[ = <var>value</var> \]</code></strong>

Returns the value of <var>element</var>’s <code>class</code> content attribute. Can be set to change it.

<strong><code><var>element</var> . <a id="ref-for-dom-element-classlist①"></a>[classList](#dom-element-classlist)</code></strong>

Allows for manipulation of <var>element</var>’s <code>class</code> content attribute as a set of whitespace-separated tokens through a <code><a id="ref-for-domtokenlist①"></a>[DOMTokenList](https://dom.spec.whatwg.org/#domtokenlist)</code> object.

<strong><code><var>element</var> . <a id="ref-for-dom-element-slot①"></a>[slot](#dom-element-slot) \[ = <var>value</var> \]</code></strong>

Returns the value of <var>element</var>’s <code>slot</code> content attribute. Can be set to change it.

IDL attributes that are defined to <a id="concept-reflect"></a><strong>reflect</strong> a string <var>name</var>, must have these getter and setter steps:

<strong>getter steps</strong>

Return the result of running <a id="ref-for-concept-element-attributes-get-value"></a>[get an attribute value](#concept-element-attributes-get-value) given <a id="ref-for-this①⑧⑤"></a>[this](https://webidl.spec.whatwg.org/#this) and <var>name</var>.

<strong>setter steps</strong>

<a id="ref-for-concept-element-attributes-set-value"></a>[Set an attribute value](#concept-element-attributes-set-value) for <a id="ref-for-this①⑧⑥"></a>[this](https://webidl.spec.whatwg.org/#this) using <var>name</var> and the given value.

The <a id="dom-element-id"></a><strong><code>id</code></strong> attribute must <a id="ref-for-concept-reflect"></a>[reflect](#concept-reflect) "<code>id</code>".

The <a id="dom-element-classname"></a><strong><code>className</code></strong> attribute must <a id="ref-for-concept-reflect①"></a>[reflect](#concept-reflect) "<code>class</code>".

The <a id="dom-element-classlist"></a><strong><code>classList</code></strong> getter steps are to return a <code><a id="ref-for-domtokenlist②"></a>[DOMTokenList](https://dom.spec.whatwg.org/#domtokenlist)</code> object whose associated <a id="ref-for-concept-element⑨⑥"></a>[element](#concept-element) is <a id="ref-for-this①⑧⑦"></a>[this](https://webidl.spec.whatwg.org/#this) and whose associated <a id="ref-for-concept-attribute③③"></a>[attribute](#concept-attribute)’s <a id="ref-for-concept-attribute-local-name①③"></a>[local name](#concept-attribute-local-name) is <code>class</code>. The <a id="ref-for-concept-dtl-tokens"></a>[token set](https://dom.spec.whatwg.org/#concept-dtl-tokens) of this particular <code><a id="ref-for-domtokenlist③"></a>[DOMTokenList](https://dom.spec.whatwg.org/#domtokenlist)</code> object are also known as the <a id="ref-for-concept-element⑨⑦"></a>[element](#concept-element)’s <a id="concept-class"></a><strong>classes</strong>.

The <a id="dom-element-slot"></a><strong><code>slot</code></strong> attribute must <a id="ref-for-concept-reflect②"></a>[reflect](#concept-reflect) "<code>slot</code>".

<strong>Note:</strong>

<code>id</code>, <code>class</code>, and <code>slot</code> are effectively superglobal attributes as they can appear on any element, regardless of that element’s namespace.

------------------------------------------------------------------------

<strong><code><var>element</var> . <a id="ref-for-dom-element-hasattributes①"></a>[hasAttributes](#dom-element-hasattributes)()</code></strong>

Returns true if <var>element</var> has attributes; otherwise false.

<strong><code><var>element</var> . <a id="ref-for-dom-element-getattributenames①"></a>[getAttributeNames](#dom-element-getattributenames)()</code></strong>

Returns the <a id="ref-for-concept-attribute-qualified-name③"></a>[qualified names](#concept-attribute-qualified-name) of all <var>element</var>’s <a id="ref-for-concept-attribute③④"></a>[attributes](#concept-attribute). Can contain duplicates.

<strong><code><var>element</var> . <a id="ref-for-dom-element-getattribute①"></a>[getAttribute](#dom-element-getattribute)(<var>qualifiedName</var>)</code></strong>

Returns <var>element</var>’s first <a id="ref-for-concept-attribute③⑤"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-qualified-name④"></a>[qualified name](#concept-attribute-qualified-name) is <var>qualifiedName</var>, and null if there is no such <a id="ref-for-concept-attribute③⑥"></a>[attribute](#concept-attribute) otherwise.

<strong><code><var>element</var> . <a id="ref-for-dom-element-getattributens①"></a>[getAttributeNS](#dom-element-getattributens)(<var>namespace</var>, <var>localName</var>)</code></strong>

Returns <var>element</var>’s <a id="ref-for-concept-attribute③⑦"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-namespace①②"></a>[namespace](#concept-attribute-namespace) is <var>namespace</var> and <a id="ref-for-concept-attribute-local-name①④"></a>[local name](#concept-attribute-local-name) is <var>localName</var>, and null if there is no such <a id="ref-for-concept-attribute③⑧"></a>[attribute](#concept-attribute) otherwise.

<strong><code><var>element</var> . <a id="ref-for-dom-element-setattribute①"></a>[setAttribute](#dom-element-setattribute)(<var>qualifiedName</var>, <var>value</var>)</code></strong>

Sets the <a id="ref-for-concept-attribute-value②⓪"></a>[value](#concept-attribute-value) of <var>element</var>’s first <a id="ref-for-concept-attribute③⑨"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-qualified-name⑤"></a>[qualified name](#concept-attribute-qualified-name) is <var>qualifiedName</var> to <var>value</var>.

<strong><code><var>element</var> . <a id="ref-for-dom-element-setattributens①"></a>[setAttributeNS](#dom-element-setattributens)(<var>namespace</var>, <var>localName</var>, <var>value</var>)</code></strong>

Sets the <a id="ref-for-concept-attribute-value②①"></a>[value](#concept-attribute-value) of <var>element</var>’s <a id="ref-for-concept-attribute④⓪"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-namespace①③"></a>[namespace](#concept-attribute-namespace) is <var>namespace</var> and <a id="ref-for-concept-attribute-local-name①⑤"></a>[local name](#concept-attribute-local-name) is <var>localName</var> to <var>value</var>.

<strong><code><var>element</var> . <a id="ref-for-dom-element-removeattribute①"></a>[removeAttribute](#dom-element-removeattribute)(<var>qualifiedName</var>)</code></strong>

Removes <var>element</var>’s first <a id="ref-for-concept-attribute④①"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-qualified-name⑥"></a>[qualified name](#concept-attribute-qualified-name) is <var>qualifiedName</var>.

<strong><code><var>element</var> . <a id="ref-for-dom-element-removeattributens①"></a>[removeAttributeNS](#dom-element-removeattributens)(<var>namespace</var>, <var>localName</var>)</code></strong>

Removes <var>element</var>’s <a id="ref-for-concept-attribute④②"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-namespace①④"></a>[namespace](#concept-attribute-namespace) is <var>namespace</var> and <a id="ref-for-concept-attribute-local-name①⑥"></a>[local name](#concept-attribute-local-name) is <var>localName</var>.

<strong><code><var>element</var> . <a id="ref-for-dom-element-toggleattribute①"></a>[toggleAttribute](#dom-element-toggleattribute)(<var>qualifiedName</var> \[, <var>force</var>\])</code></strong>

If <var>force</var> is not given, "toggles" <var>qualifiedName</var>, removing it if it is present and adding it if it is not present. If <var>force</var> is true, adds <var>qualifiedName</var>. If <var>force</var> is false, removes <var>qualifiedName</var>.

Returns true if <var>qualifiedName</var> is now present; otherwise false.

<strong><code><var>element</var> . <a id="ref-for-dom-element-hasattribute①"></a>[hasAttribute](#dom-element-hasattribute)(<var>qualifiedName</var>)</code></strong>

Returns true if <var>element</var> has an <a id="ref-for-concept-attribute④③"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-qualified-name⑦"></a>[qualified name](#concept-attribute-qualified-name) is <var>qualifiedName</var>; otherwise false.

<strong><code><var>element</var> . <a id="ref-for-dom-element-hasattributens①"></a>[hasAttributeNS](#dom-element-hasattributens)(<var>namespace</var>, <var>localName</var>)</code></strong>

Returns true if <var>element</var> has an <a id="ref-for-concept-attribute④④"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-namespace①⑤"></a>[namespace](#concept-attribute-namespace) is <var>namespace</var> and <a id="ref-for-concept-attribute-local-name①⑦"></a>[local name](#concept-attribute-local-name) is <var>localName</var>.

The <a id="dom-element-hasattributes"></a><strong><code>hasAttributes()</code></strong> method steps are to return false if <a id="ref-for-this①⑧⑧"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-element-attribute①⑤"></a>[attribute list](#concept-element-attribute) <a id="ref-for-list-is-empty⑨"></a>[is empty](https://infra.spec.whatwg.org/#list-is-empty); otherwise true.

The <a id="dom-element-attributes"></a><strong><code>attributes</code></strong> getter steps are to return the associated <code><a id="ref-for-namednodemap②"></a>[NamedNodeMap](#namednodemap)</code>.

The <a id="dom-element-getattributenames"></a><strong><code>getAttributeNames()</code></strong> method steps are to return the <a id="ref-for-concept-attribute-qualified-name⑧"></a>[qualified names](#concept-attribute-qualified-name) of the <a id="ref-for-concept-attribute④⑤"></a>[attributes](#concept-attribute) in <a id="ref-for-this①⑧⑨"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-element-attribute①⑥"></a>[attribute list](#concept-element-attribute), in order; otherwise a new <a id="ref-for-list①⑦"></a>[list](https://infra.spec.whatwg.org/#list).

<strong>Note:</strong>

These are not guaranteed to be unique.

The <a id="dom-element-getattribute"></a><strong><code>getAttribute(<var>qualifiedName</var>)</code></strong> method steps are:

1.  Let <var>attr</var> be the result of <a id="ref-for-concept-element-attributes-get-by-name①"></a>[getting an attribute](#concept-element-attributes-get-by-name) given <var>qualifiedName</var> and <a id="ref-for-this①⑨⓪"></a>[this](https://webidl.spec.whatwg.org/#this).

2.  If <var>attr</var> is null, return null.

3.  Return <var>attr</var>’s <a id="ref-for-concept-attribute-value②②"></a>[value](#concept-attribute-value).

The <a id="dom-element-getattributens"></a><strong><code>getAttributeNS(<var>namespace</var>, <var>localName</var>)</code></strong> method steps are:

1.  Let <var>attr</var> be the result of <a id="ref-for-concept-element-attributes-get-by-namespace④"></a>[getting an attribute](#concept-element-attributes-get-by-namespace) given <var>namespace</var>, <var>localName</var>, and <a id="ref-for-this①⑨①"></a>[this](https://webidl.spec.whatwg.org/#this).

2.  If <var>attr</var> is null, return null.

3.  Return <var>attr</var>’s <a id="ref-for-concept-attribute-value②③"></a>[value](#concept-attribute-value).

The <a id="dom-element-setattribute"></a><strong><code>setAttribute(<var>qualifiedName</var>, <var>value</var>)</code></strong> method steps are:

1.  If <var>qualifiedName</var> is not a <a id="ref-for-valid-attribute-local-name②"></a>[valid attribute local name](#valid-attribute-local-name), then <a id="ref-for-dfn-throw⑤⑤"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-invalidcharactererror①②"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException⑥⑨"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

    <a id="node-setAttribute-qualifiedName"></a>

    <strong>Note:</strong>

    [](#node-setAttribute-qualifiedName)Despite the parameter naming, <var>qualifiedName</var> is only used as a <a id="ref-for-concept-attribute-qualified-name⑨"></a>[qualified name](#concept-attribute-qualified-name) if an <a id="ref-for-concept-attribute④⑥"></a>[attribute](#concept-attribute) already exists with that qualified name. Otherwise, it is used as the <a id="ref-for-concept-attribute-local-name①⑧"></a>[local name](#concept-attribute-local-name) of the new attribute. We only need to validate it for the latter case.

2.  If <a id="ref-for-this①⑨②"></a>[this](https://webidl.spec.whatwg.org/#this) is in the <a id="ref-for-html-namespace②⑤"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) and its <a id="ref-for-concept-node-document④⑤"></a>[node document](#concept-node-document) is an <a id="ref-for-html-document①②"></a>[HTML document](#html-document), then set <var>qualifiedName</var> to <var>qualifiedName</var> in <a id="ref-for-ascii-lowercase④"></a>[ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).

3.  Let <var>verifiedValue</var> be the result of calling <a id="ref-for-get-trusted-type-compliant-attribute-value①"></a>[get trusted type compliant attribute value](https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-attribute-value) with <var>qualifiedName</var>, null, <a id="ref-for-this①⑨③"></a>[this](https://webidl.spec.whatwg.org/#this), and <var>value</var>. [\[TRUSTED-TYPES\]](https://dom.spec.whatwg.org/#biblio-trusted-types)

4.  Let <var>attribute</var> be the first <a id="ref-for-concept-attribute④⑦"></a>[attribute](#concept-attribute) in <a id="ref-for-this①⑨④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-element-attribute①⑦"></a>[attribute list](#concept-element-attribute) whose <a id="ref-for-concept-attribute-qualified-name①⓪"></a>[qualified name](#concept-attribute-qualified-name) is <var>qualifiedName</var>, and null otherwise.

5.  If <var>attribute</var> is non-null, then <a id="ref-for-concept-element-attributes-change①"></a>[change](#concept-element-attributes-change) <var>attribute</var> to <var>verifiedValue</var> and return.

6.  Set <var>attribute</var> to the result of <a id="ref-for-create-an-attribute③"></a>[creating an attribute](#create-an-attribute) given <a id="ref-for-this①⑨⑤"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document④⑥"></a>[node document](#concept-node-document), <var>qualifiedName</var>, null, null, and <var>verifiedValue</var>.

7.  <a id="ref-for-concept-element-attributes-append③"></a>[Append](#concept-element-attributes-append) <var>attribute</var> to <a id="ref-for-this①⑨⑥"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-element-setattributens"></a><strong><code>setAttributeNS(<var>namespace</var>, <var>qualifiedName</var>, <var>value</var>)</code></strong> method steps are:

1.  Let (<var>namespace</var>, <var>prefix</var>, <var>localName</var>) be the result of <a id="ref-for-validate-and-extract②"></a>[validating and extracting](#validate-and-extract) <var>namespace</var> and <var>qualifiedName</var> given "<code>attribute</code>".

2.  Let <var>verifiedValue</var> be the result of calling <a id="ref-for-get-trusted-type-compliant-attribute-value②"></a>[get trusted type compliant attribute value](https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-attribute-value) with <var>localName</var>, <var>namespace</var>, <a id="ref-for-this①⑨⑦"></a>[this](https://webidl.spec.whatwg.org/#this), and <var>value</var>. [\[TRUSTED-TYPES\]](https://dom.spec.whatwg.org/#biblio-trusted-types)

3.  <a id="ref-for-concept-element-attributes-set-value①"></a>[Set an attribute value](#concept-element-attributes-set-value) for <a id="ref-for-this①⑨⑧"></a>[this](https://webidl.spec.whatwg.org/#this) using <var>localName</var>, <var>verifiedValue</var>, <var>prefix</var>, and <var>namespace</var>.

The <a id="dom-element-removeattribute"></a><strong><code>removeAttribute(<var>qualifiedName</var>)</code></strong> method steps are to <a id="ref-for-concept-element-attributes-remove-by-name"></a>[remove an attribute](#concept-element-attributes-remove-by-name) given <var>qualifiedName</var> and <a id="ref-for-this①⑨⑨"></a>[this](https://webidl.spec.whatwg.org/#this), and then return undefined.

The <a id="dom-element-removeattributens"></a><strong><code>removeAttributeNS(<var>namespace</var>, <var>localName</var>)</code></strong> method steps are to <a id="ref-for-concept-element-attributes-remove-by-namespace"></a>[remove an attribute](#concept-element-attributes-remove-by-namespace) given <var>namespace</var>, <var>localName</var>, and <a id="ref-for-this②⓪⓪"></a>[this](https://webidl.spec.whatwg.org/#this), and then return undefined.

The <a id="dom-element-hasattribute"></a><strong><code>hasAttribute(<var>qualifiedName</var>)</code></strong> method steps are:

1.  If <a id="ref-for-this②⓪①"></a>[this](https://webidl.spec.whatwg.org/#this) is in the <a id="ref-for-html-namespace②⑥"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) and its <a id="ref-for-concept-node-document④⑦"></a>[node document](#concept-node-document) is an <a id="ref-for-html-document①③"></a>[HTML document](#html-document), then set <var>qualifiedName</var> to <var>qualifiedName</var> in <a id="ref-for-ascii-lowercase⑤"></a>[ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).

2.  Return true if <a id="ref-for-this②⓪②"></a>[this](https://webidl.spec.whatwg.org/#this) <a id="ref-for-concept-element-attribute-has⑤"></a>[has](#concept-element-attribute-has) an <a id="ref-for-concept-attribute④⑧"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-qualified-name①①"></a>[qualified name](#concept-attribute-qualified-name) is <var>qualifiedName</var>; otherwise false.

The <a id="dom-element-toggleattribute"></a><strong><code>toggleAttribute(<var>qualifiedName</var>, <var>force</var>)</code></strong> method steps are:

1.  If <var>qualifiedName</var> is not a <a id="ref-for-valid-attribute-local-name③"></a>[valid attribute local name](#valid-attribute-local-name), then <a id="ref-for-dfn-throw⑤⑥"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-invalidcharactererror①③"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException⑦⓪"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

    <strong>Note:</strong>

    See [the discussion above](#node-setAttribute-qualifiedName) about why we validate it as a local name, instead of a qualified name.

2.  If <a id="ref-for-this②⓪③"></a>[this](https://webidl.spec.whatwg.org/#this) is in the <a id="ref-for-html-namespace②⑦"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) and its <a id="ref-for-concept-node-document④⑧"></a>[node document](#concept-node-document) is an <a id="ref-for-html-document①④"></a>[HTML document](#html-document), then set <var>qualifiedName</var> to <var>qualifiedName</var> in <a id="ref-for-ascii-lowercase⑥"></a>[ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).

3.  Let <var>attribute</var> be the first <a id="ref-for-concept-attribute④⑨"></a>[attribute](#concept-attribute) in <a id="ref-for-this②⓪④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-element-attribute①⑧"></a>[attribute list](#concept-element-attribute) whose <a id="ref-for-concept-attribute-qualified-name①②"></a>[qualified name](#concept-attribute-qualified-name) is <var>qualifiedName</var>, and null otherwise.

4.  If <var>attribute</var> is null:

    1.  If <var>force</var> is not given or is true, then <a id="ref-for-concept-element-attributes-append④"></a>[append](#concept-element-attributes-append) the result of <a id="ref-for-create-an-attribute④"></a>[creating an attribute](#create-an-attribute) given <a id="ref-for-this②⓪⑤"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document④⑨"></a>[node document](#concept-node-document) and <var>qualifiedName</var> to <a id="ref-for-this②⓪⑥"></a>[this](https://webidl.spec.whatwg.org/#this), and then return true.

    2.  Return false.

5.  If <var>force</var> is not given or is false, <a id="ref-for-concept-element-attributes-remove-by-name①"></a>[remove an attribute](#concept-element-attributes-remove-by-name) given <var>qualifiedName</var> and <a id="ref-for-this②⓪⑦"></a>[this](https://webidl.spec.whatwg.org/#this), and then return false.

6.  Return true.

The <a id="dom-element-hasattributens"></a><strong><code>hasAttributeNS(<var>namespace</var>, <var>localName</var>)</code></strong> method steps are:

1.  If <var>namespace</var> is the empty string, then set it to null.

2.  Return true if <a id="ref-for-this②⓪⑧"></a>[this](https://webidl.spec.whatwg.org/#this) <a id="ref-for-concept-element-attribute-has⑥"></a>[has](#concept-element-attribute-has) an <a id="ref-for-concept-attribute⑤⓪"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-namespace①⑥"></a>[namespace](#concept-attribute-namespace) is <var>namespace</var> and <a id="ref-for-concept-attribute-local-name①⑨"></a>[local name](#concept-attribute-local-name) is <var>localName</var>; otherwise false.

------------------------------------------------------------------------

The <a id="dom-element-getattributenode"></a><strong><code>getAttributeNode(<var>qualifiedName</var>)</code></strong> method steps are to return the result of <a id="ref-for-concept-element-attributes-get-by-name②"></a>[getting an attribute](#concept-element-attributes-get-by-name) given <var>qualifiedName</var> and <a id="ref-for-this②⓪⑨"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-element-getattributenodens"></a><strong><code>getAttributeNodeNS(<var>namespace</var>, <var>localName</var>)</code></strong> method steps are to return the result of <a id="ref-for-concept-element-attributes-get-by-namespace⑤"></a>[getting an attribute](#concept-element-attributes-get-by-namespace) given <var>namespace</var>, <var>localName</var>, and <a id="ref-for-this②①⓪"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-element-setattributenode"></a><strong><code>setAttributeNode(<var>attr</var>)</code></strong> and <a id="dom-element-setattributenodens"></a><strong><code>setAttributeNodeNS(<var>attr</var>)</code></strong> methods steps are to return the result of <a id="ref-for-concept-element-attributes-set"></a>[setting an attribute](#concept-element-attributes-set) given <var>attr</var> and <a id="ref-for-this②①①"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-element-removeattributenode"></a><strong><code>removeAttributeNode(<var>attr</var>)</code></strong> method steps are:

1.  If <a id="ref-for-this②①②"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-element-attribute①⑨"></a>[attribute list](#concept-element-attribute) does not <a id="ref-for-list-contain①①"></a>[contain](https://infra.spec.whatwg.org/#list-contain) <var>attr</var>, then <a id="ref-for-dfn-throw⑤⑦"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notfounderror③"></a>[NotFoundError](https://webidl.spec.whatwg.org/#notfounderror)</code>" <code><a id="ref-for-idl-DOMException⑦①"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  <a id="ref-for-concept-element-attributes-remove②"></a>[Remove](#concept-element-attributes-remove) <var>attr</var>.

3.  Return <var>attr</var>.

------------------------------------------------------------------------

<strong><code><var>shadow</var> = <var>element</var> . <code><a id="ref-for-dom-element-attachshadow②"></a>[attachShadow(init)](#dom-element-attachshadow)</code></code></strong>

Creates a <a id="ref-for-concept-shadow-root④⑦"></a>[shadow root](#concept-shadow-root) for <var>element</var> and returns it.

<strong><code><var>shadow</var> = <var>element</var> . <code><a id="ref-for-dom-element-shadowroot①"></a>[shadowRoot](#dom-element-shadowroot)</code></code></strong>

Returns <var>element</var>’s <a id="ref-for-concept-element-shadow-root①⑤"></a>[shadow root](#concept-element-shadow-root), if any, and if <a id="ref-for-concept-shadow-root④⑧"></a>[shadow root](#concept-shadow-root)’s <a id="ref-for-shadowroot-mode⑦"></a>[mode](#shadowroot-mode) is "<code>open</code>"; otherwise null.

A <a id="valid-shadow-host-name"></a><strong>valid shadow host name</strong> is:

- a <a id="ref-for-valid-custom-element-name②"></a>[valid custom element name](https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name)
- "<code>article</code>", "<code>aside</code>", "<code>blockquote</code>", "<code>body</code>", "<code>div</code>", "<code>footer</code>", "<code>h1</code>", "<code>h2</code>", "<code>h3</code>", "<code>h4</code>", "<code>h5</code>", "<code>h6</code>", "<code>header</code>", "<code>main</code>", "<code>nav</code>", "<code>p</code>", "<code>section</code>", or "<code>span</code>"

<strong>Note:</strong>

This list is intentionally limited so that built-in elements can gain internal <a id="ref-for-concept-shadow-tree⑧"></a>[shadow trees](#concept-shadow-tree) over time as needed.

The <a id="dom-element-attachshadow"></a><strong><code>attachShadow(<var>init</var>)</code></strong> method steps are:

1.  Let <var>registry</var> be <a id="ref-for-this②①③"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document⑤⓪"></a>[node document](#concept-node-document)’s <a id="ref-for-document-custom-element-registry①①"></a>[custom element registry](#document-custom-element-registry).

2.  If <var>init</var>\["<code><a id="ref-for-dom-shadowrootinit-customelementregistry①"></a>[customElementRegistry](#dom-shadowrootinit-customelementregistry)</code>"\] <a id="ref-for-map-exists①⑤"></a>[exists](https://infra.spec.whatwg.org/#map-exists), then set <var>registry</var> to it.

3.  If <var>registry</var> is non-null, <var>registry</var>’s <a id="ref-for-is-scoped⑦"></a>[is scoped](https://html.spec.whatwg.org/multipage/custom-elements.html#is-scoped) is false, and <var>registry</var> is not <a id="ref-for-this②①④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document⑤①"></a>[node document](#concept-node-document)’s <a id="ref-for-document-custom-element-registry①②"></a>[custom element registry](#document-custom-element-registry), then <a id="ref-for-dfn-throw⑤⑧"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror①⑧"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑦②"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

4.  Run <a id="ref-for-concept-attach-a-shadow-root①"></a>[attach a shadow root](#concept-attach-a-shadow-root) with <a id="ref-for-this②①⑤"></a>[this](https://webidl.spec.whatwg.org/#this), <var>init</var>\["<code><a id="ref-for-dom-shadowrootinit-mode"></a>[mode](#dom-shadowrootinit-mode)</code>"\], <var>init</var>\["<code><a id="ref-for-dom-shadowrootinit-delegatesfocus"></a>[delegatesFocus](#dom-shadowrootinit-delegatesfocus)</code>"\], <var>init</var>\["<code><a id="ref-for-dom-shadowrootinit-serializable"></a>[serializable](#dom-shadowrootinit-serializable)</code>"\], <var>init</var>\["<code><a id="ref-for-dom-shadowrootinit-slotassignment"></a>[slotAssignment](#dom-shadowrootinit-slotassignment)</code>"\], <var>init</var>\["<code><a id="ref-for-dom-shadowrootinit-clonable"></a>[clonable](#dom-shadowrootinit-clonable)</code>"\], and <var>registry</var>.

5.  Return <a id="ref-for-this②①⑥"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-element-shadow-root①⑥"></a>[shadow root](#concept-element-shadow-root).

To <a id="concept-attach-a-shadow-root"></a><strong>attach a shadow root</strong>, given an <a id="ref-for-concept-element⑨⑧"></a>[element](#concept-element) <var>element</var>, a string <var>mode</var>, a boolean <var>delegatesFocus</var>, a boolean <var>serializable</var>, a string <var>slotAssignment</var>, a boolean <var>clonable</var>, and null or a <code><a id="ref-for-customelementregistry①⑧"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code> object <var>registry</var>:

1.  If <var>element</var>’s <a id="ref-for-concept-element-namespace①⑧"></a>[namespace](#concept-element-namespace) is not the <a id="ref-for-html-namespace②⑧"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace), then <a id="ref-for-dfn-throw⑤⑨"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror①⑨"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑦③"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  If <var>element</var>’s <a id="ref-for-concept-element-local-name①⑥"></a>[local name](#concept-element-local-name) is not a <a id="ref-for-valid-shadow-host-name"></a>[valid shadow host name](#valid-shadow-host-name), then <a id="ref-for-dfn-throw⑥⓪"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror②⓪"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑦④"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  If <var>element</var>’s <a id="ref-for-concept-element-local-name①⑦"></a>[local name](#concept-element-local-name) is a <a id="ref-for-valid-custom-element-name③"></a>[valid custom element name](https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name), or <var>element</var>’s <a id="ref-for-concept-element-is-value③"></a>[<code>is</code> value](#concept-element-is-value) is non-null:

    1.  Let <var>definition</var> be the result of <a id="ref-for-look-up-a-custom-element-definition①"></a>[looking up a custom element definition](https://html.spec.whatwg.org/multipage/custom-elements.html#look-up-a-custom-element-definition) given <var>element</var>’s <a id="ref-for-element-custom-element-registry①⓪"></a>[custom element registry](#element-custom-element-registry), its <a id="ref-for-concept-element-namespace①⑨"></a>[namespace](#concept-element-namespace), its <a id="ref-for-concept-element-local-name①⑧"></a>[local name](#concept-element-local-name), and its <a id="ref-for-concept-element-is-value④"></a>[<code>is</code> value](#concept-element-is-value).

    2.  If <var>definition</var> is non-null and <var>definition</var>’s <a id="ref-for-concept-custom-element-definition-disable-shadow"></a>[disable shadow](https://html.spec.whatwg.org/multipage/custom-elements.html#concept-custom-element-definition-disable-shadow) is true, then <a id="ref-for-dfn-throw⑥①"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror②①"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑦⑤"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

4.  If <var>element</var> is a <a id="ref-for-element-shadow-host⑤"></a>[shadow host](#element-shadow-host):

    1.  Let <var>currentShadowRoot</var> be <var>element</var>’s <a id="ref-for-concept-element-shadow-root①⑦"></a>[shadow root](#concept-element-shadow-root).

    2.  If any of the following are true:

        - <var>currentShadowRoot</var>’s <a id="ref-for-shadowroot-declarative②"></a>[declarative](#shadowroot-declarative) is false; or

        - <var>currentShadowRoot</var>’s <a id="ref-for-shadowroot-mode⑧"></a>[mode](#shadowroot-mode) is not <var>mode</var>,

        then <a id="ref-for-dfn-throw⑥②"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notsupportederror②②"></a>[NotSupportedError](https://webidl.spec.whatwg.org/#notsupportederror)</code>" <code><a id="ref-for-idl-DOMException⑦⑥"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

    3.  <a id="ref-for-concept-node-remove①①"></a>[Remove](#concept-node-remove) all of <var>currentShadowRoot</var>’s <a id="ref-for-concept-tree-child⑤⑥"></a>[children](#concept-tree-child), in <a id="ref-for-concept-tree-order②③"></a>[tree order](#concept-tree-order).

    4.  Set <var>currentShadowRoot</var>’s <a id="ref-for-shadowroot-declarative③"></a>[declarative](#shadowroot-declarative) to false.

    5.  Return.

5.  Let <var>shadow</var> be the result of <a id="ref-for-create-a-node④"></a>[creating a node](#create-a-node) that implements <code><a id="ref-for-shadowroot①①"></a>[ShadowRoot](#shadowroot)</code>, given <var>element</var>’s <a id="ref-for-concept-node-document⑤②"></a>[node document](#concept-node-document).

6.  Set <var>shadow</var>’s <a id="ref-for-concept-documentfragment-host①③"></a>[host](#concept-documentfragment-host) to <var>element</var>.

7.  Set <var>shadow</var>’s <a id="ref-for-shadowroot-mode⑨"></a>[mode](#shadowroot-mode) to <var>mode</var>.

8.  Set <var>shadow</var>’s <a id="ref-for-shadowroot-delegates-focus②"></a>[delegates focus](#shadowroot-delegates-focus) to <var>delegatesFocus</var>.

9.  If <var>element</var>’s <a id="ref-for-concept-element-custom-element-state⑨"></a>[custom element state](#concept-element-custom-element-state) is "<code>precustomized</code>" or "<code>custom</code>", then set <var>shadow</var>’s <a id="ref-for-shadowroot-available-to-element-internals"></a>[available to element internals](#shadowroot-available-to-element-internals) to true.

10. Set <var>shadow</var>’s <a id="ref-for-shadowroot-declarative④"></a>[declarative](#shadowroot-declarative) to false.

11. Set <var>shadow</var>’s <a id="ref-for-shadowroot-serializable②"></a>[serializable](#shadowroot-serializable) to <var>serializable</var>.

12. Set <var>shadow</var>’s <a id="ref-for-shadowroot-slot-assignment⑥"></a>[slot assignment](#shadowroot-slot-assignment) to <var>slotAssignment</var>.

13. Set <var>shadow</var>’s <a id="ref-for-shadowroot-clonable②"></a>[clonable](#shadowroot-clonable) to <var>clonable</var>.

14. Set <var>shadow</var>’s <a id="ref-for-shadowroot-custom-element-registry⑨"></a>[custom element registry](#shadowroot-custom-element-registry) to <var>registry</var>.

15. Set <var>element</var>’s <a id="ref-for-concept-element-shadow-root①⑧"></a>[shadow root](#concept-element-shadow-root) to <var>shadow</var>.

The <a id="dom-element-shadowroot"></a><strong><code>shadowRoot</code></strong> getter steps are:

1.  Let <var>shadow</var> be <a id="ref-for-this②①⑦"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-element-shadow-root①⑨"></a>[shadow root](#concept-element-shadow-root).

2.  If <var>shadow</var> is null or its <a id="ref-for-shadowroot-mode①⓪"></a>[mode](#shadowroot-mode) is "<code>closed</code>", then return null.

3.  Return <var>shadow</var>.

------------------------------------------------------------------------

<strong><code><var>registry</var> = <var>element</var> . <code><a id="ref-for-dom-element-customelementregistry①"></a>[customElementRegistry](#dom-element-customelementregistry)</code></code></strong>

Returns <var>element</var>’s <code><a id="ref-for-customelementregistry①⑨"></a>[CustomElementRegistry](https://html.spec.whatwg.org/multipage/custom-elements.html#customelementregistry)</code> object, if any; otherwise null.

The <a id="dom-element-customelementregistry"></a><strong><code>customElementRegistry</code></strong> getter steps are to return <a id="ref-for-this②①⑧"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-element-custom-element-registry①①"></a>[custom element registry](#element-custom-element-registry).

------------------------------------------------------------------------

<strong><code><var>element</var> . <code><a id="ref-for-dom-element-closest①"></a>[closest(selectors)](#dom-element-closest)</code></code></strong>

Returns the first (starting at <var>element</var>) <a id="ref-for-concept-tree-inclusive-ancestor⑤"></a>[inclusive ancestor](#concept-tree-inclusive-ancestor) that matches <var>selectors</var>, and null otherwise.

<strong><code><var>element</var> . <code><a id="ref-for-dom-element-matches①"></a>[matches(selectors)](#dom-element-matches)</code></code></strong>

Returns true if matching <var>selectors</var> against <var>element</var>’s <a id="ref-for-concept-tree-root④②"></a>[root](#concept-tree-root) yields <var>element</var>; otherwise false.

The <a id="dom-element-closest"></a><strong><code>closest(<var>selectors</var>)</code></strong> method steps are:

1.  Let <var>selector</var> be the result of <a id="ref-for-parse-a-selector①"></a>[parse a selector](https://drafts.csswg.org/selectors-4/#parse-a-selector) from <var>selectors</var>. [\[SELECTORS4\]](https://dom.spec.whatwg.org/#biblio-selectors4)

2.  If <var>selector</var> is failure, then <a id="ref-for-dfn-throw⑥③"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-syntaxerror①"></a>[SyntaxError](https://webidl.spec.whatwg.org/#syntaxerror)</code>" <code><a id="ref-for-idl-DOMException⑦⑦"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  Let <var>elements</var> be <a id="ref-for-this②①⑨"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-tree-inclusive-ancestor⑥"></a>[inclusive ancestors](#concept-tree-inclusive-ancestor) that are <a id="ref-for-concept-element⑨⑨"></a>[elements](#concept-element), in reverse <a id="ref-for-concept-tree-order②④"></a>[tree order](#concept-tree-order).

4.  For each <var>element</var> of <var>elements</var>: if <a id="ref-for-match-a-selector-against-an-element"></a>[match a selector against an element](https://drafts.csswg.org/selectors-4/#match-a-selector-against-an-element), using <var>selector</var>, <var>element</var>, and <a id="ref-for-scoping-root①"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root) <a id="ref-for-this②②⓪"></a>[this](https://webidl.spec.whatwg.org/#this), returns success, return <var>element</var>. [\[SELECTORS4\]](https://dom.spec.whatwg.org/#biblio-selectors4)

5.  Return null.

The <a id="dom-element-matches"></a><strong><code>matches(<var>selectors</var>)</code></strong> and <a id="dom-element-webkitmatchesselector"></a><strong><code>webkitMatchesSelector(<var>selectors</var>)</code></strong> method steps are:

1.  Let <var>selector</var> be the result of <a id="ref-for-parse-a-selector②"></a>[parse a selector](https://drafts.csswg.org/selectors-4/#parse-a-selector) from <var>selectors</var>. [\[SELECTORS4\]](https://dom.spec.whatwg.org/#biblio-selectors4)

2.  If <var>selector</var> is failure, then <a id="ref-for-dfn-throw⑥④"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-syntaxerror②"></a>[SyntaxError](https://webidl.spec.whatwg.org/#syntaxerror)</code>" <code><a id="ref-for-idl-DOMException⑦⑧"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  If the result of <a id="ref-for-match-a-selector-against-an-element①"></a>[match a selector against an element](https://drafts.csswg.org/selectors-4/#match-a-selector-against-an-element), using <var>selector</var>, <a id="ref-for-this②②①"></a>[this](https://webidl.spec.whatwg.org/#this), and <a id="ref-for-scoping-root②"></a>[scoping root](https://drafts.csswg.org/selectors-4/#scoping-root) <a id="ref-for-this②②②"></a>[this](https://webidl.spec.whatwg.org/#this), returns success, then return true; otherwise, return false. [\[SELECTORS4\]](https://dom.spec.whatwg.org/#biblio-selectors4)

------------------------------------------------------------------------

The <a id="dom-element-getelementsbytagname"></a><strong><code>getElementsByTagName(<var>qualifiedName</var>)</code></strong> method steps are to return the <a id="ref-for-concept-getelementsbytagname①"></a>[list of elements with qualified name <var>qualifiedName</var>](#concept-getelementsbytagname) for <a id="ref-for-this②②③"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-element-getelementsbytagnamens"></a><strong><code>getElementsByTagNameNS(<var>namespace</var>, <var>localName</var>)</code></strong> method steps are to return the <a id="ref-for-concept-getelementsbytagnamens①"></a>[list of elements with namespace <var>namespace</var> and local name <var>localName</var>](#concept-getelementsbytagnamens) for <a id="ref-for-this②②④"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-element-getelementsbyclassname"></a><strong><code>getElementsByClassName(<var>classNames</var>)</code></strong> method steps are to return the <a id="ref-for-concept-getelementsbyclassname①"></a>[list of elements with class names <var>classNames</var>](#concept-getelementsbyclassname) for <a id="ref-for-this②②⑤"></a>[this](https://webidl.spec.whatwg.org/#this).

------------------------------------------------------------------------

To <a id="insert-adjacent"></a><strong>insert adjacent</strong>, given an <a id="ref-for-concept-element①⓪⓪"></a>[element](#concept-element) <var>element</var>, string <var>where</var>, and a <a id="ref-for-concept-node①⑥⑨"></a>[node](#concept-node) <var>node</var>, run the steps associated with the first <a id="ref-for-ascii-case-insensitive②"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for <var>where</var>:

<strong>"<code>beforebegin</code>"</strong>

If <var>element</var>’s <a id="ref-for-concept-tree-parent③④"></a>[parent](#concept-tree-parent) is null, return null.

Return the result of <a id="ref-for-concept-node-pre-insert⑦"></a>[pre-inserting](#concept-node-pre-insert) <var>node</var> into <var>element</var>’s <a id="ref-for-concept-tree-parent③⑤"></a>[parent](#concept-tree-parent) before <var>element</var>.

<strong>"<code>afterbegin</code>"</strong>

Return the result of <a id="ref-for-concept-node-pre-insert⑧"></a>[pre-inserting](#concept-node-pre-insert) <var>node</var> into <var>element</var> before <var>element</var>’s <a id="ref-for-concept-tree-first-child⑤"></a>[first child](#concept-tree-first-child).

<strong>"<code>beforeend</code>"</strong>

Return the result of <a id="ref-for-concept-node-pre-insert⑨"></a>[pre-inserting](#concept-node-pre-insert) <var>node</var> into <var>element</var> before null.

<strong>"<code>afterend</code>"</strong>

If <var>element</var>’s <a id="ref-for-concept-tree-parent③⑥"></a>[parent](#concept-tree-parent) is null, return null.

Return the result of <a id="ref-for-concept-node-pre-insert①⓪"></a>[pre-inserting](#concept-node-pre-insert) <var>node</var> into <var>element</var>’s <a id="ref-for-concept-tree-parent③⑦"></a>[parent](#concept-tree-parent) before <var>element</var>’s <a id="ref-for-concept-tree-next-sibling①②"></a>[next sibling](#concept-tree-next-sibling).

<strong>Otherwise</strong>

<a id="ref-for-dfn-throw⑥⑤"></a>[Throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-syntaxerror③"></a>[SyntaxError](https://webidl.spec.whatwg.org/#syntaxerror)</code>" <code><a id="ref-for-idl-DOMException⑦⑨"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

The <a id="dom-element-insertadjacentelement"></a><strong><code>insertAdjacentElement(<var>where</var>, <var>element</var>)</code></strong> method steps are to return the result of running <a id="ref-for-insert-adjacent"></a>[insert adjacent](#insert-adjacent), give <a id="ref-for-this②②⑥"></a>[this](https://webidl.spec.whatwg.org/#this), <var>where</var>, and <var>element</var>.

The <a id="dom-element-insertadjacenttext"></a><strong><code>insertAdjacentText(<var>where</var>, <var>data</var>)</code></strong> method steps are:

1.  Let <var>text</var> be the result of <a id="ref-for-create-a-text-node④"></a>[creating a text node](#create-a-text-node) given <a id="ref-for-this②②⑦"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document⑤③"></a>[node document](#concept-node-document) and <var>data</var>.

2.  Run <a id="ref-for-insert-adjacent①"></a>[insert adjacent](#insert-adjacent), given <a id="ref-for-this②②⑧"></a>[this](https://webidl.spec.whatwg.org/#this), <var>where</var>, and <var>text</var>.

<strong>Note:</strong>

This method returns nothing because it existed before we had a chance to design it.

#### <a id="interface-namednodemap"></a>4.9.1. Interface <code><a id="ref-for-namednodemap③"></a>[NamedNodeMap](#namednodemap)</code>[](#interface-namednodemap)

<a id="ref-for-Exposed①③"></a><a id="ref-for-LegacyUnenumerableNamedProperties①"></a><a id="namednodemap"></a><a id="ref-for-idl-unsigned-long⑦"></a><a id="ref-for-dom-namednodemap-length"></a><a id="ref-for-attr②⑦"></a><a id="ref-for-dom-namednodemap-item"></a><a id="ref-for-idl-unsigned-long⑧"></a><a id="dom-namednodemap-item-index-index"></a><a id="ref-for-attr②⑧"></a><a id="ref-for-dom-namednodemap-getnameditem"></a><a id="ref-for-idl-DOMString①⓪①"></a><a id="dom-namednodemap-getnameditem-qualifiedname-qualifiedname"></a><a id="ref-for-attr②⑨"></a><a id="ref-for-dom-namednodemap-getnameditemns"></a><a id="ref-for-idl-DOMString①⓪②"></a><a id="dom-namednodemap-getnameditemns-namespace-localname-namespace"></a><a id="ref-for-idl-DOMString①⓪③"></a><a id="dom-namednodemap-getnameditemns-namespace-localname-localname"></a><a id="ref-for-cereactions③②"></a><a id="ref-for-attr③⓪"></a><a id="ref-for-dom-namednodemap-setnameditem"></a><a id="ref-for-attr③①"></a><a id="dom-namednodemap-setnameditem-attr-attr"></a><a id="ref-for-cereactions③③"></a><a id="ref-for-attr③②"></a><a id="ref-for-dom-namednodemap-setnameditemns"></a><a id="ref-for-attr③③"></a><a id="dom-namednodemap-setnameditemns-attr-attr"></a><a id="ref-for-cereactions③④"></a><a id="ref-for-attr③④"></a><a id="ref-for-dom-namednodemap-removenameditem"></a><a id="ref-for-idl-DOMString①⓪④"></a><a id="dom-namednodemap-removenameditem-qualifiedname-qualifiedname"></a><a id="ref-for-cereactions③⑤"></a><a id="ref-for-attr③⑤"></a><a id="ref-for-dom-namednodemap-removenameditemns"></a><a id="ref-for-idl-DOMString①⓪⑤"></a><a id="dom-namednodemap-removenameditemns-namespace-localname-namespace"></a><a id="ref-for-idl-DOMString①⓪⑥"></a><a id="dom-namednodemap-removenameditemns-namespace-localname-localname"></a>

``` text
[Exposed=Window,
 LegacyUnenumerableNamedProperties]
interface NamedNodeMap {
  readonly attribute unsigned long length;
  getter Attr? item(unsigned long index);
  getter Attr? getNamedItem(DOMString qualifiedName);
  Attr? getNamedItemNS(DOMString? namespace, DOMString localName);
  [CEReactions] Attr? setNamedItem(Attr attr);
  [CEReactions] Attr? setNamedItemNS(Attr attr);
  [CEReactions] Attr removeNamedItem(DOMString qualifiedName);
  [CEReactions] Attr removeNamedItemNS(DOMString? namespace, DOMString localName);
};
```

A <code><a id="ref-for-namednodemap④"></a>[NamedNodeMap](#namednodemap)</code> has an associated <a id="concept-namednodemap-element"></a><strong>element</strong> (an <a id="ref-for-concept-element①⓪①"></a>[element](#concept-element)).

A <code><a id="ref-for-namednodemap⑤"></a>[NamedNodeMap](#namednodemap)</code> object’s <a id="concept-namednodemap-attribute"></a><strong>attribute list</strong> is its <a id="ref-for-concept-namednodemap-element"></a>[element](#concept-namednodemap-element)’s <a id="ref-for-concept-element-attribute②⓪"></a>[attribute list](#concept-element-attribute).

------------------------------------------------------------------------

A <code><a id="ref-for-namednodemap⑥"></a>[NamedNodeMap](#namednodemap)</code> object’s <a id="ref-for-dfn-supported-property-indices④"></a>[supported property indices](https://webidl.spec.whatwg.org/#dfn-supported-property-indices) are the numbers in the range zero to its <a id="ref-for-concept-namednodemap-attribute"></a>[attribute list](#concept-namednodemap-attribute)’s <a id="ref-for-list-size⑤"></a>[size](https://infra.spec.whatwg.org/#list-size) − 1, unless the <a id="ref-for-concept-namednodemap-attribute①"></a>[attribute list](#concept-namednodemap-attribute) <a id="ref-for-list-is-empty①⓪"></a>[is empty](https://infra.spec.whatwg.org/#list-is-empty), in which case there are no <a id="ref-for-dfn-supported-property-indices⑤"></a>[supported property indices](https://webidl.spec.whatwg.org/#dfn-supported-property-indices).

The <a id="dom-namednodemap-length"></a><strong><code>length</code></strong> getter steps are to return the <a id="ref-for-concept-namednodemap-attribute②"></a>[attribute list](#concept-namednodemap-attribute)’s <a id="ref-for-list-size⑥"></a>[size](https://infra.spec.whatwg.org/#list-size).

The <a id="dom-namednodemap-item"></a><strong><code>item(<var>index</var>)</code></strong> method steps are:

1.  If <var>index</var> is equal to or greater than <a id="ref-for-this②②⑨"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-namednodemap-attribute③"></a>[attribute list](#concept-namednodemap-attribute)’s <a id="ref-for-list-size⑦"></a>[size](https://infra.spec.whatwg.org/#list-size), then return null.

2.  Otherwise, return <a id="ref-for-this②③⓪"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-namednodemap-attribute④"></a>[attribute list](#concept-namednodemap-attribute)\[<var>index</var>\].

A <code><a id="ref-for-namednodemap⑦"></a>[NamedNodeMap](#namednodemap)</code> object’s <a id="ref-for-dfn-supported-property-names①"></a>[supported property names](https://webidl.spec.whatwg.org/#dfn-supported-property-names) are the return value of running these steps:

1.  Let <var>names</var> be the <a id="ref-for-concept-attribute-qualified-name①③"></a>[qualified names](#concept-attribute-qualified-name) of the <a id="ref-for-concept-attribute⑤①"></a>[attributes](#concept-attribute) in this <code><a id="ref-for-namednodemap⑧"></a>[NamedNodeMap](#namednodemap)</code> object’s <a id="ref-for-concept-namednodemap-attribute⑤"></a>[attribute list](#concept-namednodemap-attribute), with duplicates omitted, in order.

2.  If this <code><a id="ref-for-namednodemap⑨"></a>[NamedNodeMap](#namednodemap)</code> object’s <a id="ref-for-concept-namednodemap-element①"></a>[element](#concept-namednodemap-element) is in the <a id="ref-for-html-namespace②⑨"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) and its <a id="ref-for-concept-node-document⑤④"></a>[node document](#concept-node-document) is an <a id="ref-for-html-document①⑤"></a>[HTML document](#html-document), then <a id="ref-for-list-iterate②⑤"></a>[for each](https://infra.spec.whatwg.org/#list-iterate) <var>name</var> of <var>names</var>:

    1.  Let <var>lowercaseName</var> be <var>name</var>, in <a id="ref-for-ascii-lowercase⑦"></a>[ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase).

    2.  If <var>lowercaseName</var> is not equal to <var>name</var>, remove <var>name</var> from <var>names</var>.

3.  Return <var>names</var>.

The <a id="dom-namednodemap-getnameditem"></a><strong><code>getNamedItem(<var>qualifiedName</var>)</code></strong> method steps are to return the result of <a id="ref-for-concept-element-attributes-get-by-name③"></a>[getting an attribute](#concept-element-attributes-get-by-name) given <var>qualifiedName</var> and <a id="ref-for-concept-namednodemap-element②"></a>[element](#concept-namednodemap-element).

The <a id="dom-namednodemap-getnameditemns"></a><strong><code>getNamedItemNS(<var>namespace</var>, <var>localName</var>)</code></strong> method steps are to return the result of <a id="ref-for-concept-element-attributes-get-by-namespace⑥"></a>[getting an attribute](#concept-element-attributes-get-by-namespace) given <var>namespace</var>, <var>localName</var>, and <a id="ref-for-concept-namednodemap-element③"></a>[element](#concept-namednodemap-element).

The <a id="dom-namednodemap-setnameditem"></a><strong><code>setNamedItem(<var>attr</var>)</code></strong> and <a id="dom-namednodemap-setnameditemns"></a><strong><code>setNamedItemNS(<var>attr</var>)</code></strong> method steps are to return the result of <a id="ref-for-concept-element-attributes-set①"></a>[setting an attribute](#concept-element-attributes-set) given <var>attr</var> and <a id="ref-for-concept-namednodemap-element④"></a>[element](#concept-namednodemap-element).

The <a id="dom-namednodemap-removenameditem"></a><strong><code>removeNamedItem(<var>qualifiedName</var>)</code></strong> method steps are:

1.  Let <var>attr</var> be the result of <a id="ref-for-concept-element-attributes-remove-by-name②"></a>[removing an attribute](#concept-element-attributes-remove-by-name) given <var>qualifiedName</var> and <a id="ref-for-concept-namednodemap-element⑤"></a>[element](#concept-namednodemap-element).

2.  If <var>attr</var> is null, then <a id="ref-for-dfn-throw⑥⑥"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notfounderror④"></a>[NotFoundError](https://webidl.spec.whatwg.org/#notfounderror)</code>" <code><a id="ref-for-idl-DOMException⑧⓪"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  Return <var>attr</var>.

The <a id="dom-namednodemap-removenameditemns"></a><strong><code>removeNamedItemNS(<var>namespace</var>, <var>localName</var>)</code></strong> method steps are:

1.  Let <var>attr</var> be the result of <a id="ref-for-concept-element-attributes-remove-by-namespace①"></a>[removing an attribute](#concept-element-attributes-remove-by-namespace) given <var>namespace</var>, <var>localName</var>, and <a id="ref-for-concept-namednodemap-element⑥"></a>[element](#concept-namednodemap-element).

2.  If <var>attr</var> is null, then <a id="ref-for-dfn-throw⑥⑦"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) a "<code><a id="ref-for-notfounderror⑤"></a>[NotFoundError](https://webidl.spec.whatwg.org/#notfounderror)</code>" <code><a id="ref-for-idl-DOMException⑧①"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  Return <var>attr</var>.

#### <a id="interface-attr"></a>4.9.2. Interface <code><a id="ref-for-attr③⑥"></a>[Attr](#attr)</code>[](#interface-attr)

<a id="ref-for-Exposed①④"></a><a id="attr"></a><a id="ref-for-node⑥④"></a><a id="ref-for-idl-DOMString①⓪⑦"></a><a id="ref-for-dom-attr-namespaceuri"></a><a id="ref-for-idl-DOMString①⓪⑧"></a><a id="ref-for-dom-attr-prefix"></a><a id="ref-for-idl-DOMString①⓪⑨"></a><a id="ref-for-dom-attr-localname"></a><a id="ref-for-idl-DOMString①①⓪"></a><a id="ref-for-dom-attr-name"></a><a id="ref-for-cereactions③⑥"></a><a id="ref-for-idl-DOMString①①①"></a><a id="ref-for-dom-attr-value"></a><a id="ref-for-element⑤⓪"></a><a id="ref-for-dom-attr-ownerelement"></a><a id="ref-for-idl-boolean⑤①"></a><a id="ref-for-dom-attr-specified"></a>

``` text
[Exposed=Window]
interface Attr : Node {
  readonly attribute DOMString? namespaceURI;
  readonly attribute DOMString? prefix;
  readonly attribute DOMString localName;
  readonly attribute DOMString name;
  [CEReactions] attribute DOMString value;

  readonly attribute Element? ownerElement;

  readonly attribute boolean specified; // historical; always returns true
};
```

<code><a id="ref-for-attr③⑦"></a>[Attr](#attr)</code> <a id="ref-for-concept-node①⑦⓪"></a>[nodes](#concept-node) are simply known as <a id="concept-attribute"></a><strong>attributes</strong>. They are sometimes referred to as <em>content attributes</em> to avoid confusion with IDL attributes.

<a id="ref-for-concept-attribute⑤②"></a>[Attributes](#concept-attribute) have a <a id="concept-attribute-namespace"></a><strong>namespace</strong> (null or a non-empty string), <a id="concept-attribute-namespace-prefix"></a><strong>namespace prefix</strong> (null or a non-empty string), <a id="concept-attribute-local-name"></a><strong>local name</strong> (a non-empty string), <a id="concept-attribute-value"></a><strong>value</strong> (a string), and <a id="concept-attribute-element"></a><strong>element</strong> (null or an <a id="ref-for-concept-element①⓪②"></a>[element](#concept-element)).

<strong>Note:</strong>

If designed today they would just have a name and value. ☹

An <a id="ref-for-concept-attribute⑤③"></a>[attribute](#concept-attribute)’s <a id="concept-attribute-qualified-name"></a><strong>qualified name</strong> is its <a id="ref-for-concept-attribute-local-name②⓪"></a>[local name](#concept-attribute-local-name) if its <a id="ref-for-concept-attribute-namespace-prefix④"></a>[namespace prefix](#concept-attribute-namespace-prefix) is null, and its <a id="ref-for-concept-attribute-namespace-prefix⑤"></a>[namespace prefix](#concept-attribute-namespace-prefix), followed by "<code>&#58;</code>", followed by its <a id="ref-for-concept-attribute-local-name②①"></a>[local name](#concept-attribute-local-name), otherwise.

<strong>Note:</strong>

User agents could have this as an internal slot as an optimization.

To <a id="create-an-attribute"></a><strong>create an attribute</strong> given a <a id="ref-for-concept-document⑤⑥"></a>[document](#concept-document) <var>document</var>, a string <var>localName</var>, and optionally a string-or-null <var>namespace</var> (default null), a string-or-null <var>prefix</var> (default null), and a string <var>value</var> (default the empty string):

1.  Let <var>attribute</var> be the result of <a id="ref-for-create-a-node⑤"></a>[creating a node](#create-a-node) that implements <code><a id="ref-for-attr③⑧"></a>[Attr](#attr)</code>, given <var>document</var>.

2.  Set <var>attribute</var>’s <a id="ref-for-concept-attribute-namespace①⑦"></a>[namespace](#concept-attribute-namespace) to <var>namespace</var>, <a id="ref-for-concept-attribute-namespace-prefix⑥"></a>[namespace prefix](#concept-attribute-namespace-prefix) to <var>prefix</var>, <a id="ref-for-concept-attribute-local-name②②"></a>[local name](#concept-attribute-local-name) to <var>localName</var>, and <a id="ref-for-concept-attribute-value②④"></a>[value](#concept-attribute-value) to <var>value</var>.

3.  Return <var>attribute</var>.

An <a id="concept-named-attribute"></a><strong><code><var>A</var></code> attribute</strong> is an <a id="ref-for-concept-attribute⑤④"></a>[attribute](#concept-attribute) whose <a id="ref-for-concept-attribute-local-name②③"></a>[local name](#concept-attribute-local-name) is <code><var>A</var></code> and whose <a id="ref-for-concept-attribute-namespace①⑧"></a>[namespace](#concept-attribute-namespace) and <a id="ref-for-concept-attribute-namespace-prefix⑦"></a>[namespace prefix](#concept-attribute-namespace-prefix) are null.

------------------------------------------------------------------------

The <a id="dom-attr-namespaceuri"></a><strong><code>namespaceURI</code></strong> getter steps are to return <a id="ref-for-this②③①"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-attribute-namespace①⑨"></a>[namespace](#concept-attribute-namespace).

The <a id="dom-attr-prefix"></a><strong><code>prefix</code></strong> getter steps are to return <a id="ref-for-this②③②"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-attribute-namespace-prefix⑧"></a>[namespace prefix](#concept-attribute-namespace-prefix).

The <a id="dom-attr-localname"></a><strong><code>localName</code></strong> getter steps are to return <a id="ref-for-this②③③"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-attribute-local-name②④"></a>[local name](#concept-attribute-local-name).

The <a id="dom-attr-name"></a><strong><code>name</code></strong> getter steps are to return <a id="ref-for-this②③④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-attribute-qualified-name①④"></a>[qualified name](#concept-attribute-qualified-name).

The <a id="dom-attr-value"></a><strong><code>value</code></strong> getter steps are to return <a id="ref-for-this②③⑤"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-attribute-value②⑤"></a>[value](#concept-attribute-value).

To <a id="set-an-existing-attribute-value"></a><strong>set an existing attribute value</strong>, given an <a id="ref-for-concept-attribute⑤⑤"></a>[attribute](#concept-attribute) <var>attribute</var> and string <var>value</var>, run these steps:

1.  If <var>attribute</var>’s <a id="ref-for-concept-attribute-element①④"></a>[element](#concept-attribute-element) is null, then set <var>attribute</var>’s <a id="ref-for-concept-attribute-value②⑥"></a>[value](#concept-attribute-value) to <var>value</var> and return.

2.  Let <var>element</var> be <var>attribute</var>’s <a id="ref-for-concept-attribute-element①⑤"></a>[element](#concept-attribute-element).

3.  Let <var>verifiedValue</var> be the result of calling <a id="ref-for-get-trusted-type-compliant-attribute-value③"></a>[get trusted type compliant attribute value](https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-attribute-value) with <var>attribute</var>’s <a id="ref-for-concept-attribute-local-name②⑤"></a>[local name](#concept-attribute-local-name), <var>attribute</var>’s <a id="ref-for-concept-attribute-namespace②⓪"></a>[namespace](#concept-attribute-namespace), <var>element</var>, and <var>value</var>. [\[TRUSTED-TYPES\]](https://dom.spec.whatwg.org/#biblio-trusted-types)

4.  If <var>attribute</var>’s <a id="ref-for-concept-attribute-element①⑥"></a>[element](#concept-attribute-element) is null, then set <var>attribute</var>’s <a id="ref-for-concept-attribute-value②⑦"></a>[value](#concept-attribute-value) to <var>verifiedValue</var> and return.

5.  <a id="ref-for-concept-element-attributes-change②"></a>[Change](#concept-element-attributes-change) <var>attribute</var> to <var>verifiedValue</var>.

The <code><a id="ref-for-dom-attr-value①"></a>[value](#dom-attr-value)</code> setter steps are to <a id="ref-for-set-an-existing-attribute-value②"></a>[set an existing attribute value](#set-an-existing-attribute-value) with <a id="ref-for-this②③⑥"></a>[this](https://webidl.spec.whatwg.org/#this) and the given value.

------------------------------------------------------------------------

The <a id="dom-attr-ownerelement"></a><strong><code>ownerElement</code></strong> getter steps are to return <a id="ref-for-this②③⑦"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-attribute-element①⑦"></a>[element](#concept-attribute-element).

------------------------------------------------------------------------

The <a id="dom-attr-specified"></a><strong><code>specified</code></strong> getter steps are to return true.

### <a id="interface-characterdata"></a>4.10. Interface <code><a id="ref-for-characterdata①⑥"></a>[CharacterData](#characterdata)</code>[](#interface-characterdata)

<a id="ref-for-Exposed①⑤"></a><a id="characterdata"></a><a id="ref-for-node⑥⑤"></a><a id="ref-for-LegacyNullToEmptyString①"></a><a id="ref-for-idl-DOMString①①②"></a><a id="ref-for-dom-characterdata-data"></a><a id="ref-for-idl-unsigned-long⑨"></a><a id="ref-for-dom-characterdata-length"></a><a id="ref-for-idl-DOMString①①③"></a><a id="ref-for-dom-characterdata-substringdata"></a><a id="ref-for-idl-unsigned-long①⓪"></a><a id="dom-characterdata-substringdata-offset-count-offset"></a><a id="ref-for-idl-unsigned-long①①"></a><a id="dom-characterdata-substringdata-offset-count-count"></a><a id="ref-for-idl-undefined②⑧"></a><a id="ref-for-dom-characterdata-appenddata"></a><a id="ref-for-idl-DOMString①①④"></a><a id="dom-characterdata-appenddata-data-data"></a><a id="ref-for-idl-undefined②⑨"></a><a id="ref-for-dom-characterdata-insertdata"></a><a id="ref-for-idl-unsigned-long①②"></a><a id="dom-characterdata-insertdata-offset-data-offset"></a><a id="ref-for-idl-DOMString①①⑤"></a><a id="dom-characterdata-insertdata-offset-data-data"></a><a id="ref-for-idl-undefined③⓪"></a><a id="ref-for-dom-characterdata-deletedata"></a><a id="ref-for-idl-unsigned-long①③"></a><a id="dom-characterdata-deletedata-offset-count-offset"></a><a id="ref-for-idl-unsigned-long①④"></a><a id="dom-characterdata-deletedata-offset-count-count"></a><a id="ref-for-idl-undefined③①"></a><a id="ref-for-dom-characterdata-replacedata"></a><a id="ref-for-idl-unsigned-long①⑤"></a><a id="dom-characterdata-replacedata-offset-count-data-offset"></a><a id="ref-for-idl-unsigned-long①⑥"></a><a id="dom-characterdata-replacedata-offset-count-data-count"></a><a id="ref-for-idl-DOMString①①⑥"></a><a id="dom-characterdata-replacedata-offset-count-data-data"></a>

``` text
[Exposed=Window]
interface CharacterData : Node {
  attribute [LegacyNullToEmptyString] DOMString data;
  readonly attribute unsigned long length;
  DOMString substringData(unsigned long offset, unsigned long count);
  undefined appendData(DOMString data);
  undefined insertData(unsigned long offset, DOMString data);
  undefined deleteData(unsigned long offset, unsigned long count);
  undefined replaceData(unsigned long offset, unsigned long count, DOMString data);
};
```

<strong>Note:</strong>

<code><a id="ref-for-characterdata①⑦"></a>[CharacterData](#characterdata)</code> is an abstract interface. You cannot get a direct instance of it. It is used by <code><a id="ref-for-text②②"></a>[Text](#text)</code>, <code><a id="ref-for-processinginstruction①③"></a>[ProcessingInstruction](#processinginstruction)</code>, and <code><a id="ref-for-comment①③"></a>[Comment](#comment)</code> <a id="ref-for-concept-node①⑦①"></a>[nodes](#concept-node).

Each <a id="ref-for-concept-node①⑦②"></a>[node](#concept-node) inheriting from the <code><a id="ref-for-characterdata①⑧"></a>[CharacterData](#characterdata)</code> interface has an associated mutable string called <a id="concept-cd-data"></a><strong>data</strong>.

To <a id="concept-cd-replace"></a><strong>replace data</strong> of a <a id="ref-for-concept-node①⑦③"></a>[node](#concept-node) <var>node</var> with an integer <var>offset</var>, integer <var>count</var>, string <var>data</var>, and an optional boolean <var>piAttributesAlreadyUpdated</var> (default false):

1.  Let <var>length</var> be <var>node</var>’s <a id="ref-for-concept-node-length⑤"></a>[length](#concept-node-length).

2.  If <var>offset</var> is greater than <var>length</var>, then <a id="ref-for-dfn-throw⑥⑧"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-indexsizeerror"></a>[IndexSizeError](https://webidl.spec.whatwg.org/#indexsizeerror)</code>" <code><a id="ref-for-idl-DOMException⑧②"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  If <var>offset</var> + <var>count</var> is greater than <var>length</var>, then set <var>count</var> to <var>length</var> − <var>offset</var>.

4.  <a id="ref-for-queue-a-mutation-record②"></a>[Queue a mutation record](https://dom.spec.whatwg.org/#queue-a-mutation-record) of "<code>characterData</code>" for <var>node</var> with null, null, <var>node</var>’s <a id="ref-for-concept-cd-data①⑥"></a>[data](#concept-cd-data), « », « », null, and null.

5.  Insert <var>data</var> into <var>node</var>’s <a id="ref-for-concept-cd-data①⑦"></a>[data](#concept-cd-data) after <var>offset</var> <a id="ref-for-code-unit"></a>[code units](https://infra.spec.whatwg.org/#code-unit).

6.  Let <var>deleteOffset</var> be <var>offset</var> + <var>data</var>’s <a id="ref-for-string-length④"></a>[length](https://infra.spec.whatwg.org/#string-length).

7.  Starting from <var>deleteOffset</var> <a id="ref-for-code-unit①"></a>[code units](https://infra.spec.whatwg.org/#code-unit), remove <var>count</var> <a id="ref-for-code-unit②"></a>[code units](https://infra.spec.whatwg.org/#code-unit) from <var>node</var>’s <a id="ref-for-concept-cd-data①⑧"></a>[data](#concept-cd-data).

8.  For each <a id="ref-for-concept-live-range⑨"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-start-node⑥"></a>[start node](https://dom.spec.whatwg.org/#concept-range-start-node) is <var>node</var> and <a id="ref-for-concept-range-start-offset⑦"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) is greater than <var>offset</var> but less than or equal to <var>offset</var> + <var>count</var>: set its <a id="ref-for-concept-range-start-offset⑧"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) to <var>offset</var>.

9.  For each <a id="ref-for-concept-live-range①⓪"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-end-node⑥"></a>[end node](https://dom.spec.whatwg.org/#concept-range-end-node) is <var>node</var> and <a id="ref-for-concept-range-end-offset⑦"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) is greater than <var>offset</var> but less than or equal to <var>offset</var> + <var>count</var>: set its <a id="ref-for-concept-range-end-offset⑧"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) to <var>offset</var>.

10. For each <a id="ref-for-concept-live-range①①"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-start-node⑦"></a>[start node](https://dom.spec.whatwg.org/#concept-range-start-node) is <var>node</var> and <a id="ref-for-concept-range-start-offset⑨"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) is greater than <var>offset</var> + <var>count</var>: increase its <a id="ref-for-concept-range-start-offset①⓪"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) by <var>data</var>’s <a id="ref-for-string-length⑤"></a>[length](https://infra.spec.whatwg.org/#string-length) and decrease it by <var>count</var>.

11. For each <a id="ref-for-concept-live-range①②"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-end-node⑦"></a>[end node](https://dom.spec.whatwg.org/#concept-range-end-node) is <var>node</var> and <a id="ref-for-concept-range-end-offset⑨"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) is greater than <var>offset</var> + <var>count</var>: increase its <a id="ref-for-concept-range-end-offset①⓪"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) by <var>data</var>’s <a id="ref-for-string-length⑥"></a>[length](https://infra.spec.whatwg.org/#string-length) and decrease it by <var>count</var>.

12. If <var>node</var> is a <code><a id="ref-for-processinginstruction①④"></a>[ProcessingInstruction](#processinginstruction)</code> <a id="ref-for-concept-node①⑦④"></a>[node](#concept-node) and <var>piAttributesAlreadyUpdated</var> is false, then <a id="ref-for-update-attributes-from-data"></a>[update attributes from data](#update-attributes-from-data) given <var>node</var>.

13. If <var>node</var>’s <a id="ref-for-concept-tree-parent③⑧"></a>[parent](#concept-tree-parent) is non-null, then run the <a id="ref-for-concept-node-children-changed-ext②"></a>[children changed steps](#concept-node-children-changed-ext) for <var>node</var>’s <a id="ref-for-concept-tree-parent③⑨"></a>[parent](#concept-tree-parent).

To <a id="concept-cd-substring"></a><strong>substring data</strong> of a <a id="ref-for-concept-node①⑦⑤"></a>[node](#concept-node) <var>node</var> with an integer <var>offset</var> and integer <var>count</var>:

1.  Let <var>length</var> be <var>node</var>’s <a id="ref-for-concept-node-length⑥"></a>[length](#concept-node-length).

2.  If <var>offset</var> is greater than <var>length</var>, then <a id="ref-for-dfn-throw⑥⑨"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-indexsizeerror①"></a>[IndexSizeError](https://webidl.spec.whatwg.org/#indexsizeerror)</code>" <code><a id="ref-for-idl-DOMException⑧③"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  If <var>offset</var> + <var>count</var> is greater than <var>length</var>, then return a string whose value is the <a id="ref-for-code-unit③"></a>[code units](https://infra.spec.whatwg.org/#code-unit) from the <var>offset</var><sup>th</sup> <a id="ref-for-code-unit④"></a>[code unit](https://infra.spec.whatwg.org/#code-unit) to the end of <var>node</var>’s <a id="ref-for-concept-cd-data①⑨"></a>[data](#concept-cd-data).

4.  Return a string whose value is the <a id="ref-for-code-unit⑤"></a>[code units](https://infra.spec.whatwg.org/#code-unit) from the <var>offset</var><sup>th</sup> <a id="ref-for-code-unit⑥"></a>[code unit](https://infra.spec.whatwg.org/#code-unit) to the <var>offset</var>+<var>count</var><sup>th</sup> <a id="ref-for-code-unit⑦"></a>[code unit](https://infra.spec.whatwg.org/#code-unit) in <var>node</var>’s <a id="ref-for-concept-cd-data②⓪"></a>[data](#concept-cd-data).

The <a id="dom-characterdata-data"></a><strong><code>data</code></strong> getter steps are to return <a id="ref-for-this②③⑧"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-cd-data②①"></a>[data](#concept-cd-data). Its setter steps are to <a id="ref-for-concept-cd-replace④"></a>[replace data](#concept-cd-replace) of <a id="ref-for-this②③⑨"></a>[this](https://webidl.spec.whatwg.org/#this) with 0, <a id="ref-for-this②④⓪"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-length⑦"></a>[length](#concept-node-length), and the given value.

The <a id="dom-characterdata-length"></a><strong><code>length</code></strong> getter steps are to return <a id="ref-for-this②④①"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-length⑧"></a>[length](#concept-node-length).

The <a id="dom-characterdata-substringdata"></a><strong><code>substringData(<var>offset</var>, <var>count</var>)</code></strong> method steps are to return the result of <a id="ref-for-concept-cd-substring"></a>[substringing data](#concept-cd-substring) of <a id="ref-for-this②④②"></a>[this](https://webidl.spec.whatwg.org/#this) with <var>offset</var> and <var>count</var>.

The <a id="dom-characterdata-appenddata"></a><strong><code>appendData(<var>data</var>)</code></strong> method steps are to <a id="ref-for-concept-cd-replace⑤"></a>[replace data](#concept-cd-replace) of <a id="ref-for-this②④③"></a>[this](https://webidl.spec.whatwg.org/#this) with <a id="ref-for-this②④④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-length⑨"></a>[length](#concept-node-length), 0, and <var>data</var>.

The <a id="dom-characterdata-insertdata"></a><strong><code>insertData(<var>offset</var>, <var>data</var>)</code></strong> method steps are to <a id="ref-for-concept-cd-replace⑥"></a>[replace data](#concept-cd-replace) of <a id="ref-for-this②④⑤"></a>[this](https://webidl.spec.whatwg.org/#this) with <var>offset</var>, 0, and <var>data</var>.

The <a id="dom-characterdata-deletedata"></a><strong><code>deleteData(<var>offset</var>, <var>count</var>)</code></strong> method steps are to <a id="ref-for-concept-cd-replace⑦"></a>[replace data](#concept-cd-replace) of <a id="ref-for-this②④⑥"></a>[this](https://webidl.spec.whatwg.org/#this) with <var>offset</var>, <var>count</var>, and the empty string.

The <a id="dom-characterdata-replacedata"></a><strong><code>replaceData(<var>offset</var>, <var>count</var>, <var>data</var>)</code></strong> method steps are to <a id="ref-for-concept-cd-replace⑧"></a>[replace data](#concept-cd-replace) of <a id="ref-for-this②④⑦"></a>[this](https://webidl.spec.whatwg.org/#this) with <var>offset</var>, <var>count</var>, and <var>data</var>.

### <a id="interface-text"></a>4.11. Interface <code><a id="ref-for-text②③"></a>[Text](#text)</code>[](#interface-text)

<a id="ref-for-Exposed①⑥"></a><a id="text"></a><a id="ref-for-characterdata①⑨"></a><a id="ref-for-dom-text-text"></a><a id="ref-for-idl-DOMString①①⑦"></a><a id="dom-text-text-data-data"></a><a id="ref-for-NewObject②②"></a><a id="ref-for-text②④"></a><a id="ref-for-dom-text-splittext"></a><a id="ref-for-idl-unsigned-long①⑦"></a><a id="dom-text-splittext-offset-offset"></a><a id="ref-for-idl-DOMString①①⑧"></a><a id="ref-for-dom-text-wholetext"></a>

``` text
[Exposed=Window]
interface Text : CharacterData {
  constructor(optional DOMString data = "");

  [NewObject] Text splitText(unsigned long offset);
  readonly attribute DOMString wholeText;
};
```

<strong><code><var>text</var> = new <a id="ref-for-dom-text-text①"></a>[Text(\[<var>data</var> = ""\])](#dom-text-text)</code></strong>

Returns a new <code><a id="ref-for-text②⑤"></a>[Text](#text)</code> <a id="ref-for-concept-node①⑦⑥"></a>[node](#concept-node) whose <a id="ref-for-concept-cd-data②②"></a>[data](#concept-cd-data) is <var>data</var>.

<strong><code><var>text</var> . <code><a id="ref-for-dom-text-splittext①"></a>[splitText(offset)](#dom-text-splittext)</code></code></strong>

Splits <a id="ref-for-concept-cd-data②③"></a>[data](#concept-cd-data) at the given <var>offset</var> and returns the remainder as <code><a id="ref-for-text②⑥"></a>[Text](#text)</code> <a id="ref-for-concept-node①⑦⑦"></a>[node](#concept-node).

<strong><code><var>text</var> . <code><a id="ref-for-dom-text-wholetext①"></a>[wholeText](#dom-text-wholetext)</code></code></strong>

Returns the combined <a id="ref-for-concept-cd-data②④"></a>[data](#concept-cd-data) of all direct <code><a id="ref-for-text②⑦"></a>[Text](#text)</code> <a id="ref-for-concept-node①⑦⑧"></a>[node](#concept-node) <a id="ref-for-concept-tree-sibling①③"></a>[siblings](#concept-tree-sibling).

------------------------------------------------------------------------

An <a id="exclusive-text-node"></a><strong>exclusive <code><a id="ref-for-text②⑧"></a>[Text](#text)</code> node</strong> is a <code><a id="ref-for-text②⑨"></a>[Text](#text)</code> <a id="ref-for-concept-node①⑦⑨"></a>[node](#concept-node) that is not a <code><a id="ref-for-cdatasection⑧"></a>[CDATASection](#cdatasection)</code> <a id="ref-for-concept-node①⑧⓪"></a>[node](#concept-node).

The <a id="contiguous-text-nodes"></a><strong>contiguous <code><a id="ref-for-text③⓪"></a>[Text](#text)</code> nodes</strong> of a <a id="ref-for-concept-node①⑧①"></a>[node](#concept-node) <var>node</var> are <var>node</var>, <var>node</var>’s <a id="ref-for-concept-tree-previous-sibling⑧"></a>[previous sibling](#concept-tree-previous-sibling) <code><a id="ref-for-text③①"></a>[Text](#text)</code> <a id="ref-for-concept-node①⑧②"></a>[node](#concept-node), if any, and its <a id="ref-for-contiguous-text-nodes"></a>[contiguous <code>Text</code> nodes](#contiguous-text-nodes), and <var>node</var>’s <a id="ref-for-concept-tree-next-sibling①③"></a>[next sibling](#concept-tree-next-sibling) <code><a id="ref-for-text③②"></a>[Text](#text)</code> <a id="ref-for-concept-node①⑧③"></a>[node](#concept-node), if any, and its <a id="ref-for-contiguous-text-nodes①"></a>[contiguous <code>Text</code> nodes](#contiguous-text-nodes), avoiding any duplicates.

The <a id="contiguous-exclusive-text-nodes"></a><strong>contiguous exclusive <code><a id="ref-for-text③③"></a>[Text](#text)</code> nodes</strong> of a <a id="ref-for-concept-node①⑧④"></a>[node](#concept-node) <var>node</var> are <var>node</var>, <var>node</var>’s <a id="ref-for-concept-tree-previous-sibling⑨"></a>[previous sibling](#concept-tree-previous-sibling) <a id="ref-for-exclusive-text-node⑧"></a>[exclusive <code>Text</code> node](#exclusive-text-node), if any, and its <a id="ref-for-contiguous-exclusive-text-nodes③"></a>[contiguous exclusive <code>Text</code> nodes](#contiguous-exclusive-text-nodes), and <var>node</var>’s <a id="ref-for-concept-tree-next-sibling①④"></a>[next sibling](#concept-tree-next-sibling) <a id="ref-for-exclusive-text-node⑨"></a>[exclusive <code>Text</code> node](#exclusive-text-node), if any, and its <a id="ref-for-contiguous-exclusive-text-nodes④"></a>[contiguous exclusive <code>Text</code> nodes](#contiguous-exclusive-text-nodes), avoiding any duplicates.

The <a id="concept-child-text-content"></a><strong>child text content</strong> of a <a id="ref-for-concept-node①⑧⑤"></a>[node](#concept-node) <var>node</var> is the <a id="ref-for-string-concatenate②"></a>[concatenation](https://infra.spec.whatwg.org/#string-concatenate) of the <a id="ref-for-concept-cd-data②⑤"></a>[data](#concept-cd-data) of all the <code><a id="ref-for-text③④"></a>[Text](#text)</code> <a id="ref-for-concept-node①⑧⑥"></a>[node](#concept-node) <a id="ref-for-concept-tree-child⑤⑦"></a>[children](#concept-tree-child) of <var>node</var>, in <a id="ref-for-concept-tree-order②⑤"></a>[tree order](#concept-tree-order).

The <a id="concept-descendant-text-content"></a><strong>descendant text content</strong> of a <a id="ref-for-concept-node①⑧⑦"></a>[node](#concept-node) <var>node</var> is the <a id="ref-for-string-concatenate③"></a>[concatenation](https://infra.spec.whatwg.org/#string-concatenate) of the <a id="ref-for-concept-cd-data②⑥"></a>[data](#concept-cd-data) of all the <code><a id="ref-for-text③⑤"></a>[Text](#text)</code> <a id="ref-for-concept-node①⑧⑧"></a>[node](#concept-node) <a id="ref-for-concept-tree-descendant③①"></a>[descendants](#concept-tree-descendant) of <var>node</var>, in <a id="ref-for-concept-tree-order②⑥"></a>[tree order](#concept-tree-order).

To <a id="create-a-text-node"></a><strong>create a text node</strong> given a <a id="ref-for-concept-document⑤⑦"></a>[document](#concept-document) <var>document</var> and a string <var>data</var>:

1.  Let <var>text</var> be the result of <a id="ref-for-create-a-node⑥"></a>[creating a node](#create-a-node) that implements <code><a id="ref-for-text③⑥"></a>[Text](#text)</code>, given <var>document</var>.

2.  Set <var>text</var>’s <a id="ref-for-concept-cd-data②⑦"></a>[data](#concept-cd-data) to <var>data</var>.

3.  Return <var>text</var>.

------------------------------------------------------------------------

The <a id="dom-text-text"></a><strong><code>new Text(<var>data</var>)</code></strong> constructor steps are to set <a id="ref-for-this②④⑧"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-cd-data②⑧"></a>[data](#concept-cd-data) to <var>data</var> and <a id="ref-for-this②④⑨"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document⑤⑤"></a>[node document](#concept-node-document) to <a id="ref-for-current-global-object②"></a>[current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object)’s <a id="ref-for-concept-document-window④"></a>[associated <code>Document</code>](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

To <a id="concept-text-split"></a><strong>split</strong> a <code><a id="ref-for-text③⑦"></a>[Text](#text)</code> <a id="ref-for-concept-node①⑧⑨"></a>[node](#concept-node) <var>node</var> with integer <var>offset</var>:

1.  Let <var>length</var> be <var>node</var>’s <a id="ref-for-concept-node-length①⓪"></a>[length](#concept-node-length).

2.  If <var>offset</var> is greater than <var>length</var>, then <a id="ref-for-dfn-throw⑦⓪"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-indexsizeerror②"></a>[IndexSizeError](https://webidl.spec.whatwg.org/#indexsizeerror)</code>" <code><a id="ref-for-idl-DOMException⑧④"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  Let <var>count</var> be <var>length</var> − <var>offset</var>.

4.  Let <var>newData</var> be the result of <a id="ref-for-concept-cd-substring①"></a>[substringing data](#concept-cd-substring) of <var>node</var> with <var>offset</var> and <var>count</var>.

5.  Let <var>newNode</var> be the result of <a id="ref-for-create-a-text-node⑤"></a>[creating a text node](#create-a-text-node) given <var>node</var>’s <a id="ref-for-concept-node-document⑤⑥"></a>[node document](#concept-node-document) and <var>newData</var>.

6.  Let <var>parent</var> be <var>node</var>’s <a id="ref-for-concept-tree-parent④⓪"></a>[parent](#concept-tree-parent).

7.  If <var>parent</var> is non-null:

    1.  <a id="ref-for-concept-node-insert⑨"></a>[Insert](#concept-node-insert) <var>newNode</var> into <var>parent</var> before <var>node</var>’s <a id="ref-for-concept-tree-next-sibling①⑤"></a>[next sibling](#concept-tree-next-sibling).

    2.  For each <a id="ref-for-concept-live-range①③"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-start-node⑧"></a>[start node](https://dom.spec.whatwg.org/#concept-range-start-node) is <var>node</var> and <a id="ref-for-concept-range-start-offset①①"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) is greater than <var>offset</var>, set its <a id="ref-for-concept-range-start-node⑨"></a>[start node](https://dom.spec.whatwg.org/#concept-range-start-node) to <var>newNode</var> and decrease its <a id="ref-for-concept-range-start-offset①②"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) by <var>offset</var>.

    3.  For each <a id="ref-for-concept-live-range①④"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-end-node⑧"></a>[end node](https://dom.spec.whatwg.org/#concept-range-end-node) is <var>node</var> and <a id="ref-for-concept-range-end-offset①①"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) is greater than <var>offset</var>, set its <a id="ref-for-concept-range-end-node⑨"></a>[end node](https://dom.spec.whatwg.org/#concept-range-end-node) to <var>newNode</var> and decrease its <a id="ref-for-concept-range-end-offset①②"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) by <var>offset</var>.

    4.  For each <a id="ref-for-concept-live-range①⑤"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-start-node①⓪"></a>[start node](https://dom.spec.whatwg.org/#concept-range-start-node) is <var>parent</var> and <a id="ref-for-concept-range-start-offset①③"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) is equal to the <a id="ref-for-concept-tree-index⑨"></a>[index](#concept-tree-index) of <var>node</var> plus 1, increase its <a id="ref-for-concept-range-start-offset①④"></a>[start offset](https://dom.spec.whatwg.org/#concept-range-start-offset) by 1.

    5.  For each <a id="ref-for-concept-live-range①⑥"></a>[live range](https://dom.spec.whatwg.org/#concept-live-range) whose <a id="ref-for-concept-range-end-node①⓪"></a>[end node](https://dom.spec.whatwg.org/#concept-range-end-node) is <var>parent</var> and <a id="ref-for-concept-range-end-offset①③"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) is equal to the <a id="ref-for-concept-tree-index①⓪"></a>[index](#concept-tree-index) of <var>node</var> plus 1, increase its <a id="ref-for-concept-range-end-offset①④"></a>[end offset](https://dom.spec.whatwg.org/#concept-range-end-offset) by 1.

8.  <a id="ref-for-concept-cd-replace⑨"></a>[Replace data](#concept-cd-replace) of <var>node</var> with <var>offset</var>, <var>count</var>, and the empty string.

9.  Return <var>newNode</var>.

The <a id="dom-text-splittext"></a><strong><code>splitText(<var>offset</var>)</code></strong> method steps are to <a id="ref-for-concept-text-split"></a>[split](#concept-text-split) <a id="ref-for-this②⑤⓪"></a>[this](https://webidl.spec.whatwg.org/#this) with offset <var>offset</var>.

The <a id="dom-text-wholetext"></a><strong><code>wholeText</code></strong> getter steps are to return the <a id="ref-for-string-concatenate④"></a>[concatenation](https://infra.spec.whatwg.org/#string-concatenate) of the <a id="ref-for-concept-cd-data②⑨"></a>[data](#concept-cd-data) of the <a id="ref-for-contiguous-text-nodes②"></a>[contiguous <code>Text</code> nodes](#contiguous-text-nodes) of <a id="ref-for-this②⑤①"></a>[this](https://webidl.spec.whatwg.org/#this), in <a id="ref-for-concept-tree-order②⑦"></a>[tree order](#concept-tree-order).

### <a id="interface-cdatasection"></a>4.12. Interface <code><a id="ref-for-cdatasection⑨"></a>[CDATASection](#cdatasection)</code>[](#interface-cdatasection)

<a id="ref-for-Exposed①⑦"></a><a id="cdatasection"></a><a id="ref-for-text③⑧"></a>

``` text
[Exposed=Window]
interface CDATASection : Text {
};
```

To <a id="create-a-cdata-section-node"></a><strong>create a CDATA section node</strong> given a <a id="ref-for-concept-document⑤⑧"></a>[document](#concept-document) <var>document</var> and a string <var>data</var>:

1.  Let <var>node</var> be the result of <a id="ref-for-create-a-node⑦"></a>[creating a node](#create-a-node) that implements <code><a id="ref-for-cdatasection①⓪"></a>[CDATASection](#cdatasection)</code>, given <var>document</var>.

2.  Set <var>node</var>’s <a id="ref-for-concept-cd-data③⓪"></a>[data](#concept-cd-data) to <var>data</var>.

3.  Return <var>node</var>.

### <a id="interface-processinginstruction"></a>4.13. Interface <code><a id="ref-for-processinginstruction①⑤"></a>[ProcessingInstruction](#processinginstruction)</code>[](#interface-processinginstruction)

<a id="ref-for-Exposed①⑧"></a><a id="processinginstruction"></a><a id="ref-for-characterdata②⓪"></a><a id="ref-for-dom-processinginstruction-processinginstruction"></a><a id="ref-for-idl-DOMString①①⑨"></a><a id="dom-processinginstruction-processinginstruction-target-data-target"></a><a id="ref-for-idl-DOMString①②⓪"></a><a id="dom-processinginstruction-processinginstruction-target-data-data"></a><a id="ref-for-idl-DOMString①②①"></a><a id="ref-for-dom-processinginstruction-target"></a><a id="ref-for-idl-boolean⑤②"></a><a id="ref-for-dom-processinginstruction-hasattributes"></a><a id="ref-for-idl-sequence⑥"></a><a id="ref-for-idl-DOMString①②②"></a><a id="ref-for-dom-processinginstruction-getattributenames"></a><a id="ref-for-idl-DOMString①②③"></a><a id="ref-for-dom-processinginstruction-getattribute"></a><a id="ref-for-idl-DOMString①②④"></a><a id="dom-processinginstruction-getattribute-name-name"></a><a id="ref-for-idl-undefined③②"></a><a id="ref-for-dom-processinginstruction-setattribute"></a><a id="ref-for-idl-DOMString①②⑤"></a><a id="dom-processinginstruction-setattribute-name-value-name"></a><a id="ref-for-idl-DOMString①②⑥"></a><a id="dom-processinginstruction-setattribute-name-value-value"></a><a id="ref-for-idl-undefined③③"></a><a id="ref-for-dom-processinginstruction-removeattribute"></a><a id="ref-for-idl-DOMString①②⑦"></a><a id="dom-processinginstruction-removeattribute-name-name"></a><a id="ref-for-idl-boolean⑤③"></a><a id="ref-for-dom-processinginstruction-toggleattribute"></a><a id="ref-for-idl-DOMString①②⑧"></a><a id="dom-processinginstruction-toggleattribute-name-force-name"></a><a id="ref-for-idl-boolean⑤④"></a><a id="dom-processinginstruction-toggleattribute-name-force-force"></a><a id="ref-for-idl-boolean⑤⑤"></a><a id="ref-for-dom-processinginstruction-hasattribute"></a><a id="ref-for-idl-DOMString①②⑨"></a><a id="dom-processinginstruction-hasattribute-name-name"></a>

``` text
[Exposed=Window]
interface ProcessingInstruction : CharacterData {
  constructor(DOMString target, optional DOMString data = "");

  readonly attribute DOMString target;

  boolean hasAttributes();
  sequence<DOMString> getAttributeNames();
  DOMString? getAttribute(DOMString name);
  undefined setAttribute(DOMString name, DOMString value);
  undefined removeAttribute(DOMString name);
  boolean toggleAttribute(DOMString name, optional boolean force);
  boolean hasAttribute(DOMString name);
};
```

<strong><code><var>pi</var> = new <a id="ref-for-dom-processinginstruction-processinginstruction①"></a>[ProcessingInstruction(<var>target</var> \[, <var>data</var> = ""\])](#dom-processinginstruction-processinginstruction)</code></strong>

Returns a new <code><a id="ref-for-processinginstruction①⑥"></a>[ProcessingInstruction](#processinginstruction)</code> <a id="ref-for-concept-node①⑨⓪"></a>[node](#concept-node) whose <a id="ref-for-concept-pi-target⑤"></a>[target](#concept-pi-target) is <var>target</var> and <a id="ref-for-concept-cd-data③①"></a>[data](#concept-cd-data) is <var>data</var>.

<strong><code><var>pi</var> . <code><a id="ref-for-dom-processinginstruction-target①"></a>[target](#dom-processinginstruction-target)</code></code></strong>

Returns the <a id="ref-for-concept-pi-target⑥"></a>[target](#concept-pi-target).

<strong><code><var>pi</var> . <a id="ref-for-dom-processinginstruction-hasattributes①"></a>[hasAttributes](#dom-processinginstruction-hasattributes)()</code></strong>

Returns true if <var>pi</var> has attributes; otherwise false.

<strong><code><var>pi</var> . <a id="ref-for-dom-processinginstruction-getattributenames①"></a>[getAttributeNames](#dom-processinginstruction-getattributenames)()</code></strong>

Returns the names of all of <var>pi</var>’s attributes. Cannot contain duplicates.

<strong><code><var>pi</var> . <a id="ref-for-dom-processinginstruction-getattribute①"></a>[getAttribute](#dom-processinginstruction-getattribute)(<var>name</var>)</code></strong>

Returns the value of <var>pi</var>’s attribute named <var>name</var>, and null if there is no such attribute.

<strong><code><var>pi</var> . <a id="ref-for-dom-processinginstruction-setattribute①"></a>[setAttribute](#dom-processinginstruction-setattribute)(<var>name</var>, <var>value</var>)</code></strong>

Sets <var>pi</var>’s attribute named <var>name</var> to <var>value</var>.

<strong><code><var>pi</var> . <a id="ref-for-dom-processinginstruction-removeattribute①"></a>[removeAttribute](#dom-processinginstruction-removeattribute)(<var>name</var>)</code></strong>

Removes <var>pi</var>’s attribute named <var>name</var>.

<strong><code><var>pi</var> . <a id="ref-for-dom-processinginstruction-toggleattribute①"></a>[toggleAttribute](#dom-processinginstruction-toggleattribute)(<var>name</var> \[, <var>force</var>\])</code></strong>

If <var>force</var> is not given, "toggles" <var>name</var>, removing it if it is present and adding it if it is not present. If <var>force</var> is true, adds <var>name</var>. If <var>force</var> is false, removes <var>name</var>.

Returns true if <var>name</var> is now present; otherwise false.

<strong><code><var>pi</var> . <a id="ref-for-dom-processinginstruction-hasattribute①"></a>[hasAttribute](#dom-processinginstruction-hasattribute)(<var>name</var>)</code></strong>

Returns true if <var>pi</var> has an attribute named <var>name</var>; otherwise false.

<code><a id="ref-for-processinginstruction①⑦"></a>[ProcessingInstruction](#processinginstruction)</code> <a id="ref-for-concept-node①⑨①"></a>[nodes](#concept-node) have an associated <a id="concept-pi-target"></a><strong>target</strong>.

<code><a id="ref-for-processinginstruction①⑧"></a>[ProcessingInstruction](#processinginstruction)</code> <a id="ref-for-concept-node①⑨②"></a>[nodes](#concept-node) have an associated <a id="concept-pi-attribute-map"></a><strong>attribute map</strong>, which is a <a id="ref-for-ordered-map①"></a>[map](https://infra.spec.whatwg.org/#ordered-map), initially empty.

To <a id="create-a-processing-instruction-node"></a><strong>create a processing instruction node</strong> given a <a id="ref-for-concept-document⑤⑨"></a>[document](#concept-document) <var>document</var>, a string <var>target</var>, and a string <var>data</var>:

1.  Let <var>pi</var> be the result of <a id="ref-for-create-a-node⑧"></a>[creating a node](#create-a-node) that implements <code><a id="ref-for-processinginstruction①⑨"></a>[ProcessingInstruction](#processinginstruction)</code>, given <var>document</var>.

2.  <a id="ref-for-processinginstruction-initialize"></a>[Initialize](#processinginstruction-initialize) <var>pi</var> with <var>target</var> and <var>data</var>.

3.  Return <var>pi</var>.

To <a id="processinginstruction-initialize"></a><strong>initialize</strong> a <code><a id="ref-for-processinginstruction②⓪"></a>[ProcessingInstruction](#processinginstruction)</code> <a id="ref-for-concept-node①⑨③"></a>[node](#concept-node) <var>pi</var>, with <var>target</var> and <var>data</var>:

1.  If <var>target</var> does not match the <code><a id="ref-for-NT-Name①"></a>[Name](https://www.w3.org/TR/xml/#NT-Name)</code> production, then <a id="ref-for-dfn-throw⑦①"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-invalidcharactererror①④"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException⑧⑤"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  If <var>data</var> contains the string "<code>?&#62;</code>", then <a id="ref-for-dfn-throw⑦②"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-invalidcharactererror①⑤"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException⑧⑥"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

3.  Set <var>pi</var>’s <a id="ref-for-concept-pi-target⑦"></a>[target](#concept-pi-target) to <var>target</var>.

4.  Set <var>pi</var>’s <a id="ref-for-concept-cd-data③②"></a>[data](#concept-cd-data) to <var>data</var>.

5.  <a id="ref-for-update-attributes-from-data①"></a>[Update attributes from data](#update-attributes-from-data) given <var>pi</var>.

To <a id="update-attributes-from-data"></a><strong>update attributes from data</strong>, given a <code><a id="ref-for-processinginstruction②①"></a>[ProcessingInstruction](#processinginstruction)</code> <a id="ref-for-concept-node①⑨④"></a>[node](#concept-node) <var>pi</var>:

1.  <a id="ref-for-map-clear"></a>[Clear](https://infra.spec.whatwg.org/#map-clear) <var>pi</var>’s <a id="ref-for-concept-pi-attribute-map"></a>[attribute map](#concept-pi-attribute-map).

2.  Let <var>result</var> be the <a id="ref-for-dt-parsing-result"></a>[parsing result](https://www.w3.org/TR/xml-stylesheet/#dt-parsing-result) of invoking the <a id="ref-for-dt-parsing"></a>[rules for parsing pseudo-attributes from a string](https://www.w3.org/TR/xml-stylesheet/#dt-parsing) given <var>pi</var>’s <a id="ref-for-concept-cd-data③③"></a>[data](#concept-cd-data).

3.  If <var>result</var> is an error, then return.

4.  For each <a id="ref-for-dt-pseudo-attribute"></a>[pseudo-attribute](https://www.w3.org/TR/xml-stylesheet/#dt-pseudo-attribute) <var>pseudoAttr</var> of <var>result</var>: <a id="ref-for-map-set④"></a>[set](https://infra.spec.whatwg.org/#map-set) <var>pi</var>’s <a id="ref-for-concept-pi-attribute-map①"></a>[attribute map](#concept-pi-attribute-map)\[<var>pseudoAttr</var>’s <a id="ref-for-dt-name"></a>[name](https://www.w3.org/TR/xml-stylesheet/#dt-name)\] to <var>pseudoAttr</var>’s <a id="ref-for-dt-value"></a>[value](https://www.w3.org/TR/xml-stylesheet/#dt-value).

To <a id="update-data-from-attributes"></a><strong>update data from attributes</strong>, given a <code><a id="ref-for-processinginstruction②②"></a>[ProcessingInstruction](#processinginstruction)</code> <a id="ref-for-concept-node①⑨⑤"></a>[node](#concept-node) <var>pi</var>:

1.  Let <var>data</var> be the empty string.

2.  <a id="ref-for-map-iterate②"></a>[For each](https://infra.spec.whatwg.org/#map-iterate) <var>name</var> → <var>value</var> of <var>pi</var>’s <a id="ref-for-concept-pi-attribute-map②"></a>[attribute map](#concept-pi-attribute-map):

    1.  If <var>data</var> is not the empty string, then append U+0020 SPACE to <var>data</var>.

    2.  Append <var>name</var> to <var>data</var>.

    3.  Append U+003D (=) to <var>data</var>.

    4.  Append U+0022 (") to <var>data</var>.

    5.  Replace any U+0026 (&) in <var>value</var> with "<code>&#38;amp;</code>".

    6.  Replace any U+003C (\<) in <var>value</var> with "<code>&#38;lt;</code>".

    7.  Replace any U+003E (\>) in <var>value</var> with "<code>&#38;gt;</code>".

    8.  Replace any U+0022 (") in <var>value</var> with "<code>&#38;quot;</code>".

    9.  Append <var>value</var> to <var>data</var>.

    10. Append U+0022 (") to <var>data</var>.

3.  <a id="ref-for-concept-cd-replace①⓪"></a>[Replace data](#concept-cd-replace) of <var>pi</var> with 0, <var>pi</var>’s <a id="ref-for-concept-node-length①①"></a>[length](#concept-node-length), <var>data</var>, and true.

To <a id="get-a-processing-instruction-attribute"></a><strong>get a processing instruction attribute</strong>, given a <code><a id="ref-for-processinginstruction②③"></a>[ProcessingInstruction](#processinginstruction)</code> <a id="ref-for-concept-node①⑨⑥"></a>[node](#concept-node) <var>pi</var> and a string <var>name</var>: return <var>pi</var>’s <a id="ref-for-concept-pi-attribute-map③"></a>[attribute map](#concept-pi-attribute-map)\[<var>name</var>\] <a id="ref-for-map-with-default①"></a>[with default](https://infra.spec.whatwg.org/#map-with-default) null.

------------------------------------------------------------------------

The <a id="dom-processinginstruction-processinginstruction"></a><strong><code>new ProcessingInstruction(<var>target</var>, <var>data</var>)</code></strong> constructor steps are:

1.  Set <a id="ref-for-this②⑤②"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document⑤⑦"></a>[node document](#concept-node-document) to <a id="ref-for-current-global-object③"></a>[current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object)’s <a id="ref-for-concept-document-window⑤"></a>[associated <code>Document</code>](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

2.  <a id="ref-for-processinginstruction-initialize①"></a>[Initialize](#processinginstruction-initialize) <a id="ref-for-this②⑤③"></a>[this](https://webidl.spec.whatwg.org/#this) with <var>target</var> and <var>data</var>.

The <a id="dom-processinginstruction-target"></a><strong><code>target</code></strong> getter steps are to return <a id="ref-for-this②⑤④"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-pi-target⑧"></a>[target](#concept-pi-target).

The <a id="dom-processinginstruction-hasattributes"></a><strong><code>hasAttributes()</code></strong> method steps are to return false if <a id="ref-for-this②⑤⑤"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-pi-attribute-map④"></a>[attribute map](#concept-pi-attribute-map) <a id="ref-for-map-is-empty①"></a>[is empty](https://infra.spec.whatwg.org/#map-is-empty); otherwise true.

The <a id="dom-processinginstruction-getattributenames"></a><strong><code>getAttributeNames()</code></strong> method steps are to return the result of <a id="ref-for-map-getting-the-keys"></a>[getting the keys](https://infra.spec.whatwg.org/#map-getting-the-keys) of <a id="ref-for-this②⑤⑥"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-pi-attribute-map⑤"></a>[attribute map](#concept-pi-attribute-map).

The <a id="dom-processinginstruction-getattribute"></a><strong><code>getAttribute(<var>name</var>)</code></strong> method steps are to return the result of <a id="ref-for-get-a-processing-instruction-attribute"></a>[getting a processing instruction attribute](#get-a-processing-instruction-attribute) given <a id="ref-for-this②⑤⑦"></a>[this](https://webidl.spec.whatwg.org/#this) and <var>name</var>.

The <a id="dom-processinginstruction-setattribute"></a><strong><code>setAttribute(<var>name</var>, <var>value</var>)</code></strong> method steps are:

1.  If <var>name</var> is not a <a id="ref-for-valid-attribute-local-name④"></a>[valid attribute local name](#valid-attribute-local-name), then <a id="ref-for-dfn-throw⑦③"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-invalidcharactererror①⑥"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException⑧⑦"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  <a id="ref-for-map-set⑤"></a>[Set](https://infra.spec.whatwg.org/#map-set) <a id="ref-for-this②⑤⑧"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-pi-attribute-map⑥"></a>[attribute map](#concept-pi-attribute-map)\[<var>name</var>\] to <var>value</var>.

3.  <a id="ref-for-update-data-from-attributes"></a>[Update data from attributes](#update-data-from-attributes) given <a id="ref-for-this②⑤⑨"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-processinginstruction-removeattribute"></a><strong><code>removeAttribute(<var>name</var>)</code></strong> method steps are:

1.  <a id="ref-for-map-remove①"></a>[Remove](https://infra.spec.whatwg.org/#map-remove) <a id="ref-for-this②⑥⓪"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-pi-attribute-map⑦"></a>[attribute map](#concept-pi-attribute-map)\[<var>name</var>\].

2.  <a id="ref-for-update-data-from-attributes①"></a>[Update data from attributes](#update-data-from-attributes) given <a id="ref-for-this②⑥①"></a>[this](https://webidl.spec.whatwg.org/#this).

The <a id="dom-processinginstruction-toggleattribute"></a><strong><code>toggleAttribute(<var>name</var>, <var>force</var>)</code></strong> method steps are:

1.  If <var>name</var> is not a <a id="ref-for-valid-attribute-local-name⑤"></a>[valid attribute local name](#valid-attribute-local-name), then <a id="ref-for-dfn-throw⑦④"></a>[throw](https://webidl.spec.whatwg.org/#dfn-throw) an "<code><a id="ref-for-invalidcharactererror①⑦"></a>[InvalidCharacterError](https://webidl.spec.whatwg.org/#invalidcharactererror)</code>" <code><a id="ref-for-idl-DOMException⑧⑧"></a>[DOMException](https://webidl.spec.whatwg.org/#idl-DOMException)</code>.

2.  Let <var>attributes</var> be <a id="ref-for-this②⑥②"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-pi-attribute-map⑧"></a>[attribute map](#concept-pi-attribute-map).

3.  If <var>attributes</var>\[<var>name</var>\] does not <a id="ref-for-map-exists①⑥"></a>[exist](https://infra.spec.whatwg.org/#map-exists):

    1.  If <var>force</var> is not given or is true, <a id="ref-for-map-set⑥"></a>[set](https://infra.spec.whatwg.org/#map-set) <var>attributes</var>\[<var>name</var>\] to the empty string, <a id="ref-for-update-data-from-attributes②"></a>[update data from attributes](#update-data-from-attributes) given <a id="ref-for-this②⑥③"></a>[this](https://webidl.spec.whatwg.org/#this), and then return true.

    2.  Return false.

4.  If <var>force</var> is not given or is false, <a id="ref-for-map-remove②"></a>[remove](https://infra.spec.whatwg.org/#map-remove) <var>attributes</var>\[<var>name</var>\], <a id="ref-for-update-data-from-attributes③"></a>[update data from attributes](#update-data-from-attributes) given <a id="ref-for-this②⑥④"></a>[this](https://webidl.spec.whatwg.org/#this), and then return false.

5.  Return true.

The <a id="dom-processinginstruction-hasattribute"></a><strong><code>hasAttribute(<var>name</var>)</code></strong> method steps are to return true if <a id="ref-for-this②⑥⑤"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-pi-attribute-map⑨"></a>[attribute map](#concept-pi-attribute-map)\[<var>name</var>\] <a id="ref-for-list-contain①②"></a>[exists](https://infra.spec.whatwg.org/#list-contain); otherwise false.

### <a id="interface-comment"></a>4.14. Interface <code><a id="ref-for-comment①④"></a>[Comment](#comment)</code>[](#interface-comment)

<a id="ref-for-Exposed①⑨"></a><a id="comment"></a><a id="ref-for-characterdata②①"></a><a id="ref-for-dom-comment-comment"></a><a id="ref-for-idl-DOMString①③⓪"></a><a id="dom-comment-comment-data-data"></a>

``` text
[Exposed=Window]
interface Comment : CharacterData {
  constructor(optional DOMString data = "");
};
```

<strong><code><var>comment</var> = new <a id="ref-for-dom-comment-comment①"></a>[Comment(\[<var>data</var> = ""\])](#dom-comment-comment)</code></strong>

Returns a new <code><a id="ref-for-comment①⑤"></a>[Comment](#comment)</code> <a id="ref-for-concept-node①⑨⑦"></a>[node](#concept-node) whose <a id="ref-for-concept-cd-data③④"></a>[data](#concept-cd-data) is <var>data</var>.

To <a id="create-a-comment-node"></a><strong>create a comment node</strong> given a <a id="ref-for-concept-document⑥⓪"></a>[document](#concept-document) <var>document</var> and a string <var>data</var>:

1.  Let <var>comment</var> be the result of <a id="ref-for-create-a-node⑨"></a>[creating a node](#create-a-node) that implements <code><a id="ref-for-comment①⑥"></a>[Comment](#comment)</code>, given <var>document</var>.

2.  Set <var>comment</var>’s <a id="ref-for-concept-cd-data③⑤"></a>[data](#concept-cd-data) to <var>data</var>.

3.  Return <var>comment</var>.

The <a id="dom-comment-comment"></a><strong><code>new Comment(<var>data</var>)</code></strong> constructor steps are to set <a id="ref-for-this②⑥⑥"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-cd-data③⑥"></a>[data](#concept-cd-data) to <var>data</var> and <a id="ref-for-this②⑥⑦"></a>[this](https://webidl.spec.whatwg.org/#this)’s <a id="ref-for-concept-node-document⑤⑧"></a>[node document](#concept-node-document) to <a id="ref-for-current-global-object④"></a>[current global object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object)’s <a id="ref-for-concept-document-window⑥"></a>[associated <code>Document</code>](https://html.spec.whatwg.org/multipage/nav-history-apis.html#concept-document-window).

## <a id="ipr"></a>Intellectual property rights[](#ipr)

Part of the revision history of the integration points related to <a id="ref-for-concept-element-custom⑧"></a>[custom](#concept-element-custom) elements can be found in [the w3c/webcomponents repository](https://github.com/w3c/webcomponents), which is available under the [W3C Software and Document License](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document).

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). This work is licensed under a [Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/). To the extent portions of it are incorporated into source code, such portions in the source code are licensed under the [BSD 3-Clause License](https://opensource.org/licenses/BSD-3-Clause) instead.

This is the Living Standard. Those interested in the patent-review version should view the [Living Standard Review Draft](https://dom.spec.whatwg.org/review-drafts/2026-06/).
