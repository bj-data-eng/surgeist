Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Grid Layout Module Level 3](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Grid Layout Module Level 3

Source snapshot: https://www.w3.org/TR/2026/WD-css-grid-3-20260121/

Snapshot SHA-256: ab3a5d476f748764136b3ad89f2b4c9c9b47d6d50304aaa5d0a02319b9f5f46b

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 9 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Grid Layout Module Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This module introduces a one-dimensional grid layout mode for [CSS Grid](https://www.w3.org/TR/css-grid-2/) containers.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-grid” in the title, like this: “\[css-grid\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-grid%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

<em>This section is not normative.</em>

Grid Layout is a layout model for CSS that has powerful abilities to control the sizing and positioning of boxes and their contents. Grid Layout is optimized for 2-dimensional layouts: those in which alignment of content is desired in both dimensions.

![An example of grid layout: two rows of items, the first being four items — the last of which spans both rows, and the second being two items — the first of which spans the first two columns — plus the spanned item from the first row.](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/grid-layout.png)

Representative Grid layout example

Although many layouts can be expressed with regular Grid Layout, restricting items into a grid in both axes also makes it impossible to express some common layouts on the Web.

<a id="ref-for-grid-layout"></a>

This module defines a variant of [Grid Layout](https://www.w3.org/TR/css-grid-2/#grid-layout) that removes that restriction so that items can be placed into Grid-like tracks in one axis while stacking one after another in the other. Items not explicitly placed into specific tracks are placed into the column (or row) with the most remaining space based on the layout size of the items placed so far.

### <a id="background"></a>1.1.  Background and Motivation

#### <a id="waterfall"></a>1.1.1.  Waterfall Layout with Auto-placed Items

<a id="ref-for-grid-lanes-layout"></a>

<a id="ref-for-auto-placement"></a>

[Grid lanes layout](#grid-lanes-layout), sometimes also called “masonry layout” or “waterfall layout”, is a common Web design pattern where a number of items—​commonly images or short article summaries—​are placed one by one into columns in a way that loosely resembles masonry construction. Unlike [multi-column layout](#biblio-css-multicol-1) and [multi-line column flex layout](#biblio-css-flexbox-1), where content is placed vertically in the first column until it must spills over to the second column, [automatic placement](https://www.w3.org/TR/css-grid-2/#auto-placement) in <a id="ref-for-grid-lanes-layout①"></a>grid lanes layout selects a column for each new item such that it is generally closer to the top of the layout than items placed later.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1d7a673e"></a> The traditional Pinterest search results page exemplifies this layout:
>
> ![An example of grid lanes layout: four columns of items, each item is placed into the column with the smallest height so far.](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/pinterest.png)
>
> Representative grid lanes layout example
>
> Here, each item has a different height (depending on the content and the width of the column), and inspecting the DOM reveals (as the visual content itself gives no indication of ordering) that each item has been placed into the column with the smallest height so far.

This layout superficially looks similar to multi-column layout; but it has the advantage that scrolling down will naturally lead to “later” items in the layout (such as those less relevant in the search results).

It’s not possible to achieve this layout using earlier CSS layout models, unless you know up-front how tall each item will be, or use JavaScript for content measurement or placement.

<a id="ref-for-propdef-display"></a>

<a id="ref-for-propdef-grid-template-columns"></a>

<a id="ref-for-grid-lanes-layout②"></a>

Using [display: grid-lanes](https://www.w3.org/TR/css-display-4/#propdef-display) together with [grid-template-columns](https://www.w3.org/TR/css-grid-2/#propdef-grid-template-columns) yields this type of [grid lanes layout](#grid-lanes-layout)

#### <a id="collapse"></a>1.1.2.  One-dimensional Grid Layout

<a id="ref-for-grid-layout①"></a>

[Grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout) allows for powerful track sizing and explicit placement in two axes, but sometimes a layout only needs alignment of its items in one dimension.

<a id="ref-for-grid-lanes-layout③"></a>

Using [grid lanes layout](#grid-lanes-layout) together with explicitly-positioned items allows for this type of one-dimensional grid layout.

<a id="ref-for-grid-layout②"></a>

<a id="ref-for-flex-layout"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a1b07c6f"></a> This example [by Douglas Graham](https://github.com/w3c/csswg-drafts/issues/10233#issuecomment-2071279204) uses explicit positioning to place each item into its assigned column; but there are no rows. Instead, items in each column stack one after the other. This layout can’t be duplicated in [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout) because the “spanning” relationships among the items in adjacent columns is not fixed: it depends on their relative heights and whether the optional banner or advertisement items are included. It also can’t be duplicated in [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout) because the source order of the items (which is used for reading, sequential navigation, and one-column mobile phone layout) goes back and forth between the two columns.
>
> ![In one-column layout: the header, followed by an optional banner, the secondary navigation, the main content area, an advertisement block, and finally the footer. In two-column layout: the header spanning both columns on top, the footer spanning both columns at the bottom, in the wider left column the optional banner followed by the main content area, and in the narrow left column the secondary navigation followed by the advertisement block.](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/masonry-page-layout.png)
>
> Comparison of one-column and two-column variants of a one-dimensional grid layout.

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units [\[CSS-VALUES-3\]](#biblio-css-values-3). Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="grid-lanes-model"></a>2. <a id="masonry-model"></a> Grid Lanes Layout Model

<a id="ref-for-grid-layout③"></a>

<a id="ref-for-flex-layout①"></a>

<a id="ref-for-grid-lanes-layout④"></a>

<a id="grid-lanes-layout"></a>Grid lanes layout lays out items into pre-defined tracks similar to [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout) in one axis (called the <a id="grid-axis"></a>grid axis), but flows them freely similar to [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout) in the other (called the <a id="stacking-axis"></a>stacking axis). Similar to <a id="ref-for-grid-layout④"></a>grid layout and unlike <a id="ref-for-flex-layout②"></a>flex layout, [grid lanes layout](#grid-lanes-layout)’s auto-placement distributes items across the tracks to keep the lengths of those tracks as similar as possible.

<a id="ref-for-grid-item"></a>

<a id="ref-for-blockify"></a>

<a id="ref-for-grid-container"></a>

[Grid items](https://www.w3.org/TR/css-grid-2/#grid-item) are formed and [blockified](https://www.w3.org/TR/css-display-4/#blockify) exactly the same as in a regular [grid container](https://www.w3.org/TR/css-grid-2/#grid-container).

<a id="ref-for-grid-container①"></a>

<a id="ref-for-propdef-order"></a>

All CSS properties work the same as in a regular [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) unless otherwise specified by this specification. For example, [order](https://www.w3.org/TR/css-display-4/#propdef-order) can be used to specify a different layout order for the items.

<a id="ref-for-grid-axis"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Subgrid items are supported, but subgridding only occurs in the [grid axis](#grid-axis); see [§ 3.2 Subgrids](#subgrids) for details.

<a id="ref-for-grid-container②"></a>

<a id="ref-for-grid-lanes-layout⑤"></a>

<a id="ref-for-grid-lanes-container"></a>

<a id="ref-for-grid-track"></a>

<a id="ref-for-stacking-axis"></a>

<a id="ref-for-block-axis"></a>

<a id="ref-for-inline-axis"></a>

A <a id="grid-lanes-container"></a>grid lanes container is a [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) whose contents participate in [grid lanes layout](#grid-lanes-layout). A [grid lanes container](#grid-lanes-container) creates column [tracks](https://www.w3.org/TR/css-grid-2/#grid-track) if its [stacking axis](#stacking-axis) is the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis), or row <a id="ref-for-grid-track①"></a>tracks if its <a id="ref-for-stacking-axis①"></a>stacking axis is the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis).

<strong>Table 1 — structured row/cell transcription</strong>

Comparing Grid Lanes Containers

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Column Lanes

<strong>Column 2 (data cell):</strong>

```text
grid-template-columns: 1fr 2fr 3fr;
```
<strong>Column 3 (data cell):</strong>

![Column grid lanes layout lays out items in columns, but ordered across the columns, placing each item in the then-shortest column.](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/masonry-columns.png)

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

Row Lanes

<strong>Column 2 (data cell):</strong>

```text
grid-template-rows: 1fr 2fr 3fr;
```
<strong>Column 3 (data cell):</strong>

![Row grid lanes layout lays out items in rows, but ordered down across the rows, placing each item in the then-shortest row.](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/masonry-rows.png)

### <a id="order-accessibility"></a>2.1.  Reordering and Accessibility

<a id="ref-for-grid-lanes-layout⑥"></a>

<a id="ref-for-propdef-flow-tolerance"></a>

<a id="ref-for-stacking-axis②"></a>

<a id="ref-for-definite-grid-position"></a>

Although [grid lanes layout](#grid-lanes-layout) generally progresses in a forwards fashion (placing the next item endward of the current item in at least one axis, matching the natural “reading order”), it can switch between endward in the inline or block axis in a seemingly arbitrary manner. In simple cases, the [flow-tolerance](#propdef-flow-tolerance) property can help reduce the feeling of backtracking due to small sizing differences in the [stacking axis](#stacking-axis) when laying out auto-placed items. But when [auto-placement](#grid-lanes-layout-algorithm) is mixed with [explicit placement](https://www.w3.org/TR/css-grid-2/#definite-grid-position) or spanning items, some amount of backtracking may occur.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0b32dd26"></a> For example, in the following markup sample, the fourth item is a spanner that doesn’t fit in the remaining empty column on the first line. It ends up positioned into the into the first column, which is the highest available space into which it will fit. The next few items, which have a span of 1, end up laying out “above” it in the empty column, violating the natural reading order.
>
> ```text
> <section class=masonry>
>   <div class=item>1</div>
>   <div class=item>2</div>
>   <div class="item tall">3</div>
>   <div class="item wide">4</div>
>   <div class=item>5</div>
>   <div class=item>6</div>
>   <div class=item>7</div>
> </section>
> <style>
> .masonry {
>   display: grid-lanes;
>   grid-template-columns: repeat(5, auto);
> }
> .item { height: 50px; }
> .item.wide { grid-column: span 3; }
> .item.tall { height: 90px; }
> </style>
> ```
>
> ![In this example, the first row is items 1, 2, 3, 5, 6, with item 3 slightly taller than the others. Item 4 spans the first three columns, and is placed just below item 3, while item 7 is tucked under item 5.](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/masonry-reorder-span.png)
>
> Auto-placed grid lanes layout with mixed-height items and mixed span sizes
>
> Similarly, items explicitly placed into specific tracks can leave gaps behind them, into which subsequent auto-placed items can be placed visually out-of-order.

<a id="ref-for-propdef-reading-flow"></a>

Authors should be aware of these possibilities and design layouts where such backtracking is minimized so that focus and reading order can be more easily followed. Alternatively, if the items do not have an inherent order, use the [reading-flow](https://www.w3.org/TR/css-display-4/#propdef-reading-flow) property to allow the UA to re-order the items for reading and linear navigation.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f3901e05"></a> Or should reordering be the default behavior for auto-placed items here?

> <strong data-conversion-semantic="note">Note</strong>
>
> Techniques for reducing backtracking include:
>
> - <a id="ref-for-propdef-flow-tolerance①"></a>
>
>   Using appropriate values for [flow-tolerance](#propdef-flow-tolerance), i.e. values large enough to avoid gratuitous differentiation among similarly-sized tracks, but not so large that meaningful differences get ignored.
>
> - Using explicit placement in ways that help group related items together, rather than ways that disrupt the natural order of items
>
> - <a id="ref-for-grid-axis①"></a>
>
>   <a id="ref-for-stacking-axis③"></a>
>
>   Avoiding the combination of mixed span sizes in the [grid axis](#grid-axis) and disparate item sizes in the [stacking axis](#stacking-axis), which can cause items to get pulled out of order (see example above).

<a id="ref-for-grid-layout⑤"></a>

<a id="ref-for-flex-layout③"></a>

<a id="ref-for-propdef-order①"></a>

As with [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout) and [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout) authors can use the [order](https://www.w3.org/TR/css-display-4/#propdef-order) property to re-order items; the same caveats apply. See [CSS Grid Layout 2 § 4 Reordering and Accessibility](https://www.w3.org/TR/css-grid-2/#order-accessibility) and [CSS Display 4 § 3.1 Reordering and Accessibility](https://www.w3.org/TR/css-display-4/#order-accessibility).

### <a id="grid-lanes-containers"></a>2.2.  Establishing Grid Lanes Layout

<strong>Table 2 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-display①"></a>

[display](https://www.w3.org/TR/css-display-4/#propdef-display)

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[New values:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one"></a>

grid-lanes [\|](https://www.w3.org/TR/css-values-4/#comb-one) inline-grid-lanes

<a id="valdef-display-grid-lanes"></a>grid-lanes  
<a id="ref-for-grid-lanes-container①"></a>

<a id="ref-for-block-level"></a>

This value causes an element to generate a [block-level](https://www.w3.org/TR/css-display-4/#block-level) [grid lanes container](#grid-lanes-container) box.

<a id="valdef-display-inline-grid-lanes"></a>inline-grid-lanes  
<a id="ref-for-grid-lanes-container②"></a>

<a id="ref-for-inline-level"></a>

This value causes an element to generate an [inline-level](https://www.w3.org/TR/css-display-4/#inline-level) [grid lanes container](#grid-lanes-container) box.

<a id="ref-for-grid-lanes-container③"></a>

<a id="ref-for-subgrid"></a>

<a id="ref-for-grid-axis②"></a>

<a id="ref-for-independent-formatting-context"></a>

A [grid lanes container](#grid-lanes-container) that is not [subgridded](https://www.w3.org/TR/css-grid-2/#subgrid) in its [grid axis](#grid-axis) establishes an [independent formatting context](https://www.w3.org/TR/css-display-4/#independent-formatting-context) for its contents.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-3be4bca6"></a> ISSUE(12820): Write up how grid/masonry formatting contexts work more formally?

### <a id="grid-lanes-orientation"></a>2.3.  Orienting Grid Lanes Layout

<a id="ref-for-grid-lanes-container④"></a>

<a id="ref-for-grid-axis③"></a>

<a id="ref-for-inline-axis①"></a>

<a id="ref-for-block-axis①"></a>

The orientation of a [grid lanes container](#grid-lanes-container), i.e. whether its [grid axis](#grid-axis) is the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) (establishing columns) or the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) (establishing rows) is determined by the <strong data-conversion-semantic="issue">Issue:</strong> <a id="issue-d4908b1e"></a>TBD property.

<a id="ref-for-initial-value"></a>

The [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value) of this property is <a id="valdef-grid-auto-flow-normal"></a>normal, which determines the orientation from the grid-template-\* properties:

- <a id="ref-for-propdef-grid-template-columns①"></a>

  <a id="ref-for-valdef-grid-template-rows-none"></a>

  <a id="ref-for-propdef-grid-template-rows"></a>

  <a id="ref-for-grid-container③"></a>

  <a id="ref-for-block-axis②"></a>

  <a id="ref-for-grid-axis④"></a>

  If [grid-template-columns](https://www.w3.org/TR/css-grid-2/#propdef-grid-template-columns) is [none](https://www.w3.org/TR/css-grid-2/#valdef-grid-template-rows-none) and [grid-template-rows](https://www.w3.org/TR/css-grid-2/#propdef-grid-template-rows) is not <a id="ref-for-valdef-grid-template-rows-none①"></a>none, the [grid container](https://www.w3.org/TR/css-grid-2/#grid-container)’s [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) is the [grid axis](#grid-axis) (establishing rows).

- <a id="ref-for-grid-container④"></a>

  <a id="ref-for-inline-axis②"></a>

  <a id="ref-for-grid-axis⑤"></a>

  Otherwise (thus by default), the [grid container](https://www.w3.org/TR/css-grid-2/#grid-container)’s [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) is the [grid axis](#grid-axis) (establishing columns).

<a id="ref-for-grid-lanes-container⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5742456b"></a> The following code will create a 2-column (“waterfall style”) [grid lanes container](#grid-lanes-container) that grows downward:
>
> ```text
> .container {
>   display: grid-lanes;
>   grid-template-columns: 100px 200px;
> }
> ```
>
> <a id="ref-for-grid-lanes-container⑥"></a>
>
> while the following code will create a 2-row (“brick wall style”) [grid lanes container](#grid-lanes-container) that grows horizontally:
>
> ```text
> .container {
>   display: grid-lanes;
>   grid-template-rows: 100px 200px;
> }
> ```
<a id="ref-for-propdef-grid-auto-flow"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-ff7d700c"></a> Figure out whether we are re-using [grid-auto-flow](https://www.w3.org/TR/css-grid-2/#propdef-grid-auto-flow) here (and what it’s values mean) or defining a new property like grid-lanes-direction. [\[Issue \#12803\]](https://github.com/w3c/csswg-drafts/issues/12803)

## <a id="grid-lanes-track-templates"></a>3. <a id="masonry-track-templates"></a> Grid Lanes Track Specification

<a id="ref-for-grid-axis⑥"></a>

<a id="ref-for-grid-layout⑥"></a>

In the [grid axis](#grid-axis), the full power of [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout) is available for track specification:

- <a id="ref-for-grid-lanes-container⑦"></a>

  <a id="ref-for-grid-axis⑦"></a>

  <a id="ref-for-grid-layout⑦"></a>

  Track sizes, line names, and areas can be specified on the [grid lanes container](#grid-lanes-container)’s [grid axis](#grid-axis), just like in [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout).

- <a id="ref-for-explicit-grid"></a>

  <a id="ref-for-implicit-grid"></a>

  <a id="ref-for-grid-container⑤"></a>

  The [explicit grid](https://www.w3.org/TR/css-grid-2/#explicit-grid) and [implicit grid](https://www.w3.org/TR/css-grid-2/#implicit-grid) are formed in the same way as for a regular [grid container](https://www.w3.org/TR/css-grid-2/#grid-container).

- <a id="ref-for-grid-layout⑧"></a>

  Items can be [placed](#grid-lanes-track-placement) against these grid templates just as in [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout).

However, auto-placed items contribute sizing to all tracks, not just the track into which they are ultimately placed; see [§ 3.4 Grid Axis Track Sizing](#track-sizing).

<a id="ref-for-available"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is because auto-placed items must be laid out <em>as</em> they are placed, so that each track knows how “full” it is (and therefore which track should receive the next auto-placed item); thus, the tracks themselves must already have a definite size so that the items know their [available space](https://www.w3.org/TR/css-sizing-3/#available) during layout.

### <a id="grid-lanes-track-properties"></a>3.1.  Declaring Grid Lanes Track Templates: the grid-template-\* properties

<a id="ref-for-propdef-grid-auto-rows"></a>

<a id="ref-for-propdef-grid-auto-columns"></a>

<a id="ref-for-grid-axis⑧"></a>

<a id="ref-for-grid-lanes-container⑧"></a>

<a id="ref-for-grid-container⑥"></a>

<a id="ref-for-stacking-axis④"></a>

The grid-template-\* and [grid-auto-rows](https://www.w3.org/TR/css-grid-2/#propdef-grid-auto-rows)/[grid-auto-columns](https://www.w3.org/TR/css-grid-2/#propdef-grid-auto-columns) properties (and their shorthands) apply in the [grid axis](#grid-axis) of the [grid lanes container](#grid-lanes-container) and establish tracks just as on regular [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container). (They are ignored in the [stacking axis](#stacking-axis).)

#### <a id="intrinsic-auto-repeat"></a>3.1.1. <a id="masonry-intrinsic-repeat"></a> Intrinsic Tracks and repeat()

<a id="ref-for-funcdef-repeat-line-color-repeat"></a>

<a id="ref-for-min-track-sizing-function"></a>

<a id="ref-for-max-track-sizing-function"></a>

<a id="ref-for-definite"></a>

<a id="ref-for-typedef-auto-repeat"></a>

Level 3 extends the [repeat()](https://www.w3.org/TR/css-gaps-1/#funcdef-repeat-line-color-repeat) notation to allow repetitions where neither the [min track sizing function](https://www.w3.org/TR/css-grid-2/#min-track-sizing-function) nor the [max track sizing function](https://www.w3.org/TR/css-grid-2/#max-track-sizing-function) is [definite](https://www.w3.org/TR/css-sizing-3/#definite); in other words, the syntax for [\<auto-repeat\>](#typedef-auto-repeat) is relaxed to the following:

<a id="typedef-auto-repeat"></a>

<a id="ref-for-typedef-auto-repeat①"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-comb-comma"></a>

<a id="ref-for-typedef-line-names"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-typedef-track-size"></a>

<a id="ref-for-mult-one-plus"></a>

<a id="ref-for-typedef-line-names①"></a>

<a id="ref-for-mult-opt①"></a>

```text
<auto-repeat> = repeat(  [ auto-fill | auto-fit ] , [ <line-names>? <track-size> ]+ <line-names>? )
```
In order to resolve the number of repetitions, a hypothetical size is calculated for such tracks by initializing the track sizes (per [CSS Grid Layout 2 § 12.4 Initialize Track Sizes](https://www.w3.org/TR/css-grid-2/#algo-init)) and resolving intrinsic track sizes (per [CSS Grid Layout 2 § 12.5 Resolve Intrinsic Track Sizes](https://www.w3.org/TR/css-grid-2/#algo-content)) in accordance with [§ 3.4 Grid Axis Track Sizing](#track-sizing) with the following assumptions:

- <a id="ref-for-automatic-grid-position"></a>

  Ignore explicit item placement. (That is, assume all items have an [automatic position](https://www.w3.org/TR/css-grid-2/#automatic-grid-position).)

- <a id="ref-for-collapsed-grid-track"></a>

  Do not [collapse](https://www.w3.org/TR/css-grid-2/#collapsed-grid-track) any tracks.

- <a id="ref-for-auto-placement①"></a>

  Expand the repeated track listing to capture all possible [automatic placements](https://www.w3.org/TR/css-grid-2/#auto-placement) of each item, i.e. repeat the track listing <code>2 + (<var>largest span</var> - 2)/(<var>number of tracks in repeat()</var>)</code> times, rounded <em>down</em> to a whole number.

<a id="ref-for-funcdef-repeat-line-color-repeat①"></a>

The hypothetical size of each track in the [repeat()](https://www.w3.org/TR/css-gaps-1/#funcdef-repeat-line-color-repeat) listing is given by the largest track corresponding to that entry (by index) in the listing.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-000330f4"></a> For example, given a template of repeat(auto-fill, 50px auto auto) and a largest spanner of 2, you need to repeat the track listing twice, giving 50px auto auto 50px auto auto. After doing the hypothetical layout, the 2nd and 5th tracks are maxed together to provide a hypothetical size for the first auto and the 3rd and 6th tracks are maxed together to provide a hypothetical size for the second auto. Those concrete sizes are then used to determine how many repetitions will fill the container.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-c70906df"></a> Should this work also in Grid Layout somehow, or fall back to a single repetition? If so, how? [\[Issue \#10915\]](https://github.com/w3c/csswg-drafts/issues/10915)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This simplified layout heuristic is defined to be "good enough", while remaining fast and consistent. Ignoring placement is required just to make the concept coherent; before you know how many repetitions you need, you can’t tell what track an item (even one with a definite placement) will end up in.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-5b45728c"></a> Technically, if your explicit placement does not enter or cross over the repetition, we can know its placement prior to resolving the reptition. Do we want to adjust the algorithm above to account for this? That is, given auto repeat(auto-fill, ...) auto, if you explicitly place an item against line 1 or -1, we could let it only affect that track (rather than pretending it’ll go in every track).

### <a id="subgrids"></a>3.2.  Subgrids

<a id="ref-for-subgrid①"></a>

<a id="ref-for-grid-lanes-container⑨"></a>

<a id="ref-for-grid-container⑦"></a>

<a id="ref-for-grid-axis⑨"></a>

<a id="ref-for-stacking-axis⑤"></a>

[Subgridding](https://www.w3.org/TR/css-grid-2/#subgrid) allows nested [grid lanes containers](#grid-lanes-container) (and [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container)) to share track sizes. If the parent’s corresponding axis is a [grid axis](#grid-axis), the subgridded axis is taken from the parent container [as specified for grid containers](https://www.w3.org/TR/css-grid-2/#subgrids); if the parent’s corresponding axis is a [stacking axis](#stacking-axis), the subgridded axis also acts as a <a id="ref-for-stacking-axis⑥"></a>stacking axis.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-ff7026b1"></a> What if this conflicts with the lanes orientation, or results in both axes stacking?

<a id="ref-for-grid-lanes-layout⑦"></a>

<a id="ref-for-subgrid②"></a>

In [grid lanes layout](#grid-lanes-layout), auto-placed [subgrids](https://www.w3.org/TR/css-grid-2/#subgrid) don’t inherit any line names from their parent grid, because that would make the placement of the item dependent on layout results; but the subgrid’s tracks are still aligned to the parent’s tracks as usual.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-be8494db"></a> Here’s a subgrid [example](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/examples/subgrid-example-1.html):
>
> ```css
> <style>
> .grid {
>   display: inline-grid-lanes;
>   grid-template-rows: auto auto 100px;
>   align-content: center;
>   height: 300px;
>   border: 1px solid;
> }
> 
> .grid > * {
>   margin: 5px;
>   background: silver;
> }
> .grid > :nth-child(2n) {
>   background: pink;
> }
> 
> .grid subgrid {
>   display: grid;
>   grid: subgrid / subgrid;
>   grid-row: 2 / span 2;
>   grid-gap: 30px;
> }
> .grid subgrid > * { background: cyan; }
> </style>
> ```
>
> ```html
> <div class="grid">
>   <item>1</item>
>   <item>2</item>
>   <item>3</item>
>   <subgrid>
>     <item style="height:100px">subgrid.1</item>
>     <item>sub.2</item>
>     <item>s.3</item>
>   </subgrid>
>   <item>4</item>
>   <item>5</item>
>   <item style="width: 80px">6</item>
>   <item>7</item>
> </div>
> ```
>
> ![](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/subgrid-example-1.png)
>
> The rendering of the subgrid example above.
>
> <a id="ref-for-stacking-axis⑦"></a>
>
> <a id="ref-for-grid-lanes-container①⓪"></a>
>
> <a id="ref-for-inline-axis③"></a>
>
> Note how the subgrid’s first item ("subgrid.1") contributes to the intrinsic size of the 2nd row in the parent grid. This is possible since the subgrid specified a definite position so we know which tracks it will occupy. Note also that trying to subgrid the parent’s [stacking axis](#stacking-axis) results in the subgrid converting to a [grid lanes container](#grid-lanes-container) with its [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) as the <a id="ref-for-stacking-axis⑧"></a>stacking axis.

<a id="ref-for-subgrid③"></a>

<a id="ref-for-grid-lanes-container①①"></a>

A [subgrid](https://www.w3.org/TR/css-grid-2/#subgrid) that is a [grid lanes container](#grid-lanes-container) can be referred to as a <a id="grid-lanes-subgrid"></a>grid lanes subgrid.

<a id="ref-for-funcdef-repeat-line-color-repeat②"></a>

### <a id="repeat-notation"></a>3.3.  Track Repetition: the [repeat()](https://www.w3.org/TR/css-gaps-1/#funcdef-repeat-line-color-repeat) notation

<a id="ref-for-funcdef-repeat-line-color-repeat③"></a>

This specification introduces new keywords and grid-lanes–specific behavior for the [repeat()](https://www.w3.org/TR/css-gaps-1/#funcdef-repeat-line-color-repeat) notation.

#### <a id="repeat-auto-fit"></a>3.3.1.  repeat(auto-fit)

<a id="ref-for-grid-lanes-container①②"></a>

<a id="ref-for-grid-container⑧"></a>

<a id="ref-for-valdef-repeat-auto-fit"></a>

<a id="ref-for-valdef-repeat-auto-fill"></a>

<a id="ref-for-collapsed-grid-track①"></a>

In [grid lanes containers](#grid-lanes-container) (as in regular [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container)) [auto-fit](https://www.w3.org/TR/css-grid-2/#valdef-repeat-auto-fit) acts like [auto-fill](https://www.w3.org/TR/css-grid-2/#valdef-repeat-auto-fill), but with empty tracks [collapsed](https://www.w3.org/TR/css-grid-2/#collapsed-grid-track). However, because placement occurs after track sizing, <a id="ref-for-grid-lanes-container①③"></a>grid lanes containers use a heuristic to determine if a track will be occupied:

- All tracks occupied by explicitly placed items are considered occupied.

- With the sum of the spans of all auto-placed items as <var>N</var>, all unoccupied tracks up to the <var>N</var>th such track are considered occupied.

<a id="ref-for-valdef-repeat-auto-fit①"></a>

<a id="ref-for-collapsed-grid-track②"></a>

All tracks produced by the [auto-fit](https://www.w3.org/TR/css-grid-2/#valdef-repeat-auto-fit) repetition and considered unoccupied by this heuristic are assumed “empty” and are [collapsed](https://www.w3.org/TR/css-grid-2/#collapsed-grid-track). A <a id="ref-for-collapsed-grid-track③"></a>collapsed grid track cannot accept placement of auto-placed items.

<a id="ref-for-valdef-repeat-auto-fill①"></a>

<a id="ref-for-valdef-repeat-auto-fit②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is possible for an auto-placed item to be placed in a track when [auto-fill](https://www.w3.org/TR/css-grid-2/#valdef-repeat-auto-fill) is used that would be collapsed if [auto-fit](https://www.w3.org/TR/css-grid-2/#valdef-repeat-auto-fit) is used if there are auto-placed items with a span greater than 1 mixed with explicitly-placed items that leave gaps too small for the auto-placed items.

### <a id="track-sizing"></a>3.4.  Grid Axis Track Sizing

Track sizing works the same as in [CSS Grid](https://www.w3.org/TR/css-grid-2/#algo-track-sizing), except that when considering which items contribute to intrinsic sizes:

- All items explicitly placed in that track contribute, and

- <a id="ref-for-automatic-grid-position①"></a>

  All items with an [automatic grid position](https://www.w3.org/TR/css-grid-2/#automatic-grid-position) contribute (regardless of whether they are ultimately placed in that track).

<a id="ref-for-grid-axis①⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6d22f73d"></a> For example, suppose there are two columns in the [grid axis](#grid-axis) and that
>
> - Items A, B, and C have no explicit position.
>
> - Item D is explicitly placed into the first column.
>
> In this case, items A, B, C, and D all contribute to sizing the first column, while only A, B, and C (and not D) contribute to the second column.

<a id="ref-for-automatic-grid-position②"></a>

In the case of spanning items with an [automatic grid position](https://www.w3.org/TR/css-grid-2/#automatic-grid-position), they are assumed to be placed at every possible start position, and contribute accordingly.

<a id="ref-for-grid-axis①①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f3e866aa"></a> For example, suppose there are 5 columns in the [grid axis](#grid-axis), with the middle having a fixed size of 100px and the other two being auto-sized. For the purpose of track sizing, an item that spans 2 tracks and has an intrinsic contribution of 220px is essentially copied and assumed to exist:
>
> - At grid line 1, contributing 110px to each of the first two tracks.
>
> - At grid line 2, contributing 120px to the second track.
>
> - At grid line 3, contributing 120px to the fourth track.
>
> - At grid line 4, contributing 110px to the fourth and fifth tracks.

<a id="ref-for-grid-axis①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This algorithm ensures that each track is at least big enough to accommodate every item that is ultimately placed in it, and does not create dependency cycles between placement and track sizing. However, depending on the variation in sizes, tracks could be larger than necessary: an exact fit is only guaranteed if all items are explicitly placed in the [grid axis](#grid-axis) or all items are the same size (or matching multiples of that size, in the case of spanning items).

#### <a id="track-sizing-subgrid"></a>3.4.1.  Subgrid Item Contributions

<a id="ref-for-grid-container⑨"></a>

<a id="ref-for-grid-lanes-container①④"></a>

<a id="ref-for-grid-lanes-subgrid"></a>

<a id="ref-for-automatic-grid-position③"></a>

When sizing the tracks of either a regular [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) or a [grid lanes container](#grid-lanes-container), a [grid lanes subgrid](#grid-lanes-subgrid) has special handling of items that have an [automatic grid position](https://www.w3.org/TR/css-grid-2/#automatic-grid-position):

- <a id="ref-for-grid-lanes-subgrid①"></a>

  <a id="ref-for-definite-grid-position①"></a>

  <a id="ref-for-automatic-grid-position④"></a>

  Any such item is placed into every possible grid track that could be spanned by the [grid lanes subgrid](#grid-lanes-subgrid). (If the subgrid has a [definite grid position](https://www.w3.org/TR/css-grid-2/#definite-grid-position), thus only the spanned tracks; if it has an [automatic grid position](https://www.w3.org/TR/css-grid-2/#automatic-grid-position), then all tracks in the parent grid.)

- Any such item receives the largest margin/border/padding contribution of each edge at which it could hypothetically be placed. If the item spans the entire subgrid, it receives both. (See [CSS Grid Layout §9](https://www.w3.org/TR/css-grid-2/#subgrid-item-contribution).)

#### <a id="track-sizing-performance"></a>3.4.2.  Optimized Track Sizing

Track sizing can be optimized by aggregating items that have the same span size and placement into a single virtual item as follows:

1.  <a id="ref-for-grid-item①"></a>

    Separate all the [grid items](https://www.w3.org/TR/css-grid-2/#grid-item) into <a id="item-groups"></a>item groups, according to the following properties:

    - the span of the item

    - the placement of the item, i.e. which tracks it is allowed to be placed in

    - <a id="ref-for-baseline-sharing-group"></a>

      the item’s [baseline-sharing group](https://www.w3.org/TR/css-align-3/#baseline-sharing-group)

    <a id="ref-for-automatic-grid-position⑤"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: For example, an item with span 2 placed in the second track will be in a different group than an item with span 2 that has an [automatic grid position](https://www.w3.org/TR/css-grid-2/#automatic-grid-position).

2.  <a id="ref-for-item-groups"></a>

    For each [item group](#item-groups), synthesize a <a id="virtual-grid-item"></a>virtual grid item that has the maximum of every intrinsic size contribution among the items in that group.

    <a id="ref-for-baseline-alignment"></a>

    <a id="ref-for-virtual-grid-item"></a>

    If the items apply [baseline alignment](https://www.w3.org/TR/css-align-3/#baseline-alignment), determine the baselines of the [virtual grid item](#virtual-grid-item) by placing all of its items into a single hypothetical grid track and finding their shared baseline(s) and shims. Increase the group’s intrinsic size contributions accordingly.

3.  <a id="ref-for-grid-lanes-container①⑤"></a>

    <a id="ref-for-grid-axis①③"></a>

    <a id="ref-for-virtual-grid-item①"></a>

    Place hypothetical copies of each [virtual grid item](#virtual-grid-item) into the [grid axis](#grid-axis) tracks in every position that the item could potentially occupy, and run the [track sizing algorithm](https://www.w3.org/TR/css-grid-2/#algo-track-sizing) with those items. The resulting track sizes are the [grid lanes container’s](#grid-lanes-container) track sizes.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This optimization should give the same results as the track sizing description [above](#track-sizing); if not this is an error, please [report it to the CSSWG](https://github.com/w3c/csswg-drafts/issues).

## <a id="grid-lanes-track-placement"></a>4. <a id="masonry-track-placement"></a> Grid Lanes Item Placement

<a id="ref-for-grid-axis①④"></a>

<a id="ref-for-grid-placement-property"></a>

<a id="ref-for-automatic-grid-position⑥"></a>

In the [grid axis](#grid-axis), items can be <em>explicitly placed</em> into tracks and span them using the familiar [grid-placement properties](https://www.w3.org/TR/css-grid-2/#grid-placement-property)’ syntax. Auto-placement, however, uses the [§ 4.4 Grid Lanes Layout and Placement Algorithm](#grid-lanes-layout-algorithm), placing each item with an [automatic grid position](https://www.w3.org/TR/css-grid-2/#automatic-grid-position) into the “shortest” track available.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c504d9b3"></a> Here’s a grid lanes layout demonstrating explicitly placed and spanning items:
>
> ```text
> .container {
>   grid-template-columns: repeat(3, auto);
> }
> .container > :nth-child(2) {
>   /* auto-placed, but spanning. */
>   grid-column: span 2;
> }
> .container > :nth-child(3) {
>   /* manually placed */
>   grid-column: 3;
> }
> /* all other children are auto-placed */
> ```
>
> ![](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/example-span-and-manual.png)
>
> Rendering of the example above.

### <a id="grid-lanes-placement"></a>4.1.  Specifying Grid Axis Item Placement: the grid-column-\* and grid-row-\* properties

<a id="ref-for-grid-axis①⑤"></a>

<a id="ref-for-grid-layout⑨"></a>

The grid-column-\* and grid-row-\* properties (and their shorthands) apply in the [grid axis](#grid-axis) of the items and establish placement just as in regular [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout).

<a id="ref-for-propdef-flow-tolerance②"></a>

### <a id="placement-tolerance"></a>4.2. <a id="item-slack"></a> Placement Precision: the [flow-tolerance](#propdef-flow-tolerance) property

<strong>Table 3 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-flow-tolerance"></a>flow-tolerance

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-length-percentage"></a>

<a id="ref-for-comb-one②"></a>

normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) <a id="ref-for-comb-one③"></a>\| infinite

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

normal

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-lanes-container①⑥"></a>

[grid lanes containers](#grid-lanes-container)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-lanes-container①⑦"></a>

<a id="ref-for-content-box"></a>

<a id="ref-for-grid-axis①⑥"></a>

relative to the [grid-axis](#grid-axis) [content box](https://www.w3.org/TR/css-box-4/#content-box) size of the [grid lanes container](#grid-lanes-container)

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-length-percentage①"></a>

a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

as length

<a id="ref-for-grid-lanes-container①⑧"></a>

<a id="ref-for-grid-item②"></a>

<a id="ref-for-grid-track②"></a>

[Grid lanes containers](#grid-lanes-container) are filled by placing each [grid item](https://www.w3.org/TR/css-grid-2/#grid-item) in whichever [grid track](https://www.w3.org/TR/css-grid-2/#grid-track) is currently the least filled. When multiple tracks are tied for least-filled, placing the items in order looks good. But if tracks are only <em>very slightly</em> different heights, it can look strange to have them not fill in order, as the height differences aren’t perceived as <em>meaningfully</em> different.

<a id="ref-for-propdef-flow-tolerance③"></a>

The [flow-tolerance](#propdef-flow-tolerance) property specifies what the threshold is for considering tracks to be “the same height”, causing them to fill in order.

<a id="ref-for-typedef-length-percentage②"></a>

<a id="valdef-flow-tolerance-length-percentage"></a>[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)

<a id="ref-for-grid-lanes-container①⑨"></a>

Specifies the <a id="grid-lanes-tie-threshold"></a>tie threshold for the [grid lanes container](#grid-lanes-container). Placement positions are considered to be equally good (“tied”) if they are within the specified distance from the shortest position.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The initial value is a “small” distance (1em) that is probably appropriate to represent “close enough”.

<a id="valdef-flow-tolerance-normal"></a>normal

<a id="ref-for-used-value"></a>

<a id="ref-for-grid-lanes-layout⑧"></a>

Resolves to a [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of 1em in [grid lanes layout](#grid-lanes-layout) and a <a id="ref-for-used-value①"></a>used value of 0 in all other layout modes.

<a id="valdef-flow-tolerance-infinite"></a>infinite

<a id="ref-for-grid-lanes-tie-threshold"></a>

Specifies an infinite [tie threshold](#grid-lanes-tie-threshold). This makes items distribute themselves strictly in order, without considering the length of the tracks at all.

<a id="ref-for-stacking-axis⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value can result in consecutive items being placed in dramatically different positions in the [stacking axis](#stacking-axis), which can be confusing to readers. If the initial value (\`1em\`) is too small, consider a larger value (such as \`10em\` or \`50vh\`) instead of \`infinite\`.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ded661dc"></a> In the following example, when placing the 5th item, the fourth column is the shortest, but the first column is <em>almost</em> as short.
>
> ![An example grid lanes element with four columns. Each column is already partially filled to different heights, with the fourth column the shortest but the first column only slightly taller. Depending on the tolerance value, the next item to be placed can choose either the first or fourth column.](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/tolerance.png)
>
> With the default tolerance of 1em, both the first and fourth columns are considered to be "tied", and so the first is chosen.
>
> If, instead, the tolerance is set to 0px, then there is no tie; only the fourth column is a possible placement.

<a id="ref-for-flex-layout④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: We expect to apply this property to [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout) in the future also, see [discussions](https://github.com/w3c/csswg-drafts/issues/3071) on how that might work.

<a id="ref-for-valdef-item-pack-dense"></a>

### <a id="grid-lanes-dense-packing"></a>4.3.  Dense Placement: the [dense](#valdef-item-pack-dense) keyword

<a id="ref-for-grid-layout①⓪"></a>

<a id="ref-for-propdef-grid-auto-flow①"></a>

<a id="ref-for-valdef-item-pack-dense①"></a>

<a id="ref-for-grid-lanes-layout⑨"></a>

In [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout), [grid-auto-flow: dense](https://www.w3.org/TR/css-grid-2/#propdef-grid-auto-flow) allows backtracking during the [grid item placement algorithm](https://www.w3.org/TR/css-grid-2/#auto-placement-algo). The [dense](#valdef-item-pack-dense) keyword similarly allows backtracking during the [grid lanes placement algorithm](#grid-lanes-layout-algorithm). However, because item placement and sizing are intertwined in [grid lanes layout](#grid-lanes-layout), an item can only backtrack into a compatible empty slot if the total used size of that slot’s tracks matches the used size of the item’s normal-placement tracks.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This restriction avoids laying out the item more than once.

### <a id="grid-lanes-layout-algorithm"></a>4.4.  Grid Lanes Layout and Placement Algorithm

<a id="ref-for-grid-axis①⑦"></a>

For each of the tracks in the [grid axis](#grid-axis), keep a <a id="running-position"></a>running position initialized to zero. Maintain also a <a id="auto-placement-cursor"></a>auto-placement cursor, initially pointing to the first line.

<a id="ref-for-order-modified-document-order"></a>

For each item in [order-modified document order](https://www.w3.org/TR/css-display-4/#order-modified-document-order):

1.  <a id="ref-for-grid-axis①⑧"></a>

    <a id="ref-for-definite-grid-position②"></a>

    If the item has a [definite grid position](https://www.w3.org/TR/css-grid-2/#definite-grid-position) in the [grid axis](#grid-axis), use that placement.

    > <strong data-conversion-semantic="issue">Issue</strong>
    >
    > <a id="issue-9c647ef6"></a> Should this also update the placement cursor?

    <a id="ref-for-grid-axis①⑨"></a>

    Otherwise, resolve its [grid axis](#grid-axis) placement using these substeps:

    1.  <a id="ref-for-running-position"></a>

        <a id="ref-for-implicit-grid①"></a>

        <a id="ref-for-grid-axis②⓪"></a>

        Starting at the first [grid axis](#grid-axis) line in the [implicit grid](https://www.w3.org/TR/css-grid-2/#implicit-grid), find the largest [running position](#running-position) of the <a id="ref-for-grid-axis②①"></a>grid axis tracks that the item would span if it were placed at this line, and call this position <var>max&#x5F;pos</var>.

    2.  Repeat the previous step for each successive line number until the item would no longer fit inside the grid.

    3.  <a id="ref-for-grid-lanes-tie-threshold①"></a>

        Let <var>possible lines</var> be the line that resulted in the smallest <var>max&#x5F;pos</var>, and all lines that result in a <var>max&#x5F;pos</var> within the [tie threshold](#grid-lanes-tie-threshold) of this <var>max&#x5F;pos</var>.

    4.  <a id="ref-for-grid-axis②②"></a>

        <a id="ref-for-auto-placement-cursor"></a>

        Choose the first line in <var>possible lines</var> greater than or equal to the [auto-placement cursor](#auto-placement-cursor) as the item’s position in the [grid axis](#grid-axis); or if there are none such, choose the first one.

    5.  <a id="ref-for-auto-placement-cursor①"></a>

        Update the [auto-placement cursor](#auto-placement-cursor) to point to item’s last line.

2.  <a id="ref-for-running-position①"></a>

    <a id="ref-for-grid-axis②③"></a>

    Place the item in its [grid axis](#grid-axis) tracks at the maximum of the [running position](#running-position)s of the tracks it spans.

3.  <a id="ref-for-propdef-grid-gap"></a>

    <a id="ref-for-outer-size"></a>

    <a id="ref-for-grid-axis②④"></a>

    <a id="ref-for-running-position②"></a>

    Calculate the size of the item’s [containing block](#containing-block) and then layout the item. Set the [running position](#running-position) of the spanned [grid axis](#grid-axis) tracks to <code><var>max&#x5F;pos</var> + <a href="https://www.w3.org/TR/css-sizing-3/#outer-size">outer size</a> + <a href="https://www.w3.org/TR/css-align-3/#propdef-grid-gap">grid-gap</a></code>.

    <a id="ref-for-outer-size①"></a>

    For this purpose, the [outer sizes](https://www.w3.org/TR/css-sizing-3/#outer-size) of the box are floored at zero.

    <a id="ref-for-running-position③"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: That is, if the box has sufficiently large negative margins, it won’t somehow count as a negative <em>size</em> here. The [running position](#running-position) of a track never decreases, so a negative-sized box won’t cause future normally-sized items placed in the track to overlap previous items.

4.  <a id="ref-for-running-position④"></a>

    <a id="ref-for-auto-placement-cursor②"></a>

    <a id="ref-for-start"></a>

    <a id="ref-for-grid-lanes-tie-threshold②"></a>

    <a id="ref-for-valdef-item-pack-dense②"></a>

    <a id="ref-for-grid-lanes-container②⓪"></a>

    If the [grid lanes container](#grid-lanes-container) uses [dense](#valdef-item-pack-dense) packing, and there exists skipped spaces in the layout (e.g. due to spanning items) into which the item, as it is sized now, could have fit if it were placed earlier, and where the spanned tracks have the same total used size as the tracks into which it is currently placed, then instead place it into the highest such space. If there are multiple valid spaces within the [tie threshold](#grid-lanes-tie-threshold) of the highest space, place it in the [start](https://www.w3.org/TR/css-writing-modes-4/#start)-most of them. Rewind the [auto-placement cursor](#auto-placement-cursor) and the [running position](#running-position) to their values before this item’s placement.

    <a id="ref-for-propdef-position"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Items that <em>visually</em> intrude into preceding empty spaces (via negative margins, [position: relative](https://www.w3.org/TR/css-position-3/#propdef-position), transforms, etc.), do not affect the size of those empty spaces. Later items can get placed in those spaces and visually overlap these previous items.

    <a id="ref-for-auto-placement-cursor③"></a>

    <a id="ref-for-valdef-item-pack-dense③"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Dense packing both ignores the [auto-placement cursor](#auto-placement-cursor) when backfilling, and does not update it after placement. If there aren’t any acceptable placement gaps to backfill, though, it places items exactly as when [dense](#valdef-item-pack-dense) were not specified.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This algorithm chooses the track that would result in the item being placed as highly as possible. If there are ties, it chooses the earliest such track, <em>after</em> the most recently placed item if possible (ensuring that it always “moves forward” even in the presence of ties).

#### <a id="containing-block"></a>4.4.1.  Containing Block

<a id="ref-for-containing-block"></a>

<a id="ref-for-grid-item③"></a>

<a id="ref-for-grid-lanes-layout①⓪"></a>

<a id="ref-for-grid-area"></a>

<a id="ref-for-grid-axis②⑤"></a>

<a id="ref-for-grid-container①⓪"></a>

<a id="ref-for-content-box①"></a>

<a id="ref-for-stacking-axis①⓪"></a>

The [containing block](https://www.w3.org/TR/css-display-4/#containing-block) for a [grid item](https://www.w3.org/TR/css-grid-2/#grid-item) participating in [grid lanes layout](#grid-lanes-layout) is formed by its [grid area](https://www.w3.org/TR/css-grid-2/#grid-area) in the [grid axis](#grid-axis) and the [grid container](https://www.w3.org/TR/css-grid-2/#grid-container)’s [content box](https://www.w3.org/TR/css-box-4/#content-box) in the [stacking axis](#stacking-axis).

#### <a id="rtl-example"></a>4.4.2.  Placement and Writing Modes

<a id="ref-for-grid-layout①①"></a>

<a id="ref-for-writing-mode"></a>

<a id="ref-for-propdef-direction"></a>

<a id="ref-for-grid-axis②⑥"></a>

<a id="ref-for-stacking-axis①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Like all of [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout), grid lanes layout and placement is sensitive to the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode). For example, for [direction: rtl](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), items are placed right-to-left rather than left-to-right, whether the inline axis is a [grid axis](#grid-axis) or a [stacking axis](#stacking-axis).

<a id="ref-for-propdef-direction①"></a>

<a id="ref-for-grid-axis②⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5f5333ca"></a> Here’s a simple [example](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/examples/rtl-grid-axis.html) using [direction: rtl](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) in the [grid axis](#grid-axis):
>
> ```css
> <style>
>   .grid {
>     display: inline-grid-lanes;
>     direction: rtl;
>     grid-template-columns: repeat(4, 2ch);
>     border: 1px solid;
>   }
> 
>   item { background: silver }
>   item:nth-child(2n+1) {
>     background: pink;
>     height: 4em;
>   }
>   </style>
> ```
>
> ```html
> <div class="grid">
>   <item>1</item>
>   <item style="grid-column:span 2">2</item>
>   <item>3</item>
>   <item>4</item>
> </div>
> ```
>
> ![](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/rtl-grid-axis.png)
>
> <a id="ref-for-propdef-direction②"></a>
>
> Rendering of the [direction: rtl](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) example above.

<a id="ref-for-propdef-direction③"></a>

<a id="ref-for-stacking-axis①②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7bed5830"></a> Here’s a simple [example](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/examples/rtl-masonry-axis.html) using [direction: rtl](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) in the [stacking axis](#stacking-axis):
>
> ```css
> <style>
> .grid {
>   display: inline-grid-lanes;
>   direction: rtl;
>   width: 10ch;
>   column-gap: 1ch;
>   grid-template-rows: repeat(4, 2em);
>   border: 1px solid;
> }
> 
> item { background: silver }
> item:nth-child(2n+1) {
>   background: pink;
>   width: 4ch;
> }
> </style>
> ```
>
> ```html
> <div class="grid">
>   <item>1</item>
>   <item style="grid-row:span 2">2</item>
>   <item>3</item>
>   <item>4</item>
> </div>
> ```
>
> ![](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/rtl-masonry-axis.png)
>
> <a id="ref-for-propdef-direction④"></a>
>
> Rendering of the [direction: rtl](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) example above.

## <a id="intrinsic-sizes"></a>5.  Sizing Grid Containers

<a id="ref-for-grid-container①①"></a>

<a id="ref-for-stacking-axis①③"></a>

<a id="ref-for-max-content"></a>

<a id="ref-for-min-content"></a>

<a id="ref-for-stacking-range"></a>

<a id="ref-for-grid-lanes-container②①"></a>

<a id="ref-for-max-content-constraint"></a>

<a id="ref-for-min-content-constraint"></a>

<a id="ref-for-outer-edge"></a>

[Sizing Grid Containers](https://www.w3.org/TR/css-grid-2/#intrinsic-sizes) works the same as for regular [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container) but with the following replacement for the [stacking axis](#stacking-axis): The [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content) ([min-content size](https://www.w3.org/TR/css-sizing-3/#min-content)) of a <a id="ref-for-grid-container①②"></a>grid container in the <a id="ref-for-stacking-axis①④"></a>stacking axis is the size of the [stacking range](#stacking-range) when the [grid lanes container](#grid-lanes-container) is sized under a [max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint) ([min-content constraint](https://www.w3.org/TR/css-sizing-3/#min-content-constraint)) in that axis. The <a id="stacking-range"></a>stacking range is the range between the startmost [outer edge](https://www.w3.org/TR/css-box-4/#outer-edge) among the first items of each track and the endmost <a id="ref-for-outer-edge①"></a>outer edge among the last items of each track.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-51eed6fc"></a> Here’s a simple [example](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/examples/grid-intrinsic-sizing-example-1.html):
>
> ```css
> <style>
> .grid {
>   display: inline-grid-lanes;
>   grid-template-columns: 50px 100px auto;
>   grid-gap: 10px;
>   border: 1px solid;
> }
> item { background: silver; margin: 5px; }
> </style>
> ```
>
> ```html
> <div class="grid">
>   <item style="border:10px solid">1</item>
>   <item>2</item>
>   <item>3</item>
>   <item style="height:50px">4</item>
>   <item>5</item>
>   <item>6</item>
> </div>
> ```
>
> ![](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/grid-intrinsic-sizing-example-1.png)
>
> <a id="ref-for-grid-container①③"></a>
>
> Rendering of the [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) intrinsic sizing example above.

## <a id="alignment"></a>6.  Alignment and Spacing

<a id="ref-for-propdef-row-gap"></a>

<a id="ref-for-propdef-column-gap"></a>

<a id="ref-for-propdef-gap"></a>

### <a id="gutters"></a>6.1.  Gutters: the [row-gap](https://www.w3.org/TR/css-align-3/#propdef-row-gap), [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap), and [gap](https://www.w3.org/TR/css-align-3/#propdef-gap) properties

<a id="ref-for-propdef-row-gap①"></a>

<a id="ref-for-propdef-column-gap①"></a>

<a id="ref-for-propdef-gap①"></a>

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-grid-axis②⑧"></a>

<a id="ref-for-grid-container①④"></a>

<a id="ref-for-stacking-axis①⑤"></a>

[Gutters](https://www.w3.org/TR/css-grid-2/#gutters) are supported in both axes using the [row-gap](https://www.w3.org/TR/css-align-3/#propdef-row-gap) and [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap) properties (and their [gap](https://www.w3.org/TR/css-align-3/#propdef-gap) [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property)). In the [grid axis](#grid-axis) they work the same as in a regular [grid container](https://www.w3.org/TR/css-grid-2/#grid-container); see [CSS Grid Layout 2 §  11. Alignment and Spacing](https://www.w3.org/TR/css-grid-2/#alignment). In the [stacking axis](#stacking-axis), the gap is applied between the margin boxes of each pair of adjacent items. Margins do not collapse in either axis.

<a id="ref-for-propdef-align-content"></a>

<a id="ref-for-propdef-justify-content"></a>

<a id="ref-for-propdef-align-self"></a>

<a id="ref-for-propdef-justify-self"></a>

<a id="ref-for-propdef-align-items"></a>

<a id="ref-for-propdef-justify-items"></a>

### <a id="grid-axis-alignment"></a>6.2.  Grid-axis Alignment: the [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content)/[justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content), [align-self](https://www.w3.org/TR/css-align-3/#propdef-align-self)/[justify-self](https://www.w3.org/TR/css-align-3/#propdef-justify-self), and [align-items](https://www.w3.org/TR/css-align-3/#propdef-align-items)/[justify-items](https://www.w3.org/TR/css-align-3/#propdef-justify-items) properties

<a id="ref-for-grid-axis②⑨"></a>

<a id="ref-for-box-alignment-properties"></a>

<a id="ref-for-grid-container①⑤"></a>

In the [grid axis](#grid-axis), the [box alignment properties](https://www.w3.org/TR/css-align-3/#box-alignment-properties) work the same as in a regular [grid container](https://www.w3.org/TR/css-grid-2/#grid-container). See [CSS Grid Layout 2 §  11. Alignment and Spacing](https://www.w3.org/TR/css-grid-2/#alignment) and [CSS Box Alignment Level 3](#biblio-css-align-3).

<a id="ref-for-propdef-align-content①"></a>

<a id="ref-for-propdef-justify-content①"></a>

### <a id="stacking-content-alignment"></a>6.3.  Stacking-axis Content Distribution: the [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content)/[justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content) properties

<a id="ref-for-stacking-axis①⑥"></a>

<a id="ref-for-alignment-subject"></a>

<a id="ref-for-stacking-range①"></a>

In the [stacking axis](#stacking-axis), [content-distribution](https://www.w3.org/TR/css-align-3/#content-distribution) is applied to the content as a whole, similarly to how it behaves in block containers. More specifically, the [alignment subject](https://www.w3.org/TR/css-align-3/#alignment-subject) is the [stacking range](#stacking-range).

![](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/masonry-box.png)

<a id="ref-for-propdef-align-content②"></a>

The extent of the alignment subject is indicated by the dashed border, while the alignment container is the whole container (indicated by the height of the column dividers). It defaults to start-aligning, but depending on (in this case) [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content), the items can move down, as a block, to center- or end-align vertically.

<a id="ref-for-alignment-subject①"></a>

<a id="ref-for-stacking-axis①⑦"></a>

<a id="ref-for-propdef-align-content③"></a>

<a id="ref-for-propdef-justify-content②"></a>

<a id="ref-for-valdef-self-position-start"></a>

<a id="ref-for-valdef-self-position-center"></a>

<a id="ref-for-valdef-self-position-end"></a>

<a id="ref-for-baseline-alignment①"></a>

<a id="ref-for-valdef-justify-content-normal"></a>

<a id="ref-for-valdef-align-content-stretch"></a>

<a id="ref-for-distributed-alignment"></a>

<a id="ref-for-fallback-alignment"></a>

<a id="ref-for-grid-item④"></a>

<a id="ref-for-grid-container①⑥"></a>

<a id="ref-for-content-box②"></a>

<a id="ref-for-stacking-range②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: There is only ever one [alignment subject](https://www.w3.org/TR/css-align-3/#alignment-subject) for these properties in the [stacking axis](#stacking-axis), so the unique [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) / [justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content) values boil down to [start](https://www.w3.org/TR/css-align-3/#valdef-self-position-start), [center](https://www.w3.org/TR/css-align-3/#valdef-self-position-center), [end](https://www.w3.org/TR/css-align-3/#valdef-self-position-end), and [baseline alignment](https://www.w3.org/TR/css-align-3/#baseline-alignment). (The behavior of [normal](https://www.w3.org/TR/css-align-3/#valdef-justify-content-normal) and [stretch](https://www.w3.org/TR/css-align-3/#valdef-align-content-stretch) is identical to <a id="ref-for-valdef-self-position-start①"></a>start, and the [distributed alignment](https://www.w3.org/TR/css-align-3/#distributed-alignment) values behave as their [fallback alignments](https://www.w3.org/TR/css-align-3/#fallback-alignment).) If the [grid items](https://www.w3.org/TR/css-grid-2/#grid-item) overflow the [grid container](https://www.w3.org/TR/css-grid-2/#grid-container)’s [content box](https://www.w3.org/TR/css-box-4/#content-box) in the <a id="ref-for-stacking-axis①⑧"></a>stacking axis, then the [stacking range](#stacking-range) will be larger than the <a id="ref-for-grid-container①⑦"></a>grid container’s <a id="ref-for-content-box③"></a>content box.

<a id="ref-for-propdef-align-self①"></a>

<a id="ref-for-propdef-justify-self①"></a>

<a id="ref-for-propdef-align-items①"></a>

<a id="ref-for-propdef-justify-items①"></a>

### <a id="stacking-self-alignment"></a>6.4.  Stacking-axis Self Alignment: the [align-self](https://www.w3.org/TR/css-align-3/#propdef-align-self)/[justify-self](https://www.w3.org/TR/css-align-3/#propdef-justify-self) and [align-items](https://www.w3.org/TR/css-align-3/#propdef-align-items)/[justify-items](https://www.w3.org/TR/css-align-3/#propdef-justify-items) properties

<a id="ref-for-stacking-axis①⑨"></a>

<a id="ref-for-self-alignment-properties"></a>

<a id="ref-for-grid-track③"></a>

<a id="ref-for-alignment-subject②"></a>

<a id="ref-for-margin-box"></a>

<a id="ref-for-alignment-container"></a>

In the [stacking axis](#stacking-axis), the [self-alignment properties](https://www.w3.org/TR/css-align-3/#self-alignment-properties) only apply to items that are adjacent to a gap in the layout, i.e. placed in their [grid track](https://www.w3.org/TR/css-grid-2/#grid-track)(s) either immediately before a spanning item or as the last item. The [alignment subject](https://www.w3.org/TR/css-align-3/#alignment-subject) is the item’s [margin box](https://www.w3.org/TR/css-box-4/#margin-box), and the [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container) is that box plus the adjacent gap.

![](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/align-self.png)

<a id="ref-for-propdef-align-items②"></a>

<a id="ref-for-alignment-container①"></a>

The three highlighted items are the only ones with "gaps" following them, so they’re the only ones that will respond to [align-items](https://www.w3.org/TR/css-align-3/#propdef-align-items). Their [alignment containers](https://www.w3.org/TR/css-align-3/#alignment-container) are indicated by the dashed areas.

<a id="ref-for-self-alignment-properties①"></a>

<a id="ref-for-stacking-axis②⓪"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-7e3f9302"></a> Is this a reasonable definition for how the [self-alignment properties](https://www.w3.org/TR/css-align-3/#self-alignment-properties) should work in the [stacking axis](#stacking-axis)? [\[Issue \#10275\]](https://github.com/w3c/csswg-drafts/issues/10275)

### <a id="grid-lanes-baseline-alignment"></a>6.5. <a id="masonry-baseline-alignment"></a> Baseline Alignment

<a id="ref-for-baseline-alignment②"></a>

<a id="ref-for-grid-axis③⓪"></a>

<a id="ref-for-grid-container①⑧"></a>

Item [baseline alignment](https://www.w3.org/TR/css-align-3/#baseline-alignment) inside the [grid axis](#grid-axis) tracks works as usual for a regular [grid container](https://www.w3.org/TR/css-grid-2/#grid-container), and the <a id="ref-for-grid-container①⑨"></a>grid container’s baseline is determined the same as for a regular <a id="ref-for-grid-container②⓪"></a>grid container in that axis.

<a id="ref-for-baseline-alignment③"></a>

<a id="ref-for-stacking-axis②①"></a>

<a id="ref-for-grid-container②①"></a>

<a id="ref-for-alignment-baseline"></a>

<a id="ref-for-grid-item⑤"></a>

[Baseline alignment](https://www.w3.org/TR/css-align-3/#baseline-alignment) is not supported in the [stacking axis](#stacking-axis). The first baseline set of the [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) in this axis is generated from the highest [alignment baseline](https://www.w3.org/TR/css-align-3/#alignment-baseline) among the [grid items](https://www.w3.org/TR/css-grid-2/#grid-item) placed first in each track, and the last baseline set from the lowest <a id="ref-for-alignment-baseline①"></a>alignment baseline among the <a id="ref-for-grid-item⑥"></a>grid items placed last in each track.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-4a2d589c"></a> We could support baseline alignment in the first row. Do we want to?

## <a id="pagination"></a>7.  Fragmentation

### <a id="stacking-axis-pagination"></a>7.1. <a id="masonry-axis-pagination"></a> Fragmentation in the stacking axis

<a id="ref-for-grid-axis③①"></a>

<a id="ref-for-stacking-axis②②"></a>

<a id="ref-for-grid-item⑦"></a>

<a id="ref-for-forced-break"></a>

<a id="ref-for-running-position⑤"></a>

<a id="ref-for-fragmentainer"></a>

<a id="ref-for-grid-container②②"></a>

Each [grid axis](#grid-axis) track is fragmented independently in the [stacking axis](#stacking-axis). If a [grid item](https://www.w3.org/TR/css-grid-2/#grid-item) is fragmented, or has a [forced break](https://www.w3.org/TR/css-break-4/#forced-break) before/after it, then the [running position](#running-position) for the tracks that it spans in the <a id="ref-for-grid-axis③②"></a>grid axis are set to the size of the [fragmentainer](https://www.w3.org/TR/css-break-4/#fragmentainer) so that no further items will be placed in those tracks. An item that is split into multiple fragments retains its placement in the <a id="ref-for-grid-axis③③"></a>grid axis for all its fragments. A grid item that is pushed, however, is placed again by the next [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) fragment. Placement continues until all items are placed or pushed to a new fragment.

<a id="ref-for-stacking-axis②③"></a>

<a id="ref-for-block-axis③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c4746f2e"></a> Here’s an [example](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/examples/fragmentation-block-axis-example.html) illustrating fragmentation of grid lanes layout with [stacking](#stacking-axis) in its [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis). It renders like this:
>
> [Embedded video resource](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/fragmentation-block-axis-example.mp4)
>
> <a id="ref-for-block-axis④"></a>
>
> <a id="ref-for-grid-lanes-layout①①"></a>
>
> Visualization of fragmentation in a [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) [grid lanes layout](#grid-lanes-layout).

### <a id="grid-axis-pagination"></a>7.2.  Fragmentation in the Grid Axis

<a id="ref-for-grid-axis③④"></a>

<a id="ref-for-grid-lanes-container②②"></a>

<a id="ref-for-grid-container②③"></a>

Fragmentation in the [grid axis](#grid-axis) of a [grid lanes container](#grid-lanes-container) is also supported. In this case the fragmentation behaves more like in a regular [grid container](https://www.w3.org/TR/css-grid-2/#grid-container); however, there’s a separate step to determine which <a id="ref-for-grid-axis③⑤"></a>grid-axis track each item is placed into, before fragmentation occurs.

<a id="ref-for-stacking-axis②④"></a>

<a id="ref-for-inline-axis④"></a>

<a id="ref-for-grid-axis③⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-dab6ab26"></a> Here’s an [example](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/examples/fragmentation-inline-axis-example.html) illustrating fragmentation of grid lanes layout with [stacking](#stacking-axis) in its [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis). In this case the breaks occurs between the [grid-axis](#grid-axis) rows. It renders like this:
>
> [Embedded video resource](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/fragmentation-inline-axis-example.mp4)
>
> <a id="ref-for-block-axis⑤"></a>
>
> <a id="ref-for-inline-axis⑤"></a>
>
> <a id="ref-for-grid-lanes-layout①②"></a>
>
> Visualization of fragmentation in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) with [inline-axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) [grid lanes layout](#grid-lanes-layout).

## <a id="abspos"></a>8.  Absolute Positioning

<a id="ref-for-grid-lanes-container②③"></a>

<a id="ref-for-grid-container②④"></a>

<a id="ref-for-stacking-axis②⑤"></a>

[Grid-aligned absolute-positioned descendants](https://www.w3.org/TR/css-grid-1/#abspos-items) are supported in [grid lanes containers](#grid-lanes-container) just as for regular [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container); however, in the [stacking axis](#stacking-axis) there exist only two lines (in addition to the auto lines) for placement:

- <a id="ref-for-stacking-range③"></a>

  line 1 (line -2) corresponds to the start edge of the [stacking range](#stacking-range)

- <a id="ref-for-stacking-range④"></a>

  line 2 (line -1) corresponds to the end edge of the [stacking range](#stacking-range)

<a id="ref-for-stacking-axis②⑥"></a>

<a id="ref-for-running-position⑥"></a>

<a id="ref-for-grid-axis③⑦"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-4007c2e6"></a> It might be useful to define a static position in the [stacking axis](#stacking-axis). Maybe it could defined as the max (or min?) current [running position](#running-position) of the [grid-axis](#grid-axis) tracks at that point? Or the end of the item before it?

## <a id="graceful-degradation"></a>9.  Graceful Degradation

<a id="ref-for-grid-layout①②"></a>

<a id="ref-for-grid-lanes-layout①③"></a>

Typically, a grid lanes design can be expected to degrade quite nicely in a UA that supports [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout) but not [grid lanes layout](#grid-lanes-layout).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c1a7752e"></a>
>
> Here’s an [example](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/examples/graceful-degradation-example.html) to illustrate this.
>
> ```css
> display: grid;
> display: grid-lanes; /* ignored in UAs that don't support grid lanes layout */
> grid-template-columns: 150px 100px 50px;
> ```
>
> <a id="ref-for-block-axis⑥"></a>
>
> <a id="ref-for-grid-lanes-layout①④"></a>
>
> This creates a layout with three columns, but will have "more gaps" in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) if the UA doesn’t support [grid lanes layout](#grid-lanes-layout). Here’s what it looks like with grid lanes support for comparison:
>
> [Embedded video resource](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/images/graceful-degradation-example.mp4)
>
> Rendering of the example in a UA with grid lanes support.

## <a id="flow-control"></a> Appendix A: Generic Layout Item Flow Controls: the item-\* properties

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-888c395d"></a> This section is likely to move to another spec, such as [\[css-display-4\]](#biblio-css-display-4), since it affects multiple display types. It is also still under discussion as to whether this is a good idea.

<a id="ref-for-propdef-flex-flow"></a>

<a id="ref-for-propdef-grid-auto-flow②"></a>

Multiple layout modes in CSS place their children as atomic "items" organized into rows and/or colummns in their container, and allow the author to configure their ordering and placement. The item-\* properties provide generic controls for these ordering and placement options, encapsulating the layout-specific [flex-flow](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-flow) and [grid-auto-flow](https://www.w3.org/TR/css-grid-2/#propdef-grid-auto-flow) properties.

<strong>Table 4 — structured row/cell transcription</strong>

Overview of Item Flow Controls

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Flow-oriented Proposal

<strong>Column 2 (header cell):</strong>

Track-oriented Proposal

<strong>Column 3 (header cell):</strong>

Value Space

<strong>Column 4 (header cell):</strong>

Description

<strong>Column 5 (header cell):</strong>

Existing flex property

<strong>Column 6 (header cell):</strong>

Existing grid property

<strong>Row 2</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-propdef-item-direction"></a>

[item-direction](#propdef-item-direction)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-item-track"></a>

[item-track](#propdef-item-track)

<strong>Column 3 (data cell):</strong>

auto \| row \| column \| row-reverse \| column-reverse

<strong>Column 4 (data cell):</strong>

Controls whether items are placed into rows or columns, and whether within those tracks they are ordered start-to-end or end-to-start.

<strong>Column 5 (data cell):</strong>

flex-direction

<strong>Column 6 (data cell):</strong>

grid-auto-flow

<strong>Row 3</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-propdef-item-wrap"></a>

[item-wrap](#propdef-item-wrap)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-item-cross"></a>

[item-cross](#propdef-item-cross)

<strong>Column 3 (data cell):</strong>

\[ auto \| nowrap \| wrap \] \|\| \[ normal \| reverse \] \| wrap-reverse

<strong>Column 4 (data cell):</strong>

Controls whether items are wrapped in the axis opposite to that controled by the row/column property, and if so if they are placed in start-to-end or end-to-start order.

<strong>Column 5 (data cell):</strong>

flex-wrap

<strong>Column 6 (data cell):</strong>

<a id="ref-for-propdef-grid-auto-flow③"></a>

Introduced into [grid-auto-flow](https://www.w3.org/TR/css-grid-2/#propdef-grid-auto-flow) in this level.

<strong>Row 4</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-propdef-item-pack"></a>

[item-pack](#propdef-item-pack)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-item-pack①"></a>

[item-pack](#propdef-item-pack)

<strong>Column 3 (data cell):</strong>

normal \| dense \|\| balance

<strong>Column 4 (data cell):</strong>

Configures how items are packed into their tracks.

<strong>Column 5 (data cell):</strong>

<strong>Column 6 (data cell):</strong>

<a id="ref-for-propdef-grid-auto-flow④"></a>

[grid-auto-flow](https://www.w3.org/TR/css-grid-2/#propdef-grid-auto-flow)

<strong>Row 5</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-propdef-flow-tolerance④"></a>

[flow-tolerance](#propdef-flow-tolerance)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-flow-tolerance⑤"></a>

[flow-tolerance](#propdef-flow-tolerance)

<strong>Column 3 (data cell):</strong>

<a id="ref-for-typedef-length-percentage③"></a>

normal \| [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) \| infinite

<strong>Column 4 (data cell):</strong>

Defines a layout-specific amount of “slack” in placement decisions.

<strong>Column 5 (data cell):</strong>

<strong>Column 6 (data cell):</strong>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-c971c9c1"></a> The CSSWG is still figuring out how these properties should be named and fit together. [\[Issue \#11480\]](https://github.com/w3c/csswg-drafts/issues/11480)

<a id="ref-for-propdef-item-track①"></a>

<a id="ref-for-propdef-item-direction①"></a>

### <a id="item-primary-axis"></a> Item Flow Axis: [item-track](#propdef-item-track)/[item-direction](#propdef-item-direction)

<strong>Table 5 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-item-direction"></a>item-direction, <a id="propdef-item-track"></a>item-track

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one④"></a>

auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) row <a id="ref-for-comb-one⑤"></a>\| column <a id="ref-for-comb-one⑥"></a>\| row-reverse <a id="ref-for-comb-one⑦"></a>\| column-reverse

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-lanes-container②④"></a>

<a id="ref-for-grid-container②⑤"></a>

<a id="ref-for-flex-container"></a>

[flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container), [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container), [grid lanes containers](#grid-lanes-container)

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

as specified

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

This property controls whether items are placed as rows or columns, and whether within those tracks they are ordered start-to-end or end-to-start.

<a id="ref-for-flex-layout⑤"></a>

<a id="ref-for-grid-layout①③"></a>

<a id="ref-for-grid-lanes-layout①⑤"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f870e6cb"></a> There two open debates on this property: a) what should it be called and b) does it describe the primary direction of placement, or the orientation of the tracks into which items are placed; in other words, is the <a id="primary-axis"></a>primary axis defined by this property the <a id="primary-placement-axis"></a>primary placement axis or the <a id="primary-track-axis"></a>primary track axis. These are identical for [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout) and [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout), but differ for [grid lanes layout](#grid-lanes-layout) whose primary placement direction is across its tracks. See [latest discussion](https://github.com/w3c/csswg-drafts/issues/12803). [\[Issue \#11480\]](https://github.com/w3c/csswg-drafts/issues/11480)

<a id="valdef-item-direction-auto"></a>auto  
<a id="ref-for-valdef-item-direction-row"></a>

<a id="ref-for-valdef-item-direction-column"></a>

Computes to either [row](#valdef-item-direction-row) or [column](#valdef-item-direction-column) depending on the layout mode:

- <a id="ref-for-flex-container①"></a>

  <a id="ref-for-grid-container②⑥"></a>

  <a id="ref-for-valdef-item-direction-row①"></a>

  On [flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container) and [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container), computes to [row](#valdef-item-direction-row).

- <a id="ref-for-grid-lanes-container②⑤"></a>

  <a id="ref-for-propdef-grid-template-rows①"></a>

  <a id="ref-for-valdef-grid-template-rows-none②"></a>

  <a id="ref-for-propdef-grid-template-columns②"></a>

  On [grid lanes containers](#grid-lanes-container), if [grid-template-rows](https://www.w3.org/TR/css-grid-2/#propdef-grid-template-rows) is not [none](https://www.w3.org/TR/css-grid-2/#valdef-grid-template-rows-none) and [grid-template-columns](https://www.w3.org/TR/css-grid-2/#propdef-grid-template-columns) is <a id="ref-for-valdef-grid-template-rows-none③"></a>none, computes to the value representing row tracks; otherwise computes to the value representing column tracks.

<a id="valdef-item-direction-row"></a>row  
<a id="ref-for-inline-axis⑥"></a>

Track-oriented Option Represents placement into rows, i.e. tracks or lines parallel to the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis). Items fill those rows in start-to-end order.

<a id="ref-for-inline-axis⑦"></a>

<a id="ref-for-flex-layout⑥"></a>

<a id="ref-for-grid-lanes-layout①⑥"></a>

Flow-oriented Option Represents row-primary item placement, i.e. placing items start-to-end in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis), producing flex row lines in [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout), and column grid tracks in [grid lanes layout](#grid-lanes-layout).

<a id="valdef-item-direction-column"></a>column  
<a id="ref-for-block-axis⑦"></a>

Track-oriented Option Represents placement into columns, i.e. tracks or lines parallel to the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis). Items fill those columns in start-to-end order.

<a id="ref-for-block-axis⑧"></a>

<a id="ref-for-flex-layout⑦"></a>

<a id="ref-for-grid-lanes-layout①⑦"></a>

Flow-oriented Option Represents column-primary item placement, i.e. placing items start-to-end in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis), producing flex column lines in [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout), and row grid tracks in [grid lanes layout](#grid-lanes-layout).

<a id="valdef-item-direction-row-reverse"></a>row-reverse  
<a id="ref-for-valdef-item-direction-row②"></a>

Same as [row](#valdef-item-direction-row), but using end-to-start placement order.

<a id="valdef-item-direction-column-reverse"></a>column-reverse  
<a id="ref-for-valdef-item-direction-column①"></a>

Same as [column](#valdef-item-direction-column), but using end-to-start placement order.

<a id="ref-for-propdef-item-cross①"></a>

<a id="ref-for-propdef-item-wrap①"></a>

### <a id="item-secondary-axis"></a> Item Cross Axis Placement Mode: [item-cross](#propdef-item-cross)/[item-wrap](#propdef-item-wrap)

<strong>Table 6 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-item-wrap"></a>item-wrap, <a id="propdef-item-cross"></a>item-cross

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-any"></a>

<a id="ref-for-comb-one⑧"></a>

\[ auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) nowrap <a id="ref-for-comb-one⑨"></a>\| wrap \] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) \[ normal <a id="ref-for-comb-one①⓪"></a>\| reverse \] <a id="ref-for-comb-one①①"></a>\| wrap-reverse

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-lanes-container②⑥"></a>

<a id="ref-for-grid-container②⑦"></a>

<a id="ref-for-flex-container②"></a>

[flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container), [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container), [grid lanes containers](#grid-lanes-container)

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

as specified

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

<a id="ref-for-primary-axis"></a>

Controls placement in the axis opposite to the [primary axis](#primary-axis).

<a id="valdef-item-wrap-auto"></a>auto  
<a id="ref-for-valdef-item-wrap-nowrap"></a>

<a id="ref-for-flex-container③"></a>

<a id="ref-for-valdef-item-wrap-wrap"></a>

Computes to [nowrap](#valdef-item-wrap-nowrap) on [flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container), and [wrap](#valdef-item-wrap-wrap) on everything else.

<a id="valdef-item-wrap-nowrap"></a>nowrap  
<a id="ref-for-primary-placement-axis"></a>

<a id="ref-for-flex-layout⑧"></a>

<a id="ref-for-single-line-flex-container"></a>

<a id="ref-for-grid-layout①④"></a>

<a id="ref-for-implicit-grid-track"></a>

Items are placed in the [primary placement axis](#primary-placement-axis) forever, even if they run out of room. In [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout) this creates a [single-line flex container](https://www.w3.org/TR/css-flexbox-1/#single-line-flex-container); in [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout) this creates [implicit tracks](https://www.w3.org/TR/css-grid-2/#implicit-grid-track) in the <a id="ref-for-primary-placement-axis①"></a>primary placement axis as necessary.

<a id="valdef-item-wrap-wrap"></a>wrap  
<a id="ref-for-primary-placement-axis②"></a>

<a id="ref-for-flex-layout⑨"></a>

<a id="ref-for-multi-line-flex-container"></a>

<a id="ref-for-grid-layout①⑤"></a>

Items wrap when the [primary placement axis](#primary-placement-axis) runs out of space. In [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout) this creates a [multi-line flex container](https://www.w3.org/TR/css-flexbox-1/#multi-line-flex-container); in [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout) auto-placement algorithm moves to the next row/column when it runs out of explicit tracks in the <a id="ref-for-primary-placement-axis③"></a>primary placement axis.

<a id="valdef-item-wrap-normal"></a>normal  
<a id="ref-for-primary-track-axis"></a>

Items are placed in start-to-end order in the axis opposite to the [primary track axis](#primary-track-axis).

<a id="ref-for-flex-layout①⓪"></a>

<a id="ref-for-grid-layout①⑥"></a>

In [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout) and [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout), this controls the direction that new tracks (flex lines or grid tracks) are placed in.

<a id="ref-for-grid-lanes-layout①⑧"></a>

In [grid lanes layout](#grid-lanes-layout), for track-oriented syntax this controls which track is selected when several are tied for equal height; for flow-oriented syntax this controls which direction items fill their track in.

<a id="valdef-item-wrap-reverse"></a>reverse  
<a id="ref-for-primary-track-axis①"></a>

Items are placed in end-to-start order in the axis opposite to the [primary track axis](#primary-track-axis).

<a id="valdef-item-wrap-wrap-reverse"></a>wrap-reverse  
Computes to wrap reverse.

<a id="ref-for-propdef-flex-wrap"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value exists for consistency with the existing [flex-wrap](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-wrap) value.

<a id="ref-for-propdef-item-direction②"></a>

<a id="ref-for-propdef-item-track②"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-b3c639a9"></a> The interpretation and naming of this property depends on the interpretation of axes for [item-direction](#propdef-item-direction)/[item-track](#propdef-item-track). [\[Issue \#11480\]](https://github.com/w3c/csswg-drafts/issues/11480)

<a id="ref-for-propdef-item-pack②"></a>

### <a id="item-pack-options"></a> Item Placement Packing Mode: the [item-pack](#propdef-item-pack) property

<strong>Table 7 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-item-pack"></a>item-pack

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-any①"></a>

<a id="ref-for-comb-one①②"></a>

normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) dense [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) balance

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

normal

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-lanes-container②⑦"></a>

<a id="ref-for-grid-container②⑧"></a>

<a id="ref-for-flex-container④"></a>

[flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container), [grid containers](https://www.w3.org/TR/css-grid-2/#grid-container), [grid lanes containers](#grid-lanes-container)

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

as specified

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

This property controls how items are distributed among the tracks in a layout-specific way.

<a id="valdef-item-pack-normal"></a>normal  
Uses the default packing strategy for the layout mode.

<a id="valdef-item-pack-dense"></a>dense  
Allows backtracking to place items in earlier spaces that were skipped. (Such spaces can exist because earlier items were too big for those spaces.)

<a id="ref-for-flex-layout①①"></a>

For example, in [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout) this allows placing items on earlier lines that still have enough empty space left over.

<a id="valdef-item-pack-balance"></a>balance  
<a id="ref-for-flex-layout①②"></a>

<a id="ref-for-propdef-text-wrap-style"></a>

In [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout), this value balances the amount of content on each line (including the last line), similar to [text-wrap-style: balance](https://www.w3.org/TR/css-text-4/#propdef-text-wrap-style).

<a id="ref-for-propdef-item-flow"></a>

### <a id="item-flow"></a> Item Placement Shorthand: the [item-flow](#propdef-item-flow) shorthand

<strong>Table 8 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-item-flow"></a>item-flow

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-flow-tolerance⑥"></a>

<a id="ref-for-propdef-item-pack③"></a>

<a id="ref-for-propdef-item-wrap②"></a>

<a id="ref-for-comb-any②"></a>

<a id="ref-for-propdef-item-direction③"></a>

[\<'item-direction'\>](#propdef-item-direction) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'item-wrap'\>](#propdef-item-wrap) <a id="ref-for-comb-any③"></a>\|\| [\<'item-pack'\>](#propdef-item-pack) <a id="ref-for-comb-any④"></a>\|\| [\<'flow-tolerance'\>](#propdef-flow-tolerance)

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

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-longhand"></a>

This [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) sets all its item-\* [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand) in a single declaration.

## <a id="acknowledgements"></a>10.  Acknowledgements

Special thanks goes to Cameron McCormack who wrote a masonry layout explainer document (from which was lifted the Background chapter) and presented it to the CSSWG, and to Mats Palmgren who developed the original version of this specification alongside a prototype implementation in Firefox. Thanks also to everyone who provided feedback on the [initial proposal](https://github.com/w3c/csswg-drafts/issues/4650) for this feature as well as the [many subsequent issues](https://github.com/w3c/csswg-drafts/issues?q=is%3Aissue%20state%3Aopen%20label%3A%22topic%3A%20masonry%22), and to Noam Rosenthal particularly for [naming it](https://martinfowler.com/bliki/TwoHardThings.html).

## <a id="security"></a>11.  Security Considerations

As a layout specification, this spec introduces no new security considerations beyond that exposed by CSS layout in general.

## <a id="privacy"></a>12.  Privacy Considerations

As a layout specification, this spec introduces no new privacy considerations beyond that exposed by CSS layout in general.

## <a id="changes"></a> Changes

### <a id="changes-3"></a> Additions Since Level 2

The following features have been added since [Level 2](https://www.w3.org/TR/css-grid-2/):

- <a id="ref-for-grid-lanes-layout①⑨"></a>

  Added [grid lanes layout](#grid-lanes-layout).

- Expanded repeat(auto-fill) and repeat(auto-fit) to accept indefinite track sizing functions. ([Issue 9321](https://github.com/w3c/csswg-drafts/issues/9321), [Issue 12899](https://github.com/w3c/csswg-drafts/issues/12899))

- <a id="ref-for-propdef-flow-tolerance⑦"></a>

  Introduced the [flow-tolerance](#propdef-flow-tolerance) property.

- <a id="ref-for-propdef-item-flow①"></a>

  Introduced the [item-flow](#propdef-item-flow) property and its longhands, as generic controls for item ordering and placement. See [Appendix A: Generic Layout Item Flow Controls: the item-\* properties](#flow-control). Note: This is still under discussion. ([Issue 11480](https://github.com/w3c/csswg-drafts/issues/11480))

### <a id="recent-changes"></a> Recent Changes

The following changes have been made since the [23 December 2025 Working Draft](https://www.w3.org/TR/2025/WD-css-grid-3-20251223/):

- <a id="ref-for-propdef-flow-tolerance⑧"></a>

  Renamed item-tolerance to [flow-tolerance](#propdef-flow-tolerance). ([Issue 10884](https://github.com/w3c/csswg-drafts/issues/10884))

The following changes have been made since the [16 December 2025 Working Draft](https://www.w3.org/TR/2025/WD-css-grid-3-20251216/):

- <a id="ref-for-valdef-grid-auto-flow-normal"></a>

  Defined [normal](#valdef-grid-auto-flow-normal) as the initial value for [setting the track orientation](#grid-lanes-orientation).

- <a id="ref-for-self-alignment-properties②"></a>

  <a id="ref-for-stacking-axis②⑦"></a>

  Defined the [self-alignment properties](https://www.w3.org/TR/css-align-3/#self-alignment-properties)’ effect in the [stacking axis](#stacking-axis). See [§ 6.4 Stacking-axis Self Alignment: the align-self/justify-self and align-items/justify-items properties](#stacking-self-alignment).

The following changes have been made since the [17 September 2025 Working Draft](https://www.w3.org/TR/2025/WD-css-grid-3-20250917/):

- <a id="ref-for-valdef-display-grid-lanes"></a>

  <a id="ref-for-grid-lanes-layout②⓪"></a>

  Defined a new inner display type, [grid-lanes](#valdef-display-grid-lanes), to establish [grid lanes layout](#grid-lanes-layout), and updated spec vocabulary to match. ([Issue 12022](https://github.com/w3c/csswg-drafts/issues/12022))

- <a id="ref-for-funcdef-repeat-line-color-repeat④"></a>

  Dropped the unnecessary auto-areas value from [repeat()](https://www.w3.org/TR/css-gaps-1/#funcdef-repeat-line-color-repeat). ([Issue 10854](https://github.com/w3c/csswg-drafts/issues/10854))

- Adjusted heuristic for repeat(auto-fill) and repeat(auto-fit) with indefinite track sizing functions. ([Issue 12899](https://github.com/w3c/csswg-drafts/issues/12899))

- <a id="ref-for-running-position⑦"></a>

  Floored the contribution of items to a track’s [running position](#running-position) at zero. ([Issue 12918](https://github.com/w3c/csswg-drafts/issues/12918))

- Renamed item-tolerance to flow-tolerance. ([Issue 10884](https://github.com/w3c/csswg-drafts/issues/10884))

See also [earlier changes](https://www.w3.org/TR/2025/WD-css-grid-3-20250917/#recent-changes).

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

- auto
  - [value for item-direction, item-track](#valdef-item-direction-auto), in § Unnumbered section
  - [value for item-wrap, item-cross](#valdef-item-wrap-auto), in § Unnumbered section
- [auto-placement cursor](#auto-placement-cursor), in § 4.4
- [\<auto-repeat\>](#typedef-auto-repeat), in § 3.1.1
- [balance](#valdef-item-pack-balance), in § Unnumbered section
- [column](#valdef-item-direction-column), in § Unnumbered section
- [column-reverse](#valdef-item-direction-column-reverse), in § Unnumbered section
- [dense](#valdef-item-pack-dense), in § Unnumbered section
- [flow-tolerance](#propdef-flow-tolerance), in § 4.2
- [grid axis](#grid-axis), in § 2
- [grid-axis](#grid-axis), in § 2
- [grid-lanes](#valdef-display-grid-lanes), in § 2.2
- [grid lanes container](#grid-lanes-container), in § 2
- [Grid lanes layout](#grid-lanes-layout), in § 2
- [grid lanes subgrid](#grid-lanes-subgrid), in § 3.2
- [infinite](#valdef-flow-tolerance-infinite), in § 4.2
- [inline-grid-lanes](#valdef-display-inline-grid-lanes), in § 2.2
- [item-cross](#propdef-item-cross), in § Unnumbered section
- [item-direction](#propdef-item-direction), in § Unnumbered section
- [item-flow](#propdef-item-flow), in § Unnumbered section
- [item groups](#item-groups), in § 3.4.2
- [item-pack](#propdef-item-pack), in § Unnumbered section
- [item-track](#propdef-item-track), in § Unnumbered section
- [item-wrap](#propdef-item-wrap), in § Unnumbered section
- [\<length-percentage\>](#valdef-flow-tolerance-length-percentage), in § 4.2
- normal
  - [value for flow-tolerance](#valdef-flow-tolerance-normal), in § 4.2
  - [value for grid-auto-flow](#valdef-grid-auto-flow-normal), in § 2.3
  - [value for item-pack](#valdef-item-pack-normal), in § Unnumbered section
  - [value for item-wrap, item-cross](#valdef-item-wrap-normal), in § Unnumbered section
- [nowrap](#valdef-item-wrap-nowrap), in § Unnumbered section
- [primary axis](#primary-axis), in § Unnumbered section
- [primary placement axis](#primary-placement-axis), in § Unnumbered section
- [primary track axis](#primary-track-axis), in § Unnumbered section
- [reverse](#valdef-item-wrap-reverse), in § Unnumbered section
- [row](#valdef-item-direction-row), in § Unnumbered section
- [row-reverse](#valdef-item-direction-row-reverse), in § Unnumbered section
- [running position](#running-position), in § 4.4
- [stacking axis](#stacking-axis), in § 2
- [stacking-axis](#stacking-axis), in § 2
- [stacking range](#stacking-range), in § 5
- [tie threshold](#grid-lanes-tie-threshold), in § 4.2
- [virtual grid item](#virtual-grid-item), in § 3.4.2
- [wrap](#valdef-item-wrap-wrap), in § Unnumbered section
- [wrap-reverse](#valdef-item-wrap-wrap-reverse), in § Unnumbered section

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ALIGN-3\] defines the following terms:
  - <a id="fde954ee"></a>align-content
  - <a id="e6b1dd7a"></a>align-items
  - <a id="9d95c839"></a>align-self
  - <a id="73f86444"></a>alignment baseline
  - <a id="fb5c7e3f"></a>alignment container
  - <a id="dc2ecc7a"></a>alignment subject
  - <a id="4d2cf2cf"></a>baseline alignment
  - <a id="207a83d3"></a>baseline-sharing group
  - <a id="567f0e7d"></a>box alignment properties
  - <a id="515ec31f"></a>center
  - <a id="b7d152c3"></a>column-gap
  - <a id="da7c24e2"></a>distributed alignment
  - <a id="020c3333"></a>end
  - <a id="3c3b0bc2"></a>fallback alignment
  - <a id="b4d83355"></a>gap
  - <a id="6f9416f0"></a>grid-gap
  - <a id="de6bd31b"></a>justify-content
  - <a id="e4237559"></a>justify-items
  - <a id="80d2b689"></a>justify-self
  - <a id="50284573"></a>normal
  - <a id="2b373266"></a>row-gap
  - <a id="34ae2cc3"></a>self-alignment properties
  - <a id="35e80fd9"></a>start
  - <a id="ec67a9e7"></a>stretch
- \[CSS-BOX-4\] defines the following terms:
  - <a id="f72f5cb4"></a>content box
  - <a id="0778a939"></a>margin box
  - <a id="7c998c9a"></a>outer edge
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="2218854e"></a>forced break
  - <a id="872c3891"></a>fragmentainer
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="6b448e93"></a>initial value
  - <a id="8f27be0f"></a>longhand property
  - <a id="e14541aa"></a>shorthand
  - <a id="980ac56a"></a>shorthand property
  - <a id="1a2b1083"></a>used value
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="a015488b"></a>block-level
  - <a id="431a9cad"></a>blockify
  - <a id="0923db9e"></a>containing block
  - <a id="e8c16097"></a>display
  - <a id="57aa8824"></a>independent formatting context
  - <a id="6b9bba07"></a>inline-level
  - <a id="bd725d5d"></a>order
  - <a id="983aad4f"></a>order-modified document order
  - <a id="4edc72b0"></a>reading-flow
- \[CSS-FLEXBOX-1\] defines the following terms:
  - <a id="cc7f0a64"></a>flex container
  - <a id="07e702cf"></a>flex layout
  - <a id="546f7867"></a>flex-flow
  - <a id="b70fab87"></a>flex-wrap
  - <a id="97651ccf"></a>multi-line flex container
  - <a id="b55ea3dd"></a>single-line flex container
- \[CSS-GAPS-1\] defines the following terms:
  - <a id="4894a76b"></a>repeat()
- \[CSS-GRID-2\] defines the following terms:
  - <a id="5fb77d58"></a>\<line-names\>
  - <a id="0195744a"></a>\<track-size\>
  - <a id="da3291f7"></a>auto-fill
  - <a id="7b233769"></a>auto-fit
  - <a id="bdee6ddc"></a>automatic grid position
  - <a id="c18c6560"></a>automatic placement
  - <a id="b48e63f1"></a>automatic position
  - <a id="a6553ee0"></a>collapsed grid track
  - <a id="d911d74f"></a>definite grid position
  - <a id="fdfd719c"></a>definite position
  - <a id="57b234bc"></a>explicit grid
  - <a id="7086fc61"></a>grid area
  - <a id="df72a52c"></a>grid container
  - <a id="ba30fc9a"></a>grid item
  - <a id="db4ae04f"></a>grid layout
  - <a id="b4a14210"></a>grid track
  - <a id="228fb890"></a>grid-auto-columns
  - <a id="4ab05c66"></a>grid-auto-flow
  - <a id="6cc286fe"></a>grid-auto-rows
  - <a id="88fad469"></a>grid-placement property
  - <a id="354cf3be"></a>grid-template-columns
  - <a id="25d528b6"></a>grid-template-rows
  - <a id="3208cbb5"></a>implicit grid
  - <a id="564e922b"></a>implicit grid track
  - <a id="b82baa6c"></a>max track sizing function
  - <a id="1881b3bf"></a>min track sizing function
  - <a id="9edcd4ef"></a>none
  - <a id="bbc693a1"></a>subgrid
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="b8c34db8"></a>position
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="c1c732b9"></a>available space
  - <a id="66f218c1"></a>definite
  - <a id="a47902ec"></a>max-content constraint
  - <a id="8a39af7f"></a>max-content size
  - <a id="451a41ae"></a>min-content constraint
  - <a id="6a444fd6"></a>min-content size
  - <a id="47ea2436"></a>outer size
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="7801e50a"></a>text-wrap-style
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="af4a190d"></a>+
  - <a id="8cd4f032"></a>,
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="fb688f4f"></a>direction
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="599428b5"></a>block-axis
  - <a id="a6eb24bb"></a>inline axis
  - <a id="82ddda8c"></a>inline-axis
  - <a id="90c7548c"></a>start
  - <a id="eb6008ce"></a>writing mode

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

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 6 November 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-flexbox-1"></a>\[CSS-FLEXBOX-1\]  
Elika Etemad; Tab Atkins Jr.; Rossen Atanassov. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 14 October 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css-gaps-1"></a>\[CSS-GAPS-1\]  
Kevin Babbitt. [CSS Gap Decorations Module Level 1](https://www.w3.org/TR/css-gaps-1/). 17 April 2025. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-gaps-1&#x2F;](https://www.w3.org/TR/css-gaps-1/)

<a id="biblio-css-grid-1"></a>\[CSS-GRID-1\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 1](https://www.w3.org/TR/css-grid-1/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-1&#x2F;](https://www.w3.org/TR/css-grid-1/)

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

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

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-css-multicol-1"></a>\[CSS-MULTICOL-1\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 1](https://www.w3.org/TR/css-multicol-1/). 16 May 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-1&#x2F;](https://www.w3.org/TR/css-multicol-1/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

## <a id="property-index"></a>Property Index

<strong>Table 9 — structured row/cell transcription</strong>

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

<a id="ref-for-propdef-flow-tolerance⑨"></a>

[flow-tolerance](#propdef-flow-tolerance)

<strong>Column 2 (data cell):</strong>

normal \| \<length-percentage\> \| infinite

<strong>Column 3 (data cell):</strong>

normal

<strong>Column 4 (data cell):</strong>

grid lanes containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

relative to the grid-axis content box size of the grid lanes container

<strong>Column 7 (data cell):</strong>

as length

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

a computed \<length-percentage\> value

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-item-cross②"></a>

[item-cross](#propdef-item-cross)

<strong>Column 2 (data cell):</strong>

\[ auto \| nowrap \| wrap \] \|\| \[ normal \| reverse \] \| wrap-reverse

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

flex containers, grid containers, grid lanes containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

as specified

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-item-direction④"></a>

[item-direction](#propdef-item-direction)

<strong>Column 2 (data cell):</strong>

auto \| row \| column \| row-reverse \| column-reverse

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

flex containers, grid containers, grid lanes containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

as specified

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-item-flow②"></a>

[item-flow](#propdef-item-flow)

<strong>Column 2 (data cell):</strong>

\<'item-direction'\> \|\| \<'item-wrap'\> \|\| \<'item-pack'\> \|\| \<'flow-tolerance'\>

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

<strong>Row 6</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-item-pack④"></a>

[item-pack](#propdef-item-pack)

<strong>Column 2 (data cell):</strong>

normal \| dense \|\| balance

<strong>Column 3 (data cell):</strong>

normal

<strong>Column 4 (data cell):</strong>

flex containers, grid containers, grid lanes containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

as specified

<strong>Row 7</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-item-track③"></a>

[item-track](#propdef-item-track)

<strong>Column 2 (data cell):</strong>

auto \| row \| column \| row-reverse \| column-reverse

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

flex containers, grid containers, grid lanes containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

as specified

<strong>Row 8</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-item-wrap③"></a>

[item-wrap](#propdef-item-wrap)

<strong>Column 2 (data cell):</strong>

\[ auto \| nowrap \| wrap \] \|\| \[ normal \| reverse \] \| wrap-reverse

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

flex containers, grid containers, grid lanes containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

as specified

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Or should reordering be the default behavior for auto-placed items here? [↵](#issue-f3901e05)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> ISSUE(12820): Write up how grid/masonry formatting contexts work more formally? [↵](#issue-3be4bca6)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> TBD [↵](#issue-d4908b1e)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Figure out whether we are re-using [grid-auto-flow](https://www.w3.org/TR/css-grid-2/#propdef-grid-auto-flow) here (and what it’s values mean) or defining a new property like grid-lanes-direction. [\[Issue \#12803\]](https://github.com/w3c/csswg-drafts/issues/12803) [↵](#issue-ff7d700c)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should this work also in Grid Layout somehow, or fall back to a single repetition? If so, how? [\[Issue \#10915\]](https://github.com/w3c/csswg-drafts/issues/10915) [↵](#issue-c70906df)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Technically, if your explicit placement does not enter or cross over the repetition, we can know its placement prior to resolving the reptition. Do we want to adjust the algorithm above to account for this? That is, given auto repeat(auto-fill, ...) auto, if you explicitly place an item against line 1 or -1, we could let it only affect that track (rather than pretending it’ll go in every track). [↵](#issue-5b45728c)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> What if this conflicts with the lanes orientation, or results in both axes stacking? [↵](#issue-ff7026b1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Should this also update the placement cursor? [↵](#issue-9c647ef6)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Is this a reasonable definition for how the [self-alignment properties](https://www.w3.org/TR/css-align-3/#self-alignment-properties) should work in the [stacking axis](#stacking-axis)? [\[Issue \#10275\]](https://github.com/w3c/csswg-drafts/issues/10275) [↵](#issue-7e3f9302)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> We could support baseline alignment in the first row. Do we want to? [↵](#issue-4a2d589c)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> It might be useful to define a static position in the [stacking axis](#stacking-axis). Maybe it could defined as the max (or min?) current [running position](#running-position) of the [grid-axis](#grid-axis) tracks at that point? Or the end of the item before it? [↵](#issue-4007c2e6)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This section is likely to move to another spec, such as [\[css-display-4\]](#biblio-css-display-4), since it affects multiple display types. It is also still under discussion as to whether this is a good idea. [↵](#issue-888c395d)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The CSSWG is still figuring out how these properties should be named and fit together. [\[Issue \#11480\]](https://github.com/w3c/csswg-drafts/issues/11480) [↵](#issue-c971c9c1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> There two open debates on this property: a) what should it be called and b) does it describe the primary direction of placement, or the orientation of the tracks into which items are placed; in other words, is the primary axis defined by this property the primary placement axis or the primary track axis. These are identical for [flex layout](https://www.w3.org/TR/css-flexbox-1/#flex-layout) and [grid layout](https://www.w3.org/TR/css-grid-2/#grid-layout), but differ for [grid lanes layout](#grid-lanes-layout) whose primary placement direction is across its tracks. See [latest discussion](https://github.com/w3c/csswg-drafts/issues/12803). [\[Issue \#11480\]](https://github.com/w3c/csswg-drafts/issues/11480) [↵](#issue-f870e6cb)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The interpretation and naming of this property depends on the interpretation of axes for [item-direction](#propdef-item-direction)/[item-track](#propdef-item-track). [\[Issue \#11480\]](https://github.com/w3c/csswg-drafts/issues/11480) [↵](#issue-b3c639a9)
