Attribution and reformatting notice added for Surgeist on 2026-10-11

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft).

This is a bounded reformatted excerpt from the [WHATWG HTML Living Standard](https://html.spec.whatwg.org/multipage/tables.html), retrieved 2026-10-11 (source header: Last Updated 9 October 2026), under [Creative Commons Attribution 4.0 International](../licenses/whatwg/LICENSE). Its original [HTML repository LICENSE](https://github.com/whatwg/html/blob/main/LICENSE) was captured and compared byte-for-byte with that bundled legal text; SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. The original remains authoritative; no endorsement is implied. This excerpt is software implementation reference material; incorporation into source code has the separate BSD 3-Clause scope described by the LICENSE.

# Provenance and boundary

Full captured HTML SHA-256 `769e9afab8720b2c762fbe060ae2b5a15bfe84e202f7a043414e732cf9ff910a` (254869 bytes); bounded input SHA-256 `7e90e654e94f55d67d0f5c15a052b9315ca31418c5e5fa5b1f3d1dd2fef0606b` (32725 bytes). Retained §4.9.12 Processing model definitions and complete §4.9.12.1 Forming a table, bounded before §4.9.12.2 Header relationships. Conversion uses existing HTML preparation, Pandoc 3.1.11.1, semantic filter, and structural/prose verification. Added one clearly identified source-status paragraph for tooling; source content otherwise remains faithful.

---

<!-- captured-body-start -->

# HTML

<a id="living-standard"></a>Living Standard — Last Updated 9 October 2026

<a id="w3c-state"></a>WHATWG HTML Living Standard, Last Updated 9 October 2026 (captured source header)

#### <a id="table-processing-model"></a>4.9.12 <a id="processing-model-2"></a>Processing model[](#table-processing-model)

The various table elements and their content attributes together define the <a id="table-model"></a><strong>table model</strong>.

A <a id="concept-table"></a><strong>table</strong> consists of cells aligned on a two-dimensional grid of <a id="concept-slots"></a><strong>slots</strong> with coordinates (<var>x</var>, <var>y</var>). The grid is finite, and is either empty or has one or more slots. If the grid has one or more slots, then the <var>x</var> coordinates are always in the range 0 ≤ <var>x</var> \< <var>x<sub>width</sub></var>, and the <var>y</var> coordinates are always in the range 0 ≤ <var>y</var> \< <var>y<sub>height</sub></var>. If one or both of <var>x<sub>width</sub></var> and <var>y<sub>height</sub></var> are zero, then the table is empty (has no slots). Tables correspond to <a id="table-processing-model&#58;the-table-element"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code> elements.

A <a id="concept-cell"></a><strong>cell</strong> is a set of slots anchored at a slot (<var>cell<sub>x</sub></var>, <var>cell<sub>y</sub></var>), and with a particular <var>width</var> and <var>height</var> such that the cell covers all the slots with coordinates (<var>x</var>, <var>y</var>) where <var>cell<sub>x</sub></var> ≤ <var>x</var> \< <var>cell<sub>x</sub></var>+<var>width</var> and <var>cell<sub>y</sub></var> ≤ <var>y</var> \< <var>cell<sub>y</sub></var>+<var>height</var>. Cells can either be <em>data cells</em> or <em>header cells</em>. Data cells correspond to <a id="table-processing-model&#58;the-td-element"></a><code>[td](https://html.spec.whatwg.org/multipage/tables.html#the-td-element)</code> elements, and header cells correspond to <a id="table-processing-model&#58;the-th-element"></a><code>[th](https://html.spec.whatwg.org/multipage/tables.html#the-th-element)</code> elements. Cells of both types can have zero or more associated header cells.

It is possible, in certain error cases, for two cells to occupy the same slot.

A <a id="concept-row"></a><strong>row</strong> is a complete set of slots from <var>x</var>=0 to <var>x</var>=<var>x<sub>width</sub></var>-1, for a particular value of <var>y</var>. Rows usually correspond to <a id="table-processing-model&#58;the-tr-element"></a><code>[tr](https://html.spec.whatwg.org/multipage/tables.html#the-tr-element)</code> elements, though a <a id="table-processing-model&#58;concept-row-group"></a>[row group](#concept-row-group) can have some implied <a id="table-processing-model&#58;concept-row"></a>[rows](#concept-row) at the end in some cases involving <a id="table-processing-model&#58;concept-cell"></a>[cells](#concept-cell) spanning multiple rows.

A <a id="concept-column"></a><strong>column</strong> is a complete set of slots from <var>y</var>=0 to <var>y</var>=<var>y<sub>height</sub></var>-1, for a particular value of <var>x</var>. Columns can correspond to <a id="table-processing-model&#58;the-col-element"></a><code>[col](https://html.spec.whatwg.org/multipage/tables.html#the-col-element)</code> elements. In the absence of <a id="table-processing-model&#58;the-col-element-2"></a><code>[col](https://html.spec.whatwg.org/multipage/tables.html#the-col-element)</code> elements, columns are implied.

A <a id="concept-row-group"></a><strong>row group</strong> is a set of <a id="table-processing-model&#58;concept-row-2"></a>[rows](#concept-row) anchored at a slot (0, <var>group<sub>y</sub></var>) with a particular <var>height</var> such that the row group covers all the slots with coordinates (<var>x</var>, <var>y</var>) where 0 ≤ <var>x</var> \< <var>x<sub>width</sub></var> and <var>group<sub>y</sub></var> ≤ <var>y</var> \< <var>group<sub>y</sub></var>+<var>height</var>. Row groups correspond to <a id="table-processing-model&#58;the-tbody-element"></a><code>[tbody](https://html.spec.whatwg.org/multipage/tables.html#the-tbody-element)</code>, <a id="table-processing-model&#58;the-thead-element"></a><code>[thead](https://html.spec.whatwg.org/multipage/tables.html#the-thead-element)</code>, and <a id="table-processing-model&#58;the-tfoot-element"></a><code>[tfoot](https://html.spec.whatwg.org/multipage/tables.html#the-tfoot-element)</code> elements. Not every row is necessarily in a row group.

A <a id="concept-column-group"></a><strong>column group</strong> is a set of <a id="table-processing-model&#58;concept-column"></a>[columns](#concept-column) anchored at a slot (<var>group<sub>x</sub></var>, 0) with a particular <var>width</var> such that the column group covers all the slots with coordinates (<var>x</var>, <var>y</var>) where <var>group<sub>x</sub></var> ≤ <var>x</var> \< <var>group<sub>x</sub></var>+<var>width</var> and 0 ≤ <var>y</var> \< <var>y<sub>height</sub></var>. Column groups correspond to <a id="table-processing-model&#58;the-colgroup-element"></a><code>[colgroup](https://html.spec.whatwg.org/multipage/tables.html#the-colgroup-element)</code> elements. Not every column is necessarily in a column group.

<a id="table-processing-model&#58;concept-row-group-2"></a>[Row groups](#concept-row-group) cannot overlap each other. Similarly, <a id="table-processing-model&#58;concept-column-group"></a>[column groups](#concept-column-group) cannot overlap each other.

A <a id="table-processing-model&#58;concept-cell-2"></a>[cell](#concept-cell) cannot cover slots that are from two or more <a id="table-processing-model&#58;concept-row-group-3"></a>[row groups](#concept-row-group). It is, however, possible for a cell to be in multiple <a id="table-processing-model&#58;concept-column-group-2"></a>[column groups](#concept-column-group). All the slots that form part of one cell are part of zero or one <a id="table-processing-model&#58;concept-row-group-4"></a>[row groups](#concept-row-group) and zero or more <a id="table-processing-model&#58;concept-column-group-3"></a>[column groups](#concept-column-group).

In addition to <a id="table-processing-model&#58;concept-cell-3"></a>[cells](#concept-cell), <a id="table-processing-model&#58;concept-column-2"></a>[columns](#concept-column), <a id="table-processing-model&#58;concept-row-3"></a>[rows](#concept-row), <a id="table-processing-model&#58;concept-row-group-5"></a>[row groups](#concept-row-group), and <a id="table-processing-model&#58;concept-column-group-4"></a>[column groups](#concept-column-group), <a id="table-processing-model&#58;concept-table"></a>[tables](#concept-table) can have a <a id="table-processing-model&#58;the-caption-element"></a><code>[caption](https://html.spec.whatwg.org/multipage/tables.html#the-caption-element)</code> element associated with them. This gives the table a heading, or legend.

A <a id="table-model-error"></a><strong>table model error</strong> is an error with the data represented by <a id="table-processing-model&#58;the-table-element-2"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code> elements and their descendants. Documents must not have table model errors.

##### <a id="forming-a-table"></a>4.9.12.1 Forming a table[](#forming-a-table)

To determine which elements correspond to which slots in a <a id="forming-a-table&#58;concept-table"></a>[table](#concept-table) associated with a <a id="forming-a-table&#58;the-table-element"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code> element, to determine the dimensions of the table (<var>x<sub>width</sub></var> and <var>y<sub>height</sub></var>), and to determine if there are any <a id="forming-a-table&#58;table-model-error"></a>[table model errors](#table-model-error), user agents must use the following algorithm:

1.  Let <var>x<sub>width</sub></var> be 0.

2.  Let <var>y<sub>height</sub></var> be 0.

3.  Let <var>pending <a id="forming-a-table&#58;the-tfoot-element"></a><code>[tfoot](https://html.spec.whatwg.org/multipage/tables.html#the-tfoot-element)</code> elements</var> be a list of <a id="forming-a-table&#58;the-tfoot-element-2"></a><code>[tfoot](https://html.spec.whatwg.org/multipage/tables.html#the-tfoot-element)</code> elements, initially empty.

4.  Let <var>the table</var> be the <a id="forming-a-table&#58;concept-table-2"></a>[table](#concept-table) represented by the <a id="forming-a-table&#58;the-table-element-2"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code> element. The <var>x<sub>width</sub></var> and <var>y<sub>height</sub></var> variables give <var>the table</var>'s dimensions. <var>The table</var> is initially empty.

5.  If the <a id="forming-a-table&#58;the-table-element-3"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code> element has no children elements, then return <var>the table</var> (which will be empty).

6.  Associate the first <a id="forming-a-table&#58;the-caption-element"></a><code>[caption](https://html.spec.whatwg.org/multipage/tables.html#the-caption-element)</code> element child of the <a id="forming-a-table&#58;the-table-element-4"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code> element with <var>the table</var>. If there are no such children, then it has no associated <a id="forming-a-table&#58;the-caption-element-2"></a><code>[caption](https://html.spec.whatwg.org/multipage/tables.html#the-caption-element)</code> element.

7.  Let the <var>current element</var> be the first element child of the <a id="forming-a-table&#58;the-table-element-5"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code> element.

    If a step in this algorithm ever requires the <var>current element</var> to be <a id="concept-table-advance"></a><strong>advanced to the next child of the <code>table</code></strong> when there is no such next child, then the user agent must jump to the step labeled <em>end</em>, near the end of this algorithm.

8.  While the <var>current element</var> is not one of the following elements, <a id="forming-a-table&#58;concept-table-advance"></a>[advance](#concept-table-advance) the <var>current element</var> to the next child of the <a id="forming-a-table&#58;the-table-element-6"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code>:

    - <a id="forming-a-table&#58;the-colgroup-element"></a><code>[colgroup](https://html.spec.whatwg.org/multipage/tables.html#the-colgroup-element)</code>
    - <a id="forming-a-table&#58;the-thead-element"></a><code>[thead](https://html.spec.whatwg.org/multipage/tables.html#the-thead-element)</code>
    - <a id="forming-a-table&#58;the-tbody-element"></a><code>[tbody](https://html.spec.whatwg.org/multipage/tables.html#the-tbody-element)</code>
    - <a id="forming-a-table&#58;the-tfoot-element-3"></a><code>[tfoot](https://html.spec.whatwg.org/multipage/tables.html#the-tfoot-element)</code>
    - <a id="forming-a-table&#58;the-tr-element"></a><code>[tr](https://html.spec.whatwg.org/multipage/tables.html#the-tr-element)</code>

9.  If the <var>current element</var> is a <a id="forming-a-table&#58;the-colgroup-element-2"></a><code>[colgroup](https://html.spec.whatwg.org/multipage/tables.html#the-colgroup-element)</code>, follow these substeps:

    1.  <em>Column groups</em>: Process the <var>current element</var> according to the appropriate case below:

        <strong>If the <var>current element</var> has any <a id="forming-a-table&#58;the-col-element"></a><code>[col](https://html.spec.whatwg.org/multipage/tables.html#the-col-element)</code> element children</strong>

        Follow these steps:

        1.  Let <var>x<sub>start</sub></var> have the value of <var>x<sub>width</sub></var>.

        2.  Let the <var>current column</var> be the first <a id="forming-a-table&#58;the-col-element-2"></a><code>[col](https://html.spec.whatwg.org/multipage/tables.html#the-col-element)</code> element child of the <a id="forming-a-table&#58;the-colgroup-element-3"></a><code>[colgroup](https://html.spec.whatwg.org/multipage/tables.html#the-colgroup-element)</code> element.

        3.  <em>Columns</em>: If the <var>current column</var> <a id="forming-a-table&#58;the-col-element-3"></a><code>[col](https://html.spec.whatwg.org/multipage/tables.html#the-col-element)</code> element has a <a id="forming-a-table&#58;attr-col-span"></a><code>[span](https://html.spec.whatwg.org/multipage/tables.html#attr-col-span)</code> attribute, then parse its value using the <a id="forming-a-table&#58;rules-for-parsing-non-negative-integers"></a>[rules for parsing non-negative integers](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#rules-for-parsing-non-negative-integers).

            If the result of parsing the value is not an error or zero, then let <var>span</var> be that value.

            Otherwise, if the <a id="forming-a-table&#58;the-col-element-4"></a><code>[col](https://html.spec.whatwg.org/multipage/tables.html#the-col-element)</code> element has no <a id="forming-a-table&#58;attr-col-span-2"></a><code>[span](https://html.spec.whatwg.org/multipage/tables.html#attr-col-span)</code> attribute, or if trying to parse the attribute's value resulted in an error or zero, then let <var>span</var> be 1.

            If <var>span</var> is greater than 1000, let it be 1000 instead.

        4.  Increase <var>x<sub>width</sub></var> by <var>span</var>.

        5.  Let the last <var>span</var> <a id="forming-a-table&#58;concept-column"></a>[columns](#concept-column) in <var>the table</var> correspond to the <var>current column</var> <a id="forming-a-table&#58;the-col-element-5"></a><code>[col](https://html.spec.whatwg.org/multipage/tables.html#the-col-element)</code> element.

        6.  If <var>current column</var> is not the last <a id="forming-a-table&#58;the-col-element-6"></a><code>[col](https://html.spec.whatwg.org/multipage/tables.html#the-col-element)</code> element child of the <a id="forming-a-table&#58;the-colgroup-element-4"></a><code>[colgroup](https://html.spec.whatwg.org/multipage/tables.html#the-colgroup-element)</code> element, then let the <var>current column</var> be the next <a id="forming-a-table&#58;the-col-element-7"></a><code>[col](https://html.spec.whatwg.org/multipage/tables.html#the-col-element)</code> element child of the <a id="forming-a-table&#58;the-colgroup-element-5"></a><code>[colgroup](https://html.spec.whatwg.org/multipage/tables.html#the-colgroup-element)</code> element, and return to the step labeled <em>columns</em>.

        7.  Let all the last <a id="forming-a-table&#58;concept-column-2"></a>[columns](#concept-column) in <var>the table</var> from x=<var>x<sub>start</sub></var> to x=<var>x<sub>width</sub></var>-1 form a new <a id="forming-a-table&#58;concept-column-group"></a>[column group](#concept-column-group), anchored at the slot (<var>x<sub>start</sub></var>, 0), with width <var>x<sub>width</sub></var>-<var>x<sub>start</sub></var>, corresponding to the <a id="forming-a-table&#58;the-colgroup-element-6"></a><code>[colgroup](https://html.spec.whatwg.org/multipage/tables.html#the-colgroup-element)</code> element.

        <strong>If the <var>current element</var> has no <a id="forming-a-table&#58;the-col-element-8"></a><code>[col](https://html.spec.whatwg.org/multipage/tables.html#the-col-element)</code> element children</strong>

        1.  If the <a id="forming-a-table&#58;the-colgroup-element-7"></a><code>[colgroup](https://html.spec.whatwg.org/multipage/tables.html#the-colgroup-element)</code> element has a <a id="forming-a-table&#58;attr-colgroup-span"></a><code>[span](https://html.spec.whatwg.org/multipage/tables.html#attr-colgroup-span)</code> attribute, then parse its value using the <a id="forming-a-table&#58;rules-for-parsing-non-negative-integers-2"></a>[rules for parsing non-negative integers](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#rules-for-parsing-non-negative-integers).

            If the result of parsing the value is not an error or zero, then let <var>span</var> be that value.

            Otherwise, if the <a id="forming-a-table&#58;the-colgroup-element-8"></a><code>[colgroup](https://html.spec.whatwg.org/multipage/tables.html#the-colgroup-element)</code> element has no <a id="forming-a-table&#58;attr-colgroup-span-2"></a><code>[span](https://html.spec.whatwg.org/multipage/tables.html#attr-colgroup-span)</code> attribute, or if trying to parse the attribute's value resulted in an error or zero, then let <var>span</var> be 1.

            If <var>span</var> is greater than 1000, let it be 1000 instead.

        2.  Increase <var>x<sub>width</sub></var> by <var>span</var>.

        3.  Let the last <var>span</var> <a id="forming-a-table&#58;concept-column-3"></a>[columns](#concept-column) in <var>the table</var> form a new <a id="forming-a-table&#58;concept-column-group-2"></a>[column group](#concept-column-group), anchored at the slot (<var>x<sub>width</sub></var>-<var>span</var>, 0), with width <var>span</var>, corresponding to the <a id="forming-a-table&#58;the-colgroup-element-9"></a><code>[colgroup](https://html.spec.whatwg.org/multipage/tables.html#the-colgroup-element)</code> element.

    2.  <a id="forming-a-table&#58;concept-table-advance-2"></a>[Advance](#concept-table-advance) the <var>current element</var> to the next child of the <a id="forming-a-table&#58;the-table-element-7"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code>.

    3.  While the <var>current element</var> is not one of the following elements, <a id="forming-a-table&#58;concept-table-advance-3"></a>[advance](#concept-table-advance) the <var>current element</var> to the next child of the <a id="forming-a-table&#58;the-table-element-8"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code>:

        - <a id="forming-a-table&#58;the-colgroup-element-10"></a><code>[colgroup](https://html.spec.whatwg.org/multipage/tables.html#the-colgroup-element)</code>
        - <a id="forming-a-table&#58;the-thead-element-2"></a><code>[thead](https://html.spec.whatwg.org/multipage/tables.html#the-thead-element)</code>
        - <a id="forming-a-table&#58;the-tbody-element-2"></a><code>[tbody](https://html.spec.whatwg.org/multipage/tables.html#the-tbody-element)</code>
        - <a id="forming-a-table&#58;the-tfoot-element-4"></a><code>[tfoot](https://html.spec.whatwg.org/multipage/tables.html#the-tfoot-element)</code>
        - <a id="forming-a-table&#58;the-tr-element-2"></a><code>[tr](https://html.spec.whatwg.org/multipage/tables.html#the-tr-element)</code>

    4.  If the <var>current element</var> is a <a id="forming-a-table&#58;the-colgroup-element-11"></a><code>[colgroup](https://html.spec.whatwg.org/multipage/tables.html#the-colgroup-element)</code> element, jump to the step labeled <em>column groups</em> above.

10. Let <var>y<sub>current</sub></var> be 0.

11. Let the <var>list of downward-growing cells</var> be an empty list.

12. <em>Rows</em>: While the <var>current element</var> is not one of the following elements, <a id="forming-a-table&#58;concept-table-advance-4"></a>[advance](#concept-table-advance) the <var>current element</var> to the next child of the <a id="forming-a-table&#58;the-table-element-9"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code>:

    - <a id="forming-a-table&#58;the-thead-element-3"></a><code>[thead](https://html.spec.whatwg.org/multipage/tables.html#the-thead-element)</code>
    - <a id="forming-a-table&#58;the-tbody-element-3"></a><code>[tbody](https://html.spec.whatwg.org/multipage/tables.html#the-tbody-element)</code>
    - <a id="forming-a-table&#58;the-tfoot-element-5"></a><code>[tfoot](https://html.spec.whatwg.org/multipage/tables.html#the-tfoot-element)</code>
    - <a id="forming-a-table&#58;the-tr-element-3"></a><code>[tr](https://html.spec.whatwg.org/multipage/tables.html#the-tr-element)</code>

13. If the <var>current element</var> is a <a id="forming-a-table&#58;the-tr-element-4"></a><code>[tr](https://html.spec.whatwg.org/multipage/tables.html#the-tr-element)</code>, then run the <a id="forming-a-table&#58;algorithm-for-processing-rows"></a>[algorithm for processing rows](#algorithm-for-processing-rows), <a id="forming-a-table&#58;concept-table-advance-5"></a>[advance](#concept-table-advance) the <var>current element</var> to the next child of the <a id="forming-a-table&#58;the-table-element-10"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code>, and return to the step labeled <em>rows</em>.

14. Run the <a id="forming-a-table&#58;algorithm-for-ending-a-row-group"></a>[algorithm for ending a row group](#algorithm-for-ending-a-row-group).

15. If the <var>current element</var> is a <a id="forming-a-table&#58;the-tfoot-element-6"></a><code>[tfoot](https://html.spec.whatwg.org/multipage/tables.html#the-tfoot-element)</code>, then add that element to the list of <var>pending <a id="forming-a-table&#58;the-tfoot-element-7"></a><code>[tfoot](https://html.spec.whatwg.org/multipage/tables.html#the-tfoot-element)</code> elements</var>, <a id="forming-a-table&#58;concept-table-advance-6"></a>[advance](#concept-table-advance) the <var>current element</var> to the next child of the <a id="forming-a-table&#58;the-table-element-11"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code>, and return to the step labeled <em>rows</em>.

16. The <var>current element</var> is either a <a id="forming-a-table&#58;the-thead-element-4"></a><code>[thead](https://html.spec.whatwg.org/multipage/tables.html#the-thead-element)</code> or a <a id="forming-a-table&#58;the-tbody-element-4"></a><code>[tbody](https://html.spec.whatwg.org/multipage/tables.html#the-tbody-element)</code>.

    Run the <a id="forming-a-table&#58;algorithm-for-processing-row-groups"></a>[algorithm for processing row groups](#algorithm-for-processing-row-groups).

17. <a id="forming-a-table&#58;concept-table-advance-7"></a>[Advance](#concept-table-advance) the <var>current element</var> to the next child of the <a id="forming-a-table&#58;the-table-element-12"></a><code>[table](https://html.spec.whatwg.org/multipage/tables.html#the-table-element)</code>.

18. Return to the step labeled <em>rows</em>.

19. <em>End</em>: For each <a id="forming-a-table&#58;the-tfoot-element-8"></a><code>[tfoot](https://html.spec.whatwg.org/multipage/tables.html#the-tfoot-element)</code> element in the list of <var>pending <a id="forming-a-table&#58;the-tfoot-element-9"></a><code>[tfoot](https://html.spec.whatwg.org/multipage/tables.html#the-tfoot-element)</code> elements</var>, in <a id="forming-a-table&#58;tree-order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order), run the <a id="forming-a-table&#58;algorithm-for-processing-row-groups-2"></a>[algorithm for processing row groups](#algorithm-for-processing-row-groups).

20. If there exists a <a id="forming-a-table&#58;concept-row"></a>[row](#concept-row) or <a id="forming-a-table&#58;concept-column-4"></a>[column](#concept-column) in <var>the table</var> containing only <a id="forming-a-table&#58;concept-slots"></a>[slots](#concept-slots) that do not have a <a id="forming-a-table&#58;concept-cell"></a>[cell](#concept-cell) anchored to them, then this is a <a id="forming-a-table&#58;table-model-error-2"></a>[table model error](#table-model-error).

21. Return <var>the table</var>.

The <a id="algorithm-for-processing-row-groups"></a><strong>algorithm for processing row groups</strong>, which is invoked by the set of steps above for processing <a id="forming-a-table&#58;the-thead-element-5"></a><code>[thead](https://html.spec.whatwg.org/multipage/tables.html#the-thead-element)</code>, <a id="forming-a-table&#58;the-tbody-element-5"></a><code>[tbody](https://html.spec.whatwg.org/multipage/tables.html#the-tbody-element)</code>, and <a id="forming-a-table&#58;the-tfoot-element-10"></a><code>[tfoot](https://html.spec.whatwg.org/multipage/tables.html#the-tfoot-element)</code> elements, is:

1.  Let <var>y<sub>start</sub></var> have the value of <var>y<sub>height</sub></var>.

2.  For each <a id="forming-a-table&#58;the-tr-element-5"></a><code>[tr](https://html.spec.whatwg.org/multipage/tables.html#the-tr-element)</code> element that is a child of the element being processed, in tree order, run the <a id="forming-a-table&#58;algorithm-for-processing-rows-2"></a>[algorithm for processing rows](#algorithm-for-processing-rows).

3.  If <var>y<sub>height</sub></var> \> <var>y<sub>start</sub></var>, then let all the last <a id="forming-a-table&#58;concept-row-2"></a>[rows](#concept-row) in <var>the table</var> from y=<var>y<sub>start</sub></var> to y=<var>y<sub>height</sub></var>-1 form a new <a id="forming-a-table&#58;concept-row-group"></a>[row group](#concept-row-group), anchored at the slot with coordinate (0, <var>y<sub>start</sub></var>), with height <var>y<sub>height</sub></var>-<var>y<sub>start</sub></var>, corresponding to the element being processed.

4.  Run the <a id="forming-a-table&#58;algorithm-for-ending-a-row-group-2"></a>[algorithm for ending a row group](#algorithm-for-ending-a-row-group).

The <a id="algorithm-for-ending-a-row-group"></a><strong>algorithm for ending a row group</strong>, which is invoked by the set of steps above when starting and ending a block of rows, is:

1.  While <var>y<sub>current</sub></var> is less than <var>y<sub>height</sub></var>, follow these steps:

    1.  Run the <a id="forming-a-table&#58;algorithm-for-growing-downward-growing-cells"></a>[algorithm for growing downward-growing cells](#algorithm-for-growing-downward-growing-cells).

    2.  Increase <var>y<sub>current</sub></var> by 1.

2.  Empty the <var>list of downward-growing cells</var>.

The <a id="algorithm-for-processing-rows"></a><strong>algorithm for processing rows</strong>, which is invoked by the set of steps above for processing <a id="forming-a-table&#58;the-tr-element-6"></a><code>[tr](https://html.spec.whatwg.org/multipage/tables.html#the-tr-element)</code> elements, is:

1.  If <var>y<sub>height</sub></var> is equal to <var>y<sub>current</sub></var>, then increase <var>y<sub>height</sub></var> by 1. (<var>y<sub>current</sub></var> is never <em>greater</em> than <var>y<sub>height</sub></var>.)

2.  Let <var>x<sub>current</sub></var> be 0.

3.  Run the <a id="forming-a-table&#58;algorithm-for-growing-downward-growing-cells-2"></a>[algorithm for growing downward-growing cells](#algorithm-for-growing-downward-growing-cells).

4.  If the <a id="forming-a-table&#58;the-tr-element-7"></a><code>[tr](https://html.spec.whatwg.org/multipage/tables.html#the-tr-element)</code> element being processed has no <a id="forming-a-table&#58;the-td-element"></a><code>[td](https://html.spec.whatwg.org/multipage/tables.html#the-td-element)</code> or <a id="forming-a-table&#58;the-th-element"></a><code>[th](https://html.spec.whatwg.org/multipage/tables.html#the-th-element)</code> element children, then increase <var>y<sub>current</sub></var> by 1, abort this set of steps, and return to the algorithm above.

5.  Let <var>current cell</var> be the first <a id="forming-a-table&#58;the-td-element-2"></a><code>[td](https://html.spec.whatwg.org/multipage/tables.html#the-td-element)</code> or <a id="forming-a-table&#58;the-th-element-2"></a><code>[th](https://html.spec.whatwg.org/multipage/tables.html#the-th-element)</code> element child in the <a id="forming-a-table&#58;the-tr-element-8"></a><code>[tr](https://html.spec.whatwg.org/multipage/tables.html#the-tr-element)</code> element being processed.

6.  <em>Cells</em>: While <var>x<sub>current</sub></var> is less than <var>x<sub>width</sub></var> and the slot with coordinate (<var>x<sub>current</sub></var>, <var>y<sub>current</sub></var>) already has a cell assigned to it, increase <var>x<sub>current</sub></var> by 1.

7.  If <var>x<sub>current</sub></var> is equal to <var>x<sub>width</sub></var>, increase <var>x<sub>width</sub></var> by 1. (<var>x<sub>current</sub></var> is never <em>greater</em> than <var>x<sub>width</sub></var>.)

8.  If the <var>current cell</var> has a <a id="forming-a-table&#58;attr-tdth-colspan"></a><code>[colspan](https://html.spec.whatwg.org/multipage/tables.html#attr-tdth-colspan)</code> attribute, then <a id="forming-a-table&#58;rules-for-parsing-non-negative-integers-3"></a>[parse that attribute's value](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#rules-for-parsing-non-negative-integers), and let <var>colspan</var> be the result.

    If parsing that value failed, or returned zero, or if the attribute is absent, then let <var>colspan</var> be 1, instead.

    If <var>colspan</var> is greater than 1000, let it be 1000 instead.

9.  If the <var>current cell</var> has a <a id="forming-a-table&#58;attr-tdth-rowspan"></a><code>[rowspan](https://html.spec.whatwg.org/multipage/tables.html#attr-tdth-rowspan)</code> attribute, then <a id="forming-a-table&#58;rules-for-parsing-non-negative-integers-4"></a>[parse that attribute's value](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#rules-for-parsing-non-negative-integers), and let <var>rowspan</var> be the result.

    If parsing that value failed or if the attribute is absent, then let <var>rowspan</var> be 1, instead.

    If <var>rowspan</var> is greater than 65534, let it be 65534 instead.

10. Let <var>cell grows downward</var> be false.

11. If <var>rowspan</var> is zero, then set <var>cell grows downward</var> to true and set <var>rowspan</var> to 1.

12. If <var>x<sub>width</sub></var> \< <var>x<sub>current</sub></var>+<var>colspan</var>, then let <var>x<sub>width</sub></var> be <var>x<sub>current</sub></var>+<var>colspan</var>.

13. If <var>y<sub>height</sub></var> \< <var>y<sub>current</sub></var>+<var>rowspan</var>, then let <var>y<sub>height</sub></var> be <var>y<sub>current</sub></var>+<var>rowspan</var>.

14. Let the slots with coordinates (<var>x</var>, <var>y</var>) such that <var>x<sub>current</sub></var> ≤ <var>x</var> \< <var>x<sub>current</sub></var>+<var>colspan</var> and <var>y<sub>current</sub></var> ≤ <var>y</var> \< <var>y<sub>current</sub></var>+<var>rowspan</var> be covered by a new <a id="forming-a-table&#58;concept-cell-2"></a>[cell](#concept-cell) <var>c</var>, anchored at (<var>x<sub>current</sub></var>, <var>y<sub>current</sub></var>), which has width <var>colspan</var> and height <var>rowspan</var>, corresponding to the <var>current cell</var> element.

    If the <var>current cell</var> element is a <a id="forming-a-table&#58;the-th-element-3"></a><code>[th](https://html.spec.whatwg.org/multipage/tables.html#the-th-element)</code> element, let this new cell <var>c</var> be a header cell; otherwise, let it be a data cell.

    To establish which header cells apply to the <var>current cell</var> element, use the <a id="forming-a-table&#58;algorithm-for-assigning-header-cells"></a>[algorithm for assigning header cells](https://html.spec.whatwg.org/multipage/tables.html#algorithm-for-assigning-header-cells) described in the next section.

    If any of the slots involved already had a <a id="forming-a-table&#58;concept-cell-3"></a>[cell](#concept-cell) covering them, then this is a <a id="forming-a-table&#58;table-model-error-3"></a>[table model error](#table-model-error). Those slots now have two cells overlapping.

15. If <var>cell grows downward</var> is true, then add the tuple {<var>c</var>, <var>x<sub>current</sub></var>, <var>colspan</var>} to the <var>list of downward-growing cells</var>.

16. Increase <var>x<sub>current</sub></var> by <var>colspan</var>.

17. If <var>current cell</var> is the last <a id="forming-a-table&#58;the-td-element-3"></a><code>[td](https://html.spec.whatwg.org/multipage/tables.html#the-td-element)</code> or <a id="forming-a-table&#58;the-th-element-4"></a><code>[th](https://html.spec.whatwg.org/multipage/tables.html#the-th-element)</code> element child in the <a id="forming-a-table&#58;the-tr-element-9"></a><code>[tr](https://html.spec.whatwg.org/multipage/tables.html#the-tr-element)</code> element being processed, then increase <var>y<sub>current</sub></var> by 1, abort this set of steps, and return to the algorithm above.

18. Let <var>current cell</var> be the next <a id="forming-a-table&#58;the-td-element-4"></a><code>[td](https://html.spec.whatwg.org/multipage/tables.html#the-td-element)</code> or <a id="forming-a-table&#58;the-th-element-5"></a><code>[th](https://html.spec.whatwg.org/multipage/tables.html#the-th-element)</code> element child in the <a id="forming-a-table&#58;the-tr-element-10"></a><code>[tr](https://html.spec.whatwg.org/multipage/tables.html#the-tr-element)</code> element being processed.

19. Return to the step labeled <em>cells</em>.

When the algorithms above require the user agent to run the <a id="algorithm-for-growing-downward-growing-cells"></a><strong>algorithm for growing downward-growing cells</strong>, the user agent must, for each {<var>cell</var>, <var>cell<sub>x</sub></var>, <var>width</var>} tuple in the <var>list of downward-growing cells</var>, if any, extend the <a id="forming-a-table&#58;concept-cell-4"></a>[cell](#concept-cell) <var>cell</var> so that it also covers the slots with coordinates (<var>x</var>, <var>y<sub>current</sub></var>), where <var>cell<sub>x</sub></var> ≤ <var>x</var> \< <var>cell<sub>x</sub></var>+<var>width</var>.
