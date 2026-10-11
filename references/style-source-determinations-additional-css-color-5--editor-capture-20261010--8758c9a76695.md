Attribution and reformatting notice added for Surgeist on 2026-10-10

This reformatted source capture accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance and representation notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material is copied from [CSS Color Module Level 5](https://drafts.csswg.org/css-color-5/), under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). The captured original copyright, liability, trademark and permissive document-license notice is retained below.

# Source provenance and capture boundary

Retrieved: 2026-10-10 America/New_York (UTC-04:00). Capture time: `2026-10-11T02:37:37.646403+00:00` UTC; this representation reuses the earlier full HTML acquisition. Source status: Editor’s Draft, 8 October 2026.

Full captured HTML SHA-256: `a53d07682aa1de30f93de499e7e29131d6460f393e3559d669ac663075b6b26b` (579722 bytes). Conversion input SHA-256: `8758c9a76695f41e1d2293c971b79d2d4f080d9b4d559b6bbfd8835bd232c8bb` (67358 bytes). Page revision metadata: `50337328484f7fcdd9ab6fe9ded6011815b1707e`.

Retained scope: `relative-color-function`, `at-profile`. Selected heading sections are complete, bounded at the next equal-or-higher heading. The original header and legal notice are retained. Other clauses remain upstream. Figures remain upstream image links. Multiline alternative descriptions use HTML image attributes with numeric whitespace entities to retain exact source alt text.

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

### <a id="relative-color-function"></a>5.1. Relative Color-Function Colors[](#relative-color-function)

Within a <a id="ref-for-relative-color①②"></a>[relative color](https://drafts.csswg.org/css-color-5/#relative-color) syntax <a id="ref-for-funcdef-color⑤"></a>[color()](https://drafts.csswg.org/css-color-5/#funcdef-color) function using <a id="ref-for-typedef-custom-params①"></a>[\<custom-params\>](https://drafts.csswg.org/css-color-5/#typedef-custom-params), the number and name of the allowed <a id="ref-for-component-keyword①②"></a>[component keywords](https://drafts.csswg.org/css-color-5/#component-keyword) are:

- defined by the <a id="ref-for-descdef-color-profile-components"></a>[components](#descdef-color-profile-components) descriptor on the corresponding <a id="ref-for-at-ruledef-profile"></a>[&#64;color-profile](#at-ruledef-profile), if present; otherwise, no relative color manipulation is valid. They are <a id="ref-for-number-value⑤①"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value)s that correspond to the <a id="ref-for-origin-color③⑦"></a>[origin color’s](https://drafts.csswg.org/css-color-5/#origin-color) components after <a id="ref-for-conversion-if-required①⑤"></a>[conversion, if required](https://drafts.csswg.org/css-color-5/#conversion-if-required) to the color space of the color profile. The value 1.0 corresponds to 100%.

Within a <a id="ref-for-relative-color①③"></a>[relative color](https://drafts.csswg.org/css-color-5/#relative-color) syntax <a id="ref-for-funcdef-color⑥"></a>[color()](https://drafts.csswg.org/css-color-5/#funcdef-color) function using <a id="ref-for-typedef-predefined-rgb-params①"></a>[\<predefined-rgb-params\>](https://drafts.csswg.org/css-color-5/#typedef-predefined-rgb-params), the allowed <a id="ref-for-component-keyword①③"></a>[component keywords](https://drafts.csswg.org/css-color-5/#component-keyword) are:

- <a id="valdef-color-r"></a><strong>r</strong>, <a id="valdef-color-g"></a><strong>g</strong>, and <a id="valdef-color-b"></a><strong>b</strong> are all <a id="ref-for-number-value⑤②"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value)s that correspond to the <a id="ref-for-origin-color③⑧"></a>[origin color’s](https://drafts.csswg.org/css-color-5/#origin-color) red, green, and blue components after <a id="ref-for-conversion-if-required①⑥"></a>[conversion, if required](https://drafts.csswg.org/css-color-5/#conversion-if-required) to the predefined RGB color space. The value 1.0 corresponds to 100%.

Within a <a id="ref-for-relative-color①④"></a>[relative color](https://drafts.csswg.org/css-color-5/#relative-color) syntax <a id="ref-for-funcdef-color⑦"></a>[color()](https://drafts.csswg.org/css-color-5/#funcdef-color) function using <a id="ref-for-typedef-xyz-params①"></a>[\<xyz-params\>](https://drafts.csswg.org/css-color-5/#typedef-xyz-params), the allowed <a id="ref-for-component-keyword①④"></a>[component keywords](https://drafts.csswg.org/css-color-5/#component-keyword) are:

- <a id="valdef-color-x"></a><strong>x</strong>, <a id="valdef-color-y"></a><strong>y</strong>, <a id="valdef-color-z"></a><strong>z</strong> are all <a id="ref-for-number-value⑤③"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value)s that correspond to the <a id="ref-for-origin-color③⑨"></a>[origin color’s](https://drafts.csswg.org/css-color-5/#origin-color) X, Y and Z components after <a id="ref-for-conversion-if-required①⑦"></a>[conversion, if required](https://drafts.csswg.org/css-color-5/#conversion-if-required) to relative CIE XYZ color space adapted to the relevant white point. The value 1.0 corresponds to 100%.

Within a <a id="ref-for-relative-color①⑤"></a>[relative color](https://drafts.csswg.org/css-color-5/#relative-color) syntax <a id="ref-for-funcdef-color⑧"></a>[color()](https://drafts.csswg.org/css-color-5/#funcdef-color) function using either <a id="ref-for-typedef-predefined-rgb-params②"></a>[\<predefined-rgb-params\>](https://drafts.csswg.org/css-color-5/#typedef-predefined-rgb-params) or <a id="ref-for-typedef-xyz-params②"></a>[\<xyz-params\>](https://drafts.csswg.org/css-color-5/#typedef-xyz-params), an additional allowed <a id="ref-for-component-keyword①⑤"></a>[component keyword](https://drafts.csswg.org/css-color-5/#component-keyword) is:

- <a id="valdef-color-alpha"></a><strong>alpha</strong> is a <a id="ref-for-number-value⑤④"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value) that corresponds to the <a id="ref-for-origin-color④⓪"></a>[origin color’s](https://drafts.csswg.org/css-color-5/#origin-color) alpha transparency. 1.0 is equivalent to 100%.

The parameters have the following form:

- An <a id="ref-for-typedef-ident"></a>[\<ident\>](https://drafts.csswg.org/css-values-4/#typedef-ident) or <a id="ref-for-typedef-dashed-ident①"></a>[\<dashed-ident\>](https://drafts.csswg.org/css-values-4/#typedef-dashed-ident) denoting the color space. If this is an <a id="ref-for-typedef-ident①"></a>\<ident\> it denotes one of the predefined color spaces [CSS Color 4 §  10. Predefined Color Spaces](https://drafts.csswg.org/css-color-4/#predefined) (such as <a id="ref-for-valdef-color-display-p3"></a>[display-p3](https://drafts.csswg.org/css-color-4/#valdef-color-display-p3)); if it is a <a id="ref-for-typedef-dashed-ident②"></a>\<dashed-ident\> it denotes a custom color space, defined by a <a id="ref-for-at-ruledef-profile①"></a>[&#64;color-profile](#at-ruledef-profile) rule. Individual predefined color spaces may further restrict whether <a id="ref-for-number-value⑤⑤"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value)s or <a id="ref-for-percentage-value②⑤"></a>[\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value)s or both, may be used.

  If the <a id="ref-for-typedef-ident②"></a>[\<ident\>](https://drafts.csswg.org/css-values-4/#typedef-ident) names a non-existent color space (a name that does not match one of the predefined color spaces), or a predefined but unsupported color space, this argument represents an <a id="ref-for-invalid-color①"></a>[invalid color](https://drafts.csswg.org/css-color-5/#invalid-color).

  If the <a id="ref-for-typedef-dashed-ident③"></a>[\<dashed-ident\>](https://drafts.csswg.org/css-values-4/#typedef-dashed-ident) names a non-existent color space ( a name that does not match an <a id="ref-for-css-color-profile"></a>[color profile’s](#css-color-profile) name, or which matches but the corresponding profile has not loaded, or does not represent a valid profile), this argument represents an <a id="ref-for-invalid-color②"></a>[invalid color](https://drafts.csswg.org/css-color-5/#invalid-color).

- One or more <a id="ref-for-number-value⑤⑥"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value)s or <a id="ref-for-percentage-value②⑥"></a>[\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value)s providing the parameter values that the color space takes.

  For custom color spaces, specified component values less than 0 or 0%, or greater than 1 or 100% are not invalid; they are clamped to the valid range at computed value time. This is because ICC profiles typically do not accept out of range input values.

  For custom color spaces, if more <a id="ref-for-number-value⑤⑦"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value)s or <a id="ref-for-percentage-value②⑦"></a>[\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value)s are provided than parameters that the color space takes, the excess <a id="ref-for-number-value⑤⑧"></a>\<number\>s at the end are ignored. The color is still a <a id="ref-for-valid-color③"></a>[valid color](https://drafts.csswg.org/css-color-5/#valid-color).

  For custom color spaces, if more <a id="ref-for-number-value⑤⑨"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value)s or <a id="ref-for-percentage-value②⑧"></a>[\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value)s are provided than components listed in the optional components descriptor, the additional values at the end are still valid, but cannot be used in Relative Color Syntax. The color is still a <a id="ref-for-valid-color④"></a>[valid color](https://drafts.csswg.org/css-color-5/#valid-color).

  For custom color spaces, if fewer <a id="ref-for-number-value⑥⓪"></a>[\<number\>](https://drafts.csswg.org/css-values-4/#number-value)s or <a id="ref-for-percentage-value②⑨"></a>[\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value)s are provided than parameters that the color space takes, the missing parameters default to 0. (This is particularly convenient for multicomponent printers where the additional inks are spot colors or varnishes that most colors on the page won’t use.) The color is still a <a id="ref-for-valid-color⑤"></a>[valid color](https://drafts.csswg.org/css-color-5/#valid-color).

  For predefined color spaces, specified component values less than 0 or 0%, or greater than 1 or 100% are not invalid; these out of gamut colors are gamut mapped to the valid range at computed value time, with a relative colorimetric intent.

- An optional slash-separated <a id="ref-for-typedef-color-alpha-value①①"></a>[\<alpha-value\>](https://drafts.csswg.org/css-color-4/#typedef-color-alpha-value).If omitted, it defaults to 100%.

Tests

- [color-computed-relative-color.html](https://wpt.fyi/results/css/css-color/parsing/color-computed-relative-color.html) [(live test)](http://wpt.live/css/css-color/parsing/color-computed-relative-color.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/parsing/color-computed-relative-color.html)

<a id="ex-RCS-XYZ"></a>

<strong>Example:</strong>

[](#ex-RCS-XYZ) For example, Relative Color Syntax in the CIE XYZ D65 colorspace is used to generate a color which has the same chromaticity but exactly half the luminance of the base color:

``` text
--base:  color(display-p3 0.7 0.5 0.1);
--dark:  color(from var(--base) xyz-d65 calc(x/2) calc(y/2) calc(z/2));
```

The origin color is color(xyz-d65 0.281 0.253 0.044) and so the relative color is color(xyz-d65 0.14 0.126 0.022).

### <a id="at-profile"></a>5.3.  Specifying a Color Profile: the <a id="ref-for-at-ruledef-profile③"></a>[&#64;color-profile](#at-ruledef-profile) at-rule [](#at-profile)

The <a id="at-ruledef-profile"></a><strong>&#64;color-profile</strong> rule defines and names a <a id="css-color-profile"></a><strong>color profile</strong> which can later be used in the <a id="ref-for-funcdef-color⑨"></a>[color()](https://drafts.csswg.org/css-color-5/#funcdef-color) function to specify a color.

It’s defined as:

<a id="ref-for-typedef-dashed-ident⑤"></a><a id="ref-for-comb-one⑧⑤"></a><a id="ref-for-typedef-declaration-list"></a>

``` text
@color-profile = @color-profile [<dashed-ident> | device-cmyk] { <declaration-list> }
```

Tests

- [at-color-profile-001.html](https://wpt.fyi/results/css/css-color/at-color-profile-001.html) [(live test)](http://wpt.live/css/css-color/at-color-profile-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-color/at-color-profile-001.html)

The <a id="ref-for-typedef-dashed-ident⑥"></a>[\<dashed-ident\>](https://drafts.csswg.org/css-values-4/#typedef-dashed-ident) gives the <a id="ref-for-css-color-profile①"></a>[color profile’s](#css-color-profile) name, by which it will be used in a CSS stylesheet. Alternatively, the device-cmyk keyword means that this color profile will, if valid, be used to resolve colors specified in device-cmyk.

The <a id="ref-for-at-ruledef-profile④"></a>[&#64;color-profile](#at-ruledef-profile) rule accepts the descriptors defined in this specification.

| Field                     | Definition                                                                                |
|---------------------------|-------------------------------------------------------------------------------------------|
| <strong>Name:</strong>    | <a id="descdef-color-profile-src"></a><strong>src</strong>                                |
| <strong>For:</strong>     | <a id="ref-for-at-ruledef-profile⑤"></a>[&#64;color-profile](#at-ruledef-profile)         |
| <strong>Value:</strong>   | <a id="ref-for-url-value"></a>[\<url\>](https://drafts.csswg.org/css-values-4/#url-value) |
| <strong>Initial:</strong> | n/a                                                                                       |

The <a id="ref-for-descdef-color-profile-src"></a>[src](#descdef-color-profile-src) descriptor specifies the URL to retrieve the color-profile information from.

If multiple <a id="ref-for-at-ruledef-profile⑥"></a>[&#64;color-profile](#at-ruledef-profile) rules are defined with the same name, the last one in document order wins, and all preceding ones are ignored.

The retrieved ICC profile is valid if

- it can be parsed as an ICC Profile

- it is an Input, Display, Output, or color space ICC profile. (Abstract, DeviceLink, and NamedColor ICC Profiles must not be used).

If the profile is not valid, all CSS colors which reference this profile are <a id="ref-for-invalid-color③"></a>[invalid color](https://drafts.csswg.org/css-color-5/#invalid-color)s.

To <a id="fetch-an-external-color-profile"></a><strong>fetch an external color profile</strong>, given a <a id="ref-for-at-ruledef-profile⑦"></a>[&#64;color-profile](#at-ruledef-profile) rule <var>rule</var>, <a id="ref-for-fetch-a-style-resource"></a>[fetch a style resource](https://drafts.csswg.org/css-values-4/#fetch-a-style-resource) given <var>rule</var>’s URL, with ruleOrDeclaration being <var>rule</var>, destination "color-profile", CORS mode "cors", and processResponse being the following steps given <a id="ref-for-concept-response"></a>[response](https://fetch.spec.whatwg.org/#concept-response) \|/res\| and null, failure or a byte stream <var>byteStream</var>: If <var>byteStream</var> is a byte stream, apply the color profile as parsed from \|byteStream.

Note: The Internet Media Type ("MIME type") for ICC profiles is [application/vnd.iccprofile](https://www.iana.org/assignments/media-types/application/vnd.iccprofile).

| Field                     | Definition                                                                                                                                                                                                                 |
|---------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>    | <a id="descdef-color-profile-rendering-intent"></a><strong>rendering-intent</strong>                                                                                                                                       |
| <strong>For:</strong>     | <a id="ref-for-at-ruledef-profile⑧"></a>[&#64;color-profile](#at-ruledef-profile)                                                                                                                                          |
| <strong>Value:</strong>   | relative-colorimetric <a id="ref-for-comb-one⑧⑥"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) absolute-colorimetric <a id="ref-for-comb-one⑧⑦"></a>\| perceptual <a id="ref-for-comb-one⑧⑧"></a>\| saturation |
| <strong>Initial:</strong> | relative-colorimetric                                                                                                                                                                                                      |

<a id="ref-for-css-color-profile②"></a>[Color profiles](#css-color-profile) contain “rendering intents”, which define how to <a id="gamut-map"></a><strong>gamut-map</strong> their color to smaller gamuts than they’re defined over. Often a profile will contain only a single intent, but when there are multiple, the <a id="ref-for-descdef-color-profile-rendering-intent"></a>[rendering-intent](#descdef-color-profile-rendering-intent) descriptor chooses one of them to use.

The four possible rendering intents are [\[ICC\]](https://drafts.csswg.org/css-color-5/#biblio-icc):

<strong><a id="valdef-color-profile-rendering-intent-relative-colorimetric"></a><strong>relative-colorimetric</strong></strong>

Media-relative colorimetric is required to leave source colors that fall inside the destination medium gamut unchanged relative to the respective media white points. Source colors that are out of the destination medium gamut are mapped to colors on the gamut boundary using a variety of different methods.

The media-relative colorimetric rendering intent is often used with black point compensation, where the source medium black point is mapped to the destination medium black point as well. This method must map the source white point to the destination white point. If black point compensation is in use, the source black point must also be mapped to the destination black point. Adaptation algorithms should be used to adjust for the change in white point. Relative relationships of colors inside both source and destination gamuts should be preserved. Relative relationships of colors outside the destination gamut may be changed.

<strong><a id="valdef-color-profile-rendering-intent-absolute-colorimetric"></a><strong>absolute-colorimetric</strong></strong>

ICC-absolute colorimetric is required to leave source colors that fall inside the destination medium gamut unchanged relative to the adopted white (a perfect reflecting diffuser). Source colors that are out of the destination medium gamut are mapped to colors on the gamut boundary using a variety of different methods. This method produces the most accurate color matching of in-gamut colors, but will result in highlight clipping if the destination medium white point is lower than the source medium white point. For this reason it is recommended for use only in applications that need exact color matching and where highlight clipping is not a concern.

This method MUST disable white point matching and black point matching when converting colors. In general, this option is not recommended except for testing purposes.

<strong><a id="valdef-color-profile-rendering-intent-perceptual"></a><strong>perceptual</strong></strong>

This method is often the preferred choice for images, especially when there are substantial differences between the source and destination (such as a screen display image reproduced on a reflection print). It takes the colors of the source image and re-optimizes the appearance for the destination medium using proprietary methods. This re-optimization may result in colors within both the source and destination gamuts being changed, although perceptual transforms are supposed to maintain the basic artistic intent of the original in the reproduction. They will not attempt to correct errors in the source image.

Note: With v2 ICC profiles there is no specified perceptual reference medium, which can cause interoperability problems. When v2 ICC profiles are used it can be safer to use the media-relative colorimetric rendering intent with black point compensation, instead of the perceptual rendering intent, unless the specific source and destination profiles to be used have been checked to ensure the combination produces the desired result.

This method should maintain relative color values among the pixels as they are mapped to the target device gamut. This method may change pixel values that were originally within the target device gamut, in order to avoid hue shifts and discontinuities and to preserve as much as possible the overall appearance of the scene.

<strong><a id="valdef-color-profile-rendering-intent-saturation"></a><strong>saturation</strong></strong>

This option was created to preserve the relative saturation (chroma) of the original, and to keep solid colors pure. However, it experienced interoperability problems like the perceptual intent, and as solid color preservation is not amenable to a reference medium solution using v4 profiles does not solve the problem. Use of this rendering intent is not recommended unless the specific source and destination profiles to be used have been checked to ensure the combination produces the desired result. This option should preserve the relative saturation (chroma) values of the original pixels. Out of gamut colors should be converted to colors that have the same saturation but fall just inside the gamut.

| Field                     | Definition                                                                                                                                                                                  |
|---------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>    | <a id="descdef-color-profile-components"></a><strong>components</strong>                                                                                                                    |
| <strong>For:</strong>     | <a id="ref-for-at-ruledef-profile⑨"></a>[&#64;color-profile](#at-ruledef-profile)                                                                                                           |
| <strong>Value:</strong>   | <a id="ref-for-typedef-ident③"></a>[\<ident\>](https://drafts.csswg.org/css-values-4/#typedef-ident)<a id="ref-for-mult-comma①"></a>[\#](https://drafts.csswg.org/css-values-4/#mult-comma) |
| <strong>Initial:</strong> | n/a                                                                                                                                                                                         |

Color profiles can define color spaces which contain a varying number of components. For example, a Cyan, Magenta, Yellow and Black (CMYK) profile has four components named c, m, y and k While a four-component additive screen profile might use four components named r, g, y and b.

The value of this descriptor is a comma-separated list of <a id="ref-for-typedef-ident④"></a>[\<ident\>](https://drafts.csswg.org/css-values-4/#typedef-ident) tokens. Each <a id="ref-for-typedef-ident⑤"></a>\<ident\>\> names a component, in the order in which they are used in the color profile, while the total number of tokens defines the number of components.

<a id="ex-components-cmyk"></a>

<strong>Example:</strong>

[](#ex-components-cmyk) This descriptor declares that there are four components named cyan, magenta, yellow and black:

``` text
components: cyan, magenta, yellow, black
```

while this descriptor opts for terser names:

``` text
components: c,m,y,k
```

<a id="ex-components-cmykogv"></a>

<strong>Example:</strong>

[](#ex-components-cmykogv) This descriptor declares that there are seven components named cyan, magenta, yellow, black, orange, green and violet:

``` text
components: cyan, magenta, yellow, black, orange, green, violet
```

If a component is an <a id="ref-for-ascii-case-insensitive"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for <a id="ref-for-valdef-light-dark-none②"></a>[none](https://drafts.csswg.org/css-color-5/#valdef-light-dark-none), the descriptor is invalid, because that would clash with the token for missing values.

If the name chosen for a component clashes with a CSS numeric constant as defined in [CSS Values 4 § 10.7.1 Numeric Constants: e, pi](https://drafts.csswg.org/css-values-4/#calc-constants) the component is still valid, but inside <a id="ref-for-funcdef-calc"></a>[calc()](https://drafts.csswg.org/css-values-4/#funcdef-calc) the component will be shadowed by the numeric constant leading to unexpected results.

<a id="ex-component-pi"></a>

<strong>Example:</strong>

[](#ex-component-pi) This descriptor unwisely calls a component <a id="ref-for-valdef-calc-pi"></a>[pi](https://drafts.csswg.org/css-values-4/#valdef-calc-pi), leading to unexpected results in Relative Color Syntax.

``` text
@color-profile --unwise {
  src: url(https://example.com/unwise);
  components: mi, pi, ni;
}
--base: color(--unwise 35% 20% 8%);
--accent: color(from var(--base) mi calc(pi * 2) calc(ni / 2));
```

Here, the component values of --accent are 35%, 3.14159265358979 \* 2 = 6.28318530717959, 4%.
