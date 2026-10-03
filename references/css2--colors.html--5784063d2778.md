Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Colors and backgrounds](https://www.w3.org/TR/2011/REC-CSS2-20110607/colors.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Colors and backgrounds

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/colors.html

Snapshot SHA-256: 5784063d27780195eedcd9824d347c923803de276fb8b766825e50cbd044c490

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

<a id="q14.0"></a>

# 14 Colors and Backgrounds

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

CSS properties allow authors to specify the foreground color and background of an element. Backgrounds may be colors or images. Background properties allow authors to position a background image, repeat it, and declare whether it should be fixed with respect to the [viewport](css2--visuren.html--3f334c530cf4.md#viewport) or scrolled along with the document.

See the section on [color units](css2--syndata.html--02e71c159e14.md#color-units) for the syntax of valid color values.

<a id="colors"></a>

## 14.1 Foreground color: the ['color'](css2--colors.html--5784063d2778.md#propdef-color) property

<a id="propdef-color"></a>

<strong>'color'</strong>

|                       |                                                                                                                                                                                  |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<color\>](css2--syndata.html--02e71c159e14.md#value-def-color) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | depends on user agent                                                                                                                                                            |
| <em>Applies to:</em>   | all elements                                                                                                                                                                     |
| <em>Inherited:</em>   | yes                                                                                                                                                                              |
| <em>Percentages:</em>   | N/A                                                                                                                                                                              |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                             |
| <em>Computed value:</em>   | as specified                                                                                                                                                                     |

This property describes the foreground color of an element's text content. There are different ways to specify red:

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> em { color: red }              /* predefined color name */
> em { color: rgb(255,0,0) }     /* RGB range 0-255   */
> ```
<a id="background"></a>

## 14.2 The background

Authors may specify the background of an element (i.e., its rendering surface) as either a color or an image. In terms of the [box model](css2--box.html--8875bbcdefe6.md#box-model), "background" refers to the background of the [content](css2--box.html--8875bbcdefe6.md#box-content-area), [padding](css2--box.html--8875bbcdefe6.md#box-padding-area) and [border](css2--box.html--8875bbcdefe6.md#box-border-area) areas. Border colors and styles are set with the [border properties](css2--box.html--8875bbcdefe6.md#border-properties). Margins are always transparent.

Background properties are not inherited, but the parent box's background will shine through by default because of the initial 'transparent' value on ['background-color'](css2--colors.html--5784063d2778.md#propdef-background-color).

The background of the root element becomes the background of the canvas and covers the entire [canvas](css2--intro.html--d40812139784.md#canvas), anchored (for ['background-position'](css2--colors.html--5784063d2778.md#propdef-background-position)) at the same point as it would be if it was painted only for the root element itself. The root element does not paint this background again.

For HTML documents, however, we recommend that authors specify the background for the BODY element rather than the HTML element. For documents whose root element is an HTML "HTML" element or an XHTML "html" element that has computed values of 'transparent' for ['background-color'](css2--colors.html--5784063d2778.md#propdef-background-color) and 'none' for ['background-image'](css2--colors.html--5784063d2778.md#propdef-background-image), user agents must instead use the computed value of the background properties from that element's first HTML "BODY" element or XHTML "body" element child when painting backgrounds for the canvas, and must not paint a background for that child element. Such backgrounds must also be anchored at the same point as they would be if they were painted only for the root element.

According to these rules, the canvas underlying the following HTML document will have a "marble" background:

```text

<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN">
    <TITLE>Setting the canvas background</TITLE>
    <STYLE type="text/css">
       BODY { background: url("http://example.com/marble.png") }
    </STYLE>
    <P>My background is marble.
```
Note that the rule for the BODY element will work even though the BODY tag has been omitted in the HTML source since the HTML parser will infer the missing tag.

Backgrounds of elements that form a stacking context (see the ['z-index'](css2--visuren.html--3f334c530cf4.md#propdef-z-index) property) are painted at the bottom of the element's stacking context, below anything in that stacking context.

<a id="background-properties"></a>

### 14.2.1 Background properties: ['background-color'](css2--colors.html--5784063d2778.md#propdef-background-color), ['background-image'](css2--colors.html--5784063d2778.md#propdef-background-image), ['background-repeat'](css2--colors.html--5784063d2778.md#propdef-background-repeat), ['background-attachment'](css2--colors.html--5784063d2778.md#propdef-background-attachment), ['background-position'](css2--colors.html--5784063d2778.md#propdef-background-position), and ['background'](css2--colors.html--5784063d2778.md#propdef-background)

<a id="propdef-background-color"></a>

<strong>'background-color'</strong>

|                       |                                                                                                                                                                                                 |
|-----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<color\>](css2--syndata.html--02e71c159e14.md#value-def-color) \| transparent \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | transparent                                                                                                                                                                                     |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                    |
| <em>Inherited:</em>   | no                                                                                                                                                                                              |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                             |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                            |
| <em>Computed value:</em>   | as specified                                                                                                                                                                                    |

<a id="x2"></a>

This property sets the background color of an element, either a [\<color\>](css2--syndata.html--02e71c159e14.md#value-def-color) value or the keyword 'transparent', to make the underlying colors shine through.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> h1 { background-color: #F00 }
> ```
<a id="propdef-background-image"></a>

<strong>'background-image'</strong>

|                       |                                                                                                                                                                                      |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<uri\>](css2--syndata.html--02e71c159e14.md#value-def-uri) \| none \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | none                                                                                                                                                                                 |
| <em>Applies to:</em>   | all elements                                                                                                                                                                         |
| <em>Inherited:</em>   | no                                                                                                                                                                                   |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                  |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                 |
| <em>Computed value:</em>   | absolute URI or none                                                                                                                                                                 |

This property sets the background image of an element. When setting a background image, authors should also specify a background color that will be used when the image is unavailable. When the image is available, it is rendered on top of the background color. (Thus, the color is visible in the transparent parts of the image).

<a id="x4"></a>

Values for this property are either [\<uri\>](css2--syndata.html--02e71c159e14.md#value-def-uri), to specify the image, or 'none', when no image is used.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> body { background-image: url("marble.png") }
> p { background-image: none }
> ```
Intrinsic dimensions expressed as percentages must be resolved relative to the dimensions of the rectangle that establishes the coordinate system for the ['background-position'](css2--colors.html--5784063d2778.md#propdef-background-position) property.

If the image has one of either an intrinsic width or an intrinsic height and an intrinsic aspect ratio, then the missing dimension is calculated from the given dimension and the ratio.

If the image has one of either an intrinsic width or an intrinsic height and no intrinsic aspect ratio, then the missing dimension is assumed to be the size of the rectangle that establishes the coordinate system for the 'background-position' property.

If the image has no intrinsic dimensions and has an intrinsic ratio the dimensions must be assumed to be the largest dimensions at that ratio such that neither dimension exceeds the dimensions of the rectangle that establishes the coordinate system for the ['background-position'](css2--colors.html--5784063d2778.md#propdef-background-position) property.

If the image has no intrinsic ratio either, then the dimensions must be assumed to be the rectangle that establishes the coordinate system for the ['background-position'](css2--colors.html--5784063d2778.md#propdef-background-position) property.

<a id="propdef-background-repeat"></a>

<strong>'background-repeat'</strong>

|                       |                                                                                                                                       |
|-----------------------|---------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | repeat \| repeat-x \| repeat-y \| no-repeat \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | repeat                                                                                                                                |
| <em>Applies to:</em>   | all elements                                                                                                                          |
| <em>Inherited:</em>   | no                                                                                                                                    |
| <em>Percentages:</em>   | N/A                                                                                                                                   |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                  |
| <em>Computed value:</em>   | as specified                                                                                                                          |

If a background image is specified, this property specifies whether the image is repeated (tiled), and how. All tiling covers the [content](css2--box.html--8875bbcdefe6.md#box-content-area), [padding](css2--box.html--8875bbcdefe6.md#box-padding-area) and [border](css2--box.html--8875bbcdefe6.md#box-border-area) areas of a box.

The tiling and positioning of the background-image on inline elements is undefined in this specification. A future level of CSS may define the tiling and positioning of the background-image on inline elements.

Values have the following meanings:

<strong>repeat</strong>  
The image is repeated both horizontally and vertically.

<strong>repeat-x</strong>  
The image is repeated horizontally only.

<strong>repeat-y</strong>  
The image is repeated vertically only.

<strong>no-repeat</strong>  
The image is not repeated: only one copy of the image is drawn.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> body { 
>   background: white url("pendant.png");
>   background-repeat: repeat-y;
>   background-position: center;
> }
> ```
>
> <a id="img-bg-repeat"></a>
>
> ![A centered background image, with copies repeated up and down the padding and content areas.](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/bg-repeat.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/bg-repeat-desc.html)
>
> One copy of the background image is centered, and other copies are put above and below it to make a vertical band behind the element.

<a id="propdef-background-attachment"></a>

<strong>'background-attachment'</strong>

|                       |                                                                                                           |
|-----------------------|-----------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | scroll \| fixed \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | scroll                                                                                                    |
| <em>Applies to:</em>   | all elements                                                                                              |
| <em>Inherited:</em>   | no                                                                                                        |
| <em>Percentages:</em>   | N/A                                                                                                       |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                      |
| <em>Computed value:</em>   | as specified                                                                                              |

If a background image is specified, this property specifies whether it is fixed with regard to the [viewport](css2--visuren.html--3f334c530cf4.md#viewport) ('fixed') or scrolls along with the containing block ('scroll').

Note that there is only one viewport per view. If an element has a scrolling mechanism (see 'overflow'), a 'fixed' background does not move with the element, and a 'scroll' background does not move with the scrolling mechanism.

Even if the image is fixed, it is still only visible when it is in the content, padding or border area of the element. Thus, unless the image is tiled ('background-repeat: repeat'), it may be invisible.

In paged media, where there is no viewport, a 'fixed' background is fixed with respect to the page box and is therefore replicated on every page.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> This example creates an infinite vertical band that remains "glued" to the viewport when the element is scrolled.
>
> ```text
> 
> body { 
>   background: red url("pendant.png");
>   background-repeat: repeat-y;
>   background-attachment: fixed;
> }
> ```
User agents that do not support 'fixed' backgrounds (for example due to limitations of the hardware platform) should ignore declarations with the keyword 'fixed'. For example:

```text

body {
  background: white url(paper.png) scroll; /* for all UAs */
  background: white url(ledger.png) fixed; /* for UAs that do fixed backgrounds */
}
```
See the section on [conformance](css2--conform.html--de58593b67d7.md#conformance) for details.

<a id="propdef-background-position"></a>

<strong>'background-position'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | \[ \[ [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| left \| center \| right \] \[ [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| top \| center \| bottom \]? \] \| \[ \[ left \| center \| right \] \|\| \[ top \| center \| bottom \] \] \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 0% 0%                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| <em>Percentages:</em>   | refer to the size of the box itself                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <em>Computed value:</em>   | for [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) the absolute value, otherwise a percentage                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |

If a background image has been specified, this property specifies its initial position. If only one value is specified, the second value is assumed to be 'center'. If at least one value is not a keyword, then the first value represents the horizontal position and the second represents the vertical position. Negative \<percentage\> and \<length\> values are allowed.

<a id="x8"></a>

[<strong>&lt;percentage&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-percentage)

A percentage X aligns the point X% across (for horizontal) or down (for vertical) the image with the point X% across (for horizontal) or down (for vertical) the element's padding box. For example, with a value pair of '0% 0%',the upper left corner of the image is aligned with the upper left corner of the padding box. A value pair of '100% 100%' places the lower right corner of the image in the lower right corner of the padding box. With a value pair of '14% 84%', the point 14% across and 84% down the image is to be placed at the point 14% across and 84% down the padding box.

<a id="x9"></a>

[<strong>&lt;length&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-length)

A length L aligns the top left corner of the image a distance L to the right of (for horizontal) or below (for vertical) the top left corner of the element's padding box. For example, with a value pair of '2cm 1cm', the upper left corner of the image is placed 2cm to the right and 1cm below the upper left corner of the padding box.

<strong>top</strong>

Equivalent to '0%' for the vertical position.

<strong>right</strong>

Equivalent to '100%' for the horizontal position.

<strong>bottom</strong>

Equivalent to '100%' for the vertical position.

<strong>left</strong>

Equivalent to '0%' for the horizontal position.

<strong>center</strong>

Equivalent to '50%' for the horizontal position if it is not otherwise given, or '50%' for the vertical position if it is.

However, the position is undefined in CSS 2.1 if the image has an intrinsic ratio, but no intrinsic size.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> body { background: url("banner.jpeg") right top }    /* 100%   0% */
> body { background: url("banner.jpeg") top center }   /*  50%   0% */
> body { background: url("banner.jpeg") center }       /*  50%  50% */
> body { background: url("banner.jpeg") bottom }       /*  50% 100% */
> ```
The tiling and positioning of the background-image on inline elements is undefined in this specification. A future level of CSS may define the tiling and positioning of the background-image on inline elements.

If the background image is fixed within the viewport (see the ['background-attachment'](css2--colors.html--5784063d2778.md#propdef-background-attachment) property), the image is placed relative to the viewport instead of the element's padding box. For example,

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> body { 
>   background-image: url("logo.png");
>   background-attachment: fixed;
>   background-position: 100% 100%;
>   background-repeat: no-repeat;
> } 
> ```
>
> In the example above, the (single) image is placed in the lower-right corner of the viewport.

<a id="propdef-background"></a>

<strong>'background'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
|-----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | \[[\<'background-color'\>](css2--colors.html--5784063d2778.md#propdef-background-color) \|\| [\<'background-image'\>](css2--colors.html--5784063d2778.md#propdef-background-image) \|\| [\<'background-repeat'\>](css2--colors.html--5784063d2778.md#propdef-background-repeat) \|\| [\<'background-attachment'\>](css2--colors.html--5784063d2778.md#propdef-background-attachment) \|\| [\<'background-position'\>](css2--colors.html--5784063d2778.md#propdef-background-position)\] \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| <em>Percentages:</em>   | allowed on 'background-position'                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| <em>Computed value:</em>   | see individual properties                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |

The ['background'](css2--colors.html--5784063d2778.md#propdef-background) property is a shorthand property for setting the individual background properties (i.e., ['background-color'](css2--colors.html--5784063d2778.md#propdef-background-color), ['background-image'](css2--colors.html--5784063d2778.md#propdef-background-image), ['background-repeat'](css2--colors.html--5784063d2778.md#propdef-background-repeat), ['background-attachment'](css2--colors.html--5784063d2778.md#propdef-background-attachment) and ['background-position'](css2--colors.html--5784063d2778.md#propdef-background-position)) at the same place in the style sheet.

Given a valid declaration, the ['background'](css2--colors.html--5784063d2778.md#propdef-background) property first sets all the individual background properties to their initial values, then assigns explicit values given in the declaration.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In the first rule of the following example, only a value for ['background-color'](css2--colors.html--5784063d2778.md#propdef-background-color) has been given and the other individual properties are set to their initial value. In the second rule, all individual properties have been specified.
>
> ```text
> 
> BODY { background: red }
> P { background: url("chess.png") gray 50% repeat fixed }
> ```