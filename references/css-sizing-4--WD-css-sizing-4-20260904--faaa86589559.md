Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Box Sizing Module Level 4](https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Box Sizing Module Level 4

Source snapshot: https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/

Snapshot SHA-256: faaa865895590c6c523bd5faf127ba940886d5113cef822f8d08eb0aac2ea468

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 11 source tables are presented as readable Markdown tables or explicit labeled layouts: 11 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.

---

# <a id="title"></a>CSS Box Sizing Module Level 4

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This module extends the CSS sizing properties with keywords that represent content-based "intrinsic" sizes and context-based "extrinsic" sizes, allowing CSS to more easily describe boxes that fit their content or fit into a particular layout context. This is a delta spec over CSS Sizing Level 3.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-sizing” in the title, like this: “\[css-sizing\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-sizing%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f7037e82"></a> This is a diff spec over [CSS Sizing Level 3](https://www.w3.org/TR/css-sizing-3/). It is currently an Exploratory Working Draft: if you are implementing anything, please use Level 3 as a reference. We will merge the Level 3 text into this draft once it reaches CR.

Tests

General sizing tests

- [dynamic-available-size-iframe.html](https://wpt.fyi/results/css/css-sizing/dynamic-available-size-iframe.html) [(live test)](http://wpt.live/css/css-sizing/dynamic-available-size-iframe.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/dynamic-available-size-iframe.html)
- [dynamic-change-inline-size-001.html](https://wpt.fyi/results/css/css-sizing/dynamic-change-inline-size-001.html) [(live test)](http://wpt.live/css/css-sizing/dynamic-change-inline-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/dynamic-change-inline-size-001.html)
- [dynamic-change-inline-size-002.html](https://wpt.fyi/results/css/css-sizing/dynamic-change-inline-size-002.html) [(live test)](http://wpt.live/css/css-sizing/dynamic-change-inline-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/dynamic-change-inline-size-002.html)
- [dynamic-change-inline-size-003.html](https://wpt.fyi/results/css/css-sizing/dynamic-change-inline-size-003.html) [(live test)](http://wpt.live/css/css-sizing/dynamic-change-inline-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/dynamic-change-inline-size-003.html)
- [dynamic-change-inline-size-004.html](https://wpt.fyi/results/css/css-sizing/dynamic-change-inline-size-004.html) [(live test)](http://wpt.live/css/css-sizing/dynamic-change-inline-size-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/dynamic-change-inline-size-004.html)
- [frameset-intrinsic-crash.html](https://wpt.fyi/results/css/css-sizing/frameset-intrinsic-crash.html) [(live test)](http://wpt.live/css/css-sizing/frameset-intrinsic-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/frameset-intrinsic-crash.html)
- [inheritance-001.html](https://wpt.fyi/results/css/css-sizing/inheritance-001.html) [(live test)](http://wpt.live/css/css-sizing/inheritance-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/inheritance-001.html)
- [inheritance-002.html](https://wpt.fyi/results/css/css-sizing/inheritance-002.html) [(live test)](http://wpt.live/css/css-sizing/inheritance-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/inheritance-002.html)
- [min-width-max-width-precedence.html](https://wpt.fyi/results/css/css-sizing/min-width-max-width-precedence.html) [(live test)](http://wpt.live/css/css-sizing/min-width-max-width-precedence.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/min-width-max-width-precedence.html)
- [min-width-max-width-precedence.html](https://wpt.fyi/results/css/css-sizing/min-width-max-width-precedence.html) [(live test)](http://wpt.live/css/css-sizing/min-width-max-width-precedence.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/min-width-max-width-precedence.html)
- [replaced-max-size-saturation.html](https://wpt.fyi/results/css/css-sizing/replaced-max-size-saturation.html) [(live test)](http://wpt.live/css/css-sizing/replaced-max-size-saturation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/replaced-max-size-saturation.html)
- [responsive-iframe-no-match-element.html](https://wpt.fyi/results/css/css-sizing/responsive-iframe/responsive-iframe-no-match-element.html) [(live test)](http://wpt.live/css/css-sizing/responsive-iframe/responsive-iframe-no-match-element.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/responsive-iframe/responsive-iframe-no-match-element.html)
- [textarea-large-padding-crash.html](https://wpt.fyi/results/css/css-sizing/textarea-large-padding-crash.html) [(live test)](http://wpt.live/css/css-sizing/textarea-large-padding-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/textarea-large-padding-crash.html)

------------------------------------------------------------------------

### <a id="placement"></a>1.1.  Module interactions

<a id="ref-for-propdef-width"></a>

<a id="ref-for-propdef-height"></a>

<a id="ref-for-propdef-min-width"></a>

<a id="ref-for-propdef-min-height"></a>

<a id="ref-for-propdef-max-width"></a>

<a id="ref-for-propdef-max-height"></a>

<a id="ref-for-propdef-column-width"></a>

This module extends the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width), [height](https://www.w3.org/TR/css-sizing-3/#propdef-height), [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width), [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height), [max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width), [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height), and [column-width](https://www.w3.org/TR/css-multicol-2/#propdef-column-width) features defined in [\[CSS2\]](#biblio-css2) chapter 10 and in [\[CSS3COL\]](#biblio-css3col)

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units [\[CSS-VALUES-3\]](#biblio-css-values-3). Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="terms"></a>2.  Terminology

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9"></a> [CSS Sizing 3 § 2 Terminology](https://www.w3.org/TR/css-sizing-3/#terms)

## <a id="specifying-sizes"></a>3. <a id="size-keywords"></a> Specifying Box Sizes

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9①"></a> [CSS Sizing 3 § 3 Specifying Box Sizes](https://www.w3.org/TR/css-sizing-3/#specifying-sizes)

### <a id="sizing-properties"></a>3.1.  Sizing Properties

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9②"></a> [CSS Sizing 3 § 3.1 Sizing Properties](https://www.w3.org/TR/css-sizing-3/#sizing-properties)

| Field               | Definition                                                                                                                                                                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-size"></a>size                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt"></a><a id="ref-for-propdef-height①"></a><a id="ref-for-propdef-width①"></a>[\<'width'\>](https://www.w3.org/TR/css-sizing-3/#propdef-width) [\<'height'\>](https://www.w3.org/TR/css-sizing-3/#propdef-height)[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                   |

<a id="ref-for-propdef-size"></a>

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-propdef-width②"></a>

<a id="ref-for-propdef-height②"></a>

The [size](#propdef-size) property is a [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) that sets [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) in a single declaration. If the second value is omitted, it is copied from the first.

<a id="ref-for-propdef-size①"></a>

<a id="ref-for-at-ruledef-page"></a>

<a id="ref-for-descdef-page-size"></a>

The [size](#propdef-size) shorthand cannot be used in [@page](https://www.w3.org/TR/css-page-3/#at-ruledef-page) contexts, as <a id="ref-for-at-ruledef-page①"></a>@page already has an unrelated [size](https://www.w3.org/TR/css-page-3/#descdef-page-size) descriptor for setting the page size.

<a id="ref-for-propdef-size②"></a>

<a id="ref-for-at-ruledef-page②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-bded40fa"></a> The [size](#propdef-size) property needs to be omitted from the <u>preferred shorthand order</u> in CSSOM, to avoid compat problems and conflicts with [@page](https://www.w3.org/TR/css-page-3/#at-ruledef-page).

| Field               | Definition                                                                                                                                                                                                                                                            |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-min-size"></a>min-size                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt①"></a><a id="ref-for-propdef-min-height①"></a><a id="ref-for-propdef-min-width①"></a>[\<'min-width'\>](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width) [\<'min-height'\>](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height)[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                           |

<a id="ref-for-propdef-min-size"></a>

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-propdef-min-width②"></a>

<a id="ref-for-propdef-min-height②"></a>

The [min-size](#propdef-min-size) property is a [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) that sets [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width) and [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) in a single declaration. If the second value is omitted, it is copied from the first.

| Field               | Definition                                                                                                                                                                                                                                                            |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-max-size"></a>max-size                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt②"></a><a id="ref-for-propdef-max-height①"></a><a id="ref-for-propdef-max-width①"></a>[\<'max-width'\>](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width) [\<'max-height'\>](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height)[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                           |

<a id="ref-for-propdef-max-size"></a>

<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-propdef-max-width②"></a>

<a id="ref-for-propdef-max-height②"></a>

The [max-size](#propdef-max-size) property is a [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) that sets [max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width) and [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height) in a single declaration. If the second value is omitted, it is copied from the first.

<a id="ref-for-valdef-width-contain"></a>

<a id="ref-for-funcdef-width-fit-content"></a>

<a id="ref-for-funcdef-width-calc-size"></a>

### <a id="sizing-values"></a>3.2. <a id="width-height-keywords"></a> New Sizing Values: the [contain](#valdef-width-contain), [fit-content()](#funcdef-width-fit-content), and [calc-size()](#funcdef-width-calc-size) values

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9③"></a> [CSS Sizing 3 § 3.2 Sizing Values: the \<length-percentage\>, auto \| none, min-content, max-content, and fit-content() values](https://www.w3.org/TR/css-sizing-3/#sizing-values)

<a id="ref-for-valdef-width-contain①"></a>

<a id="ref-for-funcdef-width-fit-content①"></a>

<a id="ref-for-funcdef-width-calc-size①"></a>

<a id="ref-for-sizing-property"></a>

<a id="ref-for-typedef-box-size"></a>

Level 4 adds the ability to use the [contain](#valdef-width-contain), [fit-content()](#funcdef-width-fit-content), and [calc-size()](#funcdef-width-calc-size) values in the [sizing properties](https://www.w3.org/TR/css-sizing-3/#sizing-property), thus expanding the [\<box-size\>](#typedef-box-size) production as follows:

<a id="typedef-box-size"></a>

<a id="ref-for-typedef-box-size①"></a>

<a id="ref-for-typedef-length-percentage"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-comb-one⑥"></a>

```text
<box-size> = <length-percentage> | stretch | contain | min-content | max-content | fit-content | fit-content() | calc-size()
```

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="ref-for-propdef-max-height③"></a><a id="ref-for-propdef-max-width③"></a><a id="ref-for-propdef-min-height③"></a><a id="ref-for-propdef-min-width③"></a><a id="ref-for-propdef-height③"></a><a id="ref-for-propdef-width③"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width), [height](https://www.w3.org/TR/css-sizing-3/#propdef-height), [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width), [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height), [max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width), [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height) |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">New values:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①"></a><a id="ref-for-comb-one⑦"></a>contain [\|](https://www.w3.org/TR/css-values-4/#comb-one) fit-content([\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage))                                                                                                                                                                                                                                                                                                                                   |

<a id="ref-for-typedef-length-percentage②"></a>

<a id="funcdef-width-fit-content"></a>fit-content([\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage))

<a id="ref-for-typedef-length-percentage④"></a>

<a id="ref-for-typedef-length-percentage③"></a>

<a id="ref-for-valdef-width-min-content"></a>

<a id="ref-for-valdef-width-max-content"></a>

<a id="ref-for-available"></a>

Use the fit-content formula with the [available space](https://www.w3.org/TR/css-sizing-3/#available) replaced by the specified argument, i.e. <code>min(<a href="https://www.w3.org/TR/css-sizing-3/#valdef-width-max-content">max-content</a>, max(<a href="https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content">min-content</a>, <a href="https://www.w3.org/TR/css-values-4/#typedef-length-percentage">&lt;length-percentage&gt;</a>))</code>, where the [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) argument is resolved exactly as for <a id="ref-for-typedef-length-percentage⑤"></a>\<length-percentage\> values standing alone.

<a id="ref-for-typedef-length-percentage⑥"></a>

Negative [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values are invalid.

Tests

- [auto-scrollbar-inside-stf-abspos.html](https://wpt.fyi/results/css/css-sizing/auto-scrollbar-inside-stf-abspos.html) [(live test)](http://wpt.live/css/css-sizing/auto-scrollbar-inside-stf-abspos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/auto-scrollbar-inside-stf-abspos.html)
- [block-fit-content-as-initial.html](https://wpt.fyi/results/css/css-sizing/block-fit-content-as-initial.html) [(live test)](http://wpt.live/css/css-sizing/block-fit-content-as-initial.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/block-fit-content-as-initial.html)
- [fit-content-length-percentage-001.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-001.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-001.html)
- [fit-content-length-percentage-002.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-002.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-002.html)
- [fit-content-length-percentage-003.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-003.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-003.html)
- [fit-content-length-percentage-004.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-004.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-004.html)
- [fit-content-length-percentage-005.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-005.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-005.html)
- [fit-content-length-percentage-006.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-006.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-006.html)
- [fit-content-length-percentage-007.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-007.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-007.html)
- [fit-content-length-percentage-008.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-008.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-008.html)
- [fit-content-length-percentage-009.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-009.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-009.html)
- [fit-content-length-percentage-010.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-010.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-010.html)
- [fit-content-length-percentage-011.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-011.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-011.html)
- [fit-content-length-percentage-012.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-012.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-012.html)
- [fit-content-length-percentage-013.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-013.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-013.html)
- [fit-content-length-percentage-014.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-014.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-014.html)
- [fit-content-length-percentage-015.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-015.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-015.html)
- [fit-content-length-percentage-016.html](https://wpt.fyi/results/css/css-sizing/fit-content-length-percentage-016.html) [(live test)](http://wpt.live/css/css-sizing/fit-content-length-percentage-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/fit-content-length-percentage-016.html)

<a id="valdef-width-contain"></a>contain

<a id="ref-for-contain-fit-sizing①"></a>

<a id="ref-for-contain-fit-sizing"></a>

<a id="ref-for-preferred-aspect-ratio"></a>

If the box has a [preferred aspect ratio](#preferred-aspect-ratio), applies [contain-fit sizing](#contain-fit-sizing), attempting to fit into the box’s constraints while maintaining its <a id="ref-for-preferred-aspect-ratio①"></a>preferred aspect ratio insofar as possible. See [§ 6.1 Contain-fit Sizing: stretching while maintaining an aspect ratio](#contain-fit-sizing).

<a id="ref-for-preferred-aspect-ratio②"></a>

<a id="ref-for-stretch-fit-size"></a>

If the box has no [preferred aspect ratio](#preferred-aspect-ratio), applies [stretch-fit sizing](https://www.w3.org/TR/css-sizing-3/#stretch-fit-size).

<a id="funcdef-width-calc-size"></a>calc-size()

See [CSS Values 5 §  10. Calculating With Intrinsic Sizes: the calc-size() function](https://www.w3.org/TR/css-values-5/#calc-size).

<a id="ref-for-valdef-max-width-none"></a>

<a id="ref-for-funcdef-width-calc-size②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [none](https://www.w3.org/TR/css-sizing-3/#valdef-max-width-none) keyword is not usable within [calc-size()](#funcdef-width-calc-size).

<a id="ref-for-funcdef-width-calc-size③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These new values add to the set of values that the definition of [\<calc-size()\>](#funcdef-width-calc-size) refers to as “allowed in the context”.

## <a id="ratios"></a>4.  Aspect Ratios

<a id="ref-for-natural-aspect-ratio"></a>

Images often have a [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio), which the CSS layout algorithms attempt to preserve as they resize the element.

<a id="ref-for-propdef-aspect-ratio"></a>

The [aspect-ratio](#propdef-aspect-ratio) property allows specifying this behavior for non-replaced elements, and for altering the effective aspect ratio of replaced elements.

<a id="ref-for-replaced-element"></a>

<a id="ref-for-preferred-aspect-ratio③"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9c24ea7a"></a> We are still working through the details of this section. If there is any behavior specified here that would cause [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element) with a [preferred aspect ratio](#preferred-aspect-ratio) to behave differently than they would under the requirements of the [CSS2](https://www.w3.org/TR/CSS2/visudet.html), [Flex Layout](https://www.w3.org/TR/css-flexbox-1/), and [Grid Layout](https://www.w3.org/TR/css-grid-1/) specs combined (without this specification in effect), <strong>this is an error and should be <a href="https://github.com/w3c/csswg-drafts/issues">reported</a> to the CSSWG</strong>. There is a [list of open aspect-ratio issues](https://github.com/w3c/csswg-drafts/issues?q=is%3Aissue%20state%3Aopen%20label%3Acss-sizing-4%20aspect-ratio%20label%3A%22Needs%20Edits%22).

<a id="ref-for-propdef-aspect-ratio①"></a>

### <a id="aspect-ratio"></a>4.1.  Preferred Aspect Ratios: the [aspect-ratio](#propdef-aspect-ratio) property

| Field               | Definition                                                                                                                                                   |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-aspect-ratio"></a>aspect-ratio                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-ratio-value"></a><a id="ref-for-comb-any"></a>auto [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-inline-box"></a>all elements except [inline boxes](https://www.w3.org/TR/css-display-4/#inline-box) and internal ruby or table boxes                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword or a pair of numbers                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                            |

Tests

- [aspect-ratio-interpolation.html](https://wpt.fyi/results/css/css-sizing/animation/aspect-ratio-interpolation.html) [(live test)](http://wpt.live/css/css-sizing/animation/aspect-ratio-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/animation/aspect-ratio-interpolation.html)
- [abspos-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-001.html)
- [abspos-002.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-002.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-002.html)
- [abspos-003.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-003.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-003.html)
- [abspos-004.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-004.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-004.html)
- [abspos-005.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-005.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-005.html)
- [abspos-006.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-006.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-006.html)
- [abspos-007.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-007.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-007.html)
- [abspos-008.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-008.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-008.html)
- [abspos-009.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-009.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-009.html)
- [abspos-010.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-010.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-010.html)
- [abspos-011.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-011.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-011.html)
- [abspos-012.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-012.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-012.html)
- [abspos-013.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-013.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-013.html)
- [abspos-014.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-014.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-014.html)
- [abspos-015.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-015.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-015.html)
- [abspos-016.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-016.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-016.html)
- [abspos-017.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-017.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-017.html)
- [abspos-018.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-018.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-018.html)
- [abspos-019.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-019.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-019.html)
- [abspos-020.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-020.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-020.html)
- [abspos-021.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/abspos-021.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/abspos-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/abspos-021.html)
- [auto-margins-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/auto-margins-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/auto-margins-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/auto-margins-001.html)
- [block-aspect-ratio-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-001.html)
- [block-aspect-ratio-002.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-002.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-002.html)
- [block-aspect-ratio-003.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-003.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-003.html)
- [block-aspect-ratio-004.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-004.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-004.html)
- [block-aspect-ratio-005.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-005.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-005.html)
- [block-aspect-ratio-006.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-006.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-006.html)
- [block-aspect-ratio-007.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-007.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-007.html)
- [block-aspect-ratio-008.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-008.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-008.html)
- [block-aspect-ratio-009.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-009.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-009.html)
- [block-aspect-ratio-010.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-010.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-010.html)
- [block-aspect-ratio-011.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-011.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-011.html)
- [block-aspect-ratio-012.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-012.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-012.html)
- [block-aspect-ratio-013.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-013.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-013.html)
- [block-aspect-ratio-014.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-014.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-014.html)
- [block-aspect-ratio-015.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-015.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-015.html)
- [block-aspect-ratio-016.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-016.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-016.html)
- [block-aspect-ratio-017.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-017.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-017.html)
- [block-aspect-ratio-018.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-018.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-018.html)
- [block-aspect-ratio-019.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-019.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-019.html)
- [block-aspect-ratio-020.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-020.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-020.html)
- [block-aspect-ratio-021.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-021.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-021.html)
- [block-aspect-ratio-022.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-022.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-022.html)
- [block-aspect-ratio-023.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-023.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-023.html)
- [block-aspect-ratio-024.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-024.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-024.html)
- [block-aspect-ratio-025.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-025.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-025.html)
- [block-aspect-ratio-026.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-026.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-026.html)
- [block-aspect-ratio-027.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-027.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-027.html)
- [block-aspect-ratio-028.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-028.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-028.html)
- [block-aspect-ratio-029-crash.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-029-crash.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-029-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-029-crash.html)
- [block-aspect-ratio-030.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-030.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-030.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-030.html)
- [block-aspect-ratio-031.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-031.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-031.html)
- [block-aspect-ratio-032.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-032.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-032.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-032.html)
- [block-aspect-ratio-033.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-033.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-033.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-033.html)
- [block-aspect-ratio-034.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-034.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-034.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-034.html)
- [block-aspect-ratio-035.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-035.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-035.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-035.html)
- [block-aspect-ratio-036.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-036.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-036.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-036.html)
- [block-aspect-ratio-037.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-037.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-037.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-037.html)
- [block-aspect-ratio-051-crash.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-051-crash.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-051-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-051-crash.html)
- [block-aspect-ratio-052.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-052.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-052.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-052.html)
- [block-aspect-ratio-053.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-053.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-053.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-053.html)
- [block-aspect-ratio-054.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-054.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-054.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-054.html)
- [block-aspect-ratio-055.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-055.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-055.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-055.html)
- [block-aspect-ratio-056.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-056.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-056.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-056.html)
- [block-aspect-ratio-058.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-058.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-058.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-058.html)
- [block-aspect-ratio-with-margin-collapsing-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-with-margin-collapsing-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-with-margin-collapsing-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-with-margin-collapsing-001.html)
- [block-aspect-ratio-with-margin-collapsing-002.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-with-margin-collapsing-002.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-with-margin-collapsing-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-with-margin-collapsing-002.html)
- [box-sizing-dimensions.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/box-sizing-dimensions.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/box-sizing-dimensions.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/box-sizing-dimensions.html)
- [box-sizing-squashed.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/box-sizing-squashed.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/box-sizing-squashed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/box-sizing-squashed.html)
- [flex-aspect-ratio-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-001.html)
- [flex-aspect-ratio-003.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-003.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-003.html)
- [flex-aspect-ratio-005.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-005.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-005.html)
- [flex-aspect-ratio-006.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-006.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-006.html)
- [flex-aspect-ratio-007.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-007.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-007.html)
- [flex-aspect-ratio-008.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-008.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-008.html)
- [flex-aspect-ratio-009.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-009.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-009.html)
- [flex-aspect-ratio-010.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-010.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-010.html)
- [flex-aspect-ratio-011.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-011.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-011.html)
- [flex-aspect-ratio-012.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-012.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-012.html)
- [flex-aspect-ratio-013.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-013.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-013.html)
- [flex-aspect-ratio-014.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-014.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-014.html)
- [flex-aspect-ratio-015.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-015.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-015.html)
- [flex-aspect-ratio-016.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-016.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-016.html)
- [flex-aspect-ratio-017.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-017.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-017.html)
- [flex-aspect-ratio-018.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-018.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-018.html)
- [flex-aspect-ratio-019.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-019.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-019.html)
- [flex-aspect-ratio-020.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-020.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-020.html)
- [flex-aspect-ratio-021.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-021.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-021.html)
- [flex-aspect-ratio-022.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-022.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-022.html)
- [flex-aspect-ratio-023.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-023.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-023.html)
- [flex-aspect-ratio-024.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-024.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-024.html)
- [flex-aspect-ratio-027.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-027.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-027.html)
- [flex-aspect-ratio-028.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-028.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-028.html)
- [flex-aspect-ratio-029.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-029.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-029.html)
- [flex-aspect-ratio-030.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-030.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-030.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-030.html)
- [flex-aspect-ratio-032.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-032.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-032.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-032.html)
- [flex-aspect-ratio-033.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-033.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-033.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-033.html)
- [flex-aspect-ratio-034.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-034.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-034.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-034.html)
- [flex-aspect-ratio-035.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-035.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-035.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-035.html)
- [flex-aspect-ratio-036.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-036.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-036.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-036.html)
- [flex-aspect-ratio-037.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-037.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-037.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-037.html)
- [flex-aspect-ratio-038.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-038.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-038.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-038.html)
- [flex-aspect-ratio-045.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-045.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-045.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-045.html)
- [flex-aspect-ratio-046.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-046.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-046.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-046.html)
- [flex-aspect-ratio-047.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-047.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-047.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-047.html)
- [flex-aspect-ratio-048.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-048.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-048.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-048.html)
- [flex-aspect-ratio-049.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-049.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-049.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-049.html)
- [flex-aspect-ratio-050.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-050.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-050.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-050.html)
- [flex-aspect-ratio-051.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-051.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-051.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-051.html)
- [flex-aspect-ratio-052.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-052.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-052.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-052.html)
- [flex-aspect-ratio-053.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-053.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-053.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-053.html)
- [flex-aspect-ratio-054.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-054.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-054.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-054.html)
- [flex-aspect-ratio-055.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-055.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-055.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-055.html)
- [floats-aspect-ratio-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/floats-aspect-ratio-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/floats-aspect-ratio-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/floats-aspect-ratio-001.html)
- [fractional-aspect-ratio.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/fractional-aspect-ratio.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/fractional-aspect-ratio.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/fractional-aspect-ratio.html)
- [grid-aspect-ratio-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-001.html)
- [grid-aspect-ratio-002.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-002.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-002.html)
- [grid-aspect-ratio-003.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-003.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-003.html)
- [grid-aspect-ratio-004.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-004.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-004.html)
- [grid-aspect-ratio-005.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-005.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-005.html)
- [grid-aspect-ratio-006.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-006.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-006.html)
- [grid-aspect-ratio-007.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-007.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-007.html)
- [grid-aspect-ratio-008.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-008.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-008.html)
- [grid-aspect-ratio-009.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-009.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-009.html)
- [grid-aspect-ratio-010.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-010.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-010.html)
- [grid-aspect-ratio-011.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-011.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-011.html)
- [grid-aspect-ratio-012.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-012.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-012.html)
- [grid-aspect-ratio-014.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-014.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-014.html)
- [grid-aspect-ratio-015.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-015.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-015.html)
- [grid-aspect-ratio-016.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-016.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-016.html)
- [grid-aspect-ratio-017.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-017.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-017.html)
- [grid-aspect-ratio-018.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-018.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-018.html)
- [grid-aspect-ratio-019.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-019.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-019.html)
- [grid-aspect-ratio-020.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-020.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-020.html)
- [grid-aspect-ratio-021.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-021.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-021.html)
- [grid-aspect-ratio-022.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-022.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-022.html)
- [grid-aspect-ratio-023.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-023.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-023.html)
- [grid-aspect-ratio-024.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-024.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-024.html)
- [grid-aspect-ratio-025.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-025.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-025.html)
- [grid-aspect-ratio-026.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-026.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-026.html)
- [grid-aspect-ratio-027.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-027.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-027.html)
- [grid-aspect-ratio-028.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-028.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-028.html)
- [grid-aspect-ratio-029.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-029.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-029.html)
- [grid-aspect-ratio-030.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-030.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-030.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-030.html)
- [grid-aspect-ratio-031.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-031.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-031.html)
- [grid-aspect-ratio-032.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-032.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-032.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-032.html)
- [grid-aspect-ratio-033.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-033.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-033.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-033.html)
- [grid-aspect-ratio-034.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-034.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-034.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-034.html)
- [grid-aspect-ratio-035.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-035.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-035.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-035.html)
- [grid-aspect-ratio-036.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-036.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-036.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-036.html)
- [grid-aspect-ratio-037.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-037.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-037.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-037.html)
- [grid-aspect-ratio-038.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-038.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-038.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-038.html)
- [grid-aspect-ratio-040.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-040.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-040.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-040.html)
- [grid-aspect-ratio-041.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-041.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-041.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-041.html)
- [inheritance.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/inheritance.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/inheritance.html)
- [intrinsic-size-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-001.html)
- [intrinsic-size-002.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-002.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-002.html)
- [intrinsic-size-003.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-003.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-003.html)
- [intrinsic-size-004.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-004.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-004.html)
- [intrinsic-size-005.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-005.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-005.html)
- [intrinsic-size-006.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-006.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-006.html)
- [intrinsic-size-007.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-007.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-007.html)
- [intrinsic-size-008.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-008.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-008.html)
- [intrinsic-size-009.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-009.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-009.html)
- [intrinsic-size-011.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-011.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-011.html)
- [intrinsic-size-012.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-012.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-012.html)
- [intrinsic-size-013.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-013.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-013.html)
- [intrinsic-size-014.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-014.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-014.html)
- [intrinsic-size-015.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-015.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-015.html)
- [intrinsic-size-016.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-016.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-016.html)
- [intrinsic-size-017.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-017.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-017.html)
- [intrinsic-size-018.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-018.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-018.html)
- [intrinsic-size-019.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-019.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-019.html)
- [intrinsic-size-020.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-020.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-020.html)
- [intrinsic-size-021.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-021.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-021.html)
- [intrinsic-size-022.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-022.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-022.html)
- [intrinsic-size-023.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-023.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-023.html)
- [intrinsic-size-024.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-024.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-024.html)
- [intrinsic-size-025.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-025.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-025.html)
- [large-aspect-ratio-crash.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/large-aspect-ratio-crash.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/large-aspect-ratio-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/large-aspect-ratio-crash.html)
- [aspect-ratio-computed.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/parsing/aspect-ratio-computed.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/parsing/aspect-ratio-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/parsing/aspect-ratio-computed.html)
- [aspect-ratio-invalid.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/parsing/aspect-ratio-invalid.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/parsing/aspect-ratio-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/parsing/aspect-ratio-invalid.html)
- [aspect-ratio-valid.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/parsing/aspect-ratio-valid.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/parsing/aspect-ratio-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/parsing/aspect-ratio-valid.html)
- [percentage-resolution-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/percentage-resolution-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/percentage-resolution-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/percentage-resolution-001.html)
- [percentage-resolution-002.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/percentage-resolution-002.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/percentage-resolution-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/percentage-resolution-002.html)
- [percentage-resolution-003.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/percentage-resolution-003.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/percentage-resolution-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/percentage-resolution-003.html)
- [percentage-resolution-004.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/percentage-resolution-004.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/percentage-resolution-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/percentage-resolution-004.html)
- [percentage-resolution-005.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/percentage-resolution-005.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/percentage-resolution-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/percentage-resolution-005.html)
- [quirks-mode-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/quirks-mode-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/quirks-mode-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/quirks-mode-001.html)
- [quirks-mode-002.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/quirks-mode-002.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/quirks-mode-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/quirks-mode-002.html)
- [quirks-mode-003.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/quirks-mode-003.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/quirks-mode-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/quirks-mode-003.html)
- [replaced-element-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-001.html)
- [replaced-element-002.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-002.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-002.html)
- [replaced-element-003.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-003.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-003.html)
- [replaced-element-004.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-004.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-004.html)
- [replaced-element-005.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-005.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-005.html)
- [replaced-element-006.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-006.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-006.html)
- [replaced-element-007.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-007.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-007.html)
- [replaced-element-008.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-008.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-008.html)
- [replaced-element-009.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-009.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-009.html)
- [replaced-element-010.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-010.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-010.html)
- [replaced-element-011.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-011.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-011.html)
- [replaced-element-012.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-012.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-012.html)
- [replaced-element-013.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-013.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-013.html)
- [replaced-element-014.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-014.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-014.html)
- [replaced-element-015.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-015.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-015.html)
- [replaced-element-016.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-016.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-016.html)
- [replaced-element-017.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-017.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-017.html)
- [replaced-element-018.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-018.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-018.html)
- [replaced-element-019.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-019.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-019.html)
- [replaced-element-020.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-020.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-020.html)
- [replaced-element-021.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-021.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-021.html)
- [replaced-element-022.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-022.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-022.html)
- [replaced-element-023.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-023.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-023.html)
- [replaced-element-024.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-024.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-024.html)
- [replaced-element-025.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-025.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-025.html)
- [replaced-element-026.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-026.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-026.html)
- [replaced-element-027.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-027.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-027.html)
- [replaced-element-028.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-028.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-028.html)
- [replaced-element-029.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-029.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-029.html)
- [replaced-element-030.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-030.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-030.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-030.html)
- [replaced-element-031.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-031.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-031.html)
- [replaced-element-034.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-034.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-034.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-034.html)
- [replaced-element-035.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-035.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-035.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-035.html)
- [replaced-element-036.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-036.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-036.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-036.html)
- [replaced-element-037.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-037.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-037.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-037.html)
- [replaced-element-042.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-042.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-042.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-042.html)
- [replaced-element-045.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-045.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-045.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-045.html)
- [replaced-element-046.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-046.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-046.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-046.html)
- [replaced-element-049.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-049.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-049.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-049.html)
- [replaced-element-dynamic-aspect-ratio.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-dynamic-aspect-ratio.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-dynamic-aspect-ratio.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-dynamic-aspect-ratio.html)
- [select-element-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/select-element-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/select-element-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/select-element-001.html)
- [sign-function-aspect-ratio.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/sign-function-aspect-ratio.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/sign-function-aspect-ratio.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/sign-function-aspect-ratio.html)
- [small-aspect-ratio-crash.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/small-aspect-ratio-crash.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/small-aspect-ratio-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/small-aspect-ratio-crash.html)
- [table-element-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/table-element-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/table-element-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/table-element-001.html)
- [zero-or-infinity-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/zero-or-infinity-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/zero-or-infinity-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/zero-or-infinity-001.html)
- [zero-or-infinity-002.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/zero-or-infinity-002.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/zero-or-infinity-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/zero-or-infinity-002.html)
- [zero-or-infinity-003.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/zero-or-infinity-003.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/zero-or-infinity-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/zero-or-infinity-003.html)
- [zero-or-infinity-004.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/zero-or-infinity-004.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/zero-or-infinity-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/zero-or-infinity-004.html)
- [zero-or-infinity-005.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/zero-or-infinity-005.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/zero-or-infinity-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/zero-or-infinity-005.html)
- [zero-or-infinity-006.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/zero-or-infinity-006.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/zero-or-infinity-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/zero-or-infinity-006.html)
- [zero-or-infinity-007.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/zero-or-infinity-007.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/zero-or-infinity-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/zero-or-infinity-007.html)
- [zero-or-infinity-008.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/zero-or-infinity-008.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/zero-or-infinity-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/zero-or-infinity-008.html)
- [zero-or-infinity-009.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/zero-or-infinity-009.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/zero-or-infinity-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/zero-or-infinity-009.html)
- [zero-or-infinity-010.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/zero-or-infinity-010.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/zero-or-infinity-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/zero-or-infinity-010.html)

<a id="ref-for-valdef-width-auto"></a>

This property sets a <a id="preferred-aspect-ratio"></a>preferred aspect ratio for the box, which will be used in the calculation of [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) sizes and some other layout functions.

<a id="valdef-aspect-ratio-auto"></a>auto

<a id="ref-for-content-box"></a>

<a id="ref-for-preferred-aspect-ratio④"></a>

<a id="ref-for-natural-aspect-ratio①"></a>

<a id="ref-for-replaced-element①"></a>

[Replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element) with a [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio) use that aspect ratio; otherwise the box has no [preferred aspect ratio](#preferred-aspect-ratio). Size calculations involving the aspect ratio work with the [content box](https://www.w3.org/TR/css-box-4/#content-box) dimensions always.

<a id="ref-for-ratio-value①"></a>

<a id="valdef-aspect-ratio-ratio"></a>[\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value)

<a id="ref-for-propdef-box-sizing"></a>

<a id="ref-for-preferred-aspect-ratio⑤"></a>

The box’s [preferred aspect ratio](#preferred-aspect-ratio) is the specified ratio of <var>width</var> / <var>height</var>. Size calculations involving the aspect ratio work with the dimensions of the box specified by [box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing).

<a id="ref-for-ratio-value②"></a>

<a id="ref-for-degenerate-ratio"></a>

<a id="ref-for-valdef-aspect-ratio-auto"></a>

If the [\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value) is [degenerate](https://www.w3.org/TR/css-values-4/#degenerate-ratio), the property instead behaves as [auto](#valdef-aspect-ratio-auto).

<a id="ref-for-ratio-value③"></a>

<a id="valdef-aspect-ratio-auto--ratio"></a>auto &#x26;&#x26; [\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value)

<a id="ref-for-content-box①"></a>

<a id="ref-for-natural-aspect-ratio②"></a>

<a id="ref-for-replaced-element②"></a>

<a id="ref-for-preferred-aspect-ratio⑥"></a>

<a id="ref-for-ratio-value④"></a>

<a id="ref-for-valdef-aspect-ratio-auto①"></a>

If both [auto](#valdef-aspect-ratio-auto) and a [\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value) are specified together, the [preferred aspect ratio](#preferred-aspect-ratio) is the specified ratio of <var>width</var> / <var>height</var> unless it is a [replaced element](https://www.w3.org/TR/css-display-4/#replaced-element) with a [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio), in which case that aspect ratio is used instead. In all cases, size calculations involving the aspect ratio work with the [content box](https://www.w3.org/TR/css-box-4/#content-box) dimensions always.

<a id="ref-for-ratio-value⑤"></a>

<a id="ref-for-degenerate-ratio①"></a>

<a id="ref-for-valdef-aspect-ratio-auto②"></a>

If the [\<ratio\>](https://www.w3.org/TR/css-values-4/#ratio-value) is [degenerate](https://www.w3.org/TR/css-values-4/#degenerate-ratio), the property instead behaves as [auto](#valdef-aspect-ratio-auto).

Tests

- [block-aspect-ratio-050.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-050.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-050.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-050.html)

<a id="ref-for-preferred-aspect-ratio⑦"></a>

<a id="ref-for-replaced-element③"></a>

<a id="ref-for-non-replaced"></a>

<a id="ref-for-absolute-position"></a>

<a id="ref-for-propdef-justify-self"></a>

<a id="ref-for-valdef-justify-self-stretch"></a>

<a id="ref-for-valdef-self-position-start"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Having a [preferred aspect ratio](#preferred-aspect-ratio) does not make a box into a [replaced element](https://www.w3.org/TR/css-display-4/#replaced-element); layout rules specific to <a id="ref-for-replaced-element④"></a>replaced elements do not generally apply to [non-replaced](https://www.w3.org/TR/css-display-4/#non-replaced) boxes with a <a id="ref-for-preferred-aspect-ratio⑧"></a>preferred aspect ratio. For example, a <a id="ref-for-non-replaced①"></a>non-replaced [absolutely-positioned](https://www.w3.org/TR/css-position-3/#absolute-position) box treats [justify-self: normal](https://www.w3.org/TR/css-align-3/#propdef-justify-self) as [stretch](https://www.w3.org/TR/css-align-3/#valdef-justify-self-stretch), not as [start](https://www.w3.org/TR/css-align-3/#valdef-self-position-start) ([CSS Box Alignment 3 § 6.1.2 Absolutely-Positioned Boxes](https://www.w3.org/TR/css-align-3/#justify-abspos)), even if it has a <a id="ref-for-preferred-aspect-ratio⑨"></a>preferred aspect ratio

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-ada2dbb9"></a> CSS2.1 does not cleanly differentiate between replaced elements vs. elements with an aspect ratio; need to figure out specific cases that are unclear and define them, either in the appropriate Level 3 spec or here.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-95a95149"></a> This example sets each item in the grid to render as a square, determining the number of items and their widths by the available space.
>
> ```text
> <ul>
>   <li>…
>   <li>…
>   <li>…
>   <li>…
> </ul>
> ```
>
> ```text
> ul {
>   display: grid;
>   grid-template-columns: repeat(auto-fill, minmax(12em, 1fr));
> }
> li {
>   aspect-ratio: 1/1;
>   overflow: auto;
> }
> ```
<a id="ref-for-the-iframe-element"></a>

<a id="ref-for-propdef-aspect-ratio②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2993b4a0"></a> This example uses the <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> element’s `width` and `height` attributes to set the [aspect-ratio](#propdef-aspect-ratio) property, giving the iframe an aspect ratio to use for sizing so that it behaves exactly like an image with that aspect ratio.
>
> ```text
> <iframe
>   src="https://www.youtube.com/embed/0Gr1XSyxZy0"
>   width=560
>   height=315>
> ```
>
> ```text
> @supports (aspect-ratio: attr(width number) / 1) {
>   iframe {
>     aspect-ratio: attr(width number) / attr(height number);
>     width: 100%;
>     height: auto;
>   }
> }
> ```
<a id="ref-for-natural-dimensions"></a>

<a id="ref-for-natural-width"></a>

<a id="ref-for-natural-height"></a>

<a id="ref-for-preferred-aspect-ratio①⓪"></a>

If a replaced element’s only [natural dimension](https://www.w3.org/TR/css-images-3/#natural-dimensions) is a [natural width](https://www.w3.org/TR/css-images-3/#natural-width) or a [natural height](https://www.w3.org/TR/css-images-3/#natural-height), giving it a [preferred aspect ratio](#preferred-aspect-ratio) also gives it an <a id="ref-for-natural-dimensions①"></a>natural height or width, whichever was missing, by transferring the existing size through the <a id="ref-for-preferred-aspect-ratio①①"></a>preferred aspect ratio.

### <a id="aspect-ratio-automatic"></a>4.2.  Effects of Preferred Aspect Ratio on Automatic Sizes

<a id="ref-for-preferred-aspect-ratio①②"></a>

<a id="ref-for-automatic-size"></a>

<a id="ref-for-replaced-element⑤"></a>

<a id="ref-for-natural-aspect-ratio③"></a>

<a id="ref-for-natural-size"></a>

<a id="ref-for-preferred-size"></a>

<a id="ref-for-definite"></a>

<a id="ref-for-ratio-dependent-axis"></a>

When a box has a [preferred aspect ratio](#preferred-aspect-ratio), its [automatic sizes](https://www.w3.org/TR/css-sizing-3/#automatic-size) are calculated the same as for a [replaced element](https://www.w3.org/TR/css-display-4/#replaced-element) with a [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio) and no [natural size](https://www.w3.org/TR/css-images-3/#natural-size) in that axis, see e.g. [CSS2 § 10](https://www.w3.org/TR/CSS2/visudet.html) and [CSS Flexible Box Model Level 1 § 9.2](https://www.w3.org/TR/css-flexbox-1/#algo-main-item). The axis in which the [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) calculation depends on this aspect ratio is called the <a id="ratio-dependent-axis"></a>ratio-dependent axis, and the resulting size is [definite](https://www.w3.org/TR/css-sizing-3/#definite) if its input sizes are also <a id="ref-for-definite①"></a>definite. The opposite axis (on which the [ratio-dependent axis](#ratio-dependent-axis) size depends) is the <a id="ratio-determining-axis"></a>ratio-determining axis.

<a id="ref-for-preferred-aspect-ratio①③"></a>

<a id="ref-for-automatic-size①"></a>

<a id="ref-for-propdef-width④"></a>

<a id="ref-for-propdef-height④"></a>

<a id="ref-for-preferred-size①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A [preferred aspect ratio](#preferred-aspect-ratio) only ever has an effect if at least one of the box’s sizes is [automatic](https://www.w3.org/TR/css-sizing-3/#automatic-size). If neither [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) nor [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) is an <a id="ref-for-automatic-size②"></a>automatic size, it can have no effect on its [preferred sizes](https://www.w3.org/TR/css-sizing-3/#preferred-size).

<a id="ref-for-preferred-size②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-36f34fa3"></a> When we move all the sizing information here, rather than crowbar-ing our way into 2.1, then the core principle here is just: the resolved [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) in the ratio-determining axis (before applying min/max) gets transferred thru the ratio. Min/max constraints get transferred afterwards, and then applied to each axis independently without regards to aspect-ratio.

Tests

- [flex-aspect-ratio-031.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-031.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-031.html)
- [grid-aspect-ratio-align-items-center.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-align-items-center.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-align-items-center.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-align-items-center.html)

#### <a id="aspect-ratio-margin-collapse"></a>4.2.1.  Margin-collapsing

<a id="ref-for-block-axis"></a>

<a id="ref-for-ratio-dependent-axis①"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-propdef-block-size"></a>

<a id="ref-for-valdef-width-auto①"></a>

For the purpose of margin collapsing ([CSS 2 § 8.3.1 Collapsing margins](https://www.w3.org/TR/CSS2/box.html#collapsing-margins)), if the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) is the [ratio-dependent axis](#ratio-dependent-axis), it is not considered to have a [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [block-size](https://www.w3.org/TR/css-logical-1/#propdef-block-size) of [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto).

### <a id="aspect-ratio-minimum"></a>4.3.  Automatic Content-based Minimum Sizes

<a id="ref-for-automatic-minimum-size"></a>

<a id="ref-for-ratio-dependent-axis②"></a>

<a id="ref-for-preferred-aspect-ratio①④"></a>

<a id="ref-for-replaced-element⑥"></a>

<a id="ref-for-scroll-container"></a>

<a id="ref-for-min-content"></a>

<a id="ref-for-max-width"></a>

In order to avoid unintentional overflow, the [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size) in the [ratio-dependent axis](#ratio-dependent-axis) of a box with a [preferred aspect ratio](#preferred-aspect-ratio) that is neither a [replaced element](https://www.w3.org/TR/css-display-4/#replaced-element), nor a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) in that axis, is its [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content) capped by its [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-514390d4"></a> In the following example, the box is as wide as the container (as usual), and its height is as tall as needed to contain its content but at least as tall as it is wide.
>
> ```text
> div {
>   aspect-ratio: 1/1;
>   /* 'width' and 'height' both default to 'auto' */
> }
> ```
>
> ```text
> +----------+  +----------+  +----------+
> | ~~~~~~~~ |  | ~~~~~~~~ |  | ~~~~~~~~ |
> | ~~~~~~~~ |  | ~~~~~~~~ |  | ~~~~~~~~ |
> | ~~~~~~~  |  | ~~~~~~~~ |  | ~~~~~~~~ |
> |          |  | ~~~      |  | ~~~~~~~~ |
> +----------+  +----------+  | ~~~~~~~~ |
>                             | ~~~~~~   |
>                             +----------+
> ```
>
> <a id="ref-for-propdef-overflow"></a>
>
> When [overflow: auto](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) is specified, however, even the box with excess content maintains the 1:1 aspect ratio (and handles overflow by becoming scrollable instead, as usual).
>
> ```text
> div {
>   overflow: auto;
>   aspect-ratio: 1/1;
> }
> ```
>
> ```text
> +----------+  +----------+  +----------+
> | ~~~~~~~~ |  | ~~~~~~~~ |  | ~~~~~~~~^|
> | ~~~~~~~~ |  | ~~~~~~~~ |  | ~~~~~~~~ |
> | ~~~~~~~  |  | ~~~~~~~~ |  | ~~~~~~~~ |
> |          |  | ~~~      |  | ~~~~~~~~v|
> +----------+  +----------+  +----------+
> ```
>
> <a id="ref-for-propdef-min-height④"></a>
>
> Overriding the [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) property also maintains the 1:1 aspect ratio, but will result in content overflowing the box if it is not otherwise handled.
>
> ```text
> div {
>   aspect-ratio: 1/1;
>   min-height: 0;
> }
> ```
>
> ```text
> +----------+  +----------+  +----------+
> | ~~~~~~~~ |  | ~~~~~~~~ |  | ~~~~~~~~ |
> | ~~~~~~~~ |  | ~~~~~~~~ |  | ~~~~~~~~ |
> | ~~~~~~~  |  | ~~~~~~~~ |  | ~~~~~~~~ |
> |          |  | ~~~      |  | ~~~~~~~~ |
> +----------+  +----------+  +-~~~~~~~~-+
>                               ~~~~~~    
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-63716969"></a> This automatic minimum operates in both axes. Consider this example:
>
> ```text
> <div style="height: 100px; aspect-ratio: 1/1;">
>   <span style="display: inline-block; width: 50px;"></span>
>   <span style="display: inline-block; width: 150px;"></span>
> </div>
> ```
>
> <a id="ref-for-propdef-width⑤"></a>
>
> <a id="ref-for-valdef-width-auto②"></a>
>
> <a id="ref-for-propdef-min-width④"></a>
>
> The [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) of the container, being [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), resolves through the aspect ratio to 100px. However, its [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width), being <a id="ref-for-valdef-width-auto③"></a>auto, resolves to 150px. The resulting width of the container is thus 150px. To ignore the contents when sizing the container, <a id="ref-for-propdef-min-width⑤"></a>min-width: 0 can be specified.

Tests

- [block-aspect-ratio-038.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-038.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-038.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-038.html)
- [block-aspect-ratio-039.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-039.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-039.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-039.html)
- [fieldset-element-001.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/fieldset-element-001.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/fieldset-element-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/fieldset-element-001.html)
- [fieldset-element-002.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/fieldset-element-002.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/fieldset-element-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/fieldset-element-002.html)
- [flex-aspect-ratio-002.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-002.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-002.html)
- [flex-aspect-ratio-004.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-004.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-004.html)
- [flex-aspect-ratio-025.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-025.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-025.html)
- [flex-aspect-ratio-026.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-026.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-026.html)
- [flex-aspect-ratio-040.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-040.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-040.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-040.html)
- [flex-aspect-ratio-041.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-041.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-041.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-041.html)
- [flex-aspect-ratio-042.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-042.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-042.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-042.html)
- [flex-aspect-ratio-043.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-043.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-043.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-043.html)
- [flex-aspect-ratio-044.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-044.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-044.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-044.html)
- [grid-aspect-ratio-039.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-039.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-039.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-039.html)
- [grid-aspect-ratio-042.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/grid-aspect-ratio-042.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/grid-aspect-ratio-042.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/grid-aspect-ratio-042.html)
- [intrinsic-size-010.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/intrinsic-size-010.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/intrinsic-size-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/intrinsic-size-010.html)

### <a id="aspect-ratio-size-transfers"></a>4.4.  Min/Max Size Transfers

<a id="ref-for-preferred-aspect-ratio①⑤"></a>

<a id="ref-for-indefinite"></a>

<a id="ref-for-min-width"></a>

<a id="ref-for-max-width①"></a>

<a id="ref-for-preferred-size③"></a>

Sizing constraints in either axis (the <var>origin</var> axis) are transferred through the [preferred aspect ratio](#preferred-aspect-ratio) and applied to any [indefinite](https://www.w3.org/TR/css-sizing-3/#indefinite) [minimum](https://www.w3.org/TR/css-sizing-3/#min-width), [maximum](https://www.w3.org/TR/css-sizing-3/#max-width), or [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) size in the other axis (the <var>destination</var> axis) as follows:

- <a id="ref-for-definite②"></a>

  <a id="ref-for-min-width①"></a>

  <a id="ref-for-preferred-size④"></a>

  <a id="ref-for-max-width②"></a>

  First, any [definite](https://www.w3.org/TR/css-sizing-3/#definite) [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width) is converted and transferred from the <var>origin</var> to <var>destination</var> axis. This transferred minimum is capped by any <a id="ref-for-definite③"></a>definite [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) or [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width) in the <var>destination</var> axis.

- <a id="ref-for-definite④"></a>

  <a id="ref-for-max-width③"></a>

  <a id="ref-for-preferred-size⑤"></a>

  <a id="ref-for-min-width②"></a>

  Then, any [definite](https://www.w3.org/TR/css-sizing-3/#definite) [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width) is converted and transferred from the <var>origin</var> to <var>destination</var>. This transferred maximum is floored by any <a id="ref-for-definite⑤"></a>definite [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) or [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width) in the <var>destination</var> axis as well as by the transferred minimum, if any.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Thus, any definite sizes are completely unaffected by a transferred constraint; and a transferred minimum will never cause an element to exceed a definite preferred/maximum size, nor will a transferred maximum cause an element to violate its preferred/minimum size.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The basic principle is that sizing constraints transfer through the aspect-ratio to the other side to preserve the aspect ratio to the extent that they can without violating any sizes specified explicitly on that affected axis. (This is the principle that drove the contents of the [constraint table in CSS2 Section 10.4](https://www.w3.org/TR/CSS2/visudet.html#min-max-widths).)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b98c11d1"></a> In the following example:
>
> ```text
> <div id=container style="height: 100px; float: left;">
>   <div id=item style="height: 100%; aspect-ratio: 1/1;">content</div>
> </div>
> ```
>
> Since the height of the `#item` is a percentage that resolves against a definite container, the width of the item resolves to 100px for both its intrinsic size contributions as well as for final layout, so the container also sizes to a width of 100px.
>
> ```text
> <div id=container style="height: auto; float: left;">
>   <div id=item style="height: 100%; aspect-ratio: 1/1;">content</div>
> </div>
> ```
>
> <a id="ref-for-behave-as-auto"></a>
>
> <a id="ref-for-automatic-size③"></a>
>
> <a id="ref-for-ratio-dependent-axis③"></a>
>
> <a id="ref-for-intrinsic-size-contribution"></a>
>
> In this next example, the percentage height of the item cannot be resolved and [behaves as auto](https://www.w3.org/TR/css-sizing-3/#behave-as-auto) (see [CSS 2 § 10.5 Content height: the 'height' property](https://www.w3.org/TR/CSS2/visudet.html#the-height-property)). Since both axes now have an [automatic size](https://www.w3.org/TR/css-sizing-3/#automatic-size), the height becomes the [ratio-dependent axis](#ratio-dependent-axis). Calculating the [intrinsic size contributions](https://www.w3.org/TR/css-sizing-3/#intrinsic-size-contribution) of the box produces a width derived from its content, and a height calculated from that width and the aspect ratio, yielding a square box (and a container) sized to the width of the word “content”.

Tests

- [replaced-element-032.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-032.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-032.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-032.html)
- [replaced-element-033.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-033.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-033.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-033.html)
- [block-aspect-ratio-040.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-040.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-040.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-040.html)
- [block-aspect-ratio-041.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-041.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-041.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-041.html)
- [block-aspect-ratio-042.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-042.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-042.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-042.html)
- [block-aspect-ratio-043.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-043.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-043.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-043.html)
- [block-aspect-ratio-044.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-044.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-044.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-044.html)
- [block-aspect-ratio-045.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-045.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-045.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-045.html)
- [block-aspect-ratio-046.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-046.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-046.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-046.html)
- [block-aspect-ratio-047.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-047.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-047.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-047.html)
- [block-aspect-ratio-048.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-048.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-048.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-048.html)
- [block-aspect-ratio-049.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/block-aspect-ratio-049.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/block-aspect-ratio-049.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/block-aspect-ratio-049.html)
- [flex-aspect-ratio-039.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/flex-aspect-ratio-039.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/flex-aspect-ratio-039.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/flex-aspect-ratio-039.html)
- [replaced-element-039.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-039.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-039.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-039.html)
- [replaced-element-040.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-040.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-040.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-040.html)
- [replaced-element-041.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-041.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-041.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-041.html)
- [replaced-element-043.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-043.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-043.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-043.html)
- [replaced-element-044.html](https://wpt.fyi/results/css/css-sizing/aspect-ratio/replaced-element-044.html) [(live test)](http://wpt.live/css/css-sizing/aspect-ratio/replaced-element-044.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/aspect-ratio/replaced-element-044.html)
- [image-max-width-and-height-behaves-as-auto.html](https://wpt.fyi/results/css/css-sizing/image-max-width-and-height-behaves-as-auto.html) [(live test)](http://wpt.live/css/css-sizing/image-max-width-and-height-behaves-as-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/image-max-width-and-height-behaves-as-auto.html)
- [min-max-content-orthogonal-flow-crash-001.html](https://wpt.fyi/results/css/css-sizing/min-max-content-orthogonal-flow-crash-001.html) [(live test)](http://wpt.live/css/css-sizing/min-max-content-orthogonal-flow-crash-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/min-max-content-orthogonal-flow-crash-001.html)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-0273742f"></a> This section might not be written correctly. [\[Issue \#6071\]](https://github.com/w3c/csswg-drafts/issues/6071)

## <a id="intrinsic"></a>5.  Intrinsic Size Determination

### <a id="intrinsic-sizes"></a>5.1.  Intrinsic Sizes

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9④"></a> [CSS Sizing 3 § 5.1 Intrinsic Sizes](https://www.w3.org/TR/css-sizing-3/#intrinsic-sizes)

### <a id="intrinsic-size-override"></a>5.2.  Overriding Contained Intrinsic Sizes: the contain-intrinsic-\* properties

| Field               | Definition                                                                                                                                                                                                                                          |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-contain-intrinsic-width"></a>contain-intrinsic-width, <a id="propdef-contain-intrinsic-height"></a>contain-intrinsic-height, <a id="propdef-contain-intrinsic-block-size"></a>contain-intrinsic-block-size, <a id="propdef-contain-intrinsic-inline-size"></a>contain-intrinsic-inline-size                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-length-value"></a><a id="ref-for-comb-one⑧"></a><a id="ref-for-mult-opt③"></a>auto[?](https://www.w3.org/TR/css-values-4/#mult-opt) \[ none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value) \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-size-containment"></a>elements with [size containment](https://www.w3.org/TR/css-contain-2/#size-containment)                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-length-value①"></a>as specified, with [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) values computed                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                              |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-contain-intrinsic-size"></a>[contain-intrinsic-size](#propdef-contain-intrinsic-size)                                                                                                                                                                        |

Tests

- [auto-014.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-014.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-014.html)
- [auto-015.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-015.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-015.html)
- [auto-016.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-016.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-016.html)
- [auto-017.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-017.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-017.html)
- [auto-018.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-018.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-018.html)
- [contain-intrinsic-size-001.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-001.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-001.html)
- [contain-intrinsic-size-002.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-002.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-002.html)
- [contain-intrinsic-size-003.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-003.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-003.html)
- [contain-intrinsic-size-004.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-004.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-004.html)
- [contain-intrinsic-size-005.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-005.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-005.html)
- [contain-intrinsic-size-006.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-006.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-006.html)
- [contain-intrinsic-size-007.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-007.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-007.html)
- [contain-intrinsic-size-008.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-008.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-008.html)
- [contain-intrinsic-size-009.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-009.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-009.html)
- [contain-intrinsic-size-010.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-010.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-010.html)
- [contain-intrinsic-size-011.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-011.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-011.html)
- [contain-intrinsic-size-012.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-012.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-012.html)
- [contain-intrinsic-size-013.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-013.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-013.html)
- [contain-intrinsic-size-014.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-014.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-014.html)
- [contain-intrinsic-size-015.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-015.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-015.html)
- [contain-intrinsic-size-016.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-016.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-016.html)
- [contain-intrinsic-size-017.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-017.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-017.html)
- [contain-intrinsic-size-018.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-018.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-018.html)
- [contain-intrinsic-size-019.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-019.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-019.html)
- [contain-intrinsic-size-020.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-020.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-020.html)
- [contain-intrinsic-size-021.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-021.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-021.html)
- [contain-intrinsic-size-022.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-022.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-022.html)
- [contain-intrinsic-size-023.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-023.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-023.html)
- [contain-intrinsic-size-024.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-024.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-024.html)
- [contain-intrinsic-size-025.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-025.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-025.html)
- [contain-intrinsic-size-026.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-026.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-026.html)
- [contain-intrinsic-size-027.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-027.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-027.html)
- [contain-intrinsic-size-028.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-028.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-028.html)
- [contain-intrinsic-size-029.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-029.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-029.html)
- [contain-intrinsic-size-030.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-030.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-030.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-030.html)
- [contain-intrinsic-size-031.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-031.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-031.html)
- [contain-intrinsic-size-032.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-032.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-032.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-032.html)
- [contain-intrinsic-size-logical-001.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-logical-001.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-logical-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-logical-001.html)
- [contain-intrinsic-size-logical-002.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-logical-002.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-logical-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-logical-002.html)
- [contain-intrinsic-size-logical-003.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-logical-003.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-logical-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-logical-003.html)
- [forget-on-disconnect-in-iframe.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/forget-on-disconnect-in-iframe.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/forget-on-disconnect-in-iframe.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/forget-on-disconnect-in-iframe.html)
- [contain-intrinsic-size-computed.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/parsing/contain-intrinsic-size-computed.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/parsing/contain-intrinsic-size-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/parsing/contain-intrinsic-size-computed.html)
- [contain-intrinsic-size-invalid.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/parsing/contain-intrinsic-size-invalid.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/parsing/contain-intrinsic-size-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/parsing/contain-intrinsic-size-invalid.html)
- [contain-intrinsic-size-valid.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/parsing/contain-intrinsic-size-valid.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/parsing/contain-intrinsic-size-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/parsing/contain-intrinsic-size-valid.html)

<a id="ref-for-size-containment①"></a>

<a id="ref-for-in-flow"></a>

<a id="ref-for-explicit-intrinsic-inner-size"></a>

These properties allow elements with [size containment](https://www.w3.org/TR/css-contain-2/#size-containment) to specify an <a id="explicit-intrinsic-inner-size"></a>explicit intrinsic inner size, causing the box to size as if its [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) content totals to a width and height matching the specified [explicit intrinsic inner size](#explicit-intrinsic-inner-size) (rather than sizing as if it were empty).

<a id="ref-for-explicit-intrinsic-inner-size①"></a>

<a id="ref-for-grid-container"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is not always equivalent to laying out as if the element had one child of the specified [explicit intrinsic inner size](#explicit-intrinsic-inner-size). For example, a [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) with one child of the specified size would still size according to the specified grid, usually ending up with a larger content size than specified.

Tests

- [contain-intrinsic-size-033.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-033.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-033.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/contain-intrinsic-size-033.html)

<a id="ref-for-length-value②"></a>

<a id="valdef-contain-intrinsic-width-none"></a>none \| <a id="valdef-contain-intrinsic-width-length"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

<a id="ref-for-propdef-contain-intrinsic-size①"></a>

<a id="ref-for-valdef-contain-intrinsic-width-auto"></a>

<a id="ref-for-explicit-intrinsic-inner-size②"></a>

<a id="ref-for-valdef-contain-intrinsic-width-none"></a>

<a id="ref-for-length-value③"></a>

If no other [contain-intrinsic-size](#propdef-contain-intrinsic-size) value (such as [auto](#valdef-contain-intrinsic-width-auto)) is providing an [explicit intrinsic inner size](#explicit-intrinsic-inner-size), the corresponding axis either doesn’t have an <a id="ref-for-explicit-intrinsic-inner-size③"></a>explicit intrinsic inner size (if [none](#valdef-contain-intrinsic-width-none) is specified) or has an <a id="ref-for-explicit-intrinsic-inner-size④"></a>explicit intrinsic inner size of the specified [\<length\>](https://www.w3.org/TR/css-values-4/#length-value).

<a id="valdef-contain-intrinsic-width-auto"></a>auto

<a id="ref-for-valdef-contain-intrinsic-width-auto①"></a>

<a id="ref-for-last-remembered"></a>

<a id="ref-for-skips-its-contents"></a>

<a id="ref-for-explicit-intrinsic-inner-size⑤"></a>

If [auto](#valdef-contain-intrinsic-width-auto) is specified and the element has a [last remembered size](#last-remembered) and is currently [skipping its contents](https://www.w3.org/TR/css-contain-2/#skips-its-contents), its [explicit intrinsic inner size](#explicit-intrinsic-inner-size) in the corresponding axis is the <a id="ref-for-last-remembered①"></a>last remembered size in that axis.

<a id="ref-for-propdef-content-visibility"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This occurs, for example, when an element with [content-visibility: auto](https://www.w3.org/TR/css-contain-2/#propdef-content-visibility) is off-screen.

Tests

- [content-visibility-058.html](https://wpt.fyi/results/css/css-contain/content-visibility/content-visibility-058.html) [(live test)](http://wpt.live/css/css-contain/content-visibility/content-visibility-058.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-contain/content-visibility/content-visibility-058.html)

<a id="ref-for-explicit-intrinsic-inner-size⑥"></a>

<a id="ref-for-size-containment②"></a>

If an element has an [explicit intrinsic inner size](#explicit-intrinsic-inner-size) in an axis, then after laying out the element as normal for [size containment](https://www.w3.org/TR/css-contain-2/#size-containment), the size of the contents in that axis are instead treated as being the <a id="ref-for-explicit-intrinsic-inner-size⑦"></a>explicit intrinsic inner size instead of what was calculated in layout, and layout is performed again if necessary. (If it has an <a id="ref-for-explicit-intrinsic-inner-size⑧"></a>explicit intrinsic inner size in both axises, this implies the first layout can be skipped.)

<a id="ref-for-logical-property-group"></a>

These four properties are part of a [logical property group](https://www.w3.org/TR/css-logical-1/#logical-property-group).

<a id="ref-for-size-containment③"></a>

<a id="ref-for-propdef-height⑤"></a>

<a id="ref-for-explicit-intrinsic-inner-size⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: An element with [size containment](https://www.w3.org/TR/css-contain-2/#size-containment) is laid out as if it had no contents [\[CSS-CONTAIN-1\]](#biblio-css-contain-1), which in many cases this will cause the element to collapse to zero inner height. This can be corrected with an explicit [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) chosen to show the expected contents, but that can have unintended effects in some layout systems, such as Flex and Grid Layout, which treat an explicit <a id="ref-for-propdef-height⑥"></a>height as a stronger command than an implicit content-based height. The element thus might lay out substantially differently than it would have were it simply filled with content up to that height. Providing an [explicit intrinsic inner size](#explicit-intrinsic-inner-size) for the element preserves the performance benefits of ignoring its contents for layout while still allowing it to size as if it had content.

| Field               | Definition                                                                                                                                                                                                                                                                                                                              |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-contain-intrinsic-size"></a>contain-intrinsic-size                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range"></a><a id="ref-for-length-value④"></a><a id="ref-for-comb-one⑨"></a><a id="ref-for-mult-opt④"></a>\[ auto[?](https://www.w3.org/TR/css-values-4/#mult-opt) \[ none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value) \] \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                             |

Tests

- [contain-intrinsic-size-interpolation.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/animation/contain-intrinsic-size-interpolation.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/animation/contain-intrinsic-size-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/animation/contain-intrinsic-size-interpolation.html)

<a id="ref-for-propdef-contain-intrinsic-size②"></a>

<a id="ref-for-propdef-contain-intrinsic-width"></a>

<a id="ref-for-propdef-contain-intrinsic-height"></a>

[contain-intrinsic-size](#propdef-contain-intrinsic-size) is a shorthand property that sets the [contain-intrinsic-width](#propdef-contain-intrinsic-width) and [contain-intrinsic-height](#propdef-contain-intrinsic-height) properties.

<a id="ref-for-propdef-contain-intrinsic-width①"></a>

<a id="ref-for-propdef-contain-intrinsic-height①"></a>

The first value represents the [contain-intrinsic-width](#propdef-contain-intrinsic-width) value, and the second represents the [contain-intrinsic-height](#propdef-contain-intrinsic-height) value. If only one value is given, it applies to both properties.

#### <a id="last-remembered"></a>5.2.1.  Last Remembered Size

<a id="ref-for-size-containment④"></a>

[Size containment](https://www.w3.org/TR/css-contain-2/#size-containment) is very valuable for ensuring a page can render efficiently, restricting the scope of layout work that can happen as a result of an element changing its rendering. However, it’s also very restrictive for the author, requiring them to correctly predict what the size of the element will be; if this guess is incorrect, even slightly, it can cause unsightly scrollbars or accidentally-hidden content.

<a id="ref-for-valdef-contain-intrinsic-width-auto②"></a>

<a id="ref-for-size-containment⑤"></a>

The [auto](#valdef-contain-intrinsic-width-auto) keyword of contain-intrinsic-size allows a middle-ground: if an element is ever <em>not</em> [size-contained](https://www.w3.org/TR/css-contain-2/#size-containment), this value causes the element to remember its size (calculated as normal by layout); then, if the element gains <a id="ref-for-size-containment⑥"></a>size containment later, it will use the remembered size, offering the performance benefits of <a id="ref-for-size-containment⑦"></a>size containment while <em>probably</em> sizing accurately to its contents.

<a id="ref-for-resizeobserver"></a>

<a id="ref-for-last-remembered②"></a>

Only elements capable of being <code><a href="https://www.w3.org/TR/resize-observer-1/#resizeobserver">ResizeObserver</a></code> targets can have a [last remembered size](#last-remembered).

<a id="ref-for-last-remembered③"></a>

The [last remembered size](#last-remembered) of an element is determined by:

- <a id="ref-for-resizeobserver①"></a>

  <a id="ref-for-valdef-contain-intrinsic-width-auto③"></a>

  <a id="ref-for-resizeobserver②"></a>

  <a id="ref-for-size-containment⑧"></a>

  <a id="ref-for-principal-box"></a>

  <a id="ref-for-last-remembered④"></a>

  At the time that <code><a href="https://www.w3.org/TR/resize-observer-1/#resizeobserver">ResizeObserver</a></code> events are determined and delivered, if an element has a [auto](#valdef-contain-intrinsic-width-auto) keyword in contain-intrinsic-size property, is capable of being a <code><a href="https://www.w3.org/TR/resize-observer-1/#resizeobserver">ResizeObserver</a></code> target, but does not have [size containment](https://www.w3.org/TR/css-contain-2/#size-containment), record the current inner dimensions of its [principal box](https://www.w3.org/TR/css-display-4/#principal-box) as its [last remembered size](#last-remembered).

- <a id="ref-for-resizeobserver③"></a>

  <a id="ref-for-last-remembered⑤"></a>

  <a id="ref-for-valdef-contain-intrinsic-width-auto④"></a>

  At the time that <code><a href="https://www.w3.org/TR/resize-observer-1/#resizeobserver">ResizeObserver</a></code> events are determined and delivered, if an element has a [last remembered size](#last-remembered) but does <em>not</em> have [auto](#valdef-contain-intrinsic-width-auto) keyword in contain-intrinsic-size property, remove its <a id="ref-for-last-remembered⑥"></a>last remembered size.

<a id="ref-for-last-remembered⑦"></a>

<a id="ref-for-valdef-contain-intrinsic-width-auto⑤"></a>

<a id="ref-for-propdef-display"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [last remembered size](#last-remembered) is state attached to the <em>element</em>, not any particular box generated by the element. So long as the element retains [auto](#valdef-contain-intrinsic-width-auto) keyword in contain-intrinsic-size property, it will remember its <a id="ref-for-last-remembered⑧"></a>last remembered size even across changes such as going to/from [display: none](https://www.w3.org/TR/css-display-4/#propdef-display).

Tests

- [auto-001.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-001.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-001.html)
- [auto-002.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-002.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-002.html)
- [auto-003.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-003.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-003.html)
- [auto-004.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-004.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-004.html)
- [auto-005.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-005.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-005.html)
- [auto-006.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-006.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-006.html)
- [auto-007.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-007.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-007.html)
- [auto-008.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-008.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-008.html)
- [auto-009.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-009.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-009.html)
- [auto-010.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-010.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-010.html)
- [auto-011.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-011.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-011.html)
- [auto-012.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-012.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-012.html)
- [auto-013.html](https://wpt.fyi/results/css/css-sizing/contain-intrinsic-size/auto-013.html) [(live test)](http://wpt.live/css/css-sizing/contain-intrinsic-size/auto-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-sizing/contain-intrinsic-size/auto-013.html)

<a id="ref-for-propdef-overflow①"></a>

#### <a id="cis-scrollbars"></a>5.2.2.  Interaction With [overflow: auto](https://www.w3.org/TR/css-overflow-3/#propdef-overflow)

<a id="ref-for-propdef-contain-intrinsic-size③"></a>

<a id="ref-for-propdef-overflow②"></a>

The [contain-intrinsic-size](#propdef-contain-intrinsic-size) property provides an estimate of how large the author expects the content of an element to be, but this estimate is not actual content and does not represent anything that needs to be shown to the user. Therefore, an element with [overflow: auto](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) must not generate scrollbars as a consequence of <a id="ref-for-propdef-contain-intrinsic-size④"></a>contain-intrinsic-size.

<a id="ref-for-propdef-contain-intrinsic-size⑤"></a>

However, if [contain-intrinsic-size](#propdef-contain-intrinsic-size) indicates a size large enough that the element would generate scrollbars if it contained actual content of that size, then the element must be <em>sized</em> as if it generated those scrollbar(s) in accordance with such hypothetical content.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b57541eb"></a> In the following example code:
>
> ```text
> div {
>   width: max-content;
>   contain-intrinsic-size: 100px 100px;
>   overflow: auto;
> }
> ```
>
> <a id="ref-for-propdef-contain-intrinsic-size⑥"></a>
>
> The element ends up being 100px wide and 100px tall: [contain-intrinsic-size](#propdef-contain-intrinsic-size) provides the max-content width, and also the height.
>
> If the element then ended up with content that was 150px tall, it would show a vertical scrollbar; if the scrollbar is not overlay, it will take up some of that 100px width, leaving a smaller amount (roughly 84px, typically) for the content to flow into. (See [CSS Overflow 3 § 4 Scrollbars and Layout](https://www.w3.org/TR/css-overflow-3/#scrollbar-layout).)
>
> <a id="ref-for-propdef-contain-intrinsic-size⑦"></a>
>
> Even though there’s now less than 100px of horizontal space available for the content, it will not generate a horizontal scrollbar just because [contain-intrinsic-size](#propdef-contain-intrinsic-size) indicates a 100px width; that would only happen if the actual content had something unbreakable and wider than the remaining space.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-577a9f87"></a> In contrast, in the following example code:
>
> ```text
> div {
>   width: max-content;
>   contain-intrinsic-size: 100px 100px;
>   height: 50px;
>   overflow: auto;
> }
> ```
>
> <a id="ref-for-propdef-contain-intrinsic-size⑧"></a>
>
> The element has a fixed 50px height, but [contain-intrinsic-size](#propdef-contain-intrinsic-size) indicates a 100px “estimated content height”. The element thus assumes that it will need a vertical scrollbar when it’s filled with actual content, resulting in a max-content width a little more than 100px (roughly 116px, typically), to accommodate the estimated 100px of max-content width from <a id="ref-for-propdef-contain-intrinsic-size⑨"></a>contain-intrinsic-size, and as well as the vertical scrollbar width (roughly 16px, typically).
>
> However, even though the element reserves space on the assumption of needing a scrollbar, it will not actually generate one unless the actual content overflows: if it ends up containing content that’s less than 50px tall, no vertical scrollbar will be generated at all, but the element will still be 116px wide.

<a id="ref-for-propdef-frame-sizing"></a>

### <a id="responsive-iframes"></a>5.3.  Responsively-sized iframes: the [frame-sizing](#propdef-frame-sizing) property

| Field               | Definition                                                                                                                                                                                                       |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-frame-sizing"></a>frame-sizing                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⓪"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) content-width <a id="ref-for-comb-one①①"></a>\| content-height <a id="ref-for-comb-one①②"></a>\| content-block-size <a id="ref-for-comb-one①③"></a>\| content-inline-size |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | replaced elements (but see below for details)                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                         |

<a id="ref-for-replaced-element⑦"></a>

<a id="ref-for-the-iframe-element①"></a>

<a id="ref-for-propdef-frame-sizing①"></a>

Some [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element) can contain "normal" flowed content, such as HTML <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code>s. For privacy and security reasons, these elements do not, by default, expose any information about their internal contents' sizing to the outside page, instead just using a static, predetermined intrinsic size, and making their contents scrollable. The [frame-sizing](#propdef-frame-sizing) property allows these elements to opt into exposing their actual content size, known as their <a id="internal-layout-intrinsic-size"></a>internal layout intrinsic size. Values have the following meaning:

<a id="valdef-frame-sizing-auto"></a>auto  
<a id="ref-for-internal-layout-intrinsic-size"></a>

The element’s [internal layout intrinsic size](#internal-layout-intrinsic-size), if any, is ignored.

<a id="valdef-frame-sizing-content-width"></a>content-width  
<a id="valdef-frame-sizing-content-height"></a>content-height  
<a id="valdef-frame-sizing-content-block-size"></a>content-block-size  
<a id="valdef-frame-sizing-content-inline-size"></a>content-inline-size  
<a id="ref-for-internal-layout-intrinsic-size①"></a>

<a id="ref-for-intrinsic-size"></a>

If the element has an [internal layout intrinsic size](#internal-layout-intrinsic-size), its [intrinsic size](https://www.w3.org/TR/css-sizing-3/#intrinsic-size) takes the corresponding dimension (either width or height) from the <a id="ref-for-internal-layout-intrinsic-size②"></a>internal layout intrinsic size. (The other dimension is determined normally.)

<a id="ref-for-writing-mode"></a>

Logical directions resolve based on the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the element. (Not the embedded document.)

<a id="ref-for-internal-layout-intrinsic-size③"></a>

<a id="ref-for-doclanguage"></a>

<a id="ref-for-the-iframe-element②"></a>

Which elements can have an [internal layout intrinsic size](#internal-layout-intrinsic-size), and how it’s determined, are decided by the [document language](https://www.w3.org/TR/CSS2/conform.html#doclanguage). In HTML, only <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> elements can have an <a id="ref-for-internal-layout-intrinsic-size④"></a>internal layout intrinsic size, and further, only when the contained document has also opted in via a `<meta name=responsive-embedded-sizing>` element. (See [§ 5.3.1 HTML iframe Details](#iframe-frame-sizing).)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8eced7a0"></a> When the embedded document has the following HTML:
>
> ```text
> <meta name="responsive-embedded-sizing">
> <div style="height: 500px"></div>
> ```
>
> and the embedding document has the following CSS:
>
> ```text
> iframe {
>   frame-sizing: content-height;
> }
> ```
>
> <a id="ref-for-the-iframe-element③"></a>
>
> The height of the <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> will initially be the default iframe height (typically 150px), but will be updated to 500px once the iframe’s document loads.

<a id="ref-for-dom-window-requestresize"></a>

<a id="ref-for-internal-layout-intrinsic-size⑤"></a>

In addition, the internal document can call <code><a href="#dom-window-requestresize">window.requestResize()</a></code> to update its [internal layout intrinsic size](#internal-layout-intrinsic-size) later.

<a id="ref-for-the-iframe-element④"></a>

#### <a id="iframe-frame-sizing"></a>5.3.1.  HTML <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> Details

<a id="ref-for-the-iframe-element⑤"></a>

<a id="ref-for-internal-layout-intrinsic-size⑥"></a>

<a id="ref-for-the-iframe-element⑥"></a>

In HTML, only the <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> element can have an [internal layout intrinsic size](#internal-layout-intrinsic-size), and only when the <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code>’s embedded document has opted in appropriately.

<a id="ref-for-document"></a>

An HTML <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code> object has an <a id="document-responsive-embedded-sizing-flag"></a>responsive embedded sizing flag associated with it, which is initially unset. It is set to true or false during the initial document parse, depending on which of the following first occurs:

- If a `<meta name=responsive-embedded-sizing>` element is encountered, it is set to true.

- <a id="ref-for-the-body-element"></a>

  If the <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> element is opened (explicitly or implicitly), or inserted into the document by the parser, it is set to false.

<a id="ref-for-document①"></a>

Once set to either true or false, the flag will not change its value again for the lifetime of the <code><a href="https://dom.spec.whatwg.org/#document">Document</a></code>.

<a id="ref-for-meta"></a>

<a id="ref-for-the-head-element"></a>

<a id="ref-for-the-iframe-element⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Currently, the HTML <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#meta">meta</a></code> element appearing in the <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-head-element">head</a></code> of an HTML document is the only way for an embedded document to opt into this feature in HTML. This means that SVG documents in <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> cannot do so.

<a id="ref-for-meta①"></a>

<a id="ref-for-the-head-element①"></a>

<a id="ref-for-the-body-element①"></a>

<a id="ref-for-the-head-element②"></a>

<a id="ref-for-the-body-element②"></a>

<a id="ref-for-the-body-element③"></a>

<a id="ref-for-the-body-element④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Due to quirks of the HTML parser, a <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#meta">meta</a></code> element can technically appear <em>between</em> the <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-head-element">head</a></code> and <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> elements, and will be reparented into the <code><a href="https://html.spec.whatwg.org/multipage/semantics.html#the-head-element">head</a></code>. This is still a valid location for the `<meta name=responsive-embedded-sizing>` element, as the <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> has not yet been opened. Note that most HTML elements will implicitly open a <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> element even if the <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> start tag is not present, and if the document completely lacks such an element, an empty one is automatically generated and inserted.

<a id="ref-for-internal-layout-intrinsic-size⑦"></a>

<a id="ref-for-event-domcontentloaded"></a>

<a id="ref-for-event-load"></a>

<a id="ref-for-window"></a>

<a id="ref-for-dom-window-requestresize①"></a>

The [internal layout intrinsic size](#internal-layout-intrinsic-size) is first set by the results of the embedded document’s first layout after it fires its <code><a href="https://html.spec.whatwg.org/multipage/indices.html#event-domcontentloaded">DOMContentLoaded</a></code> event, and again when the <code><a href="https://html.spec.whatwg.org/multipage/indices.html#event-load">load</a></code> event is fired at the <code><a href="https://html.spec.whatwg.org/multipage/nav-history-apis.html#window">Window</a></code>. Subsequent changes to content, styling or layout of the embedded document do not automatically affect the <a id="ref-for-internal-layout-intrinsic-size⑧"></a>internal layout intrinsic size, but see <code><a href="#dom-window-requestresize">requestResize()</a></code>.

<a id="ref-for-document-responsive-embedded-sizing-flag"></a>

<a id="ref-for-locked-embedded-icb-size"></a>

<a id="ref-for-initial-containing-block"></a>

An embedded document can additionally have a <a id="locked-embedded-icb-size"></a>locked embedded ICB size, which is initially null. If the document’s [responsive embedded sizing flag](#document-responsive-embedded-sizing-flag) is true when it performs a layout, and its [locked embedded ICB size](#locked-embedded-icb-size) is null, it records its current [initial containing block](https://www.w3.org/TR/css-display-4/#initial-containing-block) size as its <a id="ref-for-locked-embedded-icb-size①"></a>locked embedded ICB size. On subsequent layouts, it uses its <a id="ref-for-locked-embedded-icb-size②"></a>locked embedded ICB size instead of calculating its <a id="ref-for-initial-containing-block①"></a>initial containing block size normally.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This freezing of the ICB reduces the chance of layout loops (an iframe document sizing to slightly larger than its ICB, then the parent iframe changing size to match, and the next layout again making it slightly larger than the ICB, etc to infinity).

<a id="ref-for-locked-embedded-icb-size③"></a>

<a id="ref-for-propdef-frame-sizing②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-6b9851d9"></a> Do we want to have a way to force an iframe to forget its [locked embedded ICB size](#locked-embedded-icb-size)? Maybe turning [frame-sizing](#propdef-frame-sizing) off and on again?

<a id="ref-for-locked-embedded-icb-size④"></a>

<a id="ref-for-internal-layout-intrinsic-size⑨"></a>

<a id="ref-for-document-responsive-embedded-sizing-flag①"></a>

Navigating the iframe’s document causes it to forget its [locked embedded ICB size](#locked-embedded-icb-size). It <strong>does not</strong> forget its [internal layout intrinsic size](#internal-layout-intrinsic-size) as long as the new document’s [responsive embedded sizing flag](#document-responsive-embedded-sizing-flag) is unset. Once the flag is set (to true <em>or</em> false), the <a id="ref-for-internal-layout-intrinsic-size①⓪"></a>internal layout intrinsic size is forgotten; if the flag is set to true, both are then freshly computed as normal.

<a id="ref-for-window①"></a>

#### <a id="window-requestresize"></a>5.3.2.  Extensions to the <code><a href="https://html.spec.whatwg.org/multipage/nav-history-apis.html#window">Window</a></code> Interface

<a id="ref-for-window②"></a>

<a id="ref-for-idl-undefined"></a>

<a id="ref-for-dom-window-requestresize②"></a>

```text
partial interface Window {
  undefined requestResize();
};
```
When <a id="dom-window-requestresize"></a>`requestResize()` is invoked:

1.  <a id="ref-for-navigable"></a>

    <a id="ref-for-this"></a>

    <a id="ref-for-content-window"></a>

    <a id="ref-for-child-navigable"></a>

    <a id="ref-for-dfn-throw"></a>

    <a id="ref-for-notallowederror"></a>

    <a id="ref-for-idl-DOMException"></a>

    Let <var>navigable</var> be the [navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable) that has [this](https://webidl.spec.whatwg.org/#this) as its [content window](https://html.spec.whatwg.org/multipage/document-sequences.html#content-window). If <var>navigable</var> is not a [child navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#child-navigable), [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#notallowederror">NotAllowedError</a></code> <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code>.

2.  <a id="ref-for-navigable-container"></a>

    <a id="ref-for-content-navigable"></a>

    <a id="ref-for-the-iframe-element⑧"></a>

    <a id="ref-for-dfn-throw①"></a>

    <a id="ref-for-notallowederror①"></a>

    <a id="ref-for-idl-DOMException①"></a>

    Let <var>host element</var> be the [navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container) whose [content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable) is <var>navigable</var>. If <var>host element</var> is not an HTML <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> element, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#notallowederror">NotAllowedError</a></code> <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code>.

3.  <a id="ref-for-this①"></a>

    <a id="ref-for-nav-document"></a>

    <a id="ref-for-document-responsive-embedded-sizing-flag②"></a>

    <a id="ref-for-dfn-throw②"></a>

    <a id="ref-for-notallowederror②"></a>

    <a id="ref-for-idl-DOMException②"></a>

    Let <var>document</var> be [this’s](https://webidl.spec.whatwg.org/#this) [active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document). If <var>document</var>’s [responsive embedded sizing flag](#document-responsive-embedded-sizing-flag) is unset or false, [throw](https://webidl.spec.whatwg.org/#dfn-throw) a <code><a href="https://webidl.spec.whatwg.org/#notallowederror">NotAllowedError</a></code> <code><a href="https://webidl.spec.whatwg.org/#idl-DOMException">DOMException</a></code>.

4.  If <var>document</var> has pending style or layout changes, perform them.

5.  <a id="ref-for-internal-layout-intrinsic-size①①"></a>

    <a id="ref-for-scrolling-area"></a>

    <a id="ref-for-this②"></a>

    Set the [internal layout intrinsic size](#internal-layout-intrinsic-size) of <var>host element</var> to the width and height of the [scrolling area](https://www.w3.org/TR/cssom-view-1/#scrolling-area) of [this](https://webidl.spec.whatwg.org/#this).

### <a id="intrinsic-contribution"></a>5.4.  Intrinsic Size Contributions

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9⑤"></a> [CSS Sizing 3 § 5.2 Intrinsic Contributions](https://www.w3.org/TR/css-sizing-3/#intrinsic-contribution)

<a id="ref-for-propdef-min-intrinsic-sizing"></a>

### <a id="intrinsic-contribution-override"></a>5.5.  Zeroing Min-Content Size Contributions: the [min-intrinsic-sizing](#propdef-min-intrinsic-sizing) property

| Field               | Definition                                                                                                                                                                            |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-min-intrinsic-sizing"></a>min-intrinsic-sizing                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any①"></a><a id="ref-for-comb-one①④"></a>legacy [\|](https://www.w3.org/TR/css-values-4/#comb-one) zero-if-scroll [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) zero-if-extrinsic |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | legacy                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-inline-box①"></a>all elements except [inline boxes](https://www.w3.org/TR/css-display-4/#inline-box)                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                              |

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-b731f1ac"></a> This property seriously needs some name bikeshedding.

<a id="ref-for-min-content-contribution"></a>

<a id="ref-for-non-replaced②"></a>

This property defines whether the [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) of a [non-replaced](https://www.w3.org/TR/css-display-4/#non-replaced) box is “compressed” under certain circumstances. Values have the following meanings:

<a id="valdef-min-intrinsic-sizing-legacy"></a>legacy  
<a id="ref-for-min-content-contribution①"></a>

The box’s [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) is handled as normal.

<a id="valdef-min-intrinsic-sizing-zero-if-scroll"></a>zero-if-scroll  
<a id="ref-for-scroll-container①"></a>

<a id="ref-for-min-content-contribution②"></a>

The box’s [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) is “compressed” if it is a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) in that axis.

<a id="valdef-min-intrinsic-sizing-zero-if-extrinsic"></a>zero-if-extrinsic  
<a id="ref-for-max-width④"></a>

<a id="ref-for-preferred-size⑥"></a>

<a id="ref-for-extrinsic-sizing"></a>

<a id="ref-for-min-content-contribution③"></a>

The box’s [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) is “compressed” if has an [extrinsic](https://www.w3.org/TR/css-sizing-3/#extrinsic-sizing) [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) or [maximum](https://www.w3.org/TR/css-sizing-3/#max-width) size.

<a id="ref-for-replaced-element⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is the default behavior of most [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element).

<a id="ref-for-scroll-container②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-814e05c1"></a> The following rule will make all [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container) essentially ignore their contents when passing up their size contributions (unless they specifically requested a content-based size):
>
> ```text
> *, ::before, ::after { min-intrinsic-sizing: zero-if-scroll; }
> ```
>
> <a id="ref-for-scroll-container③"></a>
>
> <a id="ref-for-min-content①"></a>
>
> This prevents the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) from blowing up the size of its ancestors if it contains large items such as a table or long lines of unbreakable text. Meanwhile, it allows boxes that are not scroll containers to continue influencing the [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content) of their ancestors.

<a id="ref-for-valdef-min-intrinsic-sizing-zero-if-scroll"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The behavior of [zero-if-scroll](#valdef-min-intrinsic-sizing-zero-if-scroll) would have been a better default, but due to Web-compat, it cannot be the initial value. :(

<a id="ref-for-min-content-contribution④"></a>

<a id="ref-for-valdef-width-min-content①"></a>

<a id="ref-for-valdef-width-max-content①"></a>

<a id="ref-for-valdef-width-fit-content"></a>

<a id="ref-for-sizing-property①"></a>

The “compressed” [min-content contributions](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) is calculated by pretending the box were empty, except when factoring in sizing constraints imposed by explicit [min-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content), [max-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-max-content), and [fit-content](https://www.w3.org/TR/css-sizing-4/#valdef-width-fit-content) values of the [sizing properties](https://www.w3.org/TR/css-sizing-3/#sizing-property).

<a id="ref-for-valdef-column-width-stretch"></a>

<a id="ref-for-valdef-column-width-min-content"></a>

<a id="ref-for-valdef-column-width-max-content"></a>

<a id="ref-for-valdef-column-width-fit-content"></a>

<a id="ref-for-valdef-column-width-fit-content-length-percentage"></a>

### <a id="column-sizing"></a>5.6.  New Column Sizing Values: the [stretch](#valdef-column-width-stretch), [min-content](#valdef-column-width-min-content), [max-content](#valdef-column-width-max-content), [fit-content](#valdef-column-width-fit-content), and [fit-content()](#valdef-column-width-fit-content-length-percentage) values

| Field               | Definition                                                                                                                                   |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="ref-for-propdef-column-width①"></a>[column-width](https://www.w3.org/TR/css-multicol-2/#propdef-column-width)                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">New values:</a>&#xA;      </strong> | <a id="ref-for-typedef-box-size②"></a>[\<box-size\>](#typedef-box-size)                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage⑦"></a>as specified, with [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values computed |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                       |

<a id="ref-for-propdef-column-width②"></a>

When used as values for [column-width](https://www.w3.org/TR/css-multicol-2/#propdef-column-width), the new keywords specify the optimal column width:

<a id="valdef-column-width-stretch"></a>stretch

<a id="ref-for-stretch-fit-inline-size"></a>

Specifies the optimal column width as the [stretch-fit inline size](https://www.w3.org/TR/css-sizing-3/#stretch-fit-inline-size) of the multi-column container.

<a id="valdef-column-width-min-content"></a>min-content

<a id="ref-for-min-content-inline-size"></a>

Specifies the optimal column width as the [min-content inline size](https://www.w3.org/TR/css-sizing-3/#min-content-inline-size) of the multi-column container’s contents.

<a id="valdef-column-width-max-content"></a>max-content

<a id="ref-for-max-content-inline-size"></a>

Specifies the optimal column width as the [max-content inline size](https://www.w3.org/TR/css-sizing-3/#max-content-inline-size) of the multi-column container’s contents.

<a id="valdef-column-width-fit-content"></a>fit-content

<a id="ref-for-stretch-fit-inline-size①"></a>

<a id="ref-for-min-content-inline-size①"></a>

<a id="ref-for-max-content-inline-size①"></a>

Specifies the optimal column width as <code>min(<a href="https://www.w3.org/TR/css-sizing-3/#max-content-inline-size">max-content inline size</a>, max(<a href="https://www.w3.org/TR/css-sizing-3/#min-content-inline-size">min-content inline size</a>, <a href="https://www.w3.org/TR/css-sizing-3/#stretch-fit-inline-size">stretch-fit inline size</a>))</code>.

<a id="ref-for-typedef-length-percentage⑧"></a>

<a id="valdef-column-width-fit-content-length-percentage"></a>fit-content([\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage))

<a id="ref-for-typedef-length-percentage⑨"></a>

<a id="ref-for-min-content②"></a>

<a id="ref-for-max-content"></a>

Specifies the optimal column width as <code>min(<a href="https://www.w3.org/TR/css-sizing-3/#max-content">max-content size</a>, max(<a href="https://www.w3.org/TR/css-sizing-3/#min-content">min-content size</a>, <a href="https://www.w3.org/TR/css-values-4/#typedef-length-percentage">&lt;length-percentage&gt;</a>))</code>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The column width never varies by column. When the column width is informed by the multi-column container’s contents (as in the keywords above), all of its contents are taken under consideration and the calculated width is shared by all the columns.

## <a id="extrinsic"></a>6.  Extrinsic Size Determination

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9⑥"></a> [CSS Sizing 3 § 4 Extrinsic Size Determination](https://www.w3.org/TR/css-sizing-3/#extrinsic)

### <a id="contain-fit-sizing"></a>6.1.  Contain-fit Sizing: stretching while maintaining an aspect ratio

<a id="ref-for-preferred-aspect-ratio①⑥"></a>

<a id="ref-for-valdef-object-fit-contain"></a>

<a id="ref-for-propdef-object-fit"></a>

<a id="ref-for-propdef-background-size"></a>

Contain-fit sizing essentially applies stretch-fit sizing, but reduces the size of the box in one axis to maintain the box’s [preferred aspect ratio](#preferred-aspect-ratio), similar to the [contain](https://www.w3.org/TR/css-images-4/#valdef-object-fit-contain) keyword of the [object-fit](https://www.w3.org/TR/css-images-4/#propdef-object-fit) and [background-size](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-size) properties.

First, a target rectangle is determined:

1.  <a id="ref-for-stretch-fit-size①"></a>

    The initial target rectangle is the size of the box’s containing block, with any indefinite size assumed as infinity. If both dimensions are indefinite, the initial target rectangle is set to match the outer edges of the box were it [stretch-fit sized](https://www.w3.org/TR/css-sizing-3/#stretch-fit-size).

2.  <a id="ref-for-propdef-min-height⑤"></a>

    <a id="ref-for-propdef-min-width⑥"></a>

    <a id="ref-for-propdef-max-height④"></a>

    <a id="ref-for-propdef-max-width④"></a>

    <a id="ref-for-valdef-max-width-none①"></a>

    Next, if the box has a non-[none](https://www.w3.org/TR/css-sizing-3/#valdef-max-width-none) [max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width) or [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height), the target rectangle is clamped in the affected dimension to less than or equal to the “maximum size” of the box’s margin box, i.e. the size its margin box would be if the box was sized at its max-width/height. (Note that, consistent with normal [box-sizing rules](https://www.w3.org/TR/CSS2/visuren.html), this “maximum size” is floored by the effects of the box’s [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width)/[min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height).)

3.  <a id="ref-for-preferred-aspect-ratio①⑦"></a>

    Last, the target rectangle is reduced in one dimension by the minimum necessary for it to match the box’s [preferred aspect ratio](#preferred-aspect-ratio).

The contain-fit size in each dimension is the size that would result from stretch-fitting into the target rectangle.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-6e598fc0"></a> Copy whatever stretch-fit ends up doing wrt margin collapsing.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9d61aaed"></a> If there is a minimum size in one dimension that would cause overflow of the target rectangle if the aspect ratio were honored, do we honor the aspect ratio or skew the image? If the former, we need a step similar to \#2 that applies the relevant minimums.

### <a id="percentage-sizing"></a>6.2.  Percentage Sizing

…

## <a id="changes"></a> Changes

### <a id="changes-2021-05"></a> Changes since 5 May 2021 Working Draft

Significant changes since the [5 May 2021 Working Draft](https://www.w3.org/TR/2021/WD-css-sizing-4-20210520/) include:

- <a id="ref-for-propdef-size③"></a>

  <a id="ref-for-propdef-min-size①"></a>

  <a id="ref-for-propdef-max-size①"></a>

  <a id="ref-for-shorthand-property③"></a>

  Added the [size](#propdef-size), [min-size](#propdef-min-size), and [max-size](#propdef-max-size) [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property). ([Issue 820](https://github.com/w3c/csswg-drafts/issues/820))

- <a id="ref-for-funcdef-width-calc-size④"></a>

  <a id="ref-for-sizing-property②"></a>

  Added [calc-size()](#funcdef-width-calc-size) to the [sizing properties](https://www.w3.org/TR/css-sizing-3/#sizing-property). ([Issue 6265](https://github.com/w3c/csswg-drafts/issues/6265))

- <a id="ref-for-propdef-frame-sizing③"></a>

  Added [frame-sizing](#propdef-frame-sizing) property definition. ([1771](https://github.com/w3c/csswg-drafts/issues/1771))

- <a id="ref-for-funcdef-width-fit-content②"></a>

  Imported [fit-content()](#funcdef-width-fit-content) from Level 3, graduated fit-content and stretch back down to Level 3. ([Issue 10601](https://github.com/w3c/csswg-drafts/issues/10601))

- <a id="ref-for-typedef-box-size③"></a>

  Introduced [\<box-size\>](#typedef-box-size) grammar production to consolidate sizing values. ([Issue 13478](https://github.com/w3c/csswg-drafts/issues/13478))

- Disallowed re-ordering of values in contain-intrinsic-size. ([Issue 6391](https://github.com/w3c/csswg-drafts/issues/6391)).

- <a id="ref-for-valdef-contain-intrinsic-width-auto⑥"></a>

  Only used the last remembered size for the [auto](#valdef-contain-intrinsic-width-auto) value of the contain-intrinsic-\* properties when the element is skipping its contents to avoid phantom sizes. ([Issue 6308](https://github.com/w3c/csswg-drafts/issues/6308)).

- Clarify which elements get a last remembered size and when. ([Issue 6220](https://github.com/w3c/csswg-drafts/issues/6220))

- <a id="ref-for-propdef-contain-intrinsic-size①⓪"></a>

  Added logical property group for [contain-intrinsic-size](#propdef-contain-intrinsic-size) properties. ([Issue 2822](https://github.com/w3c/csswg-drafts/issues/2822)).

- <a id="ref-for-propdef-contain-intrinsic-size①①"></a>

  Changed syntax of [contain-intrinsic-size](#propdef-contain-intrinsic-size) to allow for more flexible combinations of values. ([Issue 8407](https://github.com/w3c/csswg-drafts/issues/8407)).

- Disallowed negative lengths in contain-intrinsic-\* properties. ([Issue 11945](https://github.com/w3c/csswg-drafts/issues/11945)).

- <a id="ref-for-responsive-iframes①"></a>

  Added [responsively-sized iframes](#responsive-iframes) via 'contain-intrinsic-size/from-element' value. ([Issue 1771](https://github.com/w3c/csswg-drafts/issues/1771)).

- <a id="ref-for-stretch-fit-size②"></a>

  <a id="ref-for-valdef-align-self-stretch"></a>

  Clarified that Flexbox’s [stretch-fit sizing](https://www.w3.org/TR/css-sizing-3/#stretch-fit-size) behavior is slightly different from [stretch](https://www.w3.org/TR/css-align-3/#valdef-align-self-stretch). ([Issue 11784](https://github.com/w3c/csswg-drafts/issues/11784)).

- Added Web Platform Tests coverage.

- Various other minor editorial fixes and improvements.

### <a id="changes-2020-10"></a> Changes since 20 October 2020 Working Draft

Significant changes since the [20 October 2020 Working Draft](https://www.w3.org/TR/2020/WD-css-sizing-4-20201020/) include:

- <a id="ref-for-propdef-min-intrinsic-sizing①"></a>

  Drafted [min-intrinsic-sizing](#propdef-min-intrinsic-sizing) property, to better control the min-content contributions of scroll containers. ([Issue 1865](https://github.com/w3c/csswg-drafts/issues/1865), [Issue 4585](https://github.com/w3c/csswg-drafts/issues/4585))

- <a id="ref-for-propdef-contain-intrinsic-size①②"></a>

  Added longhands to [contain-intrinsic-size](#propdef-contain-intrinsic-size) for controlling each axis independently. ([Issue 5432](https://github.com/w3c/csswg-drafts/issues/5432))

- <a id="ref-for-valdef-contain-intrinsic-width-auto⑦"></a>

  <a id="ref-for-propdef-contain-intrinsic-size①③"></a>

  Drafted [auto](#valdef-contain-intrinsic-width-auto) value to [contain-intrinsic-size](#propdef-contain-intrinsic-size) to allow “remembering” the previously-calculated size. ([Issue 5668](https://github.com/w3c/csswg-drafts/issues/5668), [Issue 5815](https://github.com/w3c/csswg-drafts/issues/5815))

- <a id="ref-for-propdef-aspect-ratio③"></a>

  Defined handling of degenerate ratios in [aspect-ratio](#propdef-aspect-ratio). ([Issue 5557](https://github.com/w3c/csswg-drafts/issues/5557))

- <a id="ref-for-propdef-aspect-ratio④"></a>

  Defined how [aspect-ratio](#propdef-aspect-ratio) impacts a replaced element’s natural sizes. ([Issue 5306](https://github.com/w3c/csswg-drafts/issues/5306))

- Fixed some errors in the [§ 4.4 Min/Max Size Transfers](#aspect-ratio-size-transfers) section, aligning the behavior to not conflict with behavior defined by CSS2 / CSS Flex Layout / etc. ([Issue 6071](https://github.com/w3c/csswg-drafts/issues/6071))

- <a id="ref-for-propdef-contain-intrinsic-size①④"></a>

  Added [contain-intrinsic-size: auto none](#propdef-contain-intrinsic-size) syntax.

Significant changes since the [26 May 2020 First Public Working Draft](https://www.w3.org/TR/2020/WD-css-sizing-4-20200526/) include:

- <a id="ref-for-ratio-determining-axis"></a>

  Define [ratio-determining axis](#ratio-determining-axis) as a term.

- Define that min/max sizing constraints are transferred across an aspect-ratio, ([Issue 5257](https://github.com/w3c/csswg-drafts/issues/5257))

  > <a id="ref-for-preferred-aspect-ratio①⑧"></a>
  >
  > Additionally, sizing constraints in either axis (the <var>origin</var> axis) are transferred through the [preferred aspect ratio](#preferred-aspect-ratio) to the other axis (the <var>destination</var> axis) as follows:
  >
  > - <a id="ref-for-definite⑥"></a>
  >
  >   <a id="ref-for-min-width③"></a>
  >
  >   <a id="ref-for-preferred-size⑦"></a>
  >
  >   <a id="ref-for-max-width⑤"></a>
  >
  >   First, any [definite](https://www.w3.org/TR/css-sizing-3/#definite) [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width) is converted and transferred from the <var>origin</var> to <var>destination</var> axis. This transferred minimum is capped by any <a id="ref-for-definite⑦"></a>definite [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) or [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width) in the <var>destination</var> axis.
  >
  > - <a id="ref-for-definite⑧"></a>
  >
  >   <a id="ref-for-max-width⑥"></a>
  >
  >   <a id="ref-for-preferred-size⑧"></a>
  >
  >   <a id="ref-for-min-width④"></a>
  >
  >   Then, any [definite](https://www.w3.org/TR/css-sizing-3/#definite) [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width) is converted and transferred from the <var>origin</var> to <var>destination</var>. This transferred maximum is floored by any <a id="ref-for-definite⑨"></a>definite [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) or [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width) in the <var>destination</var> axis as well as by the transferred minimum, if any.
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > Note: The basic principle is that sizing constraints transfer through the aspect-ratio to the other side to preserve the aspect ratio to the extent that they can without violating any sizes specified explicitly on that affected axis.

- <a id="ref-for-propdef-aspect-ratio⑤"></a>

  <a id="ref-for-replaced-element⑨"></a>

  <a id="ref-for-natural-size①"></a>

  Clarify that [aspect-ratio](#propdef-aspect-ratio) on a [replaced element](https://www.w3.org/TR/css-display-4/#replaced-element) with only one [natural size](https://www.w3.org/TR/css-images-3/#natural-size) determines the other dimension. ([Issue 5306](https://github.com/w3c/csswg-drafts/issues/5306))

  > <a id="ref-for-preferred-aspect-ratio①⑨"></a>
  >
  > If a replaced element’s only natural dimension is a natural width or a natural height, giving it a [preferred aspect ratio](#preferred-aspect-ratio) also gives it a natural height or width, whichever was missing, by transferring the existing size through the <a id="ref-for-preferred-aspect-ratio②⓪"></a>preferred aspect ratio.

- <a id="ref-for-propdef-aspect-ratio⑥"></a>

  Define that [aspect-ratio](#propdef-aspect-ratio) inhibits margin self-collapsing ([Issue 5328](https://github.com/w3c/csswg-drafts/issues/5328))

  > <a id="ref-for-block-axis①"></a>
  >
  > <a id="ref-for-ratio-dependent-axis④"></a>
  >
  > <a id="ref-for-computed-value①"></a>
  >
  > <a id="ref-for-propdef-block-size①"></a>
  >
  > <a id="ref-for-valdef-width-auto④"></a>
  >
  > For the purpose of margin collapsing ([CSS 2 § 8.3.1 Collapsing margins](https://www.w3.org/TR/CSS2/box.html#collapsing-margins)), if the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) is the [ratio-dependent axis](#ratio-dependent-axis), it is not considered to have a [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [block-size](https://www.w3.org/TR/css-logical-1/#propdef-block-size) of [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto).

### <a id="additions-L3"></a> Additions Since Level 3

- <a id="ref-for-valdef-width-contain②"></a>

  <a id="ref-for-funcdef-width-fit-content③"></a>

  Added [fit-content()](#funcdef-width-fit-content) function and [contain](#valdef-width-contain) keyword for sizing properties.

- <a id="ref-for-propdef-aspect-ratio⑦"></a>

  Added [aspect-ratio](#propdef-aspect-ratio) property.

- <a id="ref-for-propdef-contain-intrinsic-size①⑤"></a>

  <a id="ref-for-propdef-contain-intrinsic-inline-size"></a>

  <a id="ref-for-propdef-contain-intrinsic-block-size"></a>

  <a id="ref-for-propdef-contain-intrinsic-height②"></a>

  <a id="ref-for-propdef-contain-intrinsic-width②"></a>

  Added [contain-intrinsic-width](#propdef-contain-intrinsic-width), [contain-intrinsic-height](#propdef-contain-intrinsic-height), [contain-intrinsic-block-size](#propdef-contain-intrinsic-block-size), and [contain-intrinsic-inline-size](#propdef-contain-intrinsic-inline-size) properties and their shorthand [contain-intrinsic-size](#propdef-contain-intrinsic-size).

- <a id="ref-for-propdef-min-intrinsic-sizing②"></a>

  Added [min-intrinsic-sizing](#propdef-min-intrinsic-sizing) property.

- <a id="ref-for-propdef-frame-sizing④"></a>

  Added [frame-sizing](#propdef-frame-sizing) property.

## <a id="acknowledgments"></a> Acknowledgments

Special thanks go to Aaron Gustafson, L. David Baron for their contributions to this module.

## <a id="priv-sec"></a> Privacy Considerations

This specification introduces no new privacy considerations.

## <a id="sec"></a> Security Considerations

This specification introduces no new security considerations.

## <a id="w3c-conformance"></a> Conformance

### <a id="w3c-conventions"></a> Document conventions

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#biblio-rfc2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="w3c-example"></a>
>
> This is an example of an informative example.

Informative notes begin with the word “Note” and are set apart from the normative text with `class="note"`, like this:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this is an informative note.

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with `<strong class="advisement">`, like this: <strong data-conversion-semantic="advisement">Advisement:</strong> <strong>
        UAs MUST provide an accessible alternative.
    </strong>

Tests

Tests relating to the content of this specification may be documented in “Tests” blocks like this one. Any such block is non-normative.

------------------------------------------------------------------------

### <a id="w3c-conformance-classes"></a> Conformance classes

Conformance to this specification is defined for three conformance classes:

style sheet  
A [CSS style sheet](https://www.w3.org/TR/CSS21/conform.html#style-sheet).

renderer  
A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

authoring tool  
A [UA](https://www.w3.org/TR/CSS21/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to this specification if all of its statements that use syntax defined in this module are valid according to the generic CSS grammar and the individual grammars of each feature defined in this module.

A renderer is conformant to this specification if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by this specification by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to this specification if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="w3c-partial"></a> Partial implementations

So that authors can exploit the forward-compatible parsing rules to assign fallback values, CSS renderers <strong>must</strong> treat as invalid (and [ignore as appropriate](https://www.w3.org/TR/CSS21/conform.html#ignore)) any at-rules, properties, property values, keywords, and other syntactic constructs for which they have no usable level of support. In particular, user agents <strong>must not</strong> selectively ignore unsupported component values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="w3c-conform-future-proofing"></a> Implementations of Unstable and Proprietary Features

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](https://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](https://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](https://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

### <a id="w3c-testing"></a> Non-experimental implementations

Once a specification reaches the Candidate Recommendation stage, non-experimental implementations are possible, and implementors should release an unprefixed implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [aspect-ratio](#propdef-aspect-ratio), in § 4.1
- auto
  - [value for aspect-ratio](#valdef-aspect-ratio-auto), in § 4.1
  - [value for contain-intrinsic-width, contain-intrinsic-height, contain-intrinsic-block-size, contain-intrinsic-inline-size, contain-intrinsic-size](#valdef-contain-intrinsic-width-auto), in § 5.2
  - [value for frame-sizing](#valdef-frame-sizing-auto), in § 5.3
- [auto &#x26;&#x26; \<ratio\>](#valdef-aspect-ratio-auto--ratio), in § 4.1
- [\<box-size\>](#typedef-box-size), in § 3.2
- [calc-size()](#funcdef-width-calc-size), in § 3.2
- [contain](#valdef-width-contain), in § 3.2
- [contain-fit sizing](#contain-fit-sizing), in § 6
- [contain-intrinsic-block-size](#propdef-contain-intrinsic-block-size), in § 5.2
- [contain-intrinsic-height](#propdef-contain-intrinsic-height), in § 5.2
- [contain-intrinsic-inline-size](#propdef-contain-intrinsic-inline-size), in § 5.2
- [contain-intrinsic-size](#propdef-contain-intrinsic-size), in § 5.2
- [contain-intrinsic-width](#propdef-contain-intrinsic-width), in § 5.2
- [content-block-size](#valdef-frame-sizing-content-block-size), in § 5.3
- [content-height](#valdef-frame-sizing-content-height), in § 5.3
- [content-inline-size](#valdef-frame-sizing-content-inline-size), in § 5.3
- [content-width](#valdef-frame-sizing-content-width), in § 5.3
- [explicit intrinsic inner size](#explicit-intrinsic-inner-size), in § 5.2
- [fit-content](#valdef-column-width-fit-content), in § 5.6
- [fit-content()](#funcdef-width-fit-content), in § 3.2
- [fit-content(\<length-percentage\>)](#valdef-column-width-fit-content-length-percentage), in § 5.6
- [frame-sizing](#propdef-frame-sizing), in § 5.3
- [internal layout intrinsic size](#internal-layout-intrinsic-size), in § 5.3
- [last remembered size](#last-remembered), in § 5.2
- [legacy](#valdef-min-intrinsic-sizing-legacy), in § 5.5
- [\<length\>](#valdef-contain-intrinsic-width-length), in § 5.2
- [locked embedded ICB size](#locked-embedded-icb-size), in § 5.3.1
- [max-content](#valdef-column-width-max-content), in § 5.6
- [max-size](#propdef-max-size), in § 3.1
- [min-content](#valdef-column-width-min-content), in § 5.6
- [min-intrinsic-sizing](#propdef-min-intrinsic-sizing), in § 5.5
- [min-size](#propdef-min-size), in § 3.1
- [none](#valdef-contain-intrinsic-width-none), in § 5.2
- [preferred aspect ratio](#preferred-aspect-ratio), in § 4.1
- [\<ratio\>](#valdef-aspect-ratio-ratio), in § 4.1
- [ratio-dependent axis](#ratio-dependent-axis), in § 4.2
- [ratio-determining axis](#ratio-determining-axis), in § 4.2
- [requestResize()](#dom-window-requestresize), in § 5.3.2
- [responsive embedded sizing flag](#document-responsive-embedded-sizing-flag), in § 5.3.1
- [responsively-sized iframe](#responsive-iframes), in § 5.2.2
- [size](#propdef-size), in § 3.1
- [stretch](#valdef-column-width-stretch), in § 5.6
- [zero-if-extrinsic](#valdef-min-intrinsic-sizing-zero-if-extrinsic), in § 5.5
- [zero-if-scroll](#valdef-min-intrinsic-sizing-zero-if-scroll), in § 5.5

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ALIGN-3\] defines the following terms:
  - <a id="80d2b689"></a>justify-self
  - <a id="35e80fd9"></a>start
  - <a id="598fa031"></a>stretch (for align-self)
  - <a id="8adaf629"></a>stretch (for justify-self)
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="dbdacf0d"></a>background-size
- \[CSS-BOX-4\] defines the following terms:
  - <a id="f72f5cb4"></a>content box
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="e14541aa"></a>shorthand
  - <a id="980ac56a"></a>shorthand property
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="fe28a9e7"></a>content-visibility
  - <a id="3e4b15e8"></a>size containment
  - <a id="b5f3c3da"></a>skipping its contents
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="e8c16097"></a>display
  - <a id="6658d41f"></a>in-flow
  - <a id="d1ebdd75"></a>initial containing block
  - <a id="f089a6e1"></a>inline box
  - <a id="e89ddbcb"></a>non-replaced
  - <a id="7a605ac8"></a>principal box
  - <a id="a9db5d6d"></a>replaced element
- \[CSS-GRID-2\] defines the following terms:
  - <a id="df72a52c"></a>grid container
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="ffedca23"></a>natural aspect ratio
  - <a id="487e1aa9"></a>natural dimension
  - <a id="b9cef6bf"></a>natural height
  - <a id="c0cc78c8"></a>natural size
  - <a id="24ae9eec"></a>natural width
- \[CSS-IMAGES-4\] defines the following terms:
  - <a id="c8427470"></a>contain
  - <a id="99492242"></a>object-fit
- \[CSS-LOGICAL-1\] defines the following terms:
  - <a id="16a98720"></a>block-size
  - <a id="b174cda6"></a>logical property group
- \[CSS-MULTICOL-2\] defines the following terms:
  - <a id="7777143d"></a>column-width
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="add377f4"></a>overflow
  - <a id="a3cabdb1"></a>scroll container
- \[CSS-PAGE-3\] defines the following terms:
  - <a id="2c5a261b"></a>@page
  - <a id="4c32fabe"></a>size
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="58d7f97c"></a>absolutely-positioned
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="c20b5ff5"></a>auto
  - <a id="e9c67130"></a>automatic minimum size
  - <a id="37f6dbd7"></a>automatic size
  - <a id="c1c732b9"></a>available space
  - <a id="6fc38b8c"></a>behaves as auto
  - <a id="54a1fea8"></a>box-sizing
  - <a id="66f218c1"></a>definite
  - <a id="4e2743de"></a>extrinsic sizing
  - <a id="5ad01cca"></a>height
  - <a id="2a2ed19e"></a>indefinite
  - <a id="3ade8b07"></a>intrinsic size
  - <a id="59e3c405"></a>intrinsic size contribution
  - <a id="8cdc912e"></a>max-content
  - <a id="ea5592b4"></a>max-content inline size
  - <a id="8a39af7f"></a>max-content size
  - <a id="6d275904"></a>maximum size
  - <a id="d3da3539"></a>min-content
  - <a id="65c4b34c"></a>min-content contribution
  - <a id="63c3bb64"></a>min-content inline size
  - <a id="6a444fd6"></a>min-content size
  - <a id="4405c984"></a>minimum size
  - <a id="8926c8c7"></a>none
  - <a id="dd09245c"></a>preferred size
  - <a id="2ac08cff"></a>sizing property
  - <a id="de42723d"></a>stretch-fit inline size
  - <a id="97ac8088"></a>stretch-fit size
  - <a id="49731d1d"></a>width
- \[CSS-SIZING-4\] defines the following terms:
  - <a id="6b530a45"></a>fit-content
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="ee68e69a"></a>\<ratio\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="ebcca398"></a>degenerate ratio
  - <a id="3bafef5e"></a>{A,B}
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="eb6008ce"></a>writing mode
- \[CSS2\] defines the following terms:
  - <a id="0e782f73"></a>document language
  - <a id="0f0ab49f"></a>max-height
  - <a id="4d8f6525"></a>max-width
  - <a id="62b90f98"></a>min-height
  - <a id="1ecca6e7"></a>min-width
- \[CSSOM-VIEW-1\] defines the following terms:
  - <a id="6cb4791e"></a>scrolling area
- \[DOM\] defines the following terms:
  - <a id="85394472"></a>Document
- \[HTML\] defines the following terms:
  - <a id="7bcff2a1"></a>DOMContentLoaded
  - <a id="5d7209e9"></a>Window
  - <a id="35972864"></a>active document
  - <a id="2f0492ac"></a>body
  - <a id="f23dc013"></a>child navigable
  - <a id="3ecc7220"></a>content navigable
  - <a id="0cae1143"></a>content window
  - <a id="273ad187"></a>head
  - <a id="87fcd40c"></a>iframe
  - <a id="18e7dde9"></a>load
  - <a id="64b89595"></a>meta
  - <a id="9c19b7fc"></a>navigable
  - <a id="bf63a402"></a>navigable container
- \[RESIZE-OBSERVER-1\] defines the following terms:
  - <a id="5e16daa2"></a>ResizeObserver
- \[WEBIDL\] defines the following terms:
  - <a id="dca2de17"></a>DOMException
  - <a id="ba556545"></a>NotAllowedError
  - <a id="4013a022"></a>this
  - <a id="b4cfa5ce"></a>throw
  - <a id="5f90bbfb"></a>undefined

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 30 January 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-contain-1"></a>\[CSS-CONTAIN-1\]  
Tab Atkins Jr.; Florian Rivoal. [CSS Containment Module Level 1](https://www.w3.org/TR/css-contain-1/). 25 June 2024. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-1&#x2F;](https://www.w3.org/TR/css-contain-1/)

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 17 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-2&#x2F;](https://www.w3.org/TR/css-contain-2/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 6 November 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Elika Etemad; Tab Atkins Jr.; Lea Verou. [CSS Images Module Level 4](https://www.w3.org/TR/css-images-4/). 30 September 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-logical-1"></a>\[CSS-LOGICAL-1\]  
Elika Etemad; Rossen Atanassov. [CSS Logical Properties and Values Module Level 1](https://www.w3.org/TR/css-logical-1/). 4 December 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-logical-1&#x2F;](https://www.w3.org/TR/css-logical-1/)

<a id="biblio-css-multicol-2"></a>\[CSS-MULTICOL-2\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 2](https://www.w3.org/TR/css-multicol-2/). 19 December 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-2&#x2F;](https://www.w3.org/TR/css-multicol-2/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-page-3"></a>\[CSS-PAGE-3\]  
Elika Etemad. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 14 September 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-sizing-4"></a>\[CSS-SIZING-4\]  
Tab Atkins Jr.; Elika Etemad; Jen Simmons. [CSS Box Sizing Module Level 4](https://www.w3.org/TR/css-sizing-4/). 20 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-4&#x2F;](https://www.w3.org/TR/css-sizing-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-values-5"></a>\[CSS-VALUES-5\]  
Tab Atkins Jr.; Elika Etemad; Miriam Suzanne. [CSS Values and Units Module Level 5](https://www.w3.org/TR/css-values-5/). 11 November 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-5&#x2F;](https://www.w3.org/TR/css-values-5/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-css3col"></a>\[CSS3COL\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 1](https://www.w3.org/TR/css-multicol-1/). 16 May 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-1&#x2F;](https://www.w3.org/TR/css-multicol-1/)

<a id="biblio-cssom-view-1"></a>\[CSSOM-VIEW-1\]  
Simon Fraser; Emilio Cobos Álvarez. [CSSOM View Module](https://www.w3.org/TR/cssom-view-1/). 16 September 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-view-1&#x2F;](https://www.w3.org/TR/cssom-view-1/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-resize-observer-1"></a>\[RESIZE-OBSERVER-1\]  
Aleks Totic; Greg Whitworth. [Resize Observer](https://www.w3.org/TR/resize-observer-1/). 11 February 2020. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;resize-observer-1&#x2F;](https://www.w3.org/TR/resize-observer-1/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Non-Normative References

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                                                | Initial                   | Applies to                                                        | Inh.                      | %ages                     | Anim­ation type            | Canonical order | Com­puted value                                | Logical property group |
|---------------------|--------------------------------------------------------------------------------------|---------------------------|-------------------------------------------------------------------|---------------------------|---------------------------|---------------------------|-----------------|-----------------------------------------------|------------------------|
| <strong><span><a id="ref-for-propdef-aspect-ratio⑧"></a></span><a href="#propdef-aspect-ratio">aspect-ratio</a>&#xA;      </strong> | auto \|\| \<ratio\>                                                                  | auto                      | all elements except inline boxes and internal ruby or table boxes | no                        | n/a                       | by computed value         | per grammar     | specified keyword or a pair of numbers        |                        |
| <strong><span><a id="ref-for-propdef-contain-intrinsic-block-size①"></a></span><a href="#propdef-contain-intrinsic-block-size">contain-intrinsic-block-size</a>&#xA;      </strong> | auto? \[ none \| \<length \[0,∞\]\> \]                                               | none                      | elements with size containment                                    | no                        | n/a                       | by computed value type    | per grammar     | as specified, with \<length\> values computed | contain-intrinsic-size |
| <strong><span><a id="ref-for-propdef-contain-intrinsic-height③"></a></span><a href="#propdef-contain-intrinsic-height">contain-intrinsic-height</a>&#xA;      </strong> | auto? \[ none \| \<length \[0,∞\]\> \]                                               | none                      | elements with size containment                                    | no                        | n/a                       | by computed value type    | per grammar     | as specified, with \<length\> values computed | contain-intrinsic-size |
| <strong><span><a id="ref-for-propdef-contain-intrinsic-inline-size①"></a></span><a href="#propdef-contain-intrinsic-inline-size">contain-intrinsic-inline-size</a>&#xA;      </strong> | auto? \[ none \| \<length \[0,∞\]\> \]                                               | none                      | elements with size containment                                    | no                        | n/a                       | by computed value type    | per grammar     | as specified, with \<length\> values computed | contain-intrinsic-size |
| <strong><span><a id="ref-for-propdef-contain-intrinsic-size①⑥"></a></span><a href="#propdef-contain-intrinsic-size">contain-intrinsic-size</a>&#xA;      </strong> | \[ auto? \[ none \| \<length \[0,∞\]\> \] \]{1,2}                                    | see individual properties | see individual properties                                         | see individual properties | see individual properties | see individual properties | per grammar     | see individual properties                     |                        |
| <strong><span><a id="ref-for-propdef-contain-intrinsic-width③"></a></span><a href="#propdef-contain-intrinsic-width">contain-intrinsic-width</a>&#xA;      </strong> | auto? \[ none \| \<length \[0,∞\]\> \]                                               | none                      | elements with size containment                                    | no                        | n/a                       | by computed value type    | per grammar     | as specified, with \<length\> values computed | contain-intrinsic-size |
| <strong><span><a id="ref-for-propdef-frame-sizing⑤"></a></span><a href="#propdef-frame-sizing">frame-sizing</a>&#xA;      </strong> | auto \| content-width \| content-height \| content-block-size \| content-inline-size | auto                      | replaced elements (but see below for details)                     | no                        | n/a                       | discrete                  | per grammar     | as specified                                  |                        |
| <strong><span><a id="ref-for-propdef-max-size②"></a></span><a href="#propdef-max-size">max-size</a>&#xA;      </strong> | \<'max-width'\> \<'max-height'\>?                                                    | none                      | all elements                                                      | see individual properties | see individual properties | see individual properties | per grammar     | see individual properties                     |                        |
| <strong><span><a id="ref-for-propdef-min-intrinsic-sizing③"></a></span><a href="#propdef-min-intrinsic-sizing">min-intrinsic-sizing</a>&#xA;      </strong> | legacy \| zero-if-scroll \|\| zero-if-extrinsic                                      | legacy                    | all elements except inline boxes                                  | no                        | n/a                       | discrete                  | per grammar     | as specified                                  |                        |
| <strong><span><a id="ref-for-propdef-min-size②"></a></span><a href="#propdef-min-size">min-size</a>&#xA;      </strong> | \<'min-width'\> \<'min-height'\>?                                                    | auto                      | all elements                                                      | see individual properties | see individual properties | see individual properties | per grammar     | see individual properties                     |                        |
| <strong><span><a id="ref-for-propdef-size④"></a></span><a href="#propdef-size">size</a>&#xA;      </strong> | \<'width'\> \<'height'\>?                                                            | auto                      | all elements                                                      | see individual properties | see individual properties | see individual properties | per grammar     | see individual properties                     |                        |

## <a id="idl-index"></a>IDL Index

```text
partial interface Window {
  undefined requestResize();
};

```
## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This is a diff spec over [CSS Sizing Level 3](https://www.w3.org/TR/css-sizing-3/). It is currently an Exploratory Working Draft: if you are implementing anything, please use Level 3 as a reference. We will merge the Level 3 text into this draft once it reaches CR. [↵](#issue-f7037e82)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Sizing 3 § 2 Terminology](https://www.w3.org/TR/css-sizing-3/#terms) [↵](#issue-d41d8cd9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Sizing 3 § 3 Specifying Box Sizes](https://www.w3.org/TR/css-sizing-3/#specifying-sizes) [↵](#issue-d41d8cd9%E2%91%A0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Sizing 3 § 3.1 Sizing Properties](https://www.w3.org/TR/css-sizing-3/#sizing-properties) [↵](#issue-d41d8cd9%E2%91%A1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The [size](#propdef-size) property needs to be omitted from the <u>preferred shorthand order</u> in CSSOM, to avoid compat problems and conflicts with [@page](https://www.w3.org/TR/css-page-3/#at-ruledef-page). [↵](#issue-bded40fa)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Sizing 3 § 3.2 Sizing Values: the \<length-percentage\>, auto \| none, min-content, max-content, and fit-content() values](https://www.w3.org/TR/css-sizing-3/#sizing-values) [↵](#issue-d41d8cd9%E2%91%A2)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> We are still working through the details of this section. If there is any behavior specified here that would cause [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element) with a [preferred aspect ratio](#preferred-aspect-ratio) to behave differently than they would under the requirements of the [CSS2](https://www.w3.org/TR/CSS2/visudet.html), [Flex Layout](https://www.w3.org/TR/css-flexbox-1/), and [Grid Layout](https://www.w3.org/TR/css-grid-1/) specs combined (without this specification in effect), <strong>this is an error and should be <a href="https://github.com/w3c/csswg-drafts/issues">reported</a> to the CSSWG</strong>. There is a [list of open aspect-ratio issues](https://github.com/w3c/csswg-drafts/issues?q=is%3Aissue%20state%3Aopen%20label%3Acss-sizing-4%20aspect-ratio%20label%3A%22Needs%20Edits%22). [↵](#issue-9c24ea7a)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> CSS2.1 does not cleanly differentiate between replaced elements vs. elements with an aspect ratio; need to figure out specific cases that are unclear and define them, either in the appropriate Level 3 spec or here. [↵](#issue-ada2dbb9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> When we move all the sizing information here, rather than crowbar-ing our way into 2.1, then the core principle here is just: the resolved [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) in the ratio-determining axis (before applying min/max) gets transferred thru the ratio. Min/max constraints get transferred afterwards, and then applied to each axis independently without regards to aspect-ratio. [↵](#issue-36f34fa3)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This section might not be written correctly. [\[Issue \#6071\]](https://github.com/w3c/csswg-drafts/issues/6071) [↵](#issue-0273742f)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Sizing 3 § 5.1 Intrinsic Sizes](https://www.w3.org/TR/css-sizing-3/#intrinsic-sizes) [↵](#issue-d41d8cd9%E2%91%A3)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Do we want to have a way to force an iframe to forget its [locked embedded ICB size](#locked-embedded-icb-size)? Maybe turning [frame-sizing](#propdef-frame-sizing) off and on again? [↵](#issue-6b9851d9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Sizing 3 § 5.2 Intrinsic Contributions](https://www.w3.org/TR/css-sizing-3/#intrinsic-contribution) [↵](#issue-d41d8cd9%E2%91%A4)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This property seriously needs some name bikeshedding. [↵](#issue-b731f1ac)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [CSS Sizing 3 § 4 Extrinsic Size Determination](https://www.w3.org/TR/css-sizing-3/#extrinsic) [↵](#issue-d41d8cd9%E2%91%A5)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Copy whatever stretch-fit ends up doing wrt margin collapsing. [↵](#issue-6e598fc0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> If there is a minimum size in one dimension that would cause overflow of the target rectangle if the aspect ratio were honored, do we honor the aspect ratio or skew the image? If the former, we need a step similar to \#2 that applies the relevant minimums. [↵](#issue-9d61aaed)
