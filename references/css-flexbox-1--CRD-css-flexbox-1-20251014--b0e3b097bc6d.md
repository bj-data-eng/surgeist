Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/).

Original copyright notice: Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Flexible Box Layout Module Level 1

Source snapshot: https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/

Snapshot SHA-256: b0e3b097bc6db0688d4712968c290cda280f8f750eb998bc358c28df490ff3f5

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 22 source tables are presented as readable Markdown tables or explicit labeled layouts: 16 ordinary table conversions, 5 complex-table layouts, 1 already-readable table. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Flexible Box Layout Module Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

The specification describes a CSS box model optimized for user interface design. In the flex layout model, the children of a flex container can be laid out in any direction, and can “flex” their sizes, either growing to fill unused space or shrinking to avoid overflowing the parent. Both horizontal and vertical alignment of the children can be easily manipulated. Nesting of these boxes (horizontal inside vertical, or vertical inside horizontal) can be used to build layouts in two dimensions.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-flexbox” in the title, like this: “\[css-flexbox\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-flexbox%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

<em>This section is not normative.</em>

CSS 2.1 defined four layout modes — algorithms which determine the size and position of boxes based on their relationships with their sibling and ancestor boxes:

- block layout, designed for laying out documents

- inline layout, designed for laying out text

- table layout, designed for laying out 2D data in a tabular format

- positioned layout, designed for very explicit positioning without much regard for other elements in the document

This module introduces a new layout mode, <a id="flex-layout"></a>flex layout, which is designed for laying out more complex applications and webpages.

### <a id="overview"></a>1.1.  Overview

<em>This section is not normative.</em>

Flex layout is superficially similar to block layout. It lacks many of the more complex text- or document-centric properties that can be used in block layout, such as [floats](https://www.w3.org/TR/CSS2/visuren.html#floats) and [columns](https://www.w3.org/TR/css3-multicol/). In return it gains simple and powerful tools for distributing space and aligning content in ways that web apps and complex web pages often need. The contents of a flex container:

- can be laid out in any [flow direction](#flex-direction-property) (leftwards, rightwards, downwards, or even upwards!)

- <a id="ref-for-valdef-flex-direction-row-reverse"></a>

  can have their display order [reversed](#valdef-flex-direction-row-reverse) or 'order\|rearranged' at the style layer (i.e., visual order can be independent of source and speech order)

- <a id="ref-for-main-axis"></a>

  <a id="ref-for-cross-axis"></a>

  can be laid out linearly along a single ([main](#main-axis)) axis or [wrapped](#flex-wrap-property) into multiple lines along a secondary ([cross](#cross-axis)) axis

- can [“flex” their sizes](#flexibility) to respond to the available space

- <a id="ref-for-cross-axis①"></a>

  can be [aligned](#alignment) with respect to their container or each other on the secondary ([cross](#cross-axis))

- <a id="ref-for-main-axis①"></a>

  <a id="ref-for-cross-size"></a>

  can be dynamically [collapsed](#visibility-collapse) or uncollapsed along the [main axis](#main-axis) while preserving the container’s [cross size](#cross-size)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-843b1efd"></a> Here’s an example of a catalog where each item has a title, a photo, a description, and a purchase button. The designer’s intention is that each entry has the same overall size, that the photo be above the text, and that the purchase buttons are aligned at the bottom, regardless of the length of the item’s description. Flex layout makes many aspects of this design easy:
>
> - The catalog uses flex layout to lay out rows of items horizontally, and to ensure that items within a row are all equal-height. Each entry is then itself a column flex container, laying out its contents vertically.
>
> - <a id="ref-for-propdef-order"></a>
>
>   <a id="ref-for-propdef-align-self"></a>
>
>   Within each entry, the source document content is ordered logically with the title first, followed by the description and the photo. This provides a sensible ordering for speech rendering and in non-CSS browsers. For a more compelling visual presentation, however, [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) is used to pull the image up from later in the content to the top, and [align-self](#propdef-align-self) is used to center it horizontally.
>
> - An [auto margin](#auto-margins) above the purchase button forces it to the bottom within each entry box, regardless of the height of that item’s description.
>
> <a id="ref-for-propdef-align-self①"></a>
>
> <a id="ref-for-propdef-align-self②"></a>
>
> ```css
> #deals {
>   display: flex;        /* Flex layout so items have equal height  */
>   flex-flow: row wrap;  /* Allow items to wrap into multiple lines */
> }
> .sale-item {
>   display: flex;        /* Lay out each item using flex layout */
>   flex-flow: column;    /* Lay out item’s contents vertically  */
> }
> .sale-item > img {
>   order: -1;            /* Shift image before other content (in visual order) */
>   align-self: center;   /* Center the image cross-wise (horizontally)         */
> }
> .sale-item > button {
>   margin-top: auto;     /* Auto top margin pushes button to bottom */
> }
> ```
>
> ```markup
> <section id="deals">
>   <section class="sale-item">
>     <h1>Computer Starter Kit</h1>
>     <p>This is the best computer money can buy, if you don’t have much money.
>     <ul>
>       <li>Computer
>       <li>Monitor
>       <li>Keyboard
>       <li>Mouse
>     </ul>
>     <img src="images/computer.jpg"
>          alt="You get: a white computer with matching peripherals.">
>     <button>BUY NOW</button>
>   </section>
>   <section class="sale-item">
>     …
>   </section>
>   …
> </section>
> ```
>
> <a id="overview-example"></a>
>
> ![You get: a white computer with matching keyboard and monitor.](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/computer.jpg)
>
> # <a id="quiet-pubrules-1"></a>Computer Starter Kit
>
> This is the best computer money can buy, if you don’t have much money.
>
> - Computer
> - Monitor
> - Keyboard
> - Mouse
>
> ![You get: beautiful ASCII art.](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/printer.png)
>
> # <a id="quiet-pubrules-2"></a>Printer
>
> Only capable of printing ASCII art.
>
> - Paper and ink not included.
>
> BUY NOW
>
> BUY NOW
>
> An example rendering of the code above.

### <a id="placement"></a>1.2.  Module interactions

<a id="ref-for-propdef-display"></a>

<a id="ref-for-selectordef-first-line"></a>

<a id="ref-for-selectordef-first-letter"></a>

This module extends the definition of the [display](https://www.w3.org/TR/css-display-4/#propdef-display) property [\[CSS2\]](#biblio-css2), adding a new block-level and new inline-level display type, and defining a new type of formatting context along with properties to control its layout. None of the properties defined in this module apply to the [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) or [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) pseudo-elements.

<a id="ref-for-propdef-justify-content"></a>

<a id="ref-for-propdef-align-items"></a>

<a id="ref-for-propdef-align-self③"></a>

<a id="ref-for-propdef-align-content"></a>

The [CSS Box Alignment Module](https://www.w3.org/TR/css-align/) extends and supersedes the definitions of the alignment properties ([justify-content](#propdef-justify-content), [align-items](#propdef-align-items), [align-self](#propdef-align-self), [align-content](#propdef-align-content)) introduced here.

Tests

- [flexbox_first-letter.html](https://wpt.fyi/results/css/css-flexbox/flexbox_first-letter.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_first-letter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_first-letter.html)
- [flexbox_first-line.html](https://wpt.fyi/results/css/css-flexbox/flexbox_first-line.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_first-line.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_first-line.html)
- [flexbox-ignores-first-letter.html](https://wpt.fyi/results/css/css-flexbox/flexbox-ignores-first-letter.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-ignores-first-letter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-ignores-first-letter.html)

### <a id="values"></a>1.3.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="box-model"></a>2.  Flex Layout Box Model and Terminology

<a id="ref-for-propdef-display①"></a>

<a id="ref-for-valdef-display-flex"></a>

<a id="ref-for-valdef-display-inline-flex"></a>

A <a id="flex-container"></a>flex container is the box generated by an element with a computed [display](https://www.w3.org/TR/css-display-4/#propdef-display) of [flex](#valdef-display-flex) or [inline-flex](#valdef-display-inline-flex). In-flow children of a flex container are called <a id="flex-item"></a>flex items and are laid out using the flex layout model.

<a id="ref-for-propdef-flex-flow"></a>

<a id="ref-for-writing-mode"></a>

Unlike block and inline layout, whose layout calculations are biased to the [block and inline flow directions](https://www.w3.org/TR/css3-writing-modes/#abstract-box), flex layout is biased to the <a id="flex-direction"></a>flex directions. To make it easier to talk about flex layout, this section defines a set of flex flow–relative terms. The [flex-flow](#propdef-flex-flow) value and the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) determine how these terms map to physical directions (top/right/bottom/left), axes (vertical/horizontal), and sizes (width/height).

![](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/flex-direction-terms.svg)

<a id="ref-for-valdef-flex-direction-row"></a>

An illustration of the various directions and sizing terms as applied to a [row](#valdef-flex-direction-row) flex container.

<a id="main"></a>

main axis  
main dimension  
<a id="ref-for-flex-item"></a>

The <a id="main-axis"></a>main axis of a flex container is the primary axis along which [flex items](#flex-item) are laid out. It extends in the <a id="main-dimension"></a>main dimension.

main-start  
main-end  
<a id="ref-for-flex-item①"></a>

The [flex items](#flex-item) are placed within the container starting on the <a id="main-start"></a>main-start side and going toward the <a id="main-end"></a>main-end side.

main size  
main size property  
<a id="ref-for-propdef-max-height"></a>

<a id="ref-for-propdef-min-height"></a>

<a id="ref-for-propdef-max-width"></a>

<a id="ref-for-propdef-min-width"></a>

<a id="ref-for-propdef-height"></a>

<a id="ref-for-propdef-width"></a>

<a id="ref-for-main-dimension"></a>

<a id="ref-for-height"></a>

<a id="ref-for-width"></a>

<a id="ref-for-flex-item②"></a>

<a id="ref-for-flex-container"></a>

The <a id="main-size"></a>main size of a [flex container](#flex-container) or [flex item](#flex-item) refers to its [width](https://www.w3.org/TR/css-sizing-3/#width) or [height](https://www.w3.org/TR/css-sizing-3/#height), whichever is in the [main dimension](#main-dimension). Its <a id="main-size-property"></a>main size property is either its [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) or [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) property, whichever is in the <a id="ref-for-main-dimension①"></a>main dimension. Likewise, its <a id="min-main-size-property"></a>min and <a id="max-main-size-property"></a>max main size properties are its [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width)/[max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width) or [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height)/[max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height) properties, whichever are in the <a id="ref-for-main-dimension②"></a>main dimension, and determine its <a id="min-main-size"></a>min/<a id="max-main-size"></a>max main size.

<a id="ref-for-flex-layout"></a>

<a id="ref-for-main-size"></a>

<a id="ref-for-propdef-flex"></a>

<a id="ref-for-main-size-property"></a>

In [flex layout](#flex-layout), the [main size](#main-size) is controlled by the [flex](#propdef-flex) property rather than directly by the [main size property](#main-size-property).

<a id="ref-for-main-dimension③"></a>

<a id="ref-for-width①"></a>

<a id="ref-for-height①"></a>

<a id="ref-for-inline-size"></a>

<a id="ref-for-block-size"></a>

<a id="ref-for-main-size①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means any references to a flex item’s used size in the [main dimension](#main-dimension) ([width](https://www.w3.org/TR/css-sizing-3/#width), [height](https://www.w3.org/TR/css-sizing-3/#height), [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size), [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size)) refers to its post-flexing [main size](#main-size).

<a id="cross"></a>

cross axis  
cross dimension  
<a id="ref-for-main-axis②"></a>

The axis perpendicular to the [main axis](#main-axis) is called the <a id="cross-axis"></a>cross axis. It extends in the <a id="cross-dimension"></a>cross dimension.

cross-start  
cross-end  
<a id="ref-for-flex-line"></a>

[Flex lines](#flex-line) are filled with items and placed into the container starting on the <a id="cross-start"></a>cross-start side of the flex container and going toward the <a id="cross-end"></a>cross-end side.

cross size  
cross size property  
<a id="ref-for-propdef-max-height①"></a>

<a id="ref-for-propdef-min-height①"></a>

<a id="ref-for-propdef-max-width①"></a>

<a id="ref-for-propdef-min-width①"></a>

<a id="ref-for-propdef-height①"></a>

<a id="ref-for-propdef-width①"></a>

<a id="ref-for-cross-dimension"></a>

<a id="ref-for-height②"></a>

<a id="ref-for-width②"></a>

<a id="ref-for-flex-item③"></a>

<a id="ref-for-flex-container①"></a>

The <a id="cross-size"></a>cross size of a [flex container](#flex-container) or [flex item](#flex-item) refers to its [width](https://www.w3.org/TR/css-sizing-3/#width) or [height](https://www.w3.org/TR/css-sizing-3/#height), whichever is in the [cross dimension](#cross-dimension). Its <a id="cross-size-property"></a>cross size property is either its [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) or [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) property, whichever is in the <a id="ref-for-cross-dimension①"></a>cross dimension. Likewise, its <a id="min-cross-size-property"></a>min and <a id="max-cross-size-property"></a>max cross size properties are its [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width)/[max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width) or [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height)/[max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height) properties, whichever are in the <a id="ref-for-cross-dimension②"></a>cross dimension, and determine its <a id="min-cross-size"></a>min/<a id="max-cross-size"></a>max cross size.

Additional sizing terminology used in this specification is defined in [CSS Intrinsic and Extrinsic Sizing](https://www.w3.org/TR/CSS-SIZING-3/). [\[CSS-SIZING-3\]](#biblio-css-sizing-3)

Tests

- [box-sizing-min-max-sizes-001.html](https://wpt.fyi/results/css/css-flexbox/box-sizing-min-max-sizes-001.html) [(live test)](http://wpt.live/css/css-flexbox/box-sizing-min-max-sizes-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/box-sizing-min-max-sizes-001.html)

<a id="ref-for-valdef-display-flex①"></a>

<a id="ref-for-valdef-display-inline-flex①"></a>

<a id="ref-for-propdef-display②"></a>

## <a id="flex-containers"></a>3.  Flex Containers: the [flex](#valdef-display-flex) and [inline-flex](#valdef-display-inline-flex) [display](https://www.w3.org/TR/css-display-4/#propdef-display) values

| Field               | Definition                                                                             |
|---------------------|----------------------------------------------------------------------------------------|
| <strong>Name:&#xA;       </strong> | <a id="ref-for-propdef-display③"></a>[display](https://www.w3.org/TR/css-display-4/#propdef-display)     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">New values:</a>&#xA;       </strong> | <a id="ref-for-comb-one"></a>flex [\|](https://www.w3.org/TR/css-values-4/#comb-one) inline-flex |

Tests

- [inheritance.html](https://wpt.fyi/results/css/css-flexbox/inheritance.html) [(live test)](http://wpt.live/css/css-flexbox/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/inheritance.html)

<a id="valdef-display-flex"></a>flex  
<a id="ref-for-flow-layout"></a>

<a id="ref-for-block-level"></a>

<a id="ref-for-flex-container②"></a>

This value causes an element to generate a [flex container](#flex-container) box that is [block-level](https://www.w3.org/TR/css-display-4/#block-level) when placed in [flow layout](https://www.w3.org/TR/css-display-4/#flow-layout).

Tests

- [baseline-synthesis-001.html](https://wpt.fyi/results/css/css-flexbox/baseline-synthesis-001.html) [(live test)](http://wpt.live/css/css-flexbox/baseline-synthesis-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/baseline-synthesis-001.html)
- [baseline-synthesis-002.html](https://wpt.fyi/results/css/css-flexbox/baseline-synthesis-002.html) [(live test)](http://wpt.live/css/css-flexbox/baseline-synthesis-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/baseline-synthesis-002.html)
- [baseline-synthesis-003.html](https://wpt.fyi/results/css/css-flexbox/baseline-synthesis-003.html) [(live test)](http://wpt.live/css/css-flexbox/baseline-synthesis-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/baseline-synthesis-003.html)
- [baseline-synthesis-004.html](https://wpt.fyi/results/css/css-flexbox/baseline-synthesis-004.html) [(live test)](http://wpt.live/css/css-flexbox/baseline-synthesis-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/baseline-synthesis-004.html)
- [baseline-synthesis-vert-lr-line-under.html](https://wpt.fyi/results/css/css-flexbox/baseline-synthesis-vert-lr-line-under.html) [(live test)](http://wpt.live/css/css-flexbox/baseline-synthesis-vert-lr-line-under.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/baseline-synthesis-vert-lr-line-under.html)
- [display-flex-001.htm](https://wpt.fyi/results/css/css-flexbox/display-flex-001.htm) [(live test)](http://wpt.live/css/css-flexbox/display-flex-001.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/display-flex-001.htm)
- [dynamic-change-simplified-layout-002.html](https://wpt.fyi/results/css/css-flexbox/dynamic-change-simplified-layout-002.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-change-simplified-layout-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-change-simplified-layout-002.html)
- [dynamic-change-simplified-layout.html](https://wpt.fyi/results/css/css-flexbox/dynamic-change-simplified-layout.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-change-simplified-layout.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-change-simplified-layout.html)
- [fixedpos-video-in-abspos-quirk-crash.html](https://wpt.fyi/results/css/css-flexbox/fixedpos-video-in-abspos-quirk-crash.html) [(live test)](http://wpt.live/css/css-flexbox/fixedpos-video-in-abspos-quirk-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/fixedpos-video-in-abspos-quirk-crash.html)
- [flexbox_flex-0-0-0.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-0-0.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-0-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-0-0.html)
- [flexbox_flex-0-0-0-unitless.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-0-0-unitless.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-0-0-unitless.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-0-0-unitless.html)
- [flexbox_flex-0-0-1-unitless-basis.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-0-1-unitless-basis.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-0-1-unitless-basis.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-0-1-unitless-basis.html)
- [flexbox_flex-0-0-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-0-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-0-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-0-auto.html)
- [flexbox_flex-0-0-auto-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-0-auto-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-0-auto-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-0-auto-shrink.html)
- [flexbox_flex-0-0.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-0.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-0.html)
- [flexbox_flex-0-0-N.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-0-N.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-0-N.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-0-N.html)
- [flexbox_flex-0-0-Npercent.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-0-Npercent.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-0-Npercent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-0-Npercent.html)
- [flexbox_flex-0-0-Npercent-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-0-Npercent-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-0-Npercent-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-0-Npercent-shrink.html)
- [flexbox_flex-0-0-N-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-0-N-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-0-N-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-0-N-shrink.html)
- [flexbox_flex-0-0-N-unitless-basis.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-0-N-unitless-basis.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-0-N-unitless-basis.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-0-N-unitless-basis.html)
- [flexbox_flex-0-1-0.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-1-0.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-1-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-1-0.html)
- [flexbox_flex-0-1-0-unitless.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-1-0-unitless.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-1-0-unitless.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-1-0-unitless.html)
- [flexbox_flex-0-1-1-unitless-basis.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-1-1-unitless-basis.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-1-1-unitless-basis.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-1-1-unitless-basis.html)
- [flexbox_flex-0-1-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-1-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-1-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-1-auto.html)
- [flexbox_flex-0-1-auto-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-1-auto-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-1-auto-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-1-auto-shrink.html)
- [flexbox_flex-0-1.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-1.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-1.html)
- [flexbox_flex-0-1-N.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-1-N.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-1-N.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-1-N.html)
- [flexbox_flex-0-1-Npercent.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-1-Npercent.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-1-Npercent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-1-Npercent.html)
- [flexbox_flex-0-1-Npercent-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-1-Npercent-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-1-Npercent-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-1-Npercent-shrink.html)
- [flexbox_flex-0-1-N-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-1-N-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-1-N-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-1-N-shrink.html)
- [flexbox_flex-0-1-N-unitless-basis.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-1-N-unitless-basis.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-1-N-unitless-basis.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-1-N-unitless-basis.html)
- [flexbox_flex-0-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-auto.html)
- [flexbox_flex-0-N-0.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-N-0.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-N-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-N-0.html)
- [flexbox_flex-0-N-0-unitless.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-N-0-unitless.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-N-0-unitless.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-N-0-unitless.html)
- [flexbox_flex-0-N-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-N-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-N-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-N-auto.html)
- [flexbox_flex-0-N-auto-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-N-auto-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-N-auto-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-N-auto-shrink.html)
- [flexbox_flex-0-N.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-N.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-N.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-N.html)
- [flexbox_flex-0-N-N.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-N-N.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-N-N.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-N-N.html)
- [flexbox_flex-0-N-Npercent.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-N-Npercent.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-N-Npercent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-N-Npercent.html)
- [flexbox_flex-0-N-Npercent-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-N-Npercent-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-N-Npercent-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-N-Npercent-shrink.html)
- [flexbox_flex-0-N-N-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-0-N-N-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-0-N-N-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-0-N-N-shrink.html)
- [flexbox_flex-1-0-0.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-0-0.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-0-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-0-0.html)
- [flexbox_flex-1-0-0-unitless.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-0-0-unitless.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-0-0-unitless.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-0-0-unitless.html)
- [flexbox_flex-1-0-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-0-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-0-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-0-auto.html)
- [flexbox_flex-1-0-auto-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-0-auto-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-0-auto-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-0-auto-shrink.html)
- [flexbox_flex-1-0.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-0.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-0.html)
- [flexbox_flex-1-0-N.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-0-N.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-0-N.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-0-N.html)
- [flexbox_flex-1-0-Npercent.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-0-Npercent.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-0-Npercent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-0-Npercent.html)
- [flexbox_flex-1-0-Npercent-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-0-Npercent-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-0-Npercent-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-0-Npercent-shrink.html)
- [flexbox_flex-1-0-N-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-0-N-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-0-N-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-0-N-shrink.html)
- [flexbox_flex-1-1-0.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-1-0.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-1-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-1-0.html)
- [flexbox_flex-1-1-0-unitless.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-1-0-unitless.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-1-0-unitless.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-1-0-unitless.html)
- [flexbox_flex-1-1-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-1-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-1-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-1-auto.html)
- [flexbox_flex-1-1-auto-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-1-auto-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-1-auto-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-1-auto-shrink.html)
- [flexbox_flex-1-1.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-1.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-1.html)
- [flexbox_flex-1-1-N.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-1-N.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-1-N.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-1-N.html)
- [flexbox_flex-1-1-Npercent.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-1-Npercent.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-1-Npercent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-1-Npercent.html)
- [flexbox_flex-1-1-Npercent-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-1-Npercent-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-1-Npercent-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-1-Npercent-shrink.html)
- [flexbox_flex-1-1-N-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-1-N-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-1-N-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-1-N-shrink.html)
- [flexbox_flex-1-N-0.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-N-0.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-N-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-N-0.html)
- [flexbox_flex-1-N-0-unitless.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-N-0-unitless.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-N-0-unitless.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-N-0-unitless.html)
- [flexbox_flex-1-N-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-N-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-N-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-N-auto.html)
- [flexbox_flex-1-N-auto-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-N-auto-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-N-auto-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-N-auto-shrink.html)
- [flexbox_flex-1-N.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-N.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-N.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-N.html)
- [flexbox_flex-1-N-N.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-N-N.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-N-N.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-N-N.html)
- [flexbox_flex-1-N-Npercent.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-N-Npercent.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-N-Npercent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-N-Npercent.html)
- [flexbox_flex-1-N-Npercent-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-N-Npercent-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-N-Npercent-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-N-Npercent-shrink.html)
- [flexbox_flex-1-N-N-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-1-N-N-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-1-N-N-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-1-N-N-shrink.html)
- [flexbox_flex-N-0-0.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-0-0.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-0-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-0-0.html)
- [flexbox_flex-N-0-0-unitless.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-0-0-unitless.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-0-0-unitless.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-0-0-unitless.html)
- [flexbox_flex-N-0-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-0-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-0-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-0-auto.html)
- [flexbox_flex-N-0-auto-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-0-auto-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-0-auto-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-0-auto-shrink.html)
- [flexbox_flex-N-0.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-0.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-0.html)
- [flexbox_flex-N-0-N.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-0-N.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-0-N.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-0-N.html)
- [flexbox_flex-N-0-Npercent.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-0-Npercent.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-0-Npercent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-0-Npercent.html)
- [flexbox_flex-N-0-Npercent-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-0-Npercent-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-0-Npercent-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-0-Npercent-shrink.html)
- [flexbox_flex-N-0-N-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-0-N-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-0-N-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-0-N-shrink.html)
- [flexbox_flex-N-1-0.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-1-0.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-1-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-1-0.html)
- [flexbox_flex-N-1-0-unitless.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-1-0-unitless.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-1-0-unitless.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-1-0-unitless.html)
- [flexbox_flex-N-1-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-1-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-1-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-1-auto.html)
- [flexbox_flex-N-1-auto-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-1-auto-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-1-auto-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-1-auto-shrink.html)
- [flexbox_flex-N-1.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-1.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-1.html)
- [flexbox_flex-N-1-N.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-1-N.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-1-N.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-1-N.html)
- [flexbox_flex-N-1-Npercent.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-1-Npercent.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-1-Npercent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-1-Npercent.html)
- [flexbox_flex-N-1-Npercent-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-1-Npercent-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-1-Npercent-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-1-Npercent-shrink.html)
- [flexbox_flex-N-1-N-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-1-N-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-1-N-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-1-N-shrink.html)
- [flexbox_flex-N-N-0.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-N-0.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-N-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-N-0.html)
- [flexbox_flex-N-N-0-unitless.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-N-0-unitless.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-N-0-unitless.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-N-0-unitless.html)
- [flexbox_flex-N-N-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-N-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-N-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-N-auto.html)
- [flexbox_flex-N-N-auto-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-N-auto-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-N-auto-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-N-auto-shrink.html)
- [flexbox_flex-N-N.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-N.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-N.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-N.html)
- [flexbox_flex-N-N-N.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-N-N.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-N-N.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-N-N.html)
- [flexbox_flex-N-N-Npercent.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-N-Npercent.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-N-Npercent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-N-Npercent.html)
- [flexbox_flex-N-N-Npercent-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-N-Npercent-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-N-Npercent-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-N-Npercent-shrink.html)
- [flexbox_flex-N-N-N-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-N-N-N-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-N-N-N-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-N-N-N-shrink.html)
- [flexbox_flex-formatting-interop.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-formatting-interop.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-formatting-interop.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-formatting-interop.html)
- [flexbox_generated-flex.html](https://wpt.fyi/results/css/css-flexbox/flexbox_generated-flex.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_generated-flex.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_generated-flex.html)
- [flexbox_generated.html](https://wpt.fyi/results/css/css-flexbox/flexbox_generated.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_generated.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_generated.html)
- [flexbox_generated-nested-flex.html](https://wpt.fyi/results/css/css-flexbox/flexbox_generated-nested-flex.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_generated-nested-flex.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_generated-nested-flex.html)
- [flexbox-iframe-intrinsic-size-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-iframe-intrinsic-size-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-iframe-intrinsic-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-iframe-intrinsic-size-001.html)
- [flexbox_item-bottom-float.html](https://wpt.fyi/results/css/css-flexbox/flexbox_item-bottom-float.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_item-bottom-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_item-bottom-float.html)
- [flexbox_item-clear.html](https://wpt.fyi/results/css/css-flexbox/flexbox_item-clear.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_item-clear.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_item-clear.html)
- [flexbox_item-float.html](https://wpt.fyi/results/css/css-flexbox/flexbox_item-float.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_item-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_item-float.html)
- [flexbox_item-top-float.html](https://wpt.fyi/results/css/css-flexbox/flexbox_item-top-float.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_item-top-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_item-top-float.html)
- [flexbox_item-vertical-align.html](https://wpt.fyi/results/css/css-flexbox/flexbox_item-vertical-align.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_item-vertical-align.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_item-vertical-align.html)
- [flexbox_block.html](https://wpt.fyi/results/css/css-flexbox/flexbox_block.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_block.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_block.html)
- [flexbox_box-clear.html](https://wpt.fyi/results/css/css-flexbox/flexbox_box-clear.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_box-clear.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_box-clear.html)
- [flexbox_display.html](https://wpt.fyi/results/css/css-flexbox/flexbox_display.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_display.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_display.html)
- [flexbox_fbfc2.html](https://wpt.fyi/results/css/css-flexbox/flexbox_fbfc2.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_fbfc2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_fbfc2.html)
- [flexbox_fbfc.html](https://wpt.fyi/results/css/css-flexbox/flexbox_fbfc.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_fbfc.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_fbfc.html)
- [flexbox_nested-flex.html](https://wpt.fyi/results/css/css-flexbox/flexbox_nested-flex.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_nested-flex.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_nested-flex.html)
- [flexbox-root-node-001a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-root-node-001a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-root-node-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-root-node-001a.html)
- [flexbox-root-node-001b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-root-node-001b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-root-node-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-root-node-001b.html)
- [flexbox_stf-fixpos.html](https://wpt.fyi/results/css/css-flexbox/flexbox_stf-fixpos.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_stf-fixpos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_stf-fixpos.html)
- [flexbox_stf-float.html](https://wpt.fyi/results/css/css-flexbox/flexbox_stf-float.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_stf-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_stf-float.html)
- [flexbox_stf-inline-block.html](https://wpt.fyi/results/css/css-flexbox/flexbox_stf-inline-block.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_stf-inline-block.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_stf-inline-block.html)
- [flexbox_stf-table-caption.html](https://wpt.fyi/results/css/css-flexbox/flexbox_stf-table-caption.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_stf-table-caption.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_stf-table-caption.html)
- [flexbox_stf-table-cell.html](https://wpt.fyi/results/css/css-flexbox/flexbox_stf-table-cell.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_stf-table-cell.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_stf-table-cell.html)
- [flexbox_stf-table.html](https://wpt.fyi/results/css/css-flexbox/flexbox_stf-table.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_stf-table.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_stf-table.html)
- [flexbox_stf-table-row-group.html](https://wpt.fyi/results/css/css-flexbox/flexbox_stf-table-row-group.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_stf-table-row-group.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_stf-table-row-group.html)
- [flexbox_stf-table-row.html](https://wpt.fyi/results/css/css-flexbox/flexbox_stf-table-row.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_stf-table-row.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_stf-table-row.html)
- [flexbox_stf-table-singleline-2.html](https://wpt.fyi/results/css/css-flexbox/flexbox_stf-table-singleline-2.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_stf-table-singleline-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_stf-table-singleline-2.html)
- [flexbox_stf-table-singleline.html](https://wpt.fyi/results/css/css-flexbox/flexbox_stf-table-singleline.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_stf-table-singleline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_stf-table-singleline.html)
- [flexbox_table-fixed-layout.html](https://wpt.fyi/results/css/css-flexbox/flexbox_table-fixed-layout.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_table-fixed-layout.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_table-fixed-layout.html)
- [flexbox-with-multi-column-property.html](https://wpt.fyi/results/css/css-flexbox/flexbox-with-multi-column-property.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-with-multi-column-property.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-with-multi-column-property.html)
- [flexbox_computedstyle_display.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_display.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_display.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_display.html)
- [grid-flex-item-001.html](https://wpt.fyi/results/css/css-flexbox/grid-flex-item-001.html) [(live test)](http://wpt.live/css/css-flexbox/grid-flex-item-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/grid-flex-item-001.html)
- [grid-flex-item-002.html](https://wpt.fyi/results/css/css-flexbox/grid-flex-item-002.html) [(live test)](http://wpt.live/css/css-flexbox/grid-flex-item-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/grid-flex-item-002.html)
- [grid-flex-item-003.html](https://wpt.fyi/results/css/css-flexbox/grid-flex-item-003.html) [(live test)](http://wpt.live/css/css-flexbox/grid-flex-item-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/grid-flex-item-003.html)
- [grid-flex-item-004.html](https://wpt.fyi/results/css/css-flexbox/grid-flex-item-004.html) [(live test)](http://wpt.live/css/css-flexbox/grid-flex-item-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/grid-flex-item-004.html)
- [grid-flex-item-005.html](https://wpt.fyi/results/css/css-flexbox/grid-flex-item-005.html) [(live test)](http://wpt.live/css/css-flexbox/grid-flex-item-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/grid-flex-item-005.html)
- [grid-flex-item-006.html](https://wpt.fyi/results/css/css-flexbox/grid-flex-item-006.html) [(live test)](http://wpt.live/css/css-flexbox/grid-flex-item-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/grid-flex-item-006.html)
- [grid-flex-item-007.html](https://wpt.fyi/results/css/css-flexbox/grid-flex-item-007.html) [(live test)](http://wpt.live/css/css-flexbox/grid-flex-item-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/grid-flex-item-007.html)
- flexbox_interactive_flex-transitions.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_flex-transitions.html)
- flexbox_interactive_order-transitions-2.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_order-transitions-2.html)
- flexbox_interactive_order-transitions.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_order-transitions.html)
- [nested-flex-image-loading-invalidates-intrinsic-sizes.html](https://wpt.fyi/results/css/css-flexbox/nested-flex-image-loading-invalidates-intrinsic-sizes.html) [(live test)](http://wpt.live/css/css-flexbox/nested-flex-image-loading-invalidates-intrinsic-sizes.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/nested-flex-image-loading-invalidates-intrinsic-sizes.html)
- [percentage-margins-001.html](https://wpt.fyi/results/css/css-flexbox/percentage-margins-001.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-margins-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-margins-001.html)
- [stretch-after-sibling-size-change.html](https://wpt.fyi/results/css/css-flexbox/stretch-after-sibling-size-change.html) [(live test)](http://wpt.live/css/css-flexbox/stretch-after-sibling-size-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/stretch-after-sibling-size-change.html)
- [stretched-child-in-nested-flexbox-001.html](https://wpt.fyi/results/css/css-flexbox/stretched-child-in-nested-flexbox-001.html) [(live test)](http://wpt.live/css/css-flexbox/stretched-child-in-nested-flexbox-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/stretched-child-in-nested-flexbox-001.html)
- [stretched-child-in-nested-flexbox-002.html](https://wpt.fyi/results/css/css-flexbox/stretched-child-in-nested-flexbox-002.html) [(live test)](http://wpt.live/css/css-flexbox/stretched-child-in-nested-flexbox-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/stretched-child-in-nested-flexbox-002.html)
- [stretched-child-in-nested-flexbox-003.html](https://wpt.fyi/results/css/css-flexbox/stretched-child-in-nested-flexbox-003.html) [(live test)](http://wpt.live/css/css-flexbox/stretched-child-in-nested-flexbox-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/stretched-child-in-nested-flexbox-003.html)
- [stretched-child-shrink-on-relayout.html](https://wpt.fyi/results/css/css-flexbox/stretched-child-shrink-on-relayout.html) [(live test)](http://wpt.live/css/css-flexbox/stretched-child-shrink-on-relayout.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/stretched-child-shrink-on-relayout.html)
- [stretch-flex-item-checkbox-input.html](https://wpt.fyi/results/css/css-flexbox/stretch-flex-item-checkbox-input.html) [(live test)](http://wpt.live/css/css-flexbox/stretch-flex-item-checkbox-input.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/stretch-flex-item-checkbox-input.html)
- [stretch-flex-item-radio-input.html](https://wpt.fyi/results/css/css-flexbox/stretch-flex-item-radio-input.html) [(live test)](http://wpt.live/css/css-flexbox/stretch-flex-item-radio-input.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/stretch-flex-item-radio-input.html)
- [stretching-orthogonal-flows.html](https://wpt.fyi/results/css/css-flexbox/stretching-orthogonal-flows.html) [(live test)](http://wpt.live/css/css-flexbox/stretching-orthogonal-flows.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/stretching-orthogonal-flows.html)
- [table-with-percent-intrinsic-width.html](https://wpt.fyi/results/css/css-flexbox/table-with-percent-intrinsic-width.html) [(live test)](http://wpt.live/css/css-flexbox/table-with-percent-intrinsic-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-with-percent-intrinsic-width.html)

<a id="valdef-display-inline-flex"></a>inline-flex  
<a id="ref-for-flow-layout①"></a>

<a id="ref-for-inline-level"></a>

<a id="ref-for-flex-container③"></a>

This value causes an element to generate a [flex container](#flex-container) box that is [inline-level](https://www.w3.org/TR/css-display-4/#inline-level) when placed in [flow layout](https://www.w3.org/TR/css-display-4/#flow-layout).

Tests

- [flexbox_inline.html](https://wpt.fyi/results/css/css-flexbox/flexbox_inline.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_inline.html)
- [flex-inline.html](https://wpt.fyi/results/css/css-flexbox/flex-inline.html) [(live test)](http://wpt.live/css/css-flexbox/flex-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-inline.html)
- [flexbox_inline-float.html](https://wpt.fyi/results/css/css-flexbox/flexbox_inline-float.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_inline-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_inline-float.html)
- [inline-flexbox-absurd-block-size-crash.html](https://wpt.fyi/results/css/css-flexbox/inline-flexbox-absurd-block-size-crash.html) [(live test)](http://wpt.live/css/css-flexbox/inline-flexbox-absurd-block-size-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/inline-flexbox-absurd-block-size-crash.html)
- [inline-flexbox-wrap-vertically-width-calculation.html](https://wpt.fyi/results/css/css-flexbox/inline-flexbox-wrap-vertically-width-calculation.html) [(live test)](http://wpt.live/css/css-flexbox/inline-flexbox-wrap-vertically-width-calculation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/inline-flexbox-wrap-vertically-width-calculation.html)
- [inline-flex-editing-crash.html](https://wpt.fyi/results/css/css-flexbox/inline-flex-editing-crash.html) [(live test)](http://wpt.live/css/css-flexbox/inline-flex-editing-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/inline-flex-editing-crash.html)
- [inline-flex-editing-with-updating-text-crash.html](https://wpt.fyi/results/css/css-flexbox/inline-flex-editing-with-updating-text-crash.html) [(live test)](http://wpt.live/css/css-flexbox/inline-flex-editing-with-updating-text-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/inline-flex-editing-with-updating-text-crash.html)
- [inline-flex-frameset-main-axis-crash.html](https://wpt.fyi/results/css/css-flexbox/inline-flex-frameset-main-axis-crash.html) [(live test)](http://wpt.live/css/css-flexbox/inline-flex-frameset-main-axis-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/inline-flex-frameset-main-axis-crash.html)
- [inline-flex.html](https://wpt.fyi/results/css/css-flexbox/inline-flex.html) [(live test)](http://wpt.live/css/css-flexbox/inline-flex.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/inline-flex.html)
- [inline-flex-min-content-height.html](https://wpt.fyi/results/css/css-flexbox/inline-flex-min-content-height.html) [(live test)](http://wpt.live/css/css-flexbox/inline-flex-min-content-height.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/inline-flex-min-content-height.html)
- [flexbox_computedstyle_display-inline.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_display-inline.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_display-inline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_display-inline.html)
- [intrinsic-width-orthogonal-writing-mode.html](https://wpt.fyi/results/css/css-flexbox/intrinsic-width-orthogonal-writing-mode.html) [(live test)](http://wpt.live/css/css-flexbox/intrinsic-width-orthogonal-writing-mode.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/intrinsic-width-orthogonal-writing-mode.html)

<a id="ref-for-flex-container④"></a>

<a id="ref-for-propdef-overflow"></a>

A [flex container](#flex-container) establishes a new <a id="flex-formatting-context"></a>flex formatting context for its contents. This is the same as establishing a block formatting context, except that flex layout is used instead of block layout. For example, floats do not intrude into the flex container, and the flex container’s margins do not collapse with the margins of its contents. <a id="ref-for-flex-container⑤"></a>Flex containers form a containing block for their contents [exactly like block containers do](https://www.w3.org/TR/CSS2/visudet.html#containing-block-details). [\[CSS2\]](#biblio-css2) The [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) property applies to <a id="ref-for-flex-container⑥"></a>flex containers.

Tests

- [flexbox-overflow-horiz-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-overflow-horiz-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-overflow-horiz-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-overflow-horiz-001.html)
- [flexbox-overflow-horiz-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-overflow-horiz-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-overflow-horiz-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-overflow-horiz-002.html)
- [flexbox-overflow-horiz-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-overflow-horiz-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-overflow-horiz-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-overflow-horiz-003.html)
- [flexbox-overflow-horiz-004.html](https://wpt.fyi/results/css/css-flexbox/flexbox-overflow-horiz-004.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-overflow-horiz-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-overflow-horiz-004.html)
- [flexbox-overflow-horiz-005.html](https://wpt.fyi/results/css/css-flexbox/flexbox-overflow-horiz-005.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-overflow-horiz-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-overflow-horiz-005.html)
- [flexbox-overflow-padding-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-overflow-padding-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-overflow-padding-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-overflow-padding-001.html)
- [flexbox-overflow-padding-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-overflow-padding-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-overflow-padding-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-overflow-padding-002.html)
- [flexbox-overflow-vert-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-overflow-vert-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-overflow-vert-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-overflow-vert-001.html)
- [flexbox-overflow-vert-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-overflow-vert-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-overflow-vert-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-overflow-vert-002.html)
- [flexbox-overflow-vert-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-overflow-vert-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-overflow-vert-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-overflow-vert-003.html)
- [flexbox-overflow-vert-004.html](https://wpt.fyi/results/css/css-flexbox/flexbox-overflow-vert-004.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-overflow-vert-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-overflow-vert-004.html)
- [flexbox-overflow-vert-005.html](https://wpt.fyi/results/css/css-flexbox/flexbox-overflow-vert-005.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-overflow-vert-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-overflow-vert-005.html)
- [flexbox_rowspan-overflow-automatic.html](https://wpt.fyi/results/css/css-flexbox/flexbox_rowspan-overflow-automatic.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_rowspan-overflow-automatic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_rowspan-overflow-automatic.html)
- [flexbox_rowspan-overflow.html](https://wpt.fyi/results/css/css-flexbox/flexbox_rowspan-overflow.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_rowspan-overflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_rowspan-overflow.html)
- [flexbox-safe-overflow-position-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-safe-overflow-position-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-safe-overflow-position-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-safe-overflow-position-001.html)
- [flexbox-safe-overflow-position-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-safe-overflow-position-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-safe-overflow-position-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-safe-overflow-position-002.html)
- [flexbox-safe-overflow-position-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-safe-overflow-position-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-safe-overflow-position-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-safe-overflow-position-003.html)
- [flexbox-safe-overflow-position-004.html](https://wpt.fyi/results/css/css-flexbox/flexbox-safe-overflow-position-004.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-safe-overflow-position-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-safe-overflow-position-004.html)
- [flexbox-safe-overflow-position-005.html](https://wpt.fyi/results/css/css-flexbox/flexbox-safe-overflow-position-005.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-safe-overflow-position-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-safe-overflow-position-005.html)
- [flexbox-safe-overflow-position-006.html](https://wpt.fyi/results/css/css-flexbox/flexbox-safe-overflow-position-006.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-safe-overflow-position-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-safe-overflow-position-006.html)
- [flexbox_width-overflow.html](https://wpt.fyi/results/css/css-flexbox/flexbox_width-overflow.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_width-overflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_width-overflow.html)
- [min-size-auto-overflow-clip.html](https://wpt.fyi/results/css/css-flexbox/min-size-auto-overflow-clip.html) [(live test)](http://wpt.live/css/css-flexbox/min-size-auto-overflow-clip.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/min-size-auto-overflow-clip.html)
- [negative-overflow-002.html](https://wpt.fyi/results/css/css-flexbox/negative-overflow-002.html) [(live test)](http://wpt.live/css/css-flexbox/negative-overflow-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/negative-overflow-002.html)
- [negative-overflow-003.html](https://wpt.fyi/results/css/css-flexbox/negative-overflow-003.html) [(live test)](http://wpt.live/css/css-flexbox/negative-overflow-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/negative-overflow-003.html)
- [negative-overflow.html](https://wpt.fyi/results/css/css-flexbox/negative-overflow.html) [(live test)](http://wpt.live/css/css-flexbox/negative-overflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/negative-overflow.html)
- [overflow-area-001.html](https://wpt.fyi/results/css/css-flexbox/overflow-area-001.html) [(live test)](http://wpt.live/css/css-flexbox/overflow-area-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/overflow-area-001.html)
- [overflow-area-002.html](https://wpt.fyi/results/css/css-flexbox/overflow-area-002.html) [(live test)](http://wpt.live/css/css-flexbox/overflow-area-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/overflow-area-002.html)
- [overflow-area-003.html](https://wpt.fyi/results/css/css-flexbox/overflow-area-003.html) [(live test)](http://wpt.live/css/css-flexbox/overflow-area-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/overflow-area-003.html)
- [overflow-auto-001.html](https://wpt.fyi/results/css/css-flexbox/overflow-auto-001.html) [(live test)](http://wpt.live/css/css-flexbox/overflow-auto-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/overflow-auto-001.html)
- [overflow-auto-002.html](https://wpt.fyi/results/css/css-flexbox/overflow-auto-002.html) [(live test)](http://wpt.live/css/css-flexbox/overflow-auto-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/overflow-auto-002.html)
- [overflow-auto-003.html](https://wpt.fyi/results/css/css-flexbox/overflow-auto-003.html) [(live test)](http://wpt.live/css/css-flexbox/overflow-auto-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/overflow-auto-003.html)
- [overflow-auto-004.html](https://wpt.fyi/results/css/css-flexbox/overflow-auto-004.html) [(live test)](http://wpt.live/css/css-flexbox/overflow-auto-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/overflow-auto-004.html)
- [overflow-auto-005.html](https://wpt.fyi/results/css/css-flexbox/overflow-auto-005.html) [(live test)](http://wpt.live/css/css-flexbox/overflow-auto-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/overflow-auto-005.html)
- [overflow-auto-006.html](https://wpt.fyi/results/css/css-flexbox/overflow-auto-006.html) [(live test)](http://wpt.live/css/css-flexbox/overflow-auto-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/overflow-auto-006.html)
- [overflow-auto-007.html](https://wpt.fyi/results/css/css-flexbox/overflow-auto-007.html) [(live test)](http://wpt.live/css/css-flexbox/overflow-auto-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/overflow-auto-007.html)
- [overflow-auto-008.html](https://wpt.fyi/results/css/css-flexbox/overflow-auto-008.html) [(live test)](http://wpt.live/css/css-flexbox/overflow-auto-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/overflow-auto-008.html)
- [overflow-top-left.html](https://wpt.fyi/results/css/css-flexbox/overflow-top-left.html) [(live test)](http://wpt.live/css/css-flexbox/overflow-top-left.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/overflow-top-left.html)
- [padding-overflow.html](https://wpt.fyi/results/css/css-flexbox/padding-overflow.html) [(live test)](http://wpt.live/css/css-flexbox/padding-overflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/padding-overflow.html)
- [text-overflow-on-flexbox-001.html](https://wpt.fyi/results/css/css-flexbox/text-overflow-on-flexbox-001.html) [(live test)](http://wpt.live/css/css-flexbox/text-overflow-on-flexbox-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/text-overflow-on-flexbox-001.html)
- [synthesize-vrl-baseline.html](https://wpt.fyi/results/css/css-flexbox/synthesize-vrl-baseline.html) [(live test)](http://wpt.live/css/css-flexbox/synthesize-vrl-baseline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/synthesize-vrl-baseline.html)

Flex containers are not block containers, and so some properties that were designed with the assumption of block layout don’t apply in the context of flex layout. In particular:

- <a id="ref-for-propdef-float"></a>

  <a id="ref-for-propdef-clear"></a>

  <a id="ref-for-flex-item④"></a>

  [float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float) and [clear](https://www.w3.org/TR/CSS2/visuren.html#propdef-clear) do not create floating or clearance of [flex item](#flex-item), and do not take it out-of-flow.

- <a id="ref-for-propdef-vertical-align"></a>

  [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) has no effect on a flex item.

- <a id="ref-for-selectordef-first-line①"></a>

  <a id="ref-for-selectordef-first-letter①"></a>

  <a id="ref-for-flex-container⑦"></a>

  <a id="ref-for-first-formatted-line"></a>

  the [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) and [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) pseudo-elements do not apply to [flex containers](#flex-container), and <a id="ref-for-flex-container⑧"></a>flex containers do not contribute a [first formatted line](https://www.w3.org/TR/css-pseudo-4/#first-formatted-line) or first letter to their ancestors.

Tests

- [align-content-wrap-004.html](https://wpt.fyi/results/css/css-flexbox/align-content-wrap-004.html) [(live test)](http://wpt.live/css/css-flexbox/align-content-wrap-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-wrap-004.html)
- [align-items-baseline-column-vert-lr-table-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-column-vert-lr-table-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-column-vert-lr-table-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-column-vert-lr-table-item.html)
- [align-items-baseline-vert-lr-column-horz-table-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-vert-lr-column-horz-table-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-vert-lr-column-horz-table-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-vert-lr-column-horz-table-item.html)
- [align-items-baseline-vert-rl-column-horz-table-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-vert-rl-column-horz-table-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-vert-rl-column-horz-table-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-vert-rl-column-horz-table-item.html)
- [flexbox_box-clear.html](https://wpt.fyi/results/css/css-flexbox/flexbox_box-clear.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_box-clear.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_box-clear.html)
- [flexbox_first-letter.html](https://wpt.fyi/results/css/css-flexbox/flexbox_first-letter.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_first-letter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_first-letter.html)
- [flexbox_first-line.html](https://wpt.fyi/results/css/css-flexbox/flexbox_first-line.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_first-line.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_first-line.html)
- [flexbox-ignores-first-letter.html](https://wpt.fyi/results/css/css-flexbox/flexbox-ignores-first-letter.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-ignores-first-letter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-ignores-first-letter.html)
- [flexbox_item-vertical-align.html](https://wpt.fyi/results/css/css-flexbox/flexbox_item-vertical-align.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_item-vertical-align.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_item-vertical-align.html)
- [flexbox-with-pseudo-elements-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-with-pseudo-elements-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-with-pseudo-elements-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-with-pseudo-elements-001.html)
- [flexbox-with-pseudo-elements-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-with-pseudo-elements-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-with-pseudo-elements-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-with-pseudo-elements-002.html)
- [flexbox-with-pseudo-elements-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-with-pseudo-elements-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-with-pseudo-elements-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-with-pseudo-elements-003.html)
- [flexible-box-float.html](https://wpt.fyi/results/css/css-flexbox/flexible-box-float.html) [(live test)](http://wpt.live/css/css-flexbox/flexible-box-float.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexible-box-float.html)
- [flex-item-vertical-align.html](https://wpt.fyi/results/css/css-flexbox/flex-item-vertical-align.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-vertical-align.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-vertical-align.html)
- [flex-vertical-align-effect.html](https://wpt.fyi/results/css/css-flexbox/flex-vertical-align-effect.html) [(live test)](http://wpt.live/css/css-flexbox/flex-vertical-align-effect.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-vertical-align-effect.html)
- [hittest-before-pseudo.html](https://wpt.fyi/results/css/css-flexbox/hittest-before-pseudo.html) [(live test)](http://wpt.live/css/css-flexbox/hittest-before-pseudo.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/hittest-before-pseudo.html)

<a id="ref-for-propdef-display④"></a>

<a id="ref-for-valdef-display-inline-flex②"></a>

<a id="ref-for-valdef-display-flex②"></a>

If an element’s specified [display](https://www.w3.org/TR/css-display-4/#propdef-display) is [inline-flex](#valdef-display-inline-flex), then its <a id="ref-for-propdef-display⑤"></a>display property computes to [flex](#valdef-display-flex) in certain circumstances: the table in [CSS 2.1 Section 9.7](https://www.w3.org/TR/CSS2/visuren.html#dis-pos-flo) is amended to contain an additional row, with <a id="ref-for-valdef-display-inline-flex③"></a>inline-flex in the "Specified Value" column and <a id="ref-for-valdef-display-flex③"></a>flex in the "Computed Value" column.

## <a id="flex-items"></a>4.  Flex Items

<a id="ref-for-flex-item⑤"></a>

<a id="ref-for-flex-container⑨"></a>

Loosely speaking, the [flex items](#flex-item) of a [flex container](#flex-container) are boxes representing its in-flow contents.

<a id="ref-for-flex-container①⓪"></a>

<a id="ref-for-flex-item⑥"></a>

<a id="ref-for-css-text-sequence"></a>

<a id="ref-for-anonymous"></a>

<a id="ref-for-block-container"></a>

<a id="ref-for-white-space"></a>

<a id="ref-for-propdef-white-space"></a>

<a id="ref-for-text-nodes"></a>

Each in-flow child of a [flex container](#flex-container) becomes a [flex item](#flex-item), and each child [text sequence](https://www.w3.org/TR/css-display-4/#css-text-sequence) is wrapped in an [anonymous](https://www.w3.org/TR/css-display-4/#anonymous) [block container](https://www.w3.org/TR/css-display-4/#block-container) <a id="ref-for-flex-item⑦"></a>flex item. However, if the entire <a id="ref-for-css-text-sequence①"></a>text sequences contains only [document white space characters](https://www.w3.org/TR/css-text-4/#white-space) (i.e. characters that can be affected by the [white-space](https://www.w3.org/TR/css-text-4/#propdef-white-space) property) it is instead not rendered (just as if its [text nodes](https://www.w3.org/TR/css-display-4/#text-nodes) were display:none).

Tests

- [anonymous-block.html](https://wpt.fyi/results/css/css-flexbox/anonymous-block.html) [(live test)](http://wpt.live/css/css-flexbox/anonymous-block.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/anonymous-block.html)
- [anonymous-flex-item-001.html](https://wpt.fyi/results/css/css-flexbox/anonymous-flex-item-001.html) [(live test)](http://wpt.live/css/css-flexbox/anonymous-flex-item-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/anonymous-flex-item-001.html)
- [anonymous-flex-item-002.html](https://wpt.fyi/results/css/css-flexbox/anonymous-flex-item-002.html) [(live test)](http://wpt.live/css/css-flexbox/anonymous-flex-item-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/anonymous-flex-item-002.html)
- [anonymous-flex-item-003.html](https://wpt.fyi/results/css/css-flexbox/anonymous-flex-item-003.html) [(live test)](http://wpt.live/css/css-flexbox/anonymous-flex-item-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/anonymous-flex-item-003.html)
- [anonymous-flex-item-004.html](https://wpt.fyi/results/css/css-flexbox/anonymous-flex-item-004.html) [(live test)](http://wpt.live/css/css-flexbox/anonymous-flex-item-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/anonymous-flex-item-004.html)
- [anonymous-flex-item-005.html](https://wpt.fyi/results/css/css-flexbox/anonymous-flex-item-005.html) [(live test)](http://wpt.live/css/css-flexbox/anonymous-flex-item-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/anonymous-flex-item-005.html)
- [anonymous-flex-item-006.html](https://wpt.fyi/results/css/css-flexbox/anonymous-flex-item-006.html) [(live test)](http://wpt.live/css/css-flexbox/anonymous-flex-item-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/anonymous-flex-item-006.html)
- [canvas-dynamic-change-001.html](https://wpt.fyi/results/css/css-flexbox/canvas-dynamic-change-001.html) [(live test)](http://wpt.live/css/css-flexbox/canvas-dynamic-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/canvas-dynamic-change-001.html)
- [column-flex-child-with-max-width.html](https://wpt.fyi/results/css/css-flexbox/column-flex-child-with-max-width.html) [(live test)](http://wpt.live/css/css-flexbox/column-flex-child-with-max-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/column-flex-child-with-max-width.html)
- [flexbox-whitespace-handling-001a.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-whitespace-handling-001a.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-whitespace-handling-001a.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-whitespace-handling-001a.xhtml)
- [flexbox-whitespace-handling-001b.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-whitespace-handling-001b.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-whitespace-handling-001b.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-whitespace-handling-001b.xhtml)
- [flexbox-whitespace-handling-002.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-whitespace-handling-002.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-whitespace-handling-002.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-whitespace-handling-002.xhtml)
- [hittest-anonymous-box.html](https://wpt.fyi/results/css/css-flexbox/hittest-anonymous-box.html) [(live test)](http://wpt.live/css/css-flexbox/hittest-anonymous-box.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/hittest-anonymous-box.html)
- [percentage-descendant-of-anonymous-flex-item.html](https://wpt.fyi/results/css/css-flexbox/percentage-descendant-of-anonymous-flex-item.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-descendant-of-anonymous-flex-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-descendant-of-anonymous-flex-item.html)
- [percentage-size-subitems-001.html](https://wpt.fyi/results/css/css-flexbox/percentage-size-subitems-001.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-size-subitems-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-size-subitems-001.html)
- [whitespace-in-flexitem-001.html](https://wpt.fyi/results/css/css-flexbox/whitespace-in-flexitem-001.html) [(live test)](http://wpt.live/css/css-flexbox/whitespace-in-flexitem-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/whitespace-in-flexitem-001.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cbe28400"></a>
>
> Examples of flex items:
>
> ```markup
> <div style="display:flex">
> 
>     <!-- flex item: block child -->
>     <div id="item1">block</div>
> 
>     <!-- flex item: floated element; floating is ignored -->
>     <div id="item2" style="float: left;">float</div>
> 
>     <!-- flex item: anonymous block box around inline content -->
>     anonymous item 3
> 
>     <!-- flex item: inline child -->
>     <span>
>         item 4
>         <!-- flex items do not split around blocks -->
>         <q style="display: block" id=not-an-item>item 4</q>
>         item 4
>     </span>
> </div>
> ```
>
> Flex items determined from above code block
>
> [](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/examples/flex-item-determination.html)
>
> ![Flex item containing block. Flex item containing float. (Anonymous, unstyleable) flex item containing anonymous item 3. Flex item containing three blocks in succession: Anonymous block containing item 4. \<q\> element block containing item 4. Anonymous block containing item 4.](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/flex-item-determination.png)
>
> 1.  Flex item containing `block`.
> 2.  Flex item containing `float`.
> 3.  (Anonymous, unstyleable) flex item containing `anonymous item 3`.
> 4.  Flex item containing three blocks in succession:
>     - Anonymous block containing `item 4`.
>     - `<q>` element block containing `item 4`.
>     - Anonymous block containing `item 4`.
>
> Note that the inter-element white space disappears: it does not become its own flex item, even though the inter-element text <em>does</em> get wrapped in an anonymous flex item.
>
> Note also that the anonymous item’s box is unstyleable, since there is no element to assign style rules to. Its contents will however inherit styles (such as font settings) from the flex container.

<a id="ref-for-flex-item⑧"></a>

<a id="ref-for-establish-an-independent-formatting-context"></a>

A [flex item](#flex-item) [establishes an independent formatting context](https://www.w3.org/TR/css-display-4/#establish-an-independent-formatting-context) for its contents. However, flex items themselves are <a id="flex-level"></a>flex-level boxes, not block-level boxes: they participate in their container’s flex formatting context, not in a block formatting context.

------------------------------------------------------------------------

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Authors reading this spec may want to [skip past the following box-generation and static position details](#item-margins).

<a id="ref-for-computed-value"></a>

<a id="ref-for-propdef-display⑥"></a>

<a id="ref-for-valdef-display-flex④"></a>

<a id="ref-for-valdef-display-inline-flex④"></a>

<a id="ref-for-blockify"></a>

If the [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [display](https://www.w3.org/TR/css-display-4/#propdef-display) value of an element’s nearest ancestor element (skipping display:contents ancestors) is [flex](#valdef-display-flex) or [inline-flex](#valdef-display-inline-flex), the element’s own <a id="ref-for-propdef-display⑦"></a>display value is [blockified](https://www.w3.org/TR/css-display-4/#blockify). (See [CSS2.1§9.7](https://www.w3.org/TR/CSS2/visuren.html#dis-pos-flo) [\[CSS2\]](#biblio-css2) and [CSS Display 3 § 2.7 Automatic Box Type Transformations](https://www.w3.org/TR/css-display-3/#transformations) for details on this type of <a id="ref-for-propdef-display⑧"></a>display value conversion.)

<a id="ref-for-valdef-display-flex⑤"></a>

<a id="ref-for-valdef-display-inline-flex⑤"></a>

<a id="ref-for-flex-container①①"></a>

<a id="ref-for-replaced-element"></a>

<a id="ref-for-propdef-display⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Blockification still occurs even when the [flex](#valdef-display-flex) or [inline-flex](#valdef-display-inline-flex) element does not end up generating a [flex container](#flex-container) box, e.g. when it is [replaced](https://www.w3.org/TR/css-display-4/#replaced-element) or in a [display: none](https://www.w3.org/TR/css-display-4/#propdef-display) subtree.

<a id="ref-for-propdef-display①⓪"></a>

<a id="ref-for-flex-item⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some values of [display](https://www.w3.org/TR/css-display-4/#propdef-display) normally trigger the creation of anonymous boxes around the original box. If such a box is a [flex item](#flex-item), it is blockified first, and so anonymous box creation will not happen. For example, two contiguous <a id="ref-for-flex-item①⓪"></a>flex items with <a id="ref-for-propdef-display①①"></a>display: table-cell will become two separate <a id="ref-for-propdef-display①②"></a>display: block <a id="ref-for-flex-item①①"></a>flex items, instead of being wrapped into a single anonymous table.

<a id="ref-for-propdef-display①③"></a>

<a id="ref-for-flex-item①②"></a>

<a id="ref-for-propdef-align-self④"></a>

<a id="ref-for-propdef-width②"></a>

<a id="ref-for-propdef-height②"></a>

<a id="ref-for-propdef-flex①"></a>

In the case of flex items with [display: table](https://www.w3.org/TR/css-display-4/#propdef-display), the table wrapper box becomes the [flex item](#flex-item), so the [align-self](#propdef-align-self) property applies to it. The contents of any caption boxes contribute to the calculation of the table wrapper box’s min-content and max-content sizes. However, like [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height), the [flex](#propdef-flex) longhands apply to the table box as follows: the <a id="ref-for-flex-item①③"></a>flex item’s final size is calculated by performing layout as if the distance between the table wrapper box’s edges and the table box’s content edges were all part of the table box’s border+padding area, and the table box were the <a id="ref-for-flex-item①④"></a>flex item.

Tests

- [flexbox-table-fixup-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-table-fixup-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-table-fixup-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-table-fixup-001.xhtml)

### <a id="abspos-items"></a>4.1.  Absolutely-Positioned Flex Children

<a id="ref-for-flex-container①②"></a>

As it is out-of-flow, an absolutely-positioned child of a [flex container](#flex-container) does not participate in flex layout.

<a id="ref-for-cross-axis②"></a>

<a id="ref-for-static-position-rectangle"></a>

<a id="ref-for-flex-container①③"></a>

<a id="ref-for-content-edge"></a>

<a id="ref-for-main-axis③"></a>

<a id="ref-for-margin-edge"></a>

<a id="ref-for-flex-item①⑤"></a>

The [cross-axis](#cross-axis) edges of the [static-position rectangle](https://www.w3.org/TR/css-position-3/#static-position-rectangle) of an absolutely-positioned child of a [flex container](#flex-container) are the [content edges](https://www.w3.org/TR/css-box-4/#content-edge) of the <a id="ref-for-flex-container①④"></a>flex container. The [main-axis](#main-axis) edges of the <a id="ref-for-static-position-rectangle①"></a>static-position rectangle are where the [margin edges](https://www.w3.org/TR/css-box-4/#margin-edge) of the child would be positioned if it were the sole [flex item](#flex-item) in the <a id="ref-for-flex-container①⑤"></a>flex container, assuming both the child and the flex container were fixed-size boxes of their used size. (For this purpose, the child’s auto margins are treated as zero.)

Tests

- [abspos-autopos-htb-ltr.html](https://wpt.fyi/results/css/css-flexbox/abspos/abspos-autopos-htb-ltr.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/abspos-autopos-htb-ltr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/abspos-autopos-htb-ltr.html)
- [abspos-autopos-htb-rtl.html](https://wpt.fyi/results/css/css-flexbox/abspos/abspos-autopos-htb-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/abspos-autopos-htb-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/abspos-autopos-htb-rtl.html)
- [abspos-autopos-vlr-ltr.html](https://wpt.fyi/results/css/css-flexbox/abspos/abspos-autopos-vlr-ltr.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/abspos-autopos-vlr-ltr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/abspos-autopos-vlr-ltr.html)
- [abspos-autopos-vlr-rtl.html](https://wpt.fyi/results/css/css-flexbox/abspos/abspos-autopos-vlr-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/abspos-autopos-vlr-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/abspos-autopos-vlr-rtl.html)
- [abspos-autopos-vrl-ltr.html](https://wpt.fyi/results/css/css-flexbox/abspos/abspos-autopos-vrl-ltr.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/abspos-autopos-vrl-ltr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/abspos-autopos-vrl-ltr.html)
- [abspos-autopos-vrl-rtl.html](https://wpt.fyi/results/css/css-flexbox/abspos/abspos-autopos-vrl-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/abspos-autopos-vrl-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/abspos-autopos-vrl-rtl.html)
- [dynamic-align-self-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/dynamic-align-self-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/dynamic-align-self-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/dynamic-align-self-001.html)
- [flex-abspos-staticpos-align-self-safe-002.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-safe-002.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-safe-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-safe-002.html)
- [flex-abspos-staticpos-align-self-safe-003.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-safe-003.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-safe-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-safe-003.html)
- [abspos-descendent-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/abspos-descendent-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/abspos-descendent-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/abspos-descendent-001.html)
- [flex-abspos-staticpos-align-content-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-content-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-content-001.html)
- [flex-abspos-staticpos-align-self-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-001.html)
- [flex-abspos-staticpos-align-self-002.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-002.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-002.html)
- [flex-abspos-staticpos-align-self-003.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-003.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-003.html)
- [flex-abspos-staticpos-align-self-004.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-004.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-004.html)
- [flex-abspos-staticpos-align-self-005.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-005.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-005.html)
- [flex-abspos-staticpos-align-self-006.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-006.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-006.html)
- [flex-abspos-staticpos-align-self-007.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-007.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-007.html)
- [flex-abspos-staticpos-align-self-008.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-008.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-008.html)
- [flex-abspos-staticpos-align-self-rtl-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-rtl-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-rtl-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-rtl-001.html)
- [flex-abspos-staticpos-align-self-rtl-002.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-rtl-002.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-rtl-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-rtl-002.html)
- [flex-abspos-staticpos-align-self-rtl-003.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-rtl-003.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-rtl-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-rtl-003.html)
- [flex-abspos-staticpos-align-self-rtl-004.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-rtl-004.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-rtl-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-rtl-004.html)
- [flex-abspos-staticpos-align-self-safe-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-safe-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-safe-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-safe-001.html)
- [flex-abspos-staticpos-align-self-vertWM-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-vertWM-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-vertWM-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-vertWM-001.html)
- [flex-abspos-staticpos-align-self-vertWM-002.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-vertWM-002.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-vertWM-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-vertWM-002.html)
- [flex-abspos-staticpos-align-self-vertWM-003.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-vertWM-003.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-vertWM-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-vertWM-003.html)
- [flex-abspos-staticpos-align-self-vertWM-004.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-vertWM-004.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-vertWM-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-align-self-vertWM-004.html)
- [flex-abspos-staticpos-fallback-justify-content-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-fallback-justify-content-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-fallback-justify-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-fallback-justify-content-001.html)
- [flex-abspos-staticpos-justify-content-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-001.html)
- [flex-abspos-staticpos-justify-content-002.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-002.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-002.html)
- [flex-abspos-staticpos-justify-content-003.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-003.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-003.html)
- [flex-abspos-staticpos-justify-content-004.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-004.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-004.html)
- [flex-abspos-staticpos-justify-content-005.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-005.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-005.html)
- [flex-abspos-staticpos-justify-content-006.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-006.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-006.html)
- [flex-abspos-staticpos-justify-content-007.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-007.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-007.html)
- [flex-abspos-staticpos-justify-content-008.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-008.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-008.html)
- [flex-abspos-staticpos-justify-content-rtl-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-rtl-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-rtl-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-rtl-001.html)
- [flex-abspos-staticpos-justify-content-rtl-002.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-rtl-002.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-rtl-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-rtl-002.html)
- [flex-abspos-staticpos-justify-content-vertWM-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-vertWM-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-vertWM-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-vertWM-001.html)
- [flex-abspos-staticpos-justify-content-vertWM-002.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-vertWM-002.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-vertWM-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-justify-content-vertWM-002.html)
- [flex-abspos-staticpos-justify-self-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-justify-self-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-justify-self-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-justify-self-001.html)
- [flex-abspos-staticpos-margin-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-margin-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-margin-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-margin-001.html)
- [flex-abspos-staticpos-margin-002.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-margin-002.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-margin-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-margin-002.html)
- [flex-abspos-staticpos-margin-003.html](https://wpt.fyi/results/css/css-flexbox/abspos/flex-abspos-staticpos-margin-003.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flex-abspos-staticpos-margin-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flex-abspos-staticpos-margin-003.html)
- [flexbox_absolute-atomic.html](https://wpt.fyi/results/css/css-flexbox/abspos/flexbox_absolute-atomic.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flexbox_absolute-atomic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flexbox_absolute-atomic.html)
- [flexbox-abspos-child-001a.html](https://wpt.fyi/results/css/css-flexbox/abspos/flexbox-abspos-child-001a.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flexbox-abspos-child-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flexbox-abspos-child-001a.html)
- [flexbox-abspos-child-001b.html](https://wpt.fyi/results/css/css-flexbox/abspos/flexbox-abspos-child-001b.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flexbox-abspos-child-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flexbox-abspos-child-001b.html)
- [flexbox-abspos-child-002.html](https://wpt.fyi/results/css/css-flexbox/abspos/flexbox-abspos-child-002.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flexbox-abspos-child-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flexbox-abspos-child-002.html)
- [flexbox_inline-abspos.html](https://wpt.fyi/results/css/css-flexbox/abspos/flexbox_inline-abspos.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/flexbox_inline-abspos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/flexbox_inline-abspos.html)
- [position-absolute-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-001.html)
- [position-absolute-002.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-002.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-002.html)
- [position-absolute-003.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-003.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-003.html)
- [position-absolute-004.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-004.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-004.html)
- [position-absolute-005.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-005.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-005.html)
- [position-absolute-006.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-006.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-006.html)
- [position-absolute-007.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-007.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-007.html)
- [position-absolute-008.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-008.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-008.html)
- [position-absolute-009.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-009.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-009.html)
- [position-absolute-010.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-010.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-010.html)
- [position-absolute-011.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-011.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-011.html)
- [position-absolute-012.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-012.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-012.html)
- [position-absolute-013.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-013.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-013.html)
- [position-absolute-014.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-014.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-014.html)
- [position-absolute-015.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-015.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-015.html)
- [position-absolute-containing-block-001.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-containing-block-001.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-containing-block-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-containing-block-001.html)
- [position-absolute-containing-block-002.html](https://wpt.fyi/results/css/css-flexbox/abspos/position-absolute-containing-block-002.html) [(live test)](http://wpt.live/css/css-flexbox/abspos/position-absolute-containing-block-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/abspos/position-absolute-containing-block-002.html)
- [flex-content-alignment-with-abspos-001.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-content-alignment-with-abspos-001.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-content-alignment-with-abspos-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-content-alignment-with-abspos-001.html)
- [dynamic-grid-flex-abspos.html](https://wpt.fyi/results/css/css-flexbox/dynamic-grid-flex-abspos.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-grid-flex-abspos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-grid-flex-abspos.html)
- [flexbox_stf-abspos.html](https://wpt.fyi/results/css/css-flexbox/flexbox_stf-abspos.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_stf-abspos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_stf-abspos.html)
- [flex-item-position-relative-001.html](https://wpt.fyi/results/css/css-flexbox/flex-item-position-relative-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-position-relative-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-position-relative-001.html)

<a id="ref-for-propdef-align-self⑤"></a>

<a id="ref-for-flex-container①⑥"></a>

<a id="ref-for-cross-axis③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bcd8b89a"></a> The effect of this is that if you set, for example, [align-self: center;](#propdef-align-self) on an absolutely-positioned child of a [flex container](#flex-container), auto offsets on the child will center it in the <a id="ref-for-flex-container①⑦"></a>flex container’s [cross axis](#cross-axis).

### <a id="item-margins"></a>4.2.  Flex Item Margins and Paddings

<a id="ref-for-flex-item①⑥"></a>

The margins of adjacent [flex items](#flex-item) do not [collapse](https://www.w3.org/TR/CSS2/box.html#collapsing-margins).

<a id="ref-for-flex-item①⑦"></a>

<a id="ref-for-block-box"></a>

<a id="ref-for-inline-size①"></a>

<a id="ref-for-containing-block"></a>

<a id="ref-for-writing-mode①"></a>

Percentage margins and paddings on [flex items](#flex-item), like those on [block boxes](https://www.w3.org/TR/css-display-4/#block-box), are resolved against the [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) of their [containing block](https://www.w3.org/TR/css-display-4/#containing-block), e.g. left/right/top/bottom percentages all resolve against their <a id="ref-for-containing-block①"></a>containing block’s <em>width</em> in horizontal [writing modes](https://www.w3.org/TR/css-writing-modes-4/#writing-mode).

Auto margins expand to absorb extra space in the corresponding dimension. They can be used for alignment, or to push adjacent flex items apart. See [Aligning with auto margins](#auto-margins).

Tests

- [empty-flex-box-and-margin-collapsing.html](https://wpt.fyi/results/css/css-flexbox/empty-flex-box-and-margin-collapsing.html) [(live test)](http://wpt.live/css/css-flexbox/empty-flex-box-and-margin-collapsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/empty-flex-box-and-margin-collapsing.html)
- [flex-container-margin.html](https://wpt.fyi/results/css/css-flexbox/flex-container-margin.html) [(live test)](http://wpt.live/css/css-flexbox/flex-container-margin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-container-margin.html)
- [flexitem-no-margin-collapsing.html](https://wpt.fyi/results/css/css-flexbox/flexitem-no-margin-collapsing.html) [(live test)](http://wpt.live/css/css-flexbox/flexitem-no-margin-collapsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexitem-no-margin-collapsing.html)
- [flex-margin-no-collapse.html](https://wpt.fyi/results/css/css-flexbox/flex-margin-no-collapse.html) [(live test)](http://wpt.live/css/css-flexbox/flex-margin-no-collapse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-margin-no-collapse.html)
- [negative-margins-001.html](https://wpt.fyi/results/css/css-flexbox/negative-margins-001.html) [(live test)](http://wpt.live/css/css-flexbox/negative-margins-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/negative-margins-001.html)

### <a id="painting"></a>4.3.  Flex Item Z-Ordering

<a id="ref-for-flex-item①⑧"></a>

<a id="ref-for-propdef-order①"></a>

<a id="ref-for-propdef-z-index"></a>

<a id="ref-for-valdef-z-index-auto"></a>

<a id="ref-for-propdef-position"></a>

<a id="ref-for-valdef-position-static"></a>

<a id="ref-for-valdef-position-relative"></a>

[Flex items](#flex-item) paint exactly the same as inline blocks [\[CSS2\]](#biblio-css2), except that [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order)-modified document order is used in place of raw document order, and [z-index](https://www.w3.org/TR/CSS2/visuren.html#propdef-z-index) values other than [auto](https://drafts.csswg.org/css2/#valdef-z-index-auto) create a stacking context even if [position](https://www.w3.org/TR/css-position-3/#propdef-position) is [static](https://www.w3.org/TR/css-position-3/#valdef-position-static) (behaving exactly as if <a id="ref-for-propdef-position①"></a>position were [relative](https://www.w3.org/TR/css-position-3/#valdef-position-relative)).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Descendants that are positioned outside a flex item still participate in any stacking context established by the flex item.

Tests

- [flexbox-paint-ordering-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-paint-ordering-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-paint-ordering-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-paint-ordering-001.xhtml)
- [flexbox-paint-ordering-002.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-paint-ordering-002.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-paint-ordering-002.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-paint-ordering-002.xhtml)
- [flexbox-paint-ordering-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-paint-ordering-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-paint-ordering-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-paint-ordering-003.html)
- [flex-item-z-ordering-001.html](https://wpt.fyi/results/css/css-flexbox/flex-item-z-ordering-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-z-ordering-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-z-ordering-001.html)
- [flex-item-z-ordering-002.html](https://wpt.fyi/results/css/css-flexbox/flex-item-z-ordering-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-z-ordering-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-z-ordering-002.html)
- [flexbox-items-as-stacking-contexts-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-items-as-stacking-contexts-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-items-as-stacking-contexts-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-items-as-stacking-contexts-001.xhtml)
- [flexbox-items-as-stacking-contexts-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-items-as-stacking-contexts-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-items-as-stacking-contexts-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-items-as-stacking-contexts-002.html)
- [flexbox-items-as-stacking-contexts-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-items-as-stacking-contexts-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-items-as-stacking-contexts-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-items-as-stacking-contexts-003.html)
- [hittest-overlapping-margin.html](https://wpt.fyi/results/css/css-flexbox/hittest-overlapping-margin.html) [(live test)](http://wpt.live/css/css-flexbox/hittest-overlapping-margin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/hittest-overlapping-margin.html)
- [hittest-overlapping-order.html](https://wpt.fyi/results/css/css-flexbox/hittest-overlapping-order.html) [(live test)](http://wpt.live/css/css-flexbox/hittest-overlapping-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/hittest-overlapping-order.html)
- [hittest-overlapping-relative.html](https://wpt.fyi/results/css/css-flexbox/hittest-overlapping-relative.html) [(live test)](http://wpt.live/css/css-flexbox/hittest-overlapping-relative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/hittest-overlapping-relative.html)

### <a id="visibility-collapse"></a>4.4.  Collapsed Items

<a id="ref-for-collapsed-flex-item"></a>

<a id="ref-for-flex-container①⑧"></a>

<a id="ref-for-main-size②"></a>

<a id="ref-for-cross-size①"></a>

Specifying visibility:collapse on a flex item causes it to become a <a id="collapsed-flex-item"></a>collapsed flex item, producing an effect similar to visibility:collapse on a table-row or table-column: the [collapsed flex item](#collapsed-flex-item) is removed from rendering entirely, but leaves behind a "strut" that keeps the flex line’s cross-size stable. Thus, if a flex container has only one flex line, dynamically collapsing or uncollapsing items may change the [flex container](#flex-container)’s [main size](#main-size), but is guaranteed to have no effect on its [cross size](#cross-size) and won’t cause the rest of the page’s layout to "wobble". Flex line wrapping <em>is</em> re-done after collapsing, however, so the cross-size of a flex container with multiple lines might or might not change.

<a id="ref-for-collapsed-flex-item①"></a>

Though [collapsed flex items](#collapsed-flex-item) aren’t rendered, they do appear in the [formatting structure](https://www.w3.org/TR/CSS2/intro.html#formatting-structure). Therefore, unlike on display:none items [\[CSS2\]](#biblio-css2), effects that depend on a box appearing in the formatting structure (like incrementing counters or running animations and transitions) still operate on collapsed items.

Tests

- [flexbox_visibility-collapse.html](https://wpt.fyi/results/css/css-flexbox/flexbox_visibility-collapse.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_visibility-collapse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_visibility-collapse.html)
- [flexbox_visibility-collapse-line-wrapping.html](https://wpt.fyi/results/css/css-flexbox/flexbox_visibility-collapse-line-wrapping.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_visibility-collapse-line-wrapping.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_visibility-collapse-line-wrapping.html)

<a id="ref-for-propdef-visibility"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ec2b5bd9"></a> In the following example, a sidebar is sized to fit its content. [visibility: collapse](https://www.w3.org/TR/css-display-4/#propdef-visibility) is used to dynamically hide parts of a navigation sidebar without affecting its width, even though the widest item (“Architecture”) is in a collapsed section.
>
> Sample live rendering for example code below
>
> <a id="ref-for-propdef-visibility①"></a>
>
> <a id="ref-for-propdef-display①④"></a>
>
> <a id="visibility-collapse-example"></a>
>
> - <a id="nav-about"></a> [About](#nav-about)
>   - [History](css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
>   - [Mission](css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
>   - [People](css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
> - <a id="nav-projects"></a> [Projects](#nav-projects)
>   - [Art](css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
>   - [Architecture](css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
>   - [Music](css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
> - <a id="nav-interact"></a> [Interact](#nav-interact)
>   - [Blog](css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
>   - [Forums](css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
>
> Hover over the menu to the left: each section expands to show its sub-items. In order to keep the sidebar width (and this main area width) stable, [visibility: collapse](https://www.w3.org/TR/css-display-4/#propdef-visibility) is used instead of [display: none](https://www.w3.org/TR/css-display-4/#propdef-display). This results in a sidebar that is always wide enough for the word “Architecture”, even though it is not always visible.
>
> ```css
> @media (min-width: 60em) {
>   /* two column layout only when enough room (relative to default text size) */
>   div { display: flex; }
>   #main {
>     flex: 1;         /* Main takes up all remaining space */
>     order: 1;        /* Place it after (to the right of) the navigation */
>     min-width: 12em; /* Optimize main content area sizing */
>   }
> }
> /* menu items use flex layout so that visibility:collapse will work */
> nav > ul > li {
>   display: flex;
>   flex-flow: column;
> }
> /* dynamically collapse submenus when not targeted */
> nav > ul > li:not(:target):not(:hover) > ul {
>   visibility: collapse;
> }
> ```
>
> ```markup
> <div>
>   <article id="main">
>     Interesting Stuff to Read
>   </article>
>   <nav>
>     <ul>
>       <li id="nav-about"><a href="#nav-about">About</a>
>         …
>       <li id="nav-projects"><a href="#nav-projects">Projects</a>
>         <ul>
>           <li><a href="…">Art</a>
>           <li><a href="…">Architecture</a>
>           <li><a href="…">Music</a>
>         </ul>
>       <li id="nav-interact"><a href="#nav-interact">Interact</a>
>         …
>     </ul>
>   </nav>
> </div>
> <footer>
> …
> ```
<a id="ref-for-collapsed-flex-item②"></a>

To compute the size of the strut, flex layout is first performed with all items uncollapsed, and then re-run with each [collapsed flex item](#collapsed-flex-item) replaced by a strut that maintains the original cross-size of the item’s original line. See the [Flex Layout Algorithm](#layout-algorithm) for the normative definition of how visibility:collapse interacts with flex layout.

<a id="ref-for-propdef-visibility②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Using visibility:collapse on any flex items will cause the flex layout algorithm to repeat partway through, re-running the most expensive steps. It’s recommended that authors continue to use display:none to hide items if the items will not be dynamically collapsed and uncollapsed, as that is more efficient for the layout engine. (Since only part of the steps need to be repeated when [visibility](https://www.w3.org/TR/css-display-4/#propdef-visibility) is changed, however, 'visibility: collapse' is still recommended for dynamic cases.)

### <a id="min-size-auto"></a>4.5.  Automatic Minimum Size of Flex Items

<a id="ref-for-valdef-width-auto"></a>

<a id="ref-for-automatic-minimum-size"></a>

<a id="ref-for-propdef-min-width②"></a>

<a id="ref-for-propdef-min-height②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) keyword, representing an [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size), is the new initial value of the [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width) and [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) properties. The keyword was previously defined in this specification, but is now defined in the [CSS Sizing](#biblio-css-sizing-3) module.

<a id="ref-for-min-width"></a>

<a id="ref-for-flex-item①⑨"></a>

<a id="ref-for-main-axis④"></a>

<a id="ref-for-automatic-minimum-size①"></a>

<a id="ref-for-computed-value①"></a>

<a id="ref-for-propdef-overflow①"></a>

<a id="ref-for-non-scrollable-overflow-value"></a>

<a id="ref-for-content-based-minimum-size"></a>

<a id="ref-for-scroll-container"></a>

To provide a more reasonable default [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width) for [flex items](#flex-item), the used value of a [main axis](#main-axis) [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size) on a <a id="ref-for-flex-item②⓪"></a>flex item whose [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) value is [non-scrollable](https://www.w3.org/TR/css-overflow-3/#non-scrollable-overflow-value) is its [content-based minimum size](#content-based-minimum-size); for [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container) the <a id="ref-for-automatic-minimum-size②"></a>automatic minimum size is zero, as usual.

<a id="ref-for-flex-item②①"></a>

<a id="ref-for-replaced-element①"></a>

The <a id="content-based-minimum-size"></a>content-based minimum size of a [flex item](#flex-item) differs depending on whether the <a id="ref-for-flex-item②②"></a>flex item is [replaced](https://www.w3.org/TR/css-display-4/#replaced-element) or not:

<a id="ref-for-replaced-element②"></a>

For [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element)

<a id="ref-for-content-size-suggestion"></a>

<a id="ref-for-transferred-size-suggestion"></a>

<a id="ref-for-specified-size-suggestion"></a>

Use the smaller of the [content size suggestion](#content-size-suggestion) and the [transferred size suggestion](#transferred-size-suggestion) (if one exists), capped by the [specified size suggestion](#specified-size-suggestion) (if one exists).

<a id="ref-for-non-replaced"></a>

For [non-replaced elements](https://www.w3.org/TR/css-display-4/#non-replaced)

<a id="ref-for-content-size-suggestion①"></a>

<a id="ref-for-transferred-size-suggestion①"></a>

<a id="ref-for-specified-size-suggestion①"></a>

Use the larger of the [content size suggestion](#content-size-suggestion) and the [transferred size suggestion](#transferred-size-suggestion) (if one exists), capped by the [specified size suggestion](#specified-size-suggestion) (if one exists).

<a id="ref-for-max-width"></a>

<a id="ref-for-main-size③"></a>

<a id="ref-for-definite"></a>

In either case, the size is clamped by the [maximum](https://www.w3.org/TR/css-sizing-3/#max-width) [main size](#main-size) if it’s [definite](#definite).

Tests

- [image-items-flake-001.html](https://wpt.fyi/results/css/css-flexbox/image-items-flake-001.html) [(live test)](http://wpt.live/css/css-flexbox/image-items-flake-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-items-flake-001.html)

<a id="ref-for-content-size-suggestion②"></a>

<a id="ref-for-specified-size-suggestion②"></a>

<a id="ref-for-transferred-size-suggestion②"></a>

<a id="ref-for-content-based-minimum-size①"></a>

The [content size suggestion](#content-size-suggestion), [specified size suggestion](#specified-size-suggestion), and [transferred size suggestion](#transferred-size-suggestion) used in this calculation account for the relevant min/max/preferred size properties so that the [content-based minimum size](#content-based-minimum-size) does not interfere with any author-provided constraints, and are defined below:

<a id="specified-size-suggestion"></a>specified size suggestion  
<a id="ref-for-specified-size-suggestion③"></a>

<a id="ref-for-automatic-size"></a>

<a id="ref-for-definite①"></a>

<a id="ref-for-main-size④"></a>

<a id="ref-for-preferred-size"></a>

If the item’s [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) [main size](#main-size) is [definite](#definite) and not [automatic](https://www.w3.org/TR/css-sizing-3/#automatic-size), then the [specified size suggestion](#specified-size-suggestion) is that size. It is otherwise undefined.

<a id="transferred-size-suggestion"></a>transferred size suggestion  
<a id="ref-for-max-width①"></a>

<a id="ref-for-min-width①"></a>

<a id="ref-for-transferred-size-suggestion③"></a>

<a id="ref-for-definite②"></a>

<a id="ref-for-cross-size②"></a>

<a id="ref-for-preferred-size①"></a>

<a id="ref-for-preferred-aspect-ratio"></a>

If the item has a [preferred aspect ratio](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio) and its [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) [cross size](#cross-size) is [definite](#definite), then the [transferred size suggestion](#transferred-size-suggestion) is that size (clamped by its [minimum](https://www.w3.org/TR/css-sizing-3/#min-width) and [maximum](https://www.w3.org/TR/css-sizing-3/#max-width) <a id="ref-for-cross-size③"></a>cross sizes if they are <a id="ref-for-definite③"></a>definite), converted through the aspect ratio. It is otherwise undefined.

<a id="content-size-suggestion"></a>content size suggestion  
<a id="ref-for-cross-size④"></a>

<a id="ref-for-max-width②"></a>

<a id="ref-for-min-width②"></a>

<a id="ref-for-definite④"></a>

<a id="ref-for-preferred-aspect-ratio①"></a>

<a id="ref-for-main-axis⑤"></a>

<a id="ref-for-min-content"></a>

<a id="ref-for-content-size-suggestion③"></a>

The [content size suggestion](#content-size-suggestion) is the [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content) in the [main axis](#main-axis), clamped, if it has a [preferred aspect ratio](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio), by any [definite](#definite) [minimum](https://www.w3.org/TR/css-sizing-3/#min-width) and [maximum](https://www.w3.org/TR/css-sizing-3/#max-width) [cross sizes](#cross-size) converted through the aspect ratio.

Tests

- [content-height-with-scrollbars.html](https://wpt.fyi/results/css/css-flexbox/content-height-with-scrollbars.html) [(live test)](http://wpt.live/css/css-flexbox/content-height-with-scrollbars.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/content-height-with-scrollbars.html)
- [fieldset-as-item-dynamic.html](https://wpt.fyi/results/css/css-flexbox/fieldset-as-item-dynamic.html) [(live test)](http://wpt.live/css/css-flexbox/fieldset-as-item-dynamic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/fieldset-as-item-dynamic.html)
- [fieldset-as-item-overflow.html](https://wpt.fyi/results/css/css-flexbox/fieldset-as-item-overflow.html) [(live test)](http://wpt.live/css/css-flexbox/fieldset-as-item-overflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/fieldset-as-item-overflow.html)
- [flex-aspect-ratio-img-column-001.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-001.html)
- [flex-aspect-ratio-img-column-002.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-002.html)
- [flex-aspect-ratio-img-column-003.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-003.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-003.html)
- [flex-aspect-ratio-img-column-004.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-004.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-004.html)
- [flex-aspect-ratio-img-column-005.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-005.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-005.html)
- [flex-aspect-ratio-img-column-006.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-006.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-006.html)
- [flex-aspect-ratio-img-column-007.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-007.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-007.html)
- [flex-aspect-ratio-img-column-008.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-008.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-008.html)
- [flex-aspect-ratio-img-column-009.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-009.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-009.html)
- [flex-aspect-ratio-img-column-010.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-010.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-010.html)
- [flex-aspect-ratio-img-column-011.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-011.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-011.html)
- [flex-aspect-ratio-img-column-012.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-012.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-012.html)
- [flex-aspect-ratio-img-column-013.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-013.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-013.html)
- [flex-aspect-ratio-img-column-014.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-014.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-014.html)
- [flex-aspect-ratio-img-column-015.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-015.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-015.html)
- [flex-aspect-ratio-img-column-016.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-016.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-016.html)
- [flex-aspect-ratio-img-column-017.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-017.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-017.html)
- [flex-aspect-ratio-img-column-018.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-column-018.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-column-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-column-018.html)
- [flex-aspect-ratio-img-row-001.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-001.html)
- [flex-aspect-ratio-img-row-002.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-002.html)
- [flex-aspect-ratio-img-row-003.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-003.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-003.html)
- [flex-aspect-ratio-img-row-004.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-004.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-004.html)
- [flex-aspect-ratio-img-row-005.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-005.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-005.html)
- [flex-aspect-ratio-img-row-006.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-006.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-006.html)
- [flex-aspect-ratio-img-row-007.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-007.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-007.html)
- [flex-aspect-ratio-img-row-008.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-008.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-008.html)
- [flex-aspect-ratio-img-row-009.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-009.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-009.html)
- [flex-aspect-ratio-img-row-010.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-010.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-010.html)
- [flex-aspect-ratio-img-row-011.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-011.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-011.html)
- [flex-aspect-ratio-img-row-012.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-012.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-012.html)
- [flex-aspect-ratio-img-row-013.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-013.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-013.html)
- [flex-aspect-ratio-img-row-014.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-014.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-014.html)
- [flex-aspect-ratio-img-row-015.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-015.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-015.html)
- [flex-aspect-ratio-img-row-016.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-016.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-016.html)
- [flex-aspect-ratio-img-row-017.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-row-017.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-row-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-row-017.html)
- [flex-aspect-ratio-img-vert-lr.html](https://wpt.fyi/results/css/css-flexbox/flex-aspect-ratio-img-vert-lr.html) [(live test)](http://wpt.live/css/css-flexbox/flex-aspect-ratio-img-vert-lr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-aspect-ratio-img-vert-lr.html)
- [flexbox-definite-cross-size-constrained-percentage.html](https://wpt.fyi/results/css/css-flexbox/flexbox-definite-cross-size-constrained-percentage.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-definite-cross-size-constrained-percentage.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-definite-cross-size-constrained-percentage.html)
- [flexbox-min-height-auto-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-height-auto-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-height-auto-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-height-auto-001.html)
- [flexbox-min-height-auto-002a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-height-auto-002a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-height-auto-002a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-height-auto-002a.html)
- [flexbox-min-height-auto-002b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-height-auto-002b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-height-auto-002b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-height-auto-002b.html)
- [flexbox-min-height-auto-002c.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-height-auto-002c.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-height-auto-002c.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-height-auto-002c.html)
- [flexbox-min-height-auto-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-height-auto-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-height-auto-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-height-auto-003.html)
- [flexbox-min-height-auto-004.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-height-auto-004.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-height-auto-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-height-auto-004.html)
- [flexbox-min-width-auto-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-width-auto-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-width-auto-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-width-auto-001.html)
- [flexbox-min-width-auto-002a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-width-auto-002a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-width-auto-002a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-width-auto-002a.html)
- [flexbox-min-width-auto-002b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-width-auto-002b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-width-auto-002b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-width-auto-002b.html)
- [flexbox-min-width-auto-002c.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-width-auto-002c.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-width-auto-002c.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-width-auto-002c.html)
- [flexbox-min-width-auto-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-width-auto-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-width-auto-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-width-auto-003.html)
- [flexbox-min-width-auto-004.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-width-auto-004.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-width-auto-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-width-auto-004.html)
- [flexbox-min-width-auto-005.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-width-auto-005.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-width-auto-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-width-auto-005.html)
- [flexbox-min-width-auto-006.html](https://wpt.fyi/results/css/css-flexbox/flexbox-min-width-auto-006.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-min-width-auto-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-min-width-auto-006.html)
- [flex-item-compressible-001.html](https://wpt.fyi/results/css/css-flexbox/flex-item-compressible-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-compressible-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-compressible-001.html)
- [flex-item-compressible-002.html](https://wpt.fyi/results/css/css-flexbox/flex-item-compressible-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-compressible-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-compressible-002.html)
- [flexitem-stretch-image.html](https://wpt.fyi/results/css/css-flexbox/flexitem-stretch-image.html) [(live test)](http://wpt.live/css/css-flexbox/flexitem-stretch-image.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexitem-stretch-image.html)
- [flexitem-stretch-range.html](https://wpt.fyi/results/css/css-flexbox/flexitem-stretch-range.html) [(live test)](http://wpt.live/css/css-flexbox/flexitem-stretch-range.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexitem-stretch-range.html)
- [flex-minimum-height-flex-items-001.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-001.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-001.xht)
- [flex-minimum-height-flex-items-002.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-002.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-002.xht)
- [flex-minimum-height-flex-items-003.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-003.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-003.xht)
- [flex-minimum-height-flex-items-004.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-004.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-004.xht)
- [flex-minimum-height-flex-items-005.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-005.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-005.xht)
- [flex-minimum-height-flex-items-006.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-006.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-006.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-006.xht)
- [flex-minimum-height-flex-items-007.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-007.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-007.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-007.xht)
- [flex-minimum-height-flex-items-008.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-008.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-008.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-008.xht)
- [flex-minimum-height-flex-items-009.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-009.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-009.html)
- [flex-minimum-height-flex-items-010.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-010.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-010.html)
- [flex-minimum-height-flex-items-011.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-011.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-011.xht)
- [flex-minimum-height-flex-items-012.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-012.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-012.html)
- [flex-minimum-height-flex-items-013.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-013.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-013.html)
- [flex-minimum-height-flex-items-014.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-014.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-014.html)
- [flex-minimum-height-flex-items-015.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-015.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-015.html)
- [flex-minimum-height-flex-items-016.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-016.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-016.html)
- [flex-minimum-height-flex-items-017.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-017.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-017.html)
- [flex-minimum-height-flex-items-018.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-018.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-018.html)
- [flex-minimum-height-flex-items-019.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-019.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-019.html)
- [flex-minimum-height-flex-items-020.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-020.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-020.html)
- [flex-minimum-height-flex-items-021.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-021.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-021.html)
- [flex-minimum-height-flex-items-022.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-022.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-022.html)
- [flex-minimum-height-flex-items-023.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-023.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-023.html)
- [flex-minimum-height-flex-items-024.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-024.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-024.html)
- [flex-minimum-height-flex-items-025.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-025.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-025.html)
- [flex-minimum-height-flex-items-026.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-026.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-026.html)
- [flex-minimum-height-flex-items-027.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-027.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-027.html)
- [flex-minimum-height-flex-items-028.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-028.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-028.html)
- [flex-minimum-height-flex-items-029.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-029.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-029.html)
- [flex-minimum-height-flex-items-030.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-030.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-030.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-030.html)
- [flex-minimum-height-flex-items-031.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-height-flex-items-031.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-height-flex-items-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-height-flex-items-031.html)
- [flex-minimum-size-001.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-size-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-size-001.html)
- [flex-minimum-size-002.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-size-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-size-002.html)
- [flex-minimum-size-003.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-size-003.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-size-003.html)
- [flex-minimum-width-flex-items-001.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-001.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-001.xht)
- [flex-minimum-width-flex-items-002.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-002.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-002.xht)
- [flex-minimum-width-flex-items-003.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-003.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-003.xht)
- [flex-minimum-width-flex-items-004.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-004.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-004.xht)
- [flex-minimum-width-flex-items-005.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-005.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-005.xht)
- [flex-minimum-width-flex-items-006.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-006.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-006.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-006.xht)
- [flex-minimum-width-flex-items-007.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-007.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-007.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-007.xht)
- [flex-minimum-width-flex-items-008.xht](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-008.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-008.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-008.xht)
- [flex-minimum-width-flex-items-009.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-009.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-009.html)
- [flex-minimum-width-flex-items-010.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-010.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-010.html)
- [flex-minimum-width-flex-items-011.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-011.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-011.html)
- [flex-minimum-width-flex-items-012.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-012.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-012.html)
- [flex-minimum-width-flex-items-013.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-013.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-013.html)
- [flex-minimum-width-flex-items-014.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-014.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-014.html)
- [flex-minimum-width-flex-items-015.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-015.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-015.html)
- [flex-minimum-width-flex-items-016.html](https://wpt.fyi/results/css/css-flexbox/flex-minimum-width-flex-items-016.html) [(live test)](http://wpt.live/css/css-flexbox/flex-minimum-width-flex-items-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-minimum-width-flex-items-016.html)
- [flexbox_computedstyle_min-auto-size.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_min-auto-size.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_min-auto-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_min-auto-size.html)
- [flexbox_computedstyle_min-height-auto.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_min-height-auto.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_min-height-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_min-height-auto.html)
- [flexbox_computedstyle_min-width-auto.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_min-width-auto.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_min-width-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_min-width-auto.html)
- [select-element-zero-height-001.html](https://wpt.fyi/results/css/css-flexbox/select-element-zero-height-001.html) [(live test)](http://wpt.live/css/css-flexbox/select-element-zero-height-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/select-element-zero-height-001.html)
- [select-element-zero-height-002.html](https://wpt.fyi/results/css/css-flexbox/select-element-zero-height-002.html) [(live test)](http://wpt.live/css/css-flexbox/select-element-zero-height-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/select-element-zero-height-002.html)

<a id="ref-for-content-based-minimum-size②"></a>

<a id="ref-for-intrinsic-size-contribution"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [content-based minimum size](#content-based-minimum-size) is a type of [intrinsic size contribution](https://www.w3.org/TR/css-sizing-3/#intrinsic-size-contribution), and thus the cyclic percentage provisions in [CSS Sizing 3 § 5.2 Intrinsic Contributions](https://www.w3.org/TR/css-sizing-3/#intrinsic-contribution) apply.

<a id="ref-for-min-content①"></a>

<a id="ref-for-content-based-minimum-size③"></a>

<a id="ref-for-propdef-width③"></a>

<a id="ref-for-definite⑤"></a>

<a id="ref-for-behave-as-auto"></a>

For the purpose of calculating an intrinsic size of the box (e.g. the box’s [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content)), a [content-based minimum size](#content-based-minimum-size) causes the box’s size in that axis to become indefinite (even if e.g. its [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) property specifies a [definite](#definite) size). Note this means that percentages calculated against this size will [behave as auto](https://www.w3.org/TR/css-sizing-3/#behave-as-auto).

<a id="ref-for-content-based-minimum-size④"></a>

<a id="ref-for-valdef-width-min-content"></a>

<a id="ref-for-min-width③"></a>

For any purpose <em>other than</em> calculating intrinsic sizes, a [content-based minimum size](#content-based-minimum-size) (unlike an explicit [min-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content)/etc [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width)) does not force the box’s size to become indefinite. However, if a percentage resolved against the box’s size <em>before</em> this minimum was applied, it must be re-resolved against the new size after it is applied.

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="min-size-opt"></a> Note that while a content-based minimum size is often appropriate, and helps prevent content from overlapping or spilling outside its container, in some cases it is not:
>
> <a id="ref-for-propdef-min-width③"></a>
>
> In particular, if flex sizing is being used for a major content area of a document, it is better to set an explicit font-relative minimum width such as [min-width: 12em](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width). A content-based minimum width could result in a large table or large image stretching the size of the entire content area into an overflow zone, and thereby making lines of text gratuitously long and hard to read.
>
> Note also, when content-based sizing is used on an item with large amounts of content, the layout engine must traverse all of this content before finding its minimum size, whereas if the author sets an explicit minimum, this is not necessary. (For items with small amounts of content, however, this traversal is trivial and therefore not a performance concern.)

Tests

- [aspect-ratio-intrinsic-size-001.html](https://wpt.fyi/results/css/css-flexbox/aspect-ratio-intrinsic-size-001.html) [(live test)](http://wpt.live/css/css-flexbox/aspect-ratio-intrinsic-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/aspect-ratio-intrinsic-size-001.html)
- [aspect-ratio-intrinsic-size-002.html](https://wpt.fyi/results/css/css-flexbox/aspect-ratio-intrinsic-size-002.html) [(live test)](http://wpt.live/css/css-flexbox/aspect-ratio-intrinsic-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/aspect-ratio-intrinsic-size-002.html)
- [aspect-ratio-intrinsic-size-003.html](https://wpt.fyi/results/css/css-flexbox/aspect-ratio-intrinsic-size-003.html) [(live test)](http://wpt.live/css/css-flexbox/aspect-ratio-intrinsic-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/aspect-ratio-intrinsic-size-003.html)
- [aspect-ratio-intrinsic-size-004.html](https://wpt.fyi/results/css/css-flexbox/aspect-ratio-intrinsic-size-004.html) [(live test)](http://wpt.live/css/css-flexbox/aspect-ratio-intrinsic-size-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/aspect-ratio-intrinsic-size-004.html)
- [aspect-ratio-intrinsic-size-005.html](https://wpt.fyi/results/css/css-flexbox/aspect-ratio-intrinsic-size-005.html) [(live test)](http://wpt.live/css/css-flexbox/aspect-ratio-intrinsic-size-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/aspect-ratio-intrinsic-size-005.html)
- [aspect-ratio-intrinsic-size-006.html](https://wpt.fyi/results/css/css-flexbox/aspect-ratio-intrinsic-size-006.html) [(live test)](http://wpt.live/css/css-flexbox/aspect-ratio-intrinsic-size-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/aspect-ratio-intrinsic-size-006.html)
- [aspect-ratio-intrinsic-size-007.html](https://wpt.fyi/results/css/css-flexbox/aspect-ratio-intrinsic-size-007.html) [(live test)](http://wpt.live/css/css-flexbox/aspect-ratio-intrinsic-size-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/aspect-ratio-intrinsic-size-007.html)
- [aspect-ratio-intrinsic-size-008.html](https://wpt.fyi/results/css/css-flexbox/aspect-ratio-intrinsic-size-008.html) [(live test)](http://wpt.live/css/css-flexbox/aspect-ratio-intrinsic-size-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/aspect-ratio-intrinsic-size-008.html)
- [aspect-ratio-intrinsic-size-009.html](https://wpt.fyi/results/css/css-flexbox/aspect-ratio-intrinsic-size-009.html) [(live test)](http://wpt.live/css/css-flexbox/aspect-ratio-intrinsic-size-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/aspect-ratio-intrinsic-size-009.html)
- [aspect-ratio-intrinsic-size-010.html](https://wpt.fyi/results/css/css-flexbox/aspect-ratio-intrinsic-size-010.html) [(live test)](http://wpt.live/css/css-flexbox/aspect-ratio-intrinsic-size-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/aspect-ratio-intrinsic-size-010.html)
- [aspect-ratio-transferred-max-size.html](https://wpt.fyi/results/css/css-flexbox/aspect-ratio-transferred-max-size.html) [(live test)](http://wpt.live/css/css-flexbox/aspect-ratio-transferred-max-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/aspect-ratio-transferred-max-size.html)
- [flex-item-content-is-min-width-max-content.html](https://wpt.fyi/results/css/css-flexbox/flex-item-content-is-min-width-max-content.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-content-is-min-width-max-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-content-is-min-width-max-content.html)

## <a id="flow-order"></a>5.  Ordering and Orientation

<a id="ref-for-propdef-float①"></a>

<a id="ref-for-propdef-clear①"></a>

<a id="ref-for-propdef-flex-direction"></a>

<a id="ref-for-propdef-flex-wrap"></a>

<a id="ref-for-propdef-order②"></a>

The contents of a flex container can be laid out in any direction and in any order. This allows an author to trivially achieve effects that would previously have required complex or fragile methods, such as hacks using the [float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float) and [clear](https://www.w3.org/TR/CSS2/visuren.html#propdef-clear) properties. This functionality is exposed through the [flex-direction](#propdef-flex-direction), [flex-wrap](#propdef-flex-wrap), and [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) properties.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The reordering capabilities of flex layout intentionally affect <em>only the visual rendering</em>, leaving speech order and navigation based on the source order. This allows authors to manipulate the visual presentation while leaving the source order intact for non-CSS UAs and for linear models such as speech and sequential navigation. See [CSS Display 3 § 3.1 Reordering and Accessibility](https://www.w3.org/TR/css-display-3/#order-accessibility) and the [Flex Layout Overview](#overview) for examples that use this dichotomy to improve accessibility.

<a id="ref-for-propdef-order③"></a>

<a id="ref-for-propdef-flex-flow①"></a>

<a id="ref-for-propdef-flex-direction①"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Authors <em>must not</em> use <a href="https://www.w3.org/TR/css-flexbox-1/#propdef-order">order</a> or the <span>&#x2A;-reverse</span> values of <a href="#propdef-flex-flow">flex-flow</a>/<a href="#propdef-flex-direction">flex-direction</a> as a substitute for correct source ordering,
	as that can ruin the accessibility of the document.</strong>

<a id="ref-for-propdef-flex-direction②"></a>

### <a id="flex-direction-property"></a>5.1.  Flex Flow Direction: the [flex-direction](#propdef-flex-direction) property

| Field               | Definition                                                                                                                                              |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-flex-direction"></a>flex-direction                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①"></a>row [\|](https://www.w3.org/TR/css-values-4/#comb-one) row-reverse <a id="ref-for-comb-one②"></a>\| column <a id="ref-for-comb-one③"></a>\| column-reverse |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | row                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-flex-container①⑨"></a>[flex containers](#flex-container)                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                |

<a id="ref-for-propdef-flex-direction③"></a>

<a id="ref-for-flex-item②③"></a>

<a id="ref-for-main-axis⑥"></a>

The [flex-direction](#propdef-flex-direction) property specifies how [flex items](#flex-item) are placed in the flex container, by setting the direction of the flex container’s [main axis](#main-axis). This determines the direction in which flex items are laid out.

<a id="valdef-flex-direction-row"></a>row  
<a id="ref-for-inline-end"></a>

<a id="ref-for-inline-start"></a>

<a id="ref-for-main-end"></a>

<a id="ref-for-main-start"></a>

<a id="ref-for-writing-mode②"></a>

<a id="ref-for-inline-axis"></a>

<a id="ref-for-main-axis⑦"></a>

The flex container’s [main axis](#main-axis) has the same orientation as the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) of the current [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode). The [main-start](#main-start) and [main-end](#main-end) directions are equivalent to the [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) and [inline-end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) directions, respectively, of the current <a id="ref-for-writing-mode③"></a>writing mode.

Tests

- [flex-child-percent-basis-resize-1.html](https://wpt.fyi/results/css/css-flexbox/flex-child-percent-basis-resize-1.html) [(live test)](http://wpt.live/css/css-flexbox/flex-child-percent-basis-resize-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-child-percent-basis-resize-1.html)
- [flexbox-flex-basis-content-001a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-001a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-001a.html)
- [flexbox-flex-basis-content-001b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-001b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-001b.html)
- [flexbox-flex-direction-row.htm](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-direction-row.htm) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-direction-row.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-direction-row.htm)
- [flexbox_justifycontent-left-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-left-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-left-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-left-001.html)
- [flexbox-writing-mode-010.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-010.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-010.html)
- [flexbox-writing-mode-011.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-011.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-011.html)
- [flexbox-writing-mode-012.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-012.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-012.html)
- [flexbox-writing-mode-016.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-016.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-016.html)
- [percentage-size.html](https://wpt.fyi/results/css/css-flexbox/percentage-size.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-size.html)

<a id="valdef-flex-direction-row-reverse"></a>row-reverse  
<a id="ref-for-main-end①"></a>

<a id="ref-for-main-start①"></a>

<a id="ref-for-valdef-flex-direction-row①"></a>

Same as [row](#valdef-flex-direction-row), except the [main-start](#main-start) and [main-end](#main-end) directions are swapped.

Tests

- [flexbox_direction-row-reverse.html](https://wpt.fyi/results/css/css-flexbox/flexbox_direction-row-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_direction-row-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_direction-row-reverse.html)
- [flexbox-mbp-horiz-001-reverse.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-mbp-horiz-001-reverse.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-mbp-horiz-001-reverse.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-mbp-horiz-001-reverse.xhtml)
- [flexbox-mbp-horiz-001-rtl-reverse.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-mbp-horiz-001-rtl-reverse.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-mbp-horiz-001-rtl-reverse.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-mbp-horiz-001-rtl-reverse.xhtml)
- [multi-line-wrap-with-row-reverse.html](https://wpt.fyi/results/css/css-flexbox/flex-lines/multi-line-wrap-with-row-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/flex-lines/multi-line-wrap-with-row-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-lines/multi-line-wrap-with-row-reverse.html)
- [flexbox_justifycontent-right-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-right-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-right-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-right-001.html)
- [flexbox_justifycontent-start.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-start.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-start.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-start.html)
- [flexbox_justifycontent-start-rtl.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-start-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-start-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-start-rtl.html)

<a id="valdef-flex-direction-column"></a>column  
<a id="ref-for-block-end"></a>

<a id="ref-for-block-start"></a>

<a id="ref-for-main-end②"></a>

<a id="ref-for-main-start②"></a>

<a id="ref-for-writing-mode④"></a>

<a id="ref-for-block-axis"></a>

<a id="ref-for-main-axis⑧"></a>

The flex container’s [main axis](#main-axis) has the same orientation as the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) of the current [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode). The [main-start](#main-start) and [main-end](#main-end) directions are equivalent to the [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) and [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) directions, respectively, of the current <a id="ref-for-writing-mode⑤"></a>writing mode.

Tests

- [columns-height-set-via-top-bottom.html](https://wpt.fyi/results/css/css-flexbox/columns-height-set-via-top-bottom.html) [(live test)](http://wpt.live/css/css-flexbox/columns-height-set-via-top-bottom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/columns-height-set-via-top-bottom.html)
- [dynamic-bsize-change.html](https://wpt.fyi/results/css/css-flexbox/dynamic-bsize-change.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-bsize-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-bsize-change.html)
- [flexbox_direction-column.html](https://wpt.fyi/results/css/css-flexbox/flexbox_direction-column.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_direction-column.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_direction-column.html)
- [flexbox_direction-column-reverse.html](https://wpt.fyi/results/css/css-flexbox/flexbox_direction-column-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_direction-column-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_direction-column-reverse.html)
- [flexbox_rtl-flow.html](https://wpt.fyi/results/css/css-flexbox/flexbox_rtl-flow.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_rtl-flow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_rtl-flow.html)
- [flex-column-relayout-assert.html](https://wpt.fyi/results/css/css-flexbox/flex-column-relayout-assert.html) [(live test)](http://wpt.live/css/css-flexbox/flex-column-relayout-assert.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-column-relayout-assert.html)
- [flex-item-max-height-min-content.html](https://wpt.fyi/results/css/css-flexbox/flex-item-max-height-min-content.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-max-height-min-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-max-height-min-content.html)
- [flex-item-transferred-sizes-padding-border-sizing.html](https://wpt.fyi/results/css/css-flexbox/flex-item-transferred-sizes-padding-border-sizing.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-transferred-sizes-padding-border-sizing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-transferred-sizes-padding-border-sizing.html)
- [flex-item-transferred-sizes-padding-content-sizing.html](https://wpt.fyi/results/css/css-flexbox/flex-item-transferred-sizes-padding-content-sizing.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-transferred-sizes-padding-content-sizing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-transferred-sizes-padding-content-sizing.html)
- [flex-outer-flexbox-column-recalculate-height-on-resize-001.html](https://wpt.fyi/results/css/css-flexbox/flex-outer-flexbox-column-recalculate-height-on-resize-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-outer-flexbox-column-recalculate-height-on-resize-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-outer-flexbox-column-recalculate-height-on-resize-001.html)
- [image-nested-within-definite-column-flexbox.html](https://wpt.fyi/results/css/css-flexbox/image-nested-within-definite-column-flexbox.html) [(live test)](http://wpt.live/css/css-flexbox/image-nested-within-definite-column-flexbox.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-nested-within-definite-column-flexbox.html)
- [layout-with-inline-svg-001.html](https://wpt.fyi/results/css/css-flexbox/layout-with-inline-svg-001.html) [(live test)](http://wpt.live/css/css-flexbox/layout-with-inline-svg-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/layout-with-inline-svg-001.html)
- [nested-orthogonal-flexbox-relayout.html](https://wpt.fyi/results/css/css-flexbox/nested-orthogonal-flexbox-relayout.html) [(live test)](http://wpt.live/css/css-flexbox/nested-orthogonal-flexbox-relayout.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/nested-orthogonal-flexbox-relayout.html)
- [percentage-max-width-cross-axis.html](https://wpt.fyi/results/css/css-flexbox/percentage-max-width-cross-axis.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-max-width-cross-axis.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-max-width-cross-axis.html)
- [percentage-size.html](https://wpt.fyi/results/css/css-flexbox/percentage-size.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-size.html)

<a id="valdef-flex-direction-column-reverse"></a>column-reverse  
<a id="ref-for-main-end③"></a>

<a id="ref-for-main-start③"></a>

<a id="ref-for-valdef-flex-direction-column"></a>

Same as [column](#valdef-flex-direction-column), except the [main-start](#main-start) and [main-end](#main-end) directions are swapped.

Tests

- [column-reverse-gap.html](https://wpt.fyi/results/css/css-flexbox/column-reverse-gap.html) [(live test)](http://wpt.live/css/css-flexbox/column-reverse-gap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/column-reverse-gap.html)
- [flexbox_rtl-direction.html](https://wpt.fyi/results/css/css-flexbox/flexbox_rtl-direction.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_rtl-direction.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_rtl-direction.html)
- [multi-line-wrap-with-column-reverse.html](https://wpt.fyi/results/css/css-flexbox/flex-lines/multi-line-wrap-with-column-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/flex-lines/multi-line-wrap-with-column-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-lines/multi-line-wrap-with-column-reverse.html)

<a id="ref-for-propdef-writing-mode"></a>

<a id="ref-for-propdef-direction"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The reverse values do not reverse box ordering: like [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) and [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) [\[CSS3-WRITING-MODES\]](#biblio-css3-writing-modes), they only change the direction of flow. Painting order, speech order, and sequential navigation orders are not affected.

<a id="ref-for-propdef-justify-content①"></a>

<a id="ref-for-propdef-flex-direction④"></a>

<a id="ref-for-flex-container②⓪"></a>

<a id="ref-for-scroll-container①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Depending on the value of [justify-content](#propdef-justify-content), the reverse values of [flex-direction](#propdef-flex-direction) can alter the initial scroll position on [flex containers](#flex-container) that are also [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container). See [CSS Box Alignment 3 § 5.3 Alignment Overflow and Scroll Containers](https://www.w3.org/TR/css-align-3/#overflow-scroll-position).

Tests

- [discrete-no-interpolation.html](https://wpt.fyi/results/css/css-flexbox/animation/discrete-no-interpolation.html) [(live test)](http://wpt.live/css/css-flexbox/animation/discrete-no-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/animation/discrete-no-interpolation.html)
- [change-column-flex-width.html](https://wpt.fyi/results/css/css-flexbox/change-column-flex-width.html) [(live test)](http://wpt.live/css/css-flexbox/change-column-flex-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/change-column-flex-width.html)
- [column-flex-child-with-max-width.html](https://wpt.fyi/results/css/css-flexbox/column-flex-child-with-max-width.html) [(live test)](http://wpt.live/css/css-flexbox/column-flex-child-with-max-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/column-flex-child-with-max-width.html)
- [column-flex-child-with-overflow-scroll.html](https://wpt.fyi/results/css/css-flexbox/column-flex-child-with-overflow-scroll.html) [(live test)](http://wpt.live/css/css-flexbox/column-flex-child-with-overflow-scroll.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/column-flex-child-with-overflow-scroll.html)
- [cross-axis-scrollbar.html](https://wpt.fyi/results/css/css-flexbox/cross-axis-scrollbar.html) [(live test)](http://wpt.live/css/css-flexbox/cross-axis-scrollbar.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/cross-axis-scrollbar.html)
- [dynamic-orthogonal-flex-item.html](https://wpt.fyi/results/css/css-flexbox/dynamic-orthogonal-flex-item.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-orthogonal-flex-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-orthogonal-flex-item.html)
- [flexbox-writing-mode-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-001.html)
- [flexbox-writing-mode-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-002.html)
- [flexbox-writing-mode-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-003.html)
- [flexbox-writing-mode-004.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-004.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-004.html)
- [flexbox-writing-mode-005.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-005.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-005.html)
- [flexbox-writing-mode-006.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-006.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-006.html)
- [flexbox-writing-mode-007.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-007.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-007.html)
- [flexbox-writing-mode-008.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-008.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-008.html)
- [flexbox-writing-mode-009.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-009.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-009.html)
- [flexbox-writing-mode-010.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-010.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-010.html)
- [flexbox-writing-mode-011.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-011.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-011.html)
- [flexbox-writing-mode-012.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-012.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-012.html)
- [flexbox-writing-mode-013.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-013.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-013.html)
- [flexbox-writing-mode-014.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-014.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-014.html)
- [flexbox-writing-mode-015.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-015.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-015.html)
- [flexbox-writing-mode-016.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-016.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-016.html)
- [flexbox-writing-mode-slr.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-slr.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-slr.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-slr.html)
- [flexbox-writing-mode-slr-row-mix.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-slr-row-mix.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-slr-row-mix.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-slr-row-mix.html)
- [flexbox-writing-mode-slr-rtl.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-slr-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-slr-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-slr-rtl.html)
- [flexbox-writing-mode-srl.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-srl.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-srl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-srl.html)
- [flexbox-writing-mode-srl-row-mix.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-srl-row-mix.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-srl-row-mix.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-srl-row-mix.html)
- [flexbox-writing-mode-srl-rtl.html](https://wpt.fyi/results/css/css-flexbox/flexbox-writing-mode-srl-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-writing-mode-srl-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-writing-mode-srl-rtl.html)
- [flexbox_object.html](https://wpt.fyi/results/css/css-flexbox/flexbox_object.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_object.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_object.html)
- [flexbox_writing_mode_vertical_lays_out_contents_from_top_to_bottom.html](https://wpt.fyi/results/css/css-flexbox/flexbox_writing_mode_vertical_lays_out_contents_from_top_to_bottom.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_writing_mode_vertical_lays_out_contents_from_top_to_bottom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_writing_mode_vertical_lays_out_contents_from_top_to_bottom.html)
- [flexbox_computedstyle_flex-direction-column.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-column.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-column.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-column.html)
- [flexbox_computedstyle_flex-direction-column-reverse.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-column-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-column-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-column-reverse.html)
- [flexbox_computedstyle_flex-direction-invalid.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-invalid.html)
- [flexbox_computedstyle_flex-direction-row.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-row.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-row.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-row.html)
- [flexbox_computedstyle_flex-direction-row-reverse.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-row-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-row-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-direction-row-reverse.html)

<a id="ref-for-propdef-flex-wrap①"></a>

### <a id="flex-wrap-property"></a>5.2.  Flex Line Wrapping: the [flex-wrap](#propdef-flex-wrap) property

| Field               | Definition                                                                                                           |
|---------------------|----------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-flex-wrap"></a>flex-wrap                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one④"></a>nowrap [\|](https://www.w3.org/TR/css-values-4/#comb-one) wrap <a id="ref-for-comb-one⑤"></a>\| wrap-reverse |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | nowrap                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-flex-container②①"></a>[flex containers](#flex-container)                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                    |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                             |

<a id="ref-for-propdef-flex-wrap②"></a>

<a id="ref-for-single-line-flex-container"></a>

<a id="ref-for-multi-line-flex-container"></a>

<a id="ref-for-cross-axis④"></a>

The [flex-wrap](#propdef-flex-wrap) property controls whether the flex container is [single-line](#single-line-flex-container) or [multi-line](#multi-line-flex-container), and the direction of the [cross-axis](#cross-axis), which determines the direction new lines are stacked in.

<a id="valdef-flex-wrap-nowrap"></a>nowrap  
<a id="ref-for-single-line-flex-container①"></a>

The flex container is [single-line](#single-line-flex-container).

<a id="valdef-flex-wrap-wrap"></a>wrap  
<a id="ref-for-multi-line-flex-container①"></a>

The flex container is [multi-line](#multi-line-flex-container).

<a id="valdef-flex-wrap-wrap-reverse"></a>wrap-reverse  
<a id="ref-for-valdef-flex-wrap-wrap"></a>

Same as [wrap](#valdef-flex-wrap-wrap).

<a id="ref-for-valdef-flex-wrap-wrap-reverse"></a>

<a id="ref-for-cross-start"></a>

<a id="ref-for-inline-start①"></a>

<a id="ref-for-block-start①"></a>

<a id="ref-for-writing-mode⑥"></a>

<a id="ref-for-cross-axis⑤"></a>

<a id="ref-for-cross-end"></a>

<a id="ref-for-propdef-flex-wrap③"></a>

For the values that are not [wrap-reverse](#valdef-flex-wrap-wrap-reverse), the [cross-start](#cross-start) direction is equivalent to either the [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) or [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) direction of the current [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) (whichever is in the [cross axis](#cross-axis)) and the [cross-end](#cross-end) direction is the opposite direction of <a id="ref-for-cross-start①"></a>cross-start. When [flex-wrap](#propdef-flex-wrap) is <a id="ref-for-valdef-flex-wrap-wrap-reverse①"></a>wrap-reverse, the <a id="ref-for-cross-start②"></a>cross-start and <a id="ref-for-cross-end①"></a>cross-end directions are swapped.

<a id="ref-for-propdef-align-content①"></a>

<a id="ref-for-valdef-flex-wrap-wrap-reverse②"></a>

<a id="ref-for-propdef-flex-wrap④"></a>

<a id="ref-for-flex-container②②"></a>

<a id="ref-for-scroll-container②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Depending on the value of [align-content](#propdef-align-content), the [wrap-reverse](#valdef-flex-wrap-wrap-reverse) value of [flex-wrap](#propdef-flex-wrap) can alter the initial scroll position on [flex containers](#flex-container) that are also [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container). See [CSS Box Alignment 3 § 5.3 Alignment Overflow and Scroll Containers](https://www.w3.org/TR/css-align-3/#overflow-scroll-position).

Tests

- [discrete-no-interpolation.html](https://wpt.fyi/results/css/css-flexbox/animation/discrete-no-interpolation.html) [(live test)](http://wpt.live/css/css-flexbox/animation/discrete-no-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/animation/discrete-no-interpolation.html)
- [flexbox-flex-wrap-default.htm](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-wrap-default.htm) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-wrap-default.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-wrap-default.htm)
- [flexbox-flex-wrap-flexing-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-wrap-flexing-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-wrap-flexing-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-wrap-flexing-002.html)
- [flexbox-flex-wrap-flexing-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-wrap-flexing-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-wrap-flexing-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-wrap-flexing-003.html)
- [flexbox-flex-wrap-flexing.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-wrap-flexing.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-wrap-flexing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-wrap-flexing.html)
- [flexbox-flex-wrap-horiz-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-wrap-horiz-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-wrap-horiz-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-wrap-horiz-001.html)
- [flexbox-flex-wrap-horiz-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-wrap-horiz-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-wrap-horiz-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-wrap-horiz-002.html)
- [flexbox-flex-wrap-nowrap.htm](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-wrap-nowrap.htm) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-wrap-nowrap.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-wrap-nowrap.htm)
- [flexbox-flex-wrap-vert-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-wrap-vert-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-wrap-vert-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-wrap-vert-001.html)
- [flexbox-flex-wrap-vert-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-wrap-vert-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-wrap-vert-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-wrap-vert-002.html)
- [flexbox-flex-wrap-wrap.htm](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-wrap-wrap.htm) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-wrap-wrap.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-wrap-wrap.htm)
- [flexbox-flex-wrap-wrap-reverse.htm](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-wrap-wrap-reverse.htm) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-wrap-wrap-reverse.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-wrap-wrap-reverse.htm)
- [flexbox_rowspan.html](https://wpt.fyi/results/css/css-flexbox/flexbox_rowspan.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_rowspan.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_rowspan.html)
- [flexbox_rtl-flow.html](https://wpt.fyi/results/css/css-flexbox/flexbox_rtl-flow.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_rtl-flow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_rtl-flow.html)
- [flexbox_rtl-flow-reverse.html](https://wpt.fyi/results/css/css-flexbox/flexbox_rtl-flow-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_rtl-flow-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_rtl-flow-reverse.html)
- [flexbox_rtl-order.html](https://wpt.fyi/results/css/css-flexbox/flexbox_rtl-order.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_rtl-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_rtl-order.html)
- [flex-box-wrap.html](https://wpt.fyi/results/css/css-flexbox/flex-box-wrap.html) [(live test)](http://wpt.live/css/css-flexbox/flex-box-wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-box-wrap.html)
- [flexbox_wrap.html](https://wpt.fyi/results/css/css-flexbox/flexbox_wrap.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_wrap.html)
- [flexbox_wrap-long.html](https://wpt.fyi/results/css/css-flexbox/flexbox_wrap-long.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_wrap-long.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_wrap-long.html)
- [flexbox_wrap-reverse.html](https://wpt.fyi/results/css/css-flexbox/flexbox_wrap-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_wrap-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_wrap-reverse.html)
- [multi-line-wrap-reverse-column-reverse.html](https://wpt.fyi/results/css/css-flexbox/flex-lines/multi-line-wrap-reverse-column-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/flex-lines/multi-line-wrap-reverse-column-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-lines/multi-line-wrap-reverse-column-reverse.html)
- [multi-line-wrap-reverse-row-reverse.html](https://wpt.fyi/results/css/css-flexbox/flex-lines/multi-line-wrap-reverse-row-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/flex-lines/multi-line-wrap-reverse-row-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-lines/multi-line-wrap-reverse-row-reverse.html)
- [flexbox_computedstyle_flex-wrap-invalid.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-wrap-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-wrap-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-wrap-invalid.html)
- [flexbox_computedstyle_flex-wrap-nowrap.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-wrap-nowrap.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-wrap-nowrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-wrap-nowrap.html)
- [flexbox_computedstyle_flex-wrap-wrap.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-wrap-wrap.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-wrap-wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-wrap-wrap.html)
- [flexbox_computedstyle_flex-wrap-wrap-reverse.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-wrap-wrap-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-wrap-wrap-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-wrap-wrap-reverse.html)
- [multiline-min-preferred-width.html](https://wpt.fyi/results/css/css-flexbox/multiline-min-preferred-width.html) [(live test)](http://wpt.live/css/css-flexbox/multiline-min-preferred-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/multiline-min-preferred-width.html)
- [multiline-reverse-wrap-baseline.html](https://wpt.fyi/results/css/css-flexbox/multiline-reverse-wrap-baseline.html) [(live test)](http://wpt.live/css/css-flexbox/multiline-reverse-wrap-baseline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/multiline-reverse-wrap-baseline.html)

<a id="ref-for-propdef-flex-flow②"></a>

### <a id="flex-flow-property"></a>5.3.  Flex Direction and Wrap: the [flex-flow](#propdef-flex-flow) shorthand

| Field               | Definition                                                                                                                                                                                          |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-flex-flow"></a>flex-flow                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-flex-wrap⑤"></a><a id="ref-for-comb-any"></a><a id="ref-for-propdef-flex-direction⑤"></a>[\<'flex-direction'\>](#propdef-flex-direction) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'flex-wrap'\>](#propdef-flex-wrap) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                         |

Tests

- [box-sizing-001.html](https://wpt.fyi/results/css/css-flexbox/box-sizing-001.html) [(live test)](http://wpt.live/css/css-flexbox/box-sizing-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/box-sizing-001.html)
- [button-column-wrap-crash.html](https://wpt.fyi/results/css/css-flexbox/button-column-wrap-crash.html) [(live test)](http://wpt.live/css/css-flexbox/button-column-wrap-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/button-column-wrap-crash.html)
- [column-intrinsic-size-aspect-ratio-crash.html](https://wpt.fyi/results/css/css-flexbox/column-intrinsic-size-aspect-ratio-crash.html) [(live test)](http://wpt.live/css/css-flexbox/column-intrinsic-size-aspect-ratio-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/column-intrinsic-size-aspect-ratio-crash.html)
- [css-flexbox-row.html](https://wpt.fyi/results/css/css-flexbox/css-flexbox-row.html) [(live test)](http://wpt.live/css/css-flexbox/css-flexbox-row.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/css-flexbox-row.html)
- [css-flexbox-row-reverse.html](https://wpt.fyi/results/css/css-flexbox/css-flexbox-row-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/css-flexbox-row-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/css-flexbox-row-reverse.html)
- [css-flexbox-row-reverse-wrap.html](https://wpt.fyi/results/css/css-flexbox/css-flexbox-row-reverse-wrap.html) [(live test)](http://wpt.live/css/css-flexbox/css-flexbox-row-reverse-wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/css-flexbox-row-reverse-wrap.html)
- [css-flexbox-row-reverse-wrap-reverse.html](https://wpt.fyi/results/css/css-flexbox/css-flexbox-row-reverse-wrap-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/css-flexbox-row-reverse-wrap-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/css-flexbox-row-reverse-wrap-reverse.html)
- [css-flexbox-row-wrap.html](https://wpt.fyi/results/css/css-flexbox/css-flexbox-row-wrap.html) [(live test)](http://wpt.live/css/css-flexbox/css-flexbox-row-wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/css-flexbox-row-wrap.html)
- [css-flexbox-row-wrap-reverse.html](https://wpt.fyi/results/css/css-flexbox/css-flexbox-row-wrap-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/css-flexbox-row-wrap-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/css-flexbox-row-wrap-reverse.html)
- [css-flexbox-test1.html](https://wpt.fyi/results/css/css-flexbox/css-flexbox-test1.html) [(live test)](http://wpt.live/css/css-flexbox/css-flexbox-test1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/css-flexbox-test1.html)
- [flexbox-flex-direction-column.htm](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-direction-column.htm) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-direction-column.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-direction-column.htm)
- [flexbox-flex-direction-column-percentage-ignored.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-direction-column-percentage-ignored.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-direction-column-percentage-ignored.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-direction-column-percentage-ignored.html)
- [flexbox-flex-direction-column-reverse.htm](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-direction-column-reverse.htm) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-direction-column-reverse.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-direction-column-reverse.htm)
- [flexbox-flex-direction-default.htm](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-direction-default.htm) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-direction-default.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-direction-default.htm)
- [flexbox-flex-direction-row.htm](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-direction-row.htm) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-direction-row.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-direction-row.htm)
- [flexbox-flex-direction-row-reverse.htm](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-direction-row-reverse.htm) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-direction-row-reverse.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-direction-row-reverse.htm)
- flex-direction-column-001-visual.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-column-001-visual.html)
- [flex-direction-column.html](https://wpt.fyi/results/css/css-flexbox/flex-direction-column.html) [(live test)](http://wpt.live/css/css-flexbox/flex-direction-column.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-column.html)
- [flex-direction-column-overlap-001.html](https://wpt.fyi/results/css/css-flexbox/flex-direction-column-overlap-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-direction-column-overlap-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-column-overlap-001.html)
- flex-direction-column-reverse-001-visual.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-column-reverse-001-visual.html)
- flex-direction-column-reverse-002-visual.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-column-reverse-002-visual.html)
- [flex-direction-column-reverse.html](https://wpt.fyi/results/css/css-flexbox/flex-direction-column-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/flex-direction-column-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-column-reverse.html)
- [flex-direction.html](https://wpt.fyi/results/css/css-flexbox/flex-direction.html) [(live test)](http://wpt.live/css/css-flexbox/flex-direction.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction.html)
- [flex-direction-modify.html](https://wpt.fyi/results/css/css-flexbox/flex-direction-modify.html) [(live test)](http://wpt.live/css/css-flexbox/flex-direction-modify.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-modify.html)
- flex-direction-row-001-visual.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-row-001-visual.html)
- flex-direction-row-002-visual.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-row-002-visual.html)
- flex-direction-row-reverse-001-visual.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-row-reverse-001-visual.html)
- flex-direction-row-reverse-002-visual.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-row-reverse-002-visual.html)
- [flex-direction-row-reverse.html](https://wpt.fyi/results/css/css-flexbox/flex-direction-row-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/flex-direction-row-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-row-reverse.html)
- [flex-direction-row-vertical.html](https://wpt.fyi/results/css/css-flexbox/flex-direction-row-vertical.html) [(live test)](http://wpt.live/css/css-flexbox/flex-direction-row-vertical.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-row-vertical.html)
- [flex-direction-with-element-insert.html](https://wpt.fyi/results/css/css-flexbox/flex-direction-with-element-insert.html) [(live test)](http://wpt.live/css/css-flexbox/flex-direction-with-element-insert.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-direction-with-element-insert.html)
- [flexbox-flex-flow-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-flow-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-flow-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-flow-001.html)
- [flexbox-flex-flow-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-flow-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-flow-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-flow-002.html)
- [flexbox_flow-column-reverse-wrap.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flow-column-reverse-wrap.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flow-column-reverse-wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flow-column-reverse-wrap.html)
- [flexbox_flow-column-reverse-wrap-reverse.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flow-column-reverse-wrap-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flow-column-reverse-wrap-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flow-column-reverse-wrap-reverse.html)
- [flexbox_flow-column-wrap.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flow-column-wrap.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flow-column-wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flow-column-wrap.html)
- [flexbox_flow-column-wrap-reverse.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flow-column-wrap-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flow-column-wrap-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flow-column-wrap-reverse.html)
- [flexbox_flow-row-wrap.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flow-row-wrap.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flow-row-wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flow-row-wrap.html)
- [flexbox_flow-row-wrap-reverse.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flow-row-wrap-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flow-row-wrap-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flow-row-wrap-reverse.html)
- [flex-flow-001.html](https://wpt.fyi/results/css/css-flexbox/flex-flow-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-flow-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-flow-001.html)
- [flex-flow-002.html](https://wpt.fyi/results/css/css-flexbox/flex-flow-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-flow-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-flow-002.html)
- [flex-flow-003.html](https://wpt.fyi/results/css/css-flexbox/flex-flow-003.html) [(live test)](http://wpt.live/css/css-flexbox/flex-flow-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-flow-003.html)
- [flex-flow-004.html](https://wpt.fyi/results/css/css-flexbox/flex-flow-004.html) [(live test)](http://wpt.live/css/css-flexbox/flex-flow-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-flow-004.html)
- [flex-flow-005.html](https://wpt.fyi/results/css/css-flexbox/flex-flow-005.html) [(live test)](http://wpt.live/css/css-flexbox/flex-flow-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-flow-005.html)
- [flex-flow-006.html](https://wpt.fyi/results/css/css-flexbox/flex-flow-006.html) [(live test)](http://wpt.live/css/css-flexbox/flex-flow-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-flow-006.html)
- [flex-flow-007.html](https://wpt.fyi/results/css/css-flexbox/flex-flow-007.html) [(live test)](http://wpt.live/css/css-flexbox/flex-flow-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-flow-007.html)
- [flex-flow-008.html](https://wpt.fyi/results/css/css-flexbox/flex-flow-008.html) [(live test)](http://wpt.live/css/css-flexbox/flex-flow-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-flow-008.html)
- [flex-flow-009.html](https://wpt.fyi/results/css/css-flexbox/flex-flow-009.html) [(live test)](http://wpt.live/css/css-flexbox/flex-flow-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-flow-009.html)
- [flex-flow-010.html](https://wpt.fyi/results/css/css-flexbox/flex-flow-010.html) [(live test)](http://wpt.live/css/css-flexbox/flex-flow-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-flow-010.html)
- [flex-flow-011.html](https://wpt.fyi/results/css/css-flexbox/flex-flow-011.html) [(live test)](http://wpt.live/css/css-flexbox/flex-flow-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-flow-011.html)
- [flex-flow-012.html](https://wpt.fyi/results/css/css-flexbox/flex-flow-012.html) [(live test)](http://wpt.live/css/css-flexbox/flex-flow-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-flow-012.html)
- [flex-flow-013.html](https://wpt.fyi/results/css/css-flexbox/flex-flow-013.html) [(live test)](http://wpt.live/css/css-flexbox/flex-flow-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-flow-013.html)
- [flex-wrap-002.html](https://wpt.fyi/results/css/css-flexbox/flex-wrap-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-wrap-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-wrap-002.html)
- [flex-wrap-003.html](https://wpt.fyi/results/css/css-flexbox/flex-wrap-003.html) [(live test)](http://wpt.live/css/css-flexbox/flex-wrap-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-wrap-003.html)
- [flex-wrap-004.html](https://wpt.fyi/results/css/css-flexbox/flex-wrap-004.html) [(live test)](http://wpt.live/css/css-flexbox/flex-wrap-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-wrap-004.html)
- [flex-wrap-005.html](https://wpt.fyi/results/css/css-flexbox/flex-wrap-005.html) [(live test)](http://wpt.live/css/css-flexbox/flex-wrap-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-wrap-005.html)
- [flex-wrap-006.html](https://wpt.fyi/results/css/css-flexbox/flex-wrap-006.html) [(live test)](http://wpt.live/css/css-flexbox/flex-wrap-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-wrap-006.html)
- [flexbox_computedstyle_flex-flow-column.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column.html)
- [flexbox_computedstyle_flex-flow-column-nowrap.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-nowrap.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-nowrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-nowrap.html)
- [flexbox_computedstyle_flex-flow-column-reverse.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-reverse.html)
- [flexbox_computedstyle_flex-flow-column-reverse-nowrap.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-reverse-nowrap.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-reverse-nowrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-reverse-nowrap.html)
- [flexbox_computedstyle_flex-flow-column-reverse-wrap.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-reverse-wrap.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-reverse-wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-reverse-wrap.html)
- [flexbox_computedstyle_flex-flow-column-wrap.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-wrap.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-wrap.html)
- [flexbox_computedstyle_flex-flow-column-wrap-reverse.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-wrap-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-wrap-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-column-wrap-reverse.html)
- [flexbox_computedstyle_flex-flow-nowrap.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-nowrap.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-nowrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-nowrap.html)
- [flexbox_computedstyle_flex-flow-row.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row.html)
- [flexbox_computedstyle_flex-flow-row-nowrap.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-nowrap.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-nowrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-nowrap.html)
- [flexbox_computedstyle_flex-flow-row-reverse.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-reverse.html)
- [flexbox_computedstyle_flex-flow-row-reverse-nowrap.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-reverse-nowrap.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-reverse-nowrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-reverse-nowrap.html)
- [flexbox_computedstyle_flex-flow-row-reverse-wrap.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-reverse-wrap.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-reverse-wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-reverse-wrap.html)
- [flexbox_computedstyle_flex-flow-row-reverse-wrap-reverse.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-reverse-wrap-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-reverse-wrap-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-reverse-wrap-reverse.html)
- [flexbox_computedstyle_flex-flow-row-wrap.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-wrap.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-wrap.html)
- [flexbox_computedstyle_flex-flow-row-wrap-reverse.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-wrap-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-wrap-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-row-wrap-reverse.html)
- [flexbox_computedstyle_flex-flow-wrap.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-wrap.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-wrap.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-flow-wrap.html)
- [multiline-column-max-height.html](https://wpt.fyi/results/css/css-flexbox/multiline-column-max-height.html) [(live test)](http://wpt.live/css/css-flexbox/multiline-column-max-height.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/multiline-column-max-height.html)
- [flex-direction-computed.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-direction-computed.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-direction-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-direction-computed.html)
- [flex-direction-invalid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-direction-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-direction-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-direction-invalid.html)
- [flex-direction-valid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-direction-valid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-direction-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-direction-valid.html)
- [flex-flow-computed.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-flow-computed.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-flow-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-flow-computed.html)
- [flex-flow-invalid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-flow-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-flow-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-flow-invalid.html)
- [flex-flow-shorthand.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-flow-shorthand.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-flow-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-flow-shorthand.html)
- [flex-flow-valid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-flow-valid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-flow-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-flow-valid.html)
- [flex-wrap-computed.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-wrap-computed.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-wrap-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-wrap-computed.html)
- [flex-wrap-invalid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-wrap-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-wrap-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-wrap-invalid.html)
- [flex-wrap-valid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-wrap-valid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-wrap-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-wrap-valid.html)

<a id="ref-for-propdef-flex-flow③"></a>

<a id="ref-for-propdef-flex-direction⑥"></a>

<a id="ref-for-propdef-flex-wrap⑥"></a>

The [flex-flow](#propdef-flex-flow) property is a shorthand for setting the [flex-direction](#propdef-flex-direction) and [flex-wrap](#propdef-flex-wrap) properties, which together define the flex container’s main and cross axes.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c226b82c"></a> Some examples of valid flows in an English (left-to-right, horizontal writing mode) document:
>
>
> **Example 1**
>
> ```css
> div { flex-flow: row; }
> /* Initial value. Main-axis is inline, no wrapping.
>    (Items will either shrink to fit or overflow.) */
> ```
>
> ![](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/flex-flow1.svg)
>
> **Example 2**
>
> ```css
> div { flex-flow: column wrap; }
> /* Main-axis is block-direction (top to bottom)
>    and lines wrap in the inline direction (rightwards). */
> ```
>
> ![](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/flex-flow2.svg)
>
> **Example 3**
>
> ```css
> div { flex-flow: row-reverse wrap-reverse; }
> /* Main-axis is the opposite of inline direction
>    (right to left). New lines wrap upwards. */
> ```
>
> ![](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/flex-flow3.svg)
>

<a id="ref-for-propdef-flex-flow④"></a>

<a id="ref-for-writing-mode⑦"></a>

<a id="ref-for-valdef-flex-direction-row②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that the [flex-flow](#propdef-flex-flow) directions are [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) sensitive. In vertical Japanese, for example, a [row](#valdef-flex-direction-row) flex container lays out its contents from top to bottom, as seen in this example:
>
>
> **English**
>
> ```css
> flex-flow: row wrap;        writing-mode: horizontal-tb;
> ```
>
> ![](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/flex-flow-english.svg)
>
> **Japanese**
>
> ```css
> flex-flow: row wrap;        writing-mode: vertical-rl;
> ```
>
> ![](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/flex-flow-japanese.svg)
>

<a id="ref-for-propdef-order④"></a>

### <a id="order-property"></a>5.4. <a id="order-modified-document-order"></a><a id="order-accessibility"></a><a id="propdef-order"></a> Reordering and Accessibility: the [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) property

<a id="ref-for-flex-item②④"></a>

[Flex items](#flex-item) are, by default, displayed and laid out in the same order as they appear in the source document, which represents their logical ordering. This same order is used in rendering to non-visual media (such as [speech](https://www.w3.org/TR/css-speech/)), in the default traversal order of sequential navigation modes (such as cycling through links, see e.g. [`tabindex`](https://html.spec.whatwg.org/multipage/interaction.html#attr-tabindex) [\[HTML\]](#biblio-html)), and when content is represented in non-CSS UAs.

<a id="ref-for-propdef-order⑤"></a>

<a id="ref-for-order-modified-document-order"></a>

The [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) property can be used to change flex items’ ordering, laying them out in [order-modified document order](https://www.w3.org/TR/css-flexbox-1/#order-modified-document-order) instead, in order to make their spatial arrangement on the 2D visual canvas differ from their logical order in linear presentations such as speech and sequential navigation. See [CSS Display 3 § 3 Display Order: the order property](https://www.w3.org/TR/css-display-3/#order-property). [\[CSS-DISPLAY-3\]](#biblio-css-display-3)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since visual perception is two-dimensional and non-linear, the desired box order is not always the same logical order used by non-visual media and non-CSS UAs.

<a id="ref-for-propdef-order⑥"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Authors <em>must</em> use <a href="https://www.w3.org/TR/css-flexbox-1/#propdef-order">order</a> only for visual, not logical, reordering of content.
	Style sheets that use <span><span><a id="ref-for-propdef-order⑦"></a></span>order</span> to perform logical reordering are non-conforming.</strong>

<a id="ref-for-propdef-order⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d528284a"></a> Many web pages have a similar shape in the markup, with a header on top, a footer on bottom, and then a content area and one or two additional columns in the middle. Generally, it’s desirable that the content come first in the page’s source code, before the additional columns. However, this makes many common designs, such as simply having the additional columns on the left and the content area on the right, difficult to achieve. This has been addressed in many ways over the years, often going by the name "Holy Grail Layout" when there are two additional columns. [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) makes this trivial. For example, take the following sketch of a page’s code and desired layout:
>
> ```markup
> <!DOCTYPE html>
> <header>...</header>
> <main>
>    <article>...</article>
>    <nav>...</nav>
>    <aside>...</aside>
> </main>
> <footer>...</footer>
> ```
>
> ![In this page the header is at the top and the footer at the bottom, but the article is in the center, flanked by the nav on the right and the aside on the left.](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/flex-order-page.svg)
>
> This layout can be easily achieved with flex layout:
>
> ```css
> main { display: flex; }
> main > article { order: 2; min-width: 12em; flex:1; }
> main > nav     { order: 1; width: 200px; }
> main > aside   { order: 3; width: 200px; }
> ```
>
> <a id="ref-for-valdef-align-items-stretch"></a>
>
> As an added bonus, the columns will all be [equal-height](#valdef-align-items-stretch) by default, and the main content will be as wide as necessary to fill the screen. Additionally, this can then be combined with media queries to switch to an all-vertical layout on narrow screens:
>
> ```css
> @media all and (max-width: 600px) {
>   /* Too narrow to support three columns */
>   main { flex-flow: column; }
>   main > article, main > nav, main > aside {
>     /* Return them to document order */
>     order: 0; width: auto;
>   }
> }
> ```
>
> (Further use of multi-line flex containers to achieve even more intelligent wrapping left as an exercise for the reader.)

## <a id="flex-lines"></a>6.  Flex Lines

<a id="ref-for-flex-item②⑤"></a>

<a id="ref-for-flex-container②③"></a>

<a id="ref-for-single-line-flex-container②"></a>

<a id="ref-for-multi-line-flex-container②"></a>

<a id="ref-for-propdef-flex-wrap⑦"></a>

[Flex items](#flex-item) in a [flex container](#flex-container) are laid out and aligned within <a id="flex-line"></a>flex lines, hypothetical containers used for grouping and alignment by the layout algorithm. A flex container can be either [single-line](#single-line-flex-container) or [multi-line](#multi-line-flex-container), depending on the [flex-wrap](#propdef-flex-wrap) property:

- <a id="ref-for-propdef-flex-wrap⑧"></a>

  A <a id="single-line-flex-container"></a>single-line flex container (i.e. one with [flex-wrap: nowrap](#propdef-flex-wrap)) lays out all of its children in a single line, even if that would cause its contents to overflow.

- <a id="ref-for-propdef-flex-wrap⑨"></a>

  <a id="ref-for-flex-item②⑥"></a>

  <a id="ref-for-cross-axis⑥"></a>

  A <a id="multi-line-flex-container"></a>multi-line flex container (i.e. one with [flex-wrap: wrap](#propdef-flex-wrap) or <a id="ref-for-propdef-flex-wrap①⓪"></a>flex-wrap: wrap-reverse) breaks its [flex items](#flex-item) across multiple lines, similar to how text is broken onto a new line when it gets too wide to fit on the existing line. When additional lines are created, they are stacked in the flex container along the [cross axis](#cross-axis) according to the <a id="ref-for-propdef-flex-wrap①①"></a>flex-wrap property. Every line contains at least one <a id="ref-for-flex-item②⑦"></a>flex item, unless the flex container itself is completely empty.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b7468756"></a> This example shows four buttons that do not fit side-by-side horizontally, and therefore will wrap into multiple lines.
>
> ```css
> #flex {
>   display: flex;
>   flex-flow: row wrap;
>   width: 300px;
> }
> .item {
>   width: 80px;
> }
> ```
>
> ```markup
> <div id="flex">
>   <div class="item">1</div>
>   <div class="item">2</div>
>   <div class="item">3</div>
>   <div class="item">4</div>
> </div>
> ```
>
> <a id="ref-for-propdef-flex-flow⑤"></a>
>
> <a id="ref-for-multi-line-flex-container③"></a>
>
> <a id="ref-for-valdef-flex-wrap-wrap①"></a>
>
> Since the container is 300px wide, only three of the items fit onto a single line. They take up 240px, with 60px left over of remaining space. Because the [flex-flow](#propdef-flex-flow) property specifies a [multi-line](#multi-line-flex-container) flex container (due to the [wrap](#valdef-flex-wrap-wrap) keyword appearing in its value), the flex container will create an additional line to contain the last item.
>
> ![](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/multiline-no-flex.svg)
>
> An example rendering of the multi-line flex container.

<a id="ref-for-propdef-justify-content②"></a>

<a id="ref-for-propdef-align-self⑥"></a>

Once content is broken into lines, each line is laid out independently; flexible lengths and the [justify-content](#propdef-justify-content) and [align-self](#propdef-align-self) properties only consider the items on a single line at a time.

<a id="ref-for-multi-line-flex-container④"></a>

<a id="ref-for-flex-container②④"></a>

<a id="ref-for-cross-size⑤"></a>

<a id="ref-for-flex-item②⑧"></a>

<a id="ref-for-propdef-align-self⑦"></a>

<a id="ref-for-propdef-align-content②"></a>

<a id="ref-for-single-line-flex-container③"></a>

<a id="ref-for-main-size⑤"></a>

In a [multi-line](#multi-line-flex-container) [flex container](#flex-container) (even one with only a single line), the [cross size](#cross-size) of each line is the minimum size necessary to contain the [flex items](#flex-item) on the line (after alignment due to [align-self](#propdef-align-self)), and the lines are aligned within the flex container with the [align-content](#propdef-align-content) property. In a [single-line](#single-line-flex-container) <a id="ref-for-flex-container②⑤"></a>flex container, the <a id="ref-for-cross-size⑥"></a>cross size of the line is the <a id="ref-for-cross-size⑦"></a>cross size of the flex container, and <a id="ref-for-propdef-align-content③"></a>align-content has no effect. The [main size](#main-size) of a line is always the same as the <a id="ref-for-main-size⑥"></a>main size of the flex container’s content box.

Tests

- [align-content-001.htm](https://wpt.fyi/results/css/css-flexbox/align-content-001.htm) [(live test)](http://wpt.live/css/css-flexbox/align-content-001.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-001.htm)
- [align-content-002.htm](https://wpt.fyi/results/css/css-flexbox/align-content-002.htm) [(live test)](http://wpt.live/css/css-flexbox/align-content-002.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-002.htm)
- [align-content-003.htm](https://wpt.fyi/results/css/css-flexbox/align-content-003.htm) [(live test)](http://wpt.live/css/css-flexbox/align-content-003.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-003.htm)
- [align-content-004.htm](https://wpt.fyi/results/css/css-flexbox/align-content-004.htm) [(live test)](http://wpt.live/css/css-flexbox/align-content-004.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-004.htm)
- [align-content-005.htm](https://wpt.fyi/results/css/css-flexbox/align-content-005.htm) [(live test)](http://wpt.live/css/css-flexbox/align-content-005.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-005.htm)
- [align-content-006.htm](https://wpt.fyi/results/css/css-flexbox/align-content-006.htm) [(live test)](http://wpt.live/css/css-flexbox/align-content-006.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-006.htm)
- [align-content-007.htm](https://wpt.fyi/results/css/css-flexbox/align-content-007.htm) [(live test)](http://wpt.live/css/css-flexbox/align-content-007.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-007.htm)
- [flexbox-lines-must-be-stretched-by-default.html](https://wpt.fyi/results/css/css-flexbox/flexbox-lines-must-be-stretched-by-default.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-lines-must-be-stretched-by-default.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-lines-must-be-stretched-by-default.html)

<a id="ref-for-propdef-flex②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5601e71c"></a> Here’s the same example as the previous, except that the flex items have all been given [flex: auto](#propdef-flex). The first line has 60px of remaining space, and all of the items have the same flexibility, so each of the three items on that line will receive 20px of extra width, each ending up 100px wide. The remaining item is on a line of its own and will stretch to the entire width of the line, i.e. 300px.
>
> ![](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/multiline-flex.svg)
>
> <a id="ref-for-propdef-flex③"></a>
>
> A rendering of the same as above, but with the items all given [flex: auto](#propdef-flex).

## <a id="flexibility"></a>7.  Flexibility

<a id="ref-for-flex-item②⑨"></a>

<a id="ref-for-main-dimension④"></a>

<a id="ref-for-propdef-flex④"></a>

<a id="ref-for-flex-flex-grow-factor"></a>

<a id="ref-for-flex-flex-shrink-factor"></a>

The defining aspect of flex layout is the ability to make the [flex items](#flex-item) “flex”, altering their width/height to fill the available space in the [main dimension](#main-dimension). This is done with the [flex](#propdef-flex) property. A flex container distributes free space to its items (proportional to their [flex grow factor](#flex-flex-grow-factor)) to fill the container, or shrinks them (proportional to their [flex shrink factor](#flex-flex-shrink-factor)) to prevent overflow.

<a id="ref-for-flex-item③⓪"></a>

<a id="ref-for-propdef-flex-grow"></a>

<a id="ref-for-propdef-flex-shrink"></a>

A [flex item](#flex-item) is <a id="fully-inflexible"></a>fully inflexible if both its [flex-grow](#propdef-flex-grow) and [flex-shrink](#propdef-flex-shrink) values are zero, and <a id="flexible"></a>flexible otherwise.

Tests

- [flex-factor-less-than-one.html](https://wpt.fyi/results/css/css-flexbox/flex-factor-less-than-one.html) [(live test)](http://wpt.live/css/css-flexbox/flex-factor-less-than-one.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-factor-less-than-one.html)

<a id="ref-for-propdef-flex⑤"></a>

### <a id="flex-property"></a>7.1.  The [flex](#propdef-flex) Shorthand

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                    |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-flex"></a>flex                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-flex-basis"></a><a id="ref-for-comb-any①"></a><a id="ref-for-mult-opt"></a><a id="ref-for-propdef-flex-shrink①"></a><a id="ref-for-propdef-flex-grow①"></a><a id="ref-for-comb-one⑥"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ [\<'flex-grow'\>](#propdef-flex-grow) [\<'flex-shrink'\>](#propdef-flex-shrink)[?](https://www.w3.org/TR/css-values-4/#mult-opt) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'flex-basis'\>](#propdef-flex-basis) \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0 1 auto                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-flex-item③①"></a>[flex items](#flex-item)                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                   |

Tests

- [flexbox-dyn-resize-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-dyn-resize-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-dyn-resize-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-dyn-resize-001.html)
- [flex-item-max-width-min-content.html](https://wpt.fyi/results/css/css-flexbox/flex-item-max-width-min-content.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-max-width-min-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-max-width-min-content.html)
- [flex-item-min-height-min-content.html](https://wpt.fyi/results/css/css-flexbox/flex-item-min-height-min-content.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-min-height-min-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-min-height-min-content.html)
- [flex-item-min-width-min-content.html](https://wpt.fyi/results/css/css-flexbox/flex-item-min-width-min-content.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-min-width-min-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-min-width-min-content.html)
- [flex-shorthand-calc.html](https://wpt.fyi/results/css/css-flexbox/flex-shorthand-calc.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shorthand-calc.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shorthand-calc.html)
- [flex-computed.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-computed.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-computed.html)
- [flex-invalid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-invalid.html)
- [flex-shorthand.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-shorthand.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-shorthand.html)
- [flex-valid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-valid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-valid.html)

<a id="ref-for-propdef-flex⑥"></a>

<a id="ref-for-flex-flex-grow-factor①"></a>

<a id="ref-for-flex-flex-shrink-factor①"></a>

<a id="ref-for-flex-flex-basis"></a>

<a id="ref-for-flex-item③②"></a>

<a id="ref-for-main-size-property①"></a>

<a id="ref-for-main-size⑦"></a>

The [flex](#propdef-flex) property specifies the components of a flexible size: the <a id="flex-factor"></a>flex factors ([grow](#flex-flex-grow-factor) and [shrink](#flex-flex-shrink-factor)) and the [flex basis](#flex-flex-basis). When a box is a [flex item](#flex-item), <a id="ref-for-propdef-flex⑦"></a>flex is consulted <em>instead of</em> the [main size property](#main-size-property) to determine the [main size](#main-size) of the box. If a box is not a <a id="ref-for-flex-item③③"></a>flex item, <a id="ref-for-propdef-flex⑧"></a>flex has no effect.

<a id="ref-for-propdef-flex⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The initial values of the [flex](#propdef-flex) longhands are equivalent to [flex: 0 1 auto](#flex-initial). This differs from their defaults when omitted in the <a id="ref-for-propdef-flex①⓪"></a>flex shorthand (effectively 1 1 0px) so that the <a id="ref-for-propdef-flex①①"></a>flex shorthand can better accommodate [the most common cases](#flex-common).

<a id="ref-for-propdef-flex-grow②"></a>

<a id="valdef-flex-flex-grow"></a>[\<'flex-grow'\>](#propdef-flex-grow)

<a id="ref-for-flex-item③④"></a>

<a id="ref-for-propdef-flex-grow③"></a>

<a id="ref-for-number-value"></a>

This [\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) component sets [flex-grow](#propdef-flex-grow) [longhand](#flex-components) and specifies the <a id="flex-flex-grow-factor"></a>flex grow factor, which determines how much the [flex item](#flex-item) will grow relative to the rest of the <a id="ref-for-flex-item③⑤"></a>flex items in the flex container when positive free space is distributed. When omitted, it is set to 1.

Tests

- [flex-grow-interpolation.html](https://wpt.fyi/results/css/css-flexbox/animation/flex-grow-interpolation.html) [(live test)](http://wpt.live/css/css-flexbox/animation/flex-grow-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/animation/flex-grow-interpolation.html)
- [flex-001.htm](https://wpt.fyi/results/css/css-flexbox/flex-001.htm) [(live test)](http://wpt.live/css/css-flexbox/flex-001.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-001.htm)
- [flex-003.htm](https://wpt.fyi/results/css/css-flexbox/flex-003.htm) [(live test)](http://wpt.live/css/css-flexbox/flex-003.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-003.htm)
- [flex-grow-001.xht](https://wpt.fyi/results/css/css-flexbox/flex-grow-001.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-001.xht)
- [flex-grow-002.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-002.html)
- [flex-grow-003.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-003.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-003.html)
- [flex-grow-004.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-004.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-004.html)
- [flex-grow-005.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-005.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-005.html)
- [flex-grow-006.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-006.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-006.html)
- [flex-grow-007.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-007.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-007.html)
- [flex-grow-008.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-008.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-008.html)
- [flexbox_flex-natural.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-natural.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-natural.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-natural.html)
- [flexbox_flex-natural-mixed-basis-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-natural-mixed-basis-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-natural-mixed-basis-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-natural-mixed-basis-auto.html)
- [flexbox_flex-natural-mixed-basis.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-natural-mixed-basis.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-natural-mixed-basis.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-natural-mixed-basis.html)
- [flexbox_flex-natural-variable-auto-basis.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-natural-variable-auto-basis.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-natural-variable-auto-basis.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-natural-variable-auto-basis.html)
- [flexbox_flex-natural-variable-zero-basis.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-natural-variable-zero-basis.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-natural-variable-zero-basis.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-natural-variable-zero-basis.html)
- [flexbox_flex-none.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-none.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-none.html)
- [flexbox_flex-none-wrappable-content.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-none-wrappable-content.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-none-wrappable-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-none-wrappable-content.html)
- [flex-factor-less-than-one.html](https://wpt.fyi/results/css/css-flexbox/flex-factor-less-than-one.html) [(live test)](http://wpt.live/css/css-flexbox/flex-factor-less-than-one.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-factor-less-than-one.html)
- [flexbox_computedstyle_flex-shorthand-0-auto.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-0-auto.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-0-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-0-auto.html)
- [flexbox_computedstyle_flex-shorthand-auto.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-auto.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-auto.html)
- [flexbox_computedstyle_flex-shorthand.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand.html)
- [flexbox_computedstyle_flex-shorthand-initial.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-initial.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-initial.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-initial.html)
- [flexbox_computedstyle_flex-shorthand-invalid.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-invalid.html)
- [flexbox_computedstyle_flex-shorthand-none.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-none.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-none.html)
- [flexbox_computedstyle_flex-shorthand-number.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-number.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-number.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shorthand-number.html)
- [flexbox_computedstyle_flex-grow-0.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-0.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-0.html)
- [flexbox_computedstyle_flex-grow-invalid.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-invalid.html)
- [flexbox_computedstyle_flex-grow-number.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-number.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-number.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-number.html)
- flexbox_interactive_flex-grow-transitions.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_flex-grow-transitions.html)
- [flex-grow-computed.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-grow-computed.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-grow-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-grow-computed.html)
- [flex-grow-invalid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-grow-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-grow-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-grow-invalid.html)
- [flex-grow-valid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-grow-valid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-grow-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-grow-valid.html)
- [table-item-flex-percentage-min-width.html](https://wpt.fyi/results/css/css-flexbox/table-item-flex-percentage-min-width.html) [(live test)](http://wpt.live/css/css-flexbox/table-item-flex-percentage-min-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-item-flex-percentage-min-width.html)
- [table-item-flex-percentage-width.html](https://wpt.fyi/results/css/css-flexbox/table-item-flex-percentage-width.html) [(live test)](http://wpt.live/css/css-flexbox/table-item-flex-percentage-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-item-flex-percentage-width.html)

> <strong data-conversion-semantic="note">Note</strong>
>
> Flex values between 0 and 1 have a somewhat special behavior: when the sum of the flex values on the line is less than 1, they will take up less than 100% of the free space.
>
> <a id="ref-for-propdef-flex-grow④"></a>
>
> An item’s [flex-grow](#propdef-flex-grow) value is effectively a request for some proportion of the free space, with 1 meaning “100% of the free space”; then if the items on the line are requesting more than 100% in total, the requests are rebalanced to keep the same ratio but use up exactly 100% of it. However, if the items request <em>less</em> than the full amount (such as three items that are each <a id="ref-for-propdef-flex-grow⑤"></a>flex-grow: .25) then they’ll each get exactly what they request (25% of the free space to each, with the final 25% left unfilled). See [§ 9.7 Resolving Flexible Lengths](#resolve-flexible-lengths) for the exact details of how free space is distributed.
>
> <a id="ref-for-propdef-flex-grow⑥"></a>
>
> This pattern is required for continuous behavior as [flex-grow](#propdef-flex-grow) approaches zero (which means the item wants <em>none</em> of the free space). Without this, a <a id="ref-for-propdef-flex-grow⑦"></a>flex-grow: 1 item would take all of the free space; but so would a <a id="ref-for-propdef-flex-grow⑧"></a>flex-grow: 0.1 item, and a <a id="ref-for-propdef-flex-grow⑨"></a>flex-grow: 0.01 item, etc., until finally the value is small enough to underflow to zero and the item suddenly takes up none of the free space. With this behavior, the item instead gradually takes less of the free space as <a id="ref-for-propdef-flex-grow①⓪"></a>flex-grow shrinks below 1, smoothly transitioning to taking none of the free space at zero.
>
> Unless this “partial fill” behavior is <em>specifically</em> what’s desired, authors should stick to values ≥ 1; for example, using 1 and 2 is usually better than using .33 and .67, as they’re more likely to behave as intended if items are added, removed, or line-wrapped.

<a id="ref-for-propdef-flex-shrink②"></a>

<a id="valdef-flex-flex-shrink"></a>[\<'flex-shrink'\>](#propdef-flex-shrink)

<a id="ref-for-flex-item③⑥"></a>

<a id="ref-for-propdef-flex-shrink③"></a>

<a id="ref-for-number-value①"></a>

This [\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) component sets [flex-shrink](#propdef-flex-shrink) [longhand](#flex-components) and specifies the <a id="flex-flex-shrink-factor"></a>flex shrink factor, which determines how much the [flex item](#flex-item) will shrink relative to the rest of the <a id="ref-for-flex-item③⑦"></a>flex items in the flex container when negative free space is distributed. When omitted, it is set to 1.

Tests

- [flex-shrink-interpolation.html](https://wpt.fyi/results/css/css-flexbox/animation/flex-shrink-interpolation.html) [(live test)](http://wpt.live/css/css-flexbox/animation/flex-shrink-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/animation/flex-shrink-interpolation.html)
- [flex-002.htm](https://wpt.fyi/results/css/css-flexbox/flex-002.htm) [(live test)](http://wpt.live/css/css-flexbox/flex-002.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-002.htm)
- [flex-004.htm](https://wpt.fyi/results/css/css-flexbox/flex-004.htm) [(live test)](http://wpt.live/css/css-flexbox/flex-004.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-004.htm)
- [flex-factor-less-than-one.html](https://wpt.fyi/results/css/css-flexbox/flex-factor-less-than-one.html) [(live test)](http://wpt.live/css/css-flexbox/flex-factor-less-than-one.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-factor-less-than-one.html)
- [flex-shrink-001.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-001.html)
- [flex-shrink-002.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-002.html)
- [flex-shrink-003.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-003.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-003.html)
- [flex-shrink-004.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-004.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-004.html)
- [flex-shrink-005.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-005.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-005.html)
- [flex-shrink-006.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-006.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-006.html)
- [flex-shrink-007.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-007.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-007.html)
- [flex-shrink-008.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-008.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-008.html)
- [flex-shrink-large-value-crash.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-large-value-crash.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-large-value-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-large-value-crash.html)
- [flexbox_computedstyle_flex-shrink-0.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-0.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-0.html)
- [flexbox_computedstyle_flex-shrink-invalid.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-invalid.html)
- [flexbox_computedstyle_flex-shrink-number.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-number.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-number.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-number.html)
- flexbox_interactive_flex-shrink-transitions.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_flex-shrink-transitions.html)
- flexbox_interactive_flex-shrink-transitions-invalid.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_flex-shrink-transitions-invalid.html)
- [flex-shrink-computed.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-shrink-computed.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-shrink-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-shrink-computed.html)
- [flex-shrink-invalid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-shrink-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-shrink-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-shrink-invalid.html)
- [flex-shrink-valid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-shrink-valid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-shrink-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-shrink-valid.html)
- [table-item-flex-percentage-min-width.html](https://wpt.fyi/results/css/css-flexbox/table-item-flex-percentage-min-width.html) [(live test)](http://wpt.live/css/css-flexbox/table-item-flex-percentage-min-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-item-flex-percentage-min-width.html)
- [table-item-flex-percentage-width.html](https://wpt.fyi/results/css/css-flexbox/table-item-flex-percentage-width.html) [(live test)](http://wpt.live/css/css-flexbox/table-item-flex-percentage-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-item-flex-percentage-width.html)

<a id="ref-for-flex-flex-shrink-factor②"></a>

<a id="ref-for-flex-base-size"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [flex shrink factor](#flex-flex-shrink-factor) is multiplied by the [flex base size](#flex-base-size) when distributing negative space. This distributes negative space in proportion to how much the item is able to shrink, so that e.g. a small item won’t shrink to zero before a larger item has been noticeably reduced.

Tests

- [flex-item-max-width-min-content-002.html](https://wpt.fyi/results/css/css-flexbox/flex-item-max-width-min-content-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-max-width-min-content-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-max-width-min-content-002.html)

<a id="ref-for-propdef-flex-basis①"></a>

<a id="valdef-flex-flex-basis"></a>[\<'flex-basis'\>](#propdef-flex-basis)

<a id="ref-for-flex-item③⑧"></a>

<a id="ref-for-main-size⑧"></a>

<a id="ref-for-propdef-flex-basis②"></a>

This component sets the [flex-basis](#propdef-flex-basis) [longhand](#flex-components), which specifies the <a id="flex-flex-basis"></a>flex basis: the initial [main size](#main-size) of the [flex item](#flex-item), before free space is distributed according to the flex factors.

Tests

- [flex-basis-composition.html](https://wpt.fyi/results/css/css-flexbox/animation/flex-basis-composition.html) [(live test)](http://wpt.live/css/css-flexbox/animation/flex-basis-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/animation/flex-basis-composition.html)
- [flex-basis-content-crash.html](https://wpt.fyi/results/css/css-flexbox/animation/flex-basis-content-crash.html) [(live test)](http://wpt.live/css/css-flexbox/animation/flex-basis-content-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/animation/flex-basis-content-crash.html)
- [flex-basis-interpolation.html](https://wpt.fyi/results/css/css-flexbox/animation/flex-basis-interpolation.html) [(live test)](http://wpt.live/css/css-flexbox/animation/flex-basis-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/animation/flex-basis-interpolation.html)
- [dynamic-isize-change-001.html](https://wpt.fyi/results/css/css-flexbox/dynamic-isize-change-001.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-isize-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-isize-change-001.html)
- [dynamic-isize-change-002.html](https://wpt.fyi/results/css/css-flexbox/dynamic-isize-change-002.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-isize-change-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-isize-change-002.html)
- [dynamic-isize-change-003.html](https://wpt.fyi/results/css/css-flexbox/dynamic-isize-change-003.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-isize-change-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-isize-change-003.html)
- [dynamic-isize-change-004.html](https://wpt.fyi/results/css/css-flexbox/dynamic-isize-change-004.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-isize-change-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-isize-change-004.html)
- [flex-basis-001.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-001.html)
- [flex-basis-002.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-002.html)
- [flex-basis-003.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-003.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-003.html)
- [flex-basis-004.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-004.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-004.html)
- [flex-basis-005.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-005.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-005.html)
- [flex-basis-006.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-006.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-006.html)
- [flex-basis-007.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-007.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-007.html)
- [flex-basis-008.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-008.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-008.html)
- [flex-basis-009.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-009.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-009.html)
- [flex-basis-010.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-010.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-010.html)
- [flex-basis-011.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-011.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-011.html)
- [flex-basis-012.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-012.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-012.html)
- [flex-basis-013.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-013.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-013.html)
- [flex-basis-intrinsics-001.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-intrinsics-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-intrinsics-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-intrinsics-001.html)
- [flex-basis-item-margins-001.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-item-margins-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-item-margins-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-item-margins-001.html)
- [flexbox-flex-basis-content-001a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-001a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-001a.html)
- [flexbox-flex-basis-content-001b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-001b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-001b.html)
- [flexbox-flex-basis-content-002a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-002a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-002a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-002a.html)
- [flexbox-flex-basis-content-002b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-002b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-002b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-002b.html)
- [flexbox-flex-basis-content-003a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-003a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-003a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-003a.html)
- [flexbox-flex-basis-content-003b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-003b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-003b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-003b.html)
- [flexbox-flex-basis-content-004a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-004a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-004a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-004a.html)
- [flexbox-flex-basis-content-004b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-004b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-004b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-004b.html)
- [flexbox_flex-basis.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-basis.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-basis.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-basis.html)
- [flexbox_flex-basis-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-basis-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-basis-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-basis-shrink.html)
- [flex-one-sets-flex-basis-to-zero-px.html](https://wpt.fyi/results/css/css-flexbox/flex-one-sets-flex-basis-to-zero-px.html) [(live test)](http://wpt.live/css/css-flexbox/flex-one-sets-flex-basis-to-zero-px.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-one-sets-flex-basis-to-zero-px.html)
- [flex-shorthand-flex-basis-middle.html](https://wpt.fyi/results/css/css-flexbox/flex-shorthand-flex-basis-middle.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shorthand-flex-basis-middle.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shorthand-flex-basis-middle.html)
- [flexbox_computedstyle_flex-basis-0.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-0.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-0.html)
- [flexbox_computedstyle_flex-basis-0percent.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-0percent.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-0percent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-0percent.html)
- [flexbox_computedstyle_flex-basis-auto.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-auto.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-auto.html)
- [flexbox_computedstyle_flex-basis-percent.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-percent.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-percent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-percent.html)
- flexbox_interactive_flex-basis-transitions.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_flex-basis-transitions.html)
- [flex-basis-computed.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-basis-computed.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-basis-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-basis-computed.html)
- [flex-basis-invalid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-basis-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-basis-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-basis-invalid.html)
- [flex-basis-valid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-basis-valid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-basis-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-basis-valid.html)
- [table-as-item-percent-width-cell-001.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-percent-width-cell-001.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-percent-width-cell-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-percent-width-cell-001.html)

<a id="ref-for-propdef-flex-basis③"></a>

<a id="ref-for-propdef-width④"></a>

<a id="ref-for-propdef-height③"></a>

<a id="ref-for-valdef-flex-basis-auto"></a>

<a id="ref-for-valdef-flex-basis-content"></a>

[\<'flex-basis'\>](#propdef-flex-basis) accepts the same values as the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) properties (except that [auto](#valdef-flex-basis-auto) is treated differently) plus the [content](#valdef-flex-basis-content) keyword:

<a id="valdef-flex-basis-auto"></a>auto

<a id="ref-for-valdef-flex-basis-content①"></a>

<a id="ref-for-valdef-align-items-auto"></a>

<a id="ref-for-propdef-flex-basis④"></a>

<a id="ref-for-main-size-property②"></a>

<a id="ref-for-valdef-flex-basis-auto①"></a>

<a id="ref-for-flex-item③⑨"></a>

When specified on a [flex item](#flex-item), the [auto](#valdef-flex-basis-auto) keyword retrieves the value of the [main size property](#main-size-property) as the used [flex-basis](#propdef-flex-basis). If that value is itself [auto](#valdef-align-items-auto), then the used value is [content](#valdef-flex-basis-content).

<a id="valdef-flex-basis-content"></a>content

<a id="ref-for-preferred-aspect-ratio②"></a>

<a id="ref-for-max-content"></a>

<a id="ref-for-flex-item④⓪"></a>

<a id="ref-for-automatic-size①"></a>

Indicates an [automatic size](https://www.w3.org/TR/css-sizing-3/#automatic-size) based on the [flex item](#flex-item)’s content. (This is typically equivalent to the [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content), but with adjustments to handle [preferred aspect ratios](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio), intrinsic sizing constraints, and orthogonal flows; see [details](#algo-main-item) in [§ 9 Flex Layout Algorithm](#layout-algorithm).)

<a id="ref-for-propdef-width⑤"></a>

<a id="ref-for-propdef-height④"></a>

<a id="ref-for-valdef-width-auto①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value was not present in the initial release of Flexible Box Layout, and thus some older implementations will not support it. The equivalent effect can be achieved by using auto together with a main size ([width](https://www.w3.org/TR/css-sizing-3/#propdef-width) or [height](https://www.w3.org/TR/css-sizing-3/#propdef-height)) of [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto).

<a id="ref-for-propdef-width⑥"></a>

[\<'width'\>](https://www.w3.org/TR/css-sizing-3/#propdef-width)

<a id="ref-for-propdef-height⑤"></a>

<a id="ref-for-propdef-width⑦"></a>

<a id="ref-for-propdef-flex-basis⑤"></a>

For all other values, [flex-basis](#propdef-flex-basis) is resolved the same way as for [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height).

<a id="ref-for-propdef-flex①②"></a>

When omitted from the [flex](#propdef-flex) shorthand, its specified value is 0.

<a id="valdef-flex-none"></a>none

<a id="ref-for-valdef-flex-none"></a>

The keyword [none](#valdef-flex-none) expands to 0 0 auto.

![](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/rel-vs-abs-flex.svg)

A diagram showing the difference between "absolute" flex (starting from a basis of zero) and "relative" flex (starting from a basis of the item’s content size). The three items have flex factors of 1, 1, and 2, respectively: notice that the item with a flex factor of 2 grows twice as fast as the others.

<a id="ref-for-propdef-flex-basis⑥"></a>

A unitless zero that is not already preceded by two flex factors must be interpreted as a flex factor. To avoid misinterpretation or invalid declarations, authors must specify a zero [\<'flex-basis'\>](#propdef-flex-basis) component with a unit or precede it by two flex factors.

<a id="ref-for-propdef-flex①③"></a>

#### <a id="flex-common"></a>7.1.1.  Basic Values of [flex](#propdef-flex)

<em>This section is informative.</em>

<a id="ref-for-propdef-flex①④"></a>

The list below summarizes the effects of the four [flex](#propdef-flex) values that represent most commonly-desired effects:

<a id="ref-for-propdef-flex①⑤"></a>

<a id="flex-initial"></a>[flex: initial](#propdef-flex)

<a id="ref-for-main-axis⑨"></a>

<a id="ref-for-valdef-width-auto②"></a>

<a id="ref-for-main-size-property③"></a>

<a id="ref-for-propdef-height⑥"></a>

<a id="ref-for-propdef-width⑧"></a>

<a id="ref-for-propdef-flex①⑥"></a>

Equivalent to [flex: 0 1 auto](#propdef-flex). (This is the initial value.) Sizes the item based on the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)/[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) properties. (If the item’s [main size property](#main-size-property) computes to [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), this will size the flex item based on its contents.) Makes the flex item inflexible when there is positive free space, but allows it to shrink to its minimum size when there is insufficient space. The [alignment abilities](#alignment) or [auto margins](#auto-margins) can be used to align flex items along the [main axis](#main-axis).

<a id="ref-for-propdef-flex①⑦"></a>

[flex: auto](#propdef-flex)

<a id="ref-for-main-axis①⓪"></a>

<a id="ref-for-propdef-height⑦"></a>

<a id="ref-for-propdef-width⑨"></a>

<a id="ref-for-propdef-flex①⑧"></a>

Equivalent to [flex: 1 1 auto](#propdef-flex). Sizes the item based on the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)/[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) properties, but makes them fully flexible, so that they absorb any free space along the [main axis](#main-axis). If all items are either <a id="ref-for-propdef-flex①⑨"></a>flex: auto, <a id="ref-for-propdef-flex②⓪"></a>flex: initial, or <a id="ref-for-propdef-flex②①"></a>flex: none, any positive free space after the items have been sized will be distributed evenly to the items with <a id="ref-for-propdef-flex②②"></a>flex: auto.

<a id="ref-for-propdef-flex②③"></a>

[flex: none](#propdef-flex)

<a id="ref-for-valdef-all-initial"></a>

<a id="ref-for-fully-inflexible"></a>

<a id="ref-for-propdef-height⑧"></a>

<a id="ref-for-propdef-width①⓪"></a>

<a id="ref-for-propdef-flex②④"></a>

Equivalent to [flex: 0 0 auto](#propdef-flex). This value sizes the item according to the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)/[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) properties, but makes the flex item [fully inflexible](#fully-inflexible). This is similar to [initial](https://www.w3.org/TR/css-cascade-5/#valdef-all-initial), except that flex items are not allowed to shrink, even in overflow situations.

<a id="ref-for-propdef-flex②⑤"></a>

[flex: \<number \[1,∞\]\>](#propdef-flex)

<a id="ref-for-flex-flex-basis①"></a>

<a id="ref-for-propdef-flex②⑥"></a>

Equivalent to [flex: \<number \[1,∞\]\> 1 0](#propdef-flex). Makes the flex item flexible and sets the [flex basis](#flex-flex-basis) to zero, resulting in an item that receives the specified proportion of the free space in the flex container. If all items in the flex container use this pattern, their sizes will be proportional to the specified flex factor.

Tests

- [flexbox_flex-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-auto.html)
- [flexbox_flex-initial-2.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-initial-2.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-initial-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-initial-2.html)
- [flexbox_flex-initial.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-initial.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-initial.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-initial.html)
- [radiobutton-min-size.html](https://wpt.fyi/results/css/css-flexbox/radiobutton-min-size.html) [(live test)](http://wpt.live/css/css-flexbox/radiobutton-min-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/radiobutton-min-size.html)

<a id="ref-for-propdef-min-width④"></a>

<a id="ref-for-propdef-min-height③"></a>

By default, flex items won’t shrink below their minimum content size (the length of the longest word or fixed-size element). To change this, set the [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width) or [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) property. (See [§ 4.5 Automatic Minimum Size of Flex Items](#min-size-auto).)

### <a id="flex-components"></a>7.2.  Components of Flexibility

Individual components of flexibility can be controlled by independent longhand properties.

<a id="ref-for-propdef-flex②⑦"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Authors are encouraged to control flexibility using the <a href="#propdef-flex">flex</a> shorthand
	rather than with its longhand properties directly,
	as the shorthand correctly resets any unspecified components
	to accommodate <a href="#flex-common">common uses</a>.</strong>

<a id="ref-for-propdef-flex-grow①①"></a>

#### <a id="flex-grow-property"></a>7.2.1.  The [flex-grow](#propdef-flex-grow) property

| Field               | Definition                                                                                |
|---------------------|-------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-flex-grow"></a>flex-grow                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-number-value②"></a>[\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-flex-item④①"></a>[flex items](#flex-item)                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified number                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                               |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                    |

Tests

- [flex-grow-interpolation.html](https://wpt.fyi/results/css/css-flexbox/animation/flex-grow-interpolation.html) [(live test)](http://wpt.live/css/css-flexbox/animation/flex-grow-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/animation/flex-grow-interpolation.html)
- [flex-001.htm](https://wpt.fyi/results/css/css-flexbox/flex-001.htm) [(live test)](http://wpt.live/css/css-flexbox/flex-001.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-001.htm)
- [flex-003.htm](https://wpt.fyi/results/css/css-flexbox/flex-003.htm) [(live test)](http://wpt.live/css/css-flexbox/flex-003.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-003.htm)
- [flex-grow-001.xht](https://wpt.fyi/results/css/css-flexbox/flex-grow-001.xht) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-001.xht)
- [flex-grow-002.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-002.html)
- [flex-grow-003.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-003.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-003.html)
- [flex-grow-004.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-004.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-004.html)
- [flex-grow-005.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-005.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-005.html)
- [flex-grow-006.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-006.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-006.html)
- [flex-grow-007.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-007.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-007.html)
- [flex-grow-008.html](https://wpt.fyi/results/css/css-flexbox/flex-grow-008.html) [(live test)](http://wpt.live/css/css-flexbox/flex-grow-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-grow-008.html)
- [flexbox_computedstyle_flex-grow-0.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-0.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-0.html)
- [flexbox_computedstyle_flex-grow-invalid.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-invalid.html)
- [flexbox_computedstyle_flex-grow-number.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-number.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-number.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-grow-number.html)
- flexbox_interactive_flex-grow-transitions.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_flex-grow-transitions.html)
- [flex-grow-computed.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-grow-computed.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-grow-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-grow-computed.html)
- [flex-grow-invalid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-grow-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-grow-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-grow-invalid.html)
- [flex-grow-valid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-grow-valid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-grow-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-grow-valid.html)

<a id="ref-for-propdef-flex②⑧"></a>

<a id="ref-for-propdef-flex-grow①②"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Authors are encouraged to control flexibility using the <a href="#propdef-flex">flex</a> shorthand
	rather than with <a href="#propdef-flex-grow">flex-grow</a> directly,
	as the shorthand correctly resets any unspecified components
	to accommodate <a href="#flex-common">common uses</a>.</strong>

<a id="ref-for-propdef-flex-grow①③"></a>

<a id="ref-for-flex-flex-grow-factor②"></a>

<a id="ref-for-number-value③"></a>

The [flex-grow](#propdef-flex-grow) property sets the [flex grow factor](#flex-flex-grow-factor) to the provided <a id="valdef-flex-grow-number"></a>[\<number\>](https://www.w3.org/TR/css-values-4/#number-value). Negative values are not allowed.

<a id="ref-for-propdef-flex-shrink④"></a>

#### <a id="flex-shrink-property"></a>7.2.2.  The [flex-shrink](#propdef-flex-shrink) property

| Field               | Definition                                                                                |
|---------------------|-------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-flex-shrink"></a>flex-shrink                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-number-value④"></a>[\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 1                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-flex-item④②"></a>[flex items](#flex-item)                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified value                                                                           |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                               |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | number                                                                                    |

Tests

- [flex-shrink-interpolation.html](https://wpt.fyi/results/css/css-flexbox/animation/flex-shrink-interpolation.html) [(live test)](http://wpt.live/css/css-flexbox/animation/flex-shrink-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/animation/flex-shrink-interpolation.html)
- [flex-002.htm](https://wpt.fyi/results/css/css-flexbox/flex-002.htm) [(live test)](http://wpt.live/css/css-flexbox/flex-002.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-002.htm)
- [flex-004.htm](https://wpt.fyi/results/css/css-flexbox/flex-004.htm) [(live test)](http://wpt.live/css/css-flexbox/flex-004.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-004.htm)
- [flex-shrink-001.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-001.html)
- [flex-shrink-002.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-002.html)
- [flex-shrink-003.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-003.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-003.html)
- [flex-shrink-004.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-004.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-004.html)
- [flex-shrink-005.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-005.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-005.html)
- [flex-shrink-006.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-006.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-006.html)
- [flex-shrink-007.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-007.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-007.html)
- [flex-shrink-008.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-008.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-008.html)
- [flex-shrink-large-value-crash.html](https://wpt.fyi/results/css/css-flexbox/flex-shrink-large-value-crash.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shrink-large-value-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shrink-large-value-crash.html)
- [flexbox_computedstyle_flex-shrink-0.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-0.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-0.html)
- [flexbox_computedstyle_flex-shrink-invalid.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-invalid.html)
- [flexbox_computedstyle_flex-shrink-number.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-number.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-number.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-shrink-number.html)
- flexbox_interactive_flex-shrink-transitions.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_flex-shrink-transitions.html)
- flexbox_interactive_flex-shrink-transitions-invalid.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_flex-shrink-transitions-invalid.html)
- [flex-shrink-computed.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-shrink-computed.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-shrink-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-shrink-computed.html)
- [flex-shrink-invalid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-shrink-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-shrink-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-shrink-invalid.html)
- [flex-shrink-valid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-shrink-valid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-shrink-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-shrink-valid.html)

<a id="ref-for-propdef-flex②⑨"></a>

<a id="ref-for-propdef-flex-shrink⑤"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Authors are encouraged to control flexibility using the <a href="#propdef-flex">flex</a> shorthand
	rather than with <a href="#propdef-flex-shrink">flex-shrink</a> directly,
	as the shorthand correctly resets any unspecified components
	to accommodate <a href="#flex-common">common uses</a>.</strong>

<a id="ref-for-propdef-flex-shrink⑥"></a>

<a id="ref-for-flex-flex-shrink-factor③"></a>

<a id="ref-for-number-value⑤"></a>

The [flex-shrink](#propdef-flex-shrink) property sets the [flex shrink factor](#flex-flex-shrink-factor) to the provided <a id="valdef-flex-shrink-number"></a>[\<number\>](https://www.w3.org/TR/css-values-4/#number-value). Negative values are not allowed.

<a id="ref-for-propdef-flex-basis⑦"></a>

#### <a id="flex-basis-property"></a>7.2.3.  The [flex-basis](#propdef-flex-basis) property

| Field               | Definition                                                                                                                                                        |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-flex-basis"></a>flex-basis                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-width①①"></a><a id="ref-for-comb-one⑦"></a>content [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<'width'\>](https://www.w3.org/TR/css-sizing-3/#propdef-width) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-flex-item④③"></a>[flex items](#flex-item)                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-main-size⑨"></a><a id="ref-for-flex-container②⑥"></a>relative to the [flex container’s](#flex-container) inner [main size](#main-size)                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage"></a>specified keyword or a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                            |

Tests

- [flex-basis-composition.html](https://wpt.fyi/results/css/css-flexbox/animation/flex-basis-composition.html) [(live test)](http://wpt.live/css/css-flexbox/animation/flex-basis-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/animation/flex-basis-composition.html)
- [flex-basis-content-crash.html](https://wpt.fyi/results/css/css-flexbox/animation/flex-basis-content-crash.html) [(live test)](http://wpt.live/css/css-flexbox/animation/flex-basis-content-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/animation/flex-basis-content-crash.html)
- [flex-basis-interpolation.html](https://wpt.fyi/results/css/css-flexbox/animation/flex-basis-interpolation.html) [(live test)](http://wpt.live/css/css-flexbox/animation/flex-basis-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/animation/flex-basis-interpolation.html)
- [dynamic-isize-change-001.html](https://wpt.fyi/results/css/css-flexbox/dynamic-isize-change-001.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-isize-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-isize-change-001.html)
- [dynamic-isize-change-002.html](https://wpt.fyi/results/css/css-flexbox/dynamic-isize-change-002.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-isize-change-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-isize-change-002.html)
- [dynamic-isize-change-003.html](https://wpt.fyi/results/css/css-flexbox/dynamic-isize-change-003.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-isize-change-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-isize-change-003.html)
- [dynamic-isize-change-004.html](https://wpt.fyi/results/css/css-flexbox/dynamic-isize-change-004.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-isize-change-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-isize-change-004.html)
- [flex-basis-001.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-001.html)
- [flex-basis-002.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-002.html)
- [flex-basis-003.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-003.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-003.html)
- [flex-basis-004.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-004.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-004.html)
- [flex-basis-005.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-005.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-005.html)
- [flex-basis-006.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-006.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-006.html)
- [flex-basis-007.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-007.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-007.html)
- [flex-basis-008.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-008.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-008.html)
- [flex-basis-009.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-009.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-009.html)
- [flex-basis-010.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-010.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-010.html)
- [flex-basis-011.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-011.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-011.html)
- [flex-basis-012.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-012.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-012.html)
- [flex-basis-013.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-013.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-013.html)
- [flex-basis-intrinsics-001.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-intrinsics-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-intrinsics-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-intrinsics-001.html)
- [flex-basis-item-margins-001.html](https://wpt.fyi/results/css/css-flexbox/flex-basis-item-margins-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-basis-item-margins-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-basis-item-margins-001.html)
- [flexbox-flex-basis-content-001a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-001a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-001a.html)
- [flexbox-flex-basis-content-001b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-001b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-001b.html)
- [flexbox-flex-basis-content-002a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-002a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-002a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-002a.html)
- [flexbox-flex-basis-content-002b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-002b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-002b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-002b.html)
- [flexbox-flex-basis-content-003a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-003a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-003a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-003a.html)
- [flexbox-flex-basis-content-003b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-003b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-003b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-003b.html)
- [flexbox-flex-basis-content-004a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-004a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-004a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-004a.html)
- [flexbox-flex-basis-content-004b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-flex-basis-content-004b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-flex-basis-content-004b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-flex-basis-content-004b.html)
- [flexbox_flex-basis.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-basis.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-basis.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-basis.html)
- [flexbox_flex-basis-shrink.html](https://wpt.fyi/results/css/css-flexbox/flexbox_flex-basis-shrink.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_flex-basis-shrink.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_flex-basis-shrink.html)
- [flex-one-sets-flex-basis-to-zero-px.html](https://wpt.fyi/results/css/css-flexbox/flex-one-sets-flex-basis-to-zero-px.html) [(live test)](http://wpt.live/css/css-flexbox/flex-one-sets-flex-basis-to-zero-px.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-one-sets-flex-basis-to-zero-px.html)
- [flex-shorthand-flex-basis-middle.html](https://wpt.fyi/results/css/css-flexbox/flex-shorthand-flex-basis-middle.html) [(live test)](http://wpt.live/css/css-flexbox/flex-shorthand-flex-basis-middle.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-shorthand-flex-basis-middle.html)
- [flexbox_computedstyle_flex-basis-0.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-0.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-0.html)
- [flexbox_computedstyle_flex-basis-0percent.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-0percent.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-0percent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-0percent.html)
- [flexbox_computedstyle_flex-basis-auto.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-auto.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-auto.html)
- [flexbox_computedstyle_flex-basis-percent.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-percent.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-percent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_flex-basis-percent.html)
- flexbox_interactive_flex-basis-transitions.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/interactive/flexbox_interactive_flex-basis-transitions.html)
- [flex-basis-computed.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-basis-computed.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-basis-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-basis-computed.html)
- [flex-basis-invalid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-basis-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-basis-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-basis-invalid.html)
- [flex-basis-valid.html](https://wpt.fyi/results/css/css-flexbox/parsing/flex-basis-valid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/flex-basis-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/flex-basis-valid.html)

<a id="ref-for-propdef-flex③⓪"></a>

<a id="ref-for-propdef-flex-basis⑧"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Authors are encouraged to control flexibility using the <a href="#propdef-flex">flex</a> shorthand
	rather than with <a href="#propdef-flex-basis">flex-basis</a> directly,
	as the shorthand correctly resets any unspecified components
	to accommodate <a href="#flex-common">common uses</a>.</strong>

<a id="ref-for-propdef-flex-basis⑨"></a>

<a id="ref-for-flex-flex-basis②"></a>

<a id="ref-for-propdef-width①②"></a>

<a id="ref-for-propdef-height⑨"></a>

<a id="ref-for-valdef-flex-basis-content②"></a>

The [flex-basis](#propdef-flex-basis) property sets the [flex basis](#flex-flex-basis). It accepts the same values as the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) property, plus [content](#valdef-flex-basis-content).

<a id="ref-for-valdef-flex-basis-auto②"></a>

<a id="ref-for-valdef-flex-basis-content③"></a>

<a id="ref-for-propdef-flex-basis①⓪"></a>

<a id="ref-for-propdef-width①③"></a>

<a id="ref-for-valdef-width-auto③"></a>

<a id="ref-for-flex-container②⑦"></a>

<a id="ref-for-definite⑥"></a>

<a id="ref-for-propdef-box-sizing"></a>

For all values other than [auto](#valdef-flex-basis-auto) and [content](#valdef-flex-basis-content) (defined above), [flex-basis](#propdef-flex-basis) is resolved the same way as [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) in horizontal writing modes [\[CSS2\]](#biblio-css2), except that if a value would resolve to [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) for <a id="ref-for-propdef-width①④"></a>width, it instead resolves to <a id="ref-for-valdef-flex-basis-content④"></a>content for <a id="ref-for-propdef-flex-basis①①"></a>flex-basis. For example, percentage values of <a id="ref-for-propdef-flex-basis①②"></a>flex-basis are resolved against the flex item’s containing block (i.e. its [flex container](#flex-container)); and if that containing block’s size is [indefinite](#definite), the used value for <a id="ref-for-propdef-flex-basis①③"></a>flex-basis is <a id="ref-for-valdef-flex-basis-content⑤"></a>content. As another corollary, <a id="ref-for-propdef-flex-basis①④"></a>flex-basis determines the size of the content box, unless otherwise specified such as by [box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing) [\[CSS3UI\]](#biblio-css3ui).

## <a id="alignment"></a>8.  Alignment

After a flex container’s contents have finished their flexing and the dimensions of all flex items are finalized, they can then be aligned within the flex container.

<a id="ref-for-propdef-margin"></a>

<a id="ref-for-flex-item④④"></a>

<a id="ref-for-main-axis①①"></a>

<a id="ref-for-cross-axis⑦"></a>

The [margin](https://www.w3.org/TR/CSS2/box.html#propdef-margin) properties can be used to align items in a manner similar to, but more powerful than, what margins can do in block layout. [Flex items](#flex-item) also respect the alignment properties from [CSS Box Alignment](https://www.w3.org/TR/css3-align/), which allow easy keyword-based alignment of items in both the [main axis](#main-axis) and [cross axis](#cross-axis). These properties make many common types of alignment trivial, including some things that were very difficult in CSS 2.1, like horizontal and vertical centering.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While the alignment properties are defined in [CSS Box Alignment](https://www.w3.org/TR/css3-align/) [\[CSS-ALIGN-3\]](#biblio-css-align-3), Flexible Box Layout reproduces the definitions of the relevant ones here so as to not create a normative dependency that may slow down advancement of the spec. These properties apply only to flex layout until [CSS Box Alignment Level 3](https://www.w3.org/TR/css3-align/) is finished and defines their effect for other layout modes. Additionally, any new values defined in the Box Alignment module will apply to Flexible Box Layout; in other words, the Box Alignment module, once completed, will supersede the definitions here.

Tests

- [baseline-outside-flex-item.html](https://wpt.fyi/results/css/css-flexbox/baseline-outside-flex-item.html) [(live test)](http://wpt.live/css/css-flexbox/baseline-outside-flex-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/baseline-outside-flex-item.html)
- [fieldset-baseline-alignment.html](https://wpt.fyi/results/css/css-flexbox/fieldset-baseline-alignment.html) [(live test)](http://wpt.live/css/css-flexbox/fieldset-baseline-alignment.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/fieldset-baseline-alignment.html)

### <a id="auto-margins"></a>8.1.  Aligning with auto margins

<em>This section is non-normative.
		The normative definition of how margins affect flex items is in the <a href="#layout-algorithm">Flex Layout Algorithm</a> section.</em>

Auto margins on flex items have an effect very similar to auto margins in block flow:

- During calculations of flex bases and flexible lengths, auto margins are treated as 0.

- <a id="ref-for-propdef-justify-content③"></a>

  <a id="ref-for-propdef-align-self⑧"></a>

  Prior to alignment via [justify-content](#propdef-justify-content) and [align-self](#propdef-align-self), any positive free space is distributed to auto margins in that dimension.

- <a id="ref-for-end"></a>

  Overflowing boxes ignore their auto margins and overflow in the [end](https://www.w3.org/TR/css-writing-modes-3/#end) direction.

Tests

- [auto-height-column-with-border-and-padding.html](https://wpt.fyi/results/css/css-flexbox/auto-height-column-with-border-and-padding.html) [(live test)](http://wpt.live/css/css-flexbox/auto-height-column-with-border-and-padding.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/auto-height-column-with-border-and-padding.html)
- [auto-height-with-flex.html](https://wpt.fyi/results/css/css-flexbox/auto-height-with-flex.html) [(live test)](http://wpt.live/css/css-flexbox/auto-height-with-flex.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/auto-height-with-flex.html)
- [auto-margins-001.html](https://wpt.fyi/results/css/css-flexbox/auto-margins-001.html) [(live test)](http://wpt.live/css/css-flexbox/auto-margins-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/auto-margins-001.html)
- [auto-margins-002.html](https://wpt.fyi/results/css/css-flexbox/auto-margins-002.html) [(live test)](http://wpt.live/css/css-flexbox/auto-margins-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/auto-margins-002.html)
- [auto-margins-003.html](https://wpt.fyi/results/css/css-flexbox/auto-margins-003.html) [(live test)](http://wpt.live/css/css-flexbox/auto-margins-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/auto-margins-003.html)
- [flexbox-margin-auto-horiz-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-margin-auto-horiz-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-margin-auto-horiz-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-margin-auto-horiz-001.xhtml)
- [flexbox-margin-auto-horiz-002.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-margin-auto-horiz-002.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-margin-auto-horiz-002.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-margin-auto-horiz-002.xhtml)
- [flexbox_margin-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_margin-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_margin-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_margin-auto.html)
- [flexbox_margin-auto-overflow.html](https://wpt.fyi/results/css/css-flexbox/flexbox_margin-auto-overflow.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_margin-auto-overflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_margin-auto-overflow.html)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If free space is distributed to auto margins, the alignment properties will have no effect in that dimension because the margins will have stolen all the free space left over after flexing.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-243a1d6d"></a> One use of auto margins in the main axis is to separate flex items into distinct "groups". The following example shows how to use this to reproduce a common UI pattern - a single bar of actions with some aligned on the left and others aligned on the right.
>
> Sample rendering of the code below.
>
> <a id="auto-bar"></a>
>
> - [About](css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
> - [Projects](css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
> - [Interact](css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
> - <a id="login"></a>[Login](css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
>
> ```css
> nav > ul {
>   display: flex;
> }
> nav > ul > #login {
>   margin-left: auto;
> }
> ```
>
> ```markup
> <nav>
>   <ul>
>     <li><a href=/about>About</a>
>     <li><a href=/projects>Projects</a>
>     <li><a href=/interact>Interact</a>
>     <li id="login"><a href=/login>Login</a>
>   </ul>
> </nav>
> ```
<a id="ref-for-propdef-align-items①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-543514d9"></a> The figure below illustrates the difference in cross-axis alignment in overflow situations between using [auto margins](#auto-margins) and using the [alignment properties](#propdef-align-items).
>
> About
>
> Authoritarianism
>
> Blog
>
> About
>
> Authoritarianism
>
> Blog
>
> <a id="ref-for-propdef-align-self⑨"></a>
>
> The items in the figure on the left are centered with margins, while those in the figure on the right are centered with [align-self](#propdef-align-self). If this column flex container was placed against the left edge of the page, the margin behavior would be more desirable, as the long item would be fully readable. In other circumstances, the true centering behavior might be better.

<a id="ref-for-propdef-justify-content④"></a>

### <a id="justify-content-property"></a>8.2.  Axis Alignment: the [justify-content](#propdef-justify-content) property

| Field               | Definition                                                                                                                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-justify-content"></a>justify-content                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑧"></a>flex-start [\|](https://www.w3.org/TR/css-values-4/#comb-one) flex-end <a id="ref-for-comb-one⑨"></a>\| center <a id="ref-for-comb-one①⓪"></a>\| space-between <a id="ref-for-comb-one①①"></a>\| space-around |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | flex-start                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-flex-container②⑧"></a>[flex containers](#flex-container)                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                      |

Tests

- [dynamic-isize-change-001.html](https://wpt.fyi/results/css/css-flexbox/dynamic-isize-change-001.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-isize-change-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-isize-change-001.html)
- [dynamic-isize-change-002.html](https://wpt.fyi/results/css/css-flexbox/dynamic-isize-change-002.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-isize-change-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-isize-change-002.html)
- [dynamic-isize-change-003.html](https://wpt.fyi/results/css/css-flexbox/dynamic-isize-change-003.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-isize-change-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-isize-change-003.html)
- [dynamic-isize-change-004.html](https://wpt.fyi/results/css/css-flexbox/dynamic-isize-change-004.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-isize-change-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-isize-change-004.html)
- [flexbox_justifycontent-center.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-center.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-center.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-center.html)
- [flexbox_justifycontent-center-overflow.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-center-overflow.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-center-overflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-center-overflow.html)
- [flexbox_justifycontent-end.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-end.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-end.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-end.html)
- [flexbox_justifycontent-end-rtl.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-end-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-end-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-end-rtl.html)
- [flexbox_justifycontent-flex-end.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-flex-end.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-flex-end.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-flex-end.html)
- [flexbox_justifycontent-flex-start.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-flex-start.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-flex-start.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-flex-start.html)
- [flexbox-justify-content-horiz-001a.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-horiz-001a.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-horiz-001a.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-horiz-001a.xhtml)
- [flexbox-justify-content-horiz-001b.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-horiz-001b.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-horiz-001b.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-horiz-001b.xhtml)
- [flexbox-justify-content-horiz-002.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-horiz-002.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-horiz-002.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-horiz-002.xhtml)
- [flexbox-justify-content-horiz-003.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-horiz-003.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-horiz-003.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-horiz-003.xhtml)
- [flexbox-justify-content-horiz-004.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-horiz-004.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-horiz-004.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-horiz-004.xhtml)
- [flexbox-justify-content-horiz-005.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-horiz-005.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-horiz-005.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-horiz-005.xhtml)
- [flexbox-justify-content-horiz-006.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-horiz-006.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-horiz-006.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-horiz-006.xhtml)
- [flexbox_justifycontent-left-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-left-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-left-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-left-001.html)
- [flexbox_justifycontent-left-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-left-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-left-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-left-002.html)
- [flexbox_justifycontent-right-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-right-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-right-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-right-001.html)
- [flexbox_justifycontent-right-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-right-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-right-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-right-002.html)
- [flexbox_justifycontent-rtl-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-rtl-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-rtl-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-rtl-001.html)
- [flexbox_justifycontent-rtl-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-rtl-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-rtl-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-rtl-002.html)
- [flexbox_justifycontent-spacearound.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-spacearound.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-spacearound.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-spacearound.html)
- [flexbox_justifycontent-spacearound-negative.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-spacearound-negative.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-spacearound-negative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-spacearound-negative.html)
- [flexbox_justifycontent-spacearound-only.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-spacearound-only.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-spacearound-only.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-spacearound-only.html)
- [flexbox_justifycontent-spacebetween.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-spacebetween.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-spacebetween.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-spacebetween.html)
- [flexbox_justifycontent-spacebetween-negative.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-spacebetween-negative.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-spacebetween-negative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-spacebetween-negative.html)
- [flexbox_justifycontent-spacebetween-only.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-spacebetween-only.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-spacebetween-only.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-spacebetween-only.html)
- [flexbox_justifycontent-start.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-start.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-start.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-start.html)
- [flexbox_justifycontent-start-rtl.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-start-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-start-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-start-rtl.html)
- [flexbox_justifycontent-stretch.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-stretch.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-stretch.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-stretch.html)
- [flexbox-justify-content-vert-001a.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-vert-001a.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-vert-001a.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-vert-001a.xhtml)
- [flexbox-justify-content-vert-001b.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-vert-001b.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-vert-001b.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-vert-001b.xhtml)
- [flexbox-justify-content-vert-002.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-vert-002.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-vert-002.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-vert-002.xhtml)
- [flexbox-justify-content-vert-003.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-vert-003.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-vert-003.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-vert-003.xhtml)
- [flexbox-justify-content-vert-004.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-vert-004.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-vert-004.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-vert-004.xhtml)
- [flexbox-justify-content-vert-005.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-vert-005.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-vert-005.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-vert-005.xhtml)
- [flexbox-justify-content-vert-006.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-vert-006.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-vert-006.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-vert-006.xhtml)
- [flexbox-justify-content-wmvert-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-wmvert-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-wmvert-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-wmvert-001.xhtml)
- [flexbox-justify-content-wmvert-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-wmvert-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-wmvert-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-wmvert-002.html)
- [flexbox-justify-content-wmvert-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-justify-content-wmvert-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-justify-content-wmvert-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-justify-content-wmvert-003.html)
- [flexbox_margin.html](https://wpt.fyi/results/css/css-flexbox/flexbox_margin.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_margin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_margin.html)
- [flexbox_margin-left-ex.html](https://wpt.fyi/results/css/css-flexbox/flexbox_margin-left-ex.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_margin-left-ex.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_margin-left-ex.html)
- [flexbox_computedstyle_justify-content-center.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-center.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-center.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-center.html)
- [flexbox_computedstyle_justify-content-flex-end.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-flex-end.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-flex-end.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-flex-end.html)
- [flexbox_computedstyle_justify-content-flex-start.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-flex-start.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-flex-start.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-flex-start.html)
- [flexbox_computedstyle_justify-content-space-around.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-space-around.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-space-around.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-space-around.html)
- [flexbox_computedstyle_justify-content-space-between.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-space-between.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-space-between.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_justify-content-space-between.html)
- [justify-content-001.htm](https://wpt.fyi/results/css/css-flexbox/justify-content-001.htm) [(live test)](http://wpt.live/css/css-flexbox/justify-content-001.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content-001.htm)
- [justify-content-002.htm](https://wpt.fyi/results/css/css-flexbox/justify-content-002.htm) [(live test)](http://wpt.live/css/css-flexbox/justify-content-002.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content-002.htm)
- [justify-content-003.htm](https://wpt.fyi/results/css/css-flexbox/justify-content-003.htm) [(live test)](http://wpt.live/css/css-flexbox/justify-content-003.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content-003.htm)
- [justify-content-004.htm](https://wpt.fyi/results/css/css-flexbox/justify-content-004.htm) [(live test)](http://wpt.live/css/css-flexbox/justify-content-004.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content-004.htm)
- [justify-content-005.htm](https://wpt.fyi/results/css/css-flexbox/justify-content-005.htm) [(live test)](http://wpt.live/css/css-flexbox/justify-content-005.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content-005.htm)
- [justify-content-006.html](https://wpt.fyi/results/css/css-flexbox/justify-content-006.html) [(live test)](http://wpt.live/css/css-flexbox/justify-content-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content-006.html)
- [justify-content-007.html](https://wpt.fyi/results/css/css-flexbox/justify-content-007.html) [(live test)](http://wpt.live/css/css-flexbox/justify-content-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content-007.html)
- justify-content_center.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content_center.html)
- justify-content_flex-end.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content_flex-end.html)
- justify-content_flex-start.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content_flex-start.html)
- [justify-content-sideways-001.html](https://wpt.fyi/results/css/css-flexbox/justify-content-sideways-001.html) [(live test)](http://wpt.live/css/css-flexbox/justify-content-sideways-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content-sideways-001.html)
- justify-content_space-around.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content_space-around.html)
- justify-content_space-between-001.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content_space-between-001.html)
- [justify-content_space-between-002.html](https://wpt.fyi/results/css/css-flexbox/justify-content_space-between-002.html) [(live test)](http://wpt.live/css/css-flexbox/justify-content_space-between-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/justify-content_space-between-002.html)
- [scrollbars-auto.html](https://wpt.fyi/results/css/css-flexbox/scrollbars-auto.html) [(live test)](http://wpt.live/css/css-flexbox/scrollbars-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/scrollbars-auto.html)
- [scrollbars.html](https://wpt.fyi/results/css/css-flexbox/scrollbars.html) [(live test)](http://wpt.live/css/css-flexbox/scrollbars.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/scrollbars.html)
- [scrollbars-no-margin.html](https://wpt.fyi/results/css/css-flexbox/scrollbars-no-margin.html) [(live test)](http://wpt.live/css/css-flexbox/scrollbars-no-margin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/scrollbars-no-margin.html)

<a id="ref-for-propdef-justify-content⑤"></a>

<a id="ref-for-flex-item④⑤"></a>

<a id="ref-for-main-axis①②"></a>

The [justify-content](#propdef-justify-content) property aligns [flex items](#flex-item) along the [main axis](#main-axis) of the current line of the flex container. This is done <em>after</em> any flexible lengths and any [auto margins](#auto-margins) have been resolved. Typically it helps distribute extra free space leftover when either all the <a id="ref-for-flex-item④⑥"></a>flex items on a line are inflexible, or are flexible but have reached their maximum size. It also exerts some control over the alignment of items when they overflow the line.

<a id="valdef-justify-content-flex-start"></a>flex-start  
<a id="ref-for-main-start④"></a>

<a id="ref-for-flex-item④⑦"></a>

[Flex items](#flex-item) are packed toward the start of the line. The [main-start](#main-start) margin edge of the first <a id="ref-for-flex-item④⑧"></a>flex item on the line is placed flush with the <a id="ref-for-main-start⑤"></a>main-start edge of the line, and each subsequent <a id="ref-for-flex-item④⑨"></a>flex item is placed flush with the preceding item.

Tests

- [flexbox_justifycontent-start.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-start.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-start.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-start.html)
- [flexbox_justifycontent-start-rtl.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-start-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-start-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-start-rtl.html)

<a id="valdef-justify-content-flex-end"></a>flex-end  
<a id="ref-for-main-end④"></a>

<a id="ref-for-flex-item⑤⓪"></a>

[Flex items](#flex-item) are packed toward the end of the line. The [main-end](#main-end) margin edge of the last <a id="ref-for-flex-item⑤①"></a>flex item is placed flush with the <a id="ref-for-main-end⑤"></a>main-end edge of the line, and each preceding <a id="ref-for-flex-item⑤②"></a>flex item is placed flush with the subsequent item.

Tests

- [css-box-justify-content.html](https://wpt.fyi/results/css/css-flexbox/css-box-justify-content.html) [(live test)](http://wpt.live/css/css-flexbox/css-box-justify-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/css-box-justify-content.html)
- [flexbox_justifycontent-end.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-end.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-end.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-end.html)
- [flexbox_justifycontent-end-rtl.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-end-rtl.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-end-rtl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-end-rtl.html)
- [flexbox_justifycontent-flex-end.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-flex-end.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-flex-end.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-flex-end.html)

<a id="valdef-justify-content-center"></a>center  
<a id="ref-for-main-end⑥"></a>

<a id="ref-for-main-start⑥"></a>

<a id="ref-for-flex-item⑤③"></a>

[Flex items](#flex-item) are packed toward the center of the line. The <a id="ref-for-flex-item⑤④"></a>flex items on the line are placed flush with each other and aligned in the center of the line, with equal amounts of space between the [main-start](#main-start) edge of the line and the first item on the line and between the [main-end](#main-end) edge of the line and the last item on the line. (If the leftover free-space is negative, the <a id="ref-for-flex-item⑤⑤"></a>flex items will overflow equally in both directions.)

Tests

- [flexbox_justifycontent-center.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-center.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-center.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-center.html)
- [flexbox_justifycontent-center-overflow.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-center-overflow.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-center-overflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-center-overflow.html)

<a id="valdef-justify-content-space-between"></a>space-between  
<a id="ref-for-main-end⑦"></a>

<a id="ref-for-main-start⑦"></a>

<a id="ref-for-flex-item⑤⑥"></a>

[Flex items](#flex-item) are evenly distributed in the line. If the leftover free-space is negative or there is only a single <a id="ref-for-flex-item⑤⑦"></a>flex item on the line, this value falls back to safe flex-start. Otherwise, the [main-start](#main-start) margin edge of the first <a id="ref-for-flex-item⑤⑧"></a>flex item on the line is placed flush with the <a id="ref-for-main-start⑧"></a>main-start edge of the line, the [main-end](#main-end) margin edge of the last <a id="ref-for-flex-item⑤⑨"></a>flex item on the line is placed flush with the <a id="ref-for-main-end⑧"></a>main-end edge of the line, and the remaining <a id="ref-for-flex-item⑥⓪"></a>flex items on the line are distributed so that the spacing between any two adjacent items is the same.

Tests

- [flexbox_justifycontent-spacebetween.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-spacebetween.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-spacebetween.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-spacebetween.html)
- [flexbox_justifycontent-spacebetween-negative.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-spacebetween-negative.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-spacebetween-negative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-spacebetween-negative.html)
- [flexbox_justifycontent-spacebetween-only.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-spacebetween-only.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-spacebetween-only.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-spacebetween-only.html)

<a id="valdef-justify-content-space-around"></a>space-around  
<a id="ref-for-flex-container②⑨"></a>

<a id="ref-for-flex-item⑥①"></a>

[Flex items](#flex-item) are evenly distributed in the line, with half-size spaces on either end. If the leftover free-space is negative or there is only a single <a id="ref-for-flex-item⑥②"></a>flex item on the line, this value falls back to safe center. Otherwise, the <a id="ref-for-flex-item⑥③"></a>flex items on the line are distributed such that the spacing between any two adjacent <a id="ref-for-flex-item⑥④"></a>flex items on the line is the same, and the spacing between the first/last <a id="ref-for-flex-item⑥⑤"></a>flex items and the [flex container](#flex-container) edges is half the size of the spacing between <a id="ref-for-flex-item⑥⑥"></a>flex items.

Tests

- [flexbox-column-row-gap-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-column-row-gap-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-column-row-gap-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-column-row-gap-001.html)
- [flexbox-column-row-gap-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-column-row-gap-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-column-row-gap-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-column-row-gap-002.html)
- [flexbox-column-row-gap-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-column-row-gap-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-column-row-gap-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-column-row-gap-003.html)
- [flexbox-column-row-gap-004.html](https://wpt.fyi/results/css/css-flexbox/flexbox-column-row-gap-004.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-column-row-gap-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-column-row-gap-004.html)
- [flexbox_columns-flexitems-2.html](https://wpt.fyi/results/css/css-flexbox/flexbox_columns-flexitems-2.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_columns-flexitems-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_columns-flexitems-2.html)
- [flexbox_columns-flexitems.html](https://wpt.fyi/results/css/css-flexbox/flexbox_columns-flexitems.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_columns-flexitems.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_columns-flexitems.html)
- [flexbox_justifycontent-spacearound.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-spacearound.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-spacearound.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-spacearound.html)
- [flexbox_justifycontent-spacearound-negative.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-spacearound-negative.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-spacearound-negative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-spacearound-negative.html)
- [flexbox_justifycontent-spacearound-only.html](https://wpt.fyi/results/css/css-flexbox/flexbox_justifycontent-spacearound-only.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_justifycontent-spacearound-only.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_justifycontent-spacearound-only.html)

![](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/flex-pack.svg)

<a id="ref-for-propdef-justify-content⑥"></a>

An illustration of the five [justify-content](#propdef-justify-content) keywords and their effects on a flex container with three colored items.

<a id="ref-for-propdef-align-items②"></a>

<a id="ref-for-propdef-align-self①⓪"></a>

### <a id="align-items-property"></a>8.3.  Cross-axis Alignment: the [align-items](#propdef-align-items) and [align-self](#propdef-align-self) properties

| Field               | Definition                                                                                                                                                                          |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-align-items"></a>align-items                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①②"></a>flex-start [\|](https://www.w3.org/TR/css-values-4/#comb-one) flex-end <a id="ref-for-comb-one①③"></a>\| center <a id="ref-for-comb-one①④"></a>\| baseline <a id="ref-for-comb-one①⑤"></a>\| stretch |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | stretch                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-flex-container③⓪"></a>[flex containers](#flex-container)                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                            |

Tests

- [align-items-007.html](https://wpt.fyi/results/css/css-flexbox/align-items-007.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-007.html)
- [align-items-008.html](https://wpt.fyi/results/css/css-flexbox/align-items-008.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-008.html)
- [align-items-009.html](https://wpt.fyi/results/css/css-flexbox/align-items-009.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-009.html)
- [align-items-baseline-column-horz.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-column-horz.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-column-horz.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-column-horz.html)
- [align-items-baseline-column-vert.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-column-vert.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-column-vert.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-column-vert.html)
- [align-items-baseline-column-vert-lr-flexbox-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-column-vert-lr-flexbox-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-column-vert-lr-flexbox-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-column-vert-lr-flexbox-item.html)
- [align-items-baseline-column-vert-lr-grid-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-column-vert-lr-grid-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-column-vert-lr-grid-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-column-vert-lr-grid-item.html)
- [align-items-baseline-column-vert-lr-items.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-column-vert-lr-items.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-column-vert-lr-items.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-column-vert-lr-items.html)
- [align-items-baseline-column-vert-lr-table-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-column-vert-lr-table-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-column-vert-lr-table-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-column-vert-lr-table-item.html)
- [align-items-baseline-column-vert-rl-flexbox-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-column-vert-rl-flexbox-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-column-vert-rl-flexbox-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-column-vert-rl-flexbox-item.html)
- [align-items-baseline-column-vert-rl-grid-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-column-vert-rl-grid-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-column-vert-rl-grid-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-column-vert-rl-grid-item.html)
- [align-items-baseline-column-vert-rl-items.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-column-vert-rl-items.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-column-vert-rl-items.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-column-vert-rl-items.html)
- [align-items-baseline-column-vert-rl-table-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-column-vert-rl-table-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-column-vert-rl-table-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-column-vert-rl-table-item.html)
- [align-items-baseline-overflow-non-visible.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-overflow-non-visible.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-overflow-non-visible.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-overflow-non-visible.html)
- [align-items-baseline-row-horz.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-row-horz.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-row-horz.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-row-horz.html)
- [align-items-baseline-row-vert.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-row-vert.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-row-vert.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-row-vert.html)
- [align-items-baseline-vert-lr-column-horz-flexbox-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-vert-lr-column-horz-flexbox-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-vert-lr-column-horz-flexbox-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-vert-lr-column-horz-flexbox-item.html)
- [align-items-baseline-vert-lr-column-horz-grid-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-vert-lr-column-horz-grid-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-vert-lr-column-horz-grid-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-vert-lr-column-horz-grid-item.html)
- [align-items-baseline-vert-lr-column-horz-items.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-vert-lr-column-horz-items.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-vert-lr-column-horz-items.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-vert-lr-column-horz-items.html)
- [align-items-baseline-vert-lr-column-horz-table-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-vert-lr-column-horz-table-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-vert-lr-column-horz-table-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-vert-lr-column-horz-table-item.html)
- [align-items-baseline-vert-rl-column-horz-flexbox-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-vert-rl-column-horz-flexbox-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-vert-rl-column-horz-flexbox-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-vert-rl-column-horz-flexbox-item.html)
- [align-items-baseline-vert-rl-column-horz-grid-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-vert-rl-column-horz-grid-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-vert-rl-column-horz-grid-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-vert-rl-column-horz-grid-item.html)
- [align-items-baseline-vert-rl-column-horz-items.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-vert-rl-column-horz-items.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-vert-rl-column-horz-items.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-vert-rl-column-horz-items.html)
- [align-items-baseline-vert-rl-column-horz-table-item.html](https://wpt.fyi/results/css/css-flexbox/align-items-baseline-vert-rl-column-horz-table-item.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-baseline-vert-rl-column-horz-table-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-baseline-vert-rl-column-horz-table-item.html)
- [multiline-align-self.html](https://wpt.fyi/results/css/css-flexbox/alignment/multiline-align-self.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/multiline-align-self.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/multiline-align-self.html)
- [column-flex-child-with-max-width.html](https://wpt.fyi/results/css/css-flexbox/column-flex-child-with-max-width.html) [(live test)](http://wpt.live/css/css-flexbox/column-flex-child-with-max-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/column-flex-child-with-max-width.html)
- [dynamic-stretch-change.html](https://wpt.fyi/results/css/css-flexbox/dynamic-stretch-change.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-stretch-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-stretch-change.html)
- [flexbox_align-items-baseline.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-items-baseline.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-items-baseline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-items-baseline.html)
- [flexbox_align-items-center-2.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-items-center-2.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-items-center-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-items-center-2.html)
- [flexbox_align-items-center-3.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-items-center-3.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-items-center-3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-items-center-3.html)
- [flexbox_align-items-center.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-items-center.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-items-center.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-items-center.html)
- [flexbox-align-items-center-nested-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-align-items-center-nested-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-items-center-nested-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-items-center-nested-001.html)
- [flexbox_align-items-flexend-2.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-items-flexend-2.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-items-flexend-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-items-flexend-2.html)
- [flexbox_align-items-flexend.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-items-flexend.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-items-flexend.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-items-flexend.html)
- [flexbox_align-items-flexstart-2.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-items-flexstart-2.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-items-flexstart-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-items-flexstart-2.html)
- [flexbox_align-items-flexstart.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-items-flexstart.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-items-flexstart.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-items-flexstart.html)
- [flexbox_align-items-stretch-2.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-items-stretch-2.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-items-stretch-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-items-stretch-2.html)
- [flexbox_align-items-stretch-3.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-items-stretch-3.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-items-stretch-3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-items-stretch-3.html)
- [flexbox_align-items-stretch-4.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-items-stretch-4.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-items-stretch-4.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-items-stretch-4.html)
- [flexbox_align-items-stretch.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-items-stretch.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-items-stretch.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-items-stretch.html)
- [flexbox_computedstyle_align-items-baseline.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-baseline.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-baseline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-baseline.html)
- [flexbox_computedstyle_align-items-center.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-center.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-center.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-center.html)
- [flexbox_computedstyle_align-items-flex-end.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-flex-end.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-flex-end.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-flex-end.html)
- [flexbox_computedstyle_align-items-flex-start.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-flex-start.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-flex-start.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-flex-start.html)
- [flexbox_computedstyle_align-items-invalid.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-invalid.html)
- [flexbox_computedstyle_align-items-stretch.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-stretch.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-stretch.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-items-stretch.html)
- [position-relative-percentage-top-001.html](https://wpt.fyi/results/css/css-flexbox/position-relative-percentage-top-001.html) [(live test)](http://wpt.live/css/css-flexbox/position-relative-percentage-top-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/position-relative-percentage-top-001.html)
- [position-relative-percentage-top-002.html](https://wpt.fyi/results/css/css-flexbox/position-relative-percentage-top-002.html) [(live test)](http://wpt.live/css/css-flexbox/position-relative-percentage-top-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/position-relative-percentage-top-002.html)
- [position-relative-percentage-top-003.html](https://wpt.fyi/results/css/css-flexbox/position-relative-percentage-top-003.html) [(live test)](http://wpt.live/css/css-flexbox/position-relative-percentage-top-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/position-relative-percentage-top-003.html)
- [relayout-align-items.html](https://wpt.fyi/results/css/css-flexbox/relayout-align-items.html) [(live test)](http://wpt.live/css/css-flexbox/relayout-align-items.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/relayout-align-items.html)
- [stretch-input-in-column.html](https://wpt.fyi/results/css/css-flexbox/stretch-input-in-column.html) [(live test)](http://wpt.live/css/css-flexbox/stretch-input-in-column.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/stretch-input-in-column.html)
- [stretch-requires-computed-auto-size.html](https://wpt.fyi/results/css/css-flexbox/stretch-requires-computed-auto-size.html) [(live test)](http://wpt.live/css/css-flexbox/stretch-requires-computed-auto-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/stretch-requires-computed-auto-size.html)
- [table-as-item-stretch-cross-size-2.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-stretch-cross-size-2.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-stretch-cross-size-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-stretch-cross-size-2.html)
- [table-as-item-stretch-cross-size-3.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-stretch-cross-size-3.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-stretch-cross-size-3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-stretch-cross-size-3.html)
- [table-as-item-stretch-cross-size-4.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-stretch-cross-size-4.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-stretch-cross-size-4.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-stretch-cross-size-4.html)
- [table-as-item-stretch-cross-size-5.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-stretch-cross-size-5.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-stretch-cross-size-5.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-stretch-cross-size-5.html)
- [table-as-item-stretch-cross-size.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-stretch-cross-size.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-stretch-cross-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-stretch-cross-size.html)

| Field               | Definition                                                                                                                                                                                                     |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-align-self"></a>align-self                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑥"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) flex-start <a id="ref-for-comb-one①⑦"></a>\| flex-end <a id="ref-for-comb-one①⑧"></a>\| center <a id="ref-for-comb-one①⑨"></a>\| baseline <a id="ref-for-comb-one②⓪"></a>\| stretch |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-flex-item⑥⑦"></a>[flex items](#flex-item)                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                       |

Tests

- align-self-006.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-006.html)
- align-self-010.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-010.html)
- [align-self-013.html](https://wpt.fyi/results/css/css-flexbox/align-self-013.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-013.html)
- [align-self-014.html](https://wpt.fyi/results/css/css-flexbox/align-self-014.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-014.html)
- [align-self-015.html](https://wpt.fyi/results/css/css-flexbox/align-self-015.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-015.html)
- [align-self-016.html](https://wpt.fyi/results/css/css-flexbox/align-self-016.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-016.html)
- [flexbox_align-self-auto.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-self-auto.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-self-auto.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-self-auto.html)
- [flexbox-align-self-baseline-compatability.html](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-baseline-compatability.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-baseline-compatability.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-baseline-compatability.html)
- [flexbox-align-self-baseline-horiz-001a.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-baseline-horiz-001a.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-baseline-horiz-001a.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-baseline-horiz-001a.xhtml)
- [flexbox-align-self-baseline-horiz-001b.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-baseline-horiz-001b.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-baseline-horiz-001b.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-baseline-horiz-001b.xhtml)
- [flexbox-align-self-baseline-horiz-002.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-baseline-horiz-002.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-baseline-horiz-002.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-baseline-horiz-002.xhtml)
- [flexbox-align-self-baseline-horiz-003.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-baseline-horiz-003.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-baseline-horiz-003.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-baseline-horiz-003.xhtml)
- [flexbox-align-self-baseline-horiz-004.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-baseline-horiz-004.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-baseline-horiz-004.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-baseline-horiz-004.xhtml)
- [flexbox-align-self-baseline-horiz-005.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-baseline-horiz-005.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-baseline-horiz-005.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-baseline-horiz-005.xhtml)
- [flexbox-align-self-baseline-horiz-006.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-baseline-horiz-006.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-baseline-horiz-006.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-baseline-horiz-006.xhtml)
- [flexbox-align-self-baseline-horiz-007.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-baseline-horiz-007.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-baseline-horiz-007.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-baseline-horiz-007.xhtml)
- [flexbox-align-self-baseline-horiz-008.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-baseline-horiz-008.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-baseline-horiz-008.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-baseline-horiz-008.xhtml)
- [flexbox_align-self-baseline.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-self-baseline.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-self-baseline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-self-baseline.html)
- [flexbox_align-self-center.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-self-center.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-self-center.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-self-center.html)
- [flexbox_align-self-flexend.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-self-flexend.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-self-flexend.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-self-flexend.html)
- [flexbox_align-self-flexstart.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-self-flexstart.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-self-flexstart.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-self-flexstart.html)
- [flexbox-align-self-horiz-001-block.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-horiz-001-block.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-horiz-001-block.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-horiz-001-block.xhtml)
- [flexbox-align-self-horiz-001-table.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-horiz-001-table.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-horiz-001-table.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-horiz-001-table.xhtml)
- [flexbox-align-self-horiz-002.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-horiz-002.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-horiz-002.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-horiz-002.xhtml)
- [flexbox-align-self-horiz-003.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-horiz-003.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-horiz-003.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-horiz-003.xhtml)
- [flexbox-align-self-horiz-004.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-horiz-004.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-horiz-004.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-horiz-004.xhtml)
- [flexbox-align-self-horiz-005.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-horiz-005.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-horiz-005.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-horiz-005.xhtml)
- [flexbox_align-self-stretch.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-self-stretch.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-self-stretch.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-self-stretch.html)
- [flexbox-align-self-stretch-vert-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-stretch-vert-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-stretch-vert-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-stretch-vert-001.html)
- [flexbox-align-self-stretch-vert-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-stretch-vert-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-stretch-vert-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-stretch-vert-002.html)
- [flexbox-align-self-vert-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-vert-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-vert-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-vert-001.xhtml)
- [flexbox-align-self-vert-002.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-vert-002.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-vert-002.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-vert-002.xhtml)
- [flexbox-align-self-vert-003.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-vert-003.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-vert-003.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-vert-003.xhtml)
- [flexbox-align-self-vert-004.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-vert-004.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-vert-004.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-vert-004.xhtml)
- [flexbox-align-self-vert-rtl-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-vert-rtl-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-vert-rtl-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-vert-rtl-001.xhtml)
- [flexbox-align-self-vert-rtl-002.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-vert-rtl-002.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-vert-rtl-002.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-vert-rtl-002.xhtml)
- [flexbox-align-self-vert-rtl-003.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-vert-rtl-003.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-vert-rtl-003.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-vert-rtl-003.xhtml)
- [flexbox-align-self-vert-rtl-004.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-vert-rtl-004.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-vert-rtl-004.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-vert-rtl-004.xhtml)
- [flexbox-align-self-vert-rtl-005.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-align-self-vert-rtl-005.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-align-self-vert-rtl-005.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-align-self-vert-rtl-005.xhtml)
- [flexbox_computedstyle_align-self-baseline.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-baseline.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-baseline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-baseline.html)
- [flexbox_computedstyle_align-self-center.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-center.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-center.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-center.html)
- [flexbox_computedstyle_align-self-flex-end.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-flex-end.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-flex-end.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-flex-end.html)
- [flexbox_computedstyle_align-self-flex-start.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-flex-start.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-flex-start.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-flex-start.html)
- [flexbox_computedstyle_align-self-invalid.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-invalid.html)
- [flexbox_computedstyle_align-self-stretch.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-stretch.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-stretch.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-self-stretch.html)

<a id="ref-for-flex-item⑥⑧"></a>

<a id="ref-for-cross-axis⑧"></a>

<a id="ref-for-propdef-justify-content⑦"></a>

<a id="ref-for-propdef-align-items③"></a>

<a id="ref-for-propdef-align-self①①"></a>

[Flex items](#flex-item) can be aligned in the [cross axis](#cross-axis) of the current line of the flex container, similar to [justify-content](#propdef-justify-content) but in the perpendicular direction. [align-items](#propdef-align-items) sets the default alignment for all of the flex container’s <a id="ref-for-flex-item⑥⑨"></a>items, including anonymous <a id="ref-for-flex-item⑦⓪"></a>flex items. [align-self](#propdef-align-self) allows this default alignment to be overridden for individual <a id="ref-for-flex-item⑦①"></a>flex items. (For anonymous flex items, <a id="ref-for-propdef-align-self①②"></a>align-self always matches the value of <a id="ref-for-propdef-align-items④"></a>align-items on their associated flex container.)

<a id="ref-for-flex-item⑦②"></a>

<a id="ref-for-propdef-align-self①③"></a>

If either of the [flex item’s](#flex-item) cross-axis margins are auto, [align-self](#propdef-align-self) has no effect.

Values have the following meanings:

<a id="valdef-align-items-auto"></a>auto  
<a id="ref-for-propdef-align-self①④"></a>

<a id="ref-for-propdef-align-items⑤"></a>

<a id="ref-for-cross-axis⑨"></a>

Defers [cross-axis](#cross-axis) alignment control to the value of [align-items](#propdef-align-items) on the parent box. (This is the initial value of [align-self](#propdef-align-self).)

Tests

- [align-self-007.html](https://wpt.fyi/results/css/css-flexbox/align-self-007.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-007.html)
- [align-self-008.html](https://wpt.fyi/results/css/css-flexbox/align-self-008.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-008.html)
- [align-self-009.html](https://wpt.fyi/results/css/css-flexbox/align-self-009.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-009.html)
- [align-self-011.html](https://wpt.fyi/results/css/css-flexbox/align-self-011.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-011.html)
- [align-self-012.html](https://wpt.fyi/results/css/css-flexbox/align-self-012.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-012.html)

<a id="valdef-align-items-flex-start"></a>flex-start  
<a id="ref-for-flex-item⑦③"></a>

<a id="ref-for-cross-start③"></a>

The [cross-start](#cross-start) margin edge of the [flex item](#flex-item) is placed flush with the <a id="ref-for-cross-start④"></a>cross-start edge of the line.

Tests

- [align-items-002.htm](https://wpt.fyi/results/css/css-flexbox/align-items-002.htm) [(live test)](http://wpt.live/css/css-flexbox/align-items-002.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-002.htm)
- [align-items-006.html](https://wpt.fyi/results/css/css-flexbox/align-items-006.html) [(live test)](http://wpt.live/css/css-flexbox/align-items-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-006.html)
- [align-self-001.html](https://wpt.fyi/results/css/css-flexbox/align-self-001.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-001.html)

<a id="valdef-align-items-flex-end"></a>flex-end  
<a id="ref-for-flex-item⑦④"></a>

<a id="ref-for-cross-end②"></a>

The [cross-end](#cross-end) margin edge of the [flex item](#flex-item) is placed flush with the <a id="ref-for-cross-end③"></a>cross-end edge of the line.

Tests

- [align-items-003.htm](https://wpt.fyi/results/css/css-flexbox/align-items-003.htm) [(live test)](http://wpt.live/css/css-flexbox/align-items-003.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-003.htm)
- [align-self-002.html](https://wpt.fyi/results/css/css-flexbox/align-self-002.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-002.html)

<a id="valdef-align-items-center"></a>center  
<a id="ref-for-cross-size⑧"></a>

<a id="ref-for-cross-axis①⓪"></a>

<a id="ref-for-flex-item⑦⑤"></a>

The [flex item](#flex-item)’s margin box is centered in the [cross axis](#cross-axis) within the line. (If the [cross size](#cross-size) of the flex line is less than that of the <a id="ref-for-flex-item⑦⑥"></a>flex item, it will overflow equally in both directions.)

Tests

- [align-items-001.htm](https://wpt.fyi/results/css/css-flexbox/align-items-001.htm) [(live test)](http://wpt.live/css/css-flexbox/align-items-001.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-001.htm)
- [align-self-003.html](https://wpt.fyi/results/css/css-flexbox/align-self-003.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-003.html)

<a id="valdef-align-items-baseline"></a>baseline  
<a id="ref-for-synthesize-baseline"></a>

<a id="ref-for-cross-start⑤"></a>

<a id="ref-for-flex-item⑦⑦"></a>

The [flex item](#flex-item) <a id="baseline-participation"></a>participates in baseline alignment: all participating <a id="ref-for-flex-item⑦⑧"></a>flex items on the line are aligned such that their baselines align, and the item with the largest distance between its baseline and its [cross-start](#cross-start) margin edge is placed flush against the <a id="ref-for-cross-start⑥"></a>cross-start edge of the line. If the item does not have a baseline in the necessary axis, then one is [synthesized](https://www.w3.org/TR/css-align-3/#synthesize-baseline) from the <a id="ref-for-flex-item⑦⑨"></a>flex item’s border box.

Tests

- [align-items-004.htm](https://wpt.fyi/results/css/css-flexbox/align-items-004.htm) [(live test)](http://wpt.live/css/css-flexbox/align-items-004.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-004.htm)
- [align-baseline.html](https://wpt.fyi/results/css/css-flexbox/align-baseline.html) [(live test)](http://wpt.live/css/css-flexbox/align-baseline.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-baseline.html)
- [dynamic-baseline-change.html](https://wpt.fyi/results/css/css-flexbox/dynamic-baseline-change.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-baseline-change.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-baseline-change.html)
- [dynamic-baseline-change-nested.html](https://wpt.fyi/results/css/css-flexbox/dynamic-baseline-change-nested.html) [(live test)](http://wpt.live/css/css-flexbox/dynamic-baseline-change-nested.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/dynamic-baseline-change-nested.html)

<a id="valdef-align-items-stretch"></a>stretch  
<a id="ref-for-propdef-max-width②"></a>

<a id="ref-for-propdef-max-height②"></a>

<a id="ref-for-propdef-min-width⑤"></a>

<a id="ref-for-propdef-min-height④"></a>

<a id="ref-for-cross-size⑨"></a>

<a id="ref-for-cross-axis①①"></a>

<a id="ref-for-valdef-width-auto④"></a>

<a id="ref-for-flex-item⑧⓪"></a>

<a id="ref-for-cross-size-property"></a>

If the [cross size property](#cross-size-property) of the [flex item](#flex-item) computes to [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), and neither of the [cross-axis](#cross-axis) margins are auto, the <a id="ref-for-flex-item⑧①"></a>flex item is <a id="stretched"></a>stretched. Its used value is the length necessary to make the [cross size](#cross-size) of the item’s margin box as close to the same size as the line as possible, while still respecting the constraints imposed by [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height)/[min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width)/[max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height)/[max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width).

<a id="ref-for-flex-item⑧②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If the flex container’s height is constrained this value may cause the contents of the [flex item](#flex-item) to overflow the item.

<a id="ref-for-cross-start⑦"></a>

<a id="ref-for-flex-item⑧③"></a>

The [cross-start](#cross-start) margin edge of the [flex item](#flex-item) is placed flush with the <a id="ref-for-cross-start⑧"></a>cross-start edge of the line.

Tests

- [align-items-005.htm](https://wpt.fyi/results/css/css-flexbox/align-items-005.htm) [(live test)](http://wpt.live/css/css-flexbox/align-items-005.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-items-005.htm)
- [align-self-004.html](https://wpt.fyi/results/css/css-flexbox/align-self-004.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-004.html)
- [align-self-005.html](https://wpt.fyi/results/css/css-flexbox/align-self-005.html) [(live test)](http://wpt.live/css/css-flexbox/align-self-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-self-005.html)
- css-flexbox-height-animation-stretch.html (manual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/css-flexbox-height-animation-stretch.html)

![](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/flex-align.svg)

<a id="ref-for-propdef-align-items⑥"></a>

An illustration of the five [align-items](#propdef-align-items) keywords and their effects on a flex container with four colored items.

<a id="ref-for-propdef-align-content④"></a>

### <a id="align-content-property"></a>8.4.  Packing Flex Lines: the [align-content](#propdef-align-content) property

| Field               | Definition                                                                                                                                                                                                                  |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-align-content"></a>align-content                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one②①"></a>flex-start [\|](https://www.w3.org/TR/css-values-4/#comb-one) flex-end <a id="ref-for-comb-one②②"></a>\| center <a id="ref-for-comb-one②③"></a>\| space-between <a id="ref-for-comb-one②④"></a>\| space-around <a id="ref-for-comb-one②⑤"></a>\| stretch |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | stretch                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-flex-container③①"></a><a id="ref-for-multi-line-flex-container⑤"></a>[multi-line](#multi-line-flex-container) [flex containers](#flex-container)                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                    |

<a id="ref-for-propdef-align-content⑤"></a>

<a id="ref-for-cross-axis①②"></a>

<a id="ref-for-propdef-justify-content⑧"></a>

<a id="ref-for-main-axis①③"></a>

<a id="ref-for-single-line-flex-container④"></a>

<a id="ref-for-flex-container③②"></a>

The [align-content](#propdef-align-content) property aligns a flex container’s lines within the flex container when there is extra space in the [cross-axis](#cross-axis), similar to how [justify-content](#propdef-justify-content) aligns individual items within the [main-axis](#main-axis). Note, this property has no effect on a [single-line](#single-line-flex-container) [flex container](#flex-container). Values have the following meanings:

<a id="valdef-align-content-flex-start"></a>flex-start  
<a id="ref-for-cross-start⑨"></a>

Lines are packed toward the start of the flex container. The [cross-start](#cross-start) edge of the first line in the flex container is placed flush with the <a id="ref-for-cross-start①⓪"></a>cross-start edge of the flex container, and each subsequent line is placed flush with the preceding line.

<a id="valdef-align-content-flex-end"></a>flex-end  
<a id="ref-for-cross-end④"></a>

Lines are packed toward the end of the flex container. The [cross-end](#cross-end) edge of the last line is placed flush with the <a id="ref-for-cross-end⑤"></a>cross-end edge of the flex container, and each preceding line is placed flush with the subsequent line.

<a id="valdef-align-content-center"></a>center  
<a id="ref-for-cross-end⑥"></a>

<a id="ref-for-cross-start①①"></a>

Lines are packed toward the center of the flex container. The lines in the flex container are placed flush with each other and aligned in the center of the flex container, with equal amounts of space between the [cross-start](#cross-start) content edge of the flex container and the first line in the flex container, and between the [cross-end](#cross-end) content edge of the flex container and the last line in the flex container. (If the leftover free-space is negative, the lines will overflow equally in both directions.)

<a id="valdef-align-content-space-between"></a>space-between  
<a id="ref-for-cross-end⑦"></a>

<a id="ref-for-cross-start①②"></a>

<a id="ref-for-flex-line①"></a>

Lines are evenly distributed in the flex container. If the leftover free-space is negative or there is only a single [flex line](#flex-line) in the flex container, this value falls back to safe flex-start. Otherwise, the [cross-start](#cross-start) edge of the first line in the flex container is placed flush with the <a id="ref-for-cross-start①③"></a>cross-start content edge of the flex container, the [cross-end](#cross-end) edge of the last line in the flex container is placed flush with the <a id="ref-for-cross-end⑧"></a>cross-end content edge of the flex container, and the remaining lines in the flex container are distributed so that the spacing between any two adjacent lines is the same.

<a id="valdef-align-content-space-around"></a>space-around  
<a id="ref-for-flex-line②"></a>

<a id="ref-for-flex-container③③"></a>

Lines are evenly distributed in the flex container, with half-size spaces on either end. If the leftover free-space is negative this value falls back to safe center. Otherwise, the lines in the flex container are distributed such that the spacing between any two adjacent lines is the same, and the spacing between the first/last lines and the [flex container](#flex-container) edges is half the size of the spacing between [flex lines](#flex-line).

<a id="valdef-align-content-stretch"></a>stretch  
<a id="ref-for-valdef-align-content-flex-start"></a>

Lines stretch to take up the remaining space. If the leftover free-space is negative, this value falls back to [flex-start](#valdef-align-content-flex-start). Otherwise, the free-space is split equally between all of the lines, increasing their cross size.

<a id="ref-for-multi-line-flex-container⑥"></a>

<a id="ref-for-flex-container③④"></a>

<a id="ref-for-cross-axis①③"></a>

<a id="ref-for-single-line-flex-container⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Only [multi-line](#multi-line-flex-container) [flex containers](#flex-container) ever have free space in the [cross-axis](#cross-axis) for lines to be aligned in, because in a [single-line](#single-line-flex-container) flex container the sole line automatically stretches to fill the space.

![](https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/images/align-content-example.svg)

<a id="ref-for-propdef-align-content⑥"></a>

<a id="ref-for-multi-line-flex-container⑦"></a>

An illustration of the [align-content](#propdef-align-content) keywords and their effects on a [multi-line](#multi-line-flex-container) flex container.

Tests

- [align-content_center.html](https://wpt.fyi/results/css/css-flexbox/align-content_center.html) [(live test)](http://wpt.live/css/css-flexbox/align-content_center.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content_center.html)
- [align-content_flex-end.html](https://wpt.fyi/results/css/css-flexbox/align-content_flex-end.html) [(live test)](http://wpt.live/css/css-flexbox/align-content_flex-end.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content_flex-end.html)
- [align-content_flex-start.html](https://wpt.fyi/results/css/css-flexbox/align-content_flex-start.html) [(live test)](http://wpt.live/css/css-flexbox/align-content_flex-start.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content_flex-start.html)
- [align-content-horiz-001a.html](https://wpt.fyi/results/css/css-flexbox/align-content-horiz-001a.html) [(live test)](http://wpt.live/css/css-flexbox/align-content-horiz-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-horiz-001a.html)
- [align-content-horiz-001b.html](https://wpt.fyi/results/css/css-flexbox/align-content-horiz-001b.html) [(live test)](http://wpt.live/css/css-flexbox/align-content-horiz-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-horiz-001b.html)
- [align-content-horiz-002.html](https://wpt.fyi/results/css/css-flexbox/align-content-horiz-002.html) [(live test)](http://wpt.live/css/css-flexbox/align-content-horiz-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-horiz-002.html)
- [align-content_space-around.html](https://wpt.fyi/results/css/css-flexbox/align-content_space-around.html) [(live test)](http://wpt.live/css/css-flexbox/align-content_space-around.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content_space-around.html)
- [align-content_space-between.html](https://wpt.fyi/results/css/css-flexbox/align-content_space-between.html) [(live test)](http://wpt.live/css/css-flexbox/align-content_space-between.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content_space-between.html)
- [align-content_stretch.html](https://wpt.fyi/results/css/css-flexbox/align-content_stretch.html) [(live test)](http://wpt.live/css/css-flexbox/align-content_stretch.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content_stretch.html)
- [align-content-vert-001a.html](https://wpt.fyi/results/css/css-flexbox/align-content-vert-001a.html) [(live test)](http://wpt.live/css/css-flexbox/align-content-vert-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-vert-001a.html)
- [align-content-vert-001b.html](https://wpt.fyi/results/css/css-flexbox/align-content-vert-001b.html) [(live test)](http://wpt.live/css/css-flexbox/align-content-vert-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-vert-001b.html)
- [align-content-vert-002.html](https://wpt.fyi/results/css/css-flexbox/align-content-vert-002.html) [(live test)](http://wpt.live/css/css-flexbox/align-content-vert-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-vert-002.html)
- [align-content-wmvert-001.html](https://wpt.fyi/results/css/css-flexbox/align-content-wmvert-001.html) [(live test)](http://wpt.live/css/css-flexbox/align-content-wmvert-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-wmvert-001.html)
- [align-content-wrap-001.html](https://wpt.fyi/results/css/css-flexbox/align-content-wrap-001.html) [(live test)](http://wpt.live/css/css-flexbox/align-content-wrap-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-wrap-001.html)
- [align-content-wrap-002.html](https://wpt.fyi/results/css/css-flexbox/align-content-wrap-002.html) [(live test)](http://wpt.live/css/css-flexbox/align-content-wrap-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-wrap-002.html)
- [align-content-wrap-003.html](https://wpt.fyi/results/css/css-flexbox/align-content-wrap-003.html) [(live test)](http://wpt.live/css/css-flexbox/align-content-wrap-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-wrap-003.html)
- [align-content-wrap-004.html](https://wpt.fyi/results/css/css-flexbox/align-content-wrap-004.html) [(live test)](http://wpt.live/css/css-flexbox/align-content-wrap-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-wrap-004.html)
- [align-content-wrap-005.html](https://wpt.fyi/results/css/css-flexbox/align-content-wrap-005.html) [(live test)](http://wpt.live/css/css-flexbox/align-content-wrap-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/align-content-wrap-005.html)
- [flex-align-content-center.html](https://wpt.fyi/results/css/css-flexbox/flex-align-content-center.html) [(live test)](http://wpt.live/css/css-flexbox/flex-align-content-center.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-align-content-center.html)
- [flex-align-content-end.html](https://wpt.fyi/results/css/css-flexbox/flex-align-content-end.html) [(live test)](http://wpt.live/css/css-flexbox/flex-align-content-end.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-align-content-end.html)
- [flex-align-content-space-around.html](https://wpt.fyi/results/css/css-flexbox/flex-align-content-space-around.html) [(live test)](http://wpt.live/css/css-flexbox/flex-align-content-space-around.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-align-content-space-around.html)
- [flex-align-content-space-between.html](https://wpt.fyi/results/css/css-flexbox/flex-align-content-space-between.html) [(live test)](http://wpt.live/css/css-flexbox/flex-align-content-space-between.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-align-content-space-between.html)
- [flex-align-content-start.html](https://wpt.fyi/results/css/css-flexbox/flex-align-content-start.html) [(live test)](http://wpt.live/css/css-flexbox/flex-align-content-start.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-align-content-start.html)
- [flexbox_align-content-center.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-content-center.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-content-center.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-content-center.html)
- [flexbox_align-content-flexend.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-content-flexend.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-content-flexend.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-content-flexend.html)
- [flexbox_align-content-flexstart.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-content-flexstart.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-content-flexstart.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-content-flexstart.html)
- [flexbox_align-content-spacearound.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-content-spacearound.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-content-spacearound.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-content-spacearound.html)
- [flexbox_align-content-spacebetween.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-content-spacebetween.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-content-spacebetween.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-content-spacebetween.html)
- [flexbox_align-content-stretch-2.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-content-stretch-2.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-content-stretch-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-content-stretch-2.html)
- [flexbox_align-content-stretch.html](https://wpt.fyi/results/css/css-flexbox/flexbox_align-content-stretch.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_align-content-stretch.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_align-content-stretch.html)
- [flexbox_computedstyle_align-content-center.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-center.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-center.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-center.html)
- [flexbox_computedstyle_align-content-flex-end.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-flex-end.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-flex-end.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-flex-end.html)
- [flexbox_computedstyle_align-content-flex-start.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-flex-start.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-flex-start.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-flex-start.html)
- [flexbox_computedstyle_align-content-space-around.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-space-around.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-space-around.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-space-around.html)
- [flexbox_computedstyle_align-content-space-between.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-space-between.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-space-between.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_align-content-space-between.html)

### <a id="flex-baselines"></a>8.5.  Flex Container Baselines

<a id="ref-for-flex-container③⑤"></a>

<a id="ref-for-baseline-participation"></a>

<a id="ref-for-flex-item⑧④"></a>

<a id="ref-for-propdef-order⑨"></a>

<a id="ref-for-propdef-flex-direction⑦"></a>

In order for a [flex container](#flex-container) to itself [participate in baseline alignment](#baseline-participation) (e.g. when the <a id="ref-for-flex-container③⑥"></a>flex container is itself a [flex item](#flex-item) in an outer <a id="ref-for-flex-container③⑦"></a>flex container), it needs to submit the position of the baselines that will best represent its contents. To this end, the baselines of a flex container are determined as follows (after reordering with [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order), and taking [flex-direction](#propdef-flex-direction) into account):

first/last <a id="main-axis-baseline"></a> main-axis baseline set  
<a id="ref-for-main-axis①④"></a>

<a id="ref-for-flex-container③⑧"></a>

<a id="ref-for-inline-axis①"></a>

When the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) of the [flex container](#flex-container) matches its [main axis](#main-axis), its baselines are determined as follows:

1.  <a id="ref-for-flex-item⑧⑤"></a>

    <a id="ref-for-css-start"></a>

    <a id="ref-for-css-end"></a>

    <a id="ref-for-flex-line③"></a>

    <a id="ref-for-baseline-participation①"></a>

    <a id="ref-for-main-axis-baseline"></a>

    <a id="ref-for-generate-baselines"></a>

    <a id="ref-for-alignment-baseline"></a>

    If any of the [flex items](#flex-item) on the flex container’s [startmost](https://drafts.csswg.org/css-writing-modes-4/#css-start)/[endmost](https://drafts.csswg.org/css-writing-modes-4/#css-end) [flex line](#flex-line) [participate in baseline alignment](#baseline-participation), the flex container’s first/last [main-axis baseline set](#main-axis-baseline) is [generated](https://www.w3.org/TR/css-align-3/#generate-baselines) from the shared [alignment baseline](https://www.w3.org/TR/css-align-3/#alignment-baseline) of those <a id="ref-for-flex-item⑧⑥"></a>flex items.

2.  <a id="ref-for-flex-item⑧⑦"></a>

    <a id="ref-for-main-axis-baseline①"></a>

    <a id="ref-for-generate-baselines①"></a>

    <a id="ref-for-alignment-baseline①"></a>

    <a id="ref-for-css-start①"></a>

    <a id="ref-for-css-end①"></a>

    <a id="ref-for-main-axis①⑤"></a>

    <a id="ref-for-synthesize-baseline①"></a>

    Otherwise, if the flex container has at least one [flex item](#flex-item), the flex container’s first/last [main-axis baseline set](#main-axis-baseline) is [generated](https://www.w3.org/TR/css-align-3/#generate-baselines) from the [alignment baseline](https://www.w3.org/TR/css-align-3/#alignment-baseline) of the [startmost](https://drafts.csswg.org/css-writing-modes-4/#css-start)/[endmost](https://drafts.csswg.org/css-writing-modes-4/#css-end) <a id="ref-for-flex-item⑧⑧"></a>flex item. (If that item has no <a id="ref-for-alignment-baseline②"></a>alignment baseline parallel to the flex container’s [main axis](#main-axis), then one is first [synthesized](https://www.w3.org/TR/css-align-3/#synthesize-baseline) from its border edges.)

3.  <a id="ref-for-baseline-set"></a>

    <a id="ref-for-synthesize-baseline②"></a>

    <a id="ref-for-shared-alignment-context"></a>

    Otherwise, the flex container has no first/last main-axis [baseline set](https://www.w3.org/TR/css-align-3/#baseline-set), and one is [synthesized](https://www.w3.org/TR/css-align-3/#synthesize-baseline) if needed according to the rules of its [alignment context](https://www.w3.org/TR/css-align-3/#shared-alignment-context).

first/last <a id="cross-axis-baseline"></a> cross-axis baseline set  
<a id="ref-for-cross-axis①④"></a>

<a id="ref-for-flex-container③⑨"></a>

<a id="ref-for-inline-axis②"></a>

When the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) of the [flex container](#flex-container) matches its [cross axis](#cross-axis), its baselines are determined as follows:

1.  <a id="ref-for-flex-item⑧⑨"></a>

    <a id="ref-for-cross-axis-baseline"></a>

    <a id="ref-for-generate-baselines②"></a>

    <a id="ref-for-alignment-baseline③"></a>

    <a id="ref-for-css-start②"></a>

    <a id="ref-for-css-end②"></a>

    <a id="ref-for-cross-axis①⑤"></a>

    <a id="ref-for-synthesize-baseline③"></a>

    If the flex container has at least one [flex item](#flex-item), the flex container’s first/last [cross-axis baseline set](#cross-axis-baseline) is [generated](https://www.w3.org/TR/css-align-3/#generate-baselines) from the [alignment baseline](https://www.w3.org/TR/css-align-3/#alignment-baseline) of the [startmost](https://drafts.csswg.org/css-writing-modes-4/#css-start)/[endmost](https://drafts.csswg.org/css-writing-modes-4/#css-end) <a id="ref-for-flex-item⑨⓪"></a>flex item. (If that item has no <a id="ref-for-alignment-baseline④"></a>alignment baseline parallel to the flex container’s [cross axis](#cross-axis), then one is first [synthesized](https://www.w3.org/TR/css-align-3/#synthesize-baseline) from its border edges.)

2.  <a id="ref-for-baseline-set①"></a>

    <a id="ref-for-synthesize-baseline④"></a>

    <a id="ref-for-shared-alignment-context①"></a>

    Otherwise, the flex container has no first/last cross-axis [baseline set](https://www.w3.org/TR/css-align-3/#baseline-set), and one is [synthesized](https://www.w3.org/TR/css-align-3/#synthesize-baseline) if needed according to the rules of its [alignment context](https://www.w3.org/TR/css-align-3/#shared-alignment-context).

<a id="ref-for-propdef-overflow②"></a>

When calculating the baseline according to the above rules, if the box contributing a baseline has an [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) value that allows scrolling, the box must be treated as being in its initial scroll position for the purpose of determining its baseline.

When [determining the baseline of a table cell](https://www.w3.org/TR/CSS2/tables.html#height-layout), a flex container provides a baseline just as a line box or table-row does. [\[CSS2\]](#biblio-css2)

See [CSS Writing Modes 3 § 4.1 Introduction to Baselines](https://www.w3.org/TR/css-writing-modes-3/#intro-baselines) and [CSS Box Alignment 3 § 9 Baseline Alignment Details](https://www.w3.org/TR/css-align-3/#baseline-rules) for more information on baselines.

Tests

- [flex-align-baseline-001.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-001.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-001.html)
- [flex-align-baseline-002.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-002.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-002.html)
- [flex-align-baseline-003.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-003.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-003.html)
- [flex-align-baseline-004.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-004.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-004.html)
- [flex-align-baseline-005.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-005.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-005.html)
- [flex-align-baseline-006.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-006.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-006.html)
- [flex-align-baseline-007.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-007.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-007.html)
- [flex-align-baseline-column-rtl-direction.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-column-rtl-direction.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-column-rtl-direction.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-column-rtl-direction.html)
- [flex-align-baseline-column-vert-lr-rtl-wrap-reverse.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-column-vert-lr-rtl-wrap-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-column-vert-lr-rtl-wrap-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-column-vert-lr-rtl-wrap-reverse.html)
- [flex-align-baseline-column-vert-rl-rtl-wrap-reverse.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-column-vert-rl-rtl-wrap-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-column-vert-rl-rtl-wrap-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-column-vert-rl-rtl-wrap-reverse.html)
- [flex-align-baseline-fieldset-001.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-fieldset-001.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-fieldset-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-fieldset-001.html)
- [flex-align-baseline-fieldset-002.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-fieldset-002.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-fieldset-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-fieldset-002.html)
- [flex-align-baseline-fieldset-003.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-fieldset-003.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-fieldset-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-fieldset-003.html)
- [flex-align-baseline-flex-001.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-flex-001.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-flex-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-flex-001.html)
- [flex-align-baseline-flex-002.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-flex-002.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-flex-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-flex-002.html)
- [flex-align-baseline-flex-003.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-flex-003.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-flex-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-flex-003.html)
- [flex-align-baseline-flex-004.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-flex-004.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-flex-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-flex-004.html)
- [flex-align-baseline-grid-001.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-grid-001.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-grid-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-grid-001.html)
- [flex-align-baseline-grid-002.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-grid-002.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-grid-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-grid-002.html)
- [flex-align-baseline-grid-003.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-grid-003.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-grid-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-grid-003.html)
- [flex-align-baseline-multicol-001.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-multicol-001.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-multicol-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-multicol-001.html)
- [flex-align-baseline-multicol-002.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-multicol-002.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-multicol-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-multicol-002.html)
- [flex-align-baseline-multicol-003.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-multicol-003.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-multicol-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-multicol-003.html)
- [flex-align-baseline-overflow-001.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-overflow-001.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-overflow-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-overflow-001.html)
- [flex-align-baseline-overflow-002.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-overflow-002.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-overflow-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-overflow-002.html)
- [flex-align-baseline-overflow-003.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-overflow-003.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-overflow-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-overflow-003.html)
- [flex-align-baseline-table-001.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-table-001.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-table-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-table-001.html)
- [flex-align-baseline-table-002.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-table-002.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-table-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-table-002.html)
- [flex-align-baseline-table-003.html](https://wpt.fyi/results/css/css-flexbox/alignment/flex-align-baseline-table-003.html) [(live test)](http://wpt.live/css/css-flexbox/alignment/flex-align-baseline-table-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/alignment/flex-align-baseline-table-003.html)
- [flexbox-baseline-align-self-baseline-horiz-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-align-self-baseline-horiz-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-align-self-baseline-horiz-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-align-self-baseline-horiz-001.html)
- [flexbox-baseline-align-self-baseline-vert-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-align-self-baseline-vert-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-align-self-baseline-vert-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-align-self-baseline-vert-001.html)
- [flexbox-baseline-empty-001a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-empty-001a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-empty-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-empty-001a.html)
- [flexbox-baseline-empty-001b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-empty-001b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-empty-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-empty-001b.html)
- [flexbox-baseline-multi-item-horiz-001a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-multi-item-horiz-001a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-multi-item-horiz-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-multi-item-horiz-001a.html)
- [flexbox-baseline-multi-item-horiz-001b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-multi-item-horiz-001b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-multi-item-horiz-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-multi-item-horiz-001b.html)
- [flexbox-baseline-multi-item-vert-001a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-multi-item-vert-001a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-multi-item-vert-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-multi-item-vert-001a.html)
- [flexbox-baseline-multi-item-vert-001b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-multi-item-vert-001b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-multi-item-vert-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-multi-item-vert-001b.html)
- [flexbox-baseline-multi-line-horiz-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-multi-line-horiz-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-multi-line-horiz-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-multi-line-horiz-001.html)
- [flexbox-baseline-multi-line-horiz-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-multi-line-horiz-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-multi-line-horiz-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-multi-line-horiz-002.html)
- [flexbox-baseline-multi-line-horiz-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-multi-line-horiz-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-multi-line-horiz-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-multi-line-horiz-003.html)
- [flexbox-baseline-multi-line-horiz-004.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-multi-line-horiz-004.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-multi-line-horiz-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-multi-line-horiz-004.html)
- [flexbox-baseline-multi-line-vert-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-multi-line-vert-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-multi-line-vert-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-multi-line-vert-001.html)
- [flexbox-baseline-multi-line-vert-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-multi-line-vert-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-multi-line-vert-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-multi-line-vert-002.html)
- [flexbox-baseline-nested-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-nested-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-nested-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-nested-001.html)
- [flexbox-baseline-single-item-001a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-single-item-001a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-single-item-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-single-item-001a.html)
- [flexbox-baseline-single-item-001b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-baseline-single-item-001b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-baseline-single-item-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-baseline-single-item-001b.html)

## <a id="layout-algorithm"></a>9.  Flex Layout Algorithm

This section contains normative algorithms detailing the exact layout behavior of a flex container and its contents. The algorithms here are written to optimize readability and theoretical simplicity, and may not necessarily be the most efficient. Implementations may use whatever actual algorithms they wish, but must produce the same results as the algorithms described here.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This section is mainly intended for implementors. Authors writing web pages should generally be served well by the individual property descriptions, and do not need to read this section unless they have a deep-seated urge to understand arcane details of CSS layout.

The following sections define the algorithm for laying out a flex container and its contents.

<a id="ref-for-flex-item⑨①"></a>

<a id="ref-for-order-modified-document-order①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Flex layout works with the [flex items](#flex-item) in [order-modified document order](https://www.w3.org/TR/css-flexbox-1/#order-modified-document-order), not their original document order.

Tests

- [flexbox-basic-block-horiz-001v.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-block-horiz-001v.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-block-horiz-001v.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-block-horiz-001v.xhtml)
- [flexbox-basic-block-horiz-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-block-horiz-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-block-horiz-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-block-horiz-001.xhtml)
- [flexbox-basic-block-vert-001v.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-block-vert-001v.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-block-vert-001v.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-block-vert-001v.xhtml)
- [flexbox-basic-block-vert-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-block-vert-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-block-vert-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-block-vert-001.xhtml)
- [flexbox-basic-canvas-horiz-001v.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-canvas-horiz-001v.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-canvas-horiz-001v.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-canvas-horiz-001v.xhtml)
- [flexbox-basic-canvas-horiz-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-canvas-horiz-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-canvas-horiz-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-canvas-horiz-001.xhtml)
- [flexbox-basic-canvas-vert-001v.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-canvas-vert-001v.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-canvas-vert-001v.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-canvas-vert-001v.xhtml)
- [flexbox-basic-canvas-vert-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-canvas-vert-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-canvas-vert-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-canvas-vert-001.xhtml)
- [flexbox-basic-fieldset-horiz-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-fieldset-horiz-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-fieldset-horiz-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-fieldset-horiz-001.xhtml)
- [flexbox-basic-fieldset-vert-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-fieldset-vert-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-fieldset-vert-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-fieldset-vert-001.xhtml)
- [flexbox-basic-iframe-horiz-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-iframe-horiz-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-iframe-horiz-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-iframe-horiz-001.xhtml)
- [flexbox-basic-iframe-vert-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-iframe-vert-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-iframe-vert-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-iframe-vert-001.xhtml)
- [flexbox-basic-img-horiz-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-img-horiz-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-img-horiz-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-img-horiz-001.xhtml)
- [flexbox-basic-img-vert-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-img-vert-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-img-vert-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-img-vert-001.xhtml)
- [flexbox-basic-textarea-horiz-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-textarea-horiz-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-textarea-horiz-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-textarea-horiz-001.xhtml)
- [flexbox-basic-textarea-vert-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-textarea-vert-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-textarea-vert-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-textarea-vert-001.xhtml)
- [flexbox-basic-video-horiz-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-video-horiz-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-video-horiz-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-video-horiz-001.xhtml)
- [flexbox-basic-video-vert-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-basic-video-vert-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-basic-video-vert-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-basic-video-vert-001.xhtml)
- [flexbox-dyn-resize-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-dyn-resize-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-dyn-resize-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-dyn-resize-001.html)
- [flexbox-mbp-horiz-001-rtl.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-mbp-horiz-001-rtl.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-mbp-horiz-001-rtl.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-mbp-horiz-001-rtl.xhtml)
- [flexbox-mbp-horiz-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-mbp-horiz-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-mbp-horiz-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-mbp-horiz-001.xhtml)
- [flexbox-mbp-horiz-002a.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-mbp-horiz-002a.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-mbp-horiz-002a.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-mbp-horiz-002a.xhtml)
- [flexbox-mbp-horiz-002b.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-mbp-horiz-002b.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-mbp-horiz-002b.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-mbp-horiz-002b.xhtml)
- [flexbox-mbp-horiz-002v.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-mbp-horiz-002v.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-mbp-horiz-002v.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-mbp-horiz-002v.xhtml)
- [flexbox-mbp-horiz-003-reverse.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-mbp-horiz-003-reverse.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-mbp-horiz-003-reverse.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-mbp-horiz-003-reverse.xhtml)
- [flexbox-mbp-horiz-003v.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-mbp-horiz-003v.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-mbp-horiz-003v.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-mbp-horiz-003v.xhtml)
- [flexbox-mbp-horiz-003.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-mbp-horiz-003.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-mbp-horiz-003.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-mbp-horiz-003.xhtml)
- [flexbox-mbp-horiz-004.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-mbp-horiz-004.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-mbp-horiz-004.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-mbp-horiz-004.xhtml)
- [flexbox-sizing-horiz-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-sizing-horiz-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-sizing-horiz-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-sizing-horiz-001.xhtml)
- [flexbox-sizing-horiz-002.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-sizing-horiz-002.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-sizing-horiz-002.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-sizing-horiz-002.xhtml)
- [flexbox-sizing-vert-001.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-sizing-vert-001.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-sizing-vert-001.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-sizing-vert-001.xhtml)
- [flexbox-sizing-vert-002.xhtml](https://wpt.fyi/results/css/css-flexbox/flexbox-sizing-vert-002.xhtml) [(live test)](http://wpt.live/css/css-flexbox/flexbox-sizing-vert-002.xhtml) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-sizing-vert-002.xhtml)
- [percentage-max-height-001.html](https://wpt.fyi/results/css/css-flexbox/percentage-max-height-001.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-max-height-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-max-height-001.html)
- [percentage-max-height-002.html](https://wpt.fyi/results/css/css-flexbox/percentage-max-height-002.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-max-height-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-max-height-002.html)
- [percentage-max-height-003.html](https://wpt.fyi/results/css/css-flexbox/percentage-max-height-003.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-max-height-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-max-height-003.html)
- [percentage-max-height-004.html](https://wpt.fyi/results/css/css-flexbox/percentage-max-height-004.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-max-height-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-max-height-004.html)
- [percentage-max-height-005.html](https://wpt.fyi/results/css/css-flexbox/percentage-max-height-005.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-max-height-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-max-height-005.html)
- [percent-height-flex-items-cross-sizes-with-mutations.html](https://wpt.fyi/results/css/css-flexbox/percent-height-flex-items-cross-sizes-with-mutations.html) [(live test)](http://wpt.live/css/css-flexbox/percent-height-flex-items-cross-sizes-with-mutations.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percent-height-flex-items-cross-sizes-with-mutations.html)
- [table-as-item-auto-min-width.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-auto-min-width.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-auto-min-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-auto-min-width.html)
- [table-as-item-change-cell.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-change-cell.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-change-cell.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-change-cell.html)
- [table-as-item-fixed-min-width-2.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-fixed-min-width-2.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-fixed-min-width-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-fixed-min-width-2.html)
- [table-as-item-fixed-min-width-3.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-fixed-min-width-3.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-fixed-min-width-3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-fixed-min-width-3.html)
- [table-as-item-fixed-min-width.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-fixed-min-width.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-fixed-min-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-fixed-min-width.html)
- [table-as-item-narrow-content-2.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-narrow-content-2.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-narrow-content-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-narrow-content-2.html)
- [table-as-item-narrow-content.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-narrow-content.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-narrow-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-narrow-content.html)
- [table-as-item-specified-height.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-specified-height.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-specified-height.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-specified-height.html)
- [table-as-item-specified-width.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-specified-width.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-specified-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-specified-width.html)
- [table-as-item-specified-width-vertical.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-specified-width-vertical.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-specified-width-vertical.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-specified-width-vertical.html)

### <a id="box-manip"></a>9.1.  Initial Setup

1.  <a id="algo-anon-box"></a> <strong>Generate anonymous flex items</strong> as described in [§ 4 Flex Items](#flex-items).

### <a id="line-sizing"></a>9.2.  Line Length Determination

1.  <a id="ref-for-max-content-constraint"></a>

    <a id="ref-for-min-content-constraint"></a>

    <a id="ref-for-definite⑦"></a>

    <a id="ref-for-flex-container④⓪"></a>

    <a id="algo-available"></a> <strong>Determine the available main and cross space for the flex items.</strong> For each dimension, if that dimension of the [flex container](#flex-container)’s content box is a [definite size](#definite), use that; if that dimension of the <a id="ref-for-flex-container④①"></a>flex container is being sized under a [min](https://www.w3.org/TR/css-sizing-3/#min-content-constraint) or [max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint), the available space in that dimension is that constraint; otherwise, subtract the <a id="ref-for-flex-container④②"></a>flex container’s margin, border, and padding from the space available to the flex container in that dimension and use that value. <strong data-conversion-semantic="note">Note:</strong> This might result in an infinite value.

    > <strong data-conversion-semantic="example">Example</strong>
    >
    > <a id="example-34f93ad9"></a>
    > <a id="ref-for-available"></a>
    >
    > <a id="ref-for-valdef-width-auto⑤"></a>
    >
    > <a id="ref-for-flex-container④③"></a>
    >
    > For example, the [available space](https://www.w3.org/TR/css-sizing-3/#available) to a flex item in a [floated](https://www.w3.org/TR/CSS2/visuren.html#floats) [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto)-sized [flex container](#flex-container) is:
    >
    > - <a id="ref-for-flex-container④④"></a>
    >
    >   the width of the [flex container](#flex-container)’s containing block minus the <a id="ref-for-flex-container④⑤"></a>flex container’s margin, border, and padding in the horizontal dimension
    >
    > - infinite in the vertical dimension

2.  <a id="algo-main-item"></a> <strong>Determine the <dfn><span><a id="flex-base-size"></a></span>flex base size</dfn> and <dfn><span><a id="hypothetical-main-size"></a></span>hypothetical main size</dfn> of each item:</strong>

    Tests
    - [image-as-flexitem-size-001.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-001.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-001.html)
    - [image-as-flexitem-size-001v.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-001v.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-001v.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-001v.html)
    - [image-as-flexitem-size-002.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-002.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-002.html)
    - [image-as-flexitem-size-002v.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-002v.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-002v.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-002v.html)
    - [image-as-flexitem-size-003.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-003.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-003.html)
    - [image-as-flexitem-size-003v.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-003v.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-003v.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-003v.html)
    - [image-as-flexitem-size-004.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-004.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-004.html)
    - [image-as-flexitem-size-004v.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-004v.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-004v.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-004v.html)
    - [image-as-flexitem-size-005.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-005.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-005.html)
    - [image-as-flexitem-size-005v.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-005v.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-005v.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-005v.html)
    - [image-as-flexitem-size-006.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-006.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-006.html)
    - [image-as-flexitem-size-006v.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-006v.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-006v.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-006v.html)
    - [image-as-flexitem-size-007.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-007.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-007.html)
    - [image-as-flexitem-size-007v.html](https://wpt.fyi/results/css/css-flexbox/image-as-flexitem-size-007v.html) [(live test)](http://wpt.live/css/css-flexbox/image-as-flexitem-size-007v.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/image-as-flexitem-size-007v.html)
    - [item-with-table-with-infinite-max-intrinsic-width.html](https://wpt.fyi/results/css/css-flexbox/item-with-table-with-infinite-max-intrinsic-width.html) [(live test)](http://wpt.live/css/css-flexbox/item-with-table-with-infinite-max-intrinsic-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/item-with-table-with-infinite-max-intrinsic-width.html)
    - [multiline-min-max.html](https://wpt.fyi/results/css/css-flexbox/multiline-min-max.html) [(live test)](http://wpt.live/css/css-flexbox/multiline-min-max.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/multiline-min-max.html)
    - [orthogonal-writing-modes-and-intrinsic-sizing.html](https://wpt.fyi/results/css/css-flexbox/orthogonal-writing-modes-and-intrinsic-sizing.html) [(live test)](http://wpt.live/css/css-flexbox/orthogonal-writing-modes-and-intrinsic-sizing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/orthogonal-writing-modes-and-intrinsic-sizing.html)
    - [svg-root-as-flex-item-001.html](https://wpt.fyi/results/css/css-flexbox/svg-root-as-flex-item-001.html) [(live test)](http://wpt.live/css/css-flexbox/svg-root-as-flex-item-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/svg-root-as-flex-item-001.html)
    - [svg-root-as-flex-item-002.html](https://wpt.fyi/results/css/css-flexbox/svg-root-as-flex-item-002.html) [(live test)](http://wpt.live/css/css-flexbox/svg-root-as-flex-item-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/svg-root-as-flex-item-002.html)
    - [svg-root-as-flex-item-003.html](https://wpt.fyi/results/css/css-flexbox/svg-root-as-flex-item-003.html) [(live test)](http://wpt.live/css/css-flexbox/svg-root-as-flex-item-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/svg-root-as-flex-item-003.html)
    - [svg-root-as-flex-item-004.html](https://wpt.fyi/results/css/css-flexbox/svg-root-as-flex-item-004.html) [(live test)](http://wpt.live/css/css-flexbox/svg-root-as-flex-item-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/svg-root-as-flex-item-004.html)
    - [svg-root-as-flex-item-005.html](https://wpt.fyi/results/css/css-flexbox/svg-root-as-flex-item-005.html) [(live test)](http://wpt.live/css/css-flexbox/svg-root-as-flex-item-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/svg-root-as-flex-item-005.html)
    - [svg-root-as-flex-item-006.html](https://wpt.fyi/results/css/css-flexbox/svg-root-as-flex-item-006.html) [(live test)](http://wpt.live/css/css-flexbox/svg-root-as-flex-item-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/svg-root-as-flex-item-006.html)
    - [table-as-item-min-height-1.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-min-height-1.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-min-height-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-min-height-1.html)
    - [table-as-item-wide-content.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-wide-content.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-wide-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-wide-content.html)
    - [table-with-infinite-max-intrinsic-width.html](https://wpt.fyi/results/css/css-flexbox/table-with-infinite-max-intrinsic-width.html) [(live test)](http://wpt.live/css/css-flexbox/table-with-infinite-max-intrinsic-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-with-infinite-max-intrinsic-width.html)

    1.  <a id="ref-for-flex-base-size①"></a>

        <a id="ref-for-flex-flex-basis③"></a>

        <a id="ref-for-definite⑧"></a>

        If the item has a [definite](#definite) used [flex basis](#flex-flex-basis), that’s the [flex base size](#flex-base-size).

    2.  <a id="ref-for-flex-item⑨②"></a>

        <a id="ref-for-cross-size①①"></a>

        <a id="ref-for-used-value"></a>

        <a id="ref-for-flex-base-size②"></a>

        If the flex item has ...

        - <a id="ref-for-preferred-aspect-ratio③"></a>

          a [preferred aspect ratio](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio),

        - <a id="ref-for-valdef-flex-basis-content⑥"></a>

          <a id="ref-for-flex-flex-basis④"></a>

          a used [flex basis](#flex-flex-basis) of [content](#valdef-flex-basis-content), and

        - <a id="ref-for-cross-size①⓪"></a>

          <a id="ref-for-definite⑨"></a>

          a [definite](#definite) [cross size](#cross-size),

        then the [flex base size](#flex-base-size) is calculated from its [used](https://www.w3.org/TR/css-cascade-5/#used-value) [cross size](#cross-size) and the [flex item](#flex-item)’s aspect ratio.

    3.  <a id="ref-for-main-size①⓪"></a>

        <a id="ref-for-flex-base-size③"></a>

        <a id="ref-for-available①"></a>

        <a id="ref-for-valdef-flex-basis-content⑦"></a>

        <a id="ref-for-flex-flex-basis⑤"></a>

        If the used [flex basis](#flex-flex-basis) is [content](#valdef-flex-basis-content) or depends on its [available space](https://www.w3.org/TR/css-sizing-3/#available), and the flex container is being sized under a min-content or max-content constraint (e.g. when performing [automatic table layout](https://www.w3.org/TR/CSS2/tables.html#auto-table-layout) [\[CSS2\]](#biblio-css2)), size the item under that constraint. The [flex base size](#flex-base-size) is the item’s resulting [main size](#main-size).

    4.  <a id="ref-for-main-size①①"></a>

        <a id="ref-for-flex-base-size④"></a>

        <a id="ref-for-available②"></a>

        <a id="ref-for-valdef-flex-basis-content⑧"></a>

        <a id="ref-for-flex-flex-basis⑥"></a>

        Otherwise, if the used [flex basis](#flex-flex-basis) is [content](#valdef-flex-basis-content) or depends on its [available space](https://www.w3.org/TR/css-sizing-3/#available), the available main size is infinite, and the flex item’s inline axis is parallel to the main axis, lay the item out using [the rules for a box in an orthogonal flow](https://www.w3.org/TR/css3-writing-modes/#orthogonal-flows) [\[CSS3-WRITING-MODES\]](#biblio-css3-writing-modes). The [flex base size](#flex-base-size) is the item’s max-content [main size](#main-size).

        <a id="ref-for-writing-mode⑧"></a>

        <a id="ref-for-flex-item⑨③"></a>

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: This case occurs, for example, in an English document (horizontal [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode)) containing a column flex container containing a vertical Japanese (vertical <a id="ref-for-writing-mode⑨"></a>writing mode) [flex item](#flex-item).

    5.  <a id="ref-for-flex-base-size⑤"></a>

        <a id="ref-for-definite①⓪"></a>

        <a id="ref-for-valdef-align-items-auto①"></a>

        <a id="ref-for-preferred-aspect-ratio④"></a>

        <a id="ref-for-flex-item⑨④"></a>

        <a id="ref-for-cross-size①②"></a>

        <a id="ref-for-valdef-width-max-content"></a>

        <a id="ref-for-valdef-flex-basis-content⑨"></a>

        <a id="ref-for-main-size①②"></a>

        <a id="ref-for-flex-flex-basis⑦"></a>

        <a id="ref-for-available③"></a>

        Otherwise, size the item into the [available space](https://www.w3.org/TR/css-sizing-3/#available) using its used [flex basis](#flex-flex-basis) in place of its [main size](#main-size), treating a value of [content](#valdef-flex-basis-content) as [max-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-max-content). If a [cross size](#cross-size) is needed to determine the <a id="ref-for-main-size①③"></a>main size (e.g. when the [flex item](#flex-item)’s <a id="ref-for-main-size①④"></a>main size is in its block axis, or when it has a [preferred aspect ratio](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio)) and the <a id="ref-for-flex-item⑨⑤"></a>flex item’s cross size is [auto](#valdef-align-items-auto) and not [definite](#definite), in this calculation use fit-content as the <a id="ref-for-flex-item⑨⑥"></a>flex item’s <a id="ref-for-cross-size①③"></a>cross size. The [flex base size](#flex-base-size) is the item’s resulting <a id="ref-for-main-size①⑤"></a>main size.

        Tests
        - [css-flexbox-img-expand-evenly.html](https://wpt.fyi/results/css/css-flexbox/css-flexbox-img-expand-evenly.html) [(live test)](http://wpt.live/css/css-flexbox/css-flexbox-img-expand-evenly.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/css-flexbox-img-expand-evenly.html)
        - [fit-content-item-001.html](https://wpt.fyi/results/css/css-flexbox/fit-content-item-001.html) [(live test)](http://wpt.live/css/css-flexbox/fit-content-item-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/fit-content-item-001.html)
        - [fit-content-item-002.html](https://wpt.fyi/results/css/css-flexbox/fit-content-item-002.html) [(live test)](http://wpt.live/css/css-flexbox/fit-content-item-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/fit-content-item-002.html)
        - [fit-content-item-003.html](https://wpt.fyi/results/css/css-flexbox/fit-content-item-003.html) [(live test)](http://wpt.live/css/css-flexbox/fit-content-item-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/fit-content-item-003.html)
        - [fit-content-item-004.html](https://wpt.fyi/results/css/css-flexbox/fit-content-item-004.html) [(live test)](http://wpt.live/css/css-flexbox/fit-content-item-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/fit-content-item-004.html)
        - [flexbox-vert-lr-with-img.html](https://wpt.fyi/results/css/css-flexbox/flexbox-vert-lr-with-img.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-vert-lr-with-img.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-vert-lr-with-img.html)
        - [flex-container-max-content-001.html](https://wpt.fyi/results/css/css-flexbox/flex-container-max-content-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-container-max-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-container-max-content-001.html)
        - [flex-container-min-content-001.html](https://wpt.fyi/results/css/css-flexbox/flex-container-min-content-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-container-min-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-container-min-content-001.html)
        - [flex-height-min-content.html](https://wpt.fyi/results/css/css-flexbox/flex-height-min-content.html) [(live test)](http://wpt.live/css/css-flexbox/flex-height-min-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-height-min-content.html)

    <a id="ref-for-flex-base-size⑥"></a>

    <a id="ref-for-main-size①⑥"></a>

    <a id="ref-for-propdef-box-sizing①"></a>

    When determining the [flex base size](#flex-base-size), the item’s min and max [main sizes](#main-size) are ignored (no clamping occurs). Furthermore, the sizing calculations that floor the content box size at zero when applying [box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing) are also ignored. (For example, an item with a specified size of zero, positive padding, and <a id="ref-for-propdef-box-sizing②"></a>box-sizing: border-box will have an outer <a id="ref-for-flex-base-size⑦"></a>flex base size of zero—​and hence a negative inner <a id="ref-for-flex-base-size⑧"></a>flex base size.)

    <a id="ref-for-hypothetical-main-size"></a>

    <a id="ref-for-flex-base-size⑨"></a>

    <a id="ref-for-used-value①"></a>

    <a id="ref-for-main-size①⑦"></a>

    The [hypothetical main size](#hypothetical-main-size) is the item’s [flex base size](#flex-base-size) clamped according to its [used](https://www.w3.org/TR/css-cascade-5/#used-value) min and max [main sizes](#main-size) (and flooring the content box size at zero).

    Tests
    - [text-as-flexitem-size-001.html](https://wpt.fyi/results/css/css-flexbox/text-as-flexitem-size-001.html) [(live test)](http://wpt.live/css/css-flexbox/text-as-flexitem-size-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/text-as-flexitem-size-001.html)

3.  <a id="ref-for-main-size①⑧"></a>

    <a id="algo-main-container"></a> <strong>Determine the <a href="#main-size">main size</a> of the flex container</strong> using the rules of the formatting context in which it participates.

    <a id="ref-for-automatic-block-size"></a>

    <a id="ref-for-block-level①"></a>

    <a id="ref-for-flex-container④⑥"></a>

    <a id="ref-for-max-content①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > The [automatic block size](https://www.w3.org/TR/css-sizing-3/#automatic-block-size) of a [block-level](https://www.w3.org/TR/css-display-4/#block-level) [flex container](#flex-container) is its [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content).
    > The Block Layout spec should define this equivalency, but it doesn’t exist yet.

### <a id="main-sizing"></a>9.3.  Main Size Determination

1.  <a id="algo-line-break"></a> <strong>Collect flex items into flex lines:</strong>
    - <a id="ref-for-single-line-flex-container⑥"></a>

      If the flex container is [single-line](#single-line-flex-container), collect all the flex items into a single flex line.

    - <a id="ref-for-inner-size"></a>

      Otherwise, starting from the first uncollected item, collect consecutive items one by one until the first time that the <em>next</em> collected item would not fit into the flex container’s [inner](https://www.w3.org/TR/css-sizing-3/#inner-size) main size (or until a forced break is encountered, see [§ 10 Fragmenting Flex Layout](#pagination)). If the very first uncollected item wouldn’t fit, collect just it into the line.

      <a id="ref-for-outer-size"></a>

      <a id="ref-for-hypothetical-main-size①"></a>

      For this step, the size of a flex item is its [outer](https://www.w3.org/TR/css-sizing-3/#outer-size) [hypothetical main size](#hypothetical-main-size). <strong data-conversion-semantic="note">Note:</strong> (Note: This can be negative.)

      Repeat until all flex items have been collected into flex lines.

      > <strong data-conversion-semantic="note">Note</strong>
      >
      > Note that the "collect as many" line will collect zero-sized flex items onto the end of the previous line even if the last non-zero item exactly "filled up" the line.

2.  <a id="ref-for-main-size①⑨"></a>

    <a id="algo-flex"></a> <strong><a href="#resolve-flexible-lengths">Resolve the flexible lengths</a></strong> of all the flex items to find their used [main size](#main-size). See [§ 9.7 Resolving Flexible Lengths](#resolve-flexible-lengths).

Tests

- [flexbox_quirks_body.html](https://wpt.fyi/results/css/css-flexbox/flexbox_quirks_body.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_quirks_body.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_quirks_body.html)
- [ortho-table-item-001.html](https://wpt.fyi/results/css/css-flexbox/ortho-table-item-001.html) [(live test)](http://wpt.live/css/css-flexbox/ortho-table-item-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/ortho-table-item-001.html)
- [percentage-heights-000.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-000.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-000.html)
- [percentage-heights-001.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-001.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-001.html)
- [percentage-heights-002.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-002.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-002.html)
- [percentage-heights-003.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-003.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-003.html)
- [percentage-heights-004.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-004.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-004.html)
- [percentage-heights-005.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-005.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-005.html)
- [percentage-heights-006.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-006.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-006.html)
- [percentage-heights-007.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-007.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-007.html)
- [percentage-heights-008.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-008.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-008.html)
- [percentage-heights-009.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-009.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-009.html)
- [percentage-heights-010.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-010.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-010.html)
- [percentage-heights-011.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-011.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-011.html)
- [percentage-heights-012.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-012.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-012.html)
- [percentage-heights-013.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-013.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-013.html)
- [percentage-heights-014.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-014.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-014.html)
- [percentage-heights-015.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-015.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-015.html)
- [percentage-heights-016.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-016.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-016.html)
- [percentage-heights-017.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-017.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-017.html)
- [percentage-heights-018.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-018.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-018.html)
- [percentage-heights-019.html](https://wpt.fyi/results/css/css-flexbox/percentage-heights-019.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-heights-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-heights-019.html)

### <a id="cross-sizing"></a>9.4.  Cross Size Determination

1.  <a id="ref-for-valdef-width-auto⑥"></a>

    <a id="ref-for-main-size②⓪"></a>

    <a id="ref-for-block-level-box"></a>

    <a id="ref-for-in-flow"></a>

    <a id="algo-cross-item"></a> <strong>Determine the <dfn><span><a id="hypothetical-cross-size"></a></span>hypothetical cross size</dfn> of each item</strong> by performing layout as if it were an [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) [block-level box](https://www.w3.org/TR/css-display-4/#block-level-box) with the used [main size](#main-size) and the given available space, treating [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) as fit-content.

    Tests
    - [canvas-contain-size.html](https://wpt.fyi/results/css/css-flexbox/canvas-contain-size.html) [(live test)](http://wpt.live/css/css-flexbox/canvas-contain-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/canvas-contain-size.html)

2.  <a id="algo-cross-line"></a> <strong>Calculate the cross size of each flex line.</strong>

    <a id="ref-for-single-line-flex-container⑦"></a>

    <a id="ref-for-definite①①"></a>

    <a id="ref-for-cross-size①④"></a>

    <a id="ref-for-flex-line④"></a>

    <a id="ref-for-flex-container④⑦"></a>

    If the flex container is [single-line](#single-line-flex-container) and has a [definite](#definite) [cross size](#cross-size), the <a id="ref-for-cross-size①⑤"></a>cross size of the [flex line](#flex-line) is the [flex container](#flex-container)’s inner <a id="ref-for-cross-size①⑥"></a>cross size.

    Otherwise, for each flex line:

    1.  <a id="ref-for-valdef-align-items-baseline"></a>

        <a id="ref-for-propdef-align-self①⑤"></a>

        Collect all the flex items whose inline-axis is parallel to the main-axis, whose [align-self](#propdef-align-self) is [baseline](#valdef-align-items-baseline), and whose cross-axis margins are both non-auto. Find the largest of the distances between each item’s baseline and its hypothetical outer cross-start edge, and the largest of the distances between each item’s baseline and its hypothetical outer cross-end edge, and sum these two values.

    2.  <a id="ref-for-hypothetical-cross-size"></a>

        Among all the items not collected by the previous step, find the largest outer [hypothetical cross size](#hypothetical-cross-size).

    3.  <a id="ref-for-flex-line⑤"></a>

        The used cross-size of the [flex line](#flex-line) is the largest of the numbers found in the previous two steps and zero.

        <a id="ref-for-single-line-flex-container⑧"></a>

        <a id="ref-for-cross-size①⑦"></a>

        If the flex container is [single-line](#single-line-flex-container), then clamp the line’s cross-size to be within the container’s computed min and max [cross sizes](#cross-size). <strong data-conversion-semantic="note">Note:</strong> Note that if CSS 2.1’s definition of min/max-width/height applied more generally, this behavior would fall out automatically.

    Tests
    - [flexbox-single-line-clamp-1.html](https://wpt.fyi/results/css/css-flexbox/flexbox-single-line-clamp-1.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-single-line-clamp-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-single-line-clamp-1.html)
    - [flexbox-single-line-clamp-2.html](https://wpt.fyi/results/css/css-flexbox/flexbox-single-line-clamp-2.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-single-line-clamp-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-single-line-clamp-2.html)
    - [flexbox-single-line-clamp-3.html](https://wpt.fyi/results/css/css-flexbox/flexbox-single-line-clamp-3.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-single-line-clamp-3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-single-line-clamp-3.html)
    - [flex-item-contains-size-layout-001.html](https://wpt.fyi/results/css/css-flexbox/flex-item-contains-size-layout-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-contains-size-layout-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-contains-size-layout-001.html)
    - [flex-item-contains-strict.html](https://wpt.fyi/results/css/css-flexbox/flex-item-contains-strict.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-contains-strict.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-contains-strict.html)
    - [layout-algorithm_algo-cross-line-001.html](https://wpt.fyi/results/css/css-flexbox/layout-algorithm_algo-cross-line-001.html) [(live test)](http://wpt.live/css/css-flexbox/layout-algorithm_algo-cross-line-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/layout-algorithm_algo-cross-line-001.html)
    - [layout-algorithm_algo-cross-line-002.html](https://wpt.fyi/results/css/css-flexbox/layout-algorithm_algo-cross-line-002.html) [(live test)](http://wpt.live/css/css-flexbox/layout-algorithm_algo-cross-line-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/layout-algorithm_algo-cross-line-002.html)

3.  <a id="ref-for-valdef-align-content-stretch"></a>

    <a id="ref-for-propdef-align-content⑦"></a>

    <a id="ref-for-definite①②"></a>

    <a id="algo-line-stretch"></a> <strong>Handle 'align-content: stretch'.</strong> If the flex container has a [definite](#definite) cross size, [align-content](#propdef-align-content) is [stretch](#valdef-align-content-stretch), and the sum of the flex lines' cross sizes is less than the flex container’s inner cross size, increase the cross size of each flex line by equal amounts such that the sum of their cross sizes exactly equals the flex container’s inner cross size.

4.  <a id="ref-for-propdef-visibility③"></a>

    <a id="algo-visibility"></a> <strong>Collapse <span>visibility:collapse</span> items.</strong> If any flex items have [visibility: collapse](https://www.w3.org/TR/css-display-4/#propdef-visibility), note the cross size of the line they’re in as the item’s <var>strut size</var>, and restart layout from the beginning.

    <a id="ref-for-main-size②①"></a>

    In this second layout round, when [collecting items into lines](#algo-line-break), treat the collapsed items as having zero [main size](#main-size). For the rest of the algorithm following that step, ignore the collapsed items entirely (as if they were display:none) except that after [calculating the cross size of the lines](#algo-cross-line), if any line’s cross size is less than the largest <var>strut size</var> among all the collapsed items in the line, set its cross size to that <var>strut size</var>.

    Skip this step in the second layout round.

    Tests
    - [flexbox-collapsed-item-baseline-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-collapsed-item-baseline-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-collapsed-item-baseline-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-collapsed-item-baseline-001.html)
    - [flexbox-collapsed-item-horiz-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-collapsed-item-horiz-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-collapsed-item-horiz-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-collapsed-item-horiz-001.html)
    - [flexbox-collapsed-item-horiz-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-collapsed-item-horiz-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-collapsed-item-horiz-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-collapsed-item-horiz-002.html)
    - [flexbox-collapsed-item-horiz-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-collapsed-item-horiz-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-collapsed-item-horiz-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-collapsed-item-horiz-003.html)

5.  <a id="ref-for-hypothetical-cross-size①"></a>

    <a id="ref-for-cross-size①⑧"></a>

    <a id="ref-for-valdef-width-auto⑦"></a>

    <a id="ref-for-propdef-align-self①⑥"></a>

    <a id="algo-stretch"></a> <strong>Determine the used cross size of each flex item.</strong> If a flex item has [align-self: stretch](#propdef-align-self), its computed cross size property is [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), and neither of its cross-axis margins are auto, the used outer cross size is the used cross size of its flex line, clamped according to the item’s used min and max [cross sizes](#cross-size). Otherwise, the used cross size is the item’s [hypothetical cross size](#hypothetical-cross-size).

    <a id="ref-for-propdef-align-self①⑦"></a>

    If the flex item has [align-self: stretch](#propdef-align-self), redo layout for its contents, treating this used size as its definite cross size so that percentage-sized children can be resolved.

    Tests
    - [table-as-item-cross-size.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-cross-size.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-cross-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-cross-size.html)

    <a id="ref-for-main-size②②"></a>

    <a id="ref-for-preferred-aspect-ratio⑤"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note that this step does not affect the [main size](#main-size) of the flex item, even if it has a [preferred aspect ratio](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio).

Tests

- [flex-item-and-percentage-abspos.html](https://wpt.fyi/results/css/css-flexbox/flex-item-and-percentage-abspos.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-and-percentage-abspos.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-and-percentage-abspos.html)

### <a id="main-alignment"></a>9.5.  Main-Axis Alignment

1.  <a id="algo-main-align"></a> <strong>Distribute any remaining free space.</strong> For each flex line:
    1.  If the remaining free space is positive and at least one main-axis margin on this line is auto, distribute the free space equally among these margins. Otherwise, set all auto margins to zero.

    2.  <a id="ref-for-propdef-justify-content⑨"></a>

        Align the items along the main-axis per [justify-content](#propdef-justify-content).

### <a id="cross-alignment"></a>9.6.  Cross-Axis Alignment

1.  <a id="algo-cross-margins"></a> <strong>Resolve cross-axis <a>auto</a> margins.</strong> If a flex item has auto cross-axis margins:
    - If its outer cross size (treating those auto margins as zero) is less than the cross size of its flex line, distribute the difference in those sizes equally to the auto margins.

    - <a id="ref-for-inline-start②"></a>

      <a id="ref-for-block-start②"></a>

      Otherwise, if the [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) or [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) margin (whichever is in the cross axis) is auto, set it to zero. Set the opposite margin so that the outer cross size of the item equals the cross size of its flex line.

2.  <a id="ref-for-propdef-align-self①⑧"></a>

    <a id="algo-cross-align"></a> <strong>Align all flex items along the cross-axis</strong> per [align-self](#propdef-align-self), if neither of the item’s cross-axis margins are auto.

3.  <a id="ref-for-flex-line⑥"></a>

    <a id="ref-for-cross-size①⑨"></a>

    <a id="ref-for-formatting-context"></a>

    <a id="algo-cross-container"></a> <strong>Determine the flex container’s used cross size</strong> using the rules of the [formatting context](https://www.w3.org/TR/css-display-4/#formatting-context) in which it participates. If a content-based [cross size](#cross-size) is needed, use the sum of the [flex lines](#flex-line)' <a id="ref-for-cross-size②⓪"></a>cross sizes.

4.  <a id="ref-for-propdef-align-content⑧"></a>

    <a id="algo-line-align"></a> <strong>Align all flex lines</strong> per [align-content](#propdef-align-content).

### <a id="resolve-flexible-lengths"></a>9.7.  Resolving Flexible Lengths

To resolve the flexible lengths of the items within a flex line:

1.  <a id="ref-for-main-size②③"></a>

    <strong>Determine the used flex factor</strong>. Sum the outer hypothetical main sizes of all items on the line. If the sum is less than the flex container’s inner [main size](#main-size), use the flex grow factor for the rest of this algorithm; otherwise, use the flex shrink factor.

2.  <a id="ref-for-flex-base-size①⓪"></a>

    Each item in the flex line has a <a id="target-main-size"></a>target main size, initially set to its [flex base size](#flex-base-size). Each item is initially <i>unfrozen</i> and may become <i>frozen</i>.

    <a id="ref-for-target-main-size"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: An item’s [target main size](#target-main-size) doesn’t change after freezing.

3.  <a id="ref-for-hypothetical-main-size②"></a>

    <a id="ref-for-target-main-size①"></a>

    <strong>Size inflexible items.</strong> Freeze, setting its [target main size](#target-main-size) to its [hypothetical main size](#hypothetical-main-size)…

    - any item that has a flex factor of zero

    - <a id="ref-for-hypothetical-main-size③"></a>

      <a id="ref-for-flex-base-size①①"></a>

      <a id="ref-for-flex-flex-grow-factor③"></a>

      if using the [flex grow factor](#flex-flex-grow-factor): any item that has a [flex base size](#flex-base-size) greater than its [hypothetical main size](#hypothetical-main-size)

    - <a id="ref-for-hypothetical-main-size④"></a>

      <a id="ref-for-flex-base-size①②"></a>

      <a id="ref-for-flex-flex-shrink-factor④"></a>

      if using the [flex shrink factor](#flex-flex-shrink-factor): any item that has a [flex base size](#flex-base-size) smaller than its [hypothetical main size](#hypothetical-main-size)

4.  <a id="ref-for-flex-base-size①③"></a>

    <a id="ref-for-target-main-size②"></a>

    <a id="ref-for-main-size②④"></a>

    <strong>Calculate <dfn><span><a id="initial-free-space"></a></span>initial free space</dfn>.</strong> Sum the outer sizes of all items on the line, and subtract this from the flex container’s inner [main size](#main-size). For frozen items, use their outer [target main size](#target-main-size); for other items, use their outer [flex base size](#flex-base-size).

5.  Loop:
    1.  <strong>Check for flexible items.</strong> If all the flex items on the line are frozen, exit this loop.

    2.  <a id="ref-for-remaining-free-space"></a>

        <a id="ref-for-initial-free-space"></a>

        <strong>Calculate the <dfn><span><a id="remaining-free-space"></a></span>remaining free space</dfn></strong> as for [initial free space](#initial-free-space), above. If the sum of the unfrozen flex items’ flex factors is less than one, multiply the <a id="ref-for-initial-free-space①"></a>initial free space by this sum. If the magnitude of this value is less than the magnitude of the [remaining free space](#remaining-free-space), use this as the <a id="ref-for-remaining-free-space①"></a>remaining free space.

    3.  <a id="ref-for-remaining-free-space②"></a>

        If the [remaining free space](#remaining-free-space) is non-zero, <strong>distribute it proportional to the flex factors</strong>:

        <a id="ref-for-flex-flex-grow-factor④"></a>

        If using the [flex grow factor](#flex-flex-grow-factor)

        <a id="ref-for-remaining-free-space③"></a>

        <a id="ref-for-flex-base-size①④"></a>

        <a id="ref-for-target-main-size③"></a>

        For every unfrozen item on the line, find the ratio of the item’s flex grow factor to the sum of the flex grow factors of all unfrozen items on the line. Set the item’s [target main size](#target-main-size) to its [flex base size](#flex-base-size) plus a fraction of the [remaining free space](#remaining-free-space) proportional to the ratio.

        <a id="ref-for-flex-flex-shrink-factor⑤"></a>

        If using the [flex shrink factor](#flex-flex-shrink-factor)

        <a id="ref-for-main-size②⑤"></a>

        <a id="ref-for-remaining-free-space④"></a>

        <a id="ref-for-target-main-size④"></a>

        <a id="ref-for-scaled-flex-shrink-factor"></a>

        <a id="ref-for-flex-base-size①⑤"></a>

        For every unfrozen item on the line, multiply its flex shrink factor by its inner [flex base size](#flex-base-size), and note this as its <a id="scaled-flex-shrink-factor"></a>scaled flex shrink factor. Find the ratio of the item’s [scaled flex shrink factor](#scaled-flex-shrink-factor) to the sum of the <a id="ref-for-scaled-flex-shrink-factor①"></a>scaled flex shrink factors of all unfrozen items on the line. Set the item’s [target main size](#target-main-size) to its <a id="ref-for-flex-base-size①⑥"></a>flex base size minus a fraction of the absolute value of the [remaining free space](#remaining-free-space) proportional to the ratio. <strong data-conversion-semantic="note">Note:</strong> Note this may result in a negative inner [main size](#main-size); it will be corrected in the next step.

    4.  <a id="ref-for-main-size②⑥"></a>

        <a id="ref-for-used-value②"></a>

        <a id="ref-for-target-main-size⑤"></a>

        <strong>Fix min/max violations.</strong> Clamp each non-frozen item’s [target main size](#target-main-size) by its [used](https://www.w3.org/TR/css-cascade-5/#used-value) min and max [main sizes](#main-size) and floor its content-box size at zero. If the item’s <a id="ref-for-target-main-size⑥"></a>target main size was made smaller by this, it’s a max violation. If the item’s <a id="ref-for-target-main-size⑦"></a>target main size was made larger by this, it’s a min violation.

    5.  <strong>Freeze over-flexed items.</strong> The total violation is the sum of the adjustments from the previous step `∑(clamped size - unclamped size)`. If the total violation is:
        Zero  
        Freeze all items.

        Positive  
        Freeze all the items with min violations.

        Negative  
        Freeze all the items with max violations.

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: This freezes at least one item, ensuring that the loop makes progress and eventually terminates.

    6.  Return to the start of this loop.

6.  <a id="ref-for-target-main-size⑧"></a>

    <a id="ref-for-main-size②⑦"></a>

    Set each item’s used [main size](#main-size) to its [target main size](#target-main-size).

Tests

- [item-with-max-height-and-scrollbar.html](https://wpt.fyi/results/css/css-flexbox/item-with-max-height-and-scrollbar.html) [(live test)](http://wpt.live/css/css-flexbox/item-with-max-height-and-scrollbar.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/item-with-max-height-and-scrollbar.html)
- [item-with-max-height-and-scrollbar.html](https://wpt.fyi/results/css/css-flexbox/item-with-max-height-and-scrollbar.html) [(live test)](http://wpt.live/css/css-flexbox/item-with-max-height-and-scrollbar.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/item-with-max-height-and-scrollbar.html)
- [max-width-violation.html](https://wpt.fyi/results/css/css-flexbox/max-width-violation.html) [(live test)](http://wpt.live/css/css-flexbox/max-width-violation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/max-width-violation.html)
- [relayout-image-load.html](https://wpt.fyi/results/css/css-flexbox/relayout-image-load.html) [(live test)](http://wpt.live/css/css-flexbox/relayout-image-load.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/relayout-image-load.html)
- [table-as-item-inflexible-in-column-1.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-inflexible-in-column-1.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-inflexible-in-column-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-inflexible-in-column-1.html)
- [table-as-item-inflexible-in-column-2.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-inflexible-in-column-2.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-inflexible-in-column-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-inflexible-in-column-2.html)
- [table-as-item-inflexible-in-row-1.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-inflexible-in-row-1.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-inflexible-in-row-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-inflexible-in-row-1.html)
- [table-as-item-inflexible-in-row-2.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-inflexible-in-row-2.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-inflexible-in-row-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-inflexible-in-row-2.html)

### <a id="definite-sizes"></a>9.8.  Definite and Indefinite Sizes

<a id="ref-for-definite①③"></a>

<a id="ref-for-indefinite"></a>

Although CSS Sizing [\[CSS-SIZING-3\]](#biblio-css-sizing-3) defines [definite](https://www.w3.org/TR/css-sizing-3/#definite) and [indefinite](https://www.w3.org/TR/css-sizing-3/#indefinite) lengths, Flexbox has several additional cases where a length can be considered <a id="definite"></a>definite:

1.  <a id="ref-for-flex-container④⑧"></a>

    <a id="ref-for-definite①④"></a>

    <a id="ref-for-main-size②⑧"></a>

    <a id="ref-for-flex-item⑨⑦"></a>

    If the [flex container](#flex-container) has a [definite](#definite) [main size](#main-size), then the post-flexing <a id="ref-for-main-size②⑨"></a>main sizes of its [flex items](#flex-item) are treated as <a id="ref-for-definite①⑤"></a>definite.

2.  <a id="ref-for-flex-item⑨⑧"></a>

    <a id="ref-for-flex-flex-basis⑧"></a>

    <a id="ref-for-definite①⑥"></a>

    <a id="ref-for-main-size③⓪"></a>

    If a [flex item’s](#flex-item) [flex basis](#flex-flex-basis) is [definite](#definite), then its post-flexing [main size](#main-size) is also <a id="ref-for-definite①⑦"></a>definite.

3.  <a id="ref-for-single-line-flex-container⑨"></a>

    <a id="ref-for-flex-container④⑨"></a>

    <a id="ref-for-cross-size②①"></a>

    <a id="ref-for-automatic-size②"></a>

    <a id="ref-for-preferred-size②"></a>

    <a id="ref-for-outer-size①"></a>

    <a id="ref-for-stretched"></a>

    <a id="ref-for-flex-item⑨⑨"></a>

    <a id="ref-for-definite①⑧"></a>

    If a [single-line](#single-line-flex-container) [flex container](#flex-container) has a definite [cross size](#cross-size), the [automatic](https://www.w3.org/TR/css-sizing-3/#automatic-size) [preferred](https://www.w3.org/TR/css-sizing-3/#preferred-size) [outer](https://www.w3.org/TR/css-sizing-3/#outer-size) <a id="ref-for-cross-size②②"></a>cross size of any [stretched](#stretched) [flex items](#flex-item) is the flex container’s inner <a id="ref-for-cross-size②③"></a>cross size (clamped to the <a id="ref-for-flex-item①⓪⓪"></a>flex item’s min and max <a id="ref-for-cross-size②④"></a>cross size) and is considered [definite](#definite).

4.  Once the cross size of a flex line has been determined, the cross sizes of items in auto-sized flex containers are also considered definite for the purpose of layout; see [step 11](#algo-stretch).

Tests

- [flexbox-definite-cross-size-constrained-percentage.html](https://wpt.fyi/results/css/css-flexbox/flexbox-definite-cross-size-constrained-percentage.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-definite-cross-size-constrained-percentage.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-definite-cross-size-constrained-percentage.html)
- [flexbox-definite-sizes-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-definite-sizes-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-definite-sizes-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-definite-sizes-001.html)
- [flexbox-definite-sizes-002.html](https://wpt.fyi/results/css/css-flexbox/flexbox-definite-sizes-002.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-definite-sizes-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-definite-sizes-002.html)
- [flexbox-definite-sizes-003.html](https://wpt.fyi/results/css/css-flexbox/flexbox-definite-sizes-003.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-definite-sizes-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-definite-sizes-003.html)
- [flexbox-definite-sizes-004.html](https://wpt.fyi/results/css/css-flexbox/flexbox-definite-sizes-004.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-definite-sizes-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-definite-sizes-004.html)
- [flexbox-definite-sizes-005.html](https://wpt.fyi/results/css/css-flexbox/flexbox-definite-sizes-005.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-definite-sizes-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-definite-sizes-005.html)
- [flexbox-definite-sizes-006.html](https://wpt.fyi/results/css/css-flexbox/flexbox-definite-sizes-006.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-definite-sizes-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-definite-sizes-006.html)
- [height-percentage-with-dynamic-container-size.html](https://wpt.fyi/results/css/css-flexbox/height-percentage-with-dynamic-container-size.html) [(live test)](http://wpt.live/css/css-flexbox/height-percentage-with-dynamic-container-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/height-percentage-with-dynamic-container-size.html)
- [percentage-widths-001.html](https://wpt.fyi/results/css/css-flexbox/percentage-widths-001.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-widths-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-widths-001.html)
- [position-fixed-001.html](https://wpt.fyi/results/css/css-flexbox/position-fixed-001.html) [(live test)](http://wpt.live/css/css-flexbox/position-fixed-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/position-fixed-001.html)
- [stretch-obeys-min-max-001.html](https://wpt.fyi/results/css/css-flexbox/stretch-obeys-min-max-001.html) [(live test)](http://wpt.live/css/css-flexbox/stretch-obeys-min-max-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/stretch-obeys-min-max-001.html)
- [stretch-obeys-min-max-002.html](https://wpt.fyi/results/css/css-flexbox/stretch-obeys-min-max-002.html) [(live test)](http://wpt.live/css/css-flexbox/stretch-obeys-min-max-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/stretch-obeys-min-max-002.html)
- [stretch-obeys-min-max-003.html](https://wpt.fyi/results/css/css-flexbox/stretch-obeys-min-max-003.html) [(live test)](http://wpt.live/css/css-flexbox/stretch-obeys-min-max-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/stretch-obeys-min-max-003.html)

<a id="ref-for-flex-layout①"></a>

<a id="ref-for-flex-item①⓪①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that within [flex layout](#flex-layout), “definite” sizes can require performing layout. This was done to allow percentages inside of [flex items](#flex-item) to resolve where authors expected them to resolve.

### <a id="intrinsic-sizes"></a>9.9.  Intrinsic Sizes

<a id="ref-for-intrinsic-sizing"></a>

<a id="ref-for-flex-container⑤⓪"></a>

<a id="ref-for-max-content②"></a>

The [intrinsic sizing](https://www.w3.org/TR/css-sizing-3/#intrinsic-sizing) of a [flex container](#flex-container) is used to produce various types of content-based automatic sizing, such as shrink-to-fit logical widths (which use the fit-content formula) and content-based logical heights (which use the [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content)). For these computations, auto margins on flex items are treated as 0.

See [\[CSS-SIZING-3\]](#biblio-css-sizing-3) for a definition of the terms in this section.

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

#### <a id="intrinsic-main-sizes"></a>9.9.1.  Flex Container Intrinsic Main Sizes

<a id="ref-for-max-content③"></a>

<a id="ref-for-main-size③①"></a>

<a id="ref-for-flex-container⑤①"></a>

<a id="ref-for-flex-container⑤②"></a>

<a id="ref-for-flex-item①⓪②"></a>

The <strong><a href="https://www.w3.org/TR/css-sizing-3/#max-content">max-content</a> <a href="#main-size">main size</a> of a <a href="#flex-container">flex container</a></strong> is, theoretically, the smallest size the [flex container](#flex-container) can take such that when flex layout is run with that container size, each [flex item](#flex-item) ends up at least as large as its [max-content contribution](#intrinsic-item-contributions), to the extent allowed by the items’ flexibility.

<a id="ref-for-min-content②"></a>

<a id="ref-for-main-size③②"></a>

<a id="ref-for-flex-container⑤③"></a>

<a id="ref-for-flex-container⑤④"></a>

The <strong><a href="https://www.w3.org/TR/css-sizing-3/#min-content">min-content</a> <a href="#main-size">main size</a> of a <a href="#flex-container">flex container</a></strong> is, theoretically, the smallest size the [flex container](#flex-container) can take such that no items overflow it, and no item’s contents overflow the item—​setting aside the cases in which the boxes layouts are <em>defined</em> to overflow (for example with negative margins or percentage sizes that add up to more than 100%).

Tests

- [flex-container-max-content-001.html](https://wpt.fyi/results/css/css-flexbox/flex-container-max-content-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-container-max-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-container-max-content-001.html)
- [flex-container-min-content-001.html](https://wpt.fyi/results/css/css-flexbox/flex-container-min-content-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-container-min-content-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-container-min-content-001.html)

##### <a id="intrinsic-main-sizes-ideal"></a>9.9.1.1.  Ideal Algorithm

<a id="ref-for-flex-container⑤⑤"></a>

<a id="ref-for-main-size③③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The following algorithm calculates the [flex container](#flex-container)’s ideal intrinsic [main sizes](#main-size). However, because it was not implemented correctly initially, and existing content became dependent on the (unfortunately consistent) incorrect implemented behavior, [it is not Web-compatible](https://github.com/w3c/csswg-drafts/issues/8884). Implementers and the CSS Working Group are investigating to what extent Web browser implementations can safely approach this behavior, and further experimentation is welcome.

<a id="ref-for-collapsed-flex-item③"></a>

<a id="ref-for-flex-item①⓪③"></a>

Considering only non-[collapsed](#collapsed-flex-item) [flex items](#flex-item);

1.  <a id="ref-for-scaled-flex-shrink-factor②"></a>

    <a id="ref-for-flex-flex-grow-factor⑤"></a>

    <a id="ref-for-flex-base-size①⑦"></a>

    <a id="ref-for-flex-item①⓪④"></a>

    For each [flex item](#flex-item), subtract its outer [flex base size](#flex-base-size) from its [max-content contribution](#intrinsic-item-contributions) size. If that result is positive, divide it by the item’s [flex grow factor](#flex-flex-grow-factor) if the <a id="ref-for-flex-flex-grow-factor⑥"></a>flex grow factor is ≥ 1, or multiply it by the <a id="ref-for-flex-flex-grow-factor⑦"></a>flex grow factor if the <a id="ref-for-flex-flex-grow-factor⑧"></a>flex grow factor is \< 1; if the result is negative, divide it by the item’s [scaled flex shrink factor](#scaled-flex-shrink-factor) (if dividing by zero, treat the result as negative infinity). This is the item’s <var>desired flex fraction</var>.

2.  <a id="ref-for-flex-item①⓪⑤"></a>

    Place all [flex items](#flex-item) into lines of infinite length. Within each line, find the greatest (most positive) <var>desired flex fraction</var> among all the <a id="ref-for-flex-item①⓪⑥"></a>flex items. This is the line’s <var>chosen flex fraction</var>.

3.  <a id="ref-for-flex-flex-grow-factor⑨"></a>

    If the <var>chosen flex fraction</var> is positive, and the sum of the line’s [flex grow factors](#flex-flex-grow-factor) is less than 1, divide the <var>chosen flex fraction</var> by that sum.

    <a id="ref-for-flex-flex-shrink-factor⑥"></a>

    If the <var>chosen flex fraction</var> is negative, and the sum of the line’s [flex shrink factors](#flex-flex-shrink-factor) is less than 1, multiply the <var>chosen flex fraction</var> by that sum.

4.  <a id="ref-for-min-main-size"></a>

    <a id="ref-for-max-main-size"></a>

    <a id="ref-for-scaled-flex-shrink-factor③"></a>

    <a id="ref-for-flex-flex-grow-factor①⓪"></a>

    <a id="ref-for-flex-base-size①⑧"></a>

    Add each item’s [flex base size](#flex-base-size) to the product of its [flex grow factor](#flex-flex-grow-factor) ([scaled flex shrink factor](#scaled-flex-shrink-factor), if shrinking) and the <var>chosen flex fraction</var>, then clamp that result by the [max main size](#max-main-size) floored by the [min main size](#min-main-size).

5.  <a id="ref-for-max-content④"></a>

    <a id="ref-for-flex-container⑤⑥"></a>

    The [flex container](#flex-container)’s [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content) is the largest sum (among all the lines) of the afore-calculated sizes of all items within a single line.

<a id="ref-for-min-content③"></a>

<a id="ref-for-main-size③④"></a>

<a id="ref-for-single-line-flex-container①⓪"></a>

<a id="ref-for-max-content⑤"></a>

<a id="ref-for-main-size③⑤"></a>

<a id="ref-for-flex-item①⓪⑦"></a>

The <strong><a href="https://www.w3.org/TR/css-sizing-3/#min-content">min-content</a> <a href="#main-size">main size</a></strong> of a <em><a href="#single-line-flex-container">single-line</a></em> flex container is calculated identically to the [max-content](https://www.w3.org/TR/css-sizing-3/#max-content) [main size](#main-size), except that the [flex items](#flex-item)’ [min-content contributions](#intrinsic-item-contributions) are used instead of their [max-content contributions](#intrinsic-item-contributions).

> <strong data-conversion-semantic="note">Note</strong>
>
> Implications of this algorithm when the sum of flex is less than 1
>
> <a id="ref-for-flex-container⑤⑦"></a>
>
> The above algorithm is designed to give the correct behavior for two cases in particular, and make the [flex container’s](#flex-container) size continuous as you transition between the two:
>
> 1.  <a id="ref-for-flex-container⑤⑧"></a>
>
>     <a id="ref-for-flex-base-size①⑨"></a>
>
>     <a id="ref-for-propdef-width①⑤"></a>
>
>     <a id="ref-for-propdef-height①⓪"></a>
>
>     <a id="ref-for-max-content-contribution"></a>
>
>     If all items are inflexible, the [flex container](#flex-container) is sized to the sum of their [flex base size](#flex-base-size). (An inflexible <a id="ref-for-flex-base-size②⓪"></a>flex base size basically substitutes for a [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)/[height](https://www.w3.org/TR/css-sizing-3/#propdef-height), which, when specified, is what a [max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) is based on in Block Layout.)
>
> 2.  <a id="ref-for-flex-factor"></a>
>
>     <a id="ref-for-flex-container⑤⑨"></a>
>
>     <a id="ref-for-max-content-contribution①"></a>
>
>     When all items are flexible with [flex factors](#flex-factor) ≥ 1, the [flex container](#flex-container) is sized to the sum of the [max-content contributions](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) of its items (or perhaps a slightly larger size, so that every flex item is <em>at least</em> the size of its <a id="ref-for-max-content-contribution②"></a>max-content contribution, but also has the correct ratio of its size to the size of the other items, as determined by its flexibility).
>
> <a id="ref-for-flex-container⑥⓪"></a>
>
> <a id="ref-for-flex-item①⓪⑧"></a>
>
> <a id="ref-for-propdef-flex-basis①⑤"></a>
>
> <a id="ref-for-propdef-flex-grow①④"></a>
>
> For example, if a [flex container](#flex-container) has a single [flex item](#flex-item) with [flex-basis: 100px;](#propdef-flex-basis) but a max-content size of 200px, then when the item is [flex-grow: 0](#propdef-flex-grow), the <a id="ref-for-flex-container⑥①"></a>flex container (and <a id="ref-for-flex-item①⓪⑨"></a>flex item) is 100px wide, but when the item is <a id="ref-for-propdef-flex-grow①⑤"></a>flex-grow: 1 or higher, the <a id="ref-for-flex-container⑥②"></a>flex container (and flex item) is 200px wide.
>
> <a id="ref-for-flex-factor①"></a>
>
> <a id="ref-for-propdef-flex-grow①⑥"></a>
>
> <a id="ref-for-flex-container⑥③"></a>
>
> There are several possible ways to make the overall behavior continuous between these two cases, but all of them have drawbacks. We chose one we feel has the least bad implications; unfortunately, it "double-applies" the flexibility in cases with [flex factors](#flex-factor) that are \< 1. In the above example, if the item has [flex-grow: .5](#propdef-flex-grow), then the [flex container](#flex-container) ends up 150px wide, but the item then sizes normally into that available space, ending up 125px wide.
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Even more involved notes on the specific behavior chosen
> >
> > Principles:
> >
> > 1.  Don’t explode any sizes, whether growing or shrinking, as inputs approach zero.
> >
> > 2.  When flex factors are all \>=1, return the minimum size necessary for every item to be \>= max-content size.
> >
> > 3.  Items with a zero flex shouldn’t affect the sizes at all.
> >
> > 4.  Keep it continuous over variance of flex factors and item sizes.
> >
> > 5.  Keep sizing variance as linear as possible with respect to linear changes to any input variable (size, flex factor).
> >
> > 6.  When the sum of flex factors is \>=1, return the minimum size necessary for every item to be \>= max-content size.
> >
> > To get these all to work together, we have to apply some correction when either flex factors or the sum of flex factors on a line is \< 1.
> >
> > For shrink our behavior is somewhat easier; since the explosive case of 0 shrink results in a negative infinity desired fraction which we’ll never choose (since we always take the largest), we can just apply the correction at the line level, giving us double-application only when the sum is \< 1.
> >
> > For positives it’s more complicated. 0 grow naively explodes into \*positive\* infinity, which we’d choose, so we need to apply the correction at the individual item level. We do that by multiplying the space by the factor when factor is \<1. Leaving it at that would result in a double-application for items \< 1 but sum \>= 1, but a \*triple\*-application when the sum is \< 1. To avoid \*that\* ridiculousness, we apply a \*reverse\* correction when the sum is 1, dividing by the sum instead. This leaves us with a double correction in all cases for items with factors \< 1.
> >
> > <a id="ref-for-propdef-flex-grow①⑦"></a>
> >
> > We can’t eliminate the double-applications entirely without giving up other, more important principles (in particular, principle 3 —​try to come up with rules that don’t double-apply when you have two items with [flex-grow: .5](#propdef-flex-grow), but also don’t give a <a id="ref-for-propdef-flex-grow①⑧"></a>flex-grow: 0 item any power over a <a id="ref-for-propdef-flex-grow①⑨"></a>flex-grow: 1 sibling; you can’t, as far as we can tell.)

##### <a id="intrinsic-main-sizes-compat"></a>9.9.1.2.  Web-compatible Intrinsic Sizing Algorithm

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The following algorithm has been demonstrated to be Web-compatible. It may be altered in the future to bring it closer to the ideal algorithm above, if possible.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-bc420653"></a> Outline Web-compatible algorithm here, once we have one. [\[Issue \#8884\]](https://github.com/w3c/csswg-drafts/issues/8884)

##### <a id="intrinsic-main-sizes-multiline"></a>9.9.1.3.  Multi-line Min-content Algorithm

<a id="ref-for-multi-line-flex-container⑧"></a>

<a id="ref-for-min-content④"></a>

<a id="ref-for-main-size③⑥"></a>

<a id="ref-for-collapsed-flex-item④"></a>

<a id="ref-for-flex-item①①⓪"></a>

<a id="ref-for-flex-container⑥④"></a>

<a id="ref-for-flex-base-size②①"></a>

For a <em><a href="#multi-line-flex-container">multi-line</a></em> container, the [min-content](https://www.w3.org/TR/css-sizing-3/#min-content) [main size](#main-size) is simply the largest [min-content contribution](#intrinsic-item-contributions) of all the non-[collapsed](#collapsed-flex-item) [flex items](#flex-item) in the [flex container](#flex-container). For this purpose, each item’s contribution is capped by the item’s [flex base size](#flex-base-size) if the item is not growable, floored by the item’s <a id="ref-for-flex-base-size②②"></a>flex base size if the item is not shrinkable, and then further clamped by the item’s min and max main sizes.

#### <a id="intrinsic-cross-sizes"></a>9.9.2.  Flex Container Intrinsic Cross Sizes

<a id="ref-for-min-content⑤"></a>

<a id="ref-for-max-content⑥"></a>

<a id="ref-for-cross-size②⑤"></a>

<a id="ref-for-single-line-flex-container①①"></a>

<a id="ref-for-flex-container⑥⑤"></a>

<a id="ref-for-min-content-contribution"></a>

<a id="ref-for-max-content-contribution③"></a>

<a id="ref-for-flex-item①①①"></a>

The <strong><a href="https://www.w3.org/TR/css-sizing-3/#min-content">min-content</a>/<a href="https://www.w3.org/TR/css-sizing-3/#max-content">max-content</a> <a href="#cross-size">cross size</a></strong> of a <em><a href="#single-line-flex-container">single-line</a></em> [flex container](#flex-container) is the largest [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution)/[max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) (respectively) of its [flex items](#flex-item).

<a id="ref-for-multi-line-flex-container⑨"></a>

<a id="ref-for-flex-container⑥⑥"></a>

For a <em><a href="#multi-line-flex-container">multi-line</a></em> [flex container](#flex-container), the behavior depends on whether it’s a row or column flexbox:

<a id="ref-for-cross-size②⑥"></a>

<a id="ref-for-flex-container⑥⑦"></a>

<a id="ref-for-multi-line-flex-container①⓪"></a>

<a id="ref-for-valdef-flex-direction-row③"></a>

[row](#valdef-flex-direction-row) [multi-line](#multi-line-flex-container) [flex container](#flex-container) [cross-size](https://www.w3.org/TR/css-flexbox-1/#cross-size)

<a id="ref-for-min-content⑥"></a>

<a id="ref-for-max-content⑦"></a>

<a id="ref-for-cross-size②⑦"></a>

<a id="ref-for-cross-axis①⑥"></a>

<a id="ref-for-min-content-constraint①"></a>

<a id="ref-for-max-content-constraint①"></a>

The [min-content](https://www.w3.org/TR/css-sizing-3/#min-content)/[max-content](https://www.w3.org/TR/css-sizing-3/#max-content) [cross size](#cross-size) is the sum of the flex line cross sizes resulting from sizing the flex container under a [cross-axis](#cross-axis) [min-content constraint](https://www.w3.org/TR/css-sizing-3/#min-content-constraint)/[max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint) (respectively).

<a id="ref-for-cross-size②⑧"></a>

<a id="ref-for-flex-container⑥⑧"></a>

<a id="ref-for-multi-line-flex-container①①"></a>

<a id="ref-for-valdef-flex-direction-column①"></a>

[column](#valdef-flex-direction-column) [multi-line](#multi-line-flex-container) [flex container](#flex-container) [cross-size](https://www.w3.org/TR/css-flexbox-1/#cross-size)

<a id="ref-for-min-content⑦"></a>

<a id="ref-for-cross-size②⑨"></a>

<a id="ref-for-min-content-contribution①"></a>

<a id="ref-for-flex-item①①②"></a>

The [min-content](https://www.w3.org/TR/css-sizing-3/#min-content) [cross size](#cross-size) is the largest [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) among all of its [flex items](#flex-item).

<a id="ref-for-min-content⑧"></a>

<a id="ref-for-max-content⑧"></a>

<a id="ref-for-flex-container⑥⑨"></a>

<a id="ref-for-scroll-container③"></a>

<a id="ref-for-scrollport"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This heuristic effectively assumes a single flex line, in order to guarantee that the [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content) is smaller than the [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content). If the flex container has a height constraint, this will result in overflow, but if the [flex container](#flex-container) is also a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container), it will at least be large enough to fit any given column entirely within its [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport).

<a id="ref-for-max-content⑨"></a>

<a id="ref-for-cross-size③⓪"></a>

<a id="ref-for-flex-line⑦"></a>

<a id="ref-for-flex-container⑦⓪"></a>

<a id="ref-for-cross-axis①⑦"></a>

<a id="ref-for-max-content-constraint②"></a>

<a id="ref-for-max-content-contribution④"></a>

<a id="ref-for-cross-size③②"></a>

<a id="ref-for-flex-item①①③"></a>

<a id="ref-for-available④"></a>

The [max-content](https://www.w3.org/TR/css-sizing-3/#max-content) [cross size](#cross-size) is the sum of the [flex line](#flex-line) <a id="ref-for-cross-size③①"></a>cross sizes resulting from sizing the [flex container](#flex-container) under a [cross-axis](#cross-axis) [max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint), using the largest [max-content](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) [cross-size](https://www.w3.org/TR/css-flexbox-1/#cross-size) contribution among the [flex items](#flex-item) as the [available space](https://www.w3.org/TR/css-sizing-3/#available) in the <a id="ref-for-cross-axis①⑧"></a>cross axis for each of the <a id="ref-for-flex-item①①④"></a>flex items during layout.

<a id="ref-for-flex-container⑦①"></a>

<a id="ref-for-flex-item①①⑤"></a>

<a id="ref-for-max-content-contribution⑤"></a>

<a id="ref-for-flex-line⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This heuristic gives a reasonable approximation of the size that the [flex container](#flex-container) should be, with each [flex item](#flex-item) laid out at its [max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) or larger, and each [flex line](#flex-line) no larger than its largest <a id="ref-for-flex-item①①⑥"></a>flex item. It’s not a <em>perfect</em> fit in some cases, but doing it completely correct is insanely expensive, and this works reasonably well.

Tests

- [flexbox_width-change-and-relayout-children.html](https://wpt.fyi/results/css/css-flexbox/flexbox_width-change-and-relayout-children.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_width-change-and-relayout-children.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_width-change-and-relayout-children.html)
- [table-as-flex-item-max-content.html](https://wpt.fyi/results/css/css-flexbox/table-as-flex-item-max-content.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-flex-item-max-content.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-flex-item-max-content.html)
- [table-as-item-flex-cross-size.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-flex-cross-size.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-flex-cross-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-flex-cross-size.html)

#### <a id="intrinsic-item-contributions"></a>9.9.3.  Flex Item Intrinsic Size Contributions

<a id="ref-for-min-content-contribution②"></a>

<a id="ref-for-flex-item①①⑦"></a>

<a id="ref-for-min-content⑨"></a>

<a id="ref-for-preferred-size③"></a>

<a id="ref-for-valdef-width-auto⑧"></a>

<a id="ref-for-min-main-size①"></a>

<a id="ref-for-max-main-size①"></a>

The <strong>main-size <a href="https://www.w3.org/TR/css-sizing-3/#min-content-contribution">min-content contribution</a> of a <a href="#flex-item">flex item</a></strong> is the larger of its <em>outer</em> [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content) and outer [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) if that is not [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), clamped by its [min](#min-main-size)/[max main size](#max-main-size).

<a id="ref-for-max-content-contribution⑥"></a>

<a id="ref-for-flex-item①①⑧"></a>

<a id="ref-for-max-content①⓪"></a>

<a id="ref-for-preferred-size④"></a>

<a id="ref-for-valdef-width-auto⑨"></a>

<a id="ref-for-min-main-size②"></a>

<a id="ref-for-max-main-size②"></a>

The <strong>main-size <a href="https://www.w3.org/TR/css-sizing-3/#max-content-contribution">max-content contribution</a> of a <a href="#flex-item">flex item</a></strong> is the larger of its <em>outer</em> [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content) and outer [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) if that is not [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), clamped by its [min](#min-main-size)/[max main size](#max-main-size).

## <a id="pagination"></a>10.  Fragmenting Flex Layout

<a id="ref-for-multi-line-flex-container①②"></a>

<a id="ref-for-propdef-break-before"></a>

Flex containers can break across pages between items, between lines of items (in [multi-line](#multi-line-flex-container) mode), and inside items. The [break-\*](https://www.w3.org/TR/css-break-3/#propdef-break-before) properties apply to flex containers as normal for block-level or inline-level boxes. This section defines how they apply to flex items and the contents of flex items. See the [CSS Fragmentation Module](https://www.w3.org/TR/css-break/) for more context [\[CSS3-BREAK\]](#biblio-css3-break).

<a id="ref-for-fragmentation-container"></a>

<a id="ref-for-fragmentation-context"></a>

<a id="ref-for-flex-container⑦②"></a>

The following breaking rules refer to the [fragmentation container](https://www.w3.org/TR/css-break-4/#fragmentation-container) as the “page”. The same rules apply in any other [fragmentation context](https://www.w3.org/TR/css-break-4/#fragmentation-context). (Substitute “page” with the appropriate <a id="ref-for-fragmentation-container①"></a>fragmentation container type as needed.) For readability, in this section the terms "row" and "column" refer to the relative orientation of the [flex container](#flex-container) with respect to the block flow direction of the <a id="ref-for-fragmentation-context①"></a>fragmentation context, rather than to that of the <a id="ref-for-flex-container⑦③"></a>flex container itself.

<a id="ref-for-order-modified-document-order②"></a>

The exact layout of a fragmented flex container is not defined in this level of Flexible Box Layout. However, breaks inside a flex container are subject to the following rules (interpreted using [order-modified document order](https://www.w3.org/TR/css-flexbox-1/#order-modified-document-order)):

- <a id="ref-for-propdef-break-after"></a>

  <a id="ref-for-propdef-break-before①"></a>

  In a row flex container, the [break-before](https://www.w3.org/TR/css-break-3/#propdef-break-before) and [break-after](https://www.w3.org/TR/css-break-3/#propdef-break-after) values on flex items are propagated to the flex line. The <a id="ref-for-propdef-break-before②"></a>break-before values on the first line and the <a id="ref-for-propdef-break-after①"></a>break-after values on the last line are propagated to the flex container.

  <a id="ref-for-propdef-text-decoration"></a>

  <a id="ref-for-computed-value②"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: Break propagation (like [text-decoration](https://www.w3.org/TR/css-text-decor-4/#propdef-text-decoration) propagation) does not affect [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value).

- <a id="ref-for-propdef-break-after②"></a>

  <a id="ref-for-propdef-break-before③"></a>

  In a column flex container, the [break-before](https://www.w3.org/TR/css-break-3/#propdef-break-before) values on the first item and the [break-after](https://www.w3.org/TR/css-break-3/#propdef-break-after) values on the last item are propagated to the flex container. Forced breaks on other items are applied to the item itself.

- A forced break inside a flex item effectively increases the size of its contents; it does not trigger a forced break inside sibling items.

- In a row flex container, [Class A break opportunities](https://www.w3.org/TR/css3-break/#btw-blocks) occur between sibling flex lines, and [Class C break opportunities](https://www.w3.org/TR/css3-break/#end-block) occur between the first/last flex line and the flex container’s content edges. In a column flex container, [Class A break opportunities](https://www.w3.org/TR/css3-break/#btw-blocks) occur between sibling flex items, and [Class C break opportunities](https://www.w3.org/TR/css3-break/#end-block) occur between the first/last flex items on a line and the flex container’s content edges. [\[CSS3-BREAK\]](#biblio-css3-break)

- <a id="ref-for-flex-item①①⑨"></a>

  When a flex container is continued after a break, the space available to its [flex items](#flex-item) (in the block flow direction of the fragmentation context) is reduced by the space consumed by flex container fragments on previous pages. The space consumed by a flex container fragment is the size of its content box on that page. If as a result of this adjustment the available space becomes negative, it is set to zero.

- If the first fragment of the flex container is not at the top of the page, and none of its flex items fit in the remaining space on the page, the entire fragment is moved to the next page.

- <a id="ref-for-multi-line-flex-container①③"></a>

  When breaking a [multi-line](#multi-line-flex-container) column flex container, the UA may organize each fragment into its own “stack” of flex lines—​just like each fragment of a multi-column container has its own row of column boxes—​in order to ensure that content presented on earlier pages corresponds to content earlier in the box order.

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

### <a id="pagination-algo"></a>10.1.  Sample Flex Fragmentation Algorithm

This informative section presents a possible fragmentation algorithm for flex containers. Implementors are encouraged to improve on this algorithm and [provide feedback to the CSS Working Group](#sotd).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-93cdec9b"></a>
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > This algorithm assumes that pagination always proceeds only in the forward direction; therefore, in the algorithms below, alignment is mostly ignored prior to pagination. Advanced layout engines may be able to honor alignment across fragments.
>
> <a id="ref-for-single-line-flex-container①②"></a>
>
> [single-line](#single-line-flex-container) column flex container
>
> 1.  Run the flex layout algorithm (without regards to pagination) through [Cross Sizing Determination](#cross-sizing).
> 2.  Lay out as many consecutive flex items or item fragments as possible (but at least one or a fragment thereof), starting from the first, until there is no more room on the page or a forced break is encountered.
> 3.  If the previous step ran out of room and the free space is positive, the UA may reduce the distributed free space on this page (down to, but not past, zero) in order to make room for the next unbreakable flex item or fragment. Otherwise, the item or fragment that does not fit is pushed to the next page. The UA should pull up if more than 50% of the fragment would have fit in the remaining space and should push otherwise.
> 4.  If there are any flex items or fragments not laid out by the previous steps, rerun the flex layout algorithm from [Line Length Determination](#line-sizing) through [Cross Sizing Determination](#cross-sizing) with the next page’s size and <em>all</em> the contents (including those already laid out), and return to the previous step, but starting from the first item or fragment not already laid out.
> 5.  For each fragment of the flex container, continue the flex layout algorithm from [Main-Axis Alignment](#main-alignment) to its finish.
>
> <a id="ref-for-single-line-flex-container①③"></a>
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > It is the intent of this algorithm that column-direction [single-line](#single-line-flex-container) flex containers paginate very similarly to block flow. As a test of the intent, a flex container with justify-content:start and no flexible items should paginate identically to a block with in-flow children with same content, same used size and same used margins.
>
> <a id="ref-for-multi-line-flex-container①④"></a>
>
> [multi-line](#multi-line-flex-container) column flex container
>
> 1.  Run the flex layout algorithm <em>with</em> regards to pagination (limiting the flex container’s maximum line length to the space left on the page) through [Cross Sizing Determination](#cross-sizing).
> 2.  Lay out as many flex lines as possible (but at least one) until there is no more room in the flex container in the cross dimension or a forced break is encountered:
>     1.  Lay out as many consecutive flex items as possible (but at least one), starting from the first, until there is no more room on the page or a forced break is encountered. Forced breaks <em>within</em> flex items are ignored.
>     2.  If this is the first flex container fragment, this line contains only a single flex item that is larger than the space left on the page, and the flex container is not at the top of the page already, move the flex container to the next page and restart flex container layout entirely.
>     3.  If there are any flex items not laid out by the first step, rerun the flex layout algorithm from [Main Sizing Determination](#main-sizing) through [Cross Sizing Determination](#cross-sizing) using only the items not laid out on a previous line, and return to the previous step, starting from the first item not already laid out.
> 3.  If there are any flex items not laid out by the previous step, rerun the flex layout algorithm from [Line Sizing Determination](#line-sizing) through [Cross Sizing Determination](#cross-sizing) with the next page’s size and only the items not already laid out, and return to the previous step, but starting from the first item not already laid out.
> 4.  For each fragment of the flex container, continue the flex layout algorithm from [Main-Axis Alignment](#main-alignment) to its finish.
>
> <a id="ref-for-multi-line-flex-container①⑤"></a>
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > A shortcoming of this sample algorithm is that if a flex item does not entirely fit on a single page, it will <em>not</em> be paginated in [multi-line](#multi-line-flex-container) column flex containers.
>
> <a id="ref-for-single-line-flex-container①④"></a>
>
> [single-line](#single-line-flex-container) row flex container
>
> 1.  <a id="ref-for-valdef-align-items-baseline①"></a>
>
>     <a id="ref-for-valdef-align-items-flex-start"></a>
>
>     <a id="ref-for-propdef-align-self①⑨"></a>
>
>     Run the entire flex layout algorithm (without regards to pagination), except treat any [align-self](#propdef-align-self) other than [flex-start](#valdef-align-items-flex-start) or [baseline](#valdef-align-items-baseline) as <a id="ref-for-valdef-align-items-flex-start①"></a>flex-start.
>
> 2.  If an unbreakable item doesn’t fit within the space left on the page, and the flex container is not at the top of the page, move the flex container to the next page and restart flex container layout entirely.
>
> 3.  For each item, lay out as much of its contents as will fit in the space left on the page, and fragment the remaining content onto the next page, rerunning the flex layout algorithm from [Line Length Determination](#line-sizing) through [Main-Axis Alignment](#main-alignment) into the new page size using <em>all</em> the contents (including items completed on previous pages).
>     > <strong data-conversion-semantic="note">Note</strong>
>     >
>     > Any flex items that fit entirely into previous fragments still take up space in the main axis in later fragments.
>
> 4.  <a id="ref-for-valdef-align-items-flex-start②"></a>
>
>     <a id="ref-for-propdef-align-content⑨"></a>
>
>     <a id="ref-for-propdef-align-self②⓪"></a>
>
>     For each fragment of the flex container, rerun the flex layout algorithm from [Cross-Axis Alignment](#cross-alignment) to its finish. For all fragments besides the first, treat [align-self](#propdef-align-self) and [align-content](#propdef-align-content) as being [flex-start](#valdef-align-items-flex-start) for all item fragments and lines.
>
> 5.  <a id="ref-for-cross-size③③"></a>
>
>     <a id="ref-for-propdef-align-self②①"></a>
>
>     If any item, when aligned according to its original [align-self](#propdef-align-self) value into the combined [cross size](#cross-size) of all the flex container fragments, would fit entirely within a single flex container fragment, it may be shifted into that fragment and aligned appropriately.
>
> <a id="ref-for-multi-line-flex-container①⑥"></a>
>
> [multi-line](#multi-line-flex-container) row flex container
>
> 1.  Run the flex layout algorithm (without regards to pagination), through [Cross Sizing Determination](#cross-sizing).
>
> 2.  Lay out as many flex lines as possible (but at least one), starting from the first, until there is no more room on the page or a forced break is encountered.
>
>     If a line doesn’t fit on the page, and the line is not at the top of the page, move the line to the next page and restart the flex layout algorithm entirely, using only the items in and following this line.
>
>     If a flex item itself causes a forced break, rerun the flex layout algorithm from [Main Sizing Determination](#main-sizing) through [Main-Axis Alignment](#main-alignment), using only the items on this and following lines, but with the item causing the break automatically starting a new line in the [line breaking step](#algo-line-break), then continue with this step. Forced breaks <em>within</em> flex items are ignored.
>
> 3.  If there are any flex items not laid out by the previous step, rerun the flex layout algorithm from [Line Length Determination](#line-sizing) through [Main-Axis Alignment](#main-alignment) with the next page’s size and only the items not already laid out. Return to the previous step, but starting from the first line not already laid out.
>
> 4.  For each fragment of the flex container, continue the flex layout algorithm from [Cross Axis Alignment](#cross-alignment) to its finish.

## <a id="axis-mapping"></a> Appendix A: Axis Mappings

<em>This appendix is non-normative.</em>

<a id="axis-mapping-table-en"></a>

<a id="ref-for-valdef-direction-ltr"></a>

<a id="ref-for-valdef-writing-mode-horizontal-tb"></a>

<a id="ref-for-writing-mode①⓪"></a>

**Table 15**

Axis Mappings for [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr) + [horizontal-tb](https://www.w3.org/TR/css-writing-modes-4/#valdef-writing-mode-horizontal-tb) [Writing Mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) (e.g. English)

Representation note: merged header paths are written explicitly; values from merged body cells are repeated wherever they apply.

| <a id="ref-for-propdef-flex-flow⑥"></a> [flex-flow](#propdef-flex-flow) | <a id="ref-for-main-axis①⑥"></a> [main axis](#main-axis) | main axis / <a id="ref-for-start"></a> [start](https://www.w3.org/TR/css-writing-modes-3/#start) | main axis / <a id="ref-for-end①"></a> [end](https://www.w3.org/TR/css-writing-modes-3/#end) | <a id="ref-for-cross-axis①⑨"></a> [cross axis](#cross-axis) | cross axis / <a id="ref-for-start①"></a> [start](https://www.w3.org/TR/css-writing-modes-3/#start) | cross axis / <a id="ref-for-end②"></a> [end](https://www.w3.org/TR/css-writing-modes-3/#end) |
| --- | --- | --- | --- | --- | --- | --- |
| <a id="ref-for-valdef-flex-wrap-wrap②"></a> <a id="ref-for-valdef-flex-wrap-nowrap"></a> <a id="ref-for-valdef-flex-direction-row④"></a> [row](#valdef-flex-direction-row) + [nowrap](#valdef-flex-wrap-nowrap)/[wrap](#valdef-flex-wrap-wrap) | horizontal | left | right | vertical | top | bottom |
| <a id="ref-for-valdef-flex-wrap-wrap③"></a> <a id="ref-for-valdef-flex-wrap-nowrap①"></a> <a id="ref-for-valdef-flex-direction-row-reverse①"></a> [row-reverse](#valdef-flex-direction-row-reverse) + [nowrap](#valdef-flex-wrap-nowrap)/[wrap](#valdef-flex-wrap-wrap) | horizontal | right | left | vertical | top | bottom |
| <a id="ref-for-valdef-flex-wrap-wrap-reverse③"></a> <a id="ref-for-valdef-flex-direction-row⑤"></a> [row](#valdef-flex-direction-row) + [wrap-reverse](#valdef-flex-wrap-wrap-reverse) | horizontal | left | right | vertical | bottom | top |
| <a id="ref-for-valdef-flex-wrap-wrap-reverse④"></a> <a id="ref-for-valdef-flex-direction-row-reverse②"></a> [row-reverse](#valdef-flex-direction-row-reverse) + [wrap-reverse](#valdef-flex-wrap-wrap-reverse) | horizontal | right | left | vertical | bottom | top |
| <a id="ref-for-valdef-flex-wrap-wrap④"></a> <a id="ref-for-valdef-flex-wrap-nowrap②"></a> <a id="ref-for-valdef-flex-direction-column②"></a> [column](#valdef-flex-direction-column) + [nowrap](#valdef-flex-wrap-nowrap)/[wrap](#valdef-flex-wrap-wrap) | vertical | top | bottom | horizontal | left | right |
| <a id="ref-for-valdef-flex-wrap-wrap⑤"></a> <a id="ref-for-valdef-flex-wrap-nowrap③"></a> <a id="ref-for-valdef-flex-direction-column-reverse"></a> [column-reverse](#valdef-flex-direction-column-reverse) + [nowrap](#valdef-flex-wrap-nowrap)/[wrap](#valdef-flex-wrap-wrap) | vertical | bottom | top | horizontal | left | right |
| <a id="ref-for-valdef-flex-wrap-wrap-reverse⑤"></a> <a id="ref-for-valdef-flex-direction-column③"></a> [column](#valdef-flex-direction-column) + [wrap-reverse](#valdef-flex-wrap-wrap-reverse) | vertical | top | bottom | horizontal | right | left |
| <a id="ref-for-valdef-flex-wrap-wrap-reverse⑥"></a> <a id="ref-for-valdef-flex-direction-column-reverse①"></a> [column-reverse](#valdef-flex-direction-column-reverse) + [wrap-reverse](#valdef-flex-wrap-wrap-reverse) | vertical | bottom | top | horizontal | right | left |

<a id="axis-mapping-table-fa"></a>

<a id="ref-for-valdef-direction-rtl"></a>

<a id="ref-for-valdef-writing-mode-horizontal-tb①"></a>

<a id="ref-for-writing-mode①①"></a>

**Table 16**

Axis Mappings for [rtl](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-rtl) + [horizontal-tb](https://www.w3.org/TR/css-writing-modes-4/#valdef-writing-mode-horizontal-tb) [Writing Mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) (e.g. Farsi)

Representation note: merged header paths are written explicitly; values from merged body cells are repeated wherever they apply.

| <a id="ref-for-propdef-flex-flow⑦"></a> [flex-flow](#propdef-flex-flow) | <a id="ref-for-main-axis①⑦"></a> [main axis](#main-axis) | main axis / <a id="ref-for-main-start⑨"></a> [main-start](#main-start) | main axis / <a id="ref-for-main-end⑨"></a> [main-end](#main-end) | <a id="ref-for-cross-axis②⓪"></a> [cross axis](#cross-axis) | cross axis / <a id="ref-for-cross-start①④"></a> [cross-start](#cross-start) | cross axis / <a id="ref-for-cross-end⑨"></a> [cross-end](#cross-end) |
| --- | --- | --- | --- | --- | --- | --- |
| <a id="ref-for-valdef-flex-wrap-wrap⑥"></a> <a id="ref-for-valdef-flex-wrap-nowrap④"></a> <a id="ref-for-valdef-flex-direction-row⑥"></a> [row](#valdef-flex-direction-row) + [nowrap](#valdef-flex-wrap-nowrap)/[wrap](#valdef-flex-wrap-wrap) | horizontal | right | left | vertical | top | bottom |
| <a id="ref-for-valdef-flex-wrap-wrap⑦"></a> <a id="ref-for-valdef-flex-wrap-nowrap⑤"></a> <a id="ref-for-valdef-flex-direction-row-reverse③"></a> [row-reverse](#valdef-flex-direction-row-reverse) + [nowrap](#valdef-flex-wrap-nowrap)/[wrap](#valdef-flex-wrap-wrap) | horizontal | left | right | vertical | top | bottom |
| <a id="ref-for-valdef-flex-wrap-wrap-reverse⑦"></a> <a id="ref-for-valdef-flex-direction-row⑦"></a> [row](#valdef-flex-direction-row) + [wrap-reverse](#valdef-flex-wrap-wrap-reverse) | horizontal | right | left | vertical | bottom | top |
| <a id="ref-for-valdef-flex-wrap-wrap-reverse⑧"></a> <a id="ref-for-valdef-flex-direction-row-reverse④"></a> [row-reverse](#valdef-flex-direction-row-reverse) + [wrap-reverse](#valdef-flex-wrap-wrap-reverse) | horizontal | left | right | vertical | bottom | top |
| <a id="ref-for-valdef-flex-wrap-wrap⑧"></a> <a id="ref-for-valdef-flex-wrap-nowrap⑥"></a> <a id="ref-for-valdef-flex-direction-column④"></a> [column](#valdef-flex-direction-column) + [nowrap](#valdef-flex-wrap-nowrap)/[wrap](#valdef-flex-wrap-wrap) | vertical | top | bottom | horizontal | right | left |
| <a id="ref-for-valdef-flex-wrap-wrap⑨"></a> <a id="ref-for-valdef-flex-wrap-nowrap⑦"></a> <a id="ref-for-valdef-flex-direction-column-reverse②"></a> [column-reverse](#valdef-flex-direction-column-reverse) + [nowrap](#valdef-flex-wrap-nowrap)/[wrap](#valdef-flex-wrap-wrap) | vertical | bottom | top | horizontal | right | left |
| <a id="ref-for-valdef-flex-wrap-wrap-reverse⑨"></a> <a id="ref-for-valdef-flex-direction-column⑤"></a> [column](#valdef-flex-direction-column) + [wrap-reverse](#valdef-flex-wrap-wrap-reverse) | vertical | top | bottom | horizontal | left | right |
| <a id="ref-for-valdef-flex-wrap-wrap-reverse①⓪"></a> <a id="ref-for-valdef-flex-direction-column-reverse③"></a> [column-reverse](#valdef-flex-direction-column-reverse) + [wrap-reverse](#valdef-flex-wrap-wrap-reverse) | vertical | bottom | top | horizontal | left | right |

<a id="axis-mapping-table-ja"></a>

<a id="ref-for-valdef-direction-ltr①"></a>

<a id="ref-for-valdef-writing-mode-vertical-rl"></a>

<a id="ref-for-writing-mode①②"></a>

**Table 17**

Axis Mappings for [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr) + [vertical-rl](https://www.w3.org/TR/css-writing-modes-4/#valdef-writing-mode-vertical-rl) [Writing Mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) (e.g. Japanese)

Representation note: merged header paths are written explicitly; values from merged body cells are repeated wherever they apply.

| <a id="ref-for-propdef-flex-flow⑧"></a> [flex-flow](#propdef-flex-flow) | <a id="ref-for-main-axis①⑧"></a> [main axis](#main-axis) | main axis / <a id="ref-for-main-start①⓪"></a> [start](#main-start) | main axis / <a id="ref-for-main-end①⓪"></a> [end](#main-end) | <a id="ref-for-cross-axis②①"></a> [cross axis](#cross-axis) | cross axis / <a id="ref-for-cross-start①⑤"></a> [start](#cross-start) | cross axis / <a id="ref-for-cross-end①⓪"></a> [end](#cross-end) |
| --- | --- | --- | --- | --- | --- | --- |
| <a id="ref-for-valdef-flex-wrap-wrap①⓪"></a> <a id="ref-for-valdef-flex-wrap-nowrap⑧"></a> <a id="ref-for-valdef-flex-direction-row⑧"></a> [row](#valdef-flex-direction-row) + [nowrap](#valdef-flex-wrap-nowrap)/[wrap](#valdef-flex-wrap-wrap) | vertical | top | bottom | horizontal | right | left |
| <a id="ref-for-valdef-flex-wrap-wrap①①"></a> <a id="ref-for-valdef-flex-wrap-nowrap⑨"></a> <a id="ref-for-valdef-flex-direction-row-reverse⑤"></a> [row-reverse](#valdef-flex-direction-row-reverse) + [nowrap](#valdef-flex-wrap-nowrap)/[wrap](#valdef-flex-wrap-wrap) | vertical | bottom | top | horizontal | right | left |
| <a id="ref-for-valdef-flex-wrap-wrap-reverse①①"></a> <a id="ref-for-valdef-flex-direction-row⑨"></a> [row](#valdef-flex-direction-row) + [wrap-reverse](#valdef-flex-wrap-wrap-reverse) | vertical | top | bottom | horizontal | left | right |
| <a id="ref-for-valdef-flex-wrap-wrap-reverse①②"></a> <a id="ref-for-valdef-flex-direction-row-reverse⑥"></a> [row-reverse](#valdef-flex-direction-row-reverse) + [wrap-reverse](#valdef-flex-wrap-wrap-reverse) | vertical | bottom | top | horizontal | left | right |
| <a id="ref-for-valdef-flex-wrap-wrap①②"></a> <a id="ref-for-valdef-flex-wrap-nowrap①⓪"></a> <a id="ref-for-valdef-flex-direction-column⑥"></a> [column](#valdef-flex-direction-column) + [nowrap](#valdef-flex-wrap-nowrap)/[wrap](#valdef-flex-wrap-wrap) | vertical | right | left | horizontal | top | bottom |
| <a id="ref-for-valdef-flex-wrap-wrap①③"></a> <a id="ref-for-valdef-flex-wrap-nowrap①①"></a> <a id="ref-for-valdef-flex-direction-column-reverse④"></a> [column-reverse](#valdef-flex-direction-column-reverse) + [nowrap](#valdef-flex-wrap-nowrap)/[wrap](#valdef-flex-wrap-wrap) | vertical | left | right | horizontal | top | bottom |
| <a id="ref-for-valdef-flex-wrap-wrap-reverse①③"></a> <a id="ref-for-valdef-flex-direction-column⑦"></a> [column](#valdef-flex-direction-column) + [wrap-reverse](#valdef-flex-wrap-wrap-reverse) | vertical | right | left | horizontal | bottom | top |
| <a id="ref-for-valdef-flex-wrap-wrap-reverse①④"></a> <a id="ref-for-valdef-flex-direction-column-reverse⑤"></a> [column-reverse](#valdef-flex-direction-column-reverse) + [wrap-reverse](#valdef-flex-wrap-wrap-reverse) | vertical | left | right | horizontal | bottom | top |

## <a id="webkit-aliases"></a> Appendix B: `-webkit-` Legacy Properties

<em>This appendix is normative.</em>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> These aliases are <em>deprecated</em> and authors <em>should not</em> use them
	unless their content needs to support actively-used legacy UAs.</strong>

<a id="ref-for-legacy-name-alias"></a>

For compatibility with general Web content, UAs that are Web browsers must and other UAs may implement the following [legacy name aliases](https://www.w3.org/TR/css-cascade-5/#legacy-name-alias):

| Alias                   | Standard        |
|-------------------------|-----------------|
| -webkit-align-content   | align-content   |
| -webkit-align-items     | align-items     |
| -webkit-align-self      | align-self      |
| -webkit-flex            | flex            |
| -webkit-flex-basis      | flex-basis      |
| -webkit-flex-direction  | flex-direction  |
| -webkit-flex-flow       | flex-flow       |
| -webkit-flex-grow       | flex-grow       |
| -webkit-flex-shrink     | flex-shrink     |
| -webkit-flex-wrap       | flex-wrap       |
| -webkit-justify-content | justify-content |
| -webkit-order           | order           |

Tests

- [flexbox_width-wrapping-column.html](https://wpt.fyi/results/css/css-flexbox/flexbox_width-wrapping-column.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_width-wrapping-column.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_width-wrapping-column.html)
- [webkit-aliases.html](https://wpt.fyi/results/css/css-flexbox/parsing/webkit-aliases.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/webkit-aliases.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/webkit-aliases.html)
- [webkit-box-vertical-writing-mode.html](https://wpt.fyi/results/css/css-flexbox/webkit-box-vertical-writing-mode.html) [(live test)](http://wpt.live/css/css-flexbox/webkit-box-vertical-writing-mode.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/webkit-box-vertical-writing-mode.html)

## <a id="acknowledgments"></a>Acknowledgments

Thanks for feedback and contributions to

Erik Anderson, Christian Biesinger, Tony Chang, Phil Cupp, Arron Eicholz, James Elmore, Andrew Fedoniouk, Brian Heuston, Shinichiro Hamaji, Daniel Holbert, Ben Horst, John Jansen, Brad Kemper, Kang-hao Lu, Markus Mielke, Peter Moulder, Robert O’Callahan, Christoph Päper, Ning Rogers, Peter Salas, Elliott Sprehn, Morten Stenshorne, Christian Stockwell, Ojan Vafai, Eugene Veselov, Greg Whitworth, Boris Zbarsky.

## <a id="changes"></a>Changes

This section documents the changes since previous publications.

### <a id="changes-20181119"></a> Changes since the [19 November 2018 CR](https://www.w3.org/TR/2018/CR-css-flexbox-1-20181119/)

A [Disposition of Comments](https://drafts.csswg.org/css-flexbox-1/issues-cr-2018) is available. Significant changes include:

- <a id="change-2018-intrinsic-main-size-compat"></a> Identify parts of [§ 9.9.1 Flex Container Intrinsic Main Sizes](#intrinsic-main-sizes) as ideal but not Web-compatible, and mark a refined Web-compatible algorithm as an ongoing investigation. ([Issue 8884](https://github.com/w3c/csswg-drafts/issues/8884))

- <a id="change-2018-intrinsic-main-size-errors"></a> Fix errors in the flexing rules in [§ 9.9.1 Flex Container Intrinsic Main Sizes](#intrinsic-main-sizes) to closer match the actual layout algorithm and avoid dividing by zero or exploding sizes as inputs approach zero. ([Issue 7189](https://github.com/w3c/csswg-drafts/issues/7189#issuecomment-1172771501))

- <a id="change-2018-min-content-column-size-compat"></a> Reform the [cross-size intrinsic sizing rules](#intrinsic-cross-sizes) for column wrap flex containers to yield better results. (Issue [6777](https://github.com/w3c/csswg-drafts/issues/6777))

- <a id="ref-for-propdef-align-self②②"></a>

  <a id="ref-for-propdef-align-content①⓪"></a>

  <a id="ref-for-static-position-rectangle②"></a>

  <a id="ref-for-flex-container⑦④"></a>

  <a id="ref-for-content-edge①"></a>

  <a id="change-2018-static-pos-align-content"></a> Use the [content edges](https://www.w3.org/TR/css-box-4/#content-edge) of the [flex container](#flex-container) for determining the [static-position rectangle](https://www.w3.org/TR/css-position-3/#static-position-rectangle) of flex container children, rather than doing extra layout work. This will result in ignoring [align-content](#propdef-align-content) on the flex container, but allow [align-self](#propdef-align-self) on the box itself to still take effect. ([Issue 5843](https://github.com/w3c/csswg-drafts/issues/5843), [7596](https://github.com/w3c/csswg-drafts/issues/7596))

- <a id="ref-for-preferred-size⑤"></a>

  <a id="ref-for-valdef-width-auto①⓪"></a>

  <a id="change-2018-correctly-ignore-auto"></a> Correctly ignore [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) [preferred sizes](https://www.w3.org/TR/css-sizing-3/#preferred-size) in both min <em>and</em> max size contribution calculations. (It was accidentally omitted from the max, though it shows up correctly in the [changelog](#change-2017-flex-min-contribution)...) ([Issue 6455](https://github.com/w3c/csswg-drafts/issues/6455))

- <a id="ref-for-propdef-order①⓪"></a>

  <a id="change-2018-move-order"></a> Moved the [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) property definition to [\[CSS-DISPLAY-3\]](#biblio-css-display-3). See [§ 5.4 Reordering and Accessibility: the order property](#order-property) for remaining explanation. ([Issue 5865](https://github.com/w3c/csswg-drafts/issues/5865))

- <a id="ref-for-flex-flex-basis⑨"></a>

  <a id="ref-for-flex-item①②⓪"></a>

  <a id="ref-for-definite①⑨"></a>

  <a id="ref-for-main-size③⑦"></a>

  <a id="change-2018-definite-basis-definite"></a> Made [main size](#main-size) always [definite](#definite) for [flex items](#flex-item) with a <a id="ref-for-definite②⓪"></a>definite [flex basis](#flex-flex-basis), to match implementations. ([Issue 4311](https://github.com/w3c/csswg-drafts/issues/4311))

  > 1.  <a id="ref-for-definite②③"></a>
  >
  >     <a id="ref-for-main-size④⓪"></a>
  >
  >     <a id="ref-for-flex-item①②②"></a>
  >
  >     <a id="ref-for-definite②②"></a>
  >
  >     <a id="ref-for-flex-item①②①"></a>
  >
  >     <a id="ref-for-main-size③⑨"></a>
  >
  >     <a id="ref-for-main-size③⑧"></a>
  >
  >     <a id="ref-for-definite②①"></a>
  >
  >     <a id="ref-for-flex-container⑦⑤"></a>
  >
  >     If the [flex container](#flex-container) has a [definite](#definite) [main size](#main-size), <u>then the post-flexing [main sizes](#main-size) of its [flex items](#flex-item) are treated as [definite](#definite)</u> ~~a [flex item](#flex-item)’s post-flexing [main size](#main-size) is treated as [definite](#definite), even though it can rely on the <a id="ref-for-definite②④"></a>indefinite sizes of any flex items in the same line~~ .
  >
  > 2.  <a id="ref-for-main-size④①"></a>
  >
  >     <a id="ref-for-definite②⑤"></a>
  >
  >     <a id="ref-for-flex-flex-basis①⓪"></a>
  >
  >     <a id="ref-for-flex-item①②③"></a>
  >
  >     <u>If a [flex item’s](#flex-item) [flex basis](#flex-flex-basis) is [definite](#definite), then its post-flexing [main size](#main-size) is also <a id="ref-for-definite②⑥"></a>definite.</u>

  > <a id="ref-for-fully-inflexible①"></a>
  >
  > <a id="ref-for-definite②⑦"></a>
  >
  > <a id="ref-for-flex-flex-basis①①"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > ~~Note: The main size of a [fully inflexible](#fully-inflexible) item with a [definite](https://www.w3.org/TR/css-sizing-3/#definite) [flex basis](#flex-flex-basis) is, by definition, <a id="ref-for-definite②⑧"></a>definite.~~
  >
  > <a id="ref-for-flex-layout②"></a>
  >
  > <a id="ref-for-flex-item①②④"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > <u>Note: This means that within [flex layout](#flex-layout), “definite” sizes can require performing layout. This was done to allow percentages inside of [flex items](#flex-item) to resolve where authors expected them to resolve.</u>

- <a id="ref-for-flex-item①②⑤"></a>

  <a id="change-2018-flex-column-wrap-fragmentation"></a> Made the re-arrangement of [flex items](#flex-item) into independent stacks of flex lines on each page optional, and identified shortcomings in the sample algorithm. ([6855](https://github.com/w3c/csswg-drafts/issues/6855))

- <a id="ref-for-css-bracketed-range-notation"></a>

  <a id="change-2018-bracket-range-notation"></a> Updated value syntax in property definition tables to use newer [CSS bracketed range notation](https://www.w3.org/TR/css-values-4/#css-bracketed-range-notation) reflecting the prose restrictions on negative values. (Editorial.)

- <a id="ref-for-propdef-aspect-ratio"></a>

  <a id="ref-for-replaced-element③"></a>

  <a id="ref-for-preferred-aspect-ratio⑥"></a>

  <a id="change-2018-preferred-aspect-ratio"></a> Updated aspect ratio terminology to use the term [preferred aspect ratio](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio), and clarify any uses that are specific to [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element), in order to accommodate forthcoming [aspect-ratio](https://www.w3.org/TR/css-sizing-4/#propdef-aspect-ratio) property.

- <a id="ref-for-specified-size-suggestion④"></a>

  <a id="ref-for-transferred-size-suggestion④"></a>

  <a id="ref-for-automatic-minimum-size③"></a>

  <a id="ref-for-propdef-aspect-ratio①"></a>

  <a id="change-2018-auto-min-aspect-ratio"></a> Define interaction of [aspect-ratio](https://www.w3.org/TR/css-sizing-4/#propdef-aspect-ratio) and the [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size) by applying its effect to the [transferred size suggestion](#transferred-size-suggestion) and excluding it from the definiteness of the [specified size suggestion](#specified-size-suggestion). (Issues [6069](https://github.com/w3c/csswg-drafts/issues/6069) and [6794](https://github.com/w3c/csswg-drafts/issues/6794), [changeset A](https://github.com/w3c/csswg-drafts/pull/11690) and [changeset B](https://github.com/w3c/csswg-drafts/commit/ca4dc9e0a3815be0306645dab03f8ca52434baf9))

- <a id="ref-for-used-value③"></a>

  <a id="ref-for-non-scrollable-overflow-value①"></a>

  <a id="ref-for-scrollable-overflow-value"></a>

  <a id="ref-for-replaced-element④"></a>

  <a id="ref-for-scroll-container④"></a>

  <a id="ref-for-propdef-overflow③"></a>

  <a id="ref-for-computed-value③"></a>

  <a id="ref-for-automatic-minimum-size④"></a>

  <a id="change-2018-auto-min-size-overflow"></a> Pin [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size) conditional on [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) rather than on being a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) to avoid cases such as [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element) whose computed [scrollable overflow values](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-value) resolve to [non-scrollable](https://www.w3.org/TR/css-overflow-3/#non-scrollable-overflow-value) [used values](https://www.w3.org/TR/css-cascade-5/#used-value). ([7714](https://github.com/w3c/csswg-drafts/issues/7714))

- <a id="ref-for-valdef-width-auto①①"></a>

  <a id="ref-for-content-based-minimum-size⑤"></a>

  <a id="change-2018-indefinite-min-intrinsic"></a> Clarify that the rule exempting [content-based minimum sizes](#content-based-minimum-size) from making an item’s size to be indefinite is limited to [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), and does not affect other intrinsic sizing keywords. ([Issue 6457](https://github.com/w3c/csswg-drafts/issues/6457))

- <a id="ref-for-flex-container⑦⑧"></a>

  <a id="ref-for-flex-item①②⑧"></a>

  <a id="ref-for-collapsed-flex-item⑦"></a>

  <a id="ref-for-multi-line-flex-container①⑦"></a>

  <a id="ref-for-collapsed-flex-item⑤"></a>

  <a id="change-2018-collapsed-intrinsic-container"></a> Explicitly excluded [collapsed flex items](#collapsed-flex-item) from the [intrinsic main size calculations](#intrinsic-main-sizes). ([Issue 5985](https://github.com/w3c/csswg-drafts/issues/5985))

  > <a id="ref-for-max-content①①"></a>
  >
  > <a id="ref-for-main-size④②"></a>
  >
  > <a id="ref-for-flex-container⑦⑥"></a>
  >
  > <a id="ref-for-flex-container⑦⑦"></a>
  >
  > <a id="ref-for-flex-item①②⑥"></a>
  >
  > <a id="ref-for-collapsed-flex-item⑥"></a>
  >
  > <a id="ref-for-flex-item①②⑦"></a>
  >
  > The <strong><a href="https://www.w3.org/TR/css-sizing-3/#max-content">max-content</a> <a href="#main-size">main size</a> of a <a href="#flex-container">flex container</a></strong> is the smallest size the [flex container](#flex-container) can take while maintaining the [max-content contributions](#intrinsic-item-contributions) of its [flex items](#flex-item), insofar as allowed by the items’ own flexibility. <u>Considering only non-[collapsed](#collapsed-flex-item) [flex items](#flex-item):</u>

  > However, for a <em><a href="#multi-line-flex-container">multi-line</a></em> container, it is simply the largest [min-content contribution](#intrinsic-item-contributions) of all the <u>non-[collapsed](#collapsed-flex-item)</u> [flex items](#flex-item) in the [flex container](#flex-container).

- <a id="ref-for-cross-size③④"></a>

  <a id="ref-for-flex-base-size②③"></a>

  <a id="change-2018-used-cross-size"></a> Clarified that the [flex base size](#flex-base-size) calculations rely on the <em>used</em> [cross size](#cross-size). ([Issue 3736](https://github.com/w3c/csswg-drafts/issues/3736))

  > … then the flex base size is calculated from its <u>used</u> inner cross size and the flex item’s intrinsic aspect ratio.

- <a id="change-2018-flexible-length-text"></a> Slightly restructured the prose in [§ 9.7 Resolving Flexible Lengths](#resolve-flexible-lengths) to fix errors. ([Issue 5179](https://github.com/w3c/csswg-drafts/issues/5179))

- <a id="change-2018-cross-layout"></a> Clarified that “performing layout” means using the block-level layout rules. ([Issue 5188](https://github.com/w3c/csswg-drafts/issues/5188))

  > <a id="ref-for-in-flow①"></a>
  >
  > <a id="ref-for-block-level-box①"></a>
  >
  > <a id="ref-for-main-size④③"></a>
  >
  > <strong>Determine the hypothetical cross size of each item</strong> by performing layout <u>as if it were an [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) [block-level box](https://www.w3.org/TR/css-display-4/#block-level-box)</u> with the used [main size](#main-size) and the <u>given</u> available space …

- <a id="ref-for-flex-line⑨"></a>

  <a id="ref-for-cross-size③⑤"></a>

  <a id="ref-for-formatting-context①"></a>

  <a id="change-2018-cross-sizing"></a> Rewrote flex-contain cross-sizing step to depend on the rules of whatever formatting context it’s in, rather than assuming block-layout rules. ([Issue 5190](https://github.com/w3c/csswg-drafts/issues/5190))

  > <strong>Determine the flex container’s used cross size</strong> <u>using the rules of the [formatting context](https://www.w3.org/TR/css-display-4/#formatting-context) in which it participates. If a content-based [cross size](#cross-size) is needed, use the sum of the [flex lines](#flex-line)' <a id="ref-for-cross-size③⑥"></a>cross sizes.</u>
  >
  > - <a id="ref-for-flex-container⑦⑨"></a>
  >
  >   <a id="ref-for-cross-size③⑦"></a>
  >
  >   <a id="ref-for-definite②⑨"></a>
  >
  >   If the cross size property is a [definite](#definite) size, use that, clamped by the used min and max [cross sizes](#cross-size) of the [flex container](#flex-container).
  >
  > - <a id="ref-for-flex-container⑧⓪"></a>
  >
  >   <a id="ref-for-cross-size③⑧"></a>
  >
  >   Otherwise, use the sum of the flex lines' cross sizes, clamped by the used min and max [cross sizes](#cross-size) of the [flex container](#flex-container).

- <a id="ref-for-propdef-aspect-ratio②"></a>

  <a id="change-2018-aspect-ratio-box"></a> Removed explicit mention of which box to use for calculating aspect-ratio, to allow for behavior introduced by the new [aspect-ratio](https://www.w3.org/TR/css-sizing-4/#propdef-aspect-ratio) property in [\[css-sizing-4\]](#biblio-css-sizing-4). ([Issue 5246](https://github.com/w3c/csswg-drafts/issues/5246/))

  > then the flex base size is calculated from its used ~~inner~~ cross size and the flex item’s intrinsic aspect ratio.

- <a id="ref-for-intrinsic-size-contribution②"></a>

  <a id="ref-for-content-based-minimum-size⑦"></a>

  <a id="ref-for-intrinsic-size-contribution①"></a>

  <a id="ref-for-content-based-minimum-size⑥"></a>

  <a id="change-2018-auto-min-contribution"></a> Clarified that the [content-based minimum size](#content-based-minimum-size) of a flex item is a type of [intrinsic size contribution](https://www.w3.org/TR/css-sizing-3/#intrinsic-size-contribution), and thus impacted by associated rules in [\[CSS-SIZING-3\]](#biblio-css-sizing-3). ([Issue 5665](https://github.com/w3c/csswg-drafts/issues/5665#issuecomment-738391191))

  > Note: The [content-based minimum size](#content-based-minimum-size) is a type of [intrinsic size contribution](https://www.w3.org/TR/css-sizing-3/#intrinsic-size-contribution), and thus the provisions in [CSS Sizing 3 § 5.2 Intrinsic Contributions](https://www.w3.org/TR/css-sizing-3/#intrinsic-contribution) apply.

- <a id="change-2018-safe-fallback"></a> Made the fallback alignment (in case of negative free space) for the space-\* keywords apply safe. ([Issue 10154](https://github.com/w3c/csswg-drafts/issues/10154))

- <a id="ref-for-content-based-minimum-size⑧"></a>

  <a id="change-2018-content-based-editorial"></a> Redrafted the definition of [content-based minimum size](#content-based-minimum-size) for easier reading. (Editorial)

- <a id="change-2018-webkit-aliases"></a> Added [Appendix B: -webkit- Legacy Properties](#webkit-aliases) to document `-webkit-` aliases of flex layout properties that are necessary for Web-compat. ([Issue 5634](https://github.com/w3c/csswg-drafts/issues/5634))

- <a id="ref-for-computed-value④"></a>

  <a id="change-2018-blockification"></a> Rephrased blockification rules in terms of [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value). ([Issue 4065](https://github.com/w3c/csswg-drafts/issues/4065))

- Various minor editorial fixes and clarifications,

### <a id="changes-20171016"></a> Changes since the 16 October 2017 CR

A [Disposition of Comments](https://drafts.csswg.org/css-flexbox-1/issues-cr-2017) is also available.

- <a id="change-2017-margin-padding-percent"></a> Removed the option for flex-item block-axis margins and paddings to be resolved against the block dimension; they must be resolved against the inline dimension, as for blocks. ([Issue 2085](https://github.com/w3c/csswg-drafts/issues/2085))

- <a id="ref-for-max-main-size-property"></a>

  <a id="ref-for-min-main-size-property"></a>

  <a id="ref-for-flex-base-size②④"></a>

  <a id="ref-for-valdef-width-auto①②"></a>

  <a id="ref-for-propdef-height①①"></a>

  <a id="ref-for-propdef-width①⑥"></a>

  <a id="ref-for-preferred-size⑦"></a>

  <a id="ref-for-min-content①⓪"></a>

  <a id="ref-for-flex-item①③⓪"></a>

  <a id="ref-for-min-content-contribution④"></a>

  <a id="ref-for-preferred-size⑥"></a>

  <a id="ref-for-min-content-contribution③"></a>

  <a id="ref-for-flex-item①②⑨"></a>

  <a id="change-2017-flex-min-contribution"></a> Floored [flex item](#flex-item)’s [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) by their [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) ([Issue 2353](https://github.com/w3c/csswg-drafts/issues/2353)) and cleaned up associated wording to be more precise.

  > The main-size [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) of a [flex item](#flex-item) is the larger of its <em>outer</em> [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content) <u>and outer [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) (its [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)/[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) as appropriate) if that is not [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto)</u> , clamped by its [flex base size](#flex-base-size) as a maximum (if it is not growable) and/or as a minimum (if it is not shrinkable), and then further clamped by its [min](#min-main-size-property)/[max main size properties](#max-main-size-property).
  >
  > <a id="ref-for-max-content-contribution⑦"></a>
  >
  > <a id="ref-for-flex-item①③①"></a>
  >
  > <a id="ref-for-max-content①②"></a>
  >
  > <a id="ref-for-propdef-width①⑦"></a>
  >
  > <a id="ref-for-propdef-height①②"></a>
  >
  > <a id="ref-for-valdef-width-auto①③"></a>
  >
  > <a id="ref-for-flex-base-size②⑤"></a>
  >
  > <a id="ref-for-min-main-size-property①"></a>
  >
  > <a id="ref-for-max-main-size-property①"></a>
  >
  > The main-size [max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) of a [flex item](#flex-item) is the larger of its <em>outer</em> [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content) and <u>outer</u> ~~specified~~ <u>preferred</u> size (its [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)/[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) as appropriate ~~,~~ <u>)</u> if that is ~~definite~~ <u>not [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto)</u> , clamped by its [flex base size](#flex-base-size) as a maximum (if it is not growable) and/or as a minimum (if it is not shrinkable), and then further clamped by its [min](#min-main-size-property)/[max main size properties](#max-main-size-property).

- <a id="ref-for-propdef-flex-basis①⑥"></a>

  <a id="change-2017-content-desc"></a> Added some (effectively informative) prose and a cross-reference to more clearly define [flex-basis: content](#propdef-flex-basis).

  > <a id="ref-for-flex-item①③②"></a>
  >
  > <a id="ref-for-max-content①③"></a>
  >
  > Indicates ~~automatic sizing~~ <u>an [automatic size](#algo-main-item)</u> based on the [flex item](#flex-item)’s content. <u>(It is typically equivalent to the [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content), but with adjustments to handle aspect ratios, intrinsic sizing constraints, and orthogonal flows; see [details](#algo-main-item) in [§ 9 Flex Layout Algorithm](#layout-algorithm).)</u>

- <a id="ref-for-automatic-minimum-size⑤"></a>

  <a id="ref-for-propdef-min-height⑤"></a>

  <a id="ref-for-propdef-min-width⑥"></a>

  <a id="ref-for-valdef-width-auto①④"></a>

  <a id="change-2017-min-size-auto"></a> Moved the definition of the [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) keyword for [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width) and [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) to [\[CSS-SIZING-3\]](#biblio-css-sizing-3). The definition of what an [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size) for flex items is remains here. ([Issue 1920](https://github.com/w3c/csswg-drafts/issues/1920), [Issue 2103](https://github.com/w3c/csswg-drafts/issues/2103))

- <a id="ref-for-resolved-value"></a>

  <a id="ref-for-propdef-min-height⑥"></a>

  <a id="ref-for-propdef-min-width⑦"></a>

  <a id="ref-for-valdef-width-auto①⑤"></a>

  <a id="change-2017-min-size-auto-computation"></a> Altered the computation of [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) in [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width) and [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) such that it always computes to itself—​although its [resolved value](https://www.w3.org/TR/cssom-1/#resolved-value) remains zero on CSS2 display types. ([Issue 2230](https://github.com/w3c/csswg-drafts/issues/2230), [Issue 2248](https://github.com/w3c/csswg-drafts/issues/2248))

- <a id="ref-for-min-content①①"></a>

  <a id="ref-for-valdef-table-layout-auto"></a>

  <a id="ref-for-used-value④"></a>

  <a id="change-2017-used-min-sizes"></a> Clarified that min/max clamping is according to the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of the min/max size properties—​which in the case of tables with [auto](https://drafts.csswg.org/css2/#valdef-table-layout-auto) layout, is floored by the table’s [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content). ([Issue 2442](https://github.com/w3c/csswg-drafts/issues/2442))

- <a id="ref-for-order-modified-document-order③"></a>

  <a id="change-2017-break-propagation"></a> Clarified that break propagation does not affect computed values and that [order-modified document order](https://www.w3.org/TR/css-flexbox-1/#order-modified-document-order) is used. ([Issue 2614](https://github.com/w3c/csswg-drafts/issues/2614))

  > <a id="ref-for-order-modified-document-order④"></a>
  >
  > The exact layout of a fragmented flex container is not defined in this level of Flexible Box Layout. However, breaks inside a flex container are subject to the following rules <u>(interpreted using [order-modified document order](https://www.w3.org/TR/css-flexbox-1/#order-modified-document-order))</u> :
  >
  > - <a id="ref-for-propdef-break-after③"></a>
  >
  >   <a id="ref-for-propdef-break-before④"></a>
  >
  >   In a row flex container, the [break-before](https://www.w3.org/TR/css-break-3/#propdef-break-before) and [break-after](https://www.w3.org/TR/css-break-3/#propdef-break-after) values on flex items are propagated to the flex line. The <a id="ref-for-propdef-break-before⑤"></a>break-before values on the first line and the <a id="ref-for-propdef-break-after④"></a>break-after values on the last line are propagated to the flex container.
  >
  >   <a id="ref-for-propdef-text-decoration①"></a>
  >
  >   <a id="ref-for-computed-value⑤"></a>
  >
  >   > <strong data-conversion-semantic="note">Note</strong>
  >   >
  >   > Note: Break propagation (like [text-decoration](https://www.w3.org/TR/css-text-decor-4/#propdef-text-decoration) propagation) does not affect [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value).

- <a id="ref-for-content-based-minimum-size⑨"></a>

  <a id="ref-for-automatic-minimum-size⑥"></a>

  <a id="change-2017-auto-min-zero"></a> Clarified that if the [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size) resolves directly to zero rather than being a [content-based minimum size](#content-based-minimum-size), it does not cause indefiniteness.

  > <a id="ref-for-min-content①②"></a>
  >
  > <a id="ref-for-propdef-width①⑧"></a>
  >
  > <a id="ref-for-definite③⓪"></a>
  >
  > For the purpose of calculating an intrinsic size of the element (e.g. the element’s [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content)), ~~this value~~ <u>a content-based minimum size</u> causes the element’s size in that axis to become indefinite (even if e.g. its [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) property specifies a [definite](#definite) size).

- <a id="change-2017-auto-min-size"></a> Editorial improvements to the [automatic minimum size](#min-size-auto). ([Issue 2385](https://github.com/w3c/csswg-drafts/issues/2385))

- Some minor editorial fixes and clarifications, including updates to vocabulary to match updates to other CSS modules.

### <a id="changes-20160526"></a> Changes since the 26 May 2016 CR

A [Disposition of Comments](https://drafts.csswg.org/css-flexbox-1/issues-cr-20160526) is also available.

#### <a id="change-201605-substantive"></a> Substantive Changes and Bugfixes

- <a id="ref-for-target-main-size⑨"></a>

  <a id="ref-for-flex-base-size②⑥"></a>

  <a id="change-2016-box-sizing-basis"></a> To allow flex factors to actually represent absolute ratios of flex item sizes as was originally intended (see various examples), removed the flooring of content-box sizes at zero for the purpose of finding the item’s [flex base size](#flex-base-size), since this type of ratio requires a <a id="ref-for-flex-base-size②⑦"></a>flex base size of zero, which would otherwise only be possible if margins, borders, and padding are also all zero. (The flooring remains in effect, alongside the min and max size constraints, in calculating the hypothetical and final sizes of the item.) ([Issue 316](https://github.com/w3c/csswg-drafts/issues/316))

  > <a id="ref-for-flex-base-size②⑧"></a>
  >
  > <a id="ref-for-propdef-box-sizing③"></a>
  >
  > <a id="ref-for-flex-base-size②⑨"></a>
  >
  > When determining the [flex base size](#flex-base-size), the item’s min and max main size properties are ignored (no clamping occurs). <u>Furthermore, the sizing calculations that floor the content box size at zero when applying [box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing) are also ignored. (For example, an item with a specified size of zero, positive padding, and <a id="ref-for-propdef-box-sizing④"></a>box-sizing: border-box will have an outer [flex base size](#flex-base-size) of zero—​and hence a negative inner <a id="ref-for-flex-base-size③⓪"></a>flex base size.)</u>
  >
  > <a id="ref-for-hypothetical-main-size⑤"></a>
  >
  > <a id="ref-for-flex-base-size③①"></a>
  >
  > The [hypothetical main size](#hypothetical-main-size) is the item’s [flex base size](#flex-base-size) clamped according to its min and max main size properties <u>(and flooring the content box size at zero)</u> .

  > <strong>Fix min/max violations.</strong> Clamp each non-frozen item’s [target main size](#target-main-size) by its min and max main size properties <u>and floor its content-box size at zero</u> . If the item’s <a id="ref-for-target-main-size①⓪"></a>target main size was made smaller by this, it’s a max violation. If the item’s <a id="ref-for-target-main-size①①"></a>target main size was made larger by this,

- <a id="ref-for-max-content-contribution⑧"></a>

  <a id="change-2016-max-contribution"></a> To prevent empty flex items in shrink-to-fit containers from collapsing to zero even when given a specified size, the specified size is now accounted for in calculating its [max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) in [§ 9.9.3 Flex Item Intrinsic Size Contributions](#intrinsic-item-contributions). ([Issue 1435](https://github.com/w3c/csswg-drafts/issues/1435))

  > <a id="ref-for-max-content-contribution⑨"></a>
  >
  > <a id="ref-for-flex-item①③③"></a>
  >
  > <a id="ref-for-max-content①④"></a>
  >
  > <a id="ref-for-propdef-width①⑨"></a>
  >
  > <a id="ref-for-propdef-height①③"></a>
  >
  > <a id="ref-for-definite③①"></a>
  >
  > <a id="ref-for-flex-base-size③②"></a>
  >
  > <a id="ref-for-min-main-size-property②"></a>
  >
  > <a id="ref-for-max-main-size-property②"></a>
  >
  > The <strong>main-size <a href="https://www.w3.org/TR/css-sizing-3/#max-content-contribution">max-content contribution</a> of a <a href="#flex-item">flex item</a></strong> is <u>the larger of</u> its <em>outer</em> [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content) <u>and specified size (its [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)/[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) as appropriate, if that is [definite](#definite))</u> , clamped by its [flex base size](#flex-base-size) as a maximum (if it is not growable) and/or as a minimum (if it is not shrinkable), and then further clamped by its [min](#min-main-size-property)/[max main size properties](#max-main-size-property).

- <a id="change-2016-definite-basis"></a> Since at least two implementations ended up allowing percentages inside flex items with indefinite flex basis to resolve anyway, removed the condition requiring definite flex basis. ([Issue 1679](https://github.com/w3c/csswg-drafts/issues/1679))

  > <a id="ref-for-flex-item①③④"></a>
  >
  > <a id="ref-for-definite③②"></a>
  >
  > <a id="ref-for-flex-flex-basis①②"></a>
  >
  > <a id="ref-for-flex-container⑧①"></a>
  >
  > <a id="ref-for-definite③③"></a>
  >
  > <a id="ref-for-main-size④④"></a>
  >
  > <a id="ref-for-flex-item①③⑤"></a>
  >
  > <a id="ref-for-definite③⑤"></a>
  >
  > If ~~a [flex item](#flex-item) has a [definite](#definite) [flex basis](#flex-flex-basis) and~~ the [flex container](#flex-container) has a [definite](#definite) [main size](#main-size), ~~its~~ <u>a [flex item](#flex-item)’s</u> post-flexing main size is treated as <a id="ref-for-definite③④"></a>definite (even though it might technically rely on the ~~sizes of [indefinite](#definite) siblings to resolve its flexed main size~~ the <a id="ref-for-definite③⑥"></a>indefinite sizes of any flex items in the same line).

- <a id="ref-for-propdef-align-self②③"></a>

  <a id="ref-for-valdef-align-items-auto②"></a>

  <a id="change-2016-align-self-as-specified"></a> For ease of implementation, [auto](#valdef-align-items-auto) value of [align-self](#propdef-align-self) now computes to itself always. See [related previous change](#change-2015-align-self-auto) requiring this computation for absolutely-positioned elements. ([Issue 440](https://github.com/w3c/csswg-drafts/issues/440), [Issue 644](https://github.com/w3c/csswg-drafts/issues/644))

  > | Field               | Definition                                                                                                                                                         |
  > |---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------|
  > | <strong>Computed value: &#xA;         </strong> | <a id="ref-for-propdef-align-items⑦"></a><a id="ref-for-valdef-align-items-auto③"></a> ~~[auto](#valdef-align-items-auto) computes to parent’s [align-items](#propdef-align-items) value; otherwise~~ as specified |
  >
  > …
  >
  > <a id="ref-for-valdef-align-items-auto④"></a>
  >
  > <a id="ref-for-propdef-align-self②④"></a>
  >
  > <a id="ref-for-propdef-align-items⑧"></a>
  >
  > <a id="ref-for-valdef-align-items-stretch①"></a>
  >
  > ~~On absolutely positioned elements, a value of [auto](#valdef-align-items-auto) computes to itself. On all other elements, a value of <a id="ref-for-valdef-align-items-auto⑤"></a>auto for [align-self](#propdef-align-self) computes to the value of [align-items](#propdef-align-items) on the element’s parent, or [stretch](#valdef-align-items-stretch) if the element has no parent.~~

- <a id="ref-for-flex-item①③⑥"></a>

  <a id="change-2016-baseline-alignment"></a> Change [flex items](#flex-item) in orthogonal flows and <a id="ref-for-flex-item①③⑦"></a>flex items without a baseline to both synthesize their alignment baseline from the <a id="ref-for-flex-item①③⑧"></a>flex item’s border box. ([Issue 373](https://github.com/w3c/csswg-drafts/issues/373))

- <a id="ref-for-cross-axis-baseline①"></a>

  <a id="change-2016-main-cross-baseline"></a> Fix main/cross error in definition of [cross-axis baseline set](#cross-axis-baseline). ([Issue 792](https://github.com/w3c/csswg-drafts/issues/792))

  > Otherwise, the flex container has no first/last ~~main~~ <u>cross</u> -axis baseline set…

- <a id="change-2016-table-wrapper"></a> Restore accidentally-deleted text about tables as flex items. See [anonymous box change](#change-2015-anonymous-fixup). ([Issue 547](https://github.com/w3c/csswg-drafts/issues/547))

  > <a id="ref-for-propdef-display①⑤"></a>
  >
  > <a id="ref-for-flex-item①③⑨"></a>
  >
  > <a id="ref-for-propdef-order①①"></a>
  >
  > <a id="ref-for-propdef-align-self②⑤"></a>
  >
  > <a id="ref-for-propdef-width②⓪"></a>
  >
  > <a id="ref-for-propdef-height①④"></a>
  >
  > <a id="ref-for-propdef-flex③①"></a>
  >
  > In the case of flex items with [display: table](https://www.w3.org/TR/css-display-4/#propdef-display), the table wrapper box becomes the [flex item](#flex-item), and the [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) and [align-self](#propdef-align-self) properties apply to it. The contents of any caption boxes contribute to the calculation of the table wrapper box’s min-content and max-content sizes. However, like [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height), the [flex](#propdef-flex) longhands apply to the table box as follows: the <a id="ref-for-flex-item①④⓪"></a>flex item’s final size is calculated by performing layout as if the distance between the table wrapper box’s edges and the table box’s content edges were all part of the table box’s border+padding area, and the table box were the <a id="ref-for-flex-item①④①"></a>flex item.

- <a id="change-2016-auto-margin-abspos"></a> Clarified that auto margins are treated as zero for the purpose of calculating a absolutely-positioned flex container child’s static position. ([Issue 665](https://github.com/w3c/csswg-drafts/issues/665))

  > <a id="ref-for-propdef-align-self②⑥"></a>
  >
  > <a id="ref-for-valdef-self-position-start"></a>
  >
  > For this purpose, a value of [align-self: auto](#propdef-align-self) is treated identically to [start](https://www.w3.org/TR/css-align-3/#valdef-self-position-start) <u>, and auto margins are treated as zero</u> .

- <a id="ref-for-min-main-size-property③"></a>

  <a id="ref-for-max-main-size-property③"></a>

  <a id="change-2016-floor-intrinsic-size"></a> When clamping by the [max main size property](#max-main-size-property) in the [calculation of the flex container’s intrinsic size](#intrinsic-main-sizes), be sure to floor by the [min main size property](#min-main-size-property). ([Issue 361](https://github.com/w3c/csswg-drafts/issues/361))

  > <a id="ref-for-flex-item①④②"></a>
  >
  > <a id="ref-for-flex-base-size③③"></a>
  >
  > <a id="ref-for-flex-flex-grow-factor①①"></a>
  >
  > <a id="ref-for-scaled-flex-shrink-factor④"></a>
  >
  > Within each line, find the largest <var>max-content flex fraction</var> among all the [flex items](#flex-item). Add each item’s [flex base size](#flex-base-size) to the product of its [flex grow factor](#flex-flex-grow-factor) (or [scaled flex shrink factor](#scaled-flex-shrink-factor), if the chosen <var>max-content flex fraction</var> was negative) and the chosen <var>max-content flex fraction</var>, then clamp that result <u>ing item size according to</u> ~~by~~ the max <u>and min</u> main size propert ~~y~~ <u>ies</u> .

- <a id="ref-for-flex-item①④⑤"></a>

  <a id="ref-for-flex-container⑧④"></a>

  <a id="ref-for-propdef-order①②"></a>

  <a id="change-2016-abspos-no-order-fix"></a> Added missing edits for [change](#change-2016-abspos-no-order) that made [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) not apply to absolutely-positioned children of a flex container. ([Issue 1439](https://github.com/w3c/csswg-drafts/issues/1439))

  > | Field               | Definition                                                                                                                                   |
  > |---------------------|----------------------------------------------------------------------------------------------------------------------------------------------|
  > | <strong>Applies to: &#xA;         </strong> | <a id="ref-for-flex-container⑧②"></a><a id="ref-for-flex-item①④③"></a> [flex items](#flex-item) ~~and absolutely-positioned children of [flex containers](#flex-container)~~ |

  > <a id="ref-for-propdef-order①③"></a>
  >
  > The [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) property controls the order in which ~~children of a flex container~~ <u>flex items</u> appear within the flex container, by assigning them to ordinal groups. …
  >
  > <a id="ref-for-flex-container⑧③"></a>
  >
  > <a id="ref-for-propdef-order①④"></a>
  >
  > <a id="ref-for-flex-item①④④"></a>
  >
  > <u>Absolutely-positioned children of a [flex container](#flex-container) are treated as having [order: 0](https://www.w3.org/TR/css-flexbox-1/#propdef-order) for the purpose of determining their painting order relative to [flex items](#flex-item).</u>

  > Unless otherwise specified by a future specification, this property has no effect on boxes that are not ~~children of a [flex container](#flex-container)~~ <u>[flex items](#flex-item)</u> .

- <a id="ref-for-propdef-flex-direction⑧"></a>

  <a id="change-2016-flex-direction-baseline"></a> Take [flex-direction](#propdef-flex-direction) into account when determining first/last baseline of the flex container. ([Issue 995](https://github.com/w3c/csswg-drafts/issues/995))

  > <a id="ref-for-propdef-order①⑤"></a>
  >
  > <a id="ref-for-propdef-flex-direction⑨"></a>
  >
  > To this end, the baselines of a flex container are determined as follows (after reordering with [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) <u>, and taking [flex-direction](#propdef-flex-direction) into account</u> ):
  >
  > …
  >
  > 1.  <a id="ref-for-main-axis-baseline②"></a>
  >
  >     <a id="ref-for-baseline-participation②"></a>
  >
  >     <a id="ref-for-flex-line①⓪"></a>
  >
  >     <a id="ref-for-flex-item①④⑥"></a>
  >
  >     If any of the [flex items](#flex-item) on the flex container’s ~~first/last~~ <u>startmost/endmost</u> [flex line](#flex-line) [participate in baseline alignment](#baseline-participation), the flex container’s first/last [main-axis baseline set](#main-axis-baseline) …
  >
  > 2.  <a id="ref-for-alignment-baseline⑤"></a>
  >
  >     <a id="ref-for-generate-baselines③"></a>
  >
  >     <a id="ref-for-main-axis-baseline③"></a>
  >
  >     <a id="ref-for-flex-item①④⑦"></a>
  >
  >     Otherwise, if the flex container has at least one [flex item](#flex-item), the flex container’s first/last [main-axis baseline set](#main-axis-baseline) is [generated](https://www.w3.org/TR/css-align-3/#generate-baselines) from the [alignment baseline](https://www.w3.org/TR/css-align-3/#alignment-baseline) of the ~~first/last~~ <u>startmost/endmost</u> <a id="ref-for-flex-item①④⑧"></a>flex item. …
  >
  > 3.  <a id="ref-for-baseline-set②"></a>
  >
  >     Otherwise, the flex container has no first/last main-axis [baseline set](https://www.w3.org/TR/css-align-3/#baseline-set), …
  >
  > …
  >
  > 1.  <a id="ref-for-alignment-baseline⑥"></a>
  >
  >     <a id="ref-for-generate-baselines④"></a>
  >
  >     <a id="ref-for-cross-axis-baseline②"></a>
  >
  >     <a id="ref-for-flex-item①④⑨"></a>
  >
  >     If the flex container has at least one [flex item](#flex-item), the flex container’s first/last [cross-axis baseline set](#cross-axis-baseline) is [generated](https://www.w3.org/TR/css-align-3/#generate-baselines) from the [alignment baseline](https://www.w3.org/TR/css-align-3/#alignment-baseline) of the ~~first/last~~ <u>startmost/endmost</u> <a id="ref-for-flex-item①⑤⓪"></a>flex item. …

- <a id="ref-for-valdef-self-position-start①"></a>

  <a id="ref-for-flex-line①①"></a>

  <a id="ref-for-propdef-align-content①①"></a>

  <a id="change-2016-space-between-single-line"></a> Define [align-content: space-between](#propdef-align-content) handling of a single [flex line](#flex-line) as equivalent to [start](https://www.w3.org/TR/css-align-3/#valdef-self-position-start). ([Issue 718](https://github.com/w3c/csswg-drafts/issues/718))

  > <a id="ref-for-flex-line①②"></a>
  >
  > <a id="ref-for-valdef-align-content-flex-start①"></a>
  >
  > Lines are evenly distributed in the flex container. If the leftover free-space is negative <u>or there is only a single [flex line](#flex-line) in the flex container,</u> this value is identical to [flex-start](#valdef-align-content-flex-start).

- <a id="change-2016-axis-error"></a> Fixed error in [axis mapping table](#axis-mapping). ([Issue 205](https://github.com/w3c/csswg-drafts/issues/205))

- <a id="ref-for-specified-size"></a>

  <a id="ref-for-automatic-minimum-size⑦"></a>

  <a id="change-2016-reword-min-auto-fix"></a> Restored definition of the [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size) of boxes with neither [specified size](https://www.w3.org/TR/css-images-3/#specified-size) nor aspect ratio, which was lost in earlier [rewrite](#change-2016-reword-min-auto). ([Issue 671](https://github.com/w3c/csswg-drafts/issues/671))

#### <a id="change-201605-clarify"></a> Clarifications

- <a id="ref-for-flex-item①⑤①"></a>

  <a id="ref-for-flex-container⑧⑤"></a>

  <a id="ref-for-cross-size③⑨"></a>

  <a id="ref-for-main-size④⑤"></a>

  <a id="change-2016-main-cross-def"></a> Made sure that [main size](#main-size) and [cross size](#cross-size) are defined for [flex containers](#flex-container) as well as for [flex items](#flex-item). ([Issue 981](https://github.com/w3c/csswg-drafts/issues/981))

- <a id="change-2016-broken-spatial-note"></a> Tweaked final clarifying sentence of note about spatial navigation. ([Issue 1677](https://github.com/w3c/csswg-drafts/issues/1677))

  > <a id="ref-for-propdef-order①⑥"></a>
  >
  > <a id="ref-for-propdef-order①⑦"></a>
  >
  > <a id="ref-for-propdef-order①⑧"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > User agents, including browsers, accessible technology, and extensions, may offer spatial navigation features. This section does not preclude respecting the [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) property when determining element ordering in such spatial navigation modes; indeed it would need to be considered for such a feature to work. ~~However a UA that uses [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) in determining sequential navigation, but does not otherwise account for spatial relationships among elements (as expressed by the various layout features of CSS including and not limited to flex layout), is non-conforming.~~ <u>But [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) is not the only (or even the primary) CSS property that would need to be considered for such a spatial navigation feature. A well-implemented spatial navigation feature would need to consider all the layout features of CSS that modify spatial relationships.</u>

- Miscellaneous trivial editorial improvements.

### <a id="changes-20160301"></a> Changes since the 1 March 2016 CR

A [Disposition of Comments](https://drafts.csswg.org/css-flexbox-1/issues-cr-20160301) is also available.

#### <a id="change-201603-substantive"></a> Substantive Changes and Bugfixes

- <a id="ref-for-automatic-minimum-size⑧"></a>

  <a id="change-2016-intrinsic-percentage"></a> Define how percentages are handled when calculating intrinsic [automatic minimum sizes](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size). ([Issue 3](https://drafts.csswg.org/css-flexbox-1/issues-cr-20160301#issue-3))

  > <a id="ref-for-min-content①③"></a>
  >
  > <a id="ref-for-propdef-width②①"></a>
  >
  > <a id="ref-for-definite③⑦"></a>
  >
  > <a id="ref-for-valdef-width-auto①⑥"></a>
  >
  > For the purpose of calculating an intrinsic size of the element (e.g. the element’s [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content)), this value causes the element’s size in that axis to become indefinite (even if e.g. its [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) property specifies a [definite](#definite) size). Note this means that percentages calculated against this size will be treated as [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto).
  >
  > <a id="ref-for-valdef-width-min-content①"></a>
  >
  > <a id="ref-for-valdef-width-max-content①"></a>
  >
  > <u>Nonetheless,</u> although this may require an additional layout pass to re-resolve percentages in some cases, this value (like the [min-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content), [max-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-max-content), and fit-content values defined in [\[CSS-SIZING-3\]](#biblio-css-sizing-3)) does not prevent the resolution of percentage sizes within the item.

- <a id="ref-for-definite③⑧"></a>

  <a id="change-2016-definite-indefinite"></a> Switched [definite](#definite) and <a id="ref-for-definite③⑨"></a>indefinite to refer to the (more correct) definitions in [\[CSS-SIZING-3\]](#biblio-css-sizing-3) instead of defining them inline in this module. ([Issue 10](https://drafts.csswg.org/css-flexbox-1/issues-cr-20160301#issue-10))

- <a id="ref-for-propdef-order①⑨"></a>

  <a id="change-2016-abspos-no-order"></a> Abspos children of a flexbox no longer respond to the [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) property. ([Issue 12](https://drafts.csswg.org/css-flexbox-1/issues-cr-20160301#issue-12))

- <a id="ref-for-last-baseline-alignment"></a>

  <a id="ref-for-baseline-set③"></a>

  <a id="change-2016-baseline-sets"></a> Updated [§ 8.5 Flex Container Baselines](#flex-baselines) to account for [baseline sets](https://www.w3.org/TR/css-align-3/#baseline-set) and [last-baseline alignment](https://www.w3.org/TR/css-align-3/#last-baseline-alignment). ([Issue 13](https://drafts.csswg.org/css-flexbox-1/issues-cr-20160301#issue-13)))

#### <a id="change-201603-clarify"></a> Clarifications

- <a id="ref-for-propdef-order②⓪"></a>

  <a id="change-2016-spatial-nav"></a> Clarify that spatial navigation modes are allowed to handle [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order). ([Issue 1](https://drafts.csswg.org/css-flexbox-1/issues-cr-20160301#issue-1))

  > <a id="ref-for-propdef-order②①"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > User agents, including browsers, accessible technology, and extensions, may offer spatial navigation features. This section does not preclude respecting the [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) property when determining element ordering in such spatial navigation modes; indeed it would need to be considered for such a feature to work. However a UA that uses <a id="ref-for-propdef-order②②"></a>order in determining sequential navigation, but does not otherwise account for spatial relationships among elements (as expressed by the various layout features of CSS including and not limited to flex layout), is non-conforming.

- <a id="change-2016-definite-3"></a> Cross-reference an additional case of definiteness in [§ 9.8 Definite and Indefinite Sizes](#definite-sizes) ([Issue 2](https://drafts.csswg.org/css-flexbox-1/issues-cr-20160301#issue-2))

  > Once the cross size of a flex line has been determined, items in auto-sized flex containers are also considered definite for the purpose of layout; see [step 11](#algo-stretch).

- <a id="ref-for-valdef-flex-basis-content①⓪"></a>

  <a id="ref-for-flex-flex-basis①③"></a>

  <a id="change-2016-auto-content-wording"></a> Improve wording for how unresolveable percentage [flex basis](#flex-flex-basis) values transmute to [content](#valdef-flex-basis-content). ([Issue 6](https://drafts.csswg.org/css-flexbox-1/issues-cr-20160301#issue-6))

  > <a id="ref-for-valdef-flex-basis-auto③"></a>
  >
  > <a id="ref-for-valdef-flex-basis-content①①"></a>
  >
  > <a id="ref-for-propdef-flex-basis①⑦"></a>
  >
  > <a id="ref-for-propdef-width②②"></a>
  >
  > <a id="ref-for-valdef-width-auto①⑦"></a>
  >
  > <a id="ref-for-propdef-width②③"></a>
  >
  > <a id="ref-for-valdef-flex-basis-content①②"></a>
  >
  > <a id="ref-for-propdef-flex-basis①⑧"></a>
  >
  > <a id="ref-for-flex-container⑧⑥"></a>
  >
  > <a id="ref-for-definite④⓪"></a>
  >
  > <a id="ref-for-main-size④⑥"></a>
  >
  > <a id="ref-for-valdef-width-auto①⑧"></a>
  >
  > <a id="ref-for-valdef-flex-basis-content①③"></a>
  >
  > <a id="ref-for-propdef-flex-basis②⓪"></a>
  >
  > <a id="ref-for-valdef-flex-basis-content①④"></a>
  >
  > For all values other than [auto](#valdef-flex-basis-auto) and [content](#valdef-flex-basis-content) (defined above), [flex-basis](#propdef-flex-basis) is resolved the same way as [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) in horizontal writing modes [\[CSS2\]](#biblio-css2), <u>except that if a value would resolve to [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) for [width](https://www.w3.org/TR/css-sizing-3/#propdef-width), it instead resolves to [content](#valdef-flex-basis-content) for [flex-basis](#propdef-flex-basis)</u> . For example, percentage values of <a id="ref-for-propdef-flex-basis①⑨"></a>flex-basis are resolved against the flex item’s containing block (i.e. its [flex container](#flex-container)); and if that containing block’s size is [indefinite](#definite), ~~the result is the same as a [main size](#main-size) of [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) (which in this case is treated as [content](#valdef-flex-basis-content))~~ <u>the used value for [flex-basis](#propdef-flex-basis) is [content](#valdef-flex-basis-content)</u> .

- <a id="ref-for-flex-flex-basis①④"></a>

  <a id="ref-for-definite④①"></a>

  <a id="change-2016-inflexible-definite"></a> Clarify that inflexible items with a [definite](#definite) [flex basis](#flex-flex-basis) have a <a id="ref-for-definite④②"></a>definite size. ([Issue 8](https://drafts.csswg.org/css-flexbox-1/issues-cr-20160301#issue-8), [Issue 11](https://drafts.csswg.org/css-flexbox-1/issues-cr-20160301#issue-11))

  > <a id="ref-for-flex-item①⑤②"></a>
  >
  > <a id="ref-for-fully-inflexible②"></a>
  >
  > <a id="ref-for-propdef-flex-grow②⓪"></a>
  >
  > <a id="ref-for-propdef-flex-shrink⑦"></a>
  >
  > <a id="ref-for-flexible"></a>
  >
  > A [flex item](#flex-item) is [fully inflexible](#fully-inflexible) if both its [flex-grow](#propdef-flex-grow) and [flex-shrink](#propdef-flex-shrink) values are zero, and [flexible](#flexible) otherwise.

  > <a id="ref-for-fully-inflexible③"></a>
  >
  > <a id="ref-for-definite④③"></a>
  >
  > <a id="ref-for-flex-flex-basis①⑤"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > The main size of a [fully inflexible](#fully-inflexible) item with a [definite](#definite) [flex basis](#flex-flex-basis) is, by definition, <a id="ref-for-definite④④"></a>definite.

- <a id="ref-for-valdef-width-auto①⑨"></a>

  <a id="change-2016-reword-min-auto"></a> Reworded definition of the [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) value to be easier to understand. ([Issue 9](https://drafts.csswg.org/css-flexbox-1/issues-cr-20160301#issue-9))

  > <a id="ref-for-flex-item①⑤③"></a>
  >
  > <a id="ref-for-propdef-overflow④"></a>
  >
  > <a id="ref-for-valdef-overflow-visible"></a>
  >
  > <a id="ref-for-main-axis①⑨"></a>
  >
  > <a id="ref-for-automatic-minimum-size⑨"></a>
  >
  > On a [flex item](#flex-item) whose [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) is [visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible) in the [main axis](#main-axis), when specified on the <a id="ref-for-flex-item①⑤④"></a>flex item’s main-axis min-size property, ~~the following table gives the minimum size …~~ <u>specifies an [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size).</u>
  >
  > <a id="ref-for-automatic-minimum-size①⓪"></a>
  >
  > In general, the [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size) … defined below:

- <a id="change-2016-clarify-staticpos"></a> Slightly reworded the section on determining the static position of absolutely-positioned children to be clearer.

- <a id="change-2016-animation-type"></a> Adjusted format of Animatable lines to be clearer about animating keywords.

- Miscellaneous trivial editorial improvements.

### <a id="changes-201505"></a> Changes since the 14 May 2015 LCWD

A [Disposition of Comments](https://drafts.csswg.org/css-flexbox-1/issues-lc-20150514) is also available.

#### <a id="change-201505-substantive"></a> Substantive Changes and Bugfixes

- <a id="ref-for-propdef-flex-basis②①"></a>

  <a id="ref-for-propdef-flex③②"></a>

  <a id="change-2015-flex-basis-zero"></a> Revert [flex](#propdef-flex) shorthand [change](#change-2012-flex-basis-zero) of omitted [flex-basis](#propdef-flex-basis) back to 0, since that was a hacky way of solving an intrinsic size problem, and isn’t needed (and gives bad results) given a correct implementation of [§ 9.9 Intrinsic Sizes](#intrinsic-sizes). ([Issue 13](https://drafts.csswg.org/css-flexbox-1/issues-lc-20150514#issue-13))

  > <a id="ref-for-propdef-flex③③"></a>
  >
  > When omitted from the [flex](#propdef-flex) shorthand, its specified value is 0~~%~~.

  > <a id="ref-for-propdef-flex③④"></a>
  >
  > [flex: \<positive-number\>](#propdef-flex)
  >
  > <a id="ref-for-propdef-flex③⑤"></a>
  >
  > Equivalent to [flex: \<positive-number\> 1 0~~%~~](#propdef-flex).

- <a id="ref-for-flex-item②⓪⑥"></a>

  <a id="change-2015-anonymous-fixup"></a> Changed [flex item determination](#flex-item) to operate on each element directly, and not on its anonymous wrapper box, if any. ([Issue 6](https://drafts.csswg.org/css-flexbox-1/issues-lc-20150514#issue-6))

  > - <a id="ref-for-propdef-display①⑥"></a>
  >
  >   <a id="ref-for-propdef-float④"></a>
  >
  >   <a id="ref-for-flex-item①⑤⑥"></a>
  >
  >   <a id="ref-for-propdef-clear③"></a>
  >
  >   <a id="ref-for-propdef-float③"></a>
  >
  >   <a id="ref-for-flex-item①⑤⑤"></a>
  >
  >   <a id="ref-for-propdef-clear②"></a>
  >
  >   <a id="ref-for-propdef-float②"></a>
  >
  >   ~~[float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float) and [clear](https://www.w3.org/TR/CSS2/visuren.html#propdef-clear) have no effect on a [flex item](#flex-item),~~ <u>[float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float) and [clear](https://www.w3.org/TR/CSS2/visuren.html#propdef-clear) do not create floating or clearance of [flex item](#flex-item),</u> and do not take it out-of-flow. ~~However, the [float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float) property can still affect box generation by influencing the [display](https://www.w3.org/TR/css-display-4/#propdef-display) property’s computed value.~~

  > <a id="ref-for-propdef-display①⑦"></a>
  >
  > <a id="ref-for-flex-container⑧⑦"></a>
  >
  > <a id="ref-for-flex-item①⑤⑦"></a>
  >
  > Some values of [display](https://www.w3.org/TR/css-display-4/#propdef-display) trigger the creation of anonymous boxes around the original box. It’s the outermost box—the direct child of the [flex container](#flex-container) box—that becomes a [flex item](#flex-item). For example, given two contiguous child elements with <a id="ref-for-propdef-display①⑧"></a>display: table-cell, the [anonymous table wrapper box generated around them](https://www.w3.org/TR/CSS2/tables.html#anonymous-boxes) [\[CSS2\]](#biblio-css2) becomes the <a id="ref-for-flex-item①⑤⑧"></a>flex item.
  >
  > <a id="ref-for-propdef-display①⑨"></a>
  >
  > <a id="ref-for-flex-item①⑤⑨"></a>
  >
  > <a id="ref-for-propdef-order②③"></a>
  >
  > <a id="ref-for-propdef-align-self②⑦"></a>
  >
  > <a id="ref-for-propdef-width②④"></a>
  >
  > <a id="ref-for-propdef-height①⑤"></a>
  >
  > <a id="ref-for-propdef-flex③⑥"></a>
  >
  > In the case of flex items with [display: table](https://www.w3.org/TR/css-display-4/#propdef-display), the table wrapper box becomes the [flex item](#flex-item), and the [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) and [align-self](#propdef-align-self) properties apply to it. The contents of any caption boxes contribute to the calculation of the table wrapper box’s min-content and max-content sizes. However, like [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height), the [flex](#propdef-flex) longhands apply to the table box as follows: the <a id="ref-for-flex-item①⑥⓪"></a>flex item’s final size is calculated by performing layout as if the distance between the table wrapper box’s edges and the table box’s content edges were all part of the table box’s border+padding area, and the table box were the <a id="ref-for-flex-item①⑥①"></a>flex item.
  >
  > <a id="ref-for-propdef-display②⓪"></a>
  >
  > <a id="ref-for-flex-item①⑥②"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > Note: Some values of [display](https://www.w3.org/TR/css-display-4/#propdef-display) normally trigger the creation of anonymous boxes around the original box. If such a box is a [flex item](#flex-item), it is blockified first, and so anonymous box creation will not happen. For example, two contiguous <a id="ref-for-flex-item①⑥③"></a>flex items with <a id="ref-for-propdef-display②①"></a>display: table-cell will become two separate <a id="ref-for-propdef-display②②"></a>display: block <a id="ref-for-flex-item①⑥④"></a>flex items, instead of being wrapped into a single anonymous table.

- <a id="change-2015-min-auto-intrinsic-percentages"></a> Defined that any size adjustment imposed by a box’s min-width: auto is consulted when percentage-sizing any of its contents. ([Issue 3](https://drafts.csswg.org/css-flexbox-1/issues-lc-20150514#issue-3))

  > <a id="ref-for-valdef-flex-basis-auto④"></a>
  >
  > <a id="ref-for-propdef-min-height⑦"></a>
  >
  > <a id="ref-for-propdef-max-height③"></a>
  >
  > <a id="ref-for-propdef-height①⑥"></a>
  >
  > ~~In order to prevent cycling sizing, the [auto](#valdef-flex-basis-auto) value of [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) and [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height) does not factor into the percentage size resolution of the box’s contents. For example, a percentage-height block whose flex item parent has [height: 120em; min-height: auto](https://www.w3.org/TR/css-sizing-3/#propdef-height) will size itself against <a id="ref-for-propdef-height①⑦"></a>height: 120em regardless of the impact that <a id="ref-for-propdef-min-height⑧"></a>min-height might have on the used size of the flex item.~~
  >
  > <a id="ref-for-valdef-width-auto②⓪"></a>
  >
  > <a id="ref-for-propdef-min-width⑧"></a>
  >
  > <a id="ref-for-propdef-min-height⑨"></a>
  >
  > <a id="ref-for-valdef-width-min-content②"></a>
  >
  > <a id="ref-for-valdef-width-max-content②"></a>
  >
  > <u>Although this may require an additional layout pass to re-resolve percentages in some cases, the [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) value of [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width) and [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) (like the [min-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content), [max-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-max-content), and fit-content values defined in [\[CSS-SIZING-3\]](#biblio-css-sizing-3)) does not prevent the resolution of percentage sizes within the item.</u>

- <a id="change-2015-intrinsic-items"></a> Correct intrinsic sizing rules to handle inflexible items. ([Issue 1](https://drafts.csswg.org/css-flexbox-1/issues-lc-20150514#issue-1))

  > <a id="ref-for-min-content-contribution⑤"></a>
  >
  > <a id="ref-for-max-content-contribution①⓪"></a>
  >
  > <a id="ref-for-flex-item①⑥⑤"></a>
  >
  > <a id="ref-for-hypothetical-main-size⑥"></a>
  >
  > <a id="ref-for-min-content-constraint②"></a>
  >
  > <a id="ref-for-max-content-constraint③"></a>
  >
  > <a id="ref-for-min-content-contribution⑥"></a>
  >
  > <a id="ref-for-max-content-contribution①①"></a>
  >
  > <a id="ref-for-flex-item①⑥⑥"></a>
  >
  > <a id="ref-for-min-content①④"></a>
  >
  > <a id="ref-for-max-content①⑤"></a>
  >
  > <a id="ref-for-flex-base-size③④"></a>
  >
  > <a id="ref-for-min-main-size-property④"></a>
  >
  > <a id="ref-for-max-main-size-property④"></a>
  >
  > The main-size [min-content](https://www.w3.org/TR/css-sizing-3/#min-content-contribution)/[max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) of a [flex item](#flex-item) is its outer ~~[hypothetical main size](#hypothetical-main-size) when sized under a [min-content](https://www.w3.org/TR/css-sizing-3/#min-content-constraint)/[max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint) (respectively)~~ <u>The <strong>main-size <a href="https://www.w3.org/TR/css-sizing-3/#min-content-contribution">min-content</a>/<a href="https://www.w3.org/TR/css-sizing-3/#max-content-contribution">max-content contribution</a> of a <a href="#flex-item">flex item</a></strong> is its <em>outer</em> [min-content](https://www.w3.org/TR/css-sizing-3/#min-content)/[max-content size](https://www.w3.org/TR/css-sizing-3/#max-content), clamped by its [flex base size](#flex-base-size) as a maximum (if it is not growable) and/or as a minimum (if it is not shrinkable), and then further clamped by its [min](#min-main-size-property)/[max main size properties](#max-main-size-property)</u> .

- <a id="ref-for-flex-container⑧⑧"></a>

  <a id="change-2015-intrinsic-main-container"></a> Correct errors in [flex container](#flex-container) main-axis intrinsic sizing. ([Issue 1](https://drafts.csswg.org/css-flexbox-1/issues-lc-20150514#issue-1))

  > <a id="ref-for-max-content①⑥"></a>
  >
  > <a id="ref-for-main-size④⑦"></a>
  >
  > <a id="ref-for-flex-container⑧⑨"></a>
  >
  > <a id="ref-for-max-content-contribution①②"></a>
  >
  > <a id="ref-for-flex-item①⑥⑦"></a>
  >
  > The [max-content](https://www.w3.org/TR/css-sizing-3/#max-content) [main size](#main-size) of a [flex container](#flex-container) is the smallest size the <a id="ref-for-flex-container⑨⓪"></a>flex container can take while maintaining the [max-content contributions](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) of its [flex items](#flex-item):
  >
  > 1.  <a id="ref-for-flex-flex-shrink-factor⑧"></a>
  >
  >     <a id="ref-for-scaled-flex-shrink-factor⑥"></a>
  >
  >     <a id="ref-for-flex-flex-grow-factor①③"></a>
  >
  >     <a id="ref-for-flex-flex-shrink-factor⑦"></a>
  >
  >     <a id="ref-for-scaled-flex-shrink-factor⑤"></a>
  >
  >     <a id="ref-for-flex-flex-grow-factor①②"></a>
  >
  >     <a id="ref-for-max-content-contribution①③"></a>
  >
  >     <a id="ref-for-flex-base-size③⑤"></a>
  >
  >     <a id="ref-for-flex-item①⑥⑧"></a>
  >
  >     For each [flex item](#flex-item), subtract its <u>outer</u> [flex base size](#flex-base-size) from its [max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) size ~~, then divide by its [flex grow factor](#flex-flex-grow-factor), floored at 1, or by its [scaled flex shrink factor](#scaled-flex-shrink-factor) (if the result was negative, flooring the [flex shrink factor](#flex-flex-shrink-factor) at 1 if necessary)~~ . <u>If that result is not zero, divide it by (if the result was positive) its [flex grow factor](#flex-flex-grow-factor) floored at 1, or (if the result was negative) by its [scaled flex shrink factor](#scaled-flex-shrink-factor), having floored the [flex shrink factor](#flex-flex-shrink-factor) at 1.</u> This is the item’s <var>max-content flex fraction</var>.

- <a id="ref-for-flex-container⑨①"></a>

  <a id="change-2015-intrinsic-cross-container"></a> Correct errors in [flex container](#flex-container) cross-axis intrinsic sizing, and specify commonly-implemented min-content sizing heuristic for multi-line column flex containers. ([Issue 12](https://drafts.csswg.org/css-flexbox-1/issues-lc-20150514#issue-12))

  > <a id="ref-for-min-content①⑤"></a>
  >
  > <a id="ref-for-cross-size④⓪"></a>
  >
  > <a id="ref-for-max-content①⑦"></a>
  >
  > <a id="ref-for-main-axis②⓪"></a>
  >
  > <a id="ref-for-cross-axis②②"></a>
  >
  > The [min-content](https://www.w3.org/TR/css-sizing-3/#min-content) [cross size](#cross-size) and [max-content](https://www.w3.org/TR/css-sizing-3/#max-content) <a id="ref-for-cross-size④①"></a>cross size of a flex container are the <a id="ref-for-cross-size④②"></a>cross size of the flex container after performing layout into the given available [main-axis](#main-axis) space and infinite available [cross-axis](#cross-axis) space.
  >
  > <a id="ref-for-min-content①⑥"></a>
  >
  > <a id="ref-for-max-content①⑧"></a>
  >
  > <a id="ref-for-cross-size④③"></a>
  >
  > <a id="ref-for-single-line-flex-container①⑤"></a>
  >
  > <a id="ref-for-flex-container⑨②"></a>
  >
  > <a id="ref-for-min-content-contribution⑦"></a>
  >
  > <a id="ref-for-max-content-contribution①④"></a>
  >
  > <a id="ref-for-flex-item①⑥⑨"></a>
  >
  > The [min-content](https://www.w3.org/TR/css-sizing-3/#min-content)/[max-content](https://www.w3.org/TR/css-sizing-3/#max-content) [cross size](#cross-size) of a <em><a href="#single-line-flex-container">single-line</a></em> [flex container](#flex-container) is the largest [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution)/[max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) (respectively) of its [flex items](#flex-item).
  >
  > <a id="ref-for-multi-line-flex-container①⑧"></a>
  >
  > <a id="ref-for-flex-container⑨③"></a>
  >
  > <a id="ref-for-min-content①⑦"></a>
  >
  > <a id="ref-for-max-content①⑨"></a>
  >
  > <a id="ref-for-cross-size④④"></a>
  >
  > <a id="ref-for-cross-axis②③"></a>
  >
  > <a id="ref-for-min-content-constraint③"></a>
  >
  > <a id="ref-for-max-content-constraint④"></a>
  >
  > <a id="ref-for-propdef-flex-flow⑨"></a>
  >
  > <a id="ref-for-min-content-contribution⑧"></a>
  >
  > <a id="ref-for-cross-size④⑤"></a>
  >
  > <a id="ref-for-flex-item①⑦⓪"></a>
  >
  > <a id="ref-for-available⑤"></a>
  >
  > For a <em><a href="#multi-line-flex-container">multi-line</a></em> [flex container](#flex-container), the [min-content](https://www.w3.org/TR/css-sizing-3/#min-content)/[max-content](https://www.w3.org/TR/css-sizing-3/#max-content) [cross size](#cross-size) is the sum of the flex line cross sizes resulting from sizing the flex container under a [cross-axis](#cross-axis) [min-content constraint](https://www.w3.org/TR/css-sizing-3/#min-content-constraint)/[max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint) (respectively). However, if the <a id="ref-for-flex-container⑨④"></a>flex container is [flex-flow: column wrap;](#propdef-flex-flow), then it’s sized by first finding the largest [min-content](https://www.w3.org/TR/css-sizing-3/#min-content-contribution)/<a id="ref-for-min-content-contribution⑨"></a>max-content [cross-size](https://www.w3.org/TR/css-flexbox-1/#cross-size) contribution among the [flex items](#flex-item) (respectively), then using that size as the [available space](https://www.w3.org/TR/css-sizing-3/#available) in the <a id="ref-for-cross-axis②④"></a>cross axis for each of the <a id="ref-for-flex-item①⑦①"></a>flex items during layout.
  >
  > <a id="ref-for-flex-container⑨⑤"></a>
  >
  > <a id="ref-for-flex-line①③"></a>
  >
  > <a id="ref-for-flex-item①⑦②"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > This heuristic for column wrap [flex containers](#flex-container) gives a reasonable approximation of the size that the <a id="ref-for-flex-container⑨⑥"></a>flex container should be, with each flex item ending up as min(<var>item’s own max-content</var>, <var>maximum min-content among all items</var>), and each [flex line](#flex-line) no larger than its largest [flex item](#flex-item). It’s not a <em>perfect</em> fit in some cases, but doing it completely correct is insanely expensive, and this works reasonably well.

- <a id="change-2015-a11y-tools"></a> Add explicit conformance criteria on authoring tools to keep presentation and DOM order in sync unless author explicitly indicates a desire to make them out-of-sync. ([Issue 8](https://drafts.csswg.org/css-flexbox-1/issues-lc-20150514#issue-8))

  > <a id="ref-for-propdef-order②④"></a>
  >
  > In order to preserve the author’s intended ordering in all presentation modes, authoring tools—including WYSIWYG editors as well as Web-based authoring aids—​must reorder the underlying document source and not use [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) to perform reordering unless the author has explicitly indicated that the underlying document order (which determines speech and navigation order) should be <em>out-of-sync</em> with the visual order.
  >
  > > <strong data-conversion-semantic="example">Example</strong>
  > >
  > > <a id="example-a11y-tools"></a> For example, a tool might offer both drag-and-drop reordering of flex items as well as handling of media queries for alternate layouts per screen size range.
  > > <a id="ref-for-propdef-order②⑤"></a>
  > >
  > > Since most of the time, reordering should affect all screen ranges as well as navigation and speech order, the tool would perform drag-and-drop reordering at the DOM layer. In some cases, however, the author may want different visual orderings per screen size. The tool could offer this functionality by using [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) together with media queries, but also tie the smallest screen size’s ordering to the underlying DOM order (since this is most likely to be a logical linear presentation order) while using <a id="ref-for-propdef-order②⑥"></a>order to determine the visual presentation order in other size ranges.
  > >
  > > <a id="ref-for-propdef-order②⑦"></a>
  > >
  > > This tool would be conformant, whereas a tool that only ever used [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) to handle drag-and-drop reordering (however convenient it might be to implement it that way) would be non-conformant.

- <a id="ref-for-valdef-align-items-auto⑥"></a>

  <a id="ref-for-propdef-justify-self"></a>

  <a id="ref-for-propdef-align-self②⑧"></a>

  <a id="change-2015-align-self-auto"></a> Defined that an [align-self](#propdef-align-self) or [justify-self](https://www.w3.org/TR/css-align-3/#propdef-justify-self) value of [auto](#valdef-align-items-auto) computes to itself on absolutely-positioned elements, for consistency with future extensions of these properties in [\[CSS-ALIGN-3\]](#biblio-css-align-3). ([Issue 5](https://drafts.csswg.org/css-flexbox-1/issues-lc-20150514#issue-5))

  > <a id="ref-for-valdef-align-items-auto⑦"></a>
  >
  > <a id="ref-for-valdef-align-items-auto⑧"></a>
  >
  > <a id="ref-for-propdef-align-self②⑨"></a>
  >
  > <a id="ref-for-propdef-align-items⑨"></a>
  >
  > <a id="ref-for-valdef-align-items-stretch②"></a>
  >
  > <u>On absolutely positioned elements, a value of [auto](#valdef-align-items-auto) computes to itself. On all other elements, a</u> ~~A~~ value of [auto](#valdef-align-items-auto) for [align-self](#propdef-align-self) computes to the value of [align-items](#propdef-align-items) on the element’s parent, or [stretch](#valdef-align-items-stretch) if the element has no parent.

- <a id="change-2015-percentage-margins"></a> Revert change to make percentage margins and padding relative to their own axes; instead allow both behaviors. ([Issue 11](https://drafts.csswg.org/css-flexbox-1/issues-lc-20150514#issue-11), [Issue 16](https://drafts.csswg.org/css-flexbox-1/issues-lc-20150514#issue-16))

  > <a id="ref-for-flex-item①⑦③"></a>
  >
  > Percentage margins and paddings on [flex items](#flex-item) are always resolved against their respective dimensions; unlike blocks, they do not always resolve against the inline dimension of their containing block.
  >
  > <a id="ref-for-flex-item①⑦④"></a>
  >
  > Percentage margins and paddings on [flex items](#flex-item) can be resolved against either:
  >
  > - their own axis (left/right percentages resolve against width, top/bottom resolve against height), or,
  > - the inline axis (left/right/top/bottom percentages all resolve against width)
  >
  > A user agent must choose one of these two behaviors.
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > Note: This variance sucks, but it accurately captures the current state of the world (no consensus among implementations, and no consensus within the CSSWG). It is the CSSWG’s intention that browsers will converge on one of the behaviors, at which time the spec will be amended to require that.
  >
  > <a id="ref-for-flex-item①⑦⑤"></a>
  >
  > > <strong data-conversion-semantic="advisement">Advisement</strong>
  > >
  > > Authors should avoid using percentages in paddings or margins on [flex items](#flex-item) entirely, as they will get different behavior in different browsers.

- <a id="change-2015-min-max-constraint"></a> Handle min/max constraints in sizing flex items.

  > - <a id="ref-for-max-content-constraint⑤"></a>
  >
  >   <a id="ref-for-min-content-constraint④"></a>
  >
  >   <a id="ref-for-flex-container⑨⑧"></a>
  >
  >   <a id="ref-for-definite④⑤"></a>
  >
  >   <a id="ref-for-flex-container⑨⑦"></a>
  >
  >   <strong>Determine the available main and cross space for the flex items.</strong> For each dimension, if that dimension of the [flex container](#flex-container)’s content box is a [definite size](#definite), use that; <u>if that dimension of the [flex container](#flex-container) is being sized under a [min](https://www.w3.org/TR/css-sizing-3/#min-content-constraint) or [max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint), the available space in that dimension is that constraint;</u> otherwise, subtract the <a id="ref-for-flex-container⑨⑨"></a>flex container’s margin, border, and padding from the space available to the flex container in that dimension and use that value.

- <a id="ref-for-propdef-break-inside"></a>

  <a id="changes-2015-first-fragment"></a> Correct negation in flex container fragmentation rule: previous definition implied [break-inside: avoid](https://www.w3.org/TR/css-break-3/#propdef-break-inside) behavior in all cases. ([Issue 5](https://drafts.csswg.org/css-flexbox-1/issues-lc-20150514#issue-5))

  > - If the first fragment of the flex container is not at the top of the page, and ~~some~~ <u>none</u> of its flex items ~~don’t~~ fit in the remaining space on the page, the entire fragment is moved to the next page.

#### <a id="change-201505-clarify"></a> Clarifications

- Miscellaneous minor editorial improvements and fixes to errors in examples.

### <a id="changes-201409"></a> Changes since the 25 September 2014 LCWD

A [Disposition of Comments](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140925) is also available.

#### <a id="change-201409-substantive"></a> Substantive Changes and Bugfixes

- <a id="ref-for-propdef-flex-basis②②"></a>

  <a id="change-201409-content"></a> Reverted [flex-basis: auto](#propdef-flex-basis) to its original meaning. Added <a id="ref-for-propdef-flex-basis②③"></a>flex-basis: content keyword to explicitly specify automatic content-based sizing. (Issue [10](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140925#issue-10))

- <a id="ref-for-propdef-align-content①②"></a>

  <a id="change-201409-align-content-wrapping"></a> Made applicability of [align-content](#propdef-align-content) depend on wrappability rather than number of resulting flex lines. (Issue [4](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140925#issue-4))

  > <a id="ref-for-multi-line-flex-container①⑨"></a>
  >
  > <a id="ref-for-flex-container①⓪⓪"></a>
  >
  > <a id="ref-for-cross-size④⑥"></a>
  >
  > <a id="ref-for-multi-line-flex-container②⓪"></a>
  >
  > <a id="ref-for-single-line-flex-container①⑥"></a>
  >
  > <a id="ref-for-flex-container①⓪①"></a>
  >
  > <a id="ref-for-propdef-align-content①③"></a>
  >
  > ~~When a flex container has multiple lines,~~ <u>In a [multi-line](#multi-line-flex-container) [flex container](#flex-container) (even one with only a single line),</u> the [cross size](#cross-size) of each line is the minimum size necessary \[...\] ~~When a flex container (even a [multi-line](#multi-line-flex-container) one) has only one line,~~ <u>In a [single-line](#single-line-flex-container) [flex container](#flex-container),</u> the <a id="ref-for-cross-size④⑦"></a>cross size of the line is the <a id="ref-for-cross-size④⑧"></a>cross size of the flex container, and [align-content](#propdef-align-content) has no effect.

  > <a id="ref-for-single-line-flex-container①⑦"></a>
  >
  > <a id="ref-for-flex-container①⓪②"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > Note, this property has no effect ~~when the flex container has only a single line.~~ <u>on a [single-line](#single-line-flex-container) [flex container](#flex-container).</u>

  > <a id="ref-for-multi-line-flex-container②①"></a>
  >
  > <a id="ref-for-flex-container①⓪③"></a>
  >
  > <a id="ref-for-cross-axis②⑤"></a>
  >
  > <a id="ref-for-single-line-flex-container①⑧"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > Only ~~flex containers with multiple lines~~ <u>[multi-line](#multi-line-flex-container) [flex containers](#flex-container)</u> ever have free space in the [cross-axis](#cross-axis) for lines to be aligned in, because in a ~~flex container with a single line~~ <u>[single-line](#single-line-flex-container) flex container</u> the sole line automatically stretches to fill the space.

  > <a id="ref-for-flex-line①④"></a>
  >
  > <a id="ref-for-multi-line-flex-container②②"></a>
  >
  > <a id="ref-for-flex-container①⓪④"></a>
  >
  > <a id="ref-for-single-line-flex-container①⑨"></a>
  >
  > <a id="ref-for-definite④⑥"></a>
  >
  > <a id="ref-for-cross-size④⑨"></a>
  >
  > <a id="ref-for-flex-line①⑤"></a>
  >
  > <a id="ref-for-flex-container①⓪⑤"></a>
  >
  > If the flex container ~~has only one [flex line](#flex-line) (even if it’s a [multi-line](#multi-line-flex-container) [flex container](#flex-container))~~ <u>is [single-line](#single-line-flex-container)</u> and has a [definite](#definite) [cross size](#cross-size), the <a id="ref-for-cross-size⑤⓪"></a>cross size of the [flex line](#flex-line) is the [flex container](#flex-container)’s inner <a id="ref-for-cross-size⑤①"></a>cross size.

  > <a id="ref-for-single-line-flex-container②⓪"></a>
  >
  > If the flex container ~~has only one flex line (even if it’s a multi-line flex container),~~ <u>is [single-line](#single-line-flex-container),</u> then clamp the line’s cross-size to be within the container’s computed min and max cross-size properties.

- <a id="change-201409-algo-breaks"></a> Removed text that asserted forced breaking behavior, replaced with reference to fragmentation section. This resolves a conflict in the spec. (Issue [18](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140925#issue-18))

  > <a id="ref-for-propdef-page-break-before"></a>
  >
  > <a id="ref-for-propdef-page-break-after"></a>
  >
  > <a id="ref-for-propdef-break-before⑥"></a>
  >
  > <a id="ref-for-propdef-break-after⑤"></a>
  >
  > collect consecutive items one by one until the first time that the next collected item would not fit into the flex container’s inner main size, <u>(</u> or until a forced break is encountered <u>, see [§ 10 Fragmenting Flex Layout](#pagination))</u> . \[...\] ~~A break is forced wherever the CSS2.1 [page-break-before](https://www.w3.org/TR/CSS2/page.html#propdef-page-break-before)/[page-break-after](https://www.w3.org/TR/CSS2/page.html#propdef-page-break-after) [\[CSS2\]](#biblio-css2) or the CSS3 [break-before](https://www.w3.org/TR/css-break-3/#propdef-break-before)/[break-after](https://www.w3.org/TR/css-break-3/#propdef-break-after) [\[CSS3-BREAK\]](#biblio-css3-break) properties specify a fragmentation break.~~

- <a id="ref-for-flex-base-size③⑥"></a>

  <a id="ref-for-flex-flex-shrink-factor⑨"></a>

  <a id="change-201409-inner-base-size"></a> Change the [flex shrink factor](#flex-flex-shrink-factor) to multiply by the <em>inner</em> (not outer) [flex base size](#flex-base-size). (Issue [9](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140925#issue-9))

  > <a id="ref-for-scaled-flex-shrink-factor⑦"></a>
  >
  > For every unfrozen item on the line, multiply its flex shrink factor by its ~~outer~~ <u>inner</u> flex base size, and note this as its [scaled flex shrink factor](#scaled-flex-shrink-factor).

- <a id="change-201409-neither"></a> Add back in missing “n” in “neither”... (Issue [6](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140925#issue-6))

  > <a id="ref-for-cross-size-property①"></a>
  >
  > <a id="ref-for-flex-item①⑦⑥"></a>
  >
  > <a id="ref-for-valdef-width-auto②①"></a>
  >
  > <a id="ref-for-cross-axis②⑥"></a>
  >
  > <a id="ref-for-stretched①"></a>
  >
  > If the [cross size property](#cross-size-property) of the [flex item](#flex-item) computes to [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), and <u>n</u> either of the [cross-axis](#cross-axis) margins are auto, the <a id="ref-for-flex-item①⑦⑦"></a>flex item is [stretched](#stretched).

- <a id="ref-for-definite④⑦"></a>

  <a id="ref-for-main-size④⑧"></a>

  <a id="ref-for-flex-container①⓪⑥"></a>

  <a id="change-201409-flexed-definite-container"></a> Specify that the [flex container](#flex-container)’s [main size](#main-size) must also be [definite](#definite) for a flex item’s flexed main size to be <a id="ref-for-definite④⑧"></a>definite. (Issue [20](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140925#issue-20))

  > <a id="ref-for-flex-item①⑦⑧"></a>
  >
  > <a id="ref-for-definite④⑨"></a>
  >
  > <a id="ref-for-flex-flex-basis①⑥"></a>
  >
  > <a id="ref-for-flex-container①⓪⑦"></a>
  >
  > <a id="ref-for-definite⑤⓪"></a>
  >
  > <a id="ref-for-main-size④⑨"></a>
  >
  > <a id="ref-for-main-size⑤⓪"></a>
  >
  > \[If\] ... the [flex item](#flex-item) has a [definite](#definite) [flex basis](#flex-flex-basis), <u>and the [flex container](#flex-container) has a [definite](#definite) [main size](#main-size),</u> the <a id="ref-for-flex-item①⑦⑨"></a>flex item’s [main size](#main-size) must be treated as <a id="ref-for-definite⑤①"></a>definite ...

- <a id="ref-for-specified-size②"></a>

  <a id="ref-for-definite⑤②"></a>

  <a id="ref-for-main-size-property④"></a>

  <a id="ref-for-valdef-flex-basis-content①⑥"></a>

  <a id="ref-for-propdef-flex-basis②④"></a>

  <a id="ref-for-specified-size①"></a>

  <a id="ref-for-valdef-flex-basis-content①⑤"></a>

  <a id="ref-for-flex-flex-basis①⑦"></a>

  <a id="change-201409-min-auto-specified-size"></a> Remove the requirement that the [flex basis](#flex-flex-basis) be [content](#valdef-flex-basis-content) for the [specified size](https://www.w3.org/TR/css-images-3/#specified-size) to be defined. The specified size should always win if it is smaller than the intrinsic size. This is particularly important to maintain author expectations for, e.g. `<img src="…" width=40 height=40 title="100x100 image">`. (Issue [25](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140925#issue-25))

  > If the item’s ~~computed [flex-basis](#propdef-flex-basis) is [content](#valdef-flex-basis-content) and its~~ computed [main size property](#main-size-property) is [definite](#definite), then the [specified size](https://www.w3.org/TR/css-images-3/#specified-size) is that size

- <a id="ref-for-flex-item①⑧⓪"></a>

  <a id="ref-for-propdef-display②③"></a>

  <a id="change-2014-blockify"></a> Remove the requirement that anonymous block creation (for things like [display: table-cell](https://www.w3.org/TR/css-display-4/#propdef-display)) occur <em>before</em> [flex item](#flex-item) blockification. (Instead, all children now blockify immediately, consistent with abspos/float behavior.)

#### <a id="change-201409-clarify"></a> Clarifications

- <a id="ref-for-flex-base-size③⑦"></a>

  <a id="change-201409-unclamped-size"></a> Clarify that [flex base size](#flex-base-size) is unclamped. (Issue [21](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140925#issue-21))

  > <a id="ref-for-flex-base-size③⑧"></a>
  >
  > <u>When determining the [flex base size](#flex-base-size), the item’s min and max main size properties are ignored (no clamping occurs).</u>
  >
  > <a id="ref-for-hypothetical-main-size⑦"></a>
  >
  > <a id="ref-for-flex-base-size③⑨"></a>
  >
  > The [hypothetical main size](#hypothetical-main-size) is the item’s [flex base size](#flex-base-size) clamped according to its min and max main size properties.

- <a id="change-201409-table-wrappers"></a> Restored normative status of note about table wrapper boxes normative; it had been accidentally changed in the previous draft. (Issue [2](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140925#issue-2))

- <a id="ref-for-propdef-display②④"></a>

  <a id="change-201409-display-longhands"></a> Removed references to [display](https://www.w3.org/TR/css-display-4/#propdef-display) property longhands, since they will be removed from CSS Display Level 3.

- <a id="ref-for-valdef-width-max-content③"></a>

  <a id="ref-for-valdef-flex-basis-content①⑦"></a>

  <a id="ref-for-main-size⑤①"></a>

  <a id="ref-for-flex-flex-basis①⑧"></a>

  <a id="ref-for-available⑥"></a>

  <a id="change-201409-layout-vs-size"></a> Change wording to not imply an unnecessary layout pass. (Issue [22](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140925#issue-22))

  > Otherwise, ~~lay out~~ <u>size</u> the item into the [available space](https://www.w3.org/TR/css-sizing-3/#available) using its used [flex basis](#flex-flex-basis) in place of its [main size](#main-size), treating a value of [content](#valdef-flex-basis-content) as [max-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-max-content).

- <a id="change-201409-clamped-specified"></a> Renamed “clamped size” to “specified size” in the definition of height: auto.

- Various trivial fixes.

### <a id="changes-201403"></a> Changes since the 25 March 2014 LCWD

A [Disposition of Comments](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325) is also available.

#### <a id="change-201403-substantive"></a> Substantive Changes and Bugfixes

The following significant changes were made since the [25 March 2014 Last Call Working Draft](https://www.w3.org/TR/2014/WD-css-flexbox-1-20140325/)

- <a id="change-201403-min-auto-not"></a> Fixed errors (missing negation, unspecified axis) in definition of min-width: auto. (Issues [11](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-11), [18](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-18), [30](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-30))

  > <a id="ref-for-flex-item①⑧①"></a>
  >
  > <a id="ref-for-propdef-overflow⑤"></a>
  >
  > <a id="ref-for-valdef-overflow-visible①"></a>
  >
  > <a id="ref-for-main-axis②①"></a>
  >
  > On a [flex item](#flex-item) whose [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) is ~~not~~ [visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible) <u>in the [main axis](#main-axis)</u> ,

- <a id="change-201403-min-auto-ratio"></a> Expanded and rewrote definition of min-width: auto to add special handling of items with intrinsic ratios. (Issues [16](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-16) and [28](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-28))

  > <a id="ref-for-flex-item①⑧②"></a>
  >
  > <a id="ref-for-propdef-overflow⑥"></a>
  >
  > <a id="ref-for-valdef-overflow-visible②"></a>
  >
  > <a id="ref-for-valdef-width-auto②②"></a>
  >
  > On a [flex item](#flex-item) whose [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) is not [visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible), <u>the following table gives the minimum size: [<strong>&#x5B;see table&#x5D;</strong>](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto)</u>  
  > ~~this keyword specifies as the minimum size the smaller of:~~
  >
  > - <a id="ref-for-min-content①⑧"></a>
  >
  >   the [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content), or
  >
  > - <a id="ref-for-definite⑤③"></a>
  >
  >   <a id="ref-for-propdef-height①⑧"></a>
  >
  >   <a id="ref-for-propdef-width②⑤"></a>
  >
  >   the computed [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)/[height](https://www.w3.org/TR/css-sizing-3/#propdef-height), if that value is [definite](#definite).

- <a id="change-201403-min-auto-main-size-basis"></a> Adjusted min-width: auto to only apply the computed main size as a minimum in cases where the flex basis was retrieved from the main size property. (Issue [19](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-19))

  > <a id="ref-for-valdef-align-items-auto⑨"></a>
  >
  > … is defined if <u>the item’s computed flex-basis is [auto](#valdef-align-items-auto) and</u> its computed main size property is definite …

- <a id="change-201403-min-auto-intrinsic-percentages"></a> Defined that any size adjustment imposed by a box’s min-width: auto is not consulted when percentage-sizing any of its contents. (Issue [27](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-27)) This change was later reverted with an [opposite definition](#change-2015-min-auto-intrinsic-percentages).

  > <a id="ref-for-valdef-flex-basis-auto⑤"></a>
  >
  > <a id="ref-for-propdef-min-height①⓪"></a>
  >
  > <a id="ref-for-propdef-max-height④"></a>
  >
  > <a id="ref-for-propdef-height①⑨"></a>
  >
  > In order to prevent cycling sizing, the [auto](#valdef-flex-basis-auto) value of [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) and [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height) does not factor into the percentage size resolution of the box’s contents. For example, a percentage-height block whose flex item parent has [height: 120em; min-height: auto](https://www.w3.org/TR/css-sizing-3/#propdef-height) will size itself against <a id="ref-for-propdef-height②⓪"></a>height: 120em regardless of the impact that <a id="ref-for-propdef-min-height①①"></a>min-height might have on the used size of the flex item.

- <a id="ref-for-valdef-flex-basis-content①⑧"></a>

  <a id="ref-for-propdef-flex-basis②⑤"></a>

  <a id="change-201403-flex-basis-auto"></a> Introduced extra main-size keyword to [flex-basis](#propdef-flex-basis) so that “lookup from main-size property” and “automatic sizing” behaviors could each be explicitly specified. (Issue [20](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-20)) This change was later reverted with an alternative proposal solving the same problem by instead introducing the [content](#valdef-flex-basis-content) keyword.

- <a id="ref-for-main-axis②②"></a>

  <a id="ref-for-flex-flex-basis①⑨"></a>

  <a id="ref-for-definite⑤④"></a>

  <a id="ref-for-flex-item①⑧③"></a>

  <a id="change-201403-definite-flexing"></a> Defined [flex items](#flex-item) with a [definite](#definite) [flex basis](#flex-flex-basis) to also be <a id="ref-for-definite⑤⑤"></a>definite in the [main axis](#main-axis), allowing resolution of percentage-sized children even when the item itself is flexible. (Issue [26](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-26))

  > If a percentage is going to be resolved against a flex item’s main size, and the flex item has a definite flex basis, the main size must be treated as definite for the purpose of resolving the percentage, and the percentage must resolve against the flexed main size of the flex item (that is, after the layout algorithm below has been completed for the flex item’s flex container, and the flex item has acquired its final size).

- <a id="ref-for-cross-size⑤②"></a>

  <a id="change-201403-clamp-single-line"></a> Clamp a single line flexbox’s line [cross size](#cross-size) to the container’s own min/max, even when the container’s size is indefinite. (Issue [9](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-9))

  > - <a id="ref-for-flex-line①⑥"></a>
  >
  >   The used cross-size of the [flex line](#flex-line) is the largest of the numbers found in the previous two steps and zero.
  >
  >   <u>If the flex container has only one flex line (even if it’s a multi-line flex container), then clamp the line’s cross-size to be within the container’s computed min and max cross-size properties. <strong data-conversion-semantic="note">Note:</strong> Note that if CSS 2.1’s definition of min/max-width/height applied more generally, this behavior would fall out automatically.</u>

- <a id="change-201403-resolve-flex"></a> Fixed various errors in the new [Resolving Flexible Lengths](#resolve-flexible-lengths) section (see [March 2014 rewrite to create continuity between flex: 0 and flex: 1](#change-2012-flex-continuity)) and reverted the editorial structure to match the old Candidate Recommendation. (Issues [3](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-3), [4](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-4), [8](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-8), [10](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-10), [15](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-15))

- <a id="change-201403-max-intrinsic"></a> Fixed [max-content sizing of flex containers](#intrinsic-sizes) to account for flexing behavior by normalizing per flex fraction rather than merely summing the max-content sizes of the flex items. (Issue [39](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-39))

- <a id="ref-for-propdef-flex③⑦"></a>

  <a id="change-201403-flex-animation"></a> Updated [flex](#propdef-flex) property to accept animations always, now that the discontinuity between 0 and non-0 values has been [fixed](#change-2012-flex-continuity). (Issue [5](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-5))

#### <a id="change-201403-clarify"></a> Clarifications

The following significant changes were made since the [25 March 2014 Last Call Working Draft](https://www.w3.org/TR/2014/WD-css-flexbox-1-20140325/)

- <a id="change-201403-css21-staticpos"></a> Clarified how the static position of an absolutely-positioned child of a flex container is calculated by introducing an explanation of the effect more closely tied with CSS2.1 concepts and terminology. (Issue [12](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-12))

  > ~~Its~~ <u>The</u> static position <u>of an absolutely-positioned child of a flex container</u> is ~~calculated by first doing full flex layout without the absolutely-positioned children, then positioning each absolutely-positioned child~~ <u>determined such that the child is positioned</u> as if it were the sole flex item in the flex container, assuming both the child and the flex container were fixed-size boxes of their used size.
  >
  > <a id="ref-for-static-position-rectangle③"></a>
  >
  > <a id="ref-for-propdef-justify-content①⓪"></a>
  >
  > <a id="ref-for-propdef-align-self③⓪"></a>
  >
  > <u>In other words, the static position of an absolutely positioned child of a flex container is determined <em>after flex layout</em> by setting the child’s [static-position rectangle](https://www.w3.org/TR/css-position-3/#static-position-rectangle) to the flex container’s content box, then aligning the absolutely positioned child within this rectangle according to the [justify-content](#propdef-justify-content) value of the flex container and the [align-self](#propdef-align-self) value of the child itself.</u>

- <a id="ref-for-flex-container①⓪⑧"></a>

  <a id="ref-for-propdef-order②⑧"></a>

  <a id="change-201403-abspos-ordering"></a> Clarified application of [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) to absolutely-positioned children of the [flex container](#flex-container). (Note, this behavior was later [rescinded](#change-2016-abspos-no-order).)

  > <a id="ref-for-propdef-order②⑨"></a>
  >
  > An absolutely-positioned child of a flex container does not participate in flex layout ~~beyond the reordering step~~ . <u>However, it does participate in the reordering step (see [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order)), which has an effect in their painting order.</u>

  > The order property controls the order in which ~~flex items~~ <u>children of a flex container</u> appear within their flex container…
  >
  > Unless otherwise specified by a future specification, this property has no effect on boxes that are not ~~flex items~~ <u>children of a flex container</u> .

  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > Note: Absolutely-positioned children of a flex container do not participate in flex layout, but are reordered together with any flex item children.

- <a id="ref-for-stretched②"></a>

  <a id="change-201403-clarify-stretched"></a> Clarified what a [stretched](#stretched) flex item is for the purposes of special behavior (like definiteness). (Issue [25](https://drafts.csswg.org/css-flexbox-1/issues-lc-20140325#issue-25))

  > <a id="ref-for-cross-size⑤③"></a>
  >
  > If the [cross size](#cross-size) property of the flex item computes to auto, <u>and either of the cross-axis margins are auto, the flex item is stretched. Its</u> ~~its~~ used value …

### <a id="changes-2012"></a> Changes since the 18 September 2012 Candidate Recommendation

A [Disposition of Comments](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012) is also available.

#### <a id="changes-2014-substantive"></a> Substantive Changes and Bugfixes

The following significant changes were made since the [18 September 2012 Candidate Recommendation](https://www.w3.org/TR/2012/CR-css3-flexbox-20120918/):

- <a id="ref-for-propdef-min-height①②"></a>

  <a id="ref-for-propdef-min-width⑨"></a>

  <a id="ref-for-valdef-width-auto②③"></a>

  <a id="change-2012-min-width"></a> Changed the behavior of the new [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) initial value of [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width)/[min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) to

  - <a id="ref-for-valdef-overflow-visible③"></a>

    <a id="ref-for-propdef-overflow⑦"></a>

    Take into account whether [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) is [visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible), since when <a id="ref-for-propdef-overflow⑧"></a>overflow is explicitly handled, it is confusing (and unnecessary) to force enough size to show all the content.

  - <a id="ref-for-propdef-height②①"></a>

    <a id="ref-for-propdef-width②⑥"></a>

    Take into account the specified [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)/[height](https://www.w3.org/TR/css-sizing-3/#propdef-height), so that the implied minimum is never greater than the specified size.

  - <a id="ref-for-valdef-width-min-content③"></a>

    Compute to itself (not to [min-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content)) on flex items, since they are no longer equivalent (due to above changes).

  ([Issue 19](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-19))

  > auto  
  > When used as the value of a flex item’s min main size property, this keyword indicates a minimum of the min-content size, to help ensure that the item is large enough to fit its contents.
  >
  > <a id="ref-for-valdef-width-min-content④"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > It is intended that this will compute to the [min-content](https://www.w3.org/TR/css-sizing-3/#valdef-width-min-content) keyword when the specification defining it ([\[CSS-SIZING-3\]](#biblio-css-sizing-3)) is sufficiently mature.
  >
  > <a id="ref-for-flex-item①⑧④"></a>
  >
  > <a id="ref-for-propdef-overflow⑨"></a>
  >
  > <a id="ref-for-valdef-overflow-visible④"></a>
  >
  > On a [flex item](#flex-item) whose [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) is not [visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible), this keyword specifies as the minimum size the smaller of:
  >
  > - <a id="ref-for-min-content①⑨"></a>
  >
  >   the [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content), or
  >
  > - <a id="ref-for-definite⑤⑥"></a>
  >
  >   <a id="ref-for-propdef-height②②"></a>
  >
  >   <a id="ref-for-propdef-width②⑦"></a>
  >
  >   the computed [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)/[height](https://www.w3.org/TR/css-sizing-3/#propdef-height), if that value is [definite](#definite).

- <a id="change-2012-percent-margins"></a> Specified that percentage margins/paddings on flex items are resolved against their respective dimensions, not the inline dimension of the containing block like blocks do. ([Issue 16](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-16))

  > <a id="ref-for-flex-item①⑧⑤"></a>
  >
  > Percentage margins and paddings on [flex items](#flex-item) are always resolved against their respective dimensions; unlike blocks, they do not always resolve against the inline dimension of their containing block.

- <a id="change-2012-stretch-definite"></a> Pass definiteness of a single-line flex container’s size through to any stretched items. ([Issue 3](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-3))

  > <a id="ref-for-flex-item①⑧⑥"></a>
  >
  > <a id="ref-for-single-line-flex-container②①"></a>
  >
  > <a id="ref-for-flex-container①⓪⑨"></a>
  >
  > <a id="ref-for-cross-size⑤④"></a>
  >
  > <a id="ref-for-propdef-align-self③①"></a>
  >
  > <a id="ref-for-definite⑤⑦"></a>
  >
  > As a special case for handling stretched [flex items](#flex-item), if a [single-line](#single-line-flex-container) [flex container](#flex-container) has a definite [cross size](#cross-size), the outer <a id="ref-for-cross-size⑤⑤"></a>cross size of any <a id="ref-for-flex-item①⑧⑦"></a>flex items with [align-self: stretch](#propdef-align-self) is the flex container’s inner <a id="ref-for-cross-size⑤⑥"></a>cross size (clamped to the <a id="ref-for-flex-item①⑧⑧"></a>flex item’s min and max <a id="ref-for-cross-size⑤⑦"></a>cross size) and is considered [definite](#definite).

- <a id="change-2012-stretch-auto"></a> Allow percentages inside a stretched auto-height flex item to resolve by requiring a relayout pass. ([Issue 3](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-3))

  > <a id="ref-for-propdef-align-self③②"></a>
  >
  > <a id="ref-for-cross-size⑤⑧"></a>
  >
  > If the flex item has [align-self: stretch](#propdef-align-self), redo layout for its contents, treating this used size as its definite [cross size](#cross-size) so that percentage-sized children can be resolved.
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > Note that this step does not affect the main size of the flex item, even if it has an intrinsic aspect ratio.

- <a id="change-2012-intrinsic-ratio"></a> Allow intrinsic aspect ratios to inform the [main-size calculation.](#algo-main-item) ([Issue 8](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-8))

  > If the flex item has ...
  >
  > - an intrinsic aspect ratio,
  >
  > - <a id="ref-for-valdef-width-auto②④"></a>
  >
  >   <a id="ref-for-flex-flex-basis②⓪"></a>
  >
  >   a [flex basis](#flex-flex-basis) of [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), and
  >
  > - <a id="ref-for-cross-size⑤⑨"></a>
  >
  >   a definite [cross size](#cross-size)
  >
  > <a id="ref-for-flex-base-size④⓪"></a>
  >
  > <a id="ref-for-cross-size⑥⓪"></a>
  >
  > <a id="ref-for-flex-item①⑧⑨"></a>
  >
  > then the [flex base size](#flex-base-size) is calculated from its inner [cross size](#cross-size) and the [flex item](#flex-item)’s intrinsic aspect ratio.

- <a id="ref-for-cross-size⑥①"></a>

  <a id="ref-for-main-size⑤②"></a>

  <a id="ref-for-hypothetical-main-size⑧"></a>

  <a id="change-2012-main-depend-cross"></a> Define [hypothetical main size](#hypothetical-main-size) when the [main size](#main-size) depends on the [cross size](#cross-size). ([Issue 23](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-23))

  > <a id="ref-for-cross-size⑥②"></a>
  >
  > <a id="ref-for-main-size⑤③"></a>
  >
  > <a id="ref-for-flex-item①⑨⓪"></a>
  >
  > <a id="ref-for-valdef-align-items-auto①⓪"></a>
  >
  > <a id="ref-for-definite⑤⑧"></a>
  >
  > <a id="ref-for-valdef-width-fit-content"></a>
  >
  > <u>If a [cross size](#cross-size) is needed to determine the [main size](#main-size) (e.g. when the [flex item](#flex-item)’s <a id="ref-for-main-size⑤④"></a>main size is in its block axis) and the <a id="ref-for-flex-item①⑨①"></a>flex item’s <a id="ref-for-cross-size⑥③"></a>cross size is [auto](#valdef-align-items-auto) and not [definite](#definite), in this calculation use [fit-content](https://www.w3.org/TR/css-sizing-4/#valdef-width-fit-content) as the <a id="ref-for-flex-item①⑨②"></a>flex item’s <a id="ref-for-cross-size⑥④"></a>cross size.</u>

- <a id="change-2012-intrinsic-sizes"></a> Defined the [intrinsic sizes of flex containers.](#intrinsic-sizes)

  > <strong>Determine the main size of the flex container</strong> using its main size property. ~~In this calculation, the min content main size of the flex container is the maximum of the flex container’s items' min-content size contributions, and the max content main size of the flex container is the sum of the flex container’s items' max-content size contributions. The min-content/max-content main size contribution of an item is its outer hypothetical main size when sized under a min-content/max-content constraint (respectively).~~ For this computation, ‘auto’ margins on flex items are treated as ‘0’.

  > <a id="ref-for-max-content②⓪"></a>
  >
  > <a id="ref-for-main-size⑤⑤"></a>
  >
  > <a id="ref-for-flex-container①①⓪"></a>
  >
  > <a id="ref-for-max-content-contribution①⑤"></a>
  >
  > <a id="ref-for-main-axis②③"></a>
  >
  > <a id="ref-for-min-content②⓪"></a>
  >
  > <a id="ref-for-single-line-flex-container②②"></a>
  >
  > <a id="ref-for-min-content-contribution①⓪"></a>
  >
  > <a id="ref-for-multi-line-flex-container②③"></a>
  >
  > The [max-content](https://www.w3.org/TR/css-sizing-3/#max-content) [main size](#main-size) of a [flex container](#flex-container) is the sum of the flex container’s items' [max-content contributions](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) in the [main axis](#main-axis). The [min-content](https://www.w3.org/TR/css-sizing-3/#min-content) <a id="ref-for-main-size⑤⑥"></a>main size of a [single-line](#single-line-flex-container) flex container is the sum of the flex container’s items' [min-content contributions](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) in the <a id="ref-for-main-axis②④"></a>main axis; for a [multi-line](#multi-line-flex-container) container, it is the largest of those contributions.
  >
  > <a id="ref-for-min-content②①"></a>
  >
  > <a id="ref-for-cross-size⑥⑤"></a>
  >
  > <a id="ref-for-max-content②①"></a>
  >
  > <a id="ref-for-main-axis②⑤"></a>
  >
  > <a id="ref-for-cross-axis②⑦"></a>
  >
  > The [min-content](https://www.w3.org/TR/css-sizing-3/#min-content) [cross size](#cross-size) and [max-content](https://www.w3.org/TR/css-sizing-3/#max-content) <a id="ref-for-cross-size⑥⑥"></a>cross size of a flex container are the <a id="ref-for-cross-size⑥⑦"></a>cross size of the flex container after performing layout into the given available [main-axis](#main-axis) space and infinite available [cross-axis](#cross-axis) space.
  >
  > <a id="ref-for-min-content-contribution①①"></a>
  >
  > <a id="ref-for-max-content-contribution①⑥"></a>
  >
  > <a id="ref-for-flex-item①⑨③"></a>
  >
  > <a id="ref-for-hypothetical-main-size⑨"></a>
  >
  > <a id="ref-for-min-content-constraint⑤"></a>
  >
  > <a id="ref-for-max-content-constraint⑥"></a>
  >
  > The main-size [min-content](https://www.w3.org/TR/css-sizing-3/#min-content-contribution)/[max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) of a [flex item](#flex-item) is its outer [hypothetical main size](#hypothetical-main-size) when sized under a [min-content](https://www.w3.org/TR/css-sizing-3/#min-content-constraint)/[max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint) (respectively).
  >
  > See [\[CSS-SIZING-3\]](#biblio-css-sizing-3) for a definition of the terms in this section.

- <a id="ref-for-single-line-flex-container②③"></a>

  <a id="change-2012-flex-line-bug"></a> Correct an omission in the flex-line size determination, so a [single-line](#single-line-flex-container) flexbox will size to its contents if it doesn’t have a definite size.

  > <a id="ref-for-flex-line①⑦"></a>
  >
  > <a id="ref-for-multi-line-flex-container②④"></a>
  >
  > <a id="ref-for-flex-container①①①"></a>
  >
  > <a id="ref-for-definite⑤⑨"></a>
  >
  > <a id="ref-for-cross-size⑥⑧"></a>
  >
  > <a id="ref-for-cross-size⑥⑨"></a>
  >
  > If the flex container has only one [flex line](#flex-line) (even if it’s a [multi-line](#multi-line-flex-container) [flex container](#flex-container)) <u>and has a [definite](#definite) [cross size](#cross-size)</u> , the [cross size](#cross-size) of the <a id="ref-for-flex-line①⑧"></a>flex line is the <a id="ref-for-flex-container①①②"></a>flex container’s inner <a id="ref-for-cross-size⑦⓪"></a>cross size.

- <a id="change-2012-flex-line-floor"></a> Flex lines have their size floored at 0. ([Issue 2](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-2))

  > The used cross-size of the flex line is the ~~larger~~ <u>largest</u> of the numbers found in the previous two steps <u>and zero</u> .

- <a id="change-2012-atomic-painting"></a> Flex items paint like inline blocks rather than blocks. ([Issue 18](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-18))

  > <a id="ref-for-flex-item①⑨④"></a>
  >
  > [Flex items](#flex-item) paint exactly the same as ~~block-level elements in the normal flow~~ <u>inline blocks [\[CSS2\]](#biblio-css2)</u> .

- <a id="ref-for-valdef-align-items-auto①①"></a>

  <a id="ref-for-propdef-flex③⑧"></a>

  <a id="ref-for-propdef-flex-basis②⑥"></a>

  <a id="change-2012-flex-basis-zero"></a> An omitted [flex-basis](#propdef-flex-basis) component of the [flex](#propdef-flex) shorthand now resolves to 0% instead of 0px. Because percentages resolved against indefinite sizes behave as [auto](#valdef-align-items-auto), this gives better behavior in shrink-wrapped flex containers. ([Issue 20](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-20))

  > <a id="ref-for-propdef-flex③⑨"></a>
  >
  > When omitted from the [flex](#propdef-flex) shorthand, its specified value is <u>0%</u> ~~the length zero~~ .

  > <a id="ref-for-propdef-flex④⓪"></a>
  >
  > [flex: \<positive-number\>](#propdef-flex)
  >
  > <a id="ref-for-propdef-flex④①"></a>
  >
  > Equivalent to [flex: \<positive-number\> 1 ~~0px~~<u>0%</u>](#propdef-flex).

  Note: This change was [reverted](#change-2015-flex-basis-zero).

- <a id="ref-for-valdef-width-auto②⑤"></a>

  <a id="ref-for-flex-base-size④①"></a>

  <a id="change-2012-unresolvable-basis"></a> Defined that an unresolvable percentage [flex base size](#flex-base-size) is treated as [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto).

  > <a id="ref-for-propdef-flex-basis②⑦"></a>
  >
  > <a id="ref-for-definite⑥⓪"></a>
  >
  > <a id="ref-for-main-size⑤⑦"></a>
  >
  > <a id="ref-for-valdef-width-auto②⑥"></a>
  >
  > percentage values of [flex-basis](#propdef-flex-basis) are resolved against the flex item’s containing block, i.e. its flex container, and if that containing block’s size is [indefinite](#definite), the result is ~~undefined~~ <u>the same as a [main size](#main-size) of [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto)</u> .

- <a id="ref-for-flex-container①①④"></a>

  <a id="ref-for-flex-container①①③"></a>

  <a id="change-2012-static-pos"></a> Simplified the static position of abspos children of [flex containers](#flex-container) to be consistent with Grid Layout. ([Issue 6](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-6))

  > An absolutely-positioned child of a [flex container](#flex-container) does not participate in flex layout beyond the reordering step.
  >
  > <a id="ref-for-propdef-left"></a>
  >
  > <a id="ref-for-propdef-right"></a>
  >
  > <a id="ref-for-propdef-top"></a>
  >
  > <a id="ref-for-propdef-bottom"></a>
  >
  > <a id="ref-for-valdef-align-items-auto①②"></a>
  >
  > However, if both [left](https://www.w3.org/TR/css-position-3/#propdef-left) and [right](https://www.w3.org/TR/css-position-3/#propdef-right) or both [top](https://www.w3.org/TR/css-position-3/#propdef-top) and [bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom) are [auto](#valdef-align-items-auto), then the used value of those properties are computed from its static position, as follows:
  >
  > <a id="ref-for-propdef-left①"></a>
  >
  > <a id="ref-for-propdef-right①"></a>
  >
  > <a id="ref-for-valdef-align-items-auto①③"></a>
  >
  > <a id="ref-for-flex-item①⑨⑤"></a>
  >
  > <a id="ref-for-main-start①①"></a>
  >
  > <a id="ref-for-cross-start①⑥"></a>
  >
  > <a id="ref-for-static-position"></a>
  >
  > <a id="ref-for-propdef-top①"></a>
  >
  > <a id="ref-for-propdef-bottom①"></a>
  >
  > If both [left](https://www.w3.org/TR/css-position-3/#propdef-left) and [right](https://www.w3.org/TR/css-position-3/#propdef-right) are [auto](#valdef-align-items-auto), the [flex item](#flex-item) must be positioned so that its [main-start](#main-start) or [cross-start](#cross-start) edge (whichever is in the horizontal axis) is aligned with the [static position](https://www.w3.org/TR/css-position-3/#static-position). If both [top](https://www.w3.org/TR/css-position-3/#propdef-top) and [bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom) are <a id="ref-for-valdef-align-items-auto①④"></a>auto, the <a id="ref-for-flex-item①⑨⑥"></a>flex item must be positioned so that its <a id="ref-for-main-start①②"></a>main-start or <a id="ref-for-cross-start①⑦"></a>cross-start edge (whichever is in the vertical axis) is aligned with the <a id="ref-for-static-position①"></a>static position.
  >
  > <a id="ref-for-main-axis②⑥"></a>
  >
  > In the [main axis](#main-axis),
  >
  > 1.  <a id="ref-for-main-start①③"></a>
  >
  >     <a id="ref-for-static-position②"></a>
  >
  >     <a id="ref-for-flex-line①⑨"></a>
  >
  >     <a id="ref-for-flex-item①⑨⑦"></a>
  >
  >     If there is a subsequent in-flow [flex item](#flex-item) on the same [flex line](#flex-line), the [static position](https://www.w3.org/TR/css-position-3/#static-position) is the outer [main-start](#main-start) edge of that <a id="ref-for-flex-item①⑨⑧"></a>flex item.
  >
  > 2.  <a id="ref-for-main-end①①"></a>
  >
  >     <a id="ref-for-static-position③"></a>
  >
  >     <a id="ref-for-flex-line②⓪"></a>
  >
  >     <a id="ref-for-flex-item①⑨⑨"></a>
  >
  >     Otherwise, if there is a preceding in-flow [flex item](#flex-item) on the same [flex line](#flex-line), the [static position](https://www.w3.org/TR/css-position-3/#static-position) is the outer [main-end](#main-end) edge of that <a id="ref-for-flex-item②⓪⓪"></a>flex item.
  >
  > 3.  <a id="ref-for-flex-container①①⑤"></a>
  >
  >     <a id="ref-for-propdef-justify-content①①"></a>
  >
  >     <a id="ref-for-static-position④"></a>
  >
  >     Otherwise, the [static position](https://www.w3.org/TR/css-position-3/#static-position) is determined by the value of [justify-content](#propdef-justify-content) on the [flex container](#flex-container) as if the <a id="ref-for-static-position⑤"></a>static position were represented by a zero-sized flex item.
  >
  > <a id="ref-for-cross-axis②⑧"></a>
  >
  > In the [cross axis](#cross-axis),
  >
  > 1.  <a id="ref-for-flex-line②①"></a>
  >
  >     <a id="ref-for-cross-start①⑧"></a>
  >
  >     <a id="ref-for-static-position⑥"></a>
  >
  >     <a id="ref-for-flex-item②⓪①"></a>
  >
  >     If there is a preceding in-flow [flex item](#flex-item), the [static position](https://www.w3.org/TR/css-position-3/#static-position) is the [cross-start](#cross-start) edge of the [flex line](#flex-line) that item is in.
  >
  > 2.  <a id="ref-for-flex-line②②"></a>
  >
  >     <a id="ref-for-cross-start①⑨"></a>
  >
  >     <a id="ref-for-static-position⑦"></a>
  >
  >     Otherwise, the [static position](https://www.w3.org/TR/css-position-3/#static-position) is the [cross-start](#cross-start) edge of the first [flex line](#flex-line).
  >
  > <a id="ref-for-valdef-align-items-flex-start③"></a>
  >
  > <a id="ref-for-propdef-justify-content①②"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > The static position is intended to more-or-less match the position of an anonymous 0×0 in-flow [flex-start](#valdef-align-items-flex-start)-aligned flex item that participates in flex layout, the primary difference being that any packing spaces due to [justify-content: space-around](#propdef-justify-content) or <a id="ref-for-propdef-justify-content①③"></a>justify-content: space-between are suppressed around the hypothetical item: between it and the next item if there is a real item after it, else between it and the previous item (if any) if there isn’t.
  >
  > <a id="ref-for-flex-item②⓪②"></a>
  >
  > <a id="ref-for-flex-container①①⑥"></a>
  >
  > Its static position is calculated by first doing full flex layout without the absolutely-positioned children, then positioning each absolutely-positioned child as if it were the sole [flex item](#flex-item) in the [flex container](#flex-container), assuming both the child and the <a id="ref-for-flex-container①①⑦"></a>flex container were fixed size boxes of their used size.
  >
  > <a id="ref-for-propdef-justify-content①④"></a>
  >
  > <a id="ref-for-propdef-align-content①④"></a>
  >
  > <a id="ref-for-flex-container①①⑧"></a>
  >
  > > <strong data-conversion-semantic="example">Example</strong>
  > >
  > > <a id="example-0e6abe14"></a> For example, by default, the static position of an absolutely positioned child aligns it to the main-start/cross-start corner, corresponding to the default values of [justify-content](#propdef-justify-content) and [align-content](#propdef-align-content) on the [flex container](#flex-container). Setting justify-content:center on the <a id="ref-for-flex-container①①⑨"></a>flex container, however, would center it in the main axis.

- <a id="change-2012-flex-continuity"></a> Changed algorithm for [resolving flexible lengths](#resolve-flexible-lengths) to make behavior continuous as the sum of the flex factors approaches zero. (No change for a sum ≥ 1.) ([Issue 30](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-30)) Replaces [this section](https://www.w3.org/TR/2012/CR-css3-flexbox-20120918/#resolve-flexible-lengths) with [this one](#resolve-flexible-lengths).

#### <a id="changes-2014-clarify"></a> Clarifications

The following significant clarifications were also made:

- <a id="clarify-2012-abspos-items"></a> Absolutely positioned children of a flex container are no longer called "flex items" (to avoid terminology confusion). (??)

  > | Field               | Definition                                                                                                                                      |
  > |---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------|
  > | <strong>Name: &#xA;         </strong> | order                                                                                                                                           |
  > | <strong>Applies to: &#xA;         </strong> | <a id="ref-for-flex-container①②⓪"></a><a id="ref-for-flex-item②⓪③"></a> [flex items](#flex-item) <u>and absolutely-positioned children of [flex containers](#flex-container)</u> |

  > <a id="ref-for-propdef-order③⓪"></a>
  >
  > Re-order the flex items <u>and absolutely positioned flex container children</u> according to their [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order).

- <a id="ref-for-propdef-display②⑤"></a>

  <a id="ref-for-propdef-float⑤"></a>

  <a id="clarify-2012-float-display"></a> Clarified that [float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float) still affects the computed [display](https://www.w3.org/TR/css-display-4/#propdef-display) (which may affect box-fixup rules that run prior to flex item determination). ([Issue 7](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-7))

  > <a id="ref-for-propdef-float⑥"></a>
  >
  > <a id="ref-for-propdef-clear④"></a>
  >
  > <a id="ref-for-flex-item②⓪④"></a>
  >
  > <a id="ref-for-propdef-float⑦"></a>
  >
  > <a id="ref-for-propdef-display②⑥"></a>
  >
  > [float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float) and [clear](https://www.w3.org/TR/CSS2/visuren.html#propdef-clear) have no effect on a [flex item](#flex-item) <u>, and do not take it out-of-flow. However, the [float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float) property can still affect box generation by influencing the [display](https://www.w3.org/TR/css-display-4/#propdef-display) property’s computed value.</u>

- <a id="clarify-2012-white-space"></a> Clarify what is meant by “white space”. ([Issue 26](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-26))

  > <a id="ref-for-propdef-white-space①"></a>
  >
  > However, an anonymous flex item that contains only [white space](https://www.w3.org/TR/CSS2/text.html#white-space-prop) <u>(i.e. characters that can be affected by the [white-space](https://www.w3.org/TR/css-text-4/#propdef-white-space) property)</u> is not rendered, as if it were display:none.

- <a id="clarify-2012-table-anon-boxes"></a> Clarified that table anonymous box generation occurs in place of computed value conversion for internal table elements.

- <a id="ref-for-propdef-display②⑦"></a>

  <a id="clarify-2012-display-blockification"></a> Clarified interaction of flex item determination with display-inside / display-outside (the new longhands of [display](https://www.w3.org/TR/css-display-4/#propdef-display) defined in the [CSS Display Module Level 3](https://www.w3.org/TR/css-display/)).

  > <a id="ref-for-flex-container①②①"></a>
  >
  > <a id="ref-for-propdef-display②⑧"></a>
  >
  > If the specified display-outside of an in-flow child of an element that generates a [flex container](#flex-container) is inline-level, it computes to block-level. (This effectively converts any inline [display](https://www.w3.org/TR/css-display-4/#propdef-display) values to their block equivalents.)

  Note: This change was [reverted](#change-201409-display-longhands).

- <a id="ref-for-propdef-overflow①⓪"></a>

  <a id="clarify-2012-overflow-flex-containers"></a> Clarified that [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) applies to flex containers.

- <a id="ref-for-selectordef-first-letter②"></a>

  <a id="ref-for-selectordef-first-line②"></a>

  <a id="clarify-2012-first-line-letter"></a> Clarified that [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) and [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) pseudo-elements do not apply to flex containers (because they are not block containers).

- <a id="ref-for-valdef-width-auto②⑦"></a>

  <a id="ref-for-valdef-align-items-stretch③"></a>

  <a id="clarify-2012-stretch-computed-auto"></a> Clarify that [stretch](#valdef-align-items-stretch) checks for the <em>computed</em> value of the cross-size property being [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), which means that percentage cross-sizes that behave as <a id="ref-for-valdef-width-auto②⑧"></a>auto (because they don’t resolve against definite sizes) aren’t stretched. ([Issue 5](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-5))

  > stretch  
  > <a id="ref-for-cross-size-property②"></a>
  >
  > <a id="ref-for-flex-item②⓪⑤"></a>
  >
  > <a id="ref-for-valdef-width-auto②⑨"></a>
  >
  > If the [cross size property](#cross-size-property) of the [flex item](#flex-item) ~~is~~ <u>computes to</u> [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), its used value is …

  > <a id="ref-for-propdef-align-self③③"></a>
  >
  > <a id="ref-for-valdef-width-auto③⓪"></a>
  >
  > <strong>Determine the used cross size of each flex item.</strong> If a flex item has [align-self: stretch](#propdef-align-self), its <u>computed</u> cross size property is [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), and …

- <a id="clarify-2012-flex-container-sizing"></a> Clarify that the rules of the formatting context are used for determining the flex container’s main size.

  > <strong>Determine the main size of the flex container</strong> using <u>the rules of the formatting context in which it participates</u> ~~its main size property~~ .

- <a id="ref-for-propdef-order③①"></a>

  <a id="clarify-2012-painting-order"></a> Clarified that [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order)-modified document order is used instead of raw document order when painting. (This was already stated in the <a id="ref-for-propdef-order③②"></a>order section, but not in the section explicitly about painting order.)

- <a id="clarify-2012-line-breaking"></a> Clarified line-breaking to precisely handle negatively-sized flex items and zero-size items at the end of a line. ([Issue 1](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-1))

  > Otherwise, starting from the first uncollected item, collect <u>consecutive items one by one until the first time that the <em>next</em> collected item would not fit into the flex container’s inner main size, or until a forced break is encountered. If the very first uncollected item wouldn’t fit, collect just it into the line</u> ~~as many consecutive flex items as will fit or until a forced break is encountered (but collect at least one) into the flex container’s inner main size into a flex line~~ .

  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > Note that ~~items with zero main size will never start a line unless they’re the very first items in the flex container, or they’re preceded by a forced break.~~ The "collect as many" line will collect ~~them~~ <u>zero-sized flex items</u> onto the end of the previous line even if the last non-zero item exactly "filled up" the line.

- <a id="clarify-2012-clamping"></a> Clarified that flex container cross sizes are still clamped by the flex container’s min/max properties. ([Issue 24](https://drafts.csswg.org/css-flexbox-1/issues-cr-2012#issue-24))

  > - <a id="ref-for-flex-container①②②"></a>
  >
  >   <a id="ref-for-definite⑥①"></a>
  >
  >   If the cross size property is a [definite](#definite) size, use that, <u>clamped by the min and max cross size properties of the [flex container](#flex-container)</u> .
  >
  > - <a id="ref-for-flex-container①②③"></a>
  >
  >   Otherwise, use the sum of the flex lines' cross sizes, <u>clamped by the min and max cross size properties of the [flex container](#flex-container)</u> .

## <a id="privacy"></a>Privacy Considerations

No new privacy considerations have been reported on this specification.

## <a id="security"></a>Security Considerations

No new security considerations have been reported on this specification.

Tests

the order property, now in css-display-3

- [order-interpolation.html](https://wpt.fyi/results/css/css-flexbox/animation/order-interpolation.html) [(live test)](http://wpt.live/css/css-flexbox/animation/order-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/animation/order-interpolation.html)
- [flexbox-anonymous-items-001.html](https://wpt.fyi/results/css/css-flexbox/flexbox-anonymous-items-001.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-anonymous-items-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-anonymous-items-001.html)
- [flexbox_order-abspos-space-around.html](https://wpt.fyi/results/css/css-flexbox/flexbox_order-abspos-space-around.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_order-abspos-space-around.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_order-abspos-space-around.html)
- [flexbox_order-box.html](https://wpt.fyi/results/css/css-flexbox/flexbox_order-box.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_order-box.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_order-box.html)
- flexbox-order-from-lowest.html (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-order-from-lowest.html)
- [flexbox_order.html](https://wpt.fyi/results/css/css-flexbox/flexbox_order.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_order.html)
- [flexbox_order-noninteger-invalid.html](https://wpt.fyi/results/css/css-flexbox/flexbox_order-noninteger-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox_order-noninteger-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox_order-noninteger-invalid.html)
- [flexbox-order-only-flexitems.html](https://wpt.fyi/results/css/css-flexbox/flexbox-order-only-flexitems.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-order-only-flexitems.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-order-only-flexitems.html)
- [flexible-order.html](https://wpt.fyi/results/css/css-flexbox/flexible-order.html) [(live test)](http://wpt.live/css/css-flexbox/flexible-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexible-order.html)
- [flex-item-z-ordering-001.html](https://wpt.fyi/results/css/css-flexbox/flex-item-z-ordering-001.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-z-ordering-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-z-ordering-001.html)
- [flex-item-z-ordering-002.html](https://wpt.fyi/results/css/css-flexbox/flex-item-z-ordering-002.html) [(live test)](http://wpt.live/css/css-flexbox/flex-item-z-ordering-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-item-z-ordering-002.html)
- [flex-order.html](https://wpt.fyi/results/css/css-flexbox/flex-order.html) [(live test)](http://wpt.live/css/css-flexbox/flex-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flex-order.html)
- [flexbox_computedstyle_order.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order.html)
- [flexbox_computedstyle_order-inherit.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order-inherit.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order-inherit.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order-inherit.html)
- [flexbox_computedstyle_order-integer.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order-integer.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order-integer.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order-integer.html)
- [flexbox_computedstyle_order-invalid.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order-invalid.html)
- [flexbox_computedstyle_order-negative.html](https://wpt.fyi/results/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order-negative.html) [(live test)](http://wpt.live/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order-negative.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/getcomputedstyle/flexbox_computedstyle_order-negative.html)
- order-001.htm (visual test) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/order-001.htm)
- [order-abs-children-painting-order-different-container.html](https://wpt.fyi/results/css/css-flexbox/order/order-abs-children-painting-order-different-container.html) [(live test)](http://wpt.live/css/css-flexbox/order/order-abs-children-painting-order-different-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/order/order-abs-children-painting-order-different-container.html)
- [order-abs-children-painting-order.html](https://wpt.fyi/results/css/css-flexbox/order/order-abs-children-painting-order.html) [(live test)](http://wpt.live/css/css-flexbox/order/order-abs-children-painting-order.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/order/order-abs-children-painting-order.html)
- [order-with-column-reverse.html](https://wpt.fyi/results/css/css-flexbox/order/order-with-column-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/order/order-with-column-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/order/order-with-column-reverse.html)
- [order-with-row-reverse.html](https://wpt.fyi/results/css/css-flexbox/order/order-with-row-reverse.html) [(live test)](http://wpt.live/css/css-flexbox/order/order-with-row-reverse.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/order/order-with-row-reverse.html)
- [order-painting.html](https://wpt.fyi/results/css/css-flexbox/order-painting.html) [(live test)](http://wpt.live/css/css-flexbox/order-painting.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/order-painting.html)
- [order_value.html](https://wpt.fyi/results/css/css-flexbox/order_value.html) [(live test)](http://wpt.live/css/css-flexbox/order_value.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/order_value.html)
- [order-computed.html](https://wpt.fyi/results/css/css-flexbox/parsing/order-computed.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/order-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/order-computed.html)
- [order-invalid.html](https://wpt.fyi/results/css/css-flexbox/parsing/order-invalid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/order-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/order-invalid.html)
- [order-valid.html](https://wpt.fyi/results/css/css-flexbox/parsing/order-valid.html) [(live test)](http://wpt.live/css/css-flexbox/parsing/order-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/parsing/order-valid.html)

------------------------------------------------------------------------

print stuff

- [break-nested-float-in-flex-item-001-print.html](https://wpt.fyi/results/css/css-flexbox/break-nested-float-in-flex-item-001-print.html) [(live test)](http://wpt.live/css/css-flexbox/break-nested-float-in-flex-item-001-print.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/break-nested-float-in-flex-item-001-print.html)
- [break-nested-float-in-flex-item-002-print.html](https://wpt.fyi/results/css/css-flexbox/break-nested-float-in-flex-item-002-print.html) [(live test)](http://wpt.live/css/css-flexbox/break-nested-float-in-flex-item-002-print.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/break-nested-float-in-flex-item-002-print.html)
- [flexbox-break-request-horiz-001a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-break-request-horiz-001a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-break-request-horiz-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-break-request-horiz-001a.html)
- [flexbox-break-request-horiz-001b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-break-request-horiz-001b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-break-request-horiz-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-break-request-horiz-001b.html)
- [flexbox-break-request-horiz-002a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-break-request-horiz-002a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-break-request-horiz-002a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-break-request-horiz-002a.html)
- [flexbox-break-request-horiz-002b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-break-request-horiz-002b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-break-request-horiz-002b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-break-request-horiz-002b.html)
- [flexbox-break-request-vert-001a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-break-request-vert-001a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-break-request-vert-001a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-break-request-vert-001a.html)
- [flexbox-break-request-vert-001b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-break-request-vert-001b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-break-request-vert-001b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-break-request-vert-001b.html)
- [flexbox-break-request-vert-002a.html](https://wpt.fyi/results/css/css-flexbox/flexbox-break-request-vert-002a.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-break-request-vert-002a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-break-request-vert-002a.html)
- [flexbox-break-request-vert-002b.html](https://wpt.fyi/results/css/css-flexbox/flexbox-break-request-vert-002b.html) [(live test)](http://wpt.live/css/css-flexbox/flexbox-break-request-vert-002b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/flexbox-break-request-vert-002b.html)
- [inline-flexbox-vertical-rl-image-flexitem-crash-print.html](https://wpt.fyi/results/css/css-flexbox/inline-flexbox-vertical-rl-image-flexitem-crash-print.html) [(live test)](http://wpt.live/css/css-flexbox/inline-flexbox-vertical-rl-image-flexitem-crash-print.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/inline-flexbox-vertical-rl-image-flexitem-crash-print.html)

------------------------------------------------------------------------

non-specific crashers

- [contain-size-layout-abspos-flex-container-crash.html](https://wpt.fyi/results/css/css-flexbox/contain-size-layout-abspos-flex-container-crash.html) [(live test)](http://wpt.live/css/css-flexbox/contain-size-layout-abspos-flex-container-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/contain-size-layout-abspos-flex-container-crash.html)
- [frame-flex-item-crash.html](https://wpt.fyi/results/css/css-flexbox/frame-flex-item-crash.html) [(live test)](http://wpt.live/css/css-flexbox/frame-flex-item-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/frame-flex-item-crash.html)
- [frameset-crash.html](https://wpt.fyi/results/css/css-flexbox/frameset-crash.html) [(live test)](http://wpt.live/css/css-flexbox/frameset-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/frameset-crash.html)
- [grandchild-span-height.html](https://wpt.fyi/results/css/css-flexbox/grandchild-span-height.html) [(live test)](http://wpt.live/css/css-flexbox/grandchild-span-height.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/grandchild-span-height.html)
- [min-height-min-content-crash.html](https://wpt.fyi/results/css/css-flexbox/min-height-min-content-crash.html) [(live test)](http://wpt.live/css/css-flexbox/min-height-min-content-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/min-height-min-content-crash.html)
- [mixed-containing-blocks-crash.html](https://wpt.fyi/results/css/css-flexbox/mixed-containing-blocks-crash.html) [(live test)](http://wpt.live/css/css-flexbox/mixed-containing-blocks-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/mixed-containing-blocks-crash.html)
- [negative-available-size-crash.html](https://wpt.fyi/results/css/css-flexbox/negative-available-size-crash.html) [(live test)](http://wpt.live/css/css-flexbox/negative-available-size-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/negative-available-size-crash.html)
- [negative-flex-margins-crash.html](https://wpt.fyi/results/css/css-flexbox/negative-flex-margins-crash.html) [(live test)](http://wpt.live/css/css-flexbox/negative-flex-margins-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/negative-flex-margins-crash.html)
- [negative-flex-rounding-crash.html](https://wpt.fyi/results/css/css-flexbox/negative-flex-rounding-crash.html) [(live test)](http://wpt.live/css/css-flexbox/negative-flex-rounding-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/negative-flex-rounding-crash.html)
- [negative-item-margins-002-crash.html](https://wpt.fyi/results/css/css-flexbox/negative-item-margins-002-crash.html) [(live test)](http://wpt.live/css/css-flexbox/negative-item-margins-002-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/negative-item-margins-002-crash.html)
- [negative-item-margins-crash.html](https://wpt.fyi/results/css/css-flexbox/negative-item-margins-crash.html) [(live test)](http://wpt.live/css/css-flexbox/negative-item-margins-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/negative-item-margins-crash.html)
- [orthogonal-flex-item-crash.html](https://wpt.fyi/results/css/css-flexbox/orthogonal-flex-item-crash.html) [(live test)](http://wpt.live/css/css-flexbox/orthogonal-flex-item-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/orthogonal-flex-item-crash.html)
- [position-absolute-scrollbar-freeze.html](https://wpt.fyi/results/css/css-flexbox/position-absolute-scrollbar-freeze.html) [(live test)](http://wpt.live/css/css-flexbox/position-absolute-scrollbar-freeze.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/position-absolute-scrollbar-freeze.html)
- [position-relative-with-scrollable-with-abspos-crash.html](https://wpt.fyi/results/css/css-flexbox/position-relative-with-scrollable-with-abspos-crash.html) [(live test)](http://wpt.live/css/css-flexbox/position-relative-with-scrollable-with-abspos-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/position-relative-with-scrollable-with-abspos-crash.html)
- [relayout-input.html](https://wpt.fyi/results/css/css-flexbox/relayout-input.html) [(live test)](http://wpt.live/css/css-flexbox/relayout-input.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/relayout-input.html)
- [remove-out-of-flow-child-crash.html](https://wpt.fyi/results/css/css-flexbox/remove-out-of-flow-child-crash.html) [(live test)](http://wpt.live/css/css-flexbox/remove-out-of-flow-child-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/remove-out-of-flow-child-crash.html)
- [zero-content-size-with-scrollbar-crash.html](https://wpt.fyi/results/css/css-flexbox/zero-content-size-with-scrollbar-crash.html) [(live test)](http://wpt.live/css/css-flexbox/zero-content-size-with-scrollbar-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/zero-content-size-with-scrollbar-crash.html)

------------------------------------------------------------------------

need quirks mode

- [fixed-table-layout-with-percentage-width-in-flex-item.html](https://wpt.fyi/results/css/css-flexbox/fixed-table-layout-with-percentage-width-in-flex-item.html) [(live test)](http://wpt.live/css/css-flexbox/fixed-table-layout-with-percentage-width-in-flex-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/fixed-table-layout-with-percentage-width-in-flex-item.html)
- [percentage-size-quirks-002.html](https://wpt.fyi/results/css/css-flexbox/percentage-size-quirks-002.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-size-quirks-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-size-quirks-002.html)
- [percentage-size-quirks.html](https://wpt.fyi/results/css/css-flexbox/percentage-size-quirks.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-size-quirks.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-size-quirks.html)
- [quirks-auto-block-size-with-percentage-item.html](https://wpt.fyi/results/css/css-flexbox/quirks-auto-block-size-with-percentage-item.html) [(live test)](http://wpt.live/css/css-flexbox/quirks-auto-block-size-with-percentage-item.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/quirks-auto-block-size-with-percentage-item.html)

------------------------------------------------------------------------

css box 3 tests

- [percentage-padding-001.html](https://wpt.fyi/results/css/css-flexbox/percentage-padding-001.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-padding-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-padding-001.html)
- [percentage-padding-002.html](https://wpt.fyi/results/css/css-flexbox/percentage-padding-002.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-padding-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-padding-002.html)
- [percentage-padding-003.html](https://wpt.fyi/results/css/css-flexbox/percentage-padding-003.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-padding-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-padding-003.html)
- [percentage-padding-004.html](https://wpt.fyi/results/css/css-flexbox/percentage-padding-004.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-padding-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-padding-004.html)
- [percentage-padding-005.html](https://wpt.fyi/results/css/css-flexbox/percentage-padding-005.html) [(live test)](http://wpt.live/css/css-flexbox/percentage-padding-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/percentage-padding-005.html)

------------------------------------------------------------------------

unsure/nonspecific

- [remove-wrapped-001.html](https://wpt.fyi/results/css/css-flexbox/remove-wrapped-001.html) [(live test)](http://wpt.live/css/css-flexbox/remove-wrapped-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/remove-wrapped-001.html)
- [remove-wrapped-002.html](https://wpt.fyi/results/css/css-flexbox/remove-wrapped-002.html) [(live test)](http://wpt.live/css/css-flexbox/remove-wrapped-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/remove-wrapped-002.html)
- [scrollbars-auto-min-content-sizing.html](https://wpt.fyi/results/css/css-flexbox/scrollbars-auto-min-content-sizing.html) [(live test)](http://wpt.live/css/css-flexbox/scrollbars-auto-min-content-sizing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/scrollbars-auto-min-content-sizing.html)
- [select-element-multiple.html](https://wpt.fyi/results/css/css-flexbox/select-element-multiple.html) [(live test)](http://wpt.live/css/css-flexbox/select-element-multiple.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/select-element-multiple.html)
- [shrinking-column-flexbox.html](https://wpt.fyi/results/css/css-flexbox/shrinking-column-flexbox.html) [(live test)](http://wpt.live/css/css-flexbox/shrinking-column-flexbox.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/shrinking-column-flexbox.html)
- [table-as-item-large-intrinsic-size.html](https://wpt.fyi/results/css/css-flexbox/table-as-item-large-intrinsic-size.html) [(live test)](http://wpt.live/css/css-flexbox/table-as-item-large-intrinsic-size.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-as-item-large-intrinsic-size.html)
- [table-with-float-paint.html](https://wpt.fyi/results/css/css-flexbox/table-with-float-paint.html) [(live test)](http://wpt.live/css/css-flexbox/table-with-float-paint.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-flexbox/table-with-float-paint.html)

------------------------------------------------------------------------

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

- [align-content](#propdef-align-content), in § 8.4
- [align-items](#propdef-align-items), in § 8.3
- [align-self](#propdef-align-self), in § 8.3
- auto
  - [value for align-items, align-self](#valdef-align-items-auto), in § 8.3
  - [value for flex-basis](#valdef-flex-basis-auto), in § 7.1
- [baseline](#valdef-align-items-baseline), in § 8.3
- center
  - [value for align-content](#valdef-align-content-center), in § 8.4
  - [value for align-items, align-self](#valdef-align-items-center), in § 8.3
  - [value for justify-content](#valdef-justify-content-center), in § 8.2
- [collapsed](#collapsed-flex-item), in § 4.4
- [collapsed flex item](#collapsed-flex-item), in § 4.4
- [column](#valdef-flex-direction-column), in § 5.1
- [column-reverse](#valdef-flex-direction-column-reverse), in § 5.1
- [content](#valdef-flex-basis-content), in § 7.1
- [content-based minimum size](#content-based-minimum-size), in § 4.5
- [content size suggestion](#content-size-suggestion), in § 4.5
- [cross axis](#cross-axis), in § 2
- [cross-axis](#cross-axis), in § 2
- [cross-axis baseline set](#cross-axis-baseline), in § 8.5
- [cross dimension](#cross-dimension), in § 2
- [cross-end](#cross-end), in § 2
- [cross size](#cross-size), in § 2
- [cross size property](#cross-size-property), in § 2
- [cross-start](#cross-start), in § 2
- [definite](#definite), in § 9.8
- [definite size](#definite), in § 9.8
- [first cross-axis baseline set](#cross-axis-baseline), in § 8.5
- [first main-axis baseline set](#main-axis-baseline), in § 8.5
- flex
  - [(property)](#propdef-flex), in § 7.1
  - [value for display](#valdef-display-flex), in § 3
- [flex base size](#flex-base-size), in § 9.2
- [\<'flex-basis'\>](#valdef-flex-flex-basis), in § 7.1
- [flex basis](#flex-flex-basis), in § 7.1
- [flex-basis](#propdef-flex-basis), in § 7.2.3
- [flex container](#flex-container), in § 2
- [flex direction](#flex-direction), in § 2
- [flex-direction](#propdef-flex-direction), in § 5.1
- flex-end
  - [value for align-content](#valdef-align-content-flex-end), in § 8.4
  - [value for align-items, align-self](#valdef-align-items-flex-end), in § 8.3
  - [value for justify-content](#valdef-justify-content-flex-end), in § 8.2
- [flex factor](#flex-factor), in § 7.1
- [flex-flow](#propdef-flex-flow), in § 5.3
- [flex formatting context](#flex-formatting-context), in § 3
- [\<'flex-grow'\>](#valdef-flex-flex-grow), in § 7.1
- [flex-grow](#propdef-flex-grow), in § 7.2.1
- [flex grow factor](#flex-flex-grow-factor), in § 7.1
- [flexible](#flexible), in § 7
- [flex item](#flex-item), in § 2
- [flex layout](#flex-layout), in § 1
- [flex-level](#flex-level), in § 4
- [flex line](#flex-line), in § 6
- [\<'flex-shrink'\>](#valdef-flex-flex-shrink), in § 7.1
- [flex-shrink](#propdef-flex-shrink), in § 7.2.2
- [flex shrink factor](#flex-flex-shrink-factor), in § 7.1
- flex-start
  - [value for align-content](#valdef-align-content-flex-start), in § 8.4
  - [value for align-items, align-self](#valdef-align-items-flex-start), in § 8.3
  - [value for justify-content](#valdef-justify-content-flex-start), in § 8.2
- [flex-wrap](#propdef-flex-wrap), in § 5.2
- [fully inflexible](#fully-inflexible), in § 7
- [hypothetical cross size](#hypothetical-cross-size), in § 9.4
- [hypothetical main size](#hypothetical-main-size), in § 9.2
- [indefinite](#definite), in § 9.8
- [indefinite size](#definite), in § 9.8
- [initial free space](#initial-free-space), in § 9.7
- [inline-flex](#valdef-display-inline-flex), in § 3
- [justify-content](#propdef-justify-content), in § 8.2
- [last cross-axis baseline set](#cross-axis-baseline), in § 8.5
- [last main-axis baseline set](#main-axis-baseline), in § 8.5
- [main axis](#main-axis), in § 2
- [main-axis](#main-axis), in § 2
- [main-axis baseline set](#main-axis-baseline), in § 8.5
- [main dimension](#main-dimension), in § 2
- [main-end](#main-end), in § 2
- [main size](#main-size), in § 2
- [main size property](#main-size-property), in § 2
- [main-start](#main-start), in § 2
- [max cross size](#max-cross-size), in § 2
- [max cross size property](#max-cross-size-property), in § 2
- [max main size](#max-main-size), in § 2
- [max main size property](#max-main-size-property), in § 2
- [min cross size](#min-cross-size), in § 2
- [min cross size property](#min-cross-size-property), in § 2
- [min main size](#min-main-size), in § 2
- [min main size property](#min-main-size-property), in § 2
- [multi-line](#multi-line-flex-container), in § 6
- [multi-line flex container](#multi-line-flex-container), in § 6
- [none](#valdef-flex-none), in § 7.1
- [nowrap](#valdef-flex-wrap-nowrap), in § 5.2
- \<number\>
  - [value for flex-grow](#valdef-flex-grow-number), in § 7.2.1
  - [value for flex-shrink](#valdef-flex-shrink-number), in § 7.2.2
- [participates in baseline alignment](#baseline-participation), in § 8.3
- [remaining free space](#remaining-free-space), in § 9.7
- [row](#valdef-flex-direction-row), in § 5.1
- [row-reverse](#valdef-flex-direction-row-reverse), in § 5.1
- [scaled flex shrink factor](#scaled-flex-shrink-factor), in § 9.7
- [single-line](#single-line-flex-container), in § 6
- [single-line flex container](#single-line-flex-container), in § 6
- space-around
  - [value for align-content](#valdef-align-content-space-around), in § 8.4
  - [value for justify-content](#valdef-justify-content-space-around), in § 8.2
- space-between
  - [value for align-content](#valdef-align-content-space-between), in § 8.4
  - [value for justify-content](#valdef-justify-content-space-between), in § 8.2
- [specified size suggestion](#specified-size-suggestion), in § 4.5
- stretch
  - [value for align-content](#valdef-align-content-stretch), in § 8.4
  - [value for align-items, align-self](#valdef-align-items-stretch), in § 8.3
- [stretched](#stretched), in § 8.3
- [target main size](#target-main-size), in § 9.7
- [transferred size suggestion](#transferred-size-suggestion), in § 4.5
- [wrap](#valdef-flex-wrap-wrap), in § 5.2
- [wrap-reverse](#valdef-flex-wrap-wrap-reverse), in § 5.2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ALIGN-3\] defines the following terms:
  - <a id="73f86444"></a>alignment baseline
  - <a id="0e506341"></a>alignment context
  - <a id="8bbaad92"></a>baseline set
  - <a id="d3e6a513"></a>generate baselines
  - <a id="80d2b689"></a>justify-self
  - <a id="15d5b5ce"></a>last-baseline alignment
  - <a id="35e80fd9"></a>start
  - <a id="81f3a960"></a>synthesize baseline
  - <a id="a5b111a8"></a>synthesized baseline
- \[CSS-BOX-4\] defines the following terms:
  - <a id="cba8daea"></a>content edge
  - <a id="16ff1cf8"></a>margin edge
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="4904f647"></a>fragmentation container
  - <a id="7eb0e25a"></a>fragmentation context
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="762bad34"></a>initial
  - <a id="19fd0eed"></a>legacy name alias
  - <a id="1a2b1083"></a>used value
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="409c2774"></a>anonymous
  - <a id="45f9eae9"></a>block box
  - <a id="8d18d112"></a>block container
  - <a id="a015488b"></a>block-level
  - <a id="91b1f11d"></a>block-level box
  - <a id="431a9cad"></a>blockify
  - <a id="0923db9e"></a>containing block
  - <a id="e8c16097"></a>display
  - <a id="abe7937d"></a>establishes an independent formatting context
  - <a id="10a5b15f"></a>flow layout
  - <a id="ae223697"></a>formatting context
  - <a id="6658d41f"></a>in-flow
  - <a id="6b9bba07"></a>inline-level
  - <a id="cbab059a"></a>non-replaced element
  - <a id="a9db5d6d"></a>replaced element
  - <a id="c9e837e8"></a>text node
  - <a id="e4657f7f"></a>text sequence
  - <a id="aceda213"></a>visibility
- \[CSS-FLEXBOX-1\] defines the following terms:
  - <a id="4792e5bd"></a>cross-size
  - <a id="7a8d5db2"></a>order
  - <a id="11b7dd33"></a>order-modified document order
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="e99a4517"></a>specified size
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="2d8be2d9"></a>vertical-align
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="02208825"></a>non-scrollable overflow value
  - <a id="add377f4"></a>overflow
  - <a id="a3cabdb1"></a>scroll container
  - <a id="2036da9e"></a>scrollable overflow value
  - <a id="700ea31d"></a>scrollport
  - <a id="855a7562"></a>visible
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="f411d42d"></a>bottom
  - <a id="ebcbc56d"></a>left
  - <a id="b8c34db8"></a>position
  - <a id="c70a6e95"></a>relative
  - <a id="a5bae6ee"></a>right
  - <a id="35f1d972"></a>static
  - <a id="b005b3ed"></a>static position
  - <a id="a9315173"></a>static-position rectangle
  - <a id="f99d4ae2"></a>top
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="63b59bd9"></a>::first-letter
  - <a id="4bda66a9"></a>::first-line
  - <a id="99a0ef70"></a>first formatted line
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="c20b5ff5"></a>auto
  - <a id="f7062343"></a>automatic block size
  - <a id="e9c67130"></a>automatic minimum size
  - <a id="37f6dbd7"></a>automatic size
  - <a id="c1c732b9"></a>available space
  - <a id="a91daf3b"></a>behave as auto
  - <a id="54a1fea8"></a>box-sizing
  - <a id="66f218c1"></a>definite
  - <a id="5ad01cca"></a>height
  - <a id="2a2ed19e"></a>indefinite
  - <a id="de48a940"></a>inner size
  - <a id="59e3c405"></a>intrinsic size contribution
  - <a id="7cb1c6db"></a>intrinsic sizing
  - <a id="a542cf9f"></a>max-content
  - <a id="8cdc912e"></a>max-content (for width)
  - <a id="a47902ec"></a>max-content constraint
  - <a id="c9ef7223"></a>max-content contribution
  - <a id="8a39af7f"></a>max-content size
  - <a id="6d275904"></a>maximum size
  - <a id="5e04c145"></a>min-content
  - <a id="d3da3539"></a>min-content (for width)
  - <a id="451a41ae"></a>min-content constraint
  - <a id="65c4b34c"></a>min-content contribution
  - <a id="6a444fd6"></a>min-content size
  - <a id="4405c984"></a>minimum size
  - <a id="47ea2436"></a>outer size
  - <a id="dd09245c"></a>preferred size
  - <a id="49731d1d"></a>width
- \[CSS-SIZING-4\] defines the following terms:
  - <a id="d1cbb104"></a>aspect-ratio
  - <a id="6b530a45"></a>fit-content
  - <a id="b03f7c8f"></a>preferred aspect ratio
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="f9fc4a3c"></a>document white space characters
  - <a id="b093a29f"></a>white-space
- \[CSS-TEXT-DECOR-4\] defines the following terms:
  - <a id="4d38e4c5"></a>text-decoration
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="d4441b24"></a>?
  - <a id="99261030"></a>CSS bracketed range notation
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="ecef1eb5"></a>block size
  - <a id="83d2ef35"></a>block-end
  - <a id="1118d052"></a>block-start
  - <a id="db7f5876"></a>endmost
  - <a id="8ebdd273"></a>horizontal-tb
  - <a id="a6eb24bb"></a>inline axis
  - <a id="18bb1084"></a>inline size
  - <a id="4da3b716"></a>inline-end
  - <a id="0da67e16"></a>inline-start
  - <a id="24c75626"></a>ltr
  - <a id="dea08a34"></a>rtl
  - <a id="42349ef8"></a>startmost
  - <a id="ee88ce59"></a>vertical-rl
  - <a id="eb6008ce"></a>writing mode
  - <a id="37bb38a0"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="3da47525"></a>auto (for table-layout)
  - <a id="d332e4ec"></a>auto (for z-index)
  - <a id="019a586e"></a>clear
  - <a id="da486c10"></a>float
  - <a id="cfab9333"></a>margin
  - <a id="0f0ab49f"></a>max-height
  - <a id="4d8f6525"></a>max-width
  - <a id="62b90f98"></a>min-height
  - <a id="1ecca6e7"></a>min-width
  - <a id="96f69bf6"></a>page-break-after
  - <a id="8781a4dc"></a>page-break-before
  - <a id="1848f1d3"></a>z-index
- \[CSS3-BREAK\] defines the following terms:
  - <a id="51ee3396"></a>break-after
  - <a id="eb306f02"></a>break-before
  - <a id="8ae583f9"></a>break-inside
- \[CSS3-WRITING-MODES\] defines the following terms:
  - <a id="fb688f4f"></a>direction
  - <a id="ac7161d9"></a>end
  - <a id="04e5ac3a"></a>start
- \[CSSOM-1\] defines the following terms:
  - <a id="fc19454a"></a>resolved value

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 11 March 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 19 December 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-flexbox-1"></a>\[CSS-FLEXBOX-1\]  
Tab Atkins Jr.; et al. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 19 November 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Elika Etemad. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 18 December 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 27 June 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-sizing-4"></a>\[CSS-SIZING-4\]  
Tab Atkins Jr.; Elika Etemad; Jen Simmons. [CSS Box Sizing Module Level 4](https://www.w3.org/TR/css-sizing-4/). 20 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-4&#x2F;](https://www.w3.org/TR/css-sizing-4/)

<a id="biblio-css-text-4"></a>\[CSS-TEXT-4\]  
Elika Etemad; et al. [CSS Text Module Level 4](https://www.w3.org/TR/css-text-4/). 29 May 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-4&#x2F;](https://www.w3.org/TR/css-text-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-css3-break"></a>\[CSS3-BREAK\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css3-writing-modes"></a>\[CSS3-WRITING-MODES\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-css-text-decor-4"></a>\[CSS-TEXT-DECOR-4\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 4](https://www.w3.org/TR/css-text-decor-4/). 4 May 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-4&#x2F;](https://www.w3.org/TR/css-text-decor-4/)

<a id="biblio-css3ui"></a>\[CSS3UI\]  
Tantek Çelik; Florian Rivoal. [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://www.w3.org/TR/css-ui-3/). 21 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-3&#x2F;](https://www.w3.org/TR/css-ui-3/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                                        | Initial                   | Applies to                 | Inh.                      | %ages                                            | Anim­ation type            | Canonical order | Com­puted value                                              |
|---------------------|------------------------------------------------------------------------------|---------------------------|----------------------------|---------------------------|--------------------------------------------------|---------------------------|-----------------|-------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-align-content①⑤"></a></span><a href="#propdef-align-content">align-content</a>&#xA;      </strong> | flex-start \| flex-end \| center \| space-between \| space-around \| stretch | stretch                   | multi-line flex containers | no                        | n/a                                              | discrete                  | per grammar     | specified keyword                                           |
| <strong><span><a id="ref-for-propdef-align-items①⓪"></a></span><a href="#propdef-align-items">align-items</a>&#xA;      </strong> | flex-start \| flex-end \| center \| baseline \| stretch                      | stretch                   | flex containers            | no                        | n/a                                              | discrete                  | per grammar     | specified keyword                                           |
| <strong><span><a id="ref-for-propdef-align-self③④"></a></span><a href="#propdef-align-self">align-self</a>&#xA;      </strong> | auto \| flex-start \| flex-end \| center \| baseline \| stretch              | auto                      | flex items                 | no                        | n/a                                              | discrete                  | per grammar     | specified keyword                                           |
| <strong><span><a id="ref-for-propdef-flex④②"></a></span><a href="#propdef-flex">flex</a>&#xA;      </strong> | none \| \[ \<'flex-grow'\> \<'flex-shrink'\>? \|\| \<'flex-basis'\> \]       | 0 1 auto                  | flex items                 | no                        | see individual properties                        | by computed value type    | per grammar     | see individual properties                                   |
| <strong><span><a id="ref-for-propdef-flex-basis②⑧"></a></span><a href="#propdef-flex-basis">flex-basis</a>&#xA;      </strong> | content \| \<'width'\>                                                       | auto                      | flex items                 | no                        | relative to the flex container’s inner main size | by computed value type    | per grammar     | specified keyword or a computed \<length-percentage\> value |
| <strong><span><a id="ref-for-propdef-flex-direction①⓪"></a></span><a href="#propdef-flex-direction">flex-direction</a>&#xA;      </strong> | row \| row-reverse \| column \| column-reverse                               | row                       | flex containers            | no                        | n/a                                              | discrete                  | per grammar     | specified keyword                                           |
| <strong><span><a id="ref-for-propdef-flex-flow①⓪"></a></span><a href="#propdef-flex-flow">flex-flow</a>&#xA;      </strong> | \<'flex-direction'\> \|\| \<'flex-wrap'\>                                    | see individual properties | see individual properties  | see individual properties | see individual properties                        | see individual properties | per grammar     | see individual properties                                   |
| <strong><span><a id="ref-for-propdef-flex-grow②①"></a></span><a href="#propdef-flex-grow">flex-grow</a>&#xA;      </strong> | \<number \[0,∞\]\>                                                           | 0                         | flex items                 | no                        | n/a                                              | by computed value type    | per grammar     | specified number                                            |
| <strong><span><a id="ref-for-propdef-flex-shrink⑧"></a></span><a href="#propdef-flex-shrink">flex-shrink</a>&#xA;      </strong> | \<number \[0,∞\]\>                                                           | 1                         | flex items                 | no                        | n/a                                              | number                    | per grammar     | specified value                                             |
| <strong><span><a id="ref-for-propdef-flex-wrap①②"></a></span><a href="#propdef-flex-wrap">flex-wrap</a>&#xA;      </strong> | nowrap \| wrap \| wrap-reverse                                               | nowrap                    | flex containers            | no                        | n/a                                              | discrete                  | per grammar     | specified keyword                                           |
| <strong><span><a id="ref-for-propdef-justify-content①⑤"></a></span><a href="#propdef-justify-content">justify-content</a>&#xA;      </strong> | flex-start \| flex-end \| center \| space-between \| space-around            | flex-start                | flex containers            | no                        | n/a                                              | discrete                  | per grammar     | specified keyword                                           |

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Outline Web-compatible algorithm here, once we have one. [\[Issue \#8884\]](https://github.com/w3c/csswg-drafts/issues/8884) [↵](#issue-bc420653)
