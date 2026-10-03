Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Visual formatting model details](https://www.w3.org/TR/2011/REC-CSS2-20110607/visudet.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Visual formatting model details

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/visudet.html

Snapshot SHA-256: 12e8bc0e6b7c96d81c85f40690f296333342c0babe26ecee1fbf8a1fbd7f3834

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.

---

<a id="q10.0"></a>

# 10 Visual formatting model details

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

<a id="containing-block-details"></a>

## 10.1 Definition of "containing block"

<a id="x0"></a>

The position and size of an element's box(es) are sometimes calculated relative to a certain rectangle, called the containing block of the element. The containing block of an element is defined as follows:

1.  <a id="x1"></a>

    The containing block in which the [root element](css2--conform.html--de58593b67d7.md#root) lives is a rectangle called the initial containing block. For continuous media, it has the dimensions of the [viewport](css2--visuren.html--3f334c530cf4.md#viewport) and is anchored at the canvas origin; it is the [page area](css2--page.html--984664616bba.md#page-area) for paged media. The 'direction' property of the initial containing block is the same as for the root element.

2.  For other elements, if the element's position is 'relative' or 'static', the containing block is formed by the content edge of the nearest [block container](css2--visuren.html--3f334c530cf4.md#block-boxes) ancestor box.

3.  If the element has 'position: fixed', the containing block is established by the [viewport](css2--visuren.html--3f334c530cf4.md#viewport) in the case of continuous media or the page area in the case of paged media.

4.  If the element has 'position: absolute', the containing block is established by the nearest ancestor with a ['position'](css2--visuren.html--3f334c530cf4.md#propdef-position) of 'absolute', 'relative' or 'fixed', in the following way:

    1.  In the case that the ancestor is an inline element, the containing block is the bounding box around the padding boxes of the first and the last inline boxes generated for that element. In CSS 2.1, if the inline element is split across multiple lines, the containing block is undefined.
    2.  Otherwise, the containing block is formed by the [padding edge](css2--box.html--8875bbcdefe6.md#padding-edge) of the ancestor.

    If there is no such ancestor, the containing block is the initial containing block.

In paged media, an absolutely positioned element is positioned relative to its containing block ignoring any page breaks (as if the document were continuous). The element may subsequently be broken over several pages.

For absolutely positioned content that resolves to a position on a page other than the page being laid out (the current page), or resolves to a position on the current page which has already been rendered for printing, printers may place the content

- on another location on the current page,
- on a subsequent page, or
- may omit it.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that a block-level element that is split over several pages may have a different width on each page and that there may be device-specific limits.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> With no positioning, the containing blocks (C.B.) in the following document:
>
> ```text
> 
> <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN">
> <HTML>
>    <HEAD>
>       <TITLE>Illustration of containing blocks</TITLE>
>    </HEAD>
>    <BODY id="body">
>       <DIV id="div1">
>       <P id="p1">This is text in the first paragraph...</P>
>       <P id="p2">This is text <EM id="em1"> in the 
>       <STRONG id="strong1">second</STRONG> paragraph.</EM></P>
>       </DIV>
>    </BODY>
> </HTML>
> ```
>
> are established as follows:
>
> |                      |                             |
> |----------------------|-----------------------------|
> | For box generated by | <strong>C.B. is established by</strong>         |
> | html                 | initial C.B. (UA-dependent) |
> | body                 | html                        |
> | div1                 | body                        |
> | p1                   | div1                        |
> | p2                   | div1                        |
> | em1                  | p2                          |
> | strong1              | p2                          |
>
> If we position "div1":
>
> ```text
> 
>    #div1 { position: absolute; left: 50px; top: 50px }
> ```
>
> its containing block is no longer "body"; it becomes the initial containing block (since there are no other positioned ancestor boxes).
>
> If we position "em1" as well:
>
> ```text
> 
>    #div1 { position: absolute; left: 50px; top: 50px }
>    #em1  { position: absolute; left: 100px; top: 100px }
> ```
>
> the table of containing blocks becomes:
>
> |                      |                             |
> |----------------------|-----------------------------|
> | For box generated by | <strong>C.B. is established by</strong>         |
> | html                 | initial C.B. (UA-dependent) |
> | body                 | html                        |
> | div1                 | initial C.B.                |
> | p1                   | div1                        |
> | p2                   | div1                        |
> | em1                  | div1                        |
> | strong1              | em1                         |
>
> By positioning "em1", its containing block becomes the nearest positioned ancestor box (i.e., that generated by "div1").

<a id="the-width-property"></a>

## 10.2 Content width: the ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) property

<a id="propdef-width"></a>

<strong>'width'</strong>

|                       |                                                                                                                                                                                                                                                                                                |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| auto \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | auto                                                                                                                                                                                                                                                                                           |
| <em>Applies to:</em>   | all elements but non-replaced inline elements, table rows, and row groups                                                                                                                                                                                                                      |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                             |
| <em>Percentages:</em>   | refer to width of containing block                                                                                                                                                                                                                                                             |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                           |
| <em>Computed value:</em>   | the percentage or 'auto' as specified or the absolute length                                                                                                                                                                                                                                   |

This property specifies the [content width](css2--box.html--8875bbcdefe6.md#content-width) of boxes.

This property does not apply to non-replaced [inline](css2--visuren.html--3f334c530cf4.md#inline-boxes) elements. The content width of a non-replaced inline element's boxes is that of the rendered content within them (<em>before</em> any relative offset of children). Recall that inline boxes flow into [line boxes](css2--visuren.html--3f334c530cf4.md#line-box). The width of line boxes is given by the their [containing block](css2--visuren.html--3f334c530cf4.md#containing-block), but may be shorted by the presence of [floats](css2--visuren.html--3f334c530cf4.md#floats).

Values have the following meanings:

<a id="x4"></a>

[<strong>&lt;length&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-length)

Specifies the width of the content area using a length unit.

<a id="x5"></a>

[<strong>&lt;percentage&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-percentage)

Specifies a percentage width. The percentage is calculated with respect to the width of the generated box's [containing block](css2--visuren.html--3f334c530cf4.md#containing-block). If the containing block's width depends on this element's width, then the resulting layout is undefined in CSS 2.1. <strong data-conversion-semantic="note">Note:</strong> Note: For absolutely positioned elements whose containing block is based on a block container element, the percentage is calculated with respect to the width of the <em>padding box</em> of that element. This is a change from CSS1, where the percentage width was always calculated with respect to the <em>content box</em> of the parent element.

<strong>auto</strong>

The width depends on the values of other properties. See the sections below.

Negative values for ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) are illegal.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> For example, the following rule fixes the content width of paragraphs at 100 pixels:
>
> ```text
> 
> p { width: 100px }
> ```
<a id="Computing_widths_and_margins"></a>

## 10.3 Calculating widths and margins

The values of an element's ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width), ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left), ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right), ['left'](css2--visuren.html--3f334c530cf4.md#propdef-left) and ['right'](css2--visuren.html--3f334c530cf4.md#propdef-right) properties as used for layout depend on the type of box generated and on each other. (The value used for layout is sometimes referred to as the [used value](css2--cascade.html--c7aff33e6f0d.md#usedValue).) In principle, the values used are the same as the computed values, with 'auto' replaced by some suitable value, and percentages calculated based on the containing block, but there are exceptions. The following situations need to be distinguished:

1.  inline, non-replaced elements
2.  inline, replaced elements
3.  block-level, non-replaced elements in normal flow
4.  block-level, replaced elements in normal flow
5.  floating, non-replaced elements
6.  floating, replaced elements
7.  absolutely positioned, non-replaced elements
8.  absolutely positioned, replaced elements
9.  'inline-block', non-replaced elements in normal flow
10. 'inline-block', replaced elements in normal flow

For Points 1-6 and 9-10, the values of 'left' and 'right' in the case of relatively positioned elements are determined by the rules in [section 9.4.3.](css2--visuren.html--3f334c530cf4.md#relative-positioning)

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> The used value of 'width'
calculated below is a tentative value, and may have to be calculated
multiple times, depending on <a href="css2--visudet.html--12e8bc0e6b7c.md#propdef-min-width"><span>'min-width'</span></a> and <a href="css2--visudet.html--12e8bc0e6b7c.md#propdef-max-width"><span>'max-width'</span></a>, see the section <a href="#min-max-widths">Minimum and maximum widths</a> below.</em>

<a id="inline-width"></a>

### 10.3.1 Inline, non-replaced elements

The ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) property does not apply. A computed value of 'auto' for ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left) or ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right) becomes a used value of '0'.

<a id="inline-replaced-width"></a>

### 10.3.2 Inline, replaced elements

A computed value of 'auto' for ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left) or ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right) becomes a used value of '0'.

If ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) and ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) both have computed values of 'auto' and the element also has an intrinsic width, then that intrinsic width is the used value of ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width).

If ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) and ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) both have computed values of 'auto' and the element has no intrinsic width, but does have an intrinsic height and intrinsic ratio; or if ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) has a computed value of 'auto', ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) has some other computed value, and the element does have an intrinsic ratio; then the used value of ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) is:

> (used height) \* (intrinsic ratio)

If 'height' and ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) both have computed values of 'auto' and the element has an intrinsic ratio but no intrinsic height or width, then the used value of 'width' is undefined in CSS 2.1. However, it is suggested that, if the containing block's width does not itself depend on the replaced element's width, then the used value of 'width' is calculated from the constraint equation used for block-level, non-replaced elements in normal flow.

Otherwise, if ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) has a computed value of 'auto', and the element has an intrinsic width, then that intrinsic width is the used value of ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width).

Otherwise, if ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) has a computed value of 'auto', but none of the conditions above are met, then the used value of ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) becomes 300px. If 300px is too wide to fit the device, UAs should use the width of the largest rectangle that has a 2:1 ratio and fits the device instead.

<a id="blockwidth"></a>

### 10.3.3 Block-level, non-replaced elements in normal flow

<a id="width-constraints"></a>

The following constraints must hold among the used values of the other properties:

> ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left) + ['border-left-width'](css2--box.html--8875bbcdefe6.md#propdef-border-left-width) + ['padding-left'](css2--box.html--8875bbcdefe6.md#propdef-padding-left) + ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) + ['padding-right'](css2--box.html--8875bbcdefe6.md#propdef-padding-right) + ['border-right-width'](css2--box.html--8875bbcdefe6.md#propdef-border-right-width) + ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right) = width of [containing block](#containing-block-details)

If 'width' is not 'auto' and 'border-left-width' + 'padding-left' + 'width' + 'padding-right' + 'border-right-width' (plus any of 'margin-left' or 'margin-right' that are not 'auto') is larger than the width of the containing block, then any 'auto' values for 'margin-left' or 'margin-right' are, for the following rules, treated as zero.

If all of the above have a computed value other than 'auto', the values are said to be "over-constrained" and one of the used values will have to be different from its computed value. If the ['direction'](css2--visuren.html--3f334c530cf4.md#propdef-direction) property of the containing block has the value 'ltr', the specified value of ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right) is ignored and the value is calculated so as to make the equality true. If the value of ['direction'](css2--visuren.html--3f334c530cf4.md#propdef-direction) is 'rtl', this happens to ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left) instead.

If there is exactly one value specified as 'auto', its used value follows from the equality.

If ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) is set to 'auto', any other 'auto' values become '0' and ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) follows from the resulting equality.

If both ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left) and ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right) are 'auto', their used values are equal. This horizontally centers the element with respect to the edges of the containing block.

<a id="block-replaced-width"></a>

### 10.3.4 Block-level, replaced elements in normal flow

The used value of ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) is determined as for [inline replaced elements](#inline-replaced-width). Then the rules [for non-replaced block-level elements](#blockwidth) are applied to determine the margins.

<a id="float-width"></a>

### 10.3.5 Floating, non-replaced elements

If ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left), or ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right) are computed as 'auto', their used value is '0'.

If ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) is computed as 'auto', the used value is the "shrink-to-fit" width.

<a id="shrink-to-fit-float"></a>

Calculation of the shrink-to-fit width is similar to calculating the width of a table cell using the automatic table layout algorithm. Roughly: calculate the preferred width by formatting the content without breaking lines other than where explicit line breaks occur, and also calculate the preferred <em>minimum</em> width, e.g., by trying all possible line breaks. CSS 2.1 does not define the exact algorithm. Thirdly, find the <em>available width</em>: in this case, this is the width of the containing block minus the used values of 'margin-left', 'border-left-width', 'padding-left', 'padding-right', 'border-right-width', 'margin-right', and the widths of any relevant scroll bars.

Then the shrink-to-fit width is: min(max(preferred minimum width, available width), preferred width).

<a id="float-replaced-width"></a>

### 10.3.6 Floating, replaced elements

If ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left) or ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right) are computed as 'auto', their used value is '0'. The used value of ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) is determined as for [inline replaced elements](#inline-replaced-width).

<a id="abs-non-replaced-width"></a>

### 10.3.7 Absolutely positioned, non-replaced elements

<a id="static-position"></a>

For the purposes of this section and the next, the term "static position" (of an element) refers, roughly, to the position an element would have had in the normal flow. More precisely:

- The static-position containing block is the containing block of a hypothetical box that would have been the first box of the element if its specified ['position'](css2--visuren.html--3f334c530cf4.md#propdef-position) value had been 'static' and its specified 'float' had been 'none'. (Note that due to the rules in [section 9.7](css2--visuren.html--3f334c530cf4.md#dis-pos-flo) this hypothetical calculation might require also assuming a different computed value for 'display'.)
- The static position for 'left' is the distance from the left edge of the containing block to the left margin edge of a hypothetical box that would have been the first box of the element if its ['position'](css2--visuren.html--3f334c530cf4.md#propdef-position) property had been 'static' and ['float'](css2--visuren.html--3f334c530cf4.md#propdef-float) had been 'none'. The value is negative if the hypothetical box is to the left of the containing block.
- The static position for 'right' is the distance from the right edge of the containing block to the right margin edge of the same hypothetical box as above. The value is positive if the hypothetical box is to the left of the containing block's edge.

But rather than actually calculating the dimensions of that hypothetical box, user agents are free to make a guess at its probable position.

For the purposes of calculating the static position, the containing block of fixed positioned elements is the initial containing block instead of the viewport, and all scrollable boxes should be assumed to be scrolled to their origin.

The constraint that determines the used values for these elements is:

> 'left' + 'margin-left' + 'border-left-width' + 'padding-left' + 'width' + 'padding-right' + 'border-right-width' + 'margin-right' + 'right' = width of containing block

If all three of 'left', 'width', and 'right' are 'auto': First set any 'auto' values for 'margin-left' and 'margin-right' to 0. Then, if the 'direction' property of the element establishing the static-position containing block is 'ltr' set 'left' to the [static position](#static-position) and apply rule number three below; otherwise, set 'right' to the [static position](#static-position) and apply rule number one below.

If none of the three is 'auto': If both 'margin-left' and 'margin-right' are 'auto', solve the equation under the extra constraint that the two margins get equal values, unless this would make them negative, in which case when direction of the containing block is 'ltr' ('rtl'), set 'margin-left' ('margin-right') to zero and solve for 'margin-right' ('margin-left'). If one of 'margin-left' or 'margin-right' is 'auto', solve the equation for that value. If the values are over-constrained, ignore the value for 'left' (in case the 'direction' property of the containing block is 'rtl') or 'right' (in case 'direction' is 'ltr') and solve for that value.

Otherwise, set 'auto' values for 'margin-left' and 'margin-right' to 0, and pick the one of the following six rules that applies.

1.  'left' and 'width' are 'auto' and 'right' is not 'auto', then the width is shrink-to-fit. Then solve for 'left'
2.  'left' and 'right' are 'auto' and 'width' is not 'auto', then if the 'direction' property of the element establishing the static-position containing block is 'ltr' set 'left' to the [static position](#static-position), otherwise set 'right' to the [static position](#static-position). Then solve for 'left' (if 'direction is 'rtl') or 'right' (if 'direction' is 'ltr').
3.  'width' and 'right' are 'auto' and 'left' is not 'auto', then the width is shrink-to-fit . Then solve for 'right'
4.  'left' is 'auto', 'width' and 'right' are not 'auto', then solve for 'left'
5.  'width' is 'auto', 'left' and 'right' are not 'auto', then solve for 'width'
6.  'right' is 'auto', 'left' and 'width' are not 'auto', then solve for 'right'

Calculation of the shrink-to-fit width is similar to calculating the width of a table cell using the automatic table layout algorithm. Roughly: calculate the preferred width by formatting the content without breaking lines other than where explicit line breaks occur, and also calculate the preferred <em>minimum</em> width, e.g., by trying all possible line breaks. CSS 2.1 does not define the exact algorithm. Thirdly, calculate the <em>available width</em>: this is found by solving for 'width' after setting 'left' (in case 1) or 'right' (in case 3) to 0.

Then the shrink-to-fit width is: min(max(preferred minimum width, available width), preferred width).

<a id="abs-replaced-width"></a>

### 10.3.8 Absolutely positioned, replaced elements

In this case, [section 10.3.7](#abs-non-replaced-width) applies up through and including the constraint equation, but the rest of [section 10.3.7](#abs-non-replaced-width) is replaced by the following rules:

1.  The used value of ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) is determined as for [inline replaced elements](#inline-replaced-width). If ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left) or ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right) is specified as 'auto' its used value is determined by the rules below.
2.  If both ['left'](css2--visuren.html--3f334c530cf4.md#propdef-left) and ['right'](css2--visuren.html--3f334c530cf4.md#propdef-right) have the value 'auto', then if the 'direction' property of the element establishing the static-position containing block is 'ltr', set ['left'](css2--visuren.html--3f334c530cf4.md#propdef-left) to the static position; else if 'direction' is 'rtl', set ['right'](css2--visuren.html--3f334c530cf4.md#propdef-right) to the static position.
3.  If ['left'](css2--visuren.html--3f334c530cf4.md#propdef-left) or ['right'](css2--visuren.html--3f334c530cf4.md#propdef-right) are 'auto', replace any 'auto' on ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left) or ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right) with '0'.
4.  If at this point both ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left) and ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right) are still 'auto', solve the equation under the extra constraint that the two margins must get equal values, unless this would make them negative, in which case when the direction of the containing block is 'ltr' ('rtl'), set ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left) (['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right)) to zero and solve for ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right) (['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left)).
5.  If at this point there is an 'auto' left, solve the equation for that value.
6.  If at this point the values are over-constrained, ignore the value for either ['left'](css2--visuren.html--3f334c530cf4.md#propdef-left) (in case the ['direction'](css2--visuren.html--3f334c530cf4.md#propdef-direction) property of the containing block is 'rtl') or ['right'](css2--visuren.html--3f334c530cf4.md#propdef-right) (in case ['direction'](css2--visuren.html--3f334c530cf4.md#propdef-direction) is 'ltr') and solve for that value.

<a id="inlineblock-width"></a>

### 10.3.9 'Inline-block', non-replaced elements in normal flow

If ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) is 'auto', the used value is the [shrink-to-fit](#shrink-to-fit-float) width as for floating elements.

A computed value of 'auto' for ['margin-left'](css2--box.html--8875bbcdefe6.md#propdef-margin-left) or ['margin-right'](css2--box.html--8875bbcdefe6.md#propdef-margin-right) becomes a used value of '0'.

<a id="inlineblock-replaced-width"></a>

### 10.3.10 'Inline-block', replaced elements in normal flow

Exactly as [inline replaced elements.](#inline-replaced-width)

<a id="min-max-widths"></a>

## 10.4 Minimum and maximum widths: ['min-width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-width) and ['max-width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-width)

<a id="propdef-min-width"></a>

<strong>'min-width'</strong>

|                       |                                                                                                                                                                                                                                                                                        |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 0                                                                                                                                                                                                                                                                                      |
| <em>Applies to:</em>   | all elements but non-replaced inline elements, table rows, and row groups                                                                                                                                                                                                              |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                     |
| <em>Percentages:</em>   | refer to width of containing block                                                                                                                                                                                                                                                     |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                   |
| <em>Computed value:</em>   | the percentage as specified or the absolute length                                                                                                                                                                                                                                     |

<a id="propdef-max-width"></a>

<strong>'max-width'</strong>

|                       |                                                                                                                                                                                                                                                                                                |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| none \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | none                                                                                                                                                                                                                                                                                           |
| <em>Applies to:</em>   | all elements but non-replaced inline elements, table rows, and row groups                                                                                                                                                                                                                      |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                             |
| <em>Percentages:</em>   | refer to width of containing block                                                                                                                                                                                                                                                             |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                           |
| <em>Computed value:</em>   | the percentage as specified or the absolute length or 'none'                                                                                                                                                                                                                                   |

These two properties allow authors to constrain content widths to a certain range. Values have the following meanings:

<a id="x8"></a>

[<strong>&lt;length&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-length)

Specifies a fixed minimum or maximum used width.

<a id="x9"></a>

[<strong>&lt;percentage&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-percentage)

Specifies a percentage for determining the used value. The percentage is calculated with respect to the width of the generated box's [containing block](css2--visuren.html--3f334c530cf4.md#containing-block). If the containing block's width is negative, the used value is zero. If the containing block's width depends on this element's width, then the resulting layout is undefined in CSS 2.1.

<strong>none</strong>

(Only on ['max-width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-width)) No limit on the width of the box.

Negative values for ['min-width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-width) and ['max-width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-width) are illegal.

In CSS 2.1, the effect of 'min-width' and 'max-width' on tables, inline tables, table cells, table columns, and column groups is undefined.

The following algorithm describes how the two properties influence the [used value](css2--cascade.html--c7aff33e6f0d.md#computed-value) of the ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) property:

1.  The tentative used width is calculated (without ['min-width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-width) and ['max-width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-width)) following the rules under ["Calculating widths and margins"](#Computing_widths_and_margins) above.
2.  If the tentative used width is greater than ['max-width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-width), the rules [above](#Computing_widths_and_margins) are applied again, but this time using the computed value of ['max-width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-width) as the computed value for ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width).
3.  If the resulting width is smaller than ['min-width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-width), the rules [above](#Computing_widths_and_margins) are applied again, but this time using the value of ['min-width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-width) as the computed value for ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width).

> <strong data-conversion-semantic="note">Note</strong>
>
> These steps do not affect the real computed values of the above properties.

However, for replaced elements with an intrinsic ratio and both ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) and ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) specified as 'auto', the algorithm is as follows:

Select from the table the resolved height and width values for the appropriate constraint violation. Take the <var>max-width</var> and <var>max-height</var> as max(min, max) so that <var>min</var> ≤ <var>max</var> holds true. In this table <var>w</var> and <var>h</var> stand for the results of the width and height computations ignoring the ['min-width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-width), ['min-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-height), ['max-width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-width) and ['max-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-height) properties. Normally these are the intrinsic width and height, but they may not be in the case of replaced elements with intrinsic ratios.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: In cases where an explicit width or height is set and the other dimension is auto, applying a minimum or maximum constraint on the auto side can cause an over-constrained situation. The spec is clear in the behavior but it might not be what the author expects. The CSS3 object-fit property can be used to obtain different results in this situation.

| Constraint Violation                                                         | Resolved Width      | Resolved Height     |
|------------------------------------------------------------------------------|---------------------|---------------------|
| none                                                                         | <var>w</var> | <var>h</var> |
| <var>w &gt; max-width</var>                                                          | <var>max-width</var> | <var>max(max-width &#x2A; h/w, min-height)</var> |
| <var>w &lt; min-width</var>                                                          | <var>min-width</var> | <var>min(min-width &#x2A; h/w, max-height)</var> |
| <var>h &gt; max-height</var>                                                          | <var>max(max-height &#x2A; w/h, min-width)</var> | <var>max-height</var> |
| <var>h &lt; min-height</var>                                                          | <var>min(min-height &#x2A; w/h, max-width)</var> | <var>min-height</var> |
| (<var>w &gt; max-width</var>) and (<var>h &gt; max-height</var>), where (<var>max-width/w ≤ max-height/h</var>) | <var>max-width</var> | <var>max(min-height, max-width &#x2A; h/w)</var> |
| (<var>w &gt; max-width</var>) and (<var>h &gt; max-height</var>), where (<var>max-width/w &gt; max-height/h</var>) | <var>max(min-width, max-height &#x2A; w/h)</var> | <var>max-height</var> |
| (<var>w &lt; min-width</var>) and (<var>h &lt; min-height</var>), where (<var>min-width/w ≤ min-height/h</var>) | <var>min(max-width, min-height &#x2A; w/h)</var> | <var>min-height</var> |
| (<var>w &lt; min-width</var>) and (<var>h &lt; min-height</var>), where (<var>min-width/w &gt; min-height/h</var>) | <var>min-width</var> | <var>min(max-height, min-width &#x2A; h/w)</var> |
| (<var>w &lt; min-width</var>) and (<var>h &gt; max-height</var>)                              | <var>min-width</var> | <var>max-height</var> |
| (<var>w &gt; max-width</var>) and (<var>h &lt; min-height</var>)                              | <var>max-width</var> | <var>min-height</var> |

Then apply the rules under ["Calculating widths and margins"](#Computing_widths_and_margins) above, as if ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) were computed as this value.

<a id="the-height-property"></a>

## 10.5 Content height: the ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) property

<a id="propdef-height"></a>

<strong>'height'</strong>

|                       |                                                                                                                                                                                                                                                                                                |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| auto \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | auto                                                                                                                                                                                                                                                                                           |
| <em>Applies to:</em>   | all elements but non-replaced inline elements, table columns, and column groups                                                                                                                                                                                                                |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                             |
| <em>Percentages:</em>   | see prose                                                                                                                                                                                                                                                                                      |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                           |
| <em>Computed value:</em>   | the percentage or 'auto' (see prose under [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage)) or the absolute length                                                                                                                             |

This property specifies the [content height](css2--box.html--8875bbcdefe6.md#content-height) of boxes.

This property does not apply to non-replaced [inline](css2--visuren.html--3f334c530cf4.md#inline-boxes) elements. See the [section on computing heights and margins for non-replaced inline elements](#inline-non-replaced) for the rules used instead.

Values have the following meanings:

<a id="x11"></a>

[<strong>&lt;length&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-length)

Specifies the height of the content area using a length value.

<a id="x12"></a>

[<strong>&lt;percentage&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-percentage)

Specifies a percentage height. The percentage is calculated with respect to the height of the generated box's [containing block](css2--visuren.html--3f334c530cf4.md#containing-block). If the height of the containing block is not specified explicitly (i.e., it depends on content height), and this element is not absolutely positioned, the value computes to 'auto'. A percentage height on the [root element](css2--conform.html--de58593b67d7.md#root) is relative to the [initial containing block](#containing-block-details). <strong data-conversion-semantic="note">Note:</strong> Note: For absolutely positioned elements whose containing block is based on a block-level element, the percentage is calculated with respect to the height of the <em>padding box</em> of that element. This is a change from CSS1, where the percentage was always calculated with respect to the <em>content box</em> of the parent element.

<strong>auto</strong>

The height depends on the values of other properties. See the prose below.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note that the height of the containing block of an absolutely positioned element is independent of the size of the element itself, and thus a percentage height on such an element can always be resolved. However, it may be that the height is not known until elements that come later in the document have been processed.

Negative values for ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) are illegal.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> For example, the following rule sets the content height of paragraphs to 100 pixels:
>
> ```text
> 
> p { height: 100px }
> ```
>
> Paragraphs of which the height of the contents exceeds 100 pixels will [overflow](css2--visufx.html--2bc674cb7ab0.md#overflow) according to the ['overflow'](css2--visufx.html--2bc674cb7ab0.md#propdef-overflow) property.

<a id="Computing_heights_and_margins"></a>

## 10.6 Calculating heights and margins

For calculating the values of ['top'](css2--visuren.html--3f334c530cf4.md#propdef-top), ['margin-top'](css2--box.html--8875bbcdefe6.md#propdef-margin-top), ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height), ['margin-bottom'](css2--box.html--8875bbcdefe6.md#propdef-margin-bottom), and ['bottom'](css2--visuren.html--3f334c530cf4.md#propdef-bottom) a distinction must be made between various kinds of boxes:

1.  inline, non-replaced elements
2.  inline, replaced elements
3.  block-level, non-replaced elements in normal flow
4.  block-level, replaced elements in normal flow
5.  floating, non-replaced elements
6.  floating, replaced elements
7.  absolutely positioned, non-replaced elements
8.  absolutely positioned, replaced elements
9.  'inline-block', non-replaced elements in normal flow
10. 'inline-block', replaced elements in normal flow

For Points 1-6 and 9-10, the used values of 'top' and 'bottom' are determined by the rules in section 9.4.3.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: these rules apply to the root element just as to any other element.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> The used value of 'height'
calculated below is a tentative value, and may have to be calculated
multiple times, depending on <a href="css2--visudet.html--12e8bc0e6b7c.md#propdef-min-height"><span>'min-height'</span></a> and <a href="css2--visudet.html--12e8bc0e6b7c.md#propdef-max-height"><span>'max-height'</span></a>, see the section <a href="#min-max-heights">Minimum and maximum heights</a> below.</em>

<a id="inline-non-replaced"></a>

### 10.6.1 Inline, non-replaced elements

The ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) property does not apply. The height of the content area should be based on the font, but this specification does not specify how. A UA may, e.g., use the em-box or the maximum ascender and descender of the font. (The latter would ensure that glyphs with parts above or below the em-box still fall within the content area, but leads to differently sized boxes for different fonts; the former would ensure authors can control background styling relative to the 'line-height', but leads to glyphs painting outside their content area.)

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: level 3 of CSS will probably include a property to select which measure of the font is used for the content height.

The vertical padding, border and margin of an inline, non-replaced box start at the top and bottom of the content area, and has nothing to do with the ['line-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height). But only the ['line-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height) is used when calculating the height of the line box.

<a id="multi-font-inline-height"></a>

If more than one font is used (this could happen when glyphs are found in different fonts), the height of the content area is not defined by this specification. However, we suggest that the height is chosen such that the content area is just high enough for either (1) the em-boxes, or (2) the maximum ascenders and descenders, of <em>all</em> the fonts in the element. Note that this may be larger than any of the font sizes involved, depending on the baseline alignment of the fonts.

<a id="inline-replaced-height"></a>

### 10.6.2 Inline replaced elements, block-level replaced elements in normal flow, 'inline-block' replaced elements in normal flow and floating replaced elements

If ['margin-top'](css2--box.html--8875bbcdefe6.md#propdef-margin-top), or ['margin-bottom'](css2--box.html--8875bbcdefe6.md#propdef-margin-bottom) are 'auto', their used value is 0.

If ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) and ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) both have computed values of 'auto' and the element also has an intrinsic height, then that intrinsic height is the used value of ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height).

Otherwise, if ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) has a computed value of 'auto', and the element has an intrinsic ratio then the used value of ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) is:

> (used width) / (intrinsic ratio)

Otherwise, if ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) has a computed value of 'auto', and the element has an intrinsic height, then that intrinsic height is the used value of ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height).

Otherwise, if ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) has a computed value of 'auto', but none of the conditions above are met, then the used value of ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) must be set to the height of the largest rectangle that has a 2:1 ratio, has a height not greater than 150px, and has a width not greater than the device width.

<a id="normal-block"></a>

### 10.6.3 Block-level non-replaced elements in normal flow when 'overflow' computes to 'visible'

This section also applies to block-level non-replaced elements in normal flow when 'overflow' does not compute to 'visible' but has been propagated to the viewport.

If ['margin-top'](css2--box.html--8875bbcdefe6.md#propdef-margin-top), or ['margin-bottom'](css2--box.html--8875bbcdefe6.md#propdef-margin-bottom) are 'auto', their used value is 0. If ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) is 'auto', the height depends on whether the element has any block-level children and whether it has padding or borders:

The element's height is the distance from its top content edge to the first applicable of the following:

1.  the bottom edge of the last line box, if the box establishes a inline formatting context with one or more lines
2.  the bottom edge of the bottom (possibly collapsed) margin of its last in-flow child, if the child's bottom margin does not collapse with the element's bottom margin
3.  the bottom border edge of the last in-flow child whose top margin doesn't collapse with the element's bottom margin
4.  zero, otherwise

Only children in the normal flow are taken into account (i.e., floating boxes and absolutely positioned boxes are ignored, and relatively positioned boxes are considered without their offset). Note that the child box may be an [anonymous block box.](css2--visuren.html--3f334c530cf4.md#anonymous-block-level)

<a id="abs-non-replaced-height"></a>

### 10.6.4 Absolutely positioned, non-replaced elements

For the purposes of this section and the next, the term "static position" (of an element) refers, roughly, to the position an element would have had in the normal flow. More precisely, the static position for 'top' is the distance from the top edge of the containing block to the top margin edge of a hypothetical box that would have been the first box of the element if its specified ['position'](css2--visuren.html--3f334c530cf4.md#propdef-position) value had been 'static' and its specified ['float'](css2--visuren.html--3f334c530cf4.md#propdef-float) had been 'none' and its specified ['clear'](css2--visuren.html--3f334c530cf4.md#propdef-clear) had been 'none'. (Note that due to the rules in [section 9.7](css2--visuren.html--3f334c530cf4.md#dis-pos-flo) this might require also assuming a different computed value for 'display'.) The value is negative if the hypothetical box is above the containing block.

But rather than actually calculating the dimensions of that hypothetical box, user agents are free to make a guess at its probable position.

For the purposes of calculating the static position, the containing block of fixed positioned elements is the initial containing block instead of the viewport.

For absolutely positioned elements, the used values of the vertical dimensions must satisfy this constraint:

> 'top' + 'margin-top' + 'border-top-width' + 'padding-top' + 'height' + 'padding-bottom' + 'border-bottom-width' + 'margin-bottom' + 'bottom' = height of containing block

If all three of 'top', 'height', and 'bottom' are auto, set 'top' to the static position and apply rule number three below.

If none of the three are 'auto': If both 'margin-top' and 'margin-bottom' are 'auto', solve the equation under the extra constraint that the two margins get equal values. If one of 'margin-top' or 'margin-bottom' is 'auto', solve the equation for that value. If the values are over-constrained, ignore the value for 'bottom' and solve for that value.

Otherwise, pick the one of the following six rules that applies.

1.  'top' and 'height' are 'auto' and 'bottom' is not 'auto', then the height is [based on the content per 10.6.7](#root-height), set 'auto' values for 'margin-top' and 'margin-bottom' to 0, and solve for 'top'
2.  'top' and 'bottom' are 'auto' and 'height' is not 'auto', then set 'top' to the static position, set 'auto' values for 'margin-top' and 'margin-bottom' to 0, and solve for 'bottom'
3.  'height' and 'bottom' are 'auto' and 'top' is not 'auto', then the height is [based on the content per 10.6.7](#root-height), set 'auto' values for 'margin-top' and 'margin-bottom' to 0, and solve for 'bottom'
4.  'top' is 'auto', 'height' and 'bottom' are not 'auto', then set 'auto' values for 'margin-top' and 'margin-bottom' to 0, and solve for 'top'
5.  'height' is 'auto', 'top' and 'bottom' are not 'auto', then 'auto' values for 'margin-top' and 'margin-bottom' are set to 0 and solve for 'height'
6.  'bottom' is 'auto', 'top' and 'height' are not 'auto', then set 'auto' values for 'margin-top' and 'margin-bottom' to 0 and solve for 'bottom'

<a id="abs-replaced-height"></a>

### 10.6.5 Absolutely positioned, replaced elements

This situation is similar to the previous one, except that the element has an [intrinsic](css2--conform.html--de58593b67d7.md#intrinsic) height. The sequence of substitutions is now:

1.  The used value of ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) is determined as for [inline replaced elements](#inline-replaced-height). If 'margin-top' or 'margin-bottom' is specified as 'auto' its used value is determined by the rules below.
2.  If both ['top'](css2--visuren.html--3f334c530cf4.md#propdef-top) and ['bottom'](css2--visuren.html--3f334c530cf4.md#propdef-bottom) have the value 'auto', replace ['top'](css2--visuren.html--3f334c530cf4.md#propdef-top) with the element's [static position](#static-position).
3.  If ['bottom'](css2--visuren.html--3f334c530cf4.md#propdef-bottom) is 'auto', replace any 'auto' on ['margin-top'](css2--box.html--8875bbcdefe6.md#propdef-margin-top) or ['margin-bottom'](css2--box.html--8875bbcdefe6.md#propdef-margin-bottom) with '0'.
4.  If at this point both ['margin-top'](css2--box.html--8875bbcdefe6.md#propdef-margin-top) and ['margin-bottom'](css2--box.html--8875bbcdefe6.md#propdef-margin-bottom) are still 'auto', solve the equation under the extra constraint that the two margins must get equal values.
5.  If at this point there is only one 'auto' left, solve the equation for that value.
6.  If at this point the values are over-constrained, ignore the value for ['bottom'](css2--visuren.html--3f334c530cf4.md#propdef-bottom) and solve for that value.

<a id="block-root-margin"></a>

### 10.6.6 Complicated cases

This section applies to:

- Block-level, non-replaced elements in normal flow when 'overflow' does not compute to 'visible' (except if the 'overflow' property's value has been propagated to the viewport).
- 'Inline-block', non-replaced elements.
- Floating, non-replaced elements.

If ['margin-top'](css2--box.html--8875bbcdefe6.md#propdef-margin-top), or ['margin-bottom'](css2--box.html--8875bbcdefe6.md#propdef-margin-bottom) are 'auto', their used value is 0. If ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) is 'auto', the [height depends on the element's descendants per 10.6.7](#root-height).

For 'inline-block' elements, the margin box is used when calculating the height of the line box.

<a id="root-height"></a>

### 10.6.7 'Auto' heights for block formatting context roots

In certain cases (see, e.g., sections [10.6.4](#abs-non-replaced-height) and [10.6.6](#block-root-margin) above), the height of an element that establishes a block formatting context is computed as follows:

If it only has inline-level children, the height is the distance between the top of the topmost line box and the bottom of the bottommost line box.

If it has block-level children, the height is the distance between the top margin-edge of the topmost block-level child box and the bottom margin-edge of the bottommost block-level child box.

Absolutely positioned children are ignored, and relatively positioned boxes are considered without their offset. Note that the child box may be an [anonymous block box.](css2--visuren.html--3f334c530cf4.md#anonymous-block-level)

In addition, if the element has any floating descendants whose bottom margin edge is below the element's bottom content edge, then the height is increased to include those edges. Only floats that participate in this block formatting context are taken into account, e.g., floats inside absolutely positioned descendants or other floats are not.

<a id="min-max-heights"></a>

## 10.7 Minimum and maximum heights: ['min-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-height) and ['max-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-height)

It is sometimes useful to constrain the height of elements to a certain range. Two properties offer this functionality:

<a id="propdef-min-height"></a>

<strong>'min-height'</strong>

|                       |                                                                                                                                                                                                                                                                                        |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | 0                                                                                                                                                                                                                                                                                      |
| <em>Applies to:</em>   | all elements but non-replaced inline elements, table columns, and column groups                                                                                                                                                                                                        |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                     |
| <em>Percentages:</em>   | see prose                                                                                                                                                                                                                                                                              |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                   |
| <em>Computed value:</em>   | the percentage as specified or the absolute length                                                                                                                                                                                                                                     |

<a id="propdef-max-height"></a>

<strong>'max-height'</strong>

|                       |                                                                                                                                                                                                                                                                                                |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| none \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | none                                                                                                                                                                                                                                                                                           |
| <em>Applies to:</em>   | all elements but non-replaced inline elements, table columns, and column groups                                                                                                                                                                                                                |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                             |
| <em>Percentages:</em>   | see prose                                                                                                                                                                                                                                                                                      |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                           |
| <em>Computed value:</em>   | the percentage as specified or the absolute length or 'none'                                                                                                                                                                                                                                   |

These two properties allow authors to constrain box heights to a certain range. Values have the following meanings:

<a id="x15"></a>

[<strong>&lt;length&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-length)

Specifies a fixed minimum or maximum computed height.

<a id="x16"></a>

[<strong>&lt;percentage&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-percentage)

Specifies a percentage for determining the used value. The percentage is calculated with respect to the height of the generated box's [containing block](css2--visuren.html--3f334c530cf4.md#containing-block). If the height of the containing block is not specified explicitly (i.e., it depends on content height), and this element is not absolutely positioned, the percentage value is treated as '0' (for ['min-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-height)) or 'none' (for ['max-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-height)).

<strong>none</strong>

(Only on ['max-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-height)) No limit on the height of the box.

Negative values for ['min-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-height) and ['max-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-height) are illegal.

In CSS 2.1, the effect of 'min-height' and 'max-height' on tables, inline tables, table cells, table rows, and row groups is undefined.

The following algorithm describes how the two properties influence the [used value](css2--cascade.html--c7aff33e6f0d.md#computed-value) of the ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) property:

1.  The tentative used height is calculated (without ['min-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-height) and ['max-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-height)) following the rules under ["Calculating heights and margins"](#Computing_heights_and_margins) above.
2.  If this tentative height is greater than ['max-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-height), the rules [above](#Computing_heights_and_margins) are applied again, but this time using the value of ['max-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-height) as the computed value for ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height).
3.  If the resulting height is smaller than ['min-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-height), the rules [above](#Computing_heights_and_margins) are applied again, but this time using the value of ['min-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-height) as the computed value for ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height).

> <strong data-conversion-semantic="note">Note</strong>
>
> These steps do not affect the real computed values of the above properties. The change of used ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) has no effect on margin collapsing except as specifically required by rules for ['min-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-min-height) or ['max-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-max-height) in ["Collapsing margins" (8.3.1).](css2--box.html--8875bbcdefe6.md#collapsing-margins)

However, for replaced elements with both ['width'](css2--visudet.html--12e8bc0e6b7c.md#propdef-width) and ['height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-height) computed as 'auto', use the algorithm under [Minimum and maximum widths](#min-max-widths) above to find the used width and height. Then apply the rules under ["Computing heights and margins"](#Computing_heights_and_margins) above, using the resulting width and height as if they were the computed values.

<a id="line-height"></a>

## 10.8 Line height calculations: the ['line-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height) and ['vertical-align'](css2--visudet.html--12e8bc0e6b7c.md#propdef-vertical-align) properties

As described in the section on [inline formatting contexts](css2--visuren.html--3f334c530cf4.md#inline-formatting), user agents flow inline-level boxes into a vertical stack of [line boxes](css2--visuren.html--3f334c530cf4.md#line-box). The height of a line box is determined as follows:

1.  The height of each inline-level box in the line box is calculated. For replaced elements, inline-block elements, and inline-table elements, this is the height of their margin box; for inline boxes, this is their 'line-height'. (See ["Calculating heights and margins"](#Computing_heights_and_margins) and the [height of inline boxes](#inline-box-height) in ["Leading and half-leading"](#leading).)
2.  The inline-level boxes are aligned vertically according to their ['vertical-align'](css2--visudet.html--12e8bc0e6b7c.md#propdef-vertical-align) property. In case they are aligned 'top' or 'bottom', they must be aligned so as to minimize the line box height. If such boxes are tall enough, there are multiple solutions and CSS 2.1 does not define the position of the line box's baseline (i.e., the position of the [strut, see below](#strut)).
3.  The line box height is the distance between the uppermost box top and the lowermost box bottom. (This includes the [strut,](#strut) as explained under ['line-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height) below.)

Empty inline elements generate empty inline boxes, but these boxes still have margins, padding, borders and a line height, and thus influence these calculations just like elements with content.

<a id="leading"></a>

### 10.8.1 Leading and half-leading

CSS assumes that every font has font metrics that specify a characteristic height above the baseline and a depth below it. In this section we use <var>A</var> to mean that height (for a given font at a given size) and <var>D</var> the depth. We also define <var>AD</var> = <var>A</var> + <var>D</var>, the distance from the top to the bottom. (See the note below for [how to find <var>A</var> and <var>D</var> for TrueType and OpenType fonts.](#sTypoAscender)) Note that these are metrics of the font as a whole and need not correspond to the ascender and descender of any individual glyph.

User agent must align the glyphs in a non-replaced inline box to each other by their relevant baselines. Then, for each glyph, determine the <var>A</var> and <var>D</var>. Note that glyphs in a single element may come from different fonts and thus need not all have the same <var>A</var> and <var>D</var>. If the inline box contains no glyphs at all, it is considered to contain a [strut](#strut) (an invisible glyph of zero width) with the <var>A</var> and <var>D</var> of the element's first available font.

Still for each glyph, determine the leading <var>L</var> to add, where <var>L</var> = ['line-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height) - <var>AD</var>. Half the leading is added above <var>A</var> and the other half below <var>D</var>, giving the glyph and its leading a total height above the baseline of <var>A'</var> = <var>A</var> + <var>L</var>/2 and a total depth of <var>D'</var> = <var>D</var> + <var>L</var>/2.

> <strong data-conversion-semantic="note">Note</strong>
>
> <strong>Note.</strong> <var>L</var> may be negative.

<a id="inline-box-height"></a>The height of the inline box encloses all glyphs and their half-leading on each side and is thus exactly 'line-height'. Boxes of child elements do not influence this height.

Although margins, borders, and padding of non-replaced elements do not enter into the line box calculation, they are still rendered around inline boxes. This means that if the height specified by ['line-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height) is less than the content height of contained boxes, backgrounds and colors of padding and borders may "bleed" into adjoining line boxes. User agents should render the boxes in document order. This will cause the borders on subsequent lines to paint over the borders and text of previous lines.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> CSS 2.1 does not define
what the content area of an inline box is (see <a href="#inline-non-replaced">10.6.1</a> above) and thus different UAs
may draw the backgrounds and borders in different places.</em>

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="sTypoAscender"></a><em><strong>Note.</strong> It is
recommended that implementations that use OpenType or TrueType fonts
use the metrics "sTypoAscender" and "sTypoDescender" from the font's
OS/2 table for A and D (after scaling to the current element's font
size). In the absence of these metrics, the "Ascent" and "Descent"
metrics from the HHEA table should be used.</em>

<a id="propdef-line-height"></a>

<strong>'line-height'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                                                              |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | normal \| [\<number\>](css2--syndata.html--02e71c159e14.md#value-def-number) \| [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | normal                                                                                                                                                                                                                                                                                                                                                                                       |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                                                                                                                 |
| <em>Inherited:</em>   | yes                                                                                                                                                                                                                                                                                                                                                                                          |
| <em>Percentages:</em>   | refer to the font size of the element itself                                                                                                                                                                                                                                                                                                                                                 |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                                                                                                                         |
| <em>Computed value:</em>   | for [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) and [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) the absolute value; otherwise as specified                                                                                                                                                 |

<a id="strut"></a>

On a [block container element](css2--visuren.html--3f334c530cf4.md#block-boxes) whose content is composed of [inline-level](css2--visuren.html--3f334c530cf4.md#inline-level) elements, 'line-height' specifies the <em>minimal</em> height of line boxes within the element. The minimum height consists of a minimum height above the baseline and a minimum depth below it, exactly as if each line box starts with a zero-width inline box with the element's font and line height properties. We call that imaginary box a "strut." (The name is inspired by TeX.).

The height and depth of the font above and below the baseline are assumed to be metrics that are contained in the font. (For more details, see CSS level 3.)

On a non-replaced [inline](css2--visuren.html--3f334c530cf4.md#inline-boxes) element, 'line-height' specifies the height that is used in the calculation of the line box height.

Values for this property have the following meanings:

<strong>normal</strong>

<a id="x18"></a>

Tells user agents to set the used value to a "reasonable" value based on the font of the element. The value has the same meaning as [\<number\>](css2--syndata.html--02e71c159e14.md#value-def-number). We recommend a used value for 'normal' between 1.0 to 1.2. The [computed value](css2--cascade.html--c7aff33e6f0d.md#computed-value) is 'normal'.

<a id="x19"></a>

[<strong>&lt;length&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-length)

The specified length is used in the calculation of the line box height. Negative values are illegal.

<a id="x20"></a>

[<strong>&lt;number&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-number)

The used value of the property is this number multiplied by the element's font size. Negative values are illegal. The [computed value](css2--cascade.html--c7aff33e6f0d.md#computed-value) is the same as the specified value.

<a id="x21"></a>

[<strong>&lt;percentage&gt;</strong>](css2--syndata.html--02e71c159e14.md#value-def-percentage)

The [computed value](css2--cascade.html--c7aff33e6f0d.md#computed-value) of the property is this percentage multiplied by the element's computed font size. Negative values are illegal.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> The three rules in the example below have the same resultant line height:
>
> ```text
> 
> div { line-height: 1.2; font-size: 10pt }     /* number */
> div { line-height: 1.2em; font-size: 10pt }   /* length */
> div { line-height: 120%; font-size: 10pt }    /* percentage */
> ```
When an element contains text that is rendered in more than one font, user agents may determine the 'normal' ['line-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height) value according to the largest font size.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> When there is only one value of <a href="css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height"><span>'line-height'</span></a> for all inline
boxes in a block container box and they are all in the same font (and
there are no replaced elements, inline-block
elements, etc.), the above will ensure that
baselines of successive lines are exactly <a href="css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height"><span>'line-height'</span></a> apart. This is
important when columns of text in different fonts have to be aligned,
for example in a table.</em>

<a id="propdef-vertical-align"></a>

<strong>'vertical-align'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                                          |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | baseline \| sub \| super \| top \| text-top \| middle \| bottom \| text-bottom \| [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) \| [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | baseline                                                                                                                                                                                                                                                                                                                                                                 |
| <em>Applies to:</em>   | inline-level and 'table-cell' elements                                                                                                                                                                                                                                                                                                                                   |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                                                                                                       |
| <em>Percentages:</em>   | refer to the 'line-height' of the element itself                                                                                                                                                                                                                                                                                                                         |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group)                                                                                                                                                                                                                                                                                     |
| <em>Computed value:</em>   | for [\<percentage\>](css2--syndata.html--02e71c159e14.md#value-def-percentage) and [\<length\>](css2--syndata.html--02e71c159e14.md#value-def-length) the absolute length, otherwise as specified                                                                                                                            |

This property affects the vertical positioning inside a line box of the boxes generated by an inline-level element.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> Values of this property have
different meanings in the context of tables.
Please consult the section on <a href="css2--tables.html--201812dd6e3c.md#height-layout">
table height algorithms</a> for details.
</em>

The following values only have meaning with respect to a parent inline element, or to the [strut](#strut) of a parent block container element.

In the following definitions, for inline non-replaced elements, the box used for alignment is the box whose height is the 'line-height' (containing the box's glyphs and the half-leading on each side, see [above](#inline-box-height)). For all other elements, the box used for alignment is the margin box.

<strong>baseline</strong>

Align the baseline of the box with the baseline of the parent box. If the box does not have a baseline, align the bottom margin edge with the parent's baseline.

<strong>middle</strong>

Align the vertical midpoint of the box with the baseline of the parent box plus half the x-height of the parent.

<strong>sub</strong>

Lower the baseline of the box to the proper position for subscripts of the parent's box. (This value has no effect on the font size of the element's text.)

<strong>super</strong>

Raise the baseline of the box to the proper position for superscripts of the parent's box. (This value has no effect on the font size of the element's text.)

<strong>text-top</strong>

Align the top of the box with the top of the parent's content area (see [10.6.1](#inline-non-replaced)).

<strong>text-bottom</strong>

Align the bottom of the box with the bottom of the parent's content area (see [10.6.1](#inline-non-replaced)).

<a id="x23"></a>

[<strong>&lt;percentage&gt;
  </strong>](css2--syndata.html--02e71c159e14.md#value-def-percentage)

Raise (positive value) or lower (negative value) the box by this distance (a percentage of the ['line-height'](css2--visudet.html--12e8bc0e6b7c.md#propdef-line-height) value). The value '0%' means the same as 'baseline'.

<a id="x24"></a>

[<strong>&lt;length&gt;
  </strong>](css2--syndata.html--02e71c159e14.md#value-def-length)

Raise (positive value) or lower (negative value) the box by this distance. The value '0cm' means the same as 'baseline'.

The following values align the element relative to the line box. Since the element may have children aligned relative to it (which in turn may have descendants aligned relative to them), these values use the bounds of the aligned subtree. The aligned subtree of an inline element contains that element and the aligned subtrees of all children inline elements whose computed 'vertical-align' value is not 'top' or 'bottom'. The top of the aligned subtree is the highest of the tops of the boxes in the subtree, and the bottom is analogous.

<strong>top</strong>  
Align the top of the aligned subtree with the top of the line box.

<strong>bottom</strong>  
Align the bottom of the aligned subtree with the bottom of the line box.

The baseline of an 'inline-table' is the baseline of the first row of the table.

The baseline of an 'inline-block' is the baseline of its last line box in the normal flow, unless it has either no in-flow line boxes or if its 'overflow' property has a computed value other than 'visible', in which case the baseline is the bottom margin edge.
