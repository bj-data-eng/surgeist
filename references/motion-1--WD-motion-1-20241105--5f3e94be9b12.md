Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [Motion Path Module Level 1](https://www.w3.org/TR/2024/WD-motion-1-20241105/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Motion Path Module Level 1

Source snapshot: https://www.w3.org/TR/2024/WD-motion-1-20241105/

Snapshot SHA-256: 5f3e94be9b120ab48065ecc17cf4a57b47af3fd2859a79c464c406d9996891c5

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 7 source tables are presented as readable Markdown tables or explicit labeled layouts: 7 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>Motion Path Module Level 1

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

Motion path allows authors to position any graphical object and animate it along an author specified path.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of
   its publication. A list of
   current W3C publications and the latest revision of this technical report
   can be found in the <a href="https://www.w3.org/TR/">W3C technical reports
   index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

Publication as a Working Draft does not imply endorsement by W3C and its Members. This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

[GitHub Issues](https://github.com/w3c/fxtf-drafts/issues) are preferred for discussion of this specification. When filing an issue, please put the text “motion” in the title, preferably like this: “\[motion\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-fxtf-archive/), and there is also a [historical archive](https://lists.w3.org/Archives/Public/public-fx/).

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a Working Draft using the [Recommendation track](https://www.w3.org/policies/process/20231103/#recs-and-notes).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/policies/process/20231103/).

## <a id="intro"></a>1. Introduction

<em>This section is not normative.</em>

<a id="ref-for-propdef-transform"></a>

<a id="ref-for-box"></a>

The [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) property and its related properties allow a [box](https://www.w3.org/TR/css-display-3/#box) to be arbitrarily repositioned (and rotated, scaled, etc) relative to its laid out position, without disrupting the layout of any other elements on the page. These positions can be animated or transitioned with CSS, but only in relatively simple ways: moving a box in a straight line from its starting position to its ending position.

<a id="ref-for-propdef-offset"></a>

<a id="ref-for-propdef-offset-anchor"></a>

<a id="ref-for-propdef-offset-path"></a>

<a id="ref-for-propdef-offset-distance"></a>

<a id="ref-for-propdef-offset-rotate"></a>

This specification introduces the [offset](#propdef-offset) shorthand, and its suite of associated longhand properties, which define an <a id="offset-transform"></a>offset transform: a transform which aligns a particular point on an element ([offset-anchor](#propdef-offset-anchor)) to an <a id="offset-position"></a>offset position on a <em>path</em> ([offset-path](#propdef-offset-path) and [offset-distance](#propdef-offset-distance)), and optionally rotates it to follow the path direction ([offset-rotate](#propdef-offset-rotate)).

<a id="ref-for-funcdef-ray"></a>

<a id="ref-for-funcdef-transform-translate"></a>

This allows a number of powerful new transform possibilities, such as positioning using polar coordinates (with the [ray()](#funcdef-ray) function) rather than the standard rectangular coordinates used by the [translate()](https://www.w3.org/TR/css-transforms-1/#funcdef-transform-translate) function, or animating an element <em>along a defined path</em>, making it easy to define complex and beautiful 2d spatial transitions.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0a52f03b"></a> For example, the following picture shows a curving path (indicated with dotted lines), and an airplane graphic positioned at various points along the path. The plane faces in the direction of the path at each position on the path.
>
> ![Example Path](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/motion-path.svg)
>
> <a id="ref-for-propdef-offset-distance①"></a>
>
> The plane is shown at different [offset-distance](#propdef-offset-distance) values: 0%, 50%, and 100%.

### <a id="placement"></a>1.1. Module interactions

This specification defines additional types of transforms (see [\[css-transforms-1\]](#biblio-css-transforms-1)) that can be applied to an element.

<a id="ref-for-propdef-translate"></a>

<a id="ref-for-propdef-rotate"></a>

<a id="ref-for-propdef-scale"></a>

<a id="ref-for-propdef-transform①"></a>

As described in [CSS Transforms 2 § 6 Current Transformation Matrix](https://www.w3.org/TR/css-transforms-2/#ctm), the transforms defined by this document are layered after the individual transform properties ([translate](https://www.w3.org/TR/css-transforms-2/#propdef-translate)/[rotate](https://www.w3.org/TR/css-transforms-2/#propdef-rotate)/[scale](https://www.w3.org/TR/css-transforms-2/#propdef-scale), defined in [\[css-transforms-2\]](#biblio-css-transforms-2)) and before the [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) property (defined in [\[css-transforms-1\]](#biblio-css-transforms-1)).

### <a id="values"></a>1.2. Values

<a id="ref-for-typedef-basic-shape"></a>

<a id="ref-for-typedef-coord-box"></a>

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS21/about.html#property-defs) from [\[CSS21\]](#biblio-css21). The [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) type is defined in CSS Shapes Module Level 1 [\[CSS-SHAPES\]](#biblio-css-shapes). The [\<coord-box\>](https://www.w3.org/TR/css-box-4/#typedef-coord-box) type is defined in CSS Box Model Module Level 3 [\[CSS-BOX-3\]](#biblio-css-box-3). Value types not defined in these specifications are defined in CSS Values and Units Module Level 3 [\[CSS3VAL\]](#biblio-css3val).

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept CSS-wide keywords such as [initial](https://drafts.csswg.org/css-cascade-4/#valdef-all-initial) and [inherit](https://drafts.csswg.org/css-cascade-4/#valdef-all-inherit) as their property value [\[CSS3VAL\]](#biblio-css3val). For readability it has not been repeated explicitly.

## <a id="motion-paths-overview"></a>2. Motion Paths

<a id="ref-for-propdef-offset-path①"></a>

### <a id="offset-path-property"></a>2.1. Defining A Path: the [offset-path](#propdef-offset-path) property

| Field               | Definition                                                                                                                                                                                                                                                                                           |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-offset-path"></a>offset-path                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-coord-box①"></a><a id="ref-for-comb-any"></a><a id="ref-for-typedef-offset-path"></a><a id="ref-for-comb-one"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<offset-path\>](#typedef-offset-path) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<coord-box\>](https://www.w3.org/TR/css-box-4/#typedef-coord-box) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-transformable-element"></a>[transformable elements](https://www.w3.org/TR/css-transforms-1/#transformable-element)                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                                                    |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                                                                               |

Specifies the <a id="offset-path"></a>offset path, a geometrical path the box gets positioned on.

<a id="typedef-offset-path"></a>

<a id="ref-for-funcdef-ray①"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-url-value"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-typedef-basic-shape①"></a>

```text
<offset-path> = <ray()> | <url> | <basic-shape>
```
Values have the following meanings:

<a id="valdef-offset-path-none"></a>none

<a id="ref-for-offset-transform"></a>

The element does not have an [offset transform](#offset-transform).

<a id="ref-for-typedef-coord-box②"></a>

<a id="ref-for-typedef-offset-path①"></a>

<a id="valdef-offset-path-offset-path--coord-box"></a>[\<offset-path\>](#typedef-offset-path) \|\| [\<coord-box\>](https://www.w3.org/TR/css-box-4/#typedef-coord-box)

<a id="ref-for-offset-transform①"></a>

<a id="ref-for-offset-path"></a>

The element has an [offset transform](#offset-transform), defined by some [offset path](#offset-path). See [§ 2.7 Calculating The Offset Transform](#transform) for details on how to calculate the <a id="ref-for-offset-transform②"></a>offset transform.

<a id="ref-for-propdef-transform②"></a>

<a id="ref-for-x43"></a>

All the usual effects of having a [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) apply (such as creating a [stacking context](https://www.w3.org/TR/CSS21/visuren.html#x43), etc.) See [CSS Transforms 1 § 3 The Transform Rendering Model](https://www.w3.org/TR/css-transforms-1/#transform-rendering) for details.

<a id="ref-for-typedef-offset-path②"></a>

<a id="ref-for-propdef-border-radius"></a>

<a id="ref-for-containing-block"></a>

<a id="ref-for-typedef-coord-box③"></a>

If [\<offset-path\>](#typedef-offset-path) is omitted, it defaults to inset(0 round <var>X</var>), where <var>X</var> is the value of [border-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-radius) on the element that establishes the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) for this element. If [\<coord-box\>](https://www.w3.org/TR/css-box-4/#typedef-coord-box) is omitted, it defaults to border-box.

See the specific values (below) for the interpretation of each component.

<a id="ref-for-funcdef-ray②"></a>

<a id="valdef-offset-path-ray"></a>[\<ray()\>](#funcdef-ray)

<a id="ref-for-offset-path①"></a>

<a id="ref-for-ray-origin"></a>

The [offset path](#offset-path) is a line extending from the [origin](#ray-origin) at some angle. See [§ 2.1.1 The ray() Function](#ray-function) for details.

<a id="ref-for-typedef-coord-box④"></a>

<a id="ref-for-basic-shape-reference-box"></a>

The [\<coord-box\>](https://www.w3.org/TR/css-box-4/#typedef-coord-box) provides the [reference box](https://www.w3.org/TR/css-shapes-1/#basic-shape-reference-box) for the ray.

<a id="ref-for-url-value①"></a>

<a id="valdef-offset-path-url"></a>[\<url\>](https://www.w3.org/TR/css-values-4/#url-value)

<a id="ref-for-TermShapeElement"></a>

<a id="ref-for-offset-path②"></a>

<a id="ref-for-TermEquivalentPath"></a>

A URL reference to an SVG [shape element](https://www.w3.org/TR/SVG2/shapes.html#TermShapeElement). The [offset path](#offset-path) is the referenced element’s [equivalent path](https://www.w3.org/TR/SVG2/paths.html#TermEquivalentPath). [\[SVG2\]](#biblio-svg2)

<a id="ref-for-TermShapeElement①"></a>

<a id="ref-for-typedef-basic-shape②"></a>

If the URL does not reference a [shape element](https://www.w3.org/TR/SVG2/shapes.html#TermShapeElement) (because it references a different element, or resolves to a non-SVG document, or doesn’t resolve at all, etc) this behaves as path("m 0 0") (a [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape)) instead.

<a id="ref-for-typedef-coord-box⑤"></a>

<a id="ref-for-TermShapeElement②"></a>

The [\<coord-box\>](https://www.w3.org/TR/css-box-4/#typedef-coord-box) defines the viewport and user coordinate system for the [shape element](https://www.w3.org/TR/SVG2/shapes.html#TermShapeElement), with the origin (the 0,0 point) at the top left corner, and units being 1px in size.

<a id="ref-for-typedef-basic-shape③"></a>

<a id="valdef-offset-path-basic-shape"></a>[\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape)

<a id="ref-for-offset-path③"></a>

<a id="ref-for-basic-shape-equivalent-path"></a>

<a id="ref-for-typedef-basic-shape④"></a>

The [offset path](#offset-path) is the [equivalent path](#basic-shape-equivalent-path) of the [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) function.

<a id="ref-for-typedef-basic-shape⑤"></a>

<a id="ref-for-offset-starting-position"></a>

<a id="ref-for-propdef-offset-position"></a>

For all [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape)s, if they accept an at \<position\> argument but that argument is omitted, and the element defines an [offset starting position](#offset-starting-position) via [offset-position](#propdef-offset-position), it uses the specified <a id="ref-for-offset-starting-position①"></a>offset starting position for that argument. Otherwise it defaults as specified for each function.

<a id="ref-for-typedef-coord-box⑥"></a>

<a id="ref-for-typedef-basic-shape⑥"></a>

The [\<coord-box\>](https://www.w3.org/TR/css-box-4/#typedef-coord-box) provides the \[=/reference box=\] for the [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape).

<a id="ref-for-typedef-coord-box⑦"></a>

<a id="valdef-offset-path-coord-box"></a>[\<coord-box\>](https://www.w3.org/TR/css-box-4/#typedef-coord-box)

<a id="ref-for-typedef-offset-path③"></a>

Defines the box that the [\<offset-path\>](#typedef-offset-path) sizes into.

<a id="ref-for-containing-block①"></a>

In CSS contexts, the boxes being referenced are from the element that establishes the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) for this element.

In SVG contexts, all values behave as view-box.

Tests

- [offset-path-composition.html](https://wpt.fyi/results/css/motion/animation/offset-path-composition.html) [(live test)](http://wpt.live/css/motion/animation/offset-path-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-path-composition.html)
- [offset-path-interpolation-001.html](https://wpt.fyi/results/css/motion/animation/offset-path-interpolation-001.html) [(live test)](http://wpt.live/css/motion/animation/offset-path-interpolation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-path-interpolation-001.html)
- [offset-path-interpolation-002.html](https://wpt.fyi/results/css/motion/animation/offset-path-interpolation-002.html) [(live test)](http://wpt.live/css/motion/animation/offset-path-interpolation-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-path-interpolation-002.html)
- [offset-path-interpolation-003.html](https://wpt.fyi/results/css/motion/animation/offset-path-interpolation-003.html) [(live test)](http://wpt.live/css/motion/animation/offset-path-interpolation-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-path-interpolation-003.html)
- [offset-path-interpolation-004.html](https://wpt.fyi/results/css/motion/animation/offset-path-interpolation-004.html) [(live test)](http://wpt.live/css/motion/animation/offset-path-interpolation-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-path-interpolation-004.html)
- [offset-path-interpolation-005.html](https://wpt.fyi/results/css/motion/animation/offset-path-interpolation-005.html) [(live test)](http://wpt.live/css/motion/animation/offset-path-interpolation-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-path-interpolation-005.html)
- [offset-path-interpolation-006.html](https://wpt.fyi/results/css/motion/animation/offset-path-interpolation-006.html) [(live test)](http://wpt.live/css/motion/animation/offset-path-interpolation-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-path-interpolation-006.html)
- [offset-path-interpolation-007.html](https://wpt.fyi/results/css/motion/animation/offset-path-interpolation-007.html) [(live test)](http://wpt.live/css/motion/animation/offset-path-interpolation-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-path-interpolation-007.html)
- [offset-path-interpolation-008.html](https://wpt.fyi/results/css/motion/animation/offset-path-interpolation-008.html) [(live test)](http://wpt.live/css/motion/animation/offset-path-interpolation-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-path-interpolation-008.html)
- [offset-path-path-interpolation-001.html](https://wpt.fyi/results/css/motion/animation/reftests/offset-path-path-interpolation-001.html) [(live test)](http://wpt.live/css/motion/animation/reftests/offset-path-path-interpolation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/reftests/offset-path-path-interpolation-001.html)
- [offset-path-with-transforms-001.html](https://wpt.fyi/results/css/motion/animation/reftests/offset-path-with-transforms-001.html) [(live test)](http://wpt.live/css/motion/animation/reftests/offset-path-with-transforms-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/reftests/offset-path-with-transforms-001.html)
- [change-offset-path.html](https://wpt.fyi/results/css/motion/change-offset-path.html) [(live test)](http://wpt.live/css/motion/change-offset-path.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/change-offset-path.html)
- [offset-path-coord-box-001.html](https://wpt.fyi/results/css/motion/offset-path-coord-box-001.html) [(live test)](http://wpt.live/css/motion/offset-path-coord-box-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-coord-box-001.html)
- [offset-path-coord-box-002.html](https://wpt.fyi/results/css/motion/offset-path-coord-box-002.html) [(live test)](http://wpt.live/css/motion/offset-path-coord-box-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-coord-box-002.html)
- [offset-path-coord-box-003.html](https://wpt.fyi/results/css/motion/offset-path-coord-box-003.html) [(live test)](http://wpt.live/css/motion/offset-path-coord-box-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-coord-box-003.html)
- [offset-path-coord-box-004.html](https://wpt.fyi/results/css/motion/offset-path-coord-box-004.html) [(live test)](http://wpt.live/css/motion/offset-path-coord-box-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-coord-box-004.html)
- [offset-path-huge-angle-deg-001-crash.html](https://wpt.fyi/results/css/motion/offset-path-huge-angle-deg-001-crash.html) [(live test)](http://wpt.live/css/motion/offset-path-huge-angle-deg-001-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-huge-angle-deg-001-crash.html)
- [offset-path-huge-angle-grad-001-crash.html](https://wpt.fyi/results/css/motion/offset-path-huge-angle-grad-001-crash.html) [(live test)](http://wpt.live/css/motion/offset-path-huge-angle-grad-001-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-huge-angle-grad-001-crash.html)
- [offset-path-huge-angle-turn-001-crash.html](https://wpt.fyi/results/css/motion/offset-path-huge-angle-turn-001-crash.html) [(live test)](http://wpt.live/css/motion/offset-path-huge-angle-turn-001-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-huge-angle-turn-001-crash.html)
- [offset-path-ray-001.html](https://wpt.fyi/results/css/motion/offset-path-ray-001.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-001.html)
- [offset-path-ray-002.html](https://wpt.fyi/results/css/motion/offset-path-ray-002.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-002.html)
- [offset-path-ray-003.html](https://wpt.fyi/results/css/motion/offset-path-ray-003.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-003.html)
- [offset-path-ray-004.html](https://wpt.fyi/results/css/motion/offset-path-ray-004.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-004.html)
- [offset-path-ray-005.html](https://wpt.fyi/results/css/motion/offset-path-ray-005.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-005.html)
- [offset-path-ray-006.html](https://wpt.fyi/results/css/motion/offset-path-ray-006.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-006.html)
- [offset-path-ray-007.html](https://wpt.fyi/results/css/motion/offset-path-ray-007.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-007.html)
- [offset-path-ray-008.html](https://wpt.fyi/results/css/motion/offset-path-ray-008.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-008.html)
- [offset-path-ray-009.html](https://wpt.fyi/results/css/motion/offset-path-ray-009.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-009.html)
- [offset-path-ray-010.html](https://wpt.fyi/results/css/motion/offset-path-ray-010.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-010.html)
- [offset-path-ray-011.html](https://wpt.fyi/results/css/motion/offset-path-ray-011.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-011.html)
- [offset-path-ray-012.html](https://wpt.fyi/results/css/motion/offset-path-ray-012.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-012.html)
- [offset-path-ray-013.html](https://wpt.fyi/results/css/motion/offset-path-ray-013.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-013.html)
- [offset-path-ray-014.html](https://wpt.fyi/results/css/motion/offset-path-ray-014.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-014.html)
- [offset-path-ray-015.html](https://wpt.fyi/results/css/motion/offset-path-ray-015.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-015.html)
- [offset-path-ray-016.html](https://wpt.fyi/results/css/motion/offset-path-ray-016.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-016.html)
- [offset-path-ray-017.html](https://wpt.fyi/results/css/motion/offset-path-ray-017.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-017.html)
- [offset-path-ray-018.html](https://wpt.fyi/results/css/motion/offset-path-ray-018.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-018.html)
- [offset-path-ray-019.html](https://wpt.fyi/results/css/motion/offset-path-ray-019.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-019.html)
- [offset-path-ray-020.html](https://wpt.fyi/results/css/motion/offset-path-ray-020.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-020.html)
- [offset-path-ray-021.html](https://wpt.fyi/results/css/motion/offset-path-ray-021.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-021.html)
- [offset-path-ray-022.html](https://wpt.fyi/results/css/motion/offset-path-ray-022.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-022.html)
- [offset-path-ray-contain-001.html](https://wpt.fyi/results/css/motion/offset-path-ray-contain-001.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-contain-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-contain-001.html)
- [offset-path-ray-contain-002.html](https://wpt.fyi/results/css/motion/offset-path-ray-contain-002.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-contain-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-contain-002.html)
- [offset-path-ray-contain-003.html](https://wpt.fyi/results/css/motion/offset-path-ray-contain-003.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-contain-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-contain-003.html)
- [offset-path-ray-contain-004.html](https://wpt.fyi/results/css/motion/offset-path-ray-contain-004.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-contain-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-contain-004.html)
- [offset-path-ray-contain-005.html](https://wpt.fyi/results/css/motion/offset-path-ray-contain-005.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-contain-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-contain-005.html)
- [offset-path-shape-circle-001.html](https://wpt.fyi/results/css/motion/offset-path-shape-circle-001.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-circle-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-circle-001.html)
- [offset-path-shape-circle-002.html](https://wpt.fyi/results/css/motion/offset-path-shape-circle-002.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-circle-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-circle-002.html)
- [offset-path-shape-circle-003.html](https://wpt.fyi/results/css/motion/offset-path-shape-circle-003.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-circle-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-circle-003.html)
- [offset-path-shape-circle-004.html](https://wpt.fyi/results/css/motion/offset-path-shape-circle-004.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-circle-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-circle-004.html)
- [offset-path-shape-circle-005.html](https://wpt.fyi/results/css/motion/offset-path-shape-circle-005.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-circle-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-circle-005.html)
- [offset-path-shape-circle-006.html](https://wpt.fyi/results/css/motion/offset-path-shape-circle-006.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-circle-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-circle-006.html)
- [offset-path-shape-circle-007.html](https://wpt.fyi/results/css/motion/offset-path-shape-circle-007.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-circle-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-circle-007.html)
- [offset-path-shape-circle-008.html](https://wpt.fyi/results/css/motion/offset-path-shape-circle-008.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-circle-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-circle-008.html)
- [offset-path-shape-ellipse-001.html](https://wpt.fyi/results/css/motion/offset-path-shape-ellipse-001.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-ellipse-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-ellipse-001.html)
- [offset-path-shape-ellipse-002.html](https://wpt.fyi/results/css/motion/offset-path-shape-ellipse-002.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-ellipse-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-ellipse-002.html)
- [offset-path-shape-ellipse-003.html](https://wpt.fyi/results/css/motion/offset-path-shape-ellipse-003.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-ellipse-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-ellipse-003.html)
- [offset-path-shape-ellipse-004.html](https://wpt.fyi/results/css/motion/offset-path-shape-ellipse-004.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-ellipse-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-ellipse-004.html)
- [offset-path-shape-ellipse-005.html](https://wpt.fyi/results/css/motion/offset-path-shape-ellipse-005.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-ellipse-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-ellipse-005.html)
- [offset-path-shape-ellipse-006.html](https://wpt.fyi/results/css/motion/offset-path-shape-ellipse-006.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-ellipse-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-ellipse-006.html)
- [offset-path-shape-ellipse-007.html](https://wpt.fyi/results/css/motion/offset-path-shape-ellipse-007.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-ellipse-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-ellipse-007.html)
- [offset-path-shape-inset-001.html](https://wpt.fyi/results/css/motion/offset-path-shape-inset-001.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-inset-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-inset-001.html)
- [offset-path-shape-inset-002.html](https://wpt.fyi/results/css/motion/offset-path-shape-inset-002.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-inset-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-inset-002.html)
- [offset-path-shape-polygon-001.html](https://wpt.fyi/results/css/motion/offset-path-shape-polygon-001.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-polygon-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-polygon-001.html)
- [offset-path-shape-polygon-002.html](https://wpt.fyi/results/css/motion/offset-path-shape-polygon-002.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-polygon-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-polygon-002.html)
- [offset-path-shape-polygon-003.html](https://wpt.fyi/results/css/motion/offset-path-shape-polygon-003.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-polygon-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-polygon-003.html)
- [offset-path-shape-rect-001.html](https://wpt.fyi/results/css/motion/offset-path-shape-rect-001.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-rect-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-rect-001.html)
- [offset-path-shape-rect-002.html](https://wpt.fyi/results/css/motion/offset-path-shape-rect-002.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-rect-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-rect-002.html)
- [offset-path-shape-rect-003.html](https://wpt.fyi/results/css/motion/offset-path-shape-rect-003.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-rect-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-rect-003.html)
- [offset-path-shape-shape-001.html](https://wpt.fyi/results/css/motion/offset-path-shape-shape-001.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-shape-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-shape-001.html)
- [offset-path-shape-shape-002.html](https://wpt.fyi/results/css/motion/offset-path-shape-shape-002.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-shape-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-shape-002.html)
- [offset-path-shape-shape-003.html](https://wpt.fyi/results/css/motion/offset-path-shape-shape-003.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-shape-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-shape-003.html)
- [offset-path-shape-xywh-001.html](https://wpt.fyi/results/css/motion/offset-path-shape-xywh-001.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-xywh-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-xywh-001.html)
- [offset-path-shape-xywh-002.html](https://wpt.fyi/results/css/motion/offset-path-shape-xywh-002.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-xywh-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-xywh-002.html)
- [offset-path-shape-xywh-003.html](https://wpt.fyi/results/css/motion/offset-path-shape-xywh-003.html) [(live test)](http://wpt.live/css/motion/offset-path-shape-xywh-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-shape-xywh-003.html)
- [offset-path-string-001.html](https://wpt.fyi/results/css/motion/offset-path-string-001.html) [(live test)](http://wpt.live/css/motion/offset-path-string-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-string-001.html)
- [offset-path-string-002.html](https://wpt.fyi/results/css/motion/offset-path-string-002.html) [(live test)](http://wpt.live/css/motion/offset-path-string-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-string-002.html)
- [offset-path-string-003.html](https://wpt.fyi/results/css/motion/offset-path-string-003.html) [(live test)](http://wpt.live/css/motion/offset-path-string-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-string-003.html)
- [offset-path-url-001.html](https://wpt.fyi/results/css/motion/offset-path-url-001.html) [(live test)](http://wpt.live/css/motion/offset-path-url-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-url-001.html)
- [offset-path-url-002.html](https://wpt.fyi/results/css/motion/offset-path-url-002.html) [(live test)](http://wpt.live/css/motion/offset-path-url-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-url-002.html)
- [offset-path-url-003.html](https://wpt.fyi/results/css/motion/offset-path-url-003.html) [(live test)](http://wpt.live/css/motion/offset-path-url-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-url-003.html)
- [offset-path-url-004.html](https://wpt.fyi/results/css/motion/offset-path-url-004.html) [(live test)](http://wpt.live/css/motion/offset-path-url-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-url-004.html)
- [offset-path-url-005.html](https://wpt.fyi/results/css/motion/offset-path-url-005.html) [(live test)](http://wpt.live/css/motion/offset-path-url-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-url-005.html)
- [offset-path-url-006.html](https://wpt.fyi/results/css/motion/offset-path-url-006.html) [(live test)](http://wpt.live/css/motion/offset-path-url-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-url-006.html)
- [offset-path-url-007.html](https://wpt.fyi/results/css/motion/offset-path-url-007.html) [(live test)](http://wpt.live/css/motion/offset-path-url-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-url-007.html)
- [offset-path-url-008.html](https://wpt.fyi/results/css/motion/offset-path-url-008.html) [(live test)](http://wpt.live/css/motion/offset-path-url-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-url-008.html)
- [offset-path-url-009.html](https://wpt.fyi/results/css/motion/offset-path-url-009.html) [(live test)](http://wpt.live/css/motion/offset-path-url-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-url-009.html)
- [offset-path-url-010.html](https://wpt.fyi/results/css/motion/offset-path-url-010.html) [(live test)](http://wpt.live/css/motion/offset-path-url-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-url-010.html)
- [offset-path-url-011.html](https://wpt.fyi/results/css/motion/offset-path-url-011.html) [(live test)](http://wpt.live/css/motion/offset-path-url-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-url-011.html)
- [offset-path-url-crash.html](https://wpt.fyi/results/css/motion/offset-path-url-crash.html) [(live test)](http://wpt.live/css/motion/offset-path-url-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-url-crash.html)
- [offset-path-computed.html](https://wpt.fyi/results/css/motion/parsing/offset-path-computed.html) [(live test)](http://wpt.live/css/motion/parsing/offset-path-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-path-computed.html)
- [offset-path-parsing-invalid.html](https://wpt.fyi/results/css/motion/parsing/offset-path-parsing-invalid.html) [(live test)](http://wpt.live/css/motion/parsing/offset-path-parsing-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-path-parsing-invalid.html)
- [offset-path-parsing-valid.html](https://wpt.fyi/results/css/motion/parsing/offset-path-parsing-valid.html) [(live test)](http://wpt.live/css/motion/parsing/offset-path-parsing-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-path-parsing-valid.html)
- [offset-path-shape-computed.html](https://wpt.fyi/results/css/motion/parsing/offset-path-shape-computed.html) [(live test)](http://wpt.live/css/motion/parsing/offset-path-shape-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-path-shape-computed.html)
- [offset-path-shape-parsing.html](https://wpt.fyi/results/css/motion/parsing/offset-path-shape-parsing.html) [(live test)](http://wpt.live/css/motion/parsing/offset-path-shape-parsing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-path-shape-parsing.html)

<a id="ref-for-funcdef-ray③"></a>

#### <a id="ray-function"></a>2.1.1.  The [ray()](#funcdef-ray) Function

<a id="ref-for-funcdef-ray④"></a>

<a id="ref-for-offset-path④"></a>

The [ray()](#funcdef-ray) function defines an [offset path](#offset-path) as a straight line emerging from a point at some defined angle:

<a id="funcdef-ray"></a>

<a id="ref-for-angle-value"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-typedef-ray-size"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-comb-all①"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-comb-all②"></a>

<a id="ref-for-typedef-position"></a>

<a id="ref-for-mult-opt②"></a>

<a id="typedef-ray-size"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-comb-one⑥"></a>

```text
ray() = ray( <angle> && <ray-size>? && contain? && [at <position>]? )

<ray-size> = closest-side | closest-corner | farthest-side | farthest-corner | sides
```
Its arguments are:

<a id="ref-for-angle-value①"></a>

<a id="valdef-ray-angle"></a>[\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value)

<a id="ref-for-offset-path⑤"></a>

<a id="ref-for-offset-starting-position②"></a>

<a id="ref-for-angle-value②"></a>

<a id="ref-for-gradient-function"></a>

The [offset path](#offset-path) is a single line segment that starts from the [offset starting position](#offset-starting-position) and proceeds in the direction defined by the specified [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value). (Its length is determined by the other arguments.) As with [gradient functions](https://www.w3.org/TR/css-images-4/#gradient-function), <a id="ref-for-angle-value③"></a>\<angle\> values are interpreted as bearing angles, with 0deg pointing up and positive angles representing clockwise rotation.

<a id="valdef-ray-ray-size"></a>\<ray-size\>

<a id="ref-for-offset-path⑥"></a>

<a id="ref-for-propdef-offset-distance②"></a>

Specifies the length of the [offset path](#offset-path) (the distance between the [offset-distance: 0%](#propdef-offset-distance) and <a id="ref-for-propdef-offset-distance③"></a>offset-distance: 100% points) relative to the containing box.

<a id="ref-for-typedef-ray-size①"></a>

<a id="ref-for-size-closest-side"></a>

If no [\<ray-size\>](#typedef-ray-size) is specified it defaults to [closest-side](#size-closest-side).

<a id="ref-for-size-sides"></a>

<a id="ref-for-angle-value④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For [sides](#size-sides), the distance depends on the [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) specified; for all other values, the distance is constant regardless of the <a id="ref-for-angle-value⑤"></a>\<angle\>.

Individual keywords are:

<a id="size-closest-side"></a>closest-side  
<a id="ref-for-containing-block②"></a>

The distance from the ray’s starting point to whichever side of the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) is closest.

<a id="size-closest-corner"></a>closest-corner  
<a id="ref-for-containing-block③"></a>

The distance from the ray’s starting point to whichever corner of the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) is closest.

<a id="size-farthest-side"></a>farthest-side  
<a id="ref-for-containing-block④"></a>

The distance from the ray’s starting point to whichever side of the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) is farthest.

<a id="size-farthest-corner"></a>farthest-corner  
<a id="ref-for-containing-block⑤"></a>

The distance from the ray’s starting point to whichever corner of the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) is farthest.

<a id="size-sides"></a>sides  
<a id="ref-for-offset-path⑦"></a>

<a id="ref-for-containing-block⑥"></a>

The distance from the ray’s starting point to the point where the [offset path](#offset-path) intersects the [containing block’s](https://www.w3.org/TR/css-display-3/#containing-block) boundary.

<a id="ref-for-containing-block⑦"></a>

If the ray’s starting point is on the [containing block’s](https://www.w3.org/TR/css-display-3/#containing-block) boundary, or outside its bounds entirely, the distance is zero.

<a id="ref-for-size-closest-side①"></a>

<a id="ref-for-size-closest-corner"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For [closest-side](#size-closest-side) and [closest-corner](#size-closest-corner), if the ray’s starting point is <em>on</em> an edge/corner, that’s the closest one. (In other words, the distance is zero.)

<a id="ref-for-size-closest-side②"></a>

<a id="ref-for-size-farthest-side"></a>

<a id="ref-for-containing-block⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For [closest-side](#size-closest-side) and [farthest-side](#size-farthest-side), if the ray’s starting point is outside the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) entirely, the edges of the <a id="ref-for-containing-block⑨"></a>containing block are considered to extend out to infinity.

<a id="valdef-ray-contain"></a>contain

<a id="ref-for-offset-path⑧"></a>

<a id="ref-for-containing-block①⓪"></a>

<a id="ref-for-propdef-offset-distance④"></a>

The length of the [offset path](#offset-path) is reduced so that the element stays within the [containing block](https://www.w3.org/TR/css-display-3/#containing-block) even at [offset-distance: 100%](#propdef-offset-distance).

Specifically, the path’s length is reduced by half the width or half the height of the element’s border box, whichever is larger, and floored at zero.

<a id="ref-for-propdef-border-radius①"></a>

<a id="ref-for-funcdef-ray⑤"></a>

<a id="ref-for-size-closest-side③"></a>

<a id="ref-for-propdef-offset-anchor①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> This behavior is optimized for a particular case—​the element’s width and height are equal or nearly so; the element is either completely rounded by [border-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-radius) or the corners aren’t relevant to its appearance; the [ray()](#funcdef-ray) uses [closest-side](#size-closest-side) positioning; and [offset-anchor](#propdef-offset-anchor) is set to center.
>
> <a id="ref-for-propdef-offset-distance⑤"></a>
>
> Under these conditions, which are common for situations like positioning elements around the edge of a round clock face, this ensures that each element is positioned fairly snugly against the inner edge of the clock face at [offset-distance: 100%](#propdef-offset-distance).
>
> In other conditions this will act <em>similarly</em> but might not give quite as optimal a result.

<a id="ref-for-typedef-position①"></a>

<a id="valdef-ray-at-position"></a>at [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position)

<a id="ref-for-typedef-position②"></a>

<a id="ref-for-containing-block①①"></a>

Specifies the <a id="ray-origin"></a>origin of the ray, where the ray’s line begins (the 0% position). It’s resolved by using the [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) to position a 0x0 object area within the box’s [containing block](https://www.w3.org/TR/css-display-3/#containing-block).

<a id="ref-for-offset-starting-position③"></a>

<a id="ref-for-propdef-offset-position①"></a>

If omitted, it uses the [offset starting position](#offset-starting-position) of the element, given by [offset-position](#propdef-offset-position).

<a id="ref-for-offset-starting-position④"></a>

If the element doesn’t have an [offset starting position](#offset-starting-position) either, it behaves as at center.

<a id="ref-for-funcdef-ray⑥"></a>

<a id="ref-for-offset-path⑨"></a>

<a id="ref-for-propdef-offset-position②"></a>

<a id="ref-for-typedef-basic-shape⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [ray()](#funcdef-ray) is currently only usable as an [offset path](#offset-path). If it ever gets extended to other uses, its usage of [offset-position](#propdef-offset-position) will be limited solely to when it’s an <a id="ref-for-offset-path①⓪"></a>offset path, similar to other [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) functions.

Tests

- [ray-angle-interpolation-math-functions.html](https://wpt.fyi/results/css/motion/animation/ray-angle-interpolation-math-functions.html) [(live test)](http://wpt.live/css/motion/animation/ray-angle-interpolation-math-functions.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/ray-angle-interpolation-math-functions.html)
- [offset-path-ray-001.html](https://wpt.fyi/results/css/motion/offset-path-ray-001.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-001.html)
- [offset-path-ray-002.html](https://wpt.fyi/results/css/motion/offset-path-ray-002.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-002.html)
- [offset-path-ray-003.html](https://wpt.fyi/results/css/motion/offset-path-ray-003.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-003.html)
- [offset-path-ray-004.html](https://wpt.fyi/results/css/motion/offset-path-ray-004.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-004.html)
- [offset-path-ray-005.html](https://wpt.fyi/results/css/motion/offset-path-ray-005.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-005.html)
- [offset-path-ray-006.html](https://wpt.fyi/results/css/motion/offset-path-ray-006.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-006.html)
- [offset-path-ray-007.html](https://wpt.fyi/results/css/motion/offset-path-ray-007.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-007.html)
- [offset-path-ray-008.html](https://wpt.fyi/results/css/motion/offset-path-ray-008.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-008.html)
- [offset-path-ray-009.html](https://wpt.fyi/results/css/motion/offset-path-ray-009.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-009.html)
- [offset-path-ray-010.html](https://wpt.fyi/results/css/motion/offset-path-ray-010.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-010.html)
- [offset-path-ray-011.html](https://wpt.fyi/results/css/motion/offset-path-ray-011.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-011.html)
- [offset-path-ray-012.html](https://wpt.fyi/results/css/motion/offset-path-ray-012.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-012.html)
- [offset-path-ray-013.html](https://wpt.fyi/results/css/motion/offset-path-ray-013.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-013.html)
- [offset-path-ray-014.html](https://wpt.fyi/results/css/motion/offset-path-ray-014.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-014.html)
- [offset-path-ray-015.html](https://wpt.fyi/results/css/motion/offset-path-ray-015.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-015.html)
- [offset-path-ray-016.html](https://wpt.fyi/results/css/motion/offset-path-ray-016.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-016.html)
- [offset-path-ray-017.html](https://wpt.fyi/results/css/motion/offset-path-ray-017.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-017.html)
- [offset-path-ray-018.html](https://wpt.fyi/results/css/motion/offset-path-ray-018.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-018.html)
- [offset-path-ray-019.html](https://wpt.fyi/results/css/motion/offset-path-ray-019.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-019.html)
- [offset-path-ray-020.html](https://wpt.fyi/results/css/motion/offset-path-ray-020.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-020.html)
- [offset-path-ray-021.html](https://wpt.fyi/results/css/motion/offset-path-ray-021.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-021.html)
- [offset-path-ray-022.html](https://wpt.fyi/results/css/motion/offset-path-ray-022.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-022.html)
- [offset-path-ray-contain-001.html](https://wpt.fyi/results/css/motion/offset-path-ray-contain-001.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-contain-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-contain-001.html)
- [offset-path-ray-contain-002.html](https://wpt.fyi/results/css/motion/offset-path-ray-contain-002.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-contain-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-contain-002.html)
- [offset-path-ray-contain-003.html](https://wpt.fyi/results/css/motion/offset-path-ray-contain-003.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-contain-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-contain-003.html)
- [offset-path-ray-contain-004.html](https://wpt.fyi/results/css/motion/offset-path-ray-contain-004.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-contain-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-contain-004.html)
- [offset-path-ray-contain-005.html](https://wpt.fyi/results/css/motion/offset-path-ray-contain-005.html) [(live test)](http://wpt.live/css/motion/offset-path-ray-contain-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-path-ray-contain-005.html)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-9aaa1ad0"></a> all of these examples need to be rewritten.

<a id="ref-for-offset-path①①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c2ddc1d0"></a> Here are some examples. The first example shows that some parts of boxes are outside of the [offset path](#offset-path).
>
> ```text
> <style>
>     body {
>         transform-style: preserve-3d;
>         width: 200px;
>         height: 200px;
>     }
>     .box {
>         width: 50px;
>         height: 50px;
>         offset-position: 50% 50%;
>         offset-distance: 100%;
>         offset-rotate: 0deg;
>     }
>     #redBox {
>         background-color: red;
>         offset-path: ray(45deg closest-side);
>     }
>     #blueBox {
>         background-color: blue;
>         offset-path: ray(180deg closest-side);
>     }
> </style>
> <body>
>     <div class="box" id="redBox"></div>
>     <div class="box" id="blueBox"></div>
> </body>
> ```
>
> ![An image of boxes positioned without contain](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/offset_distance_without_contain.png)
>
> <a id="ref-for-propdef-offset-path②"></a>
>
> <a id="ref-for-propdef-contain"></a>
>
> [offset-path](#propdef-offset-path) without [contain](https://www.w3.org/TR/css-contain-2/#propdef-contain)
>
> <a id="ref-for-propdef-contain①"></a>
>
> <a id="ref-for-propdef-offset-path③"></a>
>
> In the second example, [contain](https://www.w3.org/TR/css-contain-2/#propdef-contain) is given to the [offset-path](#propdef-offset-path) value of each box to avoid overflowing.
>
> ```text
> <style>
>     body {
>         transform-style: preserve-3d;
>         width: 200px;
>         height: 200px;
>     }
>     .box {
>         width: 50px;
>         height: 50px;
>         offset-position: 50% 50%;
>         offset-distance: 100%;
>         offset-rotate: 0deg;
>     }
>     #redBox {
>         background-color: red;
>         offset-path: ray(45deg closest-side contain);
>     }
>     #blueBox {
>         background-color: blue;
>         offset-path: ray(180deg closest-side contain);
>     }
> </style>
> <body>
>     <div class="box" id="redBox"></div>
>     <div class="box" id="blueBox"></div>
> </body>
> ```
>
> ![An image of boxes positioned with contain](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/offset_distance_with_contain.png)
>
> <a id="ref-for-propdef-offset-path④"></a>
>
> <a id="ref-for-propdef-contain②"></a>
>
> [offset-path](#propdef-offset-path) with [contain](https://www.w3.org/TR/css-contain-2/#propdef-contain)
>
> <a id="ref-for-used-offset-distance"></a>
>
> In the third example, the path size is increased so that the box can be contained. The [used offset distance](#used-offset-distance) is negative.
>
> ```text
> <style>
>     body {
>         transform-style: preserve-3d;
>         width: 250px;
>         height: 250px;
>     }
>     .box {
>         width: 60%;
>         height: 10%;
> 
>         offset-position: 20% 20%;
>         offset-distance: 0%;
>         offset-rotate: 0deg;
>         offset-anchor: 200% -300%;
>     }
>     #blueBox {
>         background-color: blue;
>         offset-path: ray(-90deg closest-side contain);
>     }
> </style>
> <body>
>     <div class="box" id="blueBox"></div>
> </body>
> ```
>
> ![An image of an increased path size](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/increase-size.svg)
>
> <a id="ref-for-propdef-offset-path⑤"></a>
>
> [offset-path](#propdef-offset-path) with path size increased
>
> In the fourth example, the initial position is outside the containing block.
>
> ```text
> <style>
>     #container {
>         transform-style: preserve-3d;
>         width: 200px;
>         height: 200px;
>     }
>     .box {
>         width: 20%;
>         height: 20%;
>         offset-position: 140% 70%;
>         offset-distance: 100%;
>     }
>     #redBox {
>         background-color: red;
>         offset-path: ray(-90deg sides);
>     }
>     #blueBox {
>         background-color: blue;
>         offset-path: ray(180deg closest-side);
>     }
> </style>
> <div id="container">
>     <div class="box" id="redBox"></div>
>     <div class="box" id="blueBox"></div>
> </div>
> ```
>
> ![An image with initial position outside the containing block](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/initial-outside.svg)
>
> Initial position outside the containing block

<a id="ref-for-typedef-basic-shape⑧"></a>

#### <a id="example-shape"></a>2.1.2. Examples Of [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) Positioning

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-11a5a836"></a> This example uses a circle with implicit center position.
>
> ```text
> <style>
>     body {
>         width: 323px;
>         height: 131px;
>         margin: 0px;
>         border: 2px solid black;
>         padding: 8px;
>         transform-style: preserve-3d;
>     }
>     .item {
>         width:  90px;
>         height: 40px;
>         background-color: violet;
>     }
>     #middle {
>         offset-position: auto;
>         offset-path: circle(60%) margin-box;
>         offset-distance: 25%;
>         offset-anchor: left top;
>     }
> </style>
> <body>
>     <div class="item"></div>
>     <div class="item" id="middle"></div>
>     <div class="item"></div>
> </body>
> ```
>
> ![Normal flow determining circle center](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/normal-flow.svg)
>
> The circle center is determined by normal flow.

<a id="ref-for-typedef-coord-box⑧"></a>

#### <a id="example-coord"></a>2.1.3. Examples of [\<coord-box\>](https://www.w3.org/TR/css-box-4/#typedef-coord-box) Positioning

<a id="ref-for-typedef-coord-box⑨"></a>

<a id="ref-for-offset-path①②"></a>

<a id="ref-for-propdef-border-radius②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1ea93cf3"></a> This example shows how [\<coord-box\>](https://www.w3.org/TR/css-box-4/#typedef-coord-box) [offset path](#offset-path) works in combination with [border-radius](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-radius).
>
> ```text
> <style>
>     body {
>         width: 500px;
>         height: 300px;
>         border-radius: 80px;
>         border: dashed aqua;
>         margin: 0;
>     }
>     #blueBox {
>         width: 40px;
>         height: 20px;
>         background-color: blue;
>         offset-path: margin-box;
>     }
> </style>
> <body>
>     <div id="blueBox"></div>
> </body>
> ```
>
> ![An image of example for geometry-box with border-radius](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/geometry-box.svg)
>
> The initial position is the left end of the top horizontal line.

<a id="ref-for-propdef-offset-distance⑥"></a>

### <a id="offset-distance-property"></a>2.2. Position On The Path: the [offset-distance](#propdef-offset-distance) property

| Field               | Definition                                                                                                                 |
|---------------------|----------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-offset-distance"></a>offset-distance                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage"></a>[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-transformable-element①"></a>[transformable elements](https://www.w3.org/TR/css-transforms-1/#transformable-element)                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-offset-path①③"></a>relative to the [offset path](#offset-path) length                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①"></a>a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                          |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                     |

<a id="ref-for-offset-path①④"></a>

<a id="ref-for-offset-position"></a>

Specifies where along the [offset path](#offset-path) the [offset position](#offset-position) is.

<a id="ref-for-typedef-length-percentage②"></a>

[\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)

<a id="ref-for-offset-position①"></a>

<a id="ref-for-offset-path①⑤"></a>

The [offset position](#offset-position) is the point that is the specified distance along the element’s [offset path](#offset-path). See [§ 2.2.1 Calculating the computed distance along a path](#path-distance) for details about how to calculate distances along a path.

<a id="ref-for-offset-path①⑥"></a>

Percentages are relative to the total length of the [offset path](#offset-path).

Tests

- [offset-distance-composition.html](https://wpt.fyi/results/css/motion/animation/offset-distance-composition.html) [(live test)](http://wpt.live/css/motion/animation/offset-distance-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-distance-composition.html)
- [offset-distance-interpolation.html](https://wpt.fyi/results/css/motion/animation/offset-distance-interpolation.html) [(live test)](http://wpt.live/css/motion/animation/offset-distance-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-distance-interpolation.html)
- [offset-distance-interpolation-001.html](https://wpt.fyi/results/css/motion/animation/reftests/offset-distance-interpolation-001.html) [(live test)](http://wpt.live/css/motion/animation/reftests/offset-distance-interpolation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/reftests/offset-distance-interpolation-001.html)
- [offset-distance-001.html](https://wpt.fyi/results/css/motion/offset-distance-001.html) [(live test)](http://wpt.live/css/motion/offset-distance-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-distance-001.html)
- [offset-distance-002.html](https://wpt.fyi/results/css/motion/offset-distance-002.html) [(live test)](http://wpt.live/css/motion/offset-distance-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-distance-002.html)
- [offset-distance-003.html](https://wpt.fyi/results/css/motion/offset-distance-003.html) [(live test)](http://wpt.live/css/motion/offset-distance-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-distance-003.html)
- [offset-distance-004.html](https://wpt.fyi/results/css/motion/offset-distance-004.html) [(live test)](http://wpt.live/css/motion/offset-distance-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-distance-004.html)
- [offset-distance-005.html](https://wpt.fyi/results/css/motion/offset-distance-005.html) [(live test)](http://wpt.live/css/motion/offset-distance-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-distance-005.html)
- [offset-distance-006.html](https://wpt.fyi/results/css/motion/offset-distance-006.html) [(live test)](http://wpt.live/css/motion/offset-distance-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-distance-006.html)
- [offset-distance-007.html](https://wpt.fyi/results/css/motion/offset-distance-007.html) [(live test)](http://wpt.live/css/motion/offset-distance-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-distance-007.html)
- [offset-distance-008.html](https://wpt.fyi/results/css/motion/offset-distance-008.html) [(live test)](http://wpt.live/css/motion/offset-distance-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-distance-008.html)
- [offset-distance-009.html](https://wpt.fyi/results/css/motion/offset-distance-009.html) [(live test)](http://wpt.live/css/motion/offset-distance-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-distance-009.html)
- [offset-distance-computed.html](https://wpt.fyi/results/css/motion/parsing/offset-distance-computed.html) [(live test)](http://wpt.live/css/motion/parsing/offset-distance-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-distance-computed.html)
- [offset-distance-parsing-invalid.html](https://wpt.fyi/results/css/motion/parsing/offset-distance-parsing-invalid.html) [(live test)](http://wpt.live/css/motion/parsing/offset-distance-parsing-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-distance-parsing-invalid.html)
- [offset-distance-parsing-valid.html](https://wpt.fyi/results/css/motion/parsing/offset-distance-parsing-valid.html) [(live test)](http://wpt.live/css/motion/parsing/offset-distance-parsing-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-distance-parsing-valid.html)

<a id="ref-for-propdef-offset-distance⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: By animating the [offset-distance](#propdef-offset-distance), an element can easily trace out a complex path.

<a id="ref-for-offset-path①⑦"></a>

If the element does not have an [offset path](#offset-path), this property does nothing.

#### <a id="path-distance"></a>2.2.1. Calculating the computed distance along a path

<a id="ref-for-offset-path①⑧"></a>

Processing the distance along an [offset path](#offset-path) operates differently depending upon the nature of the <a id="ref-for-offset-path①⑨"></a>offset path:

- <a id="ref-for-angle-value⑥"></a>

  <a id="ref-for-offset-path②⓪"></a>

  References to [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) [offset path](#offset-path)s with contain are unclosed intervals.

- <a id="ref-for-angle-value⑦"></a>

  <a id="ref-for-offset-path②①"></a>

  References to [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) [offset path](#offset-path)s without contain are unbounded rays.

- All basic CSS shapes are closed loops.

- <a id="ref-for-offset-path②②"></a>

  [Offset path](#offset-path)s (including references to SVG Paths) are closed loops only if the final command in the path list is a closepath command ("z" or "Z"), otherwise they are unclosed intervals.

- References to SVG circles, ellipses, images, polygons and rects are closed loops.

- References to SVG lines and polylines are unclosed intervals.

<a id="ref-for-offset-path②③"></a>

To determine the <a id="used-offset-distance"></a>used offset distance for a given [offset path](#offset-path) and <a id="offset-distance"></a>offset distance:

1.  <a id="ref-for-offset-path②④"></a>

    Let the <a id="total-length"></a>total length be the total length of [offset path](#offset-path) with all sub-paths.

2.  <a id="ref-for-offset-distance"></a>

    <a id="ref-for-total-length"></a>

    Convert [offset distance](#offset-distance) to pixels, with 100% being converted to [total length](#total-length).

3.  <a id="ref-for-offset-path②⑤"></a>

    If [offset path](#offset-path) is an unbounded ray:

    <a id="ref-for-used-offset-distance①"></a>

    <a id="ref-for-offset-distance①"></a>

    Let [used offset distance](#used-offset-distance) be equal to [offset distance](#offset-distance).

    <a id="ref-for-angle-value⑧"></a>

    <a id="ref-for-offset-path②⑥"></a>

    Otherwise if [offset path](#offset-path) is an [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) path with contain:

    <a id="ref-for-used-offset-distance②"></a>

    <a id="ref-for-offset-distance②"></a>

    Let [used offset distance](#used-offset-distance) be equal to [offset distance](#offset-distance), clamped so that the box lies entirely within the path.

    <a id="ref-for-offset-path②⑦"></a>

    If [offset path](#offset-path) is any other unclosed interval:

    <a id="ref-for-used-offset-distance③"></a>

    <a id="ref-for-offset-distance③"></a>

    Let [used offset distance](#used-offset-distance) be equal to [offset distance](#offset-distance) clamped by 0 and the total length of the path.

    <a id="ref-for-offset-path②⑧"></a>

    Otherwise [offset path](#offset-path) is a closed loop:

    <a id="ref-for-used-offset-distance④"></a>

    <a id="ref-for-offset-distance④"></a>

    Let [used offset distance](#used-offset-distance) be equal to [offset distance](#offset-distance) modulo the total length of the path. If the total length of the path is 0, <a id="ref-for-used-offset-distance⑤"></a>used offset distance is also 0.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: “Modulo” here uses the traditional mathematical definition, where the output is always non-negative.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-585ba838"></a> This example shows boxes placed along an unclosed interval.
>
> ```text
> <style>
>     .item {
>         width: 100px;
>         height: 40px;
>         offset-position: 0% 0%;
>         offset-path: path('m 0 0 h 200 v 150');
>     }
>     #box1 {
>         background-color: red;
>         offset-distance: -280%;
>     }
>     #box2 {
>         background-color: green;
>         offset-distance: 190%;
>     }
> </style>
> <body>
>     <div class="item" id="box1"></div>
>     <div class="item" id="box2"></div>
> </body>
> ```
>
> ![An example of boxes placed along an unclosed interval](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/offset-distance-unclosed.svg)
>
> An example of boxes placed along an unclosed interval

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a2477893"></a> This example shows boxes placed along a closed interval.
>
> ```text
> <style>
>     .item {
>         width: 100px;
>         height: 40px;
>         offset-position: 0% 0%;
>         offset-path: path('m 0 0 h 200 v 150 z');
>     }
>     #box1 {
>         background-color: red;
>         offset-distance: -280%;
>     }
>     #box2 {
>         background-color: green;
>         offset-distance: 190%;
>     }
> </style>
> <body>
>     <div class="item" id="box1"></div>
>     <div class="item" id="box2"></div>
> </body>
> ```
>
> ![An example of boxes placed along a closed interval](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/offset-distance-closed.svg)
>
> An example of boxes placed along a closed interval

<a id="ref-for-propdef-offset-path⑥"></a>

<a id="ref-for-propdef-offset-distance⑧"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-acf2c7ce"></a> This example shows a way to align boxes within the polar coordinate system using [offset-path](#propdef-offset-path), [offset-distance](#propdef-offset-distance).
>
> ```text
> <style>
>     body {
>         transform-style: preserve-3d;
>         width: 300px;
>         height: 300px;
>         border: dashed gray;
>         border-radius: 50%;
>     }
>     .circleBox {
>         position: absolute;
>         left: 50%;
>         top: 50%;
>         width: 40px;
>         height: 40px;
>         background-color: red;
>         border-radius: 50%;
>     }
>     #circle1 {
>         offset-path: ray(0deg farthest-side);
>         offset-distance: 50%;
>     }
>     #circle2 {
>         offset-path: ray(90deg farthest-side);
>         offset-distance: 20%;
>     }
>     #circle3 {
>         offset-path: ray(225deg farthest-side);
>         offset-distance: 100%;
>     }
> </style>
> <body>
>     <div class="circleBox" id="circle1"></div>
>     <div class="circleBox" id="circle2"></div>
>     <div class="circleBox" id="circle3"></div>
> </body>
> ```
>
> ![An image of three boxes positioned to polar coordinates](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/simple_offset_position.png)
>
> An example of positioning box in polar coordinates

<a id="ref-for-propdef-offset-position③"></a>

### <a id="offset-position-property"></a>2.3. Starting Point Of The Path: the [offset-position](#propdef-offset-position) property

| Field               | Definition                                                                                                                                                                                                                                 |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-offset-position"></a>offset-position                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-position③"></a><a id="ref-for-comb-one⑦"></a>normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) auto <a id="ref-for-comb-one⑧"></a>\| [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position)                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | normal                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-transformable-element②"></a>[transformable elements](https://www.w3.org/TR/css-transforms-1/#transformable-element)                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | Refer to the size of containing block                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-position④"></a><a id="ref-for-valdef-offset-position-auto"></a><a id="ref-for-valdef-offset-position-normal"></a>The [normal](#valdef-offset-position-normal) or [auto](#valdef-offset-position-auto) keywords, or a computed [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                          |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                     |

<a id="ref-for-typedef-offset-path④"></a>

Specifies the <a id="offset-starting-position"></a>offset starting position that is used by the [\<offset-path\>](#typedef-offset-path) functions if they don’t specify their own starting position.

Values are defined as follows:

<a id="valdef-offset-position-normal"></a>normal

<a id="ref-for-offset-starting-position⑤"></a>

The element does not have an [offset starting position](#offset-starting-position).

<a id="valdef-offset-position-auto"></a>auto

<a id="ref-for-offset-starting-position⑥"></a>

The [offset starting position](#offset-starting-position) is the top-left corner of the box.

<a id="ref-for-containing-block①②"></a>

<a id="ref-for-offsetpath-pathfunc"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is the top-left corner of the element’s <em>own box</em>, not that of its [containing block](https://www.w3.org/TR/css-display-3/#containing-block)! It’s completely different from specifiying top left. It’s meant, for example, to allow a [path()](https://www.w3.org/TR/motion-1/#offsetpath-pathfunc) to start relative to the element’s own position.

<a id="ref-for-typedef-position⑤"></a>

<a id="valdef-offset-position-position"></a>[\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position)

<a id="ref-for-offset-starting-position⑦"></a>

<a id="ref-for-typedef-position⑥"></a>

<a id="ref-for-containing-block①③"></a>

The [offset starting position](#offset-starting-position) is the result of using the [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) to position a 0x0 object area within the box’s [containing block](https://www.w3.org/TR/css-display-3/#containing-block).

Tests

- [offset-position-computed.html](https://wpt.fyi/results/css/motion/parsing/offset-position-computed.html) [(live test)](http://wpt.live/css/motion/parsing/offset-position-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-position-computed.html)
- [offset-position-parsing-invalid.html](https://wpt.fyi/results/css/motion/parsing/offset-position-parsing-invalid.html) [(live test)](http://wpt.live/css/motion/parsing/offset-position-parsing-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-position-parsing-invalid.html)
- [offset-position-parsing-valid.html](https://wpt.fyi/results/css/motion/parsing/offset-position-parsing-valid.html) [(live test)](http://wpt.live/css/motion/parsing/offset-position-parsing-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-position-parsing-valid.html)

<a id="ref-for-propdef-offset-position④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d288583a"></a> This example shows positioning a box with [offset-position](#propdef-offset-position).
>
> ```text
> <style>
>     #wrap {
>         position: relative;
>         width: 300px;
>         height: 300px;
>         border: 1px solid black;
>     }
> 
>     #box {
>         width: 100px;
>         height: 100px;
>         background-color: green;
>         position: absolute;
>         top: 100px;
>         left: 80px;
>         offset-position: auto;
>         offset-anchor: center;
>         offset-path: ray(45deg);
>     }
> </style>
> <body>
>     <div id="wrap">
>         <div id="box"></div>
>     </div>
> </body>
> ```
>
> ![An image of offset-position: auto](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/offset_position_auto.png)
>
> <a id="ref-for-valdef-offset-position-auto①"></a>
>
> <a id="ref-for-propdef-offset-position⑤"></a>
>
> An example when [auto](#valdef-offset-position-auto) is given to [offset-position](#propdef-offset-position)

<a id="ref-for-propdef-transform③"></a>

<a id="ref-for-propdef-rotate①"></a>

<a id="ref-for-propdef-left"></a>

<a id="ref-for-propdef-top"></a>

<a id="ref-for-propdef-offset-position⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-43c03a15"></a> This example shows the interaction with the [transform](https://www.w3.org/TR/css-transforms-1/#propdef-transform) property, and with an individual transform property ([rotate](https://www.w3.org/TR/css-transforms-2/#propdef-rotate)). The motion path transform is a vertical translation moving ([left](https://www.w3.org/TR/css-position-3/#propdef-left), [top](https://www.w3.org/TR/css-position-3/#propdef-top)) to [offset-position](#propdef-offset-position).
>
> ```text
> <style>
>     #wrap {
>         transform-style: preserve-3d;
>         width: 400px;
>         height: 350px;
>     }
>     .item {
>         position: absolute;
>         left: 200px;
>         top: 0px;
>         offset-position: 200px 100px; /* translates by 0px,100px */
>         offset-anchor: left top;
>         transform-origin: left top;
>         width: 130px;
>         height: 80px;
>         border-top-right-radius: 23px;
>     }
>     #box1 {
>         background-color: tomato;
>         offset-position: auto;
>     }
>     #box2 {
>         background-color: green;
>     }
>     #box3 {
>         background-color: navy;
>         rotate: 90deg; /* applied before motion path transform */
>     }
>     #box4 {
>         background-color: gold;
>         transform: rotate(90deg); /* applied after motion path transform */
>     }
> </style>
> <body>
>     <div id="wrap">
>         <div class="item" id="box1"></div>
>         <div class="item" id="box2"></div>
>         <div class="item" id="box3"></div>
>         <div class="item" id="box4"></div>
>     </div>
> </body>
> ```
>
> ![An example when motion path and other transforms interact](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/position-transform.svg)
>
> An example when motion path and other transforms interact

<a id="ref-for-propdef-position"></a>

<a id="ref-for-valdef-position-static"></a>

<a id="ref-for-propdef-offset-position⑦"></a>

<a id="ref-for-propdef-scale①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c1988e48"></a> This example uses [position](https://www.w3.org/TR/css-position-3/#propdef-position) [static](https://www.w3.org/TR/css-position-3/#valdef-position-static), so [offset-position](#propdef-offset-position) generates translations from the normal flow positions. By amplifying these translations using [scale](https://www.w3.org/TR/css-transforms-2/#propdef-scale), the normal flow is rotated 180 degrees around the <a id="ref-for-propdef-offset-position⑧"></a>offset-position, and the boxes are exploded away from each other.
>
> ```text
> <style>
>     #wrap {
>         transform-style: preserve-3d;
>         width: 500px;
>         height: 250px;
>         line-height: 0px;
>     }
>     span {
>         position: static;
>         display: inline-block;
>         width: 100px;
>         height: 50px;
>         border-top-right-radius: 23px;
>         scale: 2.5 2.5; /* applied before motion path transform */
>         offset-position: center;
>         transform: scale(0.4); /* applied after motion path transform */
>     }
>     #box1 {
>         background-color: tomato;
>     }
>     #box2 {
>         background-color: green;
>     }
>     #box3 {
>         background-color: navy;
>     }
>     #box4 {
>         background-color: gold;
>     }
> </style>
> <body>
>     <div id="wrap">
>         <div>
>             <span id="box1"></span><span id="box2"></span>
>         </div>
>         <div>
>             <span id="box3"></span><span id="box4"></span>
>         </div>
>     </div>
> </body>
> ```
>
> ![An example when motion path and scale interact](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/position-scale.svg)
>
> An example when motion path and scale interact

<a id="ref-for-propdef-offset-position⑨"></a>

<a id="ref-for-propdef-offset-path⑦"></a>

<a id="ref-for-typedef-geometry-box"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3480f06b"></a> In this example, each [offset-position](#propdef-offset-position) value is ignored as [offset-path](#propdef-offset-path) is a [\<geometry-box\>](https://www.w3.org/TR/css-masking-1/#typedef-geometry-box), but the other offset properties combine to have an effect equivalent to that for <a id="ref-for-propdef-offset-position①⓪"></a>offset-position 'right bottom'.
>
> ```text
> <style>
>     #wrap {
>         transform-style: preserve-3d;
>         width: 540px;
>         height: 420px;
>     }
>     .item {
>         position: absolute;
>         width: 90px;
>         height: 70px;
>         border-top-right-radius: 23px;
>         scale: 0.8 0.8; /* applied before motion path transform */
>         offset-path: padding-box;
>         offset-distance: 50%;
>         offset-rotate: 0deg;
>         offset-anchor: right bottom;
>         transform: scale(1.25); /* applied after motion path transform */
>     }
>     #box1 {
>         background-color: tomato;
>         position: static;
>         offset-position: auto; /* ignored */
>     }
>     #box2 {
>         background-color: green;
>         right: 0px;
>         top: 0px;
>         offset-position: 23% 45%; /* ignored */
>     }
>     #box3 {
>         background-color: navy;
>         left: 0px;
>         bottom: 0px;
>         offset-position: 34% 56px; /* ignored */
>     }
>     #box4 {
>         background-color: gold;
>         right: 0px;
>         bottom: 0px;
>         offset-position: 45px 67px; /* ignored */
>     }
> </style>
> <body>
>     <div id="wrap">
>         <div class="item" id="box1"></div>
>         <div class="item" id="box2"></div>
>         <div class="item" id="box3"></div>
>         <div class="item" id="box4"></div>
>     </div>
> </body>
> ```
>
> ![An example when offset-position is ignored](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/position-absolute.svg)
>
> An example when offset-position is ignored

<a id="ref-for-propdef-offset-anchor②"></a>

### <a id="offset-anchor-property"></a>2.4. The Element’s Anchor Point: the [offset-anchor](#propdef-offset-anchor) property

| Field               | Definition                                                                                                                                                              |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-offset-anchor"></a>offset-anchor                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-position⑦"></a><a id="ref-for-comb-one⑨"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position)      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-transformable-element③"></a>[transformable elements](https://www.w3.org/TR/css-transforms-1/#transformable-element)                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-basic-shape-reference-box①"></a>relative to the width and the height of the element’s [reference box](https://www.w3.org/TR/css-shapes-1/#basic-shape-reference-box)                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-position⑧"></a><a id="ref-for-valdef-offset-anchor-auto"></a>the [auto](#valdef-offset-anchor-auto) keyword or a computed [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                       |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                  |

<a id="ref-for-offset-position②"></a>

<a id="ref-for-offset-path②⑨"></a>

Defines the element’s <a id="offset-anchor-point"></a>offset anchor point—​the point that is aligned with the [offset position](#offset-position) along the [offset path](#offset-path).

Values have the following meanings:

<a id="valdef-offset-anchor-auto"></a>auto

<a id="ref-for-offset-anchor-point"></a>

<a id="ref-for-propdef-transform-origin"></a>

The [anchor point](#offset-anchor-point) is the same as the point indicated by [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin).

<a id="ref-for-computed-value"></a>

<a id="ref-for-propdef-transform-origin①"></a>

<a id="ref-for-typedef-position⑨"></a>

<a id="ref-for-basic-shape-reference-box②"></a>

Specifically, the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [transform-origin](https://www.w3.org/TR/css-transforms-1/#propdef-transform-origin) is resolved as a [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) against the element’s [reference box](https://www.w3.org/TR/css-shapes-1/#basic-shape-reference-box).

<a id="ref-for-typedef-position①⓪"></a>

<a id="valdef-offset-anchor-position"></a>[\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position)

<a id="ref-for-offset-anchor-point①"></a>

<a id="ref-for-typedef-position①①"></a>

<a id="ref-for-basic-shape-reference-box③"></a>

The [anchor point](#offset-anchor-point) is the result of resolving the [\<position\>](https://www.w3.org/TR/css-values-5/#typedef-position) against the element’s [reference box](https://www.w3.org/TR/css-shapes-1/#basic-shape-reference-box).

Tests

- [offset-anchor-composition.html](https://wpt.fyi/results/css/motion/animation/offset-anchor-composition.html) [(live test)](http://wpt.live/css/motion/animation/offset-anchor-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-anchor-composition.html)
- [offset-anchor-interpolation.html](https://wpt.fyi/results/css/motion/animation/offset-anchor-interpolation.html) [(live test)](http://wpt.live/css/motion/animation/offset-anchor-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-anchor-interpolation.html)
- [offset-anchor-transform-box-fill-box-001.html](https://wpt.fyi/results/css/motion/offset-anchor-transform-box-fill-box-001.html) [(live test)](http://wpt.live/css/motion/offset-anchor-transform-box-fill-box-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-anchor-transform-box-fill-box-001.html)
- [offset-anchor-transform-box-fill-box-002.html](https://wpt.fyi/results/css/motion/offset-anchor-transform-box-fill-box-002.html) [(live test)](http://wpt.live/css/motion/offset-anchor-transform-box-fill-box-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-anchor-transform-box-fill-box-002.html)
- [offset-anchor-transform-box-fill-box-003.html](https://wpt.fyi/results/css/motion/offset-anchor-transform-box-fill-box-003.html) [(live test)](http://wpt.live/css/motion/offset-anchor-transform-box-fill-box-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-anchor-transform-box-fill-box-003.html)
- [offset-anchor-computed.html](https://wpt.fyi/results/css/motion/parsing/offset-anchor-computed.html) [(live test)](http://wpt.live/css/motion/parsing/offset-anchor-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-anchor-computed.html)
- [offset-anchor-parsing-invalid.html](https://wpt.fyi/results/css/motion/parsing/offset-anchor-parsing-invalid.html) [(live test)](http://wpt.live/css/motion/parsing/offset-anchor-parsing-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-anchor-parsing-invalid.html)
- [offset-anchor-parsing-valid.html](https://wpt.fyi/results/css/motion/parsing/offset-anchor-parsing-valid.html) [(live test)](http://wpt.live/css/motion/parsing/offset-anchor-parsing-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-anchor-parsing-valid.html)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-84d4a7e7"></a> Which box this is resolved against is being discussed in [Issue 503](https://github.com/w3c/fxtf-drafts/issues/503).

<a id="ref-for-offset-anchor-point②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-19a6f2eb"></a> The following explains how to set the [anchor point](#offset-anchor-point) of the box.
>
> ```text
> #plane {
>     offset-anchor: center;
> }
> ```
>
> <a id="ref-for-offset-anchor-point③"></a>
>
> The red dot in the middle of the shape indicates the [anchor point](#offset-anchor-point) of the shape.
>
> ![Shape with its anchor point](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/plane.svg)
>
> <a id="ref-for-offset-anchor-point④"></a>
>
> A red dot in the middle of a plane shape indicates the shape’s [anchor point](#offset-anchor-point).

<a id="ref-for-offset-anchor-point⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7adfb83c"></a> This example shows an alignment of four boxes with different [anchor point](#offset-anchor-point)s.
>
> ```text
> <style>
>     body {
>         transform-style: preserve-3d;
>         width: 300px;
>         height: 300px;
>         border: 2px solid gray;
>         border-radius: 50%;
>     }
>     .box {
>         width: 50px;
>         height: 50px;
>         background-color: orange;
>         offset-position: 50% 50%;
>         offset-distance: 100%;
>         offset-rotate: 0deg;
>     }
>     #item1 {
>         offset-path: ray(45deg closest-side);
>         offset-anchor: right top;
>     }
>     #item2 {
>         offset-path: ray(135deg closest-side);
>         offset-anchor: right bottom;
>     }
>     #item3 {
>         offset-path: ray(225deg closest-side);
>         offset-anchor: left bottom;
>     }
>     #item4 {
>         offset-path: ray(315deg closest-side);
>         offset-anchor: left top;
>     }
> </style>
> <body>
>     <div class="box" id="item1"></div>
>     <div class="box" id="item2"></div>
>     <div class="box" id="item3"></div>
>     <div class="box" id="item4"></div>
> </body>
> ```
>
> ![An example of offset-anchor](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/offset_anchor.png)
>
> <a id="ref-for-propdef-offset-anchor③"></a>
>
> An example of [offset-anchor](#propdef-offset-anchor)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-70b8f14f"></a> This example shows boxes centered at their offset-position.
>
> ```text
> <style>
>     body {
>         width: 500px;
>         height: 500px;
>     }
>     .box {
>         background-color: mediumpurple;
>         offset-path: none;
>         offset-anchor: center;
>     }
>     #item1 {
>         offset-position: 90% 20%;
>         width: 60%;
>         height: 20%;
>     }
>     #item2 {
>         offset-position: 100% 100%;
>         width: 30%;
>         height: 10%;
>     }
>     #item3 {
>         offset-position: 50% 100%;
>         width: 20%;
>         height: 60%;
>     }
>     #item4 {
>         offset-position: 0% 100%;
>         width: 30%;
>         height: 90%;
>     }
> </style>
> <body>
>     <div class="box" id="item1"></div>
>     <div class="box" id="item2"></div>
>     <div class="box" id="item3"></div>
>     <div class="box" id="item4"></div>
> </body>
> ```
>
> ![An example of offset-anchor: center](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/offset_anchor_center.svg)
>
> An example of 'offset-anchor: center'

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e236d2d9"></a> This example shows how offset-anchor computes to their offset-position.
>
> ```text
> <style>
>     body {
>         width: 500px;
>         height: 500px;
>     }
>     .box {
>         background-color: mediumpurple;
>         offset-path: none;
>         offset-anchor: auto;
>     }
>     #item1 {
>         offset-position: 90% 20%;
>         width: 60%;
>         height: 20%;
>     }
>     #item2 {
>         offset-position: 100% 100%;
>         width: 30%;
>         height: 10%;
>     }
>     #item3 {
>         offset-position: 50% 100%;
>         width: 20%;
>         height: 60%;
>     }
>     #item4 {
>         offset-position: 0% 100%;
>         width: 30%;
>         height: 90%;
>     }
> </style>
> <body>
>     <div class="box" id="item1"></div>
>     <div class="box" id="item2"></div>
>     <div class="box" id="item3"></div>
>     <div class="box" id="item4"></div>
> </body>
> ```
>
> ![An example of offset-anchor: auto](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/offset_anchor_auto.svg)
>
> An example of 'offset-anchor: auto'

<a id="ref-for-propdef-offset-rotate①"></a>

### <a id="offset-rotate-property"></a>2.5. Rotating To Match The Path: the [offset-rotate](#propdef-offset-rotate) property

| Field               | Definition                                                                                                                                                                                                                                       |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-offset-rotate"></a>offset-rotate                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-angle-value⑨"></a><a id="ref-for-comb-any①"></a><a id="ref-for-comb-one①⓪"></a>\[ auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) reverse \] [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [transformable elements](https://drafts.csswg.org/css-transforms-1/#transformable-element)                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-angle-value①⓪"></a>computed [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) value, optionally preceded by auto                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                |
| <strong>Media:&#xA;      </strong> | visual                                                                                                                                                                                                                                           |

<a id="ref-for-offset-transform③"></a>

<a id="ref-for-offset-path③⓪"></a>

<a id="ref-for-offset-position③"></a>

Defines a rotation component of the [offset transform](#offset-transform), possibly based on the direction of the [offset path](#offset-path) at the [offset position](#offset-position). Values have the following meanings:

<a id="ref-for-angle-value①①"></a>

<a id="valdef-offset-rotate-auto"></a>auto [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value)?

<a id="ref-for-offset-transform④"></a>

<a id="ref-for-offset-path③①"></a>

<a id="ref-for-offset-position④"></a>

<a id="ref-for-TermPathDirection"></a>

The [offset transform](#offset-transform) will have a rotation component equal to the difference between the [offset path’s](#offset-path) direction at the [offset position](#offset-position) and the direction of the positive X axis (that is, a line going toward the right). See SVG’s [direction of a path](https://www.w3.org/TR/SVG2/paths.html#TermPathDirection) for details on how to calculate this.

<a id="ref-for-angle-value①②"></a>

If specified with an [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), the angle is added to the rotation component.

<a id="ref-for-offset-path③②"></a>

<a id="ref-for-valdef-offset-rotate-auto"></a>

<a id="ref-for-angle-value①③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In other words, if the [offset path](#offset-path) is moving to the right, [auto](#valdef-offset-rotate-auto) doesn’t add any rotation. As it diverges from straight rightward, the rotation matches. By combining <a id="ref-for-valdef-offset-rotate-auto①"></a>auto with an [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), you can adjust the "starting" rotation.

<a id="ref-for-angle-value①④"></a>

<a id="valdef-offset-rotate-reverse"></a>reverse [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value)?

<a id="ref-for-valdef-offset-rotate-auto②"></a>

Identical to [auto](#valdef-offset-rotate-auto), but adds an additional 180deg to the rotation.

<a id="ref-for-angle-value①⑤"></a>

<a id="valdef-offset-rotate-angle"></a>[\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value)

<a id="ref-for-offset-transform⑤"></a>

<a id="ref-for-propdef-offset-rotate②"></a>

<a id="ref-for-propdef-transform④"></a>

When specified on its own, adds a rotation component to the [offset transform](#offset-transform) of the specified angle. (That is, [offset-rotate: 45deg;](#propdef-offset-rotate) is similar to [transform: rotate(45deg)](https://www.w3.org/TR/css-transforms-1/#propdef-transform); it’s just ordered to be part of the <a id="ref-for-offset-transform⑥"></a>offset transform.)

Tests

- [offset-rotate-composition.html](https://wpt.fyi/results/css/motion/animation/offset-rotate-composition.html) [(live test)](http://wpt.live/css/motion/animation/offset-rotate-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-rotate-composition.html)
- [offset-rotate-interpolation-math-functions.html](https://wpt.fyi/results/css/motion/animation/offset-rotate-interpolation-math-functions.html) [(live test)](http://wpt.live/css/motion/animation/offset-rotate-interpolation-math-functions.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-rotate-interpolation-math-functions.html)
- [offset-rotate-interpolation.html](https://wpt.fyi/results/css/motion/animation/offset-rotate-interpolation.html) [(live test)](http://wpt.live/css/motion/animation/offset-rotate-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-rotate-interpolation.html)
- [offset-rotate-interpolation-001.html](https://wpt.fyi/results/css/motion/animation/reftests/offset-rotate-interpolation-001.html) [(live test)](http://wpt.live/css/motion/animation/reftests/offset-rotate-interpolation-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/reftests/offset-rotate-interpolation-001.html)
- [offset-rotate-001.html](https://wpt.fyi/results/css/motion/offset-rotate-001.html) [(live test)](http://wpt.live/css/motion/offset-rotate-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-rotate-001.html)
- [offset-rotate-002.html](https://wpt.fyi/results/css/motion/offset-rotate-002.html) [(live test)](http://wpt.live/css/motion/offset-rotate-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-rotate-002.html)
- [offset-rotate-003.html](https://wpt.fyi/results/css/motion/offset-rotate-003.html) [(live test)](http://wpt.live/css/motion/offset-rotate-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-rotate-003.html)
- [offset-rotate-004.html](https://wpt.fyi/results/css/motion/offset-rotate-004.html) [(live test)](http://wpt.live/css/motion/offset-rotate-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-rotate-004.html)
- [offset-rotate-005.html](https://wpt.fyi/results/css/motion/offset-rotate-005.html) [(live test)](http://wpt.live/css/motion/offset-rotate-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-rotate-005.html)
- [offset-rotate-computed.html](https://wpt.fyi/results/css/motion/parsing/offset-rotate-computed.html) [(live test)](http://wpt.live/css/motion/parsing/offset-rotate-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-rotate-computed.html)
- [offset-rotate-parsing-invalid.html](https://wpt.fyi/results/css/motion/parsing/offset-rotate-parsing-invalid.html) [(live test)](http://wpt.live/css/motion/parsing/offset-rotate-parsing-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-rotate-parsing-invalid.html)
- [offset-rotate-parsing-valid.html](https://wpt.fyi/results/css/motion/parsing/offset-rotate-parsing-valid.html) [(live test)](http://wpt.live/css/motion/parsing/offset-rotate-parsing-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-rotate-parsing-valid.html)

<a id="ref-for-offset-anchor-point⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b0a7dcb3"></a> The following examples use the shape of a plane. The red dot in the middle of the shape indicates the [anchor point](#offset-anchor-point) of the shape. When no offset properties are set, the shape is not translated or rotated along the path.
>
> ![Path without offset](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/offset-initial.svg)
>
> A black plane at the beginning of the path, with no offset properties set.
>
> <a id="ref-for-offset-anchor-point⑦"></a>
>
> <a id="ref-for-propdef-offset-rotate③"></a>
>
> When the shape’s [anchor point](#offset-anchor-point) is placed at different positions along the path and [offset-rotate](#propdef-offset-rotate) is 0deg, the shape is not rotated.
>
> ![Path without rotation](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/offset-rotate-none.svg)
>
> A black plane at different positions on a blue dotted path without rotation transforms.
>
> <a id="ref-for-propdef-offset-rotate④"></a>
>
> <a id="ref-for-valdef-offset-rotate-auto③"></a>
>
> <a id="ref-for-offset-anchor-point⑧"></a>
>
> If the [offset-rotate](#propdef-offset-rotate) property is set to [auto](#valdef-offset-rotate-auto), and the shape’s [anchor point](#offset-anchor-point) is placed at different positions along the path, the shape is rotated based on the gradient at the current position and faces the direction of the path at this position.
>
> ![Path with auto rotation](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/offset-rotate-auto.svg)
>
> A black plane at different positions on a blue dotted path, rotated in the direction of the path.
>
> <a id="ref-for-propdef-offset-rotate⑤"></a>
>
> <a id="ref-for-valdef-offset-rotate-reverse"></a>
>
> In this example, the [offset-rotate](#propdef-offset-rotate) property is set to [reverse](#valdef-offset-rotate-reverse). The plane faces the opposite direction of the path at each position on the path.
>
> ![Path with reverse auto rotation](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/offset-rotate-reverse.svg)
>
> A black plane at different positions on a blue dotted path, rotated in the opposite direction of the path.
>
> <a id="ref-for-propdef-offset-rotate⑥"></a>
>
> The last example sets the [offset-rotate](#propdef-offset-rotate) property to -45deg. The shape is rotated anticlockwise by 45 degree once and keeps the rotation at each position on the path.
>
> ![Path with fixed rotation](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/offset-rotate-45.svg)
>
> A black plane at different positions on a blue dotted path, rotated by a fixed amount of degree.

<a id="ref-for-valdef-offset-rotate-auto④"></a>

<a id="ref-for-valdef-offset-rotate-reverse①"></a>

<a id="ref-for-angle-value①⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-552ec5f4"></a> This example shows how [auto](#valdef-offset-rotate-auto) or [reverse](#valdef-offset-rotate-reverse) work when specified in combination with [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value). The computed value of <a id="ref-for-angle-value①⑦"></a>\<angle\> is added to the computed value of <a id="ref-for-valdef-offset-rotate-auto⑤"></a>auto or <a id="ref-for-valdef-offset-rotate-reverse②"></a>reverse.
>
> ```text
> <style>
>     body {
>         width: 300px;
>         height: 300px;
>         margin: 0px;
>         border: solid gray;
>         border-radius: 50%;
>     }
>     .circle {
>         offset-position: 150px 150px;
>         offset-distance: 86%;
>         width: 42px;
>         height: 42px;
>         background-color: mediumpurple;
>         border-radius: 50%;
>         display: flex;
>         align-items: center;
>         justify-content: center;
>     }
>     #item1 {
>         offset-path: ray(0deg closest-side);
>         offset-rotate: auto 90deg;
>     }
>     #item2 {
>         offset-path: ray(45deg closest-side);
>         offset-rotate: auto 90deg;
>     }
>     #item3 {
>         offset-path: ray(135deg closest-side);
>         offset-rotate: auto -90deg;
>     }
>     #item4 {
>         offset-path: ray(180deg closest-side);
>         offset-rotate: auto -90deg;
>     }
>     #item5 {
>         offset-path: ray(225deg closest-side);
>         offset-rotate: reverse 90deg;
>     }
>     #item6 {
>         offset-path: ray(-45deg closest-side);
>         offset-rotate: reverse -90deg;
>     }
> </style>
> <body>
>     <div class="circle" id="item1">1</div>
>     <div class="circle" id="item2">2</div>
>     <div class="circle" id="item3">3</div>
>     <div class="circle" id="item4">4</div>
>     <div class="circle" id="item5">5</div>
>     <div class="circle" id="item6">6</div>
> </body>
> ```
>
> ![An image of example for offset-rotate](https://www.w3.org/TR/2024/WD-motion-1-20241105/images/rotate_by_angle_with_auto.png)
>
> <a id="ref-for-valdef-offset-rotate-auto⑥"></a>
>
> The boxes are rotated by the value of [auto](#valdef-offset-rotate-auto) with a fixed amount of degree.

<a id="ref-for-propdef-offset①"></a>

### <a id="offset-shorthand"></a>2.6. The [offset](#propdef-offset) Shorthand

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-offset"></a>offset                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt⑥"></a><a id="ref-for-propdef-offset-anchor④"></a><a id="ref-for-mult-req"></a><a id="ref-for-propdef-offset-rotate⑦"></a><a id="ref-for-comb-any②"></a><a id="ref-for-propdef-offset-distance⑨"></a><a id="ref-for-propdef-offset-path⑧"></a><a id="ref-for-mult-opt③"></a><a id="ref-for-propdef-offset-position①①"></a>\[ [\<'offset-position'\>](#propdef-offset-position)[?](https://www.w3.org/TR/css-values-4/#mult-opt) \[ [\<'offset-path'\>](#propdef-offset-path) \[ [\<'offset-distance'\>](#propdef-offset-distance) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'offset-rotate'\>](#propdef-offset-rotate) \]<a id="ref-for-mult-opt④"></a>? \]<a id="ref-for-mult-opt⑤"></a>? \][!](https://www.w3.org/TR/css-values-4/#mult-req) \[ / [\<'offset-anchor'\>](#propdef-offset-anchor) \][?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [transformable elements](https://drafts.csswg.org/css-transforms-1/#transformable-element)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |

Tests

- [offset-interpolation.html](https://wpt.fyi/results/css/motion/animation/offset-interpolation.html) [(live test)](http://wpt.live/css/motion/animation/offset-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-interpolation.html)
- [offset-position-composition.html](https://wpt.fyi/results/css/motion/animation/offset-position-composition.html) [(live test)](http://wpt.live/css/motion/animation/offset-position-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-position-composition.html)
- [offset-position-interpolation.html](https://wpt.fyi/results/css/motion/animation/offset-position-interpolation.html) [(live test)](http://wpt.live/css/motion/animation/offset-position-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/animation/offset-position-interpolation.html)
- [offset-parsing-invalid.html](https://wpt.fyi/results/css/motion/parsing/offset-parsing-invalid.html) [(live test)](http://wpt.live/css/motion/parsing/offset-parsing-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-parsing-invalid.html)
- [offset-parsing-valid.html](https://wpt.fyi/results/css/motion/parsing/offset-parsing-valid.html) [(live test)](http://wpt.live/css/motion/parsing/offset-parsing-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-parsing-valid.html)
- [offset-shorthand.html](https://wpt.fyi/results/css/motion/parsing/offset-shorthand.html) [(live test)](http://wpt.live/css/motion/parsing/offset-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/parsing/offset-shorthand.html)
- [inheritance.html](https://wpt.fyi/results/css/motion/inheritance.html) [(live test)](http://wpt.live/css/motion/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/inheritance.html)
- [offset-supports-calc.html](https://wpt.fyi/results/css/motion/offset-supports-calc.html) [(live test)](http://wpt.live/css/motion/offset-supports-calc.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/motion/offset-supports-calc.html)

<a id="ref-for-propdef-offset-position①②"></a>

<a id="ref-for-propdef-offset-path⑨"></a>

<a id="ref-for-propdef-offset-distance①⓪"></a>

<a id="ref-for-propdef-offset-rotate⑧"></a>

<a id="ref-for-propdef-offset-anchor⑤"></a>

This is a shorthand property for setting [offset-position](#propdef-offset-position), [offset-path](#propdef-offset-path), [offset-distance](#propdef-offset-distance), [offset-rotate](#propdef-offset-rotate) and [offset-anchor](#propdef-offset-anchor). Omitted values are set to their initial values.

### <a id="transform"></a>2.7. Calculating The Offset Transform

<a id="ref-for-offset-transform⑦"></a>

The [offset transform](#offset-transform) is a 2d transform, a translation followed by a rotation:

- <a id="ref-for-offset-anchor-point⑨"></a>

  <a id="ref-for-offset-position⑤"></a>

  Translate the element by the (X, Y) that aligns its [anchor point](#offset-anchor-point) with its [offset position](#offset-position).

- <a id="ref-for-propdef-offset-rotate⑨"></a>

  Rotate the element by the angle specified by [offset-rotate](#propdef-offset-rotate).

<a id="ref-for-typedef-basic-shape⑨"></a>

## <a id="paths"></a>3. Equivalent Paths For [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape)

<a id="ref-for-typedef-basic-shape①⓪"></a>

The [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) definition given by [\[css-shapes\]](#biblio-css-shapes) defines each function as producing a <em>shape</em>—​a 2-dimensional figure with an outline, an inside, and an outside.

<a id="ref-for-typedef-basic-shape①①"></a>

This specification instead uses [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) as producing a <em>path</em>—​a line with a starting point, ending point, and direction, that happens to trace out a particular shape’s outline. The details of what makes up a path are defined by SVG. [\[SVG2\]](#biblio-svg2)

<a id="ref-for-typedef-basic-shape①②"></a>

The <a id="basic-shape-equivalent-path"></a>equivalent path for all the [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) values are:

<a id="ref-for-funcdef-basic-shape-path"></a>

<a id="path-equivalent-path"></a>[\<path()\>](https://www.w3.org/TR/css-shapes-1/#funcdef-basic-shape-path)

<a id="ref-for-funcdef-shape"></a>

<a id="shape-equivalent-path"></a>[\<shape()\>](https://drafts.csswg.org/css-shapes-2/#funcdef-shape)

The path is the defined path. [\[SVG11\]](#biblio-svg11)

<a id="ref-for-funcdef-basic-shape-circle"></a>

<a id="circle-equivalent-path"></a>[\<circle()\>](https://www.w3.org/TR/css-shapes-1/#funcdef-basic-shape-circle)

<a id="ref-for-funcdef-basic-shape-ellipse"></a>

<a id="ellipse-equivalent-path"></a>[\<ellipse()\>](https://www.w3.org/TR/css-shapes-1/#funcdef-basic-shape-ellipse)

<a id="ref-for-TermSegment-CompletingClosePath"></a>

The path is the outline of the circle/ellipse. It starts at the rightmost point of the circle/ellipse, and then is composed of four circular arcs, each comprising a quarter of the circle/ellipse, proceeding clockwise, ending with a [segment-completing close path](https://www.w3.org/TR/SVG2/paths.html#TermSegment-CompletingClosePath) operation.

<a id="rect-equivalent-path"></a>rect()

<a id="inset-equivalent-path"></a>inset()

<a id="xywh-equivalent-path"></a>xywh()

<a id="ref-for-TermSegment-CompletingClosePath①"></a>

The path is the outline of the (possibly-rounded) rectangle, composed of four or eight segments (depending on whether rounded corners are specified or not), and ending with a [segment-completing close path](https://www.w3.org/TR/SVG2/paths.html#TermSegment-CompletingClosePath) operation. It starts at the left end of the top straight edge, immediately to the right of any rounded corners, and continues to the right (clockwise).

<a id="ref-for-funcdef-basic-shape-polygon"></a>

<a id="polygon-equivalent-path"></a>[\<polygon()\>](https://www.w3.org/TR/css-shapes-1/#funcdef-basic-shape-polygon)

<a id="ref-for-TermSegment-CompletingClosePath②"></a>

The path is the outline of the polygon, composed of straight line segments connecting each coordinate pair to the following coordinate pair, and finally connecting the last back to the first, with a [segment-completing close path](https://www.w3.org/TR/SVG2/paths.html#TermSegment-CompletingClosePath) operation.

For all of these, the direction at any point along the path is defined by SVG; see [SVG 2 § 9.4 Path directionality](https://www.w3.org/TR/SVG2/paths.html#PathDirectionality).

<a id="ref-for-TermShapeElement③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: All of these are meant to match the "equivalent paths" defined for the similar SVG [shape elements](https://www.w3.org/TR/SVG2/shapes.html#TermShapeElement).

<a id="ref-for-typedef-basic-shape①③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This list should be in sync with the full set of [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) functions defined in [\[css-shapes\]](#biblio-css-shapes). If anything is missing, this should be considered a specification bug. This list might move to Shapes in the future, but for now is kept here as this spec is the only consumer of this information.

## <a id="privacy"></a>4. Privacy Considerations

This specification introduces no new privacy considerations.

## <a id="security"></a>5. Security Considerations

This specification introduces no new security considerations.

## <a id="changes"></a>Changes

<em>This section is non-normative.</em>

### <a id="changes-20181218"></a>Changes since the [18 December 2018](https://www.w3.org/TR/2018/WD-motion-1-20181218/) Working Draft

- Added offset-position:normal that doesn’t override the normal position defaulting, and add an "at \<position\>"" to ray() [\#504](https://github.com/w3c/fxtf-drafts/issues/504).

- Corrected offset-path to use \<url\> instead of \<url()\> [\#508](https://github.com/w3c/fxtf-drafts/issues/508)

- Reworded the calculation of the offset transform, using terminology from CSS Transforms 1

- Clarified "offset-distance" and "offset-rotate"

- Simplified and clarified the behavior of the "contain" keyword [\#363](https://github.com/w3c/fxtf-drafts/issues/363).

- Changed the equivalent path of a circle()/ellipse() to match SVG [\#506](https://github.com/w3c/fxtf-drafts/issues/506).

- The element being referenced by coord-box is the element establishing the containing block for the transformed element, and in an SVG context coord-box is treated as view-box [\#369](https://github.com/w3c/fxtf-drafts/issues/369#issuecomment-1457239856)

- Moved the definition of \<basic-shape\> paths to an appendix.

- Allowed \<coord-box\> to be combined with any of the path functions [\#369](https://github.com/w3c/fxtf-drafts/issues/369#issuecomment-577787797)

- Added inline links to some issues: [\#503](https://github.com/w3c/fxtf-drafts/issues/503), [\#504](https://github.com/w3c/fxtf-drafts/issues/504)

- Clarified initial position

- Moved path() to the \<basic-shape\> section.

- Made \<ray-size\> optional, defaulting to "closest-side".

- Rewrote introduction.

- Moved ray() definition to its own subsection

- Clarified definition of offset path [\#66](https://github.com/w3c/fxtf-drafts/issues/66)

- Clarified that the \<coord-box\> tpe is defined in CSS Box 3.

- Corrected \<ray()\> and \<path()\> type syntax.

- Clarified that "modulo" has its mathematical, not C/JS definition [\#339](https://github.com/w3c/fxtf-drafts/issues/339).

- Fixed directionality at sharp path boundaries to match SVG. [\#209](https://github.com/w3c/fxtf-drafts/issues/209).

- Reorganized offset-path section for better readability.

- Removed note describing the concept of polar angles.

- Changed computed value of offset-distance to a computed value.

- Replaced animatable by Animation type.

### <a id="changes-20150409"></a>Changes since the [9 April 2015](https://www.w3.org/TR/2015/WD-motion-1-20150409/) First Public Working Draft

- <a id="ref-for-propdef-offset-path①⓪"></a>

  Renamed [motion-path](https://www.w3.org/TR/2015/WD-motion-1-20150409/#motion-path-property) to [offset-path](#propdef-offset-path) for integrating with [polar-angle](https://www.w3.org/TR/2016/WD-css-round-display-1-20160301/#polar-angle-property).

  - <a id="ref-for-funcdef-ray⑦"></a>

    <a id="ref-for-offset-path③③"></a>

    <a id="ref-for-angle-value①⑧"></a>

    Added the [ray()](#funcdef-ray) to define an [offset path](#offset-path) as a line segment which direction is specified by [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value).

  - <a id="ref-for-propdef-contain③"></a>

    <a id="ref-for-funcdef-ray⑧"></a>

    Added \<size\> and [contain](https://www.w3.org/TR/css-contain-2/#propdef-contain) value for the [ray()](#funcdef-ray).

- <a id="ref-for-propdef-offset-distance①①"></a>

  Renamed [motion-offset](https://www.w3.org/TR/2015/WD-motion-1-20150409/#propdef-motion-offset) to [offset-distance](#propdef-offset-distance) for integrating with [polar-distance](https://www.w3.org/TR/2016/WD-css-round-display-1-20160301/#polar-distance-property).

- <a id="ref-for-propdef-offset-rotate①⓪"></a>

  Renamed [motion-rotation](https://www.w3.org/TR/2015/WD-motion-1-20150409/#propdef-motion-rotation) to [offset-rotate](#propdef-offset-rotate).

- <a id="ref-for-propdef-offset-position①③"></a>

  <a id="ref-for-offset-starting-position⑧"></a>

  Added [offset-position](#propdef-offset-position) to specify the [offset starting position](#offset-starting-position) of the path by merging [polar-origin](https://www.w3.org/TR/2016/WD-css-round-display-1-20160301/#polar-origin-property) from [\[CSS-ROUND-DISPLAY-1\]](#biblio-css-round-display-1).

- <a id="ref-for-propdef-offset-anchor⑥"></a>

  Added [offset-anchor](#propdef-offset-anchor) to specify the origin point of the element by merging [polar-anchor](https://www.w3.org/TR/2016/WD-css-round-display-1-20160301/#polar-anchor-property) from [\[CSS-ROUND-DISPLAY-1\]](#biblio-css-round-display-1).

- <a id="ref-for-propdef-offset②"></a>

  Renamed the shorthand property [motion](https://www.w3.org/TR/2015/WD-motion-1-20150409/#propdef-motion) to [offset](#propdef-offset).

- <a id="ref-for-propdef-offset-rotate①①"></a>

  <a id="ref-for-valdef-offset-rotate-auto⑦"></a>

  <a id="ref-for-valdef-offset-rotate-reverse③"></a>

  <a id="ref-for-angle-value①⑨"></a>

  Made [offset-rotate](#propdef-offset-rotate) specify the rotation transformation by [auto](#valdef-offset-rotate-auto) or [reverse](#valdef-offset-rotate-reverse) in combination with [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value).

## <a id="acknowledgments"></a>Acknowledgments

Thanks to fantasai, Hyojin Song, and all the rest of the CSS WG members for their reviews, comments, and corrections.

## <a id="w3c-conformance"></a> Conformance

### <a id="w3c-conventions"></a> Document conventions

Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.

All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes. [\[RFC2119\]](#biblio-rfc2119)

Examples in this specification are introduced with the words “for example” or are set apart from the normative text with `class="example"`, like this:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ae2b6bc0"></a>
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

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [anchor point](#offset-anchor-point), in § 2.4
- \<angle\>
  - [value for offset-rotate](#valdef-offset-rotate-angle), in § 2.5
  - [value for ray()](#valdef-ray-angle), in § 2.1.1
- [at \<position\>](#valdef-ray-at-position), in § 2.1.1
- auto
  - [value for offset-anchor](#valdef-offset-anchor-auto), in § 2.4
  - [value for offset-position](#valdef-offset-position-auto), in § 2.3
  - [value for offset-rotate](#valdef-offset-rotate-auto), in § 2.5
- [\<basic-shape\>](#valdef-offset-path-basic-shape), in § 2.1
- [closest-corner](#size-closest-corner), in § 2.1.1
- [closest-side](#size-closest-side), in § 2.1.1
- [contain](#valdef-ray-contain), in § 2.1.1
- [\<coord-box\>](#valdef-offset-path-coord-box), in § 2.1
- equivalent path
  - [dfn for \<basic-shape\>](#basic-shape-equivalent-path), in § 3
  - [dfn for circle()](#circle-equivalent-path), in § 3
  - [dfn for ellipse()](#ellipse-equivalent-path), in § 3
  - [dfn for inset()](#inset-equivalent-path), in § 3
  - [dfn for path()](#path-equivalent-path), in § 3
  - [dfn for polygon()](#polygon-equivalent-path), in § 3
  - [dfn for rect()](#rect-equivalent-path), in § 3
  - [dfn for shape()](#shape-equivalent-path), in § 3
  - [dfn for xywh()](#xywh-equivalent-path), in § 3
- [farthest-corner](#size-farthest-corner), in § 2.1.1
- [farthest-side](#size-farthest-side), in § 2.1.1
- [none](#valdef-offset-path-none), in § 2.1
- [normal](#valdef-offset-position-normal), in § 2.3
- [offset](#propdef-offset), in § 2.6
- [offset-anchor](#propdef-offset-anchor), in § 2.4
- [offset anchor point](#offset-anchor-point), in § 2.4
- [offset distance](#offset-distance), in § 2.2.1
- [offset-distance](#propdef-offset-distance), in § 2.2
- [\<offset-path\>](#typedef-offset-path), in § 2.1
- [offset path](#offset-path), in § 2.1
- [offset-path](#propdef-offset-path), in § 2.1
- [\<offset-path\> \|\| \<coord-box\>](#valdef-offset-path-offset-path--coord-box), in § 2.1
- [offset position](#offset-position), in § 1
- [offset-position](#propdef-offset-position), in § 2.3
- [offset-rotate](#propdef-offset-rotate), in § 2.5
- [offset starting position](#offset-starting-position), in § 2.3
- [offset transform](#offset-transform), in § 1
- [origin](#ray-origin), in § 2.1.1
- \<position\>
  - [value for offset-anchor](#valdef-offset-anchor-position), in § 2.4
  - [value for offset-position](#valdef-offset-position-position), in § 2.3
- [\<ray()\>](#valdef-offset-path-ray), in § 2.1
- [ray()](#funcdef-ray), in § 2.1.1
- \<ray-size\>
  - [(type)](#typedef-ray-size), in § 2.1.1
  - [value for ray()](#valdef-ray-ray-size), in § 2.1.1
- [reverse](#valdef-offset-rotate-reverse), in § 2.5
- [sides](#size-sides), in § 2.1.1
- [total length](#total-length), in § 2.2.1
- [\<url\>](#valdef-offset-path-url), in § 2.1
- [used offset distance](#used-offset-distance), in § 2.2.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="3a4a9318"></a>border-radius
- \[CSS-BOX-4\] defines the following terms:
  - <a id="225e4b50"></a>\<coord-box\>
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="5dfeee7f"></a>contain
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="5c159f8f"></a>box
  - <a id="6b4fc208"></a>containing block
- \[CSS-IMAGES-4\] defines the following terms:
  - <a id="c73efcf9"></a>gradient function
- \[CSS-MASKING-1\] defines the following terms:
  - <a id="26027e88"></a>\<geometry-box\>
- \[CSS-POSITION-3\] defines the following terms:
  - <a id="ebcbc56d"></a>left
  - <a id="b8c34db8"></a>position
  - <a id="35f1d972"></a>static
  - <a id="f99d4ae2"></a>top
- \[CSS-SHAPES\] defines the following terms:
  - <a id="bff39085"></a>\<basic-shape\>
  - <a id="69f26c74"></a>circle()
  - <a id="4ab9aaee"></a>ellipse()
  - <a id="2091c1e6"></a>path()
  - <a id="b68a40b4"></a>polygon()
  - <a id="42b6a839"></a>reference box
- \[CSS-SHAPES-2\] defines the following terms:
  - <a id="064d79bf"></a>shape()
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="e7c6bf78"></a>transform
  - <a id="31452ed5"></a>transform-origin
  - <a id="70f2f959"></a>transformable element
  - <a id="72bd2c25"></a>translate()
- \[CSS-TRANSFORMS-2\] defines the following terms:
  - <a id="c7dad71d"></a>rotate
  - <a id="75f8cbbb"></a>scale
  - <a id="a0605489"></a>translate
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="81b3af3e"></a>!
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="d7e1d67b"></a>\<angle\>
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="699488a8"></a>\<url\>
  - <a id="d4441b24"></a>?
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-VALUES-5\] defines the following terms:
  - <a id="3df4be2e"></a>\<position\>
- \[CSS21\] defines the following terms:
  - <a id="a50c2771"></a>stacking context
- \[MOTION-1\] defines the following terms:
  - <a id="d374477f"></a>path()
- \[SVG2\] defines the following terms:
  - <a id="e531fcd9"></a>direction of a path
  - <a id="c2861a19"></a>equivalent path
  - <a id="b9e84d5d"></a>segment-completing close path
  - <a id="9d3b8726"></a>shape elements

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-box-3"></a>\[CSS-BOX-3\]  
Elika Etemad. [CSS Box Model Module Level 3](https://www.w3.org/TR/css-box-3/). 11 April 2024. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-3&#x2F;](https://www.w3.org/TR/css-box-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 17 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-2&#x2F;](https://www.w3.org/TR/css-contain-2/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 4](https://www.w3.org/TR/css-images-4/). 17 February 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-shapes"></a>\[CSS-SHAPES\]  
Rossen Atanassov; Alan Stearns. [CSS Shapes Module Level 1](https://www.w3.org/TR/css-shapes-1/). 15 November 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-shapes-1&#x2F;](https://www.w3.org/TR/css-shapes-1/)

<a id="biblio-css-shapes-2"></a>\[CSS-SHAPES-2\]  
[CSS Shapes Module Level 2](https://drafts.csswg.org/css-shapes-2/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-shapes-2&#x2F;](https://drafts.csswg.org/css-shapes-2/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-transforms-2"></a>\[CSS-TRANSFORMS-2\]  
Tab Atkins Jr.; et al. [CSS Transforms Module Level 2](https://www.w3.org/TR/css-transforms-2/). 9 November 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-2&#x2F;](https://www.w3.org/TR/css-transforms-2/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-values-5"></a>\[CSS-VALUES-5\]  
Tab Atkins Jr.; Elika Etemad; Miriam Suzanne. [CSS Values and Units Module Level 5](https://www.w3.org/TR/css-values-5/). 17 September 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-5&#x2F;](https://www.w3.org/TR/css-values-5/)

<a id="biblio-css21"></a>\[CSS21\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css3val"></a>\[CSS3VAL\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-svg11"></a>\[SVG11\]  
Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG11&#x2F;](https://www.w3.org/TR/SVG11/)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

### <a id="informative"></a>Informative References

<a id="biblio-css-masking-1"></a>\[CSS-MASKING-1\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 5 August 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-position-3"></a>\[CSS-POSITION-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/css-position-3/). 10 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-position-3&#x2F;](https://www.w3.org/TR/css-position-3/)

<a id="biblio-css-round-display-1"></a>\[CSS-ROUND-DISPLAY-1\]  
Jihye Hong. [CSS Round Display Level 1](https://www.w3.org/TR/css-round-display-1/). 22 December 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-round-display-1&#x2F;](https://www.w3.org/TR/css-round-display-1/)

<a id="biblio-motion-1"></a>\[MOTION-1\]  
Dirk Schulze; et al. [Motion Path Module Level 1](https://www.w3.org/TR/motion-1/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;motion-1&#x2F;](https://www.w3.org/TR/motion-1/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                                                                                                     | Initial                   | Applies to             | Inh.                      | %ages                                                               | Anim­ation type            | Canonical order | Com­puted value                                          | Media  |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------|---------------------------|------------------------|---------------------------|---------------------------------------------------------------------|---------------------------|-----------------|---------------------------------------------------------|--------|
| <strong><span><a id="ref-for-propdef-offset③"></a></span><a href="#propdef-offset">offset</a>&#xA;      </strong> | \[ \<'offset-position'\>? \[ \<'offset-path'\> \[ \<'offset-distance'\> \|\| \<'offset-rotate'\> \]? \]? \]! \[ / \<'offset-anchor'\> \]? | see individual properties | transformable elements | see individual properties | see individual properties                                           | see individual properties | per grammar     | see individual properties                               |        |
| <strong><span><a id="ref-for-propdef-offset-anchor⑦"></a></span><a href="#propdef-offset-anchor">offset-anchor</a>&#xA;      </strong> | auto \| \<position\>                                                                                                                      | auto                      | transformable elements | no                        | relative to the width and the height of the element’s reference box | by computed value         | per grammar     | the auto keyword or a computed \<position\>             | visual |
| <strong><span><a id="ref-for-propdef-offset-distance①②"></a></span><a href="#propdef-offset-distance">offset-distance</a>&#xA;      </strong> | \<length-percentage\>                                                                                                                     | 0                         | transformable elements | no                        | relative to the offset path length                                  | by computed value         | per grammar     | a computed \<length-percentage\> value                  | visual |
| <strong><span><a id="ref-for-propdef-offset-path①①"></a></span><a href="#propdef-offset-path">offset-path</a>&#xA;      </strong> | none \| \<offset-path\> \|\| \<coord-box\>                                                                                                | none                      | transformable elements | no                        | n/a                                                                 | by computed value         | per grammar     | as specified                                            | visual |
| <strong><span><a id="ref-for-propdef-offset-position①④"></a></span><a href="#propdef-offset-position">offset-position</a>&#xA;      </strong> | normal \| auto \| \<position\>                                                                                                            | normal                    | transformable elements | no                        | Refer to the size of containing block                               | by computed value         | per grammar     | The normal or auto keywords, or a computed \<position\> | visual |
| <strong><span><a id="ref-for-propdef-offset-rotate①②"></a></span><a href="#propdef-offset-rotate">offset-rotate</a>&#xA;      </strong> | \[ auto \| reverse \] \|\| \<angle\>                                                                                                      | auto                      | transformable elements | no                        | n/a                                                                 | by computed value         | per grammar     | computed \<angle\> value, optionally preceded by auto   | visual |

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> all of these examples need to be rewritten. [↵](#issue-9aaa1ad0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Which box this is resolved against is being discussed in [Issue 503](https://github.com/w3c/fxtf-drafts/issues/503). [↵](#issue-84d4a7e7)
