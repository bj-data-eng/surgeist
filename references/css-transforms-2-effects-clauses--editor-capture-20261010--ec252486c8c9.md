Attribution and reformatting notice added for Surgeist on 2026-10-10

This bounded, reformatted source excerpt accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Transforms Module Level 2](https://drafts.csswg.org/css-transforms-2/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and excerpt boundary

Retrieved: 2026-10-10. Source status: Editor’s Draft, 30 November 2025.

Full captured HTML SHA-256: `e3de1ccf32db61406fcb5cbb8feb0791b8375844f6f6eebf1cb016e73167f2fe` (369714 bytes).

Bounded conversion input SHA-256: `ec252486c8c9fabeaba61d4f6eebde697c96bc0e74c76f876ed06ea663625105` (83666 bytes). Page revision metadata: `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`.

Retained complete sections, bounded at the next equal-or-higher-level heading: `transform-rendering`, `ctm`, `backface-visibility-property`, `mathematical-description`. The source header and full legal notice are also retained. Other clauses remain available through upstream links; this is explicitly a partial source capture.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table content are checked against the bounded input. Figures remain links to upstream source images. Synthetic table headers and span expansion are representation changes.

Mathematical representation: the nine selected literal TeX matrix/formula paragraphs become literal blocks to preserve every backslash and row separator exactly. One tab-sensitive block uses HTML `pre` and numeric tab entities so GFM list indentation cannot expand literal tabs. Source formula-image links remain upstream. No equation is corrected or harmonized.

Selection status: evidence for resolving Surgeist #356/#563 and their concrete consumers. The current editor text still leaves the recorded questions/conflict open. Any WebKit behavior selected by Surgeist is an explicit compatibility policy, not normative consensus.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Transforms Module Level 2

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 30 November 2025

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-transforms-2/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-transforms-2/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2021/WD-css-transforms-2-20211109/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-transforms-2)

