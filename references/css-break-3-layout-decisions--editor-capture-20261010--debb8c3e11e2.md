Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Fragmentation Module Level 3](https://drafts.csswg.org/css-break-3/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-10. Source status: Editor’s Draft, 22 October 2025.

Full captured HTML SHA-256: `854ffd2e0e97b08f218c80bcfcbf2d5b4fa9711ae2f695750d13d98925e00dc2` (273730 bytes). Conversion input SHA-256: `debb8c3e11e2bd1a6a2fb82f5d93e3a4a2f2432e05f23a3c624f101f0c23831a` (131399 bytes). Page revision metadata: `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`.

Retained scope: `sotd`, `fragmentation-model`, `breaking-controls`, `breaking-rules`, `breaking-boxes`. Selected heading sections are complete, bounded at the next equal-or-higher heading. The original header and legal notice are retained. Other clauses remain upstream. Figures remain upstream image links. Multiline alternative descriptions use HTML image attributes with numeric whitespace entities to retain exact source alt text.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table contents are checked against conversion input. Synthetic table headers and span expansion are representation changes. Two one-line Grid 3 table examples omit only their terminal separator newline when represented as inline code; their exact source text remains in conversion provenance. Mathematical expressions are retained as source text; no equation is corrected. This capture preserves standards latitude and unresolved questions; Surgeist-selected policies are recorded separately in issues.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# CSS Fragmentation Module Level 3 Breaking the Web, one fragment at a time

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 22 October 2025

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-break/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-break-3/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2016/CR-css-break-3-20160114/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-break-3)

<strong>Editors:</strong>

[Rossen Atanassov](mailto:ratan@microsoft.com) (Microsoft)

