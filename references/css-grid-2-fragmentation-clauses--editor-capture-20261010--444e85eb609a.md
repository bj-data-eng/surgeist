Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-10. Source status: W3C Candidate Recommendation Draft, 26 March 2025.

Full captured HTML SHA-256: `05aa64853c4428146973943b35caf121e44c1076bdf5b8c29f8896dba9b778e2` (885659 bytes). Conversion input SHA-256: `444e85eb609a7e27708f04240dd60df1e104c651b398f6f519488d24d608a601` (53191 bytes). Page revision metadata: `84988981e65c49994b12209c52ff0160402ec15d`.

Retained scope: `sotd`, `pagination`. Selected heading sections are complete, bounded at the next equal-or-higher heading. The original header and legal notice are retained. Other clauses remain upstream. Figures remain upstream image links. Multiline alternative descriptions use HTML image attributes with numeric whitespace entities to retain exact source alt text.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table contents are checked against conversion input. Synthetic table headers and span expansion are representation changes. Two one-line Grid 3 table examples omit only their terminal separator newline when represented as inline code; their exact source text remains in conversion provenance. Mathematical expressions are retained as source text; no equation is corrected. This capture preserves standards latitude and unresolved questions; Surgeist-selected policies are recorded separately in issues.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Grid Layout Module Level 2

