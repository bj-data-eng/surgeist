Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Color Module Level 4](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium . W3C ® liability , trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Color Module Level 4

Source snapshot: https://www.w3.org/TR/2026/CRD-css-color-4-20260502/

Snapshot SHA-256: cfd58f190b6f1b873da18aac87f0c9dad816927dc4bbd23d504c9830d39d6250

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Source row-header labels in readable Markdown tables are bold; native HTML th/scope accessibility semantics are not expressible in GFM. Field/Definition headings, where used, are added non-normative presentation labels.
- 31 complex or multi-paragraph tables use source-checked readable field, case, grid or matrix layouts. Explicit header/span relationships and source cell mappings are retained; no raw HTML tables or flattened row/cell dumps remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Color Module Level 4

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-typedef-color"></a>

This specification describes CSS [\<color\>](#typedef-color) values, and properties for foreground color and group opacity. It also describes how colors are interpolated, and how to gamut map colors.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.&#xA;&#x9;A list of current W3C publications&#xA;&#x9;and the latest revision of this technical report&#xA;&#x9;can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-color” in the title, like this: “\[css-color\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-color%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

## <a id="introduction"></a>1.  Introduction

<em>This section is not normative.</em>

Tests

This section is not normative, it does not need tests.

------------------------------------------------------------------------

<a id="ref-for-typedef-color①"></a>

This module describes CSS properties which allow authors to specify the foreground color and opacity of the text content of an element. This module also describes in detail the CSS [\<color\>](#typedef-color) value type.

It not only defines the color-related properties and values that already exist in [CSS1](https://www.w3.org/TR/CSS1), [CSS2](https://www.w3.org/TR/CSS2/), and [CSS Color 3](https://www.w3.org/TR/css-color-3/), but also defines new properties and values.

<a id="ref-for-color-space"></a>

In particular, it allows specifying colors in other [color spaces](#color-space) than sRGB; previously, the more saturated colors outside the sRGB gamut could not be used in CSS even if the display device supported them.

A [draft implementation report](https://drafts.csswg.org/css-color-4/test-coverage) is available.

### <a id="values"></a>1.1.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units [\[CSS-VALUES-3\]](#biblio-css-values-3). Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="terminology"></a>2. Color Terminology

Tests

This section provides definitions used later, it does not need tests.

------------------------------------------------------------------------

A <a id="color"></a>color is a definition (numeric or textual) of the human visual perception of a light or a physical object illuminated with light. The objective study of human color perception is termed colorimetry.

The color of a physical object depends on how much light it reflects at each visible wavelength, plus the actual color of the light illuminating it (again, the amount of light at each wavelength). It is measured by a <em>spectrophotometer </em>.

The color of something that emits light (including colors on a computer screen) depends on how much light it emits at each visible wavelength. It is measured by a <em>spectroradiometer</em>.

If two objects have different spectra, but still produce the same physical sensation, we say they have the same color. We can calculate whether two colors are the same by converting the spectra to CIE XYZ (three numbers).

<a id="ref-for-calibrated"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-cal-leaf"></a>For example a green leaf, a photograph of that leaf displayed on a computer screen, and a print of that photograph, are all producing a green sensation by different means. If the screen and the printer are [calibrated](#calibrated), the green in the leaf, and the photo, and the print will look the same.

A <a id="color-space"></a>color space is an organization of colors with respect to an underlying colorimetric model, such that there is a clear, objectively-measurable meaning for any color in that color space. This also means that the same color can be expressed in multiple color spaces, or transformed from one color space to another, while still looking the same.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-leaf-spectro"></a>
>
> A leaf is measured with a spectrophotometer and found to have the color lch(51.2345% 21.2 130) which is lab(51.2345% -13.6271 16.2401).
>
> This same color could be expressed in various color spaces:
>
> ```css
>  color(sRGB 0.41587 0.503670 0.36664);
>  color(display-p3 0.43313 0.50108 0.37950);
>  color(a98-rgb 0.44091 0.49971 0.37408);
>  color(prophoto-rgb 0.36589 0.41717 0.31333);
>  color(rec2020 0.6295 0.9657 0.3633);
> ```
An <a id="additive-color-space"></a>additive color space means that the coordinate system is linear in light intensity. The CIE XYZ color space is an additive color space. The Y component of XYZ is the <a id="luminance"></a>luminance, the light intensity per unit area, or 'how bright it is'. Luminance is measured in candelas per square meter. cd/m², also called <em>nits</em>.

In an additive color space, calculations can be done to <em>accurately predict</em> color mixing. Most RGB spaces are not additive, because the components are <em>gamma encoded</em>. Undoing this gamma encoding produces linear-light values.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-additivity"></a> For example, if a light fixture contains two identical colored lights, and only one is switched on, and the color is measured to be color(xyz 0.13 0.12 0.04), then the color when both are switched on will be exactly twice that, color(xyz 0.26 0.24 0.08).
>
> If we have two differently colored spotlights shining on a stage, and one has the measured value color(xyz 0.15 0.24 0.17) while the other is color(xyz 0.11 0.06 0.06) then we can accurately predict that if the colored beams are made to overlap, the color of the mixture will be the sum of the XYZ component values, or color(xyz 0.26 0.30 0.23).

A <a id="chromaticity"></a>chromaticity is a color measurement where the lightness component has been factored out. From the identical lights example above, the <em>u',v'</em> chromaticity with one light is (0.2537, 0.5268) and the chromaticity is the same with both lights (they are the same color, it is just brighter).

Chromaticities are additive, so they accurately predict the chromaticity (but not the resulting lightness) of a mixture. Being two-dimensional, chromaticity is easily represented on a <em>chromaticity diagram</em> to predict the chromaticity of a color mixture. Any two colors can be mixed, and the resulting colors will lie on the line joining them on the diagram. Three colors form a plane, and the resulting colors will lie in the triangle they form on the diagram.

<a id="fig-ucs-display-p3"></a> ![uv chromaticity diagram of the display-p3 color space](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/UCS-display-p3.svg)

<a id="ref-for-valdef-color-display-p3"></a>

<a id="ref-for-valdef-color-srgb"></a>

A chromaticity diagram showing (in solid colors) the [display-p3](#valdef-color-display-p3) color space and for comparison (faded) the [sRGB](#valdef-color-srgb) color space. The white point (D65) is also shown.

Thus, once linearized, RGB color spaces are additive, and their gamut is defined by the chromaticities of the red, green and blue primaries, plus the chromaticity of the <a id="white-point"></a>white point (the color formed by all three primaries at full intensity).

<a id="ref-for-white-point"></a>

<a id="ref-for-d65"></a>

Most color spaces use one of a few daylight-simulating [white points](#white-point), which are named by the correlated color temperature (CCT) [\[Understanding_CCT\]](#biblio-understanding_cct) of the corresponding black-body radiator. For example, [D65](#d65) is a daylight whitepoint corresponding to a correlated color temperature of 6500 Kelvin (actually 6504, because the value of Plank’s constant has changed since the color was originally defined).

<a id="ref-for-white-point①"></a>

To avoid cumulative round-trip errors, it is important that the identical chromaticity values are used consistently, at all places in a calculation. Thus, for maximum compatibility, for this specification, the following two standard daylight-simulating [white points](#white-point) are defined:



| Name                   | x        | y        | CCT   |
|------------------------|----------|----------|-------|
| <a id="d50"></a>D50 | 0.345700 | 0.358500 | 5003K |
| <a id="d65"></a>D65 | 0.312700 | 0.329000 | 6504K |



<a id="ref-for-color-space①"></a>

When the measured physical characteristics (such as the chromaticities of the primary colors it uses, or the colors produced in response to a given set of inputs) of a [color space](#color-space) or a color-producing device are known, it is said to be <a id="characterized"></a>characterized.

If in addition adjustments have been made so that a device meets calibration targets such as white point, neutrality of greys, predictability and consistency of tone response, then it is said to be <a id="calibrated"></a>calibrated.

Real physical devices cannot yet produce every possible color that the human eye can see. The range of colors that a given device can produce is termed the <a id="gamut"></a>gamut <em>(not to be confused with gamma)</em>. Devices with a limited gamut cannot produce very saturated colors, like those found in a rainbow.

<a id="fig-three-gamuts"></a>

![](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/sRGB-DisplayP3-rec2020-in-Oklab.png)

A top-down view of three gamuts, plotted in Oklab with the positive a-axis towards the right and the positive b-axis towards the top; looking down the l-axis so white and neutrals are in the center. The largest of the three gamuts is ITU Rec BT.2020; the medium-sized one is Display P3, and the smallest is sRGB. Rendering by Alexey Ardov.

<a id="ref-for-color-space②"></a>

The gamuts of different [color space](#color-space)s may be compared by looking at the volume (in cubic Lab units) of colors that can be expressed. The following table examines the [predefined](#predefined) color spaces available in CSS.



| color space  | Volume (million Lab units) |
|--------------|----------------------------|
| sRGB         | 0.820                      |
| display-p3   | 1.233                      |
| a98-rgb      | 1.310                      |
| prophoto-rgb | 2.896                      |
| rec2020      | 2.042                      |



<a id="ref-for-valid-color"></a>

A color in CSS is either an <a id="invalid-color"></a>invalid color, as described below for each syntactic form, or a [valid color](#valid-color).

<a id="ref-for-invalid-color"></a>

Any color which is not an [invalid color](#invalid-color) is a <a id="valid-color"></a>valid color.

<a id="ref-for-valid-color①"></a>

A color may be a [valid color](#valid-color) but still be outside the range of colors that can be produced by an output device (a screen, projector, or printer)

It is said to be <a id="out-of-gamut"></a>out of gamut.

<a id="ref-for-valid-color②"></a>

<a id="ref-for-out-of-gamut"></a>

Each [valid color](#valid-color) is either <a id="in-gamut"></a>in-gamut for a particular output device (screen, or printer) or it is [out of gamut](#out-of-gamut).

<a id="ref-for-css-gamut-mapped"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-oog"></a> For example, given a screen which covers 100% of the display-p3 color space, but no more, the following color is out of gamut:
>
> ```text
>  color(prophoto-rgb 0.88 0.45 0.10)
> ```
>
> because, expressed in display-p3, one or more coordinates are either greater that 1.0 or less than 0.0:
>
> ```text
>  color(display-p3 1.0844 0.43 0.1)
> ```
>
> This color is valid, and could, for example, be used as a gradient stop, but would need to be [CSS gamut mapped](#css-gamut-mapped) for display, producing a similar-looking but lower chroma (less saturated) color.

## <a id="applying-color"></a>3. Applying Color in CSS

### <a id="accessibility"></a>3.1.  Accessibility and Conveying Information By Color

Tests

This section provides authoring guidance, it does not need tests.

------------------------------------------------------------------------

Although colors can add significant information to documents and make them more readable, color by itself should not be the sole means to convey important information. Authors should consider the W3C Web Content Accessibility Guidelines [\[WCAG21\]](#biblio-wcag21) when using color in their documents.

> [<em>1.4.1 Use of Color:</em> Color is not used as the only visual means of conveying information, indicating an action, prompting a response, or distinguishing a visual element](https://www.w3.org/TR/WCAG21/#use-of-color)

<a id="ref-for-propdef-color"></a>

### <a id="the-color-property"></a>3.2.  Foreground Color: the [color](#propdef-color) property



| Field               | Definition                                                            |
|---------------------|-----------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-color"></a>color                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-color②"></a>[\<color\>](#typedef-color)                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | CanvasText                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | all elements and text                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | computed color, see [resolving color values](#resolving-color-values) |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                |



Tests

- [color-001.html](https://wpt.fyi/results/css/css-color/color-001.html) [(live test)](http://wpt.live/css/css-color/color-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-001.html)
- [color-002.html](https://wpt.fyi/results/css/css-color/color-002.html) [(live test)](http://wpt.live/css/css-color/color-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-002.html)
- [color-003.html](https://wpt.fyi/results/css/css-color/color-003.html) [(live test)](http://wpt.live/css/css-color/color-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-003.html)
- [inheritance.html](https://wpt.fyi/results/css/css-color/inheritance.html) [(live test)](http://wpt.live/css/css-color/inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/inheritance.html)
- [color-interpolation.html](https://wpt.fyi/results/css/css-color/animation/color-interpolation.html) [(live test)](http://wpt.live/css/css-color/animation/color-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/animation/color-interpolation.html)
- [color-initial-canvastext.html](https://wpt.fyi/results/css/css-color/color-initial-canvastext.html) [(live test)](http://wpt.live/css/css-color/color-initial-canvastext.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-initial-canvastext.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)
- [color-invalid.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid.html)

<a id="ref-for-used-value"></a>

<a id="ref-for-propdef-border-color"></a>

<a id="ref-for-propdef-text-emphasis-color"></a>

This property specifies the primary foreground color of the element. This is used as the fill color of its text content, and in addition specifies the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) that currentcolor resolves to, which allows indirect references to this foreground color and affects the initial values of various other color properties such as [border-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-color) and [text-emphasis-color](https://www.w3.org/TR/css-text-decor-4/#propdef-text-emphasis-color).

<a id="ref-for-typedef-color③"></a>

[\<color\>](#typedef-color)

<a id="ref-for-typedef-color④"></a>

Sets the primary foreground color to the specified [\<color\>](#typedef-color).

<a id="ref-for-typedef-color⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-lime"></a> The [\<color\>](#typedef-color) type provides multiple ways to syntactically specify a given color. For example, the following declarations all specify the sRGB color “lime”:
>
> ```css
> em { color:  lime; }   /* color keyword  */
> em { color:  rgb(0 255 0); } /* RGB range 0-255   */
> em { color:  rgb(0% 100% 0%); } /* RGB range 0%-100% */
> em { color:  color(sRGB 0 1 0); } /* sRGB range 0.0-1.0 */
> ```
<a id="ref-for-valdef-color-currentcolor"></a>

When applied to text, this property, including its alpha component, has no effect on “color glyphs” (such as the emoji in some fonts), which are colored by a built-in palette. However, some colored fonts are able to refer to a contextual “foreground color”, such as by palette entry 0xFFFF in the `COLR` table of OpenType, or by the context-fill value in SVG-in-OpenType. In such cases, the foreground color is set by this property, identical to how it sets the [currentcolor](#valdef-color-currentcolor) value.

<a id="ref-for-propdef-opacity"></a>

### <a id="transparency"></a>3.3.  Transparency: the [opacity](#propdef-opacity) property

Opacity can be thought of as a postprocessing operation. Conceptually, after the element (including its descendants) is rendered into an RGBA offscreen image, the opacity setting specifies how to blend the offscreen rendering into the current composite rendering. See [simple alpha compositing](#alpha) for details.



| Field               | Definition                                                             |
|---------------------|------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-opacity"></a>opacity                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-opacity-opacity-value"></a>[\<opacity-value\>](#typedef-opacity-opacity-value) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 1                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | map to the range \[0,1\]                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified number, clamped to the range \[0,1\]                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value type                                                 |



Tests

- [clip-opacity-out-of-flow.html](https://wpt.fyi/results/css/css-color/clip-opacity-out-of-flow.html) [(live test)](http://wpt.live/css/css-color/clip-opacity-out-of-flow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/clip-opacity-out-of-flow.html)
- [t32-opacity-basic-0.0-a.xht](https://wpt.fyi/results/css/css-color/t32-opacity-basic-0.0-a.xht) [(live test)](http://wpt.live/css/css-color/t32-opacity-basic-0.0-a.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/t32-opacity-basic-0.0-a.xht)
- [t32-opacity-basic-0.6-a.xht](https://wpt.fyi/results/css/css-color/t32-opacity-basic-0.6-a.xht) [(live test)](http://wpt.live/css/css-color/t32-opacity-basic-0.6-a.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/t32-opacity-basic-0.6-a.xht)
- [t32-opacity-basic-1.0-a.xht](https://wpt.fyi/results/css/css-color/t32-opacity-basic-1.0-a.xht) [(live test)](http://wpt.live/css/css-color/t32-opacity-basic-1.0-a.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/t32-opacity-basic-1.0-a.xht)
- [t32-opacity-clamping-0.0-b.xht](https://wpt.fyi/results/css/css-color/t32-opacity-clamping-0.0-b.xht) [(live test)](http://wpt.live/css/css-color/t32-opacity-clamping-0.0-b.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/t32-opacity-clamping-0.0-b.xht)
- [t32-opacity-clamping-1.0-b.xht](https://wpt.fyi/results/css/css-color/t32-opacity-clamping-1.0-b.xht) [(live test)](http://wpt.live/css/css-color/t32-opacity-clamping-1.0-b.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/t32-opacity-clamping-1.0-b.xht)
- [t32-opacity-offscreen-b.xht](https://wpt.fyi/results/css/css-color/t32-opacity-offscreen-b.xht) [(live test)](http://wpt.live/css/css-color/t32-opacity-offscreen-b.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/t32-opacity-offscreen-b.xht)
- [t32-opacity-offscreen-multiple-boxes-1-c.xht](https://wpt.fyi/results/css/css-color/t32-opacity-offscreen-multiple-boxes-1-c.xht) [(live test)](http://wpt.live/css/css-color/t32-opacity-offscreen-multiple-boxes-1-c.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/t32-opacity-offscreen-multiple-boxes-1-c.xht)
- [t32-opacity-offscreen-multiple-boxes-2-c.xht](https://wpt.fyi/results/css/css-color/t32-opacity-offscreen-multiple-boxes-2-c.xht) [(live test)](http://wpt.live/css/css-color/t32-opacity-offscreen-multiple-boxes-2-c.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/t32-opacity-offscreen-multiple-boxes-2-c.xht)
- [t32-opacity-offscreen-with-alpha-c.xht](https://wpt.fyi/results/css/css-color/t32-opacity-offscreen-with-alpha-c.xht) [(live test)](http://wpt.live/css/css-color/t32-opacity-offscreen-with-alpha-c.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/t32-opacity-offscreen-with-alpha-c.xht)
- [t32-opacity-zorder-c.xht](https://wpt.fyi/results/css/css-color/t32-opacity-zorder-c.xht) [(live test)](http://wpt.live/css/css-color/t32-opacity-zorder-c.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/t32-opacity-zorder-c.xht)
- [opacity-computed.html](https://wpt.fyi/results/css/css-color/parsing/opacity-computed.html) [(live test)](http://wpt.live/css/css-color/parsing/opacity-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/opacity-computed.html)
- [opacity-valid.html](https://wpt.fyi/results/css/css-color/parsing/opacity-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/opacity-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/opacity-valid.html)
- [opacity-invalid.html](https://wpt.fyi/results/css/css-color/parsing/opacity-invalid.html) [(live test)](http://wpt.live/css/css-color/parsing/opacity-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/opacity-invalid.html)
- [composited-filters-under-opacity.html](https://wpt.fyi/results/css/css-color/composited-filters-under-opacity.html) [(live test)](http://wpt.live/css/css-color/composited-filters-under-opacity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/composited-filters-under-opacity.html)
- [filters-under-will-change-opacity.html](https://wpt.fyi/results/css/css-color/filters-under-will-change-opacity.html) [(live test)](http://wpt.live/css/css-color/filters-under-will-change-opacity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/filters-under-will-change-opacity.html)
- [color-composition.html](https://wpt.fyi/results/css/css-color/animation/color-composition.html) [(live test)](http://wpt.live/css/css-color/animation/color-composition.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/animation/color-composition.html)
- [opacity-interpolation.html](https://wpt.fyi/results/css/css-color/animation/opacity-interpolation.html) [(live test)](http://wpt.live/css/css-color/animation/opacity-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/animation/opacity-interpolation.html)
- [canvas-change-opacity.html](https://wpt.fyi/results/css/css-color/canvas-change-opacity.html) [(live test)](http://wpt.live/css/css-color/canvas-change-opacity.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/canvas-change-opacity.html)
- [opacity-animation-ending-correctly-001.html](https://wpt.fyi/results/css/css-color/animation/opacity-animation-ending-correctly-001.html) [(live test)](http://wpt.live/css/css-color/animation/opacity-animation-ending-correctly-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/animation/opacity-animation-ending-correctly-001.html)
- [opacity-animation-ending-correctly-002.html](https://wpt.fyi/results/css/css-color/animation/opacity-animation-ending-correctly-002.html) [(live test)](http://wpt.live/css/css-color/animation/opacity-animation-ending-correctly-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/animation/opacity-animation-ending-correctly-002.html)

<a id="ref-for-typedef-opacity-opacity-value①"></a>

<a id="typedef-opacity-opacity-value"></a>[\<opacity-value\>](#typedef-opacity-opacity-value)

The opacity to be applied to the element. The resulting opacity is applied to the entire element, rather than a particular color.

Opacity values outside the range \[0,1\] are not invalid, and are preserved in specified values, but are clamped to the range \[0, 1\] in computed values.

Tests

- [inline-opacity-float-child.html](https://wpt.fyi/results/css/css-color/inline-opacity-float-child.html) [(live test)](http://wpt.live/css/css-color/inline-opacity-float-child.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/inline-opacity-float-child.html)

<a id="ref-for-typedef-opacity-opacity-value②"></a>

<a id="ref-for-propdef-opacity①"></a>

Opacity in CSS is represented using the [\<opacity-value\>](#typedef-opacity-opacity-value) syntax, for example in the [opacity](#propdef-opacity) property.

<a id="ref-for-typedef-opacity-opacity-value③"></a>

<a id="ref-for-number-value"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-percentage-value"></a>

```text
<opacity-value> = <number> | <percentage>
```
<a id="ref-for-number-value①"></a>

<a id="ref-for-percentage-value①"></a>

<a id="ref-for-computed-value"></a>

Represented as a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), the useful range of the value is 0 (representing full transparency) to 1 (representing full opacity). It can also be written as a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), which [computes to](https://www.w3.org/TR/css-cascade-5/#computed-value) the equivalent <a id="ref-for-number-value②"></a>\<number\> (0% to 0, 100% to 1).

<a id="ref-for-propdef-opacity②"></a>

The [opacity](#propdef-opacity) property applies the specified opacity to the element <em>as a whole</em>, including its contents, rather than applying it to each descendant individually. This means that, for example, an opaque child occluding part of the element’s background will continue to do so even when <a id="ref-for-propdef-opacity③"></a>opacity is less than 1, but the element and child as a whole will show the underlying page through themselves.

It also means that the glyphs corresponding to all characters in the element are treated <em>as a whole</em>; any overlapping portions do not increase the opacity.

Tests

- [opacity-overlapping-letters.html](https://wpt.fyi/results/css/css-color/opacity-overlapping-letters.html) [(live test)](http://wpt.live/css/css-color/opacity-overlapping-letters.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/opacity-overlapping-letters.html)

<a id="arabic-opacity-rendering"></a> ![overlapping glyphs rendered correctly, and incorrectly](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/joining-and-transparency.svg)

<a id="ref-for-propdef-opacity④"></a>

Correct and incorrect rendering of text with an [opacity](#propdef-opacity) value of less than one, whose glyphs overlap.

<a id="ref-for-propdef-opacity⑤"></a>

If separate opacity for each glyph is desired, it can be achieved by using a color value which includes alpha, rather than setting the [opacity](#propdef-opacity) property.

<a id="ref-for-propdef-opacity⑥"></a>

<a id="ref-for-x43"></a>

If a box has [opacity](#propdef-opacity) less than 1, it forms a [stacking context](https://www.w3.org/TR/CSS2/visuren.html#x43) for its children. (This prevents its contents from interleaving in the z-axis with content outside it.)

Tests

- [body-opacity-0-to-1-stacking-context.html](https://wpt.fyi/results/css/css-color/body-opacity-0-to-1-stacking-context.html) [(live test)](http://wpt.live/css/css-color/body-opacity-0-to-1-stacking-context.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/body-opacity-0-to-1-stacking-context.html)

<a id="ref-for-propdef-z-index"></a>

<a id="ref-for-valdef-z-index-auto"></a>

Furthermore, if the [z-index](https://www.w3.org/TR/CSS2/visuren.html#propdef-z-index) property applies to the box, the [auto](https://drafts.csswg.org/css2/#valdef-z-index-auto) value is treated as 0 for the element; it is otherwise painted on the same layer within its parent stacking context as positioned elements with stack level 0 (as if it were a positioned element with z-index:0).

See [section 9.9](https://www.w3.org/TR/CSS21/visuren.html#layers) and [Appendix E](https://www.w3.org/TR/CSS21/zindex.html) of [\[CSS2\]](#biblio-css2) for more information on stacking contexts.

These rules about z-order do not apply to SVG elements, since SVG has its own [rendering model](https://www.w3.org/TR/SVG11/render.html) ([\[SVG11\]](#biblio-svg11), Chapter 3).

<a id="ref-for-propdef-opacity⑦"></a>

The value of the [opacity](#propdef-opacity) property does <em>not</em> affect hit testing.

### <a id="tagged-images"></a>3.4. Color Space of Tagged Images

An <a id="tagged-image"></a>tagged image is an image that is explicitly assigned a color profile, as defined by the image format. This is usually done by including an International Color Consortium (ICC) profile [\[ICC\]](#biblio-icc).

For example JPEG [\[JPEG\]](#biblio-jpeg), PNG [\[PNG\]](#biblio-png) and TIFF [\[TIFF\]](#biblio-tiff) all specify a means to embed an ICC profile.

Image formats may also use other, equivalent methods, often for brevity.

For example, PNG specifies a means (the [sRGB chunk](https://www.w3.org/TR/PNG/#11sRGB)) to explicitly tag an image as being in the sRGB color space, without including the sRGB ICC profile.

Similarly, PNG specifies a compact means (the [cICP chunk](https://www.w3.org/TR/png-3/#cICP-chunk)) to explicitly tag an image as being one of various SDR or HDR color spaces, such as Display P3 or BT.2100 HLG, without including an ICC profile.

Tagged RGB images, and tagged images using a transformation of RGB such as YCbCr, if the color profile or other identifying information is valid, must be treated as being in the specified color space.

Tests

- [tagged-images-001.html](https://wpt.fyi/results/css/css-color/tagged-images-001.html) [(live test)](http://wpt.live/css/css-color/tagged-images-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/tagged-images-001.html)
- [tagged-images-002.html](https://wpt.fyi/results/css/css-color/tagged-images-002.html) [(live test)](http://wpt.live/css/css-color/tagged-images-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/tagged-images-002.html)
- [tagged-images-003.html](https://wpt.fyi/results/css/css-color/tagged-images-003.html) [(live test)](http://wpt.live/css/css-color/tagged-images-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/tagged-images-003.html)
- [tagged-images-004.html](https://wpt.fyi/results/css/css-color/tagged-images-004.html) [(live test)](http://wpt.live/css/css-color/tagged-images-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/tagged-images-004.html)

<!-- -->

- [cicp-chunk.html](https://wpt.fyi/results/png/cicp-chunk.html) [(live test)](http://wpt.live/png/cicp-chunk.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/png/cicp-chunk.html)
- [fDAT-inherits-cICP.html](https://wpt.fyi/results/png/apng/fDAT-inherits-cICP.html) [(live test)](http://wpt.live/png/apng/fDAT-inherits-cICP.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/png/apng/fDAT-inherits-cICP.html)

For example, when a browser running on a system with a Display P3 monitor displays an JPEG image tagged as being in the ITU Rec BT.2020 [\[Rec.2020\]](#biblio-rec2020) color space, it must convert the colors from ITU Rec BT.2020 to Display P3 so that they display correctly. It must not treat the ITU Rec BT.2020 values as if they were Display P3 values, which would produce incorrect colors.

<a id="ref-for-untagged-image"></a>

If the color profile or other identifying information is invalid, the image is treated as described for [untagged images](#untagged-image).

### <a id="untagged"></a>3.5.  Color Spaces of Untagged Colors

<a id="ref-for-untagged-image①"></a>

For compatibility, colors specified in HTML, and [untagged images](#untagged-image) must be treated as being in the sRGB color space ([\[SRGB\]](#biblio-srgb)) unless otherwise specified.

Tests

- [untagged-images-001.html](https://wpt.fyi/results/css/css-color/untagged-images-001.html) [(live test)](http://wpt.live/css/css-color/untagged-images-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/untagged-images-001.html)

An <a id="untagged-image"></a>untagged image is an image that is not explicitly assigned a color profile, as defined by the image format.

This rule does not apply to untagged videos, since <a id="untagged-video"></a>untagged video should be presumed to be in an ITU-defined color space.

- At below 720p, it is Recommendation ITU-R BT.601 [\[ITU-R-BT.601\]](#biblio-itu-r-bt601)

- At 720p, it is SMPTE ST 296 (same colorimetry as 709) [\[SMPTE296\]](#biblio-smpte296)

- At 1080p, it is Recommendation ITU-R BT.709 [\[ITU-R-BT.709\]](#biblio-itu-r-bt709)

- At 4k (UHDTV) and above, it is ITU-R BT.2020 [\[Rec.2020\]](#biblio-rec2020) for SDR video

<a id="ref-for-typedef-color⑥"></a>

## <a id="color-type"></a>4.  Representing Colors: the [\<color\>](#typedef-color) type

Tests

This section describes a type, it is primarily tested where that type is used.

------------------------------------------------------------------------

Colors in CSS are represented as a list of color components, also sometimes called “channels”, representing axises in the color space. Each component has a minimum and maximum value, and can take any value between those two. Additionally, every color is accompanied by an <a id="alpha-channel"></a>alpha component, indicating how transparent it is, and thus how much of the backdrop one can see through the color.

CSS has several syntaxes for specifying color values:

- <a id="ref-for-hex-color"></a>

  the sRGB [hex color notation](#hex-color) which represents the RGB and alpha components in hexadecimal notation

- <a id="ref-for-color-functions"></a>

  the various [color functions](#color-functions) which can represent colors using a variety of color spaces and coordinate systems

- <a id="ref-for-named-color"></a>

  the constant [named color](#named-color) keywords

- <a id="ref-for-typedef-system-color"></a>

  <a id="ref-for-valdef-color-currentcolor①"></a>

  the variable [\<system-color\>](#typedef-system-color) keywords and [currentColor](#valdef-color-currentcolor) keyword.

<a id="ref-for-functional-notation"></a>

<a id="ref-for-color-space③"></a>

<a id="ref-for-typedef-hue"></a>

The <a id="color-functions"></a>color functions use CSS [functional notation](https://www.w3.org/TR/css-values-4/#functional-notation) to represent colors in a variety of [color spaces](#color-space) by specifying their component coordinates. Some of these use a <a id="cylindrical-polar-color"></a>cylindrical polar color model, specifying color by a [\<hue\>](#typedef-hue) angle, a central axis representing lightness (black-to-white), and a radius representing saturation or chroma (how far the color is from a neutral grey). The others use a <a id="rectangular-orthogonal-color"></a>rectangular orthogonal color model, specifying color using three orthogonal component axes.

<a id="ref-for-color-functions①"></a>

The [color functions](#color-functions) available in Level 4 are

- <a id="ref-for-funcdef-rgb"></a>

  <a id="ref-for-funcdef-rgba"></a>

  <a id="ref-for-hex-color①"></a>

  [rgb()](#funcdef-rgb) and its [rgba()](#funcdef-rgba) alias, which (like the [hex color notation](#hex-color)) specify sRGB colors directly by their red/green/blue/alpha components.

- <a id="ref-for-funcdef-hsl"></a>

  <a id="ref-for-funcdef-hsla"></a>

  [hsl()](#funcdef-hsl) and its [hsla()](#funcdef-hsla) alias, which specify sRGB colors by hue, saturation, and lightness using the [HSL](#the-hsl-notation) cylindrical coordinate model.

- <a id="ref-for-funcdef-hwb"></a>

  [hwb()](#funcdef-hwb), which specifies an sRGB color by hue, whiteness, and blackness using the [HWB](#the-hwb-notation) cylindrical coordinate model.

- <a id="ref-for-funcdef-lab"></a>

  [lab()](#funcdef-lab), which specifies a CIELAB color by CIE Lightness and its a- and b-axis hue coordinates (red/green-ness, and yellow/blue-ness) using the [CIE LAB rectangular coordinate model](#cie-lab).

- <a id="ref-for-funcdef-lch"></a>

  [lch()](#funcdef-lch) , which specifies a CIELAB color by CIE Lightness, Chroma, and hue using the [CIE LCH cylindrical coordinate model](#cie-lab)

- <a id="ref-for-funcdef-oklab"></a>

  [oklab()](#funcdef-oklab), which specifies an Oklab color by Oklab Lightness and its a- and b-axis hue coordinates (red/green-ness, and yellow/blue-ness) using the [Oklab](#ok-lab) rectangular coordinate model.

- <a id="ref-for-funcdef-oklch"></a>

  [oklch()](#funcdef-oklch) , which specifies an Oklab color by Oklab Lightness, Chroma, and hue using the [OkLCh](#ok-lab) cylindrical coordinate model.

- <a id="ref-for-funcdef-color"></a>

  [color()](#funcdef-color), which allows specifying colors in a variety of color spaces including [sRGB](#predefined-sRGB), [Linear-Light sRGB](#predefined-sRGB-linear), [Display P3](#predefined-display-p3), [Linear-Light Display P3](#predefined-display-p3-linear), [A98 RGB](#predefined-a98-rgb), [ProPhoto RGB](#predefined-prophoto-rgb), [ITU-R BT.2020-2](#predefined-rec2020), and [CIE XYZ](#predefined-xyz).

For easy reference in other specifications, <a id="opaque-black"></a>opaque black is defined as the color rgb(0 0 0 / 100%); <a id="transparent-black"></a>transparent black is the same color, but fully transparent—​i.e. rgb(0 0 0 / 0%).

Tests

- [color-computed-named-color.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-named-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-named-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-named-color.html)
- [color-computed.html](https://wpt.fyi/results/css/css-color/parsing/color-computed.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

<a id="ref-for-typedef-color⑦"></a>

### <a id="color-syntax"></a>4.1. The [\<color\>](#typedef-color) syntax

Tests

This section provides definitions used later, it does not need tests.

------------------------------------------------------------------------

<a id="ref-for-typedef-color⑧"></a>

Colors in CSS are represented by the <a id="typedef-color"></a>[\<color\>](#typedef-color) type:

<a id="ref-for-typedef-color-base"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-typedef-system-color①"></a>

<a id="typedef-color-base"></a>

<a id="ref-for-typedef-hex-color"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-typedef-color-function"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-typedef-named-color"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="typedef-color-function"></a>

<a id="ref-for-funcdef-rgb①"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-funcdef-rgba①"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-funcdef-hsl①"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-funcdef-hsla①"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-funcdef-hwb①"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-funcdef-lab①"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-funcdef-lch①"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-funcdef-oklab①"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-funcdef-oklch①"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-funcdef-color①"></a>

```text
<color> = <color-base> | currentColor | <system-color>

<color-base> = <hex-color> | <color-function> | <named-color> | transparent
<color-function> = <rgb()> | <rgba()> |
              <hsl()> | <hsla()> | <hwb()> |
              <lab()> | <lch()> | <oklab()> | <oklch()> |
              <color()>
```
<a id="ref-for-typedef-color⑨"></a>

An <a id="absolute-color"></a>absolute color is a [\<color\>](#typedef-color) whose computed value has an absolute, colorimetric interpretation. This means that the value is not:

- <a id="ref-for-valdef-color-currentcolor②"></a>

  <a id="ref-for-propdef-color①"></a>

  [currentColor](#valdef-color-currentcolor) (which depends on the value of the [color](#propdef-color) property)

- <a id="ref-for-typedef-system-color②"></a>

  a [\<system-color\>](#typedef-system-color) (which depends on the color mode)

The colors that <a id="resolve-to-srgb"></a>resolve to sRGB are:

- [hex](#hex-notation) colors

- <a id="ref-for-funcdef-rgb②"></a>

  <a id="ref-for-funcdef-rgba②"></a>

  [rgb()](#funcdef-rgb) and [rgba()](#funcdef-rgba) values

- <a id="ref-for-funcdef-hsl②"></a>

  <a id="ref-for-funcdef-hsla②"></a>

  [hsl()](#funcdef-hsl) and [hsla()](#funcdef-hsla) values

- <a id="ref-for-funcdef-hwb②"></a>

  [hwb()](#funcdef-hwb) values

- [named](#named-colors) colors

The functions that <a id="support-legacy-color-syntax"></a>support legacy color syntax are:

- <a id="ref-for-funcdef-rgb③"></a>

  <a id="ref-for-funcdef-rgba③"></a>

  [rgb()](#funcdef-rgb) and [rgba()](#funcdef-rgba)

- <a id="ref-for-funcdef-hsl③"></a>

  <a id="ref-for-funcdef-hsla③"></a>

  [hsl()](#funcdef-hsl) and [hsla()](#funcdef-hsla)

<a id="ref-for-funcdef-hsl④"></a>

<a id="ref-for-funcdef-hsla④"></a>

<a id="ref-for-funcdef-hwb③"></a>

<a id="ref-for-funcdef-lch②"></a>

<a id="ref-for-funcdef-oklch②"></a>

<a id="ref-for-color-functions②"></a>

<a id="ref-for-cylindrical-polar-color"></a>

<a id="ref-for-typedef-hue①"></a>

<a id="ref-for-rectangular-orthogonal-color"></a>

The [\<hsl()\>](#funcdef-hsl), [\<hsla()\>](#funcdef-hsla), [\<hwb()\>](#funcdef-hwb), [\<lch()\>](#funcdef-lch), and [\<oklch()\>](#funcdef-oklch) [color functions](#color-functions) are [cylindrical polar color](#cylindrical-polar-color) representations using a [\<hue\>](#typedef-hue) angle; the other <a id="ref-for-color-functions③"></a>color functions use [rectangular orthogonal color](#rectangular-orthogonal-color) representations.

#### <a id="color-syntax-modern"></a>4.1.1.  Modern (Space-separated) Color Function Syntax

<a id="ref-for-absolute-color"></a>

All of the [absolute color](#absolute-color) functional forms first defined in this specification use the <a id="modern-color-syntax"></a>modern color syntax, meaning:

- color components are separated by whitespace

- the optional alpha term is separated by a solidus ("/")

- minimum required precision [when serializing](#serializing-color-values) is defined, and may be greater than 8 bits per component

- <a id="ref-for-valdef-color-none"></a>

  <a id="ref-for-missing-color-component"></a>

  the [none](#valdef-color-none) value is allowed, to represent [missing components](#missing-color-component)

- <a id="ref-for-percentage-value②"></a>

  <a id="ref-for-number-value③"></a>

  components using [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) and [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) may be freely mixed

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-modern-syntax"></a>
>
> The following represents a saturated sRGB red that is 50% opaque:
>
> ```text
> rgb(100% 0% 0% / 50%)
> ```
#### <a id="color-syntax-legacy"></a>4.1.2.  Legacy (Comma-separated) Color Function Syntax

<a id="ref-for-funcdef-rgb④"></a>

<a id="ref-for-funcdef-rgba④"></a>

<a id="ref-for-funcdef-hsl⑤"></a>

<a id="ref-for-funcdef-hsla⑤"></a>

For Web compatibility, the syntactic forms of [rgb()](#funcdef-rgb), [rgba()](#funcdef-rgba), [hsl()](#funcdef-hsl), and [hsla()](#funcdef-hsla), (those defined in earlier specifications) also support a <a id="legacy-color-syntax"></a>legacy color syntax which has the following differences:

- color components are separated by commas (optionally preceded and/or followed by whitespace)

- <a id="ref-for-funcdef-hsla⑥"></a>

  <a id="ref-for-funcdef-hsl⑥"></a>

  non-opaque forms use a separate notation (for example [hsla()](#funcdef-hsla) rather than [hsl()](#funcdef-hsl)) and the alpha term is separated by commas (optionally preceded and/or followed by whitespace)

- minimum required precision is lower, 8 bits per component

- <a id="ref-for-valdef-color-none①"></a>

  the [none](#valdef-color-none) value is not allowed

- <a id="ref-for-percentage-value③"></a>

  <a id="ref-for-number-value④"></a>

  color components must be specified using either all-[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) or all-[\<number\>](https://www.w3.org/TR/css-values-4/#number-value), they can not be mixed.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-rgba-legacy"></a>
>
> The following represents a saturated sRGB red that is 50% opaque:
>
> ```text
> rgba(100%, 0%, 0%, 0.5)
> ```
<a id="ref-for-color-functions④"></a>

<a id="ref-for-legacy-color-syntax"></a>

For the [color functions](#color-functions) introduced in this or subsequent levels, where there is no Web compatibility issue, the [legacy color syntax](#legacy-color-syntax) is invalid.

<a id="ref-for-typedef-color-alpha-value"></a>

### <a id="alpha-syntax"></a>4.2.  Representing Transparency in Colors: the [\<alpha-value\>](#typedef-color-alpha-value) syntax

Tests

This section provides definitions used later, it does not need tests.

------------------------------------------------------------------------

<a id="typedef-color-alpha-value"></a>

<a id="ref-for-number-value⑤"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-percentage-value④"></a>

```text
<alpha-value> = <number> | <percentage>
```
<a id="ref-for-typedef-color-alpha-value①"></a>

Unless otherwise specified, an [\<alpha-value\>](#typedef-color-alpha-value) component of a color defaults to 100% when omitted. Values outside the range \[0,1\] are not invalid, but are clamped to that range at parsed-value time.

<a id="ref-for-typedef-hue②"></a>

### <a id="hue-syntax"></a>4.3.  Representing Cylindrical-coordinate Hues: the [\<hue\>](#typedef-hue) syntax

Tests

This section provides definitions used later, it does not need tests.

------------------------------------------------------------------------

Hue is represented as an angle of the color circle (the rainbow, twisted around into a circle, and with purple added between violet and red).

<a id="typedef-hue"></a>

<a id="ref-for-number-value⑥"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-angle-value"></a>

```text
<hue> = <number> | <angle>
```
<a id="ref-for-canonical-unit"></a>

Because this value is so often given in degrees, the argument can also be given as a number, which is interpreted as a number of degrees and is the [canonical unit](https://www.w3.org/TR/css-values-4/#canonical-unit).

This number is normalized to the range \[0,360).

<a id="ref-for-typedef-hue③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-hue-normalization"></a> For example, in hsl(-540 0 0) or hsl(540 0 0), the [\<hue\>](#typedef-hue) component is normalized to 180 degrees.
>
> <a id="ref-for-typedef-hue④"></a>
>
> In hsl(360 0 0) the [\<hue\>](#typedef-hue) component is normalized to 0 degrees.
>
> <a id="ref-for-typedef-hue⑤"></a>
>
> In hsl(calc(-infinity) 0 0) or hsl(calc(infinity) 0 0), the [\<hue\>](#typedef-hue) component is again normalized to 0 degrees.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The angles and spacing corresponding to particular hues depend on the color space. For example, in HSL and HWB, which use the sRGB color space, sRGB green is 120 degrees. In LCH, sRGB green is 134.39 degrees, display-p3 green is 136.01 degrees, a98-rgb green is 145.97 degrees and prophoto-rgb green is 141.04 degrees (because these are all different shades of green).

<a id="ref-for-typedef-hue⑥"></a>

<a id="ref-for-powerless-color-component"></a>

[\<hue\>](#typedef-hue) components are the most common components to become [powerless](#powerless-color-component); any color sufficiently close to the central achromatic axis will have a <a id="ref-for-powerless-color-component①"></a>powerless hue component.

<a id="ref-for-valdef-color-none②"></a>

### <a id="missing"></a>4.4.  “Missing” Color Components and the [none](#valdef-color-none) Keyword

In certain cases, a color can have one or more <a id="missing-color-component"></a>missing color components.

<a id="ref-for-valdef-color-white"></a>

In this specification, this happens automatically due to [hue-based interpolation](#hue-interpolation) for some colors (such as [white](#valdef-color-white)); other specifications can define additional situations in which components are automatically missing.

<a id="ref-for-legacy-color-syntax①"></a>

<a id="ref-for-valdef-color-none③"></a>

It can also be specified explicitly, by providing the keyword <a id="valdef-color-none"></a>none for a component in a color function. All color functions (with the exception of those using the [legacy color syntax](#legacy-color-syntax)) allow any of their components to be specified as [none](#valdef-color-none).

This should be done with care, and only when the particular effect of doing so is desired.

Tests

- [color-computed-color-function.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-color-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-color-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-color-function.html)
- [color-computed-hsl.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-hsl.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-hsl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-hsl.html)
- [color-computed-hwb.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-hwb.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-hwb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-hwb.html)
- [color-computed-lab.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-lab.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-lab.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-lab.html)
- [color-computed-relative-color.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-relative-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-relative-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-relative-color.html)
- [color-computed-rgb.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-rgb.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-rgb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-rgb.html)
- [color-invalid-hsl.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-hsl.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-hsl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-hsl.html)
- [color-invalid-rgb.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-rgb.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-rgb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-rgb.html)
- [color-valid-color-function.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-color-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-color-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-color-function.html)
- [color-valid-color-mix-function.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-color-mix-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-color-mix-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-color-mix-function.html)
- [color-valid-hsl.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-hsl.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-hsl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-hsl.html)
- [color-valid-hwb.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-hwb.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-hwb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-hwb.html)
- [color-valid-lab.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-lab.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-lab.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-lab.html)
- [color-valid-relative-color.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-relative-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-relative-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-relative-color.html)
- [color-valid-rgb.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-rgb.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-rgb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-rgb.html)

<a id="ref-for-missing-color-component①"></a>

For handling of [missing components](#missing-color-component) in situations which combine two colors, such as color interpolation, see [§ 13.2 Interpolating with Missing Components](#interpolation-missing).

<a id="ref-for-missing-color-component②"></a>

For all other purposes, a [missing component](#missing-color-component) behaves as a zero value, in the appropriate unit for that component: 0, 0%, or 0deg. This includes rendering the color directly, converting it to another color space, performing computations on the color component values, etc.

<a id="ref-for-missing-color-component③"></a>

<a id="ref-for-legacy-color-syntax②"></a>

<a id="ref-for-valdef-color-none④"></a>

If a color with a [missing component](#missing-color-component) is serialized or otherwise presented directly to an author, then for [legacy color syntax](#legacy-color-syntax) it represents that component as a zero value; otherwise, it represents that component as being the [none](#valdef-color-none) keyword.

<a id="ref-for-funcdef-color-mix"></a>

<a id="ref-for-valdef-color-white①"></a>

<a id="ref-for-missing-color-component④"></a>

<a id="ref-for-funcdef-hsl⑦"></a>

<a id="ref-for-valdef-color-green"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-missing-hue"></a> A missing hue is common when interpolating in cylindrical color spaces. For example, using the [color-mix()](https://www.w3.org/TR/css-color-5/#funcdef-color-mix) function specified in [\[CSS-COLOR-5\]](#biblio-css-color-5) one could write color-mix(in hsl, white 30%, green 70%). Since [white](#valdef-color-white) is an achromatic color, it has a [missing](#missing-color-component) hue when expressed in [hsl()](#funcdef-hsl) (effectively hsl(none 0% 100%)), since <em>any</em> hue will produce the same color) which means that the color-mix function will treat it as having the same hue as [green](#valdef-color-green) (effectively hsl(120deg 0% 100%)), and then interpolate based on those components.
>
> <a id="ref-for-valdef-color-white②"></a>
>
> The result will be a color that truly looks like a blend of green and white, rather than perhaps looking reddish (if [white](#valdef-color-white)s hue was defaulted to 0deg).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-grayscale-with-missing"></a> Explicitly specifying missing components can be useful to achieve an effect where you only <em>want</em> to interpolate certain components of a color.
>
> For example, to animate a color to "grayscale", no matter what the color is, one can interpolate it with oklch(none 0 none). This will take the hue and lightness from the starting color, but animate its chroma down to 0, rendering it into an equal-lightness gray with a steady hue across the whole animation.
>
> Doing this manually would require matching the hue and lightness of the starting color explicitly.

#### <a id="powerless"></a>4.4.1.  “Powerless” Color Components

Individual color syntaxes can specify that, in some cases, a given component of their syntax becomes a <a id="powerless-color-component"></a>powerless color component. This indicates that the value of the component doesn’t affect the rendered color; any value you give it will result in the same color displayed in the screen.

<a id="ref-for-funcdef-hsl⑧"></a>

<a id="ref-for-powerless-color-component②"></a>

For example, in [hsl()](#funcdef-hsl), the hue component is [powerless](#powerless-color-component) when the saturation component is 0%; a 0% saturation indicates a grayscale color, which has no hue at all, so 0deg and 180deg, or any other angle, will give the exact same result.

<a id="ref-for-powerless-color-component③"></a>

If a [powerless component](#powerless-color-component) is manually specified, it acts as normal; the fact that it’s <a id="ref-for-powerless-color-component④"></a>powerless has no effect.

<a id="ref-for-powerless-color-component⑤"></a>

<a id="ref-for-missing-color-component⑤"></a>

However, if a color is automatically produced by color space conversion, then any [powerless components](#powerless-color-component) in the result must instead be set to [missing](#missing-color-component), instead of whatever value was produced by the conversion process.

<a id="ref-for-cylindrical-polar-color①"></a>

<a id="ref-for-powerless-color-component⑥"></a>

<a id="ref-for-valdef-hsl-hsl"></a>

<a id="ref-for-funcdef-oklch③"></a>

When performing color space conversion to a [cylindrical polar color](#cylindrical-polar-color) space, user agents <em>shall</em> treat a hue component as [powerless](#powerless-color-component) if the chroma (or other measure of colorfulness, such as saturation in [hsl](#valdef-hsl-hsl)) is less than the epsilon (ε) specified for that color space. For example, a gray color converted into [oklch()](#funcdef-oklch) may, due to numerical errors, have an <em>extremely small</em> chroma rather than precisely 0%; as a result, the hue component is <a id="ref-for-powerless-color-component⑦"></a>powerless.

<a id="ref-for-typedef-color①⓪"></a>

### <a id="parse-color"></a>4.5.  Parsing a [\<color\>](#typedef-color) Value

Tests

This section provides a definition referenced elsewhere, it does not need tests.

------------------------------------------------------------------------

<a id="ref-for-typedef-color①①"></a>

<a id="ref-for-string"></a>

<a id="ref-for-concept-element"></a>

To <a id="parse-a-css-color-value"></a>parse a CSS [\<color\>](#typedef-color) value, given a [string](https://infra.spec.whatwg.org/#string) <var>input</var>, and an optional context [element](https://dom.spec.whatwg.org/#concept-element) <var>element</var>:

1.  <a id="ref-for-css-parse-something-according-to-a-css-grammar"></a>

    <a id="ref-for-typedef-color①②"></a>

    [Parse](https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar) <var>input</var> as a [\<color\>](#typedef-color). If the result is failure, return failure; otherwise, let <var>color</var> be the result.

2.  <a id="ref-for-used-color"></a>

    <a id="ref-for-typedef-color①③"></a>

    <a id="ref-for-valdef-color-currentcolor③"></a>

    <a id="ref-for-css-system-colors"></a>

    <a id="ref-for-initial-value"></a>

    Let <var>used color</var> be the result of [resolving](#resolving-color-values) <var>color</var> to a [used color](#used-color). If the value of other properties on the element a [\<color\>](#typedef-color) is on is required to do the resolution (such as resolving a [currentcolor](#valdef-color-currentcolor) or [system color](#css-system-colors)), use <var>element</var> if it was passed, or the [initial values](https://www.w3.org/TR/css-cascade-5/#initial-value) of the properties if not.

3.  Return <var>used color</var>.

<a id="ref-for-typedef-color①④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This algorithm is not intented to parse a CSS [\<color\>](#typedef-color) value specified in a CSS stylesheet or with a CSSOM interface, but in other places like HTML attributes or Canvas interfaces.

## <a id="numeric-srgb"></a>5.  sRGB Colors

<a id="ref-for-sRGB-space"></a>

CSS colors in the [sRGB](#sRGB-space) color space are represented by a triplet of values—​red, green, and blue—​identifying a point in the sRGB color space [\[SRGB\]](#biblio-srgb). This is an internationally-recognized, device-independent color space, and so is useful for specifying colors that will be displayed on a computer screen, but is also useful for specifying colors on other types of devices, like printers.

<a id="ref-for-color-space④"></a>

CSS also allows the use of non-sRGB [color space](#color-space)s, as described in [§ 10 Predefined Color Spaces](#predefined).

<a id="ref-for-hex-color②"></a>

<a id="ref-for-funcdef-rgb⑤"></a>

<a id="ref-for-funcdef-rgba⑤"></a>

<a id="ref-for-color-functions⑤"></a>

<a id="ref-for-funcdef-hsl⑨"></a>

<a id="ref-for-funcdef-hsla⑦"></a>

<a id="ref-for-funcdef-hwb④"></a>

<a id="ref-for-named-color①"></a>

<a id="ref-for-valdef-color-transparent"></a>

CSS provides several methods of directly specifying an sRGB color: [hex colors](#hex-color), [rgb()](#funcdef-rgb)/[rgba()](#funcdef-rgba) [color functions](#color-functions), [hsl()](#funcdef-hsl)/[hsla()](#funcdef-hsla) <a id="ref-for-color-functions⑥"></a>color functions, [hwb()](#funcdef-hwb) <a id="ref-for-color-functions⑦"></a>color function, [named colors](#named-color), and the [transparent](#valdef-color-transparent) keyword.

<a id="ref-for-funcdef-rgb⑥"></a>

<a id="ref-for-funcdef-rgba⑥"></a>

### <a id="rgb-functions"></a>5.1.  The RGB functions: [rgb()](#funcdef-rgb) and [rgba()](#funcdef-rgba)

<a id="ref-for-funcdef-rgb⑦"></a>

<a id="ref-for-funcdef-rgba⑦"></a>

The [rgb()](#funcdef-rgb) and [rgba()](#funcdef-rgba) functions define an sRGB color by specifying the r, g and b (red, green, and blue) components directly. Their syntax is:

<a id="funcdef-rgb"></a>

<a id="ref-for-typedef-legacy-rgb-syntax"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-typedef-modern-rgb-syntax"></a>

<a id="funcdef-rgba"></a>

<a id="ref-for-typedef-legacy-rgba-syntax"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-typedef-modern-rgba-syntax"></a>

<a id="typedef-legacy-rgb-syntax"></a>

<a id="ref-for-typedef-legacy-rgb-syntax①"></a>

<a id="ref-for-percentage-value⑤"></a>

<a id="ref-for-mult-comma"></a>

<a id="ref-for-comb-comma"></a>

<a id="ref-for-typedef-color-alpha-value②"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-number-value⑦"></a>

<a id="ref-for-mult-comma①"></a>

<a id="ref-for-comb-comma①"></a>

<a id="ref-for-typedef-color-alpha-value③"></a>

<a id="ref-for-mult-opt①"></a>

<a id="typedef-legacy-rgba-syntax"></a>

<a id="ref-for-typedef-legacy-rgba-syntax①"></a>

<a id="ref-for-percentage-value⑥"></a>

<a id="ref-for-mult-comma②"></a>

<a id="ref-for-comb-comma②"></a>

<a id="ref-for-typedef-color-alpha-value④"></a>

<a id="ref-for-mult-opt②"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-number-value⑧"></a>

<a id="ref-for-mult-comma③"></a>

<a id="ref-for-comb-comma③"></a>

<a id="ref-for-typedef-color-alpha-value⑤"></a>

<a id="ref-for-mult-opt③"></a>

<a id="typedef-modern-rgb-syntax"></a>

<a id="ref-for-typedef-modern-rgb-syntax①"></a>

<a id="ref-for-number-value⑨"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-percentage-value⑦"></a>

<a id="ref-for-comb-one②②"></a>

<a id="ref-for-mult-num"></a>

<a id="ref-for-typedef-color-alpha-value⑥"></a>

<a id="ref-for-comb-one②③"></a>

<a id="ref-for-mult-opt④"></a>

<a id="typedef-modern-rgba-syntax"></a>

<a id="ref-for-typedef-modern-rgba-syntax①"></a>

<a id="ref-for-number-value①⓪"></a>

<a id="ref-for-comb-one②④"></a>

<a id="ref-for-percentage-value⑧"></a>

<a id="ref-for-comb-one②⑤"></a>

<a id="ref-for-mult-num①"></a>

<a id="ref-for-typedef-color-alpha-value⑦"></a>

<a id="ref-for-comb-one②⑥"></a>

<a id="ref-for-mult-opt⑤"></a>

```text
rgb() = [ <legacy-rgb-syntax> | <modern-rgb-syntax> ]
rgba() = [ <legacy-rgba-syntax> | <modern-rgba-syntax> ]
<legacy-rgb-syntax> = 	rgb( <percentage>#{3} , <alpha-value>? ) |
                  rgb( <number>#{3} , <alpha-value>? )
<legacy-rgba-syntax> = rgba( <percentage>#{3} , <alpha-value>? ) |
                  rgba( <number>#{3} , <alpha-value>? )
<modern-rgb-syntax> = rgb(
  [ <number> | <percentage> | none]{3}
  [ / [<alpha-value> | none] ]?  )
<modern-rgba-syntax> = rgba(
  [ <number> | <percentage> | none]{3}
  [ / [<alpha-value> | none] ]?  )
```
<a id="prr-srgb"></a>



| Field               | Definition                                                             |
|---------------------|------------------------------------------------------------------------|
| <strong>Percentages&#xA;      </strong> | Allowed for r, g and b                                                 |
| <strong>Percent reference range&#xA0;&#xA;      </strong> | For r, g and b: 0% = 0.0, 100% = 255.0 For alpha: 0% = 0.0, 100% = 1.0 |



Tests

- [rgb-001.html](https://wpt.fyi/results/css/css-color/rgb-001.html) [(live test)](http://wpt.live/css/css-color/rgb-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgb-001.html)
- [rgb-002.html](https://wpt.fyi/results/css/css-color/rgb-002.html) [(live test)](http://wpt.live/css/css-color/rgb-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgb-002.html)
- [rgb-003.html](https://wpt.fyi/results/css/css-color/rgb-003.html) [(live test)](http://wpt.live/css/css-color/rgb-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgb-003.html)
- [rgb-004.html](https://wpt.fyi/results/css/css-color/rgb-004.html) [(live test)](http://wpt.live/css/css-color/rgb-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgb-004.html)
- [rgb-005.html](https://wpt.fyi/results/css/css-color/rgb-005.html) [(live test)](http://wpt.live/css/css-color/rgb-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgb-005.html)
- [rgb-006.html](https://wpt.fyi/results/css/css-color/rgb-006.html) [(live test)](http://wpt.live/css/css-color/rgb-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgb-006.html)
- [rgb-007.html](https://wpt.fyi/results/css/css-color/rgb-007.html) [(live test)](http://wpt.live/css/css-color/rgb-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgb-007.html)
- [rgb-008.html](https://wpt.fyi/results/css/css-color/rgb-008.html) [(live test)](http://wpt.live/css/css-color/rgb-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgb-008.html)
- [out-of-gamut-legacy-rgb.html](https://wpt.fyi/results/css/css-color/out-of-gamut-legacy-rgb.html) [(live test)](http://wpt.live/css/css-color/out-of-gamut-legacy-rgb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/out-of-gamut-legacy-rgb.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)
- [color-computed-rgb.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-rgb.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-rgb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-rgb.html)
- [color-invalid-rgb.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-rgb.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-rgb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-rgb.html)
- [color-valid-rgb.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-rgb.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-rgb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-rgb.html)

The first three arguments specify the r, g and b (red, green, and blue) components of the color, respectively. 0% represents the minimum value for that color component in the sRGB gamut, and 100% represents the maximum value.

The percentage reference range of the color components comes from the historical fact that many graphics engines stored the color components internally as a single byte, which can hold integers between 0 and 255. Implementations should honor the precision of the component as authored or calculated wherever possible. If this is not possible, the component should be [rounded towards +∞](https://drafts.csswg.org/css-values-4/#combine-integers).

<a id="ref-for-typedef-color-alpha-value⑧"></a>

The final argument, the [\<alpha-value\>](#typedef-color-alpha-value), specifies the alpha of the color. If omitted, it defaults to 100%.

Tests

- [background-color-rgb-001.html](https://wpt.fyi/results/css/css-color/background-color-rgb-001.html) [(live test)](http://wpt.live/css/css-color/background-color-rgb-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/background-color-rgb-001.html)
- [background-color-rgb-002.html](https://wpt.fyi/results/css/css-color/background-color-rgb-002.html) [(live test)](http://wpt.live/css/css-color/background-color-rgb-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/background-color-rgb-002.html)
- [background-color-rgb-003.html](https://wpt.fyi/results/css/css-color/background-color-rgb-003.html) [(live test)](http://wpt.live/css/css-color/background-color-rgb-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/background-color-rgb-003.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

Values outside these ranges are not invalid, but are clamped to the ranges defined here at parsed-value time.

<a id="ref-for-funcdef-rgb⑧"></a>

<a id="ref-for-funcdef-rgba⑧"></a>

<a id="ref-for-legacy-color-syntax③"></a>

For historical reasons, [rgb()](#funcdef-rgb) and [rgba()](#funcdef-rgba) also support a [legacy color syntax](#legacy-color-syntax).

Tests

- [rgba-001.html](https://wpt.fyi/results/css/css-color/rgba-001.html) [(live test)](http://wpt.live/css/css-color/rgba-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgba-001.html)
- [rgba-002.html](https://wpt.fyi/results/css/css-color/rgba-002.html) [(live test)](http://wpt.live/css/css-color/rgba-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgba-002.html)
- [rgba-003.html](https://wpt.fyi/results/css/css-color/rgba-003.html) [(live test)](http://wpt.live/css/css-color/rgba-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgba-003.html)
- [rgba-004.html](https://wpt.fyi/results/css/css-color/rgba-004.html) [(live test)](http://wpt.live/css/css-color/rgba-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgba-004.html)
- [rgba-005.html](https://wpt.fyi/results/css/css-color/rgba-005.html) [(live test)](http://wpt.live/css/css-color/rgba-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgba-005.html)
- [rgba-006.html](https://wpt.fyi/results/css/css-color/rgba-006.html) [(live test)](http://wpt.live/css/css-color/rgba-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgba-006.html)
- [rgba-007.html](https://wpt.fyi/results/css/css-color/rgba-007.html) [(live test)](http://wpt.live/css/css-color/rgba-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgba-007.html)
- [rgba-008.html](https://wpt.fyi/results/css/css-color/rgba-008.html) [(live test)](http://wpt.live/css/css-color/rgba-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rgba-008.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

### <a id="hex-notation"></a>5.2.  The RGB Hexadecimal Notations: \#RRGGBB

<a id="ref-for-funcdef-rgb⑨"></a>

The CSS <a id="hex-color"></a>hex color notation allows an sRGB color to be specified by giving the components as hexadecimal numbers, which is similar to how colors are often written directly in computer code. It’s also shorter than writing the same color out in [rgb()](#funcdef-rgb) notation.

<a id="ref-for-typedef-hash-token"></a>

The syntax of a <a id="typedef-hex-color"></a>\<hex-color\> is a [\<hash-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-hash-token) token whose value consists of 3, 4, 6, or 8 hexadecimal digits. In other words, a hex color is written as a hash character, "#", followed by some number of digits 0-9 or letters a-f (the case of the letters doesn’t matter - \#00ff00 is identical to \#00FF00).

The number of hex digits given determines how to decode the hex notation into an RGB color:

6 digits  
The first pair of digits, interpreted as a hexadecimal number, specifies the red component of the color, where 00 represents the minimum value and ff (255 in decimal) represents the maximum. The next pair of digits, interpreted in the same way, specifies the green component, and the last pair specifies the blue. The alpha component of the color is fully opaque.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-hex6"></a> In other words, \#00ff00 represents the same color as rgb(0 255 0) (a lime green).

8 digits  
The first 6 digits are interpreted identically to the 6-digit notation. The last pair of digits, interpreted as a hexadecimal number, specifies the alpha component of the color, where 00 represents a fully transparent color and ff represent a fully opaque color.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-hex8"></a> In other words, \#0000ffcc represents the same color as rgb(0 0 100% / 80%) (a slightly-transparent blue).

3 digits  
This is a shorter variant of the 6-digit notation. The first digit, interpreted as a hexadecimal number, specifies the red component of the color, where 0 represents the minimum value and f represents the maximum. The next two digits represent the green and blue components, respectively, in the same way. The alpha component of the color is fully opaque.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-hex3"></a> This syntax is often explained by saying that it’s identical to a 6-digit notation obtained by "duplicating" all of the digits. For example, the notation \#123 specifies the same color as the notation \#112233. This method of specifying a color has lower "resolution" than the 6-digit notation; there are only 4096 possible colors expressible in the 3-digit hex syntax, as opposed to approximately 17 million in 6-digit hex syntax.

4 digits  
This is a shorter variant of the 8-digit notation, "expanded" in the same way as the 3-digit notation is. The first digit, interpreted as a hexadecimal number, specifies the red component of the color, where 0 represents the minimum value and f represents the maximum. The next three digits represent the green, blue, and alpha components, respectively.

Tests

- [hex-001.html](https://wpt.fyi/results/css/css-color/hex-001.html) [(live test)](http://wpt.live/css/css-color/hex-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hex-001.html)
- [hex-002.html](https://wpt.fyi/results/css/css-color/hex-002.html) [(live test)](http://wpt.live/css/css-color/hex-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hex-002.html)
- [hex-003.html](https://wpt.fyi/results/css/css-color/hex-003.html) [(live test)](http://wpt.live/css/css-color/hex-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hex-003.html)
- [hex-004.html](https://wpt.fyi/results/css/css-color/hex-004.html) [(live test)](http://wpt.live/css/css-color/hex-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hex-004.html)
- [border-bottom-color.xht](https://wpt.fyi/results/css/css-color/border-bottom-color.xht) [(live test)](http://wpt.live/css/css-color/border-bottom-color.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/border-bottom-color.xht)
- [border-left-color.xht](https://wpt.fyi/results/css/css-color/border-left-color.xht) [(live test)](http://wpt.live/css/css-color/border-left-color.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/border-left-color.xht)
- [border-right-color.xht](https://wpt.fyi/results/css/css-color/border-right-color.xht) [(live test)](http://wpt.live/css/css-color/border-right-color.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/border-right-color.xht)
- [border-top-color.xht](https://wpt.fyi/results/css/css-color/border-top-color.xht) [(live test)](http://wpt.live/css/css-color/border-top-color.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/border-top-color.xht)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)
- [color-computed-hex-color.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-hex-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-hex-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-hex-color.html)
- [color-invalid-hex-color.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-hex-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-hex-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-hex-color.html)

## <a id="color-keywords"></a>6.  Color Keywords

<a id="ref-for-typedef-color①⑤"></a>

In addition to the various numeric syntaxes for [\<color\>](#typedef-color)s, CSS defines several sets of color keywords that can be used instead—​each with their own advantages or use cases.

### <a id="named-colors"></a>6.1.  Named Colors

<a id="ref-for-typedef-ident"></a>

<a id="ref-for-typedef-color①⑥"></a>

CSS defines a large set of <a id="named-color"></a>named colors, so that common colors can be written and read more easily. A <a id="typedef-named-color"></a>\<named-color\> is written as an [\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident), accepted anywhere a [\<color\>](#typedef-color) is. As usual for CSS-defined <a id="ref-for-typedef-ident①"></a>\<ident\>s, all of these keywords are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive).

The names resolve to colors in sRGB.

16 of CSS’s named colors come from the VGA palette originally, and were then adopted into HTML: aqua, black, blue, fuchsia, gray, green, lime, maroon, navy, olive, purple, red, silver, teal, white, and yellow. Most of the rest come from one version of the X11 color system, used in Unix-derived systems to specify colors for the console, and were then adopted into SVG.

<a id="ref-for-valdef-color-darkgray"></a>

<a id="ref-for-valdef-color-gray"></a>

<a id="ref-for-valdef-color-lightpink"></a>

<a id="ref-for-valdef-color-pink"></a>

<a id="ref-for-valdef-color-indianred"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: these color names are standardized here, <em>not because they are good</em>, but because their use and implementation has been widespread for decades and the standard needs to reflect reality. Indeed, it is often hard to imagine what each name will look like (hence the list below); the names are not evenly distributed throughout the sRGB color volume, the names are not even internally consistent ( [darkgray](#valdef-color-darkgray) is lighter than [gray](#valdef-color-gray), while [lightpink](#valdef-color-lightpink) is darker than [pink](#valdef-color-pink)), and some names (such as [indianred](#valdef-color-indianred), which was originally named after a red pigment from India), have been found to be offensive. Thus, their use is <em>not encouraged</em>.

<a id="ref-for-valdef-color-transparent①"></a>

<a id="ref-for-valdef-color-currentcolor④"></a>

(Two special color values, [transparent](#valdef-color-transparent) and [currentcolor](#valdef-color-currentcolor), are specially defined in their own sections.)

The following table defines all of the opaque named colors, by giving equivalent numeric specifications in the other color syntaxes.



| Named | Numeric | Color name          | Hex rgb  | Decimal     |
|-------|---------|---------------------|----------|-------------|
|       |         | <strong><dfn><span><a id="valdef-color-aliceblue"></a></span>aliceblue</dfn>&#xA;      </strong> | \#f0f8ff | 240 248 255 |
|       |         | <strong><dfn><span><a id="valdef-color-antiquewhite"></a></span>antiquewhite</dfn>&#xA;      </strong> | \#faebd7 | 250 235 215 |
|       |         | <strong><dfn><span><a id="valdef-color-aqua"></a></span>aqua</dfn>&#xA;      </strong> | \#00ffff | 0 255 255   |
|       |         | <strong><dfn><span><a id="valdef-color-aquamarine"></a></span>aquamarine</dfn>&#xA;      </strong> | \#7fffd4 | 127 255 212 |
|       |         | <strong><dfn><span><a id="valdef-color-azure"></a></span>azure</dfn>&#xA;      </strong> | \#f0ffff | 240 255 255 |
|       |         | <strong><dfn><span><a id="valdef-color-beige"></a></span>beige</dfn>&#xA;      </strong> | \#f5f5dc | 245 245 220 |
|       |         | <strong><dfn><span><a id="valdef-color-bisque"></a></span>bisque</dfn>&#xA;      </strong> | \#ffe4c4 | 255 228 196 |
|       |         | <strong><dfn><span><a id="valdef-color-black"></a></span>black</dfn>&#xA;      </strong> | \#000000 | 0 0 0       |
|       |         | <strong><dfn><span><a id="valdef-color-blanchedalmond"></a></span>blanchedalmond</dfn>&#xA;      </strong> | \#ffebcd | 255 235 205 |
|       |         | <strong><dfn><span><a id="valdef-color-blue"></a></span>blue</dfn>&#xA;      </strong> | \#0000ff | 0 0 255     |
|       |         | <strong><dfn><span><a id="valdef-color-blueviolet"></a></span>blueviolet</dfn>&#xA;      </strong> | \#8a2be2 | 138 43 226  |
|       |         | <strong><dfn><span><a id="valdef-color-brown"></a></span>brown</dfn>&#xA;      </strong> | \#a52a2a | 165 42 42   |
|       |         | <strong><dfn><span><a id="valdef-color-burlywood"></a></span>burlywood</dfn>&#xA;      </strong> | \#deb887 | 222 184 135 |
|       |         | <strong><dfn><span><a id="valdef-color-cadetblue"></a></span>cadetblue</dfn>&#xA;      </strong> | \#5f9ea0 | 95 158 160  |
|       |         | <strong><dfn><span><a id="valdef-color-chartreuse"></a></span>chartreuse</dfn>&#xA;      </strong> | \#7fff00 | 127 255 0   |
|       |         | <strong><dfn><span><a id="valdef-color-chocolate"></a></span>chocolate</dfn>&#xA;      </strong> | \#d2691e | 210 105 30  |
|       |         | <strong><dfn><span><a id="valdef-color-coral"></a></span>coral</dfn>&#xA;      </strong> | \#ff7f50 | 255 127 80  |
|       |         | <strong><dfn><span><a id="valdef-color-cornflowerblue"></a></span>cornflowerblue</dfn>&#xA;      </strong> | \#6495ed | 100 149 237 |
|       |         | <strong><dfn><span><a id="valdef-color-cornsilk"></a></span>cornsilk</dfn>&#xA;      </strong> | \#fff8dc | 255 248 220 |
|       |         | <strong><dfn><span><a id="valdef-color-crimson"></a></span>crimson</dfn>&#xA;      </strong> | \#dc143c | 220 20 60   |
|       |         | <strong><dfn><span><a id="valdef-color-cyan"></a></span>cyan</dfn>&#xA;      </strong> | \#00ffff | 0 255 255   |
|       |         | <strong><dfn><span><a id="valdef-color-darkblue"></a></span>darkblue</dfn>&#xA;      </strong> | \#00008b | 0 0 139     |
|       |         | <strong><dfn><span><a id="valdef-color-darkcyan"></a></span>darkcyan</dfn>&#xA;      </strong> | \#008b8b | 0 139 139   |
|       |         | <strong><dfn><span><a id="valdef-color-darkgoldenrod"></a></span>darkgoldenrod</dfn>&#xA;      </strong> | \#b8860b | 184 134 11  |
|       |         | <strong><dfn><span><a id="valdef-color-darkgray"></a></span>darkgray</dfn>&#xA;      </strong> | \#a9a9a9 | 169 169 169 |
|       |         | <strong><dfn><span><a id="valdef-color-darkgreen"></a></span>darkgreen</dfn>&#xA;      </strong> | \#006400 | 0 100 0     |
|       |         | <strong><dfn><span><a id="valdef-color-darkgrey"></a></span>darkgrey</dfn>&#xA;      </strong> | \#a9a9a9 | 169 169 169 |
|       |         | <strong><dfn><span><a id="valdef-color-darkkhaki"></a></span>darkkhaki</dfn>&#xA;      </strong> | \#bdb76b | 189 183 107 |
|       |         | <strong><dfn><span><a id="valdef-color-darkmagenta"></a></span>darkmagenta</dfn>&#xA;      </strong> | \#8b008b | 139 0 139   |
|       |         | <strong><dfn><span><a id="valdef-color-darkolivegreen"></a></span>darkolivegreen</dfn>&#xA;      </strong> | \#556b2f | 85 107 47   |
|       |         | <strong><dfn><span><a id="valdef-color-darkorange"></a></span>darkorange</dfn>&#xA;      </strong> | \#ff8c00 | 255 140 0   |
|       |         | <strong><dfn><span><a id="valdef-color-darkorchid"></a></span>darkorchid</dfn>&#xA;      </strong> | \#9932cc | 153 50 204  |
|       |         | <strong><dfn><span><a id="valdef-color-darkred"></a></span>darkred</dfn>&#xA;      </strong> | \#8b0000 | 139 0 0     |
|       |         | <strong><dfn><span><a id="valdef-color-darksalmon"></a></span>darksalmon</dfn>&#xA;      </strong> | \#e9967a | 233 150 122 |
|       |         | <strong><dfn><span><a id="valdef-color-darkseagreen"></a></span>darkseagreen</dfn>&#xA;      </strong> | \#8fbc8f | 143 188 143 |
|       |         | <strong><dfn><span><a id="valdef-color-darkslateblue"></a></span>darkslateblue</dfn>&#xA;      </strong> | \#483d8b | 72 61 139   |
|       |         | <strong><dfn><span><a id="valdef-color-darkslategray"></a></span>darkslategray</dfn>&#xA;      </strong> | \#2f4f4f | 47 79 79    |
|       |         | <strong><dfn><span><a id="valdef-color-darkslategrey"></a></span>darkslategrey</dfn>&#xA;      </strong> | \#2f4f4f | 47 79 79    |
|       |         | <strong><dfn><span><a id="valdef-color-darkturquoise"></a></span>darkturquoise</dfn>&#xA;      </strong> | \#00ced1 | 0 206 209   |
|       |         | <strong><dfn><span><a id="valdef-color-darkviolet"></a></span>darkviolet</dfn>&#xA;      </strong> | \#9400d3 | 148 0 211   |
|       |         | <strong><dfn><span><a id="valdef-color-deeppink"></a></span>deeppink</dfn>&#xA;      </strong> | \#ff1493 | 255 20 147  |
|       |         | <strong><dfn><span><a id="valdef-color-deepskyblue"></a></span>deepskyblue</dfn>&#xA;      </strong> | \#00bfff | 0 191 255   |
|       |         | <strong><dfn><span><a id="valdef-color-dimgray"></a></span>dimgray</dfn>&#xA;      </strong> | \#696969 | 105 105 105 |
|       |         | <strong><dfn><span><a id="valdef-color-dimgrey"></a></span>dimgrey</dfn>&#xA;      </strong> | \#696969 | 105 105 105 |
|       |         | <strong><dfn><span><a id="valdef-color-dodgerblue"></a></span>dodgerblue</dfn>&#xA;      </strong> | \#1e90ff | 30 144 255  |
|       |         | <strong><dfn><span><a id="valdef-color-firebrick"></a></span>firebrick</dfn>&#xA;      </strong> | \#b22222 | 178 34 34   |
|       |         | <strong><dfn><span><a id="valdef-color-floralwhite"></a></span>floralwhite</dfn>&#xA;      </strong> | \#fffaf0 | 255 250 240 |
|       |         | <strong><dfn><span><a id="valdef-color-forestgreen"></a></span>forestgreen</dfn>&#xA;      </strong> | \#228b22 | 34 139 34   |
|       |         | <strong><dfn><span><a id="valdef-color-fuchsia"></a></span>fuchsia</dfn>&#xA;      </strong> | \#ff00ff | 255 0 255   |
|       |         | <strong><dfn><span><a id="valdef-color-gainsboro"></a></span>gainsboro</dfn>&#xA;      </strong> | \#dcdcdc | 220 220 220 |
|       |         | <strong><dfn><span><a id="valdef-color-ghostwhite"></a></span>ghostwhite</dfn>&#xA;      </strong> | \#f8f8ff | 248 248 255 |
|       |         | <strong><dfn><span><a id="valdef-color-gold"></a></span>gold</dfn>&#xA;      </strong> | \#ffd700 | 255 215 0   |
|       |         | <strong><dfn><span><a id="valdef-color-goldenrod"></a></span>goldenrod</dfn>&#xA;      </strong> | \#daa520 | 218 165 32  |
|       |         | <strong><dfn><span><a id="valdef-color-gray"></a></span>gray</dfn>&#xA;      </strong> | \#808080 | 128 128 128 |
|       |         | <strong><dfn><span><a id="valdef-color-green"></a></span>green</dfn>&#xA;      </strong> | \#008000 | 0 128 0     |
|       |         | <strong><dfn><span><a id="valdef-color-greenyellow"></a></span>greenyellow</dfn>&#xA;      </strong> | \#adff2f | 173 255 47  |
|       |         | <strong><dfn><span><a id="valdef-color-grey"></a></span>grey</dfn>&#xA;      </strong> | \#808080 | 128 128 128 |
|       |         | <strong><dfn><span><a id="valdef-color-honeydew"></a></span>honeydew</dfn>&#xA;      </strong> | \#f0fff0 | 240 255 240 |
|       |         | <strong><dfn><span><a id="valdef-color-hotpink"></a></span>hotpink</dfn>&#xA;      </strong> | \#ff69b4 | 255 105 180 |
|       |         | <strong><dfn><span><a id="valdef-color-indianred"></a></span>indianred</dfn>&#xA;      </strong> | \#cd5c5c | 205 92 92   |
|       |         | <strong><dfn><span><a id="valdef-color-indigo"></a></span>indigo</dfn>&#xA;      </strong> | \#4b0082 | 75 0 130    |
|       |         | <strong><dfn><span><a id="valdef-color-ivory"></a></span>ivory</dfn>&#xA;      </strong> | \#fffff0 | 255 255 240 |
|       |         | <strong><dfn><span><a id="valdef-color-khaki"></a></span>khaki</dfn>&#xA;      </strong> | \#f0e68c | 240 230 140 |
|       |         | <strong><dfn><span><a id="valdef-color-lavender"></a></span>lavender</dfn>&#xA;      </strong> | \#e6e6fa | 230 230 250 |
|       |         | <strong><dfn><span><a id="valdef-color-lavenderblush"></a></span>lavenderblush</dfn>&#xA;      </strong> | \#fff0f5 | 255 240 245 |
|       |         | <strong><dfn><span><a id="valdef-color-lawngreen"></a></span>lawngreen</dfn>&#xA;      </strong> | \#7cfc00 | 124 252 0   |
|       |         | <strong><dfn><span><a id="valdef-color-lemonchiffon"></a></span>lemonchiffon</dfn>&#xA;      </strong> | \#fffacd | 255 250 205 |
|       |         | <strong><dfn><span><a id="valdef-color-lightblue"></a></span>lightblue</dfn>&#xA;      </strong> | \#add8e6 | 173 216 230 |
|       |         | <strong><dfn><span><a id="valdef-color-lightcoral"></a></span>lightcoral</dfn>&#xA;      </strong> | \#f08080 | 240 128 128 |
|       |         | <strong><dfn><span><a id="valdef-color-lightcyan"></a></span>lightcyan</dfn>&#xA;      </strong> | \#e0ffff | 224 255 255 |
|       |         | <strong><dfn><span><a id="valdef-color-lightgoldenrodyellow"></a></span>lightgoldenrodyellow</dfn>&#xA;      </strong> | \#fafad2 | 250 250 210 |
|       |         | <strong><dfn><span><a id="valdef-color-lightgray"></a></span>lightgray</dfn>&#xA;      </strong> | \#d3d3d3 | 211 211 211 |
|       |         | <strong><dfn><span><a id="valdef-color-lightgreen"></a></span>lightgreen</dfn>&#xA;      </strong> | \#90ee90 | 144 238 144 |
|       |         | <strong><dfn><span><a id="valdef-color-lightgrey"></a></span>lightgrey</dfn>&#xA;      </strong> | \#d3d3d3 | 211 211 211 |
|       |         | <strong><dfn><span><a id="valdef-color-lightpink"></a></span>lightpink</dfn>&#xA;      </strong> | \#ffb6c1 | 255 182 193 |
|       |         | <strong><dfn><span><a id="valdef-color-lightsalmon"></a></span>lightsalmon</dfn>&#xA;      </strong> | \#ffa07a | 255 160 122 |
|       |         | <strong><dfn><span><a id="valdef-color-lightseagreen"></a></span>lightseagreen</dfn>&#xA;      </strong> | \#20b2aa | 32 178 170  |
|       |         | <strong><dfn><span><a id="valdef-color-lightskyblue"></a></span>lightskyblue</dfn>&#xA;      </strong> | \#87cefa | 135 206 250 |
|       |         | <strong><dfn><span><a id="valdef-color-lightslategray"></a></span>lightslategray</dfn>&#xA;      </strong> | \#778899 | 119 136 153 |
|       |         | <strong><dfn><span><a id="valdef-color-lightslategrey"></a></span>lightslategrey</dfn>&#xA;      </strong> | \#778899 | 119 136 153 |
|       |         | <strong><dfn><span><a id="valdef-color-lightsteelblue"></a></span>lightsteelblue</dfn>&#xA;      </strong> | \#b0c4de | 176 196 222 |
|       |         | <strong><dfn><span><a id="valdef-color-lightyellow"></a></span>lightyellow</dfn>&#xA;      </strong> | \#ffffe0 | 255 255 224 |
|       |         | <strong><dfn><span><a id="valdef-color-lime"></a></span>lime</dfn>&#xA;      </strong> | \#00ff00 | 0 255 0     |
|       |         | <strong><dfn><span><a id="valdef-color-limegreen"></a></span>limegreen</dfn>&#xA;      </strong> | \#32cd32 | 50 205 50   |
|       |         | <strong><dfn><span><a id="valdef-color-linen"></a></span>linen</dfn>&#xA;      </strong> | \#faf0e6 | 250 240 230 |
|       |         | <strong><dfn><span><a id="valdef-color-magenta"></a></span>magenta</dfn>&#xA;      </strong> | \#ff00ff | 255 0 255   |
|       |         | <strong><dfn><span><a id="valdef-color-maroon"></a></span>maroon</dfn>&#xA;      </strong> | \#800000 | 128 0 0     |
|       |         | <strong><dfn><span><a id="valdef-color-mediumaquamarine"></a></span>mediumaquamarine</dfn>&#xA;      </strong> | \#66cdaa | 102 205 170 |
|       |         | <strong><dfn><span><a id="valdef-color-mediumblue"></a></span>mediumblue</dfn>&#xA;      </strong> | \#0000cd | 0 0 205     |
|       |         | <strong><dfn><span><a id="valdef-color-mediumorchid"></a></span>mediumorchid</dfn>&#xA;      </strong> | \#ba55d3 | 186 85 211  |
|       |         | <strong><dfn><span><a id="valdef-color-mediumpurple"></a></span>mediumpurple</dfn>&#xA;      </strong> | \#9370db | 147 112 219 |
|       |         | <strong><dfn><span><a id="valdef-color-mediumseagreen"></a></span>mediumseagreen</dfn>&#xA;      </strong> | \#3cb371 | 60 179 113  |
|       |         | <strong><dfn><span><a id="valdef-color-mediumslateblue"></a></span>mediumslateblue</dfn>&#xA;      </strong> | \#7b68ee | 123 104 238 |
|       |         | <strong><dfn><span><a id="valdef-color-mediumspringgreen"></a></span>mediumspringgreen</dfn>&#xA;      </strong> | \#00fa9a | 0 250 154   |
|       |         | <strong><dfn><span><a id="valdef-color-mediumturquoise"></a></span>mediumturquoise</dfn>&#xA;      </strong> | \#48d1cc | 72 209 204  |
|       |         | <strong><dfn><span><a id="valdef-color-mediumvioletred"></a></span>mediumvioletred</dfn>&#xA;      </strong> | \#c71585 | 199 21 133  |
|       |         | <strong><dfn><span><a id="valdef-color-midnightblue"></a></span>midnightblue</dfn>&#xA;      </strong> | \#191970 | 25 25 112   |
|       |         | <strong><dfn><span><a id="valdef-color-mintcream"></a></span>mintcream</dfn>&#xA;      </strong> | \#f5fffa | 245 255 250 |
|       |         | <strong><dfn><span><a id="valdef-color-mistyrose"></a></span>mistyrose</dfn>&#xA;      </strong> | \#ffe4e1 | 255 228 225 |
|       |         | <strong><dfn><span><a id="valdef-color-moccasin"></a></span>moccasin</dfn>&#xA;      </strong> | \#ffe4b5 | 255 228 181 |
|       |         | <strong><dfn><span><a id="valdef-color-navajowhite"></a></span>navajowhite</dfn>&#xA;      </strong> | \#ffdead | 255 222 173 |
|       |         | <strong><dfn><span><a id="valdef-color-navy"></a></span>navy</dfn>&#xA;      </strong> | \#000080 | 0 0 128     |
|       |         | <strong><dfn><span><a id="valdef-color-oldlace"></a></span>oldlace</dfn>&#xA;      </strong> | \#fdf5e6 | 253 245 230 |
|       |         | <strong><dfn><span><a id="valdef-color-olive"></a></span>olive</dfn>&#xA;      </strong> | \#808000 | 128 128 0   |
|       |         | <strong><dfn><span><a id="valdef-color-olivedrab"></a></span>olivedrab</dfn>&#xA;      </strong> | \#6b8e23 | 107 142 35  |
|       |         | <strong><dfn><span><a id="valdef-color-orange"></a></span>orange</dfn>&#xA;      </strong> | \#ffa500 | 255 165 0   |
|       |         | <strong><dfn><span><a id="valdef-color-orangered"></a></span>orangered</dfn>&#xA;      </strong> | \#ff4500 | 255 69 0    |
|       |         | <strong><dfn><span><a id="valdef-color-orchid"></a></span>orchid</dfn>&#xA;      </strong> | \#da70d6 | 218 112 214 |
|       |         | <strong><dfn><span><a id="valdef-color-palegoldenrod"></a></span>palegoldenrod</dfn>&#xA;      </strong> | \#eee8aa | 238 232 170 |
|       |         | <strong><dfn><span><a id="valdef-color-palegreen"></a></span>palegreen</dfn>&#xA;      </strong> | \#98fb98 | 152 251 152 |
|       |         | <strong><dfn><span><a id="valdef-color-paleturquoise"></a></span>paleturquoise</dfn>&#xA;      </strong> | \#afeeee | 175 238 238 |
|       |         | <strong><dfn><span><a id="valdef-color-palevioletred"></a></span>palevioletred</dfn>&#xA;      </strong> | \#db7093 | 219 112 147 |
|       |         | <strong><dfn><span><a id="valdef-color-papayawhip"></a></span>papayawhip</dfn>&#xA;      </strong> | \#ffefd5 | 255 239 213 |
|       |         | <strong><dfn><span><a id="valdef-color-peachpuff"></a></span>peachpuff</dfn>&#xA;      </strong> | \#ffdab9 | 255 218 185 |
|       |         | <strong><dfn><span><a id="valdef-color-peru"></a></span>peru</dfn>&#xA;      </strong> | \#cd853f | 205 133 63  |
|       |         | <strong><dfn><span><a id="valdef-color-pink"></a></span>pink</dfn>&#xA;      </strong> | \#ffc0cb | 255 192 203 |
|       |         | <strong><dfn><span><a id="valdef-color-plum"></a></span>plum</dfn>&#xA;      </strong> | \#dda0dd | 221 160 221 |
|       |         | <strong><dfn><span><a id="valdef-color-powderblue"></a></span>powderblue</dfn>&#xA;      </strong> | \#b0e0e6 | 176 224 230 |
|       |         | <strong><dfn><span><a id="valdef-color-purple"></a></span>purple</dfn>&#xA;      </strong> | \#800080 | 128 0 128   |
|       |         | <strong><dfn><span><a id="valdef-color-rebeccapurple"></a></span>rebeccapurple</dfn>&#xA;      </strong> | \#663399 | 102 51 153  |
|       |         | <strong><dfn><span><a id="valdef-color-red"></a></span>red</dfn>&#xA;      </strong> | \#ff0000 | 255 0 0     |
|       |         | <strong><dfn><span><a id="valdef-color-rosybrown"></a></span>rosybrown</dfn>&#xA;      </strong> | \#bc8f8f | 188 143 143 |
|       |         | <strong><dfn><span><a id="valdef-color-royalblue"></a></span>royalblue</dfn>&#xA;      </strong> | \#4169e1 | 65 105 225  |
|       |         | <strong><dfn><span><a id="valdef-color-saddlebrown"></a></span>saddlebrown</dfn>&#xA;      </strong> | \#8b4513 | 139 69 19   |
|       |         | <strong><dfn><span><a id="valdef-color-salmon"></a></span>salmon</dfn>&#xA;      </strong> | \#fa8072 | 250 128 114 |
|       |         | <strong><dfn><span><a id="valdef-color-sandybrown"></a></span>sandybrown</dfn>&#xA;      </strong> | \#f4a460 | 244 164 96  |
|       |         | <strong><dfn><span><a id="valdef-color-seagreen"></a></span>seagreen</dfn>&#xA;      </strong> | \#2e8b57 | 46 139 87   |
|       |         | <strong><dfn><span><a id="valdef-color-seashell"></a></span>seashell</dfn>&#xA;      </strong> | \#fff5ee | 255 245 238 |
|       |         | <strong><dfn><span><a id="valdef-color-sienna"></a></span>sienna</dfn>&#xA;      </strong> | \#a0522d | 160 82 45   |
|       |         | <strong><dfn><span><a id="valdef-color-silver"></a></span>silver</dfn>&#xA;      </strong> | \#c0c0c0 | 192 192 192 |
|       |         | <strong><dfn><span><a id="valdef-color-skyblue"></a></span>skyblue</dfn>&#xA;      </strong> | \#87ceeb | 135 206 235 |
|       |         | <strong><dfn><span><a id="valdef-color-slateblue"></a></span>slateblue</dfn>&#xA;      </strong> | \#6a5acd | 106 90 205  |
|       |         | <strong><dfn><span><a id="valdef-color-slategray"></a></span>slategray</dfn>&#xA;      </strong> | \#708090 | 112 128 144 |
|       |         | <strong><dfn><span><a id="valdef-color-slategrey"></a></span>slategrey</dfn>&#xA;      </strong> | \#708090 | 112 128 144 |
|       |         | <strong><dfn><span><a id="valdef-color-snow"></a></span>snow</dfn>&#xA;      </strong> | \#fffafa | 255 250 250 |
|       |         | <strong><dfn><span><a id="valdef-color-springgreen"></a></span>springgreen</dfn>&#xA;      </strong> | \#00ff7f | 0 255 127   |
|       |         | <strong><dfn><span><a id="valdef-color-steelblue"></a></span>steelblue</dfn>&#xA;      </strong> | \#4682b4 | 70 130 180  |
|       |         | <strong><dfn><span><a id="valdef-color-tan"></a></span>tan</dfn>&#xA;      </strong> | \#d2b48c | 210 180 140 |
|       |         | <strong><dfn><span><a id="valdef-color-teal"></a></span>teal</dfn>&#xA;      </strong> | \#008080 | 0 128 128   |
|       |         | <strong><dfn><span><a id="valdef-color-thistle"></a></span>thistle</dfn>&#xA;      </strong> | \#d8bfd8 | 216 191 216 |
|       |         | <strong><dfn><span><a id="valdef-color-tomato"></a></span>tomato</dfn>&#xA;      </strong> | \#ff6347 | 255 99 71   |
|       |         | <strong><dfn><span><a id="valdef-color-turquoise"></a></span>turquoise</dfn>&#xA;      </strong> | \#40e0d0 | 64 224 208  |
|       |         | <strong><dfn><span><a id="valdef-color-violet"></a></span>violet</dfn>&#xA;      </strong> | \#ee82ee | 238 130 238 |
|       |         | <strong><dfn><span><a id="valdef-color-wheat"></a></span>wheat</dfn>&#xA;      </strong> | \#f5deb3 | 245 222 179 |
|       |         | <strong><dfn><span><a id="valdef-color-white"></a></span>white</dfn>&#xA;      </strong> | \#ffffff | 255 255 255 |
|       |         | <strong><dfn><span><a id="valdef-color-whitesmoke"></a></span>whitesmoke</dfn>&#xA;      </strong> | \#f5f5f5 | 245 245 245 |
|       |         | <strong><dfn><span><a id="valdef-color-yellow"></a></span>yellow</dfn>&#xA;      </strong> | \#ffff00 | 255 255 0   |
|       |         | <strong><dfn><span><a id="valdef-color-yellowgreen"></a></span>yellowgreen</dfn>&#xA;      </strong> | \#9acd32 | 154 205 50  |



> <strong data-conversion-semantic="note">Note</strong>
>
> Note: this list of colors and their definitions is a superset of the list of [named colors defined by SVG 1.1](https://www.w3.org/TR/SVG11/types.html#ColorKeywords).

For historical reasons, this is also referred to as the X11 color set.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The history of the X11 color system is interesting, and was excellently summarized by [Alex Sexton in their talk “Peachpuffs and Lemonchiffons”](https://www.youtube.com/watch?v=HmStJQzclHc).

Tests

- [named-001.html](https://wpt.fyi/results/css/css-color/named-001.html) [(live test)](http://wpt.live/css/css-color/named-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/named-001.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)
- [color-computed-named-color.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-named-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-named-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-named-color.html)
- [color-invalid-named-color.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-named-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-named-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-named-color.html)

### <a id="css-system-colors"></a>6.2.  System Colors

<a id="ref-for-typedef-system-color③"></a>

In general, the [\<system-color\>](#typedef-system-color) keywords reflect <em>default</em> color choices made by the user, the browser, or the OS. They are typically used in the browser default stylesheet, for this reason.

<a id="ref-for-typedef-system-color④"></a>

<a id="ref-for-used-color-scheme"></a>

To maintain legibility, the [\<system-color\>](#typedef-system-color) keywords also respond to the [used color scheme](https://www.w3.org/TR/css-color-adjust-1/#used-color-scheme).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-LM-DM-links"></a> For example, traditional blue link text is legible on a white background (WCAG contrast 8.59:1, AAA pass) but would not be legible on a black background (WCAG contrast 2.44:1, AA fail). Instead, a lighter blue such as \#81D9FE would be used in dark mode (WCAG contrast 13.28:1, AAA pass).
>
> Legible link text
>
> Illegible link text
>
> Legible link text

<a id="ref-for-forced-colors-mode"></a>

However, in [forced colors mode](https://www.w3.org/TR/css-color-adjust-1/#forced-colors-mode), most colors on the page are forced into a restricted, user-chosen palette, see [CSS Color Adjustment 1 § 5.2 Forced Colors Mode Color Palettes](https://www.w3.org/TR/css-color-adjust-1/#forced-color-palettes). The <a id="typedef-system-color"></a>\<system-color\> keywords expose these user-chosen colors so that the rest of the page can integrate with this restricted palette.

<a id="ref-for-media-feature"></a>

<a id="ref-for-typedef-system-color⑤"></a>

When the [forced-colors](https://drafts.csswg.org/mediaqueries-5/#descdef-media-forced-colors) [media feature](https://www.w3.org/TR/mediaqueries-5/#media-feature) is active, authors <em>should</em> use the [\<system-color\>](#typedef-system-color) keywords as color values in properties other than those listed in [CSS Color Adjustment 1 § 3.1 Properties Affected by Forced Colors Mode](https://www.w3.org/TR/css-color-adjust-1/#forced-colors-properties), to ensure legibility and consistency across the page and avoid an uncoordinated mishmash of user-forced and page-chosen colors.

Tests

- [system-color-consistency.html](https://wpt.fyi/results/css/css-color/system-color-consistency.html) [(live test)](http://wpt.live/css/css-color/system-color-consistency.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/system-color-consistency.html)
- [system-color-support.html](https://wpt.fyi/results/css/css-color/system-color-support.html) [(live test)](http://wpt.live/css/css-color/system-color-support.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/system-color-support.html)
- [color-valid-system-color.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-system-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-system-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-system-color.html)

<a id="ref-for-typedef-system-color⑥"></a>

When the values of [\<system-color\>](#typedef-system-color) keywords come from the browser, (as opposed to being OS defaults or user choices) the browser should ensure that [matching foreground/background pairs](#system-color-pairs) have a minimum of WCAG AA contrast. However, user preferences (for higher or lower contrast), whether set as a browser preference, a user stylesheet, or by altering the OS defaults, must take precedence over this requirement.

<a id="ref-for-valdef-color-canvas"></a>

<a id="ref-for-valdef-color-buttontext"></a>

Authors <em>may</em> also use these keywords at any time, but <em>should</em> be careful to use the colors in [matching background-foreground pairs](#system-color-pairs) to ensure appropriate contrast, as any particular contrast relationship across non-matching pairs (e.g. [Canvas](#valdef-color-canvas) and [ButtonText](#valdef-color-buttontext)) is not guaranteed.

<a id="ref-for-typedef-system-color⑦"></a>

The [\<system-color\>](#typedef-system-color) keywords are defined as follows:

<a id="system-color-values"></a>

<a id="valdef-color-accentcolor"></a>AccentColor  
 Background of accented user interface controls.

<a id="valdef-color-accentcolortext"></a>AccentColorText  
 Text of accented user interface controls.

<a id="valdef-color-activetext"></a>ActiveText  
 Text in active links. For light backgrounds, traditionally red.

<a id="valdef-color-buttonborder"></a>ButtonBorder  
 The base border color for push buttons.

<a id="valdef-color-buttonface"></a>ButtonFace  
 The face background color for push buttons.

<a id="valdef-color-buttontext"></a>ButtonText  
 Text on push buttons.

<a id="valdef-color-canvas"></a>Canvas  
 Background of application content or documents.

<a id="valdef-color-canvastext"></a>CanvasText  
 Text in application content or documents.

<a id="valdef-color-field"></a>Field  
 Background of input fields.

<a id="valdef-color-fieldtext"></a>FieldText  
 Text in input fields.

<a id="valdef-color-graytext"></a>GrayText  
 Disabled text. (Often, but not necessarily, gray.)

<a id="valdef-color-highlight"></a>Highlight  
 Background of selected text, for example from ::selection.

<a id="valdef-color-highlighttext"></a>HighlightText  
 Text of selected text.

<a id="valdef-color-linktext"></a>LinkText  
 Text in non-active, non-visited links. For light backgrounds, traditionally blue.

<a id="valdef-color-mark"></a>Mark  
<a id="ref-for-the-mark-element"></a>

 Background of text that has been specially marked (such as by the HTML <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-mark-element">mark</a></code> element).

<a id="valdef-color-marktext"></a>MarkText  
<a id="ref-for-the-mark-element①"></a>

 Text that has been specially marked (such as by the HTML <code><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-mark-element">mark</a></code> element).

<a id="valdef-color-selecteditem"></a>SelectedItem  
 Background of selected items, for example a selected checkbox.

<a id="valdef-color-selecteditemtext"></a>SelectedItemText  
 Text of selected items.

<a id="valdef-color-visitedtext"></a>VisitedText  
 Text in visited links. For light backgrounds, traditionally purple.

Tests

- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)
- [relative-currentcolor-visited-getcomputedstyle.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-visited-getcomputedstyle.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-visited-getcomputedstyle.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-visited-getcomputedstyle.html)
- [system-color-compute.html](https://wpt.fyi/results/css/css-color/system-color-compute.html) [(live test)](http://wpt.live/css/css-color/system-color-compute.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/system-color-compute.html)
- [system-color-hightlights-vs-getSelection-001.html](https://wpt.fyi/results/css/css-color/system-color-hightlights-vs-getSelection-001.html) [(live test)](http://wpt.live/css/css-color/system-color-hightlights-vs-getSelection-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/system-color-hightlights-vs-getSelection-001.html)
- [system-color-hightlights-vs-getSelection-002.html](https://wpt.fyi/results/css/css-color/system-color-hightlights-vs-getSelection-002.html) [(live test)](http://wpt.live/css/css-color/system-color-hightlights-vs-getSelection-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/system-color-hightlights-vs-getSelection-002.html)

<a id="ref-for-css-keyword"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As with all other [keywords](https://www.w3.org/TR/css-values-4/#css-keyword), these names are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive). They are shown here with mixed capitalization for legibility.

For systems that do not have a particular system UI concept, the specified value should be mapped to the most closely related system color value that exists. The following <a id="system-color-pairings"></a>system color pairings are expected to form legible background-foreground colors:

<a id="system-color-pairs"></a>

- <a id="ref-for-valdef-color-canvas①"></a>

  <a id="ref-for-valdef-color-canvastext"></a>

  <a id="ref-for-valdef-color-linktext"></a>

  <a id="ref-for-valdef-color-visitedtext"></a>

  <a id="ref-for-valdef-color-activetext"></a>

  [Canvas](#valdef-color-canvas) background with [CanvasText](#valdef-color-canvastext), [LinkText](#valdef-color-linktext), [VisitedText](#valdef-color-visitedtext), [ActiveText](#valdef-color-activetext) foreground.

- <a id="ref-for-valdef-color-canvas②"></a>

  <a id="ref-for-valdef-color-buttonborder"></a>

  [Canvas](#valdef-color-canvas) background with a [ButtonBorder](#valdef-color-buttonborder) border and adjacent color <a id="ref-for-valdef-color-canvas③"></a>Canvas

- <a id="ref-for-valdef-color-buttonface"></a>

  <a id="ref-for-valdef-color-buttontext①"></a>

  [ButtonFace](#valdef-color-buttonface) background with [ButtonText](#valdef-color-buttontext) foreground.

- <a id="ref-for-valdef-color-field"></a>

  <a id="ref-for-valdef-color-fieldtext"></a>

  [Field](#valdef-color-field) background with [FieldText](#valdef-color-fieldtext) foreground.

- <a id="ref-for-valdef-color-mark"></a>

  <a id="ref-for-valdef-color-marktext"></a>

  [Mark](#valdef-color-mark) background with [MarkText](#valdef-color-marktext) foreground

- <a id="ref-for-valdef-color-buttonface①"></a>

  <a id="ref-for-valdef-color-field①"></a>

  <a id="ref-for-valdef-color-buttonborder①"></a>

  <a id="ref-for-valdef-color-canvas④"></a>

  [ButtonFace](#valdef-color-buttonface) or [Field](#valdef-color-field) background with a [ButtonBorder](#valdef-color-buttonborder) border and adjacent color [Canvas](#valdef-color-canvas)'

- <a id="ref-for-valdef-color-highlight"></a>

  <a id="ref-for-valdef-color-highlighttext"></a>

  [Highlight](#valdef-color-highlight) background with [HighlightText](#valdef-color-highlighttext) foreground.

- <a id="ref-for-valdef-color-selecteditem"></a>

  <a id="ref-for-valdef-color-selecteditemtext"></a>

  [SelectedItem](#valdef-color-selecteditem) background with [SelectedItemText](#valdef-color-selecteditemtext) foreground.

- <a id="ref-for-valdef-color-accentcolor"></a>

  <a id="ref-for-valdef-color-accentcolortext"></a>

  [AccentColor](#valdef-color-accentcolor) background with [AccentColorText](#valdef-color-accentcolortext) foreground.

<a id="ref-for-valdef-color-graytext"></a>

Additionally, [GrayText](#valdef-color-graytext) is expected to be readable, though possibly at a lower contrast rating, over any of the backgrounds.

<a id="ref-for-accent-color"></a>

<a id="ref-for-valdef-color-accentcolor①"></a>

<a id="ref-for-propdef-accent-color"></a>

<a id="ref-for-forced-colors-mode①"></a>

<a id="ref-for-valdef-color-accentcolortext①"></a>

To maintain consistency with widget [accent color](https://www.w3.org/TR/css-ui-4/#accent-color) styling, [AccentColor](#valdef-color-accentcolor) takes its value from [accent-color](https://www.w3.org/TR/css-ui-4/#propdef-accent-color), unless [Forced Colors Mode](https://www.w3.org/TR/css-color-adjust-1/#forced-colors-mode) is enabled. [AccentColorText](#valdef-color-accentcolortext) takes its value from the contrasting foreground color to <a id="ref-for-valdef-color-accentcolor②"></a>AccentColor as is described for widget <a id="ref-for-accent-color①"></a>accent color styling.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-SystemCombo"></a> For example, the system color combinations in the browser you are currently using:
>
> Canvas with CanvasText: CanvasText
>
> Canvas with LinkText: LinkText
>
> Canvas with VisitedText: VisitedText
>
> Canvas with ActiveText: ActiveText
>
> Canvas with GrayText: GrayText
>
> Canvas with ButtonBorder and adjacent Canvas: CanvasTextAdjacent
>
> ButtonFace with ButtonText: ButtonText
>
> ButtonFace with ButtonText and ButtonBorder: ButtonText
>
> ButtonFace with GrayText: GrayText
>
> Field with FieldText: FieldText
>
> Field with GrayText: GrayText
>
> Mark with MarkText: MarkText
>
> Mark with GrayText: GrayText
>
> Highlight with HighlightText: HighlightText
>
> Highlight with GrayText: GrayText
>
> SelectedItem with SelectedItemText: SelectedItemText
>
> AccentColor with AccentColorText: AccentColorText
>
> AccentColor with GrayText: GrayText

<a id="ref-for-typedef-system-color⑧"></a>

Earlier versions of CSS defined additional [\<system-color\>](#typedef-system-color)s, which have since been deprecated. These are documented in [Appendix A: Deprecated CSS System Colors](#deprecated-system-colors).

<a id="ref-for-typedef-system-color⑨"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [\<system-color\>](#typedef-system-color)s incur some privacy and security risk, as detailed in [§ 22 Privacy Considerations](#privacy) and [§ 21 Security Considerations](#security).

User agents may, to mitigate privacy and security risks such as fingerprinting, elect to return fixed values for the used value of system colors which do not reflect customisation or theming choices made by the user.

<a id="ref-for-valdef-color-transparent②"></a>

### <a id="transparent-color"></a>6.3.  The [transparent](#valdef-color-transparent) keyword

<a id="ref-for-transparent-black"></a>

<a id="ref-for-typedef-named-color①"></a>

The keyword <a id="valdef-color-transparent"></a>transparent specifies a [transparent black](#transparent-black). It is a type of [\<named-color\>](#typedef-named-color).

Tests

- [color-computed.html](https://wpt.fyi/results/css/css-color/parsing/color-computed.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)
- [t423-transparent-1-a.xht](https://wpt.fyi/results/css/css-color/t423-transparent-1-a.xht) [(live test)](http://wpt.live/css/css-color/t423-transparent-1-a.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/t423-transparent-1-a.xht)
- [t423-transparent-2-a.xht](https://wpt.fyi/results/css/css-color/t423-transparent-2-a.xht) [(live test)](http://wpt.live/css/css-color/t423-transparent-2-a.xht) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/t423-transparent-2-a.xht)

<a id="ref-for-valdef-color-currentcolor⑤"></a>

### <a id="currentcolor-color"></a>6.4.  The [currentcolor](#valdef-color-currentcolor) keyword

<a id="ref-for-propdef-color②"></a>

<a id="ref-for-typedef-named-color②"></a>

<a id="ref-for-typedef-color①⑦"></a>

<a id="ref-for-used-value①"></a>

The keyword <a id="valdef-color-currentcolor"></a>currentcolor represents value of the [color](#propdef-color) property on the same element. Unlike [\<named-color\>](#typedef-named-color)s, it is <em>not</em> restricted to sRGB; the value can be any [\<color\>](#typedef-color). Its [used values](https://www.w3.org/TR/css-cascade-5/#used-value) is determined by [resolving color values](#resolving-other-colors).

Tests

- [border-color-currentcolor.html](https://wpt.fyi/results/css/css-color/border-color-currentcolor.html) [(live test)](http://wpt.live/css/css-color/border-color-currentcolor.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/border-color-currentcolor.html)
- [color-mix-currentcolor-nested-for-color-property.html](https://wpt.fyi/results/css/css-color/color-mix-currentcolor-nested-for-color-property.html) [(live test)](http://wpt.live/css/css-color/color-mix-currentcolor-nested-for-color-property.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-currentcolor-nested-for-color-property.html)
- [currentcolor-001.html](https://wpt.fyi/results/css/css-color/currentcolor-001.html) [(live test)](http://wpt.live/css/css-color/currentcolor-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/currentcolor-001.html)
- [currentcolor-002.html](https://wpt.fyi/results/css/css-color/currentcolor-002.html) [(live test)](http://wpt.live/css/css-color/currentcolor-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/currentcolor-002.html)
- [currentcolor-003.html](https://wpt.fyi/results/css/css-color/currentcolor-003.html) [(live test)](http://wpt.live/css/css-color/currentcolor-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/currentcolor-003.html)
- [currentcolor-004.html](https://wpt.fyi/results/css/css-color/currentcolor-004.html) [(live test)](http://wpt.live/css/css-color/currentcolor-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/currentcolor-004.html)
- [currentcolor-visited-fallback.html](https://wpt.fyi/results/css/css-color/currentcolor-visited-fallback.html) [(live test)](http://wpt.live/css/css-color/currentcolor-visited-fallback.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/currentcolor-visited-fallback.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

<a id="ref-for-valdef-color-currentcolor⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-currentcolor"></a> Here’s a simple example showing how to use the [currentcolor](#valdef-color-currentcolor) keyword:
>
> ```css
> .foo {
>   color:  red;
>   background-color:  currentcolor;
> }
> ```
>
> This is equivalent to writing:
>
> ```css
> .foo {
>   color:  red;
>   background-color:  red;
> }
> ```
<a id="ref-for-propdef-text-emphasis-color①"></a>

<a id="ref-for-valdef-color-currentcolor⑦"></a>

<a id="ref-for-propdef-color③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-textemph-currentcolor"></a> For example, the [text-emphasis-color](https://www.w3.org/TR/css-text-decor-4/#propdef-text-emphasis-color) property [\[CSS3-TEXT-DECOR\]](#biblio-css3-text-decor), whose initial value is [currentcolor](#valdef-color-currentcolor), by default matches the text color even as the [color](#propdef-color) property changes across elements.
>
> ```markup
> <p><em>Some <strong>really</strong> emphasized text.</em>
> <style>
> p { color: black; }
> em { text-emphasis: dot; }
> strong { color: red; }
> </style>
> ```
>
> ![rendered emphasized text with the word 'really' in red with red emphasis dots](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/text-emphasis.png)
>
> In the above example, the emphasis marks are black over the text "Some" and "emphasized text", but red over the text "really".

<a id="ref-for-valdef-color-currentcolor⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="trivia"></a>Note: Multi-word keywords in CSS usually separate their component words with hyphens. [currentcolor](#valdef-color-currentcolor) doesn’t, because (deep breath) it was originally introduced in SVG as a property value, "current-color" with the usual CSS spelling. It (along with all other properties and their values) then became presentation attributes and attribute values, as well as properties, to make generation with XSLT easier. Then all of the presentation attributes were changed from hyphenated to camelCase, because the DOM had an issue with hyphen meaning "minus". But then, they didn’t follow CSS conventions anymore so all the properties and property values that were <em>already</em> part of CSS were changed back to hyphenated! <a id="ref-for-valdef-color-currentcolor⑨"></a>currentcolor was not a part of CSS at that time, so remained camelCased. Only later did CSS pick it up, at which point the capitalization stopped mattering, as CSS keywords are [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive).

<a id="ref-for-funcdef-hsl①⓪"></a>

<a id="ref-for-funcdef-hsla⑧"></a>

## <a id="the-hsl-notation"></a>7.  HSL Colors: [hsl()](#funcdef-hsl) and [hsla()](#funcdef-hsla) functions

The RGB system for specifying colors, while convenient for machines and graphic libraries, is often regarded as very difficult for humans to gain an intuitive grasp on. It’s not easy to tell, for example, how to alter an RGB color to produce a lighter variant of the same hue.

There are several other color schemes possible. One such is the HSL [\[HSL\]](#biblio-hsl) color scheme, which is more intuitive to use, but still maps easily back to RGB colors.

<a id="ref-for-funcdef-hsl①①"></a>

<a id="ref-for-funcdef-hsla⑨"></a>

<a id="valdef-hsl-hsl"></a>HSL colors are specified as a triplet of hue, saturation, and lightness. The syntax of the [hsl()](#funcdef-hsl) and [hsla()](#funcdef-hsla) functions is:

<a id="funcdef-hsl"></a>

<a id="ref-for-typedef-legacy-hsl-syntax"></a>

<a id="ref-for-comb-one②⑦"></a>

<a id="ref-for-typedef-modern-hsl-syntax"></a>

<a id="funcdef-hsla"></a>

<a id="ref-for-typedef-legacy-hsla-syntax"></a>

<a id="ref-for-comb-one②⑧"></a>

<a id="ref-for-typedef-modern-hsla-syntax"></a>

<a id="typedef-modern-hsl-syntax"></a>

<a id="ref-for-typedef-modern-hsl-syntax①"></a>

<a id="ref-for-typedef-hue⑦"></a>

<a id="ref-for-comb-one②⑨"></a>

<a id="ref-for-percentage-value⑨"></a>

<a id="ref-for-comb-one③⓪"></a>

<a id="ref-for-number-value①①"></a>

<a id="ref-for-comb-one③①"></a>

<a id="ref-for-percentage-value①⓪"></a>

<a id="ref-for-comb-one③②"></a>

<a id="ref-for-number-value①②"></a>

<a id="ref-for-comb-one③③"></a>

<a id="ref-for-typedef-color-alpha-value⑨"></a>

<a id="ref-for-comb-one③④"></a>

<a id="ref-for-mult-opt⑥"></a>

<a id="typedef-modern-hsla-syntax"></a>

<a id="ref-for-typedef-modern-hsla-syntax①"></a>

<a id="ref-for-typedef-hue⑧"></a>

<a id="ref-for-comb-one③⑤"></a>

<a id="ref-for-percentage-value①①"></a>

<a id="ref-for-comb-one③⑥"></a>

<a id="ref-for-number-value①③"></a>

<a id="ref-for-comb-one③⑦"></a>

<a id="ref-for-percentage-value①②"></a>

<a id="ref-for-comb-one③⑧"></a>

<a id="ref-for-number-value①④"></a>

<a id="ref-for-comb-one③⑨"></a>

<a id="ref-for-typedef-color-alpha-value①⓪"></a>

<a id="ref-for-comb-one④⓪"></a>

<a id="ref-for-mult-opt⑦"></a>

<a id="typedef-legacy-hsl-syntax"></a>

<a id="ref-for-typedef-legacy-hsl-syntax①"></a>

<a id="ref-for-typedef-hue⑨"></a>

<a id="ref-for-comb-comma④"></a>

<a id="ref-for-percentage-value①③"></a>

<a id="ref-for-comb-comma⑤"></a>

<a id="ref-for-percentage-value①④"></a>

<a id="ref-for-comb-comma⑥"></a>

<a id="ref-for-typedef-color-alpha-value①①"></a>

<a id="ref-for-mult-opt⑧"></a>

<a id="typedef-legacy-hsla-syntax"></a>

<a id="ref-for-typedef-legacy-hsla-syntax①"></a>

<a id="ref-for-typedef-hue①⓪"></a>

<a id="ref-for-comb-comma⑦"></a>

<a id="ref-for-percentage-value①⑤"></a>

<a id="ref-for-comb-comma⑧"></a>

<a id="ref-for-percentage-value①⑥"></a>

<a id="ref-for-comb-comma⑨"></a>

<a id="ref-for-typedef-color-alpha-value①②"></a>

<a id="ref-for-mult-opt⑨"></a>

```text
hsl() = [ <legacy-hsl-syntax> | <modern-hsl-syntax> ]
hsla() = [ <legacy-hsla-syntax> | <modern-hsla-syntax> ]
<modern-hsl-syntax> = hsl(
    [<hue> | none]
    [<percentage> | <number> | none]
    [<percentage> | <number> | none]
    [ / [<alpha-value> | none] ]? )
<modern-hsla-syntax> = hsla(
    [<hue> | none]
    [<percentage> | <number> | none]
    [<percentage> | <number> | none]
    [ / [<alpha-value> | none] ]? )
<legacy-hsl-syntax> = hsl( <hue>, <percentage>, <percentage>, <alpha-value>? )
<legacy-hsla-syntax> = hsla( <hue>, <percentage>, <percentage>, <alpha-value>? )
```
<a id="prr-hsl"></a>



| Field               | Definition                          |
|---------------------|-------------------------------------|
| <strong>Percentages&#xA;      </strong> | Allowed for S and L                 |
| <strong>Percent reference range&#xA0;&#xA;      </strong> | for S and L: 0% = 0.0, 100% = 100.0 |
| <strong>Powerless hue ε&#xA;      </strong> | S \<= 0.001                         |



Tests

- [hsl-001.html](https://wpt.fyi/results/css/css-color/hsl-001.html) [(live test)](http://wpt.live/css/css-color/hsl-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsl-001.html)
- [hsl-002.html](https://wpt.fyi/results/css/css-color/hsl-002.html) [(live test)](http://wpt.live/css/css-color/hsl-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsl-002.html)
- [hsl-003.html](https://wpt.fyi/results/css/css-color/hsl-003.html) [(live test)](http://wpt.live/css/css-color/hsl-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsl-003.html)
- [hsl-004.html](https://wpt.fyi/results/css/css-color/hsl-004.html) [(live test)](http://wpt.live/css/css-color/hsl-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsl-004.html)
- [hsl-005.html](https://wpt.fyi/results/css/css-color/hsl-005.html) [(live test)](http://wpt.live/css/css-color/hsl-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsl-005.html)
- [hsl-006.html](https://wpt.fyi/results/css/css-color/hsl-006.html) [(live test)](http://wpt.live/css/css-color/hsl-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsl-006.html)
- [hsl-007.html](https://wpt.fyi/results/css/css-color/hsl-007.html) [(live test)](http://wpt.live/css/css-color/hsl-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsl-007.html)
- [hsl-008.html](https://wpt.fyi/results/css/css-color/hsl-008.html) [(live test)](http://wpt.live/css/css-color/hsl-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsl-008.html)
- [hsl-clamp-negative-saturation.html](https://wpt.fyi/results/css/css-color/hsl-clamp-negative-saturation.html) [(live test)](http://wpt.live/css/css-color/hsl-clamp-negative-saturation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsl-clamp-negative-saturation.html)
- [background-color-hsl-001.html](https://wpt.fyi/results/css/css-color/background-color-hsl-001.html) [(live test)](http://wpt.live/css/css-color/background-color-hsl-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/background-color-hsl-001.html)
- [background-color-hsl-002.html](https://wpt.fyi/results/css/css-color/background-color-hsl-002.html) [(live test)](http://wpt.live/css/css-color/background-color-hsl-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/background-color-hsl-002.html)
- [background-color-hsl-003.html](https://wpt.fyi/results/css/css-color/background-color-hsl-003.html) [(live test)](http://wpt.live/css/css-color/background-color-hsl-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/background-color-hsl-003.html)
- [background-color-hsl-004.html](https://wpt.fyi/results/css/css-color/background-color-hsl-004.html) [(live test)](http://wpt.live/css/css-color/background-color-hsl-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/background-color-hsl-004.html)
- [color-computed-hsl.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-hsl.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-hsl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-hsl.html)
- [color-invalid-hsl.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-hsl.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-hsl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-hsl.html)
- [color-valid-hsl.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-hsl.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-hsl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-hsl.html)

The first argument specifies the hue angle.

In HSL (and HWB) the angle 0deg represents sRGB primary red (as does 360deg, 720deg, etc.), and the rest of the hues are spread around the circle, so 120deg represents sRGB primary green, 240deg represents sRGB primary blue, etc.

The next two arguments are the saturation and lightness, respectively. For saturation, 100% or 100 is a fully-saturated, bright color, and 0% or 0 is a fully-unsaturated gray. For lightness, 50% or 50 represents the "normal" color, while 100% or 100 is white and 0% or 0 is black.

For historical reasons, if the saturation is less than 0% it is clamped to 0% at parsed-value time, before being converted to an sRGB color.

Tests

- [color-valid-hsl.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-hsl.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-hsl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-hsl.html)

<a id="ref-for-funcdef-rgb①⓪"></a>

The final argument specifies the alpha component of the color. It’s interpreted identically to the fourth argument of the [rgb()](#funcdef-rgb) function. If omitted, it defaults to 100%.

HSL colors resolve to sRGB.

<a id="ref-for-powerless-color-component⑧"></a>

If the saturation of an HSL color is 0% or 0, then the hue component is [powerless](#powerless-color-component).

<a id="ref-for-valdef-color-red"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-hsl-primary-red"></a> For example, an ordinary red, the same color you would see from the keyword  [red](#valdef-color-red) or the hex notation  \#f00, is represented in HSL as  hsl(0deg 100% 50%).

An advantage of HSL over RGB is that it is more intuitive: people can guess at the colors they want, and then tweak.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-hsl-tweak"></a> For example, the following colors can all be generated off of the basic "green" hue, just by varying the other two arguments:
>
> ```css
> hsl(120deg 100% 50%) lime green
> hsl(120deg 100% 25%) dark green
> hsl(120deg 100% 75%) light green
> hsl(120deg 75% 85%)  pastel green
> ```
<a id="disadvantage-hsl"></a>A disadvantage of HSL over OkLCh is that hue manipulation changes the visual lightness, and that hues are not evenly spaced apart.

It is thus easier in HSL to create sets of matching colors (by keeping the hue the same and varying the saturation and lightness), compared to manipulating the sRGB component values; however, because the lightness is simply the mean of the gamma-corrected red, green and blue components it does not correspond to the visual perception of lightness across hues.

<a id="ref-for-valdef-color-blue"></a>

<a id="ref-for-valdef-color-yellow"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-hsl-sucks"></a> For example,  [blue](#valdef-color-blue) is represented in HSL as  hsl(240deg 100% 50%) while  [yellow](#valdef-color-yellow) is  hsl(60deg 100% 50%). Both have an HSL Lightness of 50%, but clearly the yellow looks much lighter than the blue.
>
> In OkLCh, sRGB blue is  oklch(0.452 0.313 264.1) while sRGB yellow is  oklch(0.968 0.211 109.8). The OkLCh Lightnesses of 0.452 and 0.968 clearly reflect the visual lightnesses of the two colors.

The hue angle in HSL is not perceptually uniform; colors appear bunched up in some areas and widely spaced in others.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-hsl-sucks-more"></a> For example, the pair of hues  hsl(220deg 100% 50%) and  hsl(250deg 100% 50%) have an HSL hue difference of 250-220 = <strong>30</strong>deg and look fairly similar, while another pair of colors  hsl(50deg 100% 50%) and  hsl(80deg 100% 50%), which <em>also</em> have a hue difference of 80-50 = <strong>30</strong>deg, look very different.
>
> In OkLCh, the same pair of colors  oklch(0.533 0.26 262.6) and  oklch(0.462 0.306 268.9) have a hue difference of 268.9 - 262.6 = <strong>6.3</strong>deg while the second pair  oklch(0.882 0.181 94.24) and  oklch(0.91 0.245 129.9) have a hue difference of 129.9 - 94.24 = <strong>35.66</strong>deg, correctly reflecting the visual separation of hues.

<a id="ref-for-funcdef-hsl①②"></a>

<a id="ref-for-funcdef-hsla①⓪"></a>

<a id="ref-for-legacy-color-syntax④"></a>

For historical reasons, [hsl()](#funcdef-hsl) and [hsla()](#funcdef-hsla) also support a [legacy color syntax](#legacy-color-syntax).

Tests

- [hsla-001.html](https://wpt.fyi/results/css/css-color/hsla-001.html) [(live test)](http://wpt.live/css/css-color/hsla-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsla-001.html)
- [hsla-002.html](https://wpt.fyi/results/css/css-color/hsla-002.html) [(live test)](http://wpt.live/css/css-color/hsla-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsla-002.html)
- [hsla-003.html](https://wpt.fyi/results/css/css-color/hsla-003.html) [(live test)](http://wpt.live/css/css-color/hsla-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsla-003.html)
- [hsla-004.html](https://wpt.fyi/results/css/css-color/hsla-004.html) [(live test)](http://wpt.live/css/css-color/hsla-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsla-004.html)
- [hsla-005.html](https://wpt.fyi/results/css/css-color/hsla-005.html) [(live test)](http://wpt.live/css/css-color/hsla-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsla-005.html)
- [hsla-006.html](https://wpt.fyi/results/css/css-color/hsla-006.html) [(live test)](http://wpt.live/css/css-color/hsla-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsla-006.html)
- [hsla-007.html](https://wpt.fyi/results/css/css-color/hsla-007.html) [(live test)](http://wpt.live/css/css-color/hsla-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsla-007.html)
- [hsla-008.html](https://wpt.fyi/results/css/css-color/hsla-008.html) [(live test)](http://wpt.live/css/css-color/hsla-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsla-008.html)
- [hsla-clamp-negative-saturation.html](https://wpt.fyi/results/css/css-color/hsla-clamp-negative-saturation.html) [(live test)](http://wpt.live/css/css-color/hsla-clamp-negative-saturation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hsla-clamp-negative-saturation.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

### <a id="hsl-to-rgb"></a>7.1.  Converting HSL Colors to sRGB

Converting an HSL color to sRGB is straightforward mathematically. Here’s a sample implementation of the conversion algorithm in JavaScript. It returns an array of three numbers representing the red, green, and blue components of the colors, which for colors in the sRGB gamut will be in the range \[0, 1\].

This code assumes that <em>parse-time</em> clamping of negative saturation has already been applied.

```javascript
/**
 * @param {number} hue - Hue as degrees 0..360
 * @param {number} sat - Saturation in reference range [0,100]
 * @param {number} light - Lightness in reference range [0,100]
 * @return {number[]} Array of sRGB components; in-gamut colors in range [0..1]
 */
function hslToRgb(hue, sat, light) {

    sat /= 100;
    light /= 100;

    function f(n) {
        let k = (n + hue/30) % 12;
        let a = sat * Math.min(light, 1 - light);
        return light - a * Math.max(-1, Math.min(k - 3, 9 - k, 1));
    }

    return [f(0), f(8), f(4)];
}
```
### <a id="rgb-to-hsl"></a>7.2.  Converting sRGB Colors to HSL

Conversion in the reverse direction proceeds similarly.

Special care is taken to deal with intermediate negative values of saturation, which can be produced by colors far outside the sRGB gamut.

```javascript
/**
 * @param {number} red - Red component 0..1
 * @param {number} green - Green component 0..1
 * @param {number} blue - Blue component 0..1
 * @return {number[]} Array of HSL values: Hue as degrees 0..360, Saturation and Lightness in reference range [0,100]
 */
function rgbToHsl (red, green, blue) {
    let max = Math.max(red, green, blue);
    let min = Math.min(red, green, blue);
    let [hue, sat, light] = [NaN, 0, (min + max)/2];
    let d = max - min;
    let epsilon = 1 / 100000;   // max Sat is 1, in this code

    if (d !== 0) {
        sat = (light === 0 || light === 1)
            ? 0
            : (max - light) / Math.min(light, 1 - light);

        switch (max) {
            case red:   hue = (green - blue) / d + (green < blue ? 6 : 0); break;
            case green: hue = (blue - red) / d + 2; break;
            case blue:  hue = (red - green) / d + 4;
        }

        hue = hue * 60;
    }

    // Very out of gamut colors can produce negative saturation
    // If so, just rotate the hue by 180 and use a positive saturation
    // see https://github.com/w3c/csswg-drafts/issues/9222
    if (sat < 0) {
        hue += 180;
        sat = Math.abs(sat);
    }

    if (hue >= 360) {
        hue -= 360;
    }

    if (sat <= epsilon) {
        hue = NaN;
    }

    return [hue, sat * 100, light * 100];
}
```
### <a id="hsl-examples"></a>7.3.  Examples of HSL Colors

<em>This section is not normative.</em>

Tests

This section is not normative, it does not need tests.

------------------------------------------------------------------------

The tables below illustrate a wide range of possible HSL colors. Each table represents one hue, selected at 30° intervals, to illustrate the common "core" hues: red, yellow, green, cyan, blue, magenta, and the six intermediary colors between these.

In each table, the X axis represents the saturation while the Y axis represents the lightness.

<a id="hsltable"></a>

**0° Reds**

Columns (X): saturation; rows (Y): lightness. Each entry is the exact source `background-color` value.

|  | 100% | 80% | 60% | 40% | 20% | 0% |
| --- | --- | --- | --- | --- | --- | --- |
| **100%** | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` |
| **90%** | `rgb(255, 204, 204)` | `rgb(250, 209, 209)` | `rgb(245, 214, 214)` | `rgb(240, 219, 219)` | `rgb(235, 224, 224)` | `rgb(230, 230, 230)` |
| **80%** | `rgb(255, 153, 153)` | `rgb(245, 163, 163)` | `rgb(235, 173, 173)` | `rgb(224, 184, 184)` | `rgb(214, 194, 194)` | `rgb(204, 204, 204)` |
| **70%** | `rgb(255, 102, 102)` | `rgb(240, 117, 117)` | `rgb(224, 133, 133)` | `rgb(209, 148, 148)` | `rgb(194, 163, 163)` | `rgb(179, 179, 179)` |
| **60%** | `rgb(255, 51, 51)` | `rgb(235, 71, 71)` | `rgb(214, 92, 92)` | `rgb(194, 112, 112)` | `rgb(173, 133, 133)` | `rgb(153, 153, 153)` |
| **50%** | `rgb(255, 0, 0)` | `rgb(230, 26, 26)` | `rgb(204, 51, 51)` | `rgb(179, 77, 77)` | `rgb(153, 102, 102)` | `rgb(128, 128, 128)` |
| **40%** | `rgb(204, 0, 0)` | `rgb(184, 20, 20)` | `rgb(163, 41, 41)` | `rgb(143, 61, 61)` | `rgb(122, 82, 82)` | `rgb(102, 102, 102)` |
| **30%** | `rgb(153, 0, 0)` | `rgb(138, 15, 15)` | `rgb(122, 31, 31)` | `rgb(107, 46, 46)` | `rgb(92, 61, 61)` | `rgb(77, 77, 77)` |
| **20%** | `rgb(102, 0, 0)` | `rgb(92, 10, 10)` | `rgb(82, 20, 20)` | `rgb(71, 31, 31)` | `rgb(61, 41, 41)` | `rgb(51, 51, 51)` |
| **10%** | `rgb(51, 0, 0)` | `rgb(46, 5, 5)` | `rgb(41, 10, 10)` | `rgb(36, 15, 15)` | `rgb(31, 20, 20)` | `rgb(26, 26, 26)` |
| **0%** | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` |

**30° Reds-Yellows (=Oranges)**

Columns (X): saturation; rows (Y): lightness. Each entry is the exact source `background-color` value.

|  | 100% | 80% | 60% | 40% | 20% | 0% |
| --- | --- | --- | --- | --- | --- | --- |
| **100%** | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` |
| **90%** | `rgb(255, 230, 204)` | `rgb(250, 230, 209)` | `rgb(245, 230, 214)` | `rgb(240, 230, 219)` | `rgb(235, 230, 224)` | `rgb(230, 230, 230)` |
| **80%** | `rgb(255, 204, 153)` | `rgb(245, 204, 163)` | `rgb(235, 204, 173)` | `rgb(224, 204, 184)` | `rgb(214, 204, 194)` | `rgb(204, 204, 204)` |
| **70%** | `rgb(255, 179, 102)` | `rgb(240, 179, 117)` | `rgb(224, 179, 133)` | `rgb(209, 179, 148)` | `rgb(194, 179, 163)` | `rgb(179, 179, 179)` |
| **60%** | `rgb(255, 153, 51)` | `rgb(235, 153, 71)` | `rgb(214, 153, 92)` | `rgb(194, 153, 112)` | `rgb(173, 153, 133)` | `rgb(153, 153, 153)` |
| **50%** | `rgb(255, 128, 0)` | `rgb(230, 128, 26)` | `rgb(204, 128, 51)` | `rgb(179, 128, 77)` | `rgb(153, 128, 102)` | `rgb(128, 128, 128)` |
| **40%** | `rgb(204, 102, 0)` | `rgb(184, 102, 20)` | `rgb(163, 102, 41)` | `rgb(143, 102, 61)` | `rgb(122, 102, 82)` | `rgb(102, 102, 102)` |
| **30%** | `rgb(153, 77, 0)` | `rgb(138, 77, 15)` | `rgb(122, 77, 31)` | `rgb(107, 77, 46)` | `rgb(92, 77, 61)` | `rgb(77, 77, 77)` |
| **20%** | `rgb(102, 51, 0)` | `rgb(92, 51, 10)` | `rgb(82, 51, 20)` | `rgb(71, 51, 31)` | `rgb(61, 51, 41)` | `rgb(51, 51, 51)` |
| **10%** | `rgb(51, 26, 0)` | `rgb(46, 26, 5)` | `rgb(41, 26, 10)` | `rgb(36, 26, 15)` | `rgb(31, 26, 20)` | `rgb(26, 26, 26)` |
| **0%** | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` |

**60° Yellows**

Columns (X): saturation; rows (Y): lightness. Each entry is the exact source `background-color` value.

|  | 100% | 80% | 60% | 40% | 20% | 0% |
| --- | --- | --- | --- | --- | --- | --- |
| **100%** | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` |
| **90%** | `rgb(255, 255, 204)` | `rgb(250, 250, 209)` | `rgb(245, 245, 214)` | `rgb(240, 240, 219)` | `rgb(235, 235, 224)` | `rgb(230, 230, 230)` |
| **80%** | `rgb(255, 255, 153)` | `rgb(245, 245, 163)` | `rgb(235, 235, 173)` | `rgb(224, 224, 184)` | `rgb(214, 214, 194)` | `rgb(204, 204, 204)` |
| **70%** | `rgb(255, 255, 102)` | `rgb(240, 240, 117)` | `rgb(224, 224, 133)` | `rgb(209, 209, 148)` | `rgb(194, 194, 163)` | `rgb(179, 179, 179)` |
| **60%** | `rgb(255, 255, 51)` | `rgb(235, 235, 71)` | `rgb(214, 214, 92)` | `rgb(194, 194, 112)` | `rgb(173, 173, 133)` | `rgb(153, 153, 153)` |
| **50%** | `rgb(255, 255, 0)` | `rgb(230, 230, 26)` | `rgb(204, 204, 51)` | `rgb(179, 179, 77)` | `rgb(153, 153, 102)` | `rgb(128, 128, 128)` |
| **40%** | `rgb(204, 204, 0)` | `rgb(184, 184, 20)` | `rgb(163, 163, 41)` | `rgb(143, 143, 61)` | `rgb(122, 122, 82)` | `rgb(102, 102, 102)` |
| **30%** | `rgb(153, 153, 0)` | `rgb(138, 138, 15)` | `rgb(122, 122, 31)` | `rgb(107, 107, 46)` | `rgb(92, 92, 61)` | `rgb(77, 77, 77)` |
| **20%** | `rgb(102, 102, 0)` | `rgb(92, 92, 10)` | `rgb(82, 82, 20)` | `rgb(71, 71, 31)` | `rgb(61, 61, 41)` | `rgb(51, 51, 51)` |
| **10%** | `rgb(51, 51, 0)` | `rgb(46, 46, 5)` | `rgb(41, 41, 10)` | `rgb(36, 36, 15)` | `rgb(31, 31, 20)` | `rgb(26, 26, 26)` |
| **0%** | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` |

**90° Yellow-Greens**

Columns (X): saturation; rows (Y): lightness. Each entry is the exact source `background-color` value.

|  | 100% | 80% | 60% | 40% | 20% | 0% |
| --- | --- | --- | --- | --- | --- | --- |
| **100%** | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` |
| **90%** | `rgb(230, 255, 204)` | `rgb(230, 250, 209)` | `rgb(230, 245, 214)` | `rgb(230, 240, 219)` | `rgb(230, 235, 224)` | `rgb(230, 230, 230)` |
| **80%** | `rgb(204, 255, 153)` | `rgb(204, 245, 163)` | `rgb(204, 235, 173)` | `rgb(204, 224, 184)` | `rgb(204, 214, 194)` | `rgb(204, 204, 204)` |
| **70%** | `rgb(179, 255, 102)` | `rgb(179, 240, 117)` | `rgb(179, 224, 133)` | `rgb(179, 209, 148)` | `rgb(179, 194, 163)` | `rgb(179, 179, 179)` |
| **60%** | `rgb(153, 255, 51)` | `rgb(153, 235, 71)` | `rgb(153, 214, 92)` | `rgb(153, 194, 112)` | `rgb(153, 173, 133)` | `rgb(153, 153, 153)` |
| **50%** | `rgb(128, 255, 0)` | `rgb(128, 230, 26)` | `rgb(128, 204, 51)` | `rgb(128, 179, 77)` | `rgb(128, 153, 102)` | `rgb(128, 128, 128)` |
| **40%** | `rgb(102, 204, 0)` | `rgb(102, 184, 20)` | `rgb(102, 163, 41)` | `rgb(102, 143, 61)` | `rgb(102, 122, 82)` | `rgb(102, 102, 102)` |
| **30%** | `rgb(77, 153, 0)` | `rgb(77, 138, 15)` | `rgb(77, 122, 31)` | `rgb(77, 107, 46)` | `rgb(77, 92, 61)` | `rgb(77, 77, 77)` |
| **20%** | `rgb(51, 102, 0)` | `rgb(51, 92, 10)` | `rgb(51, 82, 20)` | `rgb(51, 71, 31)` | `rgb(51, 61, 41)` | `rgb(51, 51, 51)` |
| **10%** | `rgb(26, 51, 0)` | `rgb(26, 46, 5)` | `rgb(26, 41, 10)` | `rgb(26, 36, 15)` | `rgb(26, 31, 20)` | `rgb(26, 26, 26)` |
| **0%** | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` |

**120° Greens**

Columns (X): saturation; rows (Y): lightness. Each entry is the exact source `background-color` value.

|  | 100% | 80% | 60% | 40% | 20% | 0% |
| --- | --- | --- | --- | --- | --- | --- |
| **100%** | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` |
| **90%** | `rgb(204, 255, 204)` | `rgb(209, 250, 209)` | `rgb(214, 245, 214)` | `rgb(219, 240, 219)` | `rgb(224, 235, 224)` | `rgb(230, 230, 230)` |
| **80%** | `rgb(153, 255, 153)` | `rgb(163, 245, 163)` | `rgb(173, 235, 173)` | `rgb(184, 224, 184)` | `rgb(194, 214, 194)` | `rgb(204, 204, 204)` |
| **70%** | `rgb(102, 255, 102)` | `rgb(117, 240, 117)` | `rgb(133, 224, 133)` | `rgb(148, 209, 148)` | `rgb(163, 194, 163)` | `rgb(179, 179, 179)` |
| **60%** | `rgb(51, 255, 51)` | `rgb(71, 235, 71)` | `rgb(92, 214, 92)` | `rgb(112, 194, 112)` | `rgb(133, 173, 133)` | `rgb(153, 153, 153)` |
| **50%** | `rgb(0, 255, 0)` | `rgb(26, 230, 26)` | `rgb(51, 204, 51)` | `rgb(77, 179, 77)` | `rgb(102, 153, 102)` | `rgb(128, 128, 128)` |
| **40%** | `rgb(0, 204, 0)` | `rgb(20, 184, 20)` | `rgb(41, 163, 41)` | `rgb(61, 143, 61)` | `rgb(82, 122, 82)` | `rgb(102, 102, 102)` |
| **30%** | `rgb(0, 153, 0)` | `rgb(15, 138, 15)` | `rgb(31, 122, 31)` | `rgb(46, 107, 46)` | `rgb(61, 92, 61)` | `rgb(77, 77, 77)` |
| **20%** | `rgb(0, 102, 0)` | `rgb(10, 92, 10)` | `rgb(20, 82, 20)` | `rgb(31, 71, 31)` | `rgb(41, 61, 41)` | `rgb(51, 51, 51)` |
| **10%** | `rgb(0, 51, 0)` | `rgb(5, 46, 5)` | `rgb(10, 41, 10)` | `rgb(15, 36, 15)` | `rgb(20, 31, 20)` | `rgb(26, 26, 26)` |
| **0%** | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` |

**150° Green-Cyans**

Columns (X): saturation; rows (Y): lightness. Each entry is the exact source `background-color` value.

|  | 100% | 80% | 60% | 40% | 20% | 0% |
| --- | --- | --- | --- | --- | --- | --- |
| **100%** | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` |
| **90%** | `rgb(204, 255, 230)` | `rgb(209, 250, 230)` | `rgb(214, 245, 230)` | `rgb(219, 240, 230)` | `rgb(224, 235, 230)` | `rgb(230, 230, 230)` |
| **80%** | `rgb(153, 255, 204)` | `rgb(163, 245, 204)` | `rgb(173, 235, 204)` | `rgb(184, 224, 204)` | `rgb(194, 214, 204)` | `rgb(204, 204, 204)` |
| **70%** | `rgb(102, 255, 179)` | `rgb(117, 240, 179)` | `rgb(133, 224, 179)` | `rgb(148, 209, 179)` | `rgb(163, 194, 179)` | `rgb(179, 179, 179)` |
| **60%** | `rgb(51, 255, 153)` | `rgb(71, 235, 153)` | `rgb(92, 214, 153)` | `rgb(112, 194, 153)` | `rgb(133, 173, 153)` | `rgb(153, 153, 153)` |
| **50%** | `rgb(0, 255, 128)` | `rgb(26, 230, 128)` | `rgb(51, 204, 128)` | `rgb(77, 179, 128)` | `rgb(102, 153, 128)` | `rgb(128, 128, 128)` |
| **40%** | `rgb(0, 204, 102)` | `rgb(20, 184, 102)` | `rgb(41, 163, 102)` | `rgb(61, 143, 102)` | `rgb(82, 122, 102)` | `rgb(102, 102, 102)` |
| **30%** | `rgb(0, 153, 77)` | `rgb(15, 138, 77)` | `rgb(31, 122, 77)` | `rgb(46, 107, 77)` | `rgb(61, 92, 77)` | `rgb(77, 77, 77)` |
| **20%** | `rgb(0, 102, 51)` | `rgb(10, 92, 51)` | `rgb(20, 82, 51)` | `rgb(31, 71, 51)` | `rgb(41, 61, 51)` | `rgb(51, 51, 51)` |
| **10%** | `rgb(0, 51, 26)` | `rgb(5, 46, 26)` | `rgb(10, 41, 26)` | `rgb(15, 36, 26)` | `rgb(20, 31, 26)` | `rgb(26, 26, 26)` |
| **0%** | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` |

**180° Cyans**

Columns (X): saturation; rows (Y): lightness. Each entry is the exact source `background-color` value.

|  | 100% | 80% | 60% | 40% | 20% | 0% |
| --- | --- | --- | --- | --- | --- | --- |
| **100%** | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` |
| **90%** | `rgb(204, 255, 255)` | `rgb(209, 250, 250)` | `rgb(214, 245, 245)` | `rgb(219, 240, 240)` | `rgb(224, 235, 235)` | `rgb(230, 230, 230)` |
| **80%** | `rgb(153, 255, 255)` | `rgb(163, 245, 245)` | `rgb(173, 235, 235)` | `rgb(184, 224, 224)` | `rgb(194, 214, 214)` | `rgb(204, 204, 204)` |
| **70%** | `rgb(102, 255, 255)` | `rgb(117, 240, 240)` | `rgb(133, 224, 224)` | `rgb(148, 209, 209)` | `rgb(163, 194, 194)` | `rgb(179, 179, 179)` |
| **60%** | `rgb(51, 255, 255)` | `rgb(71, 235, 235)` | `rgb(92, 214, 214)` | `rgb(112, 194, 194)` | `rgb(133, 173, 173)` | `rgb(153, 153, 153)` |
| **50%** | `rgb(0, 255, 255)` | `rgb(26, 230, 230)` | `rgb(51, 204, 204)` | `rgb(77, 179, 179)` | `rgb(102, 153, 153)` | `rgb(128, 128, 128)` |
| **40%** | `rgb(0, 204, 204)` | `rgb(20, 184, 184)` | `rgb(41, 163, 163)` | `rgb(61, 143, 143)` | `rgb(82, 122, 122)` | `rgb(102, 102, 102)` |
| **30%** | `rgb(0, 153, 153)` | `rgb(15, 138, 138)` | `rgb(31, 122, 122)` | `rgb(46, 107, 107)` | `rgb(61, 92, 92)` | `rgb(77, 77, 77)` |
| **20%** | `rgb(0, 102, 102)` | `rgb(10, 92, 92)` | `rgb(20, 82, 82)` | `rgb(31, 71, 71)` | `rgb(41, 61, 61)` | `rgb(51, 51, 51)` |
| **10%** | `rgb(0, 51, 51)` | `rgb(5, 46, 46)` | `rgb(10, 41, 41)` | `rgb(15, 36, 36)` | `rgb(20, 31, 31)` | `rgb(26, 26, 26)` |
| **0%** | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` |

**210° Cyan-Blues**

Columns (X): saturation; rows (Y): lightness. Each entry is the exact source `background-color` value.

|  | 100% | 80% | 60% | 40% | 20% | 0% |
| --- | --- | --- | --- | --- | --- | --- |
| **100%** | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` |
| **90%** | `rgb(204, 230, 255)` | `rgb(209, 230, 250)` | `rgb(214, 230, 245)` | `rgb(219, 230, 240)` | `rgb(224, 230, 235)` | `rgb(230, 230, 230)` |
| **80%** | `rgb(153, 204, 255)` | `rgb(163, 204, 245)` | `rgb(173, 204, 235)` | `rgb(184, 204, 224)` | `rgb(194, 204, 214)` | `rgb(204, 204, 204)` |
| **70%** | `rgb(102, 179, 255)` | `rgb(117, 179, 240)` | `rgb(133, 179, 224)` | `rgb(148, 179, 209)` | `rgb(163, 179, 194)` | `rgb(179, 179, 179)` |
| **60%** | `rgb(51, 153, 255)` | `rgb(71, 153, 235)` | `rgb(92, 153, 214)` | `rgb(112, 153, 194)` | `rgb(133, 153, 173)` | `rgb(153, 153, 153)` |
| **50%** | `rgb(0, 128, 255)` | `rgb(26, 128, 230)` | `rgb(51, 128, 204)` | `rgb(77, 128, 179)` | `rgb(102, 128, 153)` | `rgb(128, 128, 128)` |
| **40%** | `rgb(0, 102, 204)` | `rgb(20, 102, 184)` | `rgb(41, 102, 163)` | `rgb(61, 102, 143)` | `rgb(82, 102, 122)` | `rgb(102, 102, 102)` |
| **30%** | `rgb(0, 77, 153)` | `rgb(15, 77, 138)` | `rgb(31, 77, 122)` | `rgb(46, 77, 107)` | `rgb(61, 77, 92)` | `rgb(77, 77, 77)` |
| **20%** | `rgb(0, 51, 102)` | `rgb(10, 51, 92)` | `rgb(20, 51, 82)` | `rgb(31, 51, 71)` | `rgb(41, 51, 61)` | `rgb(51, 51, 51)` |
| **10%** | `rgb(0, 26, 51)` | `rgb(5, 26, 46)` | `rgb(10, 26, 41)` | `rgb(15, 26, 36)` | `rgb(20, 26, 31)` | `rgb(26, 26, 26)` |
| **0%** | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` |

**240° blues**

Columns (X): saturation; rows (Y): lightness. Each entry is the exact source `background-color` value.

|  | 100% | 80% | 60% | 40% | 20% | 0% |
| --- | --- | --- | --- | --- | --- | --- |
| **100%** | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` |
| **90%** | `rgb(204, 204, 255)` | `rgb(209, 209, 250)` | `rgb(214, 214, 245)` | `rgb(219, 219, 240)` | `rgb(224, 224, 235)` | `rgb(230, 230, 230)` |
| **80%** | `rgb(153, 153, 255)` | `rgb(163, 163, 245)` | `rgb(173, 173, 235)` | `rgb(184, 184, 224)` | `rgb(194, 194, 214)` | `rgb(204, 204, 204)` |
| **70%** | `rgb(102, 102, 255)` | `rgb(117, 117, 240)` | `rgb(133, 133, 224)` | `rgb(148, 148, 209)` | `rgb(163, 163, 194)` | `rgb(179, 179, 179)` |
| **60%** | `rgb(51, 51, 255)` | `rgb(71, 71, 235)` | `rgb(92, 92, 214)` | `rgb(112, 112, 194)` | `rgb(133, 133, 173)` | `rgb(153, 153, 153)` |
| **50%** | `rgb(0, 0, 255)` | `rgb(26, 26, 230)` | `rgb(51, 51, 204)` | `rgb(77, 77, 179)` | `rgb(102, 102, 153)` | `rgb(128, 128, 128)` |
| **40%** | `rgb(0, 0, 204)` | `rgb(20, 20, 184)` | `rgb(41, 41, 163)` | `rgb(61, 61, 143)` | `rgb(82, 82, 122)` | `rgb(102, 102, 102)` |
| **30%** | `rgb(0, 0, 153)` | `rgb(15, 15, 138)` | `rgb(31, 31, 122)` | `rgb(46, 46, 107)` | `rgb(61, 61, 92)` | `rgb(77, 77, 77)` |
| **20%** | `rgb(0, 0, 102)` | `rgb(10, 10, 92)` | `rgb(20, 20, 82)` | `rgb(31, 31, 71)` | `rgb(41, 41, 61)` | `rgb(51, 51, 51)` |
| **10%** | `rgb(0, 0, 51)` | `rgb(5, 5, 46)` | `rgb(10, 10, 41)` | `rgb(15, 15, 36)` | `rgb(20, 20, 31)` | `rgb(26, 26, 26)` |
| **0%** | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` |

**270° Blue-Magentas**

Columns (X): saturation; rows (Y): lightness. Each entry is the exact source `background-color` value.

|  | 100% | 80% | 60% | 40% | 20% | 0% |
| --- | --- | --- | --- | --- | --- | --- |
| **100%** | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` |
| **90%** | `rgb(230, 204, 255)` | `rgb(230, 209, 250)` | `rgb(230, 214, 245)` | `rgb(230, 219, 240)` | `rgb(230, 224, 235)` | `rgb(230, 230, 230)` |
| **80%** | `rgb(204, 153, 255)` | `rgb(204, 163, 245)` | `rgb(204, 173, 235)` | `rgb(204, 184, 224)` | `rgb(204, 194, 214)` | `rgb(204, 204, 204)` |
| **70%** | `rgb(179, 102, 255)` | `rgb(179, 117, 240)` | `rgb(179, 133, 224)` | `rgb(179, 148, 209)` | `rgb(179, 163, 194)` | `rgb(179, 179, 179)` |
| **60%** | `rgb(153, 51, 255)` | `rgb(153, 71, 235)` | `rgb(153, 92, 214)` | `rgb(153, 112, 194)` | `rgb(153, 133, 173)` | `rgb(153, 153, 153)` |
| **50%** | `rgb(128, 0, 255)` | `rgb(128, 26, 230)` | `rgb(128, 51, 204)` | `rgb(128, 77, 179)` | `rgb(128, 102, 153)` | `rgb(128, 128, 128)` |
| **40%** | `rgb(102, 0, 204)` | `rgb(102, 20, 184)` | `rgb(102, 41, 163)` | `rgb(102, 61, 143)` | `rgb(102, 82, 122)` | `rgb(102, 102, 102)` |
| **30%** | `rgb(77, 0, 153)` | `rgb(77, 15, 138)` | `rgb(77, 31, 122)` | `rgb(77, 46, 107)` | `rgb(77, 61, 92)` | `rgb(77, 77, 77)` |
| **20%** | `rgb(51, 0, 102)` | `rgb(51, 10, 92)` | `rgb(51, 20, 82)` | `rgb(51, 31, 71)` | `rgb(51, 41, 61)` | `rgb(51, 51, 51)` |
| **10%** | `rgb(26, 0, 51)` | `rgb(26, 5, 46)` | `rgb(26, 10, 41)` | `rgb(26, 15, 36)` | `rgb(26, 20, 31)` | `rgb(26, 26, 26)` |
| **0%** | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` |

**300° Magentas**

Columns (X): saturation; rows (Y): lightness. Each entry is the exact source `background-color` value.

|  | 100% | 80% | 60% | 40% | 20% | 0% |
| --- | --- | --- | --- | --- | --- | --- |
| **100%** | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` |
| **90%** | `rgb(255, 204, 255)` | `rgb(250, 209, 250)` | `rgb(245, 214, 245)` | `rgb(240, 219, 240)` | `rgb(235, 224, 235)` | `rgb(230, 230, 230)` |
| **80%** | `rgb(255, 153, 255)` | `rgb(245, 163, 245)` | `rgb(235, 173, 235)` | `rgb(224, 184, 224)` | `rgb(214, 194, 214)` | `rgb(204, 204, 204)` |
| **70%** | `rgb(255, 102, 255)` | `rgb(240, 117, 240)` | `rgb(224, 133, 224)` | `rgb(209, 148, 209)` | `rgb(194, 163, 194)` | `rgb(179, 179, 179)` |
| **60%** | `rgb(255, 51, 255)` | `rgb(235, 71, 235)` | `rgb(214, 92, 214)` | `rgb(194, 112, 194)` | `rgb(173, 133, 173)` | `rgb(153, 153, 153)` |
| **50%** | `rgb(255, 0, 255)` | `rgb(230, 26, 230)` | `rgb(204, 51, 204)` | `rgb(179, 77, 179)` | `rgb(153, 102, 153)` | `rgb(128, 128, 128)` |
| **40%** | `rgb(204, 0, 204)` | `rgb(184, 20, 184)` | `rgb(163, 41, 163)` | `rgb(143, 61, 143)` | `rgb(122, 82, 122)` | `rgb(102, 102, 102)` |
| **30%** | `rgb(153, 0, 153)` | `rgb(138, 15, 138)` | `rgb(122, 31, 122)` | `rgb(107, 46, 107)` | `rgb(92, 61, 92)` | `rgb(77, 77, 77)` |
| **20%** | `rgb(102, 0, 102)` | `rgb(92, 10, 92)` | `rgb(82, 20, 82)` | `rgb(71, 31, 71)` | `rgb(61, 41, 61)` | `rgb(51, 51, 51)` |
| **10%** | `rgb(51, 0, 51)` | `rgb(46, 5, 46)` | `rgb(41, 10, 41)` | `rgb(36, 15, 36)` | `rgb(31, 20, 31)` | `rgb(26, 26, 26)` |
| **0%** | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` |

**330° Magenta-Reds**

Columns (X): saturation; rows (Y): lightness. Each entry is the exact source `background-color` value.

|  | 100% | 80% | 60% | 40% | 20% | 0% |
| --- | --- | --- | --- | --- | --- | --- |
| **100%** | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` | `rgb(255, 255, 255)` |
| **90%** | `rgb(255, 204, 230)` | `rgb(250, 209, 230)` | `rgb(245, 214, 230)` | `rgb(240, 219, 230)` | `rgb(235, 224, 230)` | `rgb(230, 230, 230)` |
| **80%** | `rgb(255, 153, 204)` | `rgb(245, 163, 204)` | `rgb(235, 173, 204)` | `rgb(224, 184, 204)` | `rgb(214, 194, 204)` | `rgb(204, 204, 204)` |
| **70%** | `rgb(255, 102, 179)` | `rgb(240, 117, 179)` | `rgb(224, 133, 179)` | `rgb(209, 148, 179)` | `rgb(194, 163, 179)` | `rgb(179, 179, 179)` |
| **60%** | `rgb(255, 51, 153)` | `rgb(235, 71, 153)` | `rgb(214, 92, 153)` | `rgb(194, 112, 153)` | `rgb(173, 133, 153)` | `rgb(153, 153, 153)` |
| **50%** | `rgb(255, 0, 128)` | `rgb(230, 26, 128)` | `rgb(204, 51, 128)` | `rgb(179, 77, 128)` | `rgb(153, 102, 128)` | `rgb(128, 128, 128)` |
| **40%** | `rgb(204, 0, 102)` | `rgb(184, 20, 102)` | `rgb(163, 41, 102)` | `rgb(143, 61, 102)` | `rgb(122, 82, 102)` | `rgb(102, 102, 102)` |
| **30%** | `rgb(153, 0, 77)` | `rgb(138, 15, 77)` | `rgb(122, 31, 77)` | `rgb(107, 46, 77)` | `rgb(92, 61, 77)` | `rgb(77, 77, 77)` |
| **20%** | `rgb(102, 0, 51)` | `rgb(92, 10, 51)` | `rgb(82, 20, 51)` | `rgb(71, 31, 51)` | `rgb(61, 41, 51)` | `rgb(51, 51, 51)` |
| **10%** | `rgb(51, 0, 26)` | `rgb(46, 5, 26)` | `rgb(41, 10, 26)` | `rgb(36, 15, 26)` | `rgb(31, 20, 26)` | `rgb(26, 26, 26)` |
| **0%** | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` | `rgb(0, 0, 0)` |

<a id="ref-for-funcdef-hwb⑤"></a>

## <a id="the-hwb-notation"></a>8.  HWB Colors: [hwb()](#funcdef-hwb) function

<a id="ref-for-valdef-hsl-hsl①"></a>

<a id="valdef-hwb-hwb"></a>HWB (short for Hue-Whiteness-Blackness) [\[HWB\]](#biblio-hwb) is another method of specifying sRGB colors, similar to [HSL](#valdef-hsl-hsl)', but often even easier for humans to work with. It describes colors with a starting hue, then a degree of whiteness and blackness to mix into that base hue.

Many color-pickers are based on the HWB color system, due to its intuitiveness.

HWB colors resolve to sRGB.

<a id="fig-chrome-hwb-picker"></a> ![](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/color-picker.png)

This is a screenshot of Chrome’s color picker, shown when a user activates an `<input type="color">`. The outer wheel is used to select the hue, then the relative amounts of white and black are selected by clicking on the inner triangle.

<a id="ref-for-funcdef-hwb⑥"></a>

The syntax of the [hwb()](#funcdef-hwb) function is:

<a id="funcdef-hwb"></a>

<a id="ref-for-typedef-hue①①"></a>

<a id="ref-for-comb-one④①"></a>

<a id="ref-for-percentage-value①⑦"></a>

<a id="ref-for-comb-one④②"></a>

<a id="ref-for-number-value①⑤"></a>

<a id="ref-for-comb-one④③"></a>

<a id="ref-for-percentage-value①⑧"></a>

<a id="ref-for-comb-one④④"></a>

<a id="ref-for-number-value①⑥"></a>

<a id="ref-for-comb-one④⑤"></a>

<a id="ref-for-typedef-color-alpha-value①③"></a>

<a id="ref-for-comb-one④⑥"></a>

<a id="ref-for-mult-opt①⓪"></a>

```text
hwb() = hwb(
  [<hue> | none]
  [<percentage> | <number> | none]
  [<percentage> | <number> | none]
  [ / [<alpha-value> | none] ]? )
```
<a id="prr-hwb"></a>



| Field               | Definition                          |
|---------------------|-------------------------------------|
| <strong>Percentages&#xA;      </strong> | Allowed for W and B                 |
| <strong>Percent reference range&#xA0;&#xA;      </strong> | for W and B: 0% = 0.0, 100% = 100.0 |
| <strong>Powerless hue ε&#xA;      </strong> | W + B \>= 99.999                    |



<a id="ref-for-funcdef-hsl①③"></a>

The first argument specifies the hue, and is defined identically to [hsl()](#funcdef-hsl); this means it [suffers the same disadvantages](#disadvantage-hsl) such as hue uniformity.

The second argument specifies the amount of white to mix in, as a percentage from 0% (no whiteness) to 100% (full whiteness). Similarly, the third argument specifies the amount of black to mix in, also from 0% (no blackness) to 100% (full blackness).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-hwb-simple"></a> For example,  hwb(150 20% 10%) is the same color as  hsl(150 77.78% 55%) and  rgb(20% 90% 55%).

Values outside of these ranges are not invalid; hue angles outside the range \[0,360) will be normalized to that range and values of white and black which sum to 100% or greater will produce achromatic colors as described below.

The resulting color can be thought of conceptually as a mixture of paint in the chosen hue, white paint, and black paint, with the relative amounts of each determined by the percentages.

<a id="hwb-normalization"></a>If the sum white+black is greater than or equal to 100%, it defines an achromatic color, i.e. a shade of gray; when converted to sRGB the R, G and B values are identical and have the value white / (white + black).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-hwb-achromatic"></a> For example, in the color  hwb(45 40% 80%) white and black adds to 120, so this is an achromatic color whose R, G and B components are 40 / 40 + 80 = 0.33  rgb(33.33% 33.33% 33.33%).

<a id="ref-for-powerless-color-component⑨"></a>

Achromatic HWB colors no longer contain any hint of the chosen hue. In this case, the hue component is [powerless](#powerless-color-component).

<a id="ref-for-funcdef-rgb①①"></a>

The fourth argument specifies the alpha component of the color. It’s interpreted identically to the fourth argument of the [rgb()](#funcdef-rgb) function. If omitted, it defaults to 100%.

<a id="ref-for-valdef-hwb-hwb"></a>

<a id="ref-for-funcdef-hwb⑦"></a>

<a id="ref-for-legacy-color-syntax⑤"></a>

There is no Web compatibility issue with [hwb](#valdef-hwb-hwb), which is new in this level of the specification, and so [hwb()](#funcdef-hwb) does <em>not</em> support a [legacy color syntax](#legacy-color-syntax) that separates all of its arguments with commas. Using commas inside <a id="ref-for-funcdef-hwb⑧"></a>hwb() is an error.

Tests

- [hwb-001.html](https://wpt.fyi/results/css/css-color/hwb-001.html) [(live test)](http://wpt.live/css/css-color/hwb-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hwb-001.html)
- [hwb-002.html](https://wpt.fyi/results/css/css-color/hwb-002.html) [(live test)](http://wpt.live/css/css-color/hwb-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hwb-002.html)
- [hwb-003.html](https://wpt.fyi/results/css/css-color/hwb-003.html) [(live test)](http://wpt.live/css/css-color/hwb-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hwb-003.html)
- [hwb-004.html](https://wpt.fyi/results/css/css-color/hwb-004.html) [(live test)](http://wpt.live/css/css-color/hwb-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hwb-004.html)
- [hwb-005.html](https://wpt.fyi/results/css/css-color/hwb-005.html) [(live test)](http://wpt.live/css/css-color/hwb-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/hwb-005.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)
- [color-computed-hwb.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-hwb.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-hwb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-hwb.html)
- [color-invalid-hwb.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-hwb.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-hwb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-hwb.html)
- [color-valid-hwb.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-hwb.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-hwb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-hwb.html)

### <a id="hwb-to-rgb"></a>8.1.  Converting HWB Colors to sRGB

Converting an HWB color to sRGB is straightforward, and related to how one converts HSL to RGB. The following Javascript implementation of the algorithm first normalizes the white and black components, so their sum is no larger than 100%.

```javascript
/**
 * @param {number} hue -  Hue as degrees 0..360
 * @param {number} white -  Whiteness in reference range [0,100]
 * @param {number} black -  Blackness in reference range [0,100]
 * @return {number[]} Array of RGB components 0..1
 */
function hwbToRgb(hue, white, black) {
    white /= 100;
    black /= 100;
    if (white + black >= 1) {
        let gray = white / (white + black);
        return [gray, gray, gray];
    }
    let rgb = hslToRgb(hue, 100, 50);
    for (let i = 0; i < 3; i++) {
        rgb[i] *= (1 - white - black);
        rgb[i] += white;
    }
    return rgb;
}
```
### <a id="rgb-to-hwb"></a>8.2.  Converting sRGB Colors to HWB

Conversion in the reverse direction proceeds similarly.

```javascript
/**
 * @param {number} red - Red component 0..1
 * @param {number} green - Green component 0..1
 * @param {number} blue - Blue component 0..1
 * @return {number} Hue as degrees 0..360
 */
function rgbToHue(red, green, blue) {
    // Similar to rgbToHsl, except that saturation and lightness are not calculated, and
    // potential negative saturation is ignored.
    let max = Math.max(red, green, blue);
    let min = Math.min(red, green, blue);
    let hue = NaN;
    let d = max - min;

    if (d !== 0) {
        switch (max) {
            case red:   hue = (green - blue) / d + (green < blue ? 6 : 0); break;
            case green: hue = (blue - red) / d + 2; break;
            case blue:  hue = (red - green) / d + 4;
        }

        hue *= 60;
    }

    if (hue >= 360) {
        hue -= 360;
    }

    return hue;
}

/**
 * @param {number} red - Red component 0..1
 * @param {number} green - Green component 0..1
 * @param {number} blue - Blue component 0..1
 * @return {number[]} Array of HWB values: Hue as degrees 0..360, Whiteness and Blackness in reference range [0,100]
 */
function rgbToHwb(red, green, blue) {
    let epsilon = 1 / 100000;  // account for multiply by 100
    var hue = rgbToHue(red, green, blue);
    var white = Math.min(red, green, blue);
    var black = 1 - Math.max(red, green, blue);
    if (white + black >= 1 - epsilon) {
        hue = NaN;
    }
    return([hue, white*100, black*100]);
}
```
### <a id="hwb-examples"></a>8.3.  Examples of HWB Colors

<em>This section is not normative.</em>

Tests

This section is not normative, it does not need tests.

------------------------------------------------------------------------

**0° Reds**

Columns (B): blackness; rows (W): whiteness. Each entry is the exact source `background-color` value.

| `W\B` | 0% | 20% | 40% | 60% | 80% | 100% |
| --- | --- | --- | --- | --- | --- | --- |
| **0%** | `#ff0000` | `#cc0000` | `#990000` | `#660000` | `#330000` | `#000000` |
| **20%** | `#ff3333` | `#cc3333` | `#993333` | `#663333` | `#333333` | `#2a2b2b` |
| **40%** | `#ff6666` | `#cc6666` | `#996666` | `#666666` | `#555555` | `#494949` |
| **60%** | `#ff9999` | `#cc9999` | `#999999` | `#808080` | `#6d6d6d` | `#606060` |
| **80%** | `#ffcccc` | `#cccccc` | `#aaaaaa` | `#929292` | `#808080` | `#717171` |
| **100%** | `#ffffff` | `#d4d5d5` | `#b6b6b6` | `#9f9f9f` | `#8e8e8e` | `#808080` |

**30° Red-Yellows (Oranges)**

Columns (B): blackness; rows (W): whiteness. Each entry is the exact source `background-color` value.

| `W\B` | 0% | 20% | 40% | 60% | 80% | 100% |
| --- | --- | --- | --- | --- | --- | --- |
| **0%** | `#ff8000` | `#cc6600` | `#994d00` | `#663300` | `#331900` | `#000000` |
| **20%** | `#ff9933` | `#cc8033` | `#996633` | `#664d33` | `#333333` | `#2a2a2b` |
| **40%** | `#ffb366` | `#cc9966` | `#998066` | `#666666` | `#555555` | `#494949` |
| **60%** | `#ffcc99` | `#ccb399` | `#999999` | `#808080` | `#6d6d6d` | `#606060` |
| **80%** | `#ffe6cc` | `#cccccc` | `#aaaaaa` | `#929292` | `#808080` | `#717171` |
| **100%** | `#ffffff` | `#d4d5d5` | `#b6b6b6` | `#9f9f9f` | `#8e8e8e` | `#808080` |

**60° Yellows**

Columns (B): blackness; rows (W): whiteness. Each entry is the exact source `background-color` value.

| `W\B` | 0% | 20% | 40% | 60% | 80% | 100% |
| --- | --- | --- | --- | --- | --- | --- |
| **0%** | `#ffff00` | `#cccc00` | `#999900` | `#666600` | `#333300` | `#000000` |
| **20%** | `#ffff33` | `#cccc33` | `#999933` | `#666633` | `#333333` | `#2a2a2b` |
| **40%** | `#ffff66` | `#cccc66` | `#999966` | `#666666` | `#555555` | `#494949` |
| **60%** | `#ffff99` | `#cccc99` | `#999999` | `#808080` | `#6d6d6d` | `#606060` |
| **80%** | `#ffffcc` | `#cccccc` | `#aaaaaa` | `#929292` | `#808080` | `#717171` |
| **100%** | `#ffffff` | `#d5d4d5` | `#b6b6b6` | `#9f9f9f` | `#8e8e8e` | `#808080` |

**90° Yellow-Greens**

Columns (B): blackness; rows (W): whiteness. Each entry is the exact source `background-color` value.

| `W\B` | 0% | 20% | 40% | 60% | 80% | 100% |
| --- | --- | --- | --- | --- | --- | --- |
| **0%** | `#80ff00` | `#66cc00` | `#4d9900` | `#336600` | `#1a3300` | `#000000` |
| **20%** | `#99ff33` | `#80cc33` | `#669933` | `#4d6633` | `#333333` | `#2a2a2b` |
| **40%** | `#b3ff66` | `#99cc66` | `#809966` | `#666666` | `#555555` | `#494949` |
| **60%** | `#ccff99` | `#b3cc99` | `#999999` | `#808080` | `#6d6d6d` | `#606060` |
| **80%** | `#e6ffcc` | `#cccccc` | `#aaaaaa` | `#929292` | `#808080` | `#717171` |
| **100%** | `#ffffff` | `#d5d4d5` | `#b6b6b6` | `#9f9f9f` | `#8e8e8e` | `#808080` |

**120° Greens**

Columns (B): blackness; rows (W): whiteness. Each entry is the exact source `background-color` value.

| `W\B` | 0% | 20% | 40% | 60% | 80% | 100% |
| --- | --- | --- | --- | --- | --- | --- |
| **0%** | `#00ff00` | `#00cc00` | `#009900` | `#006600` | `#003300` | `#000000` |
| **20%** | `#33ff33` | `#33cc33` | `#339933` | `#336633` | `#333333` | `#2b2a2b` |
| **40%** | `#66ff66` | `#66cc66` | `#669966` | `#666666` | `#555555` | `#494949` |
| **60%** | `#99ff99` | `#99cc99` | `#999999` | `#808080` | `#6d6d6d` | `#606060` |
| **80%** | `#ccffcc` | `#cccccc` | `#aaaaaa` | `#929292` | `#808080` | `#717171` |
| **100%** | `#ffffff` | `#d5d4d5` | `#b6b6b6` | `#9f9f9f` | `#8e8e8e` | `#808080` |

**150° Green-Cyans**

Columns (B): blackness; rows (W): whiteness. Each entry is the exact source `background-color` value.

| `W\B` | 0% | 20% | 40% | 60% | 80% | 100% |
| --- | --- | --- | --- | --- | --- | --- |
| **0%** | `#00ff7f` | `#00cc66` | `#00994c` | `#006633` | `#003319` | `#000000` |
| **20%** | `#33ff99` | `#33cc7f` | `#339966` | `#33664c` | `#333333` | `#2b2a2a` |
| **40%** | `#66ffb2` | `#66cc99` | `#66997f` | `#666666` | `#555555` | `#494949` |
| **60%** | `#99ffcc` | `#99ccb3` | `#999999` | `#808080` | `#6d6d6d` | `#606060` |
| **80%** | `#ccffe5` | `#cccccc` | `#aaaaaa` | `#929292` | `#808080` | `#717171` |
| **100%** | `#ffffff` | `#d5d4d5` | `#b6b6b6` | `#9f9f9f` | `#8e8e8e` | `#808080` |

**180° Cyans**

Columns (B): blackness; rows (W): whiteness. Each entry is the exact source `background-color` value.

| `W\B` | 0% | 20% | 40% | 60% | 80% | 100% |
| --- | --- | --- | --- | --- | --- | --- |
| **0%** | `#00ffff` | `#00cccc` | `#009999` | `#006666` | `#003333` | `#000000` |
| **20%** | `#33ffff` | `#33cccc` | `#339999` | `#336666` | `#333333` | `#2b2a2a` |
| **40%** | `#66ffff` | `#66cccc` | `#669999` | `#666666` | `#555555` | `#494949` |
| **60%** | `#99ffff` | `#99cccc` | `#999999` | `#808080` | `#6d6d6d` | `#606060` |
| **80%** | `#ccffff` | `#cccccc` | `#aaaaaa` | `#929292` | `#808080` | `#717171` |
| **100%** | `#ffffff` | `#d5d5d5` | `#b6b6b6` | `#9f9f9f` | `#8e8e8e` | `#808080` |

**210° Cyan-Blues**

Columns (B): blackness; rows (W): whiteness. Each entry is the exact source `background-color` value.

| `W\B` | 0% | 20% | 40% | 60% | 80% | 100% |
| --- | --- | --- | --- | --- | --- | --- |
| **0%** | `#007fff` | `#0066cc` | `#004c99` | `#003366` | `#001933` | `#000000` |
| **20%** | `#3399ff` | `#337fcc` | `#336699` | `#334c66` | `#333333` | `#2b2a2a` |
| **40%** | `#66b2ff` | `#6699cc` | `#667f99` | `#666666` | `#555555` | `#494949` |
| **60%** | `#99ccff` | `#99b3cc` | `#999999` | `#808080` | `#6d6d6d` | `#606060` |
| **80%** | `#cce5ff` | `#cccccc` | `#aaaaaa` | `#929292` | `#808080` | `#717171` |
| **100%** | `#ffffff` | `#d5d5d4` | `#b6b6b6` | `#9f9f9f` | `#8e8e8e` | `#808080` |

**240° Blues**

Columns (B): blackness; rows (W): whiteness. Each entry is the exact source `background-color` value.

| `W\B` | 0% | 20% | 40% | 60% | 80% | 100% |
| --- | --- | --- | --- | --- | --- | --- |
| **0%** | `#0000ff` | `#0000cc` | `#000099` | `#000066` | `#000033` | `#000000` |
| **20%** | `#3333ff` | `#3333cc` | `#333399` | `#333366` | `#333333` | `#2b2b2a` |
| **40%** | `#6666ff` | `#6666cc` | `#666699` | `#666666` | `#555555` | `#494949` |
| **60%** | `#9999ff` | `#9999cc` | `#999999` | `#808080` | `#6d6d6d` | `#606060` |
| **80%** | `#ccccff` | `#cccccc` | `#aaaaaa` | `#929292` | `#808080` | `#717171` |
| **100%** | `#ffffff` | `#d5d5d4` | `#b6b6b6` | `#9f9f9f` | `#8e8e8e` | `#808080` |

**270° Blue-Magentas**

Columns (B): blackness; rows (W): whiteness. Each entry is the exact source `background-color` value.

| `W\B` | 0% | 20% | 40% | 60% | 80% | 100% |
| --- | --- | --- | --- | --- | --- | --- |
| **0%** | `#7f00ff` | `#6600cc` | `#4c0099` | `#330066` | `#190033` | `#000000` |
| **20%** | `#9933ff` | `#7f33cc` | `#663399` | `#4c3366` | `#333333` | `#2a2b2a` |
| **40%** | `#b266ff` | `#9966cc` | `#7f6699` | `#666666` | `#555555` | `#494949` |
| **60%** | `#cc99ff` | `#b399cc` | `#999999` | `#808080` | `#6d6d6d` | `#606060` |
| **80%** | `#e5ccff` | `#cccccc` | `#aaaaaa` | `#929292` | `#808080` | `#717171` |
| **100%** | `#ffffff` | `#d5d5d4` | `#b6b6b6` | `#9f9f9f` | `#8e8e8e` | `#808080` |

**300° Magentas**

Columns (B): blackness; rows (W): whiteness. Each entry is the exact source `background-color` value.

| `W\B` | 0% | 20% | 40% | 60% | 80% | 100% |
| --- | --- | --- | --- | --- | --- | --- |
| **0%** | `#ff00ff` | `#cc00cc` | `#990099` | `#660066` | `#330033` | `#000000` |
| **20%** | `#ff33ff` | `#cc33cc` | `#993399` | `#663366` | `#333333` | `#2a2b2a` |
| **40%** | `#ff66ff` | `#cc66cc` | `#996699` | `#666666` | `#555555` | `#494949` |
| **60%** | `#ff99ff` | `#cc99cc` | `#999999` | `#808080` | `#6d6d6d` | `#606060` |
| **80%** | `#ffccff` | `#cccccc` | `#aaaaaa` | `#929292` | `#808080` | `#717171` |
| **100%** | `#ffffff` | `#d4d5d5` | `#b6b6b6` | `#9f9f9f` | `#8e8e8e` | `#808080` |

**330° Magenta-Reds**

Columns (B): blackness; rows (W): whiteness. Each entry is the exact source `background-color` value.

| `W\B` | 0% | 20% | 40% | 60% | 80% | 100% |
| --- | --- | --- | --- | --- | --- | --- |
| **0%** | `#ff0080` | `#cc0066` | `#99004d` | `#660033` | `#33001a` | `#000000` |
| **20%** | `#ff3399` | `#cc3380` | `#993366` | `#66334d` | `#333333` | `#2a2b2a` |
| **40%** | `#ff66b3` | `#cc6699` | `#996680` | `#666666` | `#555555` | `#494949` |
| **60%** | `#ff99cc` | `#cc99b3` | `#999999` | `#808080` | `#6d6d6d` | `#606060` |
| **80%** | `#ffcce6` | `#cccccc` | `#aaaaaa` | `#929292` | `#808080` | `#717171` |
| **100%** | `#ffffff` | `#d4d5d5` | `#b6b6b6` | `#9f9f9f` | `#8e8e8e` | `#808080` |

## <a id="lab-colors"></a>9.  Device-independent Colors: CIE Lab and LCH, Oklab and OkLCh

### <a id="cie-lab"></a>9.1. CIE Lab and LCH

<em>This section is not normative.</em>

Tests

This section is not normative, it does not need tests.

------------------------------------------------------------------------

Physical measurements of a color are typically expressed in the CIE L\*a\*b\* [\[CIELAB\]](#biblio-cielab) color space, created in 1976 by the CIE and commonly referred to simply as Lab. Color conversions from one device to another may also use Lab as an intermediate step. Derived from human vision experiments, Lab represents the entire range of color that humans can see.

Lab is a rectangular coordinate system with a central Lightness (L) axis. This value is usually written as a unitless number; for compatibility with the rest of CSS, it may also be written as a percentage. 100% means an L value of 100, not 1.0. L=0% or 0 is deep black (no light at all) while L=100% or 100 is a diffuse white.

Usefully, L=50% or 50 is mid gray, by design, and equal increments in L are evenly spaced visually: the Lab color space is intended to be <em>perceptually uniform</em>.

<a id="lightness-vs-luminance"></a> ![](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/L-axis.svg) ![](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/Luminance.svg)

<a id="ref-for-luminance"></a>

This figure shows, to the left, the Lightness axis of the CIE Lab color space. Twenty-one neutral swatches are shown (L=0%, L=5%, to L=100%). The steps are equally spaced, visually. To the right, the same number of steps in [luminance](#luminance) are equally spaced in light energy but <strong>not</strong> equally spaced visually.

The a and b axes convey hue; positive values along the a axis are a purplish red while negative values are the complementary color, a green. Similarly, positive values along the b axis are yellow and negative are the complementary blue/violet. Desaturated colors have small values of a and b and are close to the L axis; saturated colors lie far from the L axis.

<a id="ref-for-d50"></a>

The illuminant is [D50](#d50) white, a standardized daylight spectrum with a color temperature of 5000K, as reflected by a perfect diffuse reflector; it approximates the color of sunlight on a sunny day. D50 is also the whitepoint used for the profile connection space in ICC color interconversion, the whitepoint used in image editors which offer Lab editing, and the value used by physical measurement devices such as spectrophotometers and spectroradiometers, when they report measured colors in Lab.

Conversion from colors specified using other white points is called a <a id="chromatic-adaptation-transform"></a>chromatic adaptation transform, which models the changes in the human visual system as we adapt to a new lighting condition. The linear Bradford algorithm [\[ICC\]](#biblio-icc) (a simplification of the original Bradford algorithm [\[Bradford-CAT\]](#biblio-bradford-cat)) is the industry standard chromatic adaptation transform, and is easy to calculate as it is a simple matrix multiplication.

CIE LCH has the same L axis as Lab, but uses polar coordinates C (chroma) and H (hue), making it a polar, cylindrical coordinate system. C is the geometric distance from the L axis and H is the angle from the positive a axis, towards the positive b axis.

<a id="fig-cie-lch-hues"></a> ![](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/CH-plane-wheel.svg)

This figure shows the L=50 plane of the CIE Lab color space. 20 degree increments in CIE LCH are displayed as circles at three levels of Chroma: 20, 40 and 60. All the 20 Chroma colors fit inside sRGB gamut, some of 40 and 60 Chroma are outside. These out of gamut colors are visualized as grey, with a red warning outer stroke.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The L axis in Lab and LCH is not to be confused with the L axis in HSL. For example, in HSL, the sRGB colors blue (#00F) and yellow (#FF0) have the same value of L (50%) even though visually, blue is much darker. This is much clearer in Lab: sRGB blue is lab(29.567% 68.298 -112.0294) while sRGB yellow is lab(97.607% -15.753 93.388). In Lab and LCH, if two colors have the same measured L value, they have identical visual lightness. HSL and related polar RGB models were developed in an attempt to give similar usability benefits for RGB that LCH gave to Lab, but are significantly less accurate.

Although the use of CIE Lab and LCH is widespread, it is known to have some problems. In particular:

Hue linearity  
In the blue region (LCH Hue between 270° and 330°), visual hue departs from what LCH predicts. Plotting a set of blues of the same hue and differing Chroma, which should lie on a straight line from the neutral axis, instead form a curve. Put another way, as a saturated blue has it’s Chroma progressively reduced, it becomes noticeably purple.

Hue uniformity  
While hues in LCH are in general evenly spaced, (and far better than HSL or HWB), uniformity is not perfect.

Over-prediction of high Chroma differences  
For high Chroma colors, changes in Chroma are less noticeable than for more neutral colors.

These deficiencies affect, for example, creation of evenly spaced gradients, gamut mapping from one color space to a smaller one, and computation of the visual difference between two colors.

To compensate for this, formulae to predict the visual difference between two colors (delta E) have been made more accurate over time (but also, much more complex to compute). The current industry standard formula, delta E 2000, works well to mitigate some of the Lab and LCH problems. A sample implementation is given in [§ 20.1 ΔE2000](#color-difference-2000).

This does not help with hue curvature, however.

### <a id="ok-lab"></a>9.2. Oklab and OkLCh

<em>This section is not normative.</em>

Tests

This section is not normative, it does not need tests.

------------------------------------------------------------------------

Recently, Oklab, an improved Lab-like space has been developed [\[Oklab\]](#biblio-oklab). The corresponding polar form is called OkLCh. It was produced by numerical optimization of a large dataset of visually similar colors, and has improved hue linearity, hue uniformity, and chroma uniformity compared to CIE LCH.

Like CIE Lab, there is a central lightness L axis which is usually written as a unitless number in the range \[0,1\]; for compatibility with the rest of CSS, it may be written as a percentage. 100% means an L value of 1.0. L=0% or 0.0 is deep black (no light at all) while L=100% or 1.0 is a diffuse white.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Unlike CIE Lab, which assumes adaptation to the diffuse white, Oklab assumes adaptation to the color being defined, which is intended to make it scale invariant.

As with CIE Lab, the a and b axes convey hue; positive values along the a axis are a purplish red while negative values are the complementary color, a green. Similarly, positive values along the b axis are yellow and negative are the complementary blue/violet.

<a id="ref-for-d65①"></a>

The illuminant is [D65](#d65), the same white point as most RGB color spaces.

OkLCh has the same L axis as Oklab, but uses polar coordinates C (chroma) and H (hue).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Unlike CIE LCH, where Chroma can reach values of 200 or more, OkLCh Chroma ranges to 0.5 or so. The hue angles between CIE LCH and OkLCh are broadly similar, but not identical.

<a id="CIELCH-blue-hueshift"></a> ![diagram showing purpling in CIE LCH](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/CIELCH-blue-slice.png)

A constant CIE LCH hue slice, showing the sRGB gamut around primary blue. A noticeable purpling is immediately evident.

<a id="OkLCh-blue-no-hueshift"></a> ![diagram showing hue constancy in OkLCh](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/OKLCH-blue-slice.png)

A constant OkLCh hue slice, showing the sRGB gamut around primary blue. The visual hue remains constant.

Because Oklab is more perceptually uniform than CIE Lab, the color difference is a straightforward distance in 3D space (root sum of squares). Although trivial, a sample implementation is give in [§ 20.2 ΔEOK](#color-difference-OK).

<a id="ref-for-funcdef-lab②"></a>

<a id="ref-for-funcdef-lch③"></a>

### <a id="specifying-lab-lch"></a>9.3.  Specifying Lab and LCH: the [lab()](#funcdef-lab) and [lch()](#funcdef-lch) functional notations

CSS allows colors to be directly expressed in Lab and LCH.

<a id="funcdef-lab"></a>

<a id="ref-for-percentage-value①⑨"></a>

<a id="ref-for-comb-one④⑦"></a>

<a id="ref-for-number-value①⑦"></a>

<a id="ref-for-comb-one④⑧"></a>

<a id="ref-for-percentage-value②⓪"></a>

<a id="ref-for-comb-one④⑨"></a>

<a id="ref-for-number-value①⑧"></a>

<a id="ref-for-comb-one⑤⓪"></a>

<a id="ref-for-percentage-value②①"></a>

<a id="ref-for-comb-one⑤①"></a>

<a id="ref-for-number-value①⑨"></a>

<a id="ref-for-comb-one⑤②"></a>

<a id="ref-for-typedef-color-alpha-value①④"></a>

<a id="ref-for-comb-one⑤③"></a>

<a id="ref-for-mult-opt①①"></a>

```text
lab() = lab( [<percentage> | <number> | none]
      [ <percentage> | <number> | none]
      [ <percentage> | <number> | none]
      [ / [<alpha-value> | none] ]? )
```
<a id="prr-lab"></a>



| Field               | Definition                                                                           |
|---------------------|--------------------------------------------------------------------------------------|
| <strong>Percentages&#xA;      </strong> | Allowed for L, a and b                                                               |
| <strong>Percent reference range&#xA0;&#xA;      </strong> | for L: 0% = 0.0, 100% = 100.0<br> for a and b: -100% = -125, 100% = 125 |



Tests

- [lab-001.html](https://wpt.fyi/results/css/css-color/lab-001.html) [(live test)](http://wpt.live/css/css-color/lab-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lab-001.html)
- [lab-002.html](https://wpt.fyi/results/css/css-color/lab-002.html) [(live test)](http://wpt.live/css/css-color/lab-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lab-002.html)
- [lab-003.html](https://wpt.fyi/results/css/css-color/lab-003.html) [(live test)](http://wpt.live/css/css-color/lab-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lab-003.html)
- [lab-004.html](https://wpt.fyi/results/css/css-color/lab-004.html) [(live test)](http://wpt.live/css/css-color/lab-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lab-004.html)
- [lab-005.html](https://wpt.fyi/results/css/css-color/lab-005.html) [(live test)](http://wpt.live/css/css-color/lab-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lab-005.html)
- [lab-006.html](https://wpt.fyi/results/css/css-color/lab-006.html) [(live test)](http://wpt.live/css/css-color/lab-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lab-006.html)
- [lab-007.html](https://wpt.fyi/results/css/css-color/lab-007.html) [(live test)](http://wpt.live/css/css-color/lab-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lab-007.html)
- [lab-008.html](https://wpt.fyi/results/css/css-color/lab-008.html) [(live test)](http://wpt.live/css/css-color/lab-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lab-008.html)
- [lab-l-over-100-1.html](https://wpt.fyi/results/css/css-color/lab-l-over-100-1.html) [(live test)](http://wpt.live/css/css-color/lab-l-over-100-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lab-l-over-100-1.html)
- [lab-l-over-100-2.html](https://wpt.fyi/results/css/css-color/lab-l-over-100-2.html) [(live test)](http://wpt.live/css/css-color/lab-l-over-100-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lab-l-over-100-2.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)
- [color-computed-lab.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-lab.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-lab.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-lab.html)
- [color-invalid-lab.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-lab.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-lab.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-lab.html)
- [color-valid-lab.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-lab.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-lab.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-lab.html)

In <a id="valdef-lab-lab"></a>Lab, the first argument specifies the CIE Lightness, L. This is a number between 0% or 0 and 100% or 100 Values less than 0% or 0 must be clamped to 0% at parsed-value time; values greater than 100% or 100 are clamped to 100% at parsed-value time.

The second and third arguments are the distances along the "a" and "b" axes in the Lab color space, as described in the previous section. These values are signed (allow both positive and negative values) and theoretically unbounded (but in practice do not exceed ±160 for real-world colors).

<a id="ref-for-typedef-color-alpha-value①⑤"></a>

<a id="ref-for-alpha-channel"></a>

There is an optional fourth [\<alpha-value\>](#typedef-color-alpha-value) component, separated by a slash, representing the [alpha component](#alpha-channel).

If the lightness of a Lab color (after clamping) is 0%, or 100% the color will be displayed as black, or white, respectively due to gamut mapping to the display.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-lab-samples"></a>
>
> ```css
>  lab(29.2345% 39.3825 20.0664);
>  lab(52.2345 40.1645 59.9971);
>  lab(60.2345 -5.3654 58.956);
>  lab(62.2345% -34.9638 47.7721);
>  lab(67.5345 -8.6911 -41.6019);
>  lab(29.69% 44.888% -29.04%)
> ```
<a id="funcdef-lch"></a>

<a id="ref-for-percentage-value②②"></a>

<a id="ref-for-comb-one⑤④"></a>

<a id="ref-for-number-value②⓪"></a>

<a id="ref-for-comb-one⑤⑤"></a>

<a id="ref-for-percentage-value②③"></a>

<a id="ref-for-comb-one⑤⑥"></a>

<a id="ref-for-number-value②①"></a>

<a id="ref-for-comb-one⑤⑦"></a>

<a id="ref-for-typedef-hue①②"></a>

<a id="ref-for-comb-one⑤⑧"></a>

<a id="ref-for-typedef-color-alpha-value①⑥"></a>

<a id="ref-for-comb-one⑤⑨"></a>

<a id="ref-for-mult-opt①②"></a>

```text
lch() = lch( [<percentage> | <number> | none]
      [ <percentage> | <number> | none]
      [ <hue> | none]
      [ / [<alpha-value> | none] ]? )
```
<a id="prr-lch"></a>



| Field               | Definition                                                               |
|---------------------|--------------------------------------------------------------------------|
| <strong>Percentages&#xA;      </strong> | Allowed for L and C                                                      |
| <strong>Percent reference range&#xA0;&#xA;      </strong> | for L: 0% = 0.0, 100% = 100.0<br> for C: 0% = 0, 100% = 150 |
| <strong>Powerless hue ε&#xA;      </strong> | C \<= 0.0015                                                             |



Tests

- [lch-001.html](https://wpt.fyi/results/css/css-color/lch-001.html) [(live test)](http://wpt.live/css/css-color/lch-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lch-001.html)
- [lch-002.html](https://wpt.fyi/results/css/css-color/lch-002.html) [(live test)](http://wpt.live/css/css-color/lch-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lch-002.html)
- [lch-003.html](https://wpt.fyi/results/css/css-color/lch-003.html) [(live test)](http://wpt.live/css/css-color/lch-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lch-003.html)
- [lch-004.html](https://wpt.fyi/results/css/css-color/lch-004.html) [(live test)](http://wpt.live/css/css-color/lch-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lch-004.html)
- [lch-005.html](https://wpt.fyi/results/css/css-color/lch-005.html) [(live test)](http://wpt.live/css/css-color/lch-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lch-005.html)
- [lch-006.html](https://wpt.fyi/results/css/css-color/lch-006.html) [(live test)](http://wpt.live/css/css-color/lch-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lch-006.html)
- [lch-007.html](https://wpt.fyi/results/css/css-color/lch-007.html) [(live test)](http://wpt.live/css/css-color/lch-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lch-007.html)
- [lch-008.html](https://wpt.fyi/results/css/css-color/lch-008.html) [(live test)](http://wpt.live/css/css-color/lch-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lch-008.html)
- [lch-009.html](https://wpt.fyi/results/css/css-color/lch-009.html) [(live test)](http://wpt.live/css/css-color/lch-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lch-009.html)
- [lch-010.html](https://wpt.fyi/results/css/css-color/lch-010.html) [(live test)](http://wpt.live/css/css-color/lch-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lch-010.html)
- [lch-l-over-100-1.html](https://wpt.fyi/results/css/css-color/lch-l-over-100-1.html) [(live test)](http://wpt.live/css/css-color/lch-l-over-100-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lch-l-over-100-1.html)
- [lch-l-over-100-2.html](https://wpt.fyi/results/css/css-color/lch-l-over-100-2.html) [(live test)](http://wpt.live/css/css-color/lch-l-over-100-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/lch-l-over-100-2.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

<a id="ref-for-funcdef-lab③"></a>

In CIE <a id="valdef-lch-lch"></a>LCH the first argument specifies the CIE Lightness L, interpreted identically to the Lightness argument of [lab()](#funcdef-lab).

The second argument is the chroma C, (roughly representing the "amount of color"). Its minimum useful value is 0, while its maximum is theoretically unbounded (but in practice does not exceed 230). If the provided value is negative, it is clamped to 0 at parsed-value time.

<a id="ref-for-typedef-hue①③"></a>

<a id="ref-for-funcdef-hsl①④"></a>

The third argument is the hue angle H. It’s interpreted similarly to the [\<hue\>](#typedef-hue) argument of [hsl()](#funcdef-hsl), but doesn’t map hues to angles in the same way because they are evenly spaced perceptually. Instead, 0deg points along the positive "a" axis (toward purplish red), (as does 360deg, 720deg, etc.); 90deg points along the positive "b" axis (toward mustard yellow), 180deg points along the negative "a" axis (toward greenish cyan), and 270deg points along the negative "b" axis (toward sky blue).

<a id="ref-for-typedef-color-alpha-value①⑦"></a>

<a id="ref-for-alpha-channel①"></a>

There is an optional fourth [\<alpha-value\>](#typedef-color-alpha-value) component, separated by a slash, representing the [alpha component](#alpha-channel).

<a id="ref-for-powerless-color-component①⓪"></a>

If the chroma of an LCH color is 0%, the hue component is [powerless](#powerless-color-component). If the lightness of an LCH color (after clamping) is 0%, or 100%, the color will be displayed as black, or white, respectively due to gamut mapping to the display.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-lch-samples"></a>
>
> ```css
>  lch(29.2345% 44.2 27);
>  lch(52.2345% 72.2 56.2);
>  lch(60.2345 59.2 95.2);
>  lch(62.2345% 59.2 126.2);
>  lch(67.5345% 42.5 258.2);
>  lch(29.69% 45.553% 327.1)
> ```
<a id="ref-for-valdef-lab-lab"></a>

<a id="ref-for-valdef-lch-lch"></a>

<a id="ref-for-funcdef-lab④"></a>

<a id="ref-for-funcdef-lch④"></a>

<a id="ref-for-legacy-color-syntax⑥"></a>

There is no Web compatibility issue with [lab](#valdef-lab-lab) or [lch](#valdef-lch-lch)', which are new in this level of the specification, and so [lab()](#funcdef-lab) and [lch()](#funcdef-lch) do <em>not</em> support a [legacy color syntax](#legacy-color-syntax) that separates all of their arguments with commas. Using commas inside these functions is an error.

<a id="ref-for-funcdef-oklab②"></a>

<a id="ref-for-funcdef-oklch④"></a>

### <a id="specifying-oklab-oklch"></a>9.4.  Specifying Oklab and OkLCh: the [oklab()](#funcdef-oklab) and [oklch()](#funcdef-oklch) functional notations

CSS allows colors to be directly expressed in Oklab and OkLCh.

<a id="funcdef-oklab"></a>

<a id="ref-for-percentage-value②④"></a>

<a id="ref-for-comb-one⑥⓪"></a>

<a id="ref-for-number-value②②"></a>

<a id="ref-for-comb-one⑥①"></a>

<a id="ref-for-percentage-value②⑤"></a>

<a id="ref-for-comb-one⑥②"></a>

<a id="ref-for-number-value②③"></a>

<a id="ref-for-comb-one⑥③"></a>

<a id="ref-for-percentage-value②⑥"></a>

<a id="ref-for-comb-one⑥④"></a>

<a id="ref-for-number-value②④"></a>

<a id="ref-for-comb-one⑥⑤"></a>

<a id="ref-for-typedef-color-alpha-value①⑧"></a>

<a id="ref-for-comb-one⑥⑥"></a>

<a id="ref-for-mult-opt①③"></a>

```text
oklab() = oklab( [ <percentage> | <number> | none]
    [ <percentage> | <number> | none]
    [ <percentage> | <number> | none]
    [ / [<alpha-value> | none] ]? )
```
<a id="prr-oklab"></a>



| Field               | Definition                                                                         |
|---------------------|------------------------------------------------------------------------------------|
| <strong>Percentages&#xA;      </strong> | Allowed for L, a and b                                                             |
| <strong>Percent reference range&#xA0;&#xA;      </strong> | for L: 0% = 0.0, 100% = 1.0<br> for a and b: -100% = -0.4, 100% = 0.4 |



Tests

- [oklab-001.html](https://wpt.fyi/results/css/css-color/oklab-001.html) [(live test)](http://wpt.live/css/css-color/oklab-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklab-001.html)
- [oklab-002.html](https://wpt.fyi/results/css/css-color/oklab-002.html) [(live test)](http://wpt.live/css/css-color/oklab-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklab-002.html)
- [oklab-003.html](https://wpt.fyi/results/css/css-color/oklab-003.html) [(live test)](http://wpt.live/css/css-color/oklab-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklab-003.html)
- [oklab-004.html](https://wpt.fyi/results/css/css-color/oklab-004.html) [(live test)](http://wpt.live/css/css-color/oklab-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklab-004.html)
- [oklab-005.html](https://wpt.fyi/results/css/css-color/oklab-005.html) [(live test)](http://wpt.live/css/css-color/oklab-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklab-005.html)
- [oklab-006.html](https://wpt.fyi/results/css/css-color/oklab-006.html) [(live test)](http://wpt.live/css/css-color/oklab-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklab-006.html)
- [oklab-007.html](https://wpt.fyi/results/css/css-color/oklab-007.html) [(live test)](http://wpt.live/css/css-color/oklab-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklab-007.html)
- [oklab-008.html](https://wpt.fyi/results/css/css-color/oklab-008.html) [(live test)](http://wpt.live/css/css-color/oklab-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklab-008.html)
- [oklab-009.html](https://wpt.fyi/results/css/css-color/oklab-009.html) [(live test)](http://wpt.live/css/css-color/oklab-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklab-009.html)
- [oklab-l-almost-0.html](https://wpt.fyi/results/css/css-color/oklab-l-almost-0.html) [(live test)](http://wpt.live/css/css-color/oklab-l-almost-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklab-l-almost-0.html)
- [oklab-l-almost-1.html](https://wpt.fyi/results/css/css-color/oklab-l-almost-1.html) [(live test)](http://wpt.live/css/css-color/oklab-l-almost-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklab-l-almost-1.html)
- [oklab-l-over-1-1.html](https://wpt.fyi/results/css/css-color/oklab-l-over-1-1.html) [(live test)](http://wpt.live/css/css-color/oklab-l-over-1-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklab-l-over-1-1.html)
- [oklab-l-over-1-2.html](https://wpt.fyi/results/css/css-color/oklab-l-over-1-2.html) [(live test)](http://wpt.live/css/css-color/oklab-l-over-1-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklab-l-over-1-2.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

In <a id="valdef-oklab-oklab"></a>Oklab the first argument specifies the Oklab Lightness. This is a number between 0% or 0 and 100% or 1.0.

Values less than 0% or 0.0 must be clamped to 0% at parsed-value time; values greater than 100% or 1.0 are clamped to 100% at parsed-value time.

The second and third arguments are the distances along the "a" and "b" axes in the Oklab color space, as described in the previous section. These values are signed (allow both positive and negative values) and theoretically unbounded (but in practice do not exceed ±0.5).

<a id="ref-for-typedef-color-alpha-value①⑨"></a>

<a id="ref-for-alpha-channel②"></a>

There is an optional fourth [\<alpha-value\>](#typedef-color-alpha-value) component, separated by a slash, representing the [alpha component](#alpha-channel).

If the lightness of an Oklab color is 0% or 0, or 100% or 1.0, the color will be displayed as black, or white, respectively due to gamut mapping to the display.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-oklab-samples"></a>
>
> ```css
>  oklab(40.101% 0.1147 0.0453);
>  oklab(59.686% 0.1009 0.1192);
>  oklab(0.65125 -0.0320 0.1274);
>  oklab(66.016% -0.1084 0.1114);
>  oklab(72.322% -0.0465 -0.1150);
>  oklab(42.1% 41% -25%)
> ```
<a id="funcdef-oklch"></a>

<a id="ref-for-percentage-value②⑦"></a>

<a id="ref-for-comb-one⑥⑦"></a>

<a id="ref-for-number-value②⑤"></a>

<a id="ref-for-comb-one⑥⑧"></a>

<a id="ref-for-percentage-value②⑧"></a>

<a id="ref-for-comb-one⑥⑨"></a>

<a id="ref-for-number-value②⑥"></a>

<a id="ref-for-comb-one⑦⓪"></a>

<a id="ref-for-typedef-hue①④"></a>

<a id="ref-for-comb-one⑦①"></a>

<a id="ref-for-typedef-color-alpha-value②⓪"></a>

<a id="ref-for-comb-one⑦②"></a>

<a id="ref-for-mult-opt①④"></a>

```text
oklch() = oklch( [ <percentage> | <number> | none]
      [ <percentage> | <number> | none]
      [ <hue> | none]
      [ / [<alpha-value> | none] ]? )
```
<a id="prr-oklch"></a>



| Field               | Definition                                                              |
|---------------------|-------------------------------------------------------------------------|
| <strong>Percentages&#xA;      </strong> | Allowed for L and C                                                     |
| <strong>Percent reference range&#xA0;&#xA;      </strong> | for L: 0% = 0.0, 100% = 1.0<br> for C: 0% = 0.0 100% = 0.4 |
| <strong>Powerless hue ε&#xA;      </strong> | C \<= 0.000004                                                          |



Tests

- [oklch-001.html](https://wpt.fyi/results/css/css-color/oklch-001.html) [(live test)](http://wpt.live/css/css-color/oklch-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-001.html)
- [oklch-002.html](https://wpt.fyi/results/css/css-color/oklch-002.html) [(live test)](http://wpt.live/css/css-color/oklch-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-002.html)
- [oklch-003.html](https://wpt.fyi/results/css/css-color/oklch-003.html) [(live test)](http://wpt.live/css/css-color/oklch-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-003.html)
- [oklch-004.html](https://wpt.fyi/results/css/css-color/oklch-004.html) [(live test)](http://wpt.live/css/css-color/oklch-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-004.html)
- [oklch-005.html](https://wpt.fyi/results/css/css-color/oklch-005.html) [(live test)](http://wpt.live/css/css-color/oklch-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-005.html)
- [oklch-006.html](https://wpt.fyi/results/css/css-color/oklch-006.html) [(live test)](http://wpt.live/css/css-color/oklch-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-006.html)
- [oklch-007.html](https://wpt.fyi/results/css/css-color/oklch-007.html) [(live test)](http://wpt.live/css/css-color/oklch-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-007.html)
- [oklch-008.html](https://wpt.fyi/results/css/css-color/oklch-008.html) [(live test)](http://wpt.live/css/css-color/oklch-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-008.html)
- [oklch-009.html](https://wpt.fyi/results/css/css-color/oklch-009.html) [(live test)](http://wpt.live/css/css-color/oklch-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-009.html)
- [oklch-010.html](https://wpt.fyi/results/css/css-color/oklch-010.html) [(live test)](http://wpt.live/css/css-color/oklch-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-010.html)
- [oklch-011.html](https://wpt.fyi/results/css/css-color/oklch-011.html) [(live test)](http://wpt.live/css/css-color/oklch-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-011.html)
- [oklch-l-almost-0.html](https://wpt.fyi/results/css/css-color/oklch-l-almost-0.html) [(live test)](http://wpt.live/css/css-color/oklch-l-almost-0.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-l-almost-0.html)
- [oklch-l-almost-1.html](https://wpt.fyi/results/css/css-color/oklch-l-almost-1.html) [(live test)](http://wpt.live/css/css-color/oklch-l-almost-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-l-almost-1.html)
- [oklch-l-over-1-1.html](https://wpt.fyi/results/css/css-color/oklch-l-over-1-1.html) [(live test)](http://wpt.live/css/css-color/oklch-l-over-1-1.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-l-over-1-1.html)
- [oklch-l-over-1-2.html](https://wpt.fyi/results/css/css-color/oklch-l-over-1-2.html) [(live test)](http://wpt.live/css/css-color/oklch-l-over-1-2.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/oklch-l-over-1-2.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

<a id="ref-for-funcdef-oklab③"></a>

In <a id="valdef-oklch-oklch"></a>OkLCh the first argument specifies the OkLCh Lightness L, interpreted identically to the Lightness argument of [oklab()](#funcdef-oklab).

The second argument is the chroma C. Its minimum useful value is 0, while its maximum is theoretically unbounded (but in practice does not exceed 0.5). If the provided value is negative, it is clamped to 0 at parsed-value time.

<a id="ref-for-typedef-hue①⑤"></a>

<a id="ref-for-funcdef-hsl①⑤"></a>

<a id="ref-for-funcdef-lch⑤"></a>

The third argument is the hue angle H. It’s interpreted similarly to the [\<hue\>](#typedef-hue) arguments of [hsl()](#funcdef-hsl) and [lch()](#funcdef-lch), but doesn’t map hues to angles in the same way. 0deg points along the positive "a" axis (toward purplish red), (as does 360deg, 720deg, etc.); 90deg points along the positive "b" axis (toward mustard yellow), 180deg points along the negative "a" axis (toward greenish cyan), and 270deg points along the negative "b" axis (toward sky blue).

<a id="ref-for-typedef-color-alpha-value②①"></a>

<a id="ref-for-alpha-channel③"></a>

There is an optional fourth [\<alpha-value\>](#typedef-color-alpha-value) component, separated by a slash, representing the [alpha component](#alpha-channel).

<a id="ref-for-powerless-color-component①①"></a>

If the chroma of an OkLCh color is 0% or 0, the hue component is [powerless](#powerless-color-component). If the lightness of an OkLCh color is 0% or 0, or 100% or 1.0, the color will be displayed as black, or white, respectively due to gamut mapping to the display.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-oklch-samples"></a>
>
> ```css
>  oklch(40.101% 0.12332 21.555);
>  oklch(59.686% 0.15619 49.7694);
>  oklch(0.65125 0.13138 104.097);
>  oklch(0.66016 0.15546 134.231);
>  oklch(72.322% 0.12403 247.996);
>  oklch(42.1% 48.25% 328.4)
> ```
<a id="ref-for-valdef-oklab-oklab"></a>

<a id="ref-for-valdef-oklch-oklch"></a>

<a id="ref-for-funcdef-oklab④"></a>

<a id="ref-for-funcdef-oklch⑤"></a>

<a id="ref-for-legacy-color-syntax⑦"></a>

There is no Web compatibility issue with [oklab](#valdef-oklab-oklab) or [oklch](#valdef-oklch-oklch)', which are new in this level of the specification, and so [oklab()](#funcdef-oklab) and [oklch()](#funcdef-oklch) do <em>not</em> support a [legacy color syntax](#legacy-color-syntax) that separates all of their arguments with commas. Using commas inside these functions is an error.

### <a id="lab-to-lch"></a>9.5.  Converting Lab or Oklab colors to LCH or OkLCh colors

Conversion to the polar form is trivial:

1.  C = sqrt(a^2 + b^2)
2.  if (C \> epsilon) H = atan2(b, a) else H is missing
3.  L is the same

<a id="ref-for-powerless-color-component①②"></a>

<a id="ref-for-missing-color-component⑥"></a>

For extremely small values of a and b (near-zero Chroma), although the visual color does not change from being on the neutral axis, small changes to the values can result in the reported hue angle swinging about wildly and being essentially random. In CSS, this means the hue is [powerless](#powerless-color-component), and treated as [missing](#missing-color-component) when converted into LCH or OkLCh; in non-CSS contexts this might be reflected as a missing value, such as NaN.

### <a id="lch-to-lab"></a>9.6.  Converting LCH or OkLCh colors to Lab or Oklab colors

Conversion to the rectangular form is trivial:

1.  If H is missing, a = b = 0
2.  Otherwise,
    1.  a = C cos(H)
    2.  b = C sin(H)
3.  L is the same

## <a id="predefined"></a>10.  Predefined Color Spaces

<a id="ref-for-valdef-color-display-p3①"></a>

<a id="ref-for-valdef-color-prophoto-rgb"></a>

<a id="ref-for-valdef-color-rec2020"></a>

CSS provides several predefined color spaces including [display-p3](#valdef-color-display-p3) [\[Display-P3\]](#biblio-display-p3), which is a wide gamut space typical of current wide-gamut monitors, [prophoto-rgb](#valdef-color-prophoto-rgb), widely used by photographers and [rec2020](#valdef-color-rec2020) [\[Rec.2020\]](#biblio-rec2020), which is a broadcast industry standard, ultra-wide gamut space capable of representing almost all visible real-world colors.

<a id="ref-for-funcdef-color②"></a>

### <a id="color-function"></a>10.1.  Specifying Predefined Colors: the [color()](#funcdef-color) function

<a id="ref-for-funcdef-color③"></a>

<a id="ref-for-color-space⑤"></a>

The [color()](#funcdef-color) function allows a color to be specified in a particular, specified [color space](#color-space) (rather than the implicit sRGB color space that most of the other color functions operate in). Its syntax is:

<a id="funcdef-color"></a>

<a id="ref-for-typedef-colorspace-params"></a>

<a id="ref-for-typedef-color-alpha-value②②"></a>

<a id="ref-for-comb-one⑦③"></a>

<a id="ref-for-mult-opt①⑤"></a>

<a id="typedef-colorspace-params"></a>

<a id="ref-for-typedef-predefined-rgb-params"></a>

<a id="ref-for-comb-one⑦④"></a>

<a id="ref-for-typedef-xyz-params"></a>

<a id="typedef-predefined-rgb-params"></a>

<a id="ref-for-typedef-predefined-rgb"></a>

<a id="ref-for-number-value②⑦"></a>

<a id="ref-for-comb-one⑦⑤"></a>

<a id="ref-for-percentage-value②⑨"></a>

<a id="ref-for-comb-one⑦⑥"></a>

<a id="ref-for-mult-num②"></a>

<a id="typedef-predefined-rgb"></a>

<a id="ref-for-valdef-color-srgb①"></a>

<a id="ref-for-comb-one⑦⑦"></a>

<a id="ref-for-valdef-color-srgb-linear"></a>

<a id="ref-for-comb-one⑦⑧"></a>

<a id="ref-for-valdef-color-display-p3②"></a>

<a id="ref-for-comb-one⑦⑨"></a>

<a id="ref-for-comb-one⑧⓪"></a>

<a id="ref-for-valdef-color-a98-rgb"></a>

<a id="ref-for-comb-one⑧①"></a>

<a id="ref-for-valdef-color-prophoto-rgb①"></a>

<a id="ref-for-comb-one⑧②"></a>

<a id="ref-for-valdef-color-rec2020①"></a>

<a id="typedef-xyz-params"></a>

<a id="ref-for-typedef-xyz-space"></a>

<a id="ref-for-number-value②⑧"></a>

<a id="ref-for-comb-one⑧③"></a>

<a id="ref-for-percentage-value③⓪"></a>

<a id="ref-for-comb-one⑧④"></a>

<a id="ref-for-mult-num③"></a>

<a id="typedef-xyz-space"></a>

<a id="ref-for-valdef-color-xyz"></a>

<a id="ref-for-comb-one⑧⑤"></a>

<a id="ref-for-valdef-color-xyz-d50"></a>

<a id="ref-for-comb-one⑧⑥"></a>

<a id="ref-for-valdef-color-xyz-d65"></a>

```text
color() = color( <colorspace-params> [ / [ <alpha-value> | none ] ]? )
<colorspace-params> = [ <predefined-rgb-params> | <xyz-params>]
<predefined-rgb-params> = <predefined-rgb> [ <number> | <percentage> | none ]{3}
<predefined-rgb> = srgb | srgb-linear | display-p3 | display-p3-linear | a98-rgb | prophoto-rgb | rec2020
<xyz-params> = <xyz-space> [ <number> | <percentage> | none ]{3}
<xyz-space> = xyz | xyz-d50 | xyz-d65
```
Tests

- [color-computed-color-function.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-color-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-color-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-color-function.html)
- [color-invalid-color-function.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-color-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-color-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-color-function.html)
- [color-valid-color-function.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-color-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-color-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-color-function.html)

The color function takes parameters specifying a color, in an explicitly listed color space.

<a id="ref-for-invalid-color①"></a>

<a id="ref-for-valid-color③"></a>

It represents either an [invalid color](#invalid-color), as described below, or a [valid color](#valid-color).

The parameters have the following form:

- <a id="ref-for-typedef-ident②"></a>

  <a id="ref-for-valdef-color-display-p3③"></a>

  <a id="ref-for-number-value②⑨"></a>

  <a id="ref-for-percentage-value③①"></a>

  An [\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident) denoting one of the [predefined color spaces](#predefined) (such as [display-p3](#valdef-color-display-p3)) Individual [predefined color spaces](#predefined) may further restrict whether [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s or both, may be used.

  <a id="ref-for-typedef-ident③"></a>

  <a id="ref-for-invalid-color②"></a>

  If the [\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident) names a non-existent color space (a name that does not match one of the [predefined color spaces](#predefined)), this argument represents an [invalid color](#invalid-color).

- The three parameter values that the color space takes (RGB or XYZ values).

<a id="ref-for-css-gamut-mapped①"></a>

An out of gamut color has component values less than 0 or 0%, or greater than 1 or 100%. These are not invalid, and are retained for intermediate computations; instead, for display, they are [css gamut mapped](#css-gamut-mapped) using a relative colorimetric intent which brings the values (in the display color space) within the range 0/0% to 1/100% at actual-value time.

- <a id="ref-for-typedef-color-alpha-value②③"></a>

  An optional slash-separated [\<alpha-value\>](#typedef-color-alpha-value).

Tests

- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

<a id="ref-for-funcdef-color④"></a>

<a id="ref-for-legacy-color-syntax⑧"></a>

There is no Web compatibility issue with [color()](#funcdef-color), which is new in this level of the specification, and so <a id="ref-for-funcdef-color⑤"></a>color() does <em>not</em> support a [legacy color syntax](#legacy-color-syntax) that separates all of its arguments with commas. Using commas inside this function is an error.

<a id="ref-for-invalid-color③"></a>

<a id="ref-for-out-of-gamut①"></a>

A color which is either an [invalid color](#invalid-color) or an [out of gamut](#out-of-gamut) color <a id="cant-be-displayed"></a>can’t be displayed.

<a id="ref-for-invalid-color④"></a>

<a id="ref-for-out-of-gamut②"></a>

<a id="ref-for-funcdef-color⑥"></a>

If the specified color <a id="can-be-displayed"></a>can be displayed, (that is, it isn’t an [invalid color](#invalid-color) and isn’t [out of gamut](#out-of-gamut)) then this is the actual value of the [color()](#funcdef-color) function.

<a id="ref-for-valid-color④"></a>

<a id="ref-for-cant-be-displayed"></a>

<a id="ref-for-css-gamut-mapped②"></a>

If the specified color is a [valid color](#valid-color) but [can’t be displayed](#cant-be-displayed), the actual value is derived from the specified color, [css gamut mapped](#css-gamut-mapped) for display.

<a id="ref-for-invalid-color⑤"></a>

<a id="ref-for-opaque-black"></a>

If the color is an [invalid color](#invalid-color), the used value is [opaque black](#opaque-black).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-2020-oog-p3"></a> This very intense lime color is in-gamut for rec.2020:
>
> ```css
> color(rec2020 0.42053 0.979780 0.00579);
> ```
>
> in LCH, that color is
>
> ```css
> lch(85.9017% 166.116 138.207);
> ```
>
> in display-p3, that color is
>
> ```css
> color(display-p3 -0.350289 1.00707 -0.144209);
> ```
>
> and is out of gamut for display-p3 (red and blue are negative, green is greater than 1). If you have a display-p3 screen, that color is:
>
> - <em>valid</em>
> - <em>in gamut</em> (for rec.2020)
> - <em>out of gamut</em> (for your display)
> - and so <em>can’t be displayed</em>
>
> The color used for display will be a less intense color produced automatically by gamut mapping.

<a id="ref-for-opaque-black①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-profoto-bad"></a> This example has a typo! An intense green is provided in profoto-rgb space (which doesn’t exist). This makes it invalid, so the used value is [opaque black](#opaque-black)
>
> ```css
> color(profoto-rgb 0.4835 0.9167 0.2188)
> ```
<a id="ref-for-valdef-color-srgb②"></a>

### <a id="predefined-sRGB"></a>10.2.  The Predefined sRGB Color Space: the [sRGB](#valdef-color-srgb) keyword

<a id="ref-for-funcdef-rgb①②"></a>

The <a id="sRGB-space"></a>sRGB predefined color space defined below is the same as is used for legacy sRGB colors, such as [rgb()](#funcdef-rgb).

<a id="valdef-color-srgb"></a>srgb  
<a id="ref-for-d65②"></a>

<a id="ref-for-valdef-color-srgb③"></a>

The [srgb](#valdef-color-srgb) [\[SRGB\]](#biblio-srgb) color space accepts three numeric parameters, representing the red, green, and blue components of the color. In-gamut colors have all three components in the range \[0, 1\]. The whitepoint is [D65](#d65).

[\[SRGB\]](#biblio-srgb) specifies two viewing conditions, <em>encoding</em> and <em>typical</em>. The [\[ICC\]](#biblio-icc) recommends using the <em>encoding</em> conditions for color conversion and for optimal viewing, which are the values in the table below.

sRGB is the default color space for CSS, used for all the legacy color functions.

It has the following characteristics:

<a id="prr-color-srgb"></a>

**Table 37**

| Chromaticity | x | y |
| --- | --- | --- |
| Red chromaticity | 0.640 | 0.330 |
| Green chromaticity | 0.300 | 0.600 |
| Blue chromaticity | 0.150 | 0.060 |

| Field | Value |
| --- | --- |
| White chromaticity | <a id="ref-for-d65③"></a> [D65](#d65) |
| Transfer function | see below |
| White luminance | 80.0 cd/m<sup>2</sup> |
| Black luminance | 0.20 cd/m<sup>2</sup> |
| Image state | display-referred |
| Percentages | Allowed for R, G and B |
| Percent reference range | for R,G,B: 0% = 0.0, 100% = 1.0 |

```javascript
let sign = c < 0? -1 : 1;
let abs = Math.abs(c);

if (abs <= 0.04045) {
  cl = c / 12.92;
}
else {
  cl = sign * (Math.pow((abs + 0.055) / 1.055, 2.4));
}
```
c is the gamma-encoded red, green or blue component. cl is the corresponding linear-light component.

<a id="fig-srgb-lch"></a> ![diagram of sRGB primaries and secondaries in LCH](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/sRGB-prim-sec.svg)

Visualization of the sRGB color space in LCH. The primaries and secondaries are shown.

Tests

- [predefined-001.html](https://wpt.fyi/results/css/css-color/predefined-001.html) [(live test)](http://wpt.live/css/css-color/predefined-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/predefined-001.html)
- [predefined-002.html](https://wpt.fyi/results/css/css-color/predefined-002.html) [(live test)](http://wpt.live/css/css-color/predefined-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/predefined-002.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

<a id="ref-for-valdef-color-srgb-linear①"></a>

### <a id="predefined-sRGB-linear"></a>10.3.  The Predefined Linear-Light sRGB Color Space: the [srgb-linear](#valdef-color-srgb-linear) keyword

<a id="ref-for-valdef-color-srgb④"></a>

The <a id="sRGB-linear-space"></a>sRGB-linear predefined color space is the same as [srgb](#valdef-color-srgb) <em>except</em> that the transfer function is linear-light (there is no gamma-encoding).

<a id="valdef-color-srgb-linear"></a>srgb-linear  
<a id="ref-for-d65④"></a>

<a id="ref-for-valdef-color-srgb-linear②"></a>

The [srgb-linear](#valdef-color-srgb-linear) [\[SRGB\]](#biblio-srgb) color space accepts three numeric parameters, representing the red, green, and blue components of the color. In-gamut colors have all three components in the range \[0, 1\]. The whitepoint is [D65](#d65).

It has the following characteristics:

<a id="prr-color-srgb-linear"></a>

**Table 38**

| Chromaticity | x | y |
| --- | --- | --- |
| Red chromaticity | 0.640 | 0.330 |
| Green chromaticity | 0.300 | 0.600 |
| Blue chromaticity | 0.150 | 0.060 |

| Field | Value |
| --- | --- |
| White chromaticity | <a id="ref-for-d65⑤"></a> [D65](#d65) |
| Transfer function | unity, see below |
| White luminance | 80.0 cd/m<sup>2</sup> |
| Black luminance | 0.20 cd/m<sup>2</sup> |
| Image state | display-referred |
| Percentages | Allowed for R, G and B |
| Percent reference range | for R,G,B: 0% = 0.0, 100% = 1.0 |

```javascript
cl = c;
```
c is the red, green or blue component. cl is the corresponding linear-light component, which is identical.

<a id="ref-for-valdef-color-srgb-linear③"></a>

<a id="ref-for-valdef-color-srgb⑤"></a>

To avoid banding artifacts, a [higher precision is required](#predefined-precision-table) for [srgb-linear](#valdef-color-srgb-linear) than for [srgb](#valdef-color-srgb).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="srgb-linear-swatches"></a> For example, these are the same color
> ```css
>  color(srgb 0.691 0.139 0.259)
>  color(srgb-linear 0.435 0.017 0.055)
> ```
Tests

- [srgb-linear-001.html](https://wpt.fyi/results/css/css-color/srgb-linear-001.html) [(live test)](http://wpt.live/css/css-color/srgb-linear-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/srgb-linear-001.html)
- [srgb-linear-002.html](https://wpt.fyi/results/css/css-color/srgb-linear-002.html) [(live test)](http://wpt.live/css/css-color/srgb-linear-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/srgb-linear-002.html)
- [srgb-linear-003.html](https://wpt.fyi/results/css/css-color/srgb-linear-003.html) [(live test)](http://wpt.live/css/css-color/srgb-linear-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/srgb-linear-003.html)
- [srgb-linear-004.html](https://wpt.fyi/results/css/css-color/srgb-linear-004.html) [(live test)](http://wpt.live/css/css-color/srgb-linear-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/srgb-linear-004.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

<a id="ref-for-valdef-color-display-p3④"></a>

### <a id="predefined-display-p3"></a>10.4.  The Predefined Display P3 Color Space: the [display-p3](#valdef-color-display-p3) keyword

<a id="valdef-color-display-p3"></a>display-p3  
<a id="ref-for-d65⑥"></a>

<a id="ref-for-valdef-color-display-p3⑤"></a>

The [display-p3](#valdef-color-display-p3) [\[Display-P3\]](#biblio-display-p3) color space accepts three numeric parameters, representing the red, green, and blue components of the color. In-gamut colors have all three components in the range \[0, 1\]. It uses the same primary chromaticities as [\[DCI-P3\]](#biblio-dci-p3), but with a [D65](#d65) whitepoint, and the same transfer curve as sRGB.

Modern displays, TVs, laptop screens and phone screens are able to display all, or nearly all, of the display-p3 gamut.

It has the following characteristics:

<a id="prr-color-display-p3"></a>

**Table 39**

| Chromaticity | x | y |
| --- | --- | --- |
| Red chromaticity | 0.680 | 0.320 |
| Green chromaticity | 0.265 | 0.690 |
| Blue chromaticity | 0.150 | 0.060 |

| Field | Value |
| --- | --- |
| White chromaticity | <a id="ref-for-d65⑦"></a> [D65](#d65) |
| Transfer function | same as srgb |
| White luminance | 80.0 cd/m<sup>2</sup> |
| Black luminance | 0.80 cd/m<sup>2</sup> |
| Image state | display-referred |
| Percentages | Allowed for R, G and B |
| Percent reference range | for R,G,B: 0% = 0.0, 100% = 1.0 |

<a id="fig-displayp3-lch"></a> ![diagram of P3 primaries and secondaries in LCH](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/P3-prim-sec.svg)

Visualization of the P3 color space in LCH. The primaries and secondaries are shown (but in sRGB, not in the correct colors). For comparison, the sRGB primaries and secondaries are also shown, as dashed circles. P3 primaries have higher Chroma.

Tests

- [predefined-005.html](https://wpt.fyi/results/css/css-color/predefined-005.html) [(live test)](http://wpt.live/css/css-color/predefined-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/predefined-005.html)
- [predefined-006.html](https://wpt.fyi/results/css/css-color/predefined-006.html) [(live test)](http://wpt.live/css/css-color/predefined-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/predefined-006.html)
- [display-p3-001.html](https://wpt.fyi/results/css/css-color/display-p3-001.html) [(live test)](http://wpt.live/css/css-color/display-p3-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/display-p3-001.html)
- [display-p3-002.html](https://wpt.fyi/results/css/css-color/display-p3-002.html) [(live test)](http://wpt.live/css/css-color/display-p3-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/display-p3-002.html)
- [display-p3-003.html](https://wpt.fyi/results/css/css-color/display-p3-003.html) [(live test)](http://wpt.live/css/css-color/display-p3-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/display-p3-003.html)
- [display-p3-004.html](https://wpt.fyi/results/css/css-color/display-p3-004.html) [(live test)](http://wpt.live/css/css-color/display-p3-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/display-p3-004.html)
- [display-p3-005.html](https://wpt.fyi/results/css/css-color/display-p3-005.html) [(live test)](http://wpt.live/css/css-color/display-p3-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/display-p3-005.html)
- [display-p3-006.html](https://wpt.fyi/results/css/css-color/display-p3-006.html) [(live test)](http://wpt.live/css/css-color/display-p3-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/display-p3-006.html)

<!-- -->

- [2d.color.space.p3.fillText.html](https://wpt.fyi/results/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.fillText.html) [(live test)](http://wpt.live/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.fillText.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.fillText.html)
- [2d.color.space.p3.fillText.shadow.html](https://wpt.fyi/results/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.fillText.shadow.html) [(live test)](http://wpt.live/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.fillText.shadow.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.fillText.shadow.html)
- [2d.color.space.p3.strokeText.html](https://wpt.fyi/results/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.strokeText.html) [(live test)](http://wpt.live/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.strokeText.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.strokeText.html)
- [2d.color.space.p3.to.p3.html](https://wpt.fyi/results/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.to.p3.html) [(live test)](http://wpt.live/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.to.p3.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.to.p3.html)
- [2d.color.space.p3.to.srgb.html](https://wpt.fyi/results/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.to.srgb.html) [(live test)](http://wpt.live/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.to.srgb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.to.srgb.html)
- [2d.color.space.p3.toBlob.p3.canvas.html](https://wpt.fyi/results/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toBlob.p3.canvas.html) [(live test)](http://wpt.live/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toBlob.p3.canvas.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toBlob.p3.canvas.html)
- [2d.color.space.p3.toBlob.with.putImageData.html](https://wpt.fyi/results/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toBlob.with.putImageData.html) [(live test)](http://wpt.live/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toBlob.with.putImageData.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toBlob.with.putImageData.html)
- [2d.color.space.p3.toDataURL.jpeg.p3.canvas.html](https://wpt.fyi/results/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toDataURL.jpeg.p3.canvas.html) [(live test)](http://wpt.live/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toDataURL.jpeg.p3.canvas.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toDataURL.jpeg.p3.canvas.html)
- [2d.color.space.p3.toDataURL.p3.canvas.html](https://wpt.fyi/results/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toDataURL.p3.canvas.html) [(live test)](http://wpt.live/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toDataURL.p3.canvas.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toDataURL.p3.canvas.html)
- [2d.color.space.p3.toDataURL.with.putImageData.html](https://wpt.fyi/results/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toDataURL.with.putImageData.html) [(live test)](http://wpt.live/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toDataURL.with.putImageData.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/html/canvas/element/wide-gamut-canvas/2d.color.space.p3.toDataURL.with.putImageData.html)

### <a id="predefined-display-p3-linear"></a>10.5.  The Predefined Linear-Light Display P3 Color Space: the display-p3-linear keyword

<a id="ref-for-valdef-color-display-p3⑥"></a>

The <a id="display-p3-linear-space"></a>display-p3-linear predefined color space is the same as [display-p3](#valdef-color-display-p3) <em>except</em> that the transfer function is linear-light (there is no gamma-encoding).

It has the following characteristics:

<a id="prr-color-display-p3-linear"></a>

**Table 40**

| Chromaticity | x | y |
| --- | --- | --- |
| Red chromaticity | 0.680 | 0.320 |
| Green chromaticity | 0.265 | 0.690 |
| Blue chromaticity | 0.150 | 0.060 |

| Field | Value |
| --- | --- |
| White chromaticity | <a id="ref-for-d65⑧"></a> [D65](#d65) |
| Transfer function | unity, see below |
| White luminance | 80.0 cd/m<sup>2</sup> |
| Black luminance | 0.80 cd/m<sup>2</sup> |
| Image state | display-referred |
| Percentages | Allowed for R, G and B |
| Percent reference range | for R,G,B: 0% = 0.0, 100% = 1.0 |

```javascript
cl = c;
```
c is the red, green or blue component. cl is the corresponding linear-light component, which is identical.

<a id="ref-for-valdef-color-display-p3⑦"></a>

To avoid banding artifacts, a [higher precision is required](#predefined-precision-table) for display-p3-linear than for [display-p3](#valdef-color-display-p3).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="display-p3-linear-swatches"></a> For example, these are the same color
>
> ```css
>  color(display-p3 0.591 0.123 0.264)
>  color(display-p3-linear 0.3081 0.014 0.0567)
> ```
Tests

- [display-p3-linear-001.html](https://wpt.fyi/results/css/css-color/display-p3-linear-001.html) [(live test)](http://wpt.live/css/css-color/display-p3-linear-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/display-p3-linear-001.html)
- [display-p3-linear-002.html](https://wpt.fyi/results/css/css-color/display-p3-linear-002.html) [(live test)](http://wpt.live/css/css-color/display-p3-linear-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/display-p3-linear-002.html)
- [display-p3-linear-003.html](https://wpt.fyi/results/css/css-color/display-p3-linear-003.html) [(live test)](http://wpt.live/css/css-color/display-p3-linear-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/display-p3-linear-003.html)
- [display-p3-linear-004.html](https://wpt.fyi/results/css/css-color/display-p3-linear-004.html) [(live test)](http://wpt.live/css/css-color/display-p3-linear-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/display-p3-linear-004.html)
- [display-p3-linear-005.html](https://wpt.fyi/results/css/css-color/display-p3-linear-005.html) [(live test)](http://wpt.live/css/css-color/display-p3-linear-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/display-p3-linear-005.html)
- [display-p3-linear-006.html](https://wpt.fyi/results/css/css-color/display-p3-linear-006.html) [(live test)](http://wpt.live/css/css-color/display-p3-linear-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/display-p3-linear-006.html)

<a id="ref-for-valdef-color-a98-rgb①"></a>

### <a id="predefined-a98-rgb"></a>10.6.  The Predefined A98 RGB Color Space: the [a98-rgb](#valdef-color-a98-rgb) keyword

<a id="valdef-color-a98-rgb"></a>a98-rgb  
<a id="ref-for-valdef-color-a98-rgb②"></a>

The [a98-rgb](#valdef-color-a98-rgb) color space accepts three numeric parameters, representing the red, green, and blue components of the color. In-gamut colors have all three components in the range \[0, 1\]. The transfer curve is a gamma function, close to but not exactly 1/2.2.

It has the following characteristics:

<a id="prr-color-a98-rgb"></a>

**Table 41**

| Chromaticity | x | y |
| --- | --- | --- |
| Red chromaticity | 0.6400 | 0.3300 |
| Green chromaticity | 0.2100 | 0.7100 |
| Blue chromaticity | 0.1500 | 0.0600 |

| Field | Value |
| --- | --- |
| White chromaticity | <a id="ref-for-d65⑨"></a> [D65](#d65) |
| Transfer function | 256/563 |
| White luminance | 160.0 cd/m<sup>2</sup> |
| Black luminance | 0.5557 cd/m<sup>2</sup> |
| Image state | display-referred |
| Percentages | Allowed for R, G and B |
| Percent reference range | for R,G,B: 0% = 0.0, 100% = 1.0 |

<a id="fig-a89-lch"></a> ![diagram of a98 primaries and secondaries in LCH](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/a98-prim-sec.svg)

Visualization of the A98 color space in LCH. The primaries and secondaries are shown (but in sRGB, not in the correct colors). For comparison, the sRGB primaries and secondaries are also shown, as dashed circles. a98 primaries have higher Chroma, especially the yellow, green and cyan.

Tests

- [predefined-007.html](https://wpt.fyi/results/css/css-color/predefined-007.html) [(live test)](http://wpt.live/css/css-color/predefined-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/predefined-007.html)
- [predefined-008.html](https://wpt.fyi/results/css/css-color/predefined-008.html) [(live test)](http://wpt.live/css/css-color/predefined-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/predefined-008.html)
- [a98rgb-001.html](https://wpt.fyi/results/css/css-color/a98rgb-001.html) [(live test)](http://wpt.live/css/css-color/a98rgb-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/a98rgb-001.html)
- [a98rgb-002.html](https://wpt.fyi/results/css/css-color/a98rgb-002.html) [(live test)](http://wpt.live/css/css-color/a98rgb-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/a98rgb-002.html)
- [a98rgb-003.html](https://wpt.fyi/results/css/css-color/a98rgb-003.html) [(live test)](http://wpt.live/css/css-color/a98rgb-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/a98rgb-003.html)
- [a98rgb-004.html](https://wpt.fyi/results/css/css-color/a98rgb-004.html) [(live test)](http://wpt.live/css/css-color/a98rgb-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/a98rgb-004.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

<a id="ref-for-valdef-color-prophoto-rgb②"></a>

### <a id="predefined-prophoto-rgb"></a>10.7.  The Predefined ProPhoto RGB Color Space: the [prophoto-rgb](#valdef-color-prophoto-rgb) keyword

<a id="valdef-color-prophoto-rgb"></a>prophoto-rgb  
<a id="ref-for-d50①"></a>

<a id="ref-for-valdef-color-prophoto-rgb③"></a>

The [prophoto-rgb](#valdef-color-prophoto-rgb) color space accepts three numeric parameters, representing the red, green, and blue components of the color. In-gamut colors have all three components in the range \[0, 1\]. The transfer curve is a gamma function with a value of 1/1.8, and a small linear portion near black. The white point is [D50](#d50), the same as is used by CIE Lab. Thus, conversion to CIE Lab does not require the chromatic adaptation step.

<a id="ref-for-valdef-color-prophoto-rgb④"></a>

The ProPhoto RGB space uses hyper-saturated, non physically realizable primaries. These were chosen to allow a wide color gamut and in particular, to minimize hue shifts under tonal manipulation. It is often used in digital photography as a wide gamut color space for the archival version of photographic images. The [prophoto-rgb](#valdef-color-prophoto-rgb) color space allows CSS to specify colors that will match colors in such images having the same RGB values.

The ProPhoto RGB space was originally developed by Kodak and is described in [\[Wolfe\]](#biblio-wolfe). It was standardized by ISO as [\[ROMM\]](#biblio-romm),[\[ROMM-RGB\]](#biblio-romm-rgb).

The white luminance is given as a range, and the viewing flare (and thus, the black luminance) is 0.5% to 1.0% of this.

It has the following characteristics:

<a id="prr-color-prophoto-rgb"></a>

**Table 42**

| Chromaticity | x | y |
| --- | --- | --- |
| Red chromaticity | 0.734699 | 0.265301 |
| Green chromaticity | 0.159597 | 0.840403 |
| Blue chromaticity | 0.036598 | 0.000105 |

| Field | Value |
| --- | --- |
| White chromaticity | <a id="ref-for-d50②"></a> [D50](#d50) |
| Transfer function | see below |
| White luminance | 160.0 to 640.0 cd/m<sup>2</sup> |
| Black luminance | See text |
| Image state | display-referred |
| Percentages | Allowed for R, G and B |
| Percent reference range | for R,G,B: 0% = 0.0, 100% = 1.0 |

```javascript
const E = 16/512;
let sign = c < 0? -1 : 1;
let abs = Math.abs(c);

if (abs <= E) {
  cl =  c / 16;
}
else {
  cl = sign * Math.pow(c, 1.8);
}
```
c is the gamma-encoded red, green or blue component. cl is the corresponding linear-light component.

<a id="fig-prophoto-lch"></a> ![diagram of prophoto primaries and secondaries in LCH](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/prophoto-prim-sec.svg)

Visualization of the prophoto-rgb color space in LCH. The primaries and secondaries are shown (but in sRGB, not in the correct colors). For comparison, the sRGB primaries and secondaries are also shown, as dashed circles. prophoto-rgb primaries and secondaries have much higher Chroma, but much of this ultrawide gamut does not correspond to physically realizable colors.

Tests

- [predefined-009.html](https://wpt.fyi/results/css/css-color/predefined-009.html) [(live test)](http://wpt.live/css/css-color/predefined-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/predefined-009.html)
- [predefined-010.html](https://wpt.fyi/results/css/css-color/predefined-010.html) [(live test)](http://wpt.live/css/css-color/predefined-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/predefined-010.html)
- [prophoto-rgb-001.html](https://wpt.fyi/results/css/css-color/prophoto-rgb-001.html) [(live test)](http://wpt.live/css/css-color/prophoto-rgb-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/prophoto-rgb-001.html)
- [prophoto-rgb-002.html](https://wpt.fyi/results/css/css-color/prophoto-rgb-002.html) [(live test)](http://wpt.live/css/css-color/prophoto-rgb-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/prophoto-rgb-002.html)
- [prophoto-rgb-003.html](https://wpt.fyi/results/css/css-color/prophoto-rgb-003.html) [(live test)](http://wpt.live/css/css-color/prophoto-rgb-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/prophoto-rgb-003.html)
- [prophoto-rgb-004.html](https://wpt.fyi/results/css/css-color/prophoto-rgb-004.html) [(live test)](http://wpt.live/css/css-color/prophoto-rgb-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/prophoto-rgb-004.html)
- [prophoto-rgb-005.html](https://wpt.fyi/results/css/css-color/prophoto-rgb-005.html) [(live test)](http://wpt.live/css/css-color/prophoto-rgb-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/prophoto-rgb-005.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

<a id="ref-for-valdef-color-rec2020②"></a>

### <a id="predefined-rec2020"></a>10.8.  The Predefined ITU-R BT.2020-2 Color Space: the [rec2020](#valdef-color-rec2020) keyword

<a id="valdef-color-rec2020"></a>rec2020  
<a id="ref-for-valdef-color-rec2020③"></a>

The [rec2020](#valdef-color-rec2020) [\[Rec.2020\]](#biblio-rec2020) color space accepts three numeric parameters, representing the red, green, and blue components of the color. In-gamut colors have all three components in the range \[0, 1\], ("full-range", in video terminology). ITU Reference 2020 is used for Ultra High Definition, 4k and 8k television.

The primaries are physically realizable, but with difficulty as they lie very close to the spectral locus.

Current displays are unable to reproduce the full gamut of rec2020. Coverage is expected to increase over time as displays improve.

It has the following characteristics:

<a id="prr-color-rec2020"></a>

**Table 43**

| Chromaticity | x | y |
| --- | --- | --- |
| Red chromaticity | 0.708 | 0.292 |
| Green chromaticity | 0.170 | 0.797 |
| Blue chromaticity | 0.131 | 0.046 |

| Field | Value |
| --- | --- |
| White chromaticity | <a id="ref-for-d65①⓪"></a> [D65](#d65) |
| Transfer function | gamma 2.40, from [\[REC_BT.1886\]](#biblio-rec_bt1886) |
| Image state | display-referred |
| Percentages | Allowed for R, G and B |
| Percent reference range | for R,G,B: 0% = 0.0, 100% = 1.0 |

<a id="fig-rec2020-lch"></a> ![diagram of rec2020 primaries and secondaries in LCH](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/2020-prim-sec.svg)

Visualization of the rec2020 color space in LCH. The primaries and secondaries are shown (but in sRGB, not in the correct colors). For comparison, the sRGB primaries and secondaries are also shown, as dashed circles. rec2020 primaries have much higher Chroma.

Tests

- [predefined-011.html](https://wpt.fyi/results/css/css-color/predefined-011.html) [(live test)](http://wpt.live/css/css-color/predefined-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/predefined-011.html)
- [predefined-012.html](https://wpt.fyi/results/css/css-color/predefined-012.html) [(live test)](http://wpt.live/css/css-color/predefined-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/predefined-012.html)
- [rec2020-001.html](https://wpt.fyi/results/css/css-color/rec2020-001.html) [(live test)](http://wpt.live/css/css-color/rec2020-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rec2020-001.html)
- [rec2020-002.html](https://wpt.fyi/results/css/css-color/rec2020-002.html) [(live test)](http://wpt.live/css/css-color/rec2020-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rec2020-002.html)
- [rec2020-003.html](https://wpt.fyi/results/css/css-color/rec2020-003.html) [(live test)](http://wpt.live/css/css-color/rec2020-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rec2020-003.html)
- [rec2020-004.html](https://wpt.fyi/results/css/css-color/rec2020-004.html) [(live test)](http://wpt.live/css/css-color/rec2020-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rec2020-004.html)
- [rec2020-005.html](https://wpt.fyi/results/css/css-color/rec2020-005.html) [(live test)](http://wpt.live/css/css-color/rec2020-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/rec2020-005.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

<a id="ref-for-valdef-color-xyz-d50①"></a>

<a id="ref-for-valdef-color-xyz-d65①"></a>

<a id="ref-for-valdef-color-xyz①"></a>

### <a id="predefined-xyz"></a>10.9.  The Predefined CIE XYZ Color Spaces: the [xyz-d50](#valdef-color-xyz-d50), [xyz-d65](#valdef-color-xyz-d65), and [xyz](#valdef-color-xyz) keywords

<a id="valdef-color-xyz-d50"></a>xyz-d50, <a id="valdef-color-xyz-d65"></a>xyz-d65, <a id="valdef-color-xyz"></a>xyz  
<a id="ref-for-luminance①"></a>

<a id="ref-for-valdef-color-xyz②"></a>

The [xyz](#valdef-color-xyz) color space accepts three numeric parameters, representing the X,Y and Z values. It represents the CIE XYZ [\[COLORIMETRY\]](#biblio-colorimetry) color space, scaled such that diffuse white has a [luminance](#luminance) (Y) of 1.0. and, if necessary, chromatically adapted to the reference white.

<a id="ref-for-valdef-color-xyz-d50②"></a>

<a id="ref-for-d50③"></a>

<a id="ref-for-valdef-color-xyz-d65②"></a>

<a id="ref-for-valdef-color-xyz③"></a>

<a id="ref-for-d65①①"></a>

The reference white for [xyz-d50](#valdef-color-xyz-d50) is [D50](#d50), while the reference white for [xyz-d65](#valdef-color-xyz-d65) and [xyz](#valdef-color-xyz) is [D65](#d65).

Values greater than 1.0/100% are allowed and must not be clamped; colors where Y is greater than 1.0 represent colors brighter than diffuse white. Values less than 0/0% are uncommon, but can occur as a result of chromatic adaptation, and likewise must not be clamped.

It has the following characteristics:

<a id="prr-color-xyz"></a>



| Field               | Definition                      |
|---------------------|---------------------------------|
| <strong>Percentages&#xA;        </strong> | Allowed for X,Y,Z               |
| <strong>Percent reference range&#xA0;&#xA;        </strong> | for X,Y,Z: 0% = 0.0, 100% = 1.0 |



> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-xyz"></a> These are exactly equivalent:
> ```css
>  #7654CD
>  rgb(46.27% 32.94% 80.39%)
>  lab(44.36% 36.05 -58.99)
>  color(xyz-d50 0.2005 0.14089 0.4472)
>  color(xyz-d65 0.21661 0.14602 0.59452)
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-xyz-white"></a> These colors are exactly equivalent, and represent white:
> ```css
>  #FFFFFF
>  color(xyz-d50 0.9643 1 0.8251)
>  color(xyz-d65 0.9505 1 1.089)
> ```
Tests

- [predefined-016.html](https://wpt.fyi/results/css/css-color/predefined-016.html) [(live test)](http://wpt.live/css/css-color/predefined-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/predefined-016.html)
- [xyz-001.html](https://wpt.fyi/results/css/css-color/xyz-001.html) [(live test)](http://wpt.live/css/css-color/xyz-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-001.html)
- [xyz-002.html](https://wpt.fyi/results/css/css-color/xyz-002.html) [(live test)](http://wpt.live/css/css-color/xyz-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-002.html)
- [xyz-003.html](https://wpt.fyi/results/css/css-color/xyz-003.html) [(live test)](http://wpt.live/css/css-color/xyz-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-003.html)
- [xyz-004.html](https://wpt.fyi/results/css/css-color/xyz-004.html) [(live test)](http://wpt.live/css/css-color/xyz-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-004.html)
- [xyz-005.html](https://wpt.fyi/results/css/css-color/xyz-005.html) [(live test)](http://wpt.live/css/css-color/xyz-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-005.html)
- [xyz-d50-001.html](https://wpt.fyi/results/css/css-color/xyz-d50-001.html) [(live test)](http://wpt.live/css/css-color/xyz-d50-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-d50-001.html)
- [xyz-d50-002.html](https://wpt.fyi/results/css/css-color/xyz-d50-002.html) [(live test)](http://wpt.live/css/css-color/xyz-d50-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-d50-002.html)
- [xyz-d50-003.html](https://wpt.fyi/results/css/css-color/xyz-d50-003.html) [(live test)](http://wpt.live/css/css-color/xyz-d50-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-d50-003.html)
- [xyz-d50-004.html](https://wpt.fyi/results/css/css-color/xyz-d50-004.html) [(live test)](http://wpt.live/css/css-color/xyz-d50-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-d50-004.html)
- [xyz-d50-005.html](https://wpt.fyi/results/css/css-color/xyz-d50-005.html) [(live test)](http://wpt.live/css/css-color/xyz-d50-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-d50-005.html)
- [xyz-d65-001.html](https://wpt.fyi/results/css/css-color/xyz-d65-001.html) [(live test)](http://wpt.live/css/css-color/xyz-d65-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-d65-001.html)
- [xyz-d65-002.html](https://wpt.fyi/results/css/css-color/xyz-d65-002.html) [(live test)](http://wpt.live/css/css-color/xyz-d65-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-d65-002.html)
- [xyz-d65-003.html](https://wpt.fyi/results/css/css-color/xyz-d65-003.html) [(live test)](http://wpt.live/css/css-color/xyz-d65-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-d65-003.html)
- [xyz-d65-004.html](https://wpt.fyi/results/css/css-color/xyz-d65-004.html) [(live test)](http://wpt.live/css/css-color/xyz-d65-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-d65-004.html)
- [xyz-d65-005.html](https://wpt.fyi/results/css/css-color/xyz-d65-005.html) [(live test)](http://wpt.live/css/css-color/xyz-d65-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/xyz-d65-005.html)
- [color-valid.html](https://wpt.fyi/results/css/css-color/parsing/color-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid.html)

### <a id="predefined-to-lab-oklab"></a>10.10.  Converting Predefined Color Spaces to Lab or Oklab

For all predefined RGB color spaces, conversion to Lab requires several steps, although in practice all but the first step are linear calculations and can be combined.

<a id="predefined-to-lab"></a>

1.  Convert from gamma-encoded RGB to linear-light RGB (undo gamma encoding)

2.  Convert from linear RGB to CIE XYZ

3.  <a id="ref-for-valdef-color-prophoto-rgb⑤"></a>

    <a id="ref-for-d50④"></a>

    <a id="ref-for-valdef-color-rec2020④"></a>

    <a id="ref-for-valdef-color-a98-rgb③"></a>

    <a id="ref-for-valdef-color-display-p3⑧"></a>

    <a id="ref-for-valdef-color-srgb⑥"></a>

    <a id="ref-for-d65①②"></a>

    If needed, convert from a [D65](#d65) whitepoint (used by [sRGB](#valdef-color-srgb), [display-p3](#valdef-color-display-p3), [a98-rgb](#valdef-color-a98-rgb) and [rec2020](#valdef-color-rec2020)) to the [D50](#d50) whitepoint used in Lab, with the linear Bradford transform. [prophoto-rgb](#valdef-color-prophoto-rgb) already has a <a id="ref-for-d50⑤"></a>D50 whitepoint.

4.  Convert D50-adapted XYZ to Lab

<a id="ref-for-valdef-color-prophoto-rgb⑥"></a>

Conversion to Oklab is similar, but the chromatic adaptation step is only needed for [prophoto-rgb](#valdef-color-prophoto-rgb).

<a id="predefined-to-oklab"></a>

1.  Convert from gamma-encoded RGB to linear-light RGB (undo gamma encoding)

2.  Convert from linear RGB to CIE XYZ

3.  <a id="ref-for-d65①③"></a>

    <a id="ref-for-valdef-color-prophoto-rgb⑦"></a>

    <a id="ref-for-d50⑥"></a>

    If needed, convert from a [D50](#d50) whitepoint (used by [prophoto-rgb](#valdef-color-prophoto-rgb)) to the [D65](#d65) whitepoint used in Oklab, with the linear Bradford transform.

4.  Convert D65-adapted XYZ to Oklab

There is sample JavaScript code for these conversions in [§ 19 Sample code for Color Conversions](#color-conversion-code).

### <a id="oklab-lab-to-predefined"></a>10.11.  Converting Lab or Oklab to Predefined RGB Color Spaces

<a id="ref-for-valdef-color-display-p3⑨"></a>

<a id="ref-for-valdef-color-rec2020⑤"></a>

Conversion from Lab to predefined spaces like [display-p3](#valdef-color-display-p3) or [rec2020](#valdef-color-rec2020) also requires multiple steps, and again in practice all but the last step are linear calculations and can be combined.

<a id="lab-to-predefined"></a>

1.  Convert Lab to (D50-adapted) XYZ

2.  <a id="ref-for-valdef-color-prophoto-rgb⑧"></a>

    <a id="ref-for-d65①④"></a>

    <a id="ref-for-d50⑦"></a>

    If needed, convert from a [D50](#d50) whitepoint (used by Lab) to the [D65](#d65) whitepoint used in sRGB and most other RGB spaces, with the linear Bradford transform. [prophoto-rgb](#valdef-color-prophoto-rgb)' does not require this step.

3.  Convert from (D65-adapted) CIE XYZ to linear RGB

4.  Convert from linear-light RGB to RGB (do gamma encoding)

<a id="ref-for-valdef-color-prophoto-rgb⑨"></a>

Conversion from Oklab is similar, but the chromatic adaptation step is only needed for [prophoto-rgb](#valdef-color-prophoto-rgb).

<a id="oklab-to-predefined"></a>

1.  Convert Oklab to (D65-adapted) XYZ

2.  <a id="ref-for-valdef-color-prophoto-rgb①⓪"></a>

    <a id="ref-for-d50⑧"></a>

    <a id="ref-for-d65①⑤"></a>

    If needed, convert from a [D65](#d65) whitepoint (used by Oklab) to the [D50](#d50) whitepoint used in [prophoto-rgb](#valdef-color-prophoto-rgb), with the linear Bradford transform.

3.  Convert from (D65-adapted) CIE XYZ to linear RGB

4.  Convert from linear-light RGB to RGB (do gamma encoding)

There is sample JavaScript code for these conversions in [§ 19 Sample code for Color Conversions](#color-conversion-code).

Implementations may choose to implement these steps in some other way (for example, using an ICC profile with relative colorimetric rendering intent) provided the results are the same for colors inside both the source and destination gamuts.

### <a id="predefined-to-predefined"></a>10.12.  Converting Between Predefined RGB Color Spaces

Conversion from one predefined RGB color space to another requires multiple steps, one of which is only needed when the whitepoints differ. To convert from <em>src</em> to <em>dest</em>:

1.  Convert from gamma-encoded <em>src</em>RGB to linear-light <em>src</em>RGB (undo gamma encoding)
2.  Convert from linear <em>src</em>RGB to CIE XYZ
3.  If <em>src</em> and <em>dest</em> have different whitepoints, convert the XYZ value from <em>src</em>White to <em>dest</em>White with the linear Bradford transform.
4.  Convert from CIE XYZ to linear <em>dest</em>RGB
5.  Convert from linear-light <em>dest</em>RGB to <em>dest</em>RGB (do gamma encoding)

There is sample JavaScript code for this conversion for the predefined RGB color spaces, in [§ 19 Sample code for Color Conversions](#color-conversion-code).

### <a id="alpha"></a>10.13.  Simple Alpha Compositing

When drawing, implementations must handle alpha according to the rules in [Section 5.1 Simple alpha compositing](https://www.w3.org/TR/compositing-1/#simplealphacompositing) of [\[Compositing\]](#biblio-compositing).

## <a id="color-conversion"></a>11.  Converting Colors

Tests

This section provides algorithms used later, it does not need tests.

------------------------------------------------------------------------

Colors may be converted from one color space to another and, provided that there is no gamut mapping and that each color space can represent out of gamut colors, (for RGB spaces, this means that the transfer function is defined over the extended range) then (subject to numerical precision and round-off error) the two colors will look the same and represent the same color sensation.

To <a id="prepare-a-color-col1-for-conversion"></a>prepare a color <var>col1</var> for conversion:

1.  <a id="ref-for-missing-color-component⑦"></a>

    <a id="ref-for-powerless-color-component①③"></a>

    <a id="powerless-to-missing"></a>Change any [powerless component](#powerless-color-component)s in <var>src</var> to [missing component](#missing-color-component)s

2.  <a id="ref-for-rectangular-orthogonal-color①"></a>

    <a id="ref-for-cylindrical-polar-color②"></a>

    <a id="convert-polrect"></a>If <var>src</var> is in a [cylindrical polar color](#cylindrical-polar-color) representation, convert <var>col1</var> to the corresponding [rectangular orthogonal color](#rectangular-orthogonal-color) representation and let this be the new <var>col1</var>.

To <a id="convert-a-color"></a>convert a color <var>col1</var> in a source color space <var>src</var> with white point <var>src-white</var> to a color <var>col2</var> in destination color space <var>dest</var> with white point <var>dest-white</var>, where <var>src</var> and <var>dest</var> are <em>different</em>:

1.  prepare <var>col1</var> for conversion

2.  <a id="ref-for-missing-color-component⑧"></a>

    <a id="convert-missing"></a>Replace any [missing component](#missing-color-component) with zero.

3.  <a id="convert-tolinear"></a>If <var>src</var> is not a linear-light representation, convert it to linear light (undo gamma-encoding) and let this be the new <var>col1</var>.

4.  <a id="convert-toXYZ"></a>Convert <var>col1</var> to CIE XYZ with a given whitepoint <var>src-white</var> and let this be <var>xyz</var>.

5.  <a id="ref-for-chromatic-adaptation-transform"></a>

    <a id="convert-CAT"></a>If <var>dest-white</var> is not the same as <var>src-white</var>, chromatically adapt <var>xyz</var> to <var>dest-white</var> using a linear Bradford [chromatic adaptation transform](#chromatic-adaptation-transform), and let this be the new <var>xyz</var>.

6.  <a id="ref-for-rectangular-orthogonal-color②"></a>

    <a id="ref-for-cylindrical-polar-color③"></a>

    <a id="convert-destpolar"></a>If <var>dest</var> is a [cylindrical polar color](#cylindrical-polar-color) representation, let <var>dest-rect</var> be the corresponding [rectangular orthogonal color](#rectangular-orthogonal-color) representation. Otherwise, let <var>dest-rect</var> be <var>dest</var>.

7.  <a id="convert-fromXYZ"></a>Convert <var>xyz</var> to <var>dest</var>, followed by applying any transfer function (gamma encoding), producing <var>col2</var>.

8.  <a id="ref-for-can-be-displayed"></a>

    <a id="ref-for-css-gamut-mapped③"></a>

    <a id="convert-display"></a>If <var>dest</var> is a physical output color space, such as a display, then <var>col2</var> must be [css gamut mapped](#css-gamut-mapped) so that it [can be displayed](#can-be-displayed).

9.  <a id="ref-for-missing-color-component⑨"></a>

    <a id="ref-for-cylindrical-polar-color④"></a>

    <a id="convert-rectpol"></a>If <var>dest-rect</var> is not the same as <var>dest</var>, in other words <var>dest</var> is a [cylindrical polar color](#cylindrical-polar-color) representation, convert from <var>dest-rect</var> to <var>dest</var>, and let this be <var>col2</var>. This may produce [missing component](#missing-color-component)s.

<a id="ref-for-typedef-color①⑧"></a>

## <a id="comparing-color-values"></a>12.  Comparing [\<color\>](#typedef-color) Values

<a id="ref-for-typedef-color①⑨"></a>

Two [\<color\>](#typedef-color) values are <a id="equivalent-colors"></a>equivalent colors when they compare as equal using the algorithm below. This comparison is used, for example, by style() container queries [\[CSS-CONDITIONAL-5\]](#biblio-css-conditional-5) and by CSS Transitions [\[CSS-TRANSITIONS-1\]](#biblio-css-transitions-1) to determine whether a color value has changed.

<a id="ref-for-typedef-color②⓪"></a>

<a id="ref-for-equivalent-colors"></a>

Given two [\<color\>](#typedef-color) values <var>C1</var> and <var>C2</var>, they are [equivalent colors](#equivalent-colors) if and only if the following algorithm returns true:

1.  <a id="ref-for-missing-color-component①⓪"></a>

    <a id="ref-for-powerless-color-component①④"></a>

    For each of <var>C1</var> and <var>C2</var>, convert any [powerless component](#powerless-color-component)s to [missing component](#missing-color-component)s.

2.  <a id="ref-for-typedef-color-space"></a>

    If <var>C1</var> and <var>C2</var> share the same [\<color-space\>](#typedef-color-space):

    1.  <a id="ref-for-missing-color-component①①"></a>

        Compare their components one by one, including the alpha channel. A [missing component](#missing-color-component) is only equal to another <a id="ref-for-missing-color-component①②"></a>missing component. Two numeric components are considered equal if they differ by no more than a small implementation-defined ε.

    2.  Return true if and only if all components compare as equal.

3.  <a id="ref-for-missing-color-component①③"></a>

    <a id="ref-for-typedef-color-space①"></a>

    Otherwise, <var>C1</var> and <var>C2</var> are in different [\<color-space\>](#typedef-color-space)s. If either color has at least one [missing component](#missing-color-component), return false.

4.  <a id="ref-for-valdef-oklab-oklab①"></a>

    <a id="ref-for-missing-color-component①④"></a>

    Otherwise, neither color has any [missing component](#missing-color-component). Convert both <var>C1</var> and <var>C2</var> to [oklab](#valdef-oklab-oklab), then return true if and only if all components (including alpha) of the converted colors compare as equal, using a standardized Oklab ε of <strong>0.00001</strong>.

<a id="ref-for-typedef-color-space②"></a>

<a id="ref-for-valdef-color-red①"></a>

<a id="ref-for-equivalent-colors①"></a>

<a id="ref-for-valdef-oklab-oklab②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Two colors that are expressed in different [\<color-space\>](#typedef-color-space)s but are colorimetrically identical—for example, [red](#valdef-color-red) and color(srgb 1 0 0)—are [equivalent colors](#equivalent-colors) by step 4 of this algorithm, since they convert to the same [oklab](#valdef-oklab-oklab) value.

<a id="ref-for-funcdef-rgb①③"></a>

<a id="ref-for-funcdef-rgba⑨"></a>

<a id="ref-for-funcdef-hsl①⑥"></a>

<a id="ref-for-funcdef-hsla①①"></a>

<a id="ref-for-funcdef-hwb⑨"></a>

<a id="ref-for-hex-color③"></a>

<a id="ref-for-named-color②"></a>

<a id="ref-for-css-system-colors①"></a>

<a id="ref-for-valdef-color-srgb⑦"></a>

<a id="ref-for-typedef-color-space③"></a>

For the purposes of this comparison, [rgb()](#funcdef-rgb), [rgba()](#funcdef-rgba), [hsl()](#funcdef-hsl), [hsla()](#funcdef-hsla), [hwb()](#funcdef-hwb), [hex colors](#hex-color), [named colors](#named-color), and [system colors](#css-system-colors) are all considered to be in the [srgb](#valdef-color-srgb) [\<color-space\>](#typedef-color-space).

Tests

- [query-style-color.html](https://wpt.fyi/results/css/css-conditional/container-queries/query-style-color.html) [(live test)](http://wpt.live/css/css-conditional/container-queries/query-style-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-conditional/container-queries/query-style-color.html)

## <a id="interpolation"></a>13.  Color Interpolation

Color interpolation happens with gradients, compositing, filters, transitions, animations, and color mixing and color modification functions.

<a id="ref-for-typedef-color②①"></a>

Interpolation between two [\<color\>](#typedef-color) values takes place by executing the following steps:

1.  <a id="ref-for-analogous-set"></a>

    <a id="ref-for-analogous-components"></a>

    checking the two colors for [analogous components](#analogous-components) and [analogous sets](#analogous-set) which will be <a id="carried-forward"></a>carried forward

2.  <a id="ref-for-missing-color-component①⑤"></a>

    <a id="ref-for-powerless-color-component①⑤"></a>

    prepare both colors for conversion. this changes any [powerless](#powerless-color-component) components to [missing](#missing-color-component) values

3.  converting them both to a given color space which will be referred to as the <a id="interpolation-color-space"></a>interpolation color space below.

4.  <a id="ref-for-carried-forward"></a>

    (if required) re-inserting [carried forward](#carried-forward) values in the converted colors

5.  <a id="ref-for-typedef-hue-interpolation-method"></a>

    (if required) fixing up the hues, depending on the selected [\<hue-interpolation-method\>](#typedef-hue-interpolation-method)

6.  <a id="ref-for-premultiplied"></a>

    changing the color components to [premultiplied](#premultiplied) form

7.  linearly interpolating each component of the computed value of the color separately

8.  <a id="ref-for-premultiplied①"></a>

    undoing [premultiplication](#premultiplied)

<a id="ref-for-valdef-color-currentcolor①⓪"></a>

Interpolating to or from [currentcolor](#valdef-color-currentcolor) is possible. The numerical value used for this purpose is the used value.

### <a id="interpolation-space"></a>13.1.  Color Space for Interpolation

Various features in CSS depend on interpolating colors.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-interpolating-specs"></a> Examples include:
>
> - <a id="ref-for-typedef-gradient"></a>
>
>   [\<gradient\>](https://www.w3.org/TR/css-images-4/#typedef-gradient)
>
> - <a id="ref-for-propdef-filter"></a>
>
>   [filter](https://www.w3.org/TR/filter-effects-1/#propdef-filter)
>
> - <a id="ref-for-propdef-animation"></a>
>
>   [animation](https://www.w3.org/TR/css-animations-1/#propdef-animation)
>
> - <a id="ref-for-propdef-transition"></a>
>
>   [transition](https://www.w3.org/TR/css-transitions-1/#propdef-transition)
>
> - <a id="ref-for-funcdef-color-mix①"></a>
>
>   [color-mix()](https://www.w3.org/TR/css-color-5/#funcdef-color-mix)
>
> - <a id="ref-for-relative-color"></a>
>
>   [relative color](https://www.w3.org/TR/css-color-5/#relative-color) syntax

<a id="ref-for-interpolation-color-space"></a>

Mixing or otherwise combining colors has different results depending on the [interpolation color space](#interpolation-color-space) used. Thus, different color spaces may be more appropriate for each interpolation use case.

- <a id="ref-for-valdef-color-xyz④"></a>

  <a id="ref-for-valdef-color-srgb-linear④"></a>

  In some cases, the result of physically mixing two colored lights is desired. In that case, the CIE [XYZ](#valdef-color-xyz), display-p3-linear or [srgb-linear](#valdef-color-srgb-linear) color spaces are appropriate, because they are linear in light intensity.

- <a id="ref-for-valdef-oklab-oklab③"></a>

  <a id="ref-for-valdef-lab-lab①"></a>

  If colors need to be evenly spaced perceptually (such as in a gradient), the [Oklab](#valdef-oklab-oklab) color space (and the older [Lab](#valdef-lab-lab)), are designed to be perceptually uniform.

- <a id="ref-for-valdef-oklch-oklch①"></a>

  <a id="ref-for-valdef-lch-lch①"></a>

  If avoiding graying out in color mixing is desired, i.e. maximizing chroma throughout the transition, [OkLCh](#valdef-oklch-oklch) (and the older [LCH](#valdef-lch-lch)) work well for that.

- <a id="ref-for-valdef-color-srgb⑧"></a>

  Lastly, compatibility with legacy Web content may be the most important consideration. The [sRGB](#valdef-color-srgb) color space, which is neither linear-light nor perceptually uniform, is the choice here, even though it produces poorer results (overly dark or greyish mixes).

These features are collectively termed the <a id="host-syntax"></a>host syntax.

<a id="ref-for-interpolation-color-space①"></a>

<a id="ref-for-color-interpolation-method"></a>

To permit a host syntax to indicate the [interpolation color space](#interpolation-color-space), this specification exports a [color-interpolation-method](#color-interpolation-method) production. It is not used by this specification itself, only exposed so that other specifications can use it; see e.g. use in [CSS Images 4 § 3.1 Linear Gradients: the linear-gradient() notation](https://www.w3.org/TR/css-images-4/#linear-gradients).

<a id="ref-for-interpolation-color-space②"></a>

<a id="ref-for-color-interpolation-method①"></a>

The host syntax should define what the <em>default</em> [interpolation color space](#interpolation-color-space) should be for each case, and preferably provide syntax for authors to override this default. If such syntax is part of a property value, it should use the [color-interpolation-method](#color-interpolation-method) production, defined below for easy reference from other specifications. This ensures consistency across CSS, and that further customizations on how color interpolation is performed can automatically percolate across all of CSS.

<a id="typedef-color-space"></a>

<a id="ref-for-comb-one⑧⑦"></a>

<a id="typedef-rectangular-color-space"></a>

<a id="ref-for-valdef-color-srgb⑨"></a>

<a id="ref-for-comb-one⑧⑧"></a>

<a id="ref-for-valdef-color-srgb-linear⑤"></a>

<a id="ref-for-comb-one⑧⑨"></a>

<a id="ref-for-valdef-color-display-p3①⓪"></a>

<a id="ref-for-comb-one⑨⓪"></a>

<a id="ref-for-comb-one⑨①"></a>

<a id="ref-for-valdef-color-a98-rgb④"></a>

<a id="ref-for-comb-one⑨②"></a>

<a id="ref-for-valdef-color-prophoto-rgb①①"></a>

<a id="ref-for-comb-one⑨③"></a>

<a id="ref-for-valdef-color-rec2020⑥"></a>

<a id="ref-for-comb-one⑨④"></a>

<a id="ref-for-valdef-lab-lab②"></a>

<a id="ref-for-comb-one⑨⑤"></a>

<a id="ref-for-valdef-oklab-oklab④"></a>

<a id="ref-for-comb-one⑨⑥"></a>

<a id="ref-for-typedef-xyz-space①"></a>

<a id="typedef-polar-color-space"></a>

<a id="ref-for-valdef-hsl-hsl②"></a>

<a id="ref-for-comb-one⑨⑦"></a>

<a id="ref-for-valdef-hwb-hwb①"></a>

<a id="ref-for-comb-one⑨⑧"></a>

<a id="ref-for-valdef-lch-lch②"></a>

<a id="ref-for-comb-one⑨⑨"></a>

<a id="ref-for-valdef-oklch-oklch②"></a>

<a id="typedef-hue-interpolation-method"></a>

<a id="ref-for-comb-one①⓪⓪"></a>

<a id="ref-for-comb-one①⓪①"></a>

<a id="ref-for-comb-one①⓪②"></a>

<a id="color-interpolation-method"></a>

<a id="ref-for-typedef-rectangular-color-space"></a>

<a id="ref-for-comb-one①⓪③"></a>

<a id="ref-for-typedef-polar-color-space"></a>

<a id="ref-for-typedef-hue-interpolation-method①"></a>

<a id="ref-for-mult-opt①⑥"></a>

```text
<color-space> = <rectangular-color-space> | <polar-color-space>
<rectangular-color-space> = srgb | srgb-linear | display-p3 | display-p3-linear | a98-rgb | prophoto-rgb | rec2020 | lab | oklab | <xyz-space>
<polar-color-space> = hsl | hwb | lch | oklch
<hue-interpolation-method> = [ shorter | longer | increasing | decreasing ] hue
<color-interpolation-method> = in [ <rectangular-color-space> | <polar-color-space> <hue-interpolation-method>? ]
```
<a id="ref-for-typedef-rectangular-color-space①"></a>

<a id="ref-for-typedef-polar-color-space①"></a>

<a id="ref-for-typedef-ident④"></a>

<a id="ref-for-funcdef-color⑦"></a>

The keywords in the definitions of [\<rectangular-color-space\>](#typedef-rectangular-color-space) and [\<polar-color-space\>](#typedef-polar-color-space) each refer to their corresponding color space, represented in CSS either by the functional syntax with the same name, or (if no such function is present), by the corresponding [\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident) in the [color()](#funcdef-color) function.

Tests

- [color-mix-percents-01.html](https://wpt.fyi/results/css/css-color/color-mix-percents-01.html) [(live test)](http://wpt.live/css/css-color/color-mix-percents-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-percents-01.html)
- [color-mix-percents-02.html](https://wpt.fyi/results/css/css-color/color-mix-percents-02.html) [(live test)](http://wpt.live/css/css-color/color-mix-percents-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-percents-02.html)

<!-- -->

- [gradients-with-transparent.html](https://wpt.fyi/results/css/css-images/gradients-with-transparent.html) [(live test)](http://wpt.live/css/css-images/gradients-with-transparent.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradients-with-transparent.html)
- [gradient-eval-001.html](https://wpt.fyi/results/css/css-images/gradient/gradient-eval-001.html) [(live test)](http://wpt.live/css/css-images/gradient/gradient-eval-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/gradient-eval-001.html)
- [gradient-eval-002.html](https://wpt.fyi/results/css/css-images/gradient/gradient-eval-002.html) [(live test)](http://wpt.live/css/css-images/gradient/gradient-eval-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/gradient-eval-002.html)
- [gradient-eval-003.html](https://wpt.fyi/results/css/css-images/gradient/gradient-eval-003.html) [(live test)](http://wpt.live/css/css-images/gradient/gradient-eval-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/gradient-eval-003.html)
- [gradient-eval-004.html](https://wpt.fyi/results/css/css-images/gradient/gradient-eval-004.html) [(live test)](http://wpt.live/css/css-images/gradient/gradient-eval-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/gradient-eval-004.html)
- [gradient-eval-005.html](https://wpt.fyi/results/css/css-images/gradient/gradient-eval-005.html) [(live test)](http://wpt.live/css/css-images/gradient/gradient-eval-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/gradient-eval-005.html)
- [gradient-eval-006.html](https://wpt.fyi/results/css/css-images/gradient/gradient-eval-006.html) [(live test)](http://wpt.live/css/css-images/gradient/gradient-eval-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/gradient-eval-006.html)
- [gradient-eval-007.html](https://wpt.fyi/results/css/css-images/gradient/gradient-eval-007.html) [(live test)](http://wpt.live/css/css-images/gradient/gradient-eval-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/gradient-eval-007.html)
- [gradient-eval-008.html](https://wpt.fyi/results/css/css-images/gradient/gradient-eval-008.html) [(live test)](http://wpt.live/css/css-images/gradient/gradient-eval-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/gradient-eval-008.html)
- [gradient-eval-009.html](https://wpt.fyi/results/css/css-images/gradient/gradient-eval-009.html) [(live test)](http://wpt.live/css/css-images/gradient/gradient-eval-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/gradient-eval-009.html)
- [gradient-none-interpolation.html](https://wpt.fyi/results/css/css-images/gradient/gradient-none-interpolation.html) [(live test)](http://wpt.live/css/css-images/gradient/gradient-none-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/gradient-none-interpolation.html)
- [oklab-gradient.html](https://wpt.fyi/results/css/css-images/gradient/oklab-gradient.html) [(live test)](http://wpt.live/css/css-images/gradient/oklab-gradient.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/oklab-gradient.html)
- [srgb-gradient.html](https://wpt.fyi/results/css/css-images/gradient/srgb-gradient.html) [(live test)](http://wpt.live/css/css-images/gradient/srgb-gradient.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/srgb-gradient.html)
- [srgb-linear-gradient.html](https://wpt.fyi/results/css/css-images/gradient/srgb-linear-gradient.html) [(live test)](http://wpt.live/css/css-images/gradient/srgb-linear-gradient.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/srgb-linear-gradient.html)
- [xyz-gradient.html](https://wpt.fyi/results/css/css-images/gradient/xyz-gradient.html) [(live test)](http://wpt.live/css/css-images/gradient/xyz-gradient.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/xyz-gradient.html)
- [gradient-interpolation-method-valid.html](https://wpt.fyi/results/css/css-images/parsing/gradient-interpolation-method-valid.html) [(live test)](http://wpt.live/css/css-images/parsing/gradient-interpolation-method-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/parsing/gradient-interpolation-method-valid.html)
- [gradient-interpolation-method-invalid.html](https://wpt.fyi/results/css/css-images/parsing/gradient-interpolation-method-invalid.html) [(live test)](http://wpt.live/css/css-images/parsing/gradient-interpolation-method-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/parsing/gradient-interpolation-method-invalid.html)
- [gradient-interpolation-method-computed.html](https://wpt.fyi/results/css/css-images/parsing/gradient-interpolation-method-computed.html) [(live test)](http://wpt.live/css/css-images/parsing/gradient-interpolation-method-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/parsing/gradient-interpolation-method-computed.html)

If the host syntax does not define what color space interpolation should take place in, it defaults to Oklab.

<a id="ref-for-typedef-polar-color-space②"></a>

<a id="ref-for-typedef-hue-interpolation-method②"></a>

<a id="ref-for-shorter"></a>

For a [\<polar-color-space\>](#typedef-polar-color-space) if the [\<hue-interpolation-method\>](#typedef-hue-interpolation-method) is not specified, it defaults to [shorter](#shorter).

<a id="ref-for-funcdef-rgb①④"></a>

<a id="ref-for-funcdef-hsl①⑦"></a>

<a id="ref-for-funcdef-hwb①⓪"></a>

However, user agents <em>must</em> handle interpolation between legacy sRGB color formats (hex colors, named colors, [rgb()](#funcdef-rgb), [hsl()](#funcdef-hsl) or [hwb()](#funcdef-hwb) and the equivalent alpha-including forms) in gamma-encoded sRGB space. This provides Web compatibility; legacy sRGB content interpolates in the sRGB space by default.

Tests

- [legacy-color-gradient.html](https://wpt.fyi/results/css/css-images/gradient/legacy-color-gradient.html) [(live test)](http://wpt.live/css/css-images/gradient/legacy-color-gradient.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/legacy-color-gradient.html)

<a id="ref-for-interpolation-color-space③"></a>

This also means that authors can choose to opt-in to better interpolation, even between sRGB colors, by using the non-legacy color(srgb r g b) form for at least one of their colors, or by explicitly specifying an [interpolation color space](#interpolation-color-space).

Tests

- [css-color-4-colors-default-to-oklab-gradient.html](https://wpt.fyi/results/css/css-images/gradient/css-color-4-colors-default-to-oklab-gradient.html) [(live test)](http://wpt.live/css/css-images/gradient/css-color-4-colors-default-to-oklab-gradient.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/css-color-4-colors-default-to-oklab-gradient.html)

<a id="ref-for-interpolation-color-space④"></a>

If the colors to be interpolated are outside the gamut of the [interpolation color space](#interpolation-color-space) , then once converted to that space, they will contain out of range values.

These are not clipped; the values must be interpolated as-is.

### <a id="interpolation-missing"></a>13.2.  Interpolating with Missing Components

<a id="ref-for-interpolation-color-space⑤"></a>

<a id="ref-for-missing-color-component①⑥"></a>

In the course of converting the two colors to the [interpolation color space](#interpolation-color-space), any [missing components](#missing-color-component) would be replaced with the value 0.

<a id="ref-for-missing-color-component①⑦"></a>

<a id="ref-for-interpolation-color-space⑥"></a>

<a id="ref-for-analogous-components①"></a>

Thus, the first stage in interpolating two colors is to classify any [missing components](#missing-color-component) in the input colors, and compare them to the components of the [interpolation color space](#interpolation-color-space). If any [analogous components](#analogous-components) which are <a id="ref-for-missing-color-component①⑧"></a>missing components are found, they will be <strong>carried forward</strong> and re-inserted in the converted color before premultiplication, and before linear interpolation takes place.

<a id="ref-for-analogous-set①"></a>

<a id="ref-for-missing-color-component①⑨"></a>

<a id="ref-for-carried-forward①"></a>

<a id="ref-for-interpolation-color-space⑦"></a>

Similarly, if every component of an [analogous set](#analogous-set) (defined below) in the original color is a [missing component](#missing-color-component), they are all [carried forward](#carried-forward) and re-inserted in the corresponding <a id="ref-for-analogous-set②"></a>analogous set of the [interpolation color space](#interpolation-color-space).

<a id="ref-for-missing-color-component②⓪"></a>

<a id="ref-for-carried-forward②"></a>

Alpha is its own analogous component (alpha is analogous to alpha), so a [missing](#missing-color-component) alpha is [carried forward](#carried-forward) in exactly the same way as any other <a id="ref-for-missing-color-component②①"></a>missing component.

The <a id="analogous-components"></a>analogous components are as follows:



| Category     | Components |
|--------------|------------|
| Reds         | r,x        |
| Greens       | g,y        |
| Blues        | b,z        |
| Lightness    | L          |
| Colorfulness | C, S       |
| Hue          | H          |
| Opponent a   | a          |
| Opponent b   | b          |
| Alpha        | alpha      |



> <strong data-conversion-semantic="note">Note</strong>
>
> Note: for the purposes of this classification, the XYZ spaces are considered super-saturated RGB spaces. Also, despite Saturation being Lightness-dependent, it falls in the same category as Chroma here. The Whiteness and Blackness components of HWB have no analogs in other color spaces.

<a id="ref-for-analogous-components②"></a>

Additionally, for any two color spaces, the components that remain after removing all [analogous components](#analogous-components) form an <a id="analogous-set"></a>analogous set of components.

<a id="ref-for-analogous-set③"></a>

<a id="ref-for-analogous-components③"></a>

<a id="ref-for-missing-color-component②②"></a>

<a id="ref-for-interpolation-color-space⑧"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because the full set of all color components is the [analogous set](#analogous-set) that remains when there are no individual [analogous components](#analogous-components), a color with all color components [missing](#missing-color-component) will have all color components <a id="ref-for-missing-color-component②③"></a>missing in the [interpolation color space](#interpolation-color-space) as well.

<a id="ref-for-analogous-components④"></a>

<a id="ref-for-valdef-oklab-a"></a>

<a id="ref-for-valdef-oklab-b"></a>

<a id="ref-for-valdef-oklch-c"></a>

<a id="ref-for-valdef-oklch-h"></a>

<a id="ref-for-analogous-set④"></a>

<a id="ref-for-missing-color-component②④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-analogous-set"></a> When converting `lab(50% none none)` to LCH for interpolation, Lightness is individually [analogous](#analogous-components). The remaining components ([a](https://www.w3.org/TR/css-color-5/#valdef-oklab-a) and [b](https://www.w3.org/TR/css-color-5/#valdef-oklab-b) in Lab; [C](https://www.w3.org/TR/css-color-5/#valdef-oklch-c) and [H](https://www.w3.org/TR/css-color-5/#valdef-oklch-h) in LCH) form an [analogous set](#analogous-set). Since both <a id="ref-for-valdef-oklab-a①"></a>a and <a id="ref-for-valdef-oklab-b①"></a>b are [missing](#missing-color-component), both <a id="ref-for-valdef-oklch-c①"></a>C and <a id="ref-for-valdef-oklch-h①"></a>H are carried forward as <a id="ref-for-missing-color-component②⑤"></a>missing, giving `lch(50% none none)` rather than `lch(50% 0 0)`.
>
> <a id="ref-for-analogous-set⑤"></a>
>
> <a id="ref-for-analogous-components⑤"></a>
>
> Similarly, `rgb(none none none / 50%)` converted to OKLab for interpolation yields `oklab(none none none / 50%)`, because the three color components form the [analogous set](#analogous-set) (there are no individual [analogous components](#analogous-components) between sRGB and OKLab).

Tests

- [gradient-none-interpolation.html](https://wpt.fyi/results/css/css-images/gradient/gradient-none-interpolation.html) [(live test)](http://wpt.live/css/css-images/gradient/gradient-none-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-images/gradient/gradient-none-interpolation.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-analogous-hue"></a> For example, if these two colors are to be interpolated in OkLCh, the missing hue in the CIE LCH color is analogous to the hue component of OkLCh and will be carried forward while the missing blue component in the second color is not analogous to any OkLCh component and will not be carried forward:
>
> ```text
>  lch(50% 0.02 none)
>  color(display-p3 0.7 0.5 none)
> ```
>
> which convert to
>
> ```text
>  oklch(56.897% 0.0001 0)
>  oklch(63.612% 0.1522 78.748)
> ```
>
> <a id="ref-for-missing-color-component②⑥"></a>
>
> and with carried forward [missing component](#missing-color-component) re-inserted, the two colors to be interpolated are:
>
> ```text
>  oklch(56.897% 0.0001 none)
>  oklch(63.612% 0.1522 78.748)
> ```
<a id="ref-for-missing-color-component②⑦"></a>

If a color with a carried forward [missing component](#missing-color-component) is interpolated with another color which is not missing that component, the <a id="ref-for-missing-color-component②⑧"></a>missing component is treated as having the <em>other color’s</em> component value.

<a id="ref-for-powerless-color-component①⑥"></a>

Therefore, the carrying-forward step must be performed <em>before</em> any [powerless component](#powerless-color-component) handling.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-oklch-missing-hue"></a> For example, if these two colors are interpolated, the second of which has a missing hue:
>
> ```text
>  oklch(78.3% 0.108 326.5)
>  oklch(39.2% 0.4 none)
> ```
>
> Then the actual colors to be interpolated are
>
> ```text
>  oklch(78.3% 0.108 326.5)
>  oklch(39.2% 0.4 326.5)
> ```
>
> and not
>
> ```text
>  oklch(78.3% 0.108 326.5)
>  oklch(39.2% 0.4 0)
> ```
<a id="ref-for-missing-color-component②⑨"></a>

<a id="ref-for-premultiplied②"></a>

If the carried forward [missing component](#missing-color-component) is alpha, the color must be [premultiplied](#premultiplied) with this carried forward value, not with the zero value that would have resulted from color conversion.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-oklch-missing-alpha"></a> For example, if these two colors are interpolated, the second of which has a missing alpha:
>
> ```text
>  oklch(0.783 0.108 326.5 / 0.5)
>  oklch(0.392 0.4 0 / none)
> ```
>
> Then the actual colors to be interpolated are
>
> ```text
>  oklch(78.3% 0.108 326.5 / 0.5)
>  oklch(39.2% 0.4 0 / 0.5)
> ```
>
> giving the premultiplied OkLCh values \[0.3915, 0.054, 326\] and \[0.196, 0.2, 0\].

<a id="ref-for-missing-color-component③⓪"></a>

If both colors are [missing](#missing-color-component) a given component, the interpolated color will also be <a id="ref-for-missing-color-component③①"></a>missing that component.

### <a id="interpolation-alpha"></a>13.3.  Interpolating with Alpha

When the colors to be interpolated are not fully opaque, they are first <a id="premultiplied"></a>premultiplied as follows:

- <a id="ref-for-valdef-color-none⑤"></a>

  If the alpha value is [none](#valdef-color-none), the premultiplied value is the un-premultiplied value. Otherwise,

- <a id="ref-for-valdef-color-none⑥"></a>

  If any component value is [none](#valdef-color-none), the premultiplied value is also <a id="ref-for-valdef-color-none⑦"></a>none.

- <a id="ref-for-rectangular-orthogonal-color③"></a>

  For [rectangular orthogonal color](#rectangular-orthogonal-color) coordinate systems, all component values are multiplied by the alpha value.

- <a id="ref-for-cylindrical-polar-color⑤"></a>

  For [cylindrical polar color](#cylindrical-polar-color) coordinate systems, the hue angle is <em>not</em> premultiplied, but the other two axes <em>are</em> premultiplied.

To obtain a color value from a premultiplied color value,

- <a id="ref-for-valdef-color-none⑧"></a>

  If the interpolated alpha value is zero or [none](#valdef-color-none), the un-premultiplied value is the premultiplied value. Otherwise,

- <a id="ref-for-valdef-color-none⑨"></a>

  If any component value is [none](#valdef-color-none), the un-premultiplied value is also <a id="ref-for-valdef-color-none①⓪"></a>none.

- otherwise, each component which had been premultiplied is divided by the interpolated alpha value.

Tests

- [color-transition-premultiplied.html](https://wpt.fyi/results/css/css-transitions/animations/color-transition-premultiplied.html) [(live test)](http://wpt.live/css/css-transitions/animations/color-transition-premultiplied.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-transitions/animations/color-transition-premultiplied.html)

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="premultiplied-explainer"></a>
>
> Why is premultiplied alpha useful?
>
> Interpolating colors using the premultiplied representations tends to produce more attractive transitions than the non-premultiplied representations, particularly when transitioning from a fully opaque color to fully transparent.
>
> Note that transitions where either the transparency or the color are held constant (for example, transitioning between `rgba(255, 0, 0, 100%)` (opaque red) and `rgba(0,0,255,100%)` (opaque blue), or `rgba(255,0,0,100%)` (opaque red) and `rgba(255,0,0,0%)` (transparent red)) have identical results whether the color interpolation is done in premultiplied or non-premultiplied color-space. Differences only arise when <em>both</em> the color and transparency differ between the two endpoints.
>
> > <strong data-conversion-semantic="example">Example</strong>
> >
> > <a id="ex-gradient-transition-premultiply"></a> The following example illustrates the difference between a gradient transitioning via pre-multiplied values (in this case sRGB, since all colors involved are legacy colors) and one transitioning (incorrectly) via non-premultiplied values. In both of these examples, the gradient is drawn over a white background. Both gradients could be written with the following value:
> >
> > ```text
> > linear-gradient(90deg, red, transparent, blue)
> > ```
> >
> > With premultiplied colors, transitions to or from "transparent" always look nice:
> >
> > (Image requires SVG)
> >
> > ![(Image requires SVG)](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/gradient2.svg)
> >
> > On the other hand, if a gradient were to incorrectly transition in non-premultiplied space, the center of the gradient would be a noticeably grayish color, because "transparent" is actually a shorthand for rgba(0,0,0,0), or transparent black, meaning that the red transitions to a black as it loses opacity, and similarly with the blue’s transition:
> >
> > (Image requires SVG)
> >
> > ![(Image requires SVG)](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/gradient3.svg)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-premultiplied-srgb"></a> For example, to interpolate, in the sRGB color space, the two sRGB colors rgb(24% 12% 98% / 0.4) and rgb(62% 26% 64% / 0.6) they would first be converted to premultiplied form \[9.6% 4.8% 39.2% \] and \[37.2% 15.6% 38.4%\] before interpolation.
>
> The midpoint of linearly interpolating these colors would be \[23.4% 10.2% 38.8%\] which, with an alpha value of 0.5, is rgb(46.8% 20.4% 77.6% / 0.5) when premultiplication is undone.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-premultiplied-lab"></a> To interpolate, in the Lab color space, the two colors rgb(76% 62% 03% / 0.4) and color(display-p3 0.84 0.19 0.72 / 0.6) they are first converted to lab lab(66.927% 4.873 68.622 / 0.4) lab(53.503% 82.672 -33.901 / 0.6) then the L, a and b coordinates are premultiplied before interpolation \[26.771% 1.949 27.449\] and \[32.102% 49.603 -20.341\].
>
> The midpoint of linearly interpolating these would be \[29.4365% 25.776 3.554\] which, with an alpha value of 0.5, is lab(58.873% 51.552 7.108) / 0.5) when premultiplication is undone.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-premultiplied-lch"></a> To interpolate, in the chroma-preserving LCH color space, the same two colors rgb(76% 62% 03% / 0.4) and color(display-p3 0.84 0.19 0.72 / 0.6) they are first converted to LCH lch(66.93% 68.79 85.94 / 0.4) lch(53.5% 89.35 337.7 / 0.6) then the L and C coordinates (but not H) are premultiplied before interpolation \[26.771% 27.516 85.94\] and \[32.102% 53.61 337.7\].
>
> The midpoint of linearly interpolating these, along the shorter hue arc (the default) would be \[29.4365% 40.563 31.82\] which, with an alpha value of 0.5, is lch(58.873% 81.126 31.82) / 0.5) when premultiplication is undone.

There is sample JavaScript code for alpha premultiplication and un-premultiplication, for both polar and rectangular color spaces, in [§ 19 Sample code for Color Conversions](#color-conversion-code).

### <a id="hue-interpolation"></a>13.4.  Hue Interpolation

For color functions with a hue angle (LCH, HSL, HWB etc), there are multiple ways to interpolate. As arcs greater than 360° are rarely desirable, hue angles are fixed up prior to interpolation so that per-component interpolation is done over less than 360°, often less than 180°.

<a id="ref-for-color-interpolation-method②"></a>

<a id="ref-for-typedef-hue-interpolation-method③"></a>

Host syntax can specify any of the following algorithms for hue interpolation (angles in the following are in degrees, but the logic is the same regardless of how they are specified). Specifying a hue interpolation strategy is already part of the [\<color-interpolation-method\>](#color-interpolation-method) syntax via the [\<hue-interpolation-method\>](#typedef-hue-interpolation-method) token.

Unless otherwise specified, if no specific hue interpolation algorithm is selected by the host syntax, the default is shorter.

Tests

- [color-mix-percents-01.html](https://wpt.fyi/results/css/css-color/color-mix-percents-01.html) [(live test)](http://wpt.live/css/css-color/color-mix-percents-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-percents-01.html)
- [color-mix-percents-02.html](https://wpt.fyi/results/css/css-color/color-mix-percents-02.html) [(live test)](http://wpt.live/css/css-color/color-mix-percents-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-percents-02.html)

<a id="ref-for-powerless-color-component①⑦"></a>

<a id="ref-for-missing-color-component③②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As a reminder, if the interpolating colors were not already in the specified interpolation color space, then converting them will turn any [powerless components](#powerless-color-component) into [missing components](#missing-color-component).

#### <a id="hue-shorter"></a>13.4.1.  <a id="shorter"></a>shorter

Hue angles are interpolated to take the <em>shorter</em> of the two arcs between the starting and ending hues.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-shorter"></a> For example, the midpoint when interpolating in OkLCh from a red  oklch(0.6 0.24 30) to a yellow  oklch(0.8 0.15 90) would be at a hue angle of 30 + (90 - 30) \* 0.5 = 60 degrees, along the shorter arc between the two colors, giving a deep orange  oklch(0.7 0.195 60)

Angles are adjusted so that <var>θ₂ - θ₁</var> ∈ \[-180, 180\]. In pseudo-Javascript:

```text
if (θ₂ - θ₁ > 180) {
  θ₁ += 360;
}
else if (θ₂ - θ₁ < -180) {
  θ₂ += 360;
}
```
#### <a id="hue-longer"></a>13.4.2.  <a id="longer"></a>longer

Hue angles are interpolated to take the <em>longer</em> of the two arcs between the starting and ending hues.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-longer"></a> For example, the midpoint when interpolating in OkLCh from a red  oklch(0.6 0.24 30) to a yellow  oklch(0.8 0.15 90) would be at a hue angle of (30 + 360 + 90) \* 0.5 = 240 degrees, along the longer arc between the two colors, giving a sky blue  oklch(0.7 0.195 240)

Angles are adjusted so that <var>θ₂ - θ₁</var> ∈ {(-360, -180\], \[180, 360)}. In pseudo-Javascript:

```text
if (0 < θ₂ - θ₁ < 180) {
  θ₁ += 360;
}
else if (-180 < θ₂ - θ₁ <= 0) {
  θ₂ += 360;
}
```
#### <a id="hue-increasing"></a>13.4.3.  <a id="increasing"></a>increasing

Hue angles are interpolated so that, as they progress from the first color to the second, the angle is always <em>increasing</em>. If the angle increases to 360 it is reset to zero, and then continues increasing.

Depending on the difference between the two angles, this will either look the same as <em>shorter</em> or as <em>longer.</em> However, if one of the hue angles is being animated, and the hue angle difference passes through 180 degrees, the interpolation will not flip to the other arc.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-increasing"></a> For example, the midpoint when interpolating in OkLCh from a deep brown  oklch(0.5 0.1 30) to a turquoise  oklch(0.7 0.1 190) would be at a hue angle of (30 + 190) \* 0.5 = 110 degrees, giving a khaki  oklch(0.6 0.1 110).
>
> However, if the hue of the second color is animated to  oklch(0.7 0.1 230), the midpoint of the interpolation will be (30 + 230) \* 0.5 = 130 degrees, continuing in the same increasing direction, giving another green  oklch(0.6 0.1 130) rather than flipping to the opponent color part-way through the animation.

Angles are adjusted so that <var>θ₂ - θ₁</var> ∈ \[0, 360). In pseudo-Javascript:

```text
if (θ₂ < θ₁) {
  θ₂ += 360;
}
```
#### <a id="hue-decreasing"></a>13.4.4.  <a id="decreasing"></a>decreasing

Hue angles are interpolated so that, as they progress from the first color to the second, the angle is always <em>decreasing</em>. If the angle decreases to 0 it is reset to 360, and then continues decreasing.

Depending on the difference between the two angles, this will either look the same as <em>shorter</em> or as <em>longer.</em> However, if one of the hue angles is being animated, and the hue angle difference passes through 180 degrees, the interpolation will not flip to the other arc.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-decreasing"></a> For example, the midpoint when interpolating in OkLCh from a deep brown  oklch(0.5 0.1 30) to a turquoise  oklch(0.7 0.1 190) would be at a hue angle of (30 + 360 + 190) \* 0.5 = 290 degrees, giving a purple  oklch(0.6 0.1 290).
>
> However, if the hue of the second color is animated to  oklch(0.7 0.1 230), the midpoint of the interpolation will be (30 + 360 + 230) \* 0.5 = 310 degrees, continuing in the same decreasing direction, giving another purple  oklch(0.6 0.1 310) rather than flipping to the opponent color part-way through the animation.

Angles are adjusted so that <var>θ₂ - θ₁</var> ∈ (-360, 0\]. In pseudo-Javascript:

```text
if (θ₁ < θ₂) {
  θ₁ += 360;
}
```
## <a id="gamut-mapping"></a>14.  Gamut Mapping 

### <a id="gamut-mapping-intro"></a>14.1.  An Introduction to Gamut Mapping

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This section provides important context for the specific requirements described elsewhere in the document.

<i>This section is non-normative</i>

Tests

This section is not normative, it does not need tests.

------------------------------------------------------------------------

When a color in an origin color space is converted to another, destination color space which has a smaller gamut, some colors will be outside the destination gamut.

For intermediate color calculations, these out of gamut values are preserved. However, if the destination is the display device (a screen, or a printer) then out of gamut values must be converted to an in-gamut color.

Gamut mapping is the process of finding an in-gamut color with the least objectionable change in visual appearance.

Some out of gamut colors correspond to real-world colors (they could be physically reproduced) while others are imaginary colors (they lie outside the spectral locus and thus would need more than 100% of a single wavelength) and could never be physically realized. Those colors tend to come from calculations such as "make this color 100x as saturated".

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-lab-corners"></a> For example, while theoretically unbounded, the CIE Lab <em>(a,b)</em> plane is often implemented such that <em>a</em> and <em>b</em> are constrained to the range ±127.
>
> Three of those four corners are outside the spectral locus and thus correspond to imaginary colors.
>
> <a id="fig-lab-corners"></a>
>
> ![Embedded object resource](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/UCS-rec2020-labcorners.svg)
>
> The four corners of the ±127 (a,b) plane, on a UCS chromaticity diagram. The outer shape is the spectral locus. The larger triangle is the rec2020 gamut, while the smaller is sRGB.

#### <a id="GM-clip"></a>14.1.1.  Clipping

The simplest and least acceptable method is simply to clip the component values to the displayable range.

Since the motivation for this method is speed, clipping is commonly done on gamma-encoded values rather than converting them to linear-light.

This changes the proportions of the three primary colors (for an RGB display), resulting in a hue shift.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-gamut-clip-hue-shift"></a> For example, consider the color `color(srgb-linear 0.5 1 3)`. Because this is a linear-light color space, we can compare the intensities of the three components and see that the amount of blue light is three times the amount of green, while the amount of red light is half that of green. There is six times as much blue primary as red. In OkLCh, this color has a hue angle of 265.1°
>
> If we now clip this color to bring it into gamut for sRGB, we get `color(srgb-linear 0.5 1 1)`. The amount of blue light is the same as green. In OkLCh, this color has a hue angle of 196.1°, a substantial change of 69°.

When colors are not too far out of gamut, clipping can give acceptable results. This is particularly true for darker (or negative) component values.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-gamut-clip-good"></a> For example, consider the color `color(rec2020 0.54 0.9 0)` which is `oklch(80.72% 0.3296 141.6)`.
>
> Converting to p3 colorspace, the negative blue value shows the color is out of gamut: `color(display-p3 0.3265 0.9165 -0.1262)` which in linear-light is `color(display-p3-linear 0.0871 0.8205 -0.0146)`.
>
> Clipping the gamma-encoded p3 color to the p3 gamut gives `color(display-p3 0.3265 0.9165 0)` which in linear-light is `color(display-p3-linear 0.0871 0.8205 0)` which, for comparison, is `oklch(80.79% 0.3221 142.3)`.
>
> This is a good result; the hue angle and lightness have barely changed but the chroma is somewhat reduced, as expected.
>
> In terms of percentages of linear-light red green and blue, the red and green are identical while the blue is -1.46% higher.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-gamut-clip-acceptable"></a> For example, consider the color `color(prophoto-rgb 0.2 1.0 0.1)` which is `oklch(85.07% 0.4873 151.4)`
>
> Converting to p3 colorspace, the color is significantly out of gamut: `color(display-p3 -0.5782 1.067 -0.2363)` which in linear-light is `color(display-p3-linear -0.2937 1.158 -0.0456)`.
>
> Clipping the gamma-encoded p3 color to the p3 gamut gives `color(display-p3 0 1 0)` which in linear-light is, again, `color(display-p3-linear 0 1 0)` (component values of exactly 0 or 1 are unaffected by gamma encoding) which, for comparison, is `oklch(84.88% 0.3685 145.6)`.
>
> A less good but still visually acceptable result. Here the hue is more affected, a 5.8° change.
>
> In terms of percentages of linear-light red green and blue, red is 57% higher, green is 6.7% lower and blue is 23% higher.

#### <a id="GM-closest"></a>14.1.2.  Closest Color (MINDE)

A better method is to map colors, in a perceptually uniform color space, by finding the closest in-gamut color (so-called minimum ΔE or <a id="minde"></a>MINDE). Clearly, the success of this technique depends on the degree of uniformity of the gamut mapping color space and the predictive accuracy of the deltaE function used.

However, when doing gamut mapping changes in Hue are <em>particularly</em> objectionable; changes in Chroma are more tolerable, and small changes in Lightness can also be acceptable especially if the alternative is a larger Chroma reduction. MINDE weights changes in each dimension equally, and thus gives suboptimal results.

#### <a id="GM-chroma"></a>14.1.3.  Chroma Reduction

To implement MINDE, colors are mapped in a perceptually uniform, <em>polar</em> color space by holding the hue constant, and reducing the chroma until the color falls in gamut.

This could be done <em>algorithmically</em> by finding the geometric intersection of a constant-lightness, constant-hue ray with the gamut boundary; or <em>iteratively</em>, reducing the chroma until it falls in gamut.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: For performance reasons, iteration is typically performed by binary search.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-gamutmap-p3-yellow-to-srgb"></a> In this example, Display P3 primary yellow (`color(display-p3 1 1 0)`) is being mapped to an sRGB display. The gamut mapping color space is OkLCh.
>
> ```css
> color(display-p3 1 1 0)
> ```
>
> is
>
> ```css
> color(srgb 1 1 -0.3463)
> ```
>
> which is
>
> ```css
> color(oklch 0.96476 0.24503 110.23)
> ```
>
> By progressively reducing the chroma component until the resulting color falls inside the sRGB gamut (has no components negative, or greater than one) a gamut mapped color is obtained.
>
> ```css
>    color(oklch 0.96476 0.21094 110.23)
> ```
>
> which is
>
> ```css
>    color(srgb 0.99116 0.99733 0.00001)
> ```
>
> <a id="gamutmap-p3-yellow"></a> ![](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/slice-ok-110.23.svg)
>
> A constant-hue slice of OkLCh color space. The vertical axis represents lightness, the horizontal axis is chroma. The color to be mapped, shown as a yellow circle, has the chroma reduced while keeping hue and lightness constant. The color therefore moves along the maroon line in the diagram, towards the neutral axis on the left. The gamut boundary of sRGB is shown in green.

#### <a id="GM-excessive-reduction"></a>14.1.4.  Excessive Chroma Reduction

Also, this simple MINDE approach will give sub-optimal results for certain colors, principally very light colors like yellow and cyan, if the upper edge of the gamut boundary is shallow, or even slightly concave. The line of constant lightness can skim just above the gamut boundary, resulting in an excessively low chroma in those cases.

The choice of color space will affect the acceptability of the gamut mapped colors.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="CIELCH-p3-yellow-noclip"></a> In this example, Display P3 primary yellow (`color(display-p3 1 1 0`) has the chroma progressively reduced in CIE LCH color space.
>
> <a id="fig-cielch-p3-yellow-noclip"></a> ![](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/lab-yellow-LCH-fade.svg)
>
> In the upper part of this diagram, colors which are inside the gamut of sRGB are displayed as-is. Colors inside the gamut of Display P3 (but outside sRGB) are in salmon. Colors outside the gamut of Display P3 are in red. The lower part of the diagram shows the linear-light intensities of the Display P3 red, green and blue components.
>
> It can be seen that reduction in CIE LCH chroma makes the red intensity curve up, out of Display P3 gamut; by the time it falls again the chroma is very low. Simple gamut mapping in CIE LCH would give unsatisfactory results.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="OkLCh-p3-yellow-noclip"></a> In this example, Display P3 primary yellow (`color(display-p3 1 1 0`) has the chroma progressively reduced, but this time in OkLCh color space.
>
> <a id="fig-oklch=pr=yellow-noclip"></a> ![](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/p3-yellow-oklab.svg)
>
> In the upper part of this diagram, colors which are inside the gamut of sRGB are displayed as-is. Colors inside the gamut of Display P3 (but outside sRGB) are in salmon. Colors outside the gamut of Display P3 are in red. The lower part of the diagram shows the linear-light intensities of the Display P3 red, green and blue components.
>
> It can be seen that reduction in OkLCh chroma is better behaved. Colors do not go outside the Display P3 gamut, and the resulting gamut-mapped yellow has good chroma. Simple gamut mapping in OK LCH would give acceptable results.

#### <a id="GM-chroma-local-MINDE"></a>14.1.5.  Chroma Reduction with Local Clipping

The simple chroma-reduction algorithm can be improved: at each step, the color difference is computed between the current mapped color and a clipped version of that color. If the current color is outside the gamut boundary, but the color difference between it and the clipped version is below the threshold for a <em>just noticeable difference</em> (JND), the clipped version of the color is returned as the mapped result. Effectively, this is doing a MINDE mapping at each stage, but constrained so the hue and lightness changes are very small, and thus are not noticeable.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="CIELCH-p3-yellow-clip"></a> In this example, Display P3 primary yellow (`color(display-p3 1 1 0`) has the chroma progressively reduced in CIE LCH color space, with the local clip modification.
>
> <a id="fig-cielch-p3-yellow-clip"></a> ![](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/lab-yellow-LCH-clip-fade.svg)
>
> In the upper part of this diagram, colors which are inside the gamut of sRGB are displayed as-is. Colors inside the gamut of Display P3 (but outside sRGB) are in salmon. Colors outside the gamut of Display P3 are in red. The lower part of the diagram shows the linear-light intensities of the Display P3 red, green and blue components.
>
> It can be seen that reduction in CIE LCH chroma still makes the red intensity curve up, out of Display P3 gamut; but less than before and the sRGB boundary is found much more quickly. Gamut mapping in CIE LCH with local clip would give acceptable results.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="OkLCh-p3-yellow-clip"></a> In this example, Display P3 primary yellow (`color(display-p3 1 1 0`) has the chroma progressively reduced, but this time in OkLCh color space and with the local clip modification.
>
> <a id="fig-oklch-p3-yellow-clip"></a> ![](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/p3-yellow-oklab-clip.svg)
>
> In the upper part of this diagram, colors which are inside the gamut of sRGB are displayed as-is. Colors inside the gamut of Display P3 (but outside sRGB) are in salmon. Colors outside the gamut of Display P3 are in red. The lower part of the diagram shows the linear-light intensities of the Display P3 red, green and blue components.
>
> It can be seen that reduction in OkLCh chroma, which was already good, is further improved by the local clip modification. Simple gamut mapping in CIE LCH with local clip would give excellent results.

#### <a id="GM-hue-curvature"></a>14.1.6.  Deviations from Perceptual Uniformity: Hue Curvature

Performing gamut mapping in the CIE LCH color space even with the deltaE2000 distance metric, is known to give suboptimal results with significant hue shifts, for colors in the hue range 270° to 330°.

<a id="fig-cielch-blue-curvature"></a> ![](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/CIELCH-blue-slice.png)

A constant-hue slice of CIE LCH color space, at a hue angle of 301.37° corresponding to sRGB primary blue. The vertical axis is Lightness, the horizontal axis is Chroma. Between chroma of 25 and 75, the hue is visibly purple, becoming more blue between 100 and 131. The same phenomenon continues past 131, but cannot be shown on an sRGB display.

Using OkLCh color space and the deltaEOK distance metric avoids this issue at all hue angles.

<a id="fig-oklch-blue-linearity"></a> ![](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/OKLCH-blue-slice.png)

A constant-hue slice of OkLCh color space, at a hue angle of 264.06° corresponding to sRGB primary blue. The vertical axis is Lightness, the horizontal axis is Chroma. The hue is visibly the same at all values of chroma, up to 0.315 (the sRGB limit at this hue). It continues to be constant beyond this point, although that cannot be shown on an sRGB diagram.

### <a id="css-gamut-mapping"></a>14.2.  CSS Gamut Mapping to an RGB Destination

Tests

Actual values of color are not exposed to script, making this hard to test in an automated manner.

------------------------------------------------------------------------

The three <a id="css-gamut-mapping-algorithms"></a>CSS gamut mapping algorithms apply to individual, Standard Dynamic Range (SDR) CSS colors which are out of gamut of an RGB display and thus require to be <a id="css-gamut-mapped"></a>css gamut mapped.

Implementations my choose any of the three algorithms based on their quality and runtime efficiency tradeoffs, and must use their chosen algorithm wherever CSS mandates that gamut mapping be performed.

- [Binary Search Gamut Mapping with Local MINDE](#GMA-Binary-local-MINDE)
- [EdgeSeeker Gamut Mapping](#GMA-EdgeSeeker)
- [Ray Trace Gamut Mapping](#GMA-Raytrace)

They all implement a relative colorimetric intent, thus colors inside the destination gamut are unchanged.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: other situations, in particular mapping to printer gamuts where the maximum black level is significantly above zero, will require different algorithms which align the respective black and white points, which will result in lightness changes for very light and very dark colors as chroma is reduced..

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: these algorithms are for individual, distinct colors; for color images, where relationships between neighboring pixels are important and the aim is to preserve detail and texture, a perceptual rendering intent is more appropriate and in that case, colors inside the destination gamut could be changed.

All three CSS gamut mapping algorithms aim at constant-lightness, constant-hue chroma reduction in the [OkLCh color space](#ok-lab).

For colors which are out of range on the Lightness axis, white is returned in the destination color space if the Lightness is greater than or equal to 1.0, while black is returned in the destination color space if the Lightness is less than or equal to 0.0.

#### <a id="GMA-Binary-local-MINDE"></a>14.2.1.  Binary Search Gamut Mapping with Local MINDE 

For this binary search algorithm, the color difference formula used is [deltaEOK](#color-difference-OK). The [local-MINDE](#GM-chroma-local-MINDE) improvement is used. At each step in the search, the deltaEOK is computed between the current mapped color and a clipped version of that color.

If the current color is <em>outside</em> the gamut boundary, but the deltaEOK between it and the clipped version is below a threshold for a <em>just noticeable difference</em> (JND), the clipped version of the color is returned as the mapped result. This gives good results with non-notivceable hue shifts, and avoids excessive chroma reduction near concave gamut surfaces, but can be computationally intensive.

For the OkLCh color space, one JND is is an OkLCh difference of 0.02.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In CIE Lab color space, where the range of the Lightness component is 0 to 100, using deltaE2000, one JND is 2. Because the range of Lightness in Oklab and OkLCh is 0 to 1, using deltaEOK, one JND is 100 times smaller.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: for the purposes of experimentation, and comparing implementations, implementations of the Binary Search with Local MINDE algorithm are available in the Coloriade library (in Python) [\[Coloraide-MINDE\]](#biblio-coloraide-minde) and the color.js library (in JavaScript) [\[colorjs-MINDE\]](#biblio-colorjs-minde).

#### <a id="pseudo-binsearch"></a>14.2.2.  Sample Pseudocode for the Binary Search Gamut Mapping with Local MINDE

To <a id="binary-search-gamut-map-with-local-minde"></a>Binary Search Gamut Map with Local MINDE a color <var>origin</var> in color space <var>origin color space</var> to be in gamut of a destination color space <var>destination</var>:

1.  if <var>destination</var> has no gamut limits (XYZ-D65, XYZ-D50, Lab, LCH, Oklab, OkLCh) convert <var>origin</var> to <var>destination</var> and return it as the gamut mapped color
2.  let <var>origin&#x5F;OkLCh</var> be <var>origin</var> converted from <var>origin color space</var> to the OkLCh color space
3.  if the Lightness of <var>origin&#x5F;OkLCh</var> is greater than or equal to 100%, convert \`oklab(1 0 0 / origin.alpha)\` to <var>destination</var> and return it as the gamut mapped color
4.  if the Lightness of <var>origin&#x5F;OkLCh</var> is less than than or equal to 0%, convert \`oklab(0 0 0 / origin.alpha)\` to <var>destination</var> and return it as the gamut mapped color
5.  let inGamut(<var>color</var>) be a function which returns true if, when passed a color, that color is inside the gamut of <var>destination</var>. For HSL and HWB, it returns true if the color is inside the gamut of sRGB.
6.  if inGamut(<var>origin&#x5F;OkLCh</var>) is true, convert <var>origin&#x5F;OkLCh</var> to <var>destination</var> and return it as the gamut mapped color
7.  otherwise, let delta(<var>one</var>, <var>two</var>) be a function which returns the deltaEOK of color <var>one</var> compared to color <var>two</var>
8.  let <var>JND</var> be 0.02
9.  let <var>epsilon</var> be 0.0001
10. let clip(<var>color</var>) be a function which converts <var>color</var> to <var>destination</var>, clamps each component to the bounds of the reference range for that component and returns the result
11. set <var>current</var> to <var>origin&#x5F;OkLCh</var>
12. set <var>clipped</var> to clip(<var>current</var>)
13. set <var>E</var> to delta(<var>clipped</var>, <var>current</var>)
14. if <var>E</var> \< <var>JND</var>
    1.  return <var>clipped</var> as the gamut mapped color
15. set <var>min</var> to zero
16. set <var>max</var> to the OkLCh chroma of <var>origin&#x5F;OkLCh</var>
17. let <var>min&#x5F;inGamut</var> be a boolean that represents when <var>min</var> is still in gamut, and set it to true
18. while (<var>max</var> - <var>min</var> is greater than <var>epsilon</var>) repeat the following steps
    1.  set <var>chroma</var> to (<var>min</var> + <var>max</var>) /2
    2.  set the chroma component of <var>current</var> to <var>chroma</var>
    3.  if <var>min&#x5F;inGamut</var> is true and also if inGamut(<var>current</var>) is true, set <var>min</var> to <var>chroma</var> and continue to repeat these steps
    4.  otherwise, carry out these steps:
        1.  set <var>clipped</var> to clip(<var>current</var>)
        2.  set <var>E</var> to delta(<var>clipped</var>, <var>current</var>)
        3.  if <var>E</var> \< <var>JND</var>
            1.  if (<var>JND</var> - <var>E</var> \< <var>epsilon</var>) return <var>clipped</var> as the gamut mapped color
            2.  otherwise,
                1.  set <var>min&#x5F;inGamut</var> to false
                2.  set <var>min</var> to <var>chroma</var>
        4.  otherwise, set <var>max</var> to <var>chroma</var> and continue to repeat these steps
19. return <var>clipped</var> as the gamut mapped color

#### <a id="GMA-EdgeSeeker"></a>14.2.3.  The EdgeSeeker Gamut Mapping 

The EdgeSeeker algorithm is a geometric and lookup-table-based approach, originally developed by Alexey Ardov for the color.js library [\[colorjs-EdgeSeeker\]](#biblio-colorjs-edgeseeker).

For any given hue, the gamut boundary slice is represented as a curved top section and a linear bottom section, joning at the highest chroma point for that hue.

To initialize this algorithm, for a given target RGB space, a lookup table (LUT) is constructed containing the highest-chroma Oklch color on each hue slice. Linear interpolation between closest LUT values is used to then estimate the highest-chroma color for the exact hue of each color to be gamut mapped.

Intersection of the constant-lightness ray with the gamut boundary is then calculated, which is fast for the lower (linear) part of the boundary and still fairly fast for the upper (curved) portion.

This gives good results, at the expense of memory for the LUT.

#### <a id="pseudo-edgeseeker"></a>14.2.4.  Sample Pseudocode for the EdgeSeeker Gamut Mapping

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-4b1038ec"></a> add pseudocode for EdgeSeeker GMA

#### <a id="GMA-Raytrace"></a>14.2.5.  The Ray Trace Gamut Mapping 

The Ray Trace algorithm is a geometric approach to RGB gamut mapping, for fast chroma reduction with constant lightness. It was originally developed by Isaac Muse for the Coloraide Python Library [\[Coloraide-Ray-Trace\]](#biblio-coloraide-ray-trace).

The color to be mapped is first converted to Oklch, and then the achromatic version of that color is generated, which will be the neutral axis anchor. These two colors are then converted to the linear-light version of the target RGB space.

Because the gamut boundary is now an axis-aligned cube, finding the intersection is faster.

A ray is cast from the inside of the RGB cube, from the anchor point to the current color. The intersection along this path with the RGB gamut surface is then found; this is the first approximation to the gamut mapped color.

As RGB spaces are not perceptually uniform, a constant hue, constant lightness ray is actually a curved path in RGB space.

The first approximation is converted back to Oklch to correct the color in the perceptual color space by projecting the point back onto the chroma reduction path, correcting the color’s hue and lightness. The corrected color becomes the new current color and should be a much closer color on the reduced chroma line.

This process is repeated (a maximum of three more times), each time finding a better, closer color on the path. Finally, simple clipping is used to account for floating point math errors.

The results are comparable to binary search with local MINDE using a low JND, but resolves much faster and within more predictable, consistent time.

![](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/images/raytrace-gma.png)

Ray Trace gamut mpping to the sRGB gamut, showing the curved path of chroma reduction as approximated over a maximum of four iterations.  
Image copyright Isaac Muse.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: for the purposes of experimentation, and comparing implementations, implementations of the Ray Trace are available in the Coloriade library (in Python) [\[Coloraide-Ray-Trace\]](#biblio-coloraide-ray-trace) and the color.js library (in JavaScript) [\[colorjs-RayTrace\]](#biblio-colorjs-raytrace).

#### <a id="pseudo-raytrace"></a>14.2.6.  Sample Pseudocode for the Ray Trace Gamut Mapping

To <a id="ray-trace-gamut-map"></a>Ray Trace Gamut Map a color <var>origin</var> in color space <var>origin color space</var> to be in gamut of an RGB destination color space <var>destination</var>:

1.  if <var>destination</var> has no gamut limits (XYZ-D65, XYZ-D50, Lab, LCH, Oklab, OkLCh) convert <var>origin</var> to <var>destination</var> and return it as the gamut mapped color
2.  let <var>origin&#x5F;OkLCh</var> be <var>origin</var> converted from <var>origin color space</var> to the OkLCh color space
3.  if the Lightness of <var>origin&#x5F;OkLCh</var> is greater than or equal to 100%, convert \`oklab(1 0 0 / origin.alpha)\` to <var>destination</var> and return it as the gamut mapped color
4.  if the Lightness of <var>origin&#x5F;OkLCh</var> is less than than or equal to 0%, convert \`oklab(0 0 0 / origin.alpha)\` to <var>destination</var> and return it as the gamut mapped color
5.  let <var>l&#x5F;origin</var> be the OkLCh lightness component of <var>origin&#x5F;OkLCh</var>
6.  let <var>h&#x5F;origin</var> be the OkLCh hue component of <var>origin&#x5F;OkLCh</var>
7.  let <var>anchor</var> be an achromatic OkLCh color formed with <var>l&#x5F;origin</var> as lightness, 0 as chroma and <var>h&#x5F;origin</var> as hue, converted to the <em>linear-light</em> form of <var>destination</var>
8.  let <var>origin&#x5F;rgb</var> be <var>origin&#x5F;OkLCh</var> converted to the <em>linear-light</em> form of <var>destination</var>
9.  if <var>origin&#x5F;rgb</var> is not in gamut
    - let <var>low</var> be 0.0 + 1E-6 [<sup>1</sup>](#raytrace-footnote-1)
    - let <var>high</var> be 1.0 - 1E-6 [<sup>2</sup>](#raytrace-footnote-2)
    - let <var>last</var> be <var>origin&#x5F;rgb</var>
    - for (i=0; i\<4; i++)
      - if (i \> 0)
        - let <var>current&#x5F;OkLCh</var> be <var>origin&#x5F;rgb</var> converted to OkLCh
        - let the lightness of <var>current&#x5F;OkLCh</var> be <var>l&#x5F;origin</var>
        - let the hue of <var>current&#x5F;OkLCh</var> be <var>h&#x5F;origin</var> [<sup>3</sup>](#raytrace-footnote-3)
        - let <var>origin&#x5F;rgb</var> be <var>current&#x5F;OkLCh</var> converted to the <em>linear-light</em> form of <var>destination</var>
      - <b>Cast a ray</b> from <var>start</var> = <var>anchor</var> to <var>end</var> = <var>origin&#x5F;rgb</var> and let <var>intersection</var> be the intersection of this ray with the gamut boundary
      - if an intersection was not found, let <var>origin&#x5F;rgb</var> be <var>last</var> and exit the loop [<sup>5</sup>](#raytrace-footnote-5)
      - if (i \>0) AND (each component of <var>origin&#x5F;rgb</var> is between <var>low</var> and <var>high</var>) then let <var>anchor</var> be <var>origin&#x5F;rgb</var> [<sup>4</sup>](#raytrace-footnote-4)
      - let <var>origin&#x5F;rgb</var> be <var>intersection</var>
      - let <var>last</var> be <var>intersection</var>
10. let clip(<var>color</var>) be a function which converts <var>color</var> to <var>destination</var>, clamps each component to the bounds of the reference range for that component and returns the result
11. set <var>clipped</var> to clip(<var>current</var>)
12. return <var>clipped</var> as the gamut mapped color

To <a id="cast-a-ray"></a>cast a ray through a linear-light RGB space from <var>start</var> to <var>end</var> (in gamut mapping, <var>start</var> is an anchor within the RGB gamut and <var>end</var> is the gamut mapped color, on the cubical gamut surface):

1.  let <var>bmin</var> and <var>bmax</var> be 3-element arrays with the gamut’s lower and upper bounds, respectively [<sup>6</sup>](#raytrace-footnote-6)
2.  let <var>tfar</var> be infinity (or some very large number)
3.  let <var>tnear</var> be -infinity (or some very large, negative number)
4.  let <var>direction</var> be a 3-element array
5.  for (i = 0; i \< 3; i++):
    - let <var>a</var> be <var>start</var> <i>&#x5B;i&#x5D;</i>
    - let <var>b</var> be <var>end</var> <i>&#x5B;i&#x5D;</i>
    - let <var>d</var> be <var>b</var> - <var>a</var>
    - let <var>direction</var> <i>&#x5B;i&#x5D;</i> be <var>d</var>
    - if abs(<var>d</var>) \< 1E-12
      - let <var>inv&#x5F;d</var> be 1 / <var>d</var>
      - let <var>t1</var> be (<var>bmin</var> <i>&#x5B;i&#x5D;</i> - <var>a</var> ) \* <var>inv&#x5F;d</var>
      - let <var>t2</var> be (<var>bmax</var> <i>&#x5B;i&#x5D;</i> - <var>a</var> ) \* <var>inv&#x5F;d</var>
      - let <var>tnear</var> be max(min(<var>t1</var>, <var>t2</var>), <var>tnear</var> )
      - let <var>tfar</var> be min(max(<var>t1</var>, <var>t2</var>), <var>tfar</var> )
    - else if (<var>a</var> \< <var>bmin</var><i>&#x5B;i&#x5D;</i> or <var>a</var> \> <var>bmax</var><i>&#x5B;i&#x5D;</i>)
      - return INTERSECTION NOT FOUND
6.  if (<var>tnear</var> \> <var>tfar</var> or <var>tfar</var> \< 0)
    - return INTERSECTION NOT FOUND
7.  if <var>tnear</var> \< 0
    - let <var>tnear</var> be <var>tfar</var> [<sup>7</sup>](#raytrace-footnote-7)
8.  if <var>tnear</var> is infinite (or matches the initial very large value)
    - return INTERSECTION NOT FOUND
9.  for (i =0; i \< 3; i++):
    - let <var>result</var> <i>&#x5B;i&#x5D;</i> be <var>start</var> <i>&#x5B;i&#x5D;</i> + <var>direction</var> <i>&#x5B;i&#x5D;</i> \* <var>tnear</var>
10. return <var>result</var>

##### <a id="raytrace-footnotes"></a>14.2.6.1. Footnotes for Ray Trace algorithm

1.  <a id="raytrace-footnote-1"></a> It is assumed the minimum value is 0 and that all channels have the same minimum. The value should be small relative to the unit type. 64 bit could easily be as small as 1e-14, but 1e-6 is fine in practice.
2.  <a id="raytrace-footnote-2"></a> 1.0 represents the maximum in-gamut channel value, and it is assumed all channels have the same maximum.
3.  <a id="raytrace-footnote-3"></a> This places the <var>current</var> color back on the chroma reduction curve, if it has deviated.
4.  <a id="raytrace-footnote-4"></a> This means <var>origin&#x5F;rgb</var> is below the gamut surface, so we use it as an anchor closer to the gamut surface.
5.  <a id="raytrace-footnote-5"></a> This is provided for catastrophic failures where a specific, perceptual mapping space completely breaks down due to ridiculously wide colors (outside the visible spectrum). It is expected that non-imaginary colors in CSS should never trigger this.
6.  <a id="raytrace-footnote-6"></a> For typical RGB spaces where the gamut bounds are 0 and 1 for each component this simplifies to a single constant rather than a 3-element array.
7.  <a id="raytrace-footnote-7"></a> favoring the first intersection in the direction <var>start</var> -\> <var>end</var> .

<a id="ref-for-typedef-color②②"></a>

## <a id="resolving-color-values"></a>15.  Resolving [\<color\>](#typedef-color) Values

<a id="ref-for-specified-value"></a>

<a id="ref-for-computed-value①"></a>

<a id="ref-for-used-value②"></a>

Unless otherwise specified for a particular property, [specified](https://www.w3.org/TR/css-cascade-5/#specified-value) colors are resolved to <a id="computed-color"></a>[computed](https://www.w3.org/TR/css-cascade-5/#computed-value) colors and then further to <a id="used-color"></a>[used](https://www.w3.org/TR/css-cascade-5/#used-value) colors as described below.

<a id="ref-for-resolved-value"></a>

<a id="ref-for-typedef-color②③"></a>

<a id="ref-for-used-value③"></a>

The [resolved value](https://www.w3.org/TR/cssom-1/#resolved-value) of a [\<color\>](#typedef-color) is its [used value](https://www.w3.org/TR/css-cascade-5/#used-value).

Tests

- [color-computed-hex-color.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-hex-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-hex-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-hex-color.html)
- [color-computed-named-color.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-named-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-named-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-named-color.html)
- [color-invalid-hex-color.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-hex-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-hex-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-hex-color.html)
- [color-invalid-named-color.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-named-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-named-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-named-color.html)
- [system-color-compute.html](https://wpt.fyi/results/css/css-color/system-color-compute.html) [(live test)](http://wpt.live/css/css-color/system-color-compute.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/system-color-compute.html)

### <a id="resolving-sRGB-values"></a>15.1.  Resolving sRGB values

This applies to:

- <a id="ref-for-hex-color④"></a>

  [hex colors](#hex-color)

- <a id="ref-for-funcdef-rgb①⑤"></a>

  <a id="ref-for-funcdef-rgba①⓪"></a>

  [rgb()](#funcdef-rgb) and [rgba()](#funcdef-rgba) values

- <a id="ref-for-funcdef-hsl①⑧"></a>

  <a id="ref-for-funcdef-hsla①②"></a>

  [hsl()](#funcdef-hsl) and [hsla()](#funcdef-hsla) values

- <a id="ref-for-funcdef-hwb①①"></a>

  [hwb()](#funcdef-hwb) values

- <a id="ref-for-named-color③"></a>

  [named colors](#named-color)

- <a id="ref-for-css-system-colors②"></a>

  [system colors](#css-system-colors)

- [deprecated-colors](#deprecated-system-colors)

It does <em>not</em> apply to:

- <a id="ref-for-funcdef-color⑧"></a>

  <a id="ref-for-valdef-color-srgb①⓪"></a>

  <a id="ref-for-valdef-color-srgb-linear⑥"></a>

  <a id="ref-for-color-space⑥"></a>

  [color()](#funcdef-color) values using the [srgb](#valdef-color-srgb) or [srgb-linear](#valdef-color-srgb-linear) [color space](#color-space)s.

<a id="ref-for-named-color④"></a>

<a id="ref-for-css-system-colors③"></a>

<a id="ref-for-declared-value"></a>

If the sRGB color was explicitly specified by the author as a [named color](#named-color), or as a [system color](#css-system-colors), the [declared value](https://www.w3.org/TR/css-cascade-5/#declared-value) is that named or system color, converted to [ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase). The computed and used value is the corresponding sRGB color, paired with the specified alpha component (after clamping to \[0, 1\]) and defaulting to opaque if unspecified).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-named-case"></a>
>
> The author-provided mixed-case form below has a declared value in all lowercase.
>
> ```css
>  pUrPlE
> 
>        purple
> ```
Otherwise, the declared, computed and used value is the corresponding sRGB color, paired with the specified alpha component (after clamping to \[0, 1\]) and defaulting to opaque if unspecified).

<a id="ref-for-funcdef-calc"></a>

For historical reasons, when [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) in sRGB colors resolves to a single value, the declared value serialises without the "calc(" ")" wrapper.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-srgb-calc-specified"></a> For example, if a color is given as rgb(calc(64 \* 2) 127 255) the declared value will be rgb(128 127 255) and not rgb(calc(128) 127 255).

<a id="ref-for-funcdef-calc①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-srgb-hsl-calc-specified"></a> For example, if a color is given as hsl(38.82 calc(2 \* 50%) 50%) the declared value will be rgb(255 165.2 0) because the [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) is lost during HSL to RGB conversion.

Also for historical reasons, when calc() is simplified down to a single value, the color values are clamped to \[0.0, 255.0\].

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-srgb-clamped-calc-specified"></a> For example, if a color is given as rgb(calc(100 \* 4) 127 calc(20 - 35)) the declared value will be rgb(255 127 0) and not rgb(calc(400) 127 calc(-15)).

<a id="ref-for-valdef-calc-infinity"></a>

<a id="ref-for-valdef-calc--infinity"></a>

<a id="ref-for-valdef-calc-nan"></a>

This clamping also takes care of values such as [Infinity](https://www.w3.org/TR/css-values-4/#valdef-calc-infinity), [-Infinity](https://www.w3.org/TR/css-values-4/#valdef-calc--infinity), and [NaN](https://www.w3.org/TR/css-values-4/#valdef-calc-nan) which will clamp at 255, 0 and 0 respectively.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-hsl-computed"></a>
>
> For example, the computed value of
>
> ```css
>  hsl(38.824 100% 50%)
> ```
>
> is
>
> ```css
>  rgb(255, 165, 0)
> ```
Tests

- [color-computed-hsl.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-hsl.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-hsl.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-hsl.html)
- [color-computed-hwb.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-hwb.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-hwb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-hwb.html)
- [color-computed-rgb.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-rgb.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-rgb.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-rgb.html)

### <a id="resolving-lab-lch-values"></a>15.2.  Resolving Lab and LCH values

<a id="ref-for-funcdef-lab⑤"></a>

<a id="ref-for-funcdef-lch⑥"></a>

This applies to [lab()](#funcdef-lab) and [lch()](#funcdef-lch) values.

<a id="ref-for-number-value③⓪"></a>

<a id="ref-for-percentage-value③②"></a>

The declared, computed and used value is the corresponding CIE Lab or LCH color (after clamping of L, C and H) paired with the specified alpha component (as a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), not a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value); and defaulting to opaque if unspecified).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-lch-computed"></a>
>
> For example, the computed value of
>
> ```css
>  lch(52.2345% 72.2 56.2 / 1)
> ```
>
> is
>
> ```css
>  lch(52.2345% 72.2 56.2)
> ```
<a id="ref-for-implementation-defined-limit-for-values-approaching-infinity"></a>

Although the values of a, b and C are theoretically unbounded, there may be an [implementation-defined limit for values approaching infinity](https://drafts.csswg.org/css-values-4/#implementation-defined-limit-for-values-approaching-infinity).

Tests

- [color-computed-lab.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-lab.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-lab.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-lab.html)

### <a id="resolving-oklab-oklch-values"></a>15.3.  Resolving Oklab and OkLCh values

<a id="ref-for-funcdef-oklab⑤"></a>

<a id="ref-for-funcdef-oklch⑥"></a>

This applies to [oklab()](#funcdef-oklab) and [oklch()](#funcdef-oklch) values.

<a id="ref-for-number-value③①"></a>

<a id="ref-for-percentage-value③③"></a>

The declared, computed and used value is the corresponding Oklab or OkLCh color (after clamping of L, C and H) paired with the specified alpha component (as a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), not a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value); and defaulting to opaque if unspecified).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-oklch-computed"></a>
>
> For example, the computed value of
>
> ```css
>  oklch(42.1% 0.192 328.6 / 1)
> ```
>
> is
>
> ```css
>  oklch(42.1% 0.192 328.6)
> ```
<a id="ref-for-implementation-defined-limit-for-values-approaching-infinity①"></a>

Although the values of a, b and C are theoretically unbounded, there may be an [implementation-defined limit for values approaching infinity](https://drafts.csswg.org/css-values-4/#implementation-defined-limit-for-values-approaching-infinity).

Tests

- [color-computed-lab.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-lab.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-lab.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-lab.html)

<a id="ref-for-funcdef-color⑨"></a>

### <a id="resolving-color-function-values"></a>15.4.  Resolving values of the [color()](#funcdef-color) function

<a id="ref-for-color-space⑦"></a>

<a id="ref-for-number-value③②"></a>

<a id="ref-for-percentage-value③④"></a>

The declared, computed and used value is the color in the specified [color space](#color-space), paired with the specified alpha component (as a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), not a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value); and defaulting to opaque if unspecified).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-p3-computed"></a>
>
> For example, the computed value of
>
> ```css
>  color(display-p3 0.823 0.6554 0.2537 /1)
> ```
>
> is
>
> ```css
>  color(display-p3 0.823 0.6554 0.2537)
> ```
<a id="ref-for-valdef-color-xyz⑤"></a>

<a id="ref-for-color-space⑧"></a>

<a id="ref-for-valdef-color-xyz-d65③"></a>

For colors specified in the [xyz](#valdef-color-xyz) [color space](#color-space), which is an alias of the [xyz-d65](#valdef-color-xyz-d65) <a id="ref-for-color-space⑨"></a>color space, the computed and used value is in the <a id="ref-for-valdef-color-xyz-d65④"></a>xyz-d65 <a id="ref-for-color-space①⓪"></a>color space.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-xyz-computed"></a>
>
> For example, the computed value of
>
> ```css
>  color(xyz 0.472 0.372 0.131)
> ```
>
> is
>
> ```css
>  color(xyz-d65 0.472 0.372 0.131)
> ```
<a id="ref-for-implementation-defined-limit-for-values-approaching-infinity②"></a>

Although the values of r, g, b, x, y and z are theoretically unbounded, there may be an [implementation-defined limit for values approaching infinity](https://drafts.csswg.org/css-values-4/#implementation-defined-limit-for-values-approaching-infinity).

Tests

- [color-computed-color-function.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-color-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-color-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-color-function.html)

### <a id="resolving-other-colors"></a>15.5. Resolving other colors

<a id="ref-for-css-system-colors④"></a>

<a id="ref-for-typedef-deprecated-color"></a>

<a id="ref-for-valdef-color-transparent③"></a>

This applies to [system colors](#css-system-colors) (including the [\<deprecated-color\>](#typedef-deprecated-color)s), [transparent](#valdef-color-transparent), and currentcolor.

<a id="ref-for-typedef-system-color①⓪"></a>

<a id="ref-for-typedef-deprecated-color①"></a>

<a id="ref-for-forced-colors-mode②"></a>

The declared value for each [\<system-color\>](#typedef-system-color) keyword and [\<deprecated-color\>](#typedef-deprecated-color) keyword is itself. The computed value is the corresponding color in its color space. However, such colors must not be altered by [forced colors mode](https://www.w3.org/TR/css-color-adjust-1/#forced-colors-mode).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-system-resolve"></a> For example, in this html:
>
> ```html
> <button style="color:  ButtonText; background:  ButtonFace"></button>
> ```
>
> The declared value of the color property is "ButtonText" while the computed value could be, for example, rgb(0, 0, 0).

<a id="ref-for-valdef-color-transparent④"></a>

<a id="ref-for-transparent-black①"></a>

The declared value of [transparent](#valdef-color-transparent) is "transparent" while the computed and used value is [transparent black](#transparent-black).

<a id="ref-for-valdef-color-currentcolor①①"></a>

The [currentcolor](#valdef-color-currentcolor) keyword computes to itself.

<a id="ref-for-propdef-color④"></a>

<a id="ref-for-inherited-value"></a>

In the [color](#propdef-color) property, the used value of currentcolor is the resolved [inherited value](https://www.w3.org/TR/css-cascade-5/#inherited-value). In any other property, its used value is the used value of the <a id="ref-for-propdef-color⑤"></a>color property on the same element.

<a id="ref-for-valdef-color-currentcolor①②"></a>

<a id="ref-for-propdef-color⑥"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means that if the [currentcolor](#valdef-color-currentcolor) value is inherited, it’s inherited as a keyword, not as the value of the [color](#propdef-color) property, so descendants will use their own <a id="ref-for-propdef-color⑦"></a>color property to resolve it.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-currentcolor-resolve"></a> For example, given this html:
>
> ```html
> <div>
>   <p>Assume this example text is long enough
>     to wrap on multiple lines.
>   </p>
> </div>
> ```
>
> and this css:
>
> ```css
> div {
>   color:  forestgreen;
>   text-shadow: currentColor;
> }
> p {
>   color:  mediumseagreen;
> }
> p::firstline {
>   color:  yellowgreen;
> }
> ```
>
> The used value of the inherited property text-shadow on the first line fragment would be yellowgreen.

Tests

- [currentcolor-001.html](https://wpt.fyi/results/css/css-color/currentcolor-001.html) [(live test)](http://wpt.live/css/css-color/currentcolor-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/currentcolor-001.html)
- [currentcolor-002.html](https://wpt.fyi/results/css/css-color/currentcolor-002.html) [(live test)](http://wpt.live/css/css-color/currentcolor-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/currentcolor-002.html)
- [currentcolor-003.html](https://wpt.fyi/results/css/css-color/currentcolor-003.html) [(live test)](http://wpt.live/css/css-color/currentcolor-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/currentcolor-003.html)
- [currentcolor-005.html](https://wpt.fyi/results/css/css-color/currentcolor-005.html) [(live test)](http://wpt.live/css/css-color/currentcolor-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/currentcolor-005.html)
- [system-color-compute.html](https://wpt.fyi/results/css/css-color/system-color-compute.html) [(live test)](http://wpt.live/css/css-color/system-color-compute.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/system-color-compute.html)

<a id="ref-for-typedef-color②④"></a>

## <a id="serializing-color-values"></a>16.  Serializing [\<color\>](#typedef-color) Values

<a id="ref-for-typedef-color②⑤"></a>

This section updates and replaces that part of CSS Object Model, section [Serializing CSS Values](https://drafts.csswg.org/cssom-1/#serializing-css-values), which relates to serializing [\<color\>](#typedef-color) values.

In this section, the strings used in the specification and the corresponding characters are as follows.



| String | Character(s)                                                                                                                                                       |
|--------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| " "    | U+0020 SPACE                                                                                                                                                       |
| "#"    | U+0023 NUMBER SIGN                                                                                                                                                 |
| ","    | U+002C COMMA                                                                                                                                                       |
| "-"    | U+002D HYPHEN-MINUS                                                                                                                                                |
| "."    | U+002E FULL STOP                                                                                                                                                   |
| "/"    | U+002F SOLIDUS                                                                                                                                                     |
| "none" | U+006E LATIN SMALL LETTER N<br> U+006F LATIN SMALL LETTER O<br> U+006E LATIN SMALL LETTER N<br> U+0065 LATIN SMALL LETTER E |



The string "." shall be used as a decimal separator, regardless of locale, and there shall be no thousands separator.

<a id="ref-for-missing-color-component③③"></a>

<a id="ref-for-valdef-color-none①①"></a>

For syntactic forms which support [missing color components](#missing-color-component), the value [none](#valdef-color-none) (equivalently NONE, nOnE, etc), shall be serialized in all-lowercase as the string "none".

### <a id="serializing-alpha-values"></a>16.1.  Serializing alpha values

<a id="ref-for-typedef-color②⑥"></a>

This applies to any [\<color\>](#typedef-color) value which can take an optional alpha value. It does not apply to the opacity property.

If, after clamping to the range \[0, 1\] the alpha is 1, it is omitted from the serialization; an implicit value of 1 (fully opaque) is the default.

If the alpha is any other value than 1, it is explicitly included in the serialization as described below.

If the value is internally represented as an integer between 0 and 255 inclusive (i.e. 8-bit unsigned integer), follow these steps:

1.  Let <var>alpha</var> be the given integer.

2.  If there exists an integer between 0 and 100 inclusive that, when multiplied with 2.55 and rounded to the closest integer (rounding up if two values are equally close), equals <var>alpha</var>, let <var>rounded</var> be that integer divided by 100.

3.  Otherwise, let <var>rounded</var> be <var>alpha</var> divided by 0.255 and rounded to the closest integer (rounding up if two values are equally close), divided by 1000.

4.  <a id="ref-for-number-value③③"></a>

    Return the result of serializing <var>rounded</var> as a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value).

<a id="ref-for-number-value③④"></a>

<a id="ref-for-percentage-value③⑤"></a>

Otherwise, return the result of serializing the given value (as a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), not a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-alpha-255"></a>
>
> For example, if the alpha is stored as the 8-bit unsigned integer 237, the integer 93 satisfies the criterion because Math.round(93 \* 2.55) is 237, and so the alpha is serialized as "0.93".
>
> However, if the alpha is stored as the 8-bit unsigned integer 236, there is no such integer (92 maps to 235 while 94 maps to 240), and so since 236 ÷ 0.255 = 925.490196078 the alpha is serialized as "0.92549" (no more than 6 figures, trailing zeroes omitted).

<a id="ref-for-number-value③⑤"></a>

The [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) value is expressed in base ten, with the "." character as decimal separator. The leading zero must not be omitted. Trailing zeroes must be omitted.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-alpha-trimzero"></a>
>
> For example, an alpha value of 70% will be serialized as the string "0.7" which has a leading zero before the decimal separator, "." as decimal separator (even if the current locale would use some other character, such as ","), and all digits after the "7" would be "0" and are omitted.

The precision with which alpha values are retained, and thus the number of decimal places in the serialized value, is not defined in this specification, but must at least be sufficient to round-trip integer percentage values. Thus, the serialized value must contain at least two decimal places (unless trailing zeroes have been removed). Values must be [rounded towards +∞](https://drafts.csswg.org/css-values-4/#combine-integers), not truncated.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-alpha-round"></a>
>
> For example, an alpha value of 12.3456789% could be serialized as the strings "0.12" or "0.123" or "0.1234" or "0.12346" (rounding the value of 5 towards +∞ because the following digit is 6) or any longer, rounded serialization of the same form.

<a id="ref-for-typedef-color-alpha-value②④"></a>

Because [\<alpha-value\>](#typedef-color-alpha-value)s which were specified outside the valid range are clamped at parse time, the declared value will be clamped. However, per [CSS Values 4 § 10.12 Range Checking](https://www.w3.org/TR/css-values-4/#calc-range), <a id="ref-for-typedef-color-alpha-value②⑤"></a>\<alpha-value\>s specified using calc() are not clamped when the specified form is serialized; but the computed values are clamped.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-alpha-clamp"></a>
>
> For example an alpha value which was specified directly as 120% would be serialized as the string "1". However, if it was specified as calc(2\*60%) the declared value would be serialized as the string "calc(1.2)".

### <a id="serializing-sRGB-values"></a>16.2.  Serializing sRGB values

The serialized form of the following sRGB values:

- <a id="ref-for-hex-color⑤"></a>

  [hex colors](#hex-color)

- <a id="ref-for-funcdef-rgb①⑥"></a>

  <a id="ref-for-funcdef-rgba①①"></a>

  [rgb()](#funcdef-rgb) and [rgba()](#funcdef-rgba) values

- <a id="ref-for-funcdef-hsl①⑨"></a>

  <a id="ref-for-funcdef-hsla①③"></a>

  [hsl()](#funcdef-hsl) and [hsla()](#funcdef-hsla) values

- <a id="ref-for-funcdef-hwb①②"></a>

  [hwb()](#funcdef-hwb) values

- <a id="ref-for-named-color⑤"></a>

  [named colors](#named-color)

- <a id="ref-for-css-system-colors⑤"></a>

  [system colors](#css-system-colors)

- [deprecated-colors](#deprecated-system-colors)

- <a id="ref-for-valdef-color-transparent⑤"></a>

  [transparent](#valdef-color-transparent)

<a id="ref-for-declared-value①"></a>

is derived from the [declared value](https://www.w3.org/TR/css-cascade-5/#declared-value).

<a id="ref-for-named-color⑥"></a>

<a id="ref-for-css-system-colors⑥"></a>

<a id="ref-for-valdef-color-transparent⑥"></a>

<a id="ref-for-declared-value②"></a>

When serializing the value of a property which was set by the author to a CSS [named color](#named-color), a [system color](#css-system-colors), a [deprecated-color](#deprecated-system-colors), or [transparent](#valdef-color-transparent) therefore, for the [declared value](https://www.w3.org/TR/css-cascade-5/#declared-value), the [ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase) keyword value is retained. For the computed and used value, the corresponding sRGB value is used.

Tests

- [system-color-compute.html](https://wpt.fyi/results/css/css-color/system-color-compute.html) [(live test)](http://wpt.live/css/css-color/system-color-compute.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/system-color-compute.html)

<a id="ref-for-valdef-color-transparent⑦"></a>

Thus, the serialized declared value of [transparent](#valdef-color-transparent) is the string "transparent", while the serialized computed value of <a id="ref-for-valdef-color-transparent⑧"></a>transparent is the string "rgba(0, 0, 0, 0)".

For all other sRGB values, the declared, computed and used value is the corresponding sRGB value.

<a id="ref-for-missing-color-component③④"></a>

<a id="ref-for-legacy-color-syntax⑨"></a>

<a id="ref-for-valdef-color-none①②"></a>

During serialization, any [missing](#missing-color-component) values are converted to 0 if the chosen serialization form (such as the [legacy color syntax](#legacy-color-syntax) with comma separators, or the [HTML-compatible serialization](#HTML-compatible-serialization-of-srgb) of sRGB values) cannot represent the [none](#valdef-color-none) keyword. When at least one component is <a id="ref-for-missing-color-component③⑤"></a>missing and the value can be serialized in a form which supports <a id="ref-for-valdef-color-none①③"></a>none, the form is chosen as described in [§ 16.2.2 CSS serialization of sRGB values](#css-serialization-of-srgb) so that <a id="ref-for-missing-color-component③⑥"></a>missing color components are preserved as <a id="ref-for-valdef-color-none①④"></a>none.

#### <a id="HTML-compatible-serialization-of-srgb"></a>16.2.1. HTML-compatible serialization of sRGB values

If the following conditions are all true:

1.  The color space is sRGB
2.  The alpha is 1
3.  The RGB component values are internally represented as integers between 0 and 255 inclusive (i.e. 8-bit unsigned integer)
4.  <a id="color-serialization-html-compatible-serialization-is-requested"></a>HTML-compatible serialization is requested

<a id="ref-for-hex-color⑥"></a>

Then corresponding sRGB values are serialized in 6-digit [hex color notation](#hex-color) as follows:

A seven-character string consisting of the character "#", followed immediately by the two-digit hexadecimal representations of the red component, the green component, and the blue component, in that order, using [ASCII lower hex digits](https://infra.spec.whatwg.org/#ascii-lower-hex-digit). No spaces are permitted.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-canvas-srgb"></a> For example, fill style is set to magenta:
>
> ```js
> context.fillStyle = "rgb(255, 0, 255)"
> console.log(context.fillStyle); // "#ff00ff"
> ```
>
> <a id="ref-for-valdef-color-none①⑤"></a>
>
> The color space is sRGB, the representation is 8 bits per component, the data format does not produce [none](#valdef-color-none) values nor does it support extended range values, and the alpha is 1.
>
> The HTML-compatible serialization is the string "#ff00ff" (not "#FF00FF").

<a id="ref-for-typedef-color②⑦"></a>

Otherwise, for sRGB the [CSS serialization of sRGB values is used](#css-serialization-of-srgb) and for other color spaces, the relevant [serialization](#serializing-color-values) of the [\<color\>](#typedef-color) value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-canvas-p3"></a> For example, fill style is set to a dark brown, in CIE Lab:
>
> ```js
> context.fillStyle = "lab(29% 39 20)";
> console.log(context.fillStyle); // "lab(29 39 20)"
> ```
>
> The CSS serialization is the string "lab(29 39 20)".

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-srgb-alpha"></a> For example, fill style is set to semi-transparent magenta:
>
> ```js
> context.fillStyle = "#ff00ffed";
> console.log(context.fillStyle); // "rgba(255, 0, 255, 0.93)"
> ```
>
> The alpha is not 1, so the CSS serialization is the string "rgba(255, 0, 255, 0.93)".

#### <a id="css-serialization-of-srgb"></a>16.2.2. CSS serialization of sRGB values

<a id="ref-for-missing-color-component③⑦"></a>

<a id="ref-for-funcdef-rgb①⑦"></a>

<a id="ref-for-funcdef-rgba①②"></a>

If the value has no [missing color components](#missing-color-component), corresponding sRGB values use either the [rgb()](#funcdef-rgb) or [rgba()](#funcdef-rgba) form (depending on whether the (clamped) alpha is exactly 1, or not), with all [ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase) letters for the function name.

<a id="ref-for-number-value③⑥"></a>

<a id="ref-for-percentage-value③⑥"></a>

For compatibility, the sRGB component values are serialized in [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) form, not [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value). Also for compatibility, the component values are serialized in base 10, with a range of \[0-255\], regardless of the bit depth with which they are stored.

<a id="ref-for-funcdef-rgb①⑧"></a>

<a id="ref-for-funcdef-rgba①③"></a>

[As noted earlier](#serializing-alpha-values), unitary alpha values are not explicitly serialized. Also, for compatibility, if the alpha is exactly 1, the [rgb()](#funcdef-rgb) form is used, with an implicit alpha; otherwise, the [rgba()](#funcdef-rgba) form is used, with an explicit alpha value.

<a id="ref-for-funcdef-rgba①④"></a>

For compatibility, the legacy form with comma separators is used; exactly one ASCII space follows each comma. This includes the comma (not slash) used to separate the blue component of [rgba()](#funcdef-rgba) from the alpha value.

<a id="ref-for-legacy-color-syntax①⓪"></a>

<a id="ref-for-valdef-color-none①⑥"></a>

<a id="ref-for-missing-color-component③⑧"></a>

<a id="ref-for-declared-value③"></a>

However, the [legacy color syntax](#legacy-color-syntax) with comma separators cannot represent [none](#valdef-color-none). If the value has at least one [missing color component](#missing-color-component), the serialization form is chosen to preserve those components as the <a id="ref-for-valdef-color-none①⑦"></a>none keyword, based on the color function of the [declared value](https://www.w3.org/TR/css-cascade-5/#declared-value):

- <a id="ref-for-funcdef-rgb①⑨"></a>

  <a id="ref-for-funcdef-rgba①⑤"></a>

  <a id="ref-for-valdef-color-none①⑧"></a>

  <a id="ref-for-hex-color⑦"></a>

  <a id="ref-for-named-color⑦"></a>

  <a id="ref-for-css-system-colors⑦"></a>

  <a id="ref-for-valdef-color-transparent⑨"></a>

  <a id="ref-for-missing-color-component③⑨"></a>

  <a id="ref-for-funcdef-color①⓪"></a>

  <a id="ref-for-valdef-color-srgb①①"></a>

  <a id="ref-for-color-space①①"></a>

  <a id="ref-for-number-value③⑦"></a>

  For [rgb()](#funcdef-rgb) and [rgba()](#funcdef-rgba) values (the only sRGB form in this list whose syntax accepts [none](#valdef-color-none); [hex colors](#hex-color), [named colors](#named-color), [system colors](#css-system-colors), [deprecated-colors](#deprecated-system-colors), and [transparent](#valdef-color-transparent) have no parametric syntax and so never have [missing color components](#missing-color-component)), the value is serialized as a [color()](#funcdef-color) function in the [srgb](#valdef-color-srgb) [color space](#color-space) rather than as the modern space-separated form of <a id="ref-for-funcdef-rgb②⓪"></a>rgb(), even though that form would also accept <a id="ref-for-valdef-color-none①⑨"></a>none: "color(srgb" followed by a single space, followed by a space-separated list of the three non-alpha components serialized as [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s in the \[0, 1\] reference range (or as <a id="ref-for-valdef-color-none②⓪"></a>none if <a id="ref-for-missing-color-component④⓪"></a>missing), followed (only if the alpha is non-unity or <a id="ref-for-missing-color-component④①"></a>missing) by " / " and the alpha component (serialized per the [alpha rules](#serializing-alpha-values), or as <a id="ref-for-valdef-color-none②①"></a>none if <a id="ref-for-missing-color-component④②"></a>missing), followed by ")".

- <a id="ref-for-funcdef-hsl②⓪"></a>

  <a id="ref-for-funcdef-hsla①④"></a>

  <a id="ref-for-number-value③⑧"></a>

  <a id="ref-for-percentage-value③⑦"></a>

  <a id="ref-for-missing-color-component④③"></a>

  <a id="ref-for-valdef-color-none②②"></a>

  For [hsl()](#funcdef-hsl) and [hsla()](#funcdef-hsla) values, the value is serialized using the modern (whitespace-separated) <a id="ref-for-funcdef-hsl②①"></a>hsl() syntax, with a slash before the alpha component when present. The function name is "hsl" (in [ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase)) regardless of whether the value was authored using the <a id="ref-for-funcdef-hsla①⑤"></a>hsla() alias. The hue is serialized as a canonicalized [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) in degrees, the saturation and lightness as [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s, and the alpha (included only if non-unity or [missing](#missing-color-component)) per the [alpha rules](#serializing-alpha-values); any <a id="ref-for-missing-color-component④④"></a>missing component is serialized as the [none](#valdef-color-none) keyword.

- <a id="ref-for-funcdef-hwb①③"></a>

  <a id="ref-for-number-value③⑨"></a>

  <a id="ref-for-percentage-value③⑧"></a>

  <a id="ref-for-missing-color-component④⑤"></a>

  <a id="ref-for-valdef-color-none②③"></a>

  For [hwb()](#funcdef-hwb) values, the value is serialized using the modern (whitespace-separated) <a id="ref-for-funcdef-hwb①④"></a>hwb() syntax, with a slash before the alpha component when present. The function name is "hwb" in [ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase). The hue is serialized as a canonicalized [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) in degrees, the whiteness and blackness as [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s, and the alpha (included only if non-unity or [missing](#missing-color-component)) per the [alpha rules](#serializing-alpha-values); any <a id="ref-for-missing-color-component④⑥"></a>missing component is serialized as the [none](#valdef-color-none) keyword.

<a id="ref-for-funcdef-hsl②②"></a>

<a id="ref-for-funcdef-hwb①⑤"></a>

<a id="ref-for-valdef-color-none②④"></a>

<a id="ref-for-funcdef-rgba①⑥"></a>

<a id="ref-for-funcdef-rgb②①"></a>

<a id="ref-for-color-space①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: this means that an [hsl()](#funcdef-hsl) or [hwb()](#funcdef-hwb) value containing [none](#valdef-color-none) round-trips through serialization in its own color function, rather than degrading to [rgba()](#funcdef-rgba) (whose legacy form cannot represent <a id="ref-for-valdef-color-none②⑤"></a>none), while an [rgb()](#funcdef-rgb) value containing <a id="ref-for-valdef-color-none②⑥"></a>none is serialized via color(srgb …). The modern space-separated form of <a id="ref-for-funcdef-rgb②②"></a>rgb() could itself represent <a id="ref-for-valdef-color-none②⑦"></a>none, but the [sRGB CSS serialization](#serializing-color-values) uses color(srgb …) instead for consistency with how all other [color spaces](#color-space) are serialized in their non-legacy form. This parallels the behavior of relative color syntax defined in [CSS Color 5 § 11.2 Serializing Origin Colors](https://www.w3.org/TR/css-color-5/#serial-origin-color).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-rgb-ser-int-rgba"></a>
>
> For example, the serialized value of
>
> ```css
>  rgb(29 164 192 / 95%)
> ```
>
> is the string "rgba(29, 164, 192, 0.95)"

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-hwb-serial"></a> For example, an author-supplied value:
>
> ```css
>  hwb(740deg 20% 30% / 50%)
> ```
>
> Would be normalized first to
>
> ```css
>  hwb(20 20% 30% / 50%)
> ```
>
> and then converted to sRGB and serialized as
>
> ```css
>  rgba(178.5, 93.5, 51, 0.5)
> ```
>
> The precision of the returned result is [described below](#sRGB-precision).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-hwb-serial-none"></a> For example, the author-supplied value
>
> ```css
> hwb(20 none 30% / none)
> ```
>
> <a id="ref-for-missing-color-component④⑦"></a>
>
> <a id="ref-for-valdef-color-none②⑧"></a>
>
> <a id="ref-for-funcdef-rgba①⑦"></a>
>
> <a id="ref-for-funcdef-hwb①⑥"></a>
>
> contains [missing color components](#missing-color-component) (both the whiteness and the alpha are [none](#valdef-color-none)), so it is <em>not</em> serialized through [rgba()](#funcdef-rgba). Instead, it is serialized using the modern [hwb()](#funcdef-hwb) syntax as
>
> ```css
> hwb(20 none 30% / none)
> ```
>
> <a id="ref-for-valdef-color-none②⑨"></a>
>
> preserving each [none](#valdef-color-none) value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-rgb-serial-none"></a> Similarly, the author-supplied value
>
> ```css
> rgb(none 0 0)
> ```
>
> is serialized as
>
> ```css
> color(srgb none 0 0)
> ```
>
> <a id="ref-for-funcdef-rgb②③"></a>
>
> <a id="ref-for-valdef-color-none③⓪"></a>
>
> because [rgb()](#funcdef-rgb) (in its serialized legacy comma form) cannot represent [none](#valdef-color-none).

<a id="ref-for-funcdef-rgb②④"></a>

<a id="ref-for-number-value④⓪"></a>

<a id="ref-for-integer-value"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: contrary to CSS Color 3, the parameters of the [rgb()](#funcdef-rgb) function are of type [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), not [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value). Thus, any higher precision than eight bits is indicated with a fractional part.

<a id="sRGB-precision"></a> The precision with which sRGB component values are retained, and thus the number of significant figures in the serialized value, is not defined in this specification, but must at least be sufficient to round-trip eight bit values. Values must be [rounded towards +∞](https://drafts.csswg.org/css-values-4/#combine-integers), not truncated.

<a id="ref-for-integer-value①"></a>

<a id="ref-for-number-value④①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: authors of scripts which expect color values returned from getComputedStyle to have [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value) component values, are advised to update them to also cope with [\<number\>](https://www.w3.org/TR/css-values-4/#number-value).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-rgb-number"></a>
>
> For example,
>
> ```css
>  rgb(146.064 107.457 131.223)
> ```
>
> is now valid, and equal to
>
> ```css
>  rgb(57.28% 42.14% 51.46%)
> ```
>
> A conformant serialized form for both, is the string "rgb(146.06, 107.46, 131.2)".

Trailing fractional zeroes in any component values must be omitted; if the fractional part consists of all zeroes, the decimal point must also be omitted. This means that sRGB colors specified with integer component values will serialize with backwards-compatible integer values.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-rgb-notrail"></a>
>
> The serialized computed value of
>
> ```css
>  ''goldenrod''
> ```
>
> is the string "rgb(218, 165, 32)" and not the string "rgb(218.000, 165.000, 32.000)"

### <a id="serializing-lab-lch"></a>16.3.  Serializing Lab and LCH values

<a id="ref-for-funcdef-lch⑦"></a>

<a id="ref-for-funcdef-lab⑥"></a>

<a id="ref-for-computed-value②"></a>

The serialized form of [lch()](#funcdef-lch) and [lab()](#funcdef-lab) values is derived from the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) and uses the <a id="ref-for-funcdef-lab⑦"></a>lab() or <a id="ref-for-funcdef-lch⑧"></a>lch() forms, with [ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase) letters for the function name.

<a id="ref-for-number-value④②"></a>

The component values are serialized in base 10; the L, a, b and C component values are serialized as [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), using the [Lab percentage reference ranges](#prr-lab) or the [LCH percentage reference ranges](#prr-lch) as appropriate to perform percentage to number conversion; thus 0% L maps to 0 and 100% L maps to 100. A single ASCII space character " " must be used as the separator between the component values.

Tests

- [color-computed.html](https://wpt.fyi/results/css/css-color/parsing/color-computed.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-lab-zero-a"></a>
>
> The serialized value of
>
> ```css
>  lab(56.200% 0.000 83.600)
> ```
>
> is the string "lab(56.2 0 83.6)"

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-lab-percent-b"></a>
>
> The serialized value of
>
> ```css
>  lab(56.200% 0.000 66.88%)
> ```
>
> is the string "lab(56.2 0 83.6)"

Trailing fractional zeroes in any component values must be omitted; if the fractional part consists of all zeroes, the decimal point must also be omitted.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-lch-serial"></a>
>
> The serialized value of
>
> ```css
>  lch(37% 105.0 305.00)
> ```
>
> is the string "lch(37 105 305)", not "lch(37 105.0 305.00)".

<a id="ref-for-funcdef-lab⑧"></a>

The precision with which [lab()](#funcdef-lab) component values are retained, and thus the number of significant figures in the serialized value, is not defined in this specification, but due to the wide gamut must be sufficient to round-trip L values between 0 and 100, and a and b values between ±127, with at least sixteen bit precision; this will result in at least three decimal places unless trailing zeroes have been omitted. (half float or float, is recommended for internal storage). Values must be [rounded towards +∞](https://drafts.csswg.org/css-values-4/#combine-integers), not truncated.

<a id="ref-for-valdef-color-prophoto-rgb①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: a and b values outside ±125 are possible with ultrawide gamut spaces. For example, <em>all</em> of the [prophoto-rgb](#valdef-color-prophoto-rgb) primaries and secondaries exceed this range, but are within ±200.

[As noted earlier](#serializing-alpha-values), unitary alpha values are not explicitly serialized. Non-unitary alpha values must be explicitly serialized, and the string " / " (an ASCII space, then forward slash, then another space) must be used to separate the b component value from the alpha value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-lch-alpha"></a>
>
> The serialized value of
>
> ```css
>  lch(56.2% 83.6 357.4 /93%)
> ```
>
> is the string "lch(56.2 83.6 357.4 / 0.93)" not "lch(56.2% 83.6 357.4 / 0.93)"

### <a id="serializing-oklab-oklch"></a>16.4.  Serializing Oklab and OkLCh values

<a id="ref-for-funcdef-oklch⑦"></a>

<a id="ref-for-funcdef-oklab⑥"></a>

<a id="ref-for-computed-value③"></a>

The serialized form of [oklch()](#funcdef-oklch) and [oklab()](#funcdef-oklab) values is derived from the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) and uses the <a id="ref-for-funcdef-oklab⑦"></a>oklab() or <a id="ref-for-funcdef-oklch⑧"></a>oklch() forms, with [ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase) letters for the function name.

<a id="ref-for-number-value④③"></a>

The component values are serialized in base 10; the L, a, b and C component values are serialized as [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) using the [Oklab percentage reference ranges](#prr-oklab) or the [OkLCh percentage reference ranges](#prr-oklch) as appropriate to perform percentage to number conversion; thus 0% L maps to 0 and 100% L maps to 1.0. A single ASCII space character " " must be used as the separator between the component values.

Tests

- [color-computed.html](https://wpt.fyi/results/css/css-color/parsing/color-computed.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-oklab-trailzero"></a>
>
> The serialized value of
>
> ```css
>      oklab(54.0% -0.10 -0.02)
> ```
>
> is the string "oklab(0.54 -0.1 -0.02)" not "oklab(54 -0.1 -0.02)" or "oklab(54% -0.1 -0.02)"

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-oklab-percent-a-b"></a>
>
> The serialized value of
>
> ```css
>      oklab(54.0 -25% -5%)
> ```
>
> is the string "oklab(0.54 -0.1 -0.02)" not "oklab(54 -0.25 -0.05)"

Trailing fractional zeroes in any component values must be omitted; if the fractional part consists of all zeroes, the decimal point must also be omitted.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-oklch-serial"></a>
>
> The serialized value of
>
> ```css
>  oklch(56.43% 0.0900 123.40)
> ```
>
> is the string "oklch(0.5643 0.09 123.4)", not "oklch(0.5643 0.0900 123.40)".

<a id="ref-for-funcdef-oklab⑧"></a>

The precision with which [oklab()](#funcdef-oklab) component values are retained, and thus the number of significant figures in the serialized value, is not defined in this specification, but due to the wide gamut must be sufficient to round-trip L values between 0 and 1 (0% and 100%), and a, b and C values between ±0.5, with at least sixteen bit precision; this will result in at least five decimal places unless trailing zeroes have been omitted. (half float or float, is recommended for internal storage). Values must be [rounded towards +∞](https://drafts.csswg.org/css-values-4/#combine-integers), not truncated.

<a id="ref-for-valdef-color-prophoto-rgb①③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: a, b and C values outside ±0.5 are possible with ultrawide gamut spaces. For example, the [prophoto-rgb](#valdef-color-prophoto-rgb) green and blue primaries exceed this range, with C of 0.526 and 1.413 respectively.

[As noted earlier](#serializing-alpha-values), unitary alpha values are not explicitly serialized. Non-unitary alpha values must be explicitly serialized, and the string " / " (an ASCII space, then forward slash, then another space) must be used to separate the final color component (b, or C) value from the alpha value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-oklch-alpha"></a>
>
> The serialized value of
>
> ```css
>  oklch(53.85% 0.1725 320.67 / 70%)
> ```
>
> is the string "oklch(0.5385 0.1725 320.67 / 0.7)"

<a id="ref-for-funcdef-color①①"></a>

### <a id="serializing-color-function-values"></a>16.5.  Serializing values of the [color()](#funcdef-color) function

<a id="ref-for-funcdef-color①②"></a>

<a id="ref-for-computed-value④"></a>

The serialized form of [color()](#funcdef-color) values is derived from the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) and uses the <a id="ref-for-funcdef-color①③"></a>color() form, with [ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase) letters for the function name and the color space name.

<a id="ref-for-number-value④④"></a>

The component values are serialized in base 10, as [\<number\>](https://www.w3.org/TR/css-values-4/#number-value). A single ASCII space character " " must be used as the separator between the component values, and also between the color space name and the first color component.

Tests

- [color-computed.html](https://wpt.fyi/results/css/css-color/parsing/color-computed.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-color-serial"></a>
>
> The serialized value of
>
> ```css
>  color(dIsPlAy-P3  0.964  0.763  0.787)
> ```
>
> is the string "color(display-p3 0.96 0.76 0.79)", if two decimal places are retained. Notice that 0.787 has rounded up to 0.79, rather than being truncated to 0.78.

Trailing fractional zeroes in any component values must be omitted; if the fractional part consists of all zeroes, the decimal point must also be omitted.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-color-trailzero"></a>
>
> The serialized value of
>
> ```css
>  color(rec2020 0.400 0.660 0.340)
> ```
>
> is the string "color(rec2020 0.4 0.66 0.34)", not "color(rec2020 0.400 0.660 0.340)".

If the color space is sRGB, the color space is still explicitly required in the serialized result.

For the predefined color spaces, the <em>minimum</em> precision for round-tripping is as follows:

<a id="predefined-precision-table"></a>



| color space                                                                                                                                           | Minimum bits |
|-------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|
| <a id="ref-for-valdef-color-srgb①②"></a>[srgb](#valdef-color-srgb)                                                                                                         | 10           |
| <a id="ref-for-valdef-color-srgb-linear⑦"></a>[srgb-linear](#valdef-color-srgb-linear)                                                                                           | 12           |
| <a id="ref-for-valdef-color-display-p3①①"></a>[display-p3](#valdef-color-display-p3)                                                                                             | 10           |
| display-p3-linear                                                                                                                                     | 12           |
| <a id="ref-for-valdef-color-a98-rgb⑤"></a>[a98-rgb](#valdef-color-a98-rgb)                                                                                                   | 10           |
| <a id="ref-for-valdef-color-prophoto-rgb①④"></a>[prophoto-rgb](#valdef-color-prophoto-rgb)                                                                                         | 12           |
| <a id="ref-for-valdef-color-rec2020⑦"></a>[rec2020](#valdef-color-rec2020)                                                                                                   | 12           |
| <a id="ref-for-valdef-color-xyz-d65⑤"></a><a id="ref-for-valdef-color-xyz-d50③"></a><a id="ref-for-valdef-color-xyz⑥"></a>[xyz](#valdef-color-xyz), [xyz-d50](#valdef-color-xyz-d50), [xyz-d65](#valdef-color-xyz-d65) | 16           |



(16bit, half-float, or float <em>per component</em> is recommended for internal storage). Values must be [rounded towards +∞](https://drafts.csswg.org/css-values-4/#combine-integers), not truncated.

<a id="ref-for-funcdef-rgb②⑤"></a>

<a id="ref-for-funcdef-hsl②③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: compared to the legacy forms such as [rgb()](#funcdef-rgb), [hsl()](#funcdef-hsl) and so on, color(srgb) has a higher minimum precision requirement. Stylesheet authors who prefer higher precision are thus encouraged to use the color(srgb) form.

[As noted earlier](#serializing-alpha-values), unitary alpha values are not explicitly serialized. Non-unitary alpha values must be explicitly serialized, and the string " / " (an ASCII space, then forward slash, then another space) must be used to separate the final color component value from the alpha value.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-color-prophoto-alpha-serial"></a>
>
> The serialized value of
>
> ```css
>  color(prophoto-rgb 0.2804 0.40283 0.42259/85%)
> ```
>
> is the string "color(prophoto-rgb 0.28 0.403 0.423 / 0.85)", if three decimal places are retained.

### <a id="serializing-other-colors"></a>16.6.  Serializing other colors

<a id="ref-for-valdef-color-currentcolor①③"></a>

This applies to [currentcolor](#valdef-color-currentcolor).

<a id="ref-for-computed-value⑤"></a>

The serialized form of this value is derived from the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) and uses [ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase) letters for the color name.

<a id="ref-for-valdef-color-currentcolor①④"></a>

The serialized form of [currentColor](#valdef-color-currentcolor) is the string "currentcolor".

<a id="ref-for-typedef-opacity-opacity-value④"></a>

## <a id="serializing-opacity-values"></a>17.  Serializing [\<opacity-value\>](#typedef-opacity-opacity-value)

This applies to the opacity property.

<a id="ref-for-typedef-percentage-token"></a>

<a id="ref-for-funcdef-calc②"></a>

<a id="ref-for-number-value④⑤"></a>

If the specified value for an opacity value matches a literal [\<percentage-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-percentage-token) (i.e. does not use [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc)) it should be serialized as the equivalent [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) (0% maps to 0, 100% maps to 1) value. value. Otherwise, the specified value for an opacity value should serialize using the standard serialization for the grammar.

<a id="ref-for-number-value④⑥"></a>

This [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) value is expressed in base ten, with the "." character as decimal separator. The leading zero must not be omitted. Trailing zeroes must be omitted.

Opacity values outside the range \[0,1\] are preserved, without clamping, in the serialized specified value.

The precision with which opacity values are retained, and thus the number of decimal places in the serialized value, is not defined in this specification, but must at least be sufficient to round-trip integer percentage values. Thus, the serialized value must contain at least two decimal places (unless trailing zeroes have been removed). Values must be [rounded towards +∞](https://drafts.csswg.org/css-values-4/#combine-integers), not truncated.

## <a id="sample"></a>18.  Default Style Rules

The following stylesheet is informative, not normative. This style sheet could be used by an implementation as part of its default styling of HTML documents.

```css
/* traditional desktop user agent colors for hyperlinks */
:link { color: LinkText; }
:visited { color: VisitedText; }
:active { color: ActiveText; }
```
## <a id="color-conversion-code"></a>19.  Sample code for Color Conversions

<em>This section is not normative.</em>

Tests

This section is not normative, it does not need tests.

------------------------------------------------------------------------

For clarity, [a library](https://www.w3.org/TR/2026/CRD-css-color-4-20260502/multiply-matrices.js) is used for matrix multiplication. (This is more readable than inlining all the multiplies and adds). The matrices are in [column-major order](https://www.scratchapixel.com/lessons/mathematics-physics-for-computer-graphics/geometry/row-major-vs-column-major-vector).

```javascript
// Sample code for color conversions
// Conversion can also be done using ICC profiles and a Color Management System
// For clarity, a library is used for matrix multiplication (multiply-matrices.js)

// standard white points, defined by 4-figure CIE x,y chromaticities
const D50 = [0.3457 / 0.3585, 1.00000, (1.0 - 0.3457 - 0.3585) / 0.3585];
const D65 = [0.3127 / 0.3290, 1.00000, (1.0 - 0.3127 - 0.3290) / 0.3290];

// sRGB-related functions

function lin_sRGB(RGB) {
	// convert an array of sRGB values
	// where in-gamut values are in the range [0 - 1]
	// to linear light (un-companded) form.
	// https://en.wikipedia.org/wiki/SRGB
	// Extended transfer function:
	// for negative values,  linear portion is extended on reflection of axis,
	// then reflected power function is used.
	return RGB.map(function (val) {
		let sign = val < 0? -1 : 1;
		let abs = Math.abs(val);

		if (abs <= 0.04045) {
			return val / 12.92;
		}

		return sign * (Math.pow((abs + 0.055) / 1.055, 2.4));
	});
}

function gam_sRGB(RGB) {
	// convert an array of linear-light sRGB values in the range 0.0-1.0
	// to gamma corrected form
	// https://en.wikipedia.org/wiki/SRGB
	// Extended transfer function:
	// For negative values, linear portion extends on reflection
	// of axis, then uses reflected pow below that
	return RGB.map(function (val) {
		let sign = val < 0? -1 : 1;
		let abs = Math.abs(val);

		if (abs > 0.0031308) {
			return sign * (1.055 * Math.pow(abs, 1/2.4) - 0.055);
		}

		return 12.92 * val;
	});
}

function lin_sRGB_to_XYZ(rgb) {
	// convert an array of linear-light sRGB values to CIE XYZ
	// using sRGB's own white, D65 (no chromatic adaptation)

	var M = [
		[ 506752 / 1228815,  87881 / 245763,   12673 /   70218 ],
		[  87098 /  409605, 175762 / 245763,   12673 /  175545 ],
		[   7918 /  409605,  87881 / 737289, 1001167 / 1053270 ],
	];
	return multiplyMatrices(M, rgb);
}

function XYZ_to_lin_sRGB(XYZ) {
	// convert XYZ to linear-light sRGB

	var M = [
		[   12831 /   3959,    -329 /    214, -1974 /   3959 ],
		[ -851781 / 878810, 1648619 / 878810, 36519 / 878810 ],
		[     705 /  12673,   -2585 /  12673,   705 /    667 ],
	];

	return multiplyMatrices(M, XYZ);
}

//  display-p3-related functions


function lin_P3(RGB) {
	// convert an array of display-p3 RGB values in the range 0.0 - 1.0
	// to linear light (un-companded) form.

	return lin_sRGB(RGB);	// same as sRGB
}

function gam_P3(RGB) {
	// convert an array of linear-light display-p3 RGB  in the range 0.0-1.0
	// to gamma corrected form

	return gam_sRGB(RGB);	// same as sRGB
}

function lin_P3_to_XYZ(rgb) {
	// convert an array of linear-light display-p3 values to CIE XYZ
	// using  D65 (no chromatic adaptation)
	// http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
	var M = [
		[ 608311 / 1250200, 189793 / 714400,  198249 / 1000160 ],
		[  35783 /  156275, 247089 / 357200,  198249 / 2500400 ],
		[      0 /       1,  32229 / 714400, 5220557 / 5000800 ],
	];

	return multiplyMatrices(M, rgb);
}

function XYZ_to_lin_P3(XYZ) {
	// convert XYZ to linear-light P3
	var M = [
		[ 446124 / 178915, -333277 / 357830, -72051 / 178915 ],
		[ -14852 /  17905,   63121 /  35810,    423 /  17905 ],
		[  11844 / 330415,  -50337 / 660830, 316169 / 330415 ],
	];

	return multiplyMatrices(M, XYZ);
}

// prophoto-rgb functions

function lin_ProPhoto(RGB) {
	// convert an array of prophoto-rgb values
	// where in-gamut colors are in the range [0.0 - 1.0]
	// to linear light (un-companded) form.
	// Transfer curve is gamma 1.8 with a small linear portion
	// Extended transfer function
	const Et2 = 16/512;
	return RGB.map(function (val) {
		let sign = val < 0? -1 : 1;
		let abs = Math.abs(val);

		if (abs <= Et2) {
			return val / 16;
		}

		return sign * Math.pow(abs, 1.8);
	});
}

function gam_ProPhoto(RGB) {
	// convert an array of linear-light prophoto-rgb  in the range 0.0-1.0
	// to gamma corrected form
	// Transfer curve is gamma 1.8 with a small linear portion
	// TODO for negative values, extend linear portion on reflection of axis, then add pow below that
	const Et = 1/512;
	return RGB.map(function (val) {
		let sign = val < 0? -1 : 1;
		let abs = Math.abs(val);

		if (abs >= Et) {
			return sign * Math.pow(abs, 1/1.8);
		}

		return 16 * val;
	});
}

function lin_ProPhoto_to_XYZ(rgb) {
	// convert an array of linear-light prophoto-rgb values to CIE D50 XYZ
	// matrix cannot be expressed in rational form, but is calculated to 64 bit accuracy
	// see https://github.com/w3c/csswg-drafts/issues/7675
	var M = [
		[ 0.79776664490064230,  0.13518129740053308,  0.03134773412839220 ],
		[ 0.28807482881940130,  0.71183523424187300,  0.00008993693872564 ],
		[ 0.00000000000000000,  0.00000000000000000,  0.82510460251046020 ]
	];

	return multiplyMatrices(M, rgb);
}

function XYZ_to_lin_ProPhoto(XYZ) {
	// convert D50 XYZ to linear-light prophoto-rgb
	var M = [
		[  1.34578688164715830, -0.25557208737979464, -0.05110186497554526 ],
        [ -0.54463070512490190,  1.50824774284514680,  0.02052744743642139 ],
        [  0.00000000000000000,  0.00000000000000000,  1.21196754563894520 ]
	];

	return multiplyMatrices(M, XYZ);
}

// a98-rgb functions

function lin_a98rgb(RGB) {
	// convert an array of a98-rgb values in the range 0.0 - 1.0
	// to linear light (un-companded) form.
	// negative values are also now accepted
	return RGB.map(function (val) {
		let sign = val < 0? -1 : 1;
		let abs = Math.abs(val);

	  	return sign * Math.pow(abs, 563/256);
	});
}

function gam_a98rgb(RGB) {
	// convert an array of linear-light a98-rgb  in the range 0.0-1.0
	// to gamma corrected form
	// negative values are also now accepted
	return RGB.map(function (val) {
		let sign = val < 0? -1 : 1;
		let abs = Math.abs(val);

		return sign * Math.pow(abs, 256/563);
	});
}

function lin_a98rgb_to_XYZ(rgb) {
	// convert an array of linear-light a98-rgb values to CIE XYZ
	// http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
	// has greater numerical precision than section 4.3.5.3 of
	// https://www.adobe.com/digitalimag/pdfs/AdobeRGB1998.pdf
	// but the values below were calculated from first principles
	// from the chromaticity coordinates of R G B W
	// see matrixmaker.html
	var M = [
		[ 573536 /  994567,  263643 / 1420810,  187206 /  994567 ],
		[ 591459 / 1989134, 6239551 / 9945670,  374412 / 4972835 ],
		[  53769 / 1989134,  351524 / 4972835, 4929758 / 4972835 ],
	];

	return multiplyMatrices(M, rgb);
}

function XYZ_to_lin_a98rgb(XYZ) {
	// convert XYZ to linear-light a98-rgb
	var M = [
		[ 1829569 /  896150, -506331 /  896150, -308931 /  896150 ],
		[ -851781 /  878810, 1648619 /  878810,   36519 /  878810 ],
		[   16779 / 1248040, -147721 / 1248040, 1266979 / 1248040 ],
	];

	return multiplyMatrices(M, XYZ);
}

//Rec. 2020-related functions

function lin_2020(RGB) {
	// convert an array of rec2020 RGB values in the range 0.0 - 1.0
	// to linear light (un-companded) form.
	//  Reference electro-optical transfer function from Rec. ITU-R BT.1886 Annex 1
	//  with b (black lift) = 0 and a (user gain) = 1
	//  defined over the extended range, not clamped

	return RGB.map(function (val) {
		let sign = val < 0? -1 : 1;
		let abs = Math.abs(val);
		return sign * Math.pow(abs, 2.4);
	});
}

function gam_2020(RGB) {
	// convert an array of linear-light rec2020 RGB  in the range 0.0-1.0
	// to gamma corrected form
	//  Reference electro-optical transfer function from Rec. ITU-R BT.1886 Annex 1
	//  with b (black lift) = 0 and a (user gain) = 1
	//  defined over the extended range, not clamped

	return RGB.map(function (val) {
		let sign = val < 0? -1 : 1;
		let abs = Math.abs(val);
		return sign * Math.pow(abs, 1 / 2.4);
	});
}

function lin_2020_to_XYZ(rgb) {
	// convert an array of linear-light rec2020 values to CIE XYZ
	// using  D65 (no chromatic adaptation)
	var M = [
		[ 63426534 / 99577255,  20160776 / 139408157,  47086771 / 278816314 ],
		[ 26158966 / 99577255, 472592308 / 697040785,   8267143 / 139408157 ],
		[        0 /        1,  19567812 / 697040785, 295819943 / 278816314 ],
	];
	// 0 is actually calculated as  4.994106574466076e-17

	return multiplyMatrices(M, rgb);
}

function XYZ_to_lin_2020(XYZ) {
	// convert XYZ to linear-light rec2020
	var M = [
		[  30757411 / 17917100, -6372589 / 17917100, -4539589 / 17917100 ],
		[ -19765991 / 29648200, 47925759 / 29648200,   467509 / 29648200 ],
		[    792561 / 44930125, -1921689 / 44930125, 42328811 / 44930125 ],
	];

	return multiplyMatrices(M, XYZ);
}

// Chromatic adaptation

function D65_to_D50(XYZ) {
	// Bradford chromatic adaptation from D65 to D50
	// The matrix below is the result of three operations:
	// - convert from XYZ to retinal cone domain
	// - scale components from one reference white to another
	// - convert back to XYZ
	// see https://github.com/LeaVerou/color.js/pull/354/files
	
	var M =  [
		[  1.0479297925449969,    0.022946870601609652,  -0.05019226628920524  ],
		[  0.02962780877005599,   0.9904344267538799,    -0.017073799063418826 ],
		[ -0.009243040646204504,  0.015055191490298152,   0.7518742814281371   ]
	];

	return multiplyMatrices(M, XYZ);
}

function D50_to_D65(XYZ) {
	// Bradford chromatic adaptation from D50 to D65
	// See https://github.com/LeaVerou/color.js/pull/360/files
	var M = [
		[  0.955473421488075,    -0.02309845494876471,   0.06325924320057072  ],
		[ -0.0283697093338637,    1.0099953980813041,    0.021041441191917323 ],
		[  0.012314014864481998, -0.020507649298898964,  1.330365926242124    ]
	];

	return multiplyMatrices(M, XYZ);
}

// CIE Lab and LCH

function XYZ_to_Lab(XYZ) {
	// Assuming XYZ is relative to D50, convert to CIE Lab
	// from CIE standard, which now defines these as a rational fraction
	var ε = 216/24389;  // 6^3/29^3
	var κ = 24389/27;   // 29^3/3^3

	// compute xyz, which is XYZ scaled relative to reference white
	var xyz = XYZ.map((value, i) => value / D50[i]);

	// now compute f
	var f = xyz.map(value => value > ε ? Math.cbrt(value) : (κ * value + 16)/116);

	return [
		(116 * f[1]) - 16, 	 // L
		500 * (f[0] - f[1]), // a
		200 * (f[1] - f[2])  // b
	];
	// L in range [0,100]. For use in CSS, add a percent
}

function Lab_to_XYZ(Lab) {
	// Convert Lab to D50-adapted XYZ
	// http://www.brucelindbloom.com/index.html?Eqn_Lab_to_XYZ.html
	var κ = 24389/27;   // 29^3/3^3
	var ε = 216/24389;  // 6^3/29^3
	var f = [];

	// compute f, starting with the luminance-related term
	f[1] = (Lab[0] + 16)/116;
	f[0] = Lab[1]/500 + f[1];
	f[2] = f[1] - Lab[2]/200;

	// compute xyz
	var xyz = [
		Math.pow(f[0],3) > ε ?   Math.pow(f[0],3)            : (116*f[0]-16)/κ,
		Lab[0] > κ * ε ?         Math.pow((Lab[0]+16)/116,3) : Lab[0]/κ,
		Math.pow(f[2],3)  > ε ?  Math.pow(f[2],3)            : (116*f[2]-16)/κ
	];

	// Compute XYZ by scaling xyz by reference white
	return xyz.map((value, i) => value * D50[i]);
}

function Lab_to_LCH(Lab) {
	var epsilon = 0.0015;
	var chroma = Math.sqrt(Math.pow(Lab[1], 2) + Math.pow(Lab[2], 2)); // Chroma
	var hue = Math.atan2(Lab[2], Lab[1]) * 180 / Math.PI;
	if (hue < 0) {
		hue = hue + 360;
	}
	if (chroma <= epsilon) {
		hue = NaN;
	}
	return [
		Lab[0], // L is still L
		chroma, // Chroma
		hue // Hue, in degrees [0 to 360)
	];
}

function LCH_to_Lab(LCH) {
	// Convert from polar form
	return [
		LCH[0], // L is still L
		LCH[1] * Math.cos(LCH[2] * Math.PI / 180), // a
		LCH[1] * Math.sin(LCH[2] * Math.PI / 180) // b
	];
}

// OKLab and OKLCH
// https://bottosson.github.io/posts/oklab/

// XYZ <-> LMS matrices recalculated for consistent reference white
// see https://github.com/w3c/csswg-drafts/issues/6642#issuecomment-943521484
// recalculated for 64bit precision
// see https://github.com/color-js/color.js/pull/357

function XYZ_to_OKLab(XYZ) {
	// Given XYZ relative to D65, convert to OKLab
	var XYZtoLMS = [
		[ 0.8190224379967030, 0.3619062600528904, -0.1288737815209879 ],
		[ 0.0329836539323885, 0.9292868615863434,  0.0361446663506424 ],
		[ 0.0481771893596242, 0.2642395317527308,  0.6335478284694309 ]
	];
	var LMStoOKLab = [
		[ 0.2104542683093140,  0.7936177747023054, -0.0040720430116193 ],
		[ 1.9779985324311684, -2.4285922420485799,  0.4505937096174110 ],
		[ 0.0259040424655478,  0.7827717124575296, -0.8086757549230774 ]
	];

	var LMS = multiplyMatrices(XYZtoLMS, XYZ);
	// JavaScript Math.cbrt returns a sign-matched cube root
	// beware if porting to other languages
	// especially if tempted to use a general power function
	return multiplyMatrices(LMStoOKLab, LMS.map(c => Math.cbrt(c)));
	// L in range [0,1]. For use in CSS, multiply by 100 and add a percent
}

function OKLab_to_XYZ(OKLab) {
	// Given OKLab, convert to XYZ relative to D65
	var LMStoXYZ =  [
		[  1.2268798758459243, -0.5578149944602171,  0.2813910456659647 ],
		[ -0.0405757452148008,  1.1122868032803170, -0.0717110580655164 ],
		[ -0.0763729366746601, -0.4214933324022432,  1.5869240198367816 ]
	];
	var OKLabtoLMS = [
		[ 1.0000000000000000,  0.3963377773761749,  0.2158037573099136 ],
		[ 1.0000000000000000, -0.1055613458156586, -0.0638541728258133 ],
		[ 1.0000000000000000, -0.0894841775298119, -1.2914855480194092 ]
    ];

	var LMSnl = multiplyMatrices(OKLabtoLMS, OKLab);
	return multiplyMatrices(LMStoXYZ, LMSnl.map(c => c ** 3));
}

function OKLab_to_OKLCH(OKLab) {
	var epsilon = 0.000004;
	var hue = Math.atan2(OKLab[2], OKLab[1]) * 180 / Math.PI;
	var chroma = Math.sqrt(OKLab[1] ** 2 + OKLab[2] ** 2);
	if (hue < 0) {
		hue = hue + 360;
	}
	if (chroma <= epsilon) {
		hue = NaN;
	}
	return [
		OKLab[0], // L is still L
		chroma,
		hue
	];
}

function OKLCH_to_OKLab(OKLCH) {
	return [
		OKLCH[0], // L is still L
		OKLCH[1] * Math.cos(OKLCH[2] * Math.PI / 180), // a
		OKLCH[1] * Math.sin(OKLCH[2] * Math.PI / 180)  // b
	];
}

// Premultiplied alpha conversions

function rectangular_premultiply(color, alpha) {
// given a color in a rectangular orthogonal colorspace
// and an alpha value
// return the premultiplied form
	return color.map((c) => c * alpha)
}

function rectangular_un_premultiply(color, alpha) {
// given a premultiplied color in a rectangular orthogonal colorspace
// and an alpha value
// return the actual color
	if (alpha === 0) {
		return color; // avoid divide by zero
	}
	return color.map((c) => c / alpha)
}

function polar_premultiply(color, alpha, hueIndex) {
	// given a color in a cylindicalpolar colorspace
	// and an alpha value
	// return the premultiplied form.
	// the index says which entry in the color array corresponds to hue angle
	// for example, in OKLCH it would be 2
	// while in HSL it would be 0
	return color.map((c, i) => c * (hueIndex === i? 1 : alpha))
}

function polar_un_premultiply(color, alpha, hueIndex) {
	// given a color in a cylindicalpolar colorspace
	// and an alpha value
	// return the actual color.
	// the hueIndex says which entry in the color array corresponds to hue angle
	// for example, in OKLCH it would be 2
	// while in HSL it would be 0
	if (alpha === 0) {
		return color; // avoid divide by zero
	}
	return color.map((c, i) => c / (hueIndex === i? 1 : alpha))
}

// Convenience functions can easily be defined, such as
function hsl_premultiply(color, alpha) {
	return polar_premultiply(color, alpha, 0);
}
```
## <a id="color-difference-code"></a>20.  Sample Code for ΔE2000 and ΔEOK Color Differences

<em>This section is not normative.</em>

Tests

This section is not normative, it does not need tests.

------------------------------------------------------------------------

### <a id="color-difference-2000"></a>20.1. ΔE2000

The simplest color difference metric, ΔE76, is simply the Euclidean distance in Lab color space. While this is a good first approximation, color-critical industries such as printing and fabric dyeing soon developed improved formulae. Currently, the most widely used formula is ΔE2000. It corrects a number of known asymmetries and non-linearities compared to ΔE76. Because the formula is complex, and critically dependent on the sign of various intermediate calculations, implementations are often incorrect [\[Sharma\]](#biblio-sharma).

The sample code below has been [validated](https://colorjs.io/test/?test=delta) to five significant figures against the test suite of paired Lab values and expected ΔE2000 published by [\[Sharma\]](#biblio-sharma) and is correct.

```javascript
// deltaE2000 is a statistically significant improvement
// over deltaE76 and deltaE94,
// and is recommended by the CIE and Idealliance
// especially for color differences less than 10 deltaE76
// but is wicked complicated
// and many implementations have small errors!

/**
 * @param {number[]} reference - Array of CIE Lab values: L as 0..100, a and b as around -150..150
 * @param {number[]} sample - Array of CIE Lab values: L as 0..100, a and b as around -150..150
 * @return {number} How different a color sample is from reference
 */

function deltaE2000 (reference, sample) {

    // Given a reference and a sample color,
    // both in CIE Lab,
    // calculate deltaE 2000.

    // This implementation assumes the parametric
    // weighting factors kL, kC and kH
    // (for the influence of viewing conditions)
    // are all 1, as seems typical.

    let [L1, a1, b1] = reference;
    let [L2, a2, b2] = sample;
    let C1 = Math.sqrt(a1 ** 2 + b1 ** 2);
    let C2 = Math.sqrt(a2 ** 2 + b2 ** 2);

	let Cbar = (C1 + C2)/2; // mean Chroma

	// calculate a-axis asymmetry factor from mean Chroma
	// this turns JND ellipses for near-neutral colors back into circles
	let C7 = Math.pow(Cbar, 7);
	const Gfactor = Math.pow(25, 7);
	let G = 0.5 * (1 - Math.sqrt(C7/(C7+Gfactor)));

	// scale a axes by asymmetry factor
	// this by the way is why there is no Lab2000 color space
	let adash1 = (1 + G) * a1;
	let adash2 = (1 + G) * a2;

	// calculate new Chroma from scaled a and original b axes
	let Cdash1 = Math.sqrt(adash1 ** 2 + b1 ** 2);
	let Cdash2 = Math.sqrt(adash2 ** 2 + b2 ** 2);

	// calculate new hues, with zero hue for true neutrals
	// and in degrees, not radians
	const π = Math.PI;
	const r2d = 180 / π;
	const d2r = π / 180;
	let h1 = (adash1 === 0 && b1 === 0)? 0: Math.atan2(b1, adash1);
	let h2 = (adash2 === 0 && b2 === 0)? 0: Math.atan2(b2, adash2);

	if (h1 < 0) {
		h1 += 2 * π;
	}
	if (h2 < 0) {
		h2 += 2 * π;
	}

	h1 *= r2d;
	h2 *= r2d;

	// Lightness and Chroma differences; sign matters
	let ΔL = L2 - L1;
	let ΔC = Cdash2 - Cdash1;

	// Hue difference, taking care to get the sign correct
	let hdiff = h2 - h1;
	let hsum = h1 + h2;
	let habs = Math.abs(hdiff);
	let Δh;

	if (Cdash1 * Cdash2 === 0) {
		Δh = 0;
	}
	else if (habs <= 180) {
		Δh = hdiff;
	}
	else if (hdiff > 180) {
		Δh = hdiff - 360;
	}
	else if (hdiff < -180) {
		Δh = hdiff + 360;
	}
	else {
		console.log("the unthinkable has happened");
	}

	// weighted Hue difference, more for larger Chroma
	let ΔH = 2 * Math.sqrt(Cdash2 * Cdash1) * Math.sin(Δh * d2r / 2);

	// calculate mean Lightness and Chroma
	let Ldash = (L1 + L2)/2;
	let Cdash = (Cdash1 + Cdash2)/2;
	let Cdash7 = Math.pow(Cdash, 7);

	// Compensate for non-linearity in the blue region of Lab.
	// Four possibilities for hue weighting factor,
	// depending on the angles, to get the correct sign
	let hdash;
	if (Cdash1 * Cdash2 === 0) {
		hdash = hsum;
	}
	else if (habs <= 180) {
		hdash = hsum / 2;
	}
	else if (hsum < 360) {
		hdash = (hsum + 360) / 2;
	}
	else {
		hdash = (hsum - 360) / 2;
	}

	// positional corrections to the lack of uniformity of CIELAB
	// These are all trying to make JND ellipsoids more like spheres

	// SL Lightness crispening factor
	// a background with L=50 is assumed
	let lsq = (Ldash - 50) ** 2;
	let SL = 1 + ((0.015 * lsq) / Math.sqrt(20 + lsq));

	// SC Chroma factor, similar to those in CMC and deltaE 94 formulae
	let SC = 1 + 0.045 * Cdash;

	// Cross term T for blue non-linearity
	let T = 1;
	T -= (0.17 * Math.cos((     hdash - 30)  * d2r));
	T += (0.24 * Math.cos(  2 * hdash        * d2r));
	T += (0.32 * Math.cos(((3 * hdash) + 6)  * d2r));
	T -= (0.20 * Math.cos(((4 * hdash) - 63) * d2r));

	// SH Hue factor depends on Chroma,
	// as well as adjusted hue angle like deltaE94.
	let SH = 1 + 0.015 * Cdash * T;

	// RT Hue rotation term compensates for rotation of JND ellipses
	// and Munsell constant hue lines
	// in the medium-high Chroma blue region
	// (Hue 225 to 315)
	let Δθ = 30 * Math.exp(-1 * (((hdash - 275)/25) ** 2));
	let RC = 2 * Math.sqrt(Cdash7/(Cdash7 + Gfactor));
	let RT = -1 * Math.sin(2 * Δθ * d2r) * RC;

	// Finally calculate the deltaE, term by term as root sum of squares
	let dE = (ΔL / SL) ** 2;
	dE += (ΔC / SC) ** 2;
	dE += (ΔH / SH) ** 2;
	dE += RT * (ΔC / SC) * (ΔH / SH);
	return Math.sqrt(dE);
	// Yay!!!
};
```
### <a id="color-difference-OK"></a>20.2. ΔEOK

Because Oklab does not suffer from the hue linearity, hue uniformity, and chroma non-linearities of CIE Lab, the color difference metric does not need to correct for them and so is simply the Euclidean distance in Oklab color space.

```javascript
// Calculate deltaE OK
// simple root sum of squares
/**
 * @param {number[]} reference - Array of OKLab values: L as 0..1, a and b as -1..1
 * @param {number[]} sample - Array of OKLab values: L as 0..1, a and b as -1..1
 * @return {number} How different a color sample is from reference
 */
function deltaEOK (reference, sample) {
    let [L1, a1, b1] = reference;
	let [L2, a2, b2] = sample;
	let ΔL = L1 - L2;
	let Δa = a1 - a2;
	let Δb = b1 - b2;
	return Math.sqrt(ΔL ** 2 + Δa ** 2 + Δb ** 2);
}
```
## <a id="deprecated-system-colors"></a> Appendix A: Deprecated CSS System Colors

<a id="ref-for-css-system-colors⑧"></a>

Earlier versions of CSS defined several additional [system colors](#css-system-colors). These color keywords have been <strong>deprecated</strong>, however, as they are insufficient for their original purpose (making website elements look like their native OS counterparts), represent a security risk by making it easier for a webpage to “spoof” a native OS dialog, and increase fingerprinting surface, compromising user privacy.

<a id="ref-for-css-system-colors⑨"></a>

User agents must support these keywords, and to mitigate fingerprinting must map them to the (undeprecated) [system colors](#css-system-colors) as listed below. <strong>Authors must not use these keywords.</strong>

<a id="ref-for-typedef-deprecated-color②"></a>

The deprecated system colors are represented as the <a id="typedef-deprecated-color"></a>[\<deprecated-color\>](#typedef-deprecated-color) sub-type, and are defined as:

<a id="valdef-color-activeborder"></a>ActiveBorder  
<a id="ref-for-valdef-color-buttonborder②"></a>

Active window border. Same as [ButtonBorder](#valdef-color-buttonborder).

<a id="valdef-color-activecaption"></a>ActiveCaption  
<a id="ref-for-valdef-color-canvas⑤"></a>

Active window caption. Same as [Canvas](#valdef-color-canvas).

<a id="valdef-color-appworkspace"></a>AppWorkspace  
<a id="ref-for-valdef-color-canvas⑥"></a>

Background color of multiple document interface. Same as [Canvas](#valdef-color-canvas).

<a id="valdef-color-background"></a>Background  
<a id="ref-for-valdef-color-canvas⑦"></a>

Desktop background. Same as [Canvas](#valdef-color-canvas).

<a id="valdef-color-buttonhighlight"></a>ButtonHighlight  
<a id="ref-for-valdef-color-buttonface②"></a>

The color of the border facing the light source for 3-D elements that appear 3-D due to one layer of surrounding border. Same as [ButtonFace](#valdef-color-buttonface).

<a id="valdef-color-buttonshadow"></a>ButtonShadow  
<a id="ref-for-valdef-color-buttonface③"></a>

The color of the border away from the light source for 3-D elements that appear 3-D due to one layer of surrounding border. Same as [ButtonFace](#valdef-color-buttonface).

<a id="valdef-color-captiontext"></a>CaptionText  
<a id="ref-for-valdef-color-canvastext①"></a>

Text in caption, size box, and scrollbar arrow box. Same as [CanvasText](#valdef-color-canvastext).

<a id="valdef-color-inactiveborder"></a>InactiveBorder  
<a id="ref-for-valdef-color-buttonborder③"></a>

Inactive window border. Same as [ButtonBorder](#valdef-color-buttonborder).

<a id="valdef-color-inactivecaption"></a>InactiveCaption  
<a id="ref-for-valdef-color-canvas⑧"></a>

Inactive window caption. Same as [Canvas](#valdef-color-canvas).

<a id="valdef-color-inactivecaptiontext"></a>InactiveCaptionText  
<a id="ref-for-valdef-color-graytext①"></a>

Color of text in an inactive caption. Same as [GrayText](#valdef-color-graytext).

<a id="valdef-color-infobackground"></a>InfoBackground  
<a id="ref-for-valdef-color-canvas⑨"></a>

Background color for tooltip controls. Same as [Canvas](#valdef-color-canvas).

<a id="valdef-color-infotext"></a>InfoText  
<a id="ref-for-valdef-color-canvastext②"></a>

Text color for tooltip controls. Same as [CanvasText](#valdef-color-canvastext).

<a id="valdef-color-menu"></a>Menu  
<a id="ref-for-valdef-color-canvas①⓪"></a>

Menu background. Same as [Canvas](#valdef-color-canvas).

<a id="valdef-color-menutext"></a>MenuText  
<a id="ref-for-valdef-color-canvastext③"></a>

Text in menus. Same as [CanvasText](#valdef-color-canvastext).

<a id="valdef-color-scrollbar"></a>Scrollbar  
<a id="ref-for-valdef-color-canvas①①"></a>

Scroll bar gray area. Same as [Canvas](#valdef-color-canvas).

<a id="valdef-color-threeddarkshadow"></a>ThreeDDarkShadow  
<a id="ref-for-valdef-color-buttonborder④"></a>

The color of the darker (generally outer) of the two borders away from the light source for 3-D elements that appear 3-D due to two concentric layers of surrounding border. Same as [ButtonBorder](#valdef-color-buttonborder).

<a id="valdef-color-threedface"></a>ThreeDFace  
<a id="ref-for-valdef-color-buttonface④"></a>

The face background color for 3-D elements that appear 3-D due to two concentric layers of surrounding border. Same as [ButtonFace](#valdef-color-buttonface).

<a id="valdef-color-threedhighlight"></a>ThreeDHighlight  
<a id="ref-for-valdef-color-buttonborder⑤"></a>

The color of the lighter (generally outer) of the two borders facing the light source for 3-D elements that appear 3-D due to two concentric layers of surrounding border. Same as [ButtonBorder](#valdef-color-buttonborder).

<a id="valdef-color-threedlightshadow"></a>ThreeDLightShadow  
<a id="ref-for-valdef-color-buttonborder⑥"></a>

The color of the darker (generally inner) of the two borders facing the light source for 3-D elements that appear 3-D due to two concentric layers of surrounding border. Same as [ButtonBorder](#valdef-color-buttonborder).

<a id="valdef-color-threedshadow"></a>ThreeDShadow  
<a id="ref-for-valdef-color-buttonborder⑦"></a>

The color of the lighter (generally inner) of the two borders away from the light source for 3-D elements that appear 3-D due to two concentric layers of surrounding border. Same as [ButtonBorder](#valdef-color-buttonborder).

<a id="valdef-color-window"></a>Window  
<a id="ref-for-valdef-color-canvas①②"></a>

Window background. Same as [Canvas](#valdef-color-canvas).

<a id="valdef-color-windowframe"></a>WindowFrame  
<a id="ref-for-valdef-color-buttonborder⑧"></a>

Window frame. Same as [ButtonBorder](#valdef-color-buttonborder).

<a id="valdef-color-windowtext"></a>WindowText  
<a id="ref-for-valdef-color-canvastext④"></a>

Text in windows. Same as [CanvasText](#valdef-color-canvastext).

Tests

- [deprecated-sameas-001.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-001.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-001.html)
- [deprecated-sameas-002.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-002.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-002.html)
- [deprecated-sameas-003.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-003.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-003.html)
- [deprecated-sameas-004.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-004.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-004.html)
- [deprecated-sameas-005.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-005.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-005.html)
- [deprecated-sameas-006.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-006.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-006.html)
- [deprecated-sameas-007.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-007.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-007.html)
- [deprecated-sameas-008.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-008.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-008.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-008.html)
- [deprecated-sameas-009.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-009.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-009.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-009.html)
- [deprecated-sameas-010.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-010.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-010.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-010.html)
- [deprecated-sameas-011.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-011.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-011.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-011.html)
- [deprecated-sameas-012.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-012.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-012.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-012.html)
- [deprecated-sameas-013.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-013.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-013.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-013.html)
- [deprecated-sameas-014.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-014.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-014.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-014.html)
- [deprecated-sameas-015.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-015.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-015.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-015.html)
- [deprecated-sameas-016.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-016.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-016.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-016.html)
- [deprecated-sameas-017.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-017.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-017.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-017.html)
- [deprecated-sameas-018.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-018.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-018.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-018.html)
- [deprecated-sameas-019.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-019.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-019.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-019.html)
- [deprecated-sameas-020.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-020.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-020.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-020.html)
- [deprecated-sameas-021.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-021.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-021.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-021.html)
- [deprecated-sameas-022.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-022.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-022.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-022.html)
- [deprecated-sameas-023.html](https://wpt.fyi/results/css/css-color/deprecated-sameas-023.html) [(live test)](http://wpt.live/css/css-color/deprecated-sameas-023.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/deprecated-sameas-023.html)

## <a id="quirky-color"></a> Appendix B: Deprecated Quirky Hex Colors

<a id="ref-for-concept-document-quirks"></a>

<a id="ref-for-typedef-quirky-color"></a>

<a id="ref-for-typedef-color②⑧"></a>

When CSS is being parsed in [quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks), <a id="typedef-quirky-color"></a>[\<quirky-color\>](#typedef-quirky-color) is a type of [\<color\>](#typedef-color) that is only valid in certain properties:

- <a id="ref-for-propdef-background-color"></a>

  [background-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-color)

- <a id="ref-for-propdef-border-color①"></a>

  [border-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-color)

- <a id="ref-for-propdef-border-top-color"></a>

  [border-top-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-top-color)

- <a id="ref-for-propdef-border-right-color"></a>

  [border-right-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-right-color)

- <a id="ref-for-propdef-border-bottom-color"></a>

  [border-bottom-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-bottom-color)

- <a id="ref-for-propdef-border-left-color"></a>

  [border-left-color](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-left-color)

- <a id="ref-for-propdef-color⑧"></a>

  [color](#propdef-color)

<a id="ref-for-propdef-background"></a>

<a id="ref-for-functional-notation①"></a>

<a id="ref-for-funcdef-color-mix②"></a>

It is <em>not</em> valid in properties that include or reference these properties, such as the [background](https://www.w3.org/TR/css-backgrounds-3/#propdef-background) shorthand, or inside [functional notations](https://www.w3.org/TR/css-values-4/#functional-notation) such as [color-mix()](https://www.w3.org/TR/css-color-5/#funcdef-color-mix)

<a id="ref-for-typedef-quirky-color①"></a>

<a id="ref-for-typedef-color②⑨"></a>

<a id="ref-for-at-ruledef-supports"></a>

<a id="ref-for-dom-css-supports-conditiontext"></a>

Additionally, while [\<quirky-color\>](#typedef-quirky-color) must be valid as a [\<color\>](#typedef-color) when parsing the affected properties in the [@supports](https://www.w3.org/TR/css-conditional-3/#at-ruledef-supports) rule, it is <em>not</em> valid for those properties when used in the <code><a href="https://www.w3.org/TR/css-conditional-3/#dom-css-supports-conditiontext">CSS.supports()</a></code> method.

<a id="ref-for-typedef-quirky-color②"></a>

<a id="ref-for-typedef-number-token"></a>

<a id="ref-for-typedef-dimension-token"></a>

<a id="ref-for-typedef-ident-token"></a>

A [\<quirky-color\>](#typedef-quirky-color) can be represented as a [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token), [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token), or [\<ident-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-ident-token), according to the following rules:

- <a id="ref-for-typedef-ident-token①"></a>

  <a id="ref-for-typedef-hex-color①"></a>

  If it’s an [\<ident-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-ident-token), the token’s representation must contain exactly 3 or 6 characters, all hexadecimal digits. It represents a [\<hex-color\>](#typedef-hex-color) with the same value.

- <a id="ref-for-typedef-number-token①"></a>

  If it’s a [\<number-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-number-token), it must have its integer flag set.

  <a id="ref-for-typedef-hex-color②"></a>

  Serialize the integer’s value. If the serialization has less than 6 characters, prepend "0" characters to it until it is 6 characters long. It represents a [\<hex-color\>](#typedef-hex-color) with the same value.

- <a id="ref-for-typedef-dimension-token①"></a>

  If it’s a [\<dimension-token\>](https://www.w3.org/TR/css-syntax-3/#typedef-dimension-token), it must have its integer flag set.

  <a id="ref-for-typedef-hex-color③"></a>

  Serialize the integer’s value, and append the representation of the token’s unit. If the result has less than 6 characters, prepend "0" characters to it until it is 6 characters long. It represents a [\<hex-color\>](#typedef-hex-color) with the same value.

(In other words, Quirks Mode allows hex colors to be written without the leading "#", but with weird parsing rules.)

Tests

quirky hex colors

------------------------------------------------------------------------

## <a id="acknowledgments"></a> Acknowledgments

In addition to [those who contributed to CSS Color 3](https://www.w3.org/TR/css-color-3/#acknowledgments), the editors would like to thank Emilio Cobos Álvarez, Alexey Ardov, Chris Bai, Amelia Bellamy-Royds, Lars Borg, Mike Bremford, Andreu Botella, Dan Burzo, Max Derhak, fantasai, Simon Fraser, Devon Govett, Phil Green, Dean Jackson, Andreas Kraushaar, Pierre-Anthony Lemieux, Tiaan Louw, Cameron McCormack, Romain Menke, Chris Murphy, Isaac Muse, Jonathan Neal, Chris Needham, Björn Ottosson, Christoph Päper, Brad Pettit, Xidorn Quan, Craig Revie, Melanie Richards, Florian Rivoal, Jacob Rus, Joseph Salowey, Simon Sapin, Igor Snitkin, Lea Verou, Mark Watson, James Stuckey Weber, Sam Weinig, and Natalie Weizenbaum.

## <a id="changes"></a> Changes

### <a id="changes-from-20250424"></a>Changes since the [Candidate Recommendation Draft of 24 April 2025](https://www.w3.org/TR/2025/CRD-css-color-4-20250424/)

- For color comparisons in Oklab, standardized ε to be <strong>0.00001</strong> ([Issue 13157](https://github.com/w3c/csswg-drafts/issues/13157#issuecomment-4329625711))

- <a id="ref-for-valdef-oklab-oklab⑤"></a>

  <a id="ref-for-missing-color-component④⑧"></a>

  <a id="ref-for-equivalent-colors②"></a>

  <a id="ref-for-typedef-color③⓪"></a>

  Added a new section defining when two [\<color\>](#typedef-color) values are [equivalent colors](#equivalent-colors), covering same-color-space component comparison, the treatment of [missing component](#missing-color-component)s, and cross-color-space comparison via [oklab](#valdef-oklab-oklab). ([Issue 13157](https://github.com/w3c/csswg-drafts/issues/13157))

- Clarified in the main Color interpolation section that, if the hue interpolaton method is not specified, shorter is the default. (This was already specified in the Hue Interpolation section). ([Issue 13788](https://github.com/w3c/csswg-drafts/issues/13788))

- <a id="ref-for-valdef-color-none③①"></a>

  Expanded the concept of analogous components to analogous sets of components, to minimize [none](#valdef-color-none) → 0 conversions ([Issue 10210](https://github.com/w3c/csswg-drafts/issues/10210))

- Split color conversion into two stages ([Issue 10211](https://github.com/w3c/csswg-drafts/issues/10211))

- Clarified how system colors react to the used color scheme ([Issue 13719](https://github.com/w3c/csswg-drafts/issues/13719))

- Updated abstract to mention color interpolation and gamut mapping.

- Clarified wording regarding the aims of CSS gamut mapping

- Corrected ray trace algorithm to not overwrite <i>end</i> ([Issue 10579](https://github.com/w3c/csswg-drafts/issues/10579))

- Added pseudocode for the ray trace gamut mapping algorithm ([Issue 10579](https://github.com/w3c/csswg-drafts/issues/10579))

- Added EdgeSeeker and Ray Trace Gamut Mapping Algorithms. Allowed choice of three GMA ([Issue 10579](https://github.com/w3c/csswg-drafts/issues/10579))

- Added a diagram showing imaginary colors in CIE Lab

- Differentiated between out of gamut but physically realizable colors, and imaginary colors

- More even-handed description of clipping, showing some cases which give acceptable results ([Issue 10579](https://github.com/w3c/csswg-drafts/issues/10579))

- Fixed discrepency in ΔE2000 sample implementation ([Issue 13322](https://github.com/w3c/csswg-drafts/issues/13322))

- <a id="ref-for-forced-colors-mode③"></a>

  <a id="ref-for-propdef-accent-color①"></a>

  <a id="ref-for-valdef-color-accentcolor③"></a>

  Updated [AccentColor](#valdef-color-accentcolor) to take its value from [accent-color](https://www.w3.org/TR/css-ui-4/#propdef-accent-color), unless in [Forced Colors Mode](https://www.w3.org/TR/css-color-adjust-1/#forced-colors-mode) ([Issue 5900](https://github.com/w3c/csswg-drafts/issues/5900))

- Defined rec2020 color space to use display-referred, 2.4 gamma ([Issue 12574](https://github.com/w3c/csswg-drafts/issues/12574))

- Added display-p3-linear to predefined colorspaces ([Issue 11250](https://github.com/w3c/csswg-drafts/issues/11250))

- Clarified serializing opacity values with calc() ([Issue 10426](https://github.com/w3c/csswg-drafts/issues/10426))

- Interpolation between legacy sRGB colors is (once again) in sRGB space, for compatibility ([Issue 7949](https://github.com/w3c/csswg-drafts/issues/7949))

- Clarified real-world CIE Lab range for a and b ([Issue 12208](https://github.com/w3c/csswg-drafts/issues/12208))

- Clarified that Opacity value does not affect hit testing ([Issue 11339](https://github.com/w3c/csswg-drafts/issues/11339))

### <a id="changes-from-20240213"></a>Changes since the [Candidate Recommendation Draft of 13 Feb 2024](https://www.w3.org/TR/2024/CRD-css-color-4-20240213/)

- Clarified that inside the color property, it is the resolved inherited value (not the raw inherited value) that is used

- Listed categories of colors, such as those that resolve to sRGB or support legacy color syntax

- Added hue normalization examples

- Corrected table of analogous components, alpha was missing, but described as analogous in prose

- Collected together and clarified serialization of opacity values into one section

- Clarified wording around color-interpolation-method and host syntax

- Defined epsilon for returning missing hue

- Used a more precise definition of achromatic colors with missing hues (sufficiently close to the central axis)

- Consistently use "color component" rather than "color channel" (both were being used).

- Correlated Color Temperature was used without being defined or explained. Added informative reference.

- Exported term premultiplied, linked to it consistently

- Equivalence of deprecated and un-deprecated system colors is no longer at-risk

- <a id="ref-for-typedef-color③①"></a>

  Clarified intended use of the "parse a CSS [\<color\>](#typedef-color)" algorithm

- Added corrected examples for HTML-compatible serialization

- Removed check on missing values for HTML-compatible serialization, they will already have been converted to zero

- Moved note about missing values becoming 0, so it applies to both HTML-compatible and CSS serializations

- Added an HTML-Compatible hex serialization for sRGB

- Added another xyz-d65 and xyz-d50 example

- Clarify which component (Y) in XYZ corresponds to brightness

- Clarified that CSS gamut mapping applies on actual, not used, values

- Removed hue normalization from the hslToRgb sample code, as the input is already normalized at parse time

- Corrected the pseudo-code for step 4 of the gamut mapping algorithm

- Clarified that interpolation is the most common situation which combines two colors, but not the only one.

- Ensured adequate contrast for text in the deltaE table

- Removed remaining use of the term \<absolute-color-function\>, use \<absolute-color\> function instead for consistency

- Updated acknowledgements section

- Added gamut mesh diagram

- Described CSSOM serialization in terms of declared values rather than specified values

- Added exported definition of luminance

- Added production rule for \<opacity-value\>

- Clarified when results from hslToRgb will be in \[0,1\]

- Clarified that once linearized, RGB spaces are additive

### <a id="changes-from-20221101"></a>Changes since the [Candidate Recommendation Draft of 1 November 2022](https://www.w3.org/TR/2022/CRD-css-color-4-20221101/) 

- Added steps for serializing a uint8_t alpha, moved from cssom-1
- Restored parse-time clamping of HSL negative saturation to 0, which is current interop behavior from CSS Color 3
- When interpolating, always convert color space, so that powerless components become missing
- Clarified when alpha 1 is omitted from serialization
- Removed redundant constraining of hue angles to \[0,360\] as this is already done.
- Corrected description of ActiveCaption, which is a background.
- Disambiguated opacity and alpha. Opacity property now uses opacity-value (which has different clamping behavior to alpha-value)
- Clarified that carrying forward happens before premultiplication
- Updated gamut mapping algorithm
- Fixed a few issues regarding hue interpolation
- Clarified that HWB white or black at 100% is insufficient criterion for an achromatic color; it is the sum which matters.
- Avoid returning negative saturation in rgb to hsl conversion; adjust the hue to point to "the other side" instead
- Use 64 bit accurate matrices for ProPhoto, which does not have a rational form
- Oklab matrices recalculated for 64bit precision (returns same results as before, at 32 bit precision)
- Consistently return output of GMA in the destination color space, even if no mapping is performed because the destination is unbounded
- Added explanation for why one JND for Oklab is 0.02, not 2
- Clarified that resolving sRGB values does not apply to the color() function
- Moved alpha value definition up to the opacity property, clarified that opacity specified values are not clamped.
- System Colors now explicitly permit spoofing, to preserve privacy
- Corrected the inverse chromatic adaptation matrix for D50 to D65
- Consistently distinguish linear Bradfrod from the original, more complicated, Bradford chromatic adaptation algorithm
- In the gamut mapping algorithm, return clipped as the gamut mapped result, avoiding un-necessary steps
- Updated chromatic adaptation matrices to higher precision
- Added Add 'parse a css color' algorithm, so non-CSS specs using colors don’t have to reinvent the machinery here.
- Clarified that geometric gamut mapping must not project chroma back beyond the original color
- Use the term "geometric" rather than "analytical" in gamut mapping discussion
- Aligned prose for HSL into line with the grammar (percent and number both allowed)
- Fixed an LCH alpha interpolation example, which was erroneously un-premultiplying the hue angle
- Corrected the sRGB and display-p3 transfer function. (This only affected the result if a component had the exact value 10.31475 / 255, which is not possible at 8 or 10 bits per component)
- Clarified that the specified values of system colors are still themselves
- Added mention of PNG cICP chunk for tagging images
- Described behaviour of hue increasing and decreasing when 0/360 is passed
- Aligned description of powerlessnes in HSL with the other polar color models
- Explicitly defined order of operations for color interpolation
- Added mention of degenerate numeric constants in calc()
- Clarified that calc() in sRGB has early resolution, and clamps the result
- Clarified that HWB hue has the same disadvantages as HSL hue
- Added luminance to lightness comparison and figure
- Added descriptions and examples for hue interpolation keywords
- Use normative prose for achromatic HWB colors
- Corrected hue interpolation angle range; \[0,360) not \[0,360\]
- Expressed that displaying as black or white when L=0% or 100% is due to gamut mapping. Removed incorrect assertions of powerlessness
- Dropped the confusing "representing black" and "representing white" comments
- Clarified that opponent a and b are analogous
- Specified RGB components using reference ranges rather than prose, for consistency
- Explicitly referenced percent reference ranges for percentage to number conversion when serializing Lab, LCH, Oklab, OkLCh
- Required Oklab interpolation, remove previous "may", describe explicit opt-out
- Labelled the Lab, LCH, Oklab and OkLCh tutorial sections as non-normative. Moved some definitions out of the non-normative section.
- Clarified that, when interpolating, checking for analogous components happens before color space conversion
- Back-ported hwb() syntax changes and reference ranges from CSS Color 5
- Defined carry-forward operations must happen before powerless operations
- Clarified it is <em>color</em> components which must be all-number or all-percentage, in legacy rgb() syntax
- Clarified for legacy syntax that color components must be all-percentage or all-number
- Added examples of specified out of range alpha, with and without calc()
- Placed examples of serializing with trimmed trailing zeroes colorer to the relevant text
- clarified example, used value of text-shadow
- Clarified resolving currentColor
- Updated acknowledgments
- Stop claiming that achromatic colors have missing a,b, or chroma
- HSL and HWB changed to unbounded gamut, to promote round-tripping
- Defined percentage reference range for HSL
- Modern color syntax hsl() and hsla() allow mixed number and percentage components
- Modern color syntax rgb() and rgba() allow mixed number and percentage components
- Define the term "modern color syntax" (legacy color syntax already defined).
- Consistently use the term "analogous components"
- Changed to allow all predefined color spaces for interpolation
- Clarified that for color(), three parameters (RGB or XYZ) are required
- Clarified serialization of named colors, system colors, and transparent
- Define specified value for Lab, LCH, Oklab, OkLCh
- Define specified value for other sRGB colors
- Define specified values for named and system colors
- Clamp alpha, Lightness, Chroma and Hue at parsed-value time
- Remove passing mention of specular white and CIE Lightness
- No longer require as-specified Hue to be retained; clamp to \[0, 360\]
- Consistent serialization of Lightness and number in examples
- Minor typos and editorial clarifications

### <a id="changes-from-20220705"></a>Changes since the [Candidate Recommendation of 5 July 2022](https://www.w3.org/TR/2022/CR-css-color-4-20220705/) 

- Removed hue interpolation "specified" value
- Defined hue interpolation angle more precisely, maintaining differences of 360deg
- Added example of carried forward alpha for premultiplication
- Clarified a,b and C,h powerless at L=100% representing white.
- Removed handwavy mention of L=400 which applies to hdr-CIELAB not CIE Lab
- Consistent capitalization of Oklab and OkLCh
- Moved definitions of valid color, invalid color, out of gamut and in gamut to terminology section
- Fixed definition of "longer" hue interpolation
- Further clarified the concept of a host syntax
- Accessibility improvements for color swatches
- Made explicit that legacy forms do not support "none"
- Remove "none" from the hue production, as it is not allowed in legacy syntax
- Removed some dangling references to CMYK and CMYKOGV, moved to CSS Color5
- Clarified how missing values in colors to be interpolated are carried forward
- Updated syntax of xyz-params so they take numbers and percentage, to align with prose
- Ensure all examples and figures have IDs, self-links
- Clarified importance to implementors of reading the gamut mapping introduction
- Removed left-over mention of custom color spaces (feature was moved to CSS Color 5)
- Refactor syntax of \<color\> and \<alpha-value\>
- Editorial refactoring for better reading order.
- Updated pseudocode for gamut mapping algorithm, remove un-needed deltaE calls

### <a id="changes-from-20220628"></a>Changes since the [Working Draft of 28 June 2022](https://www.w3.org/TR/2022/WD-css-color-4-20220628/) 

- Updated status for Candidate Recommendation

### <a id="changes-from-20220428"></a>Changes since the [Working Draft of 28 April 2022](https://www.w3.org/TR/2022/WD-css-color-4-20220428/) 

- Moved opacity property up to the top of the module, next to color property, before getting into details.

- Improved description of the color property, in particular effect on other properties

- Corrected longer hue adjust equation, for equal-modulo-360 colors

- Added two new System colors: AccentColor and AccentColorText

- Described overall color space conversion steps in new section

- <a id="ref-for-valdef-color-none③②"></a>

  Accounted for [none](#valdef-color-none) alpha in premultiplication and un-premultiplication

### <a id="changes-from-20211215"></a>Changes since the [Working Draft of 15 December 2021](https://www.w3.org/TR/2021/WD-css-color-4-20211215/) 

- Made system colors fully resolve, but forbid their alteration in forced colors mode

- Removed forgiveness for incorrect number of parameters in color() function

- Changed serialization of CIE Lightness and OK Lightness to number rather than percentage.

- Marked deprecated system color equivalences as at-risk

- Added reference ranges to percentage values for CIE and OK L,a,b,C

- Noted that there is sample code for performing and undoing premultiplication, for both rectangular and polar color spaces.

- Added out of range clamping to the gamut mapping prose, as well as the pseudocode

- Added normative reference for ProPhoto RGB / ROMM

- Corrected sRGB and Display P3 black point value for reference surround

- Added normative reference for Display P3

- Avoided an infinite loop in gamut reduction, with colors whiter than white or darker than black

- <a id="ref-for-valdef-color-none③③"></a>

  Clarified serialization of the [none](#valdef-color-none) value

- Clarified the opt-in to interpolation in Oklab, for non-legacy colors

- <a id="ref-for-valdef-color-none③④"></a>

  Defined how premultiplication works, with the [none](#valdef-color-none) value

- Clarified that missing values in rgb serialize as 0

- <a id="ref-for-valdef-color-none③⑤"></a>

  Clarified the use of calc() with the [none](#valdef-color-none) value

- Typo, inconsistent casing on System Colors

- Added example of SelectedItem with SelectedItemText

- Explicitly noted the presence or absence of legacy colors

- Added normative reference for CIE XYZ

- Added normative reference for HWB and HSL

- <a id="ref-for-funcdef-hwb①⑦"></a>

  Clarified that [hwb()](#funcdef-hwb) is not a legacy syntax so does not support the older, comma-separated syntactic form

- Clarified that only legacy colors will gamut map, the others are unbounded

- Use distinct terms, spectrophotometer and spectroradiometer

- Assorted minor typos fixed, and grammatical improvements

### <a id="changes-from-20210601"></a>Changes since the [Working Draft of 1 June 2021](https://www.w3.org/TR/2021/WD-css-color-4-20210601/) 

- Added gamut mapping section and defined a CSS gamut mapping algorithm as chroma reduction in OkLCh with local MINDE.

- Computed value of color(xyz ...) is color(xyz-d65 ...)

- Added srgb-linear to interpolation color spaces

- Updated Changes from Colors 3 section

- Added Resolving Oklab and OkLCh values section

- Added srgb-linear color space

- Moved @color-profile and device-cmyk to level 5 per CSSWG resolution

- Defined interpolation color space

- Clarified that matrices are row-major and linked to the matrix multiplication library

- Split old Security &#x26; Privacy section into separate sections

- Defined quirks-mode quirky hex colors

- Removed fallback colors from device-cmyk

- Host syntax that does not declare a default now uses Oklab by default

- Added sample code for deltaE OK

- Added sample conversion code for OKlab and OkLCh

- Added oklab() and oklch() functions <i>Added description of Oklab and OkLCh</i>

- Added description of CIE LCH deficiencies

- <a id="ref-for-valdef-color-none③⑥"></a>

  Allowed all components of a color to be "missing" via the [none](#valdef-color-none) keyword, defined when components are "powerless" and automatically become missing in some cases, and fixed all references to "NaN" components to use the "missing" concept.

- Defined explicit x,y whitepoint values, use consistently throughout

- Defined the term host syntax

- Defined context for resolving override-color colors

- Added a new pair of system colors

- Corrected HSL and HWB sample code

- Replaced table of HSL values with error-free version

- Added Lea Verou as co-editor by WG resolution

- Clarified that hue angle is unbounded

- MarkText example corrected

- Added diagrams, corrected examples

- Some editorial clarifications

- Minor typos corrected, markup corrections

### <a id="changes-from-20201112"></a>Changes since the [Working Draft of 12 November 2020](https://www.w3.org/TR/2020/WD-css-color-4-20201112/) 

- Noted indeterminate hue ssue on near-neutral Lab values converted to LCH

- Clarified which steps are linear combinations in RGB Lab interconversion

- Added components descriptor to @color-profile, for use in CSS Color 5

- All predefined RGB color spaces are defined over the extended range

- Clarified that there is no gamut mapping or gamut clipping step prior to color interpolation

- Clarified interpolation of legacy sRGB syntaxes

- <a id="ref-for-funcdef-color①④"></a>

  Removed the lab option from [color()](#funcdef-color)

- List steps to interconvert between predefined color spaces

- Consistent use of the term color space (two words)

- Provided more guidance on selecting color space for mixing

- Recalculated an example to increase precision

- Added hue interpolation example

- <a id="ref-for-funcdef-color①⑤"></a>

  Simplified [color()](#funcdef-color) syntax by removing the fallback options

- Clarified the types of ICC profile that may be linked from @color-profile

- Support for the rare ICC Named Colors was removed

- Improved precision of standard whitepoint chromaticities

- Removed a trademark from description of one predefined color space

- Rephrased interpolation to be more generic wrt to interpolation space

- Corrected Accessibility Considerations section

- <a id="ref-for-funcdef-color①⑥"></a>

  Clarified that the color space argument for [color()](#funcdef-color) is mandatory, even for sRGB

- Clarified that currentColor is not restricted to sRGB

- Small correction to the sRGB to XYZ to sRGB matrices, improve round-tripping

- Clarified the rec2020 transfer function, citing the correct ITU Rec BT.2020-2 reference

- Correct fallback examples to use the correct syntax

- Don’t force non-legacy colors to interpolate in a gamma-encoded space

- Define premultiplied alpha interpolation

- Start to address interpolation to and from currentColor

- Define hue interpolation with NaN

- Generalize color interpolation

- Define interpolation to be in Lab, with override to LCG

- Corrections to hue interpolation

- Defined hue angle interpolation

- Added interpolation section

- Corrected syntax in some examples

- <a id="ref-for-funcdef-color①⑦"></a>

  Clarify exactly which components are allowed percentages, in [color()](#funcdef-color)

- <a id="ref-for-funcdef-lab⑨"></a>

  <a id="ref-for-funcdef-lch⑨"></a>

  Change to serialize [lch()](#funcdef-lch) as itself rather than as [lab()](#funcdef-lab)

- <a id="ref-for-funcdef-color①⑧"></a>

  Minimum 10 bits per component precision for non-legacy sRGB in [color()](#funcdef-color)

- <a id="ref-for-funcdef-color①⑨"></a>

  color space no longer optional in [color()](#funcdef-color)

- Consistent minimum precision between lab() and color(lab)

- Clarified fallback procedure for the color() function – first valid in-gamut color, else first valid color which is then gamut mapped, else transparent black

- Clarified difference between opacity property and colors with opacity, notably for rendering overlapping text glyphs

- Added sample (but verified correct) code for ΔE2000

- Added definition of previously-undefined term chromaticity, with examples; define chromaticity diagram.

- Added explanation of color additivity, with examples

- Added source links to WPT tests

- Export definition of color, and valid color, for other specifications to reference

- Define minimum number of bits per component, for serialization

- Updated "applies to" definitions (CSS-wide change)

- Added image state (display referred or scene referred) for predefined color spaces

- Listed white point correlated color temperature (e.g. D65) alongside chromaticity coordinates, for clarity

- Clarified that rounding is towards +∞

- Correction of typos, markup corrections, link fixes

### <a id="changes-from-20191105"></a>Changes since [Working Draft of 5 November 2019](https://www.w3.org/TR/2019/WD-css-color-4-20191105/)

- Export some terms for use in other specifications

- Update requirement from WCAG 2.0 to 2.1

- Fully specify Unicode characters used for serialization

- Define serialization of special named colors

- Define serialization of device-cmyk()

- <a id="ref-for-funcdef-color②⓪"></a>

  Define serialization of [color()](#funcdef-color)

- Fully define RGB serialization, in maximally web-compatible way

- Define serialization of Lab and LCH

- Fully define serialization of alpha values

- Consistency pass to avoid accidental RFC2119

- Add IDs to all the examples, to enable referencing

- Separate resolved color and serialized color sections

- (Security) ICC profiles have no executable code

- Define what out-of-range means for profiled colors

- Clarify out-of-range clamping

- Add examples of specified values

- Clarify computed values

- Resist fingerprinting, with mandatory mappings for deprecated system colors

- Added explanatory note on history and reason for standardizing X11 colors

- Correct hwb sample code

- Add table of DeltaE2000 values for MacBeth patches

- Add note on ICC profile Internet Media type

- Add reference to PNG sRGB chunk

- Clarify CMYK to Lab interconversion

- Clarify RGB to Lab interconversion

- More comparison of HSL vs. LCH

- More description for Rec BT.2020 color space

- Updated description of prophoto-rgb

- Removed duplicate "keywords" from Value Definitions section

- Added an example of an invalid color

- Added example with multiple fallbacks

- Assorted typos and markup fixes

- Clarify handling for undeclared custom color spaces

- Clarify some examples and explanatory notes

- Handling for valid and invalid ICC profiles

- Define handling for images with explicit tagged color space

- Define color space for 4k, SDR video

- State that user contrast settings mst take precedence

- Clarify meaning of system colors outside for forced-color mode

- Update default style rules

- <a id="ref-for-funcdef-color②①"></a>

  Add CIE XYZ color space to [color()](#funcdef-color)

- Greater clarity on hue angles, NaN explicitly allowed

- Improve section on system color pairings, require AA accessible contrast

- Warn of interaction between overlapping glyphs and the opacity property

- Correct grammar in color definition

- Improve description of Highlight/HighlightText

- Correct prophoto-rgb transfer function

- More precision for prophoto-rgb primaries

- Started to define "can’t be displayed"

- Removed paragraph about canvas surface

- Added the buttonborder, mark, and marktext system colors

- Added reverse conversion, sRGB to HWB

- Clarified polar spaces are cylindrical, not spherical

- Added an Accessibility Considerations section

- Started to describe chroma-reduction gamut mapping rather than per-component clipping

- Corrected white chromaticity for rec2020

- Made device-cmyk available by @color-profile; updated CMYK to color algorithm to only use naive conversion as a last resort

- Added print-oriented CMYK and KCMYOGV examples

- User-defined color spaces now dashed-ident, making predefined color spaces extensible without clashes

- Added lab option to the color() function

- Added normative reference for CIE Lab

- <a id="ref-for-d50⑨"></a>

  Clarified that prophoto-rgb uses [D50](#d50) whitepoint so does not require adaptation

- Clarified direction of increasing angle in LCH

- Clarified that color names are ASCII case insensitive

- Initial value of the "color" property is now CanvasText

- Removed confusing gray() function per CSS WG resolution

- Collect scattered definitions into new [Color terminology](#terminology) section

- Add helpful figures and more examples

- Minor editorial clarifications, spell check, fixing typos, bikeshed markup fixes

### <a id="changes-from-20160705"></a>Changes since [Working Draft of 05 July 2016](https://www.w3.org/TR/2016/WD-css-color-4-20160705/)

- Changed Lightness in Lab and LCH to be a percentage, for CSS compatibility
- Clamping of color values clarified
- Percentage opacity is now allowed
- Define terms sRGB and linear-light sRGB, for use by other specs
- Add new list of CSS system colors; renaming Text to CanvasText
- Make system color keywords compute to themselves
- Add computed/used entry for system colors
- Rewrite intro to non-deprecated system colors to center their use around forced-colors mode rather than generic use
- Consistent hyphenation of predefined color spaces
- Restore text about non-opaque elements painting at layers even when not positioned
- Initial value of the "color" property is now black
- Clarify hue in LCH is modulo 360deg (change now reverted)
- Clarify allowed range of L in LCH and Lab, and meaning of L=100
- Update references for color spaces used in video
- Add prophoto-rgb predefined color space
- Correct black and white luminance levels for display-p3
- Clarify display-p3 transfer function
- Add a98-rgb color space, correct table of primary chromaticities
- Clarify that currentColor’s computed value is not the resolved color
- Update syntax is examples to conform to latest specification
- Remove the color-mod() function
- Drop the "media" from propdef tables
- Export, and consistently use, "transparent black" and "opaque black"
- Clarify calculated values such as percents
- Clarify required precision and rounding behavior for color components
- Clarify "color" property has no effect on color font glyphs (unless specifically referenced, e.g. with currentColor)
- Clarify how color values are resolved
- Clarify that HSL, HWB and named colors resolve to sRGB
- Simplify conversion from device-cmyk to sRGB
- Describe previous, comma-using color syntaxes as "legacy"; change examples to commaless form
- Remove superfluous requirement that displayed colors be restricted to device gamut (like there was any other option!)
- Rename P3 to display-p3; avoid claiming this is DCI P3, as these are not the same
- Improved description of the parameters to the "color()" function
- Disallow predefined spaces from "@color-profile" identifier
- Add canonical order to "color", "color-adjust" and "opacity" property definitions
- Switch definition of alpha compositing from SVG11 to CSS Compositing
- Clarify sample conversion code is non-normative
- Add Security and Privacy Considerations
- Update several references to most current versions
- Convert inline issues to links to GitHub issues
- Minor editorial clarifications, formatting and markup improvements

### <a id="changes-from-3"></a> Changes from Colors 3

The primary change, compared to CSS Color 3, is that CSS colors are no longer restricted to the narrow gamut of sRGB.

To support this, several brand new features have been added:

1.  predefined, wide color gamut RGB color spaces

2.  <a id="ref-for-funcdef-oklch⑨"></a>

    <a id="ref-for-funcdef-oklab⑨"></a>

    <a id="ref-for-funcdef-lch①⓪"></a>

    <a id="ref-for-funcdef-lab①⓪"></a>

    [lab()](#funcdef-lab), [lch()](#funcdef-lch), [oklab()](#funcdef-oklab) and [oklch()](#funcdef-oklch) functions, for device-independent color

Other technical changes:

1.  <a id="ref-for-typedef-color③②"></a>

    Serialization of [\<color\>](#typedef-color) is now specified here, rather than in the CSS Object Model

2.  <a id="ref-for-funcdef-hwb①⑧"></a>

    [hwb()](#funcdef-hwb) function, for specifying sRGB colors in the HWB notation.

3.  <a id="ref-for-valdef-color-rebeccapurple"></a>

    Addition of named color [rebeccapurple](#valdef-color-rebeccapurple).

In addition, there have been some syntactic changes:

1.  <a id="ref-for-integer-value②"></a>

    <a id="ref-for-number-value④⑦"></a>

    <a id="ref-for-funcdef-rgba①⑧"></a>

    <a id="ref-for-funcdef-rgb②⑥"></a>

    [rgb()](#funcdef-rgb) and [rgba()](#funcdef-rgba) functions now accept [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) rather than [\<integer\>](https://www.w3.org/TR/css-values-4/#integer-value).

2.  <a id="ref-for-number-value④⑧"></a>

    <a id="ref-for-angle-value①"></a>

    <a id="ref-for-funcdef-hsla①⑥"></a>

    <a id="ref-for-funcdef-hsl②④"></a>

    [hsl()](#funcdef-hsl) and [hsla()](#funcdef-hsla) functions now accept [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) as well as [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) for hues.

3.  <a id="ref-for-funcdef-hsla①⑦"></a>

    <a id="ref-for-funcdef-hsl②⑤"></a>

    <a id="ref-for-funcdef-rgba①⑨"></a>

    <a id="ref-for-funcdef-rgb②⑦"></a>

    [rgb()](#funcdef-rgb) and [rgba()](#funcdef-rgba), and [hsl()](#funcdef-hsl) and [hsla()](#funcdef-hsla) are now aliases of each other (all of them have an optional alpha).

4.  <a id="ref-for-funcdef-hsla①⑧"></a>

    <a id="ref-for-funcdef-hsl②⑥"></a>

    <a id="ref-for-funcdef-rgba②⓪"></a>

    <a id="ref-for-funcdef-rgb②⑧"></a>

    [rgb()](#funcdef-rgb), [rgba()](#funcdef-rgba), [hsl()](#funcdef-hsl), and [hsla()](#funcdef-hsla) have all gained a new syntax consisting of space-separated arguments and an optional slash-separated opacity. All the color functions use this syntax form now, in keeping with [CSS’s functional-notation design principles](https://wiki.csswg.org/ideas/functional-notation#general-principles).

5.  <a id="ref-for-number-value④⑨"></a>

    <a id="ref-for-percentage-value③⑨"></a>

    <a id="ref-for-typedef-color-alpha-value②⑥"></a>

    All uses of [\<alpha-value\>](#typedef-color-alpha-value) now accept [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) as well as [\<number\>](https://www.w3.org/TR/css-values-4/#number-value).

6.  4 and 8-digit hex colors have been added, to specify transparency.

7.  <a id="ref-for-valdef-color-none③⑦"></a>

    The [none](#valdef-color-none) value has been added, to represent powerless components.

## <a id="security"></a>21. Security Considerations

The system colors, if they actually correspond to the user’s system colors, pose a security risk, as they make it easier for a malware site to create user interfaces that appear to be from the system. However, as several system colors are now defined to be "generic", this risk is believed to be mitigated.

## <a id="privacy"></a>22. Privacy Considerations

This specification defines "system" colors, which theoretically can expose details of the user’s OS settings, which is a fingerprinting risk.

## <a id="a11y-sec"></a>23. Accessibility Considerations

This specification [encourages authors to not use color alone](#accessibility) as a distinguishing feature.

<a id="ref-for-css-system-colors①①"></a>

This specification [encourages browsers to ensure adequate contrast for specific system color foreground/background pairs](#css-system-colors). A harder requirement with specific AA or AAA contrast ratios was considered, but since browsers are often just passing along color choices made by the OS, or selected by users (who may have particular requirements, including lower contrast for people living with migraines or epileptic seizures), the CSSWG was unable to require a specific contrast level.

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

Advisements are normative sections styled to evoke special attention and are set apart from other normative text with `<strong class="advisement">`, like this: <strong data-conversion-semantic="advisement">Advisement:</strong> <strong>&#xA;        UAs MUST provide an accessible alternative.&#xA;    </strong>

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

- [a98-rgb](#valdef-color-a98-rgb), in § 10.6
- [absolute color](#absolute-color), in § 4.1
- [AccentColor](#valdef-color-accentcolor), in § 6.2
- [AccentColorText](#valdef-color-accentcolortext), in § 6.2
- [ActiveBorder](#valdef-color-activeborder), in § Unnumbered section
- [ActiveCaption](#valdef-color-activecaption), in § Unnumbered section
- [ActiveText](#valdef-color-activetext), in § 6.2
- [additive color space](#additive-color-space), in § 2
- [aliceblue](#valdef-color-aliceblue), in § 6.1
- [alpha channel](#alpha-channel), in § 4
- [alpha component](#alpha-channel), in § 4
- [\<alpha-value\>](#typedef-color-alpha-value), in § 4.2
- [analogous components](#analogous-components), in § 13.2
- [analogous set](#analogous-set), in § 13.2
- [antiquewhite](#valdef-color-antiquewhite), in § 6.1
- [AppWorkspace](#valdef-color-appworkspace), in § Unnumbered section
- [aqua](#valdef-color-aqua), in § 6.1
- [aquamarine](#valdef-color-aquamarine), in § 6.1
- [azure](#valdef-color-azure), in § 6.1
- [Background](#valdef-color-background), in § Unnumbered section
- [beige](#valdef-color-beige), in § 6.1
- [Binary Search Gamut Map with Local MINDE](#binary-search-gamut-map-with-local-minde), in § 14.2.2
- [bisque](#valdef-color-bisque), in § 6.1
- [black](#valdef-color-black), in § 6.1
- [blanchedalmond](#valdef-color-blanchedalmond), in § 6.1
- [blue](#valdef-color-blue), in § 6.1
- [blueviolet](#valdef-color-blueviolet), in § 6.1
- [brown](#valdef-color-brown), in § 6.1
- [burlywood](#valdef-color-burlywood), in § 6.1
- [ButtonBorder](#valdef-color-buttonborder), in § 6.2
- [ButtonFace](#valdef-color-buttonface), in § 6.2
- [ButtonHighlight](#valdef-color-buttonhighlight), in § Unnumbered section
- [ButtonShadow](#valdef-color-buttonshadow), in § Unnumbered section
- [ButtonText](#valdef-color-buttontext), in § 6.2
- [cadetblue](#valdef-color-cadetblue), in § 6.1
- [calibrated](#calibrated), in § 2
- [can be displayed](#can-be-displayed), in § 10.1
- [can’t be displayed](#cant-be-displayed), in § 10.1
- [Canvas](#valdef-color-canvas), in § 6.2
- [CanvasText](#valdef-color-canvastext), in § 6.2
- [CaptionText](#valdef-color-captiontext), in § Unnumbered section
- [carried forward](#carried-forward), in § 13
- [cast a ray](#cast-a-ray), in § 14.2.6
- [characterized](#characterized), in § 2
- [chartreuse](#valdef-color-chartreuse), in § 6.1
- [chocolate](#valdef-color-chocolate), in § 6.1
- [chromatic adaptation transform](#chromatic-adaptation-transform), in § 9.1
- [chromaticity](#chromaticity), in § 2
- [\<color\>](#typedef-color), in § 4.1
- color
  - [(property)](#propdef-color), in § 3.2
  - [definition of](#color), in § 2
- [color()](#funcdef-color), in § 10.1
- [\<color-base\>](#typedef-color-base), in § 4.1
- [\<color-function\>](#typedef-color-function), in § 4.1
- [color functions](#color-functions), in § 4
- [\<color-interpolation-method\>](#color-interpolation-method), in § 13.1
- [\<color-space\>](#typedef-color-space), in § 13.1
- [color space](#color-space), in § 2
- [\<colorspace-params\>](#typedef-colorspace-params), in § 10.1
- [computed color](#computed-color), in § 15
- [convert a color](#convert-a-color), in § 11
- [coral](#valdef-color-coral), in § 6.1
- [cornflowerblue](#valdef-color-cornflowerblue), in § 6.1
- [cornsilk](#valdef-color-cornsilk), in § 6.1
- [crimson](#valdef-color-crimson), in § 6.1
- [css gamut mapped](#css-gamut-mapped), in § 14.2
- [CSS gamut mapping algorithms](#css-gamut-mapping-algorithms), in § 14.2
- [currentcolor](#valdef-color-currentcolor), in § 6.4
- [cyan](#valdef-color-cyan), in § 6.1
- [cylindrical polar color](#cylindrical-polar-color), in § 4
- [D50](#d50), in § 2
- [D65](#d65), in § 2
- [darkblue](#valdef-color-darkblue), in § 6.1
- [darkcyan](#valdef-color-darkcyan), in § 6.1
- [darkgoldenrod](#valdef-color-darkgoldenrod), in § 6.1
- [darkgray](#valdef-color-darkgray), in § 6.1
- [darkgreen](#valdef-color-darkgreen), in § 6.1
- [darkgrey](#valdef-color-darkgrey), in § 6.1
- [darkkhaki](#valdef-color-darkkhaki), in § 6.1
- [darkmagenta](#valdef-color-darkmagenta), in § 6.1
- [darkolivegreen](#valdef-color-darkolivegreen), in § 6.1
- [darkorange](#valdef-color-darkorange), in § 6.1
- [darkorchid](#valdef-color-darkorchid), in § 6.1
- [darkred](#valdef-color-darkred), in § 6.1
- [darksalmon](#valdef-color-darksalmon), in § 6.1
- [darkseagreen](#valdef-color-darkseagreen), in § 6.1
- [darkslateblue](#valdef-color-darkslateblue), in § 6.1
- [darkslategray](#valdef-color-darkslategray), in § 6.1
- [darkslategrey](#valdef-color-darkslategrey), in § 6.1
- [darkturquoise](#valdef-color-darkturquoise), in § 6.1
- [darkviolet](#valdef-color-darkviolet), in § 6.1
- [decreasing](#decreasing), in § 13.4.4
- [deeppink](#valdef-color-deeppink), in § 6.1
- [deepskyblue](#valdef-color-deepskyblue), in § 6.1
- [\<deprecated-color\>](#typedef-deprecated-color), in § Unnumbered section
- [dimgray](#valdef-color-dimgray), in § 6.1
- [dimgrey](#valdef-color-dimgrey), in § 6.1
- [display-p3](#valdef-color-display-p3), in § 10.4
- [display-p3-linear](#display-p3-linear-space), in § 10.5
- [dodgerblue](#valdef-color-dodgerblue), in § 6.1
- [equivalent colors](#equivalent-colors), in § 12
- [Field](#valdef-color-field), in § 6.2
- [FieldText](#valdef-color-fieldtext), in § 6.2
- [firebrick](#valdef-color-firebrick), in § 6.1
- [floralwhite](#valdef-color-floralwhite), in § 6.1
- [forestgreen](#valdef-color-forestgreen), in § 6.1
- [fuchsia](#valdef-color-fuchsia), in § 6.1
- [gainsboro](#valdef-color-gainsboro), in § 6.1
- [gamut](#gamut), in § 2
- [ghostwhite](#valdef-color-ghostwhite), in § 6.1
- [gold](#valdef-color-gold), in § 6.1
- [goldenrod](#valdef-color-goldenrod), in § 6.1
- [gray](#valdef-color-gray), in § 6.1
- [GrayText](#valdef-color-graytext), in § 6.2
- [green](#valdef-color-green), in § 6.1
- [greenyellow](#valdef-color-greenyellow), in § 6.1
- [grey](#valdef-color-grey), in § 6.1
- [\<hex-color\>](#typedef-hex-color), in § 5.2
- [hex color](#hex-color), in § 5.2
- [hex color notation](#hex-color), in § 5.2
- [Highlight](#valdef-color-highlight), in § 6.2
- [HighlightText](#valdef-color-highlighttext), in § 6.2
- [honeydew](#valdef-color-honeydew), in § 6.1
- [host syntax](#host-syntax), in § 13.1
- [hotpink](#valdef-color-hotpink), in § 6.1
- [HSL](#valdef-hsl-hsl), in § 7
- [hsl()](#funcdef-hsl), in § 7
- [hsla()](#funcdef-hsla), in § 7
- [HTML-compatible serialization is requested](#color-serialization-html-compatible-serialization-is-requested), in § 16.2.1
- [\<hue\>](#typedef-hue), in § 4.3
- [\<hue-interpolation-method\>](#typedef-hue-interpolation-method), in § 13.1
- [HWB](#valdef-hwb-hwb), in § 8
- [hwb()](#funcdef-hwb), in § 8
- [InactiveBorder](#valdef-color-inactiveborder), in § Unnumbered section
- [InactiveCaption](#valdef-color-inactivecaption), in § Unnumbered section
- [InactiveCaptionText](#valdef-color-inactivecaptiontext), in § Unnumbered section
- [increasing](#increasing), in § 13.4.3
- [indianred](#valdef-color-indianred), in § 6.1
- [indigo](#valdef-color-indigo), in § 6.1
- [InfoBackground](#valdef-color-infobackground), in § Unnumbered section
- [InfoText](#valdef-color-infotext), in § Unnumbered section
- [in-gamut](#in-gamut), in § 2
- [interpolation color space](#interpolation-color-space), in § 13
- [invalid color](#invalid-color), in § 2
- [ivory](#valdef-color-ivory), in § 6.1
- [khaki](#valdef-color-khaki), in § 6.1
- [Lab](#valdef-lab-lab), in § 9.3
- [lab()](#funcdef-lab), in § 9.3
- [lavender](#valdef-color-lavender), in § 6.1
- [lavenderblush](#valdef-color-lavenderblush), in § 6.1
- [lawngreen](#valdef-color-lawngreen), in § 6.1
- [LCH](#valdef-lch-lch), in § 9.3
- [lch()](#funcdef-lch), in § 9.3
- [legacy color syntax](#legacy-color-syntax), in § 4.1.2
- [\<legacy-hsla-syntax\>](#typedef-legacy-hsla-syntax), in § 7
- [\<legacy-hsl-syntax\>](#typedef-legacy-hsl-syntax), in § 7
- [\<legacy-rgba-syntax\>](#typedef-legacy-rgba-syntax), in § 5.1
- [\<legacy-rgb-syntax\>](#typedef-legacy-rgb-syntax), in § 5.1
- [lemonchiffon](#valdef-color-lemonchiffon), in § 6.1
- [lightblue](#valdef-color-lightblue), in § 6.1
- [lightcoral](#valdef-color-lightcoral), in § 6.1
- [lightcyan](#valdef-color-lightcyan), in § 6.1
- [lightgoldenrodyellow](#valdef-color-lightgoldenrodyellow), in § 6.1
- [lightgray](#valdef-color-lightgray), in § 6.1
- [lightgreen](#valdef-color-lightgreen), in § 6.1
- [lightgrey](#valdef-color-lightgrey), in § 6.1
- [lightpink](#valdef-color-lightpink), in § 6.1
- [lightsalmon](#valdef-color-lightsalmon), in § 6.1
- [lightseagreen](#valdef-color-lightseagreen), in § 6.1
- [lightskyblue](#valdef-color-lightskyblue), in § 6.1
- [lightslategray](#valdef-color-lightslategray), in § 6.1
- [lightslategrey](#valdef-color-lightslategrey), in § 6.1
- [lightsteelblue](#valdef-color-lightsteelblue), in § 6.1
- [lightyellow](#valdef-color-lightyellow), in § 6.1
- [lime](#valdef-color-lime), in § 6.1
- [limegreen](#valdef-color-limegreen), in § 6.1
- [linen](#valdef-color-linen), in § 6.1
- [LinkText](#valdef-color-linktext), in § 6.2
- [longer](#longer), in § 13.4.2
- [luminance](#luminance), in § 2
- [magenta](#valdef-color-magenta), in § 6.1
- [Mark](#valdef-color-mark), in § 6.2
- [MarkText](#valdef-color-marktext), in § 6.2
- [maroon](#valdef-color-maroon), in § 6.1
- [mediumaquamarine](#valdef-color-mediumaquamarine), in § 6.1
- [mediumblue](#valdef-color-mediumblue), in § 6.1
- [mediumorchid](#valdef-color-mediumorchid), in § 6.1
- [mediumpurple](#valdef-color-mediumpurple), in § 6.1
- [mediumseagreen](#valdef-color-mediumseagreen), in § 6.1
- [mediumslateblue](#valdef-color-mediumslateblue), in § 6.1
- [mediumspringgreen](#valdef-color-mediumspringgreen), in § 6.1
- [mediumturquoise](#valdef-color-mediumturquoise), in § 6.1
- [mediumvioletred](#valdef-color-mediumvioletred), in § 6.1
- [Menu](#valdef-color-menu), in § Unnumbered section
- [MenuText](#valdef-color-menutext), in § Unnumbered section
- [midnightblue](#valdef-color-midnightblue), in § 6.1
- [MINDE](#minde), in § 14.1.2
- [mintcream](#valdef-color-mintcream), in § 6.1
- [missing](#missing-color-component), in § 4.4
- [missing color component](#missing-color-component), in § 4.4
- [missing component](#missing-color-component), in § 4.4
- [mistyrose](#valdef-color-mistyrose), in § 6.1
- [moccasin](#valdef-color-moccasin), in § 6.1
- [modern color syntax](#modern-color-syntax), in § 4.1.1
- [\<modern-hsla-syntax\>](#typedef-modern-hsla-syntax), in § 7
- [\<modern-hsl-syntax\>](#typedef-modern-hsl-syntax), in § 7
- [\<modern-rgba-syntax\>](#typedef-modern-rgba-syntax), in § 5.1
- [\<modern-rgb-syntax\>](#typedef-modern-rgb-syntax), in § 5.1
- [\<named-color\>](#typedef-named-color), in § 6.1
- [named color](#named-color), in § 6.1
- [navajowhite](#valdef-color-navajowhite), in § 6.1
- [navy](#valdef-color-navy), in § 6.1
- [none](#valdef-color-none), in § 4.4
- [Oklab](#valdef-oklab-oklab), in § 9.4
- [oklab()](#funcdef-oklab), in § 9.4
- [OkLCh](#valdef-oklch-oklch), in § 9.4
- [oklch()](#funcdef-oklch), in § 9.4
- [oldlace](#valdef-color-oldlace), in § 6.1
- [olive](#valdef-color-olive), in § 6.1
- [olivedrab](#valdef-color-olivedrab), in § 6.1
- [opacity](#propdef-opacity), in § 3.3
- [\<opacity-value\>](#typedef-opacity-opacity-value), in § 3.3
- [opaque black](#opaque-black), in § 4
- [orange](#valdef-color-orange), in § 6.1
- [orangered](#valdef-color-orangered), in § 6.1
- [orchid](#valdef-color-orchid), in § 6.1
- [out of gamut](#out-of-gamut), in § 2
- [palegoldenrod](#valdef-color-palegoldenrod), in § 6.1
- [palegreen](#valdef-color-palegreen), in § 6.1
- [paleturquoise](#valdef-color-paleturquoise), in § 6.1
- [palevioletred](#valdef-color-palevioletred), in § 6.1
- [papayawhip](#valdef-color-papayawhip), in § 6.1
- [parse a CSS \<color\> value](#parse-a-css-color-value), in § 4.5
- [peachpuff](#valdef-color-peachpuff), in § 6.1
- [peru](#valdef-color-peru), in § 6.1
- [pink](#valdef-color-pink), in § 6.1
- [plum](#valdef-color-plum), in § 6.1
- [\<polar-color-space\>](#typedef-polar-color-space), in § 13.1
- [powderblue](#valdef-color-powderblue), in § 6.1
- [powerless](#powerless-color-component), in § 4.4.1
- [powerless color component](#powerless-color-component), in § 4.4.1
- [powerless component](#powerless-color-component), in § 4.4.1
- [\<predefined-rgb\>](#typedef-predefined-rgb), in § 10.1
- [\<predefined-rgb-params\>](#typedef-predefined-rgb-params), in § 10.1
- [premultiplied](#premultiplied), in § 13.3
- [prepare a color col1 for conversion](#prepare-a-color-col1-for-conversion), in § 11
- [prophoto-rgb](#valdef-color-prophoto-rgb), in § 10.7
- [purple](#valdef-color-purple), in § 6.1
- [\<quirky-color\>](#typedef-quirky-color), in § Unnumbered section
- [Ray Trace Gamut Map](#ray-trace-gamut-map), in § 14.2.6
- [rebeccapurple](#valdef-color-rebeccapurple), in § 6.1
- [rec2020](#valdef-color-rec2020), in § 10.8
- [\<rectangular-color-space\>](#typedef-rectangular-color-space), in § 13.1
- [rectangular orthogonal color](#rectangular-orthogonal-color), in § 4
- [red](#valdef-color-red), in § 6.1
- [resolve to sRGB](#resolve-to-srgb), in § 4.1
- [rgb()](#funcdef-rgb), in § 5.1
- [rgba()](#funcdef-rgba), in § 5.1
- [rosybrown](#valdef-color-rosybrown), in § 6.1
- [royalblue](#valdef-color-royalblue), in § 6.1
- [saddlebrown](#valdef-color-saddlebrown), in § 6.1
- [salmon](#valdef-color-salmon), in § 6.1
- [sandybrown](#valdef-color-sandybrown), in § 6.1
- [Scrollbar](#valdef-color-scrollbar), in § Unnumbered section
- [seagreen](#valdef-color-seagreen), in § 6.1
- [seashell](#valdef-color-seashell), in § 6.1
- [SelectedItem](#valdef-color-selecteditem), in § 6.2
- [SelectedItemText](#valdef-color-selecteditemtext), in § 6.2
- [shorter](#shorter), in § 13.4.1
- [sienna](#valdef-color-sienna), in § 6.1
- [silver](#valdef-color-silver), in § 6.1
- [skyblue](#valdef-color-skyblue), in § 6.1
- [slateblue](#valdef-color-slateblue), in § 6.1
- [slategray](#valdef-color-slategray), in § 6.1
- [slategrey](#valdef-color-slategrey), in § 6.1
- [snow](#valdef-color-snow), in § 6.1
- [springgreen](#valdef-color-springgreen), in § 6.1
- [sRGB](#sRGB-space), in § 10.2
- [srgb](#valdef-color-srgb), in § 10.2
- [sRGB-linear](#sRGB-linear-space), in § 10.3
- [srgb-linear](#valdef-color-srgb-linear), in § 10.3
- [steelblue](#valdef-color-steelblue), in § 6.1
- [support legacy color syntax](#support-legacy-color-syntax), in § 4.1
- [\<system-color\>](#typedef-system-color), in § 6.2
- [system color pairings](#system-color-pairings), in § 6.2
- [system colors](#css-system-colors), in § 6.1
- [tagged image](#tagged-image), in § 3.4
- [tan](#valdef-color-tan), in § 6.1
- [teal](#valdef-color-teal), in § 6.1
- [thistle](#valdef-color-thistle), in § 6.1
- [ThreeDDarkShadow](#valdef-color-threeddarkshadow), in § Unnumbered section
- [ThreeDFace](#valdef-color-threedface), in § Unnumbered section
- [ThreeDHighlight](#valdef-color-threedhighlight), in § Unnumbered section
- [ThreeDLightShadow](#valdef-color-threedlightshadow), in § Unnumbered section
- [ThreeDShadow](#valdef-color-threedshadow), in § Unnumbered section
- [tomato](#valdef-color-tomato), in § 6.1
- [transparent](#valdef-color-transparent), in § 6.3
- [transparent black](#transparent-black), in § 4
- [turquoise](#valdef-color-turquoise), in § 6.1
- [untagged image](#untagged-image), in § 3.5
- [untagged video](#untagged-video), in § 3.5
- [used color](#used-color), in § 15
- [valid color](#valid-color), in § 2
- [violet](#valdef-color-violet), in § 6.1
- [VisitedText](#valdef-color-visitedtext), in § 6.2
- [wheat](#valdef-color-wheat), in § 6.1
- [white](#valdef-color-white), in § 6.1
- [white point](#white-point), in § 2
- [whitesmoke](#valdef-color-whitesmoke), in § 6.1
- [Window](#valdef-color-window), in § Unnumbered section
- [WindowFrame](#valdef-color-windowframe), in § Unnumbered section
- [WindowText](#valdef-color-windowtext), in § Unnumbered section
- [xyz](#valdef-color-xyz), in § 10.9
- [xyz-d50](#valdef-color-xyz-d50), in § 10.9
- [xyz-d65](#valdef-color-xyz-d65), in § 10.9
- [\<xyz-params\>](#typedef-xyz-params), in § 10.1
- [\<xyz-space\>](#typedef-xyz-space), in § 10.1
- [yellow](#valdef-color-yellow), in § 6.1
- [yellowgreen](#valdef-color-yellowgreen), in § 6.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-ANIMATIONS-1\] defines the following terms:
  - <a id="1e12dba3"></a>animation
- \[CSS-BACKGROUNDS-3\] defines the following terms:
  - <a id="db6870d5"></a>background
  - <a id="2754893b"></a>background-color
  - <a id="3bf45619"></a>border-bottom-color
  - <a id="65b3a7bc"></a>border-color
  - <a id="e2ddaaa8"></a>border-left-color
  - <a id="4e28efd4"></a>border-right-color
  - <a id="d1764055"></a>border-top-color
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="92922499"></a>declared value
  - <a id="4905669f"></a>inherited value
  - <a id="6b448e93"></a>initial value
  - <a id="d5e08d9c"></a>specified value
  - <a id="1a2b1083"></a>used value
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="4757cc3e"></a>a
  - <a id="f83d1822"></a>b
  - <a id="f487cd7d"></a>c
  - <a id="0644a74e"></a>color-mix()
  - <a id="ee6b9315"></a>h
  - <a id="b786b3c1"></a>relative color
- \[CSS-COLOR-ADJUST-1\] defines the following terms:
  - <a id="51e4fdb6"></a>forced colors mode
  - <a id="a55c0dec"></a>used color scheme
- \[CSS-CONDITIONAL-3\] defines the following terms:
  - <a id="a5d6c9d2"></a>@supports
  - <a id="087858ba"></a>supports(conditionText)
- \[CSS-IMAGES-4\] defines the following terms:
  - <a id="cda7345f"></a>\<gradient\>
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="87de393a"></a>\<dimension-token\>
  - <a id="7b8be35b"></a>\<hash-token\>
  - <a id="446c663e"></a>\<ident-token\>
  - <a id="eebbfe3d"></a>\<number-token\>
  - <a id="8a73a2e3"></a>\<percentage-token\>
  - <a id="67800454"></a>parse
- \[CSS-TEXT-DECOR-4\] defines the following terms:
  - <a id="441d3fda"></a>text-emphasis-color
- \[CSS-TRANSITIONS-1\] defines the following terms:
  - <a id="d3706df0"></a>transition
- \[CSS-UI-4\] defines the following terms:
  - <a id="4a550e0e"></a>accent color
  - <a id="8a568d22"></a>accent-color
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="c297b070"></a>\#
  - <a id="8cd4f032"></a>,
  - <a id="9f0f9416"></a>-infinity
  - <a id="d7e1d67b"></a>\<angle\>
  - <a id="dcecfc13"></a>\<ident\>
  - <a id="d73c993d"></a>\<integer\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="128295ac"></a>\<percentage\>
  - <a id="d4441b24"></a>?
  - <a id="14d3255d"></a>calc()
  - <a id="4dbf81d6"></a>canonical unit
  - <a id="8a110a7b"></a>CSS-wide keywords
  - <a id="de901111"></a>functional notation
  - <a id="7ee34834"></a>implementation-defined limit for values approaching infinity
  - <a id="f7826991"></a>infinity
  - <a id="0aac835b"></a>keyword
  - <a id="b9babad7"></a>NaN
  - <a id="8cbc2b3b"></a>{A}
  - <a id="4eb9d37e"></a>\|
- \[CSS2\] defines the following terms:
  - <a id="d332e4ec"></a>auto
  - <a id="b8c9eb8a"></a>stacking context
  - <a id="1848f1d3"></a>z-index
- \[CSSOM-1\] defines the following terms:
  - <a id="fc19454a"></a>resolved value
- \[DOM\] defines the following terms:
  - <a id="27d9b7ea"></a>element
  - <a id="fd11cdcd"></a>quirks mode
- \[FILTER-EFFECTS-1\] defines the following terms:
  - <a id="e9f6aadb"></a>filter
- \[HTML\] defines the following terms:
  - <a id="80a98d6e"></a>mark
- \[INFRA\] defines the following terms:
  - <a id="0698d556"></a>string
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="0a000463"></a>media feature

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-bradford-cat"></a>\[Bradford-CAT\]  
Ming R. Luo; R. W. G. Hunt. A Chromatic Adaptation Transform and a Colour Inconstancy Index. Color Research &#x26; Application 23(3) 154-158. June 1998.

<a id="biblio-cielab"></a>\[CIELAB\]  
[ISO/CIE 11664-4:2019(E): Colorimetry — Part 4: CIE 1976 L\*a\*b\* colour space](http://cie.co.at/publications/colorimetry-part-4-cie-1976-lab-colour-space-1). 2019. Published. URL: [http&#x3A;&#x2F;&#x2F;cie&#x2E;co&#x2E;at&#x2F;publications&#x2F;colorimetry-part-4-cie-1976-lab-colour-space-1](http://cie.co.at/publications/colorimetry-part-4-cie-1976-lab-colour-space-1)

<a id="biblio-colorimetry"></a>\[COLORIMETRY\]  
[Colorimetry, Fourth Edition. CIE 015:2018](http://www.cie.co.at/publications/colorimetry-4th-edition). 2018. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;cie&#x2E;co&#x2E;at&#x2F;publications&#x2F;colorimetry-4th-edition](http://www.cie.co.at/publications/colorimetry-4th-edition)

<a id="biblio-compositing"></a>\[Compositing\]  
Chris Harrelson. [Compositing and Blending Level 1](https://www.w3.org/TR/compositing-1/). 21 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;compositing-1&#x2F;](https://www.w3.org/TR/compositing-1/)

<a id="biblio-css-backgrounds-3"></a>\[CSS-BACKGROUNDS-3\]  
Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 11 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-adjust-1"></a>\[CSS-COLOR-ADJUST-1\]  
Elika Etemad; et al. [CSS Color Adjustment Module Level 1](https://www.w3.org/TR/css-color-adjust-1/). 16 December 2025. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-adjust-1&#x2F;](https://www.w3.org/TR/css-color-adjust-1/)

<a id="biblio-css-conditional-3"></a>\[CSS-CONDITIONAL-3\]  
Chris Lilley; David Baron; Elika Etemad. [CSS Conditional Rules Module Level 3](https://www.w3.org/TR/css-conditional-3/). 15 August 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-3&#x2F;](https://www.w3.org/TR/css-conditional-3/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-text-decor-4"></a>\[CSS-TEXT-DECOR-4\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 4](https://www.w3.org/TR/css-text-decor-4/). 4 May 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-4&#x2F;](https://www.w3.org/TR/css-text-decor-4/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Tab Atkins Jr.; Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 20 January 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 22 March 2024. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS2/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS2&#x2F;](https://www.w3.org/TR/CSS2/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-display-p3"></a>\[Display-P3\]  
Apple, Inc. [Display P3](https://www.color.org/chardata/rgb/DisplayP3.xalter). 2022-02. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;color&#x2E;org&#x2F;chardata&#x2F;rgb&#x2F;DisplayP3&#x2E;xalter](https://www.color.org/chardata/rgb/DisplayP3.xalter)

<a id="biblio-dom"></a>\[DOM\]  
Anne van Kesteren. [DOM Standard](https://dom.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;dom&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://dom.spec.whatwg.org/)

<a id="biblio-hsl"></a>\[HSL\]  
George H. Joblove, Donald Greenberg. [Color spaces for computer graphics](https://doi.org/10.1145/965139.807362). August 1978. URL: [https&#x3A;&#x2F;&#x2F;doi&#x2E;org&#x2F;10&#x2E;1145&#x2F;965139&#x2E;807362](https://doi.org/10.1145/965139.807362)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-hwb"></a>\[HWB\]  
Alvy Ray Smith. [HWB — A More Intuitive Hue-Based Color Model](http://alvyray.com/Papers/CG/HWB_JGTv208.pdf). 1996. URL: [http&#x3A;&#x2F;&#x2F;alvyray&#x2E;com&#x2F;Papers&#x2F;CG&#x2F;HWB_JGTv208&#x2E;pdf](http://alvyray.com/Papers/CG/HWB_JGTv208.pdf)

<a id="biblio-icc"></a>\[ICC\]  
[ICC.1:2022 (Profile version 4.4.0.0)](http://www.color.org/specification/ICC.1-2022-05.pdf). May 2022. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;color&#x2E;org&#x2F;specification&#x2F;ICC&#x2E;1-2022-05&#x2E;pdf](http://www.color.org/specification/ICC.1-2022-05.pdf)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-itu-r-bt601"></a>\[ITU-R-BT.601\]  
[Recommendation ITU-R BT.601](https://www.itu.int/rec/R-REC-BT.601/en). URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;itu&#x2E;int&#x2F;rec&#x2F;R-REC-BT&#x2E;601&#x2F;en](https://www.itu.int/rec/R-REC-BT.601/en)

<a id="biblio-itu-r-bt709"></a>\[ITU-R-BT.709\]  
[Recommendation ITU-R BT.709](https://www.itu.int/rec/R-REC-BT.709/en). URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;itu&#x2E;int&#x2F;rec&#x2F;R-REC-BT&#x2E;709&#x2F;en](https://www.itu.int/rec/R-REC-BT.709/en)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Tab Atkins Jr.; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 19 February 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-oklab"></a>\[Oklab\]  
Björn Ottosson. [A perceptual color space for image processing](https://bottosson.github.io/posts/oklab/). December 2020. URL: [https&#x3A;&#x2F;&#x2F;bottosson&#x2E;github&#x2E;io&#x2F;posts&#x2F;oklab&#x2F;](https://bottosson.github.io/posts/oklab/)

<a id="biblio-rec2020"></a>\[Rec.2020\]  
[Recommendation ITU-R BT.2020-2: Parameter values for ultra-high definition television systems for production and international programme exchange](http://www.itu.int/rec/R-REC-BT.2020/en). October 2015. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;itu&#x2E;int&#x2F;rec&#x2F;R-REC-BT&#x2E;2020&#x2F;en](http://www.itu.int/rec/R-REC-BT.2020/en)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-romm"></a>\[ROMM\]  
[ISO 22028-2:2013 Photography and graphic technology — Extended colour encodings for digital image storage, manipulation and interchange — Part 2: Reference output medium metric RGB colour image encoding (ROMM RGB)](https://www.iso.org/standard/56591.html). 2013-04. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;iso&#x2E;org&#x2F;standard&#x2F;56591&#x2E;html](https://www.iso.org/standard/56591.html)

<a id="biblio-smpte296"></a>\[SMPTE296\]  
[ST 296:2012, 1280 × 720 Progressive Image 4:2:2 and 4:4:4 Sample Structure — Analog and Digital Representation and Analog Interface](https://doi.org/10.5594/SMPTE.ST296.2012). 17 May 2012. Standard. URL: [https&#x3A;&#x2F;&#x2F;doi&#x2E;org&#x2F;10&#x2E;5594&#x2F;SMPTE&#x2E;ST296&#x2E;2012](https://doi.org/10.5594/SMPTE.ST296.2012)

<a id="biblio-srgb"></a>\[SRGB\]  
[Multimedia systems and equipment - Colour measurement and management - Part 2-1: Colour management - Default RGB colour space - sRGB](https://webstore.iec.ch/publication/6169). URL: [https&#x3A;&#x2F;&#x2F;webstore&#x2E;iec&#x2E;ch&#x2F;publication&#x2F;6169](https://webstore.iec.ch/publication/6169)

<a id="biblio-svg11"></a>\[SVG11\]  
Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG11&#x2F;](https://www.w3.org/TR/SVG11/)

### <a id="informative"></a>Non-Normative References

<a id="biblio-coloraide-minde"></a>\[Coloraide-MINDE\]  
Isaac Muse. [OkLCh Chroma](https://facelessuser.github.io/coloraide/gamut/#oklch-chroma). 2024. URL: [https&#x3A;&#x2F;&#x2F;facelessuser&#x2E;github&#x2E;io&#x2F;coloraide&#x2F;gamut&#x2F;#oklch-chroma](https://facelessuser.github.io/coloraide/gamut/#oklch-chroma)

<a id="biblio-coloraide-ray-trace"></a>\[Coloraide-Ray-Trace\]  
Isaac Muse. [Ray Tracing Chroma Reduction](https://facelessuser.github.io/coloraide/gamut/#ray-tracing-chroma-reduction). 2024. URL: [https&#x3A;&#x2F;&#x2F;facelessuser&#x2E;github&#x2E;io&#x2F;coloraide&#x2F;gamut&#x2F;#ray-tracing-chroma-reduction](https://facelessuser.github.io/coloraide/gamut/#ray-tracing-chroma-reduction)

<a id="biblio-colorjs-edgeseeker"></a>\[colorjs-EdgeSeeker\]  
Alexey Ardov. [EdgeSeeker Gamut Mapping algorithm](https://github.com/color-js/apps/tree/main/gamut-mapping/edge-seeker). 2023. URL: [https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;color-js&#x2F;apps&#x2F;tree&#x2F;main&#x2F;gamut-mapping&#x2F;edge-seeker](https://github.com/color-js/apps/tree/main/gamut-mapping/edge-seeker)

<a id="biblio-colorjs-minde"></a>\[colorjs-MINDE\]  
Chris Lilley. [Gamut mapping](https://colorjs.io/docs/gamut-mapping). 2022. URL: [https&#x3A;&#x2F;&#x2F;colorjs&#x2E;io&#x2F;docs&#x2F;gamut-mapping](https://colorjs.io/docs/gamut-mapping)

<a id="biblio-colorjs-raytrace"></a>\[colorjs-RayTrace\]  
Isaac Muse. [Ray Trace Gamut Mapping](https://github.com/color-js/apps/blob/2c7346dd00855f7b82eb4c7527355a09b84beeb3/gamut-mapping/methods.js#L273). 2023. URL: [https&#x3A;&#x2F;&#x2F;github&#x2E;com&#x2F;color-js&#x2F;apps&#x2F;blob&#x2F;2c7346dd00855f7b82eb4c7527355a09b84beeb3&#x2F;gamut-mapping&#x2F;methods&#x2E;js#L273](https://github.com/color-js/apps/blob/2c7346dd00855f7b82eb4c7527355a09b84beeb3/gamut-mapping/methods.js#L273)

<a id="biblio-css-animations-1"></a>\[CSS-ANIMATIONS-1\]  
David Baron; et al. [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/). 2 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-animations-1&#x2F;](https://www.w3.org/TR/css-animations-1/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; Una Kravets; Lea Verou. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 13 April 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-5&#x2F;](https://www.w3.org/TR/css-color-5/)

<a id="biblio-css-conditional-5"></a>\[CSS-CONDITIONAL-5\]  
Chris Lilley; et al. [CSS Conditional Rules Module Level 5](https://www.w3.org/TR/css-conditional-5/). 30 October 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-conditional-5&#x2F;](https://www.w3.org/TR/css-conditional-5/)

<a id="biblio-css-images-4"></a>\[CSS-IMAGES-4\]  
Elika Etemad; Tab Atkins Jr.; Lea Verou. [CSS Images Module Level 4](https://www.w3.org/TR/css-images-4/). 30 September 2025. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-4&#x2F;](https://www.w3.org/TR/css-images-4/)

<a id="biblio-css-transitions-1"></a>\[CSS-TRANSITIONS-1\]  
Chris Marrin; et al. [CSS Transitions Module Level 1](https://www.w3.org/TR/css-transitions-1/). 8 January 2026. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transitions-1&#x2F;](https://www.w3.org/TR/css-transitions-1/)

<a id="biblio-css3-text-decor"></a>\[CSS3-TEXT-DECOR\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 3](https://www.w3.org/TR/css-text-decor-3/). 5 May 2022. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-3&#x2F;](https://www.w3.org/TR/css-text-decor-3/)

<a id="biblio-dci-p3"></a>\[DCI-P3\]  
[SMPTE Recommended Practice - D-Cinema Quality — Reference Projector and Environment](http://ieeexplore.ieee.org/document/7290729/). 2011. URL: [http&#x3A;&#x2F;&#x2F;ieeexplore&#x2E;ieee&#x2E;org&#x2F;document&#x2F;7290729&#x2F;](http://ieeexplore.ieee.org/document/7290729/)

<a id="biblio-filter-effects-1"></a>\[FILTER-EFFECTS-1\]  
Dirk Schulze; Dean Jackson. [Filter Effects Module Level 1](https://www.w3.org/TR/filter-effects-1/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;filter-effects-1&#x2F;](https://www.w3.org/TR/filter-effects-1/)

<a id="biblio-jpeg"></a>\[JPEG\]  
Eric Hamilton. [JPEG File Interchange Format](https://www.w3.org/Graphics/JPEG/jfif3.pdf). September 1992. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;Graphics&#x2F;JPEG&#x2F;jfif3&#x2E;pdf](https://www.w3.org/Graphics/JPEG/jfif3.pdf)

<a id="biblio-png"></a>\[PNG\]  
Chris Lilley; et al. [Portable Network Graphics (PNG) Specification (Third Edition)](https://www.w3.org/TR/png-3/). 24 June 2025. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;png-3&#x2F;](https://www.w3.org/TR/png-3/)

<a id="biblio-rec_bt1886"></a>\[REC_BT.1886\]  
[ITU-R BT.1886 Reference electro-optical transfer function for flat panel displays used in HDTV studio production](https://www.itu.int/rec/R-REC-BT.1886/en). 2011-03. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;itu&#x2E;int&#x2F;rec&#x2F;R-REC-BT&#x2E;1886&#x2F;en](https://www.itu.int/rec/R-REC-BT.1886/en)

<a id="biblio-romm-rgb"></a>\[ROMM-RGB\]  
[ROMM RGB](https://www.color.org/chardata/rgb/rommrgb.xalter). URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;color&#x2E;org&#x2F;chardata&#x2F;rgb&#x2F;rommrgb&#x2E;xalter](https://www.color.org/chardata/rgb/rommrgb.xalter)

<a id="biblio-sharma"></a>\[Sharma\]  
G. Sharma; W. Wu; E. N. Dalal. [The CIEDE2000 Color-Difference Formula: Implementation Notes, Supplementary Test Data, and Mathematical Observations](https://www.hajim.rochester.edu/ece/sites/gsharma/ciede2000/). February 2005. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;hajim&#x2E;rochester&#x2E;edu&#x2F;ece&#x2F;sites&#x2F;gsharma&#x2F;ciede2000&#x2F;](https://www.hajim.rochester.edu/ece/sites/gsharma/ciede2000/)

<a id="biblio-tiff"></a>\[TIFF\]  
[TIFF Revision 6.0](https://www.loc.gov/preservation/digital/formats/fdd/fdd000022.shtml). 3 June 1992. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;loc&#x2E;gov&#x2F;preservation&#x2F;digital&#x2F;formats&#x2F;fdd&#x2F;fdd000022&#x2E;shtml](https://www.loc.gov/preservation/digital/formats/fdd/fdd000022.shtml)

<a id="biblio-understanding_cct"></a>\[Understanding_CCT\]  
[What is CCT? A Guide to Choosing Correlated Color Temperature for Your Lighting](https://litomatic.com/blog/what-is-cct-in-lighting/). 2024-08-14. URL: [https&#x3A;&#x2F;&#x2F;litomatic&#x2E;com&#x2F;blog&#x2F;what-is-cct-in-lighting&#x2F;](https://litomatic.com/blog/what-is-cct-in-lighting/)

<a id="biblio-wcag21"></a>\[WCAG21\]  
Michael Cooper; et al. [Web Content Accessibility Guidelines (WCAG) 2.1](https://www.w3.org/TR/WCAG21/). 6 May 2025. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;WCAG21&#x2F;](https://www.w3.org/TR/WCAG21/)

<a id="biblio-wolfe"></a>\[Wolfe\]  
Geoff Wolfe. [Design and Optimization of the ProPhoto RGB Color Encodings](http://www.realtimerendering.com/blog/2011-color-and-imaging-conference-part-vi-special-session/). 2011-12-21. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;realtimerendering&#x2E;com&#x2F;blog&#x2F;2011-color-and-imaging-conference-part-vi-special-session&#x2F;](http://www.realtimerendering.com/blog/2011-color-and-imaging-conference-part-vi-special-session/)

## <a id="property-index"></a>Property Index



| Name                | Value             | Initial    | Applies to            | Inh. | %ages                    | Anim­ation type         | Canonical order | Com­puted value                                 |
|---------------------|-------------------|------------|-----------------------|------|--------------------------|------------------------|-----------------|------------------------------------------------|
| <strong><span><a id="ref-for-propdef-color⑨"></a></span><a href="#propdef-color">color</a>&#xA;      </strong> | \<color\>         | CanvasText | all elements and text | yes  | N/A                      | by computed value type | per grammar     | computed color, see resolving color values     |
| <strong><span><a id="ref-for-propdef-opacity⑧"></a></span><a href="#propdef-opacity">opacity</a>&#xA;      </strong> | \<opacity-value\> | 1          | all elements          | no   | map to the range \[0,1\] | by computed value type | per grammar     | specified number, clamped to the range \[0,1\] |



## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> add pseudocode for EdgeSeeker GMA [↵](#issue-4b1038ec)
