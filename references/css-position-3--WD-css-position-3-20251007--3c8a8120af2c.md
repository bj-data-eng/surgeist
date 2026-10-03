Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/2025/WD-css-position-3-20251007/).

Original copyright notice: Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Positioned Layout Module Level 3

Source snapshot: https://www.w3.org/TR/2025/WD-css-position-3-20251007/

Snapshot SHA-256: 3c8a8120af2c32d9259eb2feb80506cadffb29c716dbf571b9c1154ab06079a6

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 8 source tables are presented as readable Markdown tables or explicit labeled layouts: 5 ordinary table conversions, 1 complex-table layout, 2 already-readable tables. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Positioned Layout Module Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-relative-position"></a>

<a id="ref-for-sticky-position"></a>

<a id="ref-for-absolute-position"></a>

<a id="ref-for-fixed-position"></a>

This module contains defines coordinate-based positioning and offsetting schemes of [CSS](https://www.w3.org/TR/CSS/): [relative positioning](#relative-position), [sticky positioning](#sticky-position), [absolute positioning](#absolute-position), and [fixed positioning](#fixed-position).

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-position” in the title, like this: “\[css-position\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-position%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

<em>This section is not normative.</em>

The CSS layout algorithms, by default, size and position boxes in relation to each other so that nothing overlaps.

This specification defines several ways to violate these assumptions when needed, moving elements around in ways that can make them overlap other content:

- <a id="ref-for-relative-position①"></a>

  [Relative positioning](#relative-position), which visually shifts a box relative to its laid-out location.

- <a id="ref-for-sticky-position①"></a>

  [Sticky positioning](#sticky-position), which visually shifts a box relative to its laid-out location in order to keep it visible when a scrollable ancestor would otherwise scroll it out of sight.

- <a id="ref-for-absolute-position①"></a>

  <a id="ref-for-out-of-flow"></a>

  <a id="ref-for-containing-block"></a>

  [Absolute positioning](#absolute-position), which ignores normal layout entirely, pulling the element [out of flow](https://www.w3.org/TR/css-display-4/#out-of-flow) and positioning it relative to its [containing block](https://www.w3.org/TR/css-display-4/#containing-block) with no regard for other content.

- <a id="ref-for-fixed-position①"></a>

  [Fixed positioning](#fixed-position), which absolutely positions the box and affixes it to the viewport or page frame so that it is always visible.

<a id="ref-for-positioning-scheme"></a>

<a id="ref-for-propdef-position"></a>

<a id="ref-for-inset-properties"></a>

These [positioning schemes](#positioning-scheme), controlled by the [position](#propdef-position) property and the [inset properties](#inset-properties), are powerful but easy to misuse. With appropriate care, they allow many interesting and useful layouts that couldn’t otherwise be achieved with standard layout rules; without, they allow a page to be laid out in an unusable overlapping jumble of content.

### <a id="placement"></a>1.1.  Module Interactions

<a id="ref-for-positioning-scheme①"></a>

This module replaces and extends the [positioning scheme](#positioning-scheme) features defined in [\[CSS2\]](#biblio-css2) sections:

- [9.1.2 Containing blocks](https://www.w3.org/TR/CSS2/visuren.html#containing-block)
- [9.3 Positioning schemes](https://www.w3.org/TR/CSS2/visuren.html#positioning-scheme)
- [9.4.3 Relative positioning](https://www.w3.org/TR/CSS2/visuren.html#relative-positioning)
- [9.6 Absolute positioning](https://www.w3.org/TR/CSS2/visuren.html#absolute-positioning)
- [9.7 Relationships between display, position, and float](https://www.w3.org/TR/CSS2/visuren.html#dis-pos-flo)
- [9.8 Comparison of normal flow, floats, and absolute positioning](https://www.w3.org/TR/CSS2/visuren.html#comparison)
- [10.1 Definition of "containing block"](https://www.w3.org/TR/CSS2/visudet.html#containing-block-details)
- [10.3.7 Absolutely positioned, non-replaced elements](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-width)
- [10.3.8 Absolutely positioned, replaced elements](https://www.w3.org/TR/CSS2/visudet.html#abs-replaced-width)
- [10.6.4 Absolutely positioned, non-replaced elements](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-height)
- [10.6.5 Absolutely positioned replaced elements](https://www.w3.org/TR/CSS2/visudet.html#abs-replaced-height)

It also replaces and supersedes the inset\* property definitions in [\[CSS-LOGICAL-1\]](#biblio-css-logical-1) ([CSS Logical Properties 1 § 4.3 Flow-relative Offsets: the inset-block-start, inset-block-end, inset-inline-start, inset-inline-end properties and inset-block, inset-inline, and inset shorthands](https://www.w3.org/TR/css-logical-1/#inset-properties)).

### <a id="values"></a>1.2.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

<a id="ref-for-propdef-position①"></a>

## <a id="position-property"></a>2.  Choosing A Positioning Scheme: [position](#propdef-position) property

| Field               | Definition                                                                                                                                                                    |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-position"></a>position                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one"></a>static [\|](https://www.w3.org/TR/css-values-4/#comb-one) relative <a id="ref-for-comb-one①"></a>\| absolute <a id="ref-for-comb-one②"></a>\| sticky <a id="ref-for-comb-one③"></a>\| fixed |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | static                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | all elements except table-column-group and table-column                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                      |

<a id="ref-for-propdef-position②"></a>

<a id="ref-for-valdef-position-static"></a>

The [position](#propdef-position) property determines which of the <a id="positioning-scheme"></a>positioning schemes is used to calculate the position of a box. Values other than [static](#valdef-position-static) make the box a <a id="positioned-box"></a>positioned box, and cause it to establish an <a id="absolute-positioning-containing-block"></a>absolute positioning containing block for its descendants. Values have the following meanings:

<a id="valdef-position-static"></a>static  
<a id="ref-for-inset-properties①"></a>

<a id="ref-for-formatting-context"></a>

<a id="ref-for-positioned-box"></a>

The box is not a [positioned box](#positioned-box), and is laid out according to the rules of its parent [formatting context](https://www.w3.org/TR/css-display-4/#formatting-context). The [inset properties](#inset-properties) do not apply.

<a id="valdef-position-relative"></a>relative  
<a id="ref-for-positioning-scheme②"></a>

<a id="ref-for-scrollable-overflow-region"></a>

<a id="ref-for-valdef-position-static①"></a>

The box is laid out as for [static](#valdef-position-static), then offset from the resulting position. This offsetting is a purely visual effect, and, unless otherwise specified, does not affect the size or position of any other non-descendant box except insofar as it increases the [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region) of its ancestors. This [positioning scheme](#positioning-scheme) is called <a id="relative-position"></a>relative positioning.

<a id="valdef-position-sticky"></a>sticky  
<a id="ref-for-positioning-scheme③"></a>

<a id="ref-for-containing-block①"></a>

<a id="ref-for-valdef-top-auto"></a>

<a id="ref-for-inset-properties②"></a>

<a id="ref-for-scrollport"></a>

<a id="ref-for-scroll-container"></a>

<a id="ref-for-valdef-position-relative"></a>

Identical to [relative](#valdef-position-relative), except that its offsets are automatically adjusted in reference to the nearest ancestor [scroll container’s](https://www.w3.org/TR/css-overflow-3/#scroll-container) [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport) (as modified by the [inset properties](#inset-properties)) in whichever axes the <a id="ref-for-inset-properties③"></a>inset properties are not both [auto](#valdef-top-auto), to try to keep the box in view within its [containing block](https://www.w3.org/TR/css-display-4/#containing-block) as the user scrolls. This [positioning scheme](#positioning-scheme) is called <a id="sticky-position"></a>sticky positioning.

<a id="valdef-position-absolute"></a>absolute  
<a id="ref-for-formatting-context①"></a>

<a id="ref-for-out-of-flow①"></a>

The box is taken [out of flow](https://www.w3.org/TR/css-display-4/#out-of-flow) such that it has no impact on the size or position of its siblings and ancestors, and does not participate in its parent’s [formatting context](https://www.w3.org/TR/css-display-4/#formatting-context).

<a id="ref-for-absolute-positioning-containing-block"></a>

<a id="ref-for-inset-properties④"></a>

<a id="ref-for-in-flow"></a>

<a id="ref-for-absolute-position②"></a>

<a id="ref-for-scrollable-overflow-region①"></a>

<a id="ref-for-containing-block②"></a>

<a id="ref-for-positioning-scheme④"></a>

Instead, the box is positioned and sized solely in reference to its [absolute positioning containing block](#absolute-positioning-containing-block), as modified by the box’s [inset properties](#inset-properties), see [§ 4 Absolute Positioning Layout Model](#abspos-layout). It can overlap [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) content or other [absolutely positioned](#absolute-position) elements, and is included in the [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region) of the box that generates is [containing block](https://www.w3.org/TR/css-display-4/#containing-block). This [positioning scheme](#positioning-scheme) is called <a id="absolute-position"></a>absolute positioning.

<a id="valdef-position-fixed"></a>fixed  
<a id="ref-for-absolute-position③"></a>

<a id="ref-for-positioning-scheme⑤"></a>

<a id="ref-for-paged-media"></a>

<a id="ref-for-page-area"></a>

<a id="ref-for-continuous-media"></a>

<a id="ref-for-x1"></a>

<a id="ref-for-fixed-positioning-containing-block"></a>

<a id="ref-for-valdef-position-absolute"></a>

Same as [absolute](#valdef-position-absolute), except the box is positioned and sized relative to a [fixed positioning containing block](#fixed-positioning-containing-block) (usually the [viewport](https://www.w3.org/TR/CSS2/visuren.html#x1) in [continuous media](https://www.w3.org/TR/mediaqueries-5/#continuous-media), or the [page area](https://www.w3.org/TR/css-page-3/#page-area) in [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media)). The box’s position is fixed with respect to this reference rectangle: when attached to the <a id="ref-for-x1①"></a>viewport it does not move when the document is scrolled, and when attached to the <a id="ref-for-page-area①"></a>page area is replicated on every page when the document is paginated. This [positioning scheme](#positioning-scheme) is called <a id="fixed-position"></a>fixed positioning and is considered a subset of [absolute positioning](#absolute-position).

<a id="ref-for-valdef-position-fixed"></a>

<a id="ref-for-x1②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d41e8cfe"></a> Authors may wish to specify [fixed](#valdef-position-fixed) in a media-dependent way. For instance, an author may want a box to remain at the top of the [viewport](https://www.w3.org/TR/CSS2/visuren.html#x1) on the screen, but not at the top of each printed page. The two specifications may be separated by using an ['@media'](https://www.w3.org/TR/CSS2/media.html#at-media-rule) rule, as in:
> ```text
> @media screen {
>     h1#first { position: fixed }
> }
> @media print {
>     h1#first { position: static }
> }
> ```
<a id="ref-for-propdef-position③"></a>

<a id="ref-for-valdef-position-absolute①"></a>

<a id="ref-for-valdef-position-fixed①"></a>

<a id="ref-for-blockify"></a>

<a id="ref-for-propdef-float"></a>

<a id="ref-for-valdef-float-none"></a>

<a id="ref-for-establish-an-independent-formatting-context"></a>

A [position](#propdef-position) value of [absolute](#valdef-position-absolute) or [fixed](#valdef-position-fixed) [blockifies](https://www.w3.org/TR/css-display-4/#blockify) the box, causes [float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float) to compute to [none](https://drafts.csswg.org/css2/#valdef-float-none), and forces the box to [establish an independent formatting context](https://www.w3.org/TR/css-display-4/#establish-an-independent-formatting-context).

### <a id="def-cb"></a>2.1.  Containing Blocks of Positioned Boxes

<a id="ref-for-containing-block③"></a>

<a id="ref-for-valdef-position-static②"></a>

<a id="ref-for-valdef-position-relative①"></a>

<a id="ref-for-valdef-position-sticky"></a>

<a id="ref-for-box"></a>

<a id="ref-for-formatting-context②"></a>

<a id="ref-for-valdef-position-fixed②"></a>

<a id="ref-for-valdef-position-absolute②"></a>

The [containing block](https://www.w3.org/TR/css-display-4/#containing-block) of a [static](#valdef-position-static), [relative](#valdef-position-relative), or [sticky](#valdef-position-sticky) [box](https://www.w3.org/TR/css-display-4/#box) is as defined by its [formatting context](https://www.w3.org/TR/css-display-4/#formatting-context). For [fixed](#valdef-position-fixed) and [absolute](#valdef-position-absolute) boxes, it is defined as follows:

<a id="ref-for-propdef-position④"></a>

<a id="absolute-cb"></a>If the box has [position: absolute](#propdef-position):

<a id="ref-for-absolute-positioning-containing-block①"></a>

<a id="ref-for-containing-block④"></a>

The [containing block](https://www.w3.org/TR/css-display-4/#containing-block) is established by the nearest ancestor box that establishes an [absolute positioning containing block](#absolute-positioning-containing-block), in the following way:

<a id="ref-for-inline-box"></a>

If the ancestor is not an [inline box](https://www.w3.org/TR/css-display-4/#inline-box),

<a id="ref-for-padding-edge"></a>

<a id="ref-for-containing-block⑤"></a>

the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) is formed by the [padding edge](https://www.w3.org/TR/css-box-4/#padding-edge) of the ancestor, unless otherwise specified (for example, see [CSS Grid Layout 1 § 9.1 With a Grid Container as Containing Block](https://www.w3.org/TR/css-grid-1/#abspos-items)).

<a id="ref-for-writing-mode"></a>

<a id="ref-for-inline-box①"></a>

If the ancestor is an [inline box](https://www.w3.org/TR/css-display-4/#inline-box), using the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of that box,

<a id="ref-for-end"></a>

<a id="ref-for-box-fragment"></a>

<a id="ref-for-content-edge"></a>

<a id="ref-for-start"></a>

<a id="ref-for-containing-block⑥"></a>

the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) is formed by forming a rectangle from the [start](https://www.w3.org/TR/css-writing-modes-4/#start)-most [content edges](https://www.w3.org/TR/css-box-4/#content-edge) (in both axes) of the first [box fragment](https://www.w3.org/TR/css-break-4/#box-fragment) of the ancestor, and the [end](https://www.w3.org/TR/css-writing-modes-4/#end)-most <a id="ref-for-content-edge①"></a>content edges of the <a id="ref-for-end①"></a>end-most <a id="ref-for-box-fragment①"></a>box fragment(s) of the ancestor in each axis. If there are multiple fragments on the same line (e.g. due to [bidi reordering](https://www.w3.org/TR/css-writing-modes-3/#bidi-box-model)), take the <a id="ref-for-start①"></a>start-most fragment as the first fragment.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d9a49098"></a> What is a useful containing block to form when the box is fragmented across multiple lines? [\[Issue \#8284\]](https://github.com/w3c/csswg-drafts/issues/8284)

<a id="ref-for-containing-block⑦"></a>

<a id="ref-for-inline-box②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [containing block](https://www.w3.org/TR/css-display-4/#containing-block) formed by a fragmented [inline box](https://www.w3.org/TR/css-display-4/#inline-box) was undefined in [\[CSS2\]](#biblio-css2).

<a id="ref-for-absolute-positioning-containing-block②"></a>

<a id="ref-for-initial-containing-block"></a>

If no ancestor establishes one, the [absolute positioning containing block](#absolute-positioning-containing-block) is the [initial containing block](https://www.w3.org/TR/css-display-4/#initial-containing-block).

<a id="ref-for-absolute-positioning-containing-block③"></a>

<a id="ref-for-propdef-position⑤"></a>

<a id="ref-for-propdef-transform"></a>

<a id="ref-for-propdef-will-change"></a>

<a id="ref-for-propdef-contain"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Properties that can cause a box to establish an [absolute positioning containing block](#absolute-positioning-containing-block) include [position](#propdef-position), [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform), [will-change](https://www.w3.org/TR/css-will-change-1/#propdef-will-change), [contain](https://www.w3.org/TR/css-contain-2/#propdef-contain)…

<a id="ref-for-propdef-position⑥"></a>

<a id="fixed-cb"></a>If the box has [position: fixed](#propdef-position):

<a id="ref-for-absolute-positioning-containing-block④"></a>

<a id="ref-for-containing-block⑧"></a>

The [containing block](https://www.w3.org/TR/css-display-4/#containing-block) is established by the nearest ancestor box that establishes an <a id="fixed-positioning-containing-block"></a>fixed positioning containing block, with the bounds of the <a id="ref-for-containing-block⑨"></a>containing block determined identically to the [absolute positioning containing block](#absolute-positioning-containing-block).

<a id="ref-for-fixed-positioning-containing-block①"></a>

<a id="ref-for-propdef-transform①"></a>

<a id="ref-for-propdef-will-change①"></a>

<a id="ref-for-propdef-contain①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Properties that can cause a box to establish a [fixed positioning containing block](#fixed-positioning-containing-block) include [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform), [will-change](https://www.w3.org/TR/css-will-change-1/#propdef-will-change), [contain](https://www.w3.org/TR/css-contain-2/#propdef-contain)…

<a id="ref-for-fixed-positioning-containing-block②"></a>

If no ancestor establishes one, the box’s [fixed positioning containing block](#fixed-positioning-containing-block) is the <a id="initial-fixed-containing-block"></a>initial fixed containing block:

- <a id="ref-for-continuous-media①"></a>

  <a id="ref-for-layout-viewport"></a>

  <a id="ref-for-dynamic-viewport-size"></a>

  <a id="ref-for-fixed-position②"></a>

  in [continuous media](https://www.w3.org/TR/mediaqueries-5/#continuous-media), the [layout viewport](https://www.w3.org/TR/cssom-view-1/#layout-viewport) (whose size matches the [dynamic viewport size](https://www.w3.org/TR/css-values-4/#dynamic-viewport-size)); as a result, [fixed](#fixed-position) boxes do not move when the document is scrolled.

  <a id="ref-for-propdef-background-attachment"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: In this respect, they are similar to [fixed background images](https://www.w3.org/TR/css-backgrounds-3/#background-attachment) ([background-attachment: fixed](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-attachment)).

- <a id="ref-for-paged-media①"></a>

  <a id="ref-for-page-area②"></a>

  <a id="ref-for-fixed-position③"></a>

  <a id="ref-for-box①"></a>

  <a id="ref-for-x1③"></a>

  in [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media), the [page area](https://www.w3.org/TR/css-page-3/#page-area) of each page; [fixed positioned](#fixed-position) [boxes](https://www.w3.org/TR/css-display-4/#box) are thus replicated on every page. (They are fixed with respect to the page box only, and are not affected by being seen through a [viewport](https://www.w3.org/TR/CSS2/visuren.html#x1); as in the case of print preview, for example.)

<a id="ref-for-fixed-position④"></a>

<a id="ref-for-layout-viewport①"></a>

<a id="ref-for-page-area③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As a result, parts of [fixed-positioned boxes](#fixed-position) that extend outside the [layout viewport](https://www.w3.org/TR/cssom-view-1/#layout-viewport)/[page area](https://www.w3.org/TR/css-page-3/#page-area) cannot be scrolled to and will not print.

<a id="ref-for-initial-fixed-containing-block"></a>

<a id="ref-for-initial-containing-block①"></a>

<a id="ref-for-containing-block-chain"></a>

The [initial fixed containing block](#initial-fixed-containing-block) is the parent of the [initial containing block](https://www.w3.org/TR/css-display-4/#initial-containing-block) in the [containing block chain](https://www.w3.org/TR/css-display-4/#containing-block-chain).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7f074eae"></a>
>
> With no positioning, the containing blocks (C.B.) in the following document:
>
> ```text
> <!DOCTYPE html>
> <html>
>     <head>
>         <title>Illustration of containing blocks</title>
>     </head>
>     <body id="body">
>         <div id="div1">
>         <p id="p1">This is text in the first paragraph...</p>
>         <p id="p2">This is text <em id="em1"> in the
>         <strong id="strong1">second</strong> paragraph.</em></p>
>         </div>
>     </body>
> </html>
> ```
>
> are established as follows:
>
> |                      |                             |
> |----------------------|-----------------------------|
> | For box generated by | C.B. is established by      |
> | html                 | initial C.B. (UA-dependent) |
> | body                 | html                        |
> | div1                 | body                        |
> | p1                   | div1                        |
> | p2                   | div1                        |
> | em1                  | p2                          |
> | strong1              | p2                          |
>
> If we position "div1":
>
> ```text
> #div1 { position: absolute; left: 50px; top: 50px }
> ```
>
> its containing block is no longer "body"; it becomes the initial containing block (since there are no other positioned ancestor boxes).
>
> If we position "em1" as well:
>
> ```text
> #div1 { position: absolute; left: 50px; top: 50px }
> #em1  { position: absolute; left: 100px; top: 100px }
> ```
>
> the table of containing blocks becomes:
>
> |                      |                             |
> |----------------------|-----------------------------|
> | For box generated by | C.B. is established by      |
> | html                 | initial C.B. (UA-dependent) |
> | body                 | html                        |
> | div1                 | initial C.B.                |
> | p1                   | div1                        |
> | p2                   | div1                        |
> | em1                  | div1                        |
> | strong1              | em1                         |
>
> By positioning "em1", its containing block becomes the nearest positioned ancestor box (i.e., that generated by "div1").

#### <a id="original-cb"></a>2.1.1.  Further Adjustments to the Containing Block

<a id="ref-for-containing-block①⓪"></a>

<a id="ref-for-absolute-position④"></a>

Some features can alter the effective [containing block](https://www.w3.org/TR/css-display-4/#containing-block) rectangle of [absolutely positioned](#absolute-position) boxes. These are applied in the following order, with earlier steps modifying the <a id="ref-for-containing-block①①"></a>containing block that later steps see:

1.  <a id="ref-for-grid-placement-property"></a>

    <a id="ref-for-absolute-position⑤"></a>

    <a id="ref-for-containing-block①②"></a>

    <a id="ref-for-grid-container"></a>

    <a id="ref-for-grid-area"></a>

    The [grid-placement properties](https://www.w3.org/TR/css-grid-2/#grid-placement-property) on an [absolutely positioned box](#absolute-position) whose [containing block](https://www.w3.org/TR/css-display-4/#containing-block) is generated by a [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) can change the <a id="ref-for-containing-block①③"></a>containing block rectangle to a specified [grid area](https://www.w3.org/TR/css-grid-2/#grid-area). See [CSS Grid Layout 1 § 9.1 With a Grid Container as Containing Block](https://www.w3.org/TR/css-grid-1/#abspos-items).

2.  <a id="ref-for-propdef-position-area"></a>

    <a id="ref-for-propdef-position-try"></a>

    <a id="ref-for-containing-block①④"></a>

    <a id="ref-for-position-area-grid"></a>

    The [position-area](https://www.w3.org/TR/css-anchor-position-1/#propdef-position-area) and [position-try](https://www.w3.org/TR/css-anchor-position-1/#propdef-position-try) properties can change the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) rectangle to a specified area of an [position-area grid](https://www.w3.org/TR/css-anchor-position-1/#position-area-grid). See [CSS Anchor Positioning § 3.1 The position-area Property](https://www.w3.org/TR/css-anchor-position-1/#position-area).

<a id="ref-for-containing-block①⑤"></a>

The element’s <a id="original-containing-block"></a>original containing block is its [containing block](https://www.w3.org/TR/css-display-4/#containing-block) before applying any of these effects.

### <a id="stacking"></a>2.2.  Painting Order and Stacking Contexts

<a id="ref-for-propdef-z-index"></a>

<a id="ref-for-positioned-box①"></a>

<a id="ref-for-valdef-z-index-auto"></a>

The [z-index](https://www.w3.org/TR/CSS2/visuren.html#propdef-z-index) property applies to all [positioned boxes](#positioned-box). When <a id="ref-for-propdef-z-index①"></a>z-index is [auto](https://drafts.csswg.org/css2/#valdef-z-index-auto):

- <a id="ref-for-fixed-position⑤"></a>

  <a id="ref-for-sticky-position②"></a>

  <a id="ref-for-positioned-box②"></a>

  <a id="ref-for-x43"></a>

  [Fixed](#fixed-position) and [sticky](#sticky-position) [positioned boxes](#positioned-box) nonetheless form a [stacking context](https://www.w3.org/TR/CSS2/visuren.html#x43).

- <a id="ref-for-relative-position②"></a>

  <a id="ref-for-absolute-position⑥"></a>

  <a id="ref-for-positioned-box③"></a>

  <a id="ref-for-x43①"></a>

  [Relative](#relative-position) and [absolute](#absolute-position) [positioned boxes](#positioned-box) do not form a [stacking context](https://www.w3.org/TR/CSS2/visuren.html#x43), but are painted as if those elements did generated new <a id="ref-for-x43②"></a>stacking contexts, except that their <a id="ref-for-positioned-box④"></a>positioned descendants and any would-be child <a id="ref-for-x43③"></a>stacking contexts take part in the current <a id="ref-for-x43④"></a>stacking context.

<a id="ref-for-root-element"></a>

<a id="ref-for-x43⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [root element](https://www.w3.org/TR/css-display-4/#root-element) always forms a [stacking context](https://www.w3.org/TR/CSS2/visuren.html#x43) regardless.

<a id="ref-for-propdef-z-index②"></a>

<a id="ref-for-x43⑥"></a>

See [CSS2 § 9.9 Layered presentation](https://www.w3.org/TR/CSS2/visuren.html#layers) and [Appendix E:  Elaborate description of Stacking Contexts](https://www.w3.org/TR/CSS2/zindex.html) for details about [z-index](https://www.w3.org/TR/CSS2/visuren.html#propdef-z-index), [stacking contexts](https://www.w3.org/TR/CSS2/visuren.html#x43), and painting order.

## <a id="coords"></a>3.  Positioning Coordinates

<a id="ref-for-positioned-box⑤"></a>

<a id="ref-for-physical"></a>

<a id="ref-for-inset-properties⑤"></a>

<a id="ref-for-propdef-top"></a>

<a id="ref-for-propdef-right"></a>

<a id="ref-for-propdef-bottom"></a>

<a id="ref-for-propdef-left"></a>

<a id="ref-for-flow-relative"></a>

<a id="ref-for-propdef-inset-block-start"></a>

<a id="ref-for-propdef-inset-inline-start"></a>

<a id="ref-for-propdef-inset-block-end"></a>

<a id="ref-for-propdef-inset-inline-end"></a>

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-propdef-inset-block"></a>

<a id="ref-for-propdef-inset-inline"></a>

<a id="ref-for-propdef-inset"></a>

The precise location of a [positioned box](#positioned-box) is controlled by the <a id="inset-properties"></a>inset properties: the [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) [inset properties](#inset-properties) [top](#propdef-top), [right](#propdef-right), [bottom](#propdef-bottom), [left](#propdef-left); the [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) <a id="ref-for-inset-properties⑥"></a>inset properties [inset-block-start](#propdef-inset-block-start), [inset-inline-start](#propdef-inset-inline-start), [inset-block-end](#propdef-inset-block-end), and [inset-inline-end](#propdef-inset-inline-end); and their [shorthands](https://www.w3.org/TR/css-cascade-5/#shorthand-property), [inset-block](#propdef-inset-block), [inset-inline](#propdef-inset-inline), and [inset](#propdef-inset).

<a id="ref-for-inset-properties⑦"></a>

<a id="ref-for-positioning-scheme⑥"></a>

The interpretation of these [inset properties](#inset-properties) varies by [positioning scheme](#positioning-scheme):

- <a id="ref-for-absolute-position⑦"></a>

  for [absolute positioning](#absolute-position), they represent insets from the containing block.

- <a id="ref-for-relative-position③"></a>

  for [relative positioning](#relative-position), they represent insets from the box’s original margin edge.

- <a id="ref-for-sticky-position③"></a>

  <a id="ref-for-scrollport①"></a>

  for [sticky positioning](#sticky-position), they represent insets from the [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport) edge.

<a id="ref-for-propdef-top①"></a>

<a id="ref-for-propdef-right①"></a>

<a id="ref-for-propdef-bottom①"></a>

<a id="ref-for-propdef-left①"></a>

<a id="ref-for-propdef-inset-block-start①"></a>

<a id="ref-for-propdef-inset-inline-start①"></a>

<a id="ref-for-propdef-inset-block-end①"></a>

<a id="ref-for-propdef-inset-inline-end①"></a>

### <a id="insets"></a>3.1.  Box Insets: the [top](#propdef-top), [right](#propdef-right), [bottom](#propdef-bottom), [left](#propdef-left), [inset-block-start](#propdef-inset-block-start), [inset-inline-start](#propdef-inset-inline-start), [inset-block-end](#propdef-inset-block-end), and [inset-inline-end](#propdef-inset-inline-end) properties 

| Field               | Definition                                                                                                                                                                                                                                                 |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-top"></a>top, <a id="propdef-right"></a>right, <a id="propdef-bottom"></a>bottom, <a id="propdef-left"></a>left, <a id="propdef-inset-block-start"></a>inset-block-start, <a id="propdef-inset-inline-start"></a>inset-inline-start, <a id="propdef-inset-block-end"></a>inset-block-end, <a id="propdef-inset-inline-end"></a>inset-inline-end |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage"></a><a id="ref-for-comb-one④"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | positioned elements                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-containing-block①⑥"></a>refer to size of [containing block](https://www.w3.org/TR/css-display-4/#containing-block); see prose                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①"></a><a id="ref-for-valdef-top-auto①"></a>the keyword [auto](#valdef-top-auto) or a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                                                                                                     |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-inset①"></a>[inset](#propdef-inset)                                                                                                                                                                                                                 |

<a id="ref-for-inset-properties⑧"></a>

<a id="ref-for-writing-mode①"></a>

<a id="ref-for-propdef-top②"></a>

<a id="ref-for-physical①"></a>

<a id="ref-for-flow-relative①"></a>

These [inset properties](#inset-properties) represent an inward “inset” on the corresponding side of the box (with respect to the box’s own [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode); see [CSS Writing Modes 3 § 6 Abstract Box Terminology](https://www.w3.org/TR/css-writing-modes-3/#abstract-box)). For example, [top](#propdef-top) represents a downward inset of the top edge. The [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) and [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) properties interact as defined in [\[CSS-LOGICAL-1\]](#biblio-css-logical-1). Values have the following meanings:

<a id="valdef-top-length"></a>\<length\>  
The inset is a fixed distance from the reference edge. Negative values are allowed.

<a id="valdef-top-percentage"></a>\<percentage\>  
<a id="ref-for-scrollport②"></a>

<a id="ref-for-sticky-position④"></a>

<a id="ref-for-propdef-bottom②"></a>

<a id="ref-for-propdef-top③"></a>

<a id="ref-for-propdef-right②"></a>

<a id="ref-for-propdef-left②"></a>

<a id="ref-for-containing-block①⑦"></a>

The inset is a percentage relative to the [containing block](https://www.w3.org/TR/css-display-4/#containing-block)’s size in the corresponding axis (e.g. width for [left](#propdef-left) or [right](#propdef-right), height for [top](#propdef-top) and [bottom](#propdef-bottom)). For [sticky positioned](#sticky-position) boxes, the inset is instead relative to the relevant [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport)’s size. Negative values are allowed.

<a id="valdef-top-auto"></a>auto  
<a id="ref-for-positioning-scheme⑦"></a>

Represents an unconstrained inset; the exact meaning depends on the [positioning scheme](#positioning-scheme).

<a id="ref-for-fixed-position⑥"></a>

<a id="ref-for-x1④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For [fixed positioned](#fixed-position) elements, using large values or negative values can easily move elements outside the [viewport](https://www.w3.org/TR/CSS2/visuren.html#x1) and make the contents unreachable through scrolling or other means.

<a id="ref-for-propdef-inset-block①"></a>

<a id="ref-for-propdef-inset-inline①"></a>

<a id="ref-for-propdef-inset②"></a>

### <a id="inset-shorthands"></a>3.2.  Box Insets Shorthands: the [inset-block](#propdef-inset-block), [inset-inline](#propdef-inset-inline), and [inset](#propdef-inset) properties

| Field               | Definition                                                                                                                 |
|---------------------|----------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-inset-block"></a>inset-block, <a id="propdef-inset-inline"></a>inset-inline                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range"></a><a id="ref-for-propdef-top④"></a>[\<'top'\>](#propdef-top)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | positioned elements                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                     |

<a id="ref-for-propdef-inset-block②"></a>

<a id="ref-for-propdef-inset-inline②"></a>

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-propdef-inset-block-start②"></a>

<a id="ref-for-propdef-inset-block-end②"></a>

<a id="ref-for-propdef-inset-inline-start②"></a>

<a id="ref-for-propdef-inset-inline-end②"></a>

<a id="ref-for-start②"></a>

<a id="ref-for-end②"></a>

The [inset-block](#propdef-inset-block) and [inset-inline](#propdef-inset-inline) properties are [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for setting [inset-block-start](#propdef-inset-block-start) + [inset-block-end](#propdef-inset-block-end) or [inset-inline-start](#propdef-inset-inline-start) + [inset-inline-end](#propdef-inset-inline-end), respectively, in a single declaration. The first component value sets the [start](https://www.w3.org/TR/css-writing-modes-4/#start) side, the second sets the [end](https://www.w3.org/TR/css-writing-modes-4/#end); if omitted, the second value defaults to the first.

| Field               | Definition                                                                                                                 |
|---------------------|----------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-inset"></a>inset                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range①"></a><a id="ref-for-propdef-top⑤"></a>[\<'top'\>](#propdef-top)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | positioned elements                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                     |

<a id="ref-for-propdef-inset③"></a>

<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-inset-properties⑨"></a>

<a id="ref-for-propdef-margin"></a>

The [inset](#propdef-inset) property is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) that sets all of the [inset properties](#inset-properties) in a single declaration, assigning values to the longhands representing each side exactly as the [margin](https://www.w3.org/TR/css-box-4/#propdef-margin) property does for its longhands.

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="ref-for-propdef-inset④"></a>
>
> <a id="ref-for-physical②"></a>
>
> <a id="ref-for-longhand"></a>
>
> <a id="ref-for-propdef-top⑥"></a>
>
> <a id="ref-for-propdef-right③"></a>
>
> <a id="ref-for-propdef-bottom③"></a>
>
> <a id="ref-for-propdef-left③"></a>
>
> <a id="ref-for-propdef-margin①"></a>
>
> By default, the [inset](#propdef-inset) property values are assigned to the corresponding <em><a href="https://www.w3.org/TR/css-writing-modes-4/#physical">physical</a></em> [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand)—​[top](#propdef-top), [right](#propdef-right), [bottom](#propdef-bottom), and [left](#propdef-left)—​which for historical reasons do not have an inset- prefix. This matches the behavior of other "4 values assigned to sides" properties, such as [margin](https://www.w3.org/TR/css-box-4/#propdef-margin).
>
> <a id="ref-for-flow-relative②"></a>
>
> <a id="ref-for-longhand①"></a>
>
> Allowing properties such as this to resolve to the [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) [longhands](https://www.w3.org/TR/css-cascade-5/#longhand) is under discussion in [\[CSS-LOGICAL-1\]](#biblio-css-logical-1).
>
> <a id="ref-for-propdef-inset⑤"></a>
>
> Yes, we understand it’s a little confusing that [inset](#propdef-inset) doesn’t expand to any inset-\* properties.

### <a id="relpos-insets"></a>3.3. <a id="relpos"></a> Relative Positioning

<a id="ref-for-relative-position④"></a>

<a id="ref-for-inset-properties①⓪"></a>

<a id="ref-for-propdef-left④"></a>

<a id="ref-for-propdef-right④"></a>

<a id="ref-for-used-value"></a>

For a [relatively positioned](#relative-position) box, the [inset properties](#inset-properties) move the box inward from the respective edge, without changing its size. [left](#propdef-left) moves the box to the right, [right](#propdef-right) moves it to the left, etc. Since boxes are not split or stretched as a result of <a id="ref-for-relative-position⑤"></a>relative positioning opposing [used values](https://www.w3.org/TR/css-cascade-5/#used-value) in a given axis must be negations of each other:

- <a id="ref-for-used-value①"></a>

  <a id="ref-for-initial-value"></a>

  <a id="ref-for-valdef-top-auto②"></a>

  <a id="ref-for-inset-properties①①"></a>

  If opposing [inset properties](#inset-properties) in an axis both compute to [auto](#valdef-top-auto) (their [initial values](https://www.w3.org/TR/css-cascade-5/#initial-value)), their [used values](https://www.w3.org/TR/css-cascade-5/#used-value) are zero (i.e., the boxes stay in their original position in that axis).

- <a id="ref-for-used-value②"></a>

  <a id="ref-for-valdef-top-auto③"></a>

  If only one is [auto](#valdef-top-auto), its [used value](https://www.w3.org/TR/css-cascade-5/#used-value) becomes the negation of the other, and the box is shifted by the specified amount.

- <a id="ref-for-start③"></a>

  <a id="ref-for-used-value③"></a>

  <a id="ref-for-end③"></a>

  <a id="ref-for-computed-value"></a>

  <a id="ref-for-containing-block①⑧"></a>

  <a id="ref-for-writing-mode②"></a>

  <a id="ref-for-valdef-top-auto④"></a>

  If neither is [auto](#valdef-top-auto), the position is over-constrained; (with respect to the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of its [containing block](https://www.w3.org/TR/css-display-4/#containing-block)) the [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [end](https://www.w3.org/TR/css-writing-modes-4/#end) side value is ignored, and its [used value](https://www.w3.org/TR/css-cascade-5/#used-value) becomes the negation of the [start](https://www.w3.org/TR/css-writing-modes-4/#start) side.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bf44fe11"></a>
>
> The following three rules are equivalent, and shift the box 1em to the left:
>
> ```text
> div.a8 { position: relative; direction: ltr; left: -1em; right: auto }
> div.a8 { position: relative; direction: ltr; left: auto; right: 1em }
> div.a8 { position: relative; direction: ltr; left: -1em; right: 5em }
> ```
<a id="ref-for-valdef-display-table-row-group"></a>

<a id="ref-for-valdef-display-table-header-group"></a>

<a id="ref-for-valdef-display-table-footer-group"></a>

<a id="ref-for-valdef-display-table-row"></a>

<a id="ref-for-box②"></a>

<a id="ref-for-valdef-display-table-cell"></a>

If specified on a [table-row-group](https://www.w3.org/TR/css-display-4/#valdef-display-table-row-group), [table-header-group](https://www.w3.org/TR/css-display-4/#valdef-display-table-header-group), [table-footer-group](https://www.w3.org/TR/css-display-4/#valdef-display-table-footer-group), or [table-row](https://www.w3.org/TR/css-display-4/#valdef-display-table-row) [box](https://www.w3.org/TR/css-display-4/#box) the shift affects all the contents of the box, including all [table cells](https://www.w3.org/TR/css-display-4/#valdef-display-table-cell) that originate in the affected row, but not those that don’t.

<a id="ref-for-propdef-position⑦"></a>

<a id="ref-for-valdef-display-table-column-group"></a>

<a id="ref-for-valdef-display-table-column"></a>

<a id="ref-for-relative-position⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since [position](#propdef-position) does not apply to [table-column-group](https://www.w3.org/TR/css-display-4/#valdef-display-table-column-group) or [table-column](https://www.w3.org/TR/css-display-4/#valdef-display-table-column) boxes, they are not affected by [relative positioning](#relative-position).

### <a id="stickypos-insets"></a>3.4. <a id="sticky-pos"></a> Sticky positioning

<a id="ref-for-sticky-position⑤"></a>

<a id="ref-for-relative-position⑦"></a>

<a id="ref-for-nearest-scrollport"></a>

[Sticky positioning](#sticky-position) is similar to [relative positioning](#relative-position) except the offsets are automatically calculated in reference to the [nearest scrollport](https://www.w3.org/TR/css-overflow-3/#nearest-scrollport).

<a id="ref-for-sticky-position⑥"></a>

<a id="ref-for-box③"></a>

<a id="ref-for-inset-properties①②"></a>

<a id="ref-for-nearest-scrollport①"></a>

<a id="ref-for-valdef-top-auto⑤"></a>

<a id="ref-for-sticky-view-rectangle"></a>

<a id="ref-for-border-box"></a>

<a id="ref-for-end④"></a>

<a id="ref-for-writing-mode③"></a>

<a id="ref-for-containing-block①⑨"></a>

For a [sticky positioned](#sticky-position) [box](https://www.w3.org/TR/css-display-4/#box), the [inset properties](#inset-properties) represent insets from the respective edges of the [nearest scrollport](https://www.w3.org/TR/css-overflow-3/#nearest-scrollport), defining the <a id="sticky-view-rectangle"></a>sticky view rectangle used to constrain the box’s position. (For this purpose an [auto](#valdef-top-auto) value represents a zero inset.) If this results in a [sticky view rectangle](#sticky-view-rectangle) size in any axis less than the size of the [border box](https://www.w3.org/TR/css-box-4/#border-box) of the <a id="ref-for-sticky-position⑦"></a>sticky box in that axis, then the effective [end](https://www.w3.org/TR/css-writing-modes-4/#end)-edge inset in the affected axis is reduced (possibly becoming negative) to bring the <a id="ref-for-sticky-view-rectangle①"></a>sticky view rectangle’s size up to the size of the <a id="ref-for-border-box①"></a>border box in that axis (where <a id="ref-for-end⑤"></a>end is interpreted relative to the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the [containing block](https://www.w3.org/TR/css-display-4/#containing-block)).

<a id="ref-for-nearest-scrollport②"></a>

<a id="ref-for-sticky-position⑧"></a>

<a id="ref-for-border-box②"></a>

<a id="ref-for-propdef-top⑦"></a>

<a id="ref-for-sticky-view-rectangle②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1c9f6a33"></a> For example, if the [nearest scrollport](https://www.w3.org/TR/css-overflow-3/#nearest-scrollport) is 300px tall, the [sticky](#sticky-position) box’s [border box](https://www.w3.org/TR/css-box-4/#border-box) is 200px tall, and it has [top: 20px](#propdef-top), then the top-edge inset of the <a id="ref-for-nearest-scrollport③"></a>nearest scrollport is 20px, and the bottom-edge inset is 0px, yielding a [sticky view rectangle](#sticky-view-rectangle) that is 280px tall.
>
> <a id="ref-for-nearest-scrollport④"></a>
>
> <a id="ref-for-sticky-view-rectangle③"></a>
>
> <a id="ref-for-margin-box"></a>
>
> <a id="ref-for-sticky-position⑨"></a>
>
> But if the [nearest scrollport](https://www.w3.org/TR/css-overflow-3/#nearest-scrollport) were only 100px tall, then the effective bottom-edge inset becomes -120px, resulting in a [sticky view rectangle](#sticky-view-rectangle) that’s 200px tall, enough to fully contain the [margin box](https://www.w3.org/TR/css-box-4/#margin-box) of the [sticky](#sticky-position) box.

<a id="ref-for-inset-properties①③"></a>

<a id="ref-for-valdef-top-auto⑥"></a>

<a id="ref-for-border-edge"></a>

<a id="ref-for-sticky-view-rectangle④"></a>

<a id="ref-for-position-box"></a>

<a id="ref-for-containing-block②⓪"></a>

<a id="ref-for-margin-box①"></a>

<a id="ref-for-margin-edge"></a>

<a id="ref-for-margin"></a>

For each side of the box, if the corresponding [inset property](#inset-properties) is not [auto](#valdef-top-auto), and the corresponding [border edge](https://www.w3.org/TR/css-box-4/#border-edge) of the box would be outside the corresponding edge of the [sticky view rectangle](#sticky-view-rectangle), then the box must be visually shifted ([as for relative positioning](#relpos-insets)) to be inward of that <a id="ref-for-sticky-view-rectangle⑤"></a>sticky view rectangle edge, insofar as it can while its [position box](#position-box) remains contained within its [containing block](https://www.w3.org/TR/css-display-4/#containing-block). The <a id="position-box"></a>position box is its [margin box](https://www.w3.org/TR/css-box-4/#margin-box), except that for any side for which the distance between its [margin edge](https://www.w3.org/TR/css-box-4/#margin-edge) and the corresponding edge of its <a id="ref-for-containing-block②①"></a>containing block is less than its corresponding [margin](https://www.w3.org/TR/css-box-4/#margin), that distance is used in place of that <a id="ref-for-margin①"></a>margin.

<a id="ref-for-valdef-top-auto⑦"></a>

<a id="ref-for-propdef-top⑧"></a>

<a id="ref-for-propdef-bottom④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A sticky positioned element with a non-[auto](#valdef-top-auto) [top](#propdef-top) value and an <a id="ref-for-valdef-top-auto⑧"></a>auto [bottom](#propdef-bottom) value will only ever be pushed down by sticky positioning; it will never be offset upwards.

<a id="ref-for-sticky-position①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Multiple [sticky positioned](#sticky-position) boxes in the same container are offset independently, and therefore might overlap.

#### <a id="stickypos-scroll"></a>3.4.1.  Scroll Position of Sticky-Positioned Boxes

For the purposes of any operation targeting the scroll position of a sticky positioned element (or one of its descendants), the sticky positioned element must be considered to be at its offsetted position.

<a id="ref-for-sticky-position①①"></a>

<a id="ref-for-nearest-scrollport⑤"></a>

<a id="ref-for-scroll-container①"></a>

<a id="ref-for-scrollport③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d3878f82"></a> For example, if a user clicks a link targeting a [sticky-positioned](#sticky-position) element, if the element’s [nearest scrollport](https://www.w3.org/TR/css-overflow-3/#nearest-scrollport) is currently scrolled such that the <a id="ref-for-sticky-position①②"></a>sticky positioned element is offset from its initial position, the [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container) will be scrolled back only the minimum necessary to bring it into its desired position in the [scrollport](https://www.w3.org/TR/css-overflow-3/#scrollport) (rather than scrolling all the way back to target its original, non-offsetted position).

### <a id="abspos-insets"></a>3.5. <a id="fixed-pos"></a><a id="abs-pos"></a> Absolute (and Fixed) Positioning

<a id="ref-for-absolute-position⑧"></a>

<a id="ref-for-inset-properties①④"></a>

<a id="ref-for-containing-block②②"></a>

For an [absolutely positioned](#absolute-position) box, the [inset properties](#inset-properties) effectively reduce the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) into which it is sized and positioned by the specified amounts. The resulting rectangle is called the <a id="inset-modified-containing-block"></a>inset-modified containing block. (For disambiguation, the actual <a id="ref-for-containing-block②③"></a>containing block of an <a id="ref-for-absolute-position⑨"></a>absolutely positioned box can also be called the <a id="absolute-position-containing-block"></a>absolute-position containing block.)

#### <a id="resolving-insets"></a>3.5.1.  Resolvings Insets for the “Inset-Modified Containing Block”

<a id="ref-for-inset-properties①⑤"></a>

<a id="ref-for-valdef-top-auto⑨"></a>

<a id="ref-for-self-alignment-properties"></a>

If only one [inset property](#inset-properties) in a given axis is [auto](#valdef-top-auto), it is set to zero. If both <a id="ref-for-inset-properties①⑥"></a>inset properties in a given axis are <a id="ref-for-valdef-top-auto①⓪"></a>auto, then, depending on the box’s [self-alignment property](https://www.w3.org/TR/css-align-3/#self-alignment-properties) in the relevant axis:

<a id="ref-for-valdef-self-position-self-start"></a>

for [self-start](https://www.w3.org/TR/css-align-3/#valdef-self-position-self-start) alignment or its equivalent

<a id="ref-for-static-position"></a>

<a id="ref-for-inset-properties①⑦"></a>

Set its start-edge [inset property](#inset-properties) to the [static position](#static-position), and its end-edge <a id="ref-for-inset-properties①⑧"></a>inset property to zero.

<a id="ref-for-valdef-self-position-self-end"></a>

for [self-end](https://www.w3.org/TR/css-align-3/#valdef-self-position-self-end) alignment or its equivalent

<a id="ref-for-static-position①"></a>

<a id="ref-for-inset-properties①⑨"></a>

Set its end-edge [inset property](#inset-properties) to the [static position](#static-position), and its start-edge <a id="ref-for-inset-properties②⓪"></a>inset property to zero.

<a id="ref-for-valdef-self-position-center"></a>

for [center](https://www.w3.org/TR/css-align-3/#valdef-self-position-center) alignment

<a id="ref-for-inset-properties②①"></a>

<a id="ref-for-containing-block②④"></a>

<a id="ref-for-static-position-rectangle"></a>

Let <var>start distance</var> be the distance from the center of its [static-position rectangle](#static-position-rectangle) to the start edge of its [containing block](https://www.w3.org/TR/css-display-4/#containing-block), and <var>end distance</var> be the distance from the center of its <a id="ref-for-static-position-rectangle①"></a>static-position rectangle to the end edge of its <a id="ref-for-containing-block②⑤"></a>containing block. If <var>start distance</var> is less than or equal to <var>end distance</var>, then set the start-edge [inset property](#inset-properties) to zero, and set the end-edge <a id="ref-for-inset-properties②②"></a>inset property to (<var>containing block size</var> - 2 × \|<var>start distance</var>\|); otherwise, set the end-edge <a id="ref-for-inset-properties②③"></a>inset property to zero and the start-edge <a id="ref-for-inset-properties②④"></a>inset property to (<var>containing block size</var> - 2 × \|<var>end distance</var>\|).

<a id="ref-for-overflow-alignment"></a>

<a id="ref-for-valdef-align-self-normal"></a>

<a id="ref-for-valdef-self-position-start"></a>

<a id="ref-for-baseline-alignment"></a>

<a id="ref-for-valdef-align-self-stretch"></a>

For the rules above, ignore [overflow alignment](https://www.w3.org/TR/css-align-3/#overflow-alignment), and treat [normal](https://www.w3.org/TR/css-align-3/#valdef-align-self-normal) as [start](https://www.w3.org/TR/css-align-3/#valdef-self-position-start) and any [baseline](https://www.w3.org/TR/css-align-3/#baseline-alignment) or [stretch](https://www.w3.org/TR/css-align-3/#valdef-align-self-stretch) alignment value as its fallback alignment.

<a id="ref-for-containing-block②⑥"></a>

<a id="ref-for-weaker-inset"></a>

<a id="ref-for-valdef-top-auto①①"></a>

<a id="ref-for-end⑥"></a>

<a id="ref-for-writing-mode④"></a>

If these adjustments result in an effective [containing block](https://www.w3.org/TR/css-display-4/#containing-block) size in any axis less than zero, then the [weaker inset](#weaker-inset) in the affected axis is reduced (possibly becoming negative) to bring that size up to zero. In the case that only one inset is [auto](#valdef-top-auto), that is the <a id="weaker-inset"></a>weaker inset (whose opposite inset is the <a id="stronger-inset"></a>stronger inset); otherwise the <a id="ref-for-weaker-inset①"></a>weaker inset is the inset of the [end](https://www.w3.org/TR/css-writing-modes-4/#end) edge (where <a id="ref-for-end⑦"></a>end is interpreted relative to the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the <a id="ref-for-containing-block②⑦"></a>containing block).

<a id="ref-for-absolute-position①⓪"></a>

<a id="ref-for-inset-modified-containing-block"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Sizing and positioning of the [absolutely positioned box](#absolute-position) into this [inset-modified containing block](#inset-modified-containing-block) is as described in [§ 4 Absolute Positioning Layout Model](#abspos-layout).

<a id="ref-for-self-alignment-properties①"></a>

<a id="ref-for-valdef-align-self-normal①"></a>

<a id="ref-for-resolved-value"></a>

<a id="ref-for-weaker-inset②"></a>

<a id="ref-for-inset-modified-containing-block①"></a>

<a id="ref-for-margin-box②"></a>

<a id="ref-for-used-value④"></a>

If its [self-alignment property](https://www.w3.org/TR/css-align-3/#self-alignment-properties) in an axis is [normal](https://www.w3.org/TR/css-align-3/#valdef-align-self-normal), then the [resolved value](https://www.w3.org/TR/cssom-1/#resolved-value) of its [weaker inset](#weaker-inset) in that axis is the value necessary to match that edge of its [inset-modified containing block](#inset-modified-containing-block) to the corresponding edge of its [margin box](https://www.w3.org/TR/css-box-4/#margin-box) after [layout](#abspos-layout). (Otherwise the <a id="ref-for-resolved-value①"></a>resolved value is the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) described above.)

#### <a id="staticpos-rect"></a>3.5.2.  Calculating the Static Position and the “Static-Position Rectangle”

<a id="ref-for-inset-properties②⑤"></a>

<a id="ref-for-valdef-top-auto①②"></a>

<a id="ref-for-alignment-container"></a>

<a id="ref-for-formatting-context③"></a>

<a id="ref-for-propdef-position⑧"></a>

<a id="ref-for-static-position②"></a>

When both [inset properties](#inset-properties) in a given axis are [auto](#valdef-top-auto), they are resolved into a <a id="static-position"></a>static position by aligning the box into its <a id="static-position-rectangle"></a>static-position rectangle, an [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container) derived from the [formatting context](https://www.w3.org/TR/css-display-4/#formatting-context) the box would have participated in if it were [position: static](#propdef-position) (independent of its actual containing block). The [static position](#static-position) represents an approximation of the position the box would have had if it were <a id="ref-for-propdef-position⑨"></a>position: static.

Block Layout  
<a id="ref-for-block-start"></a>

<a id="ref-for-static-position-containing-block"></a>

<a id="ref-for-static-position-rectangle②"></a>

<a id="ref-for-block-level-box"></a>

<a id="ref-for-static-position③"></a>

The [static positions](#static-position) of a [block-level box](https://www.w3.org/TR/css-display-4/#block-level-box) are defined in [\[CSS2\]](#biblio-css2) Chapter 10. The [static-position rectangle](#static-position-rectangle) is a zero-thickness rectangle spanning between the inline-axis sides of the box’s [static-position containing block](#static-position-containing-block) and positioned at its [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) <a id="ref-for-static-position④"></a>static position (see [CSS2§10.6.4](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-height)).

<a id="ref-for-static-position-rectangle③"></a>

<a id="ref-for-block-start①"></a>

<a id="ref-for-inline-start"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In block layout the [static-position rectangle](#static-position-rectangle) corresponds to the position of the “hypothetical box” described in [CSS2.1§10.3.7](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-width). Since it has no alignment properties, CSS2.1 always uses a [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start) [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) alignment of the absolutely-positioned box within the <a id="ref-for-static-position-rectangle④"></a>static-position rectangle.

Inline Layout  
<a id="ref-for-inline-start①"></a>

<a id="ref-for-line-box"></a>

<a id="ref-for-line-under"></a>

<a id="ref-for-line-over"></a>

<a id="ref-for-static-position-rectangle⑤"></a>

<a id="ref-for-inline-level-box"></a>

<a id="ref-for-static-position⑤"></a>

The [static positions](#static-position) of an [inline-level box](https://www.w3.org/TR/css-display-4/#inline-level-box) are defined in [\[CSS2\]](#biblio-css2) Chapter 10. The [static-position rectangle](#static-position-rectangle) is a zero-thickness rectangle spanning between the [line-over](https://www.w3.org/TR/css-writing-modes-4/#line-over)/[line-under](https://www.w3.org/TR/css-writing-modes-4/#line-under) sides of the [line box](https://www.w3.org/TR/css-inline-3/#line-box) that would have contained its “hypothetical box” (see [CSS2§10.3.7](https://www.w3.org/TR/CSS2/visudet.html#abs-non-replaced-width)); and positioned at its [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) <a id="ref-for-static-position⑥"></a>static position.

Flex Layout  
<a id="ref-for-main-axis"></a>

<a id="ref-for-outer-edge"></a>

<a id="ref-for-cross-axis"></a>

<a id="ref-for-content-edge②"></a>

<a id="ref-for-flex-container"></a>

<a id="ref-for-static-position-rectangle⑥"></a>

The [static-position rectangle](#static-position-rectangle) of the child of a [flex container](https://www.w3.org/TR/css-flexbox-1/#flex-container) corresponds to the [content edges](https://www.w3.org/TR/css-box-4/#content-edge) of the <a id="ref-for-flex-container①"></a>flex container in the [cross axis](https://www.w3.org/TR/css-flexbox-1/#cross-axis), and to the [outer edges](https://www.w3.org/TR/css-box-4/#outer-edge) of its hypothetical position in the [main axis](https://www.w3.org/TR/css-flexbox-1/#main-axis). See [static position of a flex container child](https://www.w3.org/TR/css-flexbox-1/#abspos-items) in [\[CSS-FLEXBOX-1\]](#biblio-css-flexbox-1).

Grid Layout  
<a id="ref-for-grid-placement-property①"></a>

<a id="ref-for-grid-area①"></a>

<a id="ref-for-containing-block②⑧"></a>

<a id="ref-for-content-edge③"></a>

<a id="ref-for-grid-container①"></a>

<a id="ref-for-static-position-rectangle⑦"></a>

By default, the [static-position rectangle](#static-position-rectangle) of the child of a [grid container](https://www.w3.org/TR/css-grid-2/#grid-container) corresponds to the [content edges](https://www.w3.org/TR/css-box-4/#content-edge) of the <a id="ref-for-grid-container②"></a>grid container. However, if that <a id="ref-for-grid-container③"></a>grid container also establishes the box’s actual [containing block](https://www.w3.org/TR/css-display-4/#containing-block), then the [grid area](https://www.w3.org/TR/css-grid-2/#grid-area) specified by the [grid-placement properties](https://www.w3.org/TR/css-grid-2/#grid-placement-property) establishes its <a id="ref-for-static-position-rectangle⑧"></a>static-position rectangle instead. See the [static position of a grid container child](https://www.w3.org/TR/css-grid-1/#static-position) in [\[CSS-GRID-1\]](#biblio-css-grid-1).

<a id="ref-for-static-position⑦"></a>

<a id="ref-for-static-position-rectangle⑨"></a>

<a id="ref-for-propdef-float①"></a>

<a id="ref-for-propdef-clear"></a>

<a id="ref-for-propdef-position①⓪"></a>

<a id="ref-for-initial-value①"></a>

<a id="ref-for-propdef-display"></a>

<a id="ref-for-containing-block②⑨"></a>

<a id="ref-for-box-alignment-properties"></a>

<a id="ref-for-static-position-containing-block①"></a>

<a id="ref-for-writing-mode⑤"></a>

<a id="ref-for-fixed-position⑦"></a>

<a id="ref-for-initial-containing-block②"></a>

<a id="ref-for-x1⑤"></a>

<a id="ref-for-scroll-container②"></a>

<a id="ref-for-initial-scroll-position"></a>

Finding the [static position](#static-position) and the [static-position rectangle](#static-position-rectangle) assumes that both [float](https://www.w3.org/TR/CSS2/visuren.html#propdef-float) and [clear](https://www.w3.org/TR/CSS2/visuren.html#propdef-clear) (as well as [position](#propdef-position)) have their [initial values](https://www.w3.org/TR/css-cascade-5/#initial-value), and can consequently require assuming a different hypothetical value for [display](https://www.w3.org/TR/css-display-4/#propdef-display) as well. (The [containing block](https://www.w3.org/TR/css-display-4/#containing-block) the element would have had under these conditions is the <a id="static-position-containing-block"></a>static-position containing block.) To the extent the [box alignment properties](https://www.w3.org/TR/css-align-3/#box-alignment-properties) have an effect, they use the [static-position containing block](#static-position-containing-block) as the effective <a id="ref-for-containing-block③⓪"></a>containing block, including using its [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) for resolving alignment axes and directions. Additionally, the <a id="ref-for-containing-block③①"></a>containing block of [fixed positioned](#fixed-position) elements is assumed to be the [initial containing block](https://www.w3.org/TR/css-display-4/#initial-containing-block) instead of the [viewport](https://www.w3.org/TR/CSS2/visuren.html#x1), and all [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container) should be assumed to be scrolled to their [initial scroll position](https://www.w3.org/TR/css-overflow-3/#initial-scroll-position). Lastly, all auto margins on the box itself are treated as zero.

<a id="ref-for-document-top-layer"></a>

<a id="ref-for-initial-containing-block③"></a>

<a id="ref-for-static-position-rectangle①⓪"></a>

Boxes in the [top layer](https://www.w3.org/TR/css-position-4/#document-top-layer) always use the [initial containing block](https://www.w3.org/TR/css-display-4/#initial-containing-block) as their [static-position rectangle](#static-position-rectangle).

#### <a id="abspos-breaking"></a>3.5.3. <a id="breaking"></a> Fragmenting Absolutely-positioned Elements

<a id="ref-for-fragmented-flow"></a>

<a id="ref-for-absolute-position①①"></a>

<a id="ref-for-containing-block③②"></a>

<a id="ref-for-fragmentation-break"></a>

<a id="ref-for-fragmentation-container"></a>

In a [fragmented flow](https://www.w3.org/TR/css-break-4/#fragmented-flow), an [absolutely positioned box](#absolute-position) is positioned relative to its [containing block](https://www.w3.org/TR/css-display-4/#containing-block) ignoring any [fragmentation breaks](https://www.w3.org/TR/css-break-4/#fragmentation-break) (as if the flow were continuous). The box may subsequently be broken over several [fragmentation containers](https://www.w3.org/TR/css-break-4/#fragmentation-container).

<a id="ref-for-paged-media②"></a>

For absolutely positioned content in [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media) that resolves to a position on a page other than the page being laid out (the current page), or resolves to a position on the current page that has already been rendered for printing, printers may place the content:

- on the current page,
- on a subsequent page, or
- may omit it altogether.

<a id="ref-for-block-level"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A [block-level](https://www.w3.org/TR/css-display-4/#block-level) element that is split over several pages can have a different width on each page, and there may be device-specific limits.

<a id="ref-for-fixed-position⑧"></a>

User agents must not paginate the content of [fixed-positioned boxes](#fixed-position).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: User agents might print invisible content in other ways. See [CSS Paged Media 3 § 3.2 Content outside the page box](https://www.w3.org/TR/css-page-3/#content-outside-box).

## <a id="abspos-layout"></a>4.  Absolute Positioning Layout Model

<a id="ref-for-absolute-position①②"></a>

<a id="ref-for-out-of-flow②"></a>

<a id="ref-for-containing-block③③"></a>

[Absolute positioning](#absolute-position) not only takes a box [out of flow](https://www.w3.org/TR/css-display-4/#out-of-flow), but also lays it out in its [containing block](https://www.w3.org/TR/css-display-4/#containing-block) (after the final size of the <a id="ref-for-containing-block③④"></a>containing block has been determined) according to the <a id="absolute-positioning-layout"></a>absolute positioning layout model:

1.  <a id="ref-for-available"></a>

    <a id="ref-for-inset-modified-containing-block②"></a>

    First, its [inset-modified containing block](#inset-modified-containing-block) is calculated, defining its [available space](https://www.w3.org/TR/css-sizing-3/#available). (See [§ 3.5 Absolute (and Fixed) Positioning](#abspos-insets).)

    <a id="ref-for-absolute-position①③"></a>

    <a id="ref-for-containing-block③⑤"></a>

    <a id="ref-for-available①"></a>

    <a id="ref-for-definite"></a>

    Because an [absolutely positioned box](#absolute-position) does not affect the size of its [containing block](https://www.w3.org/TR/css-display-4/#containing-block), its [available space](https://www.w3.org/TR/css-sizing-3/#available) is always [definite](https://www.w3.org/TR/css-sizing-3/#definite).

2.  <a id="ref-for-containing-block③⑥"></a>

    <a id="ref-for-min-width"></a>

    <a id="ref-for-max-width"></a>

    <a id="ref-for-preferred-size"></a>

    <a id="ref-for-available②"></a>

    <a id="ref-for-definite①"></a>

    Next, its width and height are resolved against this [definite](https://www.w3.org/TR/css-sizing-3/#definite) [available space](https://www.w3.org/TR/css-sizing-3/#available), as its [preferred size](https://www.w3.org/TR/css-sizing-3/#preferred-size) capped by its [maximum size](https://www.w3.org/TR/css-sizing-3/#max-width) (if any), floored by its [minimum size](https://www.w3.org/TR/css-sizing-3/#min-width). See [§ 4.1 Automatic Sizes of Absolutely-Positioned Boxes](#abspos-auto-size). Percentages, however, are resolved against the original [containing block](https://www.w3.org/TR/css-display-4/#containing-block) size.

3.  Then, the value of any auto margins are calculated, see [§ 4.2 Auto Margins of Absolutely-Positioned Boxes](#abspos-margins).

4.  <a id="ref-for-inset-modified-containing-block③"></a>

    <a id="ref-for-margin-box③"></a>

    Lastly, its [margin box](https://www.w3.org/TR/css-box-4/#margin-box) is aligned within the [inset-modified containing block](#inset-modified-containing-block), see [§ 5 Self-Alignment of Absolutely Positioned Boxes](#abspos-alignment).

### <a id="abspos-auto-size"></a>4.1.  Automatic Sizes of Absolutely-Positioned Boxes

<a id="ref-for-automatic-size"></a>

<a id="ref-for-absolute-position①④"></a>

<a id="ref-for-inset-modified-containing-block④"></a>

The [automatic size](https://www.w3.org/TR/css-sizing-3/#automatic-size) of an [absolutely positioned box](#absolute-position) is resolved against its [inset-modified containing block](#inset-modified-containing-block) as follows:

<a id="ref-for-margin②"></a>

<a id="ref-for-inset-properties②⑥"></a>

<a id="ref-for-valdef-align-self-stretch①"></a>

<a id="ref-for-self-alignment-properties②"></a>

If its [self-alignment property](https://www.w3.org/TR/css-align-3/#self-alignment-properties) in the relevant axis is [stretch](https://www.w3.org/TR/css-align-3/#valdef-align-self-stretch) and neither of its [inset properties](#inset-properties) nor [margins](https://www.w3.org/TR/css-box-4/#margin) in that axis are auto

<a id="ref-for-inset-properties②⑦"></a>

<a id="ref-for-valdef-top-auto①③"></a>

<a id="ref-for-table-wrapper-box"></a>

<a id="ref-for-non-replaced"></a>

<a id="ref-for-valdef-align-self-normal②"></a>

Or if it is [normal](https://www.w3.org/TR/css-align-3/#valdef-align-self-normal) and the box is [non-replaced](https://www.w3.org/TR/css-display-4/#non-replaced), not a [table wrapper box](https://www.w3.org/TR/css-tables-3/#table-wrapper-box), and has no [auto](#valdef-top-auto) [inset](#inset-properties) in the relevant axis

<a id="ref-for-automatic-size①"></a>

<a id="ref-for-stretch-fit-size"></a>

Its [automatic size](https://www.w3.org/TR/css-sizing-3/#automatic-size) is its [stretch-fit size](https://www.w3.org/TR/css-sizing-3/#stretch-fit-size).

Otherwise

<a id="ref-for-automatic-size②"></a>

<a id="ref-for-fit-content-size"></a>

Its [automatic size](https://www.w3.org/TR/css-sizing-3/#automatic-size) is its [fit-content size](https://www.w3.org/TR/css-sizing-3/#fit-content-size).

<a id="ref-for-automatic-size③"></a>

<a id="ref-for-ratio-dependent-axis"></a>

<a id="ref-for-max-content"></a>

<a id="ref-for-valdef-top-auto①④"></a>

<a id="ref-for-inset-properties②⑧"></a>

<a id="ref-for-block-axis"></a>

<a id="ref-for-ratio-determining-axis"></a>

However, if the box has an aspect-ratio, then an [automatic size](https://www.w3.org/TR/css-sizing-3/#automatic-size) in the [ratio-dependent axis](https://www.w3.org/TR/css-sizing-4/#ratio-dependent-axis) is instead resolved as a [max-content size](https://www.w3.org/TR/css-sizing-3/#max-content). When both axes have an <a id="ref-for-automatic-size④"></a>automatic size, if only one axis has an [auto](#valdef-top-auto) [inset](#inset-properties) then that axis is the <a id="ref-for-ratio-dependent-axis①"></a>ratio-dependent axis, else the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis) is the <a id="ref-for-ratio-dependent-axis②"></a>ratio-dependent axis. An <a id="ref-for-automatic-size⑤"></a>automatic size in the [ratio-determining axis](https://www.w3.org/TR/css-sizing-4/#ratio-determining-axis) is determined as above.

<a id="ref-for-margin③"></a>

For the purpose of computing these sizes, any auto [margins](https://www.w3.org/TR/css-box-4/#margin) are treated as zero.

<a id="ref-for-automatic-minimum-size"></a>

The [automatic minimum size](https://www.w3.org/TR/css-sizing-3/#automatic-minimum-size) of an absolutely-positioned box is always zero.

<a id="ref-for-replaced-element"></a>

<a id="ref-for-non-replaced①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: To the extent that form controls can be resized (and are not directly representing [replaced elements](https://www.w3.org/TR/css-display-4/#replaced-element) such as images), they are expected to be treated as [non-replaced](https://www.w3.org/TR/css-display-4/#non-replaced) here. In HTML, all form controls other than `<input type=image>` are treated as <a id="ref-for-non-replaced②"></a>non-replaced.

### <a id="abspos-margins"></a>4.2.  Auto Margins of Absolutely-Positioned Boxes

<a id="ref-for-inset-properties②⑨"></a>

<a id="ref-for-valdef-top-auto①⑤"></a>

<a id="ref-for-margin④"></a>

If either [inset property](#inset-properties) in the relevant axis is [auto](#valdef-top-auto), then any auto [margins](https://www.w3.org/TR/css-box-4/#margin) resolve to zero.

<a id="ref-for-inset-modified-containing-block⑤"></a>

<a id="ref-for-writing-mode⑥"></a>

<a id="ref-for-containing-block③⑦"></a>

<a id="ref-for-inline-axis"></a>

<a id="ref-for-inline-start②"></a>

<a id="ref-for-inline-end"></a>

Otherwise, the <var>remaining space</var> is calculated as the size of its [inset-modified containing block](#inset-modified-containing-block) in the relevant axis minus the box’s used size in the relevant axis, and this <var>remaining space</var> is divided among any auto margins in the relevant axis. However, (all with respect to the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the [containing block](https://www.w3.org/TR/css-display-4/#containing-block)), if in the [inline axis](https://www.w3.org/TR/css-writing-modes-4/#inline-axis) the <var>remaining space</var> is negative and both margins are auto, the [start](https://www.w3.org/TR/css-writing-modes-4/#inline-start) margin resolves to zero and the [end](https://www.w3.org/TR/css-writing-modes-4/#inline-end) margin receives the <var>remaining space</var>.

<a id="ref-for-in-flow①"></a>

<a id="ref-for-margin⑤"></a>

<a id="ref-for-absolute-position①⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Unlike typical [in-flow](https://www.w3.org/TR/css-display-4/#in-flow) layout, the space distributed to auto [margins](https://www.w3.org/TR/css-box-4/#margin) can be negative in [absolute positioning](#absolute-position).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-f171bef8"></a> What should happen in “negatively-sized” containing blocks? CSS2.1 and this draft currently conflict. [\[Issue \#11478\]](https://github.com/w3c/csswg-drafts/issues/11478)

## <a id="abspos-alignment"></a>5.  Self-Alignment of Absolutely Positioned Boxes

<a id="ref-for-computed-value①"></a>

<a id="ref-for-inset-properties③⓪"></a>

<a id="ref-for-valdef-top-auto①⑥"></a>

<a id="ref-for-margin-box④"></a>

<a id="ref-for-inset-modified-containing-block⑥"></a>

<a id="ref-for-stronger-inset"></a>

<a id="ref-for-containing-block③⑧"></a>

If the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of either [inset property](#inset-properties) in an axis is [auto](#valdef-top-auto), then its [margin box](https://www.w3.org/TR/css-box-4/#margin-box) is aligned to the edge of the [inset-modified containing block](#inset-modified-containing-block) corresponding to its [stronger inset](#stronger-inset) (even if this would overflow its [containing block](https://www.w3.org/TR/css-display-4/#containing-block)).

<a id="ref-for-margin⑥"></a>

Otherwise, if either [margin](https://www.w3.org/TR/css-box-4/#margin) is auto, its position is resolved according to [§ 4.2 Auto Margins of Absolutely-Positioned Boxes](#abspos-margins).

<a id="ref-for-self-alignment-properties③"></a>

<a id="ref-for-writing-mode⑦"></a>

<a id="ref-for-containing-block③⑨"></a>

<a id="ref-for-margin-box⑤"></a>

<a id="ref-for-alignment-subject"></a>

<a id="ref-for-inset-modified-containing-block⑦"></a>

<a id="ref-for-alignment-container①"></a>

<a id="ref-for-overflow-alignment①"></a>

Otherwise, the box is aligned as specified by its [self-alignment property](https://www.w3.org/TR/css-align-3/#self-alignment-properties) in the relevant axis (as defined by the [writing mode](https://www.w3.org/TR/css-writing-modes-4/#writing-mode) of the [containing block](https://www.w3.org/TR/css-display-4/#containing-block)), using its [margin box](https://www.w3.org/TR/css-box-4/#margin-box) as the [alignment subject](https://www.w3.org/TR/css-align-3/#alignment-subject) and the [inset-modified containing block](#inset-modified-containing-block) as the [alignment container](https://www.w3.org/TR/css-align-3/#alignment-container). However, if an explicit [overflow alignment](https://www.w3.org/TR/css-align-3/#overflow-alignment) is not specified and its <a id="ref-for-margin-box⑥"></a>margin box overflows the <a id="ref-for-inset-modified-containing-block⑧"></a>inset-modified containing block, its alignment is adjusted to minimize overflow as specified in [CSS Box Alignment 3 § 4.4.1.2 Self-Alignment for Absolutely Positioned Boxes](https://www.w3.org/TR/css-align-3/#auto-safety-position).

## <a id="abspos-old"></a>6.  Old Absolute Positioning Layout Model

<a id="ref-for-horizontal-writing-mode"></a>

<a id="ref-for-self-align"></a>

<a id="ref-for-valdef-align-self-normal③"></a>

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-3a21c2a0"></a> This section is being replaced with the new [§ 4 Absolute Positioning Layout Model](#abspos-layout) section. It is preserved here for comparison: both models should yield the same result in [horizontal writing modes](https://www.w3.org/TR/css-writing-modes-4/#horizontal-writing-mode) when the box’s [self-alignment](https://www.w3.org/TR/css-align-3/#self-align) is [normal](https://www.w3.org/TR/css-align-3/#valdef-align-self-normal).

### <a id="abs-non-replaced-width"></a>6.1.  The Width of Absolutely-Positioned, Non-Replaced Elements

The constraint that determines the used values for these elements is:

<a id="ref-for-propdef-left⑤"></a>

<a id="ref-for-propdef-margin-left"></a>

<a id="ref-for-propdef-border-left-width"></a>

<a id="ref-for-propdef-padding-left"></a>

<a id="ref-for-propdef-width"></a>

<a id="ref-for-propdef-padding-right"></a>

<a id="ref-for-propdef-border-right-width"></a>

<a id="ref-for-propdef-margin-right"></a>

<a id="ref-for-propdef-right⑤"></a>

<code>
		<a href="#propdef-left">left</a> + <a href="https://www.w3.org/TR/css-box-4/#propdef-margin-left">margin-left</a> + <a href="https://www.w3.org/TR/css-backgrounds-3/#propdef-border-left-width">border-left-width</a> + <a href="https://www.w3.org/TR/css-box-4/#propdef-padding-left">padding-left</a> + <a href="https://www.w3.org/TR/css-sizing-3/#propdef-width">width</a> +
		<a href="https://www.w3.org/TR/css-box-4/#propdef-padding-right">padding-right</a> + <a href="https://www.w3.org/TR/css-backgrounds-3/#propdef-border-right-width">border-right-width</a> + <a href="https://www.w3.org/TR/css-box-4/#propdef-margin-right">margin-right</a> +
		<a href="#propdef-right">right</a> = width of containing block
	</code>

<a id="ref-for-propdef-left⑥"></a>

<a id="ref-for-propdef-width①"></a>

<a id="ref-for-propdef-right⑥"></a>

<a id="ref-for-valdef-top-auto①⑦"></a>

<a id="ref-for-propdef-margin-left①"></a>

<a id="ref-for-propdef-margin-right①"></a>

<a id="ref-for-propdef-direction"></a>

<a id="ref-for-containing-block④⓪"></a>

<a id="ref-for-valdef-direction-ltr"></a>

If all three of [left](#propdef-left), [width](https://www.w3.org/TR/css-sizing-3/#propdef-width), and [right](#propdef-right) are [auto](#valdef-top-auto): First set any auto values for [margin-left](https://www.w3.org/TR/css-box-4/#propdef-margin-left) and [margin-right](https://www.w3.org/TR/css-box-4/#propdef-margin-right) to 0. Then, if the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) property of the element establishing the static-position [containing block](https://www.w3.org/TR/css-display-4/#containing-block) is [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr) set <a id="ref-for-propdef-left⑦"></a>left to the static position and apply rule number <em>three</em> below; otherwise, set <a id="ref-for-propdef-right⑦"></a>right to the static-position and apply rule number <em>one</em> below.

<a id="ref-for-valdef-top-auto①⑧"></a>

<a id="ref-for-propdef-margin-left②"></a>

<a id="ref-for-propdef-margin-right②"></a>

<a id="ref-for-valdef-direction-ltr①"></a>

<a id="ref-for-valdef-direction-rtl"></a>

<a id="ref-for-propdef-left⑧"></a>

<a id="ref-for-propdef-direction①"></a>

<a id="ref-for-containing-block④①"></a>

<a id="ref-for-propdef-right⑧"></a>

If none of the three is [auto](#valdef-top-auto): If both [margin-left](https://www.w3.org/TR/css-box-4/#propdef-margin-left) and [margin-right](https://www.w3.org/TR/css-box-4/#propdef-margin-right) are auto, solve the equation under the extra constraint that the two margins get equal values, unless this would make them negative, in which case when direction of the containing block is [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr) ([rtl](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-rtl)), set <a id="ref-for-propdef-margin-left③"></a>margin-left (<a id="ref-for-propdef-margin-right③"></a>margin-right) to 0 and solve for <a id="ref-for-propdef-margin-right④"></a>margin-right (<a id="ref-for-propdef-margin-left④"></a>margin-left). If one of <a id="ref-for-propdef-margin-left⑤"></a>margin-left or <a id="ref-for-propdef-margin-right⑤"></a>margin-right is auto, solve the equation for that value. If the values are over-constrained, ignore the value for [left](#propdef-left) (in case the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) property of the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) is <a id="ref-for-valdef-direction-rtl①"></a>rtl) or [right](#propdef-right) (in case <a id="ref-for-propdef-direction②"></a>direction is <a id="ref-for-valdef-direction-ltr②"></a>ltr) and solve for that value.

<a id="ref-for-propdef-margin-left⑥"></a>

<a id="ref-for-propdef-margin-right⑥"></a>

Otherwise, set auto values for [margin-left](https://www.w3.org/TR/css-box-4/#propdef-margin-left) and [margin-right](https://www.w3.org/TR/css-box-4/#propdef-margin-right) to 0, and pick one of the following six rules that apply.

1.  <a id="ref-for-propdef-right⑨"></a>

    <a id="ref-for-valdef-top-auto①⑨"></a>

    <a id="ref-for-propdef-width②"></a>

    <a id="ref-for-propdef-left⑨"></a>

    If [left](#propdef-left) and [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) are [auto](#valdef-top-auto) and [right](#propdef-right) is not <a id="ref-for-valdef-top-auto②⓪"></a>auto, then the width is shrink-to-fit. Then solve for <a id="ref-for-propdef-left①⓪"></a>left.

2.  <a id="ref-for-valdef-direction-rtl②"></a>

    <a id="ref-for-valdef-direction-ltr③"></a>

    <a id="ref-for-containing-block④②"></a>

    <a id="ref-for-propdef-direction③"></a>

    <a id="ref-for-valdef-width-auto"></a>

    <a id="ref-for-propdef-width③"></a>

    <a id="ref-for-valdef-top-auto②①"></a>

    <a id="ref-for-propdef-right①⓪"></a>

    <a id="ref-for-propdef-left①①"></a>

    If [left](#propdef-left) and [right](#propdef-right) are [auto](#valdef-top-auto) and [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) is not [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), then if the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) property of the element establishing the static-position [containing block](https://www.w3.org/TR/css-display-4/#containing-block) is [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr) set <a id="ref-for-propdef-left①②"></a>left to the static-position, otherwise set <a id="ref-for-propdef-right①①"></a>right to the static-position. Then solve for <a id="ref-for-propdef-left①③"></a>left (if <a id="ref-for-propdef-direction④"></a>direction is [rtl](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-rtl)) or <a id="ref-for-propdef-right①②"></a>right (if <a id="ref-for-propdef-direction⑤"></a>direction is <a id="ref-for-valdef-direction-ltr④"></a>ltr).

3.  <a id="ref-for-valdef-top-auto②②"></a>

    <a id="ref-for-propdef-left①④"></a>

    <a id="ref-for-valdef-width-auto①"></a>

    <a id="ref-for-propdef-right①③"></a>

    <a id="ref-for-propdef-width④"></a>

    If [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [right](#propdef-right) are [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto) and [left](#propdef-left) is not [auto](#valdef-top-auto), then the width is shrink-to-fit. Then solve for <a id="ref-for-propdef-right①④"></a>right.

4.  <a id="ref-for-valdef-width-auto②"></a>

    <a id="ref-for-propdef-right①⑤"></a>

    <a id="ref-for-propdef-width⑤"></a>

    <a id="ref-for-valdef-top-auto②③"></a>

    <a id="ref-for-propdef-left①⑤"></a>

    If [left](#propdef-left) is [auto](#valdef-top-auto), [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [right](#propdef-right) are not [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), then solve for <a id="ref-for-propdef-left①⑥"></a>left.

5.  <a id="ref-for-valdef-top-auto②④"></a>

    <a id="ref-for-propdef-right①⑥"></a>

    <a id="ref-for-propdef-left①⑦"></a>

    <a id="ref-for-valdef-width-auto③"></a>

    <a id="ref-for-propdef-width⑥"></a>

    If [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) is [auto](https://www.w3.org/TR/css-sizing-3/#valdef-width-auto), [left](#propdef-left) and [right](#propdef-right) are not [auto](#valdef-top-auto), then solve for <a id="ref-for-propdef-width⑦"></a>width.

6.  <a id="ref-for-propdef-width⑧"></a>

    <a id="ref-for-propdef-left①⑧"></a>

    <a id="ref-for-valdef-top-auto②⑤"></a>

    <a id="ref-for-propdef-right①⑦"></a>

    If [right](#propdef-right) is [auto](#valdef-top-auto), [left](#propdef-left) and [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) are not <a id="ref-for-valdef-top-auto②⑥"></a>auto, then solve for <a id="ref-for-propdef-right①⑧"></a>right.

<a id="abspos-auto"></a>

**Table 7**

Summary of rules for `dir=ltr` in horizontal writing modes

Representation note: merged conditions are repeated; each Result label refers to the matching rule list below.

| Is auto? / <a id="ref-for-propdef-left①⑨"></a> [left](#propdef-left) | Is auto? / <a id="ref-for-propdef-width⑨"></a> [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) | Is auto? / <a id="ref-for-propdef-right①⑨"></a> [right](#propdef-right) | Is auto? / <a id="ref-for-propdef-margin-left⑦"></a> [margin-left](https://www.w3.org/TR/css-box-4/#propdef-margin-left) | Is auto? / <a id="ref-for-propdef-margin-right⑦"></a> [margin-right](https://www.w3.org/TR/css-box-4/#propdef-margin-right) | Result |
| --- | --- | --- | --- | --- | --- |
| ✔ | ✔ | ✔ | any | any | Result 1 |
| ✘ | ✘ | ✘ | ✔ | ✘ | Result 2 |
| ✘ | ✘ | ✘ | ✘ | ✔ | Result 2 |
| ✘ | ✘ | ✘ | ✔ | ✔ | Result 3 |
| ✘ | ✘ | ✘ | ✘ | ✘ | Result 4 |
| ✔ | ✘ | ✔ | any | any | Result 5 |
| ✔ | ✔ | ✘ | any | any | Result 6 |
| ✘ | ✔ | ✔ | any | any | Result 7 |
| ✔ | ✘ | ✘ | any | any | Result 8 |
| ✘ | ✘ | ✔ | any | any | Result 8 |
| ✘ | ✔ | ✘ | any | any | Result 8 |

**Result 1**

- auto margins → zero
- left → static pos
- width → shrink-to-fit
- right → solve

**Result 2**

auto margin → free space

**Result 3**

- margins split positive free space
- right margin gets negative free space

**Result 4**

<a id="ref-for-valdef-top-auto②⑦"></a>

<a id="ref-for-propdef-right②⓪"></a>

treat [right](#propdef-right) as [auto](#valdef-top-auto)

**Result 5**

- auto margins → zero
- left → static pos
- width → as specified
- right → solve

**Result 6**

- auto margins → zero
- left → solve
- width → shrink-to-fit
- right → as specified

**Result 7**

- auto margins → zero
- left → as specified
- width → shrink-to-fit
- right → solve

**Result 8**

- auto margins → zero
- solve for auto

### <a id="abs-replaced-width"></a>6.2.  The width of absolute or fixed positioned, replaced elements

<a id="ref-for-propdef-height"></a>

<a id="ref-for-propdef-width①⓪"></a>

If [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) and [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) both have computed values of auto and the element also has an intrinsic width, then that intrinsic width is the used value of <a id="ref-for-propdef-width①①"></a>width.

<a id="ref-for-propdef-height①"></a>

<a id="ref-for-propdef-width①②"></a>

If [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) and [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) both have computed values of auto and the element has no intrinsic width, but does have an intrinsic height and intrinsic ratio; or if <a id="ref-for-propdef-width①③"></a>width has a computed value of auto, <a id="ref-for-propdef-height②"></a>height has some other computed value, and the element does have an intrinsic ratio; then the used value of <a id="ref-for-propdef-width①④"></a>width is:

`(used height) * (intrinsic ratio)`

<a id="ref-for-propdef-height③"></a>

<a id="ref-for-propdef-width①⑤"></a>

<a id="ref-for-containing-block④③"></a>

<a id="ref-for-normal-flow"></a>

If [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) and [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) both have computed values of auto, the element has an intrinsic ratio but no intrinsic height or width, and the [containing block’s](https://www.w3.org/TR/css-display-4/#containing-block) width does not itself depend on the replaced element’s width, then the used value of <a id="ref-for-propdef-width①⑥"></a>width is calculated from the constraint equation used for [block-level, non-replaced elements in](https://www.w3.org/TR/CSS21/visudet.html#blockwidth) [normal flow](https://www.w3.org/TR/CSS2/visuren.html#normal-flow).

<a id="ref-for-propdef-width①⑦"></a>

Otherwise, if [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) has a computed value of auto, and the element has an intrinsic width, then that intrinsic width is the used value of <a id="ref-for-propdef-width①⑧"></a>width.

<a id="ref-for-propdef-width①⑨"></a>

Otherwise, if [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) has a computed value of auto, but none of the conditions above are met, and then the used value of <a id="ref-for-propdef-width②⓪"></a>width becomes 300px. If 300px is too wide to fit the device, user agents should use the width of the largest rectangle that has a 2:1 ratio and fits the device instead.

<a id="ref-for-propdef-width②①"></a>

After establishing the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width), in order to position the replaced element, apply the following rules as appropriate.

1.  <a id="ref-for-valdef-direction-rtl③"></a>

    <a id="ref-for-valdef-direction-ltr⑤"></a>

    <a id="ref-for-containing-block④④"></a>

    <a id="ref-for-propdef-direction⑥"></a>

    <a id="ref-for-propdef-right②①"></a>

    <a id="ref-for-propdef-left②⓪"></a>

    If both [left](#propdef-left) and [right](#propdef-right) have the value auto, and if the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) property of the element establishing the static-position [containing block](https://www.w3.org/TR/css-display-4/#containing-block) is [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr), set <a id="ref-for-propdef-left②①"></a>left to the static position and solve for <a id="ref-for-propdef-right②②"></a>right; else if <a id="ref-for-propdef-direction⑦"></a>direction is [rtl](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-rtl), set <a id="ref-for-propdef-right②③"></a>right to the static position and solve for <a id="ref-for-propdef-left②②"></a>left.

2.  <a id="ref-for-propdef-margin-right⑧"></a>

    <a id="ref-for-propdef-margin-left⑧"></a>

    <a id="ref-for-propdef-right②④"></a>

    <a id="ref-for-propdef-left②③"></a>

    If [left](#propdef-left) is auto and [right](#propdef-right) is not auto, replace any auto on [margin-left](https://www.w3.org/TR/css-box-4/#propdef-margin-left) or [margin-right](https://www.w3.org/TR/css-box-4/#propdef-margin-right) with 0, then solve for <a id="ref-for-propdef-left②④"></a>left.

3.  <a id="ref-for-propdef-margin-right⑨"></a>

    <a id="ref-for-propdef-margin-left⑨"></a>

    <a id="ref-for-propdef-left②⑤"></a>

    <a id="ref-for-propdef-right②⑤"></a>

    If [right](#propdef-right) is auto and [left](#propdef-left) is not auto, replace any auto on [margin-left](https://www.w3.org/TR/css-box-4/#propdef-margin-left) or [margin-right](https://www.w3.org/TR/css-box-4/#propdef-margin-right) with 0, then solve for <a id="ref-for-propdef-right②⑥"></a>right.

4.  <a id="ref-for-valdef-direction-rtl④"></a>

    <a id="ref-for-valdef-direction-ltr⑥"></a>

    <a id="ref-for-containing-block④⑤"></a>

    <a id="ref-for-propdef-margin-right①⓪"></a>

    <a id="ref-for-propdef-margin-left①⓪"></a>

    If at this point both [margin-left](https://www.w3.org/TR/css-box-4/#propdef-margin-left) and [margin-right](https://www.w3.org/TR/css-box-4/#propdef-margin-right) are still auto, solve the equation under the extra constraint that the two margins must get equal values, unless this would make them negative, in which case when the direction of the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) is [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr) ([rtl](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-rtl)), set <a id="ref-for-propdef-margin-left①①"></a>margin-left (<a id="ref-for-propdef-margin-right①①"></a>margin-right) to 0 and solve for <a id="ref-for-propdef-margin-right①②"></a>margin-right (<a id="ref-for-propdef-margin-left①②"></a>margin-left).

5.  If at this point there is an auto remaining, solve the equation for that value.

6.  <a id="ref-for-valdef-direction-ltr⑦"></a>

    <a id="ref-for-propdef-right②⑦"></a>

    <a id="ref-for-valdef-direction-rtl⑤"></a>

    <a id="ref-for-containing-block④⑥"></a>

    <a id="ref-for-propdef-direction⑧"></a>

    <a id="ref-for-propdef-left②⑥"></a>

    If at this point the values are over-constrained, ignore the value for either [left](#propdef-left) (in case the [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction) property of the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) is [rtl](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-rtl)) or [right](#propdef-right) (in case <a id="ref-for-propdef-direction⑨"></a>direction is [ltr](https://www.w3.org/TR/css-writing-modes-4/#valdef-direction-ltr)) and solve for that value.

### <a id="abs-non-replaced-height"></a>6.3.  The Height Of Absolutely Positioned, Non-Replaced Elements

For absolutely positioned elements, the used values of the vertical dimensions must satisfy this constraint:

<a id="ref-for-propdef-top⑨"></a>

<a id="ref-for-propdef-margin-top"></a>

<a id="ref-for-propdef-border-top-width"></a>

<a id="ref-for-propdef-padding-top"></a>

<a id="ref-for-propdef-height④"></a>

<a id="ref-for-propdef-padding-bottom"></a>

<a id="ref-for-propdef-border-bottom-width"></a>

<a id="ref-for-propdef-margin-bottom"></a>

<a id="ref-for-propdef-bottom⑤"></a>

<code>
		<a href="#propdef-top">top</a> + <span><a href="https://www.w3.org/TR/css-box-4/#propdef-margin-top">margin-top</a></span> + <span><a href="https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-width">border-top-width</a></span> + <span><a href="https://www.w3.org/TR/css-box-4/#propdef-padding-top">padding-top</a></span> + <a href="https://www.w3.org/TR/css-sizing-3/#propdef-height">height</a> +
		<span><a href="https://www.w3.org/TR/css-box-4/#propdef-padding-bottom">padding-bottom</a></span> + <span><a href="https://www.w3.org/TR/css-backgrounds-3/#propdef-border-bottom-width">border-bottom-width</a></span> + <span><a href="https://www.w3.org/TR/css-box-4/#propdef-margin-bottom">margin-bottom</a></span> + <a href="#propdef-bottom">bottom</a>
		= <span>height of containing block</span>
	</code>

<a id="ref-for-propdef-top①⓪"></a>

<a id="ref-for-propdef-height⑤"></a>

<a id="ref-for-propdef-bottom⑥"></a>

<a id="ref-for-valdef-top-auto②⑧"></a>

<a id="ref-for-propdef-margin-top①"></a>

<a id="ref-for-propdef-margin-bottom①"></a>

If all three of [top](#propdef-top), [height](https://www.w3.org/TR/css-sizing-3/#propdef-height), and [bottom](#propdef-bottom) are [auto](#valdef-top-auto): First set any <a id="ref-for-valdef-top-auto②⑨"></a>auto values for [margin-top](https://www.w3.org/TR/css-box-4/#propdef-margin-top) and [margin-bottom](https://www.w3.org/TR/css-box-4/#propdef-margin-bottom) to 0, then set <a id="ref-for-propdef-top①①"></a>top to the static position, and finally apply rule number <em>three</em> below.

<a id="ref-for-valdef-top-auto③⓪"></a>

<a id="ref-for-propdef-margin-top②"></a>

<a id="ref-for-propdef-margin-bottom②"></a>

<a id="ref-for-propdef-bottom⑦"></a>

If none of the three are [auto](#valdef-top-auto): If both [margin-top](https://www.w3.org/TR/css-box-4/#propdef-margin-top) and [margin-bottom](https://www.w3.org/TR/css-box-4/#propdef-margin-bottom) are <a id="ref-for-valdef-top-auto③①"></a>auto, solve the equation under the extra constraint that the two margins get equal values. If one of <a id="ref-for-propdef-margin-top③"></a>margin-top or <a id="ref-for-propdef-margin-bottom③"></a>margin-bottom is <a id="ref-for-valdef-top-auto③②"></a>auto, solve the equation for that value. If the values are over-constrained, ignore the value for [bottom](#propdef-bottom) and solve for that value.

<a id="ref-for-valdef-top-auto③③"></a>

<a id="ref-for-propdef-margin-top④"></a>

<a id="ref-for-propdef-margin-bottom④"></a>

Otherwise, set [auto](#valdef-top-auto) values for [margin-top](https://www.w3.org/TR/css-box-4/#propdef-margin-top) and [margin-bottom](https://www.w3.org/TR/css-box-4/#propdef-margin-bottom) to 0, and pick one of the following six rules that apply.

1.  <a id="ref-for-propdef-top①②"></a>

    <a id="ref-for-propdef-height⑥"></a>

    <a id="ref-for-valdef-top-auto③④"></a>

    <a id="ref-for-propdef-bottom⑧"></a>

    If [top](#propdef-top) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) are [auto](#valdef-top-auto) and [bottom](#propdef-bottom) is not <a id="ref-for-valdef-top-auto③⑤"></a>auto, then the height is based on the [Auto heights for block formatting context roots](https://www.w3.org/TR/CSS2/visudet.html#root-height), and solve for <a id="ref-for-propdef-top①③"></a>top.

2.  <a id="ref-for-propdef-top①④"></a>

    <a id="ref-for-propdef-bottom⑨"></a>

    <a id="ref-for-valdef-top-auto③⑥"></a>

    <a id="ref-for-propdef-height⑦"></a>

    If [top](#propdef-top) and [bottom](#propdef-bottom) are [auto](#valdef-top-auto) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) is not <a id="ref-for-valdef-top-auto③⑦"></a>auto, then set <a id="ref-for-propdef-top①⑤"></a>top to the static position, then solve for <a id="ref-for-propdef-bottom①⓪"></a>bottom.

3.  <a id="ref-for-propdef-height⑧"></a>

    <a id="ref-for-propdef-bottom①①"></a>

    <a id="ref-for-valdef-top-auto③⑧"></a>

    <a id="ref-for-propdef-top①⑥"></a>

    If [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) and [bottom](#propdef-bottom) are [auto](#valdef-top-auto) and [top](#propdef-top) is not <a id="ref-for-valdef-top-auto③⑨"></a>auto, then the height is based on the [Auto heights for block formatting context roots](https://www.w3.org/TR/CSS2/visudet.html#root-height), and solve for <a id="ref-for-propdef-bottom①②"></a>bottom.

4.  <a id="ref-for-propdef-top①⑦"></a>

    <a id="ref-for-valdef-top-auto④⓪"></a>

    <a id="ref-for-propdef-height⑨"></a>

    <a id="ref-for-propdef-bottom①③"></a>

    If [top](#propdef-top) is [auto](#valdef-top-auto), [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) and [bottom](#propdef-bottom) are not <a id="ref-for-valdef-top-auto④①"></a>auto, then solve for <a id="ref-for-propdef-top①⑧"></a>top.

5.  <a id="ref-for-propdef-height①⓪"></a>

    <a id="ref-for-valdef-top-auto④②"></a>

    <a id="ref-for-propdef-top①⑨"></a>

    <a id="ref-for-propdef-bottom①④"></a>

    If [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) is [auto](#valdef-top-auto), [top](#propdef-top) and [bottom](#propdef-bottom) are not <a id="ref-for-valdef-top-auto④③"></a>auto, then solve for <a id="ref-for-propdef-height①①"></a>height.

6.  <a id="ref-for-propdef-bottom①⑤"></a>

    <a id="ref-for-valdef-top-auto④④"></a>

    <a id="ref-for-propdef-top②⓪"></a>

    <a id="ref-for-propdef-height①②"></a>

    If [bottom](#propdef-bottom) is [auto](#valdef-top-auto), [top](#propdef-top) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) are not <a id="ref-for-valdef-top-auto④⑤"></a>auto, then solve for <a id="ref-for-propdef-bottom①⑥"></a>bottom.

### <a id="abs-replaced-height"></a>6.4.  The Height Of Absolutely Positioned, Replaced Elements

<a id="ref-for-propdef-height①③"></a>

<a id="ref-for-propdef-width②②"></a>

<a id="ref-for-valdef-top-auto④⑥"></a>

If [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) and [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) both have computed values of [auto](#valdef-top-auto) and the element also has an intrinsic height, then that intrinsic height is the used value of <a id="ref-for-propdef-height①④"></a>height.

<a id="ref-for-propdef-height①⑤"></a>

<a id="ref-for-valdef-top-auto④⑦"></a>

Otherwise, if [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) has a computed value of [auto](#valdef-top-auto) and the element has an intrinsic ratio then the used value of <a id="ref-for-propdef-height①⑥"></a>height is:

`(used width) / (intrinsic ratio)`

<a id="ref-for-propdef-height①⑦"></a>

<a id="ref-for-valdef-top-auto④⑧"></a>

Otherwise, if [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) has a computed value of [auto](#valdef-top-auto) and the element has an intrinsic height, then that intrinsic height is the used value of <a id="ref-for-propdef-height①⑧"></a>height.

<a id="ref-for-propdef-height①⑨"></a>

<a id="ref-for-valdef-top-auto④⑨"></a>

Otherwise, if [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) has a computed value of [auto](#valdef-top-auto), but none of the conditions above are met, then the used value of <a id="ref-for-propdef-height②⓪"></a>height must be set to the height of the largest rectangle that has a 2:1 ratio, has a height not greater than 150px, and has a width not greater than the device width.

<a id="ref-for-propdef-height②①"></a>

After establishing the [height](https://www.w3.org/TR/css-sizing-3/#propdef-height), in order to position the replaced element, apply the following rules as appropriate.

1.  <a id="ref-for-propdef-top②①"></a>

    <a id="ref-for-propdef-bottom①⑦"></a>

    <a id="ref-for-valdef-top-auto⑤⓪"></a>

    If both [top](#propdef-top) and [bottom](#propdef-bottom) have the value [auto](#valdef-top-auto), replace <a id="ref-for-propdef-top②②"></a>top with the element’s static position.

2.  <a id="ref-for-propdef-bottom①⑧"></a>

    <a id="ref-for-valdef-top-auto⑤①"></a>

    <a id="ref-for-propdef-margin-top⑤"></a>

    <a id="ref-for-propdef-margin-bottom⑤"></a>

    If [bottom](#propdef-bottom) is [auto](#valdef-top-auto), replace any <a id="ref-for-valdef-top-auto⑤②"></a>auto on [margin-top](https://www.w3.org/TR/css-box-4/#propdef-margin-top) or [margin-bottom](https://www.w3.org/TR/css-box-4/#propdef-margin-bottom) with 0.

3.  <a id="ref-for-propdef-margin-top⑥"></a>

    <a id="ref-for-propdef-margin-bottom⑥"></a>

    <a id="ref-for-valdef-top-auto⑤③"></a>

    If at this point both [margin-top](https://www.w3.org/TR/css-box-4/#propdef-margin-top) and [margin-bottom](https://www.w3.org/TR/css-box-4/#propdef-margin-bottom) are still [auto](#valdef-top-auto), solve the equation under the extra constraint that the two margins must get equal values.

4.  <a id="ref-for-valdef-top-auto⑤④"></a>

    If at this point there is only one [auto](#valdef-top-auto) remaining, solve the equation for that value.

5.  <a id="ref-for-propdef-bottom①⑨"></a>

    If at this point the values are over-constrained, ignore the value for [bottom](#propdef-bottom) and solve for that value.

## <a id="comparison"></a>7.  Informative Comparison of Normal Flow, Floats, and Positioning

<em>This section is not normative.</em>

<a id="ref-for-normal-flow①"></a>

<a id="ref-for-relative-position⑧"></a>

<a id="ref-for-float"></a>

<a id="ref-for-absolute-position①⑥"></a>

To illustrate the differences between [normal flow](https://www.w3.org/TR/CSS2/visuren.html#normal-flow), [relative positioning](#relative-position), [floats](https://www.w3.org/TR/css-page-floats-3/#float), and [absolute positioning](#absolute-position), we provide a series of examples based on the following HTML:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5752760c"></a>
>
> ```html
> <!DOCTYPE html>
> <html>
>     <head>
>         <title>Comparison of positioning schemes</title>
>         <style>
>           body { display: block; font-size:12px; line-height: 200%;
>                   width: 400px; height: 400px }
>           p    { display: block }
>           span { display: inline }
>         </style>
>     </head>
>     <body>
>     <p>
>         Beginning of p contents.
>         <span id="outer"> Start of outer contents.
>         <span id="inner"> Inner contents.</span>
>         End of outer contents.</span>
>         End of p contents.
>     </p>
>     </body>
> </html>
> ```
<a id="ref-for-normal-flow②"></a>

The final positions of boxes generated by the <em>outer</em> and <em>inner</em> elements vary in each example. In each illustration, the numbers to the left of the illustration indicate the [normal flow](https://www.w3.org/TR/CSS2/visuren.html#normal-flow) position of the double-spaced (for clarity) lines.

<a id="ref-for-positioning-scheme⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The diagrams in this section are illustrative and not to scale. They are meant to highlight the differences between the various [positioning schemes](#positioning-scheme), and are not intended to be reference renderings of the examples given.

### <a id="comp-normal-flow"></a>7.1.  Normal Flow Example

<a id="ref-for-normal-flow③"></a>

Consider the following CSS declarations for <em>outer</em> and <em>inner</em> that do not alter the [normal flow](https://www.w3.org/TR/CSS2/visuren.html#normal-flow) of boxes:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d045d8f9"></a>
>
> ```css
> #outer { color: red }
> #inner { color: blue }
> ```
<a id="ref-for-containing-block④⑦"></a>

The P element contains all inline content: [anonymous inline text](https://www.w3.org/TR/CSS2/visuren.html#anonymous) and two SPAN elements. Therefore, all of the content will be laid out in an inline formatting context, within a [containing block](https://www.w3.org/TR/css-display-4/#containing-block) established by the P element, producing something like:

![Image illustrating the normal flow of text between parent and sibling boxes.](https://www.w3.org/TR/2025/WD-css-position-3-20251007/images/flow-generic.png)

All of the text within the P’s containing block flows together as continuous text, even though it’s located in separated nested elements.

### <a id="comp-relpos"></a>7.2.  Relative Positioning Example

<a id="ref-for-relative-position⑨"></a>

To see the effect of [relative positioning](#relative-position), we specify:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bf500659"></a>
>
> ```css
> #outer { position: relative; top: -12px; color: red }
> #inner { position: relative; top: 12px; color: blue }
> ```
<a id="ref-for-normal-flow④"></a>

Text flows normally up to the <em>outer</em> element. The <em>outer</em> text is then flowed into its [normal flow](https://www.w3.org/TR/CSS2/visuren.html#normal-flow) position and dimensions at the end of line 1. Then, the inline boxes containing the text (distributed over three lines) are shifted as a unit by -12px (upwards).

The contents of <em>inner</em>, as a child of <em>outer</em>, would normally flow immediately after the words "of outer contents" (on line 1.5). However, the <em>inner</em> contents are themselves offset relative to the <em>outer</em> contents by 12px (downwards), back to their original position on line 2.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that the content following <em>outer</em> is not affected by the relative positioning of <em>outer</em>.

![](https://www.w3.org/TR/2025/WD-css-position-3-20251007/images/flow-relative.png)

The result is identical to normal flow, except that the "outer" text is shifted 12px upward, without affecting the flow of the "body" or "inner" text.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note also that had the offset of <em>outer</em> been -24px, the text of <em>outer</em> and the body text would have overlapped.

### <a id="comp-floating"></a>7.3.  Floating Example

Now consider the effect of [floating](https://www.w3.org/TR/CSS2/visuren.html#floats) the <em>inner</em> element’s text to the right by means of the following rules:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7c19fd19"></a>
>
> ```css
> #outer { color: red }
> #inner { float: right; width: 130px; color: blue }
> ```
<a id="ref-for-float①"></a>

<a id="ref-for-propdef-width②③"></a>

Text flows normally up to the <em>inner</em> box, which is pulled out of the flow and [floated](https://www.w3.org/TR/css-page-floats-3/#float) to the right margin (its [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) has been assigned explicitly). Line boxes to the left of the float are shortened, and the document’s remaining text flows into them.

![Image illustrating the effects of floating a box.](https://www.w3.org/TR/2025/WD-css-position-3-20251007/images/flow-float.png)

The "inner" text lays out in an independent box on the right, causing the remaining "body" and "outer" text to flow around it.

<a id="ref-for-propdef-clear①"></a>

To show the effect of the [clear](https://www.w3.org/TR/CSS2/visuren.html#propdef-clear) property, we add a <em>sibling</em> element to the example:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e3784dd4"></a>
>
> ```html
> <!DOCTYPE html>
> <html>
>     <head>
>         <title>Comparison of positioning schemes II</title>
>         <style>
>           #inner { float: right; width: 130px; color: blue }
>           #sibling { color: red }
>         </style>
>     </head>
>     <body>
>     <p>
>         Beginning of p contents.
>         <span id="outer"> Start of outer contents.
>         <span id="inner"> Inner contents.</span>
>         <span id="sibling"> Sibling contents.</span>
>         End of outer contents.</span>
>         End of p contents.
>     </p>
>     </body>
> </html>
> ```
These styles cause the <em>inner</em> box to float to the right, as before, and the document’s remaining text to flow into the vacated space:

![Image illustrating the effects of floating a box without setting the clear property to control the flow of text around the box.](https://www.w3.org/TR/2025/WD-css-position-3-20251007/images/flow-clear.png)

Identical to the previous example, save that there is now "sibling" text flowing with the "body" and "outer" text.

<a id="ref-for-propdef-clear②"></a>

<a id="ref-for-propdef-right②⑧"></a>

<a id="ref-for-float②"></a>

However, if the [clear](https://www.w3.org/TR/CSS2/visuren.html#propdef-clear) property on the <em>sibling</em> element is set to [right](#propdef-right) (i.e., the generated <em>sibling</em> box will not accept a position next to [floating](https://www.w3.org/TR/css-page-floats-3/#float) boxes to its right), the <em>sibling</em> content begins to flow below the float:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-54c91152"></a>
>
> ```css
> #inner { float: right; width: 130px; color: blue }
> #sibling { clear: right; color: red }
> ```
![Image illustrating the effects of floating an element with setting the clear property to control the flow of text around the element.](https://www.w3.org/TR/2025/WD-css-position-3-20251007/images/flow-clear2.png)

Now the "sibling" text moves down to below the "inner" text’s box, leaving blank space behind. The text following the "sibling" text flows after it as normal.

### <a id="comp-abspos"></a>7.4.  Absolute Positioning Example

Next, we consider the effect of absolute positioning. Consider the following CSS declarations for <em>outer</em> and <em>inner</em>:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-90defcbb"></a>
>
> ```css
> #outer {
>     position: absolute;
>     top: 200px; left: 200px;
>     width: 200px;
>     color: red;
> }
> #inner { color: blue }
> ```
<a id="ref-for-containing-block④⑧"></a>

which cause the top of the <em>outer</em> box to be positioned with respect to its [containing block](https://www.w3.org/TR/css-display-4/#containing-block). The <a id="ref-for-containing-block④⑨"></a>containing block for a positioned box is established by the nearest positioned ancestor (or, if none exists, the initial containing block, as in our example). The top side of the <em>outer</em> box is 200px below the top of the <a id="ref-for-containing-block⑤⓪"></a>containing block and the left side is 200px from the left side. The child box of <em>outer</em> is flowed normally with respect to its parent.

![Image illustrating the effects of absolutely positioning a box.](https://www.w3.org/TR/2025/WD-css-position-3-20251007/images/flow-absolute.png)

All of the text within \#outer (the "outer" and "inner" text) moves down to an independent box in the lower right corner. The two halves of "body" text flow together.

<a id="ref-for-propdef-position①①"></a>

<a id="ref-for-valdef-position-relative②"></a>

<a id="ref-for-propdef-top②③"></a>

<a id="ref-for-propdef-left②⑦"></a>

The following example shows an absolutely positioned box that is a child of a relatively positioned box. Although the parent <em>outer</em> box is not actually offset, setting its [position](#propdef-position) property to [relative](#valdef-position-relative) means that its box may serve as the containing block for positioned descendants. Since the <em>outer</em> box is an inline box that is split across several lines, the first inline box’s top and left edges (depicted by thick dashed lines in the illustration below) serve as references for [top](#propdef-top) and [left](#propdef-left) offsets.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f34c6317"></a>
>
> ```css
> #outer {
>     position: relative;
>     color: red
> }
> #inner {
>     position: absolute;
>     top: 200px; left: -100px;
>     height: 130px; width: 130px;
>     color: blue;
> }
> ```
This results in something like the following:

![Image illustrating the effects of absolutely positioning a box with respect to a containing block.](https://www.w3.org/TR/2025/WD-css-position-3-20251007/images/flow-abs-rel.png)

The "inner" text is positioned in an independent box, relative to the top-left corner of the start of the "outer" text.

If we do not position the <em>outer</em> box:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-100563b8"></a>
>
> ```css
> #outer { color: red }
> #inner {
>     position: absolute;
>     top: 200px; left: -100px;
>     height: 130px; width: 130px;
>     color: blue;
> }
> ```
<a id="ref-for-containing-block⑤①"></a>

the [containing block](https://www.w3.org/TR/css-display-4/#containing-block) for <em>inner</em> becomes the initial containing block (in our example). The following illustration shows where the <em>inner</em> box would end up in this case.

![Image illustrating the effects of absolutely positioning a box with respect to a containing block established by a normally positioned parent.](https://www.w3.org/TR/2025/WD-css-position-3-20251007/images/flow-static.png)

Same as before, except now the "inner text" is positioned relative to the top-left corner of the page itself.

Relative and absolute positioning may be used to implement change bars, as shown in the following example. The following fragment:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3b789003"></a>
>
> ```html
> <p style="position: relative; margin-right: 10px; left: 10px;">
>   I used two red hyphens to serve as a change bar. They
>   will "float" to the left of the line containing THIS
>   <span style="position: absolute; top: auto; left: -1em; color: red;">--</span>
>   word.
> </p>
> ```
might result in something like:

![Image illustrating the use of floats to create a changebar effect.](https://www.w3.org/TR/2025/WD-css-position-3-20251007/images/changebar.png)

The two red hyphens, indicating a change, sit in the left margin of the page on the line containing the word "THIS", regardless of what line that ends up being.

<a id="ref-for-containing-block⑤②"></a>

First, the paragraph (whose [containing block](https://www.w3.org/TR/css-display-4/#containing-block) sides are shown in the illustration) is flowed normally. Then it is offset 10px from the left edge of the <a id="ref-for-containing-block⑤③"></a>containing block (thus, a right margin of 10px has been reserved in anticipation of the offset). The two hyphens acting as change bars are taken out of the flow and positioned at the current line (due to 'top: auto'), -1em from the left edge of its containing block (established by the P in its final position). The result is that the change bars seem to "float" to the left of the current line.

## <a id="ack"></a>8.  Acknowledgments

This module would not have been possible without input and support from many helpful people. Thanks to Rossen Atanassov, Bert Bos, Oriol Brufau, Tantek Çelik, Arron Eicholz Sylvain Galineau, John Jansen, Chris Jones, Ian Kilpatrick, Anton Prowse.

## <a id="changes"></a>Changes

Significant changes since the [11 March 2025 Working Draft](https://www.w3.org/TR/2025/WD-css-position-3-20250311/):

- <a id="ref-for-initial-fixed-containing-block①"></a>

  <a id="ref-for-initial-containing-block④"></a>

  Defined the [initial fixed containing block](#initial-fixed-containing-block) and its relationship to the [initial containing block](https://www.w3.org/TR/css-display-4/#initial-containing-block), to give this concept a name.

See also [Previous Changes](https://www.w3.org/TR/2025/WD-css-position-3-20250311/#changes).

## <a id="privacy"></a> Privacy Considerations

This specification introduces no new privacy considerations.

## <a id="security"></a> Security Considerations

<a id="ref-for-propdef-margin②"></a>

<a id="ref-for-propdef-transform②"></a>

If an attacker is able to inject arbitrary CSS, positioned layout can make it easier to position elements the attacker has control of over arbitrary other elements of the page, potentially tricking users of the page. (There are many routes to this attack: negative [margin](https://www.w3.org/TR/css-box-4/#propdef-margin), [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform), etc. Don’t let people apply arbitrary CSS to bits of your page.)

<a id="ref-for-propdef-position①②"></a>

[position: fixed](#propdef-position) can allow a page to emulate modal dialogs, potentially tricking a user into thinking they’re interacting with the user agent and entering in sensitive information that they page can then capture. User agents must ensure that their native dialogs are positioned in ways that the page cannot emulate; in particular, that at least some of the dialog is outside the "poisoned pixels" that web content can paint to.

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

- absolute
  - [dfn for](#absolute-position) , in § 2
  - [value for position](#valdef-position-absolute), in § 2
- [absolutely](#absolute-position), in § 2
- [absolutely position](#absolute-position), in § 2
- [absolutely-positioned](#absolute-position), in § 2
- [absolutely positioned box](#absolute-position), in § 2
- [absolutely positioned element](#absolute-position), in § 2
- [absolute position](#absolute-position), in § 2
- [absolute-position containing block](#absolute-position-containing-block), in § 3.5
- [absolute positioning containing block](#absolute-positioning-containing-block), in § 2
- [absolute positioning layout](#absolute-positioning-layout), in § 4
- [absolute positioning layout model](#absolute-positioning-layout), in § 4
- [auto](#valdef-top-auto), in § 3.1
- [bottom](#propdef-bottom), in § 3.1
- fixed
  - [dfn for](#fixed-position) , in § 2
  - [value for position](#valdef-position-fixed), in § 2
- [fixed position](#fixed-position), in § 2
- [fixed-positioned](#fixed-position), in § 2
- [fixed-positioned box](#fixed-position), in § 2
- [fixed positioning containing block](#fixed-positioning-containing-block), in § 2.1
- [initial fixed containing block](#initial-fixed-containing-block), in § 2.1
- [inset](#propdef-inset), in § 3.2
- [inset-block](#propdef-inset-block), in § 3.2
- [inset-block-end](#propdef-inset-block-end), in § 3.1
- [inset-block-start](#propdef-inset-block-start), in § 3.1
- [inset-inline](#propdef-inset-inline), in § 3.2
- [inset-inline-end](#propdef-inset-inline-end), in § 3.1
- [inset-inline-start](#propdef-inset-inline-start), in § 3.1
- [inset-modified containing block](#inset-modified-containing-block), in § 3.5
- [inset properties](#inset-properties), in § 3
- [left](#propdef-left), in § 3.1
- [\<length\>](#valdef-top-length), in § 3.1
- [original containing block](#original-containing-block), in § 2.1.1
- [\<percentage\>](#valdef-top-percentage), in § 3.1
- [position](#propdef-position), in § 2
- [position box](#position-box), in § 3.4
- [positioned](#positioned-box), in § 2
- [positioned box](#positioned-box), in § 2
- [positioning scheme](#positioning-scheme), in § 2
- relative
  - [dfn for](#relative-position) , in § 2
  - [value for position](#valdef-position-relative), in § 2
- [relatively](#relative-position), in § 2
- [relatively position](#relative-position), in § 2
- [relatively-positioned](#relative-position), in § 2
- [relatively positioned box](#relative-position), in § 2
- [relative position](#relative-position), in § 2
- [right](#propdef-right), in § 3.1
- [static](#valdef-position-static), in § 2
- [static position](#static-position), in § 3.5.2
- [static-position containing block](#static-position-containing-block), in § 3.5.2
- [static-position rectangle](#static-position-rectangle), in § 3.5.2
- sticky
  - [dfn for](#sticky-position) , in § 2
  - [value for position](#valdef-position-sticky), in § 2
- [sticky position](#sticky-position), in § 2
- [sticky-positioned](#sticky-position), in § 2
- [sticky-positioned box](#sticky-position), in § 2
- [sticky view rectangle](#sticky-view-rectangle), in § 3.4
- [stronger inset](#stronger-inset), in § 3.5.1
- [top](#propdef-top), in § 3.1
- [weaker inset](#weaker-inset), in § 3.5.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ALIGN-3\] defines the following terms:
  - <a id="fb5c7e3f"></a>alignment container
  - <a id="dc2ecc7a"></a>alignment subject
  - <a id="4d2cf2cf"></a>baseline alignment
  - <a id="567f0e7d"></a>box alignment properties
  - <a id="515ec31f"></a>center
  - <a id="fd3ce740"></a>normal
  - <a id="d767ea3c"></a>overflow alignment
  - <a id="5f80981e"></a>self-alignment
  - <a id="34ae2cc3"></a>self-alignment properties
  - <a id="9907986d"></a>self-end
  - <a id="096f263d"></a>self-start
  - <a id="35e80fd9"></a>start
  - <a id="598fa031"></a>stretch
- \[CSS-ANCHOR-POSITION-1\] defines the following terms:
  - <a id="5013f3ce"></a>position-area
  - <a id="d443f3b8"></a>position-area grid
  - <a id="e85f9c40"></a>position-try
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="8218676f"></a>background-attachment
  - <a id="bb899432"></a>border-bottom-width
  - <a id="aaf0980c"></a>border-left-width
  - <a id="47e9abf9"></a>border-right-width
  - <a id="051410c7"></a>border-top-width
- \[CSS-BOX-4\] defines the following terms:
  - <a id="85c399c0"></a>border box
  - <a id="3e6781f5"></a>border edge
  - <a id="cba8daea"></a>content edge
  - <a id="253362bb"></a>margin
  - <a id="0778a939"></a>margin box
  - <a id="16ff1cf8"></a>margin edge
  - <a id="ba64c9f5"></a>margin-bottom
  - <a id="836161df"></a>margin-left
  - <a id="76c02e00"></a>margin-right
  - <a id="58404105"></a>margin-top
  - <a id="7c998c9a"></a>outer edge
  - <a id="093a0ff1"></a>padding edge
  - <a id="c8473aa4"></a>padding-bottom
  - <a id="c22cb630"></a>padding-left
  - <a id="0a9e7084"></a>padding-right
  - <a id="f72f0a02"></a>padding-top
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="d65c0e81"></a>box fragment
  - <a id="972b685d"></a>fragmentation break
  - <a id="4904f647"></a>fragmentation container
  - <a id="1e358503"></a>fragmented flow
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="6b448e93"></a>initial value
  - <a id="36261173"></a>longhand
  - <a id="8f27be0f"></a>longhand property
  - <a id="e14541aa"></a>shorthand
  - <a id="980ac56a"></a>shorthand property
  - <a id="1a2b1083"></a>used value
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="5dfeee7f"></a>contain
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="a015488b"></a>block-level
  - <a id="91b1f11d"></a>block-level box
  - <a id="431a9cad"></a>blockify
  - <a id="95bf6f06"></a>box
  - <a id="0923db9e"></a>containing block
  - <a id="b28a080f"></a>containing block chain
  - <a id="e8c16097"></a>display
  - <a id="8be4ac1c"></a>establish an independent formatting context
  - <a id="ae223697"></a>formatting context
  - <a id="6658d41f"></a>in-flow
  - <a id="d1ebdd75"></a>initial containing block
  - <a id="f089a6e1"></a>inline box
  - <a id="febab3e8"></a>inline-level box
  - <a id="e89ddbcb"></a>non-replaced
  - <a id="1dda072a"></a>out of flow
  - <a id="a9db5d6d"></a>replaced element
  - <a id="143ef105"></a>root element
  - <a id="da0def5c"></a>table-cell
  - <a id="d49e9025"></a>table-column
  - <a id="6531a92c"></a>table-column-group
  - <a id="56e2d3bb"></a>table-footer-group
  - <a id="37fb4999"></a>table-header-group
  - <a id="8c1398e7"></a>table-row
  - <a id="201aba90"></a>table-row-group
- \[CSS-FLEXBOX-1\] defines the following terms:
  - <a id="4e512e00"></a>cross axis
  - <a id="cc7f0a64"></a>flex container
  - <a id="98f2297b"></a>main axis
- \[CSS-GRID-2\] defines the following terms:
  - <a id="7086fc61"></a>grid area
  - <a id="df72a52c"></a>grid container
  - <a id="88fad469"></a>grid-placement property
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="a9330658"></a>line box
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="a3972067"></a>initial scroll position
  - <a id="61e80d9c"></a>nearest scrollport
  - <a id="a3cabdb1"></a>scroll container
  - <a id="3ed7991e"></a>scrollable overflow area
  - <a id="700ea31d"></a>scrollport
- \[CSS-PAGE-3\] defines the following terms:
  - <a id="1fe10a71"></a>page area
- \[CSS-PAGE-FLOATS-3\] defines the following terms:
  - <a id="06eea7ac"></a>float
- \[CSS-POSITION-4\] defines the following terms:
  - <a id="e46d20c1"></a>top layer
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="c20b5ff5"></a>auto
  - <a id="e9c67130"></a>automatic minimum size
  - <a id="37f6dbd7"></a>automatic size
  - <a id="c1c732b9"></a>available space
  - <a id="66f218c1"></a>definite
  - <a id="f15ee6fc"></a>fit-content size
  - <a id="5ad01cca"></a>height
  - <a id="8a39af7f"></a>max-content size
  - <a id="6d275904"></a>maximum size
  - <a id="4405c984"></a>minimum size
  - <a id="dd09245c"></a>preferred size
  - <a id="97ac8088"></a>stretch-fit size
  - <a id="49731d1d"></a>width
- \[CSS-SIZING-4\] defines the following terms:
  - <a id="a15ecdf0"></a>ratio-dependent axis
  - <a id="21da52e0"></a>ratio-determining axis
- \[CSS-TABLES-3\] defines the following terms:
  - <a id="1b178ec1"></a>table wrapper box
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="e7c6bf78"></a>transform
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="35091ccc"></a>dynamic viewport size
  - <a id="3bafef5e"></a>{A,B}
  - <a id="4eb9d37e"></a>\|
- \[CSS-WILL-CHANGE-1\] defines the following terms:
  - <a id="81ec7485"></a>will-change
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="fb688f4f"></a>direction
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="b8dade0f"></a>block axis
  - <a id="1118d052"></a>block-start
  - <a id="e112902f"></a>end
  - <a id="303c8d41"></a>flow-relative
  - <a id="49eecea3"></a>horizontal writing mode
  - <a id="a6eb24bb"></a>inline axis
  - <a id="4da3b716"></a>inline-end
  - <a id="0da67e16"></a>inline-start
  - <a id="0ad9204c"></a>line-over
  - <a id="401cafe5"></a>line-under
  - <a id="24c75626"></a>ltr
  - <a id="e1f6e4b9"></a>physical
  - <a id="dea08a34"></a>rtl
  - <a id="90c7548c"></a>start
  - <a id="eb6008ce"></a>writing mode
- \[CSS2\] defines the following terms:
  - <a id="d332e4ec"></a>auto
  - <a id="019a586e"></a>clear
  - <a id="da486c10"></a>float
  - <a id="9927f4ce"></a>none
  - <a id="f7c9cc1a"></a>normal flow
  - <a id="b8c9eb8a"></a>stacking context
  - <a id="e12287dd"></a>viewport
  - <a id="1848f1d3"></a>z-index
- \[CSSOM-1\] defines the following terms:
  - <a id="fc19454a"></a>resolved value
- \[CSSOM-VIEW-1\] defines the following terms:
  - <a id="bc23ba5b"></a>layout viewport
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="9e462db5"></a>continuous media
  - <a id="23af89d0"></a>paged media

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-align-3"></a>\[CSS-ALIGN-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Box Alignment Module Level 3](https://www.w3.org/TR/css-align-3/). 11 March 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-align-3&#x2F;](https://www.w3.org/TR/css-align-3/)

<a id="biblio-css-anchor-position-1"></a>\[CSS-ANCHOR-POSITION-1\]  
Tab Atkins Jr.; Elika Etemad; Ian Kilpatrick. [CSS Anchor Positioning](https://www.w3.org/TR/css-anchor-position-1/). 9 May 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-anchor-position-1&#x2F;](https://www.w3.org/TR/css-anchor-position-1/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 19 December 2024. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-flexbox-1"></a>\[CSS-FLEXBOX-1\]  
Tab Atkins Jr.; et al. [CSS Flexible Box Layout Module Level 1](https://www.w3.org/TR/css-flexbox-1/). 19 November 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-flexbox-1&#x2F;](https://www.w3.org/TR/css-flexbox-1/)

<a id="biblio-css-grid-1"></a>\[CSS-GRID-1\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 1](https://www.w3.org/TR/css-grid-1/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-1&#x2F;](https://www.w3.org/TR/css-grid-1/)

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Elika Etemad. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 18 December 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-logical-1"></a>\[CSS-LOGICAL-1\]  
Rossen Atanassov; Elika Etemad. [CSS Logical Properties and Values Level 1](https://www.w3.org/TR/css-logical-1/). 27 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-logical-1&#x2F;](https://www.w3.org/TR/css-logical-1/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 29 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-page-3"></a>\[CSS-PAGE-3\]  
Elika Etemad. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 14 September 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-css-page-floats-3"></a>\[CSS-PAGE-FLOATS-3\]  
Johannes Wilm. [CSS Page Floats](https://www.w3.org/TR/css-page-floats-3/). 15 September 2015. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-floats-3&#x2F;](https://www.w3.org/TR/css-page-floats-3/)

<a id="biblio-css-position-4"></a>\[CSS-POSITION-4\]  
[CSS Positioned Layout Module Level 4](https://www.w3.org/TR/css-position-4/). 8 July 2025. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-4&#x2F;](https://www.w3.org/TR/css-position-4/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-sizing-4"></a>\[CSS-SIZING-4\]  
Tab Atkins Jr.; Elika Etemad; Jen Simmons. [CSS Box Sizing Module Level 4](https://www.w3.org/TR/css-sizing-4/). 20 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-4&#x2F;](https://www.w3.org/TR/css-sizing-4/)

<a id="biblio-css-tables-3"></a>\[CSS-TABLES-3\]  
François Remy; Greg Whitworth; David Baron. [CSS Table Module Level 3](https://www.w3.org/TR/css-tables-3/). 27 July 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-tables-3&#x2F;](https://www.w3.org/TR/css-tables-3/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-cssom-view-1"></a>\[CSSOM-VIEW-1\]  
Simon Fraser; Emilio Cobos Álvarez. [CSSOM View Module](https://www.w3.org/TR/cssom-view-1/). 16 September 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-view-1&#x2F;](https://www.w3.org/TR/cssom-view-1/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 18 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 17 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-2&#x2F;](https://www.w3.org/TR/css-contain-2/)

<a id="biblio-css-will-change-1"></a>\[CSS-WILL-CHANGE-1\]  
Tab Atkins Jr.. [CSS Will Change Module Level 1](https://www.w3.org/TR/css-will-change-1/). 5 May 2022. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-will-change-1&#x2F;](https://www.w3.org/TR/css-will-change-1/)

## <a id="property-index"></a>Property Index

| Name                | Value                                             | Initial | Applies to                                              | Inh. | %ages                                        | Anim­ation type         | Canonical order | Com­puted value                                             | Logical property group |
|---------------------|---------------------------------------------------|---------|---------------------------------------------------------|------|----------------------------------------------|------------------------|-----------------|------------------------------------------------------------|------------------------|
| <strong><span><a id="ref-for-propdef-bottom②⓪"></a></span><a href="#propdef-bottom">bottom</a>&#xA;      </strong> | auto \| \<length-percentage\>                     | auto    | positioned elements                                     | no   | refer to size of containing block; see prose | by computed value type | per grammar     | the keyword auto or a computed \<length-percentage\> value | inset                  |
| <strong><span><a id="ref-for-propdef-inset⑥"></a></span><a href="#propdef-inset">inset</a>&#xA;      </strong> | \<'top'\>{1,4}                                    | auto    | positioned elements                                     | no   | see individual properties                    | by computed value type | per grammar     | see individual properties                                  |                        |
| <strong><span><a id="ref-for-propdef-inset-block③"></a></span><a href="#propdef-inset-block">inset-block</a>&#xA;      </strong> | \<'top'\>{1,2}                                    | auto    | positioned elements                                     | no   | see individual properties                    | by computed value type | per grammar     | see individual properties                                  |                        |
| <strong><span><a id="ref-for-propdef-inset-block-end③"></a></span><a href="#propdef-inset-block-end">inset-block-end</a>&#xA;      </strong> | auto \| \<length-percentage\>                     | auto    | positioned elements                                     | no   | refer to size of containing block; see prose | by computed value type | per grammar     | the keyword auto or a computed \<length-percentage\> value | inset                  |
| <strong><span><a id="ref-for-propdef-inset-block-start③"></a></span><a href="#propdef-inset-block-start">inset-block-start</a>&#xA;      </strong> | auto \| \<length-percentage\>                     | auto    | positioned elements                                     | no   | refer to size of containing block; see prose | by computed value type | per grammar     | the keyword auto or a computed \<length-percentage\> value | inset                  |
| <strong><span><a id="ref-for-propdef-inset-inline③"></a></span><a href="#propdef-inset-inline">inset-inline</a>&#xA;      </strong> | \<'top'\>{1,2}                                    | auto    | positioned elements                                     | no   | see individual properties                    | by computed value type | per grammar     | see individual properties                                  |                        |
| <strong><span><a id="ref-for-propdef-inset-inline-end③"></a></span><a href="#propdef-inset-inline-end">inset-inline-end</a>&#xA;      </strong> | auto \| \<length-percentage\>                     | auto    | positioned elements                                     | no   | refer to size of containing block; see prose | by computed value type | per grammar     | the keyword auto or a computed \<length-percentage\> value | inset                  |
| <strong><span><a id="ref-for-propdef-inset-inline-start③"></a></span><a href="#propdef-inset-inline-start">inset-inline-start</a>&#xA;      </strong> | auto \| \<length-percentage\>                     | auto    | positioned elements                                     | no   | refer to size of containing block; see prose | by computed value type | per grammar     | the keyword auto or a computed \<length-percentage\> value | inset                  |
| <strong><span><a id="ref-for-propdef-left②⑧"></a></span><a href="#propdef-left">left</a>&#xA;      </strong> | auto \| \<length-percentage\>                     | auto    | positioned elements                                     | no   | refer to size of containing block; see prose | by computed value type | per grammar     | the keyword auto or a computed \<length-percentage\> value | inset                  |
| <strong><span><a id="ref-for-propdef-position①③"></a></span><a href="#propdef-position">position</a>&#xA;      </strong> | static \| relative \| absolute \| sticky \| fixed | static  | all elements except table-column-group and table-column | no   | N/A                                          | discrete               | per grammar     | specified keyword                                          |                        |
| <strong><span><a id="ref-for-propdef-right②⑨"></a></span><a href="#propdef-right">right</a>&#xA;      </strong> | auto \| \<length-percentage\>                     | auto    | positioned elements                                     | no   | refer to size of containing block; see prose | by computed value type | per grammar     | the keyword auto or a computed \<length-percentage\> value | inset                  |
| <strong><span><a id="ref-for-propdef-top②④"></a></span><a href="#propdef-top">top</a>&#xA;      </strong> | auto \| \<length-percentage\>                     | auto    | positioned elements                                     | no   | refer to size of containing block; see prose | by computed value type | per grammar     | the keyword auto or a computed \<length-percentage\> value | inset                  |

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> What is a useful containing block to form when the box is fragmented across multiple lines? [\[Issue \#8284\]](https://github.com/w3c/csswg-drafts/issues/8284) [↵](#issue-d9a49098)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> What should happen in “negatively-sized” containing blocks? CSS2.1 and this draft currently conflict. [\[Issue \#11478\]](https://github.com/w3c/csswg-drafts/issues/11478) [↵](#issue-f171bef8)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This section is being replaced with the new [§ 4 Absolute Positioning Layout Model](#abspos-layout) section. It is preserved here for comparison: both models should yield the same result in [horizontal writing modes](https://www.w3.org/TR/css-writing-modes-4/#horizontal-writing-mode) when the box’s [self-alignment](https://www.w3.org/TR/css-align-3/#self-align) is [normal](https://www.w3.org/TR/css-align-3/#valdef-align-self-normal). [↵](#issue-3a21c2a0)
