Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Media types](https://www.w3.org/TR/2011/REC-CSS2-20110607/media.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Media types

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/media.html

Snapshot SHA-256: 3a324a17037934085cc012d6730d86fdfd9beae5f02a0df9f97dd49696c98863

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- 1 complex or multi-paragraph tables are structured Markdown row/cell transcriptions with explicit header/data roles and row/column spans; no raw HTML tables remain.
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.

---

<a id="q7.0"></a>

# 7 Media types

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

<a id="media-intro"></a>

## 7.1 Introduction to media types

One of the most important features of style sheets is that they specify how a document is to be presented on different media: on the screen, on paper, with a speech synthesizer, with a braille device, etc.

Certain CSS properties are only designed for certain media (e.g., the ['page-break-before'](css2--page.html--984664616bba.md#propdef-page-break-before) property only applies to paged media). On occasion, however, style sheets for different media types may share a property, but require different values for that property. For example, the ['font-size'](css2--fonts.html--d52fc14f36c2.md#propdef-font-size) property is useful both for screen and print media. The two media types are different enough to require different values for the common property; a document will typically need a larger font on a computer screen than on paper. Therefore, it is necessary to express that a style sheet, or a section of a style sheet, applies to certain media types.

<a id="media-sheets"></a>

## 7.2 Specifying media-dependent style sheets

There are currently two ways to specify media dependencies for style sheets:

- <a id="x1"></a>

  <a id="x0"></a>

  Specify the target medium from a style sheet with the @media or @import at-rules.

  > <strong data-conversion-semantic="example">Example</strong>
  >
  > Example(s):
  >
  > ```text
  > 
  > @import url("fancyfonts.css") screen;
  > @media print {
  >   /* style sheet for print goes here */
  > }
  > ```
- Specify the target medium within the document language. For example, in HTML 4 ([\[HTML4\]](css2--refs.html--f208d881d0b7.md#ref-HTML4)), the "media" attribute on the LINK element specifies the target media of an external style sheet:
  ```text
  
  <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN">
  <HTML>
     <HEAD>
        <TITLE>Link to a target medium</TITLE>
        <LINK REL="stylesheet" TYPE="text/css" 
  	 MEDIA="print, handheld" HREF="foo.css">
     </HEAD>
     <BODY>
        <P>The body...
     </BODY>
  </HTML>
  ```
The [@import](css2--cascade.html--c7aff33e6f0d.md#at-import) rule is defined in the [chapter on the cascade](css2--cascade.html--c7aff33e6f0d.md).

<a id="at-media-rule"></a>

### 7.2.1 The @media rule

<a id="x2"></a>

<a id="x3"></a>

An @media rule specifies the target [media types](#media-types) (separated by commas) of a set of [statements](css2--syndata.html--02e71c159e14.md#tokenization) (delimited by curly braces). Invalid statements must be ignored per [4.1.7 "Rule sets, declaration blocks, and selectors"](css2--syndata.html--02e71c159e14.md#rule-sets) and [4.2 "Rules for handling parsing errors."](css2--syndata.html--02e71c159e14.md#parsing-errors) The @media construct allows style sheet rules for various media in the same style sheet:

> <strong data-conversion-semantic="example">Example</strong>
>
> ```text
> 
>   @media print {
>     body { font-size: 10pt }
>   }
>   @media screen {
>     body { font-size: 13px }
>   }
>   @media screen, print {
>     body { line-height: 1.2 }
>   }
> ```
Style rules outside of @media rules apply to all media types that the style sheet applies to. At-rules inside @media are invalid in CSS2.1.

<a id="media-types"></a>

## 7.3 Recognized media types

The names chosen for CSS media types reflect target devices for which the relevant properties make sense. In the following list of CSS media types the names of media types are normative, but the descriptions are informative. Likewise, the "Media" field in the description of each property is informative.

<strong>all</strong>  
Suitable for all devices.

<strong>braille</strong>  
Intended for braille tactile feedback devices.

<strong>embossed</strong>  
Intended for paged braille printers.

<strong>handheld</strong>  
Intended for handheld devices (typically small screen, limited bandwidth).

<strong>print</strong>  
Intended for paged material and for documents viewed on screen in print preview mode. Please consult the section on [paged media](css2--page.html--984664616bba.md) for information about formatting issues that are specific to paged media.

<strong>projection</strong>  
Intended for projected presentations, for example projectors. Please consult the section on [paged media](css2--page.html--984664616bba.md) for information about formatting issues that are specific to paged media.

<strong>screen</strong>  
Intended primarily for color computer screens.

<strong>speech</strong>  
Intended for speech synthesizers. Note: CSS2 had a similar media type called 'aural' for this purpose. See the appendix on [aural style sheets](css2--aural.html--2915ca92d45a.md) for details.

<strong>tty</strong>  
Intended for media using a fixed-pitch character grid (such as teletypes, terminals, or portable devices with limited display capabilities). Authors should not use [pixel units](css2--syndata.html--02e71c159e14.md#length-units) with the "tty" media type.

<strong>tv</strong>  
Intended for television-type devices (low resolution, color, limited-scrollability screens, sound available).

Media type names are case-insensitive.

Media types are mutually exclusive in the sense that a user agent can only support one media type when rendering a document. However, user agents may use different media types on different canvases. For example, a document may (simultaneously) be shown in 'screen' mode on one canvas and 'print' mode on another canvas.

Note that a multimodal media type is still only one media type. The 'tv' media type, for example, is a multimodal media type that renders both visually and aurally to a single canvas.

@media and @import rules with unknown media types (that are nonetheless valid identifiers) are treated as if the unknown media types are not present. If an @media/@import rule contains a malformed media type (not an identifier) then the statement is invalid.

> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note:</strong> Media Queries supercedes this
error handling.</em>

> <strong data-conversion-semantic="example">Example</strong>
>
> Example(s):
>
> For example, in the following snippet, the rule on the P element applies in 'screen' mode (even though the '3D' media type is not known).
>
> ```text
> 
> @media screen, 3D {
>   P { color: green; }
> }
> ```
> <strong data-conversion-semantic="note">Note</strong>
>
> <em><strong>Note.</strong>
Future updates of CSS may extend the list of media types. Authors
should not rely on media type names that are not yet defined
by a CSS specification.
</em>

<a id="media-groups"></a>

### 7.3.1 Media groups

<em>This section is informative, not normative.</em>

<a id="x4"></a>

Each CSS property definition specifies which media types the property applies to. Since properties generally apply to several media types, the "Applies to media" section of each property definition lists media groups rather than individual media types. Each property applies to all media types in the media groups listed in its definition.

CSS 2.1 defines the following media groups:

- <a id="paged-media-group"></a>

  <a id="continuous-media-group"></a>

  <strong>continuous</strong> or <strong>paged</strong>.

- <a id="tactile-media-group"></a>

  <a id="speech-media-group"></a>

  <a id="audio-media-group"></a>

  <a id="visual-media-group"></a>

  <strong>visual</strong>, <strong>audio</strong>, <strong>speech</strong>, or <strong>tactile</strong>.

- <a id="bitmap-media-group"></a>

  <a id="grid-media-group"></a>

  <strong>grid</strong> (for character grid devices), or <strong>bitmap</strong>.

- <a id="static-media-group"></a>

  <a id="interactive-media-group"></a>

  <strong>interactive</strong> (for devices that allow user interaction), or <strong>static</strong> (for those that do not).

- <a id="all-media-group"></a>

  <strong>all</strong> (includes all media types)

The following table shows the relationships between media groups and media types:

<strong>Table 1 — structured row/cell transcription</strong>

Relationship between media groups and media types

<strong>Row 1</strong>

<strong>Column 1 (header cell):</strong>

Media Types

<strong>Column 2 (header cell; column span 4):</strong>

Media Groups

<strong>Row 2</strong>

<strong>Column 1 (header cell):</strong>

 

<strong>Column 2 (header cell):</strong>

continuous/paged

<strong>Column 3 (header cell):</strong>

visual/audio/speech/tactile

<strong>Column 4 (header cell):</strong>

grid/bitmap

<strong>Column 5 (header cell):</strong>

interactive/static

<strong>Row 3</strong>

<strong>Column 1 (header cell):</strong>

braille

<strong>Column 2 (data cell):</strong>

continuous

<strong>Column 3 (data cell):</strong>

tactile

<strong>Column 4 (data cell):</strong>

grid

<strong>Column 5 (data cell):</strong>

both

<strong>Row 4</strong>

<strong>Column 1 (header cell):</strong>

embossed

<strong>Column 2 (data cell):</strong>

paged

<strong>Column 3 (data cell):</strong>

tactile

<strong>Column 4 (data cell):</strong>

grid

<strong>Column 5 (data cell):</strong>

static

<strong>Row 5</strong>

<strong>Column 1 (header cell):</strong>

handheld

<strong>Column 2 (data cell):</strong>

both

<strong>Column 3 (data cell):</strong>

visual, audio, speech

<strong>Column 4 (data cell):</strong>

both

<strong>Column 5 (data cell):</strong>

both

<strong>Row 6</strong>

<strong>Column 1 (header cell):</strong>

print

<strong>Column 2 (data cell):</strong>

paged

<strong>Column 3 (data cell):</strong>

visual

<strong>Column 4 (data cell):</strong>

bitmap

<strong>Column 5 (data cell):</strong>

static

<strong>Row 7</strong>

<strong>Column 1 (header cell):</strong>

projection

<strong>Column 2 (data cell):</strong>

paged

<strong>Column 3 (data cell):</strong>

visual

<strong>Column 4 (data cell):</strong>

bitmap

<strong>Column 5 (data cell):</strong>

interactive

<strong>Row 8</strong>

<strong>Column 1 (header cell):</strong>

screen

<strong>Column 2 (data cell):</strong>

continuous

<strong>Column 3 (data cell):</strong>

visual, audio

<strong>Column 4 (data cell):</strong>

bitmap

<strong>Column 5 (data cell):</strong>

both

<strong>Row 9</strong>

<strong>Column 1 (header cell):</strong>

speech

<strong>Column 2 (data cell):</strong>

continuous

<strong>Column 3 (data cell):</strong>

speech

<strong>Column 4 (data cell):</strong>

N/A

<strong>Column 5 (data cell):</strong>

both

<strong>Row 10</strong>

<strong>Column 1 (header cell):</strong>

tty

<strong>Column 2 (data cell):</strong>

continuous

<strong>Column 3 (data cell):</strong>

visual

<strong>Column 4 (data cell):</strong>

grid

<strong>Column 5 (data cell):</strong>

both

<strong>Row 11</strong>

<strong>Column 1 (header cell):</strong>

tv

<strong>Column 2 (data cell):</strong>

both

<strong>Column 3 (data cell):</strong>

visual, audio

<strong>Column 4 (data cell):</strong>

bitmap

<strong>Column 5 (data cell):</strong>

both
