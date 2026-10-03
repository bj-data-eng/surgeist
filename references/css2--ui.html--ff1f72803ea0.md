Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [User interface](https://www.w3.org/TR/2011/REC-CSS2-20110607/ui.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: User interface

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/ui.html

Snapshot SHA-256: ff1f72803ea09e98b983dc7e380a2c903d4d91fb1de017375dc9d7002ff2613d

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.

---

<a id="q18.0"></a>

# 18 User interface

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

<a id="cursor-props"></a>

## 18.1 Cursors: the ['cursor'](css2--ui.html--ff1f72803ea0.md#propdef-cursor) property

<a id="propdef-cursor"></a>

<strong>'cursor'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                                                          |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | \[ \[[\<uri\>](css2--syndata.html--02e71c159e14.md#value-def-uri) ,\]\* \[ auto \| crosshair \| default \| pointer \| move \| e-resize \| ne-resize \| nw-resize \| n-resize \| se-resize \| sw-resize \| s-resize \| w-resize \| text \| wait \| help \| progress \] \] \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | auto                                                                                                                                                                                                                                                                                                                                                                                     |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                                                                                                             |
| <em>Inherited:</em>   | yes                                                                                                                                                                                                                                                                                                                                                                                      |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                                                                                                                                                                                                      |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group), [interactive](css2--media.html--3a324a170379.md#interactive-media-group)                                                                                                                                                                                                     |
| <em>Computed value:</em>   | as specified, except with any relative URLs converted to absolute                                                                                                                                                                                                                                                                                                                        |

This property specifies the type of cursor to be displayed for the pointing device. Values have the following meanings:

auto

The UA determines the cursor to display based on the current context.

crosshair

A simple crosshair (e.g., short line segments resembling a "+" sign).

default

The platform-dependent default cursor. Often rendered as an arrow.

pointer

The cursor is a pointer that indicates a link.

move

Indicates something is to be moved.

e-resize, ne-resize, nw-resize, n-resize, se-resize, sw-resize, s-resize, w-resize

Indicate that some edge is to be moved. For example, the 'se-resize' cursor is used when the movement starts from the south-east corner of the box.

text

Indicates text that may be selected. Often rendered as an I-beam.

wait

Indicates that the program is busy and the user should wait. Often rendered as a watch or hourglass.

progress

A progress indicator. The program is performing some processing, but is different from 'wait' in that the user may still interact with the program. Often rendered as a spinning beach ball, or an arrow with a watch or hourglass.

help

Help is available for the object under the cursor. Often rendered as a question mark or a balloon.

<a id="x1"></a>

[\<uri\>](css2--syndata.html--02e71c159e14.md#value-def-uri)

The user agent retrieves the cursor from the resource designated by the URI. If the user agent cannot handle the first cursor of a list of cursors, it should attempt to handle the second, etc. If the user agent cannot handle any user-defined cursor, it must use the generic cursor at the end of the list. Intrinsic sizes for cursors are calculated as for [background images,](css2--colors.html--5784063d2778.md#background-properties) except that a UA-defined rectangle is used in place of the rectangle that establishes the coordinate system for the 'background-image' property. This UA-defined rectangle should be based on the size of a typical cursor on the UA's operating system. If the resulting cursor size does not fit within this rectangle, the UA may proportionally scale the resulting cursor down until it fits within the rectangle.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> ```text
> 
> :link,:visited { cursor: url(example.svg#linkcursor), url(hyper.cur), pointer }
> ```
>
> This example sets the cursor on all hyperlinks (whether visited or not) to an external [SVG cursor](https://www.w3.org/TR/SVG/interact.html#CursorElement). User agents that do not support SVG cursors would simply skip to the next value and attempt to use the "hyper.cur" cursor. If that cursor format was also not supported, the UA would skip to the next value and simply render the 'pointer' cursor.

<a id="system-colors"></a>

## 18.2 System Colors

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> 
The System Colors are deprecated in the CSS3 Color Module <a href="css2--refs.html--f208d881d0b7.md#ref-CSS3COLOR"><span>&#x5B;CSS3COLOR&#x5D;</span></a>.
</em>

In addition to being able to assign pre-defined [color values](css2--syndata.html--02e71c159e14.md#color-units) to text, backgrounds, etc., CSS2 introduced a set of named color values that allows authors to specify colors in a manner that integrates them into the operating system's graphic environment.

For systems that do not have a corresponding value, the specified value should be mapped to the nearest system value, or to a default color.

The following lists additional values for color-related CSS properties and their general meaning. Any color property (e.g., ['color'](css2--colors.html--5784063d2778.md#propdef-color) or ['background-color'](css2--colors.html--5784063d2778.md#propdef-background-color)) can take one of the following names. Although these are case-insensitive, it is recommended that the mixed capitalization shown below be used, to make the names more legible.

ActiveBorder  
Active window border.

ActiveCaption  
Active window caption.

AppWorkspace  
Background color of multiple document interface.

Background  
Desktop background.

ButtonFace  
Face color for three-dimensional display elements.

ButtonHighlight  
Highlight color for three-dimensional display elements (for edges facing away from the light source).

ButtonShadow  
Shadow color for three-dimensional display elements.

ButtonText  
Text on push buttons.

CaptionText  
Text in caption, size box, and scrollbar arrow box.

GrayText  
Grayed (disabled) text. This color is set to \#000 if the current display driver does not support a solid gray color.

Highlight  
Item(s) selected in a control.

HighlightText  
Text of item(s) selected in a control.

InactiveBorder  
Inactive window border.

InactiveCaption  
Inactive window caption.

InactiveCaptionText  
Color of text in an inactive caption.

InfoBackground  
Background color for tooltip controls.

InfoText  
Text color for tooltip controls.

Menu  
Menu background.

MenuText  
Text in menus.

Scrollbar  
Scroll bar gray area.

ThreeDDarkShadow  
Dark shadow for three-dimensional display elements.

ThreeDFace  
Face color for three-dimensional display elements.

ThreeDHighlight  
Highlight color for three-dimensional display elements.

ThreeDLightShadow  
Light color for three-dimensional display elements (for edges facing the light source).

ThreeDShadow  
Dark shadow for three-dimensional display elements.

Window  
Window background.

WindowFrame  
Window frame.

WindowText  
Text in windows.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> For example, to set the foreground and background colors of a paragraph to the same foreground and background colors of the user's window, write the following:
>
> ```text
> 
> p { color: WindowText; background-color: Window }
> ```
<a id="system-fonts"></a>

## 18.3 User preferences for fonts

As for colors, authors may specify fonts in a way that makes use of a user's system resources. Please consult the ['font'](css2--fonts.html--d52fc14f36c2.md#propdef-font) property for details.

<a id="dynamic-outlines"></a>

<a id="x2"></a>

## 18.4 Dynamic outlines: the 'outline' property

At times, style sheet authors may want to create outlines around visual objects such as buttons, active form fields, image maps, etc., to make them stand out. CSS 2.1 outlines differ from [borders](css2--box.html--8875bbcdefe6.md#border-properties) in the following ways:

1.  Outlines do not take up space.
2.  Outlines may be non-rectangular.

The outline properties control the style of these dynamic outlines.

<a id="propdef-outline"></a>

<strong>'outline'</strong>

|                       |                                                                                                                                                                                                                                                                                                                                                                                                                 |
|-----------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | \[ [\<'outline-color'\>](css2--ui.html--ff1f72803ea0.md#propdef-outline-color) \|\| [\<'outline-style'\>](css2--ui.html--ff1f72803ea0.md#propdef-outline-style) \|\| [\<'outline-width'\>](css2--ui.html--ff1f72803ea0.md#propdef-outline-width) \] \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | see individual properties                                                                                                                                                                                                                                                                                                                                                                                       |
| <em>Applies to:</em>   | all elements                                                                                                                                                                                                                                                                                                                                                                                                    |
| <em>Inherited:</em>   | no                                                                                                                                                                                                                                                                                                                                                                                                              |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                                                                                                                                                                                                                                             |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group), [interactive](css2--media.html--3a324a170379.md#interactive-media-group)                                                                                                                                                                                                                            |
| <em>Computed value:</em>   | see individual properties                                                                                                                                                                                                                                                                                                                                                                                       |

<a id="propdef-outline-width"></a>

<strong>'outline-width'</strong>

|                       |                                                                                                                                                                                            |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<border-width\>](css2--box.html--8875bbcdefe6.md#value-def-border-width) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | medium                                                                                                                                                                                     |
| <em>Applies to:</em>   | all elements                                                                                                                                                                               |
| <em>Inherited:</em>   | no                                                                                                                                                                                         |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                        |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group), [interactive](css2--media.html--3a324a170379.md#interactive-media-group)       |
| <em>Computed value:</em>   | absolute length; '0' if the outline style is 'none'                                                                                                                                        |

<a id="propdef-outline-style"></a>

<strong>'outline-style'</strong>

|                       |                                                                                                                                                                                            |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<border-style\>](css2--box.html--8875bbcdefe6.md#value-def-border-style) \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | none                                                                                                                                                                                       |
| <em>Applies to:</em>   | all elements                                                                                                                                                                               |
| <em>Inherited:</em>   | no                                                                                                                                                                                         |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                        |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group), [interactive](css2--media.html--3a324a170379.md#interactive-media-group)       |
| <em>Computed value:</em>   | as specified                                                                                                                                                                               |

<a id="propdef-outline-color"></a>

<strong>'outline-color'</strong>

|                       |                                                                                                                                                                                            |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <em>Value:</em>   | [\<color\>](css2--syndata.html--02e71c159e14.md#value-def-color) \| invert \| [inherit](css2--cascade.html--c7aff33e6f0d.md#value-def-inherit) |
| <em>Initial:</em>   | invert                                                                                                                                                                                     |
| <em>Applies to:</em>   | all elements                                                                                                                                                                               |
| <em>Inherited:</em>   | no                                                                                                                                                                                         |
| <em>Percentages:</em>   | N/A                                                                                                                                                                                        |
| <em>Media:</em>   | [visual](css2--media.html--3a324a170379.md#visual-media-group), [interactive](css2--media.html--3a324a170379.md#interactive-media-group)       |
| <em>Computed value:</em>   | as specified                                                                                                                                                                               |

The outline created with the outline properties is drawn "over" a box, i.e., the outline is always on top, and does not influence the position or size of the box, or of any other boxes. Therefore, displaying or suppressing outlines does not cause reflow or overflow.

The outline may be drawn starting just outside the [border edge](css2--box.html--8875bbcdefe6.md#border-edge).

Outlines may be non-rectangular. For example, if the element is broken across several lines, the outline is the minimum outline that encloses all the element's boxes. In contrast to [borders](css2--box.html--8875bbcdefe6.md#border-properties), the outline is not open at the line box's end or start, but is always fully connected if possible.

The ['outline-width'](css2--ui.html--ff1f72803ea0.md#propdef-outline-width) property accepts the same values as ['border-width'](css2--box.html--8875bbcdefe6.md#propdef-border-width).

The ['outline-style'](css2--ui.html--ff1f72803ea0.md#propdef-outline-style) property accepts the same values as ['border-style'](css2--box.html--8875bbcdefe6.md#propdef-border-style), except that 'hidden' is not a legal outline style.

<a id="value-def-invert"></a>

The ['outline-color'](css2--ui.html--ff1f72803ea0.md#propdef-outline-color) accepts all colors, as well as the keyword 'invert'. 'Invert' is expected to perform a color inversion on the pixels on the screen. This is a common trick to ensure the focus border is visible, regardless of color background.

Conformant UAs may ignore the 'invert' value on platforms that do not support color inversion of the pixels on the screen. If the UA does not support the 'invert' value then the initial value of the 'outline-color' property is the value of the 'color' property, similar to the initial value of the 'border-top-color' property.

The ['outline'](css2--ui.html--ff1f72803ea0.md#propdef-outline) property is a shorthand property, and sets all three of ['outline-style'](css2--ui.html--ff1f72803ea0.md#propdef-outline-style), ['outline-width'](css2--ui.html--ff1f72803ea0.md#propdef-outline-width), and ['outline-color'](css2--ui.html--ff1f72803ea0.md#propdef-outline-color).

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong> 
The outline is the same on all sides. In
contrast to borders, there is no 'outline-top' or 'outline-left'
property.
</em>

This specification does not define how multiple overlapping outlines are drawn, or how outlines are drawn for boxes that are partially obscured behind other elements.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong>
Since the outline does not affect formatting (i.e., no
space is left for it in the box model), it may well overlap
other elements on the page.
</em>

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> Here's an example of drawing a thick outline around a BUTTON element:
>
> ```text
> 
> button { outline : thick solid}
> ```
>
> Scripts may be used to dynamically change the width of the outline, without provoking a reflow.

<a id="outline-focus"></a>

### 18.4.1 Outlines and the focus

<a id="x8"></a>

Graphical user interfaces may use outlines around elements to tell the user which element on the page has the focus. These outlines are in addition to any borders, and switching outlines on and off should not cause the document to reflow. The focus is the subject of user interaction in a document (e.g., for entering text, selecting a button, etc.). User agents supporting the [interactive media group](css2--media.html--3a324a170379.md#interactive-media-group) must keep track of where the focus lies and must also represent the focus. This may be done by using dynamic outlines in conjunction with the :focus pseudo-class.

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> For example, to draw a thick black line around an element when it has the focus, and a thick red line when it is active, the following rules can be used:
>
> ```text
> 
> :focus  { outline: thick solid black }
> :active { outline: thick solid red }
> ```
<a id="magnification"></a>

## 18.5 Magnification

The CSS working group considers that the magnification of a document or portions of a document should not be specified through style sheets. User agents may support such magnification in different ways (e.g., larger images, louder sounds, etc.)

When magnifying a page, UAs should preserve the relationships between positioned elements. For example, a comic strip may be composed of images with overlaid text elements. When magnifying this page, a user agent should keep the text within the comic strip balloon.
