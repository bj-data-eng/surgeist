Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/).

Original copyright notice: Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Backgrounds and Borders Module Level 3

Source snapshot: https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/

Snapshot SHA-256: 8a291792b2fd351eee443df466626d02b889d890f5d80e315bf80181b408712a

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 29 source tables are presented as readable Markdown tables or explicit labeled layouts: 27 ordinary table conversions, 2 complex-table layouts. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Backgrounds and Borders Module Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2024 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

This draft contains the features of CSS relating to borders and backgrounds. The main extensions compared to [level 2](https://www.w3.org/TR/CSS2/) are borders consisting of images, boxes with multiple backgrounds, boxes with rounded corners and boxes with shadows.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https://www.w3.org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/2023/Process-20231103/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-backgrounds” in the title, like this: “\[css-backgrounds\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-backgrounds%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/2023/Process-20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-propdef-box-shadow"></a>

  animatability of [box-shadow](#propdef-box-shadow)

- <a id="ref-for-ruby-annotation-container-box"></a>

  <a id="ref-for-ruby-base-container-box"></a>

  <a id="ref-for-propdef-border"></a>

  applicability of [border](#propdef-border) and its longhands to [ruby base containers](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and [ruby annotation containers](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-container-box)

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="introduction"></a>1. Introduction

<em>This subsection is not normative.</em>

<a id="ref-for-content-area"></a>

<a id="ref-for-padding"></a>

<a id="ref-for-border"></a>

<a id="ref-for-margin"></a>

When elements are rendered according to the [CSS box model](https://www.w3.org/TR/css-box-3/#box-model) [\[CSS-BOX-3\]](#biblio-css-box-3), each element is either not displayed at all, or formatted as one or more rectangular boxes. Each box has a rectangular [content area](https://www.w3.org/TR/css-box-4/#content-area), a band of [padding](https://www.w3.org/TR/css-box-4/#padding) around the content, a [border](https://www.w3.org/TR/css-box-4/#border) around the padding, and a [margin](https://www.w3.org/TR/css-box-4/#margin) outside the border. (The margin may actually be negative, but margins have no influence on the background and border.)

![Diagram of a typical box, showing the content, padding, border and margin areas](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/box.png)

The various areas and edges of a typical box. (This diagram is explained in the CSS Box Model Module [\[CSS-BOX-3\]](#biblio-css-box-3).)

<a id="ref-for-border-area"></a>

<a id="ref-for-content-area①"></a>

<a id="ref-for-padding-area"></a>

<a id="ref-for-propdef-box-shadow①"></a>

The properties of this module deal with the decoration of the [border area](https://www.w3.org/TR/css-box-4/#border-area) and with the background of the [content](https://www.w3.org/TR/css-box-4/#content-area), [padding](https://www.w3.org/TR/css-box-4/#padding-area), and <a id="ref-for-border-area①"></a>border areas. Additionally the box may be given a “drop-shadow” effect with the [box-shadow](#propdef-box-shadow) property.

<a id="ref-for-box-fragment"></a>

<a id="ref-for-propdef-box-decoration-break"></a>

If an element is broken into multiple [box fragments](https://www.w3.org/TR/css-break-4/#box-fragment), [box-decoration-break](https://www.w3.org/TR/css-break-4/#propdef-box-decoration-break) [\[CSS-BREAK-3\]](#biblio-css-break-3) defines how the borders and background are divided over the various fragments. (An element can result in more than one fragment if it is broken at the end of a line, at the end of a column or at the end of a page; and continued in the next line, column or page.)

The relative stacking order of backgrounds, borders, and shadows is given in this module. For how these layers interact with other rendered content, see Appendix E “Elaborate description of Stacking Contexts” in [\[CSS2\]](#biblio-css2).

### <a id="placement"></a>1.1.  Module Interactions

This module replaces and extends the background and border features defined in [\[CSS2\]](#biblio-css2) sections 8.5 and 14.2.

<a id="ref-for-first-letter0"></a>

<a id="ref-for-pseudo-element"></a>

<a id="ref-for-sel-first-line"></a>

<a id="ref-for-propdef-border-image"></a>

<a id="ref-for-propdef-box-shadow②"></a>

All properties in this module apply to the [::first-letter](https://www.w3.org/TR/selectors-3/#first-letter0) [pseudo-element](https://www.w3.org/TR/selectors-4/#pseudo-element). The [background properties](#backgrounds) and [border-radius properties](#corners) also apply to the [::first-line](https://www.w3.org/TR/selectors-3/#sel-first-line) <a id="ref-for-pseudo-element①"></a>pseudo-element. The UA may (but is not required to) apply the [border-image](#propdef-border-image) or [box-shadow](#propdef-box-shadow) properties to <a id="ref-for-sel-first-line①"></a>::first-line. The UA must not apply the [border-color/style/width properties](#borders) to <a id="ref-for-sel-first-line②"></a>::first-line. [\[CSS2\]](#biblio-css2)

### <a id="values"></a>1.2.  Value Definitions

<a id="ref-for-propdef-background-image"></a>

<a id="ref-for-propdef-border-image①"></a>

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2) using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Values &#x26; Units \[CSS-VALUES-3\]. Combination with other CSS modules may expand the definitions of these value types. For example, combining with [CSS Images](https://www.w3.org/TR/css-images/) allows for using CSS gradients as [background-image](#propdef-background-image) or [border-image](#propdef-border-image) values. [\[CSS-IMAGES-3\]](#biblio-css-images-3)

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

## <a id="backgrounds"></a>2.  Backgrounds

<a id="ref-for-propdef-background-color"></a>

<a id="ref-for-propdef-background-image①"></a>

Each box has a background layer that may be fully transparent (the default), or filled with a color and/or one or more images. The background properties specify what color ([background-color](#propdef-background-color)) and images ([background-image](#propdef-background-image)) to use, and how they are sized, positioned, tiled, etc.

<a id="ref-for-valdef-color-transparent"></a>

<a id="ref-for-propdef-background-color①"></a>

The background properties are not inherited, but the parent box’s background will shine through by default because of the initial [transparent](https://www.w3.org/TR/css-color-4/#valdef-color-transparent) value on [background-color](#propdef-background-color).

### <a id="layering"></a>2.1.  Layering Multiple Background Images

<a id="ref-for-propdef-background-image②"></a>

<a id="ref-for-valdef-background-image-none"></a>

The background of a box can have multiple <a id="background-image-layer"></a>background image layers. The number of layers is determined by the number of comma-separated values in the [background-image](#propdef-background-image) property. Note that a value of [none](#valdef-background-image-none) still creates a layer.

<a id="ref-for-background-images"></a>

<a id="ref-for-user-agent"></a>

<a id="ref-for-used-value"></a>

Each of the [background images](#background-images) is sized, positioned, and tiled according to the corresponding value in the other background properties. The lists are matched up from the first value: excess values at the end are not used. If a property doesn’t have enough comma-separated values to match the number of layers, the [UA](https://www.w3.org/TR/css-2023/#user-agent) must calculate its [used value](https://www.w3.org/TR/css-cascade-5/#used-value) by repeating the list of values until there are enough.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bef8b176"></a> For example, this set of declarations:
>
> ```text
> background-image: url(flower.png), url(ball.png), url(grass.png);
> background-position: center center, 20% 80%, top left, bottom right;
> background-origin: border-box, content-box;
> background-repeat: no-repeat;
> ```
>
> <a id="ref-for-propdef-background-origin"></a>
>
> <a id="ref-for-propdef-background-repeat"></a>
>
> has exactly the same effect as this set, with the extra position dropped and the missing values for [background-origin](#propdef-background-origin) and [background-repeat](#propdef-background-repeat) filled in (emphasized for clarity):
>
> ```text
> background-image: url(flower.png), url(ball.png), url(grass.png);
> background-position: center center, 20% 80%, top left;
> background-origin: border-box, content-box, border-box;
> background-repeat: no-repeat, no-repeat, no-repeat;
> ```
<a id="ref-for-background-image-layer"></a>

The first image in the list is the [layer](#background-image-layer) closest to the user, the next one is painted behind the first, and so on. The background color, if present, is painted below all of the other <a id="ref-for-background-image-layer①"></a>layers.

<a id="ref-for-background-image-layer②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [border-image properties](#border-images) can also define a background image, which, if present, is painted on top of the background [layers](#background-image-layer) created by the background properties.

<a id="ref-for-propdef-background-color②"></a>

### <a id="background-color"></a>2.2.  Base Color: the [background-color](#propdef-background-color) property<a id="the-background-color"></a>

| Field               | Definition                                                                       |
|---------------------|----------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-background-color"></a>background-color                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-color"></a>[\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | transparent                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | computed color                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                |

This property sets the <a id="background-color-layer"></a>background color of a box. This color is drawn behind any background images.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ed26f60a"></a> Example:
>
> ```text
> h1 { background-color: #F00 } /* Sets background to red. */
> ```
<a id="ref-for-background-color-layer"></a>

<a id="ref-for-propdef-background-clip"></a>

<a id="ref-for-background-image-layer③"></a>

The [background color](#background-color-layer) is clipped according to the [background-clip](#propdef-background-clip) value associated with the bottom-most [background image layer](#background-image-layer).

<a id="ref-for-propdef-background-image③"></a>

### <a id="background-image"></a>2.3.  Image Sources: the [background-image](#propdef-background-image) property<a id="the-background-image"></a>

| Field               | Definition                                                                                                                                                                          |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-background-image"></a>background-image                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma"></a><a id="ref-for-typedef-bg-image"></a>[\<bg-image\>](#typedef-bg-image)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-background-image-none①"></a><a id="ref-for-typedef-image"></a>list, each item either an [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image) or the keyword [none](#valdef-background-image-none) |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                            |

<a id="ref-for-typedef-bg-image①"></a>

This property specifies the <a id="background-images"></a>background image(s) of an element. Images are drawn with the first specified one on top (closest to the user) and each subsequent image behind the previous one. The property’s value is given as a comma-separated list of [\<bg-image\>](#typedef-bg-image) values where

<a id="typedef-bg-image"></a>

<a id="ref-for-typedef-bg-image②"></a>

<a id="ref-for-typedef-image①"></a>

<a id="ref-for-comb-one"></a>

```text
<bg-image> = <image> | none
```
<a id="ref-for-background-image-layer④"></a>

A value of <a id="valdef-background-image-none"></a>none counts as a [background image layer](#background-image-layer) but draws nothing. An image that is empty (zero width or zero height), that fails to download, or that cannot be displayed (e.g., because it is not in a supported image format) likewise counts as a <a id="ref-for-background-image-layer⑤"></a>layer but draws nothing.

<a id="ref-for-propdef-background-image④"></a>

<a id="ref-for-background-image-layer⑥"></a>

See [§ 2.1 Layering Multiple Background Images](#layering) for how [background-image](#propdef-background-image) interacts with other comma-separated background properties to form each [background image layer](#background-image-layer).

<a id="ref-for-propdef-background-color③"></a>

When setting a background image, authors should also specify a [background-color](#propdef-background-color) that will preserve contrast with the text for when the image is unavailable.

For accessibility reasons, authors should not use background images as the sole method of conveying important information. See [Web Content Accessibility Guideline F3](https://www.w3.org/TR/2008/NOTE-WCAG20-TECHS-20081211/F3) [\[WCAG20\]](#biblio-wcag20). Images are not accessible in non-graphical presentations, and background images specifically might be turned off in high-contrast display modes.

<a id="ref-for-propdef-content"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Stylistic foreground images can be provided in CSS with the [content](https://www.w3.org/TR/css-content-3/#propdef-content) property. Semantically-important foreground images should be provided in the document markup, e.g. with the \<img\> tag in HTML.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [Media fragments](https://www.w3.org/TR/media-frags/#naming-space) can be used to display a portion of an image. The [CSS Images](https://www.w3.org/TR/css-images/) module will provide fallback syntax for image formats and include additional controls for image display.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b3c5ac2d"></a> Some examples specifying background images:
>
> ```text
> html { background-image: url("marble.svg") }
> p { background-image: none }
> div { background-image: url(tl.png), url(tr.png) }
> main { background-image: radial-gradient(at bottom right, transparent, white); }
> ```
Implementations may optimize by not downloading and drawing images that are not visible (e.g., because they are behind other, fully opaque images).

<a id="ref-for-propdef-background-repeat①"></a>

### <a id="background-repeat"></a>2.4.  Tiling Images: the [background-repeat](#propdef-background-repeat) property<a id="the-background-repeat"></a>

| Field               | Definition                                                                                                                          |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-background-repeat"></a>background-repeat                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma①"></a><a id="ref-for-typedef-repeat-style"></a>[\<repeat-style\>](#typedef-repeat-style)[\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | repeat                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | list, each item a pair of keywords, one per dimension                                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                            |

<a id="ref-for-background-images①"></a>

<a id="ref-for-typedef-repeat-style①"></a>

This property specifies how [background images](#background-images) are tiled after they have been [sized](#background-size) and [positioned](#background-position). The property’s value is given as a comma-separated list of [\<repeat-style\>](#typedef-repeat-style) values where

<a id="typedef-repeat-style"></a>

<a id="ref-for-typedef-repeat-style②"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-mult-num-range"></a>

```text
<repeat-style> = repeat-x | repeat-y | [repeat | space | round | no-repeat]{1,2}
```
<a id="ref-for-typedef-repeat-style③"></a>

Single values for [\<repeat-style\>](#typedef-repeat-style) have the following meanings:

<a id="valdef-background-repeat-repeat-x"></a>repeat-x

Computes to repeat no-repeat.

<a id="valdef-background-repeat-repeat-y"></a>repeat-y

Computes to no-repeat repeat.

<a id="ref-for-valdef-background-repeat-repeat"></a>

[repeat](#valdef-background-repeat-repeat)

Computes to repeat repeat.

<a id="ref-for-valdef-background-repeat-space"></a>

[space](#valdef-background-repeat-space)

Computes to space space

<a id="ref-for-valdef-background-repeat-round"></a>

[round](#valdef-background-repeat-round)

Computes to round round

<a id="ref-for-valdef-background-repeat-no-repeat"></a>

[no-repeat](#valdef-background-repeat-no-repeat)

Computes to no-repeat no-repeat

<a id="ref-for-typedef-repeat-style④"></a>

If a [\<repeat-style\>](#typedef-repeat-style) value has two keywords, the first one applies to the horizontal axis, the second to the vertical one, as follows:

<a id="valdef-background-repeat-repeat"></a>repeat  
<a id="ref-for-background-painting-area"></a>

<a id="ref-for-background-images②"></a>

The [image](#background-images) is repeated in this direction as often as needed to cover the [background painting area](#background-painting-area).

<a id="valdef-background-repeat-space"></a>space  
<a id="ref-for-propdef-background-position"></a>

<a id="ref-for-background-painting-area①"></a>

<a id="ref-for-background-positioning-area"></a>

<a id="ref-for-background-images③"></a>

The [image](#background-images) is repeated as often as will fit within the [background positioning area](#background-positioning-area) without being clipped, and then the images are spaced out to fill the area. The first and last images touch the edges of the area. If the [background painting area](#background-painting-area) is larger than the <a id="ref-for-background-positioning-area①"></a>background positioning area, then the pattern repeats to fill the background painting area. The value of [background-position](#propdef-background-position) for this direction is ignored unless there is not enough space for two copies of the image in this axis, in which case only one image is placed, and <a id="ref-for-propdef-background-position①"></a>background-position determines its position in this axis.

<a id="valdef-background-repeat-round"></a>round  
<a id="ref-for-propdef-background-size"></a>

<a id="ref-for-background-images④"></a>

The [image](#background-images) is repeated as often as will fit within the background positioning area. If it doesn’t fit a whole number of times, it is rescaled so that it does. See the formula under [background-size](#propdef-background-size). If the background painting area is larger than the background positioning area, then the pattern repeats to fill the background painting area.

<a id="valdef-background-repeat-no-repeat"></a>no-repeat  
<a id="ref-for-background-images⑤"></a>

The [image](#background-images) is placed once and not repeated in this direction.

<a id="ref-for-valdef-background-repeat-no-repeat①"></a>

<a id="ref-for-background-painting-area②"></a>

Unless one of the two keywords is [no-repeat](#valdef-background-repeat-no-repeat), the whole [background painting area](#background-painting-area) will be tiled, i.e., not just one vertical strip and one horizontal strip.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-f2ed7ece"></a>
>
> ```text
> body {
>   background: white url("pendant.png");
>   background-repeat: repeat-y;
>   background-position: center;
> }
> ```
>
> ![A centered background image, with copies repeated up and down the border, padding, and content areas.](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/bg-repeat.png)
>
> <a id="ref-for-valdef-background-repeat-repeat-y"></a>
>
> The effect of [repeat-y](#valdef-background-repeat-repeat-y): One copy of the background image is centered, and other copies are put above and below it to make a vertical band behind the element.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-64e67bc4"></a>
>
> ```text
> body {
>   background-image: url(dot.png) white;
>   background-repeat: space
> }
> ```
>
> ![Image of an element with a dotted background](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/bg-space.png)
>
> <a id="ref-for-valdef-background-repeat-space①"></a>
>
> The effect of [space](#valdef-background-repeat-space): the image of a dot is tiled to cover the whole background and the images are equally spaced.

<a id="ref-for-propdef-background-repeat②"></a>

<a id="ref-for-background-image-layer⑦"></a>

See [§ 2.1 Layering Multiple Background Images](#layering) for how [background-repeat](#propdef-background-repeat) interacts with other comma-separated background properties to form each [background image layer](#background-image-layer).

<a id="ref-for-propdef-background-attachment"></a>

### <a id="background-attachment"></a>2.5.  Affixing Images: the [background-attachment](#propdef-background-attachment) property<a id="the-background-attachment"></a>

| Field               | Definition                                                                                                                      |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-background-attachment"></a>background-attachment                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma②"></a><a id="ref-for-typedef-attachment"></a>[\<attachment\>](#typedef-attachment)[\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | scroll                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | list, each item the keyword as specified                                                                                        |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                        |

<a id="ref-for-background-images⑥"></a>

<a id="ref-for-x1"></a>

<a id="ref-for-valdef-background-attachment-fixed"></a>

<a id="ref-for-valdef-background-attachment-scroll"></a>

<a id="ref-for-valdef-background-attachment-local"></a>

<a id="ref-for-typedef-attachment①"></a>

If [background images](#background-images) are specified, this property specifies whether they are fixed with regard to the [viewport](https://www.w3.org/TR/CSS21/visuren.html#x1) ([fixed](#valdef-background-attachment-fixed)) or scroll along with the box ([scroll](#valdef-background-attachment-scroll)) or its contents ([local](#valdef-background-attachment-local)). The property’s value is given as a comma-separated list of [\<attachment\>](#typedef-attachment) keywords where

<a id="typedef-attachment"></a>

<a id="ref-for-typedef-attachment②"></a>

<a id="ref-for-comb-one⑥"></a>

<a id="ref-for-comb-one⑦"></a>

```text
<attachment> = scroll | fixed | local
```
<a id="valdef-background-attachment-fixed"></a>fixed  
<a id="ref-for-valdef-background-attachment-fixed①"></a>

<a id="ref-for-paged-media"></a>

The background is fixed with regard to the viewport. In [paged media](https://www.w3.org/TR/mediaqueries-5/#paged-media) where there is no viewport, a [fixed](#valdef-background-attachment-fixed) background is fixed with respect to the [page box](https://www.w3.org/TR/CSS2/page.html#page-box) and therefore replicated on every page.

<a id="ref-for-scroll-container"></a>

<a id="ref-for-valdef-background-attachment-fixed②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: There is only one viewport per view. Even if an box is a [scroll container](https://www.w3.org/TR/css-overflow-3/#scroll-container), a [fixed](#valdef-background-attachment-fixed) background doesn’t move with the box.

<a id="valdef-background-attachment-local"></a>local  
<a id="ref-for-valdef-background-clip-padding-box"></a>

<a id="ref-for-propdef-background-clip①"></a>

<a id="ref-for-valdef-background-clip-border-box"></a>

<a id="ref-for-scroll-container①"></a>

<a id="ref-for-border-area②"></a>

<a id="ref-for-scrollable-overflow-region"></a>

<a id="ref-for-background-positioning-area②"></a>

<a id="ref-for-background-painting-area③"></a>

The background is fixed with regard to the box’s contents: if the box has a scrolling mechanism, the background scrolls with the box’s contents, and the [background painting area](#background-painting-area) and [background positioning area](#background-positioning-area) are relative to the [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region) of the box rather than to the border framing them. Because the <a id="ref-for-scrollable-overflow-region①"></a>scrollable overflow area does not include the [border area](https://www.w3.org/TR/css-box-4/#border-area), for [scroll containers](https://www.w3.org/TR/css-overflow-3/#scroll-container) the [border-box](#valdef-background-clip-border-box) value of [background-clip](#propdef-background-clip) may be treated the same as [padding-box](#valdef-background-clip-padding-box).

<a id="valdef-background-attachment-scroll"></a>scroll  
The background is fixed with regard to the box itself and does not scroll with its contents. (It is effectively attached to the box’s border.)

<a id="ref-for-background-painting-area④"></a>

Even if the image is fixed, it is still only visible when it is in the [background painting area](#background-painting-area) of the box or otherwise unclipped. (See [§ 2.11 Backgrounds of Special Elements](#special-backgrounds) for the cases when background images are not clipped.) Thus, unless the image is tiled, it may be invisible.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-e97d52e0"></a> This example creates an infinite vertical band that remains “glued” to the viewport when the document is scrolled.
>
> ```text
> body {
>   background: red url("pendant.gif");
>   background-repeat: repeat-y;
>   background-attachment: fixed;
> }
> ```
<a id="ref-for-valdef-background-attachment-fixed③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: User agents that do not support [fixed](#valdef-background-attachment-fixed) backgrounds (for example due to limitations of the hardware platform) [will ignore declarations](https://www.w3.org/TR/CSS/#partial) with the keyword <a id="ref-for-valdef-background-attachment-fixed④"></a>fixed. For example:

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6e8f559e"></a>
>
> ```text
> body {
>   /* For all UAs: */
>   background: white url(paper.png) scroll;
>   /* For UAs that do fixed backgrounds: */
>   background: white url(ledger.png) fixed;
> }
> h1 {
>   /* For all UAs: */
>   background: silver;
>   /* For UAs that do fixed backgrounds: */
>   background: url(stripe.png) fixed, white url(ledger.png) fixed;
> }
> ```
<a id="ref-for-propdef-background-attachment①"></a>

<a id="ref-for-background-image-layer⑧"></a>

See [§ 2.1 Layering Multiple Background Images](#layering) for how [background-attachment](#propdef-background-attachment) interacts with other comma-separated background properties to form each [background image layer](#background-image-layer).

<a id="ref-for-propdef-background-position②"></a>

### <a id="background-position"></a>2.6. Positioning Images: the [background-position](#propdef-background-position) property<a id="the-background-position"></a>

| Field               | Definition                                                                                                                                                                                                                    |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-background-position"></a>background-position                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma③"></a><a id="ref-for-typedef-bg-position"></a>[\<bg-position\>](#typedef-bg-position)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0% 0%                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | refer to size of background positioning area <em>minus</em> size of background image; see text                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage"></a>list, each item a pair of offsets (horizontal and vertical) from the top left origin each given as a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | repeatable list                                                                                                                                                                                                               |

<a id="ref-for-background-images⑦"></a>

<a id="ref-for-background-positioning-area③"></a>

If [background images](#background-images) have been specified, this property specifies their initial position (after any [resizing](#background-size)) within their corresponding [background positioning area](#background-positioning-area).

<a id="ref-for-typedef-bg-position①"></a>

The property’s value is given as a comma-separated list of [\<bg-position\>](#typedef-bg-position) values where

<a id="typedef-bg-position"></a>

<a id="ref-for-typedef-bg-position②"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-comb-one①①"></a>

<a id="ref-for-comb-one①②"></a>

<a id="ref-for-typedef-length-percentage①"></a>

<a id="ref-for-comb-one①③"></a>

<a id="ref-for-comb-one①④"></a>

<a id="ref-for-comb-one①⑤"></a>

<a id="ref-for-comb-one①⑥"></a>

<a id="ref-for-typedef-length-percentage②"></a>

<a id="ref-for-comb-one①⑦"></a>

<a id="ref-for-comb-one①⑧"></a>

<a id="ref-for-comb-one①⑨"></a>

<a id="ref-for-typedef-length-percentage③"></a>

<a id="ref-for-comb-one②⓪"></a>

<a id="ref-for-comb-one②①"></a>

<a id="ref-for-comb-one②②"></a>

<a id="ref-for-typedef-length-percentage④"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-comb-all"></a>

<a id="ref-for-comb-one②③"></a>

<a id="ref-for-comb-one②④"></a>

<a id="ref-for-typedef-length-percentage⑤"></a>

<a id="ref-for-mult-opt①"></a>

```text
<bg-position> = [
  [ left | center | right | top | bottom | <length-percentage> ]
|
  [ left | center | right | <length-percentage> ]
  [ top | center | bottom | <length-percentage> ]
|
  [ center | [ left | right ] <length-percentage>? ] &&
  [ center | [ top | bottom ] <length-percentage>? ]
]
```
<a id="ref-for-valdef-background-position-center"></a>

<a id="ref-for-typedef-length-percentage⑥"></a>

<a id="ref-for-background-positioning-area④"></a>

If only one value is specified, the second value is assumed to be [center](#valdef-background-position-center). If two values are given, a [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) as the first value represents the horizontal position (or offset) and a <a id="ref-for-typedef-length-percentage⑦"></a>\<length-percentage\> as the second value represents the vertical position (or offset). The <a id="ref-for-typedef-length-percentage⑧"></a>\<length-percentage\> values here represent an offset of the top left corner of the background image from the top left corner of the [background positioning area](#background-positioning-area).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A pair of keywords can be reordered, while a combination of keyword and length or percentage cannot. So center left is valid while 50% left is not.

<a id="ref-for-typedef-length-percentage⑨"></a>

<a id="ref-for-propdef-background-position③"></a>

If three or four values are given, then each [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) represents an offset and must be preceded by a keyword, which specifies from which edge the offset is given. For example, [background-position: bottom 10px right 20px](#propdef-background-position) represents a 10px vertical offset up from the bottom edge and a 20px horizontal offset leftward from the right edge. If three values are given, the missing offset is assumed to be zero.

<a id="ref-for-background-positioning-area⑤"></a>

Positive values represent an offset <em>inward</em> from the edge of the [background positioning area](#background-positioning-area). Negative values represent an offset <em>outward</em> from the edge of the <a id="ref-for-background-positioning-area⑥"></a>background positioning area.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b28c564d"></a> The following declarations give the stated (horizontal, vertical) offsets from the top left corner:
>
> ```text
> background-position: left 10px top 15px;   /* 10px, 15px */
> background-position: left      top     ;   /*  0px,  0px */
> background-position:      10px     15px;   /* 10px, 15px */
> background-position: left          15px;   /*  0px, 15px */
> background-position:      10px top     ;   /* 10px,  0px */
> background-position: left      top 15px;   /*  0px, 15px */
> background-position: left 10px top     ;   /* 10px,  0px */
> ```
<a id="ref-for-percentage-value"></a>

<a id="valdef-background-position-percentage"></a>[\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value)

<a id="ref-for-propdef-background-size①"></a>

<a id="ref-for-background-images⑨"></a>

<a id="ref-for-background-positioning-area⑧"></a>

<a id="ref-for-background-images⑧"></a>

<a id="ref-for-background-positioning-area⑦"></a>

A percentage for the horizontal offset is relative to (<var>width of <a href="#background-positioning-area">background positioning area</a></var> - <var>width of <a href="#background-images">background image</a></var>). A percentage for the vertical offset is relative to (<var>height of <a href="#background-positioning-area">background positioning area</a></var> - <var>height of <a href="#background-images">background image</a></var>), where the size of the image is the size given by [background-size](#propdef-background-size).

<a id="ref-for-padding-edge"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c27a0cea"></a> For example, with a value pair of 0% 0%, the upper left corner of the image is aligned with the upper left corner of, usually, the box’s [padding edge](https://www.w3.org/TR/css-box-4/#padding-edge). A value pair of 100% 100% places the lower right corner of the image in the lower right corner of the area. With a value pair of 75% 50%, the point 75% across and 50% down the image is to be placed at the point 75% across and 50% down the area.
>
> ![Diagram of image position within element](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/bg-pos.png)
>
> <a id="ref-for-propdef-background-position④"></a>
>
> Diagram of the meaning of [background-position: 75% 50%](#propdef-background-position).

<a id="ref-for-length-value"></a>

<a id="valdef-background-position-length"></a>[\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

<a id="ref-for-background-positioning-area⑨"></a>

A length value gives a fixed length as the offset. For example, with a value pair of 2cm 1cm, the upper left corner of the image is placed 2cm to the right and 1cm below the upper left corner of the [background positioning area](#background-positioning-area).

<a id="ref-for-valdef-background-position-top"></a>

<a id="valdef-background-position-top"></a>[top](#valdef-background-position-top)

Computes to 0% for the vertical position if one or two values are given, otherwise specifies the top edge as the origin for the next offset.

<a id="ref-for-valdef-background-position-right"></a>

<a id="valdef-background-position-right"></a>[right](#valdef-background-position-right)

Computes to 100% for the horizontal position if one or two values are given, otherwise specifies the right edge as the origin for the next offset.

<a id="ref-for-valdef-background-position-bottom"></a>

<a id="valdef-background-position-bottom"></a>[bottom](#valdef-background-position-bottom)

Computes to 100% for the vertical position if one or two values are given, otherwise specifies the bottom edge as the origin for the next offset.

<a id="ref-for-valdef-background-position-left"></a>

<a id="valdef-background-position-left"></a>[left](#valdef-background-position-left)

Computes to 0% for the horizontal position if one or two values are given, otherwise specifies the left edge as the origin for the next offset.

<a id="ref-for-valdef-background-position-center①"></a>

<a id="valdef-background-position-center"></a>[center](#valdef-background-position-center)

Computes to 50% (left 50%) for the horizontal position if the horizontal position is not otherwise specified, or 50% (top 50%) for the vertical position if it is.

<a id="ref-for-propdef-background"></a>

<a id="ref-for-propdef-background-position⑤"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4c5187aa"></a> The following [background](#propdef-background) shorthand declarations use keywords to set [background-position](#propdef-background-position) to the stated percentage values.
>
> ```text
> body { background: url("banner.jpeg") right top }    /* 100%   0% */
> body { background: url("banner.jpeg") top center }   /*  50%   0% */
> body { background: url("banner.jpeg") center }       /*  50%  50% */
> body { background: url("banner.jpeg") bottom }       /*  50% 100% */
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-6c448699"></a> In the example below, the (single) image is placed in the lower-right corner of the viewport.
>
> ```text
> body {
>   background-image: url("logo.png");
>   background-attachment: fixed;
>   background-position: 100% 100%;
>   background-repeat: no-repeat;
> }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1d05ddd8"></a> Background positions can also be relative to other corners than the top left. For example, the following puts the background image 10px from the bottom and 3em from the right:
>
> ```text
> background-position: right 3em bottom 10px
> ```
<a id="ref-for-propdef-background-position⑥"></a>

<a id="ref-for-background-image-layer⑨"></a>

See [§ 2.1 Layering Multiple Background Images](#layering) for how [background-position](#propdef-background-position) interacts with other comma-separated background properties to form each [background image layer](#background-image-layer).

<a id="ref-for-propdef-background-position⑦"></a>

#### <a id="bg-position-serialization"></a>2.6.1.  Serialization of [background-position](#propdef-background-position) values

<a id="ref-for-specified-value"></a>

<a id="ref-for-computed-value"></a>

<a id="ref-for-typedef-bg-position③"></a>

<a id="ref-for-typedef-position"></a>

The [specified value](https://www.w3.org/TR/css-cascade-5/#specified-value) and [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of the [\<bg-position\>](#typedef-bg-position) type serialize exactly as defined in [\[CSS-VALUES-4\]](#biblio-css-values-4) for [\<position\>](https://www.w3.org/TR/css-values-3/#typedef-position). For 3-value productions (which are not valid in <a id="ref-for-typedef-position①"></a>\<position\>), the <a id="ref-for-specified-value①"></a>specified value serialization is identical to the equivalent 4-value syntax except that the omitted offset remains omitted.

<a id="ref-for-propdef-background-clip②"></a>

### <a id="background-clip"></a>2.7.  Painting Area: the [background-clip](#propdef-background-clip) property<a id="the-background-clip"></a>

| Field               | Definition                                                                                                                                                      |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-background-clip"></a>background-clip                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma④"></a><a id="ref-for-typedef-visual-box"></a>[\<visual-box\>](https://www.w3.org/TR/css-box-4/#typedef-visual-box)[\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | border-box                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | list, each item a keyword as specified                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | repeatable list                                                                                                                                                 |

Determines the <a id="background-painting-area"></a>background painting area, which determines the area within which the background is painted. Values have the following meanings:

<a id="valdef-background-clip-border-box"></a>border-box  
<a id="ref-for-border-box"></a>

The background is painted within (clipped to) the [border box](https://www.w3.org/TR/css-box-4/#border-box).

<a id="valdef-background-clip-padding-box"></a>padding-box  
<a id="ref-for-padding-box"></a>

The background is painted within (clipped to) the [padding box](https://www.w3.org/TR/css-box-4/#padding-box).

<a id="valdef-background-clip-content-box"></a>content-box  
<a id="ref-for-content-box"></a>

The background is painted within (clipped to) the [content box](https://www.w3.org/TR/css-box-4/#content-box).

<a id="ref-for-background-painting-area⑤"></a>

<a id="ref-for-propdef-background-clip③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The root element has a different [background painting area](#background-painting-area) and thus the [background-clip](#propdef-background-clip) property has no effect when specified on it. See [§ 2.11 Backgrounds of Special Elements](#special-backgrounds).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The background is always drawn <em>behind</em> the border, if any. See “Elaborate description of Stacking Contexts” in [\[CSS2\]](#biblio-css2) Appendix E.

<a id="ref-for-propdef-border-radius"></a>

<a id="ref-for-background-painting-area⑥"></a>

See [§ 4.2 Corner Shaping](#corner-shaping) for how [border-radius](#propdef-border-radius) affects the shape of the [background painting area](#background-painting-area).

<a id="ref-for-propdef-background-clip④"></a>

<a id="ref-for-background-image-layer①⓪"></a>

See [§ 2.1 Layering Multiple Background Images](#layering) for how [background-clip](#propdef-background-clip) interacts with other comma-separated background properties to form each [background image layer](#background-image-layer).

<a id="ref-for-propdef-background-origin①"></a>

### <a id="background-origin"></a>2.8.  Positioning Area: the [background-origin](#propdef-background-origin) property<a id="the-background-origin"></a>

| Field               | Definition                                                                                                                                                      |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-background-origin"></a>background-origin                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma⑤"></a><a id="ref-for-typedef-visual-box①"></a>[\<visual-box\>](https://www.w3.org/TR/css-box-4/#typedef-visual-box)[\#](https://www.w3.org/TR/css-values-4/#mult-comma) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | padding-box                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | list, each item a keyword as specified                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | repeatable list                                                                                                                                                 |

<a id="ref-for-box-fragment①"></a>

<a id="ref-for-propdef-box-decoration-break①"></a>

This property determines the <a id="background-positioning-area"></a>background positioning area: the area within which any background images are positioned. For elements rendered as multiple [box fragments](https://www.w3.org/TR/css-break-4/#box-fragment) (e.g., inline boxes on several lines, boxes on several pages), specifies which boxes [box-decoration-break](https://www.w3.org/TR/css-break-4/#propdef-box-decoration-break) [\[CSS-BREAK-3\]](#biblio-css-break-3) operates on to determine the background positioning area(s).

<a id="valdef-background-origin-padding-box"></a>padding-box  
<a id="ref-for-padding-box①"></a>

The position is relative to the [padding box](https://www.w3.org/TR/css-box-4/#padding-box). (For single boxes 0 0 is the upper left corner of the padding edge, 100% 100% is the lower right corner.)

<a id="valdef-background-origin-border-box"></a>border-box  
<a id="ref-for-border-box①"></a>

The position is relative to the [border box](https://www.w3.org/TR/css-box-4/#border-box).

<a id="valdef-background-origin-content-box"></a>content-box  
<a id="ref-for-content-box①"></a>

The position is relative to the [content box](https://www.w3.org/TR/css-box-4/#content-box).

<a id="ref-for-propdef-background-attachment②"></a>

<a id="ref-for-background-image-layer①①"></a>

<a id="ref-for-valdef-background-attachment-fixed⑤"></a>

<a id="ref-for-background-positioning-area①⓪"></a>

<a id="ref-for-initial-containing-block"></a>

If the [background-attachment](#propdef-background-attachment) value for this [layer](#background-image-layer) is [fixed](#valdef-background-attachment-fixed), then this property has no effect: in this case the [background positioning area](#background-positioning-area) is the [initial containing block](https://www.w3.org/TR/css-display-3/#initial-containing-block).

<a id="ref-for-propdef-background-clip⑤"></a>

<a id="ref-for-valdef-background-clip-padding-box①"></a>

<a id="ref-for-propdef-background-origin②"></a>

<a id="ref-for-valdef-background-origin-border-box"></a>

<a id="ref-for-propdef-background-position⑧"></a>

<a id="ref-for-background-images①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If [background-clip](#propdef-background-clip) is [padding-box](#valdef-background-clip-padding-box), [background-origin](#propdef-background-origin) is [border-box](#valdef-background-origin-border-box), [background-position](#propdef-background-position) is top left (the initial value), and the element has a non-zero border, then the top and left edges of the [background image](#background-images) will be clipped.

<a id="ref-for-propdef-background-origin③"></a>

<a id="ref-for-background-image-layer①②"></a>

See [§ 2.1 Layering Multiple Background Images](#layering) for how [background-origin](#propdef-background-origin) interacts with other comma-separated background properties to form each [background image layer](#background-image-layer).

<a id="ref-for-propdef-background-size②"></a>

### <a id="background-size"></a>2.9. Sizing Images: the [background-size](#propdef-background-size) property<a id="the-background-size"></a>

| Field               | Definition                                                                                                                                                                                                        |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-background-size"></a>background-size                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma⑥"></a><a id="ref-for-typedef-bg-size"></a>[\<bg-size\>](#typedef-bg-size)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see text                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①⓪"></a>list, each item a pair of sizes (one per axis) each represented as either a keyword or a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | repeatable list                                                                                                                                                                                                   |

<a id="ref-for-background-images①①"></a>

<a id="ref-for-typedef-bg-size①"></a>

This property specifies the size of each [background image](#background-images). The property’s value is given as a comma-separated list of [\<bg-size\>](#typedef-bg-size) values where

<a id="typedef-bg-size"></a>

<a id="ref-for-typedef-bg-size②"></a>

<a id="ref-for-typedef-length-percentage①①"></a>

<a id="ref-for-comb-one②⑤"></a>

<a id="ref-for-mult-num-range①"></a>

<a id="ref-for-comb-one②⑥"></a>

<a id="ref-for-comb-one②⑦"></a>

```text
<bg-size> = [ <length-percentage [0,∞]> | auto ]{1,2} | cover | contain
```
Values have the following meanings:

<a id="valdef-background-size-contain"></a>contain

<a id="ref-for-background-positioning-area①①"></a>

<a id="ref-for-natural-aspect-ratio"></a>

Scale the image, while preserving its [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio) (if any), to the largest size such that both its width and its height can fit inside the [background positioning area](#background-positioning-area).

<a id="valdef-background-size-cover"></a>cover

<a id="ref-for-background-positioning-area①②"></a>

<a id="ref-for-natural-aspect-ratio①"></a>

Scale the image, while preserving its [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio) (if any), to the smallest size such that both its width and its height can completely cover the [background positioning area](#background-positioning-area).

<a id="ref-for-typedef-length-percentage①②"></a>

\[ <a id="valdef-background-size-length-percentage-0"></a>[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) \| <a id="valdef-background-size-auto"></a>auto \]{1,2}

<a id="ref-for-valdef-background-size-auto"></a>

The first value gives the width of the corresponding image, the second value its height. If only one value is given the second is assumed to be [auto](#valdef-background-size-auto).

<a id="ref-for-percentage-value①"></a>

<a id="ref-for-background-positioning-area①③"></a>

A [\<percentage\>](https://www.w3.org/TR/css-values-4/#percentage-value) is relative to the [background positioning area](#background-positioning-area).

<a id="ref-for-valdef-background-size-auto①"></a>

<a id="ref-for-natural-aspect-ratio②"></a>

<a id="ref-for-natural-size"></a>

An [auto](#valdef-background-size-auto) value for one dimension is resolved by using the image’s [natural aspect ratio](https://www.w3.org/TR/css-images-3/#natural-aspect-ratio) and the size of the other dimension, or failing that, using the image’s [natural size](https://www.w3.org/TR/css-images-3/#natural-size), or failing that, treating it as 100%.

<a id="ref-for-valdef-background-size-auto②"></a>

<a id="ref-for-natural-width"></a>

<a id="ref-for-natural-height"></a>

<a id="ref-for-natural-size①"></a>

<a id="ref-for-valdef-background-size-contain"></a>

If both values are [auto](#valdef-background-size-auto) then the [natural width](https://www.w3.org/TR/css-images-3/#natural-width) and/or [height](https://www.w3.org/TR/css-images-3/#natural-height) of the image should be used, if any, the missing dimension (if any) behaving as <a id="ref-for-valdef-background-size-auto③"></a>auto as described above. If the image has neither [natural size](https://www.w3.org/TR/css-images-3/#natural-size), its size is determined as for [contain](#valdef-background-size-contain).

Negative values are invalid.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-bb1c7b0c"></a> Here are some examples. The first example stretches the background image independently in both dimensions to completely cover the content area:
>
> ```text
> div {
>   background-image: url(plasma.png);
>   background-repeat: no-repeat;
>   background-size: 100% 100%;
>   background-origin: content-box }
> ```
>
> The second example stretches the image so that exactly two copies fit horizontally. The aspect ratio is preserved:
>
> ```text
> p {
>   background-image: url(tubes.png);
>   background-size: 50% auto;
>   background-origin: border-box }
> ```
>
> This example forces the background image to be 15 by 15 pixels:
>
> ```text
> p {
>   background-size: 15px 15px;
>   background-image: url(tile.png)}
> ```
>
> This example uses the image’s natural size. Note that this is the only possible behavior in CSS level 1 and 2.
>
> ```text
> body {
> background-size: auto;            /* default */
> background-image: url(flower.png) }
> ```
>
> The following example rounds the height of the image to 33.3%, up from the specified value of 30%. At 30%, three images would fit entirely and a fourth only partially. After rounding, three images fit exactly. The width of the image is 20% of the background positioning area width and is not rounded.
>
> ```text
> p {
>   background-image: url(chain.png);
>   background-repeat: no-repeat round;
>   background-size: 20% 30% }
> ```
<a id="ref-for-propdef-background-repeat③"></a>

<a id="ref-for-valdef-background-repeat-round①"></a>

<a id="ref-for-background-positioning-area①④"></a>

If [background-repeat](#propdef-background-repeat) is [round](#valdef-background-repeat-round) for one (or both) dimensions, there is a second step. The UA must scale the image in that dimension (or both dimensions) so that it fits a whole number of times in the [background positioning area](#background-positioning-area). In the case of the width (height is analogous):

> If <var>X</var> ≠ 0 is the width of the image after step one and <var>W</var> is the width of the background positioning area, then the rounded width <var>X'</var> = <var>W</var> / round(<var>W</var> / <var>X</var>) where round() is a function that returns the nearest natural number (integer greater than zero).

<a id="ref-for-propdef-background-repeat④"></a>

<a id="ref-for-valdef-background-repeat-round②"></a>

<a id="ref-for-propdef-background-size③"></a>

<a id="ref-for-valdef-background-size-auto④"></a>

If [background-repeat](#propdef-background-repeat) is [round](#valdef-background-repeat-round) for one dimension only and if [background-size](#propdef-background-size) is [auto](#valdef-background-size-auto) for the other dimension, then there is a third step: that other dimension is scaled so that the original aspect ratio is restored.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-a61fb097"></a> In this example the background image is shown at its natural size:
>
> ```text
> div {
>   background-image: url(image1.png);
>   background-repeat: repeat;         /* default */
>   background-size: auto }            /* default */
> ```
>
> In the following example, the background is shown with a width of 3em and its height is scaled proportionally to keep the original aspect ratio:
>
> ```text
> div {
>   background-image: url(image2.png);
>   background-repeat: repeat;         /* default */
>   background-size: 3em }             /* = '3em auto' */
> ```
>
> In the following example, the background is shown with a width of approximately 3em: scaled so that it fits a whole number of times in the width of the background. The height is scaled proportionally to keep the original aspect ratio:
>
> ```text
> div {
>   background-image: url(image3.png);
>   background-repeat: round repeat;
>   background-size: 3em auto }
> ```
>
> In the following example, the background image is shown with a width of 3em and a height that is either the height corresponding to that width at the original aspect ratio or slightly less:
>
> ```text
> div {
>   background-image: url(image4.png);
>   background-repeat: repeat round;
>   background-size: 3em auto }
> ```
>
> In the following example, the background image is shown with a height of approximately 4em: scaled slightly so that it fits a whole number of times in the background height. The width is the approximately the width that corresponds to a 4em height at the original aspect ratio: scaled slightly so that it fits a whole number of times in the background width.
>
> ```text
> div {
>   background-image: url(image5.png);
>   background-repeat: round;
>   background-size: auto 4em }
> ```
If the background image’s width or height resolves to zero, this causes the image not to be displayed. (The effect is the same as if it had been a transparent image.)

<a id="ref-for-propdef-background-size④"></a>

<a id="ref-for-background-image-layer①③"></a>

See [§ 2.1 Layering Multiple Background Images](#layering) for how [background-size](#propdef-background-size) interacts with other comma-separated background properties to form each [background image layer](#background-image-layer).

<a id="ref-for-propdef-background①"></a>

### <a id="background"></a>2.10.  Backgrounds Shorthand: the [background](#propdef-background) property<a id="the-background"></a>

| Field               | Definition                                                                                                                                                                                                                                                                                                                              |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-background"></a>background                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-final-bg-layer"></a><a id="ref-for-comb-comma"></a><a id="ref-for-mult-opt②"></a><a id="ref-for-mult-comma⑦"></a><a id="ref-for-typedef-bg-layer"></a>[\<bg-layer\>](#typedef-bg-layer)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)[?](https://www.w3.org/TR/css-values-4/#mult-opt) [,](https://www.w3.org/TR/css-values-4/#comb-comma) [\<final-bg-layer\>](#typedef-final-bg-layer) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                               |

<a id="ref-for-propdef-background②"></a>

<a id="ref-for-shorthand-property"></a>

<a id="ref-for-background-image-layer①④"></a>

<a id="ref-for-propdef-background-image⑤"></a>

<a id="ref-for-propdef-background-position⑨"></a>

<a id="ref-for-propdef-background-size⑤"></a>

<a id="ref-for-propdef-background-repeat⑤"></a>

<a id="ref-for-propdef-background-origin④"></a>

<a id="ref-for-propdef-background-clip⑥"></a>

<a id="ref-for-propdef-background-attachment③"></a>

<a id="ref-for-initial-value"></a>

<a id="ref-for-propdef-background-color④"></a>

The [background](#propdef-background) property is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for setting most background properties at the same place in the style sheet. The number of comma-separated items defines the number of [background image layers](#background-image-layer). Given a valid declaration, for each layer the shorthand first sets the corresponding value of each of [background-image](#propdef-background-image), [background-position](#propdef-background-position), [background-size](#propdef-background-size), [background-repeat](#propdef-background-repeat), [background-origin](#propdef-background-origin), [background-clip](#propdef-background-clip) and [background-attachment](#propdef-background-attachment) to that property’s [initial value](https://www.w3.org/TR/css-cascade-5/#initial-value), then assigns any explicit values specified for this layer in the declaration. Finally [background-color](#propdef-background-color) is set to the specified color, if any, else set to its initial value.

This property’s value is given as a comma-separated list of values where

<a id="typedef-bg-layer"></a>

<a id="ref-for-typedef-bg-image③"></a>

<a id="ref-for-comb-any"></a>

<a id="ref-for-typedef-bg-position④"></a>

<a id="ref-for-typedef-bg-size③"></a>

<a id="ref-for-mult-opt③"></a>

<a id="ref-for-comb-any①"></a>

<a id="ref-for-typedef-repeat-style⑤"></a>

<a id="ref-for-comb-any②"></a>

<a id="ref-for-typedef-attachment③"></a>

<a id="ref-for-comb-any③"></a>

<a id="ref-for-typedef-visual-box②"></a>

<a id="ref-for-comb-any④"></a>

<a id="ref-for-typedef-visual-box③"></a>

<a id="typedef-final-bg-layer"></a>

<a id="ref-for-typedef-bg-image④"></a>

<a id="ref-for-comb-any⑤"></a>

<a id="ref-for-typedef-bg-position⑤"></a>

<a id="ref-for-typedef-bg-size④"></a>

<a id="ref-for-mult-opt④"></a>

<a id="ref-for-comb-any⑥"></a>

<a id="ref-for-typedef-repeat-style⑥"></a>

<a id="ref-for-comb-any⑦"></a>

<a id="ref-for-typedef-attachment④"></a>

<a id="ref-for-comb-any⑧"></a>

<a id="ref-for-typedef-visual-box④"></a>

<a id="ref-for-comb-any⑨"></a>

<a id="ref-for-typedef-visual-box⑤"></a>

<a id="ref-for-comb-any①⓪"></a>

<a id="ref-for-propdef-background-color⑤"></a>

```text
<bg-layer> = <bg-image> || <bg-position> [ / <bg-size> ]? || <repeat-style> || <attachment> || <visual-box> || <visual-box>
<final-bg-layer> =  <bg-image> || <bg-position> [ / <bg-size> ]? || <repeat-style> || <attachment> || <visual-box> || <visual-box> || <'background-color'>
```
<a id="ref-for-typedef-final-bg-layer①"></a>

<a id="ref-for-typedef-bg-layer①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A color is permitted in [\<final-bg-layer\>](#typedef-final-bg-layer), but not in [\<bg-layer\>](#typedef-bg-layer).

<a id="ref-for-typedef-visual-box⑥"></a>

<a id="ref-for-propdef-background-origin⑤"></a>

<a id="ref-for-propdef-background-clip⑦"></a>

If one [\<visual-box\>](https://www.w3.org/TR/css-box-4/#typedef-visual-box) value is present then it sets both [background-origin](#propdef-background-origin) and [background-clip](#propdef-background-clip) to that value. If two values are present, then the first sets <a id="ref-for-propdef-background-origin⑥"></a>background-origin and the second <a id="ref-for-propdef-background-clip⑧"></a>background-clip.

<a id="ref-for-propdef-background-color⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-15a7e3d5"></a> In the first rule of the following example, only a value for [background-color](#propdef-background-color) has been given and the other individual properties are set to their initial values. In the second rule, many individual properties have been specified.
>
> ```text
> body { background: red }
> p { background: url("chess.png") 40% / 10em gray
>                 round fixed border-box; }
> ```
>
> The first rule is equivalent to:
>
> ```text
> body {
>     background-color: red;
>     background-position: 0% 0%;
>     background-size: auto;
>     background-repeat: repeat;
>     background-clip: border-box;
>     background-origin: padding-box;
>     background-attachment: scroll;
>     background-image: none }
> ```
>
> The second is equivalent to:
>
> ```text
> p {
>     background-color: gray;
>     background-position: 40% 50%;
>     background-size: 10em auto;
>     background-repeat: round;
>     background-clip: border-box;
>     background-origin: border-box;
>     background-attachment: fixed;
>     background-image: url(chess.png) }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ff3ed279"></a> The following example shows how a both a background color (\#CCC) and a background image (url(metal.jpg)) are set. The image is rescaled to the full width of the element:
>
> ```text
> E { background: #CCC url("metal.jpg") top left / 100% auto no-repeat}
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-1a1113f1"></a> Another example shows equivalence:
>
> ```text
> div { background: padding-box url(paper.jpg) white center }
> div {
>     background-color: white;
>     background-image: url(paper.jpg);
>     background-repeat: repeat;
>     background-attachment: scroll;
>     background-position: center;
>     background-clip: padding-box;
>     background-origin: padding-box;
>     background-size: auto auto }
> ```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3524ad83"></a> The following declaration with multiple, comma-separated values
>
> ```text
> background: url(a.png) top left no-repeat,
>             url(b.png) center / 100% 100% no-repeat,
>             url(c.png) white;
> ```
>
> is equivalent to
>
> ```text
> background-image:      url(a.png),  url(b.png),          url(c.png);
> background-position:   top left,    center,              top left;
> background-repeat:     no-repeat,   no-repeat,           repeat;
> background-clip:       border-box,  border-box,          border-box;
> background-origin:     padding-box, padding-box,         padding-box;
> background-size:       auto auto,   100% 100%,           auto auto;
> background-attachment: scroll,      scroll,              scroll;
> background-color:      white;
> ```
### <a id="special-backgrounds"></a>2.11.  Backgrounds of Special Elements

<a id="ref-for-root-element"></a>

<a id="ref-for-propdef-display"></a>

<a id="ref-for-canvas-background"></a>

The document [canvas](https://www.w3.org/TR/CSS2/intro.html#the-canvas) is the infinite surface over which the document is rendered. [\[CSS2\]](#biblio-css2) Since no element corresponds to the canvas, in order to allow styling of the canvas CSS propagates the background of the [root element](https://www.w3.org/TR/css-display-3/#root-element) (or, in the case of HTML, the \<body\> element) as described below. However, if the element whose background would be used for the canvas is [display: none](https://www.w3.org/TR/CSS21/visuren.html#propdef-display), then the [canvas background](#canvas-background) is transparent.

<a id="ref-for-canvas-background①"></a>

<a id="ref-for-canvas-surface"></a>

If the [canvas background](#canvas-background) is not opaque, the <a id="canvas-surface"></a>canvas surface below it shows through. The texture of the [canvas surface](#canvas-surface) is UA-dependent (but is typically an opaque white).

#### <a id="root-background"></a>2.11.1.  The Canvas Background and the Root Element

<a id="ref-for-root-element①"></a>

<a id="ref-for-background-painting-area⑦"></a>

<a id="ref-for-background-positioning-area①⑤"></a>

<a id="ref-for-used-value①"></a>

<a id="ref-for-valdef-color-transparent①"></a>

The background of the [root element](https://www.w3.org/TR/css-display-3/#root-element) becomes the <a id="canvas-background"></a>canvas background and its [background painting area](#background-painting-area) extends to cover the entire [canvas](https://www.w3.org/TR/CSS2/intro.html#the-canvas). However, any images are sized and positioned relative to the root element’s box as if they were painted for that element alone. (In other words, the [background <em>positioning</em> area](#background-positioning-area) is determined as for the root element.) The root element does not paint this background again, i.e., the [used value](https://www.w3.org/TR/css-cascade-5/#used-value) of its background is [transparent](https://www.w3.org/TR/css-color-4/#valdef-color-transparent).

#### <a id="body-background"></a>2.11.2.  The Canvas Background and the HTML \<body\> Element

<a id="ref-for-computed-value①"></a>

<a id="ref-for-propdef-background-image⑥"></a>

<a id="ref-for-root-element②"></a>

<a id="ref-for-valdef-background-image-none②"></a>

<a id="ref-for-propdef-background-color⑦"></a>

<a id="ref-for-valdef-color-transparent②"></a>

<a id="ref-for-used-value②"></a>

<a id="ref-for-initial-value①"></a>

For documents whose root element is an HTML `HTML` element or an XHTML `html` element [\[HTML\]](#biblio-html): if the [computed value](https://www.w3.org/TR/css-cascade-5/#computed-value) of [background-image](#propdef-background-image) on the [root element](https://www.w3.org/TR/css-display-3/#root-element) is [none](#valdef-background-image-none) and its [background-color](#propdef-background-color) is [transparent](https://www.w3.org/TR/css-color-4/#valdef-color-transparent), user agents must instead propagate the <a id="ref-for-computed-value②"></a>computed values of the background properties from that element’s first HTML `BODY` or XHTML `body` child element. The [used values](https://www.w3.org/TR/css-cascade-5/#used-value) of that `BODY` element’s background properties are their [initial values](https://www.w3.org/TR/css-cascade-5/#initial-value), and the propagated values are treated as if they were specified on the root element. It is recommended that authors of HTML documents specify the canvas background using the `BODY` element rather than the `HTML` element.

<a id="ref-for-containment"></a>

<a id="ref-for-the-body-element"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Using [containment](https://www.w3.org/TR/css-contain-2/#containment) disables this special handling of the HTML <code><a href="https://html.spec.whatwg.org/multipage/sections.html#the-body-element">body</a></code> element. See the [CSS Containment 1 § 2 Strong Containment: the contain property](https://www.w3.org/TR/css-contain-1/#contain-property) for details.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8733c37a"></a> According to these rules, the canvas underlying the following HTML document will have a “marble” background:
>
> ```text
> <!DOCTYPE html PUBLIC '-//W3C//DTD HTML 4.0//EN'
>   >
> <html>
>   <head>
>     <title>Setting the canvas background</title>
>     <style type="text/css">
>        body { background: url("http://example.org/marble.png") }
>     </style>
>   </head>
>   <body>
>     <p>My background is marble.</p>
>   </body>
> </html>
> ```
<a id="ref-for-sel-first-line③"></a>

#### <a id="first-line-background"></a>2.11.3.  The [::first-line](https://www.w3.org/TR/selectors-3/#sel-first-line) Pseudo-element‘s Background

<a id="ref-for-sel-first-line④"></a>

The [::first-line](https://www.w3.org/TR/selectors-3/#sel-first-line) pseudo-element is like an inline-level element for the purposes of the background (see section 5.12.1 of [\[CSS2\]](#biblio-css2)). That means, e.g., that in a left-justified first line, the background does not necessarily extend all the way to the right edge.

## <a id="borders"></a>3.  Borders

<a id="ref-for-border①"></a>

<a id="ref-for-propdef-border-style"></a>

<a id="ref-for-propdef-border-color"></a>

<a id="ref-for-propdef-border-width"></a>

The [border](https://www.w3.org/TR/css-box-4/#border) can either be a predefined style (solid line, double line, dotted line, pseudo-3D border, etc.) or it can be an image. In the former case, various properties define the style ([border-style](#propdef-border-style)), color ([border-color](#propdef-border-color)), and thickness ([border-width](#propdef-border-width)) of the border.

<a id="ref-for-propdef-border-color①"></a>

### <a id="border-color"></a>3.1.  Line Colors: the [border-color](#propdef-border-color) properties<a id="the-border-color"></a>

| Field               | Definition                                                                                                                                                                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-top-color"></a>border-top-color, <a id="propdef-border-right-color"></a>border-right-color, <a id="propdef-border-bottom-color"></a>border-bottom-color, <a id="propdef-border-left-color"></a>border-left-color                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-color①"></a>[\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color)                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | [currentColor](https://www.w3.org/TR/css-color-4/#currentcolor-color)                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-ruby-annotation-container-box①"></a><a id="ref-for-ruby-base-container-box①"></a>all elements except [ruby base containers](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and [ruby annotation containers](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-container-box) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | computed color                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                             |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-color②"></a>[border-color](#propdef-border-color)                                                                                                                                                                                      |

| Field               | Definition                                                                                                                                                                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-color"></a>border-color                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range②"></a><a id="ref-for-typedef-color②"></a>[\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range)                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | (see individual properties)                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-ruby-annotation-container-box②"></a><a id="ref-for-ruby-base-container-box②"></a>all elements except [ruby base containers](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and [ruby annotation containers](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-container-box) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                     |

<a id="ref-for-border②"></a>

<a id="ref-for-propdef-border-style①"></a>

These properties set the foreground <a id="border-color-dfn"></a>color of the [border](https://www.w3.org/TR/css-box-4/#border) specified by the [border-style](#propdef-border-style) properties.

<a id="ref-for-propdef-border-color③"></a>

<a id="ref-for-shorthand-property①"></a>

<a id="ref-for-propdef-border-top-color"></a>

<a id="ref-for-propdef-border-right-color"></a>

<a id="ref-for-propdef-border-bottom-color"></a>

<a id="ref-for-propdef-border-left-color"></a>

The [border-color](#propdef-border-color) property is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for setting [border-top-color](#propdef-border-top-color), [border-right-color](#propdef-border-right-color), [border-bottom-color](#propdef-border-bottom-color), and [border-left-color](#propdef-border-left-color) in a single declaration.

If there is only one component value, it applies to all sides. If there are two values, the top and bottom are set to the first value and the right and left are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values they apply to the top, right, bottom, and left, respectively.

<a id="ref-for-propdef-border-style②"></a>

### <a id="border-style"></a>3.2. Line Patterns: the [border-style](#propdef-border-style) properties<a id="the-border-style"></a>

| Field               | Definition                                                                                                                                                                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-top-style"></a>border-top-style, <a id="propdef-border-right-style"></a>border-right-style, <a id="propdef-border-bottom-style"></a>border-bottom-style, <a id="propdef-border-left-style"></a>border-left-style                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-line-style"></a>[\<line-style\>](#typedef-line-style)                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-ruby-annotation-container-box③"></a><a id="ref-for-ruby-base-container-box③"></a>all elements except [ruby base containers](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and [ruby annotation containers](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-container-box) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                      |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-style③"></a>[border-style](#propdef-border-style)                                                                                                                                                                                      |

| Field               | Definition                                                                                                                                                                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-style"></a>border-style                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range③"></a><a id="ref-for-typedef-line-style①"></a>[\<line-style\>](#typedef-line-style)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range)                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | (see individual properties)                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-ruby-annotation-container-box④"></a><a id="ref-for-ruby-base-container-box④"></a>all elements except [ruby base containers](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and [ruby annotation containers](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-container-box) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                     |

<a id="ref-for-border③"></a>

These properties control whether a [border](https://www.w3.org/TR/css-box-4/#border) appears, and if it does what <a id="border-style-dfn"></a>style it’s drawn in (if it is not overridden by a [border image](#border-images)).

<a id="ref-for-propdef-border-style④"></a>

<a id="ref-for-shorthand-property②"></a>

<a id="ref-for-propdef-border-top-style"></a>

<a id="ref-for-propdef-border-right-style"></a>

<a id="ref-for-propdef-border-bottom-style"></a>

<a id="ref-for-propdef-border-left-style"></a>

The [border-style](#propdef-border-style) property is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for setting [border-top-style](#propdef-border-top-style), [border-right-style](#propdef-border-right-style), [border-bottom-style](#propdef-border-bottom-style), and [border-left-style](#propdef-border-left-style) in a single declaration.

If there is only one component value, it applies to all sides. If there are two values, the top and bottom are set to the first value and the right and left are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values they apply to the top, right, bottom, and left, respectively.

<a id="ref-for-typedef-line-style②"></a>

The style is specified as a [\<line-style\>](#typedef-line-style) keyword, where

<a id="typedef-line-style"></a>

<a id="ref-for-typedef-line-style③"></a>

<a id="ref-for-comb-one②⑧"></a>

<a id="ref-for-comb-one②⑨"></a>

<a id="ref-for-comb-one③⓪"></a>

<a id="ref-for-comb-one③①"></a>

<a id="ref-for-comb-one③②"></a>

<a id="ref-for-comb-one③③"></a>

<a id="ref-for-comb-one③④"></a>

<a id="ref-for-comb-one③⑤"></a>

<a id="ref-for-comb-one③⑥"></a>

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

![Examples of border styles](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/borderstyles.png)

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

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This specification does not define how borders of different styles should be joined in the corner. Also note that rounded corners may cause the corners and the contents to overlap, if the padding is less than the radius of the corner.

<a id="ref-for-propdef-border-width②"></a>

### <a id="border-width"></a>3.3.  Line Thickness: the [border-width](#propdef-border-width) properties<a id="the-border-width"></a>

| Field               | Definition                                                                                                                                                                                                                                                                     |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-top-width"></a>border-top-width, <a id="propdef-border-right-width"></a>border-right-width, <a id="propdef-border-bottom-width"></a>border-bottom-width, <a id="propdef-border-left-width"></a>border-left-width                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-line-width"></a>[\<line-width\>](#typedef-line-width)                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | medium                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-ruby-annotation-container-box⑤"></a><a id="ref-for-ruby-base-container-box⑤"></a>all elements except [ruby base containers](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and [ruby annotation containers](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-container-box)                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-line-style-hidden"></a><a id="ref-for-valdef-line-style-none①"></a><a id="ref-for-snap-a-length-as-a-border-width"></a>absolute length, [snapped as a border width](https://www.w3.org/TR/css-values-4/#snap-a-length-as-a-border-width); zero if the border style is [none](#valdef-line-style-none) or [hidden](#valdef-line-style-hidden) |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                              |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-width③"></a>[border-width](#propdef-border-width)                                                                                                                                                                                                                       |

| Field               | Definition                                                                                                                                                                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-width"></a>border-width                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range④"></a><a id="ref-for-typedef-line-width①"></a>[\<line-width\>](#typedef-line-width)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range)                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | (see individual properties)                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-ruby-annotation-container-box⑥"></a><a id="ref-for-ruby-base-container-box⑥"></a>all elements except [ruby base containers](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and [ruby annotation containers](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-container-box) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                     |

<a id="ref-for-border④"></a>

These properties specify the thickness of the [border](https://www.w3.org/TR/css-box-4/#border), i.e. the <a id="border-width-dfn"></a>border width. Where

<a id="typedef-line-width"></a>

<a id="ref-for-typedef-line-width②"></a>

<a id="ref-for-length-value①"></a>

<a id="ref-for-comb-one③⑦"></a>

<a id="ref-for-comb-one③⑧"></a>

<a id="ref-for-comb-one③⑨"></a>

```text
<line-width> = <length [0,∞]> | thin | medium | thick
```
Negative values are invalid. The <a id="valdef-line-width-thin"></a>thin, <a id="valdef-line-width-medium"></a>medium, and <a id="valdef-line-width-thick"></a>thick keywords are equivalent to 1px, 3px, and 5px, respectively.

<a id="ref-for-propdef-border-width④"></a>

<a id="ref-for-shorthand-property③"></a>

<a id="ref-for-propdef-border-top-width"></a>

<a id="ref-for-propdef-border-right-width"></a>

<a id="ref-for-propdef-border-bottom-width"></a>

<a id="ref-for-propdef-border-left-width"></a>

The [border-width](#propdef-border-width) property is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for setting [border-top-width](#propdef-border-top-width), [border-right-width](#propdef-border-right-width), [border-bottom-width](#propdef-border-bottom-width), and [border-left-width](#propdef-border-left-width) in a single declaration.

If there is only one component value, it applies to all sides. If there are two values, the top and bottom are set to the first value and the right and left are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values they apply to the top, right, bottom, and left, respectively.

<a id="ref-for-initial-value②"></a>

<a id="ref-for-valdef-line-width-medium"></a>

<a id="ref-for-valdef-line-style-none②"></a>

<a id="ref-for-used-value③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Although the [initial](https://www.w3.org/TR/css-cascade-5/#initial-value) width is [medium](#valdef-line-width-medium), the <a id="ref-for-initial-value③"></a>initial style is [none](#valdef-line-style-none); therefore the [used](https://www.w3.org/TR/css-cascade-5/#used-value) initial width is 0.

### <a id="border-shorthands"></a>3.4.  Border Shorthand Properties<a id="the-border-shorthands"></a>

| Field               | Definition                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-top"></a>border-top, <a id="propdef-border-right"></a>border-right, <a id="propdef-border-bottom"></a>border-bottom, <a id="propdef-border-left"></a>border-left                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-color③"></a><a id="ref-for-typedef-line-style④"></a><a id="ref-for-comb-any①①"></a><a id="ref-for-typedef-line-width③"></a>[\<line-width\>](#typedef-line-width) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<line-style\>](#typedef-line-style) <a id="ref-for-comb-any①②"></a>\|\| [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-ruby-annotation-container-box⑦"></a><a id="ref-for-ruby-base-container-box⑦"></a>all elements except [ruby base containers](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and [ruby annotation containers](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-container-box)                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                          |

<a id="ref-for-shorthand-property④"></a>

<a id="ref-for-propdef-border-width⑤"></a>

<a id="ref-for-propdef-border-color⑤"></a>

<a id="ref-for-propdef-border-style⑤"></a>

<a id="ref-for-border⑤"></a>

<a id="ref-for-initial-value④"></a>

These [shorthand properties](https://www.w3.org/TR/css-cascade-5/#shorthand-property) set the [border-width](#propdef-border-width), [border-color](#propdef-border-color), and [border-style](#propdef-border-style) of the top, right, bottom, and left [borders](https://www.w3.org/TR/css-box-4/#border) of a box. Omitted values are set to their [initial values](https://www.w3.org/TR/css-cascade-5/#initial-value).

| Field               | Definition                                                                                                                                                                                                                                                                                         |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border"></a>border                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-color④"></a><a id="ref-for-typedef-line-style⑤"></a><a id="ref-for-comb-any①③"></a><a id="ref-for-typedef-line-width④"></a>[\<line-width\>](#typedef-line-width) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<line-style\>](#typedef-line-style) <a id="ref-for-comb-any①④"></a>\|\| [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-ruby-annotation-container-box⑧"></a><a id="ref-for-ruby-base-container-box⑧"></a>all elements except [ruby base containers](https://www.w3.org/TR/css-ruby-1/#ruby-base-container-box) and [ruby annotation containers](https://www.w3.org/TR/css-ruby-1/#ruby-annotation-container-box)                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                          |

<a id="ref-for-propdef-border①"></a>

<a id="ref-for-shorthand-property⑤"></a>

<a id="ref-for-propdef-border-width⑥"></a>

<a id="ref-for-propdef-border-color⑥"></a>

<a id="ref-for-propdef-border-style⑥"></a>

<a id="ref-for-propdef-margin"></a>

<a id="ref-for-propdef-padding"></a>

The [border](#propdef-border) property is a [shorthand property](https://www.w3.org/TR/css-cascade-5/#shorthand-property) for setting the same [border-width](#propdef-border-width), [border-color](#propdef-border-color), and [border-style](#propdef-border-style) for all four borders of a box. Unlike the shorthand [margin](https://www.w3.org/TR/css-box-4/#propdef-margin) and [padding](https://www.w3.org/TR/css-box-4/#propdef-padding) properties, the <a id="ref-for-propdef-border②"></a>border property cannot set different values on the four borders. To do so, one or more of the other border properties must be used.

<a id="ref-for-propdef-border③"></a>

<a id="ref-for-propdef-border-image②"></a>

The [border](#propdef-border) shorthand also resets [border-image](#propdef-border-image) to its initial value. It is therefore recommended that authors use the <a id="ref-for-propdef-border④"></a>border shorthand, rather than other shorthands or the individual properties, to override any border settings earlier in the cascade. This will ensure that <a id="ref-for-propdef-border-image③"></a>border-image has also been reset to allow the new styles to take effect.

<a id="ref-for-propdef-border⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The CSS Working Group intends for the [border](#propdef-border) shorthand to reset all border properties in future levels of CSS as well. For example, if a border-characters property is introduced in the future to allow glyphs as borders, it will also be reset by the <a id="ref-for-propdef-border⑥"></a>border shorthand. By using the <a id="ref-for-propdef-border⑦"></a>border shorthand to reset borders, authors can be guaranteed a “blank canvas” no matter what properties are introduced in the future.

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
> <a id="ref-for-propdef-border-left"></a>
>
> <a id="ref-for-propdef-color"></a>
>
> In the above example, the color of the left border is black, while the other borders are red. This is due to [border-left](#propdef-border-left) setting the width, style, and color. Since the color value is not given by the <a id="ref-for-propdef-border-left①"></a>border-left property, it will be taken from the [color](https://www.w3.org/TR/css-color-4/#propdef-color) property. The fact that the <a id="ref-for-propdef-color①"></a>color property is set after the <a id="ref-for-propdef-border-left②"></a>border-left property is not relevant.

## <a id="corners"></a>4.  Rounded Corners

<a id="ref-for-propdef-border-radius①"></a>

### <a id="border-radius"></a>4.1.  Curve Radii: the [border-radius](#propdef-border-radius) properties<a id="the-border-radius"></a>

| Field               | Definition                                                                                                                                                                                      |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-top-left-radius"></a>border-top-left-radius, <a id="propdef-border-top-right-radius"></a>border-top-right-radius, <a id="propdef-border-bottom-right-radius"></a>border-bottom-right-radius, <a id="propdef-border-bottom-left-radius"></a>border-bottom-left-radius              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range⑤"></a><a id="ref-for-typedef-length-percentage①③"></a>[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)[{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | all elements (but see prose)                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-border-box②"></a>Refer to corresponding dimension of the [border box](https://www.w3.org/TR/css-box-4/#border-box).                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①④"></a>pair of computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                               |
| <strong><a href="https://drafts.csswg.org/css-logical-1/#logical-property-group">Logical property group:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-radius②"></a>[border-radius](#propdef-border-radius)                                                                                                                                      |

| Field               | Definition                                                                                                                                                                                                                                                                                                                                           |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-radius"></a>border-radius                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt⑤"></a><a id="ref-for-mult-num-range⑥"></a><a id="ref-for-typedef-length-percentage①⑤"></a>[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)[{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) \[ / <a id="ref-for-typedef-length-percentage①⑥"></a>\<length-percentage \[0,∞\]\><a id="ref-for-mult-num-range⑦"></a>{1,4} \][?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | all elements (but see prose)                                                                                                                                                                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-border-box③"></a>Refer to corresponding dimension of the [border box](https://www.w3.org/TR/css-box-4/#border-box).                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | see individual properties                                                                                                                                                                                                                                                                                                                            |

<a id="ref-for-typedef-length-percentage①⑦"></a>

<a id="ref-for-border-edge"></a>

<a id="ref-for-border-box④"></a>

The two [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) values of the border-\*-radius properties define the <a id="border-radii"></a>radii of a quarter ellipse that defines the shape of the corner of the outer [border edge](https://www.w3.org/TR/css-box-4/#border-edge) (see the diagram below). The first value is the horizontal radius, the second the vertical radius. If the second value is omitted it is copied from the first. If either length is zero, the corner is square, not rounded. Percentages for the horizontal radius refer to the width of the [border box](https://www.w3.org/TR/css-box-4/#border-box), whereas percentages for the vertical radius refer to the height of the <a id="ref-for-border-box⑤"></a>border box. Negative values are invalid.

![Diagram of the inscribed ellipse](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/corner.png)

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
<a id="ref-for-propdef-border-radius③"></a>

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
### <a id="corner-shaping"></a>4.2.  Corner Shaping

<a id="ref-for-padding-edge①"></a>

<a id="ref-for-content-edge"></a>

<a id="ref-for-padding①"></a>

The [padding edge](https://www.w3.org/TR/css-box-4/#padding-edge) (inner border) radius is the outer border radius minus the corresponding border thickness. In the case where this results in a negative value, the inner radius is zero. (In such cases the center of the border’s inner curve might not coincide with that of its outer curve.) Likewise the [content edge](https://www.w3.org/TR/css-box-4/#content-edge) radius is the <a id="ref-for-padding-edge②"></a>padding edge radius minus the corresponding [padding](https://www.w3.org/TR/css-box-4/#padding), or if that is negative, zero. The border and padding thicknesses in the curved region are thus interpolated from the adjoining sides, and when two adjoining borders are of different thicknesses the corner will show a smooth transition between the thicker and thinner borders.

<a id="ref-for-valdef-line-style-solid"></a>

<a id="ref-for-valdef-line-style-dotted"></a>

<a id="ref-for-valdef-line-style-inset①"></a>

All border styles ([solid](#valdef-line-style-solid), [dotted](#valdef-line-style-dotted), [inset](#valdef-line-style-inset), etc.) follow the curve of the border.

![The effect of rounded corners on unequal borders](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/smooth-radius.png)

The effect of a rounded corner when the two borders it connects are of unequal thickness (left) and the effect of a rounded corner on borders that are thicker than the radius of the corner (right).

<a id="ref-for-padding-edge③"></a>

<a id="ref-for-border-area③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If the center of a corner’s outer curve is past an opposite [padding edge](https://www.w3.org/TR/css-box-4/#padding-edge) (in the [border area](https://www.w3.org/TR/css-box-4/#border-area) of a side opposite the corner), the inner curve will not be a full quarter ellipse.

**CSS**

```text
p { width: 70px; height: 70px; border: solid 30px;
border-color: orange orange silver silver;
border-top-right-radius: 100%; }
```

**Reference image**

![The curved corner is an arc from the top left corner sweeping across the top right corner to the bottom right corner, describing a quarter-ellipse; but since the opposite sides have a border thickness the padding edge curve starts inward from the outer arc's endpoints.](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/partial-curve.png)

Where the border-radius curve extends into the opposite sides' borders, the arc of the padding edge is less than 90°.

<a id="ref-for-margin-edge"></a>

<a id="ref-for-border-edge①"></a>

<a id="ref-for-border-radii"></a>

The [margin edge](https://www.w3.org/TR/css-box-4/#margin-edge), being outside the [border edge](https://www.w3.org/TR/css-box-4/#border-edge), calculates its radius by <em>adding</em> the corresponding margin thickness to each border radius. However, in order to create a sharper corner when the border radius is small (and thus ensure continuity between round and sharp corners), when the [border radius](#border-radii) is less than the margin, the margin is multiplied by the proportion 1 + (<var>r</var>-1)<sup>3</sup>, where <var>r</var> is the ratio of the border radius to the margin, in calculating the corner radii of the margin box shape.

### <a id="corner-clipping"></a>4.3.  Corner Clipping

<a id="ref-for-propdef-border-radius④"></a>

<a id="ref-for-border-edge②"></a>

<a id="ref-for-padding-edge④"></a>

<a id="ref-for-content-edge①"></a>

<a id="ref-for-propdef-background-clip⑨"></a>

<a id="ref-for-propdef-overflow"></a>

<a id="ref-for-replaced-element"></a>

Although [border images](#border-image) are not affected by [border-radius](#propdef-border-radius), other effects that clip painting or event handling to the [border](https://www.w3.org/TR/css-box-4/#border-edge), [padding](https://www.w3.org/TR/css-box-4/#padding-edge), or [content](https://www.w3.org/TR/css-box-4/#content-edge) edge must clip to their respective curves. For example, backgrounds clip to the curve specified by [background-clip](#propdef-background-clip), [overflow](https://www.w3.org/TR/CSS21/visufx.html#propdef-overflow) values other than visible to the curved <a id="ref-for-padding-edge⑤"></a>padding edge (when <a id="ref-for-propdef-overflow①"></a>overflow on both axes is not visible), [replaced element](https://www.w3.org/TR/css-display-3/#replaced-element) content to the curved <a id="ref-for-content-edge②"></a>content edge, pointer events to the curved <a id="ref-for-border-edge③"></a>border edge, etc.

<a id="ref-for-propdef-border-radius⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As [border-radius](#propdef-border-radius) reduces the interactive area of an element authors should make sure the remaining interactive area conforms to recommended minima for the platforms they target; in particular, conforming to recommended minimum touch target sizes may require larger widths and heights when <a id="ref-for-propdef-border-radius⑥"></a>border-radius is used.

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
### <a id="corner-transitions"></a>4.4.  Color and Style Transitions

<a id="ref-for-border-width-dfn"></a>

Color and style transitions must be contained within the segment of the border that intersects the smallest rectangle that contains both border radii as well as the center of the inner curve (which may be a point representing the corner of the padding edge, if the border radii are smaller than the [border width](#border-width-dfn)).

If one of these borders is zero-width, then the other border takes up the entire transitional area. Otherwise, the center of color and style transitions between adjoining borders is a point along the curve that is a continuous monotonic function of the ratio of the border widths. However it is not defined what these transitions look like or what function maps from this ratio to a point on the curve.

![Illustration of the transition region on curved corners](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/transition-region.png)

Given these corner shapes, color and style transitions must be contained within the green region. In case D the rectangle defined by the border radii does not include the center of the inner curve (which is a sharp corner), so the transition region is expanded to include that corner. Transitions may take up the entire transition region, but are not required to: For example, a gradient color transition between two solid border styles might take up only the region bounded by the tips of the outer radii and the tips of the inner radii (represented in case D by the dark green region).

### <a id="corner-overlap"></a>4.5.  Overlapping Curves

Corner curves must not overlap: When the sum of any two adjacent border radii exceeds the size of the border box, UAs must proportionally reduce the used values of all border radii until none of them overlap. The algorithm for reducing radii is as follows:

Let <var>f</var> = min(<var>L<sub>i</sub></var>/<var>S<sub>i</sub></var>), where <var>i</var> ∈ {top, right, bottom, left}, <var>S<sub>i</sub></var> is the sum of the two corresponding radii of the corners on side <var>i</var>, and <var>L<sub>top</sub></var> = <var>L<sub>bottom</sub></var> = the width of the box, and <var>L<sub>left</sub></var> = <var>L<sub>right</sub></var> = the height of the box. If <var>f</var> \< 1, then all corner radii are reduced by multiplying them by <var>f</var>.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This formula ensures that quarter circles remain quarter circles and large radii remain larger than smaller ones, but it may reduce corners that were already small enough, which may make borders of nearby elements that should look the same look different.

If the curve interferes with UI elements such as scrollbars, the UA may further reduce the used value of the affected border radii (and only the affected border radii) as much as necessary, but no more.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-830e0800"></a> For example, the borders A of the [figure below](#reduced-radius) might be the result of
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
> <a id="reduced-radius"></a> ![\[image: rectangle with two tiny rounded corners and two very large ones, on opposite corners\]](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/corner-large-mix.png)
>
> <a id="ref-for-propdef-height"></a>
>
> These rounded corner might be the result of 'width: 6em; height: 2.5em; border-radius: 0.5em 2em 0.5em 2em'' for A; and ditto but with [height: 2em](https://www.w3.org/TR/css-sizing-3/#propdef-height) for B.

### <a id="border-radius-tables"></a>4.6.  Effect on Tables

<a id="ref-for-propdef-border-radius⑦"></a>

<a id="ref-for-value-def-inline-table"></a>

<a id="ref-for-value-def-table-cell"></a>

<a id="ref-for-propdef-border-collapse"></a>

<a id="ref-for-valdef-border-collapse-collapse"></a>

The [border-radius](#propdef-border-radius) properties do apply to table, [inline-table](https://www.w3.org/TR/CSS21/tables.html#value-def-inline-table), and [table-cell](https://www.w3.org/TR/CSS21/tables.html#value-def-table-cell) boxes in separated borders mode ([border-collapse: separate](https://www.w3.org/TR/CSS21/tables.html#propdef-border-collapse)). When <a id="ref-for-propdef-border-collapse①"></a>border-collapse is [collapse](https://drafts.csswg.org/css2/#valdef-border-collapse-collapse), they have no effect.

## <a id="border-images"></a>5.  Border Images

<a id="ref-for-propdef-border-image-source"></a>

<a id="ref-for-border-image-area"></a>

<a id="ref-for-propdef-border-width⑦"></a>

<a id="ref-for-propdef-border-style⑦"></a>

Authors can specify an image to be used in place of the border styles. In this case, the border’s design is taken from the sides and corners of an image specified with [border-image-source](#propdef-border-image-source), whose pieces may be sliced, scaled, and stretched in various ways to fit the size of the [border image area](#border-image-area). The border-image properties do not affect layout: layout of the box, its content, and surrounding content is based on the [border-width](#propdef-border-width) and [border-style](#propdef-border-style) properties only.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-8488cbd9"></a> This example creates a top and bottom border consisting of a whole number of orange diamonds and a left and right border of a single, stretched diamond. The corners are diamonds of a different color. The image to tile is as follows. Apart from the diamonds, it is transparent:
>
> ![Tile for border](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/border.png)
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
> ![element with a diamond border](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/borderresult.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-28370333"></a> This shows a more complicated example, demonstrating how the border image corresponds to the fallback border-style but can also extend beyond the border area. The border image is a wavy green border with an extended corner effect:
>
> ![Diagram: The border image shows a wavy green border with more exaggerated waves towards the corners, which are capped by a disconnected green circle. Four cuts at 124px offsets from each side divide the image into 124px-wide square corners, 124px-wide but thin side slices, and a small center square.](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/groovy-border-image-slice.png)
>
> <a id="ref-for-propdef-border-image-source①"></a>
>
> <a id="ref-for-propdef-border-image-slice"></a>
>
> The [border-image-source](#propdef-border-image-source) image, with the four [border-image-slice](#propdef-border-image-slice) cuts at 124px dividing the image into nine parts.
>
> The rest of the border properties then interact to lay out the tiles as follows:
>
> ![Diagram: The image-less (fallback) rendering has a green double border. The rendering with border-image shows the wavy green border, ith the waves getting longer as they reach the corners. The corner tiles render as 124px-wide squares and the side tiles repeat a whole number of times to fill the space in between. Because of the gradual corner effects, the tiles extend deep into the padding area. The whole border image effect is outset 31px, so that the troughs of the waves align just outside the padding edge.](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/border-image.png)
>
> Diagram of all border-image properties and how they interact, and showing the rendering with and without the border-image in effect.
>
> <a id="ref-for-propdef-border-width⑧"></a>
>
> <a id="ref-for-propdef-border-image-width①"></a>
>
> <a id="ref-for-border-image-area①"></a>
>
> <a id="ref-for-border-box⑥"></a>
>
> <a id="ref-for-margin-area"></a>
>
> Here, even though the [border-width](#propdef-border-width) is 12px, the [border-image-width](#propdef-border-image-width) property computes to 124px. The [border image area](#border-image-area) is then outset 31px from the [border box](https://www.w3.org/TR/css-box-4/#border-box) and into the [margin area](https://www.w3.org/TR/css-box-4/#margin-area). If the border-image fails to load (or if border images are not supported by the UA), the fallback rendering uses a green double border.

<a id="ref-for-propdef-border⑧"></a>

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

### <a id="border-image-source"></a>5.1.  Image Source: the [border-image-source](#propdef-border-image-source) property<a id="the-border-image-source"></a>

| Field               | Definition                                                                                                                                                                                                                                               |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-image-source"></a>border-image-source                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-image②"></a><a id="ref-for-comb-one④⓪"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image)                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-valdef-border-collapse-collapse①"></a><a id="ref-for-propdef-border-collapse②"></a>All elements, except internal table elements when [border-collapse](https://www.w3.org/TR/CSS21/tables.html#propdef-border-collapse) is [collapse](https://drafts.csswg.org/css2/#valdef-border-collapse-collapse) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-image③"></a>the keyword none or the computed [\<image\>](https://www.w3.org/TR/css-images-3/#typedef-image)                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                 |

<a id="ref-for-propdef-border-style⑧"></a>

<a id="ref-for-border-image-slice-fill"></a>

<a id="ref-for-propdef-border-image-slice①"></a>

Specifies an image to use as a border in place of the rendering specified by the [border-style](#propdef-border-style) properties and, if given the [fill](#border-image-slice-fill) keyword in [border-image-slice](#propdef-border-image-slice), as an additional image backdrop for the element. If the value is none or if the image cannot be displayed (or the property doesn’t apply), the border styles will be used; otherwise the element’s <a id="ref-for-propdef-border-style⑨"></a>border-style borders are not drawn and this <a id="border-image-dfn"></a>border image is drawn as described in the sections below.

<a id="ref-for-propdef-border-image-slice②"></a>

### <a id="border-image-slice"></a>5.2.  Image Slicing: the [border-image-slice](#propdef-border-image-slice) property<a id="the-border-image-slice"></a>

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-image-slice"></a>border-image-slice                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-opt⑥"></a><a id="ref-for-comb-all①"></a><a id="ref-for-mult-num-range⑧"></a><a id="ref-for-percentage-value②"></a><a id="ref-for-comb-one④①"></a><a id="ref-for-number-value"></a>\[[\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#percentage-value)\][{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) [&#x26;&#x26;](https://www.w3.org/TR/css-values-4/#comb-all) fill[?](https://www.w3.org/TR/css-values-4/#mult-opt) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 100%                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-valdef-border-collapse-collapse②"></a><a id="ref-for-propdef-border-collapse③"></a>All elements, except internal table elements when [border-collapse](https://www.w3.org/TR/CSS21/tables.html#propdef-border-collapse) is [collapse](https://drafts.csswg.org/css2/#valdef-border-collapse-collapse)                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | refer to size of the border image                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-border-image-slice-fill①"></a>four values, each either a number or percentage; plus a [fill](#border-image-slice-fill) keyword if specified                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |

<a id="ref-for-border-image-slice-fill②"></a>

This property specifies inward offsets from the top, right, bottom, and left edges of the image, dividing it into nine regions: four corners, four edges and a middle. The middle image part is discarded (treated as fully transparent) unless the [fill](#border-image-slice-fill) keyword is present. (It is drawn over the background; see [Drawing the Border Image](#border-image-process).)

If there is only one component value, it applies to all sides. If there are two values, the top and bottom are set to the first value and the right and left are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values they apply to the top, right, bottom, and left, respectively.

<a id="ref-for-percentage-value③"></a>

<a id="valdef-border-image-slice-percentage-0"></a>[\<percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#percentage-value)

Percentages are relative to the size of the image: the width of the image for the horizontal offsets, the height for vertical offsets.

<a id="ref-for-number-value①"></a>

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

![Diagram: two horizontal cuts and two vertical cuts through an image](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/slice.png)

Diagram illustrating the cuts corresponding to the value 25% 30% 12% 20%

<a id="ref-for-propdef-border-image-width②"></a>

### <a id="border-image-width"></a>5.3.  Drawing Areas: the [border-image-width](#propdef-border-image-width) property<a id="the-border-image-width"></a>

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                       |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-image-width"></a>border-image-width                                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range⑨"></a><a id="ref-for-number-value②"></a><a id="ref-for-comb-one④②"></a><a id="ref-for-typedef-length-percentage①⑧"></a>\[ [\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) <a id="ref-for-comb-one④③"></a>\| auto \][{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 1                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-valdef-border-collapse-collapse③"></a><a id="ref-for-propdef-border-collapse④"></a>All elements, except internal table elements when [border-collapse](https://www.w3.org/TR/CSS21/tables.html#propdef-border-collapse) is [collapse](https://drafts.csswg.org/css2/#valdef-border-collapse-collapse)                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | <a id="ref-for-border-image-area③"></a>Relative to width/height of the [border image area](#border-image-area)                                                                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-typedef-length-percentage①⑨"></a><a id="ref-for-valdef-border-image-width-auto"></a>four values, each either a number, the keyword [auto](#valdef-border-image-width-auto), or a computed [\<length-percentage\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage) value                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                                                                                                                                                |

<a id="ref-for-border-image-dfn"></a>

<a id="ref-for-border-box⑦"></a>

<a id="ref-for-propdef-border-image-outset"></a>

The [border image](#border-image-dfn) is drawn inside an area called the <a id="border-image-area"></a>border image area. This is an area whose boundaries by default correspond to the [border box](https://www.w3.org/TR/css-box-4/#border-box), see [border-image-outset](#propdef-border-image-outset).

<a id="ref-for-propdef-border-image-width③"></a>

<a id="ref-for-border-image-area④"></a>

The four values of [border-image-width](#propdef-border-image-width) specify offsets that are used to divide the [border image area](#border-image-area) into nine <a id="border-image-region"></a>regions. The offsets represent inward distances from the top, right, bottom, and left sides of the area, respectively.

If there is only one component value, it applies to all sides. If there are two values, the top and bottom are set to the first value and the right and left are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values they apply to the top, right, bottom, and left, respectively.

Values have the following meanings:

<a id="ref-for-typedef-length-percentage②⓪"></a>

<a id="valdef-border-image-width-length-percentage-0"></a>[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage)

<a id="ref-for-border-image-area⑤"></a>

Percentages refer to the size of the [border image area](#border-image-area): the width of the area for horizontal offsets, the height for vertical offsets.

<a id="ref-for-number-value③"></a>

<a id="valdef-border-image-width-number-0"></a>[\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value)

Numbers represent multiples of the corresponding computed [border-width](#border-width).

<a id="valdef-border-image-width-auto"></a>auto

<a id="ref-for-propdef-border-width⑨"></a>

<a id="ref-for-natural-dimensions①"></a>

<a id="ref-for-propdef-border-image-slice④"></a>

<a id="ref-for-natural-size②"></a>

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

### <a id="border-image-outset"></a>5.4.  Edge Overhang: the [border-image-outset](#propdef-border-image-outset) property<a id="the-border-image-outset"></a>

| Field               | Definition                                                                                                                                                                                                                                                                                                                                    |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-image-outset"></a>border-image-outset                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range①⓪"></a><a id="ref-for-number-value④"></a><a id="ref-for-comb-one④④"></a><a id="ref-for-length-value②"></a>\[ [\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value) [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value) \][{1,4}](https://www.w3.org/TR/css-values-4/#mult-num-range) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 0                                                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-valdef-border-collapse-collapse④"></a><a id="ref-for-propdef-border-collapse⑤"></a>All elements, except internal table elements when [border-collapse](https://www.w3.org/TR/CSS21/tables.html#propdef-border-collapse) is [collapse](https://drafts.csswg.org/css2/#valdef-border-collapse-collapse)                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | four values, each a number or absolute length                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | by computed value                                                                                                                                                                                                                                                                                                                             |

<a id="ref-for-border-image-area⑦"></a>

<a id="ref-for-border-box⑧"></a>

The values specify the amount by which the [border image area](#border-image-area) extends beyond the [border box](https://www.w3.org/TR/css-box-4/#border-box).

If there is only one component value, it applies to all sides. If there are two values, the top and bottom are set to the first value and the right and left are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values they apply to the top, right, bottom, and left, respectively.

<a id="ref-for-length-value③"></a>

[\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value)

Represents an outset of the specified length.

<a id="ref-for-number-value⑤"></a>

[\<number \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#number-value)

Represents an outset of the specified multiple of the corresponding computed [border-width](#border-width).

Negative values are invalid.

<a id="ref-for-border-box⑨"></a>

Portions of the border-image that are rendered outside the [border box](https://www.w3.org/TR/css-box-4/#border-box) do not trigger scrolling. Also such portions are invisible to mouse events and do not capture such events on behalf of the element.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Even though they never cause a scrolling mechanism, outset images may still be clipped by an ancestor or by the viewport.

<a id="ref-for-propdef-border-image-repeat"></a>

### <a id="border-image-repeat"></a>5.5.  Image Tiling: the [border-image-repeat](#propdef-border-image-repeat) property<a id="the-border-image-repeat"></a>

| Field               | Definition                                                                                                                                                                                                                                               |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-image-repeat"></a>border-image-repeat                                                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-num-range①①"></a><a id="ref-for-comb-one④⑤"></a>\[ stretch [\|](https://www.w3.org/TR/css-values-4/#comb-one) repeat <a id="ref-for-comb-one④⑥"></a>\| round <a id="ref-for-comb-one④⑦"></a>\| space \][{1,2}](https://www.w3.org/TR/css-values-4/#mult-num-range)                         |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | stretch                                                                                                                                                                                                                                                  |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | <a id="ref-for-valdef-border-collapse-collapse⑤"></a><a id="ref-for-propdef-border-collapse⑥"></a>All elements, except internal table elements when [border-collapse](https://www.w3.org/TR/CSS21/tables.html#propdef-border-collapse) is [collapse](https://drafts.csswg.org/css2/#valdef-border-collapse-collapse) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | two keywords, one per axis                                                                                                                                                                                                                               |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                 |

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

### <a id="border-image-process"></a>5.6.  Drawing the Border Image

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

    - <a id="ref-for-propdef-background-repeat⑥"></a>

      <a id="ref-for-valdef-background-repeat-round③"></a>

      <a id="ref-for-border-image-area⑨"></a>

      <a id="ref-for-valdef-border-image-repeat-round"></a>

      If the first keyword is [round](#valdef-border-image-repeat-round), the top, middle and bottom images are resized in width, so that exactly a whole number of them fit in the middle region of the [border image area](#border-image-area), exactly as for [round](#valdef-background-repeat-round) in the [background-repeat](#propdef-background-repeat) property.

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

### <a id="border-image"></a>5.7.  Border Image Shorthand: the [border-image](#propdef-border-image) property<a id="the-border-image"></a>

| Field               | Definition                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-border-image"></a>border-image                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-propdef-border-image-repeat②"></a><a id="ref-for-propdef-border-image-outset②"></a><a id="ref-for-mult-opt⑦"></a><a id="ref-for-comb-one④⑧"></a><a id="ref-for-propdef-border-image-width①⓪"></a><a id="ref-for-propdef-border-image-slice⑥"></a><a id="ref-for-comb-any①⑤"></a><a id="ref-for-propdef-border-image-source⑤"></a>[\<'border-image-source'\>](#propdef-border-image-source) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) [\<'border-image-slice'\>](#propdef-border-image-slice) \[ / [\<'border-image-width'\>](#propdef-border-image-width) [\|](https://www.w3.org/TR/css-values-4/#comb-one) / <a id="ref-for-propdef-border-image-width①①"></a>\<'border-image-width'\>[?](https://www.w3.org/TR/css-values-4/#mult-opt) / [\<'border-image-outset'\>](#propdef-border-image-outset) \]<a id="ref-for-mult-opt⑧"></a>? <a id="ref-for-comb-any①⑥"></a>\|\| [\<'border-image-repeat'\>](#propdef-border-image-repeat) |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | See individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |

<a id="ref-for-propdef-border-image-source⑥"></a>

<a id="ref-for-propdef-border-image-slice⑦"></a>

<a id="ref-for-propdef-border-image-width①②"></a>

<a id="ref-for-propdef-border-image-outset③"></a>

<a id="ref-for-propdef-border-image-repeat③"></a>

<a id="ref-for-initial-value⑤"></a>

This is a shorthand property for setting [border-image-source](#propdef-border-image-source), [border-image-slice](#propdef-border-image-slice), [border-image-width](#propdef-border-image-width), [border-image-outset](#propdef-border-image-outset), and [border-image-repeat](#propdef-border-image-repeat) in a single declaration. Omitted values are set to their [initial values](https://www.w3.org/TR/css-cascade-5/#initial-value).

### <a id="border-image-tables"></a>5.8. Effect on Tables

<a id="ref-for-propdef-border-image⑥"></a>

<a id="ref-for-propdef-border-collapse⑦"></a>

<a id="ref-for-valdef-border-collapse-collapse⑥"></a>

The [border-image](#propdef-border-image) properties apply to the border of tables and inline tables that have [border-collapse](https://www.w3.org/TR/CSS21/tables.html#propdef-border-collapse) set to [collapse](https://drafts.csswg.org/css2/#valdef-border-collapse-collapse). However, this specification does not define how such an image border is rendered. In particular, it does not define how the image border interacts with the borders of cells, rows and row groups at the edges of the table (see [border conflict resolution](css2--tables.html--201812dd6e3c.md#border-conflict-resolution) in [\[CSS2\]](#biblio-css2)).

It is expected that a future specification will define the rendering. It is recommended that UAs do not apply border images to tables with collapsed borders until then.

## <a id="misc"></a>6. Miscellaneous Effects

<a id="ref-for-propdef-box-decoration-break②"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [box-decoration-break](https://www.w3.org/TR/css-break-4/#propdef-box-decoration-break) property, which defines how backgrounds and borders apply to a fragmented box, has been moved to the [CSS Fragmentation Module](https://www.w3.org/TR/css-break/). [\[CSS-BREAK-3\]](#biblio-css-break-3)

<a id="ref-for-propdef-box-shadow③"></a>

### <a id="box-shadow"></a>6.1. Drop Shadows: the [box-shadow](#propdef-box-shadow) property<a id="the-box-shadow"></a>

| Field               | Definition                                                                                                                                                                                                                                                                                                              |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-box-shadow"></a>box-shadow                                                                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-mult-comma⑧"></a><a id="ref-for-typedef-shadow"></a><a id="ref-for-comb-one④⑨"></a>none [\|](https://www.w3.org/TR/css-values-4/#comb-one) [\<shadow\>](#typedef-shadow)[\#](https://www.w3.org/TR/css-values-4/#mult-comma)                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | none                                                                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | N/A                                                                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-shadow-inset"></a><a id="ref-for-box-shadow-none"></a>either the keyword [none](#box-shadow-none) or a list, each item consisting of four absolute lengths plus a computed color and optionally also a [inset](#shadow-inset) keyword                                                                                                   |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                                                             |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <a id="ref-for-shadow-inset①"></a><a id="ref-for-box-shadow-none①"></a>by computed value, treating [none](#box-shadow-none) as a zero-item list and appending blank shadows (transparent 0 0 0 0) with a corresponding [inset](#shadow-inset) keyword as needed to match the longer list if the shorter list is otherwise compatible with the longer one |

<a id="ref-for-propdef-box-shadow④"></a>

The [box-shadow](#propdef-box-shadow) property attaches one or more drop shadows to the box. The property accepts either the <a id="box-shadow-none"></a>none value, which indicates no shadows, or a comma-separated list of shadows, ordered front to back.

<a id="ref-for-typedef-shadow①"></a>

<a id="ref-for-shadow-inset②"></a>

Each shadow is given as a [\<shadow\>](#typedef-shadow), represented by 2-4 length values, an optional color, and an optional [inset](#shadow-inset) keyword. Omitted lengths are 0; omitted colors default to currentColor.

<a id="typedef-shadow"></a>

<a id="ref-for-typedef-shadow②"></a>

<a id="ref-for-typedef-color⑤"></a>

<a id="ref-for-mult-opt⑨"></a>

<a id="ref-for-comb-all②"></a>

<a id="ref-for-length-value④"></a>

<a id="ref-for-mult-num"></a>

<a id="ref-for-length-value⑤"></a>

<a id="ref-for-mult-opt①⓪"></a>

<a id="ref-for-length-value⑥"></a>

<a id="ref-for-mult-opt①①"></a>

<a id="ref-for-comb-all③"></a>

<a id="ref-for-mult-opt①②"></a>

```text
<shadow> = <color>? && [<length>{2} <length [0,∞]>? <length>?] && inset?
```
<a id="ref-for-typedef-shadow③"></a>

The components of each [\<shadow\>](#typedef-shadow) are interpreted as follows:

<a id="ref-for-length-value⑦"></a>

<a id="shadow-offset-x"></a>1st [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

Specifies the <a id="box-shadow-horizontal-offset"></a>horizontal offset of the shadow. A positive value draws a shadow that is offset to the right of the box, a negative length to the left.

<a id="ref-for-length-value⑧"></a>

<a id="shadow-offset-y"></a>2nd [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

Specifies the <a id="box-shadow-vertical-offset"></a>vertical offset of the shadow. A positive value offsets the shadow down, a negative one up.

<a id="ref-for-length-value⑨"></a>

<a id="shadow-blur-radius"></a>3rd [\<length \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#length-value)

Specifies the <a id="box-shadow-blur-radius"></a>blur radius. Negative values are invalid. If the blur value is zero, the shadow’s edge is sharp. Otherwise, the larger the value, the more the shadow’s edge is blurred. See [Shadow Blurring](#shadow-blur), below.

<a id="ref-for-length-value①⓪"></a>

<a id="shadow-spread-distance"></a>4th [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)

Specifies the <a id="box-shadow-spread-distance"></a>spread distance. Positive values cause the shadow to expand in all directions by the specified radius. Negative values cause the shadow to contract. See [Shadow Shape](#shadow-shape), below.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that for inner shadows, expanding the shadow (creating more shadow area) means contracting the shadow’s perimeter shape.

<a id="ref-for-typedef-color⑥"></a>

<a id="shadow-color"></a>[\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color)

<a id="ref-for-valdef-color-currentcolor"></a>

Specifies the color of the shadow. If the color is absent, it defaults to [currentColor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor).

<a id="shadow-inset"></a>inset

<a id="ref-for-shadow-inset③"></a>

If present, the [inset](#shadow-inset) keyword changes the drop shadow from an <a id="box-shadow-outer-box-shadow"></a>outer box-shadow (one that shadows the box onto the canvas, as if it were lifted above the canvas) to an <a id="box-shadow-inner-box-shadow"></a>inner box-shadow (one that shadows the canvas onto the box, as if the box were cut out of the canvas and shifted behind it).

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
> ![The sample code above would create a 100px×100px orange box with a 12px blue border, whose top right / bottom left corners are sharp and tob left / bottom right corners are elliptically curved. Two shadows are created: an inner one, which due to its offset and spread creates a 20px-wide band of darker orange along the top and left sides of the box (curving to match the rounded top left border shape); and an outer one, creating a 204px×204px gray duplicate of the shape seemingly behind the box, offset 24px down and 24px to the right of the box's top and left edges. Applying the 12px blur radius to the outer shadow creates a gradual shift from the shadow color to transparent along its edges which is visibly apparent for 24px centered along the edge of the shadow.](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/spread-blur.png)

#### <a id="shadow-shape"></a>6.1.1.  Shadow Shape, Spread, and Knockout

<a id="ref-for-box-shadow-outer-box-shadow"></a>

An [outer box-shadow](#box-shadow-outer-box-shadow) casts a shadow as if the border-box of the element were opaque. Assuming a spread distance of zero, its perimeter has the exact same size and shape as the border box. The shadow is drawn outside the border edge only: it is clipped inside the border-box of the element.

<a id="ref-for-box-shadow-inner-box-shadow"></a>

An [inner box-shadow](#box-shadow-inner-box-shadow) casts a shadow as if everything outside the padding edge were opaque. Assuming a spread distance of zero, its perimeter has the exact same size and shape as the padding box. The shadow is drawn inside the padding edge only: it is clipped outside the padding box of the element.

<a id="ref-for-box-shadow-spread-distance"></a>

<a id="ref-for-box-shadow-outer-box-shadow①"></a>

<a id="ref-for-box-shadow-inner-box-shadow①"></a>

If a [spread distance](#box-shadow-spread-distance) is defined, the shadow perimeter defined above is expanded outward (for [outer box-shadows](#box-shadow-outer-box-shadow)) or contracted inward (for [inner box-shadows](#box-shadow-inner-box-shadow)) by outsetting (insetting, for inner shadows) the shadow’s straight edges by the <a id="ref-for-box-shadow-spread-distance①"></a>spread distance (and flooring the resulting width/height at zero).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-96985467"></a> Below are some examples of an orange box with a blue border being given a drop shadow.
>
> <a id="box-shadow-samples"></a>
>
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
> ![A round-cornered box with a light gray shadow the same shape as the border box offset 10px to the right and 10px down from directly underneath the box.](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/shadow-outer-round.png)
>
> **Square-corner reference image**
>
> ![A square-cornered box with a light gray shadow the same shape as the border box offset 10px to the right and 10px down from directly underneath the box.](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/shadow-outer-square.png)
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
> ![A round-cornered box with a light gray shadow the inverse shape of the padding box filling 10px in from the top and left edges (just inside the border).](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/shadow-inner-round.png)
>
> **Square-corner reference image**
>
> ![A square-cornered box with a light gray shadow the inverse shape of the padding box filling 10px in from the top and left edges (just inside the border).](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/shadow-inner-square.png)
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
> ![A round-cornered box with a light gray shadow the same shape as the box but 20px taller and wider and offset so that the top and left edges of the shadow are directly underneath the top and left edges of the box.](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/shadow-outer-spread-round.png)
>
> **Square-corner reference image**
>
> ![A square-cornered box with a light gray shadow the same shape as the box but 20px taller and wider and offset so that the top and left edges of the shadow are directly underneath the top and left edges of the box.](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/shadow-outer-spread-square.png)
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
> ![A round-cornered box with a light gray shadow the inverse shape of the box but 20px narrower and shorter filling 20px in from the top and left edges (just inside the border).](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/shadow-inner-spread-round.png)
>
> **Square-corner reference image**
>
> ![A round-cornered box with a light gray shadow the inverse shape of the box but 20px narrower and shorter filling 20px in from the top and left edges (just inside the border).](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/images/shadow-inner-spread-square.png)
>

<a id="ref-for-box-shadow-spread-distance②"></a>

<a id="ref-for-border-radii①"></a>

To preserve the box’s shape when spread is applied, the corner radii of the shadow are also increased (decreased, for inner shadows) from the border-box (padding-box) radii by adding (subtracting) the [spread distance](#box-shadow-spread-distance) (and flooring at zero). However, in order to create a sharper corner when the border radius is small (and thus ensure continuity between round and sharp corners), when the [border radius](#border-radii) is less than the <a id="ref-for-box-shadow-spread-distance③"></a>spread distance (or in the case of an inner shadow, less than the absolute value of a negative <a id="ref-for-box-shadow-spread-distance④"></a>spread distance), the <a id="ref-for-box-shadow-spread-distance⑤"></a>spread distance is first multiplied by the proportion 1 + (<var>r</var>-1)<sup>3</sup>, where <var>r</var> is the ratio of the border radius to the <a id="ref-for-box-shadow-spread-distance⑥"></a>spread distance, in calculating the corner radii of the spread shadow shape. For example, if the border radius is 10px and the <a id="ref-for-box-shadow-spread-distance⑦"></a>spread distance is 20px (<var>r</var> = .5), the corner radius of the shadow shape will be 10px + 20px × (1 + (.5 - 1)<sup>3</sup>) = 27.5px rather than 30px. This adjustment is applied independently to the radii in each dimension.

<a id="ref-for-propdef-border-image⑦"></a>

The [border-image](#propdef-border-image) does not affect the shape of the box-shadow.

#### <a id="shadow-blur"></a>6.1.2.  Blurring Shadow Edges

<a id="ref-for-box-shadow-blur-radius"></a>

A non-zero [blur radius](#box-shadow-blur-radius) indicates that the resulting shadow should be blurred, such as by a Gaussian filter. The exact algorithm is not defined; however the resulting shadow must approximate (with each pixel being within 5% of its expected value) the image that would be generated by applying to the shadow a Gaussian blur with a standard deviation equal to half the blur radius.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This means for a long, straight shadow edge, the blur radius will create a visibly apparent color transition approximately the twice length of the blur radius that is perpendicular to and centered on the shadow’s edge, and that ranges from almost the full shadow color at the endpoint inside the shadow to almost fully transparent at the endpoint outside it.

#### <a id="shadow-layers"></a>6.1.3.  Layering, Layout, and Other Details

<a id="ref-for-box-shadow-outer-box-shadow②"></a>

<a id="ref-for-box-shadow-inner-box-shadow②"></a>

The shadow effects are applied front-to-back: the first shadow is on top and the others are layered behind. Shadows do not influence layout and may overlap (or be overlapped by) other boxes and text or their shadows. In terms of stacking contexts and the painting order, the [outer box-shadows](#box-shadow-outer-box-shadow) of an element are drawn immediately below the background of that element, and the [inner shadows](#box-shadow-inner-box-shadow) of an element are drawn immediately above the background of that element (below the borders and border image, if any).

<a id="ref-for-principal-box"></a>

<a id="ref-for-propdef-box-decoration-break③"></a>

Unless otherwise specified, drop shadows are only applied to the [principal box](https://www.w3.org/TR/css-display-3/#principal-box). If the affected box has multiple fragments, the shadows are applied as specified in [box-decoration-break](https://www.w3.org/TR/css-break-4/#propdef-box-decoration-break).

<a id="ref-for-scrollable-overflow-region②"></a>

Shadows do not trigger scrolling or increase the size of the [scrollable overflow area](https://www.w3.org/TR/css-overflow-3/#scrollable-overflow-region).

Outer shadows have no effect on internal table elements in the collapsing border model. If a shadow is defined for single border edge in the collapsing border model that has multiple border thicknesses (e.g. an outer shadow on a table where one row has thicker borders than the others, or an inner shadow on a rowspanning table cell that adjoins cells with different border thicknesses), the exact position and rendering of its shadows are undefined

## <a id="levels"></a>7.  Levels

<em>This section is informative.</em> CSS has different levels of features, each a subset of the other. (See [\[CSS-2017\]](#biblio-css-2017) for a full explanation.) The lists below describe which features from this specification are in each level.

### <a id="level-1"></a>7.1.  Level 1

- <a id="ref-for-propdef-background-color⑧"></a>

  [background-color](#propdef-background-color)

- <a id="ref-for-propdef-background-image⑦"></a>

  [background-image](#propdef-background-image) only one image (no layers)

- <a id="ref-for-valdef-background-repeat-no-repeat②"></a>

  <a id="ref-for-valdef-background-repeat-repeat-y①"></a>

  <a id="ref-for-valdef-background-repeat-repeat-x"></a>

  <a id="ref-for-valdef-background-repeat-repeat①"></a>

  <a id="ref-for-propdef-background-repeat⑦"></a>

  [background-repeat](#propdef-background-repeat): only [repeat](#valdef-background-repeat-repeat) \| [repeat-x](#valdef-background-repeat-repeat-x) \| [repeat-y](#valdef-background-repeat-repeat-y) \| [no-repeat](#valdef-background-repeat-no-repeat)

- <a id="ref-for-valdef-background-attachment-fixed⑥"></a>

  <a id="ref-for-valdef-background-attachment-scroll①"></a>

  <a id="ref-for-propdef-background-attachment④"></a>

  [background-attachment](#propdef-background-attachment): only [scroll](#valdef-background-attachment-scroll) \| [fixed](#valdef-background-attachment-fixed)

- <a id="ref-for-propdef-background-position①⓪"></a>

  [background-position](#propdef-background-position): only one or two values allowed

- <a id="ref-for-propdef-background③"></a>

  [background](#propdef-background) shorthand: only color, image, repeat, attachment and position

- <a id="ref-for-propdef-border-color⑦"></a>

  [border-color](#propdef-border-color) properties

- <a id="ref-for-propdef-border-style①⓪"></a>

  [border-style](#propdef-border-style) properties

- <a id="ref-for-propdef-border-width①⓪"></a>

  [border-width](#propdef-border-width) properties

- <a id="ref-for-propdef-border⑨"></a>

  <a id="ref-for-propdef-border-left③"></a>

  <a id="ref-for-propdef-border-right"></a>

  <a id="ref-for-propdef-border-bottom"></a>

  <a id="ref-for-propdef-border-top"></a>

  [border-top](#propdef-border-top), [border-bottom](#propdef-border-bottom), [border-right](#propdef-border-right), [border-left](#propdef-border-left), and [border](#propdef-border) shorthands

### <a id="level-2"></a>7.2.  Level 2

- <a id="ref-for-propdef-background-color⑨"></a>

  [background-color](#propdef-background-color)

- <a id="ref-for-propdef-background-image⑧"></a>

  [background-image](#propdef-background-image): only one image (no layers)

- <a id="ref-for-valdef-background-repeat-no-repeat③"></a>

  <a id="ref-for-valdef-background-repeat-repeat-y②"></a>

  <a id="ref-for-valdef-background-repeat-repeat-x①"></a>

  <a id="ref-for-valdef-background-repeat-repeat②"></a>

  <a id="ref-for-propdef-background-repeat⑧"></a>

  [background-repeat](#propdef-background-repeat): only [repeat](#valdef-background-repeat-repeat) \| [repeat-x](#valdef-background-repeat-repeat-x) \| [repeat-y](#valdef-background-repeat-repeat-y) \| [no-repeat](#valdef-background-repeat-no-repeat)

- <a id="ref-for-valdef-background-attachment-fixed⑦"></a>

  <a id="ref-for-valdef-background-attachment-scroll②"></a>

  <a id="ref-for-propdef-background-attachment⑤"></a>

  [background-attachment](#propdef-background-attachment): only [scroll](#valdef-background-attachment-scroll) \| [fixed](#valdef-background-attachment-fixed)

- <a id="ref-for-propdef-background-position①①"></a>

  [background-position](#propdef-background-position): only one or two values allowed

- <a id="ref-for-propdef-background④"></a>

  [background](#propdef-background): only color, image, repeat, attachment and position

- <a id="ref-for-propdef-border-color⑧"></a>

  [border-color](#propdef-border-color) properties

- <a id="ref-for-propdef-border-style①①"></a>

  [border-style](#propdef-border-style) properties

- <a id="ref-for-propdef-border-width①①"></a>

  [border-width](#propdef-border-width) properties

- <a id="ref-for-propdef-border①⓪"></a>

  <a id="ref-for-propdef-border-left④"></a>

  <a id="ref-for-propdef-border-right①"></a>

  <a id="ref-for-propdef-border-bottom①"></a>

  <a id="ref-for-propdef-border-top①"></a>

  [border-top](#propdef-border-top), [border-bottom](#propdef-border-bottom), [border-right](#propdef-border-right), [border-left](#propdef-border-left), and [border](#propdef-border) shorthands

### <a id="level-3"></a>7.3.  Level 3

- All features described in the CSS Backgrounds and Borders Module Level 3

## <a id="changes"></a>8.  Changes

### <a id="changes-2023-02"></a>8.1.  Changes since the 14 February 2023 Candidate Recommendation Snapshot

- <a id="ref-for-propdef-background-position①②"></a>

  Defined serialization of [background-position](#propdef-background-position) in [§ 2.6.1 Serialization of background-position values](#bg-position-serialization). ([Issue 2274](https://github.com/w3c/csswg-drafts/issues/2274))

- <a id="ref-for-propdef-background⑤"></a>

  Reverted [change moving \<color\> to the front of \<final-bg-layer\>](#bg-color-move) in the [background](#propdef-background) shorthand. ([Issue 8496](https://github.com/w3c/csswg-drafts/issues/8496))

- <a id="ref-for-snap-a-length-as-a-border-width①"></a>

  Specified [rounding of border widths](https://www.w3.org/TR/css-values-4/#snap-a-length-as-a-border-width) to device pixels. ([Issue 7434](https://github.com/w3c/csswg-drafts/issues/7434))

- <a id="ref-for-propdef-overflow②"></a>

  <a id="ref-for-propdef-border-radius⑧"></a>

  Specified interaction of [border-radius](#propdef-border-radius) and per-axis [overflow](https://www.w3.org/TR/CSS21/visufx.html#propdef-overflow) longhands. ([Issue 5210](https://github.com/w3c/csswg-drafts/issues/5210))

- <a id="ref-for-propdef-background-image⑨"></a>

  Fixed an error in the Computed Value line of [background-image](#propdef-background-image). ([Issue 8604](https://github.com/w3c/csswg-drafts/issues/8604))

- <a id="ref-for-typedef-visual-box⑦"></a>

  Removed the \<box\> definition (replacing it with a [\<visual-box\>](https://www.w3.org/TR/css-box-4/#typedef-visual-box) reference), as the Box Module now defines these terms.

- Aligned property definition tables with the latest expectations to include “Animation Type” and “Logical Property Group”.

- <a id="ref-for-css-value-definition-syntax"></a>

  Streamlined property grammar definitions using the latest [value definition syntax](https://www.w3.org/TR/css-values-4/#css-value-definition-syntax).

- Fixed the definition for where box shadows apply. ([Issue 9286](https://github.com/w3c/csswg-drafts/issues/9286))

### <a id="changes-2020-12"></a>8.2.  Changes since the 22 December 2020 Candidate Recommendation Snapshot

- <a id="ref-for-propdef-display①"></a>

  Clarified that the rule about not propagating backgrounds from the root when it doesn’t generate boxes only applies to [display: none](https://www.w3.org/TR/CSS21/visuren.html#propdef-display), not <a id="ref-for-propdef-display②"></a>display: contents. ([Issue 3779](https://github.com/w3c/csswg-drafts/issues/3779))

- <a id="ref-for-valdef-line-width-thick"></a>

  <a id="ref-for-valdef-line-width-medium①"></a>

  <a id="ref-for-valdef-line-width-thin"></a>

  <a id="ref-for-propdef-border-width①②"></a>

  Precisely defined the size of [border-width](#propdef-border-width) [thin](#valdef-line-width-thin), [medium](#valdef-line-width-medium), and [thick](#valdef-line-width-thick). ([Issue 7254](https://github.com/w3c/csswg-drafts/issues/7254))

- Minor editorial improvements.

### <a id="changes-2017-10"></a>8.3.  Changes since the 17 October 2017 Candidate Recommendation

- <a id="ref-for-funcdef-filter-drop-shadow"></a>

  <a id="ref-for-propdef-text-shadow"></a>

  <a id="ref-for-typedef-shadow④"></a>

  Inverted order of [\<shadow\>](#typedef-shadow) grammar to match browser serialization and [text-shadow](https://www.w3.org/TR/css-text-decor-4/#propdef-text-shadow)/[drop-shadow()](https://www.w3.org/TR/filter-effects-1/#funcdef-filter-drop-shadow). ([Issue 2305](https://github.com/w3c/csswg-drafts/issues/2305))

  > <a id="ref-for-typedef-shadow⑤"></a>
  >
  > <a id="ref-for-typedef-color⑦"></a>
  >
  > <a id="ref-for-length-value①①"></a>
  >
  > <a id="ref-for-typedef-color⑧"></a>
  >
  > ```text
  > <shadow> = inset<color>? && <length>{2,4} && <color>inset?
  > ```
- <a id="ref-for-box-shadow-spread-distance①⓪"></a>

  <a id="ref-for-border-radii②"></a>

  <a id="ref-for-box-shadow-spread-distance⑧"></a>

  Spread radius adjustment is only applied to shadows and margins where the radius of curvature grows, rather than shrinks. ([Issue 1900](https://github.com/w3c/csswg-drafts/issues/1900))

  > To preserve the box’s shape when spread is applied, the corner radii of the shadow are also increased (decreased, for inner shadows) from the border-box radii by adding (subtracting) the [spread distance](#box-shadow-spread-distance) (and flooring at zero). However, in order to create a sharper corner when the border radius is small <u>(and thus ensure continuity between round and sharp corners)</u> , when the [border radius](#border-radii) is less than the <a id="ref-for-box-shadow-spread-distance⑨"></a>spread distance <u>(or in the case of an inner shadow, less than the absolute value of a negative [spread distance](#box-shadow-spread-distance))</u> , the <a id="ref-for-box-shadow-spread-distance①①"></a>spread distance is multiplied by the proportion 1 + (<var>r</var>-1)<sup>3</sup>, where <var>r</var> is the ratio of the border radius to the <a id="ref-for-box-shadow-spread-distance①②"></a>spread distance, in calculating the corner radii of the spread shadow shape.

- <a id="ref-for-propdef-color②"></a>

  <a id="ref-for-valdef-color-currentcolor②"></a>

  <a id="ref-for-valdef-color-currentcolor①"></a>

  <a id="ref-for-typedef-shadow⑥"></a>

  <a id="ref-for-typedef-color⑨"></a>

  Clarified that an omitted [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) in a [\<shadow\>](#typedef-shadow) defaults to [currentColor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor), not some mysterious unnamed value with the same behavior. ([2766](https://github.com/w3c/csswg-drafts/issues/2766))

  > If the color is absent, <u>it defaults to [currentColor](https://www.w3.org/TR/css-color-4/#valdef-color-currentcolor)</u> ~~the used color is taken from the [color](https://www.w3.org/TR/css-color-4/#propdef-color) property.~~

- Cleaned up and regularized “Animation type” and “Computed value” lines in the property definition tables.

- <a id="ref-for-css-bracketed-range-notation"></a>

  Changed syntax to use the new [CSS bracketed range notation](https://www.w3.org/TR/css-values-4/#css-bracketed-range-notation) to reflect the prose restrictions on negative values, and corrected a few grammar definition errors introduced in the previous publication.

- Minor editorial improvements.

### <a id="changes-2014-09"></a>8.4.  Changes since the 9 September 2014 Candidate Recommendation

The following (non-trivial) changes were made to this specification since the [9 September 2014 Candidate Recommendation](https://www.w3.org/TR/2014/CR-css3-background-20140909/):

- <a id="ref-for-propdef-box-shadow⑤"></a>

  Added missing definition of [box-shadow: none](#propdef-box-shadow). (Apparently this was obvious enough that nobody noticed it was missing until now.)

- <a id="ref-for-typedef-final-bg-layer②"></a>

  <a id="ref-for-propdef-background-color①⓪"></a>

  <a id="bg-color-move"></a>Moved [\<'background-color'\>](#propdef-background-color) component of [\<final-bg-layer\>](#typedef-final-bg-layer) to the front for serialization because some authors seem to expect this even though it makes less sense?

- <a id="ref-for-propdef-border-radius⑨"></a>

  Dropped effect of [border-radius](#propdef-border-radius) from collapsed-borders tables.

- <a id="ref-for-typedef-bg-position⑥"></a>

  <a id="ref-for-typedef-position②"></a>

  Renamed [\<position\>](https://www.w3.org/TR/css-values-3/#typedef-position) back to [\<bg-position\>](#typedef-bg-position) since other properties will be eliding the three-value syntax.

### <a id="changes-2014-02"></a>8.5.  Changes since the 4 February 2014 Last Call Working Draft

The following (non-trivial) changes were made to this specification since the [4 February 2014 Last Call Working Draft](https://www.w3.org/TR/2014/WD-css3-background-20140204/):

- Fixed spread radius and margin radius calculations to only apply adjustment factor when spread/margin is larger than border radius.

- <a id="ref-for-propdef-display③"></a>

  Defined handling of canvas background when root element has [display: none](https://www.w3.org/TR/CSS21/visuren.html#propdef-display).

A full [Disposition of Comments](https://drafts.csswg.org/css-backgrounds-3/issues-lc-2014) is available.

### <a id="changes-2012-07"></a>8.6.  Changes since the 24 July 2012 Candidate Recommendation

The following (non-trivial) changes were made to this specification since the [24 July 2012 Candidate Recommendation](https://www.w3.org/TR/2012/CR-css3-background-20120724/):

- <a id="ref-for-propdef-background⑥"></a>

  <a id="ref-for-propdef-background-origin⑦"></a>

  <a id="ref-for-propdef-background-clip①⓪"></a>

  Allow [\<'background-clip'\>](#propdef-background-clip) and [\<'background-origin'\>](#propdef-background-origin) to be separated by other component values in the [background](#propdef-background) shorthand, since this is what is implemented.

- <a id="ref-for-propdef-box-shadow⑥"></a>

  <a id="ref-for-shadow-inset④"></a>

  <a id="ref-for-typedef-color①⓪"></a>

  Allow [\<color\>](https://www.w3.org/TR/css-color-5/#typedef-color) and [inset](#shadow-inset) to be interleaved in any order in [box-shadow](#propdef-box-shadow), since they are not ambiguous and CSS generally allows variant ordering where not ambiguous.

- <a id="ref-for-propdef-border-radius①⓪"></a>

  <a id="ref-for-propdef-box-shadow⑦"></a>

  Define gradually increasing corner radius formula for [box-shadow](#propdef-box-shadow) spread curvature to create continuity between sharp corners ([border-radius](#propdef-border-radius) = 0) and curved corners (<a id="ref-for-propdef-border-radius①①"></a>border-radius \> <var>spread distance</var>). This also gives better results for all intermediate states.

- Add definition for how the margin edge is curved in response to border-radius. (This is relevant for [\[CSS-SHAPES\]](#biblio-css-shapes), but does not change conformance to CSS Backgrounds and Borders Level 3.)

- <a id="ref-for-propdef-box-decoration-break④"></a>

  Removed [box-decoration-break](https://www.w3.org/TR/css-break-4/#propdef-box-decoration-break); it is now part of [\[CSS-BREAK-3\]](#biblio-css-break-3).

- <a id="ref-for-propdef-box-shadow⑧"></a>

  Tighten up the definition of spread for [box-shadow](#propdef-box-shadow).

- <a id="ref-for-propdef-border-width①③"></a>

  <a id="ref-for-propdef-border-image-width①③"></a>

  <a id="ref-for-valdef-line-style-none③"></a>

  <a id="ref-for-propdef-border-style①②"></a>

  Clarify that a [border-style](#propdef-border-style) of [none](#valdef-line-style-none) also implies an initial [border-image-width](#propdef-border-image-width) of zero (since <a id="ref-for-propdef-border-image-width①④"></a>border-image-width is initially set to the computed [border-width](#propdef-border-width), which in this case is zero).

- <a id="ref-for-propdef-background-attachment⑥"></a>

  Clarified how [background-attachment: local](#propdef-background-attachment) is affected by scrolling.

- <a id="ref-for-propdef-background-position①③"></a>

  Simplified computed value of [background-position](#propdef-background-position) to clarify that all <a id="ref-for-propdef-background-position①④"></a>background-position values are interpolable.

- Added “Animation Type” values to each property definition table.

### <a id="changes-2012-04"></a>8.7.  Changes since the 17 April 2012 Candidate Recommendation

The following (non-editorial) changes were made to this specification since the [17 April 2012 Candidate Recommendation](https://www.w3.org/TR/2012/CR-css3-background-20120417/):

- <a id="ref-for-propdef-background-position①⑤"></a>

  Fix error in computed value of [background-position](#propdef-background-position): the computed value is a list of positions.

- <a id="ref-for-propdef-border-radius①②"></a>

  Add a note pointing out that when [border-radius](#propdef-border-radius) reduces the interactive area, the width/height of the box might need to be increased.

### <a id="changes-2012LC"></a>8.8.  Changes since the 14 February 2012 “Last Call” Working Draft

The following (non-editorial) changes were made to this specification since the [14 February 2012 “Last Call” Working Draft](https://www.w3.org/TR/2012/WD-css3-background-20120214/):

These changes were in response to comments received during the review period. For details, see the full [Disposition of Comments.](https://drafts.csswg.org/css-backgrounds-3/issues-lc-2012)

- <a id="ref-for-first-letter0①"></a>

  <a id="ref-for-sel-first-line⑤"></a>

  [Section 2.1](#placement): Defined which properties from this module apply to the [::first-line](https://www.w3.org/TR/selectors-3/#sel-first-line) and [::first-letter](https://www.w3.org/TR/selectors-3/#first-letter0) pseudo-elements.

- <a id="ref-for-propdef-background-repeat⑨"></a>

  [Section 3.4](#background-repeat): Fixed the incorrect definition of the computed value of [background-repeat](#propdef-background-repeat). The value is always a pair of keywords, never a single keyword.

- <a id="ref-for-propdef-background⑦"></a>

  <a id="ref-for-propdef-background-image①⓪"></a>

  [Section 3.10](#background-image): Added the missing [background-image](#propdef-background-image) to the list of properties that the [background](#propdef-background) property reset.

- <a id="ref-for-propdef-border-width①④"></a>

  [Section 4.3](#border-width): Added that negative lengths are invalid on [border-width](#propdef-border-width).

- <a id="ref-for-propdef-border-radius①③"></a>

  [Section 5.1](#border-radius): Added that negative lengths are invalid on [border-radius](#propdef-border-radius).

- <a id="ref-for-propdef-border-image⑧"></a>

  [Section 6.8](#border-image-tables): Added a section about the effect of [border-image](#propdef-border-image) on tables with collapsed borders and added that the rendering will be defined later.

### <a id="changes-2011"></a>8.9.  Changes Since the 15 February 2011 Candidate Recommendation

The following changes were made to this specification since the [15 February 2011 Candidate Recommendation](https://www.w3.org/TR/2011/CR-css3-background-20110215/):

- <a id="ref-for-valdef-background-size-auto⑥"></a>

  <a id="ref-for-valdef-background-size-auto⑤"></a>

  <a id="ref-for-propdef-background-size⑥"></a>

  [Section 3.9](#background-size): Defined what happens if [background-size](#propdef-background-size) has two [auto](#valdef-background-size-auto) values and the image is missing a natural size.

  > If both values are [auto](#valdef-background-size-auto) then the natural width and/or height of the image should be used, if any <u>, the missing dimension (if any) behaving as ‘auto’ as described above</u> .

- [Section 5.4](#corner-transitions): Made center of [color and style transitions](#corner-transitions) undefined (within certain limits) on corner joins, since previous definition was wrong.

  > ~~The center of color and style transitions between adjoining borders is at the point on the curve that is at an angle that is proportional to the ratio of the border widths. For example, if the top and right border widths are equal, that point is at a 45° angle from the horizontal, and if the top is twice the width of the right the point is at a 30° angle from the horizontal. The line demarcating this transition is drawn between the point at that angle on the outer arc and the point at that angle on the inner arc.~~ <u>If one of these borders is zero-width, then the other border takes up the entire transitional area. Otherwise, the center of color and style transitions between adjoining borders must be proportional to the ratio of the border widths such that a function of its location is continuous with respect to this ratio. However it is not defined what these transitions look like or how “proportional” maps to a point on the curve.</u>

- [Section 6.2](#border-image-slice): Defined [slicing of border images](#border-image-slice) that must be sized first to determine slice positions. Added:

  > <u>If the image must be sized to determine the slices (for example, for SVG images with no natural size), then it is sized as for an auto-sized background, using the border image area as the default object size in place of the background positioning area.</u>

- <a id="ref-for-propdef-box-decoration-break⑤"></a>

  [Section 7.1](#misc) Optionally added bidi-imposed breaks to the types of breaks affected by [box-decoration-break](https://www.w3.org/TR/css-break-4/#propdef-box-decoration-break). Added:

  > <u>UAs may also apply ‘box-decoration-break’ to control rendering at bidi-imposed breaks, i.e. when bidi reordering causes an inline to split into non-contiguous fragments. Otherwise such breaks are always handled as ‘slice’.</u>

- <a id="ref-for-propdef-box-shadow⑨"></a>

  [Section 7.2](#box-shadow): Defined the default color of [box-shadow](#propdef-box-shadow).

  > The color is the color of the shadow. <u>If the color is absent, the used color is taken from the ‘color’ property.</u>

- <a id="ref-for-length-value①②"></a>

  <a id="ref-for-propdef-background-position①⑥"></a>

  [Section 3.6](#background-position): Clarified computed value of [background-position](#propdef-background-position).

  > ~~If one or two values are specified, for a \<length\> the absolute length, otherwise a percentage. If three or four values are specified, two pairs of a keyword plus a length or percentage.~~ <u>Two keywords representing the origin and two offsets from that origin, each given as an absolute length (if given a [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)), otherwise as a percentage.</u>

  Changed ~~Equivalent~~ to <u>Computes</u> in definition of keywords.

- Added "Animation Type" line to property definition tables.

- <a id="ref-for-typedef-position③"></a>

  [Section 3.6](#background-position): Renamed \<bg-position\> production to [\<position\>](https://www.w3.org/TR/css-values-3/#typedef-position) for easier re-use in other specifications and recast the grammar to be more verbose but easier to understand.

  > <a id="ref-for-comb-one⑤⓪"></a>
  >
  > <a id="ref-for-comb-one⑤①"></a>
  >
  > <a id="ref-for-comb-one⑤②"></a>
  >
  > <a id="ref-for-comb-one⑤③"></a>
  >
  > <a id="ref-for-comb-one⑤④"></a>
  >
  > <a id="ref-for-typedef-length-percentage②①"></a>
  >
  > <a id="ref-for-comb-one⑤⑤"></a>
  >
  > <a id="ref-for-comb-one⑤⑥"></a>
  >
  > <a id="ref-for-comb-one⑤⑦"></a>
  >
  > <a id="ref-for-comb-one⑤⑧"></a>
  >
  > <a id="ref-for-typedef-length-percentage②②"></a>
  >
  > <a id="ref-for-comb-one⑤⑨"></a>
  >
  > <a id="ref-for-comb-one⑥⓪"></a>
  >
  > <a id="ref-for-comb-one⑥①"></a>
  >
  > <a id="ref-for-typedef-length-percentage②③"></a>
  >
  > <a id="ref-for-mult-opt①③"></a>
  >
  > <a id="ref-for-comb-one⑥②"></a>
  >
  > <a id="ref-for-comb-one⑥③"></a>
  >
  > <a id="ref-for-comb-one⑥④"></a>
  >
  > <a id="ref-for-typedef-length-percentage②④"></a>
  >
  > <a id="ref-for-mult-opt①④"></a>
  >
  > <a id="ref-for-comb-all④"></a>
  >
  > <a id="ref-for-comb-one⑥⑤"></a>
  >
  > <a id="ref-for-comb-one⑥⑥"></a>
  >
  > <a id="ref-for-typedef-length-percentage②⑤"></a>
  >
  > <a id="ref-for-mult-opt①⑤"></a>
  >
  > ```text
  > bg-position<position> = [
  >   [ left | center | right | top | bottom | <length-percentage> ]
  > |
  >   [ left | center | right | <length-percentage> ]
  >   [ top | center | bottom | <length-percentage> ]?
  > |
  >   [ center | [ left | right ] <length-percentage>? ] &&
  >   [ center | [ top | bottom ] <length-percentage>? ]
  > ]
  > ```
- <a id="ref-for-propdef-border-image-repeat④"></a>

  <a id="ref-for-valdef-border-image-repeat-space③"></a>

  [Section 6.5](#border-image-repeat): Added [space](#valdef-border-image-repeat-space) keyword to [border-image-repeat](#propdef-border-image-repeat) property value table: it was included in the list of allowable values, but not in the syntax definition.

  > \[ stretch \| repeat \| round <u>\| space</u> \]{1,2}

- [Section 5.5](#corner-overlap): Corrected math error in border-radius example.

  > The height (2.5em) is enough for the specified radii (0.5em plus ~~2.5em~~ <u>2.0em</u> ).

- <a id="ref-for-propdef-border-width①⑤"></a>

  [Section 4.3](#border-width): Marked Percentages field as N/A for [border-width](#propdef-border-width), since they are not included in the property.

### <a id="changes-2009"></a>8.10.  Changes Since the 17 December 2009 Candidate Recommendation

The following changes were made to this specification since the [17 December 2009 Candidate Recommendation](https://www.w3.org/TR/2009/CR-css3-background-20091217/):

- <a id="ref-for-propdef-background-clip①①"></a>

  <a id="ref-for-valdef-background-clip-content-box"></a>

  Addition of [content-box](#valdef-background-clip-content-box) value of [background-clip](#propdef-background-clip).

- <a id="ref-for-propdef-background-origin⑧"></a>

  <a id="ref-for-propdef-background-clip①②"></a>

  <a id="ref-for-propdef-background⑧"></a>

  Change to the [background](#propdef-background) shorthand syntax for [background-clip](#propdef-background-clip) and [background-origin](#propdef-background-origin).

- <a id="ref-for-propdef-border-radius①④"></a>

  Removal of recommendation to use gradients for color transitions when [border-radius](#propdef-border-radius) produces a curve.

- (Re)Addition of box-shadow property.

- Various clarifications.

## <a id="acknowledgments"></a>9. Acknowledgments

Tapas Roy was editor of the Border Module, before it was merged with the Background Module.

<a id="ref-for-propdef-border-radius①⑤"></a>

Thanks to Ben Stucki for defining what happens with rounded corners if the two adjoining borders are of unequal thickness or one of them is zero; to Arjan Eising and Anne van Kesteren for the [border-radius](#propdef-border-radius) syntax; to Zack Weinberg for the corner transition regions diagram; and to Lea Verou, plinss, and dbaron for the corner radius adjustment formula (with special thanks to Lea for the live demo).

A set of properties for border images was initially proposed by <i>fantasai</i>. The current simplification (one image cut into nine parts) is due to Ian Hickson. (Though the original idea seems to originate with some anonymous Microsoft engineers.)

<a id="ref-for-propdef-border-image⑨"></a>

Finally, special thanks go to Brad Kemper for his feedback and suggestions for many of the features in the draft, for drawing all the box-shadow examples, and for proposing some [radical changes](http://www.bradclicks.com/cssplay/border-image/Thinking_Outside_The_Box.html) to the [border-image](#propdef-border-image) property that solved a number of problems with the earlier definition.

## <a id="privacy"></a>10. Privacy Considerations

This specification introduces no new privacy considerations.

## <a id="security"></a>11. Security Considerations

This specification introduces no new security considerations.

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

- [1st \<length\>](#shadow-offset-x), in § 6.1
- [2nd \<length\>](#shadow-offset-y), in § 6.1
- [3rd \<length \[0,∞\]\>](#shadow-blur-radius), in § 6.1
- [4th \<length\>](#shadow-spread-distance), in § 6.1
- [\<attachment\>](#typedef-attachment), in § 2.5
- auto
  - [value for background-size](#valdef-background-size-auto), in § 2.9
  - [value for border-image-width](#valdef-border-image-width-auto), in § 5.3
- [background](#propdef-background), in § 2.10
- [background-attachment](#propdef-background-attachment), in § 2.5
- [background-clip](#propdef-background-clip), in § 2.7
- [background color](#background-color-layer), in § 2.2
- [background-color](#propdef-background-color), in § 2.2
- [background image](#background-images), in § 2.3
- [background-image](#propdef-background-image), in § 2.3
- [background image layer](#background-image-layer), in § 2.1
- [background-origin](#propdef-background-origin), in § 2.8
- [background painting area](#background-painting-area), in § 2.7
- [background-position](#propdef-background-position), in § 2.6
- [background positioning area](#background-positioning-area), in § 2.8
- [background-repeat](#propdef-background-repeat), in § 2.4
- [background-size](#propdef-background-size), in § 2.9
- [\<bg-image\>](#typedef-bg-image), in § 2.3
- [\<bg-layer\>](#typedef-bg-layer), in § 2.10
- [\<bg-position\>](#typedef-bg-position), in § 2.6
- [\<bg-size\>](#typedef-bg-size), in § 2.9
- [blur radius](#box-shadow-blur-radius), in § 6.1
- [border](#propdef-border), in § 3.4
- [border-bottom](#propdef-border-bottom), in § 3.4
- [border-bottom-color](#propdef-border-bottom-color), in § 3.1
- [border-bottom-left-radius](#propdef-border-bottom-left-radius), in § 4.1
- [border-bottom-right-radius](#propdef-border-bottom-right-radius), in § 4.1
- [border-bottom-style](#propdef-border-bottom-style), in § 3.2
- [border-bottom-width](#propdef-border-bottom-width), in § 3.3
- border-box
  - [value for background-clip](#valdef-background-clip-border-box), in § 2.7
  - [value for background-origin](#valdef-background-origin-border-box), in § 2.8
- [border color](#border-color-dfn), in § 3.1
- [border-color](#propdef-border-color), in § 3.1
- [border image](#border-image-dfn), in § 5.1
- [border-image](#propdef-border-image), in § 5.7
- [border image area](#border-image-area), in § 5.3
- [border-image-outset](#propdef-border-image-outset), in § 5.4
- [border image region](#border-image-region), in § 5.3
- [border-image-repeat](#propdef-border-image-repeat), in § 5.5
- [border-image-slice](#propdef-border-image-slice), in § 5.2
- [border-image-source](#propdef-border-image-source), in § 5.1
- [border-image-width](#propdef-border-image-width), in § 5.3
- [border-left](#propdef-border-left), in § 3.4
- [border-left-color](#propdef-border-left-color), in § 3.1
- [border-left-style](#propdef-border-left-style), in § 3.2
- [border-left-width](#propdef-border-left-width), in § 3.3
- [border radius](#border-radii), in § 4.1
- [border-radius](#propdef-border-radius), in § 4.1
- [border-right](#propdef-border-right), in § 3.4
- [border-right-color](#propdef-border-right-color), in § 3.1
- [border-right-style](#propdef-border-right-style), in § 3.2
- [border-right-width](#propdef-border-right-width), in § 3.3
- [border style](#border-style-dfn), in § 3.2
- [border-style](#propdef-border-style), in § 3.2
- [border-top](#propdef-border-top), in § 3.4
- [border-top-color](#propdef-border-top-color), in § 3.1
- [border-top-left-radius](#propdef-border-top-left-radius), in § 4.1
- [border-top-right-radius](#propdef-border-top-right-radius), in § 4.1
- [border-top-style](#propdef-border-top-style), in § 3.2
- [border-top-width](#propdef-border-top-width), in § 3.3
- [border width](#border-width-dfn), in § 3.3
- [border-width](#propdef-border-width), in § 3.3
- [bottom](#valdef-background-position-bottom), in § 2.6
- [box-shadow](#propdef-box-shadow), in § 6.1
- [canvas background](#canvas-background), in § 2.11.1
- [canvas surface](#canvas-surface), in § 2.11
- [center](#valdef-background-position-center), in § 2.6
- [\<color\>](#shadow-color), in § 6.1
- [contain](#valdef-background-size-contain), in § 2.9
- content-box
  - [value for background-clip](#valdef-background-clip-content-box), in § 2.7
  - [value for background-origin](#valdef-background-origin-content-box), in § 2.8
- [cover](#valdef-background-size-cover), in § 2.9
- [dashed](#valdef-line-style-dashed), in § 3.2
- [dotted](#valdef-line-style-dotted), in § 3.2
- [double](#valdef-line-style-double), in § 3.2
- [fill](#border-image-slice-fill), in § 5.2
- [\<final-bg-layer\>](#typedef-final-bg-layer), in § 2.10
- [fixed](#valdef-background-attachment-fixed), in § 2.5
- [groove](#valdef-line-style-groove), in § 3.2
- [hidden](#valdef-line-style-hidden), in § 3.2
- [horizontal offset](#box-shadow-horizontal-offset), in § 6.1
- [image](#background-images), in § 2.3
- [inner box-shadow](#box-shadow-inner-box-shadow), in § 6.1
- [inner shadow](#box-shadow-inner-box-shadow), in § 6.1
- inset
  - [value for \<line-style\>, border-style, border-top-style, border-left-style, border-bottom-style, border-right-style, border](#valdef-line-style-inset), in § 3.2
  - [value for box-shadow](#shadow-inset), in § 6.1
- [layer](#background-image-layer), in § 2.1
- [left](#valdef-background-position-left), in § 2.6
- [\<length\>](#valdef-background-position-length), in § 2.6
- \<length-percentage \[0,∞\]\>
  - [value for background-size](#valdef-background-size-length-percentage-0), in § 2.9
  - [value for border-image-width](#valdef-border-image-width-length-percentage-0), in § 5.3
- [\<line-style\>](#typedef-line-style), in § 3.2
- [\<line-width\>](#typedef-line-width), in § 3.3
- [local](#valdef-background-attachment-local), in § 2.5
- [medium](#valdef-line-width-medium), in § 3.3
- none
  - [value for \<line-style\>, border-style, border-top-style, border-left-style, border-bottom-style, border-right-style, border](#valdef-line-style-none), in § 3.2
  - [value for background-image](#valdef-background-image-none), in § 2.3
  - [value for box-shadow](#box-shadow-none), in § 6.1
- [no-repeat](#valdef-background-repeat-no-repeat), in § 2.4
- \<number \[0,∞\]\>
  - [value for border-image-slice](#valdef-border-image-slice-number-0), in § 5.2
  - [value for border-image-width](#valdef-border-image-width-number-0), in § 5.3
- [outer box-shadow](#box-shadow-outer-box-shadow), in § 6.1
- [outer shadow](#box-shadow-outer-box-shadow), in § 6.1
- [outset](#valdef-line-style-outset), in § 3.2
- padding-box
  - [value for background-clip](#valdef-background-clip-padding-box), in § 2.7
  - [value for background-origin](#valdef-background-origin-padding-box), in § 2.8
- [\<percentage\>](#valdef-background-position-percentage), in § 2.6
- [\<percentage \[0,∞\]\>](#valdef-border-image-slice-percentage-0), in § 5.2
- [region](#border-image-region), in § 5.3
- repeat
  - [value for background-repeat](#valdef-background-repeat-repeat), in § 2.4
  - [value for border-image-repeat](#valdef-border-image-repeat-repeat), in § 5.5
- [\<repeat-style\>](#typedef-repeat-style), in § 2.4
- [repeat-x](#valdef-background-repeat-repeat-x), in § 2.4
- [repeat-y](#valdef-background-repeat-repeat-y), in § 2.4
- [ridge](#valdef-line-style-ridge), in § 3.2
- [right](#valdef-background-position-right), in § 2.6
- round
  - [value for background-repeat](#valdef-background-repeat-round), in § 2.4
  - [value for border-image-repeat](#valdef-border-image-repeat-round), in § 5.5
- [scroll](#valdef-background-attachment-scroll), in § 2.5
- [\<shadow\>](#typedef-shadow), in § 6.1
- [solid](#valdef-line-style-solid), in § 3.2
- space
  - [value for background-repeat](#valdef-background-repeat-space), in § 2.4
  - [value for border-image-repeat](#valdef-border-image-repeat-space), in § 5.5
- [spread distance](#box-shadow-spread-distance), in § 6.1
- [stretch](#valdef-border-image-repeat-stretch), in § 5.5
- [thick](#valdef-line-width-thick), in § 3.3
- [thin](#valdef-line-width-thin), in § 3.3
- [top](#valdef-background-position-top), in § 2.6
- [vertical offset](#box-shadow-vertical-offset), in § 6.1

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-2023\] defines the following terms:
  - <a id="861626b1"></a>ua
- \[CSS-BOX-4\] defines the following terms:
  - <a id="c87746d2"></a>\<visual-box\>
  - <a id="30e036e4"></a>border
  - <a id="69de5ee0"></a>border area
  - <a id="85c399c0"></a>border box
  - <a id="3e6781f5"></a>border edge
  - <a id="df86efcb"></a>content area
  - <a id="f72f5cb4"></a>content box
  - <a id="cba8daea"></a>content edge
  - <a id="253362bb"></a>margin
  - <a id="0f70e5fd"></a>margin area
  - <a id="16ff1cf8"></a>margin edge
  - <a id="a2be8c84"></a>padding
  - <a id="6694799c"></a>padding area
  - <a id="15e1e804"></a>padding box
  - <a id="093a0ff1"></a>padding edge
- \[CSS-BREAK-4\] defines the following terms:
  - <a id="d65c0e81"></a>box fragment
  - <a id="a0542bba"></a>box-decoration-break
- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="6b448e93"></a>initial value
  - <a id="980ac56a"></a>shorthand property
  - <a id="d5e08d9c"></a>specified value
  - <a id="1a2b1083"></a>used value
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="bcdf9b19"></a>color
  - <a id="a42c65ac"></a>currentcolor
  - <a id="96e27c16"></a>transparent
- \[CSS-COLOR-5\] defines the following terms:
  - <a id="d04b6986"></a>\<color\>
- \[CSS-CONTAIN-2\] defines the following terms:
  - <a id="4f31b139"></a>containment
- \[CSS-CONTENT-3\] defines the following terms:
  - <a id="f3e8378c"></a>content
- \[CSS-DISPLAY-3\] defines the following terms:
  - <a id="e26aa9bf"></a>initial containing block
  - <a id="93f98063"></a>principal box
  - <a id="299e10e4"></a>replaced element
  - <a id="8b4f8a45"></a>root element
- \[CSS-IMAGES-3\] defines the following terms:
  - <a id="35bf32f2"></a>\<image\>
  - <a id="c82a1380"></a>default object size
  - <a id="ffedca23"></a>natural aspect ratio
  - <a id="487e1aa9"></a>natural dimension
  - <a id="b9cef6bf"></a>natural height
  - <a id="c0cc78c8"></a>natural size
  - <a id="24ae9eec"></a>natural width
  - <a id="e99a4517"></a>specified size
- \[CSS-OVERFLOW-3\] defines the following terms:
  - <a id="a3cabdb1"></a>scroll container
  - <a id="3ed7991e"></a>scrollable overflow area
- \[CSS-RUBY-1\] defines the following terms:
  - <a id="7015f3a0"></a>ruby annotation container
  - <a id="153743a1"></a>ruby base container
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="5ad01cca"></a>height
- \[CSS-TEXT-DECOR-4\] defines the following terms:
  - <a id="a7f17cc4"></a>text-shadow
- \[CSS-VALUES-3\] defines the following terms:
  - <a id="31d72d5d"></a>\<position\>
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="c297b070"></a>\#
  - <a id="bdb4e757"></a>&#x26;&#x26;
  - <a id="8cd4f032"></a>,
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="61bb5e44"></a>\<number\>
  - <a id="128295ac"></a>\<percentage\>
  - <a id="d4441b24"></a>?
  - <a id="99261030"></a>css bracketed range notation
  - <a id="8a110a7b"></a>css-wide keywords
  - <a id="4f460096"></a>snap as a border width
  - <a id="15d1a46a"></a>value definition syntax
  - <a id="3bafef5e"></a>{a,b}
  - <a id="8cbc2b3b"></a>{a}
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS2\] defines the following terms:
  - <a id="29ca9f85"></a>border-collapse
  - <a id="ef9e6926"></a>display
  - <a id="a2942f61"></a>inline-table
  - <a id="244c26d9"></a>overflow
  - <a id="97bfa2a1"></a>table-cell
  - <a id="aa17b477"></a>viewport
- \[CSS22\] defines the following terms:
  - <a id="8b60933e"></a>collapse
- \[FILTER-EFFECTS-1\] defines the following terms:
  - <a id="1a2825fc"></a>drop-shadow()
- \[HTML\] defines the following terms:
  - <a id="2f0492ac"></a>body
- \[MEDIAQUERIES-5\] defines the following terms:
  - <a id="23af89d0"></a>paged media
- \[SELECTORS-3\] defines the following terms:
  - <a id="667c691b"></a>::first-letter
  - <a id="3218f3af"></a>::first-line
- \[SELECTORS-4\] defines the following terms:
  - <a id="4d06fa38"></a>pseudo-element

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-2023"></a>\[CSS-2023\]  
Chris Lilley; et al. [CSS Snapshot 2023](https://www.w3.org/TR/css-2023/). 7 December 2023. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-2023&#x2F;](https://www.w3.org/TR/css-2023/)

<a id="biblio-css-box-3"></a>\[CSS-BOX-3\]  
Elika Etemad. [CSS Box Model Module Level 3](https://www.w3.org/TR/css-box-3/). 6 April 2023. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-3&#x2F;](https://www.w3.org/TR/css-box-3/)

<a id="biblio-css-box-4"></a>\[CSS-BOX-4\]  
Elika Etemad. [CSS Box Model Module Level 4](https://www.w3.org/TR/css-box-4/). 3 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-box-4&#x2F;](https://www.w3.org/TR/css-box-4/)

<a id="biblio-css-break-4"></a>\[CSS-BREAK-4\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 4](https://www.w3.org/TR/css-break-4/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-4&#x2F;](https://www.w3.org/TR/css-break-4/)

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 1 November 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-color-5"></a>\[CSS-COLOR-5\]  
Chris Lilley; et al. [CSS Color Module Level 5](https://www.w3.org/TR/css-color-5/). 28 June 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-5&#x2F;](https://www.w3.org/TR/css-color-5/)

<a id="biblio-css-display-3"></a>\[CSS-DISPLAY-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Display Module Level 3](https://www.w3.org/TR/css-display-3/). 30 March 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-display-3&#x2F;](https://www.w3.org/TR/css-display-3/)

<a id="biblio-css-images-3"></a>\[CSS-IMAGES-3\]  
Tab Atkins Jr.; Elika Etemad; Lea Verou. [CSS Images Module Level 3](https://www.w3.org/TR/css-images-3/). 18 December 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-images-3&#x2F;](https://www.w3.org/TR/css-images-3/)

<a id="biblio-css-overflow-3"></a>\[CSS-OVERFLOW-3\]  
Elika Etemad; Florian Rivoal. [CSS Overflow Module Level 3](https://www.w3.org/TR/css-overflow-3/). 29 March 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-overflow-3&#x2F;](https://www.w3.org/TR/css-overflow-3/)

<a id="biblio-css-ruby-1"></a>\[CSS-RUBY-1\]  
Elika Etemad; et al. [CSS Ruby Annotation Layout Module Level 1](https://www.w3.org/TR/css-ruby-1/). 31 December 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ruby-1&#x2F;](https://www.w3.org/TR/css-ruby-1/)

<a id="biblio-css-text-decor-4"></a>\[CSS-TEXT-DECOR-4\]  
Elika Etemad; Koji Ishii. [CSS Text Decoration Module Level 4](https://www.w3.org/TR/css-text-decor-4/). 4 May 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-text-decor-4&#x2F;](https://www.w3.org/TR/css-text-decor-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 1 December 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 18 December 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css22"></a>\[CSS22\]  
Bert Bos. [Cascading Style Sheets Level 2 Revision 2 (CSS 2.2) Specification](https://www.w3.org/TR/CSS22/). 12 April 2016. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS22&#x2F;](https://www.w3.org/TR/CSS22/)

<a id="biblio-filter-effects-1"></a>\[FILTER-EFFECTS-1\]  
Dirk Schulze; Dean Jackson. [Filter Effects Module Level 1](https://www.w3.org/TR/filter-effects-1/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;filter-effects-1&#x2F;](https://www.w3.org/TR/filter-effects-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-mediaqueries-5"></a>\[MEDIAQUERIES-5\]  
Dean Jackson; et al. [Media Queries Level 5](https://www.w3.org/TR/mediaqueries-5/). 18 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;mediaqueries-5&#x2F;](https://www.w3.org/TR/mediaqueries-5/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

<a id="biblio-selectors-3"></a>\[SELECTORS-3\]  
Tantek Çelik; et al. [Selectors Level 3](https://www.w3.org/TR/selectors-3/). 6 November 2018. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-3&#x2F;](https://www.w3.org/TR/selectors-3/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

### <a id="informative"></a>Informative References

<a id="biblio-css-2017"></a>\[CSS-2017\]  
Tab Atkins Jr.; Elika Etemad; Florian Rivoal. [CSS Snapshot 2017](https://www.w3.org/TR/css-2017/). 31 January 2017. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-2017&#x2F;](https://www.w3.org/TR/css-2017/)

<a id="biblio-css-break-3"></a>\[CSS-BREAK-3\]  
Rossen Atanassov; Elika Etemad. [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/). 4 December 2018. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-break-3&#x2F;](https://www.w3.org/TR/css-break-3/)

<a id="biblio-css-contain-1"></a>\[CSS-CONTAIN-1\]  
Tab Atkins Jr.; Florian Rivoal. [CSS Containment Module Level 1](https://www.w3.org/TR/css-contain-1/). 25 October 2022. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-1&#x2F;](https://www.w3.org/TR/css-contain-1/)

<a id="biblio-css-contain-2"></a>\[CSS-CONTAIN-2\]  
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin. [CSS Containment Module Level 2](https://www.w3.org/TR/css-contain-2/). 17 September 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-contain-2&#x2F;](https://www.w3.org/TR/css-contain-2/)

<a id="biblio-css-content-3"></a>\[CSS-CONTENT-3\]  
Elika Etemad; Dave Cramer. [CSS Generated Content Module Level 3](https://www.w3.org/TR/css-content-3/). 2 August 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-content-3&#x2F;](https://www.w3.org/TR/css-content-3/)

<a id="biblio-css-shapes"></a>\[CSS-SHAPES\]  
Rossen Atanassov; Alan Stearns. [CSS Shapes Module Level 1](https://www.w3.org/TR/css-shapes-1/). 15 November 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-shapes-1&#x2F;](https://www.w3.org/TR/css-shapes-1/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-wcag20"></a>\[WCAG20\]  
Ben Caldwell; et al. [Web Content Accessibility Guidelines (WCAG) 2.0](https://www.w3.org/TR/WCAG20/). 11 December 2008. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;WCAG20&#x2F;](https://www.w3.org/TR/WCAG20/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                                                                                                                                               | Initial                     | Applies to                                                                    | Inh. | %ages                                                                                 | Anim­ation type                                                                                                                                                                                                                               | Canonical order | Com­puted value                                                                                                                             | Logical property group |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------|-------------------------------------------------------------------------------|------|---------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------|--------------------------------------------------------------------------------------------------------------------------------------------|------------------------|
| <strong><span><a id="ref-for-propdef-background⑨"></a></span><a href="#propdef-background">background</a>&#xA;      </strong> | \<bg-layer\>#? , \<final-bg-layer\>                                                                                                                                                 | see individual properties   | all elements                                                                  | no   | see individual properties                                                             | see individual properties                                                                                                                                                                                                                    | per grammar     | see individual properties                                                                                                                  |                        |
| <strong><span><a id="ref-for-propdef-background-attachment⑦"></a></span><a href="#propdef-background-attachment">background-attachment</a>&#xA;      </strong> | \<attachment\>#                                                                                                                                                                     | scroll                      | all elements                                                                  | no   | N/A                                                                                   | discrete                                                                                                                                                                                                                                     | per grammar     | list, each item the keyword as specified                                                                                                   |                        |
| <strong><span><a id="ref-for-propdef-background-clip①③"></a></span><a href="#propdef-background-clip">background-clip</a>&#xA;      </strong> | \<visual-box\>#                                                                                                                                                                     | border-box                  | all elements                                                                  | no   | N/A                                                                                   | repeatable list                                                                                                                                                                                                                              | per grammar     | list, each item a keyword as specified                                                                                                     |                        |
| <strong><span><a id="ref-for-propdef-background-color①①"></a></span><a href="#propdef-background-color">background-color</a>&#xA;      </strong> | \<color\>                                                                                                                                                                           | transparent                 | all elements                                                                  | no   | N/A                                                                                   | by computed value                                                                                                                                                                                                                            | per grammar     | computed color                                                                                                                             |                        |
| <strong><span><a id="ref-for-propdef-background-image①①"></a></span><a href="#propdef-background-image">background-image</a>&#xA;      </strong> | \<bg-image\>#                                                                                                                                                                       | none                        | all elements                                                                  | no   | N/A                                                                                   | discrete                                                                                                                                                                                                                                     | per grammar     | list, each item either an \<image\> or the keyword none                                                                                    |                        |
| <strong><span><a id="ref-for-propdef-background-origin⑨"></a></span><a href="#propdef-background-origin">background-origin</a>&#xA;      </strong> | \<visual-box\>#                                                                                                                                                                     | padding-box                 | all elements                                                                  | no   | N/A                                                                                   | repeatable list                                                                                                                                                                                                                              | per grammar     | list, each item a keyword as specified                                                                                                     |                        |
| <strong><span><a id="ref-for-propdef-background-position①⑦"></a></span><a href="#propdef-background-position">background-position</a>&#xA;      </strong> | \<bg-position\>#                                                                                                                                                                    | 0% 0%                       | all elements                                                                  | no   | refer to size of background positioning area minus size of background image; see text | repeatable list                                                                                                                                                                                                                              | per grammar     | list, each item a pair of offsets (horizontal and vertical) from the top left origin each given as a computed \<length-percentage\> value  |                        |
| <strong><span><a id="ref-for-propdef-background-repeat①⓪"></a></span><a href="#propdef-background-repeat">background-repeat</a>&#xA;      </strong> | \<repeat-style\>#                                                                                                                                                                   | repeat                      | all elements                                                                  | no   | N/A                                                                                   | discrete                                                                                                                                                                                                                                     | per grammar     | list, each item a pair of keywords, one per dimension                                                                                      |                        |
| <strong><span><a id="ref-for-propdef-background-size⑦"></a></span><a href="#propdef-background-size">background-size</a>&#xA;      </strong> | \<bg-size\>#                                                                                                                                                                        | auto                        | all elements                                                                  | no   | see text                                                                              | repeatable list                                                                                                                                                                                                                              | per grammar     | list, each item a pair of sizes (one per axis) each represented as either a keyword or a computed \<length-percentage\> value              |                        |
| <strong><span><a id="ref-for-propdef-border①①"></a></span><a href="#propdef-border">border</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | See individual properties   | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | see individual properties                                                                                                                                                                                                                    | per grammar     | see individual properties                                                                                                                  |                        |
| <strong><span><a id="ref-for-propdef-border-bottom②"></a></span><a href="#propdef-border-bottom">border-bottom</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | See individual properties   | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | see individual properties                                                                                                                                                                                                                    | per grammar     | see individual properties                                                                                                                  |                        |
| <strong><span><a id="ref-for-propdef-border-bottom-color①"></a></span><a href="#propdef-border-bottom-color">border-bottom-color</a>&#xA;      </strong> | \<color\>                                                                                                                                                                           | currentColor                | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | by computed value                                                                                                                                                                                                                            | per grammar     | computed color                                                                                                                             | border-color           |
| <strong><span><a id="ref-for-propdef-border-bottom-left-radius"></a></span><a href="#propdef-border-bottom-left-radius">border-bottom-left-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,2}                                                                                                                                                  | 0                           | all elements (but see prose)                                                  | no   | Refer to corresponding dimension of the border box.                                   | by computed value                                                                                                                                                                                                                            | per grammar     | pair of computed \<length-percentage\> values                                                                                              | border-radius          |
| <strong><span><a id="ref-for-propdef-border-bottom-right-radius"></a></span><a href="#propdef-border-bottom-right-radius">border-bottom-right-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,2}                                                                                                                                                  | 0                           | all elements (but see prose)                                                  | no   | Refer to corresponding dimension of the border box.                                   | by computed value                                                                                                                                                                                                                            | per grammar     | pair of computed \<length-percentage\> values                                                                                              | border-radius          |
| <strong><span><a id="ref-for-propdef-border-bottom-style①"></a></span><a href="#propdef-border-bottom-style">border-bottom-style</a>&#xA;      </strong> | \<line-style\>                                                                                                                                                                      | none                        | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | discrete                                                                                                                                                                                                                                     | per grammar     | specified keyword                                                                                                                          | border-style           |
| <strong><span><a id="ref-for-propdef-border-bottom-width①"></a></span><a href="#propdef-border-bottom-width">border-bottom-width</a>&#xA;      </strong> | \<line-width\>                                                                                                                                                                      | medium                      | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | by computed value                                                                                                                                                                                                                            | per grammar     | absolute length, snapped as a border width; zero if the border style is none or hidden                                                     | border-width           |
| <strong><span><a id="ref-for-propdef-border-color⑨"></a></span><a href="#propdef-border-color">border-color</a>&#xA;      </strong> | \<color\>{1,4}                                                                                                                                                                      | (see individual properties) | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | see individual properties                                                                                                                                                                                                                    | per grammar     | see individual properties                                                                                                                  |                        |
| <strong><span><a id="ref-for-propdef-border-image①⓪"></a></span><a href="#propdef-border-image">border-image</a>&#xA;      </strong> | \<'border-image-source'\> \|\| \<'border-image-slice'\> \[ / \<'border-image-width'\> \| / \<'border-image-width'\>? / \<'border-image-outset'\> \]? \|\| \<'border-image-repeat'\> | See individual properties   | See individual properties                                                     | no   | N/A                                                                                   | See individual properties                                                                                                                                                                                                                    | per grammar     | See individual properties                                                                                                                  |                        |
| <strong><span><a id="ref-for-propdef-border-image-outset④"></a></span><a href="#propdef-border-image-outset">border-image-outset</a>&#xA;      </strong> | \[ \<length \[0,∞\]\> \| \<number \[0,∞\]\> \]{1,4}                                                                                                                                 | 0                           | All elements, except internal table elements when border-collapse is collapse | no   | N/A                                                                                   | by computed value                                                                                                                                                                                                                            | per grammar     | four values, each a number or absolute length                                                                                              |                        |
| <strong><span><a id="ref-for-propdef-border-image-repeat⑤"></a></span><a href="#propdef-border-image-repeat">border-image-repeat</a>&#xA;      </strong> | \[ stretch \| repeat \| round \| space \]{1,2}                                                                                                                                      | stretch                     | All elements, except internal table elements when border-collapse is collapse | no   | N/A                                                                                   | discrete                                                                                                                                                                                                                                     | per grammar     | two keywords, one per axis                                                                                                                 |                        |
| <strong><span><a id="ref-for-propdef-border-image-slice⑧"></a></span><a href="#propdef-border-image-slice">border-image-slice</a>&#xA;      </strong> | \[\<number \[0,∞\]\> \| \<percentage \[0,∞\]\>\]{1,4} &#x26;&#x26; fill?                                                                                          | 100%                        | All elements, except internal table elements when border-collapse is collapse | no   | refer to size of the border image                                                     | by computed value                                                                                                                                                                                                                            | per grammar     | four values, each either a number or percentage; plus a fill keyword if specified                                                          |                        |
| <strong><span><a id="ref-for-propdef-border-image-source⑦"></a></span><a href="#propdef-border-image-source">border-image-source</a>&#xA;      </strong> | none \| \<image\>                                                                                                                                                                   | none                        | All elements, except internal table elements when border-collapse is collapse | no   | N/A                                                                                   | discrete                                                                                                                                                                                                                                     | per grammar     | the keyword none or the computed \<image\>                                                                                                 |                        |
| <strong><span><a id="ref-for-propdef-border-image-width①⑤"></a></span><a href="#propdef-border-image-width">border-image-width</a>&#xA;      </strong> | \[ \<length-percentage \[0,∞\]\> \| \<number \[0,∞\]\> \| auto \]{1,4}                                                                                                              | 1                           | All elements, except internal table elements when border-collapse is collapse | no   | Relative to width/height of the border image area                                     | by computed value                                                                                                                                                                                                                            | per grammar     | four values, each either a number, the keyword auto, or a computed \<length-percentage\> value                                             |                        |
| <strong><span><a id="ref-for-propdef-border-left⑤"></a></span><a href="#propdef-border-left">border-left</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | See individual properties   | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | see individual properties                                                                                                                                                                                                                    | per grammar     | see individual properties                                                                                                                  |                        |
| <strong><span><a id="ref-for-propdef-border-left-color①"></a></span><a href="#propdef-border-left-color">border-left-color</a>&#xA;      </strong> | \<color\>                                                                                                                                                                           | currentColor                | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | by computed value                                                                                                                                                                                                                            | per grammar     | computed color                                                                                                                             | border-color           |
| <strong><span><a id="ref-for-propdef-border-left-style①"></a></span><a href="#propdef-border-left-style">border-left-style</a>&#xA;      </strong> | \<line-style\>                                                                                                                                                                      | none                        | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | discrete                                                                                                                                                                                                                                     | per grammar     | specified keyword                                                                                                                          | border-style           |
| <strong><span><a id="ref-for-propdef-border-left-width①"></a></span><a href="#propdef-border-left-width">border-left-width</a>&#xA;      </strong> | \<line-width\>                                                                                                                                                                      | medium                      | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | by computed value                                                                                                                                                                                                                            | per grammar     | absolute length, snapped as a border width; zero if the border style is none or hidden                                                     | border-width           |
| <strong><span><a id="ref-for-propdef-border-radius①⑥"></a></span><a href="#propdef-border-radius">border-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,4} \[ / \<length-percentage \[0,∞\]\>{1,4} \]?                                                                                                      | see individual properties   | all elements (but see prose)                                                  | no   | Refer to corresponding dimension of the border box.                                   | see individual properties                                                                                                                                                                                                                    | per grammar     | see individual properties                                                                                                                  |                        |
| <strong><span><a id="ref-for-propdef-border-right②"></a></span><a href="#propdef-border-right">border-right</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | See individual properties   | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | see individual properties                                                                                                                                                                                                                    | per grammar     | see individual properties                                                                                                                  |                        |
| <strong><span><a id="ref-for-propdef-border-right-color①"></a></span><a href="#propdef-border-right-color">border-right-color</a>&#xA;      </strong> | \<color\>                                                                                                                                                                           | currentColor                | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | by computed value                                                                                                                                                                                                                            | per grammar     | computed color                                                                                                                             | border-color           |
| <strong><span><a id="ref-for-propdef-border-right-style①"></a></span><a href="#propdef-border-right-style">border-right-style</a>&#xA;      </strong> | \<line-style\>                                                                                                                                                                      | none                        | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | discrete                                                                                                                                                                                                                                     | per grammar     | specified keyword                                                                                                                          | border-style           |
| <strong><span><a id="ref-for-propdef-border-right-width①"></a></span><a href="#propdef-border-right-width">border-right-width</a>&#xA;      </strong> | \<line-width\>                                                                                                                                                                      | medium                      | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | by computed value                                                                                                                                                                                                                            | per grammar     | absolute length, snapped as a border width; zero if the border style is none or hidden                                                     | border-width           |
| <strong><span><a id="ref-for-propdef-border-style①③"></a></span><a href="#propdef-border-style">border-style</a>&#xA;      </strong> | \<line-style\>{1,4}                                                                                                                                                                 | (see individual properties) | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | see individual properties                                                                                                                                                                                                                    | per grammar     | see individual properties                                                                                                                  |                        |
| <strong><span><a id="ref-for-propdef-border-top②"></a></span><a href="#propdef-border-top">border-top</a>&#xA;      </strong> | \<line-width\> \|\| \<line-style\> \|\| \<color\>                                                                                                                                   | See individual properties   | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | see individual properties                                                                                                                                                                                                                    | per grammar     | see individual properties                                                                                                                  |                        |
| <strong><span><a id="ref-for-propdef-border-top-color①"></a></span><a href="#propdef-border-top-color">border-top-color</a>&#xA;      </strong> | \<color\>                                                                                                                                                                           | currentColor                | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | by computed value                                                                                                                                                                                                                            | per grammar     | computed color                                                                                                                             | border-color           |
| <strong><span><a id="ref-for-propdef-border-top-left-radius①"></a></span><a href="#propdef-border-top-left-radius">border-top-left-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,2}                                                                                                                                                  | 0                           | all elements (but see prose)                                                  | no   | Refer to corresponding dimension of the border box.                                   | by computed value                                                                                                                                                                                                                            | per grammar     | pair of computed \<length-percentage\> values                                                                                              | border-radius          |
| <strong><span><a id="ref-for-propdef-border-top-right-radius"></a></span><a href="#propdef-border-top-right-radius">border-top-right-radius</a>&#xA;      </strong> | \<length-percentage \[0,∞\]\>{1,2}                                                                                                                                                  | 0                           | all elements (but see prose)                                                  | no   | Refer to corresponding dimension of the border box.                                   | by computed value                                                                                                                                                                                                                            | per grammar     | pair of computed \<length-percentage\> values                                                                                              | border-radius          |
| <strong><span><a id="ref-for-propdef-border-top-style①"></a></span><a href="#propdef-border-top-style">border-top-style</a>&#xA;      </strong> | \<line-style\>                                                                                                                                                                      | none                        | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | discrete                                                                                                                                                                                                                                     | per grammar     | specified keyword                                                                                                                          | border-style           |
| <strong><span><a id="ref-for-propdef-border-top-width①"></a></span><a href="#propdef-border-top-width">border-top-width</a>&#xA;      </strong> | \<line-width\>                                                                                                                                                                      | medium                      | all elements except ruby base containers and ruby annotation containers       | no   | N/A                                                                                   | by computed value                                                                                                                                                                                                                            | per grammar     | absolute length, snapped as a border width; zero if the border style is none or hidden                                                     | border-width           |
| <strong><span><a id="ref-for-propdef-border-width①⑥"></a></span><a href="#propdef-border-width">border-width</a>&#xA;      </strong> | \<line-width\>{1,4}                                                                                                                                                                 | (see individual properties) | all elements except ruby base containers and ruby annotation containers       | no   | see individual properties                                                             | see individual properties                                                                                                                                                                                                                    | per grammar     | see individual properties                                                                                                                  |                        |
| <strong><span><a id="ref-for-propdef-box-shadow①⓪"></a></span><a href="#propdef-box-shadow">box-shadow</a>&#xA;      </strong> | none \| \<shadow\>#                                                                                                                                                                 | none                        | all elements                                                                  | no   | N/A                                                                                   | by computed value, treating none as a zero-item list and appending blank shadows (transparent 0 0 0 0) with a corresponding inset keyword as needed to match the longer list if the shorter list is otherwise compatible with the longer one | per grammar     | either the keyword none or a list, each item consisting of four absolute lengths plus a computed color and optionally also a inset keyword |                        |

