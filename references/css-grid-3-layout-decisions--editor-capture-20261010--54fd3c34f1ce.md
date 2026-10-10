Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Grid Layout Module Level 3](https://drafts.csswg.org/css-grid-3/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-10. Source status: Editor’s Draft, 8 October 2026.

Full captured HTML SHA-256: `3d85385289b72096aa6ce0af877211e42401573304821b5cb71f3f2f85ee121f` (307148 bytes). Conversion input SHA-256: `54fd3c34f1ce1b5ff1937a46dc1ac1975012b01c045577ab2b2c2926df580fa3` (149527 bytes). Page revision metadata: `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`.

Retained scope: `abstract`, `sotd`, `grid-lanes-model`, `grid-lanes-track-templates`, `grid-lanes-track-placement`, `intrinsic-sizes`, `alignment`, `pagination`, `abspos`. Selected heading sections are complete, bounded at the next equal-or-higher heading. The original header and legal notice are retained. Other clauses remain upstream. Figures remain upstream image links. Multiline alternative descriptions use HTML image attributes with numeric whitespace entities to retain exact source alt text.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table contents are checked against conversion input. Synthetic table headers and span expansion are representation changes. Two one-line Grid 3 table examples omit only their terminal separator newline when represented as inline code; their exact source text remains in conversion provenance. Mathematical expressions are retained as source text; no equation is corrected. This capture preserves standards latitude and unresolved questions; Surgeist-selected policies are recorded separately in issues.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Grid Layout Module Level 3

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 8 October 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-grid-3/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-grid-3/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-grid-3)

