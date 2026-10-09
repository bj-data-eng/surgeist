Attribution and reformatting notice added for Surgeist on 2026-10-09

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Fonts Module Level 5](https://www.w3.org/TR/2021/WD-css-fonts-5-20211221/).

Original copyright notice: Copyright © 2021 W3C ® ( MIT , ERCIM , Keio , Beihang ). W3C liability , trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2015 version](../licenses/w3c/software-license-2015.txt). Changes are format conversion, visible semantic labels, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Fonts Module Level 5

Source snapshot: https://www.w3.org/TR/2021/WD-css-fonts-5-20211221/

Snapshot SHA-256: 6dfbf10b25f174035e51ed66209af885722d6bacf2d72b8612703d8a844142a1

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Source row-header labels in readable Markdown tables are bold; native HTML th/scope accessibility semantics are not expressible in GFM. Field/Definition headings, where used, are added non-normative presentation labels.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Fonts Module Level 5

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2021 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](https://www.csail.mit.edu/), [ERCIM](https://www.ercim.eu/), [Keio](https://www.keio.ac.jp/), [Beihang](https://ev.buaa.edu.cn/)). W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [permissive document license](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document) rules apply.

## <a id="abstract"></a>Abstract

This specification defines modifications to the existing [CSS Fonts 4](https://drafts.csswg.org/css-fonts-4/) specification along with additional features.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="status"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.&#xA;&#x9;A list of current W3C publications&#xA;&#x9;and the latest revision of this technical report&#xA;&#x9;can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Working Draft</strong> using the [Recommendation track](https://www.w3.org/2021/Process-20211102/#recs-and-notes). Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-fonts” in the title, like this: “\[css-fonts\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-fonts%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [2 November 2021 W3C Process Document](https://www.w3.org/2021/Process-20211102/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

## <a id="introduction"></a>1.  Introduction

The CSS Fonts Level 4 specification ([\[CSS-FONTS-4\]](#biblio-css-fonts-4)) describes the controls CSS provides for selecting and using fonts within documents, including support for variable fonts and color fonts. The ideas here are additions or modifications to the properties and rules defined in CSS Fonts Level 4.

This specification is currently a delta to the CSS Fonts Level 4 specification. Do not assume that if something is not here, it has been dropped.

### <a id="values"></a>1.1.  Value Definitions

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="basic-font-props"></a>2.  Basic Font Properties

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9"></a>[w3c/csswg-drafts/126](https://github.com/w3c/csswg-drafts/issues/126)[\[css-fonts\] Specifying changes to parameters for fallback fonts](https://github.com/w3c/csswg-drafts/issues/126)

<a id="ref-for-propdef-font-family"></a>

### <a id="font-family-prop"></a>2.1.  Font family: the [font-family](https://www.w3.org/TR/css-fonts-4/#propdef-font-family) property

#### <a id="generic-font-families"></a>2.1.1.  Generic font families

In addition to the [CSS Fonts 4 § 2.1.3 Generic font families](https://www.w3.org/TR/css-fonts-4/#generic-font-families) in CSS Fonts Level 4, the following new generic font families are also defined.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9①"></a>[w3c/csswg-drafts/4910](https://github.com/w3c/csswg-drafts/issues/4910)[\[meta\] \[css-fonts\] Criteria for generic font families](https://github.com/w3c/csswg-drafts/issues/4910)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9②"></a>[w3c/csswg-drafts/5054](https://github.com/w3c/csswg-drafts/issues/5054)[\[css-fonts\] generic family for Pi, Picture, Symbols fonts](https://github.com/w3c/csswg-drafts/issues/5054)

<a id="xxx-def"></a><a id="valdef-font-family-xxx"></a>xxx  
Placeholder text for the xxx generic font family.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9③"></a>[w3c/csswg-drafts/4566](https://github.com/w3c/csswg-drafts/issues/4566)[\[css-fonts\] Should we start a registry for additional generic fonts?](https://github.com/w3c/csswg-drafts/issues/4566)

<a id="ref-for-propdef-font-weight"></a>

### <a id="font-weight-prop"></a>2.2. Font weight: the [font-weight](https://www.w3.org/TR/css-fonts-4/#propdef-font-weight) property

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9④"></a>[w3c/csswg-drafts/2690](https://github.com/w3c/csswg-drafts/issues/2690)[\[css-fonts-4\] Percentages in font-weight for relative weights](https://github.com/w3c/csswg-drafts/issues/2690)

<a id="ref-for-propdef-font-style"></a>

### <a id="font-style-prop"></a>2.3.  Font style: the [font-style](https://www.w3.org/TR/css-fonts-4/#propdef-font-style) property

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9⑤"></a>[w3c/csswg-drafts/4044](https://github.com/w3c/csswg-drafts/issues/4044)[\[css-fonts\] Vertical text doesn't play nicely with font-style and font-stretch](https://github.com/w3c/csswg-drafts/issues/4044)

<a id="ref-for-propdef-font-size-adjust"></a>

### <a id="font-size-adjust-prop"></a>2.4.  Relative sizing: the [font-size-adjust](#propdef-font-size-adjust) property



| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                            |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-font-size-adjust"></a>font-size-adjust                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-number-value"></a><a id="ref-for-mult-opt"></a><a id="ref-for-comb-one"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[ ex-height <a id="ref-for-comb-one①"></a>\| cap-height <a id="ref-for-comb-one②"></a>\| ch-width <a id="ref-for-comb-one③"></a>\| ic-width <a id="ref-for-comb-one④"></a>\| ic-height \][?](https://www.w3.org/TR/css-values-4/#mult-opt) \[ from-font <a id="ref-for-comb-one⑤"></a>\| [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | all elements and text                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-number-value①"></a><a id="ref-for-valdef-font-size-adjust-none"></a>the keyword [none](#valdef-font-size-adjust-none), or a pair of a metric keyword and a [\<number\>](https://www.w3.org/TR/css-values-4/#number-value)                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete if the keywords differ, otherwise by computed value type                                                                                                                                                                                                                                                                                                                                                     |



For any given font size, the apparent size and effective legibility of text varies across fonts as a function of their design. For example, for bicameral scripts such as Latin or Cyrillic that distinguish between upper and lowercase letters, the relative height of lowercase letters compared to their uppercase counterparts is a determining factor of legibility. In situations where font fallback occurs, fallback fonts might not share the same ratios as the desired font family for key typographic metrics, and will thus appear to be a different size and possibly be less readable.

<a id="ref-for-propdef-font-size-adjust①"></a>

The [font-size-adjust](#propdef-font-size-adjust) property provides a way to preserve the readability and apparent size of text when font fallback occurs. It does this by adjusting the used font size so that the specified metric is the same regardless of the font used.

Values have the following meanings:

<a id="valdef-font-size-adjust-none"></a>none

<a id="ref-for-propdef-font-size"></a>

No special [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) adjustment is applied.

<a id="valdef-font-size-adjust-ex-height--cap-height--ch-width--ic-width--ic-height"></a>ex-height \| cap-height \| ch-width \| ic-width \| ic-height

<a id="ref-for-valdef-font-size-adjust-ex-height"></a>

Specifies the font metric to normalize, defaulting to [ex-height](#valdef-font-size-adjust-ex-height):

<a id="valdef-font-size-adjust-ex-height"></a>ex-height  
Normalize the <a id="font-size-adjust-aspect-value"></a>aspect value of the fonts, using the x-height divided by the font size.

<a id="valdef-font-size-adjust-cap-height"></a>cap-height  
Normalize the cap-height of the fonts, using the cap-height by the font size.

<a id="valdef-font-size-adjust-ch-width"></a>ch-width  
Normalize the horizontal narrow pitch of the fonts, using the advance width of “0” (ZERO, U+0030) divided by the font size.

<a id="valdef-font-size-adjust-ic-width"></a>ic-width  
Normalize the horizontal wide pitch of the font, using the advance width of “水” (CJK water ideograph, U+6C34) divided by the font size.

<a id="valdef-font-size-adjust-ic-height"></a>ic-height  
Normalize the vertical wide pitch of the font, using the advance height of “水” (CJK water ideograph, U+6C34) divided by the font size.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-c72cd359"></a>[w3c/csswg-drafts/6384](https://github.com/w3c/csswg-drafts/issues/6384)[\[css-fonts-5\] font-size-adjust with missing metrics](https://github.com/w3c/csswg-drafts/issues/6384)

<a id="ref-for-number-value②"></a>

<a id="valdef-font-size-adjust-number"></a>[\<number\>](https://www.w3.org/TR/css-values-4/#number-value)

<a id="ref-for-propdef-font-size①"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-used-value"></a>

Each font’s [used](https://www.w3.org/TR/css-cascade-5/#used-value) size is normalized to match the chosen font metric to this specified proportion of the [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size). In other words, for each glyph, the adjusted font size to use <var>u</var> is calculated as:

```text
u  =  ( m / m′ ) s
```
where:

```text
s  =  computed 'font-size!!property' value
m  =  metric as specified by the 'font-size-adjust' property
m′ =  metric as specified in the actual font
u  =  adjusted font-size to use
```
Negative values are invalid.

<a id="valdef-font-size-adjust-from-font"></a>from-font

<a id="ref-for-first-available-font"></a>

<a id="ref-for-number-value③"></a>

Computes to the [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) corresponding to the specified metric of the [first available font](https://www.w3.org/TR/css-fonts-4/#first-available-font).

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-36eece46"></a>[w3c/csswg-drafts/6384](https://github.com/w3c/csswg-drafts/issues/6384)[\[css-fonts-5\] font-size-adjust with missing metrics](https://github.com/w3c/csswg-drafts/issues/6384)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-63234e3a"></a> The style defined below defines Verdana as the desired font family, but if Verdana is not available Futura or Times will be used. One paragraph also has font-size-adjust specified.
>
> ```text
> p {
>   font-family: Verdana, Futura, Times;
> }
> p.adj {
>   font-size-adjust: 0.545;
> }
> 
> <p>Lorem ipsum dolor sit amet, ...</p>
> <p class="adj">Lorem ipsum dolor sit amet, ...</p>
> ```
>
> Verdana has a relatively high aspect value of 0.545, meaning lowercase letters are relatively tall compared to uppercase letters, so at small sizes text appears legible. Times has a lower aspect value of 0.447, and so if fallback occurs, the text will be less legible at small sizes than Verdana unless font-size-adjust is also specified.
>
> > <strong data-conversion-semantic="note">Note</strong>
> >
> > Note: For text which uses diacritics, too large an x-height will actually decrease legibility as the diacritics become cramped.
>
> <a id="ref-for-propdef-font-size-adjust②"></a>
>
> How text rendered in each of these fonts compares is shown below, the columns show text rendered in Verdana, Futura and Times. The same font-size value is used across cells within each row and red lines are included to show the differences in x-height. In the upper half, each row is rendered in the same font-size value. The same is true for the lower half, but in this half the [font-size-adjust](#propdef-font-size-adjust) property is also set to 0.545, so that the actual font size is adjusted to preserve the x-height of Verdana across each row. Note how small text remains relatively legible across each row in the lower half.
>
> ![text with and without 'font-size-adjust'](https://www.w3.org/TR/2021/WD-css-fonts-5-20211221/images/fontsizeadjust.png)
>
> <a id="ref-for-propdef-font-size-adjust③"></a>
>
> Text with and without the use of [font-size-adjust](#propdef-font-size-adjust)

<a id="ref-for-propdef-font-size-adjust④"></a>

<a id="ref-for-used-value①"></a>

<a id="ref-for-propdef-font-size②"></a>

<a id="ref-for-computed-value①"></a>

<a id="ref-for-propdef-line-height"></a>

The value of [font-size-adjust](#propdef-font-size-adjust) affects the [used](https://www.w3.org/TR/css-cascade-5/#used-value) value of [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) but does not affect the [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) value. Therefore it can affect the size of relative units that are based on font metrics such as `ex` and `ch` but does not affect the size of `em` units. Since numeric values of [line-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height) refer to the <a id="ref-for-computed-value②"></a>computed size of <a id="ref-for-propdef-font-size③"></a>font-size, <a id="ref-for-propdef-font-size-adjust⑤"></a>font-size-adjust also does not affect the <a id="ref-for-used-value②"></a>used value of <a id="ref-for-propdef-line-height①"></a>line-height.

<a id="ref-for-propdef-font-size-adjust⑥"></a>

<a id="ref-for-propdef-line-height②"></a>

<a id="ref-for-font-size-adjust-aspect-value"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since [font-size-adjust](#propdef-font-size-adjust) does not factor into the [line-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height), specifying a line height too tightly can result in overlapping lines of text. For example, when a fallback font with a low [aspect value](#font-size-adjust-aspect-value) is normalized to match one with a high <a id="ref-for-font-size-adjust-aspect-value①"></a>aspect value, its ascenders and descenders are likely to extend outside the line box with <a id="ref-for-propdef-line-height③"></a>line-height: 1.

<a id="ref-for-propdef-font-size-adjust⑦"></a>

<a id="ref-for-propdef-font-family①"></a>

The [font-size-adjust](#propdef-font-size-adjust) adjustment applies to any font that is selected but in typical usage it would be based on the corresponding metric value of the first (most desired) font in the [font-family](https://www.w3.org/TR/css-fonts-4/#propdef-font-family) list. If this is specified accurately, the <code><c->(</c-><var>m</var>/<var>m′</var><c->)</c-></code> term in the adjustment formula will resolve to `1` for the first font and no adjustment occurs for that font; and the rest of the fonts will resolve to match. If the value is specified inaccurately, text rendered using the first font in the family list will display differently in older user agents that don’t support <a id="ref-for-propdef-font-size-adjust⑧"></a>font-size-adjust.

<a id="ref-for-font-size-adjust-aspect-value②"></a>

<a id="ref-for-propdef-font-size-adjust⑨"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-771aee48"></a> Authors can calculate the [aspect value](#font-size-adjust-aspect-value) for a given font by comparing spans with the same content but different [font-size-adjust](#propdef-font-size-adjust) properties. If the same font-size is used, the spans will match when the <a id="ref-for-propdef-font-size-adjust①⓪"></a>font-size-adjust value is accurate for the given font.
>
> <a id="ref-for-font-size-adjust-aspect-value③"></a>
>
> <a id="ref-for-propdef-font-size④"></a>
>
> <a id="ref-for-propdef-font-size-adjust①①"></a>
>
> Two spans with borders are used to determine the [aspect value](#font-size-adjust-aspect-value) of a font. The [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) is the same for both spans but the [font-size-adjust](#propdef-font-size-adjust) property is specified only for the right span. Starting with a value of 0.5, the aspect value can be adjusted until the borders around the two letters line up.
>
> ```text
> p {
>   font-family: Futura;
>   font-size: 500px;
> }
> 
> span {
>   border: solid 1px red;
> }
> 
> .adjust {
>   font-size-adjust: 0.5;
> }
> 
> <p><span>b</span><span class="adjust">b</span></p>
> ```
>
> ![Futura with an aspect value of 0.5](https://www.w3.org/TR/2021/WD-css-fonts-5-20211221/images/beforefontsizeadjust.png)
>
> <a id="ref-for-font-size-adjust-aspect-value④"></a>
>
> Futura with an [aspect value](#font-size-adjust-aspect-value) of 0.5
>
> <a id="ref-for-font-size-adjust-aspect-value⑤"></a>
>
> The box on the right is a bit bigger than the one on the left, so the [aspect value](#font-size-adjust-aspect-value) of this font is something less than 0.5. Adjust the value until the boxes align.

<a id="ref-for-at-font-face-rule"></a>

<a id="ref-for-descdef-font-face-size-adjust"></a>

<a id="ref-for-propdef-font-size-adjust①②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If the specified metric has been overridden in [@font-face](#at-font-face-rule), e.g. by [size-adjust](#descdef-font-face-size-adjust), then the overridden metric will be used in the [font-size-adjust](#propdef-font-size-adjust) calculation. Consequently, applying <a id="ref-for-propdef-font-size-adjust①③"></a>font-size-adjust and <a id="ref-for-descdef-font-face-size-adjust①"></a>size-adjust together means that <a id="ref-for-descdef-font-face-size-adjust②"></a>size-adjust appears to have no effect.

## <a id="font-resources"></a>3.  Font Resources

<a id="ref-for-at-font-face-rule①"></a>

### <a id="font-face-rule"></a>3.1.  The <a id="at-font-face-rule"></a>[@font-face](#at-font-face-rule) rule

<a id="ref-for-first-available-font①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Descriptors are applied per-font rather than per-element. Multiple fonts can be used within an individual element e.g. for characters not supported by the [first available font](https://www.w3.org/TR/css-fonts-4/#first-available-font).

<a id="ref-for-descdef-font-face-src"></a>

### <a id="src-desc"></a>3.2. Font reference: the [src](https://www.w3.org/TR/css-fonts-4/#descdef-font-face-src) descriptor

<a id="ref-for-descdef-font-face-src①"></a>

#### <a id="font-face-src-parsing"></a>3.2.1. Parsing the [src](https://www.w3.org/TR/css-fonts-4/#descdef-font-face-src) descriptor

<a id="ref-for-descdef-font-face-src②"></a>

The [src](https://www.w3.org/TR/css-fonts-4/#descdef-font-face-src) descriptor value must be parsed according to section [CSS Syntax 3 § 5.3.9 Parse a comma-separated list of component values](https://www.w3.org/TR/css-syntax-3/#parse-comma-separated-list-of-component-values). Then each component value is parsed according to this grammar:

<a id="ref-for-url-value"></a>

<a id="ref-for-font-format-values"></a>

<a id="ref-for-font-tech-values"></a>

<a id="ref-for-font-face-name-value"></a>

```text
<url> [ format(<font-format>)]? [ techn( <font-tech>#)]? | local(<font-face-name>)
```
<a id="font-format-values"></a>

<a id="ref-for-string-value"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-comb-one①②"></a>

```text
<font-format>= [<string> | collection | embedded-opentype | opentype
 | svg | truetype | woff | woff2 ]
```
<a id="font-tech-values"></a>

<a id="ref-for-font-feature-tech-values"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-color-font-tech-values"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-comb-one①⑥"></a>

```text
<font-tech>= [<font-feature-tech> | <color-font-tech>
 | variations | palettes | incremental ]
```
<a id="font-feature-tech-values"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-comb-one①⑧"></a>

```text
<font-feature-tech>= [feature-opentype | feature-aat | feature-graphite]
```
<a id="color-font-tech-values"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-comb-one②②"></a>

```text
<color-font-tech>= [color-COLRv0 | color-COLRv1 | color-SVG | color-sbix | color-CBDT ]
```
If a component value is parsed correctly and is a supported [CSS Fonts 4 § 11.2 Font formats](https://drafts.csswg.org/css-fonts-4/#font-format-definitions) or [CSS Fonts 4 § 11.1 Font tech](https://drafts.csswg.org/css-fonts-4/#font-tech-definitions), add it to the list of supported sources. If parsing a component value results in a parsing error or its format or tech are unsupported, do not add it to the list of supported sources.

<a id="ref-for-descdef-font-face-src③"></a>

If there are no supported entries at the end of this process, the value for the [src](https://www.w3.org/TR/css-fonts-4/#descdef-font-face-src) descriptor is a parse error.

These parsing rules allow for graceful fallback of fonts for user agents which don’t support a particular font tech or font format.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="ex-incremental"></a> For example, when incremental transfer is not supported, a woff2 compressed version of the font is supplied, for optimal performance. Then, for incremental transfer using the [range-request method](https://www.w3.org/TR/2020/NOTE-PFE-evaluation-20201015/#range-request), the raw uncompressed OpenType font is provided so that the client can perform byte range requests.
>
> ```text
> @font-face {
>   font-family: "MyIncrementallyLoadedWebFont";
>   src: url("FallbackURLForBrowsersWhichDontSupportIncrementalLoading.woff2") format("woff2");
>   src: url("MyIncrementallyLoadedWebFont.otf") format(opentype)  tech(incremental);
> }
> ```
<a id="ref-for-descdef-font-face-font-size"></a>

### <a id="font-size-desc"></a>3.3.  Font property descriptors: the [font-size](#descdef-font-face-font-size) 

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9⑥"></a>[w3c/csswg-drafts/806](https://github.com/w3c/csswg-drafts/issues/806)[\[css-fonts-5\] Add font-size descriptor to @font-face which allows ranges (for optical sizing)](https://github.com/w3c/csswg-drafts/issues/806)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9⑦"></a>[w3c/csswg-drafts/731](https://github.com/w3c/csswg-drafts/issues/731)[\[css-fonts\] font-size Descriptor for ex Unit](https://github.com/w3c/csswg-drafts/issues/731)



| Field               | Definition                                                                                                                                                                                                                                     |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-font-face-font-size"></a>font-size                                                                                                                                                                                                                   |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-font-face-rule②"></a>[@font-face](#at-font-face-rule)                                                                                                                                                                                            |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-mult-num-range"></a><a id="ref-for-number-value④"></a><a id="ref-for-comb-one②③"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) \[[\<number\>](https://www.w3.org/TR/css-values-4/#number-value)\][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong>Initial:&#xA;      </strong> | normal                                                                                                                                                                                                                                         |



<a id="valdef-font-face-font-size-auto"></a>auto

The font matches any font size

<a id="ref-for-number-value⑤"></a>

<a id="valdef-font-face-font-size-number"></a>[\<number\>](https://www.w3.org/TR/css-values-4/#number-value)

<a id="ref-for-number-value⑥"></a>

If a single [\<number\>](https://www.w3.org/TR/css-values-4/#number-value) is given the font matches that specific font size, only. If two <a id="ref-for-number-value⑦"></a>\<number\>s are given, they specify a range of font sizes which will match.

<a id="ref-for-descdef-font-face-size-adjust③"></a>

### <a id="size-adjust-desc"></a>3.4.  Glyph Size Multiplier: the [size-adjust](#descdef-font-face-size-adjust) descriptor



| Field               | Definition                                                                                        |
|---------------------|---------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-font-face-size-adjust"></a>size-adjust                                                                    |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-font-face-rule③"></a>[@font-face](#at-font-face-rule)                                               |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-percentage-value"></a>[\<percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#percentage-value) |
| <strong>Initial:&#xA;      </strong> | 100%                                                                                              |



<a id="ref-for-descdef-font-face-size-adjust④"></a>

<a id="ref-for-propdef-font-size⑤"></a>

The [size-adjust](#descdef-font-face-size-adjust) descriptor defines a multiplier for glyph outlines and metrics associated with this font, to allow the author to harmonize the designs of various fonts when rendered at the same [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size).

<a id="ref-for-at-font-face-rule④"></a>

<a id="ref-for-ex"></a>

<a id="ref-for-ch"></a>

<a id="ref-for-valdef-text-decoration-thickness-from-font"></a>

<a id="ref-for-propdef-text-decoration-thickness"></a>

<a id="ref-for-computed-value③"></a>

<a id="ref-for-propdef-font-size⑥"></a>

<a id="ref-for-em"></a>

<a id="ref-for-propdef-text-underline-offset"></a>

All metrics associated with this font—including glyph advances, baseline tables, and overrides provided by [@font-face](#at-font-face-rule) descriptors—are scaled by the given percentage, as are the rendered glyph images. Consequently, any values derived from font metrics (such as [ex](https://www.w3.org/TR/css-values-4/#ex) and [ch](https://www.w3.org/TR/css-values-4/#ch) units, or the [from-font](https://drafts.csswg.org/css-text-decor-4/#valdef-text-decoration-thickness-from-font) value of [text-decoration-thickness](https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-thickness)) are also affected when sourced from this font. However, the [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size) (and thus any values that derive from it, such as [em](https://www.w3.org/TR/css-values-4/#em) units, percentages in [text-underline-offset](https://drafts.csswg.org/css-text-decor-4/#propdef-text-underline-offset), etc.) remains unaffected.

<a id="ref-for-descdef-font-face-size-adjust⑤"></a>

<a id="ref-for-propdef-font-size-adjust①④"></a>

<a id="ref-for-computed-value④"></a>

<a id="ref-for-propdef-font-size⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [size-adjust](#descdef-font-face-size-adjust) descriptor functions similarly to the [font-size-adjust](#propdef-font-size-adjust) property, which essentially calculates an adjustment per font by matching ex heights, but likewise does not affect the [computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size).

<a id="ref-for-descdef-font-face-ascent-override"></a>

<a id="ref-for-descdef-font-face-descent-override"></a>

<a id="ref-for-descdef-font-face-line-gap-override"></a>

### <a id="font-metrics-override-desc"></a>3.5.  Line Height Font Metrics Overrides: the [ascent-override](#descdef-font-face-ascent-override), [descent-override](#descdef-font-face-descent-override), and [line-gap-override](#descdef-font-face-line-gap-override) descriptors



| Field               | Definition                                                                                                                                                                                                                                                         |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-font-face-ascent-override"></a>ascent-override                                                                                                                                                                                                                                 |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-font-face-rule⑤"></a>[@font-face](#at-font-face-rule)                                                                                                                                                                                                                |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-mult-num-range①"></a><a id="ref-for-percentage-value①"></a><a id="ref-for-comb-one②④"></a>\[ normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#percentage-value) \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong>Initial:&#xA;      </strong> | normal                                                                                                                                                                                                                                                             |





| Field               | Definition                                                                                                                                                                                                                                                         |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-font-face-descent-override"></a>descent-override                                                                                                                                                                                                                                |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-font-face-rule⑥"></a>[@font-face](#at-font-face-rule)                                                                                                                                                                                                                |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-mult-num-range②"></a><a id="ref-for-percentage-value②"></a><a id="ref-for-comb-one②⑤"></a>\[ normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#percentage-value) \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong>Initial:&#xA;      </strong> | normal                                                                                                                                                                                                                                                             |





| Field               | Definition                                                                                                                                                                                                                                                         |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-font-face-line-gap-override"></a>line-gap-override                                                                                                                                                                                                                               |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-font-face-rule⑦"></a>[@font-face](#at-font-face-rule)                                                                                                                                                                                                                |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-mult-num-range③"></a><a id="ref-for-percentage-value③"></a><a id="ref-for-comb-one②⑥"></a>\[ normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#percentage-value) \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong>Initial:&#xA;      </strong> | normal                                                                                                                                                                                                                                                             |



<a id="ref-for-descdef-font-face-ascent-override①"></a>

<a id="ref-for-descdef-font-face-descent-override①"></a>

<a id="ref-for-descdef-font-face-line-gap-override①"></a>

<a id="ref-for-ascent-metric"></a>

<a id="ref-for-descent-metric"></a>

<a id="ref-for-line-gap-metric"></a>

The [ascent-override](#descdef-font-face-ascent-override), [descent-override](#descdef-font-face-descent-override), and [line-gap-override](#descdef-font-face-line-gap-override) descriptors specify the [ascent metric](https://www.w3.org/TR/css-inline-3/#ascent-metric), [descent metric](https://www.w3.org/TR/css-inline-3/#descent-metric), and [line gap metric](https://www.w3.org/TR/css-inline-3/#line-gap-metric) of the font, respectively. The first value provides the value for the x axis, and the second value provides the value for the y axis (defaulting to normal if omitted).

<a id="valdef-ascent-overridedescriptor-normal"></a>normal

The corresponding metric value is obtained from the font as usual, as if this descriptor were absent from the `@font-face` block.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since there are multiple sources of such metrics in some font formats, this can result in text layout that varies across UAs/platforms.

<a id="ref-for-percentage-value④"></a>

<a id="valdef-ascent-overridedescriptor-percentage"></a>[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)

The corresponding metric is replaced by the given percentage multiplied by the used font size.

<a id="ref-for-propdef-font-size-adjust①⑤"></a>

<a id="ref-for-descdef-font-face-size-adjust⑥"></a>

The [font-size-adjust](#propdef-font-size-adjust) property is applied after the [size-adjust](#descdef-font-face-size-adjust) descriptor.

<a id="ref-for-propdef-font-size-adjust①⑥"></a>

<a id="ref-for-descdef-font-face-size-adjust⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The consequence of applying [font-size-adjust](#propdef-font-size-adjust) after [size-adjust](#descdef-font-face-size-adjust) is that <a id="ref-for-descdef-font-face-size-adjust⑧"></a>size-adjust appears to have no effect.

<a id="ref-for-computed-value⑤"></a>

<a id="ref-for-propdef-font-size⑧"></a>

<a id="ref-for-propdef-line-height④"></a>

<a id="ref-for-font-relative-length"></a>

<a id="ref-for-inline-level"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: None of these descriptors affect the [computation](https://www.w3.org/TR/css-cascade-5/#computed-value) of [font-size](https://www.w3.org/TR/css-fonts-4/#propdef-font-size), [line-height](https://www.w3.org/TR/CSS2/visudet.html#propdef-line-height), or [font-relative lengths](https://www.w3.org/TR/css-values-4/#font-relative-length). They can, however, affect the behavior of <a id="ref-for-propdef-line-height⑤"></a>line-height: normal and more generally the baseline alignment of [inline-level](https://www.w3.org/TR/css-display-3/#inline-level) content.

<a id="ref-for-block-axis"></a>

<a id="ref-for-typeset-upright"></a>

<a id="ref-for-typographic-mode"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since these metrics are only applicable in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis), the y-axis value will only be used when [typesetting upright](https://drafts.csswg.org/css-writing-modes-4/#typeset-upright) in vertical [typographic modes](https://www.w3.org/TR/css-writing-modes-4/#typographic-mode).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-7315a3e2"></a> The percentage is resolved against different font sizes for different elements.
>
> ```text
> @font-face {
>   font-family: overridden-font;
>   ascent-override: 50%;
>   ...
> }
> 
> <span style="font-family: overridden-font; font-size: 20px;">
>   Outer span content
>   <span style="font-size: 150%;">Inner span content</span>
> </span>
> ```
>
> <a id="ref-for-ascent-metric①"></a>
>
> The outer span uses an [ascent](https://www.w3.org/TR/css-inline-3/#ascent-metric) value of 10px, whereas the inner span uses 15px.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1e35741d"></a> We may override the metrics of a local fallback font to match the primary font, which is a web font. This reduces layout shifting when switching from fallback to the primary font.
>
> ```text
> @font-face {
>   font-family: cool-web-font;
>   src: url("https://example.com/font.woff");
> }
> 
> @font-face {
>   font-family: fallback-to-local;
>   src: local(Some Local Font);
>   /* Override metric values to match cool-web-font */
>   ascent-override: 125%;
>   descent-override: 25%;
>   line-gap-override: 0%;
>   size-adjust: 96%;
> }
> 
> <div style="font-family: cool-web-font, fallback-to-local">Title goes here</div>
> <img src="https://example.com/largeimage" alt="A large image that you don’t want to shift">
> ```
>
> The image will not be shifted as much when the user agent finishes loading and switches to use the web font (assuming the override values are similar to the web font’s natural metrics).

<a id="ref-for-descdef-font-face-superscript-position-override"></a>

<a id="ref-for-descdef-font-face-subscript-position-override"></a>

<a id="ref-for-descdef-font-face-superscript-size-override"></a>

<a id="ref-for-descdef-font-face-subscript-size-override"></a>

### <a id="font-sup-sub-override-desc"></a>3.6.  Superscript and subscript metrics overrides: the [superscript-position-override](#descdef-font-face-superscript-position-override), [subscript-position-override](#descdef-font-face-subscript-position-override),[superscript-size-override](#descdef-font-face-superscript-size-override) and [subscript-size-override](#descdef-font-face-subscript-size-override) descriptors



| Field               | Definition                                                                                                                                                                                                                                                                                 |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-font-face-superscript-position-override"></a>superscript-position-override                                                                                                                                                                                                                                           |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-font-face-rule⑧"></a>[@font-face](#at-font-face-rule)                                                                                                                                                                                                                                        |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-mult-num-range④"></a><a id="ref-for-percentage-value⑤"></a><a id="ref-for-comb-one②⑦"></a>\[ normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) from-font <a id="ref-for-comb-one②⑧"></a>\| [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong>Initial:&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                     |





| Field               | Definition                                                                                                                                                                                                                                                                                 |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-font-face-subscript-position-override"></a>subscript-position-override                                                                                                                                                                                                                                             |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-font-face-rule⑨"></a>[@font-face](#at-font-face-rule)                                                                                                                                                                                                                                        |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-mult-num-range⑤"></a><a id="ref-for-percentage-value⑥"></a><a id="ref-for-comb-one②⑨"></a>\[ normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) from-font <a id="ref-for-comb-one③⓪"></a>\| [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong>Initial:&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                     |





| Field               | Definition                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-font-face-superscript-size-override"></a>superscript-size-override                                                                                                                                                                                                                                                       |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-font-face-rule①⓪"></a>[@font-face](#at-font-face-rule)                                                                                                                                                                                                                                                |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-mult-num-range⑥"></a><a id="ref-for-percentage-value⑦"></a><a id="ref-for-comb-one③①"></a>\[ normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) from-font <a id="ref-for-comb-one③②"></a>\| [\<percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#percentage-value) \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong>Initial:&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                             |





| Field               | Definition                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="descdef-font-face-subscript-size-override"></a>subscript-size-override                                                                                                                                                                                                                                                         |
| <strong>For:&#xA;      </strong> | <a id="ref-for-at-font-face-rule①①"></a>[@font-face](#at-font-face-rule)                                                                                                                                                                                                                                                |
| <strong>Value:&#xA;      </strong> | <a id="ref-for-mult-num-range⑦"></a><a id="ref-for-percentage-value⑧"></a><a id="ref-for-comb-one③③"></a>\[ normal [\|](https://www.w3.org/TR/css-values-4/#comb-one) from-font <a id="ref-for-comb-one③④"></a>\| [\<percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#percentage-value) \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong>Initial:&#xA;      </strong> | normal                                                                                                                                                                                                                                                                                             |



<a id="ref-for-descdef-font-face-superscript-position-override①"></a>

<a id="ref-for-descdef-font-face-subscript-position-override①"></a>

<a id="ref-for-descdef-font-face-superscript-size-override①"></a>

<a id="ref-for-descdef-font-face-subscript-size-override①"></a>

<a id="ref-for-propdef-font-variant-position"></a>

The [superscript-position-override](#descdef-font-face-superscript-position-override), [subscript-position-override](#descdef-font-face-subscript-position-override), [superscript-size-override](#descdef-font-face-superscript-size-override), and [subscript-size-override](#descdef-font-face-subscript-size-override) descriptors specify the superscript offset, subscript offset, superscript size, and subscript size metrics of the font, respectively, which are used to synthesize glyphs when required by [font-variant-position](https://www.w3.org/TR/css-fonts-4/#propdef-font-variant-position). The first value provides the value for the x axis, and the second value provides the value for the y axis (defaulting to the first value if omitted).

<a id="valdef-superscript-position-overridedescriptor-normal"></a>normal

The UA determines what metrics value to use, whether derived from the font or from some heuristic.

<a id="valdef-superscript-position-overridedescriptor-from-font"></a>from-font

The corresponding metric in the font data is used, if any. (If the metric is missing, same as normal.)

<a id="ref-for-percentage-value⑨"></a>

<a id="valdef-superscript-position-overridedescriptor-percentage"></a>[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)

The corresponding metric is replaced by the given percentage multiplied by the used font size.

<a id="ref-for-block-axis①"></a>

<a id="ref-for-typeset-upright①"></a>

<a id="ref-for-vertical-typographic-mode"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Since these metrics are only applicable in the [block axis](https://www.w3.org/TR/css-writing-modes-4/#block-axis), the y-axis value will only be used when [typesetting upright](https://drafts.csswg.org/css-writing-modes-4/#typeset-upright) in [vertical typographic modes](https://drafts.csswg.org/css-writing-modes-4/#vertical-typographic-mode).

## <a id="font-rend-props"></a>4.  Font Feature Properties

<a id="ref-for-descdef-font-face-font-language-override"></a>

### <a id="font-language-override-prop"></a>4.1. Font language override: the [font-language-override](https://www.w3.org/TR/css-fonts-4/#descdef-font-face-font-language-override) property

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9⑧"></a>[w3c/csswg-drafts/5484](https://github.com/w3c/csswg-drafts/issues/5484)[\[css-fonts-5\] Removing font-language-override](https://github.com/w3c/csswg-drafts/issues/5484)

## <a id="font-feature-variation-resolution"></a>5.  Font Feature and Variation Resolution

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9⑨"></a>[w3c/csswg-drafts/5635](https://github.com/w3c/csswg-drafts/issues/5635)[\[CSS-Fonts\] Need method to interpolate variable font settings](https://github.com/w3c/csswg-drafts/issues/5635)

## <a id="font-variation-props"></a>6.  Font Variation Properties

<a id="ref-for-propdef-font-optical-sizing"></a>

### <a id="font-optical-sizing-def"></a>6.1.  Optical sizing control: the [font-optical-sizing](https://www.w3.org/TR/css-fonts-4/#propdef-font-optical-sizing) property

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-d41d8cd9①⓪"></a>[w3c/csswg-drafts/5466](https://github.com/w3c/csswg-drafts/issues/5466)[\[css-text\]\[css-fonts\] Optical bounds of a line](https://github.com/w3c/csswg-drafts/issues/5466)

## <a id="security"></a>7. Security Considerations

No new security considerations have been reported on this specification.

## <a id="privacy"></a>8. Privacy Considerations

No new privacy considerations have been reported on this specification.

## <a id="acknowledgments"></a>9.  Acknowledgments

Firstly, the editors would like to thank all of the [contributors to the previous level of this module](https://www.w3.org/TR/css-fonts-4/#acknowledgments).

Secondly, we would like to acknowledge DerKoun from PDFReactor, Xiaocheng Hu from Google, and Jonathan Kew from Mozilla, for their contributions to the improvements in this Level 5.

## <a id="changes"></a>10.  Changes 

### <a id="changes-20210729"></a>10.1. Changes since the [WD of 2021-07-29](https://www.w3.org/TR/2021/WD-css-fonts-5-20210729/)

- Renamed technology to tech
- Security and Privacy are now separate sections
- Fixed font-technology grammar
- Copied section on parsing src descriptor from CSS Fonts 4

### <a id="changes-20210629"></a>10.2. Changes since the [FPWD of 2021-06-29](https://www.w3.org/TR/2021/WD-css-fonts-5-20210629/)

- Removed scary warning notice
- Added incremental font technology to supports

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

- [ascent-override](#descdef-font-face-ascent-override), in § 3.5
- [aspect value](#font-size-adjust-aspect-value), in § 2.4
- [auto](#valdef-font-face-font-size-auto), in § 3.3
- [cap-height](#valdef-font-size-adjust-cap-height), in § 2.4
- [ch-width](#valdef-font-size-adjust-ch-width), in § 2.4
- [\<color-font-tech\>](#color-font-tech-values), in § 3.2.1
- [descent-override](#descdef-font-face-descent-override), in § 3.5
- [ex-height](#valdef-font-size-adjust-ex-height), in § 2.4
- [ex-height \| cap-height \| ch-width \| ic-width \| ic-height](#valdef-font-size-adjust-ex-height--cap-height--ch-width--ic-width--ic-height), in § 2.4
- [@font-face](#at-font-face-rule), in § 3.1
- [\<font-feature-tech\>](#font-feature-tech-values), in § 3.2.1
- [\<font-format\>](#font-format-values), in § 3.2.1
- [font-size](#descdef-font-face-font-size), in § 3.3
- [font-size-adjust](#propdef-font-size-adjust), in § 2.4
- [\<font-tech\>](#font-tech-values), in § 3.2.1
- from-font
  - [value for font-size-adjust](#valdef-font-size-adjust-from-font), in § 2.4
  - [value for superscript-position-override!!descriptor, subscript-position-override!!descriptor, superscript-size-override!!descriptor, subscript-size-override!!descriptor](#valdef-superscript-position-overridedescriptor-from-font), in § 3.6
- [ic-height](#valdef-font-size-adjust-ic-height), in § 2.4
- [ic-width](#valdef-font-size-adjust-ic-width), in § 2.4
- [line-gap-override](#descdef-font-face-line-gap-override), in § 3.5
- [none](#valdef-font-size-adjust-none), in § 2.4
- normal
  - [value for ascent-override!!descriptor, descent-override!!descriptor, line-gap-override!!descriptor](#valdef-ascent-overridedescriptor-normal), in § 3.5
  - [value for superscript-position-override!!descriptor, subscript-position-override!!descriptor, superscript-size-override!!descriptor, subscript-size-override!!descriptor](#valdef-superscript-position-overridedescriptor-normal), in § 3.6
- \<number\>
  - [value for @font-face/font-size](#valdef-font-face-font-size-number), in § 3.3
  - [value for font-size-adjust](#valdef-font-size-adjust-number), in § 2.4
- \<percentage\>
  - [value for ascent-override!!descriptor, descent-override!!descriptor, line-gap-override!!descriptor](#valdef-ascent-overridedescriptor-percentage), in § 3.5
  - [value for superscript-position-override!!descriptor, subscript-position-override!!descriptor, superscript-size-override!!descriptor, subscript-size-override!!descriptor](#valdef-superscript-position-overridedescriptor-percentage), in § 3.6
- [size-adjust](#descdef-font-face-size-adjust), in § 3.4
- [subscript-position-override](#descdef-font-face-subscript-position-override), in § 3.6
- [subscript-size-override](#descdef-font-face-subscript-size-override), in § 3.6
- [superscript-position-override](#descdef-font-face-superscript-position-override), in § 3.6
- [superscript-size-override](#descdef-font-face-superscript-size-override), in § 3.6
- [xxx](#valdef-font-family-xxx), in § 2.1.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[css-cascade-5\] defines the following terms:
  - <a id="term-for-computed-value"></a>computed value
  - <a id="term-for-used-value"></a>used value
- \[css-display-3\] defines the following terms:
  - <a id="term-for-inline-level"></a>inline-level
- \[css-fonts-3\] defines the following terms:
  - <a id="term-for-font-face-name-value"></a>\<font-face-name\>
- \[CSS-FONTS-4\] defines the following terms:
  - <a id="term-for-first-available-font"></a>first available font
  - <a id="term-for-propdef-font-family"></a>font-family
  - <a id="term-for-descdef-font-face-font-language-override"></a>font-language-override
  - <a id="term-for-propdef-font-optical-sizing"></a>font-optical-sizing
  - <a id="term-for-propdef-font-size"></a>font-size
  - <a id="term-for-propdef-font-style"></a>font-style
  - <a id="term-for-propdef-font-variant-position"></a>font-variant-position
  - <a id="term-for-propdef-font-weight"></a>font-weight
  - <a id="term-for-descdef-font-face-src"></a>src
- \[css-inline-3\] defines the following terms:
  - <a id="term-for-ascent-metric"></a>ascent metric
  - <a id="term-for-descent-metric"></a>descent metric
  - <a id="term-for-line-gap-metric"></a>line gap metric
- \[css-text-decor-4\] defines the following terms:
  - <a id="term-for-valdef-text-decoration-thickness-from-font"></a>from-font
  - <a id="term-for-propdef-text-decoration-thickness"></a>text-decoration-thickness
  - <a id="term-for-propdef-text-underline-offset"></a>text-underline-offset
- \[css-values-4\] defines the following terms:
  - <a id="term-for-number-value"></a>\<number\>
  - <a id="term-for-percentage-value"></a>\<percentage\>
  - <a id="term-for-string-value"></a>\<string\>
  - <a id="term-for-url-value"></a>\<url\>
  - <a id="term-for-mult-opt"></a>?
  - <a id="term-for-ch"></a>ch
  - <a id="term-for-css-wide-keywords"></a>css-wide keywords
  - <a id="term-for-em"></a>em
  - <a id="term-for-ex"></a>ex
  - <a id="term-for-font-relative-length"></a>font-relative lengths
  - <a id="term-for-mult-num-range"></a>{a,b}
  - <a id="term-for-comb-one"></a>\|
- \[css-writing-modes-4\] defines the following terms:
  - <a id="term-for-block-axis"></a>block axis
  - <a id="term-for-typeset-upright"></a>typesetting upright
  - <a id="term-for-typographic-mode"></a>typographic mode
  - <a id="term-for-vertical-typographic-mode"></a>vertical typographic mode
- \[CSS2\] defines the following terms:
  - <a id="term-for-propdef-line-height"></a>line-height

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 3 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-fonts-3"></a>\[CSS-FONTS-3\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 3](https://www.w3.org/TR/css-fonts-3/). 20 September 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-3&#x2F;](https://www.w3.org/TR/css-fonts-3/)

<a id="biblio-css-fonts-4"></a>\[CSS-FONTS-4\]  
John Daggett; Myles Maxfield; Chris Lilley. [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/). 29 July 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-fonts-4&#x2F;](https://www.w3.org/TR/css-fonts-4/)

<a id="biblio-css-inline-3"></a>\[CSS-INLINE-3\]  
Dave Cramer; Elika Etemad; Steve Zilles. [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/). 27 August 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-inline-3&#x2F;](https://www.w3.org/TR/css-inline-3/)

<a id="biblio-css-text-decor-4"></a>\[CSS-TEXT-DECOR-4\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 4](https://www.w3.org/TR/css-text-decor-4/). 6 May 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-4&#x2F;](https://www.w3.org/TR/css-text-decor-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 6 June 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 16 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 3 September 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-writing-modes-4"></a>\[CSS-WRITING-MODES-4\]  
Elika Etemad; Koji Ishii. [CSS Writing Modes Level 4](https://www.w3.org/TR/css-writing-modes-4/). 30 July 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-writing-modes-4&#x2F;](https://www.w3.org/TR/css-writing-modes-4/)

## <a id="property-index"></a>Property Index



| Name                | Value                                                                                                     | Initial | Applies to            | Inh. | %ages | Anim­ation type                                                    | Canonical order | Com­puted value                                                   |
|---------------------|-----------------------------------------------------------------------------------------------------------|---------|-----------------------|------|-------|-------------------------------------------------------------------|-----------------|------------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-font-size-adjust①⑦"></a></span><a href="#propdef-font-size-adjust">font-size-adjust</a>&#xA;      </strong> | none \| \[ ex-height \| cap-height \| ch-width \| ic-width \| ic-height \]? \[ from-font \| \<number\> \] | none    | all elements and text | yes  | N/A   | discrete if the keywords differ, otherwise by computed value type | per grammar     | the keyword none, or a pair of a metric keyword and a \<number\> |



<a id="ref-for-at-font-face-rule①②"></a>

### <a id="font-face-descriptor-table"></a>[@font-face](#at-font-face-rule) Descriptors



| Name                | Value                                                    | Initial |
|---------------------|----------------------------------------------------------|---------|
| <strong><span><a id="ref-for-descdef-font-face-ascent-override②"></a></span><a href="#descdef-font-face-ascent-override">ascent-override</a>&#xA;      </strong> | \[ normal \| \<percentage \[0,∞\]\> \]{1,2}              | normal  |
| <strong><span><a id="ref-for-descdef-font-face-descent-override②"></a></span><a href="#descdef-font-face-descent-override">descent-override</a>&#xA;      </strong> | \[ normal \| \<percentage \[0,∞\]\> \]{1,2}              | normal  |
| <strong><span><a id="ref-for-descdef-font-face-font-size①"></a></span><a href="#descdef-font-face-font-size">font-size</a>&#xA;      </strong> | auto \| \[\<number\>\]{1,2}                              | normal  |
| <strong><span><a id="ref-for-descdef-font-face-line-gap-override②"></a></span><a href="#descdef-font-face-line-gap-override">line-gap-override</a>&#xA;      </strong> | \[ normal \| \<percentage \[0,∞\]\> \]{1,2}              | normal  |
| <strong><span><a id="ref-for-descdef-font-face-size-adjust⑨"></a></span><a href="#descdef-font-face-size-adjust">size-adjust</a>&#xA;      </strong> | \<percentage \[0,∞\]\>                                   | 100%    |
| <strong><span><a id="ref-for-descdef-font-face-subscript-position-override②"></a></span><a href="#descdef-font-face-subscript-position-override">subscript-position-override</a>&#xA;      </strong> | \[ normal \| from-font \| \<percentage\> \]{1,2}         | normal  |
| <strong><span><a id="ref-for-descdef-font-face-subscript-size-override②"></a></span><a href="#descdef-font-face-subscript-size-override">subscript-size-override</a>&#xA;      </strong> | \[ normal \| from-font \| \<percentage \[0,∞\]\> \]{1,2} | normal  |
| <strong><span><a id="ref-for-descdef-font-face-superscript-position-override②"></a></span><a href="#descdef-font-face-superscript-position-override">superscript-position-override</a>&#xA;      </strong> | \[ normal \| from-font \| \<percentage\> \]{1,2}         | normal  |
| <strong><span><a id="ref-for-descdef-font-face-superscript-size-override②"></a></span><a href="#descdef-font-face-superscript-size-override">superscript-size-override</a>&#xA;      </strong> | \[ normal \| from-font \| \<percentage \[0,∞\]\> \]{1,2} | normal  |



## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/126](https://github.com/w3c/csswg-drafts/issues/126)[\[css-fonts\] Specifying changes to parameters for fallback fonts](https://github.com/w3c/csswg-drafts/issues/126) [↵](#issue-d41d8cd9)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/4910](https://github.com/w3c/csswg-drafts/issues/4910)[\[meta\] \[css-fonts\] Criteria for generic font families](https://github.com/w3c/csswg-drafts/issues/4910) [↵](#issue-d41d8cd9%E2%91%A0)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/5054](https://github.com/w3c/csswg-drafts/issues/5054)[\[css-fonts\] generic family for Pi, Picture, Symbols fonts](https://github.com/w3c/csswg-drafts/issues/5054) [↵](#issue-d41d8cd9%E2%91%A1)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/4566](https://github.com/w3c/csswg-drafts/issues/4566)[\[css-fonts\] Should we start a registry for additional generic fonts?](https://github.com/w3c/csswg-drafts/issues/4566) [↵](#issue-d41d8cd9%E2%91%A2)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/2690](https://github.com/w3c/csswg-drafts/issues/2690)[\[css-fonts-4\] Percentages in font-weight for relative weights](https://github.com/w3c/csswg-drafts/issues/2690) [↵](#issue-d41d8cd9%E2%91%A3)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/4044](https://github.com/w3c/csswg-drafts/issues/4044)[\[css-fonts\] Vertical text doesn't play nicely with font-style and font-stretch](https://github.com/w3c/csswg-drafts/issues/4044) [↵](#issue-d41d8cd9%E2%91%A4)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/6384](https://github.com/w3c/csswg-drafts/issues/6384)[\[css-fonts-5\] font-size-adjust with missing metrics](https://github.com/w3c/csswg-drafts/issues/6384) [↵](#issue-c72cd359)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/6384](https://github.com/w3c/csswg-drafts/issues/6384)[\[css-fonts-5\] font-size-adjust with missing metrics](https://github.com/w3c/csswg-drafts/issues/6384) [↵](#issue-36eece46)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/806](https://github.com/w3c/csswg-drafts/issues/806)[\[css-fonts-5\] Add font-size descriptor to @font-face which allows ranges (for optical sizing)](https://github.com/w3c/csswg-drafts/issues/806) [↵](#issue-d41d8cd9%E2%91%A5)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/731](https://github.com/w3c/csswg-drafts/issues/731)[\[css-fonts\] font-size Descriptor for ex Unit](https://github.com/w3c/csswg-drafts/issues/731) [↵](#issue-d41d8cd9%E2%91%A6)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/5484](https://github.com/w3c/csswg-drafts/issues/5484)[\[css-fonts-5\] Removing font-language-override](https://github.com/w3c/csswg-drafts/issues/5484) [↵](#issue-d41d8cd9%E2%91%A7)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/5635](https://github.com/w3c/csswg-drafts/issues/5635)[\[CSS-Fonts\] Need method to interpolate variable font settings](https://github.com/w3c/csswg-drafts/issues/5635) [↵](#issue-d41d8cd9%E2%91%A8)

> <strong data-conversion-semantic="issue">Issue</strong>
>
> [w3c/csswg-drafts/5466](https://github.com/w3c/csswg-drafts/issues/5466)[\[css-text\]\[css-fonts\] Optical bounds of a line](https://github.com/w3c/csswg-drafts/issues/5466) [↵](#issue-d41d8cd9%E2%91%A0%E2%93%AA)