<a id="w3c-state"></a>[W3C Candidate Recommendation Draft](https://www.w3.org/standards/types/#CRD), 26 March 2025

More details about this document

<strong>This version:</strong>

<https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-grid-2/>

<strong>Editor's Draft:</strong>

<https://drafts.csswg.org/css-grid-2/>

<strong>History:</strong>

<https://www.w3.org/standards/history/css-grid-2/>

<strong>Implementation Report:</strong>

<https://wpt.fyi/results/css/css-grid/subgrid>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-grid-2)

[CSSWG GitHub](https://github.com/w3c/csswg-drafts/issues?q=is%3Aopen+is%3Aissue+label%3Acss-grid-2)

[Inline In Spec](https://www.w3.org/TR/css-grid-2/#issues-index)

<strong>Editors:</strong>

[Tab Atkins Jr.](http://www.xanthir.com/contact/) (Google)

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

[Rossen Atanassov](mailto:ratan@microsoft.com) (Microsoft)

[Oriol Brufau](mailto:obrufau@igalia.com) (Igalia)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-grid-2/Overview.bs)

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication. A list of current W3C publications and the latest revision of this technical report can be found in the [W3C standards and drafts index at https://www.w3.org/TR/.](https://www.w3.org/TR/)</em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20231103/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-grid” in the title, like this: “\[css-grid\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-grid%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[03 November 2023 W3C Process Document](https://www.w3.org/policies/process/20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

<a id="toc"></a>

## <a id="contents"></a>Table of Contents

1.  [1 Introduction](https://www.w3.org/TR/css-grid-2/#intro)
    1.  [1.1 Background and Motivation](https://www.w3.org/TR/css-grid-2/#background)
        1.  [1.1.1 Adapting Layouts to Available Space](https://www.w3.org/TR/css-grid-2/#adapting-to-available-space)
        2.  [1.1.2 Source-Order Independence](https://www.w3.org/TR/css-grid-2/#source-independence)
    2.  [1.2 Value Definitions](https://www.w3.org/TR/css-grid-2/#values)
2.  [2 Overview](https://www.w3.org/TR/css-grid-2/#overview)
    1.  [2.1 Declaring the Grid](https://www.w3.org/TR/css-grid-2/#overview-grid)
    2.  [2.2 Placing Items](https://www.w3.org/TR/css-grid-2/#overview-placement)
    3.  [2.3 Sizing the Grid](https://www.w3.org/TR/css-grid-2/#overview-sizing)
3.  [3 Grid Layout Concepts and Terminology](https://www.w3.org/TR/css-grid-2/#grid-concepts)
    1.  [3.1 Grid Lines](https://www.w3.org/TR/css-grid-2/#grid-line-concept)
    2.  [3.2 Grid Tracks and Cells](https://www.w3.org/TR/css-grid-2/#grid-track-concept)
    3.  [3.3 Grid Areas](https://www.w3.org/TR/css-grid-2/#grid-area-concept)
    4.  [3.4 Nested vs. Subgridded Items](https://www.w3.org/TR/css-grid-2/#subgrid-items)
4.  [4 Reordering and Accessibility](https://www.w3.org/TR/css-grid-2/#order-accessibility)
5.  [5 Grid Containers](https://www.w3.org/TR/css-grid-2/#grid-model)
    1.  [5.1 Establishing Grid Containers: the grid and inline-grid display values](https://www.w3.org/TR/css-grid-2/#grid-containers)
    2.  [5.2 Sizing Grid Containers](https://www.w3.org/TR/css-grid-2/#intrinsic-sizes)
    3.  [5.3 Scrollable Grid Overflow](https://www.w3.org/TR/css-grid-2/#overflow)
    4.  [5.4 Limiting Large Grids](https://www.w3.org/TR/css-grid-2/#overlarge-grids)
6.  [6 Grid Items](https://www.w3.org/TR/css-grid-2/#grid-items)
    1.  [6.1 Grid Item Display](https://www.w3.org/TR/css-grid-2/#grid-item-display)
    2.  [6.2 Grid Item Sizing](https://www.w3.org/TR/css-grid-2/#grid-item-sizing)
    3.  [6.3 Reordered Grid Items: the order property](https://www.w3.org/TR/css-grid-2/#order-property)
    4.  [6.4 Grid Item Margins and Paddings](https://www.w3.org/TR/css-grid-2/#item-margins)
    5.  [6.5 Z-axis Ordering: the z-index property](https://www.w3.org/TR/css-grid-2/#z-order)
    6.  [6.6 Automatic Minimum Size of Grid Items](https://www.w3.org/TR/css-grid-2/#min-size-auto)
7.  [7 Defining the Grid](https://www.w3.org/TR/css-grid-2/#grid-definition)
    1.  [7.1 The Explicit Grid](https://www.w3.org/TR/css-grid-2/#explicit-grids)
    2.  [7.2 Explicit Track Sizing: the grid-template-rows and grid-template-columns properties](https://www.w3.org/TR/css-grid-2/#track-sizing)
        1.  [7.2.1 Track Sizes](https://www.w3.org/TR/css-grid-2/#track-sizes)
        2.  [7.2.2 Naming Grid Lines: the \[\<custom-ident\>\*\] syntax](https://www.w3.org/TR/css-grid-2/#named-lines)
        3.  [7.2.3 Repeating Rows and Columns: the repeat() notation](https://www.w3.org/TR/css-grid-2/#repeat-notation)
            1.  [7.2.3.1 Syntax of repeat()](https://www.w3.org/TR/css-grid-2/#repeat-syntax)
            2.  [7.2.3.2 Repeat-to-fill: auto-fill and auto-fit repetitions](https://www.w3.org/TR/css-grid-2/#auto-repeat)
            3.  [7.2.3.3 Interpolation/Combination of repeat()](https://www.w3.org/TR/css-grid-2/#repeat-interpolation)
        4.  [7.2.4 Flexible Lengths: the fr unit](https://www.w3.org/TR/css-grid-2/#fr-unit)
        5.  [7.2.5 Computed Value of a Track Listing](https://www.w3.org/TR/css-grid-2/#computed-tracks)
        6.  [7.2.6 Resolved Value of a Track Listing](https://www.w3.org/TR/css-grid-2/#resolved-track-list)
            1.  [7.2.6.1 Resolved Value of a Standalone Track Listing](https://www.w3.org/TR/css-grid-2/#resolved-track-list-standalone)
            2.  [7.2.6.2 Resolved Value of a Subgridded Track Listing](https://www.w3.org/TR/css-grid-2/#resolved-track-list-subgrid)
    3.  [7.3 Named Areas: the grid-template-areas property](https://www.w3.org/TR/css-grid-2/#grid-template-areas-property)
        1.  [7.3.1 Serialization Of Template Strings](https://www.w3.org/TR/css-grid-2/#serialize-template)
        2.  [7.3.2 Implicitly-Assigned Line Names](https://www.w3.org/TR/css-grid-2/#implicit-named-lines)
        3.  [7.3.3 Implicitly-Named Areas](https://www.w3.org/TR/css-grid-2/#implicit-named-areas)
    4.  [7.4 Explicit Grid Shorthand: the grid-template property](https://www.w3.org/TR/css-grid-2/#explicit-grid-shorthand)
    5.  [7.5 The Implicit Grid](https://www.w3.org/TR/css-grid-2/#implicit-grids)
    6.  [7.6 Implicit Track Sizing: the grid-auto-rows and grid-auto-columns properties](https://www.w3.org/TR/css-grid-2/#auto-tracks)
    7.  [7.7 Automatic Placement: the grid-auto-flow property](https://www.w3.org/TR/css-grid-2/#grid-auto-flow-property)
    8.  [7.8 Grid Definition Shorthand: the grid property](https://www.w3.org/TR/css-grid-2/#grid-shorthand)
8.  [8 Placing Grid Items](https://www.w3.org/TR/css-grid-2/#placement)
    1.  [8.1 Common Patterns for Grid Placement](https://www.w3.org/TR/css-grid-2/#common-uses)
        1.  [8.1.1 Named Areas](https://www.w3.org/TR/css-grid-2/#common-uses-named-areas)
        2.  [8.1.2 Numeric Indexes and Spans](https://www.w3.org/TR/css-grid-2/#common-uses-numeric)
        3.  [8.1.3 Named Lines and Spans](https://www.w3.org/TR/css-grid-2/#common-uses-named-lines)
        4.  [8.1.4 Auto Placement](https://www.w3.org/TR/css-grid-2/#common-uses-auto-placement)
    2.  [8.2 Grid Item Placement vs. Source Order](https://www.w3.org/TR/css-grid-2/#placement-a11y)
    3.  [8.3 Line-based Placement: the grid-row-start, grid-column-start, grid-row-end, and grid-column-end properties](https://www.w3.org/TR/css-grid-2/#line-placement)
        1.  [8.3.1 Grid Placement Conflict Handling](https://www.w3.org/TR/css-grid-2/#grid-placement-errors)
    4.  [8.4 Placement Shorthands: the grid-column, grid-row, and grid-area properties](https://www.w3.org/TR/css-grid-2/#placement-shorthands)
    5.  [8.5 Grid Item Placement Algorithm](https://www.w3.org/TR/css-grid-2/#auto-placement-algo)
9.  [9 Subgrids](https://www.w3.org/TR/css-grid-2/#subgrids)
10. [10 Absolute Positioning](https://www.w3.org/TR/css-grid-2/#abspos)
    1.  [10.1 With a Grid Container as Containing Block](https://www.w3.org/TR/css-grid-2/#abspos-items)
    2.  [10.2 With a Grid Container as Parent](https://www.w3.org/TR/css-grid-2/#static-position)
11. [11 Alignment and Spacing](https://www.w3.org/TR/css-grid-2/#alignment)
    1.  [11.1 Gutters: the row-gap, column-gap, and gap properties](https://www.w3.org/TR/css-grid-2/#gutters)
    2.  [11.2 Aligning with auto margins](https://www.w3.org/TR/css-grid-2/#auto-margins)
    3.  [11.3 Inline-axis Alignment: the justify-self and justify-items properties](https://www.w3.org/TR/css-grid-2/#row-align)
    4.  [11.4 Block-axis Alignment: the align-self and align-items properties](https://www.w3.org/TR/css-grid-2/#column-align)
    5.  [11.5 Aligning the Grid: the justify-content and align-content properties](https://www.w3.org/TR/css-grid-2/#grid-align)
    6.  [11.6 Grid Container Baselines](https://www.w3.org/TR/css-grid-2/#grid-baselines)
12. [12 Grid Layout Algorithm](https://www.w3.org/TR/css-grid-2/#layout-algorithm)
    1.  [12.1 Grid Sizing Algorithm](https://www.w3.org/TR/css-grid-2/#algo-grid-sizing)
    2.  [12.2 Track Sizing Terminology](https://www.w3.org/TR/css-grid-2/#algo-terms)
    3.  [12.3 Track Sizing Algorithm](https://www.w3.org/TR/css-grid-2/#algo-track-sizing)
    4.  [12.4 Initialize Track Sizes](https://www.w3.org/TR/css-grid-2/#algo-init)
    5.  [12.5 Resolve Intrinsic Track Sizes](https://www.w3.org/TR/css-grid-2/#algo-content)
        1.  [12.5.1 Distributing Extra Space Across Spanned Tracks](https://www.w3.org/TR/css-grid-2/#extra-space)
    6.  [12.6 Maximize Tracks](https://www.w3.org/TR/css-grid-2/#algo-grow-tracks)
    7.  [12.7 Expand Flexible Tracks](https://www.w3.org/TR/css-grid-2/#algo-flex-tracks)
        1.  [12.7.1 Find the Size of an fr](https://www.w3.org/TR/css-grid-2/#algo-find-fr-size)
    8.  [12.8 Stretch auto Tracks](https://www.w3.org/TR/css-grid-2/#algo-stretch)
13. [13 Fragmenting Grid Layout](#pagination)
    1.  [13.1 Sample Fragmentation Algorithm](#fragmentation-alg)
14. [14 Privacy Considerations](https://www.w3.org/TR/css-grid-2/#privacy)
15. [15 Security Considerations](https://www.w3.org/TR/css-grid-2/#security)
16. [16 Changes](https://www.w3.org/TR/css-grid-2/#changes)
    1.  [16.1 Changes since the 18 December 2020 CR](https://www.w3.org/TR/css-grid-2/#changes-202012)
    2.  [16.2 Changes since the August 2020 CR](https://www.w3.org/TR/css-grid-2/#changes-202008)
    3.  [ Changes since the December 2019 CSS Grid Layout Level 2 Working Draft](https://www.w3.org/TR/css-grid-2/#changes-20180904)
    4.  [16.3 Additions Since Level 1](https://www.w3.org/TR/css-grid-2/#changes-2)
17. [17 Acknowledgements](https://www.w3.org/TR/css-grid-2/#acknowledgements)
18. [ Conformance](https://www.w3.org/TR/css-grid-2/#w3c-conformance)
    1.  [ Document conventions](https://www.w3.org/TR/css-grid-2/#w3c-conventions)
    2.  [ Conformance classes](https://www.w3.org/TR/css-grid-2/#w3c-conformance-classes)
    3.  [ Partial implementations](https://www.w3.org/TR/css-grid-2/#w3c-partial)
        1.  [ Implementations of Unstable and Proprietary Features](https://www.w3.org/TR/css-grid-2/#w3c-conform-future-proofing)
    4.  [ Non-experimental implementations](https://www.w3.org/TR/css-grid-2/#w3c-testing)
    5.  [ CR exit criteria](https://www.w3.org/TR/css-grid-2/#w3c-cr-exit-criteria)
19. [ Index](https://www.w3.org/TR/css-grid-2/#index)
    1.  [ Terms defined by this specification](https://www.w3.org/TR/css-grid-2/#index-defined-here)
    2.  [ Terms defined by reference](https://www.w3.org/TR/css-grid-2/#index-defined-elsewhere)
20. [ References](https://www.w3.org/TR/css-grid-2/#references)
    1.  [ Normative References](https://www.w3.org/TR/css-grid-2/#normative)
    2.  [ Informative References](https://www.w3.org/TR/css-grid-2/#informative)
21. [ Property Index](https://www.w3.org/TR/css-grid-2/#property-index)
22. [ Issues Index](https://www.w3.org/TR/css-grid-2/#issues-index)

<a id="issue-436134ac"></a>

<strong>Issue:</strong>

[](#issue-436134ac) If you notice any inconsistencies between this Grid Layout Module and the [Flexible Box Layout Module](https://www.w3.org/TR/css-flexbox/), please report them to the CSSWG, as this is likely an error.

## <a id="pagination"></a>13.  Fragmenting Grid Layout[](#pagination)

<a id="ref-for-grid-container①②⑤"></a>[Grid containers](https://www.w3.org/TR/css-grid-2/#grid-container) can break across pages between rows or columns and inside items. The <a id="ref-for-propdef-break-before"></a>[break-\*](https://www.w3.org/TR/css-break-3/#propdef-break-before) properties apply to grid containers as normal for the formatting context in which they participate. This section defines how they apply to grid items and the contents of grid items.

The following breaking rules refer to the <a id="ref-for-fragmentation-container"></a>[fragmentation container](https://www.w3.org/TR/css-break-4/#fragmentation-container) as the “page”. The same rules apply in any other <a id="ref-for-fragmentation-context"></a>[fragmentation context](https://www.w3.org/TR/css-break-4/#fragmentation-context). (Substitute “page” with the appropriate <a id="ref-for-fragmentation-container①"></a>fragmentation container type as needed.) See the [CSS Fragmentation Module](https://www.w3.org/TR/css-break/) [\[CSS3-BREAK\]](https://www.w3.org/TR/css-grid-2/#biblio-css3-break).

The exact layout of a fragmented grid container is not defined in this level of Grid Layout. However, breaks inside a grid container are subject to the following rules:

- The <a id="ref-for-propdef-break-before①"></a>[break-before](https://www.w3.org/TR/css-break-3/#propdef-break-before) and <a id="ref-for-propdef-break-after"></a>[break-after](https://www.w3.org/TR/css-break-3/#propdef-break-after) properties on <a id="ref-for-grid-item①③①"></a>[grid items](https://www.w3.org/TR/css-grid-2/#grid-item) are propagated to their grid row. The <a id="ref-for-propdef-break-before②"></a>break-before property on the first row and the <a id="ref-for-propdef-break-after①"></a>break-after property on the last row are propagated to the grid container.
- A forced break inside a grid item effectively increases the size of its contents; it does not trigger a forced break inside sibling items.
- [Class A break opportunities](https://www.w3.org/TR/css3-break/#btw-blocks) occur between rows or columns (whichever is in the appropriate axis), and [Class C break opportunities](https://www.w3.org/TR/css3-break/#end-block) occur between the first/last row (column) and the grid container’s content edges. [\[CSS3-BREAK\]](https://www.w3.org/TR/css-grid-2/#biblio-css3-break)
- When a grid container is continued after a break, the space available to its <a id="ref-for-grid-item①③②"></a>[grid items](https://www.w3.org/TR/css-grid-2/#grid-item) (in the block flow direction of the fragmentation context) is reduced by the space consumed by grid container fragments on previous pages. The space consumed by a grid container fragment is the size of its content box on that page. If as a result of this adjustment the available space becomes negative, it is set to zero.
- Aside from the rearrangement of items imposed by the previous point, UAs should attempt to minimize distortion of the grid container with respect to unfragmented flow.

### <a id="fragmentation-alg"></a>13.1.  Sample Fragmentation Algorithm[](#fragmentation-alg)

<em>This section is non-normative.</em>

This is a rough draft of one possible fragmentation algorithm, and still needs to be severely cross-checked with the

<strong>Note:</strong>

[\[CSS-FLEXBOX-1\]](https://www.w3.org/TR/css-grid-2/#biblio-css-flexbox-1) algorithm for consistency. Feedback is welcome; please reference the rules above instead as implementation guidance.

1.  Layout the grid following the <a id="ref-for-layout-algorithm②"></a>[§ 12 Grid Layout Algorithm](https://www.w3.org/TR/css-grid-2/#layout-algorithm) by using the <a id="ref-for-fragmentation-container②"></a>[fragmentation container](https://www.w3.org/TR/css-break-4/#fragmentation-container)’s inline size and assume unlimited block size. During this step all <a id="ref-for-propdef-grid-row⑥"></a>[grid-row](https://www.w3.org/TR/css-grid-2/#propdef-grid-row) <a id="ref-for-grid-placement-auto①②"></a>[auto](https://www.w3.org/TR/css-grid-2/#grid-placement-auto) and <a id="ref-for-valdef-flex-fr⑨"></a>[fr](https://www.w3.org/TR/css-grid-2/#valdef-flex-fr) values must be resolved.
2.  Layout the grid container using the values resolved in the previous step.
3.  If a <a id="ref-for-grid-area③⑥"></a>[grid area](https://www.w3.org/TR/css-grid-2/#grid-area)’s size changes due to fragmentation (do not include items that span rows in this decision), increase the grid row size as necessary for rows that either:
    - have a content min track sizing function.
    - are in a grid that does not have an explicit height and the grid row is flexible.
4.  If the grid height is <a id="ref-for-grid-placement-auto①③"></a>[auto](https://www.w3.org/TR/css-grid-2/#grid-placement-auto), the height of the grid should be the sum of the final row sizes.
5.  If a grid area overflows the grid container due to margins being collapsed during fragmentation, extend the grid container to contain this grid area (this step is necessary in order to avoid circular layout dependencies due to fragmentation).

If the grid’s height is specified, steps three and four may cause the grid rows to overflow the grid.

<strong>Note:</strong>
