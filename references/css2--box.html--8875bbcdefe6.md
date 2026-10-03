Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Box model](https://www.w3.org/TR/2011/REC-CSS2-20110607/box.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Box model

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/box.html

Snapshot SHA-256: 8875bbcdefe6edc0d14fbd4a8d336b1062d2feb60aa65985bef60e86c530d99f

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

<a id="box-model"></a>

# 8 Box model

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

The CSS box model describes the rectangular boxes that are generated for elements in the [document tree](css2--conform.html--de58593b67d7.md#doctree) and laid out according to the [visual formatting model](css2--visuren.html--3f334c530cf4.md).

<a id="box-dimensions"></a>

## 8.1 Box dimensions

<a id="box-content-area"></a>

<a id="box-padding-area"></a>

<a id="box-border-area"></a>

<a id="box-margin-area"></a>

Each box has a content area (e.g., text, an image, etc.) and optional surrounding padding, border, and margin areas; the size of each area is specified by properties defined below. The following diagram shows how these areas relate and the terminology used to refer to pieces of margin, border, and padding:

<a id="img-boxdim"></a>

![Image illustrating the relationship between content, padding, borders, and margins.](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/boxdim.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/boxdim-desc.html)

The margin, border, and padding can be broken down into top, right, bottom, and left segments (e.g., in the diagram, "LM" for left margin, "RP" for right padding, "TB" for top border, etc.).

The perimeter of each of the four areas (content, padding, border, and margin) is called an "edge", so each box has four edges:

<a id="inner-edge"></a>

<a id="content-edge"></a>

<strong>content edge</strong> or <strong>inner edge</strong>

<a id="x10"></a>

The content edge surrounds the rectangle given by the [width](css2--visudet.html--12e8bc0e6b7c.md#Computing_widths_and_margins) and [height](css2--visudet.html--12e8bc0e6b7c.md#Computing_heights_and_margins) of the box, which often depend on the element's [rendered content](css2--conform.html--de58593b67d7.md#rendered-content). The four content edges define the box's content box.

<a id="padding-edge"></a>

<strong>padding edge</strong>

<a id="x12"></a>

The padding edge surrounds the box padding. If the padding has 0 width, the padding edge is the same as the content edge. The four padding edges define the box's padding box.

<a id="border-edge"></a>

<strong>border edge</strong>

<a id="x14"></a>

The border edge surrounds the box's border. If the border has 0 width, the border edge is the same as the padding edge. The four border edges define the box's border box.

<a id="outer-edge"></a>

<a id="margin-edge"></a>

<strong>margin edge</strong> or <strong>outer
edge</strong>

<a id="x17"></a>

The margin edge surrounds the box margin. If the margin has 0 width, the margin edge is the same as the border edge. The four margin edges define the box's margin box.

Each edge may be broken down into a top, right, bottom, and left edge.

<a id="content-width"></a>

<a id="content-height"></a>

The dimensions of the content area of a box — the content width and content height — depend on several factors: whether the element generating the box has the ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) or ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) property set, whether the box contains text or other boxes, whether the box is a table, etc. Box widths and heights are discussed in the chapter on [visual formatting model details](css2--visudet.html--12e8bc0e6b7c.md).

The background style of the content, padding, and border areas of a box is specified by the ['background'](css2--colors.html--5784063d2778.md#propdef-background) property of the generating element. Margin backgrounds are always transparent.

<a id="mpb-examples"></a>

## 8.2 Example of margins, padding, and borders

This example illustrates how margins, padding, and borders interact. The example HTML document:

```text

<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN">
<HTML>
  <HEAD>
    <TITLE>Examples of margins, padding, and borders</TITLE>
    <STYLE type="text/css">
      UL { 
        background: yellow; 
        margin: 12px 12px 12px 12px;
        padding: 3px 3px 3px 3px;
                                     /* No borders set */
      }
      LI { 
        color: white;                /* text color is white */ 
        background: blue;            /* Content, padding will be blue */
        margin: 12px 12px 12px 12px;
        padding: 12px 0px 12px 12px; /* Note 0px padding right */
        list-style: none             /* no glyphs before a list item */
                                     /* No borders set */
      }
      LI.withborder {
        border-style: dashed;
        border-width: medium;        /* sets border width on all sides */
        border-color: lime;
      }
    </STYLE>
  </HEAD>
  <BODY>
    <UL>
      <LI>First element of list
      <LI class="withborder">Second element of list is
           a bit longer to illustrate wrapping.
    </UL>
  </BODY>
</HTML>
```
results in a [document tree](css2--conform.html--de58593b67d7.md#doctree) with (among other relationships) a UL element that has two LI children.

The first of the following diagrams illustrates what this example would produce. The second illustrates the relationship between the margins, padding, and borders of the UL elements and those of its children LI elements. (Image is not to scale.)

<a id="img-boxdimeg"></a>

![Image illustrating how parent and child margins, borders, and padding relate.](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/boxdimeg.png)   [\[D\]](https://www.w3.org/TR/2011/REC-CSS2-20110607/images/longdesc/boxdimeg-desc.html)

Note that:

- The [content width](#content-width) for each LI box is calculated top-down; the [containing block](css2--visuren.html--3f334c530cf4.md#containing-block) for each LI box is established by the UL element.
- The margin box height of each LI box depends on its [content height](#content-height), plus top and bottom padding, borders, and margins. Note that vertical margins between the LI boxes [collapse.](#collapsing-margins)
- The right padding of the LI boxes has been set to zero width (the ['padding'](css2--box.html--8875bbcdefe6.md#propdef-padding) property). The effect is apparent in the second illustration.
- The margins of the LI boxes are transparent — margins are always transparent — so the background color (yellow) of the UL padding and content areas shines through them.
- The second LI element specifies a dashed border (the ['border-style'](css2--box.html--8875bbcdefe6.md#propdef-border-style) property).

<a id="margin-properties"></a>

## 8.3 Margin properties: ['margin-top'](css2--box.html--8875bbcdefe6.md#propdef-margin-top), ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right), ['margin-bottom'](css2--box.html--8875bbcdefe6.md#propdef-margin-bottom), ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left), and ['margin'](css2--box.html--8875bbcdefe6.md#propdef-margin)

Margin properties specify the width of the [margin area](#box-margin-area) of a box. The ['margin'](css2--box.html--8875bbcdefe6.md#propdef-margin) shorthand property sets the margin for all four sides while the other margin properties only set their respective side. These properties apply to all elements, but vertical margins will not have any effect on non-replaced inline elements.

<a id="value-def-margin-width"></a>

The properties defined in this section refer to the <strong>&lt;margin-width&gt;</strong> value type, which may take one of the following values:

[<strong>&lt;length&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-length)  
Specifies a fixed width.

[<strong>&lt;percentage&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-percentage)  
The percentage is calculated with respect to the <em>width</em> of the generated box's [containing block](css2--visuren.html--3f334c530cf4.md#containing-block). <strong data-conversion-semantic="note">Note:</strong> Note that this is true for ['margin-top'](css2--box.html--8875bbcdefe6.md#propdef-margin-top) and ['margin-bottom'](css2--box.html--8875bbcdefe6.md#propdef-margin-bottom) as well. If the containing block's width depends on this element, then the resulting layout is undefined in CSS 2.1.

<strong>auto</strong>  
See the section on [calculating widths and margins](css2--visudet.html--12e8bc0e6b7c.md#Computing_widths_and_margins) for behavior.

Negative values for margin properties are allowed, but there may be implementation-specific limits.

<a id="propdef-margin-bottom"></a>

<a id="propdef-margin-top"></a>

<strong>'margin-top'</strong>, <strong>'margin-bottom'</strong>

|                       |                                                                                                                                                                                            |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<margin-width\>](css2--box.html--8875bbcdefe6.md#value-def-margin-width) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 0                                                                                                                                                                                          |
| <em>Applies to:</em>   | all elements except elements with table display types other than table-caption, table and inline-table                                                                                     |
| <em>Inherited:</em>   | no                                                                                                                                                                                         |
| <em>Percentages:</em>   | refer to width of containing block                                                                                                                                                         |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                       |
| <em>Computed value:</em>   | the percentage as specified or the absolute length                                                                                                                                         |

> <strong data-conversion-semantic="note">Note</strong>
>
> These properties have no effect on non-replaced inline elements.

<a id="propdef-margin-left"></a>

<a id="propdef-margin-right"></a>

<strong>'margin-right'</strong>, <strong>'margin-left'</strong>

|                       |                                                                                                                                                                                            |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<margin-width\>](css2--box.html--8875bbcdefe6.md#value-def-margin-width) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 0                                                                                                                                                                                          |
| <em>Applies to:</em>   | all elements except elements with table display types other than table-caption, table and inline-table                                                                                     |
| <em>Inherited:</em>   | no                                                                                                                                                                                         |
| <em>Percentages:</em>   | refer to width of containing block                                                                                                                                                         |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                       |
| <em>Computed value:</em>   | the percentage as specified or the absolute length                                                                                                                                         |

These properties set the top, right, bottom, and left margin of a box.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> h1 { margin-top: 2em }
> ```
<a id="propdef-margin"></a>

<strong>'margin'</strong>

|                       |                                                                                                                                                                                                 |
|-----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<margin-width\>](css2--box.html--8875bbcdefe6.md#value-def-margin-width){1,4} \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | see individual properties                                                                                                                                                                       |
| <em>Applies to:</em>   | all elements except elements with table display types other than table-caption, table and inline-table                                                                                          |
| <em>Inherited:</em>   | no                                                                                                                                                                                              |
| <em>Percentages:</em>   | refer to width of containing block                                                                                                                                                              |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                            |
| <em>Computed value:</em>   | see individual properties                                                                                                                                                                       |

The ['margin'](css2--box.html--8875bbcdefe6.md#propdef-margin) property is a shorthand property for setting ['margin-top'](css2--box.html--8875bbcdefe6.md#propdef-margin-top), ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right), ['margin-bottom'](css2--box.html--8875bbcdefe6.md#propdef-margin-bottom), and ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left) at the same place in the style sheet.

If there is only one component value, it applies to all sides. If there are two values, the top and bottom margins are set to the first value and the right and left margins are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values, they apply to the top, right, bottom, and left, respectively.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> body { margin: 2em }         /* all margins set to 2em */
> body { margin: 1em 2em }     /* top & bottom = 1em, right & left = 2em */
> body { margin: 1em 2em 3em } /* top=1em, right=2em, bottom=3em, left=2em */
> ```
>
> The last rule of the example above is equivalent to the example below:
>
> ```text
> 
> body {
>   margin-top: 1em;
>   margin-right: 2em;
>   margin-bottom: 3em;
>   margin-left: 2em;        /* copied from opposite side (right) */
> }
> ```
<a id="collapsing-margins"></a>

### 8.3.1 Collapsing margins

<a id="x26"></a>

<a id="x27"></a>

In CSS, the adjoining margins of two or more boxes (which might or might not be siblings) can combine to form a single margin. Margins that combine this way are said to collapse, and the resulting combined margin is called a collapsed margin.

Adjoining vertical margins collapse, except:

- Margins of the root element's box do not collapse.
- If the top and bottom margins of an element with [clearance](css2--visuren.html--3f334c530cf4.md#clearance) are adjoining, its margins collapse with the adjoining margins of following siblings but that resulting margin does not collapse with the bottom margin of the parent block.

Horizontal margins never collapse.

<a id="x28"></a>

<a id="what-is-adjoining"></a>Two margins are adjoining if and only if:

- both belong to in-flow [block-level boxes](css2--visuren.html--3f334c530cf4.md#block-boxes) that participate in the same [block formatting context](css2--visuren.html--3f334c530cf4.md#block-formatting)
- no line boxes, no clearance, no padding and no border separate them <strong data-conversion-semantic="note">Note:</strong> (Note that [certain zero-height line boxes](css2--visuren.html--3f334c530cf4.md#phantom-line-box) (see [9.4.2](css2--visuren.html--3f334c530cf4.md#inline-formatting)) are ignored for this purpose.)
- both belong to vertically-adjacent box edges, i.e. form one of the following pairs:
  - top margin of a box and top margin of its first in-flow child
  - bottom margin of box and top margin of its next in-flow following sibling
  - bottom margin of a last in-flow child and bottom margin of its parent if the parent has 'auto' computed height
  - top and bottom margins of a box that does not establish a new block formatting context and that has zero computed ['min-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-height), zero or 'auto' computed ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height), and no in-flow children

A collapsed margin is considered adjoining to another margin if any of its component margins is adjoining to that margin.

> <strong data-conversion-semantic="note">Note</strong>
>
> <strong>Note.</strong> Adjoining margins can be generated by elements that are not related as siblings or ancestors.

> <strong data-conversion-semantic="note">Note</strong>
>
> <strong>Note</strong> the above rules imply that:
>
> - Margins between a [floated](css2--visuren.html--3f334c530cf4.md#floats) box and any other box do not collapse (not even between a float and its in-flow children).
> - Margins of elements that establish new block formatting contexts (such as floats and elements with ['overflow'](css2--visufx.html--2bc674cb7ab0.md#propdef-overflow) other than 'visible') do not collapse with their in-flow children.
> - Margins of [absolutely positioned](css2--visuren.html--3f334c530cf4.md#absolutely-positioned) boxes do not collapse (not even with their in-flow children).
> - Margins of inline-block boxes do not collapse (not even with their in-flow children).
> - The bottom margin of an in-flow block-level element always collapses with the top margin of its next in-flow block-level sibling, unless that sibling has clearance.
> - The top margin of an in-flow block element collapses with its first in-flow block-level child's top margin if the element has no top border, no top padding, and the child has no clearance.
> - The bottom margin of an in-flow block box with a ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) of 'auto' and a ['min-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-height) of zero collapses with its last in-flow block-level child's bottom margin if the box has no bottom padding and no bottom border and the child's bottom margin does not collapse with a top margin that has clearance.
> - A box's own margins collapse if the ['min-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-height) property is zero, and it has neither top or bottom borders nor top or bottom padding, and it has a ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) of either 0 or 'auto', and it does not contain a line box, and all of its in-flow children's margins (if any) collapse.

When two or more margins collapse, the resulting margin width is the maximum of the collapsing margins' widths. In the case of negative margins, the maximum of the absolute values of the negative adjoining margins is deducted from the maximum of the positive adjoining margins. If there are no positive margins, the maximum of the absolute values of the adjoining margins is deducted from zero.

<a id="x29"></a>

<a id="collapsed-through"></a>If the top and bottom margins of a box are adjoining, then it is possible for margins to collapse through it. In this case, the position of the element depends on its relationship with the other elements whose margins are being collapsed.

- If the element's margins are collapsed with its parent's top margin, the top border edge of the box is defined to be the same as the parent's.
- Otherwise, either the element's parent is not taking part in the margin collapsing, or only the parent's bottom margin is involved. The position of the element's top border edge is the same as it would have been if the element had a non-zero bottom border.

Note that the positions of elements that have been collapsed through have no effect on the positions of the other elements with whose margins they are being collapsed; the top border edge position is only required for laying out descendants of these elements.

<a id="padding-properties"></a>

## 8.4 Padding properties: ['padding-top'](css2--box.html--8875bbcdefe6.md#propdef-padding-top), ['padding-right'](css2--box.html--8875bbcdefe6.md#propdef-padding-right), ['padding-bottom'](css2--box.html--8875bbcdefe6.md#propdef-padding-bottom), ['padding-left'](css2--box.html--8875bbcdefe6.md#propdef-padding-left), and ['padding'](css2--box.html--8875bbcdefe6.md#propdef-padding)

The padding properties specify the width of the [padding area](#box-padding-area) of a box. The ['padding'](css2--box.html--8875bbcdefe6.md#propdef-padding) shorthand property sets the padding for all four sides while the other padding properties only set their respective side.

<a id="value-def-padding-width"></a>

The properties defined in this section refer to the <strong>&lt;padding-width&gt;</strong> value type, which may take one of the following values:

[<strong>&lt;length&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-length)  
Specifies a fixed width.

[<strong>&lt;percentage&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-percentage)  
The percentage is calculated with respect to the <em>width</em> of the generated box's [containing block](css2--visuren.html--3f334c530cf4.md#containing-block), even for ['padding-top'](css2--box.html--8875bbcdefe6.md#propdef-padding-top) and ['padding-bottom'](css2--box.html--8875bbcdefe6.md#propdef-padding-bottom). If the containing block's width depends on this element, then the resulting layout is undefined in CSS 2.1.

Unlike margin properties, values for padding values cannot be negative. Like margin properties, percentage values for padding properties refer to the width of the generated box's containing block.

<a id="propdef-padding-left"></a>

<a id="propdef-padding-bottom"></a>

<a id="propdef-padding-right"></a>

<a id="propdef-padding-top"></a>

<strong>'padding-top'</strong>, <strong>'padding-right'</strong>, <strong>'padding-bottom'</strong>, <strong>'padding-left'</strong>

|                       |                                                                                                                                                                                              |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<padding-width\>](css2--box.html--8875bbcdefe6.md#value-def-padding-width) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 0                                                                                                                                                                                            |
| <em>Applies to:</em>   | all elements except table-row-group, table-header-group, table-footer-group, table-row, table-column-group and table-column                                                                  |
| <em>Inherited:</em>   | no                                                                                                                                                                                           |
| <em>Percentages:</em>   | refer to width of containing block                                                                                                                                                           |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                         |
| <em>Computed value:</em>   | the percentage as specified or the absolute length                                                                                                                                           |

These properties set the top, right, bottom, and left padding of a box.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> blockquote { padding-top: 0.3em }
> ```
<a id="propdef-padding"></a>

<strong>'padding'</strong>

|                       |                                                                                                                                                                                                   |
|-----------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<padding-width\>](css2--box.html--8875bbcdefe6.md#value-def-padding-width){1,4} \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | see individual properties                                                                                                                                                                         |
| <em>Applies to:</em>   | all elements except table-row-group, table-header-group, table-footer-group, table-row, table-column-group and table-column                                                                       |
| <em>Inherited:</em>   | no                                                                                                                                                                                                |
| <em>Percentages:</em>   | refer to width of containing block                                                                                                                                                                |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                              |
| <em>Computed value:</em>   | see individual properties                                                                                                                                                                         |

The ['padding'](css2--box.html--8875bbcdefe6.md#propdef-padding) property is a shorthand property for setting ['padding-top'](css2--box.html--8875bbcdefe6.md#propdef-padding-top), ['padding-right'](css2--box.html--8875bbcdefe6.md#propdef-padding-right), ['padding-bottom'](css2--box.html--8875bbcdefe6.md#propdef-padding-bottom), and ['padding-left'](css2--box.html--8875bbcdefe6.md#propdef-padding-left) at the same place in the style sheet.

If there is only one component value, it applies to all sides. If there are two values, the top and bottom paddings are set to the first value and the right and left paddings are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values, they apply to the top, right, bottom, and left, respectively.

The surface color or image of the padding area is specified via the ['background'](css2--colors.html--5784063d2778.md#propdef-background) property:

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> h1 { 
>   background: white; 
>   padding: 1em 2em;
> } 
> ```
>
> The example above specifies a '1em' vertical padding (['padding-top'](css2--box.html--8875bbcdefe6.md#propdef-padding-top) and ['padding-bottom'](css2--box.html--8875bbcdefe6.md#propdef-padding-bottom)) and a '2em' horizontal padding (['padding-right'](css2--box.html--8875bbcdefe6.md#propdef-padding-right) and ['padding-left'](css2--box.html--8875bbcdefe6.md#propdef-padding-left)). The 'em' unit is [relative](css2--syndata.html--02e71c159e14.md#absrel-units) to the element's font size: '1em' is equal to the size of the font in use.

<a id="border-properties"></a>

## 8.5 Border properties

The border properties specify the width, color, and style of the [border area](#box-border-area) of a box. These properties apply to all elements.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong>
Notably for HTML, 
user agents may render borders for certain user interface elements (e.g.,
buttons, menus, etc.) differently than for
"ordinary" elements.
</em>

<a id="border-width-properties"></a>

### 8.5.1 Border width: ['border-top-width'](css2--box.html--8875bbcdefe6.md#propdef-border-top-width), ['border-right-width'](css2--box.html--8875bbcdefe6.md#propdef-border-right-width), ['border-bottom-width'](css2--box.html--8875bbcdefe6.md#propdef-border-bottom-width), ['border-left-width'](css2--box.html--8875bbcdefe6.md#propdef-border-left-width), and ['border-width'](css2--box.html--8875bbcdefe6.md#propdef-border-width)

<a id="value-def-border-width"></a>

The border width properties specify the width of the [border area](#box-border-area). The properties defined in this section refer to the <strong>&lt;border-width&gt;</strong> value type, which may take one of the following values:

<strong>thin</strong>  
A thin border.

<strong>medium</strong>  
A medium border.

<strong>thick</strong>  
A thick border.

[<strong>&lt;length&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-length)  
The border's thickness has an explicit value. Explicit border widths cannot be negative.

The interpretation of the first three values depends on the user agent. The following relationships must hold, however:

'thin' \<='medium' \<= 'thick'.

Furthermore, these widths must be constant throughout a document.

<a id="propdef-border-left-width"></a>

<a id="propdef-border-bottom-width"></a>

<a id="propdef-border-right-width"></a>

<a id="propdef-border-top-width"></a>

<strong>'border-top-width'</strong>, <strong>'border-right-width'</strong>, <strong>'border-bottom-width'</strong>, <strong>'border-left-width'</strong>

|                       |                                                                                                                                                                                            |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<border-width\>](css2--box.html--8875bbcdefe6.md#value-def-border-width) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | medium                                                                                                                                                                                     |
| <em>Applies to:</em>   | all elements                                                                                                                                                                               |
| <em>Inherited:</em>   | no                                                                                                                                                                                         |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                        |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                       |
| <em>Computed value:</em>   | absolute length; '0' if the border style is 'none' or 'hidden'                                                                                                                             |

These properties set the width of the top, right, bottom, and left border of a box.

<a id="propdef-border-width"></a>

<strong>'border-width'</strong>

|                       |                                                                                                                                                                                                 |
|-----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<border-width\>](css2--box.html--8875bbcdefe6.md#value-def-border-width){1,4} \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | see individual properties                                                                                                                                                                       |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                    |
| <em>Inherited:</em>   | no                                                                                                                                                                                              |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                             |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                            |
| <em>Computed value:</em>   | see individual properties                                                                                                                                                                       |

This property is a shorthand property for setting ['border-top-width'](css2--box.html--8875bbcdefe6.md#propdef-border-top-width), ['border-right-width'](css2--box.html--8875bbcdefe6.md#propdef-border-right-width), ['border-bottom-width'](css2--box.html--8875bbcdefe6.md#propdef-border-bottom-width), and ['border-left-width'](css2--box.html--8875bbcdefe6.md#propdef-border-left-width) at the same place in the style sheet.

If there is only one component value, it applies to all sides. If there are two values, the top and bottom borders are set to the first value and the right and left are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values, they apply to the top, right, bottom, and left, respectively.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In the examples below, the comments indicate the resulting widths of the top, right, bottom, and left borders:
>
> ```text
> 
> h1 { border-width: thin }                   /* thin thin thin thin */
> h1 { border-width: thin thick }             /* thin thick thin thick */
> h1 { border-width: thin thick medium }      /* thin thick medium thick */
> ```
<a id="border-color-properties"></a>

### 8.5.2 Border color: ['border-top-color'](css2--box.html--8875bbcdefe6.md#propdef-border-top-color), ['border-right-color'](css2--box.html--8875bbcdefe6.md#propdef-border-right-color), ['border-bottom-color'](css2--box.html--8875bbcdefe6.md#propdef-border-bottom-color), ['border-left-color'](css2--box.html--8875bbcdefe6.md#propdef-border-left-color), and ['border-color'](css2--box.html--8875bbcdefe6.md#propdef-border-color)

The border color properties specify the color of a box's border.

<a id="propdef-border-left-color"></a>

<a id="propdef-border-bottom-color"></a>

<a id="propdef-border-right-color"></a>

<a id="propdef-border-top-color"></a>

<strong>'border-top-color'</strong>, <strong>'border-right-color'</strong>, <strong>'border-bottom-color'</strong>, <strong>'border-left-color'</strong>

|                       |                                                                                                                                                                                                 |
|-----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<color\>](css2--syndata.html--02e71c159e14.md#value-def-color) \| transparent \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | the value of the 'color' property                                                                                                                                                               |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                    |
| <em>Inherited:</em>   | no                                                                                                                                                                                              |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                             |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                            |
| <em>Computed value:</em>   | when taken from the 'color' property, the computed value of 'color'; otherwise, as specified                                                                                                    |

<a id="propdef-border-color"></a>

<strong>'border-color'</strong>

|                       |                                                                                                                                                                                                            |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | \[ [\<color\>](css2--syndata.html--02e71c159e14.md#value-def-color) \| transparent \]{1,4} \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | see individual properties                                                                                                                                                                                  |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                               |
| <em>Inherited:</em>   | no                                                                                                                                                                                                         |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                        |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                       |
| <em>Computed value:</em>   | see individual properties                                                                                                                                                                                  |

The ['border-color'](css2--box.html--8875bbcdefe6.md#propdef-border-color) property sets the color of the four borders. Values have the following meanings:

<a id="x47"></a>

[<strong>&lt;color&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-color)

Specifies a color value.

<strong>transparent</strong>

The border is transparent (though it may have width).

The ['border-color'](css2--box.html--8875bbcdefe6.md#propdef-border-color) property can have from one to four component values, and the values are set on the different sides as for ['border-width'](css2--box.html--8875bbcdefe6.md#propdef-border-width).

If an element's border color is not specified with a border property, user agents must use the value of the element's ['color'](css2--colors.html--5784063d2778.md#propdef-color) property as the [computed value](css2--cascade.html--c7aff33e6f0d.md#computed-value) for the border color.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> In this example, the border will be a solid black line.
>
> ```text
> 
> p { 
>   color: black; 
>   background: white; 
>   border: solid;
> }
> ```
<a id="border-style-properties"></a>

### 8.5.3 Border style: ['border-top-style'](css2--box.html--8875bbcdefe6.md#propdef-border-top-style), ['border-right-style'](css2--box.html--8875bbcdefe6.md#propdef-border-right-style), ['border-bottom-style'](css2--box.html--8875bbcdefe6.md#propdef-border-bottom-style), ['border-left-style'](css2--box.html--8875bbcdefe6.md#propdef-border-left-style), and ['border-style'](css2--box.html--8875bbcdefe6.md#propdef-border-style)

<a id="value-def-border-style"></a>

The border style properties specify the line style of a box's border (solid, double, dashed, etc.). The properties defined in this section refer to the <strong>&lt;border-style&gt;</strong> value type, which may take one of the following values:

<a id="value-def-bo-none"></a>

<strong><span title="'none'::as border style"><a>none</a></span></strong>

No border; the computed border width is zero.

<a id="value-def-hidden"></a>

<strong><span title="'hidden'"><a>hidden</a></span></strong>

Same as 'none', except in terms of [border conflict resolution](css2--tables.html--201812dd6e3c.md#border-conflict-resolution) for [table elements](css2--tables.html--201812dd6e3c.md).

<a id="value-def-dotted"></a>

<strong><span title="'dotted'"><a>dotted</a></span></strong>

The border is a series of dots.

<a id="value-def-dashed"></a>

<strong><span title="'dashed'"><a>dashed</a></span></strong>

The border is a series of short line segments.

<a id="value-def-solid"></a>

<strong><span title="'solid'"><a>solid</a></span></strong>

The border is a single line segment.

<a id="value-def-double"></a>

<strong><span title="'double'"><a>double</a></span></strong>

The border is two solid lines. The sum of the two lines and the space between them equals the value of ['border-width'](css2--box.html--8875bbcdefe6.md#propdef-border-width).

<a id="value-def-groove"></a>

<strong><span title="'groove'"><a>groove</a></span></strong>

The border looks as though it were carved into the canvas.

<a id="value-def-ridge"></a>

<strong><span title="'ridge'"><a>ridge</a></span></strong>

The opposite of 'groove': the border looks as though it were coming out of the canvas.

<a id="value-def-inset"></a>

<strong><span title="'inset'"><a>inset</a></span></strong>

The border makes the box look as though it were embedded in the canvas.

<a id="value-def-outset"></a>

<strong><span title="'outset'"><a>outset</a></span></strong>

The opposite of 'inset': the border makes the box look as though it were coming out of the canvas.

All borders are drawn on top of the box's background. The color of borders drawn for values of 'groove', 'ridge', 'inset', and 'outset' depends on the element's [border color properties](#border-color-properties), but UAs may choose their own algorithm to calculate the actual colors used. For instance, if the 'border-color' has the value 'silver', then a UA could use a gradient of colors from white to dark gray to indicate a sloping border.

<a id="propdef-border-left-style"></a>

<a id="propdef-border-bottom-style"></a>

<a id="propdef-border-right-style"></a>

<a id="propdef-border-top-style"></a>

<strong>'border-top-style'</strong>, <strong>'border-right-style'</strong>, <strong>'border-bottom-style'</strong>, <strong>'border-left-style'</strong>

|                       |                                                                                                                                                                                            |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<border-style\>](css2--box.html--8875bbcdefe6.md#value-def-border-style) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | none                                                                                                                                                                                       |
| <em>Applies to:</em>   | all elements                                                                                                                                                                               |
| <em>Inherited:</em>   | no                                                                                                                                                                                         |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                        |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                       |
| <em>Computed value:</em>   | as specified                                                                                                                                                                               |

<a id="propdef-border-style"></a>

<strong>'border-style'</strong>

|                       |                                                                                                                                                                                                 |
|-----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<border-style\>](css2--box.html--8875bbcdefe6.md#value-def-border-style){1,4} \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | see individual properties                                                                                                                                                                       |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                    |
| <em>Inherited:</em>   | no                                                                                                                                                                                              |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                             |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                            |
| <em>Computed value:</em>   | see individual properties                                                                                                                                                                       |

The ['border-style'](css2--box.html--8875bbcdefe6.md#propdef-border-style) property sets the style of the four borders. It can have from one to four component values, and the values are set on the different sides as for ['border-width'](css2--box.html--8875bbcdefe6.md#propdef-border-width) above.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> #xy34 { border-style: solid dotted }
> ```
>
> In the above example, the horizontal borders will be 'solid' and the vertical borders will be 'dotted'.

Since the initial value of the border styles is 'none', no borders will be visible unless the border style is set.

<a id="border-shorthand-properties"></a>

### 8.5.4 Border shorthand properties: ['border-top'](css2--box.html--8875bbcdefe6.md#propdef-border-top), ['border-right'](css2--box.html--8875bbcdefe6.md#propdef-border-right), ['border-bottom'](css2--box.html--8875bbcdefe6.md#propdef-border-bottom), ['border-left'](css2--box.html--8875bbcdefe6.md#propdef-border-left), and ['border'](css2--box.html--8875bbcdefe6.md#propdef-border)

<a id="propdef-border-left"></a>

<a id="propdef-border-bottom"></a>

<a id="propdef-border-right"></a>

<a id="propdef-border-top"></a>

<strong>'border-top'</strong>, <strong>'border-right'</strong>, <strong>'border-bottom'</strong>, <strong>'border-left'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                                                                                      |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | \[ [\<border-width\>](css2--box.html--8875bbcdefe6.md#value-def-border-width) \|\| [\<border-style\>](css2--box.html--8875bbcdefe6.md#value-def-border-style) \|\| [\<'border-top-color'\>](css2--box.html--8875bbcdefe6.md#propdef-border-top-color) \] \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | see individual properties                                                                                                                                                                                                                                                                                                                                                                                            |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                                                                                                                                         |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                                                                                                                                                 |
| <em>Computed value:</em>   | see individual properties                                                                                                                                                                                                                                                                                                                                                                                            |

This is a shorthand property for setting the width, style, and color of the top, right, bottom, and left border of a box.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> h1 { border-bottom: thick solid red }
> ```
>
> The above rule will set the width, style, and color of the border <strong>below</strong> the H1 element. Omitted values are set to their [initial values](css2--about.html--c67ff594c990.md#initial-value). Since the following rule does not specify a border color, the border will have the color specified by the [ 'color'](css2--colors.html--5784063d2778.md#propdef-color) property:
>
> ```text
> 
> H1 { border-bottom: thick solid }
> ```
<a id="propdef-border"></a>

<strong>'border'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                                                                                      |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | \[ [\<border-width\>](css2--box.html--8875bbcdefe6.md#value-def-border-width) \|\| [\<border-style\>](css2--box.html--8875bbcdefe6.md#value-def-border-style) \|\| [\<'border-top-color'\>](css2--box.html--8875bbcdefe6.md#propdef-border-top-color) \] \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | see individual properties                                                                                                                                                                                                                                                                                                                                                                                            |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                                                                                                                                         |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                                                                                                                                                   |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                                                                                                                                                                                                                                  |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                                                                                                                                                 |
| <em>Computed value:</em>   | see individual properties                                                                                                                                                                                                                                                                                                                                                                                            |

The ['border'](css2--box.html--8875bbcdefe6.md#propdef-border) property is a shorthand property for setting the same width, color, and style for all four borders of a box. Unlike the shorthand ['margin'](css2--box.html--8875bbcdefe6.md#propdef-margin) and ['padding'](css2--box.html--8875bbcdefe6.md#propdef-padding) properties, the ['border'](css2--box.html--8875bbcdefe6.md#propdef-border) property cannot set different values on the four borders. To do so, one or more of the other border properties must be used.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> For example, the first rule below is equivalent to the set of four rules shown after it:
>
> ```text
> 
> p { border: solid red }
> p {
>   border-top: solid red;
>   border-right: solid red;
>   border-bottom: solid red;
>   border-left: solid red
> }
> ```
Since, to some extent, the properties have overlapping functionality, the order in which the rules are specified is important.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Consider this example:
>
> ```text
> 
> blockquote {
>   border: solid red;
>   border-left: double;
>   color: black;
> }
> ```
>
> In the above example, the color of the left border is black, while the other borders are red. This is due to ['border-left'](css2--box.html--8875bbcdefe6.md#propdef-border-left) setting the width, style, and color. Since the color value is not given by the ['border-left'](css2--box.html--8875bbcdefe6.md#propdef-border-left) property, it will be taken from the ['color'](css2--colors.html--5784063d2778.md#propdef-color) property. The fact that the ['color'](css2--colors.html--5784063d2778.md#propdef-color) property is set after the ['border-left'](css2--box.html--8875bbcdefe6.md#propdef-border-left) property is not relevant.

<a id="bidi-box-model"></a>

## 8.6 The box model for inline elements in bidirectional context

For each line box, UAs must take the inline boxes generated for each element and render the margins, borders and padding in visual order (not logical order).

When the element's ['direction'](css2--visuren.html--3f334c530cf4.md#propdef-direction) property is 'ltr', the left-most generated box of the first line box in which the element appears has the left margin, left border and left padding, and the right-most generated box of the last line box in which the element appears has the right padding, right border and right margin.

When the element's ['direction'](css2--visuren.html--3f334c530cf4.md#propdef-direction) property is 'rtl', the right-most generated box of the first line box in which the element appears has the right padding, right border and right margin, and the left-most generated box of the last line box in which the element appears has the left margin, left border and left padding.
