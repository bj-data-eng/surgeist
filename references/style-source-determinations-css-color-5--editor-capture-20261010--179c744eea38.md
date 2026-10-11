Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Color Module Level 5](https://drafts.csswg.org/css-color-5/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-10. Source status: Editor’s Draft, 8 October 2026.

Full captured HTML SHA-256: `a53d07682aa1de30f93de499e7e29131d6460f393e3559d669ac663075b6b26b` (579722 bytes). Conversion input SHA-256: `179c744eea387b7cf4fae13432d3f2f816e2ab79ffc711ecdc470c3e2fd67b80` (56214 bytes). Page revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`.

Retained scope: `device-cmyk`, `resolving-device-cmyk-values`, `serializing-device-cmyk-values`. Selected heading sections are complete, bounded at the next equal-or-higher heading. The original header and legal notice are retained. Other clauses remain upstream. Figures remain upstream image links. Multiline alternative descriptions use HTML image attributes with numeric whitespace entities to retain exact source alt text.

Conversion: existing `references/tools/html-to-markdown` preparation, Pandoc 3.1.11.1 and semantic Lua filter; scripts/styles omitted without execution. Original IDs, prose, links, literal blocks and table contents are checked against conversion input. Synthetic table headers and span expansion are representation changes. Mathematical expressions are retained as source text; no equation is corrected. This capture preserves standards latitude and unresolved questions; Surgeist-selected policies are recorded separately in issues.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Color Module Level 5

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 8 October 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-color-5/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-color-5/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2026/WD-css-color-5-20260605/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-color-5)

<strong>Editors:</strong>

[Chris Lilley](https://svgees.us/) (W3C)

[Una Kravets](https://una.im) (Google)

[Lea Verou](https://lea.verou.me/about) (Invited Expert)

<strong>Former Editor:</strong>

[Adam Argyle](https://nerdy.dev) (Google)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-color-5/Overview.bs)

<strong>Delta Spec:</strong>

yes

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-color/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

## <a id="device-cmyk"></a>6.  Uncalibrated CMYK Colors: the <a id="ref-for-funcdef-device-cmyk④"></a>[device-cmyk()](#funcdef-device-cmyk) Function [](#device-cmyk)

Sometimes, when a given printer has not been calibrated, but the output for particular ink combinations is known through experimentation, or via a printed sample swatchbook, it is useful to express CMYK colors in a device-dependent way.

Note: Because the actual resulting color can be unknown, CSS processors might attempt to approximate it. This approximation is likely to be visually very far from the actual printed result.

The <a id="ref-for-funcdef-device-cmyk⑤"></a>[device-cmyk()](#funcdef-device-cmyk) function allows authors to specify a color in this way:

<a id="funcdef-device-cmyk"></a><a id="ref-for-typedef-legacy-device-cmyk-syntax"></a><a id="ref-for-comb-one⑧⑨"></a><a id="ref-for-typedef-modern-device-cmyk-syntax"></a><a id="typedef-legacy-device-cmyk-syntax"></a><a id="ref-for-number-value⑥①"></a><a id="ref-for-mult-comma②"></a><a id="typedef-modern-device-cmyk-syntax"></a><a id="ref-for-typedef-cmyk-component"></a><a id="ref-for-mult-num④"></a><a id="ref-for-typedef-color-alpha-value①②"></a><a id="ref-for-comb-one⑨⓪"></a><a id="ref-for-valdef-light-dark-none③"></a><a id="ref-for-mult-opt②③"></a><a id="typedef-cmyk-component"></a><a id="ref-for-number-value⑥②"></a><a id="ref-for-comb-one⑨①"></a><a id="ref-for-percentage-value③⓪"></a><a id="ref-for-comb-one⑨②"></a><a id="ref-for-valdef-light-dark-none④"></a>

``` text
device-cmyk() = <legacy-device-cmyk-syntax> | <modern-device-cmyk-syntax>
<legacy-device-cmyk-syntax> = device-cmyk( <number>#{4} )
<modern-device-cmyk-syntax> = device-cmyk( <cmyk-component>{4} [ / [ <alpha-value> | none ] ]? )
<cmyk-component> = <number> | <percentage> | none
```

The arguments of the <a id="ref-for-funcdef-device-cmyk⑥"></a>[device-cmyk()](#funcdef-device-cmyk) function specify the cyan, magenta, yellow, and black components, in order, as a number between 0 and 1 or, in the modern syntax, as a percentage between 0% and 100%. These two usages are equivalent, and map to each other linearly. Values less than 0 or 0%, or greater than 1 or 100%, are not invalid; instead, they are clamped to 0/0% or 1/100% at computed-value time.

In the modern syntax, the fifth argument specifies the alpha component of the color. It’s interpreted identically to the fourth argument of the <a id="ref-for-funcdef-rgb⑤"></a>[rgb()](https://drafts.csswg.org/css-color-4/#funcdef-rgb) function. If omitted, it defaults to 100%.

For [historical reasons](https://www.w3.org/TR/2011/WD-css3-gcpm-20111129/#cmyk-colors), <a id="ref-for-funcdef-device-cmyk⑦"></a>[device-cmyk()](#funcdef-device-cmyk) also support a <a id="ref-for-legacy-color-syntax②"></a>[legacy color syntax](https://drafts.csswg.org/css-color-4/#legacy-color-syntax).

Typically, print-based applications will actually store the used colors as CMYK, and send them to the printer in that form. However, such colors do not have a colorimetric interpretation, and thus cannot be used in gradients, compositing, blending and so on.

As such, Device CMYK colors must be converted to an equivalent color. This is not trivial, like the conversion from HSL or HWB to RGB; the precise conversion depends on the precise characteristics of the output device.

1.  If the user, author, or user-agent stylesheet has an <a id="ref-for-at-ruledef-profile①①"></a>[&#64;color-profile](https://drafts.csswg.org/css-color-5/#at-ruledef-profile) definition for device-cmyk, and the resource specified by the src descriptor can be retrieved, and the resource is a valid CMYK ICC profile, and the user agent can process ICC profiles, the computed value of the <a id="ref-for-funcdef-device-cmyk⑧"></a>[device-cmyk()](#funcdef-device-cmyk) function must be the Lab value of the CMYK color.
2.  Otherwise, the computed value of the <a id="ref-for-funcdef-device-cmyk⑨"></a>[device-cmyk()](#funcdef-device-cmyk) function must be the sRGB value of the CMYK color, as converted with the following naive conversion algorithm.

<a id="ex-device-cmyk-naive"></a>

<strong>Example:</strong>

[](#ex-device-cmyk-naive) For example, with no <a id="ref-for-at-ruledef-profile①②"></a>[&#64;color-profile](https://drafts.csswg.org/css-color-5/#at-ruledef-profile), the following colors are equivalent, using the naive conversion.

``` text
color:  device-cmyk(0 81% 81% 30%);
color:  rgb(178 34 34);
color:  firebrick;
```

<a id="ex-device-cmyk-colprof"></a>

<strong>Example:</strong>

[](#ex-device-cmyk-colprof) With the <a id="ref-for-at-ruledef-profile①③"></a>[&#64;color-profile](https://drafts.csswg.org/css-color-5/#at-ruledef-profile) specified as in the example stylesheet, the following colors are equivalent, using colorimetric conversion.

``` text
color:  device-cmyk(0 81% 81% 30%);
color:  lab(45.060% 45.477 35.459)
color:  rgb(70.690% 26.851% 19.724%);
```

The naive conversion is necessarily approximate, since it has no knowledge of the colorimetry of the inks, the dot gain, the colorimetry of the RGB space, and so on.

<a id="fig-naive-cmyk"></a>

<figure>
<p>A grid of colored squares. There are six columns, labelled A to F, and four rows, labelled 1 to 4.</p>
<figcaption><p>A color checker, used for ensuring color fidelity in the print and photographic industries. Averaged measured Lab values are available for each patch. The rectangles show the Lab values, converted to sRGB. The circles show the Lab values, passed through an ICC profile to convert them to CMYK. The CMYK value are then naively converted to sRGB.</p>
<p>The table below shows, for each patch, the DeltaE 2000 between the original Lab and the Lab value after round-tripping through CMYK. A DeltaE 2000 of 1 or more is just visible, while 5 or more is just a different color altogether.</p>
<table>
<thead>
<tr class="header">
<th>Column 1</th>
<th>Column 2</th>
<th>Column 3</th>
<th>Column 4</th>
<th>Column 5</th>
<th>Column 6</th>
<th>Column 7</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<th><strong></strong></th>
<th><strong>A</strong></th>
<th><strong>B</strong></th>
<th><strong>C</strong></th>
<th><strong>D</strong></th>
<th><strong>E</strong></th>
<th><strong>F</strong></th>
</tr>

<tr class="odd">
<th><strong>1</strong></th>
<td>11.33</td>
<td>9.36</td>
<td>5.66</td>
<td>7.52</td>
<td>12.39</td>
<td>21.58</td>
</tr>
<tr class="even">
<th><strong>2</strong></th>
<td>6.40</td>
<td>8.79</td>
<td>11.77</td>
<td>17.16</td>
<td>11.91</td>
<td>3.97</td>
</tr>
<tr class="odd">
<th><strong>3</strong></th>
<td>12.1</td>
<td>17.00</td>
<td>3.38</td>
<td>1.94</td>
<td>18.08</td>
<td>14.97</td>
</tr>
<tr class="even">
<th><strong>4</strong></th>
<td>1.89</td>
<td>6.56</td>
<td>7.85</td>
<td>8.76</td>
<td>9.82</td>
<td>10.29</td>
</tr>
</tbody>
</table></figcaption>
</figure>

### <a id="cmyk-rgb"></a>6.1.  Naively Converting Between Uncalibrated CMYK and sRGB-Based Color [](#cmyk-rgb)

To <a id="naively-convert-from-cmyk-to-rgba"></a><strong>naively convert from CMYK to RGBA</strong>:

- <code>red = 1 - min(1, cyan &#42; (1 - black) + black)</code>
- <code>green = 1 - min(1, magenta &#42; (1 - black) + black)</code>
- <code>blue = 1 - min(1, yellow &#42; (1 - black) + black)</code>
- Alpha is same as for input color.

To <a id="naively-convert-from-rgba-to-cmyk"></a><strong>naively convert from RGBA to CMYK</strong>:

- <code>black = 1 - max(red, green, blue)</code>
- <code>cyan = (1 - red - black) / (1 - black), or 0 if black is 1</code>
- <code>magenta = (1 - green - black) / (1 - black), or 0 if black is 1</code>
- <code>yellow = (1 - blue - black) / (1 - black), or 0 if black is 1</code>
- alpha is the same as the input color

### <a id="resolving-device-cmyk-values"></a>10.3.  Resolving device-cmyk Values [](#resolving-device-cmyk-values)

The computed and used value is the specified device-specific CMYK color, (with components as <a id="ref-for-number-value⑥③"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value), not <a id="ref-for-percentage-value③①"></a>[\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value)) paired with the specified alpha component (as a <a id="ref-for-number-value⑥④"></a>\<number\>, not a <a id="ref-for-percentage-value③②"></a>\<percentage\>; and defaulting to opaque if unspecified).

The <a id="ref-for-actual-value"></a>[actual value](https://drafts.csswg.org/css-cascade-5/#actual-value) can vary based on the operation; for rendering to a CMYK-capable device, it may be rendered as a CMYK color; for blending with non-CMYK colors or rendering to a non-CMYK device, it must be converted as specified in [§ 6 Uncalibrated CMYK Colors: the device-cmyk() Function](#device-cmyk).

<a id="ex-device-cmyk-used"></a>

<strong>Example:</strong>

[](#ex-device-cmyk-used) For example,

``` text
 device-cmyk(0% 70% 20% 0%)
```

has the specified and actual value

``` text
 device-cmyk(0 0.7 0.2 0)
```

and will, if the implementation understands ICC profiles and has an appropriate profile installed, have the used value

``` text
 lab(63.673% 51.577 5.811)
```

Note: As with all colors, the used value is not available to script.

### <a id="serializing-device-cmyk-values"></a>11.5.  Serializing device-cmyk Values [](#serializing-device-cmyk-values)

The serialized form of <a id="ref-for-funcdef-device-cmyk①①"></a>[device-cmyk()](#funcdef-device-cmyk) values is derived from the <a id="ref-for-computed-value"></a>[computed value](https://drafts.csswg.org/css-cascade-5/#computed-value) and uses the <a id="ref-for-funcdef-device-cmyk①②"></a>device-cmyk() form, with lowercase letters for the function name.

The component values are serialized in base 10, as <a id="ref-for-number-value⑥⑤"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value). A single ASCII space character " " must be used as the separator between the component values.

Trailing fractional zeroes in any component values must be omitted; if the fractional part consists of all zeroes, the decimal point must also be omitted.

<a id="ex-device-cmyk-serial"></a>

<strong>Example:</strong>

[](#ex-device-cmyk-serial)

The serialized value of the color

``` text
  device-cmyk(0 81% 81% 30%)
```

is the string "device-cmyk(0 0.81 0.81 0.3)"

The precision with which <a id="ref-for-funcdef-device-cmyk①③"></a>[device-cmyk()](#funcdef-device-cmyk) component values are retained, and thus the number of significant figures in the serialized value, is not defined in this specification, but must at least be sufficient to round-trip values with eight bit precision; this will result in at least two decimal places unless trailing zeroes have been omitted. Values must be [rounded towards +∞](https://drafts.csswg.org/css-values-4/#combine-integers), not truncated.

Unitary alpha values are not explicitly serialized. Non-unitary alpha values must be explicitly serialized, and the string " / " (an ASCII space, then forward slash, then another space) must be used to separate the black ("k") color component value from the alpha value.