[Inline In Spec](https://drafts.csswg.org/css-transforms-2/#issues-index)

<strong>Editors:</strong>

[Tab Atkins Jr.](https://xanthir.com/contact/) ([Google](https://www.google.com))

[L. David Baron](https://dbaron.org/) ([Google](https://www.google.com))

[Simon Fraser](mailto:simon.fraser@apple.com) ([Apple Inc](https://www.apple.com/))

[Dean Jackson](mailto:dino@apple.com) ([Apple Inc](https://www.apple.com/))

[Theresa O'Connor](mailto:eoconnor@apple.com) ([Apple Inc](http://www.apple.com/))

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-transforms-2/Overview.bs)

<strong>Delta Spec:</strong>

yes

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="transform-rendering"></a>4. The Transform Rendering Model[](#transform-rendering)

This specification extends [CSS Transforms 1 § 2 The Transform Rendering Model](https://drafts.csswg.org/css-transforms-1/#transform-rendering) to account for the existence of three-dimensional transform functions, the Z value of <a id="ref-for-propdef-transform-origin"></a>[transform-origin](https://drafts.csswg.org/css-transforms-1/#propdef-transform-origin), the <a id="ref-for-propdef-perspective④"></a>[perspective](https://drafts.csswg.org/css-transforms-2/#propdef-perspective) property, and a new 3D rendering model that applies when the used value of the transform-style property is preserve-3d.

Three-dimensional transform functions conceptually extend the coordinate space into three dimensions, adding a Z axis perpendicular to the plane of the screen, that increases towards the viewer.

![Demonstration of the initial coordinate space](https://drafts.csswg.org/css-transforms-2/images/coordinates.svg)

Demonstration of the initial coordinate space.

### <a id="3d-transform-rendering"></a>4.1. 3D Transform Rendering[](#3d-transform-rendering)

Normally, elements render as flat planes, and are rendered into the same plane as their stacking context. Often this is the plane shared by the rest of the page. Two-dimensional transform functions can alter the appearance of an element, but that element is still rendered into the same plane as its stacking context.

An element with a three-dimensional transform that is not contained in a <a id="ref-for-3d-rendering-context①"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context) renders with the appropriate transform applied, but does not intersect with any other elements. The three-dimensional transform in this case can be considered just as a painting effect, like two-dimensional transforms. Similarly, the transform does not affect painting order. For example, a transform with a positive Z translation may make an element look larger, but does not cause that element to render in front of elements with no translation in Z.

<a id="issue-aae09890"></a>

<strong>Issue:</strong>

[](#issue-aae09890) describe how nested 3d-transformed elements render (perhaps with math)

<a id="example-20b12368"></a>

<strong>Example:</strong>

[](#example-20b12368)

<a id="issue-699eabea"></a>

<strong>Issue:</strong>

[](#issue-699eabea) This example doesn’t follow from the previous text.

This example shows the effect of three-dimensional transform applied to an element.

``` text
<style>
div {
    height: 150px;
    width: 150px;
}
.container {
    border: 1px solid black;
}
.transformed {
    transform: rotateY(50deg);
}
</style>

<div class="container">
    <div class="transformed"></div>
</div>
```

![Div with a rotateY transform.](https://drafts.csswg.org/css-transforms-2/examples/simple-3d-example.png)

The transform is a 50° rotation about the vertical, Y axis. Note how this makes the blue box appear narrower, but not three-dimensional.

#### <a id="perspective"></a>4.1.1. Perspective[](#perspective)

Perspective can be used to add a feeling of depth to a scene by making elements higher on the Z axis (closer to the viewer) appear larger, and those further away appear smaller. The scaling is proportional to <var>d</var>/(<var>d</var> − <var>Z</var>) where <var>d</var>, the value of <a id="ref-for-propdef-perspective⑤"></a>[perspective](https://drafts.csswg.org/css-transforms-2/#propdef-perspective), is the distance from the drawing plane to the assumed position of the viewer’s eye.

The appearance of perspective can be applied to a 3d-transformed element in two ways. First, the element’s 'transform function list' can contain the <a id="ref-for-funcdef-perspective"></a>[perspective()](https://drafts.csswg.org/css-transforms-2/#funcdef-perspective) function which computes into the element’s 'current transformation matrix'.

Second, the <a id="ref-for-propdef-perspective⑥"></a>[perspective](https://drafts.csswg.org/css-transforms-2/#propdef-perspective) and <a id="ref-for-propdef-perspective-origin③"></a>[perspective-origin](https://drafts.csswg.org/css-transforms-2/#propdef-perspective-origin) properties can be applied to an element to influence the rendering of its 3d-transformed children, giving them a shared perspective that provides the impression of them living in the same three-dimensional scene.

![Diagram of scale vs. Z position](https://drafts.csswg.org/css-transforms-2/images/perspective_distance.png)

Diagrams showing how scaling depends on the <a id="ref-for-propdef-perspective⑦"></a>[perspective](https://drafts.csswg.org/css-transforms-2/#propdef-perspective) property and Z position. In the top diagram, <var>Z</var> is half of <var>d</var>. In order to make it appear that the original circle (solid outline) appears at <var>Z</var> (dashed circle), the circle is scaled up by a factor of two, resulting in the light blue circle. In the bottom diagram, the circle is scaled down by a factor of one-third to make it appear behind the original position.

Normally the assumed position of the viewer’s eye is centered on a drawing. This position can be moved if desired – for example, if a web page contains multiple drawings that should share a common perspective – by setting <a id="ref-for-propdef-perspective-origin④"></a>[perspective-origin](https://drafts.csswg.org/css-transforms-2/#propdef-perspective-origin).

![Diagram of different perspective-origin](https://drafts.csswg.org/css-transforms-2/images/perspective_origin.png)

Diagram showing the effect of moving the perspective origin upward.

<a id="perspective-matrix-computation"></a> The <a id="ref-for-perspective-matrix"></a>[perspective matrix](https://drafts.csswg.org/css-transforms-2/#perspective-matrix) is computed as follows:

1.  Start with the identity matrix.

2.  Translate by the computed X and Y values of <a id="ref-for-propdef-perspective-origin⑤"></a>[perspective-origin](https://drafts.csswg.org/css-transforms-2/#propdef-perspective-origin)

3.  Multiply by the matrix that would be obtained from the <a id="ref-for-funcdef-perspective①"></a>[perspective()](https://drafts.csswg.org/css-transforms-2/#funcdef-perspective) transform function, where the length is provided by the value of the <a id="ref-for-propdef-perspective⑧"></a>[perspective](https://drafts.csswg.org/css-transforms-2/#propdef-perspective) property

4.  Translate by the negated computed X and Y values of <a id="ref-for-propdef-perspective-origin⑥"></a>[perspective-origin](https://drafts.csswg.org/css-transforms-2/#propdef-perspective-origin)

<a id="example-6ad0c4cd"></a>

<strong>Example:</strong>

[](#example-6ad0c4cd)

This example shows how perspective can be used to cause three-dimensional transforms to appear more realistic.

``` text
<style>
div {
  height: 150px;
  width: 150px;
}
.container {
  perspective: 500px;
  border: 1px solid black;
}
.transformed {
  transform: rotateY(50deg);
}
</style>

<div class="container">
  <div class="transformed"></div>
</div>
```

![Div with a rotateY transform, and perspective on its container](https://drafts.csswg.org/css-transforms-2/examples/simple-perspective-example.png)

The inner element has the same transform as in the previous example, but its rendering is now influenced by the perspective property on its parent element. Perspective causes vertices that have positive Z coordinates (closer to the viewer) to be scaled up in X and Y, and those further away (negative Z coordinates) to be scaled down, giving an appearance of depth.

#### <a id="3d-rendering-contexts"></a>4.1.2. 3D Rendering Contexts[](#3d-rendering-contexts)

This section specifies the rendering model for content that uses 3D-transforms and the transform-style property. In order to describe this model, we introduce the concept of a "3D rendering context".

A <a id="ref-for-3d-rendering-context②"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context) is a set of elements rooted in a common ancestor that, for the purposes of 3D-transform rendering, are considered to share a common three-dimensional coordinate system. The front-to-back rendering of elements in the a 3D rendering context depends on their z-position in that three-dimensional space, and, if the 3D transforms on those elements cause them to intersect, then they are rendered with intersection.

The position of each element in that three-dimensional space is determined by <a id="ref-for-accumulated-3d-transformation-matrix"></a>[accumulating](https://drafts.csswg.org/css-transforms-2/#accumulated-3d-transformation-matrix) the transformation matrices up from the given element to the element that establishes the <a id="ref-for-3d-rendering-context③"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context).

Elements establish and participate in 3D rendering contexts as follows:

- A <a id="ref-for-3d-rendering-context④"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context) is established by a <a id="ref-for-transformable-element"></a>[transformable element](https://drafts.csswg.org/css-transforms-1/#transformable-element) whose used value for <a id="ref-for-propdef-transform-style③"></a>[transform-style](https://drafts.csswg.org/css-transforms-2/#propdef-transform-style) is preserve-3d and which itself is not part of a 3D rendering context. An element that establishes a 3D rendering context also participates in that context.

- An element whose used value for <a id="ref-for-propdef-transform-style④"></a>[transform-style](https://drafts.csswg.org/css-transforms-2/#propdef-transform-style) is preserve-3d and which itself participates in a <a id="ref-for-3d-rendering-context⑤"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context), extends that 3D rendering context rather than establishing a new one.

- An element participates in a <a id="ref-for-3d-rendering-context⑥"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context) if its parent establishes or extends a <a id="ref-for-3d-rendering-context⑦"></a>3D rendering context.

Some CSS properties have values that are considered to force "grouping": they require that their element and its descendants are rendered as a group before being composited with other elements; these include opacity, filters and properties that affect clipping. The relevant property values are listed under [grouping property values](https://drafts.csswg.org/css-transforms-2/#grouping-property-values). Consequently, when used on an element with transform-style:preserve-3d, they change the used value to flat and prevent it from creating or extending a <a id="ref-for-3d-rendering-context⑧"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context).

In a 3D rendering context, rendering and sorting of elements is done as follows:

1.  The element establishing the 3D rendering context, and each other 3D transformed element participating in the 3D rendering context, is rendered into its own plane. This plane includes the element’s backgrounds, borders, other box decorations, content, and descendant elements, excluding any descendant elements that have their own plane (and their descendants). This rendering is done according to [CSS 2.1, Appendix E, Section E.2 Painting Order](https://www.w3.org/TR/CSS2/zindex.html#painting-order).

2.  Intersection is performed between this set of planes, according to [Newell’s algorithm](https://en.wikipedia.org/wiki/Newell%27s_algorithm), with the planes transformed by the <a id="ref-for-accumulated-3d-transformation-matrix①"></a>[accumulated 3D transformation matrix](https://drafts.csswg.org/css-transforms-2/#accumulated-3d-transformation-matrix). Coplanar <a id="ref-for-3d-transformed-element"></a>[3D transformed elements](https://drafts.csswg.org/css-transforms-2/#3d-transformed-element) are rendered in painting order.

<a id="issue-d667e5bf"></a>

<strong>Issue:</strong>

[](#issue-d667e5bf) is it OK to not pop 2D-transformed elements into their own planes?

Note: This specification previously defined that the background, borders, and other box decorations of the establishing element were rendered behind the entire 3D scene. This was changed in [\#6238](https://github.com/w3c/csswg-drafts/issues/6238). However, if the definition of 3D Rendering Contexts is changed in the future, it may be worth considering changing back.

Note that elements with transforms which have a negative z-component will render behind the content and untransformed descendants of the establishing element, and that <a id="ref-for-3d-transformed-element①"></a>[3D transformed elements](https://drafts.csswg.org/css-transforms-2/#3d-transformed-element) may interpenetrate with content and untransformed elements.

Note: Because the 3D-transformed elements in a 3D rendering context can all depth-sort and intersect with each other, they are effectively rendered as if they were siblings. The effect of transform-style: preserve-3d can then be thought of as causing all the <a id="ref-for-3d-transformed-element②"></a>[3D transformed elements](https://drafts.csswg.org/css-transforms-2/#3d-transformed-element) in a 3D rendering context to be hoisted up into the establishing element, but still rendered with their [accumulated 3D transformation matrix](#accumulated-3d-transformation-matrix-computation).

<a id="example-0bb57155"></a>

<strong>Example:</strong>

[](#example-0bb57155)

``` text
<style>
div {
  height: 150px;
  width: 150px;
}
.scene {
  background-color: rgba(0, 0, 0, 0.3);
  border: 1px solid black;
  perspective: 500px;
}
.container {
  transform-style: preserve-3d;
}
.container > div {
  position: absolute;
  left: 0;
}
.container > :first-child {
  transform: rotateY(45deg);
  background-color: orange;
  top: 10px;
  height: 135px;
}
.container > :last-child {
  transform: translateZ(40px);
  background-color: rgba(0, 0, 255, 0.6);
  top: 50px;
  height: 100px;
}
</style>

<div class="scene">
  <div class="container">
    Lorem ipsum dolor sit amet, consectetaur adipisicing elit…
    <div></div>
    <div></div>
  </div>
</div>
```

This example shows show elements in a 3D rendering context can intersect. The container element establishes a 3D rendering context for itself and its two children, and the scene element adds perspective to the 3D rendering context. The children intersect with each other, and the orange element also intersects with the container.

![Intersecting sibling elements.](https://drafts.csswg.org/css-transforms-2/examples/3d-intersection.png)

<a id="example-3892c5d2"></a>

<strong>Example:</strong>

[](#example-3892c5d2)

``` text
<style>
div {
  height: 150px;
  width: 150px;
}
.container {
  perspective: 500px;
  border: 1px solid black;
}
.transformed {
  transform: rotateY(50deg);
  background-color: blue;
}
.child {
  transform-origin: top left;
  transform: rotateX(40deg);
  background-color: lime;
}
</style>

<div class="container">
  <div class="transformed">
    <div class="child"></div>
  </div>
</div>
```

This example shows how nested 3D transforms are rendered. The blue div is transformed as in the previous example, with its rendering influenced by the perspective on its parent element. The lime element also has a 3D transform, which is a rotation about the X axis (anchored at the top, by virtue of the transform-origin). However, the lime element is being rendered into the plane of its parent because it is not a member of the same 3D rendering context. Thus the lime element only appears shorter; it does not "pop out" of the blue element.

![Nested 3D transforms, with flattening](https://drafts.csswg.org/css-transforms-2/examples/3d-rendering-context-flat.png)

#### <a id="transformed-element-hierarchies"></a>4.1.3. Transformed element hierarchies[](#transformed-element-hierarchies)

By default, <a id="ref-for-transformed-element"></a>[transformed elements](https://drafts.csswg.org/css-transforms-1/#transformed-element) do not create a <a id="ref-for-3d-rendering-context⑨"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context) and create a flattened representation of their content. However, since it is useful to construct hierarchies of transformed objects that share a common 3-dimensional space, this flattening behavior may be overridden by specifying a value of preserve-3d for the transform-style property. This allows descendants of the transformed element to share the same 3D rendering context. Non-3D-transformed descendants of such elements are rendered into the plane of the element in step C above, but 3D-transformed elements in the same 3D rendering context will "pop out" into their own planes.

<a id="example-1069e278"></a>

<strong>Example:</strong>

[](#example-1069e278)

``` text
<style>
div {
  height: 150px;
  width: 150px;
}
.container {
  perspective: 500px;
  border: 1px solid black;
}
.transformed {
  transform-style: preserve-3d;
  transform: rotateY(50deg);
  background-color: blue;
}
.child {
  transform-origin: top left;
  transform: rotateX(40deg);
  background-color: lime;
}
</style>
```

This example is identical to the previous example, with the addition of <a id="ref-for-propdef-transform-style⑤"></a>[transform-style: preserve-3d](https://drafts.csswg.org/css-transforms-2/#propdef-transform-style) on the blue element. The blue element now extends the 3D rendering context of its container. Now both blue and lime elements share a common three-dimensional space, so the lime element renders as tilting out from its parent, influenced by the perspective on the container.

![Nested 3D transforms, with preserve-3d.](https://drafts.csswg.org/css-transforms-2/examples/3d-rendering-context-3d.png)

#### <a id="accumulated-3d-transformation-matrix-computation"></a>4.1.4. Accumulated 3D Transformation Matrix Computation[](#accumulated-3d-transformation-matrix-computation)

The final value of the transform used to render an element in a <a id="ref-for-3d-rendering-context①⓪"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context) is computed by accumulating an <a id="ref-for-accumulated-3d-transformation-matrix②"></a>[accumulated 3D transformation matrix](https://drafts.csswg.org/css-transforms-2/#accumulated-3d-transformation-matrix) as follows:

1.  Let <var>transform</var> be the identity matrix.

2.  Let <var>current element</var> be the transformed element.

3.  Let <var>parent element</var> be the parent element of the transformed element.

4.  While <var>current element</var> is an element in the transformed element’s <a id="ref-for-3d-rendering-context①①"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context):

    1.  If <var>current element</var> has a value for <a id="ref-for-propdef-transform⑦"></a>[transform](https://drafts.csswg.org/css-transforms-1/#propdef-transform) which is not none, pre-multiply <var>current element</var>’s <a id="ref-for-transformation-matrix"></a>[transformation matrix](https://drafts.csswg.org/css-transforms-1/#transformation-matrix) with the <var>transform</var>.

    2.  Compute a translation matrix which represents the offset (including the scroll offset) of <var>current element</var> from its <var>parent element</var>, and pre-multiply that matrix into the <var>transform</var>.

    3.  If <var>parent element</var> has a value for <a id="ref-for-propdef-perspective⑨"></a>[perspective](https://drafts.csswg.org/css-transforms-2/#propdef-perspective) which is not <a id="ref-for-valdef-perspective-none"></a>[none](https://drafts.csswg.org/css-transforms-2/#valdef-perspective-none), pre-multiply the <var>parent element</var>’s <a id="ref-for-perspective-matrix①"></a>[perspective matrix](https://drafts.csswg.org/css-transforms-2/#perspective-matrix) into the <var>transform</var>.

    4.  Let <var>current element</var> be the <var>parent element</var>.

    5.  Let <var>parent element</var> be the <var>current element</var>’s parent.

Note: as described here, the <a id="ref-for-accumulated-3d-transformation-matrix③"></a>[accumulated 3D transformation matrix](https://drafts.csswg.org/css-transforms-2/#accumulated-3d-transformation-matrix) takes into account offsets (including the scroll offset) generated by the [visual formatting model](https://www.w3.org/TR/CSS2/visuren.html) on the transformed element, and elements in its ancestor chain up to and including the element that establishes the its <a id="ref-for-3d-rendering-context①②"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context).

#### <a id="backface-visibility"></a>4.1.5. Backface Visibility[](#backface-visibility)

Using three-dimensional transforms, it’s possible to transform an element such that its reverse side is visible. 3D-transformed elements show the same content on both sides, so the reverse side looks like a mirror-image of the front side (as if the element were projected onto a sheet of glass). Normally, elements whose reverse side is towards the viewer remain visible. However, the <a id="ref-for-propdef-backface-visibility③"></a>[backface-visibility](#propdef-backface-visibility) property allows the author to make an element invisible when its reverse side is towards the viewer. This behavior is "live"; if an element with <a id="ref-for-propdef-backface-visibility④"></a>backface-visibility: hidden were animating, such that its front and reverse sides were alternately visible, then it would only be visible when the front side were towards the viewer.

Visibility of the reverse side of an element is considered using the <a id="ref-for-accumulated-3d-transformation-matrix④"></a>[accumulated 3D transformation matrix](https://drafts.csswg.org/css-transforms-2/#accumulated-3d-transformation-matrix), and is thus relative to the parent of the element that establishes the <a id="ref-for-3d-rendering-context①③"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context).

Note: This property is useful when you place two elements back-to-back, as you would to create a playing card. Without this property, the front and back elements could switch places at times during an animation to flip the card. Another example is creating a box out of 6 elements, but where you want to see only the inside faces of the box.

<a id="example-551edcaa"></a>

<strong>Example:</strong>

[](#example-551edcaa)

This example shows how to make a "card" element that flips over when clicked. Note the "transform-style: preserve-3d" on \#card which is necessary to avoid flattening when flipped.

``` text
<style>
.body { perspective: 500px; }
#card {
  position: relative;
  height: 300px; width: 200px;
  transition: transform 1s;
  transform-style: preserve-3d;
}
#card.flipped {
  transform: rotateY(180deg);
}
.face {
  position: absolute;
  top: 0; left: 0;
  width: 100%; height: 100%;
  background-color: silver;
  border-radius: 40px;
  backface-visibility: hidden;
}
.back {
  transform: rotateY(180deg);
}
</style>
<div id="card" onclick="this.classList.toggle('flipped')">
  <div class="front face">Front</div>
  <div class="back face">Back</div>
</div>
```

<a id="issue-7b36039d"></a>

<strong>Issue:</strong>

[](#issue-7b36039d) what is the impact of backface-visibility on non-transformed or 2D-transformed elements? Do they get popped into their own planes and intersect?

### <a id="processing-of-perspective-transformed-boxes"></a>4.2. Processing of Perspective-Transformed Boxes[](#processing-of-perspective-transformed-boxes)

<a id="issue-d20360cc"></a>

<strong>Issue:</strong>

[](#issue-d20360cc) This is a first pass at an attempt to precisely specify how exactly to transform elements using the provided matrices. It might not be ideal, and implementer feedback is encouraged. See [\#912](https://github.com/w3c/csswg-drafts/issues/912).

The <a id="ref-for-accumulated-3d-transformation-matrix⑤"></a>[accumulated 3D transformation matrix](https://drafts.csswg.org/css-transforms-2/#accumulated-3d-transformation-matrix) is affected both by the perspective property, and by any perspective() transform function present in the value of the transform property.

This <a id="ref-for-accumulated-3d-transformation-matrix⑥"></a>[accumulated 3D transformation matrix](https://drafts.csswg.org/css-transforms-2/#accumulated-3d-transformation-matrix) is a 4×4 matrix, while the objects to be transformed are two-dimensional boxes. To transform each corner (<var>a</var>, <var>b</var>) of a box, the matrix must first be applied to (<var>a</var>, <var>b</var>, 0, 1), which will result in a four-dimensional point (<var>x</var>, <var>y</var>, <var>z</var>, <var>w</var>). This is transformed back to a three-dimensional point (<var>x</var>′, <var>y</var>′, <var>z</var>′) as follows:

If <var>w</var> \> 0, (<var>x</var>′, <var>y</var>′, <var>z</var>′) = (<var>x</var>/<var>w</var>, <var>y</var>/<var>w</var>, <var>z</var>/<var>w</var>).

If <var>w</var> = 0, (<var>x</var>′, <var>y</var>′, <var>z</var>′) = (<var>x</var> ⋅ <var>n</var>, <var>y</var> ⋅ <var>n</var>, <var>z</var> ⋅ <var>n</var>). <var>n</var> is an implementation-dependent value that should be chosen so that <var>x</var>′ or <var>y</var>′ is much larger than the viewport size, if possible. For example, (5px, 22px, 0px, 0) might become (5000px, 22000px, 0px), with <var>n</var> = 1000, but this value of <var>n</var> would be too small for (0.1px, 0.05px, 0px, 0). This specification does not define the value of <var>n</var> exactly. Conceptually, (<var>x</var>′, <var>y</var>′, <var>z</var>′) is [infinitely far](https://en.wikipedia.org/wiki/Plane_at_infinity) in the direction (<var>x</var>, <var>y</var>, <var>z</var>).

If <var>w</var> \< 0 for all four corners of the transformed box, the box is not rendered.

If <var>w</var> \< 0 for one to three corners of the transformed box, the box must be replaced by a polygon that has any parts with <var>w</var> \< 0 cut out. This will in general be a polygon with three to five vertices, of which exactly two will have <var>w</var> = 0 and the rest <var>w</var> \> 0. These vertices are then transformed to three-dimensional points using the rules just stated. Conceptually, a point with <var>w</var> \< 0 is "behind" the viewer, so should not be visible.

<a id="example-b7386d44"></a>

<strong>Example:</strong>

[](#example-b7386d44)

``` text
.transformed {
  height: 100px;
  width: 100px;
  background: lime;
  transform: perspective(50px) translateZ(100px);
}
```

All of the box’s corners have <var>z</var>-coordinates greater than the perspective. This means that the box is behind the viewer and will not display. Mathematically, the point (<var>x</var>, <var>y</var>) first becomes (<var>x</var>, <var>y</var>, 0, 1), then is translated to (<var>x</var>, <var>y</var>, 100, 1), and then applying the perspective results in (<var>x</var>, <var>y</var>, 100, −1). The <var>w</var>-coordinate is negative, so it does not display. An implementation that doesn’t handle the <var>w</var> \< 0 case separately might incorrectly display this point as (−<var>x</var>, −<var>y</var>, −100), dividing by −1 and mirroring the box.

<a id="example-81d81b28"></a>

<strong>Example:</strong>

[](#example-81d81b28)

``` text
.transformed {
  height: 100px;
  width: 100px;
  background: radial-gradient(yellow, blue);
  transform: perspective(50px) translateZ(50px);
}
```

Here, the box is translated upward so that it sits at the same place the viewer is looking from. This is like bringing the box closer and closer to one’s eye until it fills the entire field of vision. Since the default transform-origin is at the center of the box, which is yellow, the screen will be filled with yellow.

Mathematically, the point (<var>x</var>, <var>y</var>) first becomes (<var>x</var>, <var>y</var>, 0, 1), then is translated to (<var>x</var>, <var>y</var>, 50, 1), then becomes (<var>x</var>, <var>y</var>, 50, 0) after applying perspective. Relative to the transform-origin at the center, the upper-left corner was (−50, −50), so it becomes (−50, −50, 50, 0). This is transformed to something very far to the upper left, such as (−5000, −5000, 5000). Likewise the other corners are sent very far away. The radial gradient is stretched over the whole box, now enormous, so the part that’s visible without scrolling should be the color of the middle pixel: yellow. However, since the box is not actually infinite, the user can still scroll to the edges to see the blue parts.

<a id="example-a8aaa480"></a>

<strong>Example:</strong>

[](#example-a8aaa480)

``` text
.transformed {
  height: 50px;
  width: 50px;
  background: lime;
  border: 25px solid blue;
  transform-origin: left;
  transform: perspective(50px) rotateY(-45deg);
}
```

The box will be rotated toward the viewer, with the left edge staying fixed while the right edge swings closer. The right edge will be at about <var>z</var> = 70.7px, which is closer than the perspective of 50px. Therefore, the rightmost edge will vanish ("behind" the viewer), and the visible part will stretch out infinitely far to the right.

Mathematically, the top right vertex of the box was originally (100, −50), relative to the transform-origin. It is first expanded to (100, −50, 0, 1). After applying the transform specified, this will get mapped to about (70.71, −50, 70.71, −0.4142). This has <var>w</var> = −0.4142 \< 0, so we need to slice away the part of the box with <var>w</var> \< 0. This results in the new top-right vertex being (50, −50, 50, 0). This is then mapped to some faraway point in the same direction, such as (5000, −5000, 5000), which is up and to the right from the transform-origin. Something similar is done to the lower right corner, which gets mapped far down and to the right. The resulting box stretches far past the edge of the screen.

Again, the rendered box is still finite, so the user can scroll to see the whole thing if they choose. However, the right part has been chopped off. No matter how far the user scrolls, the rightmost 30px or so of the original box will not be visible. The blue border was only 25px wide, so it will be visible on the left, top, and bottom, but not the right.

The same basic procedure would apply if one or three vertices had <var>w</var> \< 0. However, in that case the result of truncating the <var>w</var> \< 0 part would be a triangle or pentagon instead of a quadrilateral.

## <a id="ctm"></a>6. Current Transformation Matrix[](#ctm)

<a id="transformation-matrix-computation"></a> The <a id="ref-for-transformation-matrix①"></a>[transformation matrix](https://drafts.csswg.org/css-transforms-1/#transformation-matrix) computation is amended to the following:

The transformation matrix is computed from the <a id="ref-for-propdef-transform⑨"></a>[transform](https://drafts.csswg.org/css-transforms-1/#propdef-transform), <a id="ref-for-propdef-transform-origin①"></a>[transform-origin](https://drafts.csswg.org/css-transforms-1/#propdef-transform-origin), <a id="ref-for-propdef-translate⑧"></a>[translate](https://drafts.csswg.org/css-transforms-2/#propdef-translate), <a id="ref-for-propdef-rotate⑨"></a>[rotate](https://drafts.csswg.org/css-transforms-2/#propdef-rotate), <a id="ref-for-propdef-scale⑧"></a>[scale](https://drafts.csswg.org/css-transforms-2/#propdef-scale), and <a id="ref-for-propdef-offset"></a>[offset](https://drafts.csswg.org/motion-1/#propdef-offset) properties as follows:

1.  Start with the identity matrix.

2.  Translate by the computed X, Y, and Z values of <a id="ref-for-propdef-transform-origin②"></a>[transform-origin](https://drafts.csswg.org/css-transforms-1/#propdef-transform-origin).

3.  Translate by the computed X, Y, and Z values of <a id="ref-for-propdef-translate⑨"></a>[translate](https://drafts.csswg.org/css-transforms-2/#propdef-translate).

4.  Rotate by the computed <a id="ref-for-angle-value⑤"></a>[\<angle\>](https://drafts.csswg.org/css-values-4/#angle-value) about the specified axis of <a id="ref-for-propdef-rotate①⓪"></a>[rotate](https://drafts.csswg.org/css-transforms-2/#propdef-rotate).

5.  Scale by the computed X, Y, and Z values of <a id="ref-for-propdef-scale⑨"></a>[scale](https://drafts.csswg.org/css-transforms-2/#propdef-scale).

6.  Translate and rotate by the transform specified by <a id="ref-for-propdef-offset①"></a>[offset](https://drafts.csswg.org/motion-1/#propdef-offset).

7.  Multiply by each of the transform functions in <a id="ref-for-propdef-transform①⓪"></a>[transform](https://drafts.csswg.org/css-transforms-1/#propdef-transform) from left to right.

8.  Translate by the negated computed X, Y and Z values of <a id="ref-for-propdef-transform-origin③"></a>[transform-origin](https://drafts.csswg.org/css-transforms-1/#propdef-transform-origin).

## <a id="backface-visibility-property"></a>10. The <a id="ref-for-propdef-backface-visibility⑤"></a>[backface-visibility](#propdef-backface-visibility) Property[](#backface-visibility-property)

| Field                                                                                    | Definition                                                                                                                            |
|------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-backface-visibility"></a><strong>backface-visibility</strong>                                                          |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | visible <a id="ref-for-comb-one①⓪"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) hidden                                   |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | visible                                                                                                                               |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | <a id="ref-for-transformable-element⑧"></a>[transformable elements](https://drafts.csswg.org/css-transforms-1/#transformable-element) |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | no                                                                                                                                    |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | N/A                                                                                                                                   |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | specified keyword                                                                                                                     |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                           |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | discrete                                                                                                                              |

A computed value of hidden for <a id="ref-for-propdef-backface-visibility⑥"></a>[backface-visibility](#propdef-backface-visibility) on a <a id="ref-for-transformable-element⑨"></a>[transformable element](https://drafts.csswg.org/css-transforms-1/#transformable-element) that participates in a <a id="ref-for-3d-rendering-context①⑤"></a>[3D rendering context](https://drafts.csswg.org/css-transforms-2/#3d-rendering-context) establishes both a stacking context and a <a id="ref-for-containing-block-for-all-descendants⑤"></a>[containing block for all descendants](https://drafts.csswg.org/css-transforms-1/#containing-block-for-all-descendants).

The visibility of an element with <a id="ref-for-propdef-backface-visibility⑦"></a>[backface-visibility: hidden](#propdef-backface-visibility) is determined as follows:

1.  Compute the element’s <a id="ref-for-accumulated-3d-transformation-matrix⑦"></a>[accumulated 3D transformation matrix](https://drafts.csswg.org/css-transforms-2/#accumulated-3d-transformation-matrix).

2.  If the component of the matrix in row 3, column 3 is negative, then the element should be hidden. Otherwise it is visible.

<a id="issue-4984e181"></a>

<strong>Issue:</strong>

[](#issue-4984e181) Backface-visibility cannot be tested by only looking at m33. See [\#917](https://github.com/w3c/csswg-drafts/issues/917).

Note: The reasoning for this definition is as follows. Assume elements are rectangles in the <var>x</var>–<var>y</var> plane with infinitesimal thickness. The front of the untransformed element has coordinates like (<var>x</var>, <var>y</var>, <var>ε</var>), and the back is (<var>x</var>, <var>y</var>, −<var>ε</var>), for some very small <var>ε</var>. We want to know if after the transformation, the front of the element is closer to the viewer than the back (higher <var>z</var>-value) or further away. The <var>z</var>-coordinate of the front will be m<sub>13</sub><var>x</var> + m<sub>23</sub><var>y</var> + m<sub>33</sub><var>ε</var> + m<sub>43</sub>, before accounting for perspective, and the back will be m<sub>13</sub><var>x</var> + m<sub>23</sub><var>y</var> − m<sub>33</sub><var>ε</var> + m<sub>43</sub>. The first quantity is greater than the second if and only if m<sub>33</sub> \> 0. (If it equals zero, the front and back are equally close to the viewer. This probably means something like a 90-degree rotation, which makes the element invisible anyway, so we don’t really care whether it vanishes.)

## <a id="mathematical-description"></a>16. Mathematical Description of Transform Functions[](#mathematical-description)

Mathematically, all transform functions can be represented as 4x4 transformation matrices of the following form:

``` text
$$\begin{bmatrix} m11 & m21 & m31 & m41 \\ m12 & m22 & m32 & m42 \\ m13 & m23 & m33 & m43 \\ m14 & m24 & m34 & m44 \end{bmatrix}$$
```

One translation unit on a matrix is equivalent to 1 pixel in the local coordinate system of the element.

- <a id="Translate3dDefined"></a> [](#Translate3dDefined) A 3D translation with the parameters <em>tx</em>, <em>ty</em> and <em>tz</em> is equivalent to the matrix:

  ``` text
  $$\begin{bmatrix} 1 & 0 & 0 & tx \\ 0 & 1 & 0 & ty \\ 0 & 0 & 1 & tz \\ 0 & 0 & 0 & 1 \end{bmatrix}$$
  ```

- <a id="Scale3dDefined"></a> [](#Scale3dDefined) A 3D scaling with the parameters <em>sx</em>, <em>sy</em> and <em>sz</em> is equivalent to the matrix:

  ``` text
  $$\begin{bmatrix} sx & 0 & 0 & 0 \\ 0 & sy & 0 & 0 \\ 0 & 0 & sz & 0 \\ 0 & 0 & 0 & 1 \end{bmatrix}$$
  ```

- <a id="Rotate3dDefined"></a> [](#Rotate3dDefined) A 3D rotation with the vector \[x,y,z\] and the parameter <em>alpha</em> is equivalent to the matrix:

  ``` text
  $$\begin{bmatrix} 1 - 2 \cdot (y^2 + z^2) \cdot sq & 2 \cdot (x \cdot y \cdot sq - z \cdot sc) & 2 \cdot (x \cdot z \cdot sq + y \cdot sc) & 0 \\ 2 \cdot (x \cdot y \cdot sq + z \cdot sc) & 1 - 2 \cdot (x^2 + z^2) \cdot sq & 2 \cdot (y \cdot z \cdot sq - x \cdot sc) & 0 \\ 2 \cdot (x \cdot z \cdot sq - y \cdot sc) & 2 \cdot (y \cdot z \cdot sq + x \cdot sc) & 1 - 2 \cdot (x^2 + y^2) \cdot sq & 0 \\ 0 & 0 & 0 & 1 \end{bmatrix}$$
  ```

  where:

  <pre>$$sc = \sin (\alpha/2) \cdot \cos (\alpha/2)$$
&#9;&#9;$$sq = \sin^2 (\alpha/2)$$</pre>

  and where x, y, and z have been normalized (that is, where the x, y, and z values given have been divided by the square root of the sum of their squares).

  Note that this means that a rotation around the X axis simplifies to:
  <strong>Note:</strong>

  ``` text
  $$\begin{bmatrix} 1 & 0 & 0 & 0 \\ 0 & 1 - 2 \cdot sq & -2 \cdot sc & 0 \\ 0 & 2 \cdot sc & 1 - 2 \cdot sq & 0 \\ 0 & 0 & 0 & 1 \end{bmatrix}$$
  ```

  a rotation around the Y axis simplifies to:

  ``` text
  $$\begin{bmatrix} 1 - 2 \cdot sq & 0 & 2 \cdot sc & 0 \\ 0 & 1 & 0 & 0 \\ -2 \cdot sc & 0 & 1 - 2 \cdot sq & 0 \\ 0 & 0 & 0 & 1 \end{bmatrix}$$
  ```

  and a rotation around the Z axis simplifies to:

  ``` text
  $$\begin{bmatrix} 1 - 2 \cdot sq & -2 \cdot sc & 0 & 0 \\ 2 \cdot sc & 1 - 2 \cdot sq & 0 & 0 \\ 0 & 0 & 1 & 0 \\ 0 & 0 & 0 & 1 \end{bmatrix}$$
  ```

- <a id="PerspectiveDefined"></a> [](#PerspectiveDefined) A perspective projection matrix with the parameter <var>d</var> is equivalent to the matrix:

  ``` text
  $$\begin{bmatrix} 1 & 0 & 0 & 0 \\ 0 & 1 & 0 & 0 \\ 0 & 0 & 1 & 0 \\ 0 & 0 & -1/d & 1 \end{bmatrix}$$
  ```

  If the parameter <var>d</var> is <a id="ref-for-valdef-perspective-func-none"></a>[none](https://drafts.csswg.org/css-transforms-2/#valdef-perspective-func-none) it is treated as infinity (and the resulting matrix is the identity matrix).
