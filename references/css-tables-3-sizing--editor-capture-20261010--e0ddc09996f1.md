# CSS Tables Level 3: table sizing clauses

This is an explicitly partial reference capture of [CSS Tables Level 3](https://drafts.csswg.org/css-tables-3/), retrieved on 2026-10-10. It retains the original metadata, status and legal notice, followed by complete sections 3.1 (`layout-principles`), 3.5.1 (`table-layout-property`), 3.8 (`content-measure`), 3.9 (`width-distribution`), 3.10 (`height-distribution`) and 9 (`bug-list`), including all their descendant clauses, examples and editorial warnings. Other clauses are outside this capture; links to them resolve upstream. The captured page labels itself “W3C Working Draft, 8 October 2026” and warns it is not ready for implementation. It supplies source evidence; individual Surgeist choices must be stated separately.

The retrieved URL is the drafts.csswg.org page, not the dated TR URL displayed inside it. Retrieval of that displayed dated TR URL returned HTTP 404. Full retrieved HTML: 435,301 bytes, SHA-256 `f5adcb6c9ca724aad1dd1a04e20c2dab2695b33707af6da2acae30225ec4e381`; displayed source revision `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`. Selected-clause HTML container with explicit UTF-8 declaration: 124,672 bytes, SHA-256 `e0ddc09996f1f860a83fd67a72b8fd8148e79ee22c5c6e1f0b0172b879007505`.

Conversion uses the repository [HTML-to-Markdown tools](tools/html-to-markdown/README.md), approved Pandoc 3.1.11.1 and its semantic Lua filter. The three ordinary tables remain tables, including original image contents in the percentage-height example. Source IDs, headings, prose, literals, variables, links and every table cell are compared against parsed Markdown. The original W3C notice below remains part of the capture; see the local [W3C Software and Document License](../licenses/w3c/software-license-2023.txt).

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Table Module Level 3

<a id="w3c-state"></a>[W3C Working Draft](https://www.w3.org/standards/types/#WD), 8 October 2026

More details about this document

<strong>This version:</strong>

<https://www.w3.org/TR/2026/WD-css-tables-3-20261008/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-tables-3/>

<strong>Editor's Draft:</strong>

<https://drafts.csswg.org/css-tables-3/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2019/WD-css-tables-3-20190727/>

<https://www.w3.org/TR/CSS2/tables.html>

<strong>History:</strong>

<https://www.w3.org/standards/history/css-tables-3/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-tables-3)

[Inline In Spec](https://drafts.csswg.org/css-tables-3/#issues-index)

<strong>Editor:</strong>

Keith Cirkel (Mozilla)

<strong>Former Editors:</strong>

François Remy (Invited Expert)

Greg Whitworth (Microsoft)

Bert Bos (W3C)

[L. David Baron](https://dbaron.org/) ([Google](https://www.google.com/))

Markus Mielke (Microsoft)

Saloni Mira Rai (Microsoft)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-tables-3/Overview.bs)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-tables/>

Not Ready For Implementation

This spec is not yet ready for implementation. It exists in this repository to record the ideas and promote discussion.

Before attempting to implement this spec, please contact the CSSWG at www-style&#64;w3.org.

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

### <a id="layout-principles"></a>3.1. Core layout principles[](#layout-principles)

Unlike other block-level boxes, tables do not fill their containing block by default. When their <a id="ref-for-propdef-width①"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width) computes to <code>auto</code>, they behave as if they had <code>fit-content</code> specified instead. This is different from most block-level boxes, which behave as if they had <code>stretch</code> instead.

The <a id="min-content-width-of-a-table"></a><strong>min-content width of a table</strong> is the width required to fit all of its columns min-content widths and its <a id="ref-for-undistributable-space"></a>[undistributable spaces](#undistributable-space).

The <a id="max-content-width-of-a-table"></a><strong>max-content width of a table</strong> is the width required to fit all of its columns max-content widths and its <a id="ref-for-undistributable-space①"></a>[undistributable spaces](#undistributable-space).

If the width assigned to a table is larger than its <a id="ref-for-min-content-width-of-a-table"></a>[min-content width](#min-content-width-of-a-table), the [Available Width Distribution](#width-distribution) algorithm will adjust column widths in consequence.

This section overrides the general-purpose rules that apply to calculating widths described in other specifications. In particular, if the margins of a table are set to <code>0</code> and the width to <code>auto</code>, the table will not automatically size to fill its containing block. However, once the used value of <code>width</code> for the table is found (using the algorithms given below) then the other parts of those rules do apply. Therefore, a table can be centered using left and right <code>auto</code> margins, for instance.

#### <a id="table-layout-property"></a>3.5.1. The Table-Layout property[](#table-layout-property)

| Field                                                                                    | Definition                                                                                                   |
|------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-table-layout"></a><strong>table-layout</strong>                                               |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | auto <a id="ref-for-comb-one"></a>[\|](https://www.w3.org/TR/css-values-4/#comb-one) fixed                   |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | auto                                                                                                         |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | <a id="ref-for-table-grid-box"></a>[table grid boxes](https://drafts.csswg.org/css-tables-3/#table-grid-box) |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                           |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                          |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | specified keyword                                                                                            |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                  |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | discrete                                                                                                     |

A table-root is said to be laid out <a id="in-fixed-mode"></a><strong>in fixed mode</strong> whenever the computed value of the <a id="ref-for-propdef-table-layout①"></a>[table-layout](#propdef-table-layout) property is equal to <code>fixed</code>, and the specified width of the table root is not <code>auto</code> or <code>max-content</code>. When the specified width is one of those values, or if the computed value of the <a id="ref-for-propdef-table-layout②"></a>table-layout property is <code>auto</code>, then the table-root is said to be laid out <a id="in-auto-mode"></a><strong>in auto mode</strong>.

When a table-root is laid out <a id="ref-for-in-fixed-mode①"></a>[in fixed mode](#in-fixed-mode), the content of its table-cells is ignored for the purpose of width computation, the aggregation algorithm for column sizing considers only table-cells belonging to the first row track, such that layout only depends on the values explicitly specified for the table-columns or cells of the first row of the table; columns with indefinite widths are attributed their fair share of the remaining space after the columns with a definite width have been considered, or 0px if there is no remaining space (see [§ 3.8.3 Computing Column Measures](#computing-column-measures)).

### <a id="content-measure"></a>3.8. Computing table measures[](#content-measure)

#### <a id="computing-undistributable-space"></a>3.8.1. Computing Undistributable Space[](#computing-undistributable-space)

The <a id="undistributable-space"></a><strong>undistributable space</strong> of the table is the sum of the distances between the borders of consecutive table-cells (and between the border of the table-root and the table-cells).

The distance between the borders of two consecutive table-cells is the <a id="ref-for-propdef-border-spacing③"></a>[border-spacing](https://drafts.csswg.org/css-tables-3/#propdef-border-spacing), if any.

The distance between the table border and the borders of the cells on the edge of the table is the table’s padding for that side, plus the relevant border spacing distance (if any).

<a id="example-cd69817f"></a>

<strong>Example:</strong>

[](#example-cd69817f) For example, on the right hand side, the distance is padding-right + horizontal border-spacing.

#### <a id="computing-cell-measures"></a>3.8.2. Computing Cell Measures[](#computing-cell-measures)

The following terms are parameters of tables or table cells. These parameters encapsulate the differences between tables with different values of <a id="ref-for-propdef-border-collapse④"></a>[border-collapse](https://drafts.csswg.org/css-tables-3/#propdef-border-collapse) (separate or collapse) so that the remaining subsections of this section do not need to refer to them differently.

<strong><a id="cell-intrinsic-offsets"></a><strong>cell intrinsic offsets</strong></strong>

The cell intrinsic offsets is a term to capture the parts of padding and border of a table cell that are relevant to intrinsic width calculation. It is a set of computed values for border-left-width, padding-left, padding-right, and border-right-width (along with zero values for margin-left and margin-right) defined as follows:

- <a id="ref-for-in-separated-borders-mode①"></a>[In separated-borders mode](https://drafts.csswg.org/css-tables-3/#in-separated-borders-mode): the computed horizontal padding and border of the table-cell
- <a id="ref-for-in-collapsed-borders-mode②"></a>[In collapsed-borders mode](https://drafts.csswg.org/css-tables-3/#in-collapsed-borders-mode): the computed horizontal padding of the cell and, for border values, the used border-width values of the cell (half the winning border-width)

<strong><a id="table-intrinsic-offsets"></a><strong>table intrinsic offsets</strong></strong>

The table intrinsic offsets capture the parts of the padding and border of a table that are relevant to intrinsic width calculation. It is a set of computed values for border-left-width, padding-left, padding-right, and border-right-width (along with zero values for margin-left and margin-right) defined as follows:

- <a id="ref-for-in-separated-borders-mode②"></a>[In separated-borders mode](https://drafts.csswg.org/css-tables-3/#in-separated-borders-mode): the computed horizontal padding and border of the table-root
- <a id="ref-for-in-collapsed-borders-mode③"></a>[In collapsed-borders mode](https://drafts.csswg.org/css-tables-3/#in-collapsed-borders-mode): the used border-width values of the cell (half the winning border-width)

The margins are not included in the

<strong>Note:</strong>

<a id="ref-for-table-intrinsic-offsets"></a>[table intrinsic offsets](#table-intrinsic-offsets) because handling of margins depends on the <a id="ref-for-propdef-caption-side②"></a>[caption-side](https://drafts.csswg.org/css-tables-3/#propdef-caption-side) property.

<a id="issue-941e2d76"></a>

<strong>Issue:</strong>

[](#issue-941e2d76) Handling of intrinsic offsets when in border collapsing mode [\[Issue \#608\]](https://github.com/w3c/csswg-drafts/issues/608)

<strong><a id="total-horizontal-border-spacing"></a><strong>total horizontal border spacing</strong></strong>

The total horizontal border spacing is defined for each table:

- For tables laid out <a id="ref-for-in-separated-borders-mode③"></a>[in separated-borders mode](https://drafts.csswg.org/css-tables-3/#in-separated-borders-mode) containing at least one column, the horizontal component of the computed value of the border-spacing property times one plus the number of columns in the table
- Otherwise, 0

<strong><a id="offsets-adjusted-width"></a><strong>offsets-adjusted min-width, width, and max-width</strong></strong>

- For <a id="ref-for-table-track③"></a>[table-track](https://drafts.csswg.org/css-tables-3/#table-track) and <a id="ref-for-table-track-group-element③"></a>[table-track-group](https://drafts.csswg.org/css-tables-3/#table-track-group-element) boxes, the offsets-adjusted value of width properties is their computed value, irrespective of the value of <a id="ref-for-propdef-box-sizing"></a>[box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing) applied on the element.
- For <a id="ref-for-table-cell①⑦"></a>[table-cell](https://drafts.csswg.org/css-tables-3/#table-cell) boxes, the offsets-adjusted value of width properties is their computed value from which the cell’s border-{left\|right}-width and/or padding-{left\|right} have eventually been deduced, depending on the value of <a id="ref-for-propdef-box-sizing①"></a>[box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing).
  When the table is laid out
  <strong>Note:</strong>

  <a id="ref-for-in-collapsed-borders-mode④"></a>[in collapsed-borders mode](https://drafts.csswg.org/css-tables-3/#in-collapsed-borders-mode), the border value to deduce is half the value of the winning border value on each side (see [conflict resolution explanation note](https://drafts.csswg.org/css-tables-3/#border-conflict-resolution-algorithm))

[Testcase.](https://wptest.center/#/hmmbt3) [Testcase.](https://wptest.center/#/rnfac4) [Testcase.](https://wptest.center/#/3lao88)

<strong><a id="outer-min-content"></a><strong>outer min-content</strong> and <a id="outer-max-content"></a><strong>outer max-content</strong> widths</strong>

The outer min-content and max-content widths are defined for table cells, columns, and column groups. The <a id="ref-for-propdef-width②"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width), <a id="ref-for-propdef-min-width"></a>[min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width), and <a id="ref-for-propdef-max-width"></a>[max-width](https://www.w3.org/TR/css-sizing-3/#propdef-max-width) values used in these definitions are the offsets-adjusted values defined <a id="ref-for-offsets-adjusted-width"></a>[above](#offsets-adjusted-width):

- The <strong>outer min-content width</strong> of a table-cell is <code>max(<a id="ref-for-propdef-min-width①"></a>[min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width), min-content width)</code> adjusted by the <a id="ref-for-cell-intrinsic-offsets"></a>[cell intrinsic offsets](#cell-intrinsic-offsets).
- The <strong>outer min-content width</strong> of a table-column or table-column-group is <code>max(<a id="ref-for-propdef-min-width②"></a>[min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width), <a id="ref-for-propdef-width③"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width))</code>.
- The <strong>outer max-content width</strong> of a table-cell in a <a id="ref-for-constrainedness"></a>[non-constrained column](#constrainedness) is <code>max(<a id="ref-for-propdef-min-width③"></a>[min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width), <a id="ref-for-propdef-width④"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width), min-content width, min(<a id="ref-for-propdef-max-width①"></a>[max-width](https://www.w3.org/TR/css-sizing-3/#propdef-max-width), max-content width))</code> adjusted by the <a id="ref-for-cell-intrinsic-offsets①"></a>[cell intrinsic offsets](#cell-intrinsic-offsets).
- The <strong>outer max-content width</strong> of a table-cell in a <a id="ref-for-constrainedness①"></a>[constrained column](#constrainedness) is <code>max(<a id="ref-for-propdef-min-width④"></a>[min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width), <a id="ref-for-propdef-width⑤"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width), min-content width, min(<a id="ref-for-propdef-max-width②"></a>[max-width](https://www.w3.org/TR/css-sizing-3/#propdef-max-width), <a id="ref-for-propdef-width⑥"></a>width))</code> adjusted by the <a id="ref-for-cell-intrinsic-offsets②"></a>[cell intrinsic offsets](#cell-intrinsic-offsets).
- The <strong>outer max-content width</strong> of a table-column or table-column-group is <code>max(<a id="ref-for-propdef-min-width⑤"></a>[min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width), min(<a id="ref-for-propdef-max-width③"></a>[max-width](https://www.w3.org/TR/css-sizing-3/#propdef-max-width), <a id="ref-for-propdef-width⑦"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width)))</code>.

<strong><a id="percentage-contribution"></a><strong>percentage contribution</strong>s</strong>

The percentage contribution of a table cell, column, or column group is defined in terms of the computed values of <a id="ref-for-propdef-width⑧"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and <a id="ref-for-propdef-max-width④"></a>[max-width](https://www.w3.org/TR/css-sizing-3/#propdef-max-width) that have computed values that are percentages:  
  
<code>min(percentage <a id="ref-for-propdef-width⑨"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width), percentage <a id="ref-for-propdef-max-width⑤"></a>[max-width](https://www.w3.org/TR/css-sizing-3/#propdef-max-width))</code>.  
  
If the computed values are not percentages, then <code>0%</code> is used for <a id="ref-for-propdef-width①⓪"></a>width, and an <code>infinite</code> percentage is used for <a id="ref-for-propdef-max-width⑥"></a>max-width.

Please note that

<strong>Note:</strong>

<a id="ref-for-propdef-min-width⑥"></a>[min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width) is not included in this computation. As a result, a percentage <a id="ref-for-propdef-min-width⑦"></a>min-width is ignored. Since <a id="ref-for-propdef-width①①"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width) functions like a <a id="ref-for-propdef-min-width⑧"></a>min-width in table layout and column sizing cannot be both length-based and percent-based, authors should not use <a id="ref-for-propdef-min-width⑨"></a>min-width on table-internal boxes and prefer to rely on <a id="ref-for-propdef-width①②"></a>width only instead.

#### <a id="computing-column-measures"></a>3.8.3. Computing Column Measures[](#computing-column-measures)

This subsection defines three important values associated with each column of a table: their <a id="ref-for-min-content-width-of-a-column"></a>[min-content width](#min-content-width-of-a-column) (the smallest possible width attributed to this column), their <a id="ref-for-max-content-width-of-a-column"></a>[max-content width](#max-content-width-of-a-column) (the width that would be attributed to the column if no other constraint applied), their <a id="ref-for-intrinsic-percentage-width-of-a-column"></a>[intrinsic percentage width](#intrinsic-percentage-width-of-a-column) (the percentage of the table width the column desires to get, and could end up overriding its max-content width).

To compute these values, an iterative algorithm is used. First, these values are computed ignoring any cell spanning more than one column. Then, these values are updated by taking into account cells spanning incrementally more columns. When cells that spanned all columns of the table have been considered, this algorithm ends and the values are then finalized.

<strong>Note:</strong>

For the purpose of measuring a column when laid out

<strong>Advisement:</strong>

<a id="ref-for-in-fixed-mode②"></a>[in fixed mode](#in-fixed-mode), only cells which <a id="ref-for-originate①"></a>[originate](https://drafts.csswg.org/css-tables-3/#originate) in the first row of the table (after reordering the header and footer) will be considered, if any. In addition, the min-content and max-content width of cells is considered zero unless they are directly specified as a length-percentage, in which case they are resolved based on the table width (if it is definite, otherwise use 0).

For the purpose of calculating the outer min-content width of cells, descendants of table cells whose width depends on percentages of their parent cell’s width are considered to have an auto width. [Testcase](https://jsfiddle.net/0e12ve9b/1/) [Testcase](https://jsfiddle.net/0e12ve9b/3/)

<strong>min-content width of a column based on cells of span up to 1</strong>

The largest of:

- the width specified for the column:
  - the <a id="ref-for-outer-min-content"></a>[outer min-content](#outer-min-content) width of its corresponding table-column, if any (and not auto)
  - the <a id="ref-for-outer-min-content①"></a>[outer min-content](#outer-min-content) width of its corresponding table-column-group, if any
  - or 0, if there is none
- the <a id="ref-for-outer-min-content②"></a>[outer min-content](#outer-min-content) width of each cell that <a id="ref-for-span③"></a>[spans](https://drafts.csswg.org/css-tables-3/#span) the column whose <a id="ref-for-slot⑨"></a>[colSpan](https://drafts.csswg.org/css-tables-3/#slot) is 1 (or just the one in the first row <a id="ref-for-in-fixed-mode③"></a>[in fixed mode](#in-fixed-mode)) or 0 if there is none

<strong>max-content width of a column based on cells of span up to 1</strong>

The largest of:

- the <a id="ref-for-outer-max-content"></a>[outer max-content](#outer-max-content) width of its corresponding table-column-group, if any
- the <a id="ref-for-outer-max-content①"></a>[outer max-content](#outer-max-content) width of its corresponding table-column, if any
- the <a id="ref-for-outer-max-content②"></a>[outer max-content](#outer-max-content) width of each cell that <a id="ref-for-span④"></a>[spans](https://drafts.csswg.org/css-tables-3/#span) the column whose <a id="ref-for-slot①⓪"></a>[colSpan](https://drafts.csswg.org/css-tables-3/#slot) is 1 (or just the one in the first row if <a id="ref-for-in-fixed-mode④"></a>[in fixed mode](#in-fixed-mode)) or 0 if there is no such cell

<strong>intrinsic percentage width of a column based on cells of span up to 1</strong>

The largest of the <a id="ref-for-percentage-contribution"></a>[percentage contributions](#percentage-contribution) of each cell that <a id="ref-for-span⑤"></a>[spans](https://drafts.csswg.org/css-tables-3/#span) the column whose <a id="ref-for-slot①①"></a>[colSpan](https://drafts.csswg.org/css-tables-3/#slot) is 1, of its corresponding table-column (if any), and of its corresponding table-column-group (if any)

<strong>min-content width of a column based on cells of span up to N (N \> 1)</strong>

the largest of the min-content width of the column based on cells of span up to N-1 and the contributions of the cells in the column whose <a id="ref-for-slot①②"></a>[colSpan](https://drafts.csswg.org/css-tables-3/#slot) is N, where the contribution of a cell is the result of taking the following steps:

1.  Define the baseline min-content width as the sum of the min-content widths based on cells of span up to N-1 of all columns that the cell spans.
2.  Define the baseline max-content width as the sum of the max-content widths based on cells of span up to N-1 of all columns that the cell spans.
3.  Define the baseline border spacing as the sum of the horizontal border-spacing for any columns spanned by the cell, other than the one in which the cell originates.
4.  The contribution of the cell is the sum of:
    - the min-content width of the column based on cells of span up to N-1
    - the product of:
      - the ratio of:
        - the max-content width of the column based on cells of span up to N-1 of the column minus the min-content width of the column based on cells of span up to N-1 of the column, to
        - the baseline max-content width minus the baseline min-content width

        or zero if this ratio is undefined, and
      - the outer min-content width of the cell minus the baseline min-content width and the baseline border spacing, clamped to be at least 0 and at most the difference between the baseline max-content width and the baseline min-content width
    - the product of:
      - the ratio of the max-content width based on cells of span up to N-1 of the column to the baseline max-content width
      - the outer min-content width of the cell minus the baseline max-content width and baseline border spacing, or 0 if this is negative

<strong>max-content width of a column based on cells of span up to N (N \> 1)</strong>

The largest of the max-content width based on cells of span up to N-1 and the contributions of the cells in the column whose <a id="ref-for-slot①③"></a>[colSpan](https://drafts.csswg.org/css-tables-3/#slot) is N, where the contribution of a cell is the result of taking the following steps:

1.  Define the baseline max-content width as the sum of the max-content widths based on cells of span up to N-1 of all columns that the cell spans.
2.  Define the baseline border spacing as the sum of the horizontal border-spacing for any columns spanned by the cell, other than the one in which the cell originates.
3.  The contribution of the cell is the sum of:
    - the max-content width of the column based on cells of span up to N-1
    - the product of:
      - the ratio of the max-content width based on cells of span up to N-1 of the column to the baseline max-content width
      - the outer max-content width of the cell minus the baseline max-content width and the baseline border spacing, or 0 if this is negative

<strong>intrinsic percentage width of a column based on cells of span up to N (N \> 1)</strong>

If the intrinsic percentage width of a column based on cells of span up to N-1 is greater than 0%, then the intrinsic percentage width of the column based on cells of span up to N is the same as the intrinsic percentage width of the column based on cells of span up to N-1.  
  
Otherwise, it is the largest of the contributions of the cells in the column whose <a id="ref-for-slot①④"></a>[colSpan](https://drafts.csswg.org/css-tables-3/#slot) is N, where the contribution of a cell is the result of taking the following steps:

1.  Start with the <a id="ref-for-percentage-contribution①"></a>[percentage contribution](#percentage-contribution) of the cell.
2.  Subtract the intrinsic percentage width of the column based on cells of span up to N-1 of all columns that the cell spans. If this gives a negative result, change it to 0%.
3.  Multiply by the ratio of
    - the column’s non-spanning max-content width to
    - the sum of the non-spanning max-content widths of all columns spanned by the cell that have an intrinsic percentage width of the column based on cells of span up to N-1 equal to 0%.

    However, if this ratio is undefined because the denominator is zero, instead use the 1 divided by the number of columns spanned by the cell that have an intrinsic percentage width of the column based on cells of span up to N-1 equal to zero.

<strong><a id="min-content-width-of-a-column"></a><strong>min-content width of a column</strong></strong>

the min-content width of the column based on cells of span up to N, where N is the number of columns in the table

<strong><a id="max-content-width-of-a-column"></a><strong>max-content width of a column</strong></strong>

the max-content width of the column based on cells of span up to N, where N is the number of columns in the table

<strong><a id="intrinsic-percentage-width-of-a-column"></a><strong>intrinsic percentage width of a column</strong></strong>

the smaller of:

- the intrinsic percentage width of the column based on cells of span up to N, where N is the number of columns in the table
- 100% minus the sum of the intrinsic percentage width of all prior columns in the table (further left when direction is "ltr" (right for "rtl")) [Testcase](https://wptest.center/#/45xdf3)

The clamping of the total of the intrinsic percentage widths of columns to a maximum of 100% means that the table layout algorithm is not invariant under switching of columns.

<strong>Note:</strong>

<strong><a id="constrainedness"></a><strong>constrainedness</strong></strong>

A column is constrained if its corresponding table-column-group (if any), its corresponding table-column (if any), or any of the cells spanning only that column has a computed <a id="ref-for-propdef-width①③"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width) that is not "auto", and is not a percentage.

In a future revision of this specification, this algorithm will need to account for character-alignment of cells ('<a id="ref-for-string-value"></a>[\<string\>](https://www.w3.org/TR/css-values-4/#string-value)' values of the <a id="ref-for-propdef-text-align①"></a>[text-align](https://www.w3.org/TR/css-text-3/#propdef-text-align) property). This requires (based on the 9 March 2011 editor’s draft of css3-text) separately tracking max-content widths for the part of the column before the center of the alignment string and the part of the column after the center of the alignment string. For tracking min-content widths, there are two options: either not track them, or track three values: two values as for max-content widths for any cells that do not have break points in them, and a fourth value for any cells that do have break points in them (and to which character alignment is therefore not mandatory).

<a id="issue-96c17185"></a>

<strong>Issue:</strong>

[](#issue-96c17185) EDITORIAL. The way this describes distribution of widths from colspanning cells is wrong. For min-content and max-content widths it should refer to the rules for distributing excess width to columns for intrinsic width calculation.

### <a id="width-distribution"></a>3.9. Available Width Distribution[](#width-distribution)

#### <a id="computing-the-table-width"></a>3.9.1. Computing the table width[](#computing-the-table-width)

Before deciding on the final width of all columns, it is necessary to compute the width of the table itself.

As noted before, this would usually be the sum of preferred width of all columns, plus any extra. In this case, the width distribution will result in giving each column its preferred width. There are however a few cases where the author asks for some other width explicitly, as well as a few cases where the table cannot be given the width it requires.

<strong>Note:</strong>

The <a id="capmin"></a><strong>caption width minimum (CAPMIN)</strong> is the largest of the <a id="ref-for-table-caption⑤"></a>[table captions](https://drafts.csswg.org/css-tables-3/#table-caption) <a id="ref-for-min-content-contribution"></a>[min-content contribution](https://www.w3.org/TR/css-sizing-3/#min-content-contribution).

The <a id="gridmin"></a><strong>row/column-grid width minimum (GRIDMIN)</strong> width is the sum of the <a id="ref-for-min-content-width-of-a-column①"></a>[min-content width](#min-content-width-of-a-column) of all the columns plus cell spacing or borders.

The <a id="gridmax"></a><strong>row/column-grid width maximum (GRIDMAX)</strong> width is the sum of the <a id="ref-for-max-content-width-of-a-column①"></a>[max-content width](#max-content-width-of-a-column) of all the columns plus cell spacing or borders.

The <a id="used-min-width-of-table"></a><strong>used min-width of a table</strong> is the greater of the resolved <a id="ref-for-propdef-min-width①⓪"></a>[min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width), <a id="ref-for-capmin①"></a>[CAPMIN](#capmin), and <a id="ref-for-gridmin"></a>[GRIDMIN](#gridmin).

The <a id="used-width-of-table"></a><strong>used width of a table</strong> depends on the columns and captions widths as follows:

- If the <a id="ref-for-table-root-element②②"></a>[table-root](https://drafts.csswg.org/css-tables-3/#table-root-element)’s <a id="ref-for-propdef-width①④"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width) property has a computed value (resolving to <a id="resolved-table-width"></a><strong>resolved-table-width</strong>) other than <code>auto</code>, the <a id="ref-for-used-width-of-table"></a>[used width](#used-width-of-table) is the greater of <a id="ref-for-resolved-table-width"></a>[resolved-table-width](#resolved-table-width), and the <a id="ref-for-used-min-width-of-table"></a>[used min-width of the table](#used-min-width-of-table).
  If the used width is greater than
  <strong>Note:</strong>

  <a id="ref-for-gridmin①"></a>[GRIDMIN](#gridmin), the extra width should be distributed over the columns. See [§ 3.9 Available Width Distribution](#width-distribution).
- If the <a id="ref-for-table-root-element②③"></a>[table-root](https://drafts.csswg.org/css-tables-3/#table-root-element) has 'width: auto', the <a id="ref-for-used-width-of-table①"></a>[used width](#used-width-of-table) is the greater of min(<a id="ref-for-gridmax"></a>[GRIDMAX](#gridmax), the table’s containing block width), the <a id="ref-for-used-min-width-of-table①"></a>[used min-width of the table](#used-min-width-of-table).

The <a id="assignable-table-width"></a><strong>assignable table width</strong> is the used width of the table minus the <a id="ref-for-total-horizontal-border-spacing"></a>[total horizontal border spacing](#total-horizontal-border-spacing) (if any). This is the width that we will be able to allocate to the columns.

In this algorithm, rows (and row groups) and columns (and column groups) both constrain and are constrained by the dimensions of the cells they contain. Setting the width of a column might indirectly influence the height of a row, and vice versa.

<strong>Note:</strong>

#### <a id="width-distribution-principles"></a>3.9.2. Core distribution principles[](#width-distribution-principles)

This section is not normative.

##### <a id="width-distribution-principles-rN"></a>3.9.2.1. Rules[](#width-distribution-principles-rN)

Ideally, each column should get its preferred width (usually its <a id="ref-for-max-content-width-of-a-column②"></a>[max-content width](#max-content-width-of-a-column)). However, the <a id="ref-for-assignable-table-width"></a>[assignable table width](#assignable-table-width) calculated before could be either too big or too small to achieve this result, in which case the user agent must assign adhoc widths to columns as described in the width distribution algorithm.

This algorithm follows three rules when determining a column’s used width:

<a id="width-distribution-principles-r0"></a> <strong>Rule 0:</strong> <a id="ref-for-in-fixed-mode⑤"></a>[In fixed mode](#in-fixed-mode), auto and percentages columns are assigned a minimum width of zero pixels, and percentage resolution follows a different set of rules, whose goal is to ensure pixel columns always get assigned their preferred width.

<a id="width-distribution-principles-r1"></a> <strong>Rule 1:</strong> When assigning preferred widths, specified percent columns have a higher priority than specified unit value columns, which have a higher priority than auto columns.

<a id="width-distribution-principles-r2"></a> <strong>Rule 2:</strong> Columns using the same <a id="ref-for-sizing-type"></a>[sizing type](#sizing-type) (percent columns, pixel columns, or auto columns) follow the same distribution method. For example, they all get their <a id="ref-for-min-content-width-of-a-column②"></a>[min-content width](#min-content-width-of-a-column) or they all get their <a id="ref-for-max-content-width-of-a-column③"></a>[max-content width](#max-content-width-of-a-column).  
There is one exception to this rule. When giving its preferred percent width to a percent-column, if that would result in a size smaller than its

<strong>Note:</strong>

<a id="ref-for-min-content-width-of-a-column③"></a>[min-content width](#min-content-width-of-a-column), the column will be assigned its <a id="ref-for-min-content-width-of-a-column④"></a>[min-content width](#min-content-width-of-a-column) instead though the percent-columns group as a whole is still regarded as being assigned the preferred percent widths.

<a id="width-distribution-principles-r3"></a> <strong>Rule 3:</strong> The sum of width assigned to all columns should be equal to the <a id="ref-for-assignable-table-width①"></a>[assignable table width](#assignable-table-width).

##### <a id="width-distribution-principles-rR"></a>3.9.2.2. Available sizings[](#width-distribution-principles-rR)

All three types of columns have the following possible used widths.

1.  min-content width:  
    The size required to fit the content of the column
2.  min-content width + delta:  
    A value between the min-content and preferred widths
3.  preferred width:  
    The size specified for the column, or the size required to fit the content of the column without breaking
4.  preferred width + delta  
    A value larger than the preferred width

The distribution algorithm defines those values and explains when to use them.

#### <a id="width-distribution-algorithm"></a>3.9.3. Distribution algorithm[](#width-distribution-algorithm)

When a table is laid out at a given used width, the used width of each column must be determined as follows, eventually after considering [the changes to this algorithm](#width-distribution-in-fixed-mode) applied <a id="ref-for-in-fixed-mode⑥"></a>[in fixed mode](#in-fixed-mode).

First, each column of the table is assigned a <a id="sizing-type"></a><strong>sizing type</strong>:

- <a id="table-percent-column"></a><strong>percent-column</strong>:  
  a column whose any constraint is defined to use a percentage only (with a value different from 0%)

- <a id="table-pixel-column"></a><strong>pixel-column</strong>:  
  column whose any constraint is defined to use a defined length only (and is not a percent-column)

- <a id="table-auto-column"></a><strong>auto-column</strong>:  
  any other column

Then, valid sizing methods are to be assigned to the columns by sizing type, yielding the following sizing-guesses:

1.  The <a id="min-content-sizing-guess"></a><strong>min-content sizing-guess</strong> is the set of column width assignments where each column is assigned its min-content width.
2.  The <a id="min-content-percentage-sizing-guess"></a><strong>min-content-percentage sizing-guess</strong> is the set of column width assignments where:
    - each <a id="ref-for-table-percent-column"></a>[percent-column](#table-percent-column) is assigned the larger of:
      - its intrinsic percentage width times the assignable width and
      - its min-content width.
    - all other columns are assigned their min-content width.
3.  The <a id="min-content-specified-sizing-guess"></a><strong>min-content-specified sizing-guess</strong> is the set of column width assignments where:
    - each <a id="ref-for-table-percent-column①"></a>[percent-column](#table-percent-column) is assigned the larger of:
      - its intrinsic percentage width times the assignable width and
      - its min-content width
    - any other column that is <a id="ref-for-constrainedness②"></a>[constrained](#constrainedness) is assigned its max-content width
    - all other columns are assigned their min-content width.
4.  The <a id="max-content-sizing-guess"></a><strong>max-content sizing-guess</strong> is the set of column width assignments where:
    - each <a id="ref-for-table-percent-column②"></a>[percent-column](#table-percent-column) is assigned the larger of:
      - its intrinsic percentage width times the assignable width and
      - its min-content width
    - all other columns are assigned their max-content width.

Note that:

<strong>Note:</strong>

- The <a id="ref-for-assignable-table-width②"></a>[assignable table width](#assignable-table-width) is always greater than or equal to the table width resulting from the min-content sizing-guess.

- The widths for each column in the four sizing-guesses (min-content, min-content-percentage, min-content-specified, and max-content) are in nondecreasing order.

If the <a id="ref-for-assignable-table-width③"></a>[assignable table width](#assignable-table-width) is less than or equal to the <a id="ref-for-max-content-sizing-guess"></a>[max-content sizing-guess](#max-content-sizing-guess), the used widths of the columns must be the linear combination (with weights adding to 1) of the two consecutive sizing-guesses whose width sums bound the available width.

Otherwise, the used widths of the columns are the result of starting from the <a id="ref-for-max-content-sizing-guess①"></a>[max-content sizing-guess](#max-content-sizing-guess) and distributing the excess width to the columns of the table according to the rules for <a id="ref-for-distributing-excess-width-to-columns"></a>[distributing excess width to columns](#distributing-excess-width-to-columns) (for used width).

<figure>
<p>The following schema describes the algorithm in a different way, to make it easier to understand.</p>
<p><strong>Note:</strong></p>
<p>Legend</p>
<p><strong>Sizing algorithms:</strong> Each drawing of the table represents a way of sizing the columns. The four cases on the left are the sizing-guesses described above in the spec: <a id="ref-for-min-content-sizing-guess"></a><a href="#min-content-sizing-guess">min-content</a>, <a id="ref-for-min-content-percentage-sizing-guess"></a><a href="#min-content-percentage-sizing-guess">min-content-percentage</a>, <a id="ref-for-min-content-specified-sizing-guess"></a><a href="#min-content-specified-sizing-guess">min-content-specified</a>, and <a id="ref-for-max-content-sizing-guess②"></a><a href="#max-content-sizing-guess">max-content</a>. The cases on the right are interpolations required for available sizes that do not match exactly one of the four sizing-guesses.</p>
<p><strong>Choice of sizing method:</strong> The sizing method selection always starts at the min-content sizing-guess (top left), and then proceeds by comparing the available width and the width consumed by the method currently in use. Green arrows indicate the direction you should follow if you have extra space to distribute after applying the current method. Red arrows indicate the direction you should follow if you have distributed too much space by applying the current method and need to backtrack.</p>
<p><strong>Columns types:</strong> Each type of column (auto, px, %) has its own color in the schema (yellow, blue, orange). In an interpolation: columns that get shrunk down from their size in previous sizing-guess are repainted red, and columns that get expanded from their size in previous sizing-guess are repainted green.</p>
<img src="https://drafts.csswg.org/css-tables-3/images/CSS-Tables-Column-Width-Assignment.svg" alt="[see-caption-below]" />
<figcaption>Overview of the width distribution algorithm. Not normative.</figcaption>
</figure>

##### <a id="width-distribution-in-fixed-mode"></a>3.9.3.1. Changes to width distribution in fixed mode[](#width-distribution-in-fixed-mode)

The following changes to previous algorithm apply <a id="ref-for-in-fixed-mode⑦"></a>[in fixed mode](#in-fixed-mode):

- The <a id="ref-for-min-content-width-of-a-column⑤"></a>[min-content width](#min-content-width-of-a-column) of percent-columns and auto-columns is considered to be zero

- Cells ignore their border and padding size if their width is a percentage (<a id="ref-for-propdef-box-sizing②"></a>[box-sizing](https://www.w3.org/TR/css-sizing-3/#propdef-box-sizing) is ignored)

- If, when percentages are resolved based on the <a id="ref-for-assignable-table-width④"></a>[assignable table width](#assignable-table-width), the sum of columns widths based on this resolution would exceed the assignable table width, they are instead to be resolved relative to their percentage value such that the sum of columns width meets the assignable table width exactly.

- Columns whose size is computed as a sum of a percentage and a pixel length must be sized as if they counted as two columns, one with the pixel value, the other with the percentage value. This is different from resolving the percentage away, because of how width distribution works for percentage-based columns.

##### <a id="distributing-width-to-columns"></a>3.9.3.2. Distributing excess width to columns[](#distributing-width-to-columns)

The rules for <a id="distributing-excess-width-to-columns"></a><strong>distributing excess width to columns</strong> can be invoked in two ways:

- for distributing the excess width of a table to its columns during the computation of the used widths of those columns (for used width calculation), or
- for distributing the excess max-content or min-content width of a cell spanning more than one column to the max-content or min-content widths of the columns it spans (for intrinsic width calculation).

The rules for these two cases are largely the same, but there are slight differences.

The remainder of this section uses the term <a id="distributed-width"></a><strong>distributed width</strong> to refer to the one of these widths that is being distributed, and the <a id="excess-width"></a><strong>excess width</strong> is used to refer to the amount by which the width being distributed exceeds the sum of the distributed widths of the columns it is being distributed to.

1.  If there are <a id="ref-for-constrainedness③"></a>[non-constrained columns](#constrainedness) that have originating cells with intrinsic percentage width of 0% and with nonzero max-content width <em>(aka the columns allowed to grow by this rule)</em>, the <a id="ref-for-distributed-width"></a>[distributed widths](#distributed-width) of the columns allowed to grow by this rule are increased in proportion to max-content width so the total increase adds to the <a id="ref-for-excess-width"></a>[excess width](#excess-width).
2.  Otherwise, if there are <a id="ref-for-constrainedness④"></a>[non-constrained columns](#constrainedness) that have originating cells with intrinsic percentage width of 0% <em>(aka the columns allowed to grow by this rule, which thanks to the previous rule must have zero max-content width)</em>, the <a id="ref-for-distributed-width①"></a>[distributed widths](#distributed-width) of the columns allowed to grow by this rule are increased by equal amounts so the total increase adds to the <a id="ref-for-excess-width①"></a>[excess width](#excess-width).
3.  Otherwise, if there are <a id="ref-for-constrainedness⑤"></a>[constrained columns](#constrainedness) with intrinsic percentage width of 0% and with nonzero max-content width <em>(aka the columns allowed to grow by this rule, which, due to other rules, must have originating cells)</em>, the <a id="ref-for-distributed-width②"></a>[distributed widths](#distributed-width) of the columns allowed to grow by this rule are increased in proportion to max-content width so the total increase adds to the <a id="ref-for-excess-width②"></a>[excess width](#excess-width).
4.  Otherwise, if there are columns with intrinsic percentage width greater than 0% <em>(aka the columns allowed to grow by this rule, which, due to other rules, must have originating cells)</em>, the <a id="ref-for-distributed-width③"></a>[distributed widths](#distributed-width) of the columns allowed to grow by this rule are increased in proportion to intrinsic percentage width so the total increase adds to the <a id="ref-for-excess-width③"></a>[excess width](#excess-width).
5.  Otherwise, if there is any such column, the <a id="ref-for-distributed-width④"></a>[distributed widths](#distributed-width) of all columns that have originating cells are increased by equal amounts so the total increase adds to the <a id="ref-for-excess-width④"></a>[excess width](#excess-width).
6.  Otherwise, the <a id="ref-for-distributed-width⑤"></a>[distributed widths](#distributed-width) of all columns are increased by equal amounts so the total increase adds to the <a id="ref-for-excess-width⑤"></a>[excess width](#excess-width).

These rules do not apply when the table is laid out

<strong>Advisement:</strong>

<a id="ref-for-in-fixed-mode⑧"></a>[in fixed mode](#in-fixed-mode). In this case, the simpler rules that follow apply instead:

- If there are any columns with no width specified, the <a id="ref-for-excess-width⑥"></a>[excess width](#excess-width) is distributed in equally to such columns
- otherwise, if there are columns with non-zero length widths from the base assignment, the excess width is distributed proportionally to width among those columns
- otherwise, if there are columns with non-zero percentage widths from the base assignment, the excess width is distributed proportionally to percentage width among those columns
- otherwise, the excess width is distributed equally to the zero-sized columns

### <a id="height-distribution"></a>3.10. Available Height Distribution[](#height-distribution)

#### <a id="computing-the-table-height"></a>3.10.1. Computing the table height[](#computing-the-table-height)

<strong>Note:</strong>

[?Testcase](https://jsfiddle.net/bsgt4wbx/) [?Testcase](https://jsfiddle.net/bsgt4wbx/1/) [?Testcase](https://jsfiddle.net/bsgt4wbx/3/)

The <strong>height of a table</strong> is the sum of the row heights plus any cell spacing or borders. If the table has a <a id="ref-for-propdef-height①"></a>[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) property with a value other than auto, it is treated as a minimum height for the table grid, and will eventually be distributed to the height of the rows if their collective [minimum height](https://drafts.csswg.org/css-tables-3/) ends up smaller than this number. If their collective size ends up being greater than the specified <a id="ref-for-propdef-height②"></a>height, the specified <a id="ref-for-propdef-height③"></a>height will have no effect.

The <strong>minimum height of a row</strong> is the maximum of:

- the computed <a id="ref-for-propdef-height④"></a>[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) (if definite, percentages being considered 0px) of its corresponding table-row (if any)
- the computed <a id="ref-for-propdef-height⑤"></a>[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) of each cell spanning the current row exclusively (if definite, percentages being treated as 0px), and
- the minimum height (<a id="ref-for-ROWMIN"></a>[ROWMIN](#ROWMIN)) required by the cells spanning the row.

<a id="ROWMIN"></a><strong>ROWMIN</strong> is defined as the sum of the [minimum height of the rows](#row-layout) after a first row layout pass.

To compute the <strong>height of a table</strong>, it is therefore necessary to perform a first-pass layout on all its rows, compute the sum of all minimum row heights plus spacings/borders, and return the greater of either that value or the table-root specified <a id="ref-for-propdef-height⑥"></a>[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) (min-height).

Once the table height has been determined, rows will usually get a [second layout pass](#row-relayout) (where their cells' heights are no longer considered auto), then [height distribution](#height-distribution-algorithm) will happen to adjust their heights to collectively meet the table height, then table-cell descendants might get a [second layout](#table-cell-content-relayout) (where their percentage heights are resolved).

#### <a id="row-layout"></a>3.10.2. Row layout (first pass)[](#row-layout)

The <strong>minimum height of a row</strong> (without spanning-related height distribution) is defined as the height of a hypothetical linebox containing the cells originating in the row and where cells spanning multiple rows are considered having a height of 0px (but their correct baseline). In this hypothetical linebox, cell heights are considered auto, their width (including borders and paddings) is forced to the widths and inner spacings of the columns they span, but their other properties are conserved.

For the purpose of calculating this height, descendants of table cells whose height depends on percentages of their parent cell’s height <a id="ref-for-appropriateness-of-child-percentage-resolution"></a>[(see section below)](#appropriateness-of-child-percentage-resolution) are considered to have an auto height if they have <a id="ref-for-propdef-overflow③"></a>[overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) set to <a id="ref-for-valdef-overflow-visible"></a>[visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible), <a id="ref-for-valdef-overflow-clip"></a>[clip](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-clip), or <a id="ref-for-valdef-overflow-hidden"></a>[hidden](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-hidden) or if they are replaced elements, and a 0px height if they have not. [Testcase](https://jsfiddle.net/0e12ve9b/) [!!Testcase](https://jsfiddle.net/0e12ve9b/3/)

For table-cell descendants whose percentage height was ignored as a result of the above, a second layout pass of the table-cell content will happen once height distribution has concluded to attempt to properly take this sizing into account

<strong>Note:</strong>

<a id="ref-for-resolving-percentages-heights-in-table-cell-content"></a>[(see section below)](#resolving-percentages-heights-in-table-cell-content)

The <a id="table-cell-baseline"></a><strong>baseline of a cell</strong> is defined as the baseline of the first <a id="ref-for-in-flow"></a>[in-flow](https://www.w3.org/TR/css-display-4/#in-flow) line box in the cell, or the first in-flow table-row in the cell, whichever comes first. If there is no such line box or table-row, the baseline is the bottom of content edge of the cell box.

<a id="example-0e7ec077"></a>

<strong>Example:</strong>

[](#example-0e7ec077)

Here is how this works out in practice:

``` text
td { vertical-align: baseline; outline: 3px solid silver; }
img { float: left; clear: left; width: 32px; height: 32px; }
img[title] { float: none; }

<table><tr>
  <td>Baseline</td>
  <td>Baseline<table><tr><td>After</td></tr></table></td>
  <td><table><tr><td>Baseline</td></tr></table>After</td>
  <td><table align=right><tr><td>Before</td></tr></table><p>Baseline</p></td>
  <td><img src="http://w3.org/favicon.ico"><p>Baseline</p></td>
  <td><img src="http://w3.org/favicon.ico" title="Baseline"/><br/><img src="http://w3.org/favicon.ico" title="After"></td>
  <td><img src="http://w3.org/favicon.ico"><img src="http://w3.org/favicon.ico"><!--Baseline--></td>
</tr></table>
```

<figure>
<img src="https://drafts.csswg.org/css-tables-3/images/td-baseline-example-rendering.png" alt="[see-caption-below]" />
<figcaption>Rendering of <a href="https://jsfiddle.net/x8sh0f60/3/">this example</a> in a compliant browser</figcaption>
</figure>

For the purposes of finding a baseline,

<strong>Advisement:</strong>

<a id="ref-for-in-flow①"></a>[in-flow](https://www.w3.org/TR/css-display-4/#in-flow) boxes with a scrolling mechanisms (see the <a id="ref-for-propdef-overflow④"></a>[overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) property) must be considered as if scrolled to their origin position.

The baseline of a cell may end up below its bottom border, see the example below.

<strong>Note:</strong>

<a id="example-f858e943"></a>

<strong>Example:</strong>

[](#example-f858e943)

The cell in this example has a baseline below its bottom border:

``` text
div { height: 0; overflow: hidden; }

<table>
<tr>
<td>
<div> Test </div>
</td>
</tr>
</table>
```

The <a id="ref-for-propdef-vertical-align"></a>[vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) property of each table cell determines its alignment within the row. Each cell’s content has a baseline, a top, a middle, and a bottom, as does the row itself.

In the context of table cells, values for <a id="ref-for-propdef-vertical-align①"></a>[vertical-align](https://www.w3.org/TR/css-inline-3/#propdef-vertical-align) have the following meanings:

| Field                     | Definition                                                                                                                                                                                                                                                                            |
|---------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>baseline</strong> | The baseline of the cell is aligned with the baseline of the other cells of the first row it spans (see the definition of baselines of <a id="ref-for-table-cell-baseline"></a>[cells](#table-cell-baseline) and <a id="ref-for-table-row-baseline"></a>[rows](#table-row-baseline)). |
| <strong>top</strong>      | The top of the cell box is aligned with the top of the first row it spans.                                                                                                                                                                                                            |
| <strong>bottom</strong>   | The bottom of the cell box is aligned with the bottom of the last row it spans.                                                                                                                                                                                                       |
| <strong>middle</strong>   | The center of the cell is aligned with the center of the rows it spans.                                                                                                                                                                                                               |
| <strong>...</strong>      | Other values do not apply to cells; the cell is aligned at the baseline instead.                                                                                                                                                                                                      |

The maximum distance between the top of the cell box and the baseline over all cells that have 'vertical-align: baseline' is used to set the <a id="table-row-baseline"></a><strong>baseline of the row</strong>. If a row doesn’t have any cell that has 'vertical-align: baseline', the baseline of that row is the bottom content edge of the lowest cell in the row.

The <a id="table-root-baseline"></a><strong>baseline of a table-root</strong> is the baseline of its first row, if any. Otherwise, it is the bottom content edge of the table-root.

[Testcase](https://wptest.center/#/mz09g6) [!!Testcase](https://wptest.center/#/x80356)

<strong>Advisement:</strong>

To avoid ambiguous situations, the alignment of cells proceeds in the following order:

- First the cells that are aligned on their baseline are positioned. This will establish the baseline of the row.
- Next the cells with 'vertical-align: top' are positioned. The row now has a top, possibly a baseline, and a provisional height, which is the distance from the top to the lowest bottom of the cells positioned so far.
- If any of the remaining cells, those aligned at the bottom or the middle, have a height that is larger than the current height of the row, the height of the row will be increased to the maximum of those cells, by lowering the bottom.
- Finally, assign their position to the remaining cells.

<a id="example-7b23a3cb"></a>

<strong>Example:</strong>

[](#example-7b23a3cb)

Example showing how the previous algorithm creates the various alignment lines of a row.

<figure>
<a href="https://www.w3.org/TR/CSS2/images/longdesc/cell-align-desc.html"><img src="https://drafts.csswg.org/css-tables-3/images/cell-align-explainer.png" alt="[see-caption-below]" /></a>
<figcaption>Diagram showing the effect of various values of <a id="ref-for-propdef-vertical-align②"></a><a href="https://www.w3.org/TR/css-inline-3/#propdef-vertical-align">vertical-align</a> on table cells. Cell boxes 1 and 2 are aligned at their baselines. Cell box 2 has the largest height above the baseline, so that determines the baseline of the row.</figcaption>
</figure>

Since during row layout the specified heights of cells in the row were ignored and cells that were spanning more than one row have not been sized correctly, their height will need to be eventually distributed to the set of rows they spanned. This is done by running the same algorithm as the [column measurement](#computing-column-measures), with the span=1 value being initialized (for min-content) with the largest of the resulting height of the previous row layout, the height specified on the corresponding table-row (if any), and the largest height specified on cells that span this row only (the algorithm starts by considering cells of span 2 on top of that assignment).

<a id="issue-58165e24"></a>

<strong>Issue:</strong>

[](#issue-58165e24) EDITORIAL. Import the relevant section of [§ 3.8.3 Computing Column Measures](#computing-column-measures) here.

Rows that see their size increase as a result of applying these steps adjust by lowering their bottom.

<strong>Advisement:</strong>

The cells whose position depended on the bottom of any updated row must be positioned correctly again in their respective rows.

At this point, cell boxes that are smaller than the collective height of the rows they span receive extra top and/or bottom padding such that their content does not move vertically but their top/bottom edges meet the ones of the first/last row they span.

Please note that heights being defined on row groups are being ignored by this algorithm

<strong>Note:</strong>

#### <a id="row-relayout"></a>3.10.3. Row layout (second pass)[](#row-relayout)

Once the table height has been determined, a second row layout pass will be performed, if necessary, to assign the correct minimum height to table rows, by taking percentages used in rows/cells specified <a id="ref-for-propdef-height⑦"></a>[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) into account. Other than that, all instructions for the first-pass row layout apply [(see above)](#row-layout).

Please note that this second-pass minimum height therefore still treats percentage heights of table-cell descendants as advised for the first pass

<strong>Note:</strong>

[(see above)](#row-layout). For this reason, it is not required to relayout the content of table-cells to compute the new row minimum height. If necessary, table-cell content will undergo a relayout later, after table height distribution has concluded [(see below)](#table-cell-content-relayout).

Then, if the sum of the new heights of the table rows after this second pass is different from what is needed to fill the table height previously determined, the height distribution algorithm defined [below](#height-distribution-algorithm) is applied (either to shrink rows by sizing them intermediately between their first-pass minimum height and their second-pass one, or to increase the heights of all rows beyond their second-pass minimum height to fill the available space; in neither case, this will have an impact on the baseline of the rows).

#### <a id="height-distribution-principles"></a>3.10.4. Core distribution principles[](#height-distribution-principles)

<a id="issue-b62068df"></a>

<strong>Issue:</strong>

[](#issue-b62068df) EDITORIAL. TODO. For current proposal, skip to [§ 3.10.5 Distribution algorithm](#height-distribution-algorithm).

<strong>Note:</strong>

Investigations on height distribution

Initial analysis shows that there are indeed similarities between width and height distribution. There are also differences which I described here below:

In many case, all browsers apply a distribution algorithm that favors percentages over pixels over auto. [Case 6](https://jsfiddle.net/xg2ss965/2/).

A difference with the width distribution algorithm is that if the sum of all rows' heights is higher than 100%, then all browsers enter a completely different mode. [Case 7](https://jsfiddle.net/6ec0hxgx/). NOTE: The sum counts as well percentage heights and pixels heights, since at this point you can safely resolve percentages.   

In this case, pixel-tracks are sized properly first. Then, percentage tracks get the remaining space proportionally to their height percentage up to their height percentage. Finally, auto tracks get to fill the remaining space, if there is any auto track. If there is none, percentage tracks continue growing above their height percentage until all the space is filled. [Case 9](https://jsfiddle.net/xg2ss965/1/).

The height distribution algorithm also caps the sum of percentage heights to 100% in all browsers but Edge. That means that some rows get an arbitrary 0% height. [Case 8](https://jsfiddle.net/xg2ss965/).

In Edge and Firefox, empty tracks do not get an increased size by this distribution if there are filled auto tracks. In Chrome, empty tracks count as distributable tracks as well even if there are other auto tracks.

- Case 1: [height](https://jsfiddle.net/Lh9shm8p/) vs [width](https://jsfiddle.net/wza8huh7/)
- Case 2: [height](https://jsfiddle.net/Lh9shm8p/1/) vs [width](https://jsfiddle.net/wza8huh7/1/)
- Case 3: [height](https://jsfiddle.net/Lh9shm8p/2/) vs [width](https://jsfiddle.net/wza8huh7/2/)
- Case 4: [height](https://jsfiddle.net/Lh9shm8p/3/) vs [width](https://jsfiddle.net/wza8huh7/3/)
- Case 5: [height](https://jsfiddle.net/Lh9shm8p/4/) vs [width](https://jsfiddle.net/wza8huh7/4/)

<a id="min-content-and-percentages"></a> Interesting test cases about min-content and content using percentage sizes:

- [Case 10](https://jsfiddle.net/gzapbyeg/1/).
- [Case 11](https://jsfiddle.net/gzapbyeg/2/).

  
Chrome and Edge apply percentages on the final layout. All browsers work around them during the first pass by considering them 0% (Chrome) or by ignoring the declaration (Edge, Firefox). The difference of choice is visible in [Case 12](https://jsfiddle.net/vmfrLzke/1/). [Case 13](http://codepen.io/FremyCompany/pen/obdYjv?editors=1100).

#### <a id="height-distribution-algorithm"></a>3.10.5. Distribution algorithm[](#height-distribution-algorithm)

The first step is to attribute to each row its base size and its reference size.

Its <strong>base size</strong> is the size it would have got if the table didn’t have a specified height (the one it was assigned when ROWMIN was evaluated).

Its <strong>reference size</strong> is the largest of

- its initial base height and
- its new base height (the one evaluated during the second layout pass, where percentages used in rowgroups/rows/cells' specified heights were resolved according to the table height, instead of being ignored as 0px).

The second step is to compute the final height of each row based on those sizes.

If the table height is equal or smaller than sum of reference sizes, the final height assigned to each row will be the weighted mean of the base and the reference size that yields the correct total height.

Else, if the table owns any “auto-height” row (a row whose size is only determined by its content size and none of the specified heights), each non-auto-height row receives its reference height and auto-height rows receive their reference size plus some increment which is equal to the height missing to amount to the specified table height divided by the amount of such rows.

Else, all rows receive their reference size plus some increment which is equal to the height missing to amount to the specified table height divided by the amount of rows.

<strong>Advisement:</strong>

The cells whose position depended on the bottom of any updated row must be positioned correctly again in their respective rows.

At this point, cell boxes that are smaller than the collective height of the rows they span receive extra top and/or bottom padding such that their content does not move vertically but their top/bottom edges meet the ones of the first/last row they span.

#### <a id="table-cell-content-relayout"></a>3.10.6. Table-cell content layout (second pass)[](#table-cell-content-relayout)

Once table-height distribution has concluded, and the sum of row heights plus spacing/border is equal to the table height, the content of table-cells which contained descendants whose percentage heights were ignored or treated as 0px by the first-pass row layout rules [(see above)](#row-layout) must undergo a second layout pass, as defined below.

Note that this means UAs are either required to keep track of the usage of percentages in the properties of any direct child of the table-cell including (but not limited to) the

<strong>Note:</strong>

<a id="ref-for-propdef-height⑧"></a>[height](https://www.w3.org/TR/css-sizing-3/#propdef-height) and <a id="ref-for-propdef-min-height"></a>[min-height](https://www.w3.org/TR/css-sizing-3/#propdef-min-height) properties for horizontal flows and the <a id="ref-for-propdef-width①⑤"></a>[width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and <a id="ref-for-propdef-min-width①①"></a>[min-width](https://www.w3.org/TR/css-sizing-3/#propdef-min-width) properties for vertical flows, or else required to perform this second layout pass on table-cell content in all cases.

<a id="resolving-percentages-heights-in-table-cell-content"></a> <strong>Resolve percentage heights in table-cell content:</strong> Once the final size of the table and the rows has been determined, after height distribution has concluded, the content of the table-cells must also go through a second layout pass, where, if <a id="ref-for-appropriateness-of-child-percentage-resolution①"></a>[appropriate](#appropriateness-of-child-percentage-resolution), percentage-based heights are this time resolved against their parent cell used height.

<a id="appropriateness-of-child-percentage-resolution"></a> <strong>It is appropriate to resolve percentage heights on direct children of a table-cell</strong> if the cell is considered to have its height specified explicitly or the child is absolutely positioned, see [CSS 2](https://www.w3.org/TR/CSS2/visudet.html#the-height-property).

For compat reasons, it is further clarified that a cell is considered to have its height specified explicitly if the computed height of the cell is a length, or if the computed height of its table-root ancestor is a length or percentage, regardless of whether that percentage does <a id="ref-for-behave-as-auto"></a>[behave as auto](https://www.w3.org/TR/css-sizing-3/#behave-as-auto) or not.

<a id="example-a3b7459d"></a>

<strong>Example:</strong>

[](#example-a3b7459d) To clarify the preceding statements, here is a table of the resulting "A" div height based on the value being used:

``` text
<section style="height: var(--wrapper-height)">
  <table style="height: var(--table-height)">
    <tr>
      <td style="height: var(--table-cell-height)">

        <div style="height:100%; background:yellow">A</div>

      </td>
      <td style="height: var(--other-table-cell-height)">

        B<br>C

      </td>
    </tr>
  </table>
</section>
```

| --table-cell-height   | --table-height   | result                                                                                       |
|-----------------------|------------------|----------------------------------------------------------------------------------------------|
| \<length\>            | <em>\<any\></em> | ![100%](https://drafts.csswg.org/css-tables-3/images/percentage-heights-hundred-percent.png) |
| <em>\<any\></em>      | \<length\>       | ![100%](https://drafts.csswg.org/css-tables-3/images/percentage-heights-hundred-percent.png) |
| <em>\<any\></em>      | \<percentage\>   | ![100%](https://drafts.csswg.org/css-tables-3/images/percentage-heights-hundred-percent.png) |
| auto                  | auto             | ![auto](https://drafts.csswg.org/css-tables-3/images/percentage-heights-auto.png)            |
| \<percentage\>        | auto             | ![auto](https://drafts.csswg.org/css-tables-3/images/percentage-heights-auto.png)            |

Note that neither <code>--other-table-cell-height</code> nor <code>--wrapper-height</code> do influence the algorithm’s outcome.

A previous version of this specification incorrectly stated that <code>--wrapper-height</code> was taken into account when the table had a percentage height, but compat issues appeared when an implementation landed, and the behavior was then special-cased.

It is possible that this second layout pass (where height percentages are being resolved) will make some cell contents overflow their parent cell, for instance if the sum of all percentages used is superior to 100%. This is by design.

<strong>Note:</strong>

## <a id="bug-list"></a>9. List of bugs being tracked[](#bug-list)

This section is not normative.

- [Align=center attribute overrides css margins in Edge](http://codepen.io/FremyCompany/pen/jWzpjq?editors=1100)

  Chrome and Firefox let CSS win.  
  Edge hides every border + table/row-group/row background.  
    
  This specification says that Chrome and Firefox are right.  
  [§ 10.1 Mapping between CSS & HTML attributes](https://drafts.csswg.org/css-tables-3/#mapping)

- [Chrome applies nowrap quirks mode fix in DOCTYPE documents too](http://codepen.io/FremyCompany/pen/ZQxVMo?editors=1100)

  Edge and Firefox do not apply the fix in normal mode.  
  Chrome also applies it for css widths, which is not what the spec says to do.  
    
  The WHATWG spec says that Chrome is wrong.  
  This spec is aiming to state the same thing.  
  [§ 10.1 Mapping between CSS & HTML attributes](https://drafts.csswg.org/css-tables-3/#mapping)

- [Edge does not account for widths of spanned cells](http://codepen.io/FremyCompany/pen/RrMvPR?editors=1100)

  Chrome and Firefox merge the two columns so are not affected.  
  Edge does not then is confused about what to do.  
    
  The specs says that Edge is wrong.  
  [§ 3.3 Dimensioning the row/column grid](https://drafts.csswg.org/css-tables-3/#dimensioning-the-row-column-grid)

- [Chrome and Gecko do not apply display:table-cell on \<button\>](https://jsfiddle.net/6L94pxgn/1/#6188753)

  Also [&#64;lt;fieldset&#62;](https://jsfiddle.net/6L94pxgn/3/). This specification says that Chrome and Firefox are wrong.  
  [§ 3.6 Style overrides](https://drafts.csswg.org/css-tables-3/#style-overrides)

- [Edge’s table-cell width unexplainably low due to percentage-max-width on content and colspan](http://codepen.io/FremyCompany/pen/xZWmXy?editors=1100)

  Edge has issues with percentage widths.  
    
  This specification should say that Chrome and Firefox are right, I guess.  
  [§ 3.8.2 Computing Cell Measures](#computing-cell-measures)

- [Table wrapper boxes should be wide enough to contain the caption](http://codepen.io/FremyCompany/pen/PZRdzo?editors=1100)

  Chrome uses the CSS21 algorithm that fulfill this.  
  Firefox and Edge use David Baron’s algorithm that doesn’t.  
    
  This specification has no opinion yet on the matter.  
  Chrome behavior is simpler to spec but not intuitive.  
  [§ 3.9.1 Computing the table width](#computing-the-table-width)  
    
  This may be solved by [www-list](https://lists.w3.org/Archives/Public/www-style/2010Sep/0186.html) on the width of the table boxes in relation to their table captions

- [Tables containing no row cannot have height in Chrome, but can in Edge/Firefox](http://codepen.io/FremyCompany/pen/VeXVwz?editors=1100)

  Chrome has table height = 0px.  
  Firefox and Edge have table height = specified height.  
    
  This specification has no opinion yet on the matter.  
  Firefox and Edge match author intentions better.  
  [§ 3.10.1 Computing the table height](#computing-the-table-height)

- [Table-layout:fixed causes different width distribution in Chrome](http://codepen.io/FremyCompany/pen/yeKRGW?editors=1100)

  Chrome starts to value px then percentages.  
  Others continue to value percentage then px.  
    
  This specification has no opinion yet on the matter.  
  It is intended to specify the Edge/Firefox behavior.  
  [§ 3.9 Available Width Distribution](#width-distribution)

- [Chrome distributes the height to each row differently then IE and firefox](http://codepen.io/FremyCompany/pen/XXEoWe?editors=1100)

  Chrome distributes rather equally.  
  Edge and Firefox work more similarly to width distribution.  
    
  This specification has no opinion yet on the matter.  
  I think it makes sense to behave similarly for width and height, though.  
  [§ 3.10 Available Height Distribution](#height-distribution)

- [Height of rows which can overflow varies in Chrome vs Edge (percentage heights during min-height computation)](http://codepen.io/FremyCompany/pen/PZRdOP?editors=1100)

  Chrome chooses table constraints over the no-scroll constraint.  
  Edge and Firefox do the opposite.  
    
  This specification has no opinion yet on the matter.  
  Edge’s behavior is right by default due to CSS Sizing, though.  
  It would make sense to do what Chrome is doing when elements have non-visible overflow-y.  
  [§ 3.10 Available Height Distribution](#height-distribution) [\#REF](#min-content-and-percentages)

- [Height specified on row groups is not interoperable](http://codepen.io/FremyCompany/pen/MKVPGR?editors=1100)

  Chrome ignores the height.  
  Firefox respects the height.  
  IE applies the height to each cell.  
    
  This specification has no opinion yet on the matter.  
  An author would expect the Firefox behavior.  
  [§ 3.10 Available Height Distribution](#height-distribution)

- [Min-Height specified on rows is not interoperable](http://codepen.io/FremyCompany/pen/MKVPGR?editors=1100)

  Chrome and Firefox ignore the height.  
  Edge applies the height.  
    
  The working group resolved on Edge behavior.  
  An author would expect the Edge behavior.  
  [§ 3.10 Available Height Distribution](#height-distribution)

- [Table with interleaved td\[rowspan\] rendered wrong in IE (145069)](http://jsfiddle.net/ArtyomShegeda/Ffjhn/1/)

  Edge height-sizing has issues with this crossed-rowspan table.  
    
  There is no doubt this is a bug, but it is a good test case for the algorithm.  
    
  [§ 3.10 Available Height Distribution](#height-distribution)

- [Row with explicit 'visibility:visible' lose background color when parent table has 'visibility: hidden'](http://codepen.io/FremyCompany/pen/MKVqXQ?editors=1100)

  Chrome only hides table background + border.  
  Firefox hides table background + border, and row-group background.  
  Edge hides every border + table/row-group/row background.  
    
  This specification currently says that Chrome is right.  
  [§ 5.3.2 Drawing cell backgrounds](https://drafts.csswg.org/css-tables-3/#drawing-cell-backgrounds)  
  <a id="issue-2d70544e"></a>
  <strong>Issue:</strong>

  [](#issue-2d70544e) Should we hide the row-group background by saying cells only draw the backgrounds of visibility:visible grouping elements?
