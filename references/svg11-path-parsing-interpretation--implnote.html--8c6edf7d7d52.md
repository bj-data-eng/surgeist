Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Implementation Requirements – SVG 1.1 (Second Edition)](https://www.w3.org/TR/2011/REC-SVG11-20110816/implnote.html).

Original copyright notice (from the SVG 1.1 Second Edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Implementation Requirements – SVG 1.1 (Second Edition)

Source snapshot: https://www.w3.org/TR/2011/REC-SVG11-20110816/implnote.html

Snapshot SHA-256: 8c6edf7d7d52aa3d8c11f38276d090105f2504fd557edc2f5d4348e58d58e3e5

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# Appendix F: Implementation Requirements

<strong>This appendix is normative.</strong>

## <a id="Introduction"></a>F.1 Introduction

The following are notes about implementation requirements corresponding to various features in the SVG language.

## <a id="ErrorProcessing"></a>F.2 Error processing

There are various scenarios where an SVG document fragment is technically in error:

- When the content does not conform to the [XML 1.0 specification](https://www.w3.org/TR/2008/REC-xml-20081126/) \[[XML10](https://www.w3.org/TR/2011/REC-SVG11-20110816/refs.html#ref-XML10)\], such as the use of incorrect XML syntax
- When an element or attribute is encountered in the document which is not part of the [SVG DTD](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdtd.html) and which is not properly identified as being part of another namespace (see [Namespaces in XML](https://www.w3.org/TR/2006/REC-xml-names-20060816/) \[[XML-NS](https://www.w3.org/TR/2011/REC-SVG11-20110816/refs.html#ref-XML-NS)\])
- When an element has an attribute or property value which is not permissible according to this specification
- Other situations that are described as being <em>in
      error</em> in this specification

A document can go in and out of error over time. For example, document changes from the [SVG DOM](https://www.w3.org/TR/2011/REC-SVG11-20110816/svgdom.html) or from [animation](https://www.w3.org/TR/2011/REC-SVG11-20110816/animate.html) can cause a document to become <em>in error</em> and a further change can cause the document to become correct again.

The following error processing shall occur when a document is in error:

- The document shall be rendered up to, but not including, the first element which has an error. Exceptions:
  - If a [‘path’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathElement) element is the first element which has an error and the only errors are in the [path data](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathData) specification, then render the [‘path’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathElement) up to the point of the path data error. For related information, see [‘path’ element implementation notes](svg11-path-parsing-interpretation--implnote.html--8c6edf7d7d52.md#PathElementImplementationNotes).
  - If a [‘polyline’](https://www.w3.org/TR/2011/REC-SVG11-20110816/shapes.html#PolylineElement) or [‘polygon’](https://www.w3.org/TR/2011/REC-SVG11-20110816/shapes.html#PolygonElement) element is the first element which has an error and the only errors are within the [‘points’](https://www.w3.org/TR/2011/REC-SVG11-20110816/shapes.html#PolylineElementPointsAttribute) attribute, then render the [‘polyline’](https://www.w3.org/TR/2011/REC-SVG11-20110816/shapes.html#PolylineElement) or [‘polygon’](https://www.w3.org/TR/2011/REC-SVG11-20110816/shapes.html#PolygonElement) up to the segment with the error.

  This approach will provide a visual clue to the user or developer about where the error might be in the document.
- If the document has animations, the animations shall stop at the point at which an error is encountered and the visual presentation of the document shall reflect the animated status of the document at the point the error was encountered.
- A highly perceivable indication of error shall occur. For visual rendering situations, an example of an indication of error would be to render a translucent colored pattern such as a checkerboard on top of the area where the SVG content is rendered.
- If the user agent has access to an error reporting capability such as status bar, it is recommended that the user agent provide whatever additional detail it can to enable the user or developer to quickly find the source of the error. For example, the user agent might provide an error message along with a line number and character number at which the error was encountered.

Because of situations where a block of scripting changes might cause a given SVG document fragment to go into and out of error, error processing shall occur only at times when document presentation (e.g., rendering to the display device) is updated. In particular, error processing shall be disabled whenever redraw has been suspended via DOM calls to [suspendRedraw](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#__svg__SVGSVGElement__suspendRedraw).

## <a id="VersionControl"></a>F.3 Version control

The SVG user agent must verify the reference to the PUBLIC identifier in the `<!DOCTYPE>` statement or the namespace reference in the ‘xmlns’ attribute on the [‘svg’](https://www.w3.org/TR/2011/REC-SVG11-20110816/struct.html#SVGElement) element to ensure that the given document (or document fragment) identifies a version of the SVG language which the SVG user agent supports. If the version information is missing or the version information indicates a version of the SVG language which the SVG user agent does not support, then the SVG user agent is not required to render that document or fragment. In particular, it is not required that an SVG user agent attempt to render future versions of the SVG language. If the user environment provides such an option, the user agent should alert or otherwise notify the user that the version of the file is not supported and suggest an alternate processing option (e.g., installing an updated version of the user agent) if such an option exists.

An SVG user agent which supports the SVG Recommendation should alert or otherwise notify the user whenever it encounters an SVG document (or document fragment) whose `<!DOCTYPE>` statement or corresponding ‘xmlns’ attribute corresponds to a working draft version of the SVG specification. All content based on working drafts of this specification should be updated to the SVG Recommendation.

## <a id="RangeClamping"></a>F.4 Clamping values which are restricted to a particular range

Some numeric attribute and property values have restricted ranges, such as color component values. When out-of-range values are provided, the user agent shall defer any error checking until after presentation time, as composited actions might produce intermediate values which are out-of-range but final values which are within range.

Color values are not in error if they are out-of-range, even if final computations produce an out-of-range color value at presentation time. It is recommended that user agents clamp color values to the nearest color value (possibly determined by simple clipping) which the system can process as late as possible (e.g., presentation time), although it is acceptable for user agents to clamp color values as early as parse time. Thus, implementation dependencies might preclude consistent behavior across different systems when out-of-range color values are used.

Opacity values out-of-range are not in error and should be clamped to the range 0 to 1 at the time which opacity values have to be processed (e.g., at presentation time or when it is necessary to perform intermediate filter effect calculations).

## <a id="PathElementImplementationNotes"></a>F.5 ‘path’ element implementation notes

A conforming SVG user agent must implement path rendering as follows:

- Error handling:
  - The general rule for error handling in path data is that the SVG user agent shall render a [‘path’](svg11-path-data-grammar--paths.html--bd8e7b978996.md#PathElement) element up to (but not including) the path command containing the first error in the path data specification. This will provide a visual clue to the user or developer about where the error might be in the path data specification. This rule will greatly discourage generation of invalid SVG path data.
  - If a path data command contains an incorrect set of parameters, then the given path data command is rendered up to and including the last correctly defined path segment, even if that path segment is a sub-component of a compound path data command, such as a "lineto" with several pairs of coordinates. For example, for the path data string 'M 10,10 L 20,20,30', there is an odd number of parameters for the "L" command, which requires an even number of parameters. The user agent is required to draw the line from (10,10) to (20,20) and then perform error reporting since 'L 20 20' is the last correctly defined segment of the path data specification.
  - Wherever possible, all SVG user agents shall report all errors to the user.

- Markers, directionality and zero-length path segments:
  - If markers are specified, then a marker is drawn on every applicable vertex, even if the given vertex is the end point of a zero-length path segment and even if "moveto" commands follow each other.
  - Certain line-capping and line-joining situations and markers require that a path segment have directionality at its start and end points. Zero-length path segments have no directionality. In these cases, the following algorithm is used to establish directionality: to determine the directionality of the start point of a zero-length path segment, go backwards in the path data specification within the current subpath until you find a segment which has directionality at its end point (e.g., a path segment with non-zero length) and use its ending direction; otherwise, temporarily consider the start point to lack directionality. Similarly, to determine the directionality of the end point of a zero-length path segment, go forwards in the path data specification within the current subpath until you find a segment which has directionality at its start point (e.g., a path segment with non-zero length) and use its starting direction; otherwise, temporarily consider the end point to lack directionality. If the start point has directionality but the end point doesn't, then the end point uses the start point's directionality. If the end point has directionality but the start point doesn't, then the start point uses the end point's directionality. Otherwise, set the directionality for the path segment's start and end points to align with the positive x-axis in user space.
  - As mentioned in [Stroke Properties](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeProperties), linecaps must be painted for zero length subpaths when [‘stroke-linecap’](https://www.w3.org/TR/2011/REC-SVG11-20110816/painting.html#StrokeLinecapProperty) has a value of round or square.

- The S/s commands indicate that the first control point of the given cubic Bézier segment is calculated by reflecting the previous path segments second control point relative to the current point. The exact math is as follows. If the current point is (<var>curx</var>, <var>cury</var>) and the second control point of the previous path segment is (<var>oldx2</var>, <var>oldy2</var>), then the reflected point (i.e., (<var>newx1</var>, <var>newy1</var>), the first control point of the current path segment) is:

  ```text
  
  (newx1, newy1) = (curx - (oldx2 - curx), cury - (oldy2 - cury))
                 = (2*curx - oldx2, 2*cury - oldy2)
  ```
- A non-positive radius value is an error.

- Unrecognized contents within a path data stream (i.e., contents that are not part of the path data grammar) is an error.

## <a id="ArcImplementationNotes"></a>F.6 Elliptical arc implementation notes

### <a id="ArcSyntax"></a>F.6.1 Elliptical arc syntax

An elliptical arc is a particular path command. As such, it is described by the following parameters in order:

(<var>x</var><sub>1</sub>, <var>y</var><sub>1</sub>) are the absolute coordinates of the current point on the path, obtained from the last two parameters of the previous path command.

<var>r<sub>x</sub></var> and <var>r<sub>y</sub></var> are the radii of the ellipse (also known as its semi-major and semi-minor axes).

<var>φ</var> is the angle from the x-axis of the current coordinate system to the x-axis of the ellipse.

<var>f<sub>A</sub></var> is the large arc flag, and is 0 if an arc spanning less than or equal to 180 degrees is chosen, or 1 if an arc spanning greater than 180 degrees is chosen.

<var>f<sub>S</sub></var> is the sweep flag, and is 0 if the line joining center to arc sweeps through decreasing angles, or 1 if it sweeps through increasing angles.

(<var>x</var><sub>2</sub>, <var>y</var><sub>2</sub>) are the absolute coordinates of the final point of the arc.

This parameterization of elliptical arcs will be referred to as <em>endpoint parameterization</em>. One of the advantages of endpoint parameterization is that it permits a consistent path syntax in which all path commands end in the coordinates of the new "current point". The following notes give rules and formulas to help implementers deal with endpoint parameterization.

### <a id="ArcOutOfRangeParameters"></a>F.6.2 Out-of-range parameters

Arbitrary numerical values are permitted for all elliptical arc parameters, but where these values are invalid or out-of-range, an implementation must make sense of them as follows:

If the endpoints (<var>x</var><sub>1</sub>, <var>y</var><sub>1</sub>) and (<var>x</var><sub>2</sub>, <var>y</var><sub>2</sub>) are identical, then this is equivalent to omitting the elliptical arc segment entirely.

If <var>r<sub>x</sub></var> = 0 or <var>r<sub>y</sub></var> = 0 then this arc is treated as a straight line segment (a "lineto") joining the endpoints.

If <var>r<sub>x</sub></var> or <var>r<sub>y</sub></var> have negative signs, these are dropped; the absolute value is used instead.

If <var>r<sub>x</sub></var>, <var>r<sub>y</sub></var> and <var>φ</var> are such that there is no solution (basically, the ellipse is not big enough to reach from (<var>x</var><sub>1</sub>, <var>y</var><sub>1</sub>) to (<var>x</var><sub>2</sub>, <var>y</var><sub>2</sub>)) then the ellipse is scaled up uniformly until there is exactly one solution (until the ellipse is just big enough).

<var>φ</var> is taken mod 360 degrees.

Any nonzero value for either of the flags <var>f<sub>A</sub></var> or <var>f<sub>S</sub></var> is taken to mean the value 1.

This forgiving yet consistent treatment of out-of-range values ensures that:

- The inevitable approximations arising from computer arithmetic cannot cause a valid set of values written by one SVG implementation to be treated as invalid when read by another SVG implementation. This would otherwise be a problem for common boundary cases such as a semicircular arc.
- Continuous animations that cause parameters to pass through invalid values are not a problem. The motion remains continuous.

### <a id="ArcParameterizationAlternatives"></a>F.6.3 Parameterization alternatives

An arbitrary point (<var>x</var>, <var>y</var>) on the elliptical arc can be described by the 2-dimensional matrix equation

|                                                                                                      |           |
|------------------------------------------------------------------------------------------------------|-----------|
| ![equation F.6.3.1](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image002.png) | (F.6.3.1) |

(<var>c<sub>x</sub></var>, <var>c<sub>y</sub></var>) are the coordinates of the center of the ellipse.

<var>r<sub>x</sub></var> and <var>r<sub>y</sub></var> are the radii of the ellipse (also known as its semi-major and semi-minor axes).

<var>θ</var> is the angle from the x-axis of the current coordinate system to the x-axis of the ellipse.

<var>θ</var> ranges from:

- <var>θ</var><sub>1</sub> which is the start angle of the elliptical arc prior to the stretch and rotate operations.
- <var>θ</var><sub>2</sub> which is the end angle of the elliptical arc prior to the stretch and rotate operations.
- Δ<var>θ</var> which is the difference between these two angles.

If one thinks of an ellipse as a circle that has been stretched and then rotated, then <var>θ</var><sub>1</sub>, <var>θ</var><sub>2</sub> and Δ<var>θ</var> are the start angle, end angle and sweep angle, respectively of the arc prior to the stretch and rotate operations. This leads to an alternate parameterization which is common among graphics APIs, which will be referred to as <em>center
    parameterization</em>. In the next sections, formulas are given for mapping in both directions between center parameterization and endpoint parameterization.

### <a id="ArcConversionCenterToEndpoint"></a>F.6.4 Conversion from center to endpoint parameterization

Given the following variables:

<var>c<sub>x</sub></var> <var>c<sub>y</sub></var> <var>r<sub>x</sub></var> <var>r<sub>y</sub></var> <var>φ</var> <var>θ</var><sub>1</sub> Δ<var>θ</var>

the task is to find:

<var>x</var><sub>1</sub> <var>y</var><sub>1</sub> <var>x</var><sub>2</sub> <var>y</var><sub>2</sub> <var>f<sub>A</sub></var> <var>f<sub>S</sub></var>

This can be achieved using the following formulas:

|                                                                                                      |           |
|------------------------------------------------------------------------------------------------------|-----------|
| ![Equation F.6.4.1](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image004.png) | (F.6.4.1) |
| ![Equation F.6.4.2](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image006.png) | (F.6.4.2) |
| ![Equation F.6.4.3](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image008.png) | (F.6.4.3) |
| ![Equation F.6.4.4](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image010.png) | (F.6.4.4) |

### <a id="ArcConversionEndpointToCenter"></a>F.6.5 Conversion from endpoint to center parameterization

Given the following variables:

<var>x</var><sub>1</sub> <var>y</var><sub>1</sub> <var>x</var><sub>2</sub> <var>y</var><sub>2</sub> <var>f<sub>A</sub></var> <var>f<sub>S</sub></var> <var>r<sub>x</sub></var> <var>r<sub>y</sub></var> <var>φ</var>

the task is to find:

<var>c<sub>x</sub></var> <var>c<sub>y</sub></var> <var>θ</var><sub>1</sub> Δ<var>θ</var>

The equations simplify after a translation which places the origin at the midpoint of the line joining (<var>x</var><sub>1</sub>, <var>y</var><sub>1</sub>) to (<var>x</var><sub>2</sub>, <var>y</var><sub>2</sub>), followed by a rotation to line up the coordinate axes with the axes of the ellipse. All transformed coordinates will be written with primes. They are computed as intermediate values on the way toward finding the required center parameterization variables. This procedure consists of the following steps:

- <em>Step 1: Compute</em> (<var>x</var><sub>1</sub>′, <var>y</var><sub>1</sub>′)

  |                                                                                                      |           |
  |------------------------------------------------------------------------------------------------------|-----------|
  | ![Equation F.6.5.1](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image012.png) | (F.6.5.1) |

- <em>Step 2: Compute</em> (<var>c<sub>x</sub></var>′, <var>c<sub>y</sub></var>′)

  |                                                                                                      |           |
  |------------------------------------------------------------------------------------------------------|-----------|
  | ![Equation F.6.5.2](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image014.png) | (F.6.5.2) |

  where the + sign is chosen if <var>f<sub>A</sub></var> ≠ <var>f<sub>S</sub></var>, and the − sign is chosen if <var>f<sub>A</sub></var> = <var>f<sub>S</sub></var>.

- <em>Step 3: Compute</em> (<var>c<sub>x</sub></var>, <var>c<sub>y</sub></var>) <em>from</em> (<var>c<sub>x</sub></var>′, <var>c<sub>y</sub></var>′)

  |                                                                                                      |           |
  |------------------------------------------------------------------------------------------------------|-----------|
  | ![Equation F.6.5.3](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image016.png) | (F.6.5.3) |

- <em>Step 4: Compute</em> <var>θ</var><sub>1</sub> and Δ<var>θ</var>

  In general, the angle between two vectors (<var>u<sub>x</sub></var>, <var>u<sub>y</sub></var>) and (<var>v<sub>x</sub></var>, <var>v<sub>y</sub></var>) can be computed as

  |                                                                                                      |           |
  |------------------------------------------------------------------------------------------------------|-----------|
  | ![Equation F.6.5.4](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image018.png) | (F.6.5.4) |

  where the ± sign appearing here is the sign of <var>u<sub>x</sub></var> <var>v<sub>y</sub></var> − <var>u<sub>y</sub></var> <var>v<sub>x</sub></var>.

  This angle function can be used to express <var>θ</var><sub>1</sub> and Δ<var>θ</var> as follows:

  |                                                                                                      |           |
  |------------------------------------------------------------------------------------------------------|-----------|
  | ![Equation F.6.5.5](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image020.png) | (F.6.5.5) |
  | ![Equation F.6.5.6](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image022.png) | (F.6.5.6) |

  where <var>θ</var><sub>1</sub> is fixed in the range −360° \< Δ<var>θ</var> \< 360° such that:

  if <var>f<sub>S</sub></var> = 0, then Δ<var>θ</var> \< 0,

  else if <var>f<sub>S</sub></var> = 1, then Δ<var>θ</var> \> 0.

  In other words, if <var>f<sub>S</sub></var> = 0 and the right side of (F.6.5.6) is greater than 0, then subtract 360°, whereas if <var>f<sub>S</sub></var> = 1 and the right side of (F.6.5.6) is less than 0, then add 360°. In all other cases leave it as is.

### <a id="ArcCorrectionOutOfRangeRadii"></a>F.6.6 Correction of out-of-range radii

This section formalizes the adjustments to out-of-range <var>r<sub>x</sub></var> and <var>r<sub>y</sub></var> mentioned in F.6.2. Algorithmically these adjustments consist of the following steps:

- <em>Step 1: Ensure radii are non-zero</em>

  If <var>r<sub>x</sub></var> = 0 or <var>r<sub>y</sub></var> = 0, then treat this as a straight line from (<var>x</var><sub>1</sub>, <var>y</var><sub>1</sub>) to (<var>x</var><sub>2</sub>, <var>y</var><sub>2</sub>) and stop. Otherwise,

- <em>Step 2: Ensure radii are positive</em>

  Take the absolute value of <var>r<sub>x</sub></var> and <var>r<sub>y</sub></var>:

  |                                                                                                      |           |
  |------------------------------------------------------------------------------------------------------|-----------|
  | ![Equation F.6.6.1](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image024.png) | (F.6.6.1) |

- <em>Step 3: Ensure radii are large enough</em>

  Using the primed coordinate values of equation (F.6.5.1), compute

  |                                                                                                      |           |
  |------------------------------------------------------------------------------------------------------|-----------|
  | ![Equation F.6.6.2](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image026.png) | (F.6.6.2) |

  If the result of the above equation is less than or equal to 1, then no further change need be made to <var>r<sub>x</sub></var> and <var>r<sub>y</sub></var>. If the result of the above equation is greater than 1, then make the replacements

  |                                                                                                      |           |
  |------------------------------------------------------------------------------------------------------|-----------|
  | ![Equation F.6.6.3](https://www.w3.org/TR/2011/REC-SVG11-20110816/images/implnote/arcs/image028.png) | (F.6.6.3) |

- <em>Step 4: Proceed with computations</em>

  Proceed with the remaining elliptical arc computations, such as those in section F.6.5.  Note: As a consequence of the radii corrections in this section, equation (F.6.5.2) for the center of the ellipse always has at least one solution (i.e. the radicand is never negative).  In the case that the radii are scaled up using equation (F.6.6.3), the radicand of (F.6.5.2) is zero and there is exactly one solution for the center of the ellipse.

## <a id="TextSelectionImplementationNotes"></a>F.7 Text selection implementation notes

The following implementation notes describe the algorithm for deciding which characters are selected during a [text selection](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#TextSelection) operation.

As the text selection operation occurs (e.g., while the user clicks and drags the mouse to identify the selection), the user agent determines a <em>start selection position</em> and an <em>end selection position</em>, each of which represents a position in the text string between two characters. After determining start selection position and end selection position, the user agent selects the appropriate characters, where the resulting text selection consists of either:

- no selection or
- a <em>start character</em>, an <em>end character</em> (possibly the same character), and all of the characters within the same [‘text’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#TextElement) element whose position in the DOM is logically between the start character and end character.

On systems with pointer devices, to determine the <em>start
    selection position</em>, the SVG user agent determines which boundary between characters corresponding to rendered glyphs is the best target (e.g., closest) based on the current pointer location at the time of the event that initiates the selection operation (e.g., the mouse down event). The user agent then tracks the completion of the selection operation (e.g., the mouse drag, followed ultimately by the mouse up). At the end of the selection operation, the user agent determines which boundary between characters is the best target (e.g., closest) for the <em>end selection position</em>.

If no character reordering has occurred due to [bidirectionality](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#RelationshipWithBiDirectionality), then the selection consists of all characters between the <em>start selection position</em> and <em>end selection
    position</em>. For example, if a [‘text’](https://www.w3.org/TR/2011/REC-SVG11-20110816/text.html#TextElement) element contains the string "abcdef" and the start selection position and end selection positions are 0 and 3 respectively (assuming the left side of the "a" is position zero), then the selection will consist of "abc".

When the user agent is implementing selection of bidirectional text, and when the selection starts (or ends) between characters which are not contiguous in logical order, then there might be multiple potential combinations of characters that can be considered part of the selection. The algorithms to choose among the combinations of potential selection options shall choose the selection option which most closely matches the text string's visual rendering order.

When multiple characters map inseparably to a given set of one or more glyphs, the user agent can either disallow the selection to start in the middle of the glyph set or can attempt to allocate portions of the area taken up by the glyph set to the characters that correspond to the glyph.

For systems which support pointer devices such as a mouse, the user agent is required to provide a mechanism for selecting text even when the given text has associated event handlers or links, which might block text selection due to event processing precedence rules (see [Pointer events](https://www.w3.org/TR/2011/REC-SVG11-20110816/interact.html#PointerEvents)). One implementation option: For platforms which support a pointer device such as a mouse, the user agent may provide for a small additional region around character cells which initiates text selection operations but does not initiate event handlers or links.

## <a id="PrintingImplementationNotes"></a>F.8 Printing implementation notes

For user agents which support both zooming on display devices and printing, it is recommended that the default printing option produce printed output that reflects the display device's current view of the current SVG document fragment (assuming there is no media-specific styling), taking into account any zooming and panning done by the user, the current state of animation, and any document changes due to DOM and scripting. Thus, if the user zooms into a particular area of a map on the display device and then requests a hardcopy, the hardcopy should show the same view of the map as appears on the display device. If a user pauses an animation and prints, the hardcopy should show the same graphics as the currently paused picture on the display device. If scripting has added or removed elements from the document, then the hardcopy should reflect the same changes that would be reflected on the display.

When an SVG document is rendered on a static-only device such as a printer which does not support SVG's animation and scripting and facilities, then the user agent shall ignore any animation and scripting elements in the document and render the remaining graphics elements according to the rules in this specification.
