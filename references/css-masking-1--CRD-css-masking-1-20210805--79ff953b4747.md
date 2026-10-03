Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Masking Module Level 1](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/).

Original copyright notice: Copyright © 2021 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Masking Module Level 1

Source snapshot: https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/

Snapshot SHA-256: 79ff953b47472ae5bc1eace14c0c470d7bd75d128e933806c97c50b640194671

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 24 source tables are presented as readable Markdown tables or explicit labeled layouts: 22 ordinary table conversions, 2 complex-table layouts. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Masking Module Level 1

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2021 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

CSS Masking provides two means for partially or fully hiding portions of visual elements: masking and clipping.

Masking describes how to use another graphical element or image as a luminance or alpha mask. Typically, rendering an element via CSS or SVG can conceptually be described as if the element, including its children, are drawn into a buffer and then that buffer is composited into the element’s parent. Luminance and alpha masks influence the transparency of this buffer before the compositing stage.

Clipping describes the visible region of visual elements. The region can be described by using certain SVG graphics elements or basic shapes. Anything outside of this region is not rendered.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	Other documents may supersede this document.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/Style/CSS/) as a <strong>Candidate Recommendation Draft</strong>. Publication as a Candidate Recommendation does not imply endorsement by the W3C Membership. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/fxtf-drafts/issues) (preferred), including the spec code “css-masking” in the title, like this: “\[css-masking\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-masking%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [15 September 2020 W3C Process Document](https://www.w3.org/2020/Process-20200915/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1. Introduction

<em>This section is not normative.</em>

This specification defines two different graphical operations which both fully or partly hide portions of an object: clipping and masking.

### <a id="clipping"></a>1.1. Clipping

A closed vector path, shape or polygon defines a so called <a id="clipping-path"></a>clipping path. This clipping path is a region (in the absence of anti-aliasing) where everything on the “inside” of this region is allowed to show through but everything on the outside is “clipped out” and does not appear on the canvas.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8d45bf9b"></a>
>
> ![Example Mask](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/clipping-path.svg)
>
> A clipping path (middle) is applied on a polygon shaded with different colors (left). This results in a “clipped out” shape (right).

<a id="ref-for-propdef-clip-path"></a>

<a id="ref-for-elementdef-clippath"></a>

<a id="ref-for-graphics-element"></a>

The [clip-path](#propdef-clip-path) property can use specified basic shapes as clipping path or reference an [clipPath](#elementdef-clippath) element with [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) to be used as clipping path.

### <a id="masking"></a>1.2. Masking

The effect of applying a mask to a graphical object is as if the graphical object will be painted onto the background through a mask, thus completely or partially masking out parts of the graphical object.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c9ae2e2a"></a>
>
> ![Example Mask](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/luminance-mask.svg)
>
> A luminance mask (middle) is applied on a shape filled with a gradient (left). This results in a masked shape (right).

<a id="ref-for-propdef-mask-image"></a>

<a id="ref-for-propdef-mask-border-source"></a>

Masks are applied using the [mask-image](#propdef-mask-image) or [mask-border-source](#propdef-mask-border-source) properties.

<a id="ref-for-propdef-mask-image①"></a>

<a id="ref-for-elementdef-mask"></a>

The [mask-image](#propdef-mask-image) property may reference a [mask](#elementdef-mask) element. The content of the <a id="ref-for-elementdef-mask①"></a>mask element serves as the mask.

<a id="ref-for-propdef-mask-image②"></a>

<a id="ref-for-elementdef-mask②"></a>

<a id="ref-for-propdef-mask-position"></a>

<a id="ref-for-propdef-mask-size"></a>

Alternatively, for many simple uses, the [mask-image](#propdef-mask-image) property may refer directly to images to be used as mask, forgoing the need for an explicit [mask](#elementdef-mask) element. This mask can then be sized and positioned just like CSS background images using the [mask-position](#propdef-mask-position), [mask-size](#propdef-mask-size) and other characterizing properties.

<a id="ref-for-propdef-mask-border-source①"></a>

<a id="ref-for-mask-border-image-area"></a>

<a id="ref-for-propdef-mask-border"></a>

The [mask-border-source](#propdef-mask-border-source) property splits a mask into 9 pieces. The pieces may be sliced, scaled and stretched in various ways to fit the size of the [mask border image area](#mask-border-image-area). The [mask-border](#propdef-mask-border) property serves as a shorthand property for <a id="ref-for-propdef-mask-border-source②"></a>mask-border-source and other characterizing properties.

<a id="ref-for-propdef-mask"></a>

<a id="ref-for-propdef-mask-border①"></a>

<a id="ref-for-propdef-mask-image③"></a>

The [mask](#propdef-mask) property serves as a shorthand property for all [mask-border](#propdef-mask-border) and [mask-image](#propdef-mask-image) affiliated properties.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While masking gives many possibilities for enhanced graphical effects and in general provides more control over the “visible portions” of the content, clipping paths can perform better and basic shapes are easier to interpolate.

## <a id="placement"></a>2. Module interactions

<a id="ref-for-TermStackingContext"></a>

This specification defines a set of CSS properties that affect the visual rendering of elements to which those properties are applied. These effects are applied after elements have been sized and positioned according to the [Visual formatting model](https://www.w3.org/TR/CSS2/visuren.html) from [\[CSS21\]](#biblio-css21). Some values of these properties result in the creation of a [stacking context](https://www.w3.org/TR/SVG2/render.html#TermStackingContext). Furthermore, this specification replaces the section [Clipping: the clip property](https://www.w3.org/TR/CSS2/visufx.html#clipping) from \[CSS21\].

The compositing model follows the SVG compositing model [\[SVG11\]](#biblio-svg11): First the element is styled under absence of filter effects, masking, clipping and opacity. Then the element and its descendants are drawn on a temporary canvas. In a last step the following effects are applied to the element in order: filter effects [\[FILTER-EFFECTS\]](#biblio-filter-effects), clipping, masking and opacity.

This specification allows compositing multiple mask layers with the Porter Duff compositing operators defined in CSS Compositing and Blending [\[COMPOSITING-1\]](#biblio-compositing-1).

<a id="ref-for-TermObjectBoundingBox"></a>

The term [object bounding box](https://www.w3.org/TR/SVG2/coords.html#TermObjectBoundingBox) follows the definition in SVG 1.1 [\[SVG11\]](#biblio-svg11).

## <a id="values"></a>3. Values

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS21/about.html#property-defs) from [\[CSS21\]](#biblio-css21). Basic shapes are defined in CSS Shapes Module Level 1 [\[CSS-SHAPES\]](#biblio-css-shapes). Value types not defined in these specifications are defined in CSS Values and Units Module Level 3 [\[CSS3VAL\]](#biblio-css3val).

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept CSS-wide keywords such as [inherit](https://www.w3.org/TR/CSS21/cascade.html#value-def-inherit) as their property value [\[CSS3VAL\]](#biblio-css3val). For readability it has not been repeated explicitly.

## <a id="terminology"></a>4. Terminology

Definitions of CSS properties and values in this specification are analogous to definitions in CSS Backgrounds and Borders [\[CSS3BG\]](#biblio-css3bg). To avoid redundancy, this specification relies on descriptions and definitions of CSS Backgrounds and Borders. The following terms in CSS Backgrounds and Borders have the following meaning in this specification:

<a id="term-matching"></a>

<strong>Table 1</strong>

| Term in CSS Masking | Term in [\[CSS3BG\]](#biblio-css3bg) |
| --- | --- |
| <strong><a id="ref-for-mask-layer-image"></a> [mask layer image](#mask-layer-image)</strong> | background images |
| <strong><a id="ref-for-mask-painting-area"></a> [mask painting area](#mask-painting-area)</strong> | <a id="ref-for-background-painting-area"></a> [background painting area](https://www.w3.org/TR/css-backgrounds-3/#background-painting-area) |
| <strong><a id="mask-size"></a>mask-size</strong> | background-size |
| <strong><a id="mask-position"></a>mask-position</strong> | background-position |
| <strong><a id="ref-for-mask-positioning-area"></a> [mask positioning area](#mask-positioning-area)</strong> | <a id="ref-for-background-positioning-area"></a> [background positioning area](https://www.w3.org/TR/css-backgrounds-3/#background-positioning-area) |
| <strong><a id="ref-for-mask-border-image"></a> [mask border image](#mask-border-image)</strong> | border-image |
| <strong><a id="ref-for-mask-border-image-area①"></a> [mask border image area](#mask-border-image-area)</strong> | <a id="ref-for-border-image-area"></a> [border image area](https://www.w3.org/TR/css-backgrounds-3/#border-image-area) |


## <a id="clipping-paths"></a>5. Clipping Paths

The clipping path restricts the region to which paint can be applied, the so-called <a id="clipping-region"></a>clipping region. Conceptually, any parts of the drawing that lie outside of this region are not drawn. This includes any content, background, borders, text decoration, outline and visible scrolling mechanism of the element to which the clipping path is applied, and those of its descendants.

<a id="ref-for-propdef-clip"></a>

<a id="ref-for-propdef-clip-path①"></a>

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-valdef-overflow-visible"></a>

An element’s ancestors may also clip portions of their content (e.g., via their own [clip](#propdef-clip) or [clip-path](#propdef-clip-path) properties and/or if their [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) property is not [visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible)). What is rendered is the cumulative intersection.

If the clipping region exceeds the bounds of the UA’s document window, content may be clipped to that window by the native operating environment.

<a id="ref-for-elementdef-clippath①"></a>

<a id="ref-for-propdef-clip-path②"></a>

A clipping path affects the rendering of an element. It does not affect the element’s inherent geometry. The geometry of a clipped element (i.e. an element which references a [clipPath](#elementdef-clippath) element via a [clip-path](#propdef-clip-path) property, or a child of the referencing element) must remain the same as if it were not clipped.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b04372df"></a> Consider a shape that is clipped by a clipping path applied to an ancestor:
>
> ```text
> <g clip-path="circle()">
>   <path id="shape" d="M0,0 L10,10, L 20,0 z"/>
> </g>
> ```
>
> <a id="ref-for-elementdef-use"></a>
>
> The shape is referenced by a [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element:
>
> ```text
> <use xlink:href="#shape"/>
> ```
>
> The geometry of the shape is not influenced by the circular clipping path.

<a id="ref-for-PointerEventsProperty"></a>

<a id="ref-for-clipping-region"></a>

By default, [pointer-events](https://www.w3.org/TR/SVG2/interact.html#PointerEventsProperty) must not be dispatched on the clipped-out (non-visible) regions of a shape. For example, an element with a dimension of 10px to 10px which is clipped to a circle with a radius of 5px will not receive click events outside the [clipping region](#clipping-region).

<a id="ref-for-propdef-clip-path③"></a>

### <a id="the-clip-path"></a>5.1. Clipping Shape: the [clip-path](#propdef-clip-path) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-clip-path"></a>clip-path                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-geometry-box"></a><a id="ref-for-comb-any"></a><a id="ref-for-typedef-basic-shape"></a><a id="ref-for-comb-one"></a><a id="ref-for-typedef-clip-source"></a>[\<clip-source\>](#typedef-clip-source) [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<geometry-box\>](#typedef-geometry-box) \] <a id="ref-for-comb-one①"></a>\| none                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use①"></a><a id="ref-for-graphics-element①"></a><a id="ref-for-elementdef-defs"></a><a id="ref-for-container-element"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-url-value"></a>as specified, but with [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) values made absolute                                                                                                                                                                                                                                                                                                                            |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

<a id="ref-for-elementdef-clippath②"></a>

<a id="ref-for-clipping-path"></a>

Specifies a basic shape or references a [clipPath](#elementdef-clippath) element to create a [clipping path](#clipping-path).

<a id="typedef-clip-source"></a>

<a id="ref-for-url-value①"></a>

```text
<clip-source> = <url>
```
<a id="typedef-geometry-box"></a>

<a id="ref-for-typedef-shape-box"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-comb-one④"></a>

```text
<geometry-box> = <shape-box> | fill-box | stroke-box | view-box
```
<a id="ref-for-typedef-basic-shape①"></a>

[\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape)

<a id="ref-for-valdef-mask-clip-border-box"></a>

A basic shape function as defined in the CSS Shapes module [\[CSS-SHAPES\]](#biblio-css-shapes). A basic shape makes use of the specified reference box to size and position the basic shape. If no reference box is specified, the [border-box](#valdef-mask-clip-border-box) will be used as reference box.

<a id="ref-for-typedef-geometry-box①"></a>

[\<geometry-box\>](#typedef-geometry-box)

<a id="ref-for-typedef-basic-shape②"></a>

If specified in combination with a [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) it provides the reference box for the <a id="ref-for-typedef-basic-shape③"></a>\<basic-shape\>.

<a id="ref-for-propdef-border-radius"></a>

If specified by itself, uses the edges of the specified box, including any corner shaping (e.g. defined by [border-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-radius) [\[CSS3BG\]](#biblio-css3bg)), as clipping path. See also [“Shapes from box values”](https://www.w3.org/TR/css-shapes/#shapes-from-box-values) [\[CSS-SHAPES\]](#biblio-css-shapes).

<a id="valdef-clip-path-fill-box"></a>fill-box

<a id="ref-for-TermObjectBoundingBox①"></a>

Uses the [object bounding box](https://www.w3.org/TR/SVG2/coords.html#TermObjectBoundingBox) as reference box.

<a id="valdef-clip-path-stroke-box"></a>stroke-box

<a id="ref-for-stroke-bounding-box"></a>

Uses the [stroke bounding box](#stroke-bounding-box) as reference box.

<a id="valdef-clip-path-view-box"></a>view-box

Uses the nearest [SVG viewport](https://www.w3.org/TR/SVG11/intro.html#TermSVGViewport) as reference box.

<a id="ref-for-ViewBoxAttribute"></a>

If a [viewBox](https://svgwg.org/svg2-draft/coords.html#ViewBoxAttribute) attribute is specified for the [SVG viewport](https://www.w3.org/TR/SVG11/intro.html#TermSVGViewport) creating element:

- <a id="ref-for-ViewBoxAttribute①"></a>

  The reference box is positioned at the origin of the coordinate system established by the [viewBox](https://svgwg.org/svg2-draft/coords.html#ViewBoxAttribute) attribute.

- <a id="ref-for-ViewBoxAttribute②"></a>

  The dimension of the reference box is set to the <em>width</em> and <em>height</em> values of the [viewBox](https://svgwg.org/svg2-draft/coords.html#ViewBoxAttribute) attribute.

none

<a id="ref-for-clipping-path①"></a>

No [clipping path](#clipping-path) gets created.

<a id="ref-for-used-value"></a>

<a id="ref-for-valdef-mask-clip-content-box"></a>

<a id="ref-for-valdef-mask-clip-padding-box"></a>

<a id="ref-for-valdef-clip-path-fill-box"></a>

<a id="ref-for-valdef-mask-clip-border-box①"></a>

<a id="ref-for-valuedef-margin-box0"></a>

<a id="ref-for-valdef-clip-path-stroke-box"></a>

For SVG elements without associated CSS layout box, the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) for [content-box](#valdef-mask-clip-content-box) and [padding-box](#valdef-mask-clip-padding-box) is [fill-box](#valdef-clip-path-fill-box) and for [border-box](#valdef-mask-clip-border-box) and [margin-box](https://www.w3.org/TR/css-masking-1/#valuedef-margin-box0) is [stroke-box](#valdef-clip-path-stroke-box).

<a id="ref-for-used-value①"></a>

<a id="ref-for-valdef-clip-path-fill-box①"></a>

<a id="ref-for-valdef-mask-clip-content-box①"></a>

<a id="ref-for-valdef-clip-path-stroke-box①"></a>

<a id="ref-for-valdef-clip-path-view-box"></a>

<a id="ref-for-valdef-mask-clip-border-box②"></a>

For elements with associated CSS layout box, the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) for [fill-box](#valdef-clip-path-fill-box) is [content-box](#valdef-mask-clip-content-box) and for [stroke-box](#valdef-clip-path-stroke-box) and [view-box](#valdef-clip-path-view-box) is [border-box](#valdef-mask-clip-border-box).

<a id="ref-for-TermStackingContext①"></a>

<a id="ref-for-propdef-opacity"></a>

A computed value of other than none results in the creation of a [stacking context](https://www.w3.org/TR/SVG2/render.html#TermStackingContext) [\[CSS21\]](#biblio-css21) the same way that CSS [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) [\[CSS3COLOR\]](#biblio-css3color) does for values other than 1.

<a id="ref-for-elementdef-clippath③"></a>

If the URI reference is not valid (e.g it points to an object that doesn’t exist or the object is not a [clipPath](#elementdef-clippath) element), no clipping is applied.

<a id="ref-for-funcdef-polygon"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b11e2c6d"></a> This example demonstrates the use of the basic shape [\<polygon()\>](https://www.w3.org/TR/css-shapes-1/#funcdef-polygon) as clipping path. Each space separated length pair represents one point of the polygon. The visualized clipping path can be seen in the [introduction](#clipping).
>
> ```text
> clip-path: polygon(15px 99px, 30px 87px, 65px 99px, 85px 55px,
>         122px 57px, 184px 73px, 198px 105px, 199px 150px,
>         145px 159px, 155px 139px, 126px 120px, 112px 138px,
>         80px 128px, 39px 126px, 24px 104px);
> ```
<a id="ref-for-propdef-clip-path④"></a>

<a id="ref-for-elementdef-clippath④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-32370653"></a> In this example, the [clip-path](#propdef-clip-path) property references an SVG [clipPath](#elementdef-clippath) element. Each comma separated length pair represents one point of the polygon. As for the previous example, the visualized clipping path can be seen in the [introduction](#clipping).
>
> ```text
> clip-path: url("#clip1");
> ```
>
> ```text
> <clipPath id="clip1">
>     <polygon points="15,99 30,87 65,99 85,55 122,57 184,73 198,105
>         199,150 145,159 155,139 126,120 112,138 80,128 39,126 24,104"/>
> </clipPath>
> ```
<a id="ref-for-propdef-clip-path⑤"></a>

The [clip-path](#propdef-clip-path) property is a [presentation attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) for SVG elements.

## <a id="svg-clipping-paths"></a>6. SVG Clipping Path Sources

<a id="ref-for-elementdef-clippath⑤"></a>

### <a id="ClipPathElement"></a>6.1. The [clipPath](#elementdef-clippath) element

**Name:**

<a id="elementdef-clippath"></a>`clipPath`

**Categories:**

<a id="ref-for-TermNeverRenderedElement"></a><a id="ref-for-container-element①"></a>[container elements](https://www.w3.org/TR/SVG2/struct.html#container-element), [never-rendered element](https://svgwg.org/svg2-draft/render.html#TermNeverRenderedElement)

**Content model:**

Any number of the following elements, in any order:

- <a id="ref-for-elementdef-metadata"></a><a id="ref-for-elementdef-title"></a><a id="ref-for-elementdef-desc"></a>[descriptive](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermDescriptiveElement) — [desc](https://www.w3.org/TR/SVG2/struct.html#elementdef-desc), [title](https://www.w3.org/TR/SVG2/struct.html#elementdef-title), [metadata](https://www.w3.org/TR/SVG2/struct.html#elementdef-metadata)

- <a id="ref-for-SetElement"></a><a id="ref-for-AnimateTransformElement"></a><a id="ref-for-AnimateMotionElement"></a><a id="ref-for-AnimateColorElement"></a><a id="ref-for-AnimateElement"></a>[animation](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermAnimationElement) — [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [animateColor](https://www.w3.org/TR/SVG11/animate.html#AnimateColorElement), [animateMotion](https://www.w3.org/TR/SVG11/animate.html#AnimateMotionElement), [animateTransform](https://www.w3.org/TR/SVG11/animate.html#AnimateTransformElement), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement)

- <a id="ref-for-elementdef-rect"></a><a id="ref-for-elementdef-polyline"></a><a id="ref-for-elementdef-polygon"></a><a id="ref-for-elementdef-path"></a><a id="ref-for-elementdef-line"></a><a id="ref-for-elementdef-ellipse"></a><a id="ref-for-elementdef-circle"></a>[shape](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermShapeElement) — [circle](https://www.w3.org/TR/SVG2/shapes.html#elementdef-circle), [ellipse](https://www.w3.org/TR/SVG2/shapes.html#elementdef-ellipse), [line](https://www.w3.org/TR/SVG2/shapes.html#elementdef-line), [path](https://www.w3.org/TR/SVG2/paths.html#elementdef-path), [polygon](https://www.w3.org/TR/SVG2/shapes.html#elementdef-polygon), [polyline](https://www.w3.org/TR/SVG2/shapes.html#elementdef-polyline), [rect](https://www.w3.org/TR/SVG2/shapes.html#elementdef-rect)

- <a id="ref-for-elementdef-text"></a>[text](https://www.w3.org/TR/SVG2/text.html#elementdef-text)

- <a id="ref-for-elementdef-use②"></a>[use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use)

- <a id="ref-for-elementdef-script"></a>[script](https://www.w3.org/TR/SVG2/interact.html#elementdef-script)

**Attributes:**

- [conditional processing attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermConditionalProcessingAttribute) — [‘requiredFeatures’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#RequiredFeaturesAttribute), [‘requiredExtensions’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#RequiredExtensionsAttribute), [‘systemLanguage’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#SystemLanguageAttribute)

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [‘id’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [‘xml:base’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [‘xml:lang’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [‘xml:space’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode"></a><a id="ref-for-propdef-word-spacing"></a><a id="ref-for-propdef-visibility"></a><a id="ref-for-propdef-unicode-bidi"></a><a id="ref-for-TextRenderingProperty"></a><a id="ref-for-propdef-text-decoration"></a><a id="ref-for-TextAnchorProperty"></a><a id="ref-for-StrokeWidthProperty"></a><a id="ref-for-StrokeOpacityProperty"></a><a id="ref-for-StrokeMiterlimitProperty"></a><a id="ref-for-StrokeLinejoinProperty"></a><a id="ref-for-StrokeLinecapProperty"></a><a id="ref-for-StrokeDashoffsetProperty"></a><a id="ref-for-StrokeDasharrayProperty"></a><a id="ref-for-StrokeProperty"></a><a id="ref-for-StopOpacityProperty"></a><a id="ref-for-StopColorProperty"></a><a id="ref-for-ShapeRenderingProperty"></a><a id="ref-for-PointerEventsProperty①"></a><a id="ref-for-propdef-overflow①"></a><a id="ref-for-propdef-opacity①"></a><a id="ref-for-propdef-mask①"></a><a id="ref-for-MarkerStartProperty"></a><a id="ref-for-MarkerMidProperty"></a><a id="ref-for-MarkerEndProperty"></a><a id="ref-for-MarkerProperty"></a><a id="ref-for-propdef-lighting-color"></a><a id="ref-for-propdef-letter-spacing"></a><a id="ref-for-KerningProperty"></a><a id="ref-for-propdef-image-rendering"></a><a id="ref-for-propdef-glyph-orientation-vertical"></a><a id="ref-for-GlyphOrientationHorizontalProperty"></a><a id="ref-for-propdef-font-weight"></a><a id="ref-for-propdef-font-variant"></a><a id="ref-for-propdef-font-style"></a><a id="ref-for-propdef-font-stretch"></a><a id="ref-for-propdef-font-size-adjust"></a><a id="ref-for-descdef-font-face-font-size"></a><a id="ref-for-propdef-font-family"></a><a id="ref-for-propdef-font"></a><a id="ref-for-propdef-flood-opacity"></a><a id="ref-for-propdef-flood-color"></a><a id="ref-for-propdef-filter"></a><a id="ref-for-FillRuleProperty"></a><a id="ref-for-FillOpacityProperty"></a><a id="ref-for-FillProperty"></a><a id="ref-for-EnableBackgroundProperty"></a><a id="ref-for-propdef-dominant-baseline"></a><a id="ref-for-propdef-display"></a><a id="ref-for-propdef-direction"></a><a id="ref-for-propdef-cursor"></a><a id="ref-for-ColorRenderingProperty"></a><a id="ref-for-ColorProfileProperty"></a><a id="ref-for-propdef-color-interpolation-filters"></a><a id="ref-for-ColorInterpolationProperty"></a><a id="ref-for-propdef-color"></a><a id="ref-for-propdef-clip-rule"></a><a id="ref-for-propdef-clip-path⑥"></a><a id="ref-for-propdef-clip①"></a><a id="ref-for-propdef-baseline-shift"></a><a id="ref-for-propdef-alignment-baseline"></a>[presentation attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/css-inline-3/#propdef-alignment-baseline), [baseline-shift](https://www.w3.org/TR/css-inline-3/#propdef-baseline-shift), [clip](#propdef-clip), [clip-path](#propdef-clip-path), [clip-rule](#propdef-clip-rule), [color](https://www.w3.org/TR/css-color-4/#propdef-color), [color-interpolation](https://www.w3.org/TR/SVG2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](https://www.w3.org/TR/filter-effects-1/#propdef-color-interpolation-filters), [color-profile](https://www.w3.org/TR/SVG11/color.html#ColorProfileProperty), [color-rendering](https://www.w3.org/TR/SVG2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css-ui-3/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/css-inline-3/#propdef-dominant-baseline), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/SVG2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/SVG2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/SVG2/painting.html#FillRuleProperty), [filter](https://www.w3.org/TR/filter-effects-1/#propdef-filter), [flood-color](https://www.w3.org/TR/filter-effects-1/#propdef-flood-color), [flood-opacity](https://www.w3.org/TR/filter-effects-1/#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-4/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-4/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-5/#descdef-font-face-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-5/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-4/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-4/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-4/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-4/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/css-writing-modes-4/#propdef-glyph-orientation-vertical), [image-rendering](https://www.w3.org/TR/css-images-3/#propdef-image-rendering), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](https://www.w3.org/TR/filter-effects-1/#propdef-lighting-color), [marker](https://www.w3.org/TR/SVG2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/SVG2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/SVG2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/SVG2/painting.html#MarkerStartProperty), [mask](#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/SVG2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/SVG2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/SVG2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/SVG2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/SVG2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/SVG2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/SVG2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/SVG2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/SVG2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/SVG2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/SVG2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/SVG2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/SVG2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/SVG2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS2/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- [‘class’](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [‘style’](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- [‘externalResourcesRequired’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#ExternalResourcesRequiredAttribute)

- [‘transform’](https://www.w3.org/TR/2011/REC-SVG11-20110816/coords.html#TransformAttribute)

- <a id="ref-for-element-attrdef-clippath-clippathunits"></a>‘[clipPathUnits](#element-attrdef-clippath-clippathunits)’

**DOM Interfaces:**

[SVGClipPathElement](#InterfaceSVGClipPathElement)

<em>Attribute definitions:</em>

<a id="ref-for-valdef-clippathunits-objectboundingbox"></a>

<a id="ref-for-valdef-clippathunits-userspaceonuse"></a>

<a id="element-attrdef-clippath-clippathunits"></a>`clipPathUnits` = "[userSpaceOnUse](#valdef-clippathunits-userspaceonuse) \| [objectBoundingBox](#valdef-clippathunits-objectboundingbox)"

<a id="ref-for-elementdef-clippath⑥"></a>

Defines the coordinate system for the contents of the [clipPath](#elementdef-clippath).

<a id="valdef-clippathunits-userspaceonuse"></a>userSpaceOnUse

<a id="ref-for-elementdef-clippath⑦"></a>

<a id="ref-for-user-coordinate-system"></a>

<a id="ref-for-propdef-clip-path⑦"></a>

The contents of the [clipPath](#elementdef-clippath) represent values in the current [user coordinate system](https://www.w3.org/TR/css-transforms-1/#user-coordinate-system) in place at the time when the <a id="ref-for-elementdef-clippath⑧"></a>clipPath element is referenced (i.e., the <a id="ref-for-user-coordinate-system①"></a>user coordinate system for the element referencing the <a id="ref-for-elementdef-clippath⑨"></a>clipPath element via the [clip-path](#propdef-clip-path) property).

<a id="valdef-clippathunits-objectboundingbox"></a>objectBoundingBox

<a id="ref-for-bounding-box"></a>

<a id="ref-for-px"></a>

The coordinate system has its origin at the top left corner of the [bounding box](https://www.w3.org/TR/SVG2/coords.html#bounding-box) of the element to which the clipping path applies to and the same width and height of this <a id="ref-for-bounding-box①"></a>bounding box. [User coordinates](https://www.w3.org/TR/SVG/coords.html#Units) are sized equivalently to the CSS [px](https://www.w3.org/TR/css-values-4/#px) unit.

<a id="ref-for-element-attrdef-clippath-clippathunits①"></a>

<a id="ref-for-valdef-clippathunits-userspaceonuse①"></a>

If attribute [clipPathUnits](#element-attrdef-clippath-clippathunits) is not specified, then the effect is as if a value of [userSpaceOnUse](#valdef-clippathunits-userspaceonuse) were specified.

Animatable: yes.

<a id="ref-for-elementdef-clippath①⓪"></a>

CSS properties inherit into the [clipPath](#elementdef-clippath) element from its ancestors; properties do <em>not</em> inherit from the element referencing the <a id="ref-for-elementdef-clippath①①"></a>clipPath element.

<a id="ref-for-elementdef-clippath①②"></a>

<a id="ref-for-propdef-clip-path⑧"></a>

<a id="ref-for-propdef-display①"></a>

<a id="ref-for-valdef-display-none"></a>

[clipPath](#elementdef-clippath) elements are never rendered directly; their only usage is as something that can be referenced using the [clip-path](#propdef-clip-path) property. The [display](https://www.w3.org/TR/css-display-3/#propdef-display) property does not apply to the <a id="ref-for-elementdef-clippath①③"></a>clipPath element; thus, <a id="ref-for-elementdef-clippath①④"></a>clipPath elements are not directly rendered even if the <a id="ref-for-propdef-display②"></a>display property is set to a value other than [none](https://www.w3.org/TR/css-display-3/#valdef-display-none), and <a id="ref-for-elementdef-clippath①⑤"></a>clipPath elements are available for referencing even when the <a id="ref-for-propdef-display③"></a>display property on the <a id="ref-for-elementdef-clippath①⑥"></a>clipPath element or any of its ancestors is set to <a id="ref-for-valdef-display-none①"></a>none.

<a id="ref-for-elementdef-clippath①⑦"></a>

<a id="ref-for-elementdef-path①"></a>

<a id="ref-for-elementdef-text①"></a>

<a id="ref-for-basic-shape"></a>

<a id="ref-for-elementdef-circle①"></a>

<a id="ref-for-elementdef-use③"></a>

A [clipPath](#elementdef-clippath) element can contain [path](https://www.w3.org/TR/SVG2/paths.html#elementdef-path) elements, [text](https://www.w3.org/TR/SVG2/text.html#elementdef-text) elements, [basic shapes](https://www.w3.org/TR/SVG2/shapes.html#basic-shape) (such as [circle](https://www.w3.org/TR/SVG2/shapes.html#elementdef-circle)) or a [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element. If a <a id="ref-for-elementdef-use④"></a>use element is a child of a <a id="ref-for-elementdef-clippath①⑧"></a>clipPath element, it must directly reference <a id="ref-for-elementdef-path②"></a>path, <a id="ref-for-elementdef-text②"></a>text or <a id="ref-for-basic-shape①"></a>basic shapes elements. Indirect references are an error and the <a id="ref-for-elementdef-clippath①⑨"></a>clipPath element must be ignored.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d0c561ed"></a> Firefox disables rendering of elements referencing clipPaths with violated content model. No browser ignores clipPath on use with indirect reference. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;17&#x3E;](https://github.com/w3c/csswg-drafts/issues/17)

<a id="ref-for-FillProperty①"></a>

<a id="ref-for-StrokeProperty①"></a>

<a id="ref-for-StrokeWidthProperty①"></a>

<a id="ref-for-elementdef-clippath②⓪"></a>

<a id="ref-for-propdef-display④"></a>

<a id="ref-for-propdef-visibility①"></a>

The raw geometry of each child element exclusive of rendering properties such as [fill](https://www.w3.org/TR/SVG2/painting.html#FillProperty), [stroke](https://www.w3.org/TR/SVG2/painting.html#StrokeProperty), [stroke-width](https://www.w3.org/TR/SVG2/painting.html#StrokeWidthProperty) within a [clipPath](#elementdef-clippath) conceptually defines a 1-bit mask (with the possible exception of anti-aliasing along the edge of the geometry) which represents the silhouette of the graphics associated with that element. Anything outside the outline of the object is masked out. If a child element is made invisible by [display](https://www.w3.org/TR/css-display-3/#propdef-display) or [visibility](https://www.w3.org/TR/CSS2/visufx.html#propdef-visibility) it does not contribute to the clipping path. When the <a id="ref-for-elementdef-clippath②①"></a>clipPath element contains multiple child elements, the silhouettes of the child elements are logically OR’d together to create a single silhouette which is then used to restrict the region onto which paint can be applied. Thus, a point is inside the clipping path if it is inside any of the children of the <a id="ref-for-elementdef-clippath②②"></a>clipPath.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-1eaffdcc"></a> Define raw geometry with regards to CSS properties that affect it. Especially on text. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;170&#x3E;](https://github.com/w3c/csswg-drafts/issues/170)

<a id="ref-for-graphics-element②"></a>

<a id="ref-for-propdef-clip-path⑨"></a>

For a given [graphics element](https://www.w3.org/TR/SVG2/struct.html#graphics-element), the actual clipping path used will be the intersection of the clipping path specified by its [clip-path](#propdef-clip-path) property (if any) with any clipping paths on its ancestors, as specified by the <a id="ref-for-propdef-clip-path①⓪"></a>clip-path property on the elements which establish a new viewport. (See [\[SVG11\]](#biblio-svg11))

A couple of additions:

- <a id="ref-for-elementdef-clippath②③"></a>

  The [clipPath](#elementdef-clippath) element itself and its child elements do <em>not</em> inherit clipping paths from the ancestors of the <a id="ref-for-elementdef-clippath②④"></a>clipPath element.

- <a id="ref-for-elementdef-clippath②⑤"></a>

  <a id="ref-for-propdef-clip-path①①"></a>

  The [clipPath](#elementdef-clippath) element or any of its children can specify property [clip-path](#propdef-clip-path).  
  If a valid <a id="ref-for-propdef-clip-path①②"></a>clip-path reference is placed on a <a id="ref-for-elementdef-clippath②⑥"></a>clipPath element, the resulting clipping path is the intersection of the contents of the <a id="ref-for-elementdef-clippath②⑦"></a>clipPath element with the referenced clipping path.  
  If a valid <a id="ref-for-propdef-clip-path①③"></a>clip-path reference is placed on one of the children of a <a id="ref-for-elementdef-clippath②⑧"></a>clipPath element, then the given child element is clipped by the referenced clipping path before OR’ing the silhouette of the child element with the silhouettes of the other child elements.

- <a id="ref-for-propdef-clip-path①④"></a>

  An empty clipping path will completely clip away the element that had the [clip-path](#propdef-clip-path) property applied.

<a id="ref-for-propdef-clip-rule①"></a>

### <a id="the-clip-rule"></a>6.2. Winding Rules: the [clip-rule](#propdef-clip-rule) property

| Field               | Definition                                                                                                     |
|---------------------|----------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-clip-rule"></a>clip-rule                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑤"></a>nonzero [\|](https://www.w3.org/TR/css-values-4/#comb-one) evenodd                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | nonzero                                                                                                        |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-graphics-element③"></a>Applies to SVG [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                   |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                       |
| <strong>Media:&#xA;      </strong> | visual                                                                                                         |

<a id="ref-for-propdef-clip-rule②"></a>

<a id="ref-for-clipping-region①"></a>

<a id="ref-for-graphics-element④"></a>

<a id="ref-for-FillRuleProperty①"></a>

The [clip-rule](#propdef-clip-rule) property indicates the algorithm which is to be used to determine whether a given point is inside a shape for a [clipping region](#clipping-region) created with a [graphics element](https://www.w3.org/TR/SVG2/struct.html#graphics-element). The definition of the algorithms and the <a id="ref-for-propdef-clip-rule③"></a>clip-rule values follows the definition of the [fill-rule](https://www.w3.org/TR/SVG2/painting.html#FillRuleProperty) property. See section [“Fill Properties”](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#FillProperties) in SVG 1.1 [\[SVG11\]](#biblio-svg11).

<a id="valdef-clip-rule-nonzero"></a>nonzero  
<a id="ref-for-FillRuleProperty②"></a>

See description of [fill-rule](https://www.w3.org/TR/SVG2/painting.html#FillRuleProperty) property [\[SVG11\]](#biblio-svg11).

<a id="valdef-clip-rule-evenodd"></a>evenodd  
<a id="ref-for-FillRuleProperty③"></a>

See description of [fill-rule](https://www.w3.org/TR/SVG2/painting.html#FillRuleProperty) property [\[SVG11\]](#biblio-svg11).

<a id="ref-for-propdef-clip-rule④"></a>

<a id="ref-for-graphics-element⑤"></a>

<a id="ref-for-elementdef-clippath②⑨"></a>

The [clip-rule](#propdef-clip-rule) property only applies to [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) that are contained within a [clipPath](#elementdef-clippath) element.

<a id="ref-for-propdef-clip-rule⑤"></a>

<a id="ref-for-typedef-basic-shape④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [clip-rule](#propdef-clip-rule) property does not apply to [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape)s.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3153479a"></a>
>
> <a id="ref-for-valdef-clip-rule-nonzero"></a>
>
> The following drawing illustrates the [nonzero](#valdef-clip-rule-nonzero) rule:
>
> ![Shape with nonzero rule.](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/cliprule-nonzero.svg)
>
> Three shapes from left to right: Star with 5 points drawn in one continuous, overlapping line; 2 clockwise drawn circles, one contains the other and both are subpaths of the same shape; 2 circles, one containing the other with the bigger one drawn in a clockwise direction and the smaller one in a counter-clockwise direction and both belonging to the same shape. Only the last shape has a "hole".
>
> <a id="ref-for-valdef-clip-rule-evenodd"></a>
>
> The following drawing illustrates the [evenodd](#valdef-clip-rule-evenodd) rule:
>
> ![Shape with even-odd rule.](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/cliprule-evenodd.svg)
>
> Three shapes from left to right: Star with 5 points drawn in one continuous, overlapping line; 2 clockwise drawn circles, one contains the other and both are subpaths of the same shape; 2 circles, one containing the other with the bigger one drawn in a clockwise direction and the smaller one in a counter-clockwise direction and both belonging to the same shape. All 3 shapes have a "hole".

<a id="ref-for-propdef-clip-rule⑥"></a>

<a id="ref-for-elementdef-path③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4b844436"></a> The following fragment of code will cause an even-odd clipping rule to be applied to the clipping path because [clip-rule](#propdef-clip-rule) is specified on the [path](https://www.w3.org/TR/SVG2/paths.html#elementdef-path) element that defines the clipping shape:
>
> ```text
> <g clip-rule="nonzero">
>   <clipPath id="MyClip">
>     <path d="..." clip-rule="evenodd" />
>   </clipPath>
>   <rect clip-path="url(#MyClip)" ... />
> </g>
> ```
>
> <a id="ref-for-propdef-clip-rule⑦"></a>
>
> whereas the following fragment of code will <em>not</em> cause an evenodd clipping rule to be applied because the [clip-rule](#propdef-clip-rule) is specified on the referencing element, not on the object defining the clipping shape:
>
> ```text
> <g clip-rule="nonzero">
>   <clipPath id="MyClip">
>     <path d="..." />
>   </clipPath>
>   <rect clip-path="url(#MyClip)" clip-rule="evenodd" ... />
> </g>
> ```
<a id="ref-for-propdef-clip-rule⑧"></a>

The [clip-rule](#propdef-clip-rule) property is a [presentation attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) for SVG elements.

## <a id="positioned-masks"></a>7. Positioned Masks

<a id="ref-for-propdef-mask-image④"></a>

### <a id="the-mask-image"></a>7.1. Mask Image Source: the [mask-image](#propdef-mask-image) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-image"></a>mask-image                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma"></a><a id="ref-for-typedef-mask-reference"></a>[\<mask-reference\>](#typedef-mask-reference)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use⑤"></a><a id="ref-for-graphics-element⑥"></a><a id="ref-for-elementdef-defs①"></a><a id="ref-for-container-element②"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-url-value②"></a><a id="ref-for-typedef-image"></a>the keyword none, a computed [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image), or a computed [\<url\>](https://www.w3.org/TR/css-values-4/#url-value)                                                                                                                                                                                                                                          |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

This property sets the <a id="mask-layer-image"></a>mask layer image of an element. Where:

<a id="typedef-mask-reference"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-typedef-image①"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-typedef-mask-source"></a>

```text
<mask-reference> = none | <image> | <mask-source>
```
<a id="typedef-mask-source"></a>

<a id="ref-for-url-value③"></a>

```text
<mask-source> = <url>
```
<a id="valdef-mask-image-url"></a>\<url\>  
<a id="ref-for-elementdef-mask③"></a>

A URL reference to a [mask](#elementdef-mask) element (for example url(commonmasks.svg#mask)) or to a CSS image.

none  
A value of none counts as a transparent black image layer.

<a id="ref-for-TermStackingContext②"></a>

<a id="ref-for-propdef-opacity②"></a>

A computed value of other than none results in the creation of a [stacking context](https://www.w3.org/TR/SVG2/render.html#TermStackingContext) [\[CSS21\]](#biblio-css21) the same way that CSS [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) [\[CSS3COLOR\]](#biblio-css3color) does for values other than 1.

<a id="ref-for-elementdef-mask④"></a>

A mask reference that is an empty image (zero width or zero height), that fails to download, is not a reference to an [mask](#elementdef-mask) element, is non-existent, or that cannot be displayed (e.g. because it is not in a supported image format) still counts as an image layer of transparent black.

<a id="ref-for-mask-layer-image①"></a>

See the section [“Mask processing”](#MaskValues) for how to process a [mask layer image](#mask-layer-image).

<a id="ref-for-typedef-mask-reference①"></a>

<a id="ref-for-propdef-mask-composite"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A value of none in a list of [\<mask-reference\>](#typedef-mask-reference)s may influence the masking operation depending on the used compositing operator specified by [mask-composite](#propdef-mask-composite).

<a id="ref-for-typedef-mask-source①"></a>

<a id="ref-for-typedef-mask-reference②"></a>

<a id="ref-for-typedef-image②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A [\<mask-source\>](#typedef-mask-source) counts as mask layer and can be combined in a repeatable [\<mask-reference\>](#typedef-mask-reference) list with [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) or further <a id="ref-for-typedef-mask-source②"></a>\<mask-source\> list items.

<a id="ref-for-propdef-mask-border-source③"></a>

<a id="ref-for-propdef-mask-image⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: An element can also be masked with [mask-border-source](#propdef-mask-border-source). See <a id="ref-for-propdef-mask-border-source④"></a>mask-border-source for the interaction of that property with [mask-image](#propdef-mask-image).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4c7f99bf"></a> Examples for mask references:
>
> ```text
> body { mask-image: linear-gradient(black 0%, transparent 100%) }
> p { mask-image: none }
> div { mask-image: url(resources.svg#mask2) }
> ```
<a id="ref-for-propdef-mask-image⑥"></a>

See the section [“Layering multiple mask layer images”](#layering) for how [mask-image](#propdef-mask-image) interacts with other comma-separated mask properties to form each mask layer.

<a id="ref-for-propdef-mask-mode"></a>

### <a id="the-mask-mode"></a>7.2. Mask Image Interpretation: the [mask-mode](#propdef-mask-mode) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-mode"></a>mask-mode                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma①"></a><a id="ref-for-typedef-masking-mode"></a>[\<masking-mode\>](#typedef-masking-mode)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | match-source                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use⑥"></a><a id="ref-for-graphics-element⑦"></a><a id="ref-for-elementdef-defs②"></a><a id="ref-for-container-element③"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

<a id="ref-for-propdef-mask-mode①"></a>

<a id="ref-for-typedef-mask-reference③"></a>

The [mask-mode](#propdef-mask-mode) property indicates whether the [\<mask-reference\>](#typedef-mask-reference) is treated as luminance mask or alpha mask. (See [Mask processing](#MaskValues).)

<a id="typedef-masking-mode"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-comb-one⑨"></a>

```text
<masking-mode> = alpha | luminance | match-source
```
Values have the following meanings:

<a id="valdef-mask-mode-alpha"></a>alpha  
<a id="ref-for-valdef-mask-mode-alpha"></a>

<a id="ref-for-mask-layer-image②"></a>

A value of [alpha](#valdef-mask-mode-alpha) indicates that the alpha values of the [mask layer image](#mask-layer-image) should be used as the mask values. See [Calculating mask values](#MaskValues).

<a id="valdef-mask-mode-luminance"></a>luminance  
<a id="ref-for-valdef-mask-mode-luminance"></a>

<a id="ref-for-mask-layer-image③"></a>

A value of [luminance](#valdef-mask-mode-luminance) indicates that the luminance values of the [mask layer image](#mask-layer-image) should be used as the mask values. See [Calculating mask values](#MaskValues).

<a id="valdef-mask-mode-match-source"></a>match-source  
<a id="ref-for-typedef-mask-reference④"></a>

<a id="ref-for-propdef-mask-image⑦"></a>

<a id="ref-for-typedef-mask-source③"></a>

<a id="ref-for-elementdef-mask⑤"></a>

<a id="ref-for-propdef-mask-type"></a>

If the [\<mask-reference\>](#typedef-mask-reference) of the [mask-image](#propdef-mask-image) property is of type [\<mask-source\>](#typedef-mask-source) the value specified by the referenced [mask](#elementdef-mask) element’s [mask-type](#propdef-mask-type) property must be used.

<a id="ref-for-typedef-mask-reference⑤"></a>

<a id="ref-for-propdef-mask-image⑧"></a>

<a id="ref-for-typedef-image③"></a>

<a id="ref-for-mask-layer-image④"></a>

If the [\<mask-reference\>](#typedef-mask-reference) of the [mask-image](#propdef-mask-image) property is of type [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) the alpha values of the [mask layer image](#mask-layer-image) should be used as the mask values.

<a id="ref-for-propdef-mask-type①"></a>

<a id="ref-for-elementdef-mask⑥"></a>

<a id="ref-for-valdef-mask-type-alpha"></a>

<a id="ref-for-propdef-mask-image⑨"></a>

<a id="ref-for-propdef-mask-mode②"></a>

<a id="ref-for-valdef-mask-mode-luminance①"></a>

<a id="ref-for-valdef-mask-type-luminance"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ec098273"></a> In the following example, the [mask-type](#propdef-mask-type) property sets the mask type value for the [mask](#elementdef-mask) element to [alpha](#valdef-mask-type-alpha). The [mask-image](#propdef-mask-image) property has a reference to this <a id="ref-for-elementdef-mask⑦"></a>mask element and the [mask-mode](#propdef-mask-mode) property has a value of [luminance](#valdef-mask-mode-luminance). The <a id="ref-for-propdef-mask-mode③"></a>mask-mode property will override the definition of <a id="ref-for-propdef-mask-type②"></a>mask-type to [luminance](#valdef-mask-type-luminance).
>
> <a id="ref-for-propdef-mask-mode④"></a>
>
> <a id="ref-for-propdef-mask-border-source⑤"></a>
>
> The [mask-mode](#propdef-mask-mode) property must not affect the masking mode of [mask-border-source](#propdef-mask-border-source).
>
> ```text
> <mask id="SVGMask" mask-type="alpha" maskContentUnits="objectBoundingBox">
>   <radialGradient id="radialFill">
>     <stop stop-color="white" offset="0"/>
>     <stop stop-color="black" offset="1"/>
>   </radialGradient>
>   <circle fill="url(#radialFill)" cx="0.5" cy="0.5" r="0.5"/>
> </mask>
> 
> <style>
>   rect {
>     mask-image: url(#SVGMask);
>     mask-mode: luminance;
>   }
> </style>
> 
> <rect width="200" height="200" fill="green"/>
> ```
<a id="ref-for-propdef-mask-mode⑤"></a>

See the section [“Layering multiple mask layer images”](#layering) for how [mask-mode](#propdef-mask-mode) interacts with other comma-separated mask properties to form each mask layer.

<a id="ref-for-propdef-mask-repeat"></a>

### <a id="the-mask-repeat"></a>7.3. Tiling Mask Images: the [mask-repeat](#propdef-mask-repeat) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-repeat"></a>mask-repeat                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma②"></a><a id="ref-for-typedef-repeat-style"></a>[\<repeat-style\>](https://www.w3.org/TR/css-backgrounds-3/#typedef-repeat-style)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | repeat                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use⑦"></a><a id="ref-for-graphics-element⑧"></a><a id="ref-for-elementdef-defs③"></a><a id="ref-for-container-element④"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | Consists of: two keywords, one per dimension                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

<a id="ref-for-mask-layer-image⑤"></a>

Specifies how [mask layer images](#mask-layer-image) are tiled after they have been [sized](#the-mask-size) and [positioned](#the-mask-position).

<a id="ref-for-propdef-background-repeat"></a>

See [background-repeat](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-repeat) property [\[CSS3BG\]](#biblio-css3bg) for the definitions of the property values.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6b3f7f82"></a>
>
> ```text
> body {
>     background-color: blue;
>     mask-image: url(dot-mask.png) luminance;
>     mask-repeat: space;
> }
> ```
>
> ![Image of an element with a dotted mask.](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/mask-repeat.svg)
>
> <a id="ref-for-mask-layer-image⑥"></a>
>
> <a id="ref-for-mask-painting-area①"></a>
>
> The effect of space: the [mask layer image](#mask-layer-image) of a dot is tiled to cover the whole [mask painting area](#mask-painting-area) and the <a id="ref-for-mask-layer-image⑦"></a>mask layer images are equally spaced.

<a id="ref-for-propdef-mask-repeat①"></a>

See the section [“Layering multiple mask layer images”](#layering) for how [mask-repeat](#propdef-mask-repeat) interacts with other comma-separated mask properties to form each mask layer.

<a id="ref-for-propdef-mask-position①"></a>

### <a id="the-mask-position"></a>7.4. Positioning Mask Images: the [mask-position](#propdef-mask-position) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-position"></a>mask-position                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma③"></a><a id="ref-for-typedef-position"></a>[\<position\>](https://www.w3.org/TR/css-values-4/#typedef-position)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0% 0%                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use⑧"></a><a id="ref-for-graphics-element⑨"></a><a id="ref-for-elementdef-defs④"></a><a id="ref-for-container-element⑤"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-propdef-background-position"></a><a id="ref-for-mask-layer-image⑧"></a><a id="ref-for-mask-painting-area②"></a>refer to size of [mask painting area](#mask-painting-area) <em>minus</em> size of [mask layer image](#mask-layer-image); see text [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) [\[CSS3BG\]](#biblio-css3bg)                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-length-value"></a>Consisting of: two keywords representing the origin and two offsets from that origin, each given as an absolute length (if given a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)), otherwise as a percentage.                                                                                                                                                                                                  |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | repeatable list                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

<a id="ref-for-propdef-background-position①"></a>

See the [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) property [\[CSS3BG\]](#biblio-css3bg) for the definitions of the property values.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-80025e44"></a> In the example below, the (single) image is placed in the lower-right corner of the viewport.
>
> ```text
> body {
>     mask-image: url("logo.png");
>     mask-position: 100% 100%;
>     mask-repeat: no-repeat;
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-dc3e5c49"></a> Mask positions can also be relative to other corners than the top left. E.g., the following puts the background image 10px from the bottom and 3em from the right:
>
> ```text
> mask-position: right 3em bottom 10px
> ```
<a id="ref-for-propdef-mask-position②"></a>

See the section [“Layering multiple mask layer images”](#layering) for how [mask-position](#propdef-mask-position) interacts with other comma-separated mask properties to form each mask layer.

<a id="ref-for-propdef-mask-clip"></a>

### <a id="the-mask-clip"></a>7.5. Masking Area: the [mask-clip](#propdef-mask-clip) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-clip"></a>mask-clip                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma④"></a><a id="ref-for-comb-one①⓪"></a><a id="ref-for-typedef-geometry-box②"></a>\[ [\<geometry-box\>](#typedef-geometry-box) [\|](https://www.w3.org/TR/css-values-4/#comb-one) no-clip \][\#](https://www.w3.org/TR/css-values-4/#mult-comma)                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | border-box                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use⑨"></a><a id="ref-for-graphics-element①⓪"></a><a id="ref-for-elementdef-defs⑤"></a><a id="ref-for-container-element⑥"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

<a id="ref-for-mask-layer-image⑨"></a>

<a id="ref-for-elementdef-mask⑧"></a>

<a id="ref-for-propdef-mask-clip①"></a>

For [mask layer images](#mask-layer-image) that do not reference a [mask](#elementdef-mask) element, [mask-clip](#propdef-mask-clip) determines the <a id="mask-painting-area"></a>mask painting area, which determines the area that is affected by the mask. The painted content of an element must be restricted to this area.

<a id="ref-for-propdef-mask-clip②"></a>

<a id="ref-for-mask-layer-image①⓪"></a>

<a id="ref-for-elementdef-mask⑨"></a>

<a id="ref-for-element-attrdef-mask-x"></a>

<a id="ref-for-element-attrdef-mask-y"></a>

<a id="ref-for-element-attrdef-mask-width"></a>

<a id="ref-for-element-attrdef-mask-height"></a>

<a id="ref-for-element-attrdef-mask-maskunits"></a>

<a id="ref-for-mask-painting-area③"></a>

The [mask-clip](#propdef-mask-clip) property has no affect on a [mask layer image](#mask-layer-image) that references a [mask](#elementdef-mask) element. The [x](#element-attrdef-mask-x), [y](#element-attrdef-mask-y), [width](#element-attrdef-mask-width), [height](#element-attrdef-mask-height) and [maskUnits](#element-attrdef-mask-maskunits) attributes on the <a id="ref-for-elementdef-mask①⓪"></a>mask element determine the [mask painting area](#mask-painting-area) for mask references.

Values have the following meanings:

<a id="valdef-mask-clip-content-box"></a>content-box

The painted content is restricted to (clipped to) the <em>content box</em>.

<a id="valdef-mask-clip-padding-box"></a>padding-box

The painted content is restricted to (clipped to) the <em>padding box</em>.

<a id="valdef-mask-clip-border-box"></a>border-box

The painted content is restricted to (clipped to) the <em>border box</em>.

<a id="valdef-mask-clip-fill-box"></a>fill-box

<a id="ref-for-TermObjectBoundingBox②"></a>

The painted content is restricted to (clipped to) the [object bounding box](https://www.w3.org/TR/SVG2/coords.html#TermObjectBoundingBox).

<a id="valdef-mask-clip-stroke-box"></a>stroke-box

<a id="ref-for-stroke-bounding-box①"></a>

The painted content is restricted to (clipped to) the [stroke bounding box](#stroke-bounding-box).

<a id="valdef-mask-clip-view-box"></a>view-box

Uses the nearest [SVG viewport](https://www.w3.org/TR/SVG11/intro.html#TermSVGViewport) as reference box.

<a id="ref-for-ViewBoxAttribute③"></a>

If a [viewBox](https://svgwg.org/svg2-draft/coords.html#ViewBoxAttribute) attribute is specified for the [SVG viewport](https://www.w3.org/TR/SVG11/intro.html#TermSVGViewport) creating element:

- <a id="ref-for-ViewBoxAttribute④"></a>

  The reference box is positioned at the origin of the coordinate system established by the [viewBox](https://svgwg.org/svg2-draft/coords.html#ViewBoxAttribute) attribute.

- <a id="ref-for-ViewBoxAttribute⑤"></a>

  The dimension of the reference box is set to the <em>width</em> and <em>height</em> values of the [viewBox](https://svgwg.org/svg2-draft/coords.html#ViewBoxAttribute) attribute.

<a id="valdef-mask-clip-no-clip"></a>no-clip

The painted content is not restricted (not clipped).

<a id="ref-for-used-value②"></a>

<a id="ref-for-valdef-mask-clip-content-box②"></a>

<a id="ref-for-valdef-mask-clip-padding-box①"></a>

<a id="ref-for-valdef-mask-clip-fill-box"></a>

<a id="ref-for-valdef-mask-clip-border-box③"></a>

<a id="ref-for-valuedef-margin-box0①"></a>

<a id="ref-for-valdef-mask-clip-stroke-box"></a>

For SVG elements without associated CSS layout box, the [used value](https://www.w3.org/TR/css-cascade-5/#used-value)s for [content-box](#valdef-mask-clip-content-box) and [padding-box](#valdef-mask-clip-padding-box) compute to [fill-box](#valdef-mask-clip-fill-box) and for [border-box](#valdef-mask-clip-border-box) and [margin-box](https://www.w3.org/TR/css-masking-1/#valuedef-margin-box0) compute to [stroke-box](#valdef-mask-clip-stroke-box).

<a id="ref-for-used-value③"></a>

<a id="ref-for-valdef-mask-clip-fill-box①"></a>

<a id="ref-for-valdef-mask-clip-content-box③"></a>

<a id="ref-for-valdef-mask-clip-stroke-box①"></a>

<a id="ref-for-valdef-mask-clip-view-box"></a>

<a id="ref-for-valdef-mask-clip-border-box④"></a>

For elements with associated CSS layout box, the [used value](https://www.w3.org/TR/css-cascade-5/#used-value)s for [fill-box](#valdef-mask-clip-fill-box) compute to [content-box](#valdef-mask-clip-content-box) and for [stroke-box](#valdef-mask-clip-stroke-box) and [view-box](#valdef-mask-clip-view-box) compute to [border-box](#valdef-mask-clip-border-box).

<a id="ref-for-propdef-mask-clip③"></a>

See the section [“Layering multiple mask layer images”](#layering) for how [mask-clip](#propdef-mask-clip) interacts with other comma-separated mask properties to form each mask layer.

<a id="ref-for-propdef-mask-origin"></a>

### <a id="the-mask-origin"></a>7.6. Positioning Area: the [mask-origin](#propdef-mask-origin) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-origin"></a>mask-origin                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma⑤"></a><a id="ref-for-typedef-geometry-box③"></a>[\<geometry-box\>](#typedef-geometry-box)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | border-box                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use①⓪"></a><a id="ref-for-graphics-element①①"></a><a id="ref-for-elementdef-defs⑥"></a><a id="ref-for-container-element⑦"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

<a id="ref-for-propdef-box-decoration-break"></a>

For elements rendered as a single box, specifies the <a id="mask-positioning-area"></a>mask positioning area. For elements rendered as multiple boxes (e.g., inline boxes on several lines, boxes on several pages) specifies which boxes [box-decoration-break](https://www.w3.org/TR/css-break-4/#propdef-box-decoration-break) operates on to determine the mask positioning area.

<a id="valdef-mask-origin-content-box"></a>content-box  
The position is relative to the <em>content box</em>.

<a id="valdef-mask-origin-padding-box"></a>padding-box  
The position is relative to the <em>padding box</em>. (For single boxes 0 0 is the upper left corner of the padding edge, 100% 100% is the lower right corner.)

<a id="valdef-mask-origin-border-box"></a>border-box  
The position is relative to the <em>border box</em>.

<a id="valdef-mask-origin-fill-box"></a>fill-box  
<a id="ref-for-TermObjectBoundingBox③"></a>

The position is relative to the [object bounding box](https://www.w3.org/TR/SVG2/coords.html#TermObjectBoundingBox).

<a id="valdef-mask-origin-stroke-box"></a>stroke-box  
<a id="ref-for-stroke-bounding-box②"></a>

The position is relative to the [stroke bounding box](#stroke-bounding-box).

<a id="valdef-mask-origin-view-box"></a>view-box  
Uses the nearest [SVG viewport](https://www.w3.org/TR/SVG11/intro.html#TermSVGViewport) as reference box.

<a id="ref-for-ViewBoxAttribute⑥"></a>

If a [viewBox](https://svgwg.org/svg2-draft/coords.html#ViewBoxAttribute) attribute is specified for the [SVG viewport](https://www.w3.org/TR/SVG11/intro.html#TermSVGViewport) creating element:

- <a id="ref-for-ViewBoxAttribute⑦"></a>

  The reference box is positioned at the origin of the coordinate system established by the [viewBox](https://svgwg.org/svg2-draft/coords.html#ViewBoxAttribute) attribute.

- <a id="ref-for-ViewBoxAttribute⑧"></a>

  The dimension of the reference box is set to the <em>width</em> and <em>height</em> values of the [viewBox](https://svgwg.org/svg2-draft/coords.html#ViewBoxAttribute) attribute.

<a id="ref-for-valdef-mask-origin-content-box"></a>

<a id="ref-for-valdef-mask-origin-padding-box"></a>

<a id="ref-for-valdef-mask-origin-border-box"></a>

<a id="ref-for-valdef-mask-origin-fill-box"></a>

For SVG elements without associated CSS layout box, the values [content-box](#valdef-mask-origin-content-box), [padding-box](#valdef-mask-origin-padding-box) and [border-box](#valdef-mask-origin-border-box) compute to [fill-box](#valdef-mask-origin-fill-box).

<a id="ref-for-valdef-mask-origin-fill-box①"></a>

<a id="ref-for-valdef-mask-origin-stroke-box"></a>

<a id="ref-for-valdef-mask-origin-view-box"></a>

<a id="ref-for-propdef-mask-origin①"></a>

For elements with associated CSS layout box, the values [fill-box](#valdef-mask-origin-fill-box), [stroke-box](#valdef-mask-origin-stroke-box) and [view-box](#valdef-mask-origin-view-box) compute to the initial value of [mask-origin](#propdef-mask-origin).

<a id="ref-for-propdef-mask-origin②"></a>

<a id="ref-for-propdef-background-origin"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [mask-origin](#propdef-mask-origin) property is similar to the [background-origin](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-origin) property [\[CSS3BG\]](#biblio-css3bg), but it has a different set of values, and a different initial value.

<a id="ref-for-propdef-mask-clip④"></a>

<a id="ref-for-valdef-mask-origin-padding-box①"></a>

<a id="ref-for-propdef-mask-origin③"></a>

<a id="ref-for-valdef-mask-origin-border-box①"></a>

<a id="ref-for-propdef-mask-position③"></a>

<a id="ref-for-mask-layer-image①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If [mask-clip](#propdef-mask-clip) is [padding-box](#valdef-mask-origin-padding-box), [mask-origin](#propdef-mask-origin) is [border-box](#valdef-mask-origin-border-box), [mask-position](#propdef-mask-position) is top left (the initial value), and the element has a non-zero border, then the top and left of the [mask layer image](#mask-layer-image) will be clipped.

<a id="ref-for-propdef-mask-origin④"></a>

See the section [“Layering multiple mask layer images”](#layering) for how [mask-origin](#propdef-mask-origin) interacts with other comma-separated mask properties to form each mask layer.

<a id="ref-for-propdef-mask-size①"></a>

### <a id="the-mask-size"></a>7.7. Sizing Mask Images: the [mask-size](#propdef-mask-size) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-size"></a>mask-size                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma⑥"></a><a id="ref-for-typedef-bg-size"></a>[\<bg-size\>](https://www.w3.org/TR/css-backgrounds-3/#typedef-bg-size)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use①①"></a><a id="ref-for-graphics-element①②"></a><a id="ref-for-elementdef-defs⑦"></a><a id="ref-for-container-element⑧"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified, but with lengths made absolute                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | repeatable list                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

<a id="ref-for-mask-layer-image①②"></a>

Specifies the size of the [mask layer images](#mask-layer-image).

<a id="ref-for-propdef-background-size"></a>

See [background-size](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-size) property [\[CSS3BG\]](#biblio-css3bg) for the definitions of the property values.

<a id="ref-for-propdef-mask-size②"></a>

See the section [“Layering multiple mask layer images”](#layering) for how [mask-size](#propdef-mask-size) interacts with other comma-separated mask properties to form each mask layer.

<a id="ref-for-propdef-mask-composite①"></a>

### <a id="the-mask-composite"></a>7.8. Compositing mask layers: the [mask-composite](#propdef-mask-composite) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                          |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-composite"></a>mask-composite                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma⑦"></a><a id="ref-for-typedef-compositing-operator"></a>[\<compositing-operator\>](#typedef-compositing-operator)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | add                                                                                                                                                                                                                                                                                                                                                 |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-graphics-element①③"></a><a id="ref-for-elementdef-defs⑧"></a><a id="ref-for-container-element⑨"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) without the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element and all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                                                                                                                                                        |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                            |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                              |

<a id="typedef-compositing-operator"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-comb-one①③"></a>

```text
<compositing-operator> = add | subtract | intersect | exclude
```
Each keyword represents a Porter-Duff compositing operator [\[COMPOSITING-1\]](#biblio-compositing-1) which defines the compositing operation used on the current mask layer with the mask layers below it.

In the following, the current mask layer is referred to <a id="source"></a>source, all mask layers below it (with the corresponding compositing operators applied) are referred to <a id="destination"></a>destination.

<a id="valdef-mask-composite-add"></a>add  
<a id="ref-for-source"></a>

<a id="ref-for-destination"></a>

The [source](#source) is placed over the [destination](#destination). (See Porter-Duff compositing operator [source over](https://www.w3.org/TR/compositing-1/#porterduffcompositingoperators_srcover) for more details.)

<a id="valdef-mask-composite-subtract"></a>subtract  
<a id="ref-for-source①"></a>

<a id="ref-for-destination①"></a>

The [source](#source) is placed, where it falls outside of the [destination](#destination). (See Porter-Duff compositing operator [source out](https://www.w3.org/TR/compositing-1/#porterduffcompositingoperators_srcout) for more details.)

<a id="valdef-mask-composite-intersect"></a>intersect  
<a id="ref-for-source②"></a>

<a id="ref-for-destination②"></a>

The parts of [source](#source) that overlap the [destination](#destination), replace the <a id="ref-for-destination③"></a>destination. (See Porter-Duff compositing operator [source in](https://www.w3.org/TR/compositing-1/#porterduffcompositingoperators_srcin) .)

<a id="valdef-mask-composite-exclude"></a>exclude  
<a id="ref-for-source③"></a>

<a id="ref-for-destination④"></a>

The non-overlapping regions of [source](#source) and [destination](#destination) are combined. (See Porter-Duff compositing operator [XOR](https://www.w3.org/TR/compositing-1/#porterduffcompositingoperators_xor).)

If there is no further mask layer, the compositing operator must be ignored. Mask layers must not composite with the element’s content or the content behind the element, instead they must act as if they are rendered into an isolated group.

All mask layers below the current mask layer must be composited before applying the compositing operation for the current mask layer.

<a id="ref-for-mask-layer-image①③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-065f6c44"></a> This example uses two [mask layer images](#mask-layer-image): <var>circle.svg</var> and <var>rect.svg</var>.
>
> ![Example of source-over compositing of mask layers](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/mask-source-destination.svg)
>
> <a id="ref-for-mask-layer-image①④"></a>
>
> <a id="ref-for-propdef-mask-image①⓪"></a>
>
> Both [mask layer images](#mask-layer-image) are references with the [mask-image](#propdef-mask-image) property:
>
> ```text
> mask-image: circle.svg, rect.svg;
> ```
>
> The mask layer with <var>rect.svg</var> is below the mask layer with <var>circle.svg</var>. That means <var>circle.svg</var> is closer to the user than <var>rect.svg</var>.
>
> <a id="ref-for-propdef-mask-composite②"></a>
>
> With the property [mask-composite](#propdef-mask-composite) the author may choose different ways to combine multiple mask layers.
>
> - <a id="ref-for-valdef-mask-composite-add"></a>
>
>   [add](#valdef-mask-composite-add) paints the <var>circle.svg</var> on top of <var>rect.svg</var>. The behavior is described by the compositing operator [source over](https://www.w3.org/TR/compositing-1/#porterduffcompositingoperators_srcover).
>
>   ```text
>   mask-composite: add;
>   ```
>
>   ![Example of source-over compositing of mask layers](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/mask-composite-add.svg)
>
> - <a id="ref-for-valdef-mask-composite-subtract"></a>
>
>   [subtract](#valdef-mask-composite-subtract) paints portions of <var>circle.svg</var> that do not overlap <var>rect.svg</var>. The behavior is described by the compositing operator [source out](https://www.w3.org/TR/compositing-1/#porterduffcompositingoperators_srcout).
>
>   ```text
>   mask-composite: subtract;
>   ```
>
>   ![Example of source-over compositing of mask layers](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/mask-composite-subtract.svg)
>
> - <a id="ref-for-valdef-mask-composite-intersect"></a>
>
>   [intersect](#valdef-mask-composite-intersect) paints portions of <var>circle.svg</var> that overlap <var>rect.svg</var>. The behavior is described by the compositing operator [source in](https://www.w3.org/TR/compositing-1/#porterduffcompositingoperators_srcin).
>
>   ```text
>   mask-composite: intersect;
>   ```
>
>   ![Example of source-over compositing of mask layers](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/mask-composite-intersect.svg)
>
> - <a id="ref-for-valdef-mask-composite-exclude"></a>
>
>   [exclude](#valdef-mask-composite-exclude) paints portions of <var>circle.svg</var> and <var>rect.svg</var> that do not overlap. The behavior is described by the compositing operator [XOR](https://www.w3.org/TR/compositing-1/#porterduffcompositingoperators_xor).
>
>   ```text
>   mask-composite: exclude;
>   ```
>
>   ![Example of source-over compositing of mask layers](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/mask-composite-exclude.svg)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9bff19b4"></a> The following example specifies two mask layers and two compositing operators.
>
> ```text
> mask-image: rect.svg, circle.svg;
> mask-composite: add, exclude;
> ```
>
> <a id="ref-for-valdef-mask-composite-add①"></a>
>
> <a id="ref-for-valdef-mask-composite-exclude①"></a>
>
> <var>rect.svg</var> and <var>circle.svg</var> make use of the [add](#valdef-mask-composite-add) compositing operator. There is no further mask layer to use [exclude](#valdef-mask-composite-exclude) and therefore, <a id="ref-for-valdef-mask-composite-exclude②"></a>exclude is ignored.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2266eac2"></a> This is an example of 3 mask layers with different compositing operators.
>
> ```text
> mask-image: trapeze.svg, circle.svg, rect.svg;
> mask-composite: subtract, add;
> ```
>
> First, <var>circle.svg</var> is “added” to <var>rect.svg</var>. In a second step, only portions of <var>trapeze.svg</var> that are not overlapping the compositing result of the previous two layers is visible.
>
> ![Example of source-over compositing of mask layers](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/mask-composite-subtract-add.svg)

<a id="ref-for-propdef-mask-composite③"></a>

See the section [“Layering multiple mask layer images”](#layering) for how [mask-composite](#propdef-mask-composite) interacts with other comma-separated mask properties to form each mask layer.

<a id="ref-for-propdef-mask②"></a>

### <a id="the-mask"></a>7.9. Mask Shorthand: the [mask](#propdef-mask) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask"></a>mask                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma⑧"></a><a id="ref-for-typedef-mask-layer"></a>[\<mask-layer\>](#typedef-mask-layer)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use①②"></a><a id="ref-for-graphics-element①④"></a><a id="ref-for-elementdef-defs⑨"></a><a id="ref-for-container-element①⓪"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

<a id="typedef-mask-layer"></a>

<a id="ref-for-typedef-mask-reference⑥"></a>

<a id="ref-for-comb-any①"></a>

<a id="ref-for-typedef-position①"></a>

<a id="ref-for-typedef-bg-size①"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-comb-any②"></a>

<a id="ref-for-typedef-repeat-style①"></a>

<a id="ref-for-comb-any③"></a>

<a id="ref-for-typedef-geometry-box④"></a>

<a id="ref-for-comb-any④"></a>

<a id="ref-for-typedef-geometry-box⑤"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-comb-any⑤"></a>

<a id="ref-for-typedef-compositing-operator①"></a>

<a id="ref-for-comb-any⑥"></a>

<a id="ref-for-typedef-masking-mode①"></a>

```text
<mask-layer> = <mask-reference> || <position> [ / <bg-size> ]? ||<repeat-style> || <geometry-box> || [ <geometry-box> | no-clip ] || <compositing-operator> || <masking-mode>
```
<a id="ref-for-typedef-geometry-box⑥"></a>

<a id="ref-for-valdef-mask-clip-no-clip"></a>

<a id="ref-for-propdef-mask-origin⑤"></a>

<a id="ref-for-propdef-mask-clip⑤"></a>

If one [\<geometry-box\>](#typedef-geometry-box) value and the [no-clip](#valdef-mask-clip-no-clip) keyword are present then <a id="ref-for-typedef-geometry-box⑦"></a>\<geometry-box\> sets [mask-origin](#propdef-mask-origin) and <a id="ref-for-valdef-mask-clip-no-clip①"></a>no-clip sets [mask-clip](#propdef-mask-clip) to that value.

<a id="ref-for-typedef-geometry-box⑧"></a>

<a id="ref-for-valdef-mask-clip-no-clip②"></a>

<a id="ref-for-propdef-mask-origin⑥"></a>

<a id="ref-for-propdef-mask-clip⑥"></a>

If one [\<geometry-box\>](#typedef-geometry-box) value and no [no-clip](#valdef-mask-clip-no-clip) keyword are present then <a id="ref-for-typedef-geometry-box⑨"></a>\<geometry-box\> sets both [mask-origin](#propdef-mask-origin) and [mask-clip](#propdef-mask-clip) to that value.

<a id="ref-for-typedef-geometry-box①⓪"></a>

<a id="ref-for-propdef-mask-origin⑦"></a>

<a id="ref-for-propdef-mask-clip⑦"></a>

If two [\<geometry-box\>](#typedef-geometry-box) values are present, then the first sets [mask-origin](#propdef-mask-origin) and the second [mask-clip](#propdef-mask-clip).

<a id="ref-for-used-value④"></a>

<a id="ref-for-propdef-mask-repeat②"></a>

<a id="ref-for-propdef-mask-position④"></a>

<a id="ref-for-propdef-mask-clip⑧"></a>

<a id="ref-for-propdef-mask-origin⑧"></a>

<a id="ref-for-propdef-mask-size③"></a>

<a id="ref-for-typedef-mask-reference⑦"></a>

<a id="ref-for-elementdef-mask①①"></a>

<a id="ref-for-mask-layer-image①⑤"></a>

The [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of the properties [mask-repeat](#propdef-mask-repeat), [mask-position](#propdef-mask-position), [mask-clip](#propdef-mask-clip), [mask-origin](#propdef-mask-origin) and [mask-size](#propdef-mask-size) must have no effect if [\<mask-reference\>](#typedef-mask-reference) references a [mask](#elementdef-mask) element. In this case the element defines position, sizing and clipping of the [mask layer image](#mask-layer-image).

<a id="ref-for-propdef-mask③"></a>

<a id="ref-for-propdef-mask-border②"></a>

The [mask](#propdef-mask) shorthand also resets [mask-border](#propdef-mask-border) to its initial value. It is therefore recommended that authors use the <a id="ref-for-propdef-mask④"></a>mask shorthand, rather than other shorthands or the individual properties, to override any mask settings earlier in the cascade. This will ensure that <a id="ref-for-propdef-mask-border③"></a>mask-border has also been reset to allow the new styles to take effect.

### <a id="the-mask-image-rendering-model"></a>7.10. The Mask Image Rendering Model

<a id="ref-for-propdef-mask-image①①"></a>

<a id="ref-for-TermStackingContext③"></a>

<a id="ref-for-propdef-opacity③"></a>

The application of the [mask-image](#propdef-mask-image) property with a value other than none to an element formatted with the CSS box model establishes a [stacking context](https://www.w3.org/TR/SVG2/render.html#TermStackingContext) in the same way that CSS [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) [\[CSS3COLOR\]](#biblio-css3color) does, and all the element’s descendants are rendered together as a group with the masking applied to the group as a whole.

<a id="ref-for-propdef-mask-image①②"></a>

The [mask-image](#propdef-mask-image) property has no effect on the geometry or hit-testing of any element’s CSS boxes.

<a id="ref-for-propdef-mask⑤"></a>

The [mask](#propdef-mask) property is a [presentation attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) for SVG elements.

#### <a id="MaskValues"></a>7.10.1. Mask processing

<a id="ref-for-mask-layer-image①⑥"></a>

<a id="ref-for-mask-border-image①"></a>

In the following section, <a id="mask-image"></a>mask image refers either to a [mask layer image](#mask-layer-image) or to a [mask border image](#mask-border-image).

A mask image may be interpreted using one of two different methods with regards to calculating the mask values that will be multiplied with the target alpha values.

<a id="ref-for-mask-image"></a>

The first and simplest method of calculating the mask values is to use the alpha channel of the [mask image](#mask-image). In this case the mask value at a given point is simply the value of the alpha channel at that point. The color channels do not contribute to the mask value.

The second method of calculating the mask values is to use the luminance of the mask image. In this case the mask value at a given point is computed from the color channel values and alpha channel value using the following procedure.

1.  Compute a luminance value from the color channel values.

    - <a id="ref-for-ColorInterpolationProperty①"></a>

      <a id="ref-for-elementdef-mask①②"></a>

      <a id="ref-for-valdef-color-interpolation-filters-linearrgb"></a>

      If the computed value of [color-interpolation](https://www.w3.org/TR/SVG2/painting.html#ColorInterpolationProperty) on the [mask](#elementdef-mask) element is [linearRGB](https://www.w3.org/TR/filter-effects-1/#valdef-color-interpolation-filters-linearrgb), convert the original image color values (potentially in the sRGB color space) to the linearRGB color space.

    - <a id="ref-for-elementdef-fecolormatrix"></a>

      Then, using non-premultiplied RGB color values, apply the luminance-to-alpha coefficients (as defined in the [feColorMatrix](https://www.w3.org/TR/filter-effects-1/#elementdef-fecolormatrix) filter primitive [\[SVG11\]](#biblio-svg11)) to convert the RGB color values to luminance values.

2.  Multiply the computed luminance value by the corresponding alpha value to produce the mask value.

Regardless of the method used, the procedure for calculating mask values assumes the content of the mask is a four-channel RGBA graphics object. For other types of graphics objects, special handling is required as follows.

For a three-channel RGB graphics object that is used in a mask (e.g., when referencing a three-channel image file), the effect is as if the object were converted into a four-channel RGBA image with the alpha channel uniformly set to 1.

For a single-channel image that is used in a mask (e.g., when referencing a single-channel grayscale image file), the effect is as if the object were converted into a four-channel RGBA image, where the single channel from the referenced object is used to compute the three color channels and the alpha channel is uniformly set to 1.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: When referencing a grayscale image file, the transfer curve relating the encoded grayscale values to linear light values must be taken into account when computing the color channels.

<a id="ref-for-graphics-element①⑤"></a>

<a id="ref-for-elementdef-circle②"></a>

<a id="ref-for-elementdef-text③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: SVG [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) (e.g., [circle](https://www.w3.org/TR/SVG2/shapes.html#elementdef-circle) or [text](https://www.w3.org/TR/SVG2/text.html#elementdef-text)) are all treated as four-channel RGBA images for the purposes of masking operations.

The effect of a mask is identical to what would have happened if there were no mask but instead the alpha channel of the given object were multiplied with the mask’s resulting mask values.

<a id="ref-for-mask-image①"></a>

Regions not covered by a [mask image](#mask-image) are treated as transparent black. The mask value is 0.

<a id="ref-for-mask-image②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Masks with repeating [mask image](#mask-image) tiles may have an offset to each other. The space between the <a id="ref-for-mask-image③"></a>mask images is treated as a transparent black mask.

#### <a id="layering"></a>7.10.2. Layering Multiple Mask Images

<a id="ref-for-propdef-mask-image①③"></a>

<a id="ref-for-typedef-mask-reference⑧"></a>

The mask of a box can have multiple layers. The number of layers is determined by the number of comma-separated values for the [mask-image](#propdef-mask-image) property. A value of none in a list of values with other [\<mask-reference\>](#typedef-mask-reference)s still creates a layer.

See [Layering Multiple Background Images](https://www.w3.org/TR/css3-background/#layering) [\[CSS3BG\]](#biblio-css3bg).

<a id="ref-for-propdef-mask-mode⑥"></a>

<a id="ref-for-propdef-mask-composite④"></a>

[mask-mode](#propdef-mask-mode) and [mask-composite](#propdef-mask-composite) do not have counterparts in CSS Backgrounds and Borders [\[CSS3BG\]](#biblio-css3bg). Just like for the mask properties that do have a counterpart, the list of values are matched up from the first value: excess values at the end are not used. If a property doesn’t have enough comma-separated values to match the number of layers, the UA must calculate its used value by repeating the list of values until there are enough.

<a id="ref-for-mask-layer-image①⑦"></a>

<a id="ref-for-propdef-mask-composite⑤"></a>

All [mask layer images](#mask-layer-image) are transformed to alpha masks (if necessary see [Mask processing](#MaskValues)) and combined by compositing taking the compositing operators specified by [mask-composite](#propdef-mask-composite) into account.

## <a id="mask-borders"></a>8. Border-Box Mask

<a id="ref-for-propdef-mask-border④"></a>

With [mask-border](#propdef-mask-border) an image can be split into nine pieces: four corners, four edges and the middle piece as demonstrated in the figure below.

![pieces of a mask border image](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/mask-box-image-mask.svg)

<a id="ref-for-mask-border-image②"></a>

Pieces of a [mask border image](#mask-border-image).

<a id="ref-for-mask-border-image-area②"></a>

<a id="ref-for-propdef-mask-border⑤"></a>

<a id="ref-for-propdef-border-image"></a>

These pieces may be sliced, scaled and stretched in various ways to fit the size of the [mask border image area](#mask-border-image-area). This distorted image is then used as a mask. The syntax of [mask-border](#propdef-mask-border) corresponds to the [border-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image) property of CSS Background and Borders [\[CSS3BG\]](#biblio-css3bg).

<a id="ref-for-mask-border-image③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f8023813"></a> The [mask border image](#mask-border-image) in the following example is split into four corners with dimensions of 75 pixels, four edges and the middle piece that is stretched and scaled.
>
> ![Example for 'mask-border'](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/mask-box-image.svg)
>
> <a id="ref-for-propdef-mask-border⑥"></a>
>
> Example for [mask-border](#propdef-mask-border). The object on the left is the object to mask. The second image is the alpha mask and the last image the masked object.
>
> ```text
> div {
>     background: linear-gradient(bottom, #F27BAA 0%, #FCC8AD 100%);
>     mask-border-slice: 25 fill;
>     mask-border-repeat: stretch;
>     mask-border-source: url(mask.png);
> }
> ```
<a id="ref-for-propdef-mask-border-source⑥"></a>

### <a id="the-mask-border-source"></a>8.1. Mask Border Image Source: the [mask-border-source](#propdef-mask-border-source) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-border-source"></a>mask-border-source                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-image④"></a><a id="ref-for-comb-one①⑤"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image)                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use①③"></a><a id="ref-for-graphics-element①⑥"></a><a id="ref-for-elementdef-defs①⓪"></a><a id="ref-for-container-element①①"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-image⑤"></a>they keyword none or the computed [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image)                                                                                                                                                                                                                                                                                                                                |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

Specifies an image to be used as <a id="mask-border-image"></a>mask border image.

<a id="ref-for-mask-border-image④"></a>

An image that is an empty image (zero width or zero height), that fails to download, is non-existent, or that cannot be displayed (e.g. because it is not in a supported image format) is ignored. It still counts as an [mask border image](#mask-border-image) but does not mask the element.

<a id="ref-for-mask-border-image⑤"></a>

See “[Mask processing](#MaskValues)” on how to process the [mask border image](#mask-border-image).

<a id="ref-for-TermStackingContext④"></a>

<a id="ref-for-propdef-opacity④"></a>

A computed value of other than none results in the creation of a [stacking context](https://www.w3.org/TR/SVG2/render.html#TermStackingContext) [\[CSS21\]](#biblio-css21) the same way that CSS [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) [\[CSS3COLOR\]](#biblio-css3color) does for values other than 1.

<a id="ref-for-propdef-mask-border-source⑦"></a>

<a id="ref-for-propdef-mask-image①④"></a>

[mask-border-source](#propdef-mask-border-source) and [mask-image](#propdef-mask-image) can be specified independent of each other. If both properties have a value other than none, the element is masked by both masking operations one after the other.

<a id="ref-for-propdef-mask-image①⑤"></a>

<a id="ref-for-propdef-mask-border-source⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It does not matter if [mask-image](#propdef-mask-image) is applied to the element before or after [mask-border-source](#propdef-mask-border-source). Both operation orders result in the same rendering.

<a id="ref-for-propdef-mask-border-mode"></a>

### <a id="the-mask-border-mode"></a>8.2. Mask Border Image Interpretation: the [mask-border-mode](#propdef-mask-border-mode) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-border-mode"></a>mask-border-mode                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑥"></a>luminance [\|](https://www.w3.org/TR/css-values-4/#comb-one) alpha                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | alpha                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use①④"></a><a id="ref-for-graphics-element①⑦"></a><a id="ref-for-elementdef-defs①①"></a><a id="ref-for-container-element①②"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

<a id="ref-for-propdef-mask-border-mode①"></a>

<a id="ref-for-typedef-image⑥"></a>

<a id="ref-for-propdef-mask-border-source⑨"></a>

The [mask-border-mode](#propdef-mask-border-mode) property indicates whether the [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) value for [mask-border-source](#propdef-mask-border-source) is treated as luminance mask or alpha mask. (See [Mask processing](#MaskValues).)

Values have the following meanings:

<a id="valdef-mask-border-mode-alpha"></a>alpha  
<a id="ref-for-valdef-mask-border-mode-alpha"></a>

<a id="ref-for-mask-border-image⑥"></a>

A value of [alpha](#valdef-mask-border-mode-alpha) indicates that the alpha values of the [mask border image](#mask-border-image) should be used as the mask values. See [Calculating mask values](#MaskValues).

<a id="valdef-mask-border-mode-luminance"></a>luminance  
<a id="ref-for-valdef-mask-border-mode-luminance"></a>

<a id="ref-for-mask-border-image⑦"></a>

A value of [luminance](#valdef-mask-border-mode-luminance) indicates that the luminance values of the [mask border image](#mask-border-image) should be used as the mask values. See [Calculating mask values](#MaskValues).

<a id="ref-for-propdef-mask-mode⑦"></a>

<a id="ref-for-propdef-mask-type③"></a>

<a id="ref-for-mask-border-image⑧"></a>

The [mask-mode](#propdef-mask-mode) and [mask-type](#propdef-mask-type) properties must have no affect on the [mask border image](#mask-border-image) type.

<a id="ref-for-propdef-mask-border-slice"></a>

### <a id="the-mask-border-slice"></a>8.3. Mask Border Image Slicing: the [mask-border-slice](#propdef-mask-border-slice) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-border-slice"></a>mask-border-slice                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt①"></a><a id="ref-for-mult-num-range"></a><a id="ref-for-typedef-number-percentage"></a>[\<number-percentage\>](https://www.w3.org/TR/css-values-3/#typedef-number-percentage)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) fill[?](https://www.w3.org/TR/css-values-4/#mult-opt)                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use①⑤"></a><a id="ref-for-graphics-element①⑧"></a><a id="ref-for-elementdef-defs①②"></a><a id="ref-for-container-element①③"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-mask-border-image⑨"></a>refer to size of the [mask border image](#mask-border-image)                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

<a id="ref-for-mask-border-image①⓪"></a>

This property specifies inward offsets from the top, right, bottom, and left edges of the [mask border image](#mask-border-image), dividing it into nine regions: four corners, four edges and a middle. The middle image part is discarded and treated as fully opaque white (the content covered by the middle part is not masked and shines through) unless the <a id="valdef-mask-border-slice-fill"></a>fill keyword is present.

<a id="ref-for-propdef-border-image-slice"></a>

See the [border-image-slice](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-slice) property [\[CSS3BG\]](#biblio-css3bg) for the definitions of the property values.

<a id="ref-for-propdef-mask-border-width"></a>

### <a id="the-mask-border-width"></a>8.4. Masking Areas: the [mask-border-width](#propdef-mask-border-width) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-border-width"></a>mask-border-width                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range①"></a><a id="ref-for-number-value"></a><a id="ref-for-comb-one①⑦"></a><a id="ref-for-typedef-length-percentage"></a>\[ [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) <a id="ref-for-comb-one①⑧"></a>\| auto \][{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range)                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use①⑥"></a><a id="ref-for-graphics-element①⑨"></a><a id="ref-for-elementdef-defs①③"></a><a id="ref-for-container-element①④"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-mask-border-image-area③"></a>relative to width/height of the [mask border image area](#mask-border-image-area)                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-length-value①"></a>all [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)s made absolute, otherwise as specified                                                                                                                                                                                                                                                                                                                       |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

<a id="ref-for-mask-border-image①①"></a>

<a id="ref-for-propdef-mask-border-outset"></a>

The [mask border image](#mask-border-image) is drawn inside an area called the <a id="mask-border-image-area"></a>mask border image area. This is an area whose boundaries by default correspond to the border box, see [mask-border-outset](#propdef-mask-border-outset).

<a id="ref-for-propdef-border-image-width"></a>

See the [border-image-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-width) property [\[CSS3BG\]](#biblio-css3bg) for the definitions of the property values.

<a id="ref-for-propdef-border-width"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For SVG elements without an associated layout box the [border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width) is considered to be 0.

<a id="ref-for-propdef-mask-border-outset①"></a>

### <a id="the-mask-border-outset"></a>8.5. Edge Overhang: the [mask-border-outset](#propdef-mask-border-outset) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-border-outset"></a>mask-border-outset                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range②"></a><a id="ref-for-number-value①"></a><a id="ref-for-comb-one①⑨"></a><a id="ref-for-length-value②"></a>\[ [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) \][{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range)                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use①⑦"></a><a id="ref-for-graphics-element②⓪"></a><a id="ref-for-elementdef-defs①④"></a><a id="ref-for-container-element①⑤"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-length-value③"></a>all [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)s made absolute, otherwise as specified                                                                                                                                                                                                                                                                                                                       |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

<a id="ref-for-mask-border-image-area④"></a>

The values specify the amount by which the [mask border image area](#mask-border-image-area) extends beyond the border box. If it has four values, they set the outsets on the top, right, bottom and left sides in that order. If the left is missing, it is the same as the right; if the bottom is missing, it is the same as the top; if the right is missing, it is the same as the top.

<a id="ref-for-propdef-mask-border-width①"></a>

<a id="ref-for-number-value②"></a>

<a id="ref-for-propdef-border-width①"></a>

<a id="ref-for-propdef-mask-border-outset②"></a>

As with [mask-border-width](#propdef-mask-border-width), a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) represents a multiple of the corresponding [border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width). Negative values are not allowed for any of the [mask-border-outset](#propdef-mask-border-outset) values.

<a id="ref-for-propdef-border-width②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For SVG elements without associated layout box the [border-width](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-width) is considered to be 0.

<a id="ref-for-propdef-mask-border-repeat"></a>

### <a id="the-mask-border-repeat"></a>8.6. Mask Border Image Tiling: the [mask-border-repeat](#propdef-mask-border-repeat) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-border-repeat"></a>mask-border-repeat                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range③"></a><a id="ref-for-comb-one②⓪"></a>\[ stretch [\|](https://www.w3.org/TR/css-values-4/#comb-one) repeat <a id="ref-for-comb-one②①"></a>\| round <a id="ref-for-comb-one②②"></a>\| space \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range)                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | stretch                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-use①⑧"></a><a id="ref-for-graphics-element②①"></a><a id="ref-for-elementdef-defs①⑤"></a><a id="ref-for-container-element①⑥"></a>All elements. In SVG, it applies to [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element) excluding the [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs) element, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element) and the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                             |

<a id="ref-for-mask-border-image①②"></a>

This property specifies how the images for the sides and the middle part of the [mask border image](#mask-border-image) are scaled and tiled. The first keyword applies to the horizontal sides, the second to the vertical ones. If the second keyword is absent, it is assumed to be the same as the first.

<a id="ref-for-propdef-border-image-repeat"></a>

See the [border-image-repeat](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-repeat) property [\[CSS3BG\]](#biblio-css3bg) for the definitions of the property values.

<a id="ref-for-mask-border-image①③"></a>

The exact process for scaling and tiling the [mask border image](#mask-border-image) parts is given in the section [Masking with the mask border image](#masking-with-the-mask-border-image)

<a id="ref-for-propdef-mask-border⑦"></a>

### <a id="the-mask-border"></a>8.7. Mask Border Image Shorthand: the [mask-border](#propdef-mask-border) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-border"></a>mask-border                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-mask-border-mode②"></a><a id="ref-for-propdef-mask-border-repeat①"></a><a id="ref-for-propdef-mask-border-outset③"></a><a id="ref-for-mult-opt②"></a><a id="ref-for-propdef-mask-border-width②"></a><a id="ref-for-propdef-mask-border-slice①"></a><a id="ref-for-comb-any⑦"></a><a id="ref-for-propdef-mask-border-source①⓪"></a>[\<'mask-border-source'\>](#propdef-mask-border-source) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'mask-border-slice'\>](#propdef-mask-border-slice) \[ / [\<'mask-border-width'\>](#propdef-mask-border-width)[?](https://www.w3.org/TR/css-values-4/#mult-opt) \[ / [\<'mask-border-outset'\>](#propdef-mask-border-outset) \]<a id="ref-for-mult-opt③"></a>? \]<a id="ref-for-mult-opt④"></a>? <a id="ref-for-comb-any⑧"></a>\|\| [\<'mask-border-repeat'\>](#propdef-mask-border-repeat) <a id="ref-for-comb-any⑨"></a>\|\| [\<'mask-border-mode'\>](#propdef-mask-border-mode) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong>Applies to:&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |

<a id="ref-for-propdef-mask-border-source①①"></a>

<a id="ref-for-propdef-mask-border-slice②"></a>

<a id="ref-for-propdef-mask-border-width③"></a>

<a id="ref-for-propdef-mask-border-outset④"></a>

<a id="ref-for-propdef-mask-border-repeat②"></a>

<a id="ref-for-propdef-mask-border-mode③"></a>

This is a shorthand property for setting [mask-border-source](#propdef-mask-border-source), [mask-border-slice](#propdef-mask-border-slice), [mask-border-width](#propdef-mask-border-width), [mask-border-outset](#propdef-mask-border-outset), [mask-border-repeat](#propdef-mask-border-repeat) and [mask-border-mode](#propdef-mask-border-mode). Omitted values are set to their initial values.

<a id="ref-for-propdef-mask⑥"></a>

<a id="ref-for-propdef-mask-border⑧"></a>

<a id="ref-for-propdef-mask-border-source①②"></a>

<a id="ref-for-propdef-mask-border-slice③"></a>

<a id="ref-for-propdef-mask-border-width④"></a>

<a id="ref-for-propdef-mask-border-outset⑤"></a>

<a id="ref-for-propdef-mask-border-repeat③"></a>

<a id="ref-for-propdef-mask-border-mode④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [mask](#propdef-mask) shorthand resets the properties [mask-border](#propdef-mask-border), [mask-border-source](#propdef-mask-border-source), [mask-border-slice](#propdef-mask-border-slice), [mask-border-width](#propdef-mask-border-width), [mask-border-outset](#propdef-mask-border-outset), [mask-border-repeat](#propdef-mask-border-repeat) and [mask-border-mode](#propdef-mask-border-mode).

### <a id="masking-with-the-mask-border-image"></a>8.8. Masking with the mask border image

<a id="ref-for-mask-border-image①④"></a>

<a id="ref-for-propdef-mask-border-source①③"></a>

<a id="ref-for-propdef-mask-border-slice④"></a>

After the [mask border image](#mask-border-image) given by [mask-border-source](#propdef-mask-border-source) is sliced by the [mask-border-slice](#propdef-mask-border-slice) values, the resulting nine images are scaled, positioned, and tiled into their corresponding <a id="ref-for-mask-border-image①⑤"></a>mask border image regions in four steps as described in the section [Drawing the Border Image](https://www.w3.org/TR/css3-background/#border-image-process) [\[CSS3BG\]](#biblio-css3bg).

<a id="ref-for-propdef-mask-border-source①④"></a>

<a id="ref-for-TermStackingContext⑤"></a>

<a id="ref-for-propdef-opacity⑤"></a>

The application of the [mask-border-source](#propdef-mask-border-source) property to an element formatted with the CSS box model establishes a [stacking context](https://www.w3.org/TR/SVG2/render.html#TermStackingContext) in the same way that CSS [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity) [\[CSS3COLOR\]](#biblio-css3color) does, and all the element’s descendants are rendered together as a group with the masking applied to the group as a whole.

<a id="ref-for-propdef-mask-border-source①⑤"></a>

The [mask-border-source](#propdef-mask-border-source) property has no effect on the geometry or hit-testing of any element’s CSS boxes.

## <a id="svg-masks"></a>9. SVG Mask Sources

<a id="ref-for-elementdef-mask①③"></a>

### <a id="MaskElement"></a>9.1. The [mask](#elementdef-mask) element

**Name:**

<a id="elementdef-mask"></a>`mask`

**Categories:**

<a id="ref-for-TermNeverRenderedElement①"></a><a id="ref-for-container-element①⑦"></a>[container elements](https://www.w3.org/TR/SVG2/struct.html#container-element), [never-rendered element](https://svgwg.org/svg2-draft/render.html#TermNeverRenderedElement)

**Content model:**

Any number of the following elements, in any order:

- <a id="ref-for-SetElement①"></a><a id="ref-for-AnimateTransformElement①"></a><a id="ref-for-AnimateMotionElement①"></a><a id="ref-for-AnimateColorElement①"></a><a id="ref-for-AnimateElement①"></a>[animation](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermAnimationElement) — [animate](https://www.w3.org/TR/SVG11/animate.html#AnimateElement), [animateColor](https://www.w3.org/TR/SVG11/animate.html#AnimateColorElement), [animateMotion](https://www.w3.org/TR/SVG11/animate.html#AnimateMotionElement), [animateTransform](https://www.w3.org/TR/SVG11/animate.html#AnimateTransformElement), [set](https://www.w3.org/TR/SVG11/animate.html#SetElement)

- <a id="ref-for-elementdef-metadata①"></a><a id="ref-for-elementdef-title①"></a><a id="ref-for-elementdef-desc①"></a>[descriptive](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermDescriptiveElement) — [desc](https://www.w3.org/TR/SVG2/struct.html#elementdef-desc), [title](https://www.w3.org/TR/SVG2/struct.html#elementdef-title), [metadata](https://www.w3.org/TR/SVG2/struct.html#elementdef-metadata)

- <a id="ref-for-elementdef-rect①"></a><a id="ref-for-elementdef-polyline①"></a><a id="ref-for-elementdef-polygon①"></a><a id="ref-for-elementdef-path④"></a><a id="ref-for-elementdef-line①"></a><a id="ref-for-elementdef-ellipse①"></a><a id="ref-for-elementdef-circle③"></a>[shape](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermShapeElement) — [circle](https://www.w3.org/TR/SVG2/shapes.html#elementdef-circle), [ellipse](https://www.w3.org/TR/SVG2/shapes.html#elementdef-ellipse), [line](https://www.w3.org/TR/SVG2/shapes.html#elementdef-line), [path](https://www.w3.org/TR/SVG2/paths.html#elementdef-path), [polygon](https://www.w3.org/TR/SVG2/shapes.html#elementdef-polygon), [polyline](https://www.w3.org/TR/SVG2/shapes.html#elementdef-polyline), [rect](https://www.w3.org/TR/SVG2/shapes.html#elementdef-rect)

- <a id="ref-for-elementdef-use①⑨"></a><a id="ref-for-elementdef-symbol"></a><a id="ref-for-elementdef-svg"></a><a id="ref-for-elementdef-g"></a><a id="ref-for-elementdef-defs①⑥"></a>[structural](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermStructuralElement) — [defs](https://www.w3.org/TR/SVG2/struct.html#elementdef-defs), [g](https://www.w3.org/TR/SVG2/struct.html#elementdef-g), [svg](https://www.w3.org/TR/SVG2/struct.html#elementdef-svg), [symbol](https://www.w3.org/TR/SVG2/struct.html#elementdef-symbol), [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use)

- <a id="ref-for-elementdef-radialGradient"></a><a id="ref-for-elementdef-linearGradient"></a>[gradient](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermGradientElement) — [linearGradient](https://www.w3.org/TR/SVG2/pservers.html#elementdef-linearGradient), [radialGradient](https://www.w3.org/TR/SVG2/pservers.html#elementdef-radialGradient)

- <a id="ref-for-elementdef-a"></a>[a](https://www.w3.org/TR/SVG2/linking.html#elementdef-a)

- <a id="ref-for-elementdef-clippath③⓪"></a>[clipPath](#elementdef-clippath)

- <a id="ref-for-ColorProfileElement"></a>[color-profile](https://www.w3.org/TR/SVG11/color.html#ColorProfileElement)

- <a id="ref-for-CursorElement"></a>[cursor](https://www.w3.org/TR/SVG11/interact.html#CursorElement)

- <a id="ref-for-elementdef-filter"></a>[filter](https://www.w3.org/TR/filter-effects-1/#elementdef-filter)

- <a id="ref-for-FontElement"></a>[font](https://www.w3.org/TR/SVG11/fonts.html#FontElement)

- <a id="ref-for-FontFaceElement"></a>[font-face](https://www.w3.org/TR/SVG11/fonts.html#FontFaceElement)

- <a id="ref-for-elementdef-foreignObject"></a>[foreignObject](https://www.w3.org/TR/SVG2/embedded.html#elementdef-foreignObject)

- <a id="ref-for-elementdef-image"></a>[image](https://www.w3.org/TR/SVG2/embedded.html#elementdef-image)

- <a id="ref-for-elementdef-marker"></a>[marker](https://www.w3.org/TR/SVG2/painting.html#elementdef-marker)

- <a id="ref-for-elementdef-mask①④"></a>[mask](#elementdef-mask)

- <a id="ref-for-elementdef-pattern"></a>[pattern](https://www.w3.org/TR/SVG2/pservers.html#elementdef-pattern)

- <a id="ref-for-elementdef-script①"></a>[script](https://www.w3.org/TR/SVG2/interact.html#elementdef-script)

- <a id="ref-for-elementdef-style"></a>[style](https://www.w3.org/TR/SVG2/styling.html#elementdef-style)

- <a id="ref-for-elementdef-switch"></a>[switch](https://www.w3.org/TR/SVG2/struct.html#elementdef-switch)

- <a id="ref-for-elementdef-view"></a>[view](https://www.w3.org/TR/SVG2/linking.html#elementdef-view)

- <a id="ref-for-elementdef-text④"></a>[text](https://www.w3.org/TR/SVG2/text.html#elementdef-text)

- <a id="ref-for-AlternateGlyphDefinitions"></a>[altGlyphDef](https://www.w3.org/TR/SVG11/text.html#AlternateGlyphDefinitions)

**Attributes:**

- [conditional processing attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermConditionalProcessingAttribute) — [‘requiredFeatures’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#RequiredFeaturesAttribute), [‘requiredExtensions’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#RequiredExtensionsAttribute), [‘systemLanguage’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#SystemLanguageAttribute)

- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [‘id’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [‘xml:base’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [‘xml:lang’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [‘xml:space’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)

- <a id="ref-for-propdef-writing-mode①"></a><a id="ref-for-propdef-word-spacing①"></a><a id="ref-for-propdef-visibility②"></a><a id="ref-for-propdef-unicode-bidi①"></a><a id="ref-for-TextRenderingProperty①"></a><a id="ref-for-propdef-text-decoration①"></a><a id="ref-for-TextAnchorProperty①"></a><a id="ref-for-StrokeWidthProperty②"></a><a id="ref-for-StrokeOpacityProperty①"></a><a id="ref-for-StrokeMiterlimitProperty①"></a><a id="ref-for-StrokeLinejoinProperty①"></a><a id="ref-for-StrokeLinecapProperty①"></a><a id="ref-for-StrokeDashoffsetProperty①"></a><a id="ref-for-StrokeDasharrayProperty①"></a><a id="ref-for-StrokeProperty②"></a><a id="ref-for-StopOpacityProperty①"></a><a id="ref-for-StopColorProperty①"></a><a id="ref-for-ShapeRenderingProperty①"></a><a id="ref-for-PointerEventsProperty②"></a><a id="ref-for-propdef-overflow②"></a><a id="ref-for-propdef-opacity⑥"></a><a id="ref-for-propdef-mask⑦"></a><a id="ref-for-MarkerStartProperty①"></a><a id="ref-for-MarkerMidProperty①"></a><a id="ref-for-MarkerEndProperty①"></a><a id="ref-for-MarkerProperty①"></a><a id="ref-for-propdef-lighting-color①"></a><a id="ref-for-propdef-letter-spacing①"></a><a id="ref-for-KerningProperty①"></a><a id="ref-for-propdef-image-rendering①"></a><a id="ref-for-propdef-glyph-orientation-vertical①"></a><a id="ref-for-GlyphOrientationHorizontalProperty①"></a><a id="ref-for-propdef-font-weight①"></a><a id="ref-for-propdef-font-variant①"></a><a id="ref-for-propdef-font-style①"></a><a id="ref-for-propdef-font-stretch①"></a><a id="ref-for-propdef-font-size-adjust①"></a><a id="ref-for-descdef-font-face-font-size①"></a><a id="ref-for-propdef-font-family①"></a><a id="ref-for-propdef-font①"></a><a id="ref-for-propdef-flood-opacity①"></a><a id="ref-for-propdef-flood-color①"></a><a id="ref-for-propdef-filter①"></a><a id="ref-for-FillRuleProperty④"></a><a id="ref-for-FillOpacityProperty①"></a><a id="ref-for-FillProperty②"></a><a id="ref-for-EnableBackgroundProperty①"></a><a id="ref-for-propdef-dominant-baseline①"></a><a id="ref-for-propdef-display⑤"></a><a id="ref-for-propdef-direction①"></a><a id="ref-for-propdef-cursor①"></a><a id="ref-for-ColorRenderingProperty①"></a><a id="ref-for-ColorProfileProperty①"></a><a id="ref-for-propdef-color-interpolation-filters①"></a><a id="ref-for-ColorInterpolationProperty②"></a><a id="ref-for-propdef-color①"></a><a id="ref-for-propdef-clip-rule⑨"></a><a id="ref-for-propdef-clip-path①⑤"></a><a id="ref-for-propdef-clip②"></a><a id="ref-for-propdef-baseline-shift①"></a><a id="ref-for-propdef-alignment-baseline①"></a>[presentation attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) — [alignment-baseline](https://www.w3.org/TR/css-inline-3/#propdef-alignment-baseline), [baseline-shift](https://www.w3.org/TR/css-inline-3/#propdef-baseline-shift), [clip](#propdef-clip), [clip-path](#propdef-clip-path), [clip-rule](#propdef-clip-rule), [color](https://www.w3.org/TR/css-color-4/#propdef-color), [color-interpolation](https://www.w3.org/TR/SVG2/painting.html#ColorInterpolationProperty), [color-interpolation-filters](https://www.w3.org/TR/filter-effects-1/#propdef-color-interpolation-filters), [color-profile](https://www.w3.org/TR/SVG11/color.html#ColorProfileProperty), [color-rendering](https://www.w3.org/TR/SVG2/painting.html#ColorRenderingProperty), [cursor](https://www.w3.org/TR/css-ui-3/#propdef-cursor), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), [display](https://www.w3.org/TR/css-display-3/#propdef-display), [dominant-baseline](https://www.w3.org/TR/css-inline-3/#propdef-dominant-baseline), [enable-background](https://www.w3.org/TR/SVG11/filters.html#EnableBackgroundProperty), [fill](https://www.w3.org/TR/SVG2/painting.html#FillProperty), [fill-opacity](https://www.w3.org/TR/SVG2/painting.html#FillOpacityProperty), [fill-rule](https://www.w3.org/TR/SVG2/painting.html#FillRuleProperty), [filter](https://www.w3.org/TR/filter-effects-1/#propdef-filter), [flood-color](https://www.w3.org/TR/filter-effects-1/#propdef-flood-color), [flood-opacity](https://www.w3.org/TR/filter-effects-1/#propdef-flood-opacity), [font](https://www.w3.org/TR/css-fonts-4/#propdef-font), [font-family](https://www.w3.org/TR/css-fonts-4/#propdef-font-family), [font-size](https://www.w3.org/TR/css-fonts-5/#descdef-font-face-font-size), [font-size-adjust](https://www.w3.org/TR/css-fonts-5/#propdef-font-size-adjust), [font-stretch](https://www.w3.org/TR/css-fonts-4/#propdef-font-stretch), [font-style](https://www.w3.org/TR/css-fonts-4/#propdef-font-style), [font-variant](https://www.w3.org/TR/css-fonts-4/#propdef-font-variant), [font-weight](https://www.w3.org/TR/css-fonts-4/#propdef-font-weight), [glyph-orientation-horizontal](https://www.w3.org/TR/SVG11/text.html#GlyphOrientationHorizontalProperty), [glyph-orientation-vertical](https://www.w3.org/TR/css-writing-modes-4/#propdef-glyph-orientation-vertical), [image-rendering](https://www.w3.org/TR/css-images-3/#propdef-image-rendering), [kerning](https://www.w3.org/TR/SVG11/text.html#KerningProperty), [letter-spacing](https://www.w3.org/TR/css-text-3/#propdef-letter-spacing), [lighting-color](https://www.w3.org/TR/filter-effects-1/#propdef-lighting-color), [marker](https://www.w3.org/TR/SVG2/painting.html#MarkerProperty), [marker-end](https://www.w3.org/TR/SVG2/painting.html#MarkerEndProperty), [marker-mid](https://www.w3.org/TR/SVG2/painting.html#MarkerMidProperty), [marker-start](https://www.w3.org/TR/SVG2/painting.html#MarkerStartProperty), [mask](#propdef-mask), [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow), [pointer-events](https://www.w3.org/TR/SVG2/interact.html#PointerEventsProperty), [shape-rendering](https://www.w3.org/TR/SVG2/painting.html#ShapeRenderingProperty), [stop-color](https://www.w3.org/TR/SVG2/pservers.html#StopColorProperty), [stop-opacity](https://www.w3.org/TR/SVG2/pservers.html#StopOpacityProperty), [stroke](https://www.w3.org/TR/SVG2/painting.html#StrokeProperty), [stroke-dasharray](https://www.w3.org/TR/SVG2/painting.html#StrokeDasharrayProperty), [stroke-dashoffset](https://www.w3.org/TR/SVG2/painting.html#StrokeDashoffsetProperty), [stroke-linecap](https://www.w3.org/TR/SVG2/painting.html#StrokeLinecapProperty), [stroke-linejoin](https://www.w3.org/TR/SVG2/painting.html#StrokeLinejoinProperty), [stroke-miterlimit](https://www.w3.org/TR/SVG2/painting.html#StrokeMiterlimitProperty), [stroke-opacity](https://www.w3.org/TR/SVG2/painting.html#StrokeOpacityProperty), [stroke-width](https://www.w3.org/TR/SVG2/painting.html#StrokeWidthProperty), [text-anchor](https://www.w3.org/TR/SVG2/text.html#TextAnchorProperty), [text-decoration](https://www.w3.org/TR/css-text-decor-3/#propdef-text-decoration), [text-rendering](https://www.w3.org/TR/SVG2/painting.html#TextRenderingProperty), [unicode-bidi](https://www.w3.org/TR/css-writing-modes-3/#propdef-unicode-bidi), [visibility](https://www.w3.org/TR/CSS2/visufx.html#propdef-visibility), [word-spacing](https://www.w3.org/TR/css-text-3/#propdef-word-spacing), [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode)

- [‘class’](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)

- [‘style’](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)

- <a id="ref-for-element-attrdef-mask-x①"></a>‘[x](#element-attrdef-mask-x)’

- <a id="ref-for-element-attrdef-mask-y①"></a>‘[y](#element-attrdef-mask-y)’

- <a id="ref-for-element-attrdef-mask-width①"></a>‘[width](#element-attrdef-mask-width)’

- <a id="ref-for-element-attrdef-mask-height①"></a>‘[height](#element-attrdef-mask-height)’

- <a id="ref-for-element-attrdef-mask-maskunits①"></a>‘[maskUnits](#element-attrdef-mask-maskunits)’

- <a id="ref-for-element-attrdef-mask-maskcontentunits"></a>‘[maskContentUnits](#element-attrdef-mask-maskcontentunits)’

**DOM Interfaces:**

[SVGMaskElement](#InterfaceSVGMaskElement)

<em>Attribute definitions:</em>

<a id="ref-for-valdef-maskunits-objectboundingbox"></a>

<a id="ref-for-valdef-maskunits-userspaceonuse"></a>

<a id="element-attrdef-mask-maskunits"></a>`maskUnits` = "[userSpaceOnUse](#valdef-maskunits-userspaceonuse) \| [objectBoundingBox](#valdef-maskunits-objectboundingbox)"

<a id="ref-for-element-attrdef-mask-x②"></a>

<a id="ref-for-element-attrdef-mask-y②"></a>

<a id="ref-for-element-attrdef-mask-width②"></a>

<a id="ref-for-element-attrdef-mask-height②"></a>

Defines the coordinate system for attributes [x](#element-attrdef-mask-x), [y](#element-attrdef-mask-y), [width](#element-attrdef-mask-width) and [height](#element-attrdef-mask-height).

<a id="valdef-maskunits-userspaceonuse"></a>userSpaceOnUse  
<a id="ref-for-element-attrdef-mask-x③"></a>

<a id="ref-for-element-attrdef-mask-y③"></a>

<a id="ref-for-element-attrdef-mask-width③"></a>

<a id="ref-for-element-attrdef-mask-height③"></a>

<a id="ref-for-user-coordinate-system②"></a>

<a id="ref-for-elementdef-mask①⑤"></a>

<a id="ref-for-propdef-mask⑧"></a>

[x](#element-attrdef-mask-x), [y](#element-attrdef-mask-y), [width](#element-attrdef-mask-width) and [height](#element-attrdef-mask-height) represent values in the current [user coordinate system](https://www.w3.org/TR/css-transforms-1/#user-coordinate-system) [\[CSS3-TRANSFORMS\]](#biblio-css3-transforms) in place at the time when the [mask](#elementdef-mask) element is referenced (i.e., the <a id="ref-for-user-coordinate-system③"></a>user coordinate system for the element referencing the <a id="ref-for-elementdef-mask①⑥"></a>mask element via the [mask](#propdef-mask) property).

<a id="valdef-maskunits-objectboundingbox"></a>objectBoundingBox  
<a id="ref-for-element-attrdef-mask-x④"></a>

<a id="ref-for-element-attrdef-mask-y④"></a>

<a id="ref-for-element-attrdef-mask-width④"></a>

<a id="ref-for-element-attrdef-mask-height④"></a>

<a id="ref-for-TermObjectBoundingBox④"></a>

<a id="ref-for-px①"></a>

[x](#element-attrdef-mask-x), [y](#element-attrdef-mask-y), [width](#element-attrdef-mask-width) and [height](#element-attrdef-mask-height) represent fractions or percentages of the [object bounding box](https://www.w3.org/TR/SVG2/coords.html#TermObjectBoundingBox) of the element to which the mask is applied. [User coordinates](https://www.w3.org/TR/SVG/coords.html#Units) are sized equivalently to the CSS [px](https://www.w3.org/TR/css-values-4/#px) unit.

<a id="ref-for-element-attrdef-mask-maskunits②"></a>

<a id="ref-for-valdef-maskunits-objectboundingbox①"></a>

If attribute [maskUnits](#element-attrdef-mask-maskunits) is not specified, then the effect is as if a value of [objectBoundingBox](#valdef-maskunits-objectboundingbox) were specified.

Animatable: yes.

<a id="ref-for-valdef-maskcontentunits-objectboundingbox"></a>

<a id="ref-for-valdef-maskcontentunits-userspaceonuse"></a>

<a id="element-attrdef-mask-maskcontentunits"></a>`maskContentUnits` = "[userSpaceOnUse](#valdef-maskcontentunits-userspaceonuse) \| [objectBoundingBox](#valdef-maskcontentunits-objectboundingbox)"

<a id="ref-for-elementdef-mask①⑦"></a>

Defines the coordinate system for the contents of the [mask](#elementdef-mask).

<a id="valdef-maskcontentunits-userspaceonuse"></a>userSpaceOnUse  
<a id="ref-for-user-coordinate-system④"></a>

<a id="ref-for-elementdef-mask①⑧"></a>

<a id="ref-for-propdef-mask⑨"></a>

The [user coordinate system](https://www.w3.org/TR/css-transforms-1/#user-coordinate-system) for the contents of the [mask](#elementdef-mask) element is the current <a id="ref-for-user-coordinate-system⑤"></a>user coordinate system in place at the time when the <a id="ref-for-elementdef-mask①⑨"></a>mask element is referenced (i.e., the <a id="ref-for-user-coordinate-system⑥"></a>user coordinate system for the element referencing the <a id="ref-for-elementdef-mask②⓪"></a>mask element via the [mask](#propdef-mask) property).

<a id="valdef-maskcontentunits-objectboundingbox"></a>objectBoundingBox  
<a id="ref-for-bounding-box②"></a>

<a id="ref-for-px②"></a>

The coordinate system has its origin at the top left corner of the [bounding box](https://www.w3.org/TR/SVG2/coords.html#bounding-box) of the element to which the clipping path applies to and the same width and height of this <a id="ref-for-bounding-box③"></a>bounding box. [User coordinates](https://www.w3.org/TR/SVG/coords.html#Units) are sized equivalently to the CSS [px](https://www.w3.org/TR/css-values-4/#px) unit.

<a id="ref-for-element-attrdef-mask-maskcontentunits①"></a>

<a id="ref-for-valdef-maskcontentunits-userspaceonuse①"></a>

If attribute [maskContentUnits](#element-attrdef-mask-maskcontentunits) is not specified, then the effect is as if a value of [userSpaceOnUse](#valdef-maskcontentunits-userspaceonuse) were specified.

Animatable: yes.

<a id="ref-for-typedef-length-percentage①"></a>

<a id="element-attrdef-mask-x"></a>`x` = "[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)"

<a id="ref-for-element-attrdef-mask-y⑤"></a>

<a id="ref-for-element-attrdef-mask-width⑤"></a>

<a id="ref-for-element-attrdef-mask-height⑤"></a>

The x-axis coordinate of one corner of the rectangle for the largest possible offscreen buffer. If the attribute is not specified but at least one of the attributes [y](#element-attrdef-mask-y), [width](#element-attrdef-mask-width) or [height](#element-attrdef-mask-height) are specified, the effect is as if a value of -10% were specified.

Animatable: yes.

<a id="ref-for-typedef-length-percentage②"></a>

<a id="element-attrdef-mask-y"></a>`y` = "[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)"

<a id="ref-for-element-attrdef-mask-x⑤"></a>

<a id="ref-for-element-attrdef-mask-width⑥"></a>

<a id="ref-for-element-attrdef-mask-height⑥"></a>

The y-axis coordinate of one corner of the rectangle for the largest possible offscreen buffer. If the attribute is not specified but at least one of the attributes [x](#element-attrdef-mask-x), [width](#element-attrdef-mask-width) or [height](#element-attrdef-mask-height) are specified, the effect is as if a value of -10% were specified.

Animatable: yes.

<a id="ref-for-typedef-length-percentage③"></a>

<a id="element-attrdef-mask-width"></a>`width` = "[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)"

<a id="ref-for-element-attrdef-mask-x⑥"></a>

<a id="ref-for-element-attrdef-mask-y⑥"></a>

<a id="ref-for-element-attrdef-mask-height⑦"></a>

The width of the largest possible offscreen buffer. A negative value or a value of zero disables rendering of the element. If the attribute is not specified but at least one of the attributes [x](#element-attrdef-mask-x), [y](#element-attrdef-mask-y) or [height](#element-attrdef-mask-height) are specified, the effect is as if a value of 120% were specified.

Animatable: yes.

<a id="ref-for-typedef-length-percentage④"></a>

<a id="element-attrdef-mask-height"></a>`height` = "[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)"

<a id="ref-for-element-attrdef-mask-x⑦"></a>

<a id="ref-for-element-attrdef-mask-y⑦"></a>

<a id="ref-for-element-attrdef-mask-width⑦"></a>

The height of the largest possible offscreen buffer. A negative value or a value of zero disables rendering of the element. If the attribute is not specified but at least one of the attributes [x](#element-attrdef-mask-x), [y](#element-attrdef-mask-y) or [width](#element-attrdef-mask-width) are specified, the effect is as if a value of 120% were specified.

Animatable: yes.

<a id="ref-for-element-attrdef-mask-x⑧"></a>

<a id="ref-for-element-attrdef-mask-y⑧"></a>

<a id="ref-for-element-attrdef-mask-width⑧"></a>

<a id="ref-for-element-attrdef-mask-height⑧"></a>

If at least one of the attributes [x](#element-attrdef-mask-x), [y](#element-attrdef-mask-y), [width](#element-attrdef-mask-width) or [height](#element-attrdef-mask-height) are specified, the given object and the rectangle defined by <a id="ref-for-element-attrdef-mask-x⑨"></a>x, <a id="ref-for-element-attrdef-mask-y⑨"></a>y, <a id="ref-for-element-attrdef-mask-width⑨"></a>width and <a id="ref-for-element-attrdef-mask-height⑨"></a>height establish a current clipping path. The rendered content of the mask must be clipped by this current clipping path.

<a id="ref-for-elementdef-mask②①"></a>

CSS properties inherit into the [mask](#elementdef-mask) element from its ancestors; properties do <em>not</em> inherit from the element referencing the <a id="ref-for-elementdef-mask②②"></a>mask element.

<a id="ref-for-elementdef-mask②③"></a>

<a id="ref-for-propdef-mask①⓪"></a>

<a id="ref-for-propdef-opacity⑦"></a>

<a id="ref-for-propdef-filter②"></a>

<a id="ref-for-propdef-display⑥"></a>

<a id="ref-for-valdef-display-none②"></a>

[mask](#elementdef-mask) elements are never rendered directly; their only usage is as something that can be referenced using the [mask](#propdef-mask) property. The [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity), [filter](https://www.w3.org/TR/filter-effects-1/#propdef-filter) and [display](https://www.w3.org/TR/css-display-3/#propdef-display) properties do not apply to the <a id="ref-for-elementdef-mask②④"></a>mask element; thus, <a id="ref-for-elementdef-mask②⑤"></a>mask elements are not directly rendered even if the <a id="ref-for-propdef-display⑦"></a>display property is set to a value other than [none](https://www.w3.org/TR/css-display-3/#valdef-display-none), and <a id="ref-for-elementdef-mask②⑥"></a>mask elements are available for referencing even when the <a id="ref-for-propdef-display⑧"></a>display property on the <a id="ref-for-elementdef-mask②⑦"></a>mask element or any of its ancestors is set to <a id="ref-for-valdef-display-none③"></a>none.

<a id="ref-for-propdef-mask-type④"></a>

### <a id="the-mask-type"></a>9.2. Mask Source Interpretation: the [mask-type](#propdef-mask-type) property

| Field               | Definition                                                                            |
|---------------------|---------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-mask-type"></a>mask-type                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one②③"></a>luminance [\|](https://www.w3.org/TR/css-values-4/#comb-one) alpha |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | luminance                                                                             |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-mask②⑧"></a>[mask](#elementdef-mask) elements                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                          |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                              |
| <strong>Media:&#xA;      </strong> | visual                                                                                |

<a id="ref-for-propdef-mask-type⑤"></a>

<a id="ref-for-elementdef-mask②⑨"></a>

The [mask-type](#propdef-mask-type) property defines whether the content of the [mask](#elementdef-mask) element is treated as as luminance mask or alpha mask, as described in [Calculating mask values](#MaskValues).

Values have the following meanings:

<a id="valdef-mask-type-luminance"></a>luminance  
Indicates that the luminance values of the mask should be used.

<a id="valdef-mask-type-alpha"></a>alpha  
Indicates that the alpha values of the mask should be used.

<a id="ref-for-propdef-mask-type⑥"></a>

<a id="ref-for-elementdef-mask③⓪"></a>

<a id="ref-for-propdef-mask-mode⑧"></a>

<a id="ref-for-valdef-mask-mode-match-source"></a>

The [mask-type](#propdef-mask-type) property allows the author of the [mask](#elementdef-mask) element to specify the preferred masking mode. However, the author can override this preference by setting the [mask-mode](#propdef-mask-mode) value to something different than [match-source](#valdef-mask-mode-match-source) on the masked content.

<a id="ref-for-propdef-mask-type⑦"></a>

<a id="ref-for-valdef-mask-type-luminance①"></a>

<a id="ref-for-propdef-mask-mode⑨"></a>

<a id="ref-for-valdef-mask-mode-match-source①"></a>

<a id="ref-for-elementdef-mask③①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c0f95fbd"></a> In the following example the computed value of [mask-type](#propdef-mask-type) is [luminance](#valdef-mask-type-luminance) and the computed value of [mask-mode](#propdef-mask-mode) is [match-source](#valdef-mask-mode-match-source). The UA must follow the preferred masking mode defined on the [mask](#elementdef-mask) element.
>
> ```text
> <svg>
>   <mask style="mask-type: luminance;" id="mask">
>     ...
>   </mask>
> </svg>
> 
> <p style="mask-image: url(#mask); mask-mode: auto;">
>   This is the masked content.
> </p>
> ```
>
> <a id="ref-for-propdef-mask-mode①⓪"></a>
>
> <a id="ref-for-valdef-mask-mode-alpha①"></a>
>
> <a id="ref-for-elementdef-mask③②"></a>
>
> <a id="ref-for-valdef-mask-type-luminance②"></a>
>
> <a id="ref-for-mask-layer-image①⑧"></a>
>
> In the next example the computed value of [mask-mode](#propdef-mask-mode) is [alpha](#valdef-mask-mode-alpha) and overrides the preference on the [mask](#elementdef-mask) element that is computed to [luminance](#valdef-mask-type-luminance). The [mask layer image](#mask-layer-image) is used as an alpha mask.
>
> ```text
> lt;svg>
>  <mask style="mask-type: luminance;" id="mask2">
>    ...
>  </mask>
> lt;/svg>
> 
> lt;p style="mask-image: url(#mask2); mask-mode: alpha;">
>  This is the masked content.
> lt;/p>
> ```
<a id="ref-for-propdef-mask-type⑧"></a>

The [mask-type](#propdef-mask-type) property is a [presentation attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) for SVG elements.

## <a id="priv-sec"></a>10. Privacy and Security Considerations

It is important that the timing to the masking operations is independent of the source and destination pixel. Masking operations must be implemented in such a way that they always take the same amount of time regardless of the pixel values. If this rule is not followed, an attacker could infer information and mount a timing attack.

A timing attack is a method of obtaining information about content that is otherwise protected, based on studying the amount of time it takes for an operation to occur. If, for example, red pixels took longer to draw than green pixels, one might be able to reconstruct a rough image of the element being rendered, without ever having access to the content of the element.

<a id="ref-for-typedef-mask-source④"></a>

<a id="ref-for-typedef-clip-source①"></a>

[\<mask-source\>](#typedef-mask-source)s and [\<clip-source\>](#typedef-clip-source)s have special requirements on fetching resources.

<a id="ref-for-typedef-mask-source⑤"></a>

<a id="ref-for-typedef-clip-source②"></a>

<a id="ref-for-typedef-image⑦"></a>

<a id="ref-for-propdef-mask-image①⑥"></a>

<a id="ref-for-propdef-mask-border-source①⑥"></a>

<a id="ref-for-propdef-clip-path①⑥"></a>

User agents must use the [potentially CORS-enabled fetch](https://fetch.spec.whatwg.org/#main-fetch) method defined by the [\[FETCH\]](#biblio-fetch) specification for all [\<mask-source\>](#typedef-mask-source), [\<clip-source\>](#typedef-clip-source) and [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) values on the [mask-image](#propdef-mask-image), [mask-border-source](#propdef-mask-border-source) and [clip-path](#propdef-clip-path) properties. When fetching, user agents must use “Anonymous” mode, set the referrer source to the stylesheet’s URL and set the origin to the URL of the containing document. If this results in network errors, the effect is as if the value none had been specified.

<a id="ref-for-propdef-clip③"></a>

## <a id="clip-property"></a>Appendix A: The deprecated [clip](#propdef-clip) property

| Field               | Definition                                                                                                                                                                                                                                                                                                                       |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-clip"></a>clip                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-valdef-clip-auto"></a><a id="ref-for-comb-one②④"></a><a id="ref-for-funcdef-clip-rect"></a>[rect()](#funcdef-clip-rect) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [auto](https://drafts.csswg.org/css2/#valdef-clip-auto)                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                                                                                                             |
| <strong>Applies to:&#xA;      </strong> | <a id="ref-for-elementdef-mask③③"></a><a id="ref-for-elementdef-pattern①"></a>Absolutely positioned elements. In SVG, it applies to [elements which establish a new viewport](https://www.w3.org/TR/SVG/coords.html#EstablishingANewSVGViewport), [pattern](https://www.w3.org/TR/SVG2/pservers.html#elementdef-pattern) elements and [mask](#elementdef-mask) elements. |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                                                                                                                                     |
| <strong>Canonical order:&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by [computed value](https://drafts.csswg.org/web-animations-1/#animation-type)                                                                                                                                                                                                                                                   |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                                                           |

<a id="ref-for-propdef-clip④"></a>

<a id="ref-for-propdef-clip-path①⑦"></a>

With this specification the [clip](#propdef-clip) property is deprecated. Authors are encouraged to use the [clip-path](#propdef-clip-path) property instead. UAs must support the <a id="ref-for-propdef-clip⑤"></a>clip property.

<a id="ref-for-propdef-clip⑥"></a>

<a id="ref-for-elementdef-pattern②"></a>

<a id="ref-for-elementdef-mask③④"></a>

The [clip](#propdef-clip) property applies only to absolutely positioned elements. In SVG, it applies to [elements which establish a new viewport](https://www.w3.org/TR/SVG/coords.html#EstablishingANewSVGViewport), [pattern](https://www.w3.org/TR/SVG2/pservers.html#elementdef-pattern) elements and [mask](#elementdef-mask) elements. Values have the following meanings:

auto

The element does not clip.

<a id="ref-for-typedef-clip-left"></a>

<a id="ref-for-typedef-clip-bottom"></a>

<a id="ref-for-typedef-clip-right"></a>

<a id="ref-for-typedef-clip-top"></a>

<a id="funcdef-clip-rect"></a>rect() = rect( [\<top\>](#typedef-clip-top), [\<right\>](#typedef-clip-right), [\<bottom\>](#typedef-clip-bottom), [\<left\>](#typedef-clip-left) )

<a id="ref-for-typedef-clip-top①"></a>

<a id="ref-for-typedef-clip-bottom①"></a>

<a id="ref-for-typedef-clip-right①"></a>

<a id="ref-for-typedef-clip-left①"></a>

[\<top\>](#typedef-clip-top) and [\<bottom\>](#typedef-clip-bottom) specify offsets from the top border edge of the box, and [\<right\>](#typedef-clip-right), and [\<left\>](#typedef-clip-left) specify offsets from the left border edge of the box. Authors should separate offset values with commas. User agents must support separation with commas, but may also support separation without commas (but not a combination), because a previous revision of this specification was ambiguous in this respect.

<a id="ref-for-length-value④"></a>

<a id="ref-for-valdef-clip-auto①"></a>

<a id="ref-for-typedef-clip-top②"></a>

<a id="ref-for-typedef-clip-left②"></a>

<a id="ref-for-used-value⑤"></a>

<a id="ref-for-typedef-clip-bottom②"></a>

<a id="ref-for-typedef-clip-right②"></a>

<a id="typedef-clip-top"></a>\<top\>, <a id="typedef-clip-right"></a>\<right\>, <a id="typedef-clip-bottom"></a>\<bottom\>, and <a id="typedef-clip-left"></a>\<left\> may either have a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) value or [auto](https://drafts.csswg.org/css2/#valdef-clip-auto). Negative lengths are permitted. The value <a id="ref-for-valdef-clip-auto②"></a>auto means that a given edge of the clipping region will be the same as the edge of the element’s generated border box (i.e., <a id="ref-for-valdef-clip-auto③"></a>auto means the same as 0 for [\<top\>](#typedef-clip-top) and [\<left\>](#typedef-clip-left), the same as the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of the height plus the sum of vertical padding and border widths for [\<bottom\>](#typedef-clip-bottom), and the same as the used value of the width plus the sum of the horizontal padding and border widths for [\<right\>](#typedef-clip-right), such that four <a id="ref-for-valdef-clip-auto④"></a>auto values result in the clipping region being the same as the element’s border box).

<a id="ref-for-typedef-clip-left③"></a>

<a id="ref-for-typedef-clip-right③"></a>

<a id="ref-for-typedef-clip-top③"></a>

<a id="ref-for-typedef-clip-bottom③"></a>

<a id="ref-for-valdef-clip-auto⑤"></a>

When coordinates are rounded to pixel coordinates, care should be taken that no pixels remain visible when [\<left\>](#typedef-clip-left) and [\<right\>](#typedef-clip-right) have the same value (or [\<top\>](#typedef-clip-top) and [\<bottom\>](#typedef-clip-bottom) have the same value), and conversely that no pixels within the element’s border box remain hidden when these values are [auto](https://drafts.csswg.org/css2/#valdef-clip-auto).

<a id="ref-for-propdef-clip⑦"></a>

The [clip](#propdef-clip) property is a [presentation attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) for SVG elements.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-550001e5"></a> Example: The following two rules:
>
> ```text
> p#one { clip: rect(5px, 40px, 45px, 5px); }
> p#two { clip: rect(5px, 55px, 45px, 5px); }
> ```
>
> and assuming both Ps are 50 by 55 pixel, will create, respectively, the rectangular clipping regions delimited by the dashed lines in the following illustrations:
>
> ![Values for rect shape](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/images/clip.svg)
>
> This diagram illustrates two block boxes, one next to the other, with rectangular clipping regions of different dimensions. (See [long description](https://www.w3.org/TR/CSS2/images/longdesc/clip-desc.html).)

## <a id="compute-stroke-bounding-box"></a>Appendix B: Compute stroke bounding box

The algorithm to compute the <a id="stroke-bounding-box"></a>stroke bounding box is as follows, depending on the type of <var>element</var>:

<a id="ref-for-elementdef-image①"></a>

<a id="ref-for-elementdef-use②⓪"></a>

<a id="ref-for-graphics-element②②"></a>

a [graphics element](https://www.w3.org/TR/SVG2/struct.html#graphics-element) without [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) or [image](https://www.w3.org/TR/SVG2/embedded.html#elementdef-image)

<a id="ref-for-TermTextContentElement"></a>

<a id="ref-for-elementdef-a①"></a>

an [a](https://www.w3.org/TR/SVG2/linking.html#elementdef-a) element with a [text content element](https://svgwg.org/svg2-draft/text.html#TermTextContentElement)

1.  <a id="ref-for-TermObjectBoundingBox⑤"></a>

    Let <var>box</var> be a rectangle initialized to the [object bounding box](https://www.w3.org/TR/SVG2/coords.html#TermObjectBoundingBox) of <var>element</var>.

2.  <a id="ref-for-used-value⑥"></a>

    <a id="ref-for-StrokeWidthProperty③"></a>

    <a id="ref-for-StrokeProperty③"></a>

    If the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of [stroke-width](https://www.w3.org/TR/SVG2/painting.html#StrokeWidthProperty) \<= 0 or the used value of [stroke](https://www.w3.org/TR/SVG2/painting.html#StrokeProperty) is none return <var>box</var>.

3.  <a id="ref-for-StrokeWidthProperty④"></a>

    Let <var>delta</var> be the inflation value initialized to the half of the [stroke-width](https://www.w3.org/TR/SVG2/painting.html#StrokeWidthProperty).

4.  <a id="ref-for-elementdef-rect②"></a>

    <a id="ref-for-elementdef-ellipse②"></a>

    <a id="ref-for-elementdef-circle④"></a>

    <a id="ref-for-elementdef-image②"></a>

    If <var>element</var> is not [rect](https://www.w3.org/TR/SVG2/shapes.html#elementdef-rect), [ellipse](https://www.w3.org/TR/SVG2/shapes.html#elementdef-ellipse), [circle](https://www.w3.org/TR/SVG2/shapes.html#elementdef-circle) or [image](https://www.w3.org/TR/SVG2/embedded.html#elementdef-image) just follow one of the following conditions in the order they apply:

    <a id="ref-for-valdef-stroke-linejoin-miter"></a>

    <a id="ref-for-StrokeLinejoinProperty②"></a>

    the used value for [stroke-linejoin](https://www.w3.org/TR/SVG2/painting.html#StrokeLinejoinProperty) is [miter](https://www.w3.org/TR/fill-stroke-3/#valdef-stroke-linejoin-miter)

    1.  <a id="ref-for-StrokeMiterlimitProperty②"></a>

        Let <var>miter</var> be the used value of [stroke-miterlimit](https://www.w3.org/TR/SVG2/painting.html#StrokeMiterlimitProperty).

    2.  <a id="ref-for-StrokeLinecapProperty②"></a>

        <a id="ref-for-valdef-stroke-linecap-square"></a>

        If <var>miter</var> is smaller than the square root of 2 and if the used value for [stroke-linecap](https://www.w3.org/TR/SVG2/painting.html#StrokeLinecapProperty) is [square](https://www.w3.org/TR/fill-stroke-3/#valdef-stroke-linecap-square), multiply <var>delta</var> with the square root of 2. Otherwise, multiply <var>delta</var> with <var>miter</var>.

    <a id="ref-for-valdef-stroke-linecap-square①"></a>

    <a id="ref-for-StrokeLinecapProperty③"></a>

    the used value for [stroke-linecap](https://www.w3.org/TR/SVG2/painting.html#StrokeLinecapProperty) is [square](https://www.w3.org/TR/fill-stroke-3/#valdef-stroke-linecap-square)

    1.  Multiply <var>delta</var> with the square root of 2.

5.  Inflate <var>box</var> with the value of <var>delta</var>.

6.  Return <var>box</var>.

<a id="ref-for-StrokeOpacityProperty②"></a>

<a id="ref-for-StrokeDasharrayProperty②"></a>

<a id="ref-for-StrokeDashoffsetProperty②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The values of the [stroke-opacity](https://www.w3.org/TR/SVG2/painting.html#StrokeOpacityProperty), [stroke-dasharray](https://www.w3.org/TR/SVG2/painting.html#StrokeDasharrayProperty) and [stroke-dashoffset](https://www.w3.org/TR/SVG2/painting.html#StrokeDashoffsetProperty) do not affect the calculation of the stroke bounding box.

<a id="ref-for-container-element①⑧"></a>

a [container element](https://www.w3.org/TR/SVG2/struct.html#container-element)

<a id="ref-for-elementdef-use②①"></a>

[use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use)

1.  <a id="ref-for-container-element①⑨"></a>

    <a id="ref-for-elementdef-use②②"></a>

    Let <var>parent</var> be the [container element](https://www.w3.org/TR/SVG2/struct.html#container-element) if it is one, or the root of the [use](https://www.w3.org/TR/SVG2/struct.html#elementdef-use) element’s shadow tree otherwise.

2.  For each child <var>child</var> of <var>parent</var>

    1.  Invoke the stroke bounding box algorithm with <var>child</var>.

    2.  Let <var>childBox</var> be the returned box value of the invoked algorithm.

    3.  Map <var>childBox</var> from the coordinate space of <var>child</var> to the coordinate space of <var>parent</var>.

3.  Let <var>box</var> be the union of all <var>childBox</var>es.

4.  Return <var>box</var>.

<a id="ref-for-elementdef-image③"></a>

[image](https://www.w3.org/TR/SVG2/embedded.html#elementdef-image)

1.  <a id="ref-for-TermObjectBoundingBox⑥"></a>

    Return the [object bounding box](https://www.w3.org/TR/SVG2/coords.html#TermObjectBoundingBox) of <var>element</var>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A future version of the SVG specification may override this section.

## <a id="DOMInterfaces"></a>Appendix C: DOM interfaces

### <a id="InterfaceSVGClipPathElement"></a>Interface SVGClipPathElement

<a id="ref-for-elementdef-clippath③①"></a>

The <a id="svgclippathelement"></a>`SVGClipPathElement` interface corresponds to the [clipPath](#elementdef-clippath) element.

<a id="ref-for-Exposed"></a>

<a id="ref-for-svgclippathelement"></a>

<a id="ref-for-InterfaceSVGElement"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration"></a>

<a id="ref-for-dom-svgclippathelement-clippathunits"></a>

<a id="ref-for-InterfaceSVGAnimatedTransformList"></a>

<a id="ref-for-dom-svgclippathelement-transform"></a>

```text
[Exposed=Window]
interface SVGClipPathElement : SVGElement {
  readonly attribute SVGAnimatedEnumeration clipPathUnits;
  readonly attribute SVGAnimatedTransformList transform;
};
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedEnumeration①"></a>

<a id="dom-svgclippathelement-clippathunits"></a>`clipPathUnits`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/SVG2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-clippath-clippathunits②"></a>

<a id="ref-for-elementdef-clippath③②"></a>

<a id="ref-for-InterfaceSVGUnitTypes"></a>

Corresponds to attribute [clipPathUnits](#element-attrdef-clippath-clippathunits) on the given [clipPath](#elementdef-clippath) element. Takes one of the constants defined in [SVGUnitTypes](https://www.w3.org/TR/SVG2/types.html#InterfaceSVGUnitTypes).

<a id="ref-for-InterfaceSVGAnimatedTransformList①"></a>

<a id="dom-svgclippathelement-transform"></a>`transform`, of type [SVGAnimatedTransformList](https://www.w3.org/TR/SVG2/coords.html#InterfaceSVGAnimatedTransformList), readonly

<a id="ref-for-propdef-transform"></a>

Corresponds to presentation attribute [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) on the given element.

### <a id="InterfaceSVGMaskElement"></a>Interface SVGMaskElement

<a id="ref-for-elementdef-mask③⑤"></a>

The <a id="svgmaskelement"></a>`SVGMaskElement` interface corresponds to the [mask](#elementdef-mask) element.

<a id="ref-for-Exposed①"></a>

<a id="ref-for-svgmaskelement"></a>

<a id="ref-for-InterfaceSVGElement①"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration②"></a>

<a id="ref-for-dom-svgmaskelement-maskunits"></a>

<a id="ref-for-InterfaceSVGAnimatedEnumeration③"></a>

<a id="ref-for-dom-svgmaskelement-maskcontentunits"></a>

<a id="ref-for-InterfaceSVGAnimatedLength"></a>

<a id="ref-for-dom-svgmaskelement-x"></a>

<a id="ref-for-InterfaceSVGAnimatedLength①"></a>

<a id="ref-for-dom-svgmaskelement-y"></a>

<a id="ref-for-InterfaceSVGAnimatedLength②"></a>

<a id="ref-for-dom-svgmaskelement-width"></a>

<a id="ref-for-InterfaceSVGAnimatedLength③"></a>

<a id="ref-for-dom-svgmaskelement-height"></a>

```text
[Exposed=Window]
interface SVGMaskElement : SVGElement {
  readonly attribute SVGAnimatedEnumeration maskUnits;
  readonly attribute SVGAnimatedEnumeration maskContentUnits;
  readonly attribute SVGAnimatedLength x;
  readonly attribute SVGAnimatedLength y;
  readonly attribute SVGAnimatedLength width;
  readonly attribute SVGAnimatedLength height;
};
```
Attributes:

<a id="ref-for-InterfaceSVGAnimatedEnumeration④"></a>

<a id="dom-svgmaskelement-maskunits"></a>`maskUnits`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/SVG2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-mask-maskunits③"></a>

<a id="ref-for-elementdef-mask③⑥"></a>

<a id="ref-for-InterfaceSVGUnitTypes①"></a>

Corresponds to attribute [maskUnits](#element-attrdef-mask-maskunits) on the given [mask](#elementdef-mask) element. Takes one of the constants defined in [SVGUnitTypes](https://www.w3.org/TR/SVG2/types.html#InterfaceSVGUnitTypes).

<a id="ref-for-InterfaceSVGAnimatedEnumeration⑤"></a>

<a id="dom-svgmaskelement-maskcontentunits"></a>`maskContentUnits`, of type [SVGAnimatedEnumeration](https://www.w3.org/TR/SVG2/types.html#InterfaceSVGAnimatedEnumeration), readonly

<a id="ref-for-element-attrdef-mask-maskcontentunits②"></a>

<a id="ref-for-elementdef-mask③⑦"></a>

<a id="ref-for-InterfaceSVGUnitTypes②"></a>

Corresponds to attribute [maskContentUnits](#element-attrdef-mask-maskcontentunits) on the given [mask](#elementdef-mask) element. Takes one of the constants defined in [SVGUnitTypes](https://www.w3.org/TR/SVG2/types.html#InterfaceSVGUnitTypes).

<a id="ref-for-InterfaceSVGAnimatedLength④"></a>

<a id="dom-svgmaskelement-x"></a>`x`, of type [SVGAnimatedLength](https://www.w3.org/TR/SVG2/types.html#InterfaceSVGAnimatedLength), readonly

<a id="ref-for-element-attrdef-mask-x①⓪"></a>

<a id="ref-for-elementdef-mask③⑧"></a>

Corresponds to attribute [x](#element-attrdef-mask-x) on the given [mask](#elementdef-mask) element.

<a id="ref-for-InterfaceSVGAnimatedLength⑤"></a>

<a id="dom-svgmaskelement-y"></a>`y`, of type [SVGAnimatedLength](https://www.w3.org/TR/SVG2/types.html#InterfaceSVGAnimatedLength), readonly

<a id="ref-for-element-attrdef-mask-y①⓪"></a>

<a id="ref-for-elementdef-mask③⑨"></a>

Corresponds to attribute [y](#element-attrdef-mask-y) on the given [mask](#elementdef-mask) element.

<a id="ref-for-InterfaceSVGAnimatedLength⑥"></a>

<a id="dom-svgmaskelement-width"></a>`width`, of type [SVGAnimatedLength](https://www.w3.org/TR/SVG2/types.html#InterfaceSVGAnimatedLength), readonly

<a id="ref-for-element-attrdef-mask-width①⓪"></a>

<a id="ref-for-elementdef-mask④⓪"></a>

Corresponds to attribute [width](#element-attrdef-mask-width) on the given [mask](#elementdef-mask) element.

<a id="ref-for-InterfaceSVGAnimatedLength⑦"></a>

<a id="dom-svgmaskelement-height"></a>`height`, of type [SVGAnimatedLength](https://www.w3.org/TR/SVG2/types.html#InterfaceSVGAnimatedLength), readonly

<a id="ref-for-element-attrdef-mask-height①⓪"></a>

<a id="ref-for-elementdef-mask④①"></a>

Corresponds to attribute [height](#element-attrdef-mask-height) on the given [mask](#elementdef-mask) element.

## <a id="changes"></a>Changes since last publication

The following changes were made since the [26 August 2014 Candidate Recommendation](https://www.w3.org/TR/2014/CR-css-masking-1-20140826/).

- <a id="ref-for-propdef-mask-mode①①"></a>

  <a id="ref-for-propdef-mask①①"></a>

  <a id="ref-for-propdef-mask-position⑤"></a>

  <a id="ref-for-propdef-mask-size④"></a>

  Allowed the [\<'mask-mode'\>](#propdef-mask-mode) value in the [mask](#propdef-mask) shorthand to appear anywhere other than between [\<'mask-position'\>](#propdef-mask-position) and [\<'mask-size'\>](#propdef-mask-size).

- Removed Implements SVGUnitTypes on clipPath and mask elements.

- Apply properties that apply to all graphics elements to the use element as well.

- <a id="ref-for-propdef-mask-position⑥"></a>

  <a id="ref-for-propdef-mask-repeat③"></a>

  Change initial value of [mask-position](#propdef-mask-position) to 0% 0% and of [mask-repeat](#propdef-mask-repeat) to repeat.

- <a id="ref-for-propdef-mask-origin⑨"></a>

  <a id="ref-for-propdef-mask-clip⑨"></a>

  Remove <var>margin-box</var> as possible value from [mask-origin](#propdef-mask-origin) and [mask-clip](#propdef-mask-clip).

- <a id="ref-for-propdef-mask-clip①⓪"></a>

  <a id="ref-for-mask-layer-image①⑨"></a>

  <a id="ref-for-elementdef-mask④②"></a>

  Clarify that [mask-clip](#propdef-mask-clip) has no affect on [mask layer image](#mask-layer-image)s that reference a [mask](#elementdef-mask) element.

- <a id="ref-for-propdef-mask①②"></a>

  <a id="ref-for-typedef-geometry-box①①"></a>

  <a id="ref-for-valdef-mask-clip-no-clip③"></a>

  Clarify [mask](#propdef-mask) shorthand behavior on appearance of one [\<geometry-box\>](#typedef-geometry-box) and the [no-clip](#valdef-mask-clip-no-clip) keyword.

- <a id="ref-for-propdef-mask-mode①②"></a>

  <a id="ref-for-mask-layer-image②⓪"></a>

  <a id="ref-for-elementdef-mask④③"></a>

  <a id="ref-for-valdef-mask-mode-match-source②"></a>

  <a id="ref-for-propdef-mask-type⑨"></a>

  [mask-mode](#propdef-mask-mode) on a [mask layer image](#mask-layer-image) that is a reference to a [mask](#elementdef-mask) element with a value of [match-source](#valdef-mask-mode-match-source) should take the value of the [mask-type](#propdef-mask-type) property on this <a id="ref-for-elementdef-mask④④"></a>mask element.

- Mapping of boxes for SVG elements with and without associated CSS layout box changed to match mapping in Fill and Stroke spec.

- Editorial changes.

The following changes were made since the [22 May 2014 Working Draft](https://www.w3.org/TR/2014/WD-css-masking-1-20140522/).

- <a id="ref-for-propdef-mask-size⑤"></a>

  Change the inital value of [mask-size](#propdef-mask-size) from border-box to auto.

The following changes were made since the [13 February 2014 Working Draft](https://www.w3.org/TR/2014/WD-css-masking-1-20140213/).

- Renamed mask-box\* properties and terms to mask-border\*.

- <a id="ref-for-propdef-background"></a>

  Added support for multiple mask layers. (Similar to multiple background layers for [background](https://www.w3.org/TR/css-backgrounds-3/#propdef-background).)

- <a id="ref-for-propdef-mask-composite⑥"></a>

  <a id="ref-for-mask-layer-image②①"></a>

  <a id="ref-for-valdef-mask-composite-add②"></a>

  <a id="ref-for-valdef-mask-composite-subtract①"></a>

  <a id="ref-for-valdef-mask-composite-intersect①"></a>

  <a id="ref-for-valdef-mask-composite-exclude③"></a>

  Added the [mask-composite](#propdef-mask-composite) property to control compositing of multiple [mask layer images](#mask-layer-image) with the keywords [add](#valdef-mask-composite-add), [subtract](#valdef-mask-composite-subtract), [intersect](#valdef-mask-composite-intersect) and [exclude](#valdef-mask-composite-exclude).

- <a id="ref-for-propdef-mask-border-slice⑤"></a>

  <a id="ref-for-valdef-mask-border-slice-fill"></a>

  [mask-border-slice](#propdef-mask-border-slice) without keyword [fill](#valdef-mask-border-slice-fill) does not clip middle piece of content anymore. Changed initial value from 0 fill to 0.

- <a id="ref-for-propdef-clip-rule①⓪"></a>

  <a id="ref-for-propdef-mask-mode①③"></a>

  <a id="ref-for-propdef-mask-type①⓪"></a>

  Better description for [clip-rule](#propdef-clip-rule), [mask-mode](#propdef-mask-mode) and [mask-type](#propdef-mask-type).

- <a id="ref-for-valdef-mask-border-slice-fill①"></a>

  <a id="ref-for-valdef-clip-path-fill-box②"></a>

  <a id="ref-for-valdef-clip-path-stroke-box②"></a>

  Rename [fill](#valdef-mask-border-slice-fill) and stroke keywords to [fill-box](#valdef-clip-path-fill-box) and [stroke-box](#valdef-clip-path-stroke-box).

- <a id="ref-for-stroke-bounding-box③"></a>

  Added definition for [stroke bounding box](#stroke-bounding-box).

- <a id="ref-for-mask-layer-image②②"></a>

  <a id="ref-for-mask-border-image①⑥"></a>

  Better differentiation between mask images: [mask layer image](#mask-layer-image) and [mask border image](#mask-border-image).

The following changes were made since the [29 October 2013 Last Call Working Draft](https://www.w3.org/TR/2013/WD-css-masking-1-20131029/).

- Remove note that a future version of the spec will allow controlling hit testing on clipping.

- Changed order of sections within the document.

- <a id="ref-for-used-value⑦"></a>

  <a id="ref-for-propdef-mask-image①⑦"></a>

  <a id="ref-for-propdef-mask-repeat④"></a>

  <a id="ref-for-propdef-mask-position⑦"></a>

  <a id="ref-for-propdef-mask-clip①①"></a>

  <a id="ref-for-propdef-mask-origin①⓪"></a>

  <a id="ref-for-propdef-mask-size⑥"></a>

  <a id="ref-for-typedef-mask-source⑥"></a>

  Make clear that the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of the properties [mask-image](#propdef-mask-image), [mask-repeat](#propdef-mask-repeat), [mask-position](#propdef-mask-position), [mask-clip](#propdef-mask-clip), [mask-origin](#propdef-mask-origin) and [mask-size](#propdef-mask-size) must be ignored for [\<mask-source\>](#typedef-mask-source), not the properties.

- <a id="ref-for-propdef-mask-size⑦"></a>

  <a id="ref-for-propdef-mask-position⑧"></a>

  "Animatable" section of [mask-size](#propdef-mask-size), [mask-position](#propdef-mask-position) described as repeatable list of lists. Changed to a single list.

- <a id="ref-for-propdef-mask-repeat⑤"></a>

  <a id="ref-for-propdef-mask-position⑨"></a>

  Computed value of [mask-repeat](#propdef-mask-repeat) and [mask-position](#propdef-mask-position) was described as list of items. Changed to have just one value.

- Change link to ED from [https&#x3A;&#x2F;&#x2F;drafts&#x2E;fxtf&#x2E;org&#x2F;masking&#x2F;](https://drafts.fxtf.org/masking/) to [https&#x3A;&#x2F;&#x2F;drafts&#x2E;fxtf&#x2E;org&#x2F;css-masking-1&#x2F;](https://drafts.fxtf.org/css-masking-1/)

The following significant changes were made since the [20 June 2013 Working Draft](https://www.w3.org/TR/2013/WD-css-masking-20130620/).

- <a id="ref-for-propdef-mask①③"></a>

  [mask](#propdef-mask) resets mask-box properties.

- <a id="ref-for-propdef-mask-repeat⑥"></a>

  <a id="ref-for-propdef-mask-position①⓪"></a>

  <a id="ref-for-propdef-mask-origin①①"></a>

  <a id="ref-for-valdef-mask-origin-border-box②"></a>

  Initial values for [mask-repeat](#propdef-mask-repeat), [mask-position](#propdef-mask-position) and [mask-origin](#propdef-mask-origin) changed to no-repeat, center and [border-box](#valdef-mask-origin-border-box).

- Multiple layers of mask images were deferred to a future level of this specification.

- Added security model for pixel operations and fetching of masking and clipping resources.

- Deferred "child" and select() function to next level.

The following significant changes were made since the [15 November 2012 Working Draft](https://www.w3.org/TR/2012/WD-css-masking-20121115/).

- Better integration with terms and definitions of CSS Backgrounds and Borders module.

- <a id="ref-for-propdef-mask①④"></a>

  <a id="ref-for-propdef-background①"></a>

  Syntax changes on [mask](#propdef-mask) shorthand property to be conform with [background](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) shorthand property.

- <a id="ref-for-elementdef-mask④⑤"></a>

  <a id="ref-for-elementdef-clippath③③"></a>

  Define how the implementation can differ between an SVG resource ([mask](#elementdef-mask), [clipPath](#elementdef-clippath)) and an image resource.

- <a id="ref-for-propdef-mask-mode①④"></a>

  <a id="ref-for-propdef-mask-image①⑧"></a>

  Added [mask-mode](#propdef-mask-mode) property to alter between luminance and alpha mask on [mask-image](#propdef-mask-image).

- Adapt IDL definition of SVGMaskElement and SVGClipPathElement to WebIDL.

- Further editorial changes.

See detailed list of changes in the [ChangeLog](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/ChangeLog).

## <a id="acknowledgments"></a>Acknowledgments

Thanks to Elika J. Etemad, Cameron McCormack, Liam R. E. Quin, Björn Höhrmann, Alan Stearns, Jarek Foksa, David Baron, Boris Zbarsky, Markus Stange and Sara Soueidan for their careful reviews, comments, and corrections. Special thanks to CJ Gammon for graphical assets.

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

- [add](#valdef-mask-composite-add), in § 7.8
- alpha
  - [value for mask-border-mode](#valdef-mask-border-mode-alpha), in § 8.2
  - [value for mask-mode](#valdef-mask-mode-alpha), in § 7.2
  - [value for mask-type](#valdef-mask-type-alpha), in § 9.2
- border-box
  - [value for mask-clip](#valdef-mask-clip-border-box), in § 7.5
  - [value for mask-origin](#valdef-mask-origin-border-box), in § 7.6
- [\<bottom\>](#typedef-clip-bottom), in § Unnumbered section
- [clip](#propdef-clip), in § Unnumbered section
- [clip-path](#propdef-clip-path), in § 5.1
- [clipPath](#elementdef-clippath), in § 6.1
- clipPathUnits
  - [attribute for SVGClipPathElement](#dom-svgclippathelement-clippathunits), in § Unnumbered section
  - [element-attr for clipPath](#element-attrdef-clippath-clippathunits), in § 6.1
- [clipping path](#clipping-path), in § 1.1
- [clipping region](#clipping-region), in § 5
- [clip-rule](#propdef-clip-rule), in § 6.2
- [\<clip-source\>](#typedef-clip-source), in § 5.1
- [\<compositing-operator\>](#typedef-compositing-operator), in § 7.8
- content-box
  - [value for mask-clip](#valdef-mask-clip-content-box), in § 7.5
  - [value for mask-origin](#valdef-mask-origin-content-box), in § 7.6
- [destination](#destination), in § 7.8
- [evenodd](#valdef-clip-rule-evenodd), in § 6.2
- [exclude](#valdef-mask-composite-exclude), in § 7.8
- [fill](#valdef-mask-border-slice-fill), in § 8.3
- fill-box
  - [value for clip-path](#valdef-clip-path-fill-box), in § 5.1
  - [value for mask-clip](#valdef-mask-clip-fill-box), in § 7.5
  - [value for mask-origin](#valdef-mask-origin-fill-box), in § 7.6
- [\<geometry-box\>](#typedef-geometry-box), in § 5.1
- height
  - [attribute for SVGMaskElement](#dom-svgmaskelement-height), in § Unnumbered section
  - [element-attr for mask](#element-attrdef-mask-height), in § 9.1
- [intersect](#valdef-mask-composite-intersect), in § 7.8
- [\<left\>](#typedef-clip-left), in § Unnumbered section
- luminance
  - [value for mask-border-mode](#valdef-mask-border-mode-luminance), in § 8.2
  - [value for mask-mode](#valdef-mask-mode-luminance), in § 7.2
  - [value for mask-type](#valdef-mask-type-luminance), in § 9.2
- mask
  - [(element)](#elementdef-mask), in § 9.1
  - [(property)](#propdef-mask), in § 7.9
- [mask-border](#propdef-mask-border), in § 8.7
- [mask border image](#mask-border-image), in § 8.1
- [mask border image area](#mask-border-image-area), in § 8.4
- [mask-border-mode](#propdef-mask-border-mode), in § 8.2
- [mask-border-outset](#propdef-mask-border-outset), in § 8.5
- [mask-border-repeat](#propdef-mask-border-repeat), in § 8.6
- [mask-border-slice](#propdef-mask-border-slice), in § 8.3
- [mask-border-source](#propdef-mask-border-source), in § 8.1
- [mask-border-width](#propdef-mask-border-width), in § 8.4
- [mask-clip](#propdef-mask-clip), in § 7.5
- [mask-composite](#propdef-mask-composite), in § 7.8
- maskContentUnits
  - [attribute for SVGMaskElement](#dom-svgmaskelement-maskcontentunits), in § Unnumbered section
  - [element-attr for mask](#element-attrdef-mask-maskcontentunits), in § 9.1
- [mask image](#mask-image), in § 7.10.1
- [mask-image](#propdef-mask-image), in § 7.1
- [\<masking-mode\>](#typedef-masking-mode), in § 7.2
- [\<mask-layer\>](#typedef-mask-layer), in § 7.9
- [mask layer image](#mask-layer-image), in § 7.1
- [mask-mode](#propdef-mask-mode), in § 7.2
- [mask-origin](#propdef-mask-origin), in § 7.6
- [mask painting area](#mask-painting-area), in § 7.5
- mask-position
  - [(property)](#propdef-mask-position), in § 7.4
  - [definition of](#mask-position), in § 4
- [mask positioning area](#mask-positioning-area), in § 7.6
- [\<mask-reference\>](#typedef-mask-reference), in § 7.1
- [mask-repeat](#propdef-mask-repeat), in § 7.3
- mask-size
  - [(property)](#propdef-mask-size), in § 7.7
  - [definition of](#mask-size), in § 4
- [\<mask-source\>](#typedef-mask-source), in § 7.1
- [mask-type](#propdef-mask-type), in § 9.2
- maskUnits
  - [attribute for SVGMaskElement](#dom-svgmaskelement-maskunits), in § Unnumbered section
  - [element-attr for mask](#element-attrdef-mask-maskunits), in § 9.1
- [match-source](#valdef-mask-mode-match-source), in § 7.2
- [no-clip](#valdef-mask-clip-no-clip), in § 7.5
- [nonzero](#valdef-clip-rule-nonzero), in § 6.2
- objectBoundingBox
  - [value for clipPathUnits](#valdef-clippathunits-objectboundingbox), in § 6.1
  - [value for maskContentUnits](#valdef-maskcontentunits-objectboundingbox), in § 9.1
  - [value for maskUnits](#valdef-maskunits-objectboundingbox), in § 9.1
- padding-box
  - [value for mask-clip](#valdef-mask-clip-padding-box), in § 7.5
  - [value for mask-origin](#valdef-mask-origin-padding-box), in § 7.6
- [rect()](#funcdef-clip-rect), in § Unnumbered section
- [\<right\>](#typedef-clip-right), in § Unnumbered section
- [source](#source), in § 7.8
- [stroke bounding box](#stroke-bounding-box), in § Unnumbered section
- stroke-box
  - [value for clip-path](#valdef-clip-path-stroke-box), in § 5.1
  - [value for mask-clip](#valdef-mask-clip-stroke-box), in § 7.5
  - [value for mask-origin](#valdef-mask-origin-stroke-box), in § 7.6
- [subtract](#valdef-mask-composite-subtract), in § 7.8
- [SVGClipPathElement](#svgclippathelement), in § Unnumbered section
- [SVGMaskElement](#svgmaskelement), in § Unnumbered section
- [\<top\>](#typedef-clip-top), in § Unnumbered section
- [transform](#dom-svgclippathelement-transform), in § Unnumbered section
- [\<url\>](#valdef-mask-image-url), in § 7.1
- userSpaceOnUse
  - [value for clipPathUnits](#valdef-clippathunits-userspaceonuse), in § 6.1
  - [value for maskContentUnits](#valdef-maskcontentunits-userspaceonuse), in § 9.1
  - [value for maskUnits](#valdef-maskunits-userspaceonuse), in § 9.1
- view-box
  - [value for clip-path](#valdef-clip-path-view-box), in § 5.1
  - [value for mask-clip](#valdef-mask-clip-view-box), in § 7.5
  - [value for mask-origin](#valdef-mask-origin-view-box), in § 7.6
- width
  - [attribute for SVGMaskElement](#dom-svgmaskelement-width), in § Unnumbered section
  - [element-attr for mask](#element-attrdef-mask-width), in § 9.1
- x
  - [attribute for SVGMaskElement](#dom-svgmaskelement-x), in § Unnumbered section
  - [element-attr for mask](#element-attrdef-mask-x), in § 9.1
- y
  - [attribute for SVGMaskElement](#dom-svgmaskelement-y), in § Unnumbered section
  - [element-attr for mask](#element-attrdef-mask-y), in § 9.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-break-4\] defines the following terms:
  - <a id="term-for-propdef-box-decoration-break"></a>box-decoration-break
- \[css-cascade-5\] defines the following terms:
  - <a id="term-for-used-value"></a>used value
- \[css-color-4\] defines the following terms:
  - <a id="term-for-propdef-color"></a>color
  - <a id="term-for-propdef-opacity"></a>opacity
- \[css-display-3\] defines the following terms:
  - <a id="term-for-propdef-display"></a>display
  - <a id="term-for-valdef-display-none"></a>none
- \[css-fonts-4\] defines the following terms:
  - <a id="term-for-propdef-font"></a>font
  - <a id="term-for-propdef-font-family"></a>font-family
  - <a id="term-for-propdef-font-stretch"></a>font-stretch
  - <a id="term-for-propdef-font-style"></a>font-style
  - <a id="term-for-propdef-font-variant"></a>font-variant
  - <a id="term-for-propdef-font-weight"></a>font-weight
- \[css-fonts-5\] defines the following terms:
  - <a id="term-for-descdef-font-face-font-size"></a>font-size
  - <a id="term-for-propdef-font-size-adjust"></a>font-size-adjust
- \[css-images-3\] defines the following terms:
  - <a id="term-for-typedef-image"></a>\<image\>
  - <a id="term-for-propdef-image-rendering"></a>image-rendering
- \[css-inline-3\] defines the following terms:
  - <a id="term-for-propdef-alignment-baseline"></a>alignment-baseline
  - <a id="term-for-propdef-baseline-shift"></a>baseline-shift
  - <a id="term-for-propdef-dominant-baseline"></a>dominant-baseline
- \[css-overflow-3\] defines the following terms:
  - <a id="term-for-propdef-overflow"></a>overflow
  - <a id="term-for-valdef-overflow-visible"></a>visible
- \[CSS-SHAPES\] defines the following terms:
  - <a id="term-for-typedef-basic-shape"></a>\<basic-shape\>
  - <a id="term-for-typedef-shape-box"></a>\<shape-box\>
  - <a id="term-for-funcdef-polygon"></a>polygon()
- \[css-text-3\] defines the following terms:
  - <a id="term-for-propdef-letter-spacing"></a>letter-spacing
  - <a id="term-for-propdef-word-spacing"></a>word-spacing
- \[css-text-decor-3\] defines the following terms:
  - <a id="term-for-propdef-text-decoration"></a>text-decoration
- \[css-ui-3\] defines the following terms:
  - <a id="term-for-propdef-cursor"></a>cursor
- \[css-values-4\] defines the following terms:
  - <a id="term-for-mult-comma"></a>\#
  - <a id="term-for-typedef-length-percentage"></a>\<length-percentage\>
  - <a id="term-for-length-value"></a>\<length\>
  - <a id="term-for-number-value"></a>\<number\>
  - <a id="term-for-typedef-position"></a>\<position\>
  - <a id="term-for-url-value"></a>\<url\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-px"></a>px
  - <a id="term-for-mult-num-range"></a>{a,b}
  - <a id="term-for-comb-one"></a>\|
  - <a id="term-for-comb-any"></a>\|\|
- \[css-writing-modes-3\] defines the following terms:
  - <a id="term-for-propdef-direction"></a>direction
  - <a id="term-for-propdef-unicode-bidi"></a>unicode-bidi
- \[css-writing-modes-4\] defines the following terms:
  - <a id="term-for-propdef-glyph-orientation-vertical"></a>glyph-orientation-vertical
  - <a id="term-for-propdef-writing-mode"></a>writing-mode
- \[CSS21\] defines the following terms:
  - <a id="term-for-valdef-clip-auto"></a>auto
  - <a id="term-for-propdef-visibility"></a>visibility
- \[CSS3-TRANSFORMS\] defines the following terms:
  - <a id="term-for-propdef-transform"></a>transform
  - <a id="term-for-user-coordinate-system"></a>user coordinate system
- \[CSS3BG\] defines the following terms:
  - <a id="term-for-typedef-bg-size"></a>\<bg-size\>
  - <a id="term-for-typedef-repeat-style"></a>\<repeat-style\>
  - <a id="term-for-propdef-background"></a>background
  - <a id="term-for-background-painting-area"></a>background painting area
  - <a id="term-for-background-positioning-area"></a>background positioning area
  - <a id="term-for-propdef-background-origin"></a>background-origin
  - <a id="term-for-propdef-background-position"></a>background-position
  - <a id="term-for-propdef-background-repeat"></a>background-repeat
  - <a id="term-for-propdef-background-size"></a>background-size
  - <a id="term-for-border-image-area"></a>border image area
  - <a id="term-for-propdef-border-image"></a>border-image
  - <a id="term-for-propdef-border-image-repeat"></a>border-image-repeat
  - <a id="term-for-propdef-border-image-slice"></a>border-image-slice
  - <a id="term-for-propdef-border-image-width"></a>border-image-width
  - <a id="term-for-propdef-border-radius"></a>border-radius
  - <a id="term-for-propdef-border-width"></a>border-width
- \[CSS3VAL\] defines the following terms:
  - <a id="term-for-typedef-number-percentage"></a>\<number-percentage\>
- \[fill-stroke-3\] defines the following terms:
  - <a id="term-for-valdef-stroke-linejoin-miter"></a>miter
  - <a id="term-for-valdef-stroke-linecap-square"></a>square
- \[FILTER-EFFECTS\] defines the following terms:
  - <a id="term-for-propdef-color-interpolation-filters"></a>color-interpolation-filters
  - <a id="term-for-elementdef-fecolormatrix"></a>fecolormatrix
  - <a id="term-for-propdef-filter"></a>filter
  - <a id="term-for-propdef-flood-color"></a>flood-color
  - <a id="term-for-propdef-flood-opacity"></a>flood-opacity
  - <a id="term-for-propdef-lighting-color"></a>lighting-color
  - <a id="term-for-valdef-color-interpolation-filters-linearrgb"></a>linearrgb
- \[SVG11\] defines the following terms:
  - <a id="term-for-AlternateGlyphDefinitions"></a>altglyphdef
  - <a id="term-for-AnimateElement"></a>animate
  - <a id="term-for-AnimateColorElement"></a>animatecolor
  - <a id="term-for-AnimateMotionElement"></a>animatemotion
  - <a id="term-for-AnimateTransformElement"></a>animatetransform
  - <a id="term-for-ColorProfileProperty"></a>color-profile
  - <a id="term-for-CursorElement"></a>cursor
  - <a id="term-for-EnableBackgroundProperty"></a>enable-background
  - <a id="term-for-FontElement"></a>font
  - <a id="term-for-FontFaceElement"></a>font-face
  - <a id="term-for-GlyphOrientationHorizontalProperty"></a>glyph-orientation-horizontal
  - <a id="term-for-KerningProperty"></a>kerning
  - <a id="term-for-SetElement"></a>set
- \[SVG2\] defines the following terms:
  - <a id="term-for-InterfaceSVGAnimatedEnumeration"></a>SVGAnimatedEnumeration
  - <a id="term-for-InterfaceSVGAnimatedLength"></a>SVGAnimatedLength
  - <a id="term-for-InterfaceSVGAnimatedTransformList"></a>SVGAnimatedTransformList
  - <a id="term-for-InterfaceSVGElement"></a>SVGElement
  - <a id="term-for-InterfaceSVGUnitTypes"></a>SVGUnitTypes
  - <a id="term-for-elementdef-a"></a>a
  - <a id="term-for-basic-shape"></a>basic shape
  - <a id="term-for-bounding-box"></a>bounding box
  - <a id="term-for-elementdef-circle"></a>circle
  - <a id="term-for-ColorInterpolationProperty"></a>color-interpolation
  - <a id="term-for-ColorRenderingProperty"></a>color-rendering
  - <a id="term-for-container-element"></a>container element
  - <a id="term-for-elementdef-defs"></a>defs
  - <a id="term-for-elementdef-desc"></a>desc
  - <a id="term-for-elementdef-ellipse"></a>ellipse
  - <a id="term-for-FillProperty"></a>fill
  - <a id="term-for-FillOpacityProperty"></a>fill-opacity
  - <a id="term-for-FillRuleProperty"></a>fill-rule
  - <a id="term-for-elementdef-foreignObject"></a>foreignobject
  - <a id="term-for-elementdef-g"></a>g
  - <a id="term-for-graphics-element"></a>graphics element
  - <a id="term-for-elementdef-image"></a>image
  - <a id="term-for-elementdef-line"></a>line
  - <a id="term-for-elementdef-linearGradient"></a>lineargradient
  - <a id="term-for-MarkerProperty"></a>marker
  - <a id="term-for-MarkerEndProperty"></a>marker-end
  - <a id="term-for-MarkerMidProperty"></a>marker-mid
  - <a id="term-for-MarkerStartProperty"></a>marker-start
  - <a id="term-for-elementdef-metadata"></a>metadata
  - <a id="term-for-TermNeverRenderedElement"></a>never-rendered element
  - <a id="term-for-TermObjectBoundingBox"></a>object bounding box
  - <a id="term-for-elementdef-path"></a>path
  - <a id="term-for-elementdef-pattern"></a>pattern
  - <a id="term-for-PointerEventsProperty"></a>pointer-events
  - <a id="term-for-elementdef-polygon"></a>polygon
  - <a id="term-for-elementdef-polyline"></a>polyline
  - <a id="term-for-elementdef-radialGradient"></a>radialgradient
  - <a id="term-for-elementdef-rect"></a>rect
  - <a id="term-for-elementdef-script"></a>script
  - <a id="term-for-ShapeRenderingProperty"></a>shape-rendering
  - <a id="term-for-TermStackingContext"></a>stacking contexts
  - <a id="term-for-StopColorProperty"></a>stop-color
  - <a id="term-for-StopOpacityProperty"></a>stop-opacity
  - <a id="term-for-StrokeProperty"></a>stroke
  - <a id="term-for-StrokeDasharrayProperty"></a>stroke-dasharray
  - <a id="term-for-StrokeDashoffsetProperty"></a>stroke-dashoffset
  - <a id="term-for-StrokeLinecapProperty"></a>stroke-linecap
  - <a id="term-for-StrokeLinejoinProperty"></a>stroke-linejoin
  - <a id="term-for-StrokeMiterlimitProperty"></a>stroke-miterlimit
  - <a id="term-for-StrokeOpacityProperty"></a>stroke-opacity
  - <a id="term-for-StrokeWidthProperty"></a>stroke-width
  - <a id="term-for-elementdef-style"></a>style
  - <a id="term-for-elementdef-svg"></a>svg
  - <a id="term-for-elementdef-switch"></a>switch
  - <a id="term-for-elementdef-symbol"></a>symbol
  - <a id="term-for-elementdef-text"></a>text
  - <a id="term-for-TermTextContentElement"></a>text content element
  - <a id="term-for-TextAnchorProperty"></a>text-anchor
  - <a id="term-for-TextRenderingProperty"></a>text-rendering
  - <a id="term-for-elementdef-title"></a>title
  - <a id="term-for-elementdef-use"></a>use
  - <a id="term-for-elementdef-view"></a>view
  - <a id="term-for-ViewBoxAttribute"></a>viewbox
- \[WebIDL\] defines the following terms:
  - <a id="term-for-Exposed"></a>Exposed

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-compositing-1"></a>\[COMPOSITING-1\]  
Rik Cabanier; Nikos Andronikos. [Compositing and Blending Level 1](https://www.w3.org/TR/compositing-1/). 13 January 2015. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;compositing-1&#x2F;](https://www.w3.org/TR/compositing-1/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 8 June 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 1 June 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 18 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 29 July 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-fonts-5"></a>\[CSS-FONTS-5\]  
Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 5](https://www.w3.org/TR/css-fonts-5/). 29 July 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-5&#x2F;](https://www.w3.org/TR/css-fonts-5/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 17 December 2020. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Dave Cramer; Elika Etemad; Steve Zilles. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 27 August 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
David Baron; Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 3 June 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-shapes"></a>\[CSS-SHAPES\]  
Vincent Hardy; Rossen Atanassov; Alan Stearns. [CSS Shapes Module Level 1](https://www.w3.org/TR/css-shapes-1/). 20 March 2014. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-shapes-1&#x2F;](https://www.w3.org/TR/css-shapes-1/)

<a id="biblio-css-text-3"></a>\[CSS-TEXT-3\]  
Elika Etemad; Koji Ishii; Florian Rivoal. [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/). 22 April 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-3&#x2F;](https://www.w3.org/TR/css-text-3/)

<a id="biblio-css-text-decor-3"></a>\[CSS-TEXT-DECOR-3\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 3](https://www.w3.org/TR/css-text-decor-3/). 13 August 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-3&#x2F;](https://www.w3.org/TR/css-text-decor-3/)

<a id="biblio-css-ui-3"></a>\[CSS-UI-3\]  
Tantek Çelik; Florian Rivoal. [CSS Basic User Interface Module Level 3 (CSS3 UI)](https://www.w3.org/TR/css-ui-3/). 21 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-3&#x2F;](https://www.w3.org/TR/css-ui-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 15 July 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-writing-modes-3"></a>\[CSS-WRITING-MODES-3\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 3](https://www.w3.org/TR/css-writing-modes-3/). 10 December 2019. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-3&#x2F;](https://www.w3.org/TR/css-writing-modes-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css3-transforms"></a>\[CSS3-TRANSFORMS\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css3bg"></a>\[CSS3BG\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 26 July 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css3val"></a>\[CSS3VAL\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-fetch"></a>\[FETCH\]  
Anne van Kesteren. [Fetch Standard](https://fetch.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;fetch&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://fetch.spec.whatwg.org/)

<a id="biblio-fill-stroke-3"></a>\[FILL-STROKE-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Fill and Stroke Module Level 3](https://www.w3.org/TR/fill-stroke-3/). 13 April 2017. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;fill-stroke-3&#x2F;](https://www.w3.org/TR/fill-stroke-3/)

<a id="biblio-filter-effects"></a>\[FILTER-EFFECTS\]  
Dirk Schulze; Dean Jackson. [Filter Effects Module Level 1](https://www.w3.org/TR/filter-effects-1/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;filter-effects-1&#x2F;](https://www.w3.org/TR/filter-effects-1/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-svg11"></a>\[SVG11\]  
Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG11&#x2F;](https://www.w3.org/TR/SVG11/)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

<a id="biblio-webidl"></a>\[WebIDL\]  
Boris Zbarsky. [Web IDL](https://heycam.github.io/webidl/). 15 December 2016. ED. URL: [https&#x3A;&#x2F;&#x2F;heycam&#x2E;github&#x2E;io&#x2F;webidl&#x2F;](https://heycam.github.io/webidl/)

### <a id="informative"></a>Informative References

<a id="biblio-css3color"></a>\[CSS3COLOR\]  
Tantek Çelik; Chris Lilley; David Baron. [CSS Color Module Level 3](https://www.w3.org/TR/css-color-3/). 19 June 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-3&#x2F;](https://www.w3.org/TR/css-color-3/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                                                                                                                                               | Initial                   | Applies to                                                                                                                         | Inh. | %ages                                                                                                       | Anim­ation type            | Canonical order | Com­puted value                                                                                                                                                             | Media  |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------|------------------------------------------------------------------------------------------------------------------------------------|------|-------------------------------------------------------------------------------------------------------------|---------------------------|-----------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------|
| <strong><span><a id="ref-for-propdef-clip⑧"></a></span><a href="#propdef-clip">clip</a>&#xA;      </strong> | rect() \| auto                                                                                                                                                                      | auto                      | Absolutely positioned elements. In SVG, it applies to elements which establish a new viewport, pattern elements and mask elements. | no   | n/a                                                                                                         | by computed value         | per grammar     | as specified                                                                                                                                                               | visual |
| <strong><span><a id="ref-for-propdef-clip-path①⑧"></a></span><a href="#propdef-clip-path">clip-path</a>&#xA;      </strong> | \<clip-source\> \| \[ \<basic-shape\> \|\| \<geometry-box\> \] \| none                                                                                                              | none                      | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | n/a                                                                                                         | by computed value         | per grammar     | as specified, but with \<url\> values made absolute                                                                                                                        | visual |
| <strong><span><a id="ref-for-propdef-clip-rule①①"></a></span><a href="#propdef-clip-rule">clip-rule</a>&#xA;      </strong> | nonzero \| evenodd                                                                                                                                                                  | nonzero                   | Applies to SVG graphics elements                                                                                                   | yes  | n/a                                                                                                         | discrete                  | per grammar     | as specified                                                                                                                                                               | visual |
| <strong><span><a id="ref-for-propdef-mask①⑤"></a></span><a href="#propdef-mask">mask</a>&#xA;      </strong> | \<mask-layer\>#                                                                                                                                                                     | see individual properties | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | see individual properties                                                                                   | see individual properties | per grammar     | see individual properties                                                                                                                                                  | visual |
| <strong><span><a id="ref-for-propdef-mask-border⑨"></a></span><a href="#propdef-mask-border">mask-border</a>&#xA;      </strong> | \<'mask-border-source'\> \|\| \<'mask-border-slice'\> \[ / \<'mask-border-width'\>? \[ / \<'mask-border-outset'\> \]? \]? \|\| \<'mask-border-repeat'\> \|\| \<'mask-border-mode'\> | See individual properties | See individual properties                                                                                                          | no   | n/a                                                                                                         | See individual properties | per grammar     | See individual properties                                                                                                                                                  | visual |
| <strong><span><a id="ref-for-propdef-mask-border-mode⑤"></a></span><a href="#propdef-mask-border-mode">mask-border-mode</a>&#xA;      </strong> | luminance \| alpha                                                                                                                                                                  | alpha                     | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | n/a                                                                                                         | discrete                  | per grammar     | as specified                                                                                                                                                               | visual |
| <strong><span><a id="ref-for-propdef-mask-border-outset⑥"></a></span><a href="#propdef-mask-border-outset">mask-border-outset</a>&#xA;      </strong> | \[ \<length\> \| \<number\> \]{1,4}                                                                                                                                                 | 0                         | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | n/a                                                                                                         | discrete                  | per grammar     | all \<length\>s made absolute, otherwise as specified                                                                                                                      | visual |
| <strong><span><a id="ref-for-propdef-mask-border-repeat④"></a></span><a href="#propdef-mask-border-repeat">mask-border-repeat</a>&#xA;      </strong> | \[ stretch \| repeat \| round \| space \]{1,2}                                                                                                                                      | stretch                   | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | n/a                                                                                                         | discrete                  | per grammar     | as specified                                                                                                                                                               | visual |
| <strong><span><a id="ref-for-propdef-mask-border-slice⑥"></a></span><a href="#propdef-mask-border-slice">mask-border-slice</a>&#xA;      </strong> | \<number-percentage\>{1,4} fill?                                                                                                                                                    | 0                         | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | refer to size of the mask border image                                                                      | discrete                  | per grammar     | as specified                                                                                                                                                               | visual |
| <strong><span><a id="ref-for-propdef-mask-border-source①⑦"></a></span><a href="#propdef-mask-border-source">mask-border-source</a>&#xA;      </strong> | none \| \<image\>                                                                                                                                                                   | none                      | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | n/a                                                                                                         | discrete                  | per grammar     | they keyword none or the computed \<image\>                                                                                                                                | visual |
| <strong><span><a id="ref-for-propdef-mask-border-width⑤"></a></span><a href="#propdef-mask-border-width">mask-border-width</a>&#xA;      </strong> | \[ \<length-percentage\> \| \<number\> \| auto \]{1,4}                                                                                                                              | auto                      | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | relative to width/height of the mask border image area                                                      | discrete                  | per grammar     | all \<length\>s made absolute, otherwise as specified                                                                                                                      | visual |
| <strong><span><a id="ref-for-propdef-mask-clip①②"></a></span><a href="#propdef-mask-clip">mask-clip</a>&#xA;      </strong> | \[ \<geometry-box\> \| no-clip \]#                                                                                                                                                  | border-box                | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | n/a                                                                                                         | discrete                  | per grammar     | as specified                                                                                                                                                               | visual |
| <strong><span><a id="ref-for-propdef-mask-composite⑦"></a></span><a href="#propdef-mask-composite">mask-composite</a>&#xA;      </strong> | \<compositing-operator\>#                                                                                                                                                           | add                       | All elements. In SVG, it applies to container elements without the defs element and all graphics elements                          | no   | n/a                                                                                                         | discrete                  | per grammar     | as specified                                                                                                                                                               | visual |
| <strong><span><a id="ref-for-propdef-mask-image①⑨"></a></span><a href="#propdef-mask-image">mask-image</a>&#xA;      </strong> | \<mask-reference\>#                                                                                                                                                                 | none                      | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | n/a                                                                                                         | discrete                  | per grammar     | the keyword none, a computed \<image\>, or a computed \<url\>                                                                                                              | visual |
| <strong><span><a id="ref-for-propdef-mask-mode①⑤"></a></span><a href="#propdef-mask-mode">mask-mode</a>&#xA;      </strong> | \<masking-mode\>#                                                                                                                                                                   | match-source              | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | n/a                                                                                                         | discrete                  | per grammar     | as specified                                                                                                                                                               | visual |
| <strong><span><a id="ref-for-propdef-mask-origin①②"></a></span><a href="#propdef-mask-origin">mask-origin</a>&#xA;      </strong> | \<geometry-box\>#                                                                                                                                                                   | border-box                | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | n/a                                                                                                         | discrete                  | per grammar     | as specified                                                                                                                                                               | visual |
| <strong><span><a id="ref-for-propdef-mask-position①①"></a></span><a href="#propdef-mask-position">mask-position</a>&#xA;      </strong> | \<position\>#                                                                                                                                                                       | 0% 0%                     | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | refer to size of mask painting area minus size of mask layer image; see text background-position \[CSS3BG\] | repeatable list           | per grammar     | Consisting of: two keywords representing the origin and two offsets from that origin, each given as an absolute length (if given a \<length\>), otherwise as a percentage. | visual |
| <strong><span><a id="ref-for-propdef-mask-repeat⑦"></a></span><a href="#propdef-mask-repeat">mask-repeat</a>&#xA;      </strong> | \<repeat-style\>#                                                                                                                                                                   | repeat                    | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | n/a                                                                                                         | discrete                  | per grammar     | Consists of: two keywords, one per dimension                                                                                                                               | visual |
| <strong><span><a id="ref-for-propdef-mask-size⑧"></a></span><a href="#propdef-mask-size">mask-size</a>&#xA;      </strong> | \<bg-size\>#                                                                                                                                                                        | auto                      | All elements. In SVG, it applies to container elements excluding the defs element, all graphics elements and the use element       | no   | n/a                                                                                                         | repeatable list           | per grammar     | as specified, but with lengths made absolute                                                                                                                               | visual |
| <strong><span><a id="ref-for-propdef-mask-type①①"></a></span><a href="#propdef-mask-type">mask-type</a>&#xA;      </strong> | luminance \| alpha                                                                                                                                                                  | luminance                 | mask elements                                                                                                                      | no   | n/a                                                                                                         | discrete                  | per grammar     | as specified                                                                                                                                                               | visual |

## <a id="idl-index"></a>IDL Index

```text
[Exposed=Window]
interface SVGClipPathElement : SVGElement {
  readonly attribute SVGAnimatedEnumeration clipPathUnits;
  readonly attribute SVGAnimatedTransformList transform;
};

[Exposed=Window]
interface SVGMaskElement : SVGElement {
  readonly attribute SVGAnimatedEnumeration maskUnits;
  readonly attribute SVGAnimatedEnumeration maskContentUnits;
  readonly attribute SVGAnimatedLength x;
  readonly attribute SVGAnimatedLength y;
  readonly attribute SVGAnimatedLength width;
  readonly attribute SVGAnimatedLength height;
};

```
## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Firefox disables rendering of elements referencing clipPaths with violated content model. No browser ignores clipPath on use with indirect reference. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;17&#x3E;](https://github.com/w3c/csswg-drafts/issues/17) [↵](#issue-d0c561ed)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Define raw geometry with regards to CSS properties that affect it. Especially on text. [&#x3C;https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;w3c&#x2F;csswg-drafts&#x2F;issues&#x2F;170&#x3E;](https://github.com/w3c/csswg-drafts/issues/170) [↵](#issue-1eaffdcc)
