Attribution and reformatting notice added for Surgeist on 2026-10-10

This bounded reformatted excerpt accompanies Surgeist as software implementation support. The original English document remains authoritative; this is not a new technical specification. Added provenance and representation labels are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Flexible Box Layout Module Level 1](https://drafts.csswg.org/css-flexbox-1/) under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The original copyright and legal notice are retained below.

# Source provenance and bounded scope

Retrieved: 2026-10-10. Source status: Editor’s Draft, 8 May 2026.

Original complete HTML SHA-256: `cc15b06af9bfc9b875db1072d5ae8c2e153017efbc07aa405d6aca5e3dc2a7bd` (1545971 bytes). Declared page revision: `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`. The retrieval date and hash identify this rendering; the live URL can change.

Retained scope: complete §9.9 (including all intrinsic sizing subsections) and §10 (including informative sample §10.1), plus source head/status/legal notice. All other sections, bibliography and indexes are excluded. Links to excluded fragments resolve upstream. Serialized bounded HTML SHA-256: `9e189415a389d3757a6828cb0fe8f3d9918dd597eab3eafb8914a583595d1845`.

Representation: HTML to GFM using installed Pandoc 3.1.11.1 and the repository's [conversion tools](tools/html-to-markdown/README.md). Scripts/styles are omitted without execution. Retained IDs, prose, headings, links, literal examples, image descriptions and table cells are checked against the bounded HTML. Source formulas are retained without correction. Spanning values, if any, are expanded only into applicable table columns; synthetic headings and semantic labels are non-normative. This capture contains 0 tables. Source images remain upstream links; no image assets are bundled by this capture.

The source labels the ideal intrinsic-main algorithm non-Web-compatible, the following contribution-sum algorithm Web-compatible, and §10.1 informative. These distinctions are preserved; the excerpt does not promote sample choices to normative rules or assert full engine compatibility.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Flexible Box Layout Module Level 1

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 8 May 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-flexbox/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-flexbox-1/>

<strong>Implementation Report:</strong>

<https://wpt.fyi/results/css/css-flexbox>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-flexbox-1)

<strong>Editors:</strong>

[Tab Atkins Jr.](http://xanthir.com/contact/) (Google)

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Rossen Atanassov](mailto:ratan@microsoft.com) (Microsoft)

<strong>Former Editors:</strong>

[Alex Mogilevsky](mailto:alexmog@microsoft.com) (Microsoft Corporation)

[L. David Baron](https://dbaron.org/) ([Google](https://www.google.com/))

[Neil Deakin](mailto:enndeakin@gmail.com) (Mozilla Corporation)

[Ian Hickson](mailto:ian@hixie.ch) (formerly of Opera Software)

[David Hyatt](mailto:hyatt@apple.com) (formerly of Netscape Corporation)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-flexbox-1/Overview.bs)

<strong>Issues List:</strong>

<https://drafts.csswg.org/css-flexbox-1/issues>

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-flexbox/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

### <a id="intrinsic-sizes"></a>9.9.  Intrinsic Sizes[](#intrinsic-sizes)

The <a id="ref-for-intrinsic-sizing"></a>[intrinsic sizing](https://drafts.csswg.org/css-sizing-3/#intrinsic-sizing) of a <a id="ref-for-flex-container⑤⑤"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) is used to produce various types of content-based automatic sizing, such as shrink-to-fit logical widths (which use the <a id="ref-for-valdef-width-fit-content②"></a>[fit-content](https://drafts.csswg.org/css-sizing-3/#valdef-width-fit-content) formula) and content-based logical heights (which use the <a id="ref-for-max-content②"></a>[max-content size](https://drafts.csswg.org/css-sizing-3/#max-content)). For these computations, auto margins on flex items are treated as 0.

See [\[CSS-SIZING-3\]](https://drafts.csswg.org/css-flexbox-1/#biblio-css-sizing-3) for a definition of the terms in this section.

Tests

- [flexbox-gap-position-absolute.html](https://wpt.fyi/results/css/css-flexbox/flexbox-gap-position-absolute.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-gap-position-absolute.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-gap-position-absolute.html)
- [gap-001-lr.html](https://wpt.fyi/results/css/css-flexbox/gap-001-lr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-001-lr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-001-lr.html)
- [gap-001-ltr.html](https://wpt.fyi/results/css/css-flexbox/gap-001-ltr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-001-ltr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-001-ltr.html)
- [gap-001-rl.html](https://wpt.fyi/results/css/css-flexbox/gap-001-rl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-001-rl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-001-rl.html)
- [gap-001-rtl.html](https://wpt.fyi/results/css/css-flexbox/gap-001-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-001-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-001-rtl.html)
- [gap-002-lr.html](https://wpt.fyi/results/css/css-flexbox/gap-002-lr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-002-lr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-002-lr.html)
- [gap-002-ltr.html](https://wpt.fyi/results/css/css-flexbox/gap-002-ltr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-002-ltr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-002-ltr.html)
- [gap-002-rl.html](https://wpt.fyi/results/css/css-flexbox/gap-002-rl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-002-rl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-002-rl.html)
- [gap-002-rtl.html](https://wpt.fyi/results/css/css-flexbox/gap-002-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-002-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-002-rtl.html)
- [gap-003-lr.html](https://wpt.fyi/results/css/css-flexbox/gap-003-lr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-003-lr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-003-lr.html)
- [gap-003-ltr.html](https://wpt.fyi/results/css/css-flexbox/gap-003-ltr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-003-ltr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-003-ltr.html)
- [gap-003-rl.html](https://wpt.fyi/results/css/css-flexbox/gap-003-rl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-003-rl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-003-rl.html)
- [gap-003-rtl.html](https://wpt.fyi/results/css/css-flexbox/gap-003-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-003-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-003-rtl.html)
- [gap-004-lr.html](https://wpt.fyi/results/css/css-flexbox/gap-004-lr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-004-lr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-004-lr.html)
- [gap-004-ltr.html](https://wpt.fyi/results/css/css-flexbox/gap-004-ltr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-004-ltr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-004-ltr.html)
- [gap-004-rl.html](https://wpt.fyi/results/css/css-flexbox/gap-004-rl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-004-rl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-004-rl.html)
- [gap-004-rtl.html](https://wpt.fyi/results/css/css-flexbox/gap-004-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-004-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-004-rtl.html)
- [gap-005-lr.html](https://wpt.fyi/results/css/css-flexbox/gap-005-lr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-005-lr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-005-lr.html)
- [gap-005-ltr.html](https://wpt.fyi/results/css/css-flexbox/gap-005-ltr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-005-ltr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-005-ltr.html)
- [gap-005-rl.html](https://wpt.fyi/results/css/css-flexbox/gap-005-rl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-005-rl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-005-rl.html)
- [gap-005-rtl.html](https://wpt.fyi/results/css/css-flexbox/gap-005-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-005-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-005-rtl.html)
- [gap-006-lr.html](https://wpt.fyi/results/css/css-flexbox/gap-006-lr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-006-lr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-006-lr.html)
- [gap-006-ltr.html](https://wpt.fyi/results/css/css-flexbox/gap-006-ltr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-006-ltr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-006-ltr.html)
- [gap-006-rl.html](https://wpt.fyi/results/css/css-flexbox/gap-006-rl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-006-rl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-006-rl.html)
- [gap-006-rtl.html](https://wpt.fyi/results/css/css-flexbox/gap-006-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-006-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-006-rtl.html)
- [gap-007-lr.html](https://wpt.fyi/results/css/css-flexbox/gap-007-lr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-007-lr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-007-lr.html)
- [gap-007-ltr.html](https://wpt.fyi/results/css/css-flexbox/gap-007-ltr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-007-ltr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-007-ltr.html)
- [gap-007-rl.html](https://wpt.fyi/results/css/css-flexbox/gap-007-rl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-007-rl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-007-rl.html)
- [gap-007-rtl.html](https://wpt.fyi/results/css/css-flexbox/gap-007-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/gap-007-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-007-rtl.html)
- [gap-008-ltr.html](https://wpt.fyi/results/css/css-flexbox/gap-008-ltr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-008-ltr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-008-ltr.html)
- [gap-009-ltr.html](https://wpt.fyi/results/css/css-flexbox/gap-009-ltr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-009-ltr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-009-ltr.html)
- [gap-010-ltr.html](https://wpt.fyi/results/css/css-flexbox/gap-010-ltr.html) [(live test)](http://wpt.live/css/css-flexbox/gap-010-ltr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-010-ltr.html)
- [gap-011.html](https://wpt.fyi/results/css/css-flexbox/gap-011.html) [(live test)](http://wpt.live/css/css-flexbox/gap-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-011.html)
- [gap-012.html](https://wpt.fyi/results/css/css-flexbox/gap-012.html) [(live test)](http://wpt.live/css/css-flexbox/gap-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-012.html)
- [gap-013.html](https://wpt.fyi/results/css/css-flexbox/gap-013.html) [(live test)](http://wpt.live/css/css-flexbox/gap-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-013.html)
- [gap-014.html](https://wpt.fyi/results/css/css-flexbox/gap-014.html) [(live test)](http://wpt.live/css/css-flexbox/gap-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-014.html)
- [gap-015.html](https://wpt.fyi/results/css/css-flexbox/gap-015.html) [(live test)](http://wpt.live/css/css-flexbox/gap-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-015.html)
- [gap-016.html](https://wpt.fyi/results/css/css-flexbox/gap-016.html) [(live test)](http://wpt.live/css/css-flexbox/gap-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-016.html)
- [gap-017.html](https://wpt.fyi/results/css/css-flexbox/gap-017.html) [(live test)](http://wpt.live/css/css-flexbox/gap-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-017.html)
- [gap-018.html](https://wpt.fyi/results/css/css-flexbox/gap-018.html) [(live test)](http://wpt.live/css/css-flexbox/gap-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-018.html)
- [gap-019.html](https://wpt.fyi/results/css/css-flexbox/gap-019.html) [(live test)](http://wpt.live/css/css-flexbox/gap-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-019.html)
- [gap-020.html](https://wpt.fyi/results/css/css-flexbox/gap-020.html) [(live test)](http://wpt.live/css/css-flexbox/gap-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-020.html)
- [gap-021.html](https://wpt.fyi/results/css/css-flexbox/gap-021.html) [(live test)](http://wpt.live/css/css-flexbox/gap-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/gap-021.html)
- [auto-min-size-001.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/auto-min-size-001.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/auto-min-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/auto-min-size-001.html)
- [col-wrap-001.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-001.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-001.html)
- [col-wrap-002.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-002.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-002.html)
- [col-wrap-003.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-003.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-003.html)
- [col-wrap-004.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-004.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-004.html)
- [col-wrap-005.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-005.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-005.html)
- [col-wrap-006.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-006.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-006.html)
- [col-wrap-007.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-007.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-007.html)
- [col-wrap-008.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-008.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-008.html)
- [col-wrap-009.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-009.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-009.html)
- [col-wrap-010.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-010.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-010.html)
- [col-wrap-011.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-011.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-011.html)
- [col-wrap-012.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-012.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-012.html)
- [col-wrap-013.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-013.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-013.html)
- [col-wrap-014.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-014.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-014.html)
- [col-wrap-015.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-015.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-015.html)
- [col-wrap-016.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-016.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-016.html)
- [col-wrap-017.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-017.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-017.html)
- [col-wrap-018.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-018.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-018.html)
- [col-wrap-019.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-019.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-019.html)
- [col-wrap-020.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-020.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-020.html)
- [col-wrap-crash.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/col-wrap-crash.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/col-wrap-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/col-wrap-crash.html)
- [row-001.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/row-001.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/row-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/row-001.html)
- [row-002.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/row-002.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/row-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/row-002.html)
- [row-003.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/row-003.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/row-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/row-003.html)
- [row-004.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/row-004.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/row-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/row-004.html)
- [row-005.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/row-005.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/row-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/row-005.html)
- [row-006.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/row-006.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/row-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/row-006.html)
- [row-007.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/row-007.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/row-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/row-007.html)
- [row-008.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/row-008.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/row-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/row-008.html)
- [row-compat-001.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/row-compat-001.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/row-compat-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/row-compat-001.html)
- [row-use-cases-001.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/row-use-cases-001.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/row-use-cases-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/row-use-cases-001.html)
- [row-wrap-001.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-size/row-wrap-001.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-size/row-wrap-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-size/row-wrap-001.html)
- [multiline-shrink-to-fit.html](https://wpt.fyi/results/css/css-flexbox/multiline-shrink-to-fit.html) [(live test)](http://wpt.live/css/css-flexbox/multiline-shrink-to-fit.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/multiline-shrink-to-fit.html)
- [svg-no-natural-size-grandchild.html](https://wpt.fyi/results/css/css-flexbox/svg-no-natural-size-grandchild.html) [(live test)](http://wpt.live/css/css-flexbox/svg-no-natural-size-grandchild.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/svg-no-natural-size-grandchild.html)
- [relayout-intrinsic-block-size.html](https://wpt.fyi/results/css/css-flexbox/relayout-intrinsic-block-size.html) [(live test)](http://wpt.live/css/css-flexbox/relayout-intrinsic-block-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/relayout-intrinsic-block-size.html)

#### <a id="intrinsic-main-sizes"></a>9.9.1.  Flex Container Intrinsic Main Sizes[](#intrinsic-main-sizes)

The <strong><a id="ref-for-max-content③"></a>[max-content](https://drafts.csswg.org/css-sizing-3/#max-content) <a id="ref-for-main-size③①"></a>[main size](https://drafts.csswg.org/css-flexbox-1/#main-size) of a <a id="ref-for-flex-container⑤⑥"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container)</strong> is, theoretically, the smallest size the <a id="ref-for-flex-container⑤⑦"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) can take such that when flex layout is run with that container size, each <a id="ref-for-flex-item①⓪④"></a>[flex item](https://drafts.csswg.org/css-flexbox-1/#flex-item) ends up at least as large as its [max-content contribution](#intrinsic-item-contributions), to the extent allowed by the items’ flexibility.

The <strong><a id="ref-for-min-content②"></a>[min-content](https://drafts.csswg.org/css-sizing-3/#min-content) <a id="ref-for-main-size③②"></a>[main size](https://drafts.csswg.org/css-flexbox-1/#main-size) of a <a id="ref-for-flex-container⑤⑧"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container)</strong> is, theoretically, the smallest size the <a id="ref-for-flex-container⑤⑨"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) can take such that no items overflow it, and no item’s contents overflow the item—​setting aside the cases in which the boxes layouts are <em>defined</em> to overflow (for example with negative margins or percentage sizes that add up to more than 100%).

Tests

- [flex-container-max-content-001.html](https://wpt.fyi/results/css/css-flexbox/flex-container-max-content-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-container-max-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-container-max-content-001.html)
- [flex-container-min-content-001.html](https://wpt.fyi/results/css/css-flexbox/flex-container-min-content-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-container-min-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-container-min-content-001.html)

For the min-content size of a multi-line flex container, see [§ 9.9.1.3 Multi-line Min-content Algorithm](#intrinsic-main-sizes-multiline). For max-content sizes and for single-line min-content sizes, an implementation is conformant to CSS Flexible Box Layout if it conforms to either the Ideal Algorithm or the Web-compatible Algorithm, as defined below.

##### <a id="intrinsic-main-sizes-ideal"></a>9.9.1.1.  Ideal Algorithm: Max-content Size and Min-content Single-line Size[](#intrinsic-main-sizes-ideal)

Note: The following algorithm calculates the <a id="ref-for-flex-container⑥⓪"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container)’s ideal intrinsic <a id="ref-for-main-size③③"></a>[main sizes](https://drafts.csswg.org/css-flexbox-1/#main-size). However, because it was not implemented correctly initially, and existing content became dependent on the (unfortunately consistent) incorrect implemented behavior, [it is not Web-compatible](https://github.com/w3c/csswg-drafts/issues/8884). Implementers and the CSS Working Group are investigating to what extent Web browser implementations can safely approach this behavior, and further experimentation is welcome.

Considering only non-<a id="ref-for-collapsed-flex-item③"></a>[collapsed](https://drafts.csswg.org/css-flexbox-1/#collapsed-flex-item) <a id="ref-for-flex-item①⓪⑤"></a>[flex items](https://drafts.csswg.org/css-flexbox-1/#flex-item);

1.  For each <a id="ref-for-flex-item①⓪⑥"></a>[flex item](https://drafts.csswg.org/css-flexbox-1/#flex-item), subtract its outer <a id="ref-for-flex-base-size①⑦"></a>[flex base size](https://drafts.csswg.org/css-flexbox-1/#flex-base-size) from its [max-content contribution](#intrinsic-item-contributions) size. If that result is positive, divide it by the item’s <a id="ref-for-flex-flex-grow-factor⑤"></a>[flex grow factor](https://drafts.csswg.org/css-flexbox-1/#flex-flex-grow-factor) if the <a id="ref-for-flex-flex-grow-factor⑥"></a>flex grow factor is ≥ 1, or multiply it by the <a id="ref-for-flex-flex-grow-factor⑦"></a>flex grow factor if the <a id="ref-for-flex-flex-grow-factor⑧"></a>flex grow factor is \< 1; if the result is negative, divide it by the item’s <a id="ref-for-scaled-flex-shrink-factor②"></a>[scaled flex shrink factor](https://drafts.csswg.org/css-flexbox-1/#scaled-flex-shrink-factor) (if dividing by zero, treat the result as negative infinity). This is the item’s <var>desired flex fraction</var>.

2.  Place all <a id="ref-for-flex-item①⓪⑦"></a>[flex items](https://drafts.csswg.org/css-flexbox-1/#flex-item) into lines of infinite length. Within each line, find the greatest (most positive) <var>desired flex fraction</var> among all the <a id="ref-for-flex-item①⓪⑧"></a>flex items. This is the line’s <var>chosen flex fraction</var>.

3.  If the <var>chosen flex fraction</var> is positive, and the sum of the line’s <a id="ref-for-flex-flex-grow-factor⑨"></a>[flex grow factors](https://drafts.csswg.org/css-flexbox-1/#flex-flex-grow-factor) is less than 1, divide the <var>chosen flex fraction</var> by that sum.

    If the <var>chosen flex fraction</var> is negative, and the sum of the line’s <a id="ref-for-flex-flex-shrink-factor⑥"></a>[flex shrink factors](https://drafts.csswg.org/css-flexbox-1/#flex-flex-shrink-factor) is less than 1, multiply the <var>chosen flex fraction</var> by that sum.

4.  Add each item’s <a id="ref-for-flex-base-size①⑧"></a>[flex base size](https://drafts.csswg.org/css-flexbox-1/#flex-base-size) to the product of its <a id="ref-for-flex-flex-grow-factor①⓪"></a>[flex grow factor](https://drafts.csswg.org/css-flexbox-1/#flex-flex-grow-factor) (<a id="ref-for-scaled-flex-shrink-factor③"></a>[scaled flex shrink factor](https://drafts.csswg.org/css-flexbox-1/#scaled-flex-shrink-factor), if shrinking) and the <var>chosen flex fraction</var>, then clamp that result by the <a id="ref-for-max-main-size"></a>[max main size](https://drafts.csswg.org/css-flexbox-1/#max-main-size) floored by the <a id="ref-for-min-main-size"></a>[min main size](https://drafts.csswg.org/css-flexbox-1/#min-main-size).

5.  The <a id="ref-for-flex-container⑥①"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container)’s <a id="ref-for-max-content④"></a>[max-content size](https://drafts.csswg.org/css-sizing-3/#max-content) is the largest sum (among all the lines) of the afore-calculated sizes of all items within a single line.

The <strong><a id="ref-for-min-content③"></a>[min-content](https://drafts.csswg.org/css-sizing-3/#min-content) <a id="ref-for-main-size③④"></a>[main size](https://drafts.csswg.org/css-flexbox-1/#main-size)</strong> of a <em><a id="ref-for-single-line-flex-container①⓪"></a>[single-line](https://drafts.csswg.org/css-flexbox-1/#single-line-flex-container)</em> flex container is calculated identically to the <a id="ref-for-max-content⑤"></a>[max-content](https://drafts.csswg.org/css-sizing-3/#max-content) <a id="ref-for-main-size③⑤"></a>[main size](https://drafts.csswg.org/css-flexbox-1/#main-size), except that the <a id="ref-for-flex-item①⓪⑨"></a>[flex items](https://drafts.csswg.org/css-flexbox-1/#flex-item)’ [min-content contributions](#intrinsic-item-contributions) are used instead of their [max-content contributions](#intrinsic-item-contributions).

<strong>Note:</strong>

Implications of this algorithm when the sum of flex is less than 1

The above algorithm is designed to give the correct behavior for two cases in particular, and make the <a id="ref-for-flex-container⑥②"></a>[flex container’s](https://drafts.csswg.org/css-flexbox-1/#flex-container) size continuous as you transition between the two:

1.  If all items are inflexible, the <a id="ref-for-flex-container⑥③"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) is sized to the sum of their <a id="ref-for-flex-base-size①⑨"></a>[flex base size](https://drafts.csswg.org/css-flexbox-1/#flex-base-size). (An inflexible <a id="ref-for-flex-base-size②⓪"></a>flex base size basically substitutes for a <a id="ref-for-propdef-width①⑤"></a>[width](https://drafts.csswg.org/css-sizing-3/#propdef-width)/<a id="ref-for-propdef-height①⓪"></a>[height](https://drafts.csswg.org/css-sizing-3/#propdef-height), which, when specified, is what a <a id="ref-for-max-content-contribution"></a>[max-content contribution](https://drafts.csswg.org/css-sizing-3/#max-content-contribution) is based on in Block Layout.)

2.  When all items are flexible with <a id="ref-for-flex-factor"></a>[flex factors](https://drafts.csswg.org/css-flexbox-1/#flex-factor) ≥ 1, the <a id="ref-for-flex-container⑥④"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) is sized to the sum of the <a id="ref-for-max-content-contribution①"></a>[max-content contributions](https://drafts.csswg.org/css-sizing-3/#max-content-contribution) of its items (or perhaps a slightly larger size, so that every flex item is <em>at least</em> the size of its <a id="ref-for-max-content-contribution②"></a>max-content contribution, but also has the correct ratio of its size to the size of the other items, as determined by its flexibility).

For example, if a <a id="ref-for-flex-container⑥⑤"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) has a single <a id="ref-for-flex-item①①⓪"></a>[flex item](https://drafts.csswg.org/css-flexbox-1/#flex-item) with <a id="ref-for-propdef-flex-basis①⑤"></a>[flex-basis: 100px;](https://drafts.csswg.org/css-flexbox-1/#propdef-flex-basis) but a max-content size of 200px, then when the item is <a id="ref-for-propdef-flex-grow①④"></a>[flex-grow: 0](https://drafts.csswg.org/css-flexbox-1/#propdef-flex-grow), the <a id="ref-for-flex-container⑥⑥"></a>flex container (and <a id="ref-for-flex-item①①①"></a>flex item) is 100px wide, but when the item is <a id="ref-for-propdef-flex-grow①⑤"></a>flex-grow: 1 or higher, the <a id="ref-for-flex-container⑥⑦"></a>flex container (and flex item) is 200px wide.

There are several possible ways to make the overall behavior continuous between these two cases, but all of them have drawbacks. We chose one we feel has the least bad implications; unfortunately, it "double-applies" the flexibility in cases with <a id="ref-for-flex-factor①"></a>[flex factors](https://drafts.csswg.org/css-flexbox-1/#flex-factor) that are \< 1. In the above example, if the item has <a id="ref-for-propdef-flex-grow①⑥"></a>[flex-grow: .5](https://drafts.csswg.org/css-flexbox-1/#propdef-flex-grow), then the <a id="ref-for-flex-container⑥⑧"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) ends up 150px wide, but the item then sizes normally into that available space, ending up 125px wide.

<strong>Note:</strong>

Even more involved notes on the specific behavior chosen

Principles:

1.  Don’t explode any sizes, whether growing or shrinking, as inputs approach zero.

2.  When flex factors are all \>=1, return the minimum size necessary for every item to be \>= max-content size.

3.  Items with a zero flex shouldn’t affect the sizes at all.

4.  Keep it continuous over variance of flex factors and item sizes.

5.  Keep sizing variance as linear as possible with respect to linear changes to any input variable (size, flex factor).

6.  When the sum of flex factors is \>=1, return the minimum size necessary for every item to be \>= max-content size.

To get these all to work together, we have to apply some correction when either flex factors or the sum of flex factors on a line is \< 1.

For shrink our behavior is somewhat easier; since the explosive case of 0 shrink results in a negative infinity desired fraction which we’ll never choose (since we always take the largest), we can just apply the correction at the line level, giving us double-application only when the sum is \< 1.

For positives it’s more complicated. 0 grow naively explodes into \*positive\* infinity, which we’d choose, so we need to apply the correction at the individual item level. We do that by multiplying the space by the factor when factor is \<1. Leaving it at that would result in a double-application for items \< 1 but sum \>= 1, but a \*triple\*-application when the sum is \< 1. To avoid \*that\* ridiculousness, we apply a \*reverse\* correction when the sum is 1, dividing by the sum instead. This leaves us with a double correction in all cases for items with factors \< 1.

We can’t eliminate the double-applications entirely without giving up other, more important principles (in particular, principle 3 —​try to come up with rules that don’t double-apply when you have two items with <a id="ref-for-propdef-flex-grow①⑦"></a>[flex-grow: .5](https://drafts.csswg.org/css-flexbox-1/#propdef-flex-grow), but also don’t give a <a id="ref-for-propdef-flex-grow①⑧"></a>flex-grow: 0 item any power over a <a id="ref-for-propdef-flex-grow①⑨"></a>flex-grow: 1 sibling; you can’t, as far as we can tell.)

##### <a id="intrinsic-main-sizes-compat"></a>9.9.1.2.  Web-compatible Intrinsic Sizing Algorithm: Max-content Size and Min-content Single-line Size[](#intrinsic-main-sizes-compat)

Note: The following algorithm has been demonstrated to be Web-compatible. It may be altered in the future to bring it closer to the ideal algorithm above, if possible.

- For the <a id="ref-for-max-content⑥"></a>[max-content size](https://drafts.csswg.org/css-sizing-3/#max-content) of a <a id="ref-for-flex-container⑥⑨"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container), take the sum of the [max-content contributions](#intrinsic-item-contributions) of all the non-<a id="ref-for-collapsed-flex-item④"></a>[collapsed](https://drafts.csswg.org/css-flexbox-1/#collapsed-flex-item) <a id="ref-for-flex-item①①②"></a>[flex items](https://drafts.csswg.org/css-flexbox-1/#flex-item) in the <a id="ref-for-flex-container⑦⓪"></a>flex container.

- For the <a id="ref-for-min-content④"></a>[min-content size](https://drafts.csswg.org/css-sizing-3/#min-content) of a <em><a id="ref-for-single-line-flex-container①①"></a>[single-line](https://drafts.csswg.org/css-flexbox-1/#single-line-flex-container)</em> container, take the sum of the [min-content contributions](#intrinsic-item-contributions) of all the non-<a id="ref-for-collapsed-flex-item⑤"></a>[collapsed](https://drafts.csswg.org/css-flexbox-1/#collapsed-flex-item) <a id="ref-for-flex-item①①③"></a>[flex items](https://drafts.csswg.org/css-flexbox-1/#flex-item) in the <a id="ref-for-flex-container⑦①"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container).

##### <a id="intrinsic-main-sizes-multiline"></a>9.9.1.3.  Multi-line Min-content Algorithm[](#intrinsic-main-sizes-multiline)

For a <em><a id="ref-for-multi-line-flex-container⑧"></a>[multi-line](https://drafts.csswg.org/css-flexbox-1/#multi-line-flex-container)</em> container, the <a id="ref-for-min-content⑤"></a>[min-content](https://drafts.csswg.org/css-sizing-3/#min-content) <a id="ref-for-main-size③⑥"></a>[main size](https://drafts.csswg.org/css-flexbox-1/#main-size) is simply the largest [min-content contribution](#intrinsic-item-contributions) of all the non-<a id="ref-for-collapsed-flex-item⑥"></a>[collapsed](https://drafts.csswg.org/css-flexbox-1/#collapsed-flex-item) <a id="ref-for-flex-item①①④"></a>[flex items](https://drafts.csswg.org/css-flexbox-1/#flex-item) in the <a id="ref-for-flex-container⑦②"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container).

#### <a id="intrinsic-cross-sizes"></a>9.9.2.  Flex Container Intrinsic Cross Sizes[](#intrinsic-cross-sizes)

The <strong><a id="ref-for-min-content⑥"></a>[min-content](https://drafts.csswg.org/css-sizing-3/#min-content)/<a id="ref-for-max-content⑦"></a>[max-content](https://drafts.csswg.org/css-sizing-3/#max-content) <a id="ref-for-cross-size③⓪"></a>[cross size](https://drafts.csswg.org/css-flexbox-1/#cross-size)</strong> of a <em><a id="ref-for-single-line-flex-container①②"></a>[single-line](https://drafts.csswg.org/css-flexbox-1/#single-line-flex-container)</em> <a id="ref-for-flex-container⑦③"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) is the largest <a id="ref-for-min-content-contribution"></a>[min-content contribution](https://drafts.csswg.org/css-sizing-3/#min-content-contribution)/<a id="ref-for-max-content-contribution③"></a>[max-content contribution](https://drafts.csswg.org/css-sizing-3/#max-content-contribution) (respectively) of its <a id="ref-for-flex-item①①⑤"></a>[flex items](https://drafts.csswg.org/css-flexbox-1/#flex-item).

For a <em><a id="ref-for-multi-line-flex-container⑨"></a>[multi-line](https://drafts.csswg.org/css-flexbox-1/#multi-line-flex-container)</em> <a id="ref-for-flex-container⑦④"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container), the behavior depends on whether it’s a row or column flexbox:

<strong><a id="ref-for-valdef-flex-direction-row③"></a>[row](https://drafts.csswg.org/css-flexbox-1/#valdef-flex-direction-row) <a id="ref-for-multi-line-flex-container①⓪"></a>[multi-line](https://drafts.csswg.org/css-flexbox-1/#multi-line-flex-container) <a id="ref-for-flex-container⑦⑤"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) <a id="ref-for-cross-size③①"></a>[cross size](https://drafts.csswg.org/css-flexbox-1/#cross-size)</strong>

The <a id="ref-for-min-content⑦"></a>[min-content](https://drafts.csswg.org/css-sizing-3/#min-content)/<a id="ref-for-max-content⑧"></a>[max-content](https://drafts.csswg.org/css-sizing-3/#max-content) <a id="ref-for-cross-size③②"></a>[cross size](https://drafts.csswg.org/css-flexbox-1/#cross-size) is the sum of the flex line cross sizes resulting from sizing the flex container under a <a id="ref-for-cross-axis①⑦"></a>[cross-axis](https://drafts.csswg.org/css-flexbox-1/#cross-axis) <a id="ref-for-min-content-constraint①"></a>[min-content constraint](https://drafts.csswg.org/css-sizing-3/#min-content-constraint)/<a id="ref-for-max-content-constraint①"></a>[max-content constraint](https://drafts.csswg.org/css-sizing-3/#max-content-constraint) (respectively).

<strong><a id="ref-for-valdef-flex-direction-column①"></a>[column](https://drafts.csswg.org/css-flexbox-1/#valdef-flex-direction-column) <a id="ref-for-multi-line-flex-container①①"></a>[multi-line](https://drafts.csswg.org/css-flexbox-1/#multi-line-flex-container) <a id="ref-for-flex-container⑦⑥"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) <a id="ref-for-cross-size③③"></a>[cross size](https://drafts.csswg.org/css-flexbox-1/#cross-size)</strong>

The <a id="ref-for-min-content⑧"></a>[min-content](https://drafts.csswg.org/css-sizing-3/#min-content) <a id="ref-for-cross-size③④"></a>[cross size](https://drafts.csswg.org/css-flexbox-1/#cross-size) is the largest <a id="ref-for-min-content-contribution①"></a>[min-content contribution](https://drafts.csswg.org/css-sizing-3/#min-content-contribution) among all of its <a id="ref-for-flex-item①①⑥"></a>[flex items](https://drafts.csswg.org/css-flexbox-1/#flex-item).

Note: This heuristic effectively assumes a single flex line, in order to guarantee that the <a id="ref-for-min-content⑨"></a>[min-content size](https://drafts.csswg.org/css-sizing-3/#min-content) is smaller than the <a id="ref-for-max-content⑨"></a>[max-content size](https://drafts.csswg.org/css-sizing-3/#max-content). If the flex container has a height constraint, this will result in overflow, but if the <a id="ref-for-flex-container⑦⑦"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) is also a <a id="ref-for-block-axis-scroll-container①"></a>[main-axis scroll container](https://drafts.csswg.org/css-overflow-3/#block-axis-scroll-container), it will at least be large enough to fit any given column entirely within its <a id="ref-for-scrollport"></a>[scrollport](https://drafts.csswg.org/css-overflow-3/#scrollport).

The <a id="ref-for-max-content①⓪"></a>[max-content](https://drafts.csswg.org/css-sizing-3/#max-content) <a id="ref-for-cross-size③⑤"></a>[cross size](https://drafts.csswg.org/css-flexbox-1/#cross-size) is the sum of the <a id="ref-for-flex-line①⓪"></a>[flex line](https://drafts.csswg.org/css-flexbox-1/#flex-line) cross sizes resulting from sizing the <a id="ref-for-flex-container⑦⑧"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) under a <a id="ref-for-cross-axis①⑧"></a>[cross-axis](https://drafts.csswg.org/css-flexbox-1/#cross-axis) <a id="ref-for-max-content-constraint②"></a>[max-content constraint](https://drafts.csswg.org/css-sizing-3/#max-content-constraint), using the largest <a id="ref-for-max-content-contribution④"></a>[max-content](https://drafts.csswg.org/css-sizing-3/#max-content-contribution) cross-size contribution among the <a id="ref-for-flex-item①①⑦"></a>[flex items](https://drafts.csswg.org/css-flexbox-1/#flex-item) as the <a id="ref-for-available④"></a>[available space](https://drafts.csswg.org/css-sizing-3/#available) in the <a id="ref-for-cross-axis①⑨"></a>cross axis for each of the <a id="ref-for-flex-item①①⑧"></a>flex items during layout.

Note: This heuristic gives a reasonable approximation of the size that the <a id="ref-for-flex-container⑦⑨"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) should be, with each <a id="ref-for-flex-item①①⑨"></a>[flex item](https://drafts.csswg.org/css-flexbox-1/#flex-item) laid out at its <a id="ref-for-max-content-contribution⑤"></a>[max-content contribution](https://drafts.csswg.org/css-sizing-3/#max-content-contribution) or larger, and each <a id="ref-for-flex-line①①"></a>[flex line](https://drafts.csswg.org/css-flexbox-1/#flex-line) no larger than its largest <a id="ref-for-flex-item①②⓪"></a>flex item. It’s not a <em>perfect</em> fit in some cases, but doing it completely correct is insanely expensive, and this works reasonably well.

Tests

- [flexbox_width-change-and-relayout-children.html](https://wpt.fyi/results/css/css-flexbox/flexbox_width-change-and-relayout-children.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_width-change-and-relayout-children.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_width-change-and-relayout-children.html)
- [table-as-flex-item-max-content.html](https://wpt.fyi/results/css/css-flexbox/table-as-flex-item-max-content.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-flex-item-max-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-flex-item-max-content.html)
- [table-as-item-flex-cross-size.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-flex-cross-size.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-flex-cross-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-flex-cross-size.html)

#### <a id="intrinsic-item-contributions"></a>9.9.3.  Flex Item Intrinsic Size Contributions[](#intrinsic-item-contributions)

The <strong>main-size <a id="ref-for-min-content-contribution②"></a>[min-content contribution](https://drafts.csswg.org/css-sizing-3/#min-content-contribution) of a <a id="ref-for-flex-item①②①"></a>[flex item](https://drafts.csswg.org/css-flexbox-1/#flex-item)</strong> is the larger of its <em>outer</em> <a id="ref-for-min-content①⓪"></a>[min-content size](https://drafts.csswg.org/css-sizing-3/#min-content) and outer <a id="ref-for-preferred-size③"></a>[preferred size](https://drafts.csswg.org/css-sizing-3/#preferred-size) if that is not an <a id="ref-for-automatic-size③"></a>[automatic size](https://drafts.csswg.org/css-sizing-3/#automatic-size).

The <strong>main-size <a id="ref-for-max-content-contribution⑥"></a>[max-content contribution](https://drafts.csswg.org/css-sizing-3/#max-content-contribution) of a <a id="ref-for-flex-item①②②"></a>[flex item](https://drafts.csswg.org/css-flexbox-1/#flex-item)</strong> is the larger of its <em>outer</em> <a id="ref-for-max-content①①"></a>[max-content size](https://drafts.csswg.org/css-sizing-3/#max-content) and outer <a id="ref-for-preferred-size④"></a>[preferred size](https://drafts.csswg.org/css-sizing-3/#preferred-size) if that is not an <a id="ref-for-automatic-size④"></a>[automatic size](https://drafts.csswg.org/css-sizing-3/#automatic-size).

For this purpose, each contribution is capped by the item’s <a id="ref-for-flex-base-size②①"></a>[flex base size](https://drafts.csswg.org/css-flexbox-1/#flex-base-size) if the item is not growable, floored by the item’s <a id="ref-for-flex-base-size②②"></a>flex base size if the item is not shrinkable, and then further clamped by the item’s <a id="ref-for-min-main-size①"></a>[min](https://drafts.csswg.org/css-flexbox-1/#min-main-size)/<a id="ref-for-max-main-size①"></a>[max main size](https://drafts.csswg.org/css-flexbox-1/#max-main-size).

## <a id="pagination"></a>10.  Fragmenting Flex Layout[](#pagination)

Flex containers can break across pages between items, between lines of items (in <a id="ref-for-multi-line-flex-container①②"></a>[multi-line](https://drafts.csswg.org/css-flexbox-1/#multi-line-flex-container) mode), and inside items. The <a id="ref-for-propdef-break-before"></a>[break-\*](https://drafts.csswg.org/css-break-3/#propdef-break-before) properties apply to flex containers as normal for block-level or inline-level boxes. This section defines how they apply to flex items and the contents of flex items. See the [CSS Fragmentation Module](http://www.w3.org/TR/css-break/) for more context [\[CSS3-BREAK\]](https://drafts.csswg.org/css-flexbox-1/#biblio-css3-break).

The following breaking rules refer to the <a id="ref-for-fragmentation-container"></a>[fragmentation container](https://drafts.csswg.org/css-break-4/#fragmentation-container) as the “page”. The same rules apply in any other <a id="ref-for-fragmentation-context"></a>[fragmentation context](https://drafts.csswg.org/css-break-4/#fragmentation-context). (Substitute “page” with the appropriate <a id="ref-for-fragmentation-container①"></a>fragmentation container type as needed.) For readability, in this section the terms "row" and "column" refer to the relative orientation of the <a id="ref-for-flex-container⑧⓪"></a>[flex container](https://drafts.csswg.org/css-flexbox-1/#flex-container) with respect to the block flow direction of the <a id="ref-for-fragmentation-context①"></a>fragmentation context, rather than to that of the <a id="ref-for-flex-container⑧①"></a>flex container itself.

The exact layout of a fragmented flex container is not defined in this level of Flexible Box Layout. However, breaks inside a flex container are subject to the following rules (interpreted using <a id="ref-for-order-modified-document-order③"></a>[order-modified document order](https://drafts.csswg.org/css-display-4/#order-modified-document-order)):

- In a row flex container, the <a id="ref-for-propdef-break-before①"></a>[break-before](https://drafts.csswg.org/css-break-3/#propdef-break-before) and <a id="ref-for-propdef-break-after"></a>[break-after](https://drafts.csswg.org/css-break-3/#propdef-break-after) values on flex items are propagated to the flex line. The <a id="ref-for-propdef-break-before②"></a>break-before values on the first line and the <a id="ref-for-propdef-break-after①"></a>break-after values on the last line are propagated to the flex container.

  Note: Break propagation (like <a id="ref-for-propdef-text-decoration"></a>[text-decoration](https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration) propagation) does not affect <a id="ref-for-computed-value②"></a>[computed values](https://drafts.csswg.org/css-cascade-5/#computed-value).

- In a column flex container, the <a id="ref-for-propdef-break-before③"></a>[break-before](https://drafts.csswg.org/css-break-3/#propdef-break-before) values on the first item and the <a id="ref-for-propdef-break-after②"></a>[break-after](https://drafts.csswg.org/css-break-3/#propdef-break-after) values on the last item are propagated to the flex container. Forced breaks on other items are applied to the item itself.

- A forced break inside a flex item effectively increases the size of its contents; it does not trigger a forced break inside sibling items.

- In a row flex container, [Class A break opportunities](https://www.w3.org/TR/css3-break/#btw-blocks) occur between sibling flex lines, and [Class C break opportunities](https://www.w3.org/TR/css3-break/#end-block) occur between the first/last flex line and the flex container’s content edges. In a column flex container, [Class A break opportunities](https://www.w3.org/TR/css3-break/#btw-blocks) occur between sibling flex items, and [Class C break opportunities](https://www.w3.org/TR/css3-break/#end-block) occur between the first/last flex items on a line and the flex container’s content edges. [\[CSS3-BREAK\]](https://drafts.csswg.org/css-flexbox-1/#biblio-css3-break)

- When a flex container is continued after a break, the space available to its <a id="ref-for-flex-item①②③"></a>[flex items](https://drafts.csswg.org/css-flexbox-1/#flex-item) (in the block flow direction of the fragmentation context) is reduced by the space consumed by flex container fragments on previous pages. The space consumed by a flex container fragment is the size of its content box on that page. If as a result of this adjustment the available space becomes negative, it is set to zero.

- If the first fragment of the flex container is not at the top of the page, and none of its flex items fit in the remaining space on the page, the entire fragment is moved to the next page.

- When breaking a <a id="ref-for-multi-line-flex-container①③"></a>[multi-line](https://drafts.csswg.org/css-flexbox-1/#multi-line-flex-container) column flex container, the UA may organize each fragment into its own “stack” of flex lines—​just like each fragment of a multi-column container has its own row of column boxes—​in order to ensure that content presented on earlier pages corresponds to content earlier in the box order.

- Aside from the rearrangement of items imposed by the previous point, UAs should attempt to minimize distortion of the flex container with respect to unfragmented flow.

Tests

- flexbox_interactive_break-after-column-item.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_break-after-column-item.html)
- flexbox_interactive_break-after-column-lastitem.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_break-after-column-lastitem.html)
- flexbox_interactive_break-after-container.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_break-after-container.html)
- flexbox_interactive_break-after-item.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_break-after-item.html)
- flexbox_interactive_break-after-line.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_break-after-line.html)
- flexbox_interactive_break-after-line-order.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_break-after-line-order.html)
- flexbox_interactive_break-after-multiline.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_break-after-multiline.html)
- flexbox_interactive_break-before-column-firstitem.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_break-before-column-firstitem.html)
- flexbox_interactive_break-before-column-item.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_break-before-column-item.html)
- flexbox_interactive_break-before-container.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_break-before-container.html)
- flexbox_interactive_break-before-item.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_break-before-item.html)
- flexbox_interactive_break-before-multiline.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_break-before-multiline.html)
- flexbox_interactive_break-natural.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_break-natural.html)

### <a id="pagination-algo"></a>10.1.  Sample Flex Fragmentation Algorithm[](#pagination-algo)

This informative section presents a possible fragmentation algorithm for flex containers. Implementors are encouraged to improve on this algorithm and [provide feedback to the CSS Working Group](https://drafts.csswg.org/css-flexbox-1/#sotd).

<a id="example-93cdec9b"></a>

<strong>Example:</strong>

[](#example-93cdec9b)

This algorithm assumes that pagination always proceeds only in the forward direction; therefore, in the algorithms below, alignment is mostly ignored prior to pagination. Advanced layout engines may be able to honor alignment across fragments.

<strong>Note:</strong>

<strong><a id="ref-for-single-line-flex-container①③"></a>[single-line](https://drafts.csswg.org/css-flexbox-1/#single-line-flex-container) column flex container</strong>

1.  Run the flex layout algorithm (without regards to pagination) through [Cross Sizing Determination](https://drafts.csswg.org/css-flexbox-1/#cross-sizing).
2.  Lay out as many consecutive flex items or item fragments as possible (but at least one or a fragment thereof), starting from the first, until there is no more room on the page or a forced break is encountered.
3.  If the previous step ran out of room and the free space is positive, the UA may reduce the distributed free space on this page (down to, but not past, zero) in order to make room for the next unbreakable flex item or fragment. Otherwise, the item or fragment that does not fit is pushed to the next page. The UA should pull up if more than 50% of the fragment would have fit in the remaining space and should push otherwise.
4.  If there are any flex items or fragments not laid out by the previous steps, rerun the flex layout algorithm from [Line Length Determination](https://drafts.csswg.org/css-flexbox-1/#line-sizing) through [Cross Sizing Determination](https://drafts.csswg.org/css-flexbox-1/#cross-sizing) with the next page’s size and <em>all</em> the contents (including those already laid out), and return to the previous step, but starting from the first item or fragment not already laid out.
5.  For each fragment of the flex container, continue the flex layout algorithm from [Main-Axis Alignment](https://drafts.csswg.org/css-flexbox-1/#main-alignment) to its finish.

It is the intent of this algorithm that column-direction

<strong>Note:</strong>

<a id="ref-for-single-line-flex-container①④"></a>[single-line](https://drafts.csswg.org/css-flexbox-1/#single-line-flex-container) flex containers paginate very similarly to block flow. As a test of the intent, a flex container with justify-content:start and no flexible items should paginate identically to a block with <a id="ref-for-in-flow④"></a>[in-flow](https://drafts.csswg.org/css-display-4/#in-flow) children with same content, same used size and same used margins.

<strong><a id="ref-for-multi-line-flex-container①④"></a>[multi-line](https://drafts.csswg.org/css-flexbox-1/#multi-line-flex-container) column flex container</strong>

1.  Run the flex layout algorithm <em>with</em> regards to pagination (limiting the flex container’s maximum line length to the space left on the page) through [Cross Sizing Determination](https://drafts.csswg.org/css-flexbox-1/#cross-sizing).
2.  Lay out as many flex lines as possible (but at least one) until there is no more room in the flex container in the cross dimension or a forced break is encountered:
    1.  Lay out as many consecutive flex items as possible (but at least one), starting from the first, until there is no more room on the page or a forced break is encountered. Forced breaks <em>within</em> flex items are ignored.
    2.  If this is the first flex container fragment, this line contains only a single flex item that is larger than the space left on the page, and the flex container is not at the top of the page already, move the flex container to the next page and restart flex container layout entirely.
    3.  If there are any flex items not laid out by the first step, rerun the flex layout algorithm from [Main Sizing Determination](https://drafts.csswg.org/css-flexbox-1/#main-sizing) through [Cross Sizing Determination](https://drafts.csswg.org/css-flexbox-1/#cross-sizing) using only the items not laid out on a previous line, and return to the previous step, starting from the first item not already laid out.
3.  If there are any flex items not laid out by the previous step, rerun the flex layout algorithm from [Line Sizing Determination](https://drafts.csswg.org/css-flexbox-1/#line-sizing) through [Cross Sizing Determination](https://drafts.csswg.org/css-flexbox-1/#cross-sizing) with the next page’s size and only the items not already laid out, and return to the previous step, but starting from the first item not already laid out.
4.  For each fragment of the flex container, continue the flex layout algorithm from [Main-Axis Alignment](https://drafts.csswg.org/css-flexbox-1/#main-alignment) to its finish.

A shortcoming of this sample algorithm is that if a flex item does not entirely fit on a single page, it will

<strong>Note:</strong>

<em>not</em> be paginated in <a id="ref-for-multi-line-flex-container①⑤"></a>[multi-line](https://drafts.csswg.org/css-flexbox-1/#multi-line-flex-container) column flex containers.

<strong><a id="ref-for-single-line-flex-container①⑤"></a>[single-line](https://drafts.csswg.org/css-flexbox-1/#single-line-flex-container) row flex container</strong>

1.  Run the entire flex layout algorithm (without regards to pagination), except treat any <a id="ref-for-propdef-align-self①⑦"></a>[align-self](https://drafts.csswg.org/css-flexbox-1/#propdef-align-self) other than <a id="ref-for-valdef-align-items-flex-start"></a>[flex-start](https://drafts.csswg.org/css-flexbox-1/#valdef-align-items-flex-start) or <a id="ref-for-valdef-align-items-baseline①"></a>[baseline](https://drafts.csswg.org/css-flexbox-1/#valdef-align-items-baseline) as <a id="ref-for-valdef-align-items-flex-start①"></a>flex-start.

2.  If an unbreakable item doesn’t fit within the space left on the page, and the flex container is not at the top of the page, move the flex container to the next page and restart flex container layout entirely.

3.  For each item, lay out as much of its contents as will fit in the space left on the page, and fragment the remaining content onto the next page, rerunning the flex layout algorithm from [Line Length Determination](https://drafts.csswg.org/css-flexbox-1/#line-sizing) through [Main-Axis Alignment](https://drafts.csswg.org/css-flexbox-1/#main-alignment) into the new page size using <em>all</em> the contents (including items completed on previous pages).

    Any flex items that fit entirely into previous fragments still take up space in the main axis in later fragments.

    <strong>Note:</strong>

4.  For each fragment of the flex container, rerun the flex layout algorithm from [Cross-Axis Alignment](https://drafts.csswg.org/css-flexbox-1/#cross-alignment) to its finish. For all fragments besides the first, treat <a id="ref-for-propdef-align-self①⑧"></a>[align-self](https://drafts.csswg.org/css-flexbox-1/#propdef-align-self) and <a id="ref-for-propdef-align-content⑨"></a>[align-content](https://drafts.csswg.org/css-flexbox-1/#propdef-align-content) as being <a id="ref-for-valdef-align-items-flex-start②"></a>[flex-start](https://drafts.csswg.org/css-flexbox-1/#valdef-align-items-flex-start) for all item fragments and lines.

5.  If any item, when aligned according to its original <a id="ref-for-propdef-align-self①⑨"></a>[align-self](https://drafts.csswg.org/css-flexbox-1/#propdef-align-self) value into the combined <a id="ref-for-cross-size③⑥"></a>[cross size](https://drafts.csswg.org/css-flexbox-1/#cross-size) of all the flex container fragments, would fit entirely within a single flex container fragment, it may be shifted into that fragment and aligned appropriately.

<strong><a id="ref-for-multi-line-flex-container①⑥"></a>[multi-line](https://drafts.csswg.org/css-flexbox-1/#multi-line-flex-container) row flex container</strong>

1.  Run the flex layout algorithm (without regards to pagination), through [Cross Sizing Determination](https://drafts.csswg.org/css-flexbox-1/#cross-sizing).

2.  Lay out as many flex lines as possible (but at least one), starting from the first, until there is no more room on the page or a forced break is encountered.

    If a line doesn’t fit on the page, and the line is not at the top of the page, move the line to the next page and restart the flex layout algorithm entirely, using only the items in and following this line.

    If a flex item itself causes a forced break, rerun the flex layout algorithm from [Main Sizing Determination](https://drafts.csswg.org/css-flexbox-1/#main-sizing) through [Main-Axis Alignment](https://drafts.csswg.org/css-flexbox-1/#main-alignment), using only the items on this and following lines, but with the item causing the break automatically starting a new line in the [line breaking step](https://drafts.csswg.org/css-flexbox-1/#algo-line-break), then continue with this step. Forced breaks <em>within</em> flex items are ignored.

3.  If there are any flex items not laid out by the previous step, rerun the flex layout algorithm from [Line Length Determination](https://drafts.csswg.org/css-flexbox-1/#line-sizing) through [Main-Axis Alignment](https://drafts.csswg.org/css-flexbox-1/#main-alignment) with the next page’s size and only the items not already laid out. Return to the previous step, but starting from the first line not already laid out.

4.  For each fragment of the flex container, continue the flex layout algorithm from [Cross Axis Alignment](https://drafts.csswg.org/css-flexbox-1/#cross-alignment) to its finish.
