Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Shapes Module Level 1](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/).

Original copyright notice: Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Shapes Module Level 1

Source snapshot: https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/

Snapshot SHA-256: f7e528fad3094f91fb9591ba6f849e2453353d50d0f136ca9e0749371fc31763

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 4 source tables are presented as readable Markdown tables or explicit labeled layouts: 4 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Shapes Module Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-wrap"></a>

CSS Shapes describe geometric shapes for use in CSS. For Level 1, CSS Shapes can be applied to floats. A circle shape on a float will cause inline content to [wrap](#wrap) around the circle shape instead of the float’s bounding box.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20231103/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-shapes” in the title, like this: “\[css-shapes\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-shapes%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/policies/process/20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

<em>This section is not normative.</em>

<a id="ref-for-float-area"></a>

<a id="ref-for-propdef-shape-outside"></a>

Shapes define arbitrary geometries that can be used as CSS values. This specification defines properties to control the geometry of an element’s [float area](#float-area). The [shape-outside](#propdef-shape-outside) property uses shape values to define the <a id="ref-for-float-area①"></a>float area for a float.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Future levels of CSS Shapes will allow use of shapes on elements other than floats. Other CSS modules can make use of shapes as well, such as CSS Masking [\[CSS-MASKING\]](#biblio-css-masking) and CSS Exclusions [\[CSS3-EXCLUSIONS\]](#biblio-css3-exclusions).

<a id="ref-for-propdef-shape-outside①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If a user agent implements both CSS Shapes and CSS Exclusions, the [shape-outside](#propdef-shape-outside) property defines the exclusion area for an exclusion.

<a id="ref-for-wrap①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A future level of CSS Shapes will define a shape-inside property, which will define a shape to [wrap](#wrap) content within the element.

### <a id="module-interactions"></a>1.1.  Module Interactions

This module extends the float features defined in [\[CSS2\]](#biblio-css2) chapter 9.

### <a id="values"></a>1.2. Values

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

### <a id="terminology"></a>1.3.  Terminology

<a id="wrap"></a>Wrap

<a id="ref-for-wrap②"></a>

<a id="ref-for-float-area②"></a>

This specification uses the term [wrap](#wrap) to refer to flowing content around the sides of a [float area](#float-area), defined in [\[CSS2\]](#biblio-css2) chapter 9. Content <a id="ref-for-wrap③"></a>wraps around the right side of a left-floated box, and content <a id="ref-for-wrap④"></a>wraps around the left side of a right-floated box. One result of this <a id="ref-for-wrap⑤"></a>wrapping is that line boxes next to a float are shortened as necessary to avoid intersections with the <a id="ref-for-float-area③"></a>float area.

<a id="float-area"></a>Float area

<a id="ref-for-wrap⑥"></a>

<a id="ref-for-float-area④"></a>

<a id="ref-for-margin-box"></a>

<a id="ref-for-valdef-shape-box-margin-box"></a>

<a id="ref-for-propdef-shape-outside②"></a>

<a id="ref-for-propdef-shape-margin"></a>

The area used for [wrapping](#wrap) content around a float element. The rules for float behavior use the sides of the [float area](#float-area) to determine where content flows. By default, the <a id="ref-for-float-area⑤"></a>float area is the float element’s [margin box](https://www.w3.org/TR/css-box-4/#margin-box) (note this can be different than the <a id="ref-for-float-area⑥"></a>float area produced by the [margin-box](#valdef-shape-box-margin-box) value, which includes border-radius curvature). This specification’s [shape-outside](#propdef-shape-outside) and [shape-margin](#propdef-shape-margin) properties can be used to define an arbitrary, non-rectangular <a id="ref-for-float-area⑦"></a>float area.

<a id="ref-for-direction-agnostic-size"></a>

<a id="direction-agnostic-size"></a>direction-agnostic size The [direction-agnostic size](#direction-agnostic-size) of a box is equal to the length of the diagonal of the box, divided by sqrt(2).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is a method of averaging the width and height of a box used by SVG in some cases, when a percentage of a box’s size is desired but the context doesn’t specifically favor the width or the height. For square boxes, this is the same as the width/height.

## <a id="relation-to-box-model-and-float-behavior"></a>2.  Relation to the box model and float behavior

<a id="ref-for-wrap⑦"></a>

<a id="ref-for-float-area⑧"></a>

While the boundaries used for [wrapping](#wrap) inline flow content outside a float can be defined using shapes, the actual box model does not change. If the element has specified margins, borders or padding they will be computed and rendered according to the [\[CSS3BOX\]](#biblio-css3box) module. Also, float positioning and stacking are not affected by defining a [float area](#float-area) with a shape.

<a id="ref-for-float-area⑨"></a>

When a shape is used to define a [float area](#float-area), the shape is clipped to the float’s margin box. In other words, a shape can only ever reduce a <a id="ref-for-float-area①⓪"></a>float area, not increase it. A reduced <a id="ref-for-float-area①①"></a>float area may have no effect on some line boxes that would normally be affected by the float. If a shape does not enclose any area, the shape’s edges are still used to define the <a id="ref-for-float-area①②"></a>float area.

<a id="ref-for-float-area①③"></a>

<a id="ref-for-wrap⑧"></a>

<a id="ref-for-propdef-shape-outside③"></a>

A [float area](#float-area) defined by a shape may reduce the normal <a id="ref-for-float-area①④"></a>float area on all sides, but this does not allow content to [wrap](#wrap) on both sides of a float. Left floats with a [shape-outside](#propdef-shape-outside) still only allow content <a id="ref-for-wrap⑨"></a>wrapping on the right side, and right floats only allow <a id="ref-for-wrap①⓪"></a>wrapping on the left.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-39c551b4"></a>
>
> <a id="ref-for-propdef-shape-outside④"></a>
>
> In the following example the left and right floating `img` elements specify a triangular shape using the [shape-outside](#propdef-shape-outside) property.
>
> ```text
> <img class="left" src="hand.svg"/>
> <img class="right" src="hand.svg"/>
> <p>
>   Sometimes a web page’s text content appears to be
>   funneling your attention towards a spot on the page
>   to drive you to follow a particular link. Sometimes
>   you don’t notice.
> </p>
> 
> <style type="text/css">
>   .left {
>     shape-outside: polygon(0 0, 100% 100%, 0 100%);
>     float: left;
>     width: 40%;
>     height: 12ex;
>     transform: scaleX(-1);
>   }
> 
>   .right {
>     shape-outside: polygon(100% 0, 100% 100%, 0 100%);
>     float: right;
>     width: 40%;
>     height: 12ex;
>   }
> 
>   p {
>     text-align: center;
>   }
> </style>
> ```
>
> ![Using the shape-outside property with floats](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/hand-funnel.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c9556a71"></a>
>
> Since shapes are clipped to the float’s margin box, adding this shape to the left float above would result in the same rendering.
>
> ```text
> shape-outside: polygon(0 0, 500% 500%, 0 500%);
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e8eedd7c"></a>
>
> <a id="ref-for-float-area①⑤"></a>
>
> A shape that does not enclose any area still has edges that contribute to the [float area](#float-area).
>
> This inset shape is a vertical line positioned at the midpoint of the reference box. This midpoint edge is used as the edge of the float area for wrapping content.
>
> ```text
> shape-outside: inset(0% 50% 0% 50%);
> ```
>
> If inset values add up to more than the width, [CSS Backgrounds 3 § 4.5 Overlapping Curves](https://www.w3.org/TR/css-backgrounds-3/#corner-overlap) rules are used to determine the edges of the rectangle. This shape results in a vertical edge 25% from the left side of the reference box.
>
> ```text
> shape-outside: inset(0% 150% 50% 0%);
> ```
>
> If the shape is only a horizontal line, then it is an empty float area and has no effect on wrapping. Note that in this example shape-margin must be 0px (otherwise the line would expand to enclose an area).
>
> ```text
> shape-outside: inset(50% 0% 0% 50%);
> shape-margin: 0px;
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-105c1362"></a>
>
> <a id="ref-for-propdef-shape-outside⑤"></a>
>
> <a id="ref-for-float-area①⑥"></a>
>
> <a id="ref-for-wrap①①"></a>
>
> A [shape-outside](#propdef-shape-outside) can create open areas on both the left and right of a [float area](#float-area). Content still [wraps](#wrap) only on one side of a float in this case. In the picture, the shape is rendered in blue, and the content area outside the shape in mauve.
>
> ```text
> shape-outside: polygon(50px 0px, 100px 100px, 0px 100px);
> ```
>
> ![wrapping around right side of a left-float float area](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/float-side-example.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8937ace2"></a>
>
> <a id="ref-for-wrap①②"></a>
>
> The following styling creates a shape much smaller than the float’s content area, and adds a margin-top to the float. In the picture, the shape is rendered in blue, the content area outside the shape in mauve, and the margin area of the float box in yellow. The inline content only [wraps](#wrap) around the shape, and otherwise overlays the rest of the float margin box.
>
> ```text
> .float-left {
>   shape-outside: polygon(0% 50%, 50% 100%, 0 100%);
>   float: left;
>   width: 100px;
>   height: 100px;
>   margin-top: 20px;
> }
> ```
>
> ![Adding margin-top to a float with a small shape-outside](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/float-margin-example.png)
>
> <a id="ref-for-float-area①⑦"></a>
>
> The next picture shows a possible result if two of these floats were stacked next to each other. Note that the floats are positioned using their margin boxes, not the [float area](#float-area).
>
> ![Stacking two floats with a small shape-outside](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/stacked-float-example.png)

## <a id="basic-shape-functions"></a>3.  Basic Shapes

<a id="ref-for-typedef-basic-shape"></a>

<a id="ref-for-basic-shape-reference-box"></a>

The <a id="typedef-basic-shape"></a>\<basic-shape\> type can be specified using basic shape functions. When using this syntax to define shapes, the <a id="basic-shape-reference-box"></a>reference box is defined by each property that uses [\<basic-shape\>](#typedef-basic-shape) values. The coordinate system for the shape has its origin on the top-left corner of the [reference box](#basic-shape-reference-box) with the x-axis running to the right and the y-axis running downwards. All the lengths expressed in percentages are resolved from the used dimensions of the <a id="ref-for-basic-shape-reference-box①"></a>reference box.

### <a id="supported-basic-shapes"></a>3.1.  Supported Shapes

<a id="ref-for-typedef-basic-shape①"></a>

The [\<basic-shape\>](#typedef-basic-shape) functions are:

<a id="ref-for-funcdef-basic-shape-inset"></a>

<a id="ref-for-typedef-length-percentage"></a>

<a id="ref-for-mult-num-range"></a>

<a id="ref-for-propdef-border-radius"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-funcdef-basic-shape-xywh"></a>

<a id="ref-for-typedef-length-percentage①"></a>

<a id="ref-for-mult-num"></a>

<a id="ref-for-typedef-length-percentage②"></a>

<a id="ref-for-mult-num①"></a>

<a id="ref-for-propdef-border-radius①"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-funcdef-basic-shape-rect"></a>

<a id="ref-for-typedef-length-percentage③"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-mult-num②"></a>

<a id="ref-for-propdef-border-radius②"></a>

<a id="ref-for-mult-opt②"></a>

<a id="typedef-basic-shape-rect"></a>

<a id="ref-for-funcdef-basic-shape-inset①"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-funcdef-basic-shape-rect①"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-funcdef-basic-shape-xywh①"></a>

<a id="ref-for-funcdef-basic-shape-circle"></a>

<a id="ref-for-typedef-radial-size"></a>

<a id="ref-for-mult-opt③"></a>

<a id="ref-for-typedef-position"></a>

<a id="ref-for-mult-opt④"></a>

<a id="ref-for-funcdef-basic-shape-ellipse"></a>

<a id="ref-for-typedef-radial-size①"></a>

<a id="ref-for-mult-opt⑤"></a>

<a id="ref-for-typedef-position①"></a>

<a id="ref-for-mult-opt⑥"></a>

<a id="ref-for-funcdef-basic-shape-polygon"></a>

<a id="ref-for-FillRuleProperty"></a>

<a id="ref-for-mult-opt⑦"></a>

<a id="ref-for-length-value"></a>

<a id="ref-for-mult-opt⑧"></a>

<a id="ref-for-comb-comma"></a>

<a id="ref-for-typedef-length-percentage④"></a>

<a id="ref-for-typedef-length-percentage⑤"></a>

<a id="ref-for-mult-comma"></a>

<a id="ref-for-funcdef-basic-shape-path"></a>

<a id="ref-for-FillRuleProperty①"></a>

<a id="ref-for-mult-opt⑨"></a>

<a id="ref-for-comb-comma①"></a>

<a id="ref-for-string-value"></a>

<a id="ref-for-funcdef-basic-shape-shape"></a>

<a id="ref-for-FillRuleProperty②"></a>

<a id="ref-for-mult-opt①⓪"></a>

<a id="ref-for-typedef-position②"></a>

<a id="ref-for-typedef-shape-command"></a>

<a id="ref-for-mult-comma①"></a>

```text
<inset()> = inset(
  <length-percentage>{1,4}
  [ round <'border-radius'> ]?
)

<xywh()> = xywh(
  <length-percentage>{2} <length-percentage [0,∞]>{2}
  [ round <'border-radius'> ]?
)

<rect()> = rect(
  [ <length-percentage> | auto ]{4}
  [ round <'border-radius'> ]?
)

<basic-shape-rect> = <inset()> | <rect()> | <xywh()>

<circle()> = circle(
  <radial-size>?
  [ at <position> ]?
)

<ellipse()> = ellipse(
  <radial-size>?
  [ at <position> ]?
)

<polygon()> = polygon(
  <'fill-rule'>? 
  [ round <length> ]? ,
  [<length-percentage> <length-percentage>]#
)

<path()> = path(
  <'fill-rule'>? ,
  <string>
)

<shape()> = shape(
  <'fill-rule'>?
  from <position>
  <shape-command>#
)
```
<a id="funcdef-basic-shape-inset"></a>inset()  
<a id="ref-for-basic-shape-reference-box②"></a>

Defines an inset rectangle via insets from each edge of the [reference box](#basic-shape-reference-box).

<a id="ref-for-typedef-length-percentage⑥"></a>

<a id="ref-for-propdef-margin"></a>

If less than four [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values are provided, the omitted values default in the same way as the [margin](https://www.w3.org/TR/CSS2/box.html#propdef-margin) shorthand: an omitted second or third value defaults to the first, and an omitted fourth value defaults to the second.

<a id="ref-for-typedef-length-percentage⑦"></a>

<a id="ref-for-basic-shape-reference-box③"></a>

The four [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)s define the position of the top, right, bottom, and left edges of a rectangle, respectively, as insets from the corresponding edges of the [reference box](#basic-shape-reference-box).

A pair of insets in either dimension that add up to more than the used dimension (such as left and right insets of 75% apiece) use the [CSS Backgrounds 3 § 4.5 Overlapping Curves](https://www.w3.org/TR/css-backgrounds-3/#corner-overlap) rules to proportionally reduce the inset effect to 100%.

<a id="ref-for-basic-shape-reference-box④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4a20658e"></a> For example, specifying inset(75% 0 50% 0) has the top+bottom edges summing to 125% of the [reference box’s](#basic-shape-reference-box) height. They’re proportionally reduced to sum to 100%, identical to specifying inset(60% 0 40% 0).

<a id="ref-for-propdef-border-radius③"></a>

The optional [\<'border-radius'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-radius) argument(s) define rounded corners for the rectangle using the <a id="ref-for-propdef-border-radius④"></a>border-radius shorthand syntax.

Tests

- [inset-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/inset-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/inset-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/inset-function-computed.html)
- [inset-function-invalid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/inset-function-invalid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/inset-function-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/inset-function-invalid.html)
- [inset-function-valid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/inset-function-valid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/inset-function-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/inset-function-valid.html)

<a id="funcdef-basic-shape-xywh"></a>xywh()  
<a id="ref-for-basic-shape-reference-box⑤"></a>

Defines a rectangle via offsets from the top and left edge of the [reference box](#basic-shape-reference-box), and a specified width and height.

<a id="ref-for-typedef-length-percentage⑧"></a>

<a id="ref-for-basic-shape-reference-box⑥"></a>

The four [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)s define, respectively, the inset from the left edge of the [reference box](#basic-shape-reference-box), the inset from the top edge of the <a id="ref-for-basic-shape-reference-box⑦"></a>reference box, the width of the rectangle, and the height of the rectangle.

<a id="ref-for-TermViewBox"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This syntax is inspired by the <code><a href="https://www.w3.org/TR/SVG2/coords.html#TermViewBox">viewBox</a></code> attribute from SVG.

<a id="ref-for-propdef-border-radius⑤"></a>

The optional [\<'border-radius'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-radius) argument(s) define rounded corners for the inset rectangle using the <a id="ref-for-propdef-border-radius⑥"></a>border-radius shorthand syntax.

Tests

- [xywh-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/xywh-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/xywh-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/xywh-function-computed.html)
- [xywh-function-invalid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/xywh-function-invalid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/xywh-function-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/xywh-function-invalid.html)
- [xywh-function-valid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/xywh-function-valid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/xywh-function-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/xywh-function-valid.html)

<a id="funcdef-basic-shape-rect"></a>rect() =  
<a id="ref-for-basic-shape-reference-box⑧"></a>

Defines a rectangle via insets from the top and left edges of the [reference box](#basic-shape-reference-box).

<a id="ref-for-typedef-length-percentage⑨"></a>

<a id="ref-for-basic-shape-reference-box⑨"></a>

The four [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)s define the position of the top, right, bottom, and left edges of a rectangle, respectively, as insets from the top edge of the [reference box](#basic-shape-reference-box) (for the first and third values) or the left edge of the <a id="ref-for-basic-shape-reference-box①⓪"></a>reference box (for the second and fourth values).

<a id="ref-for-basic-shape-reference-box①①"></a>

An auto value makes the edge of the box coincide with the corresponding edge of the [reference box](#basic-shape-reference-box): it’s equivalent to 0% as the first (top) or fourth (left) value, and equivalent to 100% as the second (right) or third (bottom) value.

The second (right) and third (bottom) values are floored by the fourth (left) and second (top) values, respectively.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-203cf4f2"></a> For example, specifying rect(10px 0 0 20px) would place the bottom edge higher than the top edge, and the right edge further left than the left edge, so both are corrected to not cross over the other edge, identical to specifying rect(10px 20px 10px 20px).

<a id="ref-for-funcdef-basic-shape-rect②"></a>

<a id="ref-for-propdef-clip"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This syntax is similar, but not quite identical, to the legacy [rect()](#funcdef-basic-shape-rect) function used solely by the [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip) property.

<a id="ref-for-propdef-border-radius⑦"></a>

The optional [\<'border-radius'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-radius) argument(s) define rounded corners for the rectangle using the <a id="ref-for-propdef-border-radius⑧"></a>border-radius shorthand syntax.)

Tests

- [rect-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/rect-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/rect-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/rect-function-computed.html)
- [rect-function-invalid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/rect-function-invalid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/rect-function-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/rect-function-invalid.html)
- [rect-function-valid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/rect-function-valid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/rect-function-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/rect-function-valid.html)

<a id="funcdef-basic-shape-circle"></a>circle()  
- <a id="ref-for-typedef-radial-size②"></a>

  <a id="ref-for-gradient-box"></a>

  <a id="ref-for-basic-shape-reference-box①②"></a>

  The [\<radial-size\>](https://www.w3.org/TR/css-images-3/#typedef-radial-size) argument defines the circle’s radius. Rather than referring to the [gradient box](https://www.w3.org/TR/css-images-4/#gradient-box), values are resolved against the [reference box](#basic-shape-reference-box).

- <a id="ref-for-typedef-length-percentage①⓪"></a>

  Two [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values are invalid.

  <a id="ref-for-typedef-position③"></a>

  The [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) argument defines the center of the circle. Unless otherwise specified, this defaults to center if omitted.

Tests

- [circle-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/circle-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/circle-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/circle-function-computed.html)
- [circle-function-invalid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/circle-function-invalid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/circle-function-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/circle-function-invalid.html)
- [circle-function-valid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/circle-function-valid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/circle-function-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/circle-function-valid.html)

<a id="funcdef-basic-shape-ellipse"></a>ellipse()  
- <a id="ref-for-typedef-radial-size③"></a>

  <a id="ref-for-gradient-box①"></a>

  <a id="ref-for-basic-shape-reference-box①③"></a>

  The [\<radial-size\>](https://www.w3.org/TR/css-images-3/#typedef-radial-size) argument defines the horizontal and vertical radiuses of the ellipse. Rather than referring to the [gradient box](https://www.w3.org/TR/css-images-4/#gradient-box), values are resolved against the [reference box](#basic-shape-reference-box).

- <a id="ref-for-typedef-position④"></a>

  The [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) argument defines the center of the ellipse. Unless otherwise specified, this defaults to center if omitted.

Tests

- [ellipse-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/ellipse-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/ellipse-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/ellipse-function-computed.html)
- [ellipse-function-invalid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/ellipse-function-invalid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/ellipse-function-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/ellipse-function-invalid.html)
- [ellipse-function-valid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/ellipse-function-valid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/ellipse-function-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/ellipse-function-valid.html)

<a id="funcdef-basic-shape-polygon"></a>polygon()  
- <a id="ref-for-FillRuleProperty③"></a>

  <a id="ref-for-valdef-clip-rule-nonzero"></a>

  The [\<'fill-rule'\>](https://www.w3.org/TR/SVG2/painting.html#FillRuleProperty) specifies the filling rule used to determine the interior. Defaults to [nonzero](https://www.w3.org/TR/css-masking-1/#valdef-clip-rule-nonzero) if omitted.

- <a id="ref-for-length-value①"></a>

  An optional [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) after a round keyword defines rounding for each vertex of the polygon. The length is the radius of a circle whose center lies on the bisector of the smaller angle of the vertex, and that is tangential to both sides of the vertex.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-1217097f"></a>
  > ![rounding concave and convex polygon vertices](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/vertex-rounding.png)
  > Rounding polygon vertices that are both convex and concave.

  To avoid rounding more than half of any line segment, the rounding of each vertex must be clamped separately such that the radius is never more than the smaller of `tan(angle/2) segment / 2` evaluated against both vertex line segments.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > <a id="example-f443dfa5"></a>
  > ![visualization of clamp formula](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/clamp-explanation.png)
  > This diagram shows the intent of the clamping formula.

- <a id="ref-for-typedef-length-percentage①①"></a>

  <a id="ref-for-basic-shape-reference-box①④"></a>

  Each [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) pair specifies a vertex of the polygon, as a horizontal and vertical offset from the left and top edges of the [reference box](#basic-shape-reference-box).

The UA must close a polygon by connecting the last vertex with the first vertex of the list.

Tests

- [polygon-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/polygon-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/polygon-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/polygon-function-computed.html)
- [polygon-function-invalid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/polygon-function-invalid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/polygon-function-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/polygon-function-invalid.html)
- [polygon-function-valid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/polygon-function-valid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/polygon-function-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/polygon-function-valid.html)

<a id="funcdef-basic-shape-path"></a>path()  
- <a id="ref-for-FillRuleProperty④"></a>

  <a id="ref-for-valdef-clip-rule-nonzero①"></a>

  The [\<'fill-rule'\>](https://www.w3.org/TR/SVG2/painting.html#FillRuleProperty) specifies the filling rule used to determine the interior. Defaults to [nonzero](https://www.w3.org/TR/css-masking-1/#valdef-clip-rule-nonzero) if omitted, unless the function is being used in a context such as SVG shapes where the <a id="ref-for-FillRuleProperty⑤"></a>fill-rule property is relevant. In that case an omitted value will use the computed value of the <a id="ref-for-FillRuleProperty⑥"></a>fill-rule property.

- <a id="ref-for-string-value①"></a>

  <a id="ref-for-css-invalid"></a>

  <a id="ref-for-funcdef-basic-shape-path①"></a>

  The [\<string\>](https://www.w3.org/TR/css-values-4/#string-value) represents an [SVG Path data string](https://www.w3.org/TR/SVG11/paths.html#PathData). A path data string that does not conform to the to the grammar and parsing rules of SVG 1.1, or that does conform but defines an empty path, is [invalid](https://www.w3.org/TR/css-syntax-3/#css-invalid) and causes the entire [path()](#funcdef-basic-shape-path) to be <a id="ref-for-css-invalid①"></a>invalid.

  The initial position is defined by the first “move to” argument in the path string. For the initial direction follow SVG 1.1.

<a id="ref-for-propdef-shape-outside⑥"></a>

<a id="ref-for-propdef-clip-path"></a>

The UA must close a path with an implicit closepath command ("z" or "Z") if it is not present in the string for properties that require a closed loop (such as [shape-outside](#propdef-shape-outside) and [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path)).

Tests

- [path-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/path-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/path-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/path-function-computed.html)
- [path-function-invalid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/path-function-invalid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/path-function-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/path-function-invalid.html)
- [path-function-valid.html](https://wpt.fyi/results/css/css-shapes/shape-functions/path-function-valid.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/path-function-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/path-function-valid.html)

<a id="funcdef-basic-shape-shape"></a>shape()  
See [The shape() function](#shape-function).

Tests

<a id="ref-for-funcdef-basic-shape-shape①"></a>

#### <a id="shape-function"></a>3.1.1.  The [shape()](#funcdef-basic-shape-shape) Function

<a id="ref-for-funcdef-basic-shape-path②"></a>

<a id="ref-for-funcdef-var"></a>

<a id="ref-for-px"></a>

While the [path()](#funcdef-basic-shape-path) function allows reuse of the SVG path syntax to define more arbitrary shapes than allowed by more specialized shape functions, it requires writing a path as a single string (which is not compatible with, for example, building a path piecemeal with [var()](https://www.w3.org/TR/css-variables-1/#funcdef-var)), and inherits a number of limitations from SVG, such as implicitly only allowing the [px](https://www.w3.org/TR/css-values-4/#px) unit.

<a id="ref-for-funcdef-basic-shape-shape②"></a>

<a id="ref-for-funcdef-basic-shape-path③"></a>

<a id="ref-for-propdef-clip-path①"></a>

The [shape()](#funcdef-basic-shape-shape) function uses a set of commands roughly equivalent to the ones used by [path()](#funcdef-basic-shape-path), but does so with more standard CSS syntax, and allows the full range of CSS functionality, such as additional units and math functions. The commands used by <a id="ref-for-funcdef-basic-shape-shape③"></a>shape() are dynamically turned into path segments when it is used for rendering, e.g., when computing the rendered [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path).

<a id="ref-for-funcdef-basic-shape-shape④"></a>

<a id="ref-for-funcdef-basic-shape-path④"></a>

<a id="ref-for-em"></a>

In that sense, [shape()](#funcdef-basic-shape-shape) is a superset of [path()](#funcdef-basic-shape-path). A <a id="ref-for-funcdef-basic-shape-path⑤"></a>path() can be easily converted to a <a id="ref-for-funcdef-basic-shape-shape⑤"></a>shape(), but to convert a <a id="ref-for-funcdef-basic-shape-shape⑥"></a>shape() back to a <a id="ref-for-funcdef-basic-shape-path⑥"></a>path() or to SVG requires information about the CSS environment (e.g. current values of CSS custom properties, current font size for [em](https://www.w3.org/TR/css-values-4/#em) units, etc).

<a id="ref-for-FillRuleProperty⑦"></a>

<a id="ref-for-funcdef-basic-shape-path⑦"></a>

The [\<'fill-rule'\>](https://www.w3.org/TR/SVG2/painting.html#FillRuleProperty) is interpreted identically to the same argument in [path()](#funcdef-basic-shape-path).

The rest of the arguments define a list of path data commands, identical to that of an [SVG Path](https://www.w3.org/TR/SVG11/paths.html#PathData), which the function represents.

<a id="ref-for-typedef-shape-coordinate-pair"></a>

The from [\<coordinate-pair\>](#typedef-shape-coordinate-pair) represents the starting point for the first shape-command. It adds an initial [absolute moveto](https://www.w3.org/TR/SVG/paths.html#PathDataMovetoCommands) to the list of path data commands.

<a id="ref-for-typedef-shape-command①"></a>

The sequence of <a id="typedef-shape-command"></a>[\<shape-command\>](#typedef-shape-command)s represent further [path data commands](https://www.w3.org/TR/SVG11/paths.html#PathData). Each command’s starting point is the previous command’s ending point.

<a id="ref-for-typedef-shape-command②"></a>

<a id="ref-for-typedef-shape-move-command"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-typedef-shape-line-command"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-typedef-shape-horizontal-line-command"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-typedef-shape-vertical-line-command"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-typedef-shape-curve-command"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-typedef-shape-smooth-command"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-typedef-shape-arc-command"></a>

<a id="ref-for-typedef-shape-move-command①"></a>

<a id="ref-for-typedef-shape-command-end-point"></a>

<a id="ref-for-typedef-shape-line-command①"></a>

<a id="ref-for-typedef-shape-command-end-point①"></a>

<a id="ref-for-typedef-shape-horizontal-line-command①"></a>

<a id="ref-for-typedef-length-percentage①②"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-typedef-length-percentage①③"></a>

<a id="ref-for-typedef-shape-vertical-line-command①"></a>

<a id="ref-for-typedef-length-percentage①④"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-typedef-length-percentage①⑤"></a>

<a id="ref-for-typedef-shape-curve-command①"></a>

<a id="ref-for-typedef-position⑤"></a>

<a id="ref-for-typedef-shape-control-point"></a>

<a id="ref-for-typedef-shape-control-point①"></a>

<a id="ref-for-mult-opt①①"></a>

<a id="ref-for-comb-one②②"></a>

<a id="ref-for-typedef-shape-coordinate-pair①"></a>

<a id="ref-for-typedef-shape-relative-control-point"></a>

<a id="ref-for-typedef-shape-relative-control-point①"></a>

<a id="ref-for-mult-opt①②"></a>

<a id="ref-for-typedef-shape-smooth-command①"></a>

<a id="ref-for-typedef-position⑥"></a>

<a id="ref-for-typedef-shape-control-point②"></a>

<a id="ref-for-mult-opt①③"></a>

<a id="ref-for-comb-one②③"></a>

<a id="ref-for-typedef-shape-coordinate-pair②"></a>

<a id="ref-for-typedef-shape-relative-control-point②"></a>

<a id="ref-for-mult-opt①④"></a>

<a id="ref-for-typedef-shape-arc-command①"></a>

<a id="ref-for-typedef-shape-command-end-point②"></a>

<a id="ref-for-typedef-length-percentage①⑥"></a>

<a id="ref-for-mult-num-range①"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-typedef-shape-arc-sweep"></a>

<a id="ref-for-mult-opt①⑤"></a>

<a id="ref-for-comb-all①"></a>

<a id="ref-for-typedef-shape-arc-size"></a>

<a id="ref-for-mult-opt①⑥"></a>

<a id="ref-for-comb-all②"></a>

<a id="ref-for-angle-value"></a>

<a id="ref-for-mult-opt①⑦"></a>

<a id="ref-for-typedef-shape-command-end-point③"></a>

<a id="ref-for-typedef-position⑦"></a>

<a id="ref-for-comb-one②④"></a>

<a id="ref-for-typedef-shape-coordinate-pair③"></a>

<a id="ref-for-typedef-shape-control-point③"></a>

<a id="ref-for-typedef-position⑧"></a>

<a id="ref-for-comb-one②⑤"></a>

<a id="ref-for-typedef-shape-relative-control-point③"></a>

<a id="ref-for-typedef-shape-relative-control-point④"></a>

<a id="ref-for-typedef-shape-coordinate-pair④"></a>

<a id="ref-for-comb-one②⑥"></a>

<a id="ref-for-comb-one②⑦"></a>

<a id="ref-for-mult-opt①⑧"></a>

<a id="ref-for-typedef-shape-coordinate-pair⑤"></a>

<a id="ref-for-typedef-length-percentage①⑦"></a>

<a id="ref-for-mult-num③"></a>

<a id="ref-for-typedef-shape-arc-sweep①"></a>

<a id="ref-for-comb-one②⑧"></a>

<a id="ref-for-typedef-shape-arc-size①"></a>

<a id="ref-for-comb-one②⑨"></a>

```text
<shape-command> = <move-command> | <line-command> | close |
                  <horizontal-line-command> | <vertical-line-command> |
                  <curve-command> | <smooth-command> | <arc-command>

<move-command> = move <command-end-point>
<line-command> = line <command-end-point>
<horizontal-line-command> = hline
        [ to [ <length-percentage> | left | center | right | x-start | x-end ]
        | by <length-percentage> ]
<vertical-line-command> = vline
        [ to [ <length-percentage> | top | center | bottom | y-start | y-end ]
        | by <length-percentage> ]
<curve-command> = curve
        [ [ to <position> with <control-point> [ / <control-point> ]? ]
        | [ by <coordinate-pair> with <relative-control-point> [ / <relative-control-point> ]? ] ]
<smooth-command> = smooth
        [ [ to <position> [ with <control-point> ]? ]
        | [ by <coordinate-pair> [ with <relative-control-point> ]? ] ]
<arc-command> = arc <command-end-point>
            [ [ of <length-percentage>{1,2} ]
              && <arc-sweep>? && <arc-size>? && [rotate <angle>]? ]

<command-end-point> = [ to <position> | by <coordinate-pair> ]
<control-point> = [ <position> | <relative-control-point> ]
<relative-control-point> = <coordinate-pair> [ from [ start | end | origin ] ]?
<coordinate-pair> = <length-percentage>{2}
<arc-sweep> = cw | ccw
<arc-size> = large | small
```
<a id="ref-for-typedef-length-percentage①⑧"></a>

<a id="ref-for-typedef-shape-coordinate-pair⑥"></a>

<a id="typedef-shape-coordinate-pair"></a>[\<coordinate-pair\>](#typedef-shape-coordinate-pair) = [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage){2}

<a id="ref-for-basic-shape-reference-box①⑤"></a>

Defines a pair of coordinates, representing a rightward and downward offset, respectively, from a specified reference point. Percentages are resolved against the width or height, respectively, of the [reference box](#basic-shape-reference-box).

<a id="ref-for-typedef-shape-coordinate-pair⑦"></a>

<a id="ref-for-typedef-position⑨"></a>

<a id="ref-for-typedef-shape-command-end-point④"></a>

<a id="typedef-shape-command-end-point"></a>[\<command-end-point\>](#typedef-shape-command-end-point) = \[ <a id="valdef-shape-to"></a>to [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) \| <a id="valdef-shape-by"></a>by [\<coordinate-pair\>](#typedef-shape-coordinate-pair) \]

<a id="ref-for-basic-shape-reference-box①⑥"></a>

<a id="ref-for-typedef-shape-coordinate-pair⑧"></a>

<a id="ref-for-valdef-shape-to"></a>

<a id="ref-for-valdef-shape-by"></a>

Every command can be specified in "absolute" or "relative" coordinates, determined by their [by](#valdef-shape-by) or [to](#valdef-shape-to) component. <a id="ref-for-valdef-shape-to①"></a>to indicates that any [\<coordinate-pair\>](#typedef-shape-coordinate-pair)s in the command are relative to the top-left corner of the [reference box](#basic-shape-reference-box), while <a id="ref-for-valdef-shape-by①"></a>by indicates that the <a id="ref-for-typedef-shape-coordinate-pair⑨"></a>\<coordinate-pair\>s are relative to the command’s starting point.

<a id="ref-for-typedef-shape-relative-control-point⑤"></a>

<a id="ref-for-valdef-shape-by②"></a>

<a id="ref-for-valdef-shape-to②"></a>

<a id="ref-for-typedef-shape-horizontal-line-command②"></a>

<a id="ref-for-typedef-shape-vertical-line-command②"></a>

[\<relative-control-point\>](#typedef-shape-relative-control-point) defines how [by](#valdef-shape-by) and [to](#valdef-shape-to) are interpreted for curve control points, while [\<horizontal-line-command\>](#typedef-shape-horizontal-line-command) and [\<vertical-line-command\>](#typedef-shape-vertical-line-command) define how <a id="ref-for-valdef-shape-by③"></a>by and <a id="ref-for-valdef-shape-to③"></a>to are interpreted for horizontal and vertical lines, respectively.

<a id="ref-for-valdef-shape-to④"></a>

<a id="ref-for-typedef-position①⓪"></a>

<a id="ref-for-typedef-shape-coordinate-pair①⓪"></a>

When [to](#valdef-shape-to) is used, the coordinates can be specified as [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position)s instead of [\<coordinate-pair\>](#typedef-shape-coordinate-pair)s.

<a id="ref-for-percentage-value"></a>

<a id="ref-for-typedef-shape-coordinate-pair①①"></a>

<a id="ref-for-basic-shape-reference-box①⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In either case, [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) values in [\<coordinate-pair\>](#typedef-shape-coordinate-pair)s are always computed relative to the [reference box’s](#basic-shape-reference-box) size.

<a id="ref-for-typedef-shape-command-end-point⑤"></a>

<a id="ref-for-typedef-shape-move-command②"></a>

<a id="typedef-shape-move-command"></a>[\<move-command\>](#typedef-shape-move-command) = <a id="valdef-shape-move"></a>move [\<command-end-point\>](#typedef-shape-command-end-point)

<a id="ref-for-typedef-shape-coordinate-pair①②"></a>

Adds a [moveto](https://www.w3.org/TR/SVG/paths.html#PathDataMovetoCommands) command to the list of path data commands, with an ending point specified by the [\<coordinate-pair\>](#typedef-shape-coordinate-pair).

This draws nothing, and merely "moves the pen" for the next command.

<a id="ref-for-valdef-shape-close"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This starts a new subpath, for the purpose of the [close](#valdef-shape-close) command.

<a id="ref-for-typedef-shape-command-end-point⑥"></a>

<a id="ref-for-typedef-shape-line-command②"></a>

<a id="typedef-shape-line-command"></a>[\<line-command\>](#typedef-shape-line-command) = <a id="valdef-shape-line"></a>line [\<command-end-point\>](#typedef-shape-command-end-point)

<a id="ref-for-typedef-shape-coordinate-pair①③"></a>

Adds a [lineto](https://www.w3.org/TR/SVG/paths.html#PathDataLinetoCommands) command to the list of path data commands, with an ending point specified by the [\<coordinate-pair\>](#typedef-shape-coordinate-pair).

This draws a straight line from the command’s starting point to its ending point.

<a id="ref-for-typedef-length-percentage①⑨"></a>

<a id="ref-for-typedef-shape-horizontal-line-command③"></a>

<a id="typedef-shape-horizontal-line-command"></a>[\<horizontal-line-command\>](#typedef-shape-horizontal-line-command) = hline \[ to \[ [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) \| left \| center \| right \| x-start \| x-end \] \| by <a id="ref-for-typedef-length-percentage②⓪"></a>\<length-percentage\> \]

Adds a horizontal [lineto](https://www.w3.org/TR/SVG/paths.html#PathDataLinetoCommands) command to the list of path data commands.

<a id="ref-for-valdef-shape-line"></a>

<a id="ref-for-typedef-length-percentage②①"></a>

<a id="ref-for-typedef-shape-coordinate-pair①④"></a>

<a id="ref-for-typedef-position①①"></a>

<a id="ref-for-valdef-position-left"></a>

<a id="ref-for-valdef-position-center"></a>

<a id="ref-for-valdef-position-right"></a>

<a id="ref-for-valdef-position-x-start"></a>

<a id="ref-for-valdef-position-x-end"></a>

This is equivalent to a [line](#valdef-shape-line) command with the [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) given as the horizontal component of the [\<coordinate-pair\>](#typedef-shape-coordinate-pair). Specifying the horizontal component of [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) instead of a <a id="ref-for-typedef-length-percentage②②"></a>\<length-percentage\> ([left](https://www.w3.org/TR/css-values-5/#valdef-position-left), [center](https://www.w3.org/TR/css-values-5/#valdef-position-center), [right](https://www.w3.org/TR/css-values-5/#valdef-position-right), [x-start](https://www.w3.org/TR/css-values-5/#valdef-position-x-start), or [x-end](https://www.w3.org/TR/css-values-5/#valdef-position-x-end)), would draw a line to that <a id="ref-for-typedef-position①②"></a>\<position\>, with the <a id="ref-for-typedef-position①③"></a>\<position\>’s vertical component remaining the same as the starting point.

<a id="ref-for-typedef-length-percentage②③"></a>

<a id="ref-for-typedef-shape-vertical-line-command③"></a>

<a id="typedef-shape-vertical-line-command"></a>[\<vertical-line-command\>](#typedef-shape-vertical-line-command) = vline \[ to \[ [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) \| top \| center \| bottom \| y-start \| y-end \] \| by <a id="ref-for-typedef-length-percentage②④"></a>\<length-percentage\> \]

Adds a vertical [lineto](https://www.w3.org/TR/SVG/paths.html#PathDataLinetoCommands) command to the list of path data commands.

<a id="ref-for-valdef-shape-line①"></a>

<a id="ref-for-typedef-length-percentage②⑤"></a>

<a id="ref-for-typedef-shape-coordinate-pair①⑤"></a>

<a id="ref-for-typedef-position①④"></a>

<a id="ref-for-valdef-position-top"></a>

<a id="ref-for-valdef-position-center①"></a>

<a id="ref-for-valdef-position-bottom"></a>

<a id="ref-for-valdef-position-y-start"></a>

<a id="ref-for-valdef-position-y-end"></a>

This is equivalent to a [line](#valdef-shape-line) command with the [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) given as the vertical component of the [\<coordinate-pair\>](#typedef-shape-coordinate-pair). Specifying the horizontal component of [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) ([top](https://www.w3.org/TR/css-values-5/#valdef-position-top), [center](https://www.w3.org/TR/css-values-5/#valdef-position-center), [bottom](https://www.w3.org/TR/css-values-5/#valdef-position-bottom), [y-start](https://www.w3.org/TR/css-values-5/#valdef-position-y-start), or [y-end](https://www.w3.org/TR/css-values-5/#valdef-position-y-end)) instead of a <a id="ref-for-typedef-length-percentage②⑥"></a>\<length-percentage\>, would draw a line to that <a id="ref-for-typedef-position①⑤"></a>\<position\>, with the <a id="ref-for-typedef-position①⑥"></a>\<position\>’s horizontal component remaining the same as the starting point.

<a id="ref-for-typedef-shape-relative-control-point⑥"></a>

<a id="ref-for-typedef-shape-coordinate-pair①⑥"></a>

<a id="ref-for-typedef-shape-control-point④"></a>

<a id="ref-for-typedef-position①⑦"></a>

<a id="ref-for-typedef-shape-curve-command②"></a>

<a id="typedef-shape-curve-command"></a>[\<curve-command\>](#typedef-shape-curve-command) = <a id="valdef-shape-curve"></a>curve \[ \[ to [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) with [\<control-point\>](#typedef-shape-control-point) \[ / <a id="ref-for-typedef-shape-control-point⑤"></a>\<control-point\> \]? \] \| \[ by [\<coordinate-pair\>](#typedef-shape-coordinate-pair) with [\<relative-control-point\>](#typedef-shape-relative-control-point) \[ / <a id="ref-for-typedef-shape-relative-control-point⑦"></a>\<relative-control-point\> \]? \] \]

<a id="ref-for-typedef-shape-command-end-point⑦"></a>

<a id="ref-for-valdef-shape-by④"></a>

<a id="ref-for-typedef-shape-coordinate-pair①⑦"></a>

<a id="ref-for-valdef-shape-to⑤"></a>

<a id="ref-for-typedef-position①⑧"></a>

Adds a Bézier curve command to the list of path data commands, ending at the point specified by the [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) following the [to](#valdef-shape-to) keyword, or the [\<coordinate-pair\>](#typedef-shape-coordinate-pair) following the [by](#valdef-shape-by) keyword, as specified by [\<command-end-point\>](#typedef-shape-command-end-point).

<a id="ref-for-typedef-shape-control-point⑥"></a>

<a id="ref-for-typedef-shape-relative-control-point⑧"></a>

The with component specifies control points for the curve: if a single [\<control-point\>](#typedef-shape-control-point) or [\<relative-control-point\>](#typedef-shape-relative-control-point) is provided, the command specifies a [quadratic curve](https://www.w3.org/TR/SVG/paths.html#PathDataQuadraticBezierCommands); if two <a id="ref-for-typedef-shape-control-point⑦"></a>\<control-point\>s or <a id="ref-for-typedef-shape-relative-control-point⑨"></a>\<relative-control-point\>s are provided, it specifies a [cubic curve](https://www.w3.org/TR/SVG/paths.html#PathDataCubicBezierCommands).

<a id="ref-for-typedef-shape-relative-control-point①⓪"></a>

<a id="ref-for-typedef-shape-coordinate-pair①⑧"></a>

<a id="ref-for-typedef-shape-control-point⑧"></a>

<a id="ref-for-typedef-position①⑨"></a>

<a id="ref-for-typedef-shape-smooth-command②"></a>

<a id="typedef-shape-smooth-command"></a>[\<smooth-command\>](#typedef-shape-smooth-command) = <a id="valdef-shape-smooth"></a>smooth \[ \[ to [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) \[with [\<control-point\>](#typedef-shape-control-point) \]? \] \| \[ by [\<coordinate-pair\>](#typedef-shape-coordinate-pair) \[ with [\<relative-control-point\>](#typedef-shape-relative-control-point) \]? \] \]

<a id="ref-for-typedef-shape-command-end-point⑧"></a>

<a id="ref-for-valdef-shape-by⑤"></a>

<a id="ref-for-typedef-shape-coordinate-pair①⑨"></a>

<a id="ref-for-valdef-shape-to⑥"></a>

<a id="ref-for-typedef-position②⓪"></a>

Adds a smooth Bézier curve command to the list of path data commands, ending at the point specified by the [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) following the [to](#valdef-shape-to) keyword, or the [\<coordinate-pair\>](#typedef-shape-coordinate-pair) following the [by](#valdef-shape-by) keyword, as specified by [\<command-end-point\>](#typedef-shape-command-end-point). The with component specifies control points for the curve: if it’s omitted, the command specifies a [smooth quadratic curve](https://www.w3.org/TR/SVG/paths.html#PathDataQuadraticBezierCommands); if it’s provided, if specifies a [smooth cubic curve](https://www.w3.org/TR/SVG/paths.html#PathDataCubicBezierCommands).

<a id="ref-for-valdef-shape-smooth"></a>

<a id="ref-for-valdef-shape-curve"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A [smooth](#valdef-shape-smooth) command is equivalent to a [curve](#valdef-shape-curve) command with the first control point automatically specified as the reflection of the previous curve’s second control point around the starting point, or as the starting point if the previous path data command wasn’t a curve. This ensures G1 continuity with the previous command, so the curve appears to smoothly continue from the previous command, rather than possibly making a sudden direction change.

<a id="ref-for-typedef-shape-relative-control-point①①"></a>

<a id="ref-for-typedef-position②①"></a>

<a id="ref-for-typedef-shape-control-point⑨"></a>

<a id="typedef-shape-control-point"></a>[\<control-point\>](#typedef-shape-control-point) = \[ [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) \| [\<relative-control-point\>](#typedef-shape-relative-control-point) \]

Provides a control point to a quadratic or cubic Bézier curve.

<a id="ref-for-typedef-shape-coordinate-pair②⓪"></a>

<a id="ref-for-typedef-shape-relative-control-point①②"></a>

<a id="typedef-shape-relative-control-point"></a>[\<relative-control-point\>](#typedef-shape-relative-control-point) = [\<coordinate-pair\>](#typedef-shape-coordinate-pair) \[ from \[ start \| end \| origin \] \]?

<a id="ref-for-basic-shape-reference-box①⑧"></a>

<a id="ref-for-typedef-shape-coordinate-pair②①"></a>

Provides a control point to a quadratic or cubic Bézier curve. When a from keyword is specified followed by start, end, or origin, the given [\<coordinate-pair\>](#typedef-shape-coordinate-pair) is relative to the command’s starting point, the command’s end point, or the [reference box](#basic-shape-reference-box), respectively. If such component is not provided, the <a id="ref-for-typedef-shape-coordinate-pair②②"></a>\<coordinate-pair\> is relative to the segment’s start.

<a id="ref-for-angle-value①"></a>

<a id="ref-for-typedef-shape-arc-size②"></a>

<a id="ref-for-typedef-shape-arc-sweep②"></a>

<a id="ref-for-typedef-length-percentage②⑦"></a>

<a id="ref-for-typedef-shape-command-end-point⑨"></a>

<a id="ref-for-typedef-shape-arc-command②"></a>

<a id="typedef-shape-arc-command"></a>[\<arc-command\>](#typedef-shape-arc-command) = <a id="valdef-shape-arc"></a>arc [\<command-end-point\>](#typedef-shape-command-end-point) \[of [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage){1,2}\] &#x26;&#x26; [\<arc-sweep\>](#typedef-shape-arc-sweep)? &#x26;&#x26; [\<arc-size\>](#typedef-shape-arc-size)? &#x26;&#x26; rotate [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value)? \]

<a id="ref-for-typedef-shape-command-end-point①⓪"></a>

Add an [elliptical arc](https://www.w3.org/TR/SVG/paths.html#PathDataEllipticalArcCommands) command to the list of path data commands, ending at the [\<command-end-point\>](#typedef-shape-command-end-point).

<a id="ref-for-typedef-length-percentage②⑧"></a>

<a id="ref-for-typedef-shape-coordinate-pair②③"></a>

<a id="ref-for-percentage-value①"></a>

<a id="ref-for-basic-shape-reference-box①⑨"></a>

The of component specifies the size of the ellipse that the arc is taken from. The first [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) provides the horizontal radius of the ellipse and the second provides the vertical radius. Like for [\<coordinate-pair\>](#typedef-shape-coordinate-pair)s, [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) values are resolved against the width or height of the [reference box](#basic-shape-reference-box), as appropriate.

<a id="ref-for-typedef-length-percentage②⑨"></a>

<a id="ref-for-percentage-value②"></a>

<a id="ref-for-direction-agnostic-size①"></a>

<a id="ref-for-basic-shape-reference-box②⓪"></a>

<a id="ref-for-funcdef-basic-shape-circle①"></a>

If only one [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) is provided, both radiuses use the provided value. In that case, [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) values are resolved against the [direction-agnostic size](#direction-agnostic-size) of the [reference box](#basic-shape-reference-box) (similar to the [circle()](#funcdef-basic-shape-circle) function).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that SVG has [some specific error-handling for the ellipse radiuses](https://www.w3.org/TR/SVG2/paths.html#ArcOutOfRangeParameters):
>
> - if the endpoint is the same as the starting point, the command does nothing
>
> - <a id="ref-for-typedef-shape-line-command③"></a>
>
>   if either radius is zero, the command is equivalent to a [\<line-command\>](#typedef-shape-line-command) to the ending point
>
> - if either radius is negative, its absolute value is used instead
>
> - <a id="ref-for-angle-value②"></a>
>
>   if the radiuses don’t describe an ellipse large enough to intersect both the starting point and ending point (after rotation by the specified [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value)), they are scaled up uniformly until the ellipse is just large enough to reach.

<a id="ref-for-angle-value③"></a>

<a id="ref-for-funcdef-transform-rotate"></a>

The ellipse described by the specified radiuses defaults to being axis-aligned, but can be rotated by specifying an [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value). Similar to the [rotate()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-rotate) transform function, positive angles specify a clockwise rotation, and negative angles specify a counterclockwise rotation. If omitted, this defaults to 0deg.

<a id="ref-for-typedef-shape-arc-sweep③"></a>

<a id="ref-for-typedef-shape-arc-size③"></a>

The ending point, radiuses, and angle, taken together, usually define two possible ellipses that intersect the starting point and ending point, and each ellipse can be traced in either direction, for a total of four possible arcs. The [\<arc-sweep\>](#typedef-shape-arc-sweep) and [\<arc-size\>](#typedef-shape-arc-size) components specify which of these arcs is desired:

- <a id="ref-for-typedef-shape-arc-sweep④"></a>

  <a id="ref-for-valdef-shape-ccw"></a>

  <a id="typedef-shape-arc-sweep"></a>[\<arc-sweep\>](#typedef-shape-arc-sweep) can be <a id="valdef-shape-cw"></a>cw or <a id="valdef-shape-ccw"></a>ccw, indicating that the arc that is traced around the ellipse clockwise or counter-clockwise from the center, respectively, must be chosen. If omitted, this defaults to [ccw](#valdef-shape-ccw).

  <a id="ref-for-valdef-shape-cw"></a>

  <a id="ref-for-valdef-shape-ccw①"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: In the SVG arc command, [cw](#valdef-shape-cw) corresponds to the value 1 for the sweep flag, and [ccw](#valdef-shape-ccw) to the value 0.

- <a id="ref-for-typedef-shape-arc-size④"></a>

  <a id="ref-for-valdef-shape-small"></a>

  <a id="typedef-shape-arc-size"></a>[\<arc-size\>](#typedef-shape-arc-size) can be <a id="valdef-shape-large"></a>large or <a id="valdef-shape-small"></a>small, indicating that the larger or smaller, respectively, of the two possible arcs must be chosen. If omitted, this defaults to [small](#valdef-shape-small).

  <a id="ref-for-valdef-shape-large"></a>

  <a id="ref-for-valdef-shape-small①"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: In the SVG arc command, [large](#valdef-shape-large) corresponds to the value 1 for the large flag, and [small](#valdef-shape-small) to the 0.

  <a id="ref-for-typedef-shape-arc-sweep⑤"></a>

  <a id="ref-for-typedef-shape-arc-size⑤"></a>

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: If the starting and ending points are on exactly opposite sides of the ellipse, both possible arcs are the same size, but also there is only one possible ellipse. In this case, the [\<arc-sweep\>](#typedef-shape-arc-sweep) distinguishes which of the two possible arcs will be chosen, and [\<arc-size\>](#typedef-shape-arc-size) has no effect.

![a depiction of four possible arcs given a start and end point](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/four-arcs.svg)

A depiction of the two possible ellipses, and four possible arcs, that can be chosen between.

<a id="valdef-shape-close"></a>close

Adds a [closepath](https://www.w3.org/TR/SVG/paths.html#PathDataClosePathCommand) command to the list of path data commands.

<a id="ref-for-valdef-shape-line②"></a>

<a id="ref-for-valdef-shape-close①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is similar to a [line](#valdef-shape-line) command with its ending point set to the starting point of the subpath. When specifying a raw shape, they’re identical, but if the path is stroked, the ending point of the [close](#valdef-shape-close) command is smoothly joined with the start of the subpath, which affects how line-joins and line-caps are rendered.

<a id="ref-for-funcdef-basic-shape-shape⑦"></a>

##### <a id="shape-examples"></a>3.1.1.1. Using [shape()](#funcdef-basic-shape-shape) to create responsive, parametric speech bubble

<a id="ref-for-funcdef-basic-shape-shape⑧"></a>

<a id="ref-for-funcdef-basic-shape-polygon①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c1f9dfe4"></a> The [shape()](#funcdef-basic-shape-shape) function enables shapes that are responsive, rather than scalable. While the [polygon()](#funcdef-basic-shape-polygon) shape is also responsive, it only support simple rounded corners and not complex curves.
>
> To demonstrate, let’s start with a speech bubble, such as the following:
>
> ![A speech bubble shape](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/bubble.svg)
>
> <a id="ref-for-funcdef-basic-shape-path⑧"></a>
>
> Using this shape with a clip-path can be done by using the [path()](#funcdef-basic-shape-path) function:
>
> ```text
> .bubble { clip-path: path("m 5 0 H 95 Q 100 0 100 5 V 92 Q 100 97 95 97 H 70 l -2 3 l -2 -3 H 5 Q 0 97 0 92 V 5 Q 0 0 5 0") };
> ```
>
> Altohugh this path can easily scale, the scaled results are not always desirable. e.g. when scaled to a small balloon, the arrow and corners are scaled to become almost invisible:
>
> ![scled-down speech bubble shape](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/bubble.svg)
>
> <a id="ref-for-funcdef-basic-shape-shape⑨"></a>
>
> To construct this shape using the [shape()](#funcdef-basic-shape-shape) function, let’s start by turning all the pixel values from the path function to percentages. Note that the <a id="ref-for-funcdef-basic-shape-shape①⓪"></a>shape() function begins with from:
>
> ```text
> .bubble { clip-path: shape(  from 5% 0%,
>               hline to 95%,
>               curve to 100% 5% with 100% 0%,
>               vline to 92%,
>               curve to 95% 97% with 100% 97%,
>               hline to 70%,
>               line by -2% 3%,
>               line by -2% -3%,
>               hline to 5%,
>               curve to 0% 92% with 0% 97%,
>               vline to 5%,
>               curve to 5% 0% with 0% 0%); }
> ```
>
> <a id="ref-for-px①"></a>
>
> To make this path responsive, as in, respond well to size changes, we will convert some of its units to [px](https://www.w3.org/TR/css-values-4/#px) values, specifically the ones the control the curves and arrows:
>
> ```text
> .bubble { clip-path: shape(  from 5px 0%,
>               hline to calc(100% - 5px),
>               curve to 100% 5px with 100% 0%,
>               vline to calc(100% - 8px),
>               curve to calc(100% - 5px) calc(100% - 3px) with 100% calc(100% - 3px),
>               hline to 70%,
>               line by -2px 3px,
>               line by -2px -3px,
>               hline to 5px,
>               curve to 0% calc(100% - 8px) with 0% calc(100% - 3px),
>               vline to 5px,
>               curve to 5px 0% with 0% 0%); }
> ```
>
> When applied as clip-path, it would looks like the following:
>
> ![A speech bubble shape, using clip-path](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/bubble-50.svg)
>
> The whole speech bubble is scaled to the reference box, while the curves and arrows stay more constant.
>
> <a id="ref-for-funcdef-basic-shape-shape①①"></a>
>
> Since [shape()](#funcdef-basic-shape-shape) uses CSS units, we can replace some of the edges with position values:
>
> ```text
> .bubble { clip-path: shape(from 5px 0,
>               hline to calc(100% - 5px),
>               curve to right 5px with right top,
>               vline to calc(100% - 8px),
>               curve to calc(100% - 5px) calc(100% - 3px) with right calc(100% - 3px),
>               hline to 70%,
>               line by -2px 3px,
>               line by -2px -3px,
>               hline to 5px,
>               curve to left calc(100% - 8px) with left calc(100% - 3px),
>               vline to 5px,
>               curve to 5px top with left top); }
> ```
>
> <a id="ref-for-funcdef-basic-shape-shape①②"></a>
>
> Another useful feature of [shape()](#funcdef-basic-shape-shape) is that it can be used alongside CSS properties. In this case, we can make the arrow and radius parametric:
>
> ```text
> :root {
>   --radius: 5px;
>   --arrow-length: 3px;
>   --arrow-half-width: 2px;
>   --arrow-position: 70%;
>   --arrow-bottom-offset: calc(100% - var(--radius) - var(--arrow-length));
> }
> 
> .bubble {
>   animation: bubble 100ms;
>   clip-path: shape(from var(---radius) top,
>     hline to calc(100% - var(---radius)),
>     curve to right var(---radius) with right top,
>     vline to var(---arrow-bottom-offset),
>     curve to calc(100% - var(---radius)) calc(100% - var(---arrow-length))
>               with right calc(100% - var(---arrow-length)),
>     hline to var(---arrow-position),
>     line by var(---arrow-half-width) var(---arrow-length),
>     line by var(---arrow-half-width) calc(0px - var(---arrow-length)),
>     hline to var(---radius),
>     curve to left var(---arrow-bottom-offset) with left calc(100% - var(---arrow-length)),
>     vline to var(---radius),
>     curve to var(---radius) top with left top); }
> ```
<a id="ref-for-funcdef-basic-shape-shape①③"></a>

##### <a id="interpolating-shape"></a>3.1.1.2.  Interpolating the [shape()](#funcdef-basic-shape-shape) Function

<a id="ref-for-funcdef-basic-shape-shape①④"></a>

<a id="ref-for-funcdef-basic-shape-path⑨"></a>

<a id="ref-for-interpolation"></a>

<a id="ref-for-typedef-shape-coordinate-pair②④"></a>

[shape()](#funcdef-basic-shape-shape) and [path()](#funcdef-basic-shape-path) functions can be [interpolated](https://www.w3.org/TR/css-values-4/#interpolation) with each other if their associated list of path data commands is the same length and has the same commands, in order, with the first command of the <a id="ref-for-funcdef-basic-shape-path①⓪"></a>path() function interpolating with the initial [\<coordinate-pair\>](#typedef-shape-coordinate-pair) in the <a id="ref-for-funcdef-basic-shape-shape①⑤"></a>shape() function.

<a id="ref-for-funcdef-basic-shape-path①①"></a>

<a id="ref-for-valdef-shape-move"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The first command of a [path()](#funcdef-basic-shape-path) function is guaranteed to be a [move](#valdef-shape-move), see [moveTo](https://www.w3.org/TR/SVG2/paths.html#PathDataMovetoCommands) in the SVG spec.

<a id="ref-for-funcdef-basic-shape-path①②"></a>

<a id="ref-for-funcdef-basic-shape-shape①⑥"></a>

If the starting and ending values are both [path()](#funcdef-basic-shape-path) functions, the interpolated value is a <a id="ref-for-funcdef-basic-shape-path①③"></a>path() function; otherwise it’s a [shape()](#funcdef-basic-shape-shape) function. In either case, the interpolated value must represent the same list of path data commands, with each command having its numerical components interpolated between the corresponding components of the starting and ending list.

<a id="ref-for-valdef-shape-curve①"></a>

<a id="ref-for-valdef-shape-smooth①"></a>

For this purpose, commands are "the same" if they use the same command keyword, and use the same \<by-to\> keyword. For [curve](#valdef-shape-curve) and [smooth](#valdef-shape-smooth), they also must have the same number of control points.

<a id="ref-for-valdef-shape-arc"></a>

<a id="ref-for-typedef-shape-arc-sweep⑥"></a>

<a id="ref-for-valdef-shape-cw①"></a>

<a id="ref-for-typedef-shape-arc-size⑥"></a>

<a id="ref-for-valdef-shape-large①"></a>

If an [arc](#valdef-shape-arc) command has different [\<arc-sweep\>](#typedef-shape-arc-sweep) between its starting and ending list, then the interpolated result uses [cw](#valdef-shape-cw) for any progress value between 0 and 1. If it has different [\<arc-size\>](#typedef-shape-arc-size) keywords, then the interpolated result uses [large](#valdef-shape-large) for any progress value between 0 and 1.

<a id="ref-for-valdef-shape-arc①"></a>

<a id="ref-for-elementdef-path"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [arc](#valdef-shape-arc) keyword interpolation rules are meant to match existing SVG <code><a href="https://www.w3.org/TR/SVG2/paths.html#elementdef-path">path</a></code> interpolation rules.

### <a id="basic-shape-computed-values"></a>3.2.  Computed Values of Basic Shapes

<a id="ref-for-typedef-basic-shape②"></a>

The values in a [\<basic-shape\>](#typedef-basic-shape) function are computed as specified, with these exceptions:

- Omitted values are included and compute to their defaults.

- <a id="ref-for-typedef-length-percentage③⓪"></a>

  <a id="ref-for-funcdef-basic-shape-ellipse①"></a>

  <a id="ref-for-funcdef-basic-shape-circle②"></a>

  <a id="ref-for-typedef-position②②"></a>

  A [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) value in [circle()](#funcdef-basic-shape-circle) or [ellipse()](#funcdef-basic-shape-ellipse) is computed as a pair of offsets (horizontal then vertical) from the top left origin, each given as a [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage).

- <a id="ref-for-typedef-length-percentage③①"></a>

  <a id="ref-for-typedef-basic-shape-rect"></a>

  <a id="ref-for-propdef-border-radius⑨"></a>

  A [\<'border-radius'\>](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-radius) value in a [\<basic-shape-rect\>](#typedef-basic-shape-rect) function is computed as an expanded list of all eight [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values.

- <a id="ref-for-funcdef-basic-shape-inset②"></a>

  <a id="ref-for-typedef-basic-shape-rect①"></a>

  All [\<basic-shape-rect\>](#typedef-basic-shape-rect) functions compute to the equivalent [inset()](#funcdef-basic-shape-inset) function.

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: Given rect(t r b l), the equivalent function is inset(t calc(100% - r) calc(100% - b) l). Given xywh(x y w h), the equivalent function is inset(y calc(100% - x - w) calc(100% - y - h) x).

Tests

- [shape-image-threshold-computed.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-image-threshold-computed.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-image-threshold-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-image-threshold-computed.html)
- [shape-margin-computed.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-margin-computed.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-margin-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-margin-computed.html)
- [shape-outside-computed.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-outside-computed.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-outside-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-outside-computed.html)
- [circle-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/circle-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/circle-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/circle-function-computed.html)
- [ellipse-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/ellipse-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/ellipse-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/ellipse-function-computed.html)
- [inset-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/inset-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/inset-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/inset-function-computed.html)
- [path-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/path-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/path-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/path-function-computed.html)
- [polygon-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/polygon-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/polygon-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/polygon-function-computed.html)
- [rect-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/rect-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/rect-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/rect-function-computed.html)
- [xywh-function-computed.html](https://wpt.fyi/results/css/css-shapes/shape-functions/xywh-function-computed.html) [(live test)](http://wpt.live/css/css-shapes/shape-functions/xywh-function-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-functions/xywh-function-computed.html)
- [shape-outside-computed-shape-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-computed-shape-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-computed-shape-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-computed-shape-000.html)
- [shape-outside-computed-shape-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-computed-shape-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-computed-shape-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-computed-shape-001.html)

### <a id="basic-shape-serialization"></a>3.3.  Serialization of Basic Shapes

<a id="ref-for-typedef-basic-shape③"></a>

<a id="ref-for-computed-value"></a>

To serialize the [\<basic-shape\>](#typedef-basic-shape) functions, serialize as per their individual grammars, in the order the grammars are written in, joining space-separated tokens with a single space, and following each serialized comma with a single space. For serializing [computed values](https://www.w3.org/TR/css-cascade-5/#computed-value), component values are <a id="ref-for-computed-value①"></a>computed, and omitted when possible without changing the meaning.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1f2eff1c"></a>
>
> <a id="ref-for-specified-value"></a>
>
> <a id="ref-for-computed-value②"></a>
>
> <a id="ref-for-funcdef-basic-shape-circle③"></a>
>
> <a id="ref-for-propdef-shape-outside⑦"></a>
>
> As [specified value](https://www.w3.org/TR/css-cascade-5/#specified-value) serialization of the shape functions are relatively trivial, here are some examples of [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) serializations for [circle()](#funcdef-basic-shape-circle) notations when used in [shape-outside](#propdef-shape-outside):
>
> - <a id="ref-for-typedef-position②③"></a>
>
>   <a id="ref-for-computed-value③"></a>
>
>   [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) [serialization rules](https://www.w3.org/TR/css-values-4/#position-serialization) mean that keywords [compute to](https://www.w3.org/TR/css-cascade-5/#computed-value) percentages, and serialize in horizontal-vertical order.
>
>   ```text
>   circle(at bottom left)
>   /* serializes to */
>   circle(at 0% 100%)
>   ```
>
> - Omitting optional components means that default values do not show up in the serialization.
>
>   ```text
>   circle(closest-side at center)
>   /* serializes to */
>   circle()
>   ```
>
> - Value [computation](https://www.w3.org/TR/css-cascade-5/#computed) means that some functions [canonicalize to a different form](#basic-shape-computed-values).
>
>   ```text
>   rect(10px 20px 30px 40px)
>   /* serializes to */
>   inset(10px calc(100% - 20px) calc(100% - 30px) 40px)
>   ```
Tests

- [basic-shape-circle-ellipse-serialization.html](https://wpt.fyi/results/css/css-shapes/basic-shape-circle-ellipse-serialization.html) [(live test)](http://wpt.live/css/css-shapes/basic-shape-circle-ellipse-serialization.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/basic-shape-circle-ellipse-serialization.html)

### <a id="basic-shape-interpolation"></a>3.4.  Interpolation of Basic Shapes

<a id="ref-for-valdef-clip-rule-nonzero②"></a>

For interpolating between one basic shape and a second, the rules below are applied. The values in the shape functions interpolate [by computed value](https://www.w3.org/TR/web-animations-1/#by-computed-value). The list values interpolate as length, percentage, or calc where possible. If list values are not one of those types but are identical (such as finding [nonzero](https://www.w3.org/TR/css-masking-1/#valdef-clip-rule-nonzero) in the same list position in both lists) those values do interpolate.

- <a id="ref-for-basic-shape-reference-box②①"></a>

  Both shapes must use the same [reference box](#basic-shape-reference-box).

- <a id="ref-for-typedef-length-percentage③②"></a>

  <a id="ref-for-funcdef-basic-shape-circle④"></a>

  <a id="ref-for-funcdef-basic-shape-ellipse②"></a>

  If both shapes are the same type, that type is [ellipse()](#funcdef-basic-shape-ellipse) or [circle()](#funcdef-basic-shape-circle), and the radiuses are specified as [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) (rather than keywords), interpolate between each value in the shape functions.

- <a id="ref-for-funcdef-basic-shape-inset③"></a>

  If both shapes are of type [inset()](#funcdef-basic-shape-inset), interpolate between each value in the shape functions.

- <a id="ref-for-FillRuleProperty⑧"></a>

  <a id="ref-for-funcdef-basic-shape-polygon②"></a>

  If both shapes are of type [polygon()](#funcdef-basic-shape-polygon), both polygons have the same number of vertices, and use the same [\<'fill-rule'\>](https://www.w3.org/TR/SVG2/painting.html#FillRuleProperty), interpolate between each value in the shape functions.

- In all other cases no interpolation is specified.

Tests

- [shape-image-threshold-interpolation.html](https://wpt.fyi/results/css/css-shapes/animation/shape-image-threshold-interpolation.html) [(live test)](http://wpt.live/css/css-shapes/animation/shape-image-threshold-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/animation/shape-image-threshold-interpolation.html)
- [shape-margin-composition.html](https://wpt.fyi/results/css/css-shapes/animation/shape-margin-composition.html) [(live test)](http://wpt.live/css/css-shapes/animation/shape-margin-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/animation/shape-margin-composition.html)
- [shape-margin-interpolation.html](https://wpt.fyi/results/css/css-shapes/animation/shape-margin-interpolation.html) [(live test)](http://wpt.live/css/css-shapes/animation/shape-margin-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/animation/shape-margin-interpolation.html)
- [shape-outside-composition.html](https://wpt.fyi/results/css/css-shapes/animation/shape-outside-composition.html) [(live test)](http://wpt.live/css/css-shapes/animation/shape-outside-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/animation/shape-outside-composition.html)
- [shape-outside-interpolation.html](https://wpt.fyi/results/css/css-shapes/animation/shape-outside-interpolation.html) [(live test)](http://wpt.live/css/css-shapes/animation/shape-outside-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/animation/shape-outside-interpolation.html)
- [basic-shape-interpolation.html](https://wpt.fyi/results/css/css-shapes/basic-shape-interpolation.html) [(live test)](http://wpt.live/css/css-shapes/basic-shape-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/basic-shape-interpolation.html)

## <a id="shapes-from-image"></a>4.  Shapes from Image

<a id="ref-for-typedef-image"></a>

<a id="ref-for-propdef-shape-image-threshold"></a>

Another way of defining shapes is by specifying a source [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) whose alpha channel is used to compute the shape. The shape is computed to be the path or paths that enclose the area(s) where the opacity of the specified image is greater than the [shape-image-threshold](#propdef-shape-image-threshold) value. The absence of any pixels with an alpha value greater than the specified threshold results in an empty float area that will not affect wrapping. If the <a id="ref-for-propdef-shape-image-threshold①"></a>shape-image-threshold is not specified, the initial value to be considered is 0.0.

The image is sized and positioned as if it were a replaced element whose specified width and height are the same as the element’s used content box size.

For animated raster image formats (such as [GIF](https://www.w3.org/Graphics/GIF/spec-gif89a.txt)), the first frame of the animation sequence is used.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-12b6dfcf"></a>
>
> An image is floating to the left of a paragraph. The image shows the 3D version of the CSS logo over a transparent background. The logo has a shadow using an alpha-channel.
>
> <a id="ref-for-float-area①⑧"></a>
>
> <a id="ref-for-propdef-shape-outside⑧"></a>
>
> The image defines its [float area](#float-area) through the [shape-outside](#propdef-shape-outside) property.
>
> ```text
> 
>   <p>
>     <img id="CSSlogo" src="CSS-logo1s.png"/>
>     blah blah blah blah...
>   </p>
> 
>   <style>
>     #CSSlogo {
>       float: left;
>       shape-outside: attr(src url);
>       shape-image-threshold: 0.1;
>     }
>   </style>
> ```
>
> <a id="ref-for-propdef-shape-outside⑨"></a>
>
> The [shape-outside](#propdef-shape-outside) property re-uses the url from the src attribute of the img element.
>
> <a id="ref-for-float-area①⑨"></a>
>
> It is perfectly possible to display an image and use a different image for its [float area](#float-area).
>
> In the figure below, the alpha-channel threshold is represented by the dotted line around the CSS logo.
>
> It’s then possible to affect where the lines of the paragraph start in three ways:
>
> 1.  Modifying the alpha channel in the image
>
> 2.  <a id="ref-for-propdef-shape-image-threshold②"></a>
>
>     Changing the value of the [shape-image-threshold](#propdef-shape-image-threshold) property
>
> 3.  <a id="ref-for-propdef-shape-margin①"></a>
>
>     Changing the value of the [shape-margin](#propdef-shape-margin) property (see example 8)
>
> ![A float shape around an image using its alpha-channel](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/shape-outside-image.png)
>
> A float shape around an image using its alpha-channel.

Tests

- [shape-image-threshold-interpolation.html](https://wpt.fyi/results/css/css-shapes/animation/shape-image-threshold-interpolation.html) [(live test)](http://wpt.live/css/css-shapes/animation/shape-image-threshold-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/animation/shape-image-threshold-interpolation.html)
- [shape-image-threshold-computed.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-image-threshold-computed.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-image-threshold-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-image-threshold-computed.html)
- [shape-image-threshold-invalid.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-image-threshold-invalid.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-image-threshold-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-image-threshold-invalid.html)
- [shape-image-threshold-valid.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-image-threshold-valid.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-image-threshold-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-image-threshold-valid.html)
- [float-retry-push-image.html](https://wpt.fyi/results/css/css-shapes/shape-outside/assorted/float-retry-push-image.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/assorted/float-retry-push-image.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/assorted/float-retry-push-image.html)
- [shape-outside-linear-gradient-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-001.html)
- [shape-outside-linear-gradient-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-002.html)
- [shape-outside-linear-gradient-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-003.html)
- [shape-outside-linear-gradient-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-004.html)
- [shape-outside-linear-gradient-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-005.html)
- [shape-outside-linear-gradient-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-006.html)
- [shape-outside-linear-gradient-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-007.html)
- [shape-outside-linear-gradient-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-008.html)
- [shape-outside-linear-gradient-009.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-009.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-009.html)
- [shape-outside-linear-gradient-010.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-010.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-010.html)
- [shape-outside-linear-gradient-011.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-011.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-011.html)
- [shape-outside-linear-gradient-012.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-012.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-012.html)
- [shape-outside-linear-gradient-013.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-013.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-013.html)
- [shape-outside-linear-gradient-014.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-014.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-014.html)
- [shape-outside-linear-gradient-015.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-015.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-015.html)
- [shape-outside-linear-gradient-016.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-016.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-016.html)
- [shape-outside-radial-gradient-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-001.html)
- [shape-outside-radial-gradient-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-002.html)
- [shape-outside-radial-gradient-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-003.html)
- [shape-outside-radial-gradient-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-004.html)
- [shape-image-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-000.html)
- [shape-image-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-001.html)
- [shape-image-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-002.html)
- [shape-image-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-003.html)
- [shape-image-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-004.html)
- [shape-image-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-005.html)
- [shape-image-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-006.html)
- [shape-image-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-007.html)
- [shape-image-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-008.html)
- [shape-image-009.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-009.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-009.html)
- [shape-image-010.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-010.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-010.html)
- [shape-image-011.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-011.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-011.html)
- [shape-image-012.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-012.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-012.html)
- [shape-image-013.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-013.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-013.html)
- [shape-image-014.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-014.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-014.html)
- [shape-image-015.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-015.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-015.html)
- [shape-image-016.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-016.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-016.html)
- [shape-image-017.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-017.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-017.html)
- [shape-image-018.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-018.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-018.html)
- [shape-image-019.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-019.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-019.html)
- [shape-image-020.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-020.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-020.html)
- [shape-image-021.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-021.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-021.html)
- [shape-image-022.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-022.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-022.html)
- [shape-image-023.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-023.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-023.html)
- [shape-image-024.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-024.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-024.html)
- [shape-image-025.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-025.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-025.html)
- [shape-image-026.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-026.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-026.html)
- [shape-image-027.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-027.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-027.html)
- [shape-image-028.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-028.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-028.html)
- [shape-image-029.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-029.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-029.html)

## <a id="shapes-from-box-values"></a>5.  Shapes from Box Values

<a id="ref-for-typedef-shape-box"></a>

<a id="ref-for-typedef-visual-box"></a>

<a id="ref-for-valdef-shape-box-margin-box①"></a>

Shapes can be defined by reference to edges in the [CSS Box Model](https://www.w3.org/TR/css-box-3/#box-model). These edges include [border-radius curvature](https://www.w3.org/TR/css3-background/#corner-shaping) [\[CSS3BG\]](#biblio-css3bg) from the used border-radius values. The [\<shape-box\>](#typedef-shape-box) value extends the [\<visual-box\>](https://www.w3.org/TR/css-box-4/#typedef-visual-box) value to include [margin-box](#valdef-shape-box-margin-box). Its syntax is:

<a id="typedef-shape-box"></a>

<a id="ref-for-typedef-shape-box①"></a>

<a id="ref-for-typedef-visual-box①"></a>

<a id="ref-for-comb-one③⓪"></a>

```text
<shape-box> = <visual-box> | margin-box
```
The definitions of the values are:

The <a id="valdef-shape-box-margin-box"></a>margin-box value defines the shape enclosed by the outside margin edge. The corner radii of this shape are determined by the corresponding border-radius and margin values. If the ratio of `border-radius/margin` is 1 or more, or margin is negative or zero, then the margin box corner radius is `max(border-radius + margin, 0)`. If the ratio of `border-radius/margin` is less than 1, and margin is positive, then the margin box corner radius is `border-radius + margin * (1 + (ratio-1)^3)`.

The <a id="valdef-shape-box-border-box"></a>border-box value defines the shape enclosed by the outside border edge. This shape follows all of the normal border radius shaping rules for the outside of the border.

The <a id="valdef-shape-box-padding-box"></a>padding-box value defines the shape enclosed by the outside padding edge. This shape follows all of the normal border radius shaping rules for the inside of the border.

The <a id="valdef-shape-box-content-box"></a>content-box value defines the shape enclosed by the outside content edge. Each corner radius of this box is the larger of 0 or `border-radius - border-width - padding`.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3c169cbd"></a>
>
> Given the 100px square below with 10px padding, border and margins, the box values define these shapes:
>
> - <a id="ref-for-valdef-shape-box-margin-box②"></a>
>
>   [margin-box](#valdef-shape-box-margin-box): the shape containing all of the yellow pixels
>
> - <a id="ref-for-valdef-shape-box-border-box"></a>
>
>   [border-box](#valdef-shape-box-border-box): the shape containing all of the black pixels
>
> - <a id="ref-for-valdef-shape-box-padding-box"></a>
>
>   [padding-box](#valdef-shape-box-padding-box): the shape containing all of the mauve pixels
>
> - <a id="ref-for-valdef-shape-box-content-box"></a>
>
>   [content-box](#valdef-shape-box-content-box): the shape containing all of the blue pixels
>
> ![Colored boxes representing simple box edges](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/box-edges-simple.png)
>
> Simple CSS Box Model Edges
>
> And the same definitions apply to a more complex example with the same 100px square, but with these border, padding and margin properties:
>
> ```text
> 
>   border-radius: 20px 20px 20px 40px;
>   border-width: 30px 10px 20px 10px;
>   padding: 10px 20px 10px 10px;
>   margin: 20px 10px 10px 10px;
> ```
>
> ![Colored boxes representing complex box edges](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/box-edges-complex.png)
>
> Complex CSS Box Model Edges

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ae456cb1"></a>
>
> The difference between normal float wrapping and wrapping around the shape defined by the margin-box value is that the margin-box shape includes corner shaping. Take the 100px square with 10px padding, border and margins, but with a border-radius of 60px. If you make a left float out of it, content normally wraps in this manner:
>
> ![Text wrapping around float with no shape](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/normal-wrap.png)
>
> Normal float wrapping
>
> If you add a margin-box shape to the float, then content wraps around the rounded margin-box corners.
>
> ```text
> 
>   shape-outside: margin-box;
> ```
>
> ![Text wrapping around float with margin-box shape](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/margin-box-wrap.png)
>
> Float wrapping with margin-box

Tests

- [shape-outside-border-box-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-001.html)
- [shape-outside-border-box-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-002.html)
- [shape-outside-border-box-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-003.html)
- [shape-outside-border-box-border-radius-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-001.html)
- [shape-outside-border-box-border-radius-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-002.html)
- [shape-outside-border-box-border-radius-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-003.html)
- [shape-outside-border-box-border-radius-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-004.html)
- [shape-outside-border-box-border-radius-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-005.html)
- [shape-outside-border-box-border-radius-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-006.html)
- [shape-outside-border-box-border-radius-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-007.html)
- [shape-outside-border-box-border-radius-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-008.html)
- [shape-outside-border-box-border-radius-009.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-009.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-009.html)
- [shape-outside-border-box-border-radius-010.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-010.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-010.html)
- [shape-outside-border-box-border-radius-011.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-011.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-011.html)
- [shape-outside-border-box-border-radius-012.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-012.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-012.html)
- [shape-outside-box-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-002.html)
- [shape-outside-box-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-003.html)
- [shape-outside-box-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-004.html)
- [shape-outside-box-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-006.html)
- [shape-outside-box-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-007.html)
- [shape-outside-box-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-008.html)
- [shape-outside-box-009.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-009.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-009.html)
- [shape-outside-content-box-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-001.html)
- [shape-outside-content-box-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-002.html)
- [shape-outside-content-box-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-003.html)
- [shape-outside-content-box-border-radius-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-border-radius-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-border-radius-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-border-radius-001.html)
- [shape-outside-content-box-border-radius-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-border-radius-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-border-radius-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-border-radius-002.html)
- [shape-outside-margin-box-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-001.html)
- [shape-outside-margin-box-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-002.html)
- [shape-outside-margin-box-border-radius-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-001.html)
- [shape-outside-margin-box-border-radius-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-002.html)
- [shape-outside-margin-box-border-radius-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-003.html)
- [shape-outside-margin-box-border-radius-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-004.html)
- [shape-outside-margin-box-border-radius-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-005.html)
- [shape-outside-margin-box-border-radius-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-006.html)
- [shape-outside-margin-box-border-radius-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-007.html)
- [shape-outside-margin-box-border-radius-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-008.html)
- [shape-outside-padding-box-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-001.html)
- [shape-outside-padding-box-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-002.html)
- [shape-outside-padding-box-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-003.html)
- [shape-outside-padding-box-border-radius-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-border-radius-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-border-radius-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-border-radius-001.html)
- [shape-outside-padding-box-border-radius-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-border-radius-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-border-radius-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-border-radius-002.html)
- [shape-outside-box-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-box-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-box-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-box-000.html)
- [shape-outside-shape-box-pair-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-shape-box-pair-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-shape-box-pair-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-shape-box-pair-000.html)

## <a id="declaring-shapes"></a>6.  Declaring Shapes

<a id="ref-for-propdef-shape-outside①⓪"></a>

<a id="ref-for-propdef-shape-margin②"></a>

<a id="ref-for-float-area②⓪"></a>

Shapes are declared with the [shape-outside](#propdef-shape-outside) property, with possible modifications from the [shape-margin](#propdef-shape-margin) property. The shape defined by the <a id="ref-for-propdef-shape-outside①①"></a>shape-outside and <a id="ref-for-propdef-shape-margin③"></a>shape-margin properties changes the geometry of a float element’s [float area](#float-area).

<a id="ref-for-propdef-shape-outside①②"></a>

### <a id="shape-outside-property"></a>6.1.  Float Area Shape: the [shape-outside](#propdef-shape-outside) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-shape-outside"></a>shape-outside                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-image①"></a><a id="ref-for-typedef-shape-box②"></a><a id="ref-for-comb-any"></a><a id="ref-for-typedef-basic-shape④"></a><a id="ref-for-comb-one③①"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ [\<basic-shape\>](#typedef-basic-shape) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<shape-box\>](#typedef-shape-box) \] <a id="ref-for-comb-one③②"></a>\| [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-initial-letter"></a>floats and [initial letter boxes](https://www.w3.org/TR/css-inline-3/#initial-letter)                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-image②"></a><a id="ref-for-typedef-shape-box③"></a><a id="ref-for-typedef-basic-shape⑤"></a>as [defined](#basic-shape-computed-values) for [\<basic-shape\>](#typedef-basic-shape) (with [\<shape-box\>](#typedef-shape-box) following, if supplied); else the computed [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image); else the keyword as specified                                                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <a id="ref-for-typedef-basic-shape⑥"></a>as [defined](#basic-shape-interpolation) for [\<basic-shape\>](#typedef-basic-shape), otherwise discrete                                                                                                                                                                                                                                                        |

The values of this property have the following meanings:

<a id="valdef-shape-outside-none"></a>none

<a id="ref-for-float-area②①"></a>

The [float area](#float-area) is unaffected.

<a id="ref-for-typedef-shape-box④"></a>

[\<shape-box\>](#typedef-shape-box)

<a id="ref-for-propdef-background-clip"></a>

<a id="ref-for-valdef-shape-box-content-box①"></a>

<a id="ref-for-valdef-shape-box-padding-box①"></a>

<a id="ref-for-valdef-shape-box-border-box①"></a>

<a id="ref-for-valdef-shape-box-margin-box③"></a>

If one of these values is specified by itself the shape is computed based on one of [margin-box](#valdef-shape-box-margin-box), [border-box](#valdef-shape-box-border-box), [padding-box](#valdef-shape-box-padding-box) or [content-box](#valdef-shape-box-content-box) which use their respective boxes including curvature from border-radius, similar to [background-clip](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-clip) [\[CSS3BG\]](#biblio-css3bg).

<a id="ref-for-typedef-basic-shape⑦"></a>

<a id="valdef-shape-outside-basic-shape"></a>[\<basic-shape\>](#typedef-basic-shape)

<a id="ref-for-valdef-shape-box-margin-box④"></a>

<a id="ref-for-basic-shape-reference-box②②"></a>

<a id="ref-for-typedef-shape-box⑤"></a>

<a id="ref-for-typedef-basic-shape⑧"></a>

The shape is computed based on one of the [\<basic-shape\>](#typedef-basic-shape) functions. If a [\<shape-box\>](#typedef-shape-box) is also supplied, this defines the [reference box](#basic-shape-reference-box) for the <a id="ref-for-typedef-basic-shape⑨"></a>\<basic-shape\> function. If <a id="ref-for-typedef-shape-box⑥"></a>\<shape-box\> is not supplied, then the <a id="ref-for-basic-shape-reference-box②③"></a>reference box defaults to [margin-box](#valdef-shape-box-margin-box).

<a id="ref-for-typedef-image③"></a>

<a id="valdef-shape-outside-image"></a>[\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image)

<a id="ref-for-propdef-shape-image-threshold③"></a>

<a id="ref-for-typedef-image④"></a>

The shape is extracted and computed based on the alpha channel of the specified [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) as defined by [shape-image-threshold](#propdef-shape-image-threshold).

<a id="ref-for-propdef-shape-outside①③"></a>

<a id="ref-for-valdef-shape-outside-none"></a>

User agents must use the [CORS protocol](https://fetch.spec.whatwg.org/#http-cors-protocol) defined by the [\[FETCH\]](#biblio-fetch) specification for all URLs in a [shape-outside](#propdef-shape-outside) value. When fetching, user agents must use "Anonymous" mode, set the referrer source to the stylesheet’s URL and set the origin to the URL of the containing document. If this results in network errors such that there is no valid fallback image, the effect is as if the value [none](#valdef-shape-outside-none) had been specified.

Tests

- [shape-outside-composition.html](https://wpt.fyi/results/css/css-shapes/animation/shape-outside-composition.html) [(live test)](http://wpt.live/css/css-shapes/animation/shape-outside-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/animation/shape-outside-composition.html)
- [shape-outside-interpolation.html](https://wpt.fyi/results/css/css-shapes/animation/shape-outside-interpolation.html) [(live test)](http://wpt.live/css/css-shapes/animation/shape-outside-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/animation/shape-outside-interpolation.html)
- [inheritance.html](https://wpt.fyi/results/css/css-shapes/inheritance.html) [(live test)](http://wpt.live/css/css-shapes/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/inheritance.html)
- [shape-outside-computed.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-outside-computed.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-outside-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-outside-computed.html)
- [shape-outside-invalid-position.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-outside-invalid-position.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-outside-invalid-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-outside-invalid-position.html)
- [shape-outside-valid-position.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-outside-valid-position.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-outside-valid-position.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-outside-valid-position.html)
- [shape-outside-infinite-crash.html](https://wpt.fyi/results/css/css-shapes/shape-outside-infinite-crash.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-infinite-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-infinite-crash.html)
- [shape-outside-invalid-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-001.html)
- [shape-outside-invalid-circle-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-circle-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-circle-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-circle-000.html)
- [shape-outside-invalid-circle-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-circle-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-circle-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-circle-001.html)
- [shape-outside-invalid-circle-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-circle-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-circle-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-circle-002.html)
- [shape-outside-invalid-circle-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-circle-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-circle-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-circle-003.html)
- [shape-outside-invalid-ellipse-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-ellipse-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-ellipse-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-ellipse-001.html)
- [shape-outside-invalid-ellipse-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-ellipse-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-ellipse-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-ellipse-002.html)
- [shape-outside-invalid-ellipse-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-ellipse-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-ellipse-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-ellipse-003.html)
- [shape-outside-invalid-ellipse-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-ellipse-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-ellipse-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-ellipse-004.html)
- [shape-outside-invalid-ellipse-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-ellipse-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-ellipse-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-ellipse-005.html)
- [shape-outside-invalid-ellipse-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-ellipse-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-ellipse-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-ellipse-006.html)
- [shape-outside-invalid-inset-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-inset-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-inset-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-inset-001.html)
- [shape-outside-invalid-inset-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-inset-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-inset-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-inset-002.html)
- [shape-outside-invalid-inset-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-inset-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-inset-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-inset-003.html)
- [shape-outside-invalid-inset-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside-invalid-inset-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside-invalid-inset-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside-invalid-inset-004.html)
- [float-retry-push-circle.html](https://wpt.fyi/results/css/css-shapes/shape-outside/assorted/float-retry-push-circle.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/assorted/float-retry-push-circle.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/assorted/float-retry-push-circle.html)
- [float-retry-push-image.html](https://wpt.fyi/results/css/css-shapes/shape-outside/assorted/float-retry-push-image.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/assorted/float-retry-push-image.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/assorted/float-retry-push-image.html)
- [float-retry-push-inset.html](https://wpt.fyi/results/css/css-shapes/shape-outside/assorted/float-retry-push-inset.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/assorted/float-retry-push-inset.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/assorted/float-retry-push-inset.html)
- [float-retry-push-polygon.html](https://wpt.fyi/results/css/css-shapes/shape-outside/assorted/float-retry-push-polygon.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/assorted/float-retry-push-polygon.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/assorted/float-retry-push-polygon.html)
- [float-should-push.html](https://wpt.fyi/results/css/css-shapes/shape-outside/assorted/float-should-push.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/assorted/float-should-push.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/assorted/float-should-push.html)
- [shape-outside-border-box-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-001.html)
- [shape-outside-border-box-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-002.html)
- [shape-outside-border-box-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-003.html)
- [shape-outside-border-box-border-radius-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-001.html)
- [shape-outside-border-box-border-radius-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-002.html)
- [shape-outside-border-box-border-radius-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-003.html)
- [shape-outside-border-box-border-radius-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-004.html)
- [shape-outside-border-box-border-radius-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-005.html)
- [shape-outside-border-box-border-radius-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-006.html)
- [shape-outside-border-box-border-radius-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-007.html)
- [shape-outside-border-box-border-radius-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-008.html)
- [shape-outside-border-box-border-radius-009.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-009.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-009.html)
- [shape-outside-border-box-border-radius-010.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-010.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-010.html)
- [shape-outside-border-box-border-radius-011.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-011.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-011.html)
- [shape-outside-border-box-border-radius-012.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-012.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-border-box-border-radius-012.html)
- [shape-outside-box-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-002.html)
- [shape-outside-box-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-003.html)
- [shape-outside-box-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-004.html)
- [shape-outside-box-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-006.html)
- [shape-outside-box-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-007.html)
- [shape-outside-box-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-008.html)
- [shape-outside-box-009.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-box-009.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-box-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-box-009.html)
- [shape-outside-content-box-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-001.html)
- [shape-outside-content-box-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-002.html)
- [shape-outside-content-box-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-003.html)
- [shape-outside-content-box-border-radius-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-border-radius-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-border-radius-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-border-radius-001.html)
- [shape-outside-content-box-border-radius-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-border-radius-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-border-radius-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-content-box-border-radius-002.html)
- [shape-outside-margin-box-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-001.html)
- [shape-outside-margin-box-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-002.html)
- [shape-outside-margin-box-border-radius-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-001.html)
- [shape-outside-margin-box-border-radius-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-002.html)
- [shape-outside-margin-box-border-radius-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-003.html)
- [shape-outside-margin-box-border-radius-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-004.html)
- [shape-outside-margin-box-border-radius-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-005.html)
- [shape-outside-margin-box-border-radius-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-006.html)
- [shape-outside-margin-box-border-radius-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-007.html)
- [shape-outside-margin-box-border-radius-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-008.html)
- [shape-outside-padding-box-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-001.html)
- [shape-outside-padding-box-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-002.html)
- [shape-outside-padding-box-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-003.html)
- [shape-outside-padding-box-border-radius-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-border-radius-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-border-radius-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-border-radius-001.html)
- [shape-outside-padding-box-border-radius-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-border-radius-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-border-radius-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-padding-box-border-radius-002.html)
- [shape-outside-linear-gradient-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-001.html)
- [shape-outside-linear-gradient-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-002.html)
- [shape-outside-linear-gradient-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-003.html)
- [shape-outside-linear-gradient-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-004.html)
- [shape-outside-linear-gradient-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-005.html)
- [shape-outside-linear-gradient-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-006.html)
- [shape-outside-linear-gradient-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-007.html)
- [shape-outside-linear-gradient-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-008.html)
- [shape-outside-linear-gradient-009.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-009.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-009.html)
- [shape-outside-linear-gradient-010.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-010.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-010.html)
- [shape-outside-linear-gradient-011.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-011.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-011.html)
- [shape-outside-linear-gradient-012.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-012.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-012.html)
- [shape-outside-linear-gradient-013.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-013.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-013.html)
- [shape-outside-linear-gradient-014.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-014.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-014.html)
- [shape-outside-linear-gradient-015.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-015.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-015.html)
- [shape-outside-linear-gradient-016.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-016.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-linear-gradient-016.html)
- [shape-outside-radial-gradient-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-001.html)
- [shape-outside-radial-gradient-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-002.html)
- [shape-outside-radial-gradient-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-003.html)
- [shape-outside-radial-gradient-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/gradients/shape-outside-radial-gradient-004.html)
- [shape-image-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-000.html)
- [shape-image-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-001.html)
- [shape-image-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-002.html)
- [shape-image-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-003.html)
- [shape-image-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-004.html)
- [shape-image-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-005.html)
- [shape-image-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-006.html)
- [shape-image-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-007.html)
- [shape-image-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-008.html)
- [shape-image-009.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-009.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-009.html)
- [shape-image-010.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-010.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-010.html)
- [shape-image-011.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-011.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-011.html)
- [shape-image-012.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-012.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-012.html)
- [shape-image-013.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-013.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-013.html)
- [shape-image-014.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-014.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-014.html)
- [shape-image-015.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-015.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-015.html)
- [shape-image-016.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-016.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-016.html)
- [shape-image-017.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-017.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-017.html)
- [shape-image-018.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-018.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-018.html)
- [shape-image-019.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-019.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-019.html)
- [shape-image-020.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-020.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-020.html)
- [shape-image-021.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-021.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-021.html)
- [shape-image-022.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-022.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-022.html)
- [shape-image-023.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-023.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-023.html)
- [shape-image-024.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-024.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-024.html)
- [shape-image-025.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-025.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-025.html)
- [shape-image-026.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-026.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-026.html)
- [shape-image-027.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-027.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-027.html)
- [shape-image-028.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-028.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-028.html)
- [shape-image-029.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-image/shape-image-029.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-image/shape-image-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-image/shape-image-029.html)
- [shape-outside-circle-013.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-013.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-013.html)
- [shape-outside-circle-014.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-014.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-014.html)
- [shape-outside-circle-015.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-015.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-015.html)
- [shape-outside-circle-016.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-016.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-016.html)
- [shape-outside-circle-017.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-017.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-017.html)
- [shape-outside-circle-018.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-018.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-018.html)
- [shape-outside-circle-019.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-019.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-019.html)
- [shape-outside-circle-020.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-020.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-020.html)
- [shape-outside-circle-021.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-021.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-021.html)
- [shape-outside-circle-022.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-022.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-022.html)
- [shape-outside-circle-024.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-024.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-024.html)
- [shape-outside-circle-025.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-025.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-025.html)
- [shape-outside-circle-026.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-026.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-026.html)
- [shape-outside-circle-027.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-027.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-027.html)
- [shape-outside-circle-028.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-028.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-028.html)
- [shape-outside-circle-029.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-029.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-029.html)
- [shape-outside-circle-030.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-030.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-030.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-030.html)
- [shape-outside-circle-031.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-031.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-031.html)
- [shape-outside-circle-032.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-032.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-032.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-032.html)
- [shape-outside-circle-033.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-033.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-033.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-033.html)
- [shape-outside-circle-034.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-034.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-034.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-034.html)
- [shape-outside-circle-035.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-035.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-035.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-035.html)
- [shape-outside-circle-036.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-036.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-036.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-036.html)
- [shape-outside-circle-037.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-037.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-037.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-037.html)
- [shape-outside-circle-038.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-038.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-038.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-038.html)
- [shape-outside-circle-041.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-041.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-041.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-041.html)
- [shape-outside-circle-042.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-042.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-042.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-042.html)
- [shape-outside-circle-043.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-043.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-043.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-043.html)
- [shape-outside-circle-044.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-044.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-044.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-044.html)
- [shape-outside-circle-047.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-047.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-047.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-047.html)
- [shape-outside-circle-048.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-048.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-048.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-048.html)
- [shape-outside-circle-049.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-049.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-049.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-049.html)
- [shape-outside-circle-050.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-050.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-050.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-050.html)
- [shape-outside-circle-051.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-051.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-051.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-051.html)
- [shape-outside-circle-052.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-052.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-052.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-052.html)
- [shape-outside-circle-053.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-053.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-053.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-053.html)
- [shape-outside-circle-054.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-054.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-054.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-054.html)
- [shape-outside-circle-055.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-055.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-055.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-055.html)
- [shape-outside-circle-056.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-056.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-056.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-056.html)
- [shape-outside-circle-integer-overflow-crash.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-integer-overflow-crash.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-integer-overflow-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/circle/shape-outside-circle-integer-overflow-crash.html)
- [shape-outside-ellipse-013.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-013.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-013.html)
- [shape-outside-ellipse-014.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-014.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-014.html)
- [shape-outside-ellipse-015.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-015.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-015.html)
- [shape-outside-ellipse-016.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-016.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-016.html)
- [shape-outside-ellipse-017.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-017.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-017.html)
- [shape-outside-ellipse-018.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-018.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-018.html)
- [shape-outside-ellipse-019.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-019.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-019.html)
- [shape-outside-ellipse-020.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-020.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-020.html)
- [shape-outside-ellipse-021.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-021.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-021.html)
- [shape-outside-ellipse-022.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-022.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-022.html)
- [shape-outside-ellipse-023.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-023.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-023.html)
- [shape-outside-ellipse-024.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-024.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-024.html)
- [shape-outside-ellipse-025.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-025.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-025.html)
- [shape-outside-ellipse-030.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-030.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-030.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-030.html)
- [shape-outside-ellipse-031.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-031.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-031.html)
- [shape-outside-ellipse-032.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-032.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-032.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-032.html)
- [shape-outside-ellipse-033.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-033.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-033.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-033.html)
- [shape-outside-ellipse-034.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-034.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-034.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-034.html)
- [shape-outside-ellipse-035.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-035.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-035.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-035.html)
- [shape-outside-ellipse-036.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-036.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-036.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-036.html)
- [shape-outside-ellipse-037.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-037.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-037.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-037.html)
- [shape-outside-ellipse-038.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-038.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-038.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-038.html)
- [shape-outside-ellipse-039.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-039.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-039.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-039.html)
- [shape-outside-ellipse-040.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-040.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-040.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-040.html)
- [shape-outside-ellipse-041.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-041.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-041.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-041.html)
- [shape-outside-ellipse-042.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-042.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-042.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-042.html)
- [shape-outside-ellipse-043.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-043.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-043.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-043.html)
- [shape-outside-ellipse-044.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-044.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-044.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-044.html)
- [shape-outside-ellipse-045.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-045.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-045.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-045.html)
- [shape-outside-ellipse-046.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-046.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-046.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-046.html)
- [shape-outside-ellipse-047.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-047.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-047.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-047.html)
- [shape-outside-ellipse-048.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-048.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-048.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-048.html)
- [shape-outside-ellipse-049.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-049.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-049.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-049.html)
- [shape-outside-ellipse-050.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-050.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-050.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-050.html)
- [shape-outside-ellipse-051.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-051.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-051.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-051.html)
- [shape-outside-ellipse-052.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-052.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-052.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-052.html)
- [shape-outside-ellipse-integer-overflow-crash.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-integer-overflow-crash.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-integer-overflow-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/ellipse/shape-outside-ellipse-integer-overflow-crash.html)
- [shape-outside-inset-010.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-010.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-010.html)
- [shape-outside-inset-011.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-011.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-011.html)
- [shape-outside-inset-012.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-012.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-012.html)
- [shape-outside-inset-013.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-013.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-013.html)
- [shape-outside-inset-014.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-014.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-014.html)
- [shape-outside-inset-015.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-015.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-015.html)
- [shape-outside-inset-016.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-016.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-016.html)
- [shape-outside-inset-017.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-017.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-017.html)
- [shape-outside-inset-020.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-020.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-020.html)
- [shape-outside-inset-021.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-021.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-021.html)
- [shape-outside-inset-022.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-022.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-022.html)
- [shape-outside-inset-023.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-023.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-023.html)
- [shape-outside-inset-024.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-024.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-024.html)
- [shape-outside-inset-025.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-025.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-025.html)
- [shape-outside-inset-026.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-026.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-026.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-026.html)
- [shape-outside-inset-027.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-027.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-027.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-027.html)
- [shape-outside-inset-028.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-028.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-028.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-028.html)
- [shape-outside-inset-029.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-029.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-029.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-029.html)
- [shape-outside-inset-030.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-030.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-030.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-030.html)
- [shape-outside-inset-031.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-031.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-031.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-031.html)
- [shape-outside-inset-refcrash.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-refcrash.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-refcrash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/inset/shape-outside-inset-refcrash.html)
- [shape-outside-polygon-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-007.html)
- [shape-outside-polygon-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-008.html)
- [shape-outside-polygon-009.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-009.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-009.html)
- [shape-outside-polygon-010.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-010.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-010.html)
- [shape-outside-polygon-011.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-011.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-011.html)
- [shape-outside-polygon-012.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-012.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-012.html)
- [shape-outside-polygon-013.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-013.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-013.html)
- [shape-outside-polygon-014.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-014.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-014.html)
- [shape-outside-polygon-015.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-015.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-015.html)
- [shape-outside-polygon-016.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-016.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-016.html)
- [shape-outside-polygon-017.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-017.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-017.html)
- [shape-outside-polygon-018.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-018.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-018.html)
- [shape-outside-polygon-019.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-019.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-019.html)
- [shape-outside-polygon-020.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-020.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-020.html)
- [shape-outside-polygon-021.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-021.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-021.html)
- [shape-outside-polygon-022.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-022.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-022.html)
- [shape-outside-polygon-023.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-023.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-023.html)
- [shape-outside-polygon-024.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-024.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-024.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-024.html)
- [shape-outside-polygon-025.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-025.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-025.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-025.html)
- [shape-outside-polygon-032.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-032.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-032.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-032.html)
- [shape-outside-polygon-crash.html](https://wpt.fyi/results/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-crash.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/supported-shapes/polygon/shape-outside-polygon-crash.html)
- [shape-image-threshold-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-image-threshold-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-image-threshold-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-image-threshold-000.html)
- [shape-image-threshold-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-image-threshold-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-image-threshold-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-image-threshold-001.html)
- [shape-image-threshold-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-image-threshold-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-image-threshold-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-image-threshold-002.html)
- [shape-image-threshold-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-image-threshold-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-image-threshold-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-image-threshold-003.html)
- [shape-margin-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-margin-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-margin-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-margin-000.html)
- [shape-margin-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-margin-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-margin-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-margin-001.html)
- [shape-margin-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-margin-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-margin-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-margin-002.html)
- [shape-margin-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-margin-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-margin-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-margin-003.html)
- [shape-margin-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-margin-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-margin-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-margin-004.html)
- [shape-margin-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-margin-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-margin-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-margin-005.html)
- [shape-outside-box-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-box-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-box-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-box-000.html)
- [shape-outside-circle-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-circle-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-circle-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-circle-000.html)
- [shape-outside-circle-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-circle-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-circle-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-circle-001.html)
- [shape-outside-circle-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-circle-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-circle-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-circle-002.html)
- [shape-outside-circle-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-circle-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-circle-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-circle-003.html)
- [shape-outside-circle-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-circle-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-circle-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-circle-004.html)
- [shape-outside-circle-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-circle-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-circle-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-circle-005.html)
- [shape-outside-circle-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-circle-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-circle-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-circle-006.html)
- [shape-outside-circle-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-circle-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-circle-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-circle-007.html)
- [shape-outside-circle-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-circle-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-circle-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-circle-008.html)
- [shape-outside-circle-009.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-circle-009.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-circle-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-circle-009.html)
- [shape-outside-circle-010.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-circle-010.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-circle-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-circle-010.html)
- [shape-outside-circle-011.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-circle-011.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-circle-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-circle-011.html)
- [shape-outside-computed-shape-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-computed-shape-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-computed-shape-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-computed-shape-000.html)
- [shape-outside-computed-shape-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-computed-shape-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-computed-shape-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-computed-shape-001.html)
- [shape-outside-ellipse-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-ellipse-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-ellipse-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-ellipse-000.html)
- [shape-outside-ellipse-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-ellipse-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-ellipse-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-ellipse-001.html)
- [shape-outside-ellipse-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-ellipse-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-ellipse-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-ellipse-002.html)
- [shape-outside-ellipse-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-ellipse-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-ellipse-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-ellipse-003.html)
- [shape-outside-ellipse-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-ellipse-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-ellipse-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-ellipse-004.html)
- [shape-outside-ellipse-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-ellipse-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-ellipse-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-ellipse-005.html)
- [shape-outside-ellipse-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-ellipse-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-ellipse-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-ellipse-006.html)
- [shape-outside-ellipse-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-ellipse-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-ellipse-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-ellipse-007.html)
- [shape-outside-ellipse-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-ellipse-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-ellipse-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-ellipse-008.html)
- [shape-outside-ellipse-009.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-ellipse-009.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-ellipse-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-ellipse-009.html)
- [shape-outside-ellipse-010.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-ellipse-010.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-ellipse-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-ellipse-010.html)
- [shape-outside-ellipse-011.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-ellipse-011.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-ellipse-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-ellipse-011.html)
- [shape-outside-inset-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-inset-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-inset-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-inset-000.html)
- [shape-outside-inset-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-inset-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-inset-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-inset-001.html)
- [shape-outside-inset-0010.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-inset-0010.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-inset-0010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-inset-0010.html)
- [shape-outside-inset-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-inset-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-inset-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-inset-002.html)
- [shape-outside-inset-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-inset-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-inset-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-inset-003.html)
- [shape-outside-inset-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-inset-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-inset-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-inset-004.html)
- [shape-outside-inset-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-inset-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-inset-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-inset-005.html)
- [shape-outside-inset-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-inset-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-inset-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-inset-006.html)
- [shape-outside-inset-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-inset-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-inset-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-inset-007.html)
- [shape-outside-inset-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-inset-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-inset-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-inset-008.html)
- [shape-outside-inset-009.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-inset-009.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-inset-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-inset-009.html)
- [shape-outside-polygon-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-polygon-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-polygon-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-polygon-000.html)
- [shape-outside-polygon-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-polygon-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-polygon-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-polygon-001.html)
- [shape-outside-polygon-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-polygon-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-polygon-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-polygon-002.html)
- [shape-outside-polygon-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-polygon-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-polygon-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-polygon-003.html)
- [shape-outside-polygon-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-polygon-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-polygon-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-polygon-004.html)
- [shape-outside-polygon-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-polygon-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-polygon-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-polygon-005.html)
- [shape-outside-polygon-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-polygon-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-polygon-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-polygon-006.html)
- [shape-outside-shape-arguments-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-shape-arguments-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-shape-arguments-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-shape-arguments-000.html)
- [shape-outside-shape-arguments-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-shape-arguments-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-shape-arguments-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-shape-arguments-001.html)
- [shape-outside-shape-box-pair-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-shape-box-pair-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-shape-box-pair-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-shape-box-pair-000.html)
- [shape-outside-shape-inherit-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-shape-inherit-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-shape-inherit-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-shape-inherit-000.html)
- [shape-outside-shape-initial-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-shape-initial-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-shape-initial-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-shape-initial-000.html)
- [shape-outside-shape-none-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-shape-none-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-shape-none-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-shape-none-000.html)
- [shape-outside-shape-notation-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-outside-shape-notation-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-outside-shape-notation-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-outside-shape-notation-000.html)
- [shape-outside-001.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-001.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-001.html)
- [shape-outside-002.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-002.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-002.html)
- [shape-outside-003.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-003.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-003.html)
- [shape-outside-004.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-004.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-004.html)
- [shape-outside-005.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-005.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-005.html)
- [shape-outside-006.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-006.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-006.html)
- [shape-outside-007.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-007.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-007.html)
- [shape-outside-008.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-008.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-008.html)
- [shape-outside-010.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-010.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-010.html)
- [shape-outside-011.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-011.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-011.html)
- [shape-outside-012.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-012.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-012.html)
- [shape-outside-013.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-013.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-013.html)
- [shape-outside-014.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-014.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-014.html)
- [shape-outside-015.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-015.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-015.html)
- [shape-outside-016.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-016.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-016.html)
- [shape-outside-017.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-017.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-017.html)
- [shape-outside-018.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-018.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-018.html)
- [shape-outside-019.html](https://wpt.fyi/results/css/css-shapes/spec-examples/shape-outside-019.html) [(live test)](http://wpt.live/css/css-shapes/spec-examples/shape-outside-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/spec-examples/shape-outside-019.html)

<a id="ref-for-propdef-shape-image-threshold④"></a>

### <a id="shape-image-threshold-property"></a>6.2.  Choosing Image Pixels: the [shape-image-threshold](#propdef-shape-image-threshold) property

<a id="ref-for-propdef-shape-image-threshold⑤"></a>

The [shape-image-threshold](#propdef-shape-image-threshold) defines the alpha channel threshold used to extract the shape using an image. A value of 0.5 means that the shape will enclose all the pixels that are more than 50% opaque.

| Field               | Definition                                                                                               |
|---------------------|----------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-shape-image-threshold"></a>shape-image-threshold                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-opacity-opacity-value"></a>[\<opacity-value\>](https://www.w3.org/TR/css-color-4/#typedef-opacity-opacity-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | floats                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified number, clamped to the range \[0,1\]                                                           |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                              |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                        |

The values of this property have the following meanings:

<a id="ref-for-number-value"></a>

<a id="valdef-shape-image-threshold-number"></a>[\<number\>](https://www.w3.org/TR/css-values-4/#number-value)

Sets the threshold used for extracting a shape from an image. The shape is defined by the pixels whose alpha value is greater than the threshold. A threshold value outside the range 0.0 (fully transparent) to 1.0 (fully opaque) will be clamped to this range.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A future level of CSS Shapes may define a switch to use the luminance data from an image instead of the alpha data. When this happens, shape-image-threshold will be extended to apply its threshold to either alpha or luminance, depending on the switch state.

Tests

- [shape-image-threshold-interpolation.html](https://wpt.fyi/results/css/css-shapes/animation/shape-image-threshold-interpolation.html) [(live test)](http://wpt.live/css/css-shapes/animation/shape-image-threshold-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/animation/shape-image-threshold-interpolation.html)
- [inheritance.html](https://wpt.fyi/results/css/css-shapes/inheritance.html) [(live test)](http://wpt.live/css/css-shapes/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/inheritance.html)
- [shape-image-threshold-computed.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-image-threshold-computed.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-image-threshold-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-image-threshold-computed.html)
- [shape-image-threshold-invalid.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-image-threshold-invalid.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-image-threshold-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-image-threshold-invalid.html)
- [shape-image-threshold-valid.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-image-threshold-valid.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-image-threshold-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-image-threshold-valid.html)
- [shape-image-threshold-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-image-threshold-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-image-threshold-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-image-threshold-000.html)
- [shape-image-threshold-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-image-threshold-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-image-threshold-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-image-threshold-001.html)
- [shape-image-threshold-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-image-threshold-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-image-threshold-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-image-threshold-002.html)
- [shape-image-threshold-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-image-threshold-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-image-threshold-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-image-threshold-003.html)

<a id="ref-for-propdef-shape-margin④"></a>

### <a id="shape-margin-property"></a>6.3.  Expanding a Shape: the [shape-margin](#propdef-shape-margin) property

<a id="ref-for-propdef-shape-margin⑤"></a>

<a id="ref-for-propdef-shape-outside①④"></a>

The [shape-margin](#propdef-shape-margin) property adds a margin to a [shape-outside](#propdef-shape-outside). This defines a new shape that is the smallest contour (in the shrink-wrap sense) that includes all the points that are the <a id="ref-for-propdef-shape-margin⑥"></a>shape-margin distance outward in the perpendicular direction from a point on the underlying shape. This includes any edge or line sections from the underlying shape. Note that at points where a perpendicular is not defined (e.g. sharp points or line ends) take all points on the circle centered at the point and with a radius of <a id="ref-for-propdef-shape-margin⑦"></a>shape-margin.

<a id="ref-for-propdef-shape-outside①⑤"></a>

<a id="ref-for-float-area②②"></a>

<a id="ref-for-wrap①③"></a>

The new shape produced by applying [shape-outside](#propdef-shape-outside) is what determines the [float area](#float-area), and must be constructed before making any [wrap](#wrap) decisions.

This property takes only non-negative values.

| Field               | Definition                                                                                                                    |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-shape-margin"></a>shape-margin                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage③③"></a>[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-initial-letter①"></a>floats and [initial letter boxes](https://www.w3.org/TR/css-inline-3/#initial-letter)                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-inline-size"></a>refer to the [inline size](https://www.w3.org/TR/css-writing-modes-4/#inline-size) of the containing block |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage③④"></a>computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                             |

<a id="ref-for-typedef-length-percentage③⑤"></a>

<a id="valdef-shape-margin-length-percentage-0"></a>[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)

Sets the margin of the shape to the specified value.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Adding a shape-margin does NOT allow a float area to extend outside a float’s margin box. Extra margin may need to be applied along with shape-margin to avoid clipping.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-dde61751"></a>
>
> <a id="ref-for-propdef-shape-margin⑧"></a>
>
> <a id="ref-for-propdef-shape-outside①⑥"></a>
>
> A [shape-margin](#propdef-shape-margin) creating an offset from a polygonal [shape-outside](#propdef-shape-outside). The lighter blue area shows the shape in a 100x100px float, and the darker blue area shows the 10px offset.
>
> ```text
> 
>   .float {
>       width: 100px;
>       height: 100px;
>     shape-outside: polygon(10px 10px, 90px 50px, 40px 50px, 90px 90px, 10px 90px);
>     shape-margin: 10px;
>   }
> ```
>
> ![Example of a shape-margin offset](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/nepal-flag-shape.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-dc83070e"></a>
>
> <a id="ref-for-wrap①④"></a>
>
> If shape-margin is added to the CSS logo from example 6, the line boxes [wrapping](#wrap) around the shape are shortened further. In case the image’s alpha channel runs up to the right edge of the image, some extra margin-right should be applied to ensure the shape is not clipped by the margin box.
>
> ```text
> 
>   #CSSlogo {
>     shape-margin: 35px;
>     margin-right: 35px;
>   }
> ```
>
> ![A float shape around an image using its alpha-channel with a 35 pixels shape-margin](https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/images/shape-outside-image-with-margin.png)
>
> <a id="ref-for-propdef-shape-margin⑨"></a>
>
> A float shape around an image using its alpha-channel with a 35-pixel [shape-margin](#propdef-shape-margin)

Tests

- [shape-margin-composition.html](https://wpt.fyi/results/css/css-shapes/animation/shape-margin-composition.html) [(live test)](http://wpt.live/css/css-shapes/animation/shape-margin-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/animation/shape-margin-composition.html)
- [shape-margin-interpolation.html](https://wpt.fyi/results/css/css-shapes/animation/shape-margin-interpolation.html) [(live test)](http://wpt.live/css/css-shapes/animation/shape-margin-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/animation/shape-margin-interpolation.html)
- [inheritance.html](https://wpt.fyi/results/css/css-shapes/inheritance.html) [(live test)](http://wpt.live/css/css-shapes/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/inheritance.html)
- [shape-margin-computed.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-margin-computed.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-margin-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-margin-computed.html)
- [shape-margin-invalid.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-margin-invalid.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-margin-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-margin-invalid.html)
- [shape-margin-valid.html](https://wpt.fyi/results/css/css-shapes/parsing/shape-margin-valid.html) [(live test)](http://wpt.live/css/css-shapes/parsing/shape-margin-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/parsing/shape-margin-valid.html)
- [shape-outside-margin-box-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-001.html)
- [shape-outside-margin-box-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-002.html)
- [shape-outside-margin-box-border-radius-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-001.html)
- [shape-outside-margin-box-border-radius-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-002.html)
- [shape-outside-margin-box-border-radius-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-003.html)
- [shape-outside-margin-box-border-radius-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-004.html)
- [shape-outside-margin-box-border-radius-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-005.html)
- [shape-outside-margin-box-border-radius-006.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-006.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-006.html)
- [shape-outside-margin-box-border-radius-007.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-007.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-007.html)
- [shape-outside-margin-box-border-radius-008.html](https://wpt.fyi/results/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-008.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/shape-box/shape-outside-margin-box-border-radius-008.html)
- [shape-margin-000.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-margin-000.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-margin-000.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-margin-000.html)
- [shape-margin-001.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-margin-001.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-margin-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-margin-001.html)
- [shape-margin-002.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-margin-002.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-margin-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-margin-002.html)
- [shape-margin-003.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-margin-003.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-margin-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-margin-003.html)
- [shape-margin-004.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-margin-004.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-margin-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-margin-004.html)
- [shape-margin-005.html](https://wpt.fyi/results/css/css-shapes/shape-outside/values/shape-margin-005.html) [(live test)](http://wpt.live/css/css-shapes/shape-outside/values/shape-margin-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-shapes/shape-outside/values/shape-margin-005.html)

## <a id="privacy"></a>7. Privacy Considerations

No privacy concerns have been raised against this specification.

## <a id="security"></a>8. Security Considerations

<a id="ref-for-typedef-image⑤"></a>

<a id="ref-for-propdef-shape-outside①⑦"></a>

Since the [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) value of [shape-outside](#propdef-shape-outside) can expose some image data in a new way, use is limited to images with CORS approval.

## <a id="acknowledgments"></a> Acknowledgments

This specification is made possible by input from Tab Atkins Jr., Amelia Bellamy-Royds, Oriol Brufau, Andrei Bucur, Alexandru Chiculita, Boris Chiou, Emilio Cobos Álvarez, Elika Etemad, Arron Eicholz, Sylvain Galineau, Daniel Glazman, Arno Gourdol, Zoltan Horvath, Chris Jones, Bem Jones-Bey, Ian Kilpatrick, Guillaume Lebas, Ting-Yu Lin, Eric Meyer, Marcus Mielke, Alex Mogilevsky, Hans Muller, Mihnea Ovidenie, Virgil Palanciuc, Noam Rosenthal, Robert Sanderson, Dirk Schulze, Jen Simmons, Peter Sorotokin, Bear Travis, Lea Verou, Eugene Veselov, Brad Werth, Stephen Zilles and the CSS Working Group members.

## <a id="change-log"></a> Change Log

### <a id="20221115"></a> Since [15 November 2022](https://www.w3.org/TR/2022/CRD-css-shapes-1-20221115/)

- Added Web Platform Test annotations

- Changed fill-rule default for path() in SVG shapes

- Clarified shape-margin and float area interaction

- Added polygon rounding examples

- Included all basic shapes in property prose for [issue \#9728](https://github.com/w3c/csswg-drafts/issues/9728)

- Added start of polygon vertex rounding

- Used opacity-value (the value) not opacity (the property) as value of shape-image-threshold for [issue \#8311](https://github.com/w3c/csswg-drafts/issues/8311)

- <a id="ref-for-propdef-opacity"></a>

  Replaced \<alpha-value\> with \<[opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity)\>

- Clarified that shape-outside applies to initial letter boxes for [issue \#5160](https://github.com/w3c/csswg-drafts/issues/5160)

- Rebased ellipse()/circle() definitions on top of \<radial-size\> from Images, for [issue \#824](https://github.com/w3c/csswg-drafts/issues/824)

- Made it clear that the position defaulting can be overridden by other specs (like Motion).

- Targeted fix of shape serialization section, fixed examples, for [issue \#8695](https://github.com/w3c/csswg-drafts/issues/8695)

- Fixed xywh() computed value for [issue \#9053](https://github.com/w3c/csswg-drafts/issues/9053)

- Rewrote syntax with implicitly optional comma [PR \#9650](https://github.com/w3c/csswg-drafts/pull/9650)

- Cleanup \<\*-box\> definitions [PR \#9505](https://github.com/w3c/csswg-drafts/pull/9505)

- Added range notation in property definitions

- Added vertex rounding for polygon() for [issue \#9843](https://github.com/w3c/csswg-drafts/issues/9843)

- <a id="ref-for-typedef-basic-shape①⓪"></a>

  Updated shape-outside prose to include all [\<basic-shape\>](#typedef-basic-shape)s for [issue \#9728](https://github.com/w3c/csswg-drafts/issues/9728)

- Clarified the shape-margin contribution to float areas for [issue \#2949](https://github.com/w3c/csswg-drafts/issues/2949)

- Added fill-rule default handling for SVG shapes from [issue \#3468](https://github.com/w3c/csswg-drafts/issues/3468)

- Moved 'shape()' from level 2

### <a id="20140320"></a> Since [March 20th 2014](https://www.w3.org/TR/2014/CR-css-shapes-1-20140320/)

- Clarified shape-margin computed value

- Clarified serialization of default position values, for [issue \#402](https://github.com/w3c/csswg-drafts/issues/402)

- Clarified empty circles and ellipses for [issue \#850](https://github.com/w3c/csswg-drafts/issues/850)

- Dropped the "Media:" entry from propdef tables, as with all CSS modules

- Updated box model references from CSS 2 to CSS Box Model 3

- Updated Computed Value and Animation Type in propdef tables

- Clarified computed value of shape-outside property, for [issue \#4042](https://github.com/w3c/csswg-drafts/issues/4042)

- Clarified that shape-image-threshold can take a percentage value, like any alpha value, for [issue \#4102](https://github.com/w3c/csswg-drafts/issues/4102)

- Moved path() back from level 2

- Added handling of negative margins for margin-box, for [issue \#675](https://github.com/w3c/csswg-drafts/issues/675)

- Removed special-case serialization of position values, for [issue \#2301](https://github.com/w3c/csswg-drafts/issues/2301)

- Added clarifications to shape-margin examples

- Added margin=0 case for margin-box shape, for [issue \#675](https://github.com/w3c/csswg-drafts/issues/675)

- Changed rules about degenerate shapes to use shape edges, for [issue \#2375](https://github.com/w3c/csswg-drafts/issues/2375)

- <a id="ref-for-funcdef-basic-shape-path①④"></a>

  Clarified that invalid path strings make the [path()](#funcdef-basic-shape-path) invalid, for [issue \#392](https://github.com/w3c/fxtf-drafts/issues/392)

- Rewrote definition of inset rectangles, added auto value, added examples

- Clarified computed value of basic shape rect functions

- Assorted markup fixes, including use of range notation and exporting defined terms for use in other specifications

- Split Privacy and Security considerations into separate sections

### <a id="20140211"></a> Since [February 11th 2014](https://www.w3.org/TR/2014/WD-css-shapes-1-20140211/)

- Replaced divs with images in the first example
- Add 0px to last serialization example

### <a id="20131203"></a> Since [December 3rd 2013](https://www.w3.org/TR/2013/WD-css-shapes-1-20131203/)

- Updated computed value and serialization of basic shapes
- Added a margin-box example
- Change auto to none for shape-outside
- Defined shape-box instead of redefining box
- Clarified that shape from image may produce more than one path

### <a id="20130620"></a> Since [June 20th 2013](https://www.w3.org/TR/2013/WD-css-shapes-1-20130620/)

- Added shape from box value section
- Updated basic-shape interpolation
- Allow negative insets, disallow negative radii
- Changed relevant to reference
- Remove box-sizing dependency, add relevant box keywords
- Changed circle() and ellipse() to use radial gradient syntax
- Postponed rectangle() to level 2
- Clarified shape-from-image sizing and positioning
- Change inset-rectangle() to inset()
- Future-proof shape-image-threshold to possibly apply to luminance
- Added CORS fetching to shape-outside URLs
- Changed shape-outside value from \<uri\> to \<image\>
- Remove 'percentages based on auto-sizing resolve to 0'
- Change initial value of shape-image-threshold to 0.0
- Change float positioning to be unaffected by shape-outside
- Shapes on floats clipped to float’s margin box

### <a id="20120503"></a> Since [May 3rd 2012](https://www.w3.org/TR/2012/WD-css3-exclusions-20120503/)

- Postpone shapes from SVG elements to a future Shapes level
- Postpone shape-inside to a future Shapes level
- split exclusions from shapes into separate modules
- added inset-rectangle() to basic shapes
- Changed shape-inside overflow diagrams to show exclusion behavior
- Changed shape-inside to contribute to the wrapping context
- Defined exclusion edges relative to wrapping content’s writing mode
- Made use of start, end, before and after consistent
- Added interpolation for basic shapes
- Changed basic shapes to depend on box specified with box-sizing
- Added overflow behavior for shape-inside.
- Added wrap-flow:minimum.
- Clarified processing model.
- Changed wrap-margin and wrap-padding to shape-margin and shape-padding.
- Removed wrap shorthand.

### <a id="20111213"></a> Since [December 13th 2011](https://www.w3.org/TR/2011/WD-css3-exclusions-20111213/)

- Clarified processing model.
- Clarified interaction with floats.
- Clarified that an exclusion element establishes a new block formatting context.

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

Tests

Tests relating to the content of this specification may be documented in “Tests” blocks like this one. Any such block is non-normative.

------------------------------------------------------------------------

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

### <a id="w3c-cr-exit-criteria"></a> CR exit criteria

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

- [arc](#valdef-shape-arc), in § 3.1.1
- [\<arc-command\>](#typedef-shape-arc-command), in § 3.1.1
- [\<arc-size\>](#typedef-shape-arc-size), in § 3.1.1
- [\<arc-sweep\>](#typedef-shape-arc-sweep), in § 3.1.1
- \<basic-shape\>
  - [(type)](#typedef-basic-shape), in § 3
  - [value for shape-outside](#valdef-shape-outside-basic-shape), in § 6.1
- [\<basic-shape-rect\>](#typedef-basic-shape-rect), in § 3.1
- [border-box](#valdef-shape-box-border-box), in § 5
- [by](#valdef-shape-by), in § 3.1.1
- [ccw](#valdef-shape-ccw), in § 3.1.1
- [circle()](#funcdef-basic-shape-circle), in § 3.1
- [close](#valdef-shape-close), in § 3.1.1
- [\<command-end-point\>](#typedef-shape-command-end-point), in § 3.1.1
- [content-box](#valdef-shape-box-content-box), in § 5
- [\<control-point\>](#typedef-shape-control-point), in § 3.1.1
- [\<coordinate-pair\>](#typedef-shape-coordinate-pair), in § 3.1.1
- [curve](#valdef-shape-curve), in § 3.1.1
- [\<curve-command\>](#typedef-shape-curve-command), in § 3.1.1
- [cw](#valdef-shape-cw), in § 3.1.1
- [direction-agnostic size](#direction-agnostic-size), in § 1.3
- [ellipse()](#funcdef-basic-shape-ellipse), in § 3.1
- [Float area](#float-area), in § 1.3
- [\<horizontal-line-command\>](#typedef-shape-horizontal-line-command), in § 3.1.1
- [\<image\>](#valdef-shape-outside-image), in § 6.1
- [inset()](#funcdef-basic-shape-inset), in § 3.1
- [large](#valdef-shape-large), in § 3.1.1
- [\<length-percentage \[0,∞\]\>](#valdef-shape-margin-length-percentage-0), in § 6.3
- [line](#valdef-shape-line), in § 3.1.1
- [\<line-command\>](#typedef-shape-line-command), in § 3.1.1
- [margin-box](#valdef-shape-box-margin-box), in § 5
- [move](#valdef-shape-move), in § 3.1.1
- [\<move-command\>](#typedef-shape-move-command), in § 3.1.1
- [none](#valdef-shape-outside-none), in § 6.1
- [\<number\>](#valdef-shape-image-threshold-number), in § 6.2
- [padding-box](#valdef-shape-box-padding-box), in § 5
- [path()](#funcdef-basic-shape-path), in § 3.1
- [polygon()](#funcdef-basic-shape-polygon), in § 3.1
- [rect()](#funcdef-basic-shape-rect), in § 3.1
- [reference box](#basic-shape-reference-box), in § 3
- [\<relative-control-point\>](#typedef-shape-relative-control-point), in § 3.1.1
- [shape()](#funcdef-basic-shape-shape), in § 3.1
- [\<shape-box\>](#typedef-shape-box), in § 5
- [\<shape-command\>](#typedef-shape-command), in § 3.1.1
- [shape-image-threshold](#propdef-shape-image-threshold), in § 6.2
- [shape-margin](#propdef-shape-margin), in § 6.3
- [shape-outside](#propdef-shape-outside), in § 6.1
- [small](#valdef-shape-small), in § 3.1.1
- [smooth](#valdef-shape-smooth), in § 3.1.1
- [\<smooth-command\>](#typedef-shape-smooth-command), in § 3.1.1
- [to](#valdef-shape-to), in § 3.1.1
- [\<vertical-line-command\>](#typedef-shape-vertical-line-command), in § 3.1.1
- [wrap](#wrap), in § 1.3
- [wrapping](#wrap), in § 1.3
- [xywh()](#funcdef-basic-shape-xywh), in § 3.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BOX-4\] defines the following terms:
  - <a id="c87746d2"></a>\<visual-box\>
  - <a id="0778a939"></a>margin box
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="d5e08d9c"></a>specified value
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="fda53a37"></a>\<opacity-value\>
  - <a id="3b7558dc"></a>opacity
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="35bf32f2"></a>\<image\>
  - <a id="55774cdb"></a>\<radial-size\>
- \[CSS-IMAGES-4\] defines the following terms:
  - <a id="8a30abc0"></a>gradient box
- \[CSS-INLINE-3\] defines the following terms:
  - <a id="6fea0c77"></a>initial letter box
- \[CSS-MASKING\] defines the following terms:
  - <a id="e97d95b6"></a>clip
  - <a id="e2b06daa"></a>clip-path
  - <a id="c593fda8"></a>nonzero
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="e7c3b4f7"></a>invalid
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="6441b868"></a>rotate()
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="c297b070"></a>\#
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="8cd4f032"></a>,
  - <a id="d7e1d67b"></a>\<angle\>
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="128295ac"></a>\<percentage\>
  - <a id="1d798932"></a>\<string\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="eefce2af"></a>em
  - <a id="c2e11f7b"></a>interpolate
  - <a id="20730c34"></a>px
  - <a id="3bafef5e"></a>{A,B}
  - <a id="8cbc2b3b"></a>{A}
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-VALUES-5\] defines the following terms:
  - <a id="3df4be2e"></a>\<position\>
  - <a id="04acd1ec"></a>bottom
  - <a id="edebc460"></a>center
  - <a id="12323c43"></a>left
  - <a id="d5b50296"></a>right
  - <a id="40ead293"></a>top
  - <a id="2a5e99b0"></a>x-end
  - <a id="7d20759b"></a>x-start
  - <a id="fce2095c"></a>y-end
  - <a id="5e64ec02"></a>y-start
- \[CSS-VARIABLES-1\] defines the following terms:
  - <a id="3beec8c9"></a>var()
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="18bb1084"></a>inline size
- \[CSS2\] defines the following terms:
  - <a id="cfab9333"></a>margin
- \[CSS3BG\] defines the following terms:
  - <a id="a5c1f433"></a>background-clip
  - <a id="3a4a9318"></a>border-radius
- \[SVG2\] defines the following terms:
  - <a id="a086f470"></a>fill-rule
  - <a id="441d640b"></a>path
  - <a id="b7c9858b"></a>viewBox

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Chris Lilley; Tab Atkins Jr.; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 24 April 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 4](https://www.w3.org/TR/css-images-4/). 17 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Elika Etemad. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 18 December 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-masking"></a>\[CSS-MASKING\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 5 August 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-values-5"></a>\[CSS-VALUES-5\]  
Tab Atkins Jr.; Elika Etemad; Miriam Suzanne. [CSS Values and Units Module Level 5](https://www.w3.org/TR/css-values-5/). 11 November 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-5&#x2F;](https://www.w3.org/TR/css-values-5/)

<a id="biblio-css-variables-1"></a>\[CSS-VARIABLES-1\]  
Tab Atkins Jr.. [CSS Custom Properties for Cascading Variables Module Level 1](https://www.w3.org/TR/css-variables-1/). 16 June 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-variables-1&#x2F;](https://www.w3.org/TR/css-variables-1/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-css3bg"></a>\[CSS3BG\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css3box"></a>\[CSS3BOX\]  
Elika Etemad. [CSS Box Model Module Level 3](https://www.w3.org/TR/css-box-3/). 11 April 2024. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-3&#x2F;](https://www.w3.org/TR/css-box-3/)

<a id="biblio-fetch"></a>\[FETCH\]  
Anne van Kesteren. [Fetch Standard](https://fetch.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;fetch&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://fetch.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

### <a id="informative"></a>Informative References

<a id="biblio-css3-exclusions"></a>\[CSS3-EXCLUSIONS\]  
Rossen Atanassov; Vincent Hardy; Alan Stearns. [CSS Exclusions Module Level 1](https://www.w3.org/TR/css3-exclusions/). 15 January 2015. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css3-exclusions&#x2F;](https://www.w3.org/TR/css3-exclusions/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                         | Initial | Applies to                      | Inh. | %ages                                            | Anim­ation type                                     | Canonical order | Com­puted value                                                                                                                         |
|---------------------|---------------------------------------------------------------|---------|---------------------------------|------|--------------------------------------------------|----------------------------------------------------|-----------------|----------------------------------------------------------------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-shape-image-threshold⑥"></a></span><a href="#propdef-shape-image-threshold">shape-image-threshold</a>&#xA;      </strong> | \<opacity-value\>                                             | 0       | floats                          | no   | n/a                                              | by computed value                                  | per grammar     | specified number, clamped to the range \[0,1\]                                                                                         |
| <strong><span><a id="ref-for-propdef-shape-margin①⓪"></a></span><a href="#propdef-shape-margin">shape-margin</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>                                 | 0       | floats and initial letter boxes | no   | refer to the inline size of the containing block | by computed value                                  | per grammar     | computed \<length-percentage\> value                                                                                                   |
| <strong><span><a id="ref-for-propdef-shape-outside①⑧"></a></span><a href="#propdef-shape-outside">shape-outside</a>&#xA;      </strong> | none \| \[ \<basic-shape\> \|\| \<shape-box\> \] \| \<image\> | none    | floats and initial letter boxes | no   | n/a                                              | as defined for \<basic-shape\>, otherwise discrete | per grammar     | as defined for \<basic-shape\> (with \<shape-box\> following, if supplied); else the computed \<image\>; else the keyword as specified |

