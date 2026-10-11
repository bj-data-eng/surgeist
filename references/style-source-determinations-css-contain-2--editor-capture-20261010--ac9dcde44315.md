Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Containment Module Level 2](https://drafts.csswg.org/css-contain-2/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-10. Source status: Editor’s Draft, 9 October 2026.

Full captured HTML SHA-256: `5d4ad434d4bd9ef30fbf8d52da9ca20c9d5d36dc1c22abd418c7da69bee97384` (843428 bytes). Conversion input SHA-256: `ac9dcde4431520e204a74f984735233f9bcc298a35039ac07b203065bb88d4e4` (223230 bytes). Page revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`.

Retained scope: `contain-property`, `containment-style`. Selected heading sections are complete, bounded at the next equal-or-higher heading. The original header and legal notice are retained. Other clauses remain upstream. Figures remain upstream image links. Multiline alternative descriptions use HTML image attributes with numeric whitespace entities to retain exact source alt text.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table contents are checked against conversion input. Synthetic table headers and span expansion are representation changes. Mathematical expressions are retained as source text; no equation is corrected. This capture preserves standards latitude and unresolved questions; Surgeist-selected policies are recorded separately in issues.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Containment Module Level 2

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 9 October 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-contain-2/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-contain-2/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2022/WD-css-contain-2-20220917/>

<https://www.w3.org/TR/2020/WD-css-contain-2-20201216/>

<https://www.w3.org/TR/2020/WD-css-contain-2-20200603/>

<https://www.w3.org/TR/2019/WD-css-contain-2-20191111/>

<https://www.w3.org/TR/2019/WD-css-contain-2-20191015/>

<strong>Test Suites:</strong>

<https://test.csswg.org/harness/results/css-contain-1_dev/>

<https://wpt.fyi/results/css/css-contain/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-contain-2)

[Inline In Spec](https://drafts.csswg.org/css-contain-2/#issues-index)

<strong>Editors:</strong>

[Tab Atkins](http://xanthir.com/contact/) (Google)

[Florian Rivoal](https://florian.rivoal.net/) (On behalf of Bloomberg)

[Vladimir Levin](mailto:vmpstr@google.com) (Google)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-contain-2/Overview.bs)

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="contain-property"></a>2.  Strong Containment: the <a id="ref-for-propdef-contain②"></a>[contain](#propdef-contain) property[](#contain-property)

Tests

The following tests are crash tests that relate to general usage of this feature but are not particularly tied to any particular normative statement.

- [contain-chrome-thcrash-001.html](https://wpt.fyi/results/css/css-contain/contain-chrome-thcrash-001.html) [(live test)](http://wpt.live/css/css-contain/contain-chrome-thcrash-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-chrome-thcrash-001.html)
- [contain-crash.html](https://wpt.fyi/results/css/css-contain/contain-crash.html) [(live test)](http://wpt.live/css/css-contain/contain-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-crash.html)
- [contain-flexbox-outline.html](https://wpt.fyi/results/css/css-contain/contain-flexbox-outline.html) [(live test)](http://wpt.live/css/css-contain/contain-flexbox-outline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-flexbox-outline.html)
- [contain-nested-crash-001.html](https://wpt.fyi/results/css/css-contain/crashtests/contain-nested-crash-001.html) [(live test)](http://wpt.live/css/css-contain/crashtests/contain-nested-crash-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/crashtests/contain-nested-crash-001.html)
- [contain-nested-crash-002.html](https://wpt.fyi/results/css/css-contain/crashtests/contain-nested-crash-002.html) [(live test)](http://wpt.live/css/css-contain/crashtests/contain-nested-crash-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/crashtests/contain-nested-crash-002.html)
- [contain-nested-crash-003.html](https://wpt.fyi/results/css/css-contain/crashtests/contain-nested-crash-003.html) [(live test)](http://wpt.live/css/css-contain/crashtests/contain-nested-crash-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/crashtests/contain-nested-crash-003.html)
- [contain-nested-crash-004.html](https://wpt.fyi/results/css/css-contain/crashtests/contain-nested-crash-004.html) [(live test)](http://wpt.live/css/css-contain/crashtests/contain-nested-crash-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/crashtests/contain-nested-crash-004.html)
- [contain-nested-relayout-boundary.html](https://wpt.fyi/results/css/css-contain/crashtests/contain-nested-relayout-boundary.html) [(live test)](http://wpt.live/css/css-contain/crashtests/contain-nested-relayout-boundary.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/crashtests/contain-nested-relayout-boundary.html)
- [quote-scoping-shadow-dom-crash.html](https://wpt.fyi/results/css/css-contain/quote-scoping-shadow-dom-crash.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-shadow-dom-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-shadow-dom-crash.html)
- [contain-style-remove-element-crash.html](https://wpt.fyi/results/css/css-contain/contain-style-remove-element-crash.html) [(live test)](http://wpt.live/css/css-contain/contain-style-remove-element-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-remove-element-crash.html)

------------------------------------------------------------------------

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                                                                                                                                                         |
|------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-contain"></a><strong>contain</strong>                                                                                                                                                                                                                                                                                                                                                               |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | none <a id="ref-for-comb-one"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) strict <a id="ref-for-comb-one①"></a>\| content <a id="ref-for-comb-one②"></a>\| \[ \[size <a id="ref-for-comb-one③"></a>\| inline-size\] <a id="ref-for-comb-any"></a>[\|\|](https://drafts.csswg.org/css-values-4/#comb-any) layout <a id="ref-for-comb-any①"></a>\|\| style <a id="ref-for-comb-any②"></a>\|\| paint \] |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | none                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | See [below](#contain-applies)                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | the keyword <a id="ref-for-valdef-contain-none"></a>[none](#valdef-contain-none) or one or more of <a id="ref-for-valdef-contain-size"></a>[size](#valdef-contain-size), <a id="ref-for-valdef-contain-layout"></a>[layout](#valdef-contain-layout), <a id="ref-for-valdef-contain-style"></a>[style](#valdef-contain-style), <a id="ref-for-valdef-contain-paint"></a>[paint](#valdef-contain-paint)              |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | not animatable                                                                                                                                                                                                                                                                                                                                                                                                     |

Tests

- [contain-animation-001.html](https://wpt.fyi/results/css/css-contain/contain-animation-001.html) [(live test)](http://wpt.live/css/css-contain/contain-animation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-animation-001.html)
- [inheritance.html](https://wpt.fyi/results/css/css-contain/inheritance.html) [(live test)](http://wpt.live/css/css-contain/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/inheritance.html)
- [contain-computed.html](https://wpt.fyi/results/css/css-contain/parsing/contain-computed.html) [(live test)](http://wpt.live/css/css-contain/parsing/contain-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/parsing/contain-computed.html)
- [contain-invalid.html](https://wpt.fyi/results/css/css-contain/parsing/contain-invalid.html) [(live test)](http://wpt.live/css/css-contain/parsing/contain-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/parsing/contain-invalid.html)
- [contain-valid.html](https://wpt.fyi/results/css/css-contain/parsing/contain-valid.html) [(live test)](http://wpt.live/css/css-contain/parsing/contain-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/parsing/contain-valid.html)
- [contain-computed-children.html](https://wpt.fyi/results/css/css-contain/parsing/contain-computed-children.html) [(live test)](http://wpt.live/css/css-contain/parsing/contain-computed-children.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/parsing/contain-computed-children.html)
- [contain-layout-size-003.html](https://wpt.fyi/results/css/css-contain/contain-layout-size-003.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-size-003.html)
- [contain-paint-size-001.html](https://wpt.fyi/results/css/css-contain/contain-paint-size-001.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-size-001.html)
- [contain-paint-size-002.html](https://wpt.fyi/results/css/css-contain/contain-paint-size-002.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-size-002.html)
- [contain-paint-size-003.html](https://wpt.fyi/results/css/css-contain/contain-paint-size-003.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-size-003.html)

<em>User agents are expected to support this property on all media, including non-visual ones.</em>

The <a id="ref-for-propdef-contain③"></a>[contain](#propdef-contain) property allows an author to indicate that an element and its contents are, as much as possible, <em>independent</em> of the rest of the document tree. This allows user agents to utilize much stronger optimizations when rendering a page using <a id="ref-for-propdef-contain④"></a>contain properly, and allows authors to be confident that their page won’t accidentally fall into a slow code path due to an innocuous change.

<strong><a id="valdef-contain-none"></a><strong>none</strong></strong>

This value indicates that the property has no effect. The element renders as normal, with no containment effects applied.

<strong><a id="valdef-contain-strict"></a><strong>strict</strong></strong>

This value computes to size layout paint style, and thus turns on all forms of <a id="ref-for-containment"></a>[containment](https://drafts.csswg.org/css-contain-2/#containment) for the element.

Tests

- [contain-strict-001.html](https://wpt.fyi/results/css/css-contain/contain-strict-001.html) [(live test)](http://wpt.live/css/css-contain/contain-strict-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-strict-001.html)
- [contain-strict-002.html](https://wpt.fyi/results/css/css-contain/contain-strict-002.html) [(live test)](http://wpt.live/css/css-contain/contain-strict-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-strict-002.html)
- [contain-strict-003.html](https://wpt.fyi/results/css/css-contain/contain-strict-003.html) [(live test)](http://wpt.live/css/css-contain/contain-strict-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-strict-003.html)
- [contain-strict-011.html](https://wpt.fyi/results/css/css-contain/contain-strict-011.html) [(live test)](http://wpt.live/css/css-contain/contain-strict-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-strict-011.html)
- [contain-flexbox-outline.html](https://wpt.fyi/results/css/css-contain/contain-flexbox-outline.html) [(live test)](http://wpt.live/css/css-contain/contain-flexbox-outline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-flexbox-outline.html)

<strong><a id="valdef-contain-content"></a><strong>content</strong></strong>

This value computes to layout paint style, and thus turns on all forms of <a id="ref-for-containment①"></a>[containment](https://drafts.csswg.org/css-contain-2/#containment) <em>except</em> <a id="ref-for-size-containment"></a>[size containment](https://drafts.csswg.org/css-contain-2/#size-containment) for the element.

Tests

- [contain-content-001.html](https://wpt.fyi/results/css/css-contain/contain-content-001.html) [(live test)](http://wpt.live/css/css-contain/contain-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-content-001.html)
- [contain-content-002.html](https://wpt.fyi/results/css/css-contain/contain-content-002.html) [(live test)](http://wpt.live/css/css-contain/contain-content-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-content-002.html)
- [contain-content-003.html](https://wpt.fyi/results/css/css-contain/contain-content-003.html) [(live test)](http://wpt.live/css/css-contain/contain-content-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-content-003.html)
- [contain-content-004.html](https://wpt.fyi/results/css/css-contain/contain-content-004.html) [(live test)](http://wpt.live/css/css-contain/contain-content-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-content-004.html)
- [contain-content-011.html](https://wpt.fyi/results/css/css-contain/contain-content-011.html) [(live test)](http://wpt.live/css/css-contain/contain-content-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-content-011.html)

Note: <a id="ref-for-propdef-contain⑤"></a>[contain: content](#propdef-contain) is reasonably "safe" to apply widely; its effects are fairly minor in practice, and most content won’t run afoul of its restrictions. However, because it doesn’t apply <a id="ref-for-size-containment①"></a>[size containment](https://drafts.csswg.org/css-contain-2/#size-containment), the element can still respond to the size of its contents, which can cause layout-invalidation to percolate further up the tree than desired. Use <a id="ref-for-propdef-contain⑥"></a>contain: strict when possible, to gain as much containment as you can.

<strong><a id="valdef-contain-size"></a><strong>size</strong></strong>

The value turns on <a id="ref-for-size-containment②"></a>[size containment](https://drafts.csswg.org/css-contain-2/#size-containment) for the element. This ensures that the <a id="ref-for-size-containment-box"></a>[containment box](https://drafts.csswg.org/css-contain-2/#size-containment-box) can be laid out without needing to examine its descendants.

Tests

- [contain-size-001.html](https://wpt.fyi/results/css/css-contain/contain-size-001.html) [(live test)](http://wpt.live/css/css-contain/contain-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-001.html)
- [contain-size-002.html](https://wpt.fyi/results/css/css-contain/contain-size-002.html) [(live test)](http://wpt.live/css/css-contain/contain-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-002.html)
- [contain-size-003.html](https://wpt.fyi/results/css/css-contain/contain-size-003.html) [(live test)](http://wpt.live/css/css-contain/contain-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-003.html)
- [contain-size-004.html](https://wpt.fyi/results/css/css-contain/contain-size-004.html) [(live test)](http://wpt.live/css/css-contain/contain-size-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-004.html)
- [contain-size-005.html](https://wpt.fyi/results/css/css-contain/contain-size-005.html) [(live test)](http://wpt.live/css/css-contain/contain-size-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-005.html)
- [contain-size-006.html](https://wpt.fyi/results/css/css-contain/contain-size-006.html) [(live test)](http://wpt.live/css/css-contain/contain-size-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-006.html)
- [contain-size-007.html](https://wpt.fyi/results/css/css-contain/contain-size-007.html) [(live test)](http://wpt.live/css/css-contain/contain-size-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-007.html)
- [contain-size-008.html](https://wpt.fyi/results/css/css-contain/contain-size-008.html) [(live test)](http://wpt.live/css/css-contain/contain-size-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-008.html)
- [contain-size-009.html](https://wpt.fyi/results/css/css-contain/contain-size-009.html) [(live test)](http://wpt.live/css/css-contain/contain-size-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-009.html)
- [contain-size-010.html](https://wpt.fyi/results/css/css-contain/contain-size-010.html) [(live test)](http://wpt.live/css/css-contain/contain-size-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-010.html)
- [contain-size-011.html](https://wpt.fyi/results/css/css-contain/contain-size-011.html) [(live test)](http://wpt.live/css/css-contain/contain-size-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-011.html)
- [contain-size-012.html](https://wpt.fyi/results/css/css-contain/contain-size-012.html) [(live test)](http://wpt.live/css/css-contain/contain-size-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-012.html)
- [contain-size-012b.html](https://wpt.fyi/results/css/css-contain/contain-size-012b.html) [(live test)](http://wpt.live/css/css-contain/contain-size-012b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-012b.html)
- [contain-size-013.html](https://wpt.fyi/results/css/css-contain/contain-size-013.html) [(live test)](http://wpt.live/css/css-contain/contain-size-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-013.html)
- [contain-size-021.html](https://wpt.fyi/results/css/css-contain/contain-size-021.html) [(live test)](http://wpt.live/css/css-contain/contain-size-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-021.html)
- [contain-size-023.html](https://wpt.fyi/results/css/css-contain/contain-size-023.html) [(live test)](http://wpt.live/css/css-contain/contain-size-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-023.html)
- [contain-size-025.html](https://wpt.fyi/results/css/css-contain/contain-size-025.html) [(live test)](http://wpt.live/css/css-contain/contain-size-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-025.html)
- [contain-size-027.html](https://wpt.fyi/results/css/css-contain/contain-size-027.html) [(live test)](http://wpt.live/css/css-contain/contain-size-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-027.html)
- [contain-size-041.html](https://wpt.fyi/results/css/css-contain/contain-size-041.html) [(live test)](http://wpt.live/css/css-contain/contain-size-041.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-041.html)
- [contain-size-042.html](https://wpt.fyi/results/css/css-contain/contain-size-042.html) [(live test)](http://wpt.live/css/css-contain/contain-size-042.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-042.html)
- [contain-size-051.html](https://wpt.fyi/results/css/css-contain/contain-size-051.html) [(live test)](http://wpt.live/css/css-contain/contain-size-051.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-051.html)
- [contain-size-052.html](https://wpt.fyi/results/css/css-contain/contain-size-052.html) [(live test)](http://wpt.live/css/css-contain/contain-size-052.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-052.html)
- [contain-size-056.html](https://wpt.fyi/results/css/css-contain/contain-size-056.html) [(live test)](http://wpt.live/css/css-contain/contain-size-056.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-056.html)
- [contain-size-061.html](https://wpt.fyi/results/css/css-contain/contain-size-061.html) [(live test)](http://wpt.live/css/css-contain/contain-size-061.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-061.html)
- [contain-size-062.html](https://wpt.fyi/results/css/css-contain/contain-size-062.html) [(live test)](http://wpt.live/css/css-contain/contain-size-062.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-062.html)
- [contain-size-063.html](https://wpt.fyi/results/css/css-contain/contain-size-063.html) [(live test)](http://wpt.live/css/css-contain/contain-size-063.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-063.html)
- [contain-size-baseline-001.html](https://wpt.fyi/results/css/css-contain/contain-size-baseline-001.html) [(live test)](http://wpt.live/css/css-contain/contain-size-baseline-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-baseline-001.html)
- [contain-size-borders.html](https://wpt.fyi/results/css/css-contain/contain-size-borders.html) [(live test)](http://wpt.live/css/css-contain/contain-size-borders.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-borders.html)
- [contain-size-breaks-001.html](https://wpt.fyi/results/css/css-contain/contain-size-breaks-001.html) [(live test)](http://wpt.live/css/css-contain/contain-size-breaks-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-breaks-001.html)
- [contain-size-button-001.html](https://wpt.fyi/results/css/css-contain/contain-size-button-001.html) [(live test)](http://wpt.live/css/css-contain/contain-size-button-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-button-001.html)
- [contain-size-fieldset-001.html](https://wpt.fyi/results/css/css-contain/contain-size-fieldset-001.html) [(live test)](http://wpt.live/css/css-contain/contain-size-fieldset-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-fieldset-001.html)
- [contain-size-fieldset-002.html](https://wpt.fyi/results/css/css-contain/contain-size-fieldset-002.html) [(live test)](http://wpt.live/css/css-contain/contain-size-fieldset-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-fieldset-002.html)
- [contain-size-flexbox-001.html](https://wpt.fyi/results/css/css-contain/contain-size-flexbox-001.html) [(live test)](http://wpt.live/css/css-contain/contain-size-flexbox-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-flexbox-001.html)
- [contain-size-flexbox-002.html](https://wpt.fyi/results/css/css-contain/contain-size-flexbox-002.html) [(live test)](http://wpt.live/css/css-contain/contain-size-flexbox-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-flexbox-002.html)
- [contain-size-grid-001.html](https://wpt.fyi/results/css/css-contain/contain-size-grid-001.html) [(live test)](http://wpt.live/css/css-contain/contain-size-grid-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-grid-001.html)
- [contain-size-grid-002.html](https://wpt.fyi/results/css/css-contain/contain-size-grid-002.html) [(live test)](http://wpt.live/css/css-contain/contain-size-grid-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-grid-002.html)
- [contain-size-grid-003.html](https://wpt.fyi/results/css/css-contain/contain-size-grid-003.html) [(live test)](http://wpt.live/css/css-contain/contain-size-grid-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-grid-003.html)
- [contain-size-grid-004.html](https://wpt.fyi/results/css/css-contain/contain-size-grid-004.html) [(live test)](http://wpt.live/css/css-contain/contain-size-grid-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-grid-004.html)
- [contain-size-grid-005.html](https://wpt.fyi/results/css/css-contain/contain-size-grid-005.html) [(live test)](http://wpt.live/css/css-contain/contain-size-grid-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-grid-005.html)
- [contain-size-grid-006.html](https://wpt.fyi/results/css/css-contain/contain-size-grid-006.html) [(live test)](http://wpt.live/css/css-contain/contain-size-grid-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-grid-006.html)
- [contain-size-monolithic-001.html](https://wpt.fyi/results/css/css-contain/contain-size-monolithic-001.html) [(live test)](http://wpt.live/css/css-contain/contain-size-monolithic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-monolithic-001.html)
- [contain-size-monolithic-002.html](https://wpt.fyi/results/css/css-contain/contain-size-monolithic-002.html) [(live test)](http://wpt.live/css/css-contain/contain-size-monolithic-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-monolithic-002.html)
- [contain-size-multicol-001.html](https://wpt.fyi/results/css/css-contain/contain-size-multicol-001.html) [(live test)](http://wpt.live/css/css-contain/contain-size-multicol-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-multicol-001.html)
- [contain-size-multicol-as-flex-item.html](https://wpt.fyi/results/css/css-contain/contain-size-multicol-as-flex-item.html) [(live test)](http://wpt.live/css/css-contain/contain-size-multicol-as-flex-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-multicol-as-flex-item.html)
- [contain-size-replaced-001.html](https://wpt.fyi/results/css/css-contain/contain-size-replaced-001.html) [(live test)](http://wpt.live/css/css-contain/contain-size-replaced-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-replaced-001.html)
- [contain-size-replaced-002.html](https://wpt.fyi/results/css/css-contain/contain-size-replaced-002.html) [(live test)](http://wpt.live/css/css-contain/contain-size-replaced-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-replaced-002.html)
- [contain-size-replaced-003a.html](https://wpt.fyi/results/css/css-contain/contain-size-replaced-003a.html) [(live test)](http://wpt.live/css/css-contain/contain-size-replaced-003a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-replaced-003a.html)
- [contain-size-replaced-003b.html](https://wpt.fyi/results/css/css-contain/contain-size-replaced-003b.html) [(live test)](http://wpt.live/css/css-contain/contain-size-replaced-003b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-replaced-003b.html)
- [contain-size-replaced-003c.html](https://wpt.fyi/results/css/css-contain/contain-size-replaced-003c.html) [(live test)](http://wpt.live/css/css-contain/contain-size-replaced-003c.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-replaced-003c.html)
- [contain-size-replaced-004.html](https://wpt.fyi/results/css/css-contain/contain-size-replaced-004.html) [(live test)](http://wpt.live/css/css-contain/contain-size-replaced-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-replaced-004.html)
- [contain-size-replaced-005.html](https://wpt.fyi/results/css/css-contain/contain-size-replaced-005.html) [(live test)](http://wpt.live/css/css-contain/contain-size-replaced-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-replaced-005.html)
- [contain-size-replaced-006.html](https://wpt.fyi/results/css/css-contain/contain-size-replaced-006.html) [(live test)](http://wpt.live/css/css-contain/contain-size-replaced-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-replaced-006.html)
- [contain-size-replaced-007.html](https://wpt.fyi/results/css/css-contain/contain-size-replaced-007.html) [(live test)](http://wpt.live/css/css-contain/contain-size-replaced-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-replaced-007.html)
- [contain-size-select-001.html](https://wpt.fyi/results/css/css-contain/contain-size-select-001.html) [(live test)](http://wpt.live/css/css-contain/contain-size-select-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-select-001.html)
- [contain-size-select-002.html](https://wpt.fyi/results/css/css-contain/contain-size-select-002.html) [(live test)](http://wpt.live/css/css-contain/contain-size-select-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-select-002.html)
- [contain-size-scrollbars-001.html](https://wpt.fyi/results/css/css-contain/contain-size-scrollbars-001.html) [(live test)](http://wpt.live/css/css-contain/contain-size-scrollbars-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-scrollbars-001.html)
- [contain-size-scrollbars-002.html](https://wpt.fyi/results/css/css-contain/contain-size-scrollbars-002.html) [(live test)](http://wpt.live/css/css-contain/contain-size-scrollbars-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-scrollbars-002.html)
- [contain-size-scrollbars-003.html](https://wpt.fyi/results/css/css-contain/contain-size-scrollbars-003.html) [(live test)](http://wpt.live/css/css-contain/contain-size-scrollbars-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-scrollbars-003.html)
- [contain-size-scrollbars-004.html](https://wpt.fyi/results/css/css-contain/contain-size-scrollbars-004.html) [(live test)](http://wpt.live/css/css-contain/contain-size-scrollbars-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-scrollbars-004.html)
- [contain-layout-size-003.html](https://wpt.fyi/results/css/css-contain/contain-layout-size-003.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-size-003.html)
- [contain-paint-size-001.html](https://wpt.fyi/results/css/css-contain/contain-paint-size-001.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-size-001.html)
- [contain-paint-size-002.html](https://wpt.fyi/results/css/css-contain/contain-paint-size-002.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-size-002.html)
- [contain-paint-size-003.html](https://wpt.fyi/results/css/css-contain/contain-paint-size-003.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-size-003.html)
- [contain-size-removed.html](https://wpt.fyi/results/css/css-contain/contain-size-removed.html) [(live test)](http://wpt.live/css/css-contain/contain-size-removed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-removed.html)
- [contain-size-dynamic-001.html](https://wpt.fyi/results/css/css-contain/contain-size-dynamic-001.html) [(live test)](http://wpt.live/css/css-contain/contain-size-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-dynamic-001.html)
- [contain-size-grid-indefinite-height-min-height-flex-row.html](https://wpt.fyi/results/css/css-contain/contain-size-grid-indefinite-height-min-height-flex-row.html) [(live test)](http://wpt.live/css/css-contain/contain-size-grid-indefinite-height-min-height-flex-row.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-grid-indefinite-height-min-height-flex-row.html)
- [contain-size-grid-stretches-auto-rows.html](https://wpt.fyi/results/css/css-contain/contain-size-grid-stretches-auto-rows.html) [(live test)](http://wpt.live/css/css-contain/contain-size-grid-stretches-auto-rows.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-size-grid-stretches-auto-rows.html)

<strong><a id="valdef-contain-inline-size"></a><strong>inline-size</strong></strong>

This value turns on <a id="ref-for-inline-size-containment"></a>[inline-size containment](https://drafts.csswg.org/css-contain-2/#inline-size-containment) for the element. This prevents the <a id="ref-for-inline-size"></a>[inline-size](https://drafts.csswg.org/css-writing-modes-4/#inline-size) of its <a id="ref-for-principal-box"></a>[principal box](https://drafts.csswg.org/css-display-4/#principal-box) from directly depending on its contents.

Tests

- [contain-inline-size-bfc-floats-001.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-bfc-floats-001.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-bfc-floats-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-bfc-floats-001.html)
- [contain-inline-size-bfc-floats-002.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-bfc-floats-002.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-bfc-floats-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-bfc-floats-002.html)
- [contain-inline-size-fieldset.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-fieldset.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-fieldset.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-fieldset.html)
- [contain-inline-size-flex.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-flex.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-flex.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-flex.html)
- [contain-inline-size-flexitem.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-flexitem.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-flexitem.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-flexitem.html)
- [contain-inline-size-grid.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-grid.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-grid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-grid.html)
- [contain-inline-size-intrinsic.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-intrinsic.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-intrinsic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-intrinsic.html)
- [contain-inline-size-legend.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-legend.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-legend.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-legend.html)
- [contain-inline-size-multicol.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-multicol.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-multicol.html)
- [contain-inline-size-regular-container.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-regular-container.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-regular-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-regular-container.html)
- [contain-inline-size-removed.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-removed.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-removed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-removed.html)
- [contain-inline-size-replaced.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-replaced.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-replaced.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-replaced.html)
- [contain-inline-size-table.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-table.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-table.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-table.html)
- [contain-inline-size-vertical-rl-.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-vertical-rl-.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-vertical-rl-.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-vertical-rl-.html)
- [contain-inline-size-grid-indefinite-height-min-height-flex-row.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-grid-indefinite-height-min-height-flex-row.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-grid-indefinite-height-min-height-flex-row.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-grid-indefinite-height-min-height-flex-row.html)
- [contain-inline-size-grid-stretches-auto-rows.html](https://wpt.fyi/results/css/css-contain/contain-inline-size-grid-stretches-auto-rows.html) [(live test)](http://wpt.live/css/css-contain/contain-inline-size-grid-stretches-auto-rows.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-inline-size-grid-stretches-auto-rows.html)

Note: There can still be indirect dependencies, see [§ 3.2 Inline-Size Containment](https://drafts.csswg.org/css-contain-2/#containment-inline-size).

<strong><a id="valdef-contain-layout"></a><strong>layout</strong></strong>

This value turns on <a id="ref-for-layout-containment"></a>[layout containment](https://drafts.csswg.org/css-contain-2/#layout-containment) for the element. This ensures that the <a id="ref-for-layout-containment-box"></a>[containment box](https://drafts.csswg.org/css-contain-2/#layout-containment-box) is <em>totally opaque</em> for layout purposes; nothing outside can affect its internal layout, and vice versa.

Tests

- [contain-layout-001.html](https://wpt.fyi/results/css/css-contain/contain-layout-001.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-001.html)
- [contain-layout-002.html](https://wpt.fyi/results/css/css-contain/contain-layout-002.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-002.html)
- [contain-layout-003.html](https://wpt.fyi/results/css/css-contain/contain-layout-003.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-003.html)
- [contain-layout-004.html](https://wpt.fyi/results/css/css-contain/contain-layout-004.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-004.html)
- [contain-layout-005.html](https://wpt.fyi/results/css/css-contain/contain-layout-005.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-005.html)
- [contain-layout-006.html](https://wpt.fyi/results/css/css-contain/contain-layout-006.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-006.html)
- [contain-layout-007.html](https://wpt.fyi/results/css/css-contain/contain-layout-007.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-007.html)
- [contain-layout-009.html](https://wpt.fyi/results/css/css-contain/contain-layout-009.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-009.html)
- [contain-layout-010.html](https://wpt.fyi/results/css/css-contain/contain-layout-010.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-010.html)
- [contain-layout-011.html](https://wpt.fyi/results/css/css-contain/contain-layout-011.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-011.html)
- [contain-layout-012.html](https://wpt.fyi/results/css/css-contain/contain-layout-012.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-012.html)
- [contain-layout-013.html](https://wpt.fyi/results/css/css-contain/contain-layout-013.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-013.html)
- [contain-layout-014.html](https://wpt.fyi/results/css/css-contain/contain-layout-014.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-014.html)
- [contain-layout-016.html](https://wpt.fyi/results/css/css-contain/contain-layout-016.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-016.html)
- [contain-layout-017.html](https://wpt.fyi/results/css/css-contain/contain-layout-017.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-017.html)
- [contain-layout-018.html](https://wpt.fyi/results/css/css-contain/contain-layout-018.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-018.html)
- [contain-layout-019.html](https://wpt.fyi/results/css/css-contain/contain-layout-019.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-019.html)
- [contain-layout-020.html](https://wpt.fyi/results/css/css-contain/contain-layout-020.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-020.html)
- [contain-layout-021.html](https://wpt.fyi/results/css/css-contain/contain-layout-021.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-021.html)
- [contain-layout-baseline-001.html](https://wpt.fyi/results/css/css-contain/contain-layout-baseline-001.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-baseline-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-baseline-001.html)
- [contain-layout-baseline-002.html](https://wpt.fyi/results/css/css-contain/contain-layout-baseline-002.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-baseline-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-baseline-002.html)
- [contain-layout-baseline-003.html](https://wpt.fyi/results/css/css-contain/contain-layout-baseline-003.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-baseline-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-baseline-003.html)
- [contain-layout-baseline-004.html](https://wpt.fyi/results/css/css-contain/contain-layout-baseline-004.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-baseline-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-baseline-004.html)
- [contain-layout-baseline-005.html](https://wpt.fyi/results/css/css-contain/contain-layout-baseline-005.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-baseline-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-baseline-005.html)
- [contain-layout-breaks-001.html](https://wpt.fyi/results/css/css-contain/contain-layout-breaks-001.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-breaks-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-breaks-001.html)
- [contain-layout-breaks-002.html](https://wpt.fyi/results/css/css-contain/contain-layout-breaks-002.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-breaks-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-breaks-002.html)
- [contain-layout-cell-001.html](https://wpt.fyi/results/css/css-contain/contain-layout-cell-001.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-cell-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-cell-001.html)
- [contain-layout-cell-002.html](https://wpt.fyi/results/css/css-contain/contain-layout-cell-002.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-cell-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-cell-002.html)
- [contain-layout-flexbox-001.html](https://wpt.fyi/results/css/css-contain/contain-layout-flexbox-001.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-flexbox-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-flexbox-001.html)
- [contain-layout-grid-001.html](https://wpt.fyi/results/css/css-contain/contain-layout-grid-001.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-grid-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-grid-001.html)
- [contain-layout-ifc-022.html](https://wpt.fyi/results/css/css-contain/contain-layout-ifc-022.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-ifc-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-ifc-022.html)
- [contain-layout-independent-formatting-context-001.html](https://wpt.fyi/results/css/css-contain/contain-layout-independent-formatting-context-001.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-independent-formatting-context-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-independent-formatting-context-001.html)
- [contain-layout-independent-formatting-context-002.html](https://wpt.fyi/results/css/css-contain/contain-layout-independent-formatting-context-002.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-independent-formatting-context-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-independent-formatting-context-002.html)
- [contain-layout-independent-formatting-context-003.html](https://wpt.fyi/results/css/css-contain/contain-layout-independent-formatting-context-003.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-independent-formatting-context-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-independent-formatting-context-003.html)
- [contain-layout-ink-overflow-013.html](https://wpt.fyi/results/css/css-contain/contain-layout-ink-overflow-013.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-ink-overflow-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-ink-overflow-013.html)
- [contain-layout-ink-overflow-014.html](https://wpt.fyi/results/css/css-contain/contain-layout-ink-overflow-014.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-ink-overflow-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-ink-overflow-014.html)
- [contain-layout-ink-overflow-015.html](https://wpt.fyi/results/css/css-contain/contain-layout-ink-overflow-015.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-ink-overflow-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-ink-overflow-015.html)
- [contain-layout-ink-overflow-016.html](https://wpt.fyi/results/css/css-contain/contain-layout-ink-overflow-016.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-ink-overflow-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-ink-overflow-016.html)
- [contain-layout-ink-overflow-017.html](https://wpt.fyi/results/css/css-contain/contain-layout-ink-overflow-017.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-ink-overflow-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-ink-overflow-017.html)
- [contain-layout-ink-overflow-018.html](https://wpt.fyi/results/css/css-contain/contain-layout-ink-overflow-018.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-ink-overflow-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-ink-overflow-018.html)
- [contain-layout-ink-overflow-019.html](https://wpt.fyi/results/css/css-contain/contain-layout-ink-overflow-019.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-ink-overflow-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-ink-overflow-019.html)
- [contain-layout-ink-overflow-020.html](https://wpt.fyi/results/css/css-contain/contain-layout-ink-overflow-020.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-ink-overflow-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-ink-overflow-020.html)
- [contain-layout-size-003.html](https://wpt.fyi/results/css/css-contain/contain-layout-size-003.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-size-003.html)
- [contain-subgrid-001.html](https://wpt.fyi/results/css/css-contain/contain-subgrid-001.html) [(live test)](http://wpt.live/css/css-contain/contain-subgrid-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-subgrid-001.html)
- [contain-layout-dynamic-001.html](https://wpt.fyi/results/css/css-contain/contain-layout-dynamic-001.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-dynamic-001.html)
- [contain-layout-dynamic-004.html](https://wpt.fyi/results/css/css-contain/contain-layout-dynamic-004.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-dynamic-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-dynamic-004.html)
- [contain-layout-dynamic-005.html](https://wpt.fyi/results/css/css-contain/contain-layout-dynamic-005.html) [(live test)](http://wpt.live/css/css-contain/contain-layout-dynamic-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-layout-dynamic-005.html)

<strong><a id="valdef-contain-style"></a><strong>style</strong></strong>

This value turns on <a id="ref-for-style-containment"></a>[style containment](#style-containment) for the element. This ensures that, for properties which can have effects on more than just an element and its descendants, those effects don’t escape the element.

Tests

- [contain-style-baseline-001.html](https://wpt.fyi/results/css/css-contain/contain-style-baseline-001.html) [(live test)](http://wpt.live/css/css-contain/contain-style-baseline-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-baseline-001.html)
- [contain-style-breaks-001.html](https://wpt.fyi/results/css/css-contain/contain-style-breaks-001.html) [(live test)](http://wpt.live/css/css-contain/contain-style-breaks-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-breaks-001.html)
- [contain-style-breaks-002.html](https://wpt.fyi/results/css/css-contain/contain-style-breaks-002.html) [(live test)](http://wpt.live/css/css-contain/contain-style-breaks-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-breaks-002.html)
- [contain-style-breaks-003.html](https://wpt.fyi/results/css/css-contain/contain-style-breaks-003.html) [(live test)](http://wpt.live/css/css-contain/contain-style-breaks-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-breaks-003.html)
- [contain-style-breaks-004.html](https://wpt.fyi/results/css/css-contain/contain-style-breaks-004.html) [(live test)](http://wpt.live/css/css-contain/contain-style-breaks-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-breaks-004.html)
- [contain-style-breaks-005.html](https://wpt.fyi/results/css/css-contain/contain-style-breaks-005.html) [(live test)](http://wpt.live/css/css-contain/contain-style-breaks-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-breaks-005.html)
- [contain-style-counters-001.html](https://wpt.fyi/results/css/css-contain/contain-style-counters-001.html) [(live test)](http://wpt.live/css/css-contain/contain-style-counters-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-counters-001.html)
- [contain-style-counters-002.html](https://wpt.fyi/results/css/css-contain/contain-style-counters-002.html) [(live test)](http://wpt.live/css/css-contain/contain-style-counters-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-counters-002.html)
- [contain-style-counters-003.html](https://wpt.fyi/results/css/css-contain/contain-style-counters-003.html) [(live test)](http://wpt.live/css/css-contain/contain-style-counters-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-counters-003.html)
- [contain-style-counters-004.html](https://wpt.fyi/results/css/css-contain/contain-style-counters-004.html) [(live test)](http://wpt.live/css/css-contain/contain-style-counters-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-counters-004.html)
- [contain-style-counters-005.html](https://wpt.fyi/results/css/css-contain/contain-style-counters-005.html) [(live test)](http://wpt.live/css/css-contain/contain-style-counters-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-counters-005.html)
- [contain-style-ol-ordinal-li-container.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-li-container.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-li-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-li-container.html)
- [contain-style-ol-ordinal-pseudo-reversed.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-pseudo-reversed.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-pseudo-reversed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-pseudo-reversed.html)
- [contain-style-ol-ordinal-pseudo.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-pseudo.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-pseudo.html)
- [contain-style-ol-ordinal-reversed.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-reversed.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-reversed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-reversed.html)
- [contain-style-ol-ordinal-start-reversed.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-start-reversed.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-start-reversed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-start-reversed.html)
- [contain-style-ol-ordinal-start.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-start.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-start.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-start.html)
- [contain-style-ol-ordinal.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal.html)
- [counter-scoping-001.html](https://wpt.fyi/results/css/css-contain/counter-scoping-001.html) [(live test)](http://wpt.live/css/css-contain/counter-scoping-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/counter-scoping-001.html)
- [counter-scoping-002.html](https://wpt.fyi/results/css/css-contain/counter-scoping-002.html) [(live test)](http://wpt.live/css/css-contain/counter-scoping-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/counter-scoping-002.html)
- [counter-scoping-003.html](https://wpt.fyi/results/css/css-contain/counter-scoping-003.html) [(live test)](http://wpt.live/css/css-contain/counter-scoping-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/counter-scoping-003.html)
- [counter-scoping-004.html](https://wpt.fyi/results/css/css-contain/counter-scoping-004.html) [(live test)](http://wpt.live/css/css-contain/counter-scoping-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/counter-scoping-004.html)
- [quote-scoping-001.html](https://wpt.fyi/results/css/css-contain/quote-scoping-001.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-001.html)
- [quote-scoping-002.html](https://wpt.fyi/results/css/css-contain/quote-scoping-002.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-002.html)
- [quote-scoping-003.html](https://wpt.fyi/results/css/css-contain/quote-scoping-003.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-003.html)
- [quote-scoping-004.html](https://wpt.fyi/results/css/css-contain/quote-scoping-004.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-004.html)
- [quote-scoping-invalidation-001.html](https://wpt.fyi/results/css/css-contain/quote-scoping-invalidation-001.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-invalidation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-invalidation-001.html)
- [quote-scoping-invalidation-002.html](https://wpt.fyi/results/css/css-contain/quote-scoping-invalidation-002.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-invalidation-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-invalidation-002.html)
- [quote-scoping-invalidation-003.html](https://wpt.fyi/results/css/css-contain/quote-scoping-invalidation-003.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-invalidation-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-invalidation-003.html)
- [quote-scoping-invalidation-004.html](https://wpt.fyi/results/css/css-contain/quote-scoping-invalidation-004.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-invalidation-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-invalidation-004.html)
- [contain-style-dynamic-001.html](https://wpt.fyi/results/css/css-contain/contain-style-dynamic-001.html) [(live test)](http://wpt.live/css/css-contain/contain-style-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-dynamic-001.html)

<strong><a id="valdef-contain-paint"></a><strong>paint</strong></strong>

This value turns on <a id="ref-for-paint-containment"></a>[paint containment](https://drafts.csswg.org/css-contain-2/#paint-containment) for the element. This ensures that the descendants of the <a id="ref-for-paint-containment-box"></a>[containment box](https://drafts.csswg.org/css-contain-2/#paint-containment-box) don’t display outside its bounds, so if an element is off-screen or otherwise not visible, its descendants are also guaranteed to be not visible.

Tests

- [contain-paint-001.html](https://wpt.fyi/results/css/css-contain/contain-paint-001.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-001.html)
- [contain-paint-002.html](https://wpt.fyi/results/css/css-contain/contain-paint-002.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-002.html)
- [contain-paint-004.html](https://wpt.fyi/results/css/css-contain/contain-paint-004.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-004.html)
- [contain-paint-005.html](https://wpt.fyi/results/css/css-contain/contain-paint-005.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-005.html)
- [contain-paint-006.html](https://wpt.fyi/results/css/css-contain/contain-paint-006.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-006.html)
- [contain-paint-007.html](https://wpt.fyi/results/css/css-contain/contain-paint-007.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-007.html)
- [contain-paint-008.html](https://wpt.fyi/results/css/css-contain/contain-paint-008.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-008.html)
- [contain-paint-009.html](https://wpt.fyi/results/css/css-contain/contain-paint-009.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-009.html)
- [contain-paint-010.html](https://wpt.fyi/results/css/css-contain/contain-paint-010.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-010.html)
- [contain-paint-011.html](https://wpt.fyi/results/css/css-contain/contain-paint-011.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-011.html)
- [contain-paint-012.html](https://wpt.fyi/results/css/css-contain/contain-paint-012.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-012.html)
- [contain-paint-014.html](https://wpt.fyi/results/css/css-contain/contain-paint-014.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-014.html)
- [contain-paint-015.html](https://wpt.fyi/results/css/css-contain/contain-paint-015.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-015.html)
- [contain-paint-016.html](https://wpt.fyi/results/css/css-contain/contain-paint-016.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-016.html)
- [contain-paint-017.html](https://wpt.fyi/results/css/css-contain/contain-paint-017.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-017.html)
- [contain-paint-018.html](https://wpt.fyi/results/css/css-contain/contain-paint-018.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-018.html)
- [contain-paint-019.html](https://wpt.fyi/results/css/css-contain/contain-paint-019.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-019.html)
- [contain-paint-020.html](https://wpt.fyi/results/css/css-contain/contain-paint-020.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-020.html)
- [contain-paint-021.html](https://wpt.fyi/results/css/css-contain/contain-paint-021.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-021.html)
- [contain-paint-022.html](https://wpt.fyi/results/css/css-contain/contain-paint-022.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-022.html)
- [contain-paint-023.html](https://wpt.fyi/results/css/css-contain/contain-paint-023.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-023.html)
- [contain-paint-024.html](https://wpt.fyi/results/css/css-contain/contain-paint-024.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-024.html)
- [contain-paint-025.html](https://wpt.fyi/results/css/css-contain/contain-paint-025.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-025.html)
- [contain-paint-047.html](https://wpt.fyi/results/css/css-contain/contain-paint-047.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-047.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-047.html)
- [contain-paint-048.html](https://wpt.fyi/results/css/css-contain/contain-paint-048.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-048.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-048.html)
- [contain-paint-baseline-001.html](https://wpt.fyi/results/css/css-contain/contain-paint-baseline-001.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-baseline-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-baseline-001.html)
- [contain-paint-cell-001.html](https://wpt.fyi/results/css/css-contain/contain-paint-cell-001.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-cell-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-cell-001.html)
- [contain-paint-cell-002.html](https://wpt.fyi/results/css/css-contain/contain-paint-cell-002.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-cell-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-cell-002.html)
- [contain-paint-clip-011.html](https://wpt.fyi/results/css/css-contain/contain-paint-clip-011.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-clip-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-clip-011.html)
- [contain-paint-clip-012.html](https://wpt.fyi/results/css/css-contain/contain-paint-clip-012.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-clip-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-clip-012.html)
- [contain-paint-clip-013.html](https://wpt.fyi/results/css/css-contain/contain-paint-clip-013.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-clip-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-clip-013.html)
- [contain-paint-clip-014.html](https://wpt.fyi/results/css/css-contain/contain-paint-clip-014.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-clip-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-clip-014.html)
- [contain-paint-clip-015.html](https://wpt.fyi/results/css/css-contain/contain-paint-clip-015.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-clip-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-clip-015.html)
- [contain-paint-clip-016.html](https://wpt.fyi/results/css/css-contain/contain-paint-clip-016.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-clip-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-clip-016.html)
- [contain-paint-clip-017.html](https://wpt.fyi/results/css/css-contain/contain-paint-clip-017.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-clip-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-clip-017.html)
- [contain-paint-clip-018.html](https://wpt.fyi/results/css/css-contain/contain-paint-clip-018.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-clip-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-clip-018.html)
- [contain-paint-clip-019.html](https://wpt.fyi/results/css/css-contain/contain-paint-clip-019.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-clip-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-clip-019.html)
- [contain-paint-ifc-011.html](https://wpt.fyi/results/css/css-contain/contain-paint-ifc-011.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-ifc-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-ifc-011.html)
- [contain-paint-independent-formatting-context-001.html](https://wpt.fyi/results/css/css-contain/contain-paint-independent-formatting-context-001.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-independent-formatting-context-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-independent-formatting-context-001.html)
- [contain-paint-independent-formatting-context-002.html](https://wpt.fyi/results/css/css-contain/contain-paint-independent-formatting-context-002.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-independent-formatting-context-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-independent-formatting-context-002.html)
- [contain-paint-independent-formatting-context-003.html](https://wpt.fyi/results/css/css-contain/contain-paint-independent-formatting-context-003.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-independent-formatting-context-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-independent-formatting-context-003.html)
- [contain-paint-size-001.html](https://wpt.fyi/results/css/css-contain/contain-paint-size-001.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-size-001.html)
- [contain-paint-size-002.html](https://wpt.fyi/results/css/css-contain/contain-paint-size-002.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-size-002.html)
- [contain-paint-size-003.html](https://wpt.fyi/results/css/css-contain/contain-paint-size-003.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-size-003.html)
- [contain-paint-table-001.html](https://wpt.fyi/results/css/css-contain/contain-paint-table-001.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-table-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-table-001.html)
- [contain-paint-table-002.html](https://wpt.fyi/results/css/css-contain/contain-paint-table-002.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-table-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-table-002.html)
- [contain-subgrid-001.html](https://wpt.fyi/results/css/css-contain/contain-subgrid-001.html) [(live test)](http://wpt.live/css/css-contain/contain-subgrid-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-subgrid-001.html)
- [contain-paint-change-opacity.html](https://wpt.fyi/results/css/css-contain/contain-paint-change-opacity.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-change-opacity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-change-opacity.html)
- [contain-paint-dynamic-001.html](https://wpt.fyi/results/css/css-contain/contain-paint-dynamic-001.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-dynamic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-dynamic-001.html)
- [contain-paint-dynamic-002.html](https://wpt.fyi/results/css/css-contain/contain-paint-dynamic-002.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-dynamic-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-dynamic-002.html)
- [contain-paint-dynamic-003.html](https://wpt.fyi/results/css/css-contain/contain-paint-dynamic-003.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-dynamic-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-dynamic-003.html)
- [contain-paint-dynamic-004.html](https://wpt.fyi/results/css/css-contain/contain-paint-dynamic-004.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-dynamic-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-dynamic-004.html)
- [contain-paint-dynamic-005.html](https://wpt.fyi/results/css/css-contain/contain-paint-dynamic-005.html) [(live test)](http://wpt.live/css/css-contain/contain-paint-dynamic-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-paint-dynamic-005.html)

<a id="contain-applies"></a>This property generally applies to all elements (including [CSS Pseudo-Elements 4 § 4.1 Generated Content Pseudo-elements: ::before and ::after](https://drafts.csswg.org/css-pseudo-4/#generated-content)), although some types of containment have no effect on some elements, as detailed in [§ 3 Types of Containment](https://drafts.csswg.org/css-contain-2/#containment-types). In addition, in the case of [\[SVG2\]](https://drafts.csswg.org/css-contain-2/#biblio-svg2), the <a id="ref-for-propdef-contain⑦"></a>[contain](#propdef-contain) property only applies to <code><a id="ref-for-elementdef-svg"></a>[svg](https://w3c.github.io/svgwg/svg2-draft/struct.html#elementdef-svg)</code> elements that have an associated CSS layout box.

<a id="example-ace3db03"></a>

<strong>Example:</strong>

[](#example-ace3db03) <a id="ref-for-propdef-contain⑧"></a>[contain](#propdef-contain) is useful when used widely on a page, particularly when a page contains a lot of "widgets" which are all independent.

For example, assume a micropost social network had markup something like this:

``` text
<body>
  <aside>...</aside>
  <section>
    <h2>Messages</h2>
    <article>
      Lol, check out this dog: images.example.com/jsK3jkl
    </article>
    <article>
      I had a ham sandwich today. #goodtimes
    </article>
    <article>
      I have political opinions that you need to hear!
    </article>
    …
  </section>
</body>
```

There are probably a <em>lot</em> of messages displayed on the site, but each is independent and won’t affect anything else on the site. As such, each can be marked with <a id="ref-for-propdef-contain⑨"></a>[contain: content](#propdef-contain) to communicate this to the user agent, so it can optimize the page and skip a lot of computation for messages that are off-screen. If the size of each message is known ahead of time, <a id="ref-for-propdef-contain①⓪"></a>contain: strict can be applied to communicate further restrictions.

Additionally, when any <a id="ref-for-containment②"></a>[containments](https://drafts.csswg.org/css-contain-2/#containment) are active on either the HTML <code><a id="ref-for-the-html-element"></a>[html](https://html.spec.whatwg.org/multipage/semantics.html#the-html-element)</code> or <code><a id="ref-for-the-body-element"></a>[body](https://html.spec.whatwg.org/multipage/sections.html#the-body-element)</code> elements, propagation of properties from the <code><a id="ref-for-the-body-element①"></a>[body](https://html.spec.whatwg.org/multipage/sections.html#the-body-element)</code> element to the <a id="ref-for-initial-containing-block"></a>[initial containing block](https://drafts.csswg.org/css-display-4/#initial-containing-block), the viewport, or the <a id="ref-for-canvas-background"></a>[canvas background](https://drafts.csswg.org/css-backgrounds-3/#canvas-background), is disabled. Notably, this affects:

- <a id="ref-for-propdef-writing-mode"></a>[writing-mode](https://drafts.csswg.org/css-writing-modes-4/#propdef-writing-mode), <a id="ref-for-propdef-direction"></a>[direction](https://drafts.csswg.org/css-writing-modes-3/#propdef-direction), and <a id="ref-for-propdef-text-orientation"></a>[text-orientation](https://drafts.csswg.org/css-writing-modes-4/#propdef-text-orientation) (see [CSS Writing Modes 3 § 8 The Principal Writing Mode](https://drafts.csswg.org/css-writing-modes-3/#principal-flow))

- <a id="ref-for-propdef-overflow"></a>[overflow](https://drafts.csswg.org/css-overflow-3/#propdef-overflow) and its longhands (see [CSS Overflow 3 § 3.1.4 Overflow Viewport Propagation](https://drafts.csswg.org/css-overflow-3/#overflow-propagation))

- <a id="ref-for-propdef-background"></a>[background](https://drafts.csswg.org/css-backgrounds-3/#propdef-background) and its longhands (see [CSS Backgrounds 3 § 2.11.2 The Canvas Background and the HTML \<body\> Element](https://drafts.csswg.org/css-backgrounds-3/#body-background))

Tests

- [contain-body-bg-001.html](https://wpt.fyi/results/css/css-contain/contain-body-bg-001.html) [(live test)](http://wpt.live/css/css-contain/contain-body-bg-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-bg-001.html)
- [contain-body-bg-002.html](https://wpt.fyi/results/css/css-contain/contain-body-bg-002.html) [(live test)](http://wpt.live/css/css-contain/contain-body-bg-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-bg-002.html)
- [contain-body-bg-003.html](https://wpt.fyi/results/css/css-contain/contain-body-bg-003.html) [(live test)](http://wpt.live/css/css-contain/contain-body-bg-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-bg-003.html)
- [contain-body-bg-004.html](https://wpt.fyi/results/css/css-contain/contain-body-bg-004.html) [(live test)](http://wpt.live/css/css-contain/contain-body-bg-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-bg-004.html)
- [contain-body-dir-001.html](https://wpt.fyi/results/css/css-contain/contain-body-dir-001.html) [(live test)](http://wpt.live/css/css-contain/contain-body-dir-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-dir-001.html)
- [contain-body-dir-002.html](https://wpt.fyi/results/css/css-contain/contain-body-dir-002.html) [(live test)](http://wpt.live/css/css-contain/contain-body-dir-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-dir-002.html)
- [contain-body-dir-003.html](https://wpt.fyi/results/css/css-contain/contain-body-dir-003.html) [(live test)](http://wpt.live/css/css-contain/contain-body-dir-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-dir-003.html)
- [contain-body-dir-004.html](https://wpt.fyi/results/css/css-contain/contain-body-dir-004.html) [(live test)](http://wpt.live/css/css-contain/contain-body-dir-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-dir-004.html)
- [contain-body-overflow-001.html](https://wpt.fyi/results/css/css-contain/contain-body-overflow-001.html) [(live test)](http://wpt.live/css/css-contain/contain-body-overflow-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-overflow-001.html)
- [contain-body-overflow-002.html](https://wpt.fyi/results/css/css-contain/contain-body-overflow-002.html) [(live test)](http://wpt.live/css/css-contain/contain-body-overflow-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-overflow-002.html)
- [contain-body-overflow-003.html](https://wpt.fyi/results/css/css-contain/contain-body-overflow-003.html) [(live test)](http://wpt.live/css/css-contain/contain-body-overflow-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-overflow-003.html)
- [contain-body-overflow-004.html](https://wpt.fyi/results/css/css-contain/contain-body-overflow-004.html) [(live test)](http://wpt.live/css/css-contain/contain-body-overflow-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-overflow-004.html)
- [contain-body-t-o-001.html](https://wpt.fyi/results/css/css-contain/contain-body-t-o-001.html) [(live test)](http://wpt.live/css/css-contain/contain-body-t-o-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-t-o-001.html)
- [contain-body-t-o-002.html](https://wpt.fyi/results/css/css-contain/contain-body-t-o-002.html) [(live test)](http://wpt.live/css/css-contain/contain-body-t-o-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-t-o-002.html)
- [contain-body-t-o-003.html](https://wpt.fyi/results/css/css-contain/contain-body-t-o-003.html) [(live test)](http://wpt.live/css/css-contain/contain-body-t-o-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-t-o-003.html)
- [contain-body-t-o-004.html](https://wpt.fyi/results/css/css-contain/contain-body-t-o-004.html) [(live test)](http://wpt.live/css/css-contain/contain-body-t-o-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-t-o-004.html)
- [contain-body-w-m-001.html](https://wpt.fyi/results/css/css-contain/contain-body-w-m-001.html) [(live test)](http://wpt.live/css/css-contain/contain-body-w-m-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-w-m-001.html)
- [contain-body-w-m-002.html](https://wpt.fyi/results/css/css-contain/contain-body-w-m-002.html) [(live test)](http://wpt.live/css/css-contain/contain-body-w-m-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-w-m-002.html)
- [contain-body-w-m-003.html](https://wpt.fyi/results/css/css-contain/contain-body-w-m-003.html) [(live test)](http://wpt.live/css/css-contain/contain-body-w-m-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-w-m-003.html)
- [contain-body-w-m-004.html](https://wpt.fyi/results/css/css-contain/contain-body-w-m-004.html) [(live test)](http://wpt.live/css/css-contain/contain-body-w-m-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-body-w-m-004.html)
- [contain-html-bg-001.html](https://wpt.fyi/results/css/css-contain/contain-html-bg-001.html) [(live test)](http://wpt.live/css/css-contain/contain-html-bg-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-bg-001.html)
- [contain-html-bg-002.html](https://wpt.fyi/results/css/css-contain/contain-html-bg-002.html) [(live test)](http://wpt.live/css/css-contain/contain-html-bg-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-bg-002.html)
- [contain-html-bg-003.html](https://wpt.fyi/results/css/css-contain/contain-html-bg-003.html) [(live test)](http://wpt.live/css/css-contain/contain-html-bg-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-bg-003.html)
- [contain-html-bg-004.html](https://wpt.fyi/results/css/css-contain/contain-html-bg-004.html) [(live test)](http://wpt.live/css/css-contain/contain-html-bg-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-bg-004.html)
- [contain-html-dir-001.html](https://wpt.fyi/results/css/css-contain/contain-html-dir-001.html) [(live test)](http://wpt.live/css/css-contain/contain-html-dir-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-dir-001.html)
- [contain-html-dir-002.html](https://wpt.fyi/results/css/css-contain/contain-html-dir-002.html) [(live test)](http://wpt.live/css/css-contain/contain-html-dir-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-dir-002.html)
- [contain-html-dir-003.html](https://wpt.fyi/results/css/css-contain/contain-html-dir-003.html) [(live test)](http://wpt.live/css/css-contain/contain-html-dir-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-dir-003.html)
- [contain-html-dir-004.html](https://wpt.fyi/results/css/css-contain/contain-html-dir-004.html) [(live test)](http://wpt.live/css/css-contain/contain-html-dir-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-dir-004.html)
- [contain-html-overflow-001.html](https://wpt.fyi/results/css/css-contain/contain-html-overflow-001.html) [(live test)](http://wpt.live/css/css-contain/contain-html-overflow-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-overflow-001.html)
- [contain-html-overflow-002.html](https://wpt.fyi/results/css/css-contain/contain-html-overflow-002.html) [(live test)](http://wpt.live/css/css-contain/contain-html-overflow-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-overflow-002.html)
- [contain-html-overflow-003.html](https://wpt.fyi/results/css/css-contain/contain-html-overflow-003.html) [(live test)](http://wpt.live/css/css-contain/contain-html-overflow-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-overflow-003.html)
- [contain-html-overflow-004.html](https://wpt.fyi/results/css/css-contain/contain-html-overflow-004.html) [(live test)](http://wpt.live/css/css-contain/contain-html-overflow-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-overflow-004.html)
- [contain-html-t-o-001.html](https://wpt.fyi/results/css/css-contain/contain-html-t-o-001.html) [(live test)](http://wpt.live/css/css-contain/contain-html-t-o-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-t-o-001.html)
- [contain-html-t-o-002.html](https://wpt.fyi/results/css/css-contain/contain-html-t-o-002.html) [(live test)](http://wpt.live/css/css-contain/contain-html-t-o-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-t-o-002.html)
- [contain-html-t-o-003.html](https://wpt.fyi/results/css/css-contain/contain-html-t-o-003.html) [(live test)](http://wpt.live/css/css-contain/contain-html-t-o-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-t-o-003.html)
- [contain-html-t-o-004.html](https://wpt.fyi/results/css/css-contain/contain-html-t-o-004.html) [(live test)](http://wpt.live/css/css-contain/contain-html-t-o-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-t-o-004.html)
- [contain-html-w-m-001.html](https://wpt.fyi/results/css/css-contain/contain-html-w-m-001.html) [(live test)](http://wpt.live/css/css-contain/contain-html-w-m-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-w-m-001.html)
- [contain-html-w-m-002.html](https://wpt.fyi/results/css/css-contain/contain-html-w-m-002.html) [(live test)](http://wpt.live/css/css-contain/contain-html-w-m-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-w-m-002.html)
- [contain-html-w-m-003.html](https://wpt.fyi/results/css/css-contain/contain-html-w-m-003.html) [(live test)](http://wpt.live/css/css-contain/contain-html-w-m-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-w-m-003.html)
- [contain-html-w-m-004.html](https://wpt.fyi/results/css/css-contain/contain-html-w-m-004.html) [(live test)](http://wpt.live/css/css-contain/contain-html-w-m-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-html-w-m-004.html)

Note: Propagation to the <a id="ref-for-initial-containing-block①"></a>[initial containing block](https://drafts.csswg.org/css-display-4/#initial-containing-block), the viewport, or the <a id="ref-for-canvas-background①"></a>[canvas background](https://drafts.csswg.org/css-backgrounds-3/#canvas-background), of properties set on the <code><a id="ref-for-the-html-element①"></a>[html](https://html.spec.whatwg.org/multipage/semantics.html#the-html-element)</code> element itself is unaffected.

Note: Several properties beyond <a id="ref-for-propdef-contain①①"></a>[contain](#propdef-contain) can turn on various <a id="ref-for-containment③"></a>[containments](https://drafts.csswg.org/css-contain-2/#containment) for an element. These do not affect the value of <a id="ref-for-propdef-contain①②"></a>contain; an element can have <a id="ref-for-propdef-contain①③"></a>contain: none but still have <a id="ref-for-layout-containment①"></a>[layout containment](https://drafts.csswg.org/css-contain-2/#layout-containment) turned on by <a id="ref-for-propdef-content-visibility①"></a>[content-visibility](https://drafts.csswg.org/css-contain-2/#propdef-content-visibility), for example.

### <a id="containment-style"></a>3.4.  Style Containment[](#containment-style)

Giving an element <a id="style-containment"></a><strong>style containment</strong> has the following effects:

1.  The <a id="ref-for-propdef-counter-increment"></a>[counter-increment](https://drafts.csswg.org/css-lists-3/#propdef-counter-increment) and <a id="ref-for-propdef-counter-set"></a>[counter-set](https://drafts.csswg.org/css-lists-3/#propdef-counter-set) properties must be <a id="ref-for-property-scoped-to-a-sub-tree"></a>[scoped to the element’s sub-tree](#property-scoped-to-a-sub-tree) and create a new counter.

    Tests

    - [contain-style-counters-001.html](https://wpt.fyi/results/css/css-contain/contain-style-counters-001.html) [(live test)](http://wpt.live/css/css-contain/contain-style-counters-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-counters-001.html)
    - [contain-style-counters-002.html](https://wpt.fyi/results/css/css-contain/contain-style-counters-002.html) [(live test)](http://wpt.live/css/css-contain/contain-style-counters-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-counters-002.html)
    - [contain-style-counters-003.html](https://wpt.fyi/results/css/css-contain/contain-style-counters-003.html) [(live test)](http://wpt.live/css/css-contain/contain-style-counters-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-counters-003.html)
    - [contain-style-counters-004.html](https://wpt.fyi/results/css/css-contain/contain-style-counters-004.html) [(live test)](http://wpt.live/css/css-contain/contain-style-counters-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-counters-004.html)
    - [contain-style-ol-ordinal-li-container.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-li-container.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-li-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-li-container.html)
    - [contain-style-ol-ordinal-pseudo-reversed.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-pseudo-reversed.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-pseudo-reversed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-pseudo-reversed.html)
    - [contain-style-ol-ordinal-pseudo.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-pseudo.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-pseudo.html)
    - [contain-style-ol-ordinal-reversed.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-reversed.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-reversed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-reversed.html)
    - [contain-style-ol-ordinal-start-reversed.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-start-reversed.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-start-reversed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-start-reversed.html)
    - [contain-style-ol-ordinal-start.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-start.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-start.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-start.html)
    - [contain-style-ol-ordinal.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal.html)

2.  The effects of the <a id="ref-for-propdef-content"></a>[content](https://drafts.csswg.org/css-content-3/#propdef-content) property’s <a id="ref-for-valdef-content-open-quote"></a>[open-quote](https://drafts.csswg.org/css-content-3/#valdef-content-open-quote), <a id="ref-for-valdef-content-close-quote"></a>[close-quote](https://drafts.csswg.org/css-content-3/#valdef-content-close-quote), <a id="ref-for-valdef-content-no-open-quote"></a>[no-open-quote](https://drafts.csswg.org/css-content-3/#valdef-content-no-open-quote) and <a id="ref-for-valdef-content-no-close-quote"></a>[no-close-quote](https://drafts.csswg.org/css-content-3/#valdef-content-no-close-quote) must be <a id="ref-for-property-scoped-to-a-sub-tree①"></a>[scoped to the element’s sub-tree](#property-scoped-to-a-sub-tree).

    Note: This implies that the depth of quote nesting in the subtree is unchanged and starts at the value that its context normally implies, but that changes to the depth of quote nesting by these values inside the subtree do not affect the depth of quote nesting outside the subtree.

    Tests

    - [quote-scoping-001.html](https://wpt.fyi/results/css/css-contain/quote-scoping-001.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-001.html)
    - [quote-scoping-002.html](https://wpt.fyi/results/css/css-contain/quote-scoping-002.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-002.html)
    - [quote-scoping-003.html](https://wpt.fyi/results/css/css-contain/quote-scoping-003.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-003.html)
    - [quote-scoping-004.html](https://wpt.fyi/results/css/css-contain/quote-scoping-004.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-004.html)
    - [quote-scoping-empty-style-boundaries.html](https://wpt.fyi/results/css/css-contain/quote-scoping-empty-style-boundaries.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-empty-style-boundaries.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-empty-style-boundaries.html)
    - [quote-scoping-invalidation-001.html](https://wpt.fyi/results/css/css-contain/quote-scoping-invalidation-001.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-invalidation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-invalidation-001.html)
    - [quote-scoping-invalidation-002.html](https://wpt.fyi/results/css/css-contain/quote-scoping-invalidation-002.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-invalidation-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-invalidation-002.html)
    - [quote-scoping-invalidation-003.html](https://wpt.fyi/results/css/css-contain/quote-scoping-invalidation-003.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-invalidation-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-invalidation-003.html)
    - [quote-scoping-invalidation-004.html](https://wpt.fyi/results/css/css-contain/quote-scoping-invalidation-004.html) [(live test)](http://wpt.live/css/css-contain/quote-scoping-invalidation-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/quote-scoping-invalidation-004.html)

Tests

- [contain-style-baseline-001.html](https://wpt.fyi/results/css/css-contain/contain-style-baseline-001.html) [(live test)](http://wpt.live/css/css-contain/contain-style-baseline-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-baseline-001.html)
- [contain-style-breaks-001.html](https://wpt.fyi/results/css/css-contain/contain-style-breaks-001.html) [(live test)](http://wpt.live/css/css-contain/contain-style-breaks-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-breaks-001.html)
- [contain-style-breaks-002.html](https://wpt.fyi/results/css/css-contain/contain-style-breaks-002.html) [(live test)](http://wpt.live/css/css-contain/contain-style-breaks-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-breaks-002.html)
- [contain-style-breaks-003.html](https://wpt.fyi/results/css/css-contain/contain-style-breaks-003.html) [(live test)](http://wpt.live/css/css-contain/contain-style-breaks-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-breaks-003.html)
- [contain-style-breaks-004.html](https://wpt.fyi/results/css/css-contain/contain-style-breaks-004.html) [(live test)](http://wpt.live/css/css-contain/contain-style-breaks-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-breaks-004.html)
- [contain-style-breaks-005.html](https://wpt.fyi/results/css/css-contain/contain-style-breaks-005.html) [(live test)](http://wpt.live/css/css-contain/contain-style-breaks-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-breaks-005.html)

However, giving an element <a id="ref-for-style-containment①"></a>[style containment](#style-containment) has no effect if any of the following are true:

- if the element does not generate a <a id="ref-for-principal-box①④"></a>[principal box](https://drafts.csswg.org/css-display-4/#principal-box) (as is the case with <a id="ref-for-propdef-display⑦"></a>[display: contents](https://drafts.csswg.org/css-display-3/#propdef-display) or <a id="ref-for-propdef-display⑧"></a>display: none)

Note: [\[CSS-REGIONS-1\]](https://drafts.csswg.org/css-contain-2/#biblio-css-regions-1) has normative requirements on how <a id="ref-for-style-containment②"></a>[style containment](#style-containment) affects regions.

A <a id="property-scoped"></a><strong>scoped property</strong> has its effects scoped to a particular element or subtree.

- If <a id="property-scoped-to-an-element"></a><strong>scoped to an element</strong>, it must act as if the scoping element was the root of the document for the purpose of evaluating the property’s effects: any uses of the property outside the scoping element must have no effect on the uses of the property on or in the scoping element, and vice versa.

  Note: “Scoping to an element” is currently unused. It is defined as an extension point for future specifications to use.

- If <a id="property-scoped-to-a-sub-tree"></a><strong>scoped to a sub-tree</strong>, it’s the same, except the scoping element itself is counted as "outside" the tree, like the rest of the document, and the effects of the property on that element are unaffected by scoping. When considering the effects of the scoped property on elements <em>inside</em> the subtree, the element at the base of the subtree is treated as if it was the root of the document.

  Tests

  - [counter-scoping-001.html](https://wpt.fyi/results/css/css-contain/counter-scoping-001.html) [(live test)](http://wpt.live/css/css-contain/counter-scoping-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/counter-scoping-001.html)
  - [counter-scoping-002.html](https://wpt.fyi/results/css/css-contain/counter-scoping-002.html) [(live test)](http://wpt.live/css/css-contain/counter-scoping-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/counter-scoping-002.html)
  - [counter-scoping-003.html](https://wpt.fyi/results/css/css-contain/counter-scoping-003.html) [(live test)](http://wpt.live/css/css-contain/counter-scoping-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/counter-scoping-003.html)
  - [counter-scoping-004.html](https://wpt.fyi/results/css/css-contain/counter-scoping-004.html) [(live test)](http://wpt.live/css/css-contain/counter-scoping-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/counter-scoping-004.html)
  - [contain-style-ol-ordinal-li-container.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-li-container.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-li-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-li-container.html)
  - [contain-style-ol-ordinal-pseudo-reversed.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-pseudo-reversed.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-pseudo-reversed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-pseudo-reversed.html)
  - [contain-style-ol-ordinal-pseudo.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-pseudo.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-pseudo.html)
  - [contain-style-ol-ordinal-reversed.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-reversed.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-reversed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-reversed.html)
  - [contain-style-ol-ordinal-start-reversed.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-start-reversed.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-start-reversed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-start-reversed.html)
  - [contain-style-ol-ordinal-start.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal-start.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal-start.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal-start.html)
  - [contain-style-ol-ordinal.html](https://wpt.fyi/results/css/css-contain/contain-style-ol-ordinal.html) [(live test)](http://wpt.live/css/css-contain/contain-style-ol-ordinal.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/contain-style-ol-ordinal.html)

<a id="example-21ca5120"></a>

<strong>Example:</strong>

[](#example-21ca5120) As <a id="ref-for-propdef-counter-increment①"></a>[counter-increment](https://drafts.csswg.org/css-lists-3/#propdef-counter-increment) is scoped to an element’s subtree, the first use of it within the subtree creates a fresh counter, regardless of whether the counter had been used outside the scoping element. Any increments made within the subtree have no effect on counters of the same name outside the scoping element. However, the counter() and counters() value of the <a id="ref-for-propdef-content①"></a>[content](https://drafts.csswg.org/css-content-3/#propdef-content) property is not itself scoped, and can refer to counters established outside of the subtree.

For example, the following markup and style:

``` text
<div>
  <div style="contain: style;">
    <div></div>
    <div></div>
  </div>
  <div>
    <div></div>
    <div></div>
  </div>
</div>
```

``` text
body {
  counter-reset: foo;
}
div {
  counter-increment: foo;

  /* and to help observe nesting properly */
  margin: .2em;
  padding: .2em;
  background: #0001;
}
div::before {
  content: counters(foo, ".");
}
```

Will render like:

<a id="scoped-counter-example"></a>

1

2

2.1

2.2

3

4

5

Here, the style-contained element still interacts with the foo counter normally, incrementing it to 2. Its <a id="ref-for-selectordef-before①"></a>[::before](https://drafts.csswg.org/css-pseudo-4/#selectordef-before) <a id="ref-for-pseudo-element②"></a>[pseudo-element](https://drafts.csswg.org/selectors-4/#pseudo-element), which is effectively a child and thus within the containment boundary, can still see that counter normally, and thus will render the string "<code>2</code>".

Later siblings of the style-contained element don’t see anything that goes on within the boundary, so they increment the foo counter to 3 and beyond.

Within the containment, however, the descendant divs aren’t allowed to modify the value of the outside counter, so instead they’ll create a fresh (nested) foo counter when they attempt to increment it.

#### <a id="containment-style-opt"></a>3.4.1.  Possible Style-Containment Optimizations[](#containment-style-opt)

<em>This section is non-normative.</em>

Possible optimizations that can be enabled by <a id="ref-for-style-containment③"></a>[style containment](#style-containment) include (but are not limited to):

1.  Whenever a property is changed on a descendant of an element with <a id="ref-for-style-containment④"></a>[style containment](#style-containment), calculating what part of the DOM tree is "dirtied" and might need to have its style recalculated can stop at the element with <a id="ref-for-style-containment⑤"></a>style containment.
