Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Transforms Module Level 2](https://www.w3.org/TR/2021/WD-css-transforms-2-20211109/).

Original copyright notice: Copyright © 2021 W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Transforms Module Level 2

Source snapshot: https://www.w3.org/TR/2021/WD-css-transforms-2-20211109/

Snapshot SHA-256: f1ada25106706968fc7b12ecb83bebc437de6ef309ee9dce1d990bd5fe8eead8

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 8 source tables are presented as readable Markdown tables or explicit labeled layouts: 8 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Transforms Module Level 2

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2021 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

CSS transforms allows elements styled with CSS to be transformed in two-dimensional or three-dimensional space.

This spec adds new transform functions and properties for three-dimensional transforms, and convenience functions for simple transforms.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes).

Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="intro"></a>1. Introduction

<a id="ref-for-propdef-transform"></a>

This specification is a delta spec that extends [\[css-transforms-1\]](#biblio-css-transforms-1) to allow authors to transform elements in three-dimensional space. New transform functions for the [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) property allow three-dimensional transforms, and additional properties make working with three-dimensional transforms easier, and allow the author to control how nested three-dimensional transformed elements interact.

1.  <a id="ref-for-propdef-perspective"></a>

    <a id="ref-for-propdef-perspective-origin"></a>

    The [perspective](#propdef-perspective) property allows the author to provide child elements with an extra perspective transformation. The [perspective-origin](#propdef-perspective-origin) property provides control over the origin at which perspective is applied, effectively changing the location of the "vanishing point".

2.  <a id="ref-for-propdef-transform-style"></a>

    The [transform-style](#propdef-transform-style) property allows 3D-transformed elements and their 3D-transformed descendants to share a common three-dimensional space, allowing the construction of hierarchies of three-dimensional objects.

3.  <a id="ref-for-propdef-backface-visibility"></a>

    The [backface-visibility](#propdef-backface-visibility) property comes into play when an element is flipped around via three-dimensional transforms such that its reverse side is visible to the viewer. In some situations it is desirable to hide the element in this situation, which is possible using the value of hidden for this property.

<a id="ref-for-propdef-transform①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: While some values of the [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) property allow an element to be transformed in a three-dimensional coordinate system, the elements themselves are not three-dimensional objects. Instead, they exist on a two-dimensional plane (a flat surface) and have no depth.

<a id="ref-for-propdef-scale"></a>

<a id="ref-for-propdef-translate"></a>

<a id="ref-for-propdef-rotate"></a>

This specification also adds three convenience properties, [scale](#propdef-scale), [translate](#propdef-translate) and [rotate](#propdef-rotate), that make it easier to describe and animate simple transforms.

### <a id="module-interactions"></a>1.1. Module Interactions

<a id="ref-for-3d-transform-functions"></a>

<a id="ref-for-propdef-transform②"></a>

The [3D transform functions](#3d-transform-functions) here extend the set of functions for the [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) property.

<a id="ref-for-propdef-perspective①"></a>

<a id="ref-for-propdef-transform-style①"></a>

<a id="ref-for-propdef-backface-visibility①"></a>

<a id="ref-for-containing-block-for-all-descendants"></a>

<a id="ref-for-x43"></a>

Some values of [perspective](#propdef-perspective), [transform-style](#propdef-transform-style) and [backface-visibility](#propdef-backface-visibility) result in the creation of a [containing block for all descendants](https://www.w3.org/TR/css-transforms-1/#containing-block-for-all-descendants), and/or the creation of a [stacking context](https://www.w3.org/TR/CSS2/visuren.html#x43).

Three-dimensional transforms affect the visual layering of elements, and thus override the back-to-front painting order described in [Appendix E](https://www.w3.org/TR/CSS2/zindex.html) of [\[CSS21\]](#biblio-css21).

### <a id="values"></a>1.2. Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS21\]](#biblio-css21) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="terminology"></a>2. Terminology

<a id="3d-transformed-element"></a>3D transformed element  
<a id="ref-for-propdef-transform③"></a>

<a id="ref-for-3d-transform-functions①"></a>

An element whose computed value for the [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) property includes one of the [3D transform functions](#3d-transform-functions)

<a id="3d-matrix"></a>3D matrix  
<a id="ref-for-2d-matrix"></a>

A 4x4 matrix which does not fulfill the requirements of an [2D matrix](https://www.w3.org/TR/css-transforms-1/#2d-matrix).

<a id="identity-transform-function"></a>identity transform function  
In addition to the identity transform function in CSS Transforms, examples for identity transform functions include translate3d(0, 0, 0), translateZ(0), scaleZ(1), rotate3d(1, 1, 1, 0), rotateX(0), rotateY(0), rotateZ(0) and matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1). A special case is perspective: perspective(none). The value of m<sub>34</sub> becomes infinitesimal small and the transform function is therefore assumed to be equal to the identity matrix.

<a id="perspective-matrix"></a>perspective matrix  
<a id="ref-for-propdef-perspective②"></a>

<a id="ref-for-propdef-perspective-origin①"></a>

A matrix computed from the values of the [perspective](#propdef-perspective) and [perspective-origin](#propdef-perspective-origin) properties as described [below](#perspective-matrix-computation).

<a id="accumulated-3d-transformation-matrix"></a>accumulated 3D transformation matrix  
<a id="ref-for-3d-rendering-context"></a>

A matrix computed for an element relative to the root of its [3D rendering context](#3d-rendering-context), as described [below](#accumulated-3d-transformation-matrix-computation).

<a id="3d-rendering-context"></a>3D rendering context  
A set of elements with a common ancestor which share a common three-dimensional coordinate system, as described [below](#3d-rendering-contexts).

<a id="ref-for-computed-value"></a>

<a id="ref-for-typedef-transform-list"></a>

### <a id="serialization-of-the-computed-value"></a>2.1. Serialization of the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [\<transform-list\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-list)

<a id="ref-for-typedef-transform-list①"></a>

<a id="ref-for-computed-value①"></a>

<a id="ref-for-funcdef-transform-matrix"></a>

<a id="ref-for-funcdef-matrix3d"></a>

A [\<transform-list\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-list) for the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) is serialized to either one [\<matrix()\>](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-matrix) or one [\<matrix3d()\>](#funcdef-matrix3d) function by the following algorithm:

1.  Let <var>transform</var> be a 4x4 matrix initialized to the identity matrix. The elements <var> m11</var>, <var>m22</var>, <var>m33</var> and <var>m44</var> of <var>transform</var> must be set to 1 all other elements of <var>transform</var> must be set to 0.

2.  <a id="ref-for-typedef-transform-function"></a>

    <a id="ref-for-typedef-transform-list②"></a>

    Post-multiply all [\<transform-function\>](#typedef-transform-function)s in [\<transform-list\>](https://www.w3.org/TR/css-transforms-1/#typedef-transform-list) to <var>transform</var>.

3.  <a id="ref-for-funcdef-transform-matrix①"></a>

    <a id="ref-for-funcdef-matrix3d①"></a>

    Chose between [\<matrix()\>](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-matrix) or [\<matrix3d()\>](#funcdef-matrix3d) serialization:

    <a id="ref-for-2d-matrix①"></a>

    If <var>transform</var> is a [2D matrix](https://www.w3.org/TR/css-transforms-1/#2d-matrix)

    <a id="ref-for-funcdef-transform-matrix②"></a>

    Serialize <var>transform</var> to a [\<matrix()\>](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-matrix) function.

    Otherwise

    <a id="ref-for-funcdef-matrix3d②"></a>

    Serialize <var>transform</var> to a [\<matrix3d()\>](#funcdef-matrix3d) function.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-8241ab72"></a> fix this text to add to the text in CSS Transforms 1.

## <a id="two-dimensional-subset"></a>3. Two Dimensional Subset

<a id="ref-for-propdef-transform-style②"></a>

<a id="ref-for-propdef-perspective③"></a>

<a id="ref-for-propdef-perspective-origin②"></a>

<a id="ref-for-propdef-backface-visibility②"></a>

UAs may not always be able to render three-dimensional transforms and then just support a two-dimensional subset of this specification. In this case [three-dimensional transforms](#three-d-transform-functions) and the properties [transform-style](#propdef-transform-style), [perspective](#propdef-perspective), [perspective-origin](#propdef-perspective-origin) and [backface-visibility](#propdef-backface-visibility) must not be supported. Section [3D Transform Rendering](#3d-transform-rendering) does not apply. Matrix decomposing uses the technique taken from the "unmatrix" method in "Graphics Gems II, edited by Jim Arvo", simplified for the 2D case. Section [Mathematical Description of Transform Functions](#mathematical-description) is still effective but can be reduced by using a 3x3 transformation matrix where <em>a</em> equals m<sub>11</sub>, <em>b</em> equals m<sub>12</sub>, <em>c</em> equals m<sub>21</sub>, <em>d</em> equals m<sub>22</sub>, <em>e</em> equals m<sub>41</sub> and <em>f</em> equals m<sub>42</sub> (see A 2D 3x2 matrix with six parameter).

\$\$&#x5C;begin{bmatrix} a &#x26; c &#x26; e &#x5C;&#x5C; b &#x26; d &#x26; f &#x5C;&#x5C; 0 &#x26; 0 &#x26; 1 &#x5C;end{bmatrix}\$\$

3x3 matrix for two-dimensional transformations.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d57c1ef5"></a>
>
> <a id="ref-for-propdef-transform④"></a>
>
> Authors can easily provide a fallback if UAs do not provide support for three-dimensional transforms. The following example has two property definitions for [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform). The first one consists of two two-dimensional transform functions. The second one has a two-dimensional and a three-dimensional transform function.
>
> ```text
> div {  transform: scale(2) rotate(45deg);
>   transform: scale(2) rotate3d(0, 0, 1, 45deg);
> }
> ```
>
> With 3D support, the second definition will override the first one. Without 3D support, the second definition is invalid and a UA falls back to the first definition.

## <a id="transform-rendering"></a>4. The Transform Rendering Model

<a id="ref-for-propdef-transform-origin"></a>

<a id="ref-for-propdef-perspective④"></a>

This specification extends [CSS Transforms 1 § 3 The Transform Rendering Model](https://www.w3.org/TR/css-transforms-1/#transform-rendering) to account for the existence of three-dimensional transform functions, the Z value of [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin), the [perspective](#propdef-perspective) property, and a new 3D rendering model that applies when the used value of the transform-style property is preserve-3d.

Three-dimensional transform functions conceptually extend the coordinate space into three dimensions, adding a Z axis perpendicular to the plane of the screen, that increases towards the viewer.

![Demonstration of the initial coordinate space](https://www.w3.org/TR/2021/WD-css-transforms-2-20211109/images/coordinates.svg)

Demonstration of the initial coordinate space.

<a id="ref-for-propdef-transform-origin①"></a>

<a id="ref-for-transformation-matrix"></a>

<a id="ref-for-propdef-transform⑤"></a>

<a id="transformation-matrix-computation"></a> With 3D transforms, the Z component of [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin) affects the result, so the [transformation matrix](https://www.w3.org/TR/css-transforms-1/#transformation-matrix) is computed from the [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) and <a id="ref-for-propdef-transform-origin②"></a>transform-origin properties as follows:

1.  Start with the identity matrix.

2.  <a id="ref-for-propdef-transform-origin③"></a>

    Translate by the computed X, Y and Z of [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin)

3.  <a id="ref-for-propdef-transform⑥"></a>

    Multiply by each of the transform functions in [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) property from left to right

4.  <a id="ref-for-propdef-transform-origin④"></a>

    Translate by the negated computed X, Y and Z values of [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin)

### <a id="3d-transform-rendering"></a>4.1. 3D Transform Rendering

Normally, elements render as flat planes, and are rendered into the same plane as their stacking context. Often this is the plane shared by the rest of the page. Two-dimensional transform functions can alter the appearance of an element, but that element is still rendered into the same plane as its stacking context.

<a id="ref-for-3d-rendering-context①"></a>

An element with a three-dimensional transform that is not contained in a [3D rendering context](#3d-rendering-context) renders with the appropriate transform applied, but does not intersect with any other elements. The three-dimensional transform in this case can be considered just as a painting effect, like two-dimensional transforms. Similarly, the transform does not affect painting order. For example, a transform with a positive Z translation may make an element look larger, but does not cause that element to render in front of elements with no translation in Z.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-aae09890"></a> describe how nested 3d-transformed elements render (perhaps with math)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-20b12368"></a>
>
> > <strong data-conversion-semantic="issue">Issue</strong>
> >
> > <a id="issue-699eabea"></a> This example doesn’t follow from the previous text.
>
> This example shows the effect of three-dimensional transform applied to an element.
>
> ```text
> <style>
> div {
>     height: 150px;
>     width: 150px;
> }
> .container {
>     border: 1px solid black;
> }
> .transformed {
>     transform: rotateY(50deg);
> }
> </style>
> 
> <div class="container">
>     <div class="transformed"></div>
> </div>
> ```
>
> ![Div with a rotateY transform.](https://www.w3.org/TR/2021/WD-css-transforms-2-20211109/examples/simple-3d-example.png)
>
> The transform is a 50° rotation about the vertical, Y axis. Note how this makes the blue box appear narrower, but not three-dimensional.

#### <a id="perspective"></a>4.1.1. Perspective

<a id="ref-for-propdef-perspective⑤"></a>

Perspective can be used to add a feeling of depth to a scene by making elements higher on the Z axis (closer to the viewer) appear larger, and those further away to appear smaller. The scaling is proportional to <var>d</var>/(<var>d</var> − <var>Z</var>) where <var>d</var>, the value of [perspective](#propdef-perspective), is the distance from the drawing plane to the assumed position of the viewer’s eye.

<a id="ref-for-funcdef-perspective"></a>

The appearance of perspective can be applied to a 3d-transformed element in two ways. First, the element’s 'transform function list' can contain the [perspective()](#funcdef-perspective) function which computes into the element’s 'current transformation matrix'.

<a id="ref-for-propdef-perspective⑥"></a>

<a id="ref-for-propdef-perspective-origin③"></a>

Second, the [perspective](#propdef-perspective) and [perspective-origin](#propdef-perspective-origin) properties can be applied to an element to influence the rendering of its 3d-transformed children, giving them a shared perspective that provides the impression of them living in the same three-dimensional scene.

![Diagram of scale vs. Z position](https://www.w3.org/TR/2021/WD-css-transforms-2-20211109/images/perspective_distance.png)

<a id="ref-for-propdef-perspective⑦"></a>

Diagrams showing how scaling depends on the [perspective](#propdef-perspective) property and Z position. In the top diagram, <var>Z</var> is half of <var>d</var>. In order to make it appear that the original circle (solid outline) appears at <var>Z</var> (dashed circle), the circle is scaled up by a factor of two, resulting in the light blue circle. In the bottom diagram, the circle is scaled down by a factor of one-third to make it appear behind the original position.

<a id="ref-for-propdef-perspective-origin④"></a>

Normally the assumed position of the viewer’s eye is centered on a drawing. This position can be moved if desired – for example, if a web page contains multiple drawings that should share a common perspective – by setting [perspective-origin](#propdef-perspective-origin).

![Diagram of different perspective-origin](https://www.w3.org/TR/2021/WD-css-transforms-2-20211109/images/perspective_origin.png)

Diagram showing the effect of moving the perspective origin upward.

<a id="ref-for-perspective-matrix"></a>

<a id="perspective-matrix-computation"></a> The [perspective matrix](#perspective-matrix) is computed as follows:

1.  Start with the identity matrix.

2.  <a id="ref-for-propdef-perspective-origin⑤"></a>

    Translate by the computed X and Y values of [perspective-origin](#propdef-perspective-origin)

3.  <a id="ref-for-funcdef-perspective①"></a>

    <a id="ref-for-propdef-perspective⑧"></a>

    Multiply by the matrix that would be obtained from the [perspective()](#funcdef-perspective) transform function, where the length is provided by the value of the [perspective](#propdef-perspective) property

4.  <a id="ref-for-propdef-perspective-origin⑥"></a>

    Translate by the negated computed X and Y values of [perspective-origin](#propdef-perspective-origin)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6ad0c4cd"></a>
>
> This example shows how perspective can be used to cause three-dimensional transforms to appear more realistic.
>
> ```text
> <style>
> div {
>   height: 150px;
>   width: 150px;
> }
> .container {
>   perspective: 500px;
>   border: 1px solid black;
> }
> .transformed {
>   transform: rotateY(50deg);
> }
> </style>
> 
> <div class="container">
>   <div class="transformed"></div>
> </div>
> ```
>
> ![Div with a rotateY transform, and perspective on its container](https://www.w3.org/TR/2021/WD-css-transforms-2-20211109/examples/simple-perspective-example.png)
>
> The inner element has the same transform as in the previous example, but its rendering is now influenced by the perspective property on its parent element. Perspective causes vertices that have positive Z coordinates (closer to the viewer) to be scaled up in X and Y, and those further away (negative Z coordinates) to be scaled down, giving an appearance of depth.

#### <a id="3d-rendering-contexts"></a>4.1.2. 3D Rendering Contexts

This section specifies the rendering model for content that uses 3D-transforms and the transform-style property. In order to describe this model, we introduce the concept of a "3D rendering context".

<a id="ref-for-3d-rendering-context②"></a>

A [3D rendering context](#3d-rendering-context) is a set of elements rooted in a common ancestor that, for the purposes of 3D-transform rendering, are considered to share a common three-dimensional coordinate system. The front-to-back rendering of elements in the a 3D rendering context depends on their z-position in that three-dimensional space, and, if the 3D transforms on those elements cause them to intersect, then they are rendered with intersection.

<a id="ref-for-accumulated-3d-transformation-matrix"></a>

<a id="ref-for-3d-rendering-context③"></a>

The position of each element in that three-dimensional space is determined by [accumulating](#accumulated-3d-transformation-matrix) the transformation matrices up from the given element to the element that establishes the [3D rendering context](#3d-rendering-context).

Elements establish and participate in 3D rendering contexts as follows:

- <a id="ref-for-3d-rendering-context④"></a>

  <a id="ref-for-transformable-element"></a>

  <a id="ref-for-propdef-transform-style③"></a>

  A [3D rendering context](#3d-rendering-context) is established by a [transformable element](https://www.w3.org/TR/css-transforms-1/#transformable-element) whose used value for [transform-style](#propdef-transform-style) is preserve-3d and which itself is not part of a 3D rendering context. An element that establishes a 3D rendering context also participates in that context.

- <a id="ref-for-propdef-transform-style④"></a>

  <a id="ref-for-3d-rendering-context⑤"></a>

  An element whose used value for [transform-style](#propdef-transform-style) is preserve-3d and which itself participates in a [3D rendering context](#3d-rendering-context), extends that 3D rendering context rather than establishing a new one.

- <a id="ref-for-3d-rendering-context⑥"></a>

  An element participates in a [3D rendering context](#3d-rendering-context) if its parent establishes or extends a <a id="ref-for-3d-rendering-context⑦"></a>3D rendering context.

<a id="ref-for-3d-rendering-context⑧"></a>

Some CSS properties have values that are considered to force "grouping": they require that their element and its descendants are rendered as a group before being composited with other elements; these include opacity, filters and properties that affect clipping. The relevant property values are listed under [grouping property values](#grouping-property-values). Consequently, when used on an element with transform-style:preserve-3d, they change the used value to flat and prevent it from creating or extending a [3D rendering context](#3d-rendering-context).

In a 3D rendering context, rendering and sorting of elements is done as follows:

1.  The element establishing the 3D rendering context, and each other 3D transformed element participating in the 3D rendering context, is rendered into its own plane. This plane includes the element’s backgrounds, borders, other box decorations, content, and descendant elements, excluding any descendant elements that have their own plane (and their descendants). This rendering is done according to [CSS 2.1, Appendix E, Section E.2 Painting Order](https://www.w3.org/TR/CSS2/zindex.html#painting-order).

2.  <a id="ref-for-accumulated-3d-transformation-matrix①"></a>

    <a id="ref-for-3d-transformed-element"></a>

    Intersection is performed between this set of planes, according to [Newell’s algorithm](https://en.wikipedia.org/wiki/Newell%27s_algorithm), with the planes transformed by the [accumulated 3D transformation matrix](#accumulated-3d-transformation-matrix). Coplanar [3D transformed elements](#3d-transformed-element) are rendered in painting order.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d667e5bf"></a> is it OK to not pop 2D-transformed elements into their own planes?

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification previously defined that the background, borders, and other box decorations of the establishing element were rendered behind the entire 3D scene. This was changed in [\#6238](https://github.com/w3c/csswg-drafts/issues/6238). However, if the definition of 3D Rendering Contexts is changed in the future, it may be worth considering changing back.

<a id="ref-for-3d-transformed-element①"></a>

Note that elements with transforms which have a negative z-component will render behind the content and untransformed descendants of the establishing element, and that [3D transformed elements](#3d-transformed-element) may interpenetrate with content and untransformed elements.

<a id="ref-for-3d-transformed-element②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because the 3D-transformed elements in a 3D rendering context can all depth-sort and intersect with each other, they are effectively rendered as if they were siblings. The effect of transform-style: preserve-3d can then be thought of as causing all the [3D transformed elements](#3d-transformed-element) in a 3D rendering context to be hoisted up into the establishing element, but still rendered with their [accumulated 3D transformation matrix](#accumulated-3d-transformation-matrix-computation).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0bb57155"></a>
>
> ```text
> <style>
> div {
>   height: 150px;
>   width: 150px;
> }
> .scene {
>   background-color: rgba(0, 0, 0, 0.3);
>   border: 1px solid black;
>   perspective: 500px;
> }
> .container {
>   transform-style: preserve-3d;
> }
> .container > div {
>   position: absolute;
>   left: 0;
> }
> .container > :first-child {
>   transform: rotateY(45deg);
>   background-color: orange;
>   top: 10px;
>   height: 135px;
> }
> .container > :last-child {
>   transform: translateZ(40px);
>   background-color: rgba(0, 0, 255, 0.6);
>   top: 50px;
>   height: 100px;
> }
> </style>
> 
> <div class="scene">
>   <div class="container">
>     Lorem ipsum dolor sit amet, consectetaur adipisicing elit…
>     <div></div>
>     <div></div>
>   </div>
> </div>
> ```
>
> This example shows show elements in a 3D rendering context can intersect. The container element establishes a 3D rendering context for itself and its two children, and the scene element adds perspective to the 3D rendering context. The children intersect with each other, and the orange element also intersects with the container.
>
> ![Intersecting sibling elements.](https://www.w3.org/TR/2021/WD-css-transforms-2-20211109/examples/3d-intersection.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3892c5d2"></a>
>
> ```text
> <style>
> div {
>   height: 150px;
>   width: 150px;
> }
> .container {
>   perspective: 500px;
>   border: 1px solid black;
> }
> .transformed {
>   transform: rotateY(50deg);
>   background-color: blue;
> }
> .child {
>   transform-origin: top left;
>   transform: rotateX(40deg);
>   background-color: lime;
> }
> </style>
> 
> <div class="container">
>   <div class="transformed">
>     <div class="child"></div>
>   </div>
> </div>
> ```
>
> This example shows how nested 3D transforms are rendered. The blue div is transformed as in the previous example, with its rendering influenced by the perspective on its parent element. The lime element also has a 3D transform, which is a rotation about the X axis (anchored at the top, by virtue of the transform-origin). However, the lime element is being rendered into the plane of its parent because it is not a member of the same 3D rendering context. Thus the lime element only appears shorter; it does not "pop out" of the blue element.
>
> ![Nested 3D transforms, with flattening](https://www.w3.org/TR/2021/WD-css-transforms-2-20211109/examples/3d-rendering-context-flat.png)

#### <a id="transformed-element-hierarchies"></a>4.1.3. Transformed element hierarchies

<a id="ref-for-transformed-element"></a>

<a id="ref-for-3d-rendering-context⑨"></a>

By default, [transformed elements](https://www.w3.org/TR/css-transforms-1/#transformed-element) do not create a [3D rendering context](#3d-rendering-context) and create a flattened representation of their content. However, since it is useful to construct hierarchies of transformed objects that share a common 3-dimensional space, this flattening behavior may be overridden by specifying a value of preserve-3d for the transform-style property. This allows descendants of the transformed element to share the same 3D rendering context. Non-3D-transformed descendants of such elements are rendered into the plane of the element in step C above, but 3D-transformed elements in the same 3D rendering context will "pop out" into their own planes.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1069e278"></a>
>
> ```text
> <style>
> div {
>   height: 150px;
>   width: 150px;
> }
> .container {
>   perspective: 500px;
>   border: 1px solid black;
> }
> .transformed {
>   transform-style: preserve-3d;
>   transform: rotateY(50deg);
>   background-color: blue;
> }
> .child {
>   transform-origin: top left;
>   transform: rotateX(40deg);
>   background-color: lime;
> }
> </style>
> ```
>
> <a id="ref-for-propdef-transform-style⑤"></a>
>
> This example is identical to the previous example, with the addition of [transform-style: preserve-3d](#propdef-transform-style) on the blue element. The blue element now extends the 3D rendering context of its container. Now both blue and lime elements share a common three-dimensional space, so the lime element renders as tilting out from its parent, influenced by the perspective on the container.
>
> ![Nested 3D transforms, with preserve-3d.](https://www.w3.org/TR/2021/WD-css-transforms-2-20211109/examples/3d-rendering-context-3d.png)

#### <a id="accumulated-3d-transformation-matrix-computation"></a>4.1.4. Accumulated 3D Transformation Matrix Computation

<a id="ref-for-3d-rendering-context①⓪"></a>

<a id="ref-for-accumulated-3d-transformation-matrix②"></a>

The final value of the transform used to render an element in a [3D rendering context](#3d-rendering-context) is computed by accumulating an [accumulated 3D transformation matrix](#accumulated-3d-transformation-matrix) as follows:

1.  Let <var>transform</var> be the identity matrix.

2.  Let <var>current element</var> be the transformed element.

3.  Let <var>parent element</var> be the parent element of the transformed element.

4.  <a id="ref-for-3d-rendering-context①①"></a>

    While <var>current element</var> is an element in the transformed element’s [3D rendering context](#3d-rendering-context):

    1.  <a id="ref-for-propdef-transform⑦"></a>

        <a id="ref-for-transformation-matrix①"></a>

        If <var>current element</var> has a value for [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) which is not none, pre-multiply <var>current element</var>’s [transformation matrix](https://www.w3.org/TR/css-transforms-1/#transformation-matrix) with the <var>transform</var>.

    2.  Compute a translation matrix which represents the offset (including the scroll offset) of <var>current element</var> from its <var>parent element</var>, and pre-multiply that matrix into the <var>transform</var>.

    3.  <a id="ref-for-propdef-perspective⑨"></a>

        <a id="ref-for-valdef-perspective-none"></a>

        <a id="ref-for-perspective-matrix①"></a>

        If <var>parent element</var> has a value for [perspective](#propdef-perspective) which is not [none](#valdef-perspective-none), pre-multiply the <var>parent element</var>’s [perspective matrix](#perspective-matrix) into the <var>transform</var>.

    4.  Let <var>current element</var> be the <var>parent element</var>.

    5.  Let <var>parent element</var> be the <var>current element</var>’s parent.

<a id="ref-for-accumulated-3d-transformation-matrix③"></a>

<a id="ref-for-3d-rendering-context①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: as described here, the [accumulated 3D transformation matrix](#accumulated-3d-transformation-matrix) takes into account offsets (including the scroll offset) generated by the [visual formatting model](https://www.w3.org/TR/CSS2/visuren.html) on the transformed element, and elements in its ancestor chain up to and including the element that establishes the its [3D rendering context](#3d-rendering-context).

#### <a id="backface-visibility"></a>4.1.5. Backface Visibility

<a id="ref-for-propdef-backface-visibility③"></a>

Using three-dimensional transforms, it’s possible to transform an element such that its reverse side is visible. 3D-transformed elements show the same content on both sides, so the reverse side looks like a mirror-image of the front side (as if the element were projected onto a sheet of glass). Normally, elements whose reverse side is towards the viewer remain visible. However, the [backface-visibility](#propdef-backface-visibility) property allows the author to make an element invisible when its reverse side is towards the viewer. This behavior is "live"; if an element with <a id="ref-for-propdef-backface-visibility④"></a>backface-visibility: hidden were animating, such that its front and reverse sides were alternately visible, then it would only be visible when the front side were towards the viewer.

<a id="ref-for-accumulated-3d-transformation-matrix④"></a>

<a id="ref-for-3d-rendering-context①③"></a>

Visibility of the reverse side of an element is considered using the [accumulated 3D transformation matrix](#accumulated-3d-transformation-matrix), and is thus relative to the parent of the element that establishes the [3D rendering context](#3d-rendering-context).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This property is useful when you place two elements back-to-back, as you would to create a playing card. Without this property, the front and back elements could switch places at times during an animation to flip the card. Another example is creating a box out of 6 elements, but where you want to see only the inside faces of the box.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-551edcaa"></a>
>
> This example shows how to make a "card" element that flips over when clicked. Note the "transform-style: preserve-3d" on \#card which is necessary to avoid flattening when flipped.
>
> ```text
> <style>
> .body { perspective: 500px; }
> #card {
>   position: relative;
>   height: 300px; width: 200px;
>   transition: transform 1s;
>   transform-style: preserve-3d;
> }
> #card.flipped {
>   transform: rotateY(180deg);
> }
> .face {
>   position: absolute;
>   top: 0; left: 0;
>   width: 100%; height: 100%;
>   background-color: silver;
>   border-radius: 40px;
>   backface-visibility: hidden;
> }
> .back {
>   transform: rotateY(180deg);
> }
> </style>
> <div id="card" onclick="this.classList.toggle('flipped')">
>   <div class="front face">Front</div>
>   <div class="back face">Back</div>
> </div>
> ```
> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-7b36039d"></a> what is the impact of backface-visibility on non-transformed or 2D-transformed elements? Do they get popped into their own planes and intersect?

### <a id="processing-of-perspective-transformed-boxes"></a>4.2. Processing of Perspective-Transformed Boxes

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d20360cc"></a> This is a first pass at an attempt to precisely specify how exactly to transform elements using the provided matrices. It might not be ideal, and implementer feedback is encouraged. See [\#912](https://github.com/w3c/csswg-drafts/issues/912).

<a id="ref-for-accumulated-3d-transformation-matrix⑤"></a>

The [accumulated 3D transformation matrix](#accumulated-3d-transformation-matrix) is affected both by the perspective property, and by any perspective() transform function present in the value of the transform property.

<a id="ref-for-accumulated-3d-transformation-matrix⑥"></a>

This [accumulated 3D transformation matrix](#accumulated-3d-transformation-matrix) is a 4×4 matrix, while the objects to be transformed are two-dimensional boxes. To transform each corner (<var>a</var>, <var>b</var>) of a box, the matrix must first be applied to (<var>a</var>, <var>b</var>, 0, 1), which will result in a four-dimensional point (<var>x</var>, <var>y</var>, <var>z</var>, <var>w</var>). This is transformed back to a three-dimensional point (<var>x</var>′, <var>y</var>′, <var>z</var>′) as follows:

If <var>w</var> \> 0, (<var>x</var>′, <var>y</var>′, <var>z</var>′) = (<var>x</var>/<var>w</var>, <var>y</var>/<var>w</var>, <var>z</var>/<var>w</var>).

If <var>w</var> = 0, (<var>x</var>′, <var>y</var>′, <var>z</var>′) = (<var>x</var> ⋅ <var>n</var>, <var>y</var> ⋅ <var>n</var>, <var>z</var> ⋅ <var>n</var>). <var>n</var> is an implementation-dependent value that should be chosen so that <var>x</var>′ or <var>y</var>′ is much larger than the viewport size, if possible. For example, (5px, 22px, 0px, 0) might become (5000px, 22000px, 0px), with <var>n</var> = 1000, but this value of <var>n</var> would be too small for (0.1px, 0.05px, 0px, 0). This specification does not define the value of <var>n</var> exactly. Conceptually, (<var>x</var>′, <var>y</var>′, <var>z</var>′) is [infinitely far](https://en.wikipedia.org/wiki/Plane_at_infinity) in the direction (<var>x</var>, <var>y</var>, <var>z</var>).

If <var>w</var> \< 0 for all four corners of the transformed box, the box is not rendered.

If <var>w</var> \< 0 for one to three corners of the transformed box, the box must be replaced by a polygon that has any parts with <var>w</var> \< 0 cut out. This will in general be a polygon with three to five vertices, of which exactly two will have <var>w</var> = 0 and the rest <var>w</var> \> 0. These vertices are then transformed to three-dimensional points using the rules just stated. Conceptually, a point with <var>w</var> \< 0 is "behind" the viewer, so should not be visible.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b7386d44"></a>
>
> ```text
> .transformed {
>   height: 100px;
>   width: 100px;
>   background: lime;
>   transform: perspective(50px) translateZ(100px);
> }
> ```
>
> All of the box’s corners have <var>z</var>-coordinates greater than the perspective. This means that the box is behind the viewer and will not display. Mathematically, the point (<var>x</var>, <var>y</var>) first becomes (<var>x</var>, <var>y</var>, 0, 1), then is translated to (<var>x</var>, <var>y</var>, 100, 1), and then applying the perspective results in (<var>x</var>, <var>y</var>, 100, −1). The <var>w</var>-coordinate is negative, so it does not display. An implementation that doesn’t handle the <var>w</var> \< 0 case separately might incorrectly display this point as (−<var>x</var>, −<var>y</var>, −100), dividing by −1 and mirroring the box.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-81d81b28"></a>
>
> ```text
> .transformed {
>   height: 100px;
>   width: 100px;
>   background: radial-gradient(yellow, blue);
>   transform: perspective(50px) translateZ(50px);
> }
> ```
>
> Here, the box is translated upward so that it sits at the same place the viewer is looking from. This is like bringing the box closer and closer to one’s eye until it fills the entire field of vision. Since the default transform-origin is at the center of the box, which is yellow, the screen will be filled with yellow.
>
> Mathematically, the point (<var>x</var>, <var>y</var>) first becomes (<var>x</var>, <var>y</var>, 0, 1), then is translated to (<var>x</var>, <var>y</var>, 50, 1), then becomes (<var>x</var>, <var>y</var>, 50, 0) after applying perspective. Relative to the transform-origin at the center, the upper-left corner was (−50, −50), so it becomes (−50, −50, 50, 0). This is transformed to something very far to the upper left, such as (−5000, −5000, 5000). Likewise the other corners are sent very far away. The radial gradient is stretched over the whole box, now enormous, so the part that’s visible without scrolling should be the color of the middle pixel: yellow. However, since the box is not actually infinite, the user can still scroll to the edges to see the blue parts.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-834b4b29"></a>
>
> ```text
> .transformed {
>   height: 50px;
>   width: 50px;
>   background: lime;
>   border: 25px solid blue;
>   transform-origin: left;
>   transform: perspective(50px) rotateY(-45deg);
> }
> ```
>
> The box will be rotated toward the viewer, with the left edge staying fixed while the right edge swings closer. The right edge will be at about <var>z</var> = 70.7px, which is closer than the perspective of 50px. Therefore, the rightmost edge will vanish ("behind" the viewer), and the visible part will stretch out infinitely far to the right.
>
> Mathematically, the top right vertex of the box was originally (100, −50), relative to the transform-origin. It is first expanded to (100, −50, 0, 1). After applying the transform specified, this will get mapped to about (70.71, −50, 70.71, −0.4142). This has <var>w</var> = −0.4142 \< 0, so we need to slice away the part of the box with <var>w</var> \< 0. This results in the new top-right vertex being (50, −50, 50, 0). This is then mapped to some faraway point in the same direction, such as (5000, −5000, 5000), which is up and to the right from the transform-origin. Something similar is done to the lower right corner, which gets mapped far down and to the right. The resulting box stretches far past the edge of the screen.
>
> Again, the rendered box is still finite, so the user can scroll to see the whole thing if he or she chooses. However, the right part has been chopped off. No matter how far the user scrolls, the rightmost 30px or so of the original box will not be visible. The blue border was only 25px wide, so it will be visible on the left, top, and bottom, but not the right.
>
> The same basic procedure would apply if one or three vertices had <var>w</var> \< 0. However, in that case the result of truncating the <var>w</var> \< 0 part would be a triangle or pentagon instead of a quadrilateral.

<a id="ref-for-propdef-translate①"></a>

<a id="ref-for-propdef-scale①"></a>

<a id="ref-for-propdef-rotate①"></a>

## <a id="individual-transforms"></a>5. Individual Transform Properties: the [translate](#propdef-translate), [scale](#propdef-scale), and [rotate](#propdef-rotate) properties

<a id="ref-for-propdef-translate②"></a>

<a id="ref-for-propdef-rotate②"></a>

<a id="ref-for-propdef-scale②"></a>

<a id="ref-for-propdef-transform⑧"></a>

<a id="ref-for-funcdef-transform-translate"></a>

<a id="ref-for-funcdef-transform-rotate"></a>

<a id="ref-for-funcdef-scale"></a>

The [translate](#propdef-translate), [rotate](#propdef-rotate), and [scale](#propdef-scale) properties allow authors to specify simple transforms independently, in a way that maps to typical user interface usage, rather than having to remember the order in [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) that keeps the actions of [translate()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translate), [rotate()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-rotate) and [scale()](#funcdef-scale) independent and acting in screen coordinates.

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                    |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-translate"></a>translate                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt"></a><a id="ref-for-length-value"></a><a id="ref-for-typedef-length-percentage"></a><a id="ref-for-comb-one"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) \[ <a id="ref-for-typedef-length-percentage①"></a>\<length-percentage\> [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)[?](https://www.w3.org/TR/css-values-4/#mult-opt) \]<a id="ref-for-mult-opt①"></a>? |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-transformable-element①"></a>[transformable elements](https://www.w3.org/TR/css-transforms-1/#transformable-element)                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-reference-box"></a>relative to the width of the [reference box](https://www.w3.org/TR/css-transforms-1/#reference-box) (for the first value) or the height (for the second value)                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage②"></a><a id="ref-for-valdef-translate-none"></a>the keyword [none](#valdef-translate-none) or a pair of computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values and an absolute length                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <a id="ref-for-valdef-translate-none①"></a>by computed value, but see below for [none](#valdef-translate-none)                                                                                                                                                                                                                                                                                                                        |

<a id="ref-for-propdef-translate③"></a>

The [translate](#propdef-translate) property accepts 1-3 values, each specifying a translation against one axis, in the order X, Y, then Z. When the second or third values are missing, they default to 0px.

<a id="ref-for-funcdef-transform-translate①"></a>

<a id="ref-for-funcdef-translate3d"></a>

If the third value is omitted or zero, this specifies a 2d translation, equivalent to the [translate()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translate) function. Otherwise, this specifies a 3d translation, equivalent to the [translate3d()](#funcdef-translate3d) function.

<a id="ref-for-resolved-value"></a>

<a id="ref-for-propdef-translate④"></a>

<a id="ref-for-computed-value②"></a>

<a id="ref-for-dom-window-getcomputedstyle"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [resolved value](https://www.w3.org/TR/cssom-1/#resolved-value) of the [translate](#propdef-translate) property is the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value), and thus <code><a href="https://www.w3.org/TR/cssom-1/#dom-window-getcomputedstyle">getComputedStyle()</a></code> includes percentage values in its results.

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-rotate"></a>rotate                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-all"></a><a id="ref-for-mult-num"></a><a id="ref-for-number-value"></a><a id="ref-for-angle-value"></a><a id="ref-for-comb-one①"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) <a id="ref-for-comb-one②"></a>\| \[ x <a id="ref-for-comb-one③"></a>\| y <a id="ref-for-comb-one④"></a>\| z <a id="ref-for-comb-one⑤"></a>\| [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)[{3}](https://www.w3.org/TR/css-values-4/#mult-num) \] [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) <a id="ref-for-angle-value①"></a>\<angle\> |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-transformable-element②"></a>[transformable elements](https://www.w3.org/TR/css-transforms-1/#transformable-element)                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-number-value①"></a><a id="ref-for-angle-value②"></a><a id="ref-for-valdef-translate-none②"></a>the keyword [none](#valdef-translate-none), or an [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) with an axis consisting of a list of three [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <a id="ref-for-valdef-translate-none③"></a>as SLERP, but see below for [none](#valdef-translate-none)                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |

<a id="ref-for-propdef-rotate③"></a>

The [rotate](#propdef-rotate) property accepts an angle to rotate an element, and optionally an axis to rotate it around.

<a id="ref-for-funcdef-rotatex"></a>

<a id="ref-for-funcdef-rotatey"></a>

<a id="ref-for-funcdef-rotatez"></a>

<a id="ref-for-funcdef-rotate3d"></a>

The axis can be specified with either the <a id="valdef-rotate-x"></a>x, <a id="valdef-rotate-y"></a>y, or <a id="valdef-rotate-z"></a>z keywords, which specify a rotation around that axis, equivalent to the [rotateX()](#funcdef-rotatex), [rotateY()](#funcdef-rotatey), and [rotateZ()](#funcdef-rotatez) transform functions. Alternately, the axis can be specified explicitly by giving three numbers representing the x, y, and z components of an origin-centered vector, equivalent to the [rotate3d()](#funcdef-rotate3d) function.

<a id="ref-for-angle-value③"></a>

<a id="ref-for-valdef-rotate-z"></a>

<a id="ref-for-funcdef-transform-rotate①"></a>

<a id="ref-for-propdef-rotate④"></a>

There is no difference in behavior between a rotation specified as an [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) alone and a rotation specified as being around the z-axis (whether by the [z](#valdef-rotate-z) keyword or by a vector whose first two components are zero and third component is positive); they are all 2d rotations equivalent to the [rotate()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-rotate) function. For example, [rotate: 30deg](#propdef-rotate), <a id="ref-for-propdef-rotate⑤"></a>rotate: z 30deg, and <a id="ref-for-propdef-rotate⑥"></a>rotate: 0 0 1 30deg are equivalent.

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                       |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-scale"></a>scale                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range"></a><a id="ref-for-percentage-value"></a><a id="ref-for-number-value②"></a><a id="ref-for-comb-one⑥"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) <a id="ref-for-comb-one⑦"></a>\| [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) \][{1,3}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-transformable-element③"></a>[transformable elements](https://www.w3.org/TR/css-transforms-1/#transformable-element)                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-number-value③"></a><a id="ref-for-valdef-translate-none④"></a>the keyword [none](#valdef-translate-none), or a list of 3 [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <a id="ref-for-valdef-translate-none⑤"></a>by computed value, but see below for [none](#valdef-translate-none)                                                                                                                                                                                                                                                                           |

<a id="ref-for-propdef-scale③"></a>

The [scale](#propdef-scale) property accepts 1-3 values, each specifying a scale along one axis, in order X, Y, then Z.

If the Y value is not given, then it defaults to being the same as the X value.

If the Z value is not given, then it defaults to 1.

<a id="ref-for-funcdef-scale①"></a>

<a id="ref-for-funcdef-scale3d"></a>

If the third value is omitted, 1, or 100%, this specifies a 2d scaling, equivalent to the [scale()](#funcdef-scale) function. Otherwise, this specifies a 3d scaling, equivalent to the [scale3d()](#funcdef-scale3d) function.

There is no difference in behavior between the third value being omitted and the third value being 1 or 100%.

<a id="ref-for-percentage-value①"></a>

<a id="ref-for-number-value④"></a>

<a id="ref-for-propdef-scale④"></a>

A [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) is equivalent to a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), for example [scale: 100%](#propdef-scale) is equivalent to <a id="ref-for-propdef-scale⑤"></a>scale: 1. Numbers are used during serialization of specified and computed values.

------------------------------------------------------------------------

<a id="ref-for-containing-block-for-all-descendants①"></a>

<a id="ref-for-propdef-translate⑤"></a>

All three properties accept (and default to) the value <a id="valdef-translate-none"></a>none, which produces no transform at all. In particular, this value does <em>not</em> trigger the creation of a stacking context or [containing block for all descendants](https://www.w3.org/TR/css-transforms-1/#containing-block-for-all-descendants), while all other values (including “identity” transforms like [translate: 0px](#propdef-translate)) create a stacking context and <a id="ref-for-containing-block-for-all-descendants②"></a>containing block for all descendants, per usual for transforms.

<a id="ref-for-propdef-translate⑥"></a>

<a id="ref-for-propdef-rotate⑦"></a>

<a id="ref-for-propdef-scale⑥"></a>

<a id="ref-for-valdef-translate-none⑥"></a>

When [translate](#propdef-translate), [rotate](#propdef-rotate) or [scale](#propdef-scale) are animating or transitioning, and the from value or to value (but not both) is [none](#valdef-translate-none), the value <a id="ref-for-valdef-translate-none⑦"></a>none is replaced by the equivalent identity value (0px for translate, 0deg for rotate, 1 for scale).

### <a id="individual-transform-serialization"></a>5.1. Serialization

Because these properties have two distinct modes of behavior (no transform versus transform), serialization must take this into account:

<a id="ref-for-propdef-translate⑦"></a>

for [translate](#propdef-translate)

If a translation is specified, the property must serialize with one through three values. (As usual, if the second and third values are 0px, the default, or if only the third value is 0px, then those 0px values must be omitted when serializing).

<a id="ref-for-valdef-translate-none⑧"></a>

It must serialize as the keyword [none](#valdef-translate-none) if and only if <a id="ref-for-valdef-translate-none⑨"></a>none was originally specified. (An identity transform does not count; it must serialize as 0px.)

<a id="ref-for-propdef-rotate⑧"></a>

for [rotate](#propdef-rotate)

<a id="ref-for-angle-value④"></a>

If a rotation about the z axis (that is, in 2D) is specified, the property must serialize as just an [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value).

If any other rotation is specified, the property must serialize with an axis specified. If the axis is parallel with the x or y axes, it must serialize as the appropriate keyword.

<a id="ref-for-valdef-translate-none①⓪"></a>

It must serialize as the keyword [none](#valdef-translate-none) if and only if <a id="ref-for-valdef-translate-none①①"></a>none was originally specified. (An identity transform does not count; it must serialize as 0deg.)

<a id="ref-for-propdef-scale⑦"></a>

for [scale](#propdef-scale)

If a scale is specified, the property must serialize with only one through three values. As usual, if the third value is 1, the default, then it is omitted when serializing. If the third value is omitted and the second value is the same as the first (the default), then the second value is also omitted when serializing.

<a id="ref-for-valdef-translate-none①②"></a>

It must serialize as the keyword [none](#valdef-translate-none) if and only if <a id="ref-for-valdef-translate-none①③"></a>none was originally specified. (An identity transform does not count; it must serialize as 1.)

## <a id="ctm"></a>6. Current Transformation Matrix

<a id="ref-for-transformation-matrix②"></a>

The [transformation matrix](https://www.w3.org/TR/css-transforms-1/#transformation-matrix) computation is amended to the following:

<a id="ref-for-propdef-transform⑨"></a>

<a id="ref-for-propdef-transform-origin⑤"></a>

<a id="ref-for-propdef-translate⑧"></a>

<a id="ref-for-propdef-rotate⑨"></a>

<a id="ref-for-propdef-scale⑧"></a>

<a id="ref-for-propdef-offset"></a>

The transformation matrix is computed from the [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform), [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin), [translate](#propdef-translate), [rotate](#propdef-rotate), [scale](#propdef-scale), and [offset](https://www.w3.org/TR/motion-1/#propdef-offset) properties as follows:

1.  Start with the identity matrix.

2.  <a id="ref-for-propdef-transform-origin⑥"></a>

    Translate by the computed X, Y, and Z values of [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin).

3.  <a id="ref-for-propdef-translate⑨"></a>

    Translate by the computed X, Y, and Z values of [translate](#propdef-translate).

4.  <a id="ref-for-angle-value⑤"></a>

    <a id="ref-for-propdef-rotate①⓪"></a>

    Rotate by the computed [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) about the specified axis of [rotate](#propdef-rotate).

5.  <a id="ref-for-propdef-scale⑨"></a>

    Scale by the computed X, Y, and Z values of [scale](#propdef-scale).

6.  <a id="ref-for-propdef-offset①"></a>

    Translate and rotate by the transform specified by [offset](https://www.w3.org/TR/motion-1/#propdef-offset).

7.  <a id="ref-for-propdef-transform①⓪"></a>

    Multiply by each of the transform functions in [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) from left to right.

8.  <a id="ref-for-propdef-transform-origin⑦"></a>

    Translate by the negated computed X, Y and Z values of [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin).

<a id="ref-for-propdef-transform-style⑥"></a>

## <a id="transform-style-property"></a>7. The [transform-style](#propdef-transform-style) Property

| Field               | Definition                                                                                                 |
|---------------------|------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-transform-style"></a>transform-style                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one⑧"></a>flat [\|](https://www.w3.org/TR/css-values-4/#comb-one) preserve-3d                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | flat                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-transformable-element④"></a>[transformable elements](https://www.w3.org/TR/css-transforms-1/#transformable-element) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                   |
| <strong>Used value:&#xA;      </strong> | flat if a [grouping property](#grouping-property-values) is present, specified keyword otherwise           |

<a id="ref-for-propdef-transform-style⑦"></a>

<a id="ref-for-transformable-element⑤"></a>

<a id="ref-for-containing-block-for-all-descendants③"></a>

<a id="ref-for-3d-rendering-context①④"></a>

A computed value of preserve-3d for [transform-style](#propdef-transform-style) on a [transformable element](https://www.w3.org/TR/css-transforms-1/#transformable-element) establishes both a stacking context and a [containing block for all descendants](https://www.w3.org/TR/css-transforms-1/#containing-block-for-all-descendants). If the used value is preserve-3d then it also establishes or extends a [3D rendering context](#3d-rendering-context).

### <a id="grouping-property-values"></a>7.1. Grouping property values

The following CSS property values require the user agent to create a flattened representation of the descendant elements before they can be applied, and therefore force the element to have a used style of flat for preserve-3d.

- <a id="ref-for-propdef-overflow"></a>

  <a id="ref-for-valdef-overflow-visible"></a>

  <a id="ref-for-valdef-overflow-clip"></a>

  [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow): any value other than [visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible) or [clip](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-clip).

- <a id="ref-for-propdef-opacity"></a>

  [opacity](https://www.w3.org/TR/css-color-4/#propdef-opacity): any value less than 1.

- <a id="ref-for-propdef-filter"></a>

  [filter](https://www.w3.org/TR/filter-effects-1/#propdef-filter): any value other than none.

- <a id="ref-for-propdef-clip"></a>

  <a id="ref-for-valdef-clip-auto"></a>

  [clip](https://www.w3.org/TR/css-masking-1/#propdef-clip): any value other than [auto](https://drafts.csswg.org/css2/#valdef-clip-auto).

- <a id="ref-for-propdef-clip-path"></a>

  [clip-path](https://www.w3.org/TR/css-masking-1/#propdef-clip-path): any value other than none.

- <a id="ref-for-propdef-isolation"></a>

  [isolation](https://www.w3.org/TR/compositing-1/#propdef-isolation): used value of isolate.

- <a id="ref-for-propdef-mask-image"></a>

  [mask-image](https://www.w3.org/TR/css-masking-1/#propdef-mask-image): any value other than none.

- <a id="ref-for-propdef-mask-border-source"></a>

  [mask-border-source](https://www.w3.org/TR/css-masking-1/#propdef-mask-border-source): any value other than none.

- <a id="ref-for-propdef-mix-blend-mode"></a>

  [mix-blend-mode](https://www.w3.org/TR/compositing-1/#propdef-mix-blend-mode): any value other than normal.

- <a id="ref-for-propdef-contain"></a>

  <a id="ref-for-valdef-contain-paint"></a>

  <a id="ref-for-paint-containment"></a>

  <a id="ref-for-used-value"></a>

  <a id="ref-for-propdef-contain①"></a>

  <a id="ref-for-propdef-content-visibility"></a>

  [contain](https://www.w3.org/TR/css-contain-1/#propdef-contain): [paint](https://www.w3.org/TR/css-contain-1/#valdef-contain-paint) and any other property/value combination that causes [paint containment](https://www.w3.org/TR/css-contain-1/#paint-containment). <strong data-conversion-semantic="note">Note:</strong> Note: this includes any property that affect the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of the [contain](https://www.w3.org/TR/css-contain-1/#propdef-contain) property, such as [content-visibility: hidden](https://drafts.csswg.org/css-contain-2/#propdef-content-visibility).

<a id="ref-for-propdef-perspective①⓪"></a>

## <a id="perspective-property"></a>8. The [perspective](#propdef-perspective) Property

| Field               | Definition                                                                                                                                                           |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-perspective"></a>perspective                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-length-value①"></a><a id="ref-for-comb-one⑨"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-transformable-element⑥"></a>[transformable elements](https://www.w3.org/TR/css-transforms-1/#transformable-element)                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-perspective-none①"></a>the keyword [none](#valdef-perspective-none) or an absolute length                                                                                |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                    |

<a id="ref-for-length-value②"></a>

<a id="valdef-perspective-length-0"></a>[\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value)

Distance to the center of projection.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-8678c096"></a> Verify that projection is the distance to the center of projection.

<a id="ref-for-length-value③"></a>

<a id="ref-for-propdef-perspective①①"></a>

As very small [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) values can produce bizarre rendering results and stress the numerical accuracy of transform calculations, values less than 1px must be treated as 1px for rendering purposes. (This clamping does not affect the underlying value, so [perspective: 0;](#propdef-perspective) in a stylesheet will still serialize back as 0.)

<a id="valdef-perspective-none"></a>none

<a id="ref-for-length-value④"></a>

No perspective transform is applied. The effect is mathematically similar to an infinite [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) value. All objects appear to be flat on the canvas.

<a id="ref-for-valdef-perspective-none②"></a>

<a id="ref-for-containing-block-for-all-descendants④"></a>

<a id="ref-for-propdef-transform①①"></a>

The use of this property with any value other than [none](#valdef-perspective-none) establishes a stacking context. It also establishes a [containing block for all descendants](https://www.w3.org/TR/css-transforms-1/#containing-block-for-all-descendants), just like the [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) property does.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d6818476"></a> We don’t really need to be a stacking context or containing block for perspective, but maybe webcompat means we can’t change this.

<a id="ref-for-propdef-perspective①②"></a>

<a id="ref-for-propdef-perspective-origin⑦"></a>

<a id="ref-for-perspective-matrix②"></a>

The values of the [perspective](#propdef-perspective) and [perspective-origin](#propdef-perspective-origin) properties are used to compute the [perspective matrix](#perspective-matrix), as described above.

<a id="ref-for-propdef-perspective-origin⑧"></a>

## <a id="perspective-origin-property"></a>9. The [perspective-origin](#propdef-perspective-origin) Property

<a id="ref-for-propdef-perspective-origin⑨"></a>

<a id="ref-for-propdef-perspective①③"></a>

The [perspective-origin](#propdef-perspective-origin) property establishes the origin for the [perspective](#propdef-perspective) property. It effectively sets the X and Y position at which the viewer appears to be looking at the children of the element.

| Field               | Definition                                                                                                         |
|---------------------|--------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-perspective-origin"></a>perspective-origin                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-position"></a>[\<position\>](https://www.w3.org/TR/css-values-4/#typedef-position)                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 50% 50%                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-transformable-element⑦"></a>[transformable elements](https://www.w3.org/TR/css-transforms-1/#transformable-element)         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-reference-box①"></a>refer to the size of the [reference box](https://www.w3.org/TR/css-transforms-1/#reference-box) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-propdef-background-position"></a>see [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position) |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                  |

<a id="ref-for-propdef-perspective①④"></a>

<a id="ref-for-propdef-perspective-origin①⓪"></a>

<a id="ref-for-perspective-matrix③"></a>

The values of the [perspective](#propdef-perspective) and [perspective-origin](#propdef-perspective-origin) properties are used to compute the [perspective matrix](#perspective-matrix), as described above.

<a id="ref-for-propdef-perspective-origin①①"></a>

<a id="ref-for-reference-box②"></a>

The values for [perspective-origin](#propdef-perspective-origin) represent an offset of the perspective origin from the top left corner of the [reference box](https://www.w3.org/TR/css-transforms-1/#reference-box).

<a id="ref-for-percentage-value②"></a>

<a id="valdef-perspective-origin-percentage"></a>[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)

<a id="ref-for-reference-box③"></a>

A percentage for the horizontal perspective offset is relative to the width of the [reference box](https://www.w3.org/TR/css-transforms-1/#reference-box). A percentage for the vertical offset is relative to height of the <a id="ref-for-reference-box④"></a>reference box. The value for the horizontal and vertical offset represent an offset from the top left corner of the <a id="ref-for-reference-box⑤"></a>reference box.

<a id="ref-for-length-value⑤"></a>

<a id="valdef-perspective-origin-length"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

<a id="ref-for-reference-box⑥"></a>

A length value gives a fixed length as the offset. The value for the horizontal and vertical offset represent an offset from the top left corner of the [reference box](https://www.w3.org/TR/css-transforms-1/#reference-box).

<a id="valdef-perspective-origin-top"></a>top

Computes to 0% for the vertical position if one or two values are given, otherwise specifies the top edge as the origin for the next offset.

<a id="valdef-perspective-origin-right"></a>right

Computes to 100% for the horizontal position if one or two values are given, otherwise specifies the right edge as the origin for the next offset.

<a id="valdef-perspective-origin-bottom"></a>bottom

Computes to 100% for the vertical position if one or two values are given, otherwise specifies the bottom edge as the origin for the next offset.

<a id="valdef-perspective-origin-left"></a>left

Computes to 0% for the horizontal position if one or two values are given, otherwise specifies the left edge as the origin for the next offset.

<a id="valdef-perspective-origin-center"></a>center

Computes to 50% (left 50%) for the horizontal position if the horizontal position is not otherwise specified, or 50% (top 50%) for the vertical position if it is.

<a id="ref-for-propdef-perspective-origin①②"></a>

<a id="ref-for-resolved-value-special-case-property-like-height"></a>

The [perspective-origin](#propdef-perspective-origin) property is a [resolved value special case property like height](https://www.w3.org/TR/cssom-1/#resolved-value-special-case-property-like-height). [\[CSSOM\]](#biblio-cssom)

<a id="ref-for-propdef-backface-visibility⑤"></a>

## <a id="backface-visibility-property"></a>10. The [backface-visibility](#propdef-backface-visibility) Property

| Field               | Definition                                                                                                 |
|---------------------|------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-backface-visibility"></a>backface-visibility                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⓪"></a>visible [\|](https://www.w3.org/TR/css-values-4/#comb-one) hidden                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | visible                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-transformable-element⑧"></a>[transformable elements](https://www.w3.org/TR/css-transforms-1/#transformable-element) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                   |

<a id="ref-for-propdef-backface-visibility⑥"></a>

The visibility of an element with [backface-visibility: hidden](#propdef-backface-visibility) is determined as follows:

1.  <a id="ref-for-accumulated-3d-transformation-matrix⑦"></a>

    Compute the element’s [accumulated 3D transformation matrix](#accumulated-3d-transformation-matrix).

2.  If the component of the matrix in row 3, column 3 is negative, then the element should be hidden. Otherwise it is visible.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-4984e181"></a> Backface-visibility cannot be tested by only looking at m33. See [\#917](https://github.com/w3c/csswg-drafts/issues/917).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The reasoning for this definition is as follows. Assume elements are rectangles in the <var>x</var>–<var>y</var> plane with infinitesimal thickness. The front of the untransformed element has coordinates like (<var>x</var>, <var>y</var>, <var>ε</var>), and the back is (<var>x</var>, <var>y</var>, −<var>ε</var>), for some very small <var>ε</var>. We want to know if after the transformation, the front of the element is closer to the viewer than the back (higher <var>z</var>-value) or further away. The <var>z</var>-coordinate of the front will be m<sub>13</sub><var>x</var> + m<sub>23</sub><var>y</var> + m<sub>33</sub><var>ε</var> + m<sub>43</sub>, before accounting for perspective, and the back will be m<sub>13</sub><var>x</var> + m<sub>23</sub><var>y</var> − m<sub>33</sub><var>ε</var> + m<sub>43</sub>. The first quantity is greater than the second if and only if m<sub>33</sub> \> 0. (If it equals zero, the front and back are equally close to the viewer. This probably means something like a 90-degree rotation, which makes the element invisible anyway, so we don’t really care whether it vanishes.)

## <a id="svg-three-dimensional-functions"></a>11. SVG and 3D transform functions

<a id="ref-for-container-element"></a>

<a id="ref-for-the-a-element"></a>

<a id="ref-for-elementdef-g"></a>

<a id="ref-for-elementdef-svg"></a>

<a id="ref-for-graphics-element"></a>

<a id="ref-for-graphics-referencing-element"></a>

<a id="ref-for-elementdef-foreignObject"></a>

This specification explicitly requires three-dimensional transform functions to apply to the [container elements](https://www.w3.org/TR/SVG2/struct.html#container-element): <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element">a</a></code>, <code><a href="https://www.w3.org/TR/SVG2/struct.html#elementdef-g">g</a></code>, <code><a href="https://www.w3.org/TR/SVG2/struct.html#elementdef-svg">svg</a></code>, all [graphics elements](https://www.w3.org/TR/SVG2/struct.html#graphics-element), all [graphics referencing elements](https://www.w3.org/TR/SVG2/struct.html#graphics-referencing-element) and the SVG <code><a href="https://www.w3.org/TR/SVG2/embedded.html#elementdef-foreignObject">foreignObject</a></code> element.

<a id="ref-for-propdef-perspective①⑤"></a>

<a id="ref-for-propdef-perspective-origin①③"></a>

<a id="ref-for-propdef-transform-style⑧"></a>

<a id="ref-for-propdef-backface-visibility⑦"></a>

<a id="ref-for-elementdef-clippath"></a>

<a id="ref-for-elementdef-linearGradient"></a>

<a id="ref-for-elementdef-radialGradient"></a>

<a id="ref-for-elementdef-pattern"></a>

<a id="ref-for-elementdef-clippath①"></a>

<a id="ref-for-elementdef-mask"></a>

<a id="ref-for-elementdef-pattern①"></a>

Three-dimensional transform functions and the properties [perspective](#propdef-perspective), [perspective-origin](#propdef-perspective-origin), [transform-style](#propdef-transform-style) and [backface-visibility](#propdef-backface-visibility) can not be used for the elements: <code><a href="https://www.w3.org/TR/css-masking-1/#elementdef-clippath">clipPath</a></code>, <code><a href="https://www.w3.org/TR/SVG2/pservers.html#elementdef-linearGradient">linearGradient</a></code>, <code><a href="https://www.w3.org/TR/SVG2/pservers.html#elementdef-radialGradient">radialGradient</a></code> and <code><a href="https://www.w3.org/TR/SVG2/pservers.html#elementdef-pattern">pattern</a></code>. If a transform list includes a three-dimensional transform function, the complete transform list must be ignored. The values of every previously named property must be ignored. Transformable elements that are contained by one of these elements can have three-dimensional transform functions. The <code><a href="https://www.w3.org/TR/css-masking-1/#elementdef-clippath">clipPath</a></code>, <code><a href="https://www.w3.org/TR/css-masking-1/#elementdef-mask">mask</a></code>, <code><a href="https://www.w3.org/TR/SVG2/pservers.html#elementdef-pattern">pattern</a></code> elements require the user agent to create a flattened representation of the descendant elements before they can be applied, and therefore override the behavior of <a id="ref-for-propdef-transform-style⑨"></a>transform-style: preserve-3d.

<a id="ref-for-VectorEffectProperty"></a>

<a id="ref-for-3d-rendering-context①⑤"></a>

If the [vector-effect](https://www.w3.org/TR/SVG2/coords.html#VectorEffectProperty) property is set to non-scaling-stroke and an object is within a [3D rendering context](#3d-rendering-context) the property has no affect on stroking the object.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-c054a7a7"></a> formally describe the syntax of the 3D transform functions in SVG, as is done [for the 2-D functions](https://drafts.csswg.org/css-transforms-1/#svg-syntax).

## <a id="transform-functions"></a>12. The Transform Functions

<a id="ref-for-propdef-transform①②"></a>

<a id="ref-for-angle-value⑥"></a>

<a id="ref-for-number-value⑤"></a>

<a id="ref-for-reference-box⑦"></a>

The value of the [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) property is a list of <a id="typedef-transform-function"></a>\<transform-function\>. The set of allowed transform functions is given below. Wherever [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) is used in this specification, a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that is equal to zero is also allowed, which is treated the same as an angle of zero degrees. A percentage for horizontal translations is relative to the width of the [reference box](https://www.w3.org/TR/css-transforms-1/#reference-box). A percentage for vertical translations is relative to the height of the <a id="ref-for-reference-box⑧"></a>reference box. A percentage in a scale function is equivalent to a number, and serializes as a number in specified values. For example, scale3d(50%, 100%, 150%) serializes as scale3d(0.5, 1, 1.5).

### <a id="two-d-transform-functions"></a>12.1. 2D Transform Functions

The scale functions defined in [\[css-transforms-1\]](#biblio-css-transforms-1) now support percentages.

<a id="ref-for-mult-comma"></a>

<a id="ref-for-percentage-value③"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-number-value⑥"></a>

<a id="funcdef-scale"></a>scale() = scale( \[ [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) \][\#{1,2}](https://www.w3.org/TR/css-values-4/#mult-comma) )

<a id="ref-for-percentage-value④"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-number-value⑦"></a>

<a id="funcdef-scalex"></a>scaleX() = scaleX( \[ [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) \] )

<a id="ref-for-percentage-value⑤"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-number-value⑧"></a>

<a id="funcdef-scaley"></a>scaleY() = scaleY( \[ [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) \] )

As [defined in css-transforms-1](https://www.w3.org/TR/css-transforms-1/#two-d-transform-functions), but also accepting percentages as [described above](#transform-functions).

### <a id="three-d-transform-functions"></a>12.2. 3D Transform Functions

<a id="ref-for-zero-value"></a>

In the following <a id="3d-transform-functions"></a>3d transform functions, a [\<zero\>](https://www.w3.org/TR/css-values-4/#zero-value) behaves the same as 0deg. ("Unitless 0" angles are preserved for legacy compat reasons.)

<a id="ref-for-mult-comma①"></a>

<a id="ref-for-number-value⑨"></a>

<a id="funcdef-matrix3d"></a>matrix3d() = matrix3d( [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)[\#{16}](https://www.w3.org/TR/css-values-4/#mult-comma) )

specifies a 3D transformation as a 4x4 homogeneous matrix of 16 values in column-major order.

<a id="ref-for-length-value⑥"></a>

<a id="ref-for-comb-comma"></a>

<a id="ref-for-typedef-length-percentage③"></a>

<a id="funcdef-translate3d"></a>translate3d() = translate3d( [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) [,](https://www.w3.org/TR/css-values-4/#comb-comma) <a id="ref-for-typedef-length-percentage④"></a>\<length-percentage\> <a id="ref-for-comb-comma①"></a>, [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) )

specifies a [3D translation](#Translate3dDefined) by the vector \[tx,ty,tz\], with tx, ty and tz being the first, second and third translation-value parameters respectively.

<a id="ref-for-length-value⑦"></a>

<a id="funcdef-translatez"></a>translateZ() = translateZ( [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) )

specifies a [3D translation](#Translate3dDefined) by the vector \[0,0,tz\] with the given amount in the Z direction.

<a id="ref-for-mult-comma②"></a>

<a id="ref-for-percentage-value⑥"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-number-value①⓪"></a>

<a id="funcdef-scale3d"></a>scale3d() = scale3d( \[ [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) \][\#{3}](https://www.w3.org/TR/css-values-4/#mult-comma) )

specifies a [3D scale](#Scale3dDefined) operation by the \[sx,sy,sz\] scaling vector described by the 3 parameters.

<a id="ref-for-percentage-value⑦"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-number-value①①"></a>

<a id="funcdef-scalez"></a>scaleZ() = scaleZ( \[ [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) \] )

specifies a [3D scale](#Scale3dDefined) operation using the \[1,1,sz\] scaling vector, where sz is given as the parameter.

<a id="ref-for-zero-value①"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-angle-value⑦"></a>

<a id="ref-for-comb-comma②"></a>

<a id="ref-for-number-value①②"></a>

<a id="funcdef-rotate3d"></a>rotate3d() = rotate3d( [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) [,](https://www.w3.org/TR/css-values-4/#comb-comma) <a id="ref-for-number-value①③"></a>\<number\> <a id="ref-for-comb-comma③"></a>, <a id="ref-for-number-value①④"></a>\<number\> <a id="ref-for-comb-comma④"></a>, \[ [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<zero\>](https://www.w3.org/TR/css-values-4/#zero-value) \] )

specifies a [3D rotation](#Rotate3dDefined) by the angle specified in last parameter about the \[x,y,z\] direction vector described by the first three parameters. A direction vector that cannot be normalized, such as \[0,0,0\], will cause the rotation to not be applied.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: the rotation is clockwise as one looks from the end of the vector toward the origin.

<a id="ref-for-zero-value②"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-angle-value⑧"></a>

<a id="funcdef-rotatex"></a>rotateX() = rotateX( \[ [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<zero\>](https://www.w3.org/TR/css-values-4/#zero-value) \] )

same as rotate3d(1, 0, 0, \<angle\>).

<a id="ref-for-zero-value③"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-angle-value⑨"></a>

<a id="funcdef-rotatey"></a>rotateY() = rotateY( \[ [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<zero\>](https://www.w3.org/TR/css-values-4/#zero-value) \] )

same as rotate3d(0, 1, 0, \<angle\>).

<a id="ref-for-zero-value④"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-angle-value①⓪"></a>

<a id="funcdef-rotatez"></a>rotateZ() = rotateZ( \[ [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<zero\>](https://www.w3.org/TR/css-values-4/#zero-value) \] )

same as rotate3d(0, 0, 1, \<angle\>), which is a 3d transform equivalent to the 2d transform rotate(\<angle\>).

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-length-value⑧"></a>

<a id="funcdef-perspective"></a>perspective() = perspective( [\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) <a id="valdef-perspective-func-none"></a>none )

specifies a [perspective projection matrix](#PerspectiveDefined). This matrix scales points in X and Y based on their Z value, scaling points with positive Z values away from the origin, and those with negative Z values towards the origin. Points on the z=0 plane are unchanged. The parameter represents the distance of the z=0 plane from the viewer. Lower values give a more flattened pyramid and therefore a more pronounced perspective effect. For example, a value of 1000px gives a moderate amount of foreshortening and a value of 200px gives an extreme amount.

<a id="ref-for-resolved-value①"></a>

<a id="ref-for-propdef-transform①③"></a>

If the depth value is less than 1px, it must be treated as 1px for the purpose of rendering, for computing the [resolved value](https://www.w3.org/TR/cssom-1/#resolved-value) of [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform), and when used as the endpoint of [interpolation](#interpolation-of-transform-functions).

<a id="ref-for-funcdef-perspective②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The intent of the above rules on values less than 1px is that they cover the cases where the [perspective()](#funcdef-perspective) function needs to be converted into a matrix.

### <a id="transform-primitives"></a>12.3. Transform function primitives and derivatives

Some transform functions can be represented by more generic transform functions. These transform functions are called derived transform functions, and the generic transform functions are called primitive transform functions. Three-dimensional primitives and their derived transform functions are:

<a id="ref-for-funcdef-translate3d①"></a>

<a id="translate3d-primitive"></a>[translate3d()](#funcdef-translate3d)

<a id="ref-for-funcdef-transform-translate②"></a>

<a id="ref-for-funcdef-translatez"></a>

<a id="ref-for-funcdef-transform-translatey"></a>

<a id="ref-for-funcdef-transform-translatex"></a>

for [\<translateX()\>](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translatex), [\<translateY()\>](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translatey), [translateZ()](#funcdef-translatez) and [\<translate()\>](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translate).

<a id="ref-for-funcdef-scale3d①"></a>

<a id="scale3d-primitive"></a>[scale3d()](#funcdef-scale3d)

<a id="ref-for-funcdef-scale②"></a>

<a id="ref-for-funcdef-scalez"></a>

<a id="ref-for-funcdef-scaley"></a>

<a id="ref-for-funcdef-scalex"></a>

for [\<scaleX()\>](#funcdef-scalex), [\<scaleY()\>](#funcdef-scaley), [scaleZ()](#funcdef-scalez) and [\<scale()\>](#funcdef-scale).

<a id="ref-for-funcdef-rotate3d①"></a>

<a id="rotate3d-primitive"></a>[rotate3d()](#funcdef-rotate3d)

<a id="ref-for-funcdef-rotatez①"></a>

<a id="ref-for-funcdef-rotatey①"></a>

<a id="ref-for-funcdef-rotatex①"></a>

<a id="ref-for-funcdef-transform-rotate②"></a>

for [\<rotate()\>](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-rotate), [rotateX()](#funcdef-rotatex), [rotateY()](#funcdef-rotatey) and [rotateZ()](#funcdef-rotatez).

<a id="interpolation-two-three-dimensional-function"></a> For derived transform functions that have a two-dimensional primitive and a three-dimensional primitive, the context decides about the used primitive. See [Interpolation of primitives and derived transform functions](#interpolation-of-transform-functions).

## <a id="matrix-interpolation"></a>13. Interpolation of Matrices

<a id="ref-for-3d-matrix"></a>

When interpolating between two matrices, each matrix is decomposed into the corresponding translation, rotation, scale, skew and (for a [3D matrix](#3d-matrix)) perspective values. Each corresponding component of the decomposed matrices gets interpolated numerically and recomposed back to a matrix in a final step.

### <a id="interpolation-of-3d-matrices"></a>13.1. Interpolation of 3D matrices

#### <a id="decomposing-a-3d-matrix"></a>13.1.1. Decomposing a 3D matrix

The pseudo code below is based upon the "unmatrix" method in "Graphics Gems II, edited by Jim Arvo", but modified to use Quaternions instead of Euler angles to avoid the problem of Gimbal Locks.

The following pseudocode works on a 4x4 homogeneous matrix:

```text
Input:  matrix      ; a 4x4 matrix
Output: translation ; a 3 component vector
        scale       ; a 3 component vector
        skew        ; skew factors XY,XZ,YZ represented as a 3 component vector
        perspective ; a 4 component vector
        quaternion  ; a 4 component vector
Returns false if the matrix cannot be decomposed, true if it can


// Normalize the matrix.
if (matrix[3][3] == 0)
    return false

for (i = 0; i < 4; i++)
    for (j = 0; j < 4; j++)
        matrix[i][j] /= matrix[3][3]

// perspectiveMatrix is used to solve for perspective, but it also provides
// an easy way to test for singularity of the upper 3x3 component.
perspectiveMatrix = matrix

for (i = 0; i < 3; i++)
    perspectiveMatrix[i][3] = 0

perspectiveMatrix[3][3] = 1

if (determinant(perspectiveMatrix) == 0)
    return false

// First, isolate perspective.
if (matrix[0][3] != 0 || matrix[1][3] != 0 || matrix[2][3] != 0)
    // rightHandSide is the right hand side of the equation.
    rightHandSide[0] = matrix[0][3]
    rightHandSide[1] = matrix[1][3]
    rightHandSide[2] = matrix[2][3]
    rightHandSide[3] = matrix[3][3]

    // Solve the equation by inverting perspectiveMatrix and multiplying
    // rightHandSide by the inverse.
    inversePerspectiveMatrix = inverse(perspectiveMatrix)
    transposedInversePerspectiveMatrix = transposeMatrix4(inversePerspectiveMatrix)
    perspective = multVecMatrix(rightHandSide, transposedInversePerspectiveMatrix)
else
    // No perspective.
    perspective[0] = perspective[1] = perspective[2] = 0
    perspective[3] = 1

// Next take care of translation
for (i = 0; i < 3; i++)
    translate[i] = matrix[3][i]

// Now get scale and shear. 'row' is a 3 element array of 3 component vectors
for (i = 0; i < 3; i++)
    row[i][0] = matrix[i][0]
    row[i][1] = matrix[i][1]
    row[i][2] = matrix[i][2]

// Compute X scale factor and normalize first row.
scale[0] = length(row[0])
row[0] = normalize(row[0])

// Compute XY shear factor and make 2nd row orthogonal to 1st.
skew[0] = dot(row[0], row[1])
row[1] = combine(row[1], row[0], 1.0, -skew[0])

// Now, compute Y scale and normalize 2nd row.
scale[1] = length(row[1])
row[1] = normalize(row[1])
skew[0] /= scale[1];

// Compute XZ and YZ shears, orthogonalize 3rd row
skew[1] = dot(row[0], row[2])
row[2] = combine(row[2], row[0], 1.0, -skew[1])
skew[2] = dot(row[1], row[2])
row[2] = combine(row[2], row[1], 1.0, -skew[2])

// Next, get Z scale and normalize 3rd row.
scale[2] = length(row[2])
row[2] = normalize(row[2])
skew[1] /= scale[2]
skew[2] /= scale[2]

// At this point, the matrix (in rows) is orthonormal.
// Check for a coordinate system flip.  If the determinant
// is -1, then negate the matrix and the scaling factors.
pdum3 = cross(row[1], row[2])
if (dot(row[0], pdum3) < 0)
    for (i = 0; i < 3; i++)
        scale[i] *= -1;
        row[i][0] *= -1
        row[i][1] *= -1
        row[i][2] *= -1

// Now, get the rotations out
quaternion[0] = 0.5 * sqrt(max(1 + row[0][0] - row[1][1] - row[2][2], 0))
quaternion[1] = 0.5 * sqrt(max(1 - row[0][0] + row[1][1] - row[2][2], 0))
quaternion[2] = 0.5 * sqrt(max(1 - row[0][0] - row[1][1] + row[2][2], 0))
quaternion[3] = 0.5 * sqrt(max(1 + row[0][0] + row[1][1] + row[2][2], 0))

if (row[2][1] > row[1][2])
    quaternion[0] = -quaternion[0]
if (row[0][2] > row[2][0])
    quaternion[1] = -quaternion[1]
if (row[1][0] > row[0][1])
    quaternion[2] = -quaternion[2]

return true
```
#### <a id="interpolation-of-decomposed-3d-matrix-values"></a>13.1.2. Interpolation of decomposed 3D matrix values

Each component of the decomposed values translation, scale, skew and perspective of the source matrix get linearly interpolated with each corresponding component of the destination matrix.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For instance, `translate[0]` of the source matrix and `translate[0]` of the destination matrix are interpolated numerically, and the result is used to set the translation of the animating element.

Quaternions of the decomposed source matrix are interpolated with quaternions of the decomposed destination matrix using the spherical linear interpolation (Slerp) as described by the pseudo code below:

```text
Input:  quaternionA   ; a 4 component vector
        quaternionB   ; a 4 component vector
        t             ; interpolation parameter with 0 <= t <= 1
Output: quaternionDst ; a 4 component vector


product = dot(quaternionA, quaternionB)

// Clamp product to -1.0 <= product <= 1.0
product = min(product, 1.0)
product = max(product, -1.0)

if (abs(product) == 1.0)
   quaternionDst = quaternionA
   return

theta = acos(product)
w = sin(t * theta) / sqrt(1 - product * product)

for (i = 0; i < 4; i++)
  quaternionA[i] *= cos(t * theta) - product * w
  quaternionB[i] *= w
  quaternionDst[i] = quaternionA[i] + quaternionB[i]

return
```
#### <a id="recomposing-to-a-3d-matrix"></a>13.1.3. Recomposing to a 3D matrix

After interpolation, the resulting values are used to transform the elements user space. One way to use these values is to recompose them into a 4x4 matrix. This can be done following the pseudo code below:

```text
Input:  translation ; a 3 component vector
        scale       ; a 3 component vector
        skew        ; skew factors XY,XZ,YZ represented as a 3 component vector
        perspective ; a 4 component vector
        quaternion  ; a 4 component vector
Output: matrix      ; a 4x4 matrix

Supporting functions (matrix is a 4x4 matrix):
  matrix  multiply(matrix a, matrix b)   returns the 4x4 matrix product of a * b

// apply perspective
for (i = 0; i < 4; i++)
  matrix[i][3] = perspective[i]

// apply translation
for (i = 0; i < 4; i++)
  for (j = 0; j < 3; j++)
    matrix[3][i] += translation[j] * matrix[j][i]

// apply rotation
x = quaternion[0]
y = quaternion[1]
z = quaternion[2]
w = quaternion[3]

// Construct a composite rotation matrix from the quaternion values
// rotationMatrix is a identity 4x4 matrix initially
rotationMatrix[0][0] = 1 - 2 * (y * y + z * z)
rotationMatrix[0][1] = 2 * (x * y - z * w)
rotationMatrix[0][2] = 2 * (x * z + y * w)
rotationMatrix[1][0] = 2 * (x * y + z * w)
rotationMatrix[1][1] = 1 - 2 * (x * x + z * z)
rotationMatrix[1][2] = 2 * (y * z - x * w)
rotationMatrix[2][0] = 2 * (x * z - y * w)
rotationMatrix[2][1] = 2 * (y * z + x * w)
rotationMatrix[2][2] = 1 - 2 * (x * x + y * y)

matrix = multiply(matrix, rotationMatrix)

// apply skew
// temp is a identity 4x4 matrix initially
if (skew[2])
    temp[2][1] = skew[2]
    matrix = multiply(matrix, temp)

if (skew[1])
    temp[2][1] = 0
    temp[2][0] = skew[1]
    matrix = multiply(matrix, temp)

if (skew[0])
    temp[2][0] = 0
    temp[1][0] = skew[0]
    matrix = multiply(matrix, temp)

// apply scale
for (i = 0; i < 3; i++)
  for (j = 0; j < 4; j++)
    matrix[i][j] *= scale[i]

return
```
## <a id="interpolation-of-transform-functions"></a>14. Interpolation of primitives and derived transform functions

<a id="ref-for-funcdef-transform-matrix③"></a>

<a id="ref-for-funcdef-matrix3d③"></a>

<a id="ref-for-funcdef-perspective③"></a>

Two transform functions with the same name and the same number of arguments are interpolated numerically without a former conversion. The calculated value will be of the same transform function type with the same number of arguments. Special rules apply to [\<matrix()\>](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-matrix), [\<matrix3d()\>](#funcdef-matrix3d) and [\<perspective()\>](#funcdef-perspective).

<a id="ref-for-funcdef-transform-matrix④"></a>

<a id="ref-for-funcdef-matrix3d④"></a>

<a id="ref-for-funcdef-perspective④"></a>

The transform functions [\<matrix()\>](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-matrix), [matrix3d()](#funcdef-matrix3d) and [perspective()](#funcdef-perspective) get converted into 4x4 matrices first and interpolated as defined in section [Interpolation of Matrices](#matrix-interpolation) afterwards.

<a id="ref-for-funcdef-rotate3d②"></a>

For interpolations with the primitive [rotate3d()](#funcdef-rotate3d), the direction vectors of the transform functions get normalized first. If the normalized vectors are not equal and both rotation angles are non-zero the transform functions get converted into 4x4 matrices first and interpolated as defined in section [Interpolation of Matrices](#matrix-interpolation) afterwards. Otherwise the rotation angle gets interpolated numerically and the rotation vector of the non-zero angle is used or (0, 0, 1) if both angles are zero.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1356eb1f"></a>
>
> The two transform functions translate(0) and translate(100px) are of the same type, have the same number of arguments and therefore can get interpolated numerically. translateX(100px) is not of the same type and translate(100px, 0) does not have the same number of arguments, therefore these transform functions can not get interpolated without a former conversion step.

Two different types of transform functions that share the same primitive, or transform functions of the same type with different number of arguments can be interpolated. Both transform functions need a former conversion to the common primitive first and get interpolated numerically afterwards. The computed value will be the primitive with the resulting interpolated arguments.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bd3fe5db"></a>
>
> <a id="ref-for-funcdef-transform-translate③"></a>
>
> The following example describes a transition from translateX(100px) to translateY(100px) in 3 seconds on hovering over the div box. Both transform functions derive from the same primitive [translate()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translate) and therefore can be interpolated.
>
> ```text
> div {
>   transform: translateX(100px);
> }
> 
> div:hover {
>   transform: translateY(100px);
>   transition: transform 3s;
> }
> ```
>
> For the time of the transition both transform functions get transformed to the common primitive. translateX(100px) gets converted to translate(100px, 0) and translateY(100px) gets converted to translate(0, 100px). Both transform functions can then get interpolated numerically.

If both transform functions share a primitive in the two-dimensional space, both transform functions get converted to the two-dimensional primitive. If one or both transform functions are three-dimensional transform functions, the common three-dimensional primitive is used.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-24abc2ae"></a>
>
> <a id="ref-for-funcdef-translate3d②"></a>
>
> In this example a two-dimensional transform function gets animated to a three-dimensional transform function. The common primitive is [translate3d()](#funcdef-translate3d).
>
> ```text
> div {
>   transform: translateX(100px);
> }
> 
> div:hover {
>   transform: translateZ(100px);
>   transition: transform 3s;
> }
> ```
>
> First translateX(100px) gets converted to translate3d(100px, 0, 0) and translateZ(100px) to translate3d(0, 0, 100px) respectively. Then both converted transform functions get interpolated numerically.

## <a id="combining-transform-lists"></a>15. Addition and accumulation of transform lists

<a id="ref-for-addition"></a>

<a id="ref-for-list"></a>

<a id="ref-for-list-append"></a>

[Addition](https://www.w3.org/TR/css-values-4/#addition) of two transform lists <var>V<sub>a</sub></var> and <var>V<sub>b</sub></var> is defined as [list](https://infra.spec.whatwg.org/#list) concatenation such that <var>V<sub>result</sub></var> is equal to <var>V<sub>b</sub></var> [appended](https://infra.spec.whatwg.org/#list-append) to <var>V<sub>a</sub></var>.

<a id="ref-for-accumulation"></a>

<a id="ref-for-identity-transform-function"></a>

[Accumulation](https://www.w3.org/TR/css-values-4/#accumulation) of two transform lists <var>V<sub>a</sub></var> and <var>V<sub>b</sub></var> follows the same steps as interpolation with regards to matching transform functions including padding lists with [identity transform functions](#identity-transform-function), converting none to an <a id="ref-for-identity-transform-function①"></a>identity transform function, and converting both arguments to matrices as necessary (see [CSS Transforms 1 § 11 Interpolation of Transforms](https://www.w3.org/TR/css-transforms-1/#interpolation-of-transforms)). However, instead of interpolating the individual parameters, they are combined using arithmetic addition—except in the case of parameters whose value is one in the <a id="ref-for-identity-transform-function②"></a>identity transform function (e.g. scale parameters and matrix elements <var>m11</var>, <var>m22</var>, <var>m33</var>, and <var>m44</var>), which combine using <a id="accumulation-for-one-based-values"></a>accumulation for one-based values as follows:

<var>V<sub>result</sub></var> = <var>V<sub>a</sub></var> + <var>V<sub>b</sub></var> - 1

<a id="ref-for-accumulation①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> The above definition preserves the intent of [accumulation](https://www.w3.org/TR/css-values-4/#accumulation) which is that <var>V<sub>b</sub></var> acts as a delta from <var>V<sub>a</sub></var> and allows an animation such as:
>
> ```javascript
> div.animate(
>   { transform: ['scale(1)', 'scale(2)'] },
>   {
>     duration: 1000,
>     easing: 'ease',
>   }
> );
> ```
>
> to produce the expected behavior when extended as follows:
>
> ```javascript
> div.animate(
>   { transform: ['scale(1)', 'scale(2)'] },
>   {
>     duration: 1000,
>     easing: 'ease',
>     iterations: 5,
>     iterationComposite: 'accumulate',
>   }
> );
> ```
### <a id="neutral-element"></a>15.1. Neutral element for addition

Some animations require a neutral element for addition. For transform functions this is a scalar or a list of scalars of 0. Examples of neutral elements for transform functions are translate(0), translate3d(0, 0, 0), translateX(0), translateY(0), translateZ(0), scale(0), scaleX(0), scaleY(0), scaleZ(0), rotate(0), rotate3d(vx, vy, vz, 0) (where <var>v</var> is a context dependent vector), rotateX(0), rotateY(0), rotateZ(0), skew(0, 0), skewX(0), skewY(0), matrix(0, 0, 0, 0, 0, 0), matrix3d(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0) and perspective(none).

<a id="ref-for-funcdef-transform-matrix⑤"></a>

<a id="ref-for-funcdef-matrix3d⑤"></a>

<a id="ref-for-funcdef-perspective⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Animations to or from the neutral element of additions [\<matrix()\>](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-matrix), [matrix3d()](#funcdef-matrix3d) and [perspective()](#funcdef-perspective) fall back to discrete animations (See [§ 13 Interpolation of Matrices](#matrix-interpolation)).

## <a id="mathematical-description"></a>16. Mathematical Description of Transform Functions

Mathematically, all transform functions can be represented as 4x4 transformation matrices of the following form:

\$\$&#x5C;begin{bmatrix} m11 &#x26; m21 &#x26; m31 &#x26; m41 &#x5C;&#x5C; m12 &#x26; m22 &#x26; m32 &#x26; m42 &#x5C;&#x5C; m13 &#x26; m23 &#x26; m33 &#x26; m43 &#x5C;&#x5C; m14 &#x26; m24 &#x26; m34 &#x26; m44 &#x5C;end{bmatrix}\$\$

One translation unit on a matrix is equivalent to 1 pixel in the local coordinate system of the element.

- <a id="Translate3dDefined"></a> A 3D translation with the parameters <em>tx</em>, <em>ty</em> and <em>tz</em> is equivalent to the matrix:

  \$\$&#x5C;begin{bmatrix} 1 &#x26; 0 &#x26; 0 &#x26; tx &#x5C;&#x5C; 0 &#x26; 1 &#x26; 0 &#x26; ty &#x5C;&#x5C; 0 &#x26; 0 &#x26; 1 &#x26; tz &#x5C;&#x5C; 0 &#x26; 0 &#x26; 0 &#x26; 1 &#x5C;end{bmatrix}\$\$

- <a id="Scale3dDefined"></a> A 3D scaling with the parameters <em>sx</em>, <em>sy</em> and <em>sz</em> is equivalent to the matrix:

  \$\$&#x5C;begin{bmatrix} sx &#x26; 0 &#x26; 0 &#x26; 0 &#x5C;&#x5C; 0 &#x26; sy &#x26; 0 &#x26; 0 &#x5C;&#x5C; 0 &#x26; 0 &#x26; sz &#x26; 0 &#x5C;&#x5C; 0 &#x26; 0 &#x26; 0 &#x26; 1 &#x5C;end{bmatrix}\$\$

- <a id="Rotate3dDefined"></a> A 3D rotation with the vector \[x,y,z\] and the parameter <em>alpha</em> is equivalent to the matrix:

  \$\$&#x5C;begin{bmatrix} 1 - 2 &#x5C;cdot (y^2 + z^2) &#x5C;cdot sq &#x26; 2 &#x5C;cdot (x &#x5C;cdot y &#x5C;cdot sq - z &#x5C;cdot sc) &#x26; 2 &#x5C;cdot (x &#x5C;cdot z &#x5C;cdot sq + y &#x5C;cdot sc) &#x26; 0 &#x5C;&#x5C; 2 &#x5C;cdot (x &#x5C;cdot y &#x5C;cdot sq + z &#x5C;cdot sc) &#x26; 1 - 2 &#x5C;cdot (x^2 + z^2) &#x5C;cdot sq &#x26; 2 &#x5C;cdot (y &#x5C;cdot z &#x5C;cdot sq - x &#x5C;cdot sc) &#x26; 0 &#x5C;&#x5C; 2 &#x5C;cdot (x &#x5C;cdot z &#x5C;cdot sq - y &#x5C;cdot sc) &#x26; 2 &#x5C;cdot (y &#x5C;cdot z &#x5C;cdot sq + x &#x5C;cdot sc) &#x26; 1 - 2 &#x5C;cdot (x^2 + y^2) &#x5C;cdot sq &#x26; 0 &#x5C;&#x5C; 0 &#x26; 0 &#x26; 0 &#x26; 1 &#x5C;end{bmatrix}\$\$

  where:

  \$\$sc = &#x5C;sin (&#x5C;alpha/2) &#x5C;cdot &#x5C;cos (&#x5C;alpha/2)\$\$ \$\$sq = &#x5C;sin^2 (&#x5C;alpha/2)\$\$

  and where x, y, and z have been normalized (that is, where the x, y, and z values given have been divided by the square root of the sum of their squares).

  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note that this means that a rotation around the X axis simplifies to:
  > \$\$&#x5C;begin{bmatrix} 1 &#x26; 0 &#x26; 0 &#x26; 0 &#x5C;&#x5C; 0 &#x26; 1 - 2 &#x5C;cdot sq &#x26; -2 &#x5C;cdot sc &#x26; 0 &#x5C;&#x5C; 0 &#x26; 2 &#x5C;cdot sc &#x26; 1 - 2 &#x5C;cdot sq &#x26; 0 &#x5C;&#x5C; 0 &#x26; 0 &#x26; 0 &#x26; 1 &#x5C;end{bmatrix}\$\$
  >
  > a rotation around the Y axis simplifies to:
  >
  > \$\$&#x5C;begin{bmatrix} 1 - 2 &#x5C;cdot sq &#x26; 0 &#x26; 2 &#x5C;cdot sc &#x26; 0 &#x5C;&#x5C; 0 &#x26; 1 &#x26; 0 &#x26; 0 &#x5C;&#x5C; -2 &#x5C;cdot sc &#x26; 0 &#x26; 1 - 2 &#x5C;cdot sq &#x26; 0 &#x5C;&#x5C; 0 &#x26; 0 &#x26; 0 &#x26; 1 &#x5C;end{bmatrix}\$\$
  >
  > and a rotation around the Z axis simplifies to:
  >
  > \$\$&#x5C;begin{bmatrix} 1 - 2 &#x5C;cdot sq &#x26; -2 &#x5C;cdot sc &#x26; 0 &#x26; 0 &#x5C;&#x5C; 2 &#x5C;cdot sc &#x26; 1 - 2 &#x5C;cdot sq &#x26; 0 &#x26; 0 &#x5C;&#x5C; 0 &#x26; 0 &#x26; 1 &#x26; 0 &#x5C;&#x5C; 0 &#x26; 0 &#x26; 0 &#x26; 1 &#x5C;end{bmatrix}\$\$

- <a id="PerspectiveDefined"></a> A perspective projection matrix with the parameter <var>d</var> is equivalent to the matrix:

  \$\$&#x5C;begin{bmatrix} 1 &#x26; 0 &#x26; 0 &#x26; 0 &#x5C;&#x5C; 0 &#x26; 1 &#x26; 0 &#x26; 0 &#x5C;&#x5C; 0 &#x26; 0 &#x26; 1 &#x26; 0 &#x5C;&#x5C; 0 &#x26; 0 &#x26; -1/d &#x26; 1 &#x5C;end{bmatrix}\$\$

  <a id="ref-for-valdef-perspective-func-none"></a>

  If the parameter <var>d</var> is [none](#valdef-perspective-func-none) it is treated as infinity (and the resulting matrix is the identity matrix).

<a id="ref-for-propdef-transform①④"></a>

## <a id="svg-transform"></a>17. The SVG [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) Attribute

<a id="ref-for-propdef-transform-origin⑧"></a>

<a id="ref-for-propdef-perspective①⑥"></a>

<a id="ref-for-propdef-perspective-origin①④"></a>

<a id="ref-for-propdef-transform-style①⓪"></a>

<a id="ref-for-propdef-backface-visibility⑧"></a>

This specification will also introduce the new presentation attributes [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin), [perspective](#propdef-perspective), [perspective-origin](#propdef-perspective-origin), [transform-style](#propdef-transform-style) and [backface-visibility](#propdef-backface-visibility).

Values on new introduced presentation attributes get parsed following the syntax rules on SVG Data Types [\[SVG11\]](#biblio-svg11).

## <a id="svg-animation"></a>18. SVG Animation

<a id="ref-for-AnimateElement"></a>

<a id="ref-for-SetElement"></a>

### <a id="svg-animate-element"></a>18.1. The <code><a href="https://www.w3.org/TR/SVG11/animate.html#AnimateElement">animate</a></code> and <code><a href="https://www.w3.org/TR/SVG11/animate.html#SetElement">set</a></code> element

<a id="ref-for-propdef-perspective①⑦"></a>

<a id="ref-for-propdef-perspective-origin①⑤"></a>

<a id="ref-for-propdef-transform-style①①"></a>

<a id="ref-for-propdef-backface-visibility⑨"></a>

The introduce presentation attributes [perspective](#propdef-perspective), [perspective-origin](#propdef-perspective-origin), [transform-style](#propdef-transform-style) and [backface-visibility](#propdef-backface-visibility) are animatable. <a id="ref-for-propdef-transform-style①②"></a>transform-style and <a id="ref-for-propdef-backface-visibility①⓪"></a>backface-visibility are non-additive.

## <a id="more-issues"></a>19. More Issues

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-b3f9db98"></a> Per [https&#x3A;&#x2F;&#x2F;lists&#x2E;w3&#x2E;org&#x2F;Archives&#x2F;Public&#x2F;www-style&#x2F;2015Mar&#x2F;0371&#x2E;html](https://lists.w3.org/Archives/Public/www-style/2015Mar/0371.html), the WG resolved to add a formula for decomposing a transform into a unified "scale" (the spec already defines how to decompose it into scaleX&#x2F;Y&#x2F;Z), for use by things like SVG’s non-scaling stroke spec&#x2E; [Formula is defined here.](https://www.w3.org/Graphics/SVG/WG/wiki/Proposals/Specifying_decomposition_of_scale)

## <a id="priv-sec"></a>20. Security and Privacy Considerations

This specification introduces no new security or privacy considerations.

## <a id="changes"></a> Changes

### <a id="changes-recent"></a> Recent Changes

Substantive changes since [3 March 2020 WD](https://www.w3.org/TR/2020/WD-css-transforms-2-20200303/):

- The specification no longer requires maintaining state for whether individual transform properties have 2D or 3D values, but instead requires that any value that can be expressed as 2D is treated as 2D (see [\#3305](https://github.com/w3c/csswg-drafts/issues/3305)).
  > <strong data-conversion-semantic="note">Note</strong>
  >
  > Note: An analogous change is intended for transform functions, but it has not been made yet.

- <a id="ref-for-funcdef-scaley①"></a>

  <a id="ref-for-funcdef-scalex①"></a>

  <a id="ref-for-funcdef-scale③"></a>

  <a id="ref-for-propdef-scale①⓪"></a>

  The [scale](#propdef-scale) property and [scale()](#funcdef-scale), [scaleX()](#funcdef-scalex) and [scaleY()](#funcdef-scaley) functions now support percentages (see [\#3399](https://github.com/w3c/csswg-drafts/issues/3399)).

- Fix multiple definitions to be consistent with the spec’s definition for a 3D Rendering Context:
  - Define that borders, backgrounds, and box decorations of an element establishing a 3D Rendering Context are rendered at z=0 in its 3D scene, rather than behind its 3D scene (see [\#6238](https://github.com/w3c/csswg-drafts/issues/6238)).

  - <a id="ref-for-propdef-perspective①⑧"></a>

    <a id="ref-for-propdef-transform①⑤"></a>

    <a id="ref-for-accumulated-3d-transformation-matrix⑧"></a>

    Define [accumulated 3D transformation matrix](#accumulated-3d-transformation-matrix) to include the [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) of the establishing element and the [perspective](#propdef-perspective) of its parent (see [\#6191](https://github.com/w3c/csswg-drafts/issues/6191)).

- Define that paint containment is a grouping property (see [\#6202](https://github.com/w3c/csswg-drafts/issues/6202)).

- <a id="ref-for-funcdef-perspective⑥"></a>

  <a id="ref-for-valdef-perspective-func-none①"></a>

  Add support for a [none](#valdef-perspective-func-none) argument to [perspective()](#funcdef-perspective) (see [\#6488](https://github.com/w3c/csswg-drafts/issues/6488)).

- <a id="ref-for-funcdef-perspective⑦"></a>

  Define that clamping of values of [perspective()](#funcdef-perspective) also applies to resolved values and interpolation (see [\#6320](https://github.com/w3c/csswg-drafts/issues/6320) and [\#6346](https://github.com/w3c/csswg-drafts/issues/6346)).

- <a id="ref-for-transformable-element⑨"></a>

  Clarify that the effects of preserve-3d only affect [transformable elements](https://www.w3.org/TR/css-transforms-1/#transformable-element) (see [\#6430](https://github.com/w3c/csswg-drafts/issues/6430)).

- <a id="ref-for-funcdef-perspective⑧"></a>

  Fixed the [neutral element for addition](#neutral-element) for [perspective()](#funcdef-perspective) to be perspective(none)

- <a id="ref-for-propdef-translate①⓪"></a>

  Added a note that the resolved value of [translate](#propdef-translate) includes percentages (see [\#2124](https://github.com/w3c/csswg-drafts/issues/2124)).

- Describe 3D sorting more precisely, to explain which descendants are included, and not limit Appendix E reference to steps 1-7 (see [\#926](https://github.com/w3c/csswg-drafts/issues/926)).

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

- [3D matrix](#3d-matrix), in § 2
- [3D rendering context](#3d-rendering-context), in § 2
- [3D transformed element](#3d-transformed-element), in § 2
- [3d transform functions](#3d-transform-functions), in § 12.2
- [accumulated 3D transformation matrix](#accumulated-3d-transformation-matrix), in § 2
- [accumulation for one-based values](#accumulation-for-one-based-values), in § 15
- [backface-visibility](#propdef-backface-visibility), in § 10
- [bottom](#valdef-perspective-origin-bottom), in § 9
- [center](#valdef-perspective-origin-center), in § 9
- [identity transform function](#identity-transform-function), in § 2
- [left](#valdef-perspective-origin-left), in § 9
- [\<length\>](#valdef-perspective-origin-length), in § 9
- [\<length \[0,∞\]\>](#valdef-perspective-length-0), in § 8
- [matrix3d()](#funcdef-matrix3d), in § 12.2
- none
  - [value for perspective](#valdef-perspective-none), in § 8
  - [value for perspective()](#valdef-perspective-func-none), in § 12.2
  - [value for translate, rotate, scale](#valdef-translate-none), in § 5
- [\<percentage\>](#valdef-perspective-origin-percentage), in § 9
- [perspective](#propdef-perspective), in § 8
- [perspective()](#funcdef-perspective), in § 12.2
- [perspective matrix](#perspective-matrix), in § 2
- [perspective-origin](#propdef-perspective-origin), in § 9
- [right](#valdef-perspective-origin-right), in § 9
- [rotate](#propdef-rotate), in § 5
- [rotate3d()](#funcdef-rotate3d), in § 12.2
- [rotateX()](#funcdef-rotatex), in § 12.2
- [rotateY()](#funcdef-rotatey), in § 12.2
- [rotateZ()](#funcdef-rotatez), in § 12.2
- [scale](#propdef-scale), in § 5
- [scale()](#funcdef-scale), in § 12.1
- [scale3d()](#funcdef-scale3d), in § 12.2
- [scaleX()](#funcdef-scalex), in § 12.1
- [scaleY()](#funcdef-scaley), in § 12.1
- [scaleZ()](#funcdef-scalez), in § 12.2
- [top](#valdef-perspective-origin-top), in § 9
- [\<transform-function\>](#typedef-transform-function), in § 12
- [transform-style](#propdef-transform-style), in § 7
- [translate](#propdef-translate), in § 5
- [translate3d()](#funcdef-translate3d), in § 12.2
- [translateZ()](#funcdef-translatez), in § 12.2
- [x](#valdef-rotate-x), in § 5
- [y](#valdef-rotate-y), in § 5
- [z](#valdef-rotate-z), in § 5

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[compositing-1\] defines the following terms:
  - <a id="term-for-propdef-isolation"></a>isolation
  - <a id="term-for-propdef-mix-blend-mode"></a>mix-blend-mode
- \[css-backgrounds-3\] defines the following terms:
  - <a id="term-for-propdef-background-position"></a>background-position
- \[css-cascade-5\] defines the following terms:
  - <a id="term-for-computed-value"></a>computed value
  - <a id="term-for-used-value"></a>used value
- \[css-color-4\] defines the following terms:
  - <a id="term-for-propdef-opacity"></a>opacity
- \[css-contain-1\] defines the following terms:
  - <a id="term-for-propdef-contain"></a>contain
  - <a id="term-for-valdef-contain-paint"></a>paint
  - <a id="term-for-paint-containment"></a>paint containment
- \[css-contain-2\] defines the following terms:
  - <a id="term-for-propdef-content-visibility"></a>content-visibility
- \[css-masking-1\] defines the following terms:
  - <a id="term-for-propdef-clip"></a>clip
  - <a id="term-for-propdef-clip-path"></a>clip-path
  - <a id="term-for-elementdef-clippath"></a>clippath
  - <a id="term-for-elementdef-mask"></a>mask
  - <a id="term-for-propdef-mask-border-source"></a>mask-border-source
  - <a id="term-for-propdef-mask-image"></a>mask-image
- \[css-overflow-3\] defines the following terms:
  - <a id="term-for-valdef-overflow-clip"></a>clip
  - <a id="term-for-propdef-overflow"></a>overflow
  - <a id="term-for-valdef-overflow-visible"></a>visible
- \[css-transforms-1\] defines the following terms:
  - <a id="term-for-2d-matrix"></a>2d matrix
  - <a id="term-for-typedef-transform-list"></a>\<transform-list\>
  - <a id="term-for-containing-block-for-all-descendants"></a>containing block for all descendants
  - <a id="term-for-funcdef-transform-matrix"></a>matrix()
  - <a id="term-for-reference-box"></a>reference box
  - <a id="term-for-funcdef-transform-rotate"></a>rotate()
  - <a id="term-for-propdef-transform"></a>transform
  - <a id="term-for-propdef-transform-origin"></a>transform-origin
  - <a id="term-for-transformable-element"></a>transformable element
  - <a id="term-for-transformation-matrix"></a>transformation matrix
  - <a id="term-for-transformed-element"></a>transformed element
  - <a id="term-for-funcdef-transform-translate"></a>translate()
  - <a id="term-for-funcdef-transform-translatex"></a>translatex()
  - <a id="term-for-funcdef-transform-translatey"></a>translatey()
- \[css-values-4\] defines the following terms:
  - <a id="term-for-mult-comma"></a>\#
  - <a id="term-for-comb-all"></a>&#x26;&#x26;
  - <a id="term-for-comb-comma"></a>,
  - <a id="term-for-angle-value"></a>\<angle\>
  - <a id="term-for-typedef-length-percentage"></a>\<length-percentage\>
  - <a id="term-for-length-value"></a>\<length\>
  - <a id="term-for-number-value"></a>\<number\>
  - <a id="term-for-percentage-value"></a>\<percentage\>
  - <a id="term-for-typedef-position"></a>\<position\>
  - <a id="term-for-zero-value"></a>\<zero\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-accumulation"></a>value accumulation
  - <a id="term-for-addition"></a>value addition
  - <a id="term-for-mult-num-range"></a>{a,b}
  - <a id="term-for-mult-num"></a>{a}
  - <a id="term-for-comb-one"></a>\|
- \[CSS21\] defines the following terms:
  - <a id="term-for-valdef-clip-auto"></a>auto
  - <a id="term-for-x43"></a>stacking context
- \[CSSOM\] defines the following terms:
  - <a id="term-for-dom-window-getcomputedstyle"></a>getComputedStyle(elt)
  - <a id="term-for-resolved-value"></a>resolved value
  - <a id="term-for-resolved-value-special-case-property-like-height"></a>resolved value special case property like height
- \[filter-effects-1\] defines the following terms:
  - <a id="term-for-propdef-filter"></a>filter
- \[HTML\] defines the following terms:
  - <a id="term-for-the-a-element"></a>a
- \[INFRA\] defines the following terms:
  - <a id="term-for-list-append"></a>append
  - <a id="term-for-list"></a>list
- \[motion-1\] defines the following terms:
  - <a id="term-for-propdef-offset"></a>offset
- \[SVG11\] defines the following terms:
  - <a id="term-for-AnimateElement"></a>animate
  - <a id="term-for-SetElement"></a>set
- \[SVG2\] defines the following terms:
  - <a id="term-for-container-element"></a>container element
  - <a id="term-for-elementdef-foreignObject"></a>foreignobject
  - <a id="term-for-elementdef-g"></a>g
  - <a id="term-for-graphics-element"></a>graphics element
  - <a id="term-for-graphics-referencing-element"></a>graphics referencing element
  - <a id="term-for-elementdef-linearGradient"></a>lineargradient
  - <a id="term-for-elementdef-pattern"></a>pattern
  - <a id="term-for-elementdef-radialGradient"></a>radialgradient
  - <a id="term-for-elementdef-svg"></a>svg
  - <a id="term-for-VectorEffectProperty"></a>vector-effect

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-compositing-1"></a>\[COMPOSITING-1\]  
Rik Cabanier; Nikos Andronikos. [Compositing and Blending Level 1](https://www.w3.org/TR/compositing-1/). 13 January 2015. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;compositing-1&#x2F;](https://www.w3.org/TR/compositing-1/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 26 July 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 15 October 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 1 June 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-contain-1"></a>\[CSS-CONTAIN-1\]  
Tab Atkins Jr.; Florian Rivoal. [CSS Containment Module Level 1](https://www.w3.org/TR/css-contain-1/). 22 December 2020. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-1&#x2F;](https://www.w3.org/TR/css-contain-1/)

<a id="biblio-css-masking-1"></a>\[CSS-MASKING-1\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 5 August 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
David Baron; Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 3 June 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 16 October 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-cssom"></a>\[CSSOM\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-filter-effects-1"></a>\[FILTER-EFFECTS-1\]  
Dirk Schulze; Dean Jackson. [Filter Effects Module Level 1](https://www.w3.org/TR/filter-effects-1/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;filter-effects-1&#x2F;](https://www.w3.org/TR/filter-effects-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-motion-1"></a>\[MOTION-1\]  
Dirk Schulze; et al. [Motion Path Module Level 1](https://www.w3.org/TR/motion-1/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;motion-1&#x2F;](https://www.w3.org/TR/motion-1/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-svg11"></a>\[SVG11\]  
Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG11&#x2F;](https://www.w3.org/TR/SVG11/)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

### <a id="informative"></a>Informative References

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 16 December 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-2&#x2F;](https://www.w3.org/TR/css-contain-2/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                                                            | Initial | Applies to             | Inh. | %ages                                                                                                 | Anim­ation type                            | Canonical order | Com­puted value                                                                             | Used value                                                          |
|---------------------|--------------------------------------------------------------------------------------------------|---------|------------------------|------|-------------------------------------------------------------------------------------------------------|-------------------------------------------|-----------------|--------------------------------------------------------------------------------------------|---------------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-backface-visibility①①"></a></span><a href="#propdef-backface-visibility">backface-visibility</a>&#xA;      </strong> | visible \| hidden                                                                                | visible | transformable elements | no   | N/A                                                                                                   | discrete                                  | per grammar     | specified keyword                                                                          |                                                                     |
| <strong><span><a id="ref-for-propdef-perspective①⑨"></a></span><a href="#propdef-perspective">perspective</a>&#xA;      </strong> | none \| \<length \[0,∞\]\>                                                                       | none    | transformable elements | no   | N/A                                                                                                   | by computed value                         | per grammar     | the keyword none or an absolute length                                                     |                                                                     |
| <strong><span><a id="ref-for-propdef-perspective-origin①⑥"></a></span><a href="#propdef-perspective-origin">perspective-origin</a>&#xA;      </strong> | \<position\>                                                                                     | 50% 50% | transformable elements | no   | refer to the size of the reference box                                                                | by computed value                         | per grammar     | see background-position                                                                    |                                                                     |
| <strong><span><a id="ref-for-propdef-rotate①①"></a></span><a href="#propdef-rotate">rotate</a>&#xA;      </strong> | none \| \<angle\> \| \[ x \| y \| z \| \<number\>{3} \] &#x26;&#x26; \<angle\> | none    | transformable elements | no   | n/a                                                                                                   | as SLERP, but see below for none          | per grammar     | the keyword none, or an \<angle\> with an axis consisting of a list of three \<number\>s   |                                                                     |
| <strong><span><a id="ref-for-propdef-scale①①"></a></span><a href="#propdef-scale">scale</a>&#xA;      </strong> | none \| \[ \<number\> \| \<percentage\> \]{1,3}                                                  | none    | transformable elements | no   | n/a                                                                                                   | by computed value, but see below for none | per grammar     | the keyword none, or a list of 3 \<number\>s                                               |                                                                     |
| <strong><span><a id="ref-for-propdef-transform-style①③"></a></span><a href="#propdef-transform-style">transform-style</a>&#xA;      </strong> | flat \| preserve-3d                                                                              | flat    | transformable elements | no   | N/A                                                                                                   | discrete                                  | per grammar     | specified keyword                                                                          | flat if a grouping property is present, specified keyword otherwise |
| <strong><span><a id="ref-for-propdef-translate①①"></a></span><a href="#propdef-translate">translate</a>&#xA;      </strong> | none \| \<length-percentage\> \[ \<length-percentage\> \<length\>? \]?                           | none    | transformable elements | no   | relative to the width of the reference box (for the first value) or the height (for the second value) | by computed value, but see below for none | per grammar     | the keyword none or a pair of computed \<length-percentage\> values and an absolute length |                                                                     |

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> fix this text to add to the text in CSS Transforms 1. [↵](#issue-8241ab72)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> describe how nested 3d-transformed elements render (perhaps with math) [↵](#issue-aae09890)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This example doesn’t follow from the previous text. [↵](#issue-699eabea)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> is it OK to not pop 2D-transformed elements into their own planes? [↵](#issue-d667e5bf)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> what is the impact of backface-visibility on non-transformed or 2D-transformed elements? Do they get popped into their own planes and intersect? [↵](#issue-7b36039d)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> This is a first pass at an attempt to precisely specify how exactly to transform elements using the provided matrices. It might not be ideal, and implementer feedback is encouraged. See [\#912](https://github.com/w3c/csswg-drafts/issues/912). [↵](#issue-d20360cc)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Verify that projection is the distance to the center of projection. [↵](#issue-8678c096)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> We don’t really need to be a stacking context or containing block for perspective, but maybe webcompat means we can’t change this. [↵](#issue-d6818476)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Backface-visibility cannot be tested by only looking at m33. See [\#917](https://github.com/w3c/csswg-drafts/issues/917). [↵](#issue-4984e181)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> formally describe the syntax of the 3D transform functions in SVG, as is done [for the 2-D functions](https://drafts.csswg.org/css-transforms-1/#svg-syntax). [↵](#issue-c054a7a7)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Per [https&#x3A;&#x2F;&#x2F;lists&#x2E;w3&#x2E;org&#x2F;Archives&#x2F;Public&#x2F;www-style&#x2F;2015Mar&#x2F;0371&#x2E;html](https://lists.w3.org/Archives/Public/www-style/2015Mar/0371.html), the WG resolved to add a formula for decomposing a transform into a unified "scale" (the spec already defines how to decompose it into scaleX&#x2F;Y&#x2F;Z), for use by things like SVG’s non-scaling stroke spec&#x2E; [Formula is defined here.](https://www.w3.org/Graphics/SVG/WG/wiki/Proposals/Specifying_decomposition_of_scale) [↵](#issue-b3f9db98)
