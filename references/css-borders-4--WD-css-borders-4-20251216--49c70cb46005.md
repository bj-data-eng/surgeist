Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Borders and Box Decorations Module Level 4](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/).

Original copyright notice: Copyright © 2025 World Wide Web Consortium . W3C ® liability , trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Borders and Box Decorations Module Level 4

Source snapshot: https://www.w3.org/TR/2025/WD-css-borders-4-20251216/

Snapshot SHA-256: 49c70cb460051eadf74aacc76ec41e9e87c2cc909efce80b2274c68185829ac7

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 4 MathML expressions are represented as portable fenced TeX. Independent round-trip checks cover mathematical tokens, matrix shape/order, scripts, fractions and root structure; exact source MathML is retained in verification metadata. Visual equivalence is not certified.
- Source row-header labels in readable Markdown tables are bold; native HTML th/scope accessibility semantics are not expressible in GFM. Field/Definition headings, where used, are added non-normative presentation labels.
- 2 complex or multi-paragraph tables use source-checked readable field, case, grid or matrix layouts. Explicit header/span relationships and source cell mappings are retained; no raw HTML tables or flattened row/cell dumps remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Borders and Box Decorations Module Level 4

[Copyright](https://www.w3.org/policies/#copyright) © 2025 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This module contains the features of CSS relating to the borders and decorations of boxes on the page.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.&#xA;&#x9;A list of current W3C publications&#xA;&#x9;and the latest revision of this technical report&#xA;&#x9;can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-borders” in the title, like this: “\[css-borders\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-borders%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="intro"></a>1.  Introduction

<a id="ref-for-border-area"></a>

The properties of this module deal with the decoration of the [border area](https://www.w3.org/TR/css-box-4/#border-area). It also defines decorations that can be applied to the box.

### <a id="placement"></a>1.1.  Module Interactions

This specification extends the parts related to borders and box decorations of CSS Backgrounds and Borders Module Level 3 [\[CSS3BG\]](#biblio-css3bg).

<a id="ref-for-propdef-border-shape"></a>

<a id="ref-for-propdef-border-limit"></a>

It provides specifications for the added corner-\*-shape and [border-shape](#propdef-border-shape) properties, as well as logical shorthands for border-\*-radius, box-shadow-\* longhands, and partial borders via the [border-limit](#propdef-border-limit) and border-\*-clip properties.

<a id="ref-for-selectordef-first-letter"></a>

<a id="ref-for-pseudo-element"></a>

<a id="ref-for-selectordef-first-line"></a>

<a id="ref-for-propdef-border-image"></a>

<a id="ref-for-propdef-box-shadow"></a>

All properties in this module apply to the [::first-letter](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-letter) [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element). The [border-radius properties](#corners) also apply to the [::first-line](https://www.w3.org/TR/css-pseudo-4/#selectordef-first-line) <a id="ref-for-pseudo-element①"></a>pseudo-element. The UA may (but is not required to) apply the [border-image](#propdef-border-image) or [box-shadow](#propdef-box-shadow) properties to <a id="ref-for-selectordef-first-line①"></a>::first-line. The UA must not apply the [border-color/style/width properties](#borders) to <a id="ref-for-selectordef-first-line②"></a>::first-line. [\[CSS2\]](#biblio-css2)

### <a id="values"></a>1.2.  Value Definitions

<a id="ref-for-propdef-background-image"></a>

<a id="ref-for-propdef-border-image①"></a>

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types. For example, combining with [CSS Images](https://www.w3.org/TR/css-images/) allows for using CSS gradients as [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) or [border-image](#propdef-border-image) values. [\[CSS-IMAGES-3\]](#biblio-css-images-3)

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="borders"></a>2.  Borders

<a id="ref-for-border"></a>

<a id="ref-for-propdef-border-style"></a>

<a id="ref-for-propdef-border-color"></a>

<a id="ref-for-propdef-border-width"></a>

The [border](https://www.w3.org/TR/css-box-4/#border) can either be a predefined style (solid line, double line, dotted line, pseudo-3D border, etc.) or it can be an image. In the former case, various properties define the style ([border-style](#propdef-border-style)), color ([border-color](#propdef-border-color)), and thickness ([border-width](#propdef-border-width)) of the border.

Tests

Tests for features Not yet incorporated from Backgrounds 3

- [border-image-gradient-zero-size-transform-crash.html](https://wpt.fyi/results/css/css-borders/border-image-gradient-zero-size-transform-crash.html) [(live test)](http://wpt.live/css/css-borders/border-image-gradient-zero-size-transform-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/border-image-gradient-zero-size-transform-crash.html)
- [border-image-width-interpolation-math-functions.html](https://wpt.fyi/results/css/css-borders/border-image-width-interpolation-math-functions.html) [(live test)](http://wpt.live/css/css-borders/border-image-width-interpolation-math-functions.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/border-image-width-interpolation-math-functions.html)

------------------------------------------------------------------------

<a id="ref-for-propdef-border-color①"></a>

### <a id="border-color"></a>2.1. <a id="the-border-color"></a> Line Colors: the [border-color](#propdef-border-color) properties



| Field               | Definition                                                                                                                                                                                                                                                                                                                                 |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-top-color"></a>border-top-color, <a id="propdef-border-right-color"></a>border-right-color, <a id="propdef-border-bottom-color"></a>border-bottom-color, <a id="propdef-border-left-color"></a>border-left-color, <a id="propdef-border-block-start-color"></a>border-block-start-color, <a id="propdef-border-block-end-color"></a>border-block-end-color, <a id="propdef-border-inline-start-color"></a>border-inline-start-color, <a id="propdef-border-inline-end-color"></a>border-inline-end-color |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-image-1d"></a><a id="ref-for-comb-one"></a><a id="ref-for-typedef-color"></a>[\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<image-1D\>](https://www.w3.org/TR/css-images-4/#typedef-image-1d)                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | currentcolor                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-ruby-annotation-container-box"></a><a id="ref-for-ruby-base-container-box"></a>all elements except [ruby base containers](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and [ruby annotation containers](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-container-box)                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | the computed color and/or a one-dimensional image function                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see prose                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-color②"></a>[border-color](#propdef-border-color)                                                                                                                                                                                                                                                                                   |



<a id="ref-for-border①"></a>

<a id="ref-for-propdef-border-style①"></a>

These properties set the foreground <a id="border-color-dfn"></a>color of the [border](https://www.w3.org/TR/css-box-4/#border) specified by the [border-style](#propdef-border-style) properties.

<a id="ref-for-typedef-image-1d①"></a>

<a id="ref-for-padding-edge"></a>

The stripes defined by [\<image-1D\>](https://www.w3.org/TR/css-images-4/#typedef-image-1d) follow the shape of the border on the side to which they apply, and are drawn in bands starting from the [padding edge](https://www.w3.org/TR/css-box-4/#padding-edge) and progressing outwards. The border width at each point defines the <var>total width</var> of the stripes at that point.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7b0dea52"></a> Using multiple colors for each side:
>
> ```css
> .foo {
>   border: 30px solid;
>   border-color: stripes(dodgerblue, skyblue) stripes(yellow, gold) stripes(lightgreen, limegreen) stripes(indianred, orange);
> }
> ```
>
> Sample rendering:
>
> ![A box with a border that has four sides, each side having two colors in stripes. The top border is dodger blue and sky blue, the right border is yellow and gold, the bottom border is light green and lime green, and the left border is indian red and orange.](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/multicolor-border.png)
>
> <a id="ref-for-propdef-border-style②"></a>
>
> The same border colors with [border-style: dotted](#propdef-border-style):
>
> ![A box with a border that has four sides, each side having two colors in stripes. The top border is dodger blue and sky blue, the right border is yellow and gold, the bottom border is light green and lime green, and the left border is indian red and orange. The border style is dotted, so the colors appear in dots along the border.](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/multicolor-border-dotted.png)



| Field               | Definition                                                                                                                                                                                                                                                                                                                         |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-color"></a>border-color                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range"></a><a id="ref-for-typedef-image-1d②"></a><a id="ref-for-comb-one①"></a><a id="ref-for-typedef-color①"></a>\[ [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<image-1D\>](https://www.w3.org/TR/css-images-4/#typedef-image-1d) \][{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                        |



<a id="ref-for-propdef-border-color③"></a>

<a id="ref-for-shorthand-property"></a>

[border-color](#propdef-border-color) is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for the four physical border-\*-color properties. The four values set the top, right, bottom and left border, respectively. A missing left is the same as right, a missing bottom is the same as top, and a missing right is also the same as top. This is resolved individually for each list item.

<a id="ref-for-flow-relative"></a>

<a id="ref-for-propdef-border-block-start-color"></a>

<a id="ref-for-propdef-border-block-end-color"></a>

<a id="ref-for-propdef-border-inline-start-color"></a>

<a id="ref-for-propdef-border-inline-end-color"></a>

<a id="ref-for-physical"></a>

<a id="ref-for-propdef-border-top-color"></a>

<a id="ref-for-propdef-border-bottom-color"></a>

<a id="ref-for-propdef-border-left-color"></a>

<a id="ref-for-propdef-border-right-color"></a>

<a id="ref-for-propdef-writing-mode"></a>

<a id="ref-for-propdef-direction"></a>

<a id="ref-for-propdef-text-orientation"></a>

The [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) properties [border-block-start-color](#propdef-border-block-start-color), [border-block-end-color](#propdef-border-block-end-color), [border-inline-start-color](#propdef-border-inline-start-color), and [border-inline-end-color](#propdef-border-inline-end-color) correspond to the [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) properties [border-top-color](#propdef-border-top-color), [border-bottom-color](#propdef-border-bottom-color), [border-left-color](#propdef-border-left-color), and [border-right-color](#propdef-border-right-color). The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation).



| Field               | Definition                                                                                                                                           |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-block-color"></a>border-block-color, <a id="propdef-border-inline-color"></a>border-inline-color                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range①"></a><a id="ref-for-propdef-border-top-color①"></a>[\<'border-top-color'\>](#propdef-border-top-color)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                          |



<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-propdef-border-block-start-color①"></a>

<a id="ref-for-propdef-border-block-end-color①"></a>

<a id="ref-for-propdef-border-inline-start-color①"></a>

<a id="ref-for-propdef-border-inline-end-color①"></a>

<a id="ref-for-start"></a>

<a id="ref-for-end"></a>

These two [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [border-block-start-color](#propdef-border-block-start-color) &#x26; [border-block-end-color](#propdef-border-block-end-color) and [border-inline-start-color](#propdef-border-inline-start-color) &#x26; [border-inline-end-color](#propdef-border-inline-end-color), respectively. The first value represents the [start](https://www.w3.org/TR/css-writing-modes-4/#start) side color, and the second value represents the [end](https://www.w3.org/TR/css-writing-modes-4/#end) side color. If only one value is given, it applies to both the <a id="ref-for-start①"></a>start and <a id="ref-for-end①"></a>end sides.

Tests

- [border-color-interpolation.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-color-interpolation.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-color-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-color-interpolation.html)
- [border-image-displayed-with-transparent-border-color.html](https://wpt.fyi/results/css/css-backgrounds/border-image-displayed-with-transparent-border-color.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-displayed-with-transparent-border-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-displayed-with-transparent-border-color.html)
- [color-mix-currentcolor-border-repaint-parent.html](https://wpt.fyi/results/css/css-backgrounds/color-mix-currentcolor-border-repaint-parent.html) [(live test)](http://wpt.live/css/css-backgrounds/color-mix-currentcolor-border-repaint-parent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/color-mix-currentcolor-border-repaint-parent.html)
- [color-mix-currentcolor-border-repaint.html](https://wpt.fyi/results/css/css-backgrounds/color-mix-currentcolor-border-repaint.html) [(live test)](http://wpt.live/css/css-backgrounds/color-mix-currentcolor-border-repaint.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/color-mix-currentcolor-border-repaint.html)
- [currentcolor-border-repaint-parent.html](https://wpt.fyi/results/css/css-backgrounds/currentcolor-border-repaint-parent.html) [(live test)](http://wpt.live/css/css-backgrounds/currentcolor-border-repaint-parent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/currentcolor-border-repaint-parent.html)
- [inheritance.sub.html](https://wpt.fyi/results/css/css-backgrounds/inheritance.sub.html) [(live test)](http://wpt.live/css/css-backgrounds/inheritance.sub.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/inheritance.sub.html)
- [border-color-computed.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-color-computed.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-color-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-color-computed.html)
- [border-color-invalid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-color-invalid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-color-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-color-invalid.html)
- [border-color-shorthand.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-color-shorthand.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-color-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-color-shorthand.html)
- [border-color-valid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-color-valid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-color-valid.html)

<a id="ref-for-propdef-border-style③"></a>

### <a id="border-style"></a>2.2. <a id="the-border-style"></a> Line Patterns: the [border-style](#propdef-border-style) properties



| Field               | Definition                                                                                                                                                                                                                                                                                                                                 |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-top-style"></a>border-top-style, <a id="propdef-border-right-style"></a>border-right-style, <a id="propdef-border-bottom-style"></a>border-bottom-style, <a id="propdef-border-left-style"></a>border-left-style, <a id="propdef-border-block-start-style"></a>border-block-start-style, <a id="propdef-border-block-end-style"></a>border-block-end-style, <a id="propdef-border-inline-start-style"></a>border-inline-start-style, <a id="propdef-border-inline-end-style"></a>border-inline-end-style |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-line-style"></a>[\<line-style\>](#typedef-line-style)                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-ruby-annotation-container-box①"></a><a id="ref-for-ruby-base-container-box①"></a>all elements except [ruby base containers](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and [ruby annotation containers](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-container-box)                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-style④"></a>[border-style](#propdef-border-style)                                                                                                                                                                                                                                                                                   |



<a id="ref-for-border②"></a>

These properties control whether a [border](https://www.w3.org/TR/css-box-4/#border) appears, and if it does what <a id="border-style-dfn"></a>style it’s drawn in (if it is not overridden by a [border image](#border-images)).

<a id="ref-for-flow-relative①"></a>

<a id="ref-for-propdef-border-block-start-style"></a>

<a id="ref-for-propdef-border-block-end-style"></a>

<a id="ref-for-propdef-border-inline-start-style"></a>

<a id="ref-for-propdef-border-inline-end-style"></a>

<a id="ref-for-physical①"></a>

<a id="ref-for-propdef-border-top-style"></a>

<a id="ref-for-propdef-border-bottom-style"></a>

<a id="ref-for-propdef-border-left-style"></a>

<a id="ref-for-propdef-border-right-style"></a>

<a id="ref-for-propdef-writing-mode①"></a>

<a id="ref-for-propdef-direction①"></a>

<a id="ref-for-propdef-text-orientation①"></a>

The [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) properties [border-block-start-style](#propdef-border-block-start-style), [border-block-end-style](#propdef-border-block-end-style), [border-inline-start-style](#propdef-border-inline-start-style), and [border-inline-end-style](#propdef-border-inline-end-style) correspond to the [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) properties [border-top-style](#propdef-border-top-style), [border-bottom-style](#propdef-border-bottom-style), [border-left-style](#propdef-border-left-style), and [border-right-style](#propdef-border-right-style). The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation).



| Field               | Definition                                                                                                                                           |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-block-style"></a>border-block-style, <a id="propdef-border-inline-style"></a>border-inline-style                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range②"></a><a id="ref-for-propdef-border-top-style①"></a>[\<'border-top-style'\>](#propdef-border-top-style)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                          |



<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-propdef-border-block-start-style①"></a>

<a id="ref-for-propdef-border-block-end-style①"></a>

<a id="ref-for-propdef-border-inline-start-style①"></a>

<a id="ref-for-propdef-border-inline-end-style①"></a>

<a id="ref-for-start②"></a>

<a id="ref-for-end②"></a>

These two [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [border-block-start-style](#propdef-border-block-start-style) &#x26; [border-block-end-style](#propdef-border-block-end-style) and [border-inline-start-style](#propdef-border-inline-start-style) &#x26; [border-inline-end-style](#propdef-border-inline-end-style), respectively. The first value represents the [start](https://www.w3.org/TR/css-writing-modes-4/#start) side style, and the second value represents the [end](https://www.w3.org/TR/css-writing-modes-4/#end) side style. If only one value is given, it applies to both the <a id="ref-for-start③"></a>start and <a id="ref-for-end③"></a>end sides.



| Field               | Definition                                                                                                                                           |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-style"></a>border-style                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range③"></a><a id="ref-for-propdef-border-top-style②"></a>[\<'border-top-style'\>](#propdef-border-top-style)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                          |



<a id="ref-for-propdef-border-style⑤"></a>

<a id="ref-for-shorthand-property③"></a>

[border-style](#propdef-border-style) is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for the four physical border-\*-style properties. The four values set the top, right, bottom and left border, respectively. A missing left is the same as right, a missing bottom is the same as top, and a missing right is also the same as top. This is resolved individually for each list item.

<a id="ref-for-typedef-line-style①"></a>

The style is specified as a [\<line-style\>](#typedef-line-style) keyword, where

<a id="typedef-line-style"></a>

<a id="ref-for-typedef-line-style②"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-comb-one①⓪"></a>

```text
<line-style> = none | hidden | dotted | dashed | solid | double | groove | ridge | inset | outset
```
Values have the following meanings:

<a id="valdef-line-style-none"></a>none  
<a id="ref-for-propdef-border-image-width"></a>

No border. Color and width are ignored (i.e., the border has width 0). Note this means that the initial value of [border-image-width](#propdef-border-image-width) will also resolve to zero.

<a id="valdef-line-style-hidden"></a>hidden  
<a id="ref-for-valdef-line-style-none"></a>

Same as [none](#valdef-line-style-none), but has different behavior in the border conflict resolution rules for border-collapsed tables [\[CSS2\]](#biblio-css2).

<a id="valdef-line-style-dotted"></a>dotted  
A series of round dots.

<a id="valdef-line-style-dashed"></a>dashed  
A series of square-ended dashes.

<a id="valdef-line-style-solid"></a>solid  
A single line segment.

<a id="valdef-line-style-double"></a>double  
<a id="ref-for-propdef-border-width①"></a>

Two parallel solid lines with some space between them. (The thickness of the lines is not specified, but the sum of the lines and the space must equal [border-width](#propdef-border-width).)

<a id="valdef-line-style-groove"></a>groove  
<a id="ref-for-propdef-border-color④"></a>

Looks as if it were carved in the canvas. (This is typically achieved by creating a “shadow” from two colors that are slightly lighter and darker than the specified [border-color](#propdef-border-color).)

<a id="valdef-line-style-ridge"></a>ridge  
Looks as if it were coming out of the canvas.

<a id="valdef-line-style-inset"></a>inset  
<a id="ref-for-valdef-line-style-ridge"></a>

Looks as if the content on the inside of the border is sunken into the canvas. Treated as [ridge](#valdef-line-style-ridge) in the [collapsing border model](https://www.w3.org/TR/CSS2/tables.html#collapsing-borders). [\[CSS2\]](#biblio-css2)

<a id="valdef-line-style-outset"></a>outset  
<a id="ref-for-valdef-line-style-groove"></a>

Looks as if the content on the inside of the border is raised out of the canvas. Treated as [groove](#valdef-line-style-groove) in the [collapsing border model](https://www.w3.org/TR/CSS2/tables.html#collapsing-borders). [\[CSS2\]](#biblio-css2)

Borders are drawn in front of the element’s background, but behind the element’s content (in case it overlaps).

![](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/borderstyles.png)

Example renderings of the predefined border styles.

<a id="ref-for-valdef-line-style-groove①"></a>

<a id="ref-for-valdef-line-style-ridge①"></a>

<a id="ref-for-valdef-line-style-inset"></a>

<a id="ref-for-valdef-line-style-outset"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Border colors close to black or white may need different color calculations than colors in between in order to create the required “3D” effect of [groove](#valdef-line-style-groove), [ridge](#valdef-line-style-ridge), [inset](#valdef-line-style-inset), or [outset](#valdef-line-style-outset).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: There is no control over the spacing of the dots and dashes, nor over the length of the dashes. Implementations are encouraged to choose a spacing that makes the corners symmetrical.

Tests

- [inheritance.sub.html](https://wpt.fyi/results/css/css-backgrounds/inheritance.sub.html) [(live test)](http://wpt.live/css/css-backgrounds/inheritance.sub.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/inheritance.sub.html)
- [border-style-computed.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-style-computed.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-style-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-style-computed.html)
- [border-style-invalid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-style-invalid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-style-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-style-invalid.html)
- [border-style-shorthand.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-style-shorthand.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-style-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-style-shorthand.html)
- [border-style-valid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-style-valid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-style-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-style-valid.html)

<a id="ref-for-propdef-border-width②"></a>

### <a id="border-width"></a>2.3. <a id="the-border-width"></a> Line Thickness: the [border-width](#propdef-border-width) properties



| Field               | Definition                                                                                                                                                                                                                                                                                                                                 |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-top-width"></a>border-top-width, <a id="propdef-border-right-width"></a>border-right-width, <a id="propdef-border-bottom-width"></a>border-bottom-width, <a id="propdef-border-left-width"></a>border-left-width, <a id="propdef-border-block-start-width"></a>border-block-start-width, <a id="propdef-border-block-end-width"></a>border-block-end-width, <a id="propdef-border-inline-start-width"></a>border-inline-start-width, <a id="propdef-border-inline-end-width"></a>border-inline-end-width |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-line-width"></a>[\<line-width\>](#typedef-line-width)                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | medium                                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-ruby-annotation-container-box②"></a><a id="ref-for-ruby-base-container-box②"></a>all elements except [ruby base containers](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and [ruby annotation containers](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-container-box)                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-line-style-hidden"></a><a id="ref-for-valdef-line-style-none①"></a><a id="ref-for-snap-a-length-as-a-border-width"></a>absolute length, [snapped as a border width](https://www.w3.org/TR/css-values-4/#snap-a-length-as-a-border-width); zero if the border style is [none](#valdef-line-style-none) or [hidden](#valdef-line-style-hidden)                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-width③"></a>[border-width](#propdef-border-width)                                                                                                                                                                                                                                                                                   |



<a id="ref-for-border③"></a>

These properties specify the thickness of the [border](https://www.w3.org/TR/css-box-4/#border), i.e. the <a id="border-width-dfn"></a>border width. Where

<a id="typedef-line-width"></a>

<a id="ref-for-typedef-line-width①"></a>

<a id="ref-for-length-value"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-comb-one①③"></a>

```text
<line-width> = <length [0,∞]> | thin | medium | thick
```
Negative values are invalid. The <a id="valdef-line-width-thin"></a>thin, <a id="valdef-line-width-medium"></a>medium, and <a id="valdef-line-width-thick"></a>thick keywords are equivalent to 1px, 3px, and 5px, respectively.

<a id="ref-for-flow-relative②"></a>

<a id="ref-for-propdef-border-block-start-width"></a>

<a id="ref-for-propdef-border-block-end-width"></a>

<a id="ref-for-propdef-border-inline-start-width"></a>

<a id="ref-for-propdef-border-inline-end-width"></a>

<a id="ref-for-physical②"></a>

<a id="ref-for-propdef-border-top-width"></a>

<a id="ref-for-propdef-border-bottom-width"></a>

<a id="ref-for-propdef-border-left-width"></a>

<a id="ref-for-propdef-border-right-width"></a>

<a id="ref-for-propdef-writing-mode②"></a>

<a id="ref-for-propdef-direction②"></a>

<a id="ref-for-propdef-text-orientation②"></a>

The [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) properties [border-block-start-width](#propdef-border-block-start-width), [border-block-end-width](#propdef-border-block-end-width), [border-inline-start-width](#propdef-border-inline-start-width), and [border-inline-end-width](#propdef-border-inline-end-width) correspond to the [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) properties [border-top-width](#propdef-border-top-width), [border-bottom-width](#propdef-border-bottom-width), [border-left-width](#propdef-border-left-width), and [border-right-width](#propdef-border-right-width). The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation).



| Field               | Definition                                                                                                                                           |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-block-width"></a>border-block-width, <a id="propdef-border-inline-width"></a>border-inline-width                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range④"></a><a id="ref-for-propdef-border-top-width①"></a>[\<'border-top-width'\>](#propdef-border-top-width)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                          |



<a id="ref-for-shorthand-property④"></a>

<a id="ref-for-propdef-border-block-start-width①"></a>

<a id="ref-for-propdef-border-block-end-width①"></a>

<a id="ref-for-propdef-border-inline-start-width①"></a>

<a id="ref-for-propdef-border-inline-end-width①"></a>

<a id="ref-for-start④"></a>

<a id="ref-for-end④"></a>

These two [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [border-block-start-width](#propdef-border-block-start-width) &#x26; [border-block-end-width](#propdef-border-block-end-width) and [border-inline-start-width](#propdef-border-inline-start-width) &#x26; [border-inline-end-width](#propdef-border-inline-end-width), respectively. The first value represents the [start](https://www.w3.org/TR/css-writing-modes-4/#start) side width, and the second value represents the [end](https://www.w3.org/TR/css-writing-modes-4/#end) side width. If only one value is given, it applies to both the <a id="ref-for-start⑤"></a>start and <a id="ref-for-end⑤"></a>end sides.



| Field               | Definition                                                                                                                                           |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-width"></a>border-width                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range⑤"></a><a id="ref-for-propdef-border-top-width②"></a>[\<'border-top-width'\>](#propdef-border-top-width)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                          |



<a id="ref-for-propdef-border-width④"></a>

<a id="ref-for-shorthand-property⑤"></a>

[border-width](#propdef-border-width) is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for the four physical border-\*-width properties. The four values set the top, right, bottom and left border, respectively. A missing left is the same as right, a missing bottom is the same as top, and a missing right is also the same as top. This is resolved individually for each list item.

<a id="ref-for-initial-value"></a>

<a id="ref-for-valdef-line-width-medium"></a>

<a id="ref-for-valdef-line-style-none②"></a>

<a id="ref-for-used-value"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although the [initial](https://www.w3.org/TR/css-cascade-5/#initial-value) width is [medium](#valdef-line-width-medium), the <a id="ref-for-initial-value①"></a>initial style is [none](#valdef-line-style-none); therefore the [used](https://www.w3.org/TR/css-cascade-5/#used-value) initial width is 0.

Tests

- [border-bottom-width-composition.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-bottom-width-composition.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-bottom-width-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-bottom-width-composition.html)
- [border-image-width-composition.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-image-width-composition.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-image-width-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-image-width-composition.html)
- [border-image-width-interpolation.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-image-width-interpolation.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-image-width-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-image-width-interpolation.html)
- [border-left-width-composition.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-left-width-composition.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-left-width-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-left-width-composition.html)
- [border-right-width-composition.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-right-width-composition.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-right-width-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-right-width-composition.html)
- [border-top-width-composition.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-top-width-composition.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-top-width-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-top-width-composition.html)
- [border-width-interpolation.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-width-interpolation.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-width-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-width-interpolation.html)
- [border-bottom-width-medium.html](https://wpt.fyi/results/css/css-backgrounds/border-bottom-width-medium.html) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-width-medium.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-width-medium.html)
- [border-bottom-width-thick.html](https://wpt.fyi/results/css/css-backgrounds/border-bottom-width-thick.html) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-width-thick.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-width-thick.html)
- [border-bottom-width-thin.html](https://wpt.fyi/results/css/css-backgrounds/border-bottom-width-thin.html) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-width-thin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-width-thin.html)
- [border-left-width-medium.html](https://wpt.fyi/results/css/css-backgrounds/border-left-width-medium.html) [(live test)](http://wpt.live/css/css-backgrounds/border-left-width-medium.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-left-width-medium.html)
- [border-left-width-thick.html](https://wpt.fyi/results/css/css-backgrounds/border-left-width-thick.html) [(live test)](http://wpt.live/css/css-backgrounds/border-left-width-thick.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-left-width-thick.html)
- [border-left-width-thin.html](https://wpt.fyi/results/css/css-backgrounds/border-left-width-thin.html) [(live test)](http://wpt.live/css/css-backgrounds/border-left-width-thin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-left-width-thin.html)
- [border-right-width-medium.html](https://wpt.fyi/results/css/css-backgrounds/border-right-width-medium.html) [(live test)](http://wpt.live/css/css-backgrounds/border-right-width-medium.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-right-width-medium.html)
- [border-right-width-thick.html](https://wpt.fyi/results/css/css-backgrounds/border-right-width-thick.html) [(live test)](http://wpt.live/css/css-backgrounds/border-right-width-thick.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-right-width-thick.html)
- [border-right-width-thin.html](https://wpt.fyi/results/css/css-backgrounds/border-right-width-thin.html) [(live test)](http://wpt.live/css/css-backgrounds/border-right-width-thin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-right-width-thin.html)
- [border-top-width-medium.html](https://wpt.fyi/results/css/css-backgrounds/border-top-width-medium.html) [(live test)](http://wpt.live/css/css-backgrounds/border-top-width-medium.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-width-medium.html)
- [border-top-width-thick.html](https://wpt.fyi/results/css/css-backgrounds/border-top-width-thick.html) [(live test)](http://wpt.live/css/css-backgrounds/border-top-width-thick.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-width-thick.html)
- [border-top-width-thin.html](https://wpt.fyi/results/css/css-backgrounds/border-top-width-thin.html) [(live test)](http://wpt.live/css/css-backgrounds/border-top-width-thin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-width-thin.html)
- [border-width-cssom.html](https://wpt.fyi/results/css/css-backgrounds/border-width-cssom.html) [(live test)](http://wpt.live/css/css-backgrounds/border-width-cssom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-width-cssom.html)
- [border-width-pixel-snapping-001-a.html](https://wpt.fyi/results/css/css-backgrounds/border-width-pixel-snapping-001-a.html) [(live test)](http://wpt.live/css/css-backgrounds/border-width-pixel-snapping-001-a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-width-pixel-snapping-001-a.html)
- [border-width-pixel-snapping-001-b.html](https://wpt.fyi/results/css/css-backgrounds/border-width-pixel-snapping-001-b.html) [(live test)](http://wpt.live/css/css-backgrounds/border-width-pixel-snapping-001-b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-width-pixel-snapping-001-b.html)
- [border-width-small-values-001-a.html](https://wpt.fyi/results/css/css-backgrounds/border-width-small-values-001-a.html) [(live test)](http://wpt.live/css/css-backgrounds/border-width-small-values-001-a.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-width-small-values-001-a.html)
- [border-width-small-values-001-b.html](https://wpt.fyi/results/css/css-backgrounds/border-width-small-values-001-b.html) [(live test)](http://wpt.live/css/css-backgrounds/border-width-small-values-001-b.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-width-small-values-001-b.html)
- [border-width-small-values-001-c.html](https://wpt.fyi/results/css/css-backgrounds/border-width-small-values-001-c.html) [(live test)](http://wpt.live/css/css-backgrounds/border-width-small-values-001-c.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-width-small-values-001-c.html)
- [border-width-small-values-001-d.html](https://wpt.fyi/results/css/css-backgrounds/border-width-small-values-001-d.html) [(live test)](http://wpt.live/css/css-backgrounds/border-width-small-values-001-d.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-width-small-values-001-d.html)
- [border-width-small-values-001-e.html](https://wpt.fyi/results/css/css-backgrounds/border-width-small-values-001-e.html) [(live test)](http://wpt.live/css/css-backgrounds/border-width-small-values-001-e.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-width-small-values-001-e.html)
- [inheritance.sub.html](https://wpt.fyi/results/css/css-backgrounds/inheritance.sub.html) [(live test)](http://wpt.live/css/css-backgrounds/inheritance.sub.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/inheritance.sub.html)
- [border-width-computed.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-width-computed.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-width-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-width-computed.html)
- [border-width-invalid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-width-invalid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-width-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-width-invalid.html)
- [border-width-shorthand.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-width-shorthand.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-width-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-width-shorthand.html)
- [border-width-valid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-width-valid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-width-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-width-valid.html)

<!-- -->

- [borders-on-sub-unit-sized-elements.html](https://wpt.fyi/results/css/css-borders/borders-on-sub-unit-sized-elements.html) [(live test)](http://wpt.live/css/css-borders/borders-on-sub-unit-sized-elements.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/borders-on-sub-unit-sized-elements.html)
- [subpixel-borders-with-child-border-box-sizing.html](https://wpt.fyi/results/css/css-borders/subpixel-borders-with-child-border-box-sizing.html) [(live test)](http://wpt.live/css/css-borders/subpixel-borders-with-child-border-box-sizing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/subpixel-borders-with-child-border-box-sizing.html)
- [subpixel-borders-with-child.html](https://wpt.fyi/results/css/css-borders/subpixel-borders-with-child.html) [(live test)](http://wpt.live/css/css-borders/subpixel-borders-with-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/subpixel-borders-with-child.html)

### <a id="border-shorthands"></a>2.4. <a id="the-border-shorthands"></a> Border Shorthand Properties



| Field               | Definition                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-top"></a>border-top, <a id="propdef-border-right"></a>border-right, <a id="propdef-border-bottom"></a>border-bottom, <a id="propdef-border-left"></a>border-left, <a id="propdef-border-block-start"></a>border-block-start, <a id="propdef-border-block-end"></a>border-block-end, <a id="propdef-border-inline-start"></a>border-inline-start, <a id="propdef-border-inline-end"></a>border-inline-end         |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-color②"></a><a id="ref-for-typedef-line-style③"></a><a id="ref-for-comb-any"></a><a id="ref-for-typedef-line-width②"></a>[\<line-width\>](#typedef-line-width) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<line-style\>](#typedef-line-style) <a id="ref-for-comb-any①"></a>\|\| [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-ruby-annotation-container-box③"></a><a id="ref-for-ruby-base-container-box③"></a>all elements except [ruby base containers](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and [ruby annotation containers](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-container-box)                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                        |



<a id="ref-for-shorthand-property⑥"></a>

<a id="ref-for-propdef-border-width⑤"></a>

<a id="ref-for-propdef-border-color⑤"></a>

<a id="ref-for-propdef-border-style⑥"></a>

<a id="ref-for-border④"></a>

<a id="ref-for-initial-value②"></a>

These [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [border-width](#propdef-border-width), [border-color](#propdef-border-color), and [border-style](#propdef-border-style) of one side of the [borders](https://www.w3.org/TR/css-box-4/#border) of a box. Omitted values are set to their [initial values](https://www.w3.org/TR/css-cascade-5/#initial-value).

<a id="ref-for-flow-relative③"></a>

<a id="ref-for-propdef-border-block-start"></a>

<a id="ref-for-propdef-border-block-end"></a>

<a id="ref-for-propdef-border-inline-start"></a>

<a id="ref-for-propdef-border-inline-end"></a>

<a id="ref-for-physical③"></a>

<a id="ref-for-propdef-border-top"></a>

<a id="ref-for-propdef-border-bottom"></a>

<a id="ref-for-propdef-border-left"></a>

<a id="ref-for-propdef-border-right"></a>

<a id="ref-for-propdef-writing-mode③"></a>

<a id="ref-for-propdef-direction③"></a>

<a id="ref-for-propdef-text-orientation③"></a>

The [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) properties [border-block-start](#propdef-border-block-start), [border-block-end](#propdef-border-block-end), [border-inline-start](#propdef-border-inline-start), and [border-inline-end](#propdef-border-inline-end) correspond to the [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) properties [border-top](#propdef-border-top), [border-bottom](#propdef-border-bottom), [border-left](#propdef-border-left), and [border-right](#propdef-border-right). The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation).



| Field               | Definition                                                                 |
|---------------------|----------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-block"></a>border-block, <a id="propdef-border-inline"></a>border-inline          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-block-start①"></a>[\<'border-block-start'\>](#propdef-border-block-start) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                |



<a id="ref-for-shorthand-property⑦"></a>

<a id="ref-for-propdef-border-block-start②"></a>

<a id="ref-for-propdef-border-block-end①"></a>

<a id="ref-for-propdef-border-inline-start①"></a>

<a id="ref-for-propdef-border-inline-end①"></a>

These two [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [border-block-start](#propdef-border-block-start) &#x26; [border-block-end](#propdef-border-block-end) or [border-inline-start](#propdef-border-inline-start) &#x26; [border-inline-end](#propdef-border-inline-end), respectively, both to the same style.



| Field               | Definition                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border"></a>border                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-color③"></a><a id="ref-for-typedef-line-style④"></a><a id="ref-for-comb-any②"></a><a id="ref-for-typedef-line-width③"></a>[\<line-width\>](#typedef-line-width) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<line-style\>](#typedef-line-style) <a id="ref-for-comb-any③"></a>\|\| [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                        |



Tests

- [border-invalid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-invalid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-invalid.html)
- [border-shorthand.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-shorthand.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-shorthand.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-shorthand.html)
- [border-valid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-valid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-valid.html)

<a id="ref-for-propdef-border"></a>

<a id="ref-for-shorthand-property⑧"></a>

<a id="ref-for-propdef-border-width⑥"></a>

<a id="ref-for-propdef-border-color⑥"></a>

<a id="ref-for-propdef-border-style⑦"></a>

<a id="ref-for-propdef-margin"></a>

<a id="ref-for-propdef-padding"></a>

The [border](#propdef-border) property is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for setting the same [border-width](#propdef-border-width), [border-color](#propdef-border-color), and [border-style](#propdef-border-style) for all four borders of a box. Unlike the shorthand [margin](https://www.w3.org/TR/css-box-4/#propdef-margin) and [padding](https://www.w3.org/TR/css-box-4/#propdef-padding) properties, the <a id="ref-for-propdef-border①"></a>border property cannot set different values on the four borders. To do so, one or more of the other border properties must be used.

<a id="ref-for-propdef-border②"></a>

<a id="ref-for-propdef-border-image②"></a>

The [border](#propdef-border) shorthand also resets [border-image](#propdef-border-image) to its initial value. It is therefore recommended that authors use the <a id="ref-for-propdef-border③"></a>border shorthand, rather than other shorthands or the individual properties, to override any border settings earlier in the cascade. This will ensure that <a id="ref-for-propdef-border-image③"></a>border-image has also been reset to allow the new styles to take effect.

<a id="ref-for-propdef-border④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The CSS Working Group intends for the [border](#propdef-border) shorthand to reset all border properties in future levels of CSS as well. For example, if a border-characters property is introduced in the future to allow glyphs as borders, it will also be reset by the <a id="ref-for-propdef-border⑤"></a>border shorthand. By using the <a id="ref-for-propdef-border⑥"></a>border shorthand to reset borders, authors can be guaranteed a “blank canvas” no matter what properties are introduced in the future.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e8ed94e0"></a> For example, the first rule below is equivalent to the set of five rules shown after it:
>
> ```text
> p { border: solid red }
> p {
>   border-top: solid red;
>   border-right: solid red;
>   border-bottom: solid red;
>   border-left: solid red;
>   border-image: none;
> }
> ```
Since, to some extent, the properties have overlapping functionality, the order in which the rules are specified is important.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-10fca894"></a> Consider this example:
>
> ```text
> blockquote {
>   border-color: red;
>   border-left: double;
>   color: black
> }
> ```
>
> <a id="ref-for-propdef-border-left①"></a>
>
> <a id="ref-for-propdef-color"></a>
>
> In the above example, the color of the left border is black, while the other borders are red. This is due to [border-left](#propdef-border-left) setting the width, style, and color. Since the color value is not given by the <a id="ref-for-propdef-border-left②"></a>border-left property, it will be taken from the [color](https://www.w3.org/TR/css-color-4/#propdef-color) property. The fact that the <a id="ref-for-propdef-color①"></a>color property is set after the <a id="ref-for-propdef-border-left③"></a>border-left property is not relevant.

## <a id="corners"></a>3.  Corners

<a id="ref-for-padding-edge①"></a>

<a id="ref-for-content-edge"></a>

<a id="ref-for-padding"></a>

The [padding edge](https://www.w3.org/TR/css-box-4/#padding-edge) (inner border) radius is the outer border radius minus the corresponding border thickness. In the case where this results in a negative value, the inner radius is zero. (In such cases the center of the border’s inner curve might not coincide with that of its outer curve.) Likewise the [content edge](https://www.w3.org/TR/css-box-4/#content-edge) radius is the <a id="ref-for-padding-edge②"></a>padding edge radius minus the corresponding [padding](https://www.w3.org/TR/css-box-4/#padding), or if that is negative, zero. The border and padding thicknesses in the curved region are thus interpolated from the adjoining sides, and when two adjoining borders are of different thicknesses the corner will show a smooth transition between the thicker and thinner borders.

<a id="ref-for-valdef-line-style-solid"></a>

<a id="ref-for-valdef-line-style-dotted"></a>

<a id="ref-for-valdef-line-style-inset①"></a>

All border styles ([solid](#valdef-line-style-solid), [dotted](#valdef-line-style-dotted), [inset](#valdef-line-style-inset), etc.) follow the curve of the border.

![](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/smooth-radius.png)

The effect of a rounded corner when the two borders it connects are of unequal thickness (left) and the effect of a rounded corner on borders that are thicker than the radius of the corner (right).

<a id="ref-for-padding-edge③"></a>

<a id="ref-for-border-area①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If the center of a corner’s outer curve is past an opposite [padding edge](https://www.w3.org/TR/css-box-4/#padding-edge) (in the [border area](https://www.w3.org/TR/css-box-4/#border-area) of a side opposite the corner), the inner curve will not be a full quarter ellipse.

**Table 13**

**CSS**

```text
p { width: 70px; height: 70px; border: solid 30px;
border-color: orange orange silver silver;
border-top-right-radius: 100%; }
```

**Reference image**

![The curved corner is an arc from the top left corner sweeping across the top right corner to the bottom right corner, describing a quarter-ellipse; but since the opposite sides have a border thickness the padding edge curve starts inward from the outer arc's endpoints.](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/partial-curve.png)

Where the border-radius curve extends into the opposite sides' borders, the arc of the padding edge is less than 90°.

<a id="ref-for-margin-edge"></a>

<a id="ref-for-border-edge"></a>

<a id="ref-for-outset-adjusted-border-radius"></a>

The [margin edge](https://www.w3.org/TR/css-box-4/#margin-edge), being outside the [border edge](https://www.w3.org/TR/css-box-4/#border-edge), calculates its radius by <em>adding</em> the corresponding margin thickness to each border radius, with the corresponding [outset-adjusted border radius](#outset-adjusted-border-radius) applied.

<a id="ref-for-box-box-edge"></a>

<a id="ref-for-border-radii"></a>

<a id="ref-for-margin-edge①"></a>

<a id="ref-for-propdef-box-shadow①"></a>

<a id="ref-for-propdef-overflow-clip-margin"></a>

When expanding an [edge](https://www.w3.org/TR/css-box-4/#box-box-edge) that has a [border radius](#border-radii), e.g. for computing the [margin edge](https://www.w3.org/TR/css-box-4/#margin-edge), [box-shadow](#propdef-box-shadow) spread, or [overflow-clip-margin](https://www.w3.org/TR/css-overflow-4/#propdef-overflow-clip-margin), the different <a id="ref-for-border-radii①"></a>border radius values are adjusted so that a small rounded corner with a big outset does not appear to be disproportionally round.

<a id="ref-for-outset-adjusted-border-radius①"></a>

This is done by computing the corresponding [outset-adjusted border radius](#outset-adjusted-border-radius).

<a id="ref-for-size"></a>

To compute the <a id="outset-adjusted-border-radius"></a>outset-adjusted border radius given the 2-dimensional [size](https://www.w3.org/TR/css-sizing-3/#size)’s <var>edge</var>, <var>radius</var>, and <var>outset</var>:

1.  <a id="ref-for-width"></a>

    <a id="ref-for-width①"></a>

    <a id="ref-for-height"></a>

    <a id="ref-for-height①"></a>

    Let <var>coverage</var> be <code>2&#x20;&#x2A;&#x20;min(<var>radius</var>’s&#x20;<a href="https://www.w3.org/TR/css-sizing-3/#width">width</a>&#x20;/&#x20;<var>edge</var>’s&#x20;<span>width</span>,&#x20;<var>radius</var>’s&#x20;<a href="https://www.w3.org/TR/css-sizing-3/#height">height</a>&#x20;/&#x20;<var>edge</var>’s&#x20;<span>height</span>)</code>.

2.  <a id="ref-for-adjusted-radius-dimension"></a>

    <a id="ref-for-width②"></a>

    Let <var>adustedRadiusWidth</var> be the [adjusted radius dimension](#adjusted-radius-dimension) given <var>coverage</var>, <var>radius</var>’s [width](https://www.w3.org/TR/css-sizing-3/#width), and <var>outset</var>’s <a id="ref-for-width③"></a>width.

3.  <a id="ref-for-adjusted-radius-dimension①"></a>

    <a id="ref-for-height②"></a>

    Let <var>adustedRadiusHeight</var> be the [adjusted radius dimension](#adjusted-radius-dimension) given <var>coverage</var>, <var>radius</var>’s [height](https://www.w3.org/TR/css-sizing-3/#height), and <var>outset</var>’s <a id="ref-for-height③"></a>height.

4.  Return (<var>adustedRadiusWidth</var>, <var>adustedRadiusHeight</var>).

To compute the <a id="adjusted-radius-dimension"></a>adjusted radius dimension given numbers <var>coverage</var>, <var>radius</var>, and <var>outset</var>:

1.  If <var>radius</var> is greater than <var>spread</var>, or if <var>coverage</var> is greater than 1, then return <code><var>radius</var>&#x20;+&#x20;<var>outset</var></code>.

2.  Let <var>ratio</var> be <code><var>radius</var>&#x20;/&#x20;<var>outset</var></code>.

3.  Return <code><var>radius</var>&#x20;+&#x20;<var>outset</var>&#x20;&#x2A;&#x20;(1&#x20;-&#x20;(1&#x20;-&#x20;<var>ratio</var>)<sup>3</sup>&#x20;&#x2A;&#x20;(1&#x20;-&#x20;<var>coverage</var><sup>3</sup>))</code>.

<a id="ref-for-border-radii②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: this algorithm is designed to reduce the effect of the <var>outset</var> (or spread) on the shape of the corner. The <var>coverage</var> factor makes this reduction more pronounced for rectangular shapes (where the [border radius](#border-radii) is close to 0), and less pronounced for elliptical shapes (where the <a id="ref-for-border-radii③"></a>border radius is close to 50%).

### <a id="corner-clipping"></a>3.1.  Corner Clipping

<a id="ref-for-propdef-border-radius"></a>

<a id="ref-for-border-edge①"></a>

<a id="ref-for-padding-edge④"></a>

<a id="ref-for-content-edge①"></a>

<a id="ref-for-propdef-background-clip"></a>

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-valdef-overflow-visible"></a>

<a id="ref-for-replaced-element"></a>

Although [border images](#border-image) are not affected by [border-radius](#propdef-border-radius), other effects that clip painting or event handling to the [border](https://www.w3.org/TR/css-box-4/#border-edge), [padding](https://www.w3.org/TR/css-box-4/#padding-edge), or [content](https://www.w3.org/TR/css-box-4/#content-edge) edge must clip to their respective curves. For example, backgrounds clip to the curve specified by [background-clip](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-clip), [overflow](https://www.w3.org/TR/css-overflow-3/#propdef-overflow) values other than [visible](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-visible) to the curved <a id="ref-for-padding-edge⑤"></a>padding edge (when <a id="ref-for-propdef-overflow①"></a>overflow on both axes is not <a id="ref-for-valdef-overflow-visible①"></a>visible), [replaced element](https://www.w3.org/TR/css-display-4/#replaced-element) content to the curved <a id="ref-for-content-edge②"></a>content edge, pointer events to the curved <a id="ref-for-border-edge②"></a>border edge, etc.

<a id="ref-for-propdef-border-radius①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As [border-radius](#propdef-border-radius) reduces the interactive area of an element authors should make sure the remaining interactive area conforms to recommended minima for the platforms they target; in particular, conforming to recommended minimum touch target sizes may require larger widths and heights when <a id="ref-for-propdef-border-radius②"></a>border-radius is used.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b5486bac"></a> This example adds appropriate padding, so that the contents do not overflow the corners. Note that there is no border, but the background will still have rounded corners.
>
> ```text
> DIV {
>     background: black;
>     color: white;
>     border-radius: 1em;
>     padding: 1em }
> ```
### <a id="corner-transitions"></a>3.2.  Color and Style Transitions

<a id="ref-for-border-width-dfn"></a>

Color and style transitions must be contained within the segment of the border that intersects the smallest rectangle that contains both border radii as well as the center of the inner curve (which may be a point representing the corner of the padding edge, if the border radii are smaller than the [border width](#border-width-dfn)).

If one of these borders is zero-width, then the other border takes up the entire transitional area. Otherwise, the center of color and style transitions between adjoining borders is a point along the curve that is a continuous monotonic function of the ratio of the border widths. However it is not defined what these transitions look like or what function maps from this ratio to a point on the curve.

![](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/transition-region.png)

Given these corner shapes, color and style transitions must be contained within the green region. In case D the rectangle defined by the border radii does not include the center of the inner curve (which is a sharp corner), so the transition region is expanded to include that corner. Transitions may take up the entire transition region, but are not required to: For example, a gradient color transition between two solid border styles might take up only the region bounded by the tips of the outer radii and the tips of the inner radii (represented in case D by the dark green region).

Tests

- [border-radius-currentcolor.html](https://wpt.fyi/results/css/css-borders/border-radius-currentcolor.html) [(live test)](http://wpt.live/css/css-borders/border-radius-currentcolor.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/border-radius-currentcolor.html)

### <a id="corner-overlap"></a>3.3.  Overlapping Curves

Corner curves must not overlap: When the sum of any two adjacent border radii exceeds the size of the border box, UAs must proportionally reduce the used values of all border radii until none of them overlap. The algorithm for reducing radii is as follows:

Let <var>f</var> = min(<var>L<sub>i</sub></var>/<var>S<sub>i</sub></var>), where <var>i</var> ∈ {top, right, bottom, left}, <var>S<sub>i</sub></var> is the sum of the two corresponding radii of the corners on side <var>i</var>, and <var>L<sub>top</sub></var> = <var>L<sub>bottom</sub></var> = the width of the box, and <var>L<sub>left</sub></var> = <var>L<sub>right</sub></var> = the height of the box. If <var>f</var> \< 1, then all corner radii are reduced by multiplying them by <var>f</var>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This formula ensures that quarter circles remain quarter circles and large radii remain larger than smaller ones, but it may reduce corners that were already small enough, which may make borders of nearby elements that should look the same look different.

If the curve interferes with UI elements such as scrollbars, the UA may further reduce the used value of the affected border radii (and only the affected border radii) as much as necessary, but no more.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e8de6616"></a> For example, the borders A of the [figure below](#reduced-radius) might be the result of
>
> ```text
> box-sizing: border-box;
> width: 6em;
> height: 2.5em;
> border-radius: 0.5em 2em 0.5em 2em
> ```
>
> The height (2.5em) is enough for the specified radii (0.5em plus 2.0em). However, if the height is only 2em,
>
> ```text
> box-sizing: border-box;
> width: 6em;
> height: 2em;
> border-radius: 0.5em 2em 0.5em 2em
> ```
>
> all corners need to be reduced by a factor 0.8 to make them fit. The used border radii thus are 0.4em (instead of 0.5em) and 1.6em (instead of 2em). See borders B in the figure.
>
> <a id="reduced-radius"></a> ![rectangle with two tiny rounded corners and two very large ones, on opposite corners](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/corner-large-mix.png)
>
> <a id="ref-for-propdef-width"></a>
>
> <a id="ref-for-propdef-height"></a>
>
> These rounded corner might be the result of [width: 6em; height: 2.5em; border-radius: 0.5em 2em 0.5em 2em](https://www.w3.org/TR/css-sizing-3/#propdef-width) for A; and ditto but with [height: 2em](https://www.w3.org/TR/css-sizing-3/#propdef-height) for B.

### <a id="border-radius-tables"></a>3.4.  Effect on Tables

<a id="ref-for-propdef-border-radius③"></a>

<a id="ref-for-valdef-display-table"></a>

<a id="ref-for-valdef-display-inline-table"></a>

<a id="ref-for-valdef-display-table-cell"></a>

<a id="ref-for-propdef-border-collapse"></a>

The [border-radius](#propdef-border-radius) properties do apply to [table](https://www.w3.org/TR/css-display-4/#valdef-display-table), [inline-table](https://www.w3.org/TR/css-display-4/#valdef-display-inline-table), and [table-cell](https://www.w3.org/TR/css-display-4/#valdef-display-table-cell) boxes in separated borders mode ([border-collapse: separate](https://www.w3.org/TR/CSS2/tables.html#propdef-border-collapse)). When <a id="ref-for-propdef-border-collapse①"></a>border-collapse is collapse, they have no effect.

Tests

- [ttwf-reftest-borderRadius.html](https://wpt.fyi/results/css/css-backgrounds/ttwf-reftest-borderRadius.html) [(live test)](http://wpt.live/css/css-backgrounds/ttwf-reftest-borderRadius.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/ttwf-reftest-borderRadius.html)

### <a id="border-radius"></a>3.5. <a id="the-border-radius"></a> Corner Sizing: the border-\*-\*-radius properties



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-top-left-radius"></a>border-top-left-radius, <a id="propdef-border-top-right-radius"></a>border-top-right-radius, <a id="propdef-border-bottom-right-radius"></a>border-bottom-right-radius, <a id="propdef-border-bottom-left-radius"></a>border-bottom-left-radius, <a id="propdef-border-start-start-radius"></a>border-start-start-radius, <a id="propdef-border-start-end-radius"></a>border-start-end-radius, <a id="propdef-border-end-start-radius"></a>border-end-start-radius, <a id="propdef-border-end-end-radius"></a>border-end-end-radius |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-border-radius"></a>[\<border-radius\>](#typedef-border-radius)                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | all elements (but see prose)                                                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-border-box"></a>Refer to corresponding dimension of the [border box](https://www.w3.org/TR/css-box-4/#border-box).                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage"></a>pair of computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-radius④"></a>[border-radius](#propdef-border-radius)                                                                                                                                                                                                                                                                                                         |



<a id="ref-for-typedef-border-radius①"></a>

The radius is specified as a [\<border-radius\>](#typedef-border-radius) value, where

<a id="typedef-border-radius"></a>

<a id="ref-for-typedef-border-radius②"></a>

<a id="ref-for-typedef-slash-separated-border-radius-syntax"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-typedef-legacy-border-radius-syntax"></a>

<a id="typedef-slash-separated-border-radius-syntax"></a>

<a id="ref-for-typedef-slash-separated-border-radius-syntax①"></a>

<a id="ref-for-typedef-length-percentage①"></a>

<a id="ref-for-typedef-length-percentage②"></a>

<a id="ref-for-mult-opt"></a>

<a id="typedef-legacy-border-radius-syntax"></a>

<a id="ref-for-typedef-legacy-border-radius-syntax①"></a>

<a id="ref-for-typedef-length-percentage③"></a>

<a id="ref-for-mult-num-range⑥"></a>

```text
<border-radius> = <slash-separated-border-radius-syntax> | <legacy-border-radius-syntax>
<slash-separated-border-radius-syntax> = <length-percentage [0,∞]> [ / <length-percentage [0,∞]> ]?
<legacy-border-radius-syntax> = <length-percentage [0,∞]>{1,2}
```
<a id="ref-for-typedef-length-percentage④"></a>

<a id="ref-for-border-edge③"></a>

<a id="ref-for-border-box①"></a>

The two [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values of the border-\*-radius properties define the <a id="border-radii"></a>radii of a quarter ellipse that defines the shape of the corner of the outer [border edge](https://www.w3.org/TR/css-box-4/#border-edge) (see the diagram below). The first value is the horizontal radius, the second the vertical radius. If the second value is omitted it is copied from the first. If either length is zero, the corner is square, not rounded. Percentages for the horizontal radius refer to the width of the [border box](https://www.w3.org/TR/css-box-4/#border-box), whereas percentages for the vertical radius refer to the height of the <a id="ref-for-border-box②"></a>border box. Negative values are invalid.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Authors <em>should</em> use the slash syntax, which is preferred for new content, but the legacy syntax (two values separated by whitespace) is supported for backwards compatibility.

![Diagram of the inscribed ellipse](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/corner.png)

<a id="ref-for-propdef-border-top-left-radius"></a>

The two values of [border-top-left-radius: 55pt 25pt](#propdef-border-top-left-radius) define the curvature of the corner.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6b4b7b7d"></a> This example draws ovals of 15em wide and 10em high:
>
> ```text
> DIV.standout {
>     width: 13em;
>     height: 8em;
>     border: solid black 1em;
>     border-radius: 7.5em 5em }
> ```
<a id="ref-for-flow-relative④"></a>

<a id="ref-for-propdef-border-start-start-radius"></a>

<a id="ref-for-propdef-border-start-end-radius"></a>

<a id="ref-for-propdef-border-end-start-radius"></a>

<a id="ref-for-propdef-border-end-end-radius"></a>

<a id="ref-for-physical④"></a>

<a id="ref-for-propdef-border-top-left-radius①"></a>

<a id="ref-for-propdef-border-bottom-left-radius"></a>

<a id="ref-for-propdef-border-top-right-radius"></a>

<a id="ref-for-propdef-border-bottom-right-radius"></a>

<a id="ref-for-propdef-writing-mode④"></a>

<a id="ref-for-propdef-direction④"></a>

<a id="ref-for-propdef-text-orientation④"></a>

The [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) properties [border-start-start-radius](#propdef-border-start-start-radius), [border-start-end-radius](#propdef-border-start-end-radius), [border-end-start-radius](#propdef-border-end-start-radius), and [border-end-end-radius](#propdef-border-end-end-radius) correspond to the [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) properties [border-top-left-radius](#propdef-border-top-left-radius), [border-bottom-left-radius](#propdef-border-bottom-left-radius), [border-top-right-radius](#propdef-border-top-right-radius), and [border-bottom-right-radius](#propdef-border-bottom-right-radius). The mapping depends on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation), with the first start/end giving the block axis side, and the second the inline-axis side (i.e. patterned as 'border-<var>block</var>-<var>inline</var>-radius').

<a id="ref-for-propdef-border-radius⑤"></a>

### <a id="corner-sizing"></a>3.6.  Corner Sizing Shorthands: the [border-radius](#propdef-border-radius) and border-\*-radius shorthand properties

<a id="ref-for-propdef-border-top-radius"></a>

<a id="ref-for-propdef-border-right-radius"></a>

<a id="ref-for-propdef-border-bottom-radius"></a>

<a id="ref-for-propdef-border-left-radius"></a>

<a id="ref-for-propdef-border-block-start-radius"></a>

<a id="ref-for-propdef-border-block-end-radius"></a>

<a id="ref-for-propdef-border-inline-start-radius"></a>

<a id="ref-for-propdef-border-inline-end-radius"></a>

#### <a id="corner-sizing-side-shorthands"></a>3.6.1.  Sizing The Corners Of One Side: The [border-top-radius](#propdef-border-top-radius), [border-right-radius](#propdef-border-right-radius), [border-bottom-radius](#propdef-border-bottom-radius), [border-left-radius](#propdef-border-left-radius), [border-block-start-radius](#propdef-border-block-start-radius), [border-block-end-radius](#propdef-border-block-end-radius), [border-inline-start-radius](#propdef-border-inline-start-radius), [border-inline-end-radius](#propdef-border-inline-end-radius) shorthands



| Field               | Definition                                                                                                                                                                                                                                                                                                                                           |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-top-radius"></a>border-top-radius, <a id="propdef-border-right-radius"></a>border-right-radius, <a id="propdef-border-bottom-radius"></a>border-bottom-radius, <a id="propdef-border-left-radius"></a>border-left-radius, <a id="propdef-border-block-start-radius"></a>border-block-start-radius, <a id="propdef-border-block-end-radius"></a>border-block-end-radius, <a id="propdef-border-inline-start-radius"></a>border-inline-start-radius, <a id="propdef-border-inline-end-radius"></a>border-inline-end-radius   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt①"></a><a id="ref-for-mult-num-range⑦"></a><a id="ref-for-typedef-length-percentage⑤"></a>[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) \[ / <a id="ref-for-typedef-length-percentage⑥"></a>\<length-percentage \[0,∞\]\><a id="ref-for-mult-num-range⑧"></a>{1,2} \][?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | all elements (but see prose)                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-border-box③"></a>Refer to corresponding dimension of the [border box](https://www.w3.org/TR/css-box-4/#border-box).                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                          |



<a id="ref-for-propdef-border-top-radius①"></a>

<a id="ref-for-propdef-border-right-radius①"></a>

<a id="ref-for-propdef-border-bottom-radius①"></a>

<a id="ref-for-propdef-border-left-radius①"></a>

<a id="ref-for-propdef-border-block-start-radius①"></a>

<a id="ref-for-propdef-border-block-end-radius①"></a>

<a id="ref-for-propdef-border-inline-start-radius①"></a>

<a id="ref-for-propdef-border-inline-end-radius①"></a>

The border-\*-radius shorthands set the two border-\*-\*-radius longhand properties of the related side. If values are given before and after the slash, then the values before the slash set the horizontal radius and the values after the slash set the vertical radius. If there is no slash, then the values set both radii equally. The two values for the radii are given in the order top-left, top-right for [border-top-radius](#propdef-border-top-radius), top-right, bottom-right for [border-right-radius](#propdef-border-right-radius), bottom-left, bottom-right for [border-bottom-radius](#propdef-border-bottom-radius), top-left, bottom-left for [border-left-radius](#propdef-border-left-radius), start-start, start-end for [border-block-start-radius](#propdef-border-block-start-radius), end-start, end-end for [border-block-end-radius](#propdef-border-block-end-radius) start-start, end-start for [border-inline-start-radius](#propdef-border-inline-start-radius), and start-end, end-end for [border-inline-end-radius](#propdef-border-inline-end-radius). If the second value is omitted it is copied from the first.

Tests

- [border-radius-greater-than-width.html](https://wpt.fyi/results/css/css-borders/border-radius-greater-than-width.html) [(live test)](http://wpt.live/css/css-borders/border-radius-greater-than-width.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/border-radius-greater-than-width.html)
- [border-radius-side-shorthands-001.html](https://wpt.fyi/results/css/css-borders/tentative/border-radius-side-shorthands/border-radius-side-shorthands-001.html) [(live test)](http://wpt.live/css/css-borders/tentative/border-radius-side-shorthands/border-radius-side-shorthands-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/border-radius-side-shorthands/border-radius-side-shorthands-001.html)
- [border-radius-side-shorthands-002.html](https://wpt.fyi/results/css/css-borders/tentative/border-radius-side-shorthands/border-radius-side-shorthands-002.html) [(live test)](http://wpt.live/css/css-borders/tentative/border-radius-side-shorthands/border-radius-side-shorthands-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/border-radius-side-shorthands/border-radius-side-shorthands-002.html)
- [border-block-end-radius-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-block-end-radius-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-block-end-radius-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-block-end-radius-computed.html)
- [border-block-end-radius-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-block-end-radius-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-block-end-radius-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-block-end-radius-invalid.html)
- [border-block-end-radius-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-block-end-radius-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-block-end-radius-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-block-end-radius-valid.html)
- [border-block-start-radius-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-block-start-radius-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-block-start-radius-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-block-start-radius-computed.html)
- [border-block-start-radius-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-block-start-radius-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-block-start-radius-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-block-start-radius-invalid.html)
- [border-block-start-radius-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-block-start-radius-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-block-start-radius-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-block-start-radius-valid.html)
- [border-bottom-radius-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-bottom-radius-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-bottom-radius-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-bottom-radius-computed.html)
- [border-bottom-radius-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-bottom-radius-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-bottom-radius-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-bottom-radius-invalid.html)
- [border-bottom-radius-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-bottom-radius-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-bottom-radius-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-bottom-radius-valid.html)
- [border-inline-end-radius-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-inline-end-radius-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-inline-end-radius-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-inline-end-radius-computed.html)
- [border-inline-end-radius-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-inline-end-radius-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-inline-end-radius-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-inline-end-radius-invalid.html)
- [border-inline-end-radius-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-inline-end-radius-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-inline-end-radius-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-inline-end-radius-valid.html)
- [border-inline-start-radius-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-inline-start-radius-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-inline-start-radius-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-inline-start-radius-computed.html)
- [border-inline-start-radius-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-inline-start-radius-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-inline-start-radius-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-inline-start-radius-invalid.html)
- [border-inline-start-radius-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-inline-start-radius-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-inline-start-radius-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-inline-start-radius-valid.html)
- [border-left-radius-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-left-radius-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-left-radius-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-left-radius-computed.html)
- [border-left-radius-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-left-radius-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-left-radius-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-left-radius-invalid.html)
- [border-left-radius-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-left-radius-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-left-radius-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-left-radius-valid.html)
- [border-right-radius-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-right-radius-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-right-radius-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-right-radius-computed.html)
- [border-right-radius-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-right-radius-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-right-radius-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-right-radius-invalid.html)
- [border-right-radius-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-right-radius-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-right-radius-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-right-radius-valid.html)
- [border-top-radius-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-top-radius-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-top-radius-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-top-radius-computed.html)
- [border-top-radius-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-top-radius-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-top-radius-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-top-radius-invalid.html)
- [border-top-radius-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-top-radius-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-top-radius-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-top-radius-valid.html)

<!-- -->

- [border-bottom-left-radius-composition.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-bottom-left-radius-composition.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-bottom-left-radius-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-bottom-left-radius-composition.html)
- [border-bottom-right-radius-composition.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-bottom-right-radius-composition.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-bottom-right-radius-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-bottom-right-radius-composition.html)
- [border-radius-interpolation.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-radius-interpolation.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-radius-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-radius-interpolation.html)
- [border-top-left-radius-composition.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-top-left-radius-composition.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-top-left-radius-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-top-left-radius-composition.html)
- [border-top-right-radius-composition.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-top-right-radius-composition.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-top-right-radius-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-top-right-radius-composition.html)
- [border-bottom-left-radius-001.xht](https://wpt.fyi/results/css/css-backgrounds/border-bottom-left-radius-001.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-left-radius-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-left-radius-001.xht)
- [border-bottom-left-radius-004.xht](https://wpt.fyi/results/css/css-backgrounds/border-bottom-left-radius-004.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-left-radius-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-left-radius-004.xht)
- [border-bottom-left-radius-005.xht](https://wpt.fyi/results/css/css-backgrounds/border-bottom-left-radius-005.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-left-radius-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-left-radius-005.xht)
- [border-bottom-left-radius-010.xht](https://wpt.fyi/results/css/css-backgrounds/border-bottom-left-radius-010.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-left-radius-010.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-left-radius-010.xht)
- [border-bottom-left-radius-011.xht](https://wpt.fyi/results/css/css-backgrounds/border-bottom-left-radius-011.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-left-radius-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-left-radius-011.xht)
- [border-bottom-left-radius-014.xht](https://wpt.fyi/results/css/css-backgrounds/border-bottom-left-radius-014.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-left-radius-014.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-left-radius-014.xht)
- [border-bottom-right-radius-001.xht](https://wpt.fyi/results/css/css-backgrounds/border-bottom-right-radius-001.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-right-radius-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-right-radius-001.xht)
- [border-bottom-right-radius-004.xht](https://wpt.fyi/results/css/css-backgrounds/border-bottom-right-radius-004.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-right-radius-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-right-radius-004.xht)
- [border-bottom-right-radius-005.xht](https://wpt.fyi/results/css/css-backgrounds/border-bottom-right-radius-005.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-right-radius-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-right-radius-005.xht)
- [border-bottom-right-radius-010.xht](https://wpt.fyi/results/css/css-backgrounds/border-bottom-right-radius-010.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-right-radius-010.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-right-radius-010.xht)
- [border-bottom-right-radius-011.xht](https://wpt.fyi/results/css/css-backgrounds/border-bottom-right-radius-011.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-right-radius-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-right-radius-011.xht)
- [border-bottom-right-radius-014.xht](https://wpt.fyi/results/css/css-backgrounds/border-bottom-right-radius-014.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-bottom-right-radius-014.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-bottom-right-radius-014.xht)
- [border-radius-001.xht](https://wpt.fyi/results/css/css-backgrounds/border-radius-001.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-001.xht)
- [border-radius-002.xht](https://wpt.fyi/results/css/css-backgrounds/border-radius-002.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-002.xht)
- [border-radius-003.xht](https://wpt.fyi/results/css/css-backgrounds/border-radius-003.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-003.xht)
- [border-radius-004.xht](https://wpt.fyi/results/css/css-backgrounds/border-radius-004.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-004.xht)
- [border-radius-005.xht](https://wpt.fyi/results/css/css-backgrounds/border-radius-005.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-005.xht)
- [border-radius-006.xht](https://wpt.fyi/results/css/css-backgrounds/border-radius-006.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-006.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-006.xht)
- [border-radius-007.xht](https://wpt.fyi/results/css/css-backgrounds/border-radius-007.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-007.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-007.xht)
- [border-radius-008.xht](https://wpt.fyi/results/css/css-backgrounds/border-radius-008.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-008.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-008.xht)
- [border-radius-009.xht](https://wpt.fyi/results/css/css-backgrounds/border-radius-009.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-009.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-009.xht)
- [border-radius-010.xht](https://wpt.fyi/results/css/css-backgrounds/border-radius-010.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-010.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-010.xht)
- [border-radius-011.xht](https://wpt.fyi/results/css/css-backgrounds/border-radius-011.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-011.xht)
- [border-radius-012.html](https://wpt.fyi/results/css/css-backgrounds/border-radius-012.html) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-012.html)
- [border-radius-013.html](https://wpt.fyi/results/css/css-backgrounds/border-radius-013.html) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-013.html)
- [border-radius-clip-001.html](https://wpt.fyi/results/css/css-backgrounds/border-radius-clip-001.html) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-clip-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-clip-001.html)
- [border-radius-clip-002.htm](https://wpt.fyi/results/css/css-backgrounds/border-radius-clip-002.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-clip-002.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-clip-002.htm)
- [border-radius-clipping-002.html](https://wpt.fyi/results/css/css-backgrounds/border-radius-clipping-002.html) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-clipping-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-clipping-002.html)
- [border-radius-clipping-with-transform-001.html](https://wpt.fyi/results/css/css-backgrounds/border-radius-clipping-with-transform-001.html) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-clipping-with-transform-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-clipping-with-transform-001.html)
- [border-radius-css-text.html](https://wpt.fyi/results/css/css-backgrounds/border-radius-css-text.html) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-css-text.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-css-text.html)
- [border-radius-dynamic-from-no-radius.html](https://wpt.fyi/results/css/css-backgrounds/border-radius-dynamic-from-no-radius.html) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-dynamic-from-no-radius.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-dynamic-from-no-radius.html)
- [border-radius-horizontal-value-is-zero.html](https://wpt.fyi/results/css/css-backgrounds/border-radius-horizontal-value-is-zero.html) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-horizontal-value-is-zero.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-horizontal-value-is-zero.html)
- [border-radius-shorthand-002.html](https://wpt.fyi/results/css/css-backgrounds/border-radius-shorthand-002.html) [(live test)](http://wpt.live/css/css-backgrounds/border-radius-shorthand-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-radius-shorthand-002.html)
- [border-top-left-radius-001.xht](https://wpt.fyi/results/css/css-backgrounds/border-top-left-radius-001.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-top-left-radius-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-left-radius-001.xht)
- [border-top-left-radius-004.xht](https://wpt.fyi/results/css/css-backgrounds/border-top-left-radius-004.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-top-left-radius-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-left-radius-004.xht)
- [border-top-left-radius-005.xht](https://wpt.fyi/results/css/css-backgrounds/border-top-left-radius-005.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-top-left-radius-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-left-radius-005.xht)
- [border-top-left-radius-010.xht](https://wpt.fyi/results/css/css-backgrounds/border-top-left-radius-010.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-top-left-radius-010.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-left-radius-010.xht)
- [border-top-left-radius-011.xht](https://wpt.fyi/results/css/css-backgrounds/border-top-left-radius-011.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-top-left-radius-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-left-radius-011.xht)
- [border-top-left-radius-014.xht](https://wpt.fyi/results/css/css-backgrounds/border-top-left-radius-014.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-top-left-radius-014.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-left-radius-014.xht)
- [border-top-right-radius-001.xht](https://wpt.fyi/results/css/css-backgrounds/border-top-right-radius-001.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-top-right-radius-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-right-radius-001.xht)
- [border-top-right-radius-004.xht](https://wpt.fyi/results/css/css-backgrounds/border-top-right-radius-004.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-top-right-radius-004.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-right-radius-004.xht)
- [border-top-right-radius-005.xht](https://wpt.fyi/results/css/css-backgrounds/border-top-right-radius-005.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-top-right-radius-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-right-radius-005.xht)
- [border-top-right-radius-010.xht](https://wpt.fyi/results/css/css-backgrounds/border-top-right-radius-010.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-top-right-radius-010.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-right-radius-010.xht)
- [border-top-right-radius-011.xht](https://wpt.fyi/results/css/css-backgrounds/border-top-right-radius-011.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-top-right-radius-011.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-right-radius-011.xht)
- [border-top-right-radius-014.xht](https://wpt.fyi/results/css/css-backgrounds/border-top-right-radius-014.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-top-right-radius-014.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-top-right-radius-014.xht)
- [box-shadow-border-radius-001.html](https://wpt.fyi/results/css/css-backgrounds/box-shadow-border-radius-001.html) [(live test)](http://wpt.live/css/css-backgrounds/box-shadow-border-radius-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/box-shadow-border-radius-001.html)
- [css-border-radius-001.html](https://wpt.fyi/results/css/css-backgrounds/css-border-radius-001.html) [(live test)](http://wpt.live/css/css-backgrounds/css-border-radius-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/css-border-radius-001.html)
- [inheritance.sub.html](https://wpt.fyi/results/css/css-backgrounds/inheritance.sub.html) [(live test)](http://wpt.live/css/css-backgrounds/inheritance.sub.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/inheritance.sub.html)
- [inner-border-non-renderable.html](https://wpt.fyi/results/css/css-backgrounds/inner-border-non-renderable.html) [(live test)](http://wpt.live/css/css-backgrounds/inner-border-non-renderable.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/inner-border-non-renderable.html)
- [border-radius-computed.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-radius-computed.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-radius-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-radius-computed.html)
- [border-radius-invalid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-radius-invalid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-radius-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-radius-invalid.html)
- [border-radius-valid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-radius-valid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-radius-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-radius-valid.html)
- [webkit-border-radius-valid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/webkit-border-radius-valid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/webkit-border-radius-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/webkit-border-radius-valid.html)

<a id="ref-for-propdef-border-radius⑥"></a>

#### <a id="corner-sizing-shorthand"></a>3.6.2.  Sizing All Corners At Once: The [border-radius](#propdef-border-radius) shorthand



| Field               | Definition                                                                                                                                                                                                                                                                                                                                           |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-radius"></a>border-radius                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt②"></a><a id="ref-for-mult-num-range⑨"></a><a id="ref-for-typedef-length-percentage⑦"></a>[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) \[ / <a id="ref-for-typedef-length-percentage⑧"></a>\<length-percentage \[0,∞\]\><a id="ref-for-mult-num-range①⓪"></a>{1,4} \][?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                          |



<a id="ref-for-propdef-border-radius⑦"></a>

The [border-radius](#propdef-border-radius) shorthand sets all four border-\*-radius properties. If values are given before and after the slash, then the values before the slash set the horizontal radii and the values after the slash set the vertical radii. If there is no slash, then the values set the radii in both axes equally. The four values for each radii are given in the order top-left, top-right, bottom-right, bottom-left. If bottom-left is omitted it is the same as top-right. If bottom-right is omitted it is the same as top-left. If top-right is omitted it is the same as top-left.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-19e399d6"></a>
>
> ```text
> border-radius: 4em;
> ```
>
> is equivalent to
>
> ```text
> border-top-left-radius:     4em;
> border-top-right-radius:    4em;
> border-bottom-right-radius: 4em;
> border-bottom-left-radius:  4em;
> ```
>
> and
>
> ```text
> border-radius: 2em 1em 4em / 0.5em 3em;
> ```
>
> is equivalent to
>
> ```text
> border-top-left-radius:     2em 0.5em;
> border-top-right-radius:    1em 3em;
> border-bottom-right-radius: 4em 0.5em;
> border-bottom-left-radius:  1em 3em;
> ```
### <a id="corner-shaping"></a>3.7.  Corner Shaping: the corner-\*-shape properties

<a id="ref-for-propdef-border-radius⑧"></a>

<a id="ref-for-corner-shape"></a>

By default, non-zero [border-radius](#propdef-border-radius) values define a quarter-ellipse <a id="corner-shape"></a>corner shape that rounds the affected corners, filling the <a id="corner-area"></a>corner area defined by the <a id="ref-for-propdef-border-radius⑨"></a>border-radius for that corner. However in some cases, other [corner shapes](#corner-shape) are desired. The corner-\*-shape properties (and their shorthands) specify exactly what <a id="ref-for-corner-shape①"></a>corner shapes a box will use for the region defined by its border-\*-radius values.

<a id="ref-for-corner-shape②"></a>

The different [corner shapes](#corner-shape) can all be expressed as different parameters to a <em>superellipse</em>. A superellipse is a generalization of an ellipse, and based on its \`k\` parameter can express all the shapes between a square, an ellipse, and a notch.

> <strong data-conversion-semantic="note">Note</strong>
>
> How does a superellipse work?
>
> A unit circle is defined by the equation:
>
> <strong>Mathematical expression 1</strong>
>
> TeX transcription (renderer-independent source notation):
>
> ``` language-tex
> x^{2} + y^{2} = 1
> ```
>
> Source mathematical tokens: x 2 + y 2 = 1
>
> The circle is made from all points (x,y) that satisfy the equation. A given ellipse can then be produced by scaling this shape in the X and/or Y axis.
>
> The unit superellipse equation just changes the <sup>2</sup> exponent into a variable. For this spec’s purposes, we’ll write it as 2<sup>K</sup>:
>
> <strong>Mathematical expression 2</strong>
>
> TeX transcription (renderer-independent source notation):
>
> ``` language-tex
> x^{2^{K}} + y^{2^{K}} = 1
> ```
>
> Source mathematical tokens: x 2 K + y 2 K = 1
>
> <a id="ref-for-funcdef-superellipse"></a>
>
> The <var>K</var> in this equation is the [superellipse()](#funcdef-superellipse) argument.
>
> <var>K</var> can be any value; setting <var>K</var> to 1 gives the standard circle/ellipse equation, but other values define the entire family of superellipse curves:
>
> - Values larger than 1 make it more "square": the traditional "squircle" uses a <var>K</var> of 2, and a <var>K</var> of infinity is a perfect square. (A <var>K</var> of only 10 is already nearly indistinguishable from a square; it scales very quickly.)
>
> - Values between 0 and 1 make it "flatter"; when <var>K</var> is 0 it’s a diamond with perfectly flat sides.
>
> - Negative values define concave curves, roughly opposite of what you get with positive values: a <var>K</var> of -1 gives a nearly elliptical "scoop" that’s roughly the opposite of a <var>K</var> of 1, a <var>K</var> of -2 gives a "squircle" scoop, a <var>K</var> of negative infinity gives a square scoop, etc.
>
> (Note that most literature on superellipses will write the equation with a simpler
>
> <strong>Mathematical expression 3</strong>
>
> TeX transcription (renderer-independent source notation):
>
> ``` language-tex
> x^{K}
> ```
>
> Source mathematical tokens: x K
>
> exponent. The
>
> <strong>Mathematical expression 4</strong>
>
> TeX transcription (renderer-independent source notation):
>
> ``` language-tex
> x^{2^{K}}
> ```
>
> Source mathematical tokens: x 2 K
>
> form was chosen here to make the argument ranges easier to reason about: all possible values are valid, the symmetrical shapes are just positive/negative, the "middle" bevel is 0, etc.)

<a id="ref-for-propdef-corner-shape"></a>

<a id="ref-for-funcdef-superellipse①"></a>

<a id="ref-for-typedef-corner-shape-value"></a>

To allow full expression as well as interpolation, the [corner-shape](#propdef-corner-shape) properties can provide the superellipse parameter directly using the [superellipse()](#funcdef-superellipse) function, or use one of the supplied keywords which represent commonly used parameters. See the [\<corner-shape-value\>](#typedef-corner-shape-value) definition for details.

Tests

- [corner-shape-backdrop-filter.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-backdrop-filter.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-backdrop-filter.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-backdrop-filter.html)
- [corner-shape-backdrop-filter-overflow.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-backdrop-filter-overflow.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-backdrop-filter-overflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-backdrop-filter-overflow.html)
- [corner-shape-bevel-overflow-composite.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-bevel-overflow-composite.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-bevel-overflow-composite.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-bevel-overflow-composite.html)
- [corner-shape-bevel-overflow.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-bevel-overflow.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-bevel-overflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-bevel-overflow.html)
- [corner-shape-computed.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-computed.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-computed.html)
- [corner-shape-fill.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-fill.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-fill.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-fill.html)
- [corner-shape-hittest.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-hittest.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-hittest.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-hittest.html)
- [corner-shape-iframe-border.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-iframe-border.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-iframe-border.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-iframe-border.html)
- [corner-shape-img-border.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-img-border.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-img-border.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-img-border.html)
- [corner-shape-img.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-img.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-img.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-img.html)
- [corner-shape-inset-shadow.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-inset-shadow.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-inset-shadow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-inset-shadow.html)
- [corner-shape-invalid.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-invalid.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-invalid.html)
- [corner-shape-notch.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-notch.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-notch.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-notch.html)
- [corner-shape-outside-left.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-outside-left.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-outside-left.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-outside-left.html)
- [corner-shape-outside-right.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-outside-right.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-outside-right.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-outside-right.html)
- [corner-shape-overflow-clip-margin.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-overflow-clip-margin.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-overflow-clip-margin.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-overflow-clip-margin.html)
- [corner-shape-square.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-square.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-square.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-square.html)
- [corner-shape-svg-border.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-svg-border.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-svg-border.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-svg-border.html)
- [corner-shape-valid.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-valid.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-valid.html)
- [corner-shape-video-border.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-video-border.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-video-border.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-video-border.html)
- [corner-shape-noop-keyframe.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-noop-keyframe.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-noop-keyframe.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-noop-keyframe.html)
- [corner-shape-zoom-overlap-extreme-values-crash.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-zoom-overlap-extreme-values-crash.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-zoom-overlap-extreme-values-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-zoom-overlap-extreme-values-crash.html)
- [render-corner-shape.html](https://wpt.fyi/results/css/css-borders/corner-shape/render-corner-shape.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/render-corner-shape.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/render-corner-shape.html)



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                 |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-corner-top-left-shape"></a>corner-top-left-shape, <a id="propdef-corner-top-right-shape"></a>corner-top-right-shape, <a id="propdef-corner-bottom-right-shape"></a>corner-bottom-right-shape, <a id="propdef-corner-bottom-left-shape"></a>corner-bottom-left-shape, <a id="propdef-corner-start-start-shape"></a>corner-start-start-shape, <a id="propdef-corner-start-end-shape"></a>corner-start-end-shape, <a id="propdef-corner-end-start-shape"></a>corner-end-start-shape, <a id="propdef-corner-end-end-shape"></a>corner-end-end-shape |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-corner-shape-value①"></a>[\<corner-shape-value\>](#typedef-corner-shape-value)                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | round                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-radius①⓪"></a>all elements where [border-radius](#propdef-border-radius) can apply                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-funcdef-superellipse②"></a>the corresponding [superellipse()](#funcdef-superellipse) value                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <a id="ref-for-superellipse-interpolation"></a>see [superellipse interpolation](#superellipse-interpolation)                                                                                                                                                                                                                                                                           |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-corner-shape①"></a>[corner-shape](#propdef-corner-shape)                                                                                                                                                                                                                                                                                                   |



The corner-\*-\*-shape longhand properties set the corner shape for the given corner.

<a id="ref-for-flow-relative⑤"></a>

<a id="ref-for-propdef-corner-start-start-shape"></a>

<a id="ref-for-physical⑤"></a>

<a id="ref-for-propdef-corner-top-left-shape"></a>

<a id="ref-for-propdef-writing-mode⑤"></a>

<a id="ref-for-propdef-direction⑤"></a>

<a id="ref-for-propdef-text-orientation⑤"></a>

The [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) longhands ([corner-start-start-shape](#propdef-corner-start-start-shape), etc.) correspond to the [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) longhands ([corner-top-left-shape](#propdef-corner-top-left-shape), etc.) depending on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation). The first start/end gives the block axis side, and the second the inline-axis side (i.e. patterned as corner-<var>block</var>-<var>inline</var>-shape).

<a id="typedef-corner-shape-value"></a>

<a id="ref-for-typedef-corner-shape-value②"></a>

<a id="ref-for-valdef-corner-shape-value-round"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-valdef-corner-shape-value-scoop"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-valdef-corner-shape-value-bevel"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-valdef-corner-shape-value-notch"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-valdef-corner-shape-value-square"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-valdef-corner-shape-value-squircle"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-funcdef-superellipse③"></a>

<a id="funcdef-superellipse"></a>

<a id="ref-for-number-value"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-comb-one②②"></a>

```text
<corner-shape-value> = round | scoop | bevel | notch | square | squircle |
                       <superellipse()>
superellipse() = superellipse(<number> | infinity | -infinity)
```
<a id="valdef-corner-shape-value-round"></a>round  
<a id="ref-for-corner-shape③"></a>

The [corner shape](#corner-shape) is a quarter of a convex ellipse. Equivalent to superellipse(1).

<a id="ref-for-propdef-corner-shape②"></a>

<a id="ref-for-propdef-border-radius①①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This is the initial value of [corner-shape](#propdef-corner-shape) properties, as it was the behavior of [border-radius](#propdef-border-radius) before <a id="ref-for-propdef-corner-shape③"></a>corner-shape existed.

<a id="valdef-corner-shape-value-squircle"></a>squircle  
<a id="ref-for-valdef-corner-shape-value-square①"></a>

<a id="ref-for-valdef-corner-shape-value-round①"></a>

<a id="ref-for-corner-shape④"></a>

The [corner shape](#corner-shape) is a quarter of a "squircle", a convex curve between [round](#valdef-corner-shape-value-round) and [square](#valdef-corner-shape-value-square). Equivalent to superellipse(2).

<a id="valdef-corner-shape-value-square"></a>square  
<a id="ref-for-corner-shape⑤"></a>

The [corner shape](#corner-shape) is a convex 90deg angle. Equivalent to superellipse(infinity).

<a id="ref-for-propdef-border-radius①②"></a>

<a id="ref-for-propdef-corner-shape④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This looks identical to the "normal" square corner you get from [border-radius: 0](#propdef-border-radius), but it can smoothly animate with the other [corner-shape](#propdef-corner-shape) values.

<a id="valdef-corner-shape-value-bevel"></a>bevel  
<a id="ref-for-corner-shape⑥"></a>

The [corner shape](#corner-shape) is a straight diagonal line, neither convex nor concave. Equivalent to superellipse(0).

<a id="valdef-corner-shape-value-scoop"></a>scoop  
<a id="ref-for-corner-shape⑦"></a>

The [corner shape](#corner-shape) is a concave quarter-ellipse. Equivalent to superellipse(-1).

<a id="valdef-corner-shape-value-notch"></a>notch  
<a id="ref-for-corner-shape⑧"></a>

The [corner shape](#corner-shape) is a concave 90deg angle. Equivalent to superellipse(-infinity).

<a id="valdef-corner-shape-value-superellipse-k"></a>superellipse(K)  
<a id="ref-for-corner-shape⑨"></a>

The [corner shape](#corner-shape) is a quarter of a superellipse. The argument <var>K</var> is the <a id="superellipse-parameter"></a>superellipse parameter, and it defines a superellipse using an exponent of 2<sup>K</sup>.

See the note in [§ 3.7 Corner Shaping: the corner-\*-shape properties](#corner-shaping) for an explanation of the mathematical definition of a superellipse, and what various K values mean. See [§ 3.9.4 Rendering corner-shape](#corner-shape-rendering) for precise details of how the superellipse is computed and rendered.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-94a31832"></a>
>
> ![](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/superellipse-param.svg)
>
> <a id="ref-for-funcdef-superellipse④"></a>
>
> Different [superellipse()](#funcdef-superellipse) values for the top right corner: infinity, 1, 0, -1, and -infinity.

<a id="ref-for-propdef-border-radius①③"></a>

<a id="ref-for-corner-area"></a>

<a id="ref-for-propdef-corner-shape⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If [border-radius](#propdef-border-radius) is not specified (or is set to 0), the [corner area](#corner-area) is zero-sized as well, and [corner-shape](#propdef-corner-shape) won’t have any effect.

<a id="ref-for-overflow"></a>

corner-\*-shape does not alter the [overflow](https://www.w3.org/TR/css-overflow-3/#overflow) rules for border-\*-radius, except insofar as it shapes the corners differently; elements are still clipped by the shaped border as normal.

<a id="ref-for-propdef-border-width⑦"></a>

The curve specified by corner-\*-shape defines the <em>outer</em> edge of the border. The inner edge of the border follows the curve of the outer edge (in a way that’s not necessarily expressible as a superellipse curve), with a nearly consistent distance from the outer edge throughout, (or a linearly increasing distance if the [border-width](#propdef-border-width) of the two border edges meeting at the the corner are not uniform).

<a id="ref-for-propdef-box-shadow②"></a>

<a id="ref-for-overflow-clip-edge"></a>

<a id="ref-for-propdef-corner-shape⑥"></a>

corner-\*-shape also affects the rendering of [box-shadow](#propdef-box-shadow), and how the [overflow clip edge](https://www.w3.org/TR/css-overflow-4/#overflow-clip-edge) is shaped when it’s extended from the box, but these do not directly follow the corner-\*-shape path like the inner border edge does. Instead, it scales the [corner-shape](#propdef-corner-shape) path in an axis-aligned manner.

<a id="ref-for-propdef-corner-shape⑦"></a>

### <a id="corner-shaping-shorthands"></a>3.8.  Corner Shaping Shorthands: the [corner-shape](#propdef-corner-shape) and corner-\*-shape shorthand properties

#### <a id="corner-shaping-side-shorthands"></a>3.8.1.  Shaping The Corners Of One Side: The corner-\*-shape shorthands:



| Field               | Definition                                                                                                                                                                                                                                                                                                                                 |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-corner-top-shape"></a>corner-top-shape, <a id="propdef-corner-right-shape"></a>corner-right-shape, <a id="propdef-corner-bottom-shape"></a>corner-bottom-shape, <a id="propdef-corner-left-shape"></a>corner-left-shape, <a id="propdef-corner-block-start-shape"></a>corner-block-start-shape, <a id="propdef-corner-block-end-shape"></a>corner-block-end-shape, <a id="propdef-corner-inline-start-shape"></a>corner-inline-start-shape, <a id="propdef-corner-inline-end-shape"></a>corner-inline-end-shape |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range①①"></a><a id="ref-for-propdef-corner-top-left-shape①"></a>[\<'corner-top-left-shape'\>](#propdef-corner-top-left-shape)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range)                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                |



The corner-\*-shape shorthands set the two corner-\*-\*-shape properties of the related side. If only one value is given, the second value defaults to the same value.

<a id="ref-for-propdef-corner-top-shape"></a>

<a id="ref-for-propdef-corner-top-left-shape②"></a>

For the physical shorthands ([corner-top-shape](#propdef-corner-top-shape), etc.), the values are either in left/right order, or top/bottom order, whichever axis is meaningful for the property. That is, <a id="ref-for-propdef-corner-top-shape①"></a>corner-top-shape: round square sets [corner-top-left-shape: round; corner-top-right-shape: square;](#propdef-corner-top-left-shape).

<a id="ref-for-propdef-corner-block-start-shape"></a>

<a id="ref-for-propdef-corner-start-start-shape①"></a>

For the logical shorthands ([corner-block-start-shape](#propdef-corner-block-start-shape), etc.), the values are always in start/end order in the other axis. That is, <a id="ref-for-propdef-corner-block-start-shape①"></a>corner-block-start-shape: round square sets [corner-start-start-shape: round; corner-start-end-shape: square;](#propdef-corner-start-start-shape).

<a id="ref-for-propdef-corner-shape⑧"></a>

#### <a id="corner-shaping-shorthand"></a>3.8.2.  Shaping All Corners At Once: The [corner-shape](#propdef-corner-shape) shorthand



| Field               | Definition                                                                                                                                                     |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-corner-shape"></a>corner-shape                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range①②"></a><a id="ref-for-propdef-corner-top-left-shape③"></a>[\<'corner-top-left-shape'\>](#propdef-corner-top-left-shape)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | round                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-radius①④"></a>all elements where [border-radius](#propdef-border-radius) can apply                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                    |



<a id="ref-for-propdef-corner-shape⑨"></a>

<a id="ref-for-propdef-border-radius①⑤"></a>

The [corner-shape](#propdef-corner-shape) property specifies the shape of the box’s corners, within the region specified by [border-radius](#propdef-border-radius). The four values set the top, right, bottom and left shape, respectively. A missing left is the same as right, a missing bottom is the same as top, and a missing right is also the same as top.

<a id="ref-for-propdef-corner"></a>

### <a id="corner-shorthands"></a>3.9.  Corner Sizing &#x26; Shaping Shorthands: the [corner](#propdef-corner) and corner-\* shorthand properties

<a id="ref-for-propdef-corner-top-left"></a>

<a id="ref-for-propdef-corner-top-right"></a>

<a id="ref-for-propdef-corner-bottom-right"></a>

<a id="ref-for-propdef-corner-bottom-left"></a>

<a id="ref-for-propdef-corner-start-start"></a>

<a id="ref-for-propdef-corner-start-end"></a>

<a id="ref-for-propdef-corner-end-start"></a>

<a id="ref-for-propdef-corner-end-end"></a>

#### <a id="single-corner-shorthands"></a>3.9.1.  Sizing &#x26; shaping a corner: The [corner-top-left](#propdef-corner-top-left), [corner-top-right](#propdef-corner-top-right), [corner-bottom-right](#propdef-corner-bottom-right), [corner-bottom-left](#propdef-corner-bottom-left), [corner-start-start](#propdef-corner-start-start), [corner-start-end](#propdef-corner-start-end), [corner-end-start](#propdef-corner-end-start), [corner-end-end](#propdef-corner-end-end) shorthands



| Field               | Definition                                                                                                                                                                                                                                                                                                 |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-corner-top-left"></a>corner-top-left, <a id="propdef-corner-top-right"></a>corner-top-right, <a id="propdef-corner-bottom-left"></a>corner-bottom-left, <a id="propdef-corner-bottom-right"></a>corner-bottom-right, <a id="propdef-corner-start-start"></a>corner-start-start, <a id="propdef-corner-start-end"></a>corner-start-end, <a id="propdef-corner-end-start"></a>corner-end-start, <a id="propdef-corner-end-end"></a>corner-end-end |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-corner-top-left-shape④"></a><a id="ref-for-comb-any④"></a><a id="ref-for-propdef-border-top-left-radius②"></a>[\<'border-top-left-radius'\>](#propdef-border-top-left-radius) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'corner-top-left-shape'\>](#propdef-corner-top-left-shape)                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | all elements (but see prose)                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-border-box④"></a>Refer to corresponding dimension of the [border box](https://www.w3.org/TR/css-box-4/#border-box).                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                  |



The corner-\*-\* shorthands set the two corner-\*-\*-shape and border-\*-\*-radius longhand properties of the related side. See the corresponding corner-\*-\*-shape and border-\*-\*-radius properties for further details.

<a id="ref-for-propdef-corner-top"></a>

<a id="ref-for-propdef-corner-right"></a>

<a id="ref-for-propdef-corner-bottom"></a>

<a id="ref-for-propdef-corner-left"></a>

<a id="ref-for-propdef-corner-block-start"></a>

<a id="ref-for-propdef-corner-block-end"></a>

<a id="ref-for-propdef-corner-inline-start"></a>

<a id="ref-for-propdef-corner-inline-end"></a>

#### <a id="corner-side-shorthands"></a>3.9.2.  Sizing &#x26; shaping rhe Corners Of One Side: The [corner-top](#propdef-corner-top), [corner-right](#propdef-corner-right), [corner-bottom](#propdef-corner-bottom), [corner-left](#propdef-corner-left), [corner-block-start](#propdef-corner-block-start), [corner-block-end](#propdef-corner-block-end), [corner-inline-start](#propdef-corner-inline-start), [corner-inline-end](#propdef-corner-inline-end) shorthands



| Field               | Definition                                                                                                                                                                                                                                                                                 |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-corner-top"></a>corner-top, <a id="propdef-corner-right"></a>corner-right, <a id="propdef-corner-bottom"></a>corner-bottom, <a id="propdef-corner-left"></a>corner-left, <a id="propdef-corner-block-start"></a>corner-block-start, <a id="propdef-corner-block-end"></a>corner-block-end, <a id="propdef-corner-inline-start"></a>corner-inline-start, <a id="propdef-corner-inline-end"></a>corner-inline-end |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-corner-top-shape②"></a><a id="ref-for-comb-any⑤"></a><a id="ref-for-propdef-border-top-radius②"></a>[\<'border-top-radius'\>](#propdef-border-top-radius) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'corner-top-shape'\>](#propdef-corner-top-shape)                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | all elements (but see prose)                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-border-box⑤"></a>Refer to corresponding dimension of the [border box](https://www.w3.org/TR/css-box-4/#border-box).                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                |



The corner-\* shorthands set the two corner-\*-shape longhand properties and two border-\*-radius longhand properties of the related side. See the corresponding corner-\*-shape and border-\*-radius properties for further details.

<a id="ref-for-propdef-corner①"></a>

#### <a id="corner-shorthand"></a>3.9.3.  Sizing &#x26; Shaping All Corners At Once: The [corner](#propdef-corner) shorthand



| Field               | Definition                                                                                                                                                                                              |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-corner"></a>corner                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-corner-shape①⓪"></a><a id="ref-for-comb-any⑥"></a><a id="ref-for-propdef-border-radius①⑥"></a>[\<'border-radius'\>](#propdef-border-radius) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'corner-shape'\>](#propdef-corner-shape) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | all elements (but see prose)                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-border-box⑥"></a>Refer to corresponding dimension of the [border box](https://www.w3.org/TR/css-box-4/#border-box).                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                               |



<a id="ref-for-propdef-corner②"></a>

The [corner](#propdef-corner) shorthand sets the corner-\*-shape and border-\*-radius longhands all together.

<a id="ref-for-propdef-corner-shape①①"></a>

#### <a id="corner-shape-rendering"></a>3.9.4.  Rendering [corner-shape](#propdef-corner-shape)

<a id="ref-for-border⑤"></a>

<a id="ref-for-x2"></a>

<a id="ref-for-propdef-box-shadow③"></a>

<a id="ref-for-propdef-overflow-clip-margin①"></a>

When rendering elements with shaped corners, the element’s path needs to be offset, based on [border](https://www.w3.org/TR/css-box-4/#border), [outline](https://www.w3.org/TR/CSS2/ui.html#x2), [box-shadow](#propdef-box-shadow), [overflow-clip-margin](https://www.w3.org/TR/css-overflow-4/#propdef-overflow-clip-margin) and more.

<a id="ref-for-propdef-box-shadow④"></a>

<a id="ref-for-propdef-overflow-clip-margin②"></a>

When rendering borders or outlines, the offset is aligned to the curve of the element’s shape, while when rendering [box-shadow](#propdef-box-shadow) or offsetting for [overflow-clip-margin](https://www.w3.org/TR/css-overflow-4/#propdef-overflow-clip-margin), the offset is aligned to the axis.

![Adjusting corner shapes](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/corner-shape-adjusting.svg)

Borders are aligned to the curve, shadows and clip are aligned to the axis.

<a id="ref-for-concept-element"></a>

<a id="ref-for-border-contour-path"></a>

<a id="ref-for-border-edge④"></a>

An [element](https://dom.spec.whatwg.org/#concept-element) <var>element</var>’s <a id="outer-contour"></a>outer contour is the [border contour path](#border-contour-path) given <var>element</var> and <var>element</var>’s [border edge](https://www.w3.org/TR/css-box-4/#border-edge).

<a id="ref-for-concept-element①"></a>

<a id="ref-for-border-contour-path①"></a>

<a id="ref-for-padding-edge⑥"></a>

An [element](https://dom.spec.whatwg.org/#concept-element) <var>element</var>’s <a id="inner-contour"></a>inner contour is the [border contour path](#border-contour-path) given <var>element</var> and <var>element</var>’s [padding edge](https://www.w3.org/TR/css-box-4/#padding-edge).

<a id="ref-for-concept-element②"></a>

<a id="ref-for-border⑥"></a>

<a id="ref-for-outer-contour"></a>

<a id="ref-for-inner-contour"></a>

An [element](https://dom.spec.whatwg.org/#concept-element)’s [border](https://www.w3.org/TR/css-box-4/#border) is rendered in the area between its [outer contour](#outer-contour) and its [inner contour](#inner-contour).

<a id="ref-for-concept-element③"></a>

<a id="ref-for-x2①"></a>

<a id="ref-for-outer-contour①"></a>

<a id="ref-for-used-value①"></a>

<a id="ref-for-propdef-outline-width"></a>

<a id="ref-for-propdef-outline-offset"></a>

An [element](https://dom.spec.whatwg.org/#concept-element)’s [outline](https://www.w3.org/TR/CSS2/ui.html#x2) follows the [outer contour](#outer-contour) with the [used](https://www.w3.org/TR/css-cascade-5/#used-value) [outline-width](https://www.w3.org/TR/css-ui-4/#propdef-outline-width) and [outline-offset](https://www.w3.org/TR/css-ui-4/#propdef-outline-offset). The precise way in which it is rendered is implementation-defined.

<a id="ref-for-concept-element④"></a>

<a id="ref-for-overflow①"></a>

<a id="ref-for-inner-contour①"></a>

<a id="ref-for-overflow-clip-edge①"></a>

<a id="ref-for-border-contour-path②"></a>

<a id="ref-for-padding-edge⑦"></a>

<a id="ref-for-used-value②"></a>

<a id="ref-for-propdef-overflow-clip-margin③"></a>

An [element](https://dom.spec.whatwg.org/#concept-element)’s [overflow](https://www.w3.org/TR/css-overflow-3/#overflow) area is shaped by its [inner contour](#inner-contour). An <a id="ref-for-concept-element⑤"></a>element’s [overflow clip edge](https://www.w3.org/TR/css-overflow-4/#overflow-clip-edge) is shaped by the [border contour path](#border-contour-path) given <var>element</var>, and <var>element</var>’s [padding edge](https://www.w3.org/TR/css-box-4/#padding-edge), and <var>element</var>’s [used](https://www.w3.org/TR/css-cascade-5/#used-value) [overflow-clip-margin](https://www.w3.org/TR/css-overflow-4/#propdef-overflow-clip-margin).

<a id="ref-for-concept-element⑥"></a>

<a id="ref-for-border-contour-path③"></a>

<a id="ref-for-border-edge⑤"></a>

<a id="ref-for-used-value③"></a>

<a id="ref-for-propdef-box-shadow-spread"></a>

Each shadow of [element](https://dom.spec.whatwg.org/#concept-element)’s 'box shadow' is shaped by the [border contour path](#border-contour-path) given <var>element</var>, and <var>element</var>’s [border edge](https://www.w3.org/TR/css-box-4/#border-edge), and the shadow’s [used](https://www.w3.org/TR/css-cascade-5/#used-value) [box-shadow-spread](#propdef-box-shadow-spread).

<a id="ref-for-concept-element⑦"></a>

<a id="ref-for-box-box-edge①"></a>

To compute an [element](https://dom.spec.whatwg.org/#concept-element) <var>element</var>’s <a id="border-contour-path"></a>border contour path given an [edge](https://www.w3.org/TR/css-box-4/#box-box-edge) <var>targetEdge</var> and an optional number <var>spread</var> (default 0):

1.  <a id="ref-for-unshaped-edge"></a>

    <a id="ref-for-border-edge⑥"></a>

    Let <var>outerLeft</var>, <var>outerTop</var>, <var>outerRight</var>, <var>outerBottom</var> be <var>element</var>’s [unshaped](https://www.w3.org/TR/css-box-4/#unshaped-edge) [border edge](https://www.w3.org/TR/css-box-4/#border-edge), outset by <var>spread</var>.

2.  <a id="ref-for-border-edge⑦"></a>

    <a id="ref-for-opposite-corner-scale-factor"></a>

    <a id="ref-for-outset-adjusted-border-radius②"></a>

    Let <var>topLeftHorizontalRadius</var>, <var>topLeftVericalRadius</var>, <var>topRightHorizontalRadius</var>, <var>topRightVerticalRadius</var>, <var>bottomRightHorizontalRadius</var>, <var>bottomRightVerticalRadius</var>, <var>bottomLeftHorizontalRadius</var>, and <var>bottomLeftVerticalRadius</var> be <var>element</var> [border edge](https://www.w3.org/TR/css-box-4/#border-edge)’s radii, scaled by <var>element</var>’s [opposite corner scale factor](#opposite-corner-scale-factor) and [outset-adjusted](#outset-adjusted-border-radius).

3.  <a id="ref-for-computed-value"></a>

    Let <var>topLeftShape</var>, <var>topRightShape</var>, <var>bottomRightShape</var>, and <var>bottomLeftShape</var> be <var>element</var>’s [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) corner-\*-shape values.

4.  <a id="ref-for-unshaped-edge①"></a>

    Let <var>targetLeft</var>, <var>targetTop</var>, <var>targetRight</var>, <var>targetBottom</var> [unshaped](https://www.w3.org/TR/css-box-4/#unshaped-edge) <var>targetEdge</var>.

5.  Let <var>path</var> be a new path [\[SVG2\]](#biblio-svg2).

6.  <a id="ref-for-add-corner-to-path"></a>

    <a id="ref-for-rectangle"></a>

    [Add corner to path](#add-corner-to-path) given <var>path</var>, the [rectangle](https://www.w3.org/TR/geometry-1/#rectangle) <code>(<var>outerRight</var>&#x20;-&#x20;<var>topRightHorizontalRadius</var>,&#x20;<var>outerTop</var>,&#x20;<var>topRightHorizontalRadius</var>,&#x20;<var>topRightVerticalRadius</var>)</code>, <var>targetEdge</var>, 0, <var>targetTop</var> - <var>outerTop</var>, <var>outerRight</var> - <var>targetRight</var>, and <var>topRightShape</var>.

7.  <a id="ref-for-add-corner-to-path①"></a>

    <a id="ref-for-rectangle①"></a>

    [Add corner to path](#add-corner-to-path) given <var>path</var>, the [rectangle](https://www.w3.org/TR/geometry-1/#rectangle) <code>(<var>outerRight</var>&#x20;-&#x20;<var>bottomRightHorizontalRadius</var>,&#x20;<var>outerBottom</var>&#x20;-&#x20;<var>bottomRightVerticalRadius</var>,&#x20;<var>bottomRightHorizontalRadius</var>,&#x20;<var>bottomRightVerticalRadius</var>)</code>, <var>targetEdge</var>, 1, <var>outerRight</var> - <var>targetRight</var>, <var>outerBottom</var> - <var>targetBottom</var>, and <var>bottomRightShape</var>.

8.  <a id="ref-for-add-corner-to-path②"></a>

    <a id="ref-for-rectangle②"></a>

    [Add corner to path](#add-corner-to-path) given <var>path</var>, the [rectangle](https://www.w3.org/TR/geometry-1/#rectangle) <code>(<var>outerLeft</var>,&#x20;<var>outerBottom</var>&#x20;-&#x20;<var>bottomLeftVerticalRadius</var>,&#x20;<var>bottomLeftHorizontalRadius</var>,&#x20;<var>bottomLeftVerticalRadius</var>)</code>, <var>targetEdge</var>, 2, <var>outerBottom</var> - <var>targetBottom</var>, <var>targetLeft</var> - <var>outerLeft</var>, and <var>bottomLeftShape</var>.

9.  <a id="ref-for-add-corner-to-path③"></a>

    <a id="ref-for-rectangle③"></a>

    [Add corner to path](#add-corner-to-path) given <var>path</var>, the [rectangle](https://www.w3.org/TR/geometry-1/#rectangle) <code>(<var>outerLeft</var>,&#x20;<var>outerTop</var>,&#x20;<var>topLeftHorizontalRadius</var>,&#x20;<var>topLeftVericalRadius</var>)</code>, <var>targetEdge</var>, 3, <var>targetLeft</var> - <var>outerLeft</var>, <var>targetTop</var> - <var>outerTop</var>, and <var>topLeftShape</var>.

10. Return <var>path</var>.

To <a id="add-corner-to-path"></a>add corner to path given a path <var>path</var>, a rectangle <var>cornerRect</var>, a rectangle <var>trimRect</var>, and numbers <var>orientation</var>, <var>startThickness</var>, <var>endThickness</var>, <var>curvature</var>:

1.  If <var>cornerRect</var> is empty, or if <var>curvature</var> is ∞:

    1.  <a id="ref-for-clockwise-quad"></a>

        Let <var>innerQuad</var> be <var>trimRect</var>’s [clockwise quad](#clockwise-quad) .

    2.  Extend <var>path</var> by drawing a line to <var>innerQuad</var>\[<code>(<var>orienation</var>&#x20;+&#x20;1)&#x20;%&#x20;4</code>\].

    3.  Return.

2.  <a id="ref-for-clockwise-quad①"></a>

    Let <var>cornerQuad</var> be <var>cornerRect</var>’s [clockwise quad](#clockwise-quad).

3.  If <var>curvature</var> is -∞:

    1.  Extend <var>path</var> by drawing a line from <var>cornerQuad</var>\[0\] to <var>cornerQuad</var>\[3\], trimmed by <var>trimRect</var>.

    2.  Extend <var>path</var> by drawing a line from <var>cornerQuad</var>\[3\] to <var>cornerQuad</var>\[2\], trimmed by <var>trimRect</var>.

    3.  Return.

4.  <a id="ref-for-normalized-superellipse-half-corner"></a>

    Let <var>clampedNormalizedHalfCorner</var> be the [normalized superellipse half corner](#normalized-superellipse-half-corner) given <code>clamp(<var>curvature</var>,&#x20;-1,&#x20;1)</code>.

5.  Let <var>equivalentQuadraticControlPointX</var> be <code><var>clampedNormalizedHalfCorner</var>&#x20;&#x2A;&#x20;2&#x20;-&#x20;0.5</code>.

6.  <a id="ref-for-aligned-corner-point"></a>

    Let <var>curveStartPoint</var> be the [aligned corner point](#aligned-corner-point) given <var>cornerQuad</var>\[<var>orienation</var>\], the vector (<var>equivalentQuadraticControlPointX</var>, <code>1&#x20;-&#x20;<var>equivalentQuadraticControlPointX</var></code>), <var>startThickness</var>, and <var>orientation</var> + 1.

7.  <a id="ref-for-aligned-corner-point①"></a>

    Let <var>curveEndPoint</var> by the [aligned corner point](#aligned-corner-point) given <var>cornerQuad</var>\[(<var>orientation</var> + 2) % 4\], the vector (<code><var>equivalentQuadraticControlPointX</var>&#x20;-&#x20;1</code>, <code>-<var>equivalentQuadraticControlPointX</var></code>), <var>endThickness</var>, and <var>orientation</var> + 3.

8.  <a id="ref-for-rectangle④"></a>

    Let <var>alignedCornerRect</var> be a [rectangle](https://www.w3.org/TR/geometry-1/#rectangle) that includes the points <var>curveStartPoint</var> and <var>curveEndPoint</var>.

9.  <a id="ref-for-transformation-matrix"></a>

    <a id="ref-for-rectangle-x-coordinate"></a>

    <a id="ref-for-rectangle-y-coordinate"></a>

    <a id="ref-for-rectangle-width-dimension"></a>

    <a id="ref-for-rectangle-height-dimension"></a>

    Let <var>projectionToCornerRect</var> be a [transformation matrix](https://www.w3.org/TR/css-transforms-1/#transformation-matrix), translated by <code>(<var>alignedCornerRect</var>’s&#x20;<a href="https://www.w3.org/TR/geometry-1/#rectangle-x-coordinate">x&#x20;coordinate</a>,&#x20;<var>alignedCornerRect</var>’s&#x20;<a href="https://www.w3.org/TR/geometry-1/#rectangle-y-coordinate">y&#x20;coordinate</a>)</code>, scaled by <code>(<var>alignedCornerRect</var>’s&#x20;<a href="https://www.w3.org/TR/geometry-1/#rectangle-width-dimension">width&#x20;dimension</a>,&#x20;<var>alignedCornerRect</var>’s&#x20;<a href="https://www.w3.org/TR/geometry-1/#rectangle-height-dimension">height&#x20;dimension</a>)</code>, translated by `(0.5, 0.5)`, rotated by `90deg * orientation`, and translated by `(-0.5, -0.5)`.

10. Let <var>K</var> be <code>0.5<sup>abs(<var>curvature</var>)</sup></code>.

11. For each <var>T</var> between 0 and 1:

    1.  Let <var>A</var> be <code><var>T</var><sup><var>K</var></sup></code>.

    2.  Let <var>B</var> be <code>1&#x20;-&#x20;(1&#x20;-&#x20;<var>T</var>)<sup><var>K</var></sup></code>.

    3.  Let <var>normalizedPoint</var> be <code>(<var>A</var>,&#x20;<var>B</var>)</code> if <var>curvature</var> is positive, otherwise <code>(<var>B</var>,&#x20;<var>A</var>)</code>.

    4.  Let <var>absolutePoint</var> be <var>normalizedPoint</var>, transformed by <var>projectionToCornerRect</var>.

    5.  If <var>absolutePoint</var> is within <var>trimRect</var>, extend <var>path</var> through <var>absolutePoint</var>.

    > <strong data-conversion-semantic="note">Note</strong>
    >
    > Note: User agents may approximate this algorithm, for instance, by using concatenated Bezier curves, to balance between performance and rendering accuracy.

To compute the <a id="aligned-corner-point"></a>aligned corner point given a point <var>originalPoint</var>, a two-component vector <var>offsetFromControlPoint</var>, a number <var>thickness</var>, and a number <var>orientation</var>:

1.  Let <var>length</var> be <code>hypot(<var>offsetFromControlPoint</var>.x,&#x20;<var>offsetFromControlPoint</var>.y)</code>.

2.  Rotate <var>offsetFromControlPoint</var> by <code>90deg&#x20;&#x2A;&#x20;<var>orientation</var></code>, and scale by <var>thickness</var>.

3.  Translate <var>originalPoint</var> by <code><var>offsetFromControlPoint</var>.x&#x20;/&#x20;<var>length</var>,&#x20;<var>offsetFromControlPoint</var>.y&#x20;/&#x20;<var>length</var></code>, and return the result.

<a id="ref-for-rectangle⑤"></a>

<a id="ref-for-quadrilateral"></a>

<a id="ref-for-rectangle-x-coordinate①"></a>

<a id="ref-for-rectangle-y-coordinate①"></a>

<a id="ref-for-rectangle-width-dimension①"></a>

<a id="ref-for-rectangle-height-dimension①"></a>

The <a id="clockwise-quad"></a>clockwise quad given a [rectangle](https://www.w3.org/TR/geometry-1/#rectangle) <var>rect</var>, is a [quadrilateral](https://www.w3.org/TR/geometry-1/#quadrilateral) with the points (<var>rect</var>’s [x coordinate](https://www.w3.org/TR/geometry-1/#rectangle-x-coordinate), <var>rect</var>’s [y coordinate](https://www.w3.org/TR/geometry-1/#rectangle-y-coordinate)), (<var>rect</var>’s <a id="ref-for-rectangle-x-coordinate②"></a>x coordinate + <var>rect</var>’s [width dimension](https://www.w3.org/TR/geometry-1/#rectangle-width-dimension), <var>rect</var>’s <a id="ref-for-rectangle-y-coordinate②"></a>y coordinate), (<var>rect</var>’s <a id="ref-for-rectangle-x-coordinate③"></a>x coordinate + <var>rect</var>’s <a id="ref-for-rectangle-width-dimension②"></a>width dimension, <var>rect</var>’s <a id="ref-for-rectangle-y-coordinate③"></a>y coordinate + <var>rect</var>’s [height dimension](https://www.w3.org/TR/geometry-1/#rectangle-height-dimension)), (<var>rect</var>’s <a id="ref-for-rectangle-x-coordinate④"></a>x coordinate, <var>rect</var>’s <a id="ref-for-rectangle-y-coordinate④"></a>y coordinate + <var>rect</var>’s <a id="ref-for-rectangle-height-dimension②"></a>height dimension).

Tests

- [corners-computed.html](https://wpt.fyi/results/css/css-borders/corner-shape/corners-computed.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corners-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corners-computed.html)
- [corners-invalid.html](https://wpt.fyi/results/css/css-borders/corner-shape/corners-invalid.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corners-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corners-invalid.html)
- [corners-valid.html](https://wpt.fyi/results/css/css-borders/corner-shape/corners-valid.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corners-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corners-valid.html)

#### <a id="corner-shape-constrain-radii"></a>3.9.5.  Constraining opposite radii

<a id="ref-for-propdef-corner-shape①②"></a>

<a id="ref-for-superellipse-parameter"></a>

When concave [corner-shape](#propdef-corner-shape) values are present (the [superellipse parameter](#superellipse-parameter) is negative), diagonally opposite corners might overlap each other.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f2f82722"></a>
>
> The following example would create overlapping corners if not constrained.
>
> ```css
> div {
>   corner-shape: scoop;
>   border-top-left-radius: 80%;
>   border-bottom-right-radius: 80%;
> }
> ```
To prevent this, the four radii are constrained to prevent overlaps. This is done by computing a hull polygon for each of the opposite corners, and finding the highest downscale factor which, if applied to both corners, would make it so that the polygons would not intersect.

<a id="ref-for-concept-element⑧"></a>

To compute the <a id="opposite-corner-scale-factor"></a>opposite corner scale factor given an [element](https://dom.spec.whatwg.org/#concept-element) <var>element</var>:

1.  <a id="ref-for-border-box⑦"></a>

    Let <var>rect</var> be <var>element</var>’s [border box](https://www.w3.org/TR/css-box-4/#border-box).

2.  <a id="ref-for-normalized-inner-corner-hull"></a>

    <a id="ref-for-computed-value①"></a>

    <a id="ref-for-propdef-corner-top-right-shape"></a>

    <a id="ref-for-rectangle-width-dimension③"></a>

    <a id="ref-for-propdef-border-top-right-radius①"></a>

    Let <var>topRightHull</var> be a the [normalized inner corner hull](#normalized-inner-corner-hull) given <var>element</var>’s [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [corner-top-right-shape](#propdef-corner-top-right-shape), mapped to the rectangle (<var>rect</var>’s [width dimension](https://www.w3.org/TR/geometry-1/#rectangle-width-dimension) - <var>element</var>’s <a id="ref-for-computed-value②"></a>computed horizontal [border-top-right-radius](#propdef-border-top-right-radius), 0, <var>rect</var>’s <a id="ref-for-computed-value③"></a>computed <a id="ref-for-propdef-border-top-right-radius②"></a>border-top-right-radius).

3.  <a id="ref-for-normalized-inner-corner-hull①"></a>

    <a id="ref-for-computed-value④"></a>

    <a id="ref-for-propdef-corner-bottom-right-shape"></a>

    <a id="ref-for-rectangle-width-dimension④"></a>

    <a id="ref-for-propdef-border-bottom-right-radius①"></a>

    <a id="ref-for-rectangle-height-dimension③"></a>

    Let <var>bottomRightHull</var> be a the [normalized inner corner hull](#normalized-inner-corner-hull) given <var>element</var>’s [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [corner-bottom-right-shape](#propdef-corner-bottom-right-shape), rotated by 90deg with (0.5, 0.5) as an origin, and mapped to the rectangle (<var>rect</var>’s [width dimension](https://www.w3.org/TR/geometry-1/#rectangle-width-dimension) - <var>element</var>’s <a id="ref-for-computed-value⑤"></a>computed horizontal [border-bottom-right-radius](#propdef-border-bottom-right-radius), <var>rect</var>’s [height dimension](https://www.w3.org/TR/geometry-1/#rectangle-height-dimension) - <var>element</var>’s <a id="ref-for-computed-value⑥"></a>computed vertical <a id="ref-for-propdef-border-bottom-right-radius②"></a>border-bottom-right-radius, <var>element</var>’s <a id="ref-for-computed-value⑦"></a>computed <a id="ref-for-propdef-border-bottom-right-radius③"></a>border-bottom-right-radius).

4.  <a id="ref-for-normalized-inner-corner-hull②"></a>

    <a id="ref-for-computed-value⑧"></a>

    <a id="ref-for-propdef-corner-bottom-right-shape①"></a>

    <a id="ref-for-rectangle-height-dimension④"></a>

    <a id="ref-for-propdef-border-bottom-left-radius①"></a>

    Let <var>bottomLeftHull</var> be a the [normalized inner corner hull](#normalized-inner-corner-hull) given <var>element</var>’s [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [corner-bottom-right-shape](#propdef-corner-bottom-right-shape), rotated by 180deg with (0.5, 0.5) as an origin, and mapped to the rectangle (0, <var>rect</var>’s [height dimension](https://www.w3.org/TR/geometry-1/#rectangle-height-dimension) - <var>element</var>’s <a id="ref-for-computed-value⑨"></a>computed vertical [border-bottom-left-radius](#propdef-border-bottom-left-radius), <var>element</var>’s <a id="ref-for-computed-value①⓪"></a>computed <a id="ref-for-propdef-border-bottom-left-radius②"></a>border-bottom-left-radius).

5.  <a id="ref-for-normalized-inner-corner-hull③"></a>

    <a id="ref-for-computed-value①①"></a>

    <a id="ref-for-propdef-corner-top-left-shape⑤"></a>

    <a id="ref-for-propdef-border-top-left-radius③"></a>

    Let <var>topLeftHull</var> be a the [normalized inner corner hull](#normalized-inner-corner-hull) given <var>element</var>’s [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [corner-top-left-shape](#propdef-corner-top-left-shape), rotated by 270deg with (0.5, 0.5) as an origin, mapped to (0, 0, <var>element</var>’s <a id="ref-for-computed-value①②"></a>computed [border-top-left-radius](#propdef-border-top-left-radius)).

6.  Let <var>scaleFactorA</var> be the highest number which, if both <var>topLeftHull</var> and <var>bottomRightHull</var> were scaled by, using their first point as the origin, those polygons would not intersect.

7.  Let <var>scaleFactorB</var> be the highest number which, if both <var>topRightHull</var> and <var>bottomLeftHull</var> were scaled by, using their first point as the origin, those polygons would not intersect.

8.  Return <code>min(1,&#x20;<var>scaleFactorA</var>,&#x20;<var>scaleFactorB</var>)</code>.

#### <a id="corner-shape-interpolation"></a>3.9.6.  Interpolating corner shapes

<a id="ref-for-typedef-corner-shape-value③"></a>

<a id="ref-for-funcdef-superellipse⑤"></a>

<a id="ref-for-superellipse-parameter①"></a>

Since a [\<corner-shape-value\>](#typedef-corner-shape-value) can always be expressed by a [superellipse()](#funcdef-superellipse) with an [superellipse parameter](#superellipse-parameter) variable, interpolating between two <a id="ref-for-typedef-corner-shape-value④"></a>\<corner-shape-value\>s is done by interpolating the <a id="ref-for-superellipse-parameter②"></a>superellipse parameter itself. Since it uses a `log2`, interpolating it linearly would result in an effect where concave corners interpolate at a much higher velocity than convex corners. To balance that, the <a id="superellipse-interpolation"></a>superellipse interpolation formula describes how a <a id="ref-for-superellipse-parameter③"></a>superellipse parameter is converted to a value between 0 and 1, and vice versa:

<a id="ref-for-superellipse-parameter④"></a>

To compute the <a id="normalized-superellipse-half-corner"></a>normalized superellipse half corner given a [superellipse parameter](#superellipse-parameter) <var>s</var>, return the first matching statement, switching on <var>s</var>:

-∞  
Return 0.

∞  
Return 1.

Otherwise  
1.  Let <var>k</var> be <code>0.5<sup>abs(<var>s</var>)</sup></code>.

2.  Let <var>convexHalfCorner</var> be <code>0.5<sup><var>k</var></sup></code>.

3.  If <var>s</var> is less than 0, return <code>1&#x20;-&#x20;<var>convexHalfCorner</var></code>.

4.  Return <var>convexHalfCorner</var>.

<a id="ref-for-superellipse-parameter⑤"></a>

To compute the <a id="normalized-inner-corner-hull"></a>normalized inner corner hull given a [superellipse parameter](#superellipse-parameter) <var>curvature</var>:

1.  If <var>curvature</var> is greater than or equal to zero, return a triangle betwen « (1, 1), (1, 0), (0, 1) ».

2.  Let <var>axisLineA</var> be a line between `(1, 0)` and `(1, 1)`.

3.  Let <var>axisLineB</var> be a line between `(0, 1)` and `(1, 1)`.

4.  <a id="ref-for-normalized-superellipse-half-corner①"></a>

    Let <var>normalizedHalfCorner</var> be the [normalized superellipse half corner](#normalized-superellipse-half-corner) given <var>curvature</var>.

5.  Let <var>halfCornerPoint</var> be <code>(<var>normalizedHalfCorner</var>,&#x20;1&#x20;-&#x20;<var>normalizedHalfCorner</var>)</code>.

6.  Let <var>lineFromCenterToHalfCorner</var> be a line between `(0, 0)` and <var>halfCornerPoint</var>.

7.  Let <var>tangentLine</var> be the line perpendicular to <var>lineFromCenterToHalfCorner</var>, at <var>halfCornerPoint</var>.

8.  Let <var>intersectionA</var> be the intersection between <var>axisLineA</var> and <var>tangentLine</var>.

9.  Let <var>intersectionB</var> be the intersection between <var>axisLineB</var> and <var>tangentLine</var>.

10. Return a pentagon between the points « (1, 1), (1, 0), <var>intersectionA</var>, <var>intersectionB</var>, (0, 1), (1, 1) ».

<a id="ref-for-superellipse-parameter⑥"></a>

<a id="ref-for-normalized-superellipse-half-corner②"></a>

To interpolate a [superellipse parameter](#superellipse-parameter) <var>s</var> to an interpolation value between 0 and 1, return the [normalized superellipse half corner](#normalized-superellipse-half-corner) given <var>s</var>.

<a id="ref-for-number-value①"></a>

<a id="ref-for-superellipse-parameter⑦"></a>

To convert a [\<number \[0,1\]\>](https://www.w3.org/TR/css-values-4/#number-value) <var>interpolationValue</var> back to a [superellipse parameter](#superellipse-parameter), switch on <var>interpolationValue</var>:

0  
Return -∞.

0.5  
Return 0.

1  
Return ∞.

Otherwise  
1.  Let <var>convexHalfCorner</var> be <var>interpolationValue</var>.

2.  If <var>interpolationValue</var> is less than 0.5, set <var>convexHalfCorner</var> to 1 - <var>interpolationValue</var>.

3.  Let <var>k</var> be <code>ln(0.5)&#x20;/&#x20;ln(<var>convexHalfCorner</var>)</code>.

4.  Let <var>s</var> be <code>log2(<var>k</var>)</code>.

5.  If <var>interpolationValue</var> is less than 0.5, return -<var>s</var>.

6.  Return <var>s</var>.

Tests

- [corner-shape-interpolation.html](https://wpt.fyi/results/css/css-borders/corner-shape/corner-shape-interpolation.html) [(live test)](http://wpt.live/css/css-borders/corner-shape/corner-shape-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/corner-shape/corner-shape-interpolation.html)

## <a id="border-images"></a>4.  Border Images

<a id="ref-for-propdef-border-image-source"></a>

<a id="ref-for-border-image-area"></a>

<a id="ref-for-propdef-border-width⑧"></a>

<a id="ref-for-propdef-border-style⑧"></a>

Authors can specify an image to be used in place of the border styles. In this case, the border’s design is taken from the sides and corners of an image specified with [border-image-source](#propdef-border-image-source), whose pieces may be sliced, scaled, and stretched in various ways to fit the size of the [border image area](#border-image-area). The border-image properties do not affect layout: layout of the box, its content, and surrounding content is based on the [border-width](#propdef-border-width) and [border-style](#propdef-border-style) properties only.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8488cbd9"></a> This example creates a top and bottom border consisting of a whole number of orange diamonds and a left and right border of a single, stretched diamond. The corners are diamonds of a different color. The image to tile is as follows. Apart from the diamonds, it is transparent:
>
> ![Tile for border](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/border.png)
>
> The image is 81 by 81 pixels and has to be divided into 9 equal parts. The style rules could thus be as follows:
>
> ```text
> DIV {
>   border: double orange 1em;
>   border-image: url("border.png") 27 round stretch;
>  }
> ```
>
> The result, when applied to a DIV of 12 by 5em, will be similar to this:
>
> ![Element with a diamond border](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/borderresult.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-28370333"></a> This shows a more complicated example, demonstrating how the border image corresponds to the fallback border-style but can also extend beyond the border area. The border image is a wavy green border with an extended corner effect:
>
> ![Diagram: The border image shows a wavy green border with more exaggerated waves towards the corners, which are capped by a disconnected green circle. Four cuts at 124px offsets from each side divide the image into 124px-wide square corners, 124px-wide but thin side slices, and a small center square.](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/groovy-border-image-slice.png)
>
> <a id="ref-for-propdef-border-image-source①"></a>
>
> <a id="ref-for-propdef-border-image-slice"></a>
>
> The [border-image-source](#propdef-border-image-source) image, with the four [border-image-slice](#propdef-border-image-slice) cuts at 124px dividing the image into nine parts.
>
> The rest of the border properties then interact to lay out the tiles as follows:
>
> ![Diagram: The image-less (fallback) rendering has a green double border. The rendering with border-image shows the wavy green border, ith the waves getting longer as they reach the corners. The corner tiles render as 124px-wide squares and the side tiles repeat a whole number of times to fill the space in between. Because of the gradual corner effects, the tiles extend deep into the padding area. The whole border image effect is outset 31px, so that the troughs of the waves align just outside the padding edge.](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/border-image.png)
>
> Diagram of all border-image properties and how they interact, and showing the rendering with and without the border-image in effect.
>
> <a id="ref-for-propdef-border-width⑨"></a>
>
> <a id="ref-for-propdef-border-image-width①"></a>
>
> <a id="ref-for-border-image-area①"></a>
>
> <a id="ref-for-border-box⑧"></a>
>
> <a id="ref-for-margin-area"></a>
>
> Here, even though the [border-width](#propdef-border-width) is 12px, the [border-image-width](#propdef-border-image-width) property computes to 124px. The [border image area](#border-image-area) is then outset 31px from the [border box](https://www.w3.org/TR/css-box-4/#border-box) and into the [margin area](https://www.w3.org/TR/css-box-4/#margin-area). If the border-image fails to load (or if border images are not supported by the UA), the fallback rendering uses a green double border.

<a id="ref-for-propdef-border⑦"></a>

<a id="ref-for-propdef-border-image④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="border-image-border-shorthand-example"></a> Notice that the [border](#propdef-border) shorthand resets [border-image](#propdef-border-image). This makes it easy to turn off or reset all border effects:
>
> ```text
> .notebox {
>   border: double orange;
>   /* must set 'border' shorthand first, otherwise it erases 'border-image' */
>   border-image: url("border.png") 30 round;
>   /* but other 'border' properties can be set after */
>   border-width: thin thick;
> }
> ...
> .sidebar .notebox {
>   box-shadow: 0 0 5px gray;
>   border-radius: 5px;
>   border: none; /* turn off all borders */
>   /* 'border' shorthand resets 'border-image' */
> }
> ```
<a id="ref-for-propdef-border-image-source②"></a>

### <a id="border-image-source"></a>4.1. <a id="the-border-image-source"></a> Image Source: the [border-image-source](#propdef-border-image-source) property



| Field               | Definition                                                                                                                                                         |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-image-source"></a>border-image-source                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-image"></a><a id="ref-for-comb-one②③"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image)       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-collapse②"></a>All elements, except internal table elements when [border-collapse](https://www.w3.org/TR/CSS2/tables.html#propdef-border-collapse) is collapse |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-image①"></a>the keyword none or the computed [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image)                                                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                           |



Tests

- [border-image-source-interpolation.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-image-source-interpolation.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-image-source-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-image-source-interpolation.html)
- [css3-border-image-source.html](https://wpt.fyi/results/css/css-backgrounds/css3-border-image-source.html) [(live test)](http://wpt.live/css/css-backgrounds/css3-border-image-source.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/css3-border-image-source.html)
- [border-image-source-computed.sub.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-source-computed.sub.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-source-computed.sub.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-source-computed.sub.html)
- [border-image-source-invalid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-source-invalid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-source-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-source-invalid.html)
- [border-image-source-valid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-source-valid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-source-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-source-valid.html)

<a id="ref-for-propdef-border-style⑨"></a>

<a id="ref-for-border-image-slice-fill"></a>

<a id="ref-for-propdef-border-image-slice①"></a>

Specifies an image to use as a border in place of the rendering specified by the [border-style](#propdef-border-style) properties and, if given the [fill](#border-image-slice-fill) keyword in [border-image-slice](#propdef-border-image-slice), as an additional image backdrop for the element. If the value is none or if the image cannot be displayed (or the property doesn’t apply), the border styles will be used; otherwise the element’s <a id="ref-for-propdef-border-style①⓪"></a>border-style borders are not drawn and this <a id="border-image-dfn"></a>border image is drawn as described in the sections below.

<a id="ref-for-propdef-border-image-slice②"></a>

### <a id="border-image-slice"></a>4.2. <a id="the-border-image-slice"></a> Image Slicing: the [border-image-slice](#propdef-border-image-slice) property



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-image-slice"></a>border-image-slice                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt③"></a><a id="ref-for-comb-all"></a><a id="ref-for-mult-num-range①③"></a><a id="ref-for-percentage-value"></a><a id="ref-for-comb-one②④"></a><a id="ref-for-number-value②"></a>\[[\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#percentage-value)\][{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) fill[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 100%                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-collapse③"></a>All elements, except internal table elements when [border-collapse](https://www.w3.org/TR/CSS2/tables.html#propdef-border-collapse) is collapse                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | refer to size of the border image                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-border-image-slice-fill①"></a>four values, each either a number or percentage; plus a [fill](#border-image-slice-fill) keyword if specified                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |



Tests

- [border-image-slice-composition.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-image-slice-composition.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-image-slice-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-image-slice-composition.html)
- [border-image-slice-interpolation-stability.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-image-slice-interpolation-stability.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-image-slice-interpolation-stability.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-image-slice-interpolation-stability.html)
- [border-image-slice-interpolation.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-image-slice-interpolation.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-image-slice-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-image-slice-interpolation.html)
- [border-image-slice-001.xht](https://wpt.fyi/results/css/css-backgrounds/border-image-slice-001.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-image-slice-001.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-slice-001.xht)
- [border-image-slice-002.xht](https://wpt.fyi/results/css/css-backgrounds/border-image-slice-002.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-image-slice-002.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-slice-002.xht)
- [border-image-slice-003.xht](https://wpt.fyi/results/css/css-backgrounds/border-image-slice-003.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-image-slice-003.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-slice-003.xht)
- [border-image-slice-004.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-slice-004.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-slice-004.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-slice-004.htm)
- [border-image-slice-005.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-slice-005.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-slice-005.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-slice-005.htm)
- [border-image-slice-006.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-slice-006.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-slice-006.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-slice-006.htm)
- [border-image-slice-007.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-slice-007.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-slice-007.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-slice-007.htm)
- [border-image-slice-fill-001.html](https://wpt.fyi/results/css/css-backgrounds/border-image-slice-fill-001.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-slice-fill-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-slice-fill-001.html)
- [border-image-slice-fill-002.html](https://wpt.fyi/results/css/css-backgrounds/border-image-slice-fill-002.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-slice-fill-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-slice-fill-002.html)
- [border-image-slice-fill-003.html](https://wpt.fyi/results/css/css-backgrounds/border-image-slice-fill-003.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-slice-fill-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-slice-fill-003.html)
- [border-image-slice-percentage.html](https://wpt.fyi/results/css/css-backgrounds/border-image-slice-percentage.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-slice-percentage.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-slice-percentage.html)
- [border-image-slice-shorthand-reset.html](https://wpt.fyi/results/css/css-backgrounds/border-image-slice-shorthand-reset.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-slice-shorthand-reset.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-slice-shorthand-reset.html)
- [border-image-slice-computed.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-slice-computed.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-slice-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-slice-computed.html)
- [border-image-slice-invalid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-slice-invalid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-slice-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-slice-invalid.html)
- [border-image-slice-valid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-slice-valid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-slice-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-slice-valid.html)

<a id="ref-for-border-image-slice-fill②"></a>

This property specifies inward offsets from the top, right, bottom, and left edges of the image, dividing it into nine regions: four corners, four edges and a middle. The middle image part is discarded (treated as fully transparent) unless the [fill](#border-image-slice-fill) keyword is present. (It is drawn over the background; see [Drawing the Border Image](#border-image-process).)

If there is only one component value, it applies to all sides. If there are two values, the top and bottom are set to the first value and the right and left are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values they apply to the top, right, bottom, and left, respectively.

<a id="ref-for-percentage-value①"></a>

<a id="valdef-border-image-slice-percentage-0"></a>[\<percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#percentage-value)

Percentages are relative to the size of the image: the width of the image for the horizontal offsets, the height for vertical offsets.

<a id="ref-for-number-value③"></a>

<a id="valdef-border-image-slice-number-0"></a>[\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value)

Numbers represent pixels in the image (if the image is a raster image) or vector coordinates (if the image is a vector image).

<a id="border-image-slice-fill"></a>fill

<a id="ref-for-border-image-slice-fill③"></a>

The [fill](#border-image-slice-fill) keyword, if present, causes the middle part of the border-image to be preserved. (By default it is discarded, i.e., treated as empty.)

Negative values are invalid. Computed values larger than the size of the image are interpreted as 100%.

<a id="ref-for-propdef-border-image-slice③"></a>

The regions given by the [border-image-slice](#propdef-border-image-slice) values may overlap. However if the sum of the right and left widths is equal to or greater than the width of the image, the images for the top and bottom edge and the middle part are empty—​which has the same effect as if a nonempty transparent image had been specified for those parts. Analogously for the top and bottom values.

<a id="ref-for-natural-dimensions"></a>

<a id="ref-for-specified-size"></a>

<a id="ref-for-border-image-area②"></a>

<a id="ref-for-default-object-size"></a>

If the image must be sized to determine the slices (for example, for SVG images with no [natural dimensions](https://www.w3.org/TR/css-images-3/#natural-dimensions)), then it is sized using the [default sizing algorithm](https://www.w3.org/TR/css3-images/#default-sizing) with no [specified size](https://www.w3.org/TR/css-images-3/#specified-size) and the [border image area](#border-image-area) as the [default object size](https://www.w3.org/TR/css-images-3/#default-object-size).

![Diagram: two horizontal cuts and two vertical cuts through an image](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/slice.png)

Diagram illustrating the cuts corresponding to the value 25% 30% 12% 20%

<a id="ref-for-propdef-border-image-width②"></a>

### <a id="border-image-width"></a>4.3. <a id="the-border-image-width"></a> Drawing Areas: the [border-image-width](#propdef-border-image-width) property



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                       |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-image-width"></a>border-image-width                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range①④"></a><a id="ref-for-number-value④"></a><a id="ref-for-comb-one②⑤"></a><a id="ref-for-typedef-length-percentage⑨"></a>\[ [\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) <a id="ref-for-comb-one②⑥"></a>\| auto \][{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 1                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-collapse④"></a>All elements, except internal table elements when [border-collapse](https://www.w3.org/TR/CSS2/tables.html#propdef-border-collapse) is collapse                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-border-image-area③"></a>Relative to width/height of the [border image area](#border-image-area)                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①⓪"></a><a id="ref-for-valdef-border-image-width-auto"></a>four values, each either a number, the keyword [auto](#valdef-border-image-width-auto), or a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                                                                                                                                                |



Tests

- [border-image-width-001.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-width-001.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-width-001.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-width-001.htm)
- [border-image-width-005.xht](https://wpt.fyi/results/css/css-backgrounds/border-image-width-005.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-image-width-005.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-width-005.xht)
- [border-image-width-006.xht](https://wpt.fyi/results/css/css-backgrounds/border-image-width-006.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-image-width-006.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-width-006.xht)
- [border-image-width-007.xht](https://wpt.fyi/results/css/css-backgrounds/border-image-width-007.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-image-width-007.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-width-007.xht)
- [border-image-width-008.html](https://wpt.fyi/results/css/css-backgrounds/border-image-width-008.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-width-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-width-008.html)
- [border-image-width-009.html](https://wpt.fyi/results/css/css-backgrounds/border-image-width-009.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-width-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-width-009.html)
- [border-image-width-should-extend-to-padding.html](https://wpt.fyi/results/css/css-backgrounds/border-image-width-should-extend-to-padding.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-width-should-extend-to-padding.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-width-should-extend-to-padding.html)
- [border-image-width-computed.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-width-computed.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-width-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-width-computed.html)
- [border-image-width-invalid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-width-invalid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-width-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-width-invalid.html)
- [border-image-width-valid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-width-valid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-width-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-width-valid.html)

<a id="ref-for-border-image-dfn"></a>

<a id="ref-for-border-box⑨"></a>

<a id="ref-for-propdef-border-image-outset"></a>

The [border image](#border-image-dfn) is drawn inside an area called the <a id="border-image-area"></a>border image area. This is an area whose boundaries by default correspond to the [border box](https://www.w3.org/TR/css-box-4/#border-box), see [border-image-outset](#propdef-border-image-outset).

<a id="ref-for-propdef-border-image-width③"></a>

<a id="ref-for-border-image-area④"></a>

The four values of [border-image-width](#propdef-border-image-width) specify offsets that are used to divide the [border image area](#border-image-area) into nine <a id="border-image-region"></a>regions. The offsets represent inward distances from the top, right, bottom, and left sides of the area, respectively.

If there is only one component value, it applies to all sides. If there are two values, the top and bottom are set to the first value and the right and left are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values they apply to the top, right, bottom, and left, respectively.

Values have the following meanings:

<a id="ref-for-typedef-length-percentage①①"></a>

<a id="valdef-border-image-width-length-percentage-0"></a>[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)

<a id="ref-for-border-image-area⑤"></a>

Percentages refer to the size of the [border image area](#border-image-area): the width of the area for horizontal offsets, the height for vertical offsets.

<a id="ref-for-number-value⑤"></a>

<a id="valdef-border-image-width-number-0"></a>[\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value)

Numbers represent multiples of the corresponding computed [border-width](#border-width).

<a id="valdef-border-image-width-auto"></a>auto

<a id="ref-for-propdef-border-width①⓪"></a>

<a id="ref-for-natural-dimensions①"></a>

<a id="ref-for-propdef-border-image-slice④"></a>

<a id="ref-for-natural-size"></a>

<a id="ref-for-propdef-border-image-width④"></a>

<a id="ref-for-valdef-border-image-width-auto①"></a>

If [auto](#valdef-border-image-width-auto) is specified then the used [border-image-width](#propdef-border-image-width) is the [natural](https://www.w3.org/TR/css-images-3/#natural-size) width or height (whichever is applicable) of the corresponding image slice (see [border-image-slice](#propdef-border-image-slice)). If the image does not have the required [natural dimension](https://www.w3.org/TR/css-images-3/#natural-dimensions) then the corresponding computed [border-width](#propdef-border-width) is used instead.

<a id="ref-for-propdef-border-image-width⑤"></a>

Negative values are invalid for any [border-image-width](#propdef-border-image-width) values.

<a id="ref-for-propdef-border-image-width⑥"></a>

<a id="ref-for-used-value④"></a>

<a id="ref-for-border-image-area⑥"></a>

If two opposite [border-image-width](#propdef-border-image-width) offsets are large enough that they overlap, then the [used values](https://www.w3.org/TR/css-cascade-5/#used-value) of all <a id="ref-for-propdef-border-image-width⑦"></a>border-image-width offsets are proportionally reduced until they no longer overlap. In mathematical notation: Given <var>L<sub>width</sub></var> as the width of the [border image area](#border-image-area), <var>L<sub>height</sub></var> as its height, and <var>W<sub><var>side</var></sub></var> as the <a id="ref-for-propdef-border-image-width⑧"></a>border-image-width offset for the <var>side</var> side, let <var>f</var> = min(<var>L<sub>width</sub></var>/(<var>W<sub>left</sub></var>+<var>W<sub>right</sub></var>), <var>L<sub>height</sub></var>/(<var>W<sub>top</sub></var>+<var>W<sub>bottom</sub></var>)). If <var>f</var> \< 1, then all <var>W</var> are reduced by multiplying them by <var>f</var>.

<a id="ref-for-propdef-border-image-outset①"></a>

### <a id="border-image-outset"></a>4.4. <a id="the-border-image-outset"></a> Edge Overhang: the [border-image-outset](#propdef-border-image-outset) property



| Field               | Definition                                                                                                                                                                                                                                                                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-image-outset"></a>border-image-outset                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range①⑤"></a><a id="ref-for-number-value⑥"></a><a id="ref-for-comb-one②⑦"></a><a id="ref-for-length-value①"></a>\[ [\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) \][{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-collapse⑤"></a>All elements, except internal table elements when [border-collapse](https://www.w3.org/TR/CSS2/tables.html#propdef-border-collapse) is collapse                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | four values, each a number or absolute length                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                                                                                             |



Tests

- [border-image-outset-composition.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-image-outset-composition.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-image-outset-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-image-outset-composition.html)
- [border-image-outset-interpolation.html](https://wpt.fyi/results/css/css-backgrounds/animations/border-image-outset-interpolation.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/border-image-outset-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/border-image-outset-interpolation.html)
- [border-image-outset-003.html](https://wpt.fyi/results/css/css-backgrounds/border-image-outset-003.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-outset-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-outset-003.html)
- [border-image-outset-computed.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-outset-computed.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-outset-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-outset-computed.html)
- [border-image-outset-invalid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-outset-invalid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-outset-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-outset-invalid.html)
- [border-image-outset-valid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-outset-valid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-outset-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-outset-valid.html)

<a id="ref-for-border-image-area⑦"></a>

<a id="ref-for-border-box①⓪"></a>

The values specify the amount by which the [border image area](#border-image-area) extends beyond the [border box](https://www.w3.org/TR/css-box-4/#border-box).

If there is only one component value, it applies to all sides. If there are two values, the top and bottom are set to the first value and the right and left are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values they apply to the top, right, bottom, and left, respectively.

<a id="ref-for-length-value②"></a>

[\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value)

Represents an outset of the specified length.

<a id="ref-for-number-value⑦"></a>

[\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value)

Represents an outset of the specified multiple of the corresponding computed [border-width](#border-width).

Negative values are invalid.

<a id="ref-for-border-box①①"></a>

<a id="ref-for-ink-overflow"></a>

Portions of the border-image that are rendered outside the [border box](https://www.w3.org/TR/css-box-4/#border-box) are [ink overflow](https://www.w3.org/TR/css-overflow-3/#ink-overflow) and do not trigger scrolling. Also such portions are invisible to mouse events and do not capture such events on behalf of the element.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Even though they never cause a scrolling mechanism, outset images may still be clipped by an ancestor or by the viewport.

<a id="ref-for-propdef-border-image-repeat"></a>

### <a id="border-image-repeat"></a>4.5. <a id="the-border-image-repeat"></a> Image Tiling: the [border-image-repeat](#propdef-border-image-repeat) property



| Field               | Definition                                                                                                                                                                                                                       |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-image-repeat"></a>border-image-repeat                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range①⑥"></a><a id="ref-for-comb-one②⑧"></a>\[ stretch [\|](https://www.w3.org/TR/css-values-4/#comb-one) repeat <a id="ref-for-comb-one②⑨"></a>\| round <a id="ref-for-comb-one③⓪"></a>\| space \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | stretch                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-collapse⑥"></a>All elements, except internal table elements when [border-collapse](https://www.w3.org/TR/CSS2/tables.html#propdef-border-collapse) is collapse                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | two keywords, one per axis                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                         |



Tests

- [discrete-no-interpolation.html](https://wpt.fyi/results/css/css-backgrounds/animations/discrete-no-interpolation.html) [(live test)](http://wpt.live/css/css-backgrounds/animations/discrete-no-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/discrete-no-interpolation.html)
- [border-image-repeat-002.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-002.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-002.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-002.htm)
- [border-image-repeat-004.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-004.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-004.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-004.htm)
- [border-image-repeat-005.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-005.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-005.html)
- [border-image-repeat-1.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-1.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-1.html)
- [border-image-repeat-repeat-001.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-repeat-001.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-repeat-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-repeat-001.html)
- [border-image-repeat-round-003.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-round-003.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-round-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-round-003.html)
- [border-image-repeat-round-1.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-round-1.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-round-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-round-1.html)
- [border-image-repeat-round-2.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-round-2.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-round-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-round-2.html)
- [border-image-repeat-round-stretch-001.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-round-stretch-001.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-round-stretch-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-round-stretch-001.html)
- [border-image-repeat-round.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-round.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-round.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-round.html)
- [border-image-repeat-space-011.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-space-011.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-space-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-space-011.html)
- [border-image-repeat-space-1.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-space-1.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-space-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-space-1.html)
- [border-image-repeat-space-10.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-space-10.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-space-10.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-space-10.html)
- [border-image-repeat-space-2.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-space-2.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-space-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-space-2.html)
- [border-image-repeat-space-3.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-space-3.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-space-3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-space-3.html)
- [border-image-repeat-space-4-ref-1.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-space-4-ref-1.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-space-4-ref-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-space-4-ref-1.html)
- [border-image-repeat-space-4.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-space-4.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-space-4.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-space-4.html)
- [border-image-repeat-space-5-ref-1.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-space-5-ref-1.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-space-5-ref-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-space-5-ref-1.html)
- [border-image-repeat-space-5.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-space-5.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-space-5.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-space-5.html)
- [border-image-repeat-space-6.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-space-6.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-space-6.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-space-6.html)
- [border-image-repeat-space-7.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-space-7.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-space-7.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-space-7.html)
- [border-image-repeat-space-8.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-space-8.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-space-8.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-space-8.html)
- [border-image-repeat-space-9.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-space-9.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-space-9.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-space-9.html)
- [border-image-repeat-stretch-round-001.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat-stretch-round-001.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat-stretch-round-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat-stretch-round-001.html)
- [border-image-repeat_repeatnegx_none_50px.html](https://wpt.fyi/results/css/css-backgrounds/border-image-repeat_repeatnegx_none_50px.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-repeat_repeatnegx_none_50px.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-repeat_repeatnegx_none_50px.html)
- [css3-border-image-repeat-repeat.html](https://wpt.fyi/results/css/css-backgrounds/css3-border-image-repeat-repeat.html) [(live test)](http://wpt.live/css/css-backgrounds/css3-border-image-repeat-repeat.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/css3-border-image-repeat-repeat.html)
- [css3-border-image-repeat-stretch.html](https://wpt.fyi/results/css/css-backgrounds/css3-border-image-repeat-stretch.html) [(live test)](http://wpt.live/css/css-backgrounds/css3-border-image-repeat-stretch.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/css3-border-image-repeat-stretch.html)
- [border-image-repeat-computed.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-repeat-computed.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-repeat-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-repeat-computed.html)
- [border-image-repeat-invalid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-repeat-invalid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-repeat-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-repeat-invalid.html)
- [border-image-repeat-valid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-repeat-valid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-repeat-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-repeat-valid.html)

<a id="ref-for-border-image-dfn①"></a>

This property specifies how the images for the sides and the middle part of the [border image](#border-image-dfn) are scaled and tiled. The first keyword applies to the horizontal scaling and tiling of the top, middle and bottom parts, the second to the vertical scaling and tiling of the left, middle and right parts; see [Drawing the Border Image](#border-image-process). If the second keyword is absent, it is assumed to be the same as the first. Values have the following meanings:

<a id="valdef-border-image-repeat-stretch"></a>stretch  
<a id="ref-for-border-image-region"></a>

The image is stretched to fill its corresponding [region](#border-image-region).

<a id="valdef-border-image-repeat-repeat"></a>repeat  
<a id="ref-for-border-image-region①"></a>

The image is tiled (repeated) to fill its corresponding [region](#border-image-region).

<a id="valdef-border-image-repeat-round"></a>round  
<a id="ref-for-border-image-region②"></a>

The image is tiled (repeated) to fill its corresponding [region](#border-image-region). If it does not fill the area with a whole number of tiles, the image is rescaled so that it does.

<a id="valdef-border-image-repeat-space"></a>space  
<a id="ref-for-border-image-region③"></a>

The image is tiled (repeated) to fill its corresponding [region](#border-image-region). If it does not fill the region with a whole number of tiles, the extra space is distributed around the tiles.

The exact process for scaling and tiling the border-image parts is given in the section below.

### <a id="border-image-process"></a>4.6.  Drawing the Border Image

<a id="ref-for-border-image-dfn②"></a>

<a id="ref-for-propdef-border-image-source③"></a>

<a id="ref-for-propdef-border-image-slice⑤"></a>

<a id="ref-for-border-image-region④"></a>

After the [border image](#border-image-dfn) given by [border-image-source](#propdef-border-image-source) is sliced by the [border-image-slice](#propdef-border-image-slice) values, the resulting nine images are scaled, positioned, and tiled into their corresponding [border image regions](#border-image-region) in four steps:

1.  <a id="ref-for-propdef-border-image-width⑨"></a>

    Scale to [border-image-width](#propdef-border-image-width).

    - <a id="ref-for-border-image-region⑤"></a>

      The two images for the top and bottom edges are made as tall as the top and bottom [border image regions](#border-image-region), respectively, and their width is scaled proportionally.

    - <a id="ref-for-border-image-region⑥"></a>

      The images for the left and right edges are made as wide as the left and right [border image regions](#border-image-region), respectively, and their height is scaled proportionally.

    - <a id="ref-for-border-image-region⑦"></a>

      The corner images are scaled to be as wide and as tall as the their respective [border image regions](#border-image-region).

    - The middle image’s width is scaled by the same factor as the top image unless that factor is zero or infinity, in which case the scaling factor of the bottom is substituted, and failing that, the width is not scaled. The height of the middle image is scaled by the same factor as the left image unless that factor is zero or infinity, in which case the scaling factor of the right image is substituted, and failing that, the height is not scaled.

2.  <a id="ref-for-propdef-border-image-repeat①"></a>

    Scale to [border-image-repeat](#propdef-border-image-repeat).

    - <a id="ref-for-border-image-area⑧"></a>

      <a id="ref-for-valdef-border-image-repeat-stretch"></a>

      If the first keyword is [stretch](#valdef-border-image-repeat-stretch), the top, middle and bottom images are further scaled to be as wide as the middle region of the [border image area](#border-image-area). The height is not changed any further.

    - <a id="ref-for-propdef-background-repeat"></a>

      <a id="ref-for-valdef-background-repeat-round"></a>

      <a id="ref-for-border-image-area⑨"></a>

      <a id="ref-for-valdef-border-image-repeat-round"></a>

      If the first keyword is [round](#valdef-border-image-repeat-round), the top, middle and bottom images are resized in width, so that exactly a whole number of them fit in the middle region of the [border image area](#border-image-area), exactly as for [round](https://www.w3.org/TR/css-backgrounds-3/#valdef-background-repeat-round) in the [background-repeat](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-repeat) property.

    - <a id="ref-for-valdef-border-image-repeat-space"></a>

      <a id="ref-for-valdef-border-image-repeat-repeat"></a>

      If the first keyword is [repeat](#valdef-border-image-repeat-repeat) or [space](#valdef-border-image-repeat-space), the top, middle, and bottom images are not changed any further.

    - <a id="ref-for-valdef-border-image-repeat-space①"></a>

      <a id="ref-for-valdef-border-image-repeat-repeat①"></a>

      <a id="ref-for-valdef-border-image-repeat-round①"></a>

      <a id="ref-for-valdef-border-image-repeat-stretch①"></a>

      The effects of [stretch](#valdef-border-image-repeat-stretch), [round](#valdef-border-image-repeat-round), [repeat](#valdef-border-image-repeat-repeat), and [space](#valdef-border-image-repeat-space) for the second keyword are analogous, acting on the height of the left, middle and right images.

3.  Position the first tile.
    - <a id="ref-for-border-image-area①⓪"></a>

      <a id="ref-for-valdef-border-image-repeat-repeat②"></a>

      If the first keyword is [repeat](#valdef-border-image-repeat-repeat), the top, middle, and bottom images are centered horizontally in their respective regions. Otherwise the images are placed at the left edge of their respective regions of the [border image area](#border-image-area).

    - <a id="ref-for-border-image-area①①"></a>

      <a id="ref-for-valdef-border-image-repeat-repeat③"></a>

      If the second keyword is [repeat](#valdef-border-image-repeat-repeat), the left, middle, and right images are centered vertically in their respective regions. Otherwise the images are placed at the top edge of their respective regions of the [border image area](#border-image-area).

4.  Tile and draw.
    - The images are then tiled to fill their respective regions.

    - <a id="ref-for-valdef-border-image-repeat-space②"></a>

      In the case of [space](#valdef-border-image-repeat-space), any partial tiles are discarded and the extra space distributed before, after, and between the tiles. (I.e. the gap before the first tile, the gap after the last tile, and the gaps between tiles are equalized.) <strong data-conversion-semantic="note">Note:</strong> This can result in empty border-image side regions.

    - The images are drawn at the same stacking level as normal borders: immediately in front of the background layers.

    - <a id="ref-for-propdef-border-image-source④"></a>

      <a id="ref-for-border-image-slice-fill④"></a>

      The middle image is not drawn unless [fill](#border-image-slice-fill) was specified for [border-image-source](#propdef-border-image-source).

<a id="ref-for-propdef-border-image⑤"></a>

### <a id="border-image"></a>4.7. <a id="the-border-image"></a> Border Image Shorthand: the [border-image](#propdef-border-image) property



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-image"></a>border-image                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-image-repeat②"></a><a id="ref-for-propdef-border-image-outset②"></a><a id="ref-for-mult-opt④"></a><a id="ref-for-comb-one③①"></a><a id="ref-for-propdef-border-image-width①⓪"></a><a id="ref-for-propdef-border-image-slice⑥"></a><a id="ref-for-comb-any⑦"></a><a id="ref-for-propdef-border-image-source⑤"></a>[\<'border-image-source'\>](#propdef-border-image-source) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'border-image-slice'\>](#propdef-border-image-slice) \[ / [\<'border-image-width'\>](#propdef-border-image-width) [\|](https://www.w3.org/TR/css-values-4/#comb-one) / <a id="ref-for-propdef-border-image-width①①"></a>\<'border-image-width'\>[?](https://www.w3.org/TR/css-values-4/#mult-opt) / [\<'border-image-outset'\>](#propdef-border-image-outset) \]<a id="ref-for-mult-opt⑤"></a>? <a id="ref-for-comb-any⑧"></a>\|\| [\<'border-image-repeat'\>](#propdef-border-image-repeat) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |



Tests

- [border-image-002.html](https://wpt.fyi/results/css/css-backgrounds/border-image-002.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-002.html)
- [border-image-003.html](https://wpt.fyi/results/css/css-backgrounds/border-image-003.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-003.html)
- [border-image-004.html](https://wpt.fyi/results/css/css-backgrounds/border-image-004.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-004.html)
- [border-image-006.html](https://wpt.fyi/results/css/css-backgrounds/border-image-006.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-006.html)
- [border-image-007.html](https://wpt.fyi/results/css/css-backgrounds/border-image-007.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-007.html)
- [border-image-011.html](https://wpt.fyi/results/css/css-backgrounds/border-image-011.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-011.html)
- [border-image-012.html](https://wpt.fyi/results/css/css-backgrounds/border-image-012.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-012.html)
- [border-image-013.html](https://wpt.fyi/results/css/css-backgrounds/border-image-013.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-013.html)
- [border-image-017.xht](https://wpt.fyi/results/css/css-backgrounds/border-image-017.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-image-017.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-017.xht)
- [border-image-018.xht](https://wpt.fyi/results/css/css-backgrounds/border-image-018.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-image-018.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-018.xht)
- [border-image-019.xht](https://wpt.fyi/results/css/css-backgrounds/border-image-019.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-image-019.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-019.xht)
- [border-image-020.xht](https://wpt.fyi/results/css/css-backgrounds/border-image-020.xht) [(live test)](http://wpt.live/css/css-backgrounds/border-image-020.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-020.xht)
- [border-image-calc.html](https://wpt.fyi/results/css/css-backgrounds/border-image-calc.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-calc.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-calc.html)
- [border-image-image-type-001.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-image-type-001.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-image-type-001.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-image-type-001.htm)
- [border-image-image-type-002.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-image-type-002.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-image-type-002.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-image-type-002.htm)
- [border-image-image-type-003.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-image-type-003.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-image-type-003.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-image-type-003.htm)
- [border-image-image-type-004.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-image-type-004.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-image-type-004.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-image-type-004.htm)
- [border-image-image-type-005.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-image-type-005.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-image-type-005.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-image-type-005.htm)
- [border-image-round-and-stretch.html](https://wpt.fyi/results/css/css-backgrounds/border-image-round-and-stretch.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-round-and-stretch.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-round-and-stretch.html)
- [border-image-shorthand-001.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-shorthand-001.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-shorthand-001.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-shorthand-001.htm)
- [border-image-shorthand-002.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-shorthand-002.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-shorthand-002.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-shorthand-002.htm)
- [border-image-shorthand-003.htm](https://wpt.fyi/results/css/css-backgrounds/border-image-shorthand-003.htm) [(live test)](http://wpt.live/css/css-backgrounds/border-image-shorthand-003.htm) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-shorthand-003.htm)
- [border-image-space-001.html](https://wpt.fyi/results/css/css-backgrounds/border-image-space-001.html) [(live test)](http://wpt.live/css/css-backgrounds/border-image-space-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/border-image-space-001.html)
- [border-image-invalid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-invalid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-invalid.html)
- [border-image-shorthand.sub.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-shorthand.sub.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-shorthand.sub.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-shorthand.sub.html)
- [border-image-valid.html](https://wpt.fyi/results/css/css-backgrounds/parsing/border-image-valid.html) [(live test)](http://wpt.live/css/css-backgrounds/parsing/border-image-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/parsing/border-image-valid.html)

<a id="ref-for-propdef-border-image-source⑥"></a>

<a id="ref-for-propdef-border-image-slice⑦"></a>

<a id="ref-for-propdef-border-image-width①②"></a>

<a id="ref-for-propdef-border-image-outset③"></a>

<a id="ref-for-propdef-border-image-repeat③"></a>

<a id="ref-for-initial-value③"></a>

This is a shorthand property for setting [border-image-source](#propdef-border-image-source), [border-image-slice](#propdef-border-image-slice), [border-image-width](#propdef-border-image-width), [border-image-outset](#propdef-border-image-outset), and [border-image-repeat](#propdef-border-image-repeat) in a single declaration. Omitted values are set to their [initial values](https://www.w3.org/TR/css-cascade-5/#initial-value).

### <a id="border-image-tables"></a>4.8. Effect on Tables

<a id="ref-for-propdef-border-image⑥"></a>

<a id="ref-for-propdef-border-collapse⑦"></a>

The [border-image](#propdef-border-image) properties apply to the border of tables and inline tables that have [border-collapse](https://www.w3.org/TR/CSS2/tables.html#propdef-border-collapse) set to collapse. However, this specification does not define how such an image border is rendered. In particular, it does not define how the image border interacts with the borders of cells, rows and row groups at the edges of the table (see [border conflict resolution](https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html#border-conflict-resolution) in [\[CSS2\]](#biblio-css2)).

It is expected that a future specification will define the rendering. It is recommended that UAs do not apply border images to tables with collapsed borders until then.

## <a id="partial-borders"></a>5.  Partial borders

Not Ready For Implementation

This section is not yet ready for implementation. It exists in this repository to record the ideas and promote discussion.

Before attempting to implement anything of this section, please contact the CSSWG at www-style@w3.org.

CSS borders traditionally cover an entire border edge. Sometimes, however, it can be useful to hide some parts of the border.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-76b7fb28"></a> Here are two proposals for doing this: the second one is from GCPM, the first one is an attempt to recast it more readably. The names are terrible, known problem, proposals accepted. There is a problem with conceiving this as clipping: if you have dotted borders, you want whole dots always, not parts of dots. So it should be a drawing limit, not a clip.

<a id="ref-for-propdef-border-limit①"></a>

### <a id="border-limit"></a>5.1.  Partial Borders: the [border-limit](#propdef-border-limit) property



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-limit"></a>border-limit                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt⑥"></a><a id="ref-for-typedef-length-percentage①②"></a><a id="ref-for-comb-one③②"></a>all [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ sides <a id="ref-for-comb-one③③"></a>\| corners \] [\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)[?](https://www.w3.org/TR/css-values-4/#mult-opt) <a id="ref-for-comb-one③④"></a>\| \[ top <a id="ref-for-comb-one③⑤"></a>\| right <a id="ref-for-comb-one③⑥"></a>\| bottom <a id="ref-for-comb-one③⑦"></a>\| left \] <a id="ref-for-typedef-length-percentage①③"></a>\<length-percentage \[0,∞\]\> |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | all                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-valdef-white-space-collapse-collapse"></a><a id="ref-for-propdef-border-collapse⑧"></a>all elements, except table element when [border-collapse](https://www.w3.org/TR/CSS2/tables.html#propdef-border-collapse) is [collapse](https://www.w3.org/TR/css-text-4/#valdef-white-space-collapse-collapse)                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | relative to border-box                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | as specified                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |



By default, the entire border is drawn. However, border rendering can be limited to only part of a border. The keyword specifies which part, and the length or percentage specifies how much.

<a id="ref-for-valdef-border-limit-all"></a>

<a id="valdef-border-limit-all"></a>[all](#valdef-border-limit-all)

The entire border is drawn.

<a id="ref-for-valdef-border-limit-sides"></a>

<a id="valdef-border-limit-sides"></a>[sides](#valdef-border-limit-sides)

The sides are drawn up to but not including the corners (as defined by the border radii). A length or percentage is measured from the center of each side: 50% draws the middle 50% of the border; by default the entire side is drawn.

<a id="ref-for-valdef-border-limit-corners"></a>

<a id="valdef-border-limit-corners"></a>[corners](#valdef-border-limit-corners)

The corners are drawn plus the specified distance into the sides if specified. A length is measured from the closest edge of the corner area. A percentage is measured from the absolute corner of the border box.

<a id="ref-for-valdef-border-limit-left"></a>

<a id="valdef-border-limit-left"></a>[left](#valdef-border-limit-left)

<a id="ref-for-valdef-border-limit-right"></a>

<a id="valdef-border-limit-right"></a>[right](#valdef-border-limit-right)

<a id="ref-for-valdef-border-limit-corners①"></a>

For the left and right (vertical) sides, draws the entire side and corner. For the top and bottom (horizontal) sides, draws the left/right portion, as specified. Distances are measured as for [corners](#valdef-border-limit-corners).

<a id="ref-for-valdef-border-limit-top"></a>

<a id="valdef-border-limit-top"></a>[top](#valdef-border-limit-top)

<a id="ref-for-valdef-border-limit-bottom"></a>

<a id="valdef-border-limit-bottom"></a>[bottom](#valdef-border-limit-bottom)

<a id="ref-for-valdef-border-limit-corners②"></a>

For the top and bottom (horizontal) sides, draws the entire side and corner. For the left and right (vertical) sides, draws the top/bottom portion, as specified. Distances are measured as for [corners](#valdef-border-limit-corners).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-9cf65404"></a>
>
> The following example draws only the middle 50% of the sides.
>
> ```css
> div {
>   border: solid;
>   border-limit: sides 50%;
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-de686432"></a>
>
> The following example draws only the curved parts of the corners.
>
> ```css
> div {
>   border: solid;
>   border-radius: 1em 2em;
>   border-limit: corners;
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b9feeed8"></a>
>
> The following example draws only the left 4em of the top border.
>
> ```css
> div {
>   border-top: solid;
>   border-limit: left 4em;
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cf598566"></a>
>
> The following example draws only the first 10px of each corner:
>
> ```css
> div {
>   border: solid;
>   border-limit: corners 10px;
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5c3e340d"></a>
>
> The following example draws the curved part of the corner plus 5px along the sides:
>
> ```css
> div {
>   border: solid;
>   border-radius: 5px;
>   border-limit: corners 5px;
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f2051f6a"></a>
>
> The following example draws the curved part of the corner and all of the side except the middle 40%.
>
> ```css
> div {
>   border: solid;
>   border-radius: 5px;
>   border-limit: corners 30%;
> }
> ```
<a id="ref-for-propdef-border-clip"></a>

### <a id="border-clip"></a>5.2.  The [border-clip](#propdef-border-clip) properties



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                      |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-top-clip"></a>border-top-clip, <a id="propdef-border-right-clip"></a>border-right-clip, <a id="propdef-border-bottom-clip"></a>border-bottom-clip, <a id="propdef-border-left-clip"></a>border-left-clip, <a id="propdef-border-block-start-clip"></a>border-block-start-clip, <a id="propdef-border-block-end-clip"></a>border-block-end-clip, <a id="propdef-border-inline-start-clip"></a>border-inline-start-clip, <a id="propdef-border-inline-end-clip"></a>border-inline-end-clip                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-one-plus"></a><a id="ref-for-typedef-flex"></a><a id="ref-for-typedef-length-percentage①④"></a><a id="ref-for-comb-one③⑧"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ [\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) <a id="ref-for-comb-one③⑨"></a>\| [\<flex\>](https://www.w3.org/TR/css-grid-2/#typedef-flex) \][+](https://www.w3.org/TR/css-values-4/#mult-one-plus) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | refer to length of border-edge side                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | none, or a list consisting of absolute lengths, or percentages as specified                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-clip①"></a>[border-clip](#propdef-border-clip)                                                                                                                                                                                                                                                                                                                          |



<a id="ref-for-valdef-flex-fr"></a>

These properties split their respective borders into parts along the border edge. The first part is visible, the second is invisible, the third part is visible, etc. Parts can be specified with lengths, percentages, or flexible lengths (expressed by the [fr](https://www.w3.org/TR/css-grid-2/#valdef-flex-fr) unit, as per [\[CSS3GRID\]](#biblio-css3grid)). The none value means that the border is not split, but shown normally.

<a id="ref-for-flow-relative⑥"></a>

<a id="ref-for-propdef-border-block-start-clip"></a>

<a id="ref-for-physical⑥"></a>

<a id="ref-for-propdef-border-top-clip"></a>

<a id="ref-for-propdef-writing-mode⑥"></a>

<a id="ref-for-propdef-direction⑥"></a>

<a id="ref-for-propdef-text-orientation⑥"></a>

The [flow-relative](https://www.w3.org/TR/css-writing-modes-4/#flow-relative) longhands ([border-block-start-clip](#propdef-border-block-start-clip), etc.) correspond to the [physical](https://www.w3.org/TR/css-writing-modes-4/#physical) longhands ([border-top-clip](#propdef-border-top-clip), etc.) depending on the element’s [writing-mode](https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode), [direction](https://www.w3.org/TR/css-writing-modes-3/#propdef-direction), and [text-orientation](https://www.w3.org/TR/css-writing-modes-4/#propdef-text-orientation).



| Field               | Definition                                                                  |
|---------------------|-----------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-block-clip"></a>border-block-clip, <a id="propdef-border-inline-clip"></a>border-inline-clip |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-top-clip①"></a>[\<'border-top-clip'\>](#propdef-border-top-clip)        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                 |



<a id="ref-for-shorthand-property⑨"></a>

<a id="ref-for-propdef-border-block-start-clip①"></a>

<a id="ref-for-propdef-border-block-end-clip"></a>

<a id="ref-for-propdef-border-inline-start-clip"></a>

<a id="ref-for-propdef-border-inline-end-clip"></a>

These two [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [border-block-start-clip](#propdef-border-block-start-clip) &#x26; [border-block-end-clip](#propdef-border-block-end-clip) and [border-inline-start-clip](#propdef-border-inline-start-clip) &#x26; [border-inline-end-clip](#propdef-border-inline-end-clip), respectively.



| Field               | Definition                                                           |
|---------------------|----------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-clip"></a>border-clip                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-top-clip②"></a>[\<'border-top-clip'\>](#propdef-border-top-clip) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | see individual properties                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | see individual properties                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                          |



<a id="ref-for-propdef-border-clip②"></a>

<a id="ref-for-shorthand-property①⓪"></a>

[border-clip](#propdef-border-clip) is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for the longhand properties, setting all four sides to the same value.

If the listed parts are shorter than the border, any remaining border is split proportionally between the specified flexible lengths. If there are no flexible lengths, the behavior is as if 1fr had been specified at the end of the list.

If the listed parts are longer than the border, the specified parts will be shown in full until the end of the border. In this case, all flexible lengths will be zero.

For horizontal borders, parts are listed from left to right. For vertical borders, parts are listed from top to bottom.

The exact border parts are determined by laying out the specified border parts with all flexible lengths initially set to zero. Any remaining border is split proportionally between the flexible lengths specified.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-95b349db"></a>
>
> ```text
> border-clip: 10px 1fr 10px;
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3be2a11d"></a>
>
> ```text
> border-top-clip: 10px 1fr 10px;
> border-bottom-clip: 10px 1fr 10px;
> border-right-clip: 5px 1fr 5px;
> border-left-clip: 5px 1fr 5px;
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7c7bf675"></a>
>
> By making the first part have zero length, the inverse border of the previous example can easily be created:
>
> ```text
> border-top-clip: 0 10px 1fr 10px;
> border-bottom-clip: 0 10px 1fr 10px;
> border-right-clip: 0 5px 1fr 5px;
> border-left-clip: 0 5px 1fr 5px;
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3c49a25a"></a>
>
> ```text
> border: thin solid black;
> border-clip: 0 1fr; /* hide borders */
> border-top-clip: 10px 1fr 10px; /* make certain borders visible */
> border-bottom-clip: 10px 1fr 10px;
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-814f98d2"></a>
>
> ```text
> border-top: thin solid black;
> border-bottom: thin solid black;
> border-top-clip: 10px;
> border-bottom-clip: 10px;
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1c13dad3"></a>
>
> ```text
> border-top: thin solid black;
> border-clip: 10px;
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ae44ad9a"></a>
>
> This rendering:
>
> ```text
> A sentence consists of words¹.
> ```
>
> ```text
> ¹ Most often.
> ```
>
> can be achieved with this style sheet:
>
> ```text
> @footnote {
>   border-top: thin solid black;
>   border-clip: 4em;
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4c0cf4ac"></a>
>
> ```text
> border: 4px solid black;
> border-top-clip: 40px 20px 0 1fr 20px 20px 0 1fr 40px;
> ```
>
> In this example, there will be a visible 40px border part on each end of the top border. Inside the 40px border parts, there will be an invisible border part of at least 20px. Inside these invisible border parts, there will be visible border parts, each 20px long with 20px invisible border parts between them.
>
> The fragments are shown in red for illustrative purposes; they should not be visible in compliant UAs.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b107bdca"></a>
>
> ```text
> border: 4px solid black;
> border-top-clip: 3fr 10px 2fr 10px 1fr 10px 10px 10px 1fr 10px 2fr 10px 3fr;
> ```
>
> All but one of the visible border parts are represented as flexible lengths in this example. The length of these border parts will change when the width of the element changes. Here is one rendering where 1fr ends up being 10px:
>
> Here is another rendering where 1fr ends up being 30px:
>
> The fragments are shown in red for illustrative purposes; they should be black in compliant UAs.

Tests

- [border-clip-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-clip-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-clip-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-clip-computed.html)
- [border-clip-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-clip-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-clip-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-clip-invalid.html)
- [border-clip-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-clip-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-clip-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-clip-valid.html)

## <a id="drop-shadows"></a>6. <a id="misc"></a> Drop Shadows

<a id="ref-for-propdef-box-shadow-color"></a>

### <a id="box-shadow-color"></a>6.1.  Coloring shadows: the [box-shadow-color](#propdef-box-shadow-color) property



| Field               | Definition                                                                                                                                              |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-box-shadow-color"></a>box-shadow-color                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma"></a><a id="ref-for-typedef-color④"></a>[\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color)[\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | currentcolor                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | list, each item a computed color                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                       |



<a id="ref-for-propdef-box-shadow-color①"></a>

The [box-shadow-color](#propdef-box-shadow-color) property defines one or more drop shadow colors. The property accepts a comma-separated list of shadow colors.

<a id="ref-for-propdef-box-shadow-color②"></a>

See the section [“Layering, Layout, and Other Details”](https://www.w3.org/TR/css-backgrounds-3/#shadow-layers) for how [box-shadow-color](#propdef-box-shadow-color) interacts with other comma-separated drop shadow properties to form each drop shadow layer.

Tests

- [box-shadow-color-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-color-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-color-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-color-computed.html)
- [box-shadow-color-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-color-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-color-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-color-invalid.html)
- [box-shadow-color-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-color-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-color-valid.html)

<a id="ref-for-propdef-box-shadow-offset"></a>

### <a id="box-shadow-offset"></a>6.2.  Offsetting shadows: the [box-shadow-offset](#propdef-box-shadow-offset) property



| Field               | Definition                                                                                                                                                                                                                                                                                                              |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-box-shadow-offset"></a>box-shadow-offset                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma①"></a><a id="ref-for-mult-num-range①⑦"></a><a id="ref-for-length-value③"></a><a id="ref-for-comb-one④⓪"></a>\[ none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) \][\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-shadow-offset-none"></a>list, each item either [none](#shadow-offset-none) or a pair of offsets (horizontal and vertical) from the element‘s box                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <a id="ref-for-shadow-offset-none①"></a>by computed value, treating [none](#shadow-offset-none) as 0 0 when interpolated with non-<a id="ref-for-shadow-offset-none②"></a>none values.                                                                                                                                                                            |



<a id="ref-for-propdef-box-shadow-offset①"></a>

<a id="ref-for-shadow-offset-none③"></a>

<a id="ref-for-length-value④"></a>

The [box-shadow-offset](#propdef-box-shadow-offset) property defines one or more drop shadow offsets. The property accepts a comma-separated list. Each item in that list can either be the [none](#shadow-offset-none) value, which indicates no shadow, a pair of [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) values, which define the horizontal and vertical offsets, or a single <a id="ref-for-length-value⑤"></a>\<length\> value, which sets both offsets to the same value.

<a id="shadow-offset-none"></a>none

The shadow will not be rendered. The values of other box shadow properties corresponding to this shadow have no effect.

<a id="ref-for-length-value⑥"></a>

<a id="shadow-offset-x"></a>1st [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

Specifies the <a id="valdef-box-shadow-offset-horizontal-offset"></a>horizontal offset of the shadow. A positive value draws a shadow that is offset to the right of the box, a negative length to the left.

<a id="ref-for-length-value⑦"></a>

If only one [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) value is specified, it sets both the horizontal and vertical offsets to that value.

<a id="ref-for-length-value⑧"></a>

<a id="shadow-offset-y"></a>2nd [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

Specifies the <a id="valdef-box-shadow-offset-vertical-offset"></a>vertical offset of the shadow. A positive value offsets the shadow down, a negative one up.

<a id="ref-for-propdef-box-shadow-offset②"></a>

See the section [“Layering, Layout, and Other Details”](https://www.w3.org/TR/css-backgrounds-3/#shadow-layers) for how [box-shadow-offset](#propdef-box-shadow-offset) interacts with other comma-separated drop shadow properties to form each drop shadow layer.

Tests

- [box-shadow-offset-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-offset-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-offset-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-offset-computed.html)
- [box-shadow-offset-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-offset-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-offset-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-offset-invalid.html)
- [box-shadow-offset-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-offset-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-offset-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-offset-valid.html)

<a id="ref-for-propdef-box-shadow-blur"></a>

### <a id="box-shadow-blur"></a>6.3.  Blurring shadows: the [box-shadow-blur](#propdef-box-shadow-blur) property



| Field               | Definition                                                                                                                                                       |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-box-shadow-blur"></a>box-shadow-blur                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma②"></a><a id="ref-for-length-value⑨"></a>[\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value)[\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-length-value①⓪"></a>list, each item a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)                                                              |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                |



<a id="ref-for-propdef-box-shadow-blur①"></a>

<a id="ref-for-length-value①①"></a>

The [box-shadow-blur](#propdef-box-shadow-blur) property defines one or more blur radii for drop shadows. The property accepts a comma-separated list of [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) values.

Negative values are invalid. If the blur value is zero, the shadow’s edge is sharp. Otherwise, the larger the value, the more the shadow’s edge is blurred. See [Shadow Blurring](https://www.w3.org/TR/css-backgrounds-3/#shadow-blur), below.

<a id="ref-for-propdef-box-shadow-blur②"></a>

See the section [“Layering, Layout, and Other Details”](https://www.w3.org/TR/css-backgrounds-3/#shadow-layers) for how [box-shadow-blur](#propdef-box-shadow-blur) interacts with other comma-separated drop shadow properties to form each drop shadow layer.

Tests

- [box-shadow-blur-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-blur-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-blur-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-blur-computed.html)
- [box-shadow-blur-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-blur-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-blur-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-blur-invalid.html)
- [box-shadow-blur-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-blur-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-blur-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-blur-valid.html)

<a id="ref-for-propdef-box-shadow-spread①"></a>

### <a id="box-shadow-spread"></a>6.4.  Spreading shadows: the [box-shadow-spread](#propdef-box-shadow-spread) property



| Field               | Definition                                                                                                                                               |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-box-shadow-spread"></a>box-shadow-spread                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma③"></a><a id="ref-for-length-value①②"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)[\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-length-value①③"></a>list, each item a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                        |



<a id="ref-for-propdef-box-shadow-spread②"></a>

<a id="ref-for-length-value①④"></a>

The [box-shadow-spread](#propdef-box-shadow-spread) property defines one or more spread distances for drop shadows. The property accepts a comma-separated list of [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) values.

Positive values cause the shadow to expand in all directions by the specified radius. Negative values cause the shadow to contract. See [Shadow Shape](https://www.w3.org/TR/css-backgrounds-3/#shadow-shape), below.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that for inner shadows, expanding the shadow (creating more shadow area) means contracting the shadow’s perimeter shape.

<a id="ref-for-propdef-box-shadow-spread③"></a>

See the section [“Layering, Layout, and Other Details”](https://www.w3.org/TR/css-backgrounds-3/#shadow-layers) for how [box-shadow-spread](#propdef-box-shadow-spread) interacts with other comma-separated drop shadow properties to form each drop shadow layer.

Tests

- [box-shadow-spread-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-spread-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-spread-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-spread-computed.html)
- [box-shadow-spread-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-spread-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-spread-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-spread-invalid.html)
- [box-shadow-spread-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-spread-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-spread-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-spread-valid.html)

<a id="ref-for-propdef-box-shadow-position"></a>

### <a id="box-shadow-position"></a>6.5.  Spreading shadows: the [box-shadow-position](#propdef-box-shadow-position) property



| Field               | Definition                                                                                                                                                      |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-box-shadow-position"></a>box-shadow-position                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma④"></a><a id="ref-for-comb-one④①"></a>\[ outset [\|](https://www.w3.org/TR/css-values-4/#comb-one) inset \][\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | outset                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | list, each item one of the keywords                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                               |



<a id="ref-for-propdef-box-shadow-position①"></a>

<a id="ref-for-valdef-box-shadow-position-outset"></a>

<a id="ref-for-valdef-box-shadow-position-inset"></a>

The [box-shadow-position](#propdef-box-shadow-position) property defines one or more drop shadow positions. The property accepts a comma-separated list of [outset](#valdef-box-shadow-position-outset) and [inset](#valdef-box-shadow-position-inset) keywords.

<a id="valdef-box-shadow-position-outset"></a>outset  
Causes the drop shadow to be an <a id="valdef-box-shadow-position-outer-box-shadow"></a>outer box-shadow. That means, one that shadows the box onto the canvas, as if it were lifted above the canvas.

<a id="valdef-box-shadow-position-inset"></a>inset  
Causes the drop shadow to be an <a id="valdef-box-shadow-position-inner-box-shadow"></a>inner box-shadow. That means, one that shadows the canvas onto the box, as if the box were cut out of the canvas and shifted behind it.

<a id="ref-for-propdef-box-shadow-position②"></a>

See the section [“Layering, Layout, and Other Details”](https://www.w3.org/TR/css-backgrounds-3/#shadow-layers) for how [box-shadow-position](#propdef-box-shadow-position) interacts with other comma-separated drop shadow properties to form each drop shadow layer.

Tests

- [box-shadow-position-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-position-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-position-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-position-computed.html)
- [box-shadow-position-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-position-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-position-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-position-invalid.html)
- [box-shadow-position-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/box-shadow-position-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/box-shadow-position-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/box-shadow-position-valid.html)

<a id="ref-for-propdef-box-shadow⑤"></a>

### <a id="box-shadow"></a>6.6. <a id="the-box-shadow"></a> Drop Shadows Shorthand: the [box-shadow](#propdef-box-shadow) property



| Field               | Definition                                                                                                                            |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-box-shadow"></a>box-shadow                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma⑤"></a><a id="ref-for-typedef-spread-shadow"></a>[\<spread-shadow\>](#typedef-spread-shadow)[\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                             |



<a id="ref-for-propdef-box-shadow⑥"></a>

The [box-shadow](#propdef-box-shadow) property attaches one or more drop-shadows to the box. The property accepts a comma-separated list of shadows, ordered front to back.

<a id="ref-for-typedef-spread-shadow①"></a>

<a id="ref-for-propdef-box-shadow-offset③"></a>

<a id="ref-for-length-value①⑤"></a>

<a id="ref-for-propdef-box-shadow-blur③"></a>

<a id="ref-for-propdef-box-shadow-spread④"></a>

<a id="ref-for-propdef-box-shadow-color③"></a>

<a id="ref-for-propdef-box-shadow-position③"></a>

<a id="ref-for-valdef-color-transparent"></a>

<a id="ref-for-shadow-offset-none④"></a>

<a id="ref-for-valdef-color-currentcolor"></a>

Each shadow is given as a [\<spread-shadow\>](#typedef-spread-shadow), outlining the [box-shadow-offset](#propdef-box-shadow-offset) defined by two [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) values, and optional values for the [box-shadow-blur](#propdef-box-shadow-blur), [box-shadow-spread](#propdef-box-shadow-spread), [box-shadow-color](#propdef-box-shadow-color), and [box-shadow-position](#propdef-box-shadow-position). Omitted lengths are 0; omitted colors default to [transparent](https://www.w3.org/TR/css-color-4/#valdef-color-transparent) when the specified offset is [none](#shadow-offset-none) and to [currentcolor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor) otherwise.

<a id="ref-for-length-value①⑥"></a>

<a id="ref-for-propdef-box-shadow-offset④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: To avoid ambiguities in parsing the different [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) values, the offset has to be specified as two <a id="ref-for-length-value①⑦"></a>\<length\> values, in opposite to the [box-shadow-offset](#propdef-box-shadow-offset) property, where a single <a id="ref-for-length-value①⑧"></a>\<length\> value can be used to specify both offsets.

<a id="typedef-spread-shadow"></a>

<a id="ref-for-typedef-spread-shadow②"></a>

<a id="ref-for-propdef-box-shadow-color④"></a>

<a id="ref-for-mult-opt⑦"></a>

<a id="ref-for-comb-all①"></a>

<a id="ref-for-comb-one④②"></a>

<a id="ref-for-length-value①⑨"></a>

<a id="ref-for-mult-num"></a>

<a id="ref-for-propdef-box-shadow-blur④"></a>

<a id="ref-for-propdef-box-shadow-spread⑤"></a>

<a id="ref-for-mult-opt⑧"></a>

<a id="ref-for-mult-opt⑨"></a>

<a id="ref-for-comb-all②"></a>

<a id="ref-for-propdef-box-shadow-position④"></a>

<a id="ref-for-mult-opt①⓪"></a>

```text
<spread-shadow> = <'box-shadow-color'>? && [ [ none | <length>{2} ] [ <'box-shadow-blur'> <'box-shadow-spread'>? ]? ] && <'box-shadow-position'>?
```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ad571670"></a> The example below demonstrates the effects of spread and blur on the shadow:
>
> ```text
> width: 100px; height: 100px;
> border: 12px solid blue; background-color: orange;
> border-top-left-radius: 60px 90px;
> border-bottom-right-radius: 60px 90px;
> box-shadow: 64px 64px 12px 40px rgba(0,0,0,0.4),
>             12px 12px 0px 8px rgba(0,0,0,0.4) inset;
> ```
>
> ![The sample code above would create a 100px×100px orange box with a 12px blue border, whose top right / bottom left corners are sharp and tob left / bottom right corners are elliptically curved. Two shadows are created: an inner one, which due to its offset and spread creates a 20px-wide band of darker orange along the top and left sides of the box (curving to match the rounded top left border shape); and an outer one, creating a 204px×204px gray duplicate of the shape seemingly behind the box, offset 24px down and 24px to the right of the box's top and left edges. Applying the 12px blur radius to the outer shadow creates a gradual shift from the shadow color to transparent along its edges which is visibly apparent for 24px centered along the edge of the shadow.](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/spread-blur.png)

#### <a id="shadow-shape"></a>6.6.1.  Shadow Shape, Spread, and Knockout

<a id="ref-for-box-shadow-outer-box-shadow"></a>

An [outer box-shadow](https://www.w3.org/TR/css-backgrounds-3/#box-shadow-outer-box-shadow) casts a shadow as if the border-box of the element were opaque. Assuming a spread distance of zero, its perimeter has the exact same size and shape as the border box. The shadow is drawn outside the border edge only: it is clipped inside the border-box of the element.

<a id="ref-for-box-shadow-inner-box-shadow"></a>

An [inner box-shadow](https://www.w3.org/TR/css-backgrounds-3/#box-shadow-inner-box-shadow) casts a shadow as if everything outside the padding edge were opaque. Assuming a spread distance of zero, its perimeter has the exact same size and shape as the padding box. The shadow is drawn inside the padding edge only: it is clipped outside the padding box of the element.

<a id="ref-for-box-shadow-spread-distance"></a>

<a id="ref-for-box-shadow-outer-box-shadow①"></a>

<a id="ref-for-box-shadow-inner-box-shadow①"></a>

If a [spread distance](https://www.w3.org/TR/css-backgrounds-3/#box-shadow-spread-distance) is defined, the shadow perimeter defined above is expanded outward (for [outer box-shadows](https://www.w3.org/TR/css-backgrounds-3/#box-shadow-outer-box-shadow)) or contracted inward (for [inner box-shadows](https://www.w3.org/TR/css-backgrounds-3/#box-shadow-inner-box-shadow)) by outsetting (insetting, for inner shadows) the shadow’s straight edges by the <a id="ref-for-box-shadow-spread-distance①"></a>spread distance (and flooring the resulting width/height at zero).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-96985467"></a> Below are some examples of an orange box with a blue border being given a drop shadow.
>
> <a id="box-shadow-samples"></a>
>
> **Table 39**
>
> **Shared box styling**
>
> ```text
> border:5px solid blue;
> background-color:orange;
> width: 144px;
> height: 144px;
> ```
>
> **Rounded-corner variant**
>
> ```text
> border-radius: 20px;
> ```
>
> **Square-corner variant**
>
> ```text
> border-radius: 0;
> ```
>
> **Shadow example 1**
>
> ```text
> box-shadow:
>   rgba(0,0,0,0.4)
>   10px 10px;
> ```
>
> **Rounded-corner reference image**
>
> ![A round-cornered box with a light gray shadow the same shape as the border box offset 10px to the right and 10px down from directly underneath the box.](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/shadow-outer-round.png)
>
> **Square-corner reference image**
>
> ![A square-cornered box with a light gray shadow the same shape as the border box offset 10px to the right and 10px down from directly underneath the box.](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/shadow-outer-square.png)
>
> **Shadow example 2**
>
> ```text
> box-shadow:
>   rgba(0,0,0,0.4)
>   10px 10px
>   inset
> ```
>
> **Rounded-corner reference image**
>
> ![A round-cornered box with a light gray shadow the inverse shape of the padding box filling 10px in from the top and left edges (just inside the border).](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/shadow-inner-round.png)
>
> **Square-corner reference image**
>
> ![A square-cornered box with a light gray shadow the inverse shape of the padding box filling 10px in from the top and left edges (just inside the border).](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/shadow-inner-square.png)
>
> **Shadow example 3**
>
> ```text
> box-shadow:
>   rgba(0,0,0,0.4)
>   10px 10px 0
>   10px /* spread */
> ```
>
> **Rounded-corner reference image**
>
> ![A round-cornered box with a light gray shadow the same shape as the box but 20px taller and wider and offset so that the top and left edges of the shadow are directly underneath the top and left edges of the box.](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/shadow-outer-spread-round.png)
>
> **Square-corner reference image**
>
> ![A square-cornered box with a light gray shadow the same shape as the box but 20px taller and wider and offset so that the top and left edges of the shadow are directly underneath the top and left edges of the box.](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/shadow-outer-spread-square.png)
>
> **Shadow example 4**
>
> ```text
> box-shadow:
>   rgba(0,0,0,0.4)
>   10px 10px 0
>   10px /* spread */
>   inset
> ```
>
> **Rounded-corner reference image**
>
> ![A round-cornered box with a light gray shadow the inverse shape of the box but 20px narrower and shorter filling 20px in from the top and left edges (just inside the border).](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/shadow-inner-spread-round.png)
>
> **Square-corner reference image**
>
> ![A round-cornered box with a light gray shadow the inverse shape of the box but 20px narrower and shorter filling 20px in from the top and left edges (just inside the border).](https://www.w3.org/TR/2025/WD-css-borders-4-20251216/images/shadow-inner-spread-square.png)

<a id="ref-for-box-shadow-spread-distance②"></a>

<a id="ref-for-border-radii④"></a>

<a id="ref-for-outset-adjusted-border-radius③"></a>

To preserve the box’s shape when spread is applied, the corner radii of the shadow are also increased (decreased, for inner shadows) from the border-box (padding-box) radii by adding (subtracting) the [spread distance](https://www.w3.org/TR/css-backgrounds-3/#box-shadow-spread-distance) (and flooring at zero). For outer shadows, the [border radius](#border-radii) is then [adjusted](#outset-adjusted-border-radius), independently in each dimension, to preserve the sharpness of rounded corners.

<a id="ref-for-propdef-border-image⑦"></a>

The [border-image](#propdef-border-image) does not affect the shape of the box-shadow.

#### <a id="shadow-blur"></a>6.6.2.  Blurring Shadow Edges

<a id="ref-for-box-shadow-blur-radius"></a>

A non-zero [blur radius](https://www.w3.org/TR/css-backgrounds-3/#box-shadow-blur-radius) indicates that the resulting shadow should be blurred, such as by a Gaussian filter. The exact algorithm is not defined; however the resulting shadow must approximate (with each pixel being within 5% of its expected value) the image that would be generated by applying to the shadow a Gaussian blur with a standard deviation equal to half the blur radius.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means for a long, straight shadow edge, the blur radius will create a visibly apparent color transition approximately the twice length of the blur radius that is perpendicular to and centered on the shadow’s edge, and that ranges from almost the full shadow color at the endpoint inside the shadow to almost fully transparent at the endpoint outside it.

### <a id="shadow-layers"></a>6.7.  Layering, Layout, and Other Details

<a id="ref-for-coordinated-value-list"></a>

<a id="ref-for-coordinating-list-property"></a>

<a id="ref-for-propdef-box-shadow-offset⑤"></a>

<a id="ref-for-coordinating-list-base-property"></a>

Drop shadows are declared in the [coordinated value list](https://www.w3.org/TR/css-values-4/#coordinated-value-list) constructed from the box-shadow-\* properties, which form a [coordinating list property group](https://www.w3.org/TR/css-values-4/#coordinating-list-property) with [box-shadow-offset](#propdef-box-shadow-offset) as the [coordinating list base property](https://www.w3.org/TR/css-values-4/#coordinating-list-base-property). See [CSS Values 4 § A Coordinating List-Valued Properties](https://www.w3.org/TR/css-values-4/#linked-properties).

The shadow effects are applied front-to-back: the first shadow is on top and the others are layered behind. Shadows do not influence layout and may overlap (or be overlapped by) other boxes and text or their shadows. In terms of stacking contexts and the painting order, the <i>outer box-shadows</i> of an element are drawn immediately below the background of that element, and the <i>inner shadows</i> of an element are drawn immediately above the background of that element (below the borders and border image, if any).

<a id="ref-for-principal-box"></a>

<a id="ref-for-propdef-box-decoration-break"></a>

Unless otherwise specified, drop shadows are only applied to the [principal box](https://www.w3.org/TR/css-display-4/#principal-box). If the affected box has multiple fragments, the shadows are applied as specified in [box-decoration-break](https://www.w3.org/TR/css-break-4/#propdef-box-decoration-break).

Shadows do not trigger scrolling or increase the size of the scrollable area.

Outer shadows have no effect on internal table elements in the collapsing border model. If a shadow is defined for single border edge in the collapsing border model that has multiple border thicknesses (e.g. an outer shadow on a table where one row has thicker borders than the others, or an inner shadow on a rowspanning table cell that adjoins cells with different border thicknesses), the exact position and rendering of its shadows are undefined.

## <a id="border-shape"></a>7.  Border Shaping

<a id="ref-for-propdef-corner-shape①③"></a>

<a id="ref-for-propdef-border-radius①⑦"></a>

While [corner-shape](#propdef-corner-shape) and [border-radius](#propdef-border-radius) allow some expressiveness to styling a border, they still work with the assumption that the border is rectangular.

<a id="ref-for-propdef-border-shape①"></a>

<a id="ref-for-basic-shape"></a>

The [border-shape](#propdef-border-shape) function augments these capabilities, by enabling the author to use any [basic shape](https://www.w3.org/TR/SVG2/shapes.html#basic-shape) to specify the path of the border.

<a id="ref-for-propdef-border-shape②"></a>

### <a id="border-shape-func"></a>7.1.  The [border-shape](#propdef-border-shape) property



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                       |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-shape"></a>border-shape                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range①⑧"></a><a id="ref-for-mult-opt①①"></a><a id="ref-for-typedef-geometry-box"></a><a id="ref-for-typedef-basic-shape"></a><a id="ref-for-comb-one④③"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) [\<geometry-box\>](https://www.w3.org/TR/css-masking-1/#typedef-geometry-box)[?](https://www.w3.org/TR/css-values-4/#mult-opt)\][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see prose                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | list, each item a computed color                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                                                                                                                                                                                |



<a id="ref-for-propdef-border-shape③"></a>

<a id="ref-for-typedef-basic-shape①"></a>

The [border-shape](#propdef-border-shape) property is provided with either a single [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) or two <a id="ref-for-typedef-basic-shape②"></a>\<basic-shape\>s, resulting in one or two paths, respectively.

<a id="ref-for-typedef-basic-shape③"></a>

<a id="ref-for-relevant-side-for-border-shape"></a>

<a id="ref-for-computed-value①③"></a>

<a id="ref-for-border-width-dfn①"></a>

When two [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) values are given, the border is rendered as the shape between the two paths. When only a single <a id="ref-for-typedef-basic-shape④"></a>\<basic-shape\> is given, the border is rendered as a stroke with the [relevant side](#relevant-side-for-border-shape)’s [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [border width](#border-width-dfn) as the stroke width.

<a id="ref-for-relevant-side-for-border-shape①"></a>

<a id="ref-for-computed-value①④"></a>

<a id="ref-for-propdef-border-color⑦"></a>

The fill color of the border is the [relevant side](#relevant-side-for-border-shape)’s [computed](https://www.w3.org/TR/css-cascade-5/#computed-value)[border-color](#propdef-border-color).

<a id="ref-for-typedef-geometry-box①"></a>

<a id="ref-for-typedef-basic-shape⑤"></a>

When a [\<geometry-box\>](https://www.w3.org/TR/css-masking-1/#typedef-geometry-box) is not given, the default computation of percentage differs based on the number of given [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) values.

<a id="ref-for-typedef-basic-shape⑥"></a>

<a id="ref-for-border-box①②"></a>

<a id="ref-for-padding-box"></a>

<a id="ref-for-propdef-border-width①①"></a>

When two [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) values are given, the first (outer) one defaults to the [border box](https://www.w3.org/TR/css-box-4/#border-box) and the second (inner) one defaults to the [padding box](https://www.w3.org/TR/css-box-4/#padding-box). This allows using the different [border-width](#propdef-border-width) properties to affect the final shape.

<a id="ref-for-typedef-basic-shape⑦"></a>

<a id="ref-for-typedef-geometry-box②"></a>

<a id="ref-for-valdef-shape-box-half-border-box"></a>

When a single [\<basic-shape\>](https://www.w3.org/TR/css-shapes-1/#typedef-basic-shape) value is given, the [\<geometry-box\>](https://www.w3.org/TR/css-masking-1/#typedef-geometry-box) defaults to the [half-border-box](https://drafts.csswg.org/css-shapes-1/#valdef-shape-box-half-border-box) value, which allows stroking in a way that matches the default border behavior.

<a id="ref-for-propdef-border-shape④"></a>

<a id="ref-for-propdef-border-radius①⑧"></a>

<a id="ref-for-propdef-corner-shape①④"></a>

<a id="ref-for-computed-value①⑤"></a>

The [border-shape](#propdef-border-shape) property is not compatible with [border-radius](#propdef-border-radius) and [corner-shape](#propdef-corner-shape). When an element’s [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of <a id="ref-for-propdef-border-shape⑤"></a>border-shape is not none, its <a id="ref-for-propdef-border-radius①⑨"></a>border-radius is ignored, as if it was set to 0. <a id="ref-for-propdef-corner-shape①⑤"></a>corner-shape is implicitly ignored, as it can only work in tandem with <a id="ref-for-propdef-border-radius②⓪"></a>border-radius.

<a id="ref-for-box-shadow-outer-box-shadow②"></a>

<a id="ref-for-box-shadow-inner-box-shadow②"></a>

An [outer box-shadow](https://www.w3.org/TR/css-backgrounds-3/#box-shadow-outer-box-shadow) follows the outside of the outer path, and an [inner box-shadow](https://www.w3.org/TR/css-backgrounds-3/#box-shadow-inner-box-shadow) follows the inside inner path. Both are rendered as a stroke, with a stroke width of `spread * 2`, clipped by the border shape.

<a id="ref-for-propdef-border-shape⑥"></a>

<a id="ref-for-propdef-border-width①②"></a>

[border-shape](#propdef-border-shape) does not affect geometry or layout, which is still computed using the existing [border-width](#propdef-border-width) properties.

<a id="ref-for-propdef-border-shape⑦"></a>

<a id="ref-for-propdef-shape-inside"></a>

[border-shape](#propdef-border-shape) does not affect the flow of content inside the box. Note: An author can use <a id="ref-for-propdef-border-shape⑧"></a>border-shape in tandem with [shape-inside](https://drafts.csswg.org/css-shapes-2/#propdef-shape-inside) to create effects that decorate the box and control its text flow at the same time.

<a id="ref-for-propdef-border-shape⑨"></a>

<a id="ref-for-overflow②"></a>

<a id="ref-for-propdef-border-radius②①"></a>

The inner [border-shape](#propdef-border-shape) clips the [overflow](https://www.w3.org/TR/css-overflow-3/#overflow) content of the element, in the same manner as [border-radius](#propdef-border-radius), as described in [corner clipping](https://drafts.csswg.org/css-backgrounds-3/#corner-clipping).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-e4b44b73"></a> how should this affect clipping replaced elements?

<a id="ref-for-block-start"></a>

<a id="ref-for-inline-start"></a>

<a id="ref-for-block-end"></a>

<a id="ref-for-inline-end"></a>

<a id="ref-for-valdef-line-style-none③"></a>

<a id="ref-for-border-style-dfn"></a>

An element’s <a id="relevant-side-for-border-shape"></a>relevant side for border shape is the first side (in the order [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start), [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start), [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end), and [inline-end](https://www.w3.org/TR/css-writing-modes-4/#inline-end)) that has a non-[none](#valdef-line-style-none) [border style](#border-style-dfn), or <a id="ref-for-block-start①"></a>block-start if they’re all <a id="ref-for-valdef-line-style-none④"></a>none.

1.  <a id="ref-for-computed-value①⑥"></a>

    <a id="ref-for-propdef-border-block-start-style②"></a>

    <a id="ref-for-valdef-line-style-none⑤"></a>

    <a id="ref-for-block-start②"></a>

    If <var>element</var>’s [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [border-block-start-style](#propdef-border-block-start-style) is not [none](#valdef-line-style-none), then return [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start).

2.  <a id="ref-for-computed-value①⑦"></a>

    <a id="ref-for-propdef-border-inline-start-style②"></a>

    <a id="ref-for-valdef-line-style-none⑥"></a>

    <a id="ref-for-inline-start①"></a>

    If <var>element</var>’s [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [border-inline-start-style](#propdef-border-inline-start-style) is not [none](#valdef-line-style-none), then return [inline-start](https://www.w3.org/TR/css-writing-modes-4/#inline-start).

3.  <a id="ref-for-computed-value①⑧"></a>

    <a id="ref-for-propdef-border-block-end-style②"></a>

    <a id="ref-for-valdef-line-style-none⑦"></a>

    <a id="ref-for-block-end①"></a>

    If <var>element</var>’s [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [border-block-end-style](#propdef-border-block-end-style) is not [none](#valdef-line-style-none), then return [block-end](https://www.w3.org/TR/css-writing-modes-4/#block-end).

4.  <a id="ref-for-computed-value①⑨"></a>

    <a id="ref-for-propdef-border-inline-end-style②"></a>

    <a id="ref-for-valdef-line-style-none⑧"></a>

    <a id="ref-for-inline-end①"></a>

    If <var>element</var>’s [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [border-inline-end-style](#propdef-border-inline-end-style) is not [none](#valdef-line-style-none), then return [inline-end](https://www.w3.org/TR/css-writing-modes-4/#inline-end).

5.  <a id="ref-for-block-start③"></a>

    Return [block-start](https://www.w3.org/TR/css-writing-modes-4/#block-start).

Tests

- [border-shape-clips-background.html](https://wpt.fyi/results/css/css-borders/tentative/border-shape/border-shape-clips-background.html) [(live test)](http://wpt.live/css/css-borders/tentative/border-shape/border-shape-clips-background.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/border-shape/border-shape-clips-background.html)
- [border-shape-double-shape-default.html](https://wpt.fyi/results/css/css-borders/tentative/border-shape/border-shape-double-shape-default.html) [(live test)](http://wpt.live/css/css-borders/tentative/border-shape/border-shape-double-shape-default.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/border-shape/border-shape-double-shape-default.html)
- [border-shape-inner-outer.html](https://wpt.fyi/results/css/css-borders/tentative/border-shape/border-shape-inner-outer.html) [(live test)](http://wpt.live/css/css-borders/tentative/border-shape/border-shape-inner-outer.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/border-shape/border-shape-inner-outer.html)
- [border-shape-overflow.html](https://wpt.fyi/results/css/css-borders/tentative/border-shape/border-shape-overflow.html) [(live test)](http://wpt.live/css/css-borders/tentative/border-shape/border-shape-overflow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/border-shape/border-shape-overflow.html)
- [border-shape-shadow.html](https://wpt.fyi/results/css/css-borders/tentative/border-shape/border-shape-shadow.html) [(live test)](http://wpt.live/css/css-borders/tentative/border-shape/border-shape-shadow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/border-shape/border-shape-shadow.html)
- [border-shape-geometry-box.html](https://wpt.fyi/results/css/css-borders/tentative/border-shape/border-shape-geometry-box.html) [(live test)](http://wpt.live/css/css-borders/tentative/border-shape/border-shape-geometry-box.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/border-shape/border-shape-geometry-box.html)
- [border-shape-half-border-box-default.html](https://wpt.fyi/results/css/css-borders/tentative/border-shape/border-shape-half-border-box-default.html) [(live test)](http://wpt.live/css/css-borders/tentative/border-shape/border-shape-half-border-box-default.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/border-shape/border-shape-half-border-box-default.html)
- [border-shape-overflow-solid-background.html](https://wpt.fyi/results/css/css-borders/tentative/border-shape/border-shape-overflow-solid-background.html) [(live test)](http://wpt.live/css/css-borders/tentative/border-shape/border-shape-overflow-solid-background.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/border-shape/border-shape-overflow-solid-background.html)
- [border-shape-stroke-from-border.html](https://wpt.fyi/results/css/css-borders/tentative/border-shape/border-shape-stroke-from-border.html) [(live test)](http://wpt.live/css/css-borders/tentative/border-shape/border-shape-stroke-from-border.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/border-shape/border-shape-stroke-from-border.html)
- [border-shape-stroke-invalidation.html](https://wpt.fyi/results/css/css-borders/tentative/border-shape/border-shape-stroke-invalidation.html) [(live test)](http://wpt.live/css/css-borders/tentative/border-shape/border-shape-stroke-invalidation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/border-shape/border-shape-stroke-invalidation.html)
- [border-shape-computed.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-shape-computed.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-shape-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-shape-computed.html)
- [border-shape-invalid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-shape-invalid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-shape-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-shape-invalid.html)
- [border-shape-valid.html](https://wpt.fyi/results/css/css-borders/tentative/parsing/border-shape-valid.html) [(live test)](http://wpt.live/css/css-borders/tentative/parsing/border-shape-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-borders/tentative/parsing/border-shape-valid.html)

## <a id="privacy"></a>Privacy Considerations

No new privacy considerations have been reported on this specification.

## <a id="security"></a>Security Considerations

No new security considerations have been reported on this specification.

## <a id="changes"></a> Changes

### <a id="changes-20250722"></a> Changes since the [First Public Working Draft](https://www.w3.org/TR/2025/WD-css-borders-4-20250722/) of 22 July 2025 

- Added corner-\* shorthands

- <a id="ref-for-propdef-corner③"></a>

  Renamed `corners` to [corner](#propdef-corner)

- Added Web Platform Tests coverage

- Incorporated full text of [\[CSS3BG\]](#biblio-css3bg) related to borders and shadows

- Renamed border-clip-\* properties to border-\*-clip and added logical longhands and shorthands

- Renamed normal value of border-\*-clip properties to none

- Added new syntax for border-\*-\*-radius longhands using a slash to separate horizontal and vertical radii

- <a id="ref-for-length-value②⓪"></a>

  <a id="ref-for-propdef-box-shadow-offset⑥"></a>

  Allowed a single [\<length\>](https://www.w3.org/TR/css-values-4/#length-value) value for [box-shadow-offset](#propdef-box-shadow-offset) to set both offsets to the same value

### <a id="level-changes"></a> Additions since [\[CSS3BG\]](#biblio-css3bg)

- <a id="ref-for-typedef-image-1d③"></a>

  <a id="ref-for-propdef-border-color⑧"></a>

  [\<image-1D\>](https://www.w3.org/TR/css-images-4/#typedef-image-1d) as value for [border-color](#propdef-border-color) and its longhands

- Added physical and logical border-\*-radius shorthands

- <a id="ref-for-propdef-corner-shape①⑥"></a>

  <a id="ref-for-propdef-corner④"></a>

  Added [corner-shape](#propdef-corner-shape) and corner-\*-shape shorthands, plus related [corner](#propdef-corner) and corner-\* shorthands

- <a id="ref-for-propdef-border-shape①⓪"></a>

  Added [border-shape](#propdef-border-shape)

- <a id="ref-for-propdef-border-limit②"></a>

  Added [partial borders](#partial-borders) via [border-limit](#propdef-border-limit) and border-\*-clip properties

- <a id="ref-for-propdef-box-shadow⑦"></a>

  Added box-shadow-\* longhands and turned [box-shadow](#propdef-box-shadow) into a shorthand

- Moved logical border properties from [\[CSS-LOGICAL-1\]](#biblio-css-logical-1) to this spec.

## <a id="acknowledgments"></a> Acknowledgments

In addition to the many contributors to the [\[CSS1\]](#biblio-css1), [\[CSS2\]](#biblio-css2), and [\[CSS3BG\]](#biblio-css3bg) predecessors to this module, the editors would like to thank Tab Atkins, Noam Rosenthal, Håkon Wium Lie, Oriol Brufau, and Guillaume Lebas for their suggestions and feedback specifically for this Level 4.

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

## <a id="index"></a>Index

### <a id="index-defined-here"></a>Terms defined by this specification

- [1st \<length\>](#shadow-offset-x), in § 6.2
- [2nd \<length\>](#shadow-offset-y), in § 6.2
- [add corner to path](#add-corner-to-path), in § 3.9.4
- [adjusted radius dimension](#adjusted-radius-dimension), in § 3
- [aligned corner point](#aligned-corner-point), in § 3.9.4
- [all](#valdef-border-limit-all), in § 5.1
- [auto](#valdef-border-image-width-auto), in § 4.3
- [bevel](#valdef-corner-shape-value-bevel), in § 3.7
- [border](#propdef-border), in § 2.4
- [border-block](#propdef-border-block), in § 2.4
- [border-block-clip](#propdef-border-block-clip), in § 5.2
- [border-block-color](#propdef-border-block-color), in § 2.1
- [border-block-end](#propdef-border-block-end), in § 2.4
- [border-block-end-clip](#propdef-border-block-end-clip), in § 5.2
- [border-block-end-color](#propdef-border-block-end-color), in § 2.1
- [border-block-end-radius](#propdef-border-block-end-radius), in § 3.6.1
- [border-block-end-style](#propdef-border-block-end-style), in § 2.2
- [border-block-end-width](#propdef-border-block-end-width), in § 2.3
- [border-block-start](#propdef-border-block-start), in § 2.4
- [border-block-start-clip](#propdef-border-block-start-clip), in § 5.2
- [border-block-start-color](#propdef-border-block-start-color), in § 2.1
- [border-block-start-radius](#propdef-border-block-start-radius), in § 3.6.1
- [border-block-start-style](#propdef-border-block-start-style), in § 2.2
- [border-block-start-width](#propdef-border-block-start-width), in § 2.3
- [border-block-style](#propdef-border-block-style), in § 2.2
- [border-block-width](#propdef-border-block-width), in § 2.3
- [border-bottom](#propdef-border-bottom), in § 2.4
- [border-bottom-clip](#propdef-border-bottom-clip), in § 5.2
- [border-bottom-color](#propdef-border-bottom-color), in § 2.1
- [border-bottom-left-radius](#propdef-border-bottom-left-radius), in § 3.5
- [border-bottom-radius](#propdef-border-bottom-radius), in § 3.6.1
- [border-bottom-right-radius](#propdef-border-bottom-right-radius), in § 3.5
- [border-bottom-style](#propdef-border-bottom-style), in § 2.2
- [border-bottom-width](#propdef-border-bottom-width), in § 2.3
- [border-clip](#propdef-border-clip), in § 5.2
- [border color](#border-color-dfn), in § 2.1
- [border-color](#propdef-border-color), in § 2.1
- [border contour path](#border-contour-path), in § 3.9.4
- [border-end-end-radius](#propdef-border-end-end-radius), in § 3.5
- [border-end-start-radius](#propdef-border-end-start-radius), in § 3.5
- [border image](#border-image-dfn), in § 4.1
- [border-image](#propdef-border-image), in § 4.7
- [border image area](#border-image-area), in § 4.3
- [border-image-outset](#propdef-border-image-outset), in § 4.4
- [border image region](#border-image-region), in § 4.3
- [border-image-repeat](#propdef-border-image-repeat), in § 4.5
- [border-image-slice](#propdef-border-image-slice), in § 4.2
- [border-image-source](#propdef-border-image-source), in § 4.1
- [border-image-width](#propdef-border-image-width), in § 4.3
- [border-inline](#propdef-border-inline), in § 2.4
- [border-inline-clip](#propdef-border-inline-clip), in § 5.2
- [border-inline-color](#propdef-border-inline-color), in § 2.1
- [border-inline-end](#propdef-border-inline-end), in § 2.4
- [border-inline-end-clip](#propdef-border-inline-end-clip), in § 5.2
- [border-inline-end-color](#propdef-border-inline-end-color), in § 2.1
- [border-inline-end-radius](#propdef-border-inline-end-radius), in § 3.6.1
- [border-inline-end-style](#propdef-border-inline-end-style), in § 2.2
- [border-inline-end-width](#propdef-border-inline-end-width), in § 2.3
- [border-inline-start](#propdef-border-inline-start), in § 2.4
- [border-inline-start-clip](#propdef-border-inline-start-clip), in § 5.2
- [border-inline-start-color](#propdef-border-inline-start-color), in § 2.1
- [border-inline-start-radius](#propdef-border-inline-start-radius), in § 3.6.1
- [border-inline-start-style](#propdef-border-inline-start-style), in § 2.2
- [border-inline-start-width](#propdef-border-inline-start-width), in § 2.3
- [border-inline-style](#propdef-border-inline-style), in § 2.2
- [border-inline-width](#propdef-border-inline-width), in § 2.3
- [border-left](#propdef-border-left), in § 2.4
- [border-left-clip](#propdef-border-left-clip), in § 5.2
- [border-left-color](#propdef-border-left-color), in § 2.1
- [border-left-radius](#propdef-border-left-radius), in § 3.6.1
- [border-left-style](#propdef-border-left-style), in § 2.2
- [border-left-width](#propdef-border-left-width), in § 2.3
- [border-limit](#propdef-border-limit), in § 5.1
- [\<border-radius\>](#typedef-border-radius), in § 3.5
- [border radius](#border-radii), in § 3.5
- [border-radius](#propdef-border-radius), in § 3.6.2
- [border-right](#propdef-border-right), in § 2.4
- [border-right-clip](#propdef-border-right-clip), in § 5.2
- [border-right-color](#propdef-border-right-color), in § 2.1
- [border-right-radius](#propdef-border-right-radius), in § 3.6.1
- [border-right-style](#propdef-border-right-style), in § 2.2
- [border-right-width](#propdef-border-right-width), in § 2.3
- [border-shape](#propdef-border-shape), in § 7.1
- [border-start-end-radius](#propdef-border-start-end-radius), in § 3.5
- [border-start-start-radius](#propdef-border-start-start-radius), in § 3.5
- [border style](#border-style-dfn), in § 2.2
- [border-style](#propdef-border-style), in § 2.2
- [border-top](#propdef-border-top), in § 2.4
- [border-top-clip](#propdef-border-top-clip), in § 5.2
- [border-top-color](#propdef-border-top-color), in § 2.1
- [border-top-left-radius](#propdef-border-top-left-radius), in § 3.5
- [border-top-radius](#propdef-border-top-radius), in § 3.6.1
- [border-top-right-radius](#propdef-border-top-right-radius), in § 3.5
- [border-top-style](#propdef-border-top-style), in § 2.2
- [border-top-width](#propdef-border-top-width), in § 2.3
- [border width](#border-width-dfn), in § 2.3
- [border-width](#propdef-border-width), in § 2.3
- [bottom](#valdef-border-limit-bottom), in § 5.1
- [box-shadow](#propdef-box-shadow), in § 6.6
- [box-shadow-blur](#propdef-box-shadow-blur), in § 6.3
- [box-shadow-color](#propdef-box-shadow-color), in § 6.1
- [box-shadow-offset](#propdef-box-shadow-offset), in § 6.2
- [box-shadow-position](#propdef-box-shadow-position), in § 6.5
- [box-shadow-spread](#propdef-box-shadow-spread), in § 6.4
- [clockwise quad](#clockwise-quad), in § 3.9.4
- [corner](#propdef-corner), in § 3.9.3
- [corner area](#corner-area), in § 3.7
- [corner-block-end](#propdef-corner-block-end), in § 3.9.2
- [corner-block-end-shape](#propdef-corner-block-end-shape), in § 3.8.1
- [corner-block-start](#propdef-corner-block-start), in § 3.9.2
- [corner-block-start-shape](#propdef-corner-block-start-shape), in § 3.8.1
- [corner-bottom](#propdef-corner-bottom), in § 3.9.2
- [corner-bottom-left](#propdef-corner-bottom-left), in § 3.9.1
- [corner-bottom-left-shape](#propdef-corner-bottom-left-shape), in § 3.7
- [corner-bottom-right](#propdef-corner-bottom-right), in § 3.9.1
- [corner-bottom-right-shape](#propdef-corner-bottom-right-shape), in § 3.7
- [corner-bottom-shape](#propdef-corner-bottom-shape), in § 3.8.1
- [corner-end-end](#propdef-corner-end-end), in § 3.9.1
- [corner-end-end-shape](#propdef-corner-end-end-shape), in § 3.7
- [corner-end-start](#propdef-corner-end-start), in § 3.9.1
- [corner-end-start-shape](#propdef-corner-end-start-shape), in § 3.7
- [corner-inline-end](#propdef-corner-inline-end), in § 3.9.2
- [corner-inline-end-shape](#propdef-corner-inline-end-shape), in § 3.8.1
- [corner-inline-start](#propdef-corner-inline-start), in § 3.9.2
- [corner-inline-start-shape](#propdef-corner-inline-start-shape), in § 3.8.1
- [corner-left](#propdef-corner-left), in § 3.9.2
- [corner-left-shape](#propdef-corner-left-shape), in § 3.8.1
- [corner-right](#propdef-corner-right), in § 3.9.2
- [corner-right-shape](#propdef-corner-right-shape), in § 3.8.1
- [corners](#valdef-border-limit-corners), in § 5.1
- [corner shape](#corner-shape), in § 3.7
- [corner-shape](#propdef-corner-shape), in § 3.8.2
- [\<corner-shape-value\>](#typedef-corner-shape-value), in § 3.7
- [corner-start-end](#propdef-corner-start-end), in § 3.9.1
- [corner-start-end-shape](#propdef-corner-start-end-shape), in § 3.7
- [corner-start-start](#propdef-corner-start-start), in § 3.9.1
- [corner-start-start-shape](#propdef-corner-start-start-shape), in § 3.7
- [corner-top](#propdef-corner-top), in § 3.9.2
- [corner-top-left](#propdef-corner-top-left), in § 3.9.1
- [corner-top-left-shape](#propdef-corner-top-left-shape), in § 3.7
- [corner-top-right](#propdef-corner-top-right), in § 3.9.1
- [corner-top-right-shape](#propdef-corner-top-right-shape), in § 3.7
- [corner-top-shape](#propdef-corner-top-shape), in § 3.8.1
- [dashed](#valdef-line-style-dashed), in § 2.2
- [dotted](#valdef-line-style-dotted), in § 2.2
- [double](#valdef-line-style-double), in § 2.2
- [fill](#border-image-slice-fill), in § 4.2
- [groove](#valdef-line-style-groove), in § 2.2
- [hidden](#valdef-line-style-hidden), in § 2.2
- [horizontal offset](#valdef-box-shadow-offset-horizontal-offset), in § 6.2
- [inner box-shadow](#valdef-box-shadow-position-inner-box-shadow), in § 6.5
- [inner contour](#inner-contour), in § 3.9.4
- [inner shadow](#valdef-box-shadow-position-inner-box-shadow), in § 6.5
- inset
  - [value for \<line-style\>, border-style, border-top-style, border-left-style, border-bottom-style, border-right-style, border](#valdef-line-style-inset), in § 2.2
  - [value for box-shadow-position](#valdef-box-shadow-position-inset), in § 6.5
- [left](#valdef-border-limit-left), in § 5.1
- [\<legacy-border-radius-syntax\>](#typedef-legacy-border-radius-syntax), in § 3.5
- [\<length-percentage \[0,∞\]\>](#valdef-border-image-width-length-percentage-0), in § 4.3
- [\<line-style\>](#typedef-line-style), in § 2.2
- [\<line-width\>](#typedef-line-width), in § 2.3
- [medium](#valdef-line-width-medium), in § 2.3
- none
  - [value for \<line-style\>, border-style, border-top-style, border-left-style, border-bottom-style, border-right-style, border](#valdef-line-style-none), in § 2.2
  - [value for box-shadow-offset](#shadow-offset-none), in § 6.2
- [normalized inner corner hull](#normalized-inner-corner-hull), in § 3.9.6
- [normalized superellipse half corner](#normalized-superellipse-half-corner), in § 3.9.6
- [notch](#valdef-corner-shape-value-notch), in § 3.7
- \<number \[0,∞\]\>
  - [value for border-image-slice](#valdef-border-image-slice-number-0), in § 4.2
  - [value for border-image-width](#valdef-border-image-width-number-0), in § 4.3
- [opposite corner scale factor](#opposite-corner-scale-factor), in § 3.9.5
- [outer box-shadow](#valdef-box-shadow-position-outer-box-shadow), in § 6.5
- [outer contour](#outer-contour), in § 3.9.4
- [outer shadow](#valdef-box-shadow-position-outer-box-shadow), in § 6.5
- outset
  - [value for \<line-style\>, border-style, border-top-style, border-left-style, border-bottom-style, border-right-style, border](#valdef-line-style-outset), in § 2.2
  - [value for box-shadow-position](#valdef-box-shadow-position-outset), in § 6.5
- [outset-adjusted border radius](#outset-adjusted-border-radius), in § 3
- [\<percentage \[0,∞\]\>](#valdef-border-image-slice-percentage-0), in § 4.2
- [region](#border-image-region), in § 4.3
- [relevant side for border shape](#relevant-side-for-border-shape), in § 7.1
- [repeat](#valdef-border-image-repeat-repeat), in § 4.5
- [ridge](#valdef-line-style-ridge), in § 2.2
- [right](#valdef-border-limit-right), in § 5.1
- round
  - [value for \<corner-shape-value\>, corner-shape](#valdef-corner-shape-value-round), in § 3.7
  - [value for border-image-repeat](#valdef-border-image-repeat-round), in § 4.5
- [scoop](#valdef-corner-shape-value-scoop), in § 3.7
- [sides](#valdef-border-limit-sides), in § 5.1
- [\<slash-separated-border-radius-syntax\>](#typedef-slash-separated-border-radius-syntax), in § 3.5
- [solid](#valdef-line-style-solid), in § 2.2
- [space](#valdef-border-image-repeat-space), in § 4.5
- [\<spread-shadow\>](#typedef-spread-shadow), in § 6.6
- [square](#valdef-corner-shape-value-square), in § 3.7
- [squircle](#valdef-corner-shape-value-squircle), in § 3.7
- [stretch](#valdef-border-image-repeat-stretch), in § 4.5
- [superellipse()](#funcdef-superellipse), in § 3.7
- [superellipse interpolation](#superellipse-interpolation), in § 3.9.6
- [superellipse(K)](#valdef-corner-shape-value-superellipse-k), in § 3.7
- [superellipse parameter](#superellipse-parameter), in § 3.7
- [thick](#valdef-line-width-thick), in § 2.3
- [thin](#valdef-line-width-thin), in § 2.3
- [top](#valdef-border-limit-top), in § 5.1
- [vertical offset](#valdef-box-shadow-offset-vertical-offset), in § 6.2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-BOX-4\] defines the following terms:
  - <a id="30e036e4"></a>border
  - <a id="69de5ee0"></a>border area
  - <a id="85c399c0"></a>border box
  - <a id="3e6781f5"></a>border edge
  - <a id="cba8daea"></a>content edge
  - <a id="6e7a78f3"></a>edge
  - <a id="253362bb"></a>margin
  - <a id="0f70e5fd"></a>margin area
  - <a id="16ff1cf8"></a>margin edge
  - <a id="a2be8c84"></a>padding
  - <a id="15e1e804"></a>padding box
  - <a id="093a0ff1"></a>padding edge
  - <a id="90a1affb"></a>unshaped edge
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="a0542bba"></a>box-decoration-break
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="6b448e93"></a>initial value
  - <a id="980ac56a"></a>shorthand property
  - <a id="1a2b1083"></a>used value
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="bcdf9b19"></a>color
  - <a id="a42c65ac"></a>currentcolor
  - <a id="96e27c16"></a>transparent
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="d04b6986"></a>\<color\>
- \[CSS-DISPLAY-4\] defines the following terms:
  - <a id="38e3f81d"></a>inline-table
  - <a id="7a605ac8"></a>principal box
  - <a id="a9db5d6d"></a>replaced element
  - <a id="b9611667"></a>table
  - <a id="da0def5c"></a>table-cell
- \[CSS-GRID-2\] defines the following terms:
  - <a id="b1382fd7"></a>\<flex\>
  - <a id="ca502323"></a>fr
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="35bf32f2"></a>\<image\>
  - <a id="c82a1380"></a>default object size
  - <a id="487e1aa9"></a>natural dimension
  - <a id="c0cc78c8"></a>natural size
  - <a id="e99a4517"></a>specified size
- \[CSS-IMAGES-4\] defines the following terms:
  - <a id="96bd9b1c"></a>\<image-1D\>
- \[CSS-MASKING-1\] defines the following terms:
  - <a id="26027e88"></a>\<geometry-box\>
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="00d2e365"></a>ink overflow
  - <a id="24c40f7e"></a>overflow
  - <a id="855a7562"></a>visible
- \[CSS-OVERFLOW-4\] defines the following terms:
  - <a id="c26bd888"></a>overflow clip edge
  - <a id="555f477c"></a>overflow-clip-margin
- \[CSS-PSEUDO-4\] defines the following terms:
  - <a id="63b59bd9"></a>::first-letter
  - <a id="4bda66a9"></a>::first-line
- \[CSS-RUBY-1\] defines the following terms:
  - <a id="7015f3a0"></a>ruby annotation container
  - <a id="153743a1"></a>ruby base container
- \[CSS-SHAPES-1\] defines the following terms:
  - <a id="bff39085"></a>\<basic-shape\>
  - <a id="4601dbb8"></a>half-border-box
- \[CSS-SHAPES-2\] defines the following terms:
  - <a id="ad2bb926"></a>shape-inside
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="5ad01cca"></a>height
  - <a id="d8595fdb"></a>size
  - <a id="49731d1d"></a>width
- \[CSS-TEXT-4\] defines the following terms:
  - <a id="dd41a63b"></a>collapse
- \[CSS-TRANSFORMS-1\] defines the following terms:
  - <a id="8b2d56ad"></a>transformation matrix
- \[CSS-UI-4\] defines the following terms:
  - <a id="d96b668e"></a>outline-offset
  - <a id="61948517"></a>outline-width
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="c297b070"></a>\#
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="af4a190d"></a>+
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="128295ac"></a>\<percentage\>
  - <a id="d4441b24"></a>?
  - <a id="b35178c1"></a>coordinated value list
  - <a id="d8b99652"></a>coordinating list base property
  - <a id="d9c21938"></a>coordinating list property group
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="4f460096"></a>snap as a border width
  - <a id="3bafef5e"></a>{A,B}
  - <a id="8cbc2b3b"></a>{A}
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS-WRITING-MODES-3\] defines the following terms:
  - <a id="fb688f4f"></a>direction
- \[CSS-WRITING-MODES-4\] defines the following terms:
  - <a id="83d2ef35"></a>block-end
  - <a id="1118d052"></a>block-start
  - <a id="e112902f"></a>end
  - <a id="303c8d41"></a>flow-relative
  - <a id="4da3b716"></a>inline-end
  - <a id="0da67e16"></a>inline-start
  - <a id="e1f6e4b9"></a>physical
  - <a id="90c7548c"></a>start
  - <a id="8664e85f"></a>text-orientation
  - <a id="37bb38a0"></a>writing-mode
- \[CSS2\] defines the following terms:
  - <a id="d6bab061"></a>border-collapse
  - <a id="a22680a6"></a>outline
- \[CSS3BG\] defines the following terms:
  - <a id="a5c1f433"></a>background-clip
  - <a id="5ced56d0"></a>background-image
  - <a id="e316431d"></a>background-repeat
  - <a id="b942b432"></a>blur radius
  - <a id="8530ff4d"></a>inner box-shadow
  - <a id="46036d71"></a>outer box-shadow
  - <a id="369934cd"></a>round
  - <a id="a8af1d0e"></a>spread distance
- \[DOM\] defines the following terms:
  - <a id="27d9b7ea"></a>element
- \[GEOMETRY-1\] defines the following terms:
  - <a id="81c4ca96"></a>height dimension
  - <a id="b6d517b2"></a>quadrilateral
  - <a id="f8357697"></a>rectangle
  - <a id="7f877fbe"></a>width dimension
  - <a id="a190ce32"></a>x coordinate
  - <a id="39be403c"></a>y coordinate
- \[SELECTORS-4\] defines the following terms:
  - <a id="4d06fa38"></a>pseudo-element
- \[SVG2\] defines the following terms:
  - <a id="f18eaf1b"></a>basic shape

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 4 August 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. FPWD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Chris Lilley; Tab Atkins Jr.; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 24 April 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; et al. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 18 March 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-5&#x2F;](https://www.w3.org/TR/css-color-5/)

<a id="biblio-css-display-4"></a>\[CSS-DISPLAY-4\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 4](https://www.w3.org/TR/css-display-4/). 6 November 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-4&#x2F;](https://www.w3.org/TR/css-display-4/)

<a id="biblio-css-grid-2"></a>\[CSS-GRID-2\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 2](https://www.w3.org/TR/css-grid-2/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-2&#x2F;](https://www.w3.org/TR/css-grid-2/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Elika Etemad; Tab Atkins Jr.; Lea Verou. [CSS Images Module Level 4](https://www.w3.org/TR/css-images-4/). 30 September 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-masking-1"></a>\[CSS-MASKING-1\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 5 August 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 7 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-overflow-4"></a>\[CSS-OVERFLOW-4\]  
David Baron; Florian Rivoal; Elika Etemad. [CSS Overflow Module Level 4](https://www.w3.org/TR/css-overflow-4/). 21 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-4&#x2F;](https://www.w3.org/TR/css-overflow-4/)

<a id="biblio-css-pseudo-4"></a>\[CSS-PSEUDO-4\]  
Elika Etemad; Alan Stearns. [CSS Pseudo-Elements Module Level 4](https://www.w3.org/TR/css-pseudo-4/). 27 June 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-pseudo-4&#x2F;](https://www.w3.org/TR/css-pseudo-4/)

<a id="biblio-css-ruby-1"></a>\[CSS-RUBY-1\]  
Elika Etemad; et al. [CSS Ruby Annotation Layout Module Level 1](https://www.w3.org/TR/css-ruby-1/). 31 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ruby-1&#x2F;](https://www.w3.org/TR/css-ruby-1/)

<a id="biblio-css-shapes-1"></a>\[CSS-SHAPES-1\]  
Alan Stearns; Rossen Atanassov; Noam Rosenthal. [CSS Shapes Module Level 1](https://www.w3.org/TR/css-shapes-1/). 12 June 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-shapes-1&#x2F;](https://www.w3.org/TR/css-shapes-1/)

<a id="biblio-css-shapes-2"></a>\[CSS-SHAPES-2\]  
[CSS Shapes Module Level 2](https://drafts.csswg.org/css-shapes-2/). Editor's Draft. URL: [https&#x3A;&#x2F;&#x2F;drafts&#x2E;csswg&#x2E;org&#x2F;css-shapes-2&#x2F;](https://drafts.csswg.org/css-shapes-2/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-text-4"></a>\[CSS-TEXT-4\]  
Elika Etemad; et al. [CSS Text Module Level 4](https://www.w3.org/TR/css-text-4/). 29 May 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-4&#x2F;](https://www.w3.org/TR/css-text-4/)

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 16 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

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

<a id="biblio-css3bg"></a>\[CSS3BG\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-geometry-1"></a>\[GEOMETRY-1\]  
Simon Pieters; Chris Harrelson. [Geometry Interfaces Module Level 1](https://www.w3.org/TR/geometry-1/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;geometry-1&#x2F;](https://www.w3.org/TR/geometry-1/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-svg2"></a>\[SVG2\]  
Amelia Bellamy-Royds; et al. [Scalable Vector Graphics (SVG) 2](https://www.w3.org/TR/SVG2/). 4 October 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG2&#x2F;](https://www.w3.org/TR/SVG2/)

### <a id="informative"></a>Informative References

<a id="biblio-css-logical-1"></a>\[CSS-LOGICAL-1\]  
Rossen Atanassov; Elika Etemad. [CSS Logical Properties and Values Level 1](https://www.w3.org/TR/css-logical-1/). 27 August 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-logical-1&#x2F;](https://www.w3.org/TR/css-logical-1/)

<a id="biblio-css1"></a>\[CSS1\]  
Håkon Wium Lie; Bert Bos. [Cascading Style Sheets, level 1](https://www.w3.org/TR/CSS1/). 13 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS1&#x2F;](https://www.w3.org/TR/CSS1/)

<a id="biblio-css3grid"></a>\[CSS3GRID\]  
Tab Atkins Jr.; et al. [CSS Grid Layout Module Level 1](https://www.w3.org/TR/css-grid-1/). 26 March 2025. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-grid-1&#x2F;](https://www.w3.org/TR/css-grid-1/)

## <a id="property-index"></a>Property Index



| Name                | Value                                                                                                                                                                               | Initial                   | Applies to                                                                    | Inh.                      | %ages                                               | Anim­ation type                                                                  | Canonical order | Com­puted value                                                                                    | Logical property group |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------|-------------------------------------------------------------------------------|---------------------------|-----------------------------------------------------|---------------------------------------------------------------------------------|-----------------|---------------------------------------------------------------------------------------------------|------------------------|
| <strong><span><a id="ref-for-propdef-border⑧"></a></span><a href="#propdef-border">border</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-block"></a></span><a href="#propdef-border-block">border-block</a>&#xA;      </strong> | \<'border-block-start'\>                                                                                                                                                            | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-block-clip"></a></span><a href="#propdef-border-block-clip">border-block-clip</a>&#xA;      </strong> | \<'border-top-clip'\>                                                                                                                                                               | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-block-color"></a></span><a href="#propdef-border-block-color">border-block-color</a>&#xA;      </strong> | \<'border-top-color'\>{1,2}                                                                                                                                                         | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-block-end②"></a></span><a href="#propdef-border-block-end">border-block-end</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | See individual properties | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-block-end-clip①"></a></span><a href="#propdef-border-block-end-clip">border-block-end-clip</a>&#xA;      </strong> | none \| \[ \<length-percentage \[0,∞\]\> \| \<flex\> \]+                                                                                                                            | none                      | all elements                                                                  | no                        | refer to length of border-edge side                 | by computed value                                                               | per grammar     | none, or a list consisting of absolute lengths, or percentages as specified                       | border-clip            |
| <strong><span><a id="ref-for-propdef-border-block-end-color②"></a></span><a href="#propdef-border-block-end-color">border-block-end-color</a>&#xA;      </strong> | \<color\> \| \<image-1D\>                                                                                                                                                           | currentcolor              | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see prose                                                                       | per grammar     | the computed color and/or a one-dimensional image function                                        | border-color           |
| <strong><span><a id="ref-for-propdef-border-block-end-radius②"></a></span><a href="#propdef-border-block-end-radius">border-block-end-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,2} \[ / \<length-percentage \[0,∞\]\>{1,2} \]?                                                                                                      | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-block-end-style③"></a></span><a href="#propdef-border-block-end-style">border-block-end-style</a>&#xA;      </strong> | \<line-style\>                                                                                                                                                                      | none                      | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | discrete                                                                        | per grammar     | specified keyword                                                                                 | border-style           |
| <strong><span><a id="ref-for-propdef-border-block-end-width②"></a></span><a href="#propdef-border-block-end-width">border-block-end-width</a>&#xA;      </strong> | \<line-width\>                                                                                                                                                                      | medium                    | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | by computed value                                                               | per grammar     | absolute length, snapped as a border width; zero if the border style is none or hidden            | border-width           |
| <strong><span><a id="ref-for-propdef-border-block-start③"></a></span><a href="#propdef-border-block-start">border-block-start</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | See individual properties | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-block-start-clip②"></a></span><a href="#propdef-border-block-start-clip">border-block-start-clip</a>&#xA;      </strong> | none \| \[ \<length-percentage \[0,∞\]\> \| \<flex\> \]+                                                                                                                            | none                      | all elements                                                                  | no                        | refer to length of border-edge side                 | by computed value                                                               | per grammar     | none, or a list consisting of absolute lengths, or percentages as specified                       | border-clip            |
| <strong><span><a id="ref-for-propdef-border-block-start-color②"></a></span><a href="#propdef-border-block-start-color">border-block-start-color</a>&#xA;      </strong> | \<color\> \| \<image-1D\>                                                                                                                                                           | currentcolor              | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see prose                                                                       | per grammar     | the computed color and/or a one-dimensional image function                                        | border-color           |
| <strong><span><a id="ref-for-propdef-border-block-start-radius②"></a></span><a href="#propdef-border-block-start-radius">border-block-start-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,2} \[ / \<length-percentage \[0,∞\]\>{1,2} \]?                                                                                                      | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-block-start-style③"></a></span><a href="#propdef-border-block-start-style">border-block-start-style</a>&#xA;      </strong> | \<line-style\>                                                                                                                                                                      | none                      | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | discrete                                                                        | per grammar     | specified keyword                                                                                 | border-style           |
| <strong><span><a id="ref-for-propdef-border-block-start-width②"></a></span><a href="#propdef-border-block-start-width">border-block-start-width</a>&#xA;      </strong> | \<line-width\>                                                                                                                                                                      | medium                    | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | by computed value                                                               | per grammar     | absolute length, snapped as a border width; zero if the border style is none or hidden            | border-width           |
| <strong><span><a id="ref-for-propdef-border-block-style"></a></span><a href="#propdef-border-block-style">border-block-style</a>&#xA;      </strong> | \<'border-top-style'\>{1,2}                                                                                                                                                         | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-block-width"></a></span><a href="#propdef-border-block-width">border-block-width</a>&#xA;      </strong> | \<'border-top-width'\>{1,2}                                                                                                                                                         | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-bottom①"></a></span><a href="#propdef-border-bottom">border-bottom</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | See individual properties | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-bottom-clip"></a></span><a href="#propdef-border-bottom-clip">border-bottom-clip</a>&#xA;      </strong> | none \| \[ \<length-percentage \[0,∞\]\> \| \<flex\> \]+                                                                                                                            | none                      | all elements                                                                  | no                        | refer to length of border-edge side                 | by computed value                                                               | per grammar     | none, or a list consisting of absolute lengths, or percentages as specified                       | border-clip            |
| <strong><span><a id="ref-for-propdef-border-bottom-color①"></a></span><a href="#propdef-border-bottom-color">border-bottom-color</a>&#xA;      </strong> | \<color\> \| \<image-1D\>                                                                                                                                                           | currentcolor              | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see prose                                                                       | per grammar     | the computed color and/or a one-dimensional image function                                        | border-color           |
| <strong><span><a id="ref-for-propdef-border-bottom-left-radius③"></a></span><a href="#propdef-border-bottom-left-radius">border-bottom-left-radius</a>&#xA;      </strong> | \<border-radius\>                                                                                                                                                                   | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | by computed value                                                               | per grammar     | pair of computed \<length-percentage\> values                                                     | border-radius          |
| <strong><span><a id="ref-for-propdef-border-bottom-radius②"></a></span><a href="#propdef-border-bottom-radius">border-bottom-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,2} \[ / \<length-percentage \[0,∞\]\>{1,2} \]?                                                                                                      | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-bottom-right-radius④"></a></span><a href="#propdef-border-bottom-right-radius">border-bottom-right-radius</a>&#xA;      </strong> | \<border-radius\>                                                                                                                                                                   | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | by computed value                                                               | per grammar     | pair of computed \<length-percentage\> values                                                     | border-radius          |
| <strong><span><a id="ref-for-propdef-border-bottom-style①"></a></span><a href="#propdef-border-bottom-style">border-bottom-style</a>&#xA;      </strong> | \<line-style\>                                                                                                                                                                      | none                      | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | discrete                                                                        | per grammar     | specified keyword                                                                                 | border-style           |
| <strong><span><a id="ref-for-propdef-border-bottom-width①"></a></span><a href="#propdef-border-bottom-width">border-bottom-width</a>&#xA;      </strong> | \<line-width\>                                                                                                                                                                      | medium                    | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | by computed value                                                               | per grammar     | absolute length, snapped as a border width; zero if the border style is none or hidden            | border-width           |
| <strong><span><a id="ref-for-propdef-border-clip③"></a></span><a href="#propdef-border-clip">border-clip</a>&#xA;      </strong> | \<'border-top-clip'\>                                                                                                                                                               | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-color⑨"></a></span><a href="#propdef-border-color">border-color</a>&#xA;      </strong> | \[ \<color\> \| \<image-1D\> \]{1,4}                                                                                                                                                | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-end-end-radius①"></a></span><a href="#propdef-border-end-end-radius">border-end-end-radius</a>&#xA;      </strong> | \<border-radius\>                                                                                                                                                                   | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | by computed value                                                               | per grammar     | pair of computed \<length-percentage\> values                                                     | border-radius          |
| <strong><span><a id="ref-for-propdef-border-end-start-radius①"></a></span><a href="#propdef-border-end-start-radius">border-end-start-radius</a>&#xA;      </strong> | \<border-radius\>                                                                                                                                                                   | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | by computed value                                                               | per grammar     | pair of computed \<length-percentage\> values                                                     | border-radius          |
| <strong><span><a id="ref-for-propdef-border-image⑧"></a></span><a href="#propdef-border-image">border-image</a>&#xA;      </strong> | \<'border-image-source'\> \|\| \<'border-image-slice'\> \[ / \<'border-image-width'\> \| / \<'border-image-width'\>? / \<'border-image-outset'\> \]? \|\| \<'border-image-repeat'\> | See individual properties | See individual properties                                                     | no                        | N/A                                                 | See individual properties                                                       | per grammar     | See individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-image-outset④"></a></span><a href="#propdef-border-image-outset">border-image-outset</a>&#xA;      </strong> | \[ \<length \[0,∞\]\> \| \<number \[0,∞\]\> \]{1,4}                                                                                                                                 | 0                         | All elements, except internal table elements when border-collapse is collapse | no                        | N/A                                                 | by computed value                                                               | per grammar     | four values, each a number or absolute length                                                     |                        |
| <strong><span><a id="ref-for-propdef-border-image-repeat④"></a></span><a href="#propdef-border-image-repeat">border-image-repeat</a>&#xA;      </strong> | \[ stretch \| repeat \| round \| space \]{1,2}                                                                                                                                      | stretch                   | All elements, except internal table elements when border-collapse is collapse | no                        | N/A                                                 | discrete                                                                        | per grammar     | two keywords, one per axis                                                                        |                        |
| <strong><span><a id="ref-for-propdef-border-image-slice⑧"></a></span><a href="#propdef-border-image-slice">border-image-slice</a>&#xA;      </strong> | \[\<number \[0,∞\]\> \| \<percentage \[0,∞\]\>\]{1,4} &#x26;&#x26; fill?                                                                                          | 100%                      | All elements, except internal table elements when border-collapse is collapse | no                        | refer to size of the border image                   | by computed value                                                               | per grammar     | four values, each either a number or percentage; plus a fill keyword if specified                 |                        |
| <strong><span><a id="ref-for-propdef-border-image-source⑦"></a></span><a href="#propdef-border-image-source">border-image-source</a>&#xA;      </strong> | none \| \<image\>                                                                                                                                                                   | none                      | All elements, except internal table elements when border-collapse is collapse | no                        | N/A                                                 | discrete                                                                        | per grammar     | the keyword none or the computed \<image\>                                                        |                        |
| <strong><span><a id="ref-for-propdef-border-image-width①③"></a></span><a href="#propdef-border-image-width">border-image-width</a>&#xA;      </strong> | \[ \<length-percentage \[0,∞\]\> \| \<number \[0,∞\]\> \| auto \]{1,4}                                                                                                              | 1                         | All elements, except internal table elements when border-collapse is collapse | no                        | Relative to width/height of the border image area   | by computed value                                                               | per grammar     | four values, each either a number, the keyword auto, or a computed \<length-percentage\> value    |                        |
| <strong><span><a id="ref-for-propdef-border-inline"></a></span><a href="#propdef-border-inline">border-inline</a>&#xA;      </strong> | \<'border-block-start'\>                                                                                                                                                            | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-inline-clip"></a></span><a href="#propdef-border-inline-clip">border-inline-clip</a>&#xA;      </strong> | \<'border-top-clip'\>                                                                                                                                                               | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-inline-color"></a></span><a href="#propdef-border-inline-color">border-inline-color</a>&#xA;      </strong> | \<'border-top-color'\>{1,2}                                                                                                                                                         | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-inline-end②"></a></span><a href="#propdef-border-inline-end">border-inline-end</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | See individual properties | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-inline-end-clip①"></a></span><a href="#propdef-border-inline-end-clip">border-inline-end-clip</a>&#xA;      </strong> | none \| \[ \<length-percentage \[0,∞\]\> \| \<flex\> \]+                                                                                                                            | none                      | all elements                                                                  | no                        | refer to length of border-edge side                 | by computed value                                                               | per grammar     | none, or a list consisting of absolute lengths, or percentages as specified                       | border-clip            |
| <strong><span><a id="ref-for-propdef-border-inline-end-color②"></a></span><a href="#propdef-border-inline-end-color">border-inline-end-color</a>&#xA;      </strong> | \<color\> \| \<image-1D\>                                                                                                                                                           | currentcolor              | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see prose                                                                       | per grammar     | the computed color and/or a one-dimensional image function                                        | border-color           |
| <strong><span><a id="ref-for-propdef-border-inline-end-radius②"></a></span><a href="#propdef-border-inline-end-radius">border-inline-end-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,2} \[ / \<length-percentage \[0,∞\]\>{1,2} \]?                                                                                                      | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-inline-end-style③"></a></span><a href="#propdef-border-inline-end-style">border-inline-end-style</a>&#xA;      </strong> | \<line-style\>                                                                                                                                                                      | none                      | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | discrete                                                                        | per grammar     | specified keyword                                                                                 | border-style           |
| <strong><span><a id="ref-for-propdef-border-inline-end-width②"></a></span><a href="#propdef-border-inline-end-width">border-inline-end-width</a>&#xA;      </strong> | \<line-width\>                                                                                                                                                                      | medium                    | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | by computed value                                                               | per grammar     | absolute length, snapped as a border width; zero if the border style is none or hidden            | border-width           |
| <strong><span><a id="ref-for-propdef-border-inline-start②"></a></span><a href="#propdef-border-inline-start">border-inline-start</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | See individual properties | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-inline-start-clip①"></a></span><a href="#propdef-border-inline-start-clip">border-inline-start-clip</a>&#xA;      </strong> | none \| \[ \<length-percentage \[0,∞\]\> \| \<flex\> \]+                                                                                                                            | none                      | all elements                                                                  | no                        | refer to length of border-edge side                 | by computed value                                                               | per grammar     | none, or a list consisting of absolute lengths, or percentages as specified                       | border-clip            |
| <strong><span><a id="ref-for-propdef-border-inline-start-color②"></a></span><a href="#propdef-border-inline-start-color">border-inline-start-color</a>&#xA;      </strong> | \<color\> \| \<image-1D\>                                                                                                                                                           | currentcolor              | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see prose                                                                       | per grammar     | the computed color and/or a one-dimensional image function                                        | border-color           |
| <strong><span><a id="ref-for-propdef-border-inline-start-radius②"></a></span><a href="#propdef-border-inline-start-radius">border-inline-start-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,2} \[ / \<length-percentage \[0,∞\]\>{1,2} \]?                                                                                                      | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-inline-start-style③"></a></span><a href="#propdef-border-inline-start-style">border-inline-start-style</a>&#xA;      </strong> | \<line-style\>                                                                                                                                                                      | none                      | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | discrete                                                                        | per grammar     | specified keyword                                                                                 | border-style           |
| <strong><span><a id="ref-for-propdef-border-inline-start-width②"></a></span><a href="#propdef-border-inline-start-width">border-inline-start-width</a>&#xA;      </strong> | \<line-width\>                                                                                                                                                                      | medium                    | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | by computed value                                                               | per grammar     | absolute length, snapped as a border width; zero if the border style is none or hidden            | border-width           |
| <strong><span><a id="ref-for-propdef-border-inline-style"></a></span><a href="#propdef-border-inline-style">border-inline-style</a>&#xA;      </strong> | \<'border-top-style'\>{1,2}                                                                                                                                                         | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-inline-width"></a></span><a href="#propdef-border-inline-width">border-inline-width</a>&#xA;      </strong> | \<'border-top-width'\>{1,2}                                                                                                                                                         | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-left④"></a></span><a href="#propdef-border-left">border-left</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | See individual properties | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-left-clip"></a></span><a href="#propdef-border-left-clip">border-left-clip</a>&#xA;      </strong> | none \| \[ \<length-percentage \[0,∞\]\> \| \<flex\> \]+                                                                                                                            | none                      | all elements                                                                  | no                        | refer to length of border-edge side                 | by computed value                                                               | per grammar     | none, or a list consisting of absolute lengths, or percentages as specified                       | border-clip            |
| <strong><span><a id="ref-for-propdef-border-left-color①"></a></span><a href="#propdef-border-left-color">border-left-color</a>&#xA;      </strong> | \<color\> \| \<image-1D\>                                                                                                                                                           | currentcolor              | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see prose                                                                       | per grammar     | the computed color and/or a one-dimensional image function                                        | border-color           |
| <strong><span><a id="ref-for-propdef-border-left-radius②"></a></span><a href="#propdef-border-left-radius">border-left-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,2} \[ / \<length-percentage \[0,∞\]\>{1,2} \]?                                                                                                      | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-left-style①"></a></span><a href="#propdef-border-left-style">border-left-style</a>&#xA;      </strong> | \<line-style\>                                                                                                                                                                      | none                      | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | discrete                                                                        | per grammar     | specified keyword                                                                                 | border-style           |
| <strong><span><a id="ref-for-propdef-border-left-width①"></a></span><a href="#propdef-border-left-width">border-left-width</a>&#xA;      </strong> | \<line-width\>                                                                                                                                                                      | medium                    | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | by computed value                                                               | per grammar     | absolute length, snapped as a border width; zero if the border style is none or hidden            | border-width           |
| <strong><span><a id="ref-for-propdef-border-limit③"></a></span><a href="#propdef-border-limit">border-limit</a>&#xA;      </strong> | all \| \[ sides \| corners \] \<length-percentage \[0,∞\]\>? \| \[ top \| right \| bottom \| left \] \<length-percentage \[0,∞\]\>                                                  | all                       | all elements, except table element when border-collapse is collapse           | no                        | relative to border-box                              | discrete                                                                        | per grammar     | as specified                                                                                      |                        |
| <strong><span><a id="ref-for-propdef-border-radius②②"></a></span><a href="#propdef-border-radius">border-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,4} \[ / \<length-percentage \[0,∞\]\>{1,4} \]?                                                                                                      | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-right①"></a></span><a href="#propdef-border-right">border-right</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | See individual properties | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-right-clip"></a></span><a href="#propdef-border-right-clip">border-right-clip</a>&#xA;      </strong> | none \| \[ \<length-percentage \[0,∞\]\> \| \<flex\> \]+                                                                                                                            | none                      | all elements                                                                  | no                        | refer to length of border-edge side                 | by computed value                                                               | per grammar     | none, or a list consisting of absolute lengths, or percentages as specified                       | border-clip            |
| <strong><span><a id="ref-for-propdef-border-right-color①"></a></span><a href="#propdef-border-right-color">border-right-color</a>&#xA;      </strong> | \<color\> \| \<image-1D\>                                                                                                                                                           | currentcolor              | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see prose                                                                       | per grammar     | the computed color and/or a one-dimensional image function                                        | border-color           |
| <strong><span><a id="ref-for-propdef-border-right-radius②"></a></span><a href="#propdef-border-right-radius">border-right-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,2} \[ / \<length-percentage \[0,∞\]\>{1,2} \]?                                                                                                      | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-right-style①"></a></span><a href="#propdef-border-right-style">border-right-style</a>&#xA;      </strong> | \<line-style\>                                                                                                                                                                      | none                      | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | discrete                                                                        | per grammar     | specified keyword                                                                                 | border-style           |
| <strong><span><a id="ref-for-propdef-border-right-width①"></a></span><a href="#propdef-border-right-width">border-right-width</a>&#xA;      </strong> | \<line-width\>                                                                                                                                                                      | medium                    | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | by computed value                                                               | per grammar     | absolute length, snapped as a border width; zero if the border style is none or hidden            | border-width           |
| <strong><span><a id="ref-for-propdef-border-shape①①"></a></span><a href="#propdef-border-shape">border-shape</a>&#xA;      </strong> | none \| \[ \<basic-shape\> \<geometry-box\>?\]{1,2}                                                                                                                                 | none                      | all elements                                                                  | no                        | see prose                                           | by computed value                                                               | per grammar     | list, each item a computed color                                                                  |                        |
| <strong><span><a id="ref-for-propdef-border-start-end-radius①"></a></span><a href="#propdef-border-start-end-radius">border-start-end-radius</a>&#xA;      </strong> | \<border-radius\>                                                                                                                                                                   | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | by computed value                                                               | per grammar     | pair of computed \<length-percentage\> values                                                     | border-radius          |
| <strong><span><a id="ref-for-propdef-border-start-start-radius①"></a></span><a href="#propdef-border-start-start-radius">border-start-start-radius</a>&#xA;      </strong> | \<border-radius\>                                                                                                                                                                   | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | by computed value                                                               | per grammar     | pair of computed \<length-percentage\> values                                                     | border-radius          |
| <strong><span><a id="ref-for-propdef-border-style①①"></a></span><a href="#propdef-border-style">border-style</a>&#xA;      </strong> | \<'border-top-style'\>{1,4}                                                                                                                                                         | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-top①"></a></span><a href="#propdef-border-top">border-top</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | See individual properties | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-top-clip③"></a></span><a href="#propdef-border-top-clip">border-top-clip</a>&#xA;      </strong> | none \| \[ \<length-percentage \[0,∞\]\> \| \<flex\> \]+                                                                                                                            | none                      | all elements                                                                  | no                        | refer to length of border-edge side                 | by computed value                                                               | per grammar     | none, or a list consisting of absolute lengths, or percentages as specified                       | border-clip            |
| <strong><span><a id="ref-for-propdef-border-top-color②"></a></span><a href="#propdef-border-top-color">border-top-color</a>&#xA;      </strong> | \<color\> \| \<image-1D\>                                                                                                                                                           | currentcolor              | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | see prose                                                                       | per grammar     | the computed color and/or a one-dimensional image function                                        | border-color           |
| <strong><span><a id="ref-for-propdef-border-top-left-radius④"></a></span><a href="#propdef-border-top-left-radius">border-top-left-radius</a>&#xA;      </strong> | \<border-radius\>                                                                                                                                                                   | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | by computed value                                                               | per grammar     | pair of computed \<length-percentage\> values                                                     | border-radius          |
| <strong><span><a id="ref-for-propdef-border-top-radius③"></a></span><a href="#propdef-border-top-radius">border-top-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,2} \[ / \<length-percentage \[0,∞\]\>{1,2} \]?                                                                                                      | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-border-top-right-radius③"></a></span><a href="#propdef-border-top-right-radius">border-top-right-radius</a>&#xA;      </strong> | \<border-radius\>                                                                                                                                                                   | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | by computed value                                                               | per grammar     | pair of computed \<length-percentage\> values                                                     | border-radius          |
| <strong><span><a id="ref-for-propdef-border-top-style③"></a></span><a href="#propdef-border-top-style">border-top-style</a>&#xA;      </strong> | \<line-style\>                                                                                                                                                                      | none                      | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | discrete                                                                        | per grammar     | specified keyword                                                                                 | border-style           |
| <strong><span><a id="ref-for-propdef-border-top-width③"></a></span><a href="#propdef-border-top-width">border-top-width</a>&#xA;      </strong> | \<line-width\>                                                                                                                                                                      | medium                    | all elements except ruby base containers and ruby annotation containers       | no                        | N/A                                                 | by computed value                                                               | per grammar     | absolute length, snapped as a border width; zero if the border style is none or hidden            | border-width           |
| <strong><span><a id="ref-for-propdef-border-width①③"></a></span><a href="#propdef-border-width">border-width</a>&#xA;      </strong> | \<'border-top-width'\>{1,4}                                                                                                                                                         | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-box-shadow⑧"></a></span><a href="#propdef-box-shadow">box-shadow</a>&#xA;      </strong> | \<spread-shadow\>#                                                                                                                                                                  | none                      | all elements                                                                  | no                        | N/A                                                 | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-box-shadow-blur⑤"></a></span><a href="#propdef-box-shadow-blur">box-shadow-blur</a>&#xA;      </strong> | \<length \[0,∞\]\>#                                                                                                                                                                 | 0                         | all elements                                                                  | no                        | N/A                                                 | by computed value                                                               | per grammar     | list, each item a \<length\>                                                                      |                        |
| <strong><span><a id="ref-for-propdef-box-shadow-color⑤"></a></span><a href="#propdef-box-shadow-color">box-shadow-color</a>&#xA;      </strong> | \<color\>#                                                                                                                                                                          | currentcolor              | all elements                                                                  | no                        | N/A                                                 | by computed value                                                               | per grammar     | list, each item a computed color                                                                  |                        |
| <strong><span><a id="ref-for-propdef-box-shadow-offset⑦"></a></span><a href="#propdef-box-shadow-offset">box-shadow-offset</a>&#xA;      </strong> | \[ none \| \<length\>{1,2} \]#                                                                                                                                                      | none                      | all elements                                                                  | no                        | N/A                                                 | by computed value, treating none as 0 0 when interpolated with non-none values. | per grammar     | list, each item either none or a pair of offsets (horizontal and vertical) from the element‘s box |                        |
| <strong><span><a id="ref-for-propdef-box-shadow-position⑤"></a></span><a href="#propdef-box-shadow-position">box-shadow-position</a>&#xA;      </strong> | \[ outset \| inset \]#                                                                                                                                                              | outset                    | all elements                                                                  | no                        | N/A                                                 | by computed value                                                               | per grammar     | list, each item one of the keywords                                                               |                        |
| <strong><span><a id="ref-for-propdef-box-shadow-spread⑥"></a></span><a href="#propdef-box-shadow-spread">box-shadow-spread</a>&#xA;      </strong> | \<length\>#                                                                                                                                                                         | 0                         | all elements                                                                  | no                        | N/A                                                 | by computed value                                                               | per grammar     | list, each item a \<length\>                                                                      |                        |
| <strong><span><a id="ref-for-propdef-corner⑤"></a></span><a href="#propdef-corner">corner</a>&#xA;      </strong> | \<'border-radius'\> \|\| \<'corner-shape'\>                                                                                                                                         | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-block-end①"></a></span><a href="#propdef-corner-block-end">corner-block-end</a>&#xA;      </strong> | \<'border-top-radius'\> \|\| \<'corner-top-shape'\>                                                                                                                                 | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-block-end-shape"></a></span><a href="#propdef-corner-block-end-shape">corner-block-end-shape</a>&#xA;      </strong> | \<'corner-top-left-shape'\>{1,2}                                                                                                                                                    | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-block-start①"></a></span><a href="#propdef-corner-block-start">corner-block-start</a>&#xA;      </strong> | \<'border-top-radius'\> \|\| \<'corner-top-shape'\>                                                                                                                                 | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-block-start-shape②"></a></span><a href="#propdef-corner-block-start-shape">corner-block-start-shape</a>&#xA;      </strong> | \<'corner-top-left-shape'\>{1,2}                                                                                                                                                    | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-bottom①"></a></span><a href="#propdef-corner-bottom">corner-bottom</a>&#xA;      </strong> | \<'border-top-radius'\> \|\| \<'corner-top-shape'\>                                                                                                                                 | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-bottom-left①"></a></span><a href="#propdef-corner-bottom-left">corner-bottom-left</a>&#xA;      </strong> | \<'border-top-left-radius'\> \|\| \<'corner-top-left-shape'\>                                                                                                                       | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-bottom-left-shape"></a></span><a href="#propdef-corner-bottom-left-shape">corner-bottom-left-shape</a>&#xA;      </strong> | \<corner-shape-value\>                                                                                                                                                              | round                     | all elements where border-radius can apply                                    | no                        | n/a                                                 | see superellipse interpolation                                                  | per grammar     | the corresponding superellipse() value                                                            | corner-shape           |
| <strong><span><a id="ref-for-propdef-corner-bottom-right①"></a></span><a href="#propdef-corner-bottom-right">corner-bottom-right</a>&#xA;      </strong> | \<'border-top-left-radius'\> \|\| \<'corner-top-left-shape'\>                                                                                                                       | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-bottom-right-shape②"></a></span><a href="#propdef-corner-bottom-right-shape">corner-bottom-right-shape</a>&#xA;      </strong> | \<corner-shape-value\>                                                                                                                                                              | round                     | all elements where border-radius can apply                                    | no                        | n/a                                                 | see superellipse interpolation                                                  | per grammar     | the corresponding superellipse() value                                                            | corner-shape           |
| <strong><span><a id="ref-for-propdef-corner-bottom-shape"></a></span><a href="#propdef-corner-bottom-shape">corner-bottom-shape</a>&#xA;      </strong> | \<'corner-top-left-shape'\>{1,2}                                                                                                                                                    | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-end-end①"></a></span><a href="#propdef-corner-end-end">corner-end-end</a>&#xA;      </strong> | \<'border-top-left-radius'\> \|\| \<'corner-top-left-shape'\>                                                                                                                       | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-end-end-shape"></a></span><a href="#propdef-corner-end-end-shape">corner-end-end-shape</a>&#xA;      </strong> | \<corner-shape-value\>                                                                                                                                                              | round                     | all elements where border-radius can apply                                    | no                        | n/a                                                 | see superellipse interpolation                                                  | per grammar     | the corresponding superellipse() value                                                            | corner-shape           |
| <strong><span><a id="ref-for-propdef-corner-end-start①"></a></span><a href="#propdef-corner-end-start">corner-end-start</a>&#xA;      </strong> | \<'border-top-left-radius'\> \|\| \<'corner-top-left-shape'\>                                                                                                                       | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-end-start-shape"></a></span><a href="#propdef-corner-end-start-shape">corner-end-start-shape</a>&#xA;      </strong> | \<corner-shape-value\>                                                                                                                                                              | round                     | all elements where border-radius can apply                                    | no                        | n/a                                                 | see superellipse interpolation                                                  | per grammar     | the corresponding superellipse() value                                                            | corner-shape           |
| <strong><span><a id="ref-for-propdef-corner-inline-end①"></a></span><a href="#propdef-corner-inline-end">corner-inline-end</a>&#xA;      </strong> | \<'border-top-radius'\> \|\| \<'corner-top-shape'\>                                                                                                                                 | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-inline-end-shape"></a></span><a href="#propdef-corner-inline-end-shape">corner-inline-end-shape</a>&#xA;      </strong> | \<'corner-top-left-shape'\>{1,2}                                                                                                                                                    | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-inline-start①"></a></span><a href="#propdef-corner-inline-start">corner-inline-start</a>&#xA;      </strong> | \<'border-top-radius'\> \|\| \<'corner-top-shape'\>                                                                                                                                 | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-inline-start-shape"></a></span><a href="#propdef-corner-inline-start-shape">corner-inline-start-shape</a>&#xA;      </strong> | \<'corner-top-left-shape'\>{1,2}                                                                                                                                                    | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-left①"></a></span><a href="#propdef-corner-left">corner-left</a>&#xA;      </strong> | \<'border-top-radius'\> \|\| \<'corner-top-shape'\>                                                                                                                                 | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-left-shape"></a></span><a href="#propdef-corner-left-shape">corner-left-shape</a>&#xA;      </strong> | \<'corner-top-left-shape'\>{1,2}                                                                                                                                                    | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-right①"></a></span><a href="#propdef-corner-right">corner-right</a>&#xA;      </strong> | \<'border-top-radius'\> \|\| \<'corner-top-shape'\>                                                                                                                                 | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-right-shape"></a></span><a href="#propdef-corner-right-shape">corner-right-shape</a>&#xA;      </strong> | \<'corner-top-left-shape'\>{1,2}                                                                                                                                                    | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-shape①⑦"></a></span><a href="#propdef-corner-shape">corner-shape</a>&#xA;      </strong> | \<'corner-top-left-shape'\>{1,4}                                                                                                                                                    | round                     | all elements where border-radius can apply                                    | no                        | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-start-end①"></a></span><a href="#propdef-corner-start-end">corner-start-end</a>&#xA;      </strong> | \<'border-top-left-radius'\> \|\| \<'corner-top-left-shape'\>                                                                                                                       | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-start-end-shape"></a></span><a href="#propdef-corner-start-end-shape">corner-start-end-shape</a>&#xA;      </strong> | \<corner-shape-value\>                                                                                                                                                              | round                     | all elements where border-radius can apply                                    | no                        | n/a                                                 | see superellipse interpolation                                                  | per grammar     | the corresponding superellipse() value                                                            | corner-shape           |
| <strong><span><a id="ref-for-propdef-corner-start-start①"></a></span><a href="#propdef-corner-start-start">corner-start-start</a>&#xA;      </strong> | \<'border-top-left-radius'\> \|\| \<'corner-top-left-shape'\>                                                                                                                       | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-start-start-shape②"></a></span><a href="#propdef-corner-start-start-shape">corner-start-start-shape</a>&#xA;      </strong> | \<corner-shape-value\>                                                                                                                                                              | round                     | all elements where border-radius can apply                                    | no                        | n/a                                                 | see superellipse interpolation                                                  | per grammar     | the corresponding superellipse() value                                                            | corner-shape           |
| <strong><span><a id="ref-for-propdef-corner-top①"></a></span><a href="#propdef-corner-top">corner-top</a>&#xA;      </strong> | \<'border-top-radius'\> \|\| \<'corner-top-shape'\>                                                                                                                                 | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-top-left①"></a></span><a href="#propdef-corner-top-left">corner-top-left</a>&#xA;      </strong> | \<'border-top-left-radius'\> \|\| \<'corner-top-left-shape'\>                                                                                                                       | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-top-left-shape⑥"></a></span><a href="#propdef-corner-top-left-shape">corner-top-left-shape</a>&#xA;      </strong> | \<corner-shape-value\>                                                                                                                                                              | round                     | all elements where border-radius can apply                                    | no                        | n/a                                                 | see superellipse interpolation                                                  | per grammar     | the corresponding superellipse() value                                                            | corner-shape           |
| <strong><span><a id="ref-for-propdef-corner-top-right①"></a></span><a href="#propdef-corner-top-right">corner-top-right</a>&#xA;      </strong> | \<'border-top-left-radius'\> \|\| \<'corner-top-left-shape'\>                                                                                                                       | 0                         | all elements (but see prose)                                                  | no                        | Refer to corresponding dimension of the border box. | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |
| <strong><span><a id="ref-for-propdef-corner-top-right-shape①"></a></span><a href="#propdef-corner-top-right-shape">corner-top-right-shape</a>&#xA;      </strong> | \<corner-shape-value\>                                                                                                                                                              | round                     | all elements where border-radius can apply                                    | no                        | n/a                                                 | see superellipse interpolation                                                  | per grammar     | the corresponding superellipse() value                                                            | corner-shape           |
| <strong><span><a id="ref-for-propdef-corner-top-shape③"></a></span><a href="#propdef-corner-top-shape">corner-top-shape</a>&#xA;      </strong> | \<'corner-top-left-shape'\>{1,2}                                                                                                                                                    | see individual properties | see individual properties                                                     | see individual properties | see individual properties                           | see individual properties                                                       | per grammar     | see individual properties                                                                         |                        |



## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Here are two proposals for doing this: the second one is from GCPM, the first one is an attempt to recast it more readably. The names are terrible, known problem, proposals accepted. There is a problem with conceiving this as clipping: if you have dotted borders, you want whole dots always, not parts of dots. So it should be a drawing limit, not a clip. [↵](#issue-76b7fb28)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> how should this affect clipping replaced elements? [↵](#issue-e4b44b73)
