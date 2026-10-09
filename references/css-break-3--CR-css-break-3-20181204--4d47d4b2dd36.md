Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Fragmentation Module Level 3](https://www.w3.org/TR/2018/CR-css-break-3-20181204/).

Original copyright notice: Copyright © 2018 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Fragmentation Module Level 3

Source snapshot: https://www.w3.org/TR/2018/CR-css-break-3-20181204/

Snapshot SHA-256: 4d47d4b2dd36a28e2b0275833b9734b1d5a0b18299a3f278e238bed8e2ecd509

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 6 source tables are presented as readable Markdown tables or explicit labeled layouts: 5 ordinary table conversions, 1 already-readable table. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# CSS Fragmentation Module Level 3

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2018 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](http://www.csail.mit.edu/), [ERCIM](http://www.ercim.eu/), [Keio](http://www.keio.ac.jp/), [Beihang](http://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This module describes the fragmentation model that partitions a flow into pages, columns, or regions. It builds on the Page model module and introduces and defines the fragmentation model. It adds functionality for pagination, breaking variable fragment size and orientation, widows and orphans.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, in speech, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
		Other documents may supersede this document.
		A list of current W3C publications and the latest revision of this technical report
		can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was produced by the [CSS Working Group](https://www.w3.org/Style/CSS/members) as a Candidate Recommendation. This document is intended to become a W3C Recommendation. This document will remain a Candidate Recommendation at least until 4 March 2019 in order to ensure the opportunity for wide review.

[GitHub Issues](https://github.com/w3c/csswg-drafts/issues) are preferred for discussion of this specification. When filing an issue, please put the text “css-break” in the title, preferably like this: “\[css-break\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/), and there is also a [historical archive](https://lists.w3.org/Archives/Public/www-style/).

A [draft implementation report](http://test.csswg.org/harness/results/css-break-3_dev/grouped/) is available.

Publication as a Candidate Recommendation does not imply endorsement by the W3C Membership. This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy/#sec-Disclosure).

<a id="w3c_process_revision"></a>

This document is governed by the [1 February 2018 W3C Process Document](https://www.w3.org/2018/Process-20180201/).

For changes since the last draft, see the [Changes](#changes) section.

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-valdef-break-before-avoid-region"></a>

  <a id="ref-for-valdef-break-before-region"></a>

  the [region](#valdef-break-before-region) and [avoid-region](#valdef-break-before-avoid-region) values of break-\*

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1.  Introduction

<em>This section is not normative.</em>

In paged media (e.g., paper, transparencies, photo album pages, pages displayed on computer screens as printed output simulations), as opposed to [continuous media](https://www.w3.org/TR/CSS2/media.html#continuous-media-group), the content of the document is split into one or more discrete display surfaces. In order to avoid awkward breaks (such as halfway through a line of text), the layout engine must be able to shift around content that would fall across the page break. This process is called <a id="pagination"></a>pagination.

<a id="ref-for-fragmentation"></a>

<a id="ref-for-fragmentation-container"></a>

<a id="ref-for-fragmentainer"></a>

In CSS, in addition to paged media, certain layout features such as [regions](https://www.w3.org/TR/css3-regions/) [\[CSS3-REGIONS\]](#biblio-css3-regions) and [multi-column layout](https://www.w3.org/TR/css3-multicol/) [\[CSS3COL\]](#biblio-css3col) create a similarly fragmented environment. The generic term for breaking content across containers is [fragmentation](#fragmentation). This module explains how content breaks across [fragmentation containers](#fragmentation-container) ([fragmentainers](#fragmentainer)) such as pages and columns and how such breaks can be [controlled by the author](#breaking-controls).

### <a id="placement"></a>1.1.  Module Interactions

This module replaces and extends the pagination controls defined in [\[CSS21\]](#biblio-css21) [section 13.3](https://www.w3.org/TR/CSS21/page.html#page-breaks) and in [\[CSS3PAGE\]](#biblio-css3page).

### <a id="values"></a>1.2.  Values

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS21/about.html#property-defs) from [\[CSS21\]](#biblio-css21). Value types not defined in this specification are defined in CSS Values &#x26; Units [\[CSS-VALUES-3\]](#biblio-css-values-3). Other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) keywords as their property value. For readability they have not been repeated explicitly.

## <a id="fragmentation-model"></a>2.  Fragmentation Model and Terminology

<a id="fragmentation-container"></a>fragmentation container (<a id="fragmentainer"></a>fragmentainer)  
<a id="ref-for-fragmentation-context"></a>

<a id="ref-for-fragmented-flow"></a>

A box—such as a page box, column box, or region—that contains a portion (or all) of a [fragmented flow](#fragmented-flow). Fragmentainers can be pre-defined, or generated as needed. When breakable content would overflow a fragmentainer in the block dimension, it breaks into the next container in its [fragmentation context](#fragmentation-context) instead.

<a id="fragmentation-context"></a>fragmentation context  
<a id="ref-for-fragmentation-root①"></a>

<a id="ref-for-fragmentation-root"></a>

<a id="ref-for-fragmentainer②"></a>

<a id="ref-for-fragmentainer①"></a>

An ordered series of [fragmentainers](#fragmentainer), such as created by a [multi-column container](https://www.w3.org/TR/css3-multicol/), a chain of [CSS regions](https://www.w3.org/TR/css3-regions), or a [paged media display](https://www.w3.org/TR/css3-page/). A given fragmentation context can only have one block flow direction across all its [fragmentainers](#fragmentainer). (Descendants of the [fragmentation root](#fragmentation-root) may have other block flow directions, but fragmentation proceeds according to the block flow direction applied to the [fragmentation root](#fragmentation-root).)

<a id="fragmented-flow"></a>fragmented flow  
<a id="ref-for-fragmented-flow①"></a>

<a id="ref-for-fragmentation-context①"></a>

Content that is being laid out in a [fragmentation context](#fragmentation-context). The [fragmented flow](#fragmented-flow) consists of the content of a (possibly anonymous) box called the <a id="fragmentation-root"></a>fragmentation root.

<a id="fragmentation-direction"></a>fragmentation direction  
<a id="ref-for-fragmentation-context②"></a>

The block flow direction of the [fragmentation context](#fragmentation-context), i.e. the direction in which content is fragmented. (In this level of CSS, content only fragments in one dimension.)

<a id="fragmentation"></a>fragmentation  
<a id="ref-for-fragmentation-context③"></a>

<a id="ref-for-fragmentainer③"></a>

The process of splitting a content flow across the [fragmentainers](#fragmentainer) that form a [fragmentation context](#fragmentation-context).

<a id="box-fragment"></a>box fragment or <a id="fragment"></a>fragment  
<a id="ref-for-propdef-box-decoration-break"></a>

<a id="ref-for-margin-area"></a>

<a id="ref-for-border-area"></a>

<a id="ref-for-padding-area"></a>

<a id="ref-for-fragmentainer④"></a>

The portion of a box that belongs to exactly one [fragmentainer](#fragmentainer). A box in continuous flow always consists of only one fragment. A box in a fragmented flow consists of one or more fragments. Each fragment has its own share of the box’s border, padding, and margin, and therefore has its own [padding area](https://www.w3.org/TR/css-box-3/#padding-area), [border area](https://www.w3.org/TR/css-box-3/#border-area), and [margin area](https://www.w3.org/TR/css-box-3/#margin-area). (See [box-decoration-break](#propdef-box-decoration-break), which controls how these are affected by fragmentation.)

<a id="remaining-fragmentainer-extent"></a>remaining fragmentainer extent  
<a id="ref-for-fragmentainer⑦"></a>

<a id="ref-for-fragmentainer⑥"></a>

<a id="ref-for-fragmentainer⑤"></a>

<a id="ref-for-block-axis"></a>

The remaining [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) space in the [fragmentainer](#fragmentainer) available to a given element, i.e. between the end of preceding content in [fragmentainer](#fragmentainer) and the edge of the [fragmentainer](#fragmentainer).

<a id="ref-for-fragmentainer⑧"></a>

<a id="ref-for-fragmentainer⑨"></a>

<a id="ref-for-fragmentainer①⓪"></a>

Each <a id="fragmentation-break"></a>fragmentation break (hereafter, <a id="break"></a>break) ends layout of the fragmented box in the current [fragmentainer](#fragmentainer) and causes the remaining content to be laid out in the next [fragmentainer](#fragmentainer), in some cases causing a new [fragmentainer](#fragmentainer) to be generated to hold the deferred content.

> <strong data-conversion-semantic="note">Note</strong>
>
> Breaking inline content into lines is another form of fragmentation, and similarly creates box fragments when it breaks [inline boxes](https://www.w3.org/TR/CSS21/visuren.html#inline-boxes) across [line boxes](https://www.w3.org/TR/CSS21/visuren.html#line-box). However, inline breaking is not covered here; see [\[CSS21\]](#biblio-css21)/[\[CSS3TEXT\]](#biblio-css3text).

<a id="ref-for-fragment"></a>

<a id="ref-for-display-type"></a>

<a id="ref-for-box-fragment"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> A box can be broken into multiple [fragments](#fragment) also due to bidi reordering of text (see [Applying the Bidirectional Reorderign Algorithm](https://www.w3.org/TR/css-writing-modes-3/#bidi-algo) in [CSS Writing Modes](https://www.w3.org/TR/css-writing-modes-3/#text-direction)) or higher-level [display type](https://www.w3.org/TR/css-display-3/#display-type) box splitting, e.g. [block-in-inline splitting](https://www.w3.org/TR/CSS2/visuren.html#img-anon-block) (see [CSS2§9.2](https://www.w3.org/TR/CSS2/visuren.html#box-gen)) or [column-spanner-in-block](https://www.w3.org/TR/css-multicol-1/#spanning-columns) splitting (see [CSS Multi-column Layout](https://www.w3.org/TR/css-multicol-1/#spanning-columns)). The division into [box fragments](#box-fragment) in these cases does not depend on layout (sizing/positioning of content).

### <a id="parallel-flows"></a>2.1.  Parallel Fragmentation Flows

<a id="ref-for-formatting-context"></a>

<a id="ref-for-formatting-context①"></a>

<a id="ref-for-unforced-break"></a>

<a id="ref-for-formatting-context②"></a>

<a id="ref-for-forced-break"></a>

When multiple [formatting contexts](https://www.w3.org/TR/css-display-3/#formatting-context) are laid out parallel to each other, fragmentation is performed independently in each [formatting context](https://www.w3.org/TR/css-display-3/#formatting-context). For example, if an element is floated, then a forced break inside the float will not affect the content outside the float (except insofar as it may increase the height of the float). UAs <em>may</em> (but are not required to) adjust the placement of [unforced breaks](#unforced-break) in parallel [formatting contexts](https://www.w3.org/TR/css-display-3/#formatting-context) to visually balance such side-by-side content, but <em>must not</em> do so to match a [forced break](#forced-break).

The following are examples of parallel flows whose contents will fragment independently:

- The contents of a float vs. the content wrapping outside the float.
- The contents of a float vs. the contents of an adjacent float.
- The contents of each table cell in a single table row.
- The contents of each grid item in a single grid row.
- The contents of each flex item in a flex layout row.
- The contents of absolutely-positioned elements that cover the same range of their containing block’s fragmentation context.

<a id="ref-for-fragmentation-root②"></a>

<a id="ref-for-fragmented-flow②"></a>

<a id="ref-for-fragmentainer①①"></a>

Content overflowing the content edge of a fixed-size box is considered parallel to the content after the fixed-size box and follows the normal fragmentation rules. Although overflowing content doesn’t affect the size of the [fragmentation root](#fragmentation-root) box, it does increase the length of the [fragmented flow](#fragmented-flow), spilling into or generating additional [fragmentainers](#fragmentainer) as necessary.

### <a id="nested-flows"></a>2.2.  Nested Fragmentation Flows

<a id="ref-for-fragmentainer①②"></a>

<a id="ref-for-fragmentainer①③"></a>

<a id="ref-for-fragmentainer①④"></a>

<a id="ref-for-fragmentainer①⑤"></a>

<a id="ref-for-fragmentation-context④"></a>

<a id="ref-for-fragmentation-context⑤"></a>

Breaking a [fragmentainer](#fragmentainer) <var>F</var> effectively splits the [fragmentainer](#fragmentainer) into two [fragmentainers](#fragmentainer) (<var>F<sub>1</sub></var> and <var>F<sub>2</sub></var>). The only difference is that, with regards to the content of [fragmentainer](#fragmentainer) <var>F</var>, the type of break between the two pieces <var>F<sub>1</sub></var> and <var>F<sub>2</sub></var> is the [type of break](#break-types) created by the [fragmentation context](#fragmentation-context) that split <var>F</var>, not the type of break normally created by <var>F</var>’s own [fragmentation context](#fragmentation-context).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-336a11b1"></a> For example, if a region box is broken at a page boundary, then the content of the region will be affected by a page break at that point (but not by a region break).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that when a multi-column container breaks across pages, it generates a new row of columns on the next page for the rest of its content, so that a page break within a multi-column container is always both a page break and a column break.

## <a id="breaking-controls"></a>3.  Controlling Breaks

<a id="ref-for-fragmented-flow③"></a>

<a id="ref-for-propdef-break-inside"></a>

<a id="ref-for-propdef-break-after"></a>

<a id="ref-for-propdef-break-before"></a>

<a id="ref-for-propdef-break-inside①"></a>

<a id="ref-for-propdef-widows"></a>

<a id="ref-for-propdef-orphans"></a>

The following sections explain how breaks are controlled in a [fragmented flow](#fragmented-flow). A page/column/region break opportunity between two boxes is under the influence of the containing block’s [break-inside](#propdef-break-inside) property, the [break-after](#propdef-break-after) property of the preceding element, and the [break-before](#propdef-break-before) property of the following element. A page/column/region break opportunity between line boxes is under the influence of the containing block’s [break-inside](#propdef-break-inside), [widows](#propdef-widows), and [orphans](#propdef-orphans) properties. A fragmentation break can be allowed, forced, or discouraged depending on the values of these properties. A forced break overrides any break restrictions acting at that break point. In the case of forced page breaks, the author can also specify on which page ([left or right](https://www.w3.org/TR/css3-page/#left-right-first)) the subsequent content should resume.

See the section on [rules for breaking](#breaking-rules) for the exact rules on how these properties affect fragmentation.

<a id="ref-for-propdef-break-before①"></a>

<a id="ref-for-propdef-break-after①"></a>

### <a id="break-between"></a>3.1.  Breaks Between Boxes: the [break-before](#propdef-break-before) and [break-after](#propdef-break-after) properties

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-break-before"></a>break-before, <a id="propdef-break-after"></a>break-after                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://drafts.csswg.org/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⓪"></a><a id="ref-for-comb-one⑨"></a><a id="ref-for-comb-one⑧"></a><a id="ref-for-comb-one⑦"></a><a id="ref-for-comb-one⑥"></a><a id="ref-for-comb-one⑤"></a><a id="ref-for-comb-one④"></a><a id="ref-for-comb-one③"></a><a id="ref-for-comb-one②"></a><a id="ref-for-comb-one①"></a><a id="ref-for-comb-one"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) avoid [\|](https://www.w3.org/TR/css-values-4/#comb-one) avoid-page [\|](https://www.w3.org/TR/css-values-4/#comb-one) page [\|](https://www.w3.org/TR/css-values-4/#comb-one) left [\|](https://www.w3.org/TR/css-values-4/#comb-one) right [\|](https://www.w3.org/TR/css-values-4/#comb-one) recto [\|](https://www.w3.org/TR/css-values-4/#comb-one) verso [\|](https://www.w3.org/TR/css-values-4/#comb-one) avoid-column [\|](https://www.w3.org/TR/css-values-4/#comb-one) column [\|](https://www.w3.org/TR/css-values-4/#comb-one) avoid-region [\|](https://www.w3.org/TR/css-values-4/#comb-one) region |
| <strong><a href="https://drafts.csswg.org/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong>Applies to:&#xA;      </strong> | block-level boxes, grid items, flex items, table row groups, table rows (but see prose)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://drafts.csswg.org/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://drafts.csswg.org/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://drafts.csswg.org/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://drafts.csswg.org/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |

<a id="ref-for-valdef-break-before-left"></a>

<a id="ref-for-valdef-break-before-right"></a>

<a id="ref-for-valdef-break-before-recto"></a>

<a id="ref-for-valdef-break-before-verso"></a>

<a id="ref-for-valdef-break-before-page"></a>

<a id="ref-for-valdef-break-before-column"></a>

<a id="ref-for-valdef-break-before-region①"></a>

<a id="ref-for-valdef-break-before-avoid"></a>

<a id="ref-for-valdef-break-before-avoid-page"></a>

<a id="ref-for-valdef-break-before-avoid-column"></a>

<a id="ref-for-valdef-break-before-avoid-region①"></a>

These properties specify page/column/region break behavior before/after the generated box. The <a id="forced-break-values"></a>forced break values [left](#valdef-break-before-left), [right](#valdef-break-before-right), [recto](#valdef-break-before-recto), [verso](#valdef-break-before-verso), [page](#valdef-break-before-page), [column](#valdef-break-before-column) and [region](#valdef-break-before-region) create a [forced break](#forced-breaks) in the flow while the <a id="avoid-break-values"></a>avoid break values [avoid](#valdef-break-before-avoid), [avoid-page](#valdef-break-before-avoid-page), [avoid-column](#valdef-break-before-avoid-column) and [avoid-region](#valdef-break-before-avoid-region) indicate that content should be kept together.

<a id="ref-for-propdef-break-before②"></a>

<a id="ref-for-propdef-break-after②"></a>

<a id="ref-for-fragmentation-root③"></a>

Values for [break-before](#propdef-break-before) and [break-after](#propdef-break-after) are defined in the sub-sections below. User Agents must apply these properties to boxes in the normal flow of the [fragmentation root](#fragmentation-root). User agents should also apply these properties to floated boxes whose containing block is in the normal flow of the root fragmented element. User agents may also apply these properties to other boxes. User agents must not apply these properties to absolutely-positioned boxes.

#### <a id="generic-break-values"></a> Generic Break Values

These values have an effect regardless of the type of fragmented context containing the flow.

<a id="valdef-break-before-auto"></a>auto  
Neither force nor forbid a break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

<a id="valdef-break-before-avoid"></a>avoid  
Avoid a break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

#### <a id="page-break-values"></a> Page Break Values

These values only have an effect in paginated contexts; if the flow is not paginated, they have no effect.

<a id="valdef-break-before-avoid-page"></a>avoid-page  
Avoid a page break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

<a id="valdef-break-before-page"></a>page  
Always force a page break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

<a id="valdef-break-before-left"></a>left  
Force one or two page breaks before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes) so that the next page is formatted as a left page.

<a id="valdef-break-before-right"></a>right  
Force one or two page breaks before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes) so that the next page is formatted as a right page.

<a id="valdef-break-before-recto"></a>recto  
<a id="ref-for-page-progression"></a>

Force one or two page breaks before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes) so that the next page is formatted as either a left page or a right page, whichever is second (according to the [page progression](https://www.w3.org/TR/css3-page/#page-progression)) in a page spread.

<a id="valdef-break-before-verso"></a>verso  
<a id="ref-for-page-progression①"></a>

Force one or two page breaks before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes) so that the next page is formatted as either a left page or a right page, whichever is first (according to the [page progression](https://www.w3.org/TR/css3-page/#page-progression)) in a page spread.

#### <a id="column-break-values"></a> Column Break Values

These values only have an effect in multi-column contexts; if the flow is not within a multi-column context, they have no effect.

<a id="valdef-break-before-avoid-column"></a>avoid-column  
Avoid a column break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

<a id="valdef-break-before-column"></a>column  
Always force a column break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

#### <a id="region-break-values"></a> Region Break Values

These values only have an effect in multi-region contexts; if the flow is not linked across multiple regions, these values have no effect.

<a id="valdef-break-before-avoid-region"></a>avoid-region  
Avoid a region break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

<a id="valdef-break-before-region"></a>region  
Always force a region break before/after the [principal box](https://www.w3.org/TR/CSS21/visuren.html#block-boxes).

#### <a id="break-propagation"></a>3.1.1.  Child→Parent Break Propagation

Since breaks are only allowed between siblings, not between a box and its container (see [Possible Break Points](#possible-breaks)), break values applied to children at the start/end of a parent are <a id="propagate"></a>propagated to the parent, where they can take effect.

<a id="ref-for-propdef-break-before③"></a>

<a id="ref-for-in-flow"></a>

<a id="ref-for-propagate"></a>

<a id="ref-for-propdef-break-after③"></a>

<a id="ref-for-in-flow①"></a>

<a id="ref-for-propagate①"></a>

Specifically—except in layout modes which define more specific rules to account for reordering and parallel layout (e.g. in [flex layout](https://www.w3.org/TR/css-flexbox-1/#pagination) [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1) or [grid layout](https://www.w3.org/TR/css-grid-1/#pagination) [\[CSS-GRID-1\]](#biblio-css-grid-1))—a [break-before](#propdef-break-before) value on a first [in-flow](https://www.w3.org/TR/css-display-3/#in-flow) child box is [propagated](#propagate) to its container. Likewise a [break-after](#propdef-break-after) value on a last [in-flow](https://www.w3.org/TR/css-display-3/#in-flow) child box is [propagated](#propagate) to its container. (Conflicting values [combine](#forced-breaks) as defined below.) This propagation stops before it breaks through the nearest matching fragmentation context.

<a id="ref-for-propagate②"></a>

<a id="ref-for-computed-value"></a>

Break [propagation](#propagate) does not affect [computed values](https://www.w3.org/TR/css-cascade-4/#computed-value); it is part of interpeting the elements’ computed values for layout.

<a id="ref-for-propdef-break-inside②"></a>

### <a id="break-within"></a>3.2.  Breaks Within Boxes: the [break-inside](#propdef-break-inside) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                              |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-break-inside"></a>break-inside                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://drafts.csswg.org/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①④"></a><a id="ref-for-comb-one①③"></a><a id="ref-for-comb-one①②"></a><a id="ref-for-comb-one①①"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) avoid [\|](https://www.w3.org/TR/css-values-4/#comb-one) avoid-page [\|](https://www.w3.org/TR/css-values-4/#comb-one) avoid-column [\|](https://www.w3.org/TR/css-values-4/#comb-one) avoid-region |
| <strong><a href="https://drafts.csswg.org/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                                    |
| <strong>Applies to:&#xA;      </strong> | all elements except inline-level boxes, internal ruby boxes, table column boxes, table column group boxes, absolutely-positioned boxes                                                                                                                                                                                                  |
| <strong><a href="https://drafts.csswg.org/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://drafts.csswg.org/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                     |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://drafts.csswg.org/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                                                                                                                                       |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://drafts.csswg.org/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                |

This property specifies page/column/region break behavior within the element’s principal box. Values have the following meanings:

<a id="valdef-break-inside-auto"></a>auto  
Impose no additional breaking constraints within the box.

<a id="valdef-break-inside-avoid"></a>avoid  
Avoid breaks within the box.

<a id="valdef-break-inside-avoid-page"></a>avoid-page  
Avoid a page break within the box.

<a id="valdef-break-inside-avoid-column"></a>avoid-column  
Avoid a column break within the box.

<a id="valdef-break-inside-avoid-region"></a>avoid-region  
Avoid a region break within the box.

<a id="ref-for-propdef-orphans①"></a>

<a id="ref-for-propdef-widows①"></a>

### <a id="widows-orphans"></a>3.3.  Breaks Between Lines: [orphans](#propdef-orphans), [widows](#propdef-widows)

| Field               | Definition                                                                                                                                                                                                                   |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-orphans"></a>orphans, <a id="propdef-widows"></a>widows                                                                                                                                                                        |
| <strong><a href="https://drafts.csswg.org/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-integer-value"></a>[\<integer\>](https://www.w3.org/TR/css3-values/#integer-value)                                                                                                                                           |
| <strong><a href="https://drafts.csswg.org/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 2                                                                                                                                                                                                                            |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-inline-formatting-context"></a><a id="ref-for-block-container"></a>[block containers](https://www.w3.org/TR/css-display-3/#block-container) that establish an [inline formatting context](https://www.w3.org/TR/css-display-3/#inline-formatting-context) |
| <strong><a href="https://drafts.csswg.org/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                          |
| <strong><a href="https://drafts.csswg.org/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                          |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                       |
| <strong><a href="https://drafts.csswg.org/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified integer                                                                                                                                                                                                            |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                  |
| <strong><a href="https://drafts.csswg.org/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                       |

<a id="ref-for-propdef-orphans②"></a>

<a id="ref-for-fragment①"></a>

<a id="ref-for-propdef-widows②"></a>

<a id="ref-for-fragment②"></a>

The [orphans](#propdef-orphans) property specifies the minimum number of line boxes in a block container that must be left in a [fragment](#fragment) <em>before</em> a fragmentation break. The [widows](#propdef-widows) property specifies the minimum number of line boxes of a block container that must be left in a [fragment](#fragment) <em>after</em> a break. Examples of how they are used to control fragmentation breaks are given [below](#widows-orphans-example).

<a id="ref-for-propdef-orphans③"></a>

<a id="ref-for-propdef-widows③"></a>

Only positive integers are allowed as values of [orphans](#propdef-orphans) and [widows](#propdef-widows). Negative values and zero are invalid and must cause the declaration to be [ignored](https://www.w3.org/TR/CSS21/conform.html#ignore).

<a id="ref-for-propdef-widows④"></a>

<a id="ref-for-propdef-orphans④"></a>

If a block contains fewer lines than the value of [widows](#propdef-widows) or [orphans](#propdef-orphans), the rule simply becomes that all lines in the block must be kept together.

<a id="ref-for-propdef-page-break-before"></a>

<a id="ref-for-propdef-page-break-after"></a>

<a id="ref-for-propdef-page-break-inside"></a>

### <a id="page-break-properties"></a>3.4.  Page Break Aliases: the [page-break-before](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-before), [page-break-after](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-after), and [page-break-inside](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-inside) properties

<a id="ref-for-propdef-page-break-before①"></a>

<a id="ref-for-propdef-page-break-after①"></a>

<a id="ref-for-propdef-page-break-inside①"></a>

<a id="ref-for-propdef-break-before④"></a>

<a id="ref-for-propdef-break-after④"></a>

<a id="ref-for-propdef-break-inside③"></a>

<a id="ref-for-legacy-shorthand"></a>

For compatibility with [CSS Level 2](https://www.w3.org/TR/CSS21/page.html), UAs that conform to [\[CSS21\]](#biblio-css21) must alias the [page-break-before](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-before), [page-break-after](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-after), and [page-break-inside](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-inside) properties to [break-before](#propdef-break-before), [break-after](#propdef-break-after), and [break-inside](#propdef-break-inside) by treating the page-break-\* properties as [legacy shorthands](https://www.w3.org/TR/css-cascade-4/#legacy-shorthand) for the break-\* properties with the following value mappings:

| Shorthand (page-break-\*) Values | Longhand (break-\*) Values     |
|----------------------------------|--------------------------------|
| auto \| left \| right \| avoid   | auto \| left \| right \| avoid |
| always                           | page                           |

## <a id="breaking-rules"></a>4.  Rules for Breaking

<a id="ref-for-fragmented-flow④"></a>

<a id="ref-for-fragmentainer①⑥"></a>

A [fragmented flow](#fragmented-flow) may be broken across [fragmentainers](#fragmentainer) at a number of [possible break points](#possible-breaks). In the case of [forced breaks](#forced-breaks), the UA is required to break the flow at that point. In the case of [unforced breaks](#unforced-breaks), the UA has to choose among the possible breaks that are allowed.

<a id="ref-for-block-size"></a>

To guarantee progress, fragmentainers are assumed to have a minimum [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) of 1px regardless of their used size.

### <a id="possible-breaks"></a>4.1.  Possible Break Points

Fragmentation splits boxes in the block flow dimension. In block-and-inline flow, breaks may occur at the following places:

<a id="btw-blocks"></a>Class A  
Between sibling boxes of the following types:

Block-parallel Fragmentation  
When the block flow direction of the siblings' containing block is parallel to that of the fragmentation context: [in-flow](css2--visuren.html--3f334c530cf4.md#positioning-scheme) block-level boxes, a float and an immediately-adjacent in-flow or floated box, table row group boxes, table row boxes, multi-column column row boxes.

Block-perpendicular Fragmentation  
When the block flow direction of the siblings' containing block is perpendicular to that of the fragmentation context: table column group boxes, table column boxes, multi-column column boxes.

<a id="btw-lines"></a>Class B  
Between line boxes inside a block container box.

<a id="end-block"></a>Class C  
Between the content edge of a block container box and the outer edges of its child content (margin edges of block-level children or line box edges for inline-level children) <em>if</em> there is a (non-zero) gap between them.

> <strong data-conversion-semantic="note">Note</strong>
>
> There is no inherent prioritization among these classes of break points. However, individual break points may be prioritized or de-prioritized by using the [breaking controls](#breaking-controls).

> <strong data-conversion-semantic="note">Note</strong>
>
> Other layout models may add breakpoints to the above classes. For example, [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1) adds certain points within a flex formatting context to classes A and C.

Some content is not fragmentable, for example many types of [replaced elements](https://www.w3.org/TR/CSS21/conform.html#replaced-element) [\[CSS21\]](#biblio-css21) (such as images or video), scrollable elements, or a single line of text content. Such content is considered <a id="monolithic"></a>monolithic: it contains no possible break points. Any forced breaks within such boxes therefore cannot split the box, and must therefore also be ignored by the box’s own fragmentation context.

<a id="ref-for-monolithic"></a>

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-valdef-overflow-auto"></a>

<a id="ref-for-valdef-overflow-scroll"></a>

<a id="ref-for-propdef-overflow①"></a>

<a id="ref-for-valdef-width-auto"></a>

In addition to any content which is not generally fragmentable, UAs may consider as [monolithic](#monolithic) any elements with [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) set to [auto](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-auto) or [scroll](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-scroll) and any elements with [overflow: hidden](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) and a non-[auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) [logical height](https://www.w3.org/TR/css3-writing-modes/#block-size) (and no specified maximum logical height).

<a id="ref-for-valdef-display-inline-block"></a>

<a id="ref-for-valdef-display-inline-table"></a>

<a id="ref-for-display-type①"></a>

<a id="ref-for-independent-formatting-context"></a>

<a id="ref-for-monolithic①"></a>

Since line boxes contain no possible break points, [inline-block](https://www.w3.org/TR/css-display-3/#valdef-display-inline-block) and [inline-table](https://www.w3.org/TR/css-display-3/#valdef-display-inline-table) boxes (and other inline-level [display types](https://www.w3.org/TR/css-display-3/#display-type) that establish an [independent formatting context](https://www.w3.org/TR/css-display-3/#independent-formatting-context)) may also be considered [monolithic](#monolithic): that is, in the cases where a single line box is too large to fit within its fragmentainer even by itself and the UA chooses to split the line box, it may fragment such boxes or it may treat them as monolithic.

### <a id="break-types"></a>4.2.  Types of Breaks

There are different types of breaks in CSS, defined based on the type of fragmentainers they span:

<a id="page-break"></a>page break  
A break between two [page boxes](https://www.w3.org/TR/css3-page/#page-box). [\[CSS3PAGE\]](#biblio-css3page)

<a id="spread-break"></a>spread break  
A break between two page boxes that are not associated with [facing pages](https://www.w3.org/TR/css3-page/#facing-pages). A spread break is always also a page break. [\[CSS3PAGE\]](#biblio-css3page)

<a id="column-break"></a>column break  
<a id="ref-for-region-break"></a>

<a id="ref-for-page-break"></a>

A break between two [column boxes](https://www.w3.org/TR/css3-multicol/#column-box). Note that if the column boxes are on different pages, then the break is also a [page break](#page-break). Similarly, if the column boxes are in different regions, then the break is also a [region break](#region-break). [\[CSS3COL\]](#biblio-css3col)

<a id="region-break"></a>region break  
<a id="ref-for-page-break①"></a>

A break between two [regions](https://www.w3.org/TR/css3-regions/#regions). Note that if the region boxes are on different pages, then the break is also a [page break](#page-break). [\[CSS3-REGIONS\]](#biblio-css3-regions)

> <strong data-conversion-semantic="note">Note</strong>
>
> A fifth type of break is the <a id="line-break"></a>line break, which is a break between two [line boxes](https://www.w3.org/TR/CSS21/visuren.html#line-box). These are not covered in this specification; see [\[CSS21\]](#biblio-css21) [\[CSS3TEXT\]](#biblio-css3text).

### <a id="forced-breaks"></a>4.3.  Forced Breaks

<a id="ref-for-forced-break①"></a>

<a id="ref-for-propdef-break-after⑤"></a>

<a id="ref-for-propagate③"></a>

<a id="ref-for-propdef-break-before⑤"></a>

<a id="ref-for-propagate④"></a>

<a id="ref-for-forced-break-values"></a>

<a id="ref-for-forced-break-values①"></a>

<a id="ref-for-avoid-break-values"></a>

A <a id="forced-break"></a>forced break is one explicitly indicated by the style sheet author. A [forced break](#forced-break) occurs at a [class A break point](#btw-blocks) if, among the [break-after](#propdef-break-after) properties specified on or [propagated](#propagate) to the earlier sibling box and the [break-before](#propdef-break-before) properties specified on or [propagated](#propagate) to the later sibling box there is at least one with a [forced break value](#forced-break-values). (Thus a [forced break value](#forced-break-values) effectively overrides any [avoid break value](#avoid-break-values) that also applies at that break point.)

<a id="ref-for-forced-break-values②"></a>

<a id="ref-for-valdef-break-before-left①"></a>

<a id="ref-for-valdef-break-before-right①"></a>

<a id="ref-for-valdef-break-before-recto①"></a>

<a id="ref-for-valdef-break-before-verso①"></a>

When multiple [forced break values](#forced-break-values) apply to a single break point, they combine such that all types of break are honored. When [left](#valdef-break-before-left), [right](#valdef-break-before-right), [recto](#valdef-break-before-recto), and/or [verso](#valdef-break-before-verso) are combined, the value specified on the latest element in the flow wins.

<a id="ref-for-propdef-page"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> A forced page break must also occur at a [class A break point](#btw-blocks) if the last line box above this margin and the first one below it do not have the same value for [page](https://www.w3.org/TR/css3-page/#propdef-page). See [\[CSS3PAGE\]](#biblio-css3page)

When a forced break occurs, it forces ensuing content into the next fragmentainer of the type associated with the break, breaking through as many fragmentation contexts as necessary until the specified break types are all satisfied. If the forced break is not contained within a matching type of fragmentation context, then the forced break has no effect.

### <a id="unforced-breaks"></a>4.4.  Unforced Breaks

<a id="ref-for-fragmentainer①⑦"></a>

While [breaking controls](#breaking-controls) can force breaks, they can also discourage them. An <a id="unforced-break"></a>unforced break is one that is inserted automatically by the UA in order to prevent content from overflowing the [fragmentainer](#fragmentainer). The following rules control whether unforced breaking at a [possible break point](#possible-breaks) is allowed:

Rule 1  
<a id="ref-for-valdef-break-before-avoid-region②"></a>

<a id="ref-for-valdef-break-before-avoid-column①"></a>

<a id="ref-for-valdef-break-before-avoid-page①"></a>

<a id="ref-for-valdef-break-before-avoid①"></a>

<a id="ref-for-propdef-break-before⑥"></a>

<a id="ref-for-propdef-break-after⑥"></a>

<a id="ref-for-fragmented-flow⑤"></a>

A [fragmented flow](#fragmented-flow) may break at a [class A break point](#btw-blocks) only if all the [break-after](#propdef-break-after) and [break-before](#propdef-break-before) values applicable to this break point allow it, which is when at least one of them forces a break or when none of them forbid it ([avoid](#valdef-break-before-avoid) or [avoid-page](#valdef-break-before-avoid-page)/[avoid-column](#valdef-break-before-avoid-column)/[avoid-region](#valdef-break-before-avoid-region), depending on the [break type](#break-types)).

Rule 2  
<a id="ref-for-valdef-break-inside-avoid"></a>

<a id="ref-for-propdef-break-inside④"></a>

<a id="ref-for-valdef-break-before-auto"></a>

However, if all of them are [auto](#valdef-break-before-auto) and a common ancestor of all the elements has a [break-inside](#propdef-break-inside) value of [avoid](#valdef-break-inside-avoid), then breaking here is not allowed.

Rule 3  
<a id="ref-for-propdef-widows⑤"></a>

<a id="ref-for-propdef-orphans⑤"></a>

Breaking at a [class B break point](#btw-lines) is allowed only if the number of line boxes between the break and the start of the enclosing block box is the value of [orphans](#propdef-orphans) or more, and the number of line boxes between the break and the end of the box is the value of [widows](#propdef-widows) or more.

Rule 4  
<a id="ref-for-valdef-break-inside-auto"></a>

<a id="ref-for-propdef-break-inside⑤"></a>

Additionally, breaking at [class B](#btw-blocks) or [class C](#end-block) break points is allowed only if the [break-inside](#propdef-break-inside) property of all ancestors is [auto](#valdef-break-inside-auto).

<a id="ref-for-fragmentainer①⑧"></a>

If the above doesn’t provide enough break points to keep content from overflowing the [fragmentainer](#fragmentainer), then rule 3 is dropped to provide more break points.

If that still does not lead to sufficient break points, then rules 1, 2 and 4 are dropped in order to find additional breakpoints. In this case the UA may use the avoids that are in effect at those points to weigh the appropriateness of the new breakpoints; however, this specification does not suggest a precise algorithm.

<a id="ref-for-valdef-box-decoration-break-clone"></a>

<a id="ref-for-valdef-box-decoration-break-clone①"></a>

If even that does not lead to sufficient break points, [cloned margins/border/padding](#valdef-box-decoration-break-clone) at on the block-end side are truncated; and if more room is still needed, [cloned margins/border/padding](#valdef-box-decoration-break-clone) are truncated at the block-end side as well.

<a id="ref-for-monolithic②"></a>

Finally, if there are no possible break points below the top of the fragmentainer, and not all the content fits, the UA may break anywhere in order to avoid losing content off the edge of the fragmentainer. <a id="monolithic-breaking"></a> In such cases, the UA may also fragment the contents of [monolithic](#monolithic) elements by slicing the element’s graphical representation. However, the UA must not break at the top of the page, i.e. it must place at least some content on each fragmentainer, so that each fragmentainer has a non-zero amount of content, in order to guarantee progress through the content.

### <a id="best-breaks"></a>4.5.  Optimizing Unforced Breaks

<a id="ref-for-fragmented-flow⑥"></a>

While CSS3 requires that a [fragmented flow](#fragmented-flow) must break at allowed break points in order to avoid overflowing the fragmentainers in its fragmentation context, it does not define whether content breaks at a particular [allowed break](#unforced-breaks). However, it is recommended that user agents observe the following guidelines (while recognizing that they are sometimes contradictory):

- Break as few times as possible.
- Make all fragmentainers that don’t end with a forced break appear to be equally filled with content.
- Avoid breaking inside a replaced element.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="widows-orphans-example"></a>
>
> Suppose, for example, that the style sheet contains orphans : 4, widows : 2, and there is space for 20 lines (line boxes) available at the bottom of the current page, and the next block in normal flow is considered for placement:
>
> - If the block contains 20 line boxes or fewer, it should be placed on the current page.
>
> - <a id="ref-for-propdef-widows⑥"></a>
>
>   If the block contains 21 or 22 line boxes, the second fragment of the paragraph must not violate the [widows](#propdef-widows) constraint, and so the second fragment must contain at least two line boxes; likewise the first fragment must contain at least four line boxes.
>
> - If the block contains 23 line boxes or more, the first fragment should contain 20 lines and the second fragment the remaining lines. But if any fragment of the block is placed on the current page, that fragment must contain at least four line boxes and the second fragment at least two line boxes.
>
> <a id="ref-for-propdef-orphans⑥"></a>
>
> <a id="ref-for-propdef-widows⑦"></a>
>
> Now suppose that [orphans](#propdef-orphans) is 10, [widows](#propdef-widows) is 20, and there are 8 lines available at the bottom of the current page:
>
> - If the block contains 8 lines or fewer, it should be placed on the current page.
>
> - <a id="ref-for-propdef-orphans⑦"></a>
>
>   If the block contains 9 lines or more, it must NOT be split (that would violate the [orphans](#propdef-orphans) constraint), so it must move as a block to the next page.

<a id="ref-for-box-fragment①"></a>

<a id="ref-for-fragmentation-break"></a>

<a id="ref-for-fragmentainer①⑨"></a>

Additionally, CSS imposes one requirement: a zero-sized [box fragment](#box-fragment), since it does not take up space, must appear on the earlier side of a [fragmentation break](#fragmentation-break) if it is able to fit within the [fragmentainer](#fragmentainer).

<a id="ref-for-box-fragment②"></a>

<a id="ref-for-fragmentainer②⓪"></a>

<a id="ref-for-fragmentainer②①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A zero-sized [box fragment](#box-fragment) will be pushed to the next [fragmentainer](#fragmentainer) if it is placed immediately after content that itself overflows the [fragmentainer](#fragmentainer).

## <a id="breaking-boxes"></a>5.  Box Model for Breaking

> <strong data-conversion-semantic="note">Note</strong>
>
> The sizing terminology used in this section is defined in [\[CSS3-SIZING\]](#biblio-css3-sizing).

### <a id="varying-size-boxes"></a>5.1.  Breaking into Varying-size Fragmentainers

When a flow is fragmented into varying-size fragmentainers, the following rules are observed for adapting layout:

- <a id="ref-for-block-size①"></a>

  <a id="ref-for-inline-size"></a>

  <a id="ref-for-monolithic③"></a>

  Layout is performed per-fragmentainer, with each fragmentainer continuing progress from the breakpoint on the previous, but recalculating sizes and positions using its own size as if the entire element were fragmented across fragmentainers of this size. Progress is measured in percentages (not absolute lengths) of used/remaining fragmentainer extent and in amount of used/remaining content. However, when laying out [monolithic](#monolithic) elements, the UA may instead maintain a consistent [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) and resolved [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) across fragmentainers.

- Intrinsic sizes are calculated and maintained across the entire element. Where an initial containing block size is needed to resolve an intrinsic size, assume the size of the first fragmentainer defining a fragmentation context.

- <a id="ref-for-propdef-box-decoration-break①"></a>

  <a id="ref-for-block-start①"></a>

  <a id="ref-for-block-start"></a>

  Fragments of boxes that began on a previous fragmentainer must obey placement rules with the additional constraint that fragments must not be positioned above the [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) edge of the fragmentainer. If this results in a box’s continuation fragment shifting away from the [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) edge of the fragmentainer, then [box-decoration-break: clone](#propdef-box-decoration-break), if specified, wraps the fragment with the box’s margin in addition to its padding and border.

  ![Illustration: Breaking in varying-size fragmentainers](https://www.w3.org/TR/2018/CR-css-break-3-20181204/images/Varying-Size-Fragmentainers.svg)

  Illustration of breaking in varying-size fragmentainers.

> <strong data-conversion-semantic="note">Note</strong>
>
> Since document order of elements doesn’t change during fragmentation, fragments are processed following the same rules that apply to continuous media. In particular, the order of floats is preserved across all fragments and follows the same rules as defined in CSS 2.1 9.5.

Below are listed (informatively) some implications of these rules:

- <a id="ref-for-inline-size①"></a>

  <a id="ref-for-stretch-fit-size"></a>

  Boxes (including tables) fullfilling layout constraints at their [stretch-fit](https://www.w3.org/TR/css-sizing-3/#stretch-fit-size) or percentage-based size may change [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) across pages.

- <a id="ref-for-inline-size②"></a>

  <a id="ref-for-max-content"></a>

  <a id="ref-for-min-content"></a>

  Boxes (including tables) fulfilling layout constraints at their [min-content](https://www.w3.org/TR/css-sizing-3/#min-content), [max-content](https://www.w3.org/TR/css-sizing-3/#max-content), or absolute-length size will maintain their [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) across pages.

- A block-level continuation fragment may be placed below the top of the page if, e.g. it establishes a block formatting context and is placed beside a float and both it and the float continue onto a narrower page that is too narrow to hold both of them side-by-side.

- An element adjacent to a preceding float on one page may wind up above the float’s continuation on the next page if, e.g. that float is pushed down because it no longer fits side-by-side with an earlier float that also continues onto this narrower page.

- A left float may appear on a page <em>before</em> the remaining fragments of a preceding right float if that right float does not fit on the earlier page. However another right float will be forced down until the preceding right float’s remaining fragment can be placed.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a973f990"></a>
>
> <a id="ref-for-propdef-top"></a>
>
> <a id="ref-for-propdef-height"></a>
>
> Here is an example that shows the use of percentage-based progress: Suppose we have an absolutely-positioned element that is positioned [top: calc(150% + 30px)](https://www.w3.org/TR/css3-positioning/#propdef-top) and has [height: calc(100% - 10px)](https://www.w3.org/TR/CSS21/visudet.html#propdef-height). If it is placed into a paginated context with a first page height of 400px, a second page of 200px, and a third page of 600px, its layout progresses as follows:
>
> - First, the top position is resolved against the height of the first page. This results in 630px. Since the first page has a height of only 400px, layout moves to the second page, recording progress of 400/630 = 63.49% with 36.51% left to go.
> - Now on the second page, the top position is again resolved, this time against the height of the second page. This results in 330px. The remaining 36.51% of progress thus resolves to 120.5px, placing the top edge of the element 120.5px down the second page.
> - Now the height is resolved against the second page; it resolves to 190px. Since there are only 79.5px left on the page, layout moves to the third page, recording progress of 79.5/190 = 41.84%, with 58.16% left to go.
> - On the third page, the height resolves to 590px. The remaining 58.16% of progress thus resolves to 343.1px, which fits on this page and completes the element.

### <a id="break-margins"></a>5.2.  Adjoining Margins at Breaks

<a id="ref-for-valdef-box-decoration-break-clone②"></a>

When an unforced break occurs before or after a block-level box, any margins adjoining the break are truncated to zero. When a forced break occurs there, adjoining margins before the break are truncated, but margins after the break are preserved. [Cloned margins](#valdef-box-decoration-break-clone) are always truncated to zero on block-level boxes.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: CSS Fragmentation Level 4 will introduce control over margin truncation at breaks.

### <a id="box-splitting"></a>5.3.  Splitting Boxes

<a id="ref-for-remaining-fragmentainer-extent"></a>

<a id="ref-for-propdef-box-decoration-break②"></a>

<a id="ref-for-fragmentainer②②"></a>

<a id="ref-for-fragmentation-break①"></a>

<a id="ref-for-fragmentainer②③"></a>

<a id="ref-for-block-size②"></a>

When a box breaks, its content box extends to fill any [remaining fragmentainer extent](#remaining-fragmentainer-extent) (leaving room for any margins/borders/padding applied by [box-decoration-break: clone](#propdef-box-decoration-break)) before the content resumes on the next [fragmentainer](#fragmentainer). (A [fragmentation break](#fragmentation-break) that pushes content to the next [fragmentainer](#fragmentainer) effectively increases the [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) of a box’s contents.)

<a id="ref-for-block-size③"></a>

<a id="ref-for-fragmentainer②④"></a>

<a id="ref-for-block-size④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The extra [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size) contributed by fragmenting the box (i.e. the distance from the break point to the edge of the [fragmentainer](#fragmentainer)) contributes progress towards any specified limits on the box’s [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size).

![Illustration: Filling remaining fragmentainer extent](https://www.w3.org/TR/2018/CR-css-break-3-20181204/images/Remaining-Fragmentainer-Extent.svg)

<a id="ref-for-remaining-fragmentainer-extent①"></a>

Illustration of filling the [remaining fragmentainer extent](#remaining-fragmentainer-extent).

<a id="ref-for-propdef-box-decoration-break③"></a>

### <a id="break-decoration"></a>5.4.  Fragmented Borders and Backgrounds: the [box-decoration-break](#propdef-box-decoration-break) property

| Field               | Definition                                                                        |
|---------------------|-----------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-box-decoration-break"></a>box-decoration-break                                           |
| <strong><a href="https://drafts.csswg.org/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑤"></a>slice [\|](https://www.w3.org/TR/css-values-4/#comb-one) clone |
| <strong><a href="https://drafts.csswg.org/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | slice                                                                             |
| <strong>Applies to:&#xA;      </strong> | [all elements](https://drafts.csswg.org/css-pseudo/#generated-content)            |
| <strong><a href="https://drafts.csswg.org/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                |
| <strong><a href="https://drafts.csswg.org/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                               |
| <strong>Media:&#xA;      </strong> | visual                                                                            |
| <strong><a href="https://drafts.csswg.org/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                 |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                       |
| <strong><a href="https://drafts.csswg.org/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                          |

<a id="ref-for-propdef-box-decoration-break④"></a>

When a break (page/column/region/line) splits a box, the [box-decoration-break](#propdef-box-decoration-break) property controls

- whether the box’s margins, borders, padding, and other decorations wrap the broken edges of the box fragments

- <a id="ref-for-mask-positioning-area"></a>

  how the [](https://www.w3.org/TR/css3-background/#background-positioning-area)background positioning area [\[CSS3BG\]](#biblio-css3bg) (and [mask positioning area](https://www.w3.org/TR/css-masking-1/#mask-positioning-area) [\[CSS-MASKING-1\]](#biblio-css-masking-1), shape reference box [\[CSS-SHAPES-1\]](#biblio-css-shapes-1), etc.) is derived from or duplicated across the box fragments and how the element’s background is drawn within them.

Values have the following meanings:

<a id="valdef-box-decoration-break-clone"></a>clone  
<a id="ref-for-propdef-box-shadow"></a>

<a id="ref-for-propdef-border-image"></a>

<a id="ref-for-propdef-border-radius"></a>

Each box fragment is independently wrapped with the border, padding, and margin. The [border-radius](https://www.w3.org/TR/css3-background/#propdef-border-radius) and [border-image](https://www.w3.org/TR/css3-background/#propdef-border-image) and [box-shadow](https://www.w3.org/TR/css3-background/#propdef-box-shadow), if any, are applied to each fragment independently. The background is drawn independently in each fragment of the element. A no-repeat background image will thus be rendered once in each fragment of the element.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Cloned margins are [truncated](#break-margins) on block-level boxes.

<a id="valdef-box-decoration-break-slice"></a>slice  
<a id="ref-for-propdef-border-radius①"></a>

<a id="ref-for-propdef-border-image①"></a>

The effect is as though the element were rendered with no breaks present, and then sliced by the breaks afterward: no border and no padding are inserted at a break; no box-shadow is drawn at a broken edge; and backgrounds, [border-radius](https://www.w3.org/TR/css3-background/#propdef-border-radius), and the [border-image](https://www.w3.org/TR/css3-background/#propdef-border-image) are applied to the geometry of the whole box as if it were unbroken.

![Illustration: (1) a single box cut in two in between two lines of text by a page break and (2) two boxes, one before and one after the page break, both with a border all around and their own background image](https://www.w3.org/TR/2018/CR-css-break-3-20181204/images/box-break.png)

<a id="ref-for-propdef-box-decoration-break⑤"></a>

<a id="ref-for-valdef-box-decoration-break-slice"></a>

<a id="ref-for-valdef-box-decoration-break-clone③"></a>

Two possibilities for [box-decoration-break](#propdef-box-decoration-break): on the left, the value [slice](#valdef-box-decoration-break-slice), on the right the value [clone](#valdef-box-decoration-break-clone).

<a id="ref-for-propdef-box-decoration-break⑥"></a>

<a id="ref-for-display-type②"></a>

<a id="ref-for-block-level-box"></a>

<a id="ref-for-inline-box"></a>

<a id="ref-for-block-container①"></a>

<a id="ref-for-valdef-box-decoration-break-slice①"></a>

UAs should also apply [box-decoration-break](#propdef-box-decoration-break) to control rendering at bidi-imposed breaks—i.e. when bidi reordering causes an inline to split into non-contiguous fragments—and/or at display-type–imposed breaks—i.e. when a higher-level [display type](https://www.w3.org/TR/css-display-3/#display-type) (such as a [block-level box](https://www.w3.org/TR/css-display-3/#block-level-box) / [column spanner](https://www.w3.org/TR/css-multicol-1/#spanning-columns)) splits an incompatible ancestor (such as an [inline box](https://www.w3.org/TR/css-display-3/#inline-box) / [block container](https://www.w3.org/TR/css-display-3/#block-container)). Otherwise such breaks must be handled as [slice](#valdef-box-decoration-break-slice). See [Applying the Bidirectional Reorderign Algorithm](https://www.w3.org/TR/css-writing-modes-3/#bidi-algo) in [CSS Writing Modes](https://www.w3.org/TR/css-writing-modes-3/#text-direction), [CSS2§9.2 Block-level elements and block boxes](https://www.w3.org/TR/CSS2/visuren.html#box-gen), and [CSS Multi-column Layout §6 Spanning Columns](https://www.w3.org/TR/css-multicol-1/#spanning-columns).

<a id="ref-for-propdef-direction"></a>

<a id="ref-for-propdef-direction①"></a>

<a id="ref-for-propdef-direction②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> For inline elements, which side of a fragment is considered the broken edge is determined by the parent element’s inline progression direction. For example, if an inline element whose parent has [direction: rtl](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) breaks across two lines, the <em>left</em> edge of the fragment on the first line will be the broken edge. (Note in particular that neither the element’s own [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) nor its containing block’s [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) is used.) See [\[CSS3-WRITING-MODES\]](#biblio-css3-writing-modes).

<a id="ref-for-valdef-box-decoration-break-slice②"></a>

#### <a id="joining-boxes"></a>5.4.1.  Joining Boxes for [slice](#valdef-box-decoration-break-slice)

<a id="ref-for-propdef-box-decoration-break⑦"></a>

<a id="ref-for-propdef-border-image②"></a>

For [box-decoration-break: slice](#propdef-box-decoration-break), backgrounds (and [border-image](https://www.w3.org/TR/css3-background/#propdef-border-image)) are drawn as if applied to a composite box consisting of all of the box’s fragments reassembled in visual order. This theoretical assembly occurs after the element has been laid out (including any justification, bidi reordering, page breaks, etc.). To assemble the composite box...

For boxes broken across lines  
<a id="ref-for-valdef-direction-ltr"></a>

<a id="ref-for-propdef-direction③"></a>

<a id="ref-for-inline-base-direction"></a>

First, fragments on the same line are connected in visual order. Then, fragments on subsequent lines are ordered according to the element’s [inline base direction](https://www.w3.org/TR/css-writing-modes-4/#inline-base-direction) and aligned on the element’s dominant baseline. For example, in a left-to-right containing block ([direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) is [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr)), the first fragment is the leftmost fragment on the first line and fragments from subsequent lines are put to the right of it. In a right-to-left containing block, the first fragment is the rightmost on the first line and subsequent fragments are put to the left of it.

For boxes broken across columns  
<a id="ref-for-block-flow-direction"></a>

Fragments are connected as if the column boxes were glued together in the [block flow direction](https://www.w3.org/TR/css-writing-modes-4/#block-flow-direction) of the multi-column container.

For boxes broken across pages  
<a id="ref-for-block-flow-direction①"></a>

Fragments are connected as if page content areas were glued together in the [block flow direction](https://www.w3.org/TR/css-writing-modes-4/#block-flow-direction) of the root element.

For boxes broken across regions  
<a id="ref-for-region-chain"></a>

<a id="ref-for-principal-writing-mode"></a>

<a id="ref-for-block-flow-direction②"></a>

Fragments are connected as if region content areas were glued together in the [block flow direction](https://www.w3.org/TR/css-writing-modes-4/#block-flow-direction) of the [principal writing mode](https://www.w3.org/TR/css-writing-modes-4/#principal-writing-mode) of the [region chain](https://drafts.csswg.org/css-regions-1/#region-chain).

If the box fragments have different widths (heights, if the fragments are joined horizontally), then each piece draws its portion of the background assuming that the whole element has the same width (height) as this piece. However, if the used height (width) of an image is derived from the width (height) of the box, then it is calculated using the widest fragment’s width and maintained as a fixed size. This ensures that right-aligned images stay aligned to the right edge, left-aligned images stay aligned to the left edge, centered images stay centered, and stretched images cover the background area as intended while preserving continuity across fragments.

### <a id="transforms"></a>5.5.  Transforms, Positioning, and Pagination

Fragmentation interacts with layout, and thus occurs <em>before</em> relative positioning [\[CSS21\]](#biblio-css21), transforms [\[CSS3-TRANSFORMS\]](#biblio-css3-transforms), and any other graphical effects. Such effects are applied per fragment: for example, rotation applied to a fragmented box will calculate a rotation origin for each fragment and independently rotate that fragment around its origin. (The origin of an overflow-only fragment is determined as if that content were overflowing an empty box with zero margins/borders/padding at the start of the fragmentainer.) However, in order to reduce dataloss when printing, the separation and transfer of page boxes <em>should</em> occur last; thus a transformed fragment that spans pages <em>should</em> be sliced at the page breaks and print in its entirety rather than being clipped by its originating page.

![Illustration: Transformed overflow fragmentation](https://www.w3.org/TR/2018/CR-css-break-3-20181204/images/fragmented-transforms.png)

A fixed-height box spanning 2.5 pages with overflow content spanning to a total of 4 pages. The transform origin of each fragment is the center of its border box; the fragment without a border box assumes a zero-height box at the start of the overflow.

Absolute positioning affects layout and thus interacts with fragmentation. Both the coordinate system and absolutely-positioned boxes belonging to a containing block will fragment across fragmentainers in the same fragmentation flow as the containing block.

<a id="ref-for-fragmentation-break②"></a>

<a id="ref-for-block-start②"></a>

UAs are not required to correctly position boxes that span a [fragmentation break](#fragmentation-break) and whose [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) edge position depends on where the box’s content fragments.

UAs with memory constraints that prevent them from manipulating an entire document in memory are not required to correctly position absolutely-positioned elements that end up on a previously-rendered page.

## <a id="changes"></a> Changes

The following significant changes were made since the [14 January 2016 Candidate Recommendation](https://www.w3.org/TR/2016/CR-css-break-3-20160114/):

- <a id="change-2018-aliasing"></a> Clarified the mechanism of aliasing for the page-break-\* properties. ([Issue 866](https://github.com/w3c/csswg-drafts/issues/866))

  > <a id="ref-for-legacy-shorthand①"></a>
  >
  > by treating the page-break-\* properties as ~~shorthands~~ <u>[legacy shorthands](https://www.w3.org/TR/css-cascade-4/#legacy-shorthand)</u> for the break-\* properties

- <a id="ref-for-out-of-flow"></a>

  <a id="change-2017-propagation"></a> Clarified that break propagation does not affect computed values and that other layout modes (e.g. flex and grid) make adjustments to the basic break propagation rules, and corrected child-to-parent propagation to ignore [out-of-flow](https://www.w3.org/TR/css-display-3/#out-of-flow) children. ([Issue 2614](https://github.com/w3c/csswg-drafts/issues/2614)) See [§3.1.1 Child→Parent Break Propagation](#break-propagation).

- <a id="ref-for-propdef-orphans⑧"></a>

  <a id="ref-for-propdef-widows⑧"></a>

  <a id="change-2017-widows-applies-to"></a> Clarified that [widows](#propdef-widows) and [orphans](#propdef-orphans) have no effect on block containers that do not directly contain line boxes. ([Issue 1823](https://github.com/w3c/csswg-drafts/issues/1823))

  > Applies to: block containers <u>that establish a new inline formatting context</u>

- <a id="change-2018-truncate-margins"></a> Clarified that margins adjoining a Class C break are also truncated in the same way as margins between siblings (Class A). ([Issue 3073](https://github.com/w3c/csswg-drafts/issues/3073))

  > <a id="ref-for-remaining-fragmentainer-extent②"></a>
  >
  > When an unforced break occurs ~~between~~ <u>before or after a block-level box ~~es~~ , any margins adjoining the break ~~truncate to the [remaining fragmentainer extent](#remaining-fragmentainer-extent) before the break, and~~ are truncated to zero ~~after the break~~ .</u>

- <a id="ref-for-monolithic④"></a>

  <a id="change-2017-inline-block-fragmentation"></a> Clarified what it means if a UA chooses not to treat atomic inlines as not [monolithic](#monolithic). ([Issue 1111](https://github.com/w3c/csswg-drafts/issues/1111))

  > <a id="ref-for-valdef-display-inline-block①"></a>
  >
  > <a id="ref-for-valdef-display-inline-table①"></a>
  >
  > <a id="ref-for-display-type③"></a>
  >
  > <a id="ref-for-independent-formatting-context①"></a>
  >
  > <a id="ref-for-monolithic⑤"></a>
  >
  > Since line boxes contain no possible break points, [inline-block](https://www.w3.org/TR/css-display-3/#valdef-display-inline-block) and [inline-table](https://www.w3.org/TR/css-display-3/#valdef-display-inline-table) boxes (and other inline-level [display types](https://www.w3.org/TR/css-display-3/#display-type) that establish an [independent formatting context](https://www.w3.org/TR/css-display-3/#independent-formatting-context)) may also be considered [monolithic](#monolithic) <u>that is, in the cases where a single line box is too large to fit within its fragmentainer even by itself and the UA chooses to split the line box, it may fragment such boxes or it may treat them as monolithic</u> .

- <a id="change-2017-zero-fragment"></a> Added requirement that zero-sized fragments must stay on the previous fragmentainer. ([Issue 1529](https://github.com/w3c/csswg-drafts/issues/1529))

  > <a id="ref-for-box-fragment③"></a>
  >
  > <a id="ref-for-fragmentation-break③"></a>
  >
  > <a id="ref-for-fragmentainer②⑤"></a>
  >
  > Additionally, CSS imposes one requirement: a zero-sized [box fragment](#box-fragment), since it does not take up space, must appear on the earlier side of a [fragmentation break](#fragmentation-break) if it is able to fit within the [fragmentainer](#fragmentainer).
  >
  > <a id="ref-for-box-fragment④"></a>
  >
  > <a id="ref-for-fragmentainer②⑥"></a>
  >
  > <a id="ref-for-fragmentainer②⑦"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > A zero-sized [box fragment](#box-fragment) can be pushed to the next [fragmentainer](#fragmentainer) if it is placed immediately after content that itself overflows the [fragmentainer](#fragmentainer).

- <a id="ref-for-propdef-box-decoration-break⑧"></a>

  <a id="change-2018-bidi-split-breaks"></a> Clarify that bidi-imposed breaks and block-in-inline breaks create fragments, and that their formatting should be controlled by [box-decoration-break](#propdef-box-decoration-break). ([Issue 1706](https://github.com/w3c/csswg-drafts/issues/1706))

  > <a id="ref-for-fragment③"></a>
  >
  > <a id="ref-for-display-type④"></a>
  >
  > <a id="ref-for-box-fragment⑤"></a>
  >
  > > <strong data-conversion-semantic="note">Note</strong>
  > >
  > > A box can be broken into multiple [fragments](#fragment) also due to bidi reordering of text (see [Applying the Bidirectional Reorderign Algorithm](https://www.w3.org/TR/css-writing-modes-3/#bidi-algo) in [CSS Writing Modes](https://www.w3.org/TR/css-writing-modes-3/#text-direction)) or higher-level [display type](https://www.w3.org/TR/css-display-3/#display-type) box splitting, e.g. [block-in-inline splitting](https://www.w3.org/TR/CSS2/visuren.html#img-anon-block) (see [CSS2§9.2](https://www.w3.org/TR/CSS2/visuren.html#box-gen)) or [column-spanner-in-block](https://www.w3.org/TR/css-multicol-1/#spanning-columns) splitting (see [CSS Multi-column Layout](https://www.w3.org/TR/css-multicol-1/#spanning-columns)). The division into [box fragments](#box-fragment) in these cases does not depend on layout (sizing/positioning of content).

  > <a id="ref-for-propdef-box-decoration-break⑨"></a>
  >
  > <a id="ref-for-display-type⑤"></a>
  >
  > <a id="ref-for-block-level-box①"></a>
  >
  > <a id="ref-for-inline-box①"></a>
  >
  > <a id="ref-for-block-container②"></a>
  >
  > <a id="ref-for-valdef-box-decoration-break-slice③"></a>
  >
  > UAs ~~may~~ <u>should</u> also apply [box-decoration-break](#propdef-box-decoration-break) to control rendering at bidi-imposed breaks—i.e. when bidi reordering causes an inline to split into non-contiguous fragments <u>—and/or at display-type–imposed breaks—i.e. when a higher-level [display type](https://www.w3.org/TR/css-display-3/#display-type) (such as a [block-level box](https://www.w3.org/TR/css-display-3/#block-level-box) / [column spanner](https://www.w3.org/TR/css-multicol-1/#spanning-columns)) splits an incompatible ancestor (such as an [inline box](https://www.w3.org/TR/css-display-3/#inline-box) / [block container](https://www.w3.org/TR/css-display-3/#block-container))</u> . Otherwise such breaks must be handled as [slice](#valdef-box-decoration-break-slice). <u>See [Applying the Bidirectional Reorderign Algorithm](https://www.w3.org/TR/css-writing-modes-3/#bidi-algo) in [CSS Writing Modes](https://www.w3.org/TR/css-writing-modes-3/#text-direction), [CSS2§9.2 Block-level elements and block boxes](https://www.w3.org/TR/CSS2/visuren.html#box-gen), and [CSS Multi-column Layout §6 Spanning Columns](https://www.w3.org/TR/css-multicol-1/#spanning-columns).</u>

- <a id="change-2017-trivial"></a> Made a handful of trivial wording fixes.

A [Disposition of Comments](https://drafts.csswg.org/css-break-3/issues-cr-2016) is available.

The following significant changes were made since the [29 January 2015 Working Draft](https://www.w3.org/TR/2015/WD-css3-break-20150129/):

- Dropped any and always values of break-\*.

- <a id="ref-for-propdef-orphans①⓪"></a>

  <a id="ref-for-propdef-widows①⓪"></a>

  <a id="ref-for-propdef-orphans⑨"></a>

  <a id="ref-for-propdef-widows⑨"></a>

  Switched priority of [widows](#propdef-widows) and [orphans](#propdef-orphans) vs. break-\* restrictions to make [widows](#propdef-widows) and [orphans](#propdef-orphans) lower-priority rather than higher-priority.

- <a id="ref-for-propdef-box-decoration-break①⓪"></a>

  Defined that margins are also cloned for [box-decoration-break: clone](#propdef-box-decoration-break) (but are truncated in block-level layout).

- Corrected unforced breaking rules (Class A) to handle new break types (original rules only handled page breaks).

- Allowed dropping cloned box decorations when running out of room.

A [Disposition of Comments](https://drafts.csswg.org/css-break-3/issues-lc-2015) is available.

## <a id="acknowledgments"></a> Acknowledgments

The editors would like to thank Mihai Balan, Michael Day, Alex Mogilevsky, Shinyu Murakami, Florian Rivoal, and Alan Stearns for their contributions to this module. Special thanks go to the former [\[CSS3PAGE\]](#biblio-css3page) editors Jim Bigelow (HP), Melinda Grant (HP), Håkon Wium Lie (Opera), and Jacob Refstrup (HP) for their contributions to this specification, which is a successor of their work there.

## <a id="conformance"></a> Conformance

### <a id="document-conventions"></a> Document conventions

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#biblio-rfc2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a13d9f9a"></a>
>
> This is an example of an informative example.

Informative notes begin with the word “Note” and are set apart from the normative text with `class="note"`, like this:

> <strong data-conversion-semantic="note">Note</strong>
>
> Note, this is an informative note.

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with `<strong class="advisement">`, like this: <strong data-conversion-semantic="advisement">Advisement:</strong> <strong> UAs MUST provide an accessible alternative. </strong>

### <a id="conform-classes"></a> Conformance classes

Conformance to this specification is defined for three conformance classes:

style sheet  
A [CSS style sheet](https://www.w3.org/TR/CSS2/conform.html#style-sheet).

renderer  
A [UA](https://www.w3.org/TR/CSS2/conform.html#user-agent) that interprets the semantics of a style sheet and renders documents that use them.

authoring tool  
A [UA](https://www.w3.org/TR/CSS2/conform.html#user-agent) that writes a style sheet.

A style sheet is conformant to this specification if all of its statements that use syntax defined in this module are valid according to the generic CSS grammar and the individual grammars of each feature defined in this module.

A renderer is conformant to this specification if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by this specification by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)

An authoring tool is conformant to this specification if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.

### <a id="conform-responsible"></a> Requirements for Responsible Implementation of CSS

The following sections define several conformance requirements for implementing CSS responsibly, in a way that promotes interoperability in the present and future.

#### <a id="conform-partial"></a> Partial Implementations

So that authors can exploit the forward-compatible parsing rules to assign fallback values, <strong>CSS renderers <em>must</em> treat as invalid
        (and <a href="https://www.w3.org/TR/CSS2/conform.html#ignore">ignore as appropriate</a>)
        any at-rules, properties, property values, keywords, and other syntactic constructs
        for which they have no usable level of support</strong>. In particular, user agents <em>must not</em> selectively ignore unsupported property values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.

#### <a id="conform-future-proofing"></a> Implementations of Unstable and Proprietary Features

To avoid clashes with future stable CSS features, the CSSWG recommends [following best practices](https://www.w3.org/TR/CSS/#future-proofing) for the implementation of [unstable](https://www.w3.org/TR/CSS/#unstable) features and [proprietary extensions](https://www.w3.org/TR/CSS/#proprietary-extension) to CSS.

#### <a id="conform-testing"></a> Implementations of CR-level Features

Once a specification reaches the Candidate Recommendation stage, implementers should release an [unprefixed](https://www.w3.org/TR/CSS/#vendor-prefix) implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec, and should avoid exposing a prefixed variant of that feature.

To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.

Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Style&#x2F;CSS&#x2F;Test&#x2F;](https://www.w3.org/Style/CSS/Test/)&#x2E; Questions should be directed to the [public-css-testsuite@w3.org](https://lists.w3.org/Archives/Public/public-css-testsuite) mailing list.

### <a id="cr-exit-criteria"></a> CR exit criteria

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

- auto
  - [value for break-before, break-after](#valdef-break-before-auto), in §3.1
  - [value for break-inside, page-break-inside](#valdef-break-inside-auto), in §3.2
- avoid
  - [value for break-before, break-after](#valdef-break-before-avoid), in §3.1
  - [value for break-inside, page-break-inside](#valdef-break-inside-avoid), in §3.2
- [avoid break values](#avoid-break-values), in §3.1
- avoid-column
  - [value for break-before, break-after](#valdef-break-before-avoid-column), in §3.1
  - [value for break-inside, page-break-inside](#valdef-break-inside-avoid-column), in §3.2
- avoid-page
  - [value for break-before, break-after](#valdef-break-before-avoid-page), in §3.1
  - [value for break-inside, page-break-inside](#valdef-break-inside-avoid-page), in §3.2
- avoid-region
  - [value for break-before, break-after](#valdef-break-before-avoid-region), in §3.1
  - [value for break-inside, page-break-inside](#valdef-break-inside-avoid-region), in §3.2
- [box-decoration-break](#propdef-box-decoration-break), in §5.4
- [box fragment](#box-fragment), in §2
- [break](#break), in §2
- [break-after](#propdef-break-after), in §3.1
- [break-before](#propdef-break-before), in §3.1
- [break-inside](#propdef-break-inside), in §3.2
- [clone](#valdef-box-decoration-break-clone), in §5.4
- [column](#valdef-break-before-column), in §3.1
- [column break](#column-break), in §4.2
- [forced break](#forced-break), in §4.3
- [forced break values](#forced-break-values), in §3.1
- [fragment](#fragment), in §2
- [fragmentainer](#fragmentainer), in §2
- [fragmentation](#fragmentation), in §2
- [fragmentation break](#fragmentation-break), in §2
- [fragmentation container](#fragmentation-container), in §2
- [fragmentation context](#fragmentation-context), in §2
- [fragmentation direction](#fragmentation-direction), in §2
- [fragmentation root](#fragmentation-root), in §2
- [fragmented flow](#fragmented-flow), in §2
- [left](#valdef-break-before-left), in §3.1
- [line break](#line-break), in §4.2
- [monolithic](#monolithic), in §4.1
- [orphans](#propdef-orphans), in §3.3
- [page](#valdef-break-before-page), in §3.1
- [page break](#page-break), in §4.2
- [pagination](#pagination), in §1
- [propagate](#propagate), in §3.1.1
- [propagation](#propagate), in §3.1.1
- [recto](#valdef-break-before-recto), in §3.1
- [region](#valdef-break-before-region), in §3.1
- [region break](#region-break), in §4.2
- [remaining fragmentainer extent](#remaining-fragmentainer-extent), in §2
- [right](#valdef-break-before-right), in §3.1
- [slice](#valdef-box-decoration-break-slice), in §5.4
- [spread break](#spread-break), in §4.2
- [unforced break](#unforced-break), in §4.4
- [verso](#valdef-break-before-verso), in §3.1
- [widows](#propdef-widows), in §3.3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-box-3\] defines the following terms:
  - [border area](https://www.w3.org/TR/css-box-3/#border-area)
  - [margin area](https://www.w3.org/TR/css-box-3/#margin-area)
  - [padding area](https://www.w3.org/TR/css-box-3/#padding-area)
- \[css-cascade-4\] defines the following terms:
  - [computed value](https://www.w3.org/TR/css-cascade-4/#computed-value)
  - [legacy shorthand](https://www.w3.org/TR/css-cascade-4/#legacy-shorthand)
- \[css-display-3\] defines the following terms:
  - [block container](https://www.w3.org/TR/css-display-3/#block-container)
  - [block-level box](https://www.w3.org/TR/css-display-3/#block-level-box)
  - [display type](https://www.w3.org/TR/css-display-3/#display-type)
  - [formatting context](https://www.w3.org/TR/css-display-3/#formatting-context)
  - [in-flow](https://www.w3.org/TR/css-display-3/#in-flow)
  - [independent formatting context](https://www.w3.org/TR/css-display-3/#independent-formatting-context)
  - [inline box](https://www.w3.org/TR/css-display-3/#inline-box)
  - [inline formatting context](https://www.w3.org/TR/css-display-3/#inline-formatting-context)
  - [inline-block](https://www.w3.org/TR/css-display-3/#valdef-display-inline-block)
  - [inline-table](https://www.w3.org/TR/css-display-3/#valdef-display-inline-table)
  - [out-of-flow](https://www.w3.org/TR/css-display-3/#out-of-flow)
- \[CSS-MASKING-1\] defines the following terms:
  - [mask positioning area](https://www.w3.org/TR/css-masking-1/#mask-positioning-area)
- \[css-overflow-3\] defines the following terms:
  - [auto](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-auto)
  - [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow)
  - [scroll](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-scroll)
- \[css-position-3\] defines the following terms:
  - [top](https://www.w3.org/TR/css3-positioning/#propdef-top)
- \[CSS-VALUES-3\] defines the following terms:
  - [\<integer\>](https://www.w3.org/TR/css3-values/#integer-value)
- \[css-values-4\] defines the following terms:
  - [css-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords)
  - [\|](https://www.w3.org/TR/css-values-4/#comb-one)
- \[css-writing-modes-4\] defines the following terms:
  - [block flow direction](https://www.w3.org/TR/css-writing-modes-4/#block-flow-direction)
  - [block size](https://www.w3.org/TR/css-writing-modes-4/#block-size)
  - [block-axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis)
  - [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start)
  - [inline base direction](https://www.w3.org/TR/css-writing-modes-4/#inline-base-direction)
  - [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size)
  - [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr)
  - [principal writing mode](https://www.w3.org/TR/css-writing-modes-4/#principal-writing-mode)
- \[CSS21\] defines the following terms:
  - [height](https://www.w3.org/TR/CSS21/visudet.html#propdef-height)
  - [page-break-after](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-after)
  - [page-break-before](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-before)
  - [page-break-inside](https://www.w3.org/TR/CSS21/page.html#propdef-page-break-inside)
- \[CSS3-REGIONS\] defines the following terms:
  - [region chain](https://drafts.csswg.org/css-regions-1/#region-chain)
- \[CSS3-SIZING\] defines the following terms:
  - [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto)
  - [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content)
  - [min-content size](https://www.w3.org/TR/css-sizing-3/#min-content)
  - [stretch-fit size](https://www.w3.org/TR/css-sizing-3/#stretch-fit-size)
- \[CSS3-WRITING-MODES\] defines the following terms:
  - [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction)
- \[CSS3BG\] defines the following terms:
  - [border-image](https://www.w3.org/TR/css3-background/#propdef-border-image)
  - [border-radius](https://www.w3.org/TR/css3-background/#propdef-border-radius)
  - [box-shadow](https://www.w3.org/TR/css3-background/#propdef-box-shadow)
- \[CSS3PAGE\] defines the following terms:
  - [page](https://www.w3.org/TR/css3-page/#propdef-page)
  - [page progression](https://www.w3.org/TR/css3-page/#page-progression)

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-box-3"></a>\[CSS-BOX-3\]  
Elika Etemad. [CSS Box Model Module Level 3](https://www.w3.org/TR/css-box-3/). 9 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-3&#x2F;](https://www.w3.org/TR/css-box-3/)

<a id="biblio-css-cascade-4"></a>\[CSS-CASCADE-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 4](https://www.w3.org/TR/css-cascade-4/). 28 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-4&#x2F;](https://www.w3.org/TR/css-cascade-4/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 28 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-masking-1"></a>\[CSS-MASKING-1\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 26 August 2014. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
David Baron; Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 31 July 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-shapes-1"></a>\[CSS-SHAPES-1\]  
Vincent Hardy; Rossen Atanassov; Alan Stearns. [CSS Shapes Module Level 1](https://www.w3.org/TR/css-shapes-1/). 20 March 2014. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-shapes-1&#x2F;](https://www.w3.org/TR/css-shapes-1/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 14 August 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 10 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 24 May 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-css3-regions"></a>\[CSS3-REGIONS\]  
Rossen Atanassov; Alan Stearns. [CSS Regions Module Level 1](https://www.w3.org/TR/css-regions-1/). 9 October 2014. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-regions-1&#x2F;](https://www.w3.org/TR/css-regions-1/)

<a id="biblio-css3-sizing"></a>\[CSS3-SIZING\]  
Tab Atkins Jr.; Elika Etemad. [CSS Intrinsic &#x26; Extrinsic Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 4 March 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css3-transforms"></a>\[CSS3-TRANSFORMS\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 30 November 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css3-writing-modes"></a>\[CSS3-WRITING-MODES\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 24 May 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css3bg"></a>\[CSS3BG\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 17 October 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css3col"></a>\[CSS3COL\]  
Florian Rivoal; Rachel Andrew. [CSS Multi-column Layout Module Level 1](https://www.w3.org/TR/css-multicol-1/). 28 May 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-multicol-1&#x2F;](https://www.w3.org/TR/css-multicol-1/)

<a id="biblio-css3page"></a>\[CSS3PAGE\]  
Elika Etemad; Simon Sapin. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 18 October 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://tools.ietf.org/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;tools&#x2E;ietf&#x2E;org&#x2F;html&#x2F;rfc2119](https://tools.ietf.org/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-css-flexbox-1"></a>\[CSS-FLEXBOX-1\]  
Tab Atkins Jr.; et al. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 19 November 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css-grid-1"></a>\[CSS-GRID-1\]  
Tab Atkins Jr.; Elika Etemad; Rossen Atanassov. [CSS Grid Layout Module Level 1](https://www.w3.org/TR/css-grid-1/). 14 December 2017. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-1&#x2F;](https://www.w3.org/TR/css-grid-1/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Rossen Atanassov; Arron Eicholz. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 17 May 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css3text"></a>\[CSS3TEXT\]  
Elika Etemad; Koji Ishii. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 20 September 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                                                                                      | Initial | Applies to                                                                                                                             | Inh. | %ages | Media  | Anim­ation type         | Canonical order | Com­puted value    |
|---------------------|----------------------------------------------------------------------------------------------------------------------------|---------|----------------------------------------------------------------------------------------------------------------------------------------|------|-------|--------|------------------------|-----------------|-------------------|
| <strong><span><a id="ref-for-propdef-box-decoration-break①①"></a></span><a href="#propdef-box-decoration-break">box-decoration-break</a>&#xA;      </strong> | slice \| clone                                                                                                             | slice   | all elements                                                                                                                           | no   | n/a   | visual | discrete               | per grammar     | specified keyword |
| <strong><span><a id="ref-for-propdef-break-after⑦"></a></span><a href="#propdef-break-after">break-after</a>&#xA;      </strong> | auto \| avoid \| avoid-page \| page \| left \| right \| recto \| verso \| avoid-column \| column \| avoid-region \| region | auto    | block-level boxes, grid items, flex items, table row groups, table rows (but see prose)                                                | no   | n/a   | visual | discrete               | per grammar     | specified keyword |
| <strong><span><a id="ref-for-propdef-break-before⑦"></a></span><a href="#propdef-break-before">break-before</a>&#xA;      </strong> | auto \| avoid \| avoid-page \| page \| left \| right \| recto \| verso \| avoid-column \| column \| avoid-region \| region | auto    | block-level boxes, grid items, flex items, table row groups, table rows (but see prose)                                                | no   | n/a   | visual | discrete               | per grammar     | specified keyword |
| <strong><span><a id="ref-for-propdef-break-inside⑥"></a></span><a href="#propdef-break-inside">break-inside</a>&#xA;      </strong> | auto \| avoid \| avoid-page \| avoid-column \| avoid-region                                                                | auto    | all elements except inline-level boxes, internal ruby boxes, table column boxes, table column group boxes, absolutely-positioned boxes | no   | n/a   | visual | discrete               | per grammar     | specified keyword |
| <strong><span><a id="ref-for-propdef-orphans①①"></a></span><a href="#propdef-orphans">orphans</a>&#xA;      </strong> | \<integer\>                                                                                                                | 2       | block containers that establish an inline formatting context                                                                           | yes  | n/a   | visual | by computed value type | per grammar     | specified integer |
| <strong><span><a id="ref-for-propdef-widows①①"></a></span><a href="#propdef-widows">widows</a>&#xA;      </strong> | \<integer\>                                                                                                                | 2       | block containers that establish an inline formatting context                                                                           | yes  | n/a   | visual | by computed value type | per grammar     | specified integer |

