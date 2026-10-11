Attribution and reformatting notice added for Surgeist on 2026-10-11

This bounded reformatted excerpt accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material copied from [CSS Conditional Rules Module Level 5](https://drafts.csswg.org/css-conditional-5/) under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Original copyright, liability, trademark and permissive document-license notice retained below.

# Provenance and boundary

Captured 2026-10-11. Full HTML SHA-256 `3b4a4b4ef336edeaca211cdb90481bfcf9cda1955c1cabce485b98e2fcb497e3`; 494912 bytes. Bounded HTML SHA-256 `bc340aca45c3f10dc4aba9b26d6e0acfd57ff161cb3e03afd7baa3d0254b3271`; 36199 bytes. Source revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`. Retained complete heading sections: `container-lengths`, each through the next equal-or-higher heading, plus source header/status including warnings. Other sections resolve through upstream links. This editor capture is work in progress; Surgeist policies are recorded separately and do not change the captured source.

Converted with the existing HTML preparation, Pandoc 3.1.11.1 and semantic filter.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Conditional Rules Module Level 5

<a id="w3c-state"></a>[W3C Working Draft](https://www.w3.org/standards/types/#WD), 8 October 2026

More details about this document

<strong>This version:</strong>

<https://www.w3.org/TR/2026/WD-css-conditional-5-20261008/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-conditional-5/>

<strong>Editor's Draft:</strong>

<https://drafts.csswg.org/css-conditional-5/>

<strong>History:</strong>

<https://www.w3.org/standards/history/css-conditional-5/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-conditional-5)

[Inline In Spec](https://drafts.csswg.org/css-conditional-5/#issues-index)

<strong>Editors:</strong>

[L. David Baron](https://dbaron.org/) ([Google](https://www.google.com/))

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Chris Lilley](https://svgees.us/) (W3C)

[Miriam E. Suzanne](https://www.miriamsuzanne.com/who/) (Invited Expert)

[Lea Verou](https://lea.verou.me/about) (Invited Expert)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-conditional-5/Overview.bs)

<strong>Delta Spec:</strong>

yes

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-conditional/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication. A list of current W3C publications and the latest revision of this technical report can be found in the [W3C standards and drafts index.](https://www.w3.org/TR/)</em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-conditional” in the title, like this: “\[css-conditional\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-conditional%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="container-lengths"></a>7.  Container Relative Lengths: the cqw, cqh, cqi, cqb, cqmin, cqmax units[](#container-lengths)

<a id="container-query-length"></a><strong>Container query length units</strong> specify a length relative to the dimensions of a <a id="ref-for-query-container⑥⑤"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container). Style sheets that use <a id="ref-for-container-query-length②"></a>[container query length](#container-query-length) units can more easily move components from one <a id="ref-for-query-container⑥⑥"></a>query container to another.

The <a id="ref-for-container-query-length③"></a>[container query length](#container-query-length) units are:

Informative Summary of Container Units

| unit  | relative to                                                                                                                                                                                                                   |
|-------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| cqw   | 1% of a <a id="ref-for-query-container⑥⑦"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container)’s <a id="ref-for-width①"></a>[width](https://www.w3.org/TR/css-sizing-3/#width)                  |
| cqh   | 1% of a <a id="ref-for-query-container⑥⑧"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container)’s <a id="ref-for-height①"></a>[height](https://www.w3.org/TR/css-sizing-3/#height)               |
| cqi   | 1% of a <a id="ref-for-query-container⑥⑨"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container)’s <a id="ref-for-inline-size"></a>[inline size](https://www.w3.org/TR/css-sizing-3/#inline-size) |
| cqb   | 1% of a <a id="ref-for-query-container⑦⓪"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container)’s <a id="ref-for-block-size"></a>[block size](https://www.w3.org/TR/css-sizing-3/#block-size)    |
| cqmin | The smaller value of cqi or cqb                                                                                                                                                                                               |
| cqmax | The larger value of cqi or cqb                                                                                                                                                                                                |

For each element, <a id="ref-for-container-query-length④"></a>[container query length](#container-query-length) units are evaluated using the same rules as <a id="ref-for-container-size-query⑨"></a>[container size queries](https://drafts.csswg.org/css-conditional-5/#container-size-query) on the relevant axis (or axes) described by the unit. The <a id="ref-for-query-container⑦①"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container) for each axis is the nearest ancestor container that accepts <a id="ref-for-container-size-query①⓪"></a>container size queries on that axis. If no eligible <a id="ref-for-query-container⑦②"></a>query container is available, then use the <a id="ref-for-small-viewport-size"></a>[small viewport size](https://www.w3.org/TR/css-values-4/#small-viewport-size) for that axis.

Note: In some cases cqi and cqb units on the same element will evaluate in relation to different <a id="ref-for-query-container⑦③"></a>[query containers](https://drafts.csswg.org/css-conditional-5/#query-container). Similarly, cqmin and cqmax units represent the larger or smaller of the cqi and cqb units, even when those dimensions come from different <a id="ref-for-query-container⑦④"></a>query containers.

Child elements do not inherit the relative values as specified for their parent; they inherit the <a id="ref-for-computed-value①②"></a>[computed values](https://www.w3.org/TR/css-cascade-5/#computed-value).

<a id="example-a4252068"></a>

<strong>Example:</strong>

[](#example-a4252068) Authors can ensure that <a id="ref-for-container-query-length⑤"></a>[container query length](#container-query-length) units have an appropriate <a id="ref-for-query-container⑦⑤"></a>[query container](https://drafts.csswg.org/css-conditional-5/#query-container) by applying them inside a <a id="ref-for-container-query①④"></a>[container query](https://drafts.csswg.org/css-conditional-5/#container-query) that relies on the same container-type. Custom fallback values can be defined outside the <a id="ref-for-container-query①⑤"></a>container query:

``` text
/* The fallback value does not rely on containment */
h2 { font-size: 1.2em; }

@container (inline-size >= 0px) {
  /* only applies when an inline-size container is available */
  h2 { font-size: calc(1.2em + 1cqi); }
}
```
