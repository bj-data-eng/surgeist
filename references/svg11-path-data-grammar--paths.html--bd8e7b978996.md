Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Paths – SVG 1.1 (Second Edition)](https://www.w3.org/TR/2011/REC-SVG11-20110816/paths.html).

Original copyright notice (from the SVG 1.1 Second Edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Paths – SVG 1.1 (Second Edition)

Source snapshot: https://www.w3.org/TR/2011/REC-SVG11-20110816/paths.html

Snapshot SHA-256: bd8e7b9789962497a6b2ca2b8012a4f824905a89b40beaa3b29cc9eabf5d6fc7

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 10 source tables are presented as readable Markdown tables or explicit labeled layouts: 10 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# 8 Paths

## <a id="Introduction"></a>8.1 Introduction

Paths represent the outline of a shape which can be filled, stroked, used as a clipping path, or any combination of the three. (See [Filling, Stroking and Paint Servers](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html) and [Clipping, Masking and Compositing](https://www.w3.org/TR/2011/REC-SVG11-20110816/masking.html).)

A path is described using the concept of a current point. In an analogy with drawing on paper, the current point can be thought of as the location of the pen. The position of the pen can be changed, and the outline of a shape (open or closed) can be traced by dragging the pen in either straight lines or curves.

Paths represent the geometry of the outline of an object, defined in terms of <em>moveto</em> (set a new current point), <em>lineto</em> (draw a straight line), <em>curveto</em> (draw a curve using a cubic Bézier), <em>arc</em> (elliptical or circular arc) and <em>closepath</em> (close the current shape by drawing a line to the last <em>moveto</em>) elements. Compound paths (i.e., a path with multiple subpaths) are possible to allow effects such as "donut holes" in objects.

This chapter describes the syntax, behavior and DOM interfaces for SVG paths. Various implementation notes for SVG paths can be found in [‘path’ element implementation notes](svg11-path-parsing-interpretation--implnote.html--8c6edf7d7d52.md#PathElementImplementationNotes) and [Elliptical arc implementation notes](svg11-path-parsing-interpretation--implnote.html--8c6edf7d7d52.md#ArcImplementationNotes).

A path is defined in SVG using the [‘path’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathElement) element.

## <a id="PathElement"></a>8.2 The ‘path’ element

‘path’

Categories:  
[Graphics element](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermGraphicsElement), [shape element](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermShapeElement)

Content model:  
Any number of the following elements, in any order:

- [animation elements](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermAnimationElement) — [‘animate’](https://www.w3.org/TR/2011/REC-SVG11-20110816/animate.html#AnimateElement), [‘animateColor’](https://www.w3.org/TR/2011/REC-SVG11-20110816/animate.html#AnimateColorElement), [‘animateMotion’](https://www.w3.org/TR/2011/REC-SVG11-20110816/animate.html#AnimateMotionElement), [‘animateTransform’](https://www.w3.org/TR/2011/REC-SVG11-20110816/animate.html#AnimateTransformElement), [‘set’](https://www.w3.org/TR/2011/REC-SVG11-20110816/animate.html#SetElement)
- [descriptive elements](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermDescriptiveElement) — [‘desc’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#DescElement), [‘metadata’](https://www.w3.org/TR/2011/REC-SVG11-20110816/metadata.html#MetadataElement), [‘title’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#TitleElement)

Attributes:  
- [conditional processing attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermConditionalProcessingAttribute) — [‘requiredFeatures’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#RequiredFeaturesAttribute), [‘requiredExtensions’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#RequiredExtensionsAttribute), [‘systemLanguage’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#SystemLanguageAttribute)
- [core attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermCoreAttributes) — [‘id’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#IDAttribute), [‘xml:base’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLBaseAttribute), [‘xml:lang’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLLangAttribute), [‘xml:space’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#XMLSpaceAttribute)
- [graphical event attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermGraphicalEventAttribute) — [‘onfocusin’](https://www.w3.org/TR/2011/REC-SVG11-20110816/script.html#OnFocusInEventAttribute), [‘onfocusout’](https://www.w3.org/TR/2011/REC-SVG11-20110816/script.html#OnFocusOutEventAttribute), [‘onactivate’](https://www.w3.org/TR/2011/REC-SVG11-20110816/script.html#OnActivateEventAttribute), [‘onclick’](https://www.w3.org/TR/2011/REC-SVG11-20110816/script.html#OnClickEventAttribute), [‘onmousedown’](https://www.w3.org/TR/2011/REC-SVG11-20110816/script.html#OnMouseDownEventAttribute), [‘onmouseup’](https://www.w3.org/TR/2011/REC-SVG11-20110816/script.html#OnMouseUpEventAttribute), [‘onmouseover’](https://www.w3.org/TR/2011/REC-SVG11-20110816/script.html#OnMouseOverEventAttribute), [‘onmousemove’](https://www.w3.org/TR/2011/REC-SVG11-20110816/script.html#OnMouseMoveEventAttribute), [‘onmouseout’](https://www.w3.org/TR/2011/REC-SVG11-20110816/script.html#OnMouseOutEventAttribute), [‘onload’](https://www.w3.org/TR/2011/REC-SVG11-20110816/script.html#OnLoadEventAttribute)
- [presentation attributes](https://www.w3.org/TR/2011/REC-SVG11-20110816/intro.html#TermPresentationAttribute) — [‘alignment-baseline’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#AlignmentBaselineProperty), [‘baseline-shift’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#BaselineShiftProperty), [‘clip’](https://www.w3.org/TR/2011/REC-SVG11-20110816/masking.html#ClipProperty), [‘clip-path’](https://www.w3.org/TR/2011/REC-SVG11-20110816/masking.html#ClipPathProperty), [‘clip-rule’](https://www.w3.org/TR/2011/REC-SVG11-20110816/masking.html#ClipRuleProperty), [‘color’](https://www.w3.org/TR/2011/REC-SVG11-20110816/color.html#ColorProperty), [‘color-interpolation’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#ColorInterpolationProperty), [‘color-interpolation-filters’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#ColorInterpolationFiltersProperty), [‘color-profile’](https://www.w3.org/TR/2011/REC-SVG11-20110816/color.html#ColorProfileProperty), [‘color-rendering’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#ColorRenderingProperty), [‘cursor’](https://www.w3.org/TR/2011/REC-SVG11-20110816/interact.html#CursorProperty), [‘direction’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#DirectionProperty), [‘display’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#DisplayProperty), [‘dominant-baseline’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#DominantBaselineProperty), [‘enable-background’](https://www.w3.org/TR/2011/REC-SVG11-20110816/filters.html#EnableBackgroundProperty), [‘fill’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#FillProperty), [‘fill-opacity’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#FillOpacityProperty), [‘fill-rule’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#FillRuleProperty), [‘filter’](https://www.w3.org/TR/2011/REC-SVG11-20110816/filters.html#FilterProperty), [‘flood-color’](https://www.w3.org/TR/2011/REC-SVG11-20110816/filters.html#FloodColorProperty), [‘flood-opacity’](https://www.w3.org/TR/2011/REC-SVG11-20110816/filters.html#FloodOpacityProperty), [‘font-family’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#FontFamilyProperty), [‘font-size’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#FontSizeProperty), [‘font-size-adjust’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#FontSizeAdjustProperty), [‘font-stretch’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#FontStretchProperty), [‘font-style’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#FontStyleProperty), [‘font-variant’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#FontVariantProperty), [‘font-weight’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#FontWeightProperty), [‘glyph-orientation-horizontal’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#GlyphOrientationHorizontalProperty), [‘glyph-orientation-vertical’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#GlyphOrientationVerticalProperty), [‘image-rendering’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#ImageRenderingProperty), [‘kerning’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#KerningProperty), [‘letter-spacing’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#LetterSpacingProperty), [‘lighting-color’](https://www.w3.org/TR/2011/REC-SVG11-20110816/filters.html#LightingColorProperty), [‘marker-end’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#MarkerEndProperty), [‘marker-mid’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#MarkerMidProperty), [‘marker-start’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#MarkerStartProperty), [‘mask’](https://www.w3.org/TR/2011/REC-SVG11-20110816/masking.html#MaskProperty), [‘opacity’](https://www.w3.org/TR/2011/REC-SVG11-20110816/masking.html#OpacityProperty), [‘overflow’](https://www.w3.org/TR/2011/REC-SVG11-20110816/masking.html#OverflowProperty), [‘pointer-events’](https://www.w3.org/TR/2011/REC-SVG11-20110816/interact.html#PointerEventsProperty), [‘shape-rendering’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#ShapeRenderingProperty), [‘stop-color’](https://www.w3.org/TR/2011/REC-SVG11-20110816/pservers.html#StopColorProperty), [‘stop-opacity’](https://www.w3.org/TR/2011/REC-SVG11-20110816/pservers.html#StopOpacityProperty), [‘stroke’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeProperty), [‘stroke-dasharray’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeDasharrayProperty), [‘stroke-dashoffset’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeDashoffsetProperty), [‘stroke-linecap’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeLinecapProperty), [‘stroke-linejoin’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeLinejoinProperty), [‘stroke-miterlimit’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeMiterlimitProperty), [‘stroke-opacity’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeOpacityProperty), [‘stroke-width’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeWidthProperty), [‘text-anchor’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#TextAnchorProperty), [‘text-decoration’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#TextDecorationProperty), [‘text-rendering’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#TextRenderingProperty), [‘unicode-bidi’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#UnicodeBidiProperty), [‘visibility’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#VisibilityProperty), [‘word-spacing’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#WordSpacingProperty), [‘writing-mode’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#WritingModeProperty)
- [‘class’](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#ClassAttribute)
- [‘style’](https://www.w3.org/TR/2011/REC-SVG11-20110816/styling.html#StyleAttribute)
- [‘externalResourcesRequired’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#ExternalResourcesRequiredAttribute)
- [‘transform’](https://www.w3.org/TR/2011/REC-SVG11-20110816/coords.html#TransformAttribute)
- [‘d’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#DAttribute)
- [‘pathLength’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathLengthAttribute)

DOM Interfaces:  
- [SVGPathElement](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathElement)

<em>Attribute definitions:</em>

<a id="DAttribute"></a>d = "<em>path data</em>"  
The definition of the outline of a shape. See [Path data](#PathData).  
[Animatable](https://www.w3.org/TR/2011/REC-SVG11-20110816/animate.html#Animatable): yes. Path data animation is only possible when each path data specification within an animation specification has exactly the same list of path data commands as the [‘d’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#DAttribute) attribute. If an animation is specified and the list of path data commands is not the same, then the animation specification is in error (see [Error Processing](svg11-path-parsing-interpretation--implnote.html--8c6edf7d7d52.md#ErrorProcessing)). The animation engine interpolates each parameter to each path data command separately based on the attributes to the given animation element. Flags and booleans are interpolated as fractions between zero and one, with any non-zero value considered to be a value of one/true.

<a id="PathLengthAttribute"></a>pathLength = "[\<number\>](https://www.w3.org/TR/2011/REC-SVG11-20110816/types.html#DataTypeNumber)"  
The author's computation of the total length of the path, in user units. This value is used to calibrate the user agent's own [distance-along-a-path](svg11-path-data-grammar--paths.html--bd8e7b978996.md#DistanceAlongAPath) calculations with that of the author. The user agent will scale all distance-along-a-path computations by the ratio of [‘pathLength’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathLengthAttribute) to the user agent's own computed value for total path length. [‘pathLength’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathLengthAttribute) potentially affects calculations for [text on a path](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#TextOnAPath), [motion animation](https://www.w3.org/TR/2011/REC-SVG11-20110816/animate.html#AnimateMotionElement) and various [stroke operations](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeProperties).  
A negative value is an error (see [Error processing](svg11-path-parsing-interpretation--implnote.html--8c6edf7d7d52.md#ErrorProcessing)).  
[Animatable](https://www.w3.org/TR/2011/REC-SVG11-20110816/animate.html#Animatable): yes.

## <a id="PathData"></a>8.3 Path data

### <a id="PathDataGeneralInformation"></a>8.3.1 General information about path data

A path is defined by including a [‘path’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathElement) element which contains a d="(path data)" attribute, where the [‘d’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#DAttribute) attribute contains the <em>moveto</em>, <em>line</em>, <em>curve</em> (both cubic and quadratic Béziers), <em>arc</em> and <em>closepath</em> instructions.

Example triangle01 specifies a path in the shape of a triangle. (The <strong>M</strong> indicates a <em>moveto</em>, the <strong>L</strong>s indicate <em>lineto</em>s, and the <strong>z</strong> indicates a <em>closepath</em>).

```text
<?xml version="1.0" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" 
  "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg width="4cm" height="4cm" viewBox="0 0 400 400"
     xmlns="http://www.w3.org/2000/svg" version="1.1">
  <title>Example triangle01- simple example of a 'path'</title>
  <desc>A path that draws a triangle</desc>
  <rect x="1" y="1" width="398" height="398"
        fill="none" stroke="blue" />
  <path d="M 100 100 L 300 100 L 200 300 z"
        fill="red" stroke="blue" stroke-width="3" />
</svg>
```

| Column 1                                                                                                                      |
|-------------------------------------------------------------------------------------------------------------------------------|
| ![Example triangle01 — simple example of a 'path'](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/paths/triangle01.png) |

Example triangle01

[View this example as SVG (SVG-enabled browsers only)](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/paths/triangle01.svg)

Path data can contain newline characters and thus can be broken up into multiple lines to improve readability. Because of line length limitations with certain related tools, it is recommended that SVG generators split long path data strings across multiple lines, with each line not exceeding 255 characters. Also note that newline characters are only allowed at certain places within path data.

The syntax of path data is concise in order to allow for minimal file size and efficient downloads, since many SVG files will be dominated by their path data. Some of the ways that SVG attempts to minimize the size of path data are as follows:

- All instructions are expressed as one character (e.g., a <em>moveto</em> is expressed as an <strong>M</strong>).
- Superfluous white space and separators such as commas can be eliminated (e.g., "M 100 100 L 200 200" contains unnecessary spaces and could be expressed more compactly as "M100 100L200 200").
- The command letter can be eliminated on subsequent commands if the same command is used multiple times in a row (e.g., you can drop the second "L" in "M 100 200 L 200 100 L -100 -200" and use "M 100 200 L 200 100 -100 -200" instead).
- Relative versions of all commands are available (uppercase means absolute coordinates, lowercase means relative coordinates).
- Alternate forms of <em>lineto</em> are available to optimize the special cases of horizontal and vertical lines (absolute and relative).
- Alternate forms of <em>curve</em> are available to optimize the special cases where some of the control points on the current segment can be determined automatically from the control points on the previous segment.

The path data syntax is a prefix notation (i.e., commands followed by parameters). The only allowable decimal point is a Unicode U+0046 FULL STOP (".") character (also referred to in Unicode as PERIOD, dot and decimal point) and no other delimiter characters are allowed \[[UNICODE](https://www.w3.org/TR/2011/REC-SVG11-20110816/refs.html#ref-UNICODE)\]. (For example, the following is an invalid numeric value in a path data stream: "13,000.56". Instead, say: "13000.56".)

For the relative versions of the commands, all coordinate values are relative to the current point at the start of the command.

In the tables below, the following notation is used:

- (): grouping of parameters
- +: 1 or more of the given parameter(s) is required

The following sections list the commands.

### <a id="PathDataMovetoCommands"></a>8.3.2 The <strong>"moveto"</strong> commands

The "moveto" commands (<strong>M</strong> or <strong>m</strong>) establish a new current point. The effect is as if the "pen" were lifted and moved to a new location. A path data segment (if there is one) must begin with a "moveto" command. Subsequent "moveto" commands (i.e., when the "moveto" is not the first command) represent the start of a new <em>subpath</em>:

| Command                                                                        | Name   | Parameters | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
|--------------------------------------------------------------------------------|--------|------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>M</strong> (absolute)<br> <strong>m</strong> (relative) | moveto | (x y)+     | Start a new sub-path at the given (x,y) coordinate. <strong>M</strong> (uppercase) indicates that absolute coordinates will follow; <strong>m</strong> (lowercase) indicates that relative coordinates will follow. If a moveto is followed by multiple pairs of coordinates, the subsequent pairs are treated as implicit lineto commands. Hence, implicit lineto commands will be relative if the moveto is relative, and absolute if the moveto is absolute. If a relative moveto (<strong>m</strong>) appears as the first element of the path, then it is treated as a pair of absolute coordinates. In this case, subsequent pairs of coordinates are treated as relative even though the initial moveto is interpreted as an absolute moveto. |

### <a id="PathDataClosePathCommand"></a>8.3.3 The <strong>"closepath"</strong> command

The "closepath" (<strong>Z</strong> or <strong>z</strong>) ends the current subpath and causes an automatic straight line to be drawn from the current point to the initial point of the current subpath. If a "closepath" is followed immediately by a "moveto", then the "moveto" identifies the start point of the next subpath. If a "closepath" is followed immediately by any other command, then the next subpath starts at the same initial point as the current subpath.

When a subpath ends in a "closepath," it differs in behavior from what happens when "manually" closing a subpath via a "lineto" command in how [‘stroke-linejoin’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeLinejoinProperty) and [‘stroke-linecap’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeLinecapProperty) are implemented. With "closepath", the end of the final segment of the subpath is "joined" with the start of the initial segment of the subpath using the current value of [‘stroke-linejoin’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeLinejoinProperty). If you instead "manually" close the subpath via a "lineto" command, the start of the first segment and the end of the last segment are not joined but instead are each capped using the current value of [‘stroke-linecap’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeLinecapProperty). At the end of the command, the new current point is set to the initial point of the current subpath.

| Command                                                     | Name      | Parameters | Description                                                                                                                                                                                   |
|-------------------------------------------------------------|-----------|------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Z</strong> or<br> <strong>z</strong> | closepath | (none)     | Close the current subpath by drawing a straight line from the current point to current subpath's initial point. Since the Z and z commands take no parameters, they have an identical effect. |

### <a id="PathDataLinetoCommands"></a>8.3.4 The <strong>"lineto"</strong> commands

The various "lineto" commands draw straight lines from the current point to a new point:

| Command                                                                        | Name              | Parameters | Description                                                                                                                                                                                                                                                                                                                                                                                                                                  |
|--------------------------------------------------------------------------------|-------------------|------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>L</strong> (absolute)<br> <strong>l</strong> (relative) | lineto            | (x y)+     | Draw a line from the current point to the given (x,y) coordinate which becomes the new current point. <strong>L</strong> (uppercase) indicates that absolute coordinates will follow; <strong>l</strong> (lowercase) indicates that relative coordinates will follow. A number of coordinates pairs may be specified to draw a polyline. At the end of the command, the new current point is set to the final set of coordinates provided. |
| <strong>H</strong> (absolute)<br> <strong>h</strong> (relative) | horizontal lineto | x+         | Draws a horizontal line from the current point (cpx, cpy) to (x, cpy). <strong>H</strong> (uppercase) indicates that absolute coordinates will follow; <strong>h</strong> (lowercase) indicates that relative coordinates will follow. Multiple x values can be provided (although usually this doesn't make sense). At the end of the command, the new current point becomes (x, cpy) for the final value of x.                           |
| <strong>V</strong> (absolute)<br> <strong>v</strong> (relative) | vertical lineto   | y+         | Draws a vertical line from the current point (cpx, cpy) to (cpx, y). <strong>V</strong> (uppercase) indicates that absolute coordinates will follow; <strong>v</strong> (lowercase) indicates that relative coordinates will follow. Multiple y values can be provided (although usually this doesn't make sense). At the end of the command, the new current point becomes (cpx, y) for the final value of y.                             |

### <a id="PathDataCurveCommands"></a>8.3.5 The curve commands

These three groups of commands draw curves:

- [Cubic Bézier commands](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathDataCubicBezierCommands) (<strong>C</strong>, <strong>c</strong>, <strong>S</strong> and <strong>s</strong>). A cubic Bézier segment is defined by a start point, an end point, and two control points.
- [Quadratic Bézier commands](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathDataQuadraticBezierCommands) (<strong>Q</strong>, <strong>q</strong>, <strong>T</strong> and <strong>t</strong>). A quadratic Bézier segment is defined by a start point, an end point, and one control point.
- [Elliptical arc commands](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathDataEllipticalArcCommands) (<strong>A</strong> and <strong>a</strong>). An elliptical arc segment draws a segment of an ellipse.

### <a id="PathDataCubicBezierCommands"></a>8.3.6 The cubic Bézier curve commands

The cubic Bézier commands are as follows:

| Command                                                                        | Name                     | Parameters         | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
|--------------------------------------------------------------------------------|--------------------------|--------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>C</strong> (absolute)<br> <strong>c</strong> (relative) | curveto                  | (x1 y1 x2 y2 x y)+ | Draws a cubic Bézier curve from the current point to (x,y) using (x1,y1) as the control point at the beginning of the curve and (x2,y2) as the control point at the end of the curve. <strong>C</strong> (uppercase) indicates that absolute coordinates will follow; <strong>c</strong> (lowercase) indicates that relative coordinates will follow. Multiple sets of coordinates may be specified to draw a polybézier. At the end of the command, the new current point becomes the final (x,y) coordinate pair used in the polybézier.                                                                                                                                                                                                                                                                    |
| <strong>S</strong> (absolute)<br> <strong>s</strong> (relative) | shorthand/smooth curveto | (x2 y2 x y)+       | Draws a cubic Bézier curve from the current point to (x,y). The first control point is assumed to be the reflection of the second control point on the previous command relative to the current point. (If there is no previous command or if the previous command was not an C, c, S or s, assume the first control point is coincident with the current point.) (x2,y2) is the second control point (i.e., the control point at the end of the curve). <strong>S</strong> (uppercase) indicates that absolute coordinates will follow; <strong>s</strong> (lowercase) indicates that relative coordinates will follow. Multiple sets of coordinates may be specified to draw a polybézier. At the end of the command, the new current point becomes the final (x,y) coordinate pair used in the polybézier. |

Example cubic01 shows some simple uses of cubic Bézier commands within a path. The example uses an internal CSS style sheet to assign styling properties. Note that the control point for the "S" command is computed automatically as the reflection of the control point for the previous "C" command relative to the start point of the "S" command.

```text
<?xml version="1.0" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" 
  "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg width="5cm" height="4cm" viewBox="0 0 500 400"
     xmlns="http://www.w3.org/2000/svg" version="1.1">
  <title>Example cubic01- cubic Bézier commands in path data</title>
  <desc>Picture showing a simple example of path data
        using both a "C" and an "S" command,
        along with annotations showing the control points
        and end points</desc>
  <style type="text/css"><![CDATA[
    .Border { fill:none; stroke:blue; stroke-width:1 }
    .Connect { fill:none; stroke:#888888; stroke-width:2 }
    .SamplePath { fill:none; stroke:red; stroke-width:5 }
    .EndPoint { fill:none; stroke:#888888; stroke-width:2 }
    .CtlPoint { fill:#888888; stroke:none }
    .AutoCtlPoint { fill:none; stroke:blue; stroke-width:4 }
    .Label { font-size:22; font-family:Verdana }
  ]]></style>

  <rect class="Border" x="1" y="1" width="498" height="398" />

  <polyline class="Connect" points="100,200 100,100" />
  <polyline class="Connect" points="250,100 250,200" />
  <polyline class="Connect" points="250,200 250,300" />
  <polyline class="Connect" points="400,300 400,200" />
  <path class="SamplePath" d="M100,200 C100,100 250,100 250,200
                                       S400,300 400,200" />
  <circle class="EndPoint" cx="100" cy="200" r="10" />
  <circle class="EndPoint" cx="250" cy="200" r="10" />
  <circle class="EndPoint" cx="400" cy="200" r="10" />
  <circle class="CtlPoint" cx="100" cy="100" r="10" />
  <circle class="CtlPoint" cx="250" cy="100" r="10" />
  <circle class="CtlPoint" cx="400" cy="300" r="10" />
  <circle class="AutoCtlPoint" cx="250" cy="300" r="9" />
  <text class="Label" x="25" y="70">M100,200 C100,100 250,100 250,200</text>
  <text class="Label" x="325" y="350"
        style="text-anchor:middle">S400,300 400,200</text>
</svg>
```

| Column 1                                                                                                                        |
|---------------------------------------------------------------------------------------------------------------------------------|
| ![Example cubic01 — cubic Bézier comamnds in path data](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/paths/cubic01.png) |

Example cubic01

[View this example as SVG (SVG-enabled browsers only)](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/paths/cubic01.svg)

The following picture shows some how cubic Bézier curves change their shape depending on the position of the control points. The first five examples illustrate a single cubic Bézier path segment. The example at the lower right shows a "C" command followed by an "S" command.

![Example cubic02 - cubic Bézier commands in path data](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/paths/cubic02.png)

[View this example as SVG (SVG-enabled browsers only)](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/paths/cubic02.svg)  
 

### <a id="PathDataQuadraticBezierCommands"></a>8.3.7 The quadratic Bézier curve commands

The quadratic Bézier commands are as follows:

| Command                                                                        | Name                                      | Parameters   | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
|--------------------------------------------------------------------------------|-------------------------------------------|--------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Q</strong> (absolute)<br> <strong>q</strong> (relative) | quadratic Bézier curveto                  | (x1 y1 x y)+ | Draws a quadratic Bézier curve from the current point to (x,y) using (x1,y1) as the control point. <strong>Q</strong> (uppercase) indicates that absolute coordinates will follow; <strong>q</strong> (lowercase) indicates that relative coordinates will follow. Multiple sets of coordinates may be specified to draw a polybézier. At the end of the command, the new current point becomes the final (x,y) coordinate pair used in the polybézier.                                                                                                                                                                            |
| <strong>T</strong> (absolute)<br> <strong>t</strong> (relative) | Shorthand/smooth quadratic Bézier curveto | (x y)+       | Draws a quadratic Bézier curve from the current point to (x,y). The control point is assumed to be the reflection of the control point on the previous command relative to the current point. (If there is no previous command or if the previous command was not a Q, q, T or t, assume the control point is coincident with the current point.) <strong>T</strong> (uppercase) indicates that absolute coordinates will follow; <strong>t</strong> (lowercase) indicates that relative coordinates will follow. At the end of the command, the new current point becomes the final (x,y) coordinate pair used in the polybézier. |

Example quad01 shows some simple uses of quadratic Bézier commands within a path. Note that the control point for the "T" command is computed automatically as the reflection of the control point for the previous "Q" command relative to the start point of the "T" command.

```text
<?xml version="1.0" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" 
  "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg width="12cm" height="6cm" viewBox="0 0 1200 600"
     xmlns="http://www.w3.org/2000/svg" version="1.1">
  <title>Example quad01 - quadratic Bézier commands in path data</title>
  <desc>Picture showing a "Q" a "T" command,
        along with annotations showing the control points
        and end points</desc>
  <rect x="1" y="1" width="1198" height="598"
        fill="none" stroke="blue" stroke-width="1" />

  <path d="M200,300 Q400,50 600,300 T1000,300"
        fill="none" stroke="red" stroke-width="5"  />
  <!-- End points -->
  <g fill="black" >
    <circle cx="200" cy="300" r="10"/>
    <circle cx="600" cy="300" r="10"/>
    <circle cx="1000" cy="300" r="10"/>
  </g>
  <!-- Control points and lines from end points to control points -->
  <g fill="#888888" >
    <circle cx="400" cy="50" r="10"/>
    <circle cx="800" cy="550" r="10"/>
  </g>
  <path d="M200,300 L400,50 L600,300 
           L800,550 L1000,300"
        fill="none" stroke="#888888" stroke-width="2" />
</svg>
```

| Column 1                                                                                                                          |
|-----------------------------------------------------------------------------------------------------------------------------------|
| ![Example quad01 — quadratic Bézier commands in path data](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/paths/quad01.png) |

Example quad01

[View this example as SVG (SVG-enabled browsers only)](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/paths/quad01.svg)

### <a id="PathDataEllipticalArcCommands"></a>8.3.8 The elliptical arc curve commands

The elliptical arc commands are as follows:

| Command                                                                        | Name           | Parameters                                             | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
|--------------------------------------------------------------------------------|----------------|--------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>A</strong> (absolute)<br> <strong>a</strong> (relative) | elliptical arc | (rx ry x-axis-rotation large-arc-flag sweep-flag x y)+ | Draws an elliptical arc from the current point to (<strong>x</strong>, <strong>y</strong>). The size and orientation of the ellipse are defined by two radii (<strong>rx</strong>, <strong>ry</strong>) and an <strong>x-axis-rotation</strong>, which indicates how the ellipse as a whole is rotated relative to the current coordinate system. The center (<strong>cx</strong>, <strong>cy</strong>) of the ellipse is calculated automatically to satisfy the constraints imposed by the other parameters. <strong>large-arc-flag</strong> and <strong>sweep-flag</strong> contribute to the automatic calculations and help determine how the arc is drawn. |

Example arcs01 shows some simple uses of arc commands within a path.

```text
<?xml version="1.0" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" 
  "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg width="12cm" height="5.25cm" viewBox="0 0 1200 400"
     xmlns="http://www.w3.org/2000/svg" version="1.1">
  <title>Example arcs01 - arc commands in path data</title>
  <desc>Picture of a pie chart with two pie wedges and
        a picture of a line with arc blips</desc>
  <rect x="1" y="1" width="1198" height="398"
        fill="none" stroke="blue" stroke-width="1" />

  <path d="M300,200 h-150 a150,150 0 1,0 150,-150 z"
        fill="red" stroke="blue" stroke-width="5" />
  <path d="M275,175 v-150 a150,150 0 0,0 -150,150 z"
        fill="yellow" stroke="blue" stroke-width="5" />

  <path d="M600,350 l 50,-25 
           a25,25 -30 0,1 50,-25 l 50,-25 
           a25,50 -30 0,1 50,-25 l 50,-25 
           a25,75 -30 0,1 50,-25 l 50,-25 
           a25,100 -30 0,1 50,-25 l 50,-25"
        fill="none" stroke="red" stroke-width="5"  />
</svg>
```

| Column 1                                                                                                             |
|----------------------------------------------------------------------------------------------------------------------|
| ![Example arcs01 — arc commands in path data](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/paths/arcs01.png) |

Example arcs01

[View this example as SVG (SVG-enabled browsers only)](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/paths/arcs01.svg)

The elliptical arc command draws a section of an ellipse which meets the following constraints:

- the arc starts at the current point
- the arc ends at point (<strong>x</strong>, <strong>y</strong>)
- the ellipse has the two radii (<strong>rx</strong>, <strong>ry</strong>)
- the x-axis of the ellipse is rotated by <strong>x-axis-rotation</strong> relative to the x-axis of the current coordinate system.

For most situations, there are actually four different arcs (two different ellipses, each with two different arc sweeps) that satisfy these constraints. <strong>large-arc-flag</strong> and <strong>sweep-flag</strong> indicate which one of the four arcs are drawn, as follows:

- Of the four candidate arc sweeps, two will represent an arc sweep of greater than or equal to 180 degrees (the "large-arc"), and two will represent an arc sweep of less than or equal to 180 degrees (the "small-arc"). If <strong>large-arc-flag</strong> is '1', then one of the two larger arc sweeps will be chosen; otherwise, if <strong>large-arc-flag</strong> is '0', one of the smaller arc sweeps will be chosen,
- If <strong>sweep-flag</strong> is '1', then the arc will be drawn in a "positive-angle" direction (i.e., the ellipse formula x=<strong>cx</strong>+<strong>rx</strong>\*cos(theta) and y=<strong>cy</strong>+<strong>ry</strong>\*sin(theta) is evaluated such that theta starts at an angle corresponding to the current point and increases positively until the arc reaches (x,y)). A value of 0 causes the arc to be drawn in a "negative-angle" direction (i.e., theta starts at an angle value corresponding to the current point and decreases until the arc reaches (x,y)).

The following illustrates the four combinations of <strong>large-arc-flag</strong> and <strong>sweep-flag</strong> and the four different arcs that will be drawn based on the values of these flags. For each case, the following path data command was used:

```text

<path d="M 125,75 a100,50 0 ?,? 100,50"
      style="fill:none; stroke:red; stroke-width:6"/>
```
where "?,?" is replaced by "0,0" "0,1" "1,0" and "1,1" to generate the four possible cases.

![Illustration of flags in arc commands](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/paths/arcs02.png)

[View this example as SVG (SVG-enabled browsers only)](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/paths/arcs02.svg)

Refer to [Elliptical arc implementation notes](svg11-path-parsing-interpretation--implnote.html--8c6edf7d7d52.md#ArcImplementationNotes) for detailed implementation notes for the path data elliptical arc commands.

### <a id="PathDataBNF"></a>8.3.9 The grammar for path data

The following notation is used in the Backus-Naur Form (BNF) description of the grammar for path data:

- \*: 0 or more
- +: 1 or more
- ?: 0 or 1
- (): grouping
- \|: separates alternatives
- double quotes surround literals

The following is the BNF for SVG paths.

```text

svg-path:
    wsp* moveto-drawto-command-groups? wsp*
moveto-drawto-command-groups:
    moveto-drawto-command-group
    | moveto-drawto-command-group wsp* moveto-drawto-command-groups
moveto-drawto-command-group:
    moveto wsp* drawto-commands?
drawto-commands:
    drawto-command
    | drawto-command wsp* drawto-commands
drawto-command:
    closepath
    | lineto
    | horizontal-lineto
    | vertical-lineto
    | curveto
    | smooth-curveto
    | quadratic-bezier-curveto
    | smooth-quadratic-bezier-curveto
    | elliptical-arc
moveto:
    ( "M" | "m" ) wsp* moveto-argument-sequence
moveto-argument-sequence:
    coordinate-pair
    | coordinate-pair comma-wsp? lineto-argument-sequence
closepath:
    ("Z" | "z")
lineto:
    ( "L" | "l" ) wsp* lineto-argument-sequence
lineto-argument-sequence:
    coordinate-pair
    | coordinate-pair comma-wsp? lineto-argument-sequence
horizontal-lineto:
    ( "H" | "h" ) wsp* horizontal-lineto-argument-sequence
horizontal-lineto-argument-sequence:
    coordinate
    | coordinate comma-wsp? horizontal-lineto-argument-sequence
vertical-lineto:
    ( "V" | "v" ) wsp* vertical-lineto-argument-sequence
vertical-lineto-argument-sequence:
    coordinate
    | coordinate comma-wsp? vertical-lineto-argument-sequence
curveto:
    ( "C" | "c" ) wsp* curveto-argument-sequence
curveto-argument-sequence:
    curveto-argument
    | curveto-argument comma-wsp? curveto-argument-sequence
curveto-argument:
    coordinate-pair comma-wsp? coordinate-pair comma-wsp? coordinate-pair
smooth-curveto:
    ( "S" | "s" ) wsp* smooth-curveto-argument-sequence
smooth-curveto-argument-sequence:
    smooth-curveto-argument
    | smooth-curveto-argument comma-wsp? smooth-curveto-argument-sequence
smooth-curveto-argument:
    coordinate-pair comma-wsp? coordinate-pair
quadratic-bezier-curveto:
    ( "Q" | "q" ) wsp* quadratic-bezier-curveto-argument-sequence
quadratic-bezier-curveto-argument-sequence:
    quadratic-bezier-curveto-argument
    | quadratic-bezier-curveto-argument comma-wsp? 
        quadratic-bezier-curveto-argument-sequence
quadratic-bezier-curveto-argument:
    coordinate-pair comma-wsp? coordinate-pair
smooth-quadratic-bezier-curveto:
    ( "T" | "t" ) wsp* smooth-quadratic-bezier-curveto-argument-sequence
smooth-quadratic-bezier-curveto-argument-sequence:
    coordinate-pair
    | coordinate-pair comma-wsp? smooth-quadratic-bezier-curveto-argument-sequence
elliptical-arc:
    ( "A" | "a" ) wsp* elliptical-arc-argument-sequence
elliptical-arc-argument-sequence:
    elliptical-arc-argument
    | elliptical-arc-argument comma-wsp? elliptical-arc-argument-sequence
elliptical-arc-argument:
    nonnegative-number comma-wsp? nonnegative-number comma-wsp? 
        number comma-wsp flag comma-wsp? flag comma-wsp? coordinate-pair
coordinate-pair:
    coordinate comma-wsp? coordinate
coordinate:
    number
nonnegative-number:
    integer-constant
    | floating-point-constant
number:
    sign? integer-constant
    | sign? floating-point-constant
flag:
    "0" | "1"
comma-wsp:
    (wsp+ comma? wsp*) | (comma wsp*)
comma:
    ","
integer-constant:
    digit-sequence
floating-point-constant:
    fractional-constant exponent?
    | digit-sequence exponent
fractional-constant:
    digit-sequence? "." digit-sequence
    | digit-sequence "."
exponent:
    ( "e" | "E" ) sign? digit-sequence
sign:
    "+" | "-"
digit-sequence:
    digit
    | digit digit-sequence
digit:
    "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
wsp:
    (#x20 | #x9 | #xD | #xA)
```
The processing of the BNF must consume as much of a given BNF production as possible, stopping at the point when a character is encountered which no longer satisfies the production. Thus, in the string "M 100-200", the first coordinate for the "moveto" consumes the characters "100" and stops upon encountering the minus sign because the minus sign cannot follow a digit in the production of a "coordinate". The result is that the first coordinate will be "100" and the second coordinate will be "-200".

Similarly, for the string "M 0.6.5", the first coordinate of the "moveto" consumes the characters "0.6" and stops upon encountering the second decimal point because the production of a "coordinate" only allows one decimal point. The result is that the first coordinate will be "0.6" and the second coordinate will be ".5".

Note that the BNF allows the path [‘d’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#DAttribute) attribute to be empty. This is not an error, instead it disables rendering of the path.

## <a id="DistanceAlongAPath"></a>8.4 Distance along a path

Various operations, including [text on a path](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#TextOnAPath) and [motion animation](https://www.w3.org/TR/2011/REC-SVG11-20110816/animate.html#AnimateMotionElement) and various [stroke operations](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeProperties), require that the user agent compute the distance along the geometry of a graphics element, such as a [‘path’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathElement).

Exact mathematics exist for computing distance along a path, but the formulas are highly complex and require substantial computation. It is recommended that authoring products and user agents employ algorithms that produce as precise results as possible; however, to accommodate implementation differences and to help distance calculations produce results that approximate author intent, the [‘pathLength’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathLengthAttribute) attribute can be used to provide the author's computation of the total length of the path so that the user agent can scale distance-along-a-path computations by the ratio of [‘pathLength’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathLengthAttribute) to the user agent's own computed value for total path length.

A "moveto" operation within a [‘path’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathElement) element is defined to have zero length. Only the various "lineto", "curveto" and "arcto" commands contribute to path length calculations.

## <a id="DOMInterfaces"></a>8.5 DOM interfaces

### <a id="InterfaceSVGPathSeg"></a>8.5.1 Interface SVGPathSeg

The [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) interface is a base interface that corresponds to a single command within a path data specification.

```text
interface SVGPathSeg {

  // Path Segment Types
  const unsigned short PATHSEG_UNKNOWN = 0;
  const unsigned short PATHSEG_CLOSEPATH = 1;
  const unsigned short PATHSEG_MOVETO_ABS = 2;
  const unsigned short PATHSEG_MOVETO_REL = 3;
  const unsigned short PATHSEG_LINETO_ABS = 4;
  const unsigned short PATHSEG_LINETO_REL = 5;
  const unsigned short PATHSEG_CURVETO_CUBIC_ABS = 6;
  const unsigned short PATHSEG_CURVETO_CUBIC_REL = 7;
  const unsigned short PATHSEG_CURVETO_QUADRATIC_ABS = 8;
  const unsigned short PATHSEG_CURVETO_QUADRATIC_REL = 9;
  const unsigned short PATHSEG_ARC_ABS = 10;
  const unsigned short PATHSEG_ARC_REL = 11;
  const unsigned short PATHSEG_LINETO_HORIZONTAL_ABS = 12;
  const unsigned short PATHSEG_LINETO_HORIZONTAL_REL = 13;
  const unsigned short PATHSEG_LINETO_VERTICAL_ABS = 14;
  const unsigned short PATHSEG_LINETO_VERTICAL_REL = 15;
  const unsigned short PATHSEG_CURVETO_CUBIC_SMOOTH_ABS = 16;
  const unsigned short PATHSEG_CURVETO_CUBIC_SMOOTH_REL = 17;
  const unsigned short PATHSEG_CURVETO_QUADRATIC_SMOOTH_ABS = 18;
  const unsigned short PATHSEG_CURVETO_QUADRATIC_SMOOTH_REL = 19;

  readonly attribute unsigned short pathSegType;
  readonly attribute DOMString pathSegTypeAsLetter;
};
```
Constants in group “Path Segment Types”:  
<a id="__svg__SVGPathSeg__PATHSEG_UNKNOWN"></a><b>PATHSEG&#x5F;UNKNOWN</b> (unsigned short)  
The unit type is not one of predefined types. It is invalid to attempt to define a new value of this type or to attempt to switch an existing value to this type.

<a id="__svg__SVGPathSeg__PATHSEG_CLOSEPATH"></a><b>PATHSEG&#x5F;CLOSEPATH</b> (unsigned short)  
Corresponds to a "closepath" (z) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_MOVETO_ABS"></a><b>PATHSEG&#x5F;MOVETO&#x5F;ABS</b> (unsigned short)  
Corresponds to a "absolute moveto" (M) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_MOVETO_REL"></a><b>PATHSEG&#x5F;MOVETO&#x5F;REL</b> (unsigned short)  
Corresponds to a "relative moveto" (m) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_LINETO_ABS"></a><b>PATHSEG&#x5F;LINETO&#x5F;ABS</b> (unsigned short)  
Corresponds to a "absolute lineto" (L) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_LINETO_REL"></a><b>PATHSEG&#x5F;LINETO&#x5F;REL</b> (unsigned short)  
Corresponds to a "relative lineto" (l) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_CURVETO_CUBIC_ABS"></a><b>PATHSEG&#x5F;CURVETO&#x5F;CUBIC&#x5F;ABS</b> (unsigned short)  
Corresponds to a "absolute cubic Bézier curveto" (C) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_CURVETO_CUBIC_REL"></a><b>PATHSEG&#x5F;CURVETO&#x5F;CUBIC&#x5F;REL</b> (unsigned short)  
Corresponds to a "relative cubic Bézier curveto" (c) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_CURVETO_QUADRATIC_ABS"></a><b>PATHSEG&#x5F;CURVETO&#x5F;QUADRATIC&#x5F;ABS</b> (unsigned short)  
Corresponds to a "absolute quadratic Bézier curveto" (Q) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_CURVETO_QUADRATIC_REL"></a><b>PATHSEG&#x5F;CURVETO&#x5F;QUADRATIC&#x5F;REL</b> (unsigned short)  
Corresponds to a "relative quadratic Bézier curveto" (q) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_ARC_ABS"></a><b>PATHSEG&#x5F;ARC&#x5F;ABS</b> (unsigned short)  
Corresponds to a "absolute arcto" (A) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_ARC_REL"></a><b>PATHSEG&#x5F;ARC&#x5F;REL</b> (unsigned short)  
Corresponds to a "relative arcto" (a) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_LINETO_HORIZONTAL_ABS"></a><b>PATHSEG&#x5F;LINETO&#x5F;HORIZONTAL&#x5F;ABS</b> (unsigned short)  
Corresponds to a "absolute horizontal lineto" (H) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_LINETO_HORIZONTAL_REL"></a><b>PATHSEG&#x5F;LINETO&#x5F;HORIZONTAL&#x5F;REL</b> (unsigned short)  
Corresponds to a "relative horizontal lineto" (h) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_LINETO_VERTICAL_ABS"></a><b>PATHSEG&#x5F;LINETO&#x5F;VERTICAL&#x5F;ABS</b> (unsigned short)  
Corresponds to a "absolute vertical lineto" (V) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_LINETO_VERTICAL_REL"></a><b>PATHSEG&#x5F;LINETO&#x5F;VERTICAL&#x5F;REL</b> (unsigned short)  
Corresponds to a "relative vertical lineto" (v) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_CURVETO_CUBIC_SMOOTH_ABS"></a><b>PATHSEG&#x5F;CURVETO&#x5F;CUBIC&#x5F;SMOOTH&#x5F;ABS</b> (unsigned short)  
Corresponds to a "absolute smooth cubic curveto" (S) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_CURVETO_CUBIC_SMOOTH_REL"></a><b>PATHSEG&#x5F;CURVETO&#x5F;CUBIC&#x5F;SMOOTH&#x5F;REL</b> (unsigned short)  
Corresponds to a "relative smooth cubic curveto" (s) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_CURVETO_QUADRATIC_SMOOTH_ABS"></a><b>PATHSEG&#x5F;CURVETO&#x5F;QUADRATIC&#x5F;SMOOTH&#x5F;ABS</b> (unsigned short)  
Corresponds to a "absolute smooth quadratic curveto" (T) path data command.

<a id="__svg__SVGPathSeg__PATHSEG_CURVETO_QUADRATIC_SMOOTH_REL"></a><b>PATHSEG&#x5F;CURVETO&#x5F;QUADRATIC&#x5F;SMOOTH&#x5F;REL</b> (unsigned short)  
Corresponds to a "relative smooth quadratic curveto" (t) path data command.

Attributes:  
<a id="__svg__SVGPathSeg__pathSegType"></a><b>pathSegType</b> (readonly unsigned short)  
The type of the path segment as specified by one of the constants defined on this interface.

<a id="__svg__SVGPathSeg__pathSegTypeAsLetter"></a><b>pathSegTypeAsLetter</b> (readonly DOMString)  
The type of the path segment, specified by the corresponding one character command name.

### <a id="InterfaceSVGPathSegClosePath"></a>8.5.2 Interface SVGPathSegClosePath

The [SVGPathSegClosePath](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegClosePath) interface corresponds to a "closepath" (z) path data command.

```text
interface SVGPathSegClosePath : SVGPathSeg {
};
```
### <a id="InterfaceSVGPathSegMovetoAbs"></a>8.5.3 Interface SVGPathSegMovetoAbs

The [SVGPathSegMovetoAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegMovetoAbs) interface corresponds to an "absolute moveto" (M) path data command.

```text
interface SVGPathSegMovetoAbs : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegMovetoAbs__x"></a><b>x</b> (float)  
The absolute X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegMovetoAbs__y"></a><b>y</b> (float)  
The absolute Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegMovetoRel"></a>8.5.4 Interface SVGPathSegMovetoRel

The [SVGPathSegMovetoRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegMovetoRel) interface corresponds to a "relative moveto" (m) path data command.

```text
interface SVGPathSegMovetoRel : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegMovetoRel__x"></a><b>x</b> (float)  
The relative X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegMovetoRel__y"></a><b>y</b> (float)  
The relative Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegLinetoAbs"></a>8.5.5 Interface SVGPathSegLinetoAbs

The [SVGPathSegLinetoAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoAbs) interface corresponds to an "absolute lineto" (L) path data command.

```text
interface SVGPathSegLinetoAbs : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegLinetoAbs__x"></a><b>x</b> (float)  
The absolute X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegLinetoAbs__y"></a><b>y</b> (float)  
The absolute Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegLinetoRel"></a>8.5.6 Interface SVGPathSegLinetoRel

The [SVGPathSegLinetoRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoRel) interface corresponds to a "relative lineto" (l) path data command.

```text
interface SVGPathSegLinetoRel : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegLinetoRel__x"></a><b>x</b> (float)  
The relative X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegLinetoRel__y"></a><b>y</b> (float)  
The relative Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegCurvetoCubicAbs"></a>8.5.7 Interface SVGPathSegCurvetoCubicAbs

The [SVGPathSegCurvetoCubicAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicAbs) interface corresponds to an "absolute cubic Bézier curveto" (C) path data command.

```text
interface SVGPathSegCurvetoCubicAbs : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
  attribute float x1 setraises(DOMException);
  attribute float y1 setraises(DOMException);
  attribute float x2 setraises(DOMException);
  attribute float y2 setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegCurvetoCubicAbs__x"></a><b>x</b> (float)  
The absolute X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicAbs__y"></a><b>y</b> (float)  
The absolute Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicAbs__x1"></a><b>x1</b> (float)  
The absolute X coordinate for the first control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicAbs__y1"></a><b>y1</b> (float)  
The absolute Y coordinate for the first control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicAbs__x2"></a><b>x2</b> (float)  
The absolute X coordinate for the second control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicAbs__y2"></a><b>y2</b> (float)  
The absolute Y coordinate for the second control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegCurvetoCubicRel"></a>8.5.8 Interface SVGPathSegCurvetoCubicRel

The [SVGPathSegCurvetoCubicRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicRel) interface corresponds to a "relative cubic Bézier curveto" (c) path data command.

```text
interface SVGPathSegCurvetoCubicRel : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
  attribute float x1 setraises(DOMException);
  attribute float y1 setraises(DOMException);
  attribute float x2 setraises(DOMException);
  attribute float y2 setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegCurvetoCubicRel__x"></a><b>x</b> (float)  
The relative X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicRel__y"></a><b>y</b> (float)  
The relative Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicRel__x1"></a><b>x1</b> (float)  
The relative X coordinate for the first control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicRel__y1"></a><b>y1</b> (float)  
The relative Y coordinate for the first control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicRel__x2"></a><b>x2</b> (float)  
The relative X coordinate for the second control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicRel__y2"></a><b>y2</b> (float)  
The relative Y coordinate for the second control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegCurvetoQuadraticAbs"></a>8.5.9 Interface SVGPathSegCurvetoQuadraticAbs

The [SVGPathSegCurvetoQuadraticAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticAbs) interface corresponds to an "absolute quadratic Bézier curveto" (Q) path data command.

```text
interface SVGPathSegCurvetoQuadraticAbs : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
  attribute float x1 setraises(DOMException);
  attribute float y1 setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegCurvetoQuadraticAbs__x"></a><b>x</b> (float)  
The absolute X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoQuadraticAbs__y"></a><b>y</b> (float)  
The absolute Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoQuadraticAbs__x1"></a><b>x1</b> (float)  
The absolute X coordinate for the first control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoQuadraticAbs__y1"></a><b>y1</b> (float)  
The absolute Y coordinate for the first control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegCurvetoQuadraticRel"></a>8.5.10 Interface SVGPathSegCurvetoQuadraticRel

The [SVGPathSegCurvetoQuadraticRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticRel) interface corresponds to a "relative quadratic Bézier curveto" (q) path data command.

```text
interface SVGPathSegCurvetoQuadraticRel : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
  attribute float x1 setraises(DOMException);
  attribute float y1 setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegCurvetoQuadraticRel__x"></a><b>x</b> (float)  
The relative X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoQuadraticRel__y"></a><b>y</b> (float)  
The relative Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoQuadraticRel__x1"></a><b>x1</b> (float)  
The relative X coordinate for the first control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoQuadraticRel__y1"></a><b>y1</b> (float)  
The relative Y coordinate for the first control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegArcAbs"></a>8.5.11 Interface SVGPathSegArcAbs

The [SVGPathSegArcAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegArcAbs) interface corresponds to an "absolute arcto" (A) path data command.

```text
interface SVGPathSegArcAbs : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
  attribute float r1 setraises(DOMException);
  attribute float r2 setraises(DOMException);
  attribute float angle setraises(DOMException);
  attribute boolean largeArcFlag setraises(DOMException);
  attribute boolean sweepFlag setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegArcAbs__x"></a><b>x</b> (float)  
The absolute X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegArcAbs__y"></a><b>y</b> (float)  
The absolute Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegArcAbs__r1"></a><b>r1</b> (float)  
The x-axis radius for the ellipse (i.e., r1).

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegArcAbs__r2"></a><b>r2</b> (float)  
The y-axis radius for the ellipse (i.e., r2).

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegArcAbs__angle"></a><b>angle</b> (float)  
The rotation angle in degrees for the ellipse's x-axis relative to the x-axis of the user coordinate system.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegArcAbs__largeArcFlag"></a><b>largeArcFlag</b> (boolean)  
The value of the large-arc-flag parameter.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegArcAbs__sweepFlag"></a><b>sweepFlag</b> (boolean)  
The value of the sweep-flag parameter.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegArcRel"></a>8.5.12 Interface SVGPathSegArcRel

The [SVGPathSegArcRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegArcRel) interface corresponds to a "relative arcto" (a) path data command.

```text
interface SVGPathSegArcRel : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
  attribute float r1 setraises(DOMException);
  attribute float r2 setraises(DOMException);
  attribute float angle setraises(DOMException);
  attribute boolean largeArcFlag setraises(DOMException);
  attribute boolean sweepFlag setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegArcRel__x"></a><b>x</b> (float)  
The relative X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegArcRel__y"></a><b>y</b> (float)  
The relative Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegArcRel__r1"></a><b>r1</b> (float)  
The x-axis radius for the ellipse (i.e., r1).

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegArcRel__r2"></a><b>r2</b> (float)  
The y-axis radius for the ellipse (i.e., r2).

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegArcRel__angle"></a><b>angle</b> (float)  
The rotation angle in degrees for the ellipse's x-axis relative to the x-axis of the user coordinate system.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegArcRel__largeArcFlag"></a><b>largeArcFlag</b> (boolean)  
The value of the large-arc-flag parameter.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegArcRel__sweepFlag"></a><b>sweepFlag</b> (boolean)  
The value of the sweep-flag parameter.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegLinetoHorizontalAbs"></a>8.5.13 Interface SVGPathSegLinetoHorizontalAbs

The [SVGPathSegLinetoHorizontalAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoHorizontalAbs) interface corresponds to an "absolute horizontal lineto" (H) path data command.

```text
interface SVGPathSegLinetoHorizontalAbs : SVGPathSeg {
  attribute float x setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegLinetoHorizontalAbs__x"></a><b>x</b> (float)  
The absolute X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegLinetoHorizontalRel"></a>8.5.14 Interface SVGPathSegLinetoHorizontalRel

The [SVGPathSegLinetoHorizontalRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoHorizontalRel) interface corresponds to a "relative horizontal lineto" (h) path data command.

```text
interface SVGPathSegLinetoHorizontalRel : SVGPathSeg {
  attribute float x setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegLinetoHorizontalRel__x"></a><b>x</b> (float)  
The relative X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegLinetoVerticalAbs"></a>8.5.15 Interface SVGPathSegLinetoVerticalAbs

The [SVGPathSegLinetoVerticalAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoVerticalAbs) interface corresponds to an "absolute vertical lineto" (V) path data command.

```text
interface SVGPathSegLinetoVerticalAbs : SVGPathSeg {
  attribute float y setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegLinetoVerticalAbs__y"></a><b>y</b> (float)  
The absolute Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegLinetoVerticalRel"></a>8.5.16 Interface SVGPathSegLinetoVerticalRel

The [SVGPathSegLinetoVerticalRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoVerticalRel) interface corresponds to a "relative vertical lineto" (v) path data command.

```text
interface SVGPathSegLinetoVerticalRel : SVGPathSeg {
  attribute float y setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegLinetoVerticalRel__y"></a><b>y</b> (float)  
The relative Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegCurvetoCubicSmoothAbs"></a>8.5.17 Interface SVGPathSegCurvetoCubicSmoothAbs

The [SVGPathSegCurvetoCubicSmoothAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicSmoothAbs) interface corresponds to an "absolute smooth cubic curveto" (S) path data command.

```text
interface SVGPathSegCurvetoCubicSmoothAbs : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
  attribute float x2 setraises(DOMException);
  attribute float y2 setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegCurvetoCubicSmoothAbs__x"></a><b>x</b> (float)  
The absolute X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicSmoothAbs__y"></a><b>y</b> (float)  
The absolute Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicSmoothAbs__x2"></a><b>x2</b> (float)  
The absolute X coordinate for the second control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicSmoothAbs__y2"></a><b>y2</b> (float)  
The absolute Y coordinate for the second control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegCurvetoCubicSmoothRel"></a>8.5.18 Interface SVGPathSegCurvetoCubicSmoothRel

The [SVGPathSegCurvetoCubicSmoothRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicSmoothRel) interface corresponds to a "relative smooth cubic curveto" (s) path data command.

```text
interface SVGPathSegCurvetoCubicSmoothRel : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
  attribute float x2 setraises(DOMException);
  attribute float y2 setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegCurvetoCubicSmoothRel__x"></a><b>x</b> (float)  
The relative X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicSmoothRel__y"></a><b>y</b> (float)  
The relative Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicSmoothRel__x2"></a><b>x2</b> (float)  
The relative X coordinate for the second control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoCubicSmoothRel__y2"></a><b>y2</b> (float)  
The relative Y coordinate for the second control point.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegCurvetoQuadraticSmoothAbs"></a>8.5.19 Interface SVGPathSegCurvetoQuadraticSmoothAbs

The [SVGPathSegCurvetoQuadraticSmoothAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticSmoothAbs) interface corresponds to an "absolute smooth cubic curveto" (T) path data command.

```text
interface SVGPathSegCurvetoQuadraticSmoothAbs : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegCurvetoQuadraticSmoothAbs__x"></a><b>x</b> (float)  
The absolute X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoQuadraticSmoothAbs__y"></a><b>y</b> (float)  
The absolute Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegCurvetoQuadraticSmoothRel"></a>8.5.20 Interface SVGPathSegCurvetoQuadraticSmoothRel

The [SVGPathSegCurvetoQuadraticSmoothRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticSmoothRel) interface corresponds to a "relative smooth cubic curveto" (t) path data command.

```text
interface SVGPathSegCurvetoQuadraticSmoothRel : SVGPathSeg {
  attribute float x setraises(DOMException);
  attribute float y setraises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegCurvetoQuadraticSmoothRel__x"></a><b>x</b> (float)  
The relative X coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

<a id="__svg__SVGPathSegCurvetoQuadraticSmoothRel__y"></a><b>y</b> (float)  
The relative Y coordinate for the end point of this path segment.

Exceptions on setting  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised on an attempt to change the value of a [read only attribute](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html#ReadOnlyNodes).

### <a id="InterfaceSVGPathSegList"></a>8.5.21 Interface SVGPathSegList

This interface defines a list of SVGPathSeg objects.

[SVGPathSegList](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegList) has the same attributes and methods as other SVGxxxList interfaces. Implementers may consider using a single base class to implement the various SVGxxxList interfaces.

```text
interface SVGPathSegList {

  readonly attribute unsigned long numberOfItems;

  void clear() raises(DOMException);
  SVGPathSeg initialize(in SVGPathSeg newItem) raises(DOMException);
  SVGPathSeg getItem(in unsigned long index) raises(DOMException);
  SVGPathSeg insertItemBefore(in SVGPathSeg newItem, in unsigned long index) raises(DOMException);
  SVGPathSeg replaceItem(in SVGPathSeg newItem, in unsigned long index) raises(DOMException);
  SVGPathSeg removeItem(in unsigned long index) raises(DOMException);
  SVGPathSeg appendItem(in SVGPathSeg newItem) raises(DOMException);
};
```
Attributes:  
<a id="__svg__SVGPathSegList__numberOfItems"></a><b>numberOfItems</b> (readonly unsigned long)  
The number of items in the list.

Operations:  
<a id="__svg__SVGPathSegList__clear"></a>void <b>clear</b>()  
Clears all existing current items from the list, with the result being an empty list.

Exceptions  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised when the list cannot be modified.

<a id="__svg__SVGPathSegList__initialize"></a>[SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <b>initialize</b>(in [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <var>newItem</var>)  
Clears all existing current items from the list and re-initializes the list to hold the single item specified by the parameter. If the inserted item is already in a list, it is removed from its previous list before it is inserted into this list. The inserted item is the item itself and not a copy.

Parameters  
1.  [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <var>newItem</var>
    The item which should become the only member of the list.

Returns  
The item being inserted into the list.

Exceptions  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised when the list cannot be modified.

<a id="__svg__SVGPathSegList__getItem"></a>[SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <b>getItem</b>(in unsigned long <var>index</var>)  
Returns the specified item from the list. The returned item is the item itself and not a copy. Any changes made to the item are immediately reflected in the list.

Parameters  
1.  unsigned long <var>index</var>
    The index of the item from the list which is to be returned. The first item is number 0.

Returns  
The selected item.

Exceptions  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code INDEX_SIZE_ERR  
Raised if the index number is greater than or equal to [numberOfItems](#__svg__SVGPathSegList__numberOfItems).

<a id="__svg__SVGPathSegList__insertItemBefore"></a>[SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <b>insertItemBefore</b>(in [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <var>newItem</var>, in unsigned long <var>index</var>)  
Inserts a new item into the list at the specified position. The first item is number 0. If <var>newItem</var> is already in a list, it is removed from its previous list before it is inserted into this list. The inserted item is the item itself and not a copy. If the item is already in this list, note that the index of the item to insert before is <i>before</i> the removal of the item.

Parameters  
1.  [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <var>newItem</var>
    The item which is to be inserted into the list.
2.  unsigned long <var>index</var>
    The index of the item before which the new item is to be inserted. The first item is number 0. If the index is equal to 0, then the new item is inserted at the front of the list. If the index is greater than or equal to [numberOfItems](#__svg__SVGPathSegList__numberOfItems), then the new item is appended to the end of the list.

Returns  
The inserted item.

Exceptions  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised when the list cannot be modified.

<a id="__svg__SVGPathSegList__replaceItem"></a>[SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <b>replaceItem</b>(in [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <var>newItem</var>, in unsigned long <var>index</var>)  
Replaces an existing item in the list with a new item. If <var>newItem</var> is already in a list, it is removed from its previous list before it is inserted into this list. The inserted item is the item itself and not a copy. If the item is already in this list, note that the index of the item to replace is <i>before</i> the removal of the item.

Parameters  
1.  [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <var>newItem</var>
    The item which is to be inserted into the list.
2.  unsigned long <var>index</var>
    The index of the item which is to be replaced. The first item is number 0.

Returns  
The inserted item.

Exceptions  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised when the list cannot be modified.

[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code INDEX_SIZE_ERR  
Raised if the index number is greater than or equal to [numberOfItems](#__svg__SVGPathSegList__numberOfItems).

<a id="__svg__SVGPathSegList__removeItem"></a>[SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <b>removeItem</b>(in unsigned long <var>index</var>)  
Removes an existing item from the list.

Parameters  
1.  unsigned long <var>index</var>
    The index of the item which is to be removed. The first item is number 0.

Returns  
The removed item.

Exceptions  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised when the list cannot be modified.

[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code INDEX_SIZE_ERR  
Raised if the index number is greater than or equal to [numberOfItems](#__svg__SVGPathSegList__numberOfItems).

<a id="__svg__SVGPathSegList__appendItem"></a>[SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <b>appendItem</b>(in [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <var>newItem</var>)  
Inserts a new item at the end of the list. If <var>newItem</var> is already in a list, it is removed from its previous list before it is inserted into this list. The inserted item is the item itself and not a copy.

Parameters  
1.  [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) <var>newItem</var>
    The item which is to be inserted. The first item is number 0.

Returns  
The inserted item.

Exceptions  
[DOMException](https://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-17189187), code NO_MODIFICATION_ALLOWED_ERR  
Raised when the list cannot be modified.

### <a id="InterfaceSVGAnimatedPathData"></a>8.5.22 Interface SVGAnimatedPathData

The SVGAnimatedPathData interface supports elements which have a ‘d’ attribute which holds SVG path data, and supports the ability to animate that attribute.

The [SVGAnimatedPathData](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGAnimatedPathData) interface provides two lists to access and modify the base (i.e., static) contents of the ‘d’ attribute:

- DOM attribute [pathSegList](#__svg__SVGAnimatedPathData__pathSegList) provides access to the static/base contents of the ‘d’ attribute in a form which matches one-for-one with SVG's syntax.
- DOM attribute [normalizedPathSegList](#__svg__SVGAnimatedPathData__normalizedPathSegList) provides normalized access to the static/base contents of the ‘d’ attribute where all path data commands are expressed in terms of the following subset of [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) types: SVG_PATHSEG_MOVETO_ABS (M), SVG_PATHSEG_LINETO_ABS (L), SVG_PATHSEG_CURVETO_CUBIC_ABS (C) and SVG_PATHSEG_CLOSEPATH (z).

and two lists to access the current animated values of the ‘d’ attribute:

- DOM attribute [animatedPathSegList](#__svg__SVGAnimatedPathData__animatedPathSegList) provides access to the current animated contents of the ‘d’ attribute in a form which matches one-for-one with SVG's syntax.
- DOM attribute [animatedNormalizedPathSegList](#__svg__SVGAnimatedPathData__animatedNormalizedPathSegList) provides normalized access to the current animated contents of the ‘d’ attribute where all path data commands are expressed in terms of the following subset of [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) types: SVG_PATHSEG_MOVETO_ABS (M), SVG_PATHSEG_LINETO_ABS (L), SVG_PATHSEG_CURVETO_CUBIC_ABS (C) and SVG_PATHSEG_CLOSEPATH (z).

Each of the two lists are always kept synchronized. Modifications to one list will immediately cause the corresponding list to be modified. Modifications to [normalizedPathSegList](#__svg__SVGAnimatedPathData__normalizedPathSegList) might cause entries in [pathSegList](#__svg__SVGAnimatedPathData__pathSegList) to be broken into a set of normalized path segments.

Additionally, the [‘d’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#DAttribute) attribute on the [‘path’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathElement) element accessed via the XML DOM (e.g., using the `getAttribute()` method call) will reflect any changes made to [pathSegList](#__svg__SVGAnimatedPathData__pathSegList) or [normalizedPathSegList](#__svg__SVGAnimatedPathData__normalizedPathSegList).

```text
interface SVGAnimatedPathData {
  readonly attribute SVGPathSegList pathSegList;
  readonly attribute SVGPathSegList normalizedPathSegList;
  readonly attribute SVGPathSegList animatedPathSegList;
  readonly attribute SVGPathSegList animatedNormalizedPathSegList;
};
```
Attributes:  
<a id="__svg__SVGAnimatedPathData__pathSegList"></a><b>pathSegList</b> (readonly [SVGPathSegList](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegList))  
Provides access to the base (i.e., static) contents of the ‘d’ attribute in a form which matches one-for-one with SVG's syntax. Thus, if the ‘d’ attribute has an "absolute moveto (M)" and an "absolute arcto (A)" command, then [pathSegList](#__svg__SVGAnimatedPathData__pathSegList) will have two entries: a SVG_PATHSEG_MOVETO_ABS and a SVG_PATHSEG_ARC_ABS.

<a id="__svg__SVGAnimatedPathData__normalizedPathSegList"></a><b>normalizedPathSegList</b> (readonly [SVGPathSegList](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegList))  
Provides access to the base (i.e., static) contents of the ‘d’ attribute in a form where all path data commands are expressed in terms of the following subset of [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) types: SVG_PATHSEG_MOVETO_ABS (M), SVG_PATHSEG_LINETO_ABS (L), SVG_PATHSEG_CURVETO_CUBIC_ABS (C) and SVG_PATHSEG_CLOSEPATH (z). Thus, if the ‘d’ attribute has an "absolute moveto (M)" and an "absolute arcto (A)" command, then pathSegList will have one SVG_PATHSEG_MOVETO_ABS entry followed by a series of SVG_PATHSEG_LINETO_ABS entries which approximate the arc. This alternate representation is available to provide a simpler interface to developers who would benefit from a more limited set of commands.

The only valid [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) types are SVG_PATHSEG_MOVETO_ABS (M), SVG_PATHSEG_LINETO_ABS (L), SVG_PATHSEG_CURVETO_CUBIC_ABS (C) and SVG_PATHSEG_CLOSEPATH (z).

<a id="__svg__SVGAnimatedPathData__animatedPathSegList"></a><b>animatedPathSegList</b> (readonly [SVGPathSegList](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegList))  
Provides access to the current animated contents of the ‘d’ attribute in a form which matches one-for-one with SVG's syntax. If the given attribute or property is being animated, contains the current animated value of the attribute or property, and both the object itself and its contents are read only. If the given attribute or property is not currently being animated, contains the same value as [pathSegList](#__svg__SVGAnimatedPathData__pathSegList).

<a id="__svg__SVGAnimatedPathData__animatedNormalizedPathSegList"></a><b>animatedNormalizedPathSegList</b> (readonly [SVGPathSegList](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegList))  
Provides access to the current animated contents of the ‘d’ attribute in a form where all path data commands are expressed in terms of the following subset of [SVGPathSeg](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSeg) types: SVG_PATHSEG_MOVETO_ABS (M), SVG_PATHSEG_LINETO_ABS (L), SVG_PATHSEG_CURVETO_CUBIC_ABS (C) and SVG_PATHSEG_CLOSEPATH (z). If the given attribute or property is being animated, contains the current animated value of the attribute or property, and both the object itself and its contents are read only. If the given attribute or property is not currently being animated, contains the same value as [normalizedPathSegList](#__svg__SVGAnimatedPathData__normalizedPathSegList).

### <a id="InterfaceSVGPathElement"></a>8.5.23 Interface SVGPathElement

The [SVGPathElement](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathElement) interface corresponds to the [‘path’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathElement) element.

```text
interface SVGPathElement : SVGElement,
                           SVGTests,
                           SVGLangSpace,
                           SVGExternalResourcesRequired,
                           SVGStylable,
                           SVGTransformable,
                           SVGAnimatedPathData {

  readonly attribute SVGAnimatedNumber pathLength;

  float getTotalLength();
  SVGPoint getPointAtLength(in float distance);
  unsigned long getPathSegAtLength(in float distance);
  SVGPathSegClosePath createSVGPathSegClosePath();
  SVGPathSegMovetoAbs createSVGPathSegMovetoAbs(in float x, in float y);
  SVGPathSegMovetoRel createSVGPathSegMovetoRel(in float x, in float y);
  SVGPathSegLinetoAbs createSVGPathSegLinetoAbs(in float x, in float y);
  SVGPathSegLinetoRel createSVGPathSegLinetoRel(in float x, in float y);
  SVGPathSegCurvetoCubicAbs createSVGPathSegCurvetoCubicAbs(in float x, in float y, in float x1, in float y1, in float x2, in float y2);
  SVGPathSegCurvetoCubicRel createSVGPathSegCurvetoCubicRel(in float x, in float y, in float x1, in float y1, in float x2, in float y2);
  SVGPathSegCurvetoQuadraticAbs createSVGPathSegCurvetoQuadraticAbs(in float x, in float y, in float x1, in float y1);
  SVGPathSegCurvetoQuadraticRel createSVGPathSegCurvetoQuadraticRel(in float x, in float y, in float x1, in float y1);
  SVGPathSegArcAbs createSVGPathSegArcAbs(in float x, in float y, in float r1, in float r2, in float angle, in boolean largeArcFlag, in boolean sweepFlag);
  SVGPathSegArcRel createSVGPathSegArcRel(in float x, in float y, in float r1, in float r2, in float angle, in boolean largeArcFlag, in boolean sweepFlag);
  SVGPathSegLinetoHorizontalAbs createSVGPathSegLinetoHorizontalAbs(in float x);
  SVGPathSegLinetoHorizontalRel createSVGPathSegLinetoHorizontalRel(in float x);
  SVGPathSegLinetoVerticalAbs createSVGPathSegLinetoVerticalAbs(in float y);
  SVGPathSegLinetoVerticalRel createSVGPathSegLinetoVerticalRel(in float y);
  SVGPathSegCurvetoCubicSmoothAbs createSVGPathSegCurvetoCubicSmoothAbs(in float x, in float y, in float x2, in float y2);
  SVGPathSegCurvetoCubicSmoothRel createSVGPathSegCurvetoCubicSmoothRel(in float x, in float y, in float x2, in float y2);
  SVGPathSegCurvetoQuadraticSmoothAbs createSVGPathSegCurvetoQuadraticSmoothAbs(in float x, in float y);
  SVGPathSegCurvetoQuadraticSmoothRel createSVGPathSegCurvetoQuadraticSmoothRel(in float x, in float y);
};
```
Attributes:  
<a id="__svg__SVGPathElement__pathLength"></a><b>pathLength</b> (readonly [SVGAnimatedNumber](https://www.w3.org/TR/2011/REC-SVG11-20110816/types.html#InterfaceSVGAnimatedNumber))  
Corresponds to attribute [pathLength](#__svg__SVGPathElement__pathLength) on the given [‘path’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathElement) element.

Operations:  
<a id="__svg__SVGPathElement__getTotalLength"></a>float <b>getTotalLength</b>()  
Returns the user agent's computed value for the total length of the path using the user agent's distance-along-a-path algorithm, as a distance in the current user coordinate system.

Returns  
The total length of the path.

<a id="__svg__SVGPathElement__getPointAtLength"></a>[SVGPoint](https://www.w3.org/TR/2011/REC-SVG11-20110816/coords.html#InterfaceSVGPoint) <b>getPointAtLength</b>(in float <var>distance</var>)  
Returns the (x,y) coordinate in user space which is <var>distance</var> units along the path, utilizing the user agent's distance-along-a-path algorithm.

Parameters  
1.  float <var>distance</var>
    The distance along the path, relative to the start of the path, as a distance in the current user coordinate system.

Returns  
The returned point in user space.

<a id="__svg__SVGPathElement__getPathSegAtLength"></a>unsigned long <b>getPathSegAtLength</b>(in float <var>distance</var>)  
Returns the index into [pathSegList](svg11-path-data-grammar--paths.html--bd8e7b978996.md#__svg__SVGAnimatedPathData__pathSegList) which is <var>distance</var> units along the path, utilizing the user agent's distance-along-a-path algorithm.

Parameters  
1.  float <var>distance</var>
    The distance along the path, relative to the start of the path, as a distance in the current user coordinate system.

Returns  
The index of the path segment, where the first path segment is number 0.

<a id="__svg__SVGPathElement__createSVGPathSegClosePath"></a>[SVGPathSegClosePath](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegClosePath) <b>createSVGPathSegClosePath</b>()  
Returns a stand-alone, parentless [SVGPathSegClosePath](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegClosePath) object.

Returns  
A stand-alone, parentless [SVGPathSegClosePath](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegClosePath) object.

<a id="__svg__SVGPathElement__createSVGPathSegMovetoAbs"></a>[SVGPathSegMovetoAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegMovetoAbs) <b>createSVGPathSegMovetoAbs</b>(in float <var>x</var>, in float <var>y</var>)  
Returns a stand-alone, parentless [SVGPathSegMovetoAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegMovetoAbs) object.

Parameters  
1.  float <var>x</var>
    The absolute X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The absolute Y coordinate for the end point of this path segment.

Returns  
A stand-alone, parentless [SVGPathSegMovetoAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegMovetoAbs) object.

<a id="__svg__SVGPathElement__createSVGPathSegMovetoRel"></a>[SVGPathSegMovetoRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegMovetoRel) <b>createSVGPathSegMovetoRel</b>(in float <var>x</var>, in float <var>y</var>)  
Returns a stand-alone, parentless [SVGPathSegMovetoRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegMovetoRel) object.

Parameters  
1.  float <var>x</var>
    The relative X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The relative Y coordinate for the end point of this path segment.

Returns  
A stand-alone, parentless [SVGPathSegMovetoRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegMovetoRel) object.

<a id="__svg__SVGPathElement__createSVGPathSegLinetoAbs"></a>[SVGPathSegLinetoAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoAbs) <b>createSVGPathSegLinetoAbs</b>(in float <var>x</var>, in float <var>y</var>)  
Returns a stand-alone, parentless [SVGPathSegLinetoAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoAbs) object.

Parameters  
1.  float <var>x</var>
    The absolute X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The absolute Y coordinate for the end point of this path segment.

Returns  
A stand-alone, parentless [SVGPathSegLinetoAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoAbs) object.

<a id="__svg__SVGPathElement__createSVGPathSegLinetoRel"></a>[SVGPathSegLinetoRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoRel) <b>createSVGPathSegLinetoRel</b>(in float <var>x</var>, in float <var>y</var>)  
Returns a stand-alone, parentless [SVGPathSegLinetoRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoRel) object.

Parameters  
1.  float <var>x</var>
    The relative X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The relative Y coordinate for the end point of this path segment.

Returns  
A stand-alone, parentless [SVGPathSegLinetoRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoRel) object.

<a id="__svg__SVGPathElement__createSVGPathSegCurvetoCubicAbs"></a>[SVGPathSegCurvetoCubicAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicAbs) <b>createSVGPathSegCurvetoCubicAbs</b>(in float <var>x</var>, in float <var>y</var>, in float <var>x1</var>, in float <var>y1</var>, in float <var>x2</var>, in float <var>y2</var>)  
Returns a stand-alone, parentless [SVGPathSegCurvetoCubicAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicAbs) object.

Parameters  
1.  float <var>x</var>
    The absolute X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The absolute Y coordinate for the end point of this path segment.
3.  float <var>x1</var>
    The absolute X coordinate for the first control point.
4.  float <var>y1</var>
    The absolute Y coordinate for the first control point.
5.  float <var>x2</var>
    The absolute X coordinate for the second control point.
6.  float <var>y2</var>
    The absolute Y coordinate for the second control point.

Returns  
A stand-alone, parentless [SVGPathSegCurvetoCubicAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicAbs) object.

<a id="__svg__SVGPathElement__createSVGPathSegCurvetoCubicRel"></a>[SVGPathSegCurvetoCubicRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicRel) <b>createSVGPathSegCurvetoCubicRel</b>(in float <var>x</var>, in float <var>y</var>, in float <var>x1</var>, in float <var>y1</var>, in float <var>x2</var>, in float <var>y2</var>)  
Returns a stand-alone, parentless [SVGPathSegCurvetoCubicRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicRel) object.

Parameters  
1.  float <var>x</var>
    The relative X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The relative Y coordinate for the end point of this path segment.
3.  float <var>x1</var>
    The relative X coordinate for the first control point.
4.  float <var>y1</var>
    The relative Y coordinate for the first control point.
5.  float <var>x2</var>
    The relative X coordinate for the second control point.
6.  float <var>y2</var>
    The relative Y coordinate for the second control point.

Returns  
A stand-alone, parentless [SVGPathSegCurvetoCubicRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicRel) object.

<a id="__svg__SVGPathElement__createSVGPathSegCurvetoQuadraticAbs"></a>[SVGPathSegCurvetoQuadraticAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticAbs) <b>createSVGPathSegCurvetoQuadraticAbs</b>(in float <var>x</var>, in float <var>y</var>, in float <var>x1</var>, in float <var>y1</var>)  
Returns a stand-alone, parentless [SVGPathSegCurvetoQuadraticAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticAbs) object.

Parameters  
1.  float <var>x</var>
    The absolute X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The absolute Y coordinate for the end point of this path segment.
3.  float <var>x1</var>
    The absolute X coordinate for the first control point.
4.  float <var>y1</var>
    The absolute Y coordinate for the first control point.

Returns  
A stand-alone, parentless [SVGPathSegCurvetoQuadraticAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticAbs) object.

<a id="__svg__SVGPathElement__createSVGPathSegCurvetoQuadraticRel"></a>[SVGPathSegCurvetoQuadraticRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticRel) <b>createSVGPathSegCurvetoQuadraticRel</b>(in float <var>x</var>, in float <var>y</var>, in float <var>x1</var>, in float <var>y1</var>)  
Returns a stand-alone, parentless [SVGPathSegCurvetoQuadraticRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticRel) object.

Parameters  
1.  float <var>x</var>
    The relative X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The relative Y coordinate for the end point of this path segment.
3.  float <var>x1</var>
    The relative X coordinate for the first control point.
4.  float <var>y1</var>
    The relative Y coordinate for the first control point.

Returns  
A stand-alone, parentless [SVGPathSegCurvetoQuadraticRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticRel) object.

<a id="__svg__SVGPathElement__createSVGPathSegArcAbs"></a>[SVGPathSegArcAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegArcAbs) <b>createSVGPathSegArcAbs</b>(in float <var>x</var>, in float <var>y</var>, in float <var>r1</var>, in float <var>r2</var>, in float <var>angle</var>, in boolean <var>largeArcFlag</var>, in boolean <var>sweepFlag</var>)  
Returns a stand-alone, parentless [SVGPathSegArcAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegArcAbs) object.

Parameters  
1.  float <var>x</var>
    The absolute X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The absolute Y coordinate for the end point of this path segment.
3.  float <var>r1</var>
    The x-axis radius for the ellipse (i.e., r1).
4.  float <var>r2</var>
    The y-axis radius for the ellipse (i.e., r2).
5.  float <var>angle</var>
    The rotation angle in degrees for the ellipse's x-axis relative to the x-axis of the user coordinate system.
6.  boolean <var>largeArcFlag</var>
    The value of the large-arc-flag parameter.
7.  boolean <var>sweepFlag</var>
    The value of the large-arc-flag parameter.

Returns  
A stand-alone, parentless [SVGPathSegArcAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegArcAbs) object.

<a id="__svg__SVGPathElement__createSVGPathSegArcRel"></a>[SVGPathSegArcRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegArcRel) <b>createSVGPathSegArcRel</b>(in float <var>x</var>, in float <var>y</var>, in float <var>r1</var>, in float <var>r2</var>, in float <var>angle</var>, in boolean <var>largeArcFlag</var>, in boolean <var>sweepFlag</var>)  
Returns a stand-alone, parentless [SVGPathSegArcRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegArcRel) object.

Parameters  
1.  float <var>x</var>
    The relative X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The relative Y coordinate for the end point of this path segment.
3.  float <var>r1</var>
    The x-axis radius for the ellipse (i.e., r1).
4.  float <var>r2</var>
    The y-axis radius for the ellipse (i.e., r2).
5.  float <var>angle</var>
    The rotation angle in degrees for the ellipse's x-axis relative to the x-axis of the user coordinate system.
6.  boolean <var>largeArcFlag</var>
    The value of the large-arc-flag parameter.
7.  boolean <var>sweepFlag</var>
    The value of the large-arc-flag parameter.

Returns  
A stand-alone, parentless [SVGPathSegArcRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegArcRel) object.

<a id="__svg__SVGPathElement__createSVGPathSegLinetoHorizontalAbs"></a>[SVGPathSegLinetoHorizontalAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoHorizontalAbs) <b>createSVGPathSegLinetoHorizontalAbs</b>(in float <var>x</var>)  
Returns a stand-alone, parentless [SVGPathSegLinetoHorizontalAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoHorizontalAbs) object.

Parameters  
1.  float <var>x</var>
    The absolute X coordinate for the end point of this path segment.

Returns  
A stand-alone, parentless [SVGPathSegLinetoHorizontalAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoHorizontalAbs) object.

<a id="__svg__SVGPathElement__createSVGPathSegLinetoHorizontalRel"></a>[SVGPathSegLinetoHorizontalRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoHorizontalRel) <b>createSVGPathSegLinetoHorizontalRel</b>(in float <var>x</var>)  
Returns a stand-alone, parentless [SVGPathSegLinetoHorizontalRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoHorizontalRel) object.

Parameters  
1.  float <var>x</var>
    The relative X coordinate for the end point of this path segment.

Returns  
A stand-alone, parentless [SVGPathSegLinetoHorizontalRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoHorizontalRel) object.

<a id="__svg__SVGPathElement__createSVGPathSegLinetoVerticalAbs"></a>[SVGPathSegLinetoVerticalAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoVerticalAbs) <b>createSVGPathSegLinetoVerticalAbs</b>(in float <var>y</var>)  
Returns a stand-alone, parentless [SVGPathSegLinetoVerticalAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoVerticalAbs) object.

Parameters  
1.  float <var>y</var>
    The absolute Y coordinate for the end point of this path segment.

Returns  
A stand-alone, parentless [SVGPathSegLinetoVerticalAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoVerticalAbs) object.

<a id="__svg__SVGPathElement__createSVGPathSegLinetoVerticalRel"></a>[SVGPathSegLinetoVerticalRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoVerticalRel) <b>createSVGPathSegLinetoVerticalRel</b>(in float <var>y</var>)  
Returns a stand-alone, parentless [SVGPathSegLinetoVerticalRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoVerticalRel) object.

Parameters  
1.  float <var>y</var>
    The relative Y coordinate for the end point of this path segment.

Returns  
A stand-alone, parentless [SVGPathSegLinetoVerticalRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegLinetoVerticalRel) object.

<a id="__svg__SVGPathElement__createSVGPathSegCurvetoCubicSmoothAbs"></a>[SVGPathSegCurvetoCubicSmoothAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicSmoothAbs) <b>createSVGPathSegCurvetoCubicSmoothAbs</b>(in float <var>x</var>, in float <var>y</var>, in float <var>x2</var>, in float <var>y2</var>)  
Returns a stand-alone, parentless [SVGPathSegCurvetoCubicSmoothAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicSmoothAbs) object.

Parameters  
1.  float <var>x</var>
    The absolute X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The absolute Y coordinate for the end point of this path segment.
3.  float <var>x2</var>
    The absolute X coordinate for the second control point.
4.  float <var>y2</var>
    The absolute Y coordinate for the second control point.

Returns  
A stand-alone, parentless [SVGPathSegCurvetoCubicSmoothAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicSmoothAbs) object.

<a id="__svg__SVGPathElement__createSVGPathSegCurvetoCubicSmoothRel"></a>[SVGPathSegCurvetoCubicSmoothRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicSmoothRel) <b>createSVGPathSegCurvetoCubicSmoothRel</b>(in float <var>x</var>, in float <var>y</var>, in float <var>x2</var>, in float <var>y2</var>)  
Returns a stand-alone, parentless [SVGPathSegCurvetoCubicSmoothRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicSmoothRel) object.

Parameters  
1.  float <var>x</var>
    The relative X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The relative Y coordinate for the end point of this path segment.
3.  float <var>x2</var>
    The relative X coordinate for the second control point.
4.  float <var>y2</var>
    The relative Y coordinate for the second control point.

Returns  
A stand-alone, parentless [SVGPathSegCurvetoCubicSmoothRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoCubicSmoothRel) object.

<a id="__svg__SVGPathElement__createSVGPathSegCurvetoQuadraticSmoothAbs"></a>[SVGPathSegCurvetoQuadraticSmoothAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticSmoothAbs) <b>createSVGPathSegCurvetoQuadraticSmoothAbs</b>(in float <var>x</var>, in float <var>y</var>)  
Returns a stand-alone, parentless [SVGPathSegCurvetoQuadraticSmoothAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticSmoothAbs) object.

Parameters  
1.  float <var>x</var>
    The absolute X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The absolute Y coordinate for the end point of this path segment.

Returns  
A stand-alone, parentless [SVGPathSegCurvetoQuadraticSmoothAbs](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticSmoothAbs) object.

<a id="__svg__SVGPathElement__createSVGPathSegCurvetoQuadraticSmoothRel"></a>[SVGPathSegCurvetoQuadraticSmoothRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticSmoothRel) <b>createSVGPathSegCurvetoQuadraticSmoothRel</b>(in float <var>x</var>, in float <var>y</var>)  
Returns a stand-alone, parentless [SVGPathSegCurvetoQuadraticSmoothRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticSmoothRel) object.

Parameters  
1.  float <var>x</var>
    The relative X coordinate for the end point of this path segment.
2.  float <var>y</var>
    The relative Y coordinate for the end point of this path segment.

Returns  
A stand-alone, parentless [SVGPathSegCurvetoQuadraticSmoothRel](svg11-path-data-grammar--paths.html--bd8e7b978996.md#InterfaceSVGPathSegCurvetoQuadraticSmoothRel) object.
