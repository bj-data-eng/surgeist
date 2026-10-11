Attribution and reformatting notice added for Surgeist on 2026-10-11

This bounded reformatted excerpt accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material copied from [CSS Table Module Level 3](https://drafts.csswg.org/css-tables-3/) under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Original copyright, liability, trademark and permissive document-license notice retained below.

# Provenance and boundary

Captured 2026-10-11. Full HTML SHA-256 `2d68c4e6c7b26e7f1ad438d6ea3ed012f4ce389ae7d8f2af690ad00f6e9d5241`; 435301 bytes. Bounded HTML SHA-256 `f213e96cb8a9974fe951e9fa4ef7a86f56011afba1d7c453eaac49ffc2e7e92e`; 41734 bytes. Source revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`. Retained complete heading sections: `dimensioning-the-row-column-grid`, each through the next equal-or-higher heading, plus source header/status including warnings. Other sections resolve through upstream links. This editor capture is work in progress; Surgeist policies are recorded separately and do not change the captured source.

Converted with the existing HTML preparation, Pandoc 3.1.11.1 and semantic filter.

---

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

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication. A list of current W3C publications and the latest revision of this technical report can be found in the [W3C standards and drafts index.](https://www.w3.org/TR/)</em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-tables” in the title, like this: “\[css-tables\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-tables%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

### <a id="dimensioning-the-row-column-grid"></a>3.3. Dimensioning the row/column grid[](#dimensioning-the-row-column-grid)

Like mentioned in the [Table structure](https://drafts.csswg.org/css-tables-3/#table-structure) section, how many rows and columns a table has can be determined from the table structure. Both dimensioning the row/column <a id="ref-for-grid"></a>[grid](https://www.w3.org/TR/css-grid-2/#grid) and assigning table-cells their <a id="ref-for-slot③"></a>[slot](https://drafts.csswg.org/css-tables-3/#slot)(s) in that grid do require running the HTML Algorithms for tables.

#### <a id="dimensioning-the-row-column-grid--step1"></a>3.3.1. HTML Algorithm[](#dimensioning-the-row-column-grid--step1)

CSS Boxes that do not originate from an HTML table element equivalent to their display type need to be converted to their HTML equivalent before we can apply this algorithm, see below. There is no way to specify the

<strong>Note:</strong>

<a id="ref-for-span"></a>[span](https://drafts.csswg.org/css-tables-3/#span) of a cell in css only in this level of the spec, the use of an HTML td element is required to do so.

Apply the [HTML5 Table Formatting algorithm](https://www.w3.org/TR/html5/tabular-data.html#forming-a-table), where boxes act like [the HTML element equivalent to their display type](https://drafts.csswg.org/css-tables-3/#display-types), and use the attributes of their originating element if and only if it is an HTML element of the same type (otherwise, they act like if they didn’t have any attribute).

<a id="example-f35b62a4"></a>

<strong>Example:</strong>

[](#example-f35b62a4)

``` text
<ul class="table">
  <li><b>One</b><i>1</i></li>
  <li><b>Two</b><i>2</i></li>
  <li><b>Three</b><i>3</i></li>
</ul>
<style>
  ul.table { display: table; }
  ul.table > li { display: table-row; }
  ul.table > li > * { display: table-cell; }
</style>
```

produces the same row/column grid as

``` text
<table><tbody>
  <tr>
    <td></td>
    <td></td>
  </tr>
  <tr>
    <td></td>
    <td></td>
  </tr>
  <tr>
    <td></td>
    <td></td>
  </tr>
</tbody></table>
```

<a id="example-e71aef50"></a>

<strong>Example:</strong>

[](#example-e71aef50)

``` text
<!-- built using dom api, as this would be fixed by the html parser -->
```

``` text
<grid style="display: table">
  <row style="display: table-row">
    <th rowspan="2">1</th>
    <colgroup style="display: table-cell" span="2" colspan="2">2</colgroup>
  </row>
  <tr>
    <td>A</td>
    <td>B</td>
    <td>C</td>
  </tr>
</grid>
```

produces the same row/column grid as

``` text
<table>
  <tr>
    <th rowspan="2">1</th>
    <td>2</td>
  </tr>
  <tr>
    <td>A</td>
    <td>B</td>
    <td>C</td>
  </tr>
</table>
```

Note how the second cell of the first row doesn’t have \`\`\`colspan=2\`\`\` applied, because its originating element is not an HTML TD element.

[Testcase](https://jsfiddle.net/eqrwaLyc/). [!!Testcase](https://jsfiddle.net/eqrwaLyc/1/). [!Test case](https://jsfiddle.net/sckxeLmh/2/). [!!Test case](https://jsfiddle.net/sckxeLmh/4/). [!!Test case](https://jsfiddle.net/sckxeLmh/5/).

#### <a id="dimensioning-the-row-column-grid--step2"></a>3.3.2. Track merging[](#dimensioning-the-row-column-grid--step2)

The HTML Table Formatting algorithm sometimes generates more tracks than necessary to layout the table properly. Those tracks have historically been ignored by user agents, so the next step just gets rid of them entirely to avoid dealing with them as exceptions later in the spec. We have tried to maintain the functionality with this change, but if you happen to find any issues due to this change please file an issue.

<strong>Note:</strong>

Modify iteratively the obtained grid by merging consecutive tracks as follows: As long as there exists an <a id="ref-for-eligible-track"></a>[eligible track](#eligible-track) in the obtained row/column grid such that there is no table-column/table-row box defining the said track explicitly, and both the said track and the previous one are spanned by the exact same set of cells, those two tracks must be merged into one single track for the purpose of computing the layout of the table. Reduce the <a id="ref-for-span①"></a>[span](https://drafts.csswg.org/css-tables-3/#span) of the cells that spanned the deleted track by one to compensate, and shift similarly the tracks from which cells <a id="ref-for-originate"></a>[originate](https://drafts.csswg.org/css-tables-3/#originate) when needed. (see [spanning-ghost-rows test cases](https://drafts.csswg.org/css-tables-3/#spanning-ghost-rows))

For tables <a id="ref-for-in-auto-mode"></a>[in auto mode](https://drafts.csswg.org/css-tables-3/#in-auto-mode), every track is an <a id="eligible-track"></a><strong>eligible track</strong> for the purpose of the track-merging algorithm. For tables <a id="ref-for-in-fixed-mode"></a>[in fixed mode](https://drafts.csswg.org/css-tables-3/#in-fixed-mode), only rows are eligible to be merged that way; which means that every column is preserved.

Finally, assign to the <a id="ref-for-table-root-element⑧"></a>[table-root](https://drafts.csswg.org/css-tables-3/#table-root-element) grid its correct number of rows and columns (from its mapped element), and to each <a id="ref-for-table-cell⑨"></a>[table-cell](https://drafts.csswg.org/css-tables-3/#table-cell) its accurate <a id="ref-for-slot④"></a>[rowStart/colStart/rowSpan/colSpan](https://drafts.csswg.org/css-tables-3/#slot) (from its mapped element).
