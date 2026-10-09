Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Box Model Module Level 3](https://www.w3.org/TR/2024/REC-css-box-3-20240411/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Box Model Module Level 3

Source snapshot: https://www.w3.org/TR/2024/REC-css-box-3-20240411/

Snapshot SHA-256: 5330ef95e4b8ae6ee3f87ea24f52b2b23092fb0dd0f9011736e1878db96c702e

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 5 source tables are presented as readable Markdown tables or explicit labeled layouts: 5 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Box Model Module Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This specification describes the margin and padding properties, which create spacing in and around a CSS box.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a Recommendation using the [Recommendation track](https://www.w3.org/2023/Process-20231103/#recs-and-notes).

A W3C Recommendation is a specification that, after extensive consensus-building, is endorsed by W3C and its Members, and has commitments from Working Group members to [royalty-free licensing](https://www.w3.org/Consortium/Patent-Policy/#sec-Requirements) for implementations.

W3C recommends the wide deployment of this specification as a standard for the Web.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-box” in the title, like this: “\[css-box\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-box%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/2023/Process-20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1. Introduction

<em>This subsection is not normative.</em>

<a id="ref-for-element-tree"></a>

<a id="ref-for-box"></a>

<a id="ref-for-canvas"></a>

CSS describes how each element and each string of text in a source document is laid out by transforming the [element tree](https://www.w3.org/TR/css-display-3/#element-tree) into a set of [boxes](https://www.w3.org/TR/css-display-3/#box), whose size, position, and stacking level on the [canvas](https://www.w3.org/TR/CSS21/intro.html#canvas) depend on the values of their CSS properties.

<a id="ref-for-element-tree①"></a>

<a id="ref-for-box-tree"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [CSS Cascading and Inheritance](https://www.w3.org/TR/css-cascade/) describes how properties are assigned to elements in the box tree, while [CSS Display 3 § 1 Introduction](https://www.w3.org/TR/css-display-3/#intro) describes how the [element tree](https://www.w3.org/TR/css-display-3/#element-tree) is transformed into the [box tree](https://www.w3.org/TR/css-display-3/#box-tree).

<a id="ref-for-box①"></a>

<a id="ref-for-sizing-property"></a>

<a id="ref-for-propdef-padding"></a>

<a id="ref-for-propdef-border"></a>

<a id="ref-for-propdef-margin"></a>

Each CSS [box](https://www.w3.org/TR/css-display-3/#box) has a rectangular content area, a band of padding around the content, a border around the padding, and a margin outside the border. The [sizing properties](https://www.w3.org/TR/css-sizing-3/#sizing-property) [\[css-sizing-3\]](#biblio-css-sizing-3), together with various other properties that control layout, define the size of the content area. The box styling properties—​[padding](#propdef-padding) and its longhands, [border](https://www.w3.org/TR/css-backgrounds-3/#propdef-border) and its longhands, and [margin](#propdef-margin) and its longhands—​define the sizes of these other areas.

<a id="ref-for-margin"></a>

<a id="ref-for-padding"></a>

<a id="ref-for-border"></a>

[Margins](#margin) and [padding](#padding) are defined in this module. [Borders](#border) are similarly defined in [\[css-backgrounds-3\]](#biblio-css-backgrounds-3).

<a id="ref-for-physical"></a>

<a id="ref-for-longhand"></a>

<a id="ref-for-flow-relative"></a>

<a id="ref-for-margin①"></a>

<a id="ref-for-padding①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This module only defines the [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) per-side [longhand](https://www.w3.org/TR/css-cascade-5/#longhand) properties. Additional, [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) <a id="ref-for-longhand①"></a>longhand properties are defined in [\[css-logical-1\]](#biblio-css-logical-1). Note that both sets of <a id="ref-for-longhand②"></a>longhand properties control the same [margins](#margin)/[padding](#padding): they are just different ways of indexing each side.

> <strong data-conversion-semantic="note">Note</strong>
>
> History of CSS Box module and the definition of Block Layout
>
> This module [originally contained](https://www.w3.org/TR/2018/WD-css3-box-20180731/) the CSS Level 3 specification prose relating to box generation (now defined in [\[css-display-3\]](#biblio-css-display-3)), the box model (defined here), as well as block layout (now only defined in [\[CSS2\]](#biblio-css2) Chapters 9 and 10). Since its maintenance was put aside during the development of CSS2.1, its prose was severely outdated by the time CSS2 Revision 1 was finally completed. Therefore, the block layout portion of the prose has been retired, to be re-synched to [CSS2](https://www.w3.org/TR/CSS2) and updated as input to a new Block Layout module at some point in the future. It is being split apart from this module and from the [CSS Display Module](https://www.w3.org/TR/css-display/) both because of the practical concern that it would be a huge amount of work and also in recognition that CSS now has multiple layout models ([Flex Layout](https://www.w3.org/TR/css-flexbox/), [Grid Layout](https://www.w3.org/TR/css-grid/), [Positioned Layout](https://www.w3.org/TR/css-position/), and [Table Layout](https://www.w3.org/TR/css-tables/), in addition to Block Layout) which each deserve their own parallel module.

### <a id="values"></a>1.1.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

### <a id="placement"></a>1.2.  Module Interactions

This module replaces the definitions of the margin and padding properties defined in [\[CSS2\]](#biblio-css2) sections 8.1, 8.2, 8.3 (but not 8.3.1), and 8.4.

<a id="ref-for-selectordef-first-line"></a>

<a id="ref-for-selectordef-first-letter"></a>

All properties in this module apply to the [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) and [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) pseudo-elements.

## <a id="box-model"></a>2. The CSS Box Model

<a id="ref-for-replaced-element"></a>

Each box has a <a id="content-area"></a>content area (which contains its content—​text, descendant boxes, an image or other [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element) content, etc.) and optional surrounding <a id="padding-area"></a>padding, <a id="border-area"></a>border, and <a id="margin-area"></a>margin areas; the size of each area is specified by corresponding properties, and can be zero (or in the case of margins, negative). The following diagram shows how these areas relate and the terminology used to refer to the various parts of the box:

![Diagram of a typical box, showing the content, padding, border and margin areas](https://www.w3.org/TR/2024/REC-css-box-3-20240411/images/box.png)

The various areas and edges of a typical box.

<a id="ref-for-propdef-background"></a>

<a id="ref-for-propdef-border①"></a>

The background of the content, padding, and border areas of a box is specified by its [background](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) properties. The border area can additionally be painted with a border style using the [border](https://www.w3.org/TR/css-backgrounds-3/#propdef-border) properties. Margins are always transparent. See [\[css-backgrounds-3\]](#biblio-css-backgrounds-3).

The margin, border, and padding can be broken down into top, right, bottom, and left segments, each of which can be controlled independently by its corresponding property.

### <a id="box-edges"></a>2.1. Boxes and Edges

<a id="ref-for-box-box-edge"></a>

<a id="ref-for-box②"></a>

The perimeter of each of the four areas (content, padding, border, and margin) is called an <a id="box-box-edge"></a>edge, and each [edge](#box-box-edge) can be broken down into a top, right, bottom, and left side. Thus each [box](https://www.w3.org/TR/css-display-3/#box) has four <a id="ref-for-box-box-edge①"></a>edges each composed of four sides:

<a id="content-edge"></a>content edge or <a id="inner-edge"></a>inner edge  
<a id="ref-for-content-edge"></a>

<a id="ref-for-containing-block"></a>

The content edge surrounds the rectangle given by the width and height of the box, which often depend on the element’s content and/or its [containing block](https://www.w3.org/TR/css-display-3/#containing-block) size. The four sides of the [content edge](#content-edge) together define the box’s <a id="content-box"></a>content box.

<a id="padding-edge"></a>padding edge  
<a id="ref-for-padding-area"></a>

<a id="ref-for-content-area"></a>

<a id="ref-for-padding-edge"></a>

The padding edge surrounds the box’s padding. If the padding has zero width on a given side, the padding edge coincides with the content edge on that side. The four sides of the [padding edge](#padding-edge) together define the box’s <a id="padding-box"></a>padding box, which contains both the [content](#content-area) and [padding areas](#padding-area).

<a id="border-edge"></a>border edge  
<a id="ref-for-border-area"></a>

<a id="ref-for-padding-area①"></a>

<a id="ref-for-content-area①"></a>

<a id="ref-for-border-edge"></a>

The border edge surrounds the box’s border. If the border has zero width on a given side, the border edge coincides with the padding edge on that side. The four sides of the [border edge](#border-edge) together define the box’s <a id="border-box"></a>border box, which contains the box’s [content](#content-area), [padding](#padding-area), and [border areas](#border-area).

<a id="margin-edge"></a>margin edge or <a id="outer-edge"></a>outer edge  
<a id="ref-for-margin-area"></a>

<a id="ref-for-border-area①"></a>

<a id="ref-for-padding-area②"></a>

<a id="ref-for-content-area②"></a>

<a id="ref-for-margin-edge"></a>

The margin edge surrounds the box’s margin. If the margin has zero width on a given side, the margin edge coincides with the border edge on that side. The four sides of the [margin edge](#margin-edge) together define the box’s <a id="margin-box"></a>margin box, which contains the all of the box’s [content](#content-area), [padding](#padding-area), [border](#border-area), and [margin areas](#margin-area).

### <a id="fragmentation"></a>2.2. Fragmentation

<a id="ref-for-box-fragment"></a>

<a id="ref-for-content-box"></a>

<a id="ref-for-padding-box"></a>

<a id="ref-for-border-box"></a>

<a id="ref-for-margin-box"></a>

<a id="ref-for-propdef-box-decoration-break"></a>

When a box [fragments](https://www.w3.org/TR/css-break-3/#fragmentation-model)—​is broken, as across lines or across pages, into separate [box fragments](https://www.w3.org/TR/css-break-4/#box-fragment)—​each of its boxes ([content box](#content-box), [padding box](#padding-box), [border box](#border-box), [margin box](#margin-box)) also fragments. How the content/padding/border/margin areas react to fragmentation is specified in [\[css-break-3\]](#biblio-css-break-3) and controlled by the [box-decoration-break](https://www.w3.org/TR/css-break-4/#propdef-box-decoration-break) property.

### <a id="keywords"></a>2.3. Box-edge Keywords

<a id="ref-for-typedef-box"></a>

<a id="ref-for-propdef-transform-box"></a>

<a id="ref-for-propdef-background-clip"></a>

The following <a id="typedef-box"></a>[\<box\>](#typedef-box) CSS keywords are defined for use in properties (such as [transform-box](https://www.w3.org/TR/css-transforms-1/#propdef-transform-box) and [background-clip](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-clip)) that need to refer to various box edges:

<a id="valdef-box-content-box"></a>content-box  
<a id="ref-for-valdef-box-fill-box"></a>

<a id="ref-for-content-edge①"></a>

<a id="ref-for-content-box①"></a>

Refers to the [content box](#content-box) or [content edge](#content-edge). (In an SVG context, treated as [fill-box](#valdef-box-fill-box).)

<a id="valdef-box-padding-box"></a>padding-box  
<a id="ref-for-valdef-box-fill-box①"></a>

<a id="ref-for-padding-edge①"></a>

<a id="ref-for-padding-box①"></a>

Refers to the [padding box](#padding-box) or [padding edge](#padding-edge). (In an SVG context, treated as [fill-box](#valdef-box-fill-box).)

<a id="valdef-box-border-box"></a>border-box  
<a id="ref-for-valdef-box-stroke-box"></a>

<a id="ref-for-border-edge①"></a>

<a id="ref-for-border-box①"></a>

Refers to the [border box](#border-box) or [border edge](#border-edge). (In an SVG context, treated as [stroke-box](#valdef-box-stroke-box).)

<a id="valdef-box-margin-box"></a>margin-box  
<a id="ref-for-valdef-box-stroke-box①"></a>

<a id="ref-for-margin-edge①"></a>

<a id="ref-for-margin-box①"></a>

Refers to the [margin box](#margin-box) or [margin edge](#margin-edge). (In an SVG context, treated as [stroke-box](#valdef-box-stroke-box).)

<a id="valdef-box-fill-box"></a>fill-box  
<a id="ref-for-valdef-box-content-box"></a>

<a id="ref-for-TermObjectBoundingBox"></a>

Refers to the [object bounding box](https://www.w3.org/TR/SVG2/coords.html#TermObjectBoundingBox) or its edges. (In a CSS box context, treated as [content-box](#valdef-box-content-box).)

<a id="valdef-box-stroke-box"></a>stroke-box  
<a id="ref-for-valdef-box-border-box"></a>

<a id="ref-for-TermStrokeBoundingBox"></a>

Refers to the [stroke bounding box](https://www.w3.org/TR/SVG2/coords.html#TermStrokeBoundingBox) or its edges. (In a CSS box context, treated as [border-box](#valdef-box-border-box).)

<a id="valdef-box-view-box"></a>view-box  
<a id="ref-for-valdef-box-border-box①"></a>

<a id="ref-for-TermViewBox"></a>

<a id="ref-for-TermUserCoordinateSystem"></a>

<a id="ref-for-TermSVGViewport"></a>

Refers to the nearest [SVG viewport](https://www.w3.org/TR/SVG2/coords.html#TermSVGViewport) element’s <a id="svg-viewport-origin-box"></a>origin box, which is a rectangle with the width and height of the initial SVG [user coordinate system](https://www.w3.org/TR/SVG2/coords.html#TermUserCoordinateSystem) established by the <code><a href="https://www.w3.org/TR/SVG2/coords.html#TermViewBox">viewBox</a></code> attribute for that element, positioned such that its top left corner is anchored at the coordinate system origin. (In a CSS box context, treated as [border-box](#valdef-box-border-box).)

<a id="ref-for-TermViewBox①"></a>

<a id="ref-for-svg-viewport-origin-box"></a>

<a id="ref-for-TermViewBox②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: When the <code><a href="https://www.w3.org/TR/SVG2/coords.html#TermViewBox">viewBox</a></code> includes non-zero <var>min-x</var> or <var>min-y</var> offsets, this [origin box](#svg-viewport-origin-box) does not actually correspond to the visible region rectangle defined by the <code><a href="https://www.w3.org/TR/SVG2/coords.html#TermViewBox">viewBox</a></code>!

<a id="ref-for-typedef-box①"></a>

For convenience, the following value types are defined to represents commonly used subsets of [\<box\>](#typedef-box):

<a id="typedef-visual-box"></a>

<a id="ref-for-typedef-visual-box"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-comb-one①"></a>

<a id="typedef-layout-box"></a>

<a id="ref-for-typedef-layout-box"></a>

<a id="ref-for-typedef-visual-box①"></a>

<a id="ref-for-comb-one②"></a>

<a id="typedef-paint-box"></a>

<a id="ref-for-typedef-paint-box"></a>

<a id="ref-for-typedef-visual-box②"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-comb-one④"></a>

<a id="typedef-coord-box"></a>

<a id="ref-for-typedef-coord-box"></a>

<a id="ref-for-typedef-paint-box①"></a>

<a id="ref-for-comb-one⑤"></a>

```text
<visual-box> = content-box | padding-box | border-box
<layout-box> = <visual-box> | margin-box
<paint-box> = <visual-box> | fill-box | stroke-box
<coord-box> = <paint-box> | view-box
```
## <a id="margins"></a>3. Margins

<a id="ref-for-margin-area①"></a>

<a id="ref-for-propdef-margin①"></a>

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-longhand③"></a>

<a id="ref-for-physical①"></a>

<a id="ref-for-flow-relative①"></a>

<a id="margin"></a>Margins surround the border edge of a box, providing spacing between boxes. The <a id="margin-properties"></a>margin properties specify the thickness of the [margin area](#margin-area) of a box. The [margin](#propdef-margin) [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) sets the margin for all four sides while the margin [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand) only set their respective side. This section defines the [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) <a id="ref-for-propdef-margin②"></a>margin <a id="ref-for-longhand④"></a>longhands. (Additional [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) <a id="ref-for-propdef-margin③"></a>margin <a id="ref-for-longhand⑤"></a>longhands are defined in [\[css-logical-1\]](#biblio-css-logical-1).)

<a id="ref-for-block-layout"></a>

<a id="ref-for-fragmentation-break"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Adjoining margins in [block layout](https://www.w3.org/TR/css-display-3/#block-layout) can <em>collapse</em>. See [CSS2§8.3.1 Collapsing Margins](https://www.w3.org/TR/CSS2/box.html#collapsing-margins) for details. Also, margins adjoining a [fragmentation break](https://www.w3.org/TR/css-break-4/#fragmentation-break) are sometimes truncated. See [CSS Fragmentation 3 § 5.2 Adjoining Margins at Breaks](https://www.w3.org/TR/css-break-3/#break-margins) for details.

<a id="ref-for-propdef-margin-top"></a>

<a id="ref-for-propdef-margin-right"></a>

<a id="ref-for-propdef-margin-bottom"></a>

<a id="ref-for-propdef-margin-left"></a>

### <a id="margin-physical"></a>3.1. Page-relative (Physical) Margin Properties: the [margin-top](#propdef-margin-top), [margin-right](#propdef-margin-right), [margin-bottom](#propdef-margin-bottom), and [margin-left](#propdef-margin-left) properties

| Field               | Definition                                                                                                                                                                           |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-margin-top"></a>margin-top, <a id="propdef-margin-right"></a>margin-right, <a id="propdef-margin-bottom"></a>margin-bottom, <a id="propdef-margin-left"></a>margin-left                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑥"></a><a id="ref-for-typedef-length-percentage"></a>[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) [\|](https://www.w3.org/TR/css-values-4/#comb-one) auto |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-internal-table-element"></a>all elements except [internal table elements](https://www.w3.org/TR/css-display-3/#internal-table-element)                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-logical-width"></a>refer to [logical width](https://www.w3.org/TR/css-writing-modes-4/#logical-width) of containing block                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①"></a>the keyword auto or a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                                               |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-margin④"></a>[margin](#propdef-margin)                                                                                                                                         |

<a id="ref-for-margin②"></a>

<a id="ref-for-box③"></a>

These properties set the top, right, bottom, and left [margin](#margin) of a [box](https://www.w3.org/TR/css-display-3/#box), respectively.

Negative values for margin properties are allowed, but there may be implementation-specific limits.

<a id="ref-for-internal-ruby-boxes"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Application to [internal ruby boxes](https://www.w3.org/TR/css-ruby-1/#internal-ruby-boxes) [\[CSS-RUBY-1\]](#biblio-css-ruby-1) is not defined in this specification.

<a id="ref-for-propdef-margin⑤"></a>

### <a id="margin-shorthand"></a>3.2. Margin Shorthand: the [margin](#propdef-margin) property

| Field               | Definition                                                                                                                               |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-margin"></a>margin                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range"></a><a id="ref-for-propdef-margin-top①"></a>[\<'margin-top'\>](#propdef-margin-top)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-logical-width①"></a>refer to [logical width](https://www.w3.org/TR/css-writing-modes-4/#logical-width) of containing block                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                   |

<a id="ref-for-propdef-margin⑥"></a>

<a id="ref-for-propdef-margin-top②"></a>

<a id="ref-for-propdef-margin-right①"></a>

<a id="ref-for-propdef-margin-bottom①"></a>

<a id="ref-for-propdef-margin-left①"></a>

The [margin](#propdef-margin) property is a shorthand property for setting [margin-top](#propdef-margin-top), [margin-right](#propdef-margin-right), [margin-bottom](#propdef-margin-bottom), and [margin-left](#propdef-margin-left) in a single declaration.

If there is only one component value, it applies to all sides. If there are two values, the top and bottom margins are set to the first value and the right and left margins are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values they apply to the top, right, bottom, and left, respectively.

<a id="ref-for-propdef-margin⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2e169656"></a> The following code demonstrates some possible [margin](#propdef-margin) declarations.
>
> ```text
> body { margin: 2em }         /* all margins set to 2em */
> body { margin: 1em 2em }     /* top & bottom = 1em, right & left = 2em */
> body { margin: 1em 2em 3em } /* top=1em, right=2em, bottom=3em, left=2em */
> ```
>
> The last rule of the example above is equivalent to the example below:
>
> ```text
> body {
>   margin-top: 1em;
>   margin-right: 2em;
>   margin-bottom: 3em;
>   margin-left: 2em; /* copied from opposite side (right) */
> }
> ```
## <a id="paddings"></a>4. Padding

<a id="ref-for-padding-area③"></a>

<a id="ref-for-propdef-padding①"></a>

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-longhand⑥"></a>

<a id="ref-for-physical②"></a>

<a id="ref-for-flow-relative②"></a>

<a id="padding"></a>Padding is inserted between the content edge and the padding edge of a box, providing spacing between the content and the border. The <a id="padding-properties"></a>padding properties specify the thickness of the [padding area](#padding-area) of a box. The [padding](#propdef-padding) [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) sets the padding for all four sides while the padding [longhand properties](https://www.w3.org/TR/css-cascade-5/#longhand) only set their respective side. This section defines the [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) <a id="ref-for-propdef-padding②"></a>padding <a id="ref-for-longhand⑦"></a>longhands. (Additional [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) <a id="ref-for-propdef-padding③"></a>padding <a id="ref-for-longhand⑧"></a>longhands are defined in [\[css-logical-1\]](#biblio-css-logical-1).)

<a id="ref-for-border-area②"></a>

<a id="ref-for-propdef-background-origin"></a>

<a id="ref-for-propdef-background-clip①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Backgrounds specified on the box are by default laid out and painted within the padding edges. (They are additionally painted underneath the border, in the [border area](#border-area).) This behavior can be adjusted using the [background-origin](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-origin) and [background-clip](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-clip) properties.

<a id="ref-for-propdef-padding-top"></a>

<a id="ref-for-propdef-padding-right"></a>

<a id="ref-for-propdef-padding-bottom"></a>

<a id="ref-for-propdef-padding-left"></a>

### <a id="padding-physical"></a>4.1. Page-relative (Physical) Padding Properties: the [padding-top](#propdef-padding-top), [padding-right](#propdef-padding-right), [padding-bottom](#propdef-padding-bottom), and [padding-left](#propdef-padding-left) properties

| Field               | Definition                                                                                                                                            |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-padding-top"></a>padding-top, <a id="propdef-padding-right"></a>padding-right, <a id="propdef-padding-bottom"></a>padding-bottom, <a id="propdef-padding-left"></a>padding-left                  |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage②"></a>[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-internal-table-element①"></a>all elements except: [internal table elements](https://www.w3.org/TR/css-display-3/#internal-table-element) other than table cells |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-logical-width②"></a>refer to [logical width](https://www.w3.org/TR/css-writing-modes-4/#logical-width) of containing block                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage③"></a>a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                                |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-padding④"></a>[padding](#propdef-padding)                                                                                                        |

<a id="ref-for-padding②"></a>

<a id="ref-for-box④"></a>

These properties set the top, right, bottom, and left [padding](#padding) of a [box](https://www.w3.org/TR/css-display-3/#box), respectively.

Negative values for padding properties are invalid.

<a id="ref-for-internal-ruby-boxes①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Application to [internal ruby boxes](https://www.w3.org/TR/css-ruby-1/#internal-ruby-boxes) [\[CSS-RUBY-1\]](#biblio-css-ruby-1) is not defined in this specification.

<a id="ref-for-propdef-padding⑤"></a>

### <a id="padding-shorthand"></a>4.2. Padding Shorthand: the [padding](#propdef-padding) property

| Field               | Definition                                                                                                                                 |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-padding"></a>padding                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range①"></a><a id="ref-for-propdef-padding-top①"></a>[\<'padding-top'\>](#propdef-padding-top)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-logical-width③"></a>refer to [logical width](https://www.w3.org/TR/css-writing-modes-4/#logical-width) of containing block                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                                                                                     |

<a id="ref-for-propdef-padding⑥"></a>

<a id="ref-for-propdef-padding-top②"></a>

<a id="ref-for-propdef-padding-right①"></a>

<a id="ref-for-propdef-padding-bottom①"></a>

<a id="ref-for-propdef-padding-left①"></a>

The [padding](#propdef-padding) property is a shorthand property for setting [padding-top](#propdef-padding-top), [padding-right](#propdef-padding-right), [padding-bottom](#propdef-padding-bottom), and [padding-left](#propdef-padding-left) in a single declaration.

If there is only one component value, it applies to all sides. If there are two values, the top and bottom padding are set to the first value and the right and left padding are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third.

<a id="ref-for-propdef-padding⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-23862f1f"></a> The following code demonstrates some possible [padding](#propdef-padding) declarations.
>
> ```text
> body { padding: 2em }         /* all padding set to 2em */
> body { padding: 1em 2em }     /* top & bottom = 1em, right & left = 2em */
> body { padding: 1em 2em 3em } /* top=1em, right=2em, bottom=3em, left=2em */
> ```
>
> The last rule of the example above is equivalent to the example below:
>
> ```text
> body {
>   padding-top: 1em;
>   padding-right: 2em;
>   padding-bottom: 3em;
>   padding-left: 2em; /* copied from opposite side (right) */
> }
> ```
## <a id="borders"></a>5. Borders

<a id="ref-for-border-area③"></a>

<a id="ref-for-physical③"></a>

<a id="ref-for-flow-relative③"></a>

<a id="border"></a>Borders fill the [border area](#border-area), to visually delineate the edges of the box, The <a id="border-properties"></a>border properties specify the thickness of the <a id="ref-for-border-area④"></a>border area of a box, as well as its drawing style and color. See [\[css-backgrounds-3\]](#biblio-css-backgrounds-3) for the definition of these properties, including their [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) longhands. (Additional [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) border longhands are defined in [\[css-logical-1\]](#biblio-css-logical-1).)

## <a id="changes"></a>6. Changes

### <a id="changes-recent"></a>6.1. Recent Changes

There have only been minor editorial adjustments since the [6 April 2023 Recommendation](https://www.w3.org/TR/css-box-3/):

- Adding some subheadings to [§ 2 The CSS Box Model](#box-model) and moving a paragraph accordingly.

- Changing “document tree” to “element tree” in the [§ 1 Introduction](#intro) to align with the terminology used in [\[css-display-3\]](#biblio-css-display-3).

- Adding the “Logical Property Group” line to the property definition tables.

### <a id="changes-since-L2"></a>6.2. Changes Since CSS Level 2

The following changes have been made to this module since [CSS Level 2](https://www.w3.org/TR/CSS2/box.html):

- Marked application to internal ruby boxes as undefined in this level.

- <a id="ref-for-writing-mode"></a>

  Adapting the prose slightly to account for vertical [writing modes](https://www.w3.org/TR/css-writing-modes-4/#writing-mode).

- Cross-linking to relevant concepts in [\[css-break-3\]](#biblio-css-break-3) and [\[css-backgrounds-3\]](#biblio-css-backgrounds-3).

- <a id="ref-for-propdef-background-clip②"></a>

  Providing a [centralized common definition](#keywords) of keywords that reference the various box edges for use in properties defined outside this spec (e.g. [background-clip](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-clip)) to avoid repetition (and the resulting inevitable synchronization errors).

## <a id="priv"></a>7. Privacy Considerations

No privacy considerations have been reported on this module.

## <a id="sec"></a>8. Security Considerations

No security considerations have been reported on this module.

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

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [border](#border), in § 5
- [border area](#border-area), in § 2
- [border box](#border-box), in § 2.1
- [border-box](#valdef-box-border-box), in § 2.3
- [border edge](#border-edge), in § 2.1
- [border properties](#border-properties), in § 5
- [\<box\>](#typedef-box), in § 2.3
- [box edge](#box-box-edge), in § 2.1
- [content area](#content-area), in § 2
- [content box](#content-box), in § 2.1
- [content-box](#valdef-box-content-box), in § 2.3
- [content edge](#content-edge), in § 2.1
- [\<coord-box\>](#typedef-coord-box), in § 2.3
- [edge](#box-box-edge), in § 2.1
- [fill-box](#valdef-box-fill-box), in § 2.3
- [inner edge](#inner-edge), in § 2.1
- [\<layout-box\>](#typedef-layout-box), in § 2.3
- margin
  - [(property)](#propdef-margin), in § 3.2
  - [definition of](#margin), in § 3
- [margin area](#margin-area), in § 2
- [margin-bottom](#propdef-margin-bottom), in § 3.1
- [margin box](#margin-box), in § 2.1
- [margin-box](#valdef-box-margin-box), in § 2.3
- [margin edge](#margin-edge), in § 2.1
- [margin-left](#propdef-margin-left), in § 3.1
- [margin properties](#margin-properties), in § 3
- [margin-right](#propdef-margin-right), in § 3.1
- [margin-top](#propdef-margin-top), in § 3.1
- [origin box](#svg-viewport-origin-box), in § 2.3
- [outer edge](#outer-edge), in § 2.1
- [Padding](#padding), in § 4
- [padding](#propdef-padding), in § 4.2
- [padding area](#padding-area), in § 2
- [padding-bottom](#propdef-padding-bottom), in § 4.1
- [padding box](#padding-box), in § 2.1
- [padding-box](#valdef-box-padding-box), in § 2.3
- [padding edge](#padding-edge), in § 2.1
- [padding-left](#propdef-padding-left), in § 4.1
- [padding properties](#padding-properties), in § 4
- [padding-right](#propdef-padding-right), in § 4.1
- [padding-top](#propdef-padding-top), in § 4.1
- [\<paint-box\>](#typedef-paint-box), in § 2.3
- [stroke-box](#valdef-box-stroke-box), in § 2.3
- [SVG viewport origin box](#svg-viewport-origin-box), in § 2.3
- [view-box](#valdef-box-view-box), in § 2.3
- [\<visual-box\>](#typedef-visual-box), in § 2.3

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="db6870d5"></a>background
  - <a id="a5c1f433"></a>background-clip
  - <a id="6cdd1a36"></a>background-origin
  - <a id="e1674793"></a>border
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="d65c0e81"></a>box fragment
  - <a id="a0542bba"></a>box-decoration-break
  - <a id="972b685d"></a>fragmentation break
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="36261173"></a>longhand
  - <a id="8f27be0f"></a>longhand property
  - <a id="980ac56a"></a>shorthand property
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="5f3ef37a"></a>block layout
  - <a id="5c159f8f"></a>box
  - <a id="79d4bbff"></a>box tree
  - <a id="6b4fc208"></a>containing block
  - <a id="6bf41093"></a>element tree
  - <a id="81c826e7"></a>internal table element
  - <a id="299e10e4"></a>replaced element
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="63b59bd9"></a>::first-letter
  - <a id="4bda66a9"></a>::first-line
- \[CSS-RUBY-1\] defines the following terms:
  - <a id="123f21cb"></a>internal ruby boxes
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="2ac08cff"></a>sizing property
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="dbb6f526"></a>transform-box
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="8a110a7b"></a>css-wide keywords
  - <a id="3bafef5e"></a>{a,b}
  - <a id="4eb9d37e"></a>\|
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="303c8d41"></a>flow-relative
  - <a id="99a9e10b"></a>logical width
  - <a id="e1f6e4b9"></a>physical
  - <a id="eb6008ce"></a>writing mode
- \[CSS2\] defines the following terms:
  - <a id="2a2f1579"></a>canvas
- \[SVG2\] defines the following terms:
  - <a id="a7b1a9fa"></a>object bounding box
  - <a id="c13fd5e8"></a>stroke bounding box
  - <a id="153900b0"></a>svg viewports
  - <a id="925f6904"></a>user coordinate system
  - <a id="b7c9858b"></a>viewbox

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Daniel Glazman; Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 30 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

### <a id="informative"></a>Informative References

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-logical-1"></a>\[CSS-LOGICAL-1\]  
Rossen Atanassov; Elika Etemad. [CSS Logical Properties and Values Level 1](https://www.w3.org/TR/css-logical-1/). 27 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-logical-1&#x2F;](https://www.w3.org/TR/css-logical-1/)

<a id="biblio-css-ruby-1"></a>\[CSS-RUBY-1\]  
Elika Etemad; et al. [CSS Ruby Annotation Layout Module Level 1](https://www.w3.org/TR/css-ruby-1/). 31 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ruby-1&#x2F;](https://www.w3.org/TR/css-ruby-1/)

## <a id="property-index"></a>Property Index

| Name                | Value                         | Initial | Applies to                                                          | Inh. | %ages                                      | Anim­ation type         | Canonical order | Com­puted value                                             | Logical property group |
|---------------------|-------------------------------|---------|---------------------------------------------------------------------|------|--------------------------------------------|------------------------|-----------------|------------------------------------------------------------|------------------------|
| <strong><span><a id="ref-for-propdef-margin⑧"></a></span><a href="#propdef-margin">margin</a>&#xA;      </strong> | \<'margin-top'\>{1,4}         | 0       | see individual properties                                           | no   | refer to logical width of containing block | by computed value type | per grammar     | see individual properties                                  |                        |
| <strong><span><a id="ref-for-propdef-margin-bottom②"></a></span><a href="#propdef-margin-bottom">margin-bottom</a>&#xA;      </strong> | \<length-percentage\> \| auto | 0       | all elements except internal table elements                         | no   | refer to logical width of containing block | by computed value type | per grammar     | the keyword auto or a computed \<length-percentage\> value | margin                 |
| <strong><span><a id="ref-for-propdef-margin-left②"></a></span><a href="#propdef-margin-left">margin-left</a>&#xA;      </strong> | \<length-percentage\> \| auto | 0       | all elements except internal table elements                         | no   | refer to logical width of containing block | by computed value type | per grammar     | the keyword auto or a computed \<length-percentage\> value | margin                 |
| <strong><span><a id="ref-for-propdef-margin-right②"></a></span><a href="#propdef-margin-right">margin-right</a>&#xA;      </strong> | \<length-percentage\> \| auto | 0       | all elements except internal table elements                         | no   | refer to logical width of containing block | by computed value type | per grammar     | the keyword auto or a computed \<length-percentage\> value | margin                 |
| <strong><span><a id="ref-for-propdef-margin-top③"></a></span><a href="#propdef-margin-top">margin-top</a>&#xA;      </strong> | \<length-percentage\> \| auto | 0       | all elements except internal table elements                         | no   | refer to logical width of containing block | by computed value type | per grammar     | the keyword auto or a computed \<length-percentage\> value | margin                 |
| <strong><span><a id="ref-for-propdef-padding⑧"></a></span><a href="#propdef-padding">padding</a>&#xA;      </strong> | \<'padding-top'\>{1,4}        | 0       | see individual properties                                           | no   | refer to logical width of containing block | by computed value type | per grammar     | see individual properties                                  |                        |
| <strong><span><a id="ref-for-propdef-padding-bottom②"></a></span><a href="#propdef-padding-bottom">padding-bottom</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\> | 0       | all elements except: internal table elements other than table cells | no   | refer to logical width of containing block | by computed value type | per grammar     | a computed \<length-percentage\> value                     | padding                |
| <strong><span><a id="ref-for-propdef-padding-left②"></a></span><a href="#propdef-padding-left">padding-left</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\> | 0       | all elements except: internal table elements other than table cells | no   | refer to logical width of containing block | by computed value type | per grammar     | a computed \<length-percentage\> value                     | padding                |
| <strong><span><a id="ref-for-propdef-padding-right②"></a></span><a href="#propdef-padding-right">padding-right</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\> | 0       | all elements except: internal table elements other than table cells | no   | refer to logical width of containing block | by computed value type | per grammar     | a computed \<length-percentage\> value                     | padding                |
| <strong><span><a id="ref-for-propdef-padding-top③"></a></span><a href="#propdef-padding-top">padding-top</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\> | 0       | all elements except: internal table elements other than table cells | no   | refer to logical width of containing block | by computed value type | per grammar     | a computed \<length-percentage\> value                     | padding                |

