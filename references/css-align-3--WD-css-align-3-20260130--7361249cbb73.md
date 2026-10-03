Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Box Alignment Module Level 3](https://www.w3.org/TR/2026/WD-css-align-3-20260130/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Box Alignment Module Level 3

Source snapshot: https://www.w3.org/TR/2026/WD-css-align-3-20260130/

Snapshot SHA-256: 7361249cbb73d90e811e6eb7f8cd5bc4de5bbb81c2a7add6d1cbcb9dfd800d37

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 23 source tables are presented as readable Markdown tables or explicit labeled layouts: 16 ordinary table conversions, 7 complex-table layouts. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Box Alignment Module Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This module contains the features of [CSS](https://www.w3.org/TR/CSS/) relating to the alignment of boxes within their containers in the various CSS box layout models: block layout, table layout, flex layout, and grid layout. (The alignment of text and inline-level content is defined in [\[CSS-TEXT-3\]](#biblio-css-text-3) and [\[CSS-INLINE-3\]](#biblio-css-inline-3).)

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-align” in the title, like this: “\[css-align\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-align%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

The following features are at-risk, and may be dropped during the CR period:

- \[ first \| last \]? baseline

- <a id="ref-for-typedef-overflow-position"></a>

  the [\<overflow-position\>](#typedef-overflow-position) keywords

- <a id="ref-for-typedef-overflow-position①"></a>

  <a id="ref-for-valdef-overflow-position-safe"></a>

  the scrollable-area safety trigger into [safe](#valdef-overflow-position-safe) mode when no [\<overflow-position\>](#typedef-overflow-position) is specified

- <a id="ref-for-propdef-justify-items"></a>

  <a id="ref-for-valdef-justify-items-legacy"></a>

  the [legacy](#valdef-justify-items-legacy) keyword for [justify-items](#propdef-justify-items)

- <a id="ref-for-valdef-top-auto"></a>

  <a id="ref-for-box-alignment-properties"></a>

  the effect of the [box alignment properties](#box-alignment-properties) on absolutely-positioned boxes with [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto) offsets

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1.  Introduction

<a id="ref-for-propdef-text-align"></a>

CSS Levels 1 and 2 allowed for the alignment of text via [text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align) and the alignment of blocks by balancing auto margins. However, except in table cells, vertical alignment was not possible. As CSS adds further capabilities, the ability to align boxes in various dimensions becomes more critical. This module attempts to create a cohesive and common box alignment model to share among all of CSS.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The alignment of text and inline-level content is defined in [\[CSS-TEXT-3\]](#biblio-css-text-3) and [\[CSS-INLINE-3\]](#biblio-css-inline-3).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification is not intended to change any of the behavior defined in [CSS2.1](https://www.w3.org/TR/CSS2/) when the properties defined here are set to their initial values. If implementors or anyone else notices a discrepancy, please report this to the CSSWG as an error.

<em>This section (above) is not normative.</em>

### <a id="placement"></a>1.1.  Module Interactions

<a id="ref-for-propdef-vertical-align"></a>

This module adds some new alignment capabilities to the block layout model described in [\[CSS2\]](#biblio-css2) [chapters 9](https://www.w3.org/TR/CSS2/visuren.html) and [10](https://www.w3.org/TR/CSS2/visudet.html), [redefines how overconstrained block-level box margins are resolved](#justify-block), and defines the interaction of these new alignment properties with the alignment of table cell content using [vertical-align](https://www.w3.org/TR/CSS2/visudet.html#propdef-vertical-align), as defined in \[CSS2\] [chapter 17](https://www.w3.org/TR/CSS2/tables.html#height-layout).

The interaction of these properties with Grid Layout [\[CSS-GRID-1\]](#biblio-css-grid-1) and Flexible Box Layout [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1) is defined in their respective modules. The property definitions here supersede those in \[CSS-FLEXBOX-1\] (which have a smaller, earlier subset of permissible values).

No properties in this module apply to the `::first-line` or `::first-letter` pseudo-elements.

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

### <a id="partial"></a>1.3.  Partial Implementations

Since it is expected that support for the features in this module will be deployed in stages corresponding to the various layout models affected, it is hereby clarified that the [rules for partial implementations](https://www.w3.org/TR/CSS/#partial) that require treating as invalid any unsupported feature apply to any alignment keyword which is not supported across all layout modules to which it applies for layout models in which the implementation supports the property in general.

<a id="ref-for-propdef-align-self"></a>

<a id="ref-for-valdef-self-position-start"></a>

For example, if an implementation supports [align-self](#propdef-align-self) in [\[CSS-GRID-1\]](#biblio-css-grid-1) and [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1), then it must treat [start](#valdef-self-position-start) as invalid unless it is supported in both grid and flex containers. However if that same implementation does not support <a id="ref-for-propdef-align-self①"></a>align-self for block-level elements at all, then a lack of implementation of <a id="ref-for-propdef-align-self②"></a>align-self: start does not trigger this requirement to treat it as invalid.

## <a id="overview"></a>2.  Overview of Alignment Properties

The <a id="box-alignment-properties"></a>box alignment properties in CSS are a set of 6 properties that control alignment of boxes within other boxes. They can be described along two axises:

- <a id="ref-for-block-axis"></a>

  <a id="ref-for-cross-axis"></a>

  <a id="ref-for-inline-axis"></a>

  <a id="ref-for-main-axis"></a>

  which dimension they apply to ([main](https://www.w3.org/TR/css-flexbox-1/#main-axis)/[inline](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) vs. [cross](https://www.w3.org/TR/css-flexbox-1/#cross-axis)/[block](https://www.w3.org/TR/css-writing-modes-4/#block-axis)), and

- whether they control the position of the box within its parent, or the box’s content within itself.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification uses the terms “justify” and “align” to distinguish between alignment in the main/inline and cross/block dimensions, respectively. The choice is somewhat arbitrary, but having the two terms allows for a consistent naming scheme that works across all of CSS’s layout models (including [CSS Flexbox 1 § 2 Flex Layout Box Model and Terminology](https://www.w3.org/TR/css-flexbox-1/#box-model))

<a id="ref-for-box-alignment-properties①"></a>

The following table summarizes the [box alignment properties](#box-alignment-properties) and the display types they can apply to.

**Table 1**

Representation note: merged header paths are written explicitly; values from merged body cells are repeated wherever they apply.

| Common | Axis | Aligns | Applies to |
| --- | --- | --- | --- |
| <a id="ref-for-propdef-justify-content"></a> [justify-content](#propdef-justify-content) | main/inline | ![](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/content-example.svg) content within element (effectively adjusts padding) | <a id="ref-for-grid-container"></a> <a id="ref-for-flex-container"></a> <a id="ref-for-block-container"></a> [block containers](https://www.w3.org/TR/css-display-4/#block-container), [flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container), and [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container) |
| <a id="ref-for-propdef-align-content"></a> [align-content](#propdef-align-content) | cross/block | ![](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/content-example.svg) content within element (effectively adjusts padding) | [block containers](https://www.w3.org/TR/css-display-4/#block-container), [flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container), and [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container) |
| <a id="ref-for-propdef-justify-self"></a> [justify-self](#propdef-justify-self) | inline | ![](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/self-example.svg) element within parent (effectively adjusts margins) | <a id="ref-for-grid-item"></a> block-level boxes, absolutely-positioned boxes, and [grid items](https://www.w3.org/TR/css-grid-2/#grid-item) |
| <a id="ref-for-propdef-align-self③"></a> [align-self](#propdef-align-self) | cross/block | ![](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/self-example.svg) element within parent (effectively adjusts margins) | <a id="ref-for-grid-item①"></a> <a id="ref-for-flex-item"></a> absolutely-positioned boxes, [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item), and [grid items](https://www.w3.org/TR/css-grid-2/#grid-item) |
| <a id="ref-for-propdef-justify-items①"></a> [justify-items](#propdef-justify-items) | inline | ![](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/items-example.svg) items inside box (controls child items’ justify-self: auto) | <a id="ref-for-grid-container①"></a> <a id="ref-for-block-container①"></a> [block containers](https://www.w3.org/TR/css-display-4/#block-container) and [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container) |
| <a id="ref-for-propdef-align-items"></a> [align-items](#propdef-align-items) | cross/block | ![](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/items-example.svg) items inside box (controls child items’ justify-self: auto) | <a id="ref-for-grid-container②"></a> <a id="ref-for-flex-container①"></a> [flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container) and [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container) |

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The \*-items properties don’t affect the element itself. When set on a container, they specify the interpretation of any \*-self: auto used on children of the container element.

## <a id="terms"></a>3.  Alignment Terminology

Since this module defines alignment properties for all layout modes in CSS, some abstract terminology is introduced:

<a id="alignment-subject"></a>alignment subject  
<a id="ref-for-propdef-align-content①"></a>

<a id="ref-for-propdef-justify-content①"></a>

<a id="ref-for-writing-mode"></a>

<a id="ref-for-propdef-align-self④"></a>

<a id="ref-for-propdef-justify-self①"></a>

<a id="ref-for-alignment-subject"></a>

The [alignment subject](#alignment-subject) is the thing or things being aligned by the property. For [justify-self](#propdef-justify-self) and [align-self](#propdef-align-self), the <a id="ref-for-alignment-subject①"></a>alignment subject is the margin box of the box the property is set on, and assumes the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of that box. For [justify-content](#propdef-justify-content) and [align-content](#propdef-align-content), the <a id="ref-for-alignment-subject②"></a>alignment subject is defined by the layout mode and refers to some aspect of its contents; it also assumes the <a id="ref-for-writing-mode①"></a>writing mode of the box the property is set on.

<a id="alignment-container"></a>alignment container  
<a id="ref-for-writing-mode②"></a>

<a id="ref-for-alignment-subject③"></a>

<a id="ref-for-alignment-container"></a>

The [alignment container](#alignment-container) is the rectangle that the [alignment subject](#alignment-subject) is aligned within. This is defined by the layout mode, but is usually the <a id="ref-for-alignment-subject④"></a>alignment subject’s containing block, and assumes the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the box establishing the containing block.

<a id="fallback-alignment"></a>fallback alignment  
<a id="ref-for-alignment-subject⑤"></a>

<a id="ref-for-valdef-align-content-space-between"></a>

Some alignments can only be fulfilled in certain situations or are limited in how much space they can consume; for example, [space-between](#valdef-align-content-space-between) can only operate when there is more than one [alignment subject](#alignment-subject), and baseline alignment, once fulfilled, might not be enough to absorb all the excess space. In these cases a fallback alignment takes effect (as defined below) to fully consume the excess space.

## <a id="alignment-values"></a>4.  Alignment Keywords

All of the alignment properties use a common set of keyword values, which are defined in this section. Keywords fall into three categories:

[positional alignment](#positional-values)  
<a id="ref-for-alignment-container①"></a>

These keywords define alignment as an absolute position within the [alignment container](#alignment-container).

[baseline alignment](#baseline-values)  
<a id="ref-for-shared-alignment-context"></a>

<a id="ref-for-alignment-subject⑥"></a>

These keywords define alignment as a relationship among the baselines of multiple [alignment subjects](#alignment-subject) within an [alignment context](#shared-alignment-context).

[distributed alignment](#distribution-values)  
<a id="ref-for-alignment-subject⑦"></a>

These keywords define alignment as a distribution of space among [alignment subjects](#alignment-subject).

<a id="ref-for-valdef-self-position-center"></a>

<a id="ref-for-valdef-self-position-start①"></a>

<a id="ref-for-valdef-self-position-end"></a>

<a id="ref-for-valdef-self-position-self-start"></a>

<a id="ref-for-valdef-self-position-self-end"></a>

<a id="ref-for-valdef-self-position-flex-start"></a>

<a id="ref-for-valdef-self-position-flex-end"></a>

<a id="ref-for-valdef-justify-content-left"></a>

<a id="ref-for-valdef-justify-content-right"></a>

### <a id="positional-values"></a>4.1.  Positional Alignment: the [center](#valdef-self-position-center), [start](#valdef-self-position-start), [end](#valdef-self-position-end), [self-start](#valdef-self-position-self-start), [self-end](#valdef-self-position-self-end), [flex-start](#valdef-self-position-flex-start), [flex-end](#valdef-self-position-flex-end), [left](#valdef-justify-content-left), and [right](#valdef-justify-content-right) keywords

<a id="ref-for-alignment-subject⑧"></a>

<a id="ref-for-alignment-container②"></a>

The <a id="positional-alignment"></a>positional alignment keywords specify a position for an [alignment subject](#alignment-subject) with respect to its [alignment container](#alignment-container).

Values have the following meanings:

<a id="valdef-self-position-center"></a>center ([self](#self-alignment), [content](#content-distribution))  
<a id="ref-for-alignment-container③"></a>

<a id="ref-for-alignment-subject⑨"></a>

Centers the [alignment subject](#alignment-subject) within its [alignment container](#alignment-container).

<a id="valdef-self-position-start"></a>start ([self](#self-alignment), [content](#content-distribution))  
<a id="ref-for-alignment-container④"></a>

<a id="ref-for-alignment-subject①⓪"></a>

Aligns the [alignment subject](#alignment-subject) to be flush with the [alignment container](#alignment-container)’s start edge in the appropriate axis.

<a id="valdef-self-position-end"></a>end ([self](#self-alignment), [content](#content-distribution))  
<a id="ref-for-alignment-container⑤"></a>

<a id="ref-for-alignment-subject①①"></a>

Aligns the [alignment subject](#alignment-subject) to be flush with the [alignment container](#alignment-container)’s end edge in the appropriate axis.

<a id="valdef-self-position-self-start"></a>self-start ([self](#self-alignment))  
<a id="ref-for-alignment-container⑥"></a>

<a id="ref-for-alignment-subject①②"></a>

Aligns the [alignment subject](#alignment-subject) to be flush with the edge of the [alignment container](#alignment-container) corresponding to the <a id="ref-for-alignment-subject①③"></a>alignment subject’s start side in the appropriate axis.

<a id="valdef-self-position-self-end"></a>self-end ([self](#self-alignment))  
<a id="ref-for-alignment-container⑦"></a>

<a id="ref-for-alignment-subject①④"></a>

Aligns the [alignment subject](#alignment-subject) to be flush with the edge of the [alignment container](#alignment-container) corresponding to the <a id="ref-for-alignment-subject①⑤"></a>alignment subject’s end side in the appropriate axis.

<a id="valdef-self-position-flex-start"></a>flex-start ([self](#self-alignment), [content](#content-distribution))  
<a id="ref-for-flex-container②"></a>

<a id="ref-for-alignment-container⑧"></a>

<a id="ref-for-alignment-subject①⑥"></a>

<strong>Only used in flex layout.</strong> [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1) Aligns the [alignment subject](#alignment-subject) to be flush with the edge of the [alignment container](#alignment-container) corresponding to the [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container)’s main-start or cross-start side, as appropriate.

<a id="ref-for-flex-formatting-context"></a>

<a id="ref-for-valdef-self-position-start②"></a>

<a id="ref-for-flex-item①"></a>

<a id="ref-for-static-position"></a>

<a id="ref-for-flex-container③"></a>

<a id="ref-for-self-alignment-properties"></a>

<a id="ref-for-content-distribution-properties"></a>

When used outside of a [flex formatting context](https://www.w3.org/TR/css-flexbox-1/#flex-formatting-context), this value behaves as [start](#valdef-self-position-start). That is, on boxes that are not [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item) (or pretending to be <a id="ref-for-flex-item②"></a>flex items, such as when determining the [static position](https://www.w3.org/TR/css-position-3/#static-position) of an absolutely-positioned box that is a child of a [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container)), this value behaves as <a id="ref-for-valdef-self-position-start③"></a>start when used in the [self-alignment properties](#self-alignment-properties), and on boxes that are not <a id="ref-for-flex-container④"></a>flex containers, this value behaves as <a id="ref-for-valdef-self-position-start④"></a>start when used in the [content-distribution properties](#content-distribution-properties).

<a id="valdef-self-position-flex-end"></a>flex-end ([self](#self-alignment), [content](#content-distribution))  
<a id="ref-for-flex-container⑤"></a>

<a id="ref-for-alignment-container⑨"></a>

<a id="ref-for-alignment-subject①⑦"></a>

<strong>Only used in flex layout.</strong> Aligns the [alignment subject](#alignment-subject) to be flush with the edge of the [alignment container](#alignment-container) corresponding to the [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container)’s main-end or cross-end side, as appropriate.

<a id="ref-for-flex-formatting-context①"></a>

<a id="ref-for-valdef-self-position-end①"></a>

<a id="ref-for-flex-item③"></a>

<a id="ref-for-static-position①"></a>

<a id="ref-for-flex-container⑥"></a>

<a id="ref-for-self-alignment-properties①"></a>

<a id="ref-for-content-distribution-properties①"></a>

When used outside of a [flex formatting context](https://www.w3.org/TR/css-flexbox-1/#flex-formatting-context), this value behaves as [end](#valdef-self-position-end). That is, on boxes that are not [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item) (or pretending to be <a id="ref-for-flex-item④"></a>flex items, such as when determining the [static position](https://www.w3.org/TR/css-position-3/#static-position) of an absolutely-positioned box that is a child of a [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container)), this value behaves as <a id="ref-for-valdef-self-position-end②"></a>end when used in the [self-alignment properties](#self-alignment-properties), and on boxes that are not <a id="ref-for-flex-container⑦"></a>flex containers, this value behaves as <a id="ref-for-valdef-self-position-end③"></a>end when used in the [content-distribution properties](#content-distribution-properties).

<a id="valdef-justify-content-left"></a>left (only justify-\*)  
<a id="ref-for-physical-left"></a>

<a id="ref-for-alignment-container①⓪"></a>

<a id="ref-for-alignment-subject①⑧"></a>

Aligns the [alignment subject](#alignment-subject) to be flush with the [alignment container](#alignment-container)’s [line-left](https://www.w3.org/TR/css3-writing-modes/#line-left) or [physical left](https://www.w3.org/TR/css-writing-modes-4/#physical-left) edge, whichever is in the appropriate axis.

<a id="ref-for-valdef-self-position-start⑤"></a>

If the property’s axis is not parallel with either left↔right axis, this value behaves as [start](#valdef-self-position-start). <strong data-conversion-semantic="note">Note:</strong> Currently, the only case where the property’s axis is not parallel with either left↔right axis is in a column flexbox.

<a id="valdef-justify-content-right"></a>right (only justify-\*)  
<a id="ref-for-physical-right"></a>

<a id="ref-for-alignment-container①①"></a>

<a id="ref-for-alignment-subject①⑨"></a>

Aligns the [alignment subject](#alignment-subject) to be flush with the [alignment container](#alignment-container)’s [line-right](https://www.w3.org/TR/css3-writing-modes/#line-right) or [physical right](https://www.w3.org/TR/css-writing-modes-4/#physical-right) edge, whichever is in the appropriate axis.

<a id="ref-for-valdef-self-position-start⑥"></a>

If the property’s axis is not parallel with either left↔right axis, this value behaves as [start](#valdef-self-position-start). <strong data-conversion-semantic="note">Note:</strong> Currently, the only case where the property’s axis is not parallel with either left↔right axis is in a column flexbox.

Two grammar terms are used to denote certain subsets of these values:

<a id="ref-for-typedef-self-position"></a>

[\<self-position\>](#typedef-self-position)

<a id="ref-for-propdef-align-items①"></a>

<a id="ref-for-propdef-justify-items②"></a>

<a id="ref-for-alignment-container①②"></a>

<a id="ref-for-propdef-align-self⑤"></a>

<a id="ref-for-propdef-justify-self②"></a>

This set is used by [justify-self](#propdef-justify-self) and [align-self](#propdef-align-self) to align the box within its [alignment container](#alignment-container), and also by [justify-items](#propdef-justify-items) and [align-items](#propdef-align-items) (to specify default values for <a id="ref-for-propdef-justify-self③"></a>justify-self and <a id="ref-for-propdef-align-self⑥"></a>align-self).

<a id="typedef-self-position"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-comb-one⑤"></a>

```text
<self-position> = center | start | end | self-start | self-end |
        flex-start | flex-end
```
<a id="ref-for-typedef-content-position"></a>

[\<content-position\>](#typedef-content-position)

<a id="ref-for-propdef-align-content②"></a>

<a id="ref-for-propdef-justify-content②"></a>

This set is used by [justify-content](#propdef-justify-content) and [align-content](#propdef-align-content) to align the box’s contents within itself.

<a id="typedef-content-position"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-comb-one⑨"></a>

```text
<content-position> = center | start | end | flex-start | flex-end
```
<a id="ref-for-valdef-justify-content-left①"></a>

<a id="ref-for-valdef-justify-content-right①"></a>

<a id="ref-for-typedef-self-position①"></a>

<a id="ref-for-typedef-content-position①"></a>

<a id="ref-for-positional-alignment"></a>

<a id="ref-for-propdef-justify-content③"></a>

<a id="ref-for-propdef-justify-self④"></a>

<a id="ref-for-propdef-justify-items③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [left](#valdef-justify-content-left) and [right](#valdef-justify-content-right) are excluded from [\<self-position\>](#typedef-self-position) and [\<content-position\>](#typedef-content-position), despite being valid [positional alignment](#positional-alignment) values for [justify-content](#propdef-justify-content)/[justify-self](#propdef-justify-self)/[justify-items](#propdef-justify-items), because they are not allowed in the align-\* properties. They are instead explicitly included in the justify-\* properties’ grammars.

<a id="ref-for-valdef-self-position-start⑦"></a>

<a id="ref-for-valdef-self-position-end④"></a>

<a id="ref-for-flow-relative"></a>

<a id="ref-for-writing-mode③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-862b737b"></a> The [start](#valdef-self-position-start) and [end](#valdef-self-position-end) keywords are [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative): they use the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) to determine which side to align to.
>
> ![Inline-axis 'start' alignment: Horizontal Latin and Chinese are left-aligned, while Arabic and Hebrew are right-aligned.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/start-inline-tb.svg "inline-axis 'start' alignment in various writing systems") ![Inline-axis 'end' alignment: Horizontal Latin and Chinese are right-aligned, while Arabic and Hebrew are left-aligned.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/end-inline-tb.svg "inline-axis 'end' alignment in various writing systems")
>
> <a id="ref-for-valdef-self-position-start⑧"></a>
>
> <a id="ref-for-valdef-self-position-end⑤"></a>
>
> <a id="ref-for-inline-axis①"></a>
>
> [start](#valdef-self-position-start) vs [end](#valdef-self-position-end) in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) ([typically<sup>†</sup>](#flexbox-exception) justify-\*)
>
> <a id="ref-for-valdef-justify-content-left②"></a>
>
> <a id="ref-for-valdef-justify-content-right②"></a>
>
> <a id="ref-for-flow-relative①"></a>
>
> The [left](#valdef-justify-content-left) and [right](#valdef-justify-content-right) keywords are absolute (not [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative)).
>
> ![Inline-axis 'left' alignment: Horizontal text is left-aligned, regardless of writing system.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/left-inline-tb.svg "'left' alignment in various writing systems") ![Inline-axis 'right' alignment: Horizontal text is right-aligned, regardless of writing system.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/right-inline-tb.svg "'right' alignment in various writing systems")
>
> <a id="ref-for-valdef-justify-content-left③"></a>
>
> <a id="ref-for-valdef-justify-content-right③"></a>
>
> [left](#valdef-justify-content-left) vs [right](#valdef-justify-content-right)
>
> <a id="ref-for-valdef-self-position-start⑨"></a>
>
> <a id="ref-for-valdef-self-position-end⑥"></a>
>
> <a id="ref-for-inline-axis②"></a>
>
> <a id="ref-for-block-axis①"></a>
>
> The [start](#valdef-self-position-start) and [end](#valdef-self-position-end) keywords have meaning in both the [inline](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) and [block](https://www.w3.org/TR/css-writing-modes-4/#block-axis) axes: <a id="ref-for-valdef-self-position-start①⓪"></a>start always orients to the start of the text (top left for left-to-right/top-to-bottom languages like English) while <a id="ref-for-valdef-self-position-end⑦"></a>end always orients to the end of the text.
>
> ![Block-axis 'start' alignment: Horizontal text is top-aligned in the vertical axis.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/start-block-tb.svg "block-axis 'start' alignment in various writing systems") ![Block-axis 'bottom' alignment: Horizontal text is bottom-aligned in the vertical axis.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/end-block-tb.svg "block-axis 'end' alignment in various writing systems")
>
> <a id="ref-for-valdef-self-position-start①①"></a>
>
> <a id="ref-for-valdef-self-position-end⑧"></a>
>
> <a id="ref-for-block-axis②"></a>
>
> [start](#valdef-self-position-start) vs [end](#valdef-self-position-end) in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) ([typically<sup>†</sup>](#flexbox-exception) align-\*)
>
> <a id="ref-for-valdef-self-position-start①②"></a>
>
> <a id="ref-for-valdef-self-position-end⑨"></a>
>
> <a id="ref-for-writing-mode④"></a>
>
> <a id="ref-for-alignment-container①③"></a>
>
> <a id="ref-for-alignment-subject②⓪"></a>
>
> <a id="ref-for-valdef-self-position-self-start①"></a>
>
> <a id="ref-for-valdef-self-position-self-end①"></a>
>
> The [start](#valdef-self-position-start) and [end](#valdef-self-position-end) keywords use the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the [alignment container](#alignment-container), to help keep things consistent. But if alignment using the [alignment subject](#alignment-subject)’s <a id="ref-for-writing-mode⑤"></a>writing mode is needed, the [self-start](#valdef-self-position-self-start) and [self-end](#valdef-self-position-self-end) keywords can be used.
>
> ![Inline-axis 'start' alignment in an LTR container: Alignment uses the context’s start direction, so the (RTL) Arabic and Hebrew items are left-aligned alongside the (LTR) Latin and Chinese.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/self-v-context-start-tb.svg "inline-axis 'start' alignment with LTR context") ![Inline-axis 'self-start' alignment in an LTR container: Horizontal Latin and Chinese items are right-aligned, while Arabic and Hebrew items are left-aligned.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/self-v-context-self-start-tb.svg "inline-axis 'self-start' alignment with LTR context")
>
> <a id="ref-for-valdef-self-position-start①③"></a>
>
> <a id="ref-for-valdef-self-position-self-start②"></a>
>
> [start](#valdef-self-position-start) vs [self-start](#valdef-self-position-self-start) on the individual items

<a id="ref-for-vertical-writing-mode"></a>

<a id="ref-for-valdef-self-position-start①④"></a>

<a id="ref-for-valdef-self-position-end①⓪"></a>

<a id="ref-for-valdef-justify-content-left④"></a>

<a id="ref-for-valdef-justify-content-right④"></a>

<a id="ref-for-line-left"></a>

<a id="ref-for-line-right"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-472854ee"></a> The behavior of the alignment keywords is analogous in [vertical writing modes](https://www.w3.org/TR/css-writing-modes-4/#vertical-writing-mode): [start](#valdef-self-position-start) and [end](#valdef-self-position-end) are relative to the start/end of the text in the relevant axis. The [left](#valdef-justify-content-left) and [right](#valdef-justify-content-right) keywords are interpreted as [line-left](https://www.w3.org/TR/css-writing-modes-4/#line-left) and [line-right](https://www.w3.org/TR/css-writing-modes-4/#line-right), relative to the “left” and “right” sides of LTR/RTL text.
>
> ![Inline-axis 'start' alignment in vertical-rl writing: Vertical Latin and Chinese are top-aligned, while Arabic and Hebrew are bottom-aligned.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/start-inline-rl.svg "inline-axis 'start' alignment in various writing systems") ![Inline-axis 'end' alignment in vertical-rl writing: Vertical Latin and Chinese are top-aligned, while Arabic and Hebrew are bottom-aligned.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/end-inline-rl.svg "inline-axis 'end' alignment in various writing systems")
>
> <a id="ref-for-valdef-self-position-start①⑤"></a>
>
> <a id="ref-for-valdef-self-position-end①①"></a>
>
> <a id="ref-for-inline-axis③"></a>
>
> [start](#valdef-self-position-start) vs [end](#valdef-self-position-end) in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) ([typically<sup>†</sup>](#flexbox-exception) justify-\*)
>
> ![Inline-axis 'left' alignment in vertical-rl writing: Vertical text is top-aligned, regardless of writing system.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/left-inline-rl.svg "'left' alignment in various writing systems") ![Inline-axis 'right' alignment in vertical-rl writing: Vertical text is bottom-aligned, regardless of writing system.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/right-inline-rl.svg "'right' alignment in various writing systems")
>
> <a id="ref-for-valdef-justify-content-left⑤"></a>
>
> <a id="ref-for-valdef-justify-content-right⑤"></a>
>
> [left](#valdef-justify-content-left) vs [right](#valdef-justify-content-right)
>
> ![Block-axis 'start' alignment in vertical-rl writing: Vertical text is top-aligned in the vertical axis.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/start-block-rl.svg "block-axis 'start' alignment in various writing systems") ![Block-axis 'bottom' alignment in vertical-rl writing: Vertical text is bottom-aligned in the vertical axis.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/end-block-rl.svg "block-axis 'end' alignment in various writing systems")
>
> <a id="ref-for-valdef-self-position-start①⑥"></a>
>
> <a id="ref-for-valdef-self-position-end①②"></a>
>
> <a id="ref-for-block-axis③"></a>
>
> [start](#valdef-self-position-start) vs [end](#valdef-self-position-end) in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) ([typically<sup>†</sup>](#flexbox-exception) align-\*)
>
> ![Inline-axis 'start' alignment in a vertical-rl LTR container: Alignment uses the context’s start direction, so the (RTL) Arabic and Hebrew items are top-aligned alongside the (LTR) Latin and Chinese.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/self-v-context-start-rl.svg "inline-axis 'start' alignment with LTR context") ![Inline-axis 'self-start' alignment in a vertical-rl LTR container: Horizontal Latin and Chinese items are top-aligned, while Arabic and Hebrew items are bottom-aligned.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/self-v-context-self-start-rl.svg "inline-axis 'self-start' alignment with LTR context")
>
> <a id="ref-for-valdef-self-position-start①⑦"></a>
>
> <a id="ref-for-valdef-self-position-self-start③"></a>
>
> [start](#valdef-self-position-start) vs [self-start](#valdef-self-position-self-start) on the individual items

<a id="ref-for-inline-axis④"></a>

<a id="ref-for-block-axis④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="flexbox-exception"></a> For most layout models (block, table, grid, etc), the justify-\* properties always align things in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis), while the align-\* properties always align things in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis).
>
> <a id="ref-for-main-axis①"></a>
>
> <a id="ref-for-cross-axis①"></a>
>
> <a id="ref-for-propdef-flex-direction"></a>
>
> <a id="ref-for-valdef-flex-direction-row"></a>
>
> <a id="ref-for-valdef-flex-direction-row-reverse"></a>
>
> <a id="ref-for-inline-axis⑤"></a>
>
> <a id="ref-for-block-axis⑤"></a>
>
> <a id="ref-for-valdef-flex-direction-column"></a>
>
> <a id="ref-for-valdef-flex-direction-column-reverse"></a>
>
> Flexbox, on the other hand, has justify-\* align things in the [main axis](https://www.w3.org/TR/css-flexbox-1/#main-axis) and align-\* align things in the [cross axis](https://www.w3.org/TR/css-flexbox-1/#cross-axis). This depends on the value of [flex-direction](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-direction): when <a id="ref-for-propdef-flex-direction①"></a>flex-direction is [row](https://www.w3.org/TR/css-flexbox-1/#valdef-flex-direction-row) or [row-reverse](https://www.w3.org/TR/css-flexbox-1/#valdef-flex-direction-row-reverse), it matches the other layout modes ([inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) with justify-\*, [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) with align-\*); when <a id="ref-for-propdef-flex-direction②"></a>flex-direction is [column](https://www.w3.org/TR/css-flexbox-1/#valdef-flex-direction-column) or [column-reverse](https://www.w3.org/TR/css-flexbox-1/#valdef-flex-direction-column-reverse), it has the opposite correspondence.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-bb5da799"></a> Make it easier to understand the dual-axis nature of "start" and "end" wrt orthogonal flows.

<a id="ref-for-valdef-justify-self-baseline"></a>

<a id="ref-for-valdef-justify-self-first-baseline"></a>

<a id="ref-for-valdef-justify-self-last-baseline"></a>

### <a id="baseline-values"></a>4.2.  Baseline Alignment: the [baseline](#valdef-justify-self-baseline) keyword and [first](#valdef-justify-self-first-baseline)/[last](#valdef-justify-self-last-baseline) modifiers

See [CSS Writing Modes 3 § 4.1 Introduction to Baselines](https://www.w3.org/TR/css-writing-modes-3/#intro-baselines).

<a id="ref-for-alignment-subject②①"></a>

<a id="ref-for-shared-alignment-context①"></a>

<a id="ref-for-alignment-baseline"></a>

<a id="ref-for-baseline-sharing-group"></a>

<a id="ref-for-baseline-alignment"></a>

<a id="ref-for-alignment-container①④"></a>

<a id="ref-for-fallback-alignment"></a>

<a id="baseline-alignment"></a>Baseline alignment is a form of positional alignment that aligns multiple [alignment subjects](#alignment-subject) within a [shared alignment context](#shared-alignment-context) (such as cells within a row) by matching up their [alignment baselines](#alignment-baseline). If the position of the <a id="ref-for-alignment-subject②②"></a>alignment subjects within a [baseline-sharing group](#baseline-sharing-group) is not fully constrained by [baseline alignment](#baseline-alignment) (i.e., they could be shifted within their respective [alignment containers](#alignment-container) while maintaining baseline-alignment), they are [fallback-aligned](#fallback-alignment) insofar as possible while preserving their <a id="ref-for-baseline-alignment①"></a>baseline alignment.

<a id="ref-for-typedef-baseline-position"></a>

The baseline alignment keywords are represented with the [\<baseline-position\>](#typedef-baseline-position) grammar term:

<a id="typedef-baseline-position"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-comb-all"></a>

```text
<baseline-position> = [ first | last ]? && baseline
```
<a id="ref-for-valdef-justify-self-first-baseline①"></a>

<a id="ref-for-valdef-justify-self-last-baseline①"></a>

The [first](#valdef-justify-self-first-baseline) and [last](#valdef-justify-self-last-baseline) values give a box a <a id="baseline-alignment-preference"></a>baseline alignment preference: either “first” or “last”, respectively, defaulting to “first”.

Values have the following meanings:

<a id="valdef-justify-self-baseline"></a>baseline  
<a id="ref-for-valdef-justify-self-first-baseline②"></a>

Computes to [first baseline](#valdef-justify-self-first-baseline), defined below.

<a id="valdef-justify-self-first-baseline"></a>first baseline  
<a id="ref-for-baseline-sharing-group①"></a>

<a id="ref-for-first-baseline-set"></a>

<a id="ref-for-alignment-baseline①"></a>

Specifies participation in <a id="first-baseline-alignment"></a>first-baseline alignment: aligns the [alignment baseline](#alignment-baseline) of the box’s [first baseline set](#first-baseline-set) with the corresponding baseline of its [baseline-sharing group](#baseline-sharing-group). See [§ 9.3 Aligning Boxes by Baseline](#align-by-baseline) for more details.

<a id="ref-for-fallback-alignment①"></a>

<a id="ref-for-valdef-justify-self-first-baseline③"></a>

<a id="ref-for-self-align"></a>

<a id="ref-for-content-distribute"></a>

The [fallback alignment](#fallback-alignment) for [first baseline](#valdef-justify-self-first-baseline) is safe self-start (for [self-alignment](#self-align)) or safe start (for [content-distribution](#content-distribute)).

<a id="valdef-justify-self-last-baseline"></a>last baseline  
<a id="ref-for-baseline-sharing-group②"></a>

<a id="ref-for-last-baseline-set"></a>

<a id="ref-for-alignment-baseline②"></a>

Specifies participation in <a id="last-baseline-alignment"></a>last-baseline alignment: aligns the [alignment baseline](#alignment-baseline) of the box’s [last baseline set](#last-baseline-set) with the corresponding baseline of its [baseline-sharing group](#baseline-sharing-group).&#x323; See [§ 9.3 Aligning Boxes by Baseline](#align-by-baseline) for more details.

<a id="ref-for-fallback-alignment②"></a>

<a id="ref-for-valdef-justify-self-last-baseline②"></a>

<a id="ref-for-self-align①"></a>

<a id="ref-for-content-distribute①"></a>

The [fallback alignment](#fallback-alignment) for [last baseline](#valdef-justify-self-last-baseline) is safe self-end (for [self-alignment](#self-align)) or safe end (for [content-distribution](#content-distribute)).

<a id="ref-for-propdef-align-content③"></a>

<a id="ref-for-baseline-content-alignment"></a>

When specified for [align-content](#propdef-align-content), these values trigger [baseline content-alignment](#baseline-content-alignment), shifting the content of the box within its content box, and may also affect the sizing of the box itself. See [§ 5.4 Baseline Content-Alignment](#baseline-align-content).

<a id="ref-for-propdef-align-self⑦"></a>

<a id="ref-for-propdef-justify-self⑤"></a>

<a id="ref-for-baseline-self-alignment"></a>

When specified for [align-self](#propdef-align-self)/[justify-self](#propdef-justify-self), these values trigger [baseline self-alignment](#baseline-self-alignment), shifting the entire box within its container, which may affect the sizing of its container. See [§ 6.4 Baseline Self-Alignment](#baseline-align-self).

<a id="ref-for-baseline-self-alignment①"></a>

<a id="ref-for-content-distribute②"></a>

<a id="ref-for-baseline-content-alignment①"></a>

<a id="ref-for-content-distribution-properties②"></a>

<a id="ref-for-valdef-justify-content-normal"></a>

<a id="ref-for-used-value"></a>

<a id="ref-for-valdef-self-position-start①⑧"></a>

<a id="ref-for-baseline-alignment-preference"></a>

When [baseline self-alignment](#baseline-self-alignment) is specified on a box, [content distribution](#content-distribute) is performed first, then the resulting box and its contents are <a id="ref-for-baseline-self-alignment②"></a>self-aligned However, if the box also has [baseline content-alignment](#baseline-content-alignment) in the same axis or if its [content-distribution property](#content-distribution-properties) in the same axis is [normal](#valdef-justify-content-normal), its [used](https://www.w3.org/TR/css-cascade-5/#used-value) <a id="ref-for-content-distribution-properties③"></a>content-distribution property in that axis is [start](#valdef-self-position-start) or safe end for a [baseline alignment preference](#baseline-alignment-preference) of its <a id="ref-for-baseline-self-alignment③"></a>baseline self-alignment of “first” or “last”, respectively.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-13b9ed6a"></a>Add example images here.

<a id="ref-for-shared-alignment-context②"></a>

<a id="ref-for-fallback-alignment③"></a>

<a id="ref-for-propdef-align-content④"></a>

<a id="ref-for-valdef-self-position-start①⑨"></a>

<a id="ref-for-baseline-sharing-group③"></a>

<a id="ref-for-alignment-container①⑤"></a>

If a box does not belong to a [shared alignment context](#shared-alignment-context), then the [fallback alignment](#fallback-alignment) is used. For example, [align-content: baseline](#propdef-align-content) on a block box falls back to [start](#valdef-self-position-start) alignment. The <a id="ref-for-fallback-alignment④"></a>fallback alignment is also used to align the [baseline-sharing group](#baseline-sharing-group) within its [alignment container](#alignment-container).

<a id="ref-for-valdef-justify-self-baseline①"></a>

<a id="ref-for-valdef-justify-self-first-baseline④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because they are equivalent, and [baseline](#valdef-justify-self-baseline) is shorter, the CSSOM serializes [first baseline](#valdef-justify-self-first-baseline) as <a id="ref-for-valdef-justify-self-baseline②"></a>baseline. See [CSSOM § 6.7.2 Serializing CSS Values](https://www.w3.org/TR/cssom-1/#serializing-css-values).

<a id="ref-for-propdef-vertical-align①"></a>

<a id="ref-for-valdef-justify-self-baseline③"></a>

<a id="ref-for-valdef-justify-self-first-baseline⑤"></a>

<a id="ref-for-baseline-alignment-preference①"></a>

<a id="ref-for-propdef-display"></a>

<a id="ref-for-valdef-display-inline-block"></a>

<a id="ref-for-valdef-display-inline-table"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For the somewhat-related [vertical-align](https://www.w3.org/TR/CSS2/visudet.html#propdef-vertical-align) property, due to inconsistent design decisions in CSS2.1, [baseline](#valdef-justify-self-baseline) is not equivalent to [first baseline](#valdef-justify-self-first-baseline) as an inline-level box’s [baseline alignment preference](#baseline-alignment-preference) depends on [display](https://www.w3.org/TR/css-display-3/#propdef-display). (E.g., [inline-block](https://www.w3.org/TR/css-display-4/#valdef-display-inline-block) uses its last baseline by default, while [inline-table](https://www.w3.org/TR/css-display-4/#valdef-display-inline-table) uses its first baseline by default.)

<a id="ref-for-valdef-align-content-stretch"></a>

<a id="ref-for-valdef-align-content-space-between①"></a>

<a id="ref-for-valdef-align-content-space-around"></a>

<a id="ref-for-valdef-align-content-space-evenly"></a>

### <a id="distribution-values"></a>4.3.  Distributed Alignment: the [stretch](#valdef-align-content-stretch), [space-between](#valdef-align-content-space-between), [space-around](#valdef-align-content-space-around), and [space-evenly](#valdef-align-content-space-evenly) keywords

<a id="ref-for-propdef-justify-content④"></a>

<a id="ref-for-propdef-align-content⑤"></a>

<a id="ref-for-alignment-subject②③"></a>

The <a id="distributed-alignment"></a>distributed alignment values are used by [justify-content](#propdef-justify-content) and [align-content](#propdef-align-content) to disperse a container’s extra space among its [alignment subjects](#alignment-subject).

![space-between \| space-around \| space-evenly \| stretch](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/distribute.svg)

<a id="ref-for-distributed-alignment"></a>

The [distributed alignment](#distributed-alignment) values

<a id="ref-for-fallback-alignment⑤"></a>

When space cannot be distributed in this way, these values behave as their [fallback alignment](#fallback-alignment). Each distribution value has an associated default <a id="ref-for-fallback-alignment⑥"></a>fallback alignment. (A future level of this module may allow the <a id="ref-for-fallback-alignment⑦"></a>fallback alignment to be specified explicitly.)

<a id="valdef-align-content-space-between"></a>space-between  
<a id="ref-for-alignment-container①⑥"></a>

<a id="ref-for-alignment-subject②④"></a>

The [alignment subjects](#alignment-subject) are evenly distributed in the [alignment container](#alignment-container). The first <a id="ref-for-alignment-subject②⑤"></a>alignment subject is placed flush with the start edge of the <a id="ref-for-alignment-container①⑦"></a>alignment container, the last <a id="ref-for-alignment-subject②⑥"></a>alignment subject is placed flush with the end edge of the <a id="ref-for-alignment-container①⑧"></a>alignment container, and the remaining <a id="ref-for-alignment-subject②⑦"></a>alignment subjects are distributed so that the spacing between any two adjacent <a id="ref-for-alignment-subject②⑧"></a>alignment subjects is the same.

![For example, given three items, all excess space is split in two and distributed: one half between the first two and one half between the last two items.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/space-between.svg)

<a id="ref-for-fallback-alignment⑧"></a>

<a id="ref-for-valdef-self-position-flex-start①"></a>

<a id="ref-for-valdef-self-position-start②⓪"></a>

The default [fallback alignment](#fallback-alignment) for this value is safe flex-start. <strong data-conversion-semantic="note">Note:</strong> (For layout modes other than flex layout, [flex-start](#valdef-self-position-flex-start) is identical to [start](#valdef-self-position-start).)

<a id="valdef-align-content-space-around"></a>space-around  
<a id="ref-for-alignment-container①⑨"></a>

<a id="ref-for-alignment-subject②⑨"></a>

The [alignment subjects](#alignment-subject) are evenly distributed in the [alignment container](#alignment-container), with a half-size space on either end. The <a id="ref-for-alignment-subject③⓪"></a>alignment subjects are distributed so that the spacing between any two adjacent <a id="ref-for-alignment-subject③①"></a>alignment subjects is the same, and the spacing before the first and after the last <a id="ref-for-alignment-subject③②"></a>alignment subject is half the size of the other spacing.

![For example, given three items, all excess space is split into sixths and distributed: one sixth at the start, one at the end, and two sixths (one third) each between the first two and between the last two items.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/space-around.svg)

<a id="ref-for-fallback-alignment⑨"></a>

The default [fallback alignment](#fallback-alignment) for this value is safe center.

<a id="valdef-align-content-space-evenly"></a>space-evenly  
<a id="ref-for-alignment-container②⓪"></a>

<a id="ref-for-alignment-subject③③"></a>

The [alignment subjects](#alignment-subject) are evenly distributed in the [alignment container](#alignment-container), with a full-size space on either end. The <a id="ref-for-alignment-subject③④"></a>alignment subjects are distributed so that the spacing between any two adjacent <a id="ref-for-alignment-subject③⑤"></a>alignment subjects, before the first <a id="ref-for-alignment-subject③⑥"></a>alignment subject, and after the last <a id="ref-for-alignment-subject③⑦"></a>alignment subject is the same.

![For example, given three items, all excess space is split into fourths and distributed: to the start, to the end, to between the first two, and to between the last two items.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/space-evenly.svg)

<a id="ref-for-fallback-alignment①⓪"></a>

The default [fallback alignment](#fallback-alignment) for this value is safe center.

<a id="valdef-align-content-stretch"></a>stretch  
<a id="ref-for-propdef-max-width"></a>

<a id="ref-for-propdef-max-height"></a>

<a id="ref-for-alignment-container②①"></a>

<a id="ref-for-alignment-subject③⑧"></a>

If the combined size of the [alignment subjects](#alignment-subject) is less than the size of the [alignment container](#alignment-container), any auto-sized <a id="ref-for-alignment-subject③⑨"></a>alignment subjects have their size increased equally (not proportionally), while still respecting the constraints imposed by [max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height)/[max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width) (or equivalent functionality), so that the combined size exactly fills the <a id="ref-for-alignment-container②②"></a>alignment container.

![For example, given three items, all excess space is split into thirds and distributed: one third to each item.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/space-stretch.svg)

<a id="ref-for-fallback-alignment①①"></a>

<a id="ref-for-valdef-self-position-flex-start②"></a>

<a id="ref-for-valdef-self-position-flex-start③"></a>

<a id="ref-for-valdef-self-position-start②①"></a>

The default [fallback alignment](#fallback-alignment) for this value is [flex-start](#valdef-self-position-flex-start). <strong data-conversion-semantic="note">Note:</strong> (For layout modes other than flex layout, [flex-start](#valdef-self-position-flex-start) is identical to [start](#valdef-self-position-start).)

<a id="ref-for-content-distribution-properties④"></a>

<a id="ref-for-self-alignment-properties②"></a>

<a id="ref-for-propdef-justify-self⑥"></a>

<a id="ref-for-propdef-align-self⑧"></a>

<a id="ref-for-valdef-justify-self-stretch"></a>

<a id="ref-for-alignment-subject④⓪"></a>

<a id="ref-for-alignment-container②③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This \`align-content/stretch\` definition applies to the [content-distribution properties](#content-distribution-properties); the [self-alignment properties](#self-alignment-properties) [justify-self](#propdef-justify-self)/[align-self](#propdef-align-self) have their own [stretch](#valdef-justify-self-stretch) value, which can grow <em>or shrink</em> the [alignment subject](#alignment-subject) to ensure it exactly fits the [alignment container](#alignment-container).

<a id="ref-for-typedef-content-distribution"></a>

These values are represented with the [\<content-distribution\>](#typedef-content-distribution) grammar term:

<a id="typedef-content-distribution"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-comb-one①③"></a>

```text
<content-distribution> = space-between | space-around | space-evenly | stretch
```
<a id="ref-for-valdef-overflow-position-safe①"></a>

<a id="ref-for-valdef-overflow-position-unsafe"></a>

### <a id="overflow-values"></a>4.4.  Overflow Alignment: the [safe](#valdef-overflow-position-safe) and [unsafe](#valdef-overflow-position-unsafe) keywords and scroll safety limits

<a id="ref-for-alignment-subject④①"></a>

<a id="ref-for-alignment-container②④"></a>

In some situations, aligning exactly as specified would cause the [alignment subject](#alignment-subject) to overflow its [alignment container](#alignment-container), possibly causing data loss. For example, if the contents of a sidebar are unconditionally centered, items large enough to overflow it might extend beyond the viewport’s start edge, which can’t be scrolled to.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4fa66bd0"></a> The figure below illustrates the difference in “safe” versus “unsafe” centering, using a column flexbox as an example:
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
> An example of "safe" (on the left) vs "unsafe" (on the right) centering, when the centered item is larger than its container.
>
> The items in the figure on the left are centered unless they overflow, in which case all the overflow goes off the end edge, while those in the figure on the right are are all strictly centered, even if the one that is too long to fit overflows on both sides.
>
> If the container was placed against the left edge of the page, the “safe” behavior would be more desirable, as the long item would be fully readable, rather than clipped by the left edge of the screen. In other circumstances, the “unsafe” centering behavior might be better, as it correctly centers all the items.

To control this situation, an <a id="overflow-alignment"></a>overflow alignment mode can be explicitly specified. “Unsafe” alignment honors the specified alignment mode in overflow situations, even if it causes data loss, while “safe” alignment changes the alignment mode in overflow situations in an attempt to avoid data loss. The default behavior is to contain the alignment subject within the scrollable area, though at the time of writing this safety feature is not yet implemented.

<a id="typedef-overflow-position"></a>

<a id="ref-for-comb-one①④"></a>

```text
<overflow-position> = unsafe | safe
```
<a id="valdef-overflow-position-safe"></a>safe  
<a id="ref-for-valdef-self-position-flex-start④"></a>

<a id="ref-for-alignment-container②⑤"></a>

<a id="ref-for-alignment-subject④②"></a>

If the size of the [alignment subject](#alignment-subject) overflows the [alignment container](#alignment-container), the <a id="ref-for-alignment-subject④③"></a>alignment subject is instead aligned as if the alignment mode were [flex-start](#valdef-self-position-flex-start).

<a id="valdef-overflow-position-unsafe"></a>unsafe  
<a id="ref-for-alignment-container②⑥"></a>

<a id="ref-for-alignment-subject④④"></a>

Regardless of the relative sizes of the [alignment subject](#alignment-subject) and [alignment container](#alignment-container), the given alignment value is honored.

(no value specified)  
<a id="ref-for-overflow-alignment"></a>

If the [overflow alignment](#overflow-alignment) isn’t explicitly specified, the default <a id="ref-for-overflow-alignment①"></a>overflow alignment is a blend of “safe” and “unsafe”. See [§ 4.4.1 Automatic Overflow Alignment Safety](#auto-safety) for details.

#### <a id="auto-safety"></a>4.4.1.  Automatic Overflow Alignment Safety

<a id="ref-for-overflow-alignment②"></a>

<a id="ref-for-valdef-overflow-position-safe②"></a>

<a id="ref-for-valdef-overflow-position-unsafe①"></a>

If no [overflow alignment](#overflow-alignment) mode is specified for a property, the default behavior lies somewhere between [safe](#valdef-overflow-position-safe) and [unsafe](#valdef-overflow-position-unsafe), and also varies by layout mode.

##### <a id="auto-safety-scroll"></a>4.4.1.1.  Content Distribution for Scroll Containers

<a id="ref-for-overflow-alignment③"></a>

<a id="ref-for-content-distribute③"></a>

<a id="ref-for-scroll-container"></a>

<a id="ref-for-valdef-overflow-position-unsafe②"></a>

<a id="ref-for-valdef-justify-content-normal①"></a>

<a id="ref-for-content-distribution-properties⑤"></a>

<a id="ref-for-scrollable-overflow-region"></a>

The default [overflow alignment](#overflow-alignment) behavior for [content distribution](#content-distribute) on [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container) is [unsafe](#valdef-overflow-position-unsafe). Non-[normal](#valdef-justify-content-normal) values of the [content-distribution properties](#content-distribution-properties) instead alter the [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region) in order to allow access to the overflowing content. See [§ 5.3 Alignment Overflow and Scroll Containers](#overflow-scroll-position).

##### <a id="auto-safety-position"></a>4.4.1.2.  Self-Alignment for Absolutely Positioned Boxes

<a id="ref-for-absolute-position"></a>

<a id="ref-for-alignment-subject④⑤"></a>

<a id="ref-for-used-value①"></a>

<a id="ref-for-self-alignment-properties③"></a>

<a id="ref-for-valdef-justify-self-normal"></a>

<a id="ref-for-overflow-alignment④"></a>

For [absolutely positioned](https://www.w3.org/TR/css-position-3/#absolute-position) [alignment subjects](#alignment-subject) whose relevant [used](https://www.w3.org/TR/css-cascade-5/#used-value) [self-alignment property](#self-alignment-properties) is not [normal](#valdef-justify-self-normal), the default [overflow alignment](#overflow-alignment) behavior is as follows:

1.  <a id="ref-for-alignment-subject④⑥"></a>

    <a id="ref-for-inset-modified-containing-block"></a>

    If the [alignment subject](#alignment-subject) fits within the [inset-modified containing block](https://www.w3.org/TR/css-position-3/#inset-modified-containing-block), align as specified to the extent possible without overflowing the <a id="ref-for-inset-modified-containing-block①"></a>inset-modified containing block.

2.  <a id="ref-for-alignment-subject④⑦"></a>

    <a id="ref-for-inset-modified-containing-block②"></a>

    Otherwise, if the [alignment subject](#alignment-subject) fits within the <var>overflow limit rect</var>, align the <a id="ref-for-alignment-subject④⑧"></a>alignment subject such that it fully covers the [inset-modified containing block](https://www.w3.org/TR/css-position-3/#inset-modified-containing-block) and is otherwise aligned as specified to the extent possible without overflowing the <var>overflow limit rect</var>.

3.  <a id="ref-for-alignment-subject④⑨"></a>

    Otherwise, start-align the [alignment subject](#alignment-subject) within the <var>overflow limit rect</var> (similar to safe).

<a id="ref-for-alignment-subject⑤⓪"></a>

<a id="ref-for-inset-modified-containing-block③"></a>

<a id="ref-for-original-containing-block"></a>

<a id="ref-for-scrollable-overflow-region①"></a>

<a id="ref-for-scroll-container①"></a>

<a id="ref-for-absolute-position①"></a>

<a id="ref-for-fixed-containing-block"></a>

<a id="ref-for-unreachable-scrollable-overflow-region"></a>

For this purpose, the <var>overflow limit rect</var> is the bounding rectangle of the [alignment subject’s](#alignment-subject) [inset-modified containing block](https://www.w3.org/TR/css-position-3/#inset-modified-containing-block) and its [original containing block](https://www.w3.org/TR/css-position-3/#original-containing-block). However, because the [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region) of a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) can be extended to ensure the visibility of overflowing [absolutely positioned](https://www.w3.org/TR/css-position-3/#absolute-position) boxes, if the <a id="ref-for-original-containing-block①"></a>original containing block is generated by a <a id="ref-for-scroll-container②"></a>scroll container (and is not its [fixed containing block](https://www.w3.org/TR/css-position-4/#fixed-containing-block)), the <var>overflow limit rect</var> is extended to infinity in any direction that does not extend into the [unreachable scrollable overflow region](https://www.w3.org/TR/css-overflow-3/#unreachable-scrollable-overflow-region).

<a id="ref-for-absolute-position②"></a>

<a id="ref-for-alignment-subject⑤①"></a>

(For [absolutely-positioned](https://www.w3.org/TR/css-position-3/#absolute-position) [alignment subjects](#alignment-subject) that fail the above condition, see [§ 4.4.1.3 All Other Alignment](#auto-safety-default).)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These rules constrain the position of the box to minimize overflow while honoring the specified alignment insofar as possible, and ensuring continuous behavior as the sizes of the boxes change.

##### <a id="auto-safety-default"></a>4.4.1.3.  All Other Alignment

For all other elements:

1.  <a id="ref-for-alignment-subject⑤②"></a>

    <a id="ref-for-alignment-container②⑦"></a>

    <a id="ref-for-valdef-overflow-position-unsafe③"></a>

    If the [alignment subject](#alignment-subject) overflows its [alignment container](#alignment-container), align as specified ([unsafe](#valdef-overflow-position-unsafe)).

2.  <a id="ref-for-alignment-subject⑤③"></a>

    <a id="ref-for-scrollable-overflow-region②"></a>

    <a id="ref-for-scroll-container③"></a>

    If the [alignment subject](#alignment-subject) would overflow the [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region) of its nearest ancestor [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container), (thus extending into the “unscrollable” region), then its overflow in that direction is limited by biasing any remaining overflow to the opposite side.

<a id="ref-for-valdef-overflow-position-safe③"></a>

<a id="ref-for-propdef-align-content⑥"></a>

<a id="ref-for-block-container②"></a>

<a id="ref-for-valdef-overflow-position-unsafe④"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-6c9376ad"></a> It may not be Web-compatible to implement the “smart” default behavior (though we hope so, and believe it to be likely), so UAs should pass any feedback on this point to the WG. UAs that have not implemented the “smart” default behavior must behave as [safe](#valdef-overflow-position-safe) for [align-content](#propdef-align-content) on [block containers](https://www.w3.org/TR/css-display-4/#block-container) and [unsafe](#valdef-overflow-position-unsafe) otherwise.

## <a id="content-distribution"></a>5.  Content Distribution: Aligning a Box’s Contents Within Itself

<a id="ref-for-propdef-align-content⑦"></a>

<a id="ref-for-propdef-justify-content⑤"></a>

<a id="ref-for-propdef-place-content"></a>

<a id="content-distribute"></a>Content distribution controls alignment of the box’s content within its content box. It is specified by the <a id="content-distribution-properties"></a>content-distribution properties [align-content](#propdef-align-content) and [justify-content](#propdef-justify-content) (and their [place-content](#propdef-place-content) shorthand).

![Diagram showing that the alignment of the content within the element is affected.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/content-example.svg)

<a id="ref-for-propdef-justify-content⑥"></a>

<a id="ref-for-propdef-align-content⑧"></a>

### <a id="align-justify-content"></a>5.1.  The [justify-content](#propdef-justify-content) and [align-content](#propdef-align-content) Properties

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-align-content"></a>align-content                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-content-position②"></a><a id="ref-for-mult-opt①"></a><a id="ref-for-typedef-overflow-position②"></a><a id="ref-for-typedef-content-distribution①"></a><a id="ref-for-typedef-baseline-position①"></a><a id="ref-for-comb-one①⑤"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<baseline-position\>](#typedef-baseline-position) <a id="ref-for-comb-one①⑥"></a>\| [\<content-distribution\>](#typedef-content-distribution) <a id="ref-for-comb-one①⑦"></a>\| [\<overflow-position\>](#typedef-overflow-position)[?](https://www.w3.org/TR/css-values-4/#mult-opt) [\<content-position\>](#typedef-content-position) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block containers, multicol containers, flex containers, and grid containers                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-justify-content"></a>justify-content                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-content-position③"></a><a id="ref-for-mult-opt②"></a><a id="ref-for-typedef-overflow-position③"></a><a id="ref-for-typedef-content-distribution②"></a><a id="ref-for-comb-one①⑧"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<content-distribution\>](#typedef-content-distribution) <a id="ref-for-comb-one①⑨"></a>\| [\<overflow-position\>](#typedef-overflow-position)[?](https://www.w3.org/TR/css-values-4/#mult-opt) \[ [\<content-position\>](#typedef-content-position) <a id="ref-for-comb-one②⓪"></a>\| left <a id="ref-for-comb-one②①"></a>\| right \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | multicol containers, flex containers, and grid containers                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                     |

<a id="ref-for-alignment-subject⑤④"></a>

<a id="ref-for-alignment-container②⑧"></a>

<a id="ref-for-propdef-justify-content⑦"></a>

<a id="ref-for-propdef-align-content⑨"></a>

Aligns the contents of the box as a whole (as the [alignment subject](#alignment-subject)) within the box itself (as the [alignment container](#alignment-container)): along the inline/row/main axis of the box (for [justify-content](#propdef-justify-content)) or the block/column/cross axis of the box (for [align-content](#propdef-align-content)). Values other than <a id="valdef-justify-content-normal"></a>normal are defined in [§ 4 Alignment Keywords](#alignment-values), above.

<a id="ref-for-alignment-subject⑤⑤"></a>

<a id="ref-for-alignment-container②⑨"></a>

<a id="ref-for-writing-mode⑥"></a>

For all layout modes, the [alignment subject](#alignment-subject) and [alignment container](#alignment-container) both assume the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the box the \*-content property is set on.

#### <a id="distribution-block"></a>5.1.1. Block Containers (Including Table Cells)

<a id="ref-for-alignment-container③⓪"></a>**[Alignment Container](#alignment-container)**

<a id="ref-for-block-container③"></a>The [block container](https://www.w3.org/TR/css-display-4/#block-container)’s content box.

<a id="ref-for-alignment-subject⑤⑥"></a>**[Alignment Subject(s)](#alignment-subject)**

The entire contents of the block, as a unit.

<a id="ref-for-propdef-align-content①⓪"></a>**[align-content](#propdef-align-content) Axis**

<a id="ref-for-scroll-container④"></a><a id="ref-for-block-container④"></a><a id="ref-for-typedef-overflow-position④"></a><a id="ref-for-fallback-alignment①②"></a><a id="ref-for-typedef-content-distribution③"></a><a id="ref-for-block-axis⑥"></a>The [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis). If a [\<content-distribution\>](#typedef-content-distribution) is specified its [fallback alignment](#fallback-alignment) is used instead. If no [\<overflow-position\>](#typedef-overflow-position) is specified, and the [block container](https://www.w3.org/TR/css-display-4/#block-container) is not a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container), then alignment is safe.

<a id="ref-for-propdef-justify-content⑧"></a>**[justify-content](#propdef-justify-content) Axis**

<a id="ref-for-block-container⑤"></a>Does not apply to and has no effect on [block containers](https://www.w3.org/TR/css-display-4/#block-container).

<a id="ref-for-valdef-justify-content-normal②"></a>**[normal](#valdef-justify-content-normal) Behavior**

<a id="ref-for-establish-an-independent-formatting-context"></a><a id="ref-for-valdef-justify-content-normal③"></a>All values other than [normal](#valdef-justify-content-normal) force the block container to [establish an independent formatting context](https://www.w3.org/TR/css-display-4/#establish-an-independent-formatting-context).

<a id="ref-for-propdef-align-content①①"></a><a id="ref-for-propdef-vertical-align②"></a><a id="ref-for-valdef-baseline-shift-top"></a><a id="ref-for-valdef-self-position-start②②"></a><a id="ref-for-valdef-baseline-shift-bottom"></a><a id="ref-for-valdef-self-position-end①③"></a><a id="ref-for-valdef-alignment-baseline-middle"></a><a id="ref-for-valdef-self-position-center①"></a><a id="ref-for-valdef-justify-self-baseline④"></a>For table cells, the behavior of [align-content: normal](#propdef-align-content) depends on the computed value of [vertical-align](https://www.w3.org/TR/CSS2/visudet.html#propdef-vertical-align): [top](https://www.w3.org/TR/css-inline-3/#valdef-baseline-shift-top) makes it behave as [start](#valdef-self-position-start) and [bottom](https://www.w3.org/TR/css-inline-3/#valdef-baseline-shift-bottom) makes it behave as [end](#valdef-self-position-end); otherwise [middle](https://www.w3.org/TR/css-inline-3/#valdef-alignment-baseline-middle) makes it behave as [center](#valdef-self-position-center), and all other values make it behave as [baseline](#valdef-justify-self-baseline). [\[CSS2\]](#biblio-css2)

<a id="ref-for-valdef-justify-content-normal④"></a><a id="ref-for-valdef-self-position-start②③"></a>[normal](#valdef-justify-content-normal) otherwise behaves as [start](#valdef-self-position-start).

#### <a id="distribution-multicol"></a>5.1.2. Multicol Containers

<a id="ref-for-alignment-container③①"></a>**[Alignment Container](#alignment-container)**

<a id="ref-for-multi-column-container"></a>The [multi-column container](https://www.w3.org/TR/css-multicol-1/#multi-column-container)’s content box.

<a id="ref-for-alignment-subject⑤⑦"></a>**[Alignment Subject(s)](#alignment-subject)**

The column boxes, with any spacing inserted between column boxes added to the relevant column gaps.

<a id="ref-for-propdef-align-content①②"></a>**[align-content](#propdef-align-content) Axis**

<a id="ref-for-fallback-alignment①③"></a><a id="ref-for-typedef-content-distribution④"></a><a id="ref-for-block-axis⑦"></a>The [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis), treating the column boxes (and any column-spanning elements), as a singular unit. If a [\<content-distribution\>](#typedef-content-distribution) is specified its [fallback alignment](#fallback-alignment) is used instead.

<a id="ref-for-propdef-justify-content⑨"></a>**[justify-content](#propdef-justify-content) Axis**

<a id="ref-for-inline-axis⑥"></a>The [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis).

<a id="ref-for-valdef-justify-content-normal⑤"></a>**[normal](#valdef-justify-content-normal) Behavior**

<a id="ref-for-valdef-align-content-stretch①"></a><a id="ref-for-valdef-justify-content-normal⑥"></a>[normal](#valdef-justify-content-normal) behaves as [stretch](#valdef-align-content-stretch); both are defined as described in the column-sizing rules of [\[CSS-MULTICOL-1\]](#biblio-css-multicol-1).

<a id="ref-for-valdef-column-width-auto"></a><a id="ref-for-propdef-column-width"></a><a id="ref-for-propdef-justify-content①⓪"></a><a id="ref-for-valdef-justify-content-normal⑦"></a><a id="ref-for-valdef-align-content-stretch②"></a>In the case of multi-column containers with a non-[auto](https://www.w3.org/TR/css-multicol-2/#valdef-column-width-auto) [column-width](https://www.w3.org/TR/css-multicol-2/#propdef-column-width), [justify-content](#propdef-justify-content) values other than [normal](#valdef-justify-content-normal) or [stretch](#valdef-align-content-stretch) cause the columns to take their specified <a id="ref-for-propdef-column-width①"></a>column-width rather than stretching to fill the container. The column boxes are then aligned as specified by <a id="ref-for-propdef-justify-content①①"></a>justify-content.

#### <a id="distribution-flex"></a>5.1.3. Flex Containers

<a id="ref-for-alignment-container③②"></a>**[Alignment Container](#alignment-container)**

<a id="ref-for-flex-container⑧"></a>The [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container)’s content box.

<a id="ref-for-alignment-subject⑤⑧"></a>**[Alignment Subject(s)](#alignment-subject)**

<a id="ref-for-flex-line"></a><a id="ref-for-flex-item⑤"></a><a id="ref-for-propdef-justify-content①②"></a>For [justify-content](#propdef-justify-content), the [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item) in each [flex line](https://www.w3.org/TR/css-flexbox-1/#flex-line).

<a id="ref-for-propdef-align-content①③"></a><a id="ref-for-flex-line①"></a><a id="ref-for-multi-line-flex-container"></a>For [align-content](#propdef-align-content), the [flex lines](https://www.w3.org/TR/css-flexbox-1/#flex-line). Note, this only has an effect on [multi-line flex containers](https://www.w3.org/TR/css-flexbox-1/#multi-line-flex-container).

<a id="ref-for-propdef-align-content①④"></a>**[align-content](#propdef-align-content) Axis**

<a id="ref-for-cross-axis②"></a>The [cross axis](https://www.w3.org/TR/css-flexbox-1/#cross-axis).

<a id="ref-for-propdef-justify-content①③"></a>**[justify-content](#propdef-justify-content) Axis**

<a id="ref-for-valdef-self-position-flex-start⑤"></a><a id="ref-for-valdef-align-content-stretch③"></a><a id="ref-for-propdef-flex"></a><a id="ref-for-main-axis②"></a><a id="ref-for-propdef-justify-content①④"></a>The [justify-content](#propdef-justify-content) property applies along the [main axis](https://www.w3.org/TR/css-flexbox-1/#main-axis), but since stretching in the <a id="ref-for-main-axis③"></a>main axis is controlled by [flex](https://www.w3.org/TR/css-flexbox-1/#propdef-flex), [stretch](#valdef-align-content-stretch) behaves as [flex-start](#valdef-self-position-flex-start).

<a id="ref-for-valdef-justify-content-normal⑧"></a>**[normal](#valdef-justify-content-normal) Behavior**

<a id="ref-for-valdef-align-content-stretch④"></a><a id="ref-for-valdef-justify-content-normal⑨"></a>[normal](#valdef-justify-content-normal) behaves as [stretch](#valdef-align-content-stretch).

See [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1) for details.

#### <a id="distribution-grid"></a>5.1.4. Grid Containers

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                              |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-alignment-container③③"></a></span><a href="#alignment-container">Alignment Container</a>&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-grid-container③"></a> The [grid container](https://www.w3.org/TR/css-grid-2/#grid-container)’s content box.                                                                                                                                                                                                                                                               |
| <strong><span><a id="ref-for-alignment-subject⑤⑨"></a></span><a href="#alignment-subject">Alignment Subject(s)</a>&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-collapsed-gutter"></a><a id="ref-for-gutter"></a><a id="ref-for-grid-track"></a> The [grid tracks](https://www.w3.org/TR/css-grid-2/#grid-track) in the appropriate axis, with any spacing inserted between tracks added to the relevant [gutters](#gutter), and treating [collapsed gutters](https://www.w3.org/TR/css-grid-1/#collapsed-gutter) as a single opportunity for space insertion. |
| <strong><span><a id="ref-for-propdef-align-content①⑤"></a></span><a href="#propdef-align-content">align-content</a> Axis&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-grid-row"></a><a id="ref-for-block-axis⑧"></a> The [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis), aligning the [grid rows](https://www.w3.org/TR/css-grid-2/#grid-row).                                                                                                                                                                                   |
| <strong><span><a id="ref-for-propdef-justify-content①⑤"></a></span><a href="#propdef-justify-content">justify-content</a> Axis&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-grid-column"></a><a id="ref-for-inline-axis⑦"></a> The [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis), aligning the [grid columns](https://www.w3.org/TR/css-grid-2/#grid-column).                                                                                                                                                                           |
| <strong><span><a id="ref-for-valdef-justify-content-normal①⓪"></a></span><a href="#valdef-justify-content-normal">normal</a> Behavior&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-valdef-align-content-stretch⑤"></a><a id="ref-for-valdef-justify-content-normal①①"></a> [normal](#valdef-justify-content-normal) behaves as [stretch](#valdef-align-content-stretch).                                                                                                                                                                                                                                    |

See [\[CSS-GRID-1\]](#biblio-css-grid-1) for details.

<a id="ref-for-propdef-place-content①"></a>

### <a id="place-content"></a>5.2.  Content-Distribution Shorthand: the [place-content](#propdef-place-content) property

| Field               | Definition                                                                                                                                                                                                |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-place-content"></a>place-content                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt③"></a><a id="ref-for-propdef-justify-content①⑥"></a><a id="ref-for-propdef-align-content①⑥"></a>[\<'align-content'\>](#propdef-align-content) [\<'justify-content'\>](#propdef-justify-content)[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                  |

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-propdef-align-content①⑦"></a>

<a id="ref-for-propdef-justify-content①⑦"></a>

<a id="ref-for-typedef-baseline-position②"></a>

<a id="ref-for-valdef-self-position-start②④"></a>

This [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) sets both the [align-content](#propdef-align-content) and [justify-content](#propdef-justify-content) properties in one declaration. The first value is assigned to <a id="ref-for-propdef-align-content①⑧"></a>align-content. The second value is assigned to <a id="ref-for-propdef-justify-content①⑧"></a>justify-content; if omitted, it is copied from the first value, unless that value is a [\<baseline-position\>](#typedef-baseline-position) in which case it is defaulted to [start](#valdef-self-position-start).

### <a id="overflow-scroll-position"></a>5.3.  Alignment Overflow and Scroll Containers

<a id="ref-for-content-distribution-properties⑥"></a>

<a id="ref-for-scroll-container⑤"></a>

<a id="ref-for-alignment-subject⑥⓪"></a>

<a id="ref-for-unreachable-scrollable-overflow-region①"></a>

<a id="ref-for-valdef-self-position-start②⑤"></a>

When the [content-distribution properties](#content-distribution-properties) are set on a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) with an overflowing [alignment subject](#alignment-subject), they reduce the clipping of the [unreachable scrollable overflow region](https://www.w3.org/TR/css-overflow-3/#unreachable-scrollable-overflow-region) just enough to ensure that <a id="ref-for-alignment-subject⑥①"></a>alignment subject can be scrolled into its [start](#valdef-self-position-start)-aligned position.

<a id="ref-for-propdef-justify-content①⑨"></a>

<a id="ref-for-propdef-flex-flow"></a>

<a id="ref-for-in-flow"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4f16a383"></a> For example, if a scrollable flex container is set to [justify-content: flex-end](#propdef-justify-content) (or <a id="ref-for-propdef-justify-content②⓪"></a>justify-content: flex-start with [flex-flow: row-reverse](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-flow)), its [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) content will be initially positioned to align the main-end edge of its content to the main-end edge of the flex container, and its content will appear to overflow its main-start edge. However, the viewer will be able to scroll <em>up</em> to view the overflowing <a id="ref-for-in-flow①"></a>in-flow content, just as if <a id="ref-for-propdef-justify-content②①"></a>justify-content: flex-start had been specified.

![](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/scroll-align-padding.jpg)

Issue: Replace this image with a proper SVG.

<a id="ref-for-alignment-subject⑥②"></a>

<a id="ref-for-scrollable-overflow-region③"></a>

<a id="ref-for-out-of-flow"></a>

<a id="ref-for-valdef-self-position-end①④"></a>

<a id="ref-for-scroll-container⑥"></a>

<a id="ref-for-unreachable-scrollable-overflow-region②"></a>

<a id="ref-for-in-flow②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [alignment subject](#alignment-subject) is not necessarily identical to the [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region): content overflowing the <a id="ref-for-alignment-subject⑥③"></a>alignment subject—​such as an [out-of-flow](https://www.w3.org/TR/css-display-4/#out-of-flow) box—​grows the <a id="ref-for-scrollable-overflow-region④"></a>scrollable overflow area but not the <a id="ref-for-alignment-subject⑥④"></a>alignment subject. Thus an [end](#valdef-self-position-end)-aligned [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) might not be initially scrolled all the way to the bottom, and positioned content can still be clipped if it is further into the [unreachable scrollable overflow region](https://www.w3.org/TR/css-overflow-3/#unreachable-scrollable-overflow-region) than the [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) content forming the <a id="ref-for-alignment-subject⑥⑤"></a>alignment subject.

![](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/scroll-align-overflow.jpg)

<a id="ref-for-alignment-subject⑥⑥"></a>

<a id="ref-for-scroll-container⑦"></a>

Overflow is not part of the [alignment subject](#alignment-subject), even for a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f4825563"></a> Replace this image too.

<a id="ref-for-scroll-container⑧"></a>

<a id="ref-for-content-box"></a>

<a id="ref-for-alignment-container③④"></a>

<a id="ref-for-alignment-subject⑥⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The presence of scrollbars can change the size of the [scroll container’s](https://www.w3.org/TR/css-overflow-3/#scroll-container) [content box](https://www.w3.org/TR/css-box-4/#content-box)—​and thus the size of the [alignment container](#alignment-container) and/or [alignment subject](#alignment-subject).

### <a id="baseline-align-content"></a>5.4.  Baseline Content-Alignment

<a id="ref-for-shared-alignment-context③"></a>

<a id="ref-for-alignment-baseline③"></a>

<a id="ref-for-baseline-sharing-group④"></a>

The content of boxes participating in row-like layout contexts ([shared alignment contexts](#shared-alignment-context)) can be baseline-aligned to each other. <a id="baseline-content-alignment"></a>Baseline content-alignment effectively increases the <strong>padding</strong> on the box to align the [alignment baseline](#alignment-baseline) of its contents with that of other baseline-aligned boxes in its [baseline-sharing group](#baseline-sharing-group).

<a id="ref-for-baseline-content-alignment②"></a>

<a id="ref-for-propdef-align-content①⑨"></a>

<a id="ref-for-block-axis⑨"></a>

<a id="ref-for-flex-container⑨"></a>

<a id="ref-for-fallback-alignment①④"></a>

[Baseline content-alignment](#baseline-content-alignment) can only apply if the [align-content](#propdef-align-content) axis is parallel with the box’s [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) (i.e. it does not apply to “column” [flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container)); otherwise the [fallback alignment](#fallback-alignment) is used.

<a id="ref-for-baseline-content-alignment③"></a>

The set of boxes that participate in [baseline content-alignment](#baseline-content-alignment) depends on the layout model:

Table Cells:  
<a id="ref-for-valdef-justify-self-last-baseline③"></a>

<a id="ref-for-valdef-justify-self-first-baseline⑥"></a>

<a id="ref-for-propdef-align-content②⓪"></a>

<a id="ref-for-baseline-content-alignment④"></a>

<a id="ref-for-non-replaced"></a>

A ([non-replaced](https://www.w3.org/TR/css-display-4/#non-replaced)) table cell participates in first/last [baseline content-alignment](#baseline-content-alignment) in its startmost/endmost row if its inline axis is parallel to that of the table itself and its computed [align-content](#propdef-align-content) is [first baseline](#valdef-justify-self-first-baseline) ([last baseline](#valdef-justify-self-last-baseline)).

<a id="ref-for-baseline-content-alignment⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Baseline content-alignment](#baseline-content-alignment) of cells sharing a column is not supported; however this may be added in a future level if there is sufficient demand and implementer interest.

Flex Items:  
<a id="ref-for-main-axis④"></a>

<a id="ref-for-inline-axis⑧"></a>

<a id="ref-for-valdef-justify-self-last-baseline④"></a>

<a id="ref-for-valdef-justify-self-first-baseline⑦"></a>

<a id="ref-for-propdef-align-content②①"></a>

<a id="ref-for-baseline-content-alignment⑥"></a>

<a id="ref-for-flex-item⑥"></a>

<a id="ref-for-non-replaced①"></a>

A [non-replaced](https://www.w3.org/TR/css-display-4/#non-replaced) [flex item](https://www.w3.org/TR/css-flexbox-1/#flex-item) participates in first/last [baseline content-alignment](#baseline-content-alignment) in its flex line if its computed [align-content](#propdef-align-content) is [first baseline](#valdef-justify-self-first-baseline)/[last baseline](#valdef-justify-self-last-baseline) and its [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) is parallel to the [main axis](https://www.w3.org/TR/css-flexbox-1/#main-axis).

Grid Items:  
<a id="ref-for-valdef-justify-self-last-baseline⑤"></a>

<a id="ref-for-valdef-justify-self-first-baseline⑧"></a>

<a id="ref-for-propdef-align-content②②"></a>

<a id="ref-for-inline-axis⑨"></a>

<a id="ref-for-baseline-content-alignment⑦"></a>

<a id="ref-for-grid-item②"></a>

<a id="ref-for-non-replaced②"></a>

A [non-replaced](https://www.w3.org/TR/css-display-4/#non-replaced) [grid item](https://www.w3.org/TR/css-grid-2/#grid-item) participates in first/last [baseline content-alignment](#baseline-content-alignment) in its startmost/endmost row or column (whichever is parallel to its [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis)) and if its computed [align-content](#propdef-align-content) is [first baseline](#valdef-justify-self-first-baseline)/[last baseline](#valdef-justify-self-last-baseline).

<a id="ref-for-baseline-content-alignment⑧"></a>

<a id="ref-for-baseline-sharing-group⑤"></a>

<a id="ref-for-start"></a>

<a id="ref-for-end"></a>

<a id="ref-for-margin-edge"></a>

<a id="ref-for-box-box-edge"></a>

<a id="ref-for-containing-block"></a>

<a id="ref-for-baseline-alignment-preference②"></a>

<a id="ref-for-fallback-alignment①⑤"></a>

Additionally, in order to participate in [baseline content-alignment](#baseline-content-alignment) it must also have a <a id="coordinated-self-alignment-preference"></a>coordinated self-alignment preference, to guarantee that the box lines up the relevant edge with other boxes in its [baseline-sharing group](#baseline-sharing-group). That is, the box’s [start](https://www.w3.org/TR/css-writing-modes-3/#start) ([end](https://www.w3.org/TR/css-writing-modes-3/#end)) [margin edge](https://www.w3.org/TR/css-box-4/#margin-edge) must be intended to align—​and actually align—​to the corresponding [edge](https://www.w3.org/TR/css-box-4/#box-box-edge) of its [containing block](https://www.w3.org/TR/css-display-4/#containing-block) when its [baseline alignment preference](#baseline-alignment-preference) is “first” (“last”). It otherwise takes its [fallback alignment](#fallback-alignment).

> <strong data-conversion-semantic="note">Note</strong>
>
> When is a self-alignment preference coordinated?
>
> <a id="ref-for-start①"></a>
>
> <a id="ref-for-margin-edge①"></a>
>
> <a id="ref-for-containing-block①"></a>
>
> <a id="ref-for-coordinated-self-alignment-preference"></a>
>
> <a id="ref-for-baseline-alignment-preference③"></a>
>
> A box’s [start](https://www.w3.org/TR/css-writing-modes-3/#start) [margin edge](https://www.w3.org/TR/css-box-4/#margin-edge) is aligned to the corresponding edge of its [containing block](https://www.w3.org/TR/css-display-4/#containing-block) and it has a [coordinated self-alignment preference](#coordinated-self-alignment-preference) for a “first” [baseline alignment preference](#baseline-alignment-preference) when, in the relevant axis:
>
> - <a id="ref-for-margin"></a>
>
>   <a id="ref-for-self-alignment-properties④"></a>
>
>   <a id="ref-for-valdef-align-self-stretch"></a>
>
>   <a id="ref-for-valdef-self-position-self-start④"></a>
>
>   There are no auto [margins](https://www.w3.org/TR/css-box-4/#margin) and the relevant [self-alignment property](#self-alignment-properties) either is or aligns identically to [stretch](#valdef-align-self-stretch) or [self-start](#valdef-self-position-self-start); or
>
> - <a id="ref-for-end①"></a>
>
>   <a id="ref-for-margin①"></a>
>
>   <a id="ref-for-self-alignment-properties⑤"></a>
>
>   <a id="ref-for-margin-box"></a>
>
>   <a id="ref-for-containing-block②"></a>
>
>   <a id="ref-for-writing-mode⑦"></a>
>
>   There is only an auto [end](https://www.w3.org/TR/css-writing-modes-3/#end)-edge [margin](https://www.w3.org/TR/css-box-4/#margin), which absorbs any positive free space and disables the effects of any [self-alignment property](#self-alignment-properties), <em>and</em> its [margin box](https://www.w3.org/TR/css-box-4/#margin-box) does not overflow its [containing block](https://www.w3.org/TR/css-display-4/#containing-block) under circumstances that would cause it to effectively end-align instead (such as having a <a id="ref-for-containing-block③"></a>containing block with an opposite [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode)).
>
> <a id="ref-for-end②"></a>
>
> <a id="ref-for-margin-edge②"></a>
>
> <a id="ref-for-containing-block④"></a>
>
> <a id="ref-for-coordinated-self-alignment-preference①"></a>
>
> <a id="ref-for-baseline-alignment-preference④"></a>
>
> A box’s [end](https://www.w3.org/TR/css-writing-modes-3/#end) [margin edge](https://www.w3.org/TR/css-box-4/#margin-edge) is aligned to the corresponding edge of its [containing block](https://www.w3.org/TR/css-display-4/#containing-block) and it has a [coordinated self-alignment preference](#coordinated-self-alignment-preference) for a “last” [baseline alignment preference](#baseline-alignment-preference) when, in the relevant axis:
>
> - <a id="ref-for-margin②"></a>
>
>   <a id="ref-for-self-alignment-properties⑥"></a>
>
>   <a id="ref-for-valdef-self-position-self-end②"></a>
>
>   <a id="ref-for-valdef-overflow-position-unsafe⑤"></a>
>
>   <a id="ref-for-overflow-alignment⑤"></a>
>
>   There are no auto [margins](https://www.w3.org/TR/css-box-4/#margin) and the relevant [self-alignment property](#self-alignment-properties) either is or aligns identically to [self-end](#valdef-self-position-self-end) <em>and</em> its self-alignment is what would result from an [unsafe](#valdef-overflow-position-unsafe) [overflow alignment](#overflow-alignment); or
>
> - <a id="ref-for-start②"></a>
>
>   <a id="ref-for-margin③"></a>
>
>   <a id="ref-for-self-alignment-properties⑦"></a>
>
>   <a id="ref-for-margin-box①"></a>
>
>   <a id="ref-for-containing-block⑤"></a>
>
>   There is only an auto [start](https://www.w3.org/TR/css-writing-modes-3/#start)-edge [margin](https://www.w3.org/TR/css-box-4/#margin), which absorbs any positive free space and disables the effects of any [self-alignment property](#self-alignment-properties) <em>and</em> its [margin box](https://www.w3.org/TR/css-box-4/#margin-box) does not overflow its [containing block](https://www.w3.org/TR/css-display-4/#containing-block) under circumstances that would cause it to effectively start-align instead.

<a id="ref-for-baseline-content-alignment⑨"></a>

See [§ 9.3 Aligning Boxes by Baseline](#align-by-baseline) for additional details. [Baseline content-alignment](#baseline-content-alignment) can increase the intrinsic size of the box.

## <a id="self-alignment"></a>6.  Self-Alignment: Aligning the Box Within Its Parent

<a id="ref-for-propdef-align-self⑨"></a>

<a id="ref-for-propdef-justify-self⑦"></a>

<a id="ref-for-propdef-place-self"></a>

<a id="self-align"></a>Self-alignment controls alignment of the box within its containing block. It is specified by the <a id="self-alignment-properties"></a>self-alignment properties [align-self](#propdef-align-self) and [justify-self](#propdef-justify-self) (and their [place-self](#propdef-place-self) shorthand).

![Diagram showing that the alignment of the element within its containing block is affected.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/self-example.svg)

<a id="ref-for-propdef-justify-self⑧"></a>

### <a id="justify-self-property"></a>6.1.  Inline-Axis (or Main-Axis) Self-Alignment: the [justify-self](#propdef-justify-self) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-justify-self"></a>justify-self                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-baseline-position③"></a><a id="ref-for-typedef-self-position②"></a><a id="ref-for-mult-opt④"></a><a id="ref-for-typedef-overflow-position⑤"></a><a id="ref-for-comb-one②②"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<overflow-position\>](#typedef-overflow-position)[?](https://www.w3.org/TR/css-values-4/#mult-opt) \[ normal <a id="ref-for-comb-one②③"></a>\| [\<self-position\>](#typedef-self-position) <a id="ref-for-comb-one②④"></a>\| left <a id="ref-for-comb-one②⑤"></a>\| right \] <br><a id="ref-for-comb-one②⑥"></a>\| stretch <a id="ref-for-comb-one②⑦"></a>\| [\<baseline-position\>](#typedef-baseline-position) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | block-level boxes, absolutely-positioned boxes, and grid items                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |

<a id="ref-for-alignment-subject⑥⑧"></a>

<a id="ref-for-alignment-container③⑤"></a>

Justifies the box (as the [alignment subject](#alignment-subject)) within its containing block (as the [alignment container](#alignment-container)) along the inline/row/main axis of the <a id="ref-for-alignment-container③⑥"></a>alignment container: the box’s outer edges are aligned within its <a id="ref-for-alignment-container③⑦"></a>alignment container [as described by its alignment value](#alignment-values). Values have the following meanings:

<a id="valdef-justify-self-auto"></a>auto

<a id="ref-for-valdef-justify-self-normal①"></a>

<a id="ref-for-propdef-justify-items④"></a>

<a id="ref-for-valdef-justify-items-legacy①"></a>

Behaves as [normal](#valdef-justify-self-normal) if the box has no parent, or when determining the actual position of an absolutely positioned box. It behaves as the computed [justify-items](#propdef-justify-items) value of the parent box (minus any [legacy](#valdef-justify-items-legacy) keywords) otherwise (including when determining the <em>static</em> position of an absolutely positioned box).

<a id="valdef-justify-self-normal"></a>normal

Represents the “default” alignment for the layout mode. Its behavior depends on the layout mode, as described below.

<a id="ref-for-typedef-self-position③"></a>

[\<self-position\>](#typedef-self-position) \| left \| right

Shifts the position of the box as specified, see [§ 4 Alignment Keywords](#alignment-values).

<a id="ref-for-typedef-overflow-position⑥"></a>

[\<overflow-position\>](#typedef-overflow-position)

Controls how the box aligns when it overflows, see [§ 4.4 Overflow Alignment: the safe and unsafe keywords and scroll safety limits](#overflow-values).

<a id="valdef-justify-self-stretch"></a>stretch

<a id="ref-for-computed-value"></a>

<a id="ref-for-propdef-width"></a>

<a id="ref-for-propdef-height"></a>

<a id="ref-for-valdef-width-auto"></a>

<a id="ref-for-alignment-container③⑧"></a>

<a id="ref-for-propdef-min-height"></a>

<a id="ref-for-propdef-min-width"></a>

<a id="ref-for-propdef-max-height①"></a>

<a id="ref-for-propdef-max-width①"></a>

When the box’s [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)/[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) (as appropriate to the axis) is [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) and neither of its margins (in the appropriate axis) are auto, sets the box’s used size to the length necessary to make its outer size as close to filling the [alignment container](#alignment-container) as possible while still respecting the constraints imposed by [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height)/[min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width)/[max-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-height)/[max-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width).

<a id="ref-for-valdef-self-position-flex-start⑥"></a>

<a id="ref-for-valdef-self-position-self-start⑤"></a>

<a id="ref-for-valdef-self-position-self-end③"></a>

<a id="ref-for-first-baseline-set①"></a>

<a id="ref-for-last-baseline-set①"></a>

<a id="ref-for-baseline-content-alignment①⓪"></a>

Unless otherwise specified, this value falls back to [flex-start](#valdef-self-position-flex-start) generally, and to [self-start](#valdef-self-position-self-start) or [self-end](#valdef-self-position-self-end) if the box has also specified [first baseline](#first-baseline-set) or [last baseline](#last-baseline-set) [baseline content-alignment](#baseline-content-alignment) (respectively) in the same axis.

<a id="ref-for-valdef-justify-self-stretch①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [stretch](#valdef-justify-self-stretch) keyword can cause elements to shrink, to fit their container.

<a id="ref-for-typedef-baseline-position④"></a>

[\<baseline-position\>](#typedef-baseline-position)

<a id="ref-for-baseline-self-alignment④"></a>

Indicates [baseline self-alignment](#baseline-self-alignment), as defined in [§ 4.2 Baseline Alignment: the baseline keyword and first/last modifiers](#baseline-values), [§ 6.4 Baseline Self-Alignment](#baseline-align-self), and [§ 9 Baseline Alignment Details](#baseline-rules).

<a id="ref-for-valdef-justify-self-stretch②"></a>

<a id="ref-for-propdef-width①"></a>

<a id="ref-for-propdef-height①"></a>

<a id="ref-for-valdef-width-auto①"></a>

<a id="ref-for-valdef-width-fit-content"></a>

Values other than [stretch](#valdef-justify-self-stretch) cause a [width](https://www.w3.org/TR/css-sizing-3/#propdef-width)/[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) of [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) to be treated as [fit-content](https://www.w3.org/TR/css-sizing-4/#valdef-width-fit-content).

<a id="ref-for-propdef-justify-self⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: auto margins, because they effectively adjust the size of the margin area, take precedence over [justify-self](#propdef-justify-self).

#### <a id="justify-block"></a>6.1.1. Block-Level Boxes

<a id="ref-for-propdef-justify-self①⓪"></a>**[justify-self](#propdef-justify-self) Axis**

<a id="ref-for-static-position②"></a><a id="ref-for-static-position-containing-block"></a><a id="ref-for-inline-axis①⓪"></a><a id="ref-for-containing-block⑥"></a>The block’s [containing block’s](https://www.w3.org/TR/css-display-4/#containing-block) [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis), generally. The [static-position containing block](https://www.w3.org/TR/css-position-3/#static-position-containing-block)’s <a id="ref-for-inline-axis①①"></a>inline axis when determining the [static position](https://www.w3.org/TR/css-position-3/#static-position).

<a id="ref-for-alignment-container③⑨"></a>**[Alignment Container](#alignment-container)**

<a id="ref-for-writing-mode⑧"></a><a id="ref-for-alignment-container④⓪"></a><a id="ref-for-block-formatting-context"></a><a id="ref-for-containing-block⑦"></a>The box’s [containing block](https://www.w3.org/TR/css-display-4/#containing-block), except that for block-level elements that establish a [block formatting context](https://www.w3.org/TR/css-display-4/#block-formatting-context) and are placed next to a float, the [alignment container](#alignment-container) is reduced by the space taken up by the float, assuming the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the <a id="ref-for-containing-block⑧"></a>containing block.

<a id="ref-for-alignment-subject⑥⑨"></a>**[Alignment Subject](#alignment-subject)**

<a id="ref-for-writing-mode⑨"></a>The block’s margin box, assuming the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the block.

<a id="ref-for-valdef-justify-self-normal②"></a>**[normal](#valdef-justify-self-normal) Behavior**

The box lays out according to the default rules for block layout (see [CSS2.1§10.3](https://www.w3.org/TR/CSS2/visudet.html#Computing_widths_and_margins)).

**Other Details**

<a id="ref-for-non-replaced③"></a><a id="ref-for-valdef-width-stretch"></a><a id="ref-for-valdef-width-fit-content①"></a><a id="ref-for-valdef-justify-self-normal③"></a><a id="ref-for-propdef-justify-self①①"></a><a id="ref-for-block-level-box"></a><a id="ref-for-automatic-size"></a>The [automatic size](https://www.w3.org/TR/css-sizing-3/#automatic-size) of a [block-level box](https://www.w3.org/TR/css-display-4/#block-level-box) whose [justify-self](#propdef-justify-self) is not [normal](#valdef-justify-self-normal) is equivalent to [fit-content](https://www.w3.org/TR/css-sizing-4/#valdef-width-fit-content) (rather than e.g. [stretch](https://www.w3.org/TR/css-sizing-4/#valdef-width-stretch), as is typical for a [non-replaced](https://www.w3.org/TR/css-display-4/#non-replaced) <a id="ref-for-block-level-box①"></a>block-level box). Additionally, in terms of CSS2.1 block-level formatting [\[CSS2\]](#biblio-css2), the rules for “over-constrained” computations in [section 10.3.3](https://www.w3.org/TR/CSS2/visudet.html#blockwidth) are ignored in favor of alignment as specified here and the used value of the margin properties are therefore not adjusted to correct for the over-constraint.

This property does not apply to floats.

<a id="ref-for-valdef-justify-self-normal④"></a>Anonymous block boxes always behave as [normal](#valdef-justify-self-normal).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cd8dae8e"></a> The effect of these rules is that an auto-sized block-level table, for example, can be aligned while still having side margins. If the table’s max-content size is narrower than its containing block, then it is shrink-wrapped to that size and aligned as specified. If the table’s max-content size is wider, then it fills its containing block, and the margins provide appropriate spacing from the containing block edges.

#### <a id="justify-abspos"></a>6.1.2. Absolutely-Positioned Boxes

<a id="ref-for-propdef-justify-self①②"></a>

This section describes the effect of [justify-self](#propdef-justify-self) on how the margin box of an absolutely-positioned box is positioned with respect to its (absolute-positioning) containing block.

<a id="ref-for-propdef-justify-self①③"></a>**[justify-self](#propdef-justify-self) Axis**

<a id="ref-for-inline-axis①②"></a>The block’s containing block’s [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis).

<a id="ref-for-alignment-container④①"></a>**[Alignment Container](#alignment-container)**

<a id="ref-for-static-position-containing-block①"></a><a id="ref-for-static-position③"></a><a id="ref-for-static-position-rectangle"></a><a id="ref-for-valdef-top-auto①"></a><a id="ref-for-writing-mode①⓪"></a><a id="ref-for-propdef-left"></a><a id="ref-for-propdef-bottom"></a><a id="ref-for-propdef-right"></a><a id="ref-for-propdef-top"></a><a id="ref-for-inset-properties"></a><a id="ref-for-containing-block⑨"></a>The box’s [containing block](https://www.w3.org/TR/css-display-4/#containing-block), as modified by the [inset properties](https://www.w3.org/TR/css-logical-1/#inset-properties) ([top](https://www.w3.org/TR/css-position-3/#propdef-top)/[right](https://www.w3.org/TR/css-position-3/#propdef-right)/[bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom)/[left](https://www.w3.org/TR/css-position-3/#propdef-left)), assuming the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the <a id="ref-for-containing-block①⓪"></a>containing block. If both inset properties in the relevant axis are [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto), then use the box’s [static-position rectangle](#static-position-rectangle) (i.e. set both insets to the box’s [static position](https://www.w3.org/TR/css-position-3/#static-position)) and assume the <a id="ref-for-writing-mode①①"></a>writing mode of the [static-position containing block](https://www.w3.org/TR/css-position-3/#static-position-containing-block).

<a id="ref-for-alignment-subject⑦⓪"></a>**[Alignment Subject](#alignment-subject)**

<a id="ref-for-writing-mode①②"></a>The box’s margin box, assuming the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the box.

<a id="ref-for-valdef-justify-self-normal⑤"></a>**[normal](#valdef-justify-self-normal) Behavior**

<a id="ref-for-valdef-self-position-start②⑥"></a><a id="ref-for-valdef-justify-self-stretch③"></a>Behaves as [stretch](#valdef-justify-self-stretch) or [start](#valdef-self-position-start), depending on the type of box. See [CSS Positioned Layout 3 § 4 Absolute Positioning Layout Model](https://www.w3.org/TR/css-position-3/#abspos-layout).

**Other Details**

<a id="ref-for-inset-properties①"></a>In terms of CSS2.1 formatting [\[CSS2\]](#biblio-css2), the rules for “over-constrained” computations in [section 10.3.7](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-width) are ignored in favor of alignment as specified here, and the used value of the [inset properties](https://www.w3.org/TR/css-logical-1/#inset-properties) are not adjusted to correct for the over-constraint.

<a id="ref-for-valdef-justify-self-stretch④"></a><a id="ref-for-valdef-justify-self-normal⑥"></a><a id="ref-for-fit-content-size"></a><a id="ref-for-valdef-width-auto②"></a>Values other than [stretch](#valdef-justify-self-stretch) or [normal](#valdef-justify-self-normal) cause [non-replaced absolutely-positioned boxes](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-width) to use [fit-content sizing](https://www.w3.org/TR/css-sizing-3/#fit-content-size) for calculating [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) sizes in the affected axis.

<a id="ref-for-valdef-justify-self-stretch⑤"></a>Note that [stretch](#valdef-justify-self-stretch) <em>does</em> cause replaced absolutely-positioned boxes to fill their containing block just as non-replaced ones do.

<a id="ref-for-valdef-top-auto②"></a><a id="ref-for-propdef-justify-self①④"></a><a id="ref-for-static-position-rectangle①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If only one inset property is [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto), the computations in [CSS2 section 10.3.7](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-width) fully determine its size and position, and [justify-self](#propdef-justify-self) has no effect. (If both are <a id="ref-for-valdef-top-auto③"></a>auto, then the box is statically-positioned, see above, and can be aligned within the [static-position rectangle](#static-position-rectangle).)

#### <a id="justify-cell"></a>6.1.3. Table Cells

This property does not apply to table cells, because their position and size is fully constrained by table layout.

#### <a id="justify-flex"></a>6.1.4. Flex Items

<a id="ref-for-flex-item⑦"></a>

<a id="ref-for-main-axis⑤"></a>

<a id="ref-for-propdef-flex①"></a>

<a id="ref-for-propdef-justify-content②②"></a>

This property does not apply to [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item), because there is more than one item in the [main axis](https://www.w3.org/TR/css-flexbox-1/#main-axis). See [flex](https://www.w3.org/TR/css-flexbox-1/#propdef-flex) for stretching and [justify-content](#propdef-justify-content) for <a id="ref-for-main-axis⑥"></a>main-axis alignment. [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1)

#### <a id="justify-grid"></a>6.1.5. Grid Items

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-justify-self①⑤"></a></span><a href="#propdef-justify-self">justify-self</a> Axis&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-inline-axis①③"></a> The grid’s [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis).                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><span><a id="ref-for-alignment-container④②"></a></span><a href="#alignment-container">Alignment Container</a>&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-grid-container④"></a><a id="ref-for-writing-mode①③"></a><a id="ref-for-grid-area"></a><a id="ref-for-grid-item③"></a> The [grid item’s](https://www.w3.org/TR/css-grid-2/#grid-item) [grid area](https://www.w3.org/TR/css-grid-2/#grid-area), assuming the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the [grid container](https://www.w3.org/TR/css-grid-2/#grid-container).                                                                                                                                                |
| <strong><span><a id="ref-for-alignment-subject⑦①"></a></span><a href="#alignment-subject">Alignment Subject</a>&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-writing-mode①④"></a><a id="ref-for-grid-item④"></a> The [grid item’s](https://www.w3.org/TR/css-grid-2/#grid-item) margin box, assuming the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the <a id="ref-for-grid-item⑤"></a>grid item.                                                                                                                                                                                                                                                                          |
| <strong><span><a id="ref-for-valdef-justify-self-normal⑦"></a></span><a href="#valdef-justify-self-normal">normal</a> Behavior&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-replaced-element"></a><a id="ref-for-valdef-self-position-start②⑦"></a><a id="ref-for-non-replaced④"></a><a id="ref-for-valdef-justify-self-stretch⑥"></a> Sizes as either [stretch](#valdef-justify-self-stretch) (typical [non-replaced elements](https://www.w3.org/TR/css-display-4/#non-replaced)) or [start](#valdef-self-position-start) (typical [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element)); see [Grid Item Sizing](https://www.w3.org/TR/css-grid-1/#grid-item-sizing) in [\[CSS-GRID-1\]](#biblio-css-grid-1). The resulting box is then start-aligned. |

See [\[CSS-GRID-1\]](#biblio-css-grid-1) for details.

<a id="ref-for-propdef-align-self①⓪"></a>

### <a id="align-self-property"></a>6.2.  Block-Axis (or Cross-Axis) Self-Alignment: the [align-self](#propdef-align-self) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-align-self"></a>align-self                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-baseline-position⑤"></a><a id="ref-for-typedef-self-position④"></a><a id="ref-for-mult-opt⑤"></a><a id="ref-for-typedef-overflow-position⑦"></a><a id="ref-for-comb-one②⑧"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<overflow-position\>](#typedef-overflow-position)[?](https://www.w3.org/TR/css-values-4/#mult-opt) \[ normal <a id="ref-for-comb-one②⑨"></a>\| [\<self-position\>](#typedef-self-position) \]<br><a id="ref-for-comb-one③⓪"></a>\| stretch <a id="ref-for-comb-one③①"></a>\| [\<baseline-position\>](#typedef-baseline-position) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | flex items, grid items, and absolutely-positioned boxes                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                           |

<a id="ref-for-alignment-subject⑦②"></a>

<a id="ref-for-alignment-container④③"></a>

Aligns the box (as the [alignment subject](#alignment-subject)) within its containing block (as the [alignment container](#alignment-container)) along the block/column/cross axis of the <a id="ref-for-alignment-container④④"></a>alignment container: the box’s outer edges are aligned within its <a id="ref-for-alignment-container④⑤"></a>alignment container [as described by its alignment value](#alignment-values). Values have the following meanings:

<a id="valdef-align-self-auto"></a>auto

<a id="ref-for-valdef-align-self-normal"></a>

<a id="ref-for-propdef-align-items②"></a>

<a id="ref-for-valdef-justify-items-legacy②"></a>

Behaves as [normal](#valdef-align-self-normal) if the box has no parent, or when determining the actual position of an absolutely positioned box. It behaves as the computed [align-items](#propdef-align-items) value of the parent box (minus any [legacy](#valdef-justify-items-legacy) keywords) otherwise (including when determining the <em>static</em> position of an absolutely positioned box).

<a id="valdef-align-self-normal"></a>normal

Represents the “default” alignment for the layout mode, as defined below.

<a id="ref-for-typedef-self-position⑤"></a>

[\<self-position\>](#typedef-self-position)

Shifts the position of the box as specified, see [§ 4 Alignment Keywords](#alignment-values).

<a id="ref-for-typedef-overflow-position⑧"></a>

[\<overflow-position\>](#typedef-overflow-position)

Controls how the box aligns when it overflows, see [§ 4.4 Overflow Alignment: the safe and unsafe keywords and scroll safety limits](#overflow-values).

<a id="valdef-align-self-stretch"></a>stretch

<a id="ref-for-propdef-justify-self①⑥"></a>

As defined for [justify-self](#propdef-justify-self) in [§ 6.1 Inline-Axis (or Main-Axis) Self-Alignment: the justify-self property](#justify-self-property).

<a id="ref-for-typedef-baseline-position⑥"></a>

[\<baseline-position\>](#typedef-baseline-position)

<a id="ref-for-baseline-self-alignment⑤"></a>

Indicates [baseline self-alignment](#baseline-self-alignment), as defined in [§ 4.2 Baseline Alignment: the baseline keyword and first/last modifiers](#baseline-values), [§ 6.4 Baseline Self-Alignment](#baseline-align-self), and [§ 9 Baseline Alignment Details](#baseline-rules).

<a id="ref-for-propdef-align-self①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: auto margins, because they effectively adjust the size of the margin area, take precedence over [align-self](#propdef-align-self).

#### <a id="align-block"></a>6.2.1. Block-Level Boxes

<a id="ref-for-propdef-align-self①②"></a>

<a id="ref-for-block-axis①⓪"></a>

The [align-self](#propdef-align-self) property does not apply to block-level boxes (including floats), because there is more than one item in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis).

#### <a id="align-abspos"></a>6.2.2. Absolutely-Positioned Boxes

<a id="ref-for-propdef-align-self①③"></a>

This section describes the effect of [align-self](#propdef-align-self) on how the margin box of an absolutely-positioned box is positioned with respect to its (absolute-positioning) containing block.

<a id="ref-for-propdef-align-self①④"></a>**[align-self](#propdef-align-self) Axis**

<a id="ref-for-static-position④"></a><a id="ref-for-static-position-containing-block②"></a><a id="ref-for-block-axis①①"></a><a id="ref-for-containing-block①①"></a>The box’s [containing block’s](https://www.w3.org/TR/css-display-4/#containing-block) [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis), generally. The [static-position containing block](https://www.w3.org/TR/css-position-3/#static-position-containing-block)’s <a id="ref-for-block-axis①②"></a>block axis when determining the [static position](https://www.w3.org/TR/css-position-3/#static-position).

<a id="ref-for-alignment-container④⑥"></a>**[Alignment Container](#alignment-container)**

<a id="ref-for-static-position-containing-block③"></a><a id="ref-for-static-position⑤"></a><a id="ref-for-static-position-rectangle②"></a><a id="ref-for-valdef-top-auto④"></a><a id="ref-for-writing-mode①⑤"></a><a id="ref-for-propdef-left①"></a><a id="ref-for-propdef-bottom①"></a><a id="ref-for-propdef-right①"></a><a id="ref-for-propdef-top①"></a><a id="ref-for-inset-properties②"></a><a id="ref-for-containing-block①②"></a>The box’s [containing block](https://www.w3.org/TR/css-display-4/#containing-block), as modified by the [inset properties](https://www.w3.org/TR/css-logical-1/#inset-properties) ([top](https://www.w3.org/TR/css-position-3/#propdef-top)/[right](https://www.w3.org/TR/css-position-3/#propdef-right)/[bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom)/[left](https://www.w3.org/TR/css-position-3/#propdef-left)), assuming the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the <a id="ref-for-containing-block①③"></a>containing block. If both inset properties in the relevant axis are [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto), then use the box’s [static-position rectangle](#static-position-rectangle) (i.e. set both insets to the box’s [static position](https://www.w3.org/TR/css-position-3/#static-position)) and assume the <a id="ref-for-writing-mode①⑥"></a>writing mode of the [static-position containing block](https://www.w3.org/TR/css-position-3/#static-position-containing-block).

<a id="ref-for-alignment-subject⑦③"></a>**[Alignment Subject](#alignment-subject)**

<a id="ref-for-writing-mode①⑦"></a>The box’s margin box, assuming the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the box.

<a id="ref-for-valdef-align-self-normal①"></a>**[normal](#valdef-align-self-normal) Behavior**

<a id="ref-for-valdef-self-position-start②⑧"></a><a id="ref-for-valdef-align-self-stretch①"></a>Behaves as [stretch](#valdef-align-self-stretch) or [start](#valdef-self-position-start), depending on the type of box. See [CSS Positioned Layout 3 § 4 Absolute Positioning Layout Model](https://www.w3.org/TR/css-position-3/#abspos-layout).

**Other Details**

<a id="ref-for-inset-properties③"></a>In terms of CSS2.1 formatting [\[CSS2\]](#biblio-css2), the rules for "over-constrained" computations in [section 10.6.4](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-height) are ignored in favor of alignment as specified here and the used value of the [inset properties](https://www.w3.org/TR/css-logical-1/#inset-properties) are not adjusted to correct for the over-constraint.

<a id="ref-for-valdef-justify-self-stretch⑦"></a><a id="ref-for-valdef-justify-self-normal⑧"></a><a id="ref-for-fit-content-size①"></a><a id="ref-for-valdef-width-auto③"></a>Values other than [stretch](#valdef-justify-self-stretch) or [normal](#valdef-justify-self-normal) cause [non-replaced absolutely-positioned boxes](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-height) to use [fit-content sizing](https://www.w3.org/TR/css-sizing-3/#fit-content-size) for calculating [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) sizes in the affected axis.

<a id="ref-for-valdef-justify-self-stretch⑧"></a>Note that [stretch](#valdef-justify-self-stretch) does cause replaced absolutely-positioned boxes to fill their containing block just as non-replaced ones do.

<a id="ref-for-valdef-top-auto⑤"></a><a id="ref-for-propdef-align-self①⑤"></a><a id="ref-for-static-position-rectangle③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If only one inset property is [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto), the computations in [CSS2 section 10.6.4](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-height) fully determine its size and position, and [align-self](#propdef-align-self) has no effect. (If both are <a id="ref-for-valdef-top-auto⑥"></a>auto, then the box is statically-positioned, see above, and can be aligned within the [static-position rectangle](#static-position-rectangle).)

#### <a id="align-cell"></a>6.2.3. Table Cells

This property does not apply to table cells, because their position and size is fully constrained by table layout.

#### <a id="align-flex"></a>6.2.4. Flex Items

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                             |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-align-self①⑥"></a></span><a href="#propdef-align-self">align-self</a> Axis&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-cross-axis③"></a><a id="ref-for-flex-container①⓪"></a> The [flex container’s](https://www.w3.org/TR/css-flexbox-1/#flex-container) [cross axis](https://www.w3.org/TR/css-flexbox-1/#cross-axis).                                                                                                                                                                                                      |
| <strong><span><a id="ref-for-alignment-container④⑦"></a></span><a href="#alignment-container">Alignment Container</a>&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-flex-container①①"></a><a id="ref-for-writing-mode①⑧"></a><a id="ref-for-flex-item⑧"></a><a id="ref-for-flex-line②"></a> The [flex line](https://www.w3.org/TR/css-flexbox-1/#flex-line) the [flex item](https://www.w3.org/TR/css-flexbox-1/#flex-item) is in, assuming the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container). |
| <strong><span><a id="ref-for-alignment-subject⑦④"></a></span><a href="#alignment-subject">Alignment Subject</a>&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-writing-mode①⑨"></a><a id="ref-for-flex-item⑨"></a> The [flex item’s](https://www.w3.org/TR/css-flexbox-1/#flex-item) margin box, assuming the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the <a id="ref-for-flex-item①⓪"></a>flex item.                                                                                                                                         |
| <strong><span><a id="ref-for-valdef-align-self-normal②"></a></span><a href="#valdef-align-self-normal">normal</a> Behavior&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-valdef-align-self-stretch②"></a> Behaves as [stretch](#valdef-align-self-stretch).                                                                                                                                                                                                                                                                                                                  |

See [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1) for details.

#### <a id="align-grid"></a>6.2.5. Grid Items

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-align-self①⑦"></a></span><a href="#propdef-align-self">align-self</a> Axis&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-block-axis①③"></a> The grid’s [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis).                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong><span><a id="ref-for-alignment-container④⑧"></a></span><a href="#alignment-container">Alignment Container</a>&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-grid-container⑤"></a><a id="ref-for-writing-mode②⓪"></a><a id="ref-for-grid-area①"></a><a id="ref-for-grid-item⑥"></a> The [grid item’s](https://www.w3.org/TR/css-grid-2/#grid-item) [grid area](https://www.w3.org/TR/css-grid-2/#grid-area), assuming the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the [grid container](https://www.w3.org/TR/css-grid-2/#grid-container).                                                                                                                                                |
| <strong><span><a id="ref-for-alignment-subject⑦⑤"></a></span><a href="#alignment-subject">Alignment Subject</a>&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-writing-mode②①"></a><a id="ref-for-grid-item⑦"></a> The [grid item’s](https://www.w3.org/TR/css-grid-2/#grid-item) margin box, assuming the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the <a id="ref-for-grid-item⑧"></a>grid item.                                                                                                                                                                                                                                                                          |
| <strong><span><a id="ref-for-valdef-align-self-normal③"></a></span><a href="#valdef-align-self-normal">normal</a> Behavior&#xA;&#x9;&#x9;&#x9;&#xA;      </strong> | <a id="ref-for-replaced-element①"></a><a id="ref-for-valdef-self-position-start②⑨"></a><a id="ref-for-non-replaced⑤"></a><a id="ref-for-valdef-justify-self-stretch⑨"></a> Sizes as either [stretch](#valdef-justify-self-stretch) (typical [non-replaced elements](https://www.w3.org/TR/css-display-4/#non-replaced)) or [start](#valdef-self-position-start) (typical [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element)); see [Grid Item Sizing](https://www.w3.org/TR/css-grid-1/#grid-item-sizing) in [\[CSS-GRID-1\]](#biblio-css-grid-1). The resulting box is then start-aligned. |

See [\[CSS-GRID-1\]](#biblio-css-grid-1) for details.

<a id="ref-for-propdef-place-self①"></a>

### <a id="place-self-property"></a>6.3.  Self-Alignment Shorthand: the [place-self](#propdef-place-self) property

| Field               | Definition                                                                                                                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-place-self"></a>place-self                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt⑥"></a><a id="ref-for-propdef-justify-self①⑦"></a><a id="ref-for-propdef-align-self①⑧"></a>[\<'align-self'\>](#propdef-align-self) [\<'justify-self'\>](#propdef-justify-self)[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                      |

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-propdef-align-self①⑨"></a>

<a id="ref-for-propdef-justify-self①⑧"></a>

This [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) sets both the [align-self](#propdef-align-self) and [justify-self](#propdef-justify-self) properties in a single declaration. The first value is assigned to <a id="ref-for-propdef-align-self②⓪"></a>align-self. The second value is assigned to <a id="ref-for-propdef-justify-self①⑨"></a>justify-self; if omitted, it is copied from the first value.

### <a id="baseline-align-self"></a>6.4.  Baseline Self-Alignment

<a id="ref-for-shared-alignment-context④"></a>

<a id="ref-for-alignment-baseline④"></a>

<a id="ref-for-baseline-sharing-group⑥"></a>

Boxes participating in row-like layout contexts ([shared alignment contexts](#shared-alignment-context)) can be baseline-aligned to each other. <a id="baseline-self-alignment"></a>Baseline self-alignment effectively increases the <strong>margins</strong> on the box to align its [alignment baseline](#alignment-baseline) with that of other baseline-aligned boxes in its [baseline-sharing group](#baseline-sharing-group).

<a id="ref-for-baseline-self-alignment⑥"></a>

The set of boxes that participate in [baseline self-alignment](#baseline-self-alignment) depends on the layout model:

Flex Items:  
<a id="ref-for-valdef-justify-self-last-baseline⑥"></a>

<a id="ref-for-valdef-justify-self-first-baseline⑨"></a>

<a id="ref-for-propdef-align-self②①"></a>

<a id="ref-for-baseline-self-alignment⑦"></a>

<a id="ref-for-flex-item①①"></a>

A [flex item](https://www.w3.org/TR/css-flexbox-1/#flex-item) participates in first/last [baseline self-alignment](#baseline-self-alignment) in its flex line if its computed [align-self](#propdef-align-self) is [first baseline](#valdef-justify-self-first-baseline)/[last baseline](#valdef-justify-self-last-baseline). See [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1) for details.

Grid Items:  
<a id="ref-for-valdef-justify-self-last-baseline⑦"></a>

<a id="ref-for-valdef-justify-self-first-baseline①⓪"></a>

<a id="ref-for-propdef-justify-self②⓪"></a>

<a id="ref-for-propdef-align-self②②"></a>

<a id="ref-for-baseline-self-alignment⑧"></a>

<a id="ref-for-grid-item⑨"></a>

A [grid item](https://www.w3.org/TR/css-grid-2/#grid-item) participates in first/last [baseline self-alignment](#baseline-self-alignment) in its startmost/endmost row or column if its [align-self](#propdef-align-self) or [justify-self](#propdef-justify-self) property (respectively) computes to [first baseline](#valdef-justify-self-first-baseline)/[last baseline](#valdef-justify-self-last-baseline).

<a id="ref-for-baseline-self-alignment⑨"></a>

See [§ 9.3 Aligning Boxes by Baseline](#align-by-baseline) for exact details. [Baseline self-alignment](#baseline-self-alignment) can increase the intrinsic size contribution of the box.

### <a id="abspos-sizing"></a>6.5.  Effects on Sizing of Absolutely Positioned Boxes with Static-Position Insets

<a id="ref-for-valdef-top-auto⑦"></a>

<a id="ref-for-available"></a>

<a id="ref-for-inline-size"></a>

For absolutely-positioned boxes whose inline-axis offsets are both [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto), the [available space](https://www.w3.org/TR/css-sizing-3/#available) for calculating the [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) is also affected by alignment.

<a id="ref-for-available①"></a>

<a id="ref-for-propdef-direction"></a>

<a id="ref-for-static-position-containing-block④"></a>

<a id="ref-for-valdef-top-auto⑧"></a>

<a id="ref-for-static-position-rectangle④"></a>

<a id="ref-for-containing-block①④"></a>

<a id="ref-for-self-alignment-properties⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In [\[CSS2\]](#biblio-css2), the [available space](https://www.w3.org/TR/css-sizing-3/#available) is keyed off of the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) property of the [static-position containing block](https://www.w3.org/TR/css-position-3/#static-position-containing-block). (See [CSS2§10.3.7](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-width) and [CSS2§10.3.8](https://www.w3.org/TR/CSS2/visudet.html#abs-replaced-width).) Fundamentally these rules set one of the [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto) insets (by default, the start-edge inset) to the corresponding edge of the [static-position rectangle](#static-position-rectangle) and the other to the corresponding edge of the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) (i.e. set the inset to zero). Just as the [self-alignment properties](#self-alignment-properties) replace the <a id="ref-for-containing-block①⑤"></a>containing block’s <a id="ref-for-propdef-direction①"></a>direction lookup for placement, they also replace this lookup for sizing, as specified here.

<a id="ref-for-propdef-direction②"></a>

<a id="ref-for-static-position-containing-block⑤"></a>

<a id="ref-for-propdef-align-self②③"></a>

<a id="ref-for-propdef-justify-self②①"></a>

<a id="ref-for-valdef-direction-ltr"></a>

<a id="ref-for-valdef-direction-rtl"></a>

<a id="ref-for-valdef-justify-self-normal⑨"></a>

<a id="ref-for-valdef-self-position-start③⓪"></a>

<a id="ref-for-distributed-alignment①"></a>

<a id="ref-for-fallback-alignment①⑥"></a>

Thus, when interpreting the rules in [CSS2§10.3.7](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-width) and [CSS2§10.3.8](https://www.w3.org/TR/CSS2/visudet.html#abs-replaced-width), wherever the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) property of the [static-position containing block](https://www.w3.org/TR/css-position-3/#static-position-containing-block) is referenced, instead reference the value of the [align-self](#propdef-align-self) or [justify-self](#propdef-justify-self) property (whichever is defined to apply to the relevant axis), treating left-equivalent alignment as defined for [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr) and right-equivalent alignment as defined for [rtl](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-rtl). Treat [normal](#valdef-justify-self-normal) as [start](#valdef-self-position-start) and any [distributed alignment](#distributed-alignment) value as its [fallback alignment](#fallback-alignment).

<a id="ref-for-valdef-self-position-center②"></a>

<a id="ref-for-available②"></a>

<a id="ref-for-static-position-rectangle⑤"></a>

<a id="ref-for-containing-block①⑥"></a>

In the case of [center](#valdef-self-position-center) alignment, the [available space](https://www.w3.org/TR/css-sizing-3/#available) for the box is double the distance between the center of the [static-position rectangle](#static-position-rectangle) and the closest edge of the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) in the relevant axis.

![Start alignment sizes into the space between the start edge of the static-position rectangle and the end edge of the containing block. End alignment sizes into the space between the end edge of the static-position rectangle and the start edge of the containing block. Center alignment sizes into the space between the two edges of the static-position rectangle.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/place-content-abspos.svg)

<a id="ref-for-inline-start"></a>

<a id="ref-for-static-position⑥"></a>

<a id="ref-for-inline-end"></a>

<a id="ref-for-containing-block①⑦"></a>

<a id="ref-for-valdef-top-auto⑨"></a>

<a id="ref-for-static-position-rectangle⑥"></a>

<a id="ref-for-self-align②"></a>

Instead of always sizing within the available space between the [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) [static position](https://www.w3.org/TR/css-position-3/#static-position) and the [inline-end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) [containing block](https://www.w3.org/TR/css-display-4/#containing-block) edge as specified in [\[CSS2\]](#biblio-css2), an absolutely-positioned element with [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto) insets will be sized with reference to the [static-position rectangle](#static-position-rectangle)’s edge(s) <em>most appropriate</em> to its specified [self-alignment](#self-align).

<a id="ref-for-containing-block①⑧"></a>

<a id="ref-for-propdef-direction③"></a>

<a id="ref-for-valdef-direction-ltr①"></a>

<a id="ref-for-propdef-justify-self②②"></a>

<a id="ref-for-valdef-self-position-end①⑤"></a>

<a id="ref-for-valdef-direction-rtl①"></a>

<a id="ref-for-propdef-justify-content②③"></a>

<a id="ref-for-valdef-align-content-space-between②"></a>

<a id="ref-for-valdef-self-position-center③"></a>

<a id="ref-for-static-position⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d884518f"></a> For example, when the box’s [containing block](https://www.w3.org/TR/css-display-4/#containing-block)’s [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) is [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr) and its own [justify-self](#propdef-justify-self) is [end](#valdef-self-position-end), apply the rules for <a id="ref-for-propdef-direction④"></a>direction: rtl; when <a id="ref-for-propdef-direction⑤"></a>direction is [rtl](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-rtl) and [justify-content](#propdef-justify-content) is [space-between](#valdef-align-content-space-between), apply the rules for <a id="ref-for-propdef-direction⑥"></a>direction: rtl; etc. For the case of [center](#valdef-self-position-center) (or its equivalent), set both sides to match the [static position](https://www.w3.org/TR/css-position-3/#static-position). The absolutely-positioned box is then sized into the resulting space (floored at zero).

<a id="ref-for-propdef-align-self②④"></a>

<a id="ref-for-propdef-justify-self②③"></a>

<a id="ref-for-fit-content-size②"></a>

<a id="ref-for-available③"></a>

<a id="ref-for-stretch-fit-size"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [align-self](#propdef-align-self)/[justify-self](#propdef-justify-self) properties can also modify additional aspects of sizing: for example, <a id="ref-for-propdef-justify-self②④"></a>justify-self: stretch will replace “shrink-to-fit” ([fit-content](https://www.w3.org/TR/css-sizing-3/#fit-content-size)) sizing into the [available space](https://www.w3.org/TR/css-sizing-3/#available) with [stretch-fit sizing](https://www.w3.org/TR/css-sizing-3/#stretch-fit-size) (consuming all of the <a id="ref-for-available④"></a>available space). This is an independent effect from the available space adjustment here.

<a id="ref-for-available⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This only affect how [available space](https://www.w3.org/TR/css-sizing-3/#available) is calculated for sizing the absolutely-positioned box; its alignment is as specified in previous sections.

## <a id="default-alignment"></a>7.  Default Alignment

<a id="ref-for-propdef-align-items③"></a>

<a id="ref-for-propdef-justify-items⑤"></a>

<a id="ref-for-propdef-place-items"></a>

<a id="ref-for-propdef-align-self②⑤"></a>

<a id="ref-for-propdef-justify-self②⑤"></a>

The [align-items](#propdef-align-items) and [justify-items](#propdef-justify-items) properties (and their [place-items](#propdef-place-items) shorthand) set the default [align-self](#propdef-align-self) and [justify-self](#propdef-justify-self) behavior of the element’s child boxes.

![Diagram showing that the alignment of grid items within the element is affected.](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/items-example.svg)

<a id="ref-for-propdef-justify-items⑥"></a>

### <a id="justify-items-property"></a>7.1.  Inline-Axis (or Main-Axis) Default Alignment: the [justify-items](#propdef-justify-items) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-justify-items"></a>justify-items                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-all①"></a><a id="ref-for-typedef-self-position⑥"></a><a id="ref-for-mult-opt⑦"></a><a id="ref-for-typedef-overflow-position⑨"></a><a id="ref-for-typedef-baseline-position⑦"></a><a id="ref-for-comb-one③②"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) stretch <a id="ref-for-comb-one③③"></a>\| [\<baseline-position\>](#typedef-baseline-position) <a id="ref-for-comb-one③④"></a>\| [\<overflow-position\>](#typedef-overflow-position)[?](https://www.w3.org/TR/css-values-4/#mult-opt) \[ [\<self-position\>](#typedef-self-position) <a id="ref-for-comb-one③⑤"></a>\| left <a id="ref-for-comb-one③⑥"></a>\| right \] <a id="ref-for-comb-one③⑦"></a>\| legacy <a id="ref-for-comb-one③⑧"></a>\| legacy [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) \[ left <a id="ref-for-comb-one③⑨"></a>\| right <a id="ref-for-comb-one④⓪"></a>\| center \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | legacy                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-justify-items-legacy③"></a>specified keyword(s), except for [legacy](#valdef-justify-items-legacy) (see prose)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |

<a id="ref-for-propdef-justify-self②⑥"></a>

This property specifies the default [justify-self](#propdef-justify-self) for all of the child boxes (including anonymous boxes) participating in this box’s formatting context. Values have the following meanings:

<a id="valdef-justify-items-legacy"></a>legacy  
This keyword causes the value to effectively inherit into descendants.

<a id="ref-for-valdef-justify-items-legacy④"></a>

<a id="ref-for-valdef-justify-content-left⑥"></a>

<a id="ref-for-valdef-justify-content-right⑥"></a>

<a id="ref-for-valdef-self-position-center④"></a>

<a id="ref-for-inherited-value"></a>

If the [legacy](#valdef-justify-items-legacy) keyword appears on its own (without an accompanying [left](#valdef-justify-content-left), [right](#valdef-justify-content-right), or [center](#valdef-self-position-center) keyword): if the [inherited value](https://www.w3.org/TR/css-cascade-5/#inherited-value) of justify-items includes the <a id="ref-for-valdef-justify-items-legacy⑤"></a>legacy keyword, this value computes to the <a id="ref-for-inherited-value①"></a>inherited value; otherwise it computes to normal.

<a id="ref-for-propdef-justify-items⑦"></a>

<a id="ref-for-valdef-justify-items-legacy⑥"></a>

When justify-self:auto references the value of [justify-items](#propdef-justify-items), only the alignment keyword, not the [legacy](#valdef-justify-items-legacy) keyword, is referenced by it. It exists to implement the legacy alignment behavior of HTML’s `<center>` element and `align` attribute.

<a id="ref-for-propdef-justify-self②⑦"></a>

Other values have no special handling and are merely referenced by [justify-self](#propdef-justify-self).

<a id="ref-for-propdef-align-items④"></a>

### <a id="align-items-property"></a>7.2.  Block-Axis (or Cross-Axis) Default Alignment: the [align-items](#propdef-align-items) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                        |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-align-items"></a>align-items                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-self-position⑦"></a><a id="ref-for-mult-opt⑧"></a><a id="ref-for-typedef-overflow-position①⓪"></a><a id="ref-for-typedef-baseline-position⑧"></a><a id="ref-for-comb-one④①"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) stretch <a id="ref-for-comb-one④②"></a>\| [\<baseline-position\>](#typedef-baseline-position) <a id="ref-for-comb-one④③"></a>\| [\<overflow-position\>](#typedef-overflow-position)[?](https://www.w3.org/TR/css-values-4/#mult-opt) [\<self-position\>](#typedef-self-position) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword(s)                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                          |

<a id="ref-for-propdef-align-self②⑥"></a>

This property specifies the default [align-self](#propdef-align-self) for all of the child boxes (including anonymous boxes) participating in this box’s formatting context.

<a id="ref-for-propdef-align-self②⑦"></a>

Values have no special handling and are merely referenced by [align-self](#propdef-align-self).

<a id="ref-for-propdef-place-items①"></a>

### <a id="place-items-property"></a>7.3.  Default Alignment Shorthand: the [place-items](#propdef-place-items) property

| Field               | Definition                                                                                                                                                                                        |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-place-items"></a>place-items                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt⑨"></a><a id="ref-for-propdef-justify-items⑧"></a><a id="ref-for-propdef-align-items⑤"></a>[\<'align-items'\>](#propdef-align-items) [\<'justify-items'\>](#propdef-justify-items)[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                          |

<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-propdef-align-items⑥"></a>

<a id="ref-for-propdef-justify-items⑨"></a>

This [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) sets both the [align-items](#propdef-align-items) and [justify-items](#propdef-justify-items) properties in a single declaration. The first value is assigned to <a id="ref-for-propdef-align-items⑦"></a>align-items. The second value is assigned to <a id="ref-for-propdef-justify-items①⓪"></a>justify-items; if omitted, it is copied from the first value.

## <a id="gaps"></a>8.  Gaps Between Boxes

<a id="ref-for-propdef-margin"></a>

<a id="ref-for-propdef-padding"></a>

While [margin](https://www.w3.org/TR/css-box-4/#propdef-margin) and [padding](https://www.w3.org/TR/css-box-4/#propdef-padding) can be used to specify visual spacing around individual boxes, it’s sometimes more convenient to globally specify spacing <em>between</em> adjacent boxes within a given layout context, particularly when the spacing is different between sibling boxes as opposed to between the first/last box and the container’s edge.

<a id="ref-for-propdef-gap"></a>

<a id="ref-for-propdef-row-gap"></a>

<a id="ref-for-propdef-column-gap"></a>

The [gap](#propdef-gap) property, and its [row-gap](#propdef-row-gap) and [column-gap](#propdef-column-gap) sub-properties, provide this functionality for [multi-column](https://www.w3.org/TR/css3-multicol/), [flex](https://www.w3.org/TR/css-flexbox/), and [grid layout](https://www.w3.org/TR/css-grid/).

<a id="ref-for-propdef-row-gap①"></a>

<a id="ref-for-propdef-column-gap①"></a>

### <a id="column-row-gap"></a>8.1.  Row and Column Gutters: the [row-gap](#propdef-row-gap) and [column-gap](#propdef-column-gap) properties

| Field               | Definition                                                                                                                                                                                                                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-row-gap"></a>row-gap, <a id="propdef-column-gap"></a>column-gap                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage"></a><a id="ref-for-comb-one④④"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-grid-container⑥"></a><a id="ref-for-flex-container①②"></a><a id="ref-for-multi-column-container①"></a>[multi-column containers](https://www.w3.org/TR/css-multicol-1/#multi-column-container), [flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container), [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see [§ 8.3 Percentages In gap Properties](#gap-percent)                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①"></a>specified keyword, else a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                                                                        |

<a id="ref-for-valdef-align-content-space-between③"></a>

<a id="ref-for-content-distribution-properties⑦"></a>

<a id="ref-for-propdef-column-gap②"></a>

<a id="ref-for-inline-axis①④"></a>

<a id="ref-for-propdef-row-gap②"></a>

<a id="ref-for-block-axis①④"></a>

These properties specify fixed-length <a id="gutter"></a>gutters between items in the container, adding space between them—​in a manner similar to the [space-between](#valdef-align-content-space-between) keyword of the [content-distribution properties](#content-distribution-properties), but of a fixed size instead of as a fraction of remaining space. The [column-gap](#propdef-column-gap) property specifies spacing between “columns”, separating boxes in the container’s [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) similar to <a id="ref-for-inline-axis①⑤"></a>inline-axis margin; while [row-gap](#propdef-row-gap) indicates spacing between “rows”, separating boxes in the container’s [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis).

Values have the following meanings:

<a id="ref-for-typedef-length-percentage②"></a>

[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)

Specifies a gap between “rows” or “columns”, as defined by the layout modes to which it applies; see subsections below for details.

Negative values are invalid. For percentages, see [§ 8.3 Percentages In gap Properties](#gap-percent).

<a id="valdef-row-gap-normal"></a>normal

<a id="ref-for-valdef-row-gap-normal"></a>

<a id="ref-for-multi-column-container②"></a>

The value [normal](#valdef-row-gap-normal) represents a used value of 1em on [multi-column containers](https://www.w3.org/TR/css-multicol-1/#multi-column-container), and a used value of 0px in all other contexts.

<a id="ref-for-propdef-justify-content②④"></a>

<a id="ref-for-propdef-align-content②③"></a>

<a id="ref-for-gutter①"></a>

Gutters effect a minimum spacing between items: additional spacing may be added by [justify-content](#propdef-justify-content)/[align-content](#propdef-align-content). Such additional space effectively increases the size of these [gutters](#gutter).

The exact handling of these properties varies by layout container:

<a id="ref-for-multi-column-container③"></a>

<a id="gap-multicol"></a>[multi-column containers](https://www.w3.org/TR/css-multicol-1/#multi-column-container)

<a id="ref-for-propdef-column-height"></a>

<a id="ref-for-propdef-row-gap③"></a>

<a id="ref-for-column-box"></a>

<a id="ref-for-gutter②"></a>

<a id="ref-for-propdef-column-gap③"></a>

[column-gap](#propdef-column-gap) specifies the [gutter](#gutter) between adjacent [column boxes](https://www.w3.org/TR/css-multicol-1/#column-box), see [\[CSS-MULTICOL-1\]](#biblio-css-multicol-1). [row-gap](#propdef-row-gap) specifies the <a id="ref-for-gutter③"></a>gutter between the rows of <a id="ref-for-column-box①"></a>column boxes established by [column-height](https://drafts.csswg.org/css-multicol-2/#propdef-column-height), see [\[CSS-MULTICOL-2\]](#biblio-css-multicol-2).

<a id="ref-for-flex-container①③"></a>

<a id="gap-flex"></a>[flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container)

<a id="ref-for-flex-item①②"></a>

<a id="ref-for-gutter④"></a>

<a id="ref-for-flex-container①④"></a>

<a id="ref-for-propdef-column-gap④"></a>

<a id="ref-for-main-axis⑦"></a>

When applied to the [main axis](https://www.w3.org/TR/css-flexbox-1/#main-axis) (e.g. [column-gap](#propdef-column-gap) in a row [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container)), indicates the [gutter](#gutter) between items (as if an additional fixed-size margin were inserted between adjacent [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item) in a single line).

<a id="ref-for-cross-axis④"></a>

<a id="ref-for-propdef-row-gap④"></a>

<a id="ref-for-flex-container①⑤"></a>

<a id="ref-for-gutter⑤"></a>

<a id="ref-for-flex-line③"></a>

When applied to the [cross axis](https://www.w3.org/TR/css-flexbox-1/#cross-axis) (e.g. [row-gap](#propdef-row-gap) in a row [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container)), indicates the [gutter](#gutter) between adjacent [flex lines](https://www.w3.org/TR/css-flexbox-1/#flex-line).

<a id="ref-for-grid-container⑦"></a>

<a id="gap-grid"></a>[grid containers](https://www.w3.org/TR/css-grid-2/#grid-container)

<a id="ref-for-grid-column①"></a>

<a id="ref-for-grid-row①"></a>

<a id="ref-for-gutter⑥"></a>

<a id="ref-for-grid-container⑧"></a>

<a id="ref-for-propdef-column-gap⑤"></a>

<a id="ref-for-propdef-row-gap⑤"></a>

The [row-gap](#propdef-row-gap) and [column-gap](#propdef-column-gap) properties, when specified on a [grid container](https://www.w3.org/TR/css-grid-2/#grid-container), define the [gutters](#gutter) between [grid rows](https://www.w3.org/TR/css-grid-2/#grid-row) and [grid columns](https://www.w3.org/TR/css-grid-2/#grid-column), respectively. See [CSS Grid Layout 1 § 10.1 Gutters: the row-gap, column-gap, and gap properties](https://www.w3.org/TR/css-grid-1/#gutters) for precise details.

<a id="ref-for-gutter⑦"></a>

<a id="ref-for-fragmentation-break"></a>

In all cases, the [gutter](#gutter) disappears when it coincides with a [fragmentation break](https://www.w3.org/TR/css-break-4/#fragmentation-break). [\[CSS-BREAK-3\]](#biblio-css-break-3)

<a id="ref-for-propdef-gap①"></a>

<a id="ref-for-propdef-border-spacing"></a>

<a id="ref-for-valdef-align-content-space-evenly①"></a>

<a id="ref-for-valdef-align-content-space-between④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Table boxes do not use the [gap](#propdef-gap) properties to specify separation between their cells. Instead, they use the [border-spacing](https://www.w3.org/TR/CSS2/tables.html#propdef-border-spacing) property, which has slightly different functionality: it inherits, and it also specifies the additional spacing between the outermost cells and the border of the table (similar to [space-evenly](#valdef-align-content-space-evenly) rather than [space-between](#valdef-align-content-space-between)).

<a id="ref-for-propdef-gap②"></a>

### <a id="gap-shorthand"></a>8.2.  Gap Shorthand: the [gap](#propdef-gap) property

| Field               | Definition                                                                                                                                                                                                                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-gap"></a>gap                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt①⓪"></a><a id="ref-for-propdef-column-gap⑥"></a><a id="ref-for-propdef-row-gap⑥"></a>[\<'row-gap'\>](#propdef-row-gap) [\<'column-gap'\>](#propdef-column-gap)[?](https://www.w3.org/TR/css-values-4/#mult-opt)                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-grid-container⑨"></a><a id="ref-for-flex-container①⑥"></a><a id="ref-for-multi-column-container④"></a>[multi-column containers](https://www.w3.org/TR/css-multicol-1/#multi-column-container), [flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container), [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | refer to corresponding dimension of the content area                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                                                                        |

<a id="ref-for-shorthand-property③"></a>

<a id="ref-for-propdef-row-gap⑦"></a>

<a id="ref-for-propdef-column-gap⑦"></a>

This property is a [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) that sets [row-gap](#propdef-row-gap) and [column-gap](#propdef-column-gap) in one declaration. If <a id="ref-for-propdef-column-gap⑧"></a>\<'column-gap'\> is omitted, it’s set to the same value as <a id="ref-for-propdef-row-gap⑧"></a>\<'row-gap'\>.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-06510943"></a>
>
> ![A diagram showing how margins and padding add to the visible gutter size](https://www.w3.org/TR/2026/WD-css-align-3-20260130/images/gutters-gaps.svg)
>
> <a id="ref-for-propdef-gap③"></a>
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note: The [gap](#propdef-gap) property is only one component of the visible “gutter” or “alley” created between boxes. Margins, padding, or the use of distributed alignment may increase the visible separation between boxes beyond what is specified in <a id="ref-for-propdef-gap④"></a>gap.

<a id="ref-for-propdef-gap⑤"></a>

### <a id="gap-percent"></a>8.3.  Percentages In [gap](#propdef-gap) Properties

<a id="ref-for-propdef-gap⑥"></a>

In general, gaps introduced by the [gap](#propdef-gap) properties are intended to act like an empty item/track/etc with the gap’s size; in other words, an author should be able to reproduce the effects of <a id="ref-for-propdef-gap⑦"></a>gap by just inserting additional empty items/tracks/etc into the container.

<a id="ref-for-propdef-gap⑧"></a>

<a id="ref-for-content-box①"></a>

<a id="ref-for-cyclic-percentage-size"></a>

[gap](#propdef-gap) always resolves percentages against the corresponding size of the [content box](https://www.w3.org/TR/css-box-4/#content-box) of the element. When this size is definite, the behavior is well-defined and consistent across layout modes. But since different layout modes treat [cyclic percentage sizes](https://www.w3.org/TR/css-sizing-3/#cyclic-percentage-size) for items/tracks/etc differently, <a id="ref-for-propdef-gap⑨"></a>gap does as well:

In Grid Layout  
<a id="ref-for-cyclic-percentage-size①"></a>

[As in the min size properties and margins/paddings](https://www.w3.org/TR/css-sizing-3/#percentage-sizing) [\[CSS-SIZING-3\]](#biblio-css-sizing-3), [cyclic percentage sizes](https://www.w3.org/TR/css-sizing-3/#cyclic-percentage-size) resolve against zero for determining intrinsic size contributions, but resolve against the box’s content box when laying out the box’s contents.

In Flex Layout  
<a id="ref-for-cyclic-percentage-size②"></a>

[Cyclic percentage sizes](https://www.w3.org/TR/css-sizing-3/#cyclic-percentage-size) resolve against zero in all cases.

<a id="ref-for-propdef-grid-row-gap"></a>

<a id="ref-for-propdef-grid-column-gap"></a>

<a id="ref-for-propdef-grid-gap"></a>

### <a id="gap-legacy"></a>8.4.  Legacy Gap Properties: the [grid-row-gap](#propdef-grid-row-gap), [grid-column-gap](#propdef-grid-column-gap), and [grid-gap](#propdef-grid-gap) properties

<a id="ref-for-gutter⑧"></a>

<a id="ref-for-propdef-row-gap⑨"></a>

<a id="ref-for-propdef-column-gap⑨"></a>

The Grid Layout module was originally written with its own set of [gutter](#gutter) properties, before all such properties were unified into the existing [row-gap](#propdef-row-gap)/[column-gap](#propdef-column-gap) naming. For compatibility with legacy content, these grid-prefixed names must be supported as follows:

- <a id="ref-for-legacy-name-alias"></a>

  <a id="ref-for-propdef-row-gap①⓪"></a>

  <a id="propdef-grid-row-gap"></a>grid-row-gap as a [legacy name alias](https://www.w3.org/TR/css-cascade-5/#legacy-name-alias) of the [row-gap](#propdef-row-gap) property

- <a id="ref-for-legacy-name-alias①"></a>

  <a id="ref-for-propdef-column-gap①⓪"></a>

  <a id="propdef-grid-column-gap"></a>grid-column-gap as a [legacy name alias](https://www.w3.org/TR/css-cascade-5/#legacy-name-alias) of the [column-gap](#propdef-column-gap) property

- <a id="ref-for-legacy-name-alias②"></a>

  <a id="ref-for-propdef-gap①⓪"></a>

  <a id="propdef-grid-gap"></a>grid-gap as a [legacy name alias](https://www.w3.org/TR/css-cascade-5/#legacy-name-alias) of the [gap](#propdef-gap) property

## <a id="baseline-rules"></a>9.  Baseline Alignment Details

<a id="ref-for-baseline-sharing-group⑦"></a>

<a id="ref-for-alignment-baseline⑤"></a>

<a id="ref-for-propdef-align-content②④"></a>

<a id="ref-for-first-formatted-line0"></a>

<a id="ref-for-writing-mode②②"></a>

Boxes in a [baseline-sharing group](#baseline-sharing-group) are aligned to each other using their [alignment baselines](#alignment-baseline). For example, in horizontal writing modes, specifying [align-content: baseline](#propdef-align-content) on table cells in the same row will align the baselines of their [first formatted lines](https://www.w3.org/TR/selectors-3/#first-formatted-line0). This section defines exactly how baseline alignment is performed in consideration of the myriad baselines and [writing modes](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) that exist in internationalized modern CSS.

A <a id="baseline-set"></a>baseline set is a set of baselines (alphabetic, central, etc.) associated with a common baseline table. Typically, a typesetting tradition will use only one of these, but different writing systems use different baselines, and mixing writing systems can result in using more than one within a single line. Refer to [CSS Writing Modes 3 § 4.1 Introduction to Baselines](https://www.w3.org/TR/css-writing-modes-3/#intro-baselines) for more information on baselines and writing modes.

### <a id="baseline-export"></a>9.1.  Determining the Baselines of a Box

<a id="ref-for-baseline-set"></a>

<a id="ref-for-shared-alignment-context⑤"></a>

<a id="ref-for-propdef-dominant-baseline"></a>

<a id="ref-for-propdef-alignment-baseline"></a>

Each box, for a given axis, has potentially a <a id="first-baseline-set"></a>first baseline set (and <a id="last-baseline-set"></a>last baseline set) that nominally corresponds to the [baseline set](#baseline-set) of the first/last line of text within the box. The <a id="alignment-baseline"></a>alignment baseline, which is the baseline used to align the box in its [alignment context](#shared-alignment-context), is one of the baselines in its <a id="ref-for-baseline-set①"></a>baseline set, usually the dominant baseline associated with the <a id="ref-for-shared-alignment-context⑥"></a>shared alignment context. (See the [dominant-baseline](https://www.w3.org/TR/css-inline-3/#propdef-dominant-baseline) and [alignment-baseline](https://www.w3.org/TR/css-inline-3/#propdef-alignment-baseline) properties in [\[CSS-INLINE-3\]](#biblio-css-inline-3).)

<a id="ref-for-baseline-set②"></a>

The first and last [baseline sets](#baseline-set) of a box are determined differently based on the layout model, as follows:

line box  
<a id="ref-for-root-inline-box"></a>

<a id="ref-for-generate-baselines"></a>

<a id="ref-for-baseline-set③"></a>

The first/last [baseline set](#baseline-set) of a line box is [generated](#generate-baselines) from the dominant baseline and the font settings of its [root inline box](https://www.w3.org/TR/css-inline-3/#root-inline-box).

block containers  
<a id="ref-for-in-flow③"></a>

<a id="ref-for-block-container⑥"></a>

<a id="ref-for-baseline-set④"></a>

The first/last [baseline set](#baseline-set) of a [block container](https://www.w3.org/TR/css-display-4/#block-container) is taken from the first/last [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) line box in the block container or the first/last <a id="ref-for-in-flow④"></a>in-flow block-level child in the block container that contributes a set of first/last baselines, whichever comes first/last. If there is no such line box or child, then the block container has no <a id="ref-for-baseline-set⑤"></a>baseline set.

<a id="ref-for-propdef-baseline-source"></a>

<a id="ref-for-valdef-baseline-source-auto"></a>

<a id="ref-for-initial-value"></a>

<a id="ref-for-block-level"></a>

<a id="ref-for-inline-level"></a>

<a id="ref-for-block-container⑦"></a>

<a id="ref-for-scroll-container⑨"></a>

<a id="ref-for-last-baseline-set②"></a>

<a id="ref-for-block-end"></a>

<a id="ref-for-margin-edge③"></a>

However, for legacy reasons if its [baseline-source](https://www.w3.org/TR/css-inline-3/#propdef-baseline-source) is [auto](https://www.w3.org/TR/css-inline-3/#valdef-baseline-source-auto) (the [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value)) a [block-level](https://www.w3.org/TR/css-display-4/#block-level) or [inline-level](https://www.w3.org/TR/css-display-4/#inline-level) [block container](https://www.w3.org/TR/css-display-4/#block-container) that is a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) always has a [last baseline set](#last-baseline-set), whose baselines all correspond to its [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) [margin edge](https://www.w3.org/TR/css-box-4/#margin-edge).

multi-column containers  
<a id="ref-for-alignment-baseline⑥"></a>

<a id="ref-for-block-start"></a>

<a id="ref-for-multi-column-spanner"></a>

<a id="ref-for-sec-terms-and-definitions-colun"></a>

<a id="ref-for-multi-column-container⑤"></a>

<a id="ref-for-baseline-set⑥"></a>

The first [baseline set](#baseline-set) of a [multi-column container](https://www.w3.org/TR/css-multicol-1/#multi-column-container) is the first <a id="ref-for-baseline-set⑦"></a>baseline set of the [column](https://tc39.es/ecma426/#sec-terms-and-definitions-colun) or [multi-column spanner](https://www.w3.org/TR/css-multicol-2/#multi-column-spanner) with the highest ([block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start)–most) baseline corresponding to the <a id="ref-for-multi-column-container⑥"></a>multi-column container’s [alignment baseline](#alignment-baseline). If there is no such line box or child, then the multi-column container has no first <a id="ref-for-baseline-set⑧"></a>baseline set.

<a id="ref-for-baseline-set⑨"></a>

<a id="ref-for-block-end①"></a>

The last [baseline set](#baseline-set) is analogous, but uses the <em>last</em> <a id="ref-for-baseline-set①⓪"></a>baseline set and <em>lowest</em> ([block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end)–most) baseline.

tables  
<a id="ref-for-baseline-set①①"></a>

The first/last [baseline set](#baseline-set) of a table box is the first/last <a id="ref-for-baseline-set①②"></a>baseline set of its first/last row.

When finding the first/last baseline set of an inline-block, any baselines contributed by table boxes must be skipped. (This quirk is a legacy behavior from [\[CSS2\]](#biblio-css2).)

table rows  
<a id="ref-for-synthesize-baseline"></a>

<a id="ref-for-first-available-font"></a>

<a id="ref-for-alignment-baseline⑦"></a>

<a id="ref-for-generate-baselines①"></a>

<a id="ref-for-baseline-set①③"></a>

<a id="ref-for-inline-axis①⑥"></a>

<a id="ref-for-valdef-justify-self-last-baseline⑧"></a>

<a id="ref-for-valdef-justify-self-first-baseline①①"></a>

If any cells in the row participate in [first baseline](#valdef-justify-self-first-baseline)/[last baseline](#valdef-justify-self-last-baseline) alignment along the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis), the first/last [baseline set](#baseline-set) of the row is [generated](#generate-baselines) from their shared [alignment baseline](#alignment-baseline) and the row’s [first available font](https://www.w3.org/TR/css-fonts-4/#first-available-font), after alignment has been performed. Otherwise, the first/last <a id="ref-for-baseline-set①④"></a>baseline set of the row is [synthesized](#synthesize-baseline) from the lowest and highest content edges of the cells in the row. [\[CSS2\]](#biblio-css2)

<a id="ref-for-valdef-justify-self-first-baseline①②"></a>

<a id="ref-for-valdef-justify-self-last-baseline⑨"></a>

Spanning cells participate only in the first/last row that they span for the purpose of [first baseline](#valdef-justify-self-first-baseline)/[last baseline](#valdef-justify-self-last-baseline).

flex containers  
See [Flex Baselines](https://www.w3.org/TR/css3-flexbox/#flex-baselines) in [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1).

grid containers  
See [Grid Baselines](https://www.w3.org/TR/css3-grid-layout/#grid-baselines) in [\[CSS-GRID-1\]](#biblio-css-grid-1).

<a id="ref-for-first-available-font①"></a>

To <a id="generate-baselines"></a>generate baselines for a box from a single baseline, use the baseline table from the font settings and [first available font](https://www.w3.org/TR/css-fonts-4/#first-available-font) of that box, and align that baseline set to the given single baseline.

<a id="ref-for-baseline-alignment②"></a>

<a id="ref-for-baseline-set①⑤"></a>

<a id="ref-for-alignment-baseline⑧"></a>

<a id="ref-for-synthesize-baseline①"></a>

<a id="ref-for-formatting-context"></a>

<a id="ref-for-line-under"></a>

If a box that participates in [baseline alignment](#baseline-alignment) has no [baseline set](#baseline-set), then its [alignment baseline](#alignment-baseline) is [synthesized](#synthesize-baseline) according to the rules of the [formatting context](https://www.w3.org/TR/css-display-4/#formatting-context) in which it participates. To <a id="synthesize-baseline"></a>synthesize baselines from a rectangle (or two parallel lines), synthesize the alphabetic baseline from the [line-under](https://www.w3.org/TR/css-writing-modes-4/#line-under) line, and the central baseline by averaging the positions of the two edges or lines. See [CSS Inline Layout 3 § A Synthesizing Alignment Metrics](https://www.w3.org/TR/css-inline-3/#baseline-synthesis) for rules on synthesizing additional baselines.

<a id="ref-for-synthesize-baseline②"></a>

<a id="ref-for-formatting-context①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The edges used to [synthesize](#synthesize-baseline) baselines from a box depend on their [formatting context](https://www.w3.org/TR/css-display-4/#formatting-context): inline-level boxes <a id="ref-for-synthesize-baseline③"></a>synthesize from their margin edges [\[CSS-INLINE-3\]](#biblio-css-inline-3), table cells <a id="ref-for-synthesize-baseline④"></a>synthesize from their content edges [\[CSS2\]](#biblio-css2), and grid and flex items <a id="ref-for-synthesize-baseline⑤"></a>synthesize from their border edges [\[CSS-GRID-1\]](#biblio-css-grid-1) [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1).

<a id="ref-for-writing-mode②③"></a>

<a id="ref-for-line-under①"></a>

<a id="ref-for-line-over"></a>

<a id="ref-for-block-flow-direction"></a>

<a id="ref-for-shared-alignment-context⑦"></a>

In general, the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the box, shape, or other object being aligned is used to determine the [line-under](https://www.w3.org/TR/css-writing-modes-4/#line-under) and [line-over](https://www.w3.org/TR/css-writing-modes-4/#line-over) edges for synthesis. However, when that <a id="ref-for-writing-mode②④"></a>writing mode’s [block flow direction](https://www.w3.org/TR/css-writing-modes-4/#block-flow-direction) is parallel to the axis of the [alignment context](#shared-alignment-context), an axis-compatible <a id="ref-for-writing-mode②⑤"></a>writing mode must be assumed:

- <a id="ref-for-shared-alignment-context⑧"></a>

  <a id="ref-for-block-flow-direction①"></a>

  <a id="ref-for-writing-mode②⑥"></a>

  If the box establishing the [alignment context](#shared-alignment-context) has a [block flow direction](https://www.w3.org/TR/css-writing-modes-4/#block-flow-direction) that is orthogonal to the axis of the <a id="ref-for-shared-alignment-context⑨"></a>alignment context, use its [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode).

- Otherwise:

  - <a id="ref-for-writing-mode②⑦"></a>

    <a id="ref-for-valdef-writing-mode-horizontal-tb"></a>

    If the box’s own [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) is vertical, assume [horizontal-tb](https://www.w3.org/TR/css-writing-modes-4/#valdef-writing-mode-horizontal-tb).

  - <a id="ref-for-writing-mode②⑧"></a>

    <a id="ref-for-valdef-writing-mode-vertical-lr"></a>

    <a id="ref-for-propdef-direction⑦"></a>

    <a id="ref-for-valdef-direction-ltr②"></a>

    <a id="ref-for-valdef-writing-mode-vertical-rl"></a>

    <a id="ref-for-valdef-direction-rtl②"></a>

    If the box’s own [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) is horizontal, assume [vertical-lr](https://www.w3.org/TR/css-writing-modes-4/#valdef-writing-mode-vertical-lr) if [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) is [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr) and [vertical-rl](https://www.w3.org/TR/css-writing-modes-4/#valdef-writing-mode-vertical-rl) if <a id="ref-for-propdef-direction⑧"></a>direction is [rtl](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-rtl).

<a id="ref-for-in-flow⑤"></a>

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-scroll-container①⓪"></a>

For the purposes of finding the baselines of a box, it and all its [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) descendants with a scrolling mechanism (see the [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) property) must be considered as if scrolled to their initial scroll position. Additionally, if the position of a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container)’s first/last baseline is outside its border edge, that baseline’s position is clamped to the border edge.

### <a id="baseline-terms"></a>9.2.  Baseline Alignment Grouping

A <a id="baseline-sharing-group"></a>baseline-sharing group is composed of boxes that participate in baseline alignment together. This is possible only if they both:

- <a id="ref-for-propdef-align-self②⑧"></a>

  <a id="ref-for-shared-alignment-context①⓪"></a>

  Share an [alignment context](#shared-alignment-context) along an axis perpendicular to the axis they’re being baseline-aligned in. (For example, grid items with [align-self: baseline](#propdef-align-self) are baseline-aligning along the grid’s block axis, and therefore participate with other items in their row.)

- <a id="ref-for-compatible-baseline-alignment-preferences"></a>

  Have [compatible baseline alignment preferences](#compatible-baseline-alignment-preferences) (i.e., the baselines that want to align are on the same side of the alignment context).

<a id="ref-for-baseline-content-alignment①①"></a>

<a id="ref-for-baseline-self-alignment①⓪"></a>

<a id="ref-for-baseline-sharing-group⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Boxes participating in [baseline content-alignment](#baseline-content-alignment) and boxes participating in [baseline self-alignment](#baseline-self-alignment) can be part of the same [baseline-sharing group](#baseline-sharing-group), and can thus align to each other. The difference between the two methods is in where extra space is inserted to perform the alignment (inside or outside the box’s own border).

Boxes share an <a id="shared-alignment-context"></a>alignment context, along a particular axis, and established by a particular box, when they are:

- table cells in the same row, along the table’s row (inline) axis, established by the row box

- <a id="ref-for-grid-container①⓪"></a>

  grid items in the same row, along the grid’s row (inline) axis, established by the [grid container](https://www.w3.org/TR/css-grid-2/#grid-container)

- <a id="ref-for-grid-container①①"></a>

  grid items in the same column, along the grid’s column (block) axis, established by the [grid container](https://www.w3.org/TR/css-grid-2/#grid-container)

- <a id="ref-for-flex-container①⑦"></a>

  flex items in the same flex line, along the flex container’s main axis, established by the [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container)

<a id="ref-for-propdef-vertical-align③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Conceptually, the inline-level boxes in a line box also share a self-alignment context and participate in a baseline-sharing group; however they only baseline-align in response to the [vertical-align](https://www.w3.org/TR/CSS2/visudet.html#propdef-vertical-align) property, not any of the properties defined in this module. See [\[CSS-INLINE-3\]](#biblio-css-inline-3).

<a id="ref-for-shared-alignment-context①①"></a>

<a id="ref-for-first-baseline-alignment"></a>

<a id="ref-for-last-baseline-alignment"></a>

If a box spans multiple [shared alignment contexts](#shared-alignment-context), then it participates in first/last baseline alignment within its start-most/end-most <a id="ref-for-shared-alignment-context①②"></a>shared alignment context along that axis. For example, a table cell spanning three rows participates in [first-baseline alignment](#first-baseline-alignment) with the table cells in the first row that it spans, or alternatively in [last-baseline alignment](#last-baseline-alignment) with the table cells in the last row that it spans.

<a id="ref-for-baseline-alignment-preference⑤"></a>

<a id="ref-for-baseline-sharing-group⑨"></a>

The [baseline alignment preferences](#baseline-alignment-preference) of two boxes in a [baseline-sharing group](#baseline-sharing-group) are <a id="compatible-baseline-alignment-preferences"></a>compatible if they have:

- <a id="ref-for-block-flow-direction②"></a>

  <a id="ref-for-baseline-alignment-preference⑥"></a>

  the same [block flow direction](https://www.w3.org/TR/css-writing-modes-4/#block-flow-direction) and same [baseline alignment preference](#baseline-alignment-preference)

- <a id="ref-for-block-flow-direction③"></a>

  <a id="ref-for-baseline-alignment-preference⑦"></a>

  opposite [block flow direction](https://www.w3.org/TR/css-writing-modes-4/#block-flow-direction) and opposite [baseline alignment preference](#baseline-alignment-preference)

### <a id="align-by-baseline"></a>9.3.  Aligning Boxes by Baseline

<a id="ref-for-alignment-subject⑦⑥"></a>

<a id="ref-for-baseline-sharing-group①⓪"></a>

Given a set of [alignment subjects](#alignment-subject) and their baselines that all belong to a single [baseline-sharing group](#baseline-sharing-group), the <a id="ref-for-alignment-subject⑦⑦"></a>alignment subjects are baseline-aligned as follows:

1.  <a id="ref-for-baseline-sharing-group①①"></a>

    <a id="ref-for-first-available-font②"></a>

    <a id="ref-for-shared-alignment-context①③"></a>

    <a id="ref-for-alignment-subject⑦⑧"></a>

    Generate the [baseline-sharing group](#baseline-sharing-group)’s baseline table from the [first available font](https://www.w3.org/TR/css-fonts-4/#first-available-font) of the group’s [alignment context](#shared-alignment-context) and overlay also the mirror of this baseline table by aligning their central baselines. These are the baseline “grids” to which the [alignment subjects](#alignment-subject) will align.

2.  <a id="ref-for-alignment-subject⑦⑨"></a>

    <a id="ref-for-alignment-baseline⑨"></a>

    <a id="ref-for-line-orientation"></a>

    <a id="ref-for-propdef-alignment-baseline①"></a>

    <a id="ref-for-dominant-baseline"></a>

    <a id="ref-for-shared-alignment-context①④"></a>

    Align each [alignment subject](#alignment-subject) by its specified [alignment baseline](#alignment-baseline) to the group’s baseline table or its mirror, whichever matches the <a id="ref-for-alignment-subject⑧⓪"></a>alignment subject’s [line orientation](https://www.w3.org/TR/css-writing-modes-4/#line-orientation). Unless otherwise specified (e.g. via the [alignment-baseline](https://www.w3.org/TR/css-inline-3/#propdef-alignment-baseline) property), the <a id="ref-for-alignment-baseline①⓪"></a>alignment baseline is the [dominant baseline](https://www.w3.org/TR/css-writing-modes-3/#dominant-baseline) of the [alignment context](#shared-alignment-context).

3.  <a id="ref-for-baseline-sharing-group①②"></a>

    <a id="ref-for-alignment-container④⑨"></a>

    <a id="ref-for-fallback-alignment①⑦"></a>

    <a id="ref-for-physical-direction"></a>

    Position the aligned [baseline-sharing group](#baseline-sharing-group) within the [alignment container](#alignment-container) according to its [fallback alignment](#fallback-alignment). The <a id="ref-for-fallback-alignment①⑧"></a>fallback alignment of a <a id="ref-for-baseline-sharing-group①③"></a>baseline-sharing group is the <a id="ref-for-fallback-alignment①⑨"></a>fallback alignment of its items as resolved to [physical directions](https://www.w3.org/TR/css-writing-modes-4/#physical-direction).

4.  <a id="ref-for-baseline-content-alignment①②"></a>

    <a id="ref-for-alignment-container⑤⓪"></a>

    <a id="ref-for-alignment-subject⑧①"></a>

    <a id="ref-for-shared-alignment-context①⑤"></a>

    <a id="ref-for-baseline-sharing-group①④"></a>

    For first/last [baseline <em>content</em>-alignment](#baseline-content-alignment), then add the minimum necessary extra space between the [alignment container’s](#alignment-container) start/end content edge and the [alignment subject’s](#alignment-subject) edge to align the start/end margin edges of all the <a id="ref-for-alignment-container⑤①"></a>alignment containers in the [alignment context](#shared-alignment-context) while maintaining baseline alignment within the [baseline-sharing group](#baseline-sharing-group).

## <a id="staticpos-rect"></a> Appendix A: Static Position Terminology

<a id="ref-for-inset-properties④"></a>

<a id="ref-for-valdef-top-auto①⓪"></a>

<a id="ref-for-static-position⑧"></a>

<a id="ref-for-box-alignment-properties②"></a>

<a id="ref-for-containing-block①⑨"></a>

When both [inset properties](https://www.w3.org/TR/css-logical-1/#inset-properties) in a given axis are [auto](https://www.w3.org/TR/css-position-3/#valdef-top-auto) on an [absolutely positioned box](https://www.w3.org/TR/CSS2/visuren.html#absolutely-positioned), CSS2 uses its [static position](https://www.w3.org/TR/css-position-3/#static-position) to resolve its size and position. See [CSS2.1§10.3.7](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-width). The [box alignment properties](#box-alignment-properties) modify these calculations, just as they do the sizing and positioning calculations in other layout modes. These modifications refer to a <a id="static-position-rectangle"></a>static-position rectangle, whose edges represent the <a id="ref-for-static-position⑨"></a>static position of the box from each side of its [containing block](https://www.w3.org/TR/css-display-4/#containing-block).

<a id="ref-for-static-position-rectangle⑦"></a>

<a id="ref-for-static-position①⓪"></a>

The [static-position rectangle](#static-position-rectangle) and the [static positions](https://www.w3.org/TR/css-position-3/#static-position) to which it corresponds are defined by the layout model of its “hypothetical box”:

Block Layout  
<a id="ref-for-block-start①"></a>

<a id="ref-for-static-position-containing-block⑥"></a>

<a id="ref-for-static-position-rectangle⑧"></a>

<a id="ref-for-block-level-box②"></a>

<a id="ref-for-static-position①①"></a>

The [static positions](https://www.w3.org/TR/css-position-3/#static-position) of a [block-level box](https://www.w3.org/TR/css-display-4/#block-level-box) are defined in [\[CSS2\]](#biblio-css2) Chapter 10. The [static-position rectangle](#static-position-rectangle) is a zero-thickness rectangle spanning between the inline-axis sides of the box’s [static-position containing block](https://www.w3.org/TR/css-position-3/#static-position-containing-block) (see [CSS2§10.3.7](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-width)); and positioned at its [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) <a id="ref-for-static-position①②"></a>static position (see [CSS2§10.6.4](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-height)).

Inline Layout  
<a id="ref-for-inline-start①"></a>

<a id="ref-for-line-box"></a>

<a id="ref-for-line-under②"></a>

<a id="ref-for-line-over①"></a>

<a id="ref-for-static-position-rectangle⑨"></a>

<a id="ref-for-inline-level-box"></a>

<a id="ref-for-static-position①③"></a>

The [static positions](https://www.w3.org/TR/css-position-3/#static-position) of an [inline-level box](https://www.w3.org/TR/css-display-4/#inline-level-box) are defined in [\[CSS2\]](#biblio-css2) Chapter 10. The [static-position rectangle](#static-position-rectangle) is a zero-thickness rectangle spanning between the [line-over](https://www.w3.org/TR/css-writing-modes-4/#line-over)/[line-under](https://www.w3.org/TR/css-writing-modes-4/#line-under) sides of the [line box](https://www.w3.org/TR/CSS2/visuren.html#line-box) that would have contained its “hypothetical box” (see [CSS2§10.3.7](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-width)); and positioned at its [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) <a id="ref-for-static-position①④"></a>static position.

Flex Layout  
<a id="ref-for-content-edge"></a>

<a id="ref-for-flex-container①⑧"></a>

<a id="ref-for-static-position-rectangle①⓪"></a>

The [static-position rectangle](#static-position-rectangle) of the child of a [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container) corresponds to the [content edges](https://www.w3.org/TR/css-box-4/#content-edge) of the <a id="ref-for-flex-container①⑨"></a>flex container. See [CSS Flexbox 1 § 4.1 Absolutely-Positioned Flex Children](https://www.w3.org/TR/css-flexbox-1/#abspos-items).

Grid Layout  
<a id="ref-for-grid-placement-property"></a>

<a id="ref-for-grid-area②"></a>

<a id="ref-for-containing-block②⓪"></a>

<a id="ref-for-content-edge①"></a>

<a id="ref-for-grid-container①②"></a>

<a id="ref-for-static-position-rectangle①①"></a>

By default, the [static-position rectangle](#static-position-rectangle) of the child of a [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) corresponds to the [content edges](https://www.w3.org/TR/css-box-4/#content-edge) of the <a id="ref-for-grid-container①③"></a>grid container. However, if that <a id="ref-for-grid-container①④"></a>grid container also establishes the box’s actual [containing block](https://www.w3.org/TR/css-display-4/#containing-block), then the [grid area](https://www.w3.org/TR/css-grid-2/#grid-area) specified by the [grid-placement properties](https://www.w3.org/TR/css-grid-2/#grid-placement-property) establishes its <a id="ref-for-static-position-rectangle①②"></a>static-position rectangle instead. See the [static position of a grid container child](https://www.w3.org/TR/css-grid-1/#static-position) in [\[CSS-GRID-1\]](#biblio-css-grid-1).

## <a id="changes"></a>10.  Changes

Changes since the [11 March 2025 Working Draft](https://www.w3.org/TR/2025/WD-css-align-3-20250311/) include:

- <a id="ref-for-propdef-position-area"></a>

  <a id="ref-for-used-value②"></a>

  <a id="ref-for-valdef-align-self-normal④"></a>

  Clarify that [§ 4.4.1.2 Self-Alignment for Absolutely Positioned Boxes](#auto-safety-position) applies when [position-area](https://www.w3.org/TR/css-anchor-position-1/#propdef-position-area) alters the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of [normal](#valdef-align-self-normal).

- Allow absolutely positioned boxes to honor alignment even when overflowing into the scrollable overflow area of a scroll container containing block. ([Issue 12106](https://github.com/w3c/csswg-drafts/issues/12106))

- <a id="ref-for-valdef-justify-self-normal①⓪"></a>

  Allow safe/unsafe to be specified with [normal](#valdef-justify-self-normal). ([Issue 12920](https://github.com/w3c/csswg-drafts/issues/12920))

- <a id="ref-for-valdef-self-position-flex-start⑦"></a>

  Change safe alignment to cause the container to fall back to [flex-start](#valdef-self-position-flex-start), so it is correctly safe for a scrollable reversed flexbox. ([Issue 11937](https://github.com/w3c/csswg-drafts/issues/11937))

- <a id="ref-for-propdef-justify-self②⑧"></a>

  Defined anonymous block boxes to always act like [justify-self: normal](#propdef-justify-self). ([Issue 11461](https://github.com/w3c/csswg-drafts/issues/11461))

- <a id="ref-for-propdef-justify-self②⑨"></a>

  <a id="ref-for-automatic-size①"></a>

  Defined that [justify-self](#propdef-justify-self) affects the [automatic size](https://www.w3.org/TR/css-sizing-3/#automatic-size) of block-level boxes the same way it does for flex and grid items. ([Issue 12102](https://github.com/w3c/csswg-drafts/issues/12102))

See also [previous changes](https://www.w3.org/TR/2025/WD-css-align-3-20250311/#changes).

## <a id="privacy"></a>11.  Privacy Considerations

As a simple layout spec, this introduces no new privacy considerations.

## <a id="security"></a>12.  Security Considerations

As a simple layout spec, this introduces no new security considerations.

## <a id="acknowledgments"></a> Acknowledgments

Special thanks goes to David Baron, Javier Fernandez, Markus Mielke, Alex Mogilevsky, and the participants in the CSSWG’s March 2008 F2F alignment discussions for their contributions to the alignment model described herein, and to Melanie Richards for her illustrations of the various [alignment keywords](#alignment-values).

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

- [align-content](#propdef-align-content), in § 5.1
- [align-items](#propdef-align-items), in § 7.2
- [alignment baseline](#alignment-baseline), in § 9.1
- [alignment container](#alignment-container), in § 3
- [alignment context](#shared-alignment-context), in § 9.2
- [alignment subject](#alignment-subject), in § 3
- [align-self](#propdef-align-self), in § 6.2
- auto
  - [value for align-self](#valdef-align-self-auto), in § 6.2
  - [value for justify-self](#valdef-justify-self-auto), in § 6.1
- [baseline](#valdef-justify-self-baseline), in § 4.2
- [Baseline alignment](#baseline-alignment), in § 4.2
- [baseline alignment preference](#baseline-alignment-preference), in § 4.2
- [baseline content-alignment](#baseline-content-alignment), in § 5.4
- [\<baseline-position\>](#typedef-baseline-position), in § 4.2
- [baseline self-alignment](#baseline-self-alignment), in § 6.4
- [baseline set](#baseline-set), in § 9
- [baseline-sharing group](#baseline-sharing-group), in § 9.2
- [box alignment properties](#box-alignment-properties), in § 2
- [center](#valdef-self-position-center), in § 4.1
- [column-gap](#propdef-column-gap), in § 8.1
- [compatible baseline alignment preferences](#compatible-baseline-alignment-preferences), in § 9.2
- [\<content-distribution\>](#typedef-content-distribution), in § 4.3
- [content distribution](#content-distribute), in § 5
- [content-distribution](#content-distribute), in § 5
- [content-distribution properties](#content-distribution-properties), in § 5
- [\<content-position\>](#typedef-content-position), in § 4.1
- [coordinated self-alignment preference](#coordinated-self-alignment-preference), in § 5.4
- [distributed alignment](#distributed-alignment), in § 4.3
- [end](#valdef-self-position-end), in § 4.1
- [fallback alignment](#fallback-alignment), in § 3
- [first](#valdef-justify-self-first-baseline), in § 4.2
- [first baseline](#valdef-justify-self-first-baseline), in § 4.2
- [first-baseline alignment](#first-baseline-alignment), in § 4.2
- [first-baseline content-alignment](#baseline-content-alignment), in § 5.4
- [first baselines](#first-baseline-set), in § 9.1
- [first-baseline self-alignment](#baseline-self-alignment), in § 6.4
- [first baseline set](#first-baseline-set), in § 9.1
- [flex-end](#valdef-self-position-flex-end), in § 4.1
- [flex-start](#valdef-self-position-flex-start), in § 4.1
- [gap](#propdef-gap), in § 8.2
- [generate](#generate-baselines), in § 9.1
- [generate baselines](#generate-baselines), in § 9.1
- [generated](#generate-baselines), in § 9.1
- [grid-column-gap](#propdef-grid-column-gap), in § 8.4
- [grid-gap](#propdef-grid-gap), in § 8.4
- [grid-row-gap](#propdef-grid-row-gap), in § 8.4
- [gutter](#gutter), in § 8.1
- [justify-content](#propdef-justify-content), in § 5.1
- [justify-items](#propdef-justify-items), in § 7.1
- [justify-self](#propdef-justify-self), in § 6.1
- [last](#valdef-justify-self-last-baseline), in § 4.2
- [last baseline](#valdef-justify-self-last-baseline), in § 4.2
- [last-baseline alignment](#last-baseline-alignment), in § 4.2
- [last-baseline content-alignment](#baseline-content-alignment), in § 5.4
- [last baselines](#last-baseline-set), in § 9.1
- [last-baseline self-alignment](#baseline-self-alignment), in § 6.4
- [last baseline set](#last-baseline-set), in § 9.1
- [left](#valdef-justify-content-left), in § 4.1
- [legacy](#valdef-justify-items-legacy), in § 7.1
- normal
  - [value for align-self](#valdef-align-self-normal), in § 6.2
  - [value for justify-content, align-content](#valdef-justify-content-normal), in § 5.1
  - [value for justify-self](#valdef-justify-self-normal), in § 6.1
  - [value for row-gap, column-gap, gap](#valdef-row-gap-normal), in § 8.1
- [overflow alignment](#overflow-alignment), in § 4.4
- [\<overflow-position\>](#typedef-overflow-position), in § 4.4
- [place-content](#propdef-place-content), in § 5.2
- [place-items](#propdef-place-items), in § 7.3
- [place-self](#propdef-place-self), in § 6.3
- [positional alignment](#positional-alignment), in § 4.1
- [right](#valdef-justify-content-right), in § 4.1
- [row-gap](#propdef-row-gap), in § 8.1
- [safe](#valdef-overflow-position-safe), in § 4.4
- [Self-alignment](#self-align), in § 6
- [self-alignment properties](#self-alignment-properties), in § 6
- [self-end](#valdef-self-position-self-end), in § 4.1
- [\<self-position\>](#typedef-self-position), in § 4.1
- [self-start](#valdef-self-position-self-start), in § 4.1
- [shared alignment context](#shared-alignment-context), in § 9.2
- [space-around](#valdef-align-content-space-around), in § 4.3
- [space-between](#valdef-align-content-space-between), in § 4.3
- [space-evenly](#valdef-align-content-space-evenly), in § 4.3
- [start](#valdef-self-position-start), in § 4.1
- [static-position rectangle](#static-position-rectangle), in § Unnumbered section
- stretch
  - [value for align-content, justify-content, \<content-distribution\>](#valdef-align-content-stretch), in § 4.3
  - [value for align-self](#valdef-align-self-stretch), in § 6.2
  - [value for justify-self](#valdef-justify-self-stretch), in § 6.1
- [synthesize](#synthesize-baseline), in § 9.1
- [synthesize baseline](#synthesize-baseline), in § 9.1
- [synthesized](#synthesize-baseline), in § 9.1
- [synthesized baseline](#synthesize-baseline), in § 9.1
- [unsafe](#valdef-overflow-position-unsafe), in § 4.4

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ANCHOR-POSITION-1\] defines the following terms:
  - <a id="5013f3ce"></a>position-area
- \[CSS-BOX-4\] defines the following terms:
  - <a id="60669dde"></a>box edge
  - <a id="f72f5cb4"></a>content box
  - <a id="cba8daea"></a>content edge
  - <a id="253362bb"></a>margin
  - <a id="0778a939"></a>margin box
  - <a id="16ff1cf8"></a>margin edge
  - <a id="a3a070bd"></a>padding
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="972b685d"></a>fragmentation break
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="4905669f"></a>inherited value
  - <a id="6b448e93"></a>initial value
  - <a id="19fd0eed"></a>legacy name alias
  - <a id="e14541aa"></a>shorthand
  - <a id="980ac56a"></a>shorthand property
  - <a id="1a2b1083"></a>used value
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="2ccfe434"></a>display
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="8d18d112"></a>block container
  - <a id="5a1cd654"></a>block formatting context
  - <a id="a015488b"></a>block-level
  - <a id="91b1f11d"></a>block-level box
  - <a id="0923db9e"></a>containing block
  - <a id="8be4ac1c"></a>establish an independent formatting context
  - <a id="ae223697"></a>formatting context
  - <a id="6658d41f"></a>in-flow
  - <a id="4e639b69"></a>inline-block
  - <a id="6b9bba07"></a>inline-level
  - <a id="febab3e8"></a>inline-level box
  - <a id="38e3f81d"></a>inline-table
  - <a id="e89ddbcb"></a>non-replaced
  - <a id="cbab059a"></a>non-replaced element
  - <a id="ff5f937c"></a>out-of-flow
  - <a id="a9db5d6d"></a>replaced element
- \[CSS-FLEXBOX-1\] defines the following terms:
  - <a id="dc8950b5"></a>column
  - <a id="92f00547"></a>column-reverse
  - <a id="4e512e00"></a>cross axis
  - <a id="855cee06"></a>flex
  - <a id="cc7f0a64"></a>flex container
  - <a id="31765884"></a>flex formatting context
  - <a id="9f6d5ab0"></a>flex item
  - <a id="dcaa31ad"></a>flex line
  - <a id="720fa00b"></a>flex-direction
  - <a id="546f7867"></a>flex-flow
  - <a id="98f2297b"></a>main axis
  - <a id="c7f264f9"></a>main-axis
  - <a id="97651ccf"></a>multi-line flex container
  - <a id="c599e49a"></a>row
  - <a id="bbd02de8"></a>row-reverse
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="5da00747"></a>first available font
- \[CSS-GRID-1\] defines the following terms:
  - <a id="cee62771"></a>collapsed gutter
- \[CSS-GRID-2\] defines the following terms:
  - <a id="7086fc61"></a>grid area
  - <a id="185dede5"></a>grid column
  - <a id="df72a52c"></a>grid container
  - <a id="ba30fc9a"></a>grid item
  - <a id="fb55798b"></a>grid row
  - <a id="b4a14210"></a>grid track
  - <a id="88fad469"></a>grid-placement property
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="6818bc7b"></a>alignment-baseline
  - <a id="cbd2a05c"></a>auto
  - <a id="0017eec4"></a>baseline-source
  - <a id="103cc3d2"></a>bottom
  - <a id="e06c6241"></a>dominant-baseline
  - <a id="131d41e4"></a>middle
  - <a id="8af3edff"></a>root inline box
  - <a id="32b99c6b"></a>top
- \[CSS-LOGICAL-1\] defines the following terms:
  - <a id="5a0724e6"></a>inset properties
- \[CSS-MULTICOL-1\] defines the following terms:
  - <a id="3126ae25"></a>column box
  - <a id="825824a2"></a>multi-column container
- \[CSS-MULTICOL-2\] defines the following terms:
  - <a id="8e80a770"></a>auto
  - <a id="49e139ba"></a>column-height
  - <a id="7777143d"></a>column-width
  - <a id="739c03e9"></a>multi-column spanner
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="add377f4"></a>overflow
  - <a id="a3cabdb1"></a>scroll container
  - <a id="3ed7991e"></a>scrollable overflow area
  - <a id="56c1c575"></a>unreachable scrollable overflow region
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="d9b71dda"></a>absolutely position
  - <a id="58d7f97c"></a>absolutely-positioned
  - <a id="22a281b0"></a>auto
  - <a id="f411d42d"></a>bottom
  - <a id="6562e50f"></a>inset-modified containing block
  - <a id="ebcbc56d"></a>left
  - <a id="c1223dc6"></a>original containing block
  - <a id="a5bae6ee"></a>right
  - <a id="b005b3ed"></a>static position
  - <a id="d4422ae3"></a>static-position containing block
  - <a id="f99d4ae2"></a>top
- \[CSS-POSITION-4\] defines the following terms:
  - <a id="d0cc7543"></a>fixed containing block
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="c20b5ff5"></a>auto
  - <a id="37f6dbd7"></a>automatic size
  - <a id="c1c732b9"></a>available space
  - <a id="ded576b5"></a>cyclic percentage size
  - <a id="f15ee6fc"></a>fit-content size
  - <a id="5ad01cca"></a>height
  - <a id="97ac8088"></a>stretch-fit size
  - <a id="49731d1d"></a>width
- \[CSS-SIZING-4\] defines the following terms:
  - <a id="6b530a45"></a>fit-content
  - <a id="133ee38d"></a>stretch
- \[CSS-TEXT-3\] defines the following terms:
  - <a id="36e5f32e"></a>text-align
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="4eb9d37e"></a>\|
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="fb688f4f"></a>direction
  - <a id="b095132a"></a>dominant baseline
  - <a id="ac7161d9"></a>end
  - <a id="04e5ac3a"></a>start
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="ddf25d36"></a>block flow direction
  - <a id="83d2ef35"></a>block-end
  - <a id="1118d052"></a>block-start
  - <a id="303c8d41"></a>flow-relative
  - <a id="8ebdd273"></a>horizontal-tb
  - <a id="a6eb24bb"></a>inline axis
  - <a id="18bb1084"></a>inline size
  - <a id="82ddda8c"></a>inline-axis
  - <a id="4da3b716"></a>inline-end
  - <a id="0da67e16"></a>inline-start
  - <a id="41c23fff"></a>line orientation
  - <a id="4f19c3e6"></a>line-left
  - <a id="0ad9204c"></a>line-over
  - <a id="10d0d189"></a>line-right
  - <a id="401cafe5"></a>line-under
  - <a id="24c75626"></a>ltr
  - <a id="66cecd25"></a>physical direction
  - <a id="1fe722d8"></a>physical left
  - <a id="e3eb3349"></a>physical right
  - <a id="dea08a34"></a>rtl
  - <a id="e51c8aeb"></a>vertical writing mode
  - <a id="35f596e9"></a>vertical-lr
  - <a id="ee88ce59"></a>vertical-rl
  - <a id="eb6008ce"></a>writing mode
- \[CSS2\] defines the following terms:
  - <a id="8d1fc8d9"></a>border-spacing
  - <a id="bdef30be"></a>line box
  - <a id="0f0ab49f"></a>max-height
  - <a id="4d8f6525"></a>max-width
  - <a id="62b90f98"></a>min-height
  - <a id="1ecca6e7"></a>min-width
  - <a id="5a3d5e86"></a>vertical-align
- \[ECMA-426\] defines the following terms:
  - <a id="f648eac4"></a>column
- \[SELECTORS-3\] defines the following terms:
  - <a id="e73425ca"></a>first formatted line

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-anchor-position-1"></a>\[CSS-ANCHOR-POSITION-1\]  
Tab Atkins Jr.; Elika Etemad; Ian Kilpatrick. [CSS Anchor Positioning Module Level 1](https://www.w3.org/TR/css-anchor-position-1/). 22 December 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-anchor-position-1&#x2F;](https://www.w3.org/TR/css-anchor-position-1/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 6 November 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-flexbox-1"></a>\[CSS-FLEXBOX-1\]  
Elika Etemad; Tab Atkins Jr.; Rossen Atanassov. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 14 October 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 1 February 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-grid-1"></a>\[CSS-GRID-1\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 1](https://www.w3.org/TR/css-grid-1/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-1&#x2F;](https://www.w3.org/TR/css-grid-1/)

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Elika Etemad. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 18 December 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-logical-1"></a>\[CSS-LOGICAL-1\]  
Elika Etemad; Rossen Atanassov. [CSS Logical Properties and Values Module Level 1](https://www.w3.org/TR/css-logical-1/). 4 December 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-logical-1&#x2F;](https://www.w3.org/TR/css-logical-1/)

<a id="biblio-css-multicol-1"></a>\[CSS-MULTICOL-1\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 1](https://www.w3.org/TR/css-multicol-1/). 16 May 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-1&#x2F;](https://www.w3.org/TR/css-multicol-1/)

<a id="biblio-css-multicol-2"></a>\[CSS-MULTICOL-2\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 2](https://www.w3.org/TR/css-multicol-2/). 19 December 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-2&#x2F;](https://www.w3.org/TR/css-multicol-2/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-position-4"></a>\[CSS-POSITION-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 4](https://www.w3.org/TR/css-position-4/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-4&#x2F;](https://www.w3.org/TR/css-position-4/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-sizing-4"></a>\[CSS-SIZING-4\]  
Tab Atkins Jr.; Elika Etemad; Jen Simmons. [CSS Box Sizing Module Level 4](https://www.w3.org/TR/css-sizing-4/). 20 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-4&#x2F;](https://www.w3.org/TR/css-sizing-4/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 30 September 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-ecma-426"></a>\[ECMA-426\]  
[Source map format specification](https://tc39.es/ecma426/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;tc39&#x2E;es&#x2F;ecma426&#x2F;](https://tc39.es/ecma426/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-3"></a>\[SELECTORS-3\]  
Tantek Çelik; et al. [Selectors Level 3](https://www.w3.org/TR/selectors-3/). 6 November 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-3&#x2F;](https://www.w3.org/TR/selectors-3/)

### <a id="informative"></a>Informative References

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                                                                                                                                                          | Initial                   | Applies to                                                                  | Inh. | %ages                                                | Anim­ation type         | Canonical order | Com­puted value                                                 |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------|-----------------------------------------------------------------------------|------|------------------------------------------------------|------------------------|-----------------|----------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-align-content②⑤"></a></span><a href="#propdef-align-content">align-content</a>&#xA;      </strong> | normal \| \<baseline-position\> \| \<content-distribution\> \| \<overflow-position\>? \<content-position\>                                                                                     | normal                    | block containers, multicol containers, flex containers, and grid containers | no   | n/a                                                  | discrete               | per grammar     | specified keyword(s)                                           |
| <strong><span><a id="ref-for-propdef-align-items⑧"></a></span><a href="#propdef-align-items">align-items</a>&#xA;      </strong> | normal \| stretch \| \<baseline-position\> \| \<overflow-position\>? \<self-position\>                                                                                                         | normal                    | all elements                                                                | no   | n/a                                                  | discrete               | per grammar     | specified keyword(s)                                           |
| <strong><span><a id="ref-for-propdef-align-self②⑨"></a></span><a href="#propdef-align-self">align-self</a>&#xA;      </strong> | auto \| \<overflow-position\>? \[ normal \| \<self-position\> \]\| stretch \| \<baseline-position\>                                                                                            | auto                      | flex items, grid items, and absolutely-positioned boxes                     | no   | n/a                                                  | discrete               | per grammar     | specified keyword(s)                                           |
| <strong><span><a id="ref-for-propdef-column-gap①①"></a></span><a href="#propdef-column-gap">column-gap</a>&#xA;      </strong> | normal \| \<length-percentage \[0,∞\]\>                                                                                                                                                        | normal                    | multi-column containers, flex containers, grid containers                   | no   | see                                                  | by computed value type | per grammar     | specified keyword, else a computed \<length-percentage\> value |
| <strong><span><a id="ref-for-propdef-gap①①"></a></span><a href="#propdef-gap">gap</a>&#xA;      </strong> | \<'row-gap'\> \<'column-gap'\>?                                                                                                                                                                | see individual properties | multi-column containers, flex containers, grid containers                   | no   | refer to corresponding dimension of the content area | by computed value type | per grammar     | see individual properties                                      |
| <strong><span><a id="ref-for-propdef-justify-content②⑤"></a></span><a href="#propdef-justify-content">justify-content</a>&#xA;      </strong> | normal \| \<content-distribution\> \| \<overflow-position\>? \[ \<content-position\> \| left \| right \]                                                                                       | normal                    | multicol containers, flex containers, and grid containers                   | no   | n/a                                                  | discrete               | per grammar     | specified keyword(s)                                           |
| <strong><span><a id="ref-for-propdef-justify-items①①"></a></span><a href="#propdef-justify-items">justify-items</a>&#xA;      </strong> | normal \| stretch \| \<baseline-position\> \| \<overflow-position\>? \[ \<self-position\> \| left \| right \] \| legacy \| legacy &#x26;&#x26; \[ left \| right \| center \] | legacy                    | all elements                                                                | no   | n/a                                                  | discrete               | per grammar     | specified keyword(s), except for legacy (see prose)            |
| <strong><span><a id="ref-for-propdef-justify-self③⓪"></a></span><a href="#propdef-justify-self">justify-self</a>&#xA;      </strong> | auto \| \<overflow-position\>? \[ normal \| \<self-position\> \| left \| right \] \| stretch \| \<baseline-position\>                                                                          | auto                      | block-level boxes, absolutely-positioned boxes, and grid items              | no   | n/a                                                  | discrete               | per grammar     | specified keyword(s)                                           |
| <strong><span><a id="ref-for-propdef-place-content②"></a></span><a href="#propdef-place-content">place-content</a>&#xA;      </strong> | \<'align-content'\> \<'justify-content'\>?                                                                                                                                                     | normal                    | see individual properties                                                   | no   | n/a                                                  | discrete               | per grammar     | see individual properties                                      |
| <strong><span><a id="ref-for-propdef-place-items②"></a></span><a href="#propdef-place-items">place-items</a>&#xA;      </strong> | \<'align-items'\> \<'justify-items'\>?                                                                                                                                                         | see individual properties | all elements                                                                | no   | n/a                                                  | discrete               | per grammar     | see individual properties                                      |
| <strong><span><a id="ref-for-propdef-place-self②"></a></span><a href="#propdef-place-self">place-self</a>&#xA;      </strong> | \<'align-self'\> \<'justify-self'\>?                                                                                                                                                           | auto                      | see individual properties                                                   | no   | n/a                                                  | discrete               | per grammar     | see individual properties                                      |
| <strong><span><a id="ref-for-propdef-row-gap①①"></a></span><a href="#propdef-row-gap">row-gap</a>&#xA;      </strong> | normal \| \<length-percentage \[0,∞\]\>                                                                                                                                                        | normal                    | multi-column containers, flex containers, grid containers                   | no   | see                                                  | by computed value type | per grammar     | specified keyword, else a computed \<length-percentage\> value |

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Make it easier to understand the dual-axis nature of "start" and "end" wrt orthogonal flows. [↵](#issue-bb5da799)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Add example images here. [↵](#issue-13b9ed6a)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> It may not be Web-compatible to implement the “smart” default behavior (though we hope so, and believe it to be likely), so UAs should pass any feedback on this point to the WG. UAs that have not implemented the “smart” default behavior must behave as [safe](#valdef-overflow-position-safe) for [align-content](#propdef-align-content) on [block containers](https://www.w3.org/TR/css-display-4/#block-container) and [unsafe](#valdef-overflow-position-unsafe) otherwise. [↵](#issue-6c9376ad)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Replace this image too. [↵](#issue-f4825563)