[Elika J. Etemad / fantasai](http://fantasai.inkedblade.net/contact) (Apple)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-break-3/Overview.bs)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-break/>

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="sotd"></a>Status of this document

This is a public copy of the editors’ draft. It is provided for discussion only and may change at any moment. Its publication here does not imply endorsement of its contents by W3C. Don’t cite this document other than as work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-break” in the title, like this: “\[css-break\] <em>…summary of comment…</em>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style&#64;w3.org](mailto:www-style@w3.org?Subject=%5Bcss-break%5D%20PUT%20SUBJECT%20HERE).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

The following features are at-risk, and may be dropped during the CR period:

- the <a id="ref-for-valdef-break-before-region"></a>[region](#valdef-break-before-region) and <a id="ref-for-valdef-break-before-avoid-region"></a>[avoid-region](#valdef-break-before-avoid-region) values of break-\*

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

<a id="toc"></a>

## <a id="contents"></a>Table of Contents

1.  [1 Introduction](https://drafts.csswg.org/css-break-3/#intro)
    1.  [1.1 Module Interactions](https://drafts.csswg.org/css-break-3/#placement)
    2.  [1.2 Value Definitions](https://drafts.csswg.org/css-break-3/#values)
2.  [2 Fragmentation Model and Terminology](#fragmentation-model)
    1.  [2.1 Parallel Fragmentation Flows](#parallel-flows)
    2.  [2.2 Nested Fragmentation Flows](#nested-flows)
3.  [3 Controlling Breaks](#breaking-controls)
    1.  [3.1 Breaks Between Boxes: the break-before and break-after properties](#break-between)
        1.  [ Generic Break Values](#generic-break-values)
        2.  [ Page Break Values](#page-break-values)
        3.  [ Column Break Values](#column-break-values)
        4.  [ Region Break Values](#region-break-values)
        5.  [3.1.1 Child→Parent Break Propagation](#break-propagation)
    2.  [3.2 Breaks Within Boxes: the break-inside property](#break-within)
    3.  [3.3 Breaks Between Lines: orphans, widows](#widows-orphans)
    4.  [3.4 Page Break Aliases: the page-break-before, page-break-after, and page-break-inside properties](#page-break-properties)
4.  [4 Rules for Breaking](#breaking-rules)
    1.  [4.1 Possible Break Points](#possible-breaks)
    2.  [4.2 Types of Breaks](#break-types)
    3.  [4.3 Forced Breaks](#forced-breaks)
    4.  [4.4 Unforced Breaks](#unforced-breaks)
    5.  [4.5 Optimizing Unforced Breaks](#best-breaks)
5.  [5 Box Model for Breaking](#breaking-boxes)
    1.  [5.1 Breaking into Varying-size Fragmentainers](#varying-size-boxes)
    2.  [5.2 Adjoining Margins at Breaks](#break-margins)
    3.  [5.3 Splitting Boxes](#box-splitting)
    4.  [5.4 Fragmented Borders and Backgrounds: the box-decoration-break property](#break-decoration)
        1.  [5.4.1 Joining Boxes for slice](#joining-boxes)
    5.  [5.5 Transforms, Positioning, and Pagination](#transforms)
6.  [ Changes](https://drafts.csswg.org/css-break-3/#changes)
7.  [ Conformance](https://drafts.csswg.org/css-break-3/#w3c-conformance)
    1.  [ Document conventions](https://drafts.csswg.org/css-break-3/#w3c-conventions)
    2.  [ Conformance classes](https://drafts.csswg.org/css-break-3/#w3c-conformance-classes)
    3.  [ Partial implementations](https://drafts.csswg.org/css-break-3/#w3c-partial)
        1.  [ Implementations of Unstable and Proprietary Features](https://drafts.csswg.org/css-break-3/#w3c-conform-future-proofing)
    4.  [ Non-experimental implementations](https://drafts.csswg.org/css-break-3/#w3c-testing)
8.  [ Index](https://drafts.csswg.org/css-break-3/#index)
    1.  [ Terms defined by this specification](https://drafts.csswg.org/css-break-3/#index-defined-here)
    2.  [ Terms defined by reference](https://drafts.csswg.org/css-break-3/#index-defined-elsewhere)
9.  [ References](https://drafts.csswg.org/css-break-3/#references)
    1.  [ Normative References](https://drafts.csswg.org/css-break-3/#normative)
    2.  [ Non-Normative References](https://drafts.csswg.org/css-break-3/#informative)
10. [ Property Index](https://drafts.csswg.org/css-break-3/#property-index)

## <a id="fragmentation-model"></a>2.  Fragmentation Model and Terminology[](#fragmentation-model)

<strong><a id="fragmentation-container"></a><strong>fragmentation container</strong> (<a id="fragmentainer"></a><strong>fragmentainer</strong>)</strong>

A box—such as a page box, column box, or region—that contains a portion (or all) of a <a id="ref-for-fragmented-flow"></a>[fragmented flow](#fragmented-flow). Fragmentainers can be pre-defined, or generated as needed. When breakable content would overflow a fragmentainer in the block dimension, it breaks into the next container in its <a id="ref-for-fragmentation-context"></a>[fragmentation context](#fragmentation-context) instead.

<strong><a id="fragmentation-context"></a><strong>fragmentation context</strong></strong>

An ordered series of <a id="ref-for-fragmentainer①"></a>[fragmentainers](#fragmentainer), such as created by a [multi-column container](https://www.w3.org/TR/css3-multicol/), a chain of [CSS regions](https://www.w3.org/TR/css3-regions), or a <a id="ref-for-paged-media②"></a>[paged media](https://drafts.csswg.org/mediaqueries-5/#paged-media) display. A given fragmentation context can only have one block flow direction across all its <a id="ref-for-fragmentainer②"></a>fragmentainers. (Descendants of the <a id="ref-for-fragmentation-root"></a>[fragmentation root](#fragmentation-root) may have other block flow directions, but fragmentation proceeds according to the block flow direction applied to the <a id="ref-for-fragmentation-root①"></a>fragmentation root.)

<strong><a id="fragmented-flow"></a><strong>fragmented flow</strong></strong>

Content that is being laid out in a <a id="ref-for-fragmentation-context①"></a>[fragmentation context](#fragmentation-context). The <a id="ref-for-fragmented-flow①"></a>[fragmented flow](#fragmented-flow) consists of the content of a (possibly anonymous) box called the <a id="fragmentation-root"></a><strong>fragmentation root</strong>.

<strong><a id="fragmentation-direction"></a><strong>fragmentation direction</strong></strong>

The block flow direction of the <a id="ref-for-fragmentation-context②"></a>[fragmentation context](#fragmentation-context), i.e. the direction in which content is fragmented. (In this level of CSS, content only fragments in one dimension.)

<strong><a id="fragmentation"></a><strong>fragmentation</strong></strong>

The process of splitting a content flow across the <a id="ref-for-fragmentainer③"></a>[fragmentainers](#fragmentainer) that form a <a id="ref-for-fragmentation-context③"></a>[fragmentation context](#fragmentation-context).

<strong><a id="box-fragment"></a><strong>box fragment</strong> or <a id="fragment"></a><strong>fragment</strong></strong>

The portion of a box that belongs to exactly one <a id="ref-for-fragmentainer④"></a>[fragmentainer](#fragmentainer). A box in continuous flow always consists of only one fragment. A box in a fragmented flow consists of one or more fragments. Each fragment has its own share of the box’s border, padding, and margin, and therefore has its own <a id="ref-for-padding-area"></a>[padding area](https://drafts.csswg.org/css-box-4/#padding-area), <a id="ref-for-border-area"></a>[border area](https://drafts.csswg.org/css-box-4/#border-area), and <a id="ref-for-margin-area"></a>[margin area](https://drafts.csswg.org/css-box-4/#margin-area). (See <a id="ref-for-propdef-box-decoration-break"></a>[box-decoration-break](#propdef-box-decoration-break), which controls how these are affected by fragmentation.)

<strong><a id="remaining-fragmentainer-extent"></a><strong>remaining fragmentainer extent</strong></strong>

The remaining <a id="ref-for-block-axis"></a>[block-axis](https://drafts.csswg.org/css-writing-modes-4/#block-axis) space in the <a id="ref-for-fragmentainer⑤"></a>[fragmentainer](#fragmentainer) available to a given element, i.e. between the end of preceding content in <a id="ref-for-fragmentainer⑥"></a>fragmentainer and the edge of the <a id="ref-for-fragmentainer⑦"></a>fragmentainer.

Each <a id="fragmentation-break"></a><strong>fragmentation break</strong> (hereafter, <a id="ref-for-fragmentation-break"></a>[break](#fragmentation-break)) ends layout of the fragmented box in the current <a id="ref-for-fragmentainer⑧"></a>[fragmentainer](#fragmentainer) and causes the remaining content to be laid out in the next <a id="ref-for-fragmentainer⑨"></a>fragmentainer, in some cases causing a new <a id="ref-for-fragmentainer①⓪"></a>fragmentainer to be generated to hold the deferred content.

Breaking inline content into lines is another form of fragmentation, and similarly creates box fragments when it breaks

<strong>Note:</strong>

[inline boxes](https://www.w3.org/TR/CSS21/visuren.html#inline-boxes) across [line boxes](https://www.w3.org/TR/CSS21/visuren.html#line-box). However, inline breaking is not covered here; see [\[CSS2\]](https://drafts.csswg.org/css-break-3/#biblio-css2)/[\[CSS3TEXT\]](https://drafts.csswg.org/css-break-3/#biblio-css3text).

A box can be broken into multiple

<strong>Note:</strong>

<a id="ref-for-fragment"></a>[fragments](#fragment) also due to bidi reordering of text (see [Applying the Bidirectional Reordering Algorithm](https://drafts.csswg.org/css-writing-modes-3/#bidi-algo) in [CSS Writing Modes](https://drafts.csswg.org/css-writing-modes-3/#text-direction)) or higher-level <a id="ref-for-display-type"></a>[display type](https://drafts.csswg.org/css-display-4/#display-type) box splitting, e.g. [block-in-inline splitting](https://www.w3.org/TR/CSS2/visuren.html#img-anon-block) (see [CSS2§9.2](https://www.w3.org/TR/CSS2/visuren.html#box-gen)) or [column-spanner-in-block](https://www.w3.org/TR/css-multicol-1/#spanning-columns) splitting (see [CSS Multi-column Layout](https://www.w3.org/TR/css-multicol-1/#spanning-columns)). The division into <a id="ref-for-box-fragment"></a>[box fragments](#box-fragment) in these cases does not depend on layout (sizing/positioning of content).

### <a id="parallel-flows"></a>2.1.  Parallel Fragmentation Flows[](#parallel-flows)

When multiple <a id="ref-for-formatting-context"></a>[formatting contexts](https://drafts.csswg.org/css-display-4/#formatting-context) are laid out parallel to each other, fragmentation is performed independently in each <a id="ref-for-formatting-context①"></a>formatting context. For example, if an element is floated, then a forced break inside the float will not affect the content outside the float (except insofar as it may increase the height of the float). UAs <em>may</em> (but are not required to) adjust the placement of <a id="ref-for-unforced-break"></a>[unforced breaks](#unforced-break) in parallel <a id="ref-for-formatting-context②"></a>formatting contexts to visually balance such side-by-side content, but <em>must not</em> do so to match a <a id="ref-for-forced-break"></a>[forced break](#forced-break).

The following are examples of parallel flows whose contents will fragment independently:

- The contents of a float vs. the content wrapping outside the float.
- The contents of a float vs. the contents of an adjacent float.
- The contents of each table cell in a single table row.
- The contents of each grid item in a single grid row.
- The contents of each flex item in a flex layout row.
- The contents of absolutely-positioned elements that cover the same range of their containing block’s fragmentation context.

Content overflowing the content edge of a fixed-size box is considered parallel to the content after the fixed-size box and follows the normal fragmentation rules. Although overflowing content doesn’t affect the size of the <a id="ref-for-fragmentation-root②"></a>[fragmentation root](#fragmentation-root) box, it does increase the length of the <a id="ref-for-fragmented-flow②"></a>[fragmented flow](#fragmented-flow), spilling into or generating additional <a id="ref-for-fragmentainer①①"></a>[fragmentainers](#fragmentainer) as necessary.

### <a id="nested-flows"></a>2.2.  Nested Fragmentation Flows[](#nested-flows)

Breaking a <a id="ref-for-fragmentainer①②"></a>[fragmentainer](#fragmentainer) <var>F</var> effectively splits the <a id="ref-for-fragmentainer①③"></a>fragmentainer into two <a id="ref-for-fragmentainer①④"></a>fragmentainers (<var>F<sub>1</sub></var> and <var>F<sub>2</sub></var>). The only difference is that, with regards to the content of <a id="ref-for-fragmentainer①⑤"></a>fragmentainer <var>F</var>, the type of break between the two pieces <var>F<sub>1</sub></var> and <var>F<sub>2</sub></var> is the [type of break](#break-types) created by the <a id="ref-for-fragmentation-context④"></a>[fragmentation context](#fragmentation-context) that split <var>F</var>, not the type of break normally created by <var>F</var>’s own <a id="ref-for-fragmentation-context⑤"></a>fragmentation context.

<a id="example-77c2ce76"></a>

<strong>Example:</strong>

[](#example-77c2ce76) For example, if a region box is broken at a page boundary, then the content of the region will be affected by a page break at that point (but not by a region break).

Note that when a multi-column container breaks across pages, it generates a new row of columns on the next page for the rest of its content, so that a page break within a multi-column container is always both a page break and a column break.

<strong>Note:</strong>

## <a id="breaking-controls"></a>3.  Controlling Breaks[](#breaking-controls)

The following sections explain how breaks are controlled in a <a id="ref-for-fragmented-flow③"></a>[fragmented flow](#fragmented-flow). A page/column/region break opportunity between two boxes is under the influence of the containing block’s <a id="ref-for-propdef-break-inside"></a>[break-inside](#propdef-break-inside) property, the <a id="ref-for-propdef-break-after"></a>[break-after](#propdef-break-after) property of the preceding element, and the <a id="ref-for-propdef-break-before"></a>[break-before](#propdef-break-before) property of the following element. A page/column/region break opportunity between line boxes is under the influence of the containing block’s <a id="ref-for-propdef-break-inside①"></a>break-inside, <a id="ref-for-propdef-widows"></a>[widows](#propdef-widows), and <a id="ref-for-propdef-orphans"></a>[orphans](#propdef-orphans) properties. A fragmentation break can be allowed, forced, or discouraged depending on the values of these properties. A forced break overrides any break restrictions acting at that break point. In the case of forced page breaks, the author can also specify on which page ([left or right](https://www.w3.org/TR/css3-page/#left-right-first)) the subsequent content should resume.

See the section on [rules for breaking](#breaking-rules) for the exact rules on how these properties affect fragmentation.

### <a id="break-between"></a>3.1.  Breaks Between Boxes: the <a id="ref-for-propdef-break-before①"></a>[break-before](#propdef-break-before) and <a id="ref-for-propdef-break-after①"></a>[break-after](#propdef-break-after) properties[](#break-between)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
|------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-break-before"></a><strong>break-before</strong>, <a id="propdef-break-after"></a><strong>break-after</strong>                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | auto <a id="ref-for-comb-one"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) avoid <a id="ref-for-comb-one①"></a>\| avoid-page <a id="ref-for-comb-one②"></a>\| page <a id="ref-for-comb-one③"></a>\| left <a id="ref-for-comb-one④"></a>\| right <a id="ref-for-comb-one⑤"></a>\| recto <a id="ref-for-comb-one⑥"></a>\| verso <a id="ref-for-comb-one⑦"></a>\| avoid-column <a id="ref-for-comb-one⑧"></a>\| column <a id="ref-for-comb-one⑨"></a>\| avoid-region <a id="ref-for-comb-one①⓪"></a>\| region |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | auto                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | block-level boxes, grid items, flex items, table row groups, table rows (but see prose)                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | specified keyword                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |

These properties specify page/column/region break behavior before/after the generated box. The <a id="forced-break-values"></a><strong>forced break values</strong> <a id="ref-for-valdef-break-before-left"></a>[left](#valdef-break-before-left), <a id="ref-for-valdef-break-before-right"></a>[right](#valdef-break-before-right), <a id="ref-for-valdef-break-before-recto"></a>[recto](#valdef-break-before-recto), <a id="ref-for-valdef-break-before-verso"></a>[verso](#valdef-break-before-verso), <a id="ref-for-valdef-break-before-page"></a>[page](#valdef-break-before-page), <a id="ref-for-valdef-break-before-column"></a>[column](#valdef-break-before-column) and <a id="ref-for-valdef-break-before-region①"></a>[region](#valdef-break-before-region) create a [forced break](#forced-breaks) in the flow while the <a id="avoid-break-values"></a><strong>avoid break values</strong> <a id="ref-for-valdef-break-before-avoid"></a>[avoid](#valdef-break-before-avoid), <a id="ref-for-valdef-break-before-avoid-page"></a>[avoid-page](#valdef-break-before-avoid-page), <a id="ref-for-valdef-break-before-avoid-column"></a>[avoid-column](#valdef-break-before-avoid-column) and <a id="ref-for-valdef-break-before-avoid-region①"></a>[avoid-region](#valdef-break-before-avoid-region) indicate that content should be kept together.

Values for <a id="ref-for-propdef-break-before②"></a>[break-before](#propdef-break-before) and <a id="ref-for-propdef-break-after②"></a>[break-after](#propdef-break-after) are defined in the sub-sections below. User agents must apply these properties to boxes in the normal flow of the <a id="ref-for-fragmentation-root③"></a>[fragmentation root](#fragmentation-root). User agents should also apply these properties to floated boxes whose containing block is in the normal flow of the root fragmented element. User agents may also apply these properties to other boxes. User agents must not apply these properties to absolutely-positioned boxes.

#### <a id="generic-break-values"></a> Generic Break Values[](#generic-break-values)

These values have an effect regardless of the type of fragmented context containing the flow.

<strong><a id="valdef-break-before-auto"></a><strong>auto</strong></strong>

Neither force nor forbid a break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

<strong><a id="valdef-break-before-avoid"></a><strong>avoid</strong></strong>

Avoid a break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

#### <a id="page-break-values"></a> Page Break Values[](#page-break-values)

These values only have an effect in paginated contexts; if the flow is not paginated, they have no effect.

<strong><a id="valdef-break-before-avoid-page"></a><strong>avoid-page</strong></strong>

Avoid a page break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

<strong><a id="valdef-break-before-page"></a><strong>page</strong></strong>

Always force a page break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

<strong><a id="valdef-break-before-left"></a><strong>left</strong></strong>

Force one or two page breaks before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes) so that the next page is formatted as a left page.

<strong><a id="valdef-break-before-right"></a><strong>right</strong></strong>

Force one or two page breaks before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes) so that the next page is formatted as a right page.

<strong><a id="valdef-break-before-recto"></a><strong>recto</strong></strong>

Force one or two page breaks before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes) so that the next page is formatted as either a left page or a right page, whichever is second (according to the <a id="ref-for-page-progression"></a>[page progression](https://drafts.csswg.org/css-page-3/#page-progression)) in a page spread.

<strong><a id="valdef-break-before-verso"></a><strong>verso</strong></strong>

Force one or two page breaks before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes) so that the next page is formatted as either a left page or a right page, whichever is first (according to the <a id="ref-for-page-progression①"></a>[page progression](https://drafts.csswg.org/css-page-3/#page-progression)) in a page spread.

#### <a id="column-break-values"></a> Column Break Values[](#column-break-values)

These values only have an effect in multi-column contexts; if the flow is not within a multi-column context, they have no effect.

<strong><a id="valdef-break-before-avoid-column"></a><strong>avoid-column</strong></strong>

Avoid a column break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

<strong><a id="valdef-break-before-column"></a><strong>column</strong></strong>

Always force a column break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

#### <a id="region-break-values"></a> Region Break Values[](#region-break-values)

These values only have an effect in multi-region contexts; if the flow is not linked across multiple regions, these values have no effect.

<strong><a id="valdef-break-before-avoid-region"></a><strong>avoid-region</strong></strong>

Avoid a region break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

<strong><a id="valdef-break-before-region"></a><strong>region</strong></strong>

Always force a region break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

#### <a id="break-propagation"></a>3.1.1.  Child→Parent Break Propagation[](#break-propagation)

Since breaks are only allowed between siblings, not between a box and its container (see [Possible Break Points](#possible-breaks)), break values applied to children at the start/end of a parent are <a id="propagate"></a><strong>propagated</strong> to the parent, where they can take effect.

Specifically—​except in layout modes which define more specific rules to account for reordering and parallel layout (e.g. in [flex layout](http://www.w3.org/TR/css-flexbox-1/#pagination) [\[CSS-FLEXBOX-1\]](https://drafts.csswg.org/css-break-3/#biblio-css-flexbox-1) or [grid layout](http://www.w3.org/TR/css-grid-1/#pagination) [\[CSS-GRID-1\]](https://drafts.csswg.org/css-break-3/#biblio-css-grid-1))—​a <a id="ref-for-propdef-break-before③"></a>[break-before](#propdef-break-before) value on a first <a id="ref-for-in-flow"></a>[in-flow](https://drafts.csswg.org/css-display-4/#in-flow) child box is <a id="ref-for-propagate"></a>[propagated](#propagate) to its container. Likewise a <a id="ref-for-propdef-break-after③"></a>[break-after](#propdef-break-after) value on a last <a id="ref-for-in-flow①"></a>in-flow child box is <a id="ref-for-propagate①"></a>propagated to its container. (Conflicting values [combine](#forced-breaks) as defined below.) This propagation stops before it breaks through the nearest matching fragmentation context.

Break <a id="ref-for-propagate②"></a>[propagation](#propagate) does not affect <a id="ref-for-computed-value"></a>[computed values](https://drafts.csswg.org/css-cascade-5/#computed-value); it is part of interpreting the elements’ computed values for layout.

### <a id="break-within"></a>3.2.  Breaks Within Boxes: the <a id="ref-for-propdef-break-inside②"></a>[break-inside](#propdef-break-inside) property[](#break-within)

| Field                                                                                    | Definition                                                                                                                                                                                                                                 |
|------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-break-inside"></a><strong>break-inside</strong>                                                                                                                                                                             |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | auto <a id="ref-for-comb-one①①"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) avoid <a id="ref-for-comb-one①②"></a>\| avoid-page <a id="ref-for-comb-one①③"></a>\| avoid-column <a id="ref-for-comb-one①④"></a>\| avoid-region |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | auto                                                                                                                                                                                                                                       |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | all elements except inline-level boxes, internal ruby boxes, table column boxes, table column group boxes, absolutely-positioned boxes                                                                                                     |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                                                                                                                         |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                                                                                                                                                        |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | specified keyword                                                                                                                                                                                                                          |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | discrete                                                                                                                                                                                                                                   |

This property specifies page/column/region break behavior within the element’s principal box. Values have the following meanings:

<strong><a id="valdef-break-inside-auto"></a><strong>auto</strong></strong>

Impose no additional breaking constraints within the box.

<strong><a id="valdef-break-inside-avoid"></a><strong>avoid</strong></strong>

Avoid breaks within the box.

<strong><a id="valdef-break-inside-avoid-page"></a><strong>avoid-page</strong></strong>

Avoid a page break within the box.

<strong><a id="valdef-break-inside-avoid-column"></a><strong>avoid-column</strong></strong>

Avoid a column break within the box.

<strong><a id="valdef-break-inside-avoid-region"></a><strong>avoid-region</strong></strong>

Avoid a region break within the box.

### <a id="widows-orphans"></a>3.3.  Breaks Between Lines: <a id="ref-for-propdef-orphans①"></a>[orphans](#propdef-orphans), <a id="ref-for-propdef-widows①"></a>[widows](#propdef-widows)[](#widows-orphans)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                     |
|------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-orphans"></a><strong>orphans</strong>, <a id="propdef-widows"></a><strong>widows</strong>                                                                                                                                                                       |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | <a id="ref-for-integer-value"></a>[\<integer \[1,∞\]\>](https://drafts.csswg.org/css-values-4/#integer-value)                                                                                                                                                                  |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | 2                                                                                                                                                                                                                                                                              |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | <a id="ref-for-block-container"></a>[block containers](https://drafts.csswg.org/css-display-4/#block-container) that establish an <a id="ref-for-inline-formatting-context"></a>[inline formatting context](https://drafts.csswg.org/css-display-4/#inline-formatting-context) |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | yes                                                                                                                                                                                                                                                                            |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                                                                                                                                                                                                            |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | specified integer                                                                                                                                                                                                                                                              |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                    |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | by computed value type                                                                                                                                                                                                                                                         |

The <a id="ref-for-propdef-orphans②"></a>[orphans](#propdef-orphans) property specifies the minimum number of line boxes in a block container that must be left in a <a id="ref-for-fragment①"></a>[fragment](#fragment) <em>before</em> a fragmentation break. The <a id="ref-for-propdef-widows②"></a>[widows](#propdef-widows) property specifies the minimum number of line boxes of a block container that must be left in a <a id="ref-for-fragment②"></a>fragment <em>after</em> a break. Examples of how they are used to control fragmentation breaks are given [below](#widows-orphans-example).

Tests

- [widows-001.html](https://wpt.fyi/results/css/css-break/widows-001.html) [(live test)](http://wpt.live/css/css-break/widows-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-001.html)
- [widows-orphans-001.html](https://wpt.fyi/results/css/css-break/widows-orphans-001.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-001.html)
- [widows-orphans-002.html](https://wpt.fyi/results/css/css-break/widows-orphans-002.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-002.html)
- [widows-orphans-003.html](https://wpt.fyi/results/css/css-break/widows-orphans-003.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-003.html)
- [widows-orphans-004.html](https://wpt.fyi/results/css/css-break/widows-orphans-004.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-004.html)
- [widows-orphans-005.html](https://wpt.fyi/results/css/css-break/widows-orphans-005.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-005.html)
- [widows-orphans-006.html](https://wpt.fyi/results/css/css-break/widows-orphans-006.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-006.html)
- [widows-orphans-007.html](https://wpt.fyi/results/css/css-break/widows-orphans-007.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-007.html)
- [widows-orphans-008.html](https://wpt.fyi/results/css/css-break/widows-orphans-008.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-008.html)
- [widows-orphans-009.html](https://wpt.fyi/results/css/css-break/widows-orphans-009.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-009.html)
- [widows-orphans-010.html](https://wpt.fyi/results/css/css-break/widows-orphans-010.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-010.html)
- [widows-orphans-011.html](https://wpt.fyi/results/css/css-break/widows-orphans-011.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-011.html)
- [widows-orphans-012.html](https://wpt.fyi/results/css/css-break/widows-orphans-012.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-012.html)
- [widows-orphans-013.html](https://wpt.fyi/results/css/css-break/widows-orphans-013.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-013.html)
- [widows-orphans-014.html](https://wpt.fyi/results/css/css-break/widows-orphans-014.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-014.html)
- [widows-orphans-015.html](https://wpt.fyi/results/css/css-break/widows-orphans-015.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-015.html)
- [widows-orphans-016.html](https://wpt.fyi/results/css/css-break/widows-orphans-016.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-016.html)
- [widows-orphans-017.html](https://wpt.fyi/results/css/css-break/widows-orphans-017.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-017.html)
- [widows-orphans-018.html](https://wpt.fyi/results/css/css-break/widows-orphans-018.html) [(live test)](http://wpt.live/css/css-break/widows-orphans-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-break/widows-orphans-018.html)

Only positive integers are allowed as values of <a id="ref-for-propdef-orphans③"></a>[orphans](#propdef-orphans) and <a id="ref-for-propdef-widows③"></a>[widows](#propdef-widows). Negative values and zero are invalid and must cause the declaration to be [ignored](https://www.w3.org/TR/CSS21/conform.html#ignore).

If a block contains fewer lines than the value of <a id="ref-for-propdef-widows④"></a>[widows](#propdef-widows) or <a id="ref-for-propdef-orphans④"></a>[orphans](#propdef-orphans), the rule simply becomes that all lines in the block must be kept together.

### <a id="page-break-properties"></a>3.4.  Page Break Aliases: the <a id="ref-for-propdef-page-break-before"></a>[page-break-before](https://drafts.csswg.org/css2/#propdef-page-break-before), <a id="ref-for-propdef-page-break-after"></a>[page-break-after](https://drafts.csswg.org/css2/#propdef-page-break-after), and <a id="ref-for-propdef-page-break-inside"></a>[page-break-inside](https://drafts.csswg.org/css2/#propdef-page-break-inside) properties[](#page-break-properties)

For compatibility with [CSS Level 2](https://www.w3.org/TR/CSS21/page.html), UAs that conform to [\[CSS2\]](https://drafts.csswg.org/css-break-3/#biblio-css2) must alias the <a id="ref-for-propdef-page-break-before①"></a>[page-break-before](https://drafts.csswg.org/css2/#propdef-page-break-before), <a id="ref-for-propdef-page-break-after①"></a>[page-break-after](https://drafts.csswg.org/css2/#propdef-page-break-after), and <a id="ref-for-propdef-page-break-inside①"></a>[page-break-inside](https://drafts.csswg.org/css2/#propdef-page-break-inside) properties to <a id="ref-for-propdef-break-before④"></a>[break-before](#propdef-break-before), <a id="ref-for-propdef-break-after④"></a>[break-after](#propdef-break-after), and <a id="ref-for-propdef-break-inside③"></a>[break-inside](#propdef-break-inside) by treating the page-break-\* properties as <a id="ref-for-legacy-shorthand"></a>[legacy shorthands](https://drafts.csswg.org/css-cascade-5/#legacy-shorthand) for the break-\* properties with the following value mappings:

| Shorthand (page-break-\*) Values | Longhand (break-\*) Values     |
|----------------------------------|--------------------------------|
| auto \| left \| right \| avoid   | auto \| left \| right \| avoid |
| always                           | page                           |

## <a id="breaking-rules"></a>4.  Rules for Breaking[](#breaking-rules)

A <a id="ref-for-fragmented-flow④"></a>[fragmented flow](#fragmented-flow) may be broken across <a id="ref-for-fragmentainer①⑥"></a>[fragmentainers](#fragmentainer) at a number of [possible break points](#possible-breaks). In the case of [forced breaks](#forced-breaks), the UA is required to break the flow at that point. In the case of [unforced breaks](#unforced-breaks), the UA has to choose among the possible breaks that are allowed.

To guarantee progress, fragmentainers are assumed to have a minimum <a id="ref-for-block-size"></a>[block size](https://drafts.csswg.org/css-sizing-3/#block-size) of 1px regardless of their used size.

### <a id="possible-breaks"></a>4.1.  Possible Break Points[](#possible-breaks)

Fragmentation splits boxes in the block flow dimension. In block-and-inline flow, breaks may occur at the following places:

<a id="btw-blocks"></a><strong>[](#btw-blocks)Class A</strong>

Between sibling boxes of the following types:

<strong>Block-parallel Fragmentation</strong>

When the block flow direction of the siblings' containing block is parallel to that of the fragmentation context: [in-flow](https://www.w3.org/TR/2011/REC-CSS2-20110607/visuren.html#positioning-scheme) block-level boxes, a float and an immediately-adjacent in-flow or floated box, table row group boxes, table row boxes, multi-column column row boxes.

<strong>Block-perpendicular Fragmentation</strong>

When the block flow direction of the siblings' containing block is perpendicular to that of the fragmentation context: table column group boxes, table column boxes, multi-column column boxes.

<a id="btw-lines"></a><strong>[](#btw-lines)Class B</strong>

Between line boxes inside a block container box.

<a id="end-block"></a><strong>[](#end-block)Class C</strong>

Between the content edge of a block container box and the outer edges of its child content (margin edges of block-level children or line box edges for inline-level children) <em>if</em> there is a (non-zero) gap between them.

There is no inherent prioritization among these classes of break points. However, individual break points may be prioritized or de-prioritized by using the

<strong>Note:</strong>

[breaking controls](#breaking-controls).

Other layout models may add breakpoints to the above classes. For example,

<strong>Note:</strong>

[\[CSS-FLEXBOX-1\]](https://drafts.csswg.org/css-break-3/#biblio-css-flexbox-1) adds certain points within a flex formatting context to classes A and C.

Some content is not fragmentable, for example many types of [replaced elements](https://www.w3.org/TR/CSS21/conform.html#replaced-element) [\[CSS2\]](https://drafts.csswg.org/css-break-3/#biblio-css2) (such as images or video), scrollable elements, or a single line of text content. Such content is considered <a id="monolithic"></a><strong>monolithic</strong>: it contains no possible break points. Any forced breaks within such boxes therefore cannot split the box, and must therefore also be ignored by the box’s own fragmentation context.

In addition to any content which is not generally fragmentable, UAs may consider as <a id="ref-for-monolithic"></a>[monolithic](#monolithic) any elements with <a id="ref-for-propdef-overflow"></a>[overflow](https://drafts.csswg.org/css-overflow-3/#propdef-overflow) set to <a id="ref-for-valdef-overflow-auto"></a>[auto](https://drafts.csswg.org/css-overflow-3/#valdef-overflow-auto) or <a id="ref-for-valdef-overflow-scroll"></a>[scroll](https://drafts.csswg.org/css-overflow-3/#valdef-overflow-scroll) and any elements with <a id="ref-for-propdef-overflow①"></a>overflow: hidden and a non-<a id="ref-for-valdef-width-auto"></a>[auto](https://drafts.csswg.org/css-sizing-3/#valdef-width-auto) [logical height](https://www.w3.org/TR/css3-writing-modes/#block-size) (and no specified maximum logical height).

Since line boxes contain no possible break points, <a id="ref-for-valdef-display-inline-block"></a>[inline-block](https://drafts.csswg.org/css-display-4/#valdef-display-inline-block) and <a id="ref-for-valdef-display-inline-table"></a>[inline-table](https://drafts.csswg.org/css-display-4/#valdef-display-inline-table) boxes (and other inline-level <a id="ref-for-display-type①"></a>[display types](https://drafts.csswg.org/css-display-4/#display-type) that establish an <a id="ref-for-independent-formatting-context"></a>[independent formatting context](https://drafts.csswg.org/css-display-4/#independent-formatting-context)) may also be considered <a id="ref-for-monolithic①"></a>[monolithic](#monolithic): that is, in the cases where a single line box is too large to fit within its fragmentainer even by itself and the UA chooses to split the line box, it may fragment such boxes or it may treat them as monolithic.

### <a id="break-types"></a>4.2.  Types of Breaks[](#break-types)

There are different types of breaks in CSS, defined based on the type of fragmentainers they span:

<strong><a id="page-break"></a><strong>page break</strong></strong>

A break between two [page boxes](https://www.w3.org/TR/css3-page/#page-box). [\[CSS3PAGE\]](https://drafts.csswg.org/css-break-3/#biblio-css3page)

<strong><a id="spread-break"></a><strong>spread break</strong></strong>

A break between two page boxes that are not associated with [facing pages](https://www.w3.org/TR/css3-page/#facing-pages). A spread break is always also a page break. [\[CSS3PAGE\]](https://drafts.csswg.org/css-break-3/#biblio-css3page)

<strong><a id="column-break"></a><strong>column break</strong></strong>

A break between two [column boxes](https://www.w3.org/TR/css3-multicol/#column-box). Note that if the column boxes are on different pages, then the break is also a <a id="ref-for-page-break"></a>[page break](#page-break). Similarly, if the column boxes are in different regions, then the break is also a <a id="ref-for-region-break"></a>[region break](#region-break). [\[CSS3COL\]](https://drafts.csswg.org/css-break-3/#biblio-css3col)

<strong><a id="region-break"></a><strong>region break</strong></strong>

A break between two [regions](https://www.w3.org/TR/css3-regions/#regions). Note that if the region boxes are on different pages, then the break is also a <a id="ref-for-page-break①"></a>[page break](#page-break). [\[CSS3-REGIONS\]](https://drafts.csswg.org/css-break-3/#biblio-css3-regions)

A fifth type of break is the

<strong>Note:</strong>

<a id="ref-for-line-break"></a>[line break](https://drafts.csswg.org/css-text-4/#line-break), which is a break between two [line boxes](https://www.w3.org/TR/CSS21/visuren.html#line-box). These are not covered in this specification; see [\[CSS2\]](https://drafts.csswg.org/css-break-3/#biblio-css2) [\[CSS3TEXT\]](https://drafts.csswg.org/css-break-3/#biblio-css3text).

### <a id="forced-breaks"></a>4.3.  Forced Breaks[](#forced-breaks)

A <a id="forced-break"></a><strong>forced break</strong> is one explicitly indicated by the style sheet author. A <a id="ref-for-forced-break①"></a>[forced break](#forced-break) occurs at a [class A break point](#btw-blocks) if, among the <a id="ref-for-propdef-break-after⑤"></a>[break-after](#propdef-break-after) properties specified on or <a id="ref-for-propagate③"></a>[propagated](#propagate) to the earlier sibling box and the <a id="ref-for-propdef-break-before⑤"></a>[break-before](#propdef-break-before) properties specified on or <a id="ref-for-propagate④"></a>propagated to the later sibling box there is at least one with a <a id="ref-for-forced-break-values"></a>[forced break value](#forced-break-values). (Thus a <a id="ref-for-forced-break-values①"></a>forced break value effectively overrides any <a id="ref-for-avoid-break-values"></a>[avoid break value](#avoid-break-values) that also applies at that break point.)

When multiple <a id="ref-for-forced-break-values②"></a>[forced break values](#forced-break-values) apply to a single break point, they combine such that all types of break are honored. When <a id="ref-for-valdef-break-before-left①"></a>[left](#valdef-break-before-left), <a id="ref-for-valdef-break-before-right①"></a>[right](#valdef-break-before-right), <a id="ref-for-valdef-break-before-recto①"></a>[recto](#valdef-break-before-recto), and/or <a id="ref-for-valdef-break-before-verso①"></a>[verso](#valdef-break-before-verso) are combined, the value specified on the latest element in the flow wins.

A forced page break must also occur at a

<strong>Note:</strong>

[class A break point](#btw-blocks) if the last line box above this margin and the first one below it do not have the same value for <a id="ref-for-propdef-page"></a>[page](https://drafts.csswg.org/css-page-3/#propdef-page). See [\[CSS3PAGE\]](https://drafts.csswg.org/css-break-3/#biblio-css3page)

When a forced break occurs, it forces ensuing content into the next fragmentainer of the type associated with the break, breaking through as many fragmentation contexts as necessary until the specified break types are all satisfied. If the forced break is not contained within a matching type of fragmentation context, then the forced break has no effect.

### <a id="unforced-breaks"></a>4.4.  Unforced Breaks[](#unforced-breaks)

While [breaking controls](#breaking-controls) can force breaks, they can also discourage them. An <a id="unforced-break"></a><strong>unforced break</strong> is one that is inserted automatically by the UA in order to prevent content from overflowing the <a id="ref-for-fragmentainer①⑦"></a>[fragmentainer](#fragmentainer). The following rules control whether unforced breaking at a [possible break point](#possible-breaks) is allowed:

<strong>Rule 1</strong>

A <a id="ref-for-fragmented-flow⑤"></a>[fragmented flow](#fragmented-flow) may break at a [class A break point](#btw-blocks) only if all the <a id="ref-for-propdef-break-after⑥"></a>[break-after](#propdef-break-after) and <a id="ref-for-propdef-break-before⑥"></a>[break-before](#propdef-break-before) values applicable to this break point allow it, which is when at least one of them forces a break or when none of them forbid it (<a id="ref-for-valdef-break-before-avoid①"></a>[avoid](#valdef-break-before-avoid) or <a id="ref-for-valdef-break-before-avoid-page①"></a>[avoid-page](#valdef-break-before-avoid-page)/<a id="ref-for-valdef-break-before-avoid-column①"></a>[avoid-column](#valdef-break-before-avoid-column)/<a id="ref-for-valdef-break-before-avoid-region②"></a>[avoid-region](#valdef-break-before-avoid-region), depending on the [break type](#break-types)).

<strong>Rule 2</strong>

However, if all of them are <a id="ref-for-valdef-break-before-auto"></a>[auto](#valdef-break-before-auto) and a common ancestor of all the elements has a <a id="ref-for-propdef-break-inside④"></a>[break-inside](#propdef-break-inside) value of <a id="ref-for-valdef-break-inside-avoid"></a>[avoid](#valdef-break-inside-avoid), then breaking here is not allowed.

<strong>Rule 3</strong>

Breaking at a [class B break point](#btw-lines) is allowed only if the number of line boxes between the break and the start of the enclosing block box is the value of <a id="ref-for-propdef-orphans⑤"></a>[orphans](#propdef-orphans) or more, and the number of line boxes between the break and the end of the box is the value of <a id="ref-for-propdef-widows⑤"></a>[widows](#propdef-widows) or more.

<strong>Rule 4</strong>

Additionally, breaking at [class B](#btw-blocks) or [class C](#end-block) break points is allowed only if the <a id="ref-for-propdef-break-inside⑤"></a>[break-inside](#propdef-break-inside) property of all ancestors is <a id="ref-for-valdef-break-inside-auto"></a>[auto](#valdef-break-inside-auto).

If the above doesn’t provide enough break points to keep content from overflowing the <a id="ref-for-fragmentainer①⑧"></a>[fragmentainer](#fragmentainer), then rule 3 is dropped to provide more break points.

If that still does not lead to sufficient break points, then rules 1, 2 and 4 are dropped in order to find additional breakpoints. In this case the UA may use the avoids that are in effect at those points to weigh the appropriateness of the new breakpoints; however, this specification does not suggest a precise algorithm.

If even that does not lead to sufficient break points, <a id="ref-for-valdef-box-decoration-break-clone"></a>[cloned margins/border/padding](#valdef-box-decoration-break-clone) at the <a id="ref-for-block-end"></a>[block-end](https://drafts.csswg.org/css-writing-modes-4/#block-end) side are truncated; and if more room is still needed, <a id="ref-for-valdef-box-decoration-break-clone①"></a>[cloned margins/border/padding](#valdef-box-decoration-break-clone) are truncated at the <a id="ref-for-block-start"></a>[block-start](https://drafts.csswg.org/css-writing-modes-4/#block-start) side as well.

Finally, if there are no possible break points below the top of the fragmentainer, and not all the content fits, the UA may break anywhere in order to avoid losing content off the edge of the fragmentainer. <a id="monolithic-breaking"></a> In such cases, the UA may also fragment the contents of <a id="ref-for-monolithic②"></a>[monolithic](#monolithic) elements by slicing the element’s graphical representation. However, the UA must not break at the top of the page, i.e. it must place at least some content on each fragmentainer, so that each fragmentainer has a non-zero amount of content, in order to guarantee progress through the content.

### <a id="best-breaks"></a>4.5.  Optimizing Unforced Breaks[](#best-breaks)

While CSS3 requires that a <a id="ref-for-fragmented-flow⑥"></a>[fragmented flow](#fragmented-flow) must break at allowed break points in order to avoid overflowing the fragmentainers in its fragmentation context, it does not define whether content breaks at a particular [allowed break](#unforced-breaks). However, it is recommended that user agents observe the following guidelines (while recognizing that they are sometimes contradictory):

- Break as few times as possible.
- Make all fragmentainers that don’t end with a forced break appear to be equally filled with content.
- Avoid breaking inside a replaced element.

<a id="widows-orphans-example"></a>

<strong>Example:</strong>

[](#widows-orphans-example)

Suppose, for example, that the style sheet contains orphans : 4, widows : 2, and there is space for 20 lines (line boxes) available at the bottom of the current page, and the next block in normal flow is considered for placement:

- If the block contains 20 line boxes or fewer, it should be placed on the current page.
- If the block contains 21 or 22 line boxes, the second fragment of the paragraph must not violate the <a id="ref-for-propdef-widows⑥"></a>[widows](#propdef-widows) constraint, and so the second fragment must contain at least two line boxes; likewise the first fragment must contain at least four line boxes.
- If the block contains 23 line boxes or more, the first fragment should contain 20 lines and the second fragment the remaining lines. But if any fragment of the block is placed on the current page, that fragment must contain at least four line boxes and the second fragment at least two line boxes.

Now suppose that <a id="ref-for-propdef-orphans⑥"></a>[orphans](#propdef-orphans) is 10, <a id="ref-for-propdef-widows⑦"></a>[widows](#propdef-widows) is 20, and there are 8 lines available at the bottom of the current page:

- If the block contains 8 lines or fewer, it should be placed on the current page.
- If the block contains 9 lines or more, it must NOT be split (that would violate the <a id="ref-for-propdef-orphans⑦"></a>[orphans](#propdef-orphans) constraint), so it must move as a block to the next page.

Additionally, CSS imposes one requirement: a zero-sized <a id="ref-for-box-fragment①"></a>[box fragment](#box-fragment), since it does not take up space, must appear on the earlier side of a <a id="ref-for-fragmentation-break①"></a>[fragmentation break](#fragmentation-break) if it is able to fit within the <a id="ref-for-fragmentainer①⑨"></a>[fragmentainer](#fragmentainer).

Note: A zero-sized <a id="ref-for-box-fragment②"></a>[box fragment](#box-fragment) will be pushed to the next <a id="ref-for-fragmentainer②⓪"></a>[fragmentainer](#fragmentainer) if it is placed immediately after content that itself overflows the <a id="ref-for-fragmentainer②①"></a>fragmentainer.

## <a id="breaking-boxes"></a>5.  Box Model for Breaking[](#breaking-boxes)

The sizing terminology used in this section is defined in

<strong>Note:</strong>

[\[CSS3-SIZING\]](https://drafts.csswg.org/css-break-3/#biblio-css3-sizing).

### <a id="varying-size-boxes"></a>5.1.  Breaking into Varying-size Fragmentainers[](#varying-size-boxes)

When a flow is fragmented into varying-size fragmentainers, the following rules are observed for adapting layout:

- Layout is performed per-fragmentainer, with each fragmentainer continuing progress from the breakpoint on the previous, but recalculating sizes and positions using its own size as if the entire element were fragmented across fragmentainers of this size. Progress is measured in percentages (not absolute lengths) of used/remaining fragmentainer extent and in amount of used/remaining content. However, when laying out <a id="ref-for-monolithic③"></a>[monolithic](#monolithic) elements, the UA may instead maintain a consistent <a id="ref-for-inline-size"></a>[inline size](https://drafts.csswg.org/css-sizing-3/#inline-size) and resolved <a id="ref-for-block-size①"></a>[block size](https://drafts.csswg.org/css-sizing-3/#block-size) across fragmentainers.
- Intrinsic sizes are calculated and maintained across the entire element. Where an initial containing block size is needed to resolve an intrinsic size, assume the size of the first fragmentainer defining a fragmentation context.
- Fragments of boxes that began on a previous fragmentainer must obey placement rules with the additional constraint that fragments must not be positioned above the <a id="ref-for-block-start①"></a>[block-start](https://drafts.csswg.org/css-writing-modes-4/#block-start) edge of the fragmentainer. If this results in a box’s continuation fragment shifting away from the <a id="ref-for-block-start②"></a>block-start edge of the fragmentainer, then <a id="ref-for-propdef-box-decoration-break①"></a>[box-decoration-break: clone](#propdef-box-decoration-break), if specified, wraps the fragment with the box’s margin in addition to its padding and border.
  ![Illustration: Breaking in varying-size fragmentainers](https://drafts.csswg.org/css-break-3/images/Varying-Size-Fragmentainers.svg)

  Illustration of breaking in varying-size fragmentainers.

Since document order of elements doesn’t change during fragmentation, fragments are processed following the same rules that apply to continuous media. In particular, the order of floats is preserved across all fragments and follows the same rules as defined in CSS 2.1 9.5.

<strong>Note:</strong>

Below are listed (informatively) some implications of these rules:

- Boxes (including tables) fulfilling layout constraints at their <a id="ref-for-stretch-fit-size"></a>[stretch-fit](https://drafts.csswg.org/css-sizing-3/#stretch-fit-size) or percentage-based size may change <a id="ref-for-inline-size①"></a>[inline size](https://drafts.csswg.org/css-sizing-3/#inline-size) across pages.
- Boxes (including tables) fulfilling layout constraints at their <a id="ref-for-min-content"></a>[min-content](https://drafts.csswg.org/css-sizing-3/#min-content), <a id="ref-for-max-content"></a>[max-content](https://drafts.csswg.org/css-sizing-3/#max-content), or absolute-length size will maintain their <a id="ref-for-inline-size②"></a>[inline size](https://drafts.csswg.org/css-sizing-3/#inline-size) across pages.
- A block-level continuation fragment may be placed below the top of the page if, e.g. it establishes a block formatting context and is placed beside a float and both it and the float continue onto a narrower page that is too narrow to hold both of them side-by-side.
- An element adjacent to a preceding float on one page may wind up above the float’s continuation on the next page if, e.g. that float is pushed down because it no longer fits side-by-side with an earlier float that also continues onto this narrower page.
- A left float may appear on a page <em>before</em> the remaining fragments of a preceding right float if that right float does not fit on the earlier page. However another right float will be forced down until the preceding right float’s remaining fragment can be placed.

<a id="example-fbdd0b64"></a>

<strong>Example:</strong>

[](#example-fbdd0b64)

Here is an example that shows the use of percentage-based progress: Suppose we have an absolutely-positioned element that is positioned <a id="ref-for-propdef-top"></a>[top: calc(150% + 30px)](https://drafts.csswg.org/css-position-3/#propdef-top) and has <a id="ref-for-propdef-height"></a>[height: calc(100% - 10px)](https://drafts.csswg.org/css-sizing-3/#propdef-height). If it is placed into a paginated context with a first page height of 400px, a second page of 200px, and a third page of 600px, its layout progresses as follows:

- First, the top position is resolved against the height of the first page. This results in 630px. Since the first page has a height of only 400px, layout moves to the second page, recording progress of 400/630 = 63.49% with 36.51% left to go.
- Now on the second page, the top position is again resolved, this time against the height of the second page. This results in 330px. The remaining 36.51% of progress thus resolves to 120.5px, placing the top edge of the element 120.5px down the second page.
- Now the height is resolved against the second page; it resolves to 190px. Since there are only 79.5px left on the page, layout moves to the third page, recording progress of 79.5/190 = 41.84%, with 58.16% left to go.
- On the third page, the height resolves to 590px. The remaining 58.16% of progress thus resolves to 343.1px, which fits on this page and completes the element.

### <a id="break-margins"></a>5.2.  Adjoining Margins at Breaks[](#break-margins)

When an unforced break occurs before or after a block-level box, any margins adjoining the break are truncated to zero. When a forced break occurs there, adjoining margins before the break are truncated, but margins after the break are preserved. <a id="ref-for-valdef-box-decoration-break-clone②"></a>[Cloned margins](#valdef-box-decoration-break-clone) are always truncated to zero on block-level boxes.

Note: CSS Fragmentation Level 4 will introduce control over margin truncation at breaks.

### <a id="box-splitting"></a>5.3.  Splitting Boxes[](#box-splitting)

When a box breaks, its content box extends to fill any <a id="ref-for-remaining-fragmentainer-extent"></a>[remaining fragmentainer extent](#remaining-fragmentainer-extent) (leaving room for any margins/borders/padding applied by <a id="ref-for-propdef-box-decoration-break②"></a>[box-decoration-break: clone](#propdef-box-decoration-break)) before the content resumes on the next <a id="ref-for-fragmentainer②②"></a>[fragmentainer](#fragmentainer). (A <a id="ref-for-fragmentation-break②"></a>[fragmentation break](#fragmentation-break) that pushes content to the next <a id="ref-for-fragmentainer②③"></a>fragmentainer effectively increases the <a id="ref-for-block-size②"></a>[block size](https://drafts.csswg.org/css-sizing-3/#block-size) of a box’s contents.)

The extra

<strong>Note:</strong>

<a id="ref-for-block-size③"></a>[block size](https://drafts.csswg.org/css-sizing-3/#block-size) contributed by fragmenting the box (i.e. the distance from the break point to the edge of the <a id="ref-for-fragmentainer②④"></a>[fragmentainer](#fragmentainer)) contributes progress towards any specified limits on the box’s <a id="ref-for-block-size④"></a>block size.

![Illustration: Filling remaining fragmentainer extent](https://drafts.csswg.org/css-break-3/images/Remaining-Fragmentainer-Extent.svg)

Illustration of filling the <a id="ref-for-remaining-fragmentainer-extent①"></a>[remaining fragmentainer extent](#remaining-fragmentainer-extent).

### <a id="break-decoration"></a>5.4.  Fragmented Borders and Backgrounds: the <a id="ref-for-propdef-box-decoration-break③"></a>[box-decoration-break](#propdef-box-decoration-break) property[](#break-decoration)

| Field                                                                                    | Definition                                                                                       |
|------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-box-decoration-break"></a><strong>box-decoration-break</strong>                   |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | slice <a id="ref-for-comb-one①⑤"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) clone |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | slice                                                                                            |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                              |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                               |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | n/a                                                                                              |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | specified keyword                                                                                |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                      |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | discrete                                                                                         |

When a break (page/column/region/line) splits a box, the <a id="ref-for-propdef-box-decoration-break④"></a>[box-decoration-break](#propdef-box-decoration-break) property controls

- whether the box’s margins, borders, padding, and other decorations wrap the broken edges of the box fragments
- how the [](https://www.w3.org/TR/css3-background/#background-positioning-area)<a id="ref-for-background-positioning-area"></a>[background positioning area](https://drafts.csswg.org/css-backgrounds-3/#background-positioning-area) [\[CSS3BG\]](https://drafts.csswg.org/css-break-3/#biblio-css3bg) (and <a id="ref-for-mask-positioning-area"></a>[mask positioning area](https://drafts.csswg.org/css-masking-1/#mask-positioning-area) [\[CSS-MASKING-1\]](https://drafts.csswg.org/css-break-3/#biblio-css-masking-1), shape <a id="ref-for-basic-shape-reference-box"></a>[reference box](https://drafts.csswg.org/css-shapes-1/#basic-shape-reference-box) [\[CSS-SHAPES-1\]](https://drafts.csswg.org/css-break-3/#biblio-css-shapes-1), etc.) is derived from or duplicated across the box fragments and how the element’s background is drawn within them.

Values have the following meanings:

<strong><a id="valdef-box-decoration-break-clone"></a><strong>clone</strong></strong>

Each box fragment is independently wrapped with the border, padding, and margin. The <a id="ref-for-propdef-border-radius"></a>[border-radius](https://drafts.csswg.org/css-borders-4/#propdef-border-radius) and <a id="ref-for-propdef-border-image"></a>[border-image](https://drafts.csswg.org/css-borders-4/#propdef-border-image) and <a id="ref-for-propdef-box-shadow"></a>[box-shadow](https://drafts.csswg.org/css-borders-4/#propdef-box-shadow), if any, are applied to each fragment independently. The background is drawn independently in each fragment of the element. A no-repeat background image will thus be rendered once in each fragment of the element.

Note: Cloned margins are [truncated](#break-margins) on block-level boxes.

<strong><a id="valdef-box-decoration-break-slice"></a><strong>slice</strong></strong>

The effect is as though the element were rendered with no breaks present, and then sliced by the breaks afterward: no border and no padding are inserted at a break; no box-shadow is drawn at a broken edge; and backgrounds, <a id="ref-for-propdef-border-radius①"></a>[border-radius](https://drafts.csswg.org/css-borders-4/#propdef-border-radius), and the <a id="ref-for-propdef-border-image①"></a>[border-image](https://drafts.csswg.org/css-borders-4/#propdef-border-image) are applied to the geometry of the whole box as if it were unbroken.

<img src="https://drafts.csswg.org/css-break-3/images/box-break.png" alt="Illustration:&#10;        (1) a single box cut in two in between two lines of text by a page break and&#10;        (2) two boxes, one before and one after the page break,&#10;        both with a border all around and their own background image">

Two possibilities for <a id="ref-for-propdef-box-decoration-break⑤"></a>[box-decoration-break](#propdef-box-decoration-break): on the left, the value <a id="ref-for-valdef-box-decoration-break-slice"></a>[slice](#valdef-box-decoration-break-slice), on the right the value <a id="ref-for-valdef-box-decoration-break-clone③"></a>[clone](#valdef-box-decoration-break-clone).

UAs should also apply <a id="ref-for-propdef-box-decoration-break⑥"></a>[box-decoration-break](#propdef-box-decoration-break) to control rendering at bidi-imposed breaks—​i.e. when bidi reordering causes an inline to split into non-contiguous fragments—​and/or at display-type–imposed breaks—​i.e. when a higher-level <a id="ref-for-display-type②"></a>[display type](https://drafts.csswg.org/css-display-4/#display-type) (such as a <a id="ref-for-block-level-box"></a>[block-level box](https://drafts.csswg.org/css-display-4/#block-level-box) / [column spanner](https://www.w3.org/TR/css-multicol-1/#spanning-columns)) splits an incompatible ancestor (such as an <a id="ref-for-inline-box"></a>[inline box](https://drafts.csswg.org/css-display-4/#inline-box) / <a id="ref-for-block-container①"></a>[block container](https://drafts.csswg.org/css-display-4/#block-container)). Otherwise such breaks must be handled as <a id="ref-for-valdef-box-decoration-break-slice①"></a>[slice](#valdef-box-decoration-break-slice). See [Applying the Bidirectional Reordering Algorithm](https://drafts.csswg.org/css-writing-modes-3/#bidi-algo) in [CSS Writing Modes](https://drafts.csswg.org/css-writing-modes-3/#text-direction), [CSS2§9.2 Block-level elements and block boxes](https://www.w3.org/TR/CSS2/visuren.html#box-gen), and [CSS Multi-column Layout §6 Spanning Columns](https://www.w3.org/TR/css-multicol-1/#spanning-columns).

For inline elements, which side of a fragment is considered the broken edge is determined by the parent element’s inline progression direction. For example, if an inline element whose parent has

<strong>Note:</strong>

<a id="ref-for-propdef-direction"></a>[direction: rtl](https://drafts.csswg.org/css-writing-modes-3/#propdef-direction) breaks across two lines, the <em>left</em> edge of the fragment on the first line will be the broken edge. (Note in particular that neither the element’s own <a id="ref-for-propdef-direction①"></a>direction nor its containing block’s <a id="ref-for-propdef-direction②"></a>direction is used.) See [\[CSS3-WRITING-MODES\]](https://drafts.csswg.org/css-break-3/#biblio-css3-writing-modes).

#### <a id="joining-boxes"></a>5.4.1.  Joining Boxes for <a id="ref-for-valdef-box-decoration-break-slice②"></a>[slice](#valdef-box-decoration-break-slice)[](#joining-boxes)

For <a id="ref-for-propdef-box-decoration-break⑦"></a>[box-decoration-break: slice](#propdef-box-decoration-break), backgrounds (and <a id="ref-for-propdef-border-image②"></a>[border-image](https://drafts.csswg.org/css-borders-4/#propdef-border-image)) are drawn as if applied to a composite box consisting of all of the box’s fragments reassembled in visual order. This theoretical assembly occurs after the element has been laid out (including any justification, bidi reordering, page breaks, etc.). To assemble the composite box...

<strong>For boxes broken across lines</strong>

First, fragments on the same line are connected in visual order. Then, fragments on subsequent lines are ordered according to the element’s <a id="ref-for-inline-base-direction"></a>[inline base direction](https://drafts.csswg.org/css-writing-modes-4/#inline-base-direction) and aligned on the element’s dominant baseline. For example, in a left-to-right containing block (<a id="ref-for-propdef-direction③"></a>[direction](https://drafts.csswg.org/css-writing-modes-3/#propdef-direction) is <a id="ref-for-valdef-direction-ltr"></a>[ltr](https://drafts.csswg.org/css-writing-modes-4/#valdef-direction-ltr)), the first fragment is the leftmost fragment on the first line and fragments from subsequent lines are put to the right of it. In a right-to-left containing block, the first fragment is the rightmost on the first line and subsequent fragments are put to the left of it.

<strong>For boxes broken across columns</strong>

Fragments are connected as if the column boxes were glued together in the <a id="ref-for-block-flow-direction"></a>[block flow direction](https://drafts.csswg.org/css-writing-modes-4/#block-flow-direction) of the multi-column container.

<strong>For boxes broken across pages</strong>

Fragments are connected as if page content areas were glued together in the <a id="ref-for-block-flow-direction①"></a>[block flow direction](https://drafts.csswg.org/css-writing-modes-4/#block-flow-direction) of the root element.

<strong>For boxes broken across regions</strong>

Fragments are connected as if region content areas were glued together in the <a id="ref-for-block-flow-direction②"></a>[block flow direction](https://drafts.csswg.org/css-writing-modes-4/#block-flow-direction) of the <a id="ref-for-principal-writing-mode"></a>[principal writing mode](https://drafts.csswg.org/css-writing-modes-4/#principal-writing-mode) of the <a id="ref-for-region-chain"></a>[region chain](https://drafts.csswg.org/css-regions-1/#region-chain).

If the box fragments have different widths (heights, if the fragments are joined horizontally), then each piece draws its portion of the background assuming that the whole element has the same width (height) as this piece. However, if the used height (width) of an image is derived from the width (height) of the box, then it is calculated using the widest fragment’s width and maintained as a fixed size. This ensures that right-aligned images stay aligned to the right edge, left-aligned images stay aligned to the left edge, centered images stay centered, and stretched images cover the background area as intended while preserving continuity across fragments.

### <a id="transforms"></a>5.5.  Transforms, Positioning, and Pagination[](#transforms)

Fragmentation interacts with layout, and thus occurs <em>before</em> relative positioning [\[CSS2\]](https://drafts.csswg.org/css-break-3/#biblio-css2), transforms [\[CSS3-TRANSFORMS\]](https://drafts.csswg.org/css-break-3/#biblio-css3-transforms), and any other graphical effects. Such effects are applied per fragment: for example, rotation applied to a fragmented box will calculate a rotation origin for each fragment and independently rotate that fragment around its origin. (The origin of an overflow-only fragment is determined as if that content were overflowing an empty box with zero margins/borders/padding at the start of the fragmentainer.) However, in order to reduce dataloss when printing, the separation and transfer of page boxes <em>should</em> occur last; thus a transformed fragment that spans pages <em>should</em> be sliced at the page breaks and print in its entirety rather than being clipped by its originating page.

![Illustration: Transformed overflow fragmentation](https://drafts.csswg.org/css-break-3/images/fragmented-transforms.png)

A fixed-height box spanning 2.5 pages with overflow content spanning to a total of 4 pages. The transform origin of each fragment is the center of its border box; the fragment without a border box assumes a zero-height box at the start of the overflow.

Absolute positioning affects layout and thus interacts with fragmentation. Both the coordinate system and absolutely-positioned boxes belonging to a containing block will fragment across fragmentainers in the same fragmentation flow as the containing block.

UAs are not required to correctly position boxes that span a <a id="ref-for-fragmentation-break③"></a>[fragmentation break](#fragmentation-break) and whose <a id="ref-for-block-start③"></a>[block-start](https://drafts.csswg.org/css-writing-modes-4/#block-start) edge position depends on where the box’s content fragments.

UAs with memory constraints that prevent them from manipulating an entire document in memory are not required to correctly position absolutely-positioned elements that end up on a previously-rendered page.