[Inline In Spec](https://drafts.csswg.org/css-grid-3/#issues-index)

<strong>Editors:</strong>

[Tab Atkins Jr.](http://www.xanthir.com/contact/) (Google)

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Jen Simmons](http://jensimmons.com/) (Apple)

[Brandon Stewart](https://brandonstewart.net) (Apple)

<strong>Former Editor:</strong>

[Mats Palmgren](mailto:mailto:mats@mozilla.com) (Mozilla)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-grid-3/Overview.bs)

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="abstract"></a>Abstract

This module introduces a one-dimensional grid layout mode for [CSS Grid](https://www.w3.org/TR/css-grid-2/) containers.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

This is a public copy of the editors’ draft. It is provided for discussion only and may change at any moment. Its publication here does not imply endorsement of its contents by W3C. Don’t cite this document other than as work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-grid” in the title, like this: “\[css-grid\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-grid%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

<a id="toc"></a>

## <a id="contents"></a>Table of Contents

1.  [1 Introduction](https://drafts.csswg.org/css-grid-3/#intro)
    1.  [1.1 Background and Motivation](https://drafts.csswg.org/css-grid-3/#background)
        1.  [1.1.1 Waterfall Layout with Auto-placed Items](https://drafts.csswg.org/css-grid-3/#waterfall)
        2.  [1.1.2 One-dimensional Grid Layout](https://drafts.csswg.org/css-grid-3/#collapse)
    2.  [1.2 Value Definitions](https://drafts.csswg.org/css-grid-3/#values)
2.  [2 Grid Lanes Layout Model](#grid-lanes-model)
    1.  [2.1 Reordering and Accessibility](#order-accessibility)
    2.  [2.2 Establishing Grid Lanes Layout](#grid-lanes-containers)
    3.  [2.3 Orienting Grid Lanes Layout](#grid-lanes-orientation)
3.  [3 Grid Lanes Track Specification](#grid-lanes-track-templates)
    1.  [3.1 Declaring Grid Lanes Track Templates: the grid-template-\* properties](#grid-lanes-track-properties)
        1.  [3.1.1 Intrinsic Tracks and repeat()](#intrinsic-auto-repeat)
    2.  [3.2 Subgrids](#subgrids)
    3.  [3.3 Track Repetition: the repeat() notation](#repeat-notation)
        1.  [3.3.1 repeat(auto-fit)](#repeat-auto-fit)
    4.  [3.4 Grid Axis Track Sizing](#track-sizing)
        1.  [3.4.1 Subgrid Item Contributions](#track-sizing-subgrid)
        2.  [3.4.2 Optimized Track Sizing](#track-sizing-performance)
4.  [4 Grid Lanes Item Placement](#grid-lanes-track-placement)
    1.  [4.1 Specifying Grid Axis Item Placement: the grid-column-\* and grid-row-\* properties](#grid-lanes-placement)
    2.  [4.2 Placement Precision: the fit-tolerance property](#placement-tolerance)
    3.  [4.3 Dense Placement: the dense keyword](#grid-lanes-dense-packing)
    4.  [4.4 Grid Lanes Layout and Placement Algorithm](#grid-lanes-layout-algorithm)
        1.  [4.4.1 Containing Block](#containing-block)
        2.  [4.4.2 Placement and Writing Modes](#rtl-example)
5.  [5 Sizing Grid Containers](#intrinsic-sizes)
6.  [6 Alignment and Spacing](#alignment)
    1.  [6.1 Gutters: the row-gap, column-gap, and gap properties](#gutters)
    2.  [6.2 Grid-axis Alignment: the align-content/justify-content, align-self/justify-self, and align-items/justify-items properties](#grid-axis-alignment)
    3.  [6.3 Stacking-axis Content Distribution: the align-content/justify-content properties](#stacking-content-alignment)
    4.  [6.4 Stacking-axis Self Alignment: the align-self/justify-self and align-items/justify-items properties](#stacking-self-alignment)
    5.  [6.5 Baseline Alignment](#grid-lanes-baseline-alignment)
7.  [7 Fragmentation](#pagination)
    1.  [7.1 Fragmentation in the stacking axis](#stacking-axis-pagination)
    2.  [7.2 Fragmentation in the Grid Axis](#grid-axis-pagination)
8.  [8 Absolute Positioning](#abspos)
9.  [9 Graceful Degradation](https://drafts.csswg.org/css-grid-3/#graceful-degradation)
10. [10 Acknowledgements](https://drafts.csswg.org/css-grid-3/#acknowledgements)
11. [11 Security Considerations](https://drafts.csswg.org/css-grid-3/#security)
12. [12 Privacy Considerations](https://drafts.csswg.org/css-grid-3/#privacy)
13. [ Changes](https://drafts.csswg.org/css-grid-3/#changes)
    1.  [ Additions Since Level 2](https://drafts.csswg.org/css-grid-3/#changes-3)
    2.  [ Recent Changes](https://drafts.csswg.org/css-grid-3/#recent-changes)
14. [ Conformance](https://drafts.csswg.org/css-grid-3/#w3c-conformance)
    1.  [ Document conventions](https://drafts.csswg.org/css-grid-3/#w3c-conventions)
    2.  [ Conformance classes](https://drafts.csswg.org/css-grid-3/#w3c-conformance-classes)
    3.  [ Partial implementations](https://drafts.csswg.org/css-grid-3/#w3c-partial)
        1.  [ Implementations of Unstable and Proprietary Features](https://drafts.csswg.org/css-grid-3/#w3c-conform-future-proofing)
    4.  [ Non-experimental implementations](https://drafts.csswg.org/css-grid-3/#w3c-testing)
15. [ Index](https://drafts.csswg.org/css-grid-3/#index)
    1.  [ Terms defined by this specification](https://drafts.csswg.org/css-grid-3/#index-defined-here)
    2.  [ Terms defined by reference](https://drafts.csswg.org/css-grid-3/#index-defined-elsewhere)
16. [ References](https://drafts.csswg.org/css-grid-3/#references)
    1.  [ Normative References](https://drafts.csswg.org/css-grid-3/#normative)
    2.  [ Non-Normative References](https://drafts.csswg.org/css-grid-3/#informative)
17. [ Property Index](https://drafts.csswg.org/css-grid-3/#property-index)
18. [ Issues Index](https://drafts.csswg.org/css-grid-3/#issues-index)

## <a id="grid-lanes-model"></a>2. <a id="masonry-model"></a> Grid Lanes Layout Model[](#grid-lanes-model)

<a id="grid-lanes-layout"></a><strong>Grid lanes layout</strong> lays out items into pre-defined tracks similar to <a id="ref-for-grid-layout③"></a>[grid layout](https://drafts.csswg.org/css-grid-2/#grid-layout) in one axis (called the <a id="grid-axis"></a><strong>grid axis</strong>), but flows them freely similar to <a id="ref-for-flex-layout①"></a>[flex layout](https://drafts.csswg.org/css-flexbox-2/#flex-layout) in the other (called the <a id="stacking-axis"></a><strong>stacking axis</strong>). Similar to <a id="ref-for-grid-layout④"></a>grid layout and unlike <a id="ref-for-flex-layout②"></a>flex layout, <a id="ref-for-grid-lanes-layout④"></a>[grid lanes layout](#grid-lanes-layout)’s auto-placement distributes items across the tracks to keep the lengths of those tracks as similar as possible.

<a id="ref-for-grid-item"></a>[Grid items](https://drafts.csswg.org/css-grid-2/#grid-item) are formed and <a id="ref-for-blockify"></a>[blockified](https://drafts.csswg.org/css-display-4/#blockify) exactly the same as in a regular <a id="ref-for-grid-container"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container).

All CSS properties work the same as in a regular <a id="ref-for-grid-container①"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container) unless otherwise specified by this specification. For example, <a id="ref-for-propdef-order"></a>[order](https://drafts.csswg.org/css-display-4/#propdef-order) can be used to specify a different layout order for the items.

Note: Subgrid items are supported, but subgridding only occurs in the <a id="ref-for-grid-axis"></a>[grid axis](#grid-axis); see [§ 3.2 Subgrids](#subgrids) for details.

A <a id="grid-lanes-container"></a><strong>grid lanes container</strong> is a <a id="ref-for-grid-container②"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container) whose contents participate in <a id="ref-for-grid-lanes-layout⑤"></a>[grid lanes layout](#grid-lanes-layout). A <a id="ref-for-grid-lanes-container"></a>[grid lanes container](#grid-lanes-container) creates column <a id="ref-for-grid-track"></a>[tracks](https://drafts.csswg.org/css-grid-2/#grid-track) if its <a id="ref-for-stacking-axis"></a>[stacking axis](#stacking-axis) is the <a id="ref-for-block-axis"></a>[block axis](https://drafts.csswg.org/css-writing-modes-4/#block-axis), or row <a id="ref-for-grid-track①"></a>tracks if its <a id="ref-for-stacking-axis①"></a>stacking axis is the <a id="ref-for-inline-axis"></a>[inline axis](https://drafts.csswg.org/css-writing-modes-4/#inline-axis).

Comparing Grid Lanes Containers

| Column 1                      | Column 2                                             | Column 3                                                                                                                                                                                              |
|-------------------------------|------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Column Lanes</strong> | <code>grid-template-columns&#58; 1fr 2fr 3fr;</code> | <img src="https://drafts.csswg.org/css-grid-3/images/masonry-columns.png" alt="Column grid lanes layout lays out items in columns,&#10;&#9;&#9;&#9;&#9;&#9;     but ordered across the columns,&#10;&#9;&#9;&#9;&#9;&#9;     placing each item in the then-shortest column."> |
| <strong>Row Lanes</strong>    | <code>grid-template-rows&#58; 1fr 2fr 3fr;</code>    | <img src="https://drafts.csswg.org/css-grid-3/images/masonry-rows.png" alt="Row grid lanes layout lays out items in rows,&#10;&#9;&#9;&#9;&#9;&#9;     but ordered down across the rows,&#10;&#9;&#9;&#9;&#9;&#9;     placing each item in the then-shortest row.">           |

### <a id="order-accessibility"></a>2.1.  Reordering and Accessibility[](#order-accessibility)

Although <a id="ref-for-grid-lanes-layout⑥"></a>[grid lanes layout](#grid-lanes-layout) generally progresses in a forwards fashion (placing the next item endward of the current item in at least one axis, matching the natural “reading order”), it can switch between endward in the inline or block axis in a seemingly arbitrary manner. In simple cases, the <a id="ref-for-propdef-fit-tolerance"></a>[fit-tolerance](#propdef-fit-tolerance) property can help reduce the feeling of backtracking due to small sizing differences in the <a id="ref-for-stacking-axis②"></a>[stacking axis](#stacking-axis) when laying out auto-placed items. But when [auto-placement](#grid-lanes-layout-algorithm) is mixed with <a id="ref-for-definite-grid-position"></a>[explicit placement](https://drafts.csswg.org/css-grid-2/#definite-grid-position) or spanning items, some amount of backtracking may occur.

<a id="example-0b32dd26"></a>

<strong>Example:</strong>

[](#example-0b32dd26) For example, in the following markup sample, the fourth item is a spanner that doesn’t fit in the remaining empty column on the first line. It ends up positioned into the into the first column, which is the highest available space into which it will fit. The next few items, which have a span of 1, end up laying out “above” it in the empty column, violating the natural reading order.

``` text
<section class=masonry>
  <div class=item>1</div>
  <div class=item>2</div>
  <div class="item tall">3</div>
  <div class="item wide">4</div>
  <div class=item>5</div>
  <div class=item>6</div>
  <div class=item>7</div>
</section>
<style>
.masonry {
  display: grid-lanes;
  grid-template-columns: repeat(5, auto);
}
.item { height: 50px; }
.item.wide { grid-column: span 3; }
.item.tall { height: 90px; }
</style>
```

<figure>
<img src="https://drafts.csswg.org/css-grid-3/images/masonry-reorder-span.png" alt="In this example, the first row is items 1, 2, 3, 5, 6,&#10;&#9;&#9;&#9;          with item 3 slightly taller than the others.&#10;&#9;&#9;&#9;          Item 4 spans the first three columns, and is placed&#10;&#9;&#9;&#9;          just below item 3, while item 7 is tucked under item 5.">
<figcaption>Auto-placed grid lanes layout with mixed-height items and mixed span sizes</figcaption>
</figure>

Similarly, items explicitly placed into specific tracks can leave gaps behind them, into which subsequent auto-placed items can be placed visually out-of-order.

Authors should be aware of these possibilities and design layouts where such backtracking is minimized so that focus and reading order can be more easily followed. Alternatively, if the items do not have an inherent order, use the <a id="ref-for-propdef-reading-flow"></a>[reading-flow](https://drafts.csswg.org/css-display-4/#propdef-reading-flow) property to allow the UA to re-order the items for reading and linear navigation.

<a id="issue-f3901e05"></a>

<strong>Issue:</strong>

[](#issue-f3901e05) Or should reordering be the default behavior for auto-placed items here?

Techniques for reducing backtracking include:

<strong>Note:</strong>

- Using appropriate values for <a id="ref-for-propdef-fit-tolerance①"></a>[fit-tolerance](#propdef-fit-tolerance), i.e. values large enough to avoid gratuitous differentiation among similarly-sized tracks, but not so large that meaningful differences get ignored.

- Using explicit placement in ways that help group related items together, rather than ways that disrupt the natural order of items

- Avoiding the combination of mixed span sizes in the <a id="ref-for-grid-axis①"></a>[grid axis](#grid-axis) and disparate item sizes in the <a id="ref-for-stacking-axis③"></a>[stacking axis](#stacking-axis), which can cause items to get pulled out of order (see example above).

As with <a id="ref-for-grid-layout⑤"></a>[grid layout](https://drafts.csswg.org/css-grid-2/#grid-layout) and <a id="ref-for-flex-layout③"></a>[flex layout](https://drafts.csswg.org/css-flexbox-2/#flex-layout) authors can use the <a id="ref-for-propdef-order①"></a>[order](https://drafts.csswg.org/css-display-4/#propdef-order) property to re-order items; the same caveats apply. See [CSS Grid Layout 2 § 4 Reordering and Accessibility](https://drafts.csswg.org/css-grid-2/#order-accessibility) and [CSS Display 4 § 3.1 Reordering and Accessibility](https://drafts.csswg.org/css-display-4/#order-accessibility).

### <a id="grid-lanes-containers"></a>2.2.  Establishing Grid Lanes Layout[](#grid-lanes-containers)

| Field                                                                        | Definition                                                                                                      |
|------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                       | <a id="ref-for-propdef-display①"></a>[display](https://drafts.csswg.org/css-display-4/#propdef-display)         |
| <strong>[New values:](https://www.w3.org/TR/css-values/#value-defs)</strong> | grid-lanes <a id="ref-for-comb-one"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) inline-grid-lanes |

<strong><a id="valdef-display-grid-lanes"></a><strong>grid-lanes</strong></strong>

This value causes an element to generate a <a id="ref-for-block-level"></a>[block-level](https://drafts.csswg.org/css-display-4/#block-level) <a id="ref-for-grid-lanes-container①"></a>[grid lanes container](#grid-lanes-container) box.

<strong><a id="valdef-display-inline-grid-lanes"></a><strong>inline-grid-lanes</strong></strong>

This value causes an element to generate an <a id="ref-for-inline-level"></a>[inline-level](https://drafts.csswg.org/css-display-4/#inline-level) <a id="ref-for-grid-lanes-container②"></a>[grid lanes container](#grid-lanes-container) box.

A <a id="ref-for-grid-lanes-container③"></a>[grid lanes container](#grid-lanes-container) that is not <a id="ref-for-subgrid"></a>[subgridded](https://drafts.csswg.org/css-grid-2/#subgrid) in its <a id="ref-for-grid-axis②"></a>[grid axis](#grid-axis) establishes an <a id="ref-for-independent-formatting-context"></a>[independent formatting context](https://drafts.csswg.org/css-display-4/#independent-formatting-context) for its contents.

<a id="issue-3be4bca6"></a>

<strong>Issue:</strong>

[](#issue-3be4bca6) ISSUE(12820): Write up how grid/masonry formatting contexts work more formally?

### <a id="grid-lanes-orientation"></a>2.3.  Orienting Grid Lanes Layout[](#grid-lanes-orientation)

The orientation of a <a id="ref-for-grid-lanes-container④"></a>[grid lanes container](#grid-lanes-container), i.e. whether its <a id="ref-for-grid-axis③"></a>[grid axis](#grid-axis) is the <a id="ref-for-inline-axis①"></a>[inline axis](https://drafts.csswg.org/css-writing-modes-4/#inline-axis) (establishing columns) or the <a id="ref-for-block-axis①"></a>[block axis](https://drafts.csswg.org/css-writing-modes-4/#block-axis) (establishing rows) is determined by the <a id="issue-d4908b1e"></a>

<strong>Issue:</strong>

[](#issue-d4908b1e)TBD property.

The <a id="ref-for-initial-value"></a>[initial value](https://drafts.csswg.org/css-cascade-5/#initial-value) of this property is <a id="valdef-grid-auto-flow-normal"></a><strong>normal</strong>, which determines the orientation from the grid-template-\* properties:

- If <a id="ref-for-propdef-grid-template-columns①"></a>[grid-template-columns](https://drafts.csswg.org/css-grid-2/#propdef-grid-template-columns) is <a id="ref-for-valdef-grid-template-rows-none"></a>[none](https://drafts.csswg.org/css-grid-2/#valdef-grid-template-rows-none) and <a id="ref-for-propdef-grid-template-rows"></a>[grid-template-rows](https://drafts.csswg.org/css-grid-2/#propdef-grid-template-rows) is not <a id="ref-for-valdef-grid-template-rows-none①"></a>none, the <a id="ref-for-grid-container③"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container)’s <a id="ref-for-block-axis②"></a>[block axis](https://drafts.csswg.org/css-writing-modes-4/#block-axis) is the <a id="ref-for-grid-axis④"></a>[grid axis](#grid-axis) (establishing rows).

- Otherwise (thus by default), the <a id="ref-for-grid-container④"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container)’s <a id="ref-for-inline-axis②"></a>[inline axis](https://drafts.csswg.org/css-writing-modes-4/#inline-axis) is the <a id="ref-for-grid-axis⑤"></a>[grid axis](#grid-axis) (establishing columns).

<a id="example-5742456b"></a>

<strong>Example:</strong>

[](#example-5742456b) The following code will create a 2-column (“waterfall style”) <a id="ref-for-grid-lanes-container⑤"></a>[grid lanes container](#grid-lanes-container) that grows downward:

``` text
.container {
  display: grid-lanes;
  grid-template-columns: 100px 200px;
}
```

while the following code will create a 2-row (“brick wall style”) <a id="ref-for-grid-lanes-container⑥"></a>[grid lanes container](#grid-lanes-container) that grows horizontally:

``` text
.container {
  display: grid-lanes;
  grid-template-rows: 100px 200px;
}
```

<a id="issue-ff7d700c"></a>

<strong>Issue:</strong>

[](#issue-ff7d700c) Figure out whether we are re-using <a id="ref-for-propdef-grid-auto-flow"></a>[grid-auto-flow](https://drafts.csswg.org/css-grid-2/#propdef-grid-auto-flow) here (and what it’s values mean) or defining a new property like grid-lanes-direction. [\[Issue \#12803\]](https://github.com/w3c/csswg-drafts/issues/12803)

## <a id="grid-lanes-track-templates"></a>3. <a id="masonry-track-templates"></a> Grid Lanes Track Specification[](#grid-lanes-track-templates)

In the <a id="ref-for-grid-axis⑥"></a>[grid axis](#grid-axis), the full power of <a id="ref-for-grid-layout⑥"></a>[grid layout](https://drafts.csswg.org/css-grid-2/#grid-layout) is available for track specification:

- Track sizes, line names, and areas can be specified on the <a id="ref-for-grid-lanes-container⑦"></a>[grid lanes container](#grid-lanes-container)’s <a id="ref-for-grid-axis⑦"></a>[grid axis](#grid-axis), just like in <a id="ref-for-grid-layout⑦"></a>[grid layout](https://drafts.csswg.org/css-grid-2/#grid-layout).

- The <a id="ref-for-explicit-grid"></a>[explicit grid](https://drafts.csswg.org/css-grid-2/#explicit-grid) and <a id="ref-for-implicit-grid"></a>[implicit grid](https://drafts.csswg.org/css-grid-2/#implicit-grid) are formed in the same way as for a regular <a id="ref-for-grid-container⑤"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container).

- Items can be [placed](#grid-lanes-track-placement) against these grid templates just as in <a id="ref-for-grid-layout⑧"></a>[grid layout](https://drafts.csswg.org/css-grid-2/#grid-layout).

However, auto-placed items contribute sizing to all tracks, not just the track into which they are ultimately placed; see [§ 3.4 Grid Axis Track Sizing](#track-sizing).

Note: This is because auto-placed items must be laid out <em>as</em> they are placed, so that each track knows how “full” it is (and therefore which track should receive the next auto-placed item); thus, the tracks themselves must already have a definite size so that the items know their <a id="ref-for-available"></a>[available space](https://drafts.csswg.org/css-sizing-3/#available) during layout.

### <a id="grid-lanes-track-properties"></a>3.1.  Declaring Grid Lanes Track Templates: the grid-template-\* properties[](#grid-lanes-track-properties)

The grid-template-\* and <a id="ref-for-propdef-grid-auto-rows"></a>[grid-auto-rows](https://drafts.csswg.org/css-grid-2/#propdef-grid-auto-rows)/<a id="ref-for-propdef-grid-auto-columns"></a>[grid-auto-columns](https://drafts.csswg.org/css-grid-2/#propdef-grid-auto-columns) properties (and their shorthands) apply in the <a id="ref-for-grid-axis⑧"></a>[grid axis](#grid-axis) of the <a id="ref-for-grid-lanes-container⑧"></a>[grid lanes container](#grid-lanes-container) and establish tracks just as on regular <a id="ref-for-grid-container⑥"></a>[grid containers](https://drafts.csswg.org/css-grid-2/#grid-container). (They are ignored in the <a id="ref-for-stacking-axis④"></a>[stacking axis](#stacking-axis).)

#### <a id="intrinsic-auto-repeat"></a>3.1.1. <a id="masonry-intrinsic-repeat"></a> Intrinsic Tracks and repeat()[](#intrinsic-auto-repeat)

Level 3 extends the <a id="ref-for-funcdef-repeat-line-color-repeat"></a>[repeat()](https://drafts.csswg.org/css-gaps-1/#funcdef-repeat-line-color-repeat) notation to allow repetitions where neither the <a id="ref-for-min-track-sizing-function"></a>[min track sizing function](https://drafts.csswg.org/css-grid-2/#min-track-sizing-function) nor the <a id="ref-for-max-track-sizing-function"></a>[max track sizing function](https://drafts.csswg.org/css-grid-2/#max-track-sizing-function) is <a id="ref-for-definite"></a>[definite](https://drafts.csswg.org/css-sizing-3/#definite); in other words, the syntax for <a id="ref-for-typedef-auto-repeat"></a>[\<auto-repeat\>](#typedef-auto-repeat) is relaxed to the following:

<a id="typedef-auto-repeat"></a><a id="ref-for-typedef-auto-repeat①"></a><a id="ref-for-comb-one①"></a><a id="ref-for-comb-comma"></a><a id="ref-for-typedef-line-names"></a><a id="ref-for-mult-opt"></a><a id="ref-for-typedef-track-size"></a><a id="ref-for-mult-one-plus"></a><a id="ref-for-typedef-line-names①"></a><a id="ref-for-mult-opt①"></a>

``` text
<auto-repeat> = repeat(  [ auto-fill | auto-fit ] , [ <line-names>? <track-size> ]+ <line-names>? )
```

In order to resolve the number of repetitions, a hypothetical size is calculated for such tracks by initializing the track sizes (per [CSS Grid Layout 2 § 12.4 Initialize Track Sizes](https://drafts.csswg.org/css-grid-2/#algo-init)) and resolving intrinsic track sizes (per [CSS Grid Layout 2 § 12.5 Resolve Intrinsic Track Sizes](https://drafts.csswg.org/css-grid-2/#algo-content)) in accordance with [§ 3.4 Grid Axis Track Sizing](#track-sizing) with the following assumptions:

- Ignore explicit item placement. (That is, assume all items have an <a id="ref-for-automatic-grid-position"></a>[automatic position](https://drafts.csswg.org/css-grid-2/#automatic-grid-position).)

- Do not <a id="ref-for-collapsed-grid-track"></a>[collapse](https://drafts.csswg.org/css-grid-2/#collapsed-grid-track) any tracks.

- Expand the repeated track listing to capture all possible <a id="ref-for-auto-placement①"></a>[automatic placements](https://drafts.csswg.org/css-grid-2/#auto-placement) of each item, i.e. repeat the track listing <code>2 + (<var>largest span</var> - 2)/(<var>number of tracks in repeat()</var>)</code> times, rounded <em>down</em> to a whole number.

The hypothetical size of each track in the <a id="ref-for-funcdef-repeat-line-color-repeat①"></a>[repeat()](https://drafts.csswg.org/css-gaps-1/#funcdef-repeat-line-color-repeat) listing is given by the largest track corresponding to that entry (by index) in the listing.

<a id="example-000330f4"></a>

<strong>Example:</strong>

[](#example-000330f4) For example, given a template of repeat(auto-fill, 50px auto auto) and a largest spanner of 2, you need to repeat the track listing twice, giving 50px auto auto 50px auto auto. After doing the hypothetical layout, the 2nd and 5th tracks are maxed together to provide a hypothetical size for the first auto and the 3rd and 6th tracks are maxed together to provide a hypothetical size for the second auto. Those concrete sizes are then used to determine how many repetitions will fill the container.

<a id="issue-c70906df"></a>

<strong>Issue:</strong>

[](#issue-c70906df) Should this work also in Grid Layout somehow, or fall back to a single repetition? If so, how? [\[Issue \#10915\]](https://github.com/w3c/csswg-drafts/issues/10915)

Note: This simplified layout heuristic is defined to be "good enough", while remaining fast and consistent. Ignoring placement is required just to make the concept coherent; before you know how many repetitions you need, you can’t tell what track an item (even one with a definite placement) will end up in.

<a id="issue-5b45728c"></a>

<strong>Issue:</strong>

[](#issue-5b45728c) Technically, if your explicit placement does not enter or cross over the repetition, we can know its placement prior to resolving the reptition. Do we want to adjust the algorithm above to account for this? That is, given auto repeat(auto-fill, ...) auto, if you explicitly place an item against line 1 or -1, we could let it only affect that track (rather than pretending it’ll go in every track).

### <a id="subgrids"></a>3.2.  Subgrids[](#subgrids)

<a id="ref-for-subgrid①"></a>[Subgridding](https://drafts.csswg.org/css-grid-2/#subgrid) allows nested <a id="ref-for-grid-lanes-container⑨"></a>[grid lanes containers](#grid-lanes-container) (and <a id="ref-for-grid-container⑦"></a>[grid containers](https://drafts.csswg.org/css-grid-2/#grid-container)) to share track sizes. If the parent’s corresponding axis is a <a id="ref-for-grid-axis⑨"></a>[grid axis](#grid-axis), the subgridded axis is taken from the parent container [as specified for grid containers](https://drafts.csswg.org/css-grid-2/#subgrids); if the parent’s corresponding axis is a <a id="ref-for-stacking-axis⑤"></a>[stacking axis](#stacking-axis), the subgridded axis also acts as a <a id="ref-for-stacking-axis⑥"></a>stacking axis.

<a id="issue-ff7026b1"></a>

<strong>Issue:</strong>

[](#issue-ff7026b1) What if this conflicts with the lanes orientation, or results in both axes stacking?

In <a id="ref-for-grid-lanes-layout⑦"></a>[grid lanes layout](#grid-lanes-layout), auto-placed <a id="ref-for-subgrid②"></a>[subgrids](https://drafts.csswg.org/css-grid-2/#subgrid) don’t inherit any line names from their parent grid, because that would make the placement of the item dependent on layout results; but the subgrid’s tracks are still aligned to the parent’s tracks as usual.

<a id="example-be8494db"></a>

<strong>Example:</strong>

[](#example-be8494db) Here’s a subgrid [example](https://drafts.csswg.org/css-grid-3/examples/subgrid-example-1.html):

``` text
<style>
.grid {
  display: inline-grid-lanes;
  grid-template-rows: auto auto 100px;
  align-content: center;
  height: 300px;
  border: 1px solid;
}

.grid > * {
  margin: 5px;
  background: silver;
}
.grid > :nth-child(2n) {
  background: pink;
}

.grid subgrid {
  display: grid;
  grid: subgrid / subgrid;
  grid-row: 2 / span 2;
  grid-gap: 30px;
}
.grid subgrid > * { background: cyan; }
</style>
```

``` text
<div class="grid">
  <item>1</item>
  <item>2</item>
  <item>3</item>
  <subgrid>
    <item style="height:100px">subgrid.1</item>
    <item>sub.2</item>
    <item>s.3</item>
  </subgrid>
  <item>4</item>
  <item>5</item>
  <item style="width: 80px">6</item>
  <item>7</item>
</div>
```

<figure>
<img src="https://drafts.csswg.org/css-grid-3/images/subgrid-example-1.png" />
<figcaption>The rendering of the subgrid example above.</figcaption>
</figure>

Note how the subgrid’s first item ("subgrid.1") contributes to the intrinsic size of the 2nd row in the parent grid. This is possible since the subgrid specified a definite position so we know which tracks it will occupy. Note also that trying to subgrid the parent’s <a id="ref-for-stacking-axis⑦"></a>[stacking axis](#stacking-axis) results in the subgrid converting to a <a id="ref-for-grid-lanes-container①⓪"></a>[grid lanes container](#grid-lanes-container) with its <a id="ref-for-inline-axis③"></a>[inline axis](https://drafts.csswg.org/css-writing-modes-4/#inline-axis) as the <a id="ref-for-stacking-axis⑧"></a>stacking axis.

A <a id="ref-for-subgrid③"></a>[subgrid](https://drafts.csswg.org/css-grid-2/#subgrid) that is a <a id="ref-for-grid-lanes-container①①"></a>[grid lanes container](#grid-lanes-container) can be referred to as a <a id="grid-lanes-subgrid"></a><strong>grid lanes subgrid</strong>. To distinguish, a regular <a id="ref-for-grid-container⑧"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container) <a id="ref-for-subgrid④"></a>subgrid can be referred to as a <a id="regular-grid-subgrid"></a><strong>regular grid subgrid</strong>.

### <a id="repeat-notation"></a>3.3.  Track Repetition: the <a id="ref-for-funcdef-repeat-line-color-repeat②"></a>[repeat()](https://drafts.csswg.org/css-gaps-1/#funcdef-repeat-line-color-repeat) notation[](#repeat-notation)

This specification introduces new keywords and grid-lanes–specific behavior for the <a id="ref-for-funcdef-repeat-line-color-repeat③"></a>[repeat()](https://drafts.csswg.org/css-gaps-1/#funcdef-repeat-line-color-repeat) notation.

#### <a id="repeat-auto-fit"></a>3.3.1.  repeat(auto-fit)[](#repeat-auto-fit)

In <a id="ref-for-grid-lanes-container①②"></a>[grid lanes containers](#grid-lanes-container) (as in regular <a id="ref-for-grid-container⑨"></a>[grid containers](https://drafts.csswg.org/css-grid-2/#grid-container)) <a id="ref-for-valdef-repeat-auto-fit"></a>[auto-fit](https://drafts.csswg.org/css-grid-2/#valdef-repeat-auto-fit) acts like <a id="ref-for-valdef-repeat-auto-fill"></a>[auto-fill](https://drafts.csswg.org/css-grid-2/#valdef-repeat-auto-fill), but with empty tracks <a id="ref-for-collapsed-grid-track①"></a>[collapsed](https://drafts.csswg.org/css-grid-2/#collapsed-grid-track). However, because placement occurs after track sizing, <a id="ref-for-grid-lanes-container①③"></a>grid lanes containers use a heuristic to determine if a track will be occupied:

- All tracks occupied by explicitly placed items are considered occupied.

- With the sum of the spans of all auto-placed items as <var>N</var>, all unoccupied tracks up to the <var>N</var>th such track are considered occupied.

All tracks produced by the <a id="ref-for-valdef-repeat-auto-fit①"></a>[auto-fit](https://drafts.csswg.org/css-grid-2/#valdef-repeat-auto-fit) repetition and considered unoccupied by this heuristic are assumed “empty” and are <a id="ref-for-collapsed-grid-track②"></a>[collapsed](https://drafts.csswg.org/css-grid-2/#collapsed-grid-track). A <a id="ref-for-collapsed-grid-track③"></a>collapsed grid track cannot accept placement of auto-placed items.

Note: It is possible for an auto-placed item to be placed in a track when <a id="ref-for-valdef-repeat-auto-fill①"></a>[auto-fill](https://drafts.csswg.org/css-grid-2/#valdef-repeat-auto-fill) is used that would be collapsed if <a id="ref-for-valdef-repeat-auto-fit②"></a>[auto-fit](https://drafts.csswg.org/css-grid-2/#valdef-repeat-auto-fit) is used if there are auto-placed items with a span greater than 1 mixed with explicitly-placed items that leave gaps too small for the auto-placed items.

### <a id="track-sizing"></a>3.4.  Grid Axis Track Sizing[](#track-sizing)

Track sizing works the same as in [CSS Grid](https://drafts.csswg.org/css-grid-2/#algo-track-sizing), except that when considering which items contribute to intrinsic sizes:

- All items explicitly placed in that track contribute, and

- All items with an <a id="ref-for-automatic-grid-position①"></a>[automatic grid position](https://drafts.csswg.org/css-grid-2/#automatic-grid-position) contribute (regardless of whether they are ultimately placed in that track).

<a id="example-6d22f73d"></a>

<strong>Example:</strong>

[](#example-6d22f73d) For example, suppose there are two columns in the <a id="ref-for-grid-axis①⓪"></a>[grid axis](#grid-axis) and that

- Items A, B, and C have no explicit position.

- Item D is explicitly placed into the first column.

In this case, items A, B, C, and D all contribute to sizing the first column, while only A, B, and C (and not D) contribute to the second column.

In the case of spanning items with an <a id="ref-for-automatic-grid-position②"></a>[automatic grid position](https://drafts.csswg.org/css-grid-2/#automatic-grid-position), they are assumed to be placed at every possible start position, and contribute accordingly.

<a id="example-f3e866aa"></a>

<strong>Example:</strong>

[](#example-f3e866aa) For example, suppose there are 5 columns in the <a id="ref-for-grid-axis①①"></a>[grid axis](#grid-axis), with the middle having a fixed size of 100px and the other two being auto-sized. For the purpose of track sizing, an item that spans 2 tracks and has an intrinsic contribution of 220px is essentially copied and assumed to exist:

- At grid line 1, contributing 110px to each of the first two tracks.

- At grid line 2, contributing 120px to the second track.

- At grid line 3, contributing 120px to the fourth track.

- At grid line 4, contributing 110px to the fourth and fifth tracks.

Note: This algorithm ensures that each track is at least big enough to accommodate every item that is ultimately placed in it, and does not create dependency cycles between placement and track sizing. However, depending on the variation in sizes, tracks could be larger than necessary: an exact fit is only guaranteed if all items are explicitly placed in the <a id="ref-for-grid-axis①②"></a>[grid axis](#grid-axis) or all items are the same size (or matching multiples of that size, in the case of spanning items).

#### <a id="track-sizing-subgrid"></a>3.4.1.  Subgrid Item Contributions[](#track-sizing-subgrid)

When sizing the tracks of either a regular <a id="ref-for-grid-container①⓪"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container) or a <a id="ref-for-grid-lanes-container①④"></a>[grid lanes container](#grid-lanes-container), a <a id="ref-for-grid-lanes-subgrid"></a>[grid lanes subgrid](#grid-lanes-subgrid) has special handling of items that have an <a id="ref-for-automatic-grid-position③"></a>[automatic grid position](https://drafts.csswg.org/css-grid-2/#automatic-grid-position):

- Any such item is placed into every possible <a id="ref-for-parent-grid"></a>[parent grid](https://drafts.csswg.org/css-grid-2/#parent-grid) track that could be spanned by the <a id="ref-for-grid-lanes-subgrid①"></a>[grid lanes subgrid](#grid-lanes-subgrid). (If the subgrid has a <a id="ref-for-definite-grid-position①"></a>[definite grid position](https://drafts.csswg.org/css-grid-2/#definite-grid-position), thus only the spanned tracks; if it has an <a id="ref-for-automatic-grid-position④"></a>[automatic grid position](https://drafts.csswg.org/css-grid-2/#automatic-grid-position), then all tracks in the parent grid.)

- Any such item receives the largest margin/border/padding contribution of each edge at which it could hypothetically be placed. If the item spans the entire subgrid, it receives both. (See [CSS Grid Layout §9](https://www.w3.org/TR/css-grid-2/#subgrid-item-contribution).)

When sizing the tracks of a <a id="ref-for-grid-lanes-container①⑤"></a>[grid lanes container](#grid-lanes-container), a <a id="ref-for-subgrid⑤"></a>[subgrid](https://drafts.csswg.org/css-grid-2/#subgrid) with an <a id="ref-for-automatic-grid-position⑤"></a>[automatic grid position](https://drafts.csswg.org/css-grid-2/#automatic-grid-position) (whether a <a id="ref-for-regular-grid-subgrid"></a>[regular grid subgrid](#regular-grid-subgrid) or <a id="ref-for-grid-lanes-subgrid②"></a>[grid lanes subgrid](#grid-lanes-subgrid)) also has special handling of its items:

- <em>Every</em> item is placed into every possible <a id="ref-for-parent-grid①"></a>[parent grid](https://drafts.csswg.org/css-grid-2/#parent-grid) track that could be spanned by the <a id="ref-for-subgrid⑥"></a>[subgrid](https://drafts.csswg.org/css-grid-2/#subgrid), (ignoring any explicit placement of the item).

- Explicitly-placed sub-items only receive the subgrid’s margin/border/padding contribution when their explicit placement is at an edge of the subgrid (as usual).

Note: Explicitly placed items inside an explicitly placed <a id="ref-for-subgrid⑦"></a>[subgrid](https://drafts.csswg.org/css-grid-2/#subgrid) only ever contribute to their actual <a id="ref-for-parent-grid②"></a>[parent grid](https://drafts.csswg.org/css-grid-2/#parent-grid) tracks. This is because in this case, just like in regular <a id="ref-for-grid-layout⑨"></a>[grid layout](https://drafts.csswg.org/css-grid-2/#grid-layout), the item’s placement is known prior to track sizing.

#### <a id="track-sizing-performance"></a>3.4.2.  Optimized Track Sizing[](#track-sizing-performance)

Track sizing can be optimized by aggregating items that have the same span size and placement into a single virtual item as follows:

1.  Separate all the <a id="ref-for-grid-item①"></a>[grid items](https://drafts.csswg.org/css-grid-2/#grid-item) into <a id="item-groups"></a><strong>item groups</strong>, according to the following properties:

    - the span of the item

    - the placement of the item, i.e. which tracks it is allowed to be placed in

    - the item’s <a id="ref-for-baseline-sharing-group"></a>[baseline-sharing group](https://drafts.csswg.org/css-align-3/#baseline-sharing-group)

    Note: For example, an item with span 2 placed in the second track will be in a different group than an item with span 2 that has an <a id="ref-for-automatic-grid-position⑥"></a>[automatic grid position](https://drafts.csswg.org/css-grid-2/#automatic-grid-position).

2.  For each <a id="ref-for-item-groups"></a>[item group](#item-groups), synthesize a <a id="virtual-grid-item"></a><strong>virtual grid item</strong> that has the maximum of every intrinsic size contribution among the items in that group.

    If the items apply <a id="ref-for-baseline-alignment"></a>[baseline alignment](https://drafts.csswg.org/css-align-3/#baseline-alignment), determine the baselines of the <a id="ref-for-virtual-grid-item"></a>[virtual grid item](#virtual-grid-item) by placing all of its items into a single hypothetical grid track and finding their shared baseline(s) and shims. Increase the group’s intrinsic size contributions accordingly.

3.  Place hypothetical copies of each <a id="ref-for-virtual-grid-item①"></a>[virtual grid item](#virtual-grid-item) into the <a id="ref-for-grid-axis①③"></a>[grid axis](#grid-axis) tracks in every position that the item could potentially occupy, and run the [track sizing algorithm](https://drafts.csswg.org/css-grid-2/#algo-track-sizing) with those items. The resulting track sizes are the <a id="ref-for-grid-lanes-container①⑥"></a>[grid lanes container’s](#grid-lanes-container) track sizes.

Note: This optimization should give the same results as the track sizing description [above](#track-sizing); if not this is an error, please [report it to the CSSWG](https://github.com/w3c/csswg-drafts/issues).

## <a id="grid-lanes-track-placement"></a>4. <a id="masonry-track-placement"></a> Grid Lanes Item Placement[](#grid-lanes-track-placement)

In the <a id="ref-for-grid-axis①④"></a>[grid axis](#grid-axis), items can be <em>explicitly placed</em> into tracks and span them using the familiar <a id="ref-for-grid-placement-property"></a>[grid-placement properties](https://drafts.csswg.org/css-grid-2/#grid-placement-property)’ syntax. Auto-placement, however, uses the [§ 4.4 Grid Lanes Layout and Placement Algorithm](#grid-lanes-layout-algorithm), placing each item with an <a id="ref-for-automatic-grid-position⑦"></a>[automatic grid position](https://drafts.csswg.org/css-grid-2/#automatic-grid-position) into the “shortest” track available.

<a id="example-c504d9b3"></a>

<strong>Example:</strong>

[](#example-c504d9b3) Here’s a grid lanes layout demonstrating explicitly placed and spanning items:

``` text
.container {
  grid-template-columns: repeat(3, auto);
}
.container > :nth-child(2) {
  /* auto-placed, but spanning. */
  grid-column: span 2;
}
.container > :nth-child(3) {
  /* manually placed */
  grid-column: 3;
}
/* all other children are auto-placed */
```

<figure>
<img src="https://drafts.csswg.org/css-grid-3/images/example-span-and-manual.png" />
<figcaption>Rendering of the example above.</figcaption>
</figure>

### <a id="grid-lanes-placement"></a>4.1.  Specifying Grid Axis Item Placement: the grid-column-\* and grid-row-\* properties[](#grid-lanes-placement)

The grid-column-\* and grid-row-\* properties (and their shorthands) apply in the <a id="ref-for-grid-axis①⑤"></a>[grid axis](#grid-axis) of the items and establish placement just as in regular <a id="ref-for-grid-layout①⓪"></a>[grid layout](https://drafts.csswg.org/css-grid-2/#grid-layout).

### <a id="placement-tolerance"></a>4.2. <a id="item-slack"></a> Placement Precision: the <a id="ref-for-propdef-fit-tolerance②"></a>[fit-tolerance](#propdef-fit-tolerance) property[](#placement-tolerance)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                           |
|------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-fit-tolerance"></a><strong>fit-tolerance</strong>                                                                                                                                                                                                                     |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | normal <a id="ref-for-comb-one②"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) <a id="ref-for-typedef-length-percentage"></a>[\<length-percentage \[0,∞\]\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) <a id="ref-for-comb-one③"></a>\| infinite |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | normal                                                                                                                                                                                                                                                                               |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | <a id="ref-for-grid-lanes-container①⑦"></a>[grid lanes containers](#grid-lanes-container)                                                                                                                                                                                            |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                                                                                   |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | relative to the <a id="ref-for-grid-axis①⑥"></a>[grid-axis](#grid-axis) <a id="ref-for-content-box"></a>[content box](https://drafts.csswg.org/css-box-4/#content-box) size of the <a id="ref-for-grid-lanes-container①⑧"></a>[grid lanes container](#grid-lanes-container)          |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | a computed <a id="ref-for-typedef-length-percentage①"></a>[\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage) value                                                                                                                            |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                          |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | as length                                                                                                                                                                                                                                                                            |

<a id="ref-for-grid-lanes-container①⑨"></a>[Grid lanes containers](#grid-lanes-container) are filled by placing each <a id="ref-for-grid-item②"></a>[grid item](https://drafts.csswg.org/css-grid-2/#grid-item) in whichever <a id="ref-for-grid-track②"></a>[grid track](https://drafts.csswg.org/css-grid-2/#grid-track) is currently the least filled. When multiple tracks are tied for least-filled, placing the items in order looks good. But if tracks are only <em>very slightly</em> different heights, it can look strange to have them not fill in order, as the height differences aren’t perceived as <em>meaningfully</em> different.

The <a id="ref-for-propdef-fit-tolerance③"></a>[fit-tolerance](#propdef-fit-tolerance) property specifies what the threshold is for considering tracks to be “the same height”, causing them to fill in order.

<strong><a id="valdef-fit-tolerance-length-percentage"></a><strong><a id="ref-for-typedef-length-percentage②"></a>[\<length-percentage\>](https://drafts.csswg.org/css-values-4/#typedef-length-percentage)</strong></strong>

Specifies the <a id="grid-lanes-tie-threshold"></a><strong>tie threshold</strong> for the <a id="ref-for-grid-lanes-container②⓪"></a>[grid lanes container](#grid-lanes-container). Placement positions are considered to be equally good (“tied”) if they are within the specified distance from the shortest position.

Note: The initial value is a “small” distance (1em) that is probably appropriate to represent “close enough”.

<strong><a id="valdef-fit-tolerance-normal"></a><strong>normal</strong></strong>

Resolves to a <a id="ref-for-used-value"></a>[used value](https://drafts.csswg.org/css-cascade-5/#used-value) of 1em in <a id="ref-for-grid-lanes-layout⑧"></a>[grid lanes layout](#grid-lanes-layout) and a <a id="ref-for-used-value①"></a>used value of 0 in all other layout modes.

<strong><a id="valdef-fit-tolerance-infinite"></a><strong>infinite</strong></strong>

Specifies an infinite <a id="ref-for-grid-lanes-tie-threshold"></a>[tie threshold](#grid-lanes-tie-threshold). This makes items distribute themselves strictly in order, without considering the length of the tracks at all.

Note: This value can result in consecutive items being placed in dramatically different positions in the <a id="ref-for-stacking-axis⑨"></a>[stacking axis](#stacking-axis), which can be confusing to readers. If the initial value (\`1em\`) is too small, consider a larger value (such as \`10em\` or \`50vh\`) instead of \`infinite\`.

<a id="example-ded661dc"></a>

<strong>Example:</strong>

[](#example-ded661dc) In the following example, when placing the 5th item, the fourth column is the shortest, but the first column is <em>almost</em> as short.

<figure>
<img src="https://drafts.csswg.org/css-grid-3/images/tolerance.png" alt="An example grid lanes element with four columns.&#10;&#9;&#9;&#9;&#9;      Each column is already partially filled to different heights,&#10;&#9;&#9;&#9;&#9;      with the fourth column the shortest but the first column only slightly taller.&#10;&#9;&#9;&#9;&#9;      Depending on the tolerance value, the next item to be placed can choose either the first or fourth column.">
</figure>

With the default tolerance of 1em, both the first and fourth columns are considered to be "tied", and so the first is chosen.

If, instead, the tolerance is set to 0px, then there is no tie; only the fourth column is a possible placement.

Note: We expect to apply this property to <a id="ref-for-flex-layout④"></a>[flex layout](https://drafts.csswg.org/css-flexbox-2/#flex-layout) in the future also, see [discussions](https://github.com/w3c/csswg-drafts/issues/3071) on how that might work.

### <a id="grid-lanes-dense-packing"></a>4.3.  Dense Placement: the <a id="ref-for-valdef-grid-auto-flow-dense"></a>[dense](https://drafts.csswg.org/css-grid-2/#valdef-grid-auto-flow-dense) keyword[](#grid-lanes-dense-packing)

In <a id="ref-for-grid-layout①①"></a>[grid layout](https://drafts.csswg.org/css-grid-2/#grid-layout), <a id="ref-for-propdef-grid-auto-flow①"></a>[grid-auto-flow: dense](https://drafts.csswg.org/css-grid-2/#propdef-grid-auto-flow) allows backtracking during the [grid item placement algorithm](https://drafts.csswg.org/css-grid-2/#auto-placement-algo). The <a id="ref-for-valdef-grid-auto-flow-dense①"></a>[dense](https://drafts.csswg.org/css-grid-2/#valdef-grid-auto-flow-dense) keyword similarly allows backtracking during the [grid lanes placement algorithm](#grid-lanes-layout-algorithm). However, because item placement and sizing are intertwined in <a id="ref-for-grid-lanes-layout⑨"></a>[grid lanes layout](#grid-lanes-layout), an item can only backtrack into a compatible empty slot if the total used size of that slot’s tracks matches the used size of the item’s normal-placement tracks.

Note: This restriction avoids laying out the item more than once.

### <a id="grid-lanes-layout-algorithm"></a>4.4.  Grid Lanes Layout and Placement Algorithm[](#grid-lanes-layout-algorithm)

For each of the tracks in the <a id="ref-for-grid-axis①⑦"></a>[grid axis](#grid-axis), keep a <a id="running-position"></a><strong>running position</strong> initialized to zero. Maintain also a <a id="auto-placement-cursor"></a><strong>auto-placement cursor</strong>, initially pointing to the first line.

For each item in <a id="ref-for-order-modified-document-order"></a>[order-modified document order](https://drafts.csswg.org/css-display-4/#order-modified-document-order):

1.  If the item has a <a id="ref-for-definite-grid-position②"></a>[definite grid position](https://drafts.csswg.org/css-grid-2/#definite-grid-position) in the <a id="ref-for-grid-axis①⑧"></a>[grid axis](#grid-axis), use that placement.

    <a id="issue-9c647ef6"></a>

    <strong>Issue:</strong>

    [](#issue-9c647ef6) Should this also update the placement cursor?

    Otherwise, resolve its <a id="ref-for-grid-axis①⑨"></a>[grid axis](#grid-axis) placement using these substeps:

    1.  Starting at the first <a id="ref-for-grid-axis②⓪"></a>[grid axis](#grid-axis) line in the <a id="ref-for-implicit-grid①"></a>[implicit grid](https://drafts.csswg.org/css-grid-2/#implicit-grid), find the largest <a id="ref-for-running-position"></a>[running position](#running-position) of the <a id="ref-for-grid-axis②①"></a>grid axis tracks that the item would span if it were placed at this line, and call this position <var>max_pos</var>.
    2.  Repeat the previous step for each successive line number until the item would no longer fit inside the grid.
    3.  Let <var>possible lines</var> be the line that resulted in the smallest <var>max_pos</var>, and all lines that result in a <var>max_pos</var> within the <a id="ref-for-grid-lanes-tie-threshold①"></a>[tie threshold](#grid-lanes-tie-threshold) of this <var>max_pos</var>.
    4.  Choose the first line in <var>possible lines</var> greater than or equal to the <a id="ref-for-auto-placement-cursor"></a>[auto-placement cursor](#auto-placement-cursor) as the item’s position in the <a id="ref-for-grid-axis②②"></a>[grid axis](#grid-axis); or if there are none such, choose the first one.
    5.  Update the <a id="ref-for-auto-placement-cursor①"></a>[auto-placement cursor](#auto-placement-cursor) to point to item’s last line.

2.  Place the item in its <a id="ref-for-grid-axis②③"></a>[grid axis](#grid-axis) tracks at the maximum of the <a id="ref-for-running-position①"></a>[running position](#running-position)s of the tracks it spans.

3.  Calculate the size of the item’s [containing block](#containing-block) and then layout the item. Set the <a id="ref-for-running-position②"></a>[running position](#running-position) of the spanned <a id="ref-for-grid-axis②④"></a>[grid axis](#grid-axis) tracks to <code><var>max_pos</var> + <a id="ref-for-outer-size"></a>[outer size](https://drafts.csswg.org/css-sizing-3/#outer-size) + <a id="ref-for-propdef-grid-gap"></a>[grid-gap](https://drafts.csswg.org/css-gaps-1/#propdef-grid-gap)</code>.

    For this purpose, the <a id="ref-for-outer-size①"></a>[outer sizes](https://drafts.csswg.org/css-sizing-3/#outer-size) of the box are floored at zero.

    Note: That is, if the box has sufficiently large negative margins, it won’t somehow count as a negative <em>size</em> here. The <a id="ref-for-running-position③"></a>[running position](#running-position) of a track never decreases, so a negative-sized box won’t cause future normally-sized items placed in the track to overlap previous items.

4.  If the <a id="ref-for-grid-lanes-container②①"></a>[grid lanes container](#grid-lanes-container) uses <a id="ref-for-valdef-grid-auto-flow-dense②"></a>[dense](https://drafts.csswg.org/css-grid-2/#valdef-grid-auto-flow-dense) packing, and there exists skipped spaces in the layout (e.g. due to spanning items) into which the item, as it is sized now, could have fit if it were placed earlier, and where the spanned tracks have the same total used size as the tracks into which it is currently placed, then instead place it into the highest such space. If there are multiple valid spaces within the <a id="ref-for-grid-lanes-tie-threshold②"></a>[tie threshold](#grid-lanes-tie-threshold) of the highest space, place it in the <a id="ref-for-css-start"></a>[start](https://drafts.csswg.org/css-writing-modes-4/#css-start)-most of them. Rewind the <a id="ref-for-auto-placement-cursor②"></a>[auto-placement cursor](#auto-placement-cursor) and the <a id="ref-for-running-position④"></a>[running position](#running-position) to their values before this item’s placement.

    Note: Items that <em>visually</em> intrude into preceding empty spaces (via negative margins, <a id="ref-for-propdef-position"></a>[position: relative](https://drafts.csswg.org/css-position-3/#propdef-position), transforms, etc.), do not affect the size of those empty spaces. Later items can get placed in those spaces and visually overlap these previous items.

    Note: Dense packing both ignores the <a id="ref-for-auto-placement-cursor③"></a>[auto-placement cursor](#auto-placement-cursor) when backfilling, and does not update it after placement. If there aren’t any acceptable placement gaps to backfill, though, it places items exactly as when <a id="ref-for-valdef-grid-auto-flow-dense③"></a>[dense](https://drafts.csswg.org/css-grid-2/#valdef-grid-auto-flow-dense) were not specified.

Note: This algorithm chooses the track that would result in the item being placed as highly as possible. If there are ties, it chooses the earliest such track, <em>after</em> the most recently placed item if possible (ensuring that it always “moves forward” even in the presence of ties).

#### <a id="containing-block"></a>4.4.1.  Containing Block[](#containing-block)

The <a id="ref-for-containing-block"></a>[containing block](https://drafts.csswg.org/css-display-4/#containing-block) for a <a id="ref-for-grid-item③"></a>[grid item](https://drafts.csswg.org/css-grid-2/#grid-item) participating in <a id="ref-for-grid-lanes-layout①⓪"></a>[grid lanes layout](#grid-lanes-layout) is formed by its <a id="ref-for-grid-area"></a>[grid area](https://drafts.csswg.org/css-grid-2/#grid-area) in the <a id="ref-for-grid-axis②⑤"></a>[grid axis](#grid-axis) and the <a id="ref-for-grid-container①①"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container)’s <a id="ref-for-content-box①"></a>[content box](https://drafts.csswg.org/css-box-4/#content-box) in the <a id="ref-for-stacking-axis①⓪"></a>[stacking axis](#stacking-axis).

#### <a id="rtl-example"></a>4.4.2.  Placement and Writing Modes[](#rtl-example)

Note: Like all of <a id="ref-for-grid-layout①②"></a>[grid layout](https://drafts.csswg.org/css-grid-2/#grid-layout), grid lanes layout and placement is sensitive to the <a id="ref-for-writing-mode"></a>[writing mode](https://drafts.csswg.org/css-writing-modes-4/#writing-mode). For example, for <a id="ref-for-propdef-direction"></a>[direction: rtl](https://drafts.csswg.org/css-writing-modes-3/#propdef-direction), items are placed right-to-left rather than left-to-right, whether the inline axis is a <a id="ref-for-grid-axis②⑥"></a>[grid axis](#grid-axis) or a <a id="ref-for-stacking-axis①①"></a>[stacking axis](#stacking-axis).

<a id="example-5f5333ca"></a>

<strong>Example:</strong>

[](#example-5f5333ca) Here’s a simple [example](https://drafts.csswg.org/css-grid-3/examples/rtl-grid-axis.html) using <a id="ref-for-propdef-direction①"></a>[direction: rtl](https://drafts.csswg.org/css-writing-modes-3/#propdef-direction) in the <a id="ref-for-grid-axis②⑦"></a>[grid axis](#grid-axis):

``` text
<style>
  .grid {
    display: inline-grid-lanes;
    direction: rtl;
    grid-template-columns: repeat(4, 2ch);
    border: 1px solid;
  }

  item { background: silver }
  item:nth-child(2n+1) {
    background: pink;
    height: 4em;
  }
  </style>
```

``` text
<div class="grid">
  <item>1</item>
  <item style="grid-column:span 2">2</item>
  <item>3</item>
  <item>4</item>
</div>
```

<figure>
<img src="https://drafts.csswg.org/css-grid-3/images/rtl-grid-axis.png" />
<figcaption>Rendering of the <a id="ref-for-propdef-direction②"></a><a href="https://drafts.csswg.org/css-writing-modes-3/#propdef-direction">direction: rtl</a> example above.</figcaption>
</figure>

<a id="example-7bed5830"></a>

<strong>Example:</strong>

[](#example-7bed5830) Here’s a simple [example](https://drafts.csswg.org/css-grid-3/examples/rtl-masonry-axis.html) using <a id="ref-for-propdef-direction③"></a>[direction: rtl](https://drafts.csswg.org/css-writing-modes-3/#propdef-direction) in the <a id="ref-for-stacking-axis①②"></a>[stacking axis](#stacking-axis):

``` text
<style>
.grid {
  display: inline-grid-lanes;
  direction: rtl;
  width: 10ch;
  column-gap: 1ch;
  grid-template-rows: repeat(4, 2em);
  border: 1px solid;
}

item { background: silver }
item:nth-child(2n+1) {
  background: pink;
  width: 4ch;
}
</style>
```

``` text
<div class="grid">
  <item>1</item>
  <item style="grid-row:span 2">2</item>
  <item>3</item>
  <item>4</item>
</div>
```

<figure>
<img src="https://drafts.csswg.org/css-grid-3/images/rtl-masonry-axis.png" />
<figcaption>Rendering of the <a id="ref-for-propdef-direction④"></a><a href="https://drafts.csswg.org/css-writing-modes-3/#propdef-direction">direction: rtl</a> example above.</figcaption>
</figure>

## <a id="intrinsic-sizes"></a>5.  Sizing Grid Containers[](#intrinsic-sizes)

[Sizing Grid Containers](https://drafts.csswg.org/css-grid-2/#intrinsic-sizes) works the same as for regular <a id="ref-for-grid-container①②"></a>[grid containers](https://drafts.csswg.org/css-grid-2/#grid-container) but with the following replacement for the <a id="ref-for-stacking-axis①③"></a>[stacking axis](#stacking-axis): The <a id="ref-for-max-content"></a>[max-content size](https://drafts.csswg.org/css-sizing-3/#max-content) (<a id="ref-for-min-content"></a>[min-content size](https://drafts.csswg.org/css-sizing-3/#min-content)) of a <a id="ref-for-grid-container①③"></a>grid container in the <a id="ref-for-stacking-axis①④"></a>stacking axis is the size of the <a id="ref-for-stacking-range"></a>[stacking range](#stacking-range) when the <a id="ref-for-grid-lanes-container②②"></a>[grid lanes container](#grid-lanes-container) is sized under a <a id="ref-for-max-content-constraint"></a>[max-content constraint](https://drafts.csswg.org/css-sizing-3/#max-content-constraint) (<a id="ref-for-min-content-constraint"></a>[min-content constraint](https://drafts.csswg.org/css-sizing-3/#min-content-constraint)) in that axis. The <a id="stacking-range"></a><strong>stacking range</strong> is the range between the startmost <a id="ref-for-outer-edge"></a>[outer edge](https://drafts.csswg.org/css-box-4/#outer-edge) among the first items of each track and the endmost <a id="ref-for-outer-edge①"></a>outer edge among the last items of each track.

<a id="example-51eed6fc"></a>

<strong>Example:</strong>

[](#example-51eed6fc) Here’s a simple [example](https://drafts.csswg.org/css-grid-3/examples/grid-intrinsic-sizing-example-1.html):

``` text
<style>
.grid {
  display: inline-grid-lanes;
  grid-template-columns: 50px 100px auto;
  grid-gap: 10px;
  border: 1px solid;
}
item { background: silver; margin: 5px; }
</style>
```

``` text
<div class="grid">
  <item style="border:10px solid">1</item>
  <item>2</item>
  <item>3</item>
  <item style="height:50px">4</item>
  <item>5</item>
  <item>6</item>
</div>
```

<figure>
<img src="https://drafts.csswg.org/css-grid-3/images/grid-intrinsic-sizing-example-1.png" />
<figcaption>Rendering of the <a id="ref-for-grid-container①④"></a><a href="https://drafts.csswg.org/css-grid-2/#grid-container">grid container</a> intrinsic sizing example above.</figcaption>
</figure>

## <a id="alignment"></a>6.  Alignment and Spacing[](#alignment)

### <a id="gutters"></a>6.1.  Gutters: the <a id="ref-for-propdef-row-gap"></a>[row-gap](https://drafts.csswg.org/css-gaps-1/#propdef-row-gap), <a id="ref-for-propdef-column-gap"></a>[column-gap](https://drafts.csswg.org/css-gaps-1/#propdef-column-gap), and <a id="ref-for-propdef-gap"></a>[gap](https://drafts.csswg.org/css-gaps-1/#propdef-gap) properties[](#gutters)

[Gutters](https://drafts.csswg.org/css-grid-2/#gutters) are supported in both axes using the <a id="ref-for-propdef-row-gap①"></a>[row-gap](https://drafts.csswg.org/css-gaps-1/#propdef-row-gap) and <a id="ref-for-propdef-column-gap①"></a>[column-gap](https://drafts.csswg.org/css-gaps-1/#propdef-column-gap) properties (and their <a id="ref-for-propdef-gap①"></a>[gap](https://drafts.csswg.org/css-gaps-1/#propdef-gap) <a id="ref-for-shorthand-property"></a>[shorthand](https://drafts.csswg.org/css-cascade-5/#shorthand-property)). Margins do not collapse in either axis.

In the <a id="ref-for-grid-axis②⑧"></a>[grid axis](#grid-axis), gutters are the spacing between adjacent <a id="ref-for-grid-track③"></a>[grid tracks](https://drafts.csswg.org/css-grid-2/#grid-track), as in a regular <a id="ref-for-grid-container①⑤"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container).

In the <a id="ref-for-stacking-axis①⑤"></a>[stacking axis](#stacking-axis), a gutter is placed before each <a id="ref-for-grid-item④"></a>[grid item](https://drafts.csswg.org/css-grid-2/#grid-item) except the first in a <a id="ref-for-grid-track④"></a>[grid track](https://drafts.csswg.org/css-grid-2/#grid-track), and extends across that track in the <a id="ref-for-grid-axis②⑨"></a>[grid axis](#grid-axis). The gutter is placed before item self alignment is applied, thus the self alignment of an item does not affect the gutter’s position. Each gutter has the specified gap size, any additional space left by item placement is not part of the gutter. For an item that spans multiple tracks, there is a separate gutter in each track it spans; these gutters do not join across tracks.

<figure>
<img src="https://drafts.csswg.org/css-grid-3/images/grid-lanes-gutters-span.png" />
<figcaption>Gutters in a four-column grid lanes container with spanning items; <a id="ref-for-grid-axis③⓪"></a><a href="#grid-axis">grid axis</a> gutters in orange and <a id="ref-for-stacking-axis①⑥"></a><a href="#stacking-axis">stacking axis</a> gutters in blue. The orange gutters continue behind spanning items, while each blue gutter remains within a single track.</figcaption>
</figure>

### <a id="grid-axis-alignment"></a>6.2.  Grid-axis Alignment: the <a id="ref-for-propdef-align-content"></a>[align-content](https://drafts.csswg.org/css-align-3/#propdef-align-content)/<a id="ref-for-propdef-justify-content"></a>[justify-content](https://drafts.csswg.org/css-align-3/#propdef-justify-content), <a id="ref-for-propdef-align-self"></a>[align-self](https://drafts.csswg.org/css-align-3/#propdef-align-self)/<a id="ref-for-propdef-justify-self"></a>[justify-self](https://drafts.csswg.org/css-align-3/#propdef-justify-self), and <a id="ref-for-propdef-align-items"></a>[align-items](https://drafts.csswg.org/css-align-3/#propdef-align-items)/<a id="ref-for-propdef-justify-items"></a>[justify-items](https://drafts.csswg.org/css-align-3/#propdef-justify-items) properties[](#grid-axis-alignment)

In the <a id="ref-for-grid-axis③①"></a>[grid axis](#grid-axis), the <a id="ref-for-box-alignment-properties"></a>[box alignment properties](https://drafts.csswg.org/css-align-3/#box-alignment-properties) work the same as in a regular <a id="ref-for-grid-container①⑥"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container). See [CSS Grid Layout 2 §  11. Alignment and Spacing](https://drafts.csswg.org/css-grid-2/#alignment) and [CSS Box Alignment Level 3](https://drafts.csswg.org/css-grid-3/#biblio-css-align-3).

### <a id="stacking-content-alignment"></a>6.3.  Stacking-axis Content Distribution: the <a id="ref-for-propdef-align-content①"></a>[align-content](https://drafts.csswg.org/css-align-3/#propdef-align-content)/<a id="ref-for-propdef-justify-content①"></a>[justify-content](https://drafts.csswg.org/css-align-3/#propdef-justify-content) properties[](#stacking-content-alignment)

In the <a id="ref-for-stacking-axis①⑦"></a>[stacking axis](#stacking-axis), [content-distribution](https://drafts.csswg.org/css-align-3/#content-distribution) is applied to the content as a whole, similarly to how it behaves in block containers. More specifically, the <a id="ref-for-alignment-subject"></a>[alignment subject](https://drafts.csswg.org/css-align-3/#alignment-subject) is the <a id="ref-for-stacking-range①"></a>[stacking range](#stacking-range).

<figure>
<img src="https://drafts.csswg.org/css-grid-3/images/masonry-box.png" />
<figcaption>The extent of the alignment subject is indicated by the dashed border, while the alignment container is the whole container (indicated by the height of the column dividers). It defaults to start-aligning, but depending on (in this case) <a id="ref-for-propdef-align-content②"></a><a href="https://drafts.csswg.org/css-align-3/#propdef-align-content">align-content</a>, the items can move down, as a block, to center- or end-align vertically.</figcaption>
</figure>

Note: There is only ever one <a id="ref-for-alignment-subject①"></a>[alignment subject](https://drafts.csswg.org/css-align-3/#alignment-subject) for these properties in the <a id="ref-for-stacking-axis①⑧"></a>[stacking axis](#stacking-axis), so the unique <a id="ref-for-propdef-align-content③"></a>[align-content](https://drafts.csswg.org/css-align-3/#propdef-align-content) / <a id="ref-for-propdef-justify-content②"></a>[justify-content](https://drafts.csswg.org/css-align-3/#propdef-justify-content) values boil down to <a id="ref-for-valdef-self-position-start"></a>[start](https://drafts.csswg.org/css-align-3/#valdef-self-position-start), <a id="ref-for-valdef-self-position-center"></a>[center](https://drafts.csswg.org/css-align-3/#valdef-self-position-center), <a id="ref-for-valdef-self-position-end"></a>[end](https://drafts.csswg.org/css-align-3/#valdef-self-position-end), and <a id="ref-for-baseline-alignment①"></a>[baseline alignment](https://drafts.csswg.org/css-align-3/#baseline-alignment). (The behavior of <a id="ref-for-valdef-justify-content-normal"></a>[normal](https://drafts.csswg.org/css-align-3/#valdef-justify-content-normal) and <a id="ref-for-valdef-align-content-stretch"></a>[stretch](https://drafts.csswg.org/css-align-3/#valdef-align-content-stretch) is identical to <a id="ref-for-valdef-self-position-start①"></a>start, and the <a id="ref-for-distributed-alignment"></a>[distributed alignment](https://drafts.csswg.org/css-align-3/#distributed-alignment) values behave as their <a id="ref-for-fallback-alignment"></a>[fallback alignments](https://drafts.csswg.org/css-align-3/#fallback-alignment).) If the <a id="ref-for-grid-item⑤"></a>[grid items](https://drafts.csswg.org/css-grid-2/#grid-item) overflow the <a id="ref-for-grid-container①⑦"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container)’s <a id="ref-for-content-box②"></a>[content box](https://drafts.csswg.org/css-box-4/#content-box) in the <a id="ref-for-stacking-axis①⑨"></a>stacking axis, then the <a id="ref-for-stacking-range②"></a>[stacking range](#stacking-range) will be larger than the <a id="ref-for-grid-container①⑧"></a>grid container’s <a id="ref-for-content-box③"></a>content box.

### <a id="stacking-self-alignment"></a>6.4.  Stacking-axis Self Alignment: the <a id="ref-for-propdef-align-self①"></a>[align-self](https://drafts.csswg.org/css-align-3/#propdef-align-self)/<a id="ref-for-propdef-justify-self①"></a>[justify-self](https://drafts.csswg.org/css-align-3/#propdef-justify-self) and <a id="ref-for-propdef-align-items①"></a>[align-items](https://drafts.csswg.org/css-align-3/#propdef-align-items)/<a id="ref-for-propdef-justify-items①"></a>[justify-items](https://drafts.csswg.org/css-align-3/#propdef-justify-items) properties[](#stacking-self-alignment)

In the <a id="ref-for-stacking-axis②⓪"></a>[stacking axis](#stacking-axis), the <a id="ref-for-self-alignment-properties"></a>[self-alignment properties](https://drafts.csswg.org/css-align-3/#self-alignment-properties) only apply to items that are adjacent to a “gap” in the layout, i.e. placed in their <a id="ref-for-grid-track⑤"></a>[grid track](https://drafts.csswg.org/css-grid-2/#grid-track)(s) either immediately before a spanning item or as the last item. The <a id="ref-for-alignment-subject②"></a>[alignment subject](https://drafts.csswg.org/css-align-3/#alignment-subject) is the item’s <a id="ref-for-margin-box"></a>[margin box](https://drafts.csswg.org/css-box-4/#margin-box), and the <a id="ref-for-alignment-container"></a>[alignment container](https://drafts.csswg.org/css-align-3/#alignment-container) is that box plus the adjacent “gap”. For the last item in a track, the <a id="ref-for-alignment-container①"></a>alignment container extends to the lowest bottom outer edge among all the last items in all tracks.

<figure>
<img src="https://drafts.csswg.org/css-grid-3/images/align-self.png" alt="In a 4-column grid lanes container with 6 items,&#10;&#9;&#9;&#9;&#9;where the last item spans 3 columns&#10;&#9;&#9;&#9;&#9;and is pushed down by the 2nd item,&#10;&#9;&#9;&#9;&#9;leaving a gap below the items in the other three columns,&#10;&#9;&#9;&#9;&#9;alignment is possible for the items adjacent to this gap.&#10;&#9;&#9;&#9;&#9;The alignment container stretches from the top of the item&#10;&#9;&#9;&#9;&#9;to the bottom of the gap below it.&#10;&#9;&#9;&#9;&#9;For the last column,&#10;&#9;&#9;&#9;&#9;which the spanning item doesn&#x27;t reach,&#10;&#9;&#9;&#9;&#9;it stretches to the bottom of the spanning item&#10;&#9;&#9;&#9;&#9;(the lowest item in the grid prior to alignment).">
<figcaption>The three highlighted items are the only ones with "gaps" following them, so they’re the only ones that will respond to <a id="ref-for-propdef-align-items②"></a><a href="https://drafts.csswg.org/css-align-3/#propdef-align-items">align-items</a>. Their <a id="ref-for-alignment-container②"></a><a href="https://drafts.csswg.org/css-align-3/#alignment-container">alignment containers</a> are indicated by the cross-hatched areas.</figcaption>
</figure>

<figure>
<table>
<thead>
<tr class="header">
<th><a id="ref-for-propdef-align-self②"></a><a href="https://drafts.csswg.org/css-align-3/#propdef-align-self">align-self: start</a></th>
<th><a id="ref-for-propdef-align-self③"></a><a href="https://drafts.csswg.org/css-align-3/#propdef-align-self">align-self: end</a></th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td><img src="https://drafts.csswg.org/css-grid-3/images/self-align-stack-start.png" alt="In a 3-column grid lanes container with 5 items,&#10;&#9;&#9;&#9;&#9;&#9;&#9;where the first and third items are short,&#10;&#9;&#9;&#9;&#9;&#9;&#9;the third item spans two columns,&#10;&#9;&#9;&#9;&#9;&#9;&#9;and the last item spands all three,&#10;&#9;&#9;&#9;&#9;&#9;&#9;if the second is taller than the first,&#10;&#9;&#9;&#9;&#9;&#9;&#9;and the fourth is taller than the second and third,&#10;&#9;&#9;&#9;&#9;&#9;&#9;two adjacent alignment containers are created:&#10;&#9;&#9;&#9;&#9;&#9;&#9;one as tall as the second item, into which the first item aligns,&#10;&#9;&#9;&#9;&#9;&#9;&#9;and one as tall as the third item minus the second item,&#10;&#9;&#9;&#9;&#9;&#9;&#9;into which the column-spanning fourth item aligns."></td>
<td><img src="https://drafts.csswg.org/css-grid-3/images/self-align-stack-end.png" alt="When the items are aligned to the bottom,&#10;&#9;&#9;&#9;&#9;&#9;&#9;there is empty space above the fourth item.&#10;&#9;&#9;&#9;&#9;&#9;&#9;But this space, even though it is adjacent to the space for the first item,&#10;&#9;&#9;&#9;&#9;&#9;&#9;is not available for the first item,&#10;&#9;&#9;&#9;&#9;&#9;&#9;which aligns simply to the bottom of the second item."></td>
</tr>
</tbody>
</table>
<figcaption>Alignment of an item does not affect the size or position of other items' <a id="ref-for-alignment-container③"></a><a href="https://drafts.csswg.org/css-align-3/#alignment-container">alignment containers</a>.</figcaption>
</figure>

### <a id="grid-lanes-baseline-alignment"></a>6.5. <a id="masonry-baseline-alignment"></a> Baseline Alignment[](#grid-lanes-baseline-alignment)

Item <a id="ref-for-baseline-alignment②"></a>[baseline alignment](https://drafts.csswg.org/css-align-3/#baseline-alignment) inside the <a id="ref-for-grid-axis③②"></a>[grid axis](#grid-axis) tracks works as usual for a regular <a id="ref-for-grid-container①⑨"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container), and the <a id="ref-for-grid-container②⓪"></a>grid container’s baseline is determined the same as for a regular <a id="ref-for-grid-container②①"></a>grid container in that axis.

<a id="ref-for-baseline-alignment③"></a>[Baseline alignment](https://drafts.csswg.org/css-align-3/#baseline-alignment) is not supported in the <a id="ref-for-stacking-axis②①"></a>[stacking axis](#stacking-axis). The <a id="ref-for-baseline-alignment④"></a>baseline alignment values of the <a id="ref-for-self-alignment-properties①"></a>[self-alignment properties](https://drafts.csswg.org/css-align-3/#self-alignment-properties) on <a id="ref-for-grid-item⑥"></a>[grid items](https://drafts.csswg.org/css-grid-2/#grid-item) are treated as their <a id="ref-for-fallback-alignment①"></a>[fallback alignment](https://drafts.csswg.org/css-align-3/#fallback-alignment) in this axis. The first baseline set of the <a id="ref-for-grid-container②②"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container) in this axis is generated from the highest <a id="ref-for-alignment-baseline"></a>[alignment baseline](https://drafts.csswg.org/css-align-3/#alignment-baseline) among the <a id="ref-for-grid-item⑦"></a>grid items placed first in each track, and the last baseline set from the lowest <a id="ref-for-alignment-baseline①"></a>alignment baseline among the <a id="ref-for-grid-item⑧"></a>grid items placed last in each track.

Note: Whether an item is “first” or “last” depends only on placement (including <a id="ref-for-valdef-grid-auto-flow-dense④"></a>[dense](https://drafts.csswg.org/css-grid-2/#valdef-grid-auto-flow-dense) packing adjustments), and is not affected by its actual position on the page, e.g. negative margins or <a id="ref-for-x34"></a>[relative positioning](https://www.w3.org/TR/CSS2/visuren.html#x34) are not considered.

## <a id="pagination"></a>7.  Fragmentation[](#pagination)

### <a id="stacking-axis-pagination"></a>7.1. <a id="masonry-axis-pagination"></a> Fragmentation in the stacking axis[](#stacking-axis-pagination)

Each <a id="ref-for-grid-axis③③"></a>[grid axis](#grid-axis) track is fragmented independently in the <a id="ref-for-stacking-axis②②"></a>[stacking axis](#stacking-axis). If a <a id="ref-for-grid-item⑨"></a>[grid item](https://drafts.csswg.org/css-grid-2/#grid-item) is fragmented, or has a <a id="ref-for-forced-break"></a>[forced break](https://drafts.csswg.org/css-break-4/#forced-break) before/after it, then the <a id="ref-for-running-position⑤"></a>[running position](#running-position) for the tracks that it spans in the <a id="ref-for-grid-axis③④"></a>grid axis are set to the size of the <a id="ref-for-fragmentainer"></a>[fragmentainer](https://drafts.csswg.org/css-break-4/#fragmentainer) so that no further items will be placed in those tracks. An item that is split into multiple fragments retains its placement in the <a id="ref-for-grid-axis③⑤"></a>grid axis for all its fragments. A grid item that is pushed, however, is placed again by the next <a id="ref-for-grid-container②③"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container) fragment. Placement continues until all items are placed or pushed to a new fragment.

<a id="example-c4746f2e"></a>

<strong>Example:</strong>

[](#example-c4746f2e) Here’s an [example](https://drafts.csswg.org/css-grid-3/examples/fragmentation-block-axis-example.html) illustrating fragmentation of grid lanes layout with <a id="ref-for-stacking-axis②③"></a>[stacking](#stacking-axis) in its <a id="ref-for-block-axis③"></a>[block axis](https://drafts.csswg.org/css-writing-modes-4/#block-axis). It renders like this:

<figure>

<figcaption>Visualization of fragmentation in a <a id="ref-for-block-axis④"></a><a href="https://drafts.csswg.org/css-writing-modes-4/#block-axis">block-axis</a> <a id="ref-for-grid-lanes-layout①①"></a><a href="#grid-lanes-layout">grid lanes layout</a>.</figcaption>
</figure>

### <a id="grid-axis-pagination"></a>7.2.  Fragmentation in the Grid Axis[](#grid-axis-pagination)

Fragmentation in the <a id="ref-for-grid-axis③⑥"></a>[grid axis](#grid-axis) of a <a id="ref-for-grid-lanes-container②③"></a>[grid lanes container](#grid-lanes-container) is also supported. In this case the fragmentation behaves more like in a regular <a id="ref-for-grid-container②④"></a>[grid container](https://drafts.csswg.org/css-grid-2/#grid-container); however, there’s a separate step to determine which <a id="ref-for-grid-axis③⑦"></a>grid-axis track each item is placed into, before fragmentation occurs.

<a id="example-dab6ab26"></a>

<strong>Example:</strong>

[](#example-dab6ab26) Here’s an [example](https://drafts.csswg.org/css-grid-3/examples/fragmentation-inline-axis-example.html) illustrating fragmentation of grid lanes layout with <a id="ref-for-stacking-axis②④"></a>[stacking](#stacking-axis) in its <a id="ref-for-inline-axis④"></a>[inline axis](https://drafts.csswg.org/css-writing-modes-4/#inline-axis). In this case the breaks occurs between the <a id="ref-for-grid-axis③⑧"></a>[grid-axis](#grid-axis) rows. It renders like this:

<figure>

<figcaption>Visualization of fragmentation in the <a id="ref-for-block-axis⑤"></a><a href="https://drafts.csswg.org/css-writing-modes-4/#block-axis">block axis</a> with <a id="ref-for-inline-axis⑤"></a><a href="https://drafts.csswg.org/css-writing-modes-4/#inline-axis">inline-axis</a> <a id="ref-for-grid-lanes-layout①②"></a><a href="#grid-lanes-layout">grid lanes layout</a>.</figcaption>
</figure>

## <a id="abspos"></a>8.  Absolute Positioning[](#abspos)

[Grid-aligned absolute-positioned descendants](https://drafts.csswg.org/css-grid-1/#abspos-items) are supported in <a id="ref-for-grid-lanes-container②④"></a>[grid lanes containers](#grid-lanes-container) just as for regular <a id="ref-for-grid-container②⑤"></a>[grid containers](https://drafts.csswg.org/css-grid-2/#grid-container); however, in the <a id="ref-for-stacking-axis②⑤"></a>[stacking axis](#stacking-axis) there exist only two lines (in addition to the auto lines) for placement:

- line 1 (line -2) corresponds to the start edge of the <a id="ref-for-stacking-range③"></a>[stacking range](#stacking-range)

- line 2 (line -1) corresponds to the end edge of the <a id="ref-for-stacking-range④"></a>[stacking range](#stacking-range)

<a id="issue-4007c2e6"></a>

<strong>Issue:</strong>

[](#issue-4007c2e6) It might be useful to define a static position in the <a id="ref-for-stacking-axis②⑥"></a>[stacking axis](#stacking-axis). Maybe it could defined as the max (or min?) current <a id="ref-for-running-position⑥"></a>[running position](#running-position) of the <a id="ref-for-grid-axis③⑨"></a>[grid-axis](#grid-axis) tracks at that point? Or the end of the item before it?
