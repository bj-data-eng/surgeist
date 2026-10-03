Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Color Module Level 5](https://www.w3.org/TR/2026/WD-css-color-5-20260908/).

Original copyright notice: Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Color Module Level 5

Source snapshot: https://www.w3.org/TR/2026/WD-css-color-5-20260908/

Snapshot SHA-256: f749cb75ec1c0faca7d2443546c88be4e22750526dd45f9fefcfcdd079ed3c09

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 9 source tables are presented as readable Markdown tables or explicit labeled layouts: 5 ordinary table conversions, 4 already-readable tables. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Color Module Level 5

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This module extends CSS Color [\[css-color-4\]](#biblio-css-color-4) to add color modification functions, custom color spaces (ICC profiles), contrast-color(), light-dark() and device-cmyk().

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C standards and drafts index.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-color” in the title, like this: “\[css-color\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-color%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/20200915/#sec-Disclosure).

The following features are at-risk, and may be dropped during the CR period:

- Custom Color Spaces, '@color-profile', 'device-cmyk()', Relative Alpha Colors

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1. Introduction

<em>This section is not normative.</em>

<a id="ref-for-funcdef-contrast-color"></a>

<a id="ref-for-funcdef-color-mix"></a>

<a id="ref-for-funcdef-light-dark"></a>

This module adds the new functions [contrast-color()](#funcdef-contrast-color), [color-mix()](#funcdef-color-mix) and [light-dark()](#funcdef-light-dark), and extends existing ones with [relative color syntax](#relative-colors).

<a id="ref-for-funcdef-color"></a>

It also extends the [color()](#funcdef-color) function so that not only predefined color spaces, but also custom color spaces defined by ICC profiles (including calibrated CMYK) can be used in CSS.

It also adds device-cmyk, a representation of uncalibrated cmyk color.

<a id="ref-for-typedef-color"></a>

## <a id="color-syntax"></a>2. The [\<color\>](#typedef-color) syntax

<a id="ref-for-typedef-color①"></a>

Colors in CSS are represented by the <a id="typedef-color"></a>[\<color\>](#typedef-color) type:

<a id="ref-for-typedef-color-base"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-typedef-system-color"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-funcdef-contrast-color①"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-funcdef-device-cmyk"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-typedef-light-dark-color"></a>

<a id="typedef-color-base"></a>

<a id="ref-for-typedef-hex-color"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-typedef-color-function"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-typedef-named-color"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-funcdef-color-mix①"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="typedef-color-function"></a>

<a id="ref-for-funcdef-rgb"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-funcdef-rgba"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-funcdef-hsl"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-funcdef-hsla"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-funcdef-hwb"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-funcdef-lab"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-funcdef-lch"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-funcdef-oklab"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-funcdef-oklch"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-funcdef-alpha"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-funcdef-color①"></a>

```text
<color> = <color-base> | currentColor | <system-color> | 
      <contrast-color()> | <device-cmyk()>  | <light-dark-color>

<color-base> = <hex-color> | <color-function> | <named-color> | <color-mix()> | transparent
<color-function> = <rgb()> | <rgba()> |
              <hsl()> | <hsla()> | <hwb()> |
              <lab()> | <lch()> | <oklab()> | <oklch()> |
              <alpha()> |
              <color()>
```
<a id="ref-for-typedef-color②"></a>

An <a id="absolute-color"></a>absolute color is a [\<color\>](#typedef-color) whose computed value has an absolute, colorimetric interpretation. This means that the value is not:

- <a id="ref-for-valdef-color-currentcolor"></a>

  <a id="ref-for-propdef-color"></a>

  [currentColor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor) (which depends on the value of the [color](https://www.w3.org/TR/css-color-4/#propdef-color) property)

- <a id="ref-for-typedef-system-color①"></a>

  a [\<system-color\>](https://www.w3.org/TR/css-color-4/#typedef-system-color) (which depends on the color mode)

- <a id="ref-for-typedef-light-dark-color①"></a>

  [\<light-dark-color\>](#typedef-light-dark-color) (which depends on the color mode)

- <a id="ref-for-funcdef-contrast-color②"></a>

  [\<contrast-color()\>](#funcdef-contrast-color) (which depends on the color mode)

- <a id="ref-for-funcdef-device-cmyk①"></a>

  [\<device-cmyk()\>](#funcdef-device-cmyk) (which has no colorimetric basis)

<a id="ref-for-funcdef-color-mix②"></a>

Nor are any of those values used inside [\<color-mix()\>](#funcdef-color-mix) or in relative color syntax.

The colors that <a id="resolve-to-srgb"></a>resolve to sRGB are:

- [hex](https://www.w3.org/TR/2026/WD-css-color-5-20260908/css-color-4#hex-notation) colors

- <a id="ref-for-funcdef-rgb①"></a>

  <a id="ref-for-funcdef-rgba①"></a>

  [rgb()](https://www.w3.org/TR/css-color-4/#funcdef-rgb) and [rgba()](https://www.w3.org/TR/css-color-4/#funcdef-rgba) values, including relative colors

- <a id="ref-for-funcdef-hsl①"></a>

  <a id="ref-for-funcdef-hsla①"></a>

  [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) and [hsla()](https://www.w3.org/TR/css-color-4/#funcdef-hsla) values, including relative colors

- <a id="ref-for-funcdef-hwb①"></a>

  [hwb()](#funcdef-hwb) values, including relative colors

- [named](https://www.w3.org/TR/2026/WD-css-color-5-20260908/css-color-4#named-colors) colors

The functions that <a id="support-legacy-color-syntax"></a>support legacy color syntax are:

- <a id="ref-for-funcdef-rgb②"></a>

  <a id="ref-for-funcdef-rgba②"></a>

  [rgb()](https://www.w3.org/TR/css-color-4/#funcdef-rgb) and [rgba()](https://www.w3.org/TR/css-color-4/#funcdef-rgba)

- <a id="ref-for-funcdef-hsl②"></a>

  <a id="ref-for-funcdef-hsla②"></a>

  [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) and [hsla()](https://www.w3.org/TR/css-color-4/#funcdef-hsla)

<a id="ref-for-funcdef-hsl③"></a>

<a id="ref-for-funcdef-hsla③"></a>

<a id="ref-for-funcdef-hwb②"></a>

<a id="ref-for-funcdef-lch①"></a>

<a id="ref-for-funcdef-oklch①"></a>

<a id="ref-for-color-functions"></a>

<a id="ref-for-cylindrical-polar-color"></a>

<a id="ref-for-typedef-hue"></a>

<a id="ref-for-rectangular-orthogonal-color"></a>

The [\<hsl()\>](https://www.w3.org/TR/css-color-4/#funcdef-hsl), [\<hsla()\>](https://www.w3.org/TR/css-color-4/#funcdef-hsla), [\<hwb()\>](#funcdef-hwb), [\<lch()\>](#funcdef-lch), and [\<oklch()\>](#funcdef-oklch) [color functions](https://www.w3.org/TR/css-color-4/#color-functions) are [cylindrical polar color](https://www.w3.org/TR/css-color-4/#cylindrical-polar-color) representations using a [\<hue\>](https://www.w3.org/TR/css-color-4/#typedef-hue) angle; the other <a id="ref-for-color-functions①"></a>color functions use [rectangular orthogonal color](https://www.w3.org/TR/css-color-4/#rectangular-orthogonal-color) representations.

<a id="ref-for-funcdef-color-mix③"></a>

## <a id="color-mix"></a>3. Mixing Colors: the [color-mix()](#funcdef-color-mix) Function

Web developers, design tools and design system developers often use color functions to assist in scaling the design of their component color relations. With the increasing usage of design systems that support multiple platforms and multiple user preferences, like the increased capability of Dark Mode in UI, this becomes even more useful to not need to manually set color, and to instead have a single source from which schemes are calculated.

<a id="fig-chloropleth"></a>

![LC color picker](https://www.w3.org/TR/2026/WD-css-color-5-20260908/images/LC-picker-scale.png)  
![chloropleth map of the US](https://www.w3.org/TR/2026/WD-css-color-5-20260908/images/LC-picker-map2.png)

Above, a color picker operating in CIE LCH space. Here, a pair of colors are being used to define a color scale on the Chroma-Lightness plane (constant Hue). Below, the color scale in use on a choropleth map.

Currently Sass, calc() on HSL values, or PostCSS is used to do this. However, preprocessors are unable to work on dynamically adjusted colors; all current solutions are restricted to the sRGB gamut and to the perceptual limitations of HSL (colors are bunched up in the color wheel, and two colors with visually different lightness, like yellow and blue, can have the same HSL lightness).

<a id="ref-for-typedef-color③"></a>

<a id="ref-for-typedef-color-space"></a>

To meet this need, the color-mix() function takes a list of one or more [\<color\>](#typedef-color) specifications and returns the result of mixing them, in a given [\<color-space\>](#typedef-color-space), in the specified amounts.

<a id="funcdef-color-mix"></a>

<a id="ref-for-color-interpolation-method"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-comb-comma"></a>

<a id="ref-for-typedef-color④"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-percentage-value"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-mult-comma"></a>

```text
color-mix() = color-mix( <color-interpolation-method>? , [ <color> && <percentage [0,100]>? ]#)
```
Tests

- [color-computed-color-mix-function.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-color-mix-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-color-mix-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-color-mix-function.html)
- [color-valid-color-mix-function.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-color-mix-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-color-mix-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-color-mix-function.html)

### <a id="color-mix-space"></a>3.1.  Colorspace for mixing 

If no color interpolation method is specified, assume Oklab. Otherwise, use the specified colorspace for mixing.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-omitted-colorspace"></a> For example, these two are exactly equivalent:
>
> ```css
> color-mix(in oklab, firebrick, goldenrod)
> color-mix(firebrick, goldenrod)
> ```
### <a id="color-mix-percent-norm"></a>3.2.  Percentage Normalization 

<a id="ref-for-normalize-mix-percentages"></a>

Percentages are required to be in the range 0% to 100%. Negative percentages are specifically disallowed. Percentages are normalized by [normalizing mix percentages](https://drafts.csswg.org/css-values-5/#normalize-mix-percentages).

Tests

- [color-mix-percents-01.html](https://wpt.fyi/results/css/css-color/color-mix-percents-01.html) [(live test)](http://wpt.live/css/css-color/color-mix-percents-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-percents-01.html)
- [color-mix-percents-02.html](https://wpt.fyi/results/css/css-color/color-mix-percents-02.html) [(live test)](http://wpt.live/css/css-color/color-mix-percents-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-percents-02.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-mix-syntactic"></a> These syntactic forms are thus all equivalent:
>
> ```css
> color-mix(in lch, purple 50%, plum 50%)
> color-mix(in lch, purple 50%, plum)
> color-mix(in lch, purple, plum 50%)
> color-mix(in lch, purple, plum)
> color-mix(in lch, plum, purple)
> color-mix(in lch, purple 80%, plum 80%)
> ```
>
> All produce a 50-50 mix of purple and plum, in lch: lch(51.51% 52.21 325.8) which is rgb(68.51% 36.01% 68.29%).
>
> However, this form is <em>not</em> the same, as the alpha is less than one:
>
> ```css
> color-mix(in lch, purple 30%, plum 30%)
> ```
>
> This produces lch(51.51% 52.21 325.8 / 0.6) which is rgb(68.51% 36.01% 68.29% / 0.6).

### <a id="color-mix-result"></a>3.3.  Calculating the Result of color-mix 

To <a id="calculate-a-color-mix"></a>calculate a color-mix():

1.  <a id="ref-for-normalize-mix-percentages①"></a>

    <a id="ref-for-mix-item"></a>

    [Normalize mix percentages](https://drafts.csswg.org/css-values-5/#normalize-mix-percentages) from the list of [mix items](https://drafts.csswg.org/css-values-5/#mix-item) passed to the function, with the "forced normalization" flag set to true, letting <var>items</var> and <var>leftover</var> be the result.

2.  Let <var>alpha mult</var> be <code>1 - <var>leftover</var></code>, interpreting <var>leftover</var> as a number between 0 and 1.

3.  <a id="ref-for-typedef-color-space①"></a>

    If <var>items</var> is length 1, set <var>color</var> to the color of that sole item, converted to the specified interpolation [\<color-space\>](#typedef-color-space).

    Otherwise:

    1.  <a id="ref-for-stack"></a>

        Let <var>item stack</var> be a [stack](https://infra.spec.whatwg.org/#stack) made by reversing <var>items</var>. (Thus, with the first item at the top of the stack.)

    2.  While <var>item stack</var> has length 2 or greater:

        1.  <a id="ref-for-stack-pop"></a>

            [Pop](https://infra.spec.whatwg.org/#stack-pop) from <var>item stack</var> twice, letting <var>a</var> and <var>b</var> be the two results in order. Let <var>combined percentage</var> be the sum of <var>a</var> and <var>b</var>’s percentages.

        2.  <a id="ref-for-cylindrical-polar-color①"></a>

            <a id="ref-for-typedef-hue-interpolation-method"></a>

            Interpolate <var>a</var> and <var>b</var>’s colors as described in [CSS Color 4 §  13. Color Interpolation](https://www.w3.org/TR/css-color-4/#interpolation), with a progress percentage equal to <code>(<var>b</var>’s percentage) / <var>combined percentage</var>)</code>, if <var>combined percentage</var> is greater than 0, and 0.5 otherwise. If the specified color space is a [cylindrical polar color](https://www.w3.org/TR/css-color-4/#cylindrical-polar-color) space, then the [\<hue-interpolation-method\>](#typedef-hue-interpolation-method) controls the interpolation of hue, as described in [CSS Color 4 § 13.5 Hue Interpolation](https://www.w3.org/TR/css-color-4/#hue-interpolation). If no <a id="ref-for-typedef-hue-interpolation-method①"></a>\<hue-interpolation-method\> is specified, assume shorter.

        3.  <a id="ref-for-mix-item①"></a>

            <a id="ref-for-stack-push"></a>

            Create a new [mix item](https://drafts.csswg.org/css-values-5/#mix-item) with the resulting color and a percentage of <var>combined percentage</var>, and [push](https://infra.spec.whatwg.org/#stack-push) it onto <var>item stack</var>.

    3.  Set <var>color</var> to the color of the sole remaining item in <var>item stack</var>.

4.  Multiply the alpha component of <var>color</var> by <var>alpha mult</var>.

5.  Return <var>color</var>.

<a id="ref-for-cylindrical-polar-color②"></a>

<a id="ref-for-rectangular-orthogonal-color①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In [cylindrical polar color](https://www.w3.org/TR/css-color-4/#cylindrical-polar-color) spaces, mixing is order-dependent, as which direction is “shorter” or “longer” around the hue circle can change depending on what other mixes have already been performed. This algorithm mixes each color in the specified order, mixing the result with the next in the list. For [rectangular orthogonal color](https://www.w3.org/TR/css-color-4/#rectangular-orthogonal-color) spaces, the order doesn’t matter, and the process can be simplified.

Tests

- [color-mix-basic-001.html](https://wpt.fyi/results/css/css-color/color-mix-basic-001.html) [(live test)](http://wpt.live/css/css-color/color-mix-basic-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-basic-001.html)
- [color-mix-missing-components.html](https://wpt.fyi/results/css/css-color/color-mix-missing-components.html) [(live test)](http://wpt.live/css/css-color/color-mix-missing-components.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-missing-components.html)
- [color-mix-non-srgb-001.html](https://wpt.fyi/results/css/css-color/color-mix-non-srgb-001.html) [(live test)](http://wpt.live/css/css-color/color-mix-non-srgb-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-non-srgb-001.html)
- [color-computed-color-mix-function.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-color-mix-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-color-mix-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-color-mix-function.html)
- [color-invalid-color-mix-function.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-color-mix-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-color-mix-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-color-mix-function.html)
- [color-valid-color-mix-function.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-color-mix-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-color-mix-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-color-mix-function.html)
- [color-mix-out-of-gamut.html](https://wpt.fyi/results/css/css-color/parsing/color-mix-out-of-gamut.html) [(live test)](http://wpt.live/css/css-color/parsing/color-mix-out-of-gamut.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-mix-out-of-gamut.html)
- [2d.fillStyle.colormix.html](https://wpt.fyi/results/html/canvas/element/fill-and-stroke-styles/2d.fillStyle.colormix.html) [(live test)](http://wpt.live/html/canvas/element/fill-and-stroke-styles/2d.fillStyle.colormix.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/html/canvas/element/fill-and-stroke-styles/2d.fillStyle.colormix.html)
- [2d.fillStyle.colormix.currentcolor.html](https://wpt.fyi/results/html/canvas/element/fill-and-stroke-styles/2d.fillStyle.colormix.currentcolor.html) [(live test)](http://wpt.live/html/canvas/element/fill-and-stroke-styles/2d.fillStyle.colormix.currentcolor.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/html/canvas/element/fill-and-stroke-styles/2d.fillStyle.colormix.currentcolor.html)
- [2d.strokeStyle.colormix.html](https://wpt.fyi/results/html/canvas/element/fill-and-stroke-styles/2d.strokeStyle.colormix.html) [(live test)](http://wpt.live/html/canvas/element/fill-and-stroke-styles/2d.strokeStyle.colormix.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/html/canvas/element/fill-and-stroke-styles/2d.strokeStyle.colormix.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-mix-lch-peru40"></a> This example produces a mixture of 40% peru and 60% palegoldenrod.
>
> ```css
> color-mix(in lch, peru 40%, palegoldenrod)
> ```
>
> <a id="ref-for-valdef-lch-lch"></a>
>
> The mixing is done in [lch](https://www.w3.org/TR/css-color-4/#valdef-lch-lch) color space. Here is a top-down view, looking along the neutral L axis:
>
> <a id="fig-LCH-peru-goldenrod"></a>
>
> ![A mixture of two colors, and the mixed output. We are looking down the CIE L axis onto the ab plane. There are two axes, labelled a and b which cross at the origin, which is in the centre of the plot.](https://www.w3.org/TR/2026/WD-css-color-5-20260908/images/CH-mixing.svg)
>
> A mixture of two colors, and the mixed output. We are looking down the CIE L axis onto the ab plane. There are two axes, labelled <em>a</em> and <em>b</em> which cross at the origin, which is in the centre of the plot.
>
> Mixtures of peru and palegoldenrod in CIE LCH. Peru has a hue angle, measured from the positive a axis, of 63.677 degrees while palegoldenrod has a hue angle of 98.834 degrees. Peru has a chroma, or distance from the central neutral axis, of 54.011 while palegoldenrod has a chroma of 31.406. All possible mixtures lie along the curve. A 40%/60% mixture is shown.
>
> The calculation is as follows:
>
> -  peru is lch(62.253% 54.011 63.677)
>
> -  palegoldenrod is lch(91.374% 31.406 98.834)
>
> - the mixed lightness is 62.253 \* 40/100 + 91.374 \* (100-40)/100 = 79.7256
>
> - the mixed chroma is 54.011 \* 40/100 + 31.406 \* (100-40)/100 = 40.448
>
> - the mixed hue is 63.677 \* 40/100 + 98.834 \* (100-40)/100 = 84.771
>
> - the mixed result is lch(79.7256% 40.448 84.771)

<a id="ref-for-valdef-lch-lch①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-mix-lch-teal65"></a> This example produces the mixture of teal and olive, in [lch](https://www.w3.org/TR/css-color-4/#valdef-lch-lch) color space, with each lch component being 65% of the value for teal and 35% of the value for olive.
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note: interpolating on hue and chroma keeps the intermediate colors as saturated as the endpoint colors.
>
> ```css
> color-mix(in lch, teal 65%, olive);
> ```
>
> <a id="fig-LCH-teal-olive"></a>
>
> ![A mixture of two colors, and the mixed output. We are looking down the CIE L axis onto the ab plane. There are two axes, labelled a and b which cross at the origin, which is in the centre of the plot.](https://www.w3.org/TR/2026/WD-css-color-5-20260908/images/CH-mixing3.svg)
>
> A mixture of two colors, and the mixed output. We are looking down the CIE L axis onto the ab plane. There are two axes, labelled <em>a</em> and <em>b</em> which cross at the origin, which is in the centre of the plot.
>
> Mixtures of teal and olive. Teal has a hue angle, measured from the positive a axis, of 196.4524 degrees while olive has a hue angle of 99.5746 degrees. Teal has a chroma, or distance from the central neutral axis, of 31.6903 while olive has a chroma of 56.8124. Mixtures lie along the dashed curve. A 65%/35% mixture is shown.
>
> The calculation is as follows:
>
> - sRGB teal (#008080) is lch(47.9855% 31.6903 196.4524)
>
> - sRGB olive (#808000) is lch(52.1496% 56.8124 99.5746)
>
> - mixed lightness is 47.9855 \* 0.65 + 52.1496 \* 0.35 = 49.4429
>
> - mixed chroma is 31.6903 \* 0.65 + 56.8124 \* 0.35 = 40.4830
>
> - mixed hue is 196.4524 \* 0.65 + 99.5746 \* 0.35 = 162.5452
>
> - mixed result is lch(49.4429% 40.4830 162.5452)
>
> - which is a slightly-blueish green: rgb(7.7377% 52.5730% 37.3213%)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-mix-zero-sum"></a> In this example, both percentages are zero, so their sum is also zero; thus, the alpha is zero and the color components are mixed 50% 50%:
>
> ```css
> color-mix(in oklch, teal 0%, olive 0%);
> ```
>
> The calculation is as follows:
>
> - sRGB teal (#008080) is oklch(54.31% 0.0927 194.8)
>
> - sRGB olive (#808000) is oklch(58.07% 0.1266 109.8)
>
> - mixed lightness is 54.31% \* 0.5 + 58.07% \* 0.5 = 35.0395%
>
> - mixed chroma is 0.0927 \* 0.5 + 0.1266 \* 0.5 = 0.10965
>
> - mixed hue is 194.8 \* 0.5 + 109.8 \* 0.5 = 152.3
>
> - mixed result is the fully transparent color okch(35.0395% 0.10965 152.3 / 0)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-mix-three-no-percents"></a> In this example three colors are mixed, and no percentages are given so each color contributes one-third of the final result.
>
> ```css
> color-mix(in oklab, teal, olive, blue);
> ```
>
> The calculation is as follows:
>
> -  teal (#008080) is oklab(54.31% -0.0896 -0.0236)
>
> -  olive (#808000) is oklab(58.07% -0.0428 0.1191)
>
> -  blue (#0000FF) is oklab(45.20% -0.0325 -0.3115)
>
> - mixed lightness is (54.31 + 58.07 + 45.20) / 3 = 52.53%
>
> - mixed a is (-0.0896 + -0.0428 + -0.0325) / 3 = -0.0550
>
> - mixed b is (-0.0236 + 0.1191 + -0.3115) / 3 = -0.0720
>
> - mixed result is oklab(52.53% -0.0550 -0.0720)

### <a id="color-mix-color-space-effect"></a>3.4.  Effect of Mixing Color Space on color-mix 

The choice of mixing color space can have a large effect on the end result.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-mix-colorspaces-black-white"></a> This example is a 50% mix of white and black, in three different color spaces.
>
> ```css
> color-mix(in lch, white, black);
> color-mix(in xyz, white, black);
> color-mix(in srgb, white, black);
> ```
>
> The calculation is as follows:
>
> - sRGB white (#FFF) is lch(100% 0 none)
>
> - sRGB black (#000) is lch(0% 0 none)
>
> - The mix in LCH is lch(50% 0 none)
>
> - The mix in XYZ is lch(76% 0 none)
>
> - The mix in sRGB is lch(53.4% 0 none)
>
> The mix in LCH gives an L value of 50%, a perfect mid gray, exactly as expected (mixing in Lab would do the same, as the Lightness axis is the same in LCH and Lab).
>
> The mix in XYZ gives a result that is too light; XYZ is linear-light but is not perceptually uniform. The mix in sRGB gives a result that is a bit too light; sRGB is neither perceptually uniform nor linear-light.

<a id="ref-for-valdef-color-xyz-d50"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-mix-xyz"></a> This example produces the mixture of the a red and a sky blue, in [xyz-d50](https://www.w3.org/TR/css-color-4/#valdef-color-xyz-d50) color space, with the mixture being 75.23% of that of the red (and thus, 24.77% of that of the blue).
>
> ```css
> color-mix(in xyz-d50, rgb(82.02% 30.21% 35.02%) 75.23%, rgb(5.64% 55.94% 85.31%));
> ```
>
> The calculation is as follows:
>
> -  rgb(82.02% 30.21% 35.02%) is lch(52% 58.1 22.7) which is X=0.3214, Y=0.2014, Z=0.0879.
>
> -  rgb(5.64% 55.94% 85.31%) is lch(56% 49.1 257.1) which is X=0.2070, Y=0.2391, Z=0.5249.
>
> - mixed result X=(0.3214 \* 0.7523) + (0.2070 \* (1 - 0.7523)) = 0.29306.
>
> - mixed result Y=(0.2014 \* 0.7523) + (0.2391 \* (1 - 0.7523)) = 0.21074.
>
> - mixed result Z=(0.0879 \* 0.7523) + (0.5249 \* (1 - 0.7523)) = 0.19614.
>
> - mix result is lch(53.0304% 38.9346 352.8138) which is rgb(72.300% 38.639% 53.557%)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-mix-blue-white"></a>
>
> This example is a 50% mix of white and blue, in three different color spaces.
>
> ```css
> color-mix(in lch, white, blue);
> color-mix(in oklch, white, blue);
> color-mix(in srgb, white, blue);
> ```
>
> The calculation is as follows:
>
> -  white is rgb(100% 100% 100%) which is lch(100% 0 none) which is oklch(100% 0 none)
>
> -  blue is rgb(0% 0% 100%) which is lch(29.5683% 131.201 301.364) which is oklch(45.201% 0.31321 264.052)
>
> -  mix in lch is lch(64.7841% 65.6008 301.364) which is quite purple
>
> -  mix in oklch is oklch(72.601% 0.15661 264.052)
>
> -  mix in srgb is rgb(50% 50% 100%) which is also a bit purple

<a id="ref-for-valdef-hsl-hsl"></a>

<a id="ref-for-valdef-color-srgb"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-hsl-gamut-map"></a> This example is a mix of two colors, in [hsl](https://www.w3.org/TR/css-color-4/#valdef-hsl-hsl) color space, where one of the colors to be mixed is outside the [sRGB](https://www.w3.org/TR/css-color-4/#valdef-color-srgb) gamut.
>
> ```css
> color-mix(in hsl, color(display-p3 0 1 0) 80%, yellow);
> ```
>
> The calculation is as follows:
>
> -  color(display-p3 0 1 0) is color(srgb -0.5116 1.01827 -0.3107) which is outside the sRGB gamut
>
> - <a id="ref-for-valdef-hsl-hsl①"></a>
>
>   Converted to [hsl](https://www.w3.org/TR/css-color-4/#valdef-hsl-hsl) hsl(127.879 301.946 25.334)
>
> -  yellow is hsl(60 100% 50%)
>
> - the hue is 127.879 × 0.8 + 60 × 0.2 = 114.3032
>
> - the saturation is 301.946 × 0.8 + 100 × 0.2 = 261.5568
>
> - the lightness is 25.334 × 0.8 + 50 × 0.2 = 30.2672
>
> - the mixed result is hsl(114.3032 261.5568 30.2672) which is color(srgb -0.3387 1.0943 -0.48899)

<a id="ref-for-funcdef-device-cmyk②"></a>

<a id="ref-for-funcdef-color-mix④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-device-cmyk-mix"></a> [device-cmyk()](#funcdef-device-cmyk) can be used in [color-mix()](#funcdef-color-mix) but the result will depend on how the implementation chooses to obtain a computed value.
>
> ```css
> color-mix(in lab, device-cmyk(0.091777 0.043303 0.312816 0.000000) 100%, yellow);
> ```
>
> Since the first color is at 100%, the second color is 0% and does not affect the mixed result in any way. The result is thus the computed value of the first color, in CIE Lab.
>
> To visualize the result, let us say that the device CMYK values are in fact to be printed using SWOP 2006 coated.
>
> -  device-cmyk(0.091777 0.043303 0.312816 0.000000) is lab(91.44% 4.142 20.52)
>
> <a id="ref-for-funcdef-lab①"></a>
>
> Suppose the implementation uses an ICC profile to obtain [lab()](#funcdef-lab) colors, and in this example a FOGRA39 Coated profile is used:
>
> -  device-cmyk(0.091777 0.043303 0.312816 0.000000) is lab(91.840596 -3.559090 20.449159)
>
> - The deltaE 2000 between this and the original printed color is <strong>8.17</strong> which is clearly visible.
>
> Now suppose another implementation uses the naive color conversion algorithm, giving an sRGB result.
>
> -  device-cmyk(0.091777 0.043303 0.312816 0.000000) is rgb(90.8223% 95.6697% 68.7184%) which is lab(94.02% -12.31 31.79)
>
> - The deltaE 2000 between this and the original printed color is <strong>14.3</strong> which is very visible.

### <a id="color-mix-with-alpha"></a>3.5.  Effect of Non-Unity Alpha on color-mix 

<a id="ref-for-funcdef-color-mix⑤"></a>

<a id="ref-for-premultiplied"></a>

So far, all the [color-mix()](#funcdef-color-mix) examples have used fully opaque colors. To simplify the examples, the [premultiplication](https://www.w3.org/TR/css-color-4/#premultiplied) and unpremultiplication steps were omitted because these would simply multiply by 1, and divide by 1, so the result would be unchanged.

<a id="ref-for-premultiplied①"></a>

In the general case, colors may have non-unity alpha components and thus the [premultiply](https://www.w3.org/TR/css-color-4/#premultiplied), interpolate, unpremultiply steps must not be omitted.

<a id="ref-for-premultiplied②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-premultiply-srgb"></a> This example is 25% semi-opaque red and 75% semi-opaque green. mixed in sRGB. Both the correct ([premultiplied](https://www.w3.org/TR/css-color-4/#premultiplied)) and incorrect (non-premultiplied) workings are shown.
>
> ```css
> color-mix(in srgb, rgb(100% 0% 0% / 0.7) 25%, rgb(0% 100% 0% / 0.2));
> ```
>
> The calcuation is as follows:
>
> -  rgb(100% 0% 0% / 0.7) when premultiplied, is \[0.7, 0, 0\]
>
> -  rgb(0% 100% 0% / 0.2) when premultiplied, is \[0, 0.2, 0\]
>
> - the premultiplied, interpolated result is \[0.7 \* 0.25 + 0 \* (1 - 0.25), 0 \* 0.25 + 0.2 \* (1 - 0.25), 0 \* 0.25 + 0 \* (1 - 0.25)\] which is \[0.175, 0.150, 0\]
>
> - the interpolated alpha is 0.7 \* 0.25 + 0.2 \* (1 - 0.25) = 0.325
>
> - the un-premultiplied result is \[0.175 / 0.325, 0.150 / 0.325, 0 / 0.325\] which is \[0.53846, 0.46154, 0\]
>
> - so the mixed color is color(srgb 0.53846 0.46154 0 / 0.325)
>
> The <em>incorrect</em> calculation would be:
>
> - the interpolated result is \[1 \* 0.25 + 0 \* (1 - 0.25), 0 \* 0.25 + 1 \* (1 - 0.25), 0 \* 0.25 + 0 \* (1 - 0.25)\] which is \[0.25, 0.75, 0\]
>
> - so the <em>incorrect</em> mixed color is color(srgb 0.25 0.75 0 / 0.325)
>
> This is a <em>huge</em> difference; the ΔE2000 between the correct and incorrect results is 30.7!

When the percentage normalization generates an alpha multiplier, the calculation is the same except for an additional last step.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-premultiply-srgb-2"></a> This example is similar to the previous one, 25% semi-opaque red and 75% semi-opaque green. mixed in sRGB.
>
> However in this case the percentages are specified as 20% of the first color and 60% of the second. This adds to 80% so the alpha multiplier is 0.8.
>
> The mix percentages are then scaled by a factor of 100/80:  
> 20% \* 100/80 = 25%  
> 60% \* 100/80 = 75%  
> giving the same final mix percentages as the previous example.
>
> ```css
> color-mix(in srgb, rgb(100% 0% 0% / 0.7) 20%, rgb(0% 100% 0% / 0.2) 60%);
> ```
>
> The calcuation is as follows:
>
> -  rgb(100% 0% 0% / 0.7) when premultiplied, is \[0.7, 0, 0\]
>
> -  rgb(0% 100% 0% / 0.2) when premultiplied, is \[0, 0.2, 0\]
>
> - the premultiplied, interpolated result is \[0.7 \* 0.25 + 0 \* (1 - 0.25), 0 \* 0.25 + 0.2 \* (1 - 0.25), 0 \* 0.25 + 0 \* (1 - 0.25)\] which is \[0.175, 0.150, 0\]
>
> - the interpolated alpha is 0.7 \* 0.25 + 0.2 \* (1 - 0.25) = 0.325
>
> - the un-premultiplied result is \[0.175 / 0.325, 0.150 / 0.325, 0 / 0.325\] which is \[0.53846, 0.46154, 0\]
>
> - so the mixed color would be color(srgb 0.53846 0.46154 0 / 0.325)
>
> - there is a 0.8 alpha multiplier, so the alpha of the mixed result is actually 0.325 \* 0.8 = 0.260 so the mixed color is actually color(srgb 0.53846 0.46154 0 / 0.260)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: do not multiply the interpolated alpha by the alpha multiplier and then use that to undo premultiplication. That would be correct if the mix percentages were not scaled to sum to 100%, but they are, so doing it this way would adjust the mixed color <em>twice</em>.

## <a id="relative-colors"></a>4.  Relative Colors 

### <a id="rcs-intro"></a>4.1.  Processing Model for Relative Colors 

In previous levels of this specification, the color functions could only specify colors in an absolute manner, by directly specifying all of the color components.

<a id="ref-for-modern-color-syntax"></a>

<a id="ref-for-math-function"></a>

The new <a id="relative-color"></a>relative color syntax extends [modern color syntax](https://www.w3.org/TR/css-color-4/#modern-color-syntax) to allow existing colors to be modified using the color functions: if an <a id="origin-color"></a>origin color is specified, then each color component (and the alpha component, if specified) can <em>either</em> be directly specified, or taken from the origin color (and possibly modified with [math functions](https://www.w3.org/TR/css-values-4/#math-function)).

Most relative colors allow modification of both the color components and the alpha component. However, there is a simpler and more restricted form in which only the alpha component is modified.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-easy-rcs"></a> For example, here we use relative colors to make a less vibrant variant of an existing color:
>
> ```text
> --base:  gold;
> --softerbase:  oklch(from var(--base) l calc(c * 0.9) h);
> ```
>
> While here we change just the opacity:
>
> ```text
> --marker:  teal;
> --palemarker:  alpha(from var(--marker) / 0.7);
> ```
<a id="ref-for-color-space"></a>

The origin color and the relative color need not use the same color function. Thus, we define a <a id="relative-color-processing-space"></a>relative color processing space, which is the [color space](https://www.w3.org/TR/css-color-4/#color-space) within which computation of color values takes place; this also affects the serialization of the result of the relative color.

- <a id="ref-for-relative-color-processing-space"></a>

  For the alpha-only form, the [relative color processing space](#relative-color-processing-space) is that of the origin color.

- <a id="ref-for-relative-color-processing-space①"></a>

  For the color-affecting forms, the [relative color processing space](#relative-color-processing-space) is that of the relative color function.

<a id="ref-for-relative-color-processing-space②"></a>

<a id="ref-for-origin-color"></a>

<a id="ref-for-component-keyword"></a>

<a id="conversion-if-required"></a>Conversion, if required: All operations take part in the [relative color processing space](#relative-color-processing-space). If the <a id="originally-specified-color-space"></a>originally specified color space for the [origin color](#origin-color) used a different color function, it’s first converted into the <a id="ref-for-relative-color-processing-space③"></a>relative color processing space, so it has meaningful values for the components, and color [component keywords](#component-keyword) refer to that color space.

<a id="ref-for-origin-color①"></a>

If the alpha value of the relative color is omitted, it defaults to that of the [origin color](#origin-color) (rather than defaulting to 100%, as it does in the absolute syntax).

When relative color syntax is used, <strong>color component</strong> values, whether directly specified or arising from color space conversion, are <em>not clamped</em> to the reference ranges but are retained as-is. This preserves out of gamut values, if the destination color space is capable of representing them.

However, when relative color syntax is used, <strong>alpha component</strong> values whether directly specified or arising from color space conversion, <em>are</em> clamped to the reference range.

<a id="ref-for-analogous-components"></a>

<a id="ref-for-carried-forward"></a>

Missing components are handled the same way as with [CSS Color 4 § 13.3 Interpolating with Missing Components](https://www.w3.org/TR/css-color-4/#interpolation-missing): the origin colorspace and the relative function colorspace are checked for [analogous components](https://www.w3.org/TR/css-color-4/#analogous-components) which are then [carried forward](https://www.w3.org/TR/css-color-4/#carried-forward) as missing.

<a id="ref-for-relative-color"></a>

<a id="ref-for-component-keyword①"></a>

While most uses of [relative color](#relative-color) syntax will use the [component keywords](#component-keyword) in their corresponding argument, you can use them in any position.

Beware when using components outside their normal position; when percentages are resolved to numbers, there is no "magic scaling" to account for the changed position if those numbers are used in a different place.

<a id="ref-for-funcdef-device-cmyk③"></a>

There is no relative [device-cmyk()](#funcdef-device-cmyk) syntax.

### <a id="relative-syntax"></a>4.2.  Relative Color Syntax 

<a id="ref-for-relative-color①"></a>

The precise details of each function’s syntactic changes to accommodate [relative colors](#relative-color) are listed below, but they all follow a common structure:

- <a id="ref-for-origin-color②"></a>

  <a id="ref-for-typedef-color⑤"></a>

  An [origin color](#origin-color) can be specified with a from [\<color\>](#typedef-color) value at the start of the function. This includes the optional alpha component, if specified.

- If no origin color is specified, the function is not a relative color.

- <a id="ref-for-origin-color③"></a>

  <a id="ref-for-relative-color-processing-space④"></a>

  <a id="ref-for-math-function①"></a>

  If an [origin color](#origin-color) is specified, the remaining arguments can either be specified directly, as normal, or be specified as a <a id="component-keyword"></a>component keyword referring to one of the components of the <a id="ref-for-origin-color④"></a>origin color converted to the [relative color processing space](#relative-color-processing-space). [Math functions](https://www.w3.org/TR/css-values-4/#math-function) can also use these keywords to do dynamic modifications of the <a id="ref-for-origin-color⑤"></a>origin color’s components.

- <a id="ref-for-relative-color②"></a>

  [Relative color](#relative-color) syntax doesn’t change whether an argument is required or optional.

- <a id="ref-for-modern-color-syntax①"></a>

  <a id="ref-for-legacy-color-syntax"></a>

  Relative color syntax only applies to the [modern color syntax](https://www.w3.org/TR/css-color-4/#modern-color-syntax). It <em>cannot</em> be used with [legacy color syntax](https://www.w3.org/TR/css-color-4/#legacy-color-syntax) and attempting to do so is an error.

- <a id="ref-for-origin-color⑥"></a>

  However, the [origin color](#origin-color) can use either modern or legacy syntax.

<a id="ref-for-component-keyword②"></a>

<a id="ref-for-number-value"></a>

<a id="ref-for-valdef-light-dark-none"></a>

<a id="ref-for-percentage-value①"></a>

<a id="ref-for-angle-value"></a>

<a id="ref-for-canonical-unit"></a>

The [component keywords](#component-keyword) return a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), or [none](#valdef-light-dark-none); if they were originally specified as a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) or an [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), that <a id="ref-for-percentage-value②"></a>\<percentage\> is resolved to a <a id="ref-for-number-value①"></a>\<number\> and the <a id="ref-for-angle-value①"></a>\<angle\> is resolved to a <a id="ref-for-number-value②"></a>\<number\> of degrees (which is the [canonical unit](https://www.w3.org/TR/css-values-4/#canonical-unit)) in the range \[0, 360\].

Tests

- [none-components-treated-as-zero.html](https://wpt.fyi/results/css/css-color/none-components-treated-as-zero.html) [(live test)](http://wpt.live/css/css-color/none-components-treated-as-zero.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/none-components-treated-as-zero.html)
- [relative-color-with-zoom.html](https://wpt.fyi/results/css/css-color/relative-color-with-zoom.html) [(live test)](http://wpt.live/css/css-color/relative-color-with-zoom.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-color-with-zoom.html)
- [relative-currentcolor-a98rgb-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-a98rgb-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-a98rgb-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-a98rgb-01.html)
- [relative-currentcolor-lch-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-lch-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-lch-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-lch-01.html)
- [relative-currentcolor-rgb-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-rgb-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-rgb-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-rgb-01.html)
- [relative-currentcolor-displayp3-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-displayp3-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-displayp3-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-displayp3-01.html)
- [relative-currentcolor-oklab-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-oklab-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-oklab-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-oklab-01.html)
- [relative-currentcolor-rgb-02.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-rgb-02.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-rgb-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-rgb-02.html)
- [relative-currentcolor-hsl-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-hsl-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-hsl-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-hsl-01.html)
- [relative-currentcolor-oklch-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-oklch-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-oklch-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-oklch-01.html)
- [relative-currentcolor-xyzd50-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-xyzd50-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-xyzd50-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-xyzd50-01.html)
- [relative-currentcolor-hsl-02.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-hsl-02.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-hsl-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-hsl-02.html)
- [relative-currentcolor-prophoto-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-prophoto-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-prophoto-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-prophoto-01.html)
- [relative-currentcolor-xyzd65-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-xyzd65-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-xyzd65-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-xyzd65-01.html)
- [relative-currentcolor-hwb-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-hwb-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-hwb-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-hwb-01.html)
- [relative-currentcolor-rec2020-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-rec2020-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-rec2020-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-rec2020-01.html)
- [relative-currentcolor-lab-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-lab-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-lab-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-lab-01.html)
- [relative-currentcolor-rec2020-02.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-rec2020-02.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-rec2020-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-rec2020-02.html)
- [relative-currentcolor-visited-getcomputedstyle.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-visited-getcomputedstyle.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-visited-getcomputedstyle.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-visited-getcomputedstyle.html)
- [relative-color-with-system-color.html](https://wpt.fyi/results/css/css-color/relative-color-with-system-color.html) [(live test)](http://wpt.live/css/css-color/relative-color-with-system-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-color-with-system-color.html)
- [alpha-color-computed.html](https://wpt.fyi/results/css/css-color/parsing/alpha-color-computed.html) [(live test)](http://wpt.live/css/css-color/parsing/alpha-color-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/alpha-color-computed.html)
- [alpha-color-parsing-invalid.html](https://wpt.fyi/results/css/css-color/parsing/alpha-color-parsing-invalid.html) [(live test)](http://wpt.live/css/css-color/parsing/alpha-color-parsing-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/alpha-color-parsing-invalid.html)
- [alpha-color-parsing-valid.html](https://wpt.fyi/results/css/css-color/parsing/alpha-color-parsing-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/alpha-color-parsing-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/alpha-color-parsing-valid.html)
- [color-computed-relative-color.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-relative-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-relative-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-relative-color.html)
- [color-invalid-relative-color.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-relative-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-relative-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-relative-color.html)
- [color-valid-relative-color.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-relative-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-relative-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-relative-color.html)
- [relative-color-out-of-gamut.html](https://wpt.fyi/results/css/css-color/parsing/relative-color-out-of-gamut.html) [(live test)](http://wpt.live/css/css-color/parsing/relative-color-out-of-gamut.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/relative-color-out-of-gamut.html)

<a id="ref-for-percentage-value③"></a>

<a id="ref-for-number-value③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1f6216e3"></a> For example, if a color is specified using [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), then RCS in the same colorspace will use the resolved [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) form:
>
> ```text
> html { --bluegreen:  oklab(54.3% -22.5% -5%); }
> .overlay {
>   background:  oklab(from var(--bluegreen) calc(1.0 - l) calc(a * 0.8) b);
> }
> ```
>
> In this example, the specified percentages are resolved to numbers, giving oklab(0.543 -0.09 -0.02). The resulting RCS color has l = 1 - 0.543 = 0.457, a = -0.09 \* 0.8 = -0.072, and b is unchanged at -0.02: oklab(0.457 -0.072 -0.02).

<a id="ref-for-angle-value②"></a>

<a id="ref-for-number-value④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4889a5cd"></a> For example, if the origin color has a hue [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) specified in degrees, then RCS in the same colorspace will use the resolved [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) form:
>
> ```text
> html { --base:  oklch(52.6% 0.115 44.6deg) }
> .summary {
>   background:  oklch(from var(--base) l c  calc(h + 90));
> }
> ```
>
> In this example the resulting RCS color is oklch(0.526 0.115 134.6).
>
> <a id="ref-for-angle-value③"></a>
>
> <a id="ref-for-number-value⑤"></a>
>
> Had the origin color hue [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) been specified in another unit, such as radians or turns, still the resolved [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) would be the number of degrees.

<a id="ref-for-component-keyword③"></a>

<a id="ref-for-math-function②"></a>

<a id="ref-for-origin-color⑦"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2a49656f"></a> By using the [component keywords](#component-keyword) in a [math function](https://www.w3.org/TR/css-values-4/#math-function), an [origin color](#origin-color) can be manipulated in more advanced ways.
>
> ```text
> html { --color: green; }
> .foo {
>   --darker-accent: lch(from var(--color) calc(l / 2) c h);
> }
> ```
>
> <a id="ref-for-origin-color⑧"></a>
>
> In this example, the [origin color](#origin-color) is darkened by cutting its lightness in half, without changing any other aspect of the color.
>
> <a id="ref-for-origin-color⑨"></a>
>
> <a id="ref-for-funcdef-lch②"></a>
>
> Note as well that the [origin color](#origin-color) is a color keyword (and thus, sRGB), but it’s automatically interpreted as an LCH color due to being used in the [lch()](#funcdef-lch) function.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-90bb3262"></a> For example, if a theme color is specified as opaque, but in a particular instance you need it to be partially transparent:
>
> ```text
> html { --bg-color:  blue; }
> .overlay {
>   background:  rgb(from var(--bg-color) r g b / 80%);
> }
> ```
>
> <a id="ref-for-origin-color①⓪"></a>
>
> In this example, the r, g, and b components of the [origin color](#origin-color) are unchanged, indicated by specifying them with the keywords drawing their values from the <a id="ref-for-origin-color①①"></a>origin color, but the opacity is set to 80% to make it slightly transparent, regardless of what the <a id="ref-for-origin-color①②"></a>origin color’s opacity was.
>
> While this example is syntactically correct, and illustrates what happens, there is a simpler form for this specific case of alpha-only manipulation, which avoids having to list the unchanged color components:
>
> ```text
> html { --bg-color:  blue; }
> .overlay {
>   background:  alpha(from var(--bg-color) / 80%);
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b1f4bd13"></a> For example, a Display P3 color which is outside the gamut of sRGB can still be represented, as it is not clipped.
>
> ```text
> --vivid-yellow:  color(display-p3 1 1 0); 
> --paler-yellow:  color(from var(--vivid-yellow) srgb r g calc(b + 0.5));
> ```
>
> Here --vivid-yellow, once converted to sRGB, is rgb(100% 100% -34.63%) and the negative blue component is not clamped. The result of the RCS calculation is rgb(100% 100% 15.37%)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4bad0aeb"></a> For example, attempting to double an alpha of 0.7 in the origin color results in an alpha in the result of 1, not 1.4.
>
> ```text
> --tan:  oklch(78% 0.06 75 / 0.7);
> --deeper-tan:  oklch(from var(--tan) l c h / calc(alpha * 2));
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-2bf59389"></a> For example, to do a rough approximation of grayscaling a color:
>
> ```text
> --blue-into-gray: rgb(from var(--color)
>                     calc(r * .3 + g * .59 + b * .11)
>                     calc(r * .3 + g * .59 + b * .11)
>                     calc(r * .3 + g * .59 + b * .11));
> ```
>
> <a id="ref-for-valdef-color-red"></a>
>
> <a id="ref-for-valdef-color-lime"></a>
>
> <a id="ref-for-valdef-color-blue"></a>
>
> <a id="ref-for-valdef-color-darkolivegreen"></a>
>
> Using this, [red](https://www.w3.org/TR/css-color-4/#valdef-color-red) would become rgb(76.5 76.5 76.5), [lime](https://www.w3.org/TR/css-color-4/#valdef-color-lime) would become rgb(150.45 150.45 150.45), and [blue](https://www.w3.org/TR/css-color-4/#valdef-color-blue) would become rgb(150.45 150.45 150.45). A more moderate color, like [darkolivegreen](https://www.w3.org/TR/css-color-4/#valdef-color-darkolivegreen), which has RGB values rgb(85 107 47), would become rgb(93.8 93.8 93.8).
>
> (Rough because firstly, although this looks like a luminance calculation, the red green and blue values are manipulated in gamma-encoded space rather than linear-light; secondly, the weighting factors are those for the obsolete NTSC color space, not sRGB.)
>
> <a id="ref-for-funcdef-oklch②"></a>
>
> (Note, too, that this is just to illustrate the syntax; an easier and more accurate way to grayscale a color is to use the [oklch()](#funcdef-oklch) function, as that color space is more accurate to human perception: oklch(from var(--color) l 0 h) preserves the lightness, but zeroes out the chroma, which determines how "colorful" the color is.)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-no-percentage-magic"></a>
>
> For example,
>
> ```text
> color: color(from color(srgb 0 0 0 / 60%) srgb alpha 0.6 0.6 / 0.9);
> ```
>
> <a id="ref-for-number-value⑥"></a>
>
> The alpha component is resolved as a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), giving 0.6; thus the resulting color is color(srgb 0.6 0.6 0.6 / 0.9).
>
> However, in this second example, again the alpha resolves to 0.6, giving a very different color due to the color component range of 0 to 255 in rgb() syntax:
>
> ```text
> color: rgb(from rgb(0 0 0 / 60%) alpha 153 153 / 0.9);
> ```
>
> which results in rgb(0.6 153 153 / 0.9) and <em>not</em> rgb(153 153 153 / 0.9).

<a id="ref-for-relative-color③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-missing-analogous"></a> In this example the achromatic origin color has a missing hue; the [relative color](#relative-color) also has a missing hue, which affects a gradient using that color.
>
> ```text
> html { --bg:  hsl(none 3% 50%); }
> .foo {
>   --darker-bg:  oklch(from var(--bg) calc(l * 0.8) c h);
> }
> .bar {
>   background: linear-gradient(in Oklab to right,   var(--darker-bg),   #4C3);
> }
> ```
>
> The value of --bg when converted to OkLCh is oklch(0.592 0.009 17.42) but the analogous hue component is carried forward giving oklch(0.592 0.009 none). These values are then used in the relative function, giving the darker color oklch(0.474 0.009 none).
>
> The light green in the gradient is oklch(0.743 0.222 141.6), and so, when interpolated, the other color take that hue, becoming oklch(0.474 0.009 141.6).
>
> Thus, the gradient will have a constant greenish hue.
>
> If an implementation failed to do this carrying forward, the grayish --darker-bg would have a hue of 0, giving an undesirable reddish tint at the start of the gradient.
>
> <a id="fig-gradient-missing"></a> ![](https://www.w3.org/TR/2026/WD-css-color-5-20260908/images/rectangular-missing.png)
>
> Correct (above) and incorrect (below, reddish) gradients.

<a id="ref-for-valdef-light-dark-none①"></a>

However, if calculations are done on missing values, [none](#valdef-light-dark-none) is treated as 0.

### <a id="relative-RGB"></a>4.3. Relative sRGB Colors

The meaning of sRGB colors is defined in [CSS Color 4 § 5 sRGB Colors](https://www.w3.org/TR/css-color-4/#numeric-srgb).

<a id="ref-for-modern-color-syntax②"></a>

<a id="ref-for-funcdef-rgb③"></a>

<a id="ref-for-funcdef-rgba③"></a>

The grammar of the [modern color syntax](https://www.w3.org/TR/css-color-4/#modern-color-syntax) [rgb()](https://www.w3.org/TR/css-color-4/#funcdef-rgb) and [rgba()](https://www.w3.org/TR/css-color-4/#funcdef-rgba) functions are extended as follows:

<a id="typedef-modern-rgb-syntax"></a>

<a id="ref-for-typedef-modern-rgb-syntax"></a>

<a id="ref-for-typedef-color⑥"></a>

<a id="ref-for-mult-opt②"></a>

<a id="ref-for-number-value⑦"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-percentage-value④"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-mult-num"></a>

<a id="ref-for-typedef-color-alpha-value"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-mult-opt③"></a>

<a id="typedef-modern-rgba-syntax"></a>

<a id="ref-for-typedef-modern-rgba-syntax"></a>

<a id="ref-for-typedef-color⑦"></a>

<a id="ref-for-mult-opt④"></a>

<a id="ref-for-number-value⑧"></a>

<a id="ref-for-comb-one②②"></a>

<a id="ref-for-percentage-value⑤"></a>

<a id="ref-for-comb-one②③"></a>

<a id="ref-for-mult-num①"></a>

<a id="ref-for-typedef-color-alpha-value①"></a>

<a id="ref-for-comb-one②④"></a>

<a id="ref-for-mult-opt⑤"></a>

```text
<modern-rgb-syntax> = rgb( [ from <color> ]?
        [ <number> | <percentage> | none]{3}
        [ / [<alpha-value> | none] ]?  )
<modern-rgba-syntax> = rgba( [ from <color> ]?
        [ <number> | <percentage> | none]{3}
        [ / [<alpha-value> | none] ]?  )
```
<a id="ref-for-relative-color④"></a>

<a id="ref-for-funcdef-rgb④"></a>

<a id="ref-for-funcdef-rgba④"></a>

<a id="ref-for-component-keyword④"></a>

Within a [relative color](#relative-color) syntax [rgb()](https://www.w3.org/TR/css-color-4/#funcdef-rgb) or [rgba()](https://www.w3.org/TR/css-color-4/#funcdef-rgba) function, the allowed [component keywords](#component-keyword) are:

- <a id="ref-for-number-value⑨"></a>

  <a id="ref-for-origin-color①③"></a>

  <a id="ref-for-conversion-if-required"></a>

  <a id="valdef-rgb-r"></a>r, <a id="valdef-rgb-g"></a>g, and <a id="valdef-rgb-b"></a>b are all [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s that correspond to the [origin color’s](#origin-color) red, green, and blue components after [conversion, if required](#conversion-if-required) to sRGB. 255.0 is equivalent to 100%.

- <a id="ref-for-number-value①⓪"></a>

  <a id="ref-for-origin-color①④"></a>

  <a id="valdef-rgb-alpha"></a>alpha is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) alpha transparency. 1.0 is equivalent to 100%.

Tests

- [relative-currentcolor-rgb-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-rgb-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-rgb-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-rgb-01.html)
- [relative-currentcolor-rgb-02.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-rgb-02.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-rgb-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-rgb-02.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-cebe6396"></a> To manipulate color components in the sRGB color space:
>
> ```text
> rgb(from  indianred 255 g b)
> ```
>
> This takes the sRGB value of indianred (205 92 92) and replaces the red component with 255 to give rgb(255 92 92).

Relative sRGB color syntax is <em>only</em> applicable to the non-legacy RGB syntactic forms.

<a id="ref-for-legacy-color-syntax①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-648e776c"></a> For example, this attempt to use the rgba [legacy color syntax](https://www.w3.org/TR/css-color-4/#legacy-color-syntax) with commas would be incorrect
>
> ```text
> rgba(from  darkblue 16, 32, b, 0.5 )
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c4b962d1"></a> Instead, use
>
> ```text
> rgb(from  darkblue 16 32 b / 0.5 )
> ```
>
> This takes the sRGB value of darkblue (0 0 139) and replaces the red, green and alpha components to give rgb(16 32 139 / 0.5)

### <a id="relative-HSL"></a>4.4. Relative HSL Colors

The meaning of HSL colors is defined in [CSS Color 4 § 7 HSL Colors: hsl() and hsla() functions](https://www.w3.org/TR/css-color-4/#the-hsl-notation).

<a id="ref-for-modern-color-syntax③"></a>

<a id="ref-for-funcdef-hsl④"></a>

<a id="ref-for-funcdef-hsla④"></a>

The grammar of the [modern color syntax](https://www.w3.org/TR/css-color-4/#modern-color-syntax) [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) and [hsla()](https://www.w3.org/TR/css-color-4/#funcdef-hsla) functions is extended as follows:

<a id="typedef-modern-hsl-syntax"></a>

<a id="ref-for-typedef-modern-hsl-syntax"></a>

<a id="ref-for-typedef-color⑧"></a>

<a id="ref-for-mult-opt⑥"></a>

<a id="ref-for-typedef-hue①"></a>

<a id="ref-for-comb-one②⑤"></a>

<a id="ref-for-percentage-value⑥"></a>

<a id="ref-for-comb-one②⑥"></a>

<a id="ref-for-number-value①①"></a>

<a id="ref-for-comb-one②⑦"></a>

<a id="ref-for-percentage-value⑦"></a>

<a id="ref-for-comb-one②⑧"></a>

<a id="ref-for-number-value①②"></a>

<a id="ref-for-comb-one②⑨"></a>

<a id="ref-for-typedef-color-alpha-value②"></a>

<a id="ref-for-comb-one③⓪"></a>

<a id="ref-for-mult-opt⑦"></a>

<a id="typedef-modern-hsla-syntax"></a>

<a id="ref-for-typedef-modern-hsla-syntax"></a>

<a id="ref-for-typedef-color⑨"></a>

<a id="ref-for-mult-opt⑧"></a>

<a id="ref-for-typedef-hue②"></a>

<a id="ref-for-comb-one③①"></a>

<a id="ref-for-percentage-value⑧"></a>

<a id="ref-for-comb-one③②"></a>

<a id="ref-for-number-value①③"></a>

<a id="ref-for-comb-one③③"></a>

<a id="ref-for-percentage-value⑨"></a>

<a id="ref-for-comb-one③④"></a>

<a id="ref-for-number-value①④"></a>

<a id="ref-for-comb-one③⑤"></a>

<a id="ref-for-typedef-color-alpha-value③"></a>

<a id="ref-for-comb-one③⑥"></a>

<a id="ref-for-mult-opt⑨"></a>

```text
<modern-hsl-syntax> = hsl([from <color>]?
          [<hue> | none]
          [<percentage> | <number> | none]
          [<percentage> | <number> | none]
          [ / [<alpha-value> | none] ]? )
<modern-hsla-syntax> = hsla([from <color>]?
        [<hue> | none]
        [<percentage> | <number> | none]
        [<percentage> | <number> | none]
        [ / [<alpha-value> | none] ]? )
```
<a id="ref-for-relative-color⑤"></a>

<a id="ref-for-funcdef-hsl⑤"></a>

<a id="ref-for-funcdef-hsla⑤"></a>

<a id="ref-for-component-keyword⑤"></a>

Within a [relative color](#relative-color) syntax [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) or [hsla()](https://www.w3.org/TR/css-color-4/#funcdef-hsla) function, the allowed [component keywords](#component-keyword) are:

- <a id="ref-for-number-value①⑤"></a>

  <a id="ref-for-origin-color①⑤"></a>

  <a id="ref-for-conversion-if-required①"></a>

  <a id="valdef-hsl-h"></a>h is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) HSL hue, in degrees, after [conversion, if required](#conversion-if-required) to sRGB, normalized to a \[0, 360\] range. 90 is equivalent to 90deg.

- <a id="ref-for-number-value①⑥"></a>

  <a id="ref-for-origin-color①⑥"></a>

  <a id="ref-for-conversion-if-required②"></a>

  <a id="valdef-hsl-s"></a>s and <a id="valdef-hsl-l"></a>l are [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s that correspond to the [origin color’s](#origin-color) HSL saturation and lightness, after [conversion, if required](#conversion-if-required) to sRGB. 100 is equivalent to 100%.

- <a id="ref-for-number-value①⑦"></a>

  <a id="ref-for-origin-color①⑦"></a>

  <a id="valdef-hsl-alpha"></a>alpha is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) alpha transparency 1.0 is equivalent to 100%.

Tests

- [relative-currentcolor-hsl-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-hsl-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-hsl-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-hsl-01.html)
- [relative-currentcolor-hsl-02.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-hsl-02.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-hsl-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-hsl-02.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c4b276dd"></a> This adds 180 degrees to the hue angle, giving a complementary color.
>
> ```text
> --accent:  lightseagreen;
> --complement:   hsl(from var(--accent) calc(h + 180) s l);
> ```
>
> lightseagreen is hsl(177deg 70% 41%), so --complement is hsl(357deg 70% 41%)

Relative HSL color syntax is only applicable to the non-legacy HSL syntactic forms.

### <a id="relative-HWB"></a>4.5. Relative HWB Colors

The meaning of HWB colors is defined in [CSS Color 4 § 8 HWB Colors: hwb() function](https://www.w3.org/TR/css-color-4/#the-hwb-notation).

<a id="ref-for-funcdef-hwb③"></a>

The grammar of the [hwb()](#funcdef-hwb) function is extended as follows:

<a id="funcdef-hwb"></a>

<a id="ref-for-typedef-color①⓪"></a>

<a id="ref-for-mult-opt①⓪"></a>

<a id="ref-for-typedef-hue③"></a>

<a id="ref-for-comb-one③⑦"></a>

<a id="ref-for-percentage-value①⓪"></a>

<a id="ref-for-comb-one③⑧"></a>

<a id="ref-for-number-value①⑧"></a>

<a id="ref-for-comb-one③⑨"></a>

<a id="ref-for-percentage-value①①"></a>

<a id="ref-for-comb-one④⓪"></a>

<a id="ref-for-number-value①⑨"></a>

<a id="ref-for-comb-one④①"></a>

<a id="ref-for-typedef-color-alpha-value④"></a>

<a id="ref-for-comb-one④②"></a>

<a id="ref-for-mult-opt①①"></a>

```text
hwb() = hwb([from <color>]?
        [<hue> | none]
        [<percentage> | <number> | none]
        [<percentage> | <number> | none]
        [ / [<alpha-value> | none] ]? )
```
<a id="ref-for-relative-color⑥"></a>

<a id="ref-for-funcdef-hwb④"></a>

<a id="ref-for-component-keyword⑥"></a>

Within a [relative color](#relative-color) syntax [hwb()](#funcdef-hwb) function, the allowed [component keywords](#component-keyword) are:

- <a id="ref-for-number-value②⓪"></a>

  <a id="ref-for-origin-color①⑧"></a>

  <a id="ref-for-conversion-if-required③"></a>

  <a id="valdef-hwb-h"></a>h is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) HWB hue, in degrees, after [conversion, if required](#conversion-if-required) to sRGB, normalized to a \[0, 360\] range. 90 is equivalent to 90deg.

- <a id="ref-for-number-value②①"></a>

  <a id="ref-for-origin-color①⑨"></a>

  <a id="ref-for-conversion-if-required④"></a>

  <a id="valdef-hwb-w"></a>w and <a id="valdef-hwb-b"></a>b are [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s that correspond to the [origin color’s](#origin-color) HWB whiteness and blackness after [conversion, if required](#conversion-if-required) to sRGB. 100 is equivalent to 100%.

- <a id="ref-for-number-value②②"></a>

  <a id="ref-for-origin-color②⓪"></a>

  <a id="valdef-hwb-alpha"></a>alpha is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) alpha transparency. 1.0 is equivalent to 100%.

Tests

- [relative-currentcolor-hwb-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-hwb-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-hwb-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-hwb-01.html)

### <a id="relative-Lab"></a>4.6. Relative Lab Colors

The meaning of Lab colors is defined in [CSS Color 4 § 9.1 CIE Lab and LCH](https://www.w3.org/TR/css-color-4/#cie-lab).

<a id="ref-for-funcdef-lab②"></a>

The grammar of the [lab()](#funcdef-lab) function is extended as follows:

<a id="funcdef-lab"></a>

<a id="ref-for-typedef-color①①"></a>

<a id="ref-for-mult-opt①②"></a>

<a id="ref-for-percentage-value①②"></a>

<a id="ref-for-comb-one④③"></a>

<a id="ref-for-number-value②③"></a>

<a id="ref-for-comb-one④④"></a>

<a id="ref-for-percentage-value①③"></a>

<a id="ref-for-comb-one④⑤"></a>

<a id="ref-for-number-value②④"></a>

<a id="ref-for-comb-one④⑥"></a>

<a id="ref-for-percentage-value①④"></a>

<a id="ref-for-comb-one④⑦"></a>

<a id="ref-for-number-value②⑤"></a>

<a id="ref-for-comb-one④⑧"></a>

<a id="ref-for-typedef-color-alpha-value⑤"></a>

<a id="ref-for-comb-one④⑨"></a>

<a id="ref-for-mult-opt①③"></a>

```text
lab() = lab([from <color>]?
        [<percentage> | <number> | none]
        [<percentage> | <number> | none]
        [<percentage> | <number> | none]
        [ / [<alpha-value> | none] ]? )
```
<a id="ref-for-relative-color⑦"></a>

<a id="ref-for-funcdef-lab③"></a>

<a id="ref-for-component-keyword⑦"></a>

Within a [relative color](#relative-color) syntax [lab()](#funcdef-lab) function, the allowed [component keywords](#component-keyword) are:

- <a id="ref-for-number-value②⑥"></a>

  <a id="ref-for-origin-color②①"></a>

  <a id="ref-for-conversion-if-required⑤"></a>

  <a id="valdef-lab-l"></a>l is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) CIE Lightness after [conversion, if required](#conversion-if-required), to CIE Lab. 100 is equivalent to 100%.

- <a id="ref-for-number-value②⑦"></a>

  <a id="ref-for-origin-color②②"></a>

  <a id="ref-for-conversion-if-required⑥"></a>

  <a id="valdef-lab-a"></a>a and <a id="valdef-lab-b"></a>b are [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s that correspond to the [origin color’s](#origin-color) CIE Lab a and b axes after [conversion, if required](#conversion-if-required), to CIE Lab. 125 is equivalent to 100%, while -125 is equivalent to -100%.

- <a id="ref-for-number-value②⑧"></a>

  <a id="ref-for-origin-color②③"></a>

  <a id="valdef-lab-alpha"></a>alpha is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) alpha transparency. 1.0 is equivalent to 100%.

  Tests
  - [relative-currentcolor-lab-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-lab-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-lab-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-lab-01.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b411f1d3"></a> Multiple ways to adjust the transparency of a base color:
>
> - lab(from var(--mycolor) l a b / 100%) sets the alpha of var(--mycolor) to 1.0, regardless of what it originally was.
>
> - lab(from var(--mycolor) l a b / calc(alpha \* 0.8)) reduces the alpha of var(--mycolor) by 20% of its original value.
>
> Note that all the adjustments are lossless in the sense that no gamut clipping occurs, since lab() encompasses all visible color. This is not true for the alpha adjustments in the sRGB based functions (such as 'rgb()', 'hsl()', or 'hwb()'), which would also convert to sRGB as a necessary step for calculation of HSL or HWB, in addition to adjusting the alpha transparency.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-d39afaf1"></a> Fully desaturating a color to gray, keeping the exact same lightness:
>
> ```text
> --mycolor:  orchid;
> // orchid is lab(62.753 52.460 -34.103)
> --mygray:  lab(from var(--mycolor) l 0 0)
> // mygray is lab(62.753 0 0) which is rgb(59.515% 59.515% 59.515%)
> ```
### <a id="relative-Oklab"></a>4.7. Relative Oklab Colors

The meaning of Oklab colors is defined in [CSS Color 4 § 9.2 Oklab and OkLCh](https://www.w3.org/TR/css-color-4/#ok-lab).

<a id="ref-for-funcdef-oklab①"></a>

The grammar of the [oklab()](#funcdef-oklab) function is extended as follows:

<a id="funcdef-oklab"></a>

<a id="ref-for-typedef-color①②"></a>

<a id="ref-for-mult-opt①④"></a>

<a id="ref-for-percentage-value①⑤"></a>

<a id="ref-for-comb-one⑤⓪"></a>

<a id="ref-for-number-value②⑨"></a>

<a id="ref-for-comb-one⑤①"></a>

<a id="ref-for-percentage-value①⑥"></a>

<a id="ref-for-comb-one⑤②"></a>

<a id="ref-for-number-value③⓪"></a>

<a id="ref-for-comb-one⑤③"></a>

<a id="ref-for-percentage-value①⑦"></a>

<a id="ref-for-comb-one⑤④"></a>

<a id="ref-for-number-value③①"></a>

<a id="ref-for-comb-one⑤⑤"></a>

<a id="ref-for-typedef-color-alpha-value⑥"></a>

<a id="ref-for-comb-one⑤⑥"></a>

<a id="ref-for-mult-opt①⑤"></a>

```text
oklab() = oklab([from <color>]?
          [<percentage> | <number> | none]
          [<percentage> | <number> | none]
          [<percentage> | <number> | none]
          [ / [<alpha-value> | none] ]? )
```
<a id="ref-for-relative-color⑧"></a>

<a id="ref-for-funcdef-oklab②"></a>

<a id="ref-for-component-keyword⑧"></a>

Within a [relative color](#relative-color) syntax [oklab()](#funcdef-oklab) function, the allowed [component keywords](#component-keyword) are:

- <a id="ref-for-number-value③②"></a>

  <a id="ref-for-origin-color②④"></a>

  <a id="ref-for-conversion-if-required⑦"></a>

  <a id="valdef-oklab-l"></a>l is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) Oklab Lightness after [conversion, if required](#conversion-if-required), to Oklab. 1.0 is equivalent to 100%.

- <a id="ref-for-number-value③③"></a>

  <a id="ref-for-origin-color②⑤"></a>

  <a id="ref-for-conversion-if-required⑧"></a>

  <a id="valdef-oklab-a"></a>a and <a id="valdef-oklab-b"></a>b are [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s that correspond to the [origin color’s](#origin-color) Oklab a and b axes after [conversion, if required](#conversion-if-required), to Oklab. 0.4 is equivalent to 100%, while -0.4 is equivalent to -100%.

- <a id="ref-for-number-value③④"></a>

  <a id="ref-for-origin-color②⑥"></a>

  <a id="valdef-oklab-alpha"></a>alpha is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) alpha transparency. 1.0 is equivalent to 100%.

  Tests
  - [relative-currentcolor-oklab-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-oklab-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-oklab-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-oklab-01.html)

### <a id="relative-LCH"></a>4.8. Relative LCH Colors

The meaning of LCH colors is defined in [CSS Color 4 § 9.1 CIE Lab and LCH](https://www.w3.org/TR/css-color-4/#cie-lab).

<a id="ref-for-funcdef-lch③"></a>

The grammar of the [lch()](#funcdef-lch) function is extended as follows:

<a id="funcdef-lch"></a>

<a id="ref-for-typedef-color①③"></a>

<a id="ref-for-mult-opt①⑥"></a>

<a id="ref-for-percentage-value①⑧"></a>

<a id="ref-for-comb-one⑤⑦"></a>

<a id="ref-for-number-value③⑤"></a>

<a id="ref-for-comb-one⑤⑧"></a>

<a id="ref-for-percentage-value①⑨"></a>

<a id="ref-for-comb-one⑤⑨"></a>

<a id="ref-for-number-value③⑥"></a>

<a id="ref-for-comb-one⑥⓪"></a>

<a id="ref-for-typedef-hue④"></a>

<a id="ref-for-comb-one⑥①"></a>

<a id="ref-for-typedef-color-alpha-value⑦"></a>

<a id="ref-for-comb-one⑥②"></a>

<a id="ref-for-mult-opt①⑦"></a>

```text
lch() = lch([from <color>]?
        [<percentage> | <number> | none]
        [<percentage> | <number> | none]
        [<hue> | none]
        [ / [<alpha-value> | none] ]? )
```
<a id="ref-for-relative-color⑨"></a>

<a id="ref-for-funcdef-lch④"></a>

<a id="ref-for-component-keyword⑨"></a>

Within a [relative color](#relative-color) syntax [lch()](#funcdef-lch) function, the allowed [component keywords](#component-keyword) are:

- <a id="ref-for-number-value③⑦"></a>

  <a id="ref-for-origin-color②⑦"></a>

  <a id="ref-for-conversion-if-required⑨"></a>

  <a id="valdef-lch-l"></a>l is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) CIE Lightness after [conversion, if required](#conversion-if-required), to CIE LCH. 100 is equivalent to 100%.

- <a id="ref-for-number-value③⑧"></a>

  <a id="ref-for-origin-color②⑧"></a>

  <a id="ref-for-conversion-if-required①⓪"></a>

  <a id="valdef-lch-c"></a>c is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) LCH chroma after [conversion, if required](#conversion-if-required), to CIE LCH. 150 is equivalent to 100%.

- <a id="ref-for-number-value③⑨"></a>

  <a id="ref-for-origin-color②⑨"></a>

  <a id="ref-for-conversion-if-required①①"></a>

  <a id="valdef-lch-h"></a>h is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) LCH hue, in degrees, after [conversion, if required](#conversion-if-required), to CIE LCH, normalized to a \[0, 360\] range. 90 is equivalent to 90deg.

- <a id="ref-for-number-value④⓪"></a>

  <a id="ref-for-origin-color③⓪"></a>

  <a id="valdef-lch-alpha"></a>alpha is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) alpha transparency. 1.0 is equivalent to 100%.

Tests

- [relative-currentcolor-lch-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-lch-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-lch-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-lch-01.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fef81186"></a> lch(from peru calc(l \* 0.8) c h) produces a color that is 20% darker than peru or lch(62.2532% 54.0114 63.6769), with its chroma and hue left unchanged. The result is lch(49.80256 54.0114 63.6769)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fb53cf30"></a> This adds 180 degrees to the hue angle, giving the complementary color.
>
> ```text
> --accent:  lightseagreen;
> --complement:   lch(from var(--accent) l c calc(h + 180));
> ```
>
> lightseagreen is lch(65.4937 39.4484 190.1013), so --complement is lch(65.4937 39.4484 370.1013)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-dddffb4a"></a> Fully desaturating a color to gray, keeping the exact same lightness:
>
> ```text
> --mycolor:  orchid;
> // orchid is lch(62.753 62.571 326.973)
> --mygray:  lch(from var(--mycolor) l 0 h)
> // mygray is lch(62.753 0 326.973) which is rgb(59.515% 59.515% 59.515%)
> ```
>
> But now (since the hue was preserved) <em>re-saturating</em> again
>
> ```text
> --mymuted:  lch(from var(--mygray) l 30 h);
> // mymuted is lch(62.753 30 326.973) which is rgb(72.710% 53.293% 71.224%)
> ```
However, unlike HSL, manipulations are not guaranteed to be in-gamut.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-wildly-oog"></a> In this example, the aim is to produce a new color with the same Lightness and Chroma, but the triad (hue differs by 120 degrees). The origin color is inside the RGB gamut, but rotating the hue in LCH produces an out of gamut color.
>
> ```text
> --mycolor:  lch(60% 90 320);
> lch(from var(--mycolor) l c calc(h - 120));
> ```
>
> This gives a very high-chroma blue-green, lch(60% 90 200) which is color(srgb -0.6 0.698 0.772) and thus out of gamut (negative red value) for sRGB. Indeed, it is out of gamut for display-p3: color(display-p3 -0.46 0.68 0.758) and even rec2020: color(rec2020 -0.14 0.623 0.729).
>
> The closest color inside the sRGB gamut would be lch(60.71% 37.56 201.1) which is rgb(0% 64.2% 66.3%). The difference in chroma (37.5, instead of 90) is huge.
>
> <a id="fig-LCH-rotation-oog"></a>
>
> ![Embedded object resource](https://www.w3.org/TR/2026/WD-css-color-5-20260908/images/LCH-rotation-oog.svg)
>
> Diagram of CIE CH plane showing relative color manipulation. The <em>a</em> and <em>b</em> axes are labelled, and cross in the middle. We are looking down the central Lightness axis. The maximal gamut of the sRGB color space is shown as an irregular, convex polygon.
>
> This diagram shows the sRGB gamut, in the CIE ab plane. Small circles indicate the primary and secondary color. The origin color, shown as a large circle, is in gamut for sRGB; but becomes out of gamut (shown as a grey fill and red border) when the LCH hue is rotated -120°. The gamut-mapped result has much lower chroma.
>
> Performing the same operation in HSL will return an in-gamut result. But it is unsatisfactory in other ways:
>
> ```text
> --mycolor:  lch(60% 90 320);
> hsl(from var(--mycolor) calc(h - 120) s l);
> ```
>
> In HSL, --mycolor is hsl(289.18 93.136% 65.531%) so subtracting 120 degrees gives hsl(169.18 93.136% 65.531%). Converting that result back to LCH lch(89.0345% 49.3503 178.714) we see that, due to the hue rotate in HSL, Lightness shot up from 60% to 89%, the Chroma has dropped from 90 to 49, and the Hue actually changed by 141 degrees, not 120.

### <a id="relative-OkLCh"></a>4.9. Relative OkLCh Colors

The meaning of OkLCh colors is defined in [CSS Color 4 § 9.2 Oklab and OkLCh](https://www.w3.org/TR/css-color-4/#ok-lab).

<a id="ref-for-funcdef-oklch③"></a>

The grammar of the [oklch()](#funcdef-oklch) function is extended as follows:

<a id="funcdef-oklch"></a>

<a id="ref-for-typedef-color①④"></a>

<a id="ref-for-mult-opt①⑧"></a>

<a id="ref-for-percentage-value②⓪"></a>

<a id="ref-for-comb-one⑥③"></a>

<a id="ref-for-number-value④①"></a>

<a id="ref-for-comb-one⑥④"></a>

<a id="ref-for-percentage-value②①"></a>

<a id="ref-for-comb-one⑥⑤"></a>

<a id="ref-for-number-value④②"></a>

<a id="ref-for-comb-one⑥⑥"></a>

<a id="ref-for-typedef-hue⑤"></a>

<a id="ref-for-comb-one⑥⑦"></a>

<a id="ref-for-typedef-color-alpha-value⑧"></a>

<a id="ref-for-comb-one⑥⑧"></a>

<a id="ref-for-mult-opt①⑨"></a>

```text
oklch() = oklch([from <color>]?
          [<percentage> | <number> | none]
          [<percentage> | <number> | none]
          [<hue> | none]
          [ / [<alpha-value> | none] ]? )
```
<a id="ref-for-relative-color①⓪"></a>

<a id="ref-for-funcdef-oklch④"></a>

<a id="ref-for-component-keyword①⓪"></a>

Within a [relative color](#relative-color) syntax [oklch()](#funcdef-oklch) function, the allowed [component keywords](#component-keyword) are:

- <a id="ref-for-number-value④③"></a>

  <a id="ref-for-origin-color③①"></a>

  <a id="ref-for-conversion-if-required①②"></a>

  <a id="valdef-oklch-l"></a>l is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) Oklab Lightness after [conversion, if required](#conversion-if-required), to OkLCh. 1.0 is equivalent to 100%.

- <a id="ref-for-number-value④④"></a>

  <a id="ref-for-origin-color③②"></a>

  <a id="ref-for-conversion-if-required①③"></a>

  <a id="valdef-oklch-c"></a>c is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) OkLCh chroma after [conversion, if required](#conversion-if-required), to OkLCh. 0.4 is equivalent to 100%.

- <a id="ref-for-number-value④⑤"></a>

  <a id="ref-for-origin-color③③"></a>

  <a id="ref-for-conversion-if-required①④"></a>

  <a id="valdef-oklch-h"></a>h is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) OkLCh hue, in degrees, after [conversion, if required](#conversion-if-required), to OkLCh, normalized to a \[0, 360\] range. 90 is equivalent to 90deg.

- <a id="ref-for-number-value④⑥"></a>

  <a id="ref-for-origin-color③④"></a>

  <a id="valdef-oklch-alpha"></a>alpha is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) alpha transparency. 1.0 is equivalent to 100%.

Tests

- [relative-currentcolor-oklch-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-oklch-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-oklch-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-oklch-01.html)

Because OkLCh is both perceptually uniform and chroma-preserving, and because the axes correspond to easily understood attributes of a color, OkLCh is a good choice for color manipulation.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-oklch-wildly-oog"></a> In this example, the aim is again to produce a new color with the same Lightness and Chroma, but the triad (hue differs by 120 degrees). In this example, we will do the manipulation in OkLCh. The origin color is inside the RGB gamut, but rotating the hue in OkLCh again produces an out of gamut color.
>
> ```text
> --mycolor:  lch(60% 90 320);
> oklch(from var(--mycolor) l c calc(h - 120));
> ```
>
> --mycolor is oklch(0.69012 0.25077 319.893). Subtracting 120 from the Hue gives a very high-chroma blue-green, oklch(0.69012 0.25077 199.893) which is out of sRGB gamut, color(srgb -0.6018 0.7621 0.8448) as the negative red component indicates. Bring this into gamut by reducing OkLCh Chroma, yields oklch(0.69012 0.1173 199.893). The OkLCh chroma has dropped from 0.251 to 0.117.

### <a id="relative-alpha"></a>4.10. Relative Alpha Colors

Relative alpha colors refer to an origin color, and only change the alpha channel. The meaning of alpha channels is defined in [CSS Color 4 § 4.2 Representing Transparency in Colors: the \<alpha-value\> syntax](https://www.w3.org/TR/css-color-4/#alpha-syntax).

<a id="ref-for-funcdef-alpha①"></a>

The grammar of the [alpha()](#funcdef-alpha) function, new in this level, is as follows:

<a id="funcdef-alpha"></a>

<a id="ref-for-typedef-color①⑤"></a>

<a id="ref-for-typedef-color-alpha-value⑨"></a>

<a id="ref-for-comb-one⑥⑨"></a>

<a id="ref-for-mult-opt②⓪"></a>

```text
alpha() = alpha([from <color>]
          [ / [<alpha-value> | none] ]? )
```
<a id="ref-for-relative-color①①"></a>

<a id="ref-for-funcdef-alpha②"></a>

<a id="ref-for-component-keyword①①"></a>

Within a [relative color](#relative-color) syntax [alpha()](#funcdef-alpha) function, the allowed [component keywords](#component-keyword) are:

- <a id="ref-for-number-value④⑦"></a>

  <a id="ref-for-origin-color③⑤"></a>

  <a id="valdef-alpha-alpha"></a>alpha is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) alpha transparency. 1.0 is equivalent to 100%.

<a id="ref-for-origin-color③⑥"></a>

The color components of the [origin color](#origin-color) are unchanged, the alpha component is modified or replaced. The result of this function is in the color space of the origin color.

Tests

- [alpha-color-computed.html](https://wpt.fyi/results/css/css-color/parsing/alpha-color-computed.html) [(live test)](http://wpt.live/css/css-color/parsing/alpha-color-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/alpha-color-computed.html)
- [alpha-color-parsing-invalid.html](https://wpt.fyi/results/css/css-color/parsing/alpha-color-parsing-invalid.html) [(live test)](http://wpt.live/css/css-color/parsing/alpha-color-parsing-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/alpha-color-parsing-invalid.html)
- [alpha-color-parsing-valid.html](https://wpt.fyi/results/css/css-color/parsing/alpha-color-parsing-valid.html) [(live test)](http://wpt.live/css/css-color/parsing/alpha-color-parsing-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/alpha-color-parsing-valid.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-alpha-replace"></a> For example, here the result is the same as the origin color, but the alpha is changed to 80%
>
> ```text
> --mycolor:  oklch(60% 0.25 315 / 0.3);
>  alpha(from var(--mycolor) / 80%);
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-alpha-change"></a> For example, here the result is the same as the origin color, but the alpha is modified to be half of that in the origin color
>
> ```text
> --mycolor:  oklch(60% 0.25 315 / 0.8);
>  alpha(from var(--mycolor) / calc(alpha * 0.5));
> ```
<a id="ref-for-funcdef-color②"></a>

## <a id="color-function"></a>5.  Specifying Predefined and Custom Color Spaces: the [color()](#funcdef-color) Function 

<a id="ref-for-funcdef-color③"></a>

<a id="ref-for-color-space①"></a>

The [color()](#funcdef-color) function allows a color to be specified in a particular, given [color space](https://www.w3.org/TR/css-color-4/#color-space) (rather than the implicit sRGB color space that most of the other color functions operate in).

<a id="ref-for-funcdef-color④"></a>

In this level the [color()](#funcdef-color) function is extended to allow custom color spaces, in addition to the predefined spaces from [CSS Color 4 §  10. Predefined Color Spaces](https://www.w3.org/TR/css-color-4/#predefined).

It is also extended to allow relative, rather than just absolute, colors.

Its syntax is now as follows:

<a id="funcdef-color"></a>

<a id="ref-for-typedef-color①⑥"></a>

<a id="ref-for-mult-opt②①"></a>

<a id="ref-for-typedef-colorspace-params"></a>

<a id="ref-for-typedef-color-alpha-value①⓪"></a>

<a id="ref-for-comb-one⑦⓪"></a>

<a id="ref-for-mult-opt②②"></a>

<a id="typedef-colorspace-params"></a>

<a id="ref-for-typedef-custom-params"></a>

<a id="ref-for-comb-one⑦①"></a>

<a id="ref-for-typedef-predefined-rgb-params"></a>

<a id="ref-for-comb-one⑦②"></a>

<a id="ref-for-typedef-xyz-params"></a>

<a id="typedef-custom-params"></a>

<a id="ref-for-typedef-dashed-ident"></a>

<a id="ref-for-number-value④⑧"></a>

<a id="ref-for-comb-one⑦③"></a>

<a id="ref-for-percentage-value②②"></a>

<a id="ref-for-comb-one⑦④"></a>

<a id="ref-for-mult-one-plus"></a>

<a id="typedef-predefined-rgb-params"></a>

<a id="ref-for-typedef-predefined-rgb"></a>

<a id="ref-for-number-value④⑨"></a>

<a id="ref-for-comb-one⑦⑤"></a>

<a id="ref-for-percentage-value②③"></a>

<a id="ref-for-comb-one⑦⑥"></a>

<a id="ref-for-mult-num②"></a>

<a id="typedef-predefined-rgb"></a>

<a id="ref-for-comb-one⑦⑦"></a>

<a id="ref-for-comb-one⑦⑧"></a>

<a id="ref-for-comb-one⑦⑨"></a>

<a id="ref-for-comb-one⑧⓪"></a>

<a id="ref-for-comb-one⑧①"></a>

<a id="ref-for-comb-one⑧②"></a>

<a id="typedef-xyz-params"></a>

<a id="ref-for-typedef-xyz-space"></a>

<a id="ref-for-number-value⑤⓪"></a>

<a id="ref-for-comb-one⑧③"></a>

<a id="ref-for-percentage-value②④"></a>

<a id="ref-for-comb-one⑧④"></a>

<a id="ref-for-mult-num③"></a>

```text
color() = color( [from <color>]? <colorspace-params> [ / [ <alpha-value> | none ] ]? )
<colorspace-params> = [<custom-params> | <predefined-rgb-params> | <xyz-params>]
<custom-params> = <dashed-ident> [ <number> | <percentage> | none ]+
<predefined-rgb-params> = <predefined-rgb> [ <number> | <percentage> | none ]{3}
<predefined-rgb> = srgb | srgb-linear | display-p3 | display-p3-linear | a98-rgb | prophoto-rgb | rec2020
<xyz-params> = <xyz-space> [ <number> | <percentage> | none ]{3}
```
The color function takes parameters specifying a color, in an explicitly listed color space.

<a id="ref-for-valid-color"></a>

It represents either an <a id="invalid-color"></a>invalid color, as described below, or a [valid color](#valid-color).

<a id="ref-for-invalid-color"></a>

Any color which is not an [invalid color](#invalid-color) is a <a id="valid-color"></a>valid color.

<a id="ref-for-valid-color①"></a>

A color may be a [valid color](#valid-color) but still be outside the range of colors that can be produced by an output device (a screen, projector, or printer). It is said to be <a id="out-of-gamut"></a>out of gamut for that color space.

<a id="ref-for-gamut-map"></a>

An out of gamut color has component values less than 0 or 0%, or greater than 1 or 100%. These are not invalid; instead, for display, they are [gamut-mapped](#gamut-map) using a relative colorimetric intent which brings the values within the range 0/0% to 1/100% at computed-value time.

<a id="ref-for-valid-color②"></a>

<a id="ref-for-out-of-gamut"></a>

Each [valid color](#valid-color) is either in-gamut for the output device (screen, or printer), or it is [out of gamut](#out-of-gamut).

### <a id="relative-color-function"></a>5.1. Relative Color-Function Colors

<a id="ref-for-relative-color①②"></a>

<a id="ref-for-funcdef-color⑤"></a>

<a id="ref-for-typedef-custom-params①"></a>

<a id="ref-for-component-keyword①②"></a>

Within a [relative color](#relative-color) syntax [color()](#funcdef-color) function using [\<custom-params\>](#typedef-custom-params), the number and name of the allowed [component keywords](#component-keyword) are:

- <a id="ref-for-descdef-color-profile-components"></a>

  <a id="ref-for-at-ruledef-profile"></a>

  <a id="ref-for-number-value⑤①"></a>

  <a id="ref-for-origin-color③⑦"></a>

  <a id="ref-for-conversion-if-required①⑤"></a>

  defined by the [components](#descdef-color-profile-components) descriptor on the corresponding [@color-profile](#at-ruledef-profile), if present; otherwise, no relative color manipulation is valid. They are [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s that correspond to the [origin color’s](#origin-color) components after [conversion, if required](#conversion-if-required) to the color space of the color profile. The value 1.0 corresponds to 100%.

<a id="ref-for-relative-color①③"></a>

<a id="ref-for-funcdef-color⑥"></a>

<a id="ref-for-typedef-predefined-rgb-params①"></a>

<a id="ref-for-component-keyword①③"></a>

Within a [relative color](#relative-color) syntax [color()](#funcdef-color) function using [\<predefined-rgb-params\>](#typedef-predefined-rgb-params), the allowed [component keywords](#component-keyword) are:

- <a id="ref-for-number-value⑤②"></a>

  <a id="ref-for-origin-color③⑧"></a>

  <a id="ref-for-conversion-if-required①⑥"></a>

  <a id="valdef-color-r"></a>r, <a id="valdef-color-g"></a>g, and <a id="valdef-color-b"></a>b are all [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s that correspond to the [origin color’s](#origin-color) red, green, and blue components after [conversion, if required](#conversion-if-required) to the predefined RGB color space. The value 1.0 corresponds to 100%.

<a id="ref-for-relative-color①④"></a>

<a id="ref-for-funcdef-color⑦"></a>

<a id="ref-for-typedef-xyz-params①"></a>

<a id="ref-for-component-keyword①④"></a>

Within a [relative color](#relative-color) syntax [color()](#funcdef-color) function using [\<xyz-params\>](#typedef-xyz-params), the allowed [component keywords](#component-keyword) are:

- <a id="ref-for-number-value⑤③"></a>

  <a id="ref-for-origin-color③⑨"></a>

  <a id="ref-for-conversion-if-required①⑦"></a>

  <a id="valdef-color-x"></a>x, <a id="valdef-color-y"></a>y, <a id="valdef-color-z"></a>z are all [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s that correspond to the [origin color’s](#origin-color) X, Y and Z components after [conversion, if required](#conversion-if-required) to relative CIE XYZ color space adapted to the relevant white point. The value 1.0 corresponds to 100%.

<a id="ref-for-relative-color①⑤"></a>

<a id="ref-for-funcdef-color⑧"></a>

<a id="ref-for-typedef-predefined-rgb-params②"></a>

<a id="ref-for-typedef-xyz-params②"></a>

<a id="ref-for-component-keyword①⑤"></a>

Within a [relative color](#relative-color) syntax [color()](#funcdef-color) function using either [\<predefined-rgb-params\>](#typedef-predefined-rgb-params) or [\<xyz-params\>](#typedef-xyz-params), an additional allowed [component keyword](#component-keyword) is:

- <a id="ref-for-number-value⑤④"></a>

  <a id="ref-for-origin-color④⓪"></a>

  <a id="valdef-color-alpha"></a>alpha is a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) that corresponds to the [origin color’s](#origin-color) alpha transparency. 1.0 is equivalent to 100%.

The parameters have the following form:

- <a id="ref-for-typedef-ident"></a>

  <a id="ref-for-typedef-dashed-ident①"></a>

  <a id="ref-for-valdef-color-display-p3"></a>

  <a id="ref-for-at-ruledef-profile①"></a>

  <a id="ref-for-number-value⑤⑤"></a>

  <a id="ref-for-percentage-value②⑤"></a>

  An [\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident) or [\<dashed-ident\>](https://www.w3.org/TR/css-values-4/#typedef-dashed-ident) denoting the color space. If this is an <a id="ref-for-typedef-ident①"></a>\<ident\> it denotes one of the predefined color spaces [CSS Color 4 §  10. Predefined Color Spaces](https://www.w3.org/TR/css-color-4/#predefined) (such as [display-p3](https://www.w3.org/TR/css-color-4/#valdef-color-display-p3)); if it is a <a id="ref-for-typedef-dashed-ident②"></a>\<dashed-ident\> it denotes a custom color space, defined by a [@color-profile](#at-ruledef-profile) rule. Individual predefined color spaces may further restrict whether [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s or both, may be used.

  <a id="ref-for-typedef-ident②"></a>

  <a id="ref-for-invalid-color①"></a>

  If the [\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident) names a non-existent color space (a name that does not match one of the predefined color spaces), or a predefined but unsupported color space, this argument represents an [invalid color](#invalid-color).

  <a id="ref-for-typedef-dashed-ident③"></a>

  <a id="ref-for-css-color-profile"></a>

  <a id="ref-for-invalid-color②"></a>

  If the [\<dashed-ident\>](https://www.w3.org/TR/css-values-4/#typedef-dashed-ident) names a non-existent color space ( a name that does not match an [color profile’s](#css-color-profile) name, or which matches but the corresponding profile has not loaded, or does not represent a valid profile), this argument represents an [invalid color](#invalid-color).

- <a id="ref-for-number-value⑤⑥"></a>

  <a id="ref-for-percentage-value②⑥"></a>

  One or more [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s providing the parameter values that the color space takes.

  For custom color spaces, specified component values less than 0 or 0%, or greater than 1 or 100% are not invalid; they are clamped to the valid range at computed value time. This is because ICC profiles typically do not accept out of range input values.

  <a id="ref-for-number-value⑤⑦"></a>

  <a id="ref-for-percentage-value②⑦"></a>

  <a id="ref-for-valid-color③"></a>

  For custom color spaces, if more [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s are provided than parameters that the color space takes, the excess <a id="ref-for-number-value⑤⑧"></a>\<number\>s at the end are ignored. The color is still a [valid color](#valid-color).

  <a id="ref-for-number-value⑤⑨"></a>

  <a id="ref-for-percentage-value②⑧"></a>

  <a id="ref-for-valid-color④"></a>

  For custom color spaces, if more [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s are provided than components listed in the optional components descriptor, the additional values at the end are still valid, but cannot be used in Relative Color Syntax. The color is still a [valid color](#valid-color).

  <a id="ref-for-number-value⑥⓪"></a>

  <a id="ref-for-percentage-value②⑨"></a>

  <a id="ref-for-valid-color⑤"></a>

  For custom color spaces, if fewer [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)s or [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s are provided than parameters that the color space takes, the missing parameters default to 0. (This is particularly convenient for multicomponent printers where the additional inks are spot colors or varnishes that most colors on the page won’t use.) The color is still a [valid color](#valid-color).

  For predefined color spaces, specified component values less than 0 or 0%, or greater than 1 or 100% are not invalid; these out of gamut colors are gamut mapped to the valid range at computed value time, with a relative colorimetric intent.

- <a id="ref-for-typedef-color-alpha-value①①"></a>

  An optional slash-separated [\<alpha-value\>](https://www.w3.org/TR/css-color-4/#typedef-color-alpha-value).If omitted, it defaults to 100%.

Tests

- [color-computed-relative-color.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-relative-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-relative-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-relative-color.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-RCS-XYZ"></a> For example, Relative Color Syntax in the CIE XYZ D65 colorspace is used to generate a color which has the same chromaticity but exactly half the luminance of the base color:
>
> ```css
> --base:  color(display-p3 0.7 0.5 0.1);
> --dark:  color(from var(--base) xyz-d65 calc(x/2) calc(y/2) calc(z/2));
> ```
>
> The origin color is color(xyz-d65 0.281 0.253 0.044) and so the relative color is color(xyz-d65 0.14 0.126 0.022).

### <a id="custom-color"></a>5.2.  Custom Color Spaces 

<a id="ref-for-color"></a>

CSS allows [colors](https://www.w3.org/TR/css-color-4/#color) to be specified by reference to a color profile. This could be for example a calibrated CMYK printer, or an RGB color space, or any other color or monochrome output device which has been characterized.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-profiles-colors"></a> This example specifies four calibrated colors: two are custom spaces (for a SWOP-coated CMYK press, and for a wide-gamut seven-ink printer), the other two are predefined spaces (the ProPhoto RGB, and display-p3 RGB spaces). In each case, the numerical parameters are in the range 0.0 to 1.0 (rather than, for example, 0 to 255).
>
> ```css
> color: color(--swopc 0.0134 0.8078 0.7451 0.3019);
> color: color(--indigo 0.0941 0.6274 0.3372 0.1647 0 0.0706 0.1216);
> color: color(prophoto-rgb 0.9137 0.5882 0.4784);
> color: color(display-p3 0.3804 0.9921 0.1412);
> ```
<a id="ref-for-typedef-dashed-ident④"></a>

<a id="ref-for-at-ruledef-profile②"></a>

The colors not using a predefined color space [CSS Color 4 §  10. Predefined Color Spaces](https://www.w3.org/TR/css-color-4/#predefined) are distinguished by their use of [\<dashed-ident\>](https://www.w3.org/TR/css-values-4/#typedef-dashed-ident) and also need a matching [@color-profile](#at-ruledef-profile) at-rule somewhere in the stylesheet, to connect the name with the profile data.

Tests

- [at-color-profile-001.html](https://wpt.fyi/results/css/css-color/at-color-profile-001.html) [(live test)](http://wpt.live/css/css-color/at-color-profile-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/at-color-profile-001.html)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-profiles-colors-at"></a>
>
> ```css
> @color-profile --swopc {
>   src: url('http://example.org/swop-coated.icc');}
> @color-profile --indigo {
>   src: url('http://example.org/indigo-seven.icc');}
> ```
<a id="ref-for-at-ruledef-profile③"></a>

### <a id="at-profile"></a>5.3.  Specifying a Color Profile: the [@color-profile](#at-ruledef-profile) at-rule 

<a id="ref-for-funcdef-color⑨"></a>

The <a id="at-ruledef-profile"></a>@color-profile rule defines and names a <a id="css-color-profile"></a>color profile which can later be used in the [color()](#funcdef-color) function to specify a color.

It’s defined as:

<a id="ref-for-typedef-dashed-ident⑤"></a>

<a id="ref-for-comb-one⑧⑤"></a>

<a id="ref-for-typedef-declaration-list"></a>

```text
@color-profile = @color-profile [<dashed-ident> | device-cmyk] { <declaration-list> }
```
Tests

- [at-color-profile-001.html](https://wpt.fyi/results/css/css-color/at-color-profile-001.html) [(live test)](http://wpt.live/css/css-color/at-color-profile-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/at-color-profile-001.html)

<a id="ref-for-typedef-dashed-ident⑥"></a>

<a id="ref-for-css-color-profile①"></a>

The [\<dashed-ident\>](https://www.w3.org/TR/css-values-4/#typedef-dashed-ident) gives the [color profile’s](#css-color-profile) name, by which it will be used in a CSS stylesheet. Alternatively, the device-cmyk keyword means that this color profile will, if valid, be used to resolve colors specified in device-cmyk.

<a id="ref-for-at-ruledef-profile④"></a>

The [@color-profile](#at-ruledef-profile) rule accepts the descriptors defined in this specification.

| Field               | Definition                                                                  |
|---------------------|-----------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-color-profile-src"></a>src                                                      |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-profile⑤"></a>[@color-profile](#at-ruledef-profile)                    |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-url-value"></a>[\<url\>](https://www.w3.org/TR/css-values-4/#url-value) |
| <strong>Initial:&#xA;      </strong> | n/a                                                                         |

<a id="ref-for-descdef-color-profile-src"></a>

The [src](#descdef-color-profile-src) descriptor specifies the URL to retrieve the color-profile information from.

<a id="ref-for-at-ruledef-profile⑥"></a>

If multiple [@color-profile](#at-ruledef-profile) rules are defined with the same name, the last one in document order wins, and all preceding ones are ignored.

The retrieved ICC profile is valid if

- it can be parsed as an ICC Profile

- it is an Input, Display, Output, or color space ICC profile. (Abstract, DeviceLink, and NamedColor ICC Profiles must not be used).

<a id="ref-for-invalid-color③"></a>

If the profile is not valid, all CSS colors which reference this profile are [invalid color](#invalid-color)s.

<a id="ref-for-at-ruledef-profile⑦"></a>

<a id="ref-for-fetch-a-style-resource"></a>

<a id="ref-for-concept-response"></a>

To <a id="fetch-an-external-color-profile"></a>fetch an external color profile, given a [@color-profile](#at-ruledef-profile) rule <var>rule</var>, [fetch a style resource](https://www.w3.org/TR/css-values-4/#fetch-a-style-resource) given <var>rule</var>’s URL, with ruleOrDeclaration being <var>rule</var>, destination "color-profile", CORS mode "cors", and processResponse being the following steps given [response](https://fetch.spec.whatwg.org/#concept-response) \|/res\| and null, failure or a byte stream <var>byteStream</var>: If <var>byteStream</var> is a byte stream, apply the color profile as parsed from \|byteStream.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The Internet Media Type ("MIME type") for ICC profiles is [application/vnd.iccprofile](https://www.iana.org/assignments/media-types/application/vnd.iccprofile).

| Field               | Definition                                                                                                                                                                          |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-color-profile-rendering-intent"></a>rendering-intent                                                                                                                                                 |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-profile⑧"></a>[@color-profile](#at-ruledef-profile)                                                                                                                            |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-comb-one⑧⑥"></a>relative-colorimetric [\|](https://www.w3.org/TR/css-values-4/#comb-one) absolute-colorimetric <a id="ref-for-comb-one⑧⑦"></a>\| perceptual <a id="ref-for-comb-one⑧⑧"></a>\| saturation |
| <strong>Initial:&#xA;      </strong> | relative-colorimetric                                                                                                                                                               |

<a id="ref-for-css-color-profile②"></a>

<a id="ref-for-descdef-color-profile-rendering-intent"></a>

[Color profiles](#css-color-profile) contain “rendering intents”, which define how to <a id="gamut-map"></a>gamut-map their color to smaller gamuts than they’re defined over. Often a profile will contain only a single intent, but when there are multiple, the [rendering-intent](#descdef-color-profile-rendering-intent) descriptor chooses one of them to use.

The four possible rendering intents are [\[ICC\]](#biblio-icc):

<a id="valdef-color-profile-rendering-intent-relative-colorimetric"></a>relative-colorimetric  
Media-relative colorimetric is required to leave source colors that fall inside the destination medium gamut unchanged relative to the respective media white points. Source colors that are out of the destination medium gamut are mapped to colors on the gamut boundary using a variety of different methods.

The media-relative colorimetric rendering intent is often used with black point compensation, where the source medium black point is mapped to the destination medium black point as well. This method must map the source white point to the destination white point. If black point compensation is in use, the source black point must also be mapped to the destination black point. Adaptation algorithms should be used to adjust for the change in white point. Relative relationships of colors inside both source and destination gamuts should be preserved. Relative relationships of colors outside the destination gamut may be changed.

<a id="valdef-color-profile-rendering-intent-absolute-colorimetric"></a>absolute-colorimetric  
ICC-absolute colorimetric is required to leave source colors that fall inside the destination medium gamut unchanged relative to the adopted white (a perfect reflecting diffuser). Source colors that are out of the destination medium gamut are mapped to colors on the gamut boundary using a variety of different methods. This method produces the most accurate color matching of in-gamut colors, but will result in highlight clipping if the destination medium white point is lower than the source medium white point. For this reason it is recommended for use only in applications that need exact color matching and where highlight clipping is not a concern.

This method MUST disable white point matching and black point matching when converting colors. In general, this option is not recommended except for testing purposes.

<a id="valdef-color-profile-rendering-intent-perceptual"></a>perceptual  
This method is often the preferred choice for images, especially when there are substantial differences between the source and destination (such as a screen display image reproduced on a reflection print). It takes the colors of the source image and re-optimizes the appearance for the destination medium using proprietary methods. This re-optimization may result in colors within both the source and destination gamuts being changed, although perceptual transforms are supposed to maintain the basic artistic intent of the original in the reproduction. They will not attempt to correct errors in the source image.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: With v2 ICC profiles there is no specified perceptual reference medium, which can cause interoperability problems. When v2 ICC profiles are used it can be safer to use the media-relative colorimetric rendering intent with black point compensation, instead of the perceptual rendering intent, unless the specific source and destination profiles to be used have been checked to ensure the combination produces the desired result.

This method should maintain relative color values among the pixels as they are mapped to the target device gamut. This method may change pixel values that were originally within the target device gamut, in order to avoid hue shifts and discontinuities and to preserve as much as possible the overall appearance of the scene.

<a id="valdef-color-profile-rendering-intent-saturation"></a>saturation  
This option was created to preserve the relative saturation (chroma) of the original, and to keep solid colors pure. However, it experienced interoperability problems like the perceptual intent, and as solid color preservation is not amenable to a reference medium solution using v4 profiles does not solve the problem. Use of this rendering intent is not recommended unless the specific source and destination profiles to be used have been checked to ensure the combination produces the desired result. This option should preserve the relative saturation (chroma) values of the original pixels. Out of gamut colors should be converted to colors that have the same saturation but fall just inside the gamut.

| Field               | Definition                                                                                                                                               |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-color-profile-components"></a>components                                                                                                                            |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-ruledef-profile⑨"></a>[@color-profile](#at-ruledef-profile)                                                                                                 |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-mult-comma①"></a><a id="ref-for-typedef-ident③"></a>[\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident)[\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong>Initial:&#xA;      </strong> | n/a                                                                                                                                                      |

Color profiles can define color spaces which contain a varying number of components. For example, a Cyan, Magenta, Yellow and Black (CMYK) profile has four components named c, m, y and k While a four-component additive screen profile might use four components named r, g, y and b.

<a id="ref-for-typedef-ident④"></a>

The value of this descriptor is a comma-separated list of [\<ident\>](https://www.w3.org/TR/css-values-4/#typedef-ident) tokens. Each <a id="ref-for-typedef-ident⑤"></a>\<ident\>\> names a component, in the order in which they are used in the color profile, while the total number of tokens defines the number of components.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-components-cmyk"></a> This descriptor declares that there are four components named cyan, magenta, yellow and black:
>
> ```text
> components: cyan, magenta, yellow, black
> ```
>
> while this descriptor opts for terser names:
>
> ```text
> components: c,m,y,k
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-components-cmykogv"></a> This descriptor declares that there are seven components named cyan, magenta, yellow, black, orange, green and violet:
>
> ```text
> components: cyan, magenta, yellow, black, orange, green, violet
> ```
<a id="ref-for-ascii-case-insensitive"></a>

<a id="ref-for-valdef-light-dark-none②"></a>

If a component is an [ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for [none](#valdef-light-dark-none), the descriptor is invalid, because that would clash with the token for missing values.

<a id="ref-for-funcdef-calc"></a>

If the name chosen for a component clashes with a CSS numeric constant as defined in [CSS Values 4 § 10.7.1 Numeric Constants: e, pi](https://www.w3.org/TR/css-values-4/#calc-constants) the component is still valid, but inside [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) the component will be shadowed by the numeric constant leading to unexpected results.

<a id="ref-for-valdef-calc-pi"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-component-pi"></a> This descriptor unwisely calls a component [pi](https://www.w3.org/TR/css-values-4/#valdef-calc-pi), leading to unexpected results in Relative Color Syntax.
>
> ```text
> @color-profile --unwise {
>   src: url(https://example.com/unwise);
>   components: mi, pi, ni;
> }
> --base: color(--unwise 35% 20% 8%);
> --accent: color(from var(--base) mi calc(pi * 2) calc(ni / 2));
> ```
>
> Here, the component values of --accent are 35%, 3.14159265358979 \* 2 = 6.28318530717959, 4%.

### <a id="cal-cmyk"></a>5.4.  CSS and Print: Using Calibrated CMYK and Other Printed Color Spaces 

<a id="ref-for-at-ruledef-profile①⓪"></a>

The [@color-profile](#at-ruledef-profile) at-rule is not restricted to RGB color spaces. While screens typically display colors directly in RGB, printers often represent colors with CMYK.

Calibrated four color print with Cyan, Magenta, Yellow and Black (CMYK), or high-fidelity wide gamut printing with additional inks such as Cyan Magenta Yellow Black Orange Green Violet (CMYKOGV) can also be done in CSS, provided you have an ICC profile corresponding to the combination of inks, paper, total ink coverage and equipment you will use.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-fogra"></a> For example, using offset printing to ISO 12647-2:2004 / Amd 1:2007 using the FOGRA39 characterization data on 115gsm coated paper with an ink limit of 300% Total Area Coverage [\[FOGRA39\]](#biblio-fogra39).
>
> ```css
> @color-profile --fogra39 {
>   src: url('https://example.org/Coated_Fogra39L_VIGC_300.icc');
> }
> .header {
>   background-color:   color(--fogra39 0% 70% 20% 0%);
>   }
> ```
>
> Here the color() function first states the name we have given the profile, then gives the percentage of cyan, magenta, yellow, and black.
>
> In this profile, this resolves to the color  lab(63.673303% 51.576902 5.811058) which is  rgb(93.124, 44.098% 57.491%).

Because the actual color resulting from a given CMYK combination is known, an on-screen visualization of the printed output (soft-proof) can be made.

Also, procedures that rely on knowing the color (anti-aliasing, compositing, using the color in a gradient, etc) can proceed as normal.

<a id="fig-fogra51-roundtrip"></a>

![A grid of colored squares. There are six columns, labelled A to F, and four rows, labelled 1 to 4.](https://www.w3.org/TR/2026/WD-css-color-5-20260908/images/macbeth-roundtrip.svg)

A grid of colored squares. There are six columns, labelled A to F, and four rows, labelled 1 to 4.

A color checker, used for ensuring color fidelity in the print and photographic industries. Averaged measured Lab values are available for each patch. The rectangles show the Lab values, converted to sRGB. The circles, which are barely visible, show the Lab values, passed through a FOGRA51 [\[FOGRA51\]](#biblio-fogra51) ICC profile to convert them to CMYK. The CMYK values are then passed through the same ICC profile in reverse, to yield new Lab values. These are then converted to sRGB for display.

The one patch with a more visible circle (third row, first patch) is because the color is slightly outside the gamut of the FOGRA51 CMYK space used.

The table below shows, for each patch, the DeltaE 2000 between the original Lab and the Lab value after round-tripping through CMYK. A DeltaE 2000 of 1 or more is just visible.

|     |      |      |      |      |      |      |
|-----|------|------|------|------|------|------|
|     | A    | B    | C    | D    | E    | F    |
| 1   | 0.06 | 0.07 | 0.03 | 0.04 | 0.06 | 0.17 |
| 2   | 0.03 | 0.75 | 0.05 | 0.06 | 0.03 | 0.02 |
| 3   | 1.9  | 0.04 | 0.06 | 0.05 | 0.02 | 0.05 |
| 4   | 0.03 | 0.08 | 0.03 | 0.03 | 0.04 | 0.80 |

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-swop5v2"></a> This example is using offset printing to ISO 12647-2:2004 using the CGATS/SWOP TR005 2007 characterization data on grade 5 paper with an ink limit of 300% Total Area Coverage, and medium gray component replacement (GCR).
>
> ```css
> @color-profile --swop5c {
>   src: url('https://example.org/SWOP2006_Coated5v2.icc');
> }
> .header {
>   background-color:   color(--swop5c 0% 70% 20% 0%);
> }
> ```
>
> In this profile, this amount of CMYK (the same percentages as the previous example) resolves to the color  lab(64.965217% 52.119710 5.406966) which is  rgb(94.903% 45.248% 59.104%).

Fallback colors can be specified, for example using media queries, to be used if the specified CMYK color is known to be outside the sRGB gamut.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-fogra39-fallback-mq"></a> This example uses the same FOGRA39 setup as before, but specifies a bright green which is outside the sRGB gamut. It is, however, inside the display-p3 gamut. Therefore it is displayed as-is on wide gamut screens and in print, and a less intense fallback color is used on sRGB screens.
>
> ```css
> @media (color-gamut: srgb) {
>   .header {
>     background-color:   rgb(8.154% 60.9704% 37.184%);
>     }
> }
> @media print, (color-gamut: p3){
>   .header {
>     background-color:   color(--fogra39 90% 0% 90% 0%);
>     }
> }
> ```
>
> This CMYK color corresponds to lab(56.596645% -58.995875 28.072154) or lch(56.596645% 65.33421077211648 154.5533771086801). In sRGB this would be rgb(-60.568% 62.558% 32.390%) which, as the large negative red component shows, is out of gamut.
>
> Reducing the chroma until the result is in gamut gives  lch(56.596645% 51 154.5533771086801) which is  rgb(8.154% 60.9704% 37.184%) and this has been manually specified as a fallback color.
>
> For wide gamut screens, the color is inside the display-p3 gamut (it is display-p3(0.1658 0.6147 0.3533) ).

Colors are not restricted to four inks (CMYK). For example, wide-gamut 7 Color ink sets can be used.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-fogra55beta-7color"></a> This example uses the beta FOGRA55 dataset [\[FOGRA55\]](#biblio-fogra55) for CMYKOGV seven-color printing. Four of the inks - black, cyan, magenta, and yellow - are the same as, and give the same results as, the FOGRA51 set. The other three inks are:
>
> - Orange: CIELAB 65 58 88
> - Green: CIELAB 60 -75 0
> - CIELAB 22 47 -56
>
> The measurement condition is M1, which means that optical brighteners in the paper are accounted for and the spectrophotometer has no UV-cut filter.
>
> ```css
> @color-profile --fogra55beta {
>   src: url('https://example.org/2020_13.003_FOGRA55beta_CL_Profile.icc');
> }
> .dark_skin {
>   background-color: 
>   color(--fogra55beta 0.183596 0.464444 0.461729 0.612490 0.156903 0.000000 0.000000);
> }
> .light_skin {
>   background-color: 
>   color(--fogra55beta 0.070804 0.334971 0.321802 0.215606 0.103107 0.000000 0.000000);
> }
> .blue_sky {
>   background-color: 
>   color(--fogra55beta 0.572088 0.229346 0.081708 0.282044 0.000000 0.000000 0.168260);
> }
> .foliage {
>   background-color: 
>   color(--fogra55beta 0.314566 0.145687 0.661941 0.582879 0.000000 0.234362 0.000000);
> }
> .blue_flower {
>   background-color: 
>   color(--fogra55beta 0.375515 0.259934 0.034849 0.107161 0.000000 0.000000 0.308200);
> }
> .bluish_green {
>   background-color: 
>   color(--fogra55beta 0.397575 0.010047 0.223682 0.031140 0.000000 0.317066 0.000000);
> }
> ```
### <a id="cmyk-to-lab"></a>5.5.  Converting CMYK colors to Lab 

Conversion from a calibrated CMYK color space to Lab is typically done by looking up the Lab values in an ICC profile.

### <a id="lab-to-cmyk"></a>5.6.  Converting Lab colors to CMYK 

For print, Lab colors will need to be converted to the color space of the printer.

This is typically done by looking up the CMYK values in an ICC profile.

<a id="ref-for-funcdef-device-cmyk④"></a>

## <a id="device-cmyk"></a>6.  Uncalibrated CMYK Colors: the [device-cmyk()](#funcdef-device-cmyk) Function 

Sometimes, when a given printer has not been calibrated, but the output for particular ink combinations is known through experimentation, or via a printed sample swatchbook, it is useful to express CMYK colors in a device-dependent way.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Because the actual resulting color can be unknown, CSS processors might attempt to approximate it. This approximation is likely to be visually very far from the actual printed result.

<a id="ref-for-funcdef-device-cmyk⑤"></a>

The [device-cmyk()](#funcdef-device-cmyk) function allows authors to specify a color in this way:

<a id="funcdef-device-cmyk"></a>

<a id="ref-for-typedef-legacy-device-cmyk-syntax"></a>

<a id="ref-for-comb-one⑧⑨"></a>

<a id="ref-for-typedef-modern-device-cmyk-syntax"></a>

<a id="typedef-legacy-device-cmyk-syntax"></a>

<a id="ref-for-number-value⑥①"></a>

<a id="ref-for-mult-comma②"></a>

<a id="typedef-modern-device-cmyk-syntax"></a>

<a id="ref-for-typedef-cmyk-component"></a>

<a id="ref-for-mult-num④"></a>

<a id="ref-for-typedef-color-alpha-value①②"></a>

<a id="ref-for-comb-one⑨⓪"></a>

<a id="ref-for-valdef-light-dark-none③"></a>

<a id="ref-for-mult-opt②③"></a>

<a id="typedef-cmyk-component"></a>

<a id="ref-for-number-value⑥②"></a>

<a id="ref-for-comb-one⑨①"></a>

<a id="ref-for-percentage-value③⓪"></a>

<a id="ref-for-comb-one⑨②"></a>

<a id="ref-for-valdef-light-dark-none④"></a>

```text
device-cmyk() = <legacy-device-cmyk-syntax> | <modern-device-cmyk-syntax>
<legacy-device-cmyk-syntax> = device-cmyk( <number>#{4} )
<modern-device-cmyk-syntax> = device-cmyk( <cmyk-component>{4} [ / [ <alpha-value> | none ] ]? )
<cmyk-component> = <number> | <percentage> | none
```
<a id="ref-for-funcdef-device-cmyk⑥"></a>

The arguments of the [device-cmyk()](#funcdef-device-cmyk) function specify the cyan, magenta, yellow, and black components, in order, as a number between 0 and 1 or, in the modern syntax, as a percentage between 0% and 100%. These two usages are equivalent, and map to each other linearly. Values less than 0 or 0%, or greater than 1 or 100%, are not invalid; instead, they are clamped to 0/0% or 1/100% at computed-value time.

<a id="ref-for-funcdef-rgb⑤"></a>

In the modern syntax, the fifth argument specifies the alpha component of the color. It’s interpreted identically to the fourth argument of the [rgb()](https://www.w3.org/TR/css-color-4/#funcdef-rgb) function. If omitted, it defaults to 100%.

<a id="ref-for-funcdef-device-cmyk⑦"></a>

<a id="ref-for-legacy-color-syntax②"></a>

For [historical reasons](https://www.w3.org/TR/2011/WD-css3-gcpm-20111129/#cmyk-colors), [device-cmyk()](#funcdef-device-cmyk) also support a [legacy color syntax](https://www.w3.org/TR/css-color-4/#legacy-color-syntax).

Typically, print-based applications will actually store the used colors as CMYK, and send them to the printer in that form. However, such colors do not have a colorimetric interpretation, and thus cannot be used in gradients, compositing, blending and so on.

As such, Device CMYK colors must be converted to an equivalent color. This is not trivial, like the conversion from HSL or HWB to RGB; the precise conversion depends on the precise characteristics of the output device.

1.  <a id="ref-for-funcdef-device-cmyk⑧"></a>

    <a id="ref-for-at-ruledef-profile①①"></a>

    If the user, author, or user-agent stylesheet has an [@color-profile](#at-ruledef-profile) definition for device-cmyk, and the resource specified by the src descriptor can be retrieved, and the resource is a valid CMYK ICC profile, and the user agent can process ICC profiles, the computed value of the [device-cmyk()](#funcdef-device-cmyk) function must be the Lab value of the CMYK color.

2.  <a id="ref-for-funcdef-device-cmyk⑨"></a>

    Otherwise, the computed value of the [device-cmyk()](#funcdef-device-cmyk) function must be the sRGB value of the CMYK color, as converted with the following naive conversion algorithm.

<a id="ref-for-at-ruledef-profile①②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-device-cmyk-naive"></a> For example, with no [@color-profile](#at-ruledef-profile), the following colors are equivalent, using the naive conversion.
>
> ```css
> color:  device-cmyk(0 81% 81% 30%);
> color:  rgb(178 34 34);
> color:  firebrick;
> ```
<a id="ref-for-at-ruledef-profile①③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-device-cmyk-colprof"></a> With the [@color-profile](#at-ruledef-profile) specified as in the example stylesheet, the following colors are equivalent, using colorimetric conversion.
>
> ```css
> color:  device-cmyk(0 81% 81% 30%);
> color:  lab(45.060% 45.477 35.459)
> color:  rgb(70.690% 26.851% 19.724%);
> ```
The naive conversion is necessarily approximate, since it has no knowledge of the colorimetry of the inks, the dot gain, the colorimetry of the RGB space, and so on.

<a id="fig-naive-cmyk"></a>

![A grid of colored squares. There are six columns, labelled A to F, and four rows, labelled 1 to 4.](https://www.w3.org/TR/2026/WD-css-color-5-20260908/images/macbeth-naive.svg)

A grid of colored squares. There are six columns, labelled A to F, and four rows, labelled 1 to 4.

A color checker, used for ensuring color fidelity in the print and photographic industries. Averaged measured Lab values are available for each patch. The rectangles show the Lab values, converted to sRGB. The circles show the Lab values, passed through an ICC profile to convert them to CMYK. The CMYK value are then naively converted to sRGB.

The table below shows, for each patch, the DeltaE 2000 between the original Lab and the Lab value after round-tripping through CMYK. A DeltaE 2000 of 1 or more is just visible, while 5 or more is just a different color altogether.

|     |       |       |       |       |       |       |
|-----|-------|-------|-------|-------|-------|-------|
|     | A     | B     | C     | D     | E     | F     |
| 1   | 11.33 | 9.36  | 5.66  | 7.52  | 12.39 | 21.58 |
| 2   | 6.40  | 8.79  | 11.77 | 17.16 | 11.91 | 3.97  |
| 3   | 12.1  | 17.00 | 3.38  | 1.94  | 18.08 | 14.97 |
| 4   | 1.89  | 6.56  | 7.85  | 8.76  | 9.82  | 10.29 |

### <a id="cmyk-rgb"></a>6.1.  Naively Converting Between Uncalibrated CMYK and sRGB-Based Color 

To <a id="naively-convert-from-cmyk-to-rgba"></a>naively convert from CMYK to RGBA:

- `red   = 1 - min(1, cyan * (1 - black) + black)`
- `green = 1 - min(1, magenta * (1 - black) + black)`
- `blue  = 1 - min(1, yellow  * (1 - black) + black)`
- Alpha is same as for input color.

To <a id="naively-convert-from-rgba-to-cmyk"></a>naively convert from RGBA to CMYK:

- `black   = 1 - max(red, green, blue)`
- `cyan = (1 - red - black)   / (1 - black), or 0 if black is 1`
- `magenta = (1 - green - black) / (1 - black), or 0 if black is 1`
- `yellow  = (1 - blue - black)  / (1 - black), or 0 if black is 1`
- alpha is the same as the input color

<a id="ref-for-funcdef-light-dark①"></a>

## <a id="light-dark"></a>7.  Reacting to the used color-scheme: the [light-dark()](#funcdef-light-dark) Function 

<a id="ref-for-css-system-colors"></a>

<a id="ref-for-color-scheme"></a>

<a id="ref-for-funcdef-light-dark②"></a>

[System colors](https://www.w3.org/TR/css-color-4/#css-system-colors) have the ability to react to an element’s [color scheme](https://www.w3.org/TR/css-color-adjust-1/#color-scheme). The [light-dark()](#funcdef-light-dark) function exposes the same capability to authors.

There are two forms of this function: one takes a pair of colors while the other takes a pair of images. Attempting to use one image and one color will result in a parse-time error.

<a id="funcdef-light-dark"></a>

<a id="ref-for-typedef-light-dark-color②"></a>

<a id="ref-for-comb-one⑨③"></a>

<a id="ref-for-typedef-light-dark-image"></a>

<a id="typedef-light-dark-color"></a>

<a id="ref-for-typedef-light-dark-color③"></a>

<a id="ref-for-typedef-color①⑦"></a>

<a id="ref-for-comb-comma①"></a>

<a id="ref-for-typedef-color①⑧"></a>

<a id="typedef-light-dark-image"></a>

<a id="ref-for-typedef-light-dark-image①"></a>

<a id="ref-for-typedef-image"></a>

<a id="ref-for-comb-one⑨④"></a>

<a id="ref-for-comb-comma②"></a>

<a id="ref-for-typedef-image①"></a>

<a id="ref-for-comb-one⑨⑤"></a>

```text
light-dark() = <light-dark-color> | <light-dark-image>
<light-dark-color> = light-dark(<color>, <color>)
<light-dark-image> = light-dark( [ <image> | none ] , [ <image> | none ] ) 
```
<a id="ref-for-element-color-scheme"></a>

<a id="ref-for-valdef-color-scheme-light"></a>

<a id="ref-for-valdef-color-scheme-dark"></a>

For the color form, this function computes to the computed value of the first color, if the [element color scheme](https://drafts.csswg.org/css-color-adjust-1/#element-color-scheme) is [light](https://www.w3.org/TR/css-color-adjust-1/#valdef-color-scheme-light), or to the computed value of the second color, if the <a id="ref-for-element-color-scheme①"></a>element color scheme is [dark](https://www.w3.org/TR/css-color-adjust-1/#valdef-color-scheme-dark).

<a id="ref-for-element-color-scheme②"></a>

<a id="ref-for-valdef-color-scheme-light①"></a>

<a id="ref-for-valdef-color-scheme-dark①"></a>

For the image form, this function returns the first image, if the [element color scheme](https://drafts.csswg.org/css-color-adjust-1/#element-color-scheme) is [light](https://www.w3.org/TR/css-color-adjust-1/#valdef-color-scheme-light), or the second image, if the <a id="ref-for-element-color-scheme③"></a>element color scheme is [dark](https://www.w3.org/TR/css-color-adjust-1/#valdef-color-scheme-dark).

The <a id="valdef-light-dark-none"></a>none keyword computes to image(transparent) (a fully transparent image with no natural size).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-light-dark-color"></a> For example, to maintain a legible contrast on links, for light mode and dark mode:
>
> ```css
>   a:link {
>       color: light-dark(blue, #81D9FE);
>       background-color: light-dark(white, black);
> }
> ```
>
> The traditional blue link text is legible on a white background (WCAG contrast 8.59:1, AAA pass) but would not be legible on a black background (WCAG contrast 2.44:1, AA fail). Instead, a lighter blue \#81D9FE is used in dark mode. (WCAG contrast 13.28:1, AAA pass).
>
> Legible link text
>
> Illegible link text
>
> Legible link text

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-light-dark-image"></a> For example, to provide easily visible list bullets for light mode and dark mode:
>
> ```css
> ul.fancy {
>   list-style-image: light-dark(
>     url("icons/deep-maroon-ball.png"),
>     url("icons/pale-yellow-star.png")
>   );
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-light-dark-none"></a> For example, a raster image is used in light mode, while in dark mode we want a fully-transparent image.
>
> ```css
> background-image: light-dark(url(my-light-image.png), none);
> ```
>
> This is equivalent to:
>
> ```css
> background-image: light-dark(url(my-light-image.png), image(transparent));
> ```
Tests

- [light-dark-basic.html](https://wpt.fyi/results/css/css-color/light-dark-basic.html) [(live test)](http://wpt.live/css/css-color/light-dark-basic.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/light-dark-basic.html)
- [light-dark-currentcolor.html](https://wpt.fyi/results/css/css-color/light-dark-currentcolor.html) [(live test)](http://wpt.live/css/css-color/light-dark-currentcolor.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/light-dark-currentcolor.html)
- [light-dark-image.html](https://wpt.fyi/results/css/css-color/light-dark-image.html) [(live test)](http://wpt.live/css/css-color/light-dark-image.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/light-dark-image.html)
- [light-dark-image-none-interpolation.html](https://wpt.fyi/results/css/css-color/light-dark-image-none-interpolation.html) [(live test)](http://wpt.live/css/css-color/light-dark-image-none-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/light-dark-image-none-interpolation.html)
- [light-dark-image-none.html](https://wpt.fyi/results/css/css-color/light-dark-image-none.html) [(live test)](http://wpt.live/css/css-color/light-dark-image-none.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/light-dark-image-none.html)
- [light-dark-inheritance.html](https://wpt.fyi/results/css/css-color/light-dark-inheritance.html) [(live test)](http://wpt.live/css/css-color/light-dark-inheritance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/light-dark-inheritance.html)
- [light-dark-currentcolor-in-color.html](https://wpt.fyi/results/css/css-color/light-dark-currentcolor-in-color.html) [(live test)](http://wpt.live/css/css-color/light-dark-currentcolor-in-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/light-dark-currentcolor-in-color.html)
- [highlight-styling-004.html](https://wpt.fyi/results/css/css-pseudo/highlight-styling-004.html) [(live test)](http://wpt.live/css/css-pseudo/highlight-styling-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-pseudo/highlight-styling-004.html)

<a id="ref-for-funcdef-contrast-color③"></a>

## <a id="contrast-color"></a>8.  Dynamically specifying a text color with adequate contrast: the [contrast-color()](#funcdef-contrast-color) Function 

<a id="ref-for-funcdef-contrast-color④"></a>

When colors are created dynamically, it can often be a challenge to specify a text color that provides adequate contrast with them when used as a background color. The [contrast-color()](#funcdef-contrast-color) function automatically provides a color with guaranteed color contrast when used as a text color on a solid background of the specified color.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Legibility is a complex topic, and sufficient color contrast is only one piece of the puzzle. Having a color pair with sufficient contrast does not guarantee that the text will be legible, as that also depends on a variety of factors, such as the font, the size of the text, the surrounding colors, etc.

<a id="funcdef-contrast-color"></a>

<a id="ref-for-typedef-color①⑨"></a>

```text
contrast-color() = contrast-color( <color> )
```
<a id="ref-for-funcdef-contrast-color⑤"></a>

<a id="ref-for-valdef-color-white"></a>

<a id="ref-for-valdef-color-black"></a>

[contrast-color()](#funcdef-contrast-color) resolves to either [white](https://www.w3.org/TR/css-color-4/#valdef-color-white) or [black](https://www.w3.org/TR/css-color-4/#valdef-color-black), whichever produces <strong>maximum</strong> color contrast for text when the input color is used as a solid background. If both <a id="ref-for-valdef-color-white①"></a>white and <a id="ref-for-valdef-color-black①"></a>black produce the same contrast, it resolves to <a id="ref-for-valdef-color-white②"></a>white.

The precise color contrast algorithm for determining whether to output a light or dark color is UA-defined at this level.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Future versions of this specification are expected to introduce more control over both the contrast algorithm(s) used, the use cases, as well as the returned color.

UAs are advised to not simply use the WCAG 2.1 [section 1.4.3 Contrast (Minimum)](https://www.w3.org/TR/WCAG21/#contrast-minimum) contrast ratio algorithm to decide between light and dark colors, as it has [several known issues](https://www.cedc.tools/article.html). However, colors returned by this function should still meet the WCAG 2.1 [section 1.4.3 Contrast (Minimum)](https://www.w3.org/TR/WCAG21/#contrast-minimum) for AA large text, as many authors need to meet legal requirements that mandate this.

Tests

- [contrast-color-001.html](https://wpt.fyi/results/css/css-color/contrast-color-001.html) [(live test)](http://wpt.live/css/css-color/contrast-color-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/contrast-color-001.html)
- [contrast-color-currentcolor-inherited.html](https://wpt.fyi/results/css/css-color/contrast-color-currentcolor-inherited.html) [(live test)](http://wpt.live/css/css-color/contrast-color-currentcolor-inherited.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/contrast-color-currentcolor-inherited.html)
- [contrast-color-style-query.html](https://wpt.fyi/results/css/css-color/contrast-color-style-query.html) [(live test)](http://wpt.live/css/css-color/contrast-color-style-query.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/contrast-color-style-query.html)
- [contrast-color-interpolation.html](https://wpt.fyi/results/css/css-color/animation/contrast-color-interpolation.html) [(live test)](http://wpt.live/css/css-color/animation/contrast-color-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/animation/contrast-color-interpolation.html)
- [color-computed-contrast-color-function.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-contrast-color-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-contrast-color-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-contrast-color-function.html)
- [color-invalid-contrast-color-function.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-contrast-color-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-contrast-color-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-contrast-color-function.html)
- [color-valid-contrast-color-function.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-contrast-color-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-contrast-color-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-contrast-color-function.html)
- [contrast-color-function-calc-container.html](https://wpt.fyi/results/css/css-color/parsing/contrast-color-function-calc-container.html) [(live test)](http://wpt.live/css/css-color/parsing/contrast-color-function-calc-container.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/contrast-color-function-calc-container.html)

## <a id="interpolation"></a>9.  Color Interpolation

### <a id="interpolation-space"></a>9.1.  Color Space for Interpolation

<a id="ref-for-color-interpolation-method①"></a>

The [\<color-interpolation-method\>](#color-interpolation-method), as defined in [CSS Color 4 § 13.2 Color Space for Interpolation](https://www.w3.org/TR/css-color-4/#interpolation-space), is extended to allow use of the [custom color spaces](#custom-color):

<a id="typedef-color-space"></a>

<a id="ref-for-typedef-rectangular-color-space"></a>

<a id="ref-for-comb-one⑨⑥"></a>

<a id="ref-for-typedef-polar-color-space"></a>

<a id="ref-for-comb-one⑨⑦"></a>

<a id="ref-for-typedef-custom-color-space"></a>

<a id="typedef-rectangular-color-space"></a>

<a id="ref-for-valdef-color-srgb①"></a>

<a id="ref-for-comb-one⑨⑧"></a>

<a id="ref-for-valdef-color-srgb-linear"></a>

<a id="ref-for-comb-one⑨⑨"></a>

<a id="ref-for-valdef-color-display-p3①"></a>

<a id="ref-for-comb-one①⓪⓪"></a>

<a id="ref-for-valdef-color-display-p3-linear"></a>

<a id="ref-for-comb-one①⓪①"></a>

<a id="ref-for-valdef-color-a98-rgb"></a>

<a id="ref-for-comb-one①⓪②"></a>

<a id="ref-for-valdef-color-prophoto-rgb"></a>

<a id="ref-for-comb-one①⓪③"></a>

<a id="ref-for-valdef-color-rec2020"></a>

<a id="ref-for-comb-one①⓪④"></a>

<a id="ref-for-valdef-lab-lab"></a>

<a id="ref-for-comb-one①⓪⑤"></a>

<a id="ref-for-valdef-oklab-oklab"></a>

<a id="ref-for-comb-one①⓪⑥"></a>

<a id="ref-for-typedef-xyz-space①"></a>

<a id="typedef-polar-color-space"></a>

<a id="ref-for-valdef-hsl-hsl②"></a>

<a id="ref-for-comb-one①⓪⑦"></a>

<a id="ref-for-valdef-hwb-hwb"></a>

<a id="ref-for-comb-one①⓪⑧"></a>

<a id="ref-for-valdef-lch-lch②"></a>

<a id="ref-for-comb-one①⓪⑨"></a>

<a id="ref-for-valdef-oklch-oklch"></a>

<a id="typedef-custom-color-space"></a>

<a id="ref-for-typedef-dashed-ident⑦"></a>

<a id="typedef-hue-interpolation-method"></a>

<a id="ref-for-comb-one①①⓪"></a>

<a id="ref-for-comb-one①①①"></a>

<a id="ref-for-comb-one①①②"></a>

<a id="color-interpolation-method"></a>

<a id="ref-for-typedef-rectangular-color-space①"></a>

<a id="ref-for-comb-one①①③"></a>

<a id="ref-for-typedef-polar-color-space①"></a>

<a id="ref-for-typedef-hue-interpolation-method②"></a>

<a id="ref-for-mult-opt②④"></a>

<a id="ref-for-comb-one①①④"></a>

<a id="ref-for-typedef-custom-color-space①"></a>

```text
<color-space> = <rectangular-color-space> | <polar-color-space> | <custom-color-space>
<rectangular-color-space> = srgb | srgb-linear | display-p3 | display-p3-linear | a98-rgb | prophoto-rgb | rec2020 | lab | oklab | <xyz-space>
<polar-color-space> = hsl | hwb | lch | oklch
<custom-color-space> = <dashed-ident>
<hue-interpolation-method> = [ shorter | longer | increasing | decreasing ] hue
<color-interpolation-method> = in [ <rectangular-color-space> | <polar-color-space> <hue-interpolation-method>? | <custom-color-space> ]
```
<a id="ref-for-typedef-dashed-ident⑧"></a>

<a id="ref-for-at-ruledef-profile①④"></a>

<a id="ref-for-color-interpolation-method②"></a>

The [\<dashed-ident\>](https://www.w3.org/TR/css-values-4/#typedef-dashed-ident) must have been declared in a valid [@color-profile](#at-ruledef-profile) rule, otherwise the [\<color-interpolation-method\>](#color-interpolation-method) is invalid.

<a id="ref-for-typedef-color②⓪"></a>

## <a id="resolving-color-values"></a>10.  Resolving [\<color\>](#typedef-color) Values 

<a id="ref-for-funcdef-color-mix⑥"></a>

### <a id="resolving-mix"></a>10.1.  Resolving [color-mix()](#funcdef-color-mix) Values 

<a id="ref-for-typedef-color②①"></a>

<a id="ref-for-valdef-color-currentcolor①"></a>

<a id="ref-for-funcdef-color-mix⑦"></a>

If all [\<color\>](#typedef-color) parameters resolve to the corresponding colors in their respective color spaces, the computed value is the mixed color, in the specified mixing color space, resolved according to [CSS Color 4 §  15. Resolving \<color\> Values](https://www.w3.org/TR/css-color-4/#resolving-color-values). Otherwise (if [currentColor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor) was used in the function), the computed value is the [color-mix()](#funcdef-color-mix) function with each <a id="ref-for-typedef-color②②"></a>\<color\> parameter resolved according to [CSS Color 4 §  15. Resolving \<color\> Values](https://www.w3.org/TR/css-color-4/#resolving-color-values), thus preserving inheritance into child elements.

Tests

- [color-mix-currentcolor-001.html](https://wpt.fyi/results/css/css-color/color-mix-currentcolor-001.html) [(live test)](http://wpt.live/css/css-color/color-mix-currentcolor-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-currentcolor-001.html)
- [color-mix-currentcolor-002.html](https://wpt.fyi/results/css/css-color/color-mix-currentcolor-002.html) [(live test)](http://wpt.live/css/css-color/color-mix-currentcolor-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-currentcolor-002.html)
- [color-mix-currentcolor-003.html](https://wpt.fyi/results/css/css-color/color-mix-currentcolor-003.html) [(live test)](http://wpt.live/css/css-color/color-mix-currentcolor-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-currentcolor-003.html)
- [color-mix-currentcolor-nested-for-color-property.html](https://wpt.fyi/results/css/css-color/color-mix-currentcolor-nested-for-color-property.html) [(live test)](http://wpt.live/css/css-color/color-mix-currentcolor-nested-for-color-property.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-currentcolor-nested-for-color-property.html)
- [nested-color-mix-with-currentcolor.html](https://wpt.fyi/results/css/css-color/nested-color-mix-with-currentcolor.html) [(live test)](http://wpt.live/css/css-color/nested-color-mix-with-currentcolor.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/nested-color-mix-with-currentcolor.html)

### <a id="resolving-rcs"></a>10.2.  Resolving Relative Color Syntax Values 

<a id="ref-for-typedef-color②③"></a>

<a id="ref-for-relative-color-processing-space⑤"></a>

If all [\<color\>](#typedef-color) parameters resolve to the corresponding colors in their respective color spaces, the computed value is the absolute <a id="ref-for-typedef-color②④"></a>\<color\> value, in the [relative color processing space](#relative-color-processing-space), resolved according to [CSS Color 4 §  15. Resolving \<color\> Values](https://www.w3.org/TR/css-color-4/#resolving-color-values).

Tests

- [color-computed-relative-color.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-relative-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-relative-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-relative-color.html)
- [color-valid-relative-color.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-relative-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-relative-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-relative-color.html)
- [color-invalid-relative-color.html](https://wpt.fyi/results/css/css-color/parsing/color-invalid-relative-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-invalid-relative-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-invalid-relative-color.html)

<a id="ref-for-valdef-color-currentcolor②"></a>

<a id="ref-for-typedef-color②⑤"></a>

Otherwise (if [currentColor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor) was used in the function), the computed value is the Relative Color Syntax function with the origin [\<color\>](#typedef-color) parameter resolved according to [CSS Color 4 §  15. Resolving \<color\> Values](https://www.w3.org/TR/css-color-4/#resolving-color-values), thus preserving inheritance into child elements.

Tests

- [relative-currentcolor-a98rgb-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-a98rgb-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-a98rgb-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-a98rgb-01.html)
- [relative-currentcolor-lch-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-lch-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-lch-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-lch-01.html)
- [relative-currentcolor-rgb-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-rgb-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-rgb-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-rgb-01.html)
- [relative-currentcolor-displayp3-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-displayp3-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-displayp3-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-displayp3-01.html)
- [relative-currentcolor-oklab-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-oklab-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-oklab-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-oklab-01.html)
- [relative-currentcolor-rgb-02.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-rgb-02.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-rgb-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-rgb-02.html)
- [relative-currentcolor-hsl-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-hsl-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-hsl-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-hsl-01.html)
- [relative-currentcolor-oklch-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-oklch-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-oklch-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-oklch-01.html)
- [relative-currentcolor-xyzd50-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-xyzd50-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-xyzd50-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-xyzd50-01.html)
- [relative-currentcolor-hsl-02.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-hsl-02.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-hsl-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-hsl-02.html)
- [relative-currentcolor-prophoto-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-prophoto-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-prophoto-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-prophoto-01.html)
- [relative-currentcolor-xyzd65-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-xyzd65-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-xyzd65-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-xyzd65-01.html)
- [relative-currentcolor-hwb-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-hwb-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-hwb-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-hwb-01.html)
- [relative-currentcolor-rec2020-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-rec2020-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-rec2020-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-rec2020-01.html)
- [relative-currentcolor-lab-01.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-lab-01.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-lab-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-lab-01.html)
- [relative-currentcolor-rec2020-02.html](https://wpt.fyi/results/css/css-color/relative-currentcolor-rec2020-02.html) [(live test)](http://wpt.live/css/css-color/relative-currentcolor-rec2020-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/relative-currentcolor-rec2020-02.html)

### <a id="resolving-device-cmyk-values"></a>10.3.  Resolving device-cmyk Values 

<a id="ref-for-number-value⑥③"></a>

<a id="ref-for-percentage-value③①"></a>

The computed and used value is the specified device-specific CMYK color, (with components as [\<number\>](https://www.w3.org/TR/css-values-4/#number-value), not [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)) paired with the specified alpha component (as a <a id="ref-for-number-value⑥④"></a>\<number\>, not a <a id="ref-for-percentage-value③②"></a>\<percentage\>; and defaulting to opaque if unspecified).

<a id="ref-for-actual-value"></a>

The [actual value](https://www.w3.org/TR/css-cascade-5/#actual-value) can vary based on the operation; for rendering to a CMYK-capable device, it may be rendered as a CMYK color; for blending with non-CMYK colors or rendering to a non-CMYK device, it must be converted as specified in [§ 6 Uncalibrated CMYK Colors: the device-cmyk() Function](#device-cmyk).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-device-cmyk-used"></a> For example,
>
> ```css
>  device-cmyk(0% 70% 20% 0%)
> ```
>
> has the specified and actual value
>
> ```css
>  device-cmyk(0 0.7 0.2 0)
> ```
>
> and will, if the implementation understands ICC profiles and has an appropriate profile installed, have the used value
>
> ```css
>  lab(63.673% 51.577 5.811)
> ```
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note: As with all colors, the used value is not available to script.

## <a id="serial"></a>11.  Serialization 

<a id="ref-for-funcdef-color-mix⑧"></a>

<a id="ref-for-funcdef-device-cmyk①⓪"></a>

This section extends [CSS Color 4 §  16. Serializing \<color\> Values](https://www.w3.org/TR/css-color-4/#serializing-color-values) to add serialization of the results of the [color-mix()](#funcdef-color-mix), [device-cmyk()](#funcdef-device-cmyk), and relative color functions.

In this section, the strings used in the specification and the corresponding characters are as follows.

|        |                     |
|--------|---------------------|
| String | Character           |
| " "    | U+0020 SPACE        |
| ","    | U+002C COMMA        |
| "-"    | U+002D HYPHEN-MINUS |
| "."    | U+002E FULL STOP    |
| "/"    | U+002F SOLIDUS      |

The string "." shall be used as a decimal separator, regardless of locale, and there shall be no thousands separator.

As usual, if the alpha of the result is exactly 1, it is omitted from the serialization; an implicit value of 1 (fully opaque) is the default.

### <a id="serial-color-mix"></a>11.1.  Serializing color-mix() 

<a id="ref-for-funcdef-color-mix⑨"></a>

<a id="ref-for-typedef-color-space②"></a>

<a id="ref-for-valdef-oklab-oklab①"></a>

<a id="ref-for-typedef-hue-interpolation-method③"></a>

The serialization of the declared value of a [color-mix()](#funcdef-color-mix) function is the string "color-mix(", followed by, unless the [\<color-space\>](#typedef-color-space) is [oklab](https://www.w3.org/TR/css-color-4/#valdef-oklab-oklab) (whether explicitly specified or defaulted): the string "in ", followed by the <a id="ref-for-typedef-color-space③"></a>\<color-space\> in all-lowercase, followed by, if a [\<hue-interpolation-method\>](#typedef-hue-interpolation-method) was specified and is not shorter hue, " " and the <a id="ref-for-typedef-hue-interpolation-method④"></a>\<hue-interpolation-method\> in all-lowercase, followed by ", ", followed by the serialization of each color argument (see below), separated by ", ", followed by ")".

<a id="ref-for-typedef-color②⑥"></a>

Each color argument is serialized as the serialized [\<color\>](#typedef-color), followed by, if a percentage is serialized for this argument (see below), " " and the serialized percentage.

Each color argument is serialized individually; in particular, identical colors are not collapsed into a single argument.

<a id="ref-for-funcdef-color-mix①⓪"></a>

The serialized percentages of the declared value of a [color-mix()](#funcdef-color-mix) function are determined as follows. Let <var>N</var> be the number of color arguments.

For each argument, let its <a id="effective-percentage"></a>effective percentage be:

- <a id="ref-for-percentage-value③③"></a>

  <a id="ref-for-funcdef-calc①"></a>

  its specified [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value), if one was explicitly provided and is not a [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) expression;

- <a id="ref-for-percentage-value③④"></a>

  <a id="ref-for-funcdef-calc②"></a>

  if its [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) was omitted, and none of the other specified <a id="ref-for-percentage-value③⑤"></a>\<percentage\>s are [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) expressions: <code>(100% − <var>specified sum</var>) / <var>omitted count</var></code>, where <var>specified sum</var> is the sum of the explicitly specified <a id="ref-for-percentage-value③⑥"></a>\<percentage\>s and <var>omitted count</var> is the number of arguments with omitted <a id="ref-for-percentage-value③⑦"></a>\<percentage\>s;

- otherwise, unknown.

<a id="ref-for-effective-percentage"></a>

If all [effective percentage](#effective-percentage)s are known and equal to <code>100% / <var>N</var></code>, no percentages are serialized. Otherwise, each argument’s percentage is serialized as follows:

- <a id="ref-for-percentage-value③⑧"></a>

  If a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) was explicitly specified, it is serialized as-is.

- <a id="ref-for-percentage-value③⑨"></a>

  <a id="ref-for-funcdef-calc③"></a>

  If a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) was omitted and none of the other specified <a id="ref-for-percentage-value④⓪"></a>\<percentage\>s are [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) expressions: the value <code>(100% − <var>specified sum</var>) / <var>omitted count</var></code> is serialized, where <var>specified sum</var> and <var>omitted count</var> are as defined above.

- <a id="ref-for-percentage-value④①"></a>

  <a id="ref-for-funcdef-calc④"></a>

  Otherwise (a [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) was omitted but another argument has a [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) <a id="ref-for-percentage-value④②"></a>\<percentage\>): nothing is serialized.

<a id="ref-for-funcdef-calc⑤"></a>

<a id="ref-for-percentage-value④③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [calc()](https://www.w3.org/TR/css-values-4/#funcdef-calc) values are considered unknown, so are <strong>never</strong> equal to <code>100% / <var>N</var></code>, and prevent computation of omitted [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)s.

<a id="ref-for-valdef-oklab-oklab②"></a>

<a id="ref-for-valdef-oklch-oklch①"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-serial-specified-mix"></a> For example, the serialized declared value of
>
> ```text
> color-mix(in oklab, teal, peru 40%)
> ```
>
> would be the string "color-mix(teal 60%, peru 40%)": the color space is omitted because it is the default ([oklab](https://www.w3.org/TR/css-color-4/#valdef-oklab-oklab)), and all percentages are serialized because they are not all equal to 100%/2 = 50%.
>
> The serialized declared value of
>
> ```text
> color-mix(in oklab, teal 50%, peru 50%)
> ```
>
> would be the string "color-mix(teal, peru)": both percentages equal 100%/2 = 50%, so they are all omitted.
>
> The serialized declared value of
>
> ```text
> color-mix(in oklab, teal 70%, peru 70%)
> ```
>
> would be the string "color-mix(teal 70%, peru 70%)": the specified percentages are 70%, not 50%, so they are not omitted, even though they normalize to 50% each during computation.
>
> The serialized declared value of
>
> ```text
> color-mix(in oklch longer hue, red, green, blue)
> ```
>
> would be the string "color-mix(in oklch longer hue, red, green, blue)": the color space ([oklch](https://www.w3.org/TR/css-color-4/#valdef-oklch-oklch)) is not the default, the hue interpolation method (longer) is not the default (shorter), and all percentages equal 100%/3, so they are all omitted.
>
> The serialized declared value of
>
> ```text
> color-mix(red 50%, green, blue)
> ```
>
> would be the string "color-mix(red 50%, green 25%, blue 25%)": the percentages are not all equal to 100%/3, so all are serialized, including the omitted ones which each get (100% − 50%) / 2 = 25%.

<a id="ref-for-funcdef-color-mix①①"></a>

<a id="ref-for-valdef-color-currentcolor③"></a>

<a id="ref-for-typedef-color②⑦"></a>

The serialization of the result of a [color-mix()](#funcdef-color-mix) function depends on whether the keyword [currentColor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor) is used in the mix. If so, the result is serialized as the declared value. This allows the correct mixture to be used on child elements whose color property has a different value. Otherwise, it is a [\<color\>](#typedef-color), as defined in [CSS Color 4 §  16. Serializing \<color\> Values](https://www.w3.org/TR/css-color-4/#serializing-color-values). The form used depends on the color space specified with "in":

|                    |                                |
|--------------------|--------------------------------|
| mixing color space | form                           |
| srgb               | color(srgb r g b)              |
| srgb-linear        | color(srgb-linear r g b)       |
| display-p3         | color(display-p3 r g b)        |
| a98-rgb            | color(a98-rgb r g b)           |
| prophoto-rgb       | color(prophoto-rgb r g b)      |
| rec2020            | color(rec2020 r g b)           |
| hsl                | color(srgb r g b)              |
| hwb                | color(srgb r g b)              |
| xyz-d65            | color(xyz-d65 x y z)           |
| xyz-d50            | color(xyz-d50 x y z)           |
| xyz                | color(xyz-d65 x y z) [¹](#fn1) |
| lab                | lab(l a b)                     |
| lch                | lch(l c h)                     |
| oklab              | oklab(l a b)                   |
| oklch              | oklch(l c h)                   |

<a id="fn1"></a>¹  
<a id="ref-for-valdef-color-xyz-d65"></a>

<a id="ref-for-valdef-color-xyz"></a>

Because [xyz](https://www.w3.org/TR/css-color-4/#valdef-color-xyz) is just an alias for [xyz-d65](https://www.w3.org/TR/css-color-4/#valdef-color-xyz-d65)

<a id="ref-for-valdef-hsl-hsl③"></a>

<a id="ref-for-valdef-hwb-hwb①"></a>

<a id="ref-for-color-space②"></a>

<a id="ref-for-missing-color-component"></a>

<a id="ref-for-carried-forward①"></a>

<a id="ref-for-valdef-light-dark-none⑤"></a>

However, if the result of mixing in the [hsl](https://www.w3.org/TR/css-color-4/#valdef-hsl-hsl) or [hwb](https://www.w3.org/TR/css-color-4/#valdef-hwb-hwb) [color space](https://www.w3.org/TR/css-color-4/#color-space) has at least one [missing color component](https://www.w3.org/TR/css-color-4/#missing-color-component) (including a <a id="ref-for-missing-color-component①"></a>missing alpha [carried forward](https://www.w3.org/TR/css-color-4/#carried-forward) per [CSS Color 4 § 13.3 Interpolating with Missing Components](https://www.w3.org/TR/css-color-4/#interpolation-missing)), the form used is the modern hsl(h s l / a) or hwb(h w b / a) syntax respectively, preserving the original color function and each [none](#valdef-light-dark-none) value per [CSS Color 4 § 16.2.2 CSS serialization of sRGB values](https://www.w3.org/TR/css-color-4/#css-serialization-of-srgb), rather than degrading to color(srgb r g b) (which would lose the <a id="ref-for-valdef-hsl-hsl④"></a>hsl/<a id="ref-for-valdef-hwb-hwb②"></a>hwb identity).

Tests

- [color-valid-color-mix-function.html](https://wpt.fyi/results/css/css-color/parsing/color-valid-color-mix-function.html) [(live test)](http://wpt.live/css/css-color/parsing/color-valid-color-mix-function.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-valid-color-mix-function.html)
- [color-mix-currentcolor-visited-getcomputedstyle.html](https://wpt.fyi/results/css/css-color/color-mix-currentcolor-visited-getcomputedstyle.html) [(live test)](http://wpt.live/css/css-color/color-mix-currentcolor-visited-getcomputedstyle.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-currentcolor-visited-getcomputedstyle.html)
- [color-mix-currentcolor-visited.html](https://wpt.fyi/results/css/css-color/color-mix-currentcolor-visited.html) [(live test)](http://wpt.live/css/css-color/color-mix-currentcolor-visited.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/color-mix-currentcolor-visited.html)

The <em>minimum</em> precision for round-tripping is the same as that specified in [CSS Color 4 §  16. Serializing \<color\> Values](https://www.w3.org/TR/css-color-4/#serializing-color-values).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-serial-computed-color-mix"></a> The result of this color mixture
>
> ```text
> color-mix(in lch, peru 40%, palegoldenrod)
> ```
>
> is serialized as the string "lch(79.7256 40.448 84.771)" while the result of
>
> ```text
> color-mix(in srgb, peru 40%, palegoldenrod)
> ```
>
> is serialized as the string "color(srgb 0.8816 0.7545 0.4988)".

### <a id="serial-origin-color"></a>11.2.  Serializing Origin Colors 

<a id="ref-for-origin-color④①"></a>

The serialization of a the declared value of a color used as the [origin color](#origin-color) inside of another color function as components of a declared value is:

1.  <a id="ref-for-funcdef-rgb⑥"></a>

    <a id="ref-for-funcdef-rgba⑤"></a>

    <a id="ref-for-funcdef-hsl⑥"></a>

    <a id="ref-for-funcdef-hsla⑥"></a>

    For [rgb()](https://www.w3.org/TR/css-color-4/#funcdef-rgb), [rgba()](https://www.w3.org/TR/css-color-4/#funcdef-rgba), [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl), [hsla()](https://www.w3.org/TR/css-color-4/#funcdef-hsla)

- <a id="ref-for-funcdef-rgb⑦"></a>

  <a id="ref-for-funcdef-rgba⑥"></a>

  <a id="ref-for-funcdef-hsl⑦"></a>

  <a id="ref-for-funcdef-hsla⑦"></a>

  the string identifying the canonical color function, "rgb" for [rgb()](https://www.w3.org/TR/css-color-4/#funcdef-rgb) and [rgba()](https://www.w3.org/TR/css-color-4/#funcdef-rgba), "hsl" for [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) and [hsla()](https://www.w3.org/TR/css-color-4/#funcdef-hsla), in all-lowercase, followed by "(", followed by a space separated list of the non-alpha components as specified (numbers serializing as numbers, percentages serializing as percentages, angles serializing as canonicalized angles in degrees, calc() serializing in its simplified form) with no clamping applied, followed by " / " and the alpha component as specified (using the same rules as the color components) if an alpha component is present, followed by ")".

> <strong data-conversion-semantic="note">Note</strong>
>
> NOTE: the same serialization is used regardless of whether the modern or legacy syntax was used.

2.  <a id="ref-for-funcdef-hwb⑤"></a>

    <a id="ref-for-funcdef-lab④"></a>

    <a id="ref-for-funcdef-lch⑤"></a>

    <a id="ref-for-funcdef-oklab③"></a>

    <a id="ref-for-funcdef-oklch⑤"></a>

    For [hwb()](#funcdef-hwb), [lab()](#funcdef-lab), [lch()](#funcdef-lch), [oklab()](#funcdef-oklab), [oklch()](#funcdef-oklch)

- the string identifying the color function in all-lowercase, followed by "(", followed by a space separated list of the non-alpha components as specified (numbers serializing as numbers, percentages serializing as percentages, angles serializing as canonicalized angles in degrees, calc() serializing in its simplified form) with no clamping applied, followed by " / " and the alpha component as specified (using the same rules as the color components) if an alpha component is present, followed by ")".

3.  <a id="ref-for-funcdef-color①⓪"></a>

    For [color()](#funcdef-color)

- the string "color(" followed by the canonical colorspace ("xyz-d65" for "xyz") in all-lowercase followed by a space, followed by a space separated list of the non-alpha components as specified (numbers serializing as numbers, percentages serializing as percentages, angles serializing as canonicalized angles in degrees, calc() serializing in its simplified form) with no clamping applied, followed by " / " and the alpha component as specified (using the same rules as the color components) if an alpha component is present, followed by ")".

### <a id="serial-relative-color"></a>11.3.  Serializing Relative Color Functions 

The serialization of the declared value of a relative color is:

1.  <a id="ref-for-funcdef-rgb⑧"></a>

    <a id="ref-for-funcdef-rgba⑦"></a>

    <a id="ref-for-funcdef-hsl⑧"></a>

    <a id="ref-for-funcdef-hsla⑧"></a>

    For [rgb()](https://www.w3.org/TR/css-color-4/#funcdef-rgb), [rgba()](https://www.w3.org/TR/css-color-4/#funcdef-rgba), [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl), [hsla()](https://www.w3.org/TR/css-color-4/#funcdef-hsla)

- <a id="ref-for-funcdef-rgb⑨"></a>

  <a id="ref-for-funcdef-rgba⑧"></a>

  <a id="ref-for-funcdef-hsl⑨"></a>

  <a id="ref-for-funcdef-hsla⑨"></a>

  the string identifying the canonical color function, "rgb" for [rgb()](https://www.w3.org/TR/css-color-4/#funcdef-rgb) and [rgba()](https://www.w3.org/TR/css-color-4/#funcdef-rgba), "hsl" for [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) and [hsla()](https://www.w3.org/TR/css-color-4/#funcdef-hsla), in all-lowercase, followed by "(from ", followed by the [serialization of the origin color](#serial-origin-color) using the rules for serializing nested origin colors, followed by a single space, followed by a space separated list of the non-alpha channel arguments as specified (identifiers serializing as identifiers, numbers and percentages serializing as numbers, angles serializing as canonicalized angles in degrees, calc() serializing in its simplified form), followed (if the alpha component is non-unity) by " / " and the alpha component as specified (using the same rules as the color channel arguments, but clamped) followed by ")".

2.  <a id="ref-for-funcdef-hwb⑥"></a>

    <a id="ref-for-funcdef-lab⑤"></a>

    <a id="ref-for-funcdef-lch⑥"></a>

    <a id="ref-for-funcdef-oklab④"></a>

    <a id="ref-for-funcdef-oklch⑥"></a>

    For [hwb()](#funcdef-hwb), [lab()](#funcdef-lab), [lch()](#funcdef-lch), [oklab()](#funcdef-oklab), [oklch()](#funcdef-oklch)

- the string identifying the color function in all-lowercase, followed by "(from ", followed by the [serialization of the origin color](#serial-origin-color) using the rules for serializing nested origin colors, followed by a single space, followed by a space separated list of the non-alpha channel arguments as specified (identifiers serializing as identifiers, numbers and percentages serializing as numbers, angles serializing as canonicalized angles in degrees, calc() serializing in its simplified form), followed (if the alpha component is non-unity) by " / " and the alpha component as specified (using the same rules as the color channel arguments, but clamped) followed by ")".

3.  <a id="ref-for-funcdef-color①①"></a>

    For [color()](#funcdef-color)

- the string "color(from ", followed by the [serialization of the origin color](#serial-origin-color) using the rules for serializing nested origin colors, followed by a single space, followed by the canonical colorspace ("xyz-d65" for "xyz") in all-lowercase, followed by a single space, followed by a space separated list of the non-alpha channel arguments as specified (identifiers serializing as identifiers, numbers and percentages serializing as numbers, angles serializing as canonicalized angles in degrees, calc() serializing in its simplified form), followed (if the alpha component is non-unity) by " / " and the alpha component as specified (using the same rules as the color channel arguments, but clamped) followed by ")".

4.  <a id="ref-for-funcdef-alpha③"></a>

    for [alpha()](#funcdef-alpha)

- the string "alpha(from ", followed by the [serialization of the origin color](#serial-origin-color) using the rules for serializing nested origin colors, followed by a single space, followed by the canonical colorspace ("xyz-d65" for "xyz") in all-lowercase, followed by " / " and the alpha component as specified (using the same rules as the color channel arguments, but clamped) followed by ")".

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-serial-rcs-specified-simple"></a> For example, the serialization of the declared value of
>
> ```css
> OkLcH(from peru  l    c  h)
> ```
>
> is the string "oklch(from peru l c h)"

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-serial-rcs-calc"></a> For example, the serialization of the declared value of
>
> ```css
> rgb(from red calc(r / 2) g calc(30%));
> ```
>
> is the string "rgb(from red calc(0.5 \* r) g calc(30%))", while the serialization of the computed value is the string "color(srgb 0.5 0 0.3)".

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-serial-rcs-hsl"></a> For example, the serialization of the declared value of
>
> ```css
> hsl(from hsl(0deg 10% 50%) h s l);
> ```
>
> is the string "hsl(from hsl(0deg 10% 50%) h s l)", while the serialization of the computed value is the string "color(srgb 0.55 0.45 0.45)".

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-serial-rcs-nested-none"></a> For example, the serialization of the declared value of
>
> ```css
> hsl(from hsl(none 10% 50%) h s l);
> ```
>
> is the string "hsl(from hsl(none 10% 50%) h s l)", while the serialization of the computed value is the string "hsl(none 10% 50%)".
>
> <a id="ref-for-missing-color-component②"></a>
>
> <a id="ref-for-funcdef-hsl①⓪"></a>
>
> <a id="ref-for-relative-color①⑥"></a>
>
> The computed value carries forward the [missing](https://www.w3.org/TR/css-color-4/#missing-color-component) hue, giving an [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) [relative color](#relative-color) whose hue is itself <a id="ref-for-missing-color-component③"></a>missing. Because the resolved value contains <a id="ref-for-missing-color-component④"></a>missing color components, the serialization uses the modern <a id="ref-for-funcdef-hsl①①"></a>hsl() form, yielding the string "hsl(none none none / none)" rather than color(srgb 0.55 0.45 0.45).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-serial-rcs-alpha-none"></a> For example, the serialization of the declared value of
>
> ```css
> hsl(from rebeccapurple none none none / none);
> ```
>
> is the string "hsl(from rebeccapurple none none none / none)".
>
> <a id="ref-for-missing-color-component⑤"></a>
>
> <a id="ref-for-analogous-components①"></a>
>
> <a id="ref-for-funcdef-hsl①②"></a>
>
> <a id="ref-for-relative-color①⑦"></a>
>
> The computed value carries forward the [missing](https://www.w3.org/TR/css-color-4/#missing-color-component) alpha (alpha is its own [analogous component](https://www.w3.org/TR/css-color-4/#analogous-components)), giving an [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) [relative color](#relative-color) whose hue, saturation, lightness, and alpha are all <a id="ref-for-missing-color-component⑥"></a>missing. Because the resolved value contains <a id="ref-for-missing-color-component⑦"></a>missing color components, the serialization uses the modern <a id="ref-for-funcdef-hsl①③"></a>hsl() form, yielding the string "hsl(none none none / none)" rather than color(srgb 0 0 0 / none).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-serial-rcs-hsl-unclamped"></a> For example, the serialization of the declared value of
>
> ```css
> hsl(from hsl(127.9 302% 25.33%) h s l);
> ```
>
> is the string "hsl(from hsl(127.9 302% 25.33%) h s l)", while the serialization of the computed value is the string "color(srgb -0.511666 1.018266 -0.310225)".

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-serial-rcs-currentcolor"></a> Given the following HTML (note the color property set on this element):
>
> ```html
> <div id="example" 
>   style="background-color: rgb(from currentcolor r g calc(b / 2)); 
>   color: blue;">
> </div>
> ```
>
> The serialization of the declared value of background-color is the string "rgb(from currentcolor r g calc(b / 2))" while the serialization of the computed value is the string "color(srgb 0 0 0.5)"

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-serial-rcs-modern-alpha"></a> For example, the serialization of the declared value of
>
> ```css
> alpha(from red / 0.5)
> ```
>
> is the string "color(srgb 1 0 0 / 0.5)" because trailing zeros are trimmed, while the serialization of the declared value of
>
> ```css
> alpha(from gold / calc(2/3))
> ```
>
> is the string "color(srgb 1 0.843137 0 / 0.666667)" because, by [CSS Color 4 § 16.1.2 Serializing modern alpha values](https://www.w3.org/TR/css-color-4/#serializing-modern-alpha-values) modern alpha values are serialized with six decimal places (unless there are trailing zeroes).

<a id="ref-for-valdef-color-currentcolor④"></a>

<a id="ref-for-origin-color④②"></a>

<a id="ref-for-typedef-color②⑧"></a>

The serialization of the result of a relative color function depends on whether the keyword [currentColor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor) is the [origin color](#origin-color). If so, the result is serialized as the declared value. This allows the correct value to be used on child elements whose color property has a different value. Otherwise, it is the resolved value, which is a [\<color\>](#typedef-color), as defined in [CSS Color 4 §  16. Serializing \<color\> Values](https://www.w3.org/TR/css-color-4/#serializing-color-values).

<a id="ref-for-relative-color-processing-space⑥"></a>

The form used depends on the [relative color processing space](#relative-color-processing-space):

| <a id="ref-for-relative-color-processing-space⑦"></a>[relative color processing space](#relative-color-processing-space) | form                      |
|----------------------------------------------------------------------------------------|---------------------------|
| srgb                                                                                   | color(srgb r g b)         |
| srgb-linear                                                                            | color(srgb-linear r g b)  |
| display-p3                                                                             | color(display-p3 r g b)   |
| a98-rgb                                                                                | color(a98-rgb r g b)      |
| prophoto-rgb                                                                           | color(prophoto-rgb r g b) |
| rec2020                                                                                | color(rec2020 r g b)      |
| hsl                                                                                    | color(srgb r g b)         |
| hwb                                                                                    | color(srgb r g b)         |
| xyz-d65                                                                                | color(xyz-d65 x y z)      |
| xyz-d50                                                                                | color(xyz-d50 x y z)      |
| xyz                                                                                    | color(xyz-d65 x y z)      |
| lab                                                                                    | lab(l a b)                |
| lch                                                                                    | lch(l c h)                |
| oklab                                                                                  | oklab(l a b)              |
| oklch                                                                                  | oklch(l c h)              |

<a id="ref-for-funcdef-hsl①④"></a>

<a id="ref-for-funcdef-hwb⑦"></a>

<a id="ref-for-relative-color①⑧"></a>

<a id="ref-for-missing-color-component⑧"></a>

<a id="ref-for-carried-forward②"></a>

<a id="ref-for-origin-color④③"></a>

<a id="ref-for-valdef-light-dark-none⑥"></a>

<a id="ref-for-valdef-hsl-hsl⑤"></a>

<a id="ref-for-valdef-hwb-hwb③"></a>

However, if the resolved value of an [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) or [hwb()](#funcdef-hwb) [relative color](#relative-color) has at least one [missing color component](https://www.w3.org/TR/css-color-4/#missing-color-component) (including a <a id="ref-for-missing-color-component⑨"></a>missing alpha [carried forward](https://www.w3.org/TR/css-color-4/#carried-forward) from the [origin color](#origin-color) per [CSS Color 4 § 13.3 Interpolating with Missing Components](https://www.w3.org/TR/css-color-4/#interpolation-missing)), the form used is the modern hsl(h s l / a) or hwb(h w b / a) syntax respectively, preserving the original color function and each [none](#valdef-light-dark-none) value, rather than degrading to color(srgb r g b) (which would lose the [hsl](https://www.w3.org/TR/css-color-4/#valdef-hsl-hsl)/[hwb](https://www.w3.org/TR/css-color-4/#valdef-hwb-hwb) identity). This parallels [§ 11.2 Serializing Origin Colors](#serial-origin-color), which always emits the modern slash syntax for origin colors, and follows the general sRGB serialization rules in [CSS Color 4 § 16.2.2 CSS serialization of sRGB values](https://www.w3.org/TR/css-color-4/#css-serialization-of-srgb).

Tests

The <em>minimum</em> precision for round-tripping is the same as that specified in [CSS Color 4 § 16.5 Serializing values of the color() function](https://www.w3.org/TR/css-color-4/#serializing-color-function-values).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-serial-rcs-computed"></a> The result of serializing
>
> ```css
> lch(from peru calc(l * 0.8) calc(c * 0.7) calc(h + 180)) 
> ```
>
> is the string "lch(49.80224 37.80819 243.6803)"

### <a id="serial-custom-color"></a>11.4.  Serializing Custom Color Spaces 

<a id="ref-for-funcdef-color①②"></a>

The precision with which [color()](#funcdef-color) component values are retained, and thus the number of significant figures in the serialized value, is not defined in this specification, but for CMYK color spaces must at least be sufficient to round-trip values with eight bit precision; this will result in at least two decimal places unless trailing zeroes have been omitted.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-color-swop-serial"></a>
>
> The serialized value of the color in
>
> ```css
> @color-profile --swop5c {
>     src: url('https://example.org/SWOP2006_Coated5v2.icc');
>     }
>     .header {
>     background-color:    color(--swop5c  0% 70.0% 20.00% .0%);
>     }
> ```
>
> is the string "color(--swop5c 0 0.7 0.2 0)"

### <a id="serializing-device-cmyk-values"></a>11.5.  Serializing device-cmyk Values 

<a id="ref-for-funcdef-device-cmyk①①"></a>

<a id="ref-for-computed-value"></a>

The serialized form of [device-cmyk()](#funcdef-device-cmyk) values is derived from the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) and uses the <a id="ref-for-funcdef-device-cmyk①②"></a>device-cmyk() form, with lowercase letters for the function name.

<a id="ref-for-number-value⑥⑤"></a>

The component values are serialized in base 10, as [\<number\>](https://www.w3.org/TR/css-values-4/#number-value). A single ASCII space character " " must be used as the separator between the component values.

Trailing fractional zeroes in any component values must be omitted; if the fractional part consists of all zeroes, the decimal point must also be omitted.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-device-cmyk-serial"></a>
>
> The serialized value of the color
>
> ```css
>   device-cmyk(0 81% 81% 30%)
> ```
>
> is the string "device-cmyk(0 0.81 0.81 0.3)"

<a id="ref-for-funcdef-device-cmyk①③"></a>

The precision with which [device-cmyk()](#funcdef-device-cmyk) component values are retained, and thus the number of significant figures in the serialized value, is not defined in this specification, but must at least be sufficient to round-trip values with eight bit precision; this will result in at least two decimal places unless trailing zeroes have been omitted. Values must be [rounded towards +∞](https://drafts.csswg.org/css-values-4/#combine-integers), not truncated.

Unitary alpha values are not explicitly serialized. Non-unitary alpha values must be explicitly serialized, and the string " / " (an ASCII space, then forward slash, then another space) must be used to separate the black ("k") color component value from the alpha value.

## <a id="apis"></a>12. APIs

### <a id="the-csscolorprofilerule-interface"></a>12.1. The `CSSColorProfileRule` interface

<a id="ref-for-csscolorprofilerule"></a>

<a id="ref-for-at-ruledef-profile①⑤"></a>

The [CSSColorProfileRule](#csscolorprofilerule) interface represents a [@color-profile](#at-ruledef-profile) rule.

<a id="ref-for-Exposed"></a>

<a id="csscolorprofilerule"></a>

<a id="ref-for-cssrule"></a>

<a id="ref-for-cssomstring"></a>

<a id="ref-for-dom-csscolorprofilerule-name"></a>

<a id="ref-for-cssomstring①"></a>

<a id="ref-for-dom-csscolorprofilerule-src"></a>

<a id="ref-for-cssomstring②"></a>

<a id="ref-for-dom-csscolorprofilerule-renderingintent"></a>

<a id="ref-for-cssomstring③"></a>

<a id="ref-for-dom-csscolorprofilerule-components"></a>

```text
[Exposed=Window]
interface CSSColorProfileRule : CSSRule {
  readonly attribute CSSOMString name ;
  readonly attribute CSSOMString src ;
  readonly attribute CSSOMString renderingIntent ;
  readonly attribute CSSOMString components ;
};
```
<a id="ref-for-cssomstring④"></a>

<a id="dom-csscolorprofilerule-name"></a>`name`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring), readonly

<a id="ref-for-css-color-profile③"></a>

The <var>name</var> attribute on getting must return a `CSSOMString` object that contains the serialization of the [color profile’s](#css-color-profile) <var>name</var> defined for the associated rule.

<a id="ref-for-cssomstring⑤"></a>

<a id="dom-csscolorprofilerule-src"></a>`src`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring), readonly

<a id="ref-for-cssomstring⑥"></a>

<a id="dom-csscolorprofilerule-renderingintent"></a>`renderingIntent`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring), readonly

<a id="ref-for-cssomstring⑦"></a>

<a id="dom-csscolorprofilerule-components"></a>`components`, of type [CSSOMString](https://www.w3.org/TR/cssom-1/#cssomstring), readonly

The remaining attributes on getting must return a `CSSOMString` object that contains the serialization of the associated descriptor defined for the associated rule. If the descriptor was not specified in the associated rule, the attribute must return an empty string.

## <a id="sample"></a>13.  Default Style Rules 

The following stylesheet is informative, not normative. This style sheet could be used by an implementation as part of its default styling of HTML Family documents.

```css
/* traditional desktop user agent colors for hyperlinks */
:link { color: LinkText; }
:visited { color: VisitedText; }
:active { color: ActiveText; }

/* a reasonable, conservative default for device-cmyk */
@color-profile device-cmyk {
  src: url('https://drafts.csswg.org/css-color-4/ICCprofiles/Coated_Fogra39L_VIGC_300.icc');
}
```
## <a id="color-conversion-code"></a>14.  Sample code for Color Conversions

<em>This section is not normative.</em>

The naive conversion from device-cmyk is trivial:

```javascript
function naive(cmyk) {
  // naively convert an array of CMYK values
  // to sRGB
  let [cyan, magenta, yellow, black] = cmyk;
    let red = 1 - Math.min(1, cyan * (1 - black) + black);
    let green = 1 - Math.min(1, magenta * (1 - black) + black);
    let blue = 1 - Math.min(1, yellow * (1 - black) + black);
    return [red, green, blue];
}
```
## <a id="security"></a>15.  Security Considerations 

This specification adds to CSS the on-demand downloading of ICC profiles. These do not contain executable code, and thus do not constitute an increased security risk.

## <a id="privacy"></a>16.  Privacy Considerations 

No new privacy considerations have been reported on this specification.

## <a id="a11y-sec"></a>17. Accessibility Considerations 

This specification adds a way to ensure adequate contrast for text whose background is a user-specified color, including dynamic colors.

## <a id="changes"></a>18.  Changes 

### <a id="changes-20260113"></a>18.1.  Since the [Working Draft of 13 January 2026](https://www.w3.org/TR/2026/WD-css-color-5-20260113/) 

- Changed the previous term "Required conversion" to "Conversion, if required" because the former term was misleading ([Issue 4204](https://github.com/w3c/csswg-drafts/issues/14204))
- Removed special casing of 100% from the color-mix() algorithm, thus avoiding a discontinuity near fully-transparent colors ([Issue 14014](https://github.com/w3c/csswg-drafts/issues/14014)), ([Issue 13996](https://github.com/w3c/csswg-drafts/issues/13996))
- Guarded against division by zero in the color-mix() algorithm ([Issue 14013](https://github.com/w3c/csswg-drafts/issues/14013)), ([Issue 13996](https://github.com/w3c/csswg-drafts/issues/13996))
- Added a backlink from Color Interpolation in this specification, to the same section in CSS Color 4 where most of this is defined ([Issue 13788](https://github.com/w3c/csswg-drafts/issues/13788))
- Added a second form of the light-dark() function, which takes a pair of images rather than a pair of colors ([Issue 12513](https://github.com/w3c/csswg-drafts/issues/12513))
- Added a "none" option to the image form of light-dark() ([Issue 12513](https://github.com/w3c/csswg-drafts/issues/12513))
- Added a color-mix() example with three colors, now that it is no longer restricted to just two.
- Updated color-mix() serialization: omit color space when it is the default (oklab), serialize hue interpolation method when non-default, generalize percentage rules for N colors (omit all when equal to 100%/N, otherwise serialize all), and clarify that identical colors are not collapsed ([Issue 13320](https://github.com/w3c/csswg-drafts/issues/13320))
- Improve visibility of the constraint that shorter is the default he interpolation method
- Placed Custom Color Spaces, '@color-profile', 'device-cmyk()', Relative Alpha Colors at-risk
- Preserved hsl/hwb identity when result contains none ([Issue 10254](https://github.com/w3c/csswg-drafts/issues/10254))
- Changed the computed value of light-dark() none from linear-gradient(transparent) to image(transparent) ([Issue 13897](https://github.com/w3c/csswg-drafts/pull/13897))
- Reworked the color-scheme section to use the concepts of 'page color scheme' and 'element color scheme' ([PR 13857](https://github.com/w3c/csswg-drafts/pull/13857)) ([Issue 13377](https://github.com/w3c/csswg-drafts/issues/13377)) ([Issue 7213](https://github.com/w3c/csswg-drafts/issues/7213)) ([Issue 7493](https://github.com/w3c/csswg-drafts/issues/7493))
- Guarded against division by zero in color-mix() algorithm ([Issue 14013](https://github.com/w3c/csswg-drafts/issues/14013))
- Removed special casing of 100% from the color-mix() algorithm, thus avoiding a discontinuity near fully-transparent colors ([Issue 14014](https://github.com/w3c/csswg-drafts/issues/14014))
- Defined the term relative color processing space ([Issue 13394](https://github.com/w3c/csswg-drafts/issues/13994)) ([Issue 13392](https://github.com/w3c/csswg-drafts/issues/13992))
- Serialization of relative colors (including the alpha-only form) depends on the relative color processing space ([Issue 13992](https://github.com/w3c/csswg-drafts/issues/13992))
- Defined serialization of the declared value of the alpha() function ([Issue 13992](https://github.com/w3c/csswg-drafts/issues/13992))
- Defined resolving of relative colors in terms of the relative color processing space, thus using modern syntax ([Issue 13394](https://github.com/w3c/csswg-drafts/issues/13994))

### <a id="changes-20250318"></a>18.2.  Since the [Working Draft of 18 March 2025](https://www.w3.org/TR/2025/WD-css-color-5-20250318/) 

- Added backlinks from relative colors to the corresponding definition in CSS Color 4 ([Issue 13286](https://github.com/w3c/csswg-drafts/issues/13286))
- Defined that if there are multiple @color-profile rules, the last one wins [Issue 12980](https://github.com/w3c/csswg-drafts/issues/12980))
- Default to oklab for color-mix(), allow color-interpolation-method to be omitted ([Issue 10484](https://github.com/w3c/csswg-drafts/issues/10484))
- Removed leftover text which still assumed color-mix() was limited to two colors
- Updated serialization of origin colors ([Issue 10328](https://github.com/w3c/csswg-drafts/issues/10328))
- Added display-p3-linear to color spaces for interpolation
- Added the alpha() RCS function
- Updated explanatory prose which still limited color-mix to two colors
- Clarified fetching external URLs for style resources
- Clarified that sole item in color-mix() is returned in the specified color space
- Made color mixing algorithm pass the normalization flag
- Made color-mix() accept 1+ arguments, to match \*-mix() in general
- Ported definitions for colors that "resolve to sRGB" and "support legacy color syntax" from CSS Color 4
- Added sample javascript code for naively converting from device-cmyk
- Added worked example of color-mix() using device-cmyk fallback color
- Explicitly clarified (rather than inferring) that there is no RCS for device-cmyk()
- Defined that in color-mix(), if the percentages sum to zero, return transparent
- Properly exported the term "required conversion"

### <a id="changes-20240229"></a>18.3.  Since the [Working Draft of 29 February 2024](https://www.w3.org/TR/2024/WD-css-color-5-20240229/) 

- <a id="ref-for-valdef-light-dark-none⑦"></a>

  Clarified that component keywords can return [none](#valdef-light-dark-none) as well as a number

- Added examples of serialization of nested color functions

- Defined edge cases of color-mix() with calc, by WG resolution

- Remove the "invalid if sum to zero" wording for color-mix(), per WG resolution

- Consistently use of "color component" rather than "color channel" (both were used)

- Simplified contrast-color(), per WG resolution

- Link to term premultiplied consistently

- Validate color profile components case-insensitively

- Added contrast-color() to the color type definition

- Added accessibility considerations section

- Added references to FOGRA39, 51 and 55

- Removed mention of \<hue-interpolation-method\> being an error condition for rectangular color spaces, as the grammar does not allow it

- Clarified which color space the relative color component keywords relate to

- Separated out the conceptual aspects of relative colors from the syntactic details

- Ensured adequate contrast for text in the deltaE table

- Clarified that relative color components are not clamped, while relative alpha is

### <a id="changes-20220628"></a>18.4.  Since the [Working Draft of 28 June 2022](https://www.w3.org/TR/2022/WD-css-color-5-20220628/) 

- Described CSSOM serialization in terms of declared values, rather than specified values

- <a id="ref-for-funcdef-contrast-color⑥"></a>

  Added the [contrast-color()](#funcdef-contrast-color) function

- <a id="ref-for-funcdef-color-mix①②"></a>

  Explicitly linked to the CSS color 4 section on interpolation, for [color-mix()](#funcdef-color-mix)

- Removed a leftover mention of gamut mapping to HSL

- Defined absolute colors in prose, rather than as part of the grammar

- Explicitly repeated the reference ranges for percent to number conversion in RCS for each case

- <a id="ref-for-funcdef-hwb⑧"></a>

  <a id="ref-for-funcdef-hsl①⑤"></a>

  Better defined serialization of relative colors whose origin color is currentColor. sRGB, [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) and [hwb()](#funcdef-hwb) serialize using color(srgb ...) to enable round-tripping.

- <a id="ref-for-funcdef-light-dark③"></a>

  Updated abstract to mention the [light-dark()](#funcdef-light-dark) function

- Ported forward the larger list of rectangular color spaces from CSS Color 4

- Correction to the grammar for custom params (whitespace separated, not comma separated)

- <a id="ref-for-valdef-light-dark-none⑧"></a>

  <a id="ref-for-funcdef-device-cmyk①④"></a>

  Corrections to the grammar of [device-cmyk()](#funcdef-device-cmyk) which was missing [none](#valdef-light-dark-none), and the previous (legacy) syntax from CSS GCPM

- <a id="ref-for-funcdef-color-mix①③"></a>

  [color-mix()](#funcdef-color-mix) was missing from the grammar of the color type

- Clarified serialization of specified RCS values

- <a id="ref-for-funcdef-light-dark④"></a>

  Added the [light-dark()](#funcdef-light-dark) function

- Corrected color-mix percent normalization algorithm to include explicit 50% edge case

- Updated HSL example which still used gamut mapping before sRGB to HSL conversion step

- Fixed syntax highlighting in some examples

- Clarified that RCS origin colors can include optional alpha

- Fixed erroneous use of powerless components

- Used reference ranges for percent to number conversions

- <a id="ref-for-funcdef-hwb⑨"></a>

  <a id="ref-for-funcdef-hsl①⑥"></a>

  Made [hsl()](https://www.w3.org/TR/css-color-4/#funcdef-hsl) and [hwb()](#funcdef-hwb) component values number, per CSSWG resolution. Also made all the hue angle component values number, in degrees. The RCS intro already said this but the change had not been fully propagated.

- Added an RCS example in CIE XYZ D65 colorspace

- Removed un-needed and unchanged copy of a section of CSS Color 4 which was confusing to read

- <a id="ref-for-funcdef-color①③"></a>

  Corrected section title to "Specifying Predefined and Custom Color Spaces: the [color()](#funcdef-color) Function"

- Defined that HWB now allows number, previously it was percentage only

- Clarified that, if using RCS components in unusual positions, there is no "magic scaling"; use calc() if you want that. Added example.

- Clarified which of CIE Lightness and Oklab lightness are being used

- <a id="ref-for-valdef-color-currentcolor⑤"></a>

  <a id="ref-for-funcdef-color-mix①④"></a>

  Clarified serialization of the result of a [color-mix()](#funcdef-color-mix) function if [currentColor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor) is used. Added an example of this.

- Fixed typo’s in some color-mix examples

- Fixed an example which used 0 instead of none for powerless components

- Defined serialization of specified value of color-mix and clarified that it serializes with specified, not normalized, percentages

- Removed the unchanged alpha-value definition, link to Color 4 instead

- Used separate grammar productions for modern and legacy rgb, rgba, hsl, and hsla

- Clarified that the origin color is unrestricted and can use either modern or legacy syntax

- Added a new color production, clarified that RCS can be nested

- Clarified that percentage and number can be freely mixed for RGB, HSL; not restricted to RCS any more

- Clarified that RCS only applies to the modern color syntax

- Defined required color space conversions, clarified that un-needed conversions can be skipped

- Added correct and incorrect gradient rendering images

- Clarified that un-named components in custom color spaces are still valid

- Improved some examples to make them clearer

- Defined RCS with missing components

- Added dashed-ident to color-interpolation method, can now interpolate in custom color spaces

- Clarified that hue components in RCS resolve to a number, in degrees

- Specified the resolved RCS where the origin color is currentColor

- Channel keywords can only have a single type

- Corrected invalid example of RCS on legacy syntax, to actually use the legacy syntax

- Used consistent serialization in examples

- <a id="ref-for-funcdef-device-cmyk①⑤"></a>

  Noted that a defined serialization for [device-cmyk()](#funcdef-device-cmyk) exists

- <a id="ref-for-csscolorprofilerule①"></a>

  Added the [CSSColorProfileRule](#csscolorprofilerule) interface

- Consistent capitalization of Oklab and OkLCh

- Accessibility improvements for color swatches

- Accessibility improvements for diagrams

- Fixed unwanted clipping of some color swatches

- Improved alternative text for some figures

- Added some missing color swatches

- Added row and column labels for MacBeth images and table of deltaE

- Better labelling on hue-rotate diagram

- Better descriptions of colors on diagrams, improve Accessibility

- Ensured all diagrams and figures have IDs, selflinks

### <a id="changes-20220428"></a>18.5.  Since the [Working Draft of 28 April 2022](https://www.w3.org/TR/2022/WD-css-color-5-20220428/) 

- Fixed a typo in definition of rgb()

- Editorial improvements (capitalization, spelling, clarity)

- Exported definitions for other specifications to use

- <a id="ref-for-valdef-light-dark-none⑨"></a>

  Add missing [none](#valdef-light-dark-none) to alpha in grammar of color()

- Moved the color-contrast() function to level 6

### <a id="changes-20211215"></a>18.6.  Since the [Working Draft of 15 December 2021](https://www.w3.org/TR/2021/WD-css-color-5-20211215/) 

- Forgiveness of too many/too few parameters in color() restricted to custom color spaces

- Changed RCS to allow number or percent everywhere

- Clearly described potential clash of component names with named constants such as PI

- Clarified that Relative Color Syntax does not use legacy (comma-separated) syntax

- <a id="ref-for-valdef-light-dark-none①⓪"></a>

  Corrected grammar of the rgb() function, [none](#valdef-light-dark-none)' was not listed as an option for alpha

- Changed serialization of color-mix() which uses hsl or hwb, to maximize precision

- Added an out of gamut color-mix example

- Use the term "cannot express the color" to describe HSL and HWB models which cannot represent extended, out of gamut colors.

- Fixed some spelling errors

### <a id="changes-20210601"></a>18.7.  Since the [Working Draft of 1 June 2021](https://www.w3.org/TR/2021/WD-css-color-5-20210601/) 

- Using \<hue-interpolation-method\> in rectangular spaces is an error
- Changed old \<hue-adjuster\> to new \<hue-interpolation-method\>
- Moved @color-profile and device-cmyk to level 5 per CSSWG resolution
- Excluded none as a component name
- Added OkLCh relative color syntax example
- Defined interpolation color space
- Defined loading color profiles in terms of fetch
- Clarified that contrast is calculated relative to D65-adapted CIE XYZ
- Added oklab() and oklch() to serialization of color-mix()
- Added oklab() and oklch() relative color syntax
- Added lch vs. oklch mixing example
- Prefer oklab and oklch for mixing
- Changed xyz to D65-reative, following CSS Color 4
- Added oklab and oklch color spaces
- Defined how to resolve color-contrast())
- Clarified minimum precision of serialized forms
- Clarified that CIE LCH is meant
- Added some more examples
- Removed color-adjust(), keeping relative color syntax
- Defined serialization of the results of the color-mix, color-contrast, and relative color syntaxes

### <a id="changes-20200303"></a>18.8.  Since the [FPWD of 10 June 2020](https://www.w3.org/TR/2020/WD-css-color-5-20200303/) 

- Added relative color syntax for the color() function

- Clarified that the color-adjuster is not optional

- Clarified that the percentage in color-mix is mandatory

- Moved hue-adjuster back to color-mix whee it belongs

- Added example with different mixing color spaces

- Added examples of percentage normalization in color-mix()

- Explicitly excluded negative percentages in color-mix()

- Percentages in color-mix() summing to less than 100% produce an alpha transparency less than 100%

- <a id="ref-for-typedef-color-space④"></a>

  Consistently used the term color space rather than colorspace, defined [\<color-space\>](#typedef-color-space) token

- Corrected color-contrast grammar

- Added an optional target contrast ratio to color-contrast()

- Corrected adjuster grammar

- Noted that the corner case of percentages summing to zero needs to be handled

- Clarified order of operations in color-mix()

- Updated examples to match current grammar

- Defined how percentages are normalized

- Clarify meaning of 0% and 100% in color-mix()

- Definition of adjusters moved from color-mix() to color-adjust()

- Allow arguments to color-mix() to be in any order

- Mandatory color space for color-mix()

- Allowed the percentage in color-mix() to come before the color

- Added explicit algorithm for color-mix()

- Removed adjusters from color-mix() and simplified the grammar

- Added the "in" keyword to specify the color space used for mixing

- Required color-contrast() list to have at least two items

- Improved explanation of the relative color syntax

- Link to CSS 4 definition of color serialization

- Added separate section for color spaces

- Updated color-adjust example

- Added explanatory diagrams

- Deal with unresolved percentages

- Normalize arguments to color-mix

- Allow percentages for adjusters

- Link fixes

- Updated color-mix grammar, allowing adjusters, add alpha adjuster

- Corrections to some examples

- Updated Security and Privacy section

- added vs keyword to color-contrast

- added xyz adjuster to grammar

- added hue adjuster keywords

- add XYZ color space for mixing

- defined color-adjuster and color space

- allowed mix percent to default to 50%

- added worked examples and diagrams

- corrected minor spelling, syntax and formatting issues

- Added section on resolving color-contrast() values

### <a id="changes-from-4"></a>18.9.  Changes from CSS Color 4 

One major change, compared to CSS Color 4, is that CSS colors are no longer restricted to predefined RGB spaces such as sRGB or display-p3.

To support this, several brand new features have been added:

1.  <a id="ref-for-at-ruledef-profile①⑥"></a>

    <a id="ref-for-funcdef-color①④"></a>

    The [color()](#funcdef-color) function is extended by the [@color-profile](#at-ruledef-profile) at-rule, for profiled device-dependent color, including calibrated CMYK.

2.  <a id="ref-for-funcdef-device-cmyk①⑥"></a>

    [device-cmyk()](#funcdef-device-cmyk) function, for specifying uncalibrated colors in an output-device-specific CMYK color space.

<a id="ref-for-funcdef-color-mix①⑤"></a>

In addition the new [color-mix()](#funcdef-color-mix) function allows two colors to be mixed, in a specified color space, to yield a new color.

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

- a
  - [value for lab()](#valdef-lab-a), in § 4.6
  - [value for oklab()](#valdef-oklab-a), in § 4.7
- [absolute color](#absolute-color), in § 2
- [absolute-colorimetric](#valdef-color-profile-rendering-intent-absolute-colorimetric), in § 5.3
- alpha
  - [value for alpha()](#valdef-alpha-alpha), in § 4.10
  - [value for color()](#valdef-color-alpha), in § 5.1
  - [value for hsl()](#valdef-hsl-alpha), in § 4.4
  - [value for hwb()](#valdef-hwb-alpha), in § 4.5
  - [value for lab()](#valdef-lab-alpha), in § 4.6
  - [value for lch()](#valdef-lch-alpha), in § 4.8
  - [value for oklab()](#valdef-oklab-alpha), in § 4.7
  - [value for oklch()](#valdef-oklch-alpha), in § 4.9
  - [value for rgb()](#valdef-rgb-alpha), in § 4.3
- [alpha()](#funcdef-alpha), in § 4.10
- b
  - [value for color()](#valdef-color-b), in § 5.1
  - [value for hwb()](#valdef-hwb-b), in § 4.5
  - [value for lab()](#valdef-lab-b), in § 4.6
  - [value for oklab()](#valdef-oklab-b), in § 4.7
  - [value for rgb()](#valdef-rgb-b), in § 4.3
- c
  - [value for lch()](#valdef-lch-c), in § 4.8
  - [value for oklch()](#valdef-oklch-c), in § 4.9
- [calculate a color-mix()](#calculate-a-color-mix), in § 3.3
- [\<cmyk-component\>](#typedef-cmyk-component), in § 6
- [\<color\>](#typedef-color), in § 2
- [color()](#funcdef-color), in § 5
- [\<color-base\>](#typedef-color-base), in § 2
- [\<color-function\>](#typedef-color-function), in § 2
- [\<color-interpolation-method\>](#color-interpolation-method), in § 9.1
- [color-mix()](#funcdef-color-mix), in § 3
- [@color-profile](#at-ruledef-profile), in § 5.3
- [color profile](#css-color-profile), in § 5.3
- [\<color-space\>](#typedef-color-space), in § 9.1
- [\<colorspace-params\>](#typedef-colorspace-params), in § 5
- [component keyword](#component-keyword), in § 4.2
- components
  - [attribute for CSSColorProfileRule](#dom-csscolorprofilerule-components), in § 12.1
  - [descriptor for @color-profile](#descdef-color-profile-components), in § 5.3
- [contrast-color()](#funcdef-contrast-color), in § 8
- [Conversion, if required](#conversion-if-required), in § 4.1
- [CSS color profile](#css-color-profile), in § 5.3
- [CSSColorProfileRule](#csscolorprofilerule), in § 12.1
- [\<custom-color-space\>](#typedef-custom-color-space), in § 9.1
- [\<custom-params\>](#typedef-custom-params), in § 5
- [device-cmyk()](#funcdef-device-cmyk), in § 6
- [effective percentage](#effective-percentage), in § 11.1
- [fetch an external color profile](#fetch-an-external-color-profile), in § 5.3
- g
  - [value for color()](#valdef-color-g), in § 5.1
  - [value for rgb()](#valdef-rgb-g), in § 4.3
- [gamut-map](#gamut-map), in § 5.3
- h
  - [value for hsl()](#valdef-hsl-h), in § 4.4
  - [value for hwb()](#valdef-hwb-h), in § 4.5
  - [value for lch()](#valdef-lch-h), in § 4.8
  - [value for oklch()](#valdef-oklch-h), in § 4.9
- [\<hue-interpolation-method\>](#typedef-hue-interpolation-method), in § 9.1
- [hwb()](#funcdef-hwb), in § 4.5
- [invalid color](#invalid-color), in § 5
- l
  - [value for hsl()](#valdef-hsl-l), in § 4.4
  - [value for lab()](#valdef-lab-l), in § 4.6
  - [value for lch()](#valdef-lch-l), in § 4.8
  - [value for oklab()](#valdef-oklab-l), in § 4.7
  - [value for oklch()](#valdef-oklch-l), in § 4.9
- [lab()](#funcdef-lab), in § 4.6
- [lch()](#funcdef-lch), in § 4.8
- [\<legacy-device-cmyk-syntax\>](#typedef-legacy-device-cmyk-syntax), in § 6
- [light-dark()](#funcdef-light-dark), in § 7
- [\<light-dark-color\>](#typedef-light-dark-color), in § 7
- [\<light-dark-image\>](#typedef-light-dark-image), in § 7
- [\<modern-device-cmyk-syntax\>](#typedef-modern-device-cmyk-syntax), in § 6
- [\<modern-hsla-syntax\>](#typedef-modern-hsla-syntax), in § 4.4
- [\<modern-hsl-syntax\>](#typedef-modern-hsl-syntax), in § 4.4
- [\<modern-rgba-syntax\>](#typedef-modern-rgba-syntax), in § 4.3
- [\<modern-rgb-syntax\>](#typedef-modern-rgb-syntax), in § 4.3
- [naively converted to CMYK](#naively-convert-from-rgba-to-cmyk), in § 6.1
- [naively converted to RGBA](#naively-convert-from-cmyk-to-rgba), in § 6.1
- [naively convert from CMYK to RGBA](#naively-convert-from-cmyk-to-rgba), in § 6.1
- [naively convert from RGBA to CMYK](#naively-convert-from-rgba-to-cmyk), in § 6.1
- [name](#dom-csscolorprofilerule-name), in § 12.1
- [none](#valdef-light-dark-none), in § 7
- [oklab()](#funcdef-oklab), in § 4.7
- [oklch()](#funcdef-oklch), in § 4.9
- [originally specified color space](#originally-specified-color-space), in § 4.1
- [origin color](#origin-color), in § 4.1
- [out of gamut](#out-of-gamut), in § 5
- [perceptual](#valdef-color-profile-rendering-intent-perceptual), in § 5.3
- [\<polar-color-space\>](#typedef-polar-color-space), in § 9.1
- [\<predefined-rgb\>](#typedef-predefined-rgb), in § 5
- [\<predefined-rgb-params\>](#typedef-predefined-rgb-params), in § 5
- r
  - [value for color()](#valdef-color-r), in § 5.1
  - [value for rgb()](#valdef-rgb-r), in § 4.3
- [\<rectangular-color-space\>](#typedef-rectangular-color-space), in § 9.1
- [relative color](#relative-color), in § 4.1
- [relative-colorimetric](#valdef-color-profile-rendering-intent-relative-colorimetric), in § 5.3
- [relative color processing space](#relative-color-processing-space), in § 4.1
- [rendering-intent](#descdef-color-profile-rendering-intent), in § 5.3
- [renderingIntent](#dom-csscolorprofilerule-renderingintent), in § 12.1
- [resolve to sRGB](#resolve-to-srgb), in § 2
- [s](#valdef-hsl-s), in § 4.4
- [saturation](#valdef-color-profile-rendering-intent-saturation), in § 5.3
- src
  - [attribute for CSSColorProfileRule](#dom-csscolorprofilerule-src), in § 12.1
  - [descriptor for @color-profile](#descdef-color-profile-src), in § 5.3
- [support legacy color syntax](#support-legacy-color-syntax), in § 2
- [valid color](#valid-color), in § 5
- [w](#valdef-hwb-w), in § 4.5
- [x](#valdef-color-x), in § 5.1
- [\<xyz-params\>](#typedef-xyz-params), in § 5
- [y](#valdef-color-y), in § 5.1
- [z](#valdef-color-z), in § 5.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="0948355d"></a>actual value
  - <a id="8c8e51b4"></a>computed value
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="e5db470f"></a>\<alpha-value\>
  - <a id="cee404c5"></a>\<hex-color\>
  - <a id="33347c0e"></a>\<hue\>
  - <a id="8fad2d26"></a>\<named-color\>
  - <a id="024532da"></a>\<system-color\>
  - <a id="640a9100"></a>\<xyz-space\>
  - <a id="d11b143f"></a>a98-rgb
  - <a id="4dee7468"></a>analogous components
  - <a id="0ed7201e"></a>black
  - <a id="a976c737"></a>blue
  - <a id="7398228c"></a>carried forward
  - <a id="b46a2c7d"></a>color
  - <a id="4baa5c4c"></a>color functions
  - <a id="f6f69187"></a>color space
  - <a id="a42c65ac"></a>currentcolor
  - <a id="c990915b"></a>cylindrical polar color
  - <a id="19512a47"></a>darkolivegreen
  - <a id="abed060e"></a>display-p3
  - <a id="1560a0b3"></a>display-p3-linear
  - <a id="bc4e4c18"></a>HSL
  - <a id="5bd3632a"></a>hsl()
  - <a id="239640fd"></a>hsla()
  - <a id="85dbac5b"></a>HWB
  - <a id="4a14e4dc"></a>lab
  - <a id="4b8ab2a8"></a>LCH
  - <a id="e2a9e9b9"></a>legacy color syntax
  - <a id="f8e15dd7"></a>lime
  - <a id="d3bf8831"></a>missing
  - <a id="e269247f"></a>missing color component
  - <a id="77a29323"></a>modern color syntax
  - <a id="01b14142"></a>oklab
  - <a id="404710e3"></a>OkLCh
  - <a id="f0f7ce16"></a>premultiplied
  - <a id="691e902b"></a>prophoto-rgb
  - <a id="2eb59b48"></a>rec2020
  - <a id="af48ee71"></a>rectangular orthogonal color
  - <a id="970f70ea"></a>red
  - <a id="daabf294"></a>rgb()
  - <a id="f3226176"></a>rgba()
  - <a id="ce4cacb8"></a>srgb
  - <a id="e98886a5"></a>srgb-linear
  - <a id="98c10de1"></a>system colors
  - <a id="17858dc9"></a>white
  - <a id="01b00791"></a>xyz
  - <a id="e5457abf"></a>xyz-d50
  - <a id="b3bfee63"></a>xyz-d65
- \[CSS-COLOR-ADJUST-1\] defines the following terms:
  - <a id="b22de20c"></a>color scheme
  - <a id="746d81ae"></a>dark
  - <a id="4318cd08"></a>element color scheme
  - <a id="99387b3d"></a>light
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="35bf32f2"></a>\<image\>
- \[CSS-SYNTAX-3\] defines the following terms:
  - <a id="a81b1fb8"></a>\<declaration-list\>
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="c297b070"></a>\#
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="af4a190d"></a>+
  - <a id="8cd4f032"></a>,
  - <a id="d7e1d67b"></a>\<angle\>
  - <a id="f5b34cad"></a>\<dashed-ident\>
  - <a id="dcecfc13"></a>\<ident\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="128295ac"></a>\<percentage\>
  - <a id="699488a8"></a>\<url\>
  - <a id="d4441b24"></a>?
  - <a id="14d3255d"></a>calc()
  - <a id="4dbf81d6"></a>canonical unit
  - <a id="24ecd148"></a>fetch a style resource
  - <a id="3db7b9e0"></a>math function
  - <a id="d9909296"></a>pi
  - <a id="8cbc2b3b"></a>{A}
  - <a id="4eb9d37e"></a>\|
- \[CSS-VALUES-5\] defines the following terms:
  - <a id="acb3399b"></a>mix item
  - <a id="8093bfd3"></a>normalize mix percentages
- \[CSSOM-1\] defines the following terms:
  - <a id="9d357000"></a>CSSOMString
  - <a id="0f78dbdd"></a>CSSRule
- \[FETCH\] defines the following terms:
  - <a id="ee7bba09"></a>response
- \[INFRA\] defines the following terms:
  - <a id="7f9469b5"></a>ASCII case-insensitive
  - <a id="fca74142"></a>pop
  - <a id="7c3de606"></a>push
  - <a id="ceacaa1c"></a>stack
- \[WEBIDL\] defines the following terms:
  - <a id="889e932f"></a>Exposed

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 6 August 2026. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-color-adjust-1"></a>\[CSS-COLOR-ADJUST-1\]  
Elika Etemad; et al. [CSS Color Adjustment Module Level 1](https://www.w3.org/TR/css-color-adjust-1/). 16 December 2025. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-adjust-1&#x2F;](https://www.w3.org/TR/css-color-adjust-1/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-syntax-3"></a>\[CSS-SYNTAX-3\]  
Tab Atkins Jr.; Simon Sapin. [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/). 24 December 2021. CRD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-syntax-3&#x2F;](https://www.w3.org/TR/css-syntax-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 12 March 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css-values-5"></a>\[CSS-VALUES-5\]  
Tab Atkins Jr.; Elika Etemad; Miriam Suzanne. [CSS Values and Units Module Level 5](https://www.w3.org/TR/css-values-5/). 11 November 2024. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-5&#x2F;](https://www.w3.org/TR/css-values-5/)

<a id="biblio-cssom-1"></a>\[CSSOM-1\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-fetch"></a>\[FETCH\]  
Anne van Kesteren. [Fetch Standard](https://fetch.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;fetch&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://fetch.spec.whatwg.org/)

<a id="biblio-icc"></a>\[ICC\]  
[ICC.1:2022 (Profile version 4.4.0.0)](http://www.color.org/specification/ICC.1-2022-05.pdf). May 2022. URL: [http&#x3A;&#x2F;&#x2F;www&#x2E;color&#x2E;org&#x2F;specification&#x2F;ICC&#x2E;1-2022-05&#x2E;pdf](http://www.color.org/specification/ICC.1-2022-05.pdf)

<a id="biblio-infra"></a>\[INFRA\]  
Anne van Kesteren; Domenic Denicola. [Infra Standard](https://infra.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;infra&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://infra.spec.whatwg.org/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-webidl"></a>\[WEBIDL\]  
Edgar Chen; Timothy Gu. [Web IDL Standard](https://webidl.spec.whatwg.org/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;webidl&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;](https://webidl.spec.whatwg.org/)

### <a id="informative"></a>Non-Normative References

<a id="biblio-fogra39"></a>\[FOGRA39\]  
[ISO 12647-2:2004 / Amd 1, Offset commercial and specialty printing according to ISO 12647-2, paper type 1 or 2 (gloss or matte coated offset, 115 g/m²), screen frequency 60/cm](https://www.color.org/chardata/FOGRA39.xalter). 2006. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;color&#x2E;org&#x2F;chardata&#x2F;FOGRA39&#x2E;xalter](https://www.color.org/chardata/FOGRA39.xalter)

<a id="biblio-fogra51"></a>\[FOGRA51\]  
[ISO 12647-2:2013, Process control for the production of half-tone colour separations, proof and production printsPart 2: Offset lithographic processes, PS 1, premium coated, 115 g/m², moderate substrate fluorescence](https://registry.color.org/cmyk-registry/fogra51). 2015. URL: [https&#x3A;&#x2F;&#x2F;registry&#x2E;color&#x2E;org&#x2F;cmyk-registry&#x2F;fogra51](https://registry.color.org/cmyk-registry/fogra51)

<a id="biblio-fogra55"></a>\[FOGRA55\]  
[CMYKOGV-based gamut exchange space](https://fogra.org/en/research/prepress-technology/multiprimary-printing-13003). 2021. URL: [https&#x3A;&#x2F;&#x2F;fogra&#x2E;org&#x2F;en&#x2F;research&#x2F;prepress-technology&#x2F;multiprimary-printing-13003](https://fogra.org/en/research/prepress-technology/multiprimary-printing-13003)

## <a id="property-index"></a>Property Index

No properties defined.

<a id="ref-for-at-ruledef-profile①⑦"></a>

### <a id="color-profile-descriptor-table"></a>[@color-profile](#at-ruledef-profile) Descriptors

| Name                | Value                                                                      | Initial               |
|---------------------|----------------------------------------------------------------------------|-----------------------|
| <strong><span><a id="ref-for-descdef-color-profile-components①"></a></span><a href="#descdef-color-profile-components">components</a>&#xA;      </strong> | \<ident\>#                                                                 | n/a                   |
| <strong><span><a id="ref-for-descdef-color-profile-rendering-intent①"></a></span><a href="#descdef-color-profile-rendering-intent">rendering-intent</a>&#xA;      </strong> | relative-colorimetric \| absolute-colorimetric \| perceptual \| saturation | relative-colorimetric |
| <strong><span><a id="ref-for-descdef-color-profile-src①"></a></span><a href="#descdef-color-profile-src">src</a>&#xA;      </strong> | \<url\>                                                                    | n/a                   |

## <a id="idl-index"></a>IDL Index

```text
[Exposed=Window]
interface CSSColorProfileRule : CSSRule {
  readonly attribute CSSOMString name ;
  readonly attribute CSSOMString src ;
  readonly attribute CSSOMString renderingIntent ;
  readonly attribute CSSOMString components ;
};

```