Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Multi-column Layout Module Level 1](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Multi-column Layout Module Level 1

Source snapshot: https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/

Snapshot SHA-256: 0768f4526ac5b4be220169db13f1db9fb617c81c8112062a9df4883078220d09

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 10 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Multi-column Layout Module Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This specification describes multi-column layouts in CSS, a style sheet language for the web. Using functionality described in the specification, content can be flowed into multiple columns with a gap and a rule between them.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Snapshot</strong> using the [Recommendation track](https://www.w3.org/2023/Process-20231103/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Snapshot has received [wide review](https://www.w3.org/2023/Process-20231103/#dfn-wide-review), is intended to gather implementation experience, and has commitments from Working Group members to [royalty-free licensing](https://www.w3.org/Consortium/Patent-Policy/#sec-Requirements) for implementations. This document is intended to become a W3C Recommendation; it will remain a Candidate Recommendation at least until 9 July 2024 to gather additional feedback.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-multicol” in the title, like this: “\[css-multicol\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-multicol%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/2023/Process-20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="introduction"></a>1.  Introduction

(This section is not normative.)

This module describes <a id="multi-column-layout"></a>multi-column layout in CSS. By using functionality described in this document, style sheets can declare that the content of an element is to be laid out in multiple columns.

<a id="ref-for-grid-container"></a>

<a id="ref-for-grid-item"></a>

Other layout methods in CSS, when applied to a parent element, change the display properties of the direct children. For example if a three column grid layout is created, the direct children of the [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) become [grid items](https://www.w3.org/TR/css-grid-2/#grid-item) and are placed into the column tracks, one element per cell with additional rows created as needed.

<a id="ref-for-multi-column-container"></a>

The child elements of a [multi-column container](#multi-column-container) however continue in normal flow, that flow is arranged into a number of columns. These columns have a flexible inline size, and therefore respond to available space by changing the size or number of columns displayed.

Multi-column layouts are easy to describe in CSS. Here is a simple example:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-90f03bde"></a>
>
> ```text
> body { column-width: 12em }
> ```
>
> In this example, the `body` element is set to have columns at least 12em wide. The exact number of columns will depend on the available space.

The number of columns can also be set explicitly in the style sheet:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7e147b89"></a>
>
> ```text
> body { column-count: 2 }
> ```
>
> In this case, the number of columns is fixed and the column widths will vary depending on the available width.

<a id="ref-for-propdef-columns"></a>

The shorthand [columns](#propdef-columns) property can be used to set either, or both, properties in one declaration.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6c6d6717"></a> In these examples, the number of columns, the width of columns, and both the number and width are set, respectively:
>
> ```text
> body { columns: 2 }
> body { columns: 12em }
> body { columns: 2 12em }
> ```
Another group of properties introduced in this module describe gaps and rules between columns.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-21b75f3b"></a>
>
> ```text
> body {
>   column-gap: 1em;
>   column-rule: thin solid black;
> }
> ```
>
> <a id="ref-for-propdef-column-rule"></a>
>
> The first declaration in the example above sets the gap between two adjacent columns to be 1em. Column gaps are similar to padding areas. In the middle of the gap there will be a rule which is described by the [column-rule](#propdef-column-rule) property.

<a id="ref-for-propdef-column-rule①"></a>

<a id="ref-for-propdef-border"></a>

The values of the [column-rule](#propdef-column-rule) property are similar to those of the CSS [border](https://www.w3.org/TR/css-backgrounds-3/#propdef-border) properties. Like <a id="ref-for-propdef-border①"></a>border, <a id="ref-for-propdef-column-rule②"></a>column-rule is a shorthand property.

<a id="ref-for-propdef-column-rule③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-968144c8"></a> In this example, the shorthand [column-rule](#propdef-column-rule) declaration from the above example has been expanded:
>
> ```text
> body {
>   column-gap: 1em;
>   column-rule-width: thin;
>   column-rule-style: solid;
>   column-rule-color: black;
> }
> ```
<a id="ref-for-propdef-column-fill"></a>

<a id="ref-for-propdef-column-span"></a>

The [column-fill](#propdef-column-fill) and [column-span](#propdef-column-span) properties give style sheets a wider range of visual expressions in multi-column layouts.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8550a53a"></a> In this example, columns are set to be balanced, i.e., to have approximately the same length. Also, `h2` elements are set to span across all columns.
>
> ```text
> div { column-fill: balance }
> h2 { column-span: all }
> ```
Tests

- [multicol-fill-balance-029.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-029.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-029.html)

This specification introduces ten new properties, all of which are used in the examples above.

If all column properties have their initial value, the layout of an element will be identical to a multi-column layout with only one column.

<a id="ref-for-column-gap"></a>

<a id="ref-for-column-rule"></a>

<a id="ref-for-multi-column-container①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-47371199"></a> [Column gaps](#column-gap) (diagonal hatching) and [column rules](#column-rule) are shown in this sample rendition of a multi-column container with padding (cross hatching). The hatched areas are present for illustrational purposes only. In actual implementations these areas will be determined by the background, the second image shows a rendering of a [multi-column container](#multi-column-container) with column-rules.
>
> ![a diagram showing the various parts of multi-column layout](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/initial-example.svg) ![key to the conventions used to display invisible parts of diagram](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/invisible-elements.svg)
>
> A multi-column layout with the non-visible column-span and padding inside the multicol container highlighted.
>
> ![a diagram showing the various parts of multi-column layout](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/initial-example-b.svg)
>
> The same layout as in the first image, as it would be displayed by an implementation.

### <a id="values"></a>1.1.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS21\]](#biblio-css21) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="the-multi-column-model"></a>2.  The Multi-Column Model

<a id="ref-for-propdef-column-width"></a>

<a id="ref-for-propdef-column-count"></a>

<a id="ref-for-valdef-column-width-auto"></a>

<a id="ref-for-multi-column-layout"></a>

An element whose [column-width](#propdef-column-width) or [column-count](#propdef-column-count) property is not [auto](#valdef-column-width-auto) establishes a <a id="multi-column-container"></a>multi-column container<a id="multi-column-element"></a> (or <i>multicol container</i> for short), and therefore acts as a container for [multi-column layout](#multi-column-layout).

Tests

Basic multicol tests.

- [multicol-basic-002.html](https://wpt.fyi/results/css/css-multicol/multicol-basic-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-basic-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-basic-002.html)
- [multicol-basic-004.html](https://wpt.fyi/results/css/css-multicol/multicol-basic-004.html) [(live test)](http://wpt.live/css/css-multicol/multicol-basic-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-basic-004.html)

------------------------------------------------------------------------

Tests demonstrating that auto values do not create a multicol container.

- [multicol-count-computed-004.xht](https://wpt.fyi/results/css/css-multicol/multicol-count-computed-004.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-count-computed-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-count-computed-004.xht)

------------------------------------------------------------------------

Multicol properties do not inherit.

- [inheritance.html](https://wpt.fyi/results/css/css-multicol/inheritance.html) [(live test)](http://wpt.live/css/css-multicol/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/inheritance.html)

------------------------------------------------------------------------

Multicol with scrolled columns.

- [multicol-scroll-content.html](https://wpt.fyi/results/css/css-multicol/multicol-scroll-content.html) [(live test)](http://wpt.live/css/css-multicol/multicol-scroll-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-scroll-content.html)

------------------------------------------------------------------------

Multicol with zero height.

- [multicol-zero-height-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-zero-height-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-zero-height-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-zero-height-001.xht)
- [multicol-zero-height-002.html](https://wpt.fyi/results/css/css-multicol/multicol-zero-height-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-zero-height-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-zero-height-002.html)
- [multicol-zero-height-003.html](https://wpt.fyi/results/css/css-multicol/multicol-zero-height-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-zero-height-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-zero-height-003.html)

------------------------------------------------------------------------

<a id="ref-for-fragmentation-context"></a>

<a id="ref-for-anonymous"></a>

<a id="ref-for-fragmentation-container"></a>

<a id="ref-for-column-box"></a>

<a id="ref-for-block-formatting-context"></a>

<a id="ref-for-containing-block"></a>

In the traditional CSS box model, the content of an element is flowed into the content box of the corresponding element. Multi-column layout introduces a [fragmentation context](https://www.w3.org/TR/css-break-4/#fragmentation-context) formed of [anonymous](https://www.w3.org/TR/css-display-3/#anonymous) [fragmentation containers](https://www.w3.org/TR/css-break-4/#fragmentation-container) called <a id="column-box"></a>column boxes (or <i>columns</i> for short). These [column boxes](#column-box) establish an independent [block formatting context](https://www.w3.org/TR/css-display-3/#block-formatting-context) into which the multi-column container’s content flows, and form the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) for its non-positioned children.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5cbb5b6f"></a> In this example, the width of the image is set with these rules:
>
> ```text
> img {
>   display: block;
>   width: 100%;
> }
> ```
>
> <a id="ref-for-block-formatting-context①"></a>
>
> <a id="ref-for-propdef-width"></a>
>
> Given that the column box creates a new [block formatting context](https://www.w3.org/TR/css-display-3/#block-formatting-context), the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) is calculated relative to the column box. Therefore the image will not overflow the column box:
>
> ![an image contained inside a column box](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/image-inside-column.svg)
>
> The image is constrained by the column box that it is displayed in.

<a id="ref-for-block-formatting-context②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-88e557c8"></a> Given that the column box creates a new [block formatting context](https://www.w3.org/TR/css-display-3/#block-formatting-context), a top margin set on the first child element of a multicol container will not collapse with the margins of the multicol container.
>
> ![The first paragraph has a 'margin-top' of ''1em'', which appears before the text.](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/margins-do-not-collapse.svg)
>
> The margin above the first paragraph has not collapsed, leaving a 1em margin above the first line in the multicol container.

Tests

- [multicol-margin-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-margin-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-margin-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-margin-001.xht)
- [multicol-margin-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-margin-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-margin-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-margin-002.xht)
- [multicol-margin-003.html](https://wpt.fyi/results/css/css-multicol/multicol-margin-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-margin-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-margin-003.html)
- [multicol-margin-child-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-margin-child-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-margin-child-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-margin-child-001.xht)
- [multicol-nested-margin-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-nested-margin-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-margin-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-margin-001.xht)
- [multicol-nested-margin-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-nested-margin-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-margin-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-margin-002.xht)
- [multicol-nested-margin-003.xht](https://wpt.fyi/results/css/css-multicol/multicol-nested-margin-003.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-margin-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-margin-003.xht)
- [multicol-nested-margin-004.xht](https://wpt.fyi/results/css/css-multicol/multicol-nested-margin-004.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-margin-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-margin-004.xht)
- [multicol-nested-margin-005.xht](https://wpt.fyi/results/css/css-multicol/multicol-nested-margin-005.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-margin-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-margin-005.xht)
- [multicol-collapsing-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-collapsing-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-collapsing-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-collapsing-001.xht)

<a id="ref-for-column-box①"></a>

Floats that appear inside multi-column layouts are positioned with regard to the [column box](#column-box) where the float appears.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3dc32251"></a> In this example, this CSS fragment describes the presentation of the image:
>
> ```text
> img {
>   display: block;
>   float: right;
> }
> ```
>
> In the HTML, the image appears after the sentence ending, "the leg of a chicken".
>
> ![an image floated and contained inside a column box](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/image-floated-in-column.svg)
>
> The image is floated inside the column box it appears in.

<a id="ref-for-column-box②"></a>

<a id="ref-for-block-axis"></a>

<a id="ref-for-fragment"></a>

Content overflowing a [column box](#column-box) in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) [fragments](https://www.w3.org/TR/css-break-4/#fragment) and continues in the next <a id="ref-for-column-box③"></a>column box.

<a id="ref-for-anonymous①"></a>

<a id="ref-for-containing-block①"></a>

<a id="ref-for-absolute-position"></a>

<a id="ref-for-propdef-position"></a>

<a id="ref-for-multi-column-container②"></a>

<a id="ref-for-principal-box"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Column boxes, which are [anonymous boxes](https://www.w3.org/TR/css-display-3/#anonymous), do not become the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) for [absolutely positioned boxes](https://www.w3.org/TR/css-position-3/#absolute-position). The [position](https://www.w3.org/TR/css-position-3/#propdef-position) property, which establishes a containing block for such boxes, applies to the [multicol container](#multi-column-container), it being the [principal box](https://www.w3.org/TR/css-display-3/#principal-box).

Tests

- [multicol-containing-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-containing-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-containing-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-containing-001.xht)
- [multicol-containing-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-containing-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-containing-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-containing-002.xht)
- [multicol-containing-003.html](https://wpt.fyi/results/css/css-multicol/multicol-containing-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-containing-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-containing-003.html)
- [multicol-contained-absolute.html](https://wpt.fyi/results/css/css-multicol/multicol-contained-absolute.html) [(live test)](http://wpt.live/css/css-multicol/multicol-contained-absolute.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-contained-absolute.html)
- [abspos-autopos-contained-by-viewport-000.html](https://wpt.fyi/results/css/css-multicol/abspos-autopos-contained-by-viewport-000.html) [(live test)](http://wpt.live/css/css-multicol/abspos-autopos-contained-by-viewport-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/abspos-autopos-contained-by-viewport-000.html)
- [abspos-autopos-contained-by-viewport-001.html](https://wpt.fyi/results/css/css-multicol/abspos-autopos-contained-by-viewport-001.html) [(live test)](http://wpt.live/css/css-multicol/abspos-autopos-contained-by-viewport-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/abspos-autopos-contained-by-viewport-001.html)
- [abspos-multicol-in-second-outer-clipped.html](https://wpt.fyi/results/css/css-multicol/abspos-multicol-in-second-outer-clipped.html) [(live test)](http://wpt.live/css/css-multicol/abspos-multicol-in-second-outer-clipped.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/abspos-multicol-in-second-outer-clipped.html)

<a id="ref-for-multi-column-container③"></a>

<a id="ref-for-propdef-position①"></a>

<a id="ref-for-column-box④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-aaa86638"></a> In this example, the [multi-column container](#multi-column-container) has [position: relative](https://www.w3.org/TR/css-position-3/#propdef-position) thus becoming the containing block. The image is a direct child of the <a id="ref-for-multi-column-container④"></a>multi-column container and has <a id="ref-for-propdef-position②"></a>position: absolute. It takes positioning from the <a id="ref-for-multi-column-container⑤"></a>multi-column container and not from the [column box](#column-box).
>
> ```text
> .container {
>   position: relative;
>   column-count: 3;
> }
> img {
>   position: absolute;
>   top: 20px;
>   left: 40px;
> }
> ```
>
> ![The absolutely positioned image is positioned by reference to the \[=multi-column container=\] not the \[=column box=\].](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/column-not-containing-block.svg)
>
> The figure demonstrates that the absolutely positioned image is positioned by reference to the multicol container and not the column box.

<a id="ref-for-multi-column-container⑥"></a>

Out-of-flow descendants of a [multi-column container](#multi-column-container) do affect column balancing, and the block-size of the <a id="ref-for-multi-column-container⑦"></a>multi-column container.

Tests

- [multicol-oof-inline-cb-001.html](https://wpt.fyi/results/css/css-multicol/multicol-oof-inline-cb-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-oof-inline-cb-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-oof-inline-cb-001.html)
- [multicol-oof-inline-cb-002.html](https://wpt.fyi/results/css/css-multicol/multicol-oof-inline-cb-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-oof-inline-cb-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-oof-inline-cb-002.html)

<a id="ref-for-inline-base-direction"></a>

The column boxes are ordered in the [inline base direction](https://www.w3.org/TR/css-writing-modes-4/#inline-base-direction) of the multicol container and arranged into <a id="multi-column-line"></a>multicol lines. The <a id="column-width"></a>column width is the length of the column box in the inline direction. The <a id="column-height"></a>column height is the length of the column box in the block direction. All column boxes in a line have the same column width, and all column boxes in a line have the same column height.

Tests

The following tests relate to baseline alignment of the content of columns, though this is not defined in this specification.

- [baseline-000.html](https://wpt.fyi/results/css/css-multicol/baseline-000.html) [(live test)](http://wpt.live/css/css-multicol/baseline-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/baseline-000.html)
- [baseline-001.html](https://wpt.fyi/results/css/css-multicol/baseline-001.html) [(live test)](http://wpt.live/css/css-multicol/baseline-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/baseline-001.html)
- [baseline-002.html](https://wpt.fyi/results/css/css-multicol/baseline-002.html) [(live test)](http://wpt.live/css/css-multicol/baseline-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/baseline-002.html)
- [baseline-003.html](https://wpt.fyi/results/css/css-multicol/baseline-003.html) [(live test)](http://wpt.live/css/css-multicol/baseline-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/baseline-003.html)
- [baseline-004.html](https://wpt.fyi/results/css/css-multicol/baseline-004.html) [(live test)](http://wpt.live/css/css-multicol/baseline-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/baseline-004.html)
- [baseline-005.html](https://wpt.fyi/results/css/css-multicol/baseline-005.html) [(live test)](http://wpt.live/css/css-multicol/baseline-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/baseline-005.html)
- [baseline-006.html](https://wpt.fyi/results/css/css-multicol/baseline-006.html) [(live test)](http://wpt.live/css/css-multicol/baseline-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/baseline-006.html)
- [baseline-007.html](https://wpt.fyi/results/css/css-multicol/baseline-007.html) [(live test)](http://wpt.live/css/css-multicol/baseline-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/baseline-007.html)
- [baseline-008.html](https://wpt.fyi/results/css/css-multicol/baseline-008.html) [(live test)](http://wpt.live/css/css-multicol/baseline-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/baseline-008.html)
- [as-baseline-aligned-grid-item.html](https://wpt.fyi/results/css/css-multicol/crashtests/as-baseline-aligned-grid-item.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/as-baseline-aligned-grid-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/as-baseline-aligned-grid-item.html)

------------------------------------------------------------------------

The following tests check the behavior of list items that are also muticol containers.

- [multicol-list-item-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-list-item-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-list-item-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-list-item-001.xht)
- [multicol-list-item-002.html](https://wpt.fyi/results/css/css-multicol/multicol-list-item-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-list-item-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-list-item-002.html)
- [multicol-list-item-003.html](https://wpt.fyi/results/css/css-multicol/multicol-list-item-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-list-item-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-list-item-003.html)
- [multicol-list-item-004.html](https://wpt.fyi/results/css/css-multicol/multicol-list-item-004.html) [(live test)](http://wpt.live/css/css-multicol/multicol-list-item-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-list-item-004.html)
- [multicol-list-item-005.html](https://wpt.fyi/results/css/css-multicol/multicol-list-item-005.html) [(live test)](http://wpt.live/css/css-multicol/multicol-list-item-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-list-item-005.html)
- [multicol-list-item-006.html](https://wpt.fyi/results/css/css-multicol/multicol-list-item-006.html) [(live test)](http://wpt.live/css/css-multicol/multicol-list-item-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-list-item-006.html)
- [multicol-list-item-007.html](https://wpt.fyi/results/css/css-multicol/multicol-list-item-007.html) [(live test)](http://wpt.live/css/css-multicol/multicol-list-item-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-list-item-007.html)
- [multicol-list-item-008.html](https://wpt.fyi/results/css/css-multicol/multicol-list-item-008.html) [(live test)](http://wpt.live/css/css-multicol/multicol-list-item-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-list-item-008.html)

------------------------------------------------------------------------

Testing grid items inside multicol

- [balance-grid-001.html](https://wpt.fyi/results/css/css-multicol/balance-grid-001.html) [(live test)](http://wpt.live/css/css-multicol/balance-grid-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/balance-grid-001.html)

------------------------------------------------------------------------

The following tests check the behavior of table elements.

- [multicol-table-cell-001.xht](https://wpt.fyi/results/css/css-multicol/table/multicol-table-cell-001.xht) [(live test)](http://wpt.live/css/css-multicol/table/multicol-table-cell-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/table/multicol-table-cell-001.xht)
- [multicol-table-cell-height-001.xht](https://wpt.fyi/results/css/css-multicol/table/multicol-table-cell-height-001.xht) [(live test)](http://wpt.live/css/css-multicol/table/multicol-table-cell-height-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/table/multicol-table-cell-height-001.xht)
- [multicol-table-cell-height-002.xht](https://wpt.fyi/results/css/css-multicol/table/multicol-table-cell-height-002.xht) [(live test)](http://wpt.live/css/css-multicol/table/multicol-table-cell-height-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/table/multicol-table-cell-height-002.xht)
- [multicol-table-cell-vertical-align-001.xht](https://wpt.fyi/results/css/css-multicol/table/multicol-table-cell-vertical-align-001.xht) [(live test)](http://wpt.live/css/css-multicol/table/multicol-table-cell-vertical-align-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/table/multicol-table-cell-vertical-align-001.xht)
- [table-cell-as-multicol.html](https://wpt.fyi/results/css/css-multicol/table/table-cell-as-multicol.html) [(live test)](http://wpt.live/css/css-multicol/table/table-cell-as-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/table/table-cell-as-multicol.html)
- [table-cell-content-change-000.html](https://wpt.fyi/results/css/css-multicol/table/table-cell-content-change-000.html) [(live test)](http://wpt.live/css/css-multicol/table/table-cell-content-change-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/table/table-cell-content-change-000.html)
- [table-cell-content-change-001.html](https://wpt.fyi/results/css/css-multicol/table/table-cell-content-change-001.html) [(live test)](http://wpt.live/css/css-multicol/table/table-cell-content-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/table/table-cell-content-change-001.html)
- [table-cell-multicol-nested-001.html](https://wpt.fyi/results/css/css-multicol/table/table-cell-multicol-nested-001.html) [(live test)](http://wpt.live/css/css-multicol/table/table-cell-multicol-nested-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/table/table-cell-multicol-nested-001.html)
- [table-cell-multicol-nested-002.html](https://wpt.fyi/results/css/css-multicol/table/table-cell-multicol-nested-002.html) [(live test)](http://wpt.live/css/css-multicol/table/table-cell-multicol-nested-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/table/table-cell-multicol-nested-002.html)
- [table-cell-multicol-nested-003.html](https://wpt.fyi/results/css/css-multicol/table/table-cell-multicol-nested-003.html) [(live test)](http://wpt.live/css/css-multicol/table/table-cell-multicol-nested-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/table/table-cell-multicol-nested-003.html)
- [break-before-multicol-caption.html](https://wpt.fyi/results/css/css-multicol/crashtests/break-before-multicol-caption.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/break-before-multicol-caption.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/break-before-multicol-caption.html)
- [table-cell-writing-mode-root.html](https://wpt.fyi/results/css/css-multicol/crashtests/table-cell-writing-mode-root.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/table-cell-writing-mode-root.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/table-cell-writing-mode-root.html)
- [balance-breakafter-before-table-section-crash.html](https://wpt.fyi/results/css/css-multicol/table/balance-breakafter-before-table-section-crash.html) [(live test)](http://wpt.live/css/css-multicol/table/balance-breakafter-before-table-section-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/table/balance-breakafter-before-table-section-crash.html)
- [balance-table-with-border-spacing.html](https://wpt.fyi/results/css/css-multicol/table/balance-table-with-border-spacing.html) [(live test)](http://wpt.live/css/css-multicol/table/balance-table-with-border-spacing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/table/balance-table-with-border-spacing.html)
- [balance-table-with-fractional-height-row.html](https://wpt.fyi/results/css/css-multicol/table/balance-table-with-fractional-height-row.html) [(live test)](http://wpt.live/css/css-multicol/table/balance-table-with-fractional-height-row.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/table/balance-table-with-fractional-height-row.html)

------------------------------------------------------------------------

The following tests check that paint order is correct.

- [float-and-block.html](https://wpt.fyi/results/css/css-multicol/float-and-block.html) [(live test)](http://wpt.live/css/css-multicol/float-and-block.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/float-and-block.html)
- [move-with-text-after-paint.html](https://wpt.fyi/results/css/css-multicol/move-with-text-after-paint.html) [(live test)](http://wpt.live/css/css-multicol/move-with-text-after-paint.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/move-with-text-after-paint.html)
- [resize-with-text-after-paint.html](https://wpt.fyi/results/css/css-multicol/resize-with-text-after-paint.html) [(live test)](http://wpt.live/css/css-multicol/resize-with-text-after-paint.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/resize-with-text-after-paint.html)

------------------------------------------------------------------------

The following tests relate to animation or transformation of multicol properties.

- [column-width-interpolation.html](https://wpt.fyi/results/css/css-multicol/animation/column-width-interpolation.html) [(live test)](http://wpt.live/css/css-multicol/animation/column-width-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/animation/column-width-interpolation.html)
- [discrete-no-interpolation.html](https://wpt.fyi/results/css/css-multicol/animation/discrete-no-interpolation.html) [(live test)](http://wpt.live/css/css-multicol/animation/discrete-no-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/animation/discrete-no-interpolation.html)
- [multicol-overflow-positioned-transform-001.html](https://wpt.fyi/results/css/css-multicol/multicol-overflow-positioned-transform-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-overflow-positioned-transform-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-overflow-positioned-transform-001.html)
- [multicol-overflow-transform-001.html](https://wpt.fyi/results/css/css-multicol/multicol-overflow-transform-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-overflow-transform-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-overflow-transform-001.html)
- [resize-multicol-with-fixed-size-children.html](https://wpt.fyi/results/css/css-multicol/resize-multicol-with-fixed-size-children.html) [(live test)](http://wpt.live/css/css-multicol/resize-multicol-with-fixed-size-children.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/resize-multicol-with-fixed-size-children.html)
- [resize-in-strict-containment-nested.html](https://wpt.fyi/results/css/css-multicol/resize-in-strict-containment-nested.html) [(live test)](http://wpt.live/css/css-multicol/resize-in-strict-containment-nested.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/resize-in-strict-containment-nested.html)
- [remove-child-in-strict-containment-also-spanner.html](https://wpt.fyi/results/css/css-multicol/remove-child-in-strict-containment-also-spanner.html) [(live test)](http://wpt.live/css/css-multicol/remove-child-in-strict-containment-also-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/remove-child-in-strict-containment-also-spanner.html)
- [composited-under-clip-under-multicol.html](https://wpt.fyi/results/css/css-multicol/composited-under-clip-under-multicol.html) [(live test)](http://wpt.live/css/css-multicol/composited-under-clip-under-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/composited-under-clip-under-multicol.html)
- [change-intrinsic-width.html](https://wpt.fyi/results/css/css-multicol/change-intrinsic-width.html) [(live test)](http://wpt.live/css/css-multicol/change-intrinsic-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/change-intrinsic-width.html)
- [change-fragmentainer-size-000.html](https://wpt.fyi/results/css/css-multicol/change-fragmentainer-size-000.html) [(live test)](http://wpt.live/css/css-multicol/change-fragmentainer-size-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/change-fragmentainer-size-000.html)
- [change-fragmentainer-size-001.html](https://wpt.fyi/results/css/css-multicol/change-fragmentainer-size-001.html) [(live test)](http://wpt.live/css/css-multicol/change-fragmentainer-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/change-fragmentainer-size-001.html)
- [change-fragmentainer-size-002.html](https://wpt.fyi/results/css/css-multicol/change-fragmentainer-size-002.html) [(live test)](http://wpt.live/css/css-multicol/change-fragmentainer-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/change-fragmentainer-size-002.html)
- [change-fragmentainer-size-003.html](https://wpt.fyi/results/css/css-multicol/change-fragmentainer-size-003.html) [(live test)](http://wpt.live/css/css-multicol/change-fragmentainer-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/change-fragmentainer-size-003.html)
- [hit-test-child-under-perspective.html](https://wpt.fyi/results/css/css-multicol/hit-test-child-under-perspective.html) [(live test)](http://wpt.live/css/css-multicol/hit-test-child-under-perspective.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/hit-test-child-under-perspective.html)
- [hit-test-transformed-child.html](https://wpt.fyi/results/css/css-multicol/hit-test-transformed-child.html) [(live test)](http://wpt.live/css/css-multicol/hit-test-transformed-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/hit-test-transformed-child.html)

------------------------------------------------------------------------

Tests related to implementation bugs, not linked to specific normative text.

- [multicol-with-monolithic-oof-with-multicol-with-oof.html](https://wpt.fyi/results/css/css-multicol/crashtests/multicol-with-monolithic-oof-with-multicol-with-oof.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/multicol-with-monolithic-oof-with-multicol-with-oof.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/multicol-with-monolithic-oof-with-multicol-with-oof.html)
- [chrome-bug-1293905.html](https://wpt.fyi/results/css/css-multicol/crashtests/chrome-bug-1293905.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/chrome-bug-1293905.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/chrome-bug-1293905.html)
- [chrome-bug-1297118.html](https://wpt.fyi/results/css/css-multicol/crashtests/chrome-bug-1297118.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/chrome-bug-1297118.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/chrome-bug-1297118.html)
- [chrome-bug-1301281.html](https://wpt.fyi/results/css/css-multicol/crashtests/chrome-bug-1301281.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/chrome-bug-1301281.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/chrome-bug-1301281.html)
- [chrome-bug-1303256.html](https://wpt.fyi/results/css/css-multicol/crashtests/chrome-bug-1303256.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/chrome-bug-1303256.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/chrome-bug-1303256.html)
- [chrome-bug-1314866.html](https://wpt.fyi/results/css/css-multicol/crashtests/chrome-bug-1314866.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/chrome-bug-1314866.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/chrome-bug-1314866.html)
- [dynamic-simplified-layout-break-propagation.html](https://wpt.fyi/results/css/css-multicol/crashtests/dynamic-simplified-layout-break-propagation.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/dynamic-simplified-layout-break-propagation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/dynamic-simplified-layout-break-propagation.html)
- [float-multicol-crash.html](https://wpt.fyi/results/css/css-multicol/crashtests/float-multicol-crash.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/float-multicol-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/float-multicol-crash.html)
- [change-abspos-width-in-second-column-crash.html](https://wpt.fyi/results/css/css-multicol/change-abspos-width-in-second-column-crash.html) [(live test)](http://wpt.live/css/css-multicol/change-abspos-width-in-second-column-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/change-abspos-width-in-second-column-crash.html)
- [change-out-of-flow-type-and-remove-inner-multicol-crash.html](https://wpt.fyi/results/css/css-multicol/change-out-of-flow-type-and-remove-inner-multicol-crash.html) [(live test)](http://wpt.live/css/css-multicol/change-out-of-flow-type-and-remove-inner-multicol-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/change-out-of-flow-type-and-remove-inner-multicol-crash.html)
- [monolithic-oof-in-clipped-container.html](https://wpt.fyi/results/css/css-multicol/crashtests/monolithic-oof-in-clipped-container.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/monolithic-oof-in-clipped-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/monolithic-oof-in-clipped-container.html)
- [move-linebreak-to-different-column.html](https://wpt.fyi/results/css/css-multicol/crashtests/move-linebreak-to-different-column.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/move-linebreak-to-different-column.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/move-linebreak-to-different-column.html)
- [move-newline-pre-text.html](https://wpt.fyi/results/css/css-multicol/crashtests/move-newline-pre-text.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/move-newline-pre-text.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/move-newline-pre-text.html)
- [multicol-at-page-boundary-print.html](https://wpt.fyi/results/css/css-multicol/crashtests/multicol-at-page-boundary-print.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/multicol-at-page-boundary-print.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/multicol-at-page-boundary-print.html)
- [multicol-block-in-inline-crash.html](https://wpt.fyi/results/css/css-multicol/crashtests/multicol-block-in-inline-crash.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/multicol-block-in-inline-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/multicol-block-in-inline-crash.html)
- [multicol-cached-consumed-bsize-crash.html](https://wpt.fyi/results/css/css-multicol/crashtests/multicol-cached-consumed-bsize-crash.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/multicol-cached-consumed-bsize-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/multicol-cached-consumed-bsize-crash.html)
- [multicol-column-change-crash.html](https://wpt.fyi/results/css/css-multicol/crashtests/multicol-column-change-crash.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/multicol-column-change-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/multicol-column-change-crash.html)
- [multicol-dynamic-contain-crash.html](https://wpt.fyi/results/css/css-multicol/crashtests/multicol-dynamic-contain-crash.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/multicol-dynamic-contain-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/multicol-dynamic-contain-crash.html)
- [multicol-dynamic-transform-crash.html](https://wpt.fyi/results/css/css-multicol/crashtests/multicol-dynamic-transform-crash.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/multicol-dynamic-transform-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/multicol-dynamic-transform-crash.html)
- [multicol-floats-in-ifc.html](https://wpt.fyi/results/css/css-multicol/crashtests/multicol-floats-in-ifc.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/multicol-floats-in-ifc.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/multicol-floats-in-ifc.html)
- [multicol-parallel-flow-after-spanner-in-inline.html](https://wpt.fyi/results/css/css-multicol/crashtests/multicol-parallel-flow-after-spanner-in-inline.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/multicol-parallel-flow-after-spanner-in-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/multicol-parallel-flow-after-spanner-in-inline.html)
- [outline-move-oof-with-inline.html](https://wpt.fyi/results/css/css-multicol/crashtests/outline-move-oof-with-inline.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/outline-move-oof-with-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/outline-move-oof-with-inline.html)
- [relpos-inline-with-abspos-multicol-gets-block-child.html](https://wpt.fyi/results/css/css-multicol/crashtests/relpos-inline-with-abspos-multicol-gets-block-child.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/relpos-inline-with-abspos-multicol-gets-block-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/relpos-inline-with-abspos-multicol-gets-block-child.html)
- [size-containment-become-multicol-add-inline-child.html](https://wpt.fyi/results/css/css-multicol/crashtests/size-containment-become-multicol-add-inline-child.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/size-containment-become-multicol-add-inline-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/size-containment-become-multicol-add-inline-child.html)
- [sticky-in-abs-in-sticky.html](https://wpt.fyi/results/css/css-multicol/crashtests/sticky-in-abs-in-sticky.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/sticky-in-abs-in-sticky.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/sticky-in-abs-in-sticky.html)
- [vertical-rl-column-rules-wide-columns.html](https://wpt.fyi/results/css/css-multicol/crashtests/vertical-rl-column-rules-wide-columns.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/vertical-rl-column-rules-wide-columns.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/vertical-rl-column-rules-wide-columns.html)
- [dynamic-become-multicol-add-oof-inside-inline-crash.html](https://wpt.fyi/results/css/css-multicol/dynamic-become-multicol-add-oof-inside-inline-crash.html) [(live test)](http://wpt.live/css/css-multicol/dynamic-become-multicol-add-oof-inside-inline-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/dynamic-become-multicol-add-oof-inside-inline-crash.html)
- [extremely-tall-multicol-with-extremely-tall-child-crash.html](https://wpt.fyi/results/css/css-multicol/extremely-tall-multicol-with-extremely-tall-child-crash.html) [(live test)](http://wpt.live/css/css-multicol/extremely-tall-multicol-with-extremely-tall-child-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/extremely-tall-multicol-with-extremely-tall-child-crash.html)
- [file-control-crash.html](https://wpt.fyi/results/css/css-multicol/file-control-crash.html) [(live test)](http://wpt.live/css/css-multicol/file-control-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/file-control-crash.html)
- [img-alt-as-multicol-crash.html](https://wpt.fyi/results/css/css-multicol/img-alt-as-multicol-crash.html) [(live test)](http://wpt.live/css/css-multicol/img-alt-as-multicol-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/img-alt-as-multicol-crash.html)
- [overflow-scroll-in-multicol-crash.html](https://wpt.fyi/results/css/css-multicol/overflow-scroll-in-multicol-crash.html) [(live test)](http://wpt.live/css/css-multicol/overflow-scroll-in-multicol-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/overflow-scroll-in-multicol-crash.html)
- [remove-block-sibling-of-inline-with-block-crash.html](https://wpt.fyi/results/css/css-multicol/remove-block-sibling-of-inline-with-block-crash.html) [(live test)](http://wpt.live/css/css-multicol/remove-block-sibling-of-inline-with-block-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/remove-block-sibling-of-inline-with-block-crash.html)
- [subpixel-scroll-crash.html](https://wpt.fyi/results/css/css-multicol/subpixel-scroll-crash.html) [(live test)](http://wpt.live/css/css-multicol/subpixel-scroll-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/subpixel-scroll-crash.html)
- [text-child-crash.html](https://wpt.fyi/results/css/css-multicol/text-child-crash.html) [(live test)](http://wpt.live/css/css-multicol/text-child-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/text-child-crash.html)
- [with-custom-layout-on-same-element-crash.https.html](https://wpt.fyi/results/css/css-multicol/with-custom-layout-on-same-element-crash.https.html) [(live test)](https://wpt.live/css/css-multicol/with-custom-layout-on-same-element-crash.https.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/with-custom-layout-on-same-element-crash.https.html)
- [oof-in-area-001.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-in-area-001.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-in-area-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-in-area-001.html)
- [oof-in-area-002.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-in-area-002.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-in-area-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-in-area-002.html)
- [oof-in-area-003.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-in-area-003.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-in-area-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-in-area-003.html)
- [oof-in-oof-multicol-in-multicol-spanner-in-multicol.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-in-oof-multicol-in-multicol-spanner-in-multicol.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-in-oof-multicol-in-multicol-spanner-in-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-in-oof-multicol-in-multicol-spanner-in-multicol.html)
- [oof-in-oof-multicol-in-relpos-in-oof-in-multicol-in-multicol.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-in-oof-multicol-in-relpos-in-oof-in-multicol-in-multicol.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-in-oof-multicol-in-relpos-in-oof-in-multicol-in-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-in-oof-multicol-in-relpos-in-oof-in-multicol-in-multicol.html)
- [oof-in-oof-multicol-in-relpos-spanner-in-multicol-in-relpos-multicol-in-multicol.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-in-oof-multicol-in-relpos-spanner-in-multicol-in-relpos-multicol-in-multicol.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-in-oof-multicol-in-relpos-spanner-in-multicol-in-relpos-multicol-in-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-in-oof-multicol-in-relpos-spanner-in-multicol-in-relpos-multicol-in-multicol.html)
- [oof-in-oof-multicol-in-relpos-spanner-in-spanner-multicol-in-multicol-in-multicol.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-in-oof-multicol-in-relpos-spanner-in-spanner-multicol-in-multicol-in-multicol.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-in-oof-multicol-in-relpos-spanner-in-spanner-multicol-in-multicol-in-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-in-oof-multicol-in-relpos-spanner-in-spanner-multicol-in-multicol-in-multicol.html)
- [oof-in-oof-multicol-in-spanner-in-multicol-in-spanner-in-nested-multicol.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-in-oof-multicol-in-spanner-in-multicol-in-spanner-in-nested-multicol.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-in-oof-multicol-in-spanner-in-multicol-in-spanner-in-nested-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-in-oof-multicol-in-spanner-in-multicol-in-spanner-in-nested-multicol.html)
- [oof-in-oof-multicol-in-spanner-in-nested-multicol.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-in-oof-multicol-in-spanner-in-nested-multicol.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-in-oof-multicol-in-spanner-in-nested-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-in-oof-multicol-in-spanner-in-nested-multicol.html)
- [oof-in-relpos-in-oof-multicol-in-oof-in-relpos-in-oof-multicol-in-multicol.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-in-relpos-in-oof-multicol-in-oof-in-relpos-in-oof-multicol-in-multicol.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-in-relpos-in-oof-multicol-in-oof-in-relpos-in-oof-multicol-in-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-in-relpos-in-oof-multicol-in-oof-in-relpos-in-oof-multicol-in-multicol.html)
- [oof-in-relpos-in-oof-multicol-in-relpos-in-oof-multicol-in-relpos-multicol.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-in-relpos-in-oof-multicol-in-relpos-in-oof-multicol-in-relpos-multicol.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-in-relpos-in-oof-multicol-in-relpos-in-oof-multicol-in-relpos-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-in-relpos-in-oof-multicol-in-relpos-in-oof-multicol-in-relpos-multicol.html)
- [floated-input-in-inline-next-column.html](https://wpt.fyi/results/css/css-multicol/crashtests/floated-input-in-inline-next-column.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/floated-input-in-inline-next-column.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/floated-input-in-inline-next-column.html)
- [inline-float-parallel-flow.html](https://wpt.fyi/results/css/css-multicol/crashtests/inline-float-parallel-flow.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/inline-float-parallel-flow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/inline-float-parallel-flow.html)
- [table-caption-in-clipped-overflow.html](https://wpt.fyi/results/css/css-multicol/crashtests/table-caption-in-clipped-overflow.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/table-caption-in-clipped-overflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/table-caption-in-clipped-overflow.html)
- [text-in-inline-interrupted-by-float.html](https://wpt.fyi/results/css/css-multicol/crashtests/text-in-inline-interrupted-by-float.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/text-in-inline-interrupted-by-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/text-in-inline-interrupted-by-float.html)
- [increase-prev-sibling-height.html](https://wpt.fyi/results/css/css-multicol/increase-prev-sibling-height.html) [(live test)](http://wpt.live/css/css-multicol/increase-prev-sibling-height.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/increase-prev-sibling-height.html)
- [interleaved-bfc-crash.html](https://wpt.fyi/results/css/css-multicol/crashtests/interleaved-bfc-crash.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/interleaved-bfc-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/interleaved-bfc-crash.html)
- [relayout-fixedpos-in-abspos-in-relpos-in-nested-multicol.html](https://wpt.fyi/results/css/css-multicol/crashtests/relayout-fixedpos-in-abspos-in-relpos-in-nested-multicol.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/relayout-fixedpos-in-abspos-in-relpos-in-nested-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/relayout-fixedpos-in-abspos-in-relpos-in-nested-multicol.html)
- [inline-become-oof-container-make-oof-inflow.html](https://wpt.fyi/results/css/css-multicol/crashtests/inline-become-oof-container-make-oof-inflow.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/inline-become-oof-container-make-oof-inflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/inline-become-oof-container-make-oof-inflow.html)
- [multicol-dynamic-change-inside-break-inside-avoid-001.html](https://wpt.fyi/results/css/css-multicol/multicol-dynamic-change-inside-break-inside-avoid-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-dynamic-change-inside-break-inside-avoid-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-dynamic-change-inside-break-inside-avoid-001.html)
- [multicol-dynamic-add-001.html](https://wpt.fyi/results/css/css-multicol/multicol-dynamic-add-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-dynamic-add-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-dynamic-add-001.html)
- [scroll-width-height.tentative.html](https://wpt.fyi/results/css/css-multicol/scroll-width-height.tentative.html) [(live test)](http://wpt.live/css/css-multicol/scroll-width-height.tentative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/scroll-width-height.tentative.html)
- [filter-with-abspos.html](https://wpt.fyi/results/css/css-multicol/filter-with-abspos.html) [(live test)](http://wpt.live/css/css-multicol/filter-with-abspos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/filter-with-abspos.html)
- [add-list-item-marker.html](https://wpt.fyi/results/css/css-multicol/crashtests/add-list-item-marker.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/add-list-item-marker.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/add-list-item-marker.html)

------------------------------------------------------------------------

Tests related to printing and paged media as related to multicol.

- [auto-fill-auto-size-001-print.html](https://wpt.fyi/results/css/css-multicol/auto-fill-auto-size-001-print.html) [(live test)](http://wpt.live/css/css-multicol/auto-fill-auto-size-001-print.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/auto-fill-auto-size-001-print.html)
- [auto-fill-auto-size-002-print.html](https://wpt.fyi/results/css/css-multicol/auto-fill-auto-size-002-print.html) [(live test)](http://wpt.live/css/css-multicol/auto-fill-auto-size-002-print.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/auto-fill-auto-size-002-print.html)
- [multicol-height-002-print.xht](https://wpt.fyi/results/css/css-multicol/multicol-height-002-print.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-height-002-print.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-height-002-print.xht)
- [named-page.html](https://wpt.fyi/results/css/css-multicol/named-page.html) [(live test)](http://wpt.live/css/css-multicol/named-page.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/named-page.html)
- [page-property-ignored.html](https://wpt.fyi/results/css/css-multicol/page-property-ignored.html) [(live test)](http://wpt.live/css/css-multicol/page-property-ignored.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/page-property-ignored.html)

------------------------------------------------------------------------

<a id="ref-for-propdef-column-width①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In text set using a vertical writing mode, the block direction runs horizontally. In a vertical writing mode columns are laid out horizontally, and the direction of the flow of blocks may be right to left, or left to right. The [column-width](#propdef-column-width) property therefore refers to the inline size of the column, and not the physical horizontal width.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1ed00d6a"></a>
>
> ![The first image shows horizontal text with a LTR inline direction. The second shows vertical text with blocks flowing right to left. The third shows vertical text with blocks flowing left to right.](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/writing-modes.svg)
>
> A diagram showing the different ways columns may be arranged due to writing mode.  
> From left to right: horizontal-tb, vertical-rl, vertical-lr.

Tests

Tests regarding vertical writing modes.

- [orthogonal-writing-mode-shrink-to-fit.html](https://wpt.fyi/results/css/css-multicol/orthogonal-writing-mode-shrink-to-fit.html) [(live test)](http://wpt.live/css/css-multicol/orthogonal-writing-mode-shrink-to-fit.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/orthogonal-writing-mode-shrink-to-fit.html)
- [multicol-under-vertical-rl-scroll.html](https://wpt.fyi/results/css/css-multicol/multicol-under-vertical-rl-scroll.html) [(live test)](http://wpt.live/css/css-multicol/multicol-under-vertical-rl-scroll.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-under-vertical-rl-scroll.html)
- [hit-test-in-vertical-rl.html](https://wpt.fyi/results/css/css-multicol/hit-test-in-vertical-rl.html) [(live test)](http://wpt.live/css/css-multicol/hit-test-in-vertical-rl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/hit-test-in-vertical-rl.html)

------------------------------------------------------------------------

<a id="ref-for-multi-column-line"></a>

Within each [multicol line](#multi-column-line) in the multi-column container, adjacent column boxes are separated by a <a id="column-gap"></a>column gap, which may contain a <a id="column-rule"></a>column rule. All column gaps in the same multi-column container are equal. All column rules in the same multi-column container are also equal, if they appear; column rules only appear between columns that both have content.

<a id="ref-for-fragmentation"></a>

<a id="ref-for-multi-column-spanner"></a>

<a id="ref-for-multi-column-container⑧"></a>

<a id="ref-for-multi-column-line①"></a>

In the simplest case a multicol container will contain only one line of columns, and the height of each column will be equivalent to the used height of the multi-column container’s content box. However, [fragmentation](https://www.w3.org/TR/css-break-4/#fragmentation) or [spanners](#multi-column-spanner) can split the content of the [multi-column container](#multi-column-container) into multiple [multicol lines](#multi-column-line).

If the multi-column container is paginated, the height of each column is constrained by the page and the content continues in a new line of column boxes on the next page; a column box never splits across pages.

The same effect occurs when a <i>spanning element</i> divides the multi-column container: the columns before the spanning element are balanced and shortened to fit their content. Content after the spanning element then flows into a new, subsequent line of column boxes.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c2c3431f"></a>
>
> ![a diagram showing a spanning element causing the shortened columns above the element with text continuing in new columns below](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/simple-span-example.svg)
>
> A demonstration of how the spanning element divides the multicol container.

<a id="ref-for-multi-column-container⑨"></a>

<a id="ref-for-block-container"></a>

<a id="ref-for-independent-formatting-context"></a>

<a id="ref-for-multi-column-line②"></a>

<a id="ref-for-block-level-box"></a>

<a id="ref-for-column-box⑤"></a>

<a id="ref-for-multi-column-spanner①"></a>

<a id="ref-for-propdef-display"></a>

A [multi-column container](#multi-column-container) therefore is a regular [block container](https://www.w3.org/TR/css-display-3/#block-container) that establishes a new [independent formatting context](https://www.w3.org/TR/css-display-3/#independent-formatting-context) whose contents consist of a series of [multicol lines](#multi-column-line) and multicol spanners. Each <a id="ref-for-multi-column-line③"></a>multi-column line acts as a [block-level box](https://www.w3.org/TR/css-display-3/#block-level-box) that establishes a <a id="multi-column-formatting-context"></a>multi-column formatting context for its [column boxes](#column-box); and each [spanner](#multi-column-spanner) acts as a <a id="ref-for-block-level-box①"></a>block-level box that establishes an <a id="ref-for-independent-formatting-context①"></a>independent formatting context with its type depending on its [display](https://www.w3.org/TR/css-display-3/#propdef-display) value as usual.

Nested multi-column containers are allowed, but there may be implementation-specific limits.

Tests

- [multicol-nested-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-nested-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-002.xht)
- [multicol-nested-005.xht](https://wpt.fyi/results/css/css-multicol/multicol-nested-005.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-005.xht)
- [multicol-nested-006.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-006.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-006.html)
- [multicol-nested-007.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-007.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-007.html)
- [multicol-nested-008.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-008.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-008.html)
- [multicol-nested-009.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-009.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-009.html)
- [multicol-nested-010.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-010.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-010.html)
- [multicol-nested-011.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-011.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-011.html)
- [multicol-nested-012.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-012.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-012.html)
- [multicol-nested-013.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-013.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-013.html)
- [multicol-nested-014.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-014.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-014.html)
- [multicol-nested-015.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-015.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-015.html)
- [multicol-nested-016.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-016.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-016.html)
- [multicol-nested-017.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-017.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-017.html)
- [multicol-nested-018.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-018.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-018.html)
- [multicol-nested-019.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-019.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-019.html)
- [multicol-nested-020.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-020.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-020.html)
- [multicol-nested-021.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-021.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-021.html)
- [multicol-nested-022.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-022.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-022.html)
- [multicol-nested-023.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-023.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-023.html)
- [multicol-nested-024.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-024.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-024.html)
- [multicol-nested-025.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-025.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-025.html)
- [multicol-nested-026.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-026.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-026.html)
- [multicol-nested-027.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-027.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-027.html)
- [multicol-nested-028.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-028.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-028.html)
- [multicol-nested-029.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-029.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-029.html)
- [multicol-nested-030.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-030.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-030.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-030.html)
- [multicol-nested-031.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-031.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-031.html)
- [nested-as-balanced-legend.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-as-balanced-legend.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-as-balanced-legend.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-as-balanced-legend.html)
- [nested-as-nested-balanced-legend.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-as-nested-balanced-legend.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-as-nested-balanced-legend.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-as-nested-balanced-legend.html)
- [nested-floated-multicol-with-tall-margin.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-floated-multicol-with-tall-margin.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-floated-multicol-with-tall-margin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-floated-multicol-with-tall-margin.html)
- [nested-multicol-and-float-with-tall-padding-before-float.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-multicol-and-float-with-tall-padding-before-float.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-multicol-and-float-with-tall-padding-before-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-multicol-and-float-with-tall-padding-before-float.html)
- [nested-multicol-and-float-with-tall-padding.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-multicol-and-float-with-tall-padding.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-multicol-and-float-with-tall-padding.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-multicol-and-float-with-tall-padding.html)
- [nested-multicol-fieldset-tall-trailing-border-freeze.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-multicol-fieldset-tall-trailing-border-freeze.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-multicol-fieldset-tall-trailing-border-freeze.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-multicol-fieldset-tall-trailing-border-freeze.html)
- [nested-multicol-fieldset-tall-trailing-padding.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-multicol-fieldset-tall-trailing-padding.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-multicol-fieldset-tall-trailing-padding.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-multicol-fieldset-tall-trailing-padding.html)
- [nested-multicol-in-svg-foreignobject.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-multicol-in-svg-foreignobject.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-multicol-in-svg-foreignobject.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-multicol-in-svg-foreignobject.html)
- [nested-multicol-nested-flex.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-multicol-nested-flex.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-multicol-nested-flex.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-multicol-nested-flex.html)
- [nested-multicol-with-float-between.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-multicol-with-float-between.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-multicol-with-float-between.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-multicol-with-float-between.html)
- [nested-oof-multicol-with-monolithic-child.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-oof-multicol-with-monolithic-child.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-oof-multicol-with-monolithic-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-oof-multicol-with-monolithic-child.html)
- [nested-oof-multicol-with-oof-needing-additional-columns.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-oof-multicol-with-oof-needing-additional-columns.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-oof-multicol-with-oof-needing-additional-columns.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-oof-multicol-with-oof-needing-additional-columns.html)
- [nested-oof-multicol-with-padding.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-oof-multicol-with-padding.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-oof-multicol-with-padding.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-oof-multicol-with-padding.html)
- [nested-with-fragmented-oof-negative-top-offset.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-with-fragmented-oof-negative-top-offset.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-with-fragmented-oof-negative-top-offset.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-with-fragmented-oof-negative-top-offset.html)
- [nested-with-multicol-table-caption.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-with-multicol-table-caption.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-with-multicol-table-caption.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-with-multicol-table-caption.html)
- [multicol-table-caption-parallel-flow-after-spanner-in-inline.html](https://wpt.fyi/results/css/css-multicol/crashtests/multicol-table-caption-parallel-flow-after-spanner-in-inline.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/multicol-table-caption-parallel-flow-after-spanner-in-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/multicol-table-caption-parallel-flow-after-spanner-in-inline.html)
- [nested-with-multicol-table-cell.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-with-multicol-table-cell.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-with-multicol-table-cell.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-with-multicol-table-cell.html)
- [nested-with-oof-inside-fixed-width.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-with-oof-inside-fixed-width.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-with-oof-inside-fixed-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-with-oof-inside-fixed-width.html)
- [nested-with-percentage-size-and-oof.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-with-percentage-size-and-oof.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-with-percentage-size-and-oof.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-with-percentage-size-and-oof.html)
- [nested-with-tall-padding-and-oof.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-with-tall-padding-and-oof.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-with-tall-padding-and-oof.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-with-tall-padding-and-oof.html)
- [nested-with-tall-padding.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-with-tall-padding.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-with-tall-padding.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-with-tall-padding.html)
- [oof-in-nested-line-float.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-in-nested-line-float.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-in-nested-line-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-in-nested-line-float.html)
- [oof-nested-multicol-inside-oof.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-nested-multicol-inside-oof.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-nested-multicol-inside-oof.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-nested-multicol-inside-oof.html)
- [relayout-nested-with-oof.html](https://wpt.fyi/results/css/css-multicol/crashtests/relayout-nested-with-oof.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/relayout-nested-with-oof.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/relayout-nested-with-oof.html)
- [repeated-section-in-nested-table-nested-multicol.html](https://wpt.fyi/results/css/css-multicol/crashtests/repeated-section-in-nested-table-nested-multicol.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/repeated-section-in-nested-table-nested-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/repeated-section-in-nested-table-nested-multicol.html)
- [repeated-table-footer-in-caption-nested-multicol.html](https://wpt.fyi/results/css/css-multicol/crashtests/repeated-table-footer-in-caption-nested-multicol.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/repeated-table-footer-in-caption-nested-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/repeated-table-footer-in-caption-nested-multicol.html)
- [nested-balanced-monolithic-multicol-crash.html](https://wpt.fyi/results/css/css-multicol/nested-balanced-monolithic-multicol-crash.html) [(live test)](http://wpt.live/css/css-multicol/nested-balanced-monolithic-multicol-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-balanced-monolithic-multicol-crash.html)
- [nested-balanced-very-tall-content-crash.html](https://wpt.fyi/results/css/css-multicol/nested-balanced-very-tall-content-crash.html) [(live test)](http://wpt.live/css/css-multicol/nested-balanced-very-tall-content-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-balanced-very-tall-content-crash.html)
- [nested-floated-shape-outside-multicol-with-monolithic-child-crash.html](https://wpt.fyi/results/css/css-multicol/nested-floated-shape-outside-multicol-with-monolithic-child-crash.html) [(live test)](http://wpt.live/css/css-multicol/nested-floated-shape-outside-multicol-with-monolithic-child-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-floated-shape-outside-multicol-with-monolithic-child-crash.html)
- [nested-with-overflowing-padding-crash.html](https://wpt.fyi/results/css/css-multicol/nested-with-overflowing-padding-crash.html) [(live test)](http://wpt.live/css/css-multicol/nested-with-overflowing-padding-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-with-overflowing-padding-crash.html)
- [triply-nested-with-fixedpos-in-abspos-crash.html](https://wpt.fyi/results/css/css-multicol/triply-nested-with-fixedpos-in-abspos-crash.html) [(live test)](http://wpt.live/css/css-multicol/triply-nested-with-fixedpos-in-abspos-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/triply-nested-with-fixedpos-in-abspos-crash.html)
- [fixed-in-nested-multicol-with-transform-container.html](https://wpt.fyi/results/css/css-multicol/fixed-in-nested-multicol-with-transform-container.html) [(live test)](http://wpt.live/css/css-multicol/fixed-in-nested-multicol-with-transform-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/fixed-in-nested-multicol-with-transform-container.html)
- [fixed-in-nested-multicol-with-viewport-container.html](https://wpt.fyi/results/css/css-multicol/fixed-in-nested-multicol-with-viewport-container.html) [(live test)](http://wpt.live/css/css-multicol/fixed-in-nested-multicol-with-viewport-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/fixed-in-nested-multicol-with-viewport-container.html)
- [fixed-in-nested-multicol.html](https://wpt.fyi/results/css/css-multicol/fixed-in-nested-multicol.html) [(live test)](http://wpt.live/css/css-multicol/fixed-in-nested-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/fixed-in-nested-multicol.html)
- [nested-after-float-clearance.html](https://wpt.fyi/results/css/css-multicol/nested-after-float-clearance.html) [(live test)](http://wpt.live/css/css-multicol/nested-after-float-clearance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-after-float-clearance.html)
- [nested-at-outer-boundary-as-fieldset.html](https://wpt.fyi/results/css/css-multicol/nested-at-outer-boundary-as-fieldset.html) [(live test)](http://wpt.live/css/css-multicol/nested-at-outer-boundary-as-fieldset.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-at-outer-boundary-as-fieldset.html)
- [nested-at-outer-boundary-as-float.html](https://wpt.fyi/results/css/css-multicol/nested-at-outer-boundary-as-float.html) [(live test)](http://wpt.live/css/css-multicol/nested-at-outer-boundary-as-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-at-outer-boundary-as-float.html)
- [nested-at-outer-boundary-as-legend.html](https://wpt.fyi/results/css/css-multicol/nested-at-outer-boundary-as-legend.html) [(live test)](http://wpt.live/css/css-multicol/nested-at-outer-boundary-as-legend.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-at-outer-boundary-as-legend.html)
- [nested-floated-multicol-with-monolithic-child.html](https://wpt.fyi/results/css/css-multicol/nested-floated-multicol-with-monolithic-child.html) [(live test)](http://wpt.live/css/css-multicol/nested-floated-multicol-with-monolithic-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-floated-multicol-with-monolithic-child.html)
- [nested-oofs-in-relative-multicol.html](https://wpt.fyi/results/css/css-multicol/nested-oofs-in-relative-multicol.html) [(live test)](http://wpt.live/css/css-multicol/nested-oofs-in-relative-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-oofs-in-relative-multicol.html)
- [nested-past-fragmentation-line.html](https://wpt.fyi/results/css/css-multicol/nested-past-fragmentation-line.html) [(live test)](http://wpt.live/css/css-multicol/nested-past-fragmentation-line.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-past-fragmentation-line.html)
- [nested-with-too-tall-line.html](https://wpt.fyi/results/css/css-multicol/nested-with-too-tall-line.html) [(live test)](http://wpt.live/css/css-multicol/nested-with-too-tall-line.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-with-too-tall-line.html)
- [oof-nested-in-single-column.html](https://wpt.fyi/results/css/css-multicol/oof-nested-in-single-column.html) [(live test)](http://wpt.live/css/css-multicol/oof-nested-in-single-column.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/oof-nested-in-single-column.html)
- [nested-non-auto-inline-size-offset-top.html](https://wpt.fyi/results/css/css-multicol/nested-non-auto-inline-size-offset-top.html) [(live test)](http://wpt.live/css/css-multicol/nested-non-auto-inline-size-offset-top.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-non-auto-inline-size-offset-top.html)
- [fixedpos-static-pos-with-viewport-cb-001.html](https://wpt.fyi/results/css/css-multicol/fixedpos-static-pos-with-viewport-cb-001.html) [(live test)](http://wpt.live/css/css-multicol/fixedpos-static-pos-with-viewport-cb-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/fixedpos-static-pos-with-viewport-cb-001.html)
- [fixedpos-static-pos-with-viewport-cb-002.html](https://wpt.fyi/results/css/css-multicol/fixedpos-static-pos-with-viewport-cb-002.html) [(live test)](http://wpt.live/css/css-multicol/fixedpos-static-pos-with-viewport-cb-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/fixedpos-static-pos-with-viewport-cb-002.html)
- [fixedpos-static-pos-with-viewport-cb-003.html](https://wpt.fyi/results/css/css-multicol/fixedpos-static-pos-with-viewport-cb-003.html) [(live test)](http://wpt.live/css/css-multicol/fixedpos-static-pos-with-viewport-cb-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/fixedpos-static-pos-with-viewport-cb-003.html)
- [multicol-height-block-child-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-height-block-child-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-height-block-child-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-height-block-child-001.xht)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is not possible to set properties/values on column boxes. For example, the background of a certain column box cannot be set and a column box has no concept of padding, margin or borders. Future specifications may add additional functionality. For example, columns of different widths and different backgrounds may be supported.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Multicol containers with column heights larger than the viewport may pose accessibility issues. See [Accessibility Considerations](#a11y) for more details.

## <a id="the-number-and-width-of-columns"></a>3.  The Number and Width of Columns

Finding the number and width of columns is fundamental when laying out multi-column content. These properties are used to set the number and width of columns:

- <a id="ref-for-propdef-column-count①"></a>

  [column-count](#propdef-column-count)

- <a id="ref-for-propdef-column-width②"></a>

  [column-width](#propdef-column-width)

<a id="ref-for-propdef-columns①"></a>

<a id="ref-for-propdef-column-width③"></a>

<a id="ref-for-propdef-column-count②"></a>

A third property, [columns](#propdef-columns), is a shorthand property which sets both [column-width](#propdef-column-width) and [column-count](#propdef-column-count).

Other factors, such as explicit column breaks, content, and height constraints, may influence the actual number and width of columns.

<a id="ref-for-propdef-column-width④"></a>

### <a id="cw"></a>3.1. The Inline Size of Columns: the [column-width](#propdef-column-width) property

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-column-width"></a>column-width

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-length-value"></a>

<a id="ref-for-comb-one"></a>

auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-table-wrapper-box"></a>

<a id="ref-for-block-container①"></a>

[block containers](https://www.w3.org/TR/css-display-3/#block-container) except [table wrapper boxes](https://www.w3.org/TR/css-tables-3/#table-wrapper-box)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-column-width-auto①"></a>

the keyword [auto](#valdef-column-width-auto) or an absolute length

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

This property describes the width of columns in multicol containers.

<a id="valdef-column-width-auto"></a>auto

<a id="ref-for-propdef-column-count③"></a>

means that the column width will be determined by other properties (e.g., [column-count](#propdef-column-count), if it has a non-auto value).

<a id="ref-for-length-value①"></a>

<a id="valdef-column-width-length-0"></a>[\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value)

describes the optimal column width. The actual column width may be wider (to fill the available space), or narrower (only if the available space is smaller than the specified column width). Negative values are not allowed. Used values will be clamped to a minimum of 1px.

Tests

- [zero-column-width-computed-style.html](https://wpt.fyi/results/css/css-multicol/zero-column-width-computed-style.html) [(live test)](http://wpt.live/css/css-multicol/zero-column-width-computed-style.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/zero-column-width-computed-style.html)
- [zero-column-width-layout.html](https://wpt.fyi/results/css/css-multicol/zero-column-width-layout.html) [(live test)](http://wpt.live/css/css-multicol/zero-column-width-layout.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/zero-column-width-layout.html)

Tests

- [multicol-basic-003.html](https://wpt.fyi/results/css/css-multicol/multicol-basic-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-basic-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-basic-003.html)
- [multicol-basic-008.xht](https://wpt.fyi/results/css/css-multicol/multicol-basic-008.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-basic-008.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-basic-008.xht)
- [multicol-reduce-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-reduce-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-reduce-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-reduce-000.xht)
- [multicol-width-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-width-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-width-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-width-001.xht)
- [multicol-width-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-width-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-width-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-width-002.xht)
- [multicol-width-003.xht](https://wpt.fyi/results/css/css-multicol/multicol-width-003.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-width-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-width-003.xht)
- [multicol-width-005.html](https://wpt.fyi/results/css/css-multicol/multicol-width-005.html) [(live test)](http://wpt.live/css/css-multicol/multicol-width-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-width-005.html)
- [multicol-width-ch-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-width-ch-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-width-ch-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-width-ch-001.xht)
- [multicol-width-negative-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-width-negative-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-width-negative-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-width-negative-001.xht)
- [multicol-width-invalid-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-width-invalid-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-width-invalid-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-width-invalid-001.xht)
- [multicol-width-large-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-width-large-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-width-large-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-width-large-001.xht)
- [multicol-width-large-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-width-large-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-width-large-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-width-large-002.xht)
- [multicol-inherit-003.xht](https://wpt.fyi/results/css/css-multicol/multicol-inherit-003.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-inherit-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-inherit-003.xht)
- [column-width-computed.html](https://wpt.fyi/results/css/css-multicol/parsing/column-width-computed.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-width-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-width-computed.html)
- [column-width-invalid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-width-invalid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-width-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-width-invalid.html)
- [column-width-valid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-width-valid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-width-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-width-valid.html)
- [multicol-width-small-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-width-small-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-width-small-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-width-small-001.xht)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f5259b18"></a> For example, consider this style sheet:
>
> ```text
> div {
>   width: 100px;
>   column-width: 45px;
>   column-gap: 0;
>   column-rule: none;
> }
> ```
>
> There is room for two 45px wide columns inside the 100px wide element. In order to fill the available space the actual column width will be increased to 50px.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ea5c2499"></a> Also, consider this style sheet:
>
> ```text
> div {
>   width: 40px;
>   column-width: 45px;
>   column-gap: 0;
>   column-rule: none;
> }
> ```
>
> The available space is smaller than the specified column width and the actual column width will therefore be decreased.

<a id="ref-for-propdef-column-width⑤"></a>

To ensure that [column-width](#propdef-column-width) can be used with vertical text, column width means the length of the line boxes inside the columns.

<a id="ref-for-propdef-column-width⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The reason for making [column-width](#propdef-column-width) somewhat flexible is to achieve scalable designs that can fit many screen sizes. To set an exact column width, the column gap and the width of the multicol container (assuming horizontal text) must also be specified.

<a id="ref-for-propdef-column-count④"></a>

### <a id="cc"></a>3.2. The Number of Columns: the [column-count](#propdef-column-count) property

<strong>Table 2 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-column-count"></a>column-count

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-integer-value"></a>

<a id="ref-for-comb-one①"></a>

auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<integer \[1,∞\]\>](https://www.w3.org/TR/css-values-4/#integer-value)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-table-wrapper-box①"></a>

<a id="ref-for-block-container②"></a>

[block containers](https://www.w3.org/TR/css-display-3/#block-container) except [table wrapper boxes](https://www.w3.org/TR/css-tables-3/#table-wrapper-box)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified value

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value

<a id="ref-for-multi-column-container①⓪"></a>

This property describes the number of columns of a [multicol container](#multi-column-container).

<a id="valdef-column-count-auto"></a>auto

<a id="ref-for-propdef-column-width⑦"></a>

means that the number of columns will be determined by other properties (e.g., [column-width](#propdef-column-width), if it has a non-auto value).

<a id="ref-for-integer-value①"></a>

<a id="valdef-column-count-integer-1"></a>[\<integer \[1,∞\]\>](https://www.w3.org/TR/css-values-4/#integer-value)

<a id="ref-for-propdef-column-count⑤"></a>

<a id="ref-for-propdef-column-width⑧"></a>

describes the optimal number of columns into which the content of the element will be flowed. Values must be greater than 0. If both [column-width](#propdef-column-width) and [column-count](#propdef-column-count) have non-auto values, the integer value describes the maximum number of columns.

Tests

- [multicol-count-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-count-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-count-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-count-001.xht)
- [multicol-count-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-count-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-count-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-count-002.xht)
- [multicol-basic-006.xht](https://wpt.fyi/results/css/css-multicol/multicol-basic-006.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-basic-006.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-basic-006.xht)
- [multicol-width-count-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-width-count-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-width-count-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-width-count-001.xht)
- [multicol-width-count-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-width-count-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-width-count-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-width-count-002.xht)
- [multicol-columns-toolong-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-columns-toolong-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-columns-toolong-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-columns-toolong-001.xht)
- [multicol-count-negative-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-count-negative-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-count-negative-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-count-negative-001.xht)
- [multicol-count-negative-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-count-negative-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-count-negative-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-count-negative-002.xht)
- [multicol-count-non-integer-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-count-non-integer-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-count-non-integer-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-count-non-integer-001.xht)
- [multicol-count-non-integer-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-count-non-integer-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-count-non-integer-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-count-non-integer-002.xht)
- [multicol-count-non-integer-003.xht](https://wpt.fyi/results/css/css-multicol/multicol-count-non-integer-003.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-count-non-integer-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-count-non-integer-003.xht)
- [multicol-inherit-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-inherit-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-inherit-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-inherit-001.xht)
- [multicol-inherit-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-inherit-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-inherit-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-inherit-002.xht)
- [column-count-invalid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-count-invalid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-count-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-count-invalid.html)
- [column-count-valid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-count-valid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-count-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-count-valid.html)
- [column-count-computed.html](https://wpt.fyi/results/css/css-multicol/parsing/column-count-computed.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-count-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-count-computed.html)
- [column-count-interpolation.html](https://wpt.fyi/results/css/css-multicol/animation/column-count-interpolation.html) [(live test)](http://wpt.live/css/css-multicol/animation/column-count-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/animation/column-count-interpolation.html)
- [large-actual-column-count.html](https://wpt.fyi/results/css/css-multicol/large-actual-column-count.html) [(live test)](http://wpt.live/css/css-multicol/large-actual-column-count.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/large-actual-column-count.html)
- [with-custom-layout-on-same-element.https.html](https://wpt.fyi/results/css/css-multicol/with-custom-layout-on-same-element.https.html) [(live test)](https://wpt.live/css/css-multicol/with-custom-layout-on-same-element.https.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/with-custom-layout-on-same-element.https.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6acd2708"></a> Example:
>
> ```text
> body { column-count: 3 }
> ```
<a id="ref-for-propdef-column-width⑨"></a>

<a id="ref-for-propdef-column-count⑥"></a>

<a id="ref-for-propdef-columns②"></a>

### <a id="columns"></a>3.3. The [column-width](#propdef-column-width) and [column-count](#propdef-column-count) Shorthand: The [columns](#propdef-columns) Property

<strong>Table 3 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-columns"></a>columns

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-column-count⑦"></a>

<a id="ref-for-comb-any"></a>

<a id="ref-for-propdef-column-width①⓪"></a>

[\<'column-width'\>](#propdef-column-width) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'column-count'\>](#propdef-column-count)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<a id="ref-for-propdef-column-width①①"></a>

<a id="ref-for-propdef-column-count⑧"></a>

This is a shorthand property for setting [column-width](#propdef-column-width) and [column-count](#propdef-column-count). Omitted values are set to their initial values.

<a id="ref-for-propdef-columns③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-628937c1"></a> Here are some valid declarations using the [columns](#propdef-columns) property:
>
> ```text
> columns: 12em;      /* column-width: 12em; column-count: auto */
> columns: auto 12em; /* column-width: 12em; column-count: auto */
> columns: 2;         /* column-width: auto; column-count: 2 */
> columns: 2 auto;    /* column-width: auto; column-count: 2 */
> columns: auto;      /* column-width: auto; column-count: auto */
> columns: auto auto; /* column-width: auto; column-count: auto */
> ```
Tests

- [multicol-columns-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-columns-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-columns-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-columns-001.xht)
- [multicol-columns-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-columns-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-columns-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-columns-002.xht)
- [multicol-columns-003.xht](https://wpt.fyi/results/css/css-multicol/multicol-columns-003.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-columns-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-columns-003.xht)
- [multicol-columns-004.xht](https://wpt.fyi/results/css/css-multicol/multicol-columns-004.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-columns-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-columns-004.xht)
- [multicol-columns-005.xht](https://wpt.fyi/results/css/css-multicol/multicol-columns-005.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-columns-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-columns-005.xht)
- [multicol-columns-006.xht](https://wpt.fyi/results/css/css-multicol/multicol-columns-006.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-columns-006.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-columns-006.xht)
- [multicol-columns-007.xht](https://wpt.fyi/results/css/css-multicol/multicol-columns-007.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-columns-007.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-columns-007.xht)
- [multicol-columns-invalid-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-columns-invalid-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-columns-invalid-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-columns-invalid-001.xht)
- [multicol-columns-invalid-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-columns-invalid-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-columns-invalid-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-columns-invalid-002.xht)
- [multicol-basic-001.html](https://wpt.fyi/results/css/css-multicol/multicol-basic-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-basic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-basic-001.html)
- [multicol-basic-003.html](https://wpt.fyi/results/css/css-multicol/multicol-basic-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-basic-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-basic-003.html)
- [multicol-basic-005.xht](https://wpt.fyi/results/css/css-multicol/multicol-basic-005.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-basic-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-basic-005.xht)
- [multicol-basic-007.xht](https://wpt.fyi/results/css/css-multicol/multicol-basic-007.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-basic-007.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-basic-007.xht)
- [columns-invalid.html](https://wpt.fyi/results/css/css-multicol/parsing/columns-invalid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/columns-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/columns-invalid.html)
- [columns-valid.html](https://wpt.fyi/results/css/css-multicol/parsing/columns-valid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/columns-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/columns-valid.html)

### <a id="pseudo-algorithm"></a>3.4. The Pseudo-algorithm

<a id="ref-for-propdef-column-count⑨"></a>

<a id="ref-for-propdef-column-width①②"></a>

The pseudo-algorithm below determines the used values for [column-count](#propdef-column-count) (N) and [column-width](#propdef-column-width) (W). There is one other variable in the pseudo-algorithm: U is the used width of the multi-column container.

<a id="ref-for-propdef-column-count①⓪"></a>

<a id="ref-for-propdef-column-width①③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The used width U of the multi-column container can depend on the element’s contents, in which case it also depends on the computed values of the [column-count](#propdef-column-count) and [column-width](#propdef-column-width) properties. This specification does not define how U is calculated. Another module (probably the Basic Box Model [\[CSS3BOX\]](#biblio-css3box) or the Box Sizing Module [\[CSS3-SIZING\]](#biblio-css3-sizing)) is expected to define this.

Tests

- [intrinsic-size-001.html](https://wpt.fyi/results/css/css-multicol/intrinsic-size-001.html) [(live test)](http://wpt.live/css/css-multicol/intrinsic-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/intrinsic-size-001.html)
- [intrinsic-size-002.html](https://wpt.fyi/results/css/css-multicol/intrinsic-size-002.html) [(live test)](http://wpt.live/css/css-multicol/intrinsic-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/intrinsic-size-002.html)
- [intrinsic-size-003.html](https://wpt.fyi/results/css/css-multicol/intrinsic-size-003.html) [(live test)](http://wpt.live/css/css-multicol/intrinsic-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/intrinsic-size-003.html)
- [intrinsic-size-004.html](https://wpt.fyi/results/css/css-multicol/intrinsic-size-004.html) [(live test)](http://wpt.live/css/css-multicol/intrinsic-size-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/intrinsic-size-004.html)
- [intrinsic-size-005.html](https://wpt.fyi/results/css/css-multicol/intrinsic-size-005.html) [(live test)](http://wpt.live/css/css-multicol/intrinsic-size-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/intrinsic-size-005.html)
- [as-column-flex-item.html](https://wpt.fyi/results/css/css-multicol/as-column-flex-item.html) [(live test)](http://wpt.live/css/css-multicol/as-column-flex-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/as-column-flex-item.html)
- [intrinsic-width-change-column-count.html](https://wpt.fyi/results/css/css-multicol/intrinsic-width-change-column-count.html) [(live test)](http://wpt.live/css/css-multicol/intrinsic-width-change-column-count.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/intrinsic-width-change-column-count.html)

The `floor(X)` function returns the largest integer Y ≤ X.

```text
(01)  if ((column-width = auto) and (column-count = auto)) then
(02)      exit; /* not a multicol container */
(03)  if column-width = auto then
(04)      N := column-count
(05)  else if column-count = auto then
(06)      N := max(1,
(07)        floor((U + column-gap)/(column-width + column-gap)))
(08)  else
(09)      N := min(column-count, max(1,
(10)        floor((U + column-gap)/(column-width + column-gap))))
```
And:

```text
(11)  W := max(0, ((U + column-gap)/N - column-gap))
```
For the purpose of finding the number of auto-repeated columns, the UA must floor the column size to a UA-specified value to avoid division by zero. It is suggested that this floor be 1px or less.

<a id="ref-for-paged-media"></a>

In fragmented contexts such as in [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media), user agents may perform this calculation on a per-fragment basis.

<a id="ref-for-propdef-column-count①①"></a>

The used value for [column-count](#propdef-column-count) is calculated without regard for explicit column breaks or constrained column heights, while the actual value takes these into consideration.

Tests

- [column-count-used-001.html](https://wpt.fyi/results/css/css-multicol/column-count-used-001.html) [(live test)](http://wpt.live/css/css-multicol/column-count-used-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/column-count-used-001.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-855fb04e"></a> In this example, the actual column-count is higher than the used column-count due to explicit column breaks:
>
> ```text
> div {
>   width: 40em;
>   columns: 20em;
>   column-gap: 0;
> }
> 
> p {
>   break-after: column;
> }
> ```
>
> ```text
> <div>
>   <p>one
>   <p>two
>   <p>three
> </div>
> ```
>
> ![Two columns drawn inside the container, one outside](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/column-count-higher-than-used-count.svg)
>
> The computed column-count is auto, the used column-count is 2 and the actual column-count is 3.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-69e715d6"></a> The actual column-count may be lower than the used column-count. Consider this example:
>
> ```text
> div {
>   width: 80em;
>   height: 10em;
>   columns: 20em;
>   column-gap: 0;
>   column-fill: auto;
> }
> ```
>
> ```text
> <div>foo</div>
> ```
>
> The computed column-count is auto, the used column-count is 4, and the actual column-count is 1.

### <a id="stacking-context"></a>3.5.  Stacking Context

All column boxes in a multi-column container are in the same stacking context and the drawing order of their contents is as specified in CSS 2.1. Column boxes do not establish new stacking contexts.

Tests

- [multicol-rule-stacking-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-stacking-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-stacking-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-stacking-001.xht)

## <a id="column-gaps-and-rules"></a>4. Column Gaps and Rules

<a id="ref-for-multi-column-container①①"></a>

Column gaps and rules are placed between columns in the same [multicol container](#multi-column-container). The length of the column gaps and column rules is equal to the column height. Column gaps take up space. That is, column gaps will push apart content in adjacent columns (within the same <a id="ref-for-multi-column-container①②"></a>multicol container).

Tests

- [multicol-height-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-height-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-height-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-height-001.xht)
- [multicol-nested-column-rule-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-nested-column-rule-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-column-rule-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-column-rule-001.xht)
- [multicol-nested-column-rule-002.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-column-rule-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-column-rule-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-column-rule-002.html)
- [multicol-nested-column-rule-003.html](https://wpt.fyi/results/css/css-multicol/multicol-nested-column-rule-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-nested-column-rule-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-nested-column-rule-003.html)
- [multicol-rule-nested-balancing-001.html](https://wpt.fyi/results/css/css-multicol/multicol-rule-nested-balancing-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-nested-balancing-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-nested-balancing-001.html)
- [multicol-rule-nested-balancing-002.html](https://wpt.fyi/results/css/css-multicol/multicol-rule-nested-balancing-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-nested-balancing-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-nested-balancing-002.html)
- [multicol-rule-nested-balancing-003.html](https://wpt.fyi/results/css/css-multicol/multicol-rule-nested-balancing-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-nested-balancing-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-nested-balancing-003.html)
- [multicol-rule-nested-balancing-004.html](https://wpt.fyi/results/css/css-multicol/multicol-rule-nested-balancing-004.html) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-nested-balancing-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-nested-balancing-004.html)

<a id="ref-for-column-rule①"></a>

<a id="ref-for-column-gap①"></a>

<a id="ref-for-multi-column-container①③"></a>

A [column rule](#column-rule) is drawn in the middle of the [column gap](#column-gap) with the endpoints at opposing content edges of the [multicol container](#multi-column-container). Column rules do not take up space. That is, the presence or thickness of a <a id="ref-for-column-rule②"></a>column rule will not alter the placement of anything else. If a <a id="ref-for-column-rule③"></a>column rule is wider than its gap, the adjacent column boxes will overlap the rule, and the rule may possibly extend outside the box of the <a id="ref-for-multi-column-container①④"></a>multicol container. Column rules are painted just above the border of the <a id="ref-for-multi-column-container①⑤"></a>multicol container. For scrollable multicol containers, note that while the border and background of the <a id="ref-for-multi-column-container①⑥"></a>multicol container obviously aren’t scrolled, the rules need to scroll along with the columns. Column rules are only drawn between two columns that both have content.

Tests

Basic column rule tests

- [multicol-rule-003.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-003.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-003.xht)
- [multicol-rule-004.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-004.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-004.xht)
- [multicol-rule-fraction-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-fraction-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-fraction-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-fraction-002.xht)

------------------------------------------------------------------------

If a column rule is wider than its gap, adjacent boxes overlap.

- [multicol-rule-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-001.xht)
- [multicol-rule-large-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-large-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-large-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-large-001.xht)
- multicol-rule-large-002.xht (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-large-002.xht)

------------------------------------------------------------------------

Column rules are only drawn between two columns that have content.

- [multicol-count-computed-003.xht](https://wpt.fyi/results/css/css-multicol/multicol-count-computed-003.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-count-computed-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-count-computed-003.xht)
- [multicol-count-computed-005.xht](https://wpt.fyi/results/css/css-multicol/multicol-count-computed-005.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-count-computed-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-count-computed-005.xht)
- [broken-column-rule-1.html](https://wpt.fyi/results/css/css-multicol/broken-column-rule-1.html) [(live test)](http://wpt.live/css/css-multicol/broken-column-rule-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/broken-column-rule-1.html)

------------------------------------------------------------------------

Tests for behavior of backgrounds and column rules.

- [multicol-breaking-000.html](https://wpt.fyi/results/css/css-multicol/multicol-breaking-000.html) [(live test)](http://wpt.live/css/css-multicol/multicol-breaking-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-breaking-000.html)
- [multicol-breaking-001.html](https://wpt.fyi/results/css/css-multicol/multicol-breaking-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-breaking-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-breaking-001.html)
- [multicol-breaking-002.html](https://wpt.fyi/results/css/css-multicol/multicol-breaking-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-breaking-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-breaking-002.html)
- [multicol-breaking-003.html](https://wpt.fyi/results/css/css-multicol/multicol-breaking-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-breaking-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-breaking-003.html)
- [multicol-breaking-004.html](https://wpt.fyi/results/css/css-multicol/multicol-breaking-004.html) [(live test)](http://wpt.live/css/css-multicol/multicol-breaking-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-breaking-004.html)
- [multicol-breaking-005.html](https://wpt.fyi/results/css/css-multicol/multicol-breaking-005.html) [(live test)](http://wpt.live/css/css-multicol/multicol-breaking-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-breaking-005.html)
- [multicol-breaking-006.html](https://wpt.fyi/results/css/css-multicol/multicol-breaking-006.html) [(live test)](http://wpt.live/css/css-multicol/multicol-breaking-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-breaking-006.html)
- [multicol-breaking-nobackground-000.html](https://wpt.fyi/results/css/css-multicol/multicol-breaking-nobackground-000.html) [(live test)](http://wpt.live/css/css-multicol/multicol-breaking-nobackground-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-breaking-nobackground-000.html)
- [multicol-breaking-nobackground-001.html](https://wpt.fyi/results/css/css-multicol/multicol-breaking-nobackground-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-breaking-nobackground-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-breaking-nobackground-001.html)
- [multicol-breaking-nobackground-002.html](https://wpt.fyi/results/css/css-multicol/multicol-breaking-nobackground-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-breaking-nobackground-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-breaking-nobackground-002.html)
- [multicol-breaking-nobackground-003.html](https://wpt.fyi/results/css/css-multicol/multicol-breaking-nobackground-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-breaking-nobackground-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-breaking-nobackground-003.html)
- [multicol-breaking-nobackground-004.html](https://wpt.fyi/results/css/css-multicol/multicol-breaking-nobackground-004.html) [(live test)](http://wpt.live/css/css-multicol/multicol-breaking-nobackground-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-breaking-nobackground-004.html)
- [multicol-breaking-nobackground-005.html](https://wpt.fyi/results/css/css-multicol/multicol-breaking-nobackground-005.html) [(live test)](http://wpt.live/css/css-multicol/multicol-breaking-nobackground-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-breaking-nobackground-005.html)

------------------------------------------------------------------------

<a id="ref-for-propdef-column-gap"></a>

### <a id="cg"></a>4.1. Gutters Between Columns: the [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap) property

<a id="ref-for-propdef-column-gap①"></a>

The [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap) property is defined in [\[CSS3-ALIGN\]](#biblio-css3-align).

<a id="ref-for-multi-column-formatting-context"></a>

<a id="ref-for-valdef-row-gap-normal"></a>

<a id="ref-for-propdef-column-gap②"></a>

In a [multi-column formatting context](#multi-column-formatting-context) the used value of [normal](https://www.w3.org/TR/css-align-3/#valdef-row-gap-normal) for the [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap) property is 1em. This ensures columns are readable when the initial values are used. If there is a column rule between columns, it will appear in the middle of the gap.

Tests

- [multicol-gap-fraction-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-gap-fraction-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-gap-fraction-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-gap-fraction-001.xht)
- [multicol-gap-fraction-002.html](https://wpt.fyi/results/css/css-multicol/multicol-gap-fraction-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-gap-fraction-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-gap-fraction-002.html)
- [multicol-gap-large-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-gap-large-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-gap-large-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-gap-large-001.xht)
- [multicol-gap-large-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-gap-large-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-gap-large-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-gap-large-002.xht)
- [multicol-gap-negative-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-gap-negative-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-gap-negative-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-gap-negative-001.xht)
- [multicol-gap-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-gap-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-gap-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-gap-000.xht)
- [multicol-gap-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-gap-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-gap-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-gap-002.xht)
- [multicol-gap-percentage-001.html](https://wpt.fyi/results/css/css-multicol/multicol-gap-percentage-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-gap-percentage-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-gap-percentage-001.html)
- [multicol-gap-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-gap-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-gap-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-gap-001.xht)
- [multicol-gap-003.xht](https://wpt.fyi/results/css/css-multicol/multicol-gap-003.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-gap-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-gap-003.xht)

Tests that the gap is animatable.

- [multicol-gap-animation-001.html](https://wpt.fyi/results/css/css-multicol/multicol-gap-animation-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-gap-animation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-gap-animation-001.html)
- [multicol-gap-animation-002.html](https://wpt.fyi/results/css/css-multicol/multicol-gap-animation-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-gap-animation-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-gap-animation-002.html)
- [multicol-gap-animation-003.html](https://wpt.fyi/results/css/css-multicol/multicol-gap-animation-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-gap-animation-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-gap-animation-003.html)

------------------------------------------------------------------------

<a id="ref-for-propdef-column-rule-color"></a>

### <a id="crc"></a>4.2. The Color of Column Rules: the [column-rule-color](#propdef-column-rule-color) property

<strong>Table 4 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-column-rule-color"></a>column-rule-color

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-color"></a>

[\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

currentcolor

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

multicol containers

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

computed color

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

<a id="ref-for-typedef-color①"></a>

<a id="valdef-column-rule-color-color"></a>[\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color)

<a id="ref-for-column-rule④"></a>

Specifies the color of the [column rule](#column-rule).

Tests

- [multicol-rule-color-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-color-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-color-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-color-001.xht)
- [multicol-rule-color-inherit-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-color-inherit-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-color-inherit-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-color-inherit-001.xht)
- [multicol-rule-color-inherit-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-color-inherit-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-color-inherit-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-color-inherit-002.xht)
- [column-rule-color-computed.html](https://wpt.fyi/results/css/css-multicol/parsing/column-rule-color-computed.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-rule-color-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-rule-color-computed.html)
- [column-rule-color-valid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-rule-color-valid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-rule-color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-rule-color-valid.html)
- [column-rule-color-invalid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-rule-color-invalid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-rule-color-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-rule-color-invalid.html)
- [column-rule-color-interpolation.html](https://wpt.fyi/results/css/css-multicol/animation/column-rule-color-interpolation.html) [(live test)](http://wpt.live/css/css-multicol/animation/column-rule-color-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/animation/column-rule-color-interpolation.html)

<a id="ref-for-propdef-column-rule-style"></a>

### <a id="crs"></a>4.3. The Style Of Column Rules: the [column-rule-style](#propdef-column-rule-style) property

<strong>Table 5 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-column-rule-style"></a>column-rule-style

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-line-style"></a>

[\<line-style\>](https://www.w3.org/TR/css-backgrounds-3/#typedef-line-style)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

multicol containers

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified keyword

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

<a id="ref-for-propdef-column-rule-style①"></a>

<a id="ref-for-typedef-line-style①"></a>

The [column-rule-style](#propdef-column-rule-style) property sets the style of the rule between columns of an element. The [\<line-style\>](https://www.w3.org/TR/css-backgrounds-3/#typedef-line-style) values are interpreted as in the [collapsing border model](https://www.w3.org/TR/CSS2/tables.html#collapsing-borders).

Tests

- [column-rule-style-computed.html](https://wpt.fyi/results/css/css-multicol/parsing/column-rule-style-computed.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-rule-style-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-rule-style-computed.html)
- [column-rule-style-valid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-rule-style-valid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-rule-style-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-rule-style-valid.html)
- [column-rule-style-invalid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-rule-style-invalid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-rule-style-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-rule-style-invalid.html)

<a id="ref-for-valdef-line-style-none"></a>

<a id="ref-for-propdef-column-rule-width"></a>

The [none](https://www.w3.org/TR/css-backgrounds-3/#valdef-line-style-none) value forces the computed value of [column-rule-width](#propdef-column-rule-width) to be 0.

<a id="ref-for-propdef-column-rule-width①"></a>

### <a id="crw"></a>4.4. The Width Of Column Rules: the [column-rule-width](#propdef-column-rule-width) property

<strong>Table 6 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-column-rule-width"></a>column-rule-width

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-line-width"></a>

[\<line-width\>](https://www.w3.org/TR/css-backgrounds-3/#typedef-line-width)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

medium

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

multicol containers

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-line-style-hidden"></a>

<a id="ref-for-valdef-line-style-none①"></a>

<a id="ref-for-snap-a-length-as-a-border-width"></a>

absolute length, [snapped as a border width](https://www.w3.org/TR/css-values-4/#snap-a-length-as-a-border-width); 0 if the column rule style is [none](https://www.w3.org/TR/css-backgrounds-3/#valdef-line-style-none) or [hidden](https://www.w3.org/TR/css-backgrounds-3/#valdef-line-style-hidden)

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

by computed value type

This property sets the width of the rule between columns. Negative values are not allowed.

Tests

- [multicol-rule-fraction-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-fraction-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-fraction-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-fraction-001.xht)
- [multicol-rule-fraction-003.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-fraction-003.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-fraction-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-fraction-003.xht)
- [multicol-rule-px-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-px-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-px-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-px-001.xht)
- [multicol-rule-percent-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-percent-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-percent-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-percent-001.xht)
- [subpixel-column-rule-width.tentative.html](https://wpt.fyi/results/css/css-multicol/subpixel-column-rule-width.tentative.html) [(live test)](http://wpt.live/css/css-multicol/subpixel-column-rule-width.tentative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/subpixel-column-rule-width.tentative.html)
- [column-rule-width-computed.html](https://wpt.fyi/results/css/css-multicol/parsing/column-rule-width-computed.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-rule-width-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-rule-width-computed.html)
- [column-rule-width-invalid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-rule-width-invalid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-rule-width-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-rule-width-invalid.html)
- [column-rule-width-valid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-rule-width-valid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-rule-width-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-rule-width-valid.html)
- [column-rule-width-interpolation.html](https://wpt.fyi/results/css/css-multicol/animation/column-rule-width-interpolation.html) [(live test)](http://wpt.live/css/css-multicol/animation/column-rule-width-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/animation/column-rule-width-interpolation.html)

<a id="ref-for-propdef-column-rule④"></a>

### <a id="cr"></a>4.5. Column Rule Shorthand: the [column-rule](#propdef-column-rule) property

<strong>Table 7 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-column-rule"></a>column-rule

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-column-rule-color①"></a>

<a id="ref-for-propdef-column-rule-style②"></a>

<a id="ref-for-comb-any①"></a>

<a id="ref-for-propdef-column-rule-width②"></a>

[\<'column-rule-width'\>](#propdef-column-rule-width) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'column-rule-style'\>](#propdef-column-rule-style) <a id="ref-for-comb-any②"></a>\|\| [\<'column-rule-color'\>](#propdef-column-rule-color)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

see individual properties

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<a id="ref-for-propdef-column-rule-width③"></a>

<a id="ref-for-propdef-column-rule-style③"></a>

<a id="ref-for-propdef-column-rule-color②"></a>

This property is a shorthand for setting [column-rule-width](#propdef-column-rule-width), [column-rule-style](#propdef-column-rule-style), and [column-rule-color](#propdef-column-rule-color) at the same place in the style sheet. Omitted values are set to their initial values.

Tests

- [multicol-shorthand-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-shorthand-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-shorthand-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-shorthand-001.xht)
- [multicol-rule-shorthand-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-shorthand-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-shorthand-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-shorthand-001.xht)
- [multicol-rule-shorthand-2.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-shorthand-2.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-shorthand-2.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-shorthand-2.xht)
- [multicol-rule-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-000.xht)
- [multicol-rule-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-002.xht)
- [multicol-rule-dashed-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-dashed-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-dashed-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-dashed-000.xht)
- [multicol-rule-dotted-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-dotted-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-dotted-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-dotted-000.xht)
- [multicol-rule-double-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-double-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-double-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-double-000.xht)
- [multicol-rule-outset-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-outset-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-outset-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-outset-000.xht)
- [multicol-rule-none-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-none-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-none-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-none-000.xht)
- [multicol-rule-hidden-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-hidden-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-hidden-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-hidden-000.xht)
- [multicol-rule-inset-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-inset-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-inset-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-inset-000.xht)
- [multicol-rule-groove-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-groove-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-groove-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-groove-000.xht)
- [multicol-rule-ridge-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-ridge-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-ridge-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-ridge-000.xht)
- [multicol-rule-solid-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-solid-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-solid-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-solid-000.xht)
- [column-rule-computed.html](https://wpt.fyi/results/css/css-multicol/parsing/column-rule-computed.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-rule-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-rule-computed.html)
- [column-rule-invalid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-rule-invalid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-rule-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-rule-invalid.html)
- [column-rule-valid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-rule-valid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-rule-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-rule-valid.html)
- [column-rule-shorthand.html](https://wpt.fyi/results/css/css-multicol/parsing/column-rule-shorthand.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-rule-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-rule-shorthand.html)

<a id="ref-for-column-rule⑤"></a>

<a id="ref-for-column-gap②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ef50cfc6"></a> In this example, the [column rule](#column-rule) and the [column gap](#column-gap) have the same width. Therefore, they will occupy exactly the same space.
>
> ```text
> body {
>   column-gap: 35px;
>   column-rule-width: 35px;
>   column-rule-style: solid;
>   column-rule-color: black;
> }
> ```
>
> ![The rule completely covers any gap.](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/rule-same-width-as-gap.svg)
>
> The column rule and column gap occupy the same space.
>
> Tests
>
> - [equal-gap-and-rule.html](https://wpt.fyi/results/css/css-multicol/equal-gap-and-rule.html) [(live test)](http://wpt.live/css/css-multicol/equal-gap-and-rule.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/equal-gap-and-rule.html)
> - [multicol-rule-samelength-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-rule-samelength-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-rule-samelength-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-rule-samelength-001.xht)
> - [fixed-size-child-with-overflow.html](https://wpt.fyi/results/css/css-multicol/fixed-size-child-with-overflow.html) [(live test)](http://wpt.live/css/css-multicol/fixed-size-child-with-overflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/fixed-size-child-with-overflow.html)

## <a id="column-breaks"></a>5.  Column Breaks

When content is laid out in multiple columns, the user agent must determine where column breaks are placed. The problem of breaking content into columns is similar to breaking content into pages, which is described in CSS 2.1, section 13.3.3 [\[CSS21\]](#biblio-css21).

<a id="ref-for-propdef-break-before"></a>

<a id="ref-for-propdef-break-after"></a>

<a id="ref-for-propdef-break-inside"></a>

Three new properties are introduced to allow column breaks to be described in the same properties as page breaks: [break-before](https://www.w3.org/TR/css-break-3/#propdef-break-before), [break-after](https://www.w3.org/TR/css-break-3/#propdef-break-after), and [break-inside](https://www.w3.org/TR/css-break-3/#propdef-break-inside).

<a id="ref-for-propdef-break-before①"></a>

<a id="ref-for-propdef-break-after①"></a>

<a id="ref-for-propdef-break-inside①"></a>

### <a id="break-before-break-after-break-inside"></a>5.1. Controlling Fragmentation: the [break-before](https://www.w3.org/TR/css-break-3/#propdef-break-before), [break-after](https://www.w3.org/TR/css-break-3/#propdef-break-after), [break-inside](https://www.w3.org/TR/css-break-3/#propdef-break-inside) properties

<a id="ref-for-propdef-break-before②"></a>

<a id="ref-for-propdef-break-after②"></a>

<a id="ref-for-propdef-break-inside②"></a>

[break-before](https://www.w3.org/TR/css-break-3/#propdef-break-before), [break-after](https://www.w3.org/TR/css-break-3/#propdef-break-after), and [break-inside](https://www.w3.org/TR/css-break-3/#propdef-break-inside) are defined in [\[CSS3-BREAK\]](#biblio-css3-break).

Tests

- [multicol-break-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-break-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-break-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-break-000.xht)
- [multicol-break-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-break-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-break-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-break-001.xht)
- [multicol-br-inside-avoidcolumn-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-br-inside-avoidcolumn-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-br-inside-avoidcolumn-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-br-inside-avoidcolumn-001.xht)
- moz-multicol3-column-balancing-break-inside-avoid-1.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/moz-multicol3-column-balancing-break-inside-avoid-1.html)

## <a id="spanning-columns"></a>6.  Spanning Columns

<a id="ref-for-propdef-column-span①"></a>

The [column-span](#propdef-column-span) property makes it possible for an element to span across several columns.

<a id="ref-for-propdef-column-span②"></a>

### <a id="column-span"></a>6.1. Spanning An Element Across Columns: the [column-span](#propdef-column-span) property

<strong>Table 8 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-column-span"></a>column-span

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one②"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) all

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

in-flow block-level elements

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified keyword

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

Tests

- [column-span-invalid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-span-invalid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-span-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-span-invalid.html)

This property describes how many columns an element spans across. Values are:

<a id="valdef-column-span-none"></a>none  
The element does not span multiple columns.

Tests

- [multicol-span-none-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-span-none-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-span-none-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-none-001.xht)

<a id="valdef-column-span-all"></a>all  
<a id="ref-for-independent-formatting-context②"></a>

<a id="ref-for-multi-column-line④"></a>

<a id="ref-for-block-formatting-context③"></a>

<a id="ref-for-out-of-flow"></a>

The element forces a column break and is taken [out of flow](https://www.w3.org/TR/css-display-3/#out-of-flow) to span across all columns of the nearest multicol ancestor in the same [block formatting context](https://www.w3.org/TR/css-display-3/#block-formatting-context). Content in the normal flow that appears before the element is automatically balanced across all columns in the immediately preceding [multi-column line](#multi-column-line) before the element appears, and any subsequent content flows into a new <a id="ref-for-multi-column-line⑤"></a>multi-column line after the element. The element establishes an [independent formatting context](https://www.w3.org/TR/css-display-3/#independent-formatting-context).

<a id="ref-for-formatting-context"></a>

<a id="ref-for-propdef-column-span③"></a>

<a id="ref-for-valdef-column-span-all"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Whether the element establishes a new [formatting context](https://www.w3.org/TR/css-display-3/#formatting-context) does not depend on whether the element is a descendant of a multicol or not. When [column-span](#propdef-column-span) is [all](#valdef-column-span-all), it always does. This helps with robustness of designs to later revisions that remove the multicol, or when media queries turn the multicol off in some situations.

Tests

- [multicol-span-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-span-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-span-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-000.xht)
- [multicol-span-all-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-span-all-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-001.xht)
- [multicol-span-all-003.xht](https://wpt.fyi/results/css/css-multicol/multicol-span-all-003.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-003.xht)
- [multicol-span-all-block-sibling-003.xht](https://wpt.fyi/results/css/css-multicol/multicol-span-all-block-sibling-003.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-block-sibling-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-block-sibling-003.xht)
- [multicol-span-all-margin-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-span-all-margin-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-margin-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-margin-001.xht)
- [multicol-span-all-margin-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-span-all-margin-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-margin-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-margin-002.xht)
- [multicol-span-all-margin-003.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-margin-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-margin-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-margin-003.html)
- [multicol-span-all-margin-bottom-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-span-all-margin-bottom-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-margin-bottom-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-margin-bottom-001.xht)
- [multicol-span-all-margin-nested-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-span-all-margin-nested-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-margin-nested-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-margin-nested-001.xht)
- [multicol-span-all-margin-nested-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-span-all-margin-nested-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-margin-nested-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-margin-nested-002.xht)
- [multicol-span-all-margin-nested-firstchild-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-span-all-margin-nested-firstchild-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-margin-nested-firstchild-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-margin-nested-firstchild-001.xht)
- [multicol-span-float-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-span-float-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-span-float-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-float-001.xht)
- [multicol-span-float-002.html](https://wpt.fyi/results/css/css-multicol/multicol-span-float-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-float-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-float-002.html)
- [multicol-span-float-003.html](https://wpt.fyi/results/css/css-multicol/multicol-span-float-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-float-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-float-003.html)
- [inline-block-and-column-span-all.html](https://wpt.fyi/results/css/css-multicol/inline-block-and-column-span-all.html) [(live test)](http://wpt.live/css/css-multicol/inline-block-and-column-span-all.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/inline-block-and-column-span-all.html)
- [multicol-span-all-dynamic-remove-001.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-remove-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-remove-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-remove-001.html)
- [multicol-span-all-dynamic-add-001.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-add-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-add-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-add-001.html)
- [multicol-span-all-dynamic-remove-002.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-remove-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-remove-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-remove-002.html)
- [multicol-span-all-dynamic-add-002.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-add-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-add-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-add-002.html)
- [multicol-span-all-dynamic-remove-003.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-remove-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-remove-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-remove-003.html)
- [multicol-span-all-dynamic-add-003.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-add-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-add-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-add-003.html)
- [multicol-span-all-dynamic-remove-004.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-remove-004.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-remove-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-remove-004.html)
- [multicol-span-all-dynamic-add-004.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-add-004.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-add-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-add-004.html)
- [multicol-span-all-dynamic-add-005.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-add-005.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-add-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-add-005.html)
- [multicol-span-all-dynamic-remove-005.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-remove-005.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-remove-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-remove-005.html)
- [multicol-span-all-dynamic-remove-006.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-remove-006.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-remove-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-remove-006.html)
- [multicol-span-all-dynamic-add-006.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-add-006.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-add-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-add-006.html)
- [multicol-span-all-dynamic-remove-007.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-remove-007.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-remove-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-remove-007.html)
- [multicol-span-all-dynamic-add-007.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-add-007.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-add-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-add-007.html)
- [multicol-span-all-dynamic-add-008.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-add-008.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-add-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-add-008.html)
- [multicol-span-all-dynamic-add-009.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-add-009.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-add-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-add-009.html)
- [multicol-span-all-dynamic-add-010.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-add-010.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-add-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-add-010.html)
- [multicol-span-all-dynamic-add-011.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-add-011.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-add-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-add-011.html)
- [multicol-span-all-dynamic-add-012.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-add-012.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-add-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-add-012.html)
- [multicol-span-all-dynamic-add-013.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-dynamic-add-013.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-dynamic-add-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-dynamic-add-013.html)
- [multicol-span-all-children-height-001.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-001.html)
- [multicol-span-all-children-height-002.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-002.html)
- [multicol-span-all-children-height-003.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-003.html)
- [multicol-span-all-children-height-004a.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-004a.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-004a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-004a.html)
- [multicol-span-all-children-height-004b.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-004b.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-004b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-004b.html)
- [multicol-span-all-children-height-005.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-005.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-005.html)
- [multicol-span-all-children-height-006.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-006.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-006.html)
- [multicol-span-all-children-height-007.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-007.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-007.html)
- [multicol-span-all-children-height-008.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-008.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-008.html)
- [multicol-span-all-children-height-009.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-009.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-009.html)
- [multicol-span-all-children-height-010.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-010.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-010.html)
- [multicol-span-all-children-height-011.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-011.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-011.html)
- [multicol-span-all-children-height-012.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-012.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-012.html)
- [multicol-span-all-children-height-013.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-children-height-013.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-children-height-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-children-height-013.html)
- [multicol-span-all-004.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-004.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-004.html)
- [multicol-span-all-005.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-005.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-005.html)
- [multicol-span-all-006.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-006.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-006.html)
- [multicol-span-all-007.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-007.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-007.html)
- [multicol-span-all-008.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-008.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-008.html)
- [multicol-span-all-009.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-009.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-009.html)
- [multicol-span-all-010.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-010.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-010.html)
- [multicol-span-all-011.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-011.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-011.html)
- [multicol-span-all-012.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-012.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-012.html)
- [multicol-span-all-013.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-013.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-013.html)
- [multicol-span-all-014.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-014.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-014.html)
- [multicol-span-all-015.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-015.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-015.html)
- [multicol-span-all-016.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-016.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-016.html)
- [multicol-span-all-017.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-017.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-017.html)
- [multicol-span-all-018.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-018.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-018.html)
- [multicol-span-all-019.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-019.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-019.html)
- [multicol-span-all-rule-001.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-rule-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-rule-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-rule-001.html)
- [multicol-span-all-button-001.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-button-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-button-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-button-001.html)
- [multicol-span-all-button-002.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-button-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-button-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-button-002.html)
- [multicol-span-all-button-003.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-button-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-button-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-button-003.html)
- [multicol-span-all-fieldset-001.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-fieldset-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-fieldset-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-fieldset-001.html)
- [multicol-span-all-fieldset-002.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-fieldset-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-fieldset-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-fieldset-002.html)
- [multicol-span-all-fieldset-003.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-fieldset-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-fieldset-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-fieldset-003.html)
- [multicol-span-all-restyle-001.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-restyle-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-restyle-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-restyle-001.html)
- [multicol-span-all-restyle-002.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-restyle-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-restyle-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-restyle-002.html)
- [multicol-span-all-restyle-003.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-restyle-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-restyle-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-restyle-003.html)
- [multicol-span-all-restyle-004.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-restyle-004.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-restyle-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-restyle-004.html)
- [multicol-span-all-list-item-001.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-list-item-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-list-item-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-list-item-001.html)
- [multicol-span-all-list-item-002.html](https://wpt.fyi/results/css/css-multicol/multicol-span-all-list-item-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-list-item-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-list-item-002.html)
- [float-with-line-after-spanner.html](https://wpt.fyi/results/css/css-multicol/float-with-line-after-spanner.html) [(live test)](http://wpt.live/css/css-multicol/float-with-line-after-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/float-with-line-after-spanner.html)
- [parallel-flow-after-spanner-001.html](https://wpt.fyi/results/css/css-multicol/parallel-flow-after-spanner-001.html) [(live test)](http://wpt.live/css/css-multicol/parallel-flow-after-spanner-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parallel-flow-after-spanner-001.html)
- [parallel-flow-after-spanner-002.html](https://wpt.fyi/results/css/css-multicol/parallel-flow-after-spanner-002.html) [(live test)](http://wpt.live/css/css-multicol/parallel-flow-after-spanner-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parallel-flow-after-spanner-002.html)
- [margin-and-break-before-child-spanner.html](https://wpt.fyi/results/css/css-multicol/crashtests/margin-and-break-before-child-spanner.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/margin-and-break-before-child-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/margin-and-break-before-child-spanner.html)
- [multicol-width-004.html](https://wpt.fyi/results/css/css-multicol/multicol-width-004.html) [(live test)](http://wpt.live/css/css-multicol/multicol-width-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-width-004.html)

An element that spans more than one column is called a <a id="multi-column-spanning-element"></a>multi-column spanning element and the box it creates is called a <a id="multi-column-spanner"></a>multi-column spanner.

<a id="ref-for-containing-block②"></a>

<a id="ref-for-multi-column-spanner②"></a>

<a id="ref-for-multi-column-container①⑦"></a>

<a id="ref-for-absolute-position①"></a>

<a id="ref-for-containing-block-chain"></a>

The [containing block](https://www.w3.org/TR/css-display-3/#containing-block) of the [spanner](#multi-column-spanner) is the [multicol container](#multi-column-container) itself. Consequently, in cases where the spanner itself does not establish a <a id="ref-for-containing-block③"></a>containing block for [absolutely positioned boxes](https://www.w3.org/TR/css-position-3/#absolute-position) inside the spanner, their [containing block chain](https://www.w3.org/TR/css-display-3/#containing-block-chain) skips directly to the <a id="ref-for-multi-column-container①⑧"></a>multicol container (skipping any ancestors between the <a id="ref-for-multi-column-spanner③"></a>spanner and the <a id="ref-for-multi-column-container①⑨"></a>multicol container).

<a id="ref-for-out-of-flow①"></a>

Although the spanner is taken [out-of-flow](https://www.w3.org/TR/css-display-3/#out-of-flow), this does not affect the [painting order](https://www.w3.org/TR/CSS2/zindex.html) [\[CSS21\]](#biblio-css21) of the spanning element.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b640c5a3"></a> In this example, an `h2` element has been added to the sample document after the sixth sentence (i.e., after the words "the leg of a"). This styling applies:
>
> ```text
> h2 { column-span: all; background: silver }
> ```
>
> <a id="ref-for-propdef-column-span④"></a>
>
> <a id="ref-for-valdef-column-span-all①"></a>
>
> By setting [column-span](#propdef-column-span) to [all](#valdef-column-span-all), all content that appears before the `h2` element is shown above the `h2` element.
>
> ![An element spans all three columns](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/h2-spanner.svg)
>
> The h2 element is set to column-span: all
>
> <a id="ref-for-multi-column-line⑥"></a>
>
> <a id="ref-for-column-rule⑥"></a>
>
> <a id="ref-for-column-box⑥"></a>
>
> Note that because the spanner splits the [multi-column line](#multi-column-line), it also interrupts any [column rules](#column-rule) (which are only drawn between [columns](#column-box) in a <a id="ref-for-multi-column-line⑦"></a>multi-column line).

<a id="ref-for-formatting-context①"></a>

<a id="ref-for-multi-column-container②⓪"></a>

A spanning element may be lower than the first level of descendants as long as they are part of the same [formatting context](https://www.w3.org/TR/css-display-3/#formatting-context), and there is nothing between the spanning element and [multicol container](#multi-column-container) that establishes a containing block for fixed position descendants.

<a id="ref-for-propdef-column-span⑤"></a>

<a id="ref-for-propdef-transform"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c35d3254"></a> In this example, the element with [column-span: all](#propdef-column-span) is inside an element with [transform: rotate(90deg)](https://www.w3.org/TR/css-transforms-1/#propdef-transform). The transform establishes a containing block for fixed position descendents, therefore a spanner will not be created.
>
> ```text
> <article>
>   <section>
>     <div class="spanner">Attempted spanner</div>
>   </section>
> </article>
> ```
>
> ```text
> article {
>   columns: 2;
> }
> 
> section {
>   transform: rotate(90deg);
> }
> 
> .spanner {
>   column-span: all;
>   background: silver;
> }
> ```
Tests

- [fixed-in-multicol-with-transform-container.html](https://wpt.fyi/results/css/css-multicol/fixed-in-multicol-with-transform-container.html) [(live test)](http://wpt.live/css/css-multicol/fixed-in-multicol-with-transform-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/fixed-in-multicol-with-transform-container.html)
- [change-transform-in-nested.html](https://wpt.fyi/results/css/css-multicol/change-transform-in-nested.html) [(live test)](http://wpt.live/css/css-multicol/change-transform-in-nested.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/change-transform-in-nested.html)
- [change-transform-in-second-column.html](https://wpt.fyi/results/css/css-multicol/change-transform-in-second-column.html) [(live test)](http://wpt.live/css/css-multicol/change-transform-in-second-column.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/change-transform-in-second-column.html)
- [change-transform.html](https://wpt.fyi/results/css/css-multicol/change-transform.html) [(live test)](http://wpt.live/css/css-multicol/change-transform.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/change-transform.html)

<a id="ref-for-fragment①"></a>

If the fragment before the spanner is empty, nothing special happens; the top margin/border/padding is above the spanning element, as an empty [fragment](https://www.w3.org/TR/css-break-4/#fragment).

<a id="ref-for-multi-column-container②①"></a>

<a id="ref-for-valdef-column-span-all②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fa0001d6"></a> In this example the [multicol container](#multi-column-container) is the `article` element. Inside this parent is a paragraph and then a section element. The section contains an `h2` heading set to [all](#valdef-column-span-all) this spans all three columns while the containing section remains inside the column boxes.
>
> The `h2` is the first child of the section. This means that the margin, border (shown in red in the diagram) and padding on this section appear before the spanning `h2` as an empty fragment.
>
> ```text
> <article>
>   <p>...</p>
>   <section>
>     <h2>An h2 element</h2>
>     <p>...</p>
>   </section>
> </article>
> ```
>
> ```text
> section {
>   border: 2px solid red;
>   margin-top: 65px;
>   padding-top: 20px;
> }
> 
> h2 {
>   column-span: all;
>   background: silver
> }
> ```
>
> ![An element spans all three columns, the red border around the section breaks before the spanner.](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/nested-spanner.svg)
>
> <a id="ref-for-propdef-column-span⑥"></a>
>
> The `h2` element is set to [column-span: all](#propdef-column-span), the section has a red border and top padding and margin

Tests

- [spanner-fragmentation-000.html](https://wpt.fyi/results/css/css-multicol/spanner-fragmentation-000.html) [(live test)](http://wpt.live/css/css-multicol/spanner-fragmentation-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-fragmentation-000.html)
- [spanner-fragmentation-001.html](https://wpt.fyi/results/css/css-multicol/spanner-fragmentation-001.html) [(live test)](http://wpt.live/css/css-multicol/spanner-fragmentation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-fragmentation-001.html)
- [spanner-fragmentation-002.html](https://wpt.fyi/results/css/css-multicol/spanner-fragmentation-002.html) [(live test)](http://wpt.live/css/css-multicol/spanner-fragmentation-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-fragmentation-002.html)
- [spanner-fragmentation-003.html](https://wpt.fyi/results/css/css-multicol/spanner-fragmentation-003.html) [(live test)](http://wpt.live/css/css-multicol/spanner-fragmentation-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-fragmentation-003.html)
- [spanner-fragmentation-004.html](https://wpt.fyi/results/css/css-multicol/spanner-fragmentation-004.html) [(live test)](http://wpt.live/css/css-multicol/spanner-fragmentation-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-fragmentation-004.html)
- [spanner-fragmentation-005.html](https://wpt.fyi/results/css/css-multicol/spanner-fragmentation-005.html) [(live test)](http://wpt.live/css/css-multicol/spanner-fragmentation-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-fragmentation-005.html)
- [spanner-fragmentation-006.html](https://wpt.fyi/results/css/css-multicol/spanner-fragmentation-006.html) [(live test)](http://wpt.live/css/css-multicol/spanner-fragmentation-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-fragmentation-006.html)
- [spanner-fragmentation-007.html](https://wpt.fyi/results/css/css-multicol/spanner-fragmentation-007.html) [(live test)](http://wpt.live/css/css-multicol/spanner-fragmentation-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-fragmentation-007.html)
- [spanner-fragmentation-008.html](https://wpt.fyi/results/css/css-multicol/spanner-fragmentation-008.html) [(live test)](http://wpt.live/css/css-multicol/spanner-fragmentation-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-fragmentation-008.html)
- [spanner-fragmentation-009.html](https://wpt.fyi/results/css/css-multicol/spanner-fragmentation-009.html) [(live test)](http://wpt.live/css/css-multicol/spanner-fragmentation-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-fragmentation-009.html)
- [spanner-fragmentation-010.html](https://wpt.fyi/results/css/css-multicol/spanner-fragmentation-010.html) [(live test)](http://wpt.live/css/css-multicol/spanner-fragmentation-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-fragmentation-010.html)
- [spanner-fragmentation-011.html](https://wpt.fyi/results/css/css-multicol/spanner-fragmentation-011.html) [(live test)](http://wpt.live/css/css-multicol/spanner-fragmentation-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-fragmentation-011.html)
- [spanner-fragmentation-012.html](https://wpt.fyi/results/css/css-multicol/spanner-fragmentation-012.html) [(live test)](http://wpt.live/css/css-multicol/spanner-fragmentation-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-fragmentation-012.html)

<a id="ref-for-valdef-column-span-none"></a>

A spanning element takes up more space than the element would take up otherwise. When space is limited, it may be impossible to find room for the spanning element. In these cases, user agents may treat the element as if [none](#valdef-column-span-none) had been specified on this property.

<a id="ref-for-propdef-column-span⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2980e0d6"></a> In this example, the `h2` element appears later in the content, and the height of the multicol container is constrained. Therefore, the `h2` element appears in the overflow and there is not room to make the element spanning. As a result, the element appears as if [column-span: none](#propdef-column-span) was specified.
>
> ![The h2 element is in an overflow column](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/h2-in-the-overflow-no-span.svg)
>
> The h2 element is in an overflow column and appears as if column-span none is specified

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e49da6ca"></a> This example is similar to the previous example, except that the H2 element appears naturally in the last column. Still, there is not enough room to make the element spanning.
>
> ![The h2 element is in the final column](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/h2-in-the-last-column-no-span.svg)
>
> The h2 element is in the final column and appears as if column-span none is specified

Tests

- [multicol-span-all-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-span-all-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-span-all-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-span-all-002.xht)

<a id="ref-for-paged-media①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8f69cecc"></a> In fragmented contexts spanning elements are honored in all fragments. In this example, we are in [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media), and the first three paragraphs have column breaks after them. A spanning `H2` element appears after the fourth paragraph.
>
> ![Three columns with two lines of text each](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/spanner-page-break1.svg)
>
> This would appear on the first page
>
> ![A spanning element across the three columns, text above and below.](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/spanner-page-break2.svg)
>
> This would appear on the second page

<a id="ref-for-block-formatting-context④"></a>

Spanners are block-level boxes therefore the margins of two adjacent spanners will collapse with each other. The margins of two spanners separated only by an absolutely positioned item will collapse with each other, as absolutely positioned items do not create column boxes. As column boxes establish a new [block formatting context](https://www.w3.org/TR/css-display-3/#block-formatting-context), margins on elements inside a column box will not collapse with the margin of a spanner.

Tests

- [non-adjacent-spanners-000.html](https://wpt.fyi/results/css/css-multicol/non-adjacent-spanners-000.html) [(live test)](http://wpt.live/css/css-multicol/non-adjacent-spanners-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/non-adjacent-spanners-000.html)
- [non-adjacent-spanners-001.html](https://wpt.fyi/results/css/css-multicol/non-adjacent-spanners-001.html) [(live test)](http://wpt.live/css/css-multicol/non-adjacent-spanners-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/non-adjacent-spanners-001.html)

<a id="ref-for-formatting-context②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f82c16c7"></a> Spanners establish new [formatting contexts](https://www.w3.org/TR/css-display-3/#formatting-context), but their margins can be changed by their surroundings. In this example, two spanners naturally end up at the top of a page. The top margin of the first spanner is truncated due to adjoining an unforced break. The margins between the two spanners collapse with each other. However, the bottom margin of the second spanner does not collapse with the top margin of the subsequent element.
>
> ```text
> h2 {
>   margin: 16px 0;
>   column-span: all;
>   background: silver
> }
> p { margin-top: 16px }
> ```
>
> ![Two spanning elements after a page break](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/two-spanners-margin-no-collapse.svg)
>
> Margins collapse between two spanning elements, but not the bottom margin of a spanner and top margin of next element.

Tests

Additional tests relating to spanners.

- [abspos-in-multicol-with-spanner-crash.html](https://wpt.fyi/results/css/css-multicol/abspos-in-multicol-with-spanner-crash.html) [(live test)](http://wpt.live/css/css-multicol/abspos-in-multicol-with-spanner-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/abspos-in-multicol-with-spanner-crash.html)
- [body-becomes-spanner-html-becomes-vertical-rl.html](https://wpt.fyi/results/css/css-multicol/crashtests/body-becomes-spanner-html-becomes-vertical-rl.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/body-becomes-spanner-html-becomes-vertical-rl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/body-becomes-spanner-html-becomes-vertical-rl.html)
- [fit-content-with-spanner-and-auto-scrollbar-sibling.html](https://wpt.fyi/results/css/css-multicol/crashtests/fit-content-with-spanner-and-auto-scrollbar-sibling.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/fit-content-with-spanner-and-auto-scrollbar-sibling.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/fit-content-with-spanner-and-auto-scrollbar-sibling.html)
- [float-becomes-non-float-spanner-surprises-inside.html](https://wpt.fyi/results/css/css-multicol/crashtests/float-becomes-non-float-spanner-surprises-inside.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/float-becomes-non-float-spanner-surprises-inside.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/float-becomes-non-float-spanner-surprises-inside.html)
- [float-becomes-spanner.html](https://wpt.fyi/results/css/css-multicol/crashtests/float-becomes-spanner.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/float-becomes-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/float-becomes-spanner.html)
- [multicol-floats-after-column-span-crash.html](https://wpt.fyi/results/css/css-multicol/crashtests/multicol-floats-after-column-span-crash.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/multicol-floats-after-column-span-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/multicol-floats-after-column-span-crash.html)
- [negative-margin-on-column-spanner.html](https://wpt.fyi/results/css/css-multicol/crashtests/negative-margin-on-column-spanner.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/negative-margin-on-column-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/negative-margin-on-column-spanner.html)
- [nested-spanner-with-negative-margin.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-spanner-with-negative-margin.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-spanner-with-negative-margin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-spanner-with-negative-margin.html)
- [oof-becomes-spanner.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-becomes-spanner.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-becomes-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-becomes-spanner.html)
- [oof-in-additional-column-before-spanner.html](https://wpt.fyi/results/css/css-multicol/crashtests/oof-in-additional-column-before-spanner.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/oof-in-additional-column-before-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/oof-in-additional-column-before-spanner.html)
- [relpos-spanner-with-spanner-child-becomes-regular.html](https://wpt.fyi/results/css/css-multicol/crashtests/relpos-spanner-with-spanner-child-becomes-regular.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/relpos-spanner-with-spanner-child-becomes-regular.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/relpos-spanner-with-spanner-child-becomes-regular.html)
- [remove-spanner-after-spanner-in-inline-before-inline.html](https://wpt.fyi/results/css/css-multicol/crashtests/remove-spanner-after-spanner-in-inline-before-inline.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/remove-spanner-after-spanner-in-inline-before-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/remove-spanner-after-spanner-in-inline-before-inline.html)
- [remove-spanner-in-table-caption-nested-multicol.html](https://wpt.fyi/results/css/css-multicol/crashtests/remove-spanner-in-table-caption-nested-multicol.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/remove-spanner-in-table-caption-nested-multicol.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/remove-spanner-in-table-caption-nested-multicol.html)
- [restricted-height-bottom-border-overflow-and-spanner.html](https://wpt.fyi/results/css/css-multicol/crashtests/restricted-height-bottom-border-overflow-and-spanner.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/restricted-height-bottom-border-overflow-and-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/restricted-height-bottom-border-overflow-and-spanner.html)
- [scrollable-spanner-in-nested.html](https://wpt.fyi/results/css/css-multicol/crashtests/scrollable-spanner-in-nested.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/scrollable-spanner-in-nested.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/scrollable-spanner-in-nested.html)
- [spanner-after-parallel-flow.html](https://wpt.fyi/results/css/css-multicol/crashtests/spanner-after-parallel-flow.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/spanner-after-parallel-flow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/spanner-after-parallel-flow.html)
- [spanner-in-inline-after-very-tall-content-001.html](https://wpt.fyi/results/css/css-multicol/crashtests/spanner-in-inline-after-very-tall-content-001.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/spanner-in-inline-after-very-tall-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/spanner-in-inline-after-very-tall-content-001.html)
- [spanner-in-inline-after-very-tall-content-002.html](https://wpt.fyi/results/css/css-multicol/crashtests/spanner-in-inline-after-very-tall-content-002.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/spanner-in-inline-after-very-tall-content-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/spanner-in-inline-after-very-tall-content-002.html)
- [spanner-in-overflowed-container-before-float.html](https://wpt.fyi/results/css/css-multicol/crashtests/spanner-in-overflowed-container-before-float.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/spanner-in-overflowed-container-before-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/spanner-in-overflowed-container-before-float.html)
- [nested-with-tall-padding-and-spanner-and-content.html](https://wpt.fyi/results/css/css-multicol/crashtests/nested-with-tall-padding-and-spanner-and-content.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/nested-with-tall-padding-and-spanner-and-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/nested-with-tall-padding-and-spanner-and-content.html)
- [specified-height-with-just-spanner-and-oof.html](https://wpt.fyi/results/css/css-multicol/crashtests/specified-height-with-just-spanner-and-oof.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/specified-height-with-just-spanner-and-oof.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/specified-height-with-just-spanner-and-oof.html)
- [trailing-parent-padding-between-spanners.html](https://wpt.fyi/results/css/css-multicol/crashtests/trailing-parent-padding-between-spanners.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/trailing-parent-padding-between-spanners.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/trailing-parent-padding-between-spanners.html)
- [table-caption-change-descendant-display-type.html](https://wpt.fyi/results/css/css-multicol/crashtests/table-caption-change-descendant-display-type.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/table-caption-change-descendant-display-type.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/table-caption-change-descendant-display-type.html)
- [table-caption-inline-block-remove-child.html](https://wpt.fyi/results/css/css-multicol/crashtests/table-caption-inline-block-remove-child.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/table-caption-inline-block-remove-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/table-caption-inline-block-remove-child.html)
- [remove-block-beside-spanner-in-inline-crash.html](https://wpt.fyi/results/css/css-multicol/remove-block-beside-spanner-in-inline-crash.html) [(live test)](http://wpt.live/css/css-multicol/remove-block-beside-spanner-in-inline-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/remove-block-beside-spanner-in-inline-crash.html)
- [remove-inline-with-block-beside-spanners-crash.html](https://wpt.fyi/results/css/css-multicol/remove-inline-with-block-beside-spanners-crash.html) [(live test)](http://wpt.live/css/css-multicol/remove-inline-with-block-beside-spanners-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/remove-inline-with-block-beside-spanners-crash.html)
- [remove-spanner-beside-spanner-in-inline-crash.html](https://wpt.fyi/results/css/css-multicol/remove-spanner-beside-spanner-in-inline-crash.html) [(live test)](http://wpt.live/css/css-multicol/remove-spanner-beside-spanner-in-inline-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/remove-spanner-beside-spanner-in-inline-crash.html)
- [spanning-legend-000-crash.html](https://wpt.fyi/results/css/css-multicol/spanning-legend-000-crash.html) [(live test)](http://wpt.live/css/css-multicol/spanning-legend-000-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanning-legend-000-crash.html)
- [spanning-legend-001-crash.html](https://wpt.fyi/results/css/css-multicol/spanning-legend-001-crash.html) [(live test)](http://wpt.live/css/css-multicol/spanning-legend-001-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanning-legend-001-crash.html)
- [toggle-spanner-float-crash.html](https://wpt.fyi/results/css/css-multicol/toggle-spanner-float-crash.html) [(live test)](http://wpt.live/css/css-multicol/toggle-spanner-float-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/toggle-spanner-float-crash.html)
- [abspos-after-spanner-static-pos.html](https://wpt.fyi/results/css/css-multicol/abspos-after-spanner-static-pos.html) [(live test)](http://wpt.live/css/css-multicol/abspos-after-spanner-static-pos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/abspos-after-spanner-static-pos.html)
- [abspos-after-spanner.html](https://wpt.fyi/results/css/css-multicol/abspos-after-spanner.html) [(live test)](http://wpt.live/css/css-multicol/abspos-after-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/abspos-after-spanner.html)
- [abspos-containing-block-outside-spanner.html](https://wpt.fyi/results/css/css-multicol/abspos-containing-block-outside-spanner.html) [(live test)](http://wpt.live/css/css-multicol/abspos-containing-block-outside-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/abspos-containing-block-outside-spanner.html)
- [change-transform-in-spanner.html](https://wpt.fyi/results/css/css-multicol/change-transform-in-spanner.html) [(live test)](http://wpt.live/css/css-multicol/change-transform-in-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/change-transform-in-spanner.html)
- [nested-with-padding-and-spanner.html](https://wpt.fyi/results/css/css-multicol/nested-with-padding-and-spanner.html) [(live test)](http://wpt.live/css/css-multicol/nested-with-padding-and-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/nested-with-padding-and-spanner.html)
- [orthogonal-writing-mode-spanner.html](https://wpt.fyi/results/css/css-multicol/orthogonal-writing-mode-spanner.html) [(live test)](http://wpt.live/css/css-multicol/orthogonal-writing-mode-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/orthogonal-writing-mode-spanner.html)
- [remove-inline-with-block-beside-spanners.html](https://wpt.fyi/results/css/css-multicol/remove-inline-with-block-beside-spanners.html) [(live test)](http://wpt.live/css/css-multicol/remove-inline-with-block-beside-spanners.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/remove-inline-with-block-beside-spanners.html)
- [replaced-content-spanner-auto-width.html](https://wpt.fyi/results/css/css-multicol/replaced-content-spanner-auto-width.html) [(live test)](http://wpt.live/css/css-multicol/replaced-content-spanner-auto-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/replaced-content-spanner-auto-width.html)
- [spanner-in-child-after-parallel-flow-001.html](https://wpt.fyi/results/css/css-multicol/spanner-in-child-after-parallel-flow-001.html) [(live test)](http://wpt.live/css/css-multicol/spanner-in-child-after-parallel-flow-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-in-child-after-parallel-flow-001.html)
- [spanner-in-child-after-parallel-flow-002.html](https://wpt.fyi/results/css/css-multicol/spanner-in-child-after-parallel-flow-002.html) [(live test)](http://wpt.live/css/css-multicol/spanner-in-child-after-parallel-flow-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-in-child-after-parallel-flow-002.html)
- [spanner-in-child-after-parallel-flow-003.html](https://wpt.fyi/results/css/css-multicol/spanner-in-child-after-parallel-flow-003.html) [(live test)](http://wpt.live/css/css-multicol/spanner-in-child-after-parallel-flow-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-in-child-after-parallel-flow-003.html)
- [spanner-in-child-after-parallel-flow-004.html](https://wpt.fyi/results/css/css-multicol/spanner-in-child-after-parallel-flow-004.html) [(live test)](http://wpt.live/css/css-multicol/spanner-in-child-after-parallel-flow-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-in-child-after-parallel-flow-004.html)
- [spanner-in-opacity.html](https://wpt.fyi/results/css/css-multicol/spanner-in-opacity.html) [(live test)](http://wpt.live/css/css-multicol/spanner-in-opacity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/spanner-in-opacity.html)
- [going-out-of-flow-after-spanner.html](https://wpt.fyi/results/css/css-multicol/going-out-of-flow-after-spanner.html) [(live test)](http://wpt.live/css/css-multicol/going-out-of-flow-after-spanner.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/going-out-of-flow-after-spanner.html)
- [inline-with-spanner-in-overflowed-container-before-multicol-float.html](https://wpt.fyi/results/css/css-multicol/crashtests/inline-with-spanner-in-overflowed-container-before-multicol-float.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/inline-with-spanner-in-overflowed-container-before-multicol-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/inline-with-spanner-in-overflowed-container-before-multicol-float.html)
- [spanner-in-overflowed-clipped-container.html](https://wpt.fyi/results/css/css-multicol/crashtests/spanner-in-overflowed-clipped-container.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/spanner-in-overflowed-clipped-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/spanner-in-overflowed-clipped-container.html)
- [spanner-in-overflowed-container-before-inline-content.html](https://wpt.fyi/results/css/css-multicol/crashtests/spanner-in-overflowed-container-before-inline-content.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/spanner-in-overflowed-container-before-inline-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/spanner-in-overflowed-container-before-inline-content.html)
- [spanner-inside-inline-in-overflowed-container.html](https://wpt.fyi/results/css/css-multicol/crashtests/spanner-inside-inline-in-overflowed-container.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/spanner-inside-inline-in-overflowed-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/spanner-inside-inline-in-overflowed-container.html)
- [column-span-valid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-span-valid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-span-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-span-valid.html)
- [column-span-computed.html](https://wpt.fyi/results/css/css-multicol/parsing/column-span-computed.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-span-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-span-computed.html)
- [getclientrects-000.html](https://wpt.fyi/results/css/css-multicol/getclientrects-000.html) [(live test)](http://wpt.live/css/css-multicol/getclientrects-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/getclientrects-000.html)
- [getclientrects-001.html](https://wpt.fyi/results/css/css-multicol/getclientrects-001.html) [(live test)](http://wpt.live/css/css-multicol/getclientrects-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/getclientrects-001.html)

------------------------------------------------------------------------

## <a id="filling-columns"></a>7.  Filling Columns

<a id="ref-for-propdef-widows"></a>

<a id="ref-for-propdef-orphans"></a>

There are two strategies for filling columns: columns can either be balanced, or not. If columns are balanced, user agents should try to minimize variations in column height, while honoring forced breaks, [widows](https://www.w3.org/TR/css-break-4/#propdef-widows) and [orphans](https://www.w3.org/TR/css-break-3/#propdef-orphans), and other properties that may affect column heights. If columns are not balanced, they are filled sequentially; some columns may end up partially filled, or with no content at all.

<a id="ref-for-propdef-column-fill①"></a>

### <a id="cf"></a>7.1. Column Balancing: the [column-fill](#propdef-column-fill) property

<strong>Table 9 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-column-fill"></a>column-fill

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one③"></a>

auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) balance <a id="ref-for-comb-one④"></a>\| balance-all

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

balance

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

multicol containers

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

N/A

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified keyword

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

<a id="ref-for-multi-column-line⑧"></a>

<a id="ref-for-multi-column-spanner④"></a>

This property specifies whether content in a [multi-column line](#multi-column-line) that does <em>not</em> immediately precede a [spanner](#multi-column-spanner) is balanced across columns or not.

Tests

- [always-balancing-before-column-span.html](https://wpt.fyi/results/css/css-multicol/always-balancing-before-column-span.html) [(live test)](http://wpt.live/css/css-multicol/always-balancing-before-column-span.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/always-balancing-before-column-span.html)
- [no-balancing-after-column-span.html](https://wpt.fyi/results/css/css-multicol/no-balancing-after-column-span.html) [(live test)](http://wpt.live/css/css-multicol/no-balancing-after-column-span.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/no-balancing-after-column-span.html)

The values are:

<a id="valdef-column-fill-balance"></a>balance  
Balance content equally between columns, as far as possible. In fragmented contexts, only the last fragment is balanced.

Tests

- [multicol-fill-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-fill-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-000.xht)
- [multicol-fill-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-fill-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-001.xht)
- [column-fill-invalid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-fill-invalid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-fill-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-fill-invalid.html)
- [column-fill-valid.html](https://wpt.fyi/results/css/css-multicol/parsing/column-fill-valid.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-fill-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-fill-valid.html)
- [column-fill-computed.html](https://wpt.fyi/results/css/css-multicol/parsing/column-fill-computed.html) [(live test)](http://wpt.live/css/css-multicol/parsing/column-fill-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/parsing/column-fill-computed.html)
- [column-fill-balance-orthog-block-001.html](https://wpt.fyi/results/css/css-multicol/column-fill-balance-orthog-block-001.html) [(live test)](http://wpt.live/css/css-multicol/column-fill-balance-orthog-block-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/column-fill-balance-orthog-block-001.html)
- [column-balancing-paged-001-print.html](https://wpt.fyi/results/css/css-multicol/column-balancing-paged-001-print.html) [(live test)](http://wpt.live/css/css-multicol/column-balancing-paged-001-print.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/column-balancing-paged-001-print.html)
- [multicol-fill-balance-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-001.xht)
- [multicol-fill-balance-002.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-002.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-002.html)
- [multicol-fill-balance-003.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-003.html)
- [multicol-fill-balance-004.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-004.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-004.html)
- [multicol-fill-balance-005.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-005.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-005.html)
- [multicol-fill-balance-006.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-006.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-006.html)
- [multicol-fill-balance-007.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-007.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-007.html)
- [multicol-fill-balance-008.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-008.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-008.html)
- [multicol-fill-balance-009.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-009.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-009.html)
- [multicol-fill-balance-010.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-010.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-010.html)
- [multicol-fill-balance-011.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-011.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-011.html)
- [multicol-fill-balance-012.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-012.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-012.html)
- [multicol-fill-balance-013.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-013.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-013.html)
- [multicol-fill-balance-014.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-014.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-014.html)
- [multicol-fill-balance-015.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-015.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-015.html)
- [multicol-fill-balance-016.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-016.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-016.html)
- [multicol-fill-balance-018.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-018.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-018.html)
- [multicol-fill-balance-019.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-019.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-019.html)
- [multicol-fill-balance-020.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-020.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-020.html)
- [multicol-fill-balance-021.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-021.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-021.html)
- [multicol-fill-balance-022.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-022.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-022.html)
- [multicol-fill-balance-023.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-023.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-023.html)
- [multicol-fill-balance-024.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-024.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-024.html)
- [multicol-fill-balance-025.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-025.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-025.html)
- [multicol-fill-balance-026.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-026.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-026.html)
- [multicol-fill-balance-027.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-027.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-027.html)
- [multicol-fill-balance-028.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-028.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-028.html)
- [multicol-fill-balance-nested-000.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-balance-nested-000.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-balance-nested-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-balance-nested-000.html)

<a id="valdef-column-fill-balance-all"></a>balance-all  
Balance content equally between columns, as far as possible. In fragmented contexts, all fragments are balanced.

<a id="valdef-column-fill-auto"></a>auto  
fill columns sequentially

Tests

- [multicol-fill-auto-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-fill-auto-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-auto-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-auto-001.xht)
- [multicol-fill-auto-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-fill-auto-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-auto-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-auto-002.xht)
- [multicol-fill-auto-003.xht](https://wpt.fyi/results/css/css-multicol/multicol-fill-auto-003.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-auto-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-auto-003.xht)
- [multicol-fill-auto-004.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-auto-004.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-auto-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-auto-004.html)
- [multicol-fill-auto-block-children-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-fill-auto-block-children-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-auto-block-children-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-auto-block-children-001.xht)
- [multicol-fill-auto-block-children-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-fill-auto-block-children-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-auto-block-children-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-auto-block-children-002.xht)
- [multicol-fill-auto-block-children-003.html](https://wpt.fyi/results/css/css-multicol/multicol-fill-auto-block-children-003.html) [(live test)](http://wpt.live/css/css-multicol/multicol-fill-auto-block-children-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-fill-auto-block-children-003.html)
- [columnfill-auto-max-height-001.html](https://wpt.fyi/results/css/css-multicol/columnfill-auto-max-height-001.html) [(live test)](http://wpt.live/css/css-multicol/columnfill-auto-max-height-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/columnfill-auto-max-height-001.html)
- [columnfill-auto-max-height-002.html](https://wpt.fyi/results/css/css-multicol/columnfill-auto-max-height-002.html) [(live test)](http://wpt.live/css/css-multicol/columnfill-auto-max-height-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/columnfill-auto-max-height-002.html)
- [columnfill-auto-max-height-003.html](https://wpt.fyi/results/css/css-multicol/columnfill-auto-max-height-003.html) [(live test)](http://wpt.live/css/css-multicol/columnfill-auto-max-height-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/columnfill-auto-max-height-003.html)

In continuous contexts, this property does not have any effect when there are overflow columns.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0143ee45"></a> In this example, an article only has one short paragraph which fits on three lines. The three lines are displayed in three different columns due to column balancing.
>
> ```text
> article {
>   width: 60em;
>   height: auto;
>   columns: 4;
>   column-fill: balance;
> }
> ```
>
> ![Four columns, the first three have content.](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/column-balancing-one-paragraph.svg)
>
> Three lines displayed in three columns due to column balancing.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8e97de73"></a> In this example, column balancing is turned off, and the article has a height:
>
> ```text
> article {
>   width: 60em;
>   height: 4em;
>   columns: 4;
>   column-fill: auto;
> }
> ```
>
> As a result, the first column is filled with all content:
>
> ![Four columns, the first one has content.](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/no-column-balancing-one-paragraph.svg)
>
> No balancing so the whole text is shown in one paragraph.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0c7abc2c"></a> In this example, an article has two paragraphs: first a long one, then a shorter one. This code is applied:
>
> ```text
> article {
>   width: 60em;
>   height: auto;
>   columns: 4;
>   column-fill: balance;
> }
> 
> p {
>   break-after: column;
> }
> ```
>
> The shortest column height possible contains five lines of text. After the column height has been established, columns are filled sequentially. As a result, the third column is as high as the first two columns, while the last column ends up being significantly shorter.
>
> ![Four columns, all have content.](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/column-balancing-with-column-break.svg)
>
> Once column height is established, columns are filled sequentially.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3b949a54"></a>
>
> ```text
> article {
>   width: 60em;
>   height: auto;
>   columns: 4;
>   column-fill: balance;
> }
> ```
>
> In this example, an article starts with an unbreakable figure which sets the column height. Subsequent content is filled sequentially into the remaining columns:
>
> ![Column one contains an image, two and three have content.](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/column-balancing-with-figure.svg)
>
> Column height is established by the figure.

Tests

Tests relating to column balancing in combination with out-of-flow elements.

- [column-balancing-with-span-and-oof-001.html](https://wpt.fyi/results/css/css-multicol/column-balancing-with-span-and-oof-001.html) [(live test)](http://wpt.live/css/css-multicol/column-balancing-with-span-and-oof-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/column-balancing-with-span-and-oof-001.html)
- [column-balancing-with-span-and-oof-002.html](https://wpt.fyi/results/css/css-multicol/column-balancing-with-span-and-oof-002.html) [(live test)](http://wpt.live/css/css-multicol/column-balancing-with-span-and-oof-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/column-balancing-with-span-and-oof-002.html)
- [forced-break-in-oof-in-column-balancing-nested.html](https://wpt.fyi/results/css/css-multicol/crashtests/forced-break-in-oof-in-column-balancing-nested.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/forced-break-in-oof-in-column-balancing-nested.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/forced-break-in-oof-in-column-balancing-nested.html)
- [forced-break-in-oof-in-column-balancing.html](https://wpt.fyi/results/css/css-multicol/crashtests/forced-break-in-oof-in-column-balancing.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/forced-break-in-oof-in-column-balancing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/forced-break-in-oof-in-column-balancing.html)

------------------------------------------------------------------------

Other tests relating to balancing

- [balance-extremely-tall-monolithic-content-crash.html](https://wpt.fyi/results/css/css-multicol/balance-extremely-tall-monolithic-content-crash.html) [(live test)](http://wpt.live/css/css-multicol/balance-extremely-tall-monolithic-content-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/balance-extremely-tall-monolithic-content-crash.html)
- [column-balancing-with-overflow-auto-crash.html](https://wpt.fyi/results/css/css-multicol/column-balancing-with-overflow-auto-crash.html) [(live test)](http://wpt.live/css/css-multicol/column-balancing-with-overflow-auto-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/column-balancing-with-overflow-auto-crash.html)
- [balance-break-avoidance-000.html](https://wpt.fyi/results/css/css-multicol/balance-break-avoidance-000.html) [(live test)](http://wpt.live/css/css-multicol/balance-break-avoidance-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/balance-break-avoidance-000.html)
- [balance-break-avoidance-001.html](https://wpt.fyi/results/css/css-multicol/balance-break-avoidance-001.html) [(live test)](http://wpt.live/css/css-multicol/balance-break-avoidance-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/balance-break-avoidance-001.html)
- [balance-break-avoidance-002.html](https://wpt.fyi/results/css/css-multicol/balance-break-avoidance-002.html) [(live test)](http://wpt.live/css/css-multicol/balance-break-avoidance-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/balance-break-avoidance-002.html)
- [balance-grid-container.html](https://wpt.fyi/results/css/css-multicol/balance-grid-container.html) [(live test)](http://wpt.live/css/css-multicol/balance-grid-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/balance-grid-container.html)
- [balance-orphans-widows-000.html](https://wpt.fyi/results/css/css-multicol/balance-orphans-widows-000.html) [(live test)](http://wpt.live/css/css-multicol/balance-orphans-widows-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/balance-orphans-widows-000.html)
- [balance-with-forced-break.html](https://wpt.fyi/results/css/css-multicol/crashtests/balance-with-forced-break.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/balance-with-forced-break.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/balance-with-forced-break.html)
- [balancing-flex-item-trailing-margin-freeze.html](https://wpt.fyi/results/css/css-multicol/crashtests/balancing-flex-item-trailing-margin-freeze.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/balancing-flex-item-trailing-margin-freeze.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/balancing-flex-item-trailing-margin-freeze.html)
- [balancing-tall-borders-freeze.html](https://wpt.fyi/results/css/css-multicol/crashtests/balancing-tall-borders-freeze.html) [(live test)](http://wpt.live/css/css-multicol/crashtests/balancing-tall-borders-freeze.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/crashtests/balancing-tall-borders-freeze.html)

------------------------------------------------------------------------

## <a id="overflow"></a>8.  Overflow

### <a id="overflow-inside-multicol-elements"></a>8.1.  Overflow Inside Multicol Containers

<a id="ref-for-column-box⑦"></a>

Except for cases where this would cause a column break, content that extends outside column boxes visibly overflows and is not clipped to the [column box](#column-box).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: See [§ 5 Column Breaks](#column-breaks) for column breaks and [§ 8.2 Pagination and Overflow Outside Multicol Containers](#pagination-and-overflow-outside-multicol) for whether it is clipped to the multi-column container’s content box.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-276e0e20"></a> In this example, the image is wider than the column:
>
> ![An imagine in the first column has visible overflow](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/image-overflow-not-clipped.svg)
>
> Content visibly overflows and is not clipped to the column box.

Tests

- [multicol-block-no-clip-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-block-no-clip-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-block-no-clip-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-block-no-clip-001.xht)
- [multicol-block-no-clip-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-block-no-clip-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-block-no-clip-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-block-no-clip-002.xht)
- [multicol-clip-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-clip-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-clip-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-clip-001.xht)
- [multicol-clip-002.xht](https://wpt.fyi/results/css/css-multicol/multicol-clip-002.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-clip-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-clip-002.xht)
- [multicol-clip-scrolled-content-001.html](https://wpt.fyi/results/css/css-multicol/multicol-clip-scrolled-content-001.html) [(live test)](http://wpt.live/css/css-multicol/multicol-clip-scrolled-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-clip-scrolled-content-001.html)
- [multicol-overflow-clip-auto-sized.html](https://wpt.fyi/results/css/css-multicol/multicol-overflow-clip-auto-sized.html) [(live test)](http://wpt.live/css/css-multicol/multicol-overflow-clip-auto-sized.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-overflow-clip-auto-sized.html)
- [multicol-overflow-clip-positioned.html](https://wpt.fyi/results/css/css-multicol/multicol-overflow-clip-positioned.html) [(live test)](http://wpt.live/css/css-multicol/multicol-overflow-clip-positioned.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-overflow-clip-positioned.html)
- [multicol-overflow-clip.html](https://wpt.fyi/results/css/css-multicol/multicol-overflow-clip.html) [(live test)](http://wpt.live/css/css-multicol/multicol-overflow-clip.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-overflow-clip.html)
- [relative-child-overflowing-column-gap.html](https://wpt.fyi/results/css/css-multicol/relative-child-overflowing-column-gap.html) [(live test)](http://wpt.live/css/css-multicol/relative-child-overflowing-column-gap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/relative-child-overflowing-column-gap.html)
- [relative-child-overflowing-container.html](https://wpt.fyi/results/css/css-multicol/relative-child-overflowing-container.html) [(live test)](http://wpt.live/css/css-multicol/relative-child-overflowing-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/relative-child-overflowing-container.html)

### <a id="pagination-and-overflow-outside-multicol"></a>8.2.  Pagination and Overflow Outside Multicol Containers

<a id="ref-for-propdef-overflow"></a>

Content and column rules that extend outside column boxes at the edges of the multi-column container are clipped according to the [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) property.

A multicol container can have more columns than it has room for due to:

- <a id="ref-for-propdef-max-height"></a>

  <a id="ref-for-propdef-height"></a>

  a declaration that constrains the column height (e.g., using [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) or [max-height](https://www.w3.org/TR/css-sizing-3/#propdef-max-height)). In this case, additional column boxes are created in the inline direction

- the size of the page. In this case, additional column boxes are moved to the next page(s).

- explicit column breaks. In this case, additional column boxes are created in the inline direction for continuous contexts and additional column boxes are moved to the next fragment(s) for fragmented media.

- 

Tests

- [multicol-overflow-000.xht](https://wpt.fyi/results/css/css-multicol/multicol-overflow-000.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-overflow-000.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-overflow-000.xht)
- [multicol-overflowing-001.xht](https://wpt.fyi/results/css/css-multicol/multicol-overflowing-001.xht) [(live test)](http://wpt.live/css/css-multicol/multicol-overflowing-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/multicol-overflowing-001.xht)

Tests to check that a scrollable container isn't split across multiple columns.

- [overflow-unsplittable-001.html](https://wpt.fyi/results/css/css-multicol/overflow-unsplittable-001.html) [(live test)](http://wpt.live/css/css-multicol/overflow-unsplittable-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/overflow-unsplittable-001.html)
- [overflow-unsplittable-002.html](https://wpt.fyi/results/css/css-multicol/overflow-unsplittable-002.html) [(live test)](http://wpt.live/css/css-multicol/overflow-unsplittable-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/overflow-unsplittable-002.html)
- [overflow-unsplittable-003.html](https://wpt.fyi/results/css/css-multicol/overflow-unsplittable-003.html) [(live test)](http://wpt.live/css/css-multicol/overflow-unsplittable-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-multicol/overflow-unsplittable-003.html)

------------------------------------------------------------------------

Columns that appear outside the multicol container in continuous contexts are called <a id="overflow-columns"></a>overflow columns. Overflow columns can affect the height of the multicol container.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a5e26f6e"></a> In this example, the height of the multi-column container has been constrained to a maximum height. Also, the style sheet specifies that overflowing content should be visible:
>
> ```text
> div {
>   max-height: 5em;
>   overflow: visible;
> }
> ```
>
> As a result, the number of columns is increased.
>
> ![Four columns, one outside the multicol container](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/height-constraint-overflow-inline.svg)
>
> An overflow column is created in the inline direction.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6e5c350e"></a>
>
> In continuous contexts overflow columns can affect the height of the multicol container. In this example a column appears in the overflow which has four lines of text. The multicol container is made tall enough to accommodate this column.
>
> ![Four columns, overflow column is taller than the first three](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/overflow-column-effects-height.svg)
>
> The final column is an overflow column yet is taller than the others. The container is tall enough for this column.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a048a782"></a> In fragmented contexts, the overflow content goes into columns in subsequent fragments. Given the same content as in example 31 and a page box that only has room for five lines of formatted text, this would appear on the first page:
>
> ![Three columns](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/pagination-overflow-page1.svg)
>
> The first three paragraphs appear on page one.
>
> Assuming column balancing, this would appear on the second page:
>
> ![Three columns](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/pagination-overflow-page2.svg)
>
> The overflow column is moved onto page two.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-da1d960a"></a> In this example, explicit column breaks are generated after paragraphs:
>
> ```text
> p {
>   break-after: column;
> }
> ```
>
> As a result, the number of columns increases and the extra columns are added in the inline direction:
>
> ![Four columns, one outside the multicol container](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/height-constraint-column-break-overflow-inline.svg)
>
> An overflow column is created in the inline direction.

<a id="ref-for-paged-media②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-02cf1ca4"></a> In [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media), extra columns are shown on the next page. Given the same code as the previous example, the last paragraph appears on the second page. This would appear on the first page:
>
> ![Three columns](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/pagination-column-break-overflow-page1.svg)
>
> The first three paragraphs appear on page one.
>
> This would appear on the second page:
>
> ![Three columns](https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/images/pagination-column-break-overflow-page2.svg)
>
> The overflow column is moved onto page two.
>
> Due to column balancing, the last paragraph is split across three columns.

## <a id="changes"></a>Appendix B. Changes

This appendix is <em>informative</em>.

### <a id="changes-from-20211012"></a>Changes from the [Candidate Recommendation (CR) of 12 October 2021](https://www.w3.org/TR/2021/CR-css-multicol-1-20211012/)

- <a id="ref-for-multi-column-container②②"></a>

  Added the text "and there is nothing between the spanning element and [multicol container](#multi-column-container) that establishes a containing block for fixed position descendants." [Resolved 9 Mar 2022](https://github.com/w3c/csswg-drafts/issues/6805#issuecomment-1063159219)

### <a id="changes-from-20210212"></a>Changes from the [Working Draft (WD) of 12 February 2021](https://www.w3.org/TR/2021/WD-css-multicol-1-20210212/)

- Added the text "Out-of-flow descendants of a multi-column container do affect column balancing, and the block-size of the multi-column container." [Resolved 12 May 2021](https://github.com/w3c/csswg-drafts/issues/6279#issuecomment-839912199)
- Added the text, "The margins of two spanners separated only by an absolutely positioned item will collapse with each other, as absolutely positioned items do not create column boxes." [Resolved 12 May 2021](https://github.com/w3c/csswg-drafts/issues/6265#issuecomment-839909999)
- Adds an accessibility considerations section. [Request from APA after review.](https://github.com/w3c/csswg-drafts/issues/6283)
- Adds a note and example to make clear the manner in which columns are laid out in vertical writing modes. [Request from i18n WG after review.](https://github.com/w3c/csswg-drafts/issues/6301)

### <a id="changes-from-20191015"></a>Changes from the [Working Draft (WD) of 15 October 2019](https://www.w3.org/TR/2019/WD-css-multicol-1-20191015/)

- Added the text "The spanner becomes the containing block for absolutely positioned boxes inside the spanner where the spanning element establishes a containing block, otherwise the containing block chain goes to the multicol container.". [Resolved 23 Oct 2020](https://github.com/w3c/csswg-drafts/issues/5612#issuecomment-715369246)
- Adding the text "This property specifies whether content in a multi-column line that does <em>not</em> immediately precede a spanner is balanced across columns or not." [Resolved 29 April 2020](https://github.com/w3c/csswg-drafts/issues/4689#issuecomment-621281467)

### <a id="changes-from-20180528"></a>Changes from the [Working Draft (WD) of 28 May 2018](https://www.w3.org/TR/2018/WD-css-multicol-1-20180528/)

- Removed the non-normative text "However, as described below, setting both the width and number of columns rarely makes sense." Editorial 16 Sep 2019, [issue 4291](https://github.com/w3c/csswg-drafts/issues/4291).

- <a id="ref-for-block-formatting-context⑤"></a>

  Added the paragraph, "Spanners are block-level boxes therefore the margins of two adjacent spanners will collapse with each other. As column boxes establish a new [block formatting context](https://www.w3.org/TR/css-display-3/#block-formatting-context), margins on elements inside a column box will not collapse with the margin of a spanner." [Resolved 22 Oct 2018](https://github.com/w3c/csswg-drafts/issues/2203#issuecomment-431783027), see also the resolution for [issue 2582](https://github.com/w3c/csswg-drafts/issues/2582#issuecomment-402619872).

- Clarified the spec to explain that a spanning element is taken out-of-flow, leaving a forced break. Added the paragraph, "A spanning element is taken out-of-flow, leaving a forced break. This does not affect the painting order of the spanning element." [Resolved 28 Feb 2019](https://github.com/w3c/csswg-drafts/issues/1072#issuecomment-468087733).

- <a id="ref-for-propdef-column-gap③"></a>

  Moved the definition of the [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap) property to [\[CSS3-ALIGN\]](#biblio-css3-align) and added a paragraph detailing the specifics of <a id="ref-for-propdef-column-gap④"></a>column-gap in multicol:  
  "In a multi-column formatting context the used value of normal for the column-gap property is 1em. This ensures columns are readable when the initial values are used. If there is a column rule between columns, it will appear in the middle of the gap." [Resolved 4 June 2019.](https://github.com/w3c/csswg-drafts/issues/3641)

- Reworded the section [The multi-column model](#the-multi-column-model), based on input from Morten Stenshorne in [issue 2203](https://github.com/w3c/csswg-drafts/issues/2203#issuecomment-431695940).

- <a id="ref-for-propdef-column-gap⑤"></a>

  Removed the at-risk marker from the length-percentage value for [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap). [Resolved 4 June 2019.](https://github.com/w3c/csswg-drafts/issues/3988)

- Updated the introduction to remove mention of the benefits of multicol over using tables for layout and instead refer to the unique characteristics of multicol. Editorial change referenced in [issue 3654](https://github.com/w3c/csswg-drafts/issues/3654).

- Changed the sentence added in the pseudo-algorithm section after the 7 Jan 2016 resolution, to refer to <em>columns</em> and not <em>tracks</em> as tracks are not defined in this specification. Resolved [13 March 2019](https://github.com/w3c/csswg-drafts/issues/3649#issuecomment-472505520).

- Changes and clarifications to the SVG images used in the specification.

- Changed syntax to use bracketed range notation to reflect the prose restrictions on negative/non-zero values.

### <a id="changes-from-20171005"></a>Changes from the [Working Draft (WD) of 5 October 2017](https://www.w3.org/TR/2017/WD-css-multicol-1-20171005/)

- Changed references to paged media to refer to fragmented contexts. Resolved [12 Apr 2018](https://github.com/w3c/csswg-drafts/issues/1746#issuecomment-380731574).

- Changed a line regarding the `column-fill` property:  
  “In continuous media, this property does not have any effect in overflow columns.”  
  To:  
  “In continuous media, this property does not have any effect when there are overflow columns.” [Resolved: 12 Apr 2018](https://github.com/w3c/csswg-drafts/issues/2549)

- Add a line of text plus an example to show that overflow columns can affect the multicol container height. [Resolved: 12 Apr 2018](https://github.com/w3c/csswg-drafts/issues/1745)

- Replaced the HTML mock-up examples with SVG versions, as the examples were unclear. [Issue 1087](https://github.com/w3c/csswg-drafts/issues/1087).

- Changed the value of normal for column-gap to be 1em, rather than a UA-specified length with a suggestion of 1em. [Resolved: 4 Apr 2018](https://github.com/w3c/csswg-drafts/issues/2145#issuecomment-378781507)

- Clarified that negative values are not allowed for column-width, and that while 0 may be specified, used values will be clamped to a minimum of 1px. [Resolved: 14 Mar 2018](https://github.com/w3c/csswg-drafts/issues/1741#issuecomment-373091628)

- Clarified that where there is a spanning element content is automatically balanced across all columns in the immediately preceding column row before the element appears. [Resolved: 9 Nov 2017](https://github.com/w3c/csswg-drafts/issues/1075)

- Added clarification plus an additional example that spanning elements may be lower the first level of descendants, and that in the case of margins, borders and padding on the element containing the spanning, this would be drawn above the spanner. [Resolved: 8 Nov 2017](https://github.com/w3c/csswg-drafts/issues/1072#issuecomment-342668025)

- Changed the sentence “Column rules are painted in the inline content layer, but below all inline content inside the multicol element.” to “Column rules are painted just above the border of the multicol element. For scrollable multicol elements, note that while the border and background of the multicol element obviously aren’t scrolled, the rules need to scroll along with the columns.” [Resolved: 7 Nov 2017](https://github.com/w3c/csswg-drafts/issues/1739#issuecomment-342659978)

- <a id="ref-for-propdef-position③"></a>

  Under section The Multi-column Model, removed two sentences “That is, column boxes behave like block-level, table cell, and inline-block boxes as per CSS 2.1, section 10.1, item 2 CSS21. However, column boxes do not establish block container boxes for elements with [position: fixed or position: absolute](https://www.w3.org/TR/css-position-3/#propdef-position).”. These were replaced with a clarification about the principal box and a new example showing how abspos elements refer to the multicol container. [Resolved: 7 Nov 2017](https://github.com/w3c/csswg-drafts/issues/1738#issuecomment-342661881)

- Removed the sentence "To indicate where column breaks should (or should not) appear, new keyword values are introduced." and following example (Example 7 in the WD published [5 Oct 2017](https://www.w3.org/TR/2017/WD-css-multicol-1-20171005/)) as the multicol specification no longer introduces these properties. [Editorial](https://github.com/w3c/csswg-drafts/issues/1966)

- Changed how we reference the element we have applied multicol to from multi-column or multicol “element” to multi-column or multicol “container”. [Resolved: 22 November 2017](https://github.com/w3c/csswg-drafts/issues/1965)

- Removed the example which stated "If a tall image is moved to a column on the next page to find room for it, its natural column may be left empty. If so, the column is still considered to have content for the purpose of deciding if the column rule should be drawn." [Resolved: 7 September 2017](https://github.com/w3c/csswg-drafts/issues/1740)

### <a id="changes-from-20110412"></a>Changes from the [Candidate Recommendation (CR) of 12 April 2011](https://www.w3.org/TR/2011/CR-css3-multicol-20110412/).

- Added July 2016 resolution to change the track size floor to a required UA-specified value, consistent with the CSS Grid spec. [Resolved: 7 Jan 2016](https://lists.w3.org/Archives/Public/www-style/2016Jan/0031.html)

- <a id="ref-for-propdef-column-fill②"></a>

  Remove the restriction about overflow columns only being in continuous media in the statement that [column-fill](#propdef-column-fill) has no effect on overflow columns. [Resolved: September 2013](https://lists.w3.org/Archives/Public/www-style/2013Sep/0471.html).

- Added keyword balance-all and examples to demonstrate how this should work. [Resolved: September 2013](https://lists.w3.org/Archives/Public/www-style/2013Sep/0471.html).

- The pseudo-algorithm has been revised on a number of occasions. [Latest change Feb 2013](https://lists.w3.org/Archives/Public/www-style/2013Feb/0471.html).

- <a id="ref-for-propdef-column-count①②"></a>

  <a id="ref-for-propdef-column-width①④"></a>

  <a id="ref-for-propdef-columns④"></a>

  Clarified that properties [columns](#propdef-columns), [column-width](#propdef-column-width), [column-count](#propdef-column-count) "apply to block containers". [Ref: Feb 2013](https://lists.w3.org/Archives/Public/www-style/2013Feb/0536.html).

- Breaking properties have been moved from this specification to the [CSS Fragmentation Module](https://www.w3.org/TR/css-break-3/).

- <a id="ref-for-propdef-column-fill③"></a>

  Change to [column-fill](#propdef-column-fill) wording to clarify that <a id="ref-for-propdef-column-fill④"></a>column-fill is honored before page breaks. [Ref: Jan 2012](https://lists.w3.org/Archives/Public/www-style/2012Jan/0393.html).

- Amended example and text to clarify what happens with margin collapsing and spanning elements. [Ref: Oct 2013](https://lists.w3.org/Archives/Public/www-style/2013Oct/0247.html).

- <a id="ref-for-propdef-column-rule-width④"></a>

  Clarification that [column-rule-width](#propdef-column-rule-width) does not alter the size or placement of columns. [Ref: Sep 2013](https://lists.w3.org/Archives/Public/www-style/2013Sep/0550.html).

- Added that each column spanning element establishes a separate BFC margins between them collapse. [Ref: Dec 2011](https://lists.w3.org/Archives/Public/www-style/2011Dec/0262.html).

- Column rules are painted in the inline content layer, but below all inline content inside the multicol. [Ref: Feb 2013](https://lists.w3.org/Archives/Public/www-style/2013Feb/0363.html).

- <a id="ref-for-propdef-column-span⑧"></a>

  Clarify that [column-span](#propdef-column-span) causes the element to establish a formatting context even if it is not in a multicol.

- Column spanners do not always establish a <em>block</em> formatting context.

- <a id="ref-for-length-value②"></a>

  <a id="ref-for-typedef-length-percentage"></a>

  <a id="ref-for-propdef-column-gap⑥"></a>

  Allow [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap) to accept [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) instead of just [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

- <a id="ref-for-table-wrapper-box②"></a>

  <a id="ref-for-block-container③"></a>

  <a id="ref-for-propdef-column-count①③"></a>

  <a id="ref-for-propdef-column-width①⑤"></a>

  [column-width](#propdef-column-width) and [column-count](#propdef-column-count) applies to [block containers](https://www.w3.org/TR/css-display-3/#block-container) except [table wrapper boxes](https://www.w3.org/TR/css-tables-3/#table-wrapper-box).

- Content that overflows columns is not clipped.

## <a id="privacy"></a>Privacy Considerations

Multicol introduces no new privacy leaks.

## <a id="security"></a>Security Considerations

Multicol introduces no new security considerations.

## <a id="a11y"></a>Accessibility Considerations

Setting container height and line length can pose challenges for people with visual or cognitive disabilities. See [WCAG Success Criterion 1.4.10 Reflow](https://www.w3.org/WAI/WCAG21/Understanding/reflow.html) and [WCAG 1.4.8 Visual Presentation](https://www.w3.org/WAI/WCAG21/quickref/#visual-presentation) to understand user needs.

## <a id="acknowledgments"></a> Acknowledgments

This document is based on several older proposals and comments on older proposals. Contributors include:

Alex Mogilevsky, Andy Clarke, Anton Prowse, Bert Bos, Björn Höhrmann, Cédric Savarese, Chris Lilley, Chris Wilson, Daniel Glazman and Dave Raggett, David Hyatt, David Singer, David Woolley, Elika Etemad, Giovanni Campagna, Ian Hickson. Joost de Valk, Kevin Lawver, L. David Baron, Markus Mielke, Melinda Grant, Michael Day, Morten Stenshorne, Øyvind Stenhaug, Peter Linss, Peter-Paul Koch, Robert O’Callahan, Robert Stevahn, Sergey Genkin, Shelby Moore, Steve Zilles, Sylvain Galineau, Tantek Çelik, Till Halbach

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

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with `<strong class="advisement">`, like this: <strong data-conversion-semantic="advisement">Advisement:</strong> <strong> UAs MUST provide an accessible alternative. </strong>

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

### <a id="w3c-cr-exit-criteria"></a> CR exit criteria

For this specification to be advanced to Proposed Recommendation, there must be at least two independent, interoperable implementations of each feature. Each feature may be implemented by a different set of products, there is no requirement that all features be implemented by a single product. For the purposes of this criterion, we define the following terms:

independent  
each implementation must be developed by a different party and cannot share, reuse, or derive from code used by another qualifying implementation. Sections of code that have no bearing on the implementation of this specification are exempt from this requirement.

interoperable  
passing the respective test case(s) in the official CSS test suite, or, if the implementation is not a Web browser, an equivalent test. Every relevant test in the test suite should have an equivalent test created if such a user agent (UA) is to be used to claim interoperability. In addition if such a UA is to be used to claim interoperability, then there must one or more additional UAs which can also pass those equivalent tests in the same way for the purpose of interoperability. The equivalent tests must be made publicly available for the purposes of peer review.

implementation  
a user agent which:

1.  implements the specification.
2.  is available to the general public. The implementation may be a shipping product or other publicly available version (i.e., beta version, preview release, or "nightly build"). Non-shipping product releases must have implemented the feature(s) for a period of at least one month in order to demonstrate stability.
3.  is not experimental (i.e., a version specifically designed to pass the test suite and is not intended for normal usage going forward).

The specification will remain Candidate Recommendation for at least six months.

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [all](#valdef-column-span-all), in § 6.1
- auto
  - [value for column-count](#valdef-column-count-auto), in § 3.2
  - [value for column-fill](#valdef-column-fill-auto), in § 7.1
  - [value for column-width](#valdef-column-width-auto), in § 3.1
- [balance](#valdef-column-fill-balance), in § 7.1
- [balance-all](#valdef-column-fill-balance-all), in § 7.1
- [\<color\>](#valdef-column-rule-color-color), in § 4.2
- [column](#column-box), in § 2
- [column box](#column-box), in § 2
- [column-count](#propdef-column-count), in § 3.2
- [column-fill](#propdef-column-fill), in § 7.1
- [column gap](#column-gap), in § 2
- [column height](#column-height), in § 2
- [column rule](#column-rule), in § 2
- [column-rule](#propdef-column-rule), in § 4.5
- [column-rule-color](#propdef-column-rule-color), in § 4.2
- [column-rule-style](#propdef-column-rule-style), in § 4.3
- [column-rule-width](#propdef-column-rule-width), in § 4.4
- [columns](#propdef-columns), in § 3.3
- [column-span](#propdef-column-span), in § 6.1
- [column width](#column-width), in § 2
- [column-width](#propdef-column-width), in § 3.1
- [\<integer \[1,∞\]\>](#valdef-column-count-integer-1), in § 3.2
- [\<length \[0,∞\]\>](#valdef-column-width-length-0), in § 3.1
- [multicol container](#multi-column-container), in § 2
- [multi-col line](#multi-column-line), in § 2
- [multicol line](#multi-column-line), in § 2
- [multi-column container](#multi-column-container), in § 2
- [multi-column formatting context](#multi-column-formatting-context), in § 2
- [multi-column layout](#multi-column-layout), in § 1
- [multi-column line](#multi-column-line), in § 2
- [multi-column spanner](#multi-column-spanner), in § 6.1
- [multi-column spanning element](#multi-column-spanning-element), in § 6.1
- [none](#valdef-column-span-none), in § 6.1
- [overflow columns](#overflow-columns), in § 8.2
- [spanner](#multi-column-spanner), in § 6.1
- [spanning element](#multi-column-spanning-element), in § 6.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="ffeb15ad"></a>\<line-style\>
  - <a id="5747d295"></a>\<line-width\>
  - <a id="e1674793"></a>border
  - <a id="212a3293"></a>hidden
  - <a id="1c96a88c"></a>none
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="64bdec0d"></a>fragment
  - <a id="04004305"></a>fragmentation
  - <a id="4904f647"></a>fragmentation container
  - <a id="7eb0e25a"></a>fragmentation context
  - <a id="54f91f87"></a>widows
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="d04b6986"></a>\<color\>
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="61c422b4"></a>anonymous box
  - <a id="05c40e8e"></a>block container
  - <a id="e4f1fc8b"></a>block formatting context
  - <a id="696dc9f0"></a>block-level box
  - <a id="6b4fc208"></a>containing block
  - <a id="c09ea731"></a>containing block chain
  - <a id="2ccfe434"></a>display
  - <a id="43fd67c9"></a>formatting context
  - <a id="b091c3a0"></a>independent formatting context
  - <a id="ffcb5356"></a>out of flow
  - <a id="06fd3b4c"></a>out-of-flow
  - <a id="93f98063"></a>principal box
- \[CSS-GRID-2\] defines the following terms:
  - <a id="df72a52c"></a>grid container
  - <a id="ba30fc9a"></a>grid item
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="add377f4"></a>overflow
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="dec20430"></a>absolutely positioned box
  - <a id="b8c34db8"></a>position
- \[CSS-TABLES-3\] defines the following terms:
  - <a id="1b178ec1"></a>table wrapper box
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="e7c6bf78"></a>transform
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="d73c993d"></a>\<integer\>
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="8a110a7b"></a>css-wide keywords
  - <a id="4f460096"></a>snap as a border width
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="9933fc3f"></a>inline base direction
- \[CSS3-ALIGN\] defines the following terms:
  - <a id="b7d152c3"></a>column-gap
  - <a id="e975f960"></a>normal
- \[CSS3-BREAK\] defines the following terms:
  - <a id="51ee3396"></a>break-after
  - <a id="eb306f02"></a>break-before
  - <a id="8ae583f9"></a>break-inside
  - <a id="4f75e4ec"></a>orphans
- \[CSS3-SIZING\] defines the following terms:
  - <a id="5ad01cca"></a>height
  - <a id="2d68423f"></a>max-height
  - <a id="49731d1d"></a>width
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="23af89d0"></a>paged media

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; et al. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 29 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-5&#x2F;](https://www.w3.org/TR/css-color-5/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; Elika Etemad; Rossen Atanassov. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 29 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 3 April 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-tables-3"></a>\[CSS-TABLES-3\]  
François Remy; Greg Whitworth; David Baron. [CSS Table Module Level 3](https://www.w3.org/TR/css-tables-3/). 27 July 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-tables-3&#x2F;](https://www.w3.org/TR/css-tables-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css3-align"></a>\[CSS3-ALIGN\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 17 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css3-break"></a>\[CSS3-BREAK\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css3-sizing"></a>\[CSS3-SIZING\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 18 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css3box"></a>\[CSS3BOX\]  
Elika Etemad. [CSS Box Model Module Level 3](https://www.w3.org/TR/css-box-3/). 11 April 2024. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-3&#x2F;](https://www.w3.org/TR/css-box-3/)

## <a id="property-index"></a>Property Index

<strong>Table 10 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell; scope col):</strong>

Name

<strong>Column 2 (header cell; scope col):</strong>

Value

<strong>Column 3 (header cell; scope col):</strong>

Initial

<strong>Column 4 (header cell; scope col):</strong>

Applies to

<strong>Column 5 (header cell; scope col):</strong>

Inh.

<strong>Column 6 (header cell; scope col):</strong>

%ages

<strong>Column 7 (header cell; scope col):</strong>

Anim­ation type

<strong>Column 8 (header cell; scope col):</strong>

Canonical order

<strong>Column 9 (header cell; scope col):</strong>

Com­puted value

<strong>Row 2</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-column-count①④"></a>

[column-count](#propdef-column-count)

<strong>Column 2 (data cell):</strong>

auto \| \<integer \[1,∞\]\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

block containers except table wrapper boxes

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

by computed value

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified value

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-column-fill⑤"></a>

[column-fill](#propdef-column-fill)

<strong>Column 2 (data cell):</strong>

auto \| balance \| balance-all

<strong>Column 3 (data cell):</strong>

balance

<strong>Column 4 (data cell):</strong>

multicol containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-column-rule⑤"></a>

[column-rule](#propdef-column-rule)

<strong>Column 2 (data cell):</strong>

\<'column-rule-width'\> \|\| \<'column-rule-style'\> \|\| \<'column-rule-color'\>

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-column-rule-color③"></a>

[column-rule-color](#propdef-column-rule-color)

<strong>Column 2 (data cell):</strong>

\<color\>

<strong>Column 3 (data cell):</strong>

currentcolor

<strong>Column 4 (data cell):</strong>

multicol containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

computed color

<strong>Row 6</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-column-rule-style④"></a>

[column-rule-style](#propdef-column-rule-style)

<strong>Column 2 (data cell):</strong>

\<line-style\>

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

multicol containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword

<strong>Row 7</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-column-rule-width⑤"></a>

[column-rule-width](#propdef-column-rule-width)

<strong>Column 2 (data cell):</strong>

\<line-width\>

<strong>Column 3 (data cell):</strong>

medium

<strong>Column 4 (data cell):</strong>

multicol containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

absolute length, snapped as a border width; 0 if the column rule style is none or hidden

<strong>Row 8</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-column-span⑨"></a>

[column-span](#propdef-column-span)

<strong>Column 2 (data cell):</strong>

none \| all

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

in-flow block-level elements

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword

<strong>Row 9</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-column-width①⑥"></a>

[column-width](#propdef-column-width)

<strong>Column 2 (data cell):</strong>

auto \| \<length \[0,∞\]\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

block containers except table wrapper boxes

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

by computed value type

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword auto or an absolute length

<strong>Row 10</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-columns⑤"></a>

[columns](#propdef-columns)

<strong>Column 2 (data cell):</strong>

\<'column-width'\> \|\| \<'column-count'\>

<strong>Column 3 (data cell):</strong>

see individual properties

<strong>Column 4 (data cell):</strong>

see individual properties

<strong>Column 5 (data cell):</strong>

see individual properties

<strong>Column 6 (data cell):</strong>

see individual properties

<strong>Column 7 (data cell):</strong>

see individual properties

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties
