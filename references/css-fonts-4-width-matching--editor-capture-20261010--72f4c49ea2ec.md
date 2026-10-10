Attribution and reformatting notice added for Surgeist on 2026-10-10

This bounded reformatted excerpt accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added scope and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Fonts Module Level 4](https://drafts.csswg.org/css-fonts-4/). Original copyright notice: Copyright © 2026 World Wide Web Consortium. The original legal notice and its links are retained below. License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt).

# Source provenance and bounded scope

Source: https://drafts.csswg.org/css-fonts-4/

Retrieved: 2026-10-10. Source status: Editor’s Draft, 13 September 2026.

Original complete HTML SHA-256: `72f4c49ea2ec6b3e417f2c3cb8e140d4106ffba9f2610b9b74219231424dddb7` (1489252 bytes). Declared page revision: `4f200bd6e3bd48ea9fb923b4f98f92a9bbfbb04c`. The date and byte hash identify this captured editor rendering; the live URL can change.

Scope retained: complete sections 2.3 (including 2.3.1) and 5.2, plus document head/status/legal notice. All other document sections, bibliography, indexes, and appendices are excluded. Links to excluded fragments resolve to the authoritative upstream URL. This is a bounded excerpt, not a complete edition. The exact original HTML is retained as local acquisition evidence; the serialized bounded HTML SHA-256 is `8a79b4280f16691422941c1acb19cd0520f8a3db8a621ce377a2c7df31ed99c8`.

Representation changes: HTML to GFM conversion using Pandoc 3.1.11.1 and the repository's [conversion tools](tools/html-to-markdown/README.md). Scripts and styles are omitted without execution. Retained IDs, ordinary prose, headings, links, literal examples, and table cells are verified against the bounded HTML. Spanning table values are expanded only to their applicable columns; added table headings and visible semantic labels are non-normative. This excerpt contains 2 tables.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>CSS Fonts Module Level 4

<a id="w3c-state"></a>[Editor’s Draft](https://www.w3.org/standards/types/#ED), 13 September 2026

More details about this document

<strong>This version:</strong>

<https://drafts.csswg.org/css-fonts-4/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/css-fonts-4/>

<strong>Previous Versions:</strong>

<https://www.w3.org/TR/2024/WD-css-fonts-4-20240201/>

<strong>Feedback:</strong>

[CSSWG Issues Repository](https://github.com/w3c/csswg-drafts/labels/css-fonts-4)

[Inline In Spec](https://drafts.csswg.org/css-fonts-4/#issues-index)

<strong>Editor:</strong>

[Chris Lilley](http://svgees.us) (W3C)

<strong>Former Editors:</strong>

[John Daggett](https://twitter.com/nattokirai) (Invited Expert)

[Myles C. Maxfield](mailto:mmaxfield@apple.com) (Formerly of Apple Inc.)

<strong>Suggest an Edit for this Spec:</strong>

[GitHub Editor](https://github.com/w3c/csswg-drafts/blob/main/css-fonts-4/Overview.bs)

<strong>Test Suite:</strong>

<https://wpt.fyi/results/css/css-fonts/>

[Copyright](https://www.w3.org/policies/#copyright) © 2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

------------------------------------------------------------------------

### <a id="font-width-prop"></a>2.3.  Font width: the <a id="ref-for-propdef-font-width①"></a>[font-width](#propdef-font-width) property[](#font-width-prop)

| Field                                                                                    | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
|------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:</strong>                                                                   | <a id="propdef-font-width"></a><strong>font-width</strong>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <strong>[Value:](https://www.w3.org/TR/css-values/#value-defs)</strong>                  | normal <a id="ref-for-comb-one②⑤"></a>[\|](https://drafts.csswg.org/css-values-4/#comb-one) <a id="ref-for-percentage-value"></a>[\<percentage \[0,∞\]\>](https://drafts.csswg.org/css-values-4/#percentage-value) <a id="ref-for-comb-one②⑥"></a>\| ultra-condensed <a id="ref-for-comb-one②⑦"></a>\| extra-condensed <a id="ref-for-comb-one②⑧"></a>\| condensed <a id="ref-for-comb-one②⑨"></a>\| semi-condensed <a id="ref-for-comb-one③⓪"></a>\| semi-expanded <a id="ref-for-comb-one③①"></a>\| expanded <a id="ref-for-comb-one③②"></a>\| extra-expanded <a id="ref-for-comb-one③③"></a>\| ultra-expanded |
| <strong>[Initial:](https://www.w3.org/TR/css-cascade/#initial-values)</strong>           | normal                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong>[Applies to:](https://www.w3.org/TR/css-cascade/#applies-to)</strong>            | all elements and text                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <strong>[Inherited:](https://www.w3.org/TR/css-cascade/#inherited-property)</strong>     | yes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong>[Percentages:](https://www.w3.org/TR/css-values/#percentages)</strong>           | Not resolved                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <strong>[Computed value:](https://www.w3.org/TR/css-cascade/#computed)</strong>          | a percentage, see below                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong>[Canonical order:](https://www.w3.org/TR/cssom/#serializing-css-values)</strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong>[Animation type:](https://www.w3.org/TR/web-animations/#animation-type)</strong> | by computed value type                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |

Tests

- [font-stretch-01.html](https://wpt.fyi/results/css/css-fonts/font-stretch-01.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-01.html)
- [font-stretch-02.html](https://wpt.fyi/results/css/css-fonts/font-stretch-02.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-02.html)
- [font-stretch-03.html](https://wpt.fyi/results/css/css-fonts/font-stretch-03.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-03.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-03.html)
- [font-stretch-04.html](https://wpt.fyi/results/css/css-fonts/font-stretch-04.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-04.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-04.html)
- [font-stretch-05.html](https://wpt.fyi/results/css/css-fonts/font-stretch-05.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-05.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-05.html)
- [font-stretch-06.html](https://wpt.fyi/results/css/css-fonts/font-stretch-06.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-06.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-06.html)
- [font-stretch-07.html](https://wpt.fyi/results/css/css-fonts/font-stretch-07.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-07.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-07.html)
- [font-stretch-08.html](https://wpt.fyi/results/css/css-fonts/font-stretch-08.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-08.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-08.html)
- [font-stretch-09.html](https://wpt.fyi/results/css/css-fonts/font-stretch-09.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-09.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-09.html)
- [font-stretch-10.html](https://wpt.fyi/results/css/css-fonts/font-stretch-10.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-10.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-10.html)
- [font-stretch-11.html](https://wpt.fyi/results/css/css-fonts/font-stretch-11.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-11.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-11.html)
- [font-stretch-12.html](https://wpt.fyi/results/css/css-fonts/font-stretch-12.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-12.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-12.html)
- [font-stretch-13.html](https://wpt.fyi/results/css/css-fonts/font-stretch-13.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-13.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-13.html)
- [font-stretch-14.html](https://wpt.fyi/results/css/css-fonts/font-stretch-14.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-14.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-14.html)
- [font-stretch-15.html](https://wpt.fyi/results/css/css-fonts/font-stretch-15.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-15.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-15.html)
- [font-stretch-16.html](https://wpt.fyi/results/css/css-fonts/font-stretch-16.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-16.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-16.html)
- [font-stretch-17.html](https://wpt.fyi/results/css/css-fonts/font-stretch-17.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-17.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-17.html)
- [font-stretch-18.html](https://wpt.fyi/results/css/css-fonts/font-stretch-18.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-18.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-18.html)
- [font-stretch-interpolation.html](https://wpt.fyi/results/css/css-fonts/animations/font-stretch-interpolation.html) [(live test)](http://wpt.live/css/css-fonts/animations/font-stretch-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/animations/font-stretch-interpolation.html)
- [font-width-computed.html](https://wpt.fyi/results/css/css-fonts/parsing/font-width-computed.html) [(live test)](http://wpt.live/css/css-fonts/parsing/font-width-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/parsing/font-width-computed.html)
- [font-width-invalid.html](https://wpt.fyi/results/css/css-fonts/parsing/font-width-invalid.html) [(live test)](http://wpt.live/css/css-fonts/parsing/font-width-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/parsing/font-width-invalid.html)
- [font-width-valid.html](https://wpt.fyi/results/css/css-fonts/parsing/font-width-valid.html) [(live test)](http://wpt.live/css/css-fonts/parsing/font-width-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/parsing/font-width-valid.html)
- [font-parse-numeric-stretch-style-weight.html](https://wpt.fyi/results/css/css-fonts/variations/font-parse-numeric-stretch-style-weight.html) [(live test)](http://wpt.live/css/css-fonts/variations/font-parse-numeric-stretch-style-weight.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/variations/font-parse-numeric-stretch-style-weight.html)
- [font-stretch.html](https://wpt.fyi/results/css/css-fonts/variations/font-stretch.html) [(live test)](http://wpt.live/css/css-fonts/variations/font-stretch.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/variations/font-stretch.html)

The <a id="ref-for-propdef-font-width②"></a>[font-width](#propdef-font-width) property selects a normal, condensed, or expanded face from a font family. Values are specified either as percentages or as keywords which map to a percentage as defined in the following table:

<a id="widthmappings"></a>

| Absolute keyword value                                                                          | Numeric value |
|-------------------------------------------------------------------------------------------------|---------------|
| <strong><a id="valdef-font-width-ultra-condensed"></a><strong>ultra-condensed</strong></strong> | 50%           |
| <strong><a id="valdef-font-width-extra-condensed"></a><strong>extra-condensed</strong></strong> | 62.5%         |
| <strong><a id="valdef-font-width-condensed"></a><strong>condensed</strong></strong>             | 75%           |
| <strong><a id="valdef-font-width-semi-condensed"></a><strong>semi-condensed</strong></strong>   | 87.5%         |
| <strong><a id="valdef-font-width-normal"></a><strong>normal</strong></strong>                   | 100%          |
| <strong><a id="valdef-font-width-semi-expanded"></a><strong>semi-expanded</strong></strong>     | 112.5%        |
| <strong><a id="valdef-font-width-expanded"></a><strong>expanded</strong></strong>               | 125%          |
| <strong><a id="valdef-font-width-extra-expanded"></a><strong>extra-expanded</strong></strong>   | 150%          |
| <strong><a id="valdef-font-width-ultra-expanded"></a><strong>ultra-expanded</strong></strong>   | 200%          |

<a id="valdef-font-width-percentage-0"></a><strong><a id="ref-for-percentage-value①"></a>[\<percentage \[0,∞\]\>](https://drafts.csswg.org/css-values-4/#percentage-value)</strong> values represent the fractional width of the glyphs, with 100% representing “normal” glyph widths (as defined by the font designer). Values less than 0% are <a id="ref-for-css-invalid①"></a>[invalid](https://drafts.csswg.org/css-syntax-3/#css-invalid).

When a face does not exist for a given width, values less than 100% map to a narrower face if one exists, otherwise a wider face. Conversely, values greater than or equal to 100% map to a wider face if one exists, otherwise a narrower face. Some fonts might support a range of width values; if the requested width value is not available in the font, the closest supported value is used, using the same mapping rules (see the [§ 5 Font Matching Algorithm](https://drafts.csswg.org/css-fonts-4/#font-matching-algorithm) for the precise algorithm). For TrueType / OpenType fonts that support variations, the <code>wdth</code> variation is used to implement varying widths.

<a id="ex-font-width-matching"></a>

<strong>Example:</strong>

[](#ex-font-width-matching) The figure below shows how nine font-width property settings affect font matching for a font family containing a variety of discrete widths. Grey indicates a width for which no face exists and a different width is substituted:

<figure>
<img src="https://drafts.csswg.org/css-fonts-4/images/universwidths.png" alt="width mappings for a family with condensed, normal and expanded faces" />
<figcaption>Width mappings for a font family with condensed, normal and expanded width faces</figcaption>
</figure>

<code><a id="ref-for-dom-window-getcomputedstyle"></a>[getComputedStyle()](https://drafts.csswg.org/cssom-1/#dom-window-getcomputedstyle)</code> always serializes its value as a <a id="ref-for-percentage-value②"></a>[\<percentage\>](https://drafts.csswg.org/css-values-4/#percentage-value), regardless of how the value was specified by the author, or whether or not a keyword happens to map to the value.

#### <a id="font-stretch-prop"></a>2.3.1.  Font width: the <a id="ref-for-propdef-font-stretch"></a>[font-stretch](#propdef-font-stretch) legacy name alias[](#font-stretch-prop)

For historical reasons, a <a id="propdef-font-stretch"></a><strong>font-stretch</strong> property exists which is a <a id="ref-for-legacy-name-alias"></a>[legacy name alias](https://drafts.csswg.org/css-cascade-5/#legacy-name-alias) and functions in the identical way to the <a id="ref-for-propdef-font-width③"></a>[font-width](#propdef-font-width).

<a id="ex-font-stretch-set"></a>

<strong>Example:</strong>

[](#ex-font-stretch-set) For example, here the legacy <a id="ref-for-propdef-font-stretch①"></a>[font-stretch](#propdef-font-stretch) is used on level one headings.

``` text
h1 {font-stretch: condensed; }
```

The specified value of the <a id="ref-for-propdef-font-width④"></a>[font-width](#propdef-font-width) on those headings becomes set to <a id="ref-for-valdef-font-width-condensed"></a>[condensed](#valdef-font-width-condensed)'.

<a id="ex-font-width-set"></a>

<strong>Example:</strong>

[](#ex-font-width-set) For example, here the <a id="ref-for-propdef-font-width⑤"></a>[font-width](#propdef-font-width) is used on level one headings.

``` text
h1 {font-width: condensed; }
```

The specified value of the <a id="ref-for-propdef-font-stretch②"></a>[font-stretch](#propdef-font-stretch) on those headings becomes set to <a id="ref-for-valdef-font-width-condensed①"></a>[condensed](#valdef-font-width-condensed).

Tests

- [font-stretch-01.html](https://wpt.fyi/results/css/css-fonts/font-stretch-01.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-01.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-01.html)
- [font-stretch-02.html](https://wpt.fyi/results/css/css-fonts/font-stretch-02.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-02.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-02.html)
- [font-stretch-03.html](https://wpt.fyi/results/css/css-fonts/font-stretch-03.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-03.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-03.html)
- [font-stretch-04.html](https://wpt.fyi/results/css/css-fonts/font-stretch-04.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-04.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-04.html)
- [font-stretch-05.html](https://wpt.fyi/results/css/css-fonts/font-stretch-05.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-05.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-05.html)
- [font-stretch-06.html](https://wpt.fyi/results/css/css-fonts/font-stretch-06.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-06.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-06.html)
- [font-stretch-07.html](https://wpt.fyi/results/css/css-fonts/font-stretch-07.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-07.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-07.html)
- [font-stretch-08.html](https://wpt.fyi/results/css/css-fonts/font-stretch-08.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-08.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-08.html)
- [font-stretch-09.html](https://wpt.fyi/results/css/css-fonts/font-stretch-09.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-09.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-09.html)
- [font-stretch-10.html](https://wpt.fyi/results/css/css-fonts/font-stretch-10.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-10.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-10.html)
- [font-stretch-11.html](https://wpt.fyi/results/css/css-fonts/font-stretch-11.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-11.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-11.html)
- [font-stretch-12.html](https://wpt.fyi/results/css/css-fonts/font-stretch-12.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-12.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-12.html)
- [font-stretch-13.html](https://wpt.fyi/results/css/css-fonts/font-stretch-13.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-13.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-13.html)
- [font-stretch-14.html](https://wpt.fyi/results/css/css-fonts/font-stretch-14.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-14.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-14.html)
- [font-stretch-15.html](https://wpt.fyi/results/css/css-fonts/font-stretch-15.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-15.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-15.html)
- [font-stretch-16.html](https://wpt.fyi/results/css/css-fonts/font-stretch-16.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-16.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-16.html)
- [font-stretch-17.html](https://wpt.fyi/results/css/css-fonts/font-stretch-17.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-17.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-17.html)
- [font-stretch-18.html](https://wpt.fyi/results/css/css-fonts/font-stretch-18.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-18.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-18.html)
- [font-stretch-interpolation-math-functions.html](https://wpt.fyi/results/css/css-fonts/font-stretch-interpolation-math-functions.html) [(live test)](http://wpt.live/css/css-fonts/font-stretch-interpolation-math-functions.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/font-stretch-interpolation-math-functions.html)
- [font-stretch-interpolation.html](https://wpt.fyi/results/css/css-fonts/animations/font-stretch-interpolation.html) [(live test)](http://wpt.live/css/css-fonts/animations/font-stretch-interpolation.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/animations/font-stretch-interpolation.html)
- [font-width-computed.html](https://wpt.fyi/results/css/css-fonts/parsing/font-width-computed.html) [(live test)](http://wpt.live/css/css-fonts/parsing/font-width-computed.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/parsing/font-width-computed.html)
- [font-width-invalid.html](https://wpt.fyi/results/css/css-fonts/parsing/font-width-invalid.html) [(live test)](http://wpt.live/css/css-fonts/parsing/font-width-invalid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/parsing/font-width-invalid.html)
- [font-width-valid.html](https://wpt.fyi/results/css/css-fonts/parsing/font-width-valid.html) [(live test)](http://wpt.live/css/css-fonts/parsing/font-width-valid.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/parsing/font-width-valid.html)
- [font-parse-numeric-stretch-style-weight.html](https://wpt.fyi/results/css/css-fonts/variations/font-parse-numeric-stretch-style-weight.html) [(live test)](http://wpt.live/css/css-fonts/variations/font-parse-numeric-stretch-style-weight.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/variations/font-parse-numeric-stretch-style-weight.html)
- [font-stretch.html](https://wpt.fyi/results/css/css-fonts/variations/font-stretch.html) [(live test)](http://wpt.live/css/css-fonts/variations/font-stretch.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/variations/font-stretch.html)

User agents must not synthesize condensed or expanded faces for font families which lack such faces and which do not have a width variation axis. In particular, user agents must not geometrically stretch such faces.

### <a id="font-style-matching"></a>5.2. Matching font styles[](#font-style-matching)

The procedure for choosing a font for a given character in a run of text consists of iterating over the font families named by the <a id="ref-for-propdef-font-family①①"></a>[font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) property, selecting a font face with the appropriate style based on other font properties and then determining whether a glyph exists for the given character. This is done using the <a id="character-map"></a><strong>character map</strong> of the font, data which maps characters to the default glyph for that character. A font is considered to <a id="support"></a><strong>support</strong> a given character if (1) the character is contained in the font’s <a id="ref-for-character-map③"></a>[character map](#character-map) and (2) if required by the containing script, shaping information is available for that character.

Some legacy fonts might include a given character in the <a id="ref-for-character-map④"></a>[character map](#character-map) but lack the shaping information (e.g. [OpenType layout tables](https://www.microsoft.com/typography/otspec/ttochap1.htm) or [Graphite tables](https://scripts.sil.org/cms/scripts/page.php?site_id=projects&item_id=graphite_techAbout)) necessary for correctly rendering text runs containing that character.

Codepoint sequences consisting of a base character followed by a sequence of combining characters are treated slightly differently, see the section on [cluster matching](https://drafts.csswg.org/css-fonts-4/#cluster-matching) below.

For this procedure, the <a id="default-face"></a><strong>default font face</strong> for a given font family is defined to be the face that would be selected if all font style properties were set to their initial value.

<a id="fontmatchingalg"></a>

1.  Using the computed font property values for a given element, the user agent starts with the first family name specified by the <a id="ref-for-propdef-font-family①②"></a>[font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) property.

2.  If the family name is a generic family keyword, the user agent looks up the appropriate font family name to be used. User agents may choose the generic font family to use based on the language of the containing element or the Unicode range of the character.

3.  For other family names, the user agent attempts to find the family name among fonts defined via <a id="ref-for-at-font-face-rule⑤⑥"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-4/#at-font-face-rule) rules and then among available installed fonts (this may include font aliases), matching names with a [§ 5.1 Localized name matching](https://drafts.csswg.org/css-fonts-4/#localized-name-matching) as outlined in the section above. If the font resources defined for a given face in an <a id="ref-for-at-font-face-rule⑤⑦"></a>&#64;font-face rule are either not available or contain invalid font data, then the face should be treated as not present in the family. If no faces are present for a family defined via <a id="ref-for-at-font-face-rule⑤⑧"></a>&#64;font-face rules, the family should be treated as missing; matching a platform font with the same name must not occur in this case.

4.  If a font family match occurs, the user agent assembles the set of font faces in that family and then narrows the set to a single face using other font properties in the order given below. Fonts might be present in this group which can support a range of <a id="ref-for-propdef-font-width①⑧"></a>[font-width](#propdef-font-width), <a id="ref-for-propdef-font-style①③"></a>[font-style](https://drafts.csswg.org/css-fonts-4/#propdef-font-style), or <a id="ref-for-propdef-font-weight①⑨"></a>[font-weight](https://drafts.csswg.org/css-fonts-4/#propdef-font-weight) properties. In this case, the algorithm proceeds as if each supported combination of values is a unique font in the set. If such a font is ultimately selected by this algorithm, particular values for <a id="ref-for-propdef-font-width①⑨"></a>font-width, <a id="ref-for-propdef-font-style①④"></a>font-style, and <a id="ref-for-propdef-font-weight②⓪"></a>font-weight must be applied before any layout or rendering occurs. The application of these values must be applied in the [Apply font matching variations](https://drafts.csswg.org/css-fonts-4/#apply-font-matching-variations) step detailed in [§ 7 Font Feature and Variation Resolution](https://drafts.csswg.org/css-fonts-4/#font-feature-variation-resolution). A group of faces defined via <a id="ref-for-at-font-face-rule⑤⑨"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-4/#at-font-face-rule) rules with identical font descriptor values but differing <a id="ref-for-descdef-font-face-unicode-range④"></a>[unicode-range](https://drafts.csswg.org/css-fonts-4/#descdef-font-face-unicode-range) values are considered to be a single <a id="composite-face"></a><strong>composite font face</strong> for this step:

    Tests

    - [fixed-stretch-style-over-weight.html](https://wpt.fyi/results/css/css-fonts/matching/fixed-stretch-style-over-weight.html) [(live test)](http://wpt.live/css/css-fonts/matching/fixed-stretch-style-over-weight.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/matching/fixed-stretch-style-over-weight.html)
    - [stretch-distance-over-weight-distance.html](https://wpt.fyi/results/css/css-fonts/matching/stretch-distance-over-weight-distance.html) [(live test)](http://wpt.live/css/css-fonts/matching/stretch-distance-over-weight-distance.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/matching/stretch-distance-over-weight-distance.html)
    - [style-ranges-over-weight-direction.html](https://wpt.fyi/results/css/css-fonts/matching/style-ranges-over-weight-direction.html) [(live test)](http://wpt.live/css/css-fonts/matching/style-ranges-over-weight-direction.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/matching/style-ranges-over-weight-direction.html)

    <a id="fontstylematchingalg"></a>

    1.  <a id="ref-for-propdef-font-width②⓪"></a>[font-width](#propdef-font-width) is tried first. If a font does not have any concept of varying strengths of width values, its width value is mapped according table in the [property definition](#widthmappings). If the matching set includes faces with width values containing the <a id="ref-for-propdef-font-width②①"></a>font-width desired value, faces with width values which do not include the desired width value are removed from the matching set. If there is no face which contains the desired value, a width value is chosen using the rules below:

        - If the desired width value is less than or equal to 100%, width values below the desired width value are checked in descending order followed by width values above the desired width value in ascending order until a match is found.

        - Otherwise, width values above the desired width value are checked in ascending order followed by width values below the desired width value in descending order until a match is found.

        Once the closest matching width has been determined by this process, faces with widths which do not include this determined width are removed from the matching set.

        <a id="ex-ascending-stretch"></a>
        <strong>Example:</strong>

        [](#ex-ascending-stretch) This search algorithm can be thought of as a distance function, where the lowest-distance value present in the font family is selected, and all fonts not including that value are eliminated.
        Consider a font family with three fonts, named A, B, and C, each with associated supported ranges for the <a id="ref-for-propdef-font-width②②"></a>[font-width](#propdef-font-width) descriptor. If an element is styled with "font-width: 125%", the search algorithm can be visualized as follows:

        ![algorithm](https://drafts.csswg.org/css-fonts-4/images/stretchdistance.svg)

        The font width ranges supported by fonts A, B, and C are shown in the graph above. As you can see, because font B contains the minimum width value across the entire family, font B would be selected by this algorithm. However, if font B were somehow eliminated from the family, font C would then contain the lowest distance in the family, so it would be selected.

        <a id="ex-ascending-stretch-2"></a>
        <strong>Example:</strong>

        [](#ex-ascending-stretch-2) Similar to the [previous example](#ex-ascending-stretch), here is the conceptual distance graph for an element styled with "font-width: 75%":
        ![distance graph](https://drafts.csswg.org/css-fonts-4/images/stretchdistance2.svg)

        As you can see, because font B contains the minimum width value across the entire family, font B would be selected by this algorithm. However, if font B were somehow eliminated from the family, font A would then contain the lowest distance in the family, so it would be selected.

    2.  <a id="ref-for-propdef-font-style①⑤"></a>[font-style](https://drafts.csswg.org/css-fonts-4/#propdef-font-style) is tried next (see [§ 5.2 Matching font styles](#font-style-matching)). If a font does not have any concept of varying strengths of italics or oblique angles, its style is mapped according to the description in the <a id="ref-for-propdef-font-style①⑥"></a>font-style property definition.

        If the value of <a id="ref-for-propdef-font-style①⑦"></a>[font-style](https://drafts.csswg.org/css-fonts-4/#propdef-font-style) is <a id="ref-for-valdef-font-style-italic③"></a>[italic](https://drafts.csswg.org/css-fonts-4/#valdef-font-style-italic):

        1.  If the matching set includes faces with italic values containing the mapped value of <a id="ref-for-valdef-font-style-italic④"></a>[italic](https://drafts.csswg.org/css-fonts-4/#valdef-font-style-italic), then faces with italic values which do not include the desired italic mapped value are removed from the matching set.

        2.  Otherwise, italic values above the desired italic value are checked in ascending order followed by italic values below the desired italic value, until 0 is hit. Only positive values of italic values are checked in this stage.

        3.  For variable fonts with an ital axis, a match is created by setting the ital value to 1.

        4.  If no match is found, oblique values greater than or equal to 11deg are checked in ascending order followed by oblique values below 11deg in descending order, until 0 is hit. Only positive values of oblique values are checked in this stage.

            <a id="issue-c152d461"></a>

            <strong>Issue:</strong>

            [](#issue-c152d461) The threshold for preferring oblique over normal [should be lower than the average angle](https://github.com/w3c/csswg-drafts/issues/2295).

        5.  If no match is found, italic values less than or equal to 0 are checked in descending order until a match is found.

        6.  If no match is found, oblique values less than or equal to 0deg are checked in descending order until a match is found.

            <a id="ex-ascending-italic"></a>
            <strong>Example:</strong>

            [](#ex-ascending-italic) Similar to the [previous example](#ex-ascending-stretch), here is the conceptual distance graph for an element styled with "font-style: italic":
            ![distance graph](https://drafts.csswg.org/css-fonts-4/images/styledistance.svg)

            As you can see, because font D contains the minimum italic value across the entire family, font D would be selected by this algorithm. However, if font D were somehow eliminated from the family, font E would then contain the lowest distance in the family, so it would be selected. If E were eliminated, C would be selected. If C were eliminated, font B would not be chosen immediately; instead, oblique values would be consulted and an oblique value might be chosen. However, if no oblique value is chosen, font B would then be selected, followed by font A.

        If the value of <a id="ref-for-propdef-font-style①⑧"></a>[font-style](https://drafts.csswg.org/css-fonts-4/#propdef-font-style) is <a id="ref-for-valdef-font-style-oblique①"></a>[oblique](https://drafts.csswg.org/css2/#valdef-font-style-oblique) and the requested angle is greater than or equal to 11deg,

        <a id="greater-oblique-steps"></a>

        1.  If the matching set includes faces with oblique values containing the value of <a id="ref-for-valdef-font-style-oblique②"></a>[oblique](https://drafts.csswg.org/css2/#valdef-font-style-oblique), faces with oblique values which do not include the desired oblique value are removed from the matching set.

        2.  Otherwise, oblique values above the desired oblique value are checked in ascending order followed by oblique values below the desired oblique value, until 0 is hit. Only positive values of oblique values are checked in this stage.

        3.  For variable fonts with a slnt axis, a match is created by setting the slnt value with the specified oblique value. Otherwise, if <a id="ref-for-propdef-font-synthesis-style⑤"></a>[font-synthesis-style](https://drafts.csswg.org/css-fonts-4/#propdef-font-synthesis-style) has the value <a id="ref-for-valdef-font-synthesis-style-auto"></a>[auto](https://drafts.csswg.org/css-fonts-4/#valdef-font-synthesis-style-auto), then a fallback match is produced by geometric shearing to the specified oblique value. The ital axis is not used to satisfy an <a id="ref-for-valdef-font-style-oblique③"></a>[oblique](https://drafts.csswg.org/css2/#valdef-font-style-oblique) request.

        4.  If no match is found, italic values greater than or equal to 1 are checked in ascending order followed by italic values below 1 in descending order, until 0 is hit. Only positive values of italic values are checked in this stage.

        5.  If no match is found, oblique values less than or equal to 0deg are checked in descending order until a match is found.

        6.  If no match is found, italic values less than or equal to 0 are checked in descending order until a match is found.

            <a id="ex-ascending-oblique-40"></a>
            <strong>Example:</strong>

            [](#ex-ascending-oblique-40) Similar to the [previous example](#ex-ascending-stretch), here is the conceptual distance graph for an element styled with "font-style: oblique 40deg":
            ![distance graph](https://drafts.csswg.org/css-fonts-4/images/styledistance2.svg)

            As you can see, because font D contains the minimum oblique value across the entire family, font D would be selected by this algorithm. However, if font D were somehow eliminated from the family, font E would then contain the lowest distance in the family, so it would be selected. If E were eliminated, C would be selected. If C were eliminated, font B would not be chosen immediately; instead, italic values would be consulted and an italic value might be chosen. However, if no italic value is chosen, font B would then be selected, followed by font A.

        If the value of <a id="ref-for-propdef-font-style①⑨"></a>[font-style](https://drafts.csswg.org/css-fonts-4/#propdef-font-style) is <a id="ref-for-valdef-font-style-oblique④"></a>[oblique](https://drafts.csswg.org/css2/#valdef-font-style-oblique) and the requested angle is greater than or equal to 0deg and less than 11deg,

        <a id="lesser-oblique-steps"></a>

        1.  If the matching set includes faces with oblique values containing the value of <a id="ref-for-valdef-font-style-oblique⑤"></a>[oblique](https://drafts.csswg.org/css2/#valdef-font-style-oblique), faces with oblique values which do not include the desired oblique value are removed from the matching set.

        2.  Otherwise, oblique values below the desired oblique value are checked in descending order until 0 is hit, followed by oblique values above the desired oblique value. Only positive values of oblique values are checked in this stage.

        3.  For variable fonts with a slnt axis, a match is created by setting the slnt value with the specified oblique value. Otherwise, if <a id="ref-for-propdef-font-synthesis-style⑥"></a>[font-synthesis-style](https://drafts.csswg.org/css-fonts-4/#propdef-font-synthesis-style) has the value <a id="ref-for-valdef-font-synthesis-style-auto①"></a>[auto](https://drafts.csswg.org/css-fonts-4/#valdef-font-synthesis-style-auto), then a fallback match is produced by geometric shearing to the specified oblique value. The ital axis is not used to satisfy an <a id="ref-for-valdef-font-style-oblique⑥"></a>[oblique](https://drafts.csswg.org/css2/#valdef-font-style-oblique) request.

        4.  If no match is found, italic values less than 1 are checked in descending order until 0 is hit, followed by italic values above 1 in ascending order. Only positive values of italic values are checked in this stage.

        5.  If no match is found, oblique values less than or equal to 0deg are checked in descending order until a match is found.

        6.  If no match is found, italic values less than or equal to 0 are checked in descending order until a match is found.

        <a id="ex-ascending-oblique-13"></a>
        <strong>Example:</strong>

        [](#ex-ascending-oblique-13) Similar to the [previous example](#ex-ascending-stretch), here is the conceptual distance graph for an element styled with "font-style: oblique 13deg":
        ![distance graph](https://drafts.csswg.org/css-fonts-4/images/styledistance3.svg)

        As you can see, because font D contains the minimum oblique value across the entire family, font D would be selected by this algorithm. However, if font D were somehow eliminated from the family, font C would then contain the lowest distance in the family, so it would be selected. If C were eliminated, E would be selected. If E were eliminated, font B would not be chosen immediately; instead, italic values would be consulted and an italic value might be chosen. However, if no italic value is chosen, font B would then be selected, followed by font A.

        If the value of <a id="ref-for-propdef-font-style②⓪"></a>[font-style](https://drafts.csswg.org/css-fonts-4/#propdef-font-style) is <a id="ref-for-valdef-font-style-oblique⑦"></a>[oblique](https://drafts.csswg.org/css2/#valdef-font-style-oblique) and the requested angle is less than 0deg and greater than -11deg, follow the steps [above](#lesser-oblique-steps), except with the negated values and opposite directions. If the value of <a id="ref-for-propdef-font-style②①"></a>font-style is <a id="ref-for-valdef-font-style-oblique⑧"></a>oblique and the requested angle is less than or equal to -11deg, follow the steps [above](#greater-oblique-steps), except with the negated values and opposite directions.

        If the value of <a id="ref-for-propdef-font-style②②"></a>[font-style](https://drafts.csswg.org/css-fonts-4/#propdef-font-style) is <a id="ref-for-valdef-font-style-normal②"></a>[normal](https://drafts.csswg.org/css-fonts-4/#valdef-font-style-normal),

        1.  Oblique values greater than or equal to 0 are checked in ascending order.

        2.  If no match is found, italic values greater than or equal to 0 are checked in ascending

        3.  If no match is found, oblique values less than 0deg are checked in descending order until a match is found.

        4.  If no match is found, italic values less than 0 are checked in descending order until a match is found.

        <a id="ex-ascending-normal"></a>
        <strong>Example:</strong>

        [](#ex-ascending-normal) Similar to the [previous example](#ex-ascending-stretch), here is the conceptual distance graph for an element styled with "font-style: normal":
        ![distance graph](https://drafts.csswg.org/css-fonts-4/images/styledistance4.svg)

        As you can see, because font C contains the minimum oblique value across the entire family, font C would be selected by this algorithm. However, if font C were somehow eliminated from the family, font B would not be chosen immediately; instead, italic values would be consulted and an italic value might be chosen. However, if no italic value is chosen, font B would then be selected, followed by font A.

        If an oblique angle was found in the above search, all faces which don’t include that oblique angle are excluded from the matching set. Otherwise, if an italic value was found in the above search, all faces which don’t include that italic value are excluded from the matching set.

        Tests

        - [oblique-last-resort-weight-selection.html](https://wpt.fyi/results/css/css-fonts/oblique-last-resort-weight-selection.html) [(live test)](http://wpt.live/css/css-fonts/oblique-last-resort-weight-selection.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/oblique-last-resort-weight-selection.html)
        - [oblique-request-italic-only-family-no-crash.html](https://wpt.fyi/results/css/css-fonts/oblique-request-italic-only-family-no-crash.html) [(live test)](http://wpt.live/css/css-fonts/oblique-request-italic-only-family-no-crash.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/oblique-request-italic-only-family-no-crash.html)

        User agents are not required to distinguish between italic and oblique fonts. In such user agents, the <a id="ref-for-propdef-font-style②③"></a>[font-style](https://drafts.csswg.org/css-fonts-4/#propdef-font-style) matching steps above are performed by mapping both italic values and oblique angles onto a common scale. The exact nature of this mapping is undefined, however, an italic value of 1 must map to the same value that an oblique angle of 11deg maps to. Within font families defined via <a id="ref-for-at-font-face-rule⑥⓪"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-4/#at-font-face-rule) rules, italic and oblique faces must be distinguished using the value of the <a id="ref-for-descdef-font-face-font-style①"></a>[font-style](https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-style) descriptor.

        For families that lack any italic or oblique faces, user agents may create artificial oblique faces, if this is permitted by the value of the <a id="ref-for-propdef-font-synthesis⑥"></a>[font-synthesis](https://drafts.csswg.org/css-fonts-4/#propdef-font-synthesis) property.

    3.  <a id="ref-for-propdef-font-weight②①"></a>[font-weight](https://drafts.csswg.org/css-fonts-4/#propdef-font-weight) is matched next. If a font does not have any concept of varying strengths of weights, its weight is mapped according list in the [property definition](https://drafts.csswg.org/css-fonts-4/#font-weight-numeric-values). If bolder/lighter relative weights are used, the effective weight is calculated based on the inherited weight value, as described in the definition of the <a id="ref-for-propdef-font-weight②②"></a>font-weight property. If the matching set after performing the steps above includes faces with weight values containing the font-weight desired value, faces with weight values which do not include the desired font-weight value are removed from the matching set. If there is no face which contains the desired value, a weight value is chosen using the rules below:

        - If the desired weight is inclusively between 400 and 500, weights greater than or equal to the target weight are checked in ascending order until 500 is hit and checked, followed by weights less than the target weight in descending order, followed by weights greater than 500, until a match is found.

        - If the desired weight is less than 400, weights less than or equal to the desired weight are checked in descending order followed by weights above the desired weight in ascending order until a match is found.

        - If the desired weight is greater than 500, weights greater than or equal to the desired weight are checked in ascending order followed by weights below the desired weight in descending order until a match is found.

          Tests

          - [font-weight-search-direction.html](https://wpt.fyi/results/css/css-fonts/matching/font-weight-search-direction.html) [(live test)](http://wpt.live/css/css-fonts/matching/font-weight-search-direction.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/matching/font-weight-search-direction.html)

        <a id="ex-weight-400"></a>
        <strong>Example:</strong>

        [](#ex-weight-400) Similar to the [previous example](#ex-ascending-stretch), here is the conceptual distance graph for an element styled with "font-weight: 400":
        ![distance graph](https://drafts.csswg.org/css-fonts-4/images/styleweight.svg)

        As you can see, because font B contains the minimum distance across the entire family, font B would be selected by this algorithm. However, if font B were somehow eliminated from the family, font C would then contain the lowest distance in the family, so it would be selected. If C were eliminated, D would be selected, followed by fonts A and then E.

        <a id="ex-weight-450"></a>
        <strong>Example:</strong>

        [](#ex-weight-450) Similar to the [previous example](#ex-ascending-stretch), here is the conceptual distance graph for an element styled with "font-weight: 450":
        ![distance graph](https://drafts.csswg.org/css-fonts-4/images/styleweight450.svg)

        As you can see, because font C contains the minimum distance across the entire family, font C would be selected by this algorithm. However, if font C were somehow eliminated from the family, font D would then contain the lowest distance in the family, so it would be selected. If D were also eliminated, B would be selected, followed by fonts A and then E.

        <a id="ex-weight-500"></a>
        <strong>Example:</strong>

        [](#ex-weight-500) Similar to the [previous example](#ex-ascending-stretch), here is the conceptual distance graph for an element styled with "font-weight: 500":
        ![distance graph](https://drafts.csswg.org/css-fonts-4/images/weightmatching.svg)

        As you can see, because font D contains the minimum distance across the entire family, font D would be selected by this algorithm. However, if font D were somehow eliminated from the family, font B would then contain the lowest distance in the family, so it would be selected. If B were eliminated, C would be selected, followed by fonts B, A, and then E.

        <a id="ex-weight-300"></a>
        <strong>Example:</strong>

        [](#ex-weight-300) Similar to the [previous example](#ex-ascending-stretch), here is the conceptual distance graph for an element styled with "font-weight: 300":
        ![distance graph](https://drafts.csswg.org/css-fonts-4/images/weightmatching2.svg)

        As you can see, because font B contains the minimum distance across the entire family, font B would be selected by this algorithm. However, if font B were somehow eliminated from the family, font A would then contain the lowest distance in the family, so it would be selected. If A were eliminated, C would be selected.

        Once the closest matching weight has been determined by this process, faces with weights which do not include this determined weight are removed from the matching set.

        Note: There is a small behavior change between [\[CSS-FONTS-3\]](https://drafts.csswg.org/css-fonts-4/#biblio-css-fonts-3) and this specification with the animation of the <a id="ref-for-propdef-font-weight②③"></a>[font-weight](https://drafts.csswg.org/css-fonts-4/#propdef-font-weight) property. Previously, interpolated values of font-weight were rounded to their closest multiple of 100, and the font-matching algorithm was run on these rounded values. In this specification, the font-matching algorithm is able to accept any value, so no rounding occurs. The small behavior change is due to the discontinuous nature of the font-matching algorithm.

    4.  <a id="ref-for-propdef-font-size①⑦"></a>[font-size](https://drafts.csswg.org/css-fonts-4/#propdef-font-size) must be matched within a UA-dependent margin of tolerance. (Typically, sizes for scalable fonts are rounded to the nearest whole pixel, while the tolerance for bitmapped fonts could be as large as 20%.) Further computations, e.g., by <a id="ref-for-em①"></a>[em](https://drafts.csswg.org/css-values-4/#em) values in other properties, are based on the <a id="ref-for-propdef-font-size①⑧"></a>font-size value that is used, not the one that is specified.

    Note that more than one font might be remaining in the matching set after performing the above steps. If so, the user agent must choose a single font from the matching set and continue these steps with it. The choice of which font to choose can differ between multiple user agents and multiple operating system platforms; however, it must not differ between two elements in the same document.

5.  If the matched face is defined via <a id="ref-for-at-font-face-rule⑥①"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-4/#at-font-face-rule) rules, user agents must use the procedure below to select a single font:

    1.  If the font resource has not been loaded and the range of characters defined by the <a id="ref-for-descdef-font-face-unicode-range⑤"></a>[unicode-range](https://drafts.csswg.org/css-fonts-4/#descdef-font-face-unicode-range) descriptor value includes the character in question, load the font.

    2.  After downloading, if the <a id="ref-for-effective-character-map"></a>[effective character map](https://drafts.csswg.org/css-fonts-4/#effective-character-map) supports the character in question, select that font.

    When the matched face is a <a id="ref-for-composite-face①"></a>[composite face](#composite-face), user agents must use the procedure above on each of the faces in the <a id="ref-for-composite-face②"></a>composite face in reverse order of <a id="ref-for-at-font-face-rule⑥②"></a>[&#64;font-face](https://drafts.csswg.org/css-fonts-4/#at-font-face-rule) rule definition.

    While the download occurs, user agents must either wait until the font is downloaded or render once with substituted font metrics and render again once the font is downloaded.

6.  If no matching face exists or the matched face does not contain a glyph for the character to be rendered, the next family name is selected and the previous three steps repeated. Glyphs from other faces in the family are not considered. The only exception is that user agents may optionally substitute a synthetically obliqued version of the <a id="ref-for-default-face"></a>[default face](#default-face) if that face supports a given glyph and synthesis of these faces is permitted by the value of the <a id="ref-for-propdef-font-synthesis⑦"></a>[font-synthesis](https://drafts.csswg.org/css-fonts-4/#propdef-font-synthesis) property. For example, a synthetic italic version of the regular face might be used if the italic face doesn’t support glyphs for Arabic.

7.  If there are no more font families to be evaluated and no matching face has been found, then the user agent performs an <a id="installed-font-fallback"></a><strong>installed font fallback</strong> procedure to find the best match for the character to be rendered. The result of this procedure can vary across user agents.

8.  If a particular character cannot be displayed using any font, the user agent should indicate by some means that a character is not being displayed, displaying either a symbolic representation of the missing glyph (e.g. using a [Last Resort Font](https://en.wikipedia.org/wiki/Last_resort_font)) or using the missing character glyph from a default font.

Optimizations of this process are allowed provided that an implementation behaves as if the algorithm had been followed exactly. Matching occurs in a well-defined order to ensure that the results are as consistent as possible across user agents, given an identical set of available fonts and rendering tech.

The <a id="first-available-font"></a><strong>first available font</strong>, used for example in the definition of <a id="ref-for-font-relative-length②"></a>[font-relative lengths](https://drafts.csswg.org/css-values-4/#font-relative-length) such as <a id="ref-for-ex"></a>[ex](https://drafts.csswg.org/css-values-4/#ex) or in the definition of the <a id="ref-for-propdef-line-height⑧"></a>[line-height](https://drafts.csswg.org/css2/#propdef-line-height) property, is defined to be the first font for which the character U+0020 (space) is not excluded by a <a id="ref-for-descdef-font-face-unicode-range⑥"></a>[unicode-range](https://drafts.csswg.org/css-fonts-4/#descdef-font-face-unicode-range), given the font families in the <a id="ref-for-propdef-font-family①③"></a>[font-family](https://drafts.csswg.org/css-fonts-4/#propdef-font-family) list (or a user agent’s default font if none are available).

Installed fonts referenced directly by family name, rather than via &#64;font-face rules, are considered to have a <a id="ref-for-descdef-font-face-unicode-range⑦"></a>[unicode-range](https://drafts.csswg.org/css-fonts-4/#descdef-font-face-unicode-range) that covers the entire Unicode code space.

Tests

- [first-available-font-001.html](https://wpt.fyi/results/css/css-fonts/first-available-font-001.html) [(live test)](http://wpt.live/css/css-fonts/first-available-font-001.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/first-available-font-001.html)
- [first-available-font-002.html](https://wpt.fyi/results/css/css-fonts/first-available-font-002.html) [(live test)](http://wpt.live/css/css-fonts/first-available-font-002.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/first-available-font-002.html)
- [first-available-font-003.html](https://wpt.fyi/results/css/css-fonts/first-available-font-003.html) [(live test)](http://wpt.live/css/css-fonts/first-available-font-003.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/first-available-font-003.html)
- [first-available-font-004.html](https://wpt.fyi/results/css/css-fonts/first-available-font-004.html) [(live test)](http://wpt.live/css/css-fonts/first-available-font-004.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/first-available-font-004.html)
- [first-available-font-005.html](https://wpt.fyi/results/css/css-fonts/first-available-font-005.html) [(live test)](http://wpt.live/css/css-fonts/first-available-font-005.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/first-available-font-005.html)
- [first-available-font-006.html](https://wpt.fyi/results/css/css-fonts/first-available-font-006.html) [(live test)](http://wpt.live/css/css-fonts/first-available-font-006.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/first-available-font-006.html)
- [first-available-font-007.html](https://wpt.fyi/results/css/css-fonts/first-available-font-007.html) [(live test)](http://wpt.live/css/css-fonts/first-available-font-007.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/first-available-font-007.html)
- [italic-oblique-fallback.html](https://wpt.fyi/results/css/css-fonts/italic-oblique-fallback.html) [(live test)](http://wpt.live/css/css-fonts/italic-oblique-fallback.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/italic-oblique-fallback.html)

Note: it does not matter whether that font actually has a glyph for the space character.
