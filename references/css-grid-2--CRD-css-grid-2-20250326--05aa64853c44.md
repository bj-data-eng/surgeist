Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Grid Layout Module Level 2](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/).

Original copyright notice: Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Grid Layout Module Level 2

Source snapshot: https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/

Snapshot SHA-256: 05aa64853c4428146973943b35caf121e44c1076bdf5b8c29f8896dba9b778e2

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 13 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Grid Layout Module Level 2

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This CSS module defines a two-dimensional grid-based layout system, optimized for user interface design. In the grid layout model, the children of a grid container can be positioned into arbitrary slots in a predefined flexible or fixed-size layout grid. Level 2 expands Grid by adding “subgrid” capabilities for nested grids to participate in the sizing of their parent grids.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20231103/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-grid” in the title, like this: “\[css-grid\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-grid%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/policies/process/20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-436134ac"></a> If you notice any inconsistencies between this Grid Layout Module and the [Flexible Box Layout Module](https://www.w3.org/TR/css-flexbox/), please report them to the CSSWG, as this is likely an error.

## <a id="intro"></a>1.  Introduction

<em>This section is not normative.</em>

Grid Layout is a layout model for CSS that has powerful abilities to control the sizing and positioning of boxes and their contents. Unlike [Flexible Box Layout](https://www.w3.org/TR/css-flexbox-1/), which is single-axis–oriented, Grid Layout is optimized for 2-dimensional layouts: those in which alignment of content is desired in both dimensions.

![An example of flex layout: two rows of items, the first being three items a third of the space each, and the second being five items, a fifth of the space each. There is therefore alignment along the “rows”, but not along the “columns”.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/flex-layout.png)

Representative Flex Layout Example

![An example of grid layout: two rows of items, the first being four items—the last of which spans both rows, and the second being two items—the first of which spans the first two columns— plus the spanned item from the first row.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/grid-layout.png)

Representative Grid Layout Example

In addition, due to its ability to explicitly position items in the grid, Grid Layout allows dramatic transformations in visual layout structure without requiring corresponding markup changes. By combining [media queries](https://www.w3.org/TR/css3-mediaqueries/) with the CSS properties that control layout of the grid container and its children, authors can adapt their layout to changes in device form factors, orientation, and available space, while preserving a more ideal semantic structuring of their content across presentations.

Although many layouts can be expressed with either Grid or Flexbox, they each have their specialties. Grid enforces 2-dimensional alignment, uses a top-down approach to layout, allows explicit overlapping of items, and has more powerful spanning capabilities. Flexbox focuses on space distribution within an axis, uses a simpler bottom-up approach to layout, can use a content-size–based line-wrapping system to control its secondary axis, and relies on the underlying markup hierarchy to build more complex layouts. It is expected that both will be valuable and complementary tools for CSS authors.

Grid Level 2 adds the [subgrid](#subgrids) feature: a subgridded axis is one which matches up its grid lines to lines in the element’s parent’s grid, and which derives the sizes of its tracks through this integration with the parent grid.

### <a id="background"></a>1.1.  Background and Motivation

![Image: Application layout example requiring horizontal and vertical alignment.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/basic-form.png)

Application layout example requiring horizontal and vertical alignment.

As websites evolved from simple documents into complex, interactive applications, techniques for document layout, e.g. floats, were not necessarily well suited for application layout. By using a combination of tables, JavaScript, or careful measurements on floated elements, authors discovered workarounds to achieve desired layouts. Layouts that adapted to the available space were often brittle and resulted in counter-intuitive behavior as space became constrained. As an alternative, authors of many web applications opted for a fixed layout that cannot take advantage of changes in the available rendering space on a screen.

<a id="ref-for-grid-area"></a>

The capabilities of grid layout address these problems. It provides a mechanism for authors to divide available space for layout into columns and rows using a set of predictable sizing behaviors. Authors can then precisely position and size the building block elements of their application into the [grid areas](#grid-area) defined by the intersections of these columns and rows. The following examples illustrate the adaptive capabilities of grid layout, and how it allows a cleaner separation of content and style.

#### <a id="adapting-to-available-space"></a>1.1.1.  Adapting Layouts to Available Space

![Let us consider the layout of a game in two columns and three rows: the game title in the top left corner, the menu below it, and the score in the bottom left with the game board occupying the top and middle cells on the right followed by game controls filling the bottom left. The left column is sized to exactly fit its contents (the game title, menu items, and score), with the right column filling the remaining space.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/game-smaller.png)

Five grid items arranged according to content size and available space.

![As more space becomes available in larger screens, the middle row / right column are allowed to expand to fill that space.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/game-larger.png)

Growth in the grid due to an increase in available space.

Grid layout can be used to intelligently resize elements within a webpage. The adjacent figures represent a game with five major components in the layout: the game title, stats area, game board, score area, and control area. The author’s intent is to divide the space for the game such that:

- The stats area always appears immediately under the game title.
- The game board appears to the right of the stats and title.
- The top of the game title and the game board should always align.
- The bottom of the game board and bottom of the stats area align when the game has reached its minimum height. In all other cases the game board will stretch to take advantage of all the space available to it.
- The controls are centered under the game board.
- The top of the score area is aligned to the top of the controls area.
- The score area is beneath the stats area.
- The score area is aligned to the controls beneath the stats area.

The following grid layout example shows how an author might achieve all the sizing, placement, and alignment rules declaratively.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1d4671e1"></a>
>
> <a id="ref-for-grid-item"></a>
>
> <a id="ref-for-grid-container"></a>
>
> <a id="ref-for-grid-item①"></a>
>
> <a id="ref-for-grid-item②"></a>
>
> ```css
> /**
>  * Define the space for each grid item by declaring the grid
>  * on the grid container.
>  */
> #grid {
>   /**
>    * Two columns:
>    *  1. the first sized to content,
>    *  2. the second receives the remaining space
>    *     (but is never smaller than the minimum size of the board
>    *     or the game controls, which occupy this column [Figure 4])
>    *
>    * Three rows:
>    *  3. the first sized to content,
>    *  4. the middle row receives the remaining space
>    *     (but is never smaller than the minimum height
>    *      of the board or stats areas)
>    *  5. the last sized to content.
>    */
>   display: grid;
>   grid-template-columns:
>     /* 1 */ auto
>     /* 2 */ 1fr;
>   grid-template-rows:
>     /* 3 */ auto
>     /* 4 */ 1fr
>     /* 5 */ auto;
> }
> 
> /* Specify the position of each grid item using coordinates on
>  * the 'grid-row' and 'grid-column' properties of each grid item.
>  */
> #title    { grid-column: 1; grid-row: 1; }
> #score    { grid-column: 1; grid-row: 3; }
> #stats    { grid-column: 1; grid-row: 2; align-self: start; }
> #board    { grid-column: 2; grid-row: 1 / span 2; }
> #controls { grid-column: 2; grid-row: 3; justify-self: center; }
> ```
>
> ```markup
> <div id="grid">
>   <div id="title">Game Title</div>
>   <div id="score">Score</div>
>   <div id="stats">Stats</div>
>   <div id="board">Board</div>
>   <div id="controls">Controls</div>
> </div>
> ```
<a id="ref-for-grid-item③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: There are multiple ways to specify the structure of the grid and to position and size [grid items](#grid-item), each optimized for different scenarios.

#### <a id="source-independence"></a>1.1.2.  Source-Order Independence

![Image: An arrangement suitable for portrait orientation.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/game-portrait.png)

An arrangement suitable for “portrait” orientation.

![Image: An arrangement suitable for landscape orientation.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/game-landscape.png)

An arrangement suitable for “landscape“ orientation.

Continuing the prior example, the author also wants the game to adapt to different devices. Also, the game should optimize the placement of the components when viewed either in portrait or landscape orientation (Figures 6 and 7). By combining grid layout with media queries, the author is able to use the same semantic markup, but rearrange the layout of elements independent of their source order, to achieve the desired layout in both orientations.

<a id="ref-for-grid-item④"></a>

The following example uses grid layout’s ability to name the space which will be occupied by a [grid item](#grid-item). This allows the author to avoid rewriting rules for <a id="ref-for-grid-item⑤"></a>grid items as the grid’s definition changes.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-dcdb8379"></a>
>
> ```css
> @media (orientation: portrait) {
>   #grid {
>     display: grid;
> 
>     /* The rows, columns and areas of the grid are defined visually
>      * using the grid-template-areas property.  Each string is a row,
>      * and each word an area.  The number of words in a string
>      * determines the number of columns. Note the number of words
>      * in each string must be identical. */
>     grid-template-areas: "title stats"
>                          "score stats"
>                          "board board"
>                          "ctrls ctrls";
> 
>     /* The way to size columns and rows can be assigned with the
>      * grid-template-columns and grid-template-rows properties. */
>     grid-template-columns: auto 1fr;
>     grid-template-rows: auto auto 1fr auto;
>   }
> }
> 
> @media (orientation: landscape) {
>   #grid {
>     display: grid;
> 
>     /* Again the template property defines areas of the same name,
>      * but this time positioned differently to better suit a
>      * landscape orientation. */
>     grid-template-areas: "title board"
>                          "stats board"
>                          "score ctrls";
> 
>     grid-template-columns: auto 1fr;
>     grid-template-rows: auto 1fr auto;
>   }
> }
> 
> /* The grid-area property places a grid item into a named
>  * area of the grid. */
> #title    { grid-area: title }
> #score    { grid-area: score }
> #stats    { grid-area: stats }
> #board    { grid-area: board }
> #controls { grid-area: ctrls }
> ```
>
> ```markup
> <div id="grid">
>   <div id="title">Game Title</div>
>   <div id="score">Score</div>
>   <div id="stats">Stats</div>
>   <div id="board">Board</div>
>   <div id="controls">Controls</div>
> </div>
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The reordering capabilities of grid layout intentionally affect <em>only the visual rendering</em>, leaving speech order and navigation based on the source order. This allows authors to manipulate the visual presentation while leaving the source order intact and optimized for non-CSS UAs and for linear models such as speech and sequential navigation.

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Grid item placement and reordering must not be used
	as a substitute for correct source ordering,
	as that can ruin the accessibility of the document.</strong>

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="overview"></a>2.  Overview

<em>This section is not normative.</em>

<a id="ref-for-grid"></a>

<a id="ref-for-grid-container①"></a>

Grid Layout controls the layout of its content through the use of a [grid](#grid): an intersecting set of horizontal and vertical lines which create a sizing and positioning coordinate system for the [grid container](#grid-container)’s contents. Grid Layout features

- fixed, flexible, and content-based [track sizing functions](#track-sizing)

- [explicit item placement](#placement) via forwards (positive) and backwards (negative) numerical grid coordinates, named grid lines, and named grid areas; automatic item placement into empty areas, including [reordering with order](#order-property)

- space-sensitive track repetition and automatic addition of rows or columns to accommodate additional content

- <a id="ref-for-gutter"></a>

  control over alignment and spacing with [margins](#auto-margins), [gutters](https://www.w3.org/TR/css-align-3/#gutter), and the [alignment properties](https://www.w3.org/TR/css-align/)

- the ability to overlap content and [control layering with z-index](#z-order)

<a id="ref-for-grid-container②"></a>

<a id="ref-for-flex-container"></a>

[Grid containers](#grid-container) can be nested or mixed with [flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container) as necessary to create more complex layouts.

### <a id="overview-grid"></a>2.1.  Declaring the Grid

<a id="ref-for-grid-track"></a>

<a id="ref-for-grid-row"></a>

<a id="ref-for-grid-column"></a>

<a id="ref-for-grid①"></a>

<a id="ref-for-explicit-grid"></a>

<a id="ref-for-propdef-grid"></a>

The [tracks](#grid-track) ([rows](#grid-row) and [columns](#grid-column)) of the [grid](#grid) are declared and sized either explicitly through the [explicit grid](#explicit-grid) properties or are implicitly created when items are placed outside the <a id="ref-for-explicit-grid①"></a>explicit grid. The [grid](#propdef-grid) shorthand and its sub-properties define the parameters of the grid. [§ 7 Defining the Grid](#grid-definition)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-78337715"></a> Below are some examples of grid declarations:
>
> - <a id="ref-for-valdef-flex-fr"></a>
>
>   <a id="ref-for-valdef-grid-template-columns-auto"></a>
>
>   The following declares a grid with four named areas: `H`, `A`, `B`, and `F`. The first column is sized to fit its contents ([auto](#valdef-grid-template-columns-auto)), and the second column takes up the remaining space ([1fr](#valdef-flex-fr)). Rows default to <a id="ref-for-valdef-grid-template-columns-auto①"></a>auto (content-based) sizing; the last row is given a fixed size of 30px.
>
>   ```text
>   main {
>     display: grid;
>     grid: "H    H "
>           "A    B "
>           "F    F " 30px
>     /     auto 1fr;
>   }
>   ```
>
> - <a id="ref-for-valdef-flex-fr①"></a>
>
>   The following declares a grid with as many rows of at least 5em as will fit in the height of the grid container (100vh). The grid has no explicit columns; instead columns are added as content is added, the resulting column widths are equalized ([1fr](#valdef-flex-fr)). Since content overflowing to the right won’t print, an alternate layout for printing adds rows instead.
>
>   ```text
>   main {
>     display: grid;
>     grid: repeat(auto-fill, 5em) / auto-flow 1fr;
>     height: 100vh;
>   }
>   @media print {
>     main {
>       grid: auto-flow 1fr / repeat(auto-fill, 5em);
>     }
>   }
>   ```
>
> - The following declares a grid with 5 evenly-sized columns and three rows, with the middle row taking up all remaining space (and at least enough to fit its contents).
>
>   ```text
>   main {
>     display: grid;
>     grid: auto 1fr auto / repeat(5, 1fr);
>     min-height: 100vh;
>   }
>   ```
### <a id="overview-placement"></a>2.2.  Placing Items

<a id="ref-for-grid-container③"></a>

<a id="ref-for-grid-item⑥"></a>

<a id="ref-for-flex-item"></a>

<a id="ref-for-grid-area①"></a>

<a id="ref-for-grid②"></a>

<a id="ref-for-grid-placement-property"></a>

<a id="ref-for-auto-placement"></a>

The contents of the [grid container](#grid-container) are organized into individual [grid items](#grid-item) (analogous to [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item)), which are then assigned to predefined [areas](#grid-area) in the [grid](#grid). They can be explicitly placed using coordinates through the [grid-placement properties](#grid-placement-property) or implicitly placed into empty areas using [auto-placement](#auto-placement). [§ 8 Placing Grid Items](#placement)

<a id="ref-for-propdef-grid-area"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9f280f0c"></a> Below are some examples of grid placement declarations using the [grid-area](#propdef-grid-area) shorthand:
>
> ```text
> grid-area: a;          /* Place into named grid area “a”     */
> grid-area: auto;       /* Auto-place into next empty area    */
> grid-area: 2 / 4;      /* Place into row 2, column 4         */
> grid-area: 1 / 3 / -1; /* Place into column 3, span all rows */
> grid-area: header-start / sidebar-start / footer-end / sidebar-end;
>                        /* Place using named lines            */
> ```
>
> <a id="ref-for-propdef-grid-row"></a>
>
> <a id="ref-for-propdef-grid-column"></a>
>
> These are equivalent to the following [grid-row](#propdef-grid-row) + [grid-column](#propdef-grid-column) declarations:
>
> ```text
> grid-row: a;                         grid-column: a;
> grid-row: auto;                      grid-column: auto;
> grid-row: 2;                         grid-column: 4;
> grid-row: 1 / -1;                    grid-column: 3;
> grid-row: header-start / footer-end; grid-column: sidebar-start / sidebar-end;
> ```
>
> <a id="ref-for-propdef-grid-row-start"></a>
>
> <a id="ref-for-propdef-grid-row-end"></a>
>
> <a id="ref-for-propdef-grid-column-start"></a>
>
> <a id="ref-for-propdef-grid-column-end"></a>
>
> They can further be decomposed into the [grid-row-start](#propdef-grid-row-start)/[grid-row-end](#propdef-grid-row-end)/[grid-column-start](#propdef-grid-column-start)/[grid-column-end](#propdef-grid-column-end) longhands, e.g.
>
> ```text
> grid-area: a;
> /* Equivalent to grid-row-start: a; grid-column-start: a; grid-row-end: a; grid-column-end: a; */
> 
> grid-area: 1 / 3 / -1;
> /* Equivalent to grid-row-start: 1; grid-column-start: 3; grid-row-end: -1; grid-column-end: auto; */
> ```
### <a id="overview-sizing"></a>2.3.  Sizing the Grid

<a id="ref-for-grid-item⑦"></a>

<a id="ref-for-grid-track①"></a>

Once the [grid items](#grid-item) have been [placed](#placement), the sizes of the [grid tracks](#grid-track) (rows and columns) are calculated, accounting for the sizes of their contents and/or available space as specified in the grid definition.

<a id="ref-for-grid-container④"></a>

<a id="ref-for-propdef-align-content"></a>

<a id="ref-for-propdef-justify-content"></a>

The resulting sized grid is [aligned](#grid-align) within the [grid container](#grid-container) according to the <a id="ref-for-grid-container⑤"></a>grid container’s [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) and [justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content) properties. [§ 11 Alignment and Spacing](#alignment)

<a id="ref-for-grid-container⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b6cb46e7"></a> The following example justifies all columns by distributing any extra space between them, and centers the grid in the [grid container](#grid-container) when it is smaller than 100vh.
>
> ```text
> main {
>   display: grid;
>   grid: auto-flow auto / repeat(auto-fill, 5em);
>   min-height: 100vh;
>   justify-content: space-between;
>   align-content: safe center;
> }
> ```
<a id="ref-for-grid-item⑧"></a>

<a id="ref-for-grid-area②"></a>

<a id="ref-for-box-alignment-properties"></a>

Finally each [grid item](#grid-item) is sized and aligned within its assigned [grid area](#grid-area), as specified by its own [sizing](https://www.w3.org/TR/CSS2/visudet.html) [\[CSS2\]](#biblio-css2) and [alignment properties](https://www.w3.org/TR/css-align-3/#box-alignment-properties) [\[CSS-ALIGN-3\]](#biblio-css-align-3).

## <a id="grid-concepts"></a>3.  Grid Layout Concepts and Terminology

<a id="ref-for-grid-container⑦"></a>

<a id="ref-for-grid③"></a>

<a id="ref-for-grid-line"></a>

<a id="ref-for-grid-area③"></a>

<a id="ref-for-grid-item⑨"></a>

In <a id="grid-layout"></a>grid layout, the content of a [grid container](#grid-container) is laid out by positioning and aligning it into a [grid](#grid). The <a id="grid"></a>grid is an intersecting set of horizontal and vertical [grid lines](#grid-line) that divides the <a id="ref-for-grid-container⑧"></a>grid container’s space into [grid areas](#grid-area), into which [grid items](#grid-item) (representing the <a id="ref-for-grid-container⑨"></a>grid container’s content) can be placed. There are two sets of <a id="ref-for-grid-line①"></a>grid lines: one set defining <a id="grid-column"></a>columns that run along the [block axis](https://www.w3.org/TR/css3-writing-modes/#block-axis), and an orthogonal set defining <a id="grid-row"></a>rows along the [inline axis](https://www.w3.org/TR/css3-writing-modes/#inline-axis). [\[CSS3-WRITING-MODES\]](#biblio-css3-writing-modes)

![Image: Grid Lines.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/grid-lines.png)

Grid lines: Three in the block axis and four in the inline axis.

### <a id="grid-line-concept"></a>3.1.  Grid Lines

<a id="ref-for-grid④"></a>

<a id="ref-for-grid-line②"></a>

<a id="ref-for-grid-item①⓪"></a>

<a id="grid-line"></a>Grid lines are the horizontal and vertical dividing lines of the [grid](#grid). A [grid line](#grid-line) exists on either side of a column or row. They can be referred to by numerical index, or by an author-specified name. A [grid item](#grid-item) references the <a id="ref-for-grid-line③"></a>grid lines to determine its position within the <a id="ref-for-grid⑤"></a>grid using the [grid-placement properties](#placement).

<a id="ref-for-grid-line④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-38bc117e"></a> The following two examples both create three column [grid lines](#grid-line) and four row <a id="ref-for-grid-line⑤"></a>grid lines.
>
> <a id="ref-for-grid-item①①"></a>
>
> <a id="ref-for-grid-line⑥"></a>
>
> This first example demonstrates how an author would position a [grid item](#grid-item) using [grid line](#grid-line) numbers:
>
> ```css
> #grid {
>   display: grid;
>   grid-template-columns: 150px 1fr;
>   grid-template-rows: 50px 1fr 50px;
> }
> 
> #item1 { grid-column: 2;
>          grid-row-start: 1; grid-row-end: 4; }
> ```
>
> <a id="ref-for-grid-line⑦"></a>
>
> This second example uses explicitly named [grid lines](#grid-line):
>
> ```css
> /* equivalent layout to the prior example, but using named lines */
> #grid {
>   display: grid;
>   grid-template-columns: 150px [item1-start] 1fr [item1-end];
>   grid-template-rows: [item1-start] 50px 1fr 50px [item1-end];
> }
> 
> #item1 {
>   grid-column: item1-start / item1-end;
>   grid-row: item1-start / item1-end;
> }
> ```
### <a id="grid-track-concept"></a>3.2.  Grid Tracks and Cells

<a id="ref-for-grid-column①"></a>

<a id="ref-for-grid-row①"></a>

<a id="ref-for-grid-line⑧"></a>

<a id="ref-for-grid-track②"></a>

<a id="grid-track"></a>Grid track is a generic term for a [grid column](#grid-column) or [grid row](#grid-row)—in other words, it is the space between two adjacent [grid lines](#grid-line). Each [grid track](#grid-track) is assigned a sizing function, which controls how wide or tall the column or row may grow, and thus how far apart its bounding <a id="ref-for-grid-line⑨"></a>grid lines are. Adjacent <a id="ref-for-grid-track③"></a>grid tracks can be separated by [gutters](#gutters) but are otherwise packed tightly.

<a id="ref-for-grid-item①②"></a>

A <a id="grid-cell"></a>grid cell is the intersection of a grid row and a grid column. It is the smallest unit of the grid that can be referenced when positioning [grid items](#grid-item).

<a id="ref-for-grid-container①⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1c5d4651"></a> In the following example there are two columns and three rows. The first column is fixed at 150px. The second column uses flexible sizing, which is a function of the unassigned space in the grid, and thus will vary as the width of the [grid container](#grid-container) changes. If the used width of the <a id="ref-for-grid-container①①"></a>grid container is 200px, then the second column is 50px wide. If the used width of the <a id="ref-for-grid-container①②"></a>grid container is 100px, then the second column is 0px and any content positioned in the column will overflow the <a id="ref-for-grid-container①③"></a>grid container.
>
> ```text
> #grid {
>   display: grid;
>   grid-template-columns: 150px 1fr;  /* two columns */
>   grid-template-rows: 50px 1fr 50px; /* three rows  */
> }
> ```
### <a id="grid-area-concept"></a>3.3.  Grid Areas

<a id="ref-for-grid-item①③"></a>

<a id="ref-for-grid-area④"></a>

<a id="ref-for-grid-cell"></a>

<a id="ref-for-grid-line①⓪"></a>

<a id="ref-for-grid-track④"></a>

<a id="ref-for-propdef-grid-template-areas"></a>

<a id="ref-for-grid-container①④"></a>

A <a id="grid-area"></a>grid area is the logical space used to lay out one or more [grid items](#grid-item). A [grid area](#grid-area) consists of one or more adjacent [grid cells](#grid-cell). It is bound by four [grid lines](#grid-line), one on each side of the <a id="ref-for-grid-area⑤"></a>grid area, and participates in the sizing of the [grid tracks](#grid-track) it intersects. A <a id="ref-for-grid-area⑥"></a>grid area can be named explicitly using the [grid-template-areas](#propdef-grid-template-areas) property of the [grid container](#grid-container), or referenced implicitly by its bounding <a id="ref-for-grid-line①①"></a>grid lines. A <a id="ref-for-grid-item①④"></a>grid item is assigned to a <a id="ref-for-grid-area⑦"></a>grid area using the [grid-placement properties](#placement).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-909c0a0c"></a>
>
> ```text
> /* using the template syntax */
> #grid  {
>   display: grid;
>   grid-template-areas: ". a"
>                        "b a"
>                        ". a";
>   grid-template-columns: 150px 1fr;
>   grid-template-rows: 50px 1fr 50px;
>   height: 100vh;
> }
> 
> #item1 { grid-area: a }
> #item2 { grid-area: b }
> #item3 { grid-area: b }
> 
> /* Align items 2 and 3 at different points in the grid area "b".  */
> /* By default, grid items are stretched to fit their grid area    */
> /* and these items would layer one over the other. */
> #item2 { align-self: start; }
> #item3 { justify-self: end; align-self: end; }
> ```
<a id="ref-for-grid-item①⑤"></a>

<a id="ref-for-grid-area⑧"></a>

<a id="ref-for-grid-track⑤"></a>

<a id="ref-for-intrinsic-sizing-function"></a>

<a id="ref-for-grid-line①②"></a>

A [grid item](#grid-item)’s [grid area](#grid-area) forms the containing block into which it is laid out. <a id="ref-for-grid-item①⑥"></a>Grid items placed into the same <a id="ref-for-grid-area⑨"></a>grid area do not directly affect each other’s layout. Indirectly, however, a <a id="ref-for-grid-item①⑦"></a>grid item occupying a [grid track](#grid-track) with an [intrinsic sizing function](#intrinsic-sizing-function) can affect the size of that track (and thus the positions of its bounding [grid lines](#grid-line)), which in turn can affect the position or size of another <a id="ref-for-grid-item①⑧"></a>grid item.

### <a id="subgrid-items"></a>3.4.  Nested vs. Subgridded Items

<a id="ref-for-grid-item①⑨"></a>

<a id="ref-for-grid-container①⑤"></a>

<a id="ref-for-propdef-display"></a>

A [grid item](#grid-item) can itself be a [grid container](#grid-container) by giving it [display: grid](https://www.w3.org/TR/css-display-4/#propdef-display). In the general case the layout of this <a id="nested-grid"></a>nested grid’s contents will be independent of the layout of the <a id="parent-grid"></a>parent grid it participates in.

<a id="ref-for-grid-item②⓪"></a>

<a id="ref-for-nested-grid"></a>

<a id="ref-for-grid-container①⑥"></a>

<a id="ref-for-subgrid"></a>

<a id="ref-for-parent-grid"></a>

However, in some cases it might be necessary for the contents of multiple [grid items](#grid-item) to align to each other. A [nested grid](#nested-grid) can defer the definition of its rows and/or columns to its parent [grid container](#grid-container), making it a <a id="subgrid"></a>subgrid. In this case, the <a id="ref-for-grid-item②①"></a>grid items of the [subgrid](#subgrid) participate in sizing the [parent grid](#parent-grid), allowing the contents of both grids to align. See [§ 9 Subgrids](#subgrids).

<a id="ref-for-subgrid①"></a>

<a id="ref-for-valdef-grid-template-rows-subgrid"></a>

<a id="ref-for-propdef-grid-template-rows"></a>

<a id="ref-for-propdef-grid-template-columns"></a>

<a id="ref-for-subgridded-axis"></a>

A [subgrid](#subgrid) is established by the [subgrid](#valdef-grid-template-rows-subgrid) keyword of [grid-template-rows](#propdef-grid-template-rows) or [grid-template-columns](#propdef-grid-template-columns), and can be [subgridded](#subgridded-axis) in either axis or in both. A grid that has no <a id="ref-for-subgridded-axis①"></a>subgridded axis is a <a id="standalone-grid"></a>standalone grid.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-921f3930"></a> For example, suppose we have a form consisting of a list of inputs with labels:
>
> ```html
> <ul>
>   <li><label>Name:</label> <input name=fn>
>   <li><label>Address:</label> <input name=address>
>   <li><label>Phone:</label> <input name=phone>
> </ul>
> ```
>
> We want the labels and inputs to align, and we want to style each list item with a border. This can be accomplished with subgrid layout:
>
> ```text
> ul {
>   display: grid;
>   grid: auto-flow / auto 1fr;
> }
> li {
>   grid-column: span 2;
>   display: grid;
>   grid-template-columns: subgrid;
>   border: solid;
> }
> label {
>   grid-column: 1;
> }
> input {
>   grid-column: 2;
> }
> ```
## <a id="order-accessibility"></a>4.  Reordering and Accessibility

<a id="ref-for-propdef-order"></a>

<a id="ref-for-grid-placement"></a>

Grid layout gives authors great powers of rearrangement over the document. However, these are not a substitute for correct ordering of the document source. The [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) property and [grid placement](#grid-placement) <em>do not</em> affect ordering in non-visual media (such as [speech](https://www.w3.org/TR/css3-speech/)). Likewise, rearranging grid items visually does not affect the default traversal order of sequential navigation modes (such as cycling through links, see e.g. [`tabindex`](https://html.spec.whatwg.org/multipage/interaction.html#attr-tabindex) [\[HTML\]](#biblio-html)).

<a id="ref-for-propdef-order①"></a>

<a id="ref-for-grid-placement-property①"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> Authors <em>must</em> use <a href="https://www.w3.org/TR/css-flexbox-1/#propdef-order">order</a> and the <a href="#grid-placement-property">grid-placement properties</a> only for visual, not logical, reordering of content.
	Style sheets that use these features to perform logical reordering are non-conforming.</strong>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is so that non-visual media and non-CSS UAs, which typically present content linearly, can rely on a logical source order, while grid layout’s placement and ordering features are used to tailor the visual arrangement. (Since visual perception is two-dimensional and non-linear, the desired visual order is not always equivalent to the desired reading order.)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-33799738"></a> Many web pages have a similar shape in the markup, with a header on top, a footer on bottom, and then a content area and one or two additional columns in the middle. Generally, it’s desirable that the content come first in the page’s source code, before the additional columns. However, this makes many common designs, such as simply having the additional columns on the left and the content area on the right, difficult to achieve. This has been addressed in many ways over the years, often going by the name "Holy Grail Layout" when there are two additional columns. Grid Layout makes this example trivial. For example, take the following sketch of a page’s code and desired layout:
>
> ```markup
> <!DOCTYPE html>
> <header>...</header>
> <main>...</main>
> <nav>...</nav>
> <aside>...</aside>
> <footer>...</footer>
> ```
>
> ![In this page the header is at the top and the footer at the bottom, but the main is in the center, flanked by the nav on the right and the aside on the left.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/grid-order-page.svg)
>
> This layout can be easily achieved with grid layout:
>
> ```css
> body { display: grid;
>        grid: "h h h"
>              "a b c"
>              "f f f";
>        grid-template-columns: auto 1fr 20%; }
> main    { grid-area: b; min-width: 12em;     }
> nav     { grid-area: a; /* auto min-width */ }
> aside   { grid-area: c; min-width: 12em;     }
> ```
>
> <a id="ref-for-valdef-align-self-stretch"></a>
>
> As an added bonus, the columns will all be [equal-height](https://www.w3.org/TR/css-align-3/#valdef-align-self-stretch) by default, and the main content will be as wide as necessary to fill the screen. Additionally, this can then be combined with media queries to switch to an all-vertical layout on narrow screens:
>
> ```css
> @media all and (max-width: 60em) {
>   /* Too narrow to support three columns */
>   body { display: block; }
> }
> ```
<a id="ref-for-propdef-order②"></a>

<a id="ref-for-grid-placement-property②"></a>

In order to preserve the author’s intended ordering in all presentation modes, authoring tools—including WYSIWYG editors as well as Web-based authoring aids—​must reorder the underlying document source and not use [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) or [grid-placement properties](#grid-placement-property) to perform reordering unless the author has explicitly indicated that the underlying document order (which determines speech and navigation order) should be <em>out-of-sync</em> with the visual order.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-29a4cd08"></a> For example, a tool might offer both drag-and-drop arrangement of grid items as well as handling of media queries for alternate layouts per screen size range.
>
> <a id="ref-for-grid-placement-property③"></a>
>
> Since most of the time, reordering should affect all screen ranges as well as navigation and speech order, the tool would match the resulting drag-and-drop visual arrangement by simultaneously reordering the DOM layer. In some cases, however, the author may want different visual arrangements per screen size. The tool could offer this functionality by using the [grid-placement properties](#grid-placement-property) together with media queries, but also tie the smallest screen size’s arrangement to the underlying DOM order (since this is most likely to be a logical linear presentation order) while using <a id="ref-for-grid-placement-property④"></a>grid-placement properties to rearrange the visual presentation in other size ranges.
>
> <a id="ref-for-grid-placement-property⑤"></a>
>
> This tool would be conformant, whereas a tool that only ever used the [grid-placement properties](#grid-placement-property) to handle drag-and-drop grid rearrangement (however convenient it might be to implement it that way) would be non-conformant.

## <a id="grid-model"></a>5.  Grid Containers

<a id="ref-for-valdef-display-grid"></a>

<a id="ref-for-valdef-display-inline-grid"></a>

<a id="ref-for-propdef-display①"></a>

### <a id="grid-containers"></a>5.1.  Establishing Grid Containers: the [grid](#valdef-display-grid) and [inline-grid](#valdef-display-inline-grid) [display](https://www.w3.org/TR/css-display-4/#propdef-display) values

<strong>Table 1 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-display②"></a>

[display](https://www.w3.org/TR/css-display-4/#propdef-display)

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[New values:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-one"></a>

grid [\|](https://www.w3.org/TR/css-values-4/#comb-one) inline-grid

<a id="valdef-display-grid"></a>grid  
<a id="ref-for-flow-layout"></a>

<a id="ref-for-block-level"></a>

<a id="ref-for-grid-container①⑦"></a>

This value causes an element to generate a [grid container](#grid-container) box that is [block-level](https://www.w3.org/TR/css-display-4/#block-level) when placed in [flow layout](https://www.w3.org/TR/css-display-4/#flow-layout).

<a id="valdef-display-inline-grid"></a>inline-grid  
<a id="ref-for-flow-layout①"></a>

<a id="ref-for-inline-level"></a>

<a id="ref-for-grid-container①⑧"></a>

This value causes an element to generate a [grid container](#grid-container) box that is [inline-level](https://www.w3.org/TR/css-display-4/#inline-level) when placed in [flow layout](https://www.w3.org/TR/css-display-4/#flow-layout).

<a id="ref-for-subgrid②"></a>

<a id="ref-for-independent-formatting-context"></a>

<a id="ref-for-block-formatting-context"></a>

<a id="ref-for-grid-container①⑨"></a>

<a id="ref-for-grid⑥"></a>

<a id="ref-for-grid-line①③"></a>

<a id="ref-for-grid-item②②"></a>

A <a id="grid-container"></a>grid container that is not a [subgrid](#subgrid) establishes an [independent](https://www.w3.org/TR/css-display-4/#independent-formatting-context) <a id="grid-formatting-context"></a>grid formatting context for its contents. This is the same as establishing an independent [block formatting context](https://www.w3.org/TR/css-display-4/#block-formatting-context), except that grid layout is used instead of block layout: floats do not intrude into the grid container, and the grid container’s margins do not collapse with the margins of its contents. The contents of a [grid container](#grid-container) are laid out into a [grid](#grid), with [grid lines](#grid-line) forming the boundaries of each [grid items](#grid-item)’ containing block.

<a id="ref-for-subgrid③"></a>

<a id="ref-for-grid-formatting-context"></a>

<a id="ref-for-independent-formatting-context①"></a>

Unlike those of a regular nested grid, a [subgrid](#subgrid)’s contents participate in its parent [grid formatting context](#grid-formatting-context); thus a subgrid does not establish an [independent formatting context](https://www.w3.org/TR/css-display-4/#independent-formatting-context).

Grid containers are not block containers, and so some properties that were designed with the assumption of block layout don’t apply in the context of grid layout. In particular:

- <a id="ref-for-propdef-display③"></a>

  <a id="ref-for-grid-item②③"></a>

  <a id="ref-for-propdef-clear"></a>

  <a id="ref-for-propdef-float"></a>

  [float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float) and [clear](https://www.w3.org/TR/CSS2/visuren.html#propdef-clear) have no effect on a [grid item](#grid-item). However, the <a id="ref-for-propdef-float①"></a>float property still affects the computed value of [display](https://www.w3.org/TR/css-display-4/#propdef-display) on children of a grid container, as this occurs <em>before</em> <a id="ref-for-grid-item②④"></a>grid items are determined.

- <a id="ref-for-propdef-vertical-align"></a>

  [vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) has no effect on a grid item.

- <a id="ref-for-grid-container②⓪"></a>

  <a id="ref-for-selectordef-first-letter"></a>

  <a id="ref-for-selectordef-first-line"></a>

  the [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) and [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) pseudo-elements do not apply to [grid containers](#grid-container), and <a id="ref-for-grid-container②①"></a>grid containers do not contribute a first formatted line or first letter to their ancestors.

<a id="ref-for-propdef-display④"></a>

<a id="ref-for-valdef-display-inline-grid①"></a>

<a id="ref-for-valdef-display-grid①"></a>

If an element’s specified [display](https://www.w3.org/TR/css-display-4/#propdef-display) is [inline-grid](#valdef-display-inline-grid) and the element is floated or absolutely positioned, the computed value of <a id="ref-for-propdef-display⑤"></a>display is [grid](#valdef-display-grid). The table in [CSS 2.1 Chapter 9.7](https://www.w3.org/TR/CSS2/visuren.html#dis-pos-flo) is thus amended to contain an additional row, with <a id="ref-for-valdef-display-inline-grid②"></a>inline-grid in the "Specified Value" column and <a id="ref-for-valdef-display-grid②"></a>grid in the "Computed Value" column.

### <a id="intrinsic-sizes"></a>5.2.  Sizing Grid Containers

Note see [\[CSS-SIZING-3\]](#biblio-css-sizing-3) for a definition of the terms in this section.

<a id="ref-for-grid-container②②"></a>

A [grid container](#grid-container) is sized using the rules of the formatting context in which it participates:

- <a id="ref-for-inline-size"></a>

  <a id="ref-for-grid-placement-auto"></a>

  <a id="ref-for-block-box"></a>

  <a id="ref-for-block-formatting-context①"></a>

  <a id="ref-for-block-level①"></a>

  As a [block-level](https://www.w3.org/TR/css-display-4/#block-level) box in a [block formatting context](https://www.w3.org/TR/css-display-4/#block-formatting-context), it is sized like a [block box](https://www.w3.org/TR/css-display-4/#block-box) that establishes a formatting context, with an [auto](#grid-placement-auto) [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) calculated as for non-replaced block boxes.

- <a id="ref-for-inline-formatting-context"></a>

  As an inline-level box in an [inline formatting context](https://www.w3.org/TR/css-display-4/#inline-formatting-context), it is sized as an atomic inline-level box (such as an inline-block).

<a id="ref-for-grid-container②③"></a>

<a id="ref-for-grid-placement-auto①"></a>

<a id="ref-for-block-size"></a>

In both inline and block formatting contexts, the [grid container](#grid-container)’s [auto](#grid-placement-auto) [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) is its max-content size.

<strong data-conversion-semantic="note">Note:</strong> The block layout spec should probably define this, but it isn’t written yet.

<a id="ref-for-max-content"></a>

<a id="ref-for-min-content"></a>

<a id="ref-for-grid-container②④"></a>

<a id="ref-for-max-content-constraint"></a>

<a id="ref-for-min-content-constraint"></a>

The [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content) ([min-content size](https://www.w3.org/TR/css-sizing-3/#min-content)) of a [grid container](#grid-container) is the sum of the <a id="ref-for-grid-container②⑤"></a>grid container’s track sizes (including gutters) in the appropriate axis, when the grid is sized under a [max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint) ([min-content constraint](https://www.w3.org/TR/css-sizing-3/#min-content-constraint)).

### <a id="overflow"></a>5.3.  Scrollable Grid Overflow

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-grid-container②⑥"></a>

The [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) property applies to [grid containers](#grid-container).

<a id="ref-for-grid⑦"></a>

<a id="ref-for-grid-container②⑦"></a>

<a id="ref-for-scrollable-overflow-region"></a>

Just as it is included in intrinsic sizing (see [§ 5.2 Sizing Grid Containers](#intrinsic-sizes)), the [grid](#grid) is also included in a [grid container](#grid-container)’s [scrollable overflow region](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region).

<a id="ref-for-grid-container②⑧"></a>

<a id="ref-for-scroll-container"></a>

<a id="ref-for-scrollable-overflow-rectangle"></a>

<a id="ref-for-propdef-place-content"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Beware the interaction with padding when the [grid container](#grid-container) is a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container): additional padding is defined to be added to the [scrollable overflow rectangle](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-rectangle) as needed to enable [place-content: end](https://www.w3.org/TR/css-align-3/#propdef-place-content) alignment of scrollable content. See [CSS Overflow 3 § 2.2 Scrollable Overflow](https://www.w3.org/TR/css-overflow-3/#scrollable)

### <a id="overlarge-grids"></a>5.4.  Limiting Large Grids

<a id="ref-for-implicit-grid"></a>

<a id="ref-for-clamp-a-grid-area"></a>

Since memory is limited, UAs may clamp the possible size of the [implicit grid](#implicit-grid) to be within a UA-defined limit (which should accommodate lines in the range \[-10000, 10000\]), dropping all lines outside that limit. If a grid item is placed outside this limit, its grid area must be [clamped](#clamp-a-grid-area) to within this limited grid.

To <a id="clamp-a-grid-area"></a>clamp a grid area:

- <a id="ref-for-grid-area①⓪"></a>

  <a id="ref-for-grid-span"></a>

  <a id="ref-for-grid⑧"></a>

  If the [grid area](#grid-area) would [span](#grid-span) outside the limited grid, its span is clamped to the last line of the limited [grid](#grid).

- <a id="ref-for-grid-area①①"></a>

  <a id="ref-for-grid-track⑥"></a>

  If the [grid area](#grid-area) would be placed completely outside the limited grid, its span must be truncated to 1 and the area repositioned into the last [grid track](#grid-track) on that side of the grid.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-39312220"></a> For example, if a UA only supported grids with at most 1000 tracks in each dimension, the following placement properties:
>
> ```css
> .grid-item {
>   grid-row: 500 / 1500;
>   grid-column: 2000 / 3000;
> }
> ```
>
> Would end up being equivalent to:
>
> ```css
> .grid-item {
>   grid-row: 500 / 1001;
>   grid-column: 1000 / 1001;
> }
> ```
## <a id="grid-items"></a>6.  Grid Items

<a id="ref-for-grid-container②⑨"></a>

<a id="ref-for-in-flow"></a>

Loosely speaking, the <a id="grid-item"></a>grid items of a [grid container](#grid-container) are boxes representing its [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) contents.

<a id="ref-for-in-flow①"></a>

<a id="ref-for-grid-container③⓪"></a>

<a id="ref-for-grid-item②⑤"></a>

<a id="ref-for-css-text-sequence"></a>

<a id="ref-for-anonymous"></a>

<a id="ref-for-block-container"></a>

<a id="ref-for-propdef-white-space"></a>

<a id="ref-for-text-nodes"></a>

Each [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) child of a [grid container](#grid-container) becomes a [grid item](#grid-item), and each child [text sequence](https://www.w3.org/TR/css-display-4/#css-text-sequence) is wrapped in an [anonymous](https://www.w3.org/TR/css-display-4/#anonymous) [block container](https://www.w3.org/TR/css-display-4/#block-container) <a id="ref-for-grid-item②⑥"></a>grid item. However, if the <a id="ref-for-css-text-sequence①"></a>text sequence contains only [white space](https://www.w3.org/TR/CSS2/text.html#white-space-prop) (i.e. characters that can be affected by the [white-space](https://www.w3.org/TR/css-text-4/#propdef-white-space) property) it is instead not rendered (just as if its [text nodes](https://www.w3.org/TR/css-display-4/#text-nodes) were display:none).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4842bdfd"></a>
>
> Examples of grid items:
>
> ```markup
> <div style="display: grid">
> 
>   <!-- grid item: block child -->
>   <div id="item1">block</div>
> 
>   <!-- grid item: floated element; floating is ignored -->
>   <div id="item2" style="float: left;">float</div>
> 
>   <!-- grid item: anonymous block box around inline content -->
>   anonymous item 3
> 
>   <!-- grid item: inline child -->
>   <span>
>     item 4
>     <!-- grid items do not split around blocks -->
>     <q style="display: block" id=not-an-item>item 4</q>
>     item 4
>   </span>
> </div>
> ```
>
> grid items determined from above code block
>
> [](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/examples/grid-item-determination.html)
>
> ![grid item containing block. grid item containing float. (Anonymous, unstyleable) grid item containing anonymous item 3. grid item containing three blocks in succession: Anonymous block containing item 4. \<q\> element block containing item 4. Anonymous block containing item 4.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/grid-item-determination.png)
>
> 1.  grid item containing `block`.
> 2.  grid item containing `float`.
> 3.  (Anonymous, unstyleable) grid item containing `anonymous item 3`.
> 4.  grid item containing three blocks in succession:
>     - Anonymous block containing `item 4`.
>     - `<q>` element block containing `item 4`.
>     - Anonymous block containing `item 4`.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: inter-element white space disappears: it does not become its own grid item, even though inter-element text <em>does</em> get wrapped in an anonymous grid item.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The box of a anonymous item is unstyleable, since there is no element to assign style rules to. Its contents will however inherit styles (such as font settings) from the grid container.

### <a id="grid-item-display"></a>6.1.  Grid Item Display

<a id="ref-for-subgrid④"></a>

<a id="ref-for-grid-item②⑦"></a>

<a id="ref-for-establish-an-independent-formatting-context"></a>

<a id="ref-for-grid-formatting-context①"></a>

Unless it is a [subgrid](#subgrid), a [grid item](#grid-item) [establishes an independent formatting context](https://www.w3.org/TR/css-display-4/#establish-an-independent-formatting-context) for its contents. However, grid items are <a id="grid-level"></a>grid-level boxes, not block-level boxes: they participate in their container’s [grid formatting context](#grid-formatting-context), not in a block formatting context.

<a id="ref-for-computed-value"></a>

<a id="ref-for-propdef-display⑥"></a>

<a id="ref-for-valdef-display-grid③"></a>

<a id="ref-for-valdef-display-inline-grid③"></a>

<a id="ref-for-blockify"></a>

If the [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [display](https://www.w3.org/TR/css-display-4/#propdef-display) value of an element’s nearest ancestor element (skipping display:contents ancestors) is [grid](#valdef-display-grid) or [inline-grid](#valdef-display-inline-grid), the element’s own <a id="ref-for-propdef-display⑦"></a>display value is [blockified](https://www.w3.org/TR/css-display-4/#blockify). (See [CSS2.1§9.7](https://www.w3.org/TR/CSS2/visuren.html#dis-pos-flo) [\[CSS2\]](#biblio-css2) and [CSS Display 3 § 2.7 Automatic Box Type Transformations](https://www.w3.org/TR/css-display-3/#transformations) for details on this type of <a id="ref-for-propdef-display⑧"></a>display value conversion.)

<a id="ref-for-valdef-display-grid④"></a>

<a id="ref-for-valdef-display-inline-grid④"></a>

<a id="ref-for-grid-container③①"></a>

<a id="ref-for-replaced-element"></a>

<a id="ref-for-propdef-display⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Blockification still occurs even when the [grid](#valdef-display-grid) or [inline-grid](#valdef-display-inline-grid) element does not end up generating a [grid container](#grid-container) box, e.g. when it is [replaced](https://www.w3.org/TR/css-display-4/#replaced-element) or in a [display: none](https://www.w3.org/TR/css-display-4/#propdef-display) subtree.

<a id="ref-for-propdef-display①⓪"></a>

<a id="ref-for-grid-item②⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Some values of [display](https://www.w3.org/TR/css-display-4/#propdef-display) normally trigger the creation of anonymous boxes around the original box. If such a box is a [grid item](#grid-item), it is blockified first, and so anonymous box creation will not happen. For example, two contiguous <a id="ref-for-grid-item②⑨"></a>grid items with <a id="ref-for-propdef-display①①"></a>display: table-cell will become two separate <a id="ref-for-propdef-display①②"></a>display: block <a id="ref-for-grid-item③⓪"></a>grid items, instead of being wrapped into a single anonymous table.

### <a id="grid-item-sizing"></a>6.2.  Grid Item Sizing

<a id="ref-for-grid-item③①"></a>

<a id="ref-for-grid-area①②"></a>

A [grid item](#grid-item) is sized within the containing block defined by its [grid area](#grid-area).

<a id="ref-for-grid-item③②"></a>

<a id="ref-for-automatic-size"></a>

[Grid item](#grid-item) calculations for [automatic sizes](https://www.w3.org/TR/css-sizing-3/#automatic-size) in a given dimensions vary by their [self-alignment values](https://www.w3.org/TR/css-align-3/#self-alignment):

<a id="ref-for-valdef-align-self-normal"></a>

[normal](https://www.w3.org/TR/css-align-3/#valdef-align-self-normal)

<a id="ref-for-preferred-aspect-ratio"></a>

<a id="ref-for-natural-size"></a>

<a id="ref-for-replaced-element①"></a>

<a id="ref-for-propdef-align-self"></a>

If the grid item has no [preferred aspect ratio](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio), and no [natural size](https://www.w3.org/TR/css-images-3/#natural-size) in the relevant axis (if it is a [replaced element](https://www.w3.org/TR/css-display-4/#replaced-element)), the grid item is sized as for [align-self: stretch](https://www.w3.org/TR/css-align-3/#propdef-align-self).

Otherwise, the grid item is sized consistent with the size calculation rules for block-level elements for the corresponding axis. (See [CSS 2 §  10 Visual formatting model details](https://www.w3.org/TR/CSS2/visudet.html#q10.0).)

<a id="ref-for-valdef-align-self-stretch①"></a>

[stretch](https://www.w3.org/TR/css-align-3/#valdef-align-self-stretch)

<a id="ref-for-inline-size①"></a>

<a id="ref-for-stretch-fit-size"></a>

Use the [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) calculation rules for non-replaced boxes (defined in [CSS 2 § 10.3.3 Block-level, non-replaced elements in normal flow](https://www.w3.org/TR/CSS2/visudet.html#blockwidth)), i.e. the [stretch-fit size](https://www.w3.org/TR/css-sizing-3/#stretch-fit-size).

<a id="ref-for-preferred-aspect-ratio①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This can distort the aspect ratio of an item with a [preferred aspect ratio](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio), if its size is also constrained in the other axis.

all other values

<a id="ref-for-valdef-width-fit-content"></a>

Size the item as [fit-content](https://www.w3.org/TR/css-sizing-4/#valdef-width-fit-content).

> <strong data-conversion-semantic="note">Note</strong>
>
> The following informative table summarizes the automatic sizing of grid items:
>
> <strong>Table 2 — structured row/cell transcription</strong>
>
> Summary of automatic sizing behavior of grid items
>
> <strong>Row 1</strong>
>
> <strong>Column 1 (header cell):</strong>
>
> Alignment
>
> <strong>Column 2 (header cell):</strong>
>
> Non-replaced Element Size
>
> <strong>Column 3 (header cell):</strong>
>
> Replaced Element Size
>
> <strong>Row 2</strong>
>
> <strong>Column 1 (header cell; scope row):</strong>
>
> <a id="ref-for-valdef-align-self-normal①"></a>
>
> [normal](https://www.w3.org/TR/css-align-3/#valdef-align-self-normal)
>
> <strong>Column 2 (data cell):</strong>
>
> Fill grid area
>
> <strong>Column 3 (data cell):</strong>
>
> <a id="ref-for-natural-size①"></a>
>
> Use [natural size](https://www.w3.org/TR/css-images-3/#natural-size)
>
> <strong>Row 3</strong>
>
> <strong>Column 1 (header cell; scope row):</strong>
>
> <a id="ref-for-valdef-align-self-stretch②"></a>
>
> [stretch](https://www.w3.org/TR/css-align-3/#valdef-align-self-stretch)
>
> <strong>Column 2 (data cell):</strong>
>
> Fill grid area
>
> <strong>Column 3 (data cell):</strong>
>
> Fill grid area
>
> <strong>Row 4</strong>
>
> <strong>Column 1 (header cell; scope row):</strong>
>
> <a id="ref-for-valdef-self-position-center"></a>
>
> <a id="ref-for-valdef-self-position-start"></a>
>
> [start](https://www.w3.org/TR/css-align-3/#valdef-self-position-start)/[center](https://www.w3.org/TR/css-align-3/#valdef-self-position-center)/etc.
>
> <strong>Column 2 (data cell):</strong>
>
> <a id="ref-for-valdef-width-fit-content①"></a>
>
> [fit-content](https://www.w3.org/TR/css-sizing-4/#valdef-width-fit-content) sizing (like floats)
>
> <strong>Column 3 (data cell):</strong>
>
> <a id="ref-for-natural-size②"></a>
>
> Use [natural size](https://www.w3.org/TR/css-images-3/#natural-size)

<a id="ref-for-valdef-width-auto"></a>

<a id="ref-for-propdef-min-width"></a>

<a id="ref-for-propdef-min-height"></a>

<a id="ref-for-flex-item①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) value of [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width) and [min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height) affects track sizing in the relevant axis similar to how it affects the main size of a [flex item](https://www.w3.org/TR/css-flexbox-1/#flex-item). See [§ 6.6 Automatic Minimum Size of Grid Items](#min-size-auto).

<a id="ref-for-propdef-order③"></a>

### <a id="order-property"></a>6.3.  Reordered Grid Items: the [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) property

<a id="ref-for-propdef-order④"></a>

<a id="ref-for-grid-item③③"></a>

The [order](https://www.w3.org/TR/css-flexbox-1/#propdef-order) property also applies to [grid items](#grid-item). It affects their [auto-placement](#grid-auto-flow-property) and [painting order](#z-order).

<a id="ref-for-propdef-order⑤"></a>

<strong data-conversion-semantic="advisement">Advisement:</strong> <strong> As with reordering flex items,
	the <a href="https://www.w3.org/TR/css-flexbox-1/#propdef-order">order</a> property must only be used
	when the visual order needs to be <em>out-of-sync</em> with the speech and navigation order;
	otherwise the underlying document source should be reordered instead.
	See <a href="https://www.w3.org/TR/css-flexbox-1/#order-accessibility"><cite>CSS Flexbox 1</cite> § 5.4.1 Reordering and Accessibility</a> in <a href="#biblio-css-flexbox-1" title="CSS Flexible Box Layout Module Level 1">&#x5B;CSS-FLEXBOX-1&#x5D;</a>.</strong>

### <a id="item-margins"></a>6.4.  Grid Item Margins and Paddings

<a id="ref-for-grid-area①③"></a>

<a id="ref-for-grid-item③④"></a>

As adjacent grid items are independently contained within the containing block formed by their [grid areas](#grid-area), the margins of adjacent [grid items](#grid-item) do not [collapse](https://www.w3.org/TR/CSS2/box.html#collapsing-margins).

<a id="ref-for-grid-item③⑤"></a>

<a id="ref-for-block-box①"></a>

<a id="ref-for-inline-size②"></a>

<a id="ref-for-containing-block"></a>

<a id="ref-for-writing-mode"></a>

Percentage margins and paddings on [grid items](#grid-item), like those on [block boxes](https://www.w3.org/TR/css-display-4/#block-box), are resolved against the [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) of their [containing block](https://www.w3.org/TR/css-display-4/#containing-block), e.g. left/right/top/bottom percentages all resolve against their <a id="ref-for-containing-block①"></a>containing block’s <em>width</em> in horizontal [writing modes](https://www.w3.org/TR/css-writing-modes-4/#writing-mode).

Auto margins expand to absorb extra space in the corresponding dimension, and can therefore be used for alignment. See [§ 11.2 Aligning with auto margins](#auto-margins)

<a id="ref-for-propdef-z-index"></a>

### <a id="z-order"></a>6.5.  Z-axis Ordering: the [z-index](https://www.w3.org/TR/CSS2/visuren.html#propdef-z-index) property

<a id="ref-for-grid-item③⑥"></a>

<a id="ref-for-grid-area①④"></a>

<a id="ref-for-order-modified-document-order"></a>

<a id="ref-for-propdef-z-index①"></a>

<a id="ref-for-valdef-z-index-auto"></a>

<a id="ref-for-propdef-position"></a>

<a id="ref-for-valdef-position-static"></a>

<a id="ref-for-valdef-position-relative"></a>

[Grid items](#grid-item) can overlap when they are positioned into intersecting [grid areas](#grid-area), or even when positioned in non-intersecting areas because of negative margins or positioning. The painting order of <a id="ref-for-grid-item③⑦"></a>grid items is exactly the same as inline blocks [\[CSS2\]](#biblio-css2), except that [order-modified document order](https://www.w3.org/TR/css-flexbox-1/#order-modified-document-order) is used in place of raw document order, and [z-index](https://www.w3.org/TR/CSS2/visuren.html#propdef-z-index) values other than [auto](https://drafts.csswg.org/css2/#valdef-z-index-auto) create a stacking context even if [position](https://www.w3.org/TR/css-position-3/#propdef-position) is [static](https://www.w3.org/TR/css-position-3/#valdef-position-static) (behaving exactly as if <a id="ref-for-propdef-position①"></a>position were [relative](https://www.w3.org/TR/css-position-3/#valdef-position-relative)). Thus the <a id="ref-for-propdef-z-index②"></a>z-index property can easily be used to control the z-axis order of grid items.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Descendants that are positioned outside a grid item still participate in any stacking context established by the grid item.

<a id="ref-for-propdef-z-index③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ae2f6ea5"></a> The following diagram shows several overlapping grid items, with a combination of implicit source order and explicit [z-index](https://www.w3.org/TR/CSS2/visuren.html#propdef-z-index) used to control their stacking order.
>
> ![](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/drawing-order.png)
>
> Drawing order controlled by z-index and source order.
>
> ```text
> <style type="text/css">
> #grid {
>   display: grid;
>   grid-template-columns: 1fr 1fr;
>   grid-template-rows: 1fr 1fr
> }
> #A { grid-column: 1 / span 2; grid-row: 2; align-self: end; }
> #B { grid-column: 1; grid-row: 1; z-index: 10; }
> #C { grid-column: 2; grid-row: 1; align-self: start; margin-left: -20px; }
> #D { grid-column: 2; grid-row: 2; justify-self: end; align-self: start; }
> #E { grid-column: 1 / span 2; grid-row: 1 / span 2;
>      z-index: 5; justify-self: center; align-self: center; }
> </style>
> 
> <div id="grid">
>   <div id="A">A</div>
>   <div id="B">B</div>
>   <div id="C">C</div>
>   <div id="D">D</div>
>   <div id="E">E</div>
> </div>
> ```
### <a id="min-size-auto"></a>6.6.  Automatic Minimum Size of Grid Items

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Much of the sizing terminology used in this section (and throughout the rest of the specification) is defined in [CSS Intrinsic and Extrinsic Sizing](https://www.w3.org/TR/css-sizing-3/) [\[CSS-SIZING-3\]](#biblio-css-sizing-3).

<a id="ref-for-min-width"></a>

<a id="ref-for-grid-item③⑧"></a>

<a id="ref-for-automatic-minimum-size"></a>

<a id="ref-for-content-based-minimum-size"></a>

To provide a more reasonable default [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width) for [grid items](#grid-item), the used value of its [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size) in a given axis is the [content-based minimum size](#content-based-minimum-size) if all of the following are true:

- <a id="ref-for-computed-value①"></a>

  <a id="ref-for-propdef-overflow①"></a>

  <a id="ref-for-scrollable-overflow-value"></a>

  its [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) is not a [scrollable overflow value](https://drafts.csswg.org/css-overflow-3/#scrollable-overflow-value)

- <a id="ref-for-grid-track⑦"></a>

  <a id="ref-for-min-track-sizing-function"></a>

  <a id="ref-for-valdef-grid-template-columns-auto②"></a>

  it spans at least one [track](#grid-track) in that axis whose [min track sizing function](#min-track-sizing-function) is [auto](#valdef-grid-template-columns-auto)

- <a id="ref-for-flexible-tracks"></a>

  if it spans more than one track in that axis, none of those tracks are [flexible](#flexible-tracks)

<a id="ref-for-automatic-minimum-size①"></a>

Otherwise, the [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size) is zero, as usual.

<a id="ref-for-content-based-minimum-size①"></a>

<a id="ref-for-intrinsic-size-contribution"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [content-based minimum size](#content-based-minimum-size) is a type of [intrinsic size contribution](https://www.w3.org/TR/css-sizing-3/#intrinsic-size-contribution), and thus the provisions in [CSS Sizing 3 § 5.2 Intrinsic Contributions](https://www.w3.org/TR/css-sizing-3/#intrinsic-contribution) apply.

<a id="ref-for-grid-item③⑨"></a>

<a id="ref-for-specified-size-suggestion"></a>

<a id="ref-for-transferred-size-suggestion"></a>

<a id="ref-for-replaced-element②"></a>

<a id="ref-for-content-size-suggestion"></a>

<a id="ref-for-grid-track⑧"></a>

<a id="ref-for-fixed-sizing-function"></a>

<a id="ref-for-max-track-sizing-function"></a>

<a id="ref-for-stretch-fit"></a>

<a id="ref-for-grid-area①⑤"></a>

<a id="ref-for-gutter①"></a>

The <a id="content-based-minimum-size"></a>content-based minimum size for a [grid item](#grid-item) in a given dimension is its [specified size suggestion](#specified-size-suggestion) if it exists, otherwise its [transferred size suggestion](#transferred-size-suggestion) if that exists and the element is [replaced](https://www.w3.org/TR/css-display-4/#replaced-element), else its [content size suggestion](#content-size-suggestion), see below. However, if in a given dimension the <a id="ref-for-grid-item④⓪"></a>grid item spans only [grid tracks](#grid-track) that have a [fixed](#fixed-sizing-function) [max track sizing function](#max-track-sizing-function), then its <a id="ref-for-specified-size-suggestion①"></a>specified size suggestion and <a id="ref-for-content-size-suggestion①"></a>content size suggestion in that dimension (and its input from this dimension to the <a id="ref-for-transferred-size-suggestion①"></a>transferred size suggestion in the opposite dimension) are further clamped to less than or equal to the [stretch fit](https://www.w3.org/TR/css-sizing-3/#stretch-fit) into the [grid area](#grid-area)’s maximum size in that dimension, as represented by the sum of those <a id="ref-for-grid-track⑨"></a>grid tracks’ <a id="ref-for-max-track-sizing-function①"></a>max track sizing functions plus any intervening <a id="ref-for-fixed-sizing-function①"></a>fixed [gutters](https://www.w3.org/TR/css-align-3/#gutter).

<a id="ref-for-max-width"></a>

<a id="ref-for-definite"></a>

<a id="ref-for-preferred-size"></a>

In all cases, the size suggestion is additionally clamped by the [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width) in the affected axis, if it’s [definite](https://www.w3.org/TR/css-sizing-3/#definite). If the item is a [compressible replaced element](https://www.w3.org/TR/css-sizing-3/#min-content-zero), and has a <a id="ref-for-definite①"></a>definite [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) or <a id="ref-for-max-width①"></a>maximum size in the relevant axis, the size suggestion is capped by those sizes; for this purpose, any indefinite percentages in these sizes are resolved against zero (and considered <a id="ref-for-definite②"></a>definite).

<a id="ref-for-funcdef-grid-template-columns-fit-content"></a>

<a id="ref-for-content-based-minimum-size②"></a>

<a id="ref-for-fixed-sizing-function②"></a>

<a id="ref-for-max-track-sizing-function②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The argument to [fit-content()](#funcdef-grid-template-columns-fit-content) does <em>not</em> clamp the [content-based minimum size](#content-based-minimum-size) in the same way as a [fixed](#fixed-sizing-function) [max track sizing function](#max-track-sizing-function).

<a id="ref-for-content-size-suggestion②"></a>

<a id="ref-for-specified-size-suggestion②"></a>

<a id="ref-for-transferred-size-suggestion②"></a>

<a id="ref-for-content-based-minimum-size③"></a>

The [content size suggestion](#content-size-suggestion), [specified size suggestion](#specified-size-suggestion), and [transferred size suggestion](#transferred-size-suggestion) used in this calculation account for the relevant min/max/preferred size properties so that the [content-based minimum size](#content-based-minimum-size) does not interfere with any author-provided constraints, and are defined below:

<a id="specified-size-suggestion"></a>specified size suggestion  
<a id="ref-for-specified-size-suggestion③"></a>

<a id="ref-for-definite③"></a>

<a id="ref-for-preferred-size①"></a>

If the item’s [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) in the relevant axis is [definite](https://www.w3.org/TR/css-sizing-3/#definite), then the [specified size suggestion](#specified-size-suggestion) is that size. It is otherwise undefined.

<a id="transferred-size-suggestion"></a>transferred size suggestion  
<a id="ref-for-max-width②"></a>

<a id="ref-for-min-width①"></a>

<a id="ref-for-transferred-size-suggestion③"></a>

<a id="ref-for-definite④"></a>

<a id="ref-for-preferred-size②"></a>

<a id="ref-for-preferred-aspect-ratio②"></a>

If the item has a [preferred aspect ratio](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio) and its [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) in the opposite axis is [definite](https://www.w3.org/TR/css-sizing-3/#definite), then the [transferred size suggestion](#transferred-size-suggestion) is that size (clamped by the opposite-axis [minimum](https://www.w3.org/TR/css-sizing-3/#min-width) and [maximum sizes](https://www.w3.org/TR/css-sizing-3/#max-width) if they are <a id="ref-for-definite⑤"></a>definite), converted through the aspect ratio. It is otherwise undefined.

<a id="ref-for-definite⑥"></a>

<a id="ref-for-preferred-size③"></a>

<a id="ref-for-max-width③"></a>

<a id="ref-for-transferred-size-suggestion④"></a>

If the item has a [definite](https://www.w3.org/TR/css-sizing-3/#definite) [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) or [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width) in the relevant axis, the [transferred size suggestion](#transferred-size-suggestion) is capped by those sizes; for this purpose, any indefinite percentages in these sizes are resolved against zero (and considered <a id="ref-for-definite⑦"></a>definite).

<a id="content-size-suggestion"></a>content size suggestion  
<a id="ref-for-max-width④"></a>

<a id="ref-for-min-width②"></a>

<a id="ref-for-definite⑧"></a>

<a id="ref-for-preferred-aspect-ratio③"></a>

<a id="ref-for-min-content①"></a>

<a id="ref-for-content-size-suggestion③"></a>

The [content size suggestion](#content-size-suggestion) is the [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content) in the relevant axis, clamped, if it has a [preferred aspect ratio](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio), by any [definite](https://www.w3.org/TR/css-sizing-3/#definite) opposite-axis [minimum](https://www.w3.org/TR/css-sizing-3/#min-width) and [maximum sizes](https://www.w3.org/TR/css-sizing-3/#max-width) converted through the aspect ratio.

<a id="ref-for-min-content②"></a>

<a id="ref-for-content-based-minimum-size④"></a>

<a id="ref-for-propdef-width"></a>

<a id="ref-for-definite⑨"></a>

<a id="ref-for-behave-as-auto"></a>

For the purpose of calculating an intrinsic size of the box (e.g. the box’s [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content)), a [content-based minimum size](#content-based-minimum-size) causes the box’s size in that axis to become indefinite (even if e.g. its [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) property specifies a [definite](https://www.w3.org/TR/css-sizing-3/#definite) size). Note this means that percentages calculated against this size will [behave as auto](https://www.w3.org/TR/css-sizing-3/#behave-as-auto).

<a id="ref-for-content-based-minimum-size⑤"></a>

<a id="ref-for-valdef-grid-template-columns-min-content"></a>

<a id="ref-for-min-width③"></a>

For any purpose <em>other than</em> calculating intrinsic sizes, a [content-based minimum size](#content-based-minimum-size) (unlike an explicit [min-content](#valdef-grid-template-columns-min-content)/etc [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width)) does not force the box’s size to become indefinite. However, if a percentage resolved against the box’s size <em>before</em> this minimum was applied, it must be re-resolved against the new size after it is applied.

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="min-size-opt"></a> Note that while a content-based minimum size is often appropriate, and helps prevent content from overlapping or spilling outside its container, in some cases it is not:
>
> <a id="ref-for-propdef-min-width①"></a>
>
> In particular, if grid layout is being used for a major content area of a document, it is better to set an explicit font-relative minimum width such as [min-width: 12em](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width). A content-based minimum width could result in a large table or large image stretching the size of the entire content area, potentially into an overflow zone, and thereby making lines of text needlessly long and hard to read.
>
> Note also, when content-based sizing is used on an item with large amounts of content, the layout engine must traverse all of this content before finding its minimum size, whereas if the author sets an explicit minimum, this is not necessary. (For items with small amounts of content, however, this traversal is trivial and therefore not a performance concern.)

## <a id="grid-definition"></a>7.  Defining the Grid

### <a id="explicit-grids"></a>7.1.  The Explicit Grid

<a id="ref-for-propdef-grid-template-rows①"></a>

<a id="ref-for-propdef-grid-template-columns①"></a>

<a id="ref-for-propdef-grid-template-areas①"></a>

<a id="ref-for-grid-container③②"></a>

<a id="ref-for-grid-item④①"></a>

<a id="ref-for-explicit-grid②"></a>

<a id="ref-for-propdef-grid-auto-rows"></a>

<a id="ref-for-propdef-grid-auto-columns"></a>

The three properties [grid-template-rows](#propdef-grid-template-rows), [grid-template-columns](#propdef-grid-template-columns), and [grid-template-areas](#propdef-grid-template-areas) together define the <a id="explicit-grid"></a>explicit grid of a [grid container](#grid-container) by specifying its <a id="explicit-grid-track"></a>explicit grid tracks. The final grid may end up larger due to [grid items](#grid-item) placed outside the [explicit grid](#explicit-grid); in this case implicit tracks will be created, these implicit tracks will be sized by the [grid-auto-rows](#propdef-grid-auto-rows) and [grid-auto-columns](#propdef-grid-auto-columns) properties.

<a id="ref-for-explicit-grid③"></a>

<a id="ref-for-propdef-grid-template-areas②"></a>

<a id="ref-for-propdef-grid-template-rows②"></a>

<a id="ref-for-propdef-grid-template-columns②"></a>

<a id="ref-for-propdef-grid-auto-rows①"></a>

<a id="ref-for-propdef-grid-auto-columns①"></a>

<a id="ref-for-grid-line①④"></a>

The size of the [explicit grid](#explicit-grid) is determined by the larger of the number of rows/columns defined by [grid-template-areas](#propdef-grid-template-areas) and the number of rows/columns sized by [grid-template-rows](#propdef-grid-template-rows)/[grid-template-columns](#propdef-grid-template-columns). Any rows/columns defined by <a id="ref-for-propdef-grid-template-areas③"></a>grid-template-areas but not sized by <a id="ref-for-propdef-grid-template-rows③"></a>grid-template-rows/<a id="ref-for-propdef-grid-template-columns③"></a>grid-template-columns take their size from the [grid-auto-rows](#propdef-grid-auto-rows)/[grid-auto-columns](#propdef-grid-auto-columns) properties. If these properties don’t define <em>any</em> <a id="ref-for-explicit-grid④"></a>explicit tracks the <a id="ref-for-explicit-grid⑤"></a>explicit grid still contains one [grid line](#grid-line) in each axis.

<a id="ref-for-grid-placement-property⑥"></a>

<a id="ref-for-explicit-grid⑥"></a>

<a id="ref-for-start"></a>

<a id="ref-for-end"></a>

Numeric indexes in the [grid-placement properties](#grid-placement-property) count from the edges of the [explicit grid](#explicit-grid). Positive indexes count from the [start](https://www.w3.org/TR/css-writing-modes-4/#start) side (starting from 1 for the <a id="ref-for-start①"></a>start-most <a id="ref-for-explicit-grid⑦"></a>explicit line), while negative indexes count from the [end](https://www.w3.org/TR/css-writing-modes-4/#end) side (starting from -1 for the <a id="ref-for-end①"></a>end-most <a id="ref-for-explicit-grid⑧"></a>explicit line).

<a id="ref-for-propdef-grid①"></a>

<a id="ref-for-propdef-grid-template"></a>

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-propdef-grid-template-rows④"></a>

<a id="ref-for-propdef-grid-template-columns④"></a>

<a id="ref-for-propdef-grid-template-areas④"></a>

<a id="ref-for-implicit-grid①"></a>

The [grid](#propdef-grid) and [grid-template](#propdef-grid-template) properties are [shorthands](https://www.w3.org/TR/css-cascade-5/#shorthand-property) that can be used to set all three <a id="explicit-grid-properties"></a>explicit grid properties ([grid-template-rows](#propdef-grid-template-rows), [grid-template-columns](#propdef-grid-template-columns), and [grid-template-areas](#propdef-grid-template-areas)) at the same time. The <a id="ref-for-propdef-grid②"></a>grid shorthand also resets properties controlling the [implicit grid](#implicit-grid), whereas the <a id="ref-for-propdef-grid-template①"></a>grid-template property leaves them unchanged.

<a id="ref-for-propdef-grid-template-rows⑤"></a>

<a id="ref-for-propdef-grid-template-columns⑤"></a>

### <a id="track-sizing"></a>7.2.  Explicit Track Sizing: the [grid-template-rows](#propdef-grid-template-rows) and [grid-template-columns](#propdef-grid-template-columns) properties

<strong>Table 3 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-grid-template-columns"></a>grid-template-columns, <a id="propdef-grid-template-rows"></a>grid-template-rows

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-typedef-line-name-list"></a>

<a id="ref-for-typedef-auto-track-list"></a>

<a id="ref-for-typedef-track-list"></a>

<a id="ref-for-comb-one①"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<track-list\>](#typedef-track-list) <a id="ref-for-comb-one②"></a>\| [\<auto-track-list\>](#typedef-auto-track-list) <a id="ref-for-comb-one③"></a>\| subgrid [\<line-name-list\>](#typedef-line-name-list)[?](https://www.w3.org/TR/css-values-4/#mult-opt)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-container③③"></a>

[grid containers](#grid-container)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

refer to corresponding dimension of the content area

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-computed-track-list"></a>

<a id="ref-for-valdef-grid-template-rows-none"></a>

the keyword [none](#valdef-grid-template-rows-none) or a [computed track list](#computed-track-list)

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-computed-track-list①"></a>

if the list lengths match, by computed value type per item in the [computed track list](#computed-track-list) (see [§ 7.2.5 Computed Value of a Track Listing](#computed-tracks) and [§ 7.2.3.3 Interpolation/Combination of repeat()](#repeat-interpolation)); discrete otherwise

<a id="ref-for-line-name"></a>

<a id="ref-for-grid-template-rows-track-sizing-function"></a>

<a id="ref-for-grid⑨"></a>

<a id="ref-for-propdef-grid-template-columns⑥"></a>

<a id="ref-for-track-list"></a>

<a id="ref-for-propdef-grid-template-rows⑥"></a>

These properties specify, as a space-separated <a id="track-list"></a>track list, the [line names](#line-name) and [track sizing functions](#grid-template-rows-track-sizing-function) of the [grid](#grid). The [grid-template-columns](#propdef-grid-template-columns) property specifies the [track list](#track-list) for the grid’s columns, while [grid-template-rows](#propdef-grid-template-rows) specifies the <a id="ref-for-track-list①"></a>track list for the grid’s rows.

Values have the following meanings:

<a id="valdef-grid-template-rows-none"></a>none

<a id="ref-for-propdef-grid-template-areas⑤"></a>

<a id="ref-for-explicit-grid⑨"></a>

Indicates that no [explicit](#explicit-grid) grid tracks are created by this property (though <a id="ref-for-explicit-grid①⓪"></a>explicit grid tracks could still be created by [grid-template-areas](#propdef-grid-template-areas)).

<a id="ref-for-explicit-grid①①"></a>

<a id="ref-for-propdef-grid-auto-rows②"></a>

<a id="ref-for-propdef-grid-auto-columns②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In the absence of an [explicit grid](#explicit-grid) any rows/columns will be [implicitly generated](#implicit-grids), and their size will be determined by the [grid-auto-rows](#propdef-grid-auto-rows) and [grid-auto-columns](#propdef-grid-auto-columns) properties.

<a id="ref-for-typedef-auto-track-list①"></a>

<a id="ref-for-typedef-track-list①"></a>

<a id="track-listing"></a>[\<track-list\>](#typedef-track-list) \| [\<auto-track-list\>](#typedef-auto-track-list)

<a id="ref-for-max-track-sizing-function③"></a>

<a id="ref-for-min-track-sizing-function①"></a>

<a id="ref-for-funcdef-grid-template-columns-minmax"></a>

<a id="ref-for-grid-container③④"></a>

<a id="ref-for-line-name①"></a>

<a id="ref-for-grid-template-rows-track-sizing-function①"></a>

<a id="ref-for-track-list②"></a>

Specifies the [track list](#track-list) as a series of [track sizing functions](#grid-template-rows-track-sizing-function) and [line names](#line-name). Each <a id="grid-template-rows-track-sizing-function"></a>track sizing function can be specified as a length, a percentage of the [grid container](#grid-container)’s size, a measurement of the contents occupying the column or row, or a fraction of the free space in the grid. It can also be specified as a range using the [minmax()](#funcdef-grid-template-columns-minmax) notation, which can combine any of the previously mentioned mechanisms to specify separate [min](#min-track-sizing-function) and [max track sizing functions](#max-track-sizing-function) for the column or row.

<a id="ref-for-typedef-line-name-list①"></a>

<a id="subgrid-listing"></a>subgrid [\<line-name-list\>](#typedef-line-name-list)?

<a id="ref-for-subgrid⑤"></a>

<a id="ref-for-parent-grid①"></a>

The <a id="valdef-grid-template-rows-subgrid"></a>subgrid value indicates that the grid will adopt the spanned portion of its [parent grid](#parent-grid) in that axis (the <a id="subgridded-axis"></a>subgridded axis). Rather than being specified explicitly, the sizes of the grid rows/columns will be taken from the <a id="ref-for-parent-grid②"></a>parent grid’s definition, and the [subgrid](#subgrid)’s items will participate in the [intrinsic size calculations](https://www.w3.org/TR/css-grid-1/#algo-content) ([CSS Grid Layout 1 § 11.5 Resolve Intrinsic Track Sizes](https://www.w3.org/TR/css-grid-1/#algo-content)) of any tracks shared with the <a id="ref-for-parent-grid③"></a>parent grid. Essentially, <a id="ref-for-subgrid⑥"></a>subgrids provide the ability to pass grid parameters down through nested elements, and content-based sizing information back up to their <a id="ref-for-parent-grid④"></a>parent grid.

<a id="ref-for-typedef-line-name-list②"></a>

<a id="ref-for-parent-grid⑤"></a>

<a id="ref-for-typedef-line-names"></a>

<a id="ref-for-subgrid⑦"></a>

<a id="ref-for-explicit-grid①②"></a>

The [\<line-name-list\>](#typedef-line-name-list) argument allows local naming of the grid lines shared with the [parent grid](#parent-grid): if a <a id="ref-for-typedef-line-name-list③"></a>\<line-name-list\> is given, the specified [\<line-names\>](#typedef-line-names)s are assigned to the lines of the [subgrid](#subgrid)’s [explicit grid](#explicit-grid), one per line, starting with line 1. Excess <a id="ref-for-typedef-line-names①"></a>\<line-names\> are ignored.

<a id="ref-for-parent-grid⑥"></a>

<a id="ref-for-grid-container③⑤"></a>

<a id="ref-for-independent-formatting-context②"></a>

<a id="ref-for-layout-containment"></a>

<a id="ref-for-absolute-position"></a>

<a id="ref-for-used-value"></a>

<a id="ref-for-valdef-grid-template-rows-none①"></a>

<a id="ref-for-subgrid⑧"></a>

If there is no [parent grid](#parent-grid), or if the [grid container](#grid-container) is otherwise forced to establish an [independent formatting context](https://www.w3.org/TR/css-display-4/#independent-formatting-context) (for example, due to [layout containment](https://www.w3.org/TR/css-contain-2/#layout-containment) [\[CSS-CONTAIN-2\]](#biblio-css-contain-2) or [absolute positioning](https://www.w3.org/TR/css-position-3/#absolute-position) [\[CSS-POSITION-3\]](#biblio-css-position-3)), the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) is the initial value, [none](#valdef-grid-template-rows-none), and the <a id="ref-for-grid-container③⑥"></a>grid container is not a [subgrid](#subgrid).

<a id="ref-for-subgridded-axis②"></a>

An axis that is not [subgridded](#subgridded-axis) is a <a id="standalone-axis"></a>standalone axis.

<a id="ref-for-track-list③"></a>

The syntax of a [track list](#track-list) is:

<a id="typedef-track-list"></a>

<a id="ref-for-typedef-line-names②"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-typedef-track-size"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-typedef-track-repeat"></a>

<a id="ref-for-mult-one-plus"></a>

<a id="ref-for-typedef-line-names③"></a>

<a id="ref-for-mult-opt②"></a>

<a id="typedef-auto-track-list"></a>

<a id="ref-for-typedef-line-names④"></a>

<a id="ref-for-mult-opt③"></a>

<a id="ref-for-typedef-fixed-size"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-typedef-fixed-repeat"></a>

<a id="ref-for-mult-zero-plus"></a>

<a id="ref-for-typedef-line-names⑤"></a>

<a id="ref-for-mult-opt④"></a>

<a id="ref-for-typedef-auto-repeat"></a>

<a id="ref-for-typedef-line-names⑥"></a>

<a id="ref-for-mult-opt⑤"></a>

<a id="ref-for-typedef-fixed-size①"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-typedef-fixed-repeat①"></a>

<a id="ref-for-mult-zero-plus①"></a>

<a id="ref-for-typedef-line-names⑦"></a>

<a id="ref-for-mult-opt⑥"></a>

<a id="typedef-explicit-track-list"></a>

<a id="ref-for-typedef-line-names⑧"></a>

<a id="ref-for-mult-opt⑦"></a>

<a id="ref-for-typedef-track-size①"></a>

<a id="ref-for-mult-one-plus①"></a>

<a id="ref-for-typedef-line-names⑨"></a>

<a id="ref-for-mult-opt⑧"></a>

<a id="typedef-line-name-list"></a>

<a id="ref-for-typedef-line-names①⓪"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-typedef-name-repeat"></a>

<a id="ref-for-mult-one-plus②"></a>

<a id="typedef-track-size"></a>

<a id="ref-for-typedef-track-breadth"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-typedef-inflexible-breadth"></a>

<a id="ref-for-comb-comma"></a>

<a id="ref-for-typedef-track-breadth①"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-typedef-length-percentage"></a>

<a id="typedef-fixed-size"></a>

<a id="ref-for-typedef-fixed-breadth"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-typedef-fixed-breadth①"></a>

<a id="ref-for-comb-comma①"></a>

<a id="ref-for-typedef-track-breadth②"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-typedef-inflexible-breadth①"></a>

<a id="ref-for-comb-comma②"></a>

<a id="ref-for-typedef-fixed-breadth②"></a>

<a id="typedef-track-breadth"></a>

<a id="ref-for-typedef-length-percentage①"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-typedef-flex"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="typedef-inflexible-breadth"></a>

<a id="ref-for-typedef-length-percentage②"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="typedef-fixed-breadth"></a>

<a id="ref-for-typedef-length-percentage③"></a>

<a id="typedef-line-names"></a>

<a id="ref-for-identifier-value"></a>

<a id="ref-for-mult-zero-plus②"></a>

```text
<track-list>          = [ <line-names>? [ <track-size> | <track-repeat> ] ]+ <line-names>?
<auto-track-list>     = [ <line-names>? [ <fixed-size> | <fixed-repeat> ] ]* <line-names>? <auto-repeat>
                        [ <line-names>? [ <fixed-size> | <fixed-repeat> ] ]* <line-names>?
<explicit-track-list> = [ <line-names>? <track-size> ]+ <line-names>?

<line-name-list>      = [ <line-names> | <name-repeat> ]+
<track-size>          = <track-breadth> | minmax( <inflexible-breadth> , <track-breadth> ) | fit-content( <length-percentage [0,∞]> )
<fixed-size>          = <fixed-breadth> | minmax( <fixed-breadth> , <track-breadth> ) | minmax( <inflexible-breadth> , <fixed-breadth> )
<track-breadth>       = <length-percentage [0,∞]> | <flex [0,∞]> | min-content | max-content | auto
<inflexible-breadth>  = <length-percentage [0,∞]> | min-content | max-content | auto
<fixed-breadth>       = <length-percentage [0,∞]>
<line-names>          = '[' <custom-ident>* ']'
```
Where the component values are defined as follows…

#### <a id="track-sizes"></a>7.2.1.  Track Sizes

<a id="ref-for-typedef-length-percentage④"></a>

<a id="valdef-grid-template-columns-length-percentage-0"></a>[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)

A non-negative length or percentage, as defined by CSS3 Values. [\[CSS-VALUES-3\]](#biblio-css-values-3)

<a id="ref-for-percentage-value"></a>

<a id="ref-for-inner-size"></a>

<a id="ref-for-inline-size③"></a>

<a id="ref-for-grid-container③⑦"></a>

<a id="ref-for-grid-track①⓪"></a>

<a id="ref-for-block-size①"></a>

<a id="ref-for-valdef-width-auto①"></a>

<a id="ref-for-grid①⓪"></a>

[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) values are relative to the [inner](https://www.w3.org/TR/css-sizing-3/#inner-size) [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) of the [grid container](#grid-container) in column [grid tracks](#grid-track), and the <a id="ref-for-inner-size①"></a>inner [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) of the <a id="ref-for-grid-container③⑧"></a>grid container in row <a id="ref-for-grid-track①①"></a>grid tracks. If the size of the <a id="ref-for-grid-container③⑨"></a>grid container depends on the size of its tracks, then the <a id="ref-for-percentage-value①"></a>\<percentage\> must be treated as [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), for the purpose of calculating the intrinsic sizes of the <a id="ref-for-grid-container④⓪"></a>grid container and then resolve against that resulting <a id="ref-for-grid-container④①"></a>grid container size for the purpose of laying out the [grid](#grid) and its items.

<a id="ref-for-typedef-flex①"></a>

<a id="valdef-grid-template-columns-flex-0"></a>[\<flex \[0,∞\]\>](#typedef-flex)

<a id="ref-for-leftover-space"></a>

<a id="ref-for-grid-template-columns-flex-factor"></a>

<a id="ref-for-typedef-flex②"></a>

<a id="ref-for-valdef-flex-fr②"></a>

A non-negative dimension with the unit [fr](#valdef-flex-fr) specifying the track’s <a id="grid-template-columns-flex-factor"></a>flex factor. Each [\<flex\>](#typedef-flex)-sized track takes a share of the remaining space in proportion to its [flex factor](#grid-template-columns-flex-factor). For example, given a track listing of 1fr 2fr, the tracks will take up ⅓ and ⅔ of the [leftover space](#leftover-space), respectively. See [§ 7.2.4 Flexible Lengths: the fr unit](#fr-unit) for more details.

<a id="ref-for-grid-template-columns-flex-factor①"></a>

<a id="ref-for-leftover-space①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If the sum of the [flex factors](#grid-template-columns-flex-factor) is less than 1, they’ll take up only a corresponding fraction of the [leftover space](#leftover-space), rather than expanding to fill the entire thing.

<a id="ref-for-funcdef-grid-template-columns-minmax①"></a>

<a id="ref-for-typedef-flex③"></a>

When appearing outside a [minmax()](#funcdef-grid-template-columns-minmax) notation, implies an automatic minimum (i.e. minmax(auto, [\<flex\>](#typedef-flex))).

<a id="funcdef-grid-template-columns-minmax"></a>minmax(<var>min</var>, <var>max</var>)

<a id="ref-for-grid-template-columns-flex-factor②"></a>

<a id="ref-for-typedef-flex④"></a>

Defines a size range greater than or equal to <var>min</var> and less than or equal to <var>max</var>. If the <var>max</var> is less than the <var>min</var>, then the <var>max</var> will be floored by the <var>min</var> (essentially yielding minmax(<var>min</var>, <var>min</var>)). As a maximum, a [\<flex\>](#typedef-flex) value sets the track’s [flex factor](#grid-template-columns-flex-factor); it is invalid as a minimum.

<a id="ref-for-typedef-flex⑤"></a>

<a id="ref-for-track-sizing-algorithm"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A future level of this spec may allow [\<flex\>](#typedef-flex) minimums, and will update the [track sizing algorithm](#track-sizing-algorithm) to account for this correctly

<a id="valdef-grid-template-columns-auto"></a>auto

<a id="ref-for-propdef-justify-content①"></a>

<a id="ref-for-propdef-align-content①"></a>

<a id="ref-for-valdef-grid-template-columns-max-content"></a>

<a id="ref-for-grid-track①②"></a>

<a id="ref-for-grid-item④②"></a>

<a id="ref-for-max-content-contribution"></a>

As a <em>maximum</em>: represents the largest [max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) of the [grid items](#grid-item) occupying the [grid track](#grid-track); however, unlike [max-content](#valdef-grid-template-columns-max-content), allows expansion of the track by the [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) and [justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content) properties.

<a id="ref-for-min-width④"></a>

<a id="ref-for-propdef-min-width②"></a>

<a id="ref-for-propdef-min-height①"></a>

<a id="ref-for-grid-item④③"></a>

<a id="ref-for-grid-track①③"></a>

<a id="ref-for-valdef-grid-template-columns-min-content①"></a>

As a <em>minimum</em>: represents the largest [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width) (specified by [min-width](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width)/[min-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-height)) of the [grid items](#grid-item) occupying the [grid track](#grid-track). (This initially is often, but not always, equal to a [min-content](#valdef-grid-template-columns-min-content) minimum—​see [§ 6.6 Automatic Minimum Size of Grid Items](#min-size-auto).)

<a id="ref-for-funcdef-grid-template-columns-minmax②"></a>

When appearing outside a [minmax()](#funcdef-grid-template-columns-minmax) notation: equivalent to minmax(auto, auto), representing the range between the minimum and maximum described above. (This behaves similar to minmax(min-content, max-content) in the most basic cases, but with extra abilities.)

<a id="valdef-grid-template-columns-max-content"></a>max-content

<a id="ref-for-grid-track①④"></a>

<a id="ref-for-grid-item④④"></a>

<a id="ref-for-max-content-contribution①"></a>

Represents the largest [max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) of the [grid items](#grid-item) occupying the [grid track](#grid-track).

<a id="valdef-grid-template-columns-min-content"></a>min-content

<a id="ref-for-grid-track①⑤"></a>

<a id="ref-for-grid-item④⑤"></a>

<a id="ref-for-min-content-contribution"></a>

Represents the largest [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) of the [grid items](#grid-item) occupying the [grid track](#grid-track).

<a id="ref-for-typedef-length-percentage⑤"></a>

<a id="funcdef-grid-template-columns-fit-content"></a>fit-content( [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) )

<a id="ref-for-funcdef-grid-template-columns-fit-content①"></a>

<a id="ref-for-grid-template-rows-track-sizing-function②"></a>

<a id="ref-for-valdef-grid-template-columns-min-content②"></a>

<a id="ref-for-valdef-grid-template-columns-auto③"></a>

<a id="ref-for-valdef-grid-template-columns-max-content①"></a>

Represents the formula <code>max(<var>minimum</var>, min(<var>limit</var>, <a href="#valdef-grid-template-columns-max-content">max-content</a>))</code>, where <var>minimum</var> represents an [auto](#valdef-grid-template-columns-auto) minimum (which is often, but not always, equal to a [min-content](#valdef-grid-template-columns-min-content) minimum), and <var>limit</var> is the [track sizing function](#grid-template-rows-track-sizing-function) passed as an argument to [fit-content()](#funcdef-grid-template-columns-fit-content). This is essentially calculated as the smaller of minmax(auto, max-content) and minmax(auto, <var>limit</var>).

<a id="ref-for-propdef-grid-template-columns⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6651406f"></a> Given the following [grid-template-columns](#propdef-grid-template-columns) declaration:
>
> ```text
> grid-template-columns: 100px 1fr max-content minmax(min-content, 1fr);
> ```
>
> Five grid lines are created:
>
> 1.  <a id="ref-for-grid-container④②"></a>
>
>     At the start edge of the [grid container](#grid-container).
>
> 2.  <a id="ref-for-grid-container④③"></a>
>
>     100px from the start edge of the [grid container](#grid-container).
>
> 3.  <a id="ref-for-grid-track①⑥"></a>
>
>     <a id="ref-for-grid-container④④"></a>
>
>     <a id="ref-for-free-space"></a>
>
>     A distance from the previous line equal to half the [free space](#free-space) (the width of the [grid container](#grid-container), minus the width of the non-flexible [grid tracks](#grid-track)).
>
> 4.  <a id="ref-for-grid-item④⑥"></a>
>
>     A distance from the previous line equal to the maximum size of any [grid items](#grid-item) belonging to the column between these two lines.
>
> 5.  <a id="ref-for-free-space①"></a>
>
>     <a id="ref-for-grid-item④⑦"></a>
>
>     A distance from the previous line at least as large as the largest minimum size of any [grid items](#grid-item) belonging to the column between these two lines, but no larger than the other half of the [free space](#free-space).
>
> <a id="ref-for-valdef-grid-template-columns-max-content②"></a>
>
> <a id="ref-for-valdef-grid-template-columns-min-content③"></a>
>
> <a id="ref-for-grid-container④⑤"></a>
>
> <a id="ref-for-grid-line①⑤"></a>
>
> <a id="ref-for-typedef-flex⑥"></a>
>
> <a id="ref-for-grid-track①⑦"></a>
>
> If the non-flexible sizes (100px, [max-content](#valdef-grid-template-columns-max-content), and [min-content](#valdef-grid-template-columns-min-content)) sum to larger than the [grid container](#grid-container)’s width, the final [grid line](#grid-line) will be a distance equal to their sum away from the start edge of the <a id="ref-for-grid-container④⑥"></a>grid container (the 1fr sizes both resolve to 0). If the sum is less than the <a id="ref-for-grid-container④⑦"></a>grid container’s width, the final <a id="ref-for-grid-line①⑥"></a>grid line will be exactly at the end edge of the <a id="ref-for-grid-container④⑧"></a>grid container. This is true in general whenever there’s at least one [\<flex\>](#typedef-flex) value among the [grid track](#grid-track) sizes.

<a id="ref-for-grid-track①⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-337b574e"></a> Additional examples of valid [grid track](#grid-track) definitions:
>
> ```text
> /* examples of valid track definitions */
> grid-template-rows: 1fr minmax(min-content, 1fr);
> grid-template-rows: 10px repeat(2, 1fr auto minmax(30%, 1fr));
> grid-template-rows: calc(4em - 5px);
> ```
<a id="ref-for-propdef-row-gap"></a>

<a id="ref-for-propdef-column-gap"></a>

<a id="ref-for-propdef-justify-content②"></a>

<a id="ref-for-propdef-align-content②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The size of the grid is not purely the sum of the track sizes, as [row-gap](https://www.w3.org/TR/css-align-3/#propdef-row-gap), [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap) and [justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content), [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) can add additional space between tracks.

<a id="ref-for-identifier-value①"></a>

#### <a id="named-lines"></a>7.2.2.  Naming Grid Lines: the \[[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)\*\] syntax

<a id="ref-for-grid-line①⑦"></a>

<a id="ref-for-grid-placement-property⑦"></a>

<a id="ref-for-line-name②"></a>

<a id="ref-for-propdef-grid-template-rows⑦"></a>

<a id="ref-for-propdef-grid-template-columns⑧"></a>

<a id="ref-for-implicitly-assigned-line-name"></a>

<a id="ref-for-named-grid-area"></a>

<a id="ref-for-propdef-grid-template-areas⑥"></a>

While [grid lines](#grid-line) can always be referred to by their numerical index, <a id="line-name"></a>line names can make the [grid-placement properties](#grid-placement-property) easier to understand and maintain. [Line names](#line-name) can be <a id="explicitly-assigned-line-name"></a>explicitly assigned with the [grid-template-rows](#propdef-grid-template-rows) and [grid-template-columns](#propdef-grid-template-columns) properties, or [implicitly assigned](#implicitly-assigned-line-name) by [named grid areas](#named-grid-area) with the [grid-template-areas](#propdef-grid-template-areas) property.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1cbfee50"></a> For example, the following code gives meaningful names to all of the lines in the grid. Note that some of the lines have multiple names.
>
> ```text
> #grid {
>   display: grid;
>   grid-template-columns: [first nav-start] 150px [main-start] 1fr [last];
>   grid-template-rows: [first header-start] 50px [main-start] 1fr [footer-start] 50px [last];
> }
> ```
>
> ![Image: Named Grid Lines.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/grid-named-lines.png)
>
> Named Grid Lines.

<a id="ref-for-line-name③"></a>

<a id="ref-for-identifier-value②"></a>

<a id="ref-for-typedef-line-names①①"></a>

A [line name](#line-name) cannot be span or auto, i.e. the [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) in the [\<line-names\>](#typedef-line-names) production excludes the keywords span and auto.

<a id="ref-for-funcdef-track-repeat-repeat"></a>

#### <a id="repeat-notation"></a>7.2.3.  Repeating Rows and Columns: the [repeat()](#funcdef-track-repeat-repeat) notation

<a id="ref-for-track-list④"></a>

The <a id="funcdef-track-repeat-repeat"></a>repeat() notation represents a repeated fragment of the [track list](#track-list), allowing a large number of columns or rows that exhibit a recurring pattern to be written in a more compact form.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e3dc5d22"></a> This example shows two equivalent ways of writing the same grid definition. Both declarations produce four “main” columns, each 250px wide, surrounded by 10px “gutter” columns.
>
> ```text
> grid-template-columns: 10px [col-start] 250px [col-end]
>                        10px [col-start] 250px [col-end]
>                        10px [col-start] 250px [col-end]
>                        10px [col-start] 250px [col-end] 10px;
> /* same as above, except easier to write */
> grid-template-columns: repeat(4, 10px [col-start] 250px [col-end]) 10px;
> ```
<a id="ref-for-funcdef-track-repeat-repeat①"></a>

##### <a id="repeat-syntax"></a>7.2.3.1.  Syntax of [repeat()](#funcdef-track-repeat-repeat)

<a id="ref-for-funcdef-track-repeat-repeat②"></a>

The generic form of the [repeat()](#funcdef-track-repeat-repeat) syntax is, approximately,

<a id="ref-for-integer-value"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-comb-comma③"></a>

<a id="ref-for-typedef-track-list②"></a>

```text
repeat( [ <integer [1,∞]> | auto-fill | auto-fit ] , <track-list> )
```
<a id="ref-for-track-list⑤"></a>

The first argument specifies the number of repetitions. The second argument is a [track list](#track-list), which is repeated that number of times. However, there are some restrictions:

- <a id="ref-for-funcdef-track-repeat-repeat③"></a>

  The [repeat()](#funcdef-track-repeat-repeat) notation can’t be nested.

- <a id="ref-for-valdef-repeat-auto-fill"></a>

  <a id="ref-for-valdef-repeat-auto-fit"></a>

  <a id="ref-for-intrinsic-sizing-function①"></a>

  <a id="ref-for-flexible-sizing-function"></a>

  Automatic repetitions ([auto-fill](#valdef-repeat-auto-fill) or [auto-fit](#valdef-repeat-auto-fit)) cannot be combined with fully [intrinsic](#intrinsic-sizing-function) or [flexible](#flexible-sizing-function) sizes (see grammar).

<a id="ref-for-funcdef-track-repeat-repeat④"></a>

Thus the precise syntax of the [repeat()](#funcdef-track-repeat-repeat) notation has several forms:

<a id="typedef-track-repeat"></a>

<a id="ref-for-typedef-track-repeat①"></a>

<a id="ref-for-integer-value①"></a>

<a id="ref-for-comb-comma④"></a>

<a id="ref-for-typedef-line-names①②"></a>

<a id="ref-for-mult-opt⑨"></a>

<a id="ref-for-typedef-track-size②"></a>

<a id="ref-for-mult-one-plus③"></a>

<a id="ref-for-typedef-line-names①③"></a>

<a id="ref-for-mult-opt①⓪"></a>

<a id="typedef-auto-repeat"></a>

<a id="ref-for-typedef-auto-repeat①"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-comb-comma⑤"></a>

<a id="ref-for-typedef-line-names①④"></a>

<a id="ref-for-mult-opt①①"></a>

<a id="ref-for-typedef-fixed-size②"></a>

<a id="ref-for-mult-one-plus④"></a>

<a id="ref-for-typedef-line-names①⑤"></a>

<a id="ref-for-mult-opt①②"></a>

<a id="typedef-fixed-repeat"></a>

<a id="ref-for-typedef-fixed-repeat②"></a>

<a id="ref-for-integer-value②"></a>

<a id="ref-for-comb-comma⑥"></a>

<a id="ref-for-typedef-line-names①⑥"></a>

<a id="ref-for-mult-opt①③"></a>

<a id="ref-for-typedef-fixed-size③"></a>

<a id="ref-for-mult-one-plus⑤"></a>

<a id="ref-for-typedef-line-names①⑦"></a>

<a id="ref-for-mult-opt①④"></a>

<a id="typedef-name-repeat"></a>

<a id="ref-for-typedef-name-repeat①"></a>

<a id="ref-for-integer-value③"></a>

<a id="ref-for-comb-one②②"></a>

<a id="ref-for-comb-comma⑦"></a>

<a id="ref-for-typedef-line-names①⑧"></a>

<a id="ref-for-mult-one-plus⑥"></a>

```text
<track-repeat> = repeat( [ <integer [1,∞]> ] , [ <line-names>? <track-size> ]+ <line-names>? )
<auto-repeat>  = repeat( [ auto-fill | auto-fit ] , [ <line-names>? <fixed-size> ]+ <line-names>? )
<fixed-repeat> = repeat( [ <integer [1,∞]> ] , [ <line-names>? <fixed-size> ]+ <line-names>? )
<name-repeat> = repeat( [ <integer [1,∞]> | auto-fill ], <line-names>+)
```
- <a id="ref-for-typedef-track-repeat②"></a>

  <a id="ref-for-typedef-track-size③"></a>

  The [\<track-repeat\>](#typedef-track-repeat) variant can represent the repetition of any [\<track-size\>](#typedef-track-size), but is limited to a fixed number of repetitions.

- <a id="ref-for-typedef-auto-repeat②"></a>

  <a id="ref-for-definite①⓪"></a>

  <a id="ref-for-track-list⑥"></a>

  <a id="ref-for-typedef-fixed-repeat③"></a>

  The [\<auto-repeat\>](#typedef-auto-repeat) variant can repeat automatically to fill a space, but requires [definite](https://www.w3.org/TR/css-sizing-3/#definite) track sizes so that the number of repetitions can be calculated. It can only appear once in the [track list](#track-list), but the same <a id="ref-for-track-list⑦"></a>track list can also contain [\<fixed-repeat\>](#typedef-fixed-repeat)s.

- <a id="ref-for-typedef-name-repeat②"></a>

  <a id="ref-for-line-name④"></a>

  <a id="ref-for-subgrid⑨"></a>

  <a id="ref-for-valdef-grid-template-rows-subgrid①"></a>

  The [\<name-repeat\>](#typedef-name-repeat) variant is for adding [line names](#line-name) to [subgrids](#subgrid). It can only be used with the [subgrid](#valdef-grid-template-rows-subgrid) keyword and cannot specify track sizes, only <a id="ref-for-line-name⑤"></a>line names.

<a id="ref-for-funcdef-track-repeat-repeat⑤"></a>

<a id="ref-for-typedef-name-repeat③"></a>

<a id="ref-for-typedef-line-names①⑨"></a>

If a [repeat()](#funcdef-track-repeat-repeat) function that is not a [\<name-repeat\>](#typedef-name-repeat) ends up placing two [\<line-names\>](#typedef-line-names) adjacent to each other, the name lists are merged. For example, repeat(2, \[a\] 1fr \[b\]) is equivalent to \[a\] 1fr \[b a\] 1fr \[b\].

<a id="ref-for-valdef-repeat-auto-fill①"></a>

<a id="ref-for-valdef-repeat-auto-fit①"></a>

##### <a id="auto-repeat"></a>7.2.3.2.  Repeat-to-fill: [auto-fill](#valdef-repeat-auto-fill) and [auto-fit](#valdef-repeat-auto-fit) repetitions

<a id="ref-for-subgridded-axis③"></a>

<a id="ref-for-valdef-repeat-auto-fill②"></a>

<a id="ref-for-typedef-line-name-list④"></a>

<a id="ref-for-subgrid①⓪"></a>

<a id="ref-for-grid-span①"></a>

On a [subgridded axis](#subgridded-axis), the [auto-fill](#valdef-repeat-auto-fill) keyword is only valid once per [\<line-name-list\>](#typedef-line-name-list), and repeats enough times for the name list to match the [subgrid](#subgrid)’s specified [grid span](#grid-span) (falling back to 0 if the span is already fulfilled).

<a id="ref-for-standalone-axis"></a>

<a id="ref-for-grid-container④⑨"></a>

<a id="ref-for-definite①①"></a>

<a id="ref-for-preferred-size④"></a>

<a id="ref-for-max-width⑤"></a>

<a id="ref-for-grid①①"></a>

<a id="ref-for-content-box"></a>

<a id="ref-for-propdef-gap"></a>

<a id="ref-for-min-width⑤"></a>

<a id="ref-for-track-list⑧"></a>

Otherwise on a [standalone axis](#standalone-axis), when <a id="valdef-repeat-auto-fill"></a>auto-fill is given as the repetition number, if the [grid container](#grid-container) has a [definite](https://www.w3.org/TR/css-sizing-3/#definite) [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) or [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width) in the relevant axis, then the number of repetitions is the largest possible positive integer that does not cause the [grid](#grid) to overflow the [content box](https://www.w3.org/TR/css-box-4/#content-box) of its <a id="ref-for-grid-container⑤⓪"></a>grid container taking [gap](https://www.w3.org/TR/css-align-3/#propdef-gap) into account; if any number of repetitions would overflow, then 1 repetition. Otherwise, if the <a id="ref-for-grid-container⑤①"></a>grid container has a <a id="ref-for-definite①②"></a>definite [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width) in the relevant axis, the number of repetitions is the smallest possible positive integer that fulfills that minimum requirement. Otherwise, the specified [track list](#track-list) repeats only once.

<a id="ref-for-max-track-sizing-function④"></a>

<a id="ref-for-definite①③"></a>

<a id="ref-for-min-track-sizing-function②"></a>

For this purpose, each track is treated as its [max track sizing function](#max-track-sizing-function) if that is [definite](https://www.w3.org/TR/css-sizing-3/#definite) or else its [min track sizing function](#min-track-sizing-function) if that is definite. If both are definite, floor the <a id="ref-for-max-track-sizing-function⑤"></a>max track sizing function by the <a id="ref-for-min-track-sizing-function③"></a>min track sizing function. If neither are definite, the number of repetitions is one.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5bb26bd3"></a> For example, the following code will create as many 25-character columns as will fit into the window width. If there is any remaining space, it will be distributed among the 25-character columns.
>
> ```text
> body {
>   display: grid;
>   grid-template-columns: repeat(auto-fill, minmax(25ch, 1fr));
> }
> ```
<a id="ref-for-valdef-repeat-auto-fill③"></a>

<a id="ref-for-collapsed-grid-track"></a>

<a id="ref-for-in-flow②"></a>

The <a id="valdef-repeat-auto-fit"></a>auto-fit keyword behaves the same as [auto-fill](#valdef-repeat-auto-fill), except that after [grid item placement](#auto-placement-algo) any empty repeated tracks are [collapsed](#collapsed-grid-track). An empty track is one with no [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) grid items placed into or spanning across it. (This can result in <em>all</em> tracks being <a id="ref-for-collapsed-grid-track①"></a>collapsed, if they’re all empty.)

<a id="ref-for-grid-template-rows-track-sizing-function③"></a>

<a id="ref-for-gutter②"></a>

<a id="ref-for-distributed-alignment"></a>

<a id="ref-for-collapsed-gutter"></a>

A <a id="collapsed-grid-track"></a>collapsed grid track is treated as having a fixed [track sizing function](#grid-template-rows-track-sizing-function) of 0px, and the [gutters](https://www.w3.org/TR/css-align-3/#gutter) on either side of it—​including any space allotted through [distributed alignment](https://www.w3.org/TR/css-align-3/#distributed-alignment)—​[collapse](#collapsed-gutter).

<a id="ref-for-standalone-axis①"></a>

For the purpose of finding the number of auto-repeated tracks in a [standalone axis](#standalone-axis), the UA must floor the track size to a UA-specified value to avoid division by zero. It is suggested that this floor be 1px.

<a id="ref-for-funcdef-track-repeat-repeat⑥"></a>

##### <a id="repeat-interpolation"></a>7.2.3.3.  Interpolation/Combination of [repeat()](#funcdef-track-repeat-repeat)

<a id="ref-for-funcdef-track-repeat-repeat⑦"></a>

<a id="ref-for-computed-track-list②"></a>

<a id="ref-for-by-computed-value"></a>

<a id="ref-for-discrete"></a>

If two [repeat()](#funcdef-track-repeat-repeat) notations that have the same first argument (repetition count) and the same number of tracks in their second argument (the track listing), they are combined by combining each component of their [computed track lists](#computed-track-list) [by computed value](https://www.w3.org/TR/web-animations-1/#by-computed-value) (just like combining a top-level track list). They otherwise combine [discretely](https://www.w3.org/TR/web-animations-1/#discrete).

<a id="ref-for-valdef-flex-fr③"></a>

#### <a id="fr-unit"></a>7.2.4.  Flexible Lengths: the [fr](#valdef-flex-fr) unit

<a id="ref-for-typedef-flex⑦"></a>

<a id="ref-for-leftover-space②"></a>

<a id="ref-for-grid-container⑤②"></a>

<a id="ref-for-valdef-flex-fr④"></a>

<a id="ref-for-flex-item②"></a>

<a id="ref-for-flex-container①"></a>

A <a id="flexible-length"></a>flexible length or <a id="typedef-flex"></a>[\<flex\>](#typedef-flex) is a dimension with the <a id="valdef-flex-fr"></a>fr unit, which represents a fraction of the [leftover space](#leftover-space) in the [grid container](#grid-container). Tracks sized with [fr](#valdef-flex-fr) units are called <a id="flexible-tracks"></a>flexible tracks as they flex in response to <a id="ref-for-leftover-space③"></a>leftover space similar to how [flex items](https://www.w3.org/TR/css-flexbox-1/#flex-item) with a zero base size fill space in a [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container).

<a id="ref-for-leftover-space④"></a>

<a id="ref-for-grid-template-rows-track-sizing-function④"></a>

<a id="ref-for-grid-template-columns-flex-factor③"></a>

The distribution of [leftover space](#leftover-space) occurs after all non-flexible [track sizing functions](#grid-template-rows-track-sizing-function) have reached their maximum. The total size of such rows or columns is subtracted from the available space, yielding the <a id="ref-for-leftover-space⑤"></a>leftover space, which is then divided among the flex-sized rows and columns in proportion to their [flex factor](#grid-template-columns-flex-factor).

<a id="ref-for-leftover-space⑥"></a>

<a id="ref-for-grid-template-columns-flex-factor④"></a>

Each column or row’s share of the [leftover space](#leftover-space) can be computed as the column or row’s <code>&lt;flex&gt; &#x2A; &lt;leftover space&gt; / &lt;sum of all <a href="#grid-template-columns-flex-factor">flex factors</a>&gt;</code>.

<a id="ref-for-typedef-flex⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> [\<flex\>](#typedef-flex) values between 0fr and 1fr have a somewhat special behavior: when the sum of the flex factors is less than 1, they will take up less than 100% of the leftover space.
>
> <a id="ref-for-typedef-flex⑨"></a>
>
> A track’s [\<flex\>](#typedef-flex) value is effectively a request for some proportion of the leftover space, with 1fr meaning “100% of the leftover space”; then if the tracks in that axis are requesting more than 100% in total, the requests are rebalanced to keep the same ratio but use up exactly 100% of it. However, if the tracks request <em>less</em> than the full amount (such as three tracks that are each .25fr) then they’ll each get exactly what they request (25% of the leftover space to each, with the final 25% left unfilled). See [§ 12.7 Expand Flexible Tracks](#algo-flex-tracks) for the exact details of how leftover space is distributed.
>
> <a id="ref-for-valdef-flex-fr⑤"></a>
>
> This pattern is required for continuous behavior as [fr](#valdef-flex-fr) values approach zero (which means the tracks wants <em>none</em> of the leftover space). Without this, a 1fr track would take all of the leftover space; but so would a 0.1fr track, and a 0.01fr track, etc., until finally the value is small enough to underflow to zero and the track suddenly takes up none of the leftover space. With this behavior, the track instead gradually takes less of the leftover space as its flex factor shrinks below 1fr, smoothly transitioning to taking none of the leftover space at zero.
>
> Unless this “partial fill” behavior is <em>specifically</em> what’s desired, authors should stick to values ≥ 1; for example, using 1fr and 2fr is usually better than using .33fr and .67fr, as they’re more likely to behave as intended if tracks are added or removed.

<a id="ref-for-grid-container⑤③"></a>

<a id="ref-for-indefinite"></a>

<a id="ref-for-grid-track①⑨"></a>

<a id="ref-for-valdef-grid-template-columns-max-content③"></a>

<a id="ref-for-grid-template-columns-flex-factor⑤"></a>

When the available space is infinite (which happens when the [grid container](#grid-container)’s width or height is [indefinite](https://www.w3.org/TR/css-sizing-3/#indefinite)), flex-sized [grid tracks](#grid-track) are sized to their contents while retaining their respective proportions. The used size of each flex-sized <a id="ref-for-grid-track②⓪"></a>grid track is computed by determining the [max-content](#valdef-grid-template-columns-max-content) size of each flex-sized <a id="ref-for-grid-track②①"></a>grid track and dividing that size by the respective [flex factor](#grid-template-columns-flex-factor) to determine a “hypothetical 1fr size”. The maximum of those is used as the resolved 1fr length (the <a id="flex-fraction"></a>flex fraction), which is then multiplied by each <a id="ref-for-grid-track②②"></a>grid track’s <a id="ref-for-grid-template-columns-flex-factor⑥"></a>flex factor to determine its final size.

<a id="ref-for-typedef-flex①⓪"></a>

<a id="ref-for-length-value"></a>

<a id="ref-for-percentage-value②"></a>

<a id="ref-for-funcdef-calc"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [\<flex\>](#typedef-flex) values are not [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)s (nor are they compatible with <a id="ref-for-length-value①"></a>\<length\>s, like some [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) values), so they cannot be represented in or combined with other unit types in [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) expressions.

#### <a id="computed-tracks"></a>7.2.5.  Computed Value of a Track Listing

<a id="ref-for-valdef-grid-template-rows-subgrid②"></a>

<a id="ref-for-list"></a>

<a id="ref-for-line-name-set"></a>

<a id="ref-for-track-section"></a>

The <a id="computed-track-list"></a>computed track list of a non-[subgrid](#valdef-grid-template-rows-subgrid) axis is a [list](https://infra.spec.whatwg.org/#list) alternating between [line name sets](#line-name-set) and [track sections](#track-section), with the first and last items being <a id="ref-for-line-name-set①"></a>line name sets.

<a id="ref-for-ordered-set"></a>

A <a id="line-name-set"></a>line name set is a (potentially empty) [set](https://infra.spec.whatwg.org/#ordered-set) of identifiers representing line names.

A <a id="track-section"></a>track section is either:

- <a id="ref-for-funcdef-grid-template-columns-minmax③"></a>

  <a id="ref-for-typedef-length-percentage⑥"></a>

  a [minmax()](#funcdef-grid-template-columns-minmax) functional notation representing a single track’s size, with each [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) computed (a <a id="computed-track-size"></a>computed track size)

- <a id="ref-for-funcdef-track-repeat-repeat⑧"></a>

  <a id="ref-for-integer-value④"></a>

  <a id="ref-for-typedef-track-list③"></a>

  <a id="ref-for-computed-track-list③"></a>

  a [repeat()](#funcdef-track-repeat-repeat) functional notation representing a repeated track list section, with its [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) computed and its [\<track-list\>](#typedef-track-list) represented as a [computed track list](#computed-track-list) (a <a id="computed-repeat-notation"></a>computed repeat notation)

<a id="ref-for-computed-track-list④"></a>

<a id="ref-for-valdef-grid-template-rows-subgrid③"></a>

<a id="ref-for-list①"></a>

<a id="ref-for-line-name-set②"></a>

<a id="ref-for-computed-repeat-notation"></a>

The [computed track list](#computed-track-list) of a [subgrid](#valdef-grid-template-rows-subgrid) axis is the <a id="ref-for-valdef-grid-template-rows-subgrid④"></a>subgrid keyword followed by a [list](https://infra.spec.whatwg.org/#list) of [line name sets](#line-name-set) and [computed repeat notations](#computed-repeat-notation) representing the line names specified for that axis.

#### <a id="resolved-track-list"></a>7.2.6.  Resolved Value of a Track Listing

<a id="ref-for-propdef-grid-template-rows⑧"></a>

<a id="ref-for-propdef-grid-template-columns⑨"></a>

<a id="ref-for-resolved-value-special-case-property"></a>

The [grid-template-rows](#propdef-grid-template-rows) and [grid-template-columns](#propdef-grid-template-columns) properties are [resolved value special case properties](https://www.w3.org/TR/cssom-1/#resolved-value-special-case-property). [\[CSSOM\]](#biblio-cssom)

##### <a id="resolved-track-list-standalone"></a>7.2.6.1.  Resolved Value of a Standalone Track Listing

<a id="ref-for-grid-container⑤④"></a>

<a id="ref-for-propdef-grid-template-rows⑨"></a>

<a id="ref-for-propdef-grid-template-columns①⓪"></a>

<a id="ref-for-standalone-axis②"></a>

<a id="ref-for-used-value①"></a>

When an element generates a [grid container](#grid-container) box, the [resolved value](https://www.w3.org/TR/cssom/#resolved-values) of its [grid-template-rows](#propdef-grid-template-rows) or [grid-template-columns](#propdef-grid-template-columns) property in a [standalone axis](#standalone-axis) is the [used value](https://www.w3.org/TR/css-cascade-5/#used-value), serialized with:

- <a id="ref-for-funcdef-track-repeat-repeat⑨"></a>

  Every track listed individually, whether implicitly or explicitly created, without using the [repeat()](#funcdef-track-repeat-repeat) notation.

- Every track size given as a length in pixels, regardless of sizing function.

- Adjacent line names collapsed into a single bracketed set.

<a id="ref-for-propdef-grid-template-rows①⓪"></a>

<a id="ref-for-propdef-grid-template-columns①①"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-cccd0e19"></a> The first bullet point of the above list means that implicit tracks get serialized as part of [grid-template-rows](#propdef-grid-template-rows)/etc., despite the fact that an author <em>cannot</em> actually specify implicit track sizes in those properties! So <a id="ref-for-propdef-grid-template-rows①①"></a>grid-template-rows and [grid-template-columns](#propdef-grid-template-columns) values might not round-trip correctly:
>
> ```text
> const s = getComputedStyle(gridEl);
> gridEl.style.gridTemplateRows = s.gridTemplateRows;
> // Code like this should be a no-op,
> // but if there are any implicit rows,
> // this will convert them into explicit rows,
> // possibly changing how grid items are positioned
> // and altering the overall size of the grid!
> ```
>
> This is an accidental property of an early implementation that leaked into later implementations without much thought given to it. We intend to remove it from the spec, but not until after we’ve defined a CSSOM API for getting information about implicit tracks, as currently this is the only way to get that information and a number of pages rely on that.

<a id="ref-for-propdef-display①③"></a>

<a id="ref-for-grid-container⑤⑤"></a>

<a id="ref-for-computed-value②"></a>

Otherwise, (e.g. when the element has [display: none](https://www.w3.org/TR/css-display-4/#propdef-display) or is not a [grid container](#grid-container)) the resolved value is simply the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d3b634e7"></a>
>
> ```html
> <style>
> #grid {
>   width: 500px;
>   grid-template-columns:
>     [a]     auto
>     [b]     minmax(min-content, 1fr)
>     [b c d] repeat(2, [e] 40px)
>             repeat(5, auto);
> }
> </style>
> <div id="grid">
>   <div style="grid-column-start: 1; width: 50px"></div>
>   <div style="grid-column-start: 9; width: 50px"></div>
> </div>
> <script>
>   var gridElement = document.getElementById("grid");
>   getComputedStyle(gridElement).gridTemplateColumns;
>   // [a] 50px [b] 320px [b c d e] 40px [e] 40px 0px 0px 0px 0px 50px
> </script>
> ```
<a id="ref-for-propdef-grid-template-rows①②"></a>

<a id="ref-for-propdef-grid-template-columns①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In general, resolved values are the computed values, except for a small list of legacy 2.1 properties. However, compatibility with early implementations of this module requires us to define [grid-template-rows](#propdef-grid-template-rows) and [grid-template-columns](#propdef-grid-template-columns) as returning used values.

<a id="ref-for-grid-placement-property⑧"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-e2bc4d57"></a> The CSS Working Group is considering whether to also return used values for the [grid-placement properties](#grid-placement-property) and is looking for feedback, especially from implementors. See [discussion](https://github.com/w3c/csswg-drafts/issues/2681).

##### <a id="resolved-track-list-subgrid"></a>7.2.6.2.  Resolved Value of a Subgridded Track Listing

<a id="ref-for-grid-container⑤⑥"></a>

<a id="ref-for-subgrid①①"></a>

<a id="ref-for-resolved-value"></a>

<a id="ref-for-propdef-grid-template-rows①③"></a>

<a id="ref-for-propdef-grid-template-columns①③"></a>

<a id="ref-for-used-value②"></a>

<a id="ref-for-valdef-grid-template-rows-subgrid⑤"></a>

<a id="ref-for-line-name-set③"></a>

<a id="ref-for-parent-grid⑦"></a>

<a id="ref-for-funcdef-track-repeat-repeat①⓪"></a>

When an element generates a [grid container](#grid-container) box that is a [subgrid](#subgrid), the [resolved value](https://www.w3.org/TR/cssom-1/#resolved-value) of the [grid-template-rows](#propdef-grid-template-rows) and [grid-template-columns](#propdef-grid-template-columns) properties represents the [used](https://www.w3.org/TR/css-cascade-5/#used-value) number of columns, serialized as the [subgrid](#valdef-grid-template-rows-subgrid) keyword followed by a list representing each of its lines as a [line name set](#line-name-set) of all the line’s names explicitly defined on the <a id="ref-for-subgrid①②"></a>subgrid (not including those adopted from the [parent grid](#parent-grid)), without using the [repeat()](#funcdef-track-repeat-repeat) notation.

<a id="ref-for-subgrid①③"></a>

<a id="ref-for-propdef-grid-column①"></a>

<a id="ref-for-propdef-grid-template-columns①④"></a>

<a id="ref-for-specified-value"></a>

<a id="ref-for-resolved-value①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8cc1f771"></a> For example, when applied to a [subgrid](#subgrid) with [grid-column: span 4](#propdef-grid-column), each of the following [grid-template-columns](#propdef-grid-template-columns) [specified values](https://www.w3.org/TR/css-cascade-5/#specified-value) becomes the corresponding [resolved values](https://www.w3.org/TR/cssom-1/#resolved-value):
>
> ```css
> specified: subgrid [a] repeat(auto-fill, [b]) [c]
> resolved:  subgrid [a] [b] [b] [b] [c]
> ```
>
> ```css
> specified: subgrid [a] [a] [a] [a] repeat(auto-fill, [b]) [c] [c]
> resolved:  subgrid [a] [a] [a] [a] [c]
> ```
>
> ```css
> specified: subgrid [] [a]
> resolved:  subgrid [] [a] [] [] []
> ```
>
> ```css
> specified: subgrid [a] [b] [c] [d] [e] [f]
> resolved:  subgrid [a] [b] [c] [d] [e]
> ```
<a id="ref-for-line-name-set④"></a>

<a id="ref-for-subgrid①④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This violates the general "shortest equivalent serialization" principle by serializing empty trailing [line name sets](#line-name-set), as the trailing <a id="ref-for-line-name-set⑤"></a>line name sets provide potentially-useful information about how many tracks the [subgrid](#subgrid) is spanning.

<a id="ref-for-propdef-grid-template-areas⑦"></a>

### <a id="grid-template-areas-property"></a>7.3.  Named Areas: the [grid-template-areas](#propdef-grid-template-areas) property

<strong>Table 4 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-grid-template-areas"></a>grid-template-areas

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-one-plus⑦"></a>

<a id="ref-for-string-value"></a>

<a id="ref-for-comb-one②③"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<string\>](https://www.w3.org/TR/css-values-4/#string-value)[+](https://www.w3.org/TR/css-values-4/#mult-one-plus)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-container⑤⑦"></a>

[grid containers](#grid-container)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-valdef-grid-template-areas-none"></a>

the keyword [none](#valdef-grid-template-areas-none) or a list of string values

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

<a id="ref-for-grid-item④⑧"></a>

<a id="ref-for-grid-placement-property⑨"></a>

<a id="ref-for-propdef-grid-template-areas⑧"></a>

<a id="ref-for-grid①②"></a>

<a id="ref-for-grid-container⑤⑧"></a>

This property specifies <a id="named-grid-area"></a>named grid areas, which are not associated with any particular [grid item](#grid-item), but can be referenced from the [grid-placement properties](#grid-placement-property). The syntax of the [grid-template-areas](#propdef-grid-template-areas) property also provides a visualization of the structure of the [grid](#grid), making the overall layout of the [grid container](#grid-container) easier to understand.

Values have the following meanings:

<a id="valdef-grid-template-areas-none"></a>none

<a id="ref-for-propdef-grid-template-rows①④"></a>

<a id="ref-for-propdef-grid-template-columns①⑤"></a>

<a id="ref-for-explicit-grid①③"></a>

<a id="ref-for-named-grid-area①"></a>

Indicates that no [named grid areas](#named-grid-area), and likewise no [explicit grid](#explicit-grid) tracks, are defined by this property (though <a id="ref-for-explicit-grid①④"></a>explicit grid tracks could still be created by [grid-template-columns](#propdef-grid-template-columns) or [grid-template-rows](#propdef-grid-template-rows)).

<a id="ref-for-explicit-grid①⑤"></a>

<a id="ref-for-propdef-grid-auto-rows③"></a>

<a id="ref-for-propdef-grid-auto-columns③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In the absence of an [explicit grid](#explicit-grid) any rows/columns will be [implicitly generated](#implicit-grids), and their size will be determined by the [grid-auto-rows](#propdef-grid-auto-rows) and [grid-auto-columns](#propdef-grid-auto-columns) properties.

<a id="ref-for-string-value①"></a>

<a id="valdef-grid-template-areas-string"></a>[\<string\>](https://www.w3.org/TR/css-values-4/#string-value)+

<a id="ref-for-propdef-grid-template-areas⑨"></a>

A row is created for every separate string listed for the [grid-template-areas](#propdef-grid-template-areas) property, and a column is created for each cell in the string, when parsed as follows:

Tokenize the string into a list of the following tokens, using longest-match semantics:

- <a id="ref-for-ident-code-point"></a>

  A sequence of [ident code points](https://www.w3.org/TR/css-syntax-3/#ident-code-point), representing a <a id="grid-template-areas-named-cell-token"></a>named cell token with a name consisting of its code points.

- A sequence of one or more "." (U+002E FULL STOP), representing a <a id="grid-template-areas-null-cell-token"></a>null cell token.

- <a id="ref-for-whitespace"></a>

  A sequence of [whitespace](https://www.w3.org/TR/css-syntax-3/#whitespace), representing nothing (do not produce a token).

- A sequence of any other characters, representing a <a id="grid-template-areas-trash-token"></a>trash token.

<a id="ref-for-typedef-ident"></a>

<a id="ref-for-propdef-grid-row①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: These rules can produce cell names that do not match the [\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident) syntax, such as "1st 2nd 3rd", which requires escaping when referencing those areas by name in other properties, like [grid-row: &#x5C;31st;](#propdef-grid-row) to reference the area named 1st.

- <a id="ref-for-grid-container⑤⑨"></a>

  <a id="ref-for-grid-template-areas-null-cell-token"></a>

  A [null cell token](#grid-template-areas-null-cell-token) represents an unnamed area in the [grid container](#grid-container).

- <a id="ref-for-grid-cell①"></a>

  <a id="ref-for-named-grid-area②"></a>

  <a id="ref-for-grid-template-areas-named-cell-token"></a>

  A [named cell token](#grid-template-areas-named-cell-token) creates a [named grid area](#named-grid-area) with the same name. Multiple <a id="ref-for-grid-template-areas-named-cell-token①"></a>named cell tokens within and between rows create a single <a id="ref-for-named-grid-area③"></a>named grid area that spans the corresponding [grid cells](#grid-cell).

- <a id="ref-for-grid-template-areas-trash-token"></a>

  A [trash token](#grid-template-areas-trash-token) is a syntax error, and makes the declaration invalid.

<a id="ref-for-grid-template-areas-named-cell-token②"></a>

<a id="ref-for-grid-template-areas-null-cell-token①"></a>

<a id="ref-for-named-grid-area④"></a>

<a id="ref-for-grid-cell②"></a>

All strings must define the same number of cell tokens ([named cell tokens](#grid-template-areas-named-cell-token) and/or [null cell tokens](#grid-template-areas-null-cell-token)), and at least one cell token, or else the declaration is invalid. If a [named grid area](#named-grid-area) spans multiple [grid cells](#grid-cell), but those cells do not form a single filled-in rectangle, the declaration is invalid.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Non-rectangular or disconnected regions may be permitted in a future version of this module.

<a id="ref-for-propdef-grid-template-areas①⓪"></a>

<a id="ref-for-named-grid-area⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9f77ebf7"></a> In this example, the [grid-template-areas](#propdef-grid-template-areas) property is used to create a page layout where areas are defined for header content (`head`), navigational content (`nav`), footer content (`foot`), and main content (`main`). Accordingly, the template creates three rows and two columns, with four [named grid areas](#named-grid-area). The `head` area spans both columns and the first row of the grid.
>
> ```text
> #grid {
>   display: grid;
>   grid-template-areas: "head head"
>                        "nav  main"
>                        "foot ...."
> }
> #grid > header { grid-area: head; }
> #grid > nav    { grid-area: nav; }
> #grid > main   { grid-area: main; }
> #grid > footer { grid-area: foot; }
> ```
#### <a id="serialize-template"></a>7.3.1.  Serialization Of Template Strings

<a id="ref-for-specified-value①"></a>

<a id="ref-for-computed-value③"></a>

<a id="ref-for-string-value②"></a>

<a id="ref-for-propdef-grid-template-areas①①"></a>

<a id="ref-for-grid-template-areas-null-cell-token②"></a>

When serializing either the [specified](https://www.w3.org/TR/css-cascade-5/#specified-value) or [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of a [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) value of [grid-template-areas](#propdef-grid-template-areas), each [null cell token](#grid-template-areas-null-cell-token) is serialized as a single "." (U+002E FULL STOP), and consecutive cell tokens are separated by a single space (U+0020 SPACE), with all other white space elided.

#### <a id="implicit-named-lines"></a>7.3.2.  Implicitly-Assigned Line Names

<a id="ref-for-propdef-grid-template-areas①②"></a>

<a id="ref-for-named-grid-area⑥"></a>

<a id="ref-for-implicitly-assigned-line-name①"></a>

The [grid-template-areas](#propdef-grid-template-areas) property generates <a id="implicitly-assigned-line-name"></a>implicitly-assigned line names from the [named grid areas](#named-grid-area) in the template. For each <a id="ref-for-named-grid-area⑦"></a>named grid area <var>foo</var>, four [implicitly-assigned line names](#implicitly-assigned-line-name) are created: two named <var>foo</var>-start, naming the row-start and column-start lines of the <a id="ref-for-named-grid-area⑧"></a>named grid area, and two named <var>foo</var>-end, naming the row-end and column-end lines of the <a id="ref-for-named-grid-area⑨"></a>named grid area.

<a id="ref-for-implicitly-assigned-line-name②"></a>

<a id="ref-for-line-name⑥"></a>

<a id="ref-for-propdef-grid-template-rows①⑤"></a>

<a id="ref-for-propdef-grid-template-columns①⑥"></a>

<a id="ref-for-explicitly-assigned-line-name"></a>

These [implicitly-assigned line names](#implicitly-assigned-line-name) behave just like any other [line names](#line-name), except that they do not appear in the value of [grid-template-rows](#propdef-grid-template-rows)/[grid-template-columns](#propdef-grid-template-columns). Even if an [explicitly-assigned line name](#explicitly-assigned-line-name) with the same name is defined, the <a id="ref-for-implicitly-assigned-line-name③"></a>implicitly-assigned line names are just more lines with the same name.

#### <a id="implicit-named-areas"></a>7.3.3.  Implicitly-Named Areas

<a id="ref-for-named-grid-area①⓪"></a>

<a id="ref-for-implicitly-assigned-line-name④"></a>

<a id="ref-for-propdef-grid-template-areas①③"></a>

<a id="ref-for-grid-placement-property①⓪"></a>

Since a [named grid area](#named-grid-area) is referenced by the [implicitly-assigned line names](#implicitly-assigned-line-name) it produces, explicitly adding named lines of the same form (foo-start/foo-end) effectively creates a <a id="ref-for-named-grid-area①①"></a>named grid area. Such <a id="implicitly-named-area"></a>implicitly-named areas do not appear in the value of [grid-template-areas](#propdef-grid-template-areas), but can still be referenced by the [grid-placement properties](#grid-placement-property).

<a id="ref-for-propdef-grid-template②"></a>

### <a id="explicit-grid-shorthand"></a>7.4.  Explicit Grid Shorthand: the [grid-template](#propdef-grid-template) property

<strong>Table 5 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-grid-template"></a>grid-template

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-explicit-track-list"></a>

<a id="ref-for-mult-one-plus⑧"></a>

<a id="ref-for-typedef-track-size④"></a>

<a id="ref-for-string-value③"></a>

<a id="ref-for-mult-opt①⑤"></a>

<a id="ref-for-typedef-line-names②⓪"></a>

<a id="ref-for-propdef-grid-template-columns①⑦"></a>

<a id="ref-for-propdef-grid-template-rows①⑥"></a>

<a id="ref-for-comb-one②④"></a>

none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ [\<'grid-template-rows'\>](#propdef-grid-template-rows) / [\<'grid-template-columns'\>](#propdef-grid-template-columns) \] <a id="ref-for-comb-one②⑤"></a>\| \[ [\<line-names\>](#typedef-line-names)[?](https://www.w3.org/TR/css-values-4/#mult-opt) [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) [\<track-size\>](#typedef-track-size)<a id="ref-for-mult-opt①⑥"></a>? <a id="ref-for-typedef-line-names②①"></a>\<line-names\><a id="ref-for-mult-opt①⑦"></a>? \][+](https://www.w3.org/TR/css-values-4/#mult-one-plus) \[ / [\<explicit-track-list\>](#typedef-explicit-track-list) \]<a id="ref-for-mult-opt①⑧"></a>?

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-container⑥⓪"></a>

[grid containers](#grid-container)

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

<a id="ref-for-propdef-grid-template③"></a>

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-propdef-grid-template-columns①⑧"></a>

<a id="ref-for-propdef-grid-template-rows①⑦"></a>

<a id="ref-for-propdef-grid-template-areas①④"></a>

The [grid-template](#propdef-grid-template) property is a [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for setting [grid-template-columns](#propdef-grid-template-columns), [grid-template-rows](#propdef-grid-template-rows), and [grid-template-areas](#propdef-grid-template-areas) in a single declaration. It has several distinct syntax forms:

<a id="valdef-grid-template-none"></a>none

<a id="ref-for-valdef-grid-template-rows-none②"></a>

Sets all three properties to their initial values ([none](#valdef-grid-template-rows-none)).

<a id="ref-for-propdef-grid-template-columns①⑨"></a>

<a id="ref-for-propdef-grid-template-rows①⑧"></a>

<a id="grid-template-rowcol"></a>[\<'grid-template-rows'\>](#propdef-grid-template-rows) / [\<'grid-template-columns'\>](#propdef-grid-template-columns)

<a id="ref-for-valdef-grid-template-areas-none①"></a>

<a id="ref-for-propdef-grid-template-areas①⑤"></a>

<a id="ref-for-propdef-grid-template-columns②⓪"></a>

<a id="ref-for-propdef-grid-template-rows①⑨"></a>

Sets [grid-template-rows](#propdef-grid-template-rows) and [grid-template-columns](#propdef-grid-template-columns) to the specified values, respectively, and sets [grid-template-areas](#propdef-grid-template-areas) to [none](#valdef-grid-template-areas-none).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-25dd92dd"></a>
>
> ```text
> grid-template: auto 1fr / auto 1fr auto;
> ```
>
> is equivalent to
>
> ```text
> grid-template-rows: auto 1fr;
> grid-template-columns: auto 1fr auto;
> grid-template-areas: none;
> ```
<a id="ref-for-typedef-explicit-track-list①"></a>

<a id="ref-for-typedef-track-size⑤"></a>

<a id="ref-for-string-value④"></a>

<a id="ref-for-typedef-line-names②②"></a>

<a id="grid-template-ascii"></a>\[ [\<line-names\>](#typedef-line-names)? [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) [\<track-size\>](#typedef-track-size)? <a id="ref-for-typedef-line-names②③"></a>\<line-names\>? \]+ \[ / [\<explicit-track-list\>](#typedef-explicit-track-list) \]?

- <a id="ref-for-propdef-grid-template-areas①⑥"></a>

  Sets [grid-template-areas](#propdef-grid-template-areas) to the strings listed.

- <a id="ref-for-propdef-grid-template-rows②⓪"></a>

  <a id="ref-for-typedef-track-size⑥"></a>

  <a id="ref-for-valdef-grid-template-columns-auto④"></a>

  Sets [grid-template-rows](#propdef-grid-template-rows) to the [\<track-size\>](#typedef-track-size)s following each string (filling in [auto](#valdef-grid-template-columns-auto) for any missing sizes), and splicing in the named lines defined before/after each size.

- <a id="ref-for-propdef-grid-template-columns②①"></a>

  <a id="ref-for-valdef-grid-template-rows-none③"></a>

  Sets [grid-template-columns](#propdef-grid-template-columns) to the track listing specified after the slash (or [none](#valdef-grid-template-rows-none), if not specified).

This syntax allows the author to align track names and sizes inline with their respective grid areas.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4fe2f42d"></a>
>
> ```text
> grid-template: [header-top] "a   a   a"     [header-bottom]
>                  [main-top] "b   b   b" 1fr [main-bottom]
>                           / auto 1fr auto;
> ```
>
> is equivalent to
>
> ```text
> grid-template-areas: "a a a"
>                      "b b b";
> grid-template-rows: [header-top] auto [header-bottom main-top] 1fr [main-bottom];
> grid-template-columns: auto 1fr auto;
> ```
>
> and creates the following grid:
>
> ![Three columns, sized auto, 1fr, and auto, respectively Two rows sized as auto and 1fr, respectively. A line named both “header-top” and “a-start” at the top, a line with four names—​“header-bottom”, “main-top”, “a-end”, and “b-start”—​in the middle, a line named “main-bottom” and “b-end” at the bottom. A line named “a-start” and “b-start” on the left edge, and a line named “a-end” and “b-end” on the right edge.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/grid-shorthand.svg)
>
> - <a id="ref-for-grid-placement-auto②"></a>
>
>   <a id="ref-for-valdef-grid-template-columns-auto⑤"></a>
>
>   Three columns, sized [auto](#valdef-grid-template-columns-auto), 1fr, and [auto](#grid-placement-auto), respectively
>
> - <a id="ref-for-valdef-grid-template-columns-auto⑥"></a>
>
>   Two rows sized as [auto](#valdef-grid-template-columns-auto) and 1fr, respectively.
>
> - A line named both “header-top” and “a-start” at the top, a line with four names—​“header-bottom”, “main-top”, “a-end”, and “b-start”—​in the middle, a line named “main-bottom” and “b-end” at the bottom.
>
> - A line named “a-start” and “b-start” on the left edge, and a line named “a-end” and “b-end” on the right edge.
>
> <a id="ref-for-implicitly-assigned-line-name⑤"></a>
>
> <a id="ref-for-named-grid-area①②"></a>
>
> The grid created by the declarations above. (The “a/b-start/end” names are [implicitly assigned](#implicitly-assigned-line-name) by the [named grid areas](#named-grid-area).)

<a id="ref-for-funcdef-track-repeat-repeat①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that the [repeat()](#funcdef-track-repeat-repeat) function isn’t allowed in these track listings, as the tracks are intended to visually line up one-to-one with the rows/columns in the “ASCII art”.

<a id="ref-for-propdef-grid③"></a>

<a id="ref-for-propdef-grid-template④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [grid](#propdef-grid) shorthand accepts the same syntax, but also resets the implicit grid properties to their initial values. Unless authors want those to cascade in separately, it is therefore recommended to use <a id="ref-for-propdef-grid④"></a>grid instead of [grid-template](#propdef-grid-template).

### <a id="implicit-grids"></a>7.5.  The Implicit Grid

<a id="ref-for-propdef-grid-template-rows②①"></a>

<a id="ref-for-propdef-grid-template-columns②②"></a>

<a id="ref-for-propdef-grid-template-areas①⑦"></a>

<a id="ref-for-explicit-grid①⑥"></a>

<a id="ref-for-grid-item④⑨"></a>

<a id="ref-for-grid-container⑥①"></a>

<a id="ref-for-grid①③"></a>

<a id="ref-for-propdef-grid-auto-rows④"></a>

<a id="ref-for-propdef-grid-auto-columns④"></a>

<a id="ref-for-implicit-grid-track"></a>

<a id="ref-for-explicit-grid-track"></a>

The [grid-template-rows](#propdef-grid-template-rows), [grid-template-columns](#propdef-grid-template-columns), and [grid-template-areas](#propdef-grid-template-areas) properties define a fixed number of tracks that form the [explicit grid](#explicit-grid). When [grid items](#grid-item) are positioned outside of these bounds, the [grid container](#grid-container) generates <a id="implicit-grid-track"></a>implicit grid tracks by adding <a id="implicit-grid-lines"></a>implicit grid lines to the [grid](#grid). These lines together with the <a id="ref-for-explicit-grid①⑦"></a>explicit grid form the <a id="implicit-grid"></a>implicit grid. The [grid-auto-rows](#propdef-grid-auto-rows) and [grid-auto-columns](#propdef-grid-auto-columns) properties size these [implicit grid tracks](#implicit-grid-track), as well as any [explicit grid tracks](#explicit-grid-track) created by <a id="ref-for-propdef-grid-template-areas①⑧"></a>grid-template-areas but not explicitly sized by <a id="ref-for-propdef-grid-template-rows②②"></a>grid-template-rows or <a id="ref-for-propdef-grid-template-columns②③"></a>grid-template-columns

<a id="ref-for-propdef-grid-auto-flow"></a>

<a id="ref-for-grid-item⑤⓪"></a>

<a id="ref-for-explicit-grid①⑧"></a>

<a id="ref-for-implicit-grid-track①"></a>

The [grid-auto-flow](#propdef-grid-auto-flow) property controls auto-placement of [grid items](#grid-item) without an explicit position. Once the [explicit grid](#explicit-grid) is filled (or if there is no <a id="ref-for-explicit-grid①⑨"></a>explicit grid) auto-placement will also cause the generation of [implicit grid tracks](#implicit-grid-track).

<a id="ref-for-propdef-grid⑤"></a>

<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-propdef-grid-auto-flow①"></a>

<a id="ref-for-propdef-grid-auto-rows⑤"></a>

<a id="ref-for-propdef-grid-auto-columns⑤"></a>

<a id="ref-for-explicit-grid-properties"></a>

The [grid](#propdef-grid) [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) property can set the <a id="implicit-grid-properties"></a>implicit grid properties ([grid-auto-flow](#propdef-grid-auto-flow), [grid-auto-rows](#propdef-grid-auto-rows), and [grid-auto-columns](#propdef-grid-auto-columns)) together with the [explicit grid properties](#explicit-grid-properties) in a single declaration.

<a id="ref-for-propdef-grid-auto-rows⑥"></a>

<a id="ref-for-propdef-grid-auto-columns⑥"></a>

### <a id="auto-tracks"></a>7.6.  Implicit Track Sizing: the [grid-auto-rows](#propdef-grid-auto-rows) and [grid-auto-columns](#propdef-grid-auto-columns) properties

<strong>Table 6 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-grid-auto-columns"></a>grid-auto-columns, <a id="propdef-grid-auto-rows"></a>grid-auto-rows

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-one-plus⑨"></a>

<a id="ref-for-typedef-track-size⑦"></a>

[\<track-size\>](#typedef-track-size)[+](https://www.w3.org/TR/css-values-4/#mult-one-plus)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-container⑥②"></a>

[grid containers](#grid-container)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

see [Track Sizing](#track-sizing)

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

see [Track Sizing](#track-sizing)

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

if the list lengths match, by computed value type per item; discrete otherwise

<a id="ref-for-propdef-grid-auto-columns⑦"></a>

<a id="ref-for-propdef-grid-auto-rows⑦"></a>

<a id="ref-for-propdef-grid-template-rows②③"></a>

<a id="ref-for-propdef-grid-template-columns②④"></a>

<a id="ref-for-implicit-grid-track②"></a>

<a id="ref-for-explicit-grid②⓪"></a>

The [grid-auto-columns](#propdef-grid-auto-columns) and [grid-auto-rows](#propdef-grid-auto-rows) properties specify the size of tracks not assigned a size by [grid-template-rows](#propdef-grid-template-rows) or [grid-template-columns](#propdef-grid-template-columns). If multiple track sizes are given, the pattern is repeated as necessary to find the size of the affected tracks. The first track after the last explicitly-sized track receives the first specified size, and so on forwards; and the last [implicit grid track](#implicit-grid-track) before the [explicit grid](#explicit-grid) receives the last specified size, and so on backwards.

<a id="ref-for-propdef-grid-template-rows②④"></a>

<a id="ref-for-propdef-grid-template-columns②⑤"></a>

<a id="ref-for-propdef-grid-template-areas①⑨"></a>

<a id="ref-for-implicit-grid-track③"></a>

<a id="ref-for-grid-item-placement-algorithm"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If a grid item is positioned into a row or column that is not explicitly declared by [grid-template-rows](#propdef-grid-template-rows)/[grid-template-columns](#propdef-grid-template-columns) and/or [grid-template-areas](#propdef-grid-template-areas), [implicit grid tracks](#implicit-grid-track) are created to hold it. This can happen either by explicitly positioning into a row or column that is out of range, or by the [auto-placement algorithm](#grid-item-placement-algorithm) creating additional rows or columns.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b32ffb8f"></a>
>
> ```html
> <style>
>   #grid {
>     display: grid;
>     grid-template-columns: 20px;
>     grid-auto-columns: 40px;
>     grid-template-rows: 20px;
>     grid-auto-rows: 40px;
>   }
>   #A { grid-column: 1; grid-row: 1; }
>   #B { grid-column: 2; grid-row: 1; }
>   #C { grid-column: 1; grid-row: 2; }
>   #D { grid-column: 2; grid-row: 2; }
> </style>
> 
> <div id="grid">
>   <div id="A">A</div>
>   <div id="B">B</div>
>   <div id="C">C</div>
>   <div id="D">D</div>
> </div>
> ```
>
> ![](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/auto-flow.svg)
>
> A 2×2 grid with one explicit 20px×20px grid cell in the first row+column and three additional cells resulting from the implicit 40px column and row generated to hold the additional grid items.

<a id="ref-for-propdef-grid-auto-flow②"></a>

### <a id="grid-auto-flow-property"></a>7.7.  <a id="auto-placement"></a>Automatic Placement: the [grid-auto-flow](#propdef-grid-auto-flow) property

<strong>Table 7 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-grid-auto-flow"></a>grid-auto-flow

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-comb-any"></a>

<a id="ref-for-comb-one②⑥"></a>

\[ row [\|](https://www.w3.org/TR/css-values-4/#comb-one) column \] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) dense

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

row

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-container⑥③"></a>

[grid containers](#grid-container)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified keyword(s)

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

<a id="ref-for-grid-item⑤①"></a>

<a id="ref-for-grid-container⑥④"></a>

<a id="ref-for-grid-item-placement-algorithm①"></a>

<a id="ref-for-propdef-grid-auto-flow③"></a>

[Grid items](#grid-item) that aren’t explicitly placed are automatically placed into an unoccupied space in the [grid container](#grid-container) by the [auto-placement algorithm](#grid-item-placement-algorithm). [grid-auto-flow](#propdef-grid-auto-flow) controls how the <a id="ref-for-grid-item-placement-algorithm②"></a>auto-placement algorithm works, specifying exactly how auto-placed items get flowed into the grid. See [§ 8.5 Grid Item Placement Algorithm](#auto-placement-algo) for details on precisely how the auto-placement algorithm works.

<a id="valdef-grid-auto-flow-row"></a>row  
<a id="ref-for-valdef-grid-auto-flow-column"></a>

<a id="ref-for-valdef-grid-auto-flow-row"></a>

<a id="ref-for-grid-item-placement-algorithm③"></a>

The [auto-placement algorithm](#grid-item-placement-algorithm) places items by filling each row in turn, adding new rows as necessary. If neither [row](#valdef-grid-auto-flow-row) nor [column](#valdef-grid-auto-flow-column) is provided, <a id="ref-for-valdef-grid-auto-flow-row①"></a>row is assumed.

<a id="valdef-grid-auto-flow-column"></a>column  
<a id="ref-for-grid-item-placement-algorithm④"></a>

The [auto-placement algorithm](#grid-item-placement-algorithm) places items by filling each column in turn, adding new columns as necessary.

<a id="valdef-grid-auto-flow-dense"></a>dense  
<a id="ref-for-grid-item-placement-algorithm⑤"></a>

If specified, the [auto-placement algorithm](#grid-item-placement-algorithm) uses a “dense” packing algorithm, which attempts to fill in holes earlier in the grid if smaller items come up later. This may cause items to appear out-of-order, when doing so would fill in holes left by larger items.

If omitted, a “sparse” algorithm is used, where the placement algorithm only ever moves “forward” in the grid when placing items, never backtracking to fill holes. This ensures that all of the auto-placed items appear “in order”, even if this leaves holes that could have been filled by later items.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A future level of this module is expected to add a value that flows auto-positioned items together into a single “default” cell.

<a id="ref-for-grid-item⑤②"></a>

<a id="ref-for-order-modified-document-order①"></a>

Auto-placement takes [grid items](#grid-item) in [order-modified document order](https://www.w3.org/TR/css-flexbox-1/#order-modified-document-order).

<a id="ref-for-propdef-grid-auto-flow④"></a>

<a id="ref-for-valdef-grid-auto-flow-row②"></a>

<a id="ref-for-grid-item⑤③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-deb30762"></a> In the following example, there are three columns, each auto-sized to their contents. No rows are explicitly defined. The [grid-auto-flow](#propdef-grid-auto-flow) property is [row](#valdef-grid-auto-flow-row) which instructs the grid to search across its three columns starting with the first row, then the next, adding rows as needed until sufficient space is located to accommodate the position of any auto-placed [grid item](#grid-item).
>
> ![Image: A form arranged using automatic placement.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/auto-placed-form.png)
>
> A form arranged using automatic placement.
>
> ```text
> <style type="text/css">
> form {
>   display: grid;
>   /* Define three columns, all content-sized,
>      and name the corresponding lines. */
>   grid-template-columns: [labels] auto [controls] auto [oversized] auto;
>   grid-auto-flow: row dense;
> }
> form > label {
>   /* Place all labels in the "labels" column and
>      automatically find the next available row. */
>   grid-column: labels;
>   grid-row: auto;
> }
> form > input, form > select {
>   /* Place all controls in the "controls" column and
>      automatically find the next available row. */
>   grid-column: controls;
>   grid-row: auto;
> }
> 
> #department-block {
>   /* Auto place this item in the "oversized" column
>      in the first row where an area that spans three rows
>      won’t overlap other explicitly placed items or areas
>      or any items automatically placed prior to this area. */
>   grid-column: oversized;
>   grid-row: span 3;
> }
> 
> /* Place all the buttons of the form
>    in the explicitly defined grid area. */
> #buttons {
>   grid-row: auto;
> 
>   /* Ensure the button area spans the entire grid element
>      in the inline axis. */
>   grid-column: 1 / -1;
>   text-align: end;
> }
> </style>
> <form>
>   <label for="firstname">First name:</label>
>   <input type="text" id="firstname" name="firstname">
>   <label for="lastname">Last name:</label>
>   <input type="text" id="lastname" name="lastname">
>   <label for="address">Address:</label>
>   <input type="text" id="address" name="address">
>   <label for="address2">Address 2:</label>
>   <input type="text" id="address2" name="address2">
>   <label for="city">City:</label>
>   <input type="text" id="city" name="city">
>   <label for="state">State:</label>
>   <select type="text" id="state" name="state">
>     <option value="WA">Washington</option>
>   </select>
>   <label for="zip">Zip:</label>
>   <input type="text" id="zip" name="zip">
> 
>   <div id="department-block">
>     <label for="department">Department:</label>
>     <select id="department" name="department" multiple>
>       <option value="finance">Finance</option>
>       <option value="humanresources">Human Resources</option>
>       <option value="marketing">Marketing</option>
>     </select>
>   </div>
> 
>   <div id="buttons">
>     <button id="cancel">Cancel</button>
>     <button id="back">Back</button>
>     <button id="next">Next</button>
>   </div>
> </form>
> ```
<a id="ref-for-propdef-grid⑥"></a>

### <a id="grid-shorthand"></a>7.8.  Grid Definition Shorthand: the [grid](#propdef-grid) property

<strong>Table 8 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-grid"></a>grid

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-grid-template-columns②⑥"></a>

<a id="ref-for-propdef-grid-auto-rows⑧"></a>

<a id="ref-for-mult-opt②①"></a>

<a id="ref-for-comb-all①"></a>

<a id="ref-for-propdef-grid-auto-columns⑧"></a>

<a id="ref-for-mult-opt①⑨"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-propdef-grid-template-rows②⑤"></a>

<a id="ref-for-comb-one②⑦"></a>

<a id="ref-for-propdef-grid-template⑤"></a>

[\<'grid-template'\>](#propdef-grid-template) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<'grid-template-rows'\>](#propdef-grid-template-rows) / \[ auto-flow [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) dense[?](https://www.w3.org/TR/css-values-4/#mult-opt) \] [\<'grid-auto-columns'\>](#propdef-grid-auto-columns)<a id="ref-for-mult-opt②⓪"></a>? <a id="ref-for-comb-one②⑧"></a>\| \[ auto-flow [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) dense[?](https://www.w3.org/TR/css-values-4/#mult-opt) \] [\<'grid-auto-rows'\>](#propdef-grid-auto-rows)<a id="ref-for-mult-opt②②"></a>? / [\<'grid-template-columns'\>](#propdef-grid-template-columns)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

none

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-container⑥⑤"></a>

[grid containers](#grid-container)

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

<a id="ref-for-propdef-grid⑦"></a>

<a id="ref-for-shorthand-property③"></a>

<a id="ref-for-explicit-grid-properties①"></a>

<a id="ref-for-propdef-grid-template-rows②⑥"></a>

<a id="ref-for-propdef-grid-template-columns②⑦"></a>

<a id="ref-for-propdef-grid-template-areas②⓪"></a>

<a id="ref-for-implicit-grid-properties"></a>

<a id="ref-for-propdef-grid-auto-rows⑨"></a>

<a id="ref-for-propdef-grid-auto-columns⑨"></a>

<a id="ref-for-propdef-grid-auto-flow⑤"></a>

<a id="ref-for-gutter③"></a>

The [grid](#propdef-grid) property is a [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) that sets all of the [explicit grid properties](#explicit-grid-properties) ([grid-template-rows](#propdef-grid-template-rows), [grid-template-columns](#propdef-grid-template-columns), and [grid-template-areas](#propdef-grid-template-areas)), and all the [implicit grid properties](#implicit-grid-properties) ([grid-auto-rows](#propdef-grid-auto-rows), [grid-auto-columns](#propdef-grid-auto-columns), and [grid-auto-flow](#propdef-grid-auto-flow)), in a single declaration. (It does not reset the [gutter](https://www.w3.org/TR/css-align-3/#gutter) properties.)

<a id="ref-for-propdef-grid-template⑥"></a>

Its syntax matches [grid-template](#propdef-grid-template), plus an additional syntax form for defining auto-flow grids:

<a id="ref-for-propdef-grid-template⑦"></a>

[\<'grid-template'\>](#propdef-grid-template)

<a id="ref-for-propdef-grid-template⑧"></a>

Sets the [grid-template](#propdef-grid-template) longhands as as for <a id="ref-for-propdef-grid-template⑨"></a>grid-template, and the grid-auto-\* longhands to their initial values.

<a id="ref-for-propdef-grid-auto-columns①⓪"></a>

<a id="ref-for-propdef-grid-template-rows②⑦"></a>

<a id="grid-s-auto-row"></a>[\<'grid-template-rows'\>](#propdef-grid-template-rows) / \[ auto-flow &#x26;&#x26; dense? \] [\<'grid-auto-columns'\>](#propdef-grid-auto-columns)?

<a id="ref-for-propdef-grid-template-columns②⑧"></a>

<a id="ref-for-propdef-grid-auto-rows①⓪"></a>

<a id="grid-s-auto-column"></a>\[ auto-flow &#x26;&#x26; dense? \] [\<'grid-auto-rows'\>](#propdef-grid-auto-rows)? / [\<'grid-template-columns'\>](#propdef-grid-template-columns)

<a id="ref-for-valdef-grid-auto-flow-dense"></a>

<a id="ref-for-valdef-grid-auto-flow-column①"></a>

<a id="ref-for-valdef-grid-auto-flow-row③"></a>

<a id="ref-for-propdef-grid-auto-flow⑥"></a>

<a id="ref-for-propdef-grid-auto-columns①①"></a>

<a id="ref-for-propdef-grid-auto-rows①①"></a>

<a id="ref-for-valdef-grid-template-rows-none④"></a>

<a id="ref-for-propdef-grid-template-columns②⑨"></a>

<a id="ref-for-propdef-grid-template-rows②⑧"></a>

Sets up auto-flow, by setting the tracks in one axis explicitly (setting either [grid-template-rows](#propdef-grid-template-rows) or [grid-template-columns](#propdef-grid-template-columns) as specified, and setting the other to [none](#valdef-grid-template-rows-none)), and specifying how to auto-repeat the tracks in the other axis (setting either [grid-auto-rows](#propdef-grid-auto-rows) or [grid-auto-columns](#propdef-grid-auto-columns) as specified, and setting the other to auto). [grid-auto-flow](#propdef-grid-auto-flow) is also set to either [row](#valdef-grid-auto-flow-row) or [column](#valdef-grid-auto-flow-column) accordingly, with [dense](#valdef-grid-auto-flow-dense) if it’s specified.

<a id="ref-for-propdef-grid⑧"></a>

<a id="ref-for-longhand"></a>

All other [grid](#propdef-grid) [sub-properties](https://www.w3.org/TR/css-cascade-5/#longhand) are reset to their initial values.

<a id="ref-for-propdef-grid⑨"></a>

<a id="ref-for-shorthand-property④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that you can only specify the explicit <em>or</em> the implicit grid properties in a single [grid](#propdef-grid) declaration. The sub-properties you don’t specify are set to their initial value, as normal for [shorthands](https://www.w3.org/TR/css-cascade-5/#shorthand-property).

<a id="ref-for-propdef-grid-template①⓪"></a>

<a id="ref-for-explicit-grid②①"></a>

<a id="ref-for-propdef-grid①⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-dec34e0f"></a> In addition to accepting the [grid-template](#propdef-grid-template) shorthand syntax for setting up the [explicit grid](#explicit-grid), the [grid](#propdef-grid) shorthand can also easily set up parameters for an auto-formatted grid. For example, <a id="ref-for-propdef-grid①①"></a>grid: auto-flow 1fr / 100px; is equivalent to
>
> ```text
> grid-template: none / 100px;
> grid-auto-flow: row;
> grid-auto-rows: 1fr;
> grid-auto-columns: auto;
> ```
>
> <a id="ref-for-propdef-grid①②"></a>
>
> Similarly, [grid: none / auto-flow 1fr](#propdef-grid) is equivalent to
>
> ```text
> grid-template: none;
> grid-auto-flow: column;
> grid-auto-rows: auto;
> grid-auto-columns: 1fr;
> ```
<a id="ref-for-propdef-grid-template①①"></a>

When serializing, if all the grid-auto-\* longhands have their initial values, the [grid-template](#propdef-grid-template) syntax is used.

## <a id="placement"></a>8.  Placing Grid Items

<a id="ref-for-grid-item⑤④"></a>

<a id="ref-for-grid-area①⑥"></a>

<a id="ref-for-grid-cell③"></a>

<a id="ref-for-containing-block②"></a>

<a id="ref-for-propdef-justify-self"></a>

<a id="ref-for-propdef-align-self①"></a>

<a id="ref-for-layout-algorithm"></a>

Every [grid item](#grid-item) is associated with a [grid area](#grid-area), a rectangular set of adjacent [grid cells](#grid-cell) that the <a id="ref-for-grid-item⑤⑤"></a>grid item occupies. This <a id="ref-for-grid-area①⑦"></a>grid area defines the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) for the <a id="ref-for-grid-item⑤⑥"></a>grid item within which the self-alignment properties ([justify-self](https://www.w3.org/TR/css-align-3/#propdef-justify-self) and [align-self](https://www.w3.org/TR/css-align-3/#propdef-align-self)) determine their actual position. The cells that a <a id="ref-for-grid-item⑤⑦"></a>grid item occupies also influence the sizing of the grid’s rows and columns, defined in [§ 12 Grid Layout Algorithm](#layout-algorithm).

<a id="ref-for-grid-item⑤⑧"></a>

<a id="ref-for-grid-area①⑧"></a>

<a id="ref-for-grid①④"></a>

<a id="ref-for-grid-position"></a>

<a id="ref-for-grid-span②"></a>

The location of a [grid item’s](#grid-item) [grid area](#grid-area) within the [grid](#grid) is defined by its <a id="grid-placement"></a>placement, which consists of a [grid position](#grid-position) and a [grid span](#grid-span):

<a id="grid-position"></a>grid position  
<a id="ref-for-auto-placement①"></a>

<a id="ref-for-grid-position①"></a>

<a id="ref-for-grid①⑤"></a>

<a id="ref-for-grid-item⑤⑨"></a>

The [grid item](#grid-item)’s location in the [grid](#grid) in each axis. A [grid position](#grid-position) can be either <a id="definite-grid-position"></a>definite (explicitly specified) or <a id="automatic-grid-position"></a>automatic (determined by [auto-placement](#auto-placement)).

<a id="grid-span"></a>grid span  
<a id="ref-for-automatic-grid-span"></a>

<a id="ref-for-explicit-grid-span"></a>

<a id="ref-for-implicit-grid-span"></a>

<a id="ref-for-grid-span③"></a>

<a id="ref-for-grid-item⑥⓪"></a>

<a id="ref-for-grid-track②③"></a>

How many [grid tracks](#grid-track) the [grid item](#grid-item) occupies in each axis. The [grid span](#grid-span) in an axis can be [implicit](#implicit-grid-span), [explicit](#explicit-grid-span), or [automatic](#automatic-grid-span):

- <a id="ref-for-grid-placement-property①①"></a>

  <a id="ref-for-grid-span④"></a>

  If both the \*-start and \*-end values of its [grid-placement properties](#grid-placement-property) specify a line, its [grid span](#grid-span) is <a id="implicit-grid-span"></a>implicit.

- <a id="ref-for-grid-span⑤"></a>

  If it has an explicit span value, its [grid span](#grid-span) is <a id="explicit-grid-span"></a>explicit.

- <a id="ref-for-grid-span⑥"></a>

  <a id="ref-for-subgridded-axis④"></a>

  <a id="ref-for-typedef-line-name-list⑤"></a>

  Otherwise, its [grid span](#grid-span) is <a id="automatic-grid-span"></a>automatic: if it is [subgridded](#subgridded-axis) in that axis, its <a id="ref-for-grid-span⑦"></a>grid span is [determined](#subgrid-span) from its [\<line-name-list\>](#typedef-line-name-list); otherwise its <a id="ref-for-grid-span⑧"></a>grid span is 1.

<a id="ref-for-propdef-grid-row-start①"></a>

<a id="ref-for-propdef-grid-row-end①"></a>

<a id="ref-for-propdef-grid-column-start①"></a>

<a id="ref-for-propdef-grid-column-end①"></a>

<a id="ref-for-propdef-grid-row②"></a>

<a id="ref-for-propdef-grid-column②"></a>

<a id="ref-for-propdef-grid-area①"></a>

<a id="ref-for-grid-item⑥①"></a>

<a id="ref-for-grid-placement①"></a>

The <a id="grid-placement-property"></a>grid-placement properties—​the longhands [grid-row-start](#propdef-grid-row-start), [grid-row-end](#propdef-grid-row-end), [grid-column-start](#propdef-grid-column-start), [grid-column-end](#propdef-grid-column-end), and their shorthands [grid-row](#propdef-grid-row), [grid-column](#propdef-grid-column), and [grid-area](#propdef-grid-area)—​allow the author to specify a [grid item](#grid-item)’s [placement](#grid-placement) by providing any (or none) of the following six pieces of information:

|                     | Row            | Column            |
|---------------------|----------------|-------------------|
| <var>Start</var> | row-start line | column-start line |
| <var>End</var> | row-end line   | column-end line   |
| <var>Span</var> | row span       | column span       |

A definite value for any two of <var>Start</var>, <var>End</var>, and <var>Span</var> in a given dimension implies a definite value for the third.

### <a id="common-uses"></a>8.1.  Common Patterns for Grid Placement

<em>This section is informative.</em>

<a id="ref-for-grid-placement-property①②"></a>

The [grid-placement property](#grid-placement-property) longhands are organized into three shorthands:

<a id="grid-property-breakdown"></a>

<strong>Table 10 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (data cell; column span 4):</strong>

<a id="ref-for-propdef-grid-area②"></a>

[grid-area](#propdef-grid-area)

<strong>Row 2</strong>

<strong>Column 1 (data cell; column span 2):</strong>

<a id="ref-for-propdef-grid-column③"></a>

[grid-column](#propdef-grid-column)

<strong>Column 3 (data cell; column span 2):</strong>

<a id="ref-for-propdef-grid-row③"></a>

[grid-row](#propdef-grid-row)

<strong>Row 3</strong>

<strong>Column 1 (data cell):</strong>

<a id="ref-for-propdef-grid-column-start②"></a>

[grid-column-start](#propdef-grid-column-start)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-propdef-grid-column-end②"></a>

[grid-column-end](#propdef-grid-column-end)

<strong>Column 3 (data cell):</strong>

<a id="ref-for-propdef-grid-row-start②"></a>

[grid-row-start](#propdef-grid-row-start)

<strong>Column 4 (data cell):</strong>

<a id="ref-for-propdef-grid-row-end②"></a>

[grid-row-end](#propdef-grid-row-end)

#### <a id="common-uses-named-areas"></a>8.1.1.  Named Areas

<a id="ref-for-named-grid-area①③"></a>

<a id="ref-for-propdef-grid-template-areas②①"></a>

<a id="ref-for-propdef-grid-area③"></a>

An item can be placed into a [named grid area](#named-grid-area) (such as those produced by the template in [grid-template-areas](#propdef-grid-template-areas)) by specifying the area’s name in [grid-area](#propdef-grid-area):

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a024ab3a"></a>
>
> ```text
> article {
>   grid-area: main;
>   /* Places item into the named area "main". */
> }
> ```
<a id="ref-for-named-grid-area①④"></a>

An item can also be <em>partially</em> aligned with a [named grid area](#named-grid-area), with other edges aligned to some other line:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-676a9f77"></a>
>
> ```text
> .one {
>   grid-row-start: main;
>   /* Align the row-start edge to the start edge of the "main" named area. */
> }
> ```
#### <a id="common-uses-numeric"></a>8.1.2.  Numeric Indexes and Spans

Grid items can be positioned and sized by number, which is particularly helpful for script-driven layouts:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-85154965"></a>
>
> ```text
> .two {
>   grid-row: 2;    /* Place item in the second row. */
>   grid-column: 3; /* Place item in the third column. */
>   /* Equivalent to grid-area: 2 / 3; */
> }
> ```
By default, a grid item has a span of 1. Different spans can be given explicitly:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a58ad9ce"></a>
>
> ```text
> .three {
>   grid-row: 2 / span 5;
>   /* Starts in the 2nd row,
>      spans 5 rows down (ending in the 7th row). */
> }
> 
> .four {
>   grid-row: span 5 / 7;
>   /* Ends in the 7th row,
>      spans 5 rows up (starting in the 2nd row). */
> }
> ```
<a id="ref-for-writing-mode①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that grid indexes are [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) relative. For example, in a right-to-left language like Arabic, the first column is the rightmost column.

#### <a id="common-uses-named-lines"></a>8.1.3.  Named Lines and Spans

<a id="ref-for-line-name⑦"></a>

Instead of counting lines by number, lines can be referenced by their [line name](#line-name):

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9c11bd3f"></a>
>
> ```text
> .five {
>   grid-column: first / middle;
>   /* Span from line "first" to line "middle". */
> }
> ```
<a id="ref-for-named-grid-area①⑤"></a>

<a id="ref-for-line-name⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that if a [named grid area](#named-grid-area) has the same name as a [line name](#line-name), the placement algorithm will prefer to use <a id="ref-for-named-grid-area①⑥"></a>named grid area’s lines instead.

If there are multiple lines of the same name, they effectively establish a named set of grid lines, which can be exclusively indexed by filtering the placement by name:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ca2f749c"></a>
>
> ```text
> .six {
>   grid-row: text 5 / text 7;
>   /* Span between the 5th and 7th lines named "text". */
>   grid-row: text 5 / span text 2;
>   /* Same as above - start at the 5th line named "text",
>      then span across two more "text" lines, to the 7th. */
> }
> ```
#### <a id="common-uses-auto-placement"></a>8.1.4.  Auto Placement

<a id="ref-for-grid-item⑥②"></a>

<a id="ref-for-grid-cell④"></a>

<a id="ref-for-grid①⑥"></a>

A [grid item](#grid-item) can be automatically placed into the next available empty [grid cell](#grid-cell), growing the [grid](#grid) if there’s no space left.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b79de521"></a>
>
> ```text
> .eight {
>   grid-area: auto; /* Initial value */
> }
> ```
This can be used, for example, to list a number of sale items on a catalog site in a grid pattern.

Auto-placement can be combined with an explicit span, if the item should take up more than one cell:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f32a97df"></a>
>
> ```text
> .nine {
>   grid-area: span 2 / span 3;
>   /* Auto-placed item, covering two rows and three columns. */
> }
> ```
<a id="ref-for-grid-item-placement-algorithm⑥"></a>

<a id="ref-for-propdef-grid-auto-flow⑦"></a>

Whether the [auto-placement algorithm](#grid-item-placement-algorithm) searches across and adds rows, or searches across and adds columns, is controlled by the [grid-auto-flow](#propdef-grid-auto-flow) property.

<a id="ref-for-grid-item-placement-algorithm⑦"></a>

<a id="ref-for-valdef-grid-auto-flow-dense①"></a>

<a id="ref-for-propdef-grid-auto-flow⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: By default, the [auto-placement algorithm](#grid-item-placement-algorithm) looks linearly through the grid without backtracking; if it has to skip some empty spaces to place a larger item, it will not return to fill those spaces. To change this behavior, specify the [dense](#valdef-grid-auto-flow-dense) keyword in [grid-auto-flow](#propdef-grid-auto-flow).

### <a id="placement-a11y"></a>8.2.  Grid Item Placement vs. Source Order

> “With great power comes great responsibility.”

<a id="ref-for-grid-placement-property①③"></a>

<a id="ref-for-grid①⑦"></a>

<a id="ref-for-media-query"></a>

The abilities of the [grid-placement properties](#grid-placement-property) allow content to be freely arranged and reordered within the [grid](#grid), such that the visual presentation can be largely disjoint from the underlying document source order. These abilities allow the author great freedom in tailoring the rendering to different devices and modes of presentation e.g. using [media queries](https://www.w3.org/TR/mediaqueries-5/#media-query). However <strong>they are not a substitute for correct source ordering</strong>.

Correct source order is important for speech, for sequential navigation (such as keyboard navigation), and non-CSS UAs such as search engines, tactile browsers, etc. Grid placement <em>only</em> affects the visual presentation! This allows authors to optimize the document source for non-CSS/non-visual interaction modes, and use grid placement techniques to further manipulate the visual presentation so as to leave that source order intact.

<a id="ref-for-propdef-grid-row-start③"></a>

<a id="ref-for-propdef-grid-column-start③"></a>

<a id="ref-for-propdef-grid-row-end③"></a>

<a id="ref-for-propdef-grid-column-end③"></a>

### <a id="line-placement"></a>8.3.  Line-based Placement: the [grid-row-start](#propdef-grid-row-start), [grid-column-start](#propdef-grid-column-start), [grid-row-end](#propdef-grid-row-end), and [grid-column-end](#propdef-grid-column-end) properties

<strong>Table 11 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-grid-row-start"></a>grid-row-start, <a id="propdef-grid-column-start"></a>grid-column-start, <a id="propdef-grid-row-end"></a>grid-row-end, <a id="propdef-grid-column-end"></a>grid-column-end

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-typedef-grid-row-start-grid-line"></a>

[\<grid-line\>](#typedef-grid-row-start-grid-line)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-container⑥⑥"></a>

<a id="ref-for-grid-item⑥③"></a>

[grid items](#grid-item) and absolutely-positioned boxes whose containing block is a [grid container](#grid-container)

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)

<strong>Column 2 (data cell):</strong>

no

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

[Percentages:](https://www.w3.org/TR/css-values/#percentages)

<strong>Column 2 (data cell):</strong>

n/a

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

[Computed value:](https://www.w3.org/TR/css-cascade/#computed)

<strong>Column 2 (data cell):</strong>

specified keyword, identifier, and/or integer

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

<a id="typedef-grid-row-start-grid-line"></a>

<a id="ref-for-typedef-grid-row-start-grid-line①"></a>

<a id="ref-for-comb-one②⑨"></a>

<a id="ref-for-identifier-value③"></a>

<a id="ref-for-comb-one③⓪"></a>

<a id="ref-for-integer-value⑤"></a>

<a id="ref-for-comb-one③①"></a>

<a id="ref-for-integer-value⑥"></a>

<a id="ref-for-comb-all②"></a>

<a id="ref-for-identifier-value④"></a>

<a id="ref-for-mult-opt②③"></a>

<a id="ref-for-comb-one③②"></a>

<a id="ref-for-comb-all③"></a>

<a id="ref-for-integer-value⑦"></a>

<a id="ref-for-comb-any①"></a>

<a id="ref-for-identifier-value⑤"></a>

```text
<grid-line> =
  auto |
  <custom-ident> |
  [ [ <integer [-∞,-1]> | <integer [1,∞]> ] && <custom-ident>? ] |
  [ span && [ <integer [1,∞]> || <custom-ident> ] ]
```
<a id="ref-for-propdef-grid-row-start④"></a>

<a id="ref-for-propdef-grid-column-start④"></a>

<a id="ref-for-propdef-grid-row-end④"></a>

<a id="ref-for-propdef-grid-column-end④"></a>

<a id="ref-for-grid-item⑥④"></a>

<a id="ref-for-grid①⑧"></a>

<a id="ref-for-grid-placement②"></a>

<a id="ref-for-inline-start"></a>

<a id="ref-for-block-start"></a>

<a id="ref-for-inline-end"></a>

<a id="ref-for-block-end"></a>

<a id="ref-for-grid-area①⑨"></a>

The [grid-row-start](#propdef-grid-row-start), [grid-column-start](#propdef-grid-column-start), [grid-row-end](#propdef-grid-row-end), and [grid-column-end](#propdef-grid-column-end) properties determine a [grid item](#grid-item)’s size and location within the [grid](#grid) by contributing a line, a span, or nothing (automatic) to its [grid placement](#grid-placement), thereby specifying the [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start), [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start), [inline-end](https://www.w3.org/TR/css-writing-modes-4/#inline-end), and [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end) edges of its [grid area](#grid-area).

Values have the following meanings:

<a id="ref-for-identifier-value⑥"></a>

<a id="grid-placement-slot"></a>[\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)

<a id="ref-for-grid-placement③"></a>

<a id="ref-for-grid-item⑥⑤"></a>

<a id="ref-for-identifier-value⑧"></a>

<a id="ref-for-identifier-value⑦"></a>

<a id="ref-for-line-name⑨"></a>

<a id="ref-for-grid-line①⑧"></a>

<a id="ref-for-named-grid-area①⑦"></a>

<a id="ref-for-grid-area②⓪"></a>

First attempt to match the [grid area](#grid-area)’s edge to a [named grid area](#named-grid-area): if there is a [grid line](#grid-line) whose [line name](#line-name) is [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)-start (for grid-\*-start) / [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)-end (for grid-\*-end), contributes the first such line to the [grid item](#grid-item)’s [placement](#grid-placement).

<a id="ref-for-named-grid-area①⑧"></a>

<a id="ref-for-implicitly-assigned-line-name⑥"></a>

<a id="ref-for-propdef-grid-row-start⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Named grid areas](#named-grid-area) automatically generate [implicitly-assigned line names](#implicitly-assigned-line-name) of this form, so specifying [grid-row-start: foo](#propdef-grid-row-start) will choose the start edge of that <a id="ref-for-named-grid-area①⑨"></a>named grid area (unless another line named foo-start was explicitly specified before it).

<a id="ref-for-identifier-value⑨"></a>

Otherwise, treat this as if the integer 1 had been specified along with the [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value).

<a id="ref-for-identifier-value①⓪"></a>

<a id="ref-for-integer-value⑧"></a>

<a id="grid-placement-int"></a>\[ [\<integer \[-∞,-1\]\>](https://www.w3.org/TR/css-values-4/#integer-value) \| <a id="ref-for-integer-value⑨"></a>\<integer \[1,∞\]\> \] &#x26;&#x26; [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value)?

<a id="ref-for-explicit-grid②②"></a>

<a id="ref-for-grid-placement④"></a>

<a id="ref-for-grid-item⑥⑥"></a>

<a id="ref-for-grid-line①⑨"></a>

Contributes the <var>N</var>th [grid line](#grid-line) to the [grid item](#grid-item)’s [placement](#grid-placement). If a negative integer is given, it instead counts in reverse, starting from the end edge of the [explicit grid](#explicit-grid).

<a id="ref-for-identifier-value①①"></a>

<a id="ref-for-implicit-grid-lines"></a>

If a name is given as a [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value), only lines with that name are counted. If not enough lines with that name exist, all [implicit grid lines](#implicit-grid-lines) are assumed to have that name for the purpose of finding this position.

<a id="ref-for-integer-value①⓪"></a>

An [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) value of zero makes the declaration invalid.

<a id="ref-for-identifier-value①②"></a>

<a id="ref-for-integer-value①①"></a>

<a id="grid-placement-span-int"></a>span &#x26;&#x26; \[ [\<integer \[1,∞\]\>](https://www.w3.org/TR/css-values-4/#integer-value) \|\| [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) \]

<a id="ref-for-propdef-grid-column-start⑤"></a>

<a id="ref-for-propdef-grid-column-end⑤"></a>

<a id="ref-for-grid-area②①"></a>

<a id="ref-for-grid-placement⑤"></a>

<a id="ref-for-grid-item⑥⑦"></a>

<a id="ref-for-grid-span⑨"></a>

Contributes a [grid span](#grid-span) to the [grid item](#grid-item)’s [placement](#grid-placement) such that the corresponding edge of the <a id="ref-for-grid-item⑥⑧"></a>grid item’s [grid area](#grid-area) is <var>N</var> lines from its opposite edge in the corresponding direction. For example, [grid-column-end: span 2](#propdef-grid-column-end) indicates the second grid line in the endward direction from the [grid-column-start](#propdef-grid-column-start) line.

<a id="ref-for-identifier-value①③"></a>

<a id="ref-for-implicit-grid-lines①"></a>

<a id="ref-for-explicit-grid②③"></a>

If a name is given as a [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value), only lines with that name are counted. If not enough lines with that name exist, all [implicit grid lines](#implicit-grid-lines) on the side of the [explicit grid](#explicit-grid) corresponding to the search direction are assumed to have that name for the purpose of counting this span.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4e2ea185"></a> For example, given the following declarations:
>
> ```css
> .grid { grid-template-columns: 100px; }
> .griditem { grid-column: span foo / 4; }
> ```
>
> <a id="ref-for-grid-container⑥⑦"></a>
>
> <a id="ref-for-explicit-grid②④"></a>
>
> <a id="ref-for-grid-item⑥⑨"></a>
>
> <a id="ref-for-implicit-grid②"></a>
>
> The [grid container](#grid-container) has an [explicit grid](#explicit-grid) with two grid lines, numbered 1 and 2. The [grid item’s](#grid-item) column-end edge is specified to be at line 4, so two lines are generated in the endward side of the [implicit grid](#implicit-grid).
>
> <a id="ref-for-implicit-grid③"></a>
>
> <a id="ref-for-explicit-grid②⑤"></a>
>
> <a id="ref-for-propdef-grid-column-start⑥"></a>
>
> Its column-start edge must be the first "foo" line it can find startward of that. There is no "foo" line in the grid, though, so the only possibility is a line in the [implicit grid](#implicit-grid). Line 3 is not a candidate, because it’s on the endward side of the [explicit grid](#explicit-grid), while the [grid-column-start](#propdef-grid-column-start) span forces it to search startward. So, the only option is for the <a id="ref-for-implicit-grid④"></a>implicit grid to generate a line on the startward side of the <a id="ref-for-explicit-grid②⑥"></a>explicit grid.
>
> ![](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/implicit-lines-search.svg)
>
> An illustration of the result.

<a id="ref-for-integer-value①②"></a>

If the [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) is omitted, it defaults to 1. Negative integers or zero are invalid.

<a id="grid-placement-auto"></a>auto

<a id="ref-for-auto-placement②"></a>

<a id="ref-for-grid-placement⑥"></a>

<a id="ref-for-grid-item⑦⓪"></a>

The property contributes nothing to the [grid item](#grid-item)’s [placement](#grid-placement), indicating [auto-placement](#auto-placement) or a default span of one. (See [§ 8 Placing Grid Items](#placement), above.)

<a id="ref-for-identifier-value①④"></a>

In all the above productions, the [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value) additionally excludes the keywords span and auto.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4a841210"></a> Given a single-row, 8-column grid and the following 9 named lines:
>
> ```text
> 1  2  3  4  5  6  7  8  9
> +--+--+--+--+--+--+--+--+
> |  |  |  |  |  |  |  |  |
> A  B  C  A  B  C  A  B  C
> |  |  |  |  |  |  |  |  |
> +--+--+--+--+--+--+--+--+
> ```
>
> The following declarations place the grid item between the lines indicated by index:
>
> ```text
> grid-column-start: 4; grid-column-end: auto;
> /* Line 4 to line 5 */
> 
> grid-column-start: auto; grid-column-end: 6;
> /* Line 5 to line 6 */
> 
> grid-column-start: C; grid-column-end: C -1;
> /* Line 3 to line 9 */
> 
> grid-column-start: C; grid-column-end: span C;
> /* Line 3 to line 6 */
> 
> grid-column-start: span C; grid-column-end: C -1;
> /* Line 6 to line 9 */
> 
> grid-column-start: span C; grid-column-end: span C;
> /* Error: The end span is ignored, and an auto-placed
>    item can’t span to a named line.
>    Equivalent to ''grid-column: span 1;''. */
> 
> grid-column-start: 5; grid-column-end: C -1;
> /* Line 5 to line 9 */
> 
> grid-column-start: 5; grid-column-end: span C;
> /* Line 5 to line 6 */
> 
> grid-column-start: 8; grid-column-end: 8;
> /* Error: line 8 to line 9 */
> 
> grid-column-start: B 2; grid-column-end: span 1;
> /* Line 5 to line 6 */
> ```
#### <a id="grid-placement-errors"></a>8.3.1.  Grid Placement Conflict Handling

<a id="ref-for-grid-placement⑦"></a>

<a id="ref-for-grid-item⑦①"></a>

<a id="ref-for-start②"></a>

<a id="ref-for-end②"></a>

If the [placement](#grid-placement) for a [grid item](#grid-item) contains two lines, and the [start](https://www.w3.org/TR/css-writing-modes-4/#start) line is further end-ward than the [end](https://www.w3.org/TR/css-writing-modes-4/#end) line, swap the two lines. If the <a id="ref-for-start③"></a>start line is <em>equal</em> to the <a id="ref-for-end③"></a>end line, remove the <a id="ref-for-end④"></a>end line.

<a id="ref-for-grid-placement⑧"></a>

<a id="ref-for-end⑤"></a>

<a id="ref-for-grid-placement-property①④"></a>

If the [placement](#grid-placement) contains two spans, remove the one contributed by the [end](https://www.w3.org/TR/css-writing-modes-4/#end) [grid-placement property](#grid-placement-property).

<a id="ref-for-grid-placement⑨"></a>

If the [placement](#grid-placement) contains only a span for a named line, replace it with a span of 1.

<a id="ref-for-propdef-grid-column④"></a>

<a id="ref-for-propdef-grid-row④"></a>

<a id="ref-for-propdef-grid-area④"></a>

### <a id="placement-shorthands"></a>8.4.  Placement Shorthands: the [grid-column](#propdef-grid-column), [grid-row](#propdef-grid-row), and [grid-area](#propdef-grid-area) properties

<strong>Table 12 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-grid-row"></a>grid-row, <a id="propdef-grid-column"></a>grid-column

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-opt②④"></a>

<a id="ref-for-typedef-grid-row-start-grid-line②"></a>

[\<grid-line\>](#typedef-grid-row-start-grid-line) \[ / <a id="ref-for-typedef-grid-row-start-grid-line③"></a>\<grid-line\> \][?](https://www.w3.org/TR/css-values-4/#mult-opt)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-container⑥⑧"></a>

<a id="ref-for-grid-item⑦②"></a>

[grid items](#grid-item) and absolutely-positioned boxes whose containing block is a [grid container](#grid-container)

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

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<a id="ref-for-propdef-grid-row⑤"></a>

<a id="ref-for-propdef-grid-column⑤"></a>

<a id="ref-for-propdef-grid-row-start⑥"></a>

<a id="ref-for-propdef-grid-row-end⑤"></a>

<a id="ref-for-propdef-grid-column-start⑦"></a>

<a id="ref-for-propdef-grid-column-end⑥"></a>

The [grid-row](#propdef-grid-row) and [grid-column](#propdef-grid-column) properties are shorthands for [grid-row-start](#propdef-grid-row-start)/[grid-row-end](#propdef-grid-row-end) and [grid-column-start](#propdef-grid-column-start)/[grid-column-end](#propdef-grid-column-end), respectively.

<a id="ref-for-typedef-grid-row-start-grid-line④"></a>

<a id="ref-for-propdef-grid-row-start⑦"></a>

<a id="ref-for-propdef-grid-column-start⑧"></a>

<a id="ref-for-propdef-grid-row-end⑥"></a>

<a id="ref-for-propdef-grid-column-end⑦"></a>

If two [\<grid-line\>](#typedef-grid-row-start-grid-line) values are specified, the [grid-row-start](#propdef-grid-row-start)/[grid-column-start](#propdef-grid-column-start) longhand is set to the value before the slash, and the [grid-row-end](#propdef-grid-row-end)/[grid-column-end](#propdef-grid-column-end) longhand is set to the value after the slash.

<a id="ref-for-identifier-value①⑤"></a>

<a id="ref-for-propdef-grid-row-end⑦"></a>

<a id="ref-for-propdef-grid-column-end⑧"></a>

<a id="ref-for-grid-placement-auto③"></a>

When the second value is omitted, if the first value is a [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value), the [grid-row-end](#propdef-grid-row-end)/[grid-column-end](#propdef-grid-column-end) longhand is also set to that <a id="ref-for-identifier-value①⑥"></a>\<custom-ident\>; otherwise, it is set to [auto](#grid-placement-auto).

<strong>Table 13 — structured row/cell transcription</strong>

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Name:

<strong>Column 2 (data cell):</strong>

<a id="propdef-grid-area"></a>grid-area

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

[Value:](https://www.w3.org/TR/css-values/#value-defs)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-mult-num-range"></a>

<a id="ref-for-typedef-grid-row-start-grid-line⑤"></a>

[\<grid-line\>](#typedef-grid-row-start-grid-line) \[ / <a id="ref-for-typedef-grid-row-start-grid-line⑥"></a>\<grid-line\> \][{0,3}](https://www.w3.org/TR/css-values-4/#mult-num-range)

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)

<strong>Column 2 (data cell):</strong>

auto

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)

<strong>Column 2 (data cell):</strong>

<a id="ref-for-grid-container⑥⑨"></a>

<a id="ref-for-grid-item⑦③"></a>

[grid items](#grid-item) and absolutely-positioned boxes whose containing block is a [grid container](#grid-container)

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

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)

<strong>Column 2 (data cell):</strong>

discrete

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)

<strong>Column 2 (data cell):</strong>

per grammar

<a id="ref-for-propdef-grid-area⑤"></a>

<a id="ref-for-shorthand-property⑤"></a>

<a id="ref-for-propdef-grid-row-start⑧"></a>

<a id="ref-for-propdef-grid-column-start⑨"></a>

<a id="ref-for-propdef-grid-row-end⑧"></a>

<a id="ref-for-propdef-grid-column-end⑨"></a>

The [grid-area](#propdef-grid-area) property is a [shorthand](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for [grid-row-start](#propdef-grid-row-start), [grid-column-start](#propdef-grid-column-start), [grid-row-end](#propdef-grid-row-end) and [grid-column-end](#propdef-grid-column-end).

<a id="ref-for-typedef-grid-row-start-grid-line⑦"></a>

<a id="ref-for-propdef-grid-row-start⑨"></a>

<a id="ref-for-propdef-grid-column-start①⓪"></a>

<a id="ref-for-propdef-grid-row-end⑨"></a>

<a id="ref-for-propdef-grid-column-end①⓪"></a>

If four [\<grid-line\>](#typedef-grid-row-start-grid-line) values are specified, [grid-row-start](#propdef-grid-row-start) is set to the first value, [grid-column-start](#propdef-grid-column-start) is set to the second value, [grid-row-end](#propdef-grid-row-end) is set to the third value, and [grid-column-end](#propdef-grid-column-end) is set to the fourth value.

<a id="ref-for-propdef-grid-column-end①①"></a>

<a id="ref-for-propdef-grid-column-start①①"></a>

<a id="ref-for-identifier-value①⑦"></a>

<a id="ref-for-grid-placement-auto④"></a>

When [grid-column-end](#propdef-grid-column-end) is omitted, if [grid-column-start](#propdef-grid-column-start) is a [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value), <a id="ref-for-propdef-grid-column-end①②"></a>grid-column-end is set to that <a id="ref-for-identifier-value①⑧"></a>\<custom-ident\>; otherwise, it is set to [auto](#grid-placement-auto).

<a id="ref-for-propdef-grid-row-end①⓪"></a>

<a id="ref-for-propdef-grid-row-start①⓪"></a>

<a id="ref-for-identifier-value①⑨"></a>

<a id="ref-for-grid-placement-auto⑤"></a>

When [grid-row-end](#propdef-grid-row-end) is omitted, if [grid-row-start](#propdef-grid-row-start) is a [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value), <a id="ref-for-propdef-grid-row-end①①"></a>grid-row-end is set to that <a id="ref-for-identifier-value②⓪"></a>\<custom-ident\>; otherwise, it is set to [auto](#grid-placement-auto).

<a id="ref-for-propdef-grid-column-start①②"></a>

<a id="ref-for-propdef-grid-row-start①①"></a>

<a id="ref-for-identifier-value②①"></a>

<a id="ref-for-grid-placement-auto⑥"></a>

When [grid-column-start](#propdef-grid-column-start) is omitted, if [grid-row-start](#propdef-grid-row-start) is a [\<custom-ident\>](https://www.w3.org/TR/css-values-4/#identifier-value), all four longhands are set to that value. Otherwise, it is set to [auto](#grid-placement-auto).

<a id="ref-for-propdef-margin"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The resolution order for this shorthand is row-start/column-start/row-end/column-end, which goes CCW for LTR pages, the opposite direction of the related 4-edge properties using physical directions, like [margin](https://www.w3.org/TR/CSS2/box.html#propdef-margin).

### <a id="auto-placement-algo"></a>8.5.  Grid Item Placement Algorithm

<a id="ref-for-automatic-grid-position"></a>

<a id="ref-for-grid-item⑦④"></a>

<a id="ref-for-definite-grid-position"></a>

<a id="ref-for-grid-area②②"></a>

<a id="ref-for-grid-span①⓪"></a>

The following <a id="grid-item-placement-algorithm"></a>grid item placement algorithm resolves [automatic positions](#automatic-grid-position) of [grid items](#grid-item) into [definite positions](#definite-grid-position), ensuring that every <a id="ref-for-grid-item⑦⑤"></a>grid item has a well-defined [grid area](#grid-area) to lay out into. ([Grid spans](#grid-span) need no special resolution; if they’re not explicitly specified, they default to 1.)

<a id="ref-for-implicit-grid⑤"></a>

<a id="ref-for-explicit-grid②⑦"></a>

<a id="ref-for-grid-item⑦⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This algorithm can result in the creation of new rows or columns in the [implicit grid](#implicit-grid), if there is no room in the [explicit grid](#explicit-grid) to place an auto-positioned [grid item](#grid-item).

<a id="ref-for-grid-cell⑤"></a>

<a id="ref-for-explicit-grid②⑧"></a>

<a id="ref-for-implicit-grid⑥"></a>

<a id="ref-for-occupied"></a>

<a id="ref-for-grid-area②③"></a>

<a id="ref-for-grid-item⑦⑦"></a>

<a id="ref-for-definite-grid-position①"></a>

<a id="ref-for-unoccupied"></a>

Every [grid cell](#grid-cell) (in both the [explicit](#explicit-grid) and [implicit grids](#implicit-grid)) can be <a id="occupied"></a>occupied or <a id="unoccupied"></a>unoccupied. A cell is [occupied](#occupied) if it’s covered by the [grid area](#grid-area) of a [grid item](#grid-item) with a [definite grid position](#definite-grid-position); otherwise, the cell is [unoccupied](#unoccupied). A cell’s <a id="ref-for-occupied①"></a>occupied/<a id="ref-for-unoccupied①"></a>unoccupied status can change during this algorithm.

<a id="ref-for-propdef-grid-auto-flow⑨"></a>

<a id="ref-for-valdef-grid-auto-flow-row④"></a>

<a id="ref-for-valdef-grid-auto-flow-column②"></a>

To aid in clarity, this algorithm is written with the assumption that [grid-auto-flow](#propdef-grid-auto-flow) has [row](#valdef-grid-auto-flow-row) specified. If it is instead set to [column](#valdef-grid-auto-flow-column), swap all mentions of rows and columns, inline and block, etc. in this algorithm.

<a id="ref-for-grid-item-placement-algorithm⑧"></a>

<a id="ref-for-grid-item⑦⑧"></a>

<a id="ref-for-order-modified-document-order②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [auto-placement algorithm](#grid-item-placement-algorithm) works with the [grid items](#grid-item) in [order-modified document order](https://www.w3.org/TR/css-flexbox-1/#order-modified-document-order), not their original document order.

0.  <a id="ref-for-grid-item⑦⑨"></a>

    <a id="ref-for-grid-placement-property①⑤"></a>

    <strong>Generate anonymous grid items</strong> as described in [§ 6 Grid Items](#grid-items). (Anonymous [grid items](#grid-item) are always auto-placed, since their boxes can’t have any [grid-placement properties](#grid-placement-property) specified.)

1.  <strong>Position anything that’s not auto-positioned.</strong>

2.  <strong>Process the items locked to a given row.</strong>

    <a id="ref-for-grid-item⑧⓪"></a>

    <a id="ref-for-definite-grid-position②"></a>

    <a id="ref-for-propdef-grid-row-start①②"></a>

    <a id="ref-for-propdef-grid-row-end①②"></a>

    <a id="ref-for-order-modified-document-order③"></a>

    For each [grid item](#grid-item) with a [definite row position](#definite-grid-position) (that is, the [grid-row-start](#propdef-grid-row-start) and [grid-row-end](#propdef-grid-row-end) properties define a <a id="ref-for-definite-grid-position③"></a>definite grid position), in [order-modified document order](https://www.w3.org/TR/css-flexbox-1/#order-modified-document-order):

    “sparse” packing (default behavior)

    <a id="ref-for-grid-placement①⓪"></a>

    <a id="ref-for-grid-area②④"></a>

    <a id="ref-for-occupied②"></a>

    <a id="ref-for-grid-item⑧①"></a>

    Set the column-start line of its [placement](#grid-placement) to the earliest (smallest positive index) line index that ensures this item’s [grid area](#grid-area) will not overlap any [occupied](#occupied) grid cells and that is past any [grid items](#grid-item) previously placed in this row by this step.

    <a id="ref-for-valdef-grid-auto-flow-dense②"></a>

    “dense” packing ([dense](#valdef-grid-auto-flow-dense) specified)

    <a id="ref-for-grid-placement①①"></a>

    <a id="ref-for-grid-area②⑤"></a>

    <a id="ref-for-occupied③"></a>

    Set the column-start line of its [placement](#grid-placement) to the earliest (smallest positive index) line index that ensures this item’s [grid area](#grid-area) will not overlap any [occupied](#occupied) grid cells.

3.  <strong>Determine the columns in the implicit grid.</strong>

    <a id="ref-for-implicit-grid⑦"></a>

    Create columns in the [implicit grid](#implicit-grid):

    1.  <a id="ref-for-explicit-grid②⑨"></a>

        Start with the columns from the [explicit grid](#explicit-grid).

    2.  <a id="ref-for-definite-grid-position④"></a>

        <a id="ref-for-implicit-grid⑧"></a>

        Among all the items with a [definite column position](#definite-grid-position) (explicitly positioned items, items positioned in the previous step, and items not yet positioned but with a definite column) add columns to the beginning and end of the [implicit grid](#implicit-grid) as necessary to accommodate those items.

    3.  <a id="ref-for-grid-span①①"></a>

        <a id="ref-for-definite-grid-position⑤"></a>

        <a id="ref-for-implicit-grid⑨"></a>

        If the largest [column span](#grid-span) among all the items <em>without</em> a [definite column position](#definite-grid-position) is larger than the width of the [implicit grid](#implicit-grid), add columns to the end of the <a id="ref-for-implicit-grid①⓪"></a>implicit grid to accommodate that <a id="ref-for-grid-span①②"></a>column span.

    > <strong data-conversion-semantic="example">Example</strong>
    >
    > <a id="example-236efe03"></a> For example, in the following style fragment:
    > ```text
    > #grid {
    >   display: grid;
    >   grid-template-columns: repeat(5, 100px);
    >   grid-auto-flow: row;
    > }
    > #grid-item {
    >   grid-column: 4 / span 3;
    > }
    > ```
    >
    > <a id="ref-for-explicit-grid③⓪"></a>
    >
    > <a id="ref-for-propdef-grid-template-columns③⓪"></a>
    >
    > <a id="ref-for-implicit-grid①①"></a>
    >
    > The number of columns needed is 6. The [explicit grid](#explicit-grid) provides 5 columns (from [grid-template-columns](#propdef-grid-template-columns)) with lines number 1 through 6, but `#grid-item`’s column position means it ends on line 7, which requires an additional column added to the end of the [implicit grid](#implicit-grid).

4.  <strong>Position the remaining grid items.</strong>

    <a id="ref-for-grid-line②⓪"></a>

    <a id="ref-for-auto-placement-cursor"></a>

    <a id="ref-for-implicit-grid①②"></a>

    The <a id="auto-placement-cursor"></a>auto-placement cursor defines the current “insertion point” in the grid, specified as a pair of row and column [grid lines](#grid-line). Initially the [auto-placement cursor](#auto-placement-cursor) is set to the start-most row and column lines in the [implicit grid](#implicit-grid).

    <a id="ref-for-propdef-grid-auto-flow①⓪"></a>

    The [grid-auto-flow](#propdef-grid-auto-flow) value in use determines how to position the items:

    “sparse” packing (default behavior)

    <a id="ref-for-grid-item⑧②"></a>

    <a id="ref-for-order-modified-document-order④"></a>

    For each [grid item](#grid-item) that hasn’t been positioned by the previous steps, in [order-modified document order](https://www.w3.org/TR/css-flexbox-1/#order-modified-document-order):

    <a id="ref-for-definite-grid-position⑥"></a>

    If the item has a [definite column position](#definite-grid-position):

    1.  <a id="ref-for-auto-placement-cursor①"></a>

        <a id="ref-for-grid-item⑧③"></a>

        Set the column position of the [cursor](#auto-placement-cursor) to the [grid item’s](#grid-item) column-start line. If this is less than the previous column position of the <a id="ref-for-auto-placement-cursor②"></a>cursor, increment the row position by 1.

    2.  <a id="ref-for-auto-placement-cursor③"></a>

        <a id="ref-for-grid-item⑧④"></a>

        <a id="ref-for-occupied④"></a>

        <a id="ref-for-implicit-grid①③"></a>

        Increment the [cursor](#auto-placement-cursor)’s row position until a value is found where the [grid item](#grid-item) does not overlap any [occupied](#occupied) grid cells (creating new rows in the [implicit grid](#implicit-grid) as necessary).

    3.  <a id="ref-for-auto-placement-cursor④"></a>

        Set the item’s row-start line to the [cursor’s](#auto-placement-cursor) row position, and set the item’s row-end line according to its span from that position.

    <a id="ref-for-automatic-grid-position①"></a>

    If the item has an [automatic grid position](#automatic-grid-position) in both axes:

    1.  <a id="ref-for-auto-placement-cursor⑤"></a>

        <a id="ref-for-grid-area②⑥"></a>

        <a id="ref-for-occupied⑤"></a>

        Increment the column position of the [auto-placement cursor](#auto-placement-cursor) until either this item’s [grid area](#grid-area) does not overlap any [occupied](#occupied) grid cells, or the <a id="ref-for-auto-placement-cursor⑥"></a>cursor’s column position, plus the item’s column span, overflow the number of columns in the implicit grid, as determined earlier in this algorithm.

    2.  <a id="ref-for-auto-placement-cursor⑦"></a>

        <a id="ref-for-implicit-grid①④"></a>

        If a non-overlapping position was found in the previous step, set the item’s row-start and column-start lines to the [cursor’s](#auto-placement-cursor) position. Otherwise, increment the <a id="ref-for-auto-placement-cursor⑧"></a>auto-placement cursor’s row position (creating new rows in the [implicit grid](#implicit-grid) as necessary), set its column position to the start-most column line in the <a id="ref-for-implicit-grid①⑤"></a>implicit grid, and return to the previous step.

    <a id="ref-for-valdef-grid-auto-flow-dense③"></a>

    “dense” packing ([dense](#valdef-grid-auto-flow-dense) specified)

    <a id="ref-for-grid-item⑧⑤"></a>

    <a id="ref-for-order-modified-document-order⑤"></a>

    For each [grid item](#grid-item) that hasn’t been positioned by the previous steps, in [order-modified document order](https://www.w3.org/TR/css-flexbox-1/#order-modified-document-order):

    <a id="ref-for-definite-grid-position⑦"></a>

    If the item has a [definite column position](#definite-grid-position):

    1.  <a id="ref-for-implicit-grid①⑥"></a>

        <a id="ref-for-grid-item⑧⑥"></a>

        Set the row position of the cursor to the start-most row line in the [implicit grid](#implicit-grid). Set the column position of the cursor to the [grid item’s](#grid-item) column-start line.

    2.  <a id="ref-for-auto-placement-cursor⑨"></a>

        <a id="ref-for-grid-item⑧⑦"></a>

        <a id="ref-for-occupied⑥"></a>

        <a id="ref-for-implicit-grid①⑦"></a>

        Increment the [auto-placement cursor](#auto-placement-cursor)’s row position until a value is found where the [grid item](#grid-item) does not overlap any [occupied](#occupied) grid cells (creating new rows in the [implicit grid](#implicit-grid) as necessary).

    3.  <a id="ref-for-auto-placement-cursor①⓪"></a>

        Set the item’s row-start line index to the [cursor’s](#auto-placement-cursor) row position. (Implicitly setting the item’s row-end line according to its span, as well.)

    <a id="ref-for-automatic-grid-position②"></a>

    If the item has an [automatic grid position](#automatic-grid-position) in both axes:

    1.  <a id="ref-for-implicit-grid①⑧"></a>

        Set the cursor’s row and column positions to start-most row and column lines in the [implicit grid](#implicit-grid).

    2.  <a id="ref-for-auto-placement-cursor①①"></a>

        <a id="ref-for-grid-area②⑦"></a>

        <a id="ref-for-occupied⑦"></a>

        Increment the column position of the [auto-placement cursor](#auto-placement-cursor) until either this item’s [grid area](#grid-area) does not overlap any [occupied](#occupied) grid cells, or the <a id="ref-for-auto-placement-cursor①②"></a>cursor’s column position, plus the item’s column span, overflow the number of columns in the implicit grid, as determined earlier in this algorithm.

    3.  <a id="ref-for-auto-placement-cursor①③"></a>

        <a id="ref-for-implicit-grid①⑨"></a>

        If a non-overlapping position was found in the previous step, set the item’s row-start and column-start lines to the [cursor’s](#auto-placement-cursor) position. Otherwise, increment the <a id="ref-for-auto-placement-cursor①④"></a>auto-placement cursor’s row position (creating new rows in the [implicit grid](#implicit-grid) as necessary), reset its column position to the start-most column line in the <a id="ref-for-implicit-grid②⓪"></a>implicit grid, and return to the previous step.

## <a id="subgrids"></a>9.  Subgrids

<a id="ref-for-subgrid①⑤"></a>

<a id="ref-for-grid-container⑦⓪"></a>

A [subgrid](#subgrid) behaves just like a normal [grid container](#grid-container) except that:

- <a id="ref-for-explicit-grid③①"></a>

  <a id="ref-for-parent-grid⑧"></a>

  <a id="ref-for-subgridded-axis⑤"></a>

  <a id="ref-for-subgrid①⑥"></a>

  <a id="subgrid-tracks"></a> Placing the [subgrid](#subgrid) creates a correspondence between its [subgridded](#subgridded-axis) tracks and those that it spans in its [parent grid](#parent-grid). The grid lines thus shared between the <a id="ref-for-subgrid①⑦"></a>subgrid and its parent form the subgrid’s [explicit grid](#explicit-grid), and its track sizes are governed by the <a id="ref-for-parent-grid⑨"></a>parent grid.

- <a id="ref-for-grid①⑨"></a>

  <a id="ref-for-grid-track②④"></a>

  <a id="ref-for-subgridded-axis⑥"></a>

  <a id="ref-for-subgrid①⑧"></a>

  <a id="subgrid-span"></a> The number of explicit tracks in the [subgrid](#subgrid) in a [subgridded](#subgridded-axis) dimension always corresponds to the number of [grid tracks](#grid-track) that it spans in its parent [grid](#grid):

  - <a id="ref-for-implicit-grid-span①"></a>

    <a id="ref-for-explicit-grid-span①"></a>

    <a id="ref-for-grid-span①③"></a>

    <a id="ref-for-subgrid①⑨"></a>

    If the [subgrid](#subgrid)’s [grid span](#grid-span) in the subgridded dimension is [explicit](#explicit-grid-span) or [implicit](#implicit-grid-span), then the number of explicit tracks in each subgridded dimension is taken from its used <a id="ref-for-grid-span①④"></a>grid span in that dimension (regardless of its grid-template-\* properties).

  - <a id="ref-for-grid-span①⑤"></a>

    <a id="ref-for-automatic-grid-span①"></a>

    If it has an [automatic grid span](#automatic-grid-span), then its used [grid span](#grid-span) is taken from the number of explicit tracks specified for that axis by its grid-template-\* properties, floored at one.

  <a id="ref-for-subgrid②⓪"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: The explicit grid determined here can be further truncated if the placement of the [subgrid](#subgrid) is clamped by its parent grid. See [the "no implicit tracks" bullet point](#subgrid-implicit).

  <a id="ref-for-typedef-line-name-list⑥"></a>

  <a id="ref-for-subgridded-axis⑦"></a>

  If the grid-template-\* properties specify a [\<line-name-list\>](#typedef-line-name-list) in a [subgridded axis](#subgridded-axis), the used value is truncated to match the used number of explicit tracks.

- <a id="ref-for-parent-grid①⓪"></a>

  <a id="ref-for-explicit-grid③②"></a>

  <a id="ref-for-grid-item⑧⑧"></a>

  <a id="ref-for-subgrid②①"></a>

  <a id="ref-for-grid-placement-property①⑥"></a>

  <a id="subgrid-indexing"></a> The [grid-placement properties](#grid-placement-property) of the [subgrid](#subgrid)’s [grid items](#grid-item) and the line numbers they use are scoped to the lines covered by the <a id="ref-for-subgrid②②"></a>subgrid, exactly consistent with the lines outside the <a id="ref-for-subgrid②③"></a>subgrid being excluded from its [explicit grid](#explicit-grid). E.g. numeric indices count starting from the first line of the <a id="ref-for-subgrid②④"></a>subgrid rather than the first line of the [parent grid](#parent-grid).

  <a id="ref-for-subgrid②⑤"></a>

  <a id="ref-for-writing-mode②"></a>

  Line numbering and placement rules obey the [subgrid](#subgrid)’s own [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode), just as they would for a nested independent grid.

- <a id="ref-for-grid②⓪"></a>

  <a id="ref-for-explicitly-assigned-line-name①"></a>

  <a id="ref-for-subgridded-axis⑧"></a>

  <a id="ref-for-subgrid②⑥"></a>

  <a id="subgrid-line-name-inheritance"></a> Since [subgrids](#subgrid) can be placed before their contents are placed, the [subgridded](#subgridded-axis) lines automatically receive the [explicitly-assigned line names](#explicitly-assigned-line-name) specified on the corresponding lines of the parent [grid](#grid).

  <a id="ref-for-line-name①⓪"></a>

  <a id="ref-for-subgrid②⑦"></a>

  These names are in <em>addition</em> to any [line names](#line-name) specified locally on the [subgrid](#subgrid).

- <a id="ref-for-implicitly-assigned-line-name⑦"></a>

  <a id="ref-for-propdef-grid-template-areas②②"></a>

  <a id="ref-for-named-grid-area②⓪"></a>

  <a id="ref-for-subgrid②⑧"></a>

  <a id="subgrid-area-inheritance"></a> When a [subgrid](#subgrid) overlaps a [named grid area](#named-grid-area) in its parent that was created by a [grid-template-areas](#propdef-grid-template-areas) property declaration, [implicitly-assigned line names](#implicitly-assigned-line-name) are assigned to represent the parent’s <a id="ref-for-named-grid-area②①"></a>named grid area within the <a id="ref-for-subgrid②⑨"></a>subgrid.

  <a id="ref-for-named-grid-area②②"></a>

  <a id="ref-for-subgrid③⓪"></a>

  <a id="ref-for-implicitly-assigned-line-name⑧"></a>

  <a id="ref-for-line-name①①"></a>

  <a id="ref-for-grid②①"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: If a [named grid area](#named-grid-area) only partially overlaps the [subgrid](#subgrid), its [implicitly-assigned line names](#implicitly-assigned-line-name) will be assigned to the first and/or last line of the <a id="ref-for-subgrid③①"></a>subgrid such that a <a id="ref-for-named-grid-area②③"></a>named grid area exists representing that partially overlapped area of the <a id="ref-for-subgrid③②"></a>subgrid; thus the [line name](#line-name) assignments of the <a id="ref-for-subgrid③③"></a>subgrid might not always correspond exactly to the <a id="ref-for-line-name①②"></a>line name assignments of the parent [grid](#grid).

  <a id="ref-for-line-name①③"></a>

  <a id="ref-for-subgrid③④"></a>

  These names are also in <em>addition</em> to any [line names](#line-name) specified locally on the [subgrid](#subgrid).

  <a id="ref-for-propdef-grid-template-areas②③"></a>

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-fffff4da"></a> In the following example, the 4-column grand-parent grid has both explicit line names and implicit ones generated by [grid-template-areas](#propdef-grid-template-areas):
  > ```text
  > <style type="css">
  >   .outer {
  >     display: grid;
  >     grid-template-columns:
  >       [outer-edge] 20px [main-start] 1fr [center] 1fr max-content [main-end];
  >     grid-template-areas:
  >       "gutter info info photos";
  >   }
  >   .middle {
  >     grid-column: main-start / main-end;
  >     display: grid;
  >     grid: subgrid / subgrid;
  >   }
  >   .inner {
  >     grid-column: center / -1;
  >     display: grid;
  >     grid: subgrid / subgrid;
  >   }
  > </style>
  > 
  > <div class="outer">
  >   <div class="middle">
  >     <div class="inner">&hellip;</div>
  >   </div>
  > </div>
  > ```
  >
  > After all types of name resolution, the names for each grid will be:
  >
  > ```text
  > .outer = [outer-edge gutter-start] [gutter-end info-start main-start] [center] [info-end photos-start] [main-end photos-end]
  > .middle = [info-start main-start] [center] [info-end photos-start] [main-end photos-end]
  > .inner = [center info-start] [info-end photos-start] [main-end photos-end]
  > ```
  >
  > <a id="ref-for-explicitly-assigned-line-name②"></a>
  >
  > <a id="ref-for-implicitly-assigned-line-name⑨"></a>
  >
  > <a id="ref-for-subgrid③⑤"></a>
  >
  > <a id="ref-for-named-grid-area②④"></a>
  >
  > Notice that all the [explicitly-assigned line names](#explicitly-assigned-line-name) inherit straight through to .inner, but the [implicitly-assigned line names](#implicitly-assigned-line-name) are calculated based on each [subgrid](#subgrid)’s overlap of the original [named grid area](#named-grid-area).

- <a id="ref-for-clamp-a-grid-area①"></a>

  <a id="ref-for-grid-area②⑧"></a>

  <a id="ref-for-grid-item⑧⑨"></a>

  <a id="ref-for-explicit-grid③③"></a>

  <a id="ref-for-implicit-grid-lines②"></a>

  <a id="ref-for-subgridded-axis⑨"></a>

  <a id="ref-for-implicit-grid-track④"></a>

  <a id="ref-for-subgrid③⑥"></a>

  <a id="subgrid-implicit"></a> The [subgrid](#subgrid) does not have any [implicit grid tracks](#implicit-grid-track) in the [subgridded](#subgridded-axis) dimension(s). Hypothetical [implicit grid lines](#implicit-grid-lines) are used to resolve placement as usual when the [explicit grid](#explicit-grid) does not have enough lines; however each [grid item](#grid-item)’s [grid area](#grid-area) is [clamped](#clamp-a-grid-area) to the <a id="ref-for-subgrid③⑦"></a>subgrid’s <a id="ref-for-explicit-grid③④"></a>explicit grid (using the same procedure as for clamping placement in an overly-large grid).

  <a id="ref-for-grid-item⑨⓪"></a>

  <a id="ref-for-propdef-grid-column⑥"></a>

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-f40fdc9d"></a> For example, if a span 1 subgrid has a [grid item](#grid-item) with [grid-column: 2 / span 3;](#propdef-grid-column), then that item is instead forced into (and limited to) the first (only) track in the subgrid.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This means that a subgrid might have fewer tracks than it expected, if its parent is also a subgrid and therefore has a fixed number of tracks. (A subgrid might likewise have fewer tracks than expected because its parent is hitting the [UA limit on grid tracks](#overlarge-grids).)

- <a id="ref-for-subgridded-axis①⓪"></a>

  <a id="ref-for-parent-grid①①"></a>

  <a id="ref-for-grid-item⑨①"></a>

  <a id="ref-for-subgrid③⑧"></a>

  <a id="subgrid-size-contribution"></a> The [subgrid](#subgrid) itself lays out as an ordinary [grid item](#grid-item) in its [parent grid](#parent-grid), but acts as if it was completely empty for track sizing purposes in the [subgridded](#subgridded-axis) dimension.

- <a id="ref-for-subgridded-axis①①"></a>

  <a id="ref-for-parent-grid①②"></a>

  <a id="ref-for-grid-item⑨②"></a>

  <a id="ref-for-subgrid③⑨"></a>

  <a id="subgrid-item-contribution"></a> The [subgrid](#subgrid)’s own [grid items](#grid-item) participate in the sizing of its [parent grid](#parent-grid) in the [subgridded](#subgridded-axis) dimension(s) and are aligned to it in those dimensions.

  <a id="ref-for-subgrid④⓪"></a>

  <a id="ref-for-scrollbar-gutter"></a>

  <a id="subgrid-margins"></a> In this process, the sum of the [subgrid](#subgrid)’s margin, padding, [scrollbar gutter](https://www.w3.org/TR/css-overflow-3/#scrollbar-gutter), and border at each edge are applied as an extra layer of (potentially negative) margin to the items at those edges. This extra layer of “margin” accumulates through multiple levels of <a id="ref-for-subgrid④①"></a>subgrids.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-edd2713b"></a> For example, if we have a 3×3 grid with the following tracks:
  > ```text
  > #parent-grid { grid-template-columns: 300px auto 300px; }
  > ```
  >
  > If a subgrid covers the last two tracks, its first two columns correspond to the parent grid’s last two columns, and any items positioned into those tracks participate in sizing the parent grid. Specifically, an item positioned in the first track of the subgrid influences the auto-sizing of the parent grid’s middle track.
  >
  > ```text
  > #subgrid { grid-column: 2 / span 2; } /* cover parent’s 2nd and 3rd tracks */
  > #subgrid > :first-child { grid-column: 1; } /* subgrid’s 1st track, parent grid’s 2nd track */
  > ```
  >
  > If the subgrid has margins/borders/padding, the size of those margins/borders/padding also influences sizing. For example, if the subgrid has 100px padding:
  >
  > ```text
  > #subgrid { padding: 100px; }
  > ```
  >
  > <a id="ref-for-grid-item⑨③"></a>
  >
  > <a id="ref-for-subgrid④②"></a>
  >
  > Then a [grid item](#grid-item) in the [subgrid’s](#subgrid) first track acts as if it has an additional 100px of top, left, and bottom margin, influencing the sizing of the parent grid’s tracks and the <a id="ref-for-grid-item⑨④"></a>grid item’s own position.

  <a id="ref-for-subgrid④③"></a>

  <a id="ref-for-gutter④"></a>

  <a id="ref-for-propdef-row-gap①"></a>

  <a id="ref-for-propdef-column-gap①"></a>

  <a id="ref-for-parent-grid①③"></a>

  <a id="ref-for-valdef-row-gap-normal"></a>

  <a id="subgrid-gaps"></a> Meanwhile, half the size of the difference between the [subgrid](#subgrid)’s [gutters](https://www.w3.org/TR/css-align-3/#gutter) ([row-gap](https://www.w3.org/TR/css-align-3/#propdef-row-gap)/[column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap)) and its [parent grid](#parent-grid)’s <a id="ref-for-gutter⑤"></a>gutters is applied as an extra layer of (potentially negative) margin to the items not at those edges. This extra layer of “margin” also accumulates through multiple levels of <a id="ref-for-subgrid④④"></a>subgrids. A value of [normal](https://www.w3.org/TR/css-align-3/#valdef-row-gap-normal) indicates that the <a id="ref-for-subgrid④⑤"></a>subgrid has the same size <a id="ref-for-gutter⑥"></a>gutters as its <a id="ref-for-parent-grid①④"></a>parent grid, i.e. the applied difference is zero.

  <a id="ref-for-subgrid④⑥"></a>

  <a id="ref-for-gutter⑦"></a>

  <a id="ref-for-parent-grid①⑤"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: The end result will be that the parent’s grid tracks will be sized as specified, and that the [subgrid](#subgrid)’s [gutters](https://www.w3.org/TR/css-align-3/#gutter) will visually center-align with the [parent grid](#parent-grid)’s <a id="ref-for-gutter⑧"></a>gutters.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-4627a772"></a>
  > For example, suppose we have a 300px-wide outer grid with 50px gaps and its columns specified as 100px 1fr. A subgrid spanning both tracks would have…
  >
  > <a id="ref-for-valdef-row-gap-normal①"></a>
  >
  > <a id="ref-for-propdef-column-gap②"></a>
  >
  > … if its [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap) were [normal](https://www.w3.org/TR/css-align-3/#valdef-row-gap-normal) (or 50px):
  > - A grid item in its left column sized and laid out (and contributing its size to the parent grid’s sizing calculations) without any special adjustment, thus stretching to 100px wide while remaining aligned to the subgrid’s left edge.
  >
  > - A grid item in its right column sized and laid out (and contributing its size to the parent grid’s sizing calculations) without any special adjustment, thus stretching to 150px wide, while remaining aligned to the subgrid’s right edge.
  >
  > - <a id="ref-for-gutter⑨"></a>
  >
  >   An effective visual [gutter](https://www.w3.org/TR/css-align-3/#gutter) between the items of 50px, exactly matching its parent grid.
  >
  > ![](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/subgrid-gap-normal.png)
  >
  > <a id="ref-for-propdef-column-gap③"></a>
  >
  > … if its [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap) were 0:
  > - A grid item in its left column sized and laid out (and contributing its size to the parent grid’s sizing calculations) as if it had a -25px right margin, thus stretching to 125px wide while remaining aligned to the subgrid’s left edge.
  >
  > - A grid item in its right column sized and laid out (and contributing its size to the parent grid’s sizing calculations) as if it had a -25px left margin, thus stretching to 175px wide, while remaining aligned to the subgrid’s right edge.
  >
  > - <a id="ref-for-propdef-column-gap④"></a>
  >
  >   <a id="ref-for-gutter①⓪"></a>
  >
  >   An effective visual [gutter](https://www.w3.org/TR/css-align-3/#gutter) between the items of zero, as specified by its [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap).
  >
  > ![](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/subgrid-gap-0px.png)
  >
  > <a id="ref-for-propdef-column-gap⑤"></a>
  >
  > … if its [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap) were 25px:
  > - A grid item in its left column sized and laid out (and contributing its size to the parent grid’s sizing calculations) as if it had a -12.5px right margin, thus stretching to 112.5px wide while remaining aligned to the subgrid’s left edge.
  >
  > - A grid item in its right column sized and laid out (and contributing its size to the parent grid’s sizing calculations) as if it had a -12.5px left margin, thus stretching to 162.5px wide, while remaining aligned to the subgrid’s right edge.
  >
  > - <a id="ref-for-propdef-column-gap⑥"></a>
  >
  >   <a id="ref-for-gutter①①"></a>
  >
  >   An effective visual [gutter](https://www.w3.org/TR/css-align-3/#gutter) between the items of 25px, as specified by its [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap).
  >
  > ![](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/subgrid-gap-25px.png)
  >
  > <a id="ref-for-propdef-column-gap⑦"></a>
  >
  > … if its [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap) were 75px:
  > - A grid item in its left column sized and laid out (and contributing its size to the parent grid’s sizing calculations) as if it had a 12.5px right margin, thus stretching to 87.5px wide while remaining aligned to the subgrid’s left edge.
  >
  > - A grid item in its right column sized and laid out (and contributing its size to the parent grid’s sizing calculations) as if it had a 12.5px left margin, thus stretching to 137.5px wide, while remaining aligned to the subgrid’s right edge.
  >
  > - <a id="ref-for-propdef-column-gap⑧"></a>
  >
  >   <a id="ref-for-gutter①②"></a>
  >
  >   An effective visual [gutter](https://www.w3.org/TR/css-align-3/#gutter) between the items of 75px, as specified by its [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap).
  >
  > ![](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/subgrid-gap-75px.png)

- <a id="ref-for-subgrid④⑦"></a>

  <a id="subgrid-edge-placeholders"></a> For each edge of a non-empty [subgrid](#subgrid), to account for the subgrid’s margin/border/padding (and any scrollbar gutter) at that edge, a hypothetical item is contributed to the track sizing algorithm for each span size in the set of items spanning into the occupied track closest to that edge of the subgrid. The hypothetical item’s sizes are taken from the sizes of the largest such item of each span size, and are additionally inflated by the subgrid’s own margin/border/padding/gutter at that edge. Correspondingly, the hypothetical item’s span is taken from that same real item’s span, and inflated by the number of empty tracks between it and the relevant <a id="ref-for-subgrid④⑧"></a>subgrid’s edge(s).

  <a id="ref-for-subgrid④⑨"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: This step can be shortcut if the tracks closest to the [subgrid](#subgrid)’s edges contain real items, which would have already [accounted for the subgrid’s margin/border/padding](#subgrid-item-contribution) as described above.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-97fd1d9b"></a> For example, in the following subgrid layout:
  > ```text
  > 5px auto auto 5px
  > .   aaaaaaaaa .
  > .   bbbb cccc .
  > ```
  >
  > Assuming subgrid items <var>a</var>, <var>b</var>, and <var>c</var> occupying their corresponding grid areas and a subgrid padding of 25px, two hypothetical grid items would be contributed to the track sizing algorithm for the purpose of handling the subgrid’s inline-start padding: one with the size of <var>b</var> plus 25px, spanning the first two columns; and one with the size of <var>a</var> plus 25px, spanning the first three columns.

- <a id="ref-for-propdef-justify-self①"></a>

  <a id="ref-for-propdef-align-self②"></a>

  <a id="ref-for-subgridded-axis①②"></a>

  <a id="ref-for-subgrid⑤⓪"></a>

  <a id="subgrid-box-alignment"></a> The [subgrid](#subgrid) is always stretched in its [subgridded](#subgridded-axis) dimension(s): the [align-self](https://www.w3.org/TR/css-align-3/#propdef-align-self)/[justify-self](https://www.w3.org/TR/css-align-3/#propdef-justify-self) properties on it are ignored, as are any specified width/height constraints.

- <a id="ref-for-subgridded-axis①③"></a>

  <a id="ref-for-propdef-justify-content③"></a>

  <a id="ref-for-propdef-align-content③"></a>

  <a id="ref-for-grid②②"></a>

  <a id="ref-for-subgrid⑤①"></a>

  <a id="subgrid-grid-alignment"></a> Layoutwise, the [subgrid](#subgrid)’s [grid](#grid) is always aligned with the corresponding section of the parent <a id="ref-for-grid②③"></a>grid; the [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content)/[justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content) properties on it are also ignored in the [subgridded](#subgridded-axis) dimension.

- <a id="ref-for-subgrid⑤②"></a>

  <a id="ref-for-propdef-overflow②"></a>

  <a id="subgrid-overflow"></a> The [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) property does apply to [subgrids](#subgrid), so that overflowing contents of the subgrid can be scrolled into view. (Note: the act of scrolling does not affect layout.)

- <a id="ref-for-subgrid⑤③"></a>

  <a id="ref-for-x34"></a>

  <a id="subgrid-relpos"></a> [Relative positioning](https://www.w3.org/TR/CSS2/visuren.html#x34) applies to [subgrids](#subgrid) as normal, and shifts the box and its content together as usual. (Note: Relative positioning takes place after alignment, and does not affect track sizing.)

## <a id="abspos"></a>10.  Absolute Positioning

### <a id="abspos-items"></a>10.1.  With a Grid Container as Containing Block

<a id="ref-for-containing-block③"></a>

<a id="ref-for-grid-container⑦①"></a>

<a id="ref-for-grid-area②⑨"></a>

<a id="ref-for-grid-placement-property①⑦"></a>

<a id="ref-for-propdef-top"></a>

<a id="ref-for-propdef-right"></a>

<a id="ref-for-propdef-bottom"></a>

<a id="ref-for-propdef-left"></a>

If an absolutely positioned element’s [containing block](https://www.w3.org/TR/css-display-4/#containing-block) is generated by a [grid container](#grid-container), the containing block corresponds to the [grid area](#grid-area) determined by its [grid-placement properties](#grid-placement-property). The offset properties ([top](https://www.w3.org/TR/css-position-3/#propdef-top)/[right](https://www.w3.org/TR/css-position-3/#propdef-right)/[bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom)/[left](https://www.w3.org/TR/css-position-3/#propdef-left)) then indicate offsets inwards from the corresponding edges of this <a id="ref-for-containing-block④"></a>containing block, as normal.

<a id="ref-for-grid-container⑦②"></a>

<a id="ref-for-grid-line②①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While absolutely-positioning an element to a [grid container](#grid-container) does allow it to align to that container’s [grid lines](#grid-line), such elements do not take up space or otherwise participate in the layout of the grid.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8b8bc36e"></a>
>
> <a id="ref-for-grid-container⑦③"></a>
>
> <a id="ref-for-grid-container⑦④"></a>
>
> <a id="ref-for-containing-block⑤"></a>
>
> <a id="ref-for-containing-block⑥"></a>
>
> <a id="ref-for-grid-container⑦⑤"></a>
>
> ```text
> .grid {
>   grid: 1fr 1fr 1fr 1fr / 10rem 10rem 10rem 10rem;
>   /* 4 equal-height rows filling the grid container,
>      4 columns of ''10rem'' each */
>   justify-content: center;
>   /* center the grid horizontally within the grid container */
>   position: relative;
>   /* Establish abspos containing block */
> }
> 
> .abspos {
>   grid-row-start: 1;     /* 1st grid row line = top of grid container */
>   grid-row-end: span 2;  /* 3rd grid row line */
>   grid-column-start: 3;  /* 3rd grid col line */
>   grid-column-end: auto; /* right padding edge */
>   /* Containing block covers the top right quadrant of the grid container */
> 
>   position: absolute;
>   top: 70px;
>   bottom: 40px;
>   left: 100px;
>   right: 30px;
> }
> ```
>
> ![](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/abspos-grid.svg)
>
> <a id="ref-for-grid-placement-property①⑧"></a>
>
> <a id="ref-for-flow-relative"></a>
>
> <a id="ref-for-propdef-left①"></a>
>
> <a id="ref-for-propdef-right①"></a>
>
> <a id="ref-for-propdef-top①"></a>
>
> <a id="ref-for-propdef-bottom①"></a>
>
> <a id="ref-for-physical"></a>
>
> <a id="ref-for-propdef-direction"></a>
>
> <a id="ref-for-propdef-writing-mode"></a>
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note: Grids and the [grid-placement properties](#grid-placement-property) are [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative), while the offset properties ([left](https://www.w3.org/TR/css-position-3/#propdef-left), [right](https://www.w3.org/TR/css-position-3/#propdef-right), [top](https://www.w3.org/TR/css-position-3/#propdef-top), and [bottom](https://www.w3.org/TR/css-position-3/#propdef-bottom)) are [physical](https://www.w3.org/TR/css-writing-modes-4/#physical), so if the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) or [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode) properties change, the grid will transform to match, but the offsets won’t.

<a id="ref-for-grid-placement-auto⑦"></a>

<a id="ref-for-grid-placement-property①⑨"></a>

<a id="ref-for-grid-placement①②"></a>

<a id="ref-for-grid-container⑦⑥"></a>

Instead of auto-placement, an [auto](#grid-placement-auto) value for a [grid-placement property](#grid-placement-property) contributes a special line to the [placement](#grid-placement) whose position is that of the corresponding padding edge of the [grid container](#grid-container) (the padding edge of the scrollable area, if the <a id="ref-for-grid-container⑦⑦"></a>grid container overflows). These lines become the first and last lines (0th and -0th) of the <a id="augmented-grid"></a>augmented grid used for positioning absolutely-positioned items.

<a id="ref-for-containing-block⑦"></a>

<a id="ref-for-grid-container⑦⑧"></a>

<a id="ref-for-block-container①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Thus, by default, the absolutely-positioned box’s [containing block](https://www.w3.org/TR/css-display-4/#containing-block) will correspond to the padding edges of the [grid container](#grid-container), as it does for [block containers](https://www.w3.org/TR/css-display-4/#block-container).

<a id="ref-for-grid②④"></a>

<a id="ref-for-in-flow③"></a>

<a id="ref-for-grid-placement-property②⓪"></a>

<a id="ref-for-implicit-grid②①"></a>

<a id="ref-for-grid-placement-auto⑧"></a>

<a id="ref-for-implicit-grid-lines③"></a>

Absolute positioning occurs after layout of the [grid](#grid) and its [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) contents, and does not contribute to the sizing of any grid tracks or affect the size/configuration of the grid in any way. If a [grid-placement property](#grid-placement-property) refers to a non-existent line either by explicitly specifying such a line or by spanning outside of the existing [implicit grid](#implicit-grid), it is instead treated as specifying [auto](#grid-placement-auto) (instead of creating new [implicit grid lines](#implicit-grid-lines)).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Remember that implicit lines are assumed to have all line names, so a referenced line might exist even though it is not explicitly named.

<a id="ref-for-grid-placement①③"></a>

<a id="ref-for-grid-span①⑥"></a>

<a id="ref-for-grid-placement-auto⑨"></a>

<a id="ref-for-grid-placement-property②①"></a>

If the [placement](#grid-placement) only contains a [grid span](#grid-span), replace it with the two [auto](#grid-placement-auto) lines in that axis. (This happens when both [grid-placement properties](#grid-placement-property) in an axis contributed a span originally, and [§ 8.3.1 Grid Placement Conflict Handling](#grid-placement-errors) caused the second span to be ignored.)

### <a id="static-position"></a>10.2.  With a Grid Container as Parent

<a id="ref-for-grid-container⑦⑨"></a>

<a id="ref-for-out-of-flow"></a>

<a id="ref-for-grid-item⑨⑤"></a>

An absolutely-positioned child of a [grid container](#grid-container) is [out-of-flow](https://www.w3.org/TR/css-display-4/#out-of-flow) and not a [grid item](#grid-item), and so does not affect the placement of other items or the sizing of the grid.

<a id="ref-for-grid-container⑧⓪"></a>

<a id="ref-for-grid-area③⓪"></a>

The [static position](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-width) [\[CSS2\]](#biblio-css2) of an absolutely-positioned child of a [grid container](#grid-container) is determined as if it were the sole grid item in a [grid area](#grid-area) whose edges coincide with the content edges of the <a id="ref-for-grid-container⑧①"></a>grid container.

<a id="ref-for-propdef-justify-self②"></a>

<a id="ref-for-propdef-align-self③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Note that this position is affected by the values of [justify-self](https://www.w3.org/TR/css-align-3/#propdef-justify-self) and [align-self](https://www.w3.org/TR/css-align-3/#propdef-align-self) on the child, and that, as in most other layout models, the absolutely-positioned child has no effect on the size of the containing block or layout of its contents.

## <a id="alignment"></a>11.  Alignment and Spacing

<a id="ref-for-grid-container⑧②"></a>

<a id="ref-for-grid-track②⑤"></a>

<a id="ref-for-grid-item⑨⑥"></a>

<a id="ref-for-grid-area③①"></a>

After a [grid container](#grid-container)’s [grid tracks](#grid-track) have been sized, and the dimensions of all [grid items](#grid-item) are finalized, <a id="ref-for-grid-item⑨⑦"></a>grid items can be aligned within their [grid areas](#grid-area).

<a id="ref-for-grid-item⑨⑧"></a>

<a id="ref-for-subgrid⑤④"></a>

<a id="ref-for-baseline-alignment"></a>

<a id="ref-for-parent-grid①⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [grid items](#grid-item) of [subgrids](#subgrid) participate in alignment, including [baseline alignment](https://www.w3.org/TR/css-align-3/#baseline-alignment), together with their [parent grid](#parent-grid)’s items; see [Subgrids (h)](#subgrid-item-contribution).

<a id="ref-for-propdef-margin①"></a>

<a id="ref-for-grid-item⑨⑨"></a>

<a id="ref-for-box-alignment-properties①"></a>

The [margin](https://www.w3.org/TR/CSS2/box.html#propdef-margin) properties can be used to align items in a manner similar to what margins can do in block layout. [Grid items](#grid-item) also respect the [box alignment properties](https://www.w3.org/TR/css-align-3/#box-alignment-properties) from the [CSS Box Alignment Module](https://www.w3.org/TR/css-align/) [\[CSS-ALIGN-3\]](#biblio-css-align-3), which allow easy keyword-based alignment of items in both the rows and columns.

<a id="ref-for-grid-item①⓪⓪"></a>

<a id="ref-for-grid-area③②"></a>

<a id="ref-for-propdef-justify-self③"></a>

<a id="ref-for-propdef-align-self④"></a>

<a id="ref-for-valdef-justify-self-stretch"></a>

By default, [grid items](#grid-item) stretch to fill their [grid area](#grid-area). However, if [justify-self](https://www.w3.org/TR/css-align-3/#propdef-justify-self) or [align-self](https://www.w3.org/TR/css-align-3/#propdef-align-self) compute to a value other than [stretch](https://www.w3.org/TR/css-align-3/#valdef-justify-self-stretch) or margins are auto, <a id="ref-for-grid-item①⓪①"></a>grid items will auto-size to fit their contents.

<a id="ref-for-propdef-row-gap②"></a>

<a id="ref-for-propdef-column-gap⑨"></a>

<a id="ref-for-propdef-gap①"></a>

### <a id="gutters"></a>11.1.  Gutters: the [row-gap](https://www.w3.org/TR/css-align-3/#propdef-row-gap), [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap), and [gap](https://www.w3.org/TR/css-align-3/#propdef-gap) properties

<a id="ref-for-propdef-row-gap③"></a>

<a id="ref-for-propdef-column-gap①⓪"></a>

<a id="ref-for-propdef-gap②"></a>

<a id="ref-for-grid-container⑧③"></a>

<a id="ref-for-gutter①③"></a>

<a id="ref-for-grid-row②"></a>

<a id="ref-for-grid-column②"></a>

The [row-gap](https://www.w3.org/TR/css-align-3/#propdef-row-gap) and [column-gap](https://www.w3.org/TR/css-align-3/#propdef-column-gap) properties (and their [gap](https://www.w3.org/TR/css-align-3/#propdef-gap) shorthand), when specified on a [grid container](#grid-container), define the [gutters](https://www.w3.org/TR/css-align-3/#gutter) between [grid rows](#grid-row) and [grid columns](#grid-column). Their syntax is defined in [CSS Box Alignment 3 § 8 Gaps Between Boxes](https://www.w3.org/TR/css-align-3/#gaps).

<a id="ref-for-grid-line②②"></a>

<a id="ref-for-grid-track②⑥"></a>

<a id="ref-for-gutter①④"></a>

<a id="ref-for-grid-item①⓪②"></a>

The effect of these properties is as though the affected [grid lines](#grid-line) acquired thickness: the [grid track](#grid-track) between two <a id="ref-for-grid-line②③"></a>grid lines is the space between the [gutters](https://www.w3.org/TR/css-align-3/#gutter) that represent them. For the purpose of [track sizing](#algo-track-sizing), each <a id="ref-for-gutter①⑤"></a>gutter is treated as an extra, empty, fixed-size track of the specified size, which is spanned by any [grid items](#grid-item) that span across its corresponding <a id="ref-for-grid-line②④"></a>grid line.

<a id="ref-for-propdef-justify-content④"></a>

<a id="ref-for-propdef-align-content④"></a>

<a id="ref-for-algo-grid-sizing"></a>

<a id="ref-for-gutter①⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Additional spacing may be added between tracks due to [justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content)/[align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content). See [§ 12.1 Grid Sizing Algorithm](#algo-grid-sizing). This space effectively increases the size of the [gutters](https://www.w3.org/TR/css-align-3/#gutter).

<a id="ref-for-grid②⑤"></a>

<a id="ref-for-fragment"></a>

<a id="ref-for-gutter①⑦"></a>

If a [grid](#grid) is [fragmented](https://www.w3.org/TR/css-break-3/#fragment) between tracks, the [gutter](https://www.w3.org/TR/css-align-3/#gutter) spacing between those tracks must be suppressed. <strong data-conversion-semantic="note">Note:</strong> Note that gutters are suppressed even after forced breaks, [unlike margins](https://www.w3.org/TR/css-break-3/#break-margins).

<a id="ref-for-gutter①⑧"></a>

<a id="ref-for-implicit-grid②②"></a>

<a id="ref-for-augmented-grid"></a>

[Gutters](https://www.w3.org/TR/css-align-3/#gutter) only appear <em>between</em> tracks of the [implicit grid](#implicit-grid); there is no gutter before the first track or after the last track. (In particular, there is no <a id="ref-for-gutter①⑨"></a>gutter between the first/last track of the <a id="ref-for-implicit-grid②③"></a>implicit grid and the “auto” lines in the [augmented grid](#augmented-grid).)

<a id="ref-for-collapsed-grid-track②"></a>

<a id="ref-for-implicit-grid②④"></a>

When a [collapsed track](#collapsed-grid-track)’s gutters <a id="collapsed-gutter"></a>collapse, they coincide exactly—​the two gutters overlap so that their start and end edges coincide. If one side of a <a id="ref-for-collapsed-grid-track③"></a>collapsed track does not have a gutter (e.g. if it is the first or last track of the [implicit grid](#implicit-grid)), then collapsing its gutters results in no gutter on either “side” of the <a id="ref-for-collapsed-grid-track④"></a>collapsed track.

### <a id="auto-margins"></a>11.2.  Aligning with auto margins

<a id="ref-for-margin"></a>

<a id="ref-for-grid-item①⓪③"></a>

<a id="ref-for-block-layout"></a>

Auto [margins](https://www.w3.org/TR/css-box-4/#margin) on [grid items](#grid-item) have an effect very similar to auto margins in [block layout](https://www.w3.org/TR/css-display-4/#block-layout):

- <a id="ref-for-grid-track②⑦"></a>

  During calculations of [grid track](#grid-track) sizes, auto margins are treated as 0.

- <a id="ref-for-self-alignment-properties"></a>

  <a id="ref-for-box-alignment-properties②"></a>

  <a id="ref-for-block-layout①"></a>

  <a id="ref-for-inline-axis"></a>

  As defined for the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) of [block layout](https://www.w3.org/TR/css-display-4/#block-layout) (see [CSS2§10.3.3](https://www.w3.org/TR/CSS2/visudet.html#blockwidth)), auto margins in either axis absorb positive free space prior to alignment via the [box alignment properties](https://www.w3.org/TR/css-align-3/#box-alignment-properties), thereby disabling the effects of any [self-alignment properties](https://www.w3.org/TR/css-align-3/#self-alignment-properties) in that axis.

- <a id="ref-for-box-alignment-properties③"></a>

  <a id="ref-for-grid-item①⓪④"></a>

  Overflowing [grid items](#grid-item) resolve their auto margins to zero and overflow as specified by their [box alignment properties](https://www.w3.org/TR/css-align-3/#box-alignment-properties).

<a id="ref-for-propdef-justify-self④"></a>

<a id="ref-for-propdef-justify-items"></a>

### <a id="row-align"></a>11.3.  Inline-axis Alignment: the [justify-self](https://www.w3.org/TR/css-align-3/#propdef-justify-self) and [justify-items](https://www.w3.org/TR/css-align-3/#propdef-justify-items) properties

<a id="ref-for-grid-item①⓪⑤"></a>

<a id="ref-for-propdef-justify-self⑤"></a>

<a id="ref-for-propdef-justify-items①"></a>

<a id="ref-for-grid-container⑧④"></a>

[Grid items](#grid-item) can be aligned in the inline dimension by using the [justify-self](https://www.w3.org/TR/css-align-3/#propdef-justify-self) property on the <a id="ref-for-grid-item①⓪⑥"></a>grid item or [justify-items](https://www.w3.org/TR/css-align-3/#propdef-justify-items) property on the [grid container](#grid-container), as defined in [\[CSS-ALIGN-3\]](#biblio-css-align-3).

<a id="ref-for-grid-item①⓪⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e0479e1b"></a> For example, for an English document, the inline axis is horizontal, and so the justify-\* properties align the [grid items](#grid-item) horizontally.

<a id="ref-for-baseline-alignment①"></a>

<a id="ref-for-grid-item①⓪⑧"></a>

<a id="ref-for-fallback-alignment"></a>

<a id="ref-for-typedef-flex①①"></a>

<a id="ref-for-grid-container⑧⑤"></a>

<a id="ref-for-indefinite①"></a>

If [baseline alignment](https://www.w3.org/TR/css-align-3/#baseline-alignment) is specified on a [grid item](#grid-item) whose size in that axis depends on the size of an intrinsically-sized track (whose size is therefore dependent on both the item’s size and baseline alignment, creating a cyclic dependency), that item does not participate in baseline alignment, and instead uses its [fallback alignment](https://www.w3.org/TR/css-align-3/#fallback-alignment) as if that were originally specified. For this purpose, [\<flex\>](#typedef-flex) track sizes count as “intrinsically-sized” when the [grid container](#grid-container) has an [indefinite](https://www.w3.org/TR/css-sizing-3/#indefinite) size in the relevant axis.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Whether the fallback alignment is used or not does not change over the course of layout: if a cycle exists, it exists.

<a id="ref-for-propdef-align-self⑤"></a>

<a id="ref-for-propdef-align-items"></a>

### <a id="column-align"></a>11.4.  Block-axis Alignment: the [align-self](https://www.w3.org/TR/css-align-3/#propdef-align-self) and [align-items](https://www.w3.org/TR/css-align-3/#propdef-align-items) properties

<a id="ref-for-grid-item①⓪⑨"></a>

<a id="ref-for-propdef-align-self⑥"></a>

<a id="ref-for-propdef-align-items①"></a>

<a id="ref-for-grid-container⑧⑥"></a>

[Grid items](#grid-item) can also be aligned in the block dimension (perpendicular to the inline dimension) by using the [align-self](https://www.w3.org/TR/css-align-3/#propdef-align-self) property on the <a id="ref-for-grid-item①①⓪"></a>grid item or [align-items](https://www.w3.org/TR/css-align-3/#propdef-align-items) property on the [grid container](#grid-container), as defined in [\[CSS-ALIGN-3\]](#biblio-css-align-3).

<a id="ref-for-baseline-alignment②"></a>

<a id="ref-for-grid-item①①①"></a>

<a id="ref-for-fallback-alignment①"></a>

<a id="ref-for-typedef-flex①②"></a>

<a id="ref-for-grid-container⑧⑦"></a>

<a id="ref-for-indefinite②"></a>

If [baseline alignment](https://www.w3.org/TR/css-align-3/#baseline-alignment) is specified on a [grid item](#grid-item) whose size in that axis depends on the size of an intrinsically-sized track (whose size is therefore dependent on both the item’s size and baseline alignment, creating a cyclic dependency), that item does not participate in baseline alignment, and instead uses its [fallback alignment](https://www.w3.org/TR/css-align-3/#fallback-alignment) as if that were originally specified. For this purpose, [\<flex\>](#typedef-flex) track sizes count as “intrinsically-sized” when the [grid container](#grid-container) has an [indefinite](https://www.w3.org/TR/css-sizing-3/#indefinite) size in the relevant axis.

<a id="ref-for-propdef-justify-content⑤"></a>

<a id="ref-for-propdef-align-content⑤"></a>

### <a id="grid-align"></a>11.5.  Aligning the Grid: the [justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content) and [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) properties

<a id="ref-for-grid②⑥"></a>

<a id="ref-for-grid-container⑧⑧"></a>

<a id="ref-for-grid-track②⑧"></a>

<a id="ref-for-propdef-justify-content⑥"></a>

<a id="ref-for-propdef-align-content⑥"></a>

If the [grid](#grid)’s outer edges do not correspond to the [grid container](#grid-container)’s content edges (for example, if no columns are flex-sized), the [grid tracks](#grid-track) are aligned within the content box according to the [justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content) and [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) properties on the <a id="ref-for-grid-container⑧⑨"></a>grid container.

<a id="ref-for-grid-container⑨⓪"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2790843a"></a> For example, the following grid is centered vertically, and aligned to the right edge of its [grid container](#grid-container):
>
> ```text
> .grid {
>   display: grid;
>   grid: 12rem 12rem 12rem 12rem / 10rem 10rem 10rem 10rem;
>   justify-content: end;
>   align-content: center;
>   min-height: 60rem;
> }
> ```
>
> ![](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/align-justify-content.svg)

<a id="ref-for-grid-track②⑨"></a>

<a id="ref-for-explicit-grid③⑤"></a>

<a id="ref-for-implicit-grid②⑤"></a>

<a id="ref-for-grid-line②⑤"></a>

<a id="ref-for-grid-container⑨①"></a>

If there are no [grid tracks](#grid-track) (the [explicit grid](#explicit-grid) is empty, and no tracks were created in the [implicit grid](#implicit-grid)), the sole [grid line](#grid-line) in each axis is aligned with the start edge of the [grid container](#grid-container).

<a id="ref-for-propdef-justify-content⑦"></a>

<a id="ref-for-propdef-align-content⑦"></a>

<a id="ref-for-valdef-align-content-space-around"></a>

<a id="ref-for-valdef-align-content-space-between"></a>

<a id="ref-for-valdef-align-content-space-evenly"></a>

<a id="ref-for-valdef-align-content-stretch"></a>

<a id="ref-for-grid②⑦"></a>

<a id="ref-for-fragment①"></a>

Note that certain values of [justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content) and [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) can cause the tracks to be spaced apart ([space-around](https://www.w3.org/TR/css-align-3/#valdef-align-content-space-around), [space-between](https://www.w3.org/TR/css-align-3/#valdef-align-content-space-between), [space-evenly](https://www.w3.org/TR/css-align-3/#valdef-align-content-space-evenly)) or to be resized ([stretch](https://www.w3.org/TR/css-align-3/#valdef-align-content-stretch)). If the [grid](#grid) is [fragmented](https://www.w3.org/TR/css-break-3/#fragment) between tracks, any such additional spacing between those tracks must be suppressed.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-aafda37f"></a> For example, in the following grid, the spanning item’s grid area is increased to accommodate the extra space assigned to the gutters due to alignment:
>
> ```text
> .wrapper {
>   display: grid;
>   /* 3-row / 4-column grid container */
>   grid: repeat(3, auto) / repeat(4, auto);
>   gap: 10px;
>   align-content: space-around;
>   justify-content: space-between;
> }
> 
> .item1 { grid-column: 1 / 5; }
> .item2 { grid-column: 1 / 3; grid-row: 2 / 4; }
> .item3 { grid-column: 3 / 5; }
> /* last two items auto-place into the last two grid cells */
> ```
>
> ![Grid with 10px gap and an element spanning all columns. The sum of the columns is less than the width of the grid container.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/spanned-gap.svg)
>
> Grid before alignment
>
> ![Same grid with increased gaps absorbing the excess grid container width. The spanning element has grown to accommodate the extra space assigned to the gap it crosses.](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/images/spanned-gap-align.svg)
>
> Grid after alignment
>
> <a id="ref-for-propdef-gap③"></a>
>
> Note that alignment (unlike [gap](https://www.w3.org/TR/css-align-3/#propdef-gap) spacing) happens after the grid tracks are sized, so if the track sizes are determined by the contents of the spanned item, it will gain excess space in the alignment stage to accommodate the alignment spacing.

### <a id="grid-baselines"></a>11.6.  Grid Container Baselines

<a id="ref-for-grid-container⑨②"></a>

The first (last) baselines of a [grid container](#grid-container) are determined as follows:

1.  <a id="ref-for-grid-item①①②"></a>

    <a id="ref-for-grid-container⑨③"></a>

    Find the first (last) row of the [grid container](#grid-container) containing at least one [grid item](#grid-item).

    <a id="ref-for-grid-item①①③"></a>

    <a id="ref-for-baseline-alignment③"></a>

    <a id="ref-for-baseline-set"></a>

    <a id="ref-for-generate-baselines"></a>

    <a id="ref-for-alignment-baseline"></a>

    If any of the [grid items](#grid-item) intersecting this row participate in [baseline alignment](https://www.w3.org/TR/css-align-3/#baseline-alignment) in that row, the grid container’s [baseline set](https://www.w3.org/TR/css-align-3/#baseline-set) is [generated](https://www.w3.org/TR/css-align-3/#generate-baselines) from the shared [alignment baseline](https://www.w3.org/TR/css-align-3/#alignment-baseline) of those <a id="ref-for-grid-item①①④"></a>grid items.

    <a id="ref-for-generate-baselines①"></a>

    <a id="ref-for-alignment-baseline①"></a>

    <a id="ref-for-grid-item①①⑤"></a>

    <a id="ref-for-grid-order"></a>

    <a id="ref-for-writing-mode③"></a>

    <a id="ref-for-grid-container⑨④"></a>

    <a id="ref-for-synthesize-baseline"></a>

    Otherwise, the grid container’s first (last) baseline set is [generated](https://www.w3.org/TR/css-align-3/#generate-baselines) from the [alignment baseline](https://www.w3.org/TR/css-align-3/#alignment-baseline) of the first (last) [grid item](#grid-item) in row-major [grid order](#grid-order) (according to the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the [grid container](#grid-container)). If the <a id="ref-for-grid-item①①⑥"></a>grid item has no <a id="ref-for-alignment-baseline②"></a>alignment baseline in the grid’s inline axis, then one is first [synthesized](https://www.w3.org/TR/css-align-3/#synthesize-baseline) from its border edges.

2.  <a id="ref-for-shared-alignment-context"></a>

    <a id="ref-for-synthesize-baseline①"></a>

    <a id="ref-for-baseline-set①"></a>

    <a id="ref-for-grid-item①①⑦"></a>

    <a id="ref-for-grid-container⑨⑤"></a>

    If the [grid container](#grid-container) does not contain any [grid items](#grid-item), the grid container has no first (last) [baseline set](https://www.w3.org/TR/css-align-3/#baseline-set), and one is [synthesized](https://www.w3.org/TR/css-align-3/#synthesize-baseline) if needed according to the rules of its [alignment context](https://www.w3.org/TR/css-align-3/#shared-alignment-context). Exit from this algorithm.

<a id="ref-for-grid-item①①⑧"></a>

<a id="ref-for-grid-cell⑥"></a>

<a id="ref-for-order-modified-document-order⑥"></a>

<a id="grid-order"></a>Grid-modified document order (grid order) is the order in which [grid items](#grid-item) are encountered when traversing the grid’s [grid cells](#grid-cell). If two items are encountered at the same time, they are taken in [order-modified document order](https://www.w3.org/TR/css-flexbox-1/#order-modified-document-order).

<a id="ref-for-propdef-overflow③"></a>

When calculating the baseline according to the above rules, if the box contributing a baseline has an [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) value that allows scrolling, the box must be treated as being in its initial scroll position for the purpose of determining its baseline.

When [determining the baseline of a table cell](https://www.w3.org/TR/CSS2/tables.html#height-layout), a grid container provides a baseline just as a line box or table-row does. [\[CSS2\]](#biblio-css2)

See [CSS Writing Modes 3 § 4.1 Introduction to Baselines](https://www.w3.org/TR/css-writing-modes-3/#intro-baselines) and [CSS Box Alignment 3 § 9 Baseline Alignment Details](https://www.w3.org/TR/css-align-3/#baseline-rules) for more information on baselines.

## <a id="layout-algorithm"></a>12.  Grid Layout Algorithm<a id="algo-overview"></a>

<a id="ref-for-grid-container⑨⑥"></a>

<a id="ref-for-grid-track③⓪"></a>

<a id="ref-for-grid-item①①⑨"></a>

<a id="ref-for-grid-item-placement-algorithm⑨"></a>

<a id="ref-for-grid-area③③"></a>

This section defines the <b>grid layout algorithm</b>, which sizes the [grid container](#grid-container), sizes and positions all the [grid tracks](#grid-track), and lays out the [grid items](#grid-item) which have been [placed](#grid-item-placement-algorithm) into its [grid areas](#grid-area).

1.  <a id="ref-for-grid-item-placement-algorithm①⓪"></a>

    <a id="ref-for-grid-item①②⓪"></a>

    <a id="ref-for-subgrid⑤⑤"></a>

    <a id="ref-for-grid②⑧"></a>

    Run the [Grid Item Placement Algorithm](#grid-item-placement-algorithm) to resolve the placement of all [grid items](#grid-item) (including [subgrids](#subgrid) and their sub-items) in the [grid](#grid).

2.  <a id="ref-for-grid-container⑨⑦"></a>

    Find the size of the [grid container](#grid-container), per [§ 5.2 Sizing Grid Containers](#intrinsic-sizes).

    <a id="ref-for-percentage-value③"></a>

    <a id="ref-for-valdef-grid-template-columns-auto⑦"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: During this phase, cyclic [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s in track sizes are treated as [auto](#valdef-grid-template-columns-auto).

3.  <a id="ref-for-grid-container⑨⑧"></a>

    <a id="ref-for-algo-grid-sizing②"></a>

    <a id="ref-for-grid②⑨"></a>

    Given the resulting [grid container](#grid-container) size, run the [Grid Sizing Algorithm](#algo-grid-sizing) to size the [grid](#grid).

    <a id="ref-for-percentage-value④"></a>

    <a id="ref-for-grid-container⑨⑨"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: During this phase, [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s in track sizes are resolved against the [grid container](#grid-container) size.

4.  <a id="ref-for-grid-item①②①"></a>

    <a id="ref-for-grid-area③④"></a>

    <a id="ref-for-definite①④"></a>

    Lay out the [grid items](#grid-item) into their respective containing blocks. Each [grid area’s](#grid-area) width and height are considered [definite](https://www.w3.org/TR/css-sizing-3/#definite) for this purpose.

    <a id="ref-for-stretch-fit①"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Since formulas calculated using only definite sizes, such as the [stretch fit](https://www.w3.org/TR/css-sizing-3/#stretch-fit) formula, are also definite, the size of a grid item which is stretched is also considered definite.

### <a id="algo-grid-sizing"></a>12.1.  Grid Sizing Algorithm

<a id="ref-for-grid-track③①"></a>

This section defines the <b>grid sizing algorithm</b>, which determines the size of all [grid tracks](#grid-track) and, by extension, the entire grid.

<a id="ref-for-min-track-sizing-function④"></a>

<a id="ref-for-max-track-sizing-function⑥"></a>

<a id="ref-for-grid-template-rows-track-sizing-function⑤"></a>

Each track has specified [minimum](#min-track-sizing-function) and [maximum](#max-track-sizing-function) [sizing functions](#grid-template-rows-track-sizing-function) (which may be the same). Each <a id="ref-for-grid-template-rows-track-sizing-function⑥"></a>sizing function is either:

- <a id="ref-for-percentage-value⑤"></a>

  <a id="ref-for-length-value②"></a>

  A <a id="fixed-sizing-function"></a>fixed sizing function ([\<length\>](https://www.w3.org/TR/css-values-4/#length-value) or resolvable [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)).

- <a id="ref-for-funcdef-grid-template-columns-fit-content②"></a>

  <a id="ref-for-valdef-grid-template-columns-auto⑧"></a>

  <a id="ref-for-valdef-grid-template-columns-max-content④"></a>

  <a id="ref-for-valdef-grid-template-columns-min-content④"></a>

  An <a id="intrinsic-sizing-function"></a>intrinsic sizing function ([min-content](#valdef-grid-template-columns-min-content), [max-content](#valdef-grid-template-columns-max-content), [auto](#valdef-grid-template-columns-auto), [fit-content()](#funcdef-grid-template-columns-fit-content)).

- <a id="ref-for-typedef-flex①③"></a>

  A <a id="flexible-sizing-function"></a>flexible sizing function ([\<flex\>](#typedef-flex)).

<a id="ref-for-algo-grid-sizing①"></a>

The [grid sizing algorithm](#algo-grid-sizing) defines how to resolve these sizing constraints into used track sizes.

1.  <a id="ref-for-grid-column③"></a>

    <a id="ref-for-track-sizing-algorithm①"></a>

    First, the [track sizing algorithm](#track-sizing-algorithm) is used to resolve the sizes of the [grid columns](#grid-column).

    <a id="ref-for-grid-item①②②"></a>

    <a id="ref-for-grid-container①⓪⓪"></a>

    <a id="ref-for-inline-axis①"></a>

    In this process, any [grid item](#grid-item) which is subgridded in the [grid container](#grid-container)’s [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) is treated as empty and its <a id="ref-for-grid-item①②③"></a>grid items (the grandchildren) are treated as direct children of the <a id="ref-for-grid-container①⓪①"></a>grid container (their grandparent). This introspection is recursive.

    <a id="ref-for-block-axis"></a>

    <a id="ref-for-grid-container①⓪②"></a>

    <a id="ref-for-inline-axis②"></a>

    Items which are subgridded only in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis), and whose [grid container](#grid-container) size in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) depends on the size of its contents are also introspected: since the size of the item in this dimension can be dependent on the sizing of its subgridded tracks in the other, the size contribution of any such item to this grid’s column sizing (see [Resolve Intrinsic Track Sizes](https://www.w3.org/TR/css-grid-1/#algo-content)) is taken under the provision of having determined its track sizing only up to the same point in the Grid Sizing Algorithm as this itself. E.g. for the first pass through this step, the item will have its tracks sized only through this first step; if a second pass of this step is triggered then the item will have completed a first pass through steps 1-3 as well as the second pass of this step prior to returning its size for consideration in this grid’s column sizing. Again, this introspection is recursive.

    <a id="ref-for-grid-item①②④"></a>

    <a id="ref-for-available"></a>

    <a id="ref-for-block-axis①"></a>

    <a id="ref-for-definite①⑤"></a>

    <a id="ref-for-max-track-sizing-function⑦"></a>

    <a id="ref-for-grid-container①⓪③"></a>

    <a id="ref-for-propdef-align-content⑧"></a>

    If calculating the layout of a [grid item](#grid-item) in this step depends on the [available space](https://www.w3.org/TR/css-sizing-3/#available) in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis), assume the <a id="ref-for-available①"></a>available space that it would have if any row with a [definite](https://www.w3.org/TR/css-sizing-3/#definite) [max track sizing function](#max-track-sizing-function) had that size and all other rows were infinite. If both the [grid container](#grid-container) and all tracks have <a id="ref-for-definite①⑥"></a>definite sizes, also apply [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) to find the final effective size of any gaps spanned by such items; otherwise ignore the effects of track alignment in this estimation.

2.  <a id="ref-for-grid-row③"></a>

    <a id="ref-for-track-sizing-algorithm②"></a>

    Next, the [track sizing algorithm](#track-sizing-algorithm) resolves the sizes of the [grid rows](#grid-row).

    <a id="ref-for-grid-item①②⑤"></a>

    <a id="ref-for-grid-container①⓪④"></a>

    <a id="ref-for-block-axis②"></a>

    In this process, any [grid item](#grid-item) which is subgridded in the [grid container](#grid-container)’s [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) is treated as empty and its <a id="ref-for-grid-item①②⑥"></a>grid items (the grandchildren) are treated as direct children of the <a id="ref-for-grid-container①⓪⑤"></a>grid container (their grandparent). This introspection is recursive.

    <a id="ref-for-inline-axis③"></a>

    <a id="ref-for-grid-container①⓪⑥"></a>

    <a id="ref-for-block-axis③"></a>

    As with sizing columns, items which are subgridded only in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis), and whose [grid container](#grid-container) size in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) depends on the size of its contents are also introspected. (As with sizing columns, the size contribution to this grid’s row sizing is taken under the provision of having determined its track sizing only up to this corresponding point in the algorithm; and again, this introspection is recursive.)

    <a id="ref-for-inline-axis④"></a>

    <a id="ref-for-available②"></a>

    <a id="ref-for-block-axis④"></a>

    <a id="ref-for-grid-column④"></a>

    <a id="ref-for-grid-container①⓪⑦"></a>

    <a id="ref-for-inline-size④"></a>

    <a id="ref-for-definite①⑦"></a>

    <a id="ref-for-propdef-justify-content⑧"></a>

    To find the [inline-axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) [available space](https://www.w3.org/TR/css-sizing-3/#available) for any items whose [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) size contributions require it, use the [grid column](#grid-column) sizes calculated in the previous step. If the [grid container](#grid-container)’s [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) is [definite](https://www.w3.org/TR/css-sizing-3/#definite), also apply [justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content) to account for the effective column gap sizes.

3.  <a id="ref-for-max-content-contribution②"></a>

    <a id="ref-for-grid-column⑤"></a>

    <a id="ref-for-min-content-contribution①"></a>

    Then, if the [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) of any grid item has changed based on the row sizes and alignment calculated in step 2, re-resolve the sizes of the [grid columns](#grid-column) with the new <a id="ref-for-min-content-contribution②"></a>min-content and [max-content contributions](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) (once only).

    <a id="ref-for-block-axis⑤"></a>

    <a id="ref-for-available③"></a>

    <a id="ref-for-inline-axis⑤"></a>

    <a id="ref-for-grid-row④"></a>

    <a id="ref-for-grid-container①⓪⑧"></a>

    <a id="ref-for-block-size②"></a>

    <a id="ref-for-definite①⑧"></a>

    <a id="ref-for-propdef-align-content⑨"></a>

    To find the [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) [available space](https://www.w3.org/TR/css-sizing-3/#available) for any items whose [inline-axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) size contributions require it, use the [grid row](#grid-row) sizes calculated in the previous step. If the [grid container](#grid-container)’s [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) is [definite](https://www.w3.org/TR/css-sizing-3/#definite), also apply [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) to account for the effective row gap sizes.

    <a id="ref-for-inline-size⑤"></a>

    <a id="ref-for-grid-item①②⑦"></a>

    <a id="ref-for-block-size③"></a>

    <a id="ref-for-grid-area③⑤"></a>

    <a id="ref-for-flex-container②"></a>

    <a id="ref-for-propdef-flex-flow"></a>

    <a id="ref-for-establish-an-orthogonal-flow"></a>

    <a id="ref-for-propdef-writing-mode①"></a>

    <a id="ref-for-multi-column-container"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > This repetition is necessary for cases where the [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) of a [grid item](#grid-item) depends on the [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) of its [grid area](#grid-area). Examples include wrapped column [flex containers](https://www.w3.org/TR/css-flexbox-1/#flex-container) ([flex-flow: column wrap](https://www.w3.org/TR/css-flexbox-1/#propdef-flex-flow)), [orthogonal flows](https://www.w3.org/TR/css-writing-modes-4/#establish-an-orthogonal-flow) ([writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)), [multi-column containers](https://www.w3.org/TR/css-multicol-2/#multi-column-container), and items with an aspect-ratio (or with a child with an aspect ratio) whose size depends on the size of the row.

4.  <a id="ref-for-max-content-contribution③"></a>

    <a id="ref-for-grid-row⑤"></a>

    <a id="ref-for-min-content-contribution③"></a>

    Next, if the [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution) of any grid item has changed based on the column sizes and alignment calculated in step 3, re-resolve the sizes of the [grid rows](#grid-row) with the new <a id="ref-for-min-content-contribution④"></a>min-content and [max-content contributions](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) (once only).

    <a id="ref-for-inline-axis⑥"></a>

    <a id="ref-for-available④"></a>

    <a id="ref-for-block-axis⑥"></a>

    <a id="ref-for-grid-column⑥"></a>

    <a id="ref-for-grid-container①⓪⑨"></a>

    <a id="ref-for-inline-size⑥"></a>

    <a id="ref-for-definite①⑨"></a>

    <a id="ref-for-propdef-justify-content⑨"></a>

    To find the [inline-axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) [available space](https://www.w3.org/TR/css-sizing-3/#available) for any items whose [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) size contributions require it, use the [grid column](#grid-column) sizes calculated in the previous step. If the [grid container](#grid-container)’s [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) is [definite](https://www.w3.org/TR/css-sizing-3/#definite), also apply [justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content) to account for the effective column gap sizes.

5.  <a id="ref-for-propdef-justify-content①⓪"></a>

    <a id="ref-for-propdef-align-content①⓪"></a>

    <a id="ref-for-grid-container①①⓪"></a>

    Finally, align the tracks within the [grid container](#grid-container) according to the [align-content](https://www.w3.org/TR/css-align-3/#propdef-align-content) and [justify-content](https://www.w3.org/TR/css-align-3/#propdef-justify-content) properties.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This can introduce extra space between tracks, potentially enlarging the grid area of any grid items spanning the gaps beyond the space allotted to during track sizing.

<a id="ref-for-subgridded-axis①④"></a>

<a id="ref-for-parent-grid①⑦"></a>

<a id="ref-for-subgrid⑤⑥"></a>

<a id="ref-for-establish-an-orthogonal-flow①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Track sizing in a [subgridded](#subgridded-axis) dimension treats each item in a given track in that axis as members of the [parent grid](#parent-grid). This interlacing requires that grid sizing drill down per axis into [subgrids](#subgrid), rather than completing both axes as it recurses. Note this means that a <a id="ref-for-subgrid⑤⑦"></a>subgrid establishing an [orthogonal flow](https://www.w3.org/TR/css-writing-modes-4/#establish-an-orthogonal-flow) would have the order of its track sizing inverted compared to a nested grid.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-da309425"></a> The following example illustrates how per-axis subgrids are sized:
>
> Suppose we have a parent grid container <var>A</var> which contains an item <var>B</var> that has subgridded columns and contains a grandchild <var>B</var> that has subgridded rows and grandchild <var>D</var> that is simply a nested grid.
>
> ```text
> <grid-A>
>   <grid-B subgrid=columns>
>     <grid-C subgrid=rows></grid-C>
>     <grid-D></grid-D>
>   </grid-B>
> <grid-A>
> ```
>
> When <var>A</var> sizes its columns it treats <var>B</var>’s items as slotted into to <var>A</var>’s corresponding columns, but when <var>A</var> sizes its rows it treats <var>B</var> as a single item (a grid container with its own rows and some items including items <var>C</var> and <var>D</var>). Similarly when <var>B</var> sizes its rows, it treats <var>C</var>’s items as slotted into <var>B</var>’s rows, but when <var>B</var> sizes its columns, it treats <var>C</var> as a single item, just as it does with <var>D</var>. There is no relationship between <var>C</var>’s rows and <var>A</var>’s rows, because the rows in <var>B</var> are nested, not subgridded.
>
> At a high level, the grid algorithm is:
>
> 1.  Size the columns
> 2.  Size the rows
> 3.  Adjust the columns (if needed based on final row sizes)
>
> The grid sizing algorithm in this example would thus look like this:
>
> 1.  <strong>Resolve sizes of <var>A</var>’s grid columns,
				using the sizes of <var>A</var>’s grid items,
				treating <var>B</var> as empty
				but treating its children
				(including <var>C</var> and <var>D</var>)
				as items in grid <var>A</var>.</strong>
>
>     The grid algorithm simply recurses into <var>D</var>. For <var>C</var>, it’s more complicated:
>
>     1.  Size <var>C</var>’s columns.
>     2.  Size <var>C</var>’s rows by sizing <var>B</var>’s rows.
>     3.  Adjust <var>C</var>’s columns.
>     4.  Return <var>C</var>’s final column sizes.
>
>     A correct size for <var>B</var>’s rows requires <var>C</var>’s final column sizes, because the row size depends on the column size, and thus <var>B</var>’s rows could very well depend on <var>C</var>’s final column sizes. To break this cyclic dependency, we need to split the algorithm to depend on the initial approximation of <var>C</var>’s final column sizes, and do the adjustment pass later. So for <var>C</var>, we need to recurse into column sizing only, and pass that initial size up to <var>A</var> for its initial column sizing.
>
>     When we size <var>B</var>’s rows later on, we will size <var>C</var>’s rows (which are subgridded), and finish up <var>C</var>’s sizing by finalizing its columns. If this resulted in a change, we have the opportunity to trigger an adjustment pass for <var>A</var>’s columns during its adjustment pass.
>
> 2.  <strong>Next, resolve sizes of <var>A</var>’s rows,
				using the sizes of <var>A</var>’s grid items,
				treating <var>B</var> as a single item.</strong>
>
>     Since <var>B</var>, as a subgrid, has its sizing is split out into the multiple passes, the grid algorithm issues only a row-sizing recursion into <var>B</var>: Size <var>B</var>’s rows, treating D as a single item, requesting its final size, and treating <var>C</var> as an empty item and hoisting its children as items into grid <var>B</var>.
>
>     <var>B</var> returns its final row size, which factors into <var>A</var>’s row sizing pass.
>
> 3.  <strong>Last, finalize <var>A</var>’s column sizes.</strong> If <var>C</var>’s final size changes as a result of the row-sizing pass through <var>B</var>, this should trigger a resizing of <var>B</var>’s columns, which should trigger a resizing pass on <var>A</var>’s column.

### <a id="algo-terms"></a>12.2.  Track Sizing Terminology

<a id="min-track-sizing-function"></a>min track sizing function  
<a id="ref-for-valdef-grid-template-columns-auto⑨"></a>

<a id="ref-for-funcdef-grid-template-columns-fit-content③"></a>

<a id="ref-for-typedef-flex①④"></a>

<a id="ref-for-funcdef-grid-template-columns-minmax④"></a>

If the track was sized with a [minmax()](#funcdef-grid-template-columns-minmax) function, this is the first argument to that function. If the track was sized with a [\<flex\>](#typedef-flex) value or [fit-content()](#funcdef-grid-template-columns-fit-content) function, [auto](#valdef-grid-template-columns-auto). Otherwise, the track’s sizing function.

<a id="max-track-sizing-function"></a>max track sizing function  
<a id="ref-for-valdef-grid-template-columns-max-content⑤"></a>

<a id="ref-for-funcdef-grid-template-columns-fit-content④"></a>

<a id="ref-for-valdef-grid-template-columns-auto①⓪"></a>

<a id="ref-for-funcdef-grid-template-columns-minmax⑤"></a>

If the track was sized with a [minmax()](#funcdef-grid-template-columns-minmax) function, this is the second argument to that function. Otherwise, the track’s sizing function. In all cases, treat [auto](#valdef-grid-template-columns-auto) and [fit-content()](#funcdef-grid-template-columns-fit-content) as [max-content](#valdef-grid-template-columns-max-content), except where specified otherwise for <a id="ref-for-funcdef-grid-template-columns-fit-content⑤"></a>fit-content().

<a id="available-grid-space"></a>available grid space  
<a id="ref-for-available-grid-space"></a>

Independently in each dimension, the [available grid space](#available-grid-space) is:

- <a id="ref-for-grid-container①①①"></a>

  If the [grid container’s](#grid-container) size is definite, then use the size of its content box.

- <a id="ref-for-grid-container①①②"></a>

  <a id="ref-for-min-content-constraint①"></a>

  <a id="ref-for-max-content-constraint①"></a>

  <a id="ref-for-available-grid-space①"></a>

  If the [grid container](#grid-container) is being sized under a [min-content constraint](https://www.w3.org/TR/css-sizing-3/#min-content-constraint) or [max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint) then the [available grid space](#available-grid-space) is that constraint (and is indefinite).

<a id="ref-for-grid-placement-auto①⓪"></a>

<a id="ref-for-valdef-grid-template-columns-max-content⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [auto](#grid-placement-auto) sizes that indicate content-based sizing (e.g. the height of a block-level box in horizontal writing modes) are equivalent to [max-content](#valdef-grid-template-columns-max-content).

<a id="ref-for-available-grid-space②"></a>

<a id="ref-for-grid-container①①③"></a>

In all cases, clamp the [available grid space](#available-grid-space) according to the [grid container’s](#grid-container) min/max-width/height properties, if they are definite.

<a id="free-space"></a>free space  
<a id="ref-for-free-space②"></a>

<a id="ref-for-indefinite③"></a>

<a id="ref-for-base-size"></a>

<a id="ref-for-available-grid-space③"></a>

Equal to the [available grid space](#available-grid-space) minus the sum of the [base sizes](#base-size) of all the grid tracks (including gutters), floored at zero. If <a id="ref-for-available-grid-space④"></a>available grid space is [indefinite](https://www.w3.org/TR/css-sizing-3/#indefinite), the [free space](#free-space) is <a id="ref-for-indefinite④"></a>indefinite as well.

<a id="span-count"></a>span count  
<a id="ref-for-grid-item①②⑧"></a>

<a id="ref-for-grid-track③②"></a>

The number of [grid tracks](#grid-track) crossed by a [grid item](#grid-item) in the applicable dimension.

<a id="ref-for-track-sizing-algorithm③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Remember that [gutters](#gutters) are treated as fixed-size tracks—​tracks with their min and max sizing functions both set to the gutter’s used size—​for the purpose of the grid sizing algorithm. Their widths need to be incorporated into the [track sizing algorithm](#track-sizing-algorithm)’s calculations accordingly.

### <a id="algo-track-sizing"></a>12.3.  Track Sizing Algorithm

<a id="ref-for-min-track-sizing-function⑤"></a>

<a id="ref-for-max-track-sizing-function⑧"></a>

<a id="ref-for-length-value③"></a>

<a id="ref-for-base-size①"></a>

The remainder of this section is the <a id="track-sizing-algorithm"></a>track sizing algorithm, which calculates from the [min](#min-track-sizing-function) and [max track sizing functions](#max-track-sizing-function) the used track size. Each track has a <a id="base-size"></a>base size, a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) which grows throughout the algorithm and which will eventually be the track’s final size, and a <a id="growth-limit"></a>growth limit, a <a id="ref-for-length-value④"></a>\<length\> which provides a desired maximum size for the [base size](#base-size). There are 5 steps:

1.  [Initialize Track Sizes](#algo-init)
2.  [Resolve Intrinsic Track Sizes](#algo-content)
3.  [Maximize Tracks](#algo-grow-tracks)
4.  [Expand Flexible Tracks](#algo-flex-tracks)
5.  [Expand Stretched auto Tracks](#algo-stretch)

### <a id="algo-init"></a>12.4.  Initialize Track Sizes

<a id="ref-for-min-track-sizing-function⑥"></a>

<strong>Initialize each track’s base size and growth limit.</strong> For each track, if the track’s [min track sizing function](#min-track-sizing-function) is:

<a id="ref-for-fixed-sizing-function③"></a>

A [fixed sizing function](#fixed-sizing-function)

<a id="ref-for-base-size②"></a>

Resolve to an absolute length and use that size as the track’s initial [base size](#base-size).

<a id="ref-for-indefinite⑤"></a>

<a id="ref-for-valdef-grid-template-columns-auto①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Indefinite](https://www.w3.org/TR/css-sizing-3/#indefinite) lengths cannot occur, as they’re treated as [auto](#valdef-grid-template-columns-auto).

<a id="ref-for-intrinsic-sizing-function②"></a>

An [intrinsic sizing function](#intrinsic-sizing-function)

<a id="ref-for-base-size③"></a>

Use an initial [base size](#base-size) of zero.

<a id="ref-for-max-track-sizing-function⑨"></a>

For each track, if the track’s [max track sizing function](#max-track-sizing-function) is:

<a id="ref-for-fixed-sizing-function④"></a>

A [fixed sizing function](#fixed-sizing-function)

<a id="ref-for-growth-limit"></a>

Resolve to an absolute length and use that size as the track’s initial [growth limit](#growth-limit).

<a id="ref-for-intrinsic-sizing-function③"></a>

An [intrinsic sizing function](#intrinsic-sizing-function)

<a id="ref-for-flexible-sizing-function①"></a>

A [flexible sizing function](#flexible-sizing-function)

<a id="ref-for-growth-limit①"></a>

Use an initial [growth limit](#growth-limit) of infinity.

<a id="ref-for-growth-limit②"></a>

<a id="ref-for-base-size④"></a>

In all cases, if the [growth limit](#growth-limit) is less than the [base size](#base-size), increase the <a id="ref-for-growth-limit③"></a>growth limit to match the <a id="ref-for-base-size⑤"></a>base size.

<a id="ref-for-gutter②⓪"></a>

<a id="ref-for-track-sizing-algorithm④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Gutters](https://www.w3.org/TR/css-align-3/#gutter) are treated as empty fixed-size tracks for the purpose of the [track sizing algorithm](#track-sizing-algorithm).

### <a id="algo-content"></a>12.5.  Resolve Intrinsic Track Sizes

<a id="ref-for-grid-template-rows-track-sizing-function⑦"></a>

This step resolves intrinsic track [sizing functions](#grid-template-rows-track-sizing-function) to absolute lengths. First it resolves those sizes based on items that are contained wholly within a single track. Then it gradually adds in the space requirements of items that span multiple tracks, evenly distributing the extra space across those tracks insofar as possible.

<a id="ref-for-base-size⑥"></a>

<a id="ref-for-growth-limit④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: When this step is complete, all intrinsic [base sizes](#base-size) and [growth limits](#growth-limit) will have been resolved to absolute lengths.

<a id="ref-for-funcdef-grid-template-columns-fit-content⑥"></a>

<a id="ref-for-valdef-grid-template-columns-auto①②"></a>

<a id="ref-for-max-track-sizing-function①⓪"></a>

<a id="ref-for-valdef-grid-template-columns-max-content⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Remember that](#algo-terms) [fit-content()](#funcdef-grid-template-columns-fit-content) and [auto](#valdef-grid-template-columns-auto) [max track sizing functions](#max-track-sizing-function) are treated the same as [max-content](#valdef-grid-template-columns-max-content) except where explicitly specified otherwise.

1.  <a id="ref-for-baseline-sharing-group"></a>

    <a id="algo-baseline-shims"></a> <strong>Shim baseline-aligned items
			so their intrinsic size contributions reflect their baseline alignment.</strong> For the items in each [baseline-sharing group](https://www.w3.org/TR/css-align-3/#baseline-sharing-group), add a “shim” (effectively, additional margin) on the start/end side (for first/last-baseline alignment) of each item so that, when start/end-aligned together their [baselines align as specified](https://www.w3.org/TR/css-align-3/#baseline-values).

    Consider these “shims” as part of the items’ intrinsic size contribution for the purpose of track sizing, below. If an item uses multiple intrinsic size contributions, it can have different shims for each one.

    <a id="ref-for-grid-container①①④"></a>

    <a id="ref-for-indefinite⑥"></a>

    > <strong data-conversion-semantic="example">Example</strong>
    >
    > <a id="example-497433d9"></a> For example, when the [grid container](#grid-container) has an [indefinite](https://www.w3.org/TR/css-sizing-3/#indefinite) size, it is first laid out under min/max-content constraints to find the size, then laid out "for real" with that size (which can affect things like percentage tracks). The "shims" added for each phase are independent, and only affect the layout during that phase.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Note that both [baseline self-aligned](https://www.w3.org/TR/css-align-3/#baseline-align-self) and [baseline content-aligned](https://www.w3.org/TR/css-align-3/#baseline-align-content) items are considered in this step.

    <a id="ref-for-grid-item①②⑨"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: Since [grid items](#grid-item) whose own size depends on the size of an intrinsically-sized track [do not participate in baseline alignment](#row-align), they are not shimmed.

2.  <a id="ref-for-flexible-sizing-function②"></a>

    <a id="ref-for-grid-template-rows-track-sizing-function⑧"></a>

    <a id="algo-single-span-items"></a> <strong>Size tracks to fit non-spanning items:</strong> For each track with an intrinsic [track sizing function](#grid-template-rows-track-sizing-function) and not a [flexible sizing function](#flexible-sizing-function), consider the items in it with a span of 1:

    For min-content minimums:  
    <a id="ref-for-min-content-contribution⑤"></a>

    <a id="ref-for-base-size⑦"></a>

    <a id="ref-for-min-track-sizing-function⑦"></a>

    <a id="ref-for-valdef-grid-template-columns-min-content⑤"></a>

    If the track has a [min-content](#valdef-grid-template-columns-min-content) [min track sizing function](#min-track-sizing-function), set its [base size](#base-size) to the maximum of the items’ [min-content contributions](https://www.w3.org/TR/css-sizing-3/#min-content-contribution), floored at zero.

    For max-content minimums:  
    <a id="ref-for-max-content-contribution④"></a>

    <a id="ref-for-base-size⑧"></a>

    <a id="ref-for-min-track-sizing-function⑧"></a>

    <a id="ref-for-valdef-grid-template-columns-max-content⑧"></a>

    If the track has a [max-content](#valdef-grid-template-columns-max-content) [min track sizing function](#min-track-sizing-function), set its [base size](#base-size) to the maximum of the items’ [max-content contributions](https://www.w3.org/TR/css-sizing-3/#max-content-contribution), floored at zero.

    For auto minimums:  
    <a id="ref-for-minimum-contribution"></a>

    <a id="ref-for-fixed-sizing-function⑤"></a>

    <a id="ref-for-funcdef-grid-template-columns-fit-content⑦"></a>

    <a id="ref-for-max-track-sizing-function①①"></a>

    <a id="ref-for-max-content-contribution⑤"></a>

    <a id="ref-for-min-content-contribution⑥"></a>

    <a id="ref-for-limited-contribution"></a>

    <a id="ref-for-base-size⑨"></a>

    <a id="ref-for-max-content-constraint②"></a>

    <a id="ref-for-min-content-constraint②"></a>

    <a id="ref-for-grid-container①①⑤"></a>

    <a id="ref-for-min-track-sizing-function⑨"></a>

    <a id="ref-for-grid-placement-auto①①"></a>

    If the track has an [auto](#grid-placement-auto) [min track sizing function](#min-track-sizing-function) and the [grid container](#grid-container) is being sized under a [min-](https://www.w3.org/TR/css-sizing-3/#min-content-constraint)/[max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint), set the track’s [base size](#base-size) to the maximum of its items’ [limited min-content contributions](#limited-contribution), floored at zero. The <a id="limited-contribution"></a>limited min-/max-content contribution of an item is (for this purpose) its [min-](https://www.w3.org/TR/css-sizing-3/#min-content-contribution)/[max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution) (accordingly), limited by the [max track sizing function](#max-track-sizing-function) (which could be the argument to a [fit-content()](#funcdef-grid-template-columns-fit-content) track sizing function) if that is [fixed](#fixed-sizing-function) and ultimately floored by its [minimum contribution](#minimum-contribution) (defined below).

    <a id="ref-for-base-size①⓪"></a>

    <a id="ref-for-minimum-contribution①"></a>

    <a id="ref-for-outer-size"></a>

    <a id="ref-for-preferred-size⑤"></a>

    <a id="ref-for-behave-as-auto①"></a>

    <a id="ref-for-containing-block⑧"></a>

    <a id="ref-for-min-width⑥"></a>

    <a id="ref-for-min-content-contribution⑦"></a>

    <a id="ref-for-intrinsic-size-contribution①"></a>

    Otherwise, set the track’s [base size](#base-size) to the maximum of its items’ [minimum contributions](#minimum-contribution), floored at zero. The <a id="minimum-contribution"></a>minimum contribution<a id="min-size-contribution"></a> of an item is the smallest [outer size](https://www.w3.org/TR/css-sizing-3/#outer-size) it can have. Specifically, if the item’s computed [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) [behaves as auto](https://www.w3.org/TR/css-sizing-3/#behave-as-auto) or depends on the size of its [containing block](https://www.w3.org/TR/css-display-4/#containing-block) in the relevant axis, its <a id="ref-for-minimum-contribution②"></a>minimum contribution is the <a id="ref-for-outer-size①"></a>outer size that would result from assuming the item’s used [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width) as its <a id="ref-for-preferred-size⑥"></a>preferred size; else the item’s <a id="ref-for-minimum-contribution③"></a>minimum contribution is its [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution). Because the <a id="ref-for-minimum-contribution④"></a>minimum contribution often depends on the size of the item’s content, it is considered a type of [intrinsic size contribution](https://www.w3.org/TR/css-sizing-3/#intrinsic-size-contribution).

    <a id="ref-for-valdef-width-auto②"></a>

    <a id="ref-for-minimum-contribution⑤"></a>

    <a id="ref-for-min-content-contribution⑧"></a>

    <a id="ref-for-max-content-contribution⑥"></a>

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: For items with a specified minimum size of [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) (the initial value), the [minimum contribution](#minimum-contribution) is usually equivalent to the [min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution)—​but can differ in some cases, see [§ 6.6 Automatic Minimum Size of Grid Items](#min-size-auto). Also, <a id="ref-for-minimum-contribution⑥"></a>minimum contribution ≤ <a id="ref-for-min-content-contribution⑨"></a>min-content contribution ≤ [max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution).

    For min-content maximums:  
    <a id="ref-for-min-content-contribution①⓪"></a>

    <a id="ref-for-growth-limit⑤"></a>

    <a id="ref-for-max-track-sizing-function①②"></a>

    <a id="ref-for-valdef-grid-template-columns-min-content⑥"></a>

    If the track has a [min-content](#valdef-grid-template-columns-min-content) [max track sizing function](#max-track-sizing-function), set its [growth limit](#growth-limit) to the maximum of the items’ [min-content contributions](https://www.w3.org/TR/css-sizing-3/#min-content-contribution).

    For max-content maximums:  
    <a id="ref-for-funcdef-grid-template-columns-fit-content⑧"></a>

    <a id="ref-for-max-content-contribution⑦"></a>

    <a id="ref-for-growth-limit⑥"></a>

    <a id="ref-for-max-track-sizing-function①③"></a>

    <a id="ref-for-valdef-grid-template-columns-max-content⑨"></a>

    If the track has a [max-content](#valdef-grid-template-columns-max-content) [max track sizing function](#max-track-sizing-function), set its [growth limit](#growth-limit) to the maximum of the items’ [max-content contributions](https://www.w3.org/TR/css-sizing-3/#max-content-contribution). For [fit-content()](#funcdef-grid-template-columns-fit-content) maximums, furthermore clamp this <a id="ref-for-growth-limit⑦"></a>growth limit by the <a id="ref-for-funcdef-grid-template-columns-fit-content⑨"></a>fit-content() argument.

    <a id="ref-for-growth-limit⑧"></a>

    <a id="ref-for-base-size①①"></a>

    In all cases, if a track’s [growth limit](#growth-limit) is now less than its [base size](#base-size), increase the <a id="ref-for-growth-limit⑨"></a>growth limit to match the <a id="ref-for-base-size①②"></a>base size.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: This step is a simplification of the steps below for handling spanning items, and should yield the same behavior as running those instructions on items with a span of 1.

3.  <a id="ref-for-flexible-sizing-function③"></a>

    <a id="algo-spanning-items"></a> <strong>Increase sizes to accommodate spanning items crossing content-sized tracks:</strong> Next, consider the items with a span of 2 that do not span a track with a [flexible sizing function](#flexible-sizing-function).

    1.  <a id="ref-for-minimum-contribution⑦"></a>

        <a id="ref-for-min-track-sizing-function①⓪"></a>

        <a id="ref-for-intrinsic-sizing-function④"></a>

        <a id="ref-for-base-size①③"></a>

        <a id="ref-for-distribute-extra-space"></a>

        <a id="track-size-intrinsic-min"></a> <strong>For intrinsic minimums:</strong> First [distribute extra space](#distribute-extra-space) to [base sizes](#base-size) of tracks with an [intrinsic](#intrinsic-sizing-function) [min track sizing function](#min-track-sizing-function), to accommodate these items’ [minimum contributions](#minimum-contribution).

        <a id="ref-for-min-content-constraint③"></a>

        <a id="ref-for-max-content-constraint③"></a>

        <a id="ref-for-limited-contribution①"></a>

        <a id="ref-for-minimum-contribution⑧"></a>

        <a id="ref-for-fixed-sizing-function⑥"></a>

        <a id="ref-for-max-track-sizing-function①④"></a>

        If the grid container is being sized under a [min-](https://www.w3.org/TR/css-sizing-3/#min-content-constraint) or [max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint), use the items’ [limited min-content contributions](#limited-contribution) in place of their [minimum contributions](#minimum-contribution) here. (For an item spanning multiple tracks, the upper limit used to calculate its <a id="ref-for-limited-contribution②"></a>limited min-/max-content contribution is the <em>sum</em> of the [fixed](#fixed-sizing-function) [max track sizing functions](#max-track-sizing-function) of any tracks it spans, and is applied if it only spans such tracks.)

    2.  <a id="ref-for-min-content-contribution①①"></a>

        <a id="ref-for-valdef-grid-template-columns-max-content①⓪"></a>

        <a id="ref-for-valdef-grid-template-columns-min-content⑦"></a>

        <a id="ref-for-min-track-sizing-function①①"></a>

        <a id="ref-for-base-size①④"></a>

        <a id="ref-for-distribute-extra-space①"></a>

        <a id="track-size-content-min"></a> <strong>For content-based minimums:</strong> Next continue to [distribute extra space](#distribute-extra-space) to the [base sizes](#base-size) of tracks with a [min track sizing function](#min-track-sizing-function) of [min-content](#valdef-grid-template-columns-min-content) or [max-content](#valdef-grid-template-columns-max-content), to accommodate these items' [min-content contributions](https://www.w3.org/TR/css-sizing-3/#min-content-contribution).

    3.  <a id="ref-for-limited-contribution③"></a>

        <a id="ref-for-valdef-grid-template-columns-max-content①①"></a>

        <a id="ref-for-valdef-grid-template-columns-auto①③"></a>

        <a id="ref-for-min-track-sizing-function①②"></a>

        <a id="ref-for-base-size①⑤"></a>

        <a id="ref-for-distribute-extra-space②"></a>

        <a id="ref-for-max-content-constraint④"></a>

        <a id="track-size-max-content-min"></a> <strong>For max-content minimums:</strong> Next, if the grid container is being sized under a [max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint), continue to [distribute extra space](#distribute-extra-space) to the [base sizes](#base-size) of tracks with a [min track sizing function](#min-track-sizing-function) of [auto](#valdef-grid-template-columns-auto) or [max-content](#valdef-grid-template-columns-max-content), to accommodate these items' [limited max-content contributions](#limited-contribution).

        <a id="ref-for-distribute-extra-space③"></a>

        <a id="ref-for-base-size①⑥"></a>

        <a id="ref-for-min-track-sizing-function①③"></a>

        <a id="ref-for-valdef-grid-template-columns-max-content①②"></a>

        <a id="ref-for-max-content-contribution⑧"></a>

        In all cases, continue to [distribute extra space](#distribute-extra-space) to the [base sizes](#base-size) of tracks with a [min track sizing function](#min-track-sizing-function) of [max-content](#valdef-grid-template-columns-max-content), to accommodate these items' [max-content contributions](https://www.w3.org/TR/css-sizing-3/#max-content-contribution).

    4.  <a id="ref-for-base-size①⑦"></a>

        <a id="ref-for-growth-limit①⓪"></a>

        If at this point any track’s [growth limit](#growth-limit) is now less than its [base size](#base-size), increase its <a id="ref-for-growth-limit①①"></a>growth limit to match its <a id="ref-for-base-size①⑧"></a>base size.

    5.  <a id="ref-for-min-content-contribution①②"></a>

        <a id="ref-for-max-track-sizing-function①⑤"></a>

        <a id="ref-for-intrinsic-sizing-function⑤"></a>

        <a id="ref-for-growth-limit①②"></a>

        <a id="ref-for-distribute-extra-space④"></a>

        <strong>For intrinsic maximums:</strong> Next [distribute extra space](#distribute-extra-space) to the [growth limits](#growth-limit) of tracks with [intrinsic](#intrinsic-sizing-function) [max track sizing function](#max-track-sizing-function), to accommodate these items' [min-content contributions](https://www.w3.org/TR/css-sizing-3/#min-content-contribution). Mark any tracks whose <a id="ref-for-growth-limit①③"></a>growth limit changed from infinite to finite in this step as <a id="infinitely-growable"></a>infinitely growable for the next step.

        <a id="ref-for-infinitely-growable"></a>

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Why does the [infinitely growable](#infinitely-growable) flag exist?
        > [Peter Salas explains](https://lists.w3.org/Archives/Public/www-style/2014Mar/0500.html):
        >
        > > ```text
        > > Consider the following case:
        > > 
        > > Two "auto" tracks (i.e. ''minmax(min-content, max-content) minmax(min-content, max-content)'').
        > > Item 1 is in track 1, and has min-content = max-content = 10.
        > > Item 2 spans tracks 1 and 2, and has min-content = 30, max-content = 100.
        > > 
        > > After resolving min-content/max-content for the first item, we have this.
        > > 
        > > track 1: base size = 10 growth limit = 10
        > > 
        > > track 2: base size = 0 growth limit = infinity
        > > 
        > > Then we resolve min-content/max-content for the second item.
        > > 
        > > Phase 1 sets the base size of track 2 to 20 so that the two tracks' base sizes sum to 30.
        > > Phase 2 does nothing because there are no relevant tracks.
        > > Phase 3 sets the growth limit of track 2 to 20 so that the two tracks' growth limits sum to 30.
        > > In phase 4, we need to grow the sum of the growth limits by 70 to accommodate item 2.
        > > Two options are:
        > > 
        > > 1. Grow each track’s growth limit equally,
        > >   and end up with growth limits = [45, 55].
        > > 2. Grow only the second track’s growth limit,
        > >   and end up with growth limits = [10, 90].
        > > 
        > > By not considering the just-set growth limit as a constraint during space distribution
        > > (i.e. by treating it as infinity),
        > > we get the second result,
        > > which we considered a better result because the first track remains sized exactly to the first item.
        > > ```
    6.  <a id="ref-for-max-content-contribution⑨"></a>

        <a id="ref-for-valdef-grid-template-columns-max-content①③"></a>

        <a id="ref-for-max-track-sizing-function①⑥"></a>

        <a id="ref-for-growth-limit①④"></a>

        <a id="ref-for-distribute-extra-space⑤"></a>

        <strong>For max-content maximums:</strong> Lastly continue to [distribute extra space](#distribute-extra-space) to the [growth limits](#growth-limit) of tracks with a [max track sizing function](#max-track-sizing-function) of [max-content](#valdef-grid-template-columns-max-content), to accommodate these items' [max-content contributions](https://www.w3.org/TR/css-sizing-3/#max-content-contribution).

    Repeat incrementally for items with greater spans until all items have been considered.

4.  <a id="ref-for-flexible-sizing-function④"></a>

    <a id="ref-for-flexible-tracks①"></a>

    <a id="algo-spanning-flex-items"></a> <strong>Increase sizes to accommodate spanning items crossing <a href="#flexible-tracks">flexible tracks</a>:</strong> Next, repeat the previous step instead considering (together, rather than grouped by span size) all items that <em>do</em> span a track with a [flexible sizing function](#flexible-sizing-function) while

    - <a id="ref-for-fixed-sizing-function⑦"></a>

      <a id="ref-for-flexible-tracks②"></a>

      distributing space <em>only</em> to [flexible tracks](#flexible-tracks) (i.e. treating all other tracks as having a [fixed sizing function](#fixed-sizing-function))

    - <a id="ref-for-flexible-tracks③"></a>

      <a id="ref-for-flexible-sizing-function⑤"></a>

      if the sum of the [flexible sizing functions](#flexible-sizing-function) of all [flexible tracks](#flexible-tracks) spanned by the item is greater than or equal to one, distributing space to such tracks according to the ratios of their <a id="ref-for-flexible-sizing-function⑥"></a>flexible sizing functions rather than distributing space equally; and if the sum is less than one, distributing that proportion of space according to the ratios of their <a id="ref-for-flexible-sizing-function⑦"></a>flexible sizing functions and the rest equally

5.  <a id="ref-for-base-size①⑨"></a>

    <a id="ref-for-flexible-tracks④"></a>

    <a id="ref-for-growth-limit①⑤"></a>

    <a id="algo-finite-growth"></a> If any track still has an infinite [growth limit](#growth-limit) (because, for example, it had no items placed in it or it is a [flexible track](#flexible-tracks)), set its <a id="ref-for-growth-limit①⑥"></a>growth limit to its [base size](#base-size).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: There is no single way to satisfy intrinsic sizing constraints when items span across multiple tracks. This algorithm embodies a number of heuristics which have been seen to deliver good results on real-world use-cases, such as the “game” examples earlier in this specification. This algorithm may be updated in the future to take into account more advanced heuristics as they are identified.

#### <a id="extra-space"></a>12.5.1.  Distributing Extra Space Across Spanned Tracks

To <a id="distribute-extra-space"></a>distribute extra space, perform the following steps, with these inputs:

- <a id="ref-for-base-size②⓪"></a>

  <a id="ref-for-growth-limit①⑦"></a>

  whether to affect [base sizes](#base-size) or [growth limits](#growth-limit) (the <var>affected size</var>s).

- which tracks to affect (the <var>affected track</var>s).

- what intrinsic size contributions are being accommodated (the <var>size contribution</var>s) of which grid items spanning those tracks (the <var>item</var>s).

1.  Maintain separately for each <var>affected track</var> a <var>planned increase</var>, initially set to 0. (This prevents the size increases from becoming order-dependent.)

2.  <a id="ref-for-list-iterate"></a>

    [For each](https://infra.spec.whatwg.org/#list-iterate) accommodated <var>item</var>, considering only tracks the item spans:

    1.  <a id="ref-for-base-size②①"></a>

        <a id="ref-for-growth-limit①⑧"></a>

        <strong>Find the space to distribute:</strong> Subtract the <var>affected size</var> of every spanned track (not just the <var>affected track</var>s) from the item’s <var>size contribution</var>, flooring it at zero. (For infinite [growth limits](#growth-limit), substitute the track’s [base size](#base-size).) This remaining size contribution is the <var>space</var> to distribute.

        ```text
        space = max(0, size contribution - ∑track-sizes)
        ```
    2.  <strong>Distribute <var>space</var> up to limits:</strong>

        Find the <var>item-incurred increase</var> for each <var>affected track</var> by: distributing the <var>space</var> equally among these tracks, freezing a track’s <var>item-incurred increase</var> as its <var>affected size</var> + <var>item-incurred increase</var> reaches its <var>limit</var> (and continuing to grow the unfrozen tracks as needed).

        <a id="ref-for-base-size②②"></a>

        <a id="ref-for-growth-limit①⑨"></a>

        <a id="ref-for-funcdef-grid-template-columns-fit-content①⓪"></a>

        <a id="ref-for-infinitely-growable①"></a>

        <a id="ref-for-grid-template-rows-track-sizing-function⑨"></a>

        For [base sizes](#base-size), the <var>limit</var> is its [growth limit](#growth-limit), capped by its [fit-content()](#funcdef-grid-template-columns-fit-content) argument if any. For <a id="ref-for-growth-limit②⓪"></a>growth limits, the <var>limit</var> is the <a id="ref-for-growth-limit②①"></a>growth limit if the <a id="ref-for-growth-limit②②"></a>growth limit is finite and the track is not [infinitely growable](#infinitely-growable), otherwise its <a id="ref-for-funcdef-grid-template-columns-fit-content①①"></a>fit-content() argument if it has a <a id="ref-for-funcdef-grid-template-columns-fit-content①②"></a>fit-content() [track sizing function](#grid-template-rows-track-sizing-function), and infinity otherwise.

        <a id="ref-for-growth-limit②③"></a>

        <a id="ref-for-infinitely-growable②"></a>

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: If the <var>affected size</var> was a [growth limit](#growth-limit) and the track is not marked [infinitely growable](#infinitely-growable), then each <var>item-incurred increase</var> will be zero.

    3.  <strong>Distribute <var>space</var> to non-affected tracks:</strong>

        If extra <var>space</var> remains at this point, and the item spans both <var>affected tracks</var> and non-<var>affected tracks</var>, distribute space as for the previous step, but into the non-<var>affected tracks</var> instead.

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: This distributes any remaining space into tracks that have not yet reached their growth limits, instead of violating the growth limits of the <var>affected tracks</var>.

    4.  <strong>Distribute <var>space</var> beyond limits:</strong>

        If extra <var>space</var> remains at this point, unfreeze and continue to distribute <var>space</var> to the <var>item-incurred increase</var> of…

        - <a id="ref-for-max-track-sizing-function①⑦"></a>

          <a id="ref-for-base-size②③"></a>

          when [accommodating minimum contributions](#track-size-intrinsic-min) or [accommodating min-content contributions](#track-size-content-min) into [base sizes](#base-size): any <var>affected track</var> that happens to also have an intrinsic [max track sizing function](#max-track-sizing-function); if there are no such tracks, then all <var>affected track</var>s.

        - <a id="ref-for-max-track-sizing-function①⑧"></a>

          <a id="ref-for-valdef-grid-template-columns-max-content①④"></a>

          <a id="ref-for-base-size②④"></a>

          when [accommodating max-content contributions](#track-size-max-content-min) into [base sizes](#base-size): any <var>affected track</var> that happens to also have a [max-content](#valdef-grid-template-columns-max-content) [max track sizing function](#max-track-sizing-function); if there are no such tracks, then all <var>affected track</var>s.

        - <a id="ref-for-max-track-sizing-function①⑨"></a>

          <a id="ref-for-growth-limit②④"></a>

          when accommodating any contribution into [growth limits](#growth-limit): any <var>affected track</var> that has an intrinsic [max track sizing function](#max-track-sizing-function).

        <a id="ref-for-max-track-sizing-function②⓪"></a>

        <a id="ref-for-funcdef-grid-template-columns-fit-content①③"></a>

        <a id="ref-for-valdef-grid-template-columns-max-content①⑤"></a>

        <a id="ref-for-fixed-sizing-function⑧"></a>

        For this purpose, the [max track sizing function](#max-track-sizing-function) of a [fit-content()](#funcdef-grid-template-columns-fit-content) track is treated as [max-content](#valdef-grid-template-columns-max-content) until the track reaches the limit specified as the <a id="ref-for-funcdef-grid-template-columns-fit-content①④"></a>fit-content() argument, after which its <a id="ref-for-max-track-sizing-function②①"></a>max track sizing function is treated as being a [fixed sizing function](#fixed-sizing-function) of that argument (which can change which tracks continue to receive space in this step).

        <a id="ref-for-max-track-sizing-function②②"></a>

        > <strong data-conversion-semantic="note">Note</strong>
        >
        > Note: This step prioritizes the distribution of space for accommodating <var>size contribution</var>s beyond the tracks' current growth limits based on the types of their [max track sizing functions](#max-track-sizing-function).

    5.  For each <var>affected track</var>, if the track’s <var>item-incurred increase</var> is larger than the track’s <var>planned increase</var> set the track’s <var>planned increase</var> to that value.

3.  <a id="ref-for-base-size②⑤"></a>

    <a id="ref-for-growth-limit②⑤"></a>

    <strong>Update the tracks' <var>affected size</var>s</strong> by adding in the <var>planned increase</var>, so that the next round of space distribution will account for the increase. (If the affected size is an infinite [growth limit](#growth-limit), set it to the track’s [base size](#base-size) plus the <var>planned increase</var>.)

### <a id="algo-grow-tracks"></a>12.6.  Maximize Tracks

<a id="ref-for-free-space③"></a>

<a id="ref-for-base-size②⑥"></a>

<a id="ref-for-growth-limit②⑥"></a>

If the [free space](#free-space) is positive, distribute it equally to the [base sizes](#base-size) of all tracks, freezing tracks as they reach their [growth limits](#growth-limit) (and continuing to grow the unfrozen tracks as needed).

<a id="ref-for-grid-container①①⑥"></a>

<a id="ref-for-max-content-constraint⑤"></a>

<a id="ref-for-free-space④"></a>

<a id="ref-for-min-content-constraint④"></a>

For the purpose of this step: if sizing the [grid container](#grid-container) under a [max-content constraint](https://www.w3.org/TR/css-sizing-3/#max-content-constraint), the [free space](#free-space) is infinite; if sizing under a [min-content constraint](https://www.w3.org/TR/css-sizing-3/#min-content-constraint), the <a id="ref-for-free-space⑤"></a>free space is zero.

<a id="ref-for-grid-container①①⑦"></a>

<a id="ref-for-inner-size②"></a>

<a id="ref-for-propdef-max-width"></a>

<a id="ref-for-available-grid-space⑤"></a>

If this would cause the grid to be larger than the [grid container’s](#grid-container) [inner size](https://www.w3.org/TR/css-sizing-3/#inner-size) as limited by its [max-width/height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width), then redo this step, treating the [available grid space](#available-grid-space) as equal to the <a id="ref-for-grid-container①①⑧"></a>grid container’s <a id="ref-for-inner-size③"></a>inner size when it’s sized to its <a id="ref-for-propdef-max-width①"></a>max-width/height.

### <a id="algo-flex-tracks"></a>12.7.  Expand Flexible Tracks

<a id="ref-for-flexible-tracks⑤"></a>

<a id="ref-for-valdef-flex-fr⑥"></a>

<a id="ref-for-available⑤"></a>

This step sizes [flexible tracks](#flexible-tracks) using the largest value it can assign to an [fr](#valdef-flex-fr) without exceeding the [available space](https://www.w3.org/TR/css-sizing-3/#available).

<a id="ref-for-flex-fraction"></a>

First, find the grid’s used [flex fraction](#flex-fraction):

<a id="ref-for-min-content-constraint⑤"></a>

<a id="ref-for-grid-container①①⑨"></a>

<a id="ref-for-free-space⑥"></a>

If the [free space](#free-space) is zero or if sizing the [grid container](#grid-container) under a [min-content constraint](https://www.w3.org/TR/css-sizing-3/#min-content-constraint):

<a id="ref-for-flex-fraction①"></a>

The used [flex fraction](#flex-fraction) is zero.

<a id="ref-for-definite②⓪"></a>

<a id="ref-for-free-space⑦"></a>

Otherwise, if the [free space](#free-space) is a [definite](https://www.w3.org/TR/css-sizing-3/#definite) length:

<a id="ref-for-available-grid-space⑥"></a>

<a id="ref-for-space-to-fill"></a>

<a id="ref-for-grid-track③③"></a>

<a id="ref-for-flex-fraction②"></a>

The used [flex fraction](#flex-fraction) is the result of [finding the size of an fr](#algo-find-fr-size) using all of the [grid tracks](#grid-track) and a [space to fill](#space-to-fill) of the [available grid space](#available-grid-space).

<a id="ref-for-indefinite⑦"></a>

<a id="ref-for-free-space⑧"></a>

Otherwise, if the [free space](#free-space) is an [indefinite](https://www.w3.org/TR/css-sizing-3/#indefinite) length:

<a id="ref-for-flex-fraction③"></a>

The used [flex fraction](#flex-fraction) is the maximum of:

- <a id="ref-for-base-size②⑦"></a>

  <a id="ref-for-grid-template-columns-flex-factor⑦"></a>

  For each flexible track, if the flexible track’s [flex factor](#grid-template-columns-flex-factor) is greater than one, the result of dividing the track’s [base size](#base-size) by its <a id="ref-for-grid-template-columns-flex-factor⑧"></a>flex factor; otherwise, the track’s <a id="ref-for-base-size②⑧"></a>base size.

- <a id="ref-for-max-content-contribution①⓪"></a>

  <a id="ref-for-space-to-fill①"></a>

  <a id="ref-for-grid-item①③⓪"></a>

  For each [grid item](#grid-item) that crosses a flexible track, the result of [finding the size of an fr](#algo-find-fr-size) using all the grid tracks that the item crosses and a [space to fill](#space-to-fill) of the item’s [max-content contribution](https://www.w3.org/TR/css-sizing-3/#max-content-contribution).

<a id="ref-for-flex-fraction④"></a>

<a id="ref-for-grid③⓪"></a>

<a id="ref-for-grid-container①②⓪"></a>

<a id="ref-for-propdef-min-width③"></a>

<a id="ref-for-propdef-max-width②"></a>

<a id="ref-for-free-space⑨"></a>

<a id="ref-for-available-grid-space⑦"></a>

<a id="ref-for-inner-size④"></a>

If using this [flex fraction](#flex-fraction) would cause the [grid](#grid) to be smaller than the [grid container’s](#grid-container) [min-width/height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width) (or larger than the <a id="ref-for-grid-container①②①"></a>grid container’s [max-width/height](https://www.w3.org/TR/CSS2/visudet.html#propdef-max-width)), then redo this step, treating the [free space](#free-space) as definite and the [available grid space](#available-grid-space) as equal to the <a id="ref-for-grid-container①②②"></a>grid container’s [inner size](https://www.w3.org/TR/css-sizing-3/#inner-size) when it’s sized to its <a id="ref-for-propdef-min-width④"></a>min-width/height (<a id="ref-for-propdef-max-width③"></a>max-width/height).

<a id="ref-for-flexible-tracks⑥"></a>

<a id="ref-for-flex-fraction⑤"></a>

<a id="ref-for-grid-template-columns-flex-factor⑨"></a>

<a id="ref-for-base-size②⑨"></a>

For each [flexible track](#flexible-tracks), if the product of the used [flex fraction](#flex-fraction) and the track’s [flex factor](#grid-template-columns-flex-factor) is greater than the track’s [base size](#base-size), set its <a id="ref-for-base-size③⓪"></a>base size to that product.

<a id="ref-for-valdef-flex-fr⑦"></a>

#### <a id="algo-find-fr-size"></a>12.7.1.  Find the Size of an [fr](#valdef-flex-fr)

<a id="ref-for-valdef-flex-fr⑧"></a>

<a id="ref-for-grid-track③④"></a>

This algorithm finds the largest size that an [fr](#valdef-flex-fr) unit can be without exceeding the target size. It must be called with a set of [grid tracks](#grid-track) and some quantity of <a id="space-to-fill"></a>space to fill.

1.  <a id="ref-for-grid-track③⑤"></a>

    <a id="ref-for-base-size③①"></a>

    <a id="ref-for-space-to-fill②"></a>

    Let <a id="leftover-space"></a>leftover space be the [space to fill](#space-to-fill) minus the [base sizes](#base-size) of the non-flexible [grid tracks](#grid-track).

2.  <a id="ref-for-flexible-tracks⑦"></a>

    <a id="ref-for-grid-template-columns-flex-factor①⓪"></a>

    Let <a id="flex-factor-sum"></a>flex factor sum be the sum of the [flex factors](#grid-template-columns-flex-factor) of the [flexible tracks](#flexible-tracks). If this value is less than 1, set it to 1 instead.

3.  <a id="ref-for-flex-factor-sum"></a>

    <a id="ref-for-leftover-space⑦"></a>

    Let the <a id="hypothetical-fr-size"></a>hypothetical fr size be the [leftover space](#leftover-space) divided by the [flex factor sum](#flex-factor-sum).

4.  <a id="ref-for-grid-template-columns-flex-factor①①"></a>

    <a id="ref-for-flexible-tracks⑧"></a>

    <a id="ref-for-hypothetical-fr-size"></a>

    If the product of the [hypothetical fr size](#hypothetical-fr-size) and a [flexible track](#flexible-tracks)’s [flex factor](#grid-template-columns-flex-factor) is less than the track’s base size, restart this algorithm treating all such tracks as inflexible.

5.  <a id="ref-for-hypothetical-fr-size①"></a>

    Return the [hypothetical fr size](#hypothetical-fr-size).

### <a id="algo-stretch"></a>12.8.  Stretch auto Tracks

<a id="ref-for-content-distribution-properties"></a>

<a id="ref-for-grid-container①②③"></a>

<a id="ref-for-valdef-justify-content-normal"></a>

<a id="ref-for-valdef-align-content-stretch①"></a>

<a id="ref-for-valdef-grid-template-columns-auto①④"></a>

<a id="ref-for-max-track-sizing-function②③"></a>

<a id="ref-for-definite②①"></a>

<a id="ref-for-free-space①⓪"></a>

<a id="ref-for-indefinite⑧"></a>

<a id="ref-for-propdef-min-width⑤"></a>

When the [content-distribution property](https://www.w3.org/TR/css-align-3/#content-distribution-properties) of the [grid container](#grid-container) is [normal](https://www.w3.org/TR/css-align-3/#valdef-justify-content-normal) or [stretch](https://www.w3.org/TR/css-align-3/#valdef-align-content-stretch) in this axis, this step expands tracks that have an [auto](#valdef-grid-template-columns-auto) [max track sizing function](#max-track-sizing-function) by dividing any remaining positive, [definite](https://www.w3.org/TR/css-sizing-3/#definite) [free space](#free-space) equally amongst them. If the <a id="ref-for-free-space①①"></a>free space is [indefinite](https://www.w3.org/TR/css-sizing-3/#indefinite), but the <a id="ref-for-grid-container①②④"></a>grid container has a <a id="ref-for-definite②②"></a>definite [min-width/height](https://www.w3.org/TR/CSS2/visudet.html#propdef-min-width), use that size to calculate the <a id="ref-for-free-space①②"></a>free space for this step instead.

## <a id="pagination"></a>13.  Fragmenting Grid Layout

<a id="ref-for-grid-container①②⑤"></a>

<a id="ref-for-propdef-break-before"></a>

[Grid containers](#grid-container) can break across pages between rows or columns and inside items. The [break-\*](https://www.w3.org/TR/css-break-3/#propdef-break-before) properties apply to grid containers as normal for the formatting context in which they participate. This section defines how they apply to grid items and the contents of grid items.

<a id="ref-for-fragmentation-container"></a>

<a id="ref-for-fragmentation-context"></a>

The following breaking rules refer to the [fragmentation container](https://www.w3.org/TR/css-break-4/#fragmentation-container) as the “page”. The same rules apply in any other [fragmentation context](https://www.w3.org/TR/css-break-4/#fragmentation-context). (Substitute “page” with the appropriate <a id="ref-for-fragmentation-container①"></a>fragmentation container type as needed.) See the [CSS Fragmentation Module](https://www.w3.org/TR/css-break/) [\[CSS3-BREAK\]](#biblio-css3-break).

The exact layout of a fragmented grid container is not defined in this level of Grid Layout. However, breaks inside a grid container are subject to the following rules:

- <a id="ref-for-grid-item①③①"></a>

  <a id="ref-for-propdef-break-after"></a>

  <a id="ref-for-propdef-break-before①"></a>

  The [break-before](https://www.w3.org/TR/css-break-3/#propdef-break-before) and [break-after](https://www.w3.org/TR/css-break-3/#propdef-break-after) properties on [grid items](#grid-item) are propagated to their grid row. The <a id="ref-for-propdef-break-before②"></a>break-before property on the first row and the <a id="ref-for-propdef-break-after①"></a>break-after property on the last row are propagated to the grid container.

- A forced break inside a grid item effectively increases the size of its contents; it does not trigger a forced break inside sibling items.

- [Class A break opportunities](https://www.w3.org/TR/css3-break/#btw-blocks) occur between rows or columns (whichever is in the appropriate axis), and [Class C break opportunities](https://www.w3.org/TR/css3-break/#end-block) occur between the first/last row (column) and the grid container’s content edges. [\[CSS3-BREAK\]](#biblio-css3-break)

- <a id="ref-for-grid-item①③②"></a>

  When a grid container is continued after a break, the space available to its [grid items](#grid-item) (in the block flow direction of the fragmentation context) is reduced by the space consumed by grid container fragments on previous pages. The space consumed by a grid container fragment is the size of its content box on that page. If as a result of this adjustment the available space becomes negative, it is set to zero.

- Aside from the rearrangement of items imposed by the previous point, UAs should attempt to minimize distortion of the grid container with respect to unfragmented flow.

### <a id="fragmentation-alg"></a>13.1.  Sample Fragmentation Algorithm

<em>This section is non-normative.</em>

> <strong data-conversion-semantic="note">Note</strong>
>
> This is a rough draft of one possible fragmentation algorithm, and still needs to be severely cross-checked with the [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1) algorithm for consistency. Feedback is welcome; please reference the rules above instead as implementation guidance.

1.  <a id="ref-for-valdef-flex-fr⑨"></a>

    <a id="ref-for-grid-placement-auto①②"></a>

    <a id="ref-for-propdef-grid-row⑥"></a>

    <a id="ref-for-fragmentation-container②"></a>

    <a id="ref-for-layout-algorithm②"></a>

    Layout the grid following the [§ 12 Grid Layout Algorithm](#layout-algorithm) by using the [fragmentation container](https://www.w3.org/TR/css-break-4/#fragmentation-container)’s inline size and assume unlimited block size. During this step all [grid-row](#propdef-grid-row) [auto](#grid-placement-auto) and [fr](#valdef-flex-fr) values must be resolved.

2.  Layout the grid container using the values resolved in the previous step.

3.  <a id="ref-for-grid-area③⑥"></a>

    If a [grid area](#grid-area)’s size changes due to fragmentation (do not include items that span rows in this decision), increase the grid row size as necessary for rows that either:

    - have a content min track sizing function.
    - are in a grid that does not have an explicit height and the grid row is flexible.

4.  <a id="ref-for-grid-placement-auto①③"></a>

    If the grid height is [auto](#grid-placement-auto), the height of the grid should be the sum of the final row sizes.

5.  If a grid area overflows the grid container due to margins being collapsed during fragmentation, extend the grid container to contain this grid area (this step is necessary in order to avoid circular layout dependencies due to fragmentation).

> <strong data-conversion-semantic="note">Note</strong>
>
> If the grid’s height is specified, steps three and four may cause the grid rows to overflow the grid.

## <a id="privacy"></a>14. Privacy Considerations

Grid Layout introduces no new privacy leaks.

## <a id="security"></a>15. Security Considerations

Grid Layout introduces no new security considerations.

## <a id="changes"></a>16.  Changes

### <a id="changes-202012"></a>16.1.  Changes since the [18 December 2020 CR](https://www.w3.org/TR/2020/CRD-css-grid-2-20201218/)

- <a id="ref-for-subgrid⑤⑧"></a>

  <a id="ref-for-x34①"></a>

  Clarified how [relative positioning](https://www.w3.org/TR/CSS2/visuren.html#x34) applies to [subgrids](#subgrid). ([Issue 7123](https://github.com/w3c/csswg-drafts/issues/7123))

  > - <a id="ref-for-subgrid⑤⑨"></a>
  >
  >   <a id="ref-for-x34②"></a>
  >
  >   <u>[Relative positioning](https://www.w3.org/TR/CSS2/visuren.html#x34) applies to [subgrids](#subgrid) as normal, and shifts the box and its content together as usual. (Note: Relative positioning takes place after alignment, and does not affect track sizing.)</u>

- <a id="ref-for-scrollbar-gutter①"></a>

  Clarified that the [scrollbar gutter](https://www.w3.org/TR/css-overflow-3/#scrollbar-gutter) is accounted for alongside the margin/border/padding when calculating the contribution of the subgrid itself (not just when accounting for its children). ([Issue 9935](https://github.com/w3c/csswg-drafts/issues/9935))

  > <a id="ref-for-subgrid⑥⓪"></a>
  >
  > For each edge of a non-empty [subgrid](#subgrid), to account for the subgrid’s margin/border/padding <u>(and any scrollbar gutter)</u> at that edge, … and are additionally inflated by the subgrid’s own margin/border/padding <u>/gutter</u> at that edge.

- Also incorporated [all changes to CSS Grid Level 1 since its previous publication](css-grid-1--CRD-css-grid-1-20250326--53a47980a217.md#changes-202012).

### <a id="changes-202008"></a>16.2.  Changes since the [August 2020 CR](https://www.w3.org/TR/2020/CR-css-grid-2-20200818/)

- <a id="ref-for-propdef-aspect-ratio"></a>

  <a id="change-2020-align-normal-aspect-ratio"></a> Fix errors introduced by in the previous CR of CSS Grid 1 by an undocumented attempt to clarify [§ 6.2 Grid Item Sizing](#grid-item-sizing) interaction with [aspect-ratio](https://www.w3.org/TR/css-sizing-4/#propdef-aspect-ratio).

- <a id="ref-for-preferred-aspect-ratio④"></a>

  <a id="change-2020-preferred-aspect-ratio"></a> Updated some instances of “intrinsic aspect ratio” to use the more generic term [preferred aspect ratio](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio). ([Issue 4962](https://github.com/w3c/csswg-drafts/issues/4962))

- <a id="change-2020-editorial"></a> Minor editorial clean-up and better alignment of terminology across specs.

### <a id="changes-20180904"></a> Changes since the [December 2019 CSS Grid Layout Level 2 Working Draft](https://www.w3.org/TR/2019/WD-css-grid-2-20191203/)

None, except the incorporation of the full text of [CSS Grid level 1](https://www.w3.org/TR/css-grid-1/).

See [Changes during Working Draft](https://www.w3.org/TR/2019/WD-css-grid-2-20191203/#changes) for previous changes.

### <a id="changes-2"></a>16.3.  Additions Since Level 1

The following features have been added since [Level 1](https://www.w3.org/TR/css-grid-1/):

- <a id="ref-for-valdef-grid-template-rows-subgrid⑥"></a>

  [Subgrids](#subgrids) ([subgrid](#valdef-grid-template-rows-subgrid))

## <a id="acknowledgements"></a>17.  Acknowledgements

Many thanks to Mats Palmgren of Mozilla, without whose support and feedback the subgrid feature would not be able to move forward. Thanks also to Daniel Tonon, who insisted on intelligent handling of gaps in subgrids and contributed illustrations; and Rachel Andrew and Jen Simmons who helped bridge the feedback gap between the CSS Working Group and the Web design/authoring community.

Lastly, the acknowledgements section of CSS Grid Level 2 would be incomplete without acknowledgement of everyone who made the monumental task of [CSS Grid Level 1](https://www.w3.org/TR/css-grid-1/) possible.

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

- [augmented grid](#augmented-grid), in § 10.1
- auto
  - [value for \<grid-line\>](#grid-placement-auto), in § 8.3
  - [value for grid-template-columns, grid-template-rows](#valdef-grid-template-columns-auto), in § 7.2.1
- [auto-fill](#valdef-repeat-auto-fill), in § 7.2.3.2
- [auto-fit](#valdef-repeat-auto-fit), in § 7.2.3.2
- [\[ auto-flow &#x26;&#x26; dense? \] \<'grid-auto-rows'\>? / \<'grid-template-columns'\>](#grid-s-auto-column), in § 7.8
- [automatic column position](#automatic-grid-position), in § 8
- [automatic column span](#automatic-grid-span), in § 8
- [automatic grid position](#automatic-grid-position), in § 8
- [automatic grid span](#automatic-grid-span), in § 8
- [automatic placement](#auto-placement), in § 7.7
- [automatic position](#automatic-grid-position), in § 8
- [automatic row position](#automatic-grid-position), in § 8
- [automatic row span](#automatic-grid-span), in § 8
- [automatic span](#automatic-grid-span), in § 8
- [auto-placement](#auto-placement), in § 7.7
- [auto-placement algorithm](#grid-item-placement-algorithm), in § 8.5
- [auto-placement cursor](#auto-placement-cursor), in § 8.5
- [\<auto-repeat\>](#typedef-auto-repeat), in § 7.2.3.1
- [\<auto-track-list\>](#typedef-auto-track-list), in § 7.2
- [available grid space](#available-grid-space), in § 12.2
- [base size](#base-size), in § 12.3
- [clamp](#clamp-a-grid-area), in § 5.4
- [clamp a grid area](#clamp-a-grid-area), in § 5.4
- [collapse](#collapsed-grid-track), in § 7.2.3.2
- [collapsed grid track](#collapsed-grid-track), in § 7.2.3.2
- [collapsed gutter](#collapsed-gutter), in § 11.1
- [collapsed track](#collapsed-grid-track), in § 7.2.3.2
- column
  - [definition of](#grid-column), in § 3
  - [value for grid-auto-flow](#valdef-grid-auto-flow-column), in § 7.7
- [column position](#grid-position), in § 8
- [column span](#grid-span), in § 8
- [computed repeat notation](#computed-repeat-notation), in § 7.2.5
- [computed track list](#computed-track-list), in § 7.2.5
- [computed track size](#computed-track-size), in § 7.2.5
- [content-based minimum size](#content-based-minimum-size), in § 6.6
- [content size suggestion](#content-size-suggestion), in § 6.6
- [cursor](#auto-placement-cursor), in § 8.5
- [\<custom-ident\>](#grid-placement-slot), in § 8.3
- [definite column position](#definite-grid-position), in § 8
- [definite grid position](#definite-grid-position), in § 8
- [definite position](#definite-grid-position), in § 8
- [definite row position](#definite-grid-position), in § 8
- [dense](#valdef-grid-auto-flow-dense), in § 7.7
- [distribute extra space](#distribute-extra-space), in § 12.5.1
- [explicit](#explicit-grid), in § 7.1
- [explicit column span](#explicit-grid-span), in § 8
- [explicit grid](#explicit-grid), in § 7.1
- [explicit grid column](#explicit-grid-track), in § 7.1
- [explicit grid properties](#explicit-grid-properties), in § 7.1
- [explicit grid row](#explicit-grid-track), in § 7.1
- [explicit grid span](#explicit-grid-span), in § 8
- [explicit grid track](#explicit-grid-track), in § 7.1
- [explicitly-assigned line name](#explicitly-assigned-line-name), in § 7.2.2
- [explicit row span](#explicit-grid-span), in § 8
- [explicit span](#explicit-grid-span), in § 8
- [\<explicit-track-list\>](#typedef-explicit-track-list), in § 7.2
- [fit-content()](#funcdef-grid-template-columns-fit-content), in § 7.2.1
- [\<fixed-breadth\>](#typedef-fixed-breadth), in § 7.2
- [\<fixed-repeat\>](#typedef-fixed-repeat), in § 7.2.3.1
- [\<fixed-size\>](#typedef-fixed-size), in § 7.2
- [fixed sizing function](#fixed-sizing-function), in § 12.1
- [\<flex\>](#typedef-flex), in § 7.2.4
- [\<flex \[0,∞\]\>](#valdef-grid-template-columns-flex-0), in § 7.2.1
- [flex factor](#grid-template-columns-flex-factor), in § 7.2.1
- [flex factor sum](#flex-factor-sum), in § 12.7.1
- [flex fraction](#flex-fraction), in § 7.2.4
- [flexible length](#flexible-length), in § 7.2.4
- [flexible sizing function](#flexible-sizing-function), in § 12.1
- [flexible tracks](#flexible-tracks), in § 7.2.4
- [fr](#valdef-flex-fr), in § 7.2.4
- [free space](#free-space), in § 12.2
- [fr unit](#valdef-flex-fr), in § 7.2.4
- grid
  - [(property)](#propdef-grid), in § 7.8
  - [definition of](#grid), in § 3
  - [value for display](#valdef-display-grid), in § 5.1
- [grid area](#grid-area), in § 3.3
- [grid-area](#propdef-grid-area), in § 8.4
- [grid-auto-columns](#propdef-grid-auto-columns), in § 7.6
- [grid-auto-flow](#propdef-grid-auto-flow), in § 7.7
- [grid-auto-rows](#propdef-grid-auto-rows), in § 7.6
- [grid cell](#grid-cell), in § 3.2
- [grid column](#grid-column), in § 3
- [grid-column](#propdef-grid-column), in § 8.4
- [grid-column-end](#propdef-grid-column-end), in § 8.3
- [grid column line](#grid-line), in § 3.1
- [grid-column-start](#propdef-grid-column-start), in § 8.3
- [grid container](#grid-container), in § 5.1
- [grid formatting context](#grid-formatting-context), in § 5.1
- [grid item](#grid-item), in § 6
- [grid item placement algorithm](#grid-item-placement-algorithm), in § 8.5
- [grid layout](#grid-layout), in § 3
- [Grid Layout Algorithm](#layout-algorithm), in § 11.6
- [grid-level](#grid-level), in § 6.1
- [\<grid-line\>](#typedef-grid-row-start-grid-line), in § 8.3
- [grid line](#grid-line), in § 3.1
- [grid-modified document order](#grid-order), in § 11.6
- [grid order](#grid-order), in § 11.6
- [grid placement](#grid-placement), in § 8
- [grid-placement property](#grid-placement-property), in § 8
- [grid position](#grid-position), in § 8
- [grid row](#grid-row), in § 3
- [grid-row](#propdef-grid-row), in § 8.4
- [grid-row-end](#propdef-grid-row-end), in § 8.3
- [grid row line](#grid-line), in § 3.1
- [grid-row-start](#propdef-grid-row-start), in § 8.3
- [Grid Sizing Algorithm](#algo-grid-sizing), in § 12
- [grid span](#grid-span), in § 8
- [grid-template](#propdef-grid-template), in § 7.4
- [grid-template-areas](#propdef-grid-template-areas), in § 7.3
- [grid-template-columns](#propdef-grid-template-columns), in § 7.2
- [grid-template-rows](#propdef-grid-template-rows), in § 7.2
- [\<'grid-template-rows'\> / \[ auto-flow &#x26;&#x26; dense? \] \<'grid-auto-columns'\>?](#grid-s-auto-row), in § 7.8
- [\<'grid-template-rows'\> / \<'grid-template-columns'\>](#grid-template-rowcol), in § 7.4
- [Grid track](#grid-track), in § 3.2
- [growth limit](#growth-limit), in § 12.3
- [hypothetical fr size](#hypothetical-fr-size), in § 12.7.1
- [implicit](#implicit-grid), in § 7.5
- [implicit column span](#implicit-grid-span), in § 8
- [implicit grid](#implicit-grid), in § 7.5
- [implicit grid column](#implicit-grid-track), in § 7.5
- [implicit grid lines](#implicit-grid-lines), in § 7.5
- [implicit grid properties](#implicit-grid-properties), in § 7.5
- [implicit grid row](#implicit-grid-track), in § 7.5
- [implicit grid span](#implicit-grid-span), in § 8
- [implicit grid track](#implicit-grid-track), in § 7.5
- [implicitly-assigned line name](#implicitly-assigned-line-name), in § 7.3.2
- [implicitly-named area](#implicitly-named-area), in § 7.3.3
- [implicit row span](#implicit-grid-span), in § 8
- [implicit span](#implicit-grid-span), in § 8
- [infinitely growable](#infinitely-growable), in § 12.5
- [\<inflexible-breadth\>](#typedef-inflexible-breadth), in § 7.2
- [inline-grid](#valdef-display-inline-grid), in § 5.1
- [\[ \<integer \[-∞,-1\]\> \| \<integer \[1,∞\]\> \] &#x26;&#x26; \<custom-ident\>?](#grid-placement-int), in § 8.3
- [intrinsic sizing function](#intrinsic-sizing-function), in § 12.1
- [leftover space](#leftover-space), in § 12.7.1
- [\<length-percentage \[0,∞\]\>](#valdef-grid-template-columns-length-percentage-0), in § 7.2.1
- [limited max-content contribution](#limited-contribution), in § 12.5
- [limited min-content contribution](#limited-contribution), in § 12.5
- [line name](#line-name), in § 7.2.2
- [\<line-name-list\>](#typedef-line-name-list), in § 7.2
- [\<line-names\>](#typedef-line-names), in § 7.2
- [line name set](#line-name-set), in § 7.2.5
- [\[ \<line-names\>? \<string\> \<track-size\>? \<line-names\>? \]+ \[ / \<explicit-track-list\> \]?](#grid-template-ascii), in § 7.4
- [max-content](#valdef-grid-template-columns-max-content), in § 7.2.1
- [max track sizing function](#max-track-sizing-function), in § 12.2
- [min-content](#valdef-grid-template-columns-min-content), in § 7.2.1
- [minimum contribution](#minimum-contribution), in § 12.5
- [minmax()](#funcdef-grid-template-columns-minmax), in § 7.2.1
- [min track sizing function](#min-track-sizing-function), in § 12.2
- [named cell token](#grid-template-areas-named-cell-token), in § 7.3
- [named grid area](#named-grid-area), in § 7.3
- [\<name-repeat\>](#typedef-name-repeat), in § 7.2.3.1
- [nested grid](#nested-grid), in § 3.4
- none
  - [value for grid-template](#valdef-grid-template-none), in § 7.4
  - [value for grid-template-areas](#valdef-grid-template-areas-none), in § 7.3
  - [value for grid-template-rows, grid-template-columns](#valdef-grid-template-rows-none), in § 7.2
- [null cell token](#grid-template-areas-null-cell-token), in § 7.3
- [occupied](#occupied), in § 8.5
- [parent grid](#parent-grid), in § 3.4
- [placement](#grid-placement), in § 8
- [position](#grid-position), in § 8
- [repeat()](#funcdef-track-repeat-repeat), in § 7.2.3
- row
  - [definition of](#grid-row), in § 3
  - [value for grid-auto-flow](#valdef-grid-auto-flow-row), in § 7.7
- [row position](#grid-position), in § 8
- [row span](#grid-span), in § 8
- [sizing function](#grid-template-rows-track-sizing-function), in § 7.2
- [space to fill](#space-to-fill), in § 12.7.1
- [span](#grid-span), in § 8
- [span count](#span-count), in § 12.2
- [span &#x26;&#x26; \[ \<integer \[1,∞\]\> \|\| \<custom-ident\> \]](#grid-placement-span-int), in § 8.3
- [specified size suggestion](#specified-size-suggestion), in § 6.6
- [standalone axis](#standalone-axis), in § 7.2
- [standalone grid](#standalone-grid), in § 3.4
- [\<string\>+](#valdef-grid-template-areas-string), in § 7.3
- subgrid
  - [definition of](#subgrid), in § 3.4
  - [value for grid-template-rows, grid-template-columns](#valdef-grid-template-rows-subgrid), in § 7.2
- [subgridded](#subgridded-axis), in § 7.2
- [subgridded axis](#subgridded-axis), in § 7.2
- [subgrid \<line-name-list\>?](#subgrid-listing), in § 7.2
- [track](#grid-track), in § 3.2
- [\<track-breadth\>](#typedef-track-breadth), in § 7.2
- [\<track-list\>](#typedef-track-list), in § 7.2
- [track list](#track-list), in § 7.2
- [\<track-list\> \| \<auto-track-list\>](#track-listing), in § 7.2
- [\<track-repeat\>](#typedef-track-repeat), in § 7.2.3.1
- [track section](#track-section), in § 7.2.5
- [\<track-size\>](#typedef-track-size), in § 7.2
- [track sizing algorithm](#track-sizing-algorithm), in § 12.3
- [track sizing function](#grid-template-rows-track-sizing-function), in § 7.2
- [transferred size suggestion](#transferred-size-suggestion), in § 6.6
- [trash token](#grid-template-areas-trash-token), in § 7.3
- [unoccupied](#unoccupied), in § 8.5

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ALIGN-3\] defines the following terms:
  - <a id="fde954ee"></a>align-content
  - <a id="e6b1dd7a"></a>align-items
  - <a id="9d95c839"></a>align-self
  - <a id="73f86444"></a>alignment baseline
  - <a id="0e506341"></a>alignment context
  - <a id="4d2cf2cf"></a>baseline alignment
  - <a id="8bbaad92"></a>baseline set
  - <a id="207a83d3"></a>baseline-sharing group
  - <a id="567f0e7d"></a>box alignment properties
  - <a id="515ec31f"></a>center
  - <a id="b7d152c3"></a>column-gap
  - <a id="c4db3a90"></a>content-distribution properties
  - <a id="da7c24e2"></a>distributed alignment
  - <a id="3c3b0bc2"></a>fallback alignment
  - <a id="b4d83355"></a>gap
  - <a id="d3e6a513"></a>generate baselines
  - <a id="2ef18aa0"></a>gutter
  - <a id="de6bd31b"></a>justify-content
  - <a id="e4237559"></a>justify-items
  - <a id="80d2b689"></a>justify-self
  - <a id="fd3ce740"></a>normal (for align-self)
  - <a id="50284573"></a>normal (for justify-content)
  - <a id="e975f960"></a>normal (for row-gap)
  - <a id="0ef4bcd3"></a>place-content
  - <a id="2b373266"></a>row-gap
  - <a id="34ae2cc3"></a>self-alignment properties
  - <a id="5a961383"></a>space-around
  - <a id="e65dab9b"></a>space-between
  - <a id="920a13c0"></a>space-evenly
  - <a id="35e80fd9"></a>start
  - <a id="ec67a9e7"></a>stretch (for align-content)
  - <a id="598fa031"></a>stretch (for align-self)
  - <a id="8adaf629"></a>stretch (for justify-self)
  - <a id="81f3a960"></a>synthesize baseline
- \[CSS-BOX-4\] defines the following terms:
  - <a id="f72f5cb4"></a>content box
  - <a id="d049494a"></a>margin
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="4904f647"></a>fragmentation container
  - <a id="7eb0e25a"></a>fragmentation context
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="e14541aa"></a>shorthand
  - <a id="d5e08d9c"></a>specified value
  - <a id="b49aeda5"></a>sub-property
  - <a id="1a2b1083"></a>used value
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="7507b495"></a>layout containment
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="409c2774"></a>anonymous
  - <a id="45f9eae9"></a>block box
  - <a id="8d18d112"></a>block container
  - <a id="5a1cd654"></a>block formatting context
  - <a id="5b74f1e0"></a>block layout
  - <a id="a015488b"></a>block-level
  - <a id="431a9cad"></a>blockify
  - <a id="0923db9e"></a>containing block
  - <a id="e8c16097"></a>display
  - <a id="abe7937d"></a>establishes an independent formatting context
  - <a id="10a5b15f"></a>flow layout
  - <a id="6658d41f"></a>in-flow
  - <a id="57aa8824"></a>independent formatting context
  - <a id="6345c890"></a>inline formatting context
  - <a id="6b9bba07"></a>inline-level
  - <a id="ff5f937c"></a>out-of-flow
  - <a id="a9db5d6d"></a>replaced element
  - <a id="c9e837e8"></a>text node
  - <a id="e4657f7f"></a>text sequence
- \[CSS-FLEXBOX-1\] defines the following terms:
  - <a id="cc7f0a64"></a>flex container
  - <a id="9f6d5ab0"></a>flex item
  - <a id="546f7867"></a>flex-flow
  - <a id="7a8d5db2"></a>order
  - <a id="11b7dd33"></a>order-modified document order
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="c0cc78c8"></a>natural size
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="2d8be2d9"></a>vertical-align
- \[CSS-MULTICOL-2\] defines the following terms:
  - <a id="ec0972a7"></a>multi-column container
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="add377f4"></a>overflow
  - <a id="a3cabdb1"></a>scroll container
  - <a id="b1f927bc"></a>scrollable overflow rectangle
  - <a id="a1e893e7"></a>scrollable overflow region
  - <a id="79ac2a0a"></a>scrollable overflow value
  - <a id="79e20b40"></a>scrollbar gutter
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="4cf3c57e"></a>absolute position
  - <a id="f411d42d"></a>bottom
  - <a id="ebcbc56d"></a>left
  - <a id="b8c34db8"></a>position
  - <a id="c70a6e95"></a>relative
  - <a id="a5bae6ee"></a>right
  - <a id="35f1d972"></a>static
  - <a id="f99d4ae2"></a>top
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="63b59bd9"></a>::first-letter
  - <a id="4bda66a9"></a>::first-line
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="c20b5ff5"></a>auto
  - <a id="e9c67130"></a>automatic minimum size
  - <a id="37f6dbd7"></a>automatic size
  - <a id="c1c732b9"></a>available space
  - <a id="a91daf3b"></a>behave as auto
  - <a id="6fc38b8c"></a>behaves as auto
  - <a id="66f218c1"></a>definite
  - <a id="2a2ed19e"></a>indefinite
  - <a id="de48a940"></a>inner size
  - <a id="59e3c405"></a>intrinsic size contribution
  - <a id="a47902ec"></a>max-content constraint
  - <a id="c9ef7223"></a>max-content contribution
  - <a id="8a39af7f"></a>max-content size
  - <a id="6d275904"></a>maximum size
  - <a id="451a41ae"></a>min-content constraint
  - <a id="65c4b34c"></a>min-content contribution
  - <a id="6a444fd6"></a>min-content size
  - <a id="4405c984"></a>minimum size
  - <a id="47ea2436"></a>outer size
  - <a id="dd09245c"></a>preferred size
  - <a id="96676f36"></a>stretch fit
  - <a id="97ac8088"></a>stretch-fit size
  - <a id="49731d1d"></a>width
- \[CSS-SIZING-4\] defines the following terms:
  - <a id="d1cbb104"></a>aspect-ratio
  - <a id="6b530a45"></a>fit-content
  - <a id="b03f7c8f"></a>preferred aspect ratio
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="d63e6472"></a>ident code point
  - <a id="fa522ba4"></a>whitespace
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="b093a29f"></a>white-space
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="ef9f8297"></a>\*
  - <a id="af4a190d"></a>+
  - <a id="8cd4f032"></a>,
  - <a id="a0144f62"></a>\<custom-ident\>
  - <a id="dcecfc13"></a>\<ident\>
  - <a id="d73c993d"></a>\<integer\>
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="128295ac"></a>\<percentage\>
  - <a id="1d798932"></a>\<string\>
  - <a id="d4441b24"></a>?
  - <a id="14d3255d"></a>calc()
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="3bafef5e"></a>{A,B}
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="ecef1eb5"></a>block size
  - <a id="599428b5"></a>block-axis
  - <a id="83d2ef35"></a>block-end
  - <a id="1118d052"></a>block-start
  - <a id="e112902f"></a>end
  - <a id="303c8d41"></a>flow-relative
  - <a id="a6eb24bb"></a>inline axis
  - <a id="18bb1084"></a>inline size
  - <a id="82ddda8c"></a>inline-axis
  - <a id="4da3b716"></a>inline-end
  - <a id="0da67e16"></a>inline-start
  - <a id="7c77b965"></a>orthogonal flow
  - <a id="e1f6e4b9"></a>physical
  - <a id="90c7548c"></a>start
  - <a id="eb6008ce"></a>writing mode
  - <a id="37bb38a0"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="d332e4ec"></a>auto
  - <a id="019a586e"></a>clear
  - <a id="da486c10"></a>float
  - <a id="cfab9333"></a>margin
  - <a id="4d8f6525"></a>max-width
  - <a id="62b90f98"></a>min-height
  - <a id="1ecca6e7"></a>min-width
  - <a id="f285fdb1"></a>relative positioning
  - <a id="1848f1d3"></a>z-index
- \[CSS3-BREAK\] defines the following terms:
  - <a id="51ee3396"></a>break-after
  - <a id="eb306f02"></a>break-before
  - <a id="62c772f5"></a>fragment
- \[CSS3-WRITING-MODES\] defines the following terms:
  - <a id="fb688f4f"></a>direction
- \[CSSOM\] defines the following terms:
  - <a id="fc19454a"></a>resolved value
  - <a id="b1b54f44"></a>resolved value special case property
- \[INFRA\] defines the following terms:
  - <a id="16d07e10"></a>for each
  - <a id="649608b9"></a>list
  - <a id="15e48c39"></a>set
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="3ea2fcbb"></a>media query
- \[WEB-ANIMATIONS-1\] defines the following terms:
  - <a id="8a1a3001"></a>by computed value
  - <a id="2cec4673"></a>discrete

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

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 17 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-2&#x2F;](https://www.w3.org/TR/css-contain-2/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 19 December 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-flexbox-1"></a>\[CSS-FLEXBOX-1\]  
Tab Atkins Jr.; et al. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 19 November 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css-grid-1"></a>\[CSS-GRID-1\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 1](https://www.w3.org/TR/css-grid-1/). 18 December 2020. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-1&#x2F;](https://www.w3.org/TR/css-grid-1/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Elika Etemad. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 18 December 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 29 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 11 March 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 30 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-sizing-4"></a>\[CSS-SIZING-4\]  
Tab Atkins Jr.; Elika Etemad; Jen Simmons. [CSS Box Sizing Module Level 4](https://www.w3.org/TR/css-sizing-4/). 20 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-4&#x2F;](https://www.w3.org/TR/css-sizing-4/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

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

<a id="biblio-cssom"></a>\[CSSOM\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 18 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-web-animations-1"></a>\[WEB-ANIMATIONS-1\]  
Brian Birtles; et al. [Web Animations](https://www.w3.org/TR/web-animations-1/). 5 June 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;web-animations-1&#x2F;](https://www.w3.org/TR/web-animations-1/)

### <a id="informative"></a>Informative References

<a id="biblio-css-multicol-2"></a>\[CSS-MULTICOL-2\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 2](https://www.w3.org/TR/css-multicol-2/). 19 December 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-2&#x2F;](https://www.w3.org/TR/css-multicol-2/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

## <a id="property-index"></a>Property Index

<strong>Table 14 — structured row/cell transcription</strong>

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

<a id="ref-for-propdef-grid①③"></a>

[grid](#propdef-grid)

<strong>Column 2 (data cell):</strong>

\<'grid-template'\> \| \<'grid-template-rows'\> / \[ auto-flow &#x26;&#x26; dense? \] \<'grid-auto-columns'\>? \| \[ auto-flow &#x26;&#x26; dense? \] \<'grid-auto-rows'\>? / \<'grid-template-columns'\>

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

grid containers

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

<strong>Row 3</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-area⑥"></a>

[grid-area](#propdef-grid-area)

<strong>Column 2 (data cell):</strong>

\<grid-line\> \[ / \<grid-line\> \]{0,3}

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

grid items and absolutely-positioned boxes whose containing block is a grid container

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Row 4</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-auto-columns①②"></a>

[grid-auto-columns](#propdef-grid-auto-columns)

<strong>Column 2 (data cell):</strong>

\<track-size\>+

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

grid containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

see Track Sizing

<strong>Column 7 (data cell):</strong>

if the list lengths match, by computed value type per item; discrete otherwise

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see Track Sizing

<strong>Row 5</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-auto-flow①①"></a>

[grid-auto-flow](#propdef-grid-auto-flow)

<strong>Column 2 (data cell):</strong>

\[ row \| column \] \|\| dense

<strong>Column 3 (data cell):</strong>

row

<strong>Column 4 (data cell):</strong>

grid containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword(s)

<strong>Row 6</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-auto-rows①②"></a>

[grid-auto-rows](#propdef-grid-auto-rows)

<strong>Column 2 (data cell):</strong>

\<track-size\>+

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

grid containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

see Track Sizing

<strong>Column 7 (data cell):</strong>

if the list lengths match, by computed value type per item; discrete otherwise

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see Track Sizing

<strong>Row 7</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-column⑦"></a>

[grid-column](#propdef-grid-column)

<strong>Column 2 (data cell):</strong>

\<grid-line\> \[ / \<grid-line\> \]?

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

grid items and absolutely-positioned boxes whose containing block is a grid container

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Row 8</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-column-end①③"></a>

[grid-column-end](#propdef-grid-column-end)

<strong>Column 2 (data cell):</strong>

\<grid-line\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

grid items and absolutely-positioned boxes whose containing block is a grid container

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword, identifier, and/or integer

<strong>Row 9</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-column-start①③"></a>

[grid-column-start](#propdef-grid-column-start)

<strong>Column 2 (data cell):</strong>

\<grid-line\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

grid items and absolutely-positioned boxes whose containing block is a grid container

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword, identifier, and/or integer

<strong>Row 10</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-row⑦"></a>

[grid-row](#propdef-grid-row)

<strong>Column 2 (data cell):</strong>

\<grid-line\> \[ / \<grid-line\> \]?

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

grid items and absolutely-positioned boxes whose containing block is a grid container

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

N/A

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

see individual properties

<strong>Row 11</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-row-end①③"></a>

[grid-row-end](#propdef-grid-row-end)

<strong>Column 2 (data cell):</strong>

\<grid-line\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

grid items and absolutely-positioned boxes whose containing block is a grid container

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword, identifier, and/or integer

<strong>Row 12</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-row-start①③"></a>

[grid-row-start](#propdef-grid-row-start)

<strong>Column 2 (data cell):</strong>

\<grid-line\>

<strong>Column 3 (data cell):</strong>

auto

<strong>Column 4 (data cell):</strong>

grid items and absolutely-positioned boxes whose containing block is a grid container

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

specified keyword, identifier, and/or integer

<strong>Row 13</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-template①②"></a>

[grid-template](#propdef-grid-template)

<strong>Column 2 (data cell):</strong>

none \| \[ \<'grid-template-rows'\> / \<'grid-template-columns'\> \] \| \[ \<line-names\>? \<string\> \<track-size\>? \<line-names\>? \]+ \[ / \<explicit-track-list\> \]?

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

grid containers

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

<strong>Row 14</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-template-areas②④"></a>

[grid-template-areas](#propdef-grid-template-areas)

<strong>Column 2 (data cell):</strong>

none \| \<string\>+

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

grid containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

n/a

<strong>Column 7 (data cell):</strong>

discrete

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword none or a list of string values

<strong>Row 15</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-template-columns③①"></a>

[grid-template-columns](#propdef-grid-template-columns)

<strong>Column 2 (data cell):</strong>

none \| \<track-list\> \| \<auto-track-list\> \| subgrid \<line-name-list\>?

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

grid containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

refer to corresponding dimension of the content area

<strong>Column 7 (data cell):</strong>

if the list lengths match, by computed value type per item in the computed track list (see and ); discrete otherwise

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword none or a computed track list

<strong>Row 16</strong>

<strong>Column 1 (header cell; scope row):</strong>

<a id="ref-for-propdef-grid-template-rows②⑨"></a>

[grid-template-rows](#propdef-grid-template-rows)

<strong>Column 2 (data cell):</strong>

none \| \<track-list\> \| \<auto-track-list\> \| subgrid \<line-name-list\>?

<strong>Column 3 (data cell):</strong>

none

<strong>Column 4 (data cell):</strong>

grid containers

<strong>Column 5 (data cell):</strong>

no

<strong>Column 6 (data cell):</strong>

refer to corresponding dimension of the content area

<strong>Column 7 (data cell):</strong>

if the list lengths match, by computed value type per item in the computed track list (see and ); discrete otherwise

<strong>Column 8 (data cell):</strong>

per grammar

<strong>Column 9 (data cell):</strong>

the keyword none or a computed track list

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> If you notice any inconsistencies between this Grid Layout Module and the [Flexible Box Layout Module](https://www.w3.org/TR/css-flexbox/), please report them to the CSSWG, as this is likely an error. [↵](#issue-436134ac)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The first bullet point of the above list means that implicit tracks get serialized as part of [grid-template-rows](#propdef-grid-template-rows)/etc., despite the fact that an author <em>cannot</em> actually specify implicit track sizes in those properties! So grid-template-rows and [grid-template-columns](#propdef-grid-template-columns) values might not round-trip correctly:
>
> ```text
> const s = getComputedStyle(gridEl);
> gridEl.style.gridTemplateRows = s.gridTemplateRows;
> // Code like this should be a no-op,
> // but if there are any implicit rows,
> // this will convert them into explicit rows,
> // possibly changing how grid items are positioned
> // and altering the overall size of the grid!
> ```
>
> This is an accidental property of an early implementation that leaked into later implementations without much thought given to it. We intend to remove it from the spec, but not until after we’ve defined a CSSOM API for getting information about implicit tracks, as currently this is the only way to get that information and a number of pages rely on that.
>
> [↵](#issue-cccd0e19)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> The CSS Working Group is considering whether to also return used values for the [grid-placement properties](#grid-placement-property) and is looking for feedback, especially from implementors. See [discussion](https://github.com/w3c/csswg-drafts/issues/2681). [↵](#issue-e2bc4d57)
