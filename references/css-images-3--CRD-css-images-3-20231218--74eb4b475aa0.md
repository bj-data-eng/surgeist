Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Images Module Level 3](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/).

Original copyright notice: Copyright © 2023 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Images Module Level 3

Source snapshot: https://www.w3.org/TR/2023/CRD-css-images-3-20231218/

Snapshot SHA-256: 74eb4b475aa0845a50ebedb4bbb3ba7bf3966f975817d1c5cf8e3d512f53447e

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- The 5 source tables are presented as readable Markdown tables or explicit labeled layouts: 5 ordinary table conversions. Source cell content, links and relationships are retained.
- Added table headings and layout labels are non-normative presentation aids. Source header/data roles and span models remain in the conversion checks; GFM cannot reproduce native HTML th/scope/rowspan/colspan accessibility semantics. Source row-header labels are bold where used in ordinary Markdown tables.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.
- Canonically unstable or combining Unicode characters and escape-sensitive punctuation are shielded as numeric entities in prose/semantic inline HTML. Literal source code stays literal.
- Existing external image/media URLs are resolved against the pinned source. Assets are not downloaded or availability-tested; image-only formulas/diagrams still require their source resources.

---

# <a id="title"></a>CSS Images Module Level 3

[Copyright](https://www.w3.org/policies/#copyright) © 2023 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license/) rules apply.

## <a id="abstract"></a>Abstract

<a id="ref-for-typedef-image"></a>

<a id="ref-for-url-value"></a>

This module contains the features of CSS level 3 relating to the [\<image\>](#typedef-image) type and some replaced elements. It includes and extends the functionality of CSS level 2 [\[CSS2\]](#biblio-css2). The main extensions compared to CSS2.1 are the generalization of the [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) type to the <a id="ref-for-typedef-image①"></a>\<image\> type, several additions to the <a id="ref-for-typedef-image②"></a>\<image\> type, a generic sizing algorithm for images and other replaced content in CSS, definitions for interpolating several <a id="ref-for-typedef-image③"></a>\<image\> types, and several properties controlling the interaction of replaced elements and CSS’s layout models.

[CSS](https://www.w3.org/TR/CSS/) is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.

## <a id="sotd"></a>Status of this document

<em>This section describes the status of this document at the time of its publication.
	A list of current W3C publications
	and the latest revision of this technical report
	can be found in the <a href="https://www.w3.org/TR/">W3C technical reports index at https&#58;//www&#46;w3&#46;org/TR/.</a></em>

This document was published by the [CSS Working Group](https://www.w3.org/groups/wg/css) as a <strong>Candidate Recommendation Draft</strong> using the [Recommendation track](https://www.w3.org/2023/Process-20231103/#recs-and-notes). Publication as a Candidate Recommendation does not imply endorsement by W3C and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.

This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

Please send feedback by [filing issues in GitHub](https://github.com/w3c/csswg-drafts/issues) (preferred), including the spec code “css-images” in the title, like this: “\[css-images\] <i>…summary of comment…</i>”. All issues and comments are [archived](https://lists.w3.org/Archives/Public/public-css-archive/). Alternately, feedback can be sent to the ([archived](https://lists.w3.org/Archives/Public/www-style/)) public mailing list [www-style@w3.org](mailto:www-style@w3.org?Subject=%5Bcss-images%5D%20PUT%20SUBJECT%20HERE).

<a id="w3c_process_revision"></a>

This document is governed by the [03 November 2023 W3C Process Document](https://www.w3.org/2023/Process-20231103/).

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/css/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20200915/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20200915/#sec-Disclosure).

The following features are at-risk, and may be dropped during the CR period:

- <a id="ref-for-propdef-image-orientation"></a>

  [image-orientation](#propdef-image-orientation)

“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.

## <a id="intro"></a>1. Introduction

<a id="ref-for-propdef-background-image"></a>

In CSS Levels 1 and 2, image values, such as those used in the [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) property, could only be given by a single URL value. This module introduces additional ways of representing 2D images, for example as [a gradient](#gradients).

This module also defines several properties for [manipulating raster images](#image-processing) and for [sizing](#the-object-fit) or [positioning](#the-object-position) replaced elements such as images within the box determined by the CSS layout algorithms. It also defines in a generic way CSS’s [sizing algorithm](#sizing) for images and other similar replaced elements.

<em>This subsection (above) is not normative.</em>

### <a id="placement"></a>1.1. Module Interactions

<a id="ref-for-typedef-image④"></a>

<a id="ref-for-url-value①"></a>

<a id="ref-for-propdef-background-image①"></a>

<a id="ref-for-propdef-cursor"></a>

<a id="ref-for-propdef-list-style-image"></a>

<a id="ref-for-propdef-content"></a>

This module defines and extends the [\<image\>](#typedef-image) value type defined in [\[CSS-VALUES-3\]](#biblio-css-values-3). It also replaces the [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) type with <a id="ref-for-typedef-image⑤"></a>\<image\> in the [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image), [cursor](https://www.w3.org/TR/css-ui-4/#propdef-cursor), and [list-style-image](https://www.w3.org/TR/css-lists-3/#propdef-list-style-image) definitions in CSS1 and CSS2 and adds <a id="ref-for-typedef-image⑥"></a>\<image\> as an alternative to <a id="ref-for-url-value②"></a>\<url\> in the [content](https://www.w3.org/TR/css-content-3/#propdef-content) property’s value. It is presumed that CSS specifications beyond CSS2.1 will use the <a id="ref-for-typedef-image⑦"></a>\<image\> notation in place of <a id="ref-for-url-value③"></a>\<url\> where 2D images are expected. (See e.g. [\[CSS3BG\]](#biblio-css3bg).)

<a id="ref-for-propdef-image-rendering"></a>

None of the properties defined in this module, only [image-rendering](#propdef-image-rendering) applies to `::first-line` and `::first-letter`.

### <a id="values"></a>1.2. Value Definitions

<a id="ref-for-valdef-all-initial"></a>

This specification follows the [CSS property definition conventions](https://www.w3.org/TR/CSS2/about.html#property-defs) from [\[CSS2\]](#biblio-css2). using the [value definition syntax](https://www.w3.org/TR/css-values-3/#value-defs) from [\[CSS-VALUES-3\]](#biblio-css-values-3). Value types not defined in this specification are defined in CSS Level 2 Revision 1 \[CSS2\]. Other CSS modules may expand the definitions of these value types: for example \[CSS-VALUES-3\], when combined with this module, adds the [initial](https://www.w3.org/TR/css-cascade-5/#valdef-all-initial) keyword as a possible property value.

<a id="ref-for-css-wide-keywords"></a>

In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the [CSS-wide keywords](https://www.w3.org/TR/css-values-4/#css-wide-keywords) as their property value. For readability they have not been repeated explicitly.

<a id="ref-for-typedef-image⑧"></a>

## <a id="image-values"></a>2. Image Values: the [\<image\>](#typedef-image) type

<a id="ref-for-typedef-image⑨"></a>

The [\<image\>](#typedef-image) value type denotes a 2D image. It can be a [url reference](#url-notation) or a [color gradient](#gradients). Its syntax is:

<a id="typedef-image"></a>

<a id="ref-for-url-value④"></a>

<a id="ref-for-comb-one"></a>

<a id="ref-for-typedef-gradient"></a>

```text
<image> = <url> | <gradient>
```
<a id="ref-for-typedef-image①⓪"></a>

<a id="ref-for-propdef-background-image②"></a>

<a id="ref-for-propdef-list-style-image①"></a>

<a id="ref-for-propdef-cursor①"></a>

<a id="ref-for-url-value⑤"></a>

An [\<image\>](#typedef-image) can be used in many CSS properties, including the [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image), [list-style-image](https://www.w3.org/TR/css-lists-3/#propdef-list-style-image), [cursor](https://www.w3.org/TR/css-ui-4/#propdef-cursor) properties [\[CSS2\]](#biblio-css2) (where it replaces the [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) component in the property’s value).

<a id="ref-for-url-value⑥"></a>

<a id="ref-for-valdef-color-transparent"></a>

<a id="ref-for-natural-dimensions"></a>

<a id="ref-for-invalid-image"></a>

<a id="ref-for-propdef-list-style-image②"></a>

<a id="ref-for-valdef-list-style-type-none"></a>

<a id="ref-for-propdef-list-style-type"></a>

In some cases an image is invalid, such as a [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) pointing to a resource that is not a valid image format or that has failed to load. An <a id="invalid-image"></a>invalid image is rendered as a solid-color [transparent](https://www.w3.org/TR/css-color-4/#valdef-color-transparent) image with no [natural dimensions](#natural-dimensions). However, [invalid images](#invalid-image) can trigger error-handling clauses in some contexts. For example, an <a id="ref-for-invalid-image①"></a>invalid image in [list-style-image](https://www.w3.org/TR/css-lists-3/#propdef-list-style-image) it is treated as [none](https://www.w3.org/TR/css-lists-3/#valdef-list-style-type-none), allowing the [list-style-type](https://www.w3.org/TR/css-lists-3/#propdef-list-style-type) to render in its place. [\[CSS2\]](#biblio-css2)

<a id="ref-for-loading-image"></a>

<a id="ref-for-invalid-image②"></a>

<a id="ref-for-valdef-color-transparent①"></a>

<a id="ref-for-natural-dimensions①"></a>

While an image is loading, is a <a id="loading-image"></a>loading image. [Loading images](#loading-image) are <em>not</em> [invalid images](#invalid-image), but have similar behavior: they are rendered as a solid-color [transparent](https://www.w3.org/TR/css-color-4/#valdef-color-transparent) image with no [natural dimensions](#natural-dimensions), and may trigger fallback rendering in contexts that offer it, but must not trigger loading of fallback resources. Alternately, if a <a id="ref-for-loading-image①"></a>loading image happens to be replacing an already-loaded image (for example due to changes in the document or style sheet) and the UA is tracking this information, it may continue to render the already-loaded image in place of the <a id="ref-for-loading-image②"></a>loading image.

<a id="ref-for-natural-dimensions②"></a>

<a id="ref-for-loading-image③"></a>

Partially-loaded images (whose [natural dimensions](#natural-dimensions) are known, but whose image data is not fully loaded) may be either treated as [loading images](#loading-image) or as loaded images rendered with partial data. For example, a UA may render an interlaced GIF in place as soon as its first pass of pixel data has loaded or even as soon as the image header (which contains sizing data) has parsed and refresh the rendering as more data loads; or it may wait until the entire image has loaded before using it.

<a id="ref-for-computed-value"></a>

<a id="ref-for-typedef-image①①"></a>

<a id="ref-for-specified-value"></a>

<a id="ref-for-url-value⑦"></a>

<a id="ref-for-typedef-color"></a>

<a id="ref-for-length-value"></a>

A <a id="computed-image"></a>[computed](https://www.w3.org/TR/css-cascade-5/#computed-value) [\<image\>](#typedef-image) value is the [specified value](https://www.w3.org/TR/css-cascade-5/#specified-value) with any [\<url\>](https://www.w3.org/TR/css-values-4/#url-value)s, [\<color\>](https://www.w3.org/TR/css-color-4/#typedef-color)s, and [\<length\>](https://www.w3.org/TR/css-values-4/#length-value)s computed.

<a id="ref-for-funcdef-url"></a>

### <a id="url-notation"></a>2.1. Image References: the [url()](https://www.w3.org/TR/css-values-4/#funcdef-url) notation

<a id="ref-for-funcdef-url①"></a>

The simplest way to indicate an image is to reference an image file by URL. This can be done with the [url()](https://www.w3.org/TR/css-values-4/#funcdef-url) notation, defined in [\[CSS-VALUES-3\]](#biblio-css-values-3).

<a id="ref-for-funcdef-url②"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-4bc4e081"></a> In the example below, a background image is specified with [url()](https://www.w3.org/TR/css-values-4/#funcdef-url)syntax:
>
> ```text
> background-image: url(wavy.png);
> ```
<a id="ref-for-invalid-image③"></a>

If the UA cannot download, parse, or otherwise successfully display the contents at the URL as an image (i.e. if the image is not fully [fully decodable](https://html.spec.whatwg.org/multipage/webappapis.html#concept-imagebitmap-good)) it must be treated as an [invalid image](#invalid-image).

#### <a id="ambiguous-urls"></a>2.1.1. Ambiguous Reference-or-Image URLs

<a id="ref-for-propdef-mask-image"></a>

<a id="ref-for-url-value⑧"></a>

<a id="ref-for-typedef-image①②"></a>

URLs are used in many contexts for many types of resources, and therefore can be interpreted in many ways. Usually the context the URL appears in makes it clear how to interpret the resource, but in some instances it can be ambiguous. For example, a [mask-image](https://www.w3.org/TR/css-masking-1/#propdef-mask-image) [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) value pointing to an SVG file could be interpreted as a reference to an element in the file or as an [\<image\>](#typedef-image).

<a id="ref-for-url-value⑨"></a>

<a id="ref-for-typedef-image①③"></a>

<a id="ref-for-css-ambiguous-image-url"></a>

<a id="ref-for-elementdef-mask"></a>

<a id="ref-for-propdef-mask-image①"></a>

An <a id="css-ambiguous-image-url"></a>ambiguous image URL is a [\<url\>](https://www.w3.org/TR/css-values-4/#url-value) value that can be interpreted as either an [\<image\>](#typedef-image) or an element reference. If an [ambiguous image URL](#css-ambiguous-image-url) is a [fragment-only URL](https://www.w3.org/TR/css-values-3/#local-urls), then it must be treated as an element reference. Otherwise, if the <a id="ref-for-css-ambiguous-image-url①"></a>ambiguous image URL has a fragment that references an element in the resource that is an appropriate type of element for the context in which the <a id="ref-for-url-value①⓪"></a>\<url\> appears (such as a <code><a href="https://www.w3.org/TR/css-masking-1/#elementdef-mask">mask</a></code> element for the [mask-image](https://www.w3.org/TR/css-masking-1/#propdef-mask-image) property), it is interpreted as an element reference. Otherwise, it is treated as an <a id="ref-for-typedef-image①④"></a>\<image\>.

<a id="ref-for-css-ambiguous-image-url②"></a>

Specs using the [ambiguous image URL](#css-ambiguous-image-url) concept must define what elements are valid references for the URL, and any additional conditions that might apply.

<a id="ref-for-propdef-mask-image②"></a>

<a id="ref-for-target-pseudo"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-73bb4dd5"></a> For example, a reference like [mask-image: url(icon.svg#foo)](https://www.w3.org/TR/css-masking-1/#propdef-mask-image) might be pointing to a `<mask id="foo">` element in the SVG document, <em>or</em> be pointing to a `<g id="foo">` element and depending on the [:target](https://www.w3.org/TR/selectors-4/#target-pseudo) pseudo-class to change how it renders as an image.
>
> <a id="ref-for-elementdef-mask①"></a>
>
> <a id="ref-for-funcdef-url③"></a>
>
> When this occurs, the "icon.svg" file is loaded up and examined; if the \#foo element is indeed a <code><a href="https://www.w3.org/TR/css-masking-1/#elementdef-mask">mask</a></code>, the [url()](https://www.w3.org/TR/css-values-4/#funcdef-url) is treated as a reference to that element; otherwise, it’s interpreted as an image.

#### <a id="url-metadata"></a>2.1.2. Image Metadata

Images can contain metadata such as resolution and orientation which specifies how to render the image. Some image formats are flexible in where this metadata can be placed in the file; however, if the metadata occurs <em>after</em> the actual image data, it harms the UA’s ability to “progressively decode” the image and display it as the image’s data streams in.

To reduce the impact of this issue:

- Authors <em>must</em> produce their image files so that such metadata occurs before the image data in the image file. (Note: This is the default for most images already.)

- User agents <em>should</em> ignore any layout-impacting metadata (such as orientation or resolution) that occurs after the image data begins in the file. (Note: This rule does not impact metadata that does not affect layout, such as color space information.)

  If a user agent cannot ignore the metadata based its location in the file (for example, if the decoder being used does not report where in the file the metadata was located), it <em>must</em> use the metadata in all cases. (In particular, it is not valid to use the metadata only when the image is "small" and the entire file is downloaded quickly, but to ignore it if the image is large and the metadata isn’t downloaded until well after the image starts being displayed.)

## <a id="gradients"></a>3. Gradients

<a id="ref-for-typedef-gradient①"></a>

A gradient is an image that smoothly fades from one color to another. These are commonly used for subtle shading in background images, buttons, and many other things. The <a id="gradient-function"></a>gradient functions described in this section allow an author to specify such an image in a terse syntax, so that the UA can generate the image automatically when rendering the page. The syntax of a [\<gradient\>](#typedef-gradient) is:

<a id="typedef-gradient"></a>

<a id="ref-for-funcdef-linear-gradient"></a>

<a id="ref-for-comb-one①"></a>

<a id="ref-for-funcdef-repeating-linear-gradient"></a>

<a id="ref-for-comb-one②"></a>

<a id="ref-for-funcdef-radial-gradient"></a>

<a id="ref-for-comb-one③"></a>

<a id="ref-for-funcdef-repeating-radial-gradient"></a>

```text
<gradient> =
  <linear-gradient()> | <repeating-linear-gradient()> |
  <radial-gradient()> | <repeating-radial-gradient()>
```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-3bdc7fad"></a>
>
> <a id="ref-for-typedef-image①⑤"></a>
>
> As with the other [\<image\>](#typedef-image) types defined in this specification, gradients can be used in any property that accepts images. For example:
>
> - `background: linear-gradient(white, gray);`
>
> - `list-style-image: radial-gradient(circle, #006, #00a 90%, #0000af 100%, white 100%)`

<a id="ref-for-concrete-object-size"></a>

<a id="ref-for-natural-dimensions③"></a>

A gradient is drawn into a box with the dimensions of the [concrete object size](#concrete-object-size), referred to as the <a id="gradient-box"></a>gradient box. However, the gradient itself has no [natural dimensions](#natural-dimensions).

<a id="ref-for-gradient-box"></a>

<a id="ref-for-propdef-background-size"></a>

<a id="ref-for-propdef-list-style-image③"></a>

<a id="ref-for-default-object-size"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-aa187bec"></a> For example, if you use a gradient as a background, by default the gradient will draw into a [gradient box](#gradient-box) the size of the element’s padding box. If [background-size](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-size) is explicitly set to a value such as 100px 200px, then the <a id="ref-for-gradient-box①"></a>gradient box will be 100px wide and 200px tall. Similarly, for a gradient used as a [list-style-image](https://www.w3.org/TR/css-lists-3/#propdef-list-style-image), the box would be a 1em square, which is the [default object size](#default-object-size) for that property.

<a id="ref-for-gradient-line"></a>

Gradients are specified by defining the <a id="starting-point"></a>starting point and <a id="ending-point"></a>ending point of a <a id="gradient-line"></a>gradient line (which, depending on the type of gradient, may geometrically be a line, or a ray, or a spiral), and then specifying colors at points along this line. The colors are smoothly blended to fill in the rest of the line, and then each type of gradient defines how to use the color of the [gradient line](#gradient-line) to produce the actual gradient.

<a id="ref-for-funcdef-linear-gradient①"></a>

### <a id="linear-gradients"></a>3.1. Linear Gradients: the [linear-gradient()](#funcdef-linear-gradient) notation

<a id="ref-for-gradient-line①"></a>

A linear gradient is created by specifying a straight [gradient line](#gradient-line), and then several colors placed along that line. The image is constructed by creating an infinite canvas and painting it with lines perpendicular to the gradient line, with the color of the painted line being the color of the gradient line where the two intersect. This produces a smooth fade from each color to the next, progressing in the specified direction.

#### <a id="linear-gradient-syntax"></a>3.1.1. linear-gradient() syntax

The <a id="funcdef-linear-gradient"></a>linear-gradient() notation specifies a linear gradient in CSS. Its syntax is as follows:

<a id="ref-for-funcdef-linear-gradient②"></a>

<a id="ref-for-typedef-linear-gradient-syntax"></a>

<a id="typedef-linear-gradient-syntax"></a>

<a id="ref-for-angle-value"></a>

<a id="ref-for-comb-one④"></a>

<a id="ref-for-typedef-side-or-corner"></a>

<a id="ref-for-mult-opt"></a>

<a id="ref-for-comb-comma"></a>

<a id="ref-for-typedef-color-stop-list"></a>

<a id="typedef-side-or-corner"></a>

<a id="ref-for-comb-one⑤"></a>

<a id="ref-for-comb-any"></a>

<a id="ref-for-comb-one⑥"></a>

```text
<linear-gradient()> = linear-gradient( [ <linear-gradient-syntax> ] )

<linear-gradient-syntax> = [ <angle> | to <side-or-corner> ]? , <color-stop-list>

<side-or-corner> = [left | right] || [top | bottom]
```
<a id="ref-for-gradient-line②"></a>

The first argument to the function specifies the [gradient line](#gradient-line), which gives the gradient a direction and determines how color-stops are positioned. It may be omitted; if so, it defaults to to bottom.

<a id="ref-for-gradient-line③"></a>

The [gradient line’s](#gradient-line) direction may be specified in two ways:

<a id="ref-for-angle-value①"></a>

using [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value)

For the purpose of this argument, 0deg points upward, and positive angles represent clockwise rotation, so 90deg point toward the right.

<a id="ref-for-angle-value②"></a>

The unit identifier may be omitted if the [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) is zero.

using keywords

<a id="ref-for-gradient-line④"></a>

If the argument is to top, to right, to bottom, or to left, the angle of the [gradient line](#gradient-line) is 0deg, 90deg, 180deg, or 270deg, respectively.

<a id="ref-for-gradient-line⑤"></a>

<a id="ref-for-gradient-box②"></a>

If the argument instead specifies a corner of the box such as to top left, the [gradient line](#gradient-line) must be angled such that it points into the same quadrant as the specified corner, and is perpendicular to a line intersecting the two neighboring corners of the [gradient box](#gradient-box). <strong data-conversion-semantic="note">Note:</strong> This causes a color-stop at 50% to intersect the two neighboring corners (see [example](#corner-gradient-example)).

<a id="ref-for-gradient-box③"></a>

<a id="ref-for-gradient-line⑥"></a>

Starting from the center of the [gradient box](#gradient-box), extend a line at the specified angle in both directions. The ending point is the point on the [gradient line](#gradient-line) where a line drawn perpendicular to the <a id="ref-for-gradient-line⑦"></a>gradient line would intersect the corner of the <a id="ref-for-gradient-box④"></a>gradient box in the specified direction. The starting point is determined identically, but in the opposite direction.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is expected that the next level of this module will provide the ability to define the gradient’s direction relative to the current text direction and writing-mode.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b6f5099c"></a>
>
> ![\[An image showing a box with a background shading gradually from white in the bottom-left corner to black in the top-right corner. There is a line, illustrating the gradient line, angled at 45 degrees and passing through the center of the box. The starting point and ending point of the gradient line are indicated by the intersection of the gradient line with two additional lines that pass through the bottom-left and top-right corners of the box.\]](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/gradient-diagram.png)
>
> <a id="ref-for-gradient-line⑧"></a>
>
> <a id="ref-for-propdef-background"></a>
>
> This example illustrates visually how to calculate the [gradient line](#gradient-line) from the rules above. This shows the starting and ending point of the <a id="ref-for-gradient-line⑨"></a>gradient line, along with the actual gradient, produced by an element with [background: linear-gradient(45deg, white, black);](https://www.w3.org/TR/css-backgrounds-3/#propdef-background).
>
> Notice how, though the starting point and ending point are outside of the box, they’re positioned precisely right so that the gradient is pure white <em>exactly</em> at the corner, and pure black <em>exactly</em> at the opposite corner. That’s intentional, and will always be true for linear gradients.

> <strong data-conversion-semantic="note">Note</strong>
>
> Given:
>
> - <var>A</var> the angle (in any quadrant) defining the gradient line’s direction such that 0 degrees points upwards and positive angles represent clockwise rotation,
>
> - <var>W</var> the width of the gradient box,
>
> - <var>H</var> the height of the gradient box,
>
> <a id="ref-for-starting-point"></a>
>
> <a id="ref-for-ending-point"></a>
>
> The length of the gradient line (between the [starting point](#starting-point) and [ending point](#ending-point)) is:
>
> <code>abs(<var>W</var> &#x2A; sin(<var>A</var>)) + abs(<var>H</var> &#x2A; cos(<var>A</var>))</code>

<a id="ref-for-starting-point①"></a>

<a id="ref-for-ending-point①"></a>

<a id="ref-for-gradient-line①⓪"></a>

The gradient’s color stops are typically placed between the [starting point](#starting-point) and [ending point](#ending-point) on the [gradient line](#gradient-line), but this isn’t required: the <a id="ref-for-gradient-line①①"></a>gradient line extends infinitely in both directions. The starting point and ending point are merely arbitrary location markers, the starting point defining where 0%, 0px, etc are located when specifying color-stops, and the ending point defines where 100% is located. Color-stops are allowed to have positions before 0% or after 100%.

<a id="ref-for-gradient-line①②"></a>

The color of a linear gradient at any point is determined by finding the unique line passing through that point that is perpendicular to the [gradient line](#gradient-line). The point’s color is the color of the <a id="ref-for-gradient-line①③"></a>gradient line at the point where this line intersects it.

#### <a id="linear-gradient-examples"></a>3.1.2. Linear Gradient Examples

<a id="ref-for-funcdef-linear-gradient③"></a>

All of the following [linear-gradient()](#funcdef-linear-gradient) examples are presumed to be backgrounds applied to a box that is 200px wide and 100px tall.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-369769bb"></a> Below are various ways of specifying a basic vertical gradient:
>
> ```text
> linear-gradient(yellow, blue);
> linear-gradient(to bottom, yellow, blue);
> linear-gradient(180deg, yellow, blue);
> linear-gradient(to top, blue, yellow);
> linear-gradient(to bottom, yellow 0%, blue 100%);
> ```
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/linear1.png)

<a id="ref-for-gradient-line①④"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-ef19999d"></a> This demonstrates the use of an angle in the gradient. Note that, though the angle is not exactly the same as the angle between the corners, the [gradient line](#gradient-line) is still sized so as to make the gradient yellow exactly at the upper-left corner, and blue exactly at the lower-right corner.
>
> ```text
> linear-gradient(135deg, yellow, blue);
> linear-gradient(-45deg, blue, yellow);
> ```
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/linear3.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-08549192"></a>
>
> This demonstrates a 3-color gradient, and how to specify the location of a stop explicitly:
>
> ```text
> linear-gradient(yellow, blue 20%, #0f0);
> ```
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/linear4.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="corner-gradient-example"></a> This demonstrates a corner-to-corner gradient specified with keywords. Note how the gradient is red and blue exactly in the bottom-left and top-right corners, respectively, exactly like the second example. Additionally, the angle of the gradient is automatically computed so that the color at 50% (in this case, white) stretches across the top-left and bottom-right corners.
>
> ```text
> linear-gradient(to top right, red, white, blue)
> ```
>
> (Image requires SVG)
>
> ![(Image requires SVG)](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/gradient1.svg)

<a id="ref-for-funcdef-radial-gradient①"></a>

### <a id="radial-gradients"></a>3.2. Radial Gradients: the [radial-gradient()](#funcdef-radial-gradient) notation

<a id="ref-for-gradient-box⑤"></a>

In a radial gradient, rather than colors smoothly fading from one side of the [gradient box](#gradient-box) to the other as with linear gradients, they instead emerge from a single point and smoothly spread outward in a circular or elliptical shape.

<a id="ref-for-funcdef-linear-gradient④"></a>

<a id="ref-for-radial-gradient-gradient-center"></a>

<a id="ref-for-ending-shape"></a>

The <a id="funcdef-radial-gradient"></a>radial-gradient() notation specifies a radial gradient by indicating the center of the gradient (where the 0% ellipse will be) and the size and shape of the <a id="ending-shape"></a>ending shape (the 100% ellipse). Color stops are given as a list, just as for [linear-gradient()](#funcdef-linear-gradient). Starting from the [gradient center](#radial-gradient-gradient-center) and progressing towards (and potentially beyond) the [ending shape](#ending-shape), uniformly-scaled concentric ellipses are drawn and colored according to the specified color stops.

#### <a id="radial-gradient-syntax"></a>3.2.1. radial-gradient() Syntax

The radial gradient syntax is:

<a id="ref-for-funcdef-radial-gradient②"></a>

<a id="ref-for-typedef-radial-gradient-syntax"></a>

<a id="typedef-radial-gradient-syntax"></a>

<a id="ref-for-typedef-radial-shape"></a>

<a id="ref-for-comb-any①"></a>

<a id="ref-for-typedef-radial-size"></a>

<a id="ref-for-mult-opt①"></a>

<a id="ref-for-typedef-position"></a>

<a id="ref-for-mult-opt②"></a>

<a id="ref-for-comb-comma①"></a>

<a id="ref-for-typedef-color-stop-list①"></a>

<a id="typedef-radial-size"></a>

<a id="ref-for-typedef-radial-extent"></a>

<a id="ref-for-comb-one⑦"></a>

<a id="ref-for-length-value①"></a>

<a id="ref-for-comb-one⑧"></a>

<a id="ref-for-typedef-length-percentage"></a>

<a id="ref-for-mult-num"></a>

<a id="typedef-radial-extent"></a>

<a id="ref-for-comb-one⑨"></a>

<a id="ref-for-comb-one①⓪"></a>

<a id="ref-for-comb-one①①"></a>

<a id="typedef-radial-shape"></a>

<a id="ref-for-comb-one①②"></a>

```text
<radial-gradient()> = radial-gradient( [ <radial-gradient-syntax> ] )

<radial-gradient-syntax> =
  [ <radial-shape> || <radial-size> ]? [ at <position> ]? ,
  <color-stop-list>

<radial-size> = <radial-extent> | <length [0,∞]> | <length-percentage [0,∞]>{2}

<radial-extent> = closest-corner | closest-side | farthest-corner | farthest-side

<radial-shape> = circle | ellipse
```
> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-c8f2f477"></a> Here is an example of a circular radial gradient 5em wide and positioned with its center in the top left corner:
>
> ```text
> radial-gradient(5em circle at top left, yellow, blue)
> ```
<a id="ref-for-typedef-position①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: A future level may add the ability to move the focus of the gradient, as in the original -webkit-gradient() function. See [proposal](https://lists.w3.org/Archives/Public/www-style/2011Nov/0210.html) tracked in [Issue 1575](https://github.com/w3c/csswg-drafts/issues/1575) for "from [\<position\>](https://www.w3.org/TR/css-values-4/#typedef-position)" and "from offset \<offset\>".

The arguments are defined as follows:

<a id="ref-for-typedef-position②"></a>

<a id="valdef-radial-gradient-position"></a>[\<position\>](https://www.w3.org/TR/css-values-4/#typedef-position)

<a id="ref-for-gradient-box⑥"></a>

<a id="ref-for-propdef-background-position"></a>

<a id="ref-for-typedef-position③"></a>

Determines the <a id="radial-gradient-gradient-center"></a>center of the gradient. The [\<position\>](https://www.w3.org/TR/css-values-4/#typedef-position) value type (which is also used for [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position)) is defined in [\[CSS-VALUES-3\]](#biblio-css-values-3), and is resolved using the center-point as the object area and the [gradient box](#gradient-box) as the positioning area. If this argument is omitted, it defaults to center.

<a id="ref-for-typedef-radial-shape①"></a>

<a id="valdef-radial-gradient-radial-shape"></a>[\<radial-shape\>](#typedef-radial-shape)

<a id="ref-for-length-value②"></a>

<a id="ref-for-typedef-radial-size①"></a>

<a id="ref-for-typedef-radial-shape②"></a>

<a id="ref-for-ending-shape①"></a>

Can be either <a id="valdef-radial-shape-circle"></a>circle or <a id="valdef-radial-shape-ellipse"></a>ellipse; determines whether the gradient’s [ending shape](#ending-shape) is a circle or an ellipse, respectively. If [\<radial-shape\>](#typedef-radial-shape) is omitted, the <a id="ref-for-ending-shape②"></a>ending shape defaults to a circle if the [\<radial-size\>](#typedef-radial-size) is a single [\<length\>](https://www.w3.org/TR/css-values-4/#length-value), and to an ellipse otherwise.

<a id="ref-for-typedef-radial-size②"></a>

<a id="valdef-radial-gradient-radial-size"></a>[\<radial-size\>](#typedef-radial-size)

<a id="ref-for-gradient-box⑦"></a>

<a id="ref-for-valdef-radial-extent-farthest-corner"></a>

<a id="ref-for-ending-shape③"></a>

Determines the size of the gradient’s [ending shape](#ending-shape). If omitted it defaults to [farthest-corner](#valdef-radial-extent-farthest-corner). It can be given explicitly or by keyword. For the purpose of the keyword definitions, consider the [gradient box](#gradient-box) edges as extending infinitely in both directions, rather than being finite line segments.

If the ending-shape is an ellipse, its axises are aligned with the horizontal and vertical axises.

<a id="ref-for-valdef-radial-shape-circle"></a>

<a id="ref-for-valdef-radial-shape-ellipse"></a>

<a id="ref-for-typedef-radial-extent①"></a>

Both [circle](#valdef-radial-shape-circle) and [ellipse](#valdef-radial-shape-ellipse) gradients accept the following [\<radial-extent\>](#typedef-radial-extent) values:

<a id="valdef-radial-extent-closest-side"></a>closest-side  
<a id="ref-for-gradient-box⑧"></a>

<a id="ref-for-ending-shape④"></a>

The [ending shape](#ending-shape) is sized so that it exactly meets the side of the [gradient box](#gradient-box) closest to the gradient’s center. If the shape is an ellipse, it exactly meets the closest side in each dimension.

<a id="valdef-radial-extent-farthest-side"></a>farthest-side  
<a id="ref-for-ending-shape⑤"></a>

<a id="ref-for-valdef-radial-extent-closest-side"></a>

Same as [closest-side](#valdef-radial-extent-closest-side), except the [ending shape](#ending-shape) is sized based on the farthest side(s).

<a id="valdef-radial-extent-closest-corner"></a>closest-corner  
<a id="ref-for-valdef-radial-extent-closest-side①"></a>

<a id="ref-for-gradient-box⑨"></a>

<a id="ref-for-ending-shape⑥"></a>

The [ending shape](#ending-shape) is sized so that it passes through the corner of the [gradient box](#gradient-box) closest to the gradient’s center. If the shape is an ellipse, the <a id="ref-for-ending-shape⑦"></a>ending shape is given the same aspect-ratio it would have if [closest-side](#valdef-radial-extent-closest-side) were specified.

<a id="valdef-radial-extent-farthest-corner"></a>farthest-corner  
<a id="ref-for-valdef-radial-extent-farthest-side"></a>

<a id="ref-for-ending-shape⑧"></a>

<a id="ref-for-valdef-radial-extent-closest-corner"></a>

Same as [closest-corner](#valdef-radial-extent-closest-corner), except the [ending shape](#ending-shape) is sized based on the farthest corner. If the shape is an ellipse, the <a id="ref-for-ending-shape⑨"></a>ending shape is given the same aspect ratio it would have if [farthest-side](#valdef-radial-extent-farthest-side) were specified.

<a id="ref-for-typedef-radial-shape③"></a>

<a id="ref-for-valdef-radial-shape-circle①"></a>

<a id="ref-for-typedef-radial-size③"></a>

If [\<radial-shape\>](#typedef-radial-shape) is specified as [circle](#valdef-radial-shape-circle) or is omitted, the [\<radial-size\>](#typedef-radial-size) may be given explicitly as:

<a id="valdef-radial-size-length-0"></a>\<length \[0,∞\]\>  
Gives the radius of the circle explicitly. Negative values are invalid.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Percentages are <em>not</em> allowed here; they can only be used to specify the size of an elliptical gradient, not a circular one. This restriction exists because there is are multiple reasonable answers as to which dimension the percentage should be relative to. A future level of this module may provide the ability to size circles with percentages, perhaps with more explicit controls over which dimension is used.

<a id="ref-for-typedef-radial-shape④"></a>

<a id="ref-for-valdef-radial-shape-ellipse①"></a>

<a id="ref-for-typedef-radial-size④"></a>

If [\<radial-shape\>](#typedef-radial-shape) is specified as [ellipse](#valdef-radial-shape-ellipse) or is omitted, [\<radial-size\>](#typedef-radial-size) may instead be given explicitly as:

<a id="ref-for-typedef-length-percentage①"></a>

<a id="valdef-radial-size-length-percentage-0-2"></a>[\<length-percentage \[0,∞\]\>](https://www.w3.org/TR/css-values-4/#typedef-length-percentage){2}

<a id="ref-for-gradient-box①⓪"></a>

Gives the size of the ellipse explicitly. The first value represents the horizontal radius, the second the vertical radius. Percentages values are relative to the corresponding dimension of the [gradient box](#gradient-box). Negative values are invalid.

> <strong data-conversion-semantic="note">Note</strong>
>
> Expanded with the above definitions, the grammar becomes:
>
> <a id="ref-for-length-value③"></a>
>
> <a id="ref-for-typedef-position④"></a>
>
> <a id="ref-for-typedef-length-percentage②"></a>
>
> <a id="ref-for-typedef-position⑤"></a>
>
> <a id="ref-for-typedef-radial-extent②"></a>
>
> <a id="ref-for-typedef-position⑥"></a>
>
> <a id="ref-for-typedef-position⑦"></a>
>
> <a id="ref-for-typedef-color-stop-list②"></a>
>
> ```text
> radial-gradient() = radial-gradient(
>   [ [ circle               || <length [0,∞]> ]                          [ at <position> ]? , |
>     [ ellipse              || <length-percentage [0,∞]>{2} ]            [ at <position> ]? , |
>     [ [ circle | ellipse ] || <radial-extent> ]                     [ at <position> ]? , |
>     at <position> ,
>   ]?
>   <color-stop-list>
> )
> ```
#### <a id="radial-color-stops"></a>3.2.2. Placing Color Stops

<a id="ref-for-gradient-line①⑤"></a>

<a id="ref-for-starting-point②"></a>

<a id="ref-for-ending-point②"></a>

<a id="ref-for-ending-shape①⓪"></a>

Color-stops are placed on a [gradient line](#gradient-line) shaped like a ray (a line that starts at one point, and extends infinitely in a one direction), similar to the <a id="ref-for-gradient-line①⑥"></a>gradient line of linear gradients. The <a id="ref-for-gradient-line①⑦"></a>gradient line’s [starting point](#starting-point) is at the center of the gradient, and it extends toward the right, with the [ending point](#ending-point) on the point where the <a id="ref-for-gradient-line①⑧"></a>gradient line intersects the [ending shape](#ending-shape). A color-stop can be placed at a location before 0%; though the negative region of the <a id="ref-for-gradient-line①⑨"></a>gradient line is never directly consulted for rendering, color stops placed there can affect the color of non-negative locations on the <a id="ref-for-gradient-line②⓪"></a>gradient line through interpolation or repetition (see [repeating gradients](#repeating-gradients)). For example, radial-gradient(red -50px, yellow 100px) produces an elliptical gradient that starts with a reddish-orange color in the center (specifically, \#f50) and transitions to yellow. Locations greater than 100% simply specify a location a correspondingly greater distance from the center of the gradient.

<a id="ref-for-gradient-line②①"></a>

The color of the gradient at any point is determined by first finding the unique ellipse passing through that point with the same center, orientation, and ratio between major and minor axises as the ending-shape. The point’s color is then the color of the positive section of the [gradient line](#gradient-line) at the location where this ellipse intersects it.

#### <a id="degenerate-radials"></a>3.2.3. Degenerate Radial Gradients

<a id="ref-for-gradient-box①①"></a>

<a id="ref-for-valdef-radial-extent-closest-side②"></a>

<a id="ref-for-valdef-radial-extent-closest-corner①"></a>

Some combinations of position, size, and shape will produce a circle or ellipse with a radius of 0. This will occur, for example, if the center is on a [gradient box](#gradient-box) edge and [closest-side](#valdef-radial-extent-closest-side) or [closest-corner](#valdef-radial-extent-closest-corner) is specified or if the size and shape are given explicitly and either of the radiuses is zero. In these degenerate cases, the gradient must be rendered as follows:

<a id="ref-for-ending-shape①①"></a>

If the [ending shape](#ending-shape) is a circle with zero radius:

<a id="ref-for-ending-shape①②"></a>

Render as if the [ending shape](#ending-shape) was a circle whose radius was an arbitrary very small number greater than zero. <strong data-conversion-semantic="note">Note:</strong> This will make the gradient continue to look like a circle.

<a id="ref-for-ending-shape①③"></a>

If the [ending shape](#ending-shape) has zero width (regardless of the height):

<a id="ref-for-ending-shape①④"></a>

Render as if the [ending shape](#ending-shape) was an ellipse whose height was an arbitrary very large number and whose width was an arbitrary very small number greater than zero. <strong data-conversion-semantic="note">Note:</strong> This will make the gradient look similar to a horizontal linear gradient that is mirrored across the center of the ellipse. It also means that all color-stop positions specified with a percentage resolve to 0px.

<a id="ref-for-ending-shape①⑤"></a>

Otherwise, if the [ending shape](#ending-shape) has zero height:

<a id="ref-for-ending-shape①⑥"></a>

Render as if the [ending shape](#ending-shape) was an ellipse whose width was an arbitrary very large number and whose height was an arbitrary very small number greater than zero. <strong data-conversion-semantic="note">Note:</strong> This will make the gradient look like a solid-color image equal to the color of the last color-stop, or equal to the average color of the gradient if it’s repeating.

#### <a id="radial-gradient-examples"></a>3.2.4. Radial Gradient Examples

All of the following examples are applied to a box that is 200px wide and 100px tall.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-21e5434c"></a> These examples demonstrate different ways to write the basic syntax for radial gradients:
>
> ```text
> radial-gradient(yellow, green);
> radial-gradient(ellipse at center, yellow 0%, green 100%);
> radial-gradient(farthest-corner at 50% 50%, yellow, green);
> ```
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/radial1.png)
>
> ```text
> radial-gradient(circle, yellow, green);
> ```
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/radial2.png)
>
> ```text
> radial-gradient(red, yellow, green);
> ```
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/radial3.png)

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-efad25ac"></a> This image shows a gradient originating from somewhere other than the center of the box:
>
> ```text
> radial-gradient(farthest-side at left bottom, red, yellow 50px, green);
> ```
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/radial4.png)

<a id="ref-for-valdef-radial-extent-closest-side③"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-fe6adef0"></a> Here we illustrate a [closest-side](#valdef-radial-extent-closest-side) gradient.
>
> ```text
> radial-gradient(closest-side at 20px 30px, red, yellow, green);
> radial-gradient(20px 30px at 20px 30px, red, yellow, green);
> ```
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/radial6.png)
>
> ```text
> radial-gradient(closest-side circle at 20px 30px, red, yellow, green);
> radial-gradient(20px 20px at 20px 30px, red, yellow, green);
> ```
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/radial7.png)

<a id="ref-for-funcdef-repeating-linear-gradient①"></a>

<a id="ref-for-funcdef-repeating-radial-gradient①"></a>

### <a id="repeating-gradients"></a>3.3. Repeating Gradients: the [repeating-linear-gradient()](#funcdef-repeating-linear-gradient) and [repeating-radial-gradient()](#funcdef-repeating-radial-gradient) notations

<a id="ref-for-funcdef-linear-gradient⑤"></a>

<a id="ref-for-funcdef-radial-gradient③"></a>

In addition to [linear-gradient()](#funcdef-linear-gradient) and [radial-gradient()](#funcdef-radial-gradient), this specification defines <a id="funcdef-repeating-linear-gradient"></a>repeating-linear-gradient() and <a id="funcdef-repeating-radial-gradient"></a>repeating-radial-gradient() values. These notations take the same values and are interpreted the same as their respective non-repeating siblings defined previously.

<a id="ref-for-funcdef-repeating-linear-gradient②"></a>

<a id="ref-for-typedef-linear-gradient-syntax①"></a>

<a id="ref-for-funcdef-repeating-radial-gradient②"></a>

<a id="ref-for-typedef-radial-gradient-syntax①"></a>

```text
<repeating-linear-gradient()> = repeating-linear-gradient( [ <linear-gradient-syntax> ] )
<repeating-radial-gradient()> = repeating-radial-gradient( [ <radial-gradient-syntax> ] )
```
When rendered, however, the color-stops are repeated infinitely in both directions, with their positions shifted by multiples of the difference between the last specified color-stop’s position and the first specified color-stop’s position. For example, repeating-linear-gradient(red 10px, blue 50px) is equivalent to linear-gradient(..., red -30px, blue 10px, red 10px, blue 50px, red 50px, blue 90px, ...). Note that the last color-stop and first color-stop will always coincide at the boundaries of each group, which will produce sharp transitions if the gradient does not start and end with the same color.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-0122a133"></a> Repeating gradient syntax is identical to that of non-repeating gradients:
>
> ```text
> repeating-linear-gradient(red, blue 20px, red 40px)
> ```
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/repeating1.png)
>
> ```text
> repeating-radial-gradient(red, blue 20px, red 40px)
> ```
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/repeating2.png)
>
> ```text
> repeating-radial-gradient(circle closest-side at 20px 30px, red, yellow, green 100%, yellow 150%, red 200%)
> ```
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/repeating3.png)

<a id="ref-for-gradient-average-color"></a>

If the distance between the first and last color-stops is non-zero, but is small enough that the implementation knows that the physical resolution of the output device is insufficient to faithfully render the gradient, the implementation must [find the average color of the gradient](#gradient-average-color) and render the gradient as a solid-color image equal to the average color.

<a id="ref-for-gradient-average-color①"></a>

If the distance between the first and last color-stops is zero (or rounds to zero due to implementation limitations), the implementation must [find the average color of a gradient](#gradient-average-color) with the same number and color of color-stops, but with the first and last color-stop an arbitrary non-zero distance apart, and the remaining color-stops equally spaced between them. Then it must render the gradient as a solid-color image equal to that average color.

<a id="ref-for-gradient-average-color②"></a>

If the width of the ending shape of a repeating radial gradient is non-zero and the height is zero, or is close enough to zero that the implementation knows that the physical resolution of the output device is insufficient to faithfully render the gradient, the implementation must [find the average color of the gradient](#gradient-average-color) and render the gradient as a solid-color image equal to the average color.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [Degenerate Radial Gradients](#degenerate-radials) section describes how the ending shape is adjusted when its width is zero.

To <a id="gradient-average-color"></a>find the average color of a gradient, run these steps:

1.  Define <var>list</var> as an initially-empty list of premultiplied RGBA colors, and <var>total-length</var> as the distance between first and last color stops.

2.  For each adjacent pair of color-stops, define <var>weight</var> as half the distance between the two color-stops, divided by <var>total-length</var>. Add two entries to <var>list</var>, the first obtained by representing the color of the first color-stop in premultiplied sRGBA and scaling all of the components by <var>weight</var>, and the second obtained in the same way with the second color-stop.

3.  Sum the entries of <var>list</var> component-wise to produce the average color, and return it.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: As usual, implementations may use whatever algorithm they wish, so long as it produces the same result as the above.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-179d6dec"></a> For example, the following gradient is rendered as a solid light-purple image (equal to `rgb(75%,50%,75%)`):
>
> ```text
> repeating-linear-gradient(red 0px, white 0px, blue 0px);
> ```
>
> The following gradient would render the same as the previous under normal circumstances (because desktop monitors can’t faithfully render color-stops 1/10th of a pixel apart), but would render as a normal repeating gradient if, for example, the author applied "zoom:100;" to the element on which the gradient appears:
>
> ```text
> repeating-linear-gradient(red 0px, white .1px, blue .2px);
> ```
### <a id="gradient-colors"></a>3.4. Defining Gradient Color

<a id="ref-for-typedef-color①"></a>

<a id="ref-for-gradient-line②②"></a>

<a id="ref-for-color-stop"></a>

<a id="ref-for-gradient-function"></a>

<a id="ref-for-starting-point③"></a>

<a id="ref-for-ending-point③"></a>

The colors in gradients are specified using <a id="color-stop"></a>color stops (a [\<color\>](https://www.w3.org/TR/css-color-4/#typedef-color) and a corresponding position on the [gradient line](#gradient-line)) and <a id="color-transition-hint"></a>color transition hints (a position between two [color stops](#color-stop) representing the halfway point in the color transition) which are placed on the <a id="ref-for-gradient-line②③"></a>gradient line, defining the color at every point of the line. (Each [gradient function](#gradient-function) defines the shape and length of the <a id="ref-for-gradient-line②④"></a>gradient line, along with its [starting point](#starting-point) and [ending point](#ending-point); see above.)

<a id="ref-for-gradient-line②⑤"></a>

Colors throughout the gradient field are then determined by tying them to specific points along the [gradient line](#gradient-line) as specified by the gradient function. UAs may “dither” gradient colors slightly (randomly alternate individual pixels with nearby colors on the gradient line) to effect a smoother gradient.

#### <a id="color-stop-syntax"></a>3.4.1.  Color Stop Lists

<a id="ref-for-color-stop①"></a>

<a id="ref-for-color-transition-hint"></a>

[Color stops](#color-stop) and [transition hints](#color-transition-hint) are specified in a <a id="color-stop-list"></a>color stop list, which is a list of two or more <a id="ref-for-color-stop②"></a>color stops interleaved with optional <a id="ref-for-color-transition-hint①"></a>transition hints:

<a id="typedef-color-stop-list"></a>

<a id="ref-for-typedef-linear-color-stop"></a>

<a id="ref-for-comb-comma②"></a>

<a id="ref-for-typedef-linear-color-hint"></a>

<a id="ref-for-mult-opt③"></a>

<a id="ref-for-comb-comma③"></a>

<a id="ref-for-typedef-linear-color-stop①"></a>

<a id="ref-for-mult-comma"></a>

<a id="typedef-linear-color-stop"></a>

<a id="ref-for-typedef-color②"></a>

<a id="ref-for-typedef-length-percentage③"></a>

<a id="ref-for-mult-opt④"></a>

<a id="typedef-linear-color-hint"></a>

<a id="ref-for-typedef-length-percentage④"></a>

```text
<color-stop-list> =
  <linear-color-stop> , [ <linear-color-hint>? , <linear-color-stop> ]#
<linear-color-stop> = <color> <length-percentage>?
<linear-color-hint> = <length-percentage>
```
<a id="ref-for-gradient-line②⑥"></a>

<a id="ref-for-starting-point④"></a>

<a id="ref-for-ending-point④"></a>

Percentages are resolved against the length of the [gradient line](#gradient-line) between the [starting point](#starting-point) and [ending point](#ending-point), with 0% being at the starting point and 100% being at the ending point. Lengths are measured along the <a id="ref-for-gradient-line②⑦"></a>gradient line from the <a id="ref-for-starting-point⑤"></a>starting point in the direction of the <a id="ref-for-ending-point⑤"></a>ending point.

<a id="ref-for-color-stop③"></a>

<a id="ref-for-color-transition-hint②"></a>

<a id="ref-for-starting-point⑥"></a>

<a id="ref-for-ending-point⑥"></a>

<a id="ref-for-gradient-line②⑧"></a>

[Color stop](#color-stop) and [transition hint](#color-transition-hint) positions are usually placed between the [starting point](#starting-point) and [ending point](#ending-point), but that’s not required: the gradient line extends infinitely in both directions, and positions can be specified anywhere on the [gradient line](#gradient-line).

<a id="ref-for-color-stop④"></a>

<a id="ref-for-color-stop-list"></a>

<a id="ref-for-gradient-line②⑨"></a>

<a id="ref-for-starting-point⑦"></a>

<a id="ref-for-ending-point⑦"></a>

When the position of a [color stop](#color-stop) is omitted, it is automatically assigned a position. The first or last <a id="ref-for-color-stop⑤"></a>color stop in the [color stop list](#color-stop-list) is assigned the [gradient line’s](#gradient-line) [starting point](#starting-point) or [ending point](#ending-point) (respectively). Otherwise, it’s assigned the position halfway between the two surrounding stops. If multiple stops in a row lack a position, they space themselves out equally between the surrounding positioned stops. See [§ 3.4.3 Color Stop “Fixup”](#color-stop-fixup) for details.

#### <a id="coloring-gradient-line"></a>3.4.2.  Coloring the Gradient Line

<a id="ref-for-color-stop⑥"></a>

<a id="ref-for-gradient-line③⓪"></a>

At each [color stop](#color-stop) position, the [gradient line](#gradient-line) is the color of the <a id="ref-for-color-stop⑦"></a>color stop. Before the first <a id="ref-for-color-stop⑧"></a>color stop, the <a id="ref-for-gradient-line③①"></a>gradient line is the color of the first <a id="ref-for-color-stop⑨"></a>color stop, and after the last <a id="ref-for-color-stop①⓪"></a>color stop, the <a id="ref-for-gradient-line③②"></a>gradient line is the color of the last <a id="ref-for-color-stop①①"></a>color stop. Between two <a id="ref-for-color-stop①②"></a>color stops, the <a id="ref-for-gradient-line③③"></a>gradient line’s color is interpolated between the colors of the two <a id="ref-for-color-stop①③"></a>color stops, with the interpolation taking place in [premultiplied RGBA space](#premultiplied).

<a id="ref-for-color-stop①④"></a>

By default, this interpolation is linear—​at 25%, 50%, or 75% of the distance between two [color stops](#color-stop), the color is a 25%, 50%, or 75% blend of the colors of the two stops.

<a id="ref-for-color-transition-hint③"></a>

<a id="ref-for-color-stop①⑤"></a>

However, if a [transition hint](#color-transition-hint) was provided between two [color stops](#color-stop), the interpolation is non-linear, and controlled by the hint:

1.  <a id="ref-for-color-stop①⑥"></a>

    <a id="ref-for-color-transition-hint④"></a>

    Determine the location of the [transition hint](#color-transition-hint) as a percentage of the distance between the two [color stops](#color-stop), denoted as a number between 0 and 1, where 0 indicates the hint is placed right on the first <a id="ref-for-color-stop①⑦"></a>color stop, and 1 indicates the hint is placed right on the second <a id="ref-for-color-stop①⑧"></a>color stop. Let this percentage be <var>H</var>.

2.  <a id="ref-for-color-stop①⑨"></a>

    For any given point between the two color stops, determine the point’s location as a percentage of the distance between the two [color stops](#color-stop), in the same way as the previous step. Let this percentage be <var>P</var>.

3.  Let <var>C</var>, the color weighting at that point, be equal to <code><var>P</var><sup>log<sub><var>H</var></sub>(.5)</sup></code>.

4.  <a id="ref-for-color-stop②⓪"></a>

    The color at that point is then a linear blend between the colors of the two [color stops](#color-stop), blending <code>(1 - <var>C</var>)</code> of the first stop and <var>C</var> of the second stop.

<a id="ref-for-color-transition-hint⑤"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [transition hint](#color-transition-hint) specifies where the “halfway color”—​the 50% blend between the colors of the two surrounding color stops—​should be placed. When the hint is exactly halfway between the two surrounding color stops, the above interpolation algorithm happens to produce the ordinary linear interpolation. If the hint is placed anywhere else, it produces a smooth exponential curve between the surrounding color stops, with the “halfway color” occurring exactly where the hint specifies.

> <strong data-conversion-semantic="issue">Issue</strong>
>
> <a id="issue-7a141efc"></a> Add a visual example of a color hint being used.

<a id="ref-for-color-stop②①"></a>

If multiple [color stops](#color-stop) have the same position, they produce an infinitesimal transition from the one specified first in the list to the one specified last. In effect, the color suddenly changes at that position rather than smoothly transitioning.

> <strong data-conversion-semantic="note">Note</strong>
>
> <a id="premultiplied"></a>
>
> What does “pre-multiplied” mean?
>
> A “pre-multiplied” color is written in a form where the alpha channel is multiplied into the color channels, rather than being processed independently. For example, a partially-transparent blue may be given as `rgba(0, 0, 255, .5)`, which would then be expressed as `[0, 0, 127.5, .5]` in its premultiplied representation.
>
> Interpolating colors using the premultiplied representations rather than the plain rgba representations tends to produce more attractive transitions, particularly when transitioning from a fully opaque color to fully transparent.
>
> Note that transitions where either the transparency or the color are held constant (for example, transitioning between `rgba(255, 0, 0, 100%)` (opaque red) and `rgba(0,0,255,100%)` (opaque blue), or `rgba(255,0,0,100%)` (opaque red) and `rgba(255,0,0,0%)` (transparent red)) have identical results whether the color interpolation is done in premultiplied or non-premultiplied color-space. Differences only arise when <em>both</em> the color and transparency differ between the two endpoints.
>
> > <strong data-conversion-semantic="example">Example</strong>
> >
> > <a id="example-77596609"></a> The following example illustrates the difference between a gradient transitioning in pre-multiplied sRGBA and one transitioning (incorrectly) in non-premultiplied. In both of these example, the gradient is drawn over a white background. Both gradients could be written with the following value:
> >
> > ```text
> > linear-gradient(90deg, red, transparent, blue)
> > ```
> >
> > With premultiplied colors, transitions to or from "transparent" always look nice:
> >
> > (Image requires SVG)
> >
> > ![(Image requires SVG)](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/gradient2.svg)
> >
> > On the other hand, if a gradient were to incorrectly transition in non-premultiplied space, the center of the gradient would be a noticeably grayish color, because "transparent" is actually a shorthand for rgba(0,0,0,0), or transparent black, meaning that the red transitions to a black as it loses opacity, and similarly with the blue’s transition:
> >
> > (Image requires SVG)
> >
> > ![(Image requires SVG)](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/gradient3.svg)

#### <a id="color-stop-fixup"></a>3.4.3.  Color Stop “Fixup”

<a id="ref-for-used-value"></a>

<a id="ref-for-color-stop②②"></a>

When resolving the [used](https://www.w3.org/TR/css-cascade-5/#used-value) positions of each [color stop](#color-stop), the following steps must be applied <em>in order</em>:

1.  <a id="ref-for-color-stop②③"></a>

    If the first [color stop](#color-stop) does not have a position, set its position to 0%. If the last <a id="ref-for-color-stop②④"></a>color stop does not have a position, set its position to 100%.

2.  <a id="ref-for-color-transition-hint⑥"></a>

    <a id="ref-for-color-stop②⑤"></a>

    If a [color stop](#color-stop) or [transition hint](#color-transition-hint) has a position that is less than the specified position of any <a id="ref-for-color-stop②⑥"></a>color stop or <a id="ref-for-color-transition-hint⑦"></a>transition hint before it in the list, set its position to be equal to the largest specified position of any <a id="ref-for-color-stop②⑦"></a>color stop or <a id="ref-for-color-transition-hint⑧"></a>transition hint before it.

3.  <a id="ref-for-color-stop②⑧"></a>

    If any [color stop](#color-stop) still does not have a position, then, for each run of adjacent <a id="ref-for-color-stop②⑨"></a>color stops without positions, set their positions so that they are evenly spaced between the preceding and following <a id="ref-for-color-stop③⓪"></a>color stops with positions.

<a id="ref-for-color-stop③①"></a>

<a id="ref-for-color-transition-hint⑨"></a>

After applying these rules, all [color stops](#color-stop) and [transition hints](#color-transition-hint) will have a definite position and color and they will be in ascending order.

<a id="ref-for-color-stop③②"></a>

<a id="ref-for-propdef-background-image③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: It is recommended that authors exercise caution when mixing different types of units, such as px, em, or %, as this can cause a [color stop](#color-stop) to unintentionally try to move before an earlier one. For example, the rule [background-image: linear-gradient(yellow 100px, blue 50%)](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) wouldn’t trigger any fix-up while the background area is at least 200px tall. If it was 150px tall, however, the blue <a id="ref-for-color-stop③③"></a>color stop’s position would be equivalent to 75px, which precedes the yellow <a id="ref-for-color-stop③④"></a>color stop, and would be corrected to a position of 100px. Additionally, since the relative ordering of such color stops cannot be determined without performing layout, they will not interpolate smoothly in [animations](https://www.w3.org/TR/css-animations/) or [transitions](https://www.w3.org/TR/css-transitions/).

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-996853e0"></a> Below are several pairs of gradients. The latter of each pair is a manually “fixed-up” version of the former, obtained by applying the above rules. For each pair, both gradients will render identically. <strong data-conversion-semantic="note">Note:</strong> The numbers in each arrow specify which fixup steps are invoked in the transformation.
>
> ```text
> 1. linear-gradient(red, white 20%, blue)
>    =1=>
>    linear-gradient(red 0%, white 20%, blue 100%)
> 
> 2. linear-gradient(red 40%, white, black, blue)
>    =1,3=>
>    linear-gradient(red 40%, white 60%, black 80%, blue 100%)
> 
> 3. linear-gradient(red -50%, white, blue)
>    =1,3=>
>    linear-gradient(red -50%, white 25%, blue 100%)
> 
> 4. linear-gradient(red -50px, white, blue)
>    =1,3=>
>    linear-gradient(red -50px, white calc(-25px + 50%), blue 100%)
> 
> 5. linear-gradient(red 20px, white 0px, blue 40px)
>    =2=>
>    linear-gradient(red 20px, white 20px, blue 40px)
> 
> 6. linear-gradient(red, white -50%, black 150%, blue)
>    =1,2=>
>    linear-gradient(red 0%, white 0%, black 150%, blue 150%)
> 
> 7. linear-gradient(red 80px, white 0px, black, blue 100px)
>    =2,3=>
>    linear-gradient(red 80px, white 80px, black 90px, blue 100px)
> ```
## <a id="sizing"></a>4. Sizing Images and Objects in CSS

Images used in CSS may come from a number of sources: from binary image formats (such as gif, jpeg, etc), dedicated markup formats (such as SVG), and CSS-specific formats (such as the linear-gradient() value type defined in this specification). As well, a document may contain many other types of objects, such as video, plugins, or nested documents. These images and objects (just <a id="objects"></a>objects hereafter) may offer many types of sizing information to CSS, or none at all. This section defines generically the size negotiation model between the object and the CSS layout algorithms.

### <a id="sizing-terms"></a>4.1. Object-Sizing Terminology

In order to define this handling, we define a few terms, to make it easier to refer to various concepts:

<a id="natural-dimensions"></a>natural dimensions<a id="intrinsic-dimensions"></a>  
<a id="ref-for-objects"></a>

<a id="ref-for-natural-dimensions④"></a>

The term [natural dimensions](#natural-dimensions) refers to the set of the <a id="natural-height"></a>natural height<a id="intrinsic-height"></a>, <a id="natural-width"></a>natural width<a id="intrinsic-width"></a>, and <a id="natural-aspect-ratio"></a>natural aspect ratio<a id="intrinsic-aspect-ratio"></a> (the ratio between the width and height), each of which may or may not exist for a given [object](#objects). These natural dimensions represent the preferred sizing intrinsic to the object itself; that is, they are not a function of the context in which the object is used. CSS does not define how the natural dimensions are found in general.

<a id="ref-for-objects①"></a>

<a id="ref-for-natural-aspect-ratio"></a>

<a id="ref-for-natural-width"></a>

<a id="ref-for-natural-height"></a>

<a id="ref-for-natural-dimensions⑤"></a>

<a id="ref-for-the-iframe-element"></a>

Raster images are an example of an [object](#objects) with all three natural dimensions. SVG images designed to scale might have only an [natural aspect ratio](#natural-aspect-ratio); SVG images can also be created with only an [natural width](#natural-width) or [height](#natural-height). CSS gradients, defined in this specification, are an example of an object with no [natural dimensions](#natural-dimensions) at all. Another example of this is embedded documents, such as the <code><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code> element in HTML. Many <a id="ref-for-objects②"></a>objects, such as most images, cannot have only two <a id="ref-for-natural-dimensions⑥"></a>natural dimensions, as any two automatically define the third. However some types of replaced elements, such as form controls, can have a <a id="ref-for-natural-width①"></a>natural width and a <a id="ref-for-natural-height①"></a>natural height, but no <a id="ref-for-natural-aspect-ratio①"></a>natural aspect ratio.

<a id="ref-for-objects③"></a>

<a id="ref-for-degenerate-ratio"></a>

<a id="ref-for-natural-aspect-ratio②"></a>

If an [object](#objects) has a [degenerate](https://www.w3.org/TR/css-values-4/#degenerate-ratio) [natural aspect ratio](#natural-aspect-ratio) (at least one part being zero or infinity), it is treated as having no <a id="ref-for-natural-aspect-ratio③"></a>natural aspect ratio.

<a id="ref-for-objects④"></a>

<a id="ref-for-natural-dimensions⑦"></a>

<a id="ref-for-default-object-size①"></a>

<a id="ref-for-contain-constraint"></a>

If an [object](#objects) (such as an icon) has multiple sizes, then the largest size (by area) is taken as its [natural dimensions](#natural-dimensions). If it has multiple aspect ratios at that size, or has multiple aspect ratios and no size, then the aspect ratio closest to the aspect ratio of the [default object size](#default-object-size) is used. Determine this by seeing which aspect ratio produces the largest area when fitting it within the <a id="ref-for-default-object-size②"></a>default object size using a [contain constraint](#contain-constraint) fit; if multiple sizes tie for the largest area, the widest size is chosen as its <a id="ref-for-natural-dimensions⑧"></a>natural dimensions.

<a id="ref-for-natural-width②"></a>

<a id="ref-for-natural-height②"></a>

The [natural width](#natural-width) and [natural height](#natural-height) are collectively referred to the <a id="natural-size"></a>natural sizes.

<a id="specified-size"></a>specified size  
<a id="ref-for-propdef-background-size①"></a>

<a id="ref-for-propdef-height"></a>

<a id="ref-for-propdef-width"></a>

<a id="ref-for-objects⑤"></a>

The specified size of an [object](#objects) is given by CSS, such as through the [width](https://www.w3.org/TR/css-sizing-3/#propdef-width) and [height](https://www.w3.org/TR/css-sizing-3/#propdef-height) or [background-size](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-size) properties. The specified size can be a definite width and height, a set of constraints, or a combination thereof.

<a id="concrete-object-size"></a>concrete object size  
<a id="ref-for-default-object-size③"></a>

<a id="ref-for-specified-size"></a>

<a id="ref-for-natural-dimensions⑨"></a>

<a id="ref-for-objects⑥"></a>

<a id="ref-for-concrete-object-size①"></a>

The [concrete object size](#concrete-object-size) is the result of combining an [object’s](#objects) [natural dimensions](#natural-dimensions) and [specified size](#specified-size) with the [default object size](#default-object-size) of the context it’s used in, producing a rectangle with an absolute width and height.

<a id="default-object-size"></a>default object size  
<a id="ref-for-specified-size①"></a>

<a id="ref-for-natural-dimensions①⓪"></a>

<a id="ref-for-concrete-object-size②"></a>

<a id="ref-for-default-object-size④"></a>

The [default object size](#default-object-size) is a rectangle with an absolute height and width used to determine the [concrete object size](#concrete-object-size) when both the [natural dimensions](#natural-dimensions) and [specified size](#specified-size) are missing dimensions.

### <a id="object-negotiation"></a>4.2. CSS⇋Object Negotiation

<a id="ref-for-objects⑦"></a>

[Objects](#objects) in CSS are sized and rendered by the <a id="object-size-negotiation"></a>object size negotiation algorithm as follows:

1.  <a id="ref-for-objects⑧"></a>

    <a id="ref-for-funcdef-url④"></a>

    <a id="ref-for-propdef-background-image④"></a>

    <a id="ref-for-attr-img-src"></a>

    <a id="ref-for-the-img-element"></a>

    <a id="ref-for-natural-dimensions①①"></a>

    When an [object](#objects) is specified in a document, such as through a [url()](https://www.w3.org/TR/css-values-4/#funcdef-url) value in a [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) property or a <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#attr-img-src">src</a></code> attribute on an <code><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code> element, CSS queries the object for its [natural dimensions](#natural-dimensions).

2.  <a id="ref-for-natural-dimensions①②"></a>

    <a id="ref-for-specified-size②"></a>

    <a id="ref-for-default-object-size⑤"></a>

    <a id="ref-for-objects⑨"></a>

    <a id="ref-for-concrete-object-size③"></a>

    Using the [natural dimensions](#natural-dimensions), the [specified size](#specified-size), and the [default object size](#default-object-size) for the context the [object](#objects) is used in, CSS then computes a [concrete object size](#concrete-object-size). (See the [following section](#default-sizing).) This defines the size and position of the region the <a id="ref-for-objects①⓪"></a>object will render in.

3.  <a id="ref-for-objects①①"></a>

    <a id="ref-for-concrete-object-size④"></a>

    <a id="ref-for-natural-dimensions①③"></a>

    CSS asks the [object](#objects) to render itself at the [concrete object size](#concrete-object-size). CSS does not define how <a id="ref-for-objects①②"></a>objects render when the <a id="ref-for-concrete-object-size⑤"></a>concrete object size is different from the <a id="ref-for-objects①③"></a>object’s [natural dimensions](#natural-dimensions). The <a id="ref-for-objects①④"></a>object may adjust itself to match the <a id="ref-for-concrete-object-size⑥"></a>concrete object size in some way, or even render itself larger or smaller than the <a id="ref-for-concrete-object-size⑦"></a>concrete object size to satisfy sizing constraints of its own.

4.  <a id="ref-for-objects①⑤"></a>

    <a id="ref-for-concrete-object-size⑧"></a>

    Unless otherwise specified by CSS, the [object](#objects) is then clipped to the [concrete object size](#concrete-object-size).

### <a id="concrete-size-resolution"></a>4.3. Concrete Object Size Resolution

<a id="ref-for-objects①⑥"></a>

Currently the rules for sizing [objects](#objects) are described in each context that such <a id="ref-for-objects①⑦"></a>objects are used. This section defines some common sizing constraints and how to resolve them so that future specs can refer to them instead of redefining size resolution in each instance.

#### <a id="default-sizing"></a>4.3.1. Default Sizing Algorithm

<a id="ref-for-objects①⑧"></a>

<a id="ref-for-concrete-object-size⑨"></a>

<a id="ref-for-natural-dimensions①④"></a>

<a id="ref-for-specified-size③"></a>

The <a id="default-sizing-algorithm"></a>default sizing algorithm is a set of rules commonly used to find an [object’s](#objects) [concrete object size](#concrete-object-size). It resolves the simultaneous constraints presented by the <a id="ref-for-objects①⑨"></a>object’s [natural dimensions](#natural-dimensions) and either an unconstrained [specified size](#specified-size) or one consisting of only a definite width and/or height.

<a id="ref-for-objects②⓪"></a>

<a id="ref-for-propdef-list-style-image④"></a>

<a id="ref-for-default-sizing-algorithm"></a>

<a id="ref-for-propdef-border-image"></a>

<a id="ref-for-concrete-object-size①⓪"></a>

Some [object](#objects) sizing rules (such as those for [list-style-image](https://www.w3.org/TR/css-lists-3/#propdef-list-style-image)) correspond exactly to the [default sizing algorithm](#default-sizing-algorithm). Others (such as those for [border-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image)) invoke the default sizing algorithm but also apply additional sizing rules before arriving at a final [concrete object size](#concrete-object-size).

<a id="ref-for-default-sizing-algorithm①"></a>

The [default sizing algorithm](#default-sizing-algorithm) is defined as follows:

- <a id="ref-for-specified-size④"></a>

  <a id="ref-for-concrete-object-size①①"></a>

  If the [specified size](#specified-size) is a definite width and height, the [concrete object size](#concrete-object-size) is given that width and height.

- <a id="ref-for-specified-size⑤"></a>

  <a id="ref-for-concrete-object-size①②"></a>

  If the [specified size](#specified-size) is only a width or height (but not both) then the [concrete object size](#concrete-object-size) is given that specified width or height. The other dimension is calculated as follows:

  1.  <a id="ref-for-objects②①"></a>

      <a id="ref-for-natural-aspect-ratio④"></a>

      <a id="ref-for-concrete-object-size①③"></a>

      If the [object](#objects) has a [natural aspect ratio](#natural-aspect-ratio), the missing dimension of the [concrete object size](#concrete-object-size) is calculated using that aspect ratio and the present dimension.

  2.  <a id="ref-for-natural-dimensions①⑤"></a>

      <a id="ref-for-objects②②"></a>

      Otherwise, if the missing dimension is present in the object’s [natural dimensions](#natural-dimensions), the missing dimension is taken from the [object’s](#objects) <a id="ref-for-natural-dimensions①⑥"></a>natural dimensions.

  3.  <a id="ref-for-concrete-object-size①④"></a>

      <a id="ref-for-default-object-size⑥"></a>

      Otherwise, the missing dimension of the [concrete object size](#concrete-object-size) is taken from the [default object size](#default-object-size).

- <a id="ref-for-specified-size⑥"></a>

  If the [specified size](#specified-size) has no constraints:

  1.  <a id="ref-for-objects②③"></a>

      <a id="ref-for-natural-dimensions①⑦"></a>

      <a id="ref-for-specified-size⑦"></a>

      If the [object](#objects) has a [natural](#natural-dimensions) height or width, its size is resolved as if its <a id="ref-for-natural-dimensions①⑧"></a>natural dimensions were given as the [specified size](#specified-size).

  2.  <a id="ref-for-contain-constraint①"></a>

      <a id="ref-for-default-object-size⑦"></a>

      Otherwise, its size is resolved as a [contain constraint](#contain-constraint) against the [default object size](#default-object-size).

#### <a id="cover-contain"></a>4.3.2. Cover and Contain Constraint Sizing

<a id="ref-for-contain-constraint②"></a>

<a id="ref-for-cover-constraint"></a>

<a id="ref-for-objects②④"></a>

<a id="ref-for-preferred-aspect-ratio"></a>

Two other common specified sizes are the [contain constraint](#contain-constraint) and the [cover constraint](#cover-constraint), both of which are resolved against a specified <a id="constraint-rectangle"></a>constraint rectangle using the [object’s](#objects) [preferred aspect ratio](https://www.w3.org/TR/css-sizing-4/#preferred-aspect-ratio):

- <a id="ref-for-concrete-object-size①⑤"></a>

  <a id="ref-for-objects②⑤"></a>

  <a id="ref-for-natural-aspect-ratio⑤"></a>

  A <a id="contain-constraint"></a>contain constraint is resolved by setting the [concrete object size](#concrete-object-size) to the largest rectangle that has the [object’s](#objects) [natural aspect ratio](#natural-aspect-ratio) and additionally has neither width nor height larger than the constraint rectangle’s width and height, respectively.

- <a id="ref-for-concrete-object-size①⑥"></a>

  <a id="ref-for-objects②⑥"></a>

  <a id="ref-for-natural-aspect-ratio⑥"></a>

  A <a id="cover-constraint"></a>cover constraint is resolved by setting the [concrete object size](#concrete-object-size) to the smallest rectangle that has the [object’s](#objects) [natural aspect ratio](#natural-aspect-ratio) and additionally has neither width nor height smaller than the constraint rectangle’s width and height, respectively.

<a id="ref-for-natural-aspect-ratio⑦"></a>

<a id="ref-for-concrete-object-size①⑦"></a>

In both cases, if the object doesn’t have a [natural aspect ratio](#natural-aspect-ratio), the [concrete object size](#concrete-object-size) is the specified constraint rectangle.

### <a id="object-sizing-examples"></a>4.4. Examples of CSS Object Sizing

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-47bacc07"></a> The following examples show how the [CSS 2.1](https://www.w3.org/TR/CSS2/) and [CSS3 Backgrounds &#x26; Borders](https://www.w3.org/TR/css3-background/) sizing algorithms correspond to concepts defined in this specification.
>
> <a id="ref-for-propdef-background-image⑤"></a>
>
> [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image)
>
> <a id="ref-for-valdef-background-repeat-round"></a>
>
> <a id="ref-for-propdef-background-repeat"></a>
>
> <a id="ref-for-cover-constraint①"></a>
>
> <a id="ref-for-contain-constraint③"></a>
>
> <a id="ref-for-propdef-background-size②"></a>
>
> <a id="ref-for-default-object-size⑧"></a>
>
> <a id="ref-for-specified-size⑧"></a>
>
> <a id="ref-for-default-sizing-algorithm②"></a>
>
> <a id="ref-for-concrete-object-size①⑧"></a>
>
> The rules for calculating the [concrete object size](#concrete-object-size) of a background are defined in [CSS2.1§14.2.1](https://www.w3.org/TR/CSS2/colors.html#background-properties) and [CSS3BG§3.9](https://www.w3.org/TR/css3-background/#the-background-size). CSS2.1 uses the [default sizing algorithm](#default-sizing-algorithm) with no [specified size](#specified-size) and the background positioning area as the [default object size](#default-object-size). [\[CSS2\]](#biblio-css2) In CSS3, [background-size](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-size) property can give a sizing constraint, invoking either the <a id="ref-for-default-sizing-algorithm③"></a>default sizing algorithm or one of the [contain](#contain-constraint) or [cover](#cover-constraint) constraints. The concrete object size is further adjusted in later steps if [background-repeat](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-repeat) has a [round](https://www.w3.org/TR/css-backgrounds-3/#valdef-background-repeat-round) value. [\[CSS3BG\]](#biblio-css3bg)
>
> <a id="ref-for-propdef-list-style-image⑤"></a>
>
> [list-style-image](https://www.w3.org/TR/css-lists-3/#propdef-list-style-image)
>
> <a id="ref-for-default-object-size⑨"></a>
>
> <a id="ref-for-specified-size⑨"></a>
>
> <a id="ref-for-default-sizing-algorithm④"></a>
>
> <a id="ref-for-concrete-object-size①⑨"></a>
>
> The rules for calculating the [concrete object size](#concrete-object-size) of a list-style image are defined in [CSS2.1§12.5.1](https://www.w3.org/TR/CSS2/generate.html#propdef-list-style-image). They use the [default sizing algorithm](#default-sizing-algorithm) with no [specified size](#specified-size) and a [default object size](#default-object-size) of 1em square.
>
> <a id="ref-for-propdef-border-image①"></a>
>
> [border-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image)
>
> <a id="ref-for-propdef-border-image-repeat"></a>
>
> <a id="ref-for-default-object-size①⓪"></a>
>
> <a id="ref-for-specified-size①⓪"></a>
>
> <a id="ref-for-default-sizing-algorithm⑤"></a>
>
> Border images are sized twice: first the entire image is sized to determine the slice points, then the slices are sized to decorate the border. The first sizing operation is defined in [CSS3BG§6.2](https://www.w3.org/TR/css3-background/#the-border-image-slice) and uses the [default sizing algorithm](#default-sizing-algorithm) with no [specified size](#specified-size), and the [border image area](https://www.w3.org/TR/css3-background/#border-image-area) as the [default object size](#default-object-size). The second operation is defined in [CSS3BG§6.2](https://www.w3.org/TR/css3-background/#border-image-process): the <a id="ref-for-default-sizing-algorithm⑥"></a>default sizing algorithm is used to determine an initial size for each slice with the corresponding border image area part as the default object size. By default the <a id="ref-for-specified-size①①"></a>specified size matches this <a id="ref-for-default-object-size①①"></a>default object size; however the [border-image-repeat](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image-repeat) property can drop the specified size in one or more directions and may also apply an additional rounding step. [\[CSS3BG\]](#biblio-css3bg)
>
> <a id="ref-for-propdef-cursor②"></a>
>
> [cursor](https://www.w3.org/TR/css-ui-4/#propdef-cursor)
>
> <a id="ref-for-default-object-size①②"></a>
>
> <a id="ref-for-concrete-object-size②⓪"></a>
>
> The rules for calculating the [concrete object size](#concrete-object-size) of a cursor are defined in [CSS2.1 § 18.1: Cursors](https://www.w3.org/TR/CSS2/ui.html#cursor-props). The [default object size](#default-object-size) is a UA-defined size that should be based on the size of a typical cursor on the UA’s operating system. [\[CSS2\]](#biblio-css2)
>
> <a id="ref-for-propdef-content①"></a>
>
> [content](https://www.w3.org/TR/css-content-3/#propdef-content)
>
> <a id="ref-for-propdef-height①"></a>
>
> <a id="ref-for-propdef-width①"></a>
>
> <a id="ref-for-propdef-content②"></a>
>
> Objects inserted via the CSS2.1 [content](https://www.w3.org/TR/css-content-3/#propdef-content) property are anonymous [replaced elements](https://www.w3.org/TR/CSS2/conform.html#replaced-element), and are sized the same way. [\[CSS2\]](#biblio-css2) Note that such anonymous elements have all their non-inherited properties (including [width](https://www.w3.org/TR/css-sizing-3/#propdef-width), [height](https://www.w3.org/TR/css-sizing-3/#propdef-height), etc.) set to their initial values.
>
> [replaced elements](https://www.w3.org/TR/CSS2/conform.html#replaced-element)
>
> <a id="ref-for-concrete-object-size②①"></a>
>
> <a id="ref-for-propdef-object-fit"></a>
>
> <a id="ref-for-propdef-content③"></a>
>
> [CSS 2.1](https://www.w3.org/TR/CSS2/) defines the sizing of replaced elements (including those inserted as [generated content](https://www.w3.org/TR/CSS2/generate.html#content) via [content](https://www.w3.org/TR/css-content-3/#propdef-content)) in sections [10.3.2](https://www.w3.org/TR/CSS2/visudet.html#inline-replaced-width), [10.4](https://www.w3.org/TR/CSS2/visudet.html#min-max-widths), [10.6.2](https://www.w3.org/TR/CSS2/visudet.html#inline-replaced-height), and [10.7](https://www.w3.org/TR/CSS2/visudet.html#min-max-heights). [\[CSS2\]](#biblio-css2) The [object-fit](#propdef-object-fit) property defined below defines how the [concrete object size](#concrete-object-size) corresponds to the element’s used width and height; by default they coincide.

<a id="ref-for-propdef-object-fit①"></a>

### <a id="the-object-fit"></a>4.5. Sizing Objects: the [object-fit](#propdef-object-fit) property

| Field               | Definition                                                                                                                                                                 |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-object-fit"></a>object-fit                                                                                                                                              |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①③"></a>fill [\|](https://www.w3.org/TR/css-values-4/#comb-one) contain <a id="ref-for-comb-one①④"></a>\| cover <a id="ref-for-comb-one①⑤"></a>\| none <a id="ref-for-comb-one①⑥"></a>\| scale-down |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | fill                                                                                                                                                                       |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | replaced elements                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                                         |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                        |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                   |

<a id="ref-for-propdef-object-fit②"></a>

The [object-fit](#propdef-object-fit) property specifies how the contents of a replaced element should be fitted to the box established by its used height and width.

<a id="valdef-object-fit-fill"></a>fill  
<a id="ref-for-concrete-object-size②②"></a>

The replaced content is sized to fill the element’s content box: the object’s [concrete object size](#concrete-object-size) is the element’s used width and height.

<a id="valdef-object-fit-contain"></a>contain  
<a id="ref-for-contain-constraint④"></a>

<a id="ref-for-concrete-object-size②③"></a>

<a id="ref-for-natural-aspect-ratio⑧"></a>

The replaced content is sized to maintain its [natural aspect ratio](#natural-aspect-ratio) while fitting within the element’s content box: its [concrete object size](#concrete-object-size) is resolved as a [contain constraint](#contain-constraint) against the element’s used width and height.

<a id="valdef-object-fit-cover"></a>cover  
<a id="ref-for-cover-constraint②"></a>

<a id="ref-for-concrete-object-size②④"></a>

<a id="ref-for-natural-aspect-ratio⑨"></a>

The replaced content is sized to maintain its [natural aspect ratio](#natural-aspect-ratio) while filling the element’s entire content box: its [concrete object size](#concrete-object-size) is resolved as a [cover constraint](#cover-constraint) against the element’s used width and height.

<a id="valdef-object-fit-none"></a>none  
<a id="ref-for-default-object-size①③"></a>

<a id="ref-for-default-sizing-algorithm⑦"></a>

<a id="ref-for-concrete-object-size②⑤"></a>

The replaced content is not resized to fit inside the element’s content box: determine the object’s [concrete object size](#concrete-object-size) using the [default sizing algorithm](#default-sizing-algorithm) with no specified size, and a [default object size](#default-object-size) equal to the replaced element’s used width and height.

<a id="valdef-object-fit-scale-down"></a>scale-down  
<a id="ref-for-concrete-object-size②⑥"></a>

<a id="ref-for-valdef-object-fit-contain"></a>

<a id="ref-for-valdef-object-fit-none"></a>

Size the content as if [none](#valdef-object-fit-none) or [contain](#valdef-object-fit-contain) were specified, whichever would result in a smaller [concrete object size](#concrete-object-size).

<a id="ref-for-valdef-object-fit-none①"></a>

<a id="ref-for-valdef-object-fit-contain①"></a>

<a id="ref-for-natural-aspect-ratio①⓪"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Both [none](#valdef-object-fit-none) and [contain](#valdef-object-fit-contain) respect the content’s [natural aspect ratio](#natural-aspect-ratio), so the concept of "smaller" is well-defined.

<a id="ref-for-propdef-object-position"></a>

If the content does not completely fill the replaced element’s content box, the unfilled space shows the replaced element’s background. Since replaced elements always clip their contents to the content box, the content will never overflow. See the [object-position](#propdef-object-position) property for positioning the object with respect to the content box.

![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/img_scale.png)

<a id="ref-for-propdef-object-fit③"></a>

<a id="ref-for-propdef-object-position①"></a>

<a id="ref-for-valdef-object-fit-scale-down"></a>

<a id="ref-for-valdef-object-fit-contain②"></a>

An example showing how four of the values of [object-fit](#propdef-object-fit) cause the replaced element (blue figure) to be scaled to fit its height/width box (shown with a green background), using the initial value for [object-position](#propdef-object-position). The fifth value, [scale-down](#valdef-object-fit-scale-down), in this case looks identical to [contain](#valdef-object-fit-contain).

<a id="ref-for-propdef-object-fit④"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: The [object-fit](#propdef-object-fit) property has similar semantics to the `fit` attribute in [\[SMIL10\]](#biblio-smil10) and the \<meetOrSlice\> parameter on the [`preserveAspectRatio` attribute](https://www.w3.org/TR/SVG11/coords.html#PreserveAspectRatioAttribute) in [\[SVG11\]](#biblio-svg11).

<a id="ref-for-object-size-negotiation"></a>

<a id="ref-for-concrete-object-size②⑦"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Per the [object size negotiation](#object-size-negotiation) algorithm, the [concrete object size](#concrete-object-size) (or, in this case, the size of the content) does not directly scale the object itself - it is merely passed to the object as information about the size of the visible canvas. How to then draw into that size is up to the image format. In particular, raster images always scale to the given size, while SVG uses the given size as the size of the "SVG Viewport" (a term defined by SVG) and then uses the values of several attributes on the root `<svg>` element to determine how to draw itself.

<a id="ref-for-propdef-object-position②"></a>

### <a id="the-object-position"></a>4.6. Positioning Objects: the [object-position](#propdef-object-position) property

| Field               | Definition                                                                                                                                                  |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-object-position"></a>object-position                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-typedef-position⑧"></a>[\<position\>](https://www.w3.org/TR/css-values-4/#typedef-position)                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | 50% 50%                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | replaced elements                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | no                                                                                                                                                          |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | refer to width and height of element itself                                                                                                                 |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-propdef-background-position①"></a>as for [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position)                                       |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | <a id="ref-for-typedef-position⑨"></a>the horizontal component of the [\<position\>](https://www.w3.org/TR/css-values-4/#typedef-position), followed by the vertical component |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | <a id="ref-for-propdef-background-position②"></a>as for [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position)                                       |

<a id="ref-for-propdef-object-position③"></a>

<a id="ref-for-typedef-position①⓪"></a>

<a id="ref-for-propdef-background-position③"></a>

<a id="ref-for-concrete-object-size②⑧"></a>

The [object-position](#propdef-object-position) property determines the alignment of the replaced element inside its box. The [\<position\>](https://www.w3.org/TR/css-values-4/#typedef-position) value type (which is also used for [background-position](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-position)) is defined in [\[CSS-VALUES-3\]](#biblio-css-values-3), and is resolved using the [concrete object size](#concrete-object-size) as the object area and the content box as the positioning area.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: Areas of the box not covered by the replaced element will show the element’s background.

## <a id="image-processing"></a>5. Image Processing

<a id="ref-for-propdef-image-orientation①"></a>

### <a id="the-image-orientation"></a>5.1. Orienting an Image on the Page: the [image-orientation](#propdef-image-orientation) property 

<a id="ref-for-propdef-image-orientation②"></a>

If a picture is taken with a camera turned on its side, or a document isn’t positioned correctly within a scanner, the resultant image may be "sideways" or even upside-down. The [image-orientation](#propdef-image-orientation) property provides a way to apply an "out-of-band" rotation to image source data to correctly orient an image.

| Field               | Definition                                                                                                                                                                                                                                                                     |
|---------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-image-orientation"></a>image-orientation                                                                                                                                                                                                                                           |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-any②"></a><a id="ref-for-angle-value③"></a><a id="ref-for-comb-one①⑦"></a>from-image [\|](https://www.w3.org/TR/css-values-4/#comb-one) none <a id="ref-for-comb-one①⑧"></a>\| \[ [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) [\|\|](https://www.w3.org/TR/css-values-4/#comb-any) flip \] |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | from-image                                                                                                                                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | <a id="ref-for-valdef-image-orientation-angle"></a><a id="ref-for-angle-value④"></a>the specified keyword, or an [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value), rounded and normalized (see text), plus optionally a [flip](#valdef-image-orientation-angle) keyword                                          |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                                                                                                                       |

<strong>This property is <em>optional</em> for implementations.</strong>

<a id="ref-for-elementdef-feimage"></a>

<a id="ref-for-propdef-background-image⑥"></a>

<a id="ref-for-typedef-image①⑥"></a>

This property specifies an orthogonal rotation to be applied to the element’s images before they are used in the document. It applies to content images (e.g. replaced elements and generated content) and image sources referenced by SVG elements (such as <code><a href="https://www.w3.org/TR/filter-effects-1/#elementdef-feimage">feImage</a></code>), as well as to decorative images applied via CSS rules (such as [background-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-background-image) and other [\<image\>](#typedef-image) properties). It does not apply to the rendering of images outside the document, e.g. favicons in the UA’s navigation toolbars or menus, etc.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This property is not intended to specify layout transformations such as arbitrary rotation or flipping the image in the horizontal or vertical direction. (See [\[CSS-TRANSFORMS-1\]](#biblio-css-transforms-1) for a feature designed to do that.) It is also not needed to correctly orient an image when printing in landscape versus portrait orientation, as that rotation is done as part of layout. (See [\[CSS3PAGE\]](#biblio-css3page).) It should only be used to correct incorrectly-oriented images.

Values have the following meanings:

<a id="valdef-image-orientation-none"></a>none

No additional rotation is applied: the image is oriented as encoded.

<a id="valdef-image-orientation-from-image"></a>from-image

<a id="ref-for-valdef-image-orientation-none"></a>

<a id="ref-for-angle-value⑤"></a>

If the image has an orientation specified in its metadata, such as EXIF, this value computes to the angle that the metadata specifies is necessary to correctly orient the image. If necessary, this angle is then rounded and normalized as described above for an [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) value. If there is no orientation specified in its metadata, this value computes to [none](#valdef-image-orientation-none).

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: [§ 2.1.2 Image Metadata](#url-metadata) imposes some restrictions on what metadata can be used.

<a id="ref-for-angle-value⑥"></a>

<a id="valdef-image-orientation-angle"></a>[\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) \|\| flip

<a id="ref-for-angle-value⑦"></a>

Positive [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) values cause the image to be rotated to the right (in a clockwise direction), while negative values cause a rotation to the left. If the <a id="ref-for-angle-value⑧"></a>\<angle\> is omitted, it defaults to 0deg.

<a id="ref-for-valdef-image-orientation-angle①"></a>

If [flip](#valdef-image-orientation-angle) is specified, after rotation the image is flipped horizontally.

<a id="ref-for-valdef-image-orientation-from-image"></a>

This value only applies to content images; decorative images continue to behave as [from-image](#valdef-image-orientation-from-image). This value is deprecated and is optional for implementations except those conforming to [\[CSS-PRINT\]](#biblio-css-print).

<a id="ref-for-valdef-image-orientation-from-image①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This value allows all 8 possible EXIF orientations that [from-image](#valdef-image-orientation-from-image) can produce to be manually reproduced.

<a id="ref-for-angle-value⑨"></a>

The computed value of the property is calculated by rounding the [\<angle\>](https://www.w3.org/TR/css-values-4/#angle-value) to the nearest quarter turn, rounding towards positive infinity when that’s ambiguous, then moduloing the value by 1turn (so that it lies in the half-open range \[0turn, 1turn)).

<a id="ref-for-valdef-image-orientation-none①"></a>

<a id="ref-for-valdef-image-orientation-from-image②"></a>

Values other than [none](#valdef-image-orientation-none) and [from-image](#valdef-image-orientation-from-image) are <em>optional</em> to implement and <em>deprecated</em> in CSS.

All CSS layout and rendering processes use the image <em>after</em> rotation, exactly as if the image were originally encoded in its rotated form. This implies, for example:

- <a id="ref-for-natural-dimensions①⑨"></a>

  The [natural](#natural-dimensions) height and width are derived from the rotated rather than the original image dimensions.

- The height (width) property applies to the vertical (horizontal) dimension of the image, <em>after</em> rotation.

- <a id="ref-for-propdef-cursor③"></a>

  The hotspot coordinates of an image [cursor](https://www.w3.org/TR/css-ui-4/#propdef-cursor) are relative to the image after rotation.

- <a id="ref-for-propdef-border-image②"></a>

  Border images (see [border-image](https://www.w3.org/TR/css-backgrounds-3/#propdef-border-image)) are sliced after rotation.

- <a id="ref-for-propdef-image-orientation③"></a>

  Other transformations, such as those in [\[CSS-TRANSFORMS-1\]](#biblio-css-transforms-1), are applied to the image <em>after</em> [image-orientation](#propdef-image-orientation) is applied.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-5e0592b8"></a> The following example rotates the image 90 degrees clockwise:
>
> ```text
> img.ninety     { image-orientation: 90deg }
> ...
> <img class="ninety" src=...>
> ```
>
> The same effect could be achieved with, for example, an angle of -270deg or 450deg.

<a id="ref-for-valdef-image-orientation-none②"></a>

<a id="ref-for-valdef-image-orientation-from-image③"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: This property previously used [none](#valdef-image-orientation-none) as its initial value. It is believed that using [from-image](#valdef-image-orientation-from-image) as the initial value will produce a generally better user experience, and minimal breakage, but future compat data as UAs attempt to make the change will confirm that. If that is confirmed, then it is likely that this property will be removed from CSS unless use cases other than “correct for incorrect orientation” are raised for its other values.

<a id="ref-for-propdef-image-rendering①"></a>

### <a id="the-image-rendering"></a>5.2. Determining How To Scale an Image: the [image-rendering](#propdef-image-rendering) property

| Field               | Definition                                                                                                                                                                             |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| <strong>Name:&#xA;      </strong> | <a id="propdef-image-rendering"></a>image-rendering                                                                                                                                                     |
| <strong><a href="https://www.w3.org/TR/css-values/#value-defs">Value:</a>&#xA;      </strong> | <a id="ref-for-comb-one①⑨"></a>auto [\|](https://www.w3.org/TR/css-values-4/#comb-one) smooth <a id="ref-for-comb-one②⓪"></a>\| high-quality <a id="ref-for-comb-one②①"></a>\| pixelated <a id="ref-for-comb-one②②"></a>\| crisp-edges |
| <strong><a href="https://www.w3.org/TR/css-cascade/#initial-values">Initial:</a>&#xA;      </strong> | auto                                                                                                                                                                                   |
| <strong><a href="https://www.w3.org/TR/css-cascade/#applies-to">Applies to:</a>&#xA;      </strong> | [all elements](https://www.w3.org/TR/css-pseudo/#generated-content)                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#inherited-property">Inherited:</a>&#xA;      </strong> | yes                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-values/#percentages">Percentages:</a>&#xA;      </strong> | n/a                                                                                                                                                                                    |
| <strong><a href="https://www.w3.org/TR/css-cascade/#computed">Computed value:</a>&#xA;      </strong> | specified keyword                                                                                                                                                                      |
| <strong><a href="https://www.w3.org/TR/cssom/#serializing-css-values">Canonical order:</a>&#xA;      </strong> | per grammar                                                                                                                                                                            |
| <strong><a href="https://www.w3.org/TR/web-animations/#animation-type">Animation type:</a>&#xA;      </strong> | discrete                                                                                                                                                                               |

<a id="ref-for-propdef-image-rendering②"></a>

The [image-rendering](#propdef-image-rendering) property provides a hint to the user-agent about what aspects of an image are most important to preserve when the image is scaled, to aid the user-agent in the choice of an appropriate scaling algorithm. When specified on an element, it applies to all images given in properties for the element, such as background images, list-style images, or the content of replaced elements when they represent an image that must be scaled. The values of the <a id="ref-for-propdef-image-rendering③"></a>image-rendering property are interpreted as follows:

<a id="valdef-image-rendering-auto"></a>auto  
The scaling algorithm is UA-dependent.

<a id="valdef-image-rendering-smooth"></a>smooth  
The image should be scaled with an algorithm that maximizes the appearance of the image. In particular, scaling algorithms that "smooth" colors are acceptable, such as bilinear interpolation. This is intended for images such as photos.

<a id="valdef-image-rendering-high-quality"></a>high-quality  
<a id="ref-for-valdef-image-rendering-smooth"></a>

<a id="ref-for-valdef-image-rendering-high-quality"></a>

Identical to [smooth](#valdef-image-rendering-smooth), but with a preference for higher-quality scaling. If system resources are constrained, images with [high-quality](#valdef-image-rendering-high-quality) should be prioritized over those with any other value, when considering which images to degrade the quality of and to what degree.

<a id="ref-for-valdef-image-rendering-auto"></a>

<a id="ref-for-valdef-image-rendering-high-quality①"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> This value <em>does not</em> prevent the image quality from being degraded when the system resources are constrained. It merely expresses a preference that these images should receive extra scaling resources relative to the [auto](#valdef-image-rendering-auto) images. If all images on the page have [high-quality](#valdef-image-rendering-high-quality) applied, it’s equivalent to all of them having <a id="ref-for-valdef-image-rendering-auto①"></a>auto applied—​they’re all treated the same.
> <a id="ref-for-valdef-image-rendering-high-quality②"></a>
>
> To get the most value out of [high-quality](#valdef-image-rendering-high-quality), only apply it to the most important images on the page.

<a id="valdef-image-rendering-pixelated"></a>pixelated  
The image is scaled in a way that preserves the pixelation of the original as much as possible, but allows minor smoothing as necessary to avoid distorting the image when the target size is not a clean multiple of the original.

<a id="ref-for-nearest-neighbor"></a>

<a id="ref-for-valdef-image-rendering-smooth①"></a>

For each axis independently, first determine the integer multiple of its natural size that puts it closest to the target size and is greater than zero. Scale it to this integer-multiple-size using [nearest neighbor](#nearest-neighbor), then scale it the rest of the way to the target size as for [smooth](#valdef-image-rendering-smooth).

<a id="ref-for-valdef-image-rendering-crisp-edges"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: At integer multiples of the natural size, this gives the same results as [crisp-edges](#valdef-image-rendering-crisp-edges). At non-integer multiples, this usually gives better visual results, even for pixel art, but it does incur a performance penalty due to the "two-step" rendering requirement.

<a id="valdef-image-rendering-crisp-edges"></a>crisp-edges  
The image is scaled in a way that preserves contrast and edges, and which avoids smoothing colors or introducing blur to the image in the process. This is intended for images such as line drawings.

<a id="ref-for-nearest-neighbor①"></a>

The image <em>may</em> be scaled using [nearest neighbor](#nearest-neighbor) or any other UA-chosen algorithm that does not blur edges or blend colors from the source image. It can, however, detect diagonal or curved lines and render them as such (rather than as jagged-looking “giant pixels”).

<a id="ref-for-nearest-neighbor②"></a>

<a id="ref-for-valdef-image-rendering-pixelated"></a>

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: If the new size is not an integer multiple of the original size, the [nearest neighbor](#nearest-neighbor) algorithm can introduce significant “aliasing” bugs; lines that were the same thickness in the original image might be a pixel thinner or thicker in the scaled image depending on where they appear, etc. For most purposes, [pixelated](#valdef-image-rendering-pixelated) will produce a more suitable rendering than <a id="ref-for-nearest-neighbor③"></a>nearest neighbor.

<a id="ref-for-valdef-image-rendering-pixelated①"></a>

<a id="ref-for-propdef-image-rendering④"></a>

Other than the first step of [pixelated](#valdef-image-rendering-pixelated), this property does not dictate any particular scaling algorithm to be used. For example, with [image-rendering: auto](#propdef-image-rendering), a user agent could scale images with bilinear interpolation by default, switch to nearest-neighbor interpolation in high-load situations, and switch to a higher-quality scaling algorithm like Lanczos interpolation for static images that aren’t moving or changing.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-30207ff9"></a> For example, given the following small image:
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/pixel-art-small.gif)
>
> A small pixel-art image.
>
> <a id="ref-for-propdef-image-rendering⑤"></a>
>
> Scaling it up 3x might look like the following, depending on the value of [image-rendering](#propdef-image-rendering):
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/pixel-art-small.gif)
>
> <a id="ref-for-valdef-image-rendering-smooth②"></a>
>
> The image scaled 3x with [smooth](#valdef-image-rendering-smooth)
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/pixel-art-nn.png)
>
> <a id="ref-for-valdef-image-rendering-pixelated②"></a>
>
> The image scaled 3x with [pixelated](#valdef-image-rendering-pixelated)
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/pixel-art-smooth.png)
>
> <a id="ref-for-valdef-image-rendering-crisp-edges①"></a>
>
> <a id="ref-for-nearest-neighbor④"></a>
>
> <a id="ref-for-valdef-image-rendering-pixelated③"></a>
>
> The image scaled 3x with [crisp-edges](#valdef-image-rendering-crisp-edges) interpreted with an edge-preserving algorithm.  
> (Interpreting as [nearest neighbor](#nearest-neighbor) would give the same results as [pixelated](#valdef-image-rendering-pixelated) in this case.)

The <a id="nearest-neighbor"></a>nearest neighbor (or NN) image scaling algorithm treats the source image’s pixels as literal rectangles of color on the source canvas, then colors each destination pixel by choosing one point in the pixel’s area (usually either the center or top-left) and using the color of the corresponding point on the source canvas.

> <strong data-conversion-semantic="note">Note</strong>
>
> When the target size is an integer multiple of the source, this results in “big pixels”, as if you’d merely zoomed in on the source image. When the target size is a non-integer multiple, it still produces crisp pixels entirely out of source-image colors (no blending or blurring), but can produce aliasing quirks where the “pixel grid” can appear somewhat irregular.
>
> <a id="ref-for-nearest-neighbor⑤"></a>
>
> For example, using [nearest neighbor](#nearest-neighbor) to scale up an image by 2.5x will result in each pixel of the source image being used for two or three pixels of the destination image, in an alternating fashion, while scaling an image down to 0.5x will skip every second pixel in the source image.

<a id="ref-for-valdef-image-rendering-pixelated④"></a>

<a id="ref-for-nearest-neighbor⑥"></a>

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-b8e65805"></a> At 3x scaling as in the preceding example, both [pixelated](#valdef-image-rendering-pixelated) and pure [nearest neighbor](#nearest-neighbor) give identical results. At scale ratios between integer multiples, however, they’ll act differently:
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/pixel-art-small.gif)
>
> The same small pixel-art image as before
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/pixel-art-nn.png)
>
> <a id="ref-for-valdef-image-rendering-pixelated⑤"></a>
>
> The image scaled 2.5x with [pixelated](#valdef-image-rendering-pixelated)
>
> ![](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/images/pixel-art-nn-2p5.png)
>
> <a id="ref-for-valdef-image-rendering-crisp-edges②"></a>
>
> <a id="ref-for-nearest-neighbor⑦"></a>
>
> The image scaled 2.5x with [crisp-edges](#valdef-image-rendering-crisp-edges) interpreted as [nearest neighbor](#nearest-neighbor)
>
> <a id="ref-for-valdef-image-rendering-pixelated⑥"></a>
>
> <a id="ref-for-valdef-image-rendering-smooth③"></a>
>
> <a id="ref-for-nearest-neighbor⑧"></a>
>
> The [pixelated](#valdef-image-rendering-pixelated) version maintains the overall <em>look</em> of simply scaling the pixels up, at the cost of <em>slight</em> blurring, though much less blurring than the [smooth](#valdef-image-rendering-smooth) scaling gives. Meanwhile, [nearest neighbor](#nearest-neighbor) avoids introducing any blurring at all, at the cost of aliasing artifacts making the “pixels” look irregularly sized.

<a id="ref-for-valdef-image-rendering-crisp-edges③"></a>

<a id="ref-for-valdef-image-rendering-smooth④"></a>

This property previously accepted the values optimizeSpeed and optimizeQuality. These are now deprecated; a user agent must accept them as valid values but must treat them as having the same behavior as [crisp-edges](#valdef-image-rendering-crisp-edges) and [smooth](#valdef-image-rendering-smooth) respectively, and authors must not use them.

## <a id="interpolation"></a>6. Interpolation

<a id="ref-for-typedef-image①⑦"></a>

Interpolation of [\<image\>](#typedef-image) values is not defined in this level. Implementations must abruptly transition them (at 50% transition progress, like other unsupported interpolations), unless otherwise defined by a future specification.

> <strong data-conversion-semantic="note">Note</strong>
>
> Note: CSS Images Level 4 is expected to define specialized interpolation for gradients, and define that all other images interpolate by cross-fading.

## <a id="serialization"></a>7. Serialization

This section describes the serialization of all new properties and value types introduced in this specification, for the purpose of interfacing with the CSS Object Model [\[CSSOM\]](#biblio-cssom).

To serialize any function defined in this module, serialize it per its individual grammar, in the order its grammar is written in, omitting components when possible without changing the meaning, joining space-separated tokens with a single space, and following each serialized comma with a single space.

> <strong data-conversion-semantic="example">Example</strong>
>
> <a id="example-33854bc0"></a> For example, a gradient specified as:
>
> ```text
> Linear-Gradient( to bottom, red 0%,yellow,black 100px)
> ```
>
> must serialize as:
>
> ```text
> linear-gradient(red, yellow, black 100px)
> ```
## <a id="privacy"></a> Privacy Considerations

This specification introduces no new privacy concerns.

## <a id="security"></a> Security Considerations

<a id="ref-for-natural-dimensions②⓪"></a>

This specification allows rendering of cross-origin images by default, which exposes some information of those images programmatically—​specifically, the [natural dimensions](#natural-dimensions) and resolution of those images.

## <a id="acknowledgments"></a> Acknowledgments

<a id="ref-for-propdef-object-fit⑤"></a>

<a id="ref-for-propdef-object-position④"></a>

<a id="ref-for-propdef-image-orientation④"></a>

Thanks to the Webkit team, Brad Kemper, Brian Manthos, and Alan Gresley for their contributions to the definition of gradients; to Melinda Grant for her work on [object-fit](#propdef-object-fit), [object-position](#propdef-object-position), and [image-orientation](#propdef-image-orientation); and to L. David Baron, Kang-Hao Lu, Leif Arne Storset, Erik Dahlstrom, and Øyvind Stenhaug for their careful review, comments, and corrections.

## <a id="changes"></a> Changes

### <a id="changes-20191010"></a> Changes Since the [10 October 2019 Candidate Recommendation](https://www.w3.org/TR/2019/CR-css-images-3-20191010/)

Significant changes since the [17 December 2020 CRD](https://www.w3.org/TR/2020/CRD-css-images-3-20201217/):

- <a id="ref-for-typedef-linear-color-stop②"></a>

  Remove &#x26;&#x26; which allows reordering from [\<linear-color-stop\>](#typedef-linear-color-stop). ([Issue 8021](https://github.com/w3c/csswg-drafts/pull/8021))

- <a id="ref-for-valdef-image-rendering-pixelated⑦"></a>

  <a id="ref-for-nearest-neighbor⑨"></a>

  Specify [pixelated](#valdef-image-rendering-pixelated) to use smooth scaling from a [nearest neighbor](#nearest-neighbor) scale to the nearest pixel multiple, in order to avoid distortion. ([Issue 5837](https://github.com/w3c/csswg-drafts/issues/5837))

- <a id="ref-for-nearest-neighbor①⓪"></a>

  <a id="ref-for-valdef-image-rendering-crisp-edges④"></a>

  Explicitly allow [nearest neighbor](#nearest-neighbor) as the implementation for [crisp-edges](#valdef-image-rendering-crisp-edges), since this is what implementations currently do. ([Issue 6038](https://github.com/w3c/csswg-drafts/issues/6038))

- Various editorial / typo fixes in grammar productions.

Significant changes since the [10 October 2019 CR](https://www.w3.org/TR/2019/CR-css-images-3-20191010/):

- Define handling of degenerate aspect ratios ([Issue 4572](https://github.com/w3c/csswg-drafts/issues/4572))

- Define that layout-affecting metadata occurring after the image data should be ignored ([Issue 5165](https://github.com/w3c/csswg-drafts/issues/5165))

- Explicitly allow dithering in gradients ([Issue 4793](https://github.com/w3c/csswg-drafts/issues/4793))

- <a id="ref-for-propdef-image-orientation⑤"></a>

  Define that [image-orientation](#propdef-image-orientation) applies to both decorative and content images ([Issue 5245](https://github.com/w3c/csswg-drafts/issues/5245))

- <a id="ref-for-natural-dimensions②①"></a>

  <a id="ref-for-intrinsic-size"></a>

  Rename “intrinsic dimensions” to [natural dimensions](#natural-dimensions) to avoid confusion with [intrinsic sizes](https://www.w3.org/TR/css-sizing-3/#intrinsic-size). ([Issue 4961](https://github.com/w3c/csswg-drafts/issues/4961))

### <a id="changes-20120407"></a> Changes Since the [17 April 2012 Candidate Recommendation](https://www.w3.org/TR/2012/CR-css3-images-20120417/)

Major changes include:

- The image() notation has been deferred to Level 4.

- The image-resolution property has been deferred to Level 4.

- <a id="ref-for-propdef-image-orientation⑥"></a>

  The [image-orientation](#propdef-image-orientation) property has been marked as deprecated, optional, and at-risk. Additionally:

  - <a id="ref-for-valdef-image-orientation-from-image④"></a>

    <a id="ref-for-valdef-image-orientation-none③"></a>

    Added the [from-image](#valdef-image-orientation-from-image) and [none](#valdef-image-orientation-none) keywords

  - Added the flip values

  - [Swapped to "mod then round" ordering.](https://github.com/w3c/csswg-drafts/issues/1206)

- <a id="ref-for-propdef-image-rendering⑥"></a>

  Added the [image-rendering](#propdef-image-rendering) property.

- <a id="ref-for-resolution-value"></a>

  Moved the [\<resolution\>](https://www.w3.org/TR/css-values-4/#resolution-value) type to [\[CSS-VALUES-3\]](#biblio-css-values-3).

- Better defined the handling of invalid and partially-loaded images.

- <a id="ref-for-typedef-image①⑧"></a>

  Defined the general computed form of [\<image\>](#typedef-image).

- Defined concept and handling of URLs that are ambiguous between being images or element references.

- <a id="ref-for-funcdef-linear-gradient⑥"></a>

  Defined that the unit can be omitted for zero angles in [linear-gradient()](#funcdef-linear-gradient) due to compat.

- Slightly clarified handling of degenerate repeating radial gradients.

- <a id="ref-for-color-transition-hint①⓪"></a>

  <a id="ref-for-typedef-color-stop-list③"></a>

  Added [transition hints](#color-transition-hint) to [\<color-stop-list\>](#typedef-color-stop-list), and editorially rewrote section on gradient color stops to better accommodate the new prose.

- Added "Canonical Order" and "Animation Type" to all property definition tables.

- <a id="ref-for-typedef-image①⑨"></a>

  Defined interpolation and serialization of [\<image\>](#typedef-image) values.

- Various minor clarifications.

A [Disposition of Comments](https://drafts.csswg.org/issues?spec=css-images-3&doc=cr-2012) is available.

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

- [ambiguous image URL](#css-ambiguous-image-url), in § 2.1.1
- [\<angle\>](#valdef-image-orientation-angle), in § 5.1
- [auto](#valdef-image-rendering-auto), in § 5.2
- [circle](#valdef-radial-shape-circle), in § 3.2.1
- [closest-corner](#valdef-radial-extent-closest-corner), in § 3.2.1
- [closest-side](#valdef-radial-extent-closest-side), in § 3.2.1
- [color stop](#color-stop), in § 3.4
- [\<color-stop-list\>](#typedef-color-stop-list), in § 3.4.1
- [color stop list](#color-stop-list), in § 3.4.1
- [color transition hint](#color-transition-hint), in § 3.4
- [computed \<image\>](#computed-image), in § 2
- [concrete object size](#concrete-object-size), in § 4.1
- [constraint rectangle](#constraint-rectangle), in § 4.3.2
- [contain](#valdef-object-fit-contain), in § 4.5
- [contain constraint](#contain-constraint), in § 4.3.2
- [cover](#valdef-object-fit-cover), in § 4.5
- [cover constraint](#cover-constraint), in § 4.3.2
- [crisp-edges](#valdef-image-rendering-crisp-edges), in § 5.2
- [default object size](#default-object-size), in § 4.1
- [default sizing algorithm](#default-sizing-algorithm), in § 4.3.1
- [ellipse](#valdef-radial-shape-ellipse), in § 3.2.1
- [ending point](#ending-point), in § 3
- [ending shape](#ending-shape), in § 3.2
- [farthest-corner](#valdef-radial-extent-farthest-corner), in § 3.2.1
- [farthest-side](#valdef-radial-extent-farthest-side), in § 3.2.1
- [fill](#valdef-object-fit-fill), in § 4.5
- [flip](#valdef-image-orientation-angle), in § 5.1
- [from-image](#valdef-image-orientation-from-image), in § 5.1
- [\<gradient\>](#typedef-gradient), in § 3
- [gradient-average-color](#gradient-average-color), in § 3.3
- [gradient box](#gradient-box), in § 3
- [gradient center](#radial-gradient-gradient-center), in § 3.2.1
- [gradient function](#gradient-function), in § 3
- [gradient line](#gradient-line), in § 3
- [high-quality](#valdef-image-rendering-high-quality), in § 5.2
- [\<image\>](#typedef-image), in § 2
- [image-orientation](#propdef-image-orientation), in § 5.1
- [image-rendering](#propdef-image-rendering), in § 5.2
- [invalid image](#invalid-image), in § 2
- [\<length \[0,∞\]\>](#valdef-radial-size-length-0), in § 3.2.1
- [\<length-percentage \[0,∞\]\>{2}](#valdef-radial-size-length-percentage-0-2), in § 3.2.1
- [\<linear-color-hint\>](#typedef-linear-color-hint), in § 3.4.1
- [\<linear-color-stop\>](#typedef-linear-color-stop), in § 3.4.1
- [linear-gradient()](#funcdef-linear-gradient), in § 3.1.1
- [\<linear-gradient-syntax\>](#typedef-linear-gradient-syntax), in § 3.1.1
- [loading image](#loading-image), in § 2
- [natural aspect ratio](#natural-aspect-ratio), in § 4.1
- [natural dimension](#natural-dimensions), in § 4.1
- [natural height](#natural-height), in § 4.1
- [natural size](#natural-size), in § 4.1
- [natural width](#natural-width), in § 4.1
- [nearest neighbor](#nearest-neighbor), in § 5.2
- none
  - [value for image-orientation](#valdef-image-orientation-none), in § 5.1
  - [value for object-fit](#valdef-object-fit-none), in § 4.5
- [object](#objects), in § 4
- [object-fit](#propdef-object-fit), in § 4.5
- [object-position](#propdef-object-position), in § 4.6
- [objects](#objects), in § 4
- [object size negotiation](#object-size-negotiation), in § 4.2
- [pixelated](#valdef-image-rendering-pixelated), in § 5.2
- [\<position\>](#valdef-radial-gradient-position), in § 3.2.1
- [\<radial-extent\>](#typedef-radial-extent), in § 3.2.1
- [radial-gradient()](#funcdef-radial-gradient), in § 3.2
- [\<radial-gradient-syntax\>](#typedef-radial-gradient-syntax), in § 3.2.1
- \<radial-shape\>
  - [(type)](#typedef-radial-shape), in § 3.2.1
  - [value for radial-gradient(), repeating-radial-gradient()](#valdef-radial-gradient-radial-shape), in § 3.2.1
- \<radial-size\>
  - [(type)](#typedef-radial-size), in § 3.2.1
  - [value for radial-gradient(), repeating-radial-gradient()](#valdef-radial-gradient-radial-size), in § 3.2.1
- [repeating-linear-gradient()](#funcdef-repeating-linear-gradient), in § 3.3
- [repeating-radial-gradient()](#funcdef-repeating-radial-gradient), in § 3.3
- [scale-down](#valdef-object-fit-scale-down), in § 4.5
- [\<side-or-corner\>](#typedef-side-or-corner), in § 3.1.1
- [smooth](#valdef-image-rendering-smooth), in § 5.2
- [specified size](#specified-size), in § 4.1
- [starting point](#starting-point), in § 3
- [transition hint](#color-transition-hint), in § 3.4
- [valid image](#invalid-image), in § 2

### <a id="index-defined-elsewhere"></a>Terms defined by reference

- \[CSS-CASCADE-5\] defines the following terms:
  - <a id="8c8e51b4"></a>computed value
  - <a id="762bad34"></a>initial
  - <a id="d5e08d9c"></a>specified value
  - <a id="1a2b1083"></a>used value
- \[CSS-COLOR-4\] defines the following terms:
  - <a id="329ecc08"></a>\<color\>
  - <a id="96e27c16"></a>transparent
- \[CSS-CONTENT-3\] defines the following terms:
  - <a id="f3e8378c"></a>content
- \[CSS-LISTS-3\] defines the following terms:
  - <a id="237f9b49"></a>list-style-image
  - <a id="d065f190"></a>list-style-type
  - <a id="32dc2011"></a>none
- \[CSS-MASKING-1\] defines the following terms:
  - <a id="24d00f64"></a>mask
  - <a id="2bb33078"></a>mask-image
- \[CSS-SIZING-3\] defines the following terms:
  - <a id="5ad01cca"></a>height
  - <a id="3ade8b07"></a>intrinsic size
  - <a id="49731d1d"></a>width
- \[CSS-SIZING-4\] defines the following terms:
  - <a id="b03f7c8f"></a>preferred aspect ratio
- \[CSS-UI-4\] defines the following terms:
  - <a id="5b085f76"></a>cursor
- \[CSS-VALUES-4\] defines the following terms:
  - <a id="c297b070"></a>\#
  - <a id="8cd4f032"></a>,
  - <a id="d7e1d67b"></a>\<angle\>
  - <a id="4fd7e54f"></a>\<length-percentage\>
  - <a id="98ddb9b0"></a>\<length\>
  - <a id="40c6f879"></a>\<position\>
  - <a id="9108b09d"></a>\<resolution\>
  - <a id="699488a8"></a>\<url\>
  - <a id="d4441b24"></a>?
  - <a id="8a110a7b"></a>css-wide keywords
  - <a id="ebcca398"></a>degenerate ratio
  - <a id="3fa441fa"></a>url()
  - <a id="8cbc2b3b"></a>{a}
  - <a id="4eb9d37e"></a>\|
  - <a id="a0336d84"></a>\|\|
- \[CSS3BG\] defines the following terms:
  - <a id="db6870d5"></a>background
  - <a id="5ced56d0"></a>background-image
  - <a id="f2249e38"></a>background-position
  - <a id="e316431d"></a>background-repeat
  - <a id="dbdacf0d"></a>background-size
  - <a id="bb65d94f"></a>border-image
  - <a id="4ab170b5"></a>border-image-repeat
  - <a id="369934cd"></a>round
- \[FILTER-EFFECTS-1\] defines the following terms:
  - <a id="11c51e4c"></a>feimage
- \[HTML\] defines the following terms:
  - <a id="87fcd40c"></a>iframe
  - <a id="f0811ff8"></a>img
  - <a id="b996b941"></a>src
- \[SELECTORS-4\] defines the following terms:
  - <a id="0459c0bd"></a>:target

## <a id="references"></a>References

### <a id="normative"></a>Normative References

<a id="biblio-css-cascade-5"></a>\[CSS-CASCADE-5\]  
Elika Etemad; Miriam Suzanne; Tab Atkins Jr.. [CSS Cascading and Inheritance Level 5](https://www.w3.org/TR/css-cascade-5/). 13 January 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-cascade-5&#x2F;](https://www.w3.org/TR/css-cascade-5/)

<a id="biblio-css-color-4"></a>\[CSS-COLOR-4\]  
Tab Atkins Jr.; Chris Lilley; Lea Verou. [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/). 1 November 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-color-4&#x2F;](https://www.w3.org/TR/css-color-4/)

<a id="biblio-css-content-3"></a>\[CSS-CONTENT-3\]  
Elika Etemad; Dave Cramer. [CSS Generated Content Module Level 3](https://www.w3.org/TR/css-content-3/). 2 August 2019. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-content-3&#x2F;](https://www.w3.org/TR/css-content-3/)

<a id="biblio-css-lists-3"></a>\[CSS-LISTS-3\]  
Elika Etemad; Tab Atkins Jr.. [CSS Lists and Counters Module Level 3](https://www.w3.org/TR/css-lists-3/). 17 November 2020. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-lists-3&#x2F;](https://www.w3.org/TR/css-lists-3/)

<a id="biblio-css-masking-1"></a>\[CSS-MASKING-1\]  
Dirk Schulze; Brian Birtles; Tab Atkins Jr.. [CSS Masking Module Level 1](https://www.w3.org/TR/css-masking-1/). 5 August 2021. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-masking-1&#x2F;](https://www.w3.org/TR/css-masking-1/)

<a id="biblio-css-print"></a>\[CSS-PRINT\]  
Elika Etemad; Melinda Grant. [CSS Print Profile](https://www.w3.org/TR/css-print/). 14 March 2013. NOTE. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-print&#x2F;](https://www.w3.org/TR/css-print/)

<a id="biblio-css-sizing-3"></a>\[CSS-SIZING-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Box Sizing Module Level 3](https://www.w3.org/TR/css-sizing-3/). 17 December 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-3&#x2F;](https://www.w3.org/TR/css-sizing-3/)

<a id="biblio-css-sizing-4"></a>\[CSS-SIZING-4\]  
Tab Atkins Jr.; Elika Etemad; Jen Simmons. [CSS Box Sizing Module Level 4](https://www.w3.org/TR/css-sizing-4/). 20 May 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-sizing-4&#x2F;](https://www.w3.org/TR/css-sizing-4/)

<a id="biblio-css-ui-4"></a>\[CSS-UI-4\]  
Florian Rivoal. [CSS Basic User Interface Module Level 4](https://www.w3.org/TR/css-ui-4/). 16 March 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-ui-4&#x2F;](https://www.w3.org/TR/css-ui-4/)

<a id="biblio-css-values-3"></a>\[CSS-VALUES-3\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/). 1 December 2022. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-3&#x2F;](https://www.w3.org/TR/css-values-3/)

<a id="biblio-css-values-4"></a>\[CSS-VALUES-4\]  
Tab Atkins Jr.; Elika Etemad. [CSS Values and Units Module Level 4](https://www.w3.org/TR/css-values-4/). 27 October 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-values-4&#x2F;](https://www.w3.org/TR/css-values-4/)

<a id="biblio-css2"></a>\[CSS2\]  
Bert Bos; et al. [Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification](https://www.w3.org/TR/CSS21/). 7 June 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;CSS21&#x2F;](https://www.w3.org/TR/CSS21/)

<a id="biblio-css3bg"></a>\[CSS3BG\]  
Bert Bos; Elika Etemad; Brad Kemper. [CSS Backgrounds and Borders Module Level 3](https://www.w3.org/TR/css-backgrounds-3/). 14 February 2023. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-backgrounds-3&#x2F;](https://www.w3.org/TR/css-backgrounds-3/)

<a id="biblio-filter-effects-1"></a>\[FILTER-EFFECTS-1\]  
Dirk Schulze; Dean Jackson. [Filter Effects Module Level 1](https://www.w3.org/TR/filter-effects-1/). 18 December 2018. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;filter-effects-1&#x2F;](https://www.w3.org/TR/filter-effects-1/)

<a id="biblio-html"></a>\[HTML\]  
Anne van Kesteren; et al. [HTML Standard](https://html.spec.whatwg.org/multipage/). Living Standard. URL: [https&#x3A;&#x2F;&#x2F;html&#x2E;spec&#x2E;whatwg&#x2E;org&#x2F;multipage&#x2F;](https://html.spec.whatwg.org/multipage/)

<a id="biblio-rfc2119"></a>\[RFC2119\]  
S. Bradner. [Key words for use in RFCs to Indicate Requirement Levels](https://datatracker.ietf.org/doc/html/rfc2119). March 1997. Best Current Practice. URL: [https&#x3A;&#x2F;&#x2F;datatracker&#x2E;ietf&#x2E;org&#x2F;doc&#x2F;html&#x2F;rfc2119](https://datatracker.ietf.org/doc/html/rfc2119)

### <a id="informative"></a>Informative References

<a id="biblio-css-transforms-1"></a>\[CSS-TRANSFORMS-1\]  
Simon Fraser; et al. [CSS Transforms Module Level 1](https://www.w3.org/TR/css-transforms-1/). 14 February 2019. CR. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-transforms-1&#x2F;](https://www.w3.org/TR/css-transforms-1/)

<a id="biblio-css3page"></a>\[CSS3PAGE\]  
Elika Etemad. [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/). 14 September 2023. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;css-page-3&#x2F;](https://www.w3.org/TR/css-page-3/)

<a id="biblio-cssom"></a>\[CSSOM\]  
Daniel Glazman; Emilio Cobos Álvarez. [CSS Object Model (CSSOM)](https://www.w3.org/TR/cssom-1/). 26 August 2021. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;cssom-1&#x2F;](https://www.w3.org/TR/cssom-1/)

<a id="biblio-selectors-4"></a>\[SELECTORS-4\]  
Elika Etemad; Tab Atkins Jr.. [Selectors Level 4](https://www.w3.org/TR/selectors-4/). 11 November 2022. WD. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;selectors-4&#x2F;](https://www.w3.org/TR/selectors-4/)

<a id="biblio-smil10"></a>\[SMIL10\]  
Philipp Hoschka. [Synchronized Multimedia Integration Language (SMIL) 1.0 Specification](https://www.w3.org/TR/1998/REC-smil-19980615/). 15 June 1998. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;1998&#x2F;REC-smil-19980615&#x2F;](https://www.w3.org/TR/1998/REC-smil-19980615/)

<a id="biblio-svg11"></a>\[SVG11\]  
Erik Dahlström; et al. [Scalable Vector Graphics (SVG) 1.1 (Second Edition)](https://www.w3.org/TR/SVG11/). 16 August 2011. REC. URL: [https&#x3A;&#x2F;&#x2F;www&#x2E;w3&#x2E;org&#x2F;TR&#x2F;SVG11&#x2F;](https://www.w3.org/TR/SVG11/)

## <a id="property-index"></a>Property Index

| Name                | Value                                                      | Initial    | Applies to        | Inh. | %ages                                       | Anim­ation type             | Canonical order                                                                  | Com­puted value                                                                                            |
|---------------------|------------------------------------------------------------|------------|-------------------|------|---------------------------------------------|----------------------------|----------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------|
| <strong><span><a id="ref-for-propdef-image-orientation⑦"></a></span><a href="#propdef-image-orientation">image-orientation</a>&#xA;      </strong> | from-image \| none \| \[ \<angle\> \|\| flip \]            | from-image | all elements      | yes  | n/a                                         | discrete                   | per grammar                                                                      | the specified keyword, or an \<angle\>, rounded and normalized (see text), plus optionally a flip keyword |
| <strong><span><a id="ref-for-propdef-image-rendering⑦"></a></span><a href="#propdef-image-rendering">image-rendering</a>&#xA;      </strong> | auto \| smooth \| high-quality \| pixelated \| crisp-edges | auto       | all elements      | yes  | n/a                                         | discrete                   | per grammar                                                                      | specified keyword                                                                                         |
| <strong><span><a id="ref-for-propdef-object-fit⑥"></a></span><a href="#propdef-object-fit">object-fit</a>&#xA;      </strong> | fill \| contain \| cover \| none \| scale-down             | fill       | replaced elements | no   | n/a                                         | discrete                   | per grammar                                                                      | specified keyword                                                                                         |
| <strong><span><a id="ref-for-propdef-object-position⑤"></a></span><a href="#propdef-object-position">object-position</a>&#xA;      </strong> | \<position\>                                               | 50% 50%    | replaced elements | no   | refer to width and height of element itself | as for background-position | the horizontal component of the \<position\>, followed by the vertical component | as for background-position                                                                                |

## <a id="issues-index"></a>Issues Index

> <strong data-conversion-semantic="issue">Issue</strong>
>
> Add a visual example of a color hint being used. [↵](#issue-7a141efc)

CanIUse

<b>Support:</b>Android Browser4.4+Baidu Browser13.18+Blackberry Browser10+Chrome26+Chrome for Android119+Edge12+Firefox16+Firefox for Android119+IE10+IE Mobile10+KaiOS Browser2.5+Opera12.1+Opera MiniNoneOpera Mobile12.1+QQ Browser13.1+Safari6.1+Safari on iOS7.0+Samsung Internet4+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=css-repeating-gradients) as of 2023-12-13

CanIUse

<b>Support:</b>Android Browser119+Baidu Browser13.18+Blackberry BrowserNoneChrome81+Chrome for Android119+Edge81+Firefox26+Firefox for Android119+IENoneIE MobileNoneKaiOS Browser2.5+Opera68+Opera MiniNoneOpera Mobile73+QQ BrowserNoneSafari13.1+Safari on iOS14.0+Samsung Internet13.0+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=css-image-orientation) as of 2023-12-13

CanIUse

<b>Support:</b>Android Browser119+Baidu Browser13.18+Blackberry BrowserNoneChrome41+Chrome for Android119+Edge79+Firefox65+Firefox for Android119+IENoneIE MobileNoneKaiOS Browser2.5+Opera28+Opera MiniNoneOpera Mobile73+QQ Browser13.1+Safari10+Safari on iOS10.0+Samsung Internet4+UC Browser for Android15.5+

Source: [caniuse.com](https://caniuse.com/#feat=css-crisp-edges) as of 2023-12-13
