Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Tables](https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Tables

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html

Snapshot SHA-256: 201812dd6e3c9ce9133afbc8c5ac576c7033514b6748f764aa111cc563036dc1

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

<a id="q17.0"></a>

# 17 Tables

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

<a id="tables-intro"></a>

## 17.1 Introduction to tables

This chapter defines the processing model for tables in CSS. Part of this processing model is the layout. For the layout, this chapter introduces two algorithms; the first, the fixed table layout algorithm, is well-defined, but the second, the automatic table layout algorithm, is not fully defined by this specification.

For the automatic table layout algorithm, some widely deployed implementations have achieved relatively close interoperability.

<a id="x0"></a>

Table layout can be used to represent tabular relationships between data. Authors specify these relationships in the [document language](css2--conform.html--de58593b67d7.md#doclanguage) and can specify their <em>presentation</em> using CSS 2.1.

In a visual medium, CSS tables can also be used to achieve specific layouts. In this case, authors should not use table-related elements in the document language, but should apply the CSS to the relevant structural elements to achieve the desired layout.

Authors may specify the visual formatting of a table as a rectangular grid of cells. Rows and columns of cells may be organized into row groups and column groups. Rows, columns, row groups, column groups, and cells may have borders drawn around them (there are two border models in CSS 2.1). Authors may align data vertically or horizontally within a cell and align data in all cells of a row or column.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Here is a simple three-row, three-column table described in HTML 4:
>
> ```text
> 
> <TABLE>
> <CAPTION>This is a simple 3x3 table</CAPTION>
> <TR id="row1">
>    <TH>Header 1  <TD>Cell 1  <TD>Cell 2
> <TR id="row2">
>    <TH>Header 2  <TD>Cell 3  <TD>Cell 4
> <TR id="row3">
>    <TH>Header 3  <TD>Cell 5  <TD>Cell 6
> </TABLE>
> ```
>
> This code creates one table (the TABLE element), three rows (the TR elements), three header cells (the TH elements), and six data cells (the TD elements). Note that the three columns of this example are specified implicitly: there are as many columns in the table as required by header and data cells.
>
> The following CSS rule centers the text horizontally in the header cells and presents the text in the header cells with a bold font weight:
>
> ```text
> 
> th { text-align: center; font-weight: bold }
> ```
>
> The next rules align the text of the header cells on their baseline and vertically center the text in each data cell:
>
> ```text
> 
> th { vertical-align: baseline }
> td { vertical-align: middle }
> ```
>
> The next rules specify that the top row will be surrounded by a 3px solid blue border and each of the other rows will be surrounded by a 1px solid black border:
>
> ```text
> 
> table   { border-collapse: collapse }
> tr#row1 { border: 3px solid blue }
> tr#row2 { border: 1px solid black }
> tr#row3 { border: 1px solid black }
> ```
>
> Note, however, that the borders around the rows overlap where the rows meet. What color (black or blue) and thickness (1px or 3px) will the border between row1 and row2 be? We discuss this in the section on [border conflict resolution.](#border-conflict-resolution)
>
> The following rule puts the table caption above the table:
>
> ```text
> 
> caption { caption-side: top }
> ```
<a id="x1"></a>

The preceding example shows how CSS works with HTML 4 elements; in HTML 4, the semantics of the various table elements (TABLE, CAPTION, THEAD, TBODY, TFOOT, COL, COLGROUP, TH, and TD) are well-defined. In other document languages (such as XML applications), there may not be pre-defined table elements. Therefore, CSS 2.1 allows authors to "map" document language elements to table elements via the ['display'](css2--visuren.html--3f334c530cf4.md#propdef-display) property. For example, the following rule makes the FOO element act like an HTML TABLE element and the BAR element act like a CAPTION element:

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> 
> FOO { display : table }
> BAR { display : table-caption }
> ```
<a id="internal"></a>

<a id="x2"></a>

<a id="x3"></a>

<a id="internal-table-element"></a>

We discuss the various table elements in the following section. In this specification, the term table element refers to any element involved in the creation of a table. An internal table element is one that produces a row, row group, column, column group, or cell.

<a id="table-display"></a>

## 17.2 The CSS table model

The CSS table model is based on the HTML4 table model, in which the structure of a table closely parallels the visual layout of the table. In this model, a table consists of an optional caption and any number of rows of cells. The table model is said to be "row primary" since authors specify rows, not columns, explicitly in the document language. Columns are derived once all the rows have been specified -- the first cell of each row belongs to the first column, the second to the second column, etc.). Rows and columns may be grouped structurally and this grouping reflected in presentation (e.g., a border may be drawn around a group of rows).

<a id="x5"></a>

Thus, the table model consists of tables, captions, rows, row groups (including header groups and footer groups), columns, column groups, and cells.

The CSS model does not require that the [document language](css2--conform.html--de58593b67d7.md#doclanguage) include elements that correspond to each of these components. For document languages (such as XML applications) that do not have pre-defined table elements, authors must map document language elements to table elements; this is done with the ['display'](css2--visuren.html--3f334c530cf4.md#propdef-display) property. The following ['display'](css2--visuren.html--3f334c530cf4.md#propdef-display) values assign table formatting rules to an arbitrary element:

<a id="value-def-table"></a>

<strong><span title="table"><a>table</a></span></strong> (In HTML: TABLE)

Specifies that an element defines a [block-level](css2--visuren.html--3f334c530cf4.md#block-level) table: it is a rectangular block that participates in a [block formatting context](css2--visuren.html--3f334c530cf4.md#block-formatting).

<a id="value-def-inline-table"></a>

<strong><span title="inline-table"><a>inline-table</a></span></strong> (In HTML: TABLE)

Specifies that an element defines an [inline-level](css2--visuren.html--3f334c530cf4.md#inline-level) table: it is a rectangular block that participates in an [inline formatting context](css2--visuren.html--3f334c530cf4.md#inline-formatting)).

<a id="value-def-table-row"></a>

<strong><span title="table-row"><a>table-row</a></span></strong> (In HTML: TR)

Specifies that an element is a row of cells.

<a id="value-def-table-row-group"></a>

<strong><span title="table-row-group"><a>table-row-group</a></span></strong> (In HTML: TBODY)

Specifies that an element groups one or more rows.

<a id="value-def-table-header-group"></a>

<strong><span title="table-header-group"><a>table-header-group</a></span></strong> (In HTML: THEAD)

Like 'table-row-group', but for visual formatting, the row group is always displayed before all other rows and row groups and after any top captions. Print user agents may repeat header rows on each page spanned by a table. If a table contains multiple elements with 'display: table-header-group', only the first is rendered as a header; the others are treated as if they had 'display: table-row-group'.

<a id="value-def-table-footer-group"></a>

<strong><span title="table-footer-group"><a>table-footer-group</a></span></strong> (In HTML: TFOOT)

Like 'table-row-group', but for visual formatting, the row group is always displayed after all other rows and row groups and before any bottom captions. Print user agents may repeat footer rows on each page spanned by a table. If a table contains multiple elements with 'display: table-footer-group', only the first is rendered as a footer; the others are treated as if they had 'display: table-row-group'.

<a id="value-def-table-column"></a>

<strong><span title="table-column"><a>table-column</a></span></strong> (In HTML: COL)

Specifies that an element describes a column of cells.

<a id="value-def-table-column-group"></a>

<strong><span title="table-column-group"><a>table-column-group</a></span></strong> (In HTML: COLGROUP)

Specifies that an element groups one or more columns.

<a id="value-def-table-cell"></a>

<strong><span title="table-cell"><a>table-cell</a></span></strong> (In HTML: TD, TH)

Specifies that an element represents a table cell.

<a id="value-def-table-caption"></a>

<strong><span title="table-caption"><a>table-caption</a></span></strong> (In HTML: CAPTION)

Specifies a caption for the table. All elements with 'display: table-caption' must be rendered, as described in [section 17.4.](#model)

Replaced elements with these ['display'](css2--visuren.html--3f334c530cf4.md#propdef-display) values are treated as their given display types during layout. For example, an image that is set to 'display: table-cell' will fill the available cell space, and its dimensions might contribute towards the table sizing algorithms, as with an ordinary cell.

Elements with ['display'](css2--visuren.html--3f334c530cf4.md#propdef-display) set to 'table-column' or 'table-column-group' are not rendered (exactly as if they had 'display: none'), but they are useful, because they may have attributes which induce a certain style for the columns they represent.

The [default style sheet for HTML4](css2--sample.html--133769fd2555.md) in the appendix illustrates the use of these values for HTML4:

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> 
> table    { display: table }
> tr       { display: table-row }
> thead    { display: table-header-group }
> tbody    { display: table-row-group }
> tfoot    { display: table-footer-group }
> col      { display: table-column }
> colgroup { display: table-column-group }
> td, th   { display: table-cell }
> caption  { display: table-caption }
> ```
User agents may [ignore](css2--syndata.html--02e71c159e14.md#ignore) these ['display'](css2--visuren.html--3f334c530cf4.md#propdef-display) property values for HTML table elements, since HTML tables may be rendered using other algorithms intended for backwards compatible rendering. However, this is not meant to discourage the use of 'display: table' on other, non-table elements in HTML.

<a id="anonymous-boxes"></a>

### 17.2.1 Anonymous table objects

Document languages other than HTML may not contain all the elements in the CSS 2.1 table model. In these cases, the "missing" elements must be assumed in order for the table model to work. Any table element will automatically generate necessary anonymous table objects around itself, consisting of at least three nested objects corresponding to a 'table'/'inline-table' element, a 'table-row' element, and a 'table-cell' element. Missing elements generate [anonymous](css2--visuren.html--3f334c530cf4.md#anonymous) objects (e.g., anonymous boxes in visual table layout) according to the following rules:

For the purposes of these rules, the following terms are defined:

<a id="x16"></a>

row group box

A 'table-row-group', 'table-header-group', or 'table-footer-group'

<a id="x17"></a>

proper table child

A 'table-row' box, row group box, 'table-column' box, 'table-column-group' box, or 'table-caption' box.

<a id="x18"></a>

proper table row parent

A 'table' or 'inline-table' box or row group box

<a id="x19"></a>

internal table box

A 'table-cell' box, 'table-row' box, row group box, 'table-column' box, or 'table-column-group' box.

<a id="x20"></a>

<a id="tabular-container"></a>tabular container

A 'table-row' box or proper table row parent

<a id="x21"></a>

consecutive

Two sibling boxes are consecutive if they have no intervening siblings other than, optionally, an anonymous inline containing only white spaces. A sequence of sibling boxes is consecutive if each box in the sequence is consecutive to the one before it in the sequence.

For the purposes of these rules, out-of-flow elements are represented as inline elements of zero width and height. Their containing blocks are chosen accordingly.

The following steps are performed in three stages.

1.  Remove irrelevant boxes:
    1.  All child boxes of a 'table-column' parent are treated as if they had 'display: none'.
    2.  If a child <var>C</var> of a 'table-column-group' parent is not a 'table-column' box, then it is treated as if it had 'display: none'.
    3.  If a child <var>C</var> of a tabular container <var>P</var> is an anonymous inline box that contains only white space, and its immediately preceding and following siblings, if any, are proper table descendants of <var>P</var> and are either 'table-caption' or internal table boxes, then it is treated as if it had 'display: none'. A box <var>D</var> is a proper table descendant of <var>A</var> if <var>D</var> can be a descendant of <var>A</var> without causing the generation of any intervening 'table' or 'inline-table' boxes.
    4.  If a box <var>B</var> is an anonymous inline containing only white space, and is between two immediate siblings each of which is either an internal table box or a 'table-caption' box then <var>B</var> is treated as if it had 'display: none'.
2.  Generate missing child wrappers:
    1.  If a child <var>C</var> of a 'table' or 'inline-table' box is not a proper table child, then generate an anonymous 'table-row' box around <var>C</var> and all consecutive siblings of <var>C</var> that are not proper table children.
    2.  If a child <var>C</var> of a row group box is not a 'table-row' box, then generate an anonymous 'table-row' box around <var>C</var> and all consecutive siblings of <var>C</var> that are not 'table-row' boxes.
    3.  If a child <var>C</var> of a 'table-row' box is not a 'table-cell', then generate an anonymous 'table-cell' box around <var>C</var> and all consecutive siblings of <var>C</var> that are not 'table-cell' boxes.
3.  Generate missing parents:
    1.  For each 'table-cell' box <var>C</var> in a sequence of consecutive internal table and 'table-caption' siblings, if <var>C</var>'s parent is not a 'table-row' then generate an anonymous 'table-row' box around <var>C</var> and all consecutive siblings of <var>C</var> that are 'table-cell' boxes.
    2.  For each proper table child <var>C</var> in a sequence of consecutive proper table children, if <var>C</var> is misparented then generate an anonymous 'table' or 'inline-table' box <var>T</var> around <var>C</var> and all consecutive siblings of <var>C</var> that are proper table children. (If C's parent is an 'inline' box, then <var>T</var> must be an 'inline-table' box; otherwise it must be a 'table' box.)
        - A 'table-row' is misparented if its parent is neither a row group box nor a 'table' or 'inline-table' box.
        - A 'table-column' box is misparented if its parent is neither a 'table-column-group' box nor a 'table' or 'inline-table' box.
        - A row group box, 'table-column-group' box, or 'table-caption' box is misparented if its parent is neither a 'table' box nor an 'inline-table' box.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In this XML example, a 'table' element is assumed to contain the HBOX element:
>
> ```text
> 
> <HBOX>
>   <VBOX>George</VBOX>
>   <VBOX>4287</VBOX>
>   <VBOX>1998</VBOX>
> </HBOX>
> ```
>
> because the associated style sheet is:
>
> ```text
> 
> HBOX { display: table-row }
> VBOX { display: table-cell }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In this example, three 'table-cell' elements are assumed to contain the text in the ROWs. Note that the text is further encapsulated in anonymous inline boxes, as explained in [visual formatting model](css2--visuren.html--3f334c530cf4.md#anonymous):
>
> ```text
> 
> <STACK>
>   <ROW>This is the <D>top</D> row.</ROW>
>   <ROW>This is the <D>middle</D> row.</ROW>
>   <ROW>This is the <D>bottom</D> row.</ROW>
> </STACK>
> ```
>
> The style sheet is:
>
> ```text
> 
> STACK { display: inline-table }
> ROW   { display: table-row }
> D     { display: inline; font-weight: bolder }
> ```
<a id="columns"></a>

## 17.3 Columns

Table cells may belong to two contexts: rows and columns. However, in the source document cells are descendants of rows, never of columns. Nevertheless, some aspects of cells can be influenced by setting properties on columns.

The following properties apply to column and column-group elements:

['border'](css2--box.html--8875bbcdefe6.md#propdef-border)  
The various border properties apply to columns only if ['border-collapse'](css2--tables.html--201812dd6e3c.md#propdef-border-collapse) is set to 'collapse' on the table element. In that case, borders set on columns and column groups are input to the [conflict resolution algorithm](#border-conflict-resolution) that selects the border styles at every cell edge.

['background'](css2--colors.html--5784063d2778.md#propdef-background)  
The background properties set the background for cells in the column, but only if both the cell and row have transparent backgrounds. See ["Table layers and transparency."](#table-layers)

['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width)  
The ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) property gives the minimum width for the column.

['visibility'](css2--visufx.html--2bc674cb7ab0.md#propdef-visibility)  
If the 'visibility' of a column is set to 'collapse', none of the cells in the column are rendered, and cells that span into other columns are clipped. In addition, the width of the table is diminished by the width the column would have taken up. See ["Dynamic effects"](#dynamic-effects) below. Other values for 'visibility' have no effect.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Here are some examples of style rules that set properties on columns. The first two rules together implement the "rules" attribute of HTML 4 with a value of "cols". The third rule makes the "totals" column blue, the final two rules shows how to make a column a fixed size, by using the [fixed layout algorithm](#fixed-table-layout).
>
> ```text
> 
> col { border-style: none solid }
> table { border-style: hidden }
> col.totals { background: blue }
> table { table-layout: fixed }
> col.totals { width: 5em }
> ```
<a id="model"></a>

## 17.4 Tables in the visual formatting model

In terms of the visual formatting model, a table can behave like a [block-level](css2--visuren.html--3f334c530cf4.md#block-level) (for 'display: table') or [inline-level](css2--visuren.html--3f334c530cf4.md#inline-level) (for 'display: inline-table') element.

In both cases, the table generates a principal block box called the table wrapper box that contains the table box itself and any caption boxes (in document order). The table box is a block-level box that contains the table's internal table boxes. The caption boxes are block-level boxes that retain their own content, padding, margin, and border areas, and are rendered as normal block boxes inside the table wrapper box. Whether the caption boxes are placed before or after the table box is decided by the 'caption-side' property, as described below.

The table wrapper box is a 'block' box if the table is block-level, and an 'inline-block' box if the table is inline-level. The table wrapper box establishes a block formatting context. The table box (not the table wrapper box) is used when doing baseline vertical alignment for an 'inline-table'. The width of the table wrapper box is the border-edge width of the table box inside it, as described by section 17.5.2. Percentages on 'width' and 'height' on the table are relative to the table wrapper box's containing block, not the table wrapper box itself.

The computed values of properties 'position', 'float', 'margin-\*', 'top', 'right', 'bottom', and 'left' on the table element are used on the table wrapper box and not the table box; all other values of non-inheritable properties are used on the table box and not the table wrapper box. (Where the table element's values are not used on the table and table wrapper boxes, the initial values are used instead.)

![A table with a caption above it](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/table_container.png)

Diagram of a table with a caption above it.

<a id="caption-position"></a>

### 17.4.1 Caption position and alignment

<a id="propdef-caption-side"></a>

<strong>'caption-side'</strong>

|                       |                                                                                                         |
|-----------------------|---------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | top \| bottom \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | top                                                                                                     |
| <em>Applies to:</em>   | 'table-caption' elements                                                                                |
| <em>Inherited:</em>   | yes                                                                                                     |
| <em>Percentages:</em>   | N/A                                                                                                     |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                    |
| <em>Computed value:</em>   | as specified                                                                                            |

This property specifies the position of the caption box with respect to the table box. Values have the following meanings:

<strong>top</strong>  
Positions the caption box above the table box.

<strong>bottom</strong>  
Positions the caption box below the table box.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note:</strong> CSS2 described a different width and
horizontal alignment behavior. That behavior will be introduced in
CSS3 using the values 'top-outside' and 'bottom-outside' on this
property.</em>

To align caption content horizontally within the caption box, use the ['text-align'](css2--text.html--467a8857ae69.md#propdef-text-align) property.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In this example, the ['caption-side'](css2--tables.html--201812dd6e3c.md#propdef-caption-side) property places captions below tables. The caption will be as wide as the parent of the table, and caption text will be left-justified.
>
> ```text
> 
> caption { caption-side: bottom; 
>           width: auto;
>           text-align: left }
> ```
<a id="table-layout"></a>

## 17.5 Visual layout of table contents

Internal table elements generate rectangular [boxes](css2--box.html--8875bbcdefe6.md#box-dimensions) with content and borders. Cells have padding as well. Internal table elements do not have margins.

The visual layout of these boxes is governed by a rectangular, irregular grid of rows and columns. Each box occupies a whole number of grid cells, determined according to the following rules. These rules do not apply to HTML 4 or earlier HTML versions; HTML imposes its own limitations on row and column spans.

1.  Each row box occupies one row of grid cells. Together, the row boxes fill the table from top to bottom in the order they occur in the source document (i.e., the table occupies exactly as many grid rows as there are row elements).
2.  A row group occupies the same grid cells as the rows it contains.
3.  A column box occupies one or more columns of grid cells. Column boxes are placed next to each other in the order they occur. The first column box may be either on the left or on the right, depending on the value of the ['direction'](css2--visuren.html--3f334c530cf4.md#propdef-direction) property of the table.
4.  A column group box occupies the same grid cells as the columns it contains.
5.  Cells may span several rows or columns. (Although CSS 2.1 does not define how the number of spanned rows or columns is determined, a user agent may have special knowledge about the source document; a future update of CSS may provide a way to express this knowledge in CSS syntax.) Each cell is thus a rectangular box, one or more grid cells wide and high. The top row of this rectangle is in the row specified by the cell's parent. The rectangle must be as far to the left as possible, but the part of the cell in the first column it occupies must not overlap with any other cell box (i.e., a row-spanning cell starting in a prior row), and the cell must be to the right of all cells in the same row that are earlier in the source document. If this position would cause a column-spanning cell to overlap a row-spanning cell from a prior row, CSS does not define the results: implementations may either overlap the cells (as is done in many HTML implementations) or may shift the later cell to the right to avoid such overlap. (This constraint holds if the 'direction' property of the table is 'ltr'; if the 'direction' is 'rtl', interchange "left" and "right" in the previous two sentences.)
6.  A cell box cannot extend beyond the last row box of a table or row group; the user agents must shorten it until it fits.

The edges of the rows, columns, row groups and column groups in the [collapsing borders model](#collapsing-borders) coincide with the hypothetical grid lines on which the borders of the cells are centered. (And thus, in this model, the rows together exactly cover the table, leaving no gaps; ditto for the columns.) In the [separated borders model,](#separated-borders) the edges coincide with the [border edges](css2--box.html--8875bbcdefe6.md#border-edge) of cells. (And thus, in this model, there may be gaps between the rows, columns, row groups or column groups, corresponding to the ['border-spacing'](css2--tables.html--201812dd6e3c.md#propdef-border-spacing) property.)

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> Positioning and floating of table cells
can cause them not to be table cells anymore, according to the rules
in <a href="css2--visuren.html--3f334c530cf4.md#dis-pos-flo">section 9.7</a>. When floating
is used, the <a>rules on anonymous table objects</a> may cause an
anonymous cell object to be created as well.</em>

Here is an example illustrating rule 5. The following illegal (X)HTML snippet defines conflicting cells:

```text

<table>
<tr><td>1 </td><td rowspan="2">2 </td><td>3 </td><td>4 </td></tr>
<tr><td colspan="2">5 </td></tr>
</table>
```
User agents are free to visually overlap the cells, as in the figure on the left, or to shift the cell to avoid the visual overlap, as in the figure on the right.

<a id="img-table-overlap"></a>

![One table with overlapping cells and one without](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/table-overlap.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/table-overlap-desc.html)

Two possible renderings of an erroneous HTML table.

<a id="table-layers"></a>

### 17.5.1 Table layers and transparency

For the purposes of finding the background of each table cell, the different table elements may be thought of as being on six superimposed layers. The background set on an element in one of the layers will only be visible if the layers above it have a transparent background.

<a id="img-tbl-layers"></a>

![schema of table layers](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/tbl-layers.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/tbl-layers-desc.html)

Schema of table layers.

1.  The lowest layer is a single plane, representing the table box itself. Like all boxes, it may be transparent.
2.  The next layer contains the column groups. Each column group extends from the top of the cells in the top row to the bottom of the cells on the bottom row and from the left edge of its leftmost column to the right edge of its rightmost column. The background covers exactly the full area of all cells that originate in the column group, even if they span outside the column group, but this difference in area does not affect background image positioning.
3.  On top of the column groups are the areas representing the column boxes. Each column is as tall as the column groups and as wide as a normal (single-column-spanning) cell in the column. The background covers exactly the full area of all cells that originate in the column, even if they span outside the column, but this difference in area does not affect background image positioning.
4.  Next is the layer containing the row groups. Each row group extends from the top left corner of its topmost cell in the first column to the bottom right corner of its bottommost cell in the last column.
5.  The next to last layer contains the rows. Each row is as wide as the row groups and as tall as a normal (single-row-spanning) cell in the row. As with columns, the background covers exactly the full area of all cells that originate in the row, even if they span outside the row, but this difference in area does not affect background image positioning.
6.  The topmost layer contains the cells themselves. As the figure shows, although all rows contain the same number of cells, not every cell may have specified content. In the [separated borders model](#separated-borders) (['border-collapse'](css2--tables.html--201812dd6e3c.md#propdef-border-collapse) is 'separate'), if the value of their ['empty-cells'](css2--tables.html--201812dd6e3c.md#propdef-empty-cells) property is 'hide' these "empty" cells are transparent through the cell, row, row group, column and column group backgrounds, letting the table background show through.

A "missing cell" is a cell in the row/column grid that is not occupied by an element or pseudo-element. Missing cells are rendered as if an anonymous table-cell box occupied their position in the grid.

In the following example, the first row contains four non-empty cells, but the second row contains only one non-empty cell, and thus the table background shines through, except where a cell from the first row spans into this row. The following HTML code and style rules

```text

<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN">
<HTML>
  <HEAD>
    <TITLE>Table example</TITLE>
    <STYLE type="text/css">
      TABLE  { background: #ff0; border: solid black;
               empty-cells: hide }
      TR.top { background: red }
      TD     { border: solid black }
    </STYLE>
  </HEAD>
  <BODY>
    <TABLE>
      <TR CLASS="top">
        <TD> 1 
        <TD rowspan="2"> 2
        <TD> 3 
        <TD> 4 
      <TR>
        <TD> 5
        <TD>
    </TABLE> 
  </BODY>
</HTML>
```
might be formatted as follows:

<a id="img-tbl-empty"></a>

![Table with three empty cells in bottom row](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/tbl-empty.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/tbl-empty-desc.html)

Table with empty cells in the bottom row.

Note that if the table has 'border-collapse: separate', the background of the area given by the ['border-spacing'](css2--tables.html--201812dd6e3c.md#propdef-border-spacing) property is always the background of the table element. See [the separated borders model](#separated-borders).

<a id="width-layout"></a>

### 17.5.2 Table width algorithms: the ['table-layout'](css2--tables.html--201812dd6e3c.md#propdef-table-layout) property

CSS does not define an "optimal" layout for tables since, in many cases, what is optimal is a matter of taste. CSS does define constraints that user agents must respect when laying out a table. User agents may use any algorithm they wish to do so, and are free to prefer rendering speed over precision, except when the "fixed layout algorithm" is selected.

Note that this section overrides the rules that apply to calculating widths as described in [section 10.3](css2--visudet.html--12e8bc0e6b7c.md#Computing_widths_and_margins). In particular, if the margins of a table are set to '0' and the width to 'auto', the table will not automatically size to fill its containing block. However, once the calculated value of 'width' for the table is found (using the algorithms given below or, when appropriate, some other UA dependent algorithm) then the other parts of section 10.3 do apply. Therefore a table <em>can</em> be centered using left and right 'auto' margins, for instance.

Future updates of CSS may introduce ways of making tables automatically fit their containing blocks.

<a id="propdef-table-layout"></a>

<strong>'table-layout'</strong>

|                       |                                                                                                         |
|-----------------------|---------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | auto \| fixed \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | auto                                                                                                    |
| <em>Applies to:</em>   | 'table' and 'inline-table' elements                                                                     |
| <em>Inherited:</em>   | no                                                                                                      |
| <em>Percentages:</em>   | N/A                                                                                                     |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                    |
| <em>Computed value:</em>   | as specified                                                                                            |

The ['table-layout'](css2--tables.html--201812dd6e3c.md#propdef-table-layout) property controls the algorithm used to lay out the table cells, rows, and columns. Values have the following meaning:

<strong>fixed</strong>  
Use the fixed table layout algorithm

<strong>auto</strong>  
Use any automatic table layout algorithm

The two algorithms are described below.

<a id="fixed-table-layout"></a>

#### 17.5.2.1 Fixed table layout

With this (fast) algorithm, the horizontal layout of the table does not depend on the contents of the cells; it only depends on the table's width, the width of the columns, and borders or cell spacing.

The table's width may be specified explicitly with the ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) property. A value of 'auto' (for both 'display: table' and 'display: inline-table') means use the [automatic table layout](#auto-table-layout) algorithm. However, if the table is a block-level table ('display: table') in normal flow, a UA may (but does not have to) use the algorithm of [10.3.3](css2--visudet.html--12e8bc0e6b7c.md#blockwidth) to compute a width and apply fixed table layout even if the specified width is 'auto'.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> If a UA supports fixed table layout when 'width' is 'auto', the following will create a table that is 4em narrower than its containing block:
>
> ```text
> 
> table { table-layout: fixed;
>         margin-left: 2em;
>         margin-right: 2em }
> ```
In the fixed table layout algorithm, the width of each column is determined as follows:

1.  A column element with a value other than 'auto' for the ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) property sets the width for that column.
2.  Otherwise, a cell in the first row with a value other than 'auto' for the ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) property determines the width for that column. If the cell spans more than one column, the width is divided over the columns.
3.  Any remaining columns equally divide the remaining horizontal table space (minus borders or cell spacing).

The width of the table is then the greater of the value of the ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) property for the table element and the sum of the column widths (plus cell spacing or borders). If the table is wider than the columns, the extra space should be distributed over the columns.

If a subsequent row has more columns than the greater of the number determined by the table-column elements and the number determined by the first row, then additional columns may not be rendered. CSS 2.1 does not define the width of the columns and the table if they <em>are</em> rendered. When using 'table-layout: fixed', authors should not omit columns from the first row.

In this manner, the user agent can begin to lay out the table once the entire first row has been received. Cells in subsequent rows do not affect column widths. Any cell that has content that overflows uses the ['overflow'](css2--visufx.html--2bc674cb7ab0.md#propdef-overflow) property to determine whether to clip the overflow content.

<a id="auto-table-layout"></a>

#### 17.5.2.2 Automatic table layout

In this algorithm (which generally requires no more than two passes), the table's width is given by the width of its columns (and intervening [borders](#borders)). This algorithm reflects the behavior of several popular HTML user agents at the writing of this specification. UAs are not required to implement this algorithm to determine the table layout in the case that ['table-layout'](css2--tables.html--201812dd6e3c.md#propdef-table-layout) is 'auto'; they can use any other algorithm even if it results in different behavior.

Input to the automatic table layout must only include the width of the containing block and the content of, and any CSS properties set on, the table and any of its descendants.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> This may be defined in more detail in
CSS3.</em>

<em>The remainder of this section is non-normative.</em>

This algorithm may be inefficient since it requires the user agent to have access to all the content in the table before determining the final layout and may demand more than one pass.

Column widths are determined as follows:

1.  Calculate the minimum content width (MCW) of each cell: the formatted content may span any number of lines but may not overflow the cell box. If the specified ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) (W) of the cell is greater than MCW, W is the minimum cell width. A value of 'auto' means that MCW is the minimum cell width.

    Also, calculate the "maximum" cell width of each cell: formatting the content without breaking lines other than where explicit line breaks occur.

2.  For each column, determine a maximum and minimum column width from the cells that span only that column. The minimum is that required by the cell with the largest minimum cell width (or the column ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width), whichever is larger). The maximum is that required by the cell with the largest maximum cell width (or the column ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width), whichever is larger).

3.  For each cell that spans more than one column, increase the minimum widths of the columns it spans so that together, they are at least as wide as the cell. Do the same for the maximum widths. If possible, widen all spanned columns by approximately the same amount.

4.  For each column group element with a 'width' other than 'auto', increase the minimum widths of the columns it spans, so that together they are at least as wide as the column group's 'width'.

This gives a maximum and minimum width for each column.

The caption width minimum (CAPMIN) is determined by calculating for each caption the minimum caption outer width as the MCW of a hypothetical table cell that contains the caption formatted as "display: block". The greatest of the minimum caption outer widths is CAPMIN.

Column and caption widths influence the final table width as follows:

1.  If the 'table' or 'inline-table' element's ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) property has a computed value (W) other than 'auto', the used width is the greater of W, CAPMIN, and the minimum width required by all the columns plus cell spacing or borders (MIN). If the used width is greater than MIN, the extra width should be distributed over the columns.
2.  If the 'table' or 'inline-table' element has 'width: auto', the used width is the greater of the table's containing block width, CAPMIN, and MIN. However, if either CAPMIN or the maximum width required by the columns plus cell spacing or borders (MAX) is less than that of the containing block, use max(MAX, CAPMIN).

A percentage value for a column width is relative to the table width. If the table has 'width: auto', a percentage represents a constraint on the column's width, which a UA should try to satisfy. (Obviously, this is not always possible: if the column's width is '110%', the constraint cannot be satisfied.)

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> In this algorithm, rows (and row
groups) and columns (and column groups) both constrain and are
constrained by the dimensions of the cells they contain. Setting the
width of a column may indirectly influence the height of a row, and
vice versa.</em>

<a id="height-layout"></a>

### 17.5.3 Table height algorithms

The height of a table is given by the ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) property for the 'table' or 'inline-table' element. A value of 'auto' means that the height is the sum of the row heights plus any cell spacing or borders. Any other value is treated as a minimum height. CSS 2.1 does not define how extra space is distributed when the 'height' property causes the table to be taller than it otherwise would be.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> Future
updates of CSS may specify this further.</em>

The height of a 'table-row' element's box is calculated once the user agent has all the cells in the row available: it is the maximum of the row's computed ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height), the computed ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) of each cell in the row, and the minimum height (MIN) required by the cells. A ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) value of 'auto' for a 'table-row' means the row height used for layout is MIN. MIN depends on cell box heights and cell box alignment (much like the calculation of a [line box](css2--visudet.html--12e8bc0e6b7c.md#line-height) height). CSS 2.1 does not define how the height of table cells and table rows is calculated when their height is specified using percentage values. CSS 2.1 does not define the meaning of ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) on row groups.

In CSS 2.1, the height of a cell box is the minimum height required by the content. The table cell's ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) property can influence the height of the row (see above), but it does not increase the height of the cell box.

CSS 2.1 does not specify how cells that span more than one row affect row height calculations except that the sum of the row heights involved must be great enough to encompass the cell spanning the rows.

The ['vertical-align'](css2--visudet.html--12e8bc0e6b7c.md#propdef-vertical-align) property of each table cell determines its alignment within the row. Each cell's content has a baseline, a top, a middle, and a bottom, as does the row itself. In the context of tables, values for ['vertical-align'](css2--visudet.html--12e8bc0e6b7c.md#propdef-vertical-align) have the following meanings:

<strong>baseline</strong>  
The baseline of the cell is put at the same height as the baseline of the first of the rows it spans (see below for the definition of baselines of cells and rows).

<strong>top</strong>  
The top of the cell box is aligned with the top of the first row it spans.

<strong>bottom</strong>  
The bottom of the cell box is aligned with the bottom of the last row it spans.

<strong>middle</strong>  
The center of the cell is aligned with the center of the rows it spans.

<strong>sub, super, text-top, text-bottom, &lt;length&gt;,
  &lt;percentage&gt;</strong>  
These values do not apply to cells; the cell is aligned at the baseline instead.

The baseline of a cell is the baseline of the first in-flow [line box](css2--visuren.html--3f334c530cf4.md#line-box) in the cell, or the first in-flow table-row in the cell, whichever comes first. If there is no such line box or table-row, the baseline is the bottom of content edge of the cell box. For the purposes of finding a baseline, in-flow boxes with a scrolling mechanisms (see the ['overflow'](css2--visufx.html--2bc674cb7ab0.md#propdef-overflow) property) must be considered as if scrolled to their origin position. Note that the baseline of a cell may end up below its bottom border, see the [example](#baseline-below) below.

The maximum distance between the top of the cell box and the baseline over all cells that have 'vertical-align: baseline' is used to set the baseline of the row. Here is an example:

<a id="img-cell-align"></a>

![Example of vertically aligning the cells](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/cell-align.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/cell-align-desc.html)

Diagram showing the effect of various values of 'vertical-align' on table cells.

Cell boxes 1 and 2 are aligned at their baselines. Cell box 2 has the largest height above the baseline, so that determines the baseline of the row.

If a row has no cell box aligned to its baseline, the baseline of that row is the bottom content edge of the lowest cell in the row.

To avoid ambiguous situations, the alignment of cells proceeds in the following order:

1.  First the cells that are aligned on their baseline are positioned. This will establish the baseline of the row. Next the cells with 'vertical-align: top' are positioned.
2.  The row now has a top, possibly a baseline, and a provisional height, which is the distance from the top to the lowest bottom of the cells positioned so far. (See conditions on the cell padding below.)
3.  If any of the remaining cells, those aligned at the bottom or the middle, have a height that is larger than the current height of the row, the height of the row will be increased to the maximum of those cells, by lowering the bottom.
4.  Finally the remaining cells are positioned.

Cell boxes that are smaller than the height of the row receive extra top or bottom padding.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="baseline-below"></a>
>
> The cell in this example has a baseline below its bottom border:
>
> ```text
> 
> div { height: 0; overflow: hidden; }
> 
> <table>
>  <tr>
>   <td>
>    <div> Test </div>
>   </td>
>  </tr>
> </table>
> ```
<a id="column-alignment"></a>

### 17.5.4 Horizontal alignment in a column

The horizontal alignment of inline-level content within a cell box can be specified by the value of the ['text-align'](css2--text.html--467a8857ae69.md#propdef-text-align) property on the cell.

<a id="dynamic-effects"></a>

### 17.5.5 Dynamic row and column effects

The ['visibility'](css2--visufx.html--2bc674cb7ab0.md#propdef-visibility) property takes the value 'collapse' for row, row group, column, and column group elements. This value causes the entire row or column to be removed from the display, and the space normally taken up by the row or column to be made available for other content. Contents of spanned rows and columns that intersect the collapsed column or row are clipped. The suppression of the row or column, however, does not otherwise affect the layout of the table. This allows dynamic effects to remove table rows or columns without forcing a re-layout of the table in order to account for the potential change in column constraints.

<a id="borders"></a>

## 17.6 Borders

<a id="x24"></a>

There are two distinct models for setting borders on table cells in CSS. One is most suitable for so-called separated borders around individual cells, the other is suitable for borders that are continuous from one end of the table to the other. Many border styles can be achieved with either model, so it is often a matter of taste which one is used.

<a id="propdef-border-collapse"></a>

<strong>'border-collapse'</strong>

|                       |                                                                                                                |
|-----------------------|----------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | collapse \| separate \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | separate                                                                                                       |
| <em>Applies to:</em>   | 'table' and 'inline-table' elements                                                                            |
| <em>Inherited:</em>   | yes                                                                                                            |
| <em>Percentages:</em>   | N/A                                                                                                            |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                           |
| <em>Computed value:</em>   | as specified                                                                                                   |

This property selects a table's border model. The value 'separate' selects the separated borders border model. The value 'collapse' selects the collapsing borders model. The models are described below.

<a id="separated-borders"></a>

### 17.6.1 The separated borders model

<a id="propdef-border-spacing"></a>

<strong>'border-spacing'</strong>

|                       |                                                                                                                                                                                                                                                                              |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length)? \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 0                                                                                                                                                                                                                                                                            |
| <em>Applies to:</em>   | 'table' and 'inline-table' elements\*                                                                                                                                                                                                                                        |
| <em>Inherited:</em>   | yes                                                                                                                                                                                                                                                                          |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                                                                                          |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                         |
| <em>Computed value:</em>   | two absolute lengths                                                                                                                                                                                                                                                         |

> <strong data-conversion-semantic="note">Note</strong>
>
> \*) Note: user agents may also apply the 'border-spacing' property to 'frameset' elements. Which elements are 'frameset' elements is not defined by this specification and is up to the document language. For example, HTML4 defines a \<FRAMESET\> element, and XHTML 1.0 defines a \<frameset\> element. The 'border-spacing' property on a 'frameset' element can be thus used as a valid substitute for the non-standard 'framespacing' attribute.

The lengths specify the distance that separates adjoining cell borders. If one length is specified, it gives both the horizontal and vertical spacing. If two are specified, the first gives the horizontal spacing and the second the vertical spacing. Lengths may not be negative.

The distance between the table border and the borders of the cells on the edge of the table is the table's padding for that side, plus the relevant border spacing distance. For example, on the right hand side, the distance is <var>padding-right</var> + <var>horizontal
border-spacing</var>.

The width of the table is the distance from the left inner padding edge to the right inner padding edge (including the border spacing but excluding padding and border).

However, in HTML and XHTML1, the width of the \<table\> element is the distance from the left border edge to the right border edge.

> <strong data-conversion-semantic="note">Note</strong>
>
> <strong>Note:</strong> In CSS3 this peculiar requirement will be defined in terms of UA style sheet rules and the 'box-sizing' property.

<a id="x27"></a>

In this model, each cell has an individual border. The ['border-spacing'](css2--tables.html--201812dd6e3c.md#propdef-border-spacing) property specifies the distance between the borders of adjoining cells. In this space, the row, column, row group, and column group backgrounds are invisible, allowing the table background to show through. Rows, columns, row groups, and column groups cannot have borders (i.e., user agents must [ignore](css2--syndata.html--02e71c159e14.md#ignore) the border properties for those elements).

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> The table in the figure below could be the result of a style sheet like this:
>
> ```text
> 
> table      { border: outset 10pt; 
>              border-collapse: separate;
>              border-spacing: 15pt }
> td         { border: inset 5pt }
> td.special { border: inset 10pt }  /* The top-left cell */
> ```
>
> <a id="img-tbl-spacing"></a>
>
> ![A table with border-spacing](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/tbl-spacing.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/tbl-spacing-desc.html)
>
> A table with ['border-spacing'](css2--tables.html--201812dd6e3c.md#propdef-border-spacing) set to a length value. Note that each cell has its own border, and the table has a separate border as well.

<a id="empty-cells"></a>

#### 17.6.1.1 Borders and Backgrounds around empty cells: the ['empty-cells'](css2--tables.html--201812dd6e3c.md#propdef-empty-cells) property

<a id="propdef-empty-cells"></a>

<strong>'empty-cells'</strong>

|                       |                                                                                                        |
|-----------------------|--------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | show \| hide \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | show                                                                                                   |
| <em>Applies to:</em>   | 'table-cell' elements                                                                                  |
| <em>Inherited:</em>   | yes                                                                                                    |
| <em>Percentages:</em>   | N/A                                                                                                    |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                   |
| <em>Computed value:</em>   | as specified                                                                                           |

In the separated borders model, this property controls the rendering of borders and backgrounds around cells that have no visible content. Empty cells and cells with the ['visibility'](css2--visufx.html--2bc674cb7ab0.md#propdef-visibility) property set to 'hidden' are considered to have no visible content. Cells are empty unless they contain one or more of the following:

- floating content (including empty elements),
- in-flow content (including empty elements) other than white space that has been collapsed away by the ['white-space'](css2--text.html--467a8857ae69.md#propdef-white-space) property handling.

When this property has the value 'show', borders and backgrounds are drawn around/behind empty cells (like normal cells).

A value of 'hide' means that no borders or backgrounds are drawn around/behind empty cells (see point 6 in [17.5.1](#table-layers)). Furthermore, if all the cells in a row have a value of 'hide' and have no visible content, then the row has zero height and there is vertical border-spacing on only one side of the row.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> The following rule causes borders and backgrounds to be drawn around all cells:
>
> ```text
> 
> table { empty-cells: show }
> ```
<a id="collapsing-borders"></a>

### 17.6.2 The collapsing border model

In the collapsing border model, it is possible to specify borders that surround all or part of a cell, row, row group, column, and column group. Borders for HTML's "rules" attribute can be specified this way.

Borders are centered on the grid lines between the cells. User agents must find a consistent rule for rounding off in the case of an odd number of discrete units (screen pixels, printer dots).

The diagram below shows how the width of the table, the widths of the borders, the padding, and the cell width interact. Their relation is given by the following equation, which holds for every row of the table:

> <var>row-width</var> = (0.5 \* <var>border-width</var><sub>0</sub>) + <var>padding-left</var><sub>1</sub> + <var>width</var><sub>1</sub> + <var>padding-right</var><sub>1</sub> + <var>border-width</var><sub>1</sub> + <var>padding-left</var><sub>2</sub> +...+ <var>padding-right</var><sub><var>n</var></sub> + (0.5 \* <var>border-width</var><sub><var>n</var></sub>)

Here <var>n</var> is the number of cells in the row, <var>padding-left</var><sub><var>i</var></sub> and <var>padding-right</var><sub><var>i</var></sub> refer to the left (resp., right) padding of cell <var>i</var>, and <var>border-width</var><sub><var>i</var></sub> refers to the border between cells <var>i</var> and <var>i</var> + 1.

UAs must compute an initial left and right border width for the table by examining the first and last cells in the first row of the table. The left border width of the table is half of the first cell's collapsed left border, and the right border width of the table is half of the last cell's collapsed right border. If subsequent rows have larger collapsed left and right borders, then any excess spills into the margin area of the table.

The top border width of the table is computed by examining all cells who collapse their top borders with the top border of the table. The top border width of the table is equal to half of the maximum collapsed top border. The bottom border width is computed by examining all cells whose bottom borders collapse with the bottom of the table. The bottom border width is equal to half of the maximum collapsed bottom border.

Any borders that spill into the margin are taken into account when determining if the table overflows some ancestor (see ['overflow'](css2--visufx.html--2bc674cb7ab0.md#propdef-overflow)).

<a id="img-tbl-width"></a>

![Schema showing the widths of cells and borders and the padding of cells](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/tbl-width.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/tbl-width-desc.html)

Schema showing the widths of cells and borders and the padding of cells.

Note that in this model, the width of the table includes half the table border. Also, in this model, a table does not have padding (but does have margins).

CSS 2.1 does not define where the edge of a background on a table element lies.

<a id="border-conflict-resolution"></a>

#### 17.6.2.1 Border conflict resolution

In the collapsing border model, borders at every edge of every cell may be specified by border properties on a variety of elements that meet at that edge (cells, rows, row groups, columns, column groups, and the table itself), and these borders may vary in width, style, and color. The rule of thumb is that at each edge the most "eye catching" border style is chosen, except that any occurrence of the style 'hidden' unconditionally turns the border off.

The following rules determine which border style "wins" in case of a conflict:

1.  Borders with the ['border-style'](css2--box.html--8875bbcdefe6.md#propdef-border-style) of 'hidden' take precedence over all other conflicting borders. Any border with this value suppresses all borders at this location.
2.  Borders with a style of 'none' have the lowest priority. Only if the border properties of all the elements meeting at this edge are 'none' will the border be omitted (but note that 'none' is the default value for the border style.)
3.  If none of the styles are 'hidden' and at least one of them is not 'none', then narrow borders are discarded in favor of wider ones. If several have the same ['border-width'](css2--box.html--8875bbcdefe6.md#propdef-border-width) then styles are preferred in this order: 'double', 'solid', 'dashed', 'dotted', 'ridge', 'outset', 'groove', and the lowest: 'inset'.
4.  If border styles differ only in color, then a style set on a cell wins over one on a row, which wins over a row group, column, column group and, lastly, table. When two elements of the same type conflict, then the one further to the left (if the table's 'direction' is 'ltr'; right, if it is 'rtl') and further to the top wins.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> The following example illustrates the application of these precedence rules. This style sheet:
>
> ```text
> 
> table          { border-collapse: collapse;
>                  border: 5px solid yellow; }
> *#col1         { border: 3px solid black; }
> td             { border: 1px solid red; padding: 1em; }
> td.cell5       { border: 5px dashed blue; }
> td.cell6       { border: 5px solid green; }
> ```
>
> with this HTML source:
>
> ```text
> 
> <TABLE>
> <COL id="col1"><COL id="col2"><COL id="col3">
> <TR id="row1">
>     <TD> 1
>     <TD> 2
>     <TD> 3
> </TR>
> <TR id="row2">
>     <TD> 4 
>     <TD class="cell5"> 5
>     <TD class="cell6"> 6
> </TR>
> <TR id="row3">
>     <TD> 7
>     <TD> 8
>     <TD> 9
> </TR>
> <TR id="row4">
>     <TD> 10
>     <TD> 11
>     <TD> 12
> </TR>
> <TR id="row5">
>     <TD> 13
>     <TD> 14
>     <TD> 15
> </TR>
> </TABLE>
> ```
>
> would produce something like this:
>
> <a id="img-tbl-border-conflict"></a>
>
> ![An example of a table with collapsed borders](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/tbl-border-conflict.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/tbl-border-conflict-desc.html)
>
> An example of a table with collapsed borders.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Here is an example of hidden collapsing borders:
>
> <a id="img-CSStbl3"></a>
>
> ![Table with two omitted borders](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/CSStbl3.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/CSStbl3-desc.html)
>
> Table with two omitted internal borders.
>
> HTML source:
>
> ```text
> 
> <TABLE style="border-collapse: collapse; border: solid;">
> <TR><TD style="border-right: hidden; border-bottom: hidden">foo</TD>
>     <TD style="border: solid">bar</TD></TR>
> <TR><TD style="border: none">foo</TD>
>     <TD style="border: solid">bar</TD></TR>
> </TABLE>
> ```
<a id="table-border-styles"></a>

### 17.6.3 Border styles

<a id="x29"></a>

Some of the values of the ['border-style'](css2--box.html--8875bbcdefe6.md#propdef-border-style) have different meanings in tables than for other elements. In the list below they are marked with an asterisk.

[<strong>none</strong>](css2--box.html--8875bbcdefe6.md#value-def-bo-none)

<a id="x30"></a>

No border.

<a id="x31"></a>

<strong>&#x2A;<span title="'hidden"><a href="css2--box.html--8875bbcdefe6.md#value-def-hidden"><span>hidden</span></a></span></strong>

Same as 'none', but in the [collapsing border model](#collapsing-borders), also inhibits any other border (see the section on [border conflicts](#border-conflict-resolution)).

<a id="x32"></a>

<strong><span title="'dotted'"><a href="css2--box.html--8875bbcdefe6.md#value-def-dotted"><span>dotted</span></a></span></strong>

The border is a series of dots.

<a id="x33"></a>

<strong><span title="'dashed'"><a href="css2--box.html--8875bbcdefe6.md#value-def-dashed"><span>dashed</span></a></span></strong>

The border is a series of short line segments.

<a id="x34"></a>

<strong><span title="'solid'"><a href="css2--box.html--8875bbcdefe6.md#value-def-solid"><span>solid</span></a></span></strong>

The border is a single line segment.

<a id="x35"></a>

<strong><span title="'double'"><a href="css2--box.html--8875bbcdefe6.md#value-def-double"><span>double</span></a></span></strong>

The border is two solid lines. The sum of the two lines and the space between them equals the value of ['border-width'](css2--box.html--8875bbcdefe6.md#propdef-border-width).

<a id="x36"></a>

<strong><span title="'groove'"><a href="css2--box.html--8875bbcdefe6.md#value-def-groove"><span>groove</span></a></span></strong>

The border looks as though it were carved into the canvas.

<a id="x37"></a>

<strong><span title="'ridge'"><a href="css2--box.html--8875bbcdefe6.md#value-def-ridge"><span>ridge</span></a></span></strong>

The opposite of 'groove': the border looks as though it were coming out of the canvas.

<a id="x38"></a>

<strong>&#x2A;<span title="'inset'"><a href="css2--box.html--8875bbcdefe6.md#value-def-inset"><span>inset</span></a></span></strong>

In the [separated borders model](#separated-borders), the border makes the entire box look as though it were embedded in the canvas. In the [collapsing border model](#collapsing-borders), drawn the same as 'ridge'.

<a id="x39"></a>

<strong>&#x2A;<span title="'outset'"><a href="css2--box.html--8875bbcdefe6.md#value-def-outset"><span>outset</span></a></span></strong>

In the [separated borders model](#separated-borders), the border makes the entire box look as though it were coming out of the canvas. In the [collapsing border model](#collapsing-borders), drawn the same as 'groove'.
